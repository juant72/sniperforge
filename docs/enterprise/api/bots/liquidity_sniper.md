# 🎯 Liquidity Sniper Bot API Documentation

## Overview

The **LiquiditySniperBot** is an enterprise-grade automated trading system designed to detect and execute trades on newly created liquidity pools across Solana DEXes with ultra-low latency and advanced risk management.

---

## 📋 Class Structure

### Core Class

```rust
pub struct LiquiditySniperBot {
    pub id: Uuid,
    pub config: SniperConfig,
    pub state: RwLock<SniperState>,
    pub is_running: Arc<AtomicBool>,
    pub pool_monitor: Arc<PoolMonitor>,
    pub analyzer: Arc<OpportunityAnalyzer>,
    pub executor: Arc<TradeExecutor>,
    pub risk_manager: Arc<Mutex<RiskManager>>,
    pub position_manager: Arc<PositionManager>,
    pub cost_analyzer: Arc<CostAnalyzer>,
    pub metrics: RwLock<SniperMetrics>,
    pub performance_tracker: Arc<RwLock<PerformanceTracker>>,
}
```

---

## 🔧 Constructor

### `new(id: Uuid, config: SniperConfig) -> Result<Self>`

Creates a new enterprise liquidity sniper bot instance.

**Input Parameters:**
- `id: Uuid` - Unique identifier for the bot instance
- `config: SniperConfig` - Configuration parameters for the bot

**Returns:**
- `Result<LiquiditySniperBot>` - Initialized bot instance or error

**Example:**
```rust
let bot_id = Uuid::new_v4();
let config = SniperConfig::default();
let bot = LiquiditySniperBot::new(bot_id, config).await?;
```

**Error Cases:**
- Invalid configuration parameters
- Failed component initialization
- Network connectivity issues

---

## 🚀 Primary Methods

### `start_hunting() -> Result<()>`

Starts the enterprise sniper hunting process with world-class execution.

**Input:** None
**Output:** `Result<()>`

**Functionality:**
- Initializes all monitoring tasks
- Starts opportunity detection loop
- Activates risk management systems
- Begins position monitoring

**Example:**
```rust
bot.start_hunting().await?;
```

**Internal Process:**
1. Update state to `SniperState::Monitoring`
2. Create opportunity channel (capacity: 1000)
3. Launch pool monitoring tasks for all DEXes
4. Start position monitoring task
5. Process opportunities in main loop

---

### `stop() -> Result<()>`

Gracefully stops the sniper bot and closes all positions.

**Input:** None
**Output:** `Result<()>`

**Functionality:**
- Sets state to `SniperState::Inactive`
- Closes all active positions
- Stops monitoring tasks
- Cleanup resources

**Example:**
```rust
bot.stop().await?;
```

---

## 📊 State Management Methods

### `get_state() -> SniperState`

Returns the current state of the sniper bot.

**Input:** None
**Output:** `SniperState`

**Possible States:**
```rust
pub enum SniperState {
    Inactive,                                    // Bot is stopped
    Monitoring,                                  // Scanning for opportunities
    AnalyzingOpportunity(OpportunityData),      // Evaluating detected opportunity
    ExecutingTrade(TradeData),                  // Executing trade
    ManagingPosition(PositionData),             // Managing open position
    Error(String),                              // Error state with message
}
```

**Example:**
```rust
let current_state = bot.get_state().await;
match current_state {
    SniperState::Monitoring => println!("Bot is hunting..."),
    SniperState::ExecutingTrade(trade) => println!("Executing trade: {}", trade.token_address),
    _ => println!("Bot state: {:?}", current_state),
}
```

---

### `get_metrics() -> SniperMetrics`

Returns comprehensive performance metrics.

**Input:** None
**Output:** `SniperMetrics`

**Metrics Structure:**
```rust
pub struct SniperMetrics {
    pub total_opportunities_detected: u64,       // Total opportunities found
    pub total_opportunities_executed: u64,       // Total trades executed
    pub execution_rate_percent: f64,             // Execution success rate
    pub total_profit_sol: f64,                   // Total profit in SOL
    pub total_loss_sol: f64,                     // Total losses in SOL
    pub net_profit_sol: f64,                     // Net profit (profit - loss)
    pub profit_factor: f64,                      // Profit/Loss ratio
    pub win_rate_percent: f64,                   // Percentage of winning trades
    pub average_execution_time_ms: f64,          // Average execution time
    pub sharpe_ratio: f64,                       // Risk-adjusted returns
    pub active_positions: u32,                   // Current open positions
    pub total_gas_spent_sol: f64,               // Total gas costs
    pub uptime_percent: f64,                     // System uptime percentage
    pub error_rate_percent: f64,                // Error rate percentage
    pub pools_monitored_today: u64,             // Pools monitored today
    pub unique_tokens_traded: u64,              // Unique tokens traded
    pub market_conditions: MarketCondition,      // Current market assessment
}
```

**Example:**
```rust
let metrics = bot.get_metrics().await;
println!("Total Profit: {} SOL", metrics.net_profit_sol);
println!("Win Rate: {:.1}%", metrics.win_rate_percent);
println!("Execution Time: {:.1}ms", metrics.average_execution_time_ms);
```

---

### `get_performance_analytics() -> PerformanceTracker`

Returns detailed performance analytics and tracking data.

**Input:** None
**Output:** `PerformanceTracker`

**Analytics Structure:**
```rust
pub struct PerformanceTracker {
    pub daily_stats: HashMap<String, DailyStats>,
    pub all_time_high_profit: f64,
    pub max_drawdown_percent: f64,
    pub current_streak: i32,
    pub best_streak: i32,
    pub worst_streak: i32,
    pub average_daily_profit: f64,
    pub volatility_index: f64,
    pub risk_score: f64,
}
```

**Example:**
```rust
let analytics = bot.get_performance_analytics().await;
println!("Max Drawdown: {:.2}%", analytics.max_drawdown_percent);
println!("Current Streak: {}", analytics.current_streak);
```

---

## 🔄 Internal Processing Methods

### `process_opportunity(opportunity: OpportunityData) -> Result<()>`

Processes a detected trading opportunity with enterprise safeguards.

**Input:**
- `opportunity: OpportunityData` - Detected opportunity data

**Output:** `Result<()>`

**Process Flow:**
1. Update metrics (total opportunities detected)
2. Set state to `AnalyzingOpportunity`
3. Perform enterprise risk assessment
4. Check opportunity score threshold (>= 0.75)
5. Calculate optimal position size
6. Execute trade with enterprise guarantees
7. Record execution metrics
8. Start position management if successful

**Risk Assessment Criteria:**
- Liquidity validation
- Smart contract verification
- MEV protection analysis
- Slippage impact calculation
- Capital allocation limits

---

### `execute_sniper_trade(opportunity: &OpportunityData, position_size: f64) -> Result<TradeResult>`

Executes a sniper trade with MEV protection and enterprise guarantees.

**Input:**
- `opportunity: &OpportunityData` - Target opportunity
- `position_size: f64` - Position size in SOL

**Output:** `Result<TradeResult>`

**Execution Features:**
- MEV protection mechanisms
- Priority fee optimization
- Slippage protection
- Transaction retry logic
- Real-time price validation

**TradeResult Structure:**
```rust
pub struct TradeResult {
    pub success: bool,
    pub transaction_signature: Option<String>,
    pub execution_time_ms: u64,
    pub actual_price: f64,
    pub slippage_percent: f64,
    pub gas_cost_sol: f64,
    pub position: Option<PositionData>,
    pub error: Option<String>,
}
```

---

### `calculate_position_size(opportunity: &OpportunityData) -> Result<f64>`

Calculates optimal position size based on risk parameters.

**Input:**
- `opportunity: &OpportunityData` - Opportunity to evaluate

**Output:** `Result<f64>` - Position size in SOL

**Calculation Factors:**
- Available capital allocation
- Maximum position size percentage
- Risk score of opportunity
- Current portfolio exposure
- Market volatility assessment

**Formula:**
```
position_size = min(
    capital_allocation * max_position_percent,
    available_capital * (1.0 - risk_score),
    liquidity_limit
)
```

---

## ⚙️ Configuration

### `SniperConfig` Structure

```rust
pub struct SniperConfig {
    pub capital_allocation: f64,                 // 10.0 SOL (default)
    pub max_position_size_percent: f64,          // 20.0% (default)
    pub min_liquidity_usd: f64,                  // $10,000 (default)
    pub max_risk_score: f64,                     // 0.7 (default)
    pub target_profit_percent: f64,              // 15.0% (default)
    pub stop_loss_percent: f64,                  // 5.0% (default)
    pub max_slippage_bps: u16,                   // 50 bps (0.5%, default)
    pub priority_fee_lamports: u64,              // 100,000 lamports (default)
    pub max_detection_latency_ms: u64,           // 500ms (default)
    pub max_execution_time_ms: u64,              // 200ms (default)
    pub monitored_dexes: Vec<DexType>,           // [Raydium, Orca, Jupiter]
    pub environment: Environment,                // Mainnet (default)
    pub mev_protection_enabled: bool,            // true (default)
    pub use_private_mempool: bool,               // true (default)
    pub advanced_analytics: bool,                // true (default)
    pub max_positions: u32,                      // 3 (default)
}
```

---

## 🎯 Performance Guarantees

### Speed Targets
- **Detection Latency:** < 500ms from pool creation
- **Analysis Time:** < 100ms per opportunity
- **Execution Speed:** < 200ms from decision to blockchain
- **Total Response Time:** < 800ms end-to-end

### Success Metrics
- **Execution Rate:** > 85% of analyzed opportunities
- **Win Rate:** > 70% profitable trades
- **Uptime:** > 99.5% operational availability
- **Error Rate:** < 2% of total operations

### Risk Management
- **Maximum Drawdown:** < 5% of allocated capital
- **Position Limits:** Enforced at all times
- **Stop Loss:** Automatic execution at configured levels
- **MEV Protection:** Built-in sandwich attack prevention

---

## 🔌 Integration Examples

### Basic Bot Setup
```rust
use sniperforge::bots::liquidity_sniper::{LiquiditySniperBot, SniperConfig};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Create configuration
    let config = SniperConfig {
        capital_allocation: 50.0,  // 50 SOL
        target_profit_percent: 20.0,  // 20% profit target
        ..SniperConfig::default()
    };
    
    // Initialize bot
    let bot_id = Uuid::new_v4();
    let bot = LiquiditySniperBot::new(bot_id, config).await?;
    
    // Start hunting
    bot.start_hunting().await?;
    
    Ok(())
}
```

### Monitoring Setup
```rust
// Monitor bot performance
tokio::spawn(async move {
    loop {
        let metrics = bot.get_metrics().await;
        let state = bot.get_state().await;
        
        println!("State: {:?}", state);
        println!("Profit: {} SOL", metrics.net_profit_sol);
        println!("Win Rate: {:.1}%", metrics.win_rate_percent);
        
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
});
```

---

## 🛡️ Error Handling

### Common Error Types
- `ConfigurationError` - Invalid configuration parameters
- `NetworkError` - RPC or WebSocket connectivity issues
- `ExecutionError` - Trade execution failures
- `RiskManagementError` - Risk limit violations
- `InsufficientFundsError` - Inadequate capital for trade

### Error Recovery
- Automatic retry mechanisms for transient failures
- Graceful degradation during network issues
- State preservation across restarts
- Comprehensive error logging and reporting

---

## 📈 Monitoring & Alerts

### Key Metrics to Monitor
- Net profit/loss trends
- Execution success rates
- Average response times
- Error frequencies
- Capital utilization
- Risk exposure levels

### Alert Triggers
- Drawdown exceeding thresholds
- Extended execution failures
- Network connectivity loss
- Risk limit violations
- Performance degradation

---

*Last Updated: August 8, 2025*
*API Version: 3.0*
