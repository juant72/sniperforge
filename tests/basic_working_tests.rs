//! Basic Working Tests - Simple validation that core components work

use sniperforge::{
    config::SimpleConfig,
    types::{Token, ArbitragePair, ArbitrageOpportunity, PriceInfo},
};
use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::Duration;

#[test]
fn test_token_creation() {
    let token = Token {
        symbol: "SOL".to_string(),
        mint: "So11111111111111111111111111111111111111112".to_string(),
        decimals: 9,
    };
    
    assert_eq!(token.symbol, "SOL");
    assert_eq!(token.decimals, 9);
    println!("✅ Token creation works");
}

#[test]
fn test_arbitrage_pair_creation() {
    let base_token = Token {
        symbol: "SOL".to_string(),
        mint: "So11111111111111111111111111111111111111112".to_string(),
        decimals: 9,
    };
    
    let quote_token = Token {
        symbol: "USDC".to_string(),
        mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        decimals: 6,
    };
    
    let pair = ArbitragePair {
        base_token,
        quote_token,
        pool_address: Some("TestPool".to_string()),
        fee_rate: 0.003,
    };
    
    assert_eq!(pair.base_token.symbol, "SOL");
    assert_eq!(pair.quote_token.symbol, "USDC");
    println!("✅ ArbitragePair creation works");
}

#[test]
fn test_arbitrage_opportunity_creation() {
    let base_token = Token {
        symbol: "SOL".to_string(),
        mint: "So11111111111111111111111111111111111111112".to_string(),
        decimals: 9,
    };
    
    let quote_token = Token {
        symbol: "USDC".to_string(),
        mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        decimals: 6,
    };
    
    let pair = ArbitragePair {
        base_token,
        quote_token,
        pool_address: Some("TestPool".to_string()),
        fee_rate: 0.003,
    };
    
    let opportunity = ArbitrageOpportunity {
        pair,
        buy_exchange: "Raydium".to_string(),
        sell_exchange: "Orca".to_string(),
        buy_price: 100.0,
        sell_price: 101.0,
        profit_percentage: 1.0,
        volume_required: 10.0,
        estimated_gas_cost: 0.01,
        confidence_score: 0.95,
        timestamp: chrono::Utc::now(),
        execution_time_window: Duration::from_secs(60),
    };
    
    assert_eq!(opportunity.buy_exchange, "Raydium");
    assert_eq!(opportunity.sell_exchange, "Orca");
    assert!(opportunity.profit_percentage > 0.0);
    println!("✅ ArbitrageOpportunity creation works");
}

#[test]
fn test_price_info_creation() {
    let price_info = PriceInfo {
        mint: "So11111111111111111111111111111111111111112".to_string(),
        usd: Decimal::from_str("100.50").unwrap(),
        timestamp: Utc::now(),
        source: "Test".to_string(),
    };
    
    assert_eq!(price_info.mint, "So11111111111111111111111111111111111111112");
    assert!(price_info.usd > Decimal::ZERO);
    println!("✅ PriceInfo creation works");
}

#[test]
fn test_config_creation() {
    let config = SimpleConfig::default();
    
    assert!(!config.solana_rpc_url.is_empty());
    assert!(config.min_profit_threshold > 0.0);
    assert!(config.max_slippage > 0.0);
    println!("✅ SimpleConfig creation works");
}
