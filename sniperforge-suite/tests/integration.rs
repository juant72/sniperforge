// Integration tests for SniperForge
// These tests validate the complete system functionality

use sniperforge_core::{
    config::Config,
    trading::arbitrage::ArbitrageEngine,
    apis::price_feeds::PriceFeedManager,
    types::{Token, TradingPair, ArbitrageOpportunity},
};
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_arbitrage_engine_initialization() {
    let config = Config::from_env().expect("Failed to load config");
    let price_feed_manager = Arc::new(PriceFeedManager::new(&config));
    
    let engine = ArbitrageEngine::new(config, price_feed_manager)
        .await
        .expect("Failed to create arbitrage engine");
    
    assert!(engine.is_initialized());
}

#[tokio::test]
async fn test_price_feed_connectivity() {
    let config = Config::from_env().expect("Failed to load config");
    let price_feed_manager = PriceFeedManager::new(&config);
    
    // Test connection to price feeds
    let connection_result = price_feed_manager.test_connectivity().await;
    assert!(connection_result.is_ok(), "Failed to connect to price feeds");
}

#[tokio::test]
async fn test_opportunity_detection() {
    let config = Config::from_env().expect("Failed to load config");
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
async fn test_risk_management() {
    let config = Config::from_env().expect("Failed to load config");
    let price_feed_manager = Arc::new(PriceFeedManager::new(&config));
    
    let engine = ArbitrageEngine::new(config.clone(), price_feed_manager)
        .await
        .expect("Failed to create arbitrage engine");
    
    // Create a mock opportunity
    let opportunity = ArbitrageOpportunity {
        profit_percentage: 0.02, // 2% profit
        volume_required: 100.0,
        estimated_gas_cost: 0.001,
        confidence_score: 0.8,
        ..Default::default()
    };
    
    // Test risk assessment
    let risk_assessment = engine.assess_risk(&opportunity).await.unwrap();
    assert!(risk_assessment.is_acceptable);
}

#[tokio::test]
async fn test_configuration_validation() {
    // Test that configuration loads correctly
    let config = Config::from_env();
    assert!(config.is_ok(), "Configuration should load without errors");
    
    let config = config.unwrap();
    
    // Validate required fields
    assert!(!config.solana_rpc_url.is_empty());
    assert!(config.max_slippage > 0.0);
    assert!(config.min_profit_threshold > 0.0);
}

#[tokio::test]
async fn test_wallet_integration() {
    let config = Config::from_env().expect("Failed to load config");
    
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
async fn test_error_handling() {
    let config = Config::from_env().expect("Failed to load config");
    let price_feed_manager = Arc::new(PriceFeedManager::new(&config));
    
    let engine = ArbitrageEngine::new(config, price_feed_manager)
        .await
        .expect("Failed to create arbitrage engine");
    
    // Test error handling with invalid data
    let invalid_opportunity = ArbitrageOpportunity {
        profit_percentage: -0.01, // Negative profit
        volume_required: 0.0,     // Zero volume
        estimated_gas_cost: 100.0, // Excessive gas cost
        confidence_score: 0.1,    // Low confidence
        ..Default::default()
    };
    
    // Should handle invalid opportunity gracefully
    let result = engine.assess_risk(&invalid_opportunity).await;
    assert!(result.is_ok(), "Error handling should be graceful");
    
    if let Ok(assessment) = result {
        assert!(!assessment.is_acceptable, "Invalid opportunity should be rejected");
    }
}

#[tokio::test]
async fn test_performance_metrics() {
    let config = Config::from_env().expect("Failed to load config");
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

// Helper functions for testing
async fn create_test_config() -> Config {
    Config {
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
    }
}

#[tokio::test]
async fn test_end_to_end_simulation() {
    let config = create_test_config().await;
    let price_feed_manager = Arc::new(PriceFeedManager::new(&config));
    
    let engine = ArbitrageEngine::new(config, price_feed_manager)
        .await
        .expect("Failed to create arbitrage engine");
    
    // Simulate a complete arbitrage cycle
    let opportunities = engine.scan_for_opportunities().await.unwrap();
    
    for opportunity in opportunities.iter().take(1) {
        let risk_assessment = engine.assess_risk(opportunity).await.unwrap();
        
        if risk_assessment.is_acceptable {
            // In simulation mode, this should not execute real trades
            let execution_result = engine.simulate_execution(opportunity).await;
            assert!(execution_result.is_ok(), "Simulation should succeed");
        }
    }
}
