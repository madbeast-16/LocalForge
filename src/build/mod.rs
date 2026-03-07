use crate::config::{Backend, BackendName};
use crate::hardware::HardwareInfo;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use sysinfo::System;

/// Phases of the build pipeline, used to report progress to the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildPhase {
    Preflight,
    Clone,
    Configure,
    Compile,
    Verify,
    Done,
}

impl BuildPhase {
    pub fn label(&self) -> &'static str {
        match self {
            BuildPhase::Preflight => "Checking prerequisites",
            BuildPhase::Clone => "Cloning llama.cpp",
            BuildPhase::Configure => "CMake configuration",
            BuildPhase::Compile => "Compiling",
            BuildPhase::Verify => "Verifying build",
            BuildPhase::Done => "Complete",
        }
    }
}

/// Accumulated build log lines.
pub struct BuildLog {
    pub lines: Vec<String>,
    pub phase: BuildPhase,
    pub success: bool,
}

impl BuildLog {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            phase: BuildPhase::Preflight,
            success: false,
        }
    }

    fn log(&mut self, msg: impl Into<String>) {
        self.lines.push(msg.into());
    }
}

// ---------------------------------------------------------------------------
// Prerequisite checks
// ---------------------------------------------------------------------------

/// Check whether an executable is available on PATH and return its version string.
fn check_command(name: &str, version_arg: &str) -> Result<String, String> {
    match Command::new(name).arg(version_arg).output() {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout);
            let first_line = text.lines().next().unwrap_or("").to_string();
            if first_line.is_empty() {
                // Some tools print version to stderr (e.g. cmake)
                let stderr_text = String::from_utf8_lossy(&output.stderr);
                let first_stderr = stderr_text.lines().next().unwrap_or("").to_string();
                if first_stderr.is_empty() {
                    Ok(format!("{} (version unknown)", name))
                } else {
                    Ok(first_stderr)
                }
            } else {
                Ok(first_line)
            }
        }
        Err(_) => Err(format!("'{}' not found on PATH", name)),
    }
}

/// Return platform-specific install instructions for missing tools.
fn install_hint(tool: &str) -> String {
    let os = std::env::consts::OS;
    match (tool, os) {
        ("cmake", "linux") => {
            "Install cmake:\n  Ubuntu/Debian: sudo apt install cmake\n  Fedora:        sudo dnf install cmake\n  Arch:          sudo pacman -S cmake".to_string()
        }
        ("cmake", "macos") => "Install cmake:\n  brew install cmake".to_string(),
        ("cmake", "windows") => {
            "Install cmake:\n  winget install Kitware.CMake\n  or download from https://cmake.org/download/".to_string()
        }
        ("git", "linux") => {
            "Install git:\n  Ubuntu/Debian: sudo apt install git\n  Fedora:        sudo dnf install git\n  Arch:          sudo pacman -S git".to_string()
        }
        ("git", "macos") => {
            "Install git:\n  xcode-select --install\n  or: brew install git".to_string()
        }
        ("git", "windows") => {
            "Install git:\n  winget install Git.Git\n  or download from https://git-scm.com/".to_string()
        }
        ("cc", "linux") => {
            "Install a C/C++ compiler:\n  Ubuntu/Debian: sudo apt install build-essential\n  Fedora:        sudo dnf install gcc gcc-c++ make\n  Arch:          sudo pacman -S base-devel".to_string()
        }
        ("cc", "macos") => {
            "Install a C/C++ compiler:\n  xcode-select --install".to_string()
        }
        ("cc", "windows") => {
            "Install a C/C++ compiler:\n  Install Visual Studio Build Tools from https://visualstudio.microsoft.com/downloads/".to_string()
        }
        _ => format!("Please install '{}' and ensure it is on your PATH.", tool),
    }
}

/// Check that a C or C++ compiler is available.
fn check_c_compiler() -> Result<String, String> {
    // Try common compiler names in order of preference
    let candidates = if cfg!(target_os = "windows") {
        vec!["cl", "gcc", "clang"]
    } else {
        vec!["cc", "gcc", "clang"]
    };

    for name in &candidates {
        if let Ok(version) = check_command(name, "--version") {
            return Ok(version);
        }
    }

    Err("No C/C++ compiler found (tried cc, gcc, clang)".to_string())
}

/// Backend-specific prerequisite hint.
fn backend_hint(backend: BackendName) -> Option<&'static str> {
    match backend {
        BackendName::Cuda => Some(
            "CUDA backend requires the NVIDIA CUDA Toolkit.\n  \
             Download from: https://developer.nvidia.com/cuda-downloads\n  \
             Verify with:   nvcc --version",
        ),
        BackendName::Hip => Some(
            "HIP/ROCm backend requires AMD ROCm and clang.\n  \
             Install from: https://rocm.docs.amd.com/\n  \
             Verify with:  clang --version && hipcc --version",
        ),
        BackendName::Metal => Some(
            "Metal backend requires Xcode Command Line Tools.\n  \
             Install with: xcode-select --install",
        ),
        BackendName::Vulkan => Some(
            "Vulkan backend requires the Vulkan SDK.\n  \
             Download from: https://vulkan.lunarg.com/sdk/home\n  \
             Verify with:   vulkaninfo --summary",
        ),
        BackendName::OpenBlas => Some(
            "OpenBLAS backend requires the OpenBLAS library.\n  \
             Ubuntu/Debian: sudo apt install libopenblas-dev\n  \
             Fedora:        sudo dnf install openblas-devel\n  \
             macOS:         brew install openblas",
        ),
        BackendName::CpuOnly => None,
    }
}

/// Run all prerequisite checks. Returns a list of problems (empty = all good).
fn preflight_checks(blog: &mut BuildLog, _backend: &Backend) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();

    // Check git
    match check_command("git", "--version") {
        Ok(v) => blog.log(format!("  git:      {}", v)),
        Err(e) => {
            blog.log(format!("  git:      MISSING"));
            problems.push(format!("{}\n\n{}", e, install_hint("git")));
        }
    }

    // Check cmake
    match check_command("cmake", "--version") {
        Ok(v) => blog.log(format!("  cmake:    {}", v)),
        Err(e) => {
            blog.log(format!("  cmake:    MISSING"));
            problems.push(format!("{}\n\n{}", e, install_hint("cmake")));
        }
    }

    // Check C/C++ compiler
    match check_c_compiler() {
        Ok(v) => blog.log(format!("  compiler: {}", v)),
        Err(e) => {
            blog.log(format!("  compiler: MISSING"));
            problems.push(format!("{}\n\n{}", e, install_hint("cc")));
        }
    }

    problems
}

// ---------------------------------------------------------------------------
// Build pipeline
// ---------------------------------------------------------------------------

/// Run the full build pipeline synchronously.
/// Returns the build log for display.
pub fn run_build(prefix: &str, _hw: &HardwareInfo, backend: &Backend) -> Result<BuildLog> {
    let mut blog = BuildLog::new();

    // Phase 0: Preflight checks
    blog.phase = BuildPhase::Preflight;
    blog.log("Checking build prerequisites...");
    let problems = preflight_checks(&mut blog, backend);
    if !problems.is_empty() {
        let mut msg = String::from("Missing required build tools:\n\n");
        for (i, p) in problems.iter().enumerate() {
            msg.push_str(&format!("{}. {}\n\n", i + 1, p));
        }
        if let Some(hint) = backend_hint(backend.name) {
            msg.push_str(&format!("Backend note ({}):\n  {}\n", backend.name, hint));
        }
        bail!("{}", msg.trim_end());
    }
    blog.log("All prerequisites satisfied.");

    let install_dir = Path::new(prefix);
    let llama_dir = install_dir.join("share/llama.cpp");
    let build_dir = llama_dir.join("build");
    let source_dir = llama_dir.join("source");

    // Phase 1: Clone
    blog.phase = BuildPhase::Clone;
    if !source_dir.exists() {
        blog.log("Cloning llama.cpp repository...");
        clone_repo(&source_dir)?;
        blog.log("Clone complete.");
    } else {
        blog.log(format!(
            "Source already exists at {}, skipping clone.",
            source_dir.display()
        ));
    }

    // Phase 2: Configure
    blog.phase = BuildPhase::Configure;
    blog.log(format!(
        "Running CMake configuration ({})...",
        backend.description
    ));
    blog.log(format!("  Flags: {}", backend.cmake_flags.join(" ")));
    run_cmake(&source_dir, &build_dir, &backend.cmake_flags, backend.name)?;
    blog.log("CMake configuration successful.");

    // Phase 3: Compile
    blog.phase = BuildPhase::Compile;
    let jobs = parallel_jobs();
    blog.log(format!("Building with {} parallel jobs...", jobs));
    run_make(&build_dir, jobs)?;
    blog.log("Build successful.");

    // Phase 4: Verify
    blog.phase = BuildPhase::Verify;
    let found = verify_build(&build_dir);
    if found.is_empty() {
        blog.log("Warning: could not locate built binaries.");
    } else {
        blog.log(format!("Found binaries: {}", found.join(", ")));
    }

    blog.phase = BuildPhase::Done;
    blog.success = true;
    blog.log(format!(
        "Build complete. Binaries in {}/bin/",
        build_dir.display()
    ));
    Ok(blog)
}

fn clone_repo(dest: &Path) -> Result<()> {
    let output = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "https://github.com/ggerganov/llama.cpp.git",
            dest.to_str().unwrap_or("."),
        ])
        .output()
        .context("Failed to run 'git'. Is git installed and on your PATH?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Git clone failed (exit code {}):\n{}",
            output.status.code().unwrap_or(-1),
            stderr
        );
    }
    Ok(())
}

fn run_cmake(
    source_dir: &Path,
    build_dir: &Path,
    flags: &[String],
    backend_name: BackendName,
) -> Result<()> {
    if build_dir.exists() {
        fs::remove_dir_all(build_dir).ok();
    }
    fs::create_dir_all(build_dir).context("Failed to create build directory")?;

    let mut cmd = Command::new("cmake");
    cmd.args(["-S", source_dir.to_str().unwrap_or(".")]);
    cmd.args(["-B", build_dir.to_str().unwrap_or(".")]);
    cmd.args(flags);

    let output = cmd.output().context(
        "Failed to run 'cmake'. Is cmake installed and on your PATH?\n\
         Install it with:\n  \
         Ubuntu/Debian: sudo apt install cmake\n  \
         macOS:         brew install cmake\n  \
         Windows:       winget install Kitware.CMake",
    )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut msg = format!(
            "CMake configuration failed (exit code {}).\n\nBackend: {}\nFlags:   {}\n",
            output.status.code().unwrap_or(-1),
            backend_name,
            flags.join(" "),
        );

        if !stderr.is_empty() {
            msg.push_str(&format!("\n--- stderr ---\n{}\n", stderr.trim()));
        }
        if !stdout.is_empty() {
            msg.push_str(&format!("\n--- stdout ---\n{}\n", stdout.trim()));
        }

        // Add backend-specific troubleshooting
        if let Some(hint) = backend_hint(backend_name) {
            msg.push_str(&format!("\nHint:\n  {}\n", hint));
        }

        bail!("{}", msg.trim_end());
    }
    Ok(())
}

fn run_make(build_dir: &Path, jobs: usize) -> Result<()> {
    let output = Command::new("cmake")
        .args([
            "--build",
            build_dir.to_str().unwrap_or("."),
            "--config",
            "Release",
            "-j",
            &jobs.to_string(),
        ])
        .output()
        .context("Build failed: could not invoke 'cmake --build'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut msg = format!(
            "Compilation failed (exit code {}).\n",
            output.status.code().unwrap_or(-1),
        );

        if !stderr.is_empty() {
            msg.push_str(&format!("\n--- stderr ---\n{}\n", stderr.trim()));
        }
        if !stdout.is_empty() {
            // Only include last 50 lines of stdout to avoid overwhelming output
            let stdout_lines: Vec<&str> = stdout.trim().lines().collect();
            let tail: Vec<&str> = if stdout_lines.len() > 50 {
                stdout_lines[stdout_lines.len() - 50..].to_vec()
            } else {
                stdout_lines
            };
            msg.push_str(&format!(
                "\n--- stdout (last lines) ---\n{}\n",
                tail.join("\n")
            ));
        }

        bail!("{}", msg.trim_end());
    }
    Ok(())
}

fn verify_build(build_dir: &Path) -> Vec<String> {
    let bin_dir = build_dir.join("bin");
    let expected = ["llama-cli", "llama-server", "llama-quantize", "llama-bench"];
    expected
        .iter()
        .filter(|name| bin_dir.join(name).exists())
        .map(|s| s.to_string())
        .collect()
}

/// Determine the number of parallel build jobs.
pub fn parallel_jobs() -> usize {
    let sys = System::new_all();
    let cpus = sys.cpus().len();
    if cpus > 1 {
        cpus
    } else {
        1
    }
}
