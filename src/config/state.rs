use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpertiseLevel {
    Beginner,
    Intermediate,
    Expert,
}

impl Default for ExpertiseLevel {
    fn default() -> Self {
        ExpertiseLevel::Intermediate
    }
}

impl ExpertiseLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExpertiseLevel::Beginner => "Beginner",
            ExpertiseLevel::Intermediate => "Intermediate",
            ExpertiseLevel::Expert => "Expert",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ExpertiseLevel::Beginner => "Show more explanations, hide advanced options",
            ExpertiseLevel::Intermediate => "Show recommendations, allow overrides",
            ExpertiseLevel::Expert => "Show all flags and internals, minimal hand-holding",
        }
    }

    pub fn all() -> &'static [ExpertiseLevel] {
        &[
            ExpertiseLevel::Beginner,
            ExpertiseLevel::Intermediate,
            ExpertiseLevel::Expert,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub expertise: ExpertiseLevel,
    pub setup_complete: bool,
    pub build_complete: bool,
    pub llama_swap_installed: bool,
    pub downloaded_models: Vec<String>,
    pub active_model: Option<String>,
    pub server_running: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            expertise: ExpertiseLevel::default(),
            setup_complete: false,
            build_complete: false,
            llama_swap_installed: false,
            downloaded_models: Vec::new(),
            active_model: None,
            server_running: false,
        }
    }
}

impl AppState {
    pub fn default_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("localforge")
            .join("state.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::default_path();
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let state: AppState = toml::from_str(&content)
            .map_err(|e| crate::error::LocalForgeError::Serialization(e.to_string()))?;
        Ok(state)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::default_path();
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::LocalForgeError::Serialization(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
