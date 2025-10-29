use std::env;
use tracing_appender::{
    non_blocking,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_logging() -> non_blocking::WorkerGuard {
    let env_filter = env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

    let console_layer = fmt::layer()
        .with_target(false)
        .with_file(true)
        .with_level(true);

    let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "gitpulse.log");
    let (non_blocking_writer, guard) = non_blocking(file_appender);
    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking_writer)
        .with_target(false)
        .with_level(true);

    tracing_subscriber::registry()
        .with(EnvFilter::new(env_filter))
        .with(console_layer)
        .with(file_layer)
        .init();

    guard
}
