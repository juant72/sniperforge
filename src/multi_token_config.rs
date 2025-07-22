// ===== PROPOSAL-003: MULTI-TOKEN CONFIGURATION SYSTEM =====
// FASE 1C: Sistema de configuración modular para soporte multi-token
// Integración segura con sistema existente sin breaking changes

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tracing::{info, warn, error, debug};

use crate::multi_token_manager::{TokenPairManager, PairConfig, TokenTier, TokenRiskLevel};
use crate::types::{TokenPairSettings, TokenPairRiskAssessment, RiskLevel};

// ===== CONFIGURACIÓN MULTI-TOKEN =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTokenConfig {
    pub enabled: bool,
    pub enabled_tiers: Vec<TokenTier>,
    pub max_pairs_active: usize,
    pub pair_rotation_enabled: bool,
    pub performance_based_selection: bool,
    pub triangular_arbitrage_enabled: bool,
    pub auto_discover_new_pairs: bool,
    pub risk_management: MultiTokenRiskConfig,
    pub execution_settings: MultiTokenExecutionConfig,
    pub monitoring_settings: MultiTokenMonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTokenRiskConfig {
    pub max_total_exposure_usd: f64,
    pub max_per_token_exposure_usd: f64,
    pub max_correlation_threshold: f64,
    pub volatility_adjustment_enabled: bool,
    pub emergency_stop_loss_percentage: f64,
    pub min_liquidity_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTokenExecutionConfig {
    pub max_concurrent_trades: usize,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub slippage_tolerance_bps: u64,
    pub gas_optimization_enabled: bool,
    pub batch_transactions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTokenMonitoringConfig {
    pub price_update_interval_ms: u64,
    pub correlation_update_interval_ms: u64,
    pub performance_tracking_enabled: bool,
    pub detailed_logging: bool,
    pub alerts_enabled: bool,
    pub metric_retention_days: u32,
}

// ===== MANAGER DE CONFIGURACIÓN =====

#[derive(Debug, Clone)]
pub struct MultiTokenConfigManager {
    pub config: MultiTokenConfig,
    pub token_manager: TokenPairManager,
    pub risk_assessments: HashMap<(String, String), TokenPairRiskAssessment>,
    pub performance_metrics: HashMap<(String, String), crate::types::TokenPairMetrics>,
    pub last_config_update: DateTime<Utc>,
}

impl MultiTokenConfigManager {
    /// Crear nuevo config manager con configuración conservadora
    pub fn new() -> Self {
        let conservative_config = MultiTokenConfig {
            enabled: true,
            enabled_tiers: vec![TokenTier::Tier1Major], // Solo Tier 1 inicialmente
            max_pairs_active: 5, // Límite muy conservador
            pair_rotation_enabled: false, // Deshabilitado inicialmente
            performance_based_selection: true,
            triangular_arbitrage_enabled: false, // Deshabilitado inicialmente
            auto_discover_new_pairs: false, // Deshabilitado inicialmente
            risk_management: MultiTokenRiskConfig {
                max_total_exposure_usd: 50000.0,
                max_per_token_exposure_usd: 20000.0,
                max_correlation_threshold: 0.8,
                volatility_adjustment_enabled: true,
                emergency_stop_loss_percentage: 5.0,
                min_liquidity_usd: 100000.0,
            },
            execution_settings: MultiTokenExecutionConfig {
                max_concurrent_trades: 3,
                timeout_seconds: 30,
                retry_attempts: 2,
                slippage_tolerance_bps: 200,
                gas_optimization_enabled: true,
                batch_transactions: false,
            },
            monitoring_settings: MultiTokenMonitoringConfig {
                price_update_interval_ms: 5000,
                correlation_update_interval_ms: 30000,
                performance_tracking_enabled: true,
                detailed_logging: true,
                alerts_enabled: true,
                metric_retention_days: 30,
            },
        };

        Self {
            config: conservative_config,
            token_manager: TokenPairManager::new(),
            risk_assessments: HashMap::new(),
            performance_metrics: HashMap::new(),
            last_config_update: Utc::now(),
        }
    }

    /// FASE 1C: Inicialización segura del sistema multi-token
    pub async fn initialize_safe_multitoken(&mut self) -> Result<()> {
        info!("🚀 PROPOSAL-003 FASE 1C: Inicializando sistema multi-token de forma segura");

        // 1. Inicializar token manager con Tier 1 únicamente
        self.token_manager.initialize_tier1_tokens().await?;
        self.token_manager.initialize_tier1_pairs().await?;

        // 2. Validar configuración
        self.validate_configuration()?;

        // 3. Crear assessments de riesgo iniciales para pares Tier 1
        self.initialize_tier1_risk_assessments().await?;

        // 4. Inicializar métricas de performance
        self.initialize_performance_tracking().await?;

        info!("✅ PROPOSAL-003 FASE 1C: Sistema multi-token inicializado de forma segura");
        info!("📊 Tokens soportados: {}", self.token_manager.supported_tokens.len());
        info!("🔗 Pares configurados: {}", self.token_manager.pair_configurations.len());
        
        Ok(())
    }

    /// Validar que la configuración es segura y no va a romper el sistema
    fn validate_configuration(&self) -> Result<()> {
        debug!("🔍 Validando configuración multi-token");

        // Validar límites razonables
        if self.config.max_pairs_active > 20 {
            return Err(anyhow!("max_pairs_active demasiado alto: {}", self.config.max_pairs_active));
        }

        if self.config.risk_management.max_total_exposure_usd > 100000.0 {
            warn!("⚠️  Alto exposure total configurado: ${}", self.config.risk_management.max_total_exposure_usd);
        }

        if self.config.execution_settings.max_concurrent_trades > 10 {
            return Err(anyhow!("max_concurrent_trades demasiado alto: {}", self.config.execution_settings.max_concurrent_trades));
        }

        // Validar que solo Tier 1 está habilitado inicialmente
        if self.config.enabled_tiers.len() > 1 {
            warn!("⚠️  Múltiples tiers habilitados. Se recomienda solo Tier1Major inicialmente");
        }

        if self.config.triangular_arbitrage_enabled {
            warn!("⚠️  Arbitraje triangular habilitado. Se recomienda deshabilitarlo inicialmente");
        }

        debug!("✅ Configuración multi-token validada");
        Ok(())
    }

    /// Inicializar risk assessments para pares Tier 1
    async fn initialize_tier1_risk_assessments(&mut self) -> Result<()> {
        info!("🛡️  PROPOSAL-003: Inicializando risk assessments Tier 1");

        let active_pairs = self.token_manager.get_active_pairs().await?;
        
        for (token_a, token_b) in active_pairs {
            let risk_assessment = self.calculate_pair_risk_assessment(&token_a, &token_b).await?;
            let pair_key = (token_a.clone(), token_b.clone());
            self.risk_assessments.insert(pair_key, risk_assessment);
            
            debug!("🔍 Risk assessment creado para par: {}/{}", token_a, token_b);
        }

        info!("✅ PROPOSAL-003: {} risk assessments inicializados", self.risk_assessments.len());
        Ok(())
    }

    /// Calcular risk assessment para un par específico
    async fn calculate_pair_risk_assessment(&self, token_a: &str, token_b: &str) -> Result<TokenPairRiskAssessment> {
        // Obtener información de tokens
        let token_a_info = self.token_manager.get_token(token_a)
            .ok_or_else(|| anyhow!("Token {} no encontrado", token_a))?;
        let token_b_info = self.token_manager.get_token(token_b)
            .ok_or_else(|| anyhow!("Token {} no encontrado", token_b))?;

        // Calcular componentes de riesgo
        let volatility_component = (token_a_info.volatility_index + token_b_info.volatility_index) / 2.0;
        
        // Correlación (simplificada para Fase 1C)
        let correlation_component = 0.1; // Valor conservador por defecto
        
        // Liquidez (simplificada para Fase 1C)
        let liquidity_component = 0.05; // Valor conservador por defecto
        
        // Risk score combinado
        let overall_risk = volatility_component + correlation_component + liquidity_component;
        
        // Determinar recomendación
        let recommendation = if overall_risk < 0.1 {
            RiskLevel::Low
        } else if overall_risk < 0.3 {
            RiskLevel::Medium
        } else {
            RiskLevel::High
        };

        Ok(TokenPairRiskAssessment {
            token_a: token_a.to_string(),
            token_b: token_b.to_string(),
            overall_risk,
            volatility_component,
            correlation_component,
            liquidity_component,
            recommendation,
            computed_at: Utc::now(),
        })
    }

    /// Inicializar tracking de performance
    async fn initialize_performance_tracking(&mut self) -> Result<()> {
        info!("📊 PROPOSAL-003: Inicializando performance tracking");

        let active_pairs = self.token_manager.get_active_pairs().await?;
        
        for (token_a, token_b) in active_pairs {
            let metrics = crate::types::TokenPairMetrics {
                token_a: token_a.clone(),
                token_b: token_b.clone(),
                total_opportunities: 0,
                successful_trades: 0,
                total_profit_usd: 0.0,
                success_rate: 0.0,
                average_execution_time_ms: 0.0,
                average_slippage: 0.0,
                best_profit_usd: 0.0,
                worst_loss_usd: 0.0,
                last_profitable_trade: None,
                tracking_period_start: Utc::now(),
            };
            
            let pair_key = (token_a.clone(), token_b.clone());
            self.performance_metrics.insert(pair_key, metrics);
            
            debug!("📈 Performance tracking iniciado para par: {}/{}", token_a, token_b);
        }

        info!("✅ PROPOSAL-003: Performance tracking configurado para {} pares", self.performance_metrics.len());
        Ok(())
    }

    /// Obtener pares habilitados considerando configuración y riesgo
    pub async fn get_enabled_trading_pairs(&self) -> Result<Vec<(String, String)>> {
        let mut enabled_pairs = Vec::new();
        let active_pairs = self.token_manager.get_active_pairs().await?;
        let total_active_pairs = active_pairs.len(); // Guardar el tamaño antes del loop

        for (token_a, token_b) in active_pairs {
            // Verificar que el par cumple criterios de riesgo
            if let Some(risk_assessment) = self.risk_assessments.get(&(token_a.clone(), token_b.clone())) {
                if matches!(risk_assessment.recommendation, RiskLevel::Low | RiskLevel::Medium) {
                    enabled_pairs.push((token_a, token_b));
                } else {
                    debug!("⚠️  Par {}/{} filtrado por alto riesgo", token_a, token_b);
                }
            }
        }

        // Aplicar límite de pares activos
        enabled_pairs.truncate(self.config.max_pairs_active);

        info!("📋 Pares habilitados para trading: {} de {} totales", 
              enabled_pairs.len(), total_active_pairs);
        
        Ok(enabled_pairs)
    }

    /// Obtener configuración para un par específico
    pub fn get_pair_trading_settings(&self, token_a: &str, token_b: &str) -> Option<TokenPairSettings> {
        if let Some(config) = self.token_manager.get_pair_config(token_a, token_b) {
            Some(TokenPairSettings {
                base_token: token_a.to_string(),
                quote_token: token_b.to_string(),
                min_profit_threshold_bps: config.min_profit_threshold_bps,
                max_position_size_usd: config.max_position_size_usd,
                max_slippage_bps: config.max_slippage_bps,
                priority_weight: config.priority as f64,
                enabled: config.enabled,
                risk_multiplier: config.volatility_multiplier,
            })
        } else {
            None
        }
    }

    /// Verificar si el sistema multi-token está listo para operar
    pub fn is_multitoken_ready(&self) -> bool {
        self.config.enabled &&
        !self.token_manager.supported_tokens.is_empty() &&
        !self.token_manager.pair_configurations.is_empty() &&
        !self.risk_assessments.is_empty()
    }

    /// Obtener estadísticas del sistema multi-token
    pub fn get_multitoken_statistics(&self) -> MultiTokenStats {
        let token_stats = self.token_manager.get_statistics();
        let risk_assessments_count = self.risk_assessments.len();
        let performance_metrics_count = self.performance_metrics.len();
        
        MultiTokenStats {
            enabled: self.config.enabled,
            tokens_supported: token_stats.total_tokens,
            pairs_configured: token_stats.total_pairs,
            pairs_active: token_stats.active_pairs,
            risk_assessments: risk_assessments_count,
            performance_metrics: performance_metrics_count,
            enabled_tiers: self.config.enabled_tiers.clone(),
            max_pairs_active: self.config.max_pairs_active,
        }
    }
}

// ===== ESTADÍSTICAS =====

#[derive(Debug, Clone)]
pub struct MultiTokenStats {
    pub enabled: bool,
    pub tokens_supported: usize,
    pub pairs_configured: usize,
    pub pairs_active: usize,
    pub risk_assessments: usize,
    pub performance_metrics: usize,
    pub enabled_tiers: Vec<TokenTier>,
    pub max_pairs_active: usize,
}

impl std::fmt::Display for MultiTokenStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "🚀 PROPOSAL-003 MULTI-TOKEN SYSTEM STATUS:\n\
             ✅ Enabled: {}\n\
             🪙 Tokens Supported: {}\n\
             🔗 Pairs Configured: {}\n\
             📊 Pairs Active: {}\n\
             🛡️  Risk Assessments: {}\n\
             📈 Performance Metrics: {}\n\
             🎯 Enabled Tiers: {:?}\n\
             🔄 Max Active Pairs: {}",
            self.enabled,
            self.tokens_supported,
            self.pairs_configured,
            self.pairs_active,
            self.risk_assessments,
            self.performance_metrics,
            self.enabled_tiers,
            self.max_pairs_active
        )
    }
}

// ===== DEFAULT IMPLEMENTATIONS =====

impl Default for MultiTokenConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

// ===== TESTS UNITARIOS =====

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_multitoken_config_initialization() {
        let mut config_manager = MultiTokenConfigManager::new();
        
        // Test inicialización segura
        assert!(config_manager.initialize_safe_multitoken().await.is_ok());
        
        // Test validación
        assert!(config_manager.is_multitoken_ready());
        
        // Test obtener pares habilitados
        let enabled_pairs = config_manager.get_enabled_trading_pairs().await.unwrap();
        assert!(!enabled_pairs.is_empty());
        
        // Test estadísticas
        let stats = config_manager.get_multitoken_statistics();
        assert!(stats.enabled);
        assert!(stats.tokens_supported > 0);
    }

    #[test]
    fn test_configuration_validation() {
        let config_manager = MultiTokenConfigManager::new();
        
        // Test validación exitosa con configuración conservadora
        assert!(config_manager.validate_configuration().is_ok());
    }
}
