use crate::nvidia::NvidiaNimProvider;
use crate::ollama::OllamaProvider;
use crate::provider::{InferenceProvider, StreamResult};
use crate::types::{InferenceMode, ModelMetadata, UnifiedChatRequest};
use std::sync::Arc;

use futures::StreamExt;

#[derive(Debug, Clone, Default)]
pub struct ChatResponse {
    pub text: String,
    pub thinking: Option<String>,
}

pub struct InferenceRouter {
    pub mode: InferenceMode,
    pub active_model: String,
    cloud_provider: Option<Arc<NvidiaNimProvider>>,
    local_provider: Arc<OllamaProvider>,
}

impl InferenceRouter {
    pub fn new(nvidia_api_key: Option<String>, local_endpoint: Option<String>) -> Self {
        let cloud = nvidia_api_key.map(|k| Arc::new(NvidiaNimProvider::new(k)));
        let local = Arc::new(match local_endpoint {
            Some(e) => OllamaProvider::new(e),
            None => OllamaProvider::default(),
        });

        let (mode, active_model) = if cloud.is_some() {
            (
                InferenceMode::CloudApi,
                "nvidia/nemotron-3.5-lightning-30b-a3b".to_string(),
            )
        } else {
            (InferenceMode::Local, "qwen2.5-coder:7b".to_string())
        };

        Self {
            mode,
            active_model,
            cloud_provider: cloud,
            local_provider: local,
        }
    }

    pub fn set_api_key(&mut self, key: String) {
        let trimmed = key.trim().to_string();
        if !trimmed.is_empty() {
            self.cloud_provider = Some(Arc::new(NvidiaNimProvider::new(trimmed)));
        } else {
            self.cloud_provider = None;
        }
    }

    pub fn has_api_key(&self) -> bool {
        self.cloud_provider.is_some()
    }

    pub fn set_mode(&mut self, mode: InferenceMode) {
        self.mode = mode;
        if self.mode == InferenceMode::CloudApi {
            self.active_model = "nvidia/nemotron-3.5-lightning-30b-a3b".to_string();
        } else {
            self.active_model = "qwen2.5-coder:7b".to_string();
        }
    }

    pub async fn list_models(&self) -> Vec<ModelMetadata> {
        match self.mode {
            InferenceMode::CloudApi => {
                if let Some(cloud) = &self.cloud_provider {
                    cloud.list_models().await.unwrap_or_default()
                } else {
                    self.local_provider.list_models().await.unwrap_or_default()
                }
            }
            InferenceMode::Local => self.local_provider.list_models().await.unwrap_or_default(),
        }
    }

    pub async fn stream_chat(&self, req: UnifiedChatRequest) -> Result<StreamResult, String> {
        match self.mode {
            InferenceMode::CloudApi => {
                if let Some(cloud) = &self.cloud_provider {
                    match cloud.stream_chat(req.clone()).await {
                        Ok(stream) => Ok(stream),
                        Err(err) => {
                            eprintln!("Cloud NIM failed, falling back to local: {}", err);
                            // Resilient auto-fallback to local Ollama
                            let mut local_req = req;
                            local_req.model = "qwen2.5-coder:7b".to_string();
                            self.local_provider.stream_chat(local_req).await
                        }
                    }
                } else {
                    self.local_provider.stream_chat(req).await
                }
            }
            InferenceMode::Local => self.local_provider.stream_chat(req).await,
        }
    }

    pub async fn chat(&self, req: UnifiedChatRequest) -> Result<ChatResponse, String> {
        let mut stream = self.stream_chat(req).await?;
        let mut text = String::new();
        let mut thinking = String::new();

        while let Some(chunk_res) = stream.next().await {
            match chunk_res {
                Ok(chunk) => {
                    if chunk.is_thinking {
                        thinking.push_str(&chunk.text_delta);
                    } else {
                        text.push_str(&chunk.text_delta);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        let thinking_opt = if thinking.trim().is_empty() {
            None
        } else {
            Some(thinking)
        };

        Ok(ChatResponse {
            text,
            thinking: thinking_opt,
        })
    }
}
