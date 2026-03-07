use crate::hardware::{CpuInfo, HardwareInfo, MemoryInfo};
use std::process::Command;
use sysinfo::System;

pub fn detect_cpu() -> CpuInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let name = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());

    let cores_physical = System::physical_core_count().unwrap_or(0);
    let cores_logical = sys.cpus().len();

    let vendor = sys
        .cpus()
        .first()
        .map(|c| c.vendor_id().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let architecture = std::env::consts::ARCH.to_string();

    CpuInfo {
        name,
        cores_physical,
        cores_logical,
        vendor,
        architecture,
    }
}

pub fn detect_memory() -> MemoryInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_mb = sys.total_memory() / (1024 * 1024);
    let available_mb = sys.available_memory() / (1024 * 1024);

    MemoryInfo {
        total_mb,
        available_mb,
    }
}

pub fn detect_os() -> String {
    if cfg!(target_os = "linux") {
        if std::path::Path::new("/proc/version").exists() {
            if let Ok(contents) = std::fs::read_to_string("/proc/version") {
                if contents.contains("Microsoft") || contents.contains("WSL") {
                    return "Linux (WSL)".to_string();
                }
            }
        }
        "Linux".to_string()
    } else if cfg!(target_os = "macos") {
        "macOS".to_string()
    } else if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else {
        std::env::consts::OS.to_string()
    }
}

pub fn detect_cuda() -> (bool, Option<String>) {
    if let Ok(output) = Command::new("nvcc").arg("--version").output() {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("release") {
                    let version = line
                        .split("release")
                        .nth(1)
                        .map(|v| v.trim().split(',').next().unwrap_or("").trim().to_string());
                    return (true, version);
                }
            }
            return (true, None);
        }
    }
    (false, None)
}

pub fn detect_rocm() -> bool {
    if let Ok(rocm_path) = std::env::var("ROCM_PATH") {
        return std::path::Path::new(&rocm_path).exists();
    }
    std::path::Path::new("/opt/rocm").exists() || std::path::Path::new("/usr/local/rocm").exists()
}

pub fn detect_all() -> anyhow::Result<HardwareInfo> {
    let cpu = detect_cpu();
    let memory = detect_memory();
    let os = detect_os();
    let (cuda_available, cuda_version) = detect_cuda();
    let rocm_available = detect_rocm();
    let gpu = crate::hardware::gpu::detect_gpu(cuda_available);

    Ok(HardwareInfo {
        cpu,
        gpu,
        memory,
        os,
        cuda_available,
        cuda_version,
        rocm_available,
    })
}
