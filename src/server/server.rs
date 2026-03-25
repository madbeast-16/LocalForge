use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::error::{LocalForgeError, Result};
use super::api::{self, ApiState, SharedState};
use super::config::ServerConfig;

use std::sync::Arc;
use tokio::sync::RwLock;

/// The LocalForge HTTP server — wraps axum with optional TLS.
pub struct LocalForgeServer {
    config: ServerConfig,
    state: SharedState,
}

impl LocalForgeServer {
    pub fn new(config: ServerConfig) -> Self {
        let state = Arc::new(RwLock::new(ApiState::new(config.clone())));
        Self { config, state }
    }

    /// Get a handle to the shared API state (for setting loaded model, etc.)
    pub fn state(&self) -> SharedState {
        self.state.clone()
    }

    /// Set the currently loaded model name.
    pub async fn set_loaded_model(&self, model: Option<String>) {
        let mut st = self.state.write().await;
        st.loaded_model = model;
    }

    /// Generate a self-signed TLS certificate for the configured host.
    fn generate_self_signed_cert(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let cert = rcgen::generate_simple_self_signed(vec![
            self.config.host.clone(),
            "localhost".into(),
        ])
        .map_err(|e| LocalForgeError::ServerError(format!("Cert generation failed: {}", e)))?;

        let cert_pem = cert.cert.pem().into_bytes();
        let key_pem = cert.key_pair.serialize_pem().into_bytes();
        Ok((cert_pem, key_pem))
    }

    /// Run the server, blocking until shutdown.
    pub async fn run(self) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|e: std::net::AddrParseError| {
                LocalForgeError::ServerError(format!("Invalid address: {}", e))
            })?;

        let router = api::build_router(self.state.clone());

        tracing::info!("LocalForge API server listening on http://{}", addr);

        // For now, start plain HTTP. TLS can be added when axum-server
        // or tokio-rustls is available as a dependency.
        if self.config.tls.is_some() {
            tracing::info!("TLS configured but using HTTP for now (add axum-server dep for HTTPS)");
            let (_cert, _key) = self.generate_self_signed_cert()?;
            tracing::info!("Self-signed certificate generated successfully");
        }

        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            LocalForgeError::ServerError(format!("Failed to bind {}: {}", addr, e))
        })?;

        axum::serve(listener, router).await.map_err(|e| {
            LocalForgeError::ServerError(format!("Server error: {}", e))
        })?;

        Ok(())
    }
}