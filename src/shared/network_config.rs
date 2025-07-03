// Configuración de Program IDs parametrizada por red
// Evita hardcoding y permite swaps reales en diferentes redes

use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub name: String,
    pub rpc_endpoint: String,
    pub program_ids: ProgramIds,
    pub token_addresses: TokenAddresses,
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

#[derive(Debug, Clone)]
pub struct TokenAddresses {
    pub sol: Pubkey, // Wrapped SOL
    pub usdc: Option<Pubkey>,
    pub ray: Option<Pubkey>,
    pub bonk: Option<Pubkey>,
}

impl NetworkConfig {
    /// Get DevNet configuration
    pub fn devnet() -> Self {
        Self {
            name: "DevNet".to_string(),
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
            token_addresses: TokenAddresses {
                sol: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
                // USDC en DevNet - mismo que MainNet pero funciona en DevNet
                usdc: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").ok(),
                // RAY token - usar USDC para testing en DevNet
                ray: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").ok(), // USDC en lugar de RAY para DevNet
                // BONK en DevNet
                bonk: None,
            },
        }
    }

    /// Get MainNet configuration
    pub fn mainnet() -> Self {
        Self {
            name: "MainNet".to_string(),
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
            token_addresses: TokenAddresses {
                sol: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
                // USDC en MainNet
                usdc: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").ok(),
                // RAY token en MainNet
                ray: Pubkey::from_str("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R").ok(),
                // BONK en MainNet
                bonk: Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").ok(),
            },
        }
    }

    /// Get testnet configuration
    pub fn testnet() -> Self {
        let mut config = Self::devnet();
        config.name = "TestNet".to_string();
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
        (self.token_addresses.sol, self.token_addresses.usdc)
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
                name: name.to_string(),
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
                token_addresses: TokenAddresses {
                    sol: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
                    usdc: None,
                    ray: None,
                    bonk: None,
                },
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
        self.config.token_addresses.usdc = Some(
            Pubkey::from_str(token_address).map_err(|e| format!("Invalid USDC address: {}", e))?
        );
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
        assert_eq!(config.name, "DevNet");
        assert!(config.validate().is_ok());
        assert!(config.available_dexes().contains(&"Native".to_string()));
    }

    #[test]
    fn test_mainnet_config() {
        let config = NetworkConfig::mainnet();
        assert_eq!(config.name, "MainNet");
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
        
        assert_eq!(config.name, "Custom");
        assert!(config.has_jupiter());
    }
}
