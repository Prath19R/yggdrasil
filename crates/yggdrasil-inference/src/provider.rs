use crate::types::{ModelMetadata, StreamChunk, UnifiedChatRequest};
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

pub type StreamResult = Pin<Box<dyn Stream<Item = Result<StreamChunk, String>> + Send>>;

#[async_trait]
pub trait InferenceProvider: Send + Sync {
    fn provider_name(&self) -> &'static str;
    
    async fn list_models(&self) -> Result<Vec<ModelMetadata>, String>;

    async fn stream_chat(
        &self,
        request: UnifiedChatRequest,
    ) -> Result<StreamResult, String>;
}
