//! Network Configuration for SniperForge Enterprise
//! 
//! Provides parametrized configuration for different Solana networks
//! Avoids hardcoding and enables real swaps across different networks

use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub network: String,
    pub rpc_endpoint: String,
    pub program_ids: ProgramIds,
    pub token_addresses: HashMap<String, TokenInfo>,
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
        
        // Core tokens for DevNet
        token_addresses.insert(
            "SOL".to_string(),
            TokenInfo {
                address: "So11111111111111111111111111111111111111112".to_string(),
                symbol: "SOL".to_string(),
                name: "Wrapped SOL".to_string(),
                decimals: 9,
                verified: true,
                tradeable: true,
            },
        );
        
        token_addresses.insert(
            "USDC".to_string(),
            TokenInfo {
                address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                symbol: "USDC".to_string(),
                name: "USD Coin".to_string(),
                decimals: 6,
                verified: true,
                tradeable: true,
            },
        );

        Self {
            network: "DevNet".to_string(),
            rpc_endpoint: "https://api.devnet.solana.com".to_string(),
            program_ids: ProgramIds {
                system_program: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                token_program: Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
                associated_token_program: Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap(),
                compute_budget_program: Pubkey::from_str("ComputeBudget111111111111111111111111111111").unwrap(),
                jupiter_program: Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").ok(),
                orca_whirlpool_program: Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").ok(),
                raydium_amm_program: None,
                spl_token_swap_program: None,
            },
            token_addresses,
        }
    }

    /// Get MainNet configuration
    pub fn mainnet() -> Self {
        let mut token_addresses = HashMap::new();
        
        // Core tokens for MainNet
        token_addresses.insert(
            "SOL".to_string(),
            TokenInfo {
                address: "So11111111111111111111111111111111111111112".to_string(),
                symbol: "SOL".to_string(),
                name: "Wrapped SOL".to_string(),
                decimals: 9,
                verified: true,
                tradeable: true,
            },
        );

        Self {
            network: "MainNet".to_string(),
            rpc_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            program_ids: ProgramIds {
                system_program: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                token_program: Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
                associated_token_program: Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap(),
                compute_budget_program: Pubkey::from_str("ComputeBudget111111111111111111111111111111").unwrap(),
                jupiter_program: Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").ok(),
                orca_whirlpool_program: Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").ok(),
                raydium_amm_program: None,
                spl_token_swap_program: None,
            },
            token_addresses,
        }
    }
}
