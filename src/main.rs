#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use tracing::info;

pub mod config;
pub mod platform;
pub mod bots;
pub mod shared;
pub mod types;
mod cli;

use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging()?;
    
    info!("🚀 Starting SniperForge Multi-Bot Platform v0.1.0");
    
    // Always use the modern CLI system
    cli::run_cli().await
}

fn init_logging() -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    let file_appender = tracing_appender::rolling::daily("logs", "sniperforge.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sniperforge=info,warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
        )
        .init();
    
    // Keep the guard alive for the duration of the program
    std::mem::forget(_guard);
      Ok(())
}
