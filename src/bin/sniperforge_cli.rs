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
            Command::new("status")
                .about("Get status of a specific bot")
                .arg(Arg::new("bot-id")
                    .long("bot-id")
                    .value_name("UUID")
                    .help("Bot ID")
                    .required(true))
        )
        .subcommand(
            Command::new("metrics")
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
        Some(("status", sub_matches)) => {
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
        Some(("metrics", sub_matches)) => {
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
                TcpResponse::Success(_) => println!("✅ Bot started successfully"),
                TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
                _ => println!("❌ Unexpected response: {:?}", response),
            }
        }
        Some(("stop-bot", sub_matches)) => {
            let bot_id_str = sub_matches.get_one::<String>("bot-id").unwrap();
            let bot_id = Uuid::parse_str(bot_id_str)?;
            
            let response = client.send_command(TcpCommand::StopBot { bot_id }).await?;
            match response {
                TcpResponse::Success(_) => println!("✅ Bot stopped successfully"),
                TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
                _ => println!("❌ Unexpected response: {:?}", response),
            }
        }
        _ => {
            println!("❌ No subcommand provided. Use --help for usage information.");
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

fn create_default_bot_config(bot_id: Uuid) -> BotConfig {
    BotConfig {
        config_id: Uuid::new_v4(),
        bot_id,
        bot_type: BotType::EnhancedArbitrage,
        environment: Environment::Development,
        parameters: serde_json::Value::Null,
        resources: ResourceLimits {
            max_cpu: 2.0,
            max_memory_mb: 1024,
            max_disk_mb: 512,
            max_network_mbps: Some(100),
        },
        network: NetworkConfig {
            solana_rpc_urls: vec!["https://api.mainnet-beta.solana.com".to_string()],
            websocket_urls: vec![],
            api_endpoints: HashMap::new(),
            timeouts: NetworkTimeouts {
                rpc_timeout_seconds: 30,
                websocket_timeout_seconds: 30,
                api_timeout_seconds: 30,
            },
        },
        security: SecurityConfig {
            wallet: WalletConfig {
                wallet_type: "keypair".to_string(),
                address: "".to_string(),
                private_key_path: None,
                use_env_keys: true,
            },
            api_keys: HashMap::new(),
            encryption_enabled: false,
            auth_required: false,
        },
        metadata: ConfigMetadata {
            name: "Default Bot Config".to_string(),
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: "CLI".to_string(),
            tags: vec!["default".to_string()],
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
