/// Quantization recommendations based on available hardware.

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantLevel {
    Q4_K_M,
    Q5_K_M,
    Q6_K,
    Q8_0,
    F16,
}

impl QuantLevel {
    pub fn label(&self) -> &'static str {
        match self {
            QuantLevel::Q4_K_M => "Q4_K_M",
            QuantLevel::Q5_K_M => "Q5_K_M",
            QuantLevel::Q6_K => "Q6_K",
            QuantLevel::Q8_0 => "Q8_0",
            QuantLevel::F16 => "F16",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            QuantLevel::Q4_K_M => "Best trade-off between quality and size (recommended)",
            QuantLevel::Q5_K_M => "Good balance, slightly better quality than Q4",
            QuantLevel::Q6_K => "High quality, needs more VRAM",
            QuantLevel::Q8_0 => "Near-lossless, large memory footprint",
            QuantLevel::F16 => "Full precision, maximum quality (2x size)",
        }
    }

    /// Approximate multiplier vs raw parameter count (bytes per param).
    pub fn bytes_per_param(&self) -> f64 {
        match self {
            QuantLevel::Q4_K_M => 0.55,
            QuantLevel::Q5_K_M => 0.65,
            QuantLevel::Q6_K => 0.75,
            QuantLevel::Q8_0 => 1.0,
            QuantLevel::F16 => 2.0,
        }
    }
}

/// Recommend a quantization level based on available VRAM.
///
/// The heuristic is:
/// - >= 48 GB VRAM: F16 (full precision)
/// - >= 24 GB VRAM: Q8_0 (near-lossless)
/// - >= 16 GB VRAM: Q6_K (high quality)
/// - >= 8 GB VRAM:  Q5_K_M (balanced)
/// - < 8 GB VRAM:   Q4_K_M (best trade-off)
pub fn recommend_quant(vram_mb: u64) -> QuantLevel {
    if vram_mb >= 49152 {
        QuantLevel::F16
    } else if vram_mb >= 24576 {
        QuantLevel::Q8_0
    } else if vram_mb >= 16384 {
        QuantLevel::Q6_K
    } else if vram_mb >= 8192 {
        QuantLevel::Q5_K_M
    } else {
        QuantLevel::Q4_K_M
    }
}

/// Return all quantization levels from lowest to highest quality.
pub fn all_quants() -> &'static [QuantLevel] {
    &[
        QuantLevel::Q4_K_M,
        QuantLevel::Q5_K_M,
        QuantLevel::Q6_K,
        QuantLevel::Q8_0,
        QuantLevel::F16,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_vram_recommends_q4() {
        assert_eq!(recommend_quant(4096), QuantLevel::Q4_K_M);
    }

    #[test]
    fn test_mid_vram_recommends_q5() {
        assert_eq!(recommend_quant(12288), QuantLevel::Q5_K_M);
    }

    #[test]
    fn test_high_vram_recommends_q8() {
        assert_eq!(recommend_quant(24576), QuantLevel::Q8_0);
    }

    #[test]
    fn test_very_high_vram_recommends_f16() {
        assert_eq!(recommend_quant(49152), QuantLevel::F16);
    }
}
