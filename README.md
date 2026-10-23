<div align="center">

# 🤖 tgpt

### Lightning-fast, terminal-native AI assistant built in Rust 🦀

[![Crates.io](https://img.shields.io/crates/v/tgpt.svg?style=flat-square&color=orange)](https://crates.io/crates/tgpt)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![CI Status](https://img.shields.io/github/actions/workflow/status/jeonghoemoon/tgpt/ci.yml?branch=main&style=flat-square)](https://github.com/jeonghoemoon/tgpt/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.85+-ea6341.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)

```
  ████████╗ ██████╗ ██████╗ ████████╗
     ██╔══╝██╔════╝ ██╔══██╗╚══██╔══╝
     ██║   ██║  ███╗██████╔╝   ██║   
     ██║   ██║   ██║██╔═══╝    ██║   
     ██║   ╚██████╔╝██║        ██║   
     ╚═╝    ╚═════╝ ╚═╝        ╚═╝   
```

*Chat with LLMs, generate and execute shell commands, and perform instant code reviews — right inside your terminal with zero bloat.*

[Features](#-key-features) • [Installation](#-installation) • [Quick Start](#-quick-start) • [Usage](#-usage-guide) • [Configuration](#-configuration) • [Contributing](#-contributing)

---

</div>

## 💡 Why `tgpt`?

Most AI command-line tools rely on heavy runtime environments (like Node.js or Python) and lag behind when streaming responses. 

`tgpt` is crafted with **pure Rust**:
- ⚡ **Zero runtime dependency**: Single compiled native binary (~3MB).
- 🏎️ **Ultra-low latency**: Instant startup (<10ms) and real-time SSE token streaming.
- 🔌 **Provider-agnostic**: Seamlessly switch between **OpenAI**, **Anthropic (Claude)**, **Google Gemini**, and **local Ollama** without changing your workflow.
- 🛡️ **Safe execution**: AI command generator always clarifies what the command does and asks for confirmation before executing.

---

## ✨ Key Features

| Feature | Description |
|---|---|
| 💬 **One-Shot Queries** | Instant answers without leaving your command line: `tgpt "How do mutexes work in Rust?"` |
| 🗣️ **Interactive Chat** | Multi-turn conversational REPL with session context and slash commands: `tgpt chat` |
| ⚡ **Smart Command Runner** | Turn natural language into runnable shell commands: `tgpt run "find files > 500MB"` |
| 🔍 **Source Code Review** | Pass any source code file for bug discovery, security audits, and optimizations: `tgpt review main.rs` |
| 🌐 **Multi-Provider Support** | Google Gemini (Gemini 3 Flash), OpenAI (GPT-4o), Anthropic (Claude 3.5 Sonnet), and Local Ollama (Llama 3.2, DeepSeek Coder, Mistral, Qwen) |
| 🌊 **Token Streaming** | Smooth, real-time token rendering directly onto your terminal |
| 🔒 **OS-Native Key Storage** | Secure local configuration using your platform's configuration standard |

---

## 🚀 Installation

### 1. Via Cargo (Recommended)

```bash
cargo install tgpt
```

### 2. From Source

Ensure you have Rust installed (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`):

```bash
git clone https://github.com/jeonghoemoon/tgpt.git
cd tgpt
cargo build --release
```
The optimized executable will be located at `target/release/tgpt` (or `tgpt.exe` on Windows).

### 3. Pre-built Binaries

Download pre-compiled standalone binaries for Linux, macOS, and Windows directly from the [GitHub Releases](https://github.com/jeonghoemoon/tgpt/releases) page.

---

## ⚡ Quick Start

```bash
# 1. Set your preferred API key (Gemini, OpenAI, or Anthropic)
tgpt config set-key gemini AIzaSy...

# 2. Ask a quick question
tgpt "Explain TCP handshakes in simple terms"

# 3. Generate and safely run a command
tgpt run "archive all .tar.gz files created this week"

# 4. Review a code file
tgpt review ./src/main.rs --focus "potential race conditions"
```

---

## 📖 Usage Guide

### 1. One-Shot Query
Directly pass your question or prompt as arguments:

```bash
tgpt "Convert 172.16.0.0/12 to CIDR IP range"
tgpt "Write an awk one-liner to calculate the sum of column 3 in a CSV"
```

Override provider on the fly with `--provider` (`-p`):
```bash
tgpt -p gemini "Explain quantum computing in one sentence"
tgpt -p anthropic "Refactor this function to be functional and idempotent"
```

### 2. Interactive Chat (`tgpt chat`)
Launch an interactive session with memory:

```bash
tgpt chat
tgpt chat --provider gemini
```

**Built-in Chat Commands:**
- `/clear` — Wipe current conversation history and restart context
- `/save [filename]` — Export current dialogue to a formatted JSON session
- `/info` — View current token counts and message statistics
- `/help` — Display list of commands
- `/quit` or `Ctrl+C` — Exit session

### 3. Natural Language Command Generation (`tgpt run`)
Translate human intent into shell commands:

```bash
# Preview command, see AI explanation, and confirm before execution
tgpt run "find all listening ports and kill PID on port 3000"

# Auto-execute directly with --yes flag (useful for scripting)
tgpt run --yes "git clean -fd"
```

### 4. Code Review (`tgpt review`)
Audit any file in your project:

```bash
tgpt review src/server.rs
tgpt review package.json --focus "security audit on outdated dependencies"
```

---

## ⚙️ Configuration

`tgpt` stores its configuration in the platform-standard config directory:
- **Linux/macOS**: `~/.config/tgpt/config.json`
- **Windows**: `%APPDATA%\tgpt\config.json`

### CLI Configuration Commands

```bash
# Set API keys
tgpt config set-key gemini <YOUR_KEY>
tgpt config set-key openai <YOUR_KEY>
tgpt config set-key anthropic <YOUR_KEY>

# Set default model
tgpt config set-model gemini gemini-2.5-flash
tgpt config set-model openai gpt-4o-mini
tgpt config set-model anthropic claude-3-5-sonnet-20241022

# Change default provider
tgpt config set-provider gemini

# Configure Local Ollama
tgpt config set-provider ollama
tgpt config set-model ollama llama3.2
tgpt config set-url ollama http://localhost:11434

# Display current configuration
tgpt config show

# Open config in your default system editor ($EDITOR)
tgpt config edit
```

---

## 🔌 Supported Providers

| Provider | Default Model | Configuration Required |
|---|---|---|
| **Google Gemini** | `gemini-3-flash-preview` | `tgpt config set-key gemini <KEY>` or `export GEMINI_API_KEY="..."` |
| **OpenAI** | `gpt-4o` | `tgpt config set-key openai <KEY>` or `export OPENAI_API_KEY="..."` |
| **Anthropic** | `claude-3-5-sonnet-20241022` | `tgpt config set-key anthropic <KEY>` or `export ANTHROPIC_API_KEY="..."` |
| **Ollama** (Local) | `llama3.2` | Runs 100% locally with zero API key requirement (`http://localhost:11434`) |

### 🦙 Running Local AI Models with Ollama (100% Free & Offline)

You can run any open-source model completely offline without an API key:

1. Install [Ollama](https://ollama.ai)
2. Pull your model of choice:
   ```bash
   ollama run llama3.2          # Meta Llama 3.2 (fast & lightweight)
   ollama run deepseek-coder    # DeepSeek Coder (optimized for programming)
   ollama run mistral           # Mistral 7B (high performance reasoning)
   ollama run qwen2.5-coder     # Qwen 2.5 Coder
   ```
3. Point `tgpt` to the model:
   ```bash
   # Switch to DeepSeek for coding
   tgpt config set-provider ollama
   tgpt config set-model ollama deepseek-coder

   # Or switch to Mistral
   tgpt config set-model ollama mistral
   ```

> **Pro Tip**: Any OpenAI-compatible inference provider (such as **Groq**, **DeepSeek**, **Together AI**, or **vLLM**) can also be used by overriding the endpoint URL:
> ```bash
> tgpt config set-url openai https://api.groq.com/openai/v1
> ```

---

## 🗺️ Roadmap

- [x] Multi-provider support (OpenAI, Anthropic, Gemini, Ollama)
- [x] Streaming SSE output for instant feedback
- [x] Interactive conversational chat mode
- [x] Shell command generator with confirmation prompt
- [x] File code reviewer
- [ ] TUI interface powered by `ratatui`
- [ ] Contextual git diff analysis (`tgpt diff`)
- [ ] Shell autocomplete generation (`bash`, `zsh`, `fish`)
- [ ] Local vector store for repository-level codebase RAG

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'feat: add some amazing feature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

---

## 📝 License

Distributed under the MIT License. See [`LICENSE`](LICENSE) for more information.

<div align="center">
  <sub>Built with ❤️ and 🦀 Rust</sub>
</div>
