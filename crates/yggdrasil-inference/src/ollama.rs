use crate::provider::{InferenceProvider, StreamResult};
use crate::types::{ModelMetadata, StreamChunk, UnifiedChatRequest};
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

pub struct OllamaProvider {
    endpoint: String,
    client: reqwest::Client,
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            client: reqwest::Client::new(),
        }
    }
}

impl OllamaProvider {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OpenAiCompatiblePayload<'a> {
    model: &'a str,
    messages: &'a [crate::types::UnifiedMessage],
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

#[derive(Deserialize)]
struct OpenAiChatChunk {
    choices: Option<Vec<ChunkChoice>>,
}

#[derive(Deserialize)]
struct ChunkChoice {
    delta: Option<ChunkDelta>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ChunkDelta {
    content: Option<String>,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Option<Vec<OllamaModelTag>>,
}

#[derive(Deserialize)]
struct OllamaModelTag {
    name: String,
}

#[async_trait]
impl InferenceProvider for OllamaProvider {
    fn provider_name(&self) -> &'static str {
        "Ollama (Local)"
    }

    async fn list_models(&self) -> Result<Vec<ModelMetadata>, String> {
        let url = format!("{}/api/tags", self.endpoint);
        if let Ok(resp) = self.client.get(&url).send().await {
            if let Ok(tags) = resp.json::<OllamaTagsResponse>().await {
                if let Some(models) = tags.models {
                    return Ok(models
                        .into_iter()
                        .map(|m| ModelMetadata {
                            id: m.name.clone(),
                            name: m.name,
                            is_multimodal: false,
                            supports_thinking: false,
                            context_length: 32768,
                        })
                        .collect());
                }
            }
        }

        // Default fallback list for offline development
        Ok(vec![
            ModelMetadata {
                id: "qwen2.5-coder:7b".into(),
                name: "Qwen 2.5 Coder 7B (Local)".into(),
                is_multimodal: false,
                supports_thinking: false,
                context_length: 32768,
            },
            ModelMetadata {
                id: "deepseek-r1:8b".into(),
                name: "DeepSeek R1 8B (Local Thinking)".into(),
                is_multimodal: false,
                supports_thinking: true,
                context_length: 32768,
            },
            ModelMetadata {
                id: "llama3.2:3b".into(),
                name: "Llama 3.2 3B (Ultra Fast)".into(),
                is_multimodal: false,
                supports_thinking: false,
                context_length: 8192,
            },
        ])
    }

    async fn stream_chat(&self, req: UnifiedChatRequest) -> Result<StreamResult, String> {
        let url = format!("{}/v1/chat/completions", self.endpoint);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = OpenAiCompatiblePayload {
            model: &req.model,
            messages: &req.messages,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Ollama local server unreachable at {}: {}", self.endpoint, e))?;

        if !response.status().is_success() {
            let status = response.status();
            let err_text = response.text().await.unwrap_or_default();
            return Err(format!("Ollama Error ({status}): {err_text}"));
        }

        let event_stream = response.bytes_stream().eventsource();
        let mapped_stream = event_stream.map(|event_result| match event_result {
            Ok(event) => {
                if event.data == "[DONE]" {
                    return Ok(StreamChunk {
                        text_delta: String::new(),
                        is_thinking: false,
                        is_done: true,
                    });
                }

                if let Ok(chunk) = serde_json::from_str::<OpenAiChatChunk>(&event.data) {
                    if let Some(choices) = chunk.choices {
                        if let Some(first) = choices.into_iter().next() {
                            let is_done = first.finish_reason.is_some();
                            if let Some(delta) = first.delta {
                                if let Some(content) = delta.content {
                                    return Ok(StreamChunk {
                                        text_delta: content,
                                        is_thinking: false,
                                        is_done,
                                    });
                                }
                            }
                        }
                    }
                }
                Ok(StreamChunk {
                    text_delta: String::new(),
                    is_thinking: false,
                    is_done: false,
                })
            }
            Err(e) => Err(format!("Ollama stream parsing error: {}", e)),
        });

        Ok(Box::pin(mapped_stream))
    }
}
