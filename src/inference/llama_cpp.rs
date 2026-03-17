use async_trait::async_trait;
use crate::error::Result;
use crate::models::ModelEntry;
use super::{InferenceEngine, ChatSession, ContextUsage};

pub struct LlamaCppEngine;

#[async_trait]
impl InferenceEngine for LlamaCppEngine {
    async fn load_model(&mut self, model: &ModelEntry) -> Result<()> {
        todo!("Load model via llama.cpp")
    }
    
    async fn unload_model(&mut self) -> Result<()> {
        todo!("Unload model")
    }
    
    async fn generate(&mut self, session: &ChatSession) -> Result<String> {
        todo!("Generate text")
    }
    
    async fn get_context_usage(&self) -> Result<ContextUsage> {
        todo!("Get context usage")
    }
}