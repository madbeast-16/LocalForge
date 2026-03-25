use crate::models::ModelEntry;

/// Return the curated model registry, a static list of well-known GGUF models
/// grouped by VRAM tier. This avoids any network calls at startup.
pub fn curated_models() -> Vec<ModelEntry> {
    vec![
        // --- Tier 1: >= 16 GB VRAM ---
        ModelEntry {
            name: "Qwen2.5-14B-Instruct".into(),
            repo: "Qwen/Qwen2.5-14B-Instruct-GGUF".into(),
            params: "14B".into(),
            quantization: "Q5_K_M".into(),
            size_label: "~11 GB".into(),
            context_length: "32K".into(),
            min_vram_mb: 12288,
            min_ram_mb: 16384,
            description: "High-quality 14B instruction model, excellent reasoning".into(),
            estimated_toks: "~15 tok/s".into(),
        },
        ModelEntry {
            name: "Llama-3.1-70B-Instruct".into(),
            repo: "bartowski/Meta-Llama-3.1-70B-Instruct-GGUF".into(),
            params: "70B".into(),
            quantization: "Q4_K_M".into(),
            size_label: "~40 GB".into(),
            context_length: "128K".into(),
            min_vram_mb: 40960,
            min_ram_mb: 65536,
            description: "Meta flagship 70B, state-of-art open model".into(),
            estimated_toks: "~5 tok/s".into(),
        },
        // --- Tier 2: >= 8 GB VRAM ---
        ModelEntry {
            name: "Qwen2.5-7B-Instruct".into(),
            repo: "Qwen/Qwen2.5-7B-Instruct-GGUF".into(),
            params: "7B".into(),
            quantization: "Q5_K_M".into(),
            size_label: "~5.5 GB".into(),
            context_length: "32K".into(),
            min_vram_mb: 6144,
            min_ram_mb: 8192,
            description: "Balanced 7B model with strong multilingual support".into(),
            estimated_toks: "~30 tok/s".into(),
        },
        ModelEntry {
            name: "Llama-3.1-8B-Instruct".into(),
            repo: "bartowski/Meta-Llama-3.1-8B-Instruct-GGUF".into(),
            params: "8B".into(),
            quantization: "Q5_K_M".into(),
            size_label: "~5.7 GB".into(),
            context_length: "128K".into(),
            min_vram_mb: 6144,
            min_ram_mb: 8192,
            description: "Meta 8B, excellent general-purpose model".into(),
            estimated_toks: "~28 tok/s".into(),
        },
        ModelEntry {
            name: "Mistral-7B-Instruct-v0.3".into(),
            repo: "bartowski/Mistral-7B-Instruct-v0.3-GGUF".into(),
            params: "7B".into(),
            quantization: "Q5_K_M".into(),
            size_label: "~5.1 GB".into(),
            context_length: "32K".into(),
            min_vram_mb: 6144,
            min_ram_mb: 8192,
            description: "Fast and efficient 7B instruction model".into(),
            estimated_toks: "~32 tok/s".into(),
        },
        // --- Tier 3: >= 6 GB VRAM ---
        ModelEntry {
            name: "Phi-3.5-mini-instruct".into(),
            repo: "bartowski/Phi-3.5-mini-instruct-GGUF".into(),
            params: "3.8B".into(),
            quantization: "Q5_K_M".into(),
            size_label: "~2.8 GB".into(),
            context_length: "128K".into(),
            min_vram_mb: 4096,
            min_ram_mb: 6144,
            description: "Microsoft compact model, great for constrained hardware".into(),
            estimated_toks: "~45 tok/s".into(),
        },
        ModelEntry {
            name: "Qwen2.5-3B-Instruct".into(),
            repo: "Qwen/Qwen2.5-3B-Instruct-GGUF".into(),
            params: "3B".into(),
            quantization: "Q5_K_M".into(),
            size_label: "~2.4 GB".into(),
            context_length: "32K".into(),
            min_vram_mb: 3072,
            min_ram_mb: 4096,
            description: "Compact 3B model, punches above its weight".into(),
            estimated_toks: "~50 tok/s".into(),
        },
        // --- Tier 4: >= 4 GB VRAM / low-end ---
        ModelEntry {
            name: "Qwen2.5-1.5B-Instruct".into(),
            repo: "Qwen/Qwen2.5-1.5B-Instruct-GGUF".into(),
            params: "1.5B".into(),
            quantization: "Q5_K_M".into(),
            size_label: "~1.2 GB".into(),
            context_length: "32K".into(),
            min_vram_mb: 2048,
            min_ram_mb: 3072,
            description: "Tiny but capable 1.5B model".into(),
            estimated_toks: "~70 tok/s".into(),
        },
        ModelEntry {
            name: "TinyLlama-1.1B-Chat".into(),
            repo: "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF".into(),
            params: "1.1B".into(),
            quantization: "Q4_K_M".into(),
            size_label: "~0.7 GB".into(),
            context_length: "2K".into(),
            min_vram_mb: 1024,
            min_ram_mb: 2048,
            description: "Ultra-lightweight chat model, runs anywhere".into(),
            estimated_toks: "~90 tok/s".into(),
        },
        ModelEntry {
            name: "SmolLM2-1.7B-Instruct".into(),
            repo: "bartowski/SmolLM2-1.7B-Instruct-GGUF".into(),
            params: "1.7B".into(),
            quantization: "Q5_K_M".into(),
            size_label: "~1.3 GB".into(),
            context_length: "8K".into(),
            min_vram_mb: 2048,
            min_ram_mb: 3072,
            description: "HuggingFace compact model, efficient inference".into(),
            estimated_toks: "~65 tok/s".into(),
        },
    ]
}

/// Recommend models that fit within the user's hardware budget.
pub fn recommend(vram_mb: u64, ram_mb: u64) -> Vec<ModelEntry> {
    let all = curated_models();
    all.into_iter()
        .filter(|m| {
            if vram_mb > 0 {
                m.min_vram_mb <= vram_mb
            } else {
                m.min_ram_mb <= ram_mb
            }
        })
        .take(6)
        .collect()
}

/// Return a single best-fit recommendation.
pub fn best_fit(vram_mb: u64, ram_mb: u64) -> Option<ModelEntry> {
    let recs = recommend(vram_mb, ram_mb);
    recs.into_iter().next()
}
