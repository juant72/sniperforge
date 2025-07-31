// ================================================================================
// ARBITRAGE BOT CON CONFIGURACIÓN JSON - PHASE 4.5 ENHANCED
// ================================================================================
// Sistema de arbitraje con configuración centralizada en JSON
// Elimina hardcoding y variables de entorno
// ================================================================================

use anyhow::Result;
use tracing::{info, warn, error};
use std::env;
use sniperforge::arbitrage_settings::ArbitrageSettings;

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar logging básico
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 Iniciando Arbitrage Bot con Configuración JSON");
    info!("📋 Cargando configuración desde arbitrage_settings.json...");

    // Cargar configuración desde JSON
    let settings = match ArbitrageSettings::load_default() {
        Ok(settings) => {
            info!("✅ Configuración cargada exitosamente");
            settings
        }
        Err(e) => {
            error!("❌ Error cargando configuración: {}", e);
            error!("💡 Tip: Asegúrate de que existe el archivo arbitrage_settings.json");
            error!("💡 Tip: Puedes crear uno con: cargo run --bin create_default_config");
            return Err(e);
        }
    };

    // Mostrar configuración cargada
    display_configuration(&settings);

    // Verificar modo de trading
    if settings.is_real_trading_enabled() {
        warn!("🔥 MODO TRADING REAL ACTIVADO");
        warn!("💰 Máximo por trade: {} SOL", settings.trading.max_trade_sol);
        warn!("🔑 Wallet file: {}", settings.wallet.keypair_file);
        
        // Verificar que existe el archivo de wallet
        if !std::path::Path::new(&settings.wallet.keypair_file).exists() {
            error!("❌ Archivo de wallet no encontrado: {}", settings.wallet.keypair_file);
            error!("💡 Tip: Configura 'wallet.keypair_file' en arbitrage_settings.json");
            return Err(anyhow::anyhow!("Wallet file not found"));
        }
    } else {
        info!("🛡️ Modo simulación activado - Trading seguro");
    }

    // TODO: Aquí iría la lógica principal del sistema de arbitraje
    // usando la configuración cargada desde JSON
    
    info!("🎯 Sistema configurado correctamente");
    info!("🔧 Para cambiar configuración, edita: arbitrage_settings.json");
    info!("🚀 Para activar trading real, cambia 'trading.mode' a 'real' en el JSON");
    
    // Simular ejecución por ahora
    for i in 1..=5 {
        info!("⏳ Ciclo de arbitraje #{} (configuración JSON aplicada)", i);
        tokio::time::sleep(tokio::time::Duration::from_secs(settings.dashboard.refresh_rate_seconds)).await;
    }
    
    info!("✅ Demo completada - Sistema listo para integración completa");
    
    Ok(())
}

fn display_configuration(settings: &ArbitrageSettings) {
    info!("═══════════════════════════════════════════════════");
    info!("📋 CONFIGURACIÓN CARGADA DESDE JSON");
    info!("═══════════════════════════════════════════════════");
    
    // Trading config
    info!("💰 TRADING:");
    info!("   • Modo: {}", settings.trading.mode);
    info!("   • Real trading: {}", settings.is_real_trading_enabled());
    info!("   • Max trade: {} SOL", settings.trading.max_trade_sol);
    info!("   • Min profit: {} SOL", settings.trading.min_profit_threshold_sol);
    info!("   • Min confidence: {:.1}%", settings.trading.min_confidence_threshold * 100.0);
    
    // Wallet config
    info!("🔑 WALLET:");
    info!("   • Keypair file: {}", settings.wallet.keypair_file);
    info!("   • Backup file: {}", settings.wallet.backup_keypair_file);
    info!("   • Use env key: {}", settings.wallet.use_env_private_key);
    
    // RPC config
    info!("🌐 RPC:");
    info!("   • Primary: {}", settings.rpc.primary_url);
    info!("   • Backups: {} configured", settings.rpc.backup_urls.len());
    info!("   • Timeout: {}s", settings.rpc.timeout_seconds);
    
    // API config
    info!("🔌 APIs:");
    info!("   • DexScreener: {}", if settings.apis.dexscreener.enabled { "✅" } else { "❌" });
    info!("   • Jupiter: {}", if settings.apis.jupiter.enabled { "✅" } else { "❌" });
    info!("   • Coinbase: {}", if settings.apis.coinbase.enabled { "✅" } else { "❌" });
    info!("   • Birdeye: {}", if settings.apis.birdeye.enabled { "✅" } else { "❌" });
    
    // Anti-circular config
    info!("🛡️ ANTI-CIRCULAR:");
    info!("   • Enabled: {}", settings.anti_circular.enabled);
    info!("   • Prevent same DEX: {}", settings.anti_circular.prevent_same_dex_arbitrage);
    info!("   • Circular detection: {}", settings.anti_circular.circular_detection_enabled);
    
    // ML config
    info!("🧠 MACHINE LEARNING:");
    info!("   • Enabled: {}", settings.ml_analysis.enabled);
    info!("   • Min score: {:.2}", settings.ml_analysis.min_score_threshold);
    info!("   • Pattern recognition: {}", settings.ml_analysis.pattern_recognition_enabled);
    
    // Target tokens
    let enabled_tokens = settings.get_enabled_tokens();
    info!("🎯 TARGET TOKENS:");
    info!("   • Total enabled: {}", enabled_tokens.len());
    for token in enabled_tokens.iter().take(5) {
        info!("   • {} (Priority: {})", token.symbol, token.priority);
    }
    if enabled_tokens.len() > 5 {
        info!("   • ... and {} more", enabled_tokens.len() - 5);
    }
    
    // Performance config
    info!("⚡ PERFORMANCE:");
    info!("   • Max concurrent: {}", settings.performance.max_concurrent_discoveries);
    info!("   • Cycle delay: {}s", settings.performance.discovery_cycle_delay_seconds);
    info!("   • Latency target: {}ms", settings.performance.latency_target_ms);
    
    info!("═══════════════════════════════════════════════════");
}

/// Función de ayuda para mostrar cómo cambiar configuración
fn show_configuration_help() {
    println!();
    println!("📋 CÓMO CAMBIAR CONFIGURACIÓN:");
    println!("═══════════════════════════════════════════");
    println!("1. Edita el archivo: arbitrage_settings.json");
    println!("2. Cambia los valores que necesites:");
    println!("   • trading.mode: 'simulation' o 'real'");
    println!("   • trading.max_trade_sol: cantidad máxima");
    println!("   • wallet.keypair_file: ruta a tu wallet");
    println!("   • target_tokens: habilita/deshabilita tokens");
    println!("3. Reinicia el programa (NO necesitas recompilar)");
    println!();
    println!("🔥 PARA ACTIVAR TRADING REAL:");
    println!("   Cambia 'trading.mode' a 'real' en el JSON");
    println!();
}
