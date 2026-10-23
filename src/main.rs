mod cli;
mod config;
mod llm;
mod session;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine provider override from global flag
    let provider_override = cli.provider.as_deref();

    match cli.command {
        // ── tgpt chat ──────────────────────────────────────────────────────
        Some(Commands::Chat { provider }) => {
            let cfg = config::Config::load()?;
            let provider_name = provider
                .as_deref()
                .or(provider_override)
                .unwrap_or(&cfg.default_provider)
                .to_string();

            ui::print_banner();
            session::run_chat_session(&cfg, &provider_name).await?;
        }

        // ── tgpt run <task> ────────────────────────────────────────────────
        Some(Commands::Run { query, yes }) => {
            let cfg = config::Config::load()?;
            let query_str = query.join(" ");
            if query_str.trim().is_empty() {
                eprintln!("{}", "Error: Please provide a task description.".red());
                std::process::exit(1);
            }
            let provider_name = provider_override.unwrap_or(&cfg.default_provider);
            session::run_command_generation(&cfg, &query_str, yes, provider_name).await?;
        }

        // ── tgpt review <file> ─────────────────────────────────────────────
        Some(Commands::Review { file, focus, provider }) => {
            let cfg = config::Config::load()?;

            let file_path = if let Some(f) = focus {
                let _ = f;
                file
            } else {
                file
            };

            let provider_name = provider
                .as_deref()
                .or(provider_override)
                .unwrap_or(&cfg.default_provider)
                .to_string();

            session::run_code_review(&cfg, &file_path, &provider_name).await?;
        }

        // ── tgpt config ────────────────────────────────────────────────────
        Some(Commands::Config(args)) => {
            config::handle_config_command(args.command)?;
        }

        // ── tgpt <query> (no subcommand) ───────────────────────────────────
        None => {
            if cli.query.is_empty() {
                print_welcome();
            } else {
                let cfg = config::Config::load()?;
                let query = cli.query.join(" ");
                let provider_name = provider_override.unwrap_or(&cfg.default_provider);
                session::run_oneshot(&cfg, &query, provider_name).await?;
            }
        }
    }

    Ok(())
}

fn print_welcome() {
    ui::print_banner();
    println!("{}", "Usage:".bold());
    println!("  tgpt {}                  Ask a one-shot question", "\"your question\"".yellow());
    println!("  tgpt {}                      Start interactive chat", "chat".cyan());
    println!("  tgpt {} {}     Generate a shell command", "run".cyan(), "\"list big files\"".yellow());
    println!("  tgpt {} {}       Review a source file", "review".cyan(), "main.rs".yellow());
    println!("  tgpt {}                    Manage API keys & settings", "config show".cyan());
    println!();
    println!("{}", "Quick Start:".bold());
    println!("  1. Set your API key:");
    println!("     {}", "tgpt config set-key gemini AIzaSy...".yellow());
    println!("  2. Start chatting:");
    println!("     {}", "tgpt \"What is the meaning of life?\"".yellow());
    println!();
    println!("Run {} for full usage.", "tgpt --help".cyan());
}
