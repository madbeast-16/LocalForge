# llama-install

Zero-overhead TUI tool to install and configure llama.cpp with optimal hardware-aware settings.

**3.3 MB static binary. No runtime bloat. Fast cold start.**

## Features

- **Hardware Detection** — Auto-detect CPU, GPU (NVIDIA/AMD/Intel/Apple Silicon), VRAM, CUDA, ROCm
- **Interactive TUI** — Modern terminal interface with vim keybindings, built on ratatui
- **Model Recommendations** — Curated GGUF model registry matched to your hardware budget
- **HuggingFace Search** — Search and browse models directly from the TUI
- **Headless CLI** — Full scripting support with `--headless` flag
- **Config File** — Persistent `config.toml` at `~/.config/llama-install/`
- **Pluggable Backends** — CUDA, HIP (ROCm), Metal, Vulkan, OpenBLAS, CPU-only

## Installation

### One-line install (curl)

```bash
curl -fsSL https://raw.githubusercontent.com/madbeast-16/llama-install/llama-install-v1/install.sh | bash
```

The script auto-detects your OS/architecture, downloads the correct binary, and falls back to building from source if no pre-built binary is available.

You can control the install location and version:

```bash
INSTALL_DIR=/usr/local/bin VERSION=v0.2.0 curl -fsSL https://raw.githubusercontent.com/madbeast-16/llama-install/llama-install-v1/install.sh | bash
```

### Cargo (from crates.io)

```bash
cargo install llama-install
```

### Cargo (from source)

```bash
git clone https://github.com/madbeast-16/llama-install.git
cd llama-install
cargo install --path .
```

### Homebrew (macOS / Linux)

```bash
brew tap madbeast-16/llama-install https://github.com/madbeast-16/llama-install
brew install llama-install
```

### Manual download

Download pre-built binaries from the [GitHub Releases](https://github.com/madbeast-16/llama-install/releases) page.

Available platforms:
- `x86_64-unknown-linux-gnu` (Linux x86_64)
- `aarch64-unknown-linux-gnu` (Linux ARM64)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-pc-windows-msvc` (Windows)

## Usage

### Launch TUI (default)

```bash
llama-install
```

The TUI guides you through:
1. Hardware detection
2. Backend selection (with auto-recommendation)
3. Model browsing and selection
4. Building llama.cpp
5. Summary with quick-start commands

### Headless / scripting mode

```bash
llama-install --headless
llama-install --headless build --cuda
```

### Detect hardware

```bash
llama-install detect
```

```
  CPU:   Intel(R) Core(TM) Ultra 9 285H (16 cores)
  GPU:   NVIDIA GeForce RTX 5070 Ti Laptop GPU (NVIDIA)
  VRAM:  11.9 GB
  RAM:   31.1 GB (28.6 GB available)
  OS:    Linux (WSL)
  CUDA:  12.0
```

### Build llama.cpp

```bash
llama-install build                # auto-detect best backend
llama-install build --cuda         # force CUDA
llama-install build --metal        # force Metal (macOS)
llama-install build --vulkan       # force Vulkan
llama-install build --cpu-only     # CPU only
```

### Model recommendations

```bash
llama-install models               # hardware-fitted recommendations
llama-install models --all         # show all curated models
```

### Search HuggingFace

```bash
llama-install search "codellama gguf"
```

### Configuration

```bash
llama-install config               # show current config
llama-install config --init        # write default config.toml
```

Config file location: `~/.config/llama-install/config.toml`

```toml
install_prefix = "~/.local"
models_dir = "~/.local/share/llama-install/models"
headless = false
# backend = "Cuda"        # optional: lock to a specific backend
# parallel_jobs = 8       # optional: override build parallelism
```

## Options

```
  -v, --verbose...       Verbosity level (-v, -vv, -vvv)
  -p, --prefix <PREFIX>  Installation prefix [default: ~/.local]
      --headless         Run in headless mode (no TUI, for scripting)
  -h, --help             Print help
  -V, --version          Print version
```

## TUI Keybindings

| Key | Action |
|-----|--------|
| `Enter` | Select / continue |
| `Esc` / `Backspace` | Go back |
| `j` / `k` or `Up` / `Down` | Navigate |
| `/` | Search HuggingFace (in model view) |
| `s` | Skip model selection |
| `q` / `Ctrl+C` | Quit |

## Supported Backends

| Backend | GPU | Flags |
|---------|-----|-------|
| CUDA | NVIDIA (CC >= 6.0) | `-DGGML_CUDA=ON -DGGML_CUDA_F16=ON` |
| HIP | AMD (ROCm) | `-DGGML_HIPBLAS=ON` |
| Metal | Apple Silicon | `-DGGML_METAL=ON -DGGML_METAL_EMBED_LIBRARY=ON` |
| Vulkan | Intel / AMD / NVIDIA | `-DGGML_VULKAN=ON` |
| OpenBLAS | CPU | `-DGGML_BLAS=ON -DGGML_BLAS_VENDOR=OpenBLAS` |
| CPU Only | CPU | `-DGGML_NATIVE=ON` |

## Architecture

```
src/
├── main.rs              # Thin CLI shell (clap)
├── lib.rs               # Library crate root
├── app.rs               # TUI state machine (6 screens)
├── build/               # Build pipeline: clone -> cmake -> compile -> verify
├── cli/                 # Headless CLI mode
├── config/              # AppConfig, Backend types, CMake flag generation
├── hardware/            # CPU, GPU, memory, CUDA, ROCm detection
├── models/              # Curated registry, HuggingFace API, download manager
└── tui/                 # Terminal framework, theme, 6 view modules
```

The binary is a thin shell over a fully reusable library crate:

```rust
use llama_install::{AppConfig, HardwareInfo, ModelEntry, BackendName};
```

## License

MIT
# llama-install
