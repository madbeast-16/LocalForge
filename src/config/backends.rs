use crate::config::cmake::generate_cmake_flags;
use crate::config::{Backend, BackendName};
use crate::hardware::{GpuVendor, HardwareInfo};
use anyhow::Result;

/// Auto-select the best backend based on detected hardware and optional overrides.
pub fn select_backend(
    hw: &HardwareInfo,
    force_cpu: bool,
    force_cuda: bool,
    force_metal: bool,
    force_vulkan: bool,
) -> Result<Backend> {
    if force_cpu {
        return Ok(make_backend(BackendName::CpuOnly, None));
    }
    if force_metal {
        return Ok(make_backend(BackendName::Metal, None));
    }
    if force_cuda {
        let arch = map_compute_capability(&hw.gpu.compute_capability);
        return Ok(make_backend(BackendName::Cuda, arch));
    }
    if force_vulkan {
        return Ok(make_backend(BackendName::Vulkan, None));
    }
    auto_select(hw)
}

/// Auto-select backend from hardware info alone (no overrides).
pub fn auto_select(hw: &HardwareInfo) -> Result<Backend> {
    if hw.cuda_available && hw.gpu.vendor == GpuVendor::Nvidia {
        let arch = map_compute_capability(&hw.gpu.compute_capability);
        return Ok(make_backend(BackendName::Cuda, arch));
    }
    if hw.rocm_available && hw.gpu.vendor == GpuVendor::Amd {
        return Ok(make_backend(BackendName::Hip, None));
    }
    if hw.gpu.vendor == GpuVendor::Apple {
        return Ok(make_backend(BackendName::Metal, None));
    }
    if hw.gpu.vendor == GpuVendor::Intel || hw.gpu.vendor == GpuVendor::Amd {
        return Ok(make_backend(BackendName::Vulkan, None));
    }
    Ok(make_backend(BackendName::CpuOnly, None))
}

/// Build a `Backend` from a name.
pub fn make_backend(name: BackendName, cuda_arch: Option<String>) -> Backend {
    let cmake_flags = generate_cmake_flags(name, cuda_arch.as_deref());
    Backend {
        name,
        description: name.as_str().to_string(),
        cmake_flags,
        cuda_arch,
    }
}

/// Return the list of backends available on this hardware.
pub fn available_backends(hw: &HardwareInfo) -> Vec<BackendName> {
    let mut backends = Vec::new();
    if hw.cuda_available && hw.gpu.vendor == GpuVendor::Nvidia {
        backends.push(BackendName::Cuda);
    }
    if hw.rocm_available && hw.gpu.vendor == GpuVendor::Amd {
        backends.push(BackendName::Hip);
    }
    if hw.gpu.vendor == GpuVendor::Apple {
        backends.push(BackendName::Metal);
    }
    if hw.gpu.vendor != GpuVendor::Unknown {
        backends.push(BackendName::Vulkan);
    }
    backends.push(BackendName::OpenBlas);
    backends.push(BackendName::CpuOnly);
    backends
}

pub fn map_compute_capability(cc: &Option<String>) -> Option<String> {
    cc.as_ref().map(|v| v.replace(".", ""))
}
