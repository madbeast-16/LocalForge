use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls: Option<TlsConfig>,
    #[serde(skip)]
    pub api_keys: Vec<ApiKey>,
    pub rate_limit_per_key: Option<u32>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8080,
            tls: None,
            api_keys: Vec::new(),
            rate_limit_per_key: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Clone)]
pub struct ApiKey {
    pub id: String,
    pub key: String,
    pub name: String,
}
