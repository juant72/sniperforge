#[cfg(test)]
mod tests {
    use sniperforge::config::network::{
        NetworkConfig, NetworkConfigBuilder, TokenInfo, ProgramIds,
        DexConfiguration, ValidationRules
    };
    use std::fs;

    #[test]
    fn test_network_config_from_config_devnet() {
        // Ensure config files exist
        assert!(std::path::Path::new("config/network_config.json").exists());
        assert!(std::path::Path::new("data/tokens/devnet_verified_tokens.json").exists());
        
        let config = NetworkConfig::from_config("devnet").expect("Failed to load devnet config");
        
        assert_eq!(config.network, "DevNet");
        assert_eq!(config.rpc_endpoint, "https://api.devnet.solana.com");
        assert!(config.token_addresses.contains_key("SOL"));
        assert!(config.token_addresses.contains_key("USDC"));
        assert_eq!(config.dex_configuration.preferred_dexs, vec!["jupiter", "orca"]);
        assert!(config.dex_configuration.dex_detection_enabled);
    }

    #[test]
    fn test_network_config_from_config_mainnet() {
        let config = NetworkConfig::from_config("mainnet").expect("Failed to load mainnet config");
        
        assert_eq!(config.network, "MainNet");
        assert_eq!(config.rpc_endpoint, "https://api.mainnet-beta.solana.com");
        assert!(config.token_addresses.contains_key("SOL"));
        assert!(config.token_addresses.contains_key("USDC"));
        assert_eq!(config.dex_configuration.preferred_dexs, vec!["jupiter", "raydium", "orca"]);
        assert!(config.validation_rules.require_verified_tokens);
    }

    #[test]
    fn test_dex_availability() {
        let config = NetworkConfig::from_config("devnet").expect("Failed to load devnet config");
        
        assert!(config.is_dex_available("jupiter"));
        assert!(config.is_dex_available("orca"));
        assert!(!config.is_dex_available("raydium")); // Not available in devnet
    }

    #[test]
    fn test_preferred_dexs() {
        let config = NetworkConfig::from_config("devnet").expect("Failed to load devnet config");
        let preferred = config.get_preferred_dexs();
        
        assert!(preferred.contains(&"jupiter".to_string()));
        assert!(preferred.contains(&"orca".to_string()));
        assert!(!preferred.contains(&"raydium".to_string())); // Not available
    }

    #[test]
    fn test_token_validation() {
        let config = NetworkConfig::from_config("devnet").expect("Failed to load devnet config");
        
        assert!(config.validate_token("SOL").expect("Validation failed"));
        assert!(config.validate_token("USDC").expect("Validation failed"));
        
        // Test non-existent token
        assert!(config.validate_token("NONEXISTENT").is_err());
    }

    #[test]
    fn test_get_token_info() {
        let config = NetworkConfig::from_config("mainnet").expect("Failed to load mainnet config");
        
        let sol_info = config.get_token_info("SOL").expect("SOL not found");
        assert_eq!(sol_info.symbol, "SOL");
        assert_eq!(sol_info.decimals, 9);
        assert!(sol_info.verified);
        assert!(sol_info.tradeable);
    }

    #[test]
    fn test_get_verified_tokens() {
        let config = NetworkConfig::from_config("mainnet").expect("Failed to load mainnet config");
        let verified = config.get_verified_tokens();
        
        assert!(!verified.is_empty());
        for token in verified {
            assert!(token.verified);
        }
    }

    #[test]
    fn test_get_tradeable_tokens() {
        let config = NetworkConfig::from_config("devnet").expect("Failed to load devnet config");
        let tradeable = config.get_tradeable_tokens();
        
        assert!(!tradeable.is_empty());
        for token in tradeable {
            assert!(token.tradeable);
        }
    }

    #[test]
    fn test_builder_pattern() {
        let program_ids = ProgramIds {
            system_program: solana_sdk::pubkey::Pubkey::from_str("11111111111111111111111111111111").unwrap(),
            token_program: solana_sdk::pubkey::Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
            associated_token_program: solana_sdk::pubkey::Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap(),
            compute_budget_program: solana_sdk::pubkey::Pubkey::from_str("ComputeBudget111111111111111111111111111111").unwrap(),
            jupiter_program: None,
            orca_whirlpool_program: None,
            raydium_amm_program: None,
            spl_token_swap_program: None,
        };

        let token_info = TokenInfo {
            address: "So11111111111111111111111111111111111111112".to_string(),
            symbol: "SOL".to_string(),
            name: "Wrapped SOL".to_string(),
            decimals: 9,
            verified: true,
            tradeable: true,
            liquidity_pools: vec!["jupiter".to_string()],
            price_sources: vec!["jupiter".to_string()],
        };

        let config = NetworkConfig::builder()
            .network("CustomNet".to_string())
            .rpc_endpoint("https://custom.endpoint.com".to_string())
            .program_ids(program_ids)
            .add_token("SOL".to_string(), token_info)
            .build()
            .expect("Failed to build config");

        assert_eq!(config.network, "CustomNet");
        assert_eq!(config.rpc_endpoint, "https://custom.endpoint.com");
        assert!(config.token_addresses.contains_key("SOL"));
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that old methods still work
        let devnet_config = NetworkConfig::devnet().expect("Failed to create devnet config");
        assert_eq!(devnet_config.network, "DevNet");

        let mainnet_config = NetworkConfig::mainnet().expect("Failed to create mainnet config");
        assert_eq!(mainnet_config.network, "MainNet");
    }

    #[test]
    fn test_testnet_config() {
        let config = NetworkConfig::testnet().expect("Failed to load testnet config");
        
        assert_eq!(config.network, "TestNet");
        assert_eq!(config.rpc_endpoint, "https://api.testnet.solana.com");
        assert!(!config.dex_configuration.dex_detection_enabled);
        assert!(!config.validation_rules.require_verified_tokens);
    }

    #[test]
    fn test_config_file_missing_network() {
        let result = NetworkConfig::from_config("nonexistent_network");
        assert!(result.is_err());
    }
}
