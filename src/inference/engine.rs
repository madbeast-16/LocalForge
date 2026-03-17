use async_trait::async_trait;
use crate::error::Result;
use crate::models::ModelEntry;

use super::session::ChatSession;

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn load_model(&mut self, model: &ModelEntry) -> Result<()>;
    async fn unload_model(&mut self) -> Result<()>;
    async fn generate(&mut self, session: &ChatSession) -> Result<String>;
    async fn get_context_usage(&self) -> Result<super::session::ContextUsage>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceEngineType {
    LlamaCpp,
    LlamaSwap,
}