use super::kv_cache::{KvCacheConfig, KV_DTYPE};

#[derive(Debug, Clone)]
pub struct SlotAdvisorInput {
    pub model_size_mb: usize,
    pub total_vram_mb: usize,
    pub context_length: usize,
    pub kv_dtype: KV_DTYPE,
    pub target_concurrent_users: usize,
}

#[derive(Debug, Clone)]
pub struct SlotRecommendation {
    pub parallel_slots: usize,
    pub ctx_size: usize,
    pub cache_type_k: KV_DTYPE,
    pub cache_type_v: KV_DTYPE,
    pub continuous_batching: bool,
    pub estimated_total_vram_mb: usize,
    pub safety_margin: f64,
}

pub struct SlotAdvisor;

impl SlotAdvisor {
    pub fn new() -> Self {
        Self
    }

    pub fn recommend(&self, input: &SlotAdvisorInput) -> SlotRecommendation {
        let safety_threshold = 0.85;
        let max_usable_vram = (input.total_vram_mb as f64 * safety_threshold) as usize;
        let model_required = input.model_size_mb;
        let available_for_kv = max_usable_vram.saturating_sub(model_required);

        if available_for_kv == 0 {
            return SlotRecommendation {
                parallel_slots: 1,
                ctx_size: 2048,
                cache_type_k: KV_DTYPE::Q8_0,
                cache_type_v: KV_DTYPE::Q8_0,
                continuous_batching: false,
                estimated_total_vram_mb: model_required,
                safety_margin: 0.0,
            };
        }

        let layers = 32;
        let heads = 32;
        let head_dim = 128;
        let batch_size = input.target_concurrent_users.max(1);

        let cache_config = KvCacheConfig {
            layers,
            heads,
            head_dim,
            context_length: input.context_length,
            dtype: input.kv_dtype,
            batch_size,
        };

        let kv_per_slot_mb = super::kv_cache::calculate_kv_cache_size_mb(&cache_config);
        let slots = (available_for_kv / kv_per_slot_mb.max(1)).max(1);

        let total_vram = model_required + (kv_per_slot_mb * slots);
        let margin = 1.0 - (total_vram as f64 / input.total_vram_mb as f64);

        SlotRecommendation {
            parallel_slots: slots,
            ctx_size: input.context_length,
            cache_type_k: input.kv_dtype,
            cache_type_v: input.kv_dtype,
            continuous_batching: slots > 2,
            estimated_total_vram_mb: total_vram,
            safety_margin: margin,
        }
    }
}
