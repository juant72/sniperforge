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
pub mod strategies;
pub mod analysis;
mod cli_phase6a;

use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("🎯 SniperForge Phase 6A: Advanced Trading Strategies");
    info!("🚀 Starting CLI interface...");

    // Run Phase 6A CLI
    cli_phase6a::run_cli().await?;

    Ok(())
}
