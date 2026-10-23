use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::ProviderConfig;
use super::{LlmBackend, Message, Role, StreamChunk};

// ── Anthropic request/response ────────────────────────────────────────────────

#[derive(Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    kind: String,
    delta: Option<AnthropicDelta>,
}

#[derive(Deserialize)]
struct AnthropicDelta {
    text: Option<String>,
}

// ── Backend ───────────────────────────────────────────────────────────────────

pub struct AnthropicBackend {
    client: Client,
    cfg: ProviderConfig,
    api_key: String,
}

impl AnthropicBackend {
    pub fn new(cfg: ProviderConfig) -> Result<Self> {
        let api_key = cfg
            .api_key
            .clone()
            .filter(|k| !k.is_empty())
            .context("Anthropic API key not set")
            .or_else(|_| {
                std::env::var("ANTHROPIC_API_KEY")
                    .context("Anthropic API key not found in config or ANTHROPIC_API_KEY env var")
            })?;

        Ok(Self { client: Client::new(), cfg, api_key })
    }

    fn build_body<'a>(&self, messages: &'a [Message], stream: bool) -> serde_json::Value {
        let system_msg = messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.as_str())
            .unwrap_or("You are a helpful assistant.");

        let filtered: Vec<AnthropicMessage> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| AnthropicMessage {
                role: match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::System => unreachable!(),
                },
                content: &m.content,
            })
            .collect();

        json!({
            "model": self.cfg.model,
            "system": system_msg,
            "messages": filtered,
            "max_tokens": self.cfg.max_tokens,
            "stream": stream,
        })
    }
}

#[async_trait]
impl LlmBackend for AnthropicBackend {
    fn name(&self) -> &str { "anthropic" }
    fn model(&self) -> &str { &self.cfg.model }

    async fn complete(&self, messages: &[Message]) -> Result<String> {
        let url = format!("{}/v1/messages", self.cfg.base_url);
        let body = self.build_body(messages, false);

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .context("Failed to connect to Anthropic API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("Anthropic API error {}: {}", status, text);
        }

        let data: AnthropicResponse = resp.json().await?;
        data.content
            .into_iter()
            .find(|c| c.kind == "text")
            .and_then(|c| c.text)
            .context("Empty response from Anthropic")
    }

    async fn stream(
        &self,
        messages: &[Message],
        on_chunk: &mut (dyn FnMut(StreamChunk) + Send),
    ) -> Result<String> {
        let url = format!("{}/v1/messages", self.cfg.base_url);
        let body = self.build_body(messages, true);

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .context("Failed to connect to Anthropic API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("Anthropic API error {}: {}", status, text);
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

                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(event) = serde_json::from_str::<AnthropicStreamEvent>(data) {
                            match event.kind.as_str() {
                                "content_block_delta" => {
                                    if let Some(delta) = event.delta {
                                        if let Some(text) = delta.text {
                                            full_text.push_str(&text);
                                            on_chunk(StreamChunk { text, done: false });
                                        }
                                    }
                                }
                                "message_stop" => {
                                    on_chunk(StreamChunk { text: String::new(), done: true });
                                    return Ok(full_text);
                                }
                                _ => {}
                            }
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
