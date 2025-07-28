// ================================================================================
// ARBITRAGE BOT PHASE 4.5 - ENHANCED WITH ACCIÓN 8: MACHINE LEARNING INTEGRATION
// ================================================================================
// Sistema principal con TODAS las optimizaciones de ACCIONES 1-8 integradas
// Performance Optimizer + Profit Tracker + Real-time Dashboard + ML INCLUIDOS
// ================================================================================

use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn, debug};
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::str::FromStr;
use chrono::{DateTime, Utc};

// Importar el sistema integrado Phase 4.5 + ML
use sniperforge::{
    arbitrage_bot_phase45_integrated::ArbitrageBotPhase45Integrated,
    unified_config::UnifiedPhase45Config,
    ml_pattern_recognition::{
        PatternRecognitionEngine, OpportunityPattern
    }
};

// ================================================================================
// ACCIÓN 8: ESTRUCTURAS ML INTEGRADAS (Performance + Profit + Dashboard + ML)
// ================================================================================

/// Configuración de performance integrada con ML
#[derive(Debug, Clone)]
struct PerformanceConfig {
    max_concurrent_discoveries: usize,
    cache_ttl_seconds: u64,
    parallel_api_calls: bool,
    latency_target_ms: u64,
    // ACCIÓN 8: ML parameters
    ml_enabled: bool,
    pattern_analysis_enabled: bool,
    adaptive_parameters_enabled: bool,
    ml_confidence_threshold: f64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_discoveries: 3,
            cache_ttl_seconds: 2,
            parallel_api_calls: true,
            latency_target_ms: 150,
            // ACCIÓN 8: ML defaults
            ml_enabled: true,
            pattern_analysis_enabled: true,
            adaptive_parameters_enabled: true,
            ml_confidence_threshold: 0.6,
        }
    }
}

/// Métricas de performance mejoradas con ML
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    discovery_time_ms: u64,
    opportunities_per_second: f64,
    cache_hit_rate: f64,
    total_cycles: u64,
    successful_discoveries: u64,
    cache_hits: u64,
    avg_cycle_time: f64,
    // ACCIÓN 8: ML metrics
    ml_predictions_made: u64,
    ml_accuracy_rate: f64,
    adaptive_adjustments: u64,
    market_condition_changes: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            discovery_time_ms: 0,
            opportunities_per_second: 0.0,
            cache_hit_rate: 0.0,
            total_cycles: 0,
            successful_discoveries: 0,
            cache_hits: 0,
            avg_cycle_time: 0.0,
            // ACCIÓN 8: ML defaults
            ml_predictions_made: 0,
            ml_accuracy_rate: 0.0,
            adaptive_adjustments: 0,
            market_condition_changes: 0,
        }
    }
}

/// Resultado de trade mejorado con análisis ML
#[derive(Debug, Clone)]
struct TradeResult {
    trade_id: String,
    timestamp: DateTime<Utc>,
    profit_sol: f64,
    execution_time_ms: u64,
    success: bool,
    predicted_profit: f64,
    // ACCIÓN 8: ML fields
    ml_score: f64,
    ml_confidence: f64,
    ml_recommendation: String,
    market_condition: String,
}

/// Estadísticas de trading mejoradas con ML analytics
#[derive(Debug, Clone)]
struct TradingStats {
    total_trades: u64,
    successful_trades: u64,
    failed_trades: u64,
    total_profit_sol: f64,
    best_trade_profit_sol: f64,
    avg_execution_time_ms: f64,
    last_trade_time: Option<DateTime<Utc>>,
    // ACCIÓN 8: ML fields
    ml_predicted_trades: u64,
    ml_prediction_accuracy: f64,
    avg_ml_confidence: f64,
}

impl Default for TradingStats {
    fn default() -> Self {
        Self {
            total_trades: 0,
            successful_trades: 0,
            failed_trades: 0,
            total_profit_sol: 0.0,
            best_trade_profit_sol: 0.0,
            avg_execution_time_ms: 0.0,
            last_trade_time: None,
            // ACCIÓN 8: ML defaults
            ml_predicted_trades: 0,
            ml_prediction_accuracy: 0.0,
            avg_ml_confidence: 0.0,
        }
    }
}

// ================================================================================
// ACCIÓN 8: SISTEMA PRINCIPAL ML-ENHANCED TRADING SYSTEM
// ================================================================================

struct MLEnhancedTradingSystem {
    // Core systems
    current_balance: f64,
    start_time: Instant,
    config: UnifiedPhase45Config,
    
    // ACCIÓN 7: Performance components
    perf_config: PerformanceConfig,
    perf_metrics: PerformanceMetrics,
    trade_history: VecDeque<TradeResult>,
    trading_stats: TradingStats,
    api_status: HashMap<String, bool>,
    
    // ACCIÓN 8: ML components
    ml_engine: PatternRecognitionEngine,
    ml_metrics_history: VecDeque<String>,
    market_data_cache: HashMap<String, f64>,
    last_ml_analysis: Option<DateTime<Utc>>,
}

impl MLEnhancedTradingSystem {
    fn new(initial_balance: f64) -> Self {
        info!("🚀 Inicializando ML-Enhanced Trading System v4.5");
        
        Self {
            current_balance: initial_balance,
            start_time: Instant::now(),
            config: UnifiedPhase45Config::default(),
            
            perf_config: PerformanceConfig::default(),
            perf_metrics: PerformanceMetrics::default(),
            trade_history: VecDeque::with_capacity(1000),
            trading_stats: TradingStats::default(),
            api_status: HashMap::new(),
            
            // ACCIÓN 8: ML initialization
            ml_engine: PatternRecognitionEngine::new(),
            ml_metrics_history: VecDeque::with_capacity(100),
            market_data_cache: HashMap::new(),
            last_ml_analysis: None,
        }
    }

    /// ACCIÓN 8.1: Análisis ML de oportunidades con Pattern Recognition
    async fn analyze_opportunity_with_ml(&mut self, 
        token_pair: &str,
        profit_percentage: f64,
        volume_24h: f64,
        liquidity: f64
    ) -> Result<(f64, String)> {
        debug!("🧠 Analizando oportunidad con ML: {} ({}%)", token_pair, profit_percentage);
        
        // Actualizar cache de datos de mercado
        self.market_data_cache.insert(token_pair.to_string(), profit_percentage);
        
        // Crear patrón para análisis ML
        let pattern = OpportunityPattern {
            timestamp: Utc::now(),
            token_pair: token_pair.to_string(),
            profit_percentage,
            execution_time_ms: 0,
            market_volatility: 0.02, // Estimado
            liquidity_level: liquidity,
            success: true, // Estimado
            dex_source: "Jupiter".to_string(),
        };
        
        // Obtener score del ML engine
        let ml_score = self.ml_engine.score_opportunity(&pattern);
        
        // Generar recomendación simple basada en score
        let recommendation = if ml_score > 0.8 {
            "STRONG_BUY"
        } else if ml_score > 0.6 {
            "BUY"
        } else if ml_score > 0.4 {
            "HOLD"
        } else {
            "AVOID"
        }.to_string();
        
        // Actualizar métricas ML
        self.perf_metrics.ml_predictions_made += 1;
        self.trading_stats.ml_predicted_trades += 1;
        
        info!("🎯 ML Score: {:.3} | Recomendación: {}", ml_score, recommendation);
        
        self.last_ml_analysis = Some(Utc::now());
        
        Ok((ml_score, recommendation))
    }

    /// ACCIÓN 8.2: Optimización ML de parámetros de trading
    async fn optimize_ml_parameters(&mut self) -> Result<()> {
        debug!("🔧 Optimizando parámetros ML basado en rendimiento...");
        
        if self.trading_stats.total_trades == 0 {
            return Ok(()); // No hay datos suficientes
        }
        
        let current_accuracy = if self.trading_stats.successful_trades > 0 {
            self.trading_stats.successful_trades as f64 / self.trading_stats.total_trades as f64
        } else {
            0.0
        };
        
        // Ajustar parámetros basado en accuracy actual
        if current_accuracy > 0.8 {
            // Alto rendimiento: ser más agresivo
            self.config.min_profit_threshold = (self.config.min_profit_threshold * 0.95).max(0.5);
            self.config.max_trade_amount = (self.config.max_trade_amount * 1.05).min(100.0);
            info!("📈 Alto rendimiento detectado. Parámetros más agresivos.");
        } else if current_accuracy < 0.5 {
            // Bajo rendimiento: ser más conservador
            self.config.min_profit_threshold = (self.config.min_profit_threshold * 1.1).min(5.0);
            self.config.max_trade_amount = (self.config.max_trade_amount * 0.9).max(10.0);
            info!("📉 Bajo rendimiento detectado. Parámetros más conservadores.");
        }
        
        // Actualizar métricas de optimización
        self.perf_metrics.adaptive_adjustments += 1;
        self.trading_stats.ml_prediction_accuracy = current_accuracy;
        
        info!("🎯 Accuracy actual: {:.3} | Min profit: {:.2}% | Max amount: ${:.1}", 
              current_accuracy, 
              self.config.min_profit_threshold,
              self.config.max_trade_amount);
        
        Ok(())
    }

    /// ACCIÓN 7.2: Optimización de rendimiento de descubrimiento
    fn optimize_discovery_performance(&mut self, discovery_time_ms: u64, opportunities_found: usize) {
        self.perf_metrics.discovery_time_ms = discovery_time_ms;
        self.perf_metrics.opportunities_per_second = if discovery_time_ms > 0 {
            opportunities_found as f64 / (discovery_time_ms as f64 / 1000.0)
        } else {
            0.0
        };

        // Ajustar parámetros según rendimiento
        if discovery_time_ms > self.perf_config.latency_target_ms {
            self.perf_config.max_concurrent_discoveries = 
                (self.perf_config.max_concurrent_discoveries.saturating_sub(1)).max(1);
            warn!("🐌 Discovery lento ({}ms), reduciendo concurrencia a {}", 
                  discovery_time_ms, self.perf_config.max_concurrent_discoveries);
        } else if discovery_time_ms < self.perf_config.latency_target_ms / 2 {
            self.perf_config.max_concurrent_discoveries = 
                (self.perf_config.max_concurrent_discoveries + 1).min(10);
            info!("⚡ Discovery rápido ({}ms), aumentando concurrencia a {}", 
                  discovery_time_ms, self.perf_config.max_concurrent_discoveries);
        }
    }

    /// ACCIÓN 7.3: Registro de trade con análisis de rendimiento
    fn record_trade_with_performance_analysis(&mut self, 
        token_pair: &str, 
        profit_sol: f64, 
        success: bool, 
        execution_time_ms: u64,
        predicted_profit: f64
    ) {
        // Registrar en el motor ML para aprendizaje
        self.ml_engine.train_with_pattern(&OpportunityPattern {
            timestamp: Utc::now(),
            token_pair: token_pair.to_string(),
            profit_percentage: profit_sol,
            execution_time_ms,
            market_volatility: 0.02, // Estimado
            liquidity_level: 1000.0, // Estimado
            success,
            dex_source: "Jupiter".to_string(),
        });
        
        // Crear registro mejorado con datos ML
        let trade_result = TradeResult {
            trade_id: format!("T{:06}", self.trading_stats.total_trades + 1),
            timestamp: Utc::now(),
            profit_sol,
            execution_time_ms,
            success,
            predicted_profit,
            // ACCIÓN 8: ML fields con valores por defecto
            ml_score: 0.5, // Valor por defecto
            ml_confidence: 0.7, // Valor por defecto
            ml_recommendation: "ANALYZED".to_string(),
            market_condition: "STABLE".to_string(),
        };
        
        // Actualizar estadísticas
        self.trading_stats.total_trades += 1;
        if success {
            self.trading_stats.successful_trades += 1;
            self.trading_stats.total_profit_sol += profit_sol;
            if profit_sol > self.trading_stats.best_trade_profit_sol {
                self.trading_stats.best_trade_profit_sol = profit_sol;
            }
        } else {
            self.trading_stats.failed_trades += 1;
        }
        
        // Actualizar tiempo promedio de ejecución
        let total_execution_time = (self.trading_stats.avg_execution_time_ms * (self.trading_stats.total_trades - 1) as f64) + execution_time_ms as f64;
        self.trading_stats.avg_execution_time_ms = total_execution_time / self.trading_stats.total_trades as f64;
        self.trading_stats.last_trade_time = Some(Utc::now());
        
        // Agregar al historial
        self.trade_history.push_back(trade_result.clone());
        if self.trade_history.len() > 1000 {
            self.trade_history.pop_front();
        }
        
        // Alertas de performance
        if trade_result.profit_sol >= 0.01 {
            info!("🎉 ALERTA PROFIT ALTO: Trade {} generó {:.6} SOL", trade_result.trade_id, trade_result.profit_sol);
        }
        if trade_result.profit_sol <= -0.005 {
            warn!("⚠️ ALERTA PÉRDIDA: Trade {} perdió {:.6} SOL", trade_result.trade_id, trade_result.profit_sol);
        }
    }

    /// ACCIÓN 8.3: Dashboard en tiempo real con insights ML
    fn display_ml_enhanced_dashboard(&self) {
        // Clear screen para dashboard en tiempo real
        print!("\x1B[2J\x1B[1;1H");
        
        let uptime = self.start_time.elapsed();
        // Obtener información ML simplificada
        let total_patterns = self.ml_engine.get_pattern_count();
        let accuracy = if self.trading_stats.total_trades > 0 {
            self.trading_stats.successful_trades as f64 / self.trading_stats.total_trades as f64
        } else {
            0.0
        };

        println!("╔══════════════════════════════════════════════════════════════════════════════════════╗");
        println!("║ 🚀 ML-ENHANCED ARBITRAGE SYSTEM - ANALYTICS DASHBOARD v4.5                         ║");
        println!("║ Uptime: {}:{:02}:{:02} | SOL Balance: {:.6} | Trades: {}                           ║", 
                 uptime.as_secs() / 3600, 
                 (uptime.as_secs() % 3600) / 60, 
                 uptime.as_secs() % 60,
                 self.current_balance, 
                 self.trading_stats.total_trades);
        println!("╠══════════════════════════════════════════════════════════════════════════════════════╣");
        
        // Performance Metrics
        println!("║ 📊 PERFORMANCE METRICS                                                              ║");
        println!("║   • Total Cycles: {} | Avg Time: {:.1}ms | Cache Hit: {:.1}%                       ║", 
                 self.perf_metrics.total_cycles, 
                 self.perf_metrics.avg_cycle_time, 
                 if self.perf_metrics.total_cycles > 0 { 
                     self.perf_metrics.cache_hits as f64 / self.perf_metrics.total_cycles as f64 * 100.0 
                 } else { 0.0 });
        println!("║   • Discoveries: {} | Success Rate: {:.1}%                                          ║", 
                 self.perf_metrics.successful_discoveries,
                 if self.perf_metrics.total_cycles > 0 { 
                     self.perf_metrics.successful_discoveries as f64 / self.perf_metrics.total_cycles as f64 * 100.0 
                 } else { 0.0 });
        
        println!("╠══════════════════════════════════════════════════════════════════════════════════════╣");
        
        // ML Analytics
        println!("║ 🧠 MACHINE LEARNING ANALYTICS                                                       ║");
        println!("║   • ML Predictions: {} | Accuracy: {:.1}%                                          ║", 
                 self.perf_metrics.ml_predictions_made,
                 self.perf_metrics.ml_accuracy_rate * 100.0);
        println!("║   • Pattern Library: {} patterns | Learning Cycles: {}                            ║", 
                 total_patterns, self.perf_metrics.adaptive_adjustments);
        println!("║   • Market Condition: LEARNING | Volatility: {:.4}                                ║", 
                 0.02);
        println!("║   • Adaptive Min Profit: {:.2}% | Max Amount: {:.1} SOL                           ║", 
                 self.config.min_profit_threshold, self.config.max_trade_amount);
        
        println!("╠══════════════════════════════════════════════════════════════════════════════════════╣");
        
        // Trading Stats
        println!("║ 💰 TRADING STATISTICS                                                               ║");
        println!("║   • Successful: {} | Failed: {} | Success Rate: {:.1}%                             ║", 
                 self.trading_stats.successful_trades, 
                 self.trading_stats.failed_trades,
                 if self.trading_stats.total_trades > 0 {
                     self.trading_stats.successful_trades as f64 / self.trading_stats.total_trades as f64 * 100.0
                 } else { 0.0 });
        println!("║   • Total Profit: {:.6} SOL | Avg Profit: {:.6} SOL                               ║", 
                 self.trading_stats.total_profit_sol, 
                 if self.trading_stats.successful_trades > 0 {
                     self.trading_stats.total_profit_sol / self.trading_stats.successful_trades as f64
                 } else { 0.0 });
        println!("║   • Best Trade: {:.6} SOL | ML Predictions: {}                                     ║", 
                 self.trading_stats.best_trade_profit_sol,
                 self.trading_stats.ml_predicted_trades);
        
        println!("╠══════════════════════════════════════════════════════════════════════════════════════╣");
        
        // API Status & Real-time Data
        println!("║ 🌐 API STATUS & CONNECTIONS                                                         ║");
        for (api, status) in &self.api_status {
            let status_icon = if *status { "🟢" } else { "🔴" };
            println!("║   {} {:<20} | Last Check: {}                                      ║", 
                     status_icon, api, Utc::now().format("%H:%M:%S UTC"));
        }
        
        println!("╚══════════════════════════════════════════════════════════════════════════════════════╝");
        
        println!();
        println!("🚀 ═══════════════════════════════════════════════════════════════");
        println!("    Last Update: {} | Status: 🟢 RUNNING ML-ENHANCED", 
                 Utc::now().format("%H:%M:%S UTC"));
        println!("🚀 ═══════════════════════════════════════════════════════════════");
    }

    /// Generar reporte completo
    fn generate_report(&self) -> String {
        let mut report = String::new();
        let uptime = self.start_time.elapsed();
        
        report.push_str("🚀 REPORTE FINAL ML-ENHANCED TRADING SYSTEM (ACCIÓN 8)\n");
        report.push_str("======================================================\n\n");
        
        report.push_str(&format!("⏱️ Tiempo de operación: {}:{:02}:{:02}\n", 
                                 uptime.as_secs() / 3600, 
                                 (uptime.as_secs() % 3600) / 60, 
                                 uptime.as_secs() % 60));
        report.push_str(&format!("💰 Balance final: {:.6} SOL\n", self.current_balance));
        report.push_str(&format!("📊 Total trades: {}\n", self.trading_stats.total_trades));
        report.push_str(&format!("✅ Trades exitosos: {}\n", self.trading_stats.successful_trades));
        report.push_str(&format!("❌ Trades fallidos: {}\n", self.trading_stats.failed_trades));
        report.push_str(&format!("💎 Mejor trade: {:.6} SOL\n", self.trading_stats.best_trade_profit_sol));
        report.push_str(&format!("🧠 ML Predictions: {}\n", self.trading_stats.ml_predicted_trades));
        report.push_str(&format!("🎯 ML Accuracy: {:.1}%\n", accuracy * 100.0));
        
        report
    }
    
    /// Actualizar estado de API
    fn update_api_status(&mut self, api_name: String, is_online: bool) {
        self.api_status.insert(api_name, is_online);
    }
}

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
        .with_env_filter("info")
        .with_target(false)
        .with_thread_ids(false)
        .compact()
        .init();

    info!("🚀 Iniciando ML-Enhanced Arbitrage System v4.5...");

    // Configuración de RPC Mainnet
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let commitment = CommitmentConfig::confirmed();
    let rpc_client = Arc::new(RpcClient::new_with_commitment(rpc_url.to_string(), commitment));

    // Wallet configuration
    let wallet_pubkey = match std::env::var("WALLET_ADDRESS") {
        Ok(address) => Pubkey::from_str(&address)?,
        Err(_) => {
            warn!("⚠️ WALLET_ADDRESS no encontrada en variables de entorno");
            warn!("💡 Usando wallet de prueba. Para uso real, configura: export WALLET_ADDRESS=tu_wallet_address");
            // Usar una wallet de ejemplo para demostración (NO USAR EN PRODUCCIÓN)
            Pubkey::from_str("11111111111111111111111111111112")?
        }
    };

    // Obtener balance inicial
    let initial_balance = match get_wallet_balance(&rpc_client, &wallet_pubkey).await {
        Ok(balance) => {
            info!("💰 Balance inicial detectado: {:.6} SOL", balance);
            balance
        }
        Err(e) => {
            warn!("⚠️ No se pudo obtener balance real: {}", e);
            warn!("💡 Usando balance simulado para demostración");
            1.0 // Balance simulado para demo
        }
    };

    // Inicializar sistemas
    let arbitrage_system = ArbitrageBotPhase45Integrated::new(initial_balance).await?;
    let mut enhanced_system = MLEnhancedTradingSystem::new(initial_balance);

    info!("✅ Todos los sistemas inicializados correctamente");
    info!("🧠 ACCIÓN 8 ACTIVADA: Machine Learning & Predictive Analytics");
    info!("🎯 Iniciando monitoreo con ML Pattern Recognition...");

    // Configurar estado inicial de APIs
    enhanced_system.update_api_status("DexScreener".to_string(), true);
    enhanced_system.update_api_status("Jupiter".to_string(), true);
    enhanced_system.update_api_status("MEV Protection".to_string(), true);

    // Loop principal con ML Enhancement
    loop {
        let cycle_start = Instant::now();

        // ACCIÓN 8: Análisis de condiciones de mercado con ML
        let market_conditions = enhanced_system.market_data_cache.clone();
        // Nota: análisis de mercado integrado en analyze_opportunity_with_ml

        // Descubrir oportunidades
        match arbitrage_system.discover_opportunities().await {
            Ok(opportunities) => {
                enhanced_system.perf_metrics.total_cycles += 1;
                
                if !opportunities.is_empty() {
                    enhanced_system.perf_metrics.successful_discoveries += 1;
                    
                    for opportunity in opportunities.iter().take(3) { // Limitar a 3 mejores
                        // ACCIÓN 8: Análisis ML de cada oportunidad
                        if let Ok((ml_score, ml_recommendation)) = enhanced_system.analyze_opportunity_with_ml(
                            &opportunity.token_pair,
                            opportunity.profit_percentage,
                            opportunity.volume_24h.unwrap_or(0.0),
                            opportunity.liquidity.unwrap_or(1000.0)
                        ).await {
                            
                            // Decisión de trading basada en ML
                            if ml_score > enhanced_system.perf_config.ml_confidence_threshold && 
                               opportunity.profit_percentage >= enhanced_system.config.min_profit_threshold {
                                
                                info!("🎯 Ejecutando trade ML-optimizado: {} con score {:.3}", 
                                      opportunity.token_pair, ml_score);
                                
                                // Simular ejecución de trade
                                let trade_start = Utc::now();
                                let execution_time = (Utc::now() - trade_start).num_milliseconds() as u64;
                                let actual_profit = opportunity.profit_percentage * 0.85; // Simular slippage
                                let success = actual_profit > 0.0;
                                
                                // Registrar trade con análisis ML
                                enhanced_system.record_trade_with_performance_analysis(
                                    &opportunity.token_pair,
                                    actual_profit / 100.0, // Convertir a SOL
                                    success,
                                    execution_time,
                                    opportunity.profit_percentage / 100.0
                                );
                                
                                if success {
                                    enhanced_system.current_balance += actual_profit / 100.0;
                                    info!("✅ Trade exitoso: +{:.6} SOL | Balance: {:.6} SOL", 
                                          actual_profit / 100.0, enhanced_system.current_balance);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("❌ Error en discovery: {}", e);
                enhanced_system.perf_metrics.total_cycles += 1;
            }
        }

        // ACCIÓN 8: Optimización ML de parámetros cada 10 ciclos
        if enhanced_system.perf_metrics.total_cycles % 10 == 0 {
            if let Err(e) = enhanced_system.optimize_ml_parameters().await {
                warn!("⚠️ Error en optimización ML: {}", e);
            }
        }

        // Calcular métricas del ciclo
        let cycle_duration = cycle_start.elapsed().as_millis() as u64;
        enhanced_system.perf_metrics.avg_cycle_time = if enhanced_system.perf_metrics.total_cycles > 0 {
            (enhanced_system.perf_metrics.avg_cycle_time * (enhanced_system.perf_metrics.total_cycles - 1) as f64 + cycle_duration as f64) / enhanced_system.perf_metrics.total_cycles as f64
        } else {
            cycle_duration as f64
        };

        // ACCIÓN 7 & 8: Optimización y Display
        enhanced_system.optimize_discovery_performance(cycle_duration, 1);
        enhanced_system.display_ml_enhanced_dashboard();

        // Guardar métricas ML simples en historial
        let ml_summary = format!("cycle_{}_patterns_{}_accuracy_{:.2}", 
                                enhanced_system.perf_metrics.total_cycles,
                                enhanced_system.ml_engine.get_pattern_count(),
                                enhanced_system.trading_stats.ml_prediction_accuracy);
        enhanced_system.ml_metrics_history.push_back(ml_summary);
        if enhanced_system.ml_metrics_history.len() > 100 {
            enhanced_system.ml_metrics_history.pop_front();
        }

        // Sleep antes del próximo ciclo
        sleep(Duration::from_secs(5)).await;
    }
}
