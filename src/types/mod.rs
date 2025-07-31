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
