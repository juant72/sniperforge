# ⚡ Arbitrage Engine API Documentation

## Overview

The **ArbitrageEngine** is a sophisticated trading system that identifies and executes arbitrage opportunities across multiple Solana DEXes with enterprise-grade performance monitoring and risk management.

---

## 📋 Class Structure

### Core Class

```rust
pub struct ArbitrageEngine {
    config: SimpleConfig,
    rpc_client: Arc<RpcClient>,
    price_feed_manager: Arc<PriceFeedManager>,
    risk_manager: RiskManager,
    wallet: Arc<Keypair>,
    active_pairs: Arc<RwLock<HashMap<String, ArbitragePair>>>,
    last_scan_time: Arc<RwLock<Instant>>,
    is_initialized: Arc<RwLock<bool>>,
    
    // Enterprise Performance Features
    enhanced_trading_stats: Arc<RwLock<EnhancedTradingStats>>,
    trade_history: Arc<RwLock<VecDeque<TradeResult>>>,
    perf_config: Arc<RwLock<PerformanceConfig>>,
    perf_metrics: Arc<RwLock<PerformanceMetrics>>,
    api_status: Arc<RwLock<HashMap<String, bool>>>,
    market_data_cache: Arc<RwLock<HashMap<String, f64>>>,
    hourly_profits: Arc<RwLock<VecDeque<(DateTime<Utc>, f64)>>>,
    current_balance: Arc<RwLock<f64>>,
}
```

---

## 🔧 Constructor

### `new(config: SimpleConfig, price_feed_manager: Arc<PriceFeedManager>) -> Result<Self>`

Creates a new enterprise arbitrage engine with enhanced working bot functionality.

**Input Parameters:**
- `config: SimpleConfig` - System configuration parameters
- `price_feed_manager: Arc<PriceFeedManager>` - Price feed management system

**Returns:**
- `Result<ArbitrageEngine>` - Initialized engine instance or error

**Example:**
```rust
let config = SimpleConfig::default();
let price_feed_manager = Arc::new(PriceFeedManager::new(&config));
let engine = ArbitrageEngine::new(config, price_feed_manager).await?;
```

**Initialization Process:**
1. Creates RPC client with confirmed commitment
2. Loads wallet from configuration path
3. Initializes risk management system
4. Gets initial wallet balance
5. Sets up performance monitoring
6. Initializes API status tracking
7. Loads default trading pairs

**Error Cases:**
- Wallet file not found or invalid
- RPC connection failure
- Insufficient balance for operations

---

## 🔍 Core Trading Methods

### `scan_for_opportunities() -> Result<Vec<ArbitrageOpportunity>>`

Scans across all configured DEXes for arbitrage opportunities.

**Input:** None
**Output:** `Result<Vec<ArbitrageOpportunity>>`

**Process:**
1. Updates last scan timestamp
2. Retrieves current market data
3. Analyzes each active trading pair
4. Sorts opportunities by profitability
5. Returns filtered results

**Example:**
```rust
let opportunities = engine.scan_for_opportunities().await?;
for opportunity in opportunities {
    println!("Found: {:.2}% profit on {}", 
             opportunity.profit_percentage * 100.0,
             opportunity.pair.base_token.symbol);
}
```

**ArbitrageOpportunity Structure:**
```rust
pub struct ArbitrageOpportunity {
    pub pair: ArbitragePair,
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit_percentage: f64,
    pub volume_required: f64,
    pub estimated_gas_cost: f64,
    pub confidence_score: f64,
    pub timestamp: DateTime<Utc>,
    pub execution_time_window: Duration,
}
```

---

### `validate_execution(opportunity: &ArbitrageOpportunity) -> Result<()>`

Validates execution parameters for an arbitrage opportunity.

**Input:**
- `opportunity: &ArbitrageOpportunity` - Opportunity to validate

**Output:** `Result<()>`

**Validation Steps:**
1. Validates opportunity parameters
2. Calculates optimal execution amounts
3. Verifies liquidity requirements
4. Simulates buy/sell orders
5. Calculates final profit estimation

**Example:**
```rust
for opportunity in opportunities {
    match engine.validate_execution(&opportunity).await {
        Ok(_) => println!("✅ Opportunity validated"),
        Err(e) => println!("❌ Validation failed: {}", e),
    }
}
```

---

### `assess_risk(opportunity: &ArbitrageOpportunity) -> Result<RiskAssessment>`

Performs comprehensive risk assessment for an arbitrage opportunity.

**Input:**
- `opportunity: &ArbitrageOpportunity` - Opportunity to assess

**Output:** `Result<RiskAssessment>`

**Risk Factors Analyzed:**
- Liquidity depth analysis
- Slippage impact calculation
- Smart contract risks
- Market volatility assessment
- Execution timing risks

**RiskAssessment Structure:**
```rust
pub struct RiskAssessment {
    pub overall_risk_score: f64,        // 0.0 to 1.0
    pub liquidity_risk: f64,
    pub execution_risk: f64,
    pub market_risk: f64,
    pub recommended_position_size: f64,
    pub risk_factors: Vec<String>,
    pub approved_for_execution: bool,
}
```

---

## 📊 Performance & Analytics Methods

### `get_enhanced_stats() -> EnhancedTradingStats`

Returns comprehensive trading statistics with enterprise metrics.

**Input:** None
**Output:** `EnhancedTradingStats`

**Statistics Structure:**
```rust
pub struct EnhancedTradingStats {
    pub total_trades: u64,
    pub successful_trades: u64,
    pub total_profit_sol: f64,
    pub total_loss_sol: f64,
    pub net_profit_sol: f64,
    pub win_rate_percentage: f64,
    pub average_profit_per_trade: f64,
    pub largest_profit: f64,
    pub largest_loss: f64,
    pub total_gas_costs: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub profit_factor: f64,
    pub current_streak: i32,
    pub best_winning_streak: i32,
    pub worst_losing_streak: i32,
}
```

**Example:**
```rust
let stats = engine.get_enhanced_stats().await;
println!("Total Profit: {} SOL", stats.net_profit_sol);
println!("Win Rate: {:.1}%", stats.win_rate_percentage);
println!("Sharpe Ratio: {:.2}", stats.sharpe_ratio);
```

---

### `get_performance_metrics() -> PerformanceMetrics`

Returns real-time performance metrics and ML optimization data.

**Input:** None
**Output:** `PerformanceMetrics`

**Metrics Structure:**
```rust
pub struct PerformanceMetrics {
    pub total_cycles: u64,
    pub successful_discoveries: u64,
    pub discovery_time_ms: u64,
    pub opportunities_per_second: f64,
    pub adaptive_adjustments: u64,
    pub ml_confidence_score: f64,
    pub api_response_times: HashMap<String, u64>,
    pub last_optimization: DateTime<Utc>,
}
```

---

### `display_ml_enhanced_dashboard(is_real_trading: bool)`

Displays comprehensive enterprise dashboard with real-time metrics.

**Input:**
- `is_real_trading: bool` - Whether real trading is enabled

**Output:** None (console display)

**Dashboard Features:**
- Real-time profit/loss tracking
- Performance metrics display
- API status monitoring
- ML optimization indicators
- Risk management status

**Example:**
```rust
// Display dashboard every 30 seconds
tokio::spawn(async move {
    loop {
        engine.display_ml_enhanced_dashboard(false).await;
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
});
```

---

## 🔄 Optimization Methods

### `optimize_discovery_performance(discovery_time_ms: u64, opportunities_found: usize)`

Optimizes discovery performance using ML-based adaptive algorithms.

**Input:**
- `discovery_time_ms: u64` - Time taken for last discovery cycle
- `opportunities_found: usize` - Number of opportunities found

**Output:** None

**Optimization Features:**
- Adaptive concurrency adjustment
- Latency-based parameter tuning
- Performance degradation detection
- Automatic system optimization

**Example:**
```rust
let start = Instant::now();
let opportunities = engine.scan_for_opportunities().await?;
let discovery_time = start.elapsed().as_millis() as u64;

engine.optimize_discovery_performance(discovery_time, opportunities.len()).await;
```

---

## 💰 Wallet & Balance Methods

### `get_wallet_balance() -> Result<f64>`

Retrieves current wallet balance in SOL.

**Input:** None
**Output:** `Result<f64>` - Balance in SOL

**Example:**
```rust
let balance = engine.get_wallet_balance().await?;
println!("Current balance: {:.6} SOL", balance);
```

---

### `update_balance() -> Result<f64>`

Updates and returns current wallet balance with caching.

**Input:** None
**Output:** `Result<f64>` - Updated balance in SOL

**Example:**
```rust
let updated_balance = engine.update_balance().await?;
```

---

### `get_wallet_pubkey() -> Pubkey`

Returns the public key of the configured wallet.

**Input:** None
**Output:** `Pubkey`

---

## 📈 API Status & Monitoring

### `get_api_status() -> HashMap<String, bool>`

Returns current status of all connected APIs and services.

**Input:** None
**Output:** `HashMap<String, bool>`

**Monitored APIs:**
- DexScreener
- Jupiter
- Raydium
- Orca
- RPC endpoints

**Example:**
```rust
let api_status = engine.get_api_status().await;
for (api, status) in api_status {
    let icon = if status { "✅" } else { "❌" };
    println!("{} {}", icon, api);
}
```

---

### `get_statistics() -> EngineStatistics`

Returns general engine statistics and health metrics.

**Input:** None
**Output:** `EngineStatistics`

**Statistics Structure:**
```rust
pub struct EngineStatistics {
    pub pairs_monitored: usize,
    pub last_scan_time: Instant,
    pub uptime: Duration,
    pub is_active: bool,
}
```

---

## 🏗️ Trading Pair Management

### `initialize_trading_pairs() -> Result<()>`

Initializes default trading pairs for monitoring.

**Input:** None
**Output:** `Result<()>`

**Default Pairs:**
- SOL/USDC
- Additional pairs as configured

**ArbitragePair Structure:**
```rust
pub struct ArbitragePair {
    pub base_token: Token,
    pub quote_token: Token,
    pub pool_address: Option<String>,
    pub fee_rate: f64,
}

pub struct Token {
    pub symbol: String,
    pub mint: String,
    pub decimals: u8,
}
```

---

## ⚙️ Configuration

### `SimpleConfig` Integration

The ArbitrageEngine integrates with the system's `SimpleConfig`:

```rust
pub struct SimpleConfig {
    pub solana_rpc_url: String,
    pub private_key_path: String,
    pub trading_amount: f64,
    pub profit_threshold: f64,
    pub max_price_age_seconds: u64,
    pub risk_percentage: f64,
    pub enable_simulation: bool,
    pub enable_ml_analysis: bool,
    // ... additional configuration options
}
```

---

## 🎯 Performance Guarantees

### Speed Targets
- **Opportunity Scanning:** < 1 second across all DEXes
- **Risk Assessment:** < 100ms per opportunity
- **Validation:** < 500ms per opportunity
- **Balance Updates:** < 200ms

### Reliability Targets
- **Uptime:** > 99.9%
- **API Availability:** > 99.5%
- **Data Accuracy:** > 99.8%
- **Error Recovery:** < 10 seconds

### Scalability
- **Concurrent Pairs:** Up to 100 pairs
- **Opportunities/Second:** 50+ opportunities
- **API Requests:** 1000+ requests/minute
- **Memory Usage:** < 512MB

---

## 🔌 Integration Examples

### Basic Engine Setup
```rust
use sniperforge::trading::arbitrage::ArbitrageEngine;
use sniperforge::config::SimpleConfig;
use sniperforge::apis::price_feeds::PriceFeedManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = SimpleConfig::from_env()?;
    
    // Create price feed manager
    let price_feeds = Arc::new(PriceFeedManager::new(&config));
    
    // Initialize engine
    let engine = ArbitrageEngine::new(config, price_feeds).await?;
    
    // Start monitoring
    loop {
        let opportunities = engine.scan_for_opportunities().await?;
        
        for opportunity in opportunities {
            if engine.validate_execution(&opportunity).await.is_ok() {
                println!("Valid opportunity: {:.2}% profit", 
                         opportunity.profit_percentage * 100.0);
            }
        }
        
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
```

### Performance Monitoring Setup
```rust
// Monitor performance metrics
tokio::spawn(async move {
    loop {
        let stats = engine.get_enhanced_stats().await;
        let metrics = engine.get_performance_metrics().await;
        
        // Log key metrics
        tracing::info!("Trades: {} | Profit: {} SOL | Win Rate: {:.1}%",
                      stats.total_trades,
                      stats.net_profit_sol,
                      stats.win_rate_percentage);
        
        // Display dashboard
        engine.display_ml_enhanced_dashboard(true).await;
        
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
});
```

---

## 🛡️ Error Handling

### Common Error Types
- `ConnectionError` - RPC or API connectivity issues
- `WalletError` - Wallet loading or balance issues
- `ValidationError` - Opportunity validation failures
- `ConfigurationError` - Invalid configuration parameters

### Error Recovery Strategies
- Automatic RPC endpoint switching
- API fallback mechanisms
- Graceful degradation during outages
- Comprehensive error logging

---

## 📊 Monitoring & Alerts

### Key Performance Indicators
- Discovery latency trends
- Opportunity success rates
- API response times
- Profit/loss ratios
- System resource usage

### Alert Conditions
- Discovery latency > 2 seconds
- API failure rates > 5%
- Unexpected profit/loss patterns
- System resource exhaustion
- Network connectivity issues

---

*Last Updated: August 8, 2025*
*API Version: 3.0*
