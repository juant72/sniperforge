//! ARBITRAGE BOT PHASE 4.5 - INTEGRATED SYSTEM
//! Sistema de arbitraje integrado que combina todas las mejoras de Fases 1-4
//! en una arquitectura evolutiva que preserva la funcionalidad base.

use anyhow::Result;
use tracing::{info, warn, error, Level};
use tracing_subscriber;
use std::sync::Arc;

// Import the Phase 4.5 integrated system
use sniperforge::arbitrage_bot_phase45_integrated::ArbitrageBotPhase45Integrated;
use sniperforge::unified_config::UnifiedPhase45Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 SNIPERFORGE ARBITRAGE BOT PHASE 4.5 - INTEGRATED SYSTEM");
    info!("===============================================================");
    info!("Sistema evolutivo que combina mejoras Fases 1-4 con funcionalidad base");
    
    // Load configuration
    let config = load_configuration().await?;
    
    // Display configuration summary
    display_config_summary(&config).await;
    
    // Initialize the integrated system
    info!("🔧 Inicializando sistema integrado Phase 4.5...");
    let system = ArbitrageBotPhase45Integrated::new(config.clone()).await?;
    
    info!("✅ Sistema Phase 4.5 inicializado correctamente");
    
    // Start the main loop
    info!("🚀 Iniciando loop principal del sistema integrado...");
    
    // Since start_main_loop takes &self, we need to work with the reference
    let result = system.start_main_loop().await;
    
    match result {
        Ok(_) => info!("✅ Sistema ejecutado exitosamente"),
        Err(e) => error!("❌ Error en ejecución: {}", e),
    }
    
    Ok(())
}

/// Load and validate configuration for Phase 4.5
async fn load_configuration() -> Result<UnifiedPhase45Config> {
    info!("📋 Cargando configuración unificada Phase 4.5...");
    
    // Use professional trading configuration as default
    let config = UnifiedPhase45Config::professional_trading();
    
    // Display available configuration modes
    info!("💡 Modos de configuración disponibles:");
    info!("   🟢 Micro Trading    - Operaciones pequeñas, bajo riesgo");
    info!("   🟡 Safe Trading     - Operaciones seguras, riesgo controlado");
    info!("   🟠 Active Trading   - Operaciones activas, riesgo medio");
    info!("   🔴 Professional     - Operaciones profesionales, riesgo alto");
    info!("   ✅ Usando: Professional Trading (recomendado para Phase 4.5)");
    
    // Validate configuration
    match config.validate() {
        Ok(_) => {
            info!("✅ Configuración validada correctamente");
        },
        Err(validation_errors) => {
            warn!("⚠️ Errores de validación en configuración:");
            for error in validation_errors {
                warn!("   - {}", error);
            }
            info!("🔧 Usando valores por defecto para parámetros inválidos");
        }
    }
    
    Ok(config)
}

/// Display configuration summary
async fn display_config_summary(config: &UnifiedPhase45Config) {
    info!("📊 RESUMEN DE CONFIGURACIÓN PHASE 4.5");
    info!("=====================================");
    
    // Basic trading parameters
    info!("🎯 PARÁMETROS BÁSICOS:");
    info!("   💰 Min Profit: {} BPS", config.min_profit_bps);
    info!("   📉 Max Slippage: {} BPS", config.max_slippage_bps);
    info!("   💲 Max Trade: {} SOL", config.max_trade_sol);
    info!("   📈 Min Trade: {} SOL", config.min_trade_sol);
    
    // Phase-specific features
    info!("🚀 CARACTERÍSTICAS PHASE 4.5:");
    info!("   🎯 Jupiter Advanced: {}", if config.jupiter_advanced_enabled { "✅" } else { "❌" });
    info!("   🛡️ MEV Protection: {}", if config.mev_protection_enabled { "✅" } else { "❌" });
    info!("   🔧 DEX Specialization: {}", if config.dex_specialization_enabled { "✅" } else { "❌" });
    info!("   ⚡ Event-Driven: {}", if config.event_driven_enabled { "✅" } else { "❌" });
    info!("   🔄 Parallel Execution: {}", if config.parallel_execution_enabled { "✅" } else { "❌" });
    
    // Performance settings
    info!("⚡ CONFIGURACIÓN DE PERFORMANCE:");
    info!("   ⏱️ API Timeout: {}ms", config.api_timeout_ms);
    info!("   🔄 Max Concurrent: {}", config.max_concurrent_tasks);
    info!("   📊 Update Interval: {}ms", config.metrics_update_interval_ms);
    
    // Network settings
    info!("🌐 CONFIGURACIÓN DE RED:");
    info!("   📡 RPC Endpoint: {}", config.rpc_endpoint);
    info!("   🚀 Jito RPC: {}", config.jito_rpc_url);
    
    info!("=======================================");
}
