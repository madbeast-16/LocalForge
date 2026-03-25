use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::config::ServerConfig;

// ── Shared application state for the API ──

pub struct ApiState {
    pub config: ServerConfig,
    pub loaded_model: Option<String>,
}

impl ApiState {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            loaded_model: None,
        }
    }
}

pub type SharedState = Arc<RwLock<ApiState>>;

// ── Request / Response types (OpenAI-compatible) ──

#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessageReq>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default)]
    pub stream: bool,
}

fn default_max_tokens() -> u32 {
    2048
}
fn default_temperature() -> f32 {
    0.7
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatMessageReq {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessageReq,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub owned_by: String,
}

#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    pub object: String,
    pub data: Vec<ModelInfo>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

// ── Route handlers ──

/// GET /health
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

/// GET /v1/models
async fn models_handler(State(state): State<SharedState>) -> Json<ModelsResponse> {
    let st = state.read().await;
    let mut models = Vec::new();

    if let Some(ref model) = st.loaded_model {
        models.push(ModelInfo {
            id: model.clone(),
            object: "model".into(),
            owned_by: "localforge".into(),
        });
    }

    Json(ModelsResponse {
        object: "list".into(),
        data: models,
    })
}

/// POST /v1/chat/completions
async fn chat_completions_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(req): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, (StatusCode, String)> {
    let st = state.read().await;

    // Check API key if keys are configured
    if !st.config.api_keys.is_empty() {
        let auth = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let token = auth.strip_prefix("Bearer ").unwrap_or("");
        let valid = st.config.api_keys.iter().any(|k| k.key == token);

        if !valid {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid API key".into(),
            ));
        }
    }

    let model_name = req
        .model
        .unwrap_or_else(|| st.loaded_model.clone().unwrap_or("unknown".into()));

    // TODO: proxy to actual inference engine
    // For now, return a placeholder response
    let response = ChatCompletionResponse {
        id: format!("chatcmpl-{}", generate_id()),
        object: "chat.completion".into(),
        created: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        model: model_name,
        choices: vec![Choice {
            index: 0,
            message: ChatMessageReq {
                role: "assistant".into(),
                content: "(LocalForge API: inference engine not yet connected)".into(),
            },
            finish_reason: "stop".into(),
        }],
        usage: Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    };

    Ok(Json(response))
}

/// Generate a simple random ID for response tracking.
fn generate_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let id: u64 = rng.gen();
    format!("{:016x}", id)
}

// ── Router builder ──

/// Build the complete API router with all routes.
pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/models", get(models_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .with_state(state)
}
