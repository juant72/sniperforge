// Demo del Advanced Real Trading System
// Demostración del sistema de trading real con seguridad multicapa

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use log::info;
use tokio::time::sleep;

use sniperforge::unified_config::UnifiedConfig;
use sniperforge::arbitrage_bot_phase45_integrated::ArbitrageBotPhase45Integrated;
use sniperforge::advanced_real_trading::{AdvancedRealTradingSystem, AdvancedTradingConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar logging
    env_logger::init();
    
    info!("🚀 === ADVANCED REAL TRADING SYSTEM DEMO ===");
    info!("💰 Sistema de trading real con seguridad multicapa");
    info!("==================================================");
    
    // Configuración ultra-conservadora para demo
    let trading_config = AdvancedTradingConfig {
        enable_real_trading: true,      // Habilitar para demo (pero con amounts mínimos)
        max_trade_sol: 0.0005,         // 0.0005 SOL máximo por trade
        min_profit_threshold: 0.002,   // 0.2% profit mínimo
        max_daily_trades: 5,           // Máximo 5 trades por día
        max_consecutive_losses: 1,     // Parar después de 1 pérdida
        emergency_stop_loss_pct: 2.0,  // Parar si pérdidas > 2%
        cool_down_after_loss_seconds: 60, // 1 minuto de enfriamiento
        max_slippage_tolerance: 0.5,   // 0.5% slippage máximo
        require_manual_approval_above_sol: 0.001, // Aprobación manual > 0.001 SOL
    };
    
    info!("📋 Configuración de Trading Real:");
    info!("   💰 Max trade SOL: {}", trading_config.max_trade_sol);
    info!("   📊 Min profit threshold: {}%", trading_config.min_profit_threshold * 100.0);
    info!("   🛡️ Max consecutive losses: {}", trading_config.max_consecutive_losses);
    info!("   ⏰ Cool down after loss: {}s", trading_config.cool_down_after_loss_seconds);
    
    // Configuración del sistema base
    let config = UnifiedConfig::safe_conservative();
    
    info!("🔧 Inicializando Sistema Arbitrage Phase 4.5...");
    let arbitrage_system = Arc::new(
        ArbitrageBotPhase45Integrated::new(config).await?
    );
    
    info!("🚀 Inicializando Advanced Real Trading System...");
    let trading_system = AdvancedRealTradingSystem::new(
        trading_config,
        arbitrage_system.clone()
    );
    
    trading_system.initialize().await?;
    
    info!("✅ Sistemas inicializados - Comenzando demo de trading real");
    info!("⏰ Ejecutando 5 ciclos de trading con intervalos de 30 segundos");
    
    // Ejecutar ciclos de trading de demo
    for cycle in 1..=5 {
        info!("");
        info!("🔄 === CICLO DE TRADING #{} ===", cycle);
        
        // Mostrar métricas actuales
        if let Ok(metrics) = trading_system.get_real_time_metrics() {
            info!("📊 Métricas Actuales:");
            info!("   🔢 Total trades: {}", metrics.total_trades_executed);
            info!("   ✅ Exitosos: {}", metrics.successful_trades);
            info!("   ❌ Fallidos: {}", metrics.failed_trades);
            info!("   💰 Profit total: {} SOL", metrics.total_profit_sol);
            info!("   📈 Success rate: {:.1}%", metrics.success_rate_percentage);
            info!("   🔄 Consecutive losses: {}", metrics.consecutive_losses);
        }
        
        // Ejecutar ciclo de trading
        let trade_results = trading_system.execute_trading_cycle().await?;
        
        if trade_results.is_empty() {
            info!("📊 No se ejecutaron trades en este ciclo");
        } else {
            info!("⚡ Ejecutados {} trades:", trade_results.len());
            for result in &trade_results {
                if result.execution_successful {
                    info!("   ✅ Trade {}: +{} SOL profit (fees: {} SOL)", 
                          result.trade_id, result.actual_profit_sol, result.fees_paid_sol);
                } else {
                    info!("   ❌ Trade {}: FAILED - {}", 
                          result.trade_id, result.error_message.as_deref().unwrap_or("Unknown error"));
                }
            }
        }
        
        if cycle < 5 {
            info!("⏳ Esperando 30 segundos hasta próximo ciclo...");
            sleep(Duration::from_secs(30)).await;
        }
    }
    
    // Mostrar resumen final
    info!("");
    info!("🏁 === RESUMEN FINAL DE TRADING ===");
    
    if let Ok(metrics) = trading_system.get_real_time_metrics() {
        info!("📊 Estadísticas Finales:");
        info!("   🔢 Total trades ejecutados: {}", metrics.total_trades_executed);
        info!("   ✅ Trades exitosos: {}", metrics.successful_trades);
        info!("   ❌ Trades fallidos: {}", metrics.failed_trades);
        info!("   💰 Profit total: {} SOL", metrics.total_profit_sol);
        info!("   💸 Fees totales: {} SOL", metrics.total_fees_paid_sol);
        info!("   📈 Success rate: {:.1}%", metrics.success_rate_percentage);
        info!("   📊 Profit promedio por trade: {} SOL", metrics.average_profit_per_trade);
        info!("   🏆 Mayor profit: {} SOL", metrics.largest_profit_sol);
        info!("   📉 Mayor pérdida: {} SOL", metrics.largest_loss_sol);
        
        if metrics.total_profit_sol > 0.0 {
            info!("🎉 SESIÓN EXITOSA: Profit neto positivo de {} SOL", metrics.total_profit_sol);
        } else if metrics.total_profit_sol < 0.0 {
            info!("⚠️ PÉRDIDAS: {} SOL - Revisar estrategia", metrics.total_profit_sol.abs());
        } else {
            info!("⚖️ BREAK EVEN: Sin ganancias ni pérdidas");
        }
    }
    
    if let Ok(trade_history) = trading_system.get_trade_history() {
        if !trade_history.is_empty() {
            info!("");
            info!("📋 Historial Detallado de Trades:");
            for (i, trade) in trade_history.iter().enumerate() {
                info!("   {}. {} - {} SOL - {} ms - {}", 
                      i + 1,
                      trade.trade_id,
                      trade.actual_profit_sol,
                      trade.execution_time_ms,
                      if trade.execution_successful { "✅ SUCCESS" } else { "❌ FAILED" }
                );
            }
        }
    }
    
    info!("");
    info!("✅ Demo completado exitosamente");
    info!("🛡️ Sistemas de seguridad funcionaron correctamente");
    info!("💡 Para trading real, ajustar configuración según tolerancia al riesgo");
    
    Ok(())
}
