use clap::{Args, Parser, Subcommand};

/// 🤖 tgpt — AI-powered terminal assistant
///
/// Chat with AI, generate shell commands, and review code — all from your terminal.
#[derive(Parser, Debug)]
#[command(
    name = "tgpt",
    version,
    about = "🤖 AI-powered terminal assistant",
    long_about = "Chat with AI, generate runnable shell commands, and review code — all from your terminal.\n\nSupports OpenAI, Anthropic (Claude), Google Gemini, and local Ollama models.",
    arg_required_else_help = false
)]
pub struct Cli {
    /// Ask a one-shot question (no subcommand needed)
    #[arg(value_name = "QUERY", num_args = 0..)]
    pub query: Vec<String>,

    /// Override the AI provider for this invocation
    #[arg(short, long, value_name = "PROVIDER", global = true)]
    pub provider: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start an interactive multi-turn chat session
    Chat {
        /// AI provider to use (openai | anthropic | gemini | ollama)
        #[arg(short, long)]
        provider: Option<String>,
    },

    /// Generate a shell command from natural language, then optionally run it
    #[command(alias = "cmd")]
    Run {
        /// What you want to do (natural language)
        #[arg(value_name = "TASK", num_args = 1..)]
        query: Vec<String>,

        /// Automatically execute the generated command without asking
        #[arg(short, long)]
        yes: bool,
    },

    /// Review or explain a source code file
    Review {
        /// Path to the file to review
        #[arg(value_name = "FILE")]
        file: String,

        /// Specific question or focus for the review
        #[arg(short, long)]
        focus: Option<String>,

        /// AI provider to use
        #[arg(short, long)]
        provider: Option<String>,
    },

    /// Manage configuration (API keys, models, defaults)
    Config(ConfigArgs),
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Set an API key for a provider
    SetKey {
        /// Provider name (openai | anthropic | gemini | ollama)
        provider: String,
        /// API key value
        key: String,
    },

    /// Set the default model for a provider
    SetModel {
        /// Provider name
        provider: String,
        /// Model name (e.g. gpt-4o, claude-3-5-sonnet-20241022, gemini-2.5-flash)
        model: String,
    },

    /// Set the default provider
    SetProvider {
        /// Provider name (openai | anthropic | gemini | ollama)
        provider: String,
    },

    /// Set a custom base URL for a provider (useful for OpenAI-compatible APIs)
    SetUrl {
        /// Provider name
        provider: String,
        /// Base URL
        url: String,
    },

    /// Print current configuration
    Show,

    /// Open the config file in your default editor
    Edit,

    /// Reset configuration to defaults
    Reset,
}
