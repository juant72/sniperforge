// ================================================================================
// TRADING REAL SIMPLIFICADO - 100% FUNCIONAL
// ================================================================================
// Sistema de arbitraje configurado para trading real en mainnet
// ================================================================================

use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use tracing::{info, error, warn};
use anyhow::Result;

// Importar el sistema existente que ya funciona
use sniperforge::{
    arbitrage_bot_phase45_integrated::ArbitrageBotPhase45Integrated,
    unified_config::UnifiedPhase45Config
};

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("💰 TRADING REAL ACTIVADO - SISTEMA SNIPERFORGE");
    info!("================================================");
    info!("🌐 Network: MAINNET (Trading con dinero real)");
    info!("⚠️  ATENCIÓN: Trades reales con SOL real activado");
    info!("================================================");

    // Crear configuración para trading real conservador
    let mut config = UnifiedPhase45Config::safe_trading();
    
    // Ajustes para trading real
    config.max_trade_sol = 0.02;  // Máximo 0.02 SOL por trade para seguridad
    config.min_profit_bps = 30;   // Mínimo 0.30% profit para cubrir fees
    config.mev_protection_enabled = true;  // MEV protection obligatorio
    config.jupiter_advanced_enabled = true; // Jupiter para mejores rutas
    
    // Mostrar configuración real
    info!("💼 CONFIGURACIÓN DE TRADING REAL:");
    info!("   💰 Max Trade: {:.3} SOL (DINERO REAL)", config.max_trade_sol);
    info!("   📈 Min Profit: {} BPS ({}%)", config.min_profit_bps, config.min_profit_bps as f64 / 100.0);
    info!("   🛡️ MEV Protection: ✅ ACTIVO");
    info!("   🚀 Jupiter Advanced: ✅ ACTIVO");
    info!("   🎯 Mode: CONSERVATIVE REAL TRADING");

    // Verificación de seguridad
    info!("🔐 Verificaciones de seguridad para trading real...");
    info!("   ✅ Montos limitados a 0.02 SOL máximo");
    info!("   ✅ Profit mínimo aumentado para cubrir fees");
    info!("   ✅ MEV protection activa");
    info!("   ✅ Configuración conservadora activada");

    // Inicializar sistema
    info!("🔧 Inicializando sistema para TRADING REAL...");
    let system = match ArbitrageBotPhase45Integrated::new(config).await {
        Ok(system) => {
            info!("✅ Sistema REAL inicializado correctamente");
            Arc::new(system)
        }
        Err(e) => {
            error!("❌ Error inicializando sistema REAL: {}", e);
            return Err(e);
        }
    };

    // Confirmación final
    info!("⚠️  CONFIRMACIÓN FINAL: Trading real iniciará en 15 segundos...");
    info!("   💰 Cada trade usará SOL real de la wallet");
    info!("   📊 Ganancias y pérdidas serán reales");
    info!("   🛑 Presiona Ctrl+C para cancelar");
    
    // Countdown de seguridad
    for i in (1..=15).rev() {
        info!("   ⏰ Iniciando en {} segundos...", i);
        sleep(Duration::from_secs(1)).await;
    }

    info!("🎯 TRADING REAL INICIADO - SISTEMA ACTIVO");
    info!("==========================================");
    
    let mut cycle_count = 0;
    let mut total_profit_sol = 0.0;
    let mut successful_trades = 0;
    let mut failed_trades = 0;
    let start_time = Instant::now();

    loop {
        cycle_count += 1;
        info!("");
        info!("🔄 CICLO REAL #{} - Buscando oportunidades rentables...", cycle_count);

        match system.discover_opportunities().await {
            Ok(opportunities) => {
                let filtered_opportunities: Vec<_> = opportunities.into_iter()
                    .filter(|opp| opp.get_estimated_profit() >= 0.0006) // Mínimo 0.0006 SOL profit
                    .take(2) // Máximo 2 oportunidades por ciclo para trading real
                    .collect();

                info!("📊 Oportunidades RENTABLES encontradas: {}", filtered_opportunities.len());
                
                if filtered_opportunities.is_empty() {
                    info!("   ⏸️  No hay oportunidades rentables en este ciclo");
                } else {
                    for (i, opportunity) in filtered_opportunities.iter().enumerate() {
                        let profit_expected = opportunity.get_estimated_profit();
                        info!("⚡ EJECUTANDO TRADE REAL #{}: Profit esperado {:.6} SOL", 
                              i + 1, profit_expected);
                        
                        let trade_start = Instant::now();
                        match system.execute_opportunity(opportunity.clone()).await {
                            Ok(result) => {
                                let execution_time = trade_start.elapsed();
                                if result.success {
                                    successful_trades += 1;
                                    total_profit_sol += result.actual_profit_sol;
                                    info!("✅ TRADE REAL EXITOSO #{}: +{:.6} SOL en {:.2}s", 
                                          i + 1, result.actual_profit_sol, execution_time.as_secs_f64());
                                    info!("   💰 Profit acumulado: +{:.6} SOL", total_profit_sol);
                                    
                                    if !result.transaction_signatures.is_empty() {
                                        info!("   📝 TX Hash: {}", result.transaction_signatures[0]);
                                    }
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
                        
                        // Pausa entre trades para evitar spam de la red
                        if i < filtered_opportunities.len() - 1 {
                            sleep(Duration::from_secs(3)).await;
                        }
                    }
                }
            }
            Err(e) => {
                error!("⚠️  Error buscando oportunidades en ciclo #{}: {}", cycle_count, e);
            }
        }

        // Estadísticas en tiempo real
        let runtime = start_time.elapsed();
        let success_rate = if (successful_trades + failed_trades) > 0 { 
            (successful_trades as f64 / (successful_trades + failed_trades) as f64) * 100.0 
        } else { 0.0 };

        info!("📊 ESTADÍSTICAS TRADING REAL (Runtime: {:.1}m):", runtime.as_secs_f64() / 60.0);
        info!("   💰 Profit total: +{:.6} SOL", total_profit_sol);
        info!("   ✅ Trades exitosos: {} | ❌ Fallidos: {}", successful_trades, failed_trades);
        info!("   📈 Tasa de éxito: {:.1}%", success_rate);
        info!("   🔄 Ciclos completados: {}", cycle_count);

        // Pausa entre ciclos (90 segundos para trading real)
        info!("⏸️  Pausa 90s antes del próximo ciclo de trading real...");
        sleep(Duration::from_secs(90)).await;
        
        // Para trading real, ejecutar hasta ser detenido manualmente
        if cycle_count >= 50 {  // Safety limit: máximo 50 ciclos por sesión
            info!("🛑 Límite de seguridad alcanzado - 50 ciclos completados");
            break;
        }
    }

    let total_runtime = start_time.elapsed();
    info!("");
    info!("🏁 SESIÓN DE TRADING REAL FINALIZADA");
    info!("====================================");
    info!("⏰ Duración total: {:.1} minutos", total_runtime.as_secs_f64() / 60.0);
    info!("💰 Profit total generado: +{:.6} SOL", total_profit_sol);
    info!("✅ Trades exitosos: {}", successful_trades);
    info!("❌ Trades fallidos: {}", failed_trades);
    info!("📈 Tasa de éxito final: {:.1}%", 
          if (successful_trades + failed_trades) > 0 { 
              (successful_trades as f64 / (successful_trades + failed_trades) as f64) * 100.0 
          } else { 0.0 });
    info!("🎯 Ciclos ejecutados: {}", cycle_count);
    
    if total_profit_sol > 0.0 {
        info!("🎉 TRADING REAL EXITOSO - PROFIT GENERADO");
    } else {
        info!("📊 Trading real completado - Sin profit neto");
    }
    
    Ok(())
}
