# 🔧 CLI DIRECTA - Implementación Detallada

**Fecha:** 4 de Agosto, 2025  
**Milestone:** Semana 1-2 Agosto 2025  
**Estado:** Especificación Técnica  

## 🎯 **OBJETIVO PRIORIDAD 1**

Implementar **CLI Directa** que se comunique directamente con los bots sin overhead de HTTP/REST, optimizada para **trading de alta frecuencia** y **control de bots en tiempo real**.

---

## 📊 **ANÁLISIS ARQUITECTURA ACTUAL**

### **CLI Existente (✅ Funcional):**
```
Ubicación: old-root-archive/src/cli.rs (3,500+ líneas)
Estado: Completamente funcional
Comandos: start, status, config, wallet, test, interactive
```

### **Bot Interface Existente (✅ Funcional):**
```
Ubicación: src/api/bot_interface.rs  
Trait: BotInterface con métodos async
Implementación: EnhancedArbitrageBot funcional
```

### **Gap Identificado (❌ Falta):**
```
CLI ↔ Bot Interface Communication: NO EXISTE
- CLI llama a HTTP API (lento)
- No hay comunicación directa
- Sin control directo del bot lifecycle
```

---

## 🏗️ **DISEÑO CLI DIRECTA**

### **Arquitectura Propuesta:**

```mermaid
graph TB
    subgraph "CLI Layer"
        A[CLI Commands]
        B[Command Parser]
        C[Direct Bot Manager]
    end
    
    subgraph "Bot Management"
        D[Bot Factory]
        E[Bot Registry]
        F[Config Manager]
        G[State Manager]
    end
    
    subgraph "Bot Instances"
        H[Sniper Bot]
        I[Arbitrage Bot]
        J[Analytics Bot]
    end
    
    A --> B
    B --> C
    C --> D
    C --> E
    C --> F
    C --> G
    
    D --> H
    D --> I
    D --> J
    
    E --> H
    E --> I
    E --> J
```

### **Ventajas CLI Directa:**
```
✅ Latencia: <1ms (vs 50-100ms HTTP)
✅ Control: Estado directo de bots
✅ Performance: Sin serialización JSON
✅ Reliability: Sin network layer  
✅ Debugging: Stack traces directo
✅ Real-time: Streaming de datos
```

---

## 🔧 **IMPLEMENTACIÓN TÉCNICA**

### **Estructura de Archivos:**
```
src/cli/
├── mod.rs                    # Module exports
├── direct_interface.rs       # Core DirectBotManager  
├── command_handlers.rs       # Command execution logic
├── sniper_commands.rs        # Sniper bot specific commands
├── bot_factory.rs           # Direct bot instantiation
├── state_manager.rs         # Bot state management
└── config_loader.rs         # Configuration handling
```

### **1. Core Direct Interface:**

```rust
// src/cli/direct_interface.rs
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::RwLock;
use anyhow::Result;
use crate::api::bot_interface::{BotInterface, BotConfig, BotStatus, BotMetrics};

pub struct DirectBotManager {
    /// Active bot instances
    bots: RwLock<HashMap<Uuid, Box<dyn BotInterface>>>,
    
    /// Bot factory for creating new instances
    factory: BotFactory,
    
    /// Configuration manager
    config_manager: ConfigManager,
    
    /// State persistence
    state_manager: StateManager,
    
    /// Performance metrics
    metrics_collector: MetricsCollector,
}

impl DirectBotManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            bots: RwLock::new(HashMap::new()),
            factory: BotFactory::new(),
            config_manager: ConfigManager::new()?,
            state_manager: StateManager::new()?,
            metrics_collector: MetricsCollector::new(),
        })
    }
    
    /// Create a new sniper bot instance
    pub async fn create_sniper_bot(&self, config: SniperBotConfig) -> Result<Uuid> {
        let bot_id = Uuid::new_v4();
        
        // Create bot instance directly
        let bot = self.factory.create_sniper_bot(bot_id, config).await?;
        
        // Register in local registry
        {
            let mut bots = self.bots.write().await;
            bots.insert(bot_id, bot);
        }
        
        // Persist state
        self.state_manager.save_bot_state(bot_id).await?;
        
        println!("✅ Created Sniper Bot: {}", bot_id);
        Ok(bot_id)
    }
    
    /// Start bot with direct communication
    pub async fn start_bot(&self, bot_id: Uuid, config: BotConfig) -> Result<()> {
        let mut bots = self.bots.write().await;
        
        if let Some(bot) = bots.get_mut(&bot_id) {
            // Direct method call - no HTTP overhead
            bot.start(config).await?;
            
            println!("🚀 Started bot: {}", bot_id);
            
            // Start metrics collection
            self.metrics_collector.start_tracking(bot_id).await;
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Bot not found: {}", bot_id))
        }
    }
    
    /// Get real-time bot status - direct access
    pub async fn get_bot_status(&self, bot_id: Uuid) -> Result<BotStatus> {
        let bots = self.bots.read().await;
        
        if let Some(bot) = bots.get(&bot_id) {
            // Direct method call - immediate response
            let status = bot.status().await;
            Ok(status)
        } else {
            Err(anyhow::anyhow!("Bot not found: {}", bot_id))
        }
    }
    
    /// Get real-time metrics - direct access
    pub async fn get_bot_metrics(&self, bot_id: Uuid) -> Result<BotMetrics> {
        let bots = self.bots.read().await;
        
        if let Some(bot) = bots.get(&bot_id) {
            let metrics = bot.metrics().await;
            Ok(metrics)
        } else {
            Err(anyhow::anyhow!("Bot not found: {}", bot_id))
        }
    }
    
    /// Stream real-time updates (for monitoring)
    pub async fn stream_bot_updates(&self, bot_id: Uuid) -> Result<impl Stream<Item = BotUpdate>> {
        // Implementation for real-time streaming
        todo!("Implement real-time streaming")
    }
    
    /// Stop bot gracefully
    pub async fn stop_bot(&self, bot_id: Uuid) -> Result<()> {
        let mut bots = self.bots.write().await;
        
        if let Some(bot) = bots.get_mut(&bot_id) {
            bot.stop().await?;
            println!("🛑 Stopped bot: {}", bot_id);
            
            // Stop metrics collection
            self.metrics_collector.stop_tracking(bot_id).await;
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Bot not found: {}", bot_id))
        }
    }
    
    /// List all active bots
    pub async fn list_bots(&self) -> Result<Vec<BotSummary>> {
        let bots = self.bots.read().await;
        let mut summaries = Vec::new();
        
        for (id, bot) in bots.iter() {
            let status = bot.status().await;
            let metrics = bot.metrics().await;
            
            summaries.push(BotSummary {
                id: *id,
                bot_type: bot.bot_type(),
                status,
                metrics,
            });
        }
        
        Ok(summaries)
    }
}

#[derive(Debug)]
pub struct BotSummary {
    pub id: Uuid,
    pub bot_type: BotType,
    pub status: BotStatus,
    pub metrics: BotMetrics,
}
```

### **2. Sniper Commands Implementation:**

```rust
// src/cli/sniper_commands.rs
use clap::{Arg, ArgMatches, Command};
use crate::cli::direct_interface::DirectBotManager;

pub fn build_sniper_commands() -> Command {
    Command::new("sniper")
        .about("🎯 Liquidity Sniper Bot Management")
        .subcommand(
            Command::new("create")
                .about("Create new sniper bot instance")
                .arg(Arg::new("config")
                    .long("config")
                    .value_name("CONFIG_FILE")
                    .help("Sniper bot configuration file")
                    .required(true))
                .arg(Arg::new("capital")
                    .long("capital")
                    .value_name("AMOUNT")
                    .help("Trading capital allocation")
                    .default_value("100"))
                .arg(Arg::new("network")
                    .long("network")
                    .value_name("NET")
                    .help("Network: devnet or mainnet")
                    .required(true)
                    .value_parser(["devnet", "mainnet"]))
        )
        .subcommand(
            Command::new("start")
                .about("Start sniper bot")
                .arg(Arg::new("bot-id")
                    .long("bot-id")
                    .value_name("UUID")
                    .help("Bot instance ID")
                    .required(true))
        )
        .subcommand(
            Command::new("monitor")
                .about("Monitor sniper bot performance")
                .arg(Arg::new("bot-id")
                    .long("bot-id")
                    .value_name("UUID")
                    .help("Bot instance ID")
                    .required(true))
                .arg(Arg::new("live")
                    .long("live")
                    .action(clap::ArgAction::SetTrue)
                    .help("Live monitoring mode"))
                .arg(Arg::new("interval")
                    .long("interval")
                    .value_name("SECONDS")
                    .help("Update interval in seconds")
                    .default_value("1"))
        )
        .subcommand(
            Command::new("opportunities")
                .about("Show detected liquidity opportunities")
                .arg(Arg::new("bot-id")
                    .long("bot-id")
                    .value_name("UUID")
                    .help("Bot instance ID")
                    .required(true))
                .arg(Arg::new("last")
                    .long("last")
                    .value_name("COUNT")
                    .help("Show last N opportunities")
                    .default_value("10"))
        )
        .subcommand(
            Command::new("trades")
                .about("Show executed trades")
                .arg(Arg::new("bot-id")
                    .long("bot-id")
                    .value_name("UUID")
                    .help("Bot instance ID")
                    .required(true))
                .arg(Arg::new("today")
                    .long("today")
                    .action(clap::ArgAction::SetTrue)
                    .help("Show only today's trades"))
        )
        .subcommand(
            Command::new("stop")
                .about("Stop sniper bot")
                .arg(Arg::new("bot-id")
                    .long("bot-id")
                    .value_name("UUID")
                    .help("Bot instance ID")
                    .required(true))
        )
        .subcommand(
            Command::new("list")
                .about("List all sniper bot instances")
        )
}

pub async fn handle_sniper_command(
    manager: &DirectBotManager,
    matches: &ArgMatches,
) -> Result<()> {
    match matches.subcommand() {
        Some(("create", sub_matches)) => {
            handle_create_sniper(manager, sub_matches).await
        }
        Some(("start", sub_matches)) => {
            handle_start_sniper(manager, sub_matches).await
        }
        Some(("monitor", sub_matches)) => {
            handle_monitor_sniper(manager, sub_matches).await
        }
        Some(("opportunities", sub_matches)) => {
            handle_opportunities(manager, sub_matches).await
        }
        Some(("trades", sub_matches)) => {
            handle_trades(manager, sub_matches).await
        }
        Some(("stop", sub_matches)) => {
            handle_stop_sniper(manager, sub_matches).await
        }
        Some(("list", _)) => {
            handle_list_snipers(manager).await
        }
        _ => {
            println!("Use 'sniper --help' for available commands");
            Ok(())
        }
    }
}

async fn handle_create_sniper(
    manager: &DirectBotManager,
    matches: &ArgMatches,
) -> Result<()> {
    let config_file = matches.get_one::<String>("config").unwrap();
    let capital: f64 = matches.get_one::<String>("capital").unwrap().parse()?;
    let network = matches.get_one::<String>("network").unwrap();
    
    println!("🔧 Creating sniper bot...");
    println!("   Config: {}", config_file);
    println!("   Capital: ${:.2}", capital);
    println!("   Network: {}", network);
    
    let config = SniperBotConfig::load_from_file(config_file)?;
    let bot_id = manager.create_sniper_bot(config).await?;
    
    println!("✅ Sniper bot created successfully!");
    println!("   Bot ID: {}", bot_id);
    println!("   Use 'sniper start --bot-id {}' to start trading", bot_id);
    
    Ok(())
}

async fn handle_start_sniper(
    manager: &DirectBotManager,
    matches: &ArgMatches,
) -> Result<()> {
    let bot_id_str = matches.get_one::<String>("bot-id").unwrap();
    let bot_id = Uuid::parse_str(bot_id_str)?;
    
    println!("🚀 Starting sniper bot: {}", bot_id);
    
    // Load default config for now
    let config = BotConfig::default_sniper();
    
    manager.start_bot(bot_id, config).await?;
    
    println!("✅ Sniper bot started successfully!");
    println!("   Use 'sniper monitor --bot-id {} --live' to watch real-time", bot_id);
    
    Ok(())
}

async fn handle_monitor_sniper(
    manager: &DirectBotManager,
    matches: &ArgMatches,
) -> Result<()> {
    let bot_id_str = matches.get_one::<String>("bot-id").unwrap();
    let bot_id = Uuid::parse_str(bot_id_str)?;
    let live = matches.get_flag("live");
    let interval: u64 = matches.get_one::<String>("interval").unwrap().parse()?;
    
    if live {
        println!("📊 Starting live monitoring for bot: {}", bot_id);
        println!("   Press Ctrl+C to stop");
        
        loop {
            let status = manager.get_bot_status(bot_id).await?;
            let metrics = manager.get_bot_metrics(bot_id).await?;
            
            // Clear screen and show updated info
            print!("\x1B[2J\x1B[1;1H");
            println!("🎯 SNIPER BOT LIVE MONITOR");
            println!("══════════════════════════");
            println!("Bot ID: {}", bot_id);
            println!("Status: {:?}", status);
            println!("Uptime: {:.2} hours", metrics.uptime_hours);
            println!("Trades Today: {}", metrics.trades_count);
            println!("Total Profit: ${:.2}", metrics.total_profit);
            println!("Win Rate: {:.1}%", metrics.win_rate * 100.0);
            println!("Last Update: {}", chrono::Utc::now().format("%H:%M:%S"));
            
            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    } else {
        // Single snapshot
        let status = manager.get_bot_status(bot_id).await?;
        let metrics = manager.get_bot_metrics(bot_id).await?;
        
        println!("📊 Sniper Bot Status: {}", bot_id);
        println!("   Status: {:?}", status);
        println!("   Metrics: {:?}", metrics);
    }
    
    Ok(())
}

async fn handle_list_snipers(manager: &DirectBotManager) -> Result<()> {
    println!("📋 Active Sniper Bots:");
    
    let bots = manager.list_bots().await?;
    
    if bots.is_empty() {
        println!("   No active bots found.");
        println!("   Use 'sniper create' to create a new bot.");
        return Ok(());
    }
    
    for bot in bots {
        println!("   🎯 {} | {:?} | Profit: ${:.2}", 
                 bot.id, 
                 bot.status, 
                 bot.metrics.total_profit);
    }
    
    Ok(())
}
```

### **3. Integration with Existing CLI:**

```rust
// Modificar: old-root-archive/src/cli.rs
// Agregar importación
use crate::cli::direct_interface::DirectBotManager;
use crate::cli::sniper_commands::{build_sniper_commands, handle_sniper_command};

pub async fn run_cli() -> Result<()> {
    // Initialize direct bot manager
    let bot_manager = DirectBotManager::new()?;
    
    let matches = Command::new("SniperForge CLI")
        .version("0.2.0")
        .about("Direct CLI for SniperForge Multi-Bot Platform")
        // ... existing commands ...
        
        // Add sniper commands
        .subcommand(build_sniper_commands())
        
        .get_matches();

    match matches.subcommand() {
        // ... existing handlers ...
        
        Some(("sniper", sub_matches)) => {
            handle_sniper_command(&bot_manager, sub_matches).await?
        }
        
        // ... rest of handlers ...
    }

    Ok(())
}
```

---

## 📊 **PERFORMANCE COMPARISON**

### **HTTP API vs CLI Directa:**

| Métrica | HTTP API | CLI Directa | Mejora |
|---------|----------|-------------|--------|
| **Latencia Command** | 50-100ms | <1ms | 50-100x |
| **Throughput** | 20 req/sec | Unlimited | ∞ |
| **Memory Overhead** | 5-10MB | 1-2MB | 5x menos |
| **Network Dependency** | Sí | No | 100% reliable |
| **Real-time Streaming** | WebSocket | Direct | Nativo |
| **Error Handling** | HTTP codes | Rust Result | Type-safe |

### **Latency Critical Commands:**
```
start_bot():        HTTP: 80ms  → Direct: 0.5ms
get_status():       HTTP: 30ms  → Direct: 0.1ms  
get_metrics():      HTTP: 50ms  → Direct: 0.2ms
stop_bot():         HTTP: 100ms → Direct: 1ms
```

---

## ✅ **PLAN DE TESTING**

### **Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_sniper_bot() {
        let manager = DirectBotManager::new().unwrap();
        let config = SniperBotConfig::default();
        
        let bot_id = manager.create_sniper_bot(config).await.unwrap();
        assert_ne!(bot_id, Uuid::nil());
        
        let bots = manager.list_bots().await.unwrap();
        assert_eq!(bots.len(), 1);
        assert_eq!(bots[0].id, bot_id);
    }
    
    #[tokio::test]
    async fn test_bot_lifecycle() {
        let manager = DirectBotManager::new().unwrap();
        let config = SniperBotConfig::default();
        let bot_id = manager.create_sniper_bot(config).await.unwrap();
        
        // Test start
        let start_config = BotConfig::default_sniper();
        manager.start_bot(bot_id, start_config).await.unwrap();
        
        let status = manager.get_bot_status(bot_id).await.unwrap();
        assert_eq!(status, BotStatus::Running);
        
        // Test stop
        manager.stop_bot(bot_id).await.unwrap();
        
        let status = manager.get_bot_status(bot_id).await.unwrap();
        assert_eq!(status, BotStatus::Stopped);
    }
    
    #[tokio::test]
    async fn test_performance_latency() {
        let manager = DirectBotManager::new().unwrap();
        let config = SniperBotConfig::default();
        let bot_id = manager.create_sniper_bot(config).await.unwrap();
        
        let start = std::time::Instant::now();
        let _status = manager.get_bot_status(bot_id).await.unwrap();
        let duration = start.elapsed();
        
        // Assert latency < 1ms for direct calls
        assert!(duration.as_millis() < 1);
    }
}
```

### **Integration Tests:**
```rust
#[tokio::test]
async fn test_cli_sniper_commands_integration() {
    // Test full CLI command flow
    let args = vec![
        "sniperforge", "sniper", "create", 
        "--config", "test-config.json",
        "--capital", "100",
        "--network", "devnet"
    ];
    
    // Execute CLI command
    let result = execute_cli_with_args(args).await;
    assert!(result.is_ok());
    
    // Verify bot was created
    // ... rest of integration test
}
```

---

## 📋 **DELIVERABLES SEMANA 1-2**

### **Semana 1 (Días 1-7):**
- [ ] `src/cli/mod.rs` - Module structure
- [ ] `src/cli/direct_interface.rs` - Core DirectBotManager
- [ ] `src/cli/sniper_commands.rs` - Sniper command handlers
- [ ] Integration con CLI existente
- [ ] Unit tests básicos

### **Semana 2 (Días 8-14):**
- [ ] `src/cli/bot_factory.rs` - Direct bot instantiation
- [ ] `src/cli/state_manager.rs` - State persistence
- [ ] Performance optimizations
- [ ] Integration tests completos
- [ ] Documentation y examples

---

## 🎯 **SUCCESS CRITERIA**

### **Performance Targets:**
- ✅ Command latency: <1ms average
- ✅ Bot lifecycle: Create/Start/Stop < 2 seconds
- ✅ Memory usage: <50MB for CLI + 5 bots
- ✅ CPU usage: <5% background

### **Functional Targets:**
- ✅ All sniper commands working
- ✅ Real-time monitoring functional
- ✅ Bot state persistence
- ✅ Error handling robusto
- ✅ Test coverage >90%

---

**Estado:** ✅ Especificación completa CLI Directa  
**Implementación:** Semana 1-2 Agosto 2025  
**Performance Goal:** 50-100x mejora vs HTTP API  
**Next Step:** Comenzar implementación DirectBotManager  
