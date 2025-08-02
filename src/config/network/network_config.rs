//! Network Configuration for SniperForge Enterprise
//! 
//! Provides parametrized configuration for different Solana networks
//! Avoids hardcoding and enables real swaps across different networks
//! Uses external configuration files for maximum flexibility

use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub network: String,
    pub rpc_endpoint: String,
    pub program_ids: ProgramIds,
    pub token_addresses: HashMap<String, TokenInfo>,
    pub dex_configuration: DexConfiguration,
    pub validation_rules: ValidationRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub verified: bool,
    pub tradeable: bool,
    #[serde(default)]
    pub liquidity_pools: Vec<String>,
    #[serde(default)]
    pub price_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramIds {
    #[serde(with = "pubkey_string")]
    pub system_program: Pubkey,
    #[serde(with = "pubkey_string")]
    pub token_program: Pubkey,
    #[serde(with = "pubkey_string")]
    pub associated_token_program: Pubkey,
    #[serde(with = "pubkey_string")]
    pub compute_budget_program: Pubkey,
    #[serde(with = "pubkey_option_string")]
    pub jupiter_program: Option<Pubkey>,
    #[serde(with = "pubkey_option_string")]
    pub orca_whirlpool_program: Option<Pubkey>,
    #[serde(with = "pubkey_option_string")]
    pub raydium_amm_program: Option<Pubkey>,
    #[serde(with = "pubkey_option_string")]
    pub spl_token_swap_program: Option<Pubkey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexConfiguration {
    pub preferred_dexs: Vec<String>,
    pub dex_detection_enabled: bool,
    pub priority_routing: bool,
    pub fallback_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub require_verified_tokens: bool,
    pub min_liquidity_threshold: u64,
    pub max_slippage_tolerance: f64,
    pub enable_price_impact_checks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfigFile {
    pub networks: HashMap<String, NetworkConfigData>,
    pub global_settings: GlobalSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfigData {
    pub network: String,
    pub rpc_endpoint: String,
    pub program_ids: HashMap<String, Option<String>>,
    pub dex_configuration: DexConfiguration,
    pub token_lists: Vec<String>,
    pub validation_rules: ValidationRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub default_network: String,
    pub config_refresh_interval: u64,
    pub cache_token_lists: bool,
    pub enable_network_switching: bool,
}

// Serde helper modules for Pubkey serialization
mod pubkey_string {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(pubkey: &Pubkey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&pubkey.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Pubkey::from_str(&s).map_err(serde::de::Error::custom)
    }
}

mod pubkey_option_string {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(pubkey: &Option<Pubkey>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match pubkey {
            Some(pk) => serializer.serialize_some(&pk.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Pubkey>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => Ok(Some(Pubkey::from_str(&s).map_err(serde::de::Error::custom)?)),
            None => Ok(None),
        }
    }
}

impl NetworkConfig {
    /// Create NetworkConfig from configuration files
    pub fn from_config(network_name: &str) -> Result<Self> {
        let config_path = "config/network_config.json";
        let config_file = Self::load_config_file(config_path)?;
        
        let network_data = config_file.networks.get(network_name)
            .context(format!("Network '{}' not found in configuration", network_name))?;
        
        // Load program IDs
        let program_ids = Self::parse_program_ids(&network_data.program_ids)?;
        
        // Load token addresses from external files
        let token_addresses = Self::load_token_addresses(&network_data.token_lists)?;
        
        Ok(NetworkConfig {
            network: network_data.network.clone(),
            rpc_endpoint: network_data.rpc_endpoint.clone(),
            program_ids,
            token_addresses,
            dex_configuration: network_data.dex_configuration.clone(),
            validation_rules: network_data.validation_rules.clone(),
        })
    }

    /// Load configuration file
    fn load_config_file(path: &str) -> Result<NetworkConfigFile> {
        let content = fs::read_to_string(path)
            .context(format!("Failed to read configuration file: {}", path))?;
        
        let config: NetworkConfigFile = serde_json::from_str(&content)
            .context("Failed to parse configuration file")?;
        
        Ok(config)
    }

    /// Parse program IDs from configuration
    fn parse_program_ids(program_ids_config: &HashMap<String, Option<String>>) -> Result<ProgramIds> {
        let get_required_pubkey = |key: &str| -> Result<Pubkey> {
            let pubkey_str = program_ids_config.get(key)
                .and_then(|opt| opt.as_ref())
                .context(format!("Required program ID '{}' not found", key))?;
            Pubkey::from_str(pubkey_str).context(format!("Invalid pubkey for '{}'", key))
        };

        let get_optional_pubkey = |key: &str| -> Option<Pubkey> {
            program_ids_config.get(key)
                .and_then(|opt| opt.as_ref())
                .and_then(|s| Pubkey::from_str(s).ok())
        };

        Ok(ProgramIds {
            system_program: get_required_pubkey("system_program")?,
            token_program: get_required_pubkey("token_program")?,
            associated_token_program: get_required_pubkey("associated_token_program")?,
            compute_budget_program: get_required_pubkey("compute_budget_program")?,
            jupiter_program: get_optional_pubkey("jupiter_program"),
            orca_whirlpool_program: get_optional_pubkey("orca_whirlpool_program"),
            raydium_amm_program: get_optional_pubkey("raydium_amm_program"),
            spl_token_swap_program: get_optional_pubkey("spl_token_swap_program"),
        })
    }

    /// Load token addresses from external files
    fn load_token_addresses(token_lists: &[String]) -> Result<HashMap<String, TokenInfo>> {
        let mut token_addresses = HashMap::new();
        
        for token_list_path in token_lists {
            if Path::new(token_list_path).exists() {
                let content = fs::read_to_string(token_list_path)
                    .context(format!("Failed to read token list: {}", token_list_path))?;
                
                let tokens: Vec<TokenInfo> = serde_json::from_str(&content)
                    .context(format!("Failed to parse token list: {}", token_list_path))?;
                
                for token in tokens {
                    token_addresses.insert(token.symbol.clone(), token);
                }
            }
        }
        
        Ok(token_addresses)
    }

    /// Get DevNet configuration (backward compatibility)
    pub fn devnet() -> Result<Self> {
        Self::from_config("devnet")
    }

    /// Get MainNet configuration (backward compatibility)
    pub fn mainnet() -> Result<Self> {
        Self::from_config("mainnet")
    }

    /// Get TestNet configuration
    pub fn testnet() -> Result<Self> {
        Self::from_config("testnet")
    }

    /// Builder pattern for custom configuration
    pub fn builder() -> NetworkConfigBuilder {
        NetworkConfigBuilder::new()
    }

    /// Check if DEX is available for this network
    pub fn is_dex_available(&self, dex_name: &str) -> bool {
        match dex_name.to_lowercase().as_str() {
            "jupiter" => self.program_ids.jupiter_program.is_some(),
            "orca" => self.program_ids.orca_whirlpool_program.is_some(),
            "raydium" => self.program_ids.raydium_amm_program.is_some(),
            "spl_token_swap" => self.program_ids.spl_token_swap_program.is_some(),
            _ => false,
        }
    }

    /// Get preferred DEXs for this network
    pub fn get_preferred_dexs(&self) -> Vec<String> {
        self.dex_configuration.preferred_dexs
            .iter()
            .filter(|dex| self.is_dex_available(dex))
            .cloned()
            .collect()
    }

    /// Validate token for trading
    pub fn validate_token(&self, symbol: &str) -> Result<bool> {
        let token = self.token_addresses.get(symbol)
            .context(format!("Token '{}' not found", symbol))?;

        if self.validation_rules.require_verified_tokens && !token.verified {
            return Ok(false);
        }

        if !token.tradeable {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get token info by symbol
    pub fn get_token_info(&self, symbol: &str) -> Option<&TokenInfo> {
        self.token_addresses.get(symbol)
    }

    /// Get all verified tokens
    pub fn get_verified_tokens(&self) -> Vec<&TokenInfo> {
        self.token_addresses.values()
            .filter(|token| token.verified)
            .collect()
    }

    /// Get all tradeable tokens
    pub fn get_tradeable_tokens(&self) -> Vec<&TokenInfo> {
        self.token_addresses.values()
            .filter(|token| token.tradeable)
            .collect()
    }
}

/// Builder pattern for NetworkConfig
pub struct NetworkConfigBuilder {
    network: Option<String>,
    rpc_endpoint: Option<String>,
    program_ids: Option<ProgramIds>,
    token_addresses: HashMap<String, TokenInfo>,
    dex_configuration: Option<DexConfiguration>,
    validation_rules: Option<ValidationRules>,
}

impl NetworkConfigBuilder {
    pub fn new() -> Self {
        Self {
            network: None,
            rpc_endpoint: None,
            program_ids: None,
            token_addresses: HashMap::new(),
            dex_configuration: None,
            validation_rules: None,
        }
    }

    pub fn network(mut self, network: String) -> Self {
        self.network = Some(network);
        self
    }

    pub fn rpc_endpoint(mut self, endpoint: String) -> Self {
        self.rpc_endpoint = Some(endpoint);
        self
    }

    pub fn program_ids(mut self, program_ids: ProgramIds) -> Self {
        self.program_ids = Some(program_ids);
        self
    }

    pub fn add_token(mut self, symbol: String, token_info: TokenInfo) -> Self {
        self.token_addresses.insert(symbol, token_info);
        self
    }

    pub fn dex_configuration(mut self, dex_config: DexConfiguration) -> Self {
        self.dex_configuration = Some(dex_config);
        self
    }

    pub fn validation_rules(mut self, validation: ValidationRules) -> Self {
        self.validation_rules = Some(validation);
        self
    }

    pub fn build(self) -> Result<NetworkConfig> {
        Ok(NetworkConfig {
            network: self.network.context("Network name is required")?,
            rpc_endpoint: self.rpc_endpoint.context("RPC endpoint is required")?,
            program_ids: self.program_ids.context("Program IDs are required")?,
            token_addresses: self.token_addresses,
            dex_configuration: self.dex_configuration.unwrap_or_else(|| DexConfiguration {
                preferred_dexs: vec![],
                dex_detection_enabled: false,
                priority_routing: false,
                fallback_strategy: "basic".to_string(),
            }),
            validation_rules: self.validation_rules.unwrap_or_else(|| ValidationRules {
                require_verified_tokens: true,
                min_liquidity_threshold: 1000,
                max_slippage_tolerance: 0.05,
                enable_price_impact_checks: true,
            }),
        })
    }
}

impl Default for NetworkConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
