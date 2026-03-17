#!/bin/bash
# =============================================================================
# llama-install — one-line installer
#
# Usage:
# curl -fsSL https://raw.githubusercontent.com/madbeast-16/LocalForge/main/install.sh | bash
#
# Options (via env vars):
#   INSTALL_DIR   — where to place the binary  (default: ~/.local/bin)
#   VERSION       — specific release tag        (default: latest)
# =============================================================================

set -euo pipefail

REPO_OWNER="madbeast-16"
REPO_NAME="LocalForge"
BINARY_NAME="localforge"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

info()  { printf "\033[1;34m==>\033[0m %s\n" "$*"; }
ok()    { printf "\033[1;32m ✓\033[0m  %s\n" "$*"; }
warn()  { printf "\033[1;33m !\033[0m  %s\n" "$*"; }
err()   { printf "\033[1;31m ✗\033[0m  %s\n" "$*" >&2; exit 1; }

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        err "Required command not found: $1"
    fi
}

# ---------------------------------------------------------------------------
# Detect platform
# ---------------------------------------------------------------------------

detect_platform() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)
            case "$arch" in
                x86_64)         PLATFORM="x86_64-unknown-linux-gnu"   ;;
                aarch64|arm64)  PLATFORM="aarch64-unknown-linux-gnu"  ;;
                *)              err "Unsupported Linux architecture: $arch" ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                x86_64)         PLATFORM="x86_64-apple-darwin"        ;;
                aarch64|arm64)  PLATFORM="aarch64-apple-darwin"       ;;
                *)              err "Unsupported macOS architecture: $arch" ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            PLATFORM="x86_64-pc-windows-msvc"
            BINARY_NAME="llama-install.exe"
            ;;
        *)
            err "Unsupported OS: $os"
            ;;
    esac

    info "Detected platform: $PLATFORM"
}

# ---------------------------------------------------------------------------
# Resolve version
# ---------------------------------------------------------------------------

resolve_version() {
    if [ "$VERSION" = "latest" ]; then
        info "Fetching latest release version..."
        VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest" \
            | grep '"tag_name"' \
            | head -1 \
            | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')

        if [ -z "$VERSION" ]; then
            err "Could not determine latest release. Set VERSION=vX.Y.Z manually."
        fi
    fi

    info "Version: $VERSION"
}

# ---------------------------------------------------------------------------
# Download & install
# ---------------------------------------------------------------------------

download_and_install() {
    local url asset tmpdir

    asset="${BINARY_NAME}-${PLATFORM}.tar.gz"
    url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${VERSION}/${asset}"

    info "Downloading ${asset}..."

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    if command -v curl > /dev/null 2>&1; then
        curl -fSL --progress-bar "$url" -o "${tmpdir}/${asset}" || try_raw_binary "$tmpdir"
    elif command -v wget > /dev/null 2>&1; then
        wget -q --show-progress "$url" -O "${tmpdir}/${asset}" || try_raw_binary "$tmpdir"
    else
        err "Neither curl nor wget found. Install one of them first."
    fi

    info "Extracting..."
    if [ -f "${tmpdir}/${asset}" ]; then
        tar -xzf "${tmpdir}/${asset}" -C "$tmpdir" 2>/dev/null || {
            # If tar fails, the download might be a raw binary
            mv "${tmpdir}/${asset}" "${tmpdir}/${BINARY_NAME}"
        }
    fi

    # Find the binary in extracted files
    local binary_path=""
    if [ -f "${tmpdir}/${BINARY_NAME}" ]; then
        binary_path="${tmpdir}/${BINARY_NAME}"
    elif [ -f "${tmpdir}/target/release/${BINARY_NAME}" ]; then
        binary_path="${tmpdir}/target/release/${BINARY_NAME}"
    fi

    if [ -z "$binary_path" ]; then
        # Try downloading raw binary (no tarball)
        try_raw_binary "$tmpdir"
        binary_path="${tmpdir}/${BINARY_NAME}"
    fi

    if [ ! -f "$binary_path" ]; then
        err "Binary not found after extraction. The release may not have pre-built binaries yet."
    fi

    mkdir -p "$INSTALL_DIR"
    mv "$binary_path" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    ok "Installed to ${INSTALL_DIR}/${BINARY_NAME}"
}

try_raw_binary() {
    local tmpdir="$1"
    local raw_url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${VERSION}/${BINARY_NAME}-${PLATFORM}"

    info "Trying raw binary download..."
    if command -v curl > /dev/null 2>&1; then
        curl -fSL --progress-bar "$raw_url" -o "${tmpdir}/${BINARY_NAME}" 2>/dev/null || true
    elif command -v wget > /dev/null 2>&1; then
        wget -q "$raw_url" -O "${tmpdir}/${BINARY_NAME}" 2>/dev/null || true
    fi
}

# ---------------------------------------------------------------------------
# Verify PATH
# ---------------------------------------------------------------------------

ensure_path() {
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*)
            ;;
        *)
            warn "${INSTALL_DIR} is not in your PATH"
            echo ""
            echo "  Add it to your shell config:"
            echo ""

            if [ -n "${BASH_VERSION:-}" ] || [ -f "$HOME/.bashrc" ]; then
                echo "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc"
                echo "    source ~/.bashrc"
            fi
            if [ -n "${ZSH_VERSION:-}" ] || [ -f "$HOME/.zshrc" ]; then
                echo "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc"
                echo "    source ~/.zshrc"
            fi
            if [ -f "$HOME/.config/fish/config.fish" ]; then
                echo "    fish_add_path ${INSTALL_DIR}"
            fi
            echo ""
            ;;
    esac
}

# ---------------------------------------------------------------------------
# Fallback: build from source
# ---------------------------------------------------------------------------

build_from_source() {
    info "Pre-built binary not available. Building from source..."

    need_cmd git
    need_cmd cargo

    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    info "Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO_OWNER}/${REPO_NAME}.git" "$tmpdir/src"

    info "Building release binary (this may take a minute)..."
    cargo build --release --manifest-path "${tmpdir}/src/Cargo.toml"

    mkdir -p "$INSTALL_DIR"
    cp "${tmpdir}/src/target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    ok "Built and installed to ${INSTALL_DIR}/${BINARY_NAME}"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

main() {
    echo ""
    echo "  ┌──────────────────────────────────────┐"
    echo "  │       llama-install  installer        │"
    echo "  └──────────────────────────────────────┘"
    echo ""

    detect_platform
    resolve_version
    download_and_install || build_from_source
    ensure_path

    echo ""
    ok "Installation complete!"
    echo ""
    echo "  Run:  llama-install          # launch TUI"
    echo "        llama-install detect   # show hardware"
    echo "        llama-install --help   # all commands"
    echo ""
}

main "$@"
