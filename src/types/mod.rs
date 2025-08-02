//! Common types used across the SniperForge suite

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

/// Result type for SniperForge operations
pub type ApiResult<T> = std::result::Result<T, String>;

/// Token identifier on Solana
pub type TokenMint = String;

/// Wallet public key
pub type WalletAddress = String;

/// Transaction signature
pub type TransactionSignature = String;

/// Amount in lamports
pub type Lamports = u64;

/// Price type
pub type Price = f64;

/// Volume type
pub type Volume = f64;

/// Percentage type
pub type Percentage = f64;

/// Balance type
pub type Balance = f64;

/// Timestamp type
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Enterprise Trading Mode - Real blockchain execution environments
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TradingMode {
    /// DevNet environment - Real transactions on development network
    DevNet,
    /// MainNet environment - Real transactions on production network  
    MainNet,
    /// TestNet environment - Real transactions on test network
    TestNet,
    /// Simulation mode - No real transactions, simulation only
    Simulation,
}

impl TradingMode {
    /// Check if mode involves real transactions
    pub fn is_real_trading(&self) -> bool {
        matches!(self, TradingMode::DevNet | TradingMode::MainNet | TradingMode::TestNet)
    }
    
    /// Check if mode is production environment
    pub fn is_production(&self) -> bool {
        matches!(self, TradingMode::MainNet)
    }
    
    /// Check if mode is safe for testing
    pub fn is_safe_testing(&self) -> bool {
        matches!(self, TradingMode::DevNet | TradingMode::TestNet | TradingMode::Simulation)
    }
    
    /// Get environment name
    pub fn environment_name(&self) -> &'static str {
        match self {
            TradingMode::DevNet => "DevNet",
            TradingMode::MainNet => "MainNet", 
            TradingMode::TestNet => "TestNet",
            TradingMode::Simulation => "Simulation",
        }
    }
    
    /// Get RPC endpoint suffix for Solana networks
    pub fn rpc_suffix(&self) -> &'static str {
        match self {
            TradingMode::DevNet => "devnet",
            TradingMode::MainNet => "mainnet-beta",
            TradingMode::TestNet => "testnet", 
            TradingMode::Simulation => "devnet", // Use devnet for simulation
        }
    }
}

impl Default for TradingMode {
    fn default() -> Self {
        TradingMode::DevNet // Safe default for development
    }
}

impl std::fmt::Display for TradingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.environment_name())
    }
}

/// Core token representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Token {
    pub symbol: String,
    pub mint: String,
    pub decimals: u8,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            decimals: 9,
        }
    }
}

/// Trading pair representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TradingPair {
    /// Base token mint
    pub base: TokenMint,
    /// Quote token mint  
    pub quote: TokenMint,
}

impl TradingPair {
    /// Create a new trading pair
    pub fn new(base: TokenMint, quote: TokenMint) -> Self {
        Self { base, quote }
    }
    
    /// Get the reverse pair (quote/base)
    pub fn reverse(&self) -> Self {
        Self {
            base: self.quote.clone(),
            quote: self.base.clone(),
        }
    }
}

/// Price information for a token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceInfo {
    /// Token mint address
    pub mint: TokenMint,
    /// Price in USD
    pub usd: Decimal,
    /// Price timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Source of the price
    pub source: String,
}

/// Token balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    /// Token mint
    pub mint: TokenMint,
    /// Amount in native units
    pub amount: u64,
    /// Decimal places for display
    pub decimals: u8,
    /// USD value (if available)
    pub usd_value: Option<Decimal>,
}

/// Trading opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    /// Unique identifier
    pub id: String,
    /// Trading pair
    pub pair: TradingPair,
    /// Expected profit percentage
    pub profit_bps: u16,
    /// Recommended trade amount
    pub amount: u64,
    /// Route information
    pub route: Vec<String>,
    /// Confidence score (0-100)
    pub confidence: u8,
    /// Expiration time
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Error types used throughout the system
#[derive(Debug, thiserror::Error)]
pub enum SniperForgeError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Trading error
    #[error("Trading error: {0}")]
    Trading(String),
    
    /// API error
    #[error("API error: {0}")]
    Api(String),
    
    /// Security error
    #[error("Security error: {0}")]
    Security(String),
    
    /// Network error
    #[error("Network error: {0}")]
    Network(String),
    
    /// Insufficient funds
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },
    
    /// Timeout error
    #[error("Operation timed out after {seconds} seconds")]
    Timeout { seconds: u64 },
}

impl From<String> for SniperForgeError {
    fn from(error: String) -> Self {
        SniperForgeError::Config(error)
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, SniperForgeError>;

/// Common constants
pub mod constants {
    /// SOL mint address
    pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
    
    /// USDC mint address
    pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    
    /// USDT mint address  
    pub const USDT_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    
    /// Lamports per SOL
    pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
    
    /// Basis points in 100%
    pub const BPS_DENOMINATOR: u16 = 10_000;
}

/// Enhanced trading pair for arbitrage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitragePair {
    pub base_token: Token,
    pub quote_token: Token,
    pub pool_address: Option<String>,
    pub fee_rate: f64,
}

impl Default for ArbitragePair {
    fn default() -> Self {
        Self {
            base_token: Token::default(),
            quote_token: Token {
                symbol: "USDC".to_string(),
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                decimals: 6,
            },
            pool_address: None,
            fee_rate: 0.003,
        }
    }
}

/// Arbitrage opportunity representation
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub execution_time_window: Duration,
}

impl Default for ArbitrageOpportunity {
    fn default() -> Self {
        Self {
            pair: ArbitragePair::default(),
            buy_exchange: "Raydium".to_string(),
            sell_exchange: "Orca".to_string(),
            buy_price: 100.0,
            sell_price: 100.5,
            profit_percentage: 0.005,
            volume_required: 10.0,
            estimated_gas_cost: 0.001,
            confidence_score: 0.8,
            timestamp: chrono::Utc::now(),
            execution_time_window: Duration::from_secs(30),
        }
    }
}

/// Market data container
#[derive(Debug, Clone, Default)]
pub struct MarketData {
    pub prices: HashMap<String, f64>,
    pub volumes: HashMap<String, f64>,
    pub liquidity: HashMap<String, f64>,
    pub last_updated: Option<Instant>,
    /// Current price for the primary trading pair
    pub current_price: f64,
    /// 24-hour trading volume
    pub volume_24h: f64,
    /// Bid-ask spread percentage
    pub bid_ask_spread: f64,
}

impl MarketData {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn get_price(&self, symbol: &str) -> Option<f64> {
        self.prices.get(symbol).copied()
    }
    
    pub fn set_price(&mut self, symbol: String, price: f64) {
        self.prices.insert(symbol, price);
        self.last_updated = Some(Instant::now());
    }
    
    pub fn get_volume(&self, symbol: &str) -> Option<f64> {
        self.volumes.get(symbol).copied()
    }
    
    pub fn set_volume(&mut self, symbol: String, volume: f64) {
        self.volumes.insert(symbol, volume);
    }
    
    pub fn get_liquidity(&self, symbol: &str) -> Option<f64> {
        self.liquidity.get(symbol).copied()
    }
    
    pub fn set_liquidity(&mut self, symbol: String, liquidity: f64) {
        self.liquidity.insert(symbol, liquidity);
    }
    
    pub fn is_stale(&self, max_age: Duration) -> bool {
        match self.last_updated {
            Some(updated) => updated.elapsed() > max_age,
            None => true,
        }
    }
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub rpc_status: ConnectionStatus,
    pub websocket_status: ConnectionStatus,
    pub api_status: HashMap<String, ConnectionStatus>,
    pub last_trade: Option<chrono::DateTime<chrono::Utc>>,
    pub uptime: Duration,
    pub memory_usage: f64,
    pub cpu_usage: f64,
}

/// Connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error(String),
    Unknown,
}

/// Platform-specific error types for enterprise trading system
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    /// Wallet management error
    #[error("Wallet management error: {0}")]
    WalletManagement(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Network connectivity error
    #[error("Network error: {0}")]
    Network(String),
    
    /// Trading execution error
    #[error("Trading error: {0}")]
    Trading(String),
    
    /// Jupiter API connection error
    #[error("Jupiter connection error: {0}")]
    JupiterConnectionError(String),
    
    /// Jupiter quote error
    #[error("Jupiter quote error: {0}")]
    JupiterQuoteError(String),
    
    /// RPC pool initialization error
    #[error("RPC pool initialization error: {0}")]
    RpcPoolInitError(String),
    
    /// RPC operation error
    #[error("RPC error: {0}")]
    RpcError(String),
    
    /// RPC timeout error
    #[error("RPC timeout: {0}")]
    RpcTimeout(String),
    
    /// Wallet manager initialization error
    #[error("Wallet manager initialization error: {0}")]
    WalletManagerInitError(String),
    
    /// Wallet not found error
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    
    /// Wallet validation error
    #[error("Wallet validation error: {0}")]
    WalletValidationError(String),
}

/// Detailed health status for individual components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentHealthStatus {
    /// Component is healthy and functioning normally
    Healthy,
    /// Component is degraded with specific issues
    Degraded(Vec<String>),
    /// Component is unhealthy or offline
    Unhealthy(String),
}

impl ComponentHealthStatus {
    /// Check if component is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, ComponentHealthStatus::Healthy)
    }
    
    /// Check if component is degraded
    pub fn is_degraded(&self) -> bool {
        matches!(self, ComponentHealthStatus::Degraded(_))
    }
    
    /// Check if component is unhealthy
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, ComponentHealthStatus::Unhealthy(_))
    }
    
    /// Get status description
    pub fn description(&self) -> String {
        match self {
            ComponentHealthStatus::Healthy => "Healthy".to_string(),
            ComponentHealthStatus::Degraded(issues) => format!("Degraded: {}", issues.join(", ")),
            ComponentHealthStatus::Unhealthy(reason) => format!("Unhealthy: {}", reason),
        }
    }
}

/// Type of trading opportunity for strategy framework
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OpportunityType {
    /// Arbitrage opportunity between exchanges
    Arbitrage,
    /// Momentum trading opportunity
    Momentum,
    /// Mean reversion opportunity
    MeanReversion,
    /// Trend following opportunity
    TrendFollowing,
    /// Grid trading opportunity
    Grid,
}

impl std::fmt::Display for OpportunityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpportunityType::Arbitrage => write!(f, "Arbitrage"),
            OpportunityType::Momentum => write!(f, "Momentum"),
            OpportunityType::MeanReversion => write!(f, "MeanReversion"),
            OpportunityType::TrendFollowing => write!(f, "TrendFollowing"),
            OpportunityType::Grid => write!(f, "Grid"),
        }
    }
}

/// Trading opportunity structure for strategy framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingOpportunity {
    /// Type of opportunity
    pub opportunity_type: OpportunityType,
    /// Trading pair (e.g., "SOL/USDC")
    pub token_pair: String,
    /// Expected profit percentage
    pub profit_percentage: f64,
    /// 24-hour volume
    pub volume_24h: f64,
    /// Available liquidity
    pub liquidity: f64,
    /// Source exchange
    pub source_exchange: String,
    /// Target exchange
    pub target_exchange: String,
    /// Entry price
    pub entry_price: f64,
    /// Exit price
    pub exit_price: f64,
    /// Risk score (0.0-1.0)
    pub risk_score: f64,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
    /// Timestamp when opportunity was detected
    pub timestamp: Timestamp,
    /// Maximum execution time window
    pub execution_window: Duration,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for TradingOpportunity {
    fn default() -> Self {
        Self {
            opportunity_type: OpportunityType::Arbitrage,
            token_pair: String::new(),
            profit_percentage: 0.0,
            volume_24h: 0.0,
            liquidity: 0.0,
            source_exchange: String::new(),
            target_exchange: String::new(),
            entry_price: 0.0,
            exit_price: 0.0,
            risk_score: 0.0,
            confidence: 0.0,
            timestamp: chrono::Utc::now(),
            execution_window: Duration::from_secs(30),
            metadata: HashMap::new(),
        }
    }
}

impl TradingOpportunity {
    /// Create a new trading opportunity
    pub fn new(
        opportunity_type: OpportunityType,
        token_pair: String,
        profit_percentage: f64,
        source_exchange: String,
        target_exchange: String,
    ) -> Self {
        Self {
            opportunity_type,
            token_pair,
            profit_percentage,
            source_exchange,
            target_exchange,
            timestamp: chrono::Utc::now(),
            ..Default::default()
        }
    }
    
    /// Check if opportunity is still valid based on timestamp
    pub fn is_valid(&self) -> bool {
        let elapsed = chrono::Utc::now()
            .signed_duration_since(self.timestamp)
            .to_std()
            .unwrap_or(Duration::from_secs(0));
        elapsed < self.execution_window
    }
    
    /// Calculate potential profit in USD
    pub fn calculate_profit(&self, investment: f64) -> f64 {
        investment * (self.profit_percentage / 100.0)
    }
}
