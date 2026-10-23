pub mod anthropic;
pub mod ollama;
pub mod openai;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ── Message types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: content.into() }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self { role: Role::User, content: content.into() }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: content.into() }
    }
}

// ── Streaming chunk ───────────────────────────────────────────────────────────

pub struct StreamChunk {
    pub text: String,
    pub done: bool,
}

// ── LLM backend trait ─────────────────────────────────────────────────────────

#[async_trait]
pub trait LlmBackend: Send + Sync {
    /// Send messages and return the complete response text
    async fn complete(&self, messages: &[Message]) -> Result<String>;

    /// Send messages and stream the response, calling `on_chunk` for each token
    async fn stream(
        &self,
        messages: &[Message],
        on_chunk: &mut (dyn FnMut(StreamChunk) + Send),
    ) -> Result<String>;

    /// Provider name for display
    fn name(&self) -> &str;

    /// Model name for display
    fn model(&self) -> &str;
}

// ── Factory ───────────────────────────────────────────────────────────────────

use crate::config::Config;

pub fn build_backend(cfg: &Config, provider: &str) -> Result<Box<dyn LlmBackend>> {
    let pcfg = cfg.provider(provider)?;
    match provider {
        "openai" => Ok(Box::new(openai::OpenAiBackend::new(pcfg.clone())?)),
        "anthropic" => Ok(Box::new(anthropic::AnthropicBackend::new(pcfg.clone())?)),
        "ollama" => Ok(Box::new(ollama::OllamaBackend::new(pcfg.clone()))),
        other => anyhow::bail!(
            "Unknown provider '{}'. Supported providers: openai, anthropic, ollama",
            other
        ),
    }
}
