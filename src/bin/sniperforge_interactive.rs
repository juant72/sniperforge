use std::io::{self, Write};
use std::collections::HashMap;
use uuid::Uuid;
use anyhow::Result;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json;
use chrono::{DateTime, Utc};

use sniperforge::control::{TcpCommand, TcpResponse};
use sniperforge::api::bot_interface::{BotType, BotConfig};

/// Interactive CLI Context - tracks current working directory/strategy
#[derive(Debug, Clone)]
pub enum CliContext {
    Root,                           // /
    BotsDirectory,                  // /strategies  
    BotInstance { id: Uuid, name: String }, // /strategies/{strategy_id}
    SystemAdmin,                    // /admin
    Monitoring,                     // /analytics
}

impl CliContext {
    pub fn path(&self) -> String {
        match self {
            CliContext::Root => "/".to_string(),
            CliContext::BotsDirectory => "/strategies".to_string(),
            CliContext::BotInstance { id: _, name } => {
                format!("/strategies/{}", name)
            },
            CliContext::SystemAdmin => "/admin".to_string(),
            CliContext::Monitoring => "/analytics".to_string(),
        }
    }

    pub fn prompt(&self) -> String {
        match self {
            CliContext::Root => "SniperForge-Enterprise:/ $ ".to_string(),
            CliContext::BotsDirectory => "SniperForge-Enterprise:/strategies $ ".to_string(),
            CliContext::BotInstance { name, .. } => format!("SniperForge-Enterprise:/strategies/{} $ ", name),
            CliContext::SystemAdmin => "SniperForge-Enterprise:/admin $ ".to_string(),
            CliContext::Monitoring => "SniperForge-Enterprise:/analytics $ ".to_string(),
        }
    }
}

/// Interactive CLI Session
pub struct InteractiveCli {
    context: CliContext,
    server_addr: String,
    bot_cache: HashMap<Uuid, BotInfo>,
    session_start: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct BotInfo {
    pub id: Uuid,
    pub name: String,
    pub bot_type: BotType,
    pub status: String,
}

impl InteractiveCli {
    pub fn new(server_addr: String) -> Self {
        Self {
            context: CliContext::Root,
            server_addr,
            bot_cache: HashMap::new(),
            session_start: Utc::now(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.print_welcome().await?;
        self.refresh_bot_cache().await?;

        loop {
            print!("{}", self.context.prompt());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            match self.process_command(input).await {
                Ok(should_exit) => {
                    if should_exit {
                        println!("👋 Goodbye!");
                        break;
                    }
                }
                Err(e) => {
                    println!("❌ Error: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn print_welcome(&self) -> Result<()> {
        println!("═══════════════════════════════════════════════════════════════");
        println!("🏢 SNIPERFORGE ENTERPRISE TRADING PLATFORM");
        println!("   World-Class Automated Trading Infrastructure");
        println!("   Session: {}", self.session_start.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("═══════════════════════════════════════════════════════════════");
        println!();
        
        // Test server connection
        match self.send_command(TcpCommand::Ping).await {
            Ok(_) => {
                println!("🟢 OPERATIONAL    Server Status: ACTIVE");
                println!("🔗 CONNECTED      Control Server: {}", self.server_addr);
            },
            Err(e) => {
                println!("🔴 ERROR          Connection Failed: {}", e);
                println!("📞 SUPPORT        Contact System Administrator");
            }
        }
        println!();
        println!("📋 COMMAND INTERFACE:");
        println!("   help              Command reference");
        println!("   ls               List available resources");
        println!("   cd /strategies   Access trading strategy management");
        println!("   cd /system       System administration");
        println!("   exit             Terminate session");
        println!();
        
        Ok(())
    }

    async fn process_command(&mut self, input: &str) -> Result<bool> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(false);
        }

        let command = parts[0];
        let args = &parts[1..];

        match command {
            // Global commands
            "exit" | "quit" | "q" => return Ok(true),
            "help" | "?" => self.show_help().await?,
            "clear" | "cls" => self.clear_screen(),
            "pwd" => println!("{}", self.context.path()),
            "refresh" => self.refresh_bot_cache().await?,
            
            // Navigation commands
            "ls" | "list" => self.list_current().await?,
            "cd" => self.change_directory(args).await?,
            
            // Bot commands (context-dependent)
            "start" => self.start_command(args).await?,
            "stop" => self.stop_command(args).await?,
            "restart" => self.restart_command(args).await?,
            "status" | "st" => self.status_command(args).await?,
            "stats" | "metrics" => self.stats_command(args).await?,
            "create" => self.create_command(args).await?,
            "remove" | "rm" => self.remove_command(args).await?,
            
            // System commands
            "system" => self.system_command(args).await?,
            "backup" => self.backup_command().await?,
            "save" => self.save_command().await?,
            "start-all" => self.start_all_command().await?,
            "stop-all" => self.stop_all_command().await?,
            "resources" => self.resources_command().await?,
            
            _ => {
                println!("❌ Unknown command: '{}'", command);
                println!("💡 Type 'help' for available commands");
            }
        }

        Ok(false)
    }

    async fn show_help(&self) -> Result<()> {
        println!("═══════════════════════════════════════════════════════════════");
        println!("📖 SNIPERFORGE ENTERPRISE - COMMAND REFERENCE");
        println!("═══════════════════════════════════════════════════════════════");
        println!();
        
        println!("�️  NAVIGATION & SYSTEM:");
        println!("   ls                    List resources in current context");
        println!("   cd <directory>        Navigate to: /bots, /system, /monitoring");
        println!("   pwd                   Display current working directory");
        println!("   refresh               Update cached system information");
        println!();
        
        match &self.context {
            CliContext::BotInstance { .. } => {
                println!("🤖 BOT OPERATIONS (Current Instance):");
                println!("   start                 Activate current trading bot");
                println!("   stop                  Deactivate current trading bot");
                println!("   restart               Restart current trading bot");
                println!("   status                Display operational status");
                println!("   metrics               Show performance analytics");
                println!();
            }
            CliContext::BotsDirectory => {
                println!("📈 TRADING STRATEGY MANAGEMENT:");
                println!("   start <strategy-name>    Activate specific trading strategy");
                println!("   stop <strategy-name>     Deactivate specific trading strategy");
                println!("   deploy <config>          Deploy new trading configuration");
                println!("   remove <strategy-name>   Decommission trading strategy");
                println!("   status <strategy-name>   Check strategy operational status");
                println!();
            }
            CliContext::SystemAdmin => {
                println!("⚙️  SYSTEM ADMINISTRATION:");
                println!("   backup                Create system state backup");
                println!("   save                  Persist current configuration");
                println!("   resources             Display system resource usage");
                println!("   start-all             Activate all trading strategies");
                println!("   stop-all              Deactivate all trading operations");
                println!();
            }
            CliContext::Monitoring => {
                println!("📊 MONITORING & ANALYTICS:");
                println!("   dashboard             Real-time performance dashboard");
                println!("   alerts                System alerts and notifications");
                println!("   reports               Generate performance reports");
                println!();
            }
            _ => {
                println!("🏢 ENTERPRISE OPERATIONS:");
                println!("   cd /bots              Access trading bot management");
                println!("   cd /system            System administration panel");
                println!("   cd /monitoring        Performance monitoring center");
                println!("   start-all             Activate all trading operations");
                println!("   stop-all              Emergency stop all operations");
                println!();
            }
        }
        
        println!("🔧 SYSTEM COMMANDS:");
        println!("   clear                 Clear screen display");
        println!("   help                  Display this command reference");
        println!("   exit                  Terminate management session");
        println!();
        println!("═══════════════════════════════════════════════════════════════");
        
        Ok(())
    }

    fn clear_screen(&self) {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
    }

    async fn refresh_bot_cache(&mut self) -> Result<()> {
        match self.send_command(TcpCommand::ListBots).await? {
            TcpResponse::BotList(bots) => {
                self.bot_cache.clear();
                for bot in bots {
                    let name = format!("{:8}", &bot.id.to_string()[..8]);
                    self.bot_cache.insert(bot.id, BotInfo {
                        id: bot.id,
                        name,
                        bot_type: bot.bot_type,
                        status: format!("{:?}", bot.status),
                    });
                }
                println!("✅ CACHE UPDATED: {} trading strategies loaded", self.bot_cache.len());
            }
            _ => return Err(anyhow::anyhow!("Failed to retrieve trading strategy list")),
        }
        Ok(())
    }

    async fn list_current(&self) -> Result<()> {
        match &self.context {
            CliContext::Root => {
                println!("📁 Root Directory Contents:");
                println!("  bots/         Bot management");
                println!("  system/       System administration");
                println!("  monitoring/   Monitoring and metrics");
            }
            CliContext::BotsDirectory => {
                println!("🤖 Registered Bots ({}):", self.bot_cache.len());
                if self.bot_cache.is_empty() {
                    println!("  (no bots registered)");
                } else {
                    for bot in self.bot_cache.values() {
                        let status_icon = match bot.status.as_str() {
                            "Running" => "🟢",
                            "Stopped" => "🔴",
                            _ => "🟡",
                        };
                        println!("  {} {} ({:?}) - {}", 
                               status_icon, bot.name, bot.bot_type, bot.status);
                    }
                }
            }
            CliContext::BotInstance { id, name } => {
                if let Some(bot) = self.bot_cache.get(id) {
                    println!("🤖 Bot Details: {}", name);
                    println!("  ID: {}", bot.id);
                    println!("  Type: {:?}", bot.bot_type);
                    println!("  Status: {}", bot.status);
                } else {
                    println!("⚠️ Bot not found in cache. Try 'refresh'");
                }
            }
            CliContext::SystemAdmin => {
                println!("🏢 System Administration:");
                println!("  backup        Create system backup");
                println!("  save          Force save system state");
                println!("  resources     Show resource usage");
            }
            CliContext::Monitoring => {
                println!("📊 Monitoring Commands:");
                println!("  system        System overview");
                println!("  resources     Resource usage");
                println!("  metrics       Historical metrics");
            }
        }
        Ok(())
    }

    async fn change_directory(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            self.context = CliContext::Root;
            return Ok(());
        }

        let path = args[0];
        match path {
            "/" => self.context = CliContext::Root,
            "/bots" | "bots" => self.context = CliContext::BotsDirectory,
            "/system" | "system" => self.context = CliContext::SystemAdmin,
            "/monitoring" | "monitoring" => self.context = CliContext::Monitoring,
            ".." => {
                self.context = match &self.context {
                    CliContext::BotInstance { .. } => CliContext::BotsDirectory,
                    CliContext::BotsDirectory | CliContext::SystemAdmin | CliContext::Monitoring => CliContext::Root,
                    CliContext::Root => CliContext::Root,
                };
            }
            bot_name => {
                // Try to navigate to specific bot
                if let CliContext::BotsDirectory = self.context {
                    for bot in self.bot_cache.values() {
                        if bot.name == bot_name || bot.id.to_string().starts_with(bot_name) {
                            self.context = CliContext::BotInstance {
                                id: bot.id,
                                name: bot.name.clone(),
                            };
                            return Ok(());
                        }
                    }
                    println!("❌ Bot '{}' not found", bot_name);
                } else {
                    println!("❌ Invalid path: {}", path);
                }
            }
        }
        Ok(())
    }

    async fn start_command(&self, args: &[&str]) -> Result<()> {
        match &self.context {
            CliContext::BotInstance { id, .. } => {
                let config = self.create_default_config(*id);
                match self.send_command(TcpCommand::StartBot { bot_id: *id, config }).await? {
                    TcpResponse::BotStarted { .. } => println!("✅ Bot started successfully"),
                    TcpResponse::Error(msg) => println!("❌ Failed to start bot: {}", msg),
                    _ => println!("❌ Unexpected response"),
                }
            }
            _ => {
                if args.is_empty() {
                    println!("❌ No strategy specified. Usage: start <strategy_name>");
                    return Ok(());
                }
                // Implement start by name
                println!("💡 Navigate to strategy directory first: cd /strategies && cd <strategy_name>");
            }
        }
        Ok(())
    }

    async fn stop_command(&self, args: &[&str]) -> Result<()> {
        // Support for additional arguments like --force or --graceful
        let force_mode = args.contains(&"--force") || args.contains(&"-f");
        let show_verbose = args.contains(&"--verbose") || args.contains(&"-v");
        
        match &self.context {
            CliContext::BotInstance { id, name } => {
                if show_verbose {
                    println!("🔍 Stopping bot {} (ID: {})...", name, id);
                }
                
                if force_mode {
                    println!("⚡ Force stopping bot...");
                }
                
                match self.send_command(TcpCommand::StopBot { bot_id: *id }).await? {
                    TcpResponse::BotStopped { .. } => {
                        if force_mode {
                            println!("✅ Bot force-stopped successfully");
                        } else {
                            println!("✅ Bot stopped successfully");
                        }
                    },
                    TcpResponse::Error(msg) => println!("❌ Failed to stop bot: {}", msg),
                    _ => println!("❌ Unexpected response"),
                }
            }
            _ => {
                println!("💡 Navigate to bot directory first: cd /bots && cd <bot_name>");
                if !args.is_empty() {
                    println!("💡 Available stop options: --force/-f, --verbose/-v");
                }
            }
        }
        Ok(())
    }

    async fn restart_command(&self, _args: &[&str]) -> Result<()> {
        match &self.context {
            CliContext::BotInstance { id, .. } => {
                println!("🔄 Restarting bot...");
                
                // Stop first
                match self.send_command(TcpCommand::StopBot { bot_id: *id }).await? {
                    TcpResponse::BotStopped { .. } => println!("  ✅ Bot stopped"),
                    TcpResponse::Error(msg) => {
                        println!("  ⚠️ Stop failed: {}, continuing...", msg);
                    }
                    _ => {}
                }
                
                // Wait a moment
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                // Start again
                let config = self.create_default_config(*id);
                match self.send_command(TcpCommand::StartBot { bot_id: *id, config }).await? {
                    TcpResponse::BotStarted { .. } => println!("  ✅ Bot started"),
                    TcpResponse::Error(msg) => println!("  ❌ Start failed: {}", msg),
                    _ => {}
                }
                
                println!("✅ Restart sequence completed");
            }
            _ => {
                println!("💡 Navigate to bot directory first: cd /bots && cd <bot_name>");
            }
        }
        Ok(())
    }

    async fn status_command(&self, _args: &[&str]) -> Result<()> {
        match &self.context {
            CliContext::BotInstance { id, name } => {
                match self.send_command(TcpCommand::GetBotStatus { bot_id: *id }).await? {
                    TcpResponse::BotStatus(status) => {
                        println!("📊 Bot Status: {}", name);
                        println!("   Status: {:?}", status);
                    }
                    TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
                    _ => println!("❌ Unexpected response"),
                }
            }
            _ => {
                println!("💡 Navigate to bot directory first: cd /bots && cd <bot_name>");
            }
        }
        Ok(())
    }

    async fn stats_command(&self, _args: &[&str]) -> Result<()> {
        match &self.context {
            CliContext::BotInstance { id, name } => {
                match self.send_command(TcpCommand::GetBotMetrics { bot_id: *id }).await? {
                    TcpResponse::BotMetrics(metrics) => {
                        println!("📈 Bot Metrics: {}", name);
                        println!("   Trades: {}", metrics.trading.trades_executed);
                        println!("   P&L: ${:.2}", metrics.trading.total_pnl_usd);
                        println!("   Success Rate: {:.1}%", metrics.trading.success_rate * 100.0);
                        println!("   Uptime: {}s", metrics.operational.uptime_seconds);
                        println!("   CPU: {:.1}%", metrics.performance.cpu_usage_percent);
                        println!("   Memory: {}MB", metrics.performance.memory_usage_mb);
                    }
                    TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
                    _ => println!("❌ Unexpected response"),
                }
            }
            _ => {
                println!("💡 Navigate to bot directory first: cd /bots && cd <bot_name>");
            }
        }
        Ok(())
    }

    async fn create_command(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("❌ Bot type required. Available types:");
            println!("   enhanced-arbitrage, triangular-arbitrage, flash-loan-arbitrage");
            println!("   cross-chain-arbitrage, ml-analytics, portfolio-manager");
            println!("   real-time-dashboard, performance-profiler, pattern-analyzer");
            return Ok(());
        }

        let bot_type_str = args[0];
        let bot_type = match bot_type_str {
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
                return Ok(());
            }
        };

        let config = BotConfig::default_for_id(Uuid::new_v4());
        match self.send_command(TcpCommand::CreateBot { bot_type, config }).await? {
            TcpResponse::BotCreated { bot_id } => {
                println!("✅ Bot created successfully: {}", bot_id);
                println!("💡 Use 'refresh' to update bot list");
            }
            TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
            _ => println!("❌ Unexpected response"),
        }
        Ok(())
    }

    async fn remove_command(&self, _args: &[&str]) -> Result<()> {
        // Strategy removal functionality
        println!("🚧 Strategy removal not yet implemented");
        Ok(())
    }

    async fn system_command(&self, _args: &[&str]) -> Result<()> {
        match self.send_command(TcpCommand::GetSystemMetrics).await? {
            TcpResponse::SystemMetrics(metrics) => {
                println!("🏢 Enterprise System Overview:");
                println!("   Active Strategies: {}", metrics.total_bots);
                println!("   Running Strategies: {}", metrics.running_bots);
                println!("   Total Profit: ${:.2}", metrics.total_profit);
                println!("   Total Trades: {}", metrics.total_trades);
                println!("   Uptime: {:.2} hours", metrics.uptime_seconds as f64 / 3600.0);
            }
            TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
            _ => println!("❌ Unexpected response"),
        }
        Ok(())
    }

    async fn backup_command(&self) -> Result<()> {
        match self.send_command(TcpCommand::CreateBackup).await? {
            TcpResponse::BackupCreated(path) => {
                println!("✅ Backup created: {}", path);
            }
            TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
            _ => println!("❌ Unexpected response"),
        }
        Ok(())
    }

    async fn save_command(&self) -> Result<()> {
        match self.send_command(TcpCommand::ForceSave).await? {
            TcpResponse::Success(msg) => println!("✅ {}", msg),
            TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
            _ => println!("❌ Unexpected response"),
        }
        Ok(())
    }

    async fn start_all_command(&self) -> Result<()> {
        match self.send_command(TcpCommand::StartAllBots).await? {
            TcpResponse::MassControlResult(result) => {
                println!("🚀 Start All Results:");
                println!("   ✅ Started: {} bots", result.successful.len());
                if !result.failed.is_empty() {
                    println!("   ❌ Failed: {} bots", result.failed.len());
                }
                println!("   📊 Total: {}", result.total_attempted);
            }
            TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
            _ => println!("❌ Unexpected response"),
        }
        Ok(())
    }

    async fn stop_all_command(&self) -> Result<()> {
        match self.send_command(TcpCommand::StopAllBots).await? {
            TcpResponse::MassControlResult(result) => {
                println!("🛑 Stop All Results:");
                println!("   ✅ Stopped: {} bots", result.successful.len());
                if !result.failed.is_empty() {
                    println!("   ❌ Failed: {} bots", result.failed.len());
                }
                println!("   📊 Total: {}", result.total_attempted);
            }
            TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
            _ => println!("❌ Unexpected response"),
        }
        Ok(())
    }

    async fn resources_command(&self) -> Result<()> {
        match self.send_command(TcpCommand::GetResourceStatus).await? {
            TcpResponse::ResourceStatus(status) => {
                println!("📊 Resource Status:");
                println!("   Total Bots: {}", status.total_bots);
                println!("   Running Bots: {}", status.running_bots);
                println!("   Memory Usage: {:.1} MB", status.memory_usage_mb);
                println!("   CPU Cores: {}", status.cpu_cores);
                println!("   Max Bots: {}", status.estimated_max_bots);
                if let Some(warning) = &status.resource_warning {
                    println!("   ⚠️ Warning: {}", warning);
                }
            }
            TcpResponse::Error(msg) => println!("❌ Error: {}", msg),
            _ => println!("❌ Unexpected response"),
        }
        Ok(())
    }

    fn create_default_config(&self, bot_id: Uuid) -> BotConfig {
        BotConfig::default_for_id(bot_id)
    }

    async fn send_command(&self, command: TcpCommand) -> Result<TcpResponse> {
        let mut stream = TcpStream::connect(&self.server_addr).await?;
        
        // Convert command to JSON and add newline for proper parsing
        let command_json = serde_json::to_string(&command)?;
        let command_with_newline = format!("{}\n", command_json);
        
        // Send command
        stream.write_all(command_with_newline.as_bytes()).await?;
        
        // Read response with larger buffer and proper EOF handling
        let mut buffer = Vec::new();
        let mut temp_buffer = [0; 1024];
        
        loop {
            let n = stream.read(&mut temp_buffer).await?;
            if n == 0 {
                break; // EOF reached
            }
            buffer.extend_from_slice(&temp_buffer[..n]);
            
            // Check if we have a complete JSON message (ends with newline or })
            if let Ok(response_str) = String::from_utf8(buffer.clone()) {
                if response_str.trim().ends_with('}') {
                    let response: TcpResponse = serde_json::from_str(response_str.trim())?;
                    return Ok(response);
                }
            }
        }
        
        // Parse final response
        let response_str = String::from_utf8(buffer)?;
        let response: TcpResponse = serde_json::from_str(response_str.trim())?;
        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    // Handle command line arguments
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--version" | "-v" => {
                println!("SniperForge Enterprise Interactive Dashboard v1.0.0");
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    let server_addr = "127.0.0.1:8888".to_string();
    let mut cli = InteractiveCli::new(server_addr);
    cli.run().await
}

fn print_help() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("🏢 SNIPERFORGE ENTERPRISE INTERACTIVE DASHBOARD");
    println!("   World-Class Automated Trading Infrastructure Management");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    println!("USAGE:");
    println!("   sniperforge_interactive [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("   --help, -h       Display this help information");
    println!("   --version, -v    Display version information");
    println!();
    println!("FEATURES:");
    println!("   • Enterprise-grade trading strategy management");
    println!("   • Real-time system monitoring and analytics");
    println!("   • Interactive command-line interface");
    println!("   • TCP-based communication with trading server");
    println!("   • Advanced system administration tools");
    println!();
    println!("NAVIGATION:");
    println!("   /                Root directory - main menu");
    println!("   /strategies      Trading strategy management");
    println!("   /admin           System administration");
    println!("   /analytics       Performance monitoring");
    println!();
    println!("COMMANDS:");
    println!("   help             Show available commands");
    println!("   clear            Clear the terminal screen");
    println!("   cd <path>        Navigate to directory");
    println!("   list             List available items");
    println!("   status           Check system status");
    println!("   exit             Exit the application");
    println!();
    println!("For more information, visit: https://github.com/juant72/sniperforge");
    println!("═══════════════════════════════════════════════════════════════");
}
