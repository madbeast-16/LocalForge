class Localforge < Formula
  desc "Local-first TUI tool for a fully working local LLM stack"
  homepage "https://github.com/madbeast-16/LocalForge"
  license "MIT"
  version "0.1.5"

  on_macos do
    on_arm do
      url "https://github.com/madbeast-16/LocalForge/releases/download/v#{version}/localforge-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64_MACOS"
    end
    on_intel do
      url "https://github.com/madbeast-16/LocalForge/releases/download/v#{version}/localforge-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64_MACOS"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/madbeast-16/LocalForge/releases/download/v#{version}/localforge-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64_LINUX"
    end
    on_intel do
      url "https://github.com/madbeast-16/LocalForge/releases/download/v#{version}/localforge-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64_LINUX"
    end
  end

  depends_on "rust" => :build
  depends_on "cmake"
  depends_on "git"

  def install
    if File.exist?("localforge")
      bin.install "localforge"
    else
      system "cargo", "install", *std_cargo_args
    end
  end

  test do
    assert_match "localforge", shell_output("#{bin}/localforge --version")
  end
end
