use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::ProviderConfig;
use super::{LlmBackend, Message, Role, StreamChunk};

// ── Ollama request/response ───────────────────────────────────────────────────

#[derive(Serialize)]
struct OllamaMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OllamaStreamChunk {
    message: Option<OllamaStreamMessage>,
    done: bool,
}

#[derive(Deserialize)]
struct OllamaStreamMessage {
    content: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaStreamMessage,
}

// ── Backend ───────────────────────────────────────────────────────────────────

pub struct OllamaBackend {
    client: Client,
    cfg: ProviderConfig,
}

impl OllamaBackend {
    pub fn new(cfg: ProviderConfig) -> Self {
        Self { client: Client::new(), cfg }
    }

    fn build_messages<'a>(&self, messages: &'a [Message]) -> Vec<OllamaMessage<'a>> {
        messages
            .iter()
            .map(|m| OllamaMessage {
                role: match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
                content: &m.content,
            })
            .collect()
    }
}

#[async_trait]
impl LlmBackend for OllamaBackend {
    fn name(&self) -> &str { "ollama" }
    fn model(&self) -> &str { &self.cfg.model }

    async fn complete(&self, messages: &[Message]) -> Result<String> {
        let url = format!("{}/api/chat", self.cfg.base_url);
        let body = json!({
            "model": self.cfg.model,
            "messages": self.build_messages(messages),
            "stream": false,
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to connect to Ollama at {}", self.cfg.base_url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("Ollama error {}: {}", status, text);
        }

        let data: OllamaResponse = resp.json().await.context("Failed to parse Ollama response")?;
        Ok(data.message.content)
    }

    async fn stream(
        &self,
        messages: &[Message],
        on_chunk: &mut (dyn FnMut(StreamChunk) + Send),
    ) -> Result<String> {
        let url = format!("{}/api/chat", self.cfg.base_url);
        let body = json!({
            "model": self.cfg.model,
            "messages": self.build_messages(messages),
            "stream": true,
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to connect to Ollama at {}", self.cfg.base_url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("Ollama error {}: {}", status, text);
        }

        let mut stream = resp.bytes_stream();
        let mut full_text = String::new();
        let mut buf = String::new();

        while let Some(item) = stream.next().await {
            let chunk = item.context("Stream read error")?;
            buf.push_str(std::str::from_utf8(&chunk).unwrap_or(""));

            loop {
                if let Some(nl) = buf.find('\n') {
                    let line = buf[..nl].trim().to_string();
                    buf.drain(..=nl);

                    if line.is_empty() { continue; }

                    if let Ok(event) = serde_json::from_str::<OllamaStreamChunk>(&line) {
                        if let Some(msg) = event.message {
                            full_text.push_str(&msg.content);
                            on_chunk(StreamChunk { text: msg.content, done: false });
                        }
                        if event.done {
                            on_chunk(StreamChunk { text: String::new(), done: true });
                            return Ok(full_text);
                        }
                    }
                } else {
                    break;
                }
            }
        }

        Ok(full_text)
    }
}
