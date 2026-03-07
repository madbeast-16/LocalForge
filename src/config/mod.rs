pub mod backends;
pub mod cmake;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Backend types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendName {
    Cuda,
    Hip,
    Metal,
    Vulkan,
    OpenBlas,
    CpuOnly,
}

impl BackendName {
    pub fn as_str(&self) -> &'static str {
        match self {
            BackendName::Cuda => "CUDA (NVIDIA)",
            BackendName::Hip => "HIP (AMD ROCm)",
            BackendName::Metal => "Metal (Apple Silicon)",
            BackendName::Vulkan => "Vulkan",
            BackendName::OpenBlas => "OpenBLAS (CPU optimized)",
            BackendName::CpuOnly => "CPU Only",
        }
    }

    pub fn all() -> &'static [BackendName] {
        &[
            BackendName::Cuda,
            BackendName::Hip,
            BackendName::Metal,
            BackendName::Vulkan,
            BackendName::OpenBlas,
            BackendName::CpuOnly,
        ]
    }
}

impl std::fmt::Display for BackendName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backend {
    pub name: BackendName,
    pub description: String,
    pub cmake_flags: Vec<String>,
    pub cuda_arch: Option<String>,
}

// ---------------------------------------------------------------------------
// Application config (config.toml)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub install_prefix: String,
    pub models_dir: String,
    pub backend: Option<BackendName>,
    pub parallel_jobs: Option<usize>,
    pub headless: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("llama-install");
        Self {
            install_prefix: "~/.local".into(),
            models_dir: data_dir.join("models").to_string_lossy().into_owned(),
            backend: None,
            parallel_jobs: None,
            headless: false,
        }
    }
}

impl AppConfig {
    /// Standard config file path: `~/.config/llama-install/config.toml`
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("llama-install")
            .join("config.toml")
    }

    /// Load from disk, falling back to defaults.
    pub fn load() -> Self {
        let path = Self::default_path();
        Self::load_from(&path).unwrap_or_default()
    }

    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let cfg: AppConfig = toml::from_str(&content)?;
        Ok(cfg)
    }

    /// Persist current config to disk.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::default_path();
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Resolve the install prefix (expand tilde).
    pub fn resolved_prefix(&self) -> String {
        shellexpand::tilde(&self.install_prefix).into_owned()
    }

    /// Resolve the models directory.
    pub fn resolved_models_dir(&self) -> String {
        shellexpand::tilde(&self.models_dir).into_owned()
    }
}
