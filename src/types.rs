use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Bot error: {0}")]
    Bot(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Wallet error: {0}")]
    Wallet(String),
    #[error("Trading error: {0}")]
    Trading(String),
    #[error("Bot management error: {0}")]
    BotManagement(String),
    #[error("Event bus error: {0}")]
    EventBus(String),
    #[error("Resource management error: {0}")]
    ResourceManagement(String),
    #[error("Wallet management error: {0}")]
    WalletManagement(String),
    #[error("Monitoring error: {0}")]
    Monitoring(String),
    #[error("Data feed error: {0}")]
    DataFeed(String),
    #[error("Resource error: {0}")]
    Resource(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

// ============================================================================
// Core Platform Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BotId(pub Uuid);

impl BotId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for BotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BotType {
    LpSniper,
    CopyTrading,
    Arbitrage,
    Mev,
    GridTrading,
    DollarCostAverage,
    Custom(String),
}

impl std::fmt::Display for BotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotType::LpSniper => write!(f, "LP Sniper"),
            BotType::CopyTrading => write!(f, "Copy Trading"),
            BotType::Arbitrage => write!(f, "Arbitrage"),
            BotType::Mev => write!(f, "MEV"),
            BotType::GridTrading => write!(f, "Grid Trading"),
            BotType::DollarCostAverage => write!(f, "DCA"),
            BotType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BotStatus {
    Stopped,
    Starting,
    Running,
    Paused,
    Error(String),
    Stopping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotMetrics {
    pub bot_id: BotId,
    pub bot_type: BotType,
    pub uptime_seconds: u64,
    pub operations_count: u64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub last_activity: DateTime<Utc>,
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for BotMetrics {
    fn default() -> Self {
        Self {
            bot_id: BotId::new(),
            bot_type: BotType::Custom("unknown".to_string()),
            uptime_seconds: 0,
            operations_count: 0,
            success_rate: 0.0,
            avg_latency_ms: 0.0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            last_activity: chrono::Utc::now(),
            custom_metrics: HashMap::new(),
        }
    }
}

// ============================================================================
// Bot Management Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BotConfig {
    LpSniper(LpSniperConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LpSniperConfig {
    pub enabled: bool,
    pub trade_amount_sol: f64,
    pub max_slippage_percent: f64,
    pub min_liquidity_usd: f64,
    pub max_pool_age_seconds: u64,
    pub settings: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotInstance {
    pub id: Uuid,
    pub bot_type: BotType,
    pub name: String,
    pub status: BotStatus,
    pub config: BotConfig,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub metrics: BotMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BotCommand {
    Start,
    Stop { bot_id: Uuid },
    Pause,
    Resume,
    UpdateConfig(BotConfig),
    GetStatus,
    GetMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub token: Pubkey,
    pub price_usd: f64,
    pub price_sol: Option<f64>,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub market_cap: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Trading Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub mint: Pubkey,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub supply: Option<u64>,
    pub is_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub pool_id: Pubkey,
    pub dex: DexType,
    pub token_a: TokenInfo,
    pub token_b: TokenInfo,
    pub liquidity_usd: f64,
    pub volume_24h_usd: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub detected_at: DateTime<Utc>,
    pub is_new: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DexType {
    Raydium,
    Orca,
    Jupiter,
    Serum,
    Meteora,
    Other(String),
}

impl std::fmt::Display for DexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DexType::Raydium => write!(f, "Raydium"),
            DexType::Orca => write!(f, "Orca"),
            DexType::Jupiter => write!(f, "Jupiter"),
            DexType::Serum => write!(f, "Serum"),
            DexType::Meteora => write!(f, "Meteora"),
            DexType::Other(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingOpportunity {
    pub id: Uuid,
    pub opportunity_type: OpportunityType,
    pub pool_info: PoolInfo,
    pub confidence_score: f64,
    pub estimated_profit_usd: f64,
    pub risk_level: RiskLevel,
    pub expires_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpportunityType {
    NewPool,
    PriceDivergence,
    MevOpportunity,
    WalletActivity,
    GridRebalance,
    DcaTrigger,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingAction {
    pub action_id: Uuid,
    pub bot_id: BotId,
    pub action_type: ActionType,
    pub opportunity_id: Option<Uuid>,
    pub amount_in: u64,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub expected_amount_out: u64,
    pub max_slippage_percent: f64,
    pub priority_fee: Option<u64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionType {
    Buy,
    Sell,
    Swap,
    AddLiquidity,
    RemoveLiquidity,
    Arbitrage,
    MevExtraction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub action_id: Uuid,
    pub bot_id: BotId,
    pub signature: Option<Signature>,
    pub status: TradeStatus,
    pub executed_at: Option<DateTime<Utc>>,
    pub actual_amount_out: Option<u64>,
    pub actual_slippage_percent: Option<f64>,
    pub gas_used: Option<u64>,
    pub profit_loss_usd: Option<f64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TradeStatus {
    Pending,
    Submitted,
    Confirmed,
    Failed,
    Cancelled,
}

// ============================================================================
// Event System Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlatformEvent {
    MarketEvent(MarketEvent),
    BotEvent(BotEvent),
    SystemEvent(SystemEvent),
    OpportunityEvent(OpportunityEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEvent {
    NewPool(PoolInfo),
    PriceChange {
        token: Pubkey,
        old_price: f64,
        new_price: f64,
        change_percent: f64,
    },
    VolumeSpike {
        pool_id: Pubkey,
        volume_usd: f64,
        spike_factor: f64,
    },
    LiquidityChange {
        pool_id: Pubkey,
        old_liquidity: f64,
        new_liquidity: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BotEvent {
    BotStarted(BotId, BotType),
    BotStopped(BotId, BotType),
    BotError(BotId, String),
    TradeExecuted(BotId, TradeResult),
    OpportunityDetected(BotId, TradingOpportunity),
    StatusChanged(BotStatus),
    Error { bot_id: BotId, error: String },
    MetricsUpdate { bot_id: BotId, metrics: BotMetrics },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    PlatformStarted,
    PlatformStopping,
    ResourceAllocationChanged,
    ConfigurationUpdated,
    HealthCheckFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityEvent {
    OpportunityCreated(TradingOpportunity),
    OpportunityAssigned(Uuid, BotId),
    OpportunityExpired(Uuid),
    OpportunityConflict(Vec<Uuid>),
}

// ============================================================================
// Resource Management Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub network_connections: u32,
    pub storage_mb: u64,
    pub priority: Priority,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub cpu_allocation: CpuAllocation,
    pub memory_allocation: MemoryAllocation,
    pub network_allocation: NetworkAllocation,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuAllocation {
    pub allocated_percent: f64,
    pub thread_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAllocation {
    pub allocated_mb: u64,
    pub max_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAllocation {
    pub max_connections: u32,
    pub rate_limit_per_second: u32,
}

// ============================================================================
// Utility Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub component: String,
    pub message: Option<String>,
    pub checked_at: DateTime<Utc>,
    pub metrics: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformMetrics {
    pub total_bots: usize,
    pub active_bots: usize,
    pub total_trades: u64,
    pub successful_trades: u64,
    pub total_volume_usd: f64,
    pub total_profit_usd: f64,
    pub avg_latency_ms: f64,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub uptime_seconds: u64,
    pub last_updated: DateTime<Utc>,
}
