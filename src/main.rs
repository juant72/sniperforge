#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use tracing::info;
use dotenv::dotenv;

pub mod config;
pub mod platform;
pub mod bots;
pub mod shared;
pub mod types;
mod cli;
mod jupiter_speed_test;
mod cache_safety_test;

use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Initialize rustls crypto provider first
    init_crypto_provider();
    
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

fn init_crypto_provider() {
    // Initialize rustls default crypto provider to fix:
    // "no process-level CryptoProvider available"
    
    eprintln!("🔐 Setting up crypto provider for TLS connections...");
    
    // For rustls 0.23+, we need to explicitly install a crypto provider
    // This MUST be done once at program startup before any TLS operations
    
    // Try to install the ring crypto provider
    let result = rustls::crypto::ring::default_provider().install_default();
    
    match result {
        Ok(()) => {
            eprintln!("✅ Ring crypto provider installed successfully");
        }
        Err(_) => {
            // Provider was already installed, which is fine
            eprintln!("ℹ️  Crypto provider was already installed");
        }
    }
    
    eprintln!("✅ Crypto setup completed");
}
