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
        let config = NetworkConfig::devnet();
        
        assert_eq!(config.network, "DevNet");
        assert_eq!(config.rpc_endpoint, "https://api.devnet.solana.com");
        assert!(config.program_ids.system_program.to_string().len() > 0);
    }

    #[test]
    fn test_network_config_mainnet_basic() {
        let config = NetworkConfig::mainnet();
        
        assert_eq!(config.network, "MainNet");
        assert_eq!(config.rpc_endpoint, "https://api.mainnet-beta.solana.com");
        assert!(config.program_ids.system_program.to_string().len() > 0);
    }

    #[test]
    fn test_network_config_testnet_basic() {
        let config = NetworkConfig::testnet();
        
        assert_eq!(config.network, "TestNet");
        assert_eq!(config.rpc_endpoint, "https://api.testnet.solana.com");
    }

    #[test]
    fn test_token_info_creation() {
        let token = TokenInfo {
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            address: "So11111111111111111111111111111111111111112".to_string(),
            decimals: 9,
            verified: true,
        };
        
        assert_eq!(token.symbol, "SOL");
        assert_eq!(token.decimals, 9);
        assert!(token.verified);
    }

    #[test]
    fn test_program_ids_creation() {
        let program_ids = ProgramIds {
            system_program: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
            token_program: Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
            associated_token_program: Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap(),
            compute_budget_program: Pubkey::from_str("ComputeBudget111111111111111111111111111111").unwrap(),
            jupiter_program: Some(Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").unwrap()),
            orca_program: Some(Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").unwrap()),
            raydium_program: Some(Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap()),
        };
        
        assert!(program_ids.jupiter_program.is_some());
        assert!(program_ids.orca_program.is_some());
        assert!(program_ids.raydium_program.is_some());
    }

    #[test]
    fn test_network_config_dex_detection_basic() {
        let config = NetworkConfig::devnet();
        
        // Test basic DEX availability detection
        let has_jupiter = config.program_ids.jupiter_program.is_some();
        let has_orca = config.program_ids.orca_program.is_some();
        
        assert!(has_jupiter, "Jupiter should be available on DevNet");
        assert!(has_orca, "Orca should be available on DevNet");
    }
}
