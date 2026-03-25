use async_trait::async_trait;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::error::{LocalForgeError, Result};
use crate::models::ModelEntry;
use super::{InferenceEngine, ChatSession, ContextUsage};

/// LlamaSwap inference engine — manages a `llama-swap` process
/// that handles hot-swapping between models without server restarts.
pub struct LlamaSwapEngine {
    /// Path to the llama-swap binary
    swap_bin: String,
    /// Running swap process (if any)
    process: Option<Child>,
    /// Port the swap proxy listens on
    port: u16,
    /// Currently active model
    active_model: Option<String>,
    /// Context size
    ctx_size: usize,
}

impl LlamaSwapEngine {
    pub fn new(swap_bin: String) -> Self {
        Self {
            swap_bin,
            process: None,
            port: 8082,
            active_model: None,
            ctx_size: 4096,
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Wait for the llama-swap to be ready.
    async fn wait_for_ready(&self) -> Result<()> {
        let url = format!("http://127.0.0.1:{}/health", self.port);
        let client = reqwest::Client::new();

        for _ in 0..60 {
            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                _ => tokio::time::sleep(Duration::from_millis(500)).await,
            }
        }

        Err(LocalForgeError::InferenceError(
            "llama-swap failed to become ready within 30s".into(),
        ))
    }
}

impl Drop for LlamaSwapEngine {
    fn drop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[async_trait]
impl InferenceEngine for LlamaSwapEngine {
    async fn load_model(&mut self, model: &ModelEntry) -> Result<()> {
        // If the swap proxy isn't running yet, start it
        if self.process.is_none() {
            let child = Command::new(&self.swap_bin)
                .arg("--port")
                .arg(self.port.to_string())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| {
                    LocalForgeError::InferenceError(format!(
                        "Failed to start llama-swap: {}",
                        e
                    ))
                })?;

            self.process = Some(child);
            self.wait_for_ready().await?;
        }

        // Tell llama-swap to swap to the requested model
        let url = format!(
            "http://127.0.0.1:{}/v1/chat/completions",
            self.port
        );
        let client = reqwest::Client::new();

        // Send a model swap hint via the model field
        let body = serde_json::json!({
            "model": model.name,
            "messages": [{"role": "system", "content": "ping"}],
            "max_tokens": 1,
        });

        let resp = client.post(&url).json(&body).send().await.map_err(|e| {
            LocalForgeError::InferenceError(format!("Model swap request failed: {}", e))
        })?;

        if !resp.status().is_success() {
            return Err(LocalForgeError::InferenceError(
                "Failed to swap model".into(),
            ));
        }

        self.active_model = Some(model.name.clone());
        Ok(())
    }

    async fn unload_model(&mut self) -> Result<()> {
        // llama-swap manages its own model lifecycle
        self.active_model = None;
        Ok(())
    }

    async fn generate(&mut self, session: &ChatSession) -> Result<String> {
        if self.process.is_none() {
            return Err(LocalForgeError::InferenceError(
                "llama-swap not running".into(),
            ));
        }

        let url = format!(
            "http://127.0.0.1:{}/v1/chat/completions",
            self.port
        );

        let messages: Vec<serde_json::Value> = session
            .messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    super::session::ChatRole::System => "system",
                    super::session::ChatRole::User => "user",
                    super::session::ChatRole::Assistant => "assistant",
                };
                serde_json::json!({
                    "role": role,
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": self.active_model.as_deref().unwrap_or("default"),
            "messages": messages,
            "stream": false,
        });

        let client = reqwest::Client::new();
        let resp = client.post(&url).json(&body).send().await.map_err(|e| {
            LocalForgeError::InferenceError(format!("Request failed: {}", e))
        })?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(LocalForgeError::InferenceError(format!(
                "llama-swap error: {}",
                text
            )));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| {
            LocalForgeError::InferenceError(format!("Parse failed: {}", e))
        })?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    async fn get_context_usage(&self) -> Result<ContextUsage> {
        Ok(ContextUsage {
            used_tokens: 0,
            total_tokens: self.ctx_size,
            percentage: 0.0,
            estimated_kv_vram_mb: 0,
        })
    }
}