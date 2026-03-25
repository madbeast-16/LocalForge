use async_trait::async_trait;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::error::{LocalForgeError, Result};
use crate::models::ModelEntry;
use super::{InferenceEngine, ChatSession, ContextUsage};

/// LlamaCpp inference engine — manages a `llama-server` child process
/// and communicates via its HTTP API (OpenAI-compatible on localhost).
pub struct LlamaCppEngine {
    /// Path to the llama-server binary
    server_bin: String,
    /// Running server process (if any)
    process: Option<Child>,
    /// Port the server listens on
    port: u16,
    /// Currently loaded model path
    loaded_model: Option<String>,
    /// Context size
    ctx_size: usize,
    /// Parallel slots
    n_parallel: usize,
}

impl LlamaCppEngine {
    pub fn new(server_bin: String) -> Self {
        Self {
            server_bin,
            process: None,
            port: 8081,
            loaded_model: None,
            ctx_size: 4096,
            n_parallel: 1,
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_ctx_size(mut self, ctx: usize) -> Self {
        self.ctx_size = ctx;
        self
    }

    pub fn with_parallel(mut self, n: usize) -> Self {
        self.n_parallel = n;
        self
    }

    /// Wait for the llama-server to be ready (poll /health).
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
            "llama-server failed to become ready within 30s".into(),
        ))
    }

    /// Build the OpenAI-compatible chat completion request body.
    fn build_chat_request(session: &ChatSession) -> serde_json::Value {
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

        serde_json::json!({
            "messages": messages,
            "stream": false,
        })
    }
}

impl Drop for LlamaCppEngine {
    fn drop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[async_trait]
impl InferenceEngine for LlamaCppEngine {
    async fn load_model(&mut self, model: &ModelEntry) -> Result<()> {
        // Kill existing process if any
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        // Resolve model path
        let model_path = format!(
            "{}/models/{}.gguf",
            dirs::data_local_dir()
                .unwrap_or_default()
                .join("localforge")
                .display(),
            model.name
        );

        // Start llama-server
        let child = Command::new(&self.server_bin)
            .arg("-m")
            .arg(&model_path)
            .arg("--port")
            .arg(self.port.to_string())
            .arg("-c")
            .arg(self.ctx_size.to_string())
            .arg("-np")
            .arg(self.n_parallel.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                LocalForgeError::InferenceError(format!(
                    "Failed to start llama-server: {}",
                    e
                ))
            })?;

        self.process = Some(child);
        self.loaded_model = Some(model.name.clone());

        // Wait for server to be ready
        self.wait_for_ready().await?;

        Ok(())
    }

    async fn unload_model(&mut self) -> Result<()> {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.loaded_model = None;
        Ok(())
    }

    async fn generate(&mut self, session: &ChatSession) -> Result<String> {
        if self.process.is_none() {
            return Err(LocalForgeError::InferenceError(
                "No model loaded".into(),
            ));
        }

        let url = format!(
            "http://127.0.0.1:{}/v1/chat/completions",
            self.port
        );
        let body = Self::build_chat_request(session);

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                LocalForgeError::InferenceError(format!("Request failed: {}", e))
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(LocalForgeError::InferenceError(format!(
                "llama-server returned {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| {
            LocalForgeError::InferenceError(format!("Failed to parse response: {}", e))
        })?;

        // Extract the assistant's response from OpenAI format
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    async fn get_context_usage(&self) -> Result<ContextUsage> {
        // Query /health or /slots for context info
        if self.process.is_none() {
            return Ok(ContextUsage {
                used_tokens: 0,
                total_tokens: self.ctx_size,
                percentage: 0.0,
                estimated_kv_vram_mb: 0,
            });
        }

        let url = format!("http://127.0.0.1:{}/health", self.port);
        let client = reqwest::Client::new();

        match client.get(&url).send().await {
            Ok(_) => Ok(ContextUsage {
                used_tokens: 0,
                total_tokens: self.ctx_size,
                percentage: 0.0,
                estimated_kv_vram_mb: 0,
            }),
            Err(_) => Ok(ContextUsage {
                used_tokens: 0,
                total_tokens: self.ctx_size,
                percentage: 0.0,
                estimated_kv_vram_mb: 0,
            }),
        }
    }
}