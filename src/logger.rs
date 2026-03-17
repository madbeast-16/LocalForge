use std::path::PathBuf;
use tracing::subscriber::DefaultGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn setup(log_dir: Option<PathBuf>) -> DefaultGuard {
    let filter = EnvFilter::from_default_env()
        .add_directive("localforge=debug".parse().unwrap())
        .add_directive("tokio=info".parse().unwrap())
        .add_directive("tower=warn".parse().unwrap())
        .add_directive("reqwest=warn".parse().unwrap());

    let stdout_layer = fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_ansi(true);

    match log_dir {
        Some(log_dir) => {
            let file_appender = tracing_appender::rolling::daily(log_dir, "localforge.log");
            let file_layer = fmt::layer()
                .with_ansi(false)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_writer(file_appender);

            tracing_subscriber::Registry::default()
                .with(filter)
                .with(stdout_layer)
                .with(file_layer)
                .init();

            tracing::info!("LocalForge logging initialized (file + stdout)");

            tracing::dispatcher::set_default(&tracing::Dispatch::default())
        }
        None => {
            tracing_subscriber::Registry::default()
                .with(filter)
                .with(stdout_layer)
                .init();

            tracing::info!("LocalForge logging initialized (stdout only)");

            tracing::dispatcher::set_default(&tracing::Dispatch::default())
        }
    }
}

pub fn get_log_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("localforge")
        .join("logs")
}

pub fn log_dir_with_env() -> String {
    std::env::var("LOCALFORGE_LOG_DIR")
        .unwrap_or_else(|_| get_log_dir().to_string_lossy().into_owned())
}

pub fn init() -> DefaultGuard {
    let log_dir = get_log_dir();
    if let Ok(_) = std::fs::create_dir_all(&log_dir) {
        setup(Some(log_dir))
    } else {
        setup(None)
    }
}
