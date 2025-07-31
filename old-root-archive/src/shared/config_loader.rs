// Configuración basada en archivos JSON
// Sistema completamente paramétrico sin hardcoding

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfigFile {
    pub network: String,
    pub display_name: String,
    pub rpc_endpoint: String,
    pub explorer_base_url: String,
    pub explorer_cluster_param: String,
    pub program_ids: ProgramIdsConfig,
    pub token_addresses: HashMap<String, TokenConfig>,
    pub trading_pairs: Vec<TradingPairConfig>,
    pub dex_config: DexConfigMap,
    pub safety_limits: SafetyLimitsConfig,
    pub feature_flags: FeatureFlagsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramIdsConfig {
    pub system_program: String,
    pub token_program: String,
    pub associated_token_program: String,
    pub compute_budget_program: String,
    pub jupiter_program: Option<String>,
    pub orca_whirlpool_program: Option<String>,
    pub raydium_amm_program: Option<String>,
    pub spl_token_swap_program: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub verified: bool,
    pub tradeable: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPairConfig {
    pub base: String,
    pub quote: String,
    pub min_trade_amount: f64,
    pub max_trade_amount: f64,
    pub default_slippage_bps: u16,
    pub priority: u8,
    pub enabled: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexConfigMap {
    pub jupiter: DexConfig,
    pub orca: DexConfig,
    pub raydium: Option<DexConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexConfig {
    pub enabled: bool,
    pub base_url: Option<String>,
    pub quote_url: Option<String>,
    pub swap_url: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub max_retries: Option<u32>,
    pub priority: u8,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyLimitsConfig {
    pub max_swap_amount_sol: f64,
    pub min_balance_reserve_sol: f64,
    pub max_slippage_bps: u16,
    pub transaction_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagsConfig {
    pub enable_real_transactions: bool,
    pub enable_transaction_simulation: bool,
    pub enable_balance_checks: bool,
    pub enable_program_id_validation: bool,
    pub verbose_logging: bool,
}

// Parsed configuration with Pubkey types
#[derive(Debug, Clone)]
pub struct ParsedNetworkConfig {
    pub network: String,
    pub display_name: String,
    pub rpc_endpoint: String,
    pub explorer_base_url: String,
    pub explorer_cluster_param: String,
    pub program_ids: ParsedProgramIds,
    pub tokens: HashMap<String, ParsedTokenConfig>,
    pub trading_pairs: Vec<TradingPairConfig>,
    pub dex_config: DexConfigMap,
    pub safety_limits: SafetyLimitsConfig,
    pub feature_flags: FeatureFlagsConfig,
}

#[derive(Debug, Clone)]
pub struct ParsedProgramIds {
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
pub struct ParsedTokenConfig {
    pub address: Pubkey,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub verified: bool,
    pub tradeable: bool,
    pub note: Option<String>,
}

impl NetworkConfigFile {
    /// Load configuration from JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| anyhow!("Failed to read config file {:?}: {}", path.as_ref(), e))?;

        let config: NetworkConfigFile = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse config JSON: {}", e))?;

        Ok(config)
    }

    /// Load DevNet configuration
    pub fn load_devnet() -> Result<Self> {
        Self::load_from_file("config/devnet.json")
    }

    /// Load MainNet configuration
    pub fn load_mainnet() -> Result<Self> {
        Self::load_from_file("config/mainnet.json")
    }

    /// Load configuration by network name
    pub fn load_by_network(network: &str) -> Result<Self> {
        let config_path = format!("config/{}.json", network.to_lowercase());
        Self::load_from_file(config_path)
    }

    /// Parse string addresses to Pubkey types
    pub fn parse(&self) -> Result<ParsedNetworkConfig> {
        // Parse program IDs
        let program_ids = ParsedProgramIds {
            system_program: Pubkey::from_str(&self.program_ids.system_program)?,
            token_program: Pubkey::from_str(&self.program_ids.token_program)?,
            associated_token_program: Pubkey::from_str(&self.program_ids.associated_token_program)?,
            compute_budget_program: Pubkey::from_str(&self.program_ids.compute_budget_program)?,
            jupiter_program: self
                .program_ids
                .jupiter_program
                .as_ref()
                .map(|s| Pubkey::from_str(s))
                .transpose()?,
            orca_whirlpool_program: self
                .program_ids
                .orca_whirlpool_program
                .as_ref()
                .map(|s| Pubkey::from_str(s))
                .transpose()?,
            raydium_amm_program: self
                .program_ids
                .raydium_amm_program
                .as_ref()
                .map(|s| Pubkey::from_str(s))
                .transpose()?,
            spl_token_swap_program: self
                .program_ids
                .spl_token_swap_program
                .as_ref()
                .map(|s| Pubkey::from_str(s))
                .transpose()?,
        };

        // Parse token addresses
        let mut tokens = HashMap::new();
        for (key, token_config) in &self.token_addresses {
            let parsed_token = ParsedTokenConfig {
                address: Pubkey::from_str(&token_config.address)?,
                symbol: token_config.symbol.clone(),
                name: token_config.name.clone(),
                decimals: token_config.decimals,
                verified: token_config.verified,
                tradeable: token_config.tradeable,
                note: token_config.note.clone(),
            };
            tokens.insert(key.clone(), parsed_token);
        }

        Ok(ParsedNetworkConfig {
            network: self.network.clone(),
            display_name: self.display_name.clone(),
            rpc_endpoint: self.rpc_endpoint.clone(),
            explorer_base_url: self.explorer_base_url.clone(),
            explorer_cluster_param: self.explorer_cluster_param.clone(),
            program_ids,
            tokens,
            trading_pairs: self.trading_pairs.clone(),
            dex_config: self.dex_config.clone(),
            safety_limits: self.safety_limits.clone(),
            feature_flags: self.feature_flags.clone(),
        })
    }

    /// Get explorer URL for a transaction
    pub fn get_explorer_url(&self, tx_signature: &str) -> String {
        format!(
            "{}tx/{}?cluster={}",
            self.explorer_base_url, tx_signature, self.explorer_cluster_param
        )
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate required tokens exist
        if !self.token_addresses.contains_key("sol") {
            return Err(anyhow!("SOL token configuration is required"));
        }

        // Validate trading pairs reference existing tokens
        for pair in &self.trading_pairs {
            if !self.token_addresses.contains_key(&pair.base) {
                return Err(anyhow!(
                    "Trading pair references unknown base token: {}",
                    pair.base
                ));
            }
            if !self.token_addresses.contains_key(&pair.quote) {
                return Err(anyhow!(
                    "Trading pair references unknown quote token: {}",
                    pair.quote
                ));
            }
        }

        // Validate program IDs format
        Pubkey::from_str(&self.program_ids.system_program)
            .map_err(|_| anyhow!("Invalid system program ID"))?;

        Ok(())
    }
}

impl ParsedNetworkConfig {
    /// Get token by symbol
    pub fn get_token(&self, symbol: &str) -> Option<&ParsedTokenConfig> {
        self.tokens.get(symbol)
    }

    /// Get enabled trading pairs
    pub fn get_enabled_trading_pairs(&self) -> Vec<&TradingPairConfig> {
        self.trading_pairs
            .iter()
            .filter(|pair| pair.enabled)
            .collect()
    }

    /// Get trading pair by symbols
    pub fn get_trading_pair(&self, base: &str, quote: &str) -> Option<&TradingPairConfig> {
        self.trading_pairs
            .iter()
            .find(|pair| pair.base == base && pair.quote == quote)
    }

    /// Check if real transactions are enabled
    pub fn transactions_enabled(&self) -> bool {
        self.feature_flags.enable_real_transactions
    }

    /// Get maximum swap amount for safety
    pub fn max_swap_amount(&self) -> f64 {
        self.safety_limits.max_swap_amount_sol
    }

    /// Get explorer URL for a transaction
    pub fn get_explorer_url(&self, tx_signature: &str) -> String {
        format!(
            "{}/tx/{}?cluster={}",
            self.explorer_base_url, tx_signature, self.explorer_cluster_param
        )
    }
}
