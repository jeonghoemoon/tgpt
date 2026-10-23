use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Confirm;
use std::{
    fs,
    io::{self, BufRead},
    path::PathBuf,
};

use crate::{
    config::Config,
    llm::{self, Message},
    ui,
};

// ── System prompts ────────────────────────────────────────────────────────────

const DEFAULT_SYSTEM: &str =
    "You are tgpt, an AI assistant embedded in the terminal. \
     Be concise, direct, and technically precise. \
     Format code with markdown fences when appropriate. \
     Avoid unnecessary preamble.";

const COMMAND_SYSTEM: &str =
    "You are a shell command expert. \
     When given a task description, respond with ONLY the shell command(s) needed. \
     Do NOT include explanation, markdown fences, or any text other than the raw command. \
     Use cross-platform commands when possible. \
     If multiple commands are needed, chain them with && or put them on separate lines.";

const REVIEW_SYSTEM: &str =
    "You are an expert code reviewer. \
     Analyse the provided code for: bugs, security issues, performance problems, \
     style violations, and suggest improvements. \
     Be specific, cite line numbers where possible, and be constructive.";

// ── One-shot query ────────────────────────────────────────────────────────────

pub async fn run_oneshot(cfg: &Config, query: &str) -> Result<()> {
    let provider = cfg.default_provider.clone();
    let backend = llm::build_backend(cfg, &provider)?;
    ui::print_provider_header(backend.name(), backend.model());

    let system = cfg
        .system_prompt
        .clone()
        .unwrap_or_else(|| DEFAULT_SYSTEM.to_string());

    let messages = vec![Message::system(system), Message::user(query)];

    if cfg.stream {
        ui::print_ai_prefix(backend.name());
        let mut cb = |chunk: llm::StreamChunk| {
            if !chunk.done {
                ui::print_assistant_chunk(&chunk.text);
            }
        };
        backend.stream(&messages, &mut cb).await?;
        ui::print_newline();
    } else {
        let mut spinner = ui::Spinner::new("Thinking…");
        // Fake spinner tick (non-blocking; actual wait is inside complete())
        spinner.tick();
        let reply = backend.complete(&messages).await?;
        spinner.clear();
        ui::print_ai_prefix(backend.name());
        println!("{}", reply);
    }

    Ok(())
}

// ── Interactive chat ──────────────────────────────────────────────────────────

pub async fn run_chat_session(cfg: &Config, provider: &str) -> Result<()> {
    let backend = llm::build_backend(cfg, provider)?;
    ui::print_provider_header(backend.name(), backend.model());
    ui::print_separator();
    println!(
        "{}",
        "Type your message and press Enter. /help for commands, Ctrl+C to exit.".dimmed()
    );

    let system = cfg
        .system_prompt
        .clone()
        .unwrap_or_else(|| DEFAULT_SYSTEM.to_string());

    let mut history: Vec<Message> = vec![Message::system(system)];

    let stdin = io::stdin();

    loop {
        ui::print_user_prompt();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        let input = line.trim().to_string();

        if input.is_empty() {
            continue;
        }

        // Handle slash commands
        match input.as_str() {
            "/quit" | "/exit" | "/q" => {
                println!("{}", "\nGoodbye! 👋".cyan());
                break;
            }
            "/clear" => {
                history.truncate(1); // keep system message
                println!("{}", "✓ Conversation cleared".green());
                continue;
            }
            "/help" => {
                ui::print_chat_help();
                continue;
            }
            "/info" => {
                println!(
                    "\n{} {} messages in history ({} user turns)",
                    "ℹ".blue().bold(),
                    history.len(),
                    history.iter().filter(|m| matches!(m.role, crate::llm::Role::User)).count()
                );
                continue;
            }
            cmd if cmd.starts_with("/save") => {
                let path = cmd
                    .split_whitespace()
                    .nth(1)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| {
                        let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
                        PathBuf::from(format!("tgpt_chat_{}.json", ts))
                    });
                save_history(&history, &path)?;
                println!("{} Saved to {}", "✓".green().bold(), path.display());
                continue;
            }
            _ => {}
        }

        history.push(Message::user(&input));

        ui::print_ai_prefix(backend.name());

        if cfg.stream {
            let mut reply = String::new();
            let mut cb = |chunk: llm::StreamChunk| {
                if !chunk.done {
                    reply.push_str(&chunk.text);
                    ui::print_assistant_chunk(&chunk.text);
                }
            };
            backend.stream(&history, &mut cb).await?;
            ui::print_newline();
            history.push(Message::assistant(reply));
        } else {
            let reply = backend.complete(&history).await?;
            println!("{}", reply);
            history.push(Message::assistant(&reply));
        }
    }

    Ok(())
}

// ── Command generation ────────────────────────────────────────────────────────

pub async fn run_command_generation(cfg: &Config, query: &str, auto_run: bool) -> Result<()> {
    let provider = cfg.default_provider.clone();
    let backend = llm::build_backend(cfg, &provider)?;
    ui::print_provider_header(backend.name(), backend.model());

    let messages = vec![
        Message::system(COMMAND_SYSTEM),
        Message::user(query),
    ];

    println!("{}", "Generating command…".dimmed());
    let cmd = backend.complete(&messages).await?;
    let cmd = cmd.trim().to_string();

    ui::print_command_box(&cmd);

    // Now ask AI to briefly explain the command
    let explain_messages = vec![
        Message::system(
            "Explain what the following shell command does in one or two sentences. \
             Be very brief and direct.",
        ),
        Message::user(&cmd),
    ];
    let explanation = backend.complete(&explain_messages).await?;
    println!("{} {}", "→".dimmed(), explanation.trim().dimmed());
    println!();

    // Ask to run
    if auto_run {
        run_shell_command(&cmd)?;
    } else {
        let confirmed = Confirm::new()
            .with_prompt("Execute this command?")
            .default(false)
            .interact()?;

        if confirmed {
            run_shell_command(&cmd)?;
        } else {
            ui::print_info("Command not executed. Copied to your display above.");
        }
    }

    Ok(())
}

fn run_shell_command(cmd: &str) -> Result<()> {
    println!("{}", format!("$ {}", cmd).yellow());
    println!("{}", "─".repeat(60).dimmed());

    #[cfg(target_os = "windows")]
    let output = std::process::Command::new("cmd")
        .args(["/C", cmd])
        .status()
        .context("Failed to run command")?;

    #[cfg(not(target_os = "windows"))]
    let output = std::process::Command::new("sh")
        .args(["-c", cmd])
        .status()
        .context("Failed to run command")?;

    if output.success() {
        ui::print_success("Command completed successfully");
    } else {
        ui::print_warning(&format!("Command exited with code {:?}", output.code()));
    }

    Ok(())
}

// ── Code review ───────────────────────────────────────────────────────────────

pub async fn run_code_review(cfg: &Config, file_path: &str, provider: &str) -> Result<()> {
    let code = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    let backend = llm::build_backend(cfg, provider)?;
    ui::print_provider_header(backend.name(), backend.model());
    println!("{}", format!("Reviewing: {}", file_path).dimmed());
    ui::print_separator();

    let user_msg = format!(
        "Please review the following code from file `{}`:\n\n```\n{}\n```",
        file_path, code
    );

    let messages = vec![
        Message::system(REVIEW_SYSTEM),
        Message::user(user_msg),
    ];

    if cfg.stream {
        let mut cb = |chunk: llm::StreamChunk| {
            if !chunk.done {
                ui::print_assistant_chunk(&chunk.text);
            }
        };
        backend.stream(&messages, &mut cb).await?;
        ui::print_newline();
    } else {
        let reply = backend.complete(&messages).await?;
        println!("{}", reply);
    }

    Ok(())
}

// ── Session save ──────────────────────────────────────────────────────────────

fn save_history(history: &[Message], path: &PathBuf) -> Result<()> {
    let json = serde_json::to_string_pretty(history)?;
    fs::write(path, json)?;
    Ok(())
}
