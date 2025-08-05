use clap::{Arg, Command};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json;
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;

use sniperforge::control::{TcpCommand, TcpResponse};
use sniperforge::api::bot_interface::{BotType, BotConfig, Environment, ResourceLimits, NetworkConfig, SecurityConfig, ConfigMetadata, WalletConfig, NetworkTimeouts};
use sniperforge::api::BotMetrics;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let app = Command::new("sniperforge-cli")
        .version("1.0.0")
        .about("SniperForge External Bot Control CLI")
        .arg(Arg::new("server")
            .long("server")
            .value_name("HOST:PORT")
            .help("TCP server address")
            .default_value("127.0.0.1:8888"))
        .subcommand(
            Command::new("ping")
                .about("Test server connection")
        )
        .subcommand(
            Command::new("list-bots")
                .about("List all registered bots")
        )
        .subcommand(
            Command::new("create-bot")
                .about("Create a new bot")
                .arg(Arg::new("type")
                    .value_name("BOT_TYPE")
                    .help("Bot type (enhanced-arbitrage, flash-loan, cross-chain)")
                    .required(true)
                    .index(1))
        )
        .subcommand(
            Command::new("bot-status")
                .about("Get status of a specific bot")
                .arg(Arg::new("bot-id")
                    .long("bot-id")
                    .value_name("UUID")
                    .help("Bot ID")
                    .required(true))
        )
        .subcommand(
            Command::new("bot-metrics")
                .about("Get metrics of a specific bot")
                .arg(Arg::new("bot-id")
                    .long("bot-id")
                    .value_name("UUID")
                    .help("Bot ID")
                    .required(true))
        )
        .subcommand(
            Command::new("system-metrics")
                .about("Get system-wide metrics")
        )
        .subcommand(
            Command::new("start-bot")
                .about("Start a specific bot")
                .arg(Arg::new("bot-id")
                    .long("bot-id")
                    .value_name("UUID")
                    .help("Bot ID")
                    .required(true))
        )
        .subcommand(
            Command::new("stop-bot")
                .about("Stop a specific bot")
                .arg(Arg::new("bot-id")
                    .long("bot-id")
                    .value_name("UUID")
                    .help("Bot ID")
                    .required(true))
        );

    let matches = app.get_matches();

    // Verificar si hay subcomando antes de conectar al servidor
    match matches.subcommand() {
        None => {
            println!("❌ No subcommand provided.");
            println!("\nAvailable commands:");
            println!("  ping              Test server connection");
            println!("  list-bots         List all registered bots");
            println!("  create-bot <TYPE> Create a new bot (enhanced-arbitrage, flash-loan, cross-chain)");
            println!("  bot-status        Get status of a specific bot");
            println!("  bot-metrics       Get metrics of a specific bot");
            println!("  system-metrics    Get system-wide metrics");
            println!("  start-bot         Start a specific bot");
            println!("  stop-bot          Stop a specific bot");
            println!("\nUse: {} <COMMAND> --help for more information", std::env::args().next().unwrap_or("sniperforge-cli".to_string()));
            return Ok(());
        }
        _ => {}
    }

    let server_addr = matches.get_one::<String>("server").unwrap();
    let mut client = TcpBotClient::new(server_addr).await?;

    match matches.subcommand() {
        Some(("ping", _)) => {
            let response = client.send_command(TcpCommand::Ping).await?;
            match response {
                TcpResponse::Pong => println!("✅ Server is responsive"),
                _ => println!("❌ Unexpected response: {:?}", response),
            }
        }
        Some(("list-bots", _)) => {
            let response = client.send_command(TcpCommand::ListBots).await?;
            match response {
                TcpResponse::BotList(bots) => {
                    println!("📋 Registered Bots ({}):", bots.len());
                    for bot in bots {
                        println!("  🤖 {} ({:?})", bot.id, bot.status);
                        if bot.is_default {
                            println!("    🌟 Default arbitrage bot");
                        }
                    }
                }
                TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
                _ => println!("❌ Unexpected response: {:?}", response),
            }
        }
        Some(("create-bot", sub_matches)) => {
            let bot_type_str = sub_matches.get_one::<String>("type").unwrap();
            let bot_type = match bot_type_str.as_str() {
                "enhanced-arbitrage" => BotType::EnhancedArbitrage,
                "triangular-arbitrage" => BotType::TriangularArbitrage,
                "flash-loan-arbitrage" => BotType::FlashLoanArbitrage,
                "cross-chain-arbitrage" => BotType::CrossChainArbitrage,
                "ml-analytics" => BotType::MLAnalytics,
                "portfolio-manager" => BotType::PortfolioManager,
                "real-time-dashboard" => BotType::RealTimeDashboard,
                "performance-profiler" => BotType::PerformanceProfiler,
                "pattern-analyzer" => BotType::PatternAnalyzer,
                _ => {
                    println!("❌ Invalid bot type: {}", bot_type_str);
                    println!("Available types: enhanced-arbitrage, triangular-arbitrage, flash-loan-arbitrage, cross-chain-arbitrage, ml-analytics, portfolio-manager, real-time-dashboard, performance-profiler, pattern-analyzer");
                    return Ok(());
                }
            };
            
            let config = create_default_bot_config_for_type(bot_type.clone());
            
            let response = client.send_command(TcpCommand::CreateBot { bot_type, config }).await?;
            match response {
                TcpResponse::BotCreated { bot_id } => {
                    println!("✅ Bot created successfully: {}", bot_id);
                    println!("   Type: {:?}", bot_type_str);
                    println!("   Use 'start-bot --bot-id {}' to start it", bot_id);
                }
                TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
                _ => println!("❌ Unexpected response: {:?}", response),
            }
        }
        Some(("bot-status", sub_matches)) => {
            let bot_id_str = sub_matches.get_one::<String>("bot-id").unwrap();
            let bot_id = Uuid::parse_str(bot_id_str)?;
            
            let response = client.send_command(TcpCommand::GetBotStatus { bot_id }).await?;
            match response {
                TcpResponse::BotStatus(status) => {
                    println!("🤖 Bot Status: {:?}", status);
                }
                TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
                _ => println!("❌ Unexpected response: {:?}", response),
            }
        }
        Some(("bot-metrics", sub_matches)) => {
            let bot_id_str = sub_matches.get_one::<String>("bot-id").unwrap();
            let bot_id = Uuid::parse_str(bot_id_str)?;
            
            let response = client.send_command(TcpCommand::GetBotMetrics { bot_id }).await?;
            match response {
                TcpResponse::BotMetrics(metrics) => {
                    print_bot_metrics(&metrics);
                }
                TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
                _ => println!("❌ Unexpected response: {:?}", response),
            }
        }
        Some(("system-metrics", _)) => {
            let response = client.send_command(TcpCommand::GetSystemMetrics).await?;
            match response {
                TcpResponse::SystemMetrics(metrics) => {
                    println!("📊 System Metrics:");
                    println!("   - Total Bots: {}", metrics.total_bots);
                    println!("   - Running Bots: {}", metrics.running_bots);
                    println!("   - Total Profit: ${:.2}", metrics.total_profit);
                    println!("   - Total Trades: {}", metrics.total_trades);
                    println!("   - Uptime: {:.2} hours", metrics.uptime_seconds as f64 / 3600.0);
                }
                TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
                _ => println!("❌ Unexpected response: {:?}", response),
            }
        }
        Some(("start-bot", sub_matches)) => {
            let bot_id_str = sub_matches.get_one::<String>("bot-id").unwrap();
            let bot_id = Uuid::parse_str(bot_id_str)?;
            
            // Create a default config for now
            let config = create_default_bot_config(bot_id);
            
            let response = client.send_command(TcpCommand::StartBot { bot_id, config }).await?;
            match response {
                TcpResponse::BotStarted { bot_id: started_id } => {
                    println!("✅ Bot started successfully: {}", started_id);
                }
                TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
                _ => println!("❌ Unexpected response: {:?}", response),
            }
        }
        Some(("stop-bot", sub_matches)) => {
            let bot_id_str = sub_matches.get_one::<String>("bot-id").unwrap();
            let bot_id = Uuid::parse_str(bot_id_str)?;
            
            let response = client.send_command(TcpCommand::StopBot { bot_id }).await?;
            match response {
                TcpResponse::BotStopped { bot_id: stopped_id } => {
                    println!("✅ Bot stopped successfully: {}", stopped_id);
                }
                TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
                _ => println!("❌ Unexpected response: {:?}", response),
            }
        }
        Some((unknown_cmd, _)) => {
            println!("❌ Unknown subcommand: {}", unknown_cmd);
        }
        None => {
            // Este caso ya fue manejado arriba, pero necesario para completitud
            unreachable!("None case should have been handled earlier");
        }
    }

    Ok(())
}

fn print_bot_metrics(metrics: &BotMetrics) {
    println!("📈 Bot Metrics:");
    println!("   - Trades Executed: {}", metrics.trading.trades_executed);
    println!("   - Total P&L: ${:.2}", metrics.trading.total_pnl_usd);
    println!("   - Success Rate: {:.1}%", metrics.trading.success_rate * 100.0);
    println!("   - Uptime: {} seconds", metrics.operational.uptime_seconds);
    println!("   - CPU Usage: {:.1}%", metrics.performance.cpu_usage_percent);
    println!("   - Memory Usage: {} MB", metrics.performance.memory_usage_mb);
}

fn create_default_bot_config(_bot_id: Uuid) -> BotConfig {
    create_default_bot_config_for_type(BotType::EnhancedArbitrage)
}

fn create_default_bot_config_for_type(bot_type: BotType) -> BotConfig {
    let bot_id = Uuid::new_v4();
    
    // Read real environment variables or use conservative defaults
    let solana_rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()); // Devnet by default for safety
    
    let max_cpu = std::env::var("BOT_MAX_CPU")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0); // Conservative default: 1 CPU core
    
    let max_memory_mb = std::env::var("BOT_MAX_MEMORY_MB")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(256); // Conservative default: 256MB
    
    let rpc_timeout = std::env::var("RPC_TIMEOUT_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10); // Conservative default: 10 seconds

    // Adjust resources based on bot type
    let (memory_multiplier, name_suffix) = match bot_type {
        BotType::EnhancedArbitrage => (1.0, "Enhanced Arbitrage"),
        BotType::TriangularArbitrage => (1.2, "Triangular Arbitrage"),
        BotType::FlashLoanArbitrage => (1.5, "Flash Loan Arbitrage"),
        BotType::CrossChainArbitrage => (2.0, "Cross Chain Arbitrage"),
        BotType::MLAnalytics => (1.8, "ML Analytics"),
        BotType::PortfolioManager => (1.3, "Portfolio Manager"),
        BotType::RealTimeDashboard => (0.8, "Real Time Dashboard"),
        BotType::PerformanceProfiler => (1.1, "Performance Profiler"),
        BotType::PatternAnalyzer => (1.6, "Pattern Analyzer"),
    };

    BotConfig {
        config_id: Uuid::new_v4(),
        bot_id,
        bot_type: bot_type.clone(),
        environment: Environment::Development, // Safe default
        parameters: serde_json::Value::Null,
        resources: ResourceLimits {
            max_cpu,
            max_memory_mb: (max_memory_mb as f64 * memory_multiplier) as u64,
            max_disk_mb: 128, // Conservative: 128MB disk
            max_network_mbps: Some(10), // Conservative: 10 Mbps
        },
        network: NetworkConfig {
            solana_rpc_urls: vec![solana_rpc_url], // Real environment-based URL
            websocket_urls: vec![],
            api_endpoints: HashMap::new(),
            timeouts: NetworkTimeouts {
                rpc_timeout_seconds: rpc_timeout,
                websocket_timeout_seconds: rpc_timeout,
                api_timeout_seconds: rpc_timeout,
            },
        },
        security: SecurityConfig {
            wallet: WalletConfig {
                wallet_type: "keypair".to_string(),
                address: "".to_string(),
                private_key_path: None,
                use_env_keys: true, // Always use environment for security
            },
            api_keys: HashMap::new(),
            encryption_enabled: true, // Security by default
            auth_required: true,      // Security by default
        },
        metadata: ConfigMetadata {
            name: format!("{} Bot Config {}", name_suffix, bot_id), // Real bot-specific name
            version: env!("CARGO_PKG_VERSION").to_string(), // Real package version
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: format!("CLI-{}", std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())), // Real user
            tags: vec!["cli-created".to_string(), "development".to_string()],
        },
    }
}

struct TcpBotClient {
    stream: TcpStream,
}

impl TcpBotClient {
    async fn new(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }

    async fn send_command(&mut self, command: TcpCommand) -> Result<TcpResponse> {
        // Send command
        let command_json = serde_json::to_string(&command)?;
        let data = format!("{}\n", command_json);
        self.stream.write_all(data.as_bytes()).await?;

        // Read response
        let mut buffer = vec![0; 8192];
        let n = self.stream.read(&mut buffer).await?;
        let response_str = String::from_utf8_lossy(&buffer[..n]);
        
        // Parse response
        let response: TcpResponse = serde_json::from_str(response_str.trim())?;
        Ok(response)
    }
}
