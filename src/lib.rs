pub mod app;
pub mod build;
pub mod cli;
pub mod config;
pub mod hardware;
pub mod models;
pub mod tui;

// Re-export key types for library consumers
pub use config::{AppConfig, Backend, BackendName};
pub use hardware::HardwareInfo;
pub use models::ModelEntry;
