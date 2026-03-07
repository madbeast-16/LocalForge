use crate::config::BackendName;
use std::process::Command;

/// Generate CMake flags for the given backend.
pub fn generate_cmake_flags(backend: BackendName, cuda_arch: Option<&str>) -> Vec<String> {
    let mut flags = vec!["-DCMAKE_BUILD_TYPE=Release".into()];

    match backend {
        BackendName::Cuda => {
            flags.push("-DGGML_CUDA=ON".into());
            flags.push("-DGGML_CUDA_F16=ON".into());
            let resolved = resolve_cuda_arch(cuda_arch);
            flags.push(format!("-DCMAKE_CUDA_ARCHITECTURES={resolved}"));
        }
        BackendName::Hip => {
            flags.push("-DGGML_HIPBLAS=ON".into());
            flags.push("-DCMAKE_C_COMPILER=clang".into());
            flags.push("-DCMAKE_CXX_COMPILER=clang++".into());
        }
        BackendName::Metal => {
            flags.push("-DGGML_METAL=ON".into());
            flags.push("-DGGML_METAL_EMBED_LIBRARY=ON".into());
        }
        BackendName::Vulkan => {
            flags.push("-DGGML_VULKAN=ON".into());
        }
        BackendName::OpenBlas => {
            flags.push("-DGGML_BLAS=ON".into());
            flags.push("-DGGML_BLAS_VENDOR=OpenBLAS".into());
        }
        BackendName::CpuOnly => {
            flags.push("-DGGML_NATIVE=ON".into());
        }
    }

    flags
}

/// Format a human-readable cmake invocation string.
pub fn format_cmake_command(source_dir: &str, build_dir: &str, flags: &[String]) -> String {
    let flag_str = flags
        .iter()
        .map(|f| format!("  {f}"))
        .collect::<Vec<_>>()
        .join(" \\\n");
    format!("cmake -S {source_dir} -B {build_dir} \\\n{flag_str}")
}

/// Resolve a CUDA architecture value that nvcc actually supports.
///
/// 1. Strips dots from the input (e.g. "12.0" -> "120").
/// 2. Queries `nvcc --list-gpu-arch` for the set of supported architectures.
/// 3. If the requested arch is supported, uses it.
/// 4. Otherwise falls back to the highest arch nvcc supports, or "native"
///    if we can't determine anything.
fn resolve_cuda_arch(requested: Option<&str>) -> String {
    let supported = query_nvcc_supported_archs();

    let arch = match requested {
        Some(raw) => raw.replace('.', ""),
        None => {
            // No arch detected — use "native" and let CMake figure it out.
            return "native".into();
        }
    };

    if supported.is_empty() {
        // nvcc not found or failed — "native" is the safest fallback.
        return "native".into();
    }

    if supported.contains(&arch) {
        return arch;
    }

    // The requested arch is newer than what this nvcc supports.
    // Fall back to the highest supported arch, or "native".
    if let Some(best) = supported.last() {
        best.clone()
    } else {
        "native".into()
    }
}

/// Run `nvcc --list-gpu-arch` and return a sorted list of numeric arch codes
/// (e.g. ["50", "52", "60", "70", "75", "80", "86", "89", "90"]).
fn query_nvcc_supported_archs() -> Vec<String> {
    let output = match Command::new("nvcc").arg("--list-gpu-arch").output() {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut archs: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            // Lines look like "compute_90" — extract the number.
            let line = line.trim();
            line.strip_prefix("compute_").map(|n| n.to_string())
        })
        .collect();

    archs.sort_by(|a, b| {
        a.parse::<u32>()
            .unwrap_or(0)
            .cmp(&b.parse::<u32>().unwrap_or(0))
    });
    archs.dedup();
    archs
}
