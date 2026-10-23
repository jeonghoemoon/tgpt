# tgpt 🤖

> AI-powered terminal assistant built with Rust — chat, generate shell commands, and review code, all without leaving your terminal.

[![Rust](https://img.shields.io/badge/rust-1.83+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/tgpt.svg)](https://crates.io/crates/tgpt)

```
  ████████╗ ██████╗ ██████╗ ████████╗
     ██╔══╝██╔════╝ ██╔══██╗╚══██╔══╝
     ██║   ██║  ███╗██████╔╝   ██║   
     ██║   ██║   ██║██╔═══╝    ██║   
     ██║   ╚██████╔╝██║        ██║   
     ╚═╝    ╚═════╝ ╚═╝        ╚═╝   
```

## ✨ Features

| Feature | Description |
|---|---|
| 💬 **One-shot queries** | Ask anything directly: `tgpt "explain async/await in Rust"` |
| 🗣️ **Interactive chat** | Multi-turn conversation with full history: `tgpt chat` |
| ⚡ **Command generation** | Natural language → shell command + optional auto-run: `tgpt run "find large files"` |
| 🔍 **Code review** | AI-powered review of any source file: `tgpt review main.rs` |
| 🔌 **Multi-provider** | OpenAI, Anthropic (Claude), and local Ollama models |
| 🚀 **Streaming output** | Real-time token streaming for a snappy feel |
| 🔒 **Secure config** | API keys stored in your OS config directory |

## 🚀 Installation

### From crates.io
```bash
cargo install tgpt
```

### From source
```bash
git clone https://github.com/your-username/tgpt
cd tgpt
cargo build --release
# Binary is at: target/release/tgpt
```

### Pre-built binaries
Download from the [Releases](https://github.com/your-username/tgpt/releases) page.

## ⚡ Quick Start

```bash
# 1. Set your API key
tgpt config set-key openai sk-...

# 2. Ask something
tgpt "What is the best sorting algorithm for nearly-sorted data?"

# 3. Start a chat session
tgpt chat

# 4. Generate a shell command
tgpt run "delete all .log files older than 7 days"

# 5. Review your code
tgpt review src/main.rs
```

## 📖 Usage

### One-shot query
```bash
tgpt "your question here"
tgpt "explain this regex: ^[a-z]{3,}$"
```

### Interactive chat (`tgpt chat`)
```bash
tgpt chat
tgpt chat --provider anthropic
```

**In-chat commands:**
| Command | Description |
|---|---|
| `/clear` | Clear conversation history |
| `/save [file]` | Save conversation to JSON |
| `/info` | Show session info |
| `/help` | Show available commands |
| `/quit` | Exit |

### Command generation (`tgpt run`)
```bash
tgpt run "compress all PNG files in current directory"
tgpt run "find which process is using port 8080"
tgpt run --yes "list the 10 largest files"   # auto-execute
```

tgpt will:
1. Generate the shell command
2. Explain what it does
3. Ask for confirmation before running (unless `--yes`)

### Code review (`tgpt review`)
```bash
tgpt review src/main.rs
tgpt review app.py --focus "security vulnerabilities"
tgpt review Cargo.toml --provider anthropic
```

### Configuration (`tgpt config`)
```bash
# Set API keys
tgpt config set-key openai sk-...
tgpt config set-key anthropic sk-ant-...

# Change models
tgpt config set-model openai gpt-4o-mini
tgpt config set-model anthropic claude-3-haiku-20240307

# Set default provider
tgpt config set-provider anthropic

# Use local Ollama
tgpt config set-provider ollama
tgpt config set-model ollama llama3.2
tgpt config set-url ollama http://localhost:11434

# Show current config
tgpt config show

# Open config file in editor
tgpt config edit
```

## 🔌 Supported Providers

| Provider | Models | Notes |
|---|---|---|
| **OpenAI** | `gpt-4o`, `gpt-4o-mini`, `gpt-4-turbo`, `gpt-3.5-turbo`, ... | Requires API key from [platform.openai.com](https://platform.openai.com) |
| **Anthropic** | `claude-3-5-sonnet-20241022`, `claude-3-haiku-20240307`, ... | Requires API key from [console.anthropic.com](https://console.anthropic.com) |
| **Ollama** | `llama3.2`, `mistral`, `deepseek-coder`, ... | Free, runs locally — install [Ollama](https://ollama.ai) first |

> **Tip:** Any OpenAI-compatible API (LM Studio, Together.ai, Groq, etc.) works with `tgpt config set-url openai <your-url>`.

## ⚙️ Configuration File

Config is stored at:
- **Linux/macOS:** `~/.config/tgpt/config.json`
- **Windows:** `%APPDATA%\tgpt\config.json`

```json
{
  "default_provider": "openai",
  "stream": true,
  "system_prompt": null,
  "providers": {
    "openai": {
      "api_key": "sk-...",
      "model": "gpt-4o",
      "base_url": "https://api.openai.com/v1",
      "max_tokens": 4096
    },
    "anthropic": {
      "api_key": "sk-ant-...",
      "model": "claude-3-5-sonnet-20241022",
      "base_url": "https://api.anthropic.com",
      "max_tokens": 4096
    },
    "ollama": {
      "model": "llama3.2",
      "base_url": "http://localhost:11434",
      "max_tokens": 4096
    }
  }
}
```

## 🔐 Security Note

API keys are stored in plaintext in the config file. Make sure to set appropriate file permissions:
```bash
chmod 600 ~/.config/tgpt/config.json  # Linux/macOS
```
Alternatively, you can use environment variables: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`.

## 🏗️ Architecture

```
src/
├── main.rs          # Entry point, CLI dispatch
├── cli.rs           # Clap CLI definitions
├── config.rs        # Config load/save, config subcommands
├── session.rs       # Chat, one-shot, command-gen, review logic
├── ui.rs            # Terminal rendering (colors, spinners, boxes)
└── llm/
    ├── mod.rs       # LlmBackend trait, Message types, factory
    ├── openai.rs    # OpenAI backend (streaming SSE)
    ├── anthropic.rs # Anthropic backend (streaming SSE)
    └── ollama.rs    # Ollama backend (streaming NDJSON)
```

## 🤝 Contributing

Contributions are welcome! Here are some ideas:

- [ ] Shell history context (pass recent commands to AI)
- [ ] Plugin system for custom commands
- [ ] TUI mode with `ratatui`
- [ ] Token usage tracking and cost estimation
- [ ] Conversation export to Markdown
- [ ] `--model` flag to override per-call
- [ ] Image input support (vision models)

```bash
git clone https://github.com/your-username/tgpt
cd tgpt
cargo test
cargo clippy
cargo fmt
```

## 📄 License

MIT — see [LICENSE](LICENSE) for details.

---

<p align="center">Built with ❤️ and 🦀 Rust</p>
