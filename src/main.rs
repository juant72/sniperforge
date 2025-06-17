use anyhow::Result;
use clap::{Arg, Command};
use colored::*;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn, error};

pub mod config;
pub mod platform;
pub mod bots;
pub mod shared;
pub mod types;
mod cli;

use config::Config;
use platform::SniperForgePlatform;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging()?;
    
    info!("🚀 Starting SniperForge Multi-Bot Platform v0.1.0");
    
    // Parse command line arguments
    let matches = Command::new("SniperForge")
        .version("0.1.0")
        .about("Multi-Bot Trading Platform for Solana")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .default_value("config/platform.toml")
        )
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_name("MODE")
                .help("Operating mode: development, staging, production")
                .default_value("development")
        )
        .arg(
            Arg::new("bot")
                .short('b')
                .long("bot")
                .value_name("BOT_TYPE")
                .help("Run specific bot type: lp-sniper, arbitrage, mev, copy-trading")
                .action(clap::ArgAction::Append)
        )
        .get_matches();
    
    // Load configuration
    let config_path = matches.get_one::<String>("config").unwrap();
    let config = Config::load(config_path)?;
    
    let mode = matches.get_one::<String>("mode").unwrap();
    info!("🔧 Running in {} mode", mode.bright_yellow());
    
    // Display startup banner
    display_banner();
    
    // Initialize the platform
    let platform = Arc::new(SniperForgePlatform::new(config).await?);
      // Handle specific bot selection
    if let Some(bot_types) = matches.get_many::<String>("bot") {
        let bot_types_vec: Vec<String> = bot_types.cloned().collect();
        info!("🤖 Starting specific bots: {:?}", bot_types_vec);
        platform.start_specific_bots(bot_types_vec).await?;
    } else {
        info!("🏢 Starting full platform with all enabled bots");
        platform.start_platform().await?;
    }
    
    // Setup graceful shutdown
    setup_shutdown_handler(platform.clone()).await?;
    
    // Main event loop
    info!("✅ SniperForge platform is running. Press Ctrl+C to stop.");
    platform.run().await?;
    
    Ok(())
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

fn display_banner() {
    println!("{}", "
███████╗███╗   ██╗██╗██████╗ ███████╗██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
██╔════╝████╗  ██║██║██╔══██╗██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
███████╗██╔██╗ ██║██║██████╔╝█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗█████╗  
╚════██║██║╚██╗██║██║██╔═══╝ ██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝  
███████║██║ ╚████║██║██║     ███████╗██║  ██║██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
╚══════╝╚═╝  ╚═══╝╚═╝╚═╝     ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
".bright_cyan());
    
    println!("{}", "🤖 Multi-Bot Trading Platform for Solana".bright_green().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    println!();
}

async fn setup_shutdown_handler(platform: Arc<SniperForgePlatform>) -> Result<()> {
    let platform_clone = platform.clone();
    
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                warn!("🛑 Shutdown signal received, stopping platform gracefully...");
                if let Err(e) = platform_clone.shutdown().await {
                    error!("❌ Error during shutdown: {}", e);
                } else {
                    info!("✅ Platform shutdown completed");
                }
                std::process::exit(0);
            }
            Err(err) => {
                error!("❌ Unable to listen for shutdown signal: {}", err);
            }
        }
    });
    
    Ok(())
}
