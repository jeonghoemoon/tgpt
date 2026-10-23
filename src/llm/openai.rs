use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::ProviderConfig;
use super::{LlmBackend, Message, Role, StreamChunk};

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Serialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiChoiceMessage,
}

#[derive(Deserialize)]
struct OpenAiChoiceMessage {
    content: String,
}

#[derive(Deserialize)]
struct OpenAiStreamDelta {
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

// ── Backend ───────────────────────────────────────────────────────────────────

pub struct OpenAiBackend {
    client: Client,
    cfg: ProviderConfig,
    api_key: String,
}

impl OpenAiBackend {
    pub fn new(cfg: ProviderConfig) -> Result<Self> {
        let api_key = cfg
            .api_key
            .clone()
            .filter(|k| !k.is_empty())
            .context(
                "OpenAI API key is not set.\n\
                 Run:  tgpt config set-key openai <YOUR_KEY>\n\
                 Or set the OPENAI_API_KEY environment variable.",
            )
            .or_else(|_| {
                std::env::var("OPENAI_API_KEY")
                    .context("OpenAI API key not found in config or OPENAI_API_KEY env var")
            })?;

        Ok(Self {
            client: Client::new(),
            cfg,
            api_key,
        })
    }

    fn messages_to_json<'a>(&self, messages: &'a [Message]) -> Vec<OpenAiMessage<'a>> {
        messages
            .iter()
            .map(|m| OpenAiMessage {
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
impl LlmBackend for OpenAiBackend {
    fn name(&self) -> &str { "openai" }
    fn model(&self) -> &str { &self.cfg.model }

    async fn complete(&self, messages: &[Message]) -> Result<String> {
        let url = format!("{}/chat/completions", self.cfg.base_url);
        let body = json!({
            "model": self.cfg.model,
            "messages": self.messages_to_json(messages),
            "max_tokens": self.cfg.max_tokens,
        });

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .context("Failed to connect to OpenAI API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("OpenAI API error {}: {}", status, text);
        }

        let data: OpenAiResponse = resp.json().await.context("Failed to parse OpenAI response")?;
        data.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .context("Empty response from OpenAI")
    }

    async fn stream(
        &self,
        messages: &[Message],
        on_chunk: &mut (dyn FnMut(StreamChunk) + Send),
    ) -> Result<String> {
        let url = format!("{}/chat/completions", self.cfg.base_url);
        let body = json!({
            "model": self.cfg.model,
            "messages": self.messages_to_json(messages),
            "max_tokens": self.cfg.max_tokens,
            "stream": true,
        });

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .context("Failed to connect to OpenAI API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("OpenAI API error {}: {}", status, text);
        }

        let mut stream = resp.bytes_stream();
        let mut full_text = String::new();
        let mut buf = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Stream read error")?;
            let text = std::str::from_utf8(&chunk).unwrap_or("");
            buf.push_str(text);

            // Process complete SSE lines
            loop {
                if let Some(nl) = buf.find('\n') {
                    let line = buf[..nl].trim().to_string();
                    buf.drain(..=nl);

                    if line.is_empty() || line == ":" {
                        continue;
                    }

                    let data = if let Some(d) = line.strip_prefix("data: ") {
                        d
                    } else {
                        continue;
                    };

                    if data == "[DONE]" {
                        on_chunk(StreamChunk { text: String::new(), done: true });
                        return Ok(full_text);
                    }

                    if let Ok(parsed) = serde_json::from_str::<OpenAiStreamChunk>(data) {
                        for choice in parsed.choices {
                            if let Some(token) = choice.delta.content {
                                full_text.push_str(&token);
                                on_chunk(StreamChunk { text: token, done: false });
                            }
                            if choice.finish_reason.is_some() {
                                on_chunk(StreamChunk { text: String::new(), done: true });
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
