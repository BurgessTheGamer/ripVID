use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize the logging system with both file and console output
pub fn init_logging(app_data_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Create logs directory
    let logs_dir = app_data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)?;

    // Set up file appender with daily rotation
    let file_appender = RollingFileAppender::new(Rotation::DAILY, logs_dir, "ripvid.log");

    // Determine if we're in debug mode
    let is_debug = cfg!(debug_assertions);

    // Create environment filter
    // In debug mode: show debug and above
    // In release mode: show info and above
    let env_filter = if is_debug {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    // Set up console layer (only in debug mode for better performance in production)
    let console_layer = if is_debug {
        Some(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_line_number(true)
                .with_ansi(true)
                .pretty(),
        )
    } else {
        None
    };

    // Set up file layer (always active)
    let file_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_ansi(false) // No ANSI codes in file logs
        .json() // Use JSON format for easier parsing
        .with_writer(file_appender);

    // Build and initialize the subscriber
    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer);

    if let Some(console) = console_layer {
        subscriber.with(console).init();
    } else {
        subscriber.init();
    }

    if is_debug {
        tracing::info!("Logging initialized in DEBUG mode (console + file)");
    } else {
        tracing::info!("Logging initialized in RELEASE mode (file only)");
    }

    Ok(())
}
