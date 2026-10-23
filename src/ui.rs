use colored::Colorize;
use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::io::{self, Write};

// ── Banner ────────────────────────────────────────────────────────────────────

pub fn print_banner() {
    println!(
        "{}",
        r#"
  ████████╗ ██████╗ ██████╗ ████████╗
     ██╔══╝██╔════╝ ██╔══██╗╚══██╔══╝
     ██║   ██║  ███╗██████╔╝   ██║   
     ██║   ██║   ██║██╔═══╝    ██║   
     ██║   ╚██████╔╝██║        ██║   
     ╚═╝    ╚═════╝ ╚═╝        ╚═╝   
"#
        .cyan()
        .bold()
    );
    println!(
        "  {} — AI-powered terminal assistant\n",
        "tgpt v0.1.0".cyan()
    );
}

// ── Spinner ───────────────────────────────────────────────────────────────────

pub struct Spinner {
    frames: &'static [&'static str],
    idx: usize,
    label: String,
}

impl Spinner {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            frames: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            idx: 0,
            label: label.into(),
        }
    }

    pub fn tick(&mut self) {
        let frame = self.frames[self.idx % self.frames.len()].cyan().to_string();
        print!("\r{} {}", frame, self.label.dimmed());
        let _ = io::stdout().flush();
        self.idx += 1;
    }

    pub fn clear(&self) {
        let _ = io::stdout().execute(cursor::MoveToColumn(0));
        let _ = io::stdout().execute(Clear(ClearType::CurrentLine));
    }
}

// ── Headers ───────────────────────────────────────────────────────────────────

pub fn print_provider_header(provider: &str, model: &str) {
    println!(
        "\n{} {} {}",
        "▶".cyan(),
        provider.cyan().bold(),
        format!("({})", model).dimmed()
    );
}

pub fn print_separator() {
    println!("{}", "─".repeat(60).dimmed());
}

pub fn print_user_prompt() {
    print!("\n{} ", "You ›".green().bold());
    let _ = io::stdout().flush();
}

pub fn print_ai_prefix(name: &str) {
    println!("\n{}", format!("AI ({}) ›", name).cyan().bold());
}

pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

#[allow(dead_code)]
pub fn print_error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

// ── Command display ───────────────────────────────────────────────────────────

pub fn print_command_box(cmd: &str) {
    println!();
    println!("{}", "┌─ Generated Command ─────────────────────────────────".cyan());
    println!("{}", format!("│  {}", cmd).yellow().bold());
    println!("{}", "└─────────────────────────────────────────────────────".cyan());
}

// ── Chat history display ──────────────────────────────────────────────────────

pub fn print_assistant_chunk(text: &str) {
    print!("{}", text);
    let _ = io::stdout().flush();
}

pub fn print_newline() {
    println!();
}

// ── Help ──────────────────────────────────────────────────────────────────────

pub fn print_chat_help() {
    println!();
    println!("{}", "Chat Commands:".bold().underline());
    println!("  {}  Clear conversation history", "/clear".cyan());
    println!("  {}   Save conversation to file", "/save".cyan());
    println!("  {}   Show token/session info", "/info".cyan());
    println!("  {}   Show this help", "/help".cyan());
    println!("  {}   Exit chat", "/quit  or Ctrl+C".cyan());
    println!();
}
