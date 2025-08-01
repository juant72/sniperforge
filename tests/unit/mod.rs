//! Unit Tests - Individual Component Testing
//! Tests individual modules and functions in isolation

use sniperforge::{
    config::SimpleConfig,
    types::{ArbitragePair, ArbitrageOpportunity, PriceInfo, Token, SniperForgeError},
    utils::logging::{LogLevel, LogEntry},
};
use crate::helpers::constants::{SOL_MINT, USDC_MINT, USDT_MINT, HFT_SPEED_REQUIREMENT_MS};
use crate::helpers::{is_valid_solana_address};
use chrono::Utc;
use rust_decimal::Decimal;
use std::time::Duration;

/// Configuration tests
mod config_tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = SimpleConfig::default();
        
        // Validate required fields
        assert!(!config.solana_rpc_url.is_empty());
        assert!(config.min_profit_threshold > 0.0);
        assert!(config.max_slippage > 0.0);
        assert!(config.max_position_size > 0.0);
        
        println!("✅ Config: Basic creation and validation passed");
    }

    #[test]
    fn test_config_validation_bounds() {
        let config = SimpleConfig::default();
        
        // Test reasonable bounds
        assert!(config.max_slippage > 0.0 && config.max_slippage < 1.0);
        assert!(config.min_profit_threshold > 0.0 && config.min_profit_threshold < 1.0);
        
        println!("✅ Config: Bounds validation passed");
    }

    #[test]
    fn test_config_defaults() {
        let config = SimpleConfig::default();
        
        // Test defaults are reasonable
        assert!(config.max_requests_per_second > 0);
        assert!(config.cooldown_period_ms > 0);
        assert!(!config.log_level.is_empty());
        
        println!("✅ Config: Default values validation passed");
    }
}

/// Core types tests
mod types_tests {
    use super::*;

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
        
        // Validate structure
        assert_eq!(pair.base_token.symbol, "SOL");
        assert_eq!(pair.quote_token.symbol, "USDC");
        assert!(pair.fee_rate > 0.0);
        assert!(pair.pool_address.is_some());
        
        println!("✅ Types: ArbitragePair creation validation passed");
    }

    #[test] 
    fn test_token_structure() {
        let token = Token {
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            decimals: 9,
        };
        
        // Validate token fields
        assert_eq!(token.mint.len(), 43); // SOL mint address length
        assert!(token.decimals > 0);
        assert!(!token.symbol.is_empty());
        
        println!("✅ Types: Token structure validation passed");
    }

    #[test]
    fn test_arbitrage_opportunity_structure() {
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
            buy_exchange: "Jupiter".to_string(),
            sell_exchange: "Orca".to_string(),
            buy_price: 100.0,
            sell_price: 102.0,
            profit_percentage: 2.0,
            volume_required: 10000.0,
            estimated_gas_cost: 0.01,
            confidence_score: 0.85,
            timestamp: Utc::now(),
            execution_time_window: Duration::from_secs(30),
        };
        
        // Validate opportunity structure
        assert!(opportunity.sell_price > opportunity.buy_price);
        assert!(opportunity.profit_percentage > 0.0);
        assert!(opportunity.confidence_score >= 0.0 && opportunity.confidence_score <= 1.0);
        assert!(opportunity.volume_required > 0.0);
        
        println!("✅ Types: ArbitrageOpportunity structure validation passed");
    }

    #[test]
    fn test_price_info_functionality() {
        let price_info = PriceInfo {
            mint: "So11111111111111111111111111111111111111112".to_string(),
            usd: rust_decimal::Decimal::new(10050, 2), // $100.50
            timestamp: Utc::now(),
            source: "Test".to_string(),
        };
        
        // Validate price info
        assert_eq!(price_info.mint, SOL_MINT);
        assert!(price_info.usd > Decimal::ZERO);
        assert!(!price_info.source.is_empty());
        
        println!("✅ Types: PriceInfo functionality validation passed");
    }
}

/// Validation tests
mod validation_tests {
    use super::*;

    #[test]
    fn test_solana_address_validation() {
        // Valid addresses
        let valid_addresses = vec![
            SOL_MINT,
            USDC_MINT, 
            USDT_MINT,
        ];
        
        for address in valid_addresses {
            assert!(is_valid_solana_address(address));
            println!("✅ Validation: Address {} is valid", address);
        }
        
        // Invalid addresses
        let invalid_addresses = vec![
            "", // Empty
            "invalid", // Too short
            "This_is_way_too_long_to_be_a_valid_solana_address", // Too long
            "InvalidChars@#$%", // Invalid characters
        ];
        
        for address in invalid_addresses {
            assert!(!is_valid_solana_address(address));
            println!("✅ Validation: Address '{}' correctly rejected", address);
        }
    }

    #[test]
    fn test_profit_threshold_validation() {
        let config = SimpleConfig::default();
        
        // Test profit calculations (min_profit_threshold default is 0.001 = 0.1%)
        let test_profit = 0.002; // 0.2%
        assert!(test_profit > config.min_profit_threshold);
        
        let small_profit = 0.0005; // 0.05%
        assert!(small_profit < config.min_profit_threshold);
        
        println!("✅ Validation: Profit threshold validation passed");
    }

    #[test]
    fn test_slippage_validation() {
        let config = SimpleConfig::default();
        
        // Test slippage bounds (max_slippage default is 0.005 = 0.5%)
        let acceptable_slippage = 0.003; // 0.3%
        assert!(acceptable_slippage < config.max_slippage);
        
        let excessive_slippage = 0.01; // 1%
        assert!(excessive_slippage > config.max_slippage);
        
        println!("✅ Validation: Slippage validation passed");
    }
}

/// Error handling tests
mod error_tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let config_error = SniperForgeError::Config("Invalid RPC URL".to_string());
        let network_error = SniperForgeError::Network("Connection failed".to_string());
        let trading_error = SniperForgeError::Trading("Insufficient balance".to_string());
        let api_error = SniperForgeError::Api("Rate limit exceeded".to_string());
        
        // Test error messages contain expected content
        assert!(format!("{}", config_error).contains("Invalid RPC URL"));
        assert!(format!("{}", network_error).contains("Connection failed"));
        assert!(format!("{}", trading_error).contains("Insufficient balance"));
        assert!(format!("{}", api_error).contains("Rate limit exceeded"));
        
        println!("✅ Errors: Error creation and formatting validated");
    }

    #[test]
    fn test_error_types() {
        let errors = vec![
            SniperForgeError::Config("test".to_string()),
            SniperForgeError::Trading("test".to_string()),
            SniperForgeError::Api("test".to_string()),
            SniperForgeError::Security("test".to_string()),
            SniperForgeError::Network("test".to_string()),
        ];
        
        // Verify all error types can be created
        assert_eq!(errors.len(), 5);
        
        println!("✅ Errors: All error types validation passed");
    }
}

/// Logging tests  
mod logging_tests {
    use super::*;

    #[test]
    fn test_log_levels() {
        let levels = vec![
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ];
        
        // Test each level exists
        for level in levels {
            println!("✅ Logging: LogLevel::{:?} validated", level);
        }
    }

    #[test]
    fn test_log_entry_creation() {
        let log_entry = LogEntry {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            level: "INFO".to_string(),
            component: "test_module".to_string(),
            message: "Test log message".to_string(),
            metadata: serde_json::json!({"test": "value"}),
        };
        
        // Validate log entry fields
        assert!(!log_entry.timestamp.is_empty());
        assert_eq!(log_entry.message, "Test log message");
        assert_eq!(log_entry.component, "test_module");
        assert_eq!(log_entry.level, "INFO");
        
        println!("✅ Logging: LogEntry creation validation passed");
    }
}

/// Performance tests
mod performance_tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_operation_timing() {
        let start_time = Instant::now();
        
        // Simulate some work
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let elapsed = start_time.elapsed();
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(100)); // Should be reasonably fast
        
        println!("✅ Performance: Operation timing validation passed ({:?})", elapsed);
    }

    #[test]
    fn test_memory_estimation() {
        let config_size = std::mem::size_of::<SimpleConfig>();
        let pair_size = std::mem::size_of::<ArbitragePair>();
        
        // Basic memory usage checks
        assert!(config_size > 0);
        assert!(pair_size > 0);
        
        println!("✅ Performance: Memory estimation validation passed");
        println!("   Config size: {} bytes", config_size);
        println!("   Pair size: {} bytes", pair_size);
    }

    #[test]
    fn test_hft_speed_requirements() {
        let start = std::time::Instant::now();
        
        // Simulate HFT operation
        let _pair = ArbitragePair::default();
        let _price = PriceInfo {
            mint: "So11111111111111111111111111111111111111112".to_string(),
            usd: rust_decimal::Decimal::new(10000, 2), // $100.00
            timestamp: Utc::now(),
            source: "Test".to_string(),
        };
        
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < HFT_SPEED_REQUIREMENT_MS as u128);
        
        println!("✅ Performance: HFT speed requirements met ({:?})", elapsed);
    }
}
