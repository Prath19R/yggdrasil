use crate::provider::{InferenceProvider, StreamResult};
use crate::types::{ModelMetadata, StreamChunk, UnifiedChatRequest};
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

const NVIDIA_API_URL: &str = "https://integrate.api.nvidia.com/v1/chat/completions";

pub struct NvidiaNimProvider {
    api_key: String,
    client: reqwest::Client,
}

impl NvidiaNimProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct NvidiaPayload<'a> {
    model: &'a str,
    messages: &'a [crate::types::UnifiedMessage],
    temperature: f32,
    max_tokens: u32,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<ChatTemplateKwargs>,
}

#[derive(Serialize)]
struct ChatTemplateKwargs {
    enable_thinking: bool,
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
    reasoning_content: Option<String>,
}

#[async_trait]
impl InferenceProvider for NvidiaNimProvider {
    fn provider_name(&self) -> &'static str {
        "NVIDIA NIM (Cloud)"
    }

    async fn list_models(&self) -> Result<Vec<ModelMetadata>, String> {
        Ok(vec![
            ModelMetadata {
                id: "nvidia/nemotron-3.5-lightning-30b-a3b".into(),
                name: "Nemotron 3.5 Lightning (Reasoning)".into(),
                is_multimodal: false,
                supports_thinking: true,
                context_length: 32768,
            },
            ModelMetadata {
                id: "mistralai/mistral-large-4-1t-a52b".into(),
                name: "Mistral Large Multimodal (Vision & Architect)".into(),
                is_multimodal: true,
                supports_thinking: true,
                context_length: 131072,
            },
            ModelMetadata {
                id: "qwen/qwen2.5-coder-32b-instruct".into(),
                name: "Qwen 2.5 Coder 32B (High-Speed Code)".into(),
                is_multimodal: false,
                supports_thinking: false,
                context_length: 32768,
            },
            ModelMetadata {
                id: "deepseek-ai/deepseek-r1".into(),
                name: "DeepSeek R1 (Deep Algorithmic Logic)".into(),
                is_multimodal: false,
                supports_thinking: true,
                context_length: 65536,
            },
        ])
    }

    async fn stream_chat(&self, req: UnifiedChatRequest) -> Result<StreamResult, String> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let auth_val = format!("Bearer {}", self.api_key);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_val).map_err(|e| e.to_string())?,
        );

        let kwargs = if req.enable_thinking {
            Some(ChatTemplateKwargs { enable_thinking: true })
        } else {
            None
        };

        let payload = NvidiaPayload {
            model: &req.model,
            messages: &req.messages,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            stream: true,
            reasoning_effort: req.reasoning_effort.as_deref(),
            chat_template_kwargs: kwargs,
        };

        let response = self
            .client
            .post(NVIDIA_API_URL)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("NVIDIA API connection error: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let err_text = response.text().await.unwrap_or_default();
            return Err(format!("NVIDIA NIM API Error ({status}): {err_text}"));
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
                                if let Some(reasoning) = delta.reasoning_content {
                                    return Ok(StreamChunk {
                                        text_delta: reasoning,
                                        is_thinking: true,
                                        is_done,
                                    });
                                }
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
            Err(e) => Err(format!("SSE Stream parsing error: {}", e)),
        });

        Ok(Box::pin(mapped_stream))
    }
}
