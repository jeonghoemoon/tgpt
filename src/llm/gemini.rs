use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::ProviderConfig;
use super::{LlmBackend, Message, Role, StreamChunk};

// ── Google Gemini Request/Response Models ────────────────────────────────────

#[derive(Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct GeminiContent<'a> {
    role: &'a str,
    parts: Vec<GeminiPart<'a>>,
}

#[derive(Serialize)]
struct ThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    thinking_budget: u32,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none", rename = "thinkingConfig")]
    thinking_config: Option<ThinkingConfig>,
}

#[derive(Serialize)]
struct GeminiRequestBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction<'a>>,
    contents: Vec<GeminiContent<'a>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "generationConfig")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize)]
struct GeminiSystemInstruction<'a> {
    parts: Vec<GeminiPart<'a>>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiResponseContent>,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Option<Vec<GeminiResponsePart>>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: Option<String>,
}

// ── Backend Implementation ───────────────────────────────────────────────────

pub struct GeminiBackend {
    client: Client,
    cfg: ProviderConfig,
    api_key: String,
}

impl GeminiBackend {
    pub fn new(cfg: ProviderConfig) -> Result<Self> {
        let api_key = cfg
            .api_key
            .clone()
            .filter(|k| !k.is_empty())
            .or_else(|| std::env::var("GEMINI_API_KEY").ok())
            .context(
                "Gemini API key is not set.\n\
                 Run:  tgpt config set-key gemini <YOUR_KEY>\n\
                 Or set the GEMINI_API_KEY environment variable.",
            )?;

        Ok(Self {
            client: Client::new(),
            cfg,
            api_key,
        })
    }

    fn build_request_body<'a>(&self, messages: &'a [Message]) -> GeminiRequestBody<'a> {
        let system_text = messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.as_str());

        let system_instruction = system_text.map(|txt| GeminiSystemInstruction {
            parts: vec![GeminiPart { text: txt }],
        });

        let contents: Vec<GeminiContent> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| GeminiContent {
                role: match m.role {
                    Role::User => "user",
                    Role::Assistant => "model",
                    Role::System => unreachable!(),
                },
                parts: vec![GeminiPart {
                    text: &m.content,
                }],
            })
            .collect();

        // Disable extended thinking latency for snappy instant terminal streaming
        let generation_config = Some(GenerationConfig {
            thinking_config: Some(ThinkingConfig { thinking_budget: 0 }),
        });

        GeminiRequestBody {
            system_instruction,
            contents,
            generation_config,
        }
    }
}

#[async_trait]
impl LlmBackend for GeminiBackend {
    fn name(&self) -> &str {
        "gemini"
    }

    fn model(&self) -> &str {
        &self.cfg.model
    }

    async fn complete(&self, messages: &[Message]) -> Result<String> {
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.cfg.base_url.trim_end_matches('/'),
            self.cfg.model,
            self.api_key
        );

        let body = self.build_request_body(messages);

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to connect to Gemini API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("Gemini API error {}: {}", status, text);
        }

        let data: GeminiResponse = resp.json().await.context("Failed to parse Gemini response")?;

        let text = data
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts)
            .and_then(|parts| {
                let s = parts
                    .into_iter()
                    .filter_map(|p| p.text)
                    .collect::<Vec<_>>()
                    .join("");
                if s.is_empty() { None } else { Some(s) }
            })
            .context("Empty response from Gemini")?;

        Ok(text)
    }

    async fn stream(
        &self,
        messages: &[Message],
        on_chunk: &mut (dyn FnMut(StreamChunk) + Send),
    ) -> Result<String> {
        let url = format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            self.cfg.base_url.trim_end_matches('/'),
            self.cfg.model,
            self.api_key
        );

        let body = self.build_request_body(messages);

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to connect to Gemini API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("Gemini API error {}: {}", status, text);
        }

        let mut stream = resp.bytes_stream();
        let mut full_text = String::new();
        let mut buf = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Stream read error")?;
            buf.push_str(std::str::from_utf8(&chunk).unwrap_or(""));

            loop {
                if let Some(nl) = buf.find('\n') {
                    let line = buf[..nl].trim().to_string();
                    buf.drain(..=nl);

                    if line.is_empty() || line.starts_with(':') {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(parsed) = serde_json::from_str::<GeminiResponse>(data) {
                            if let Some(candidate) = parsed.candidates.and_then(|mut c| c.pop()) {
                                if let Some(content) = candidate.content {
                                    if let Some(parts) = content.parts {
                                        for part in parts {
                                            if let Some(token) = part.text {
                                                full_text.push_str(&token);
                                                on_chunk(StreamChunk {
                                                    text: token,
                                                    done: false,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    break;
                }
            }
        }

        on_chunk(StreamChunk {
            text: String::new(),
            done: true,
        });

        Ok(full_text)
    }
}
