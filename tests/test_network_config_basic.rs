//! NetworkConfig Basic Tests - Enterprise Enhanced
//! Tests básicos que validan funcionalidades core

#[cfg(test)]
mod tests {
    use sniperforge::config::network::{NetworkConfig, TokenInfo, ProgramIds};
    use std::str::FromStr;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_network_config_devnet_basic() {
        // Test que NetworkConfig::devnet() funciona (backward compatibility)
        let config = NetworkConfig::devnet().unwrap();
        
        assert_eq!(config.network, "devnet");
        assert!(config.rpc_endpoint.contains("devnet"));
        assert!(config.program_ids.system_program.to_string().len() > 0);
    }

    #[test]
    fn test_network_config_mainnet_basic() {
        let config = NetworkConfig::mainnet().unwrap();
        
        assert_eq!(config.network, "mainnet");
        assert!(config.rpc_endpoint.contains("mainnet"));
        assert!(config.program_ids.system_program.to_string().len() > 0);
    }

    #[test]
    fn test_network_config_testnet_basic() {
        let config = NetworkConfig::testnet().unwrap();
        
        assert_eq!(config.network, "testnet");
        assert!(config.rpc_endpoint.contains("testnet"));
    }

    #[test]
    fn test_token_info_creation() {
        let token = TokenInfo {
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            address: "So11111111111111111111111111111111111111112".to_string(),
            decimals: 9,
            verified: true,
            tradeable: true,
            liquidity_pools: vec!["jupiter".to_string(), "orca".to_string()],
            price_sources: vec!["jupiter".to_string(), "coinmarketcap".to_string()],
        };
        
        assert_eq!(token.symbol, "SOL");
        assert_eq!(token.decimals, 9);
        assert!(token.verified);
        assert!(token.tradeable);
    }

    #[test]
    fn test_program_ids_creation() {
        let program_ids = ProgramIds {
            system_program: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
            token_program: Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
            associated_token_program: Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap(),
            compute_budget_program: Pubkey::from_str("ComputeBudget111111111111111111111111111111").unwrap(),
            jupiter_program: Some(Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").unwrap()),
            orca_whirlpool_program: Some(Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").unwrap()),
            raydium_amm_program: Some(Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap()),
            spl_token_swap_program: None,
        };
        
        assert!(program_ids.jupiter_program.is_some());
        assert!(program_ids.orca_whirlpool_program.is_some());
        assert!(program_ids.raydium_amm_program.is_some());
    }

    #[test]
    fn test_network_config_dex_detection_basic() {
        let config = NetworkConfig::devnet().unwrap();
        
        // Test basic DEX availability detection using enhanced methods
        assert!(config.has_jupiter(), "Jupiter should be available on DevNet");
        assert!(config.has_orca(), "Orca should be available on DevNet");
        
        // Test available_dexes method
        let available_dexes = config.available_dexes();
        assert!(!available_dexes.is_empty(), "Should have at least one DEX available");
    }
}
