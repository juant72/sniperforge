// ================================================================================
// ARBITRAGE SETTINGS - SISTEMA DE CONFIGURACIÓN CENTRALIZADO
// ================================================================================
// Carga toda la configuración desde arbitrage_settings.json
// Elimina hardcoding y variables de entorno
// ================================================================================

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{info, warn, error};

/// Configuración principal del sistema de arbitraje
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageSettings {
    pub trading: TradingConfig,
    pub wallet: WalletConfig,
    pub rpc: RpcConfig,
    pub apis: ApiConfig,
    pub anti_circular: AntiCircularConfig,
    pub ml_analysis: MlAnalysisConfig,
    pub performance: PerformanceConfig,
    pub triangular_arbitrage: TriangularArbitrageConfig,
    pub mev_protection: MevProtectionConfig,
    pub logging: LoggingConfig,
    pub target_tokens: Vec<TargetToken>,
    pub risk_management: RiskManagementConfig,
    pub dashboard: DashboardConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub mode: String, // "simulation" or "real"
    pub force_real_transactions: bool,
    pub max_trade_sol: f64,
    pub min_profit_threshold_sol: f64,
    pub min_confidence_threshold: f64,
    pub max_concurrent_trades: usize,
    pub trade_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub keypair_file: String,
    pub backup_keypair_file: String,
    pub use_env_private_key: bool,
    pub env_key_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub primary_url: String,
    pub backup_urls: Vec<String>,
    pub timeout_seconds: u64,
    pub retry_attempts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub dexscreener: DexScreenerConfig,
    pub jupiter: JupiterConfig,
    pub coinbase: CoinbaseConfig,
    pub birdeye: BirdeyeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerConfig {
    pub enabled: bool,
    pub timeout_seconds: u64,
    pub max_retries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterConfig {
    pub enabled: bool,
    pub timeout_seconds: u64,
    pub max_retries: usize,
    pub quote_api_url: String,
    pub price_api_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseConfig {
    pub enabled: bool,
    pub timeout_seconds: u64,
    pub max_retries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdeyeConfig {
    pub enabled: bool,
    pub api_key: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiCircularConfig {
    pub enabled: bool,
    pub prevent_same_dex_arbitrage: bool,
    pub max_same_token_repeats: usize,
    pub circular_detection_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlAnalysisConfig {
    pub enabled: bool,
    pub min_score_threshold: f64,
    pub pattern_recognition_enabled: bool,
    pub adaptive_parameters_enabled: bool,
    pub ml_confidence_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_discoveries: usize,
    pub cache_ttl_seconds: u64,
    pub parallel_api_calls: bool,
    pub latency_target_ms: u64,
    pub discovery_cycle_delay_seconds: u64,
    pub cache_enabled: bool,
    pub batch_processing_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularArbitrageConfig {
    pub enabled: bool,
    pub max_hops: usize,
    pub min_net_profit_bps: u64,
    pub circular_detection_enabled: bool,
    pub max_same_token_repeats: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevProtectionConfig {
    pub enabled: bool,
    pub jito_rpc_url: String,
    pub jito_tip_lamports: u64,
    pub sandwich_detection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_enabled: bool,
    pub file_path: String,
    pub console_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetToken {
    pub symbol: String,
    pub mint: String,
    pub enabled: bool,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    pub max_slippage_bps: u64,
    pub max_daily_loss_sol: f64,
    pub stop_loss_enabled: bool,
    pub emergency_stop_enabled: bool,
    pub min_liquidity_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub require_wallet_confirmation: bool,
    pub max_transaction_retries: u32,
    pub transaction_timeout_seconds: u64,
    pub sandwich_detection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub refresh_rate_seconds: u64,
    pub show_detailed_metrics: bool,
    pub show_api_status: bool,
    pub clear_screen_enabled: bool,
}

impl ArbitrageSettings {
    /// Cargar configuración desde archivo JSON
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            error!("❌ Archivo de configuración no encontrado: {}", path.display());
            return Err(anyhow!("Archivo de configuración no encontrado: {}", path.display()));
        }

        info!("📋 Cargando configuración desde: {}", path.display());
        
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("Error leyendo archivo de configuración: {}", e))?;
        
        let settings: ArbitrageSettings = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Error parseando JSON de configuración: {}", e))?;
        
        // Validar configuración
        settings.validate()?;
        
        info!("✅ Configuración cargada exitosamente");
        info!("   🎯 Modo: {}", settings.trading.mode);
        info!("   💰 Max trade: {} SOL", settings.trading.max_trade_sol);
        info!("   🔑 Wallet file: {}", settings.wallet.keypair_file);
        info!("   🛡️ Anti-circular: {}", if settings.anti_circular.enabled { "ENABLED" } else { "DISABLED" });
        
        Ok(settings)
    }

    /// Cargar configuración con archivo por defecto
    pub fn load_default() -> Result<Self> {
        Self::load_from_file("./arbitrage_settings.json")
    }

    /// Guardar configuración actual a archivo
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Error serializando configuración: {}", e))?;
        
        fs::write(path.as_ref(), content)
            .map_err(|e| anyhow!("Error escribiendo archivo de configuración: {}", e))?;
        
        info!("✅ Configuración guardada en: {}", path.as_ref().display());
        Ok(())
    }

    /// Validar configuración
    fn validate(&self) -> Result<()> {
        // Validar modo de trading
        if !["simulation", "real"].contains(&self.trading.mode.as_str()) {
            return Err(anyhow!("Modo de trading inválido: {}. Debe ser 'simulation' o 'real'", self.trading.mode));
        }

        // Validar límites de trading
        if self.trading.max_trade_sol <= 0.0 {
            return Err(anyhow!("max_trade_sol debe ser mayor que 0"));
        }

        if self.trading.min_profit_threshold_sol < 0.0 {
            return Err(anyhow!("min_profit_threshold_sol no puede ser negativo"));
        }

        // Validar archivo de wallet
        if self.wallet.keypair_file.is_empty() {
            return Err(anyhow!("keypair_file no puede estar vacío"));
        }

        // Validar URLs
        if self.rpc.primary_url.is_empty() {
            return Err(anyhow!("primary_url no puede estar vacío"));
        }

        // Validar tokens objetivo
        if self.target_tokens.is_empty() {
            warn!("⚠️ No hay tokens objetivo configurados");
        }

        info!("✅ Configuración validada correctamente");
        Ok(())
    }

    /// Verificar si el trading real está habilitado
    pub fn is_real_trading_enabled(&self) -> bool {
        self.trading.mode == "real" || self.trading.force_real_transactions
    }

    /// Obtener tokens habilitados ordenados por prioridad
    pub fn get_enabled_tokens(&self) -> Vec<&TargetToken> {
        let mut tokens: Vec<&TargetToken> = self.target_tokens
            .iter()
            .filter(|token| token.enabled)
            .collect();
        
        tokens.sort_by_key(|token| token.priority);
        tokens
    }

    /// Crear configuración por defecto para testing
    pub fn default_for_testing() -> Self {
        Self {
            trading: TradingConfig {
                mode: "simulation".to_string(),
                force_real_transactions: false,
                max_trade_sol: 0.001,
                min_profit_threshold_sol: 0.000001,
                min_confidence_threshold: 0.1,
                max_concurrent_trades: 1,
                trade_timeout_seconds: 10,
            },
            wallet: WalletConfig {
                keypair_file: "./test_keypair.json".to_string(),
                backup_keypair_file: "".to_string(),
                use_env_private_key: false,
                env_key_name: "SOLANA_PRIVATE_KEY".to_string(),
            },
            rpc: RpcConfig {
                primary_url: "https://api.devnet.solana.com".to_string(),
                backup_urls: vec![],
                timeout_seconds: 10,
                retry_attempts: 2,
            },
            apis: ApiConfig {
                dexscreener: DexScreenerConfig {
                    enabled: true,
                    timeout_seconds: 5,
                    max_retries: 1,
                },
                jupiter: JupiterConfig {
                    enabled: false,
                    timeout_seconds: 3,
                    max_retries: 1,
                    quote_api_url: "https://quote-api.jup.ag/v6".to_string(),
                    price_api_url: "https://api.jup.ag/price/v2".to_string(),
                },
                coinbase: CoinbaseConfig {
                    enabled: false,
                    timeout_seconds: 3,
                    max_retries: 1,
                },
                birdeye: BirdeyeConfig {
                    enabled: false,
                    api_key: "".to_string(),
                    timeout_seconds: 5,
                },
            },
            anti_circular: AntiCircularConfig {
                enabled: true,
                prevent_same_dex_arbitrage: true,
                max_same_token_repeats: 1,
                circular_detection_enabled: true,
            },
            ml_analysis: MlAnalysisConfig {
                enabled: false,
                min_score_threshold: 0.1,
                pattern_recognition_enabled: false,
                adaptive_parameters_enabled: false,
                ml_confidence_threshold: 0.3,
            },
            performance: PerformanceConfig {
                max_concurrent_discoveries: 1,
                cache_ttl_seconds: 1,
                parallel_api_calls: false,
                latency_target_ms: 1000,
                discovery_cycle_delay_seconds: 2,
                cache_enabled: false,
                batch_processing_enabled: false,
            },
            triangular_arbitrage: TriangularArbitrageConfig {
                enabled: false,
                max_hops: 2,
                min_net_profit_bps: 100,
                circular_detection_enabled: true,
                max_same_token_repeats: 1,
            },
            mev_protection: MevProtectionConfig {
                enabled: false,
                jito_rpc_url: "".to_string(),
                jito_tip_lamports: 1000,
                sandwich_detection: false,
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
                file_enabled: false,
                file_path: "./test.log".to_string(),
                console_enabled: true,
            },
            target_tokens: vec![
                TargetToken {
                    symbol: "SOL".to_string(),
                    mint: "So11111111111111111111111111111111111111112".to_string(),
                    enabled: true,
                    priority: 1,
                },
            ],
            risk_management: RiskManagementConfig {
                max_slippage_bps: 100,
                max_daily_loss_sol: 0.01,
                stop_loss_enabled: false,
                emergency_stop_enabled: false,
                min_liquidity_usd: 100.0,
            },
            dashboard: DashboardConfig {
                refresh_rate_seconds: 10,
                show_detailed_metrics: false,
                show_api_status: false,
                clear_screen_enabled: false,
            },
            security: SecurityConfig {
                require_wallet_confirmation: false,
                max_transaction_retries: 2,
                transaction_timeout_seconds: 20,
                sandwich_detection: false,
            },
        }
    }
}

/// Utilidad para crear configuración inicial
pub fn create_default_config_file() -> Result<()> {
    let default_settings = ArbitrageSettings::load_default().unwrap_or_else(|_| {
        warn!("📝 Creando archivo de configuración por defecto");
        
        // Configuración por defecto optimizada para producción
        ArbitrageSettings {
            trading: TradingConfig {
                mode: "simulation".to_string(),
                force_real_transactions: false,
                max_trade_sol: 0.01,
                min_profit_threshold_sol: 0.000005,
                min_confidence_threshold: 0.3,
                max_concurrent_trades: 3,
                trade_timeout_seconds: 30,
            },
            wallet: WalletConfig {
                keypair_file: "./keypair.json".to_string(),
                backup_keypair_file: "~/.config/solana/id.json".to_string(),
                use_env_private_key: false,
                env_key_name: "SOLANA_PRIVATE_KEY".to_string(),
            },
            rpc: RpcConfig {
                primary_url: "https://api.mainnet-beta.solana.com".to_string(),
                backup_urls: vec![
                    "https://solana-api.projectserum.com".to_string(),
                    "https://rpc.ankr.com/solana".to_string(),
                ],
                timeout_seconds: 15,
                retry_attempts: 3,
            },
            apis: ApiConfig {
                dexscreener: DexScreenerConfig {
                    enabled: true,
                    timeout_seconds: 10,
                    max_retries: 2,
                },
                jupiter: JupiterConfig {
                    enabled: true,
                    timeout_seconds: 5,
                    max_retries: 3,
                    quote_api_url: "https://quote-api.jup.ag/v6".to_string(),
                    price_api_url: "https://api.jup.ag/price/v2".to_string(),
                },
                coinbase: CoinbaseConfig {
                    enabled: true,
                    timeout_seconds: 8,
                    max_retries: 2,
                },
                birdeye: BirdeyeConfig {
                    enabled: false,
                    api_key: "".to_string(),
                    timeout_seconds: 10,
                },
            },
            anti_circular: AntiCircularConfig {
                enabled: true,
                prevent_same_dex_arbitrage: true,
                max_same_token_repeats: 1,
                circular_detection_enabled: true,
            },
            ml_analysis: MlAnalysisConfig {
                enabled: true,
                min_score_threshold: 0.2,
                pattern_recognition_enabled: true,
                adaptive_parameters_enabled: true,
                ml_confidence_threshold: 0.6,
            },
            performance: PerformanceConfig {
                max_concurrent_discoveries: 3,
                cache_ttl_seconds: 2,
                parallel_api_calls: true,
                latency_target_ms: 150,
                discovery_cycle_delay_seconds: 5,
                cache_enabled: true,
                batch_processing_enabled: true,
            },
            triangular_arbitrage: TriangularArbitrageConfig {
                enabled: true,
                max_hops: 3,
                min_net_profit_bps: 50,
                circular_detection_enabled: true,
                max_same_token_repeats: 1,
            },
            mev_protection: MevProtectionConfig {
                enabled: true,
                jito_rpc_url: "https://mainnet.block-engine.jito.wtf".to_string(),
                jito_tip_lamports: 10000,
                sandwich_detection: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_enabled: false,
                file_path: "./logs/arbitrage.log".to_string(),
                console_enabled: true,
            },
            target_tokens: vec![
                TargetToken {
                    symbol: "SOL".to_string(),
                    mint: "So11111111111111111111111111111111111111112".to_string(),
                    enabled: true,
                    priority: 1,
                },
                TargetToken {
                    symbol: "USDC".to_string(),
                    mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                    enabled: true,
                    priority: 2,
                },
            ],
            risk_management: RiskManagementConfig {
                max_slippage_bps: 50,
                max_daily_loss_sol: 0.1,
                stop_loss_enabled: true,
                emergency_stop_enabled: true,
                min_liquidity_usd: 1000.0,
            },
            dashboard: DashboardConfig {
                refresh_rate_seconds: 5,
                show_detailed_metrics: true,
                show_api_status: true,
                clear_screen_enabled: true,
            },
            security: SecurityConfig {
                require_wallet_confirmation: true,
                max_transaction_retries: 3,
                transaction_timeout_seconds: 30,
                sandwich_detection: true,
            },
        }
    });

    default_settings.save_to_file("./arbitrage_settings.json")?;
    info!("✅ Archivo de configuración creado: ./arbitrage_settings.json");
    
    Ok(())
}

/// Implementación del trait Default para ArbitrageSettings
impl Default for ArbitrageSettings {
    fn default() -> Self {
        ArbitrageSettings {
            trading: TradingConfig {
                mode: "simulation".to_string(),
                force_real_transactions: false,
                max_trade_sol: 0.01,
                min_profit_threshold_sol: 0.000005,
                min_confidence_threshold: 0.3,
                max_concurrent_trades: 3,
                trade_timeout_seconds: 30,
            },
            wallet: WalletConfig {
                keypair_file: "./keypair.json".to_string(),
                backup_keypair_file: "~/.config/solana/id.json".to_string(),
                use_env_private_key: false,
                env_key_name: "SOLANA_PRIVATE_KEY".to_string(),
            },
            rpc: RpcConfig {
                primary_url: "https://api.mainnet-beta.solana.com".to_string(),
                backup_urls: vec![
                    "https://solana-api.projectserum.com".to_string(),
                    "https://rpc.ankr.com/solana".to_string(),
                ],
                timeout_seconds: 15,
                retry_attempts: 3,
            },
            apis: ApiConfig {
                dexscreener: DexScreenerConfig {
                    enabled: true,
                    timeout_seconds: 10,
                    max_retries: 2,
                },
                jupiter: JupiterConfig {
                    enabled: true,
                    timeout_seconds: 5,
                    max_retries: 3,
                    quote_api_url: "https://quote-api.jup.ag/v6".to_string(),
                    price_api_url: "https://api.jup.ag/price/v2".to_string(),
                },
                coinbase: CoinbaseConfig {
                    enabled: true,
                    timeout_seconds: 8,
                    max_retries: 2,
                },
                birdeye: BirdeyeConfig {
                    enabled: false,
                    api_key: "".to_string(),
                    timeout_seconds: 10,
                },
            },
            anti_circular: AntiCircularConfig {
                enabled: true,
                prevent_same_dex_arbitrage: true,
                max_same_token_repeats: 1,
                circular_detection_enabled: true,
            },
            ml_analysis: MlAnalysisConfig {
                enabled: true,
                min_score_threshold: 0.2,
                pattern_recognition_enabled: true,
                adaptive_parameters_enabled: true,
                ml_confidence_threshold: 0.6,
            },
            target_tokens: vec![
                TargetToken {
                    symbol: "SOL".to_string(),
                    mint: "So11111111111111111111111111111111111111112".to_string(),
                    enabled: true,
                    priority: 1,
                },
                TargetToken {
                    symbol: "WIF".to_string(),
                    mint: "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm".to_string(),
                    enabled: true,
                    priority: 2,
                },
                TargetToken {
                    symbol: "PYTH".to_string(),
                    mint: "HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3".to_string(),
                    enabled: true,
                    priority: 3,
                },
                TargetToken {
                    symbol: "JUP".to_string(),
                    mint: "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN".to_string(),
                    enabled: true,
                    priority: 4,
                },
                TargetToken {
                    symbol: "ORCA".to_string(),
                    mint: "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE".to_string(),
                    enabled: true,
                    priority: 5,
                },
            ],
            performance: PerformanceConfig {
                max_concurrent_discoveries: 10,
                discovery_cycle_delay_seconds: 5,
                latency_target_ms: 500,
                cache_enabled: true,
                cache_ttl_seconds: 30,
                parallel_api_calls: true,
                batch_processing_enabled: true,
            },
            risk_management: RiskManagementConfig {
                max_slippage_bps: 100,
                max_daily_loss_sol: 0.1,
                stop_loss_enabled: true,
                emergency_stop_enabled: true,
                min_liquidity_usd: 1000.0,
            },
            dashboard: DashboardConfig {
                refresh_rate_seconds: 5,
                show_detailed_metrics: true,
                show_api_status: true,
                clear_screen_enabled: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_enabled: true,
                file_path: "./arbitrage.log".to_string(),
                console_enabled: true,
            },
            triangular_arbitrage: TriangularArbitrageConfig {
                enabled: true,
                max_hops: 3,
                min_net_profit_bps: 50,
                circular_detection_enabled: true,
                max_same_token_repeats: 1,
            },
            mev_protection: MevProtectionConfig {
                enabled: true,
                jito_rpc_url: "https://mainnet.block-engine.jito.wtf".to_string(),
                jito_tip_lamports: 10000,
                sandwich_detection: true,
            },
            security: SecurityConfig {
                require_wallet_confirmation: true,
                max_transaction_retries: 3,
                transaction_timeout_seconds: 30,
                sandwich_detection: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_and_validate_config() {
        let config = ArbitrageSettings::default_for_testing();
        assert!(config.validate().is_ok());
        assert!(!config.is_real_trading_enabled());
    }

    #[test]
    fn test_enabled_tokens() {
        let config = ArbitrageSettings::default_for_testing();
        let enabled_tokens = config.get_enabled_tokens();
        assert_eq!(enabled_tokens.len(), 1);
        assert_eq!(enabled_tokens[0].symbol, "SOL");
    }
}
