pub mod download;
pub mod huggingface;
pub mod registry;

use serde::{Deserialize, Serialize};

/// A model entry in the curated registry or from HuggingFace search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub name: String,
    pub repo: String,
    pub quantization: String,
    pub size_label: String,
    pub min_vram_mb: u64,
    pub min_ram_mb: u64,
    pub description: String,
}

impl std::fmt::Display for ModelEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} [{}] ({})",
            self.name, self.quantization, self.size_label
        )
    }
}

/// Result from a HuggingFace search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub downloads: u64,
}
