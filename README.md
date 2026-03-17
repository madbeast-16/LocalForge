# localforge

Local-first TUI tool that takes you from **zero to a fully working local LLM stack**.

**Async, cross-platform, zero-overhead. Production-grade Rust.**

## High-level Vision

LocalForge is a terminal-based application that guides you through:

1. **Setup Flow** (Wizard): Hardware detection → Backend selection → llama.cpp build → Model download 2. **Main Workspace**: Chat interface with streaming + Settings for server exposure

All steps are resumable and expertise-aware (Beginner/Intermediate/Expert).

## Features

### Setup & Installation
- **Hardware Detection** — Auto-detect CPU, GPU (NVIDIA/AMD/Apple), VRAM, CUDA, ROCm, WSL
- **Optimal Backend Selection** — Ranked recommendations (CUDA, Metal, ROCm, Vulkan, CPU)
- **Automated llama.cpp Build** — Async build with tuned CMake flags per backend
- **Optional llama-swap** — Hot-swap models without restarting server
- **Curated Model Registry** — Hardware-fitted GGUF recommendations
- **HuggingFace Search** — Async search with GGUF filtering
- **Robust Downloads** — Async downloads with resume support

### Main Workspace
- **Chat Interface** — Streaming token generation, context meter, history
- **Model Hot-Swap** — Switch models instantly (if llama-swap installed)
- **HTTPS Server** — OpenAI-compatible API with API key authentication
- **KV Cache Advisor** — Intelligent slot/VRAM management for 5-10 concurrent users
- **Expertise-Aware UI** — Dynamic tooltips based on user level

### Technical Foundation
- **Async Runtime** — Built on Tokio for responsive I/O
- **Structured Logging** — Tracing with rotating file appenders
- **Typed Errors** — Thiserror for robust error handling
- **Retry Logic** — Exponential backoff for all network ops
- **Minimal Dependencies** — Single binary, no heavy runtimes

## Prerequisites

Before building llama.cpp, you need **Git**, **CMake**, and a **C/C++ Compiler**:

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install git cmake build-essential pkg-config libssl-dev
```

**Linux (Fedora):**
```bash
sudo dnf install git cmake gcc gcc-c++ make
```

**macOS:**
```bash
xcode-select --install
brew install cmake pkg-config
```

**Windows:**
```powershell
winget install Git.Git Kitware.CMake
# Install Visual Studio Build Tools (C++ workload)
```

### Backend-specific SDKs
- **CUDA (NVIDIA):** [NVIDIA CUDA Toolkit](https://developer.nvidia.com/cuda-downloads)
- **HIP (AMD):** [AMD ROCm](https://rocm.docs.amd.com/)
- **Vulkan:** [Vulkan SDK](https://vulkan.lunarg.com/sdk/home)

## Installation

### One-line install (curl)

```bash
curl -fsSL https://raw.githubusercontent.com/madbeast-16/LocalForge/main/install.sh | bash
```

The install script auto-detects your OS/architecture, downloads the v0.1.5 binary from GitHub Releases, and falls back to building from source if needed.

You can control the installation:
```bash
# Install to a specific directory
INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/madbeast-16/LocalForge/main/install.sh | bash

# Install specific version
VERSION=v0.1.5 curl -fsSL https://raw.githubusercontent.com/madbeast-16/LocalForge/main/install.sh | bash
```

### Download binary manually

Pre-built binaries are available at: https://github.com/madbeast-16/LocalForge/releases/tag/v0.1.5

Supported platforms:
- `x86_64-unknown-linux-gnu` (Linux x86_64)
- `aarch64-unknown-linux-gnu` (Linux ARM64)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-pc-windows-msvc` (Windows)

### Build from Source
```bash
git clone https://github.com/madbeast-16/LocalForge
cd LocalForge
cargo build --release
```

Binary will be at `target/release/localforge` (~3-4 MB).

### Install via Cargo
```bash
cargo install --path .
```

## Usage

### Launch TUI (default)
```bash
localforge
```

The TUI guides you through the complete setup flow.

### Headless Mode
```bash
localforge --headless build --cuda     # Force CUDA build
localforge --headless build --metal    # Force Metal build
```

### Commands

**Detect hardware:**
```bash
localforge detect
```

**Show model recommendations:**
```bash
localforge models          # Hardware-fitted
localforge models --all    # Show all curated
```

**Search HuggingFace:**
```bash
localforge search "llama 3"
```

**Build llama.cpp:**
```bash
localforge build                    # Auto-detect
localforge build --cuda            # Force CUDA
localforge build --vulkan          # Force Vulkan
```

**Configuration:**
```bash
localforge config show
localforge config --init           # Write default config
```

### Config File
Location: `~/.config/localforge/config.toml`

```toml
install_prefix = "~/.local"
models_dir = "~/.local/share/localforge/models"
headless = false
# backend = "Cuda"
# parallel_jobs = 8
```

## Architecture

Clean, layered module structure:

```
src/
├── main.rs          # Entry point, CLI args
├── lib.rs           # Library root
├── app.rs           # App state & routing
├── config/          # Config/state persistence
├── error.rs         # Typed errors + retry
├── logger.rs        # Tracing setup
├── hardware/        # Hardware detection
├── build/           # llama.cpp pipeline
├── models/          # Model discovery & download
├── inference/       # Inference engines
├── server/          # HTTP API server
├── compute/         # KV cache calculator
└── tui/             # Terminal UI
```

### Key Types

**AppState:** Central state with expertise level, persistence
```rust
struct AppState {
    expertise: ExpertiseLevel,      // Beginner/Intermediate/Expert
    setup_complete: bool,
    build_complete: bool,
    downloaded_models: Vec<String>,
    server_running: bool,
}
```

**RetryPolicy:** Exponential backoff with jitter
```rust
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}
```

**InferenceEngine:** Pluggable inference backend
```rust
#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn load_model(&mut self, model: &ModelEntry) -> Result<()>;
    async fn generate(&mut self, session: &ChatSession) -> Result<String>;
}
```

## Development

### Build
```bash
cargo build --release
```

### Run tests
```bash
cargo test
```

### Generate docs
```bash
cargo doc --no-deps --open
```

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux x86_64 | ✅ Full | Primary target |
| Linux ARM64 | ✅ Full | Tested on AWS Graviton |
| macOS Intel | ✅ Full | Tested |
| macOS Apple Silicon | ✅ Full | Native Metal support |
| Windows x86_64 | ✅ Full | Native + WSL |

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run `cargo fmt` and `cargo clippy`
6. Submit a PR

## Goals

**Binary Size:** < 12 MB (release, stripped)
**Idle RAM:** < 30 MB
**Zero Dependencies:** Single static binary

## License

MIT

## Acknowledgments

Built with:
- [ratatui](https://github.com/ratatui-org/ratatui) for TUI
- [tokio](https://tokio.rs) for async runtime
- [tracing](https://github.com/tokio-rs/tracing) for logging
- [axum](https://github.com/tokio-rs/axum) for HTTP server
- [llama.cpp](https://github.com/ggerganov/llama.cpp) for inference
