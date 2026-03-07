use crate::config::Backend;
use crate::hardware::HardwareInfo;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use sysinfo::System;

/// Phases of the build pipeline, used to report progress to the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildPhase {
    Clone,
    Configure,
    Compile,
    Verify,
    Done,
}

impl BuildPhase {
    pub fn label(&self) -> &'static str {
        match self {
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
            phase: BuildPhase::Clone,
            success: false,
        }
    }

    fn log(&mut self, msg: impl Into<String>) {
        self.lines.push(msg.into());
    }
}

/// Run the full build pipeline synchronously.
/// Returns the build log for display.
pub fn run_build(prefix: &str, _hw: &HardwareInfo, backend: &Backend) -> Result<BuildLog> {
    let mut blog = BuildLog::new();

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
    blog.log("Running CMake configuration...");
    run_cmake(&source_dir, &build_dir, &backend.cmake_flags)?;
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
        .context("Failed to execute git clone")?;

    if !output.status.success() {
        bail!(
            "Git clone failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn run_cmake(source_dir: &Path, build_dir: &Path, flags: &[String]) -> Result<()> {
    if build_dir.exists() {
        fs::remove_dir_all(build_dir).ok();
    }
    fs::create_dir_all(build_dir).context("Failed to create build directory")?;

    let mut cmd = Command::new("cmake");
    cmd.args(["-S", source_dir.to_str().unwrap_or(".")]);
    cmd.args(["-B", build_dir.to_str().unwrap_or(".")]);
    cmd.args(flags);

    let output = cmd.output().context("CMake configuration failed")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("CMake configuration failed:\n{}", stderr);
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
        .context("Build failed")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Build failed:\n{}", stderr);
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
