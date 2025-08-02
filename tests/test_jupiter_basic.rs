//! Jupiter API Basic Tests - Enterprise Enhanced
//! Tests básicos que validan funcionalidades core sin dependencias complejas

#[cfg(test)]
mod tests {
    use sniperforge::apis::jupiter::{
        JupiterApiConfig, JupiterClient, QuoteRequest
    };

    #[test]
    fn test_jupiter_client_creation() {
        let config = JupiterApiConfig::devnet();
        let result = JupiterClient::new(config);
        assert!(result.is_ok(), "Failed to create Jupiter client");
    }

    #[test]
    fn test_quote_request_creation() {
        let quote_request = QuoteRequest::new(
            "So11111111111111111111111111111111111111112".to_string(), // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            1000000 // 1 SOL in lamports
        );
        
        assert_eq!(quote_request.input_mint, "So11111111111111111111111111111111111111112");
        assert_eq!(quote_request.output_mint, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
        assert_eq!(quote_request.amount, 1000000);
        assert_eq!(quote_request.slippage_bps, Some(50)); // Default 0.5%
    }

    #[test]
    fn test_quote_request_with_user_key() {
        let quote_request = QuoteRequest::new(
            "So11111111111111111111111111111111111111112".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1000000
        ).with_user_public_key("11111111111111111111111111111111".to_string());
        
        assert!(quote_request.user_public_key.is_some());
        assert_eq!(quote_request.user_public_key.unwrap(), "11111111111111111111111111111111");
    }

    #[test]
    fn test_quote_request_with_slippage() {
        let quote_request = QuoteRequest::new(
            "So11111111111111111111111111111111111111112".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1000000
        ).with_slippage_bps(100); // 1%
        
        assert_eq!(quote_request.slippage_bps, Some(100));
    }

    #[test]
    fn test_jupiter_api_config_devnet() {
        let config = JupiterApiConfig::devnet();
        
        assert!(config.enabled);
        assert_eq!(config.base_url, "https://quote-api.jup.ag");
        assert_eq!(config.network_name, "devnet");
        assert_eq!(config.rpc_endpoint, "https://api.devnet.solana.com");
        assert!(config.timeout_seconds > 0);
        assert!(config.max_retries > 0);
    }

    #[test]
    fn test_jupiter_api_config_mainnet() {
        let config = JupiterApiConfig::mainnet();
        
        assert!(config.enabled);
        assert_eq!(config.base_url, "https://quote-api.jup.ag");
        assert_eq!(config.network_name, "mainnet");
        assert_eq!(config.rpc_endpoint, "https://api.mainnet-beta.solana.com");
    }
}
