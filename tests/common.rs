//! Common test utilities and helpers

use sniperforge::config::SimpleConfig;
use sniperforge::types::{ArbitragePair, Token};

/// Create a standard test configuration with enterprise features
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
        
        // 🚀 Enterprise test suite configuration
        trading_amount: 0.01,
        profit_threshold: 0.5,
        max_price_age_seconds: 30,
        risk_percentage: 2.0,
        enable_ml_analysis: true,                // Enable ML for comprehensive testing
        enable_sentiment_analysis: true,         // Enable sentiment analysis testing
        enable_technical_analysis: true,         // Enable technical analysis testing
        max_concurrent_trades: 3,                // Conservative for test stability
        portfolio_rebalancing: true,
        stop_loss_percentage: 5.0,
        take_profit_percentage: 10.0,
        
        // RPC Configuration fields (as Options)
        use_secondary_rpc: Some(false),
        rpc_retry_attempts: Some(3),
        rpc_timeout_ms: Some(5000),
    }
}

// This function is kept for potential future tests but marked as allow(dead_code)
#[allow(dead_code)]
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
        pool_address: Some("TestPoolAddress123".to_string()),
        fee_rate: 0.003, // 0.3%
    }
}
