use crate::models::ModelEntry;

/// Fit classification for a model against available hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitLevel {
    /// Model fits entirely in VRAM with room to spare
    Perfect,
    /// Model fits in VRAM with tight margin
    Good,
    /// Model may require partial CPU offloading
    Marginal,
    /// Model is too large for available resources
    TooTight,
}

impl FitLevel {
    pub fn label(&self) -> &'static str {
        match self {
            FitLevel::Perfect => "★ Perfect",
            FitLevel::Good => "● Good",
            FitLevel::Marginal => "◐ Marginal",
            FitLevel::TooTight => "✗ Too Tight",
        }
    }

    pub fn short(&self) -> &'static str {
        match self {
            FitLevel::Perfect => "Perfect",
            FitLevel::Good => "Good",
            FitLevel::Marginal => "Marginal",
            FitLevel::TooTight => "Too Tight",
        }
    }
}

/// Score how well a model fits the user's hardware.
///
/// Uses VRAM as the primary constraint. Falls back to RAM for CPU-only setups.
/// The scoring considers model size vs available memory with margins for
/// KV cache, context, and OS overhead.
pub fn score_fit(model: &ModelEntry, vram_mb: u64, ram_mb: u64) -> FitLevel {
    // Use VRAM if available, otherwise fall back to RAM
    let available = if vram_mb > 0 { vram_mb } else { ram_mb };
    let required = if vram_mb > 0 {
        model.min_vram_mb
    } else {
        model.min_ram_mb
    };

    if available == 0 || required == 0 {
        return FitLevel::Marginal;
    }

    let ratio = available as f64 / required as f64;

    if ratio >= 1.5 {
        FitLevel::Perfect
    } else if ratio >= 1.1 {
        FitLevel::Good
    } else if ratio >= 0.8 {
        FitLevel::Marginal
    } else {
        FitLevel::TooTight
    }
}

/// Score all models in a list and return them paired with fit levels.
pub fn score_all(models: &[ModelEntry], vram_mb: u64, ram_mb: u64) -> Vec<(ModelEntry, FitLevel)> {
    models
        .iter()
        .map(|m| (m.clone(), score_fit(m, vram_mb, ram_mb)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_model(min_vram: u64, min_ram: u64) -> ModelEntry {
        ModelEntry {
            name: "test".into(),
            repo: "test/test".into(),
            params: "7B".into(),
            quantization: "Q4_K_M".into(),
            size_label: "~5 GB".into(),
            context_length: "8K".into(),
            min_vram_mb: min_vram,
            min_ram_mb: min_ram,
            description: "test model".into(),
            estimated_toks: "~30 tok/s".into(),
        }
    }

    #[test]
    fn test_perfect_fit() {
        let m = test_model(4096, 8192);
        assert_eq!(score_fit(&m, 8192, 16384), FitLevel::Perfect);
    }

    #[test]
    fn test_good_fit() {
        let m = test_model(6000, 8192);
        assert_eq!(score_fit(&m, 7000, 16384), FitLevel::Good);
    }

    #[test]
    fn test_marginal_fit() {
        let m = test_model(6000, 8192);
        assert_eq!(score_fit(&m, 5500, 16384), FitLevel::Marginal);
    }

    #[test]
    fn test_too_tight() {
        let m = test_model(12288, 16384);
        assert_eq!(score_fit(&m, 4096, 8192), FitLevel::TooTight);
    }

    #[test]
    fn test_cpu_fallback() {
        let m = test_model(6000, 4096);
        // VRAM = 0 means CPU-only, so uses RAM
        assert_eq!(score_fit(&m, 0, 16384), FitLevel::Perfect);
    }
}
