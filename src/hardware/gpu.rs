use crate::hardware::{GpuInfo, GpuVendor};
use std::process::Command;

pub fn detect_gpu(cuda_available: bool) -> GpuInfo {
    if let Some(nvidia_info) = detect_nvidia(cuda_available) {
        return nvidia_info;
    }
    if let Some(amd_info) = detect_amd() {
        return amd_info;
    }
    if let Some(intel_info) = detect_intel() {
        return intel_info;
    }
    if cfg!(target_os = "macos") {
        if let Some(apple_info) = detect_apple_metal() {
            return apple_info;
        }
    }

    GpuInfo {
        name: "No dedicated GPU detected".to_string(),
        vendor: GpuVendor::Unknown,
        vram_mb: 0,
        driver_version: None,
        cuda_support: false,
        compute_capability: None,
    }
}

fn detect_nvidia(cuda_available: bool) -> Option<GpuInfo> {
    if std::env::consts::OS != "linux" && std::env::consts::OS != "windows" && !cuda_available {
        return None;
    }

    if let Some(nvml_info) = detect_nvidia_nvml(cuda_available) {
        return Some(nvml_info);
    }

    detect_nvidia_smi(cuda_available)
}

fn detect_nvidia_nvml(cuda_available: bool) -> Option<GpuInfo> {
    let nvml = nvml_wrapper::Nvml::init().ok()?;
    let device = nvml.device_by_index(0).ok()?;
    let name = device.name().ok()?.trim().to_string();
    let vram = device.memory_info().ok()?.total / (1024 * 1024);
    let compute_capability = device
        .cuda_compute_capability()
        .ok()
        .map(|cc| format!("{}.{}", cc.major, cc.minor));
    let driver = get_nvidia_driver_version();

    Some(GpuInfo {
        name,
        vendor: GpuVendor::Nvidia,
        vram_mb: vram,
        driver_version: driver,
        cuda_support: cuda_available,
        compute_capability,
    })
}

fn detect_nvidia_smi(cuda_available: bool) -> Option<GpuInfo> {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total", "--format=csv,noheader"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?;
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 2 {
        return None;
    }

    let name = parts[0].trim().to_string();
    let vram_str = parts[1].trim().replace(" MiB", "").replace("MB", "");
    let vram: u64 = vram_str.parse().ok()?;
    let driver = get_nvidia_driver_version();
    let compute_capability = detect_cuda_capability();

    Some(GpuInfo {
        name,
        vendor: GpuVendor::Nvidia,
        vram_mb: vram,
        driver_version: driver,
        cuda_support: cuda_available,
        compute_capability,
    })
}

fn detect_cuda_capability() -> Option<String> {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=compute_cap", "--format=csv,noheader"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let cc = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if cc.is_empty() {
        return None;
    }
    Some(cc)
}

fn detect_amd() -> Option<GpuInfo> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let output = Command::new("lspci").output().ok()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("amd")
            && (line_lower.contains("radeon") || line_lower.contains("graphics"))
        {
            let name = line.split(':').nth(2)?.trim().to_string();
            let vram = detect_amd_vram();
            return Some(GpuInfo {
                name,
                vendor: GpuVendor::Amd,
                vram_mb: vram,
                driver_version: None,
                cuda_support: false,
                compute_capability: None,
            });
        }
    }
    None
}

fn detect_amd_vram() -> u64 {
    // Try reading from DRM sysfs first (accurate for dedicated GPUs)
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let mem_path = entry.path().join("device/mem_info_vram_total");
            if mem_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&mem_path) {
                    if let Ok(bytes) = content.trim().parse::<u64>() {
                        return bytes / (1024 * 1024);
                    }
                }
            }
        }
    }
    // Fallback: conservative estimate
    4096
}

fn detect_intel() -> Option<GpuInfo> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let output = Command::new("lspci").output().ok()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("intel")
            && (line_lower.contains("graphics")
                || line_lower.contains("uhd")
                || line_lower.contains("iris"))
        {
            let name = line.split(':').nth(2)?.trim().to_string();
            return Some(GpuInfo {
                name,
                vendor: GpuVendor::Intel,
                vram_mb: 2048,
                driver_version: None,
                cuda_support: false,
                compute_capability: None,
            });
        }
    }
    None
}

fn detect_apple_metal() -> Option<GpuInfo> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    let output = Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
        .ok()?;

    let cpu_name = String::from_utf8_lossy(&output.stdout);
    if cpu_name.contains("Apple") {
        return Some(GpuInfo {
            name: "Apple Silicon (Metal)".to_string(),
            vendor: GpuVendor::Apple,
            vram_mb: detect_apple_unified_memory(),
            driver_version: None,
            cuda_support: false,
            compute_capability: None,
        });
    }
    None
}

fn detect_apple_unified_memory() -> u64 {
    if cfg!(target_os = "macos") {
        if let Ok(output) = Command::new("sysctl").args(["-n", "hw.memsize"]).output() {
            if let Ok(memory) = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<u64>()
            {
                return memory / (1024 * 1024);
            }
        }
    }
    8192
}

fn get_nvidia_driver_version() -> Option<String> {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=driver_version", "--format=csv,noheader"])
        .output()
        .ok()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        return Some(version.trim().to_string());
    }
    None
}
