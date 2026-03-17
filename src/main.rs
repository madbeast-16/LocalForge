use anyhow::Result;
use clap::{Parser, Subcommand};

use localforge::app;
use localforge::cli;
use localforge::config;

#[derive(Parser)]
#[command(name = "localforge")]
#[command(version = "0.1.0")]
#[command(about = "Local-first TUI tool for a fully working local LLM stack")]
#[command(author, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Installation prefix path
    #[arg(short, long, global = true, default_value = "~/.local")]
    prefix: String,

    /// Run in headless mode (no TUI, for scripting)
    #[arg(long, global = true)]
    headless: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch interactive TUI wizard (default)
    Tui,

    /// Detect and display hardware capabilities
    Detect,

    /// Build llama.cpp with optimal settings
    Build {
        /// Force CPU-only build
        #[arg(long)]
        cpu_only: bool,
        /// Force CUDA backend
        #[arg(long)]
        cuda: bool,
        /// Force Metal backend (macOS)
        #[arg(long)]
        metal: bool,
        /// Force Vulkan backend
        #[arg(long)]
        vulkan: bool,
    },

    /// Show model recommendations for your hardware
    Models {
        /// Show all curated models, not just hardware-fitted
        #[arg(short, long)]
        all: bool,
    },

    /// Search HuggingFace for GGUF models
    Search {
        /// Search query
        query: String,
    },

    /// Show or generate config file
    Config {
        /// Write default config to ~/.config/localforge/config.toml
        #[arg(long)]
        init: bool,
    },
}

fn main() -> Result<()> {
    // Initialize tracing
    let _guard = localforge::logger::init();

    let cli_args = Cli::parse();

    // Load config from disk, then override with CLI args
    let mut cfg = config::AppConfig::load();
    if cli_args.prefix != "~/.local" {
        cfg.install_prefix = cli_args.prefix.clone();
    }
    if cli_args.headless {
        cfg.headless = true;
    }

    let prefix = cfg.resolved_prefix();
    if prefix.is_empty() {
        anyhow::bail!("Prefix path cannot be empty");
    }

    match cli_args.command {
        // No subcommand: launch TUI by default (unless --headless)
        None | Some(Commands::Tui) => {
            if cfg.headless {
                cli::run_headless(&cfg, false, false, false, false)?;
            } else {
                let mut application = app::App::new(cfg);
                application.run()?;
            }
        }

        Some(Commands::Detect) => {
            cli::run_detect()?;
        }

        Some(Commands::Build {
            cpu_only,
            cuda,
            metal,
            vulkan,
        }) => {
            cli::run_headless(&cfg, cpu_only, cuda, metal, vulkan)?;
        }

        Some(Commands::Models { all }) => {
            cli::run_models(all)?;
        }

        Some(Commands::Search { query }) => {
            cli::run_search(&query)?;
        }

        Some(Commands::Config { init }) => {
            if init {
                let path = config::AppConfig::default_path();
                cfg.save()?;
                println!("Config written to {}", path.display());
            } else {
                let content = toml::to_string_pretty(&cfg)?;
                println!("{}", content);
            }
        }
    }

    Ok(())
}
