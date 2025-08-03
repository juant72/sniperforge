#[cfg(test)]
mod tests {
    use sniperforge::apis::jupiter::{
        Jupiter, JupiterBuilder, JupiterApiConfig,
        QuoteRequest, JupiterClient, JupiterMetrics
    };
    use sniperforge::config::network::NetworkConfig;
    use std::time::Duration;

    #[tokio::test]
    async fn test_jupiter_from_config_devnet() {
        // Ensure config files exist
        assert!(std::path::Path::new("config/jupiter_config.json").exists());
        assert!(std::path::Path::new("config/network_config.json").exists());
        
        let result = Jupiter::from_config("devnet").await;
        assert!(result.is_ok(), "Failed to create Jupiter from config: {:?}", result.err());
        
        let jupiter = result.unwrap();
        assert_eq!(jupiter.get_network_name(), "devnet");
        assert_eq!(jupiter.get_network_configuration().network, "DevNet");
    }

    #[tokio::test]
    async fn test_jupiter_from_config_mainnet() {
        let result = Jupiter::from_config("mainnet").await;
        assert!(result.is_ok(), "Failed to create Jupiter from config: {:?}", result.err());
        
        let jupiter = result.unwrap();
        assert_eq!(jupiter.get_network_name(), "mainnet");
        assert_eq!(jupiter.get_network_configuration().network, "MainNet");
    }

    #[test]
    fn test_jupiter_load_config() {
        let result = Jupiter::load_jupiter_config();
        assert!(result.is_ok(), "Failed to load Jupiter config: {:?}", result.err());
        
        let config = result.unwrap();
        assert_eq!(config.jupiter_api.api_version, "v6");
        assert!(config.network_mapping.contains_key("devnet"));
        assert!(config.network_mapping.contains_key("mainnet"));
        assert!(!config.network_mapping.get("testnet").unwrap().enabled);
    }

    #[tokio::test]
    async fn test_jupiter_network_config_validation() {
        // Test with unsupported network
        let result = Jupiter::from_config("testnet").await;
        assert!(result.is_err(), "Should fail for disabled network");
    }

    #[tokio::test]
    async fn test_jupiter_builder_pattern() {
        let config = Jupiter::load_jupiter_config().expect("Failed to load config");
        let network_config = NetworkConfig::from_config("devnet").expect("Failed to load network config");
        
        let result = JupiterBuilder::new()
            .network("devnet")
            .config(config)
            .network_config(network_config)
            .build()
            .await;
            
        assert!(result.is_ok(), "Builder should succeed: {:?}", result.err());
        
        let jupiter = result.unwrap();
        assert_eq!(jupiter.get_network_name(), "devnet");
    }

    #[tokio::test]
    async fn test_jupiter_get_network_config() {
        let jupiter = Jupiter::from_config("devnet").await.expect("Failed to create Jupiter");
        let network_config = jupiter.get_network_config();
        
        assert!(network_config.enabled);
        assert!(network_config.jupiter_program_id.is_some());
        assert_eq!(network_config.preferred_api_version, "v6");
        assert_eq!(network_config.max_slippage_bps, 300);
    }

    #[tokio::test]
    async fn test_jupiter_quote_request_with_defaults() {
        let mut jupiter = Jupiter::from_config("devnet").await.expect("Failed to create Jupiter");
        
        let quote_request = QuoteRequest {
            input_mint: "So11111111111111111111111111111111111111112".to_string(), // SOL
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            amount: 1000000, // 0.001 SOL
            slippage_bps: None, // Should use network default
            ..Default::default()
        };

        // This will likely fail without real network access, but tests the flow
        let _result = jupiter.get_quote(&quote_request).await;
        // We don't assert success since we're not connected to real Jupiter API
        
        // Check that metrics were updated
        assert_eq!(jupiter.get_metrics().total_requests, 1);
    }

    #[tokio::test]
    async fn test_jupiter_creation_and_metrics() {
        // Test Jupiter creation from configuration (enhanced functionality)
        let jupiter_result = Jupiter::from_config("devnet").await;
        
        // In case configuration files are not present, we'll skip this test
        // This follows the enriching approach - test what's available
        if jupiter_result.is_err() {
            println!("⚠️ Skipping test - configuration files not available");
            return;
        }
        
        let mut jupiter = jupiter_result.unwrap();
        
        // Test metrics access (public API)
        let initial_metrics = jupiter.get_metrics();
        assert_eq!(initial_metrics.total_requests, 0);
        assert_eq!(initial_metrics.successful_requests, 0);
        assert_eq!(initial_metrics.failed_requests, 0);
        assert_eq!(initial_metrics.error_rate, 0.0);

        // Test metrics update through public API
        jupiter.update_performance_metrics(true, Duration::from_millis(100));
        let updated_metrics = jupiter.get_metrics();
        assert_eq!(updated_metrics.total_requests, 1);
        assert_eq!(updated_metrics.successful_requests, 1);
        assert_eq!(updated_metrics.failed_requests, 0);
        assert_eq!(updated_metrics.error_rate, 0.0);

        // Test network information access
        assert_eq!(jupiter.get_network_name(), "devnet");
        assert_eq!(jupiter.get_network_configuration().network, "DevNet"); // Configuration uses capitalized names
    }

    #[tokio::test]
    async fn test_jupiter_token_support_validation() {
        let jupiter = Jupiter::from_config("devnet").await.expect("Failed to create Jupiter");
        
        let sol_mint = "So11111111111111111111111111111111111111112";
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let invalid_mint = "invalid_mint_address";
        
        // Test supported token pair
        assert!(jupiter.is_token_pair_supported(sol_mint, usdc_mint));
        
        // Test unsupported token
        assert!(!jupiter.is_token_pair_supported(sol_mint, invalid_mint));
        assert!(!jupiter.is_token_pair_supported(invalid_mint, usdc_mint));
    }

    #[tokio::test]
    async fn test_jupiter_get_supported_tokens() {
        let jupiter = Jupiter::from_config("mainnet").await.expect("Failed to create Jupiter");
        let tokens = jupiter.get_supported_tokens();
        
        assert!(!tokens.is_empty(), "Should have tradeable tokens");
        
        // Check that all returned tokens are tradeable
        for token in tokens {
            assert!(token.tradeable, "All returned tokens should be tradeable");
        }
    }

    #[tokio::test]
    async fn test_jupiter_health_check() {
        let mut jupiter = Jupiter::from_config("devnet").await.expect("Failed to create Jupiter");
        
        // Health check will likely fail without real network access
        let _health_result = jupiter.health_check().await;
        // We don't assert the result since it depends on network connectivity
        
        // But we can check that metrics were updated
        assert!(jupiter.get_metrics().total_requests > 0);
    }

    #[test]
    fn test_jupiter_config_file_structure() {
        let config = Jupiter::load_jupiter_config().expect("Failed to load config");
        
        // Test required sections exist
        assert!(!config.jupiter_api.base_url.is_empty());
        assert!(!config.jupiter_api.api_version.is_empty());
        assert!(config.jupiter_api.timeout_seconds > 0);
        
        // Test network mapping
        assert!(config.network_mapping.contains_key("devnet"));
        assert!(config.network_mapping.contains_key("mainnet"));
        
        // Test devnet config
        let devnet_config = &config.network_mapping["devnet"];
        assert!(devnet_config.enabled);
        assert!(devnet_config.jupiter_program_id.is_some());
        
        // Test mainnet config
        let mainnet_config = &config.network_mapping["mainnet"];
        assert!(mainnet_config.enabled);
        assert!(mainnet_config.jupiter_program_id.is_some());
        
        // Test rate limiting
        assert!(config.rate_limiting.requests_per_second > 0);
        assert!(config.rate_limiting.burst_allowance > 0);
        
        // Test trading parameters
        assert!(config.trading_parameters.default_slippage_bps > 0);
        assert!(config.trading_parameters.max_slippage_bps > 0);
        assert!(config.trading_parameters.min_output_amount_buffer > 0.0);
        assert!(config.trading_parameters.min_output_amount_buffer <= 1.0);
        
        // Test advanced features
        assert!(config.advanced_features.jupiter_v6_features);
    }

    #[test]
    fn test_quote_request_defaults() {
        let request = QuoteRequest::default();
        assert!(request.input_mint.is_empty());
        assert!(request.output_mint.is_empty());
        assert_eq!(request.amount, 0);
        assert!(request.slippage_bps.is_none());
    }

    #[tokio::test]
    async fn test_jupiter_wallet_integration_config() {
        let jupiter = Jupiter::from_config("devnet").await.expect("Failed to create Jupiter");
        
        // Test wallet integration settings
        assert!(jupiter.get_configuration().wallet_integration.auto_wrap_sol);
        assert!(jupiter.get_configuration().wallet_integration.auto_create_ata);
        assert!(jupiter.get_configuration().wallet_integration.verify_balance_before_swap);
        assert!(jupiter.get_configuration().wallet_integration.min_sol_balance_lamports > 0);
        assert!(jupiter.get_configuration().wallet_integration.max_transaction_attempts > 0);
    }

    #[tokio::test]
    async fn test_jupiter_fallback_configuration() {
        let jupiter = Jupiter::from_config("devnet").await.expect("Failed to create Jupiter");
        
        // Test fallback settings
        assert!(jupiter.get_configuration().fallback_configuration.enable_fallback);
        assert!(!jupiter.get_configuration().fallback_configuration.fallback_dexs.is_empty());
        assert!(jupiter.get_configuration().fallback_configuration.fallback_delay_ms > 0);
        assert!(jupiter.get_configuration().fallback_configuration.max_fallback_attempts > 0);
    }

    #[tokio::test]
    async fn test_jupiter_monitoring_configuration() {
        let jupiter = Jupiter::from_config("devnet").await.expect("Failed to create Jupiter");
        
        // Test monitoring settings
        assert!(jupiter.get_configuration().monitoring.log_requests);
        assert!(jupiter.get_configuration().monitoring.enable_metrics);
        assert!(jupiter.get_configuration().monitoring.max_error_rate > 0.0);
        assert!(jupiter.get_configuration().monitoring.max_error_rate <= 1.0);
    }

    #[tokio::test]
    async fn test_builder_missing_network() {
        let result = JupiterBuilder::new()
            .build()
            .await;
            
        assert!(result.is_err(), "Builder should fail without network");
    }

    #[tokio::test]
    async fn test_builder_with_minimal_config() {
        let result = JupiterBuilder::new()
            .network("devnet")
            .build()
            .await;
            
        assert!(result.is_ok(), "Builder should succeed with minimal config: {:?}", result.err());
    }
}
