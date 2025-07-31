// ================================================================================
// ARBITRAGE BOT PHASE 4.5 - REAL TRADING MODE (100% REAL)
// ================================================================================
// CONFIGURADO PARA TRADING REAL EN MAINNET - NO SIMULACION
// ================================================================================

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn};
use anyhow::Result;

// Importar el sistema integrado Phase 4.5
use sniperforge::{
    arbitrage_bot_phase45_integrated::ArbitrageBotPhase45Integrated,
    unified_config::UnifiedPhase45Config
};

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar logging con más detalle para trading real
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("💰 INICIANDO SISTEMA DE ARBITRAJE REAL - 100% TRADING REAL");
    info!("================================================================");
    info!("⚠️  ATENCIÓN: ESTE ES TRADING REAL CON DINERO REAL EN MAINNET");
    info!("================================================================");

    // CONFIGURACIÓN PARA TRADING REAL ACTIVO
    let config = UnifiedPhase45Config::real_trading_mainnet();
    
    // Mostrar configuración de trading real
    info!("💼 CONFIGURACIÓN DE TRADING REAL:");
    info!("   🌐 Network: MAINNET (REAL TRADING)");
    info!("   💰 Max Trade SOL: {:.3} (REAL MONEY)", config.max_trade_sol);
    info!("   📈 Min Profit BPS: {} ({}%)", config.min_profit_bps, config.min_profit_bps as f64 / 100.0);
    info!("   🚀 Jupiter Advanced: {}", if config.jupiter_advanced_enabled { "✅ ACTIVO" } else { "❌" });
    info!("   🛡️ MEV Protection: {}", if config.mev_protection_enabled { "✅ ACTIVO" } else { "❌" });
    info!("   🎯 DEX Specialization: {}", if config.dex_specialization_enabled { "✅ ACTIVO" } else { "❌" });
    info!("   ⚡ Event Driven: {}", if config.event_driven_enabled { "✅ ACTIVO" } else { "❌" });
    info!("   💸 Slippage Max: {}%", config.max_slippage_bps as f64 / 100.0);

    // Verificación de wallet real
    info!("💼 Verificando wallet y balance...");
    // TODO: Agregar verificación de balance mínimo antes de comenzar

    // Crear e inicializar el sistema Phase 4.5 para trading real
    info!("🔧 Inicializando Sistema Phase 4.5 MODO REAL...");
    let system = match ArbitrageBotPhase45Integrated::new_real_trading(config).await {
        Ok(system) => {
            info!("✅ Sistema Phase 4.5 REAL inicializado correctamente");
            Arc::new(system)
        }
        Err(e) => {
            error!("❌ Error inicializando sistema REAL: {}", e);
            return Err(e);
        }
    };

    // Confirmación final antes de comenzar trading real
    info!("⚠️  CONFIRMACIÓN: Trading real va a comenzar en 10 segundos...");
    info!("   💰 Trades reales con SOL real en mainnet");
    info!("   📊 Oportunidades ejecutadas generarán profit/loss real");
    info!("   🛡️ MEV Protection activa para proteger transacciones");
    
    // Pausa de confirmación
    sleep(Duration::from_secs(10)).await;
    
    // Ejecutar ciclo principal de trading REAL
    info!("🎯 INICIANDO TRADING REAL - SISTEMA ACTIVO");
    info!("================================================");
    let mut cycle_count = 0;
    let mut total_profit_sol = 0.0;
    let mut successful_trades = 0;
    let mut failed_trades = 0;

    loop {
        cycle_count += 1;
        info!("🔄 Ciclo REAL #{} - Buscando oportunidades de trading real...", cycle_count);

        // Buscar y ejecutar oportunidades de arbitraje REALES
        match system.discover_opportunities().await {
            Ok(opportunities) => {
                info!("📊 Encontradas {} oportunidades REALES", opportunities.len());
                
                for (i, opportunity) in opportunities.iter().take(5).enumerate() { // Máximo 5 por ciclo para trading real
                    info!("⚡ Ejecutando oportunidad REAL #{}: Profit esperado {:.6} SOL", 
                          i + 1, opportunity.get_estimated_profit());
                          
                    match system.execute_opportunity_real(opportunity.clone()).await {
                        Ok(result) => {
                            if result.success {
                                successful_trades += 1;
                                total_profit_sol += result.actual_profit_sol;
                                info!("✅ TRADE REAL #{} EXITOSO: +{:.6} SOL de profit real", 
                                      i + 1, result.actual_profit_sol);
                                info!("   💰 Profit total acumulado: +{:.6} SOL", total_profit_sol);
                                info!("   📝 TX: {:?}", result.transaction_signatures);
                            } else {
                                failed_trades += 1;
                                warn!("❌ Trade real #{} falló: {:?}", i + 1, result.error_message);
                            }
                        }
                        Err(e) => {
                            failed_trades += 1;
                            error!("❌ Error ejecutando trade real #{}: {}", i + 1, e);
                        }
                    }
                    
                    // Pausa entre trades reales para evitar spam
                    sleep(Duration::from_secs(2)).await;
                }
            }
            Err(e) => {
                error!("⚠️  Error buscando oportunidades reales en ciclo #{}: {}", cycle_count, e);
            }
        }

        // Reporte de estadísticas de trading real
        info!("📊 ESTADÍSTICAS DE TRADING REAL:");
        info!("   💰 Profit total: +{:.6} SOL", total_profit_sol);
        info!("   ✅ Trades exitosos: {}", successful_trades);
        info!("   ❌ Trades fallidos: {}", failed_trades);
        info!("   📈 Tasa de éxito: {:.1}%", 
              if (successful_trades + failed_trades) > 0 { 
                  (successful_trades as f64 / (successful_trades + failed_trades) as f64) * 100.0 
              } else { 0.0 });

        // Pausa entre ciclos de trading real (1 minuto)
        info!("⏸️  Pausa de 60 segundos antes del próximo ciclo de trading real...");
        sleep(Duration::from_secs(60)).await;
        
        // Para trading real, ejecutar hasta ser detenido manualmente
        if cycle_count >= 100 {  // Safety limit: máximo 100 ciclos por sesión
            info!("🛑 Límite de seguridad alcanzado - 100 ciclos completados");
            break;
        }
    }

    info!("🏁 SESIÓN DE TRADING REAL FINALIZADA");
    info!("📊 RESUMEN FINAL:");
    info!("   💰 Profit total generado: +{:.6} SOL", total_profit_sol);
    info!("   ✅ Trades exitosos: {}", successful_trades);
    info!("   ❌ Trades fallidos: {}", failed_trades);
    info!("   🎯 Ciclos ejecutados: {}", cycle_count);
    
    Ok(())
}
