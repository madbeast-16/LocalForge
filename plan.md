You are an expert Rust architect and TUI tool developer. You are helping me design and implement a cross-platform, zero-overhead Rust TUI application called **LocalForge**.

## High-level goal

Design and implement a local-first TUI tool that takes a technically-equipped user from **zero to a fully working local LLM stack**, with:

- Hardware detection and optimal backend selection
- Automated llama.cpp build (with tuned CMake flags)
- Optional llama-swap installation and integration
- Curated + searchable GGUF model selection and download
- Chat interface for local inference
- HTTPS server exposure with API keys for multi-user / agent access

The code should be production-grade, minimal in dependencies, and suitable for an open-source project.

---

## Platforms and constraints

- Platforms: **Windows (native + WSL), macOS, Linux** — all first-class
- No heavy runtimes (no Python, Node, Electron, Java). **Single Rust binary**.
- Minimize memory and disk footprint:
  - Binary size target: `< 12 MB` (stripped, release build)
  - Idle RAM usage target: `< 30 MB`
- Use **Rust async** where appropriate (Tokio)
- Use a **TUI stack** similar to `llmfit`:
  - `ratatui` for rendering
  - `crossterm` for terminal I/O
- Use **structured logging** and robust error handling:
  - `thiserror` for typed errors
  - `tracing` + file appender for logs
  - Retry logic for all external operations (builds, downloads, API calls)

---

## Core UX flow

The app is logically divided into two screens:

1. **Setup Flow (wizard-like):**
   - Screen 0: Welcome + expertise selector
   - Tab 1: Hardware detection & backend selection
   - Tab 2: llama.cpp build
   - Tab 3: Optional llama-swap install
   - Tab 4: Model discovery & download (curated + HuggingFace search)
   - Final confirmation: summary + “Proceed” (then transition to workspace)

2. **Main Workspace:**
   - Chat tab: local inference with selected model(s), optional swap
   - Settings tab: server exposure (HTTPS + API keys + concurrency), general config

All steps should be **resumable**: partial progress is persisted to a config/state file and restored on restart.

---

## Expertise-aware UX

On the **Welcome screen**, ask the user for their expertise level:

- `beginner`
- `intermediate`
- `expert`

Store this in config and let it influence:

- How much explanation is shown in tooltips and modals
- Whether advanced flags (CMake, llama-server flags) are visible or hidden
- How aggressive the “recommended default” behavior is

Design this as a first-class concern in the architecture (e.g. `ExpertiseLevel` enum in `AppState`).

---

## Architecture and modules

Propose a clean, layered module structure like this (you can refine):

- `src/main.rs` — entry, CLI, initializes TUI and runtime
- `src/app.rs` — `AppState`, router, high-level event loop
- `src/config.rs` — read/write `config.toml` and `state.toml`
- `src/logger.rs` — `tracing` setup, rotating log files
- `src/error.rs` — shared error types + retry helper

**Hardware detection**
- `src/hardware/`
  - Detect CPU, RAM, GPU and environment (native vs WSL)
  - Determine available backends: CUDA, Metal, ROCm, Vulkan, SYCL, CPU
  - Expose a `HardwareProfile` and `BackendProfile` with ranking

**Build subsystem**
- `src/build/`
  - llama.cpp pipeline:
    - Check deps (cmake, compiler, git, make/ninja)
    - Clone repository
    - Configure CMake with tuned flags per backend
    - Build and verify binaries
  - llama-swap pipeline (optional)
  - Async process runner with:
    - Retry policy (max attempts, backoff)
    - Full stdout/stderr capture to log files

**Model discovery & fit**
- `src/models/`
  - `curated.rs`: built-in curated GGUF model list (embedded JSON or TOML)
  - `hf_search.rs`: HuggingFace model search client (GGUF-filtered)
  - `fit.rs`: model “fit” scoring, inspired by `llmfit`:
    - Use VRAM, model size, quantization, and context window
    - Classify fit as: Perfect / Good / Marginal / Too Tight
  - `download.rs`: robust downloader:
    - Chunked downloads with resume support
    - Retries with exponential backoff
    - Per-file progress reporting
  - `quant.rs`: recommend quantization per hardware (Q4_K_M, Q5, Q8, etc.)

**Inference**
- `src/inference/`
  - `InferenceEngine` trait with implementations:
    - `LlamaCppEngine`: wraps `llama-cli` / `llama-server`
    - `LlamaSwapEngine`: wraps llama-swap for hot-swapping
  - `session.rs`: chat session state and streaming integration

**Concurrency & KV cache**
- `src/compute/`
  - `kv_cache.rs`: calculate KV cache per slot based on:
    - Layers, heads, head dim, context length, dtype
  - `slot_advisor.rs`:
    - Given VRAM, model size, context, KV dtype, and target users,
      recommend:
      - `--parallel` (slots)
      - `--ctx-size`
      - `--cache-type-k/v`
      - Whether `--cont-batching` should be on
    - Ensure VRAM usage stays within safe thresholds (≈ 80–85% target)
  - Integrate with server settings so user can configure concurrency
    for 5–10 concurrent “users” (human + AI agents)

**Server exposure**
- `src/server/`
  - Lightweight HTTP server (suggest `axum`)
  - Optional HTTPS via self-signed cert (`rustls` + `rcgen`)
  - API-key protected:
    - API keys stored in OS keychain (`keyring` crate) not in plaintext
  - Acts as a proxy to llama.cpp:
    - Expose OpenAI-compatible `/v1/chat/completions`
    - Inject flags produced by `slot_advisor.rs`
  - Optional rate limiting per API key (protect against noisy agents)

**UI**
- `src/ui/`
  - `theme.rs`: TUI themes (e.g., Nord, Gruvbox, Dracula)
  - Shared components:
    - Tables (sortable, scrollable)
    - Progress bars (build & download steps)
    - Log panel (tail latest log lines)
    - Modal dialogs (confirmations, error details)
    - Tooltip system that adapts to expertise level
  - Screens:
    - `welcome.rs`: banner, description, expertise selector
    - `hardware.rs`: show hardware + backend options, VRAM overrides
    - `build.rs`: llama.cpp build progress + retry + log view
    - `llama_swap.rs`: explanation, optional install, status
    - `models.rs`: curated list, HF search, multi-select, download manager
    - `chat.rs`: chat UI with streaming tokens, model selection, context meter
    - `settings.rs`: server config, concurrency, TLS, API keys, theme, paths
  - `events.rs`: keyboard event routing, focus management

Please design the data structures, module boundaries, and key function signatures for this architecture.

---

## Setup flow details

### Screen 0 — Welcome + expertise

- Show:
  - Project name and short description
  - Detected platform (OS + backend summary)
- Ask user to choose expertise:
  - Beginner: show more explanations, fewer knobs
  - Intermediate: show recommendations, allow overrides
  - Expert: show all flags and internals; minimal hand-holding
- Persist this choice in config.

### Tab 1 — Hardware & backend

- Detect:
  - CPU, number of cores, architecture
  - Total RAM
  - GPU(s): vendor, VRAM, driver/runtime (CUDA, Metal, ROCm, etc.)
  - WSL environment (if applicable)
- Offer backends ranked by expected performance:
  - CUDA, Metal, ROCm, Vulkan, SYCL, CPU
- Show recommended backend with short rationale.
- Allow VRAM manual override for unreliable detection.
- Persist chosen backend and overrides.

### Tab 2 — llama.cpp build

- Given backend choice, generate CMake flags:
  - Examples:
    - CUDA: `-DGGML_CUDA=ON -DCMAKE_CUDA_ARCHITECTURES=auto`
    - Metal: `-DGGML_METAL=ON`
    - ROCm: `-DGGML_HIPBLAS=ON -DAMDGPU_TARGETS=auto`
    - CPU (x86): `-DGGML_AVX2=ON -DGGML_FMA=ON` etc.
- Show them in the UI:
  - Beginner/Intermediate: read-only, with short explanation
  - Expert: editable text field
- Build process:
  - Multi-step progress: deps -> clone -> configure -> build -> verify
  - On failure:
    - Show high-level error
    - Provide path to full build log
    - Offer “Retry” with backoff and max attempts
    - Optional “Open log” view for last N lines
- Persist installation path and build metadata.

### Tab 3 — llama-swap (optional)

- Explain what llama-swap is and why a user might want it:
  - Hot-swapping models without restarting server
  - Important if using multiple models or agents
- Allow:
  - Install
  - Skip for now
- Track installation status; expose swap-only features later in chat if installed.

### Tab 4 — Models (curated + HF search + downloads)

- **Curated Models view:**
  - Use an embedded list of popular GGUF models:
    - Llama 3.x family, Qwen3, Mistral, etc.
  - Columns: name, params, gguf size, quant, context, fit score, estimated tok/s
  - Fit score based on hardware and VRAM (inspired by `llmfit`):
    - Perfect / Good / Marginal / Too Tight
- **Search view:**
  - Search HuggingFace GGUF models by text query.
  - Merge search results with local fit scoring.
- **Download manager:**
  - Support multi-select model downloads.
  - Show recommended quantization based on hardware:
    - Example: recommend Q4_K_M for 16GB VRAM as “best trade-off”
  - Allow user to override quant.
  - Download behavior:
    - Chunked downloads with resume
    - Retries with exponential backoff
    - Progress bars, ETA, speed
  - Track downloaded models in config (path, size, checksum).

### Final step in setup

- Show summary:
  - Backend selected
  - llama.cpp status
  - llama-swap status
  - Downloaded models and where they’re stored
- Confirm:
  - “All good, proceed to workspace”
- Transition to main workspace (Chat + Settings).

---

## Main Workspace

### Chat tab

- Choose active model from downloaded list.
- If llama-swap is installed:
  - Allow switching models without restarting server (hot swap).
- Chat UI:
  - Scrollback history
  - Streaming token rendering
  - Input buffer at bottom
  - System prompt editor (per model, persisted)
  - Context usage meter (tokens used vs context limit)

### Settings tab (with concurrency/KV awareness)

- General:
  - Model storage directory
  - Default quantization preference
  - Theme selector
  - Log level, “open logs” / “clear logs”
- Server exposure:
  - Configure:
    - Host (127.0.0.1, 0.0.0.0, etc.)
    - Port
    - TLS on/off (self-signed via `rcgen` + `rustls`)
    - API key (stored via `keyring`, not in config file)
  - Concurrency & KV cache for **home-lab with 5–10 users including AI agents**:
    - Input:
      - Expected concurrent users (slider or numeric input)
      - Expected context per session (e.g. 8K, 16K, 32K)
      - Toggle “includes AI agents” (more aggressive assumptions)
      - KV cache dtype (q8_0 recommended, f16, etc.)
    - Show:
      - Model size in VRAM
      - Calculated KV cache usage for given slots and context
      - Total VRAM usage and safety margin
      - Recommended `llama-server` flags (`--parallel`, `--ctx-size`, `--cache-type-*`, `--cont-batching`, `--flash-attn` if supported)
    - Let users copy the generated command.
  - Optional rate limiting per API key.

---

## Error handling, retries, and logging

Explicitly design:

- A `RetryPolicy` struct and `with_retry` helper for:
  - Network calls (HF API, downloads)
  - Git, CMake, build commands
- Typed error enums for:
  - Hardware detection
  - Build failures
  - Download issues
  - Inference startup
  - Config read/write
- Logging:
  - Centralized `tracing` subscriber
  - Log file locations:
    - Main app log: `~/.local/share/localforge/logs/localforge.log`
    - Build logs: `.../build_<timestamp>.log`
    - Downloads: `.../download_<model>_<timestamp>.log`
  - Integrate a basic log viewer into the UI for recent lines.

---

## What I want from you

1. Refine and finalize the **module layout and key data structures** for this architecture.
2. Propose **concrete Rust type definitions** (structs/enums/traits) for:
   - `AppState`
   - `HardwareProfile`, `BackendProfile`
   - `BuildManager` and build steps
   - `ModelRecord`, `FitLevel`, `Quantization`
   - `InferenceEngine` trait
   - `ServerConfig` and concurrency/KV configuration
3. Outline the **initial project scaffold**:
   - `cargo new`
   - Dependencies in `Cargo.toml`
   - File structure and stub modules
4. Provide **incremental implementation steps** suitable for an OpenCode “build mode” workflow:
   - Step-by-step commits/milestones (e.g., “implement hardware detection module skeleton”, “wire up basic TUI with welcome + expertise selection”, etc.)

Keep the output focused, structured, and code-oriented so I can paste it into OpenCode and iterate in “build” mode.