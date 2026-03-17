use async_trait::async_trait;
use crate::error::Result;
use crate::models::ModelEntry;
use super::{InferenceEngine, ChatSession, ContextUsage};

pub struct LlamaSwapEngine;

#[async_trait]
impl InferenceEngine for LlamaSwapEngine {
    async fn load_model(&mut self, model: &ModelEntry) -> Result<()> {
        todo!("Swap to model via llama-swap")
    }
    
    async fn unload_model(&mut self) -> Result<()> {
        todo!("Unload model")
    }
    
    async fn generate(&mut self, session: &ChatSession) -> Result<String> {
        todo!("Generate text via swap")
    }
    
    async fn get_context_usage(&self) -> Result<ContextUsage> {
        todo!("Get context usage")
    }
}