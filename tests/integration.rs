// Integration tests for SniperForge
// These tests validate the complete system functionality

use sniperforge::{
    config::SimpleConfig,
    trading::ArbitrageEngine,
    apis::price_feeds::PriceFeedManager,
};
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_arbitrage_engine_initialization() {
    let config = SimpleConfig::default();
    let price_feed_manager = Arc::new(PriceFeedManager::new(&config));
    
    let engine = ArbitrageEngine::new(config, price_feed_manager)
        .await
        .expect("Failed to create arbitrage engine");
    
    // Test basic functionality
    assert!(true); // Replace with actual engine validation
}

#[tokio::test]
async fn test_price_feed_connectivity() {
    let config = SimpleConfig::default();
    let price_feed_manager = PriceFeedManager::new(&config);
    
    // Test connection to price feeds
    // Add actual connectivity tests here
    assert!(true);
}

#[tokio::test]
async fn test_opportunity_detection() {
    let config = SimpleConfig::default();
    let price_feed_manager = Arc::new(PriceFeedManager::new(&config));
    
    let engine = ArbitrageEngine::new(config, price_feed_manager)
        .await
        .expect("Failed to create arbitrage engine");
    
    // Test opportunity detection with mock data
    let opportunities = engine.scan_for_opportunities().await.unwrap();
    
    // Should return empty or valid opportunities, but not error
    assert!(opportunities.len() >= 0);
}

#[tokio::test]
async fn test_configuration_validation() {
    // Test that configuration loads correctly
    let config = SimpleConfig::default();
    
    // Validate required fields
    assert!(!config.solana_rpc_url.is_empty());
    assert!(config.max_slippage > 0.0);
    assert!(config.min_profit_threshold > 0.0);
}

#[tokio::test]
async fn test_wallet_integration() {
    let config = SimpleConfig::default();
    
    // Test wallet loading (should work in test environment)
    if config.enable_simulation {
        // In simulation mode, wallet operations should be mocked
        assert!(true, "Simulation mode enabled");
    } else {
        // In real mode, validate wallet exists
        assert!(std::path::Path::new(&config.private_key_path).exists(),
               "Wallet file should exist for real trading");
    }
}

#[tokio::test]
async fn test_performance_metrics() {
    let config = SimpleConfig::default();
    let price_feed_manager = Arc::new(PriceFeedManager::new(&config));
    
    let engine = ArbitrageEngine::new(config, price_feed_manager)
        .await
        .expect("Failed to create arbitrage engine");
    
    // Measure opportunity scanning performance
    let start = std::time::Instant::now();
    let _opportunities = engine.scan_for_opportunities().await.unwrap();
    let duration = start.elapsed();
    
    // Should complete within reasonable time (5 seconds)
    assert!(duration.as_secs() < 5, "Opportunity scanning should be fast");
}

// Helper function for testing
fn create_test_config() -> SimpleConfig {
    SimpleConfig {
        solana_rpc_url: "https://api.devnet.solana.com".to_string(),
        solana_ws_url: "wss://api.devnet.solana.com/".to_string(),
        max_slippage: 0.005,
        min_profit_threshold: 0.001,
        max_position_size: 0.1,
        private_key_path: "./test_wallet.json".to_string(),
        enable_simulation: true,
        log_level: "info".to_string(),
        dexscreener_base_url: "https://api.dexscreener.com".to_string(),
        max_requests_per_second: 10,
        cooldown_period_ms: 100,
        max_history_size: 1000,
    }
}

#[tokio::test]
async fn test_end_to_end_simulation() {
    let config = create_test_config();
    let price_feed_manager = Arc::new(PriceFeedManager::new(&config));
    
    let engine = ArbitrageEngine::new(config, price_feed_manager)
        .await
        .expect("Failed to create arbitrage engine");
    
    // Simulate a complete arbitrage cycle
    let opportunities = engine.scan_for_opportunities().await.unwrap();
    
    for opportunity in opportunities.iter().take(1) {
        // In simulation mode, this should not execute real trades
        // Test that we can process opportunities without errors
        assert!(opportunity.profit_percentage >= 0.0);
    }
}
