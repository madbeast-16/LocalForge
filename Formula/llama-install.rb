class LlamaInstall < Formula
  desc "Zero-overhead TUI tool to install and configure llama.cpp with optimal hardware-aware settings"
  homepage "https://github.com/AlexsJones/llama-install"
  license "MIT"
  version "0.2.0"

  on_macos do
    on_arm do
      url "https://github.com/AlexsJones/llama-install/releases/download/v#{version}/llama-install-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64_MACOS"
    end
    on_intel do
      url "https://github.com/AlexsJones/llama-install/releases/download/v#{version}/llama-install-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64_MACOS"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/AlexsJones/llama-install/releases/download/v#{version}/llama-install-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64_LINUX"
    end
    on_intel do
      url "https://github.com/AlexsJones/llama-install/releases/download/v#{version}/llama-install-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64_LINUX"
    end
  end

  # Build from source if no pre-built binary is available
  depends_on "rust" => :build
  depends_on "cmake"
  depends_on "git"

  def install
    # If a pre-built binary was downloaded, just copy it
    if File.exist?("llama-install")
      bin.install "llama-install"
    else
      # Build from source
      system "cargo", "install", *std_cargo_args
    end
  end

  test do
    assert_match "llama-install", shell_output("#{bin}/llama-install --version")
  end
end
