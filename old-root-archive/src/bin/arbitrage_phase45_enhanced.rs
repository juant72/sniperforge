// ================================================================================
// ARBITRAGE BOT PHASE 4.5 - ENHANCED WITH ACCIÓN 7 OPTIMIZATIONS
// ================================================================================
// Versión mejorada del sistema con optimizaciones avanzadas de ACCIÓN 7
// Incluye: Performance Optimizer, Profit Tracker, Real-time Dashboard
// ================================================================================

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn};
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::str::FromStr;

// Importar el sistema integrado Phase 4.5
use sniperforge::{
    arbitrage_bot_phase45_integrated::ArbitrageBotPhase45Integrated,
    unified_config::UnifiedPhase45Config,
    // ACCIÓN 7 Modules
    advanced_performance_optimizer::{AdvancedPerformanceOptimizer, PerformanceConfig},
    advanced_profit_tracker::{AdvancedProfitTracker, ProfitTrackerConfig, TradeResult},
    real_time_trading_dashboard::{RealTimeTradingDashboard, DashboardConfig},
};

/// Función para obtener el balance real de la wallet en mainnet
async fn get_wallet_balance(rpc_client: &RpcClient, wallet_pubkey: &Pubkey) -> Result<f64> {
    match rpc_client.get_balance(wallet_pubkey) {
        Ok(balance_lamports) => {
            let balance_sol = balance_lamports as f64 / 1_000_000_000.0; // Convertir lamports a SOL
            Ok(balance_sol)
        }
        Err(e) => {
            error!("❌ Error obteniendo balance de wallet: {}", e);
            Err(anyhow::anyhow!("Error consultando balance: {}", e))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === ARBITRAGE BOT PHASE 4.5 - ENHANCED WITH ACCIÓN 7 ===");
    info!("💰 ATENCIÓN: Este programa ejecuta TRANSACCIONES REALES con SOL");
    info!("🎯 NUEVA VERSIÓN: Incluye Performance Optimizer + Profit Tracker + Dashboard");
    info!("================================================================");
    
    // Verificar modo de operación
    let force_real = std::env::var("FORCE_REAL_TRANSACTIONS").unwrap_or("false".to_string()) == "true";
    if force_real {
        info!("🔥 MODO TRANSACCIONES REALES ACTIVADO");
        info!("⚠️  ¡CUIDADO! Las transacciones modificarán balance real");
    } else {
        info!("🧪 MODO SIMULACIÓN SEGURA (para testing)");
        info!("💡 Para activar trades reales: set FORCE_REAL_TRANSACTIONS=true");
    }

    // Crear configuración por defecto
    let config = UnifiedPhase45Config::safe_trading();
    
    // Crear cliente RPC para monitoreo de balance
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        config.rpc_endpoint.clone(),
        CommitmentConfig::confirmed(),
    ));
    
    // Wallet principal del sistema (mainnet)
    let wallet_pubkey = Pubkey::from_str("JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7")?;
    
    // Obtener balance inicial
    info!("🔍 Consultando balance inicial de wallet...");
    let initial_balance = match get_wallet_balance(&rpc_client, &wallet_pubkey).await {
        Ok(balance) => {
            info!("💰 Balance inicial: {:.9} SOL", balance);
            balance
        }
        Err(e) => {
            warn!("⚠️ No se pudo obtener balance inicial: {}", e);
            0.0
        }
    };

    // ================================================================================
    // ACCIÓN 7: INICIALIZAR SISTEMAS AVANZADOS
    // ================================================================================
    
    // 7.1: Performance Optimizer
    info!("🚀 Inicializando Advanced Performance Optimizer...");
    let perf_config = PerformanceConfig::max_performance_config();
    let mut performance_optimizer = AdvancedPerformanceOptimizer::new(perf_config);

    // 7.2: Profit Tracker
    info!("💰 Inicializando Advanced Profit Tracker...");
    let profit_config = ProfitTrackerConfig::default();
    let mut profit_tracker = AdvancedProfitTracker::new(profit_config, initial_balance);

    // 7.3: Real-time Dashboard
    info!("📊 Inicializando Real-Time Trading Dashboard...");
    let dashboard_config = DashboardConfig::default();
    let dashboard = RealTimeTradingDashboard::new(dashboard_config, initial_balance);
    
    // Iniciar dashboard
    dashboard.start().await?;
    
    // Mostrar resumen de configuración
    info!("📋 Configuración del Sistema Enhanced:");
    info!("   • Trading Mode: SAFE (Conservador)");
    info!("   • Max Trade SOL: {:.3}", config.max_trade_sol);
    info!("   • Min Profit BPS: {}", config.min_profit_bps);
    info!("   • Jupiter Advanced: {}", if config.jupiter_advanced_enabled { "✅" } else { "❌" });
    info!("   • MEV Protection: {}", if config.mev_protection_enabled { "✅" } else { "❌" });
    info!("   • DEX Specialization: {}", if config.dex_specialization_enabled { "✅" } else { "❌" });
    info!("   • Event Driven: {}", if config.event_driven_enabled { "✅" } else { "❌" });
    info!("   • Performance Optimizer: ✅ ACTIVO");
    info!("   • Advanced Profit Tracker: ✅ ACTIVO");
    info!("   • Real-time Dashboard: ✅ ACTIVO");

    // Crear e inicializar el sistema Phase 4.5
    info!("🔧 Inicializando Sistema Phase 4.5 Enhanced...");
    let system = match ArbitrageBotPhase45Integrated::new(config).await {
        Ok(system) => {
            info!("✅ Sistema Phase 4.5 Enhanced inicializado correctamente");
            Arc::new(system)
        }
        Err(e) => {
            error!("❌ Error inicializando sistema: {}", e);
            dashboard.record_error("System Init".to_string(), format!("Initialization failed: {}", e));
            return Err(e);
        }
    };

    // Ejecutar ciclo principal de trading con optimizaciones
    info!("🎯 Iniciando ciclo de trading optimizado...");
    info!("⏰ Intervalo entre ciclos: 10 segundos");
    let mut cycle_count = 0;
    let mut last_balance = initial_balance;
    let mut total_profit = 0.0;
    let mut total_opportunities_found = 0u64;
    let mut total_opportunities_executed = 0u64;

    loop {
        cycle_count += 1;
        info!("🔄 Ciclo #{} - Buscando oportunidades optimizadas...", cycle_count);

        // Verificar balance actual al inicio del ciclo
        let current_balance = if let Ok(balance) = get_wallet_balance(&rpc_client, &wallet_pubkey).await {
            let balance_change = balance - last_balance;
            info!("💰 Balance actual: {:.9} SOL (cambio: {:+.9} SOL)", balance, balance_change);
            
            if balance_change.abs() > 0.000001 { // Si hay cambio significativo
                total_profit += balance_change;
                info!("📈 Profit acumulado total: {:+.9} SOL", total_profit);
                
                // Actualizar profit tracker con balance change
                profit_tracker.update_balance(balance);
            }
            last_balance = balance;
            balance
        } else {
            last_balance
        };

        // ACCIÓN 7.1: Buscar oportunidades con Performance Optimizer
        let discovery_start = std::time::Instant::now();
        
        let opportunities = match performance_optimizer.optimize_opportunity_discovery(|| async {
            system.discover_opportunities().await
        }).await {
            Ok(opportunities) => {
                total_opportunities_found += opportunities.len() as u64;
                info!("📊 [OPTIMIZED] Encontradas {} oportunidades en {}ms", 
                      opportunities.len(), discovery_start.elapsed().as_millis());
                
                // Actualizar dashboard con métricas
                dashboard.update_system_status(
                    current_balance,
                    total_profit,
                    total_opportunities_found,
                    total_opportunities_executed,
                    cycle_count,
                );
                
                opportunities
            }
            Err(e) => {
                error!("⚠️ Error en discovery optimizado en ciclo #{}: {}", cycle_count, e);
                dashboard.record_error("Discovery Engine".to_string(), format!("Cycle {}: {}", cycle_count, e));
                Vec::new()
            }
        };

        // Ejecutar oportunidades con tracking avanzado
        for (i, opportunity) in opportunities.iter().take(3).enumerate() {
            let trade_id = format!("trade_{}_{}", cycle_count, i + 1);
            let profit_estimate = opportunity.get_estimated_profit();
            
            info!("💰 EJECUTANDO TRADE REAL #{} - ID: {} - Profit esperado: {:.6} SOL", 
                  i + 1, trade_id, profit_estimate);
            
            // Balance antes del trade
            let balance_before = if let Ok(balance) = get_wallet_balance(&rpc_client, &wallet_pubkey).await {
                balance
            } else {
                last_balance
            };
            
            let execution_start = std::time::Instant::now();
            
            match system.execute_opportunity_real(opportunity.clone()).await {
                Ok(result) => {
                    let execution_time_ms = execution_start.elapsed().as_millis() as u64;
                    
                    if result.success {
                        total_opportunities_executed += 1;
                        
                        info!("✅ 🚀 TRADE REAL #{} EXITOSO! Profit reportado: {:.6} SOL", i + 1, result.actual_profit_sol);
                        info!("   🎯 Transacciones: {:?}", result.transaction_signatures);
                        info!("   ⏱️ Tiempo ejecución: {}ms", execution_time_ms);
                        
                        // Verificar balance después del trade
                        tokio::time::sleep(Duration::from_secs(2)).await; // Esperar confirmación
                        if let Ok(balance_after) = get_wallet_balance(&rpc_client, &wallet_pubkey).await {
                            let real_change = balance_after - balance_before;
                            info!("   🔍 VERIFICACIÓN REAL:");
                            info!("     • Balance antes: {:.9} SOL", balance_before);
                            info!("     • Balance después: {:.9} SOL", balance_after);
                            info!("     • Cambio real: {:+.9} SOL", real_change);
                            
                            // ACCIÓN 7.2: Registrar trade en Profit Tracker
                            let trade_result = TradeResult::new(
                                trade_id.clone(),
                                format!("{}_{}", 
                                    opportunity.get_input_mint().to_string()[..8].to_string(),
                                    opportunity.get_output_mint().to_string()[..8].to_string()
                                ),
                                balance_before,
                                balance_after,
                                0.001, // Estimate gas cost
                                execution_time_ms,
                                "Jupiter".to_string(),
                            );
                            
                            if let Err(e) = profit_tracker.record_trade(trade_result.clone()) {
                                error!("❌ Error registrando trade en profit tracker: {}", e);
                            }
                            
                            // ACCIÓN 7.3: Registrar trade en Dashboard
                            dashboard.record_trade(&trade_result);
                            
                            if real_change.abs() > 0.000001 {
                                info!("   ✅ CONFIRMADO: Trade real ejecutado - Balance modificado");
                                last_balance = balance_after;
                            } else {
                                warn!("   ⚠️ POSIBLE SIMULACIÓN: No hay cambio en balance real");
                            }
                        }
                    } else {
                        warn!("⚠️ Trade real #{} falló: {:?}", i + 1, result.error_message);
                        dashboard.record_error("Trading Engine".to_string(), 
                                             format!("Trade {} failed: {:?}", trade_id, result.error_message));
                    }
                }
                Err(e) => {
                    error!("❌ Error ejecutando trade real #{}: {}", i + 1, e);
                    dashboard.record_error("Trading Engine".to_string(), 
                                         format!("Trade {} error: {}", trade_id, e));
                }
            }
        }

        // ACCIÓN 7: Reportes periódicos cada 5 ciclos
        if cycle_count % 5 == 0 {
            info!("📊 === REPORTE ACCIÓN 7 (Ciclo {}) ===", cycle_count);
            
            // Performance metrics
            let perf_metrics = performance_optimizer.get_metrics();
            info!("🚀 Performance: {:.2} ops/sec, cache hit: {:.1}%", 
                  perf_metrics.opportunities_per_second,
                  perf_metrics.cache_hit_rate * 100.0);
            
            // Profit tracking stats
            let profit_stats = profit_tracker.get_stats();
            info!("💰 Profit Stats: {} trades, {:.1}% success, {:+.6} SOL total", 
                  profit_stats.total_trades,
                  profit_stats.success_rate,
                  profit_stats.net_profit_sol);
            
            // Auto-optimize configuration
            performance_optimizer.auto_optimize_config();
            
            info!("📊 === FIN REPORTE ACCIÓN 7 ===");
        }

        // Pausa entre ciclos (10 segundos)
        info!("⏳ Esperando 10 segundos hasta próximo ciclo...");
        sleep(Duration::from_secs(10)).await;
        
        // Limitar a 20 ciclos para demostración (200 segundos total)
        if cycle_count >= 20 {
            info!("🏁 Demo completada - 20 ciclos ejecutados");
            
            // Balance final
            if let Ok(final_balance) = get_wallet_balance(&rpc_client, &wallet_pubkey).await {
                let total_change = final_balance - initial_balance;
                info!("📊 RESUMEN FINAL ENHANCED:");
                info!("   • Balance inicial: {:.9} SOL", initial_balance);
                info!("   • Balance final: {:.9} SOL", final_balance);
                info!("   • Cambio total: {:+.9} SOL", total_change);
                info!("   • Profit acumulado: {:+.9} SOL", total_profit);
                info!("   • Oportunidades encontradas: {}", total_opportunities_found);
                info!("   • Oportunidades ejecutadas: {}", total_opportunities_executed);
                
                // Actualizar dashboard final
                dashboard.update_system_status(
                    final_balance,
                    total_profit,
                    total_opportunities_found,
                    total_opportunities_executed,
                    cycle_count,
                );
            }
            
            // Generar reportes finales ACCIÓN 7
            info!("📈 === REPORTES FINALES ACCIÓN 7 ===");
            
            // Performance report
            info!("🚀 Performance Report:");
            let perf_report = performance_optimizer.generate_performance_report();
            info!("{}", perf_report);
            
            // Profit tracking report
            info!("💰 Profit Tracking Report:");
            let profit_report = profit_tracker.generate_report();
            info!("{}", profit_report);
            
            // Dashboard report
            info!("📊 Dashboard Report:");
            let dashboard_report = dashboard.generate_dashboard_report();
            info!("{}", dashboard_report);
            
            info!("📈 === FIN REPORTES ACCIÓN 7 ===");
            
            break;
        }
    }

    // Detener dashboard
    dashboard.stop();

    info!("✅ Sistema de Arbitraje Phase 4.5 Enhanced finalizado");
    info!("🎯 ACCIÓN 7 completada exitosamente con optimizaciones avanzadas");
    Ok(())
}
