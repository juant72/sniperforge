//! SniperForge Enterprise Test Suite
//! Professional testing organization with clear separation of concerns

pub mod unit;
pub mod integration;
pub mod security;
pub mod performance;

/// Test utilities and helpers
pub mod helpers {
    use chrono::Utc;
    use sniperforge::{
        config::SimpleConfig,
        types::{ArbitragePair, Token, PriceInfo},
    };
    use rust_decimal::Decimal;
    use std::str::FromStr;

    /// Create a standard test configuration
    pub fn create_test_config() -> SimpleConfig {
        SimpleConfig {
            solana_rpc_url: "https://api.devnet.solana.com".to_string(),
            solana_ws_url: "wss://api.devnet.solana.com".to_string(),
            min_profit_threshold: 0.01,
            max_slippage: 0.05,
            max_position_size: 1.0,
            private_key_path: "./test-wallet.json".to_string(),
            enable_simulation: true,
            log_level: "info".to_string(),
            dexscreener_base_url: "https://api.dexscreener.com".to_string(),
            max_requests_per_second: 10,
            cooldown_period_ms: 1000,
            max_history_size: 1000,
        }
    }

    /// Create a test SOL/USDC arbitrage pair
    pub fn create_test_sol_usdc_pair() -> ArbitragePair {
        ArbitragePair {
            base_token: Token {
                symbol: "SOL".to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                decimals: 9,
            },
            quote_token: Token {
                symbol: "USDC".to_string(),
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                decimals: 6,
            },
            pool_address: Some("TestPoolAddress".to_string()),
            fee_rate: 0.003,
        }
    }

    /// Create test price info
    pub fn create_test_price_info(mint: &str, price: &str) -> PriceInfo {
        PriceInfo {
            mint: mint.to_string(),
            usd: Decimal::from_str(price).unwrap(),
            timestamp: Utc::now(),
            source: "test".to_string(),
        }
    }

    /// Validate Solana token address format
    pub fn is_valid_solana_address(address: &str) -> bool {
        address.len() == 44 && 
        address.chars().all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c))
    }

    /// Common test constants
    pub mod constants {
        pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
        pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        pub const USDT_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
        
        pub const TEST_TIMEOUT_SECS: u64 = 30;
        pub const HFT_SPEED_REQUIREMENT_MS: u64 = 10;
    }
}
