// ================================================================================
// SNIPERFORGE CORE - INFRAESTRUCTURA REUTILIZABLE
// ================================================================================
// Core modules compartidos entre todos los bots (arbitrage, sniper, mev, etc.)
// Diseño modular para máxima reutilización y mantenibilidad
// ================================================================================

#![allow(dead_code)]
#![allow(unused_imports)]

// ===== MÓDULOS CORE =====
pub mod config;     // Configuration management system
pub mod jupiter;    // Jupiter V6 integration layer
pub mod wallet;     // Wallet management and transactions
pub mod fees;       // Fee calculation and optimization
pub mod feeds;      // Price feeds and market data
pub mod utils;      // Common utilities and helpers
pub mod feeds;      // Price feeds from multiple sources
pub mod utils;      // Shared utilities and helpers

// ===== RE-EXPORTS PRINCIPALES =====
pub use config::{ConfigManager, ConfigError};
pub use jupiter::{JupiterClient, JupiterError, SwapQuote, SwapResult};
pub use wallet::{WalletManager, WalletError, TransactionBuilder};
pub use fees::{FeeCalculator, FeeBreakdown, FeeOptimizer};
pub use feeds::{PriceFeedManager, PriceSource, MarketData};
pub use utils::{Logger, ErrorHandler, PerformanceMonitor};

// ===== TIPOS COMPARTIDOS =====
use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use anyhow::Result;

/// Resultado estándar para todas las operaciones
pub type CoreResult<T> = Result<T, CoreError>;

/// Errores core del sistema
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Jupiter integration error: {0}")]
    Jupiter(#[from] JupiterError),
    
    #[error("Wallet management error: {0}")]
    Wallet(#[from] WalletError),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Token información estándar
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub mint: Pubkey,
    pub symbol: String,
    pub decimals: u8,
    pub name: String,
    pub verified: bool,
}

/// Precio de token con metadata
#[derive(Debug, Clone)]
pub struct TokenPrice {
    pub token: TokenInfo,
    pub price_usd: f64,
    pub price_sol: f64,
    pub volume_24h: f64,
    pub liquidity: f64,
    pub source: String,
    pub timestamp: u64,
}

/// Oportunidad de trading base
#[derive(Debug, Clone)]
pub struct TradingOpportunity {
    pub id: String,
    pub token_in: TokenInfo,
    pub token_out: TokenInfo,
    pub amount_in: u64,
    pub expected_out: u64,
    pub price_impact: f64,
    pub fees: FeeBreakdown,
    pub route: Vec<String>,
    pub confidence: f64,
    pub timestamp: u64,
}

/// Configuración base para todos los bots
#[derive(Debug, Clone)]
pub struct CoreConfig {
    pub rpc_url: String,
    pub commitment: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub log_level: String,
    pub environment: Environment,
}

/// Entorno de ejecución
#[derive(Debug, Clone)]
pub enum Environment {
    Development,
    Testing,
    Production,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            commitment: "confirmed".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            log_level: "info".to_string(),
            environment: Environment::Development,
        }
    }
}

// ===== TRAITS COMPARTIDOS =====

/// Trait para todos los bots
pub trait Bot {
    type Config;
    type Error;
    type Opportunity;
    
    /// Inicializar el bot con configuración
    fn new(config: Self::Config) -> CoreResult<Self> 
    where 
        Self: Sized;
    
    /// Ejecutar un ciclo de búsqueda de oportunidades
    async fn discover_opportunities(&mut self) -> CoreResult<Vec<Self::Opportunity>>;
    
    /// Ejecutar una oportunidad específica
    async fn execute_opportunity(&mut self, opportunity: &Self::Opportunity) -> CoreResult<()>;
    
    /// Obtener estadísticas del bot
    fn get_statistics(&self) -> BotStatistics;
    
    /// Pausar/reanudar el bot
    fn set_active(&mut self, active: bool);
}

/// Estadísticas estándar para todos los bots
#[derive(Debug, Clone)]
pub struct BotStatistics {
    pub total_opportunities: u64,
    pub executed_trades: u64,
    pub successful_trades: u64,
    pub failed_trades: u64,
    pub total_profit_sol: f64,
    pub total_profit_usd: f64,
    pub success_rate: f64,
    pub average_profit: f64,
    pub uptime_seconds: u64,
    pub last_activity: u64,
}

impl Default for BotStatistics {
    fn default() -> Self {
        Self {
            total_opportunities: 0,
            executed_trades: 0,
            successful_trades: 0,
            failed_trades: 0,
            total_profit_sol: 0.0,
            total_profit_usd: 0.0,
            success_rate: 0.0,
            average_profit: 0.0,
            uptime_seconds: 0,
            last_activity: 0,
        }
    }
}

/// Trait para manejo de configuración
pub trait Configurable {
    type Config;
    
    fn load_config(path: &str) -> CoreResult<Self::Config>;
    fn save_config(config: &Self::Config, path: &str) -> CoreResult<()>;
    fn validate_config(config: &Self::Config) -> CoreResult<()>;
}

/// Trait para logging estructurado
pub trait Loggable {
    fn log_info(&self, message: &str);
    fn log_warn(&self, message: &str);
    fn log_error(&self, message: &str);
    fn log_debug(&self, message: &str);
}

// ===== CONSTANTES GLOBALES =====

/// Tokens principales de Solana
pub mod tokens {
    use super::*;
    use std::str::FromStr;
    
    pub fn sol() -> TokenInfo {
        TokenInfo {
            mint: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
            symbol: "SOL".to_string(),
            decimals: 9,
            name: "Solana".to_string(),
            verified: true,
        }
    }
    
    pub fn usdc() -> TokenInfo {
        TokenInfo {
            mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
            symbol: "USDC".to_string(),
            decimals: 6,
            name: "USD Coin".to_string(),
            verified: true,
        }
    }
    
    pub fn usdt() -> TokenInfo {
        TokenInfo {
            mint: Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB").unwrap(),
            symbol: "USDT".to_string(),
            decimals: 6,
            name: "Tether USD".to_string(),
            verified: true,
        }
    }
}

/// URLs y endpoints comunes
pub mod endpoints {
    pub const JUPITER_V6_QUOTE: &str = "https://quote-api.jup.ag/v6";
    pub const JUPITER_V6_PRICE: &str = "https://api.jup.ag/price/v2";
    pub const DEXSCREENER_API: &str = "https://api.dexscreener.com/latest/dex";
    pub const SOLANA_MAINNET: &str = "https://api.mainnet-beta.solana.com";
    pub const SOLANA_DEVNET: &str = "https://api.devnet.solana.com";
}

/// Configuraciones por defecto
pub mod defaults {
    pub const MIN_SOL_BALANCE: f64 = 0.01;
    pub const DEFAULT_SLIPPAGE_BPS: u16 = 100; // 1%
    pub const DEFAULT_TIMEOUT_MS: u64 = 30000;
    pub const MAX_RETRIES: u32 = 3;
    pub const PRIORITY_FEE_LAMPORTS: u64 = 1000;
}

// ===== HELPERS PÚBLICOS =====

/// Convertir SOL a lamports
pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1_000_000_000.0) as u64
}

/// Convertir lamports a SOL
pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000_000.0
}

/// Obtener timestamp actual
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Validar dirección Solana
pub fn is_valid_pubkey(address: &str) -> bool {
    Pubkey::from_str(address).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sol_to_lamports() {
        assert_eq!(sol_to_lamports(1.0), 1_000_000_000);
        assert_eq!(sol_to_lamports(0.001), 1_000_000);
    }
    
    #[test]
    fn test_lamports_to_sol() {
        assert_eq!(lamports_to_sol(1_000_000_000), 1.0);
        assert_eq!(lamports_to_sol(1_000_000), 0.001);
    }
    
    #[test]
    fn test_token_constants() {
        let sol = tokens::sol();
        assert_eq!(sol.symbol, "SOL");
        assert_eq!(sol.decimals, 9);
        assert!(sol.verified);
    }
}
