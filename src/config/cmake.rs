use crate::config::BackendName;

/// Generate CMake flags for the given backend.
pub fn generate_cmake_flags(backend: BackendName, cuda_arch: Option<&str>) -> Vec<String> {
    let mut flags = vec!["-DCMAKE_BUILD_TYPE=Release".into()];

    match backend {
        BackendName::Cuda => {
            flags.push("-DGGML_CUDA=ON".into());
            flags.push("-DGGML_CUDA_F16=ON".into());
            if let Some(arch) = cuda_arch {
                // CMake requires an integer (e.g. "89", "120"), not a dotted
                // version like "8.9" or "12.0".  Strip any dots defensively.
                let arch_clean = arch.replace('.', "");
                flags.push(format!("-DCMAKE_CUDA_ARCHITECTURES={arch_clean}"));
            }
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
