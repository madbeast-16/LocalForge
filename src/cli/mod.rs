use anyhow::Result;

use crate::build;
use crate::config::backends::select_backend;
use crate::config::AppConfig;
use crate::hardware::{self, HardwareInfo};
use crate::models;

/// Run the full pipeline in headless / scripting mode (no TUI).
pub fn run_headless(
    config: &AppConfig,
    force_cpu: bool,
    force_cuda: bool,
    force_metal: bool,
    force_vulkan: bool,
) -> Result<()> {
    println!("llama-install v0.2.0  (headless mode)\n");

    // Step 1: Detect hardware
    println!("=== Hardware Detection ===");
    let hw = hardware::detect::detect_all()?;
    print_hw_summary(&hw);

    // Step 2: Select backend
    println!("\n=== Backend Selection ===");
    let backend = if let Some(name) = config.backend {
        println!("Using config backend: {}", name);
        crate::config::backends::make_backend(name, hw.gpu.compute_capability.clone())
    } else {
        select_backend(&hw, force_cpu, force_cuda, force_metal, force_vulkan)?
    };
    println!("Selected: {}", backend.description);
    println!("CMake flags: {}", backend.cmake_flags.join(" "));

    // Step 3: Build
    println!("\n=== Building llama.cpp ===");
    let prefix = config.resolved_prefix();
    let blog = build::run_build(&prefix, &hw, &backend)?;
    for line in &blog.lines {
        println!("  {}", line);
    }

    // Step 4: Model recommendation
    println!("\n=== Model Recommendations ===");
    let recs = models::registry::recommend(hw.total_vram_mb(), hw.total_memory_mb());
    if recs.is_empty() {
        println!("  No specific recommendations. Consider a Q4_K_M quantized model.");
    } else {
        for m in &recs {
            println!(
                "  - {} [{}]  {}  ({})",
                m.name, m.quantization, m.size_label, m.repo
            );
        }
    }

    println!("\nDone.");
    Ok(())
}

/// Detect-only command for headless use.
pub fn run_detect() -> Result<()> {
    let hw = hardware::detect::detect_all()?;
    print_hw_summary(&hw);
    Ok(())
}

/// Models-only command for headless use.
pub fn run_models(show_all: bool) -> Result<()> {
    let hw = hardware::detect::detect_all()?;
    let recs = if show_all {
        models::registry::curated_models()
    } else {
        models::registry::recommend(hw.total_vram_mb(), hw.total_memory_mb())
    };

    println!("=== Model Recommendations ===\n");
    println!(
        "  Hardware: {} VRAM  |  {} RAM\n",
        HardwareInfo::format_memory(hw.gpu.vram_mb),
        HardwareInfo::format_memory(hw.memory.total_mb),
    );

    if recs.is_empty() {
        println!("  No recommendations for your hardware.");
    } else {
        for m in &recs {
            println!(
                "  - {} [{}]  {}  ({})",
                m.name, m.quantization, m.size_label, m.repo
            );
            println!("    {}", m.description);
        }
    }
    Ok(())
}

/// Search HuggingFace in headless mode.
pub fn run_search(query: &str) -> Result<()> {
    println!("Searching HuggingFace for '{}'...\n", query);
    let results = models::huggingface::search_models(query)?;
    if results.is_empty() {
        println!("  No results found.");
    } else {
        for r in &results {
            println!("  {} ({} downloads)", r.id, r.downloads);
        }
    }
    Ok(())
}

fn print_hw_summary(hw: &HardwareInfo) {
    println!("  CPU:   {} ({} cores)", hw.cpu.name, hw.cpu.cores_logical);
    println!("  GPU:   {} ({})", hw.gpu.name, hw.gpu.vendor.as_str());
    println!("  VRAM:  {}", HardwareInfo::format_memory(hw.gpu.vram_mb));
    println!(
        "  RAM:   {} ({} available)",
        HardwareInfo::format_memory(hw.memory.total_mb),
        HardwareInfo::format_memory(hw.memory.available_mb),
    );
    println!("  OS:    {}", hw.os);
    if hw.cuda_available {
        println!(
            "  CUDA:  {}",
            hw.cuda_version.as_deref().unwrap_or("Available")
        );
    }
    if hw.rocm_available {
        println!("  ROCm:  Available");
    }
}
