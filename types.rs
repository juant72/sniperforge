// 🏗️ TYPES AND STRUCTURES - Core data types for the arbitrage system
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant, SystemTime};
use solana_sdk::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;
use serde::{Deserialize, Serialize};

// ===== CORE CONSTANTS =====
pub const MIN_TRADE_SIZE_SOL: f64 = 0.1;
pub const MAX_TRADE_SIZE_SOL: f64 = 100.0;
pub const MAX_SLIPPAGE_BPS: u64 = 200; // 2.0%
pub const MILITARY_MIN_PROFIT_BPS: u64 = 50; // 0.5%

// ===== POOL TYPES =====
#[derive(Debug, Clone, PartialEq)]
pub enum PoolType {
    Raydium,
    Orca, 
    OrcaWhirlpool,
    Meteora,
    Jupiter,
    Lifinity,
    Phoenix,
    Saber,
    Cropper,
    Aldrin,
    Step,
    Marinade,
    Mercurial,
    Quarry,
    Unknown,
}

// ===== MARKET SENTIMENT =====
#[derive(Debug, Clone, PartialEq)]
pub enum MarketSentiment {
    Bullish,
    Bearish,
    Neutral,
    HighVolatility,
}

// ===== POOL DATA STRUCTURES =====
#[derive(Debug, Clone)]
pub struct PoolData {
    pub address: Pubkey,
    pub pool_type: PoolType,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub fee_rate: u64, // in basis points
    pub tvl_usd: f64,
    pub last_updated: SystemTime,
}

// ===== DISCOVERED POOL STRUCTURE =====
#[derive(Debug, Clone)]
pub struct DiscoveredPool {
    pub address: Pubkey,
    pub pool_type: PoolType,
    pub token_a_symbol: String,
    pub token_b_symbol: String,
}

#[derive(Debug, Clone)]
pub struct RealPoolData {
    pub token_a_reserve: u64,
    pub token_b_reserve: u64,
    pub fee_rate: u64,
}

// ===== PRICE FEED STRUCTURES =====
#[derive(Debug, Clone)]
pub struct PriceData {
    pub price: f64,
    pub timestamp: SystemTime,
    pub source: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub symbol: String,
    pub mint: String,
    pub decimals: u8,
    pub price_usd: f64,
    pub volume_24h: f64,
    pub last_update: u64,
    pub liquidity: f64,
    pub market_cap: f64,
}

// ===== ARBITRAGE STRUCTURES =====
#[derive(Debug, Clone)]
pub struct DirectOpportunity {
    pub pool_a: PoolData,
    pub pool_b: PoolData,
    pub intermediate_token: Pubkey,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount_in: u64,
    pub expected_amount_out: u64,
    pub profit_lamports: i64,
    pub profit_percentage: f64,
    pub fees_lamports: u64,
    pub route_type: String,
}

#[derive(Debug, Clone)]
pub struct PoolPerformanceData {
    pub total_volume: f64,
    pub average_spread: f64,
    pub success_rate: f64,
    pub last_profitable_trade: Option<SystemTime>,
    pub volatility_score: f64,
}

// ===== JUPITER API STRUCTURES =====
#[derive(Debug, Clone)]
pub struct JupiterQuote {
    pub out_amount: u64,
    pub price_impact_pct: f64,
    pub route_plan: Vec<String>,
}

// ===== METRICS STRUCTURES =====
#[derive(Debug, Clone)]
pub struct RiskMetrics {
    pub max_exposure_usd: f64,
    pub current_exposure_usd: f64,
    pub daily_pnl: f64,
    pub success_rate: f64,
    pub average_profit_bps: f64,
    pub max_drawdown: f64,
}

#[derive(Debug, Clone)]
pub struct MarketMetrics {
    pub timestamp: u64,
    pub total_volume_24h: f64,
    pub average_spread: f64,
    pub volatility_index: f64,
    pub liquidity_score: f64,
    pub market_sentiment: MarketSentiment,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_trades: u64,
    pub successful_trades: u64,
    pub total_profit_usd: f64,
    pub average_execution_time_ms: f64,
    pub best_profit_opportunity: f64,
    pub hourly_pnl: VecDeque<f64>,
}

#[derive(Debug, Clone)]
pub struct AdaptiveConfig {
    pub max_slippage_bps: u64,
    pub min_profit_threshold: u64,
    pub max_trade_amount: u64,
    pub risk_multiplier: f64,
    pub volatility_adjustment: f64,
    pub latency_compensation: f64,
}

// ===== EXECUTION MODE ENUM =====
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    Simulation,
    RealTrading,
}

// ===== MAIN ENGINE STRUCTURE =====
pub struct ProfessionalArbitrageEngine {
    // Core infrastructure
    pub client: RpcClient,
    pub wallet_address: Pubkey,
    pub jupiter_client: reqwest::Client,
    
    // Professional modules
    pub price_feeds: crate::price_feeds::ProfessionalPriceFeeds,
    pub pool_validator: crate::pool_validator::PoolValidator,
    
    // Enhanced data management
    pub operational_pools: HashMap<Pubkey, PoolData>,
    pub pool_performance: HashMap<Pubkey, PoolPerformanceData>,
    pub monitoring_pools: Vec<String>,
    
    // Risk management
    pub risk_metrics: RiskMetrics,
    pub market_metrics: MarketMetrics,
    pub performance_metrics: PerformanceMetrics,
    
    // Configuration and state
    pub adaptive_config: AdaptiveConfig,
    pub is_running: AtomicBool,
    pub emergency_stop: AtomicBool,
    
    // Performance tracking
    pub last_price_update: Instant,
    pub execution_times: VecDeque<u64>,
    pub profit_history: VecDeque<f64>,
    
    // Statistics
    pub total_opportunities_found: AtomicU64,
    pub successful_trades: AtomicU64,
    pub total_profit_lamports: AtomicU64,
    pub risk_events: AtomicU64,
    
    // NEW: Real execution components (simplified)
    pub execution_mode: ExecutionMode,
    pub wallet_keypair: Option<solana_sdk::signature::Keypair>,
    pub real_executor: Option<()>, // Placeholder - functionality is in internal modules
    
    // PROPOSAL-003: Multi-token arbitrage support (optional - backward compatible)
    pub multi_token_config: Option<Box<dyn std::any::Any + Send + Sync>>, // Type-erased para evitar imports circulares
    pub multi_token_enabled: bool,
    pub multi_token_tier2_enabled: Option<bool>, // PROPOSAL-003 Phase 2: Tier 2 ecosystem support
}
}
