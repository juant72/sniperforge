// ================================================================================
// ARBITRAGE BOT MODULE - Módulo Principal
// ================================================================================

//! # Arbitrage Bot
//! 
//! Bot de arbitraje avanzado para Solana DeFi que incluye:
//! - Arbitraje simple entre DEXs
//! - Arbitraje triangular
//! - Machine Learning para predicción de oportunidades
//! - Sistema de fases avanzadas (Quantum, Autonomous, Ecosystem)
//! 
//! ## Características principales:
//! - Detección automática de oportunidades de arbitraje
//! - Integración con Jupiter V6 para ejecución optimizada
//! - Gestión de riesgo avanzada
//! - Análisis ML para scoring de oportunidades
//! - Soporte para múltiples DEXs (Raydium, Orca, Serum, Phoenix)

// Re-export main components
pub mod main;
pub mod config;
pub mod engine;
pub mod triangular;
pub mod ml;
pub mod phases;

// Public API
pub use main::{ArbitrageBot, CycleResult, PhaseManager, PhaseStats};
pub use config::{
    ArbitrageConfig, GeneralConfig, EngineConfig, TriangularConfig, 
    MlConfig, TradingConfig, PhaseConfig, RiskConfig
};
pub use engine::{ArbitrageEngine, ArbitrageOpportunity, EngineStatistics};
pub use triangular::{TriangularEngine, TriangularOpportunity};
pub use ml::{PatternRecognition, OpportunityPattern};

// Re-export important types from core
pub use sniperforge_core::{Bot, BotStatistics, CoreResult, CoreError, TradingOpportunity};

/// Versión del bot de arbitraje
pub const VERSION: &str = "1.0.0";

/// Nombre del bot
pub const BOT_NAME: &str = "ArbitrageBot";

/// Descripción del bot
pub const BOT_DESCRIPTION: &str = "Advanced DeFi arbitrage bot with ML and multi-phase optimization";

/// Crear configuración por defecto para arbitraje
pub fn default_config() -> ArbitrageConfig {
    ArbitrageConfig::default()
}

/// Crear configuración conservadora
pub fn conservative_config() -> ArbitrageConfig {
    ArbitrageConfig::conservative()
}

/// Crear configuración agresiva
pub fn aggressive_config() -> ArbitrageConfig {
    ArbitrageConfig::aggressive()
}

/// Crear configuración para devnet
pub fn devnet_config() -> ArbitrageConfig {
    ArbitrageConfig::devnet()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = default_config();
        assert_eq!(config.general.enabled, true);
        assert!(config.engine.min_profit_sol > 0.0);
    }
    
    #[test]
    fn test_conservative_config() {
        let config = conservative_config();
        assert!(config.engine.min_profit_sol > default_config().engine.min_profit_sol);
        assert_eq!(config.general.max_concurrent_trades, 1);
    }
    
    #[test]
    fn test_aggressive_config() {
        let config = aggressive_config();
        assert!(config.engine.min_profit_sol < default_config().engine.min_profit_sol);
        assert!(config.general.max_concurrent_trades > default_config().general.max_concurrent_trades);
    }
}
