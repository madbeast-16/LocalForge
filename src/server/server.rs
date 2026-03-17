use axum::Router;
use std::net::SocketAddr;
use crate::error::Result;
use crate::server::config::ServerConfig;

pub struct LocalForgeServer {
    config: ServerConfig,
    router: Router,
}

impl LocalForgeServer {
    pub fn new(config: ServerConfig) -> Self {
        let router = Router::new();
        Self { config, router }
    }
    
    pub async fn run(self) -> Result<()> {
        let addr_str = format!("{}:{}", self.config.host, self.config.port);
        let addr: SocketAddr = addr_str
            .parse()
            .map_err(|e: std::net::AddrParseError| crate::error::LocalForgeError::ServerError(e.to_string()))?;
        
        Ok(())
    }
}