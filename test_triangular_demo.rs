use sniperforge_core::{
    init,
    trading::{TriangularArbitrageEngine, TriangularArbitrageConfig},
};
use tokio::time::{sleep, Duration};
use tracing::{info, error};
use anyhow::Result;

/// Demostración del Triangular Arbitrage Engine migrado del bot que funciona
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize SniperForge Core
    init()?;
    
    info!("🚀 Demostración del Triangular Arbitrage Engine");
    info!("📚 Funcionalidad migrada exitosamente del arbitrage_phase45_clean.rs");
    
    // Initialize Triangular Arbitrage Engine con configuración personalizada
    let triangular_config = TriangularArbitrageConfig {
        enabled: true,
        min_profit_threshold: 0.002, // 0.2% mínimo para detectar más oportunidades
        max_cost_bps: 500,           // 5% máximo costos
        max_execution_risk_score: 0.8,
        min_liquidity_usd: 30000.0,  // $30K mínimo (más permisivo)
        max_execution_duration_ms: 30000, // 30 segundos
        max_dexs_involved: 3,
    };
    
    let mut triangular_engine = TriangularArbitrageEngine::new(Some(triangular_config));
    info!("✅ Triangular Arbitrage Engine inicializado con protección anti-circular");
    
    info!("🎯 Iniciando búsqueda de oportunidades triangulares...");
    
    // Ejecutar 3 ciclos de búsqueda para demostrar funcionalidad
    for cycle in 1..=3 {
        info!("\n🔄 ===== CICLO #{} =====", cycle);
        
        let cycle_start = std::time::Instant::now();
        
        // Buscar oportunidades triangulares
        match triangular_engine.find_triangular_opportunities().await {
            Ok(opportunities) => {
                let discovery_time = cycle_start.elapsed().as_millis();
                
                if opportunities.is_empty() {
                    info!("⏳ No se detectaron oportunidades triangulares en este ciclo");
                } else {
                    info!("🎉 ¡Encontradas {} oportunidades triangulares!", opportunities.len());
                    
                    for (i, opp) in opportunities.iter().enumerate() {
                        info!("📈 Oportunidad #{}: {}", i + 1, opp.id);
                        info!("   💰 Profit estimado: {:.4}% ({:.6} SOL en decimal)", 
                              opp.estimated_net_profit * 100.0,
                              opp.estimated_net_profit);
                        info!("   ⚡ Risk Score: {:.3}/1.0", opp.execution_risk_score);
                        info!("   💧 Liquidez mínima: ${:.0}K", opp.liquidity_constraint / 1000.0);
                        info!("   💸 Costos totales: {} BPS ({:.2}%)", 
                              opp.total_cost_bps, 
                              opp.total_cost_bps as f64 / 100.0);
                        info!("   ⏱️  Duración estimada: {}ms", opp.estimated_duration_ms);
                        
                        // Mostrar el path completo
                        info!("   🔺 Path triangular:");
                        for (j, hop) in opp.path.iter().enumerate() {
                            info!("      {}. {} → {} (via {}, fee: {} BPS, rate: {:.6})", 
                                  j + 1,
                                  hop.from_token, 
                                  hop.to_token,
                                  hop.dex_name,
                                  hop.swap_fee_bps,
                                  hop.exchange_rate);
                        }
                        
                        // Evaluación de calidad
                        let quality = if opp.estimated_net_profit > 0.01 && opp.execution_risk_score < 0.5 {
                            "🟢 EXCELENTE"
                        } else if opp.estimated_net_profit > 0.005 && opp.execution_risk_score < 0.7 {
                            "🟡 BUENA"
                        } else {
                            "🔴 MARGINAL"
                        };
                        
                        info!("   🎯 Calidad: {}", quality);
                        
                        if opp.estimated_net_profit > 0.008 {
                            info!("   ✅ OPORTUNIDAD DE ALTA CALIDAD - En producción ejecutaría");
                        }
                        
                        info!(""); // Separador
                    }
                }
                
                // Mostrar estadísticas del motor
                let stats = triangular_engine.get_statistics();
                info!("📊 Estadísticas del Motor Triangular:");
                info!("   🔍 Paths analizados: {}", stats.total_paths_analyzed);
                info!("   💾 Precios en cache: {}", stats.cached_prices);
                info!("   🚨 Patrones sospechosos detectados: {}", stats.suspicious_patterns);
                info!("   🗺️  Grafo de tokens: {} nodos", stats.token_graph_size);
                info!("   ⚙️  Config: min_profit={:.2}%, max_cost={}BPS", 
                      stats.config.min_profit_threshold * 100.0,
                      stats.config.max_cost_bps);
                
                info!("⏱️  Tiempo de discovery: {}ms", discovery_time);
            }
            Err(e) => {
                error!("❌ Error en triangular discovery: {}", e);
            }
        }
        
        if cycle < 3 {
            info!("😴 Esperando 8 segundos antes del próximo ciclo...");
            sleep(Duration::from_secs(8)).await;
        }
    }
    
    info!("\n🎊 ===== DEMOSTRACIÓN COMPLETADA =====");
    info!("✅ El Triangular Arbitrage Engine ha sido migrado exitosamente");
    info!("🔧 Funcionalidad completa:");
    info!("   • Detección de paths triangulares válidos");
    info!("   • Protección anti-circular avanzada"); 
    info!("   • Cálculos de profit reales con fees");
    info!("   • Estimación de riesgo y liquidez");
    info!("   • Integración con APIs de precios");
    info!("   • Estadísticas detalladas de rendimiento");
    info!("🚀 Listo para integración con ejecución real!");
    
    Ok(())
}
