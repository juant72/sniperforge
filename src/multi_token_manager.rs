// ===== PROPOSAL-003: MULTI-TOKEN ARBITRAGE SUPPORT =====
// FASE 1A: Core Token Management Infrastructure
// Sistema modular de gestión de tokens con soporte multi-token

use std::collections::HashMap;
use std::str::FromStr;
use anyhow::{Result, anyhow};
use solana_sdk::pubkey::Pubkey;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

// ===== CORE TOKEN TYPES =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTokenInfo {
    pub symbol: String,
    pub mint_address: Pubkey,
    pub decimals: u8,
    pub tier: TokenTier,
    pub risk_level: TokenRiskLevel,
    pub average_daily_volume: f64,
    pub volatility_index: f64,
    pub correlation_coefficients: HashMap<String, f64>,
    pub verified: bool,
    pub tradeable: bool,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenTier {
    Tier1Major,      // SOL, ETH, BTC, USDC, USDT
    Tier2Ecosystem,  // BONK, RAY, ORCA, PYTH, JTO
    Tier3Stable,     // DAI, FRAX, UXD
    Tier4Experimental, // New/smaller tokens
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenRiskLevel {
    VeryLow,  // Stablecoins
    Low,      // Major tokens (SOL, ETH, BTC)
    Medium,   // Ecosystem tokens
    High,     // Meme tokens, new tokens
    VeryHigh, // Experimental/unknown tokens
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairConfig {
    pub token_a: String,
    pub token_b: String,
    pub enabled: bool,
    pub priority: u8,
    pub max_position_size_usd: f64,
    pub min_profit_threshold_bps: u64,
    pub max_slippage_bps: u64,
    pub volatility_multiplier: f64,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

// ===== TOKEN PAIR MANAGER =====

#[derive(Debug, Clone)]
pub struct TokenPairManager {
    pub supported_tokens: HashMap<String, MultiTokenInfo>,
    pub pair_configurations: HashMap<(String, String), PairConfig>,
    pub enabled_tiers: Vec<TokenTier>,
    pub max_pairs_active: usize,
}

impl TokenPairManager {
    /// Crear nuevo token manager con configuración por defecto
    pub fn new() -> Self {
        Self {
            supported_tokens: HashMap::new(),
            pair_configurations: HashMap::new(),
            enabled_tiers: vec![TokenTier::Tier1Major], // Solo Tier 1 inicialmente
            max_pairs_active: 10, // Límite conservador inicial
        }
    }

    /// FASE 1A: Inicializar solo tokens Tier 1 (majores)
    pub async fn initialize_tier1_tokens(&mut self) -> Result<()> {
        info!("🔧 PROPOSAL-003 FASE 1A: Inicializando tokens Tier 1");
        
        // SOL - Token nativo
        self.add_token("SOL", MultiTokenInfo {
            symbol: "SOL".to_string(),
            mint_address: Pubkey::from_str("So11111111111111111111111111111111111111112")?,
            decimals: 9,
            tier: TokenTier::Tier1Major,
            risk_level: TokenRiskLevel::Low,
            average_daily_volume: 50_000_000.0,
            volatility_index: 0.05,
            correlation_coefficients: HashMap::new(),
            verified: true,
            tradeable: true,
            last_updated: Utc::now(),
        })?;

        // USDC - Stablecoin principal
        self.add_token("USDC", MultiTokenInfo {
            symbol: "USDC".to_string(),
            mint_address: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?,
            decimals: 6,
            tier: TokenTier::Tier1Major,
            risk_level: TokenRiskLevel::VeryLow,
            average_daily_volume: 100_000_000.0,
            volatility_index: 0.001,
            correlation_coefficients: HashMap::new(),
            verified: true,
            tradeable: true,
            last_updated: Utc::now(),
        })?;

        // USDT - Stablecoin secundario
        self.add_token("USDT", MultiTokenInfo {
            symbol: "USDT".to_string(),
            mint_address: Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")?,
            decimals: 6,
            tier: TokenTier::Tier1Major,
            risk_level: TokenRiskLevel::VeryLow,
            average_daily_volume: 80_000_000.0,
            volatility_index: 0.001,
            correlation_coefficients: HashMap::new(),
            verified: true,
            tradeable: true,
            last_updated: Utc::now(),
        })?;

        info!("✅ PROPOSAL-003 FASE 1A: {} tokens Tier 1 inicializados", self.supported_tokens.len());
        Ok(())
    }

    /// FASE 1A: Crear configuraciones iniciales de pares Tier 1
    pub async fn initialize_tier1_pairs(&mut self) -> Result<()> {
        info!("🔧 PROPOSAL-003 FASE 1A: Configurando pares Tier 1");

        // SOL/USDC - Par principal existente
        self.add_pair_config("SOL", "USDC", PairConfig {
            token_a: "SOL".to_string(),
            token_b: "USDC".to_string(),
            enabled: true,
            priority: 1,
            max_position_size_usd: 10000.0,
            min_profit_threshold_bps: 50,
            max_slippage_bps: 200,
            volatility_multiplier: 1.0,
            created_at: Utc::now(),
            last_modified: Utc::now(),
        })?;

        // SOL/USDT - Nuevo par Tier 1
        self.add_pair_config("SOL", "USDT", PairConfig {
            token_a: "SOL".to_string(),
            token_b: "USDT".to_string(),
            enabled: true,
            priority: 2,
            max_position_size_usd: 8000.0,
            min_profit_threshold_bps: 60,
            max_slippage_bps: 250,
            volatility_multiplier: 1.1,
            created_at: Utc::now(),
            last_modified: Utc::now(),
        })?;

        // USDC/USDT - Par de stablecoins
        self.add_pair_config("USDC", "USDT", PairConfig {
            token_a: "USDC".to_string(),
            token_b: "USDT".to_string(),
            enabled: true,
            priority: 3,
            max_position_size_usd: 15000.0,
            min_profit_threshold_bps: 25, // Threshold más bajo para stablecoins
            max_slippage_bps: 100,
            volatility_multiplier: 0.5,
            created_at: Utc::now(),
            last_modified: Utc::now(),
        })?;

        info!("✅ PROPOSAL-003 FASE 1A: {} pares Tier 1 configurados", self.pair_configurations.len());
        Ok(())
    }

    /// Agregar token al manager
    pub fn add_token(&mut self, symbol: &str, token_info: MultiTokenInfo) -> Result<()> {
        if self.supported_tokens.contains_key(symbol) {
            return Err(anyhow!("Token {} ya existe", symbol));
        }
        
        self.supported_tokens.insert(symbol.to_string(), token_info);
        info!("➕ Token agregado: {} ({:?})", symbol, self.supported_tokens[symbol].tier);
        Ok(())
    }

    /// Agregar configuración de par
    pub fn add_pair_config(&mut self, token_a: &str, token_b: &str, config: PairConfig) -> Result<()> {
        // Verificar que ambos tokens existen
        if !self.supported_tokens.contains_key(token_a) {
            return Err(anyhow!("Token {} no existe", token_a));
        }
        if !self.supported_tokens.contains_key(token_b) {
            return Err(anyhow!("Token {} no existe", token_b));
        }

        let pair_key = (token_a.to_string(), token_b.to_string());
        if self.pair_configurations.contains_key(&pair_key) {
            return Err(anyhow!("Par {}/{} ya existe", token_a, token_b));
        }

        self.pair_configurations.insert(pair_key, config);
        info!("➕ Par configurado: {}/{}", token_a, token_b);
        Ok(())
    }

    /// Obtener pares activos (habilitados)
    pub async fn get_active_pairs(&self) -> Result<Vec<(String, String)>> {
        let active_pairs: Vec<(String, String)> = self.pair_configurations
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|((a, b), _)| (a.clone(), b.clone()))
            .collect();

        info!("📊 PROPOSAL-003: {} pares activos de {} totales", 
              active_pairs.len(), self.pair_configurations.len());
        Ok(active_pairs)
    }

    /// Obtener configuración de un par específico
    pub fn get_pair_config(&self, token_a: &str, token_b: &str) -> Option<&PairConfig> {
        let pair_key = (token_a.to_string(), token_b.to_string());
        self.pair_configurations.get(&pair_key)
            .or_else(|| {
                // Intentar orden inverso
                let pair_key_inverse = (token_b.to_string(), token_a.to_string());
                self.pair_configurations.get(&pair_key_inverse)
            })
    }

    /// Obtener información de token por símbolo
    pub fn get_token(&self, symbol: &str) -> Option<&MultiTokenInfo> {
        self.supported_tokens.get(symbol)
    }

    /// Verificar si un par es válido para trading
    pub fn is_pair_tradeable(&self, token_a: &str, token_b: &str) -> bool {
        if let Some(config) = self.get_pair_config(token_a, token_b) {
            if !config.enabled {
                return false;
            }
        } else {
            return false;
        }

        // Verificar que ambos tokens son tradeables
        if let (Some(token_a_info), Some(token_b_info)) = (self.get_token(token_a), self.get_token(token_b)) {
            return token_a_info.tradeable && token_b_info.tradeable;
        }

        false
    }

    /// Obtener estadísticas del manager
    pub fn get_statistics(&self) -> TokenManagerStats {
        let total_tokens = self.supported_tokens.len();
        let total_pairs = self.pair_configurations.len();
        let active_pairs = self.pair_configurations.values().filter(|c| c.enabled).count();
        
        let tier_distribution = self.supported_tokens.values()
            .fold(HashMap::new(), |mut acc, token| {
                *acc.entry(token.tier.clone()).or_insert(0) += 1;
                acc
            });

        TokenManagerStats {
            total_tokens,
            total_pairs,
            active_pairs,
            tier_distribution,
        }
    }
}

// ===== ESTADÍSTICAS =====

#[derive(Debug, Clone)]
pub struct TokenManagerStats {
    pub total_tokens: usize,
    pub total_pairs: usize,
    pub active_pairs: usize,
    pub tier_distribution: HashMap<TokenTier, usize>,
}

impl std::fmt::Display for TokenManagerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "🏛️  PROPOSAL-003 TOKEN MANAGER STATS:\n\
             📊 Total Tokens: {}\n\
             🔗 Total Pairs: {}\n\
             ✅ Active Pairs: {}\n\
             📈 Tier Distribution: {:?}",
            self.total_tokens,
            self.total_pairs,
            self.active_pairs,
            self.tier_distribution
        )
    }
}

// ===== IMPORTS PARA LOGGING =====

use tracing::{info, warn, error};

// ===== TESTS UNITARIOS =====

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_manager_initialization() {
        let mut manager = TokenPairManager::new();
        
        // Test inicialización Tier 1
        assert!(manager.initialize_tier1_tokens().await.is_ok());
        assert_eq!(manager.supported_tokens.len(), 3); // SOL, USDC, USDT
        
        // Test configuración de pares
        assert!(manager.initialize_tier1_pairs().await.is_ok());
        assert_eq!(manager.pair_configurations.len(), 3); // SOL/USDC, SOL/USDT, USDC/USDT
        
        // Test pares activos
        let active_pairs = manager.get_active_pairs().await.unwrap();
        assert_eq!(active_pairs.len(), 3);
        
        // Test token lookup
        assert!(manager.get_token("SOL").is_some());
        assert!(manager.get_token("INVALID").is_none());
        
        // Test pair tradeable
        assert!(manager.is_pair_tradeable("SOL", "USDC"));
        assert!(!manager.is_pair_tradeable("SOL", "INVALID"));
    }
}
