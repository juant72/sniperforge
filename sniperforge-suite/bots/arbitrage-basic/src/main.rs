use sniperforge_core::{
    init,
    SniperForgeConfig,
    apis::{RealPriceFeeds},
    analytics::{PatternRecognitionEngine, OpportunityPattern},
};
use tokio::time::{sleep, Duration};
use tracing::{info, error};
use anyhow::Result;
use chrono::Utc;

/// Basic arbitrage bot using migrated functionality from working bot
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize SniperForge Core
    init()?;
    
    info!("🚀 Iniciando Arbitrage Bot Básico con SniperForge Core Library");
    info!("📚 Usando funcionalidad migrada del bot que funciona");
    
    // Load configuration (using default for now)
    let config = SniperForgeConfig::default();
    let simple_config = config.to_simple_config();
    
    info!("✅ Configuración cargada");
    
    // Initialize Real Price Feeds (migrated from working bot)
    let real_price_feeds = RealPriceFeeds::new();
    info!("✅ Real Price Feeds inicializado");
    
    // Initialize ML Pattern Recognition Engine (migrated from working bot)
    let mut ml_engine = PatternRecognitionEngine::new();
    info!("✅ ML Pattern Recognition Engine inicializado");
    
    // Initialize Arbitrage Engine with migrated functionality
    // Note: For now we'll simulate the price feed manager
    // This will be enhanced in next iterations
    info!("🔄 Simulando inicialización completa del Arbitrage Engine...");
    
    let mut cycle_count = 0;
    
    info!("🎯 Iniciando ciclo principal de arbitraje...");
    
    // Main arbitrage loop
    loop {
        cycle_count += 1;
        let cycle_start = std::time::Instant::now();
        
        info!("🔄 Ciclo #{} - Buscando oportunidades de arbitraje...", cycle_count);
        
        // Use Real Price Feeds to find opportunities (migrated functionality)
        match real_price_feeds.find_real_arbitrage_opportunities().await {
            Ok(opportunities) => {
                let discovery_time = cycle_start.elapsed().as_millis() as u64;
                
                if opportunities.is_empty() {
                    info!("⏳ Sin oportunidades detectadas en este ciclo");
                } else {
                    info!("🎉 Encontradas {} oportunidades reales", opportunities.len());
                    
                    // Process each opportunity with ML analysis
                    for opportunity in &opportunities {
                        // Extract data for ML analysis
                        let token_pair = format!("{}-SOL", opportunity.token_symbol);
                        let profit_percentage = opportunity.price_difference_pct;
                        let liquidity = opportunity.min_liquidity_usd;
                        let volatility = 0.02; // Estimated volatility
                        
                        // ML Analysis (migrated from working bot)
                        let ml_score = ml_engine.score_opportunity(
                            &token_pair,
                            profit_percentage,
                            liquidity,
                            volatility,
                        );
                        
                        info!("🧠 ML Analysis - {}: Score {:.3}, Confidence: {:.1}%", 
                              opportunity.token_symbol, 
                              ml_score.composite_score,
                              opportunity.confidence_score * 100.0);
                        
                        // Record opportunity pattern for ML learning
                        let pattern = OpportunityPattern {
                            timestamp: Utc::now(),
                            token_pair: token_pair.clone(),
                            profit_percentage,
                            execution_time_ms: 2000, // Estimated
                            market_volatility: volatility,
                            liquidity_level: liquidity,
                            success: ml_score.composite_score > 0.5, // Estimated success
                            dex_source: opportunity.dex_a.dex_name.clone(),
                        };
                        
                        ml_engine.record_opportunity(pattern);
                        
                        // Decision logic (from migrated functionality)
                        if ml_score.composite_score > 0.5 && profit_percentage > 0.01 {
                            info!("✅ OPORTUNIDAD VÁLIDA: {} - Profit: {:.2}%, ML Score: {:.3}", 
                                  opportunity.token_symbol, profit_percentage, ml_score.composite_score);
                            
                            // In a real implementation, this would execute the trade
                            info!("🚀 Simulando ejecución de arbitraje...");
                            
                            // Mark ML prediction result
                            ml_engine.mark_prediction_result(true); // Simulate success
                        } else {
                            info!("⚠️ Oportunidad marginal: {} - Profit: {:.2}%, ML Score: {:.3}", 
                                  opportunity.token_symbol, profit_percentage, ml_score.composite_score);
                            ml_engine.mark_prediction_result(false);
                        }
                    }
                }
                
                // Display performance stats every 10 cycles
                if cycle_count % 10 == 0 {
                    display_performance_dashboard(&ml_engine, cycle_count, discovery_time);
                }
            }
            Err(e) => {
                error!("❌ Error en discovery: {}", e);
                info!("🔄 Continuando en próximo ciclo - errores temporales son normales");
            }
        }
        
        // Wait before next cycle
        sleep(Duration::from_secs(10)).await;
    }
}

/// Display performance dashboard (migrated from working bot)
fn display_performance_dashboard(
    ml_engine: &PatternRecognitionEngine, 
    cycle_count: u32, 
    last_discovery_time: u64
) {
    let ml_stats = ml_engine.get_performance_stats();
    
    info!("╔══════════════════════════════════════════════════════════════╗");
    info!("║                    ARBITRAGE BOT DASHBOARD                   ║");
    info!("║                  (SniperForge Core Library)                  ║");
    info!("╠══════════════════════════════════════════════════════════════╣");
    info!("║ 🔄 Cycles Completed: {}                                ║", cycle_count);
    info!("║ ⚡ Last Discovery Time: {}ms                          ║", last_discovery_time);
    info!("║ 🧠 ML Predictions: {:.0}                                ║", 
          ml_stats.get("predictions_made").unwrap_or(&0.0));
    info!("║ 🎯 ML Accuracy: {:.1}%                                  ║", 
          ml_stats.get("accuracy").unwrap_or(&0.0) * 100.0);
    info!("║ 📊 Data Points: {:.0}                                   ║", 
          ml_stats.get("data_points").unwrap_or(&0.0));
    info!("║ ⚖️ Profit Weight: {:.3}                                 ║", 
          ml_stats.get("profit_weight").unwrap_or(&0.0));
    info!("║ ⏰ Timing Weight: {:.3}                                 ║", 
          ml_stats.get("timing_weight").unwrap_or(&0.0));
    info!("╚══════════════════════════════════════════════════════════════╝");
}
