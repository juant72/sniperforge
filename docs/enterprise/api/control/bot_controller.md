# BotController API Documentation

## Overview

The **BotController** is the central orchestration system for managing bot lifecycles, configurations, and state persistence in SniperForge Enterprise. It provides comprehensive bot management capabilities with enterprise-grade reliability and monitoring.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    BotController                            │
├─────────────────────────────────────────────────────────────┤
│  Core Components:                                           │
│  • Bot Registry (HashMap<Uuid, BotInstance>)               │
│  • ConfigManager (Validation & Hot-reload)                 │
│  • StatePersistenceManager (YAML Persistence)              │
│  • MetricsCollector (Performance Analytics)                │
│  • System Resource Monitor                                 │
└─────────────────────────────────────────────────────────────┘
```

## Constructor

### `new() -> Result<Self>`

Creates a new BotController with enterprise-grade initialization.

**Returns:**
- `Result<BotController>` - Configured controller instance

**Features:**
- Automatic state recovery from persistence
- ConfigManager initialization for validation
- MetricsCollector setup with custom windows
- System resource monitoring setup

**Example:**
```rust
let controller = BotController::new().await?;
```

**Error Handling:**
- Returns `Err` if persistence initialization fails
- Returns `Err` if system resources are insufficient

---

## Bot Lifecycle Management

### `create_bot(bot_type: BotType, config: BotConfig) -> Result<Uuid>`

Creates a new bot instance with enterprise validation and persistence.

**Parameters:**
- `bot_type: BotType` - The type of bot to create
- `config: BotConfig` - Initial configuration for the bot

**Returns:**
- `Result<Uuid>` - Unique identifier for the created bot

**Process Flow:**
1. Configuration validation through ConfigManager
2. Bot instance creation and registration
3. State persistence to YAML
4. Metrics recording for enterprise analytics

**Example:**
```rust
let config = BotConfig {
    bot_type: BotType::LiquiditySniper,
    trading_pairs: vec!["BTC/USDT".to_string()],
    max_position_size: 1000.0,
    // ... other configuration
};

let bot_id = controller.create_bot(BotType::LiquiditySniper, config).await?;
```

**Error Handling:**
- `ConfigValidationError` - Configuration validation failed
- `PersistenceError` - Failed to save bot state
- `ResourceLimitError` - System resource limits reached

### `start_bot(bot_id: Uuid, config: BotConfig) -> Result<()>`

Starts a specific bot with enhanced lifecycle management.

**Parameters:**
- `bot_id: Uuid` - Unique identifier of the bot to start
- `config: BotConfig` - Configuration to use for startup

**Returns:**
- `Result<()>` - Success or error

**Process Flow:**
1. Pre-start configuration validation
2. Bot interface initialization
3. Status update to Running
4. Persistence state synchronization
5. Metrics collection activation

**Example:**
```rust
controller.start_bot(bot_id, updated_config).await?;
```

**Performance Guarantees:**
- Startup time: < 500ms for standard bots
- Configuration validation: < 100ms
- State persistence: < 50ms

### `stop_bot(bot_id: Uuid) -> Result<()>`

Stops a specific bot with graceful shutdown procedures.

**Parameters:**
- `bot_id: Uuid` - Unique identifier of the bot to stop

**Returns:**
- `Result<()>` - Success or error

**Process Flow:**
1. Graceful bot interface shutdown
2. Final metrics collection
3. Status update to Stopped
4. State persistence
5. Resource cleanup

**Example:**
```rust
controller.stop_bot(bot_id).await?;
```

**Shutdown Guarantees:**
- Graceful shutdown timeout: 10 seconds
- Force shutdown if timeout exceeded
- All trades properly closed before shutdown

---

## Bot Information & Monitoring

### `get_bot_status(bot_id: Uuid) -> Result<BotStatus>`

Retrieves the current status of a specific bot.

**Parameters:**
- `bot_id: Uuid` - Unique identifier of the bot

**Returns:**
- `Result<BotStatus>` - Current bot status

**Status Types:**
- `Running` - Bot is actively trading
- `Stopped` - Bot is not running
- `Paused` - Bot is paused (temporary)
- `Error` - Bot encountered an error

**Example:**
```rust
let status = controller.get_bot_status(bot_id).await?;
match status {
    BotStatus::Running => println!("Bot is active"),
    BotStatus::Stopped => println!("Bot is stopped"),
    // ... handle other statuses
}
```

### `get_bot_metrics(bot_id: Uuid) -> Result<BotMetrics>`

Retrieves comprehensive metrics for a specific bot.

**Parameters:**
- `bot_id: Uuid` - Unique identifier of the bot

**Returns:**
- `Result<BotMetrics>` - Complete bot performance metrics

**Metrics Include:**
- Trading performance (PnL, trades executed, win rate)
- Technical indicators (RSI, MACD, etc.)
- Resource usage (CPU, memory)
- Error counts and types
- Execution timing statistics

**Example:**
```rust
let metrics = controller.get_bot_metrics(bot_id).await?;
println!("Total PnL: ${:.2}", metrics.trading.total_pnl_usd);
println!("Win Rate: {:.2}%", metrics.trading.win_rate);
```

### `list_bots() -> Result<Vec<BotSummary>>`

Lists all registered bots with summary information.

**Returns:**
- `Result<Vec<BotSummary>>` - List of bot summaries

**BotSummary Structure:**
```rust
pub struct BotSummary {
    pub id: Uuid,
    pub bot_type: BotType,
    pub status: BotStatus,
    pub metrics: BotMetrics,
    pub is_default: bool,
}
```

**Example:**
```rust
let bots = controller.list_bots().await?;
for bot in bots {
    println!("Bot {}: {:?} - {:?}", bot.id, bot.bot_type, bot.status);
}
```

---

## System-Wide Operations

### `get_system_metrics() -> Result<SystemMetrics>`

Retrieves comprehensive system-wide metrics with enhanced data collection.

**Returns:**
- `Result<SystemMetrics>` - System performance metrics

**SystemMetrics Structure:**
```rust
pub struct SystemMetrics {
    pub total_bots: usize,
    pub running_bots: usize,
    pub total_profit: f64,
    pub total_trades: u64,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
}
```

**Example:**
```rust
let metrics = controller.get_system_metrics().await?;
println!("System: {} bots ({} running)", metrics.total_bots, metrics.running_bots);
println!("Total Profit: ${:.2}", metrics.total_profit);
```

### `hot_reload_configs() -> Result<()>`

Hot-reloads all configurations from disk without system restart.

**Returns:**
- `Result<()>` - Success or error

**Capabilities:**
- Reloads system configuration templates
- Updates active bot configurations
- Validates new configurations before applying
- Non-disruptive to running bots

**Example:**
```rust
controller.hot_reload_configs().await?;
```

**Performance:**
- Configuration reload: < 200ms
- Zero downtime for running bots
- Automatic rollback on validation failure

---

## Mass Control Operations

### `start_all_bots() -> Result<MassControlResult>`

Starts all registered bots that are currently stopped.

**Returns:**
- `Result<MassControlResult>` - Results of mass operation

**MassControlResult Structure:**
```rust
pub struct MassControlResult {
    pub successful: Vec<Uuid>,
    pub failed: Vec<(Uuid, String)>,
    pub total_attempted: usize,
}
```

**Example:**
```rust
let result = controller.start_all_bots().await?;
println!("Started {}/{} bots successfully", 
         result.successful.len(), result.total_attempted);
```

### `stop_all_bots() -> Result<MassControlResult>`

Stops all currently running bots with graceful shutdown.

**Returns:**
- `Result<MassControlResult>` - Results of mass operation

**Example:**
```rust
let result = controller.stop_all_bots().await?;
for (bot_id, error) in result.failed {
    eprintln!("Failed to stop bot {}: {}", bot_id, error);
}
```

---

## Persistence & State Management

### `get_system_state_summary() -> Result<SystemStateSummary>`

Retrieves comprehensive system state information for enterprise monitoring.

**Returns:**
- `Result<SystemStateSummary>` - Current system state

**SystemStateSummary Structure:**
```rust
pub struct SystemStateSummary {
    pub total_registered_bots: usize,
    pub server_restart_count: u64,
    pub system_start_time: DateTime<Utc>,
    pub persisted_bots: Vec<Uuid>,
}
```

### `get_historical_metrics(hours: u32) -> Result<Vec<PersistedSystemMetrics>>`

Retrieves historical system metrics from persistence.

**Parameters:**
- `hours: u32` - Number of hours of history to retrieve

**Returns:**
- `Result<Vec<PersistedSystemMetrics>>` - Time-series metrics data

**Example:**
```rust
let history = controller.get_historical_metrics(24).await?;
for metric in history {
    println!("{}: {} bots, ${:.2} profit", 
             metric.timestamp, metric.total_bots, metric.total_profit);
}
```

### `create_system_backup() -> Result<String>`

Creates a complete backup of current system state.

**Returns:**
- `Result<String>` - Path to created backup file

**Backup Contents:**
- All bot configurations
- Current bot states
- Historical metrics
- System configuration
- Trading history

**Example:**
```rust
let backup_path = controller.create_system_backup().await?;
println!("Backup created: {}", backup_path);
```

### `force_save_all_state() -> Result<()>`

Forces immediate save of all system state to persistence.

**Returns:**
- `Result<()>` - Success or error

**Use Cases:**
- Before system maintenance
- After critical configuration changes
- Emergency state preservation

---

## Resource Management

### `get_system_resource_status() -> Result<SystemResourceStatus>`

Retrieves comprehensive system resource usage and limits.

**Returns:**
- `Result<SystemResourceStatus>` - Resource status information

**SystemResourceStatus Structure:**
```rust
pub struct SystemResourceStatus {
    pub total_bots: usize,
    pub running_bots: usize,
    pub memory_usage_mb: f64,
    pub estimated_max_bots: usize,
    pub resource_warning: Option<String>,
    pub cpu_cores: usize,
}
```

**Example:**
```rust
let resources = controller.get_system_resource_status().await?;
if let Some(warning) = resources.resource_warning {
    println!("⚠️ Resource Warning: {}", warning);
}
```

---

## Error Handling

### Common Error Types

- **ConfigurationError**: Invalid bot configuration
- **BotNotFoundError**: Specified bot does not exist
- **ResourceLimitError**: System resource limits exceeded
- **PersistenceError**: Failed to save/load state
- **NetworkError**: Trading network connectivity issues

### Error Recovery

The BotController implements comprehensive error recovery:

1. **Configuration Errors**: Automatic rollback to last known good config
2. **Bot Failures**: Automatic restart with exponential backoff
3. **Persistence Failures**: In-memory backup with periodic retry
4. **Resource Exhaustion**: Graceful degradation and alerting

---

## Performance Characteristics

### Latency Guarantees

- Bot creation: < 200ms
- Bot start/stop: < 500ms
- Status queries: < 10ms
- Metrics retrieval: < 50ms
- Configuration reload: < 200ms

### Throughput

- Concurrent bot operations: 100+ bots
- Metrics collection frequency: 1Hz
- Configuration updates: 10/second
- State persistence: Every 30 seconds

### Resource Usage

- Base memory overhead: ~15MB
- Per-bot overhead: ~2.5MB
- Running bot overhead: +5MB
- CPU usage: < 5% for 50 bots

---

## Integration Examples

### Complete Bot Lifecycle Example

```rust
// Initialize controller
let controller = BotController::new().await?;

// Create and configure bot
let config = BotConfig::default_for_type(BotType::LiquiditySniper);
let bot_id = controller.create_bot(BotType::LiquiditySniper, config.clone()).await?;

// Start bot
controller.start_bot(bot_id, config).await?;

// Monitor bot
loop {
    let status = controller.get_bot_status(bot_id).await?;
    let metrics = controller.get_bot_metrics(bot_id).await?;
    
    println!("Status: {:?}, PnL: ${:.2}", status, metrics.trading.total_pnl_usd);
    
    if matches!(status, BotStatus::Error) {
        break;
    }
    
    tokio::time::sleep(Duration::from_secs(5)).await;
}

// Stop bot
controller.stop_bot(bot_id).await?;
```

### Enterprise Monitoring Setup

```rust
// Set up comprehensive monitoring
let controller = BotController::new().await?;

// Get system overview
let system_metrics = controller.get_system_metrics().await?;
let resource_status = controller.get_system_resource_status().await?;
let state_summary = controller.get_system_state_summary().await?;

// Log enterprise metrics
println!("=== Enterprise System Status ===");
println!("Uptime: {}s", system_metrics.uptime_seconds);
println!("Bots: {}/{} running", system_metrics.running_bots, system_metrics.total_bots);
println!("Memory: {:.1}MB / {} cores", resource_status.memory_usage_mb, resource_status.cpu_cores);
println!("Restarts: {}", state_summary.server_restart_count);

// Historical analysis
let history = controller.get_historical_metrics(24).await?;
let avg_profit = history.iter().map(|h| h.total_profit).sum::<f64>() / history.len() as f64;
println!("24h Avg Profit: ${:.2}", avg_profit);
```
