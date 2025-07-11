// Configuración de Program IDs parametrizada por red
// Evita hardcoding y permite swaps reales en diferentes redes

use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
// use crate::arbitrage::types::ArbitrageSettings; // Comentado temporalmente

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub network: String,
    pub rpc_endpoint: String,
    pub program_ids: ProgramIds,
    pub token_addresses: HashMap<String, TokenInfo>,
    // pub arbitrage_settings: Option<ArbitrageSettings>, // Comentado temporalmente
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub verified: bool,
    pub tradeable: bool,
}

#[derive(Debug, Clone)]
pub struct ProgramIds {
    pub system_program: Pubkey,
    pub token_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub compute_budget_program: Pubkey,
    pub jupiter_program: Option<Pubkey>,
    pub orca_whirlpool_program: Option<Pubkey>,
    pub raydium_amm_program: Option<Pubkey>,
    pub spl_token_swap_program: Option<Pubkey>,
}

impl NetworkConfig {
    /// Get DevNet configuration
    pub fn devnet() -> Self {
        let mut token_addresses = HashMap::new();
        token_addresses.insert("SOL".to_string(), TokenInfo {
            address: "So11111111111111111111111111111111111111112".to_string(),
            symbol: "SOL".to_string(),
            name: "Wrapped SOL".to_string(),
            decimals: 9,
            verified: true,
            tradeable: true,
        });
        token_addresses.insert("USDC".to_string(), TokenInfo {
            address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
            verified: true,
            tradeable: true,
        });
        token_addresses.insert("RAY".to_string(), TokenInfo {
            address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: "RAY".to_string(),
            name: "Raydium".to_string(),
            decimals: 6,
            verified: true,
            tradeable: true,
        });
        
        Self {
            network: "DevNet".to_string(),
            rpc_endpoint: "https://api.devnet.solana.com".to_string(),
            program_ids: ProgramIds {
                system_program: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                token_program: Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
                associated_token_program: Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap(),
                compute_budget_program: Pubkey::from_str("ComputeBudget111111111111111111111111111111").unwrap(),
                // Jupiter usa el mismo Program ID en DevNet y MainNet
                jupiter_program: Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").ok(),
                // Orca Whirlpool en DevNet
                orca_whirlpool_program: Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").ok(),
                // Raydium en DevNet - puede no estar disponible
                raydium_amm_program: None,
                // SPL Token Swap básico
                spl_token_swap_program: Pubkey::from_str("SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8").ok(),
            },
            token_addresses,
            // arbitrage_settings: None, // Comentado temporalmente
        }
    }

    /// Get MainNet configuration
    pub fn mainnet() -> Self {
        let mut token_addresses = HashMap::new();
        token_addresses.insert("SOL".to_string(), TokenInfo {
            address: "So11111111111111111111111111111111111111112".to_string(),
            symbol: "SOL".to_string(),
            name: "Wrapped SOL".to_string(),
            decimals: 9,
            verified: true,
            tradeable: true,
        });
        token_addresses.insert("USDC".to_string(), TokenInfo {
            address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
            verified: true,
            tradeable: true,
        });
        token_addresses.insert("RAY".to_string(), TokenInfo {
            address: "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(),
            symbol: "RAY".to_string(),
            name: "Raydium".to_string(),
            decimals: 6,
            verified: true,
            tradeable: true,
        });
        token_addresses.insert("BONK".to_string(), TokenInfo {
            address: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
            symbol: "BONK".to_string(),
            name: "Bonk".to_string(),
            decimals: 5,
            verified: true,
            tradeable: true,
        });
        
        Self {
            network: "MainNet".to_string(),
            rpc_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            program_ids: ProgramIds {
                system_program: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                token_program: Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
                associated_token_program: Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap(),
                compute_budget_program: Pubkey::from_str("ComputeBudget111111111111111111111111111111").unwrap(),
                // Jupiter V6 en MainNet
                jupiter_program: Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").ok(),
                // Orca Whirlpool en MainNet
                orca_whirlpool_program: Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").ok(),
                // Raydium AMM en MainNet
                raydium_amm_program: Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").ok(),
                // SPL Token Swap
                spl_token_swap_program: Pubkey::from_str("SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8").ok(),
            },
            token_addresses,
            // arbitrage_settings: None, // Comentado temporalmente
        }
    }

    /// Get testnet configuration
    pub fn testnet() -> Self {
        let mut config = Self::devnet();
        config.network = "TestNet".to_string();
        config.rpc_endpoint = "https://api.testnet.solana.com".to_string();
        config
    }

    /// Get configuration by name
    pub fn by_name(network: &str) -> Result<Self, String> {
        match network.to_lowercase().as_str() {
            "devnet" | "dev" => Ok(Self::devnet()),
            "mainnet" | "main" | "mainnet-beta" => Ok(Self::mainnet()),
            "testnet" | "test" => Ok(Self::testnet()),
            _ => Err(format!("Unsupported network: {}", network)),
        }
    }

    /// Check if Jupiter is available for this network
    pub fn has_jupiter(&self) -> bool {
        self.program_ids.jupiter_program.is_some()
    }

    /// Check if Orca is available for this network
    pub fn has_orca(&self) -> bool {
        self.program_ids.orca_whirlpool_program.is_some()
    }

    /// Check if Raydium is available for this network
    pub fn has_raydium(&self) -> bool {
        self.program_ids.raydium_amm_program.is_some()
    }

    /// Get available DEXes for this network
    pub fn available_dexes(&self) -> Vec<String> {
        let mut dexes = vec!["Native".to_string()]; // System program siempre disponible
        
        if self.has_jupiter() {
            dexes.push("Jupiter".to_string());
        }
        if self.has_orca() {
            dexes.push("Orca".to_string());
        }
        if self.has_raydium() {
            dexes.push("Raydium".to_string());
        }
        if self.program_ids.spl_token_swap_program.is_some() {
            dexes.push("SPL-Swap".to_string());
        }
        
        dexes
    }

    /// Get safe test token pair for this network
    pub fn get_test_token_pair(&self) -> (Pubkey, Option<Pubkey>) {
        (self.token_addresses["SOL"].address.parse().unwrap(), self.token_addresses["USDC"].address.parse().ok())
    }

    /// Validate network configuration
    pub fn validate(&self) -> Result<(), String> {
        // Verificar que los programas básicos estén configurados
        if self.program_ids.system_program.to_string() == "11111111111111111111111111111111" &&
           self.program_ids.token_program.to_string().starts_with("Token") {
            Ok(())
        } else {
            Err("Invalid basic program configuration".to_string())
        }
    }
}

/// Helper para crear configuraciones personalizadas
#[derive(Debug, Clone)]
pub struct NetworkConfigBuilder {
    config: NetworkConfig,
}

impl NetworkConfigBuilder {
    pub fn new(name: &str, rpc_endpoint: &str) -> Self {
        Self {
            config: NetworkConfig {
                network: name.to_string(),
                rpc_endpoint: rpc_endpoint.to_string(),
                program_ids: ProgramIds {
                    system_program: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                    token_program: Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
                    associated_token_program: Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap(),
                    compute_budget_program: Pubkey::from_str("ComputeBudget11111111111111111111111111111111").unwrap(),
                    jupiter_program: None,
                    orca_whirlpool_program: None,
                    raydium_amm_program: None,
                    spl_token_swap_program: None,
                },
                token_addresses: HashMap::new(),
                // arbitrage_settings: None, // Comentado temporalmente
            },
        }
    }
    
    pub fn with_jupiter(mut self, program_id: &str) -> Result<Self, String> {
        self.config.program_ids.jupiter_program = Some(
            Pubkey::from_str(program_id).map_err(|e| format!("Invalid Jupiter program ID: {}", e))?
        );
        Ok(self)
    }
    
    pub fn with_orca(mut self, program_id: &str) -> Result<Self, String> {
        self.config.program_ids.orca_whirlpool_program = Some(
            Pubkey::from_str(program_id).map_err(|e| format!("Invalid Orca program ID: {}", e))?
        );
        Ok(self)
    }
    
    pub fn with_usdc(mut self, token_address: &str) -> Result<Self, String> {
        self.config.token_addresses.insert("USDC".to_string(), TokenInfo {
            address: token_address.to_string(),
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
            verified: true,
            tradeable: true,
        });
        Ok(self)
    }
    
    pub fn build(self) -> NetworkConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devnet_config() {
        let config = NetworkConfig::devnet();
        assert_eq!(config.network, "DevNet");
        assert!(config.validate().is_ok());
        assert!(config.available_dexes().contains(&"Native".to_string()));
    }

    #[test]
    fn test_mainnet_config() {
        let config = NetworkConfig::mainnet();
        assert_eq!(config.network, "MainNet");
        assert!(config.validate().is_ok());
        assert!(config.has_jupiter());
        assert!(config.has_orca());
    }

    #[test]
    fn test_config_builder() {
        let config = NetworkConfigBuilder::new("Custom", "https://custom.rpc")
            .with_jupiter("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4")
            .unwrap()
            .build();
        
        assert_eq!(config.network, "Custom");
        assert!(config.has_jupiter());
    }
}
