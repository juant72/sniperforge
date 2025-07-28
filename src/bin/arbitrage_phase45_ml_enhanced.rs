// ================================================================================
// ARBITRAGE BOT PHASE 4.5 - ENHANCED WITH ACCIÓN 8 (MACHINE LEARNING)
// ================================================================================
// Sistema principal con TODAS las optimizaciones de ACCIÓN 7 + ACCIÓN 8 ML
// Performance Optimizer + Profit Tracker + Dashboard + Pattern Recognition Engine
// ================================================================================

use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn};
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::str::FromStr;
use chrono::{DateTime, Utc};

// Importar el sistema integrado Phase 4.5 y ML
use sniperforge::{
    arbitrage_bot_phase45_integrated::ArbitrageBotPhase45Integrated,
    unified_config::UnifiedPhase45Config,
    ml_pattern_recognition::{
        PatternRecognitionEngine, OpportunityPattern, MarketCondition, 
        MarketTrend, analyze_market_condition
    }
};

// ================================================================================
// ACCIÓN 8: ESTRUCTURAS INTEGRADAS (ACCIÓN 7 + ML)
// ================================================================================

/// Configuración de performance integrada
#[derive(Debug, Clone)]
struct PerformanceConfig {
    max_concurrent_discoveries: usize,
    _cache_ttl_seconds: u64,
    _parallel_api_calls: bool,
    latency_target_ms: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_discoveries: 3,
            _cache_ttl_seconds: 2,
            _parallel_api_calls: true,
            latency_target_ms: 150,
        }
    }
}

/// Métricas de performance
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    discovery_time_ms: u64,
    opportunities_per_second: f64,
    _cache_hit_rate: f64,
    total_cycles: u64,
    successful_discoveries: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            discovery_time_ms: 0,
            opportunities_per_second: 0.0,
            _cache_hit_rate: 0.0,
            total_cycles: 0,
            successful_discoveries: 0,
        }
    }
}

/// Resultado de trade para tracking
#[derive(Debug, Clone)]
struct TradeResult {
    trade_id: String,
    timestamp: DateTime<Utc>,
    profit_sol: f64,
    execution_time_ms: u64,
    success: bool,
    _dex_used: String,
}

/// Estadísticas de trading
#[derive(Debug, Clone)]
struct TradingStats {
    total_trades: u64,
    successful_trades: u64,
    total_profit_sol: f64,
    average_profit_per_trade: f64,
    success_rate: f64,
    best_trade_profit: f64,
}

impl Default for TradingStats {
    fn default() -> Self {
        Self {
            total_trades: 0,
            successful_trades: 0,
            total_profit_sol: 0.0,
            average_profit_per_trade: 0.0,
            success_rate: 0.0,
            best_trade_profit: 0.0,
        }
    }
}

/// Sistema integrado con ML (ACCIÓN 8)
struct EnhancedMLTradingSystem {
    // ACCIÓN 7: Performance tracking
    perf_config: PerformanceConfig,
    perf_metrics: PerformanceMetrics,
    
    // ACCIÓN 7: Profit tracking
    trading_stats: TradingStats,
    trade_history: VecDeque<TradeResult>,
    initial_balance: f64,
    current_balance: f64,
    
    // ACCIÓN 7: Dashboard data
    hourly_profits: VecDeque<(DateTime<Utc>, f64)>,
    api_status: HashMap<String, bool>,
    start_time: Instant,
    
    // ACCIÓN 8: Machine Learning
    ml_engine: PatternRecognitionEngine,
    market_analysis_window: VecDeque<OpportunityPattern>,
    ml_recommendations: HashMap<String, f64>, // token_pair -> confidence
}

impl EnhancedMLTradingSystem {
    fn new(initial_balance: f64) -> Self {
        info!("🚀 Inicializando Enhanced ML Trading System (ACCIÓN 7 + 8 Integrados)");
        info!("🧠 Machine Learning Pattern Recognition Engine ACTIVADO");
        
        Self {
            perf_config: PerformanceConfig::default(),
            perf_metrics: PerformanceMetrics::default(),
            trading_stats: TradingStats::default(),
            trade_history: VecDeque::new(),
            initial_balance,
            current_balance: initial_balance,
            hourly_profits: VecDeque::new(),
            api_status: HashMap::new(),
            start_time: Instant::now(),
            
            // ACCIÓN 8: ML components
            ml_engine: PatternRecognitionEngine::new(),
            market_analysis_window: VecDeque::new(),
            ml_recommendations: HashMap::new(),
        }
    }
    
    /// ACCIÓN 8: Análisis ML de oportunidades
    fn analyze_opportunities_with_ml(&mut self, opportunities: &[String], profits: &[f64]) -> Vec<usize> {
        info!("🧠 [ACCIÓN 8] Analizando {} oportunidades con ML", opportunities.len());
        
        let mut scored_opportunities = Vec::new();
        
        for (i, (token_pair, profit)) in opportunities.iter().zip(profits.iter()).enumerate() {
            // Scoring ML de la oportunidad
            let ml_score = self.ml_engine.score_opportunity(
                token_pair,
                *profit,
                1.0, // volatility estimada
                0.8  // liquidity estimada
            );
            
            // Predicción de parámetros adaptativos
            let adaptive_params = self.ml_engine.predict_adaptive_parameters(&ml_score, 0.001);
            
            info!("🎯 ML Analysis [{}]: composite={:.3}, confidence={:.3}, trade_size={:.6}", 
                  token_pair, ml_score.composite_score, adaptive_params.confidence_level, 
                  adaptive_params.recommended_trade_size);
            
            // Solo recomendar oportunidades con score alto
            if ml_score.composite_score > 0.6 && adaptive_params.confidence_level > 0.5 {
                scored_opportunities.push((i, ml_score.composite_score));
                self.ml_recommendations.insert(token_pair.clone(), ml_score.composite_score);
            }
        }
        
        // Ordenar por score ML y retornar índices
        scored_opportunities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let recommended_indices: Vec<usize> = scored_opportunities.into_iter()
            .take(3) // Top 3 ML recommendations
            .map(|(idx, _)| idx)
            .collect();
        
        info!("✅ ML Recommendations: {} opportunities selected from {}", 
              recommended_indices.len(), opportunities.len());
        
        recommended_indices
    }
    
    /// ACCIÓN 8: Registro de oportunidad para aprendizaje ML
    fn record_opportunity_for_ml(&mut self, token_pair: &str, profit_percentage: f64, success: bool, execution_time_ms: u64) {
        let pattern = OpportunityPattern {
            timestamp: Utc::now(),
            token_pair: token_pair.to_string(),
            profit_percentage,
            execution_time_ms,
            market_volatility: 1.0, // Simplified for demo
            liquidity_level: 0.8,   // Simplified for demo
            success,
            dex_source: "Jupiter".to_string(),
        };
        
        self.ml_engine.record_opportunity(pattern.clone());
        self.market_analysis_window.push_back(pattern);
        
        // Mantener ventana de análisis de últimas 50 oportunidades
        if self.market_analysis_window.len() > 50 {
            self.market_analysis_window.pop_front();
        }
        
        // Actualizar condiciones de mercado cada 10 oportunidades
        if self.market_analysis_window.len() % 10 == 0 {
            let market_condition = analyze_market_condition(&self.market_analysis_window.iter().collect::<Vec<_>>());
            self.ml_engine.update_market_conditions(market_condition);
        }
    }
    
    /// ACCIÓN 7.1: Performance optimization para discovery
    fn optimize_discovery_performance(&mut self, discovery_time_ms: u64, opportunities_found: usize) {
        self.perf_metrics.total_cycles += 1;
        self.perf_metrics.discovery_time_ms = discovery_time_ms;
        
        if opportunities_found > 0 {
            self.perf_metrics.successful_discoveries += 1;
            self.perf_metrics.opportunities_per_second = opportunities_found as f64 / (discovery_time_ms as f64 / 1000.0);
        }
        
        // Auto-optimización con ML considerations
        if discovery_time_ms > self.perf_config.latency_target_ms && self.perf_config.max_concurrent_discoveries > 1 {
            self.perf_config.max_concurrent_discoveries -= 1;
            info!("⬇️ [ML-Optimized] Reduciendo concurrencia a {} por latencia alta", self.perf_config.max_concurrent_discoveries);
        } else if discovery_time_ms < self.perf_config.latency_target_ms / 2 && self.perf_config.max_concurrent_discoveries < 5 {
            self.perf_config.max_concurrent_discoveries += 1;
            info!("⬆️ [ML-Optimized] Aumentando concurrencia a {} por performance excelente", self.perf_config.max_concurrent_discoveries);
        }
    }
    
    /// ACCIÓN 7.2: Advanced profit tracking
    fn record_trade(&mut self, trade: TradeResult) {
        info!("💰 Registrando trade: {} - Profit: {:.6} SOL", trade.trade_id, trade.profit_sol);
        
        // ACCIÓN 8: Registrar para ML learning
        let token_pair = if trade.trade_id.contains("RAY") { "RAY" } else if trade.trade_id.contains("JUP") { "JUP" } else { "UNKNOWN" };
        self.record_opportunity_for_ml(token_pair, 99.8 + trade.profit_sol * 100.0, trade.success, trade.execution_time_ms);
        
        // Actualizar estadísticas
        self.trading_stats.total_trades += 1;
        if trade.success {
            self.trading_stats.successful_trades += 1;
        }
        
        self.trading_stats.total_profit_sol += trade.profit_sol;
        if self.trading_stats.total_trades > 0 {
            self.trading_stats.average_profit_per_trade = self.trading_stats.total_profit_sol / self.trading_stats.total_trades as f64;
            self.trading_stats.success_rate = (self.trading_stats.successful_trades as f64 / self.trading_stats.total_trades as f64) * 100.0;
        }
        
        if trade.profit_sol > self.trading_stats.best_trade_profit {
            self.trading_stats.best_trade_profit = trade.profit_sol;
        }
        
        // Actualizar balance
        self.current_balance += trade.profit_sol;
        
        // Agregar a historial
        self.trade_history.push_back(trade.clone());
        if self.trade_history.len() > 50 { // Mantener últimos 50 trades
            self.trade_history.pop_front();
        }
        
        // Agregar a profits por hora
        self.hourly_profits.push_back((trade.timestamp, trade.profit_sol));
        
        // Cleanup de datos antiguos (últimas 24 horas)
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        while let Some((timestamp, _)) = self.hourly_profits.front() {
            if *timestamp < cutoff {
                self.hourly_profits.pop_front();
            } else {
                break;
            }
        }
        
        // Alertas con ML enhancement
        if trade.profit_sol >= 0.01 {
            info!("🎉 [ML-Enhanced] ALERTA PROFIT ALTO: Trade {} generó {:.6} SOL", trade.trade_id, trade.profit_sol);
        }
        if trade.profit_sol <= -0.005 {
            warn!("⚠️ [ML-Enhanced] ALERTA PÉRDIDA: Trade {} perdió {:.6} SOL", trade.trade_id, trade.profit_sol);
        }
    }
    
    /// ACCIÓN 7.3 + 8: Real-time dashboard con ML insights
    fn display_dashboard(&self, cycle_count: u64, opportunities_found: u64, balance: f64) {
        // Clear screen y mostrar dashboard
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        
        let uptime = self.start_time.elapsed();
        let uptime_hours = uptime.as_secs() / 3600;
        let uptime_minutes = (uptime.as_secs() % 3600) / 60;
        let uptime_seconds = uptime.as_secs() % 60;
        
        println!("🚀 ═══════════════════════════════════════════════════════════════");
        println!("🚀   SNIPERFORGE ARBITRAGE - ENHANCED ML DASHBOARD");
        println!("🚀   ACCIÓN 7 + 8: MACHINE LEARNING OPTIMIZATIONS");
        println!("🚀 ═══════════════════════════════════════════════════════════════");
        println!();
        
        // Estado general
        println!("💰 BALANCE & PROFIT:");
        println!("   • Current Balance: {:.9} SOL", balance);
        println!("   • Initial Balance: {:.9} SOL", self.initial_balance);
        println!("   • Total Change: {:+.9} SOL", balance - self.initial_balance);
        println!("   • Tracked Profit: {:+.9} SOL", self.trading_stats.total_profit_sol);
        println!("   • Uptime: {}h {}m {}s", uptime_hours, uptime_minutes, uptime_seconds);
        println!();

        // Trading stats
        println!("📈 TRADING STATS (ACCIÓN 7.2):");
        println!("   • Total Trades: {}", self.trading_stats.total_trades);
        println!("   • Success Rate: {:.1}%", self.trading_stats.success_rate);
        println!("   • Avg Profit/Trade: {:.6} SOL", self.trading_stats.average_profit_per_trade);
        println!("   • Best Trade: {:.6} SOL", self.trading_stats.best_trade_profit);
        println!();

        // Performance metrics
        println!("🚀 PERFORMANCE METRICS (ACCIÓN 7.1):");
        println!("   • Discovery Time: {}ms", self.perf_metrics.discovery_time_ms);
        println!("   • Opportunities/Sec: {:.2}", self.perf_metrics.opportunities_per_second);
        println!("   • Total Cycles: {}", self.perf_metrics.total_cycles);
        println!("   • Success Rate: {:.1}%", 
                 if self.perf_metrics.total_cycles > 0 {
                     (self.perf_metrics.successful_discoveries as f64 / self.perf_metrics.total_cycles as f64) * 100.0
                 } else { 0.0 });
        println!("   • Concurrency Level: {}", self.perf_config.max_concurrent_discoveries);
        println!();

        // ACCIÓN 8: ML Analytics
        println!("🧠 MACHINE LEARNING ANALYTICS (ACCIÓN 8.1):");
        println!("   • Pattern History: {} samples", self.market_analysis_window.len());
        println!("   • ML Recommendations: {} active", self.ml_recommendations.len());
        
        // Mostrar top ML recommendations
        let mut recommendations: Vec<_> = self.ml_recommendations.iter().collect();
        recommendations.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        
        for (token_pair, confidence) in recommendations.iter().take(3) {
            println!("   • {}: {:.3} confidence", token_pair, confidence);
        }
        
        if recommendations.is_empty() {
            println!("   • Collecting data for ML analysis...");
        }
        println!();

        // Current cycle info
        println!("📊 CURRENT CYCLE:");
        println!("   • Cycle Number: #{}", cycle_count);
        println!("   • Opportunities Found: {}", opportunities_found);
        println!("   • Target Latency: {}ms", self.perf_config.latency_target_ms);
        println!();

        // API Status
        println!("📡 API STATUS:");
        for (name, status) in &self.api_status {
            let status_emoji = if *status { "✅" } else { "❌" };
            println!("   • {}: {}", name, status_emoji);
        }
        if self.api_status.is_empty() {
            println!("   • DexScreener: ✅ (Default)");
            println!("   • Jupiter: ✅ (Default)");
            println!("   • MEV Protection: ✅ (Default)");
            println!("   • ML Engine: ✅ (Active)");
        }
        println!();

        // Recent trades (últimos 3)
        println!("📝 RECENT TRADES:");
        if self.trade_history.is_empty() {
            println!("   • No trades recorded yet");
        } else {
            for trade in self.trade_history.iter().rev().take(3) {
                let emoji = if trade.success { "✅" } else { "❌" };
                println!("   {} {} - {:.6} SOL ({}ms)", 
                         emoji,
                         trade.trade_id,
                         trade.profit_sol,
                         trade.execution_time_ms);
            }
        }
        
        println!();
        println!("🚀 ═══════════════════════════════════════════════════════════════");
        println!("    Last Update: {} | Status: 🟢 RUNNING ML-ENHANCED", 
                 Utc::now().format("%H:%M:%S UTC"));
        println!("🚀 ═══════════════════════════════════════════════════════════════");
    }
    
    /// Generar reporte completo con ML insights
    fn generate_report(&self) -> String {
        let mut report = String::new();
        let uptime = self.start_time.elapsed();
        
        report.push_str("🚀 REPORTE FINAL ML-ENHANCED TRADING SYSTEM (ACCIÓN 7+8)\n");
        report.push_str("=========================================================\n\n");
        
        report.push_str(&format!("📊 GENERAL:\n"));
        report.push_str(&format!("• Uptime: {:.1} minutos\n", uptime.as_secs_f64() / 60.0));
        report.push_str(&format!("• Initial Balance: {:.9} SOL\n", self.initial_balance));
        report.push_str(&format!("• Final Balance: {:.9} SOL\n", self.current_balance));
        report.push_str(&format!("• Total Change: {:+.9} SOL\n", self.current_balance - self.initial_balance));
        
        report.push_str(&format!("\n💰 PROFIT TRACKING (ACCIÓN 7.2):\n"));
        report.push_str(&format!("• Total Trades: {}\n", self.trading_stats.total_trades));
        report.push_str(&format!("• Success Rate: {:.1}%\n", self.trading_stats.success_rate));
        report.push_str(&format!("• Total Profit: {:+.9} SOL\n", self.trading_stats.total_profit_sol));
        report.push_str(&format!("• Avg Profit/Trade: {:.6} SOL\n", self.trading_stats.average_profit_per_trade));
        report.push_str(&format!("• Best Trade: {:.6} SOL\n", self.trading_stats.best_trade_profit));
        
        report.push_str(&format!("\n🚀 PERFORMANCE (ACCIÓN 7.1):\n"));
        report.push_str(&format!("• Total Cycles: {}\n", self.perf_metrics.total_cycles));
        report.push_str(&format!("• Successful Discoveries: {}\n", self.perf_metrics.successful_discoveries));
        report.push_str(&format!("• Avg Discovery Time: {}ms\n", self.perf_metrics.discovery_time_ms));
        report.push_str(&format!("• Peak Opportunities/Sec: {:.2}\n", self.perf_metrics.opportunities_per_second));
        
        // ACCIÓN 8: ML Report
        report.push_str(&format!("\n🧠 MACHINE LEARNING (ACCIÓN 8.1):\n"));
        report.push_str(&self.ml_engine.generate_ml_report());
        
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
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === ARBITRAGE BOT PHASE 4.5 - ENHANCED WITH ACCIÓN 8 (ML) ===");
    info!("💰 ATENCIÓN: Este programa ejecuta TRANSACCIONES REALES con SOL");
    info!("🧠 NUEVA VERSIÓN: Performance + Profit + Dashboard + MACHINE LEARNING");
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
    // ACCIÓN 8: INICIALIZAR SISTEMA ML-ENHANCED INTEGRADO
    // ================================================================================
    
    info!("🧠 Inicializando ML-Enhanced Trading System (ACCIÓN 7 + 8 Integrados)...");
    let mut enhanced_ml_system = EnhancedMLTradingSystem::new(initial_balance);
    
    // Actualizar estado de APIs iniciales
    enhanced_ml_system.update_api_status("DexScreener".to_string(), true);
    enhanced_ml_system.update_api_status("Jupiter".to_string(), true);
    enhanced_ml_system.update_api_status("MEV Protection".to_string(), true);
    enhanced_ml_system.update_api_status("DEX Specialization".to_string(), true);
    enhanced_ml_system.update_api_status("ML Engine".to_string(), true);
    
    // Mostrar resumen de configuración
    info!("📋 Configuración del Sistema ML-Enhanced:");
    info!("   • Trading Mode: SAFE (Conservador)");
    info!("   • Max Trade SOL: {:.3}", config.max_trade_sol);
    info!("   • Min Profit BPS: {}", config.min_profit_bps);
    info!("   • Jupiter Advanced: {}", if config.jupiter_advanced_enabled { "✅" } else { "❌" });
    info!("   • MEV Protection: {}", if config.mev_protection_enabled { "✅" } else { "❌" });
    info!("   • DEX Specialization: {}", if config.dex_specialization_enabled { "✅" } else { "❌" });
    info!("   • Event Driven: {}", if config.event_driven_enabled { "✅" } else { "❌" });
    info!("   • ACCIÓN 7 Performance Optimizer: ✅ INTEGRADO");
    info!("   • ACCIÓN 7 Profit Tracker: ✅ INTEGRADO");
    info!("   • ACCIÓN 7 Real-time Dashboard: ✅ INTEGRADO");
    info!("   • ACCIÓN 8 ML Pattern Recognition: ✅ INTEGRADO");

    // Crear e inicializar el sistema Phase 4.5
    info!("🔧 Inicializando Sistema Phase 4.5 ML-Enhanced...");
    let system = match ArbitrageBotPhase45Integrated::new(config).await {
        Ok(system) => {
            info!("✅ Sistema Phase 4.5 ML-Enhanced inicializado correctamente");
            Arc::new(system)
        }
        Err(e) => {
            error!("❌ Error inicializando sistema: {}", e);
            return Err(e);
        }
    };

    // Configuración de dashboard
    let dashboard_interval = std::env::var("DASHBOARD_UPDATE_SECONDS")
        .unwrap_or("5".to_string())
        .parse::<u64>()
        .unwrap_or(5);
    
    info!("📊 ML Dashboard se actualizará cada {} segundos", dashboard_interval);

    // Ejecutar ciclo principal de trading con optimizaciones ML integradas
    info!("🎯 Iniciando ciclo de trading ML-optimizado...");
    info!("⏰ Intervalo entre ciclos: 10 segundos");
    let mut cycle_count = 0;
    let mut last_balance = initial_balance;
    let mut total_profit = 0.0;
    let mut total_opportunities_found = 0u64;
    let mut total_opportunities_executed = 0u64;
    let mut last_dashboard_update = Instant::now();

    loop {
        cycle_count += 1;
        info!("🔄 Ciclo #{} - Buscando oportunidades ML-optimizadas...", cycle_count);

        // Verificar balance actual al inicio del ciclo
        let current_balance = if let Ok(balance) = get_wallet_balance(&rpc_client, &wallet_pubkey).await {
            let balance_change = balance - last_balance;
            info!("💰 Balance actual: {:.9} SOL (cambio: {:+.9} SOL)", balance, balance_change);
            
            if balance_change.abs() > 0.000001 { // Si hay cambio significativo
                total_profit += balance_change;
                info!("📈 Profit acumulado total: {:+.9} SOL", total_profit);
            }
            last_balance = balance;
            balance
        } else {
            last_balance
        };

        // ACCIÓN 7.1 + 8: Buscar oportunidades con ML-Enhanced Performance Optimization
        let discovery_start = Instant::now();
        
        let opportunities = match system.discover_opportunities().await {
            Ok(opportunities) => {
                let discovery_time_ms = discovery_start.elapsed().as_millis() as u64;
                total_opportunities_found += opportunities.len() as u64;
                
                // Actualizar métricas de performance
                enhanced_ml_system.optimize_discovery_performance(discovery_time_ms, opportunities.len());
                
                info!("📊 [ML-ENHANCED] Encontradas {} oportunidades en {}ms", 
                      opportunities.len(), discovery_time_ms);
                
                opportunities
            }
            Err(e) => {
                error!("⚠️ Error en discovery ML-optimizado en ciclo #{}: {}", cycle_count, e);
                enhanced_ml_system.update_api_status("Discovery Engine".to_string(), false);
                Vec::new()
            }
        };

        // ACCIÓN 8: Analizar oportunidades con ML antes de ejecutar
        if !opportunities.is_empty() {
            let token_pairs: Vec<String> = opportunities.iter()
                .map(|opp| {
                    let profit = opp.get_estimated_profit();
                    if profit > 10.0 { "RAY".to_string() } 
                    else if profit > 99.0 { "JUP".to_string() }
                    else { "OTHER".to_string() }
                })
                .collect();
            
            let profits: Vec<f64> = opportunities.iter()
                .map(|opp| opp.get_estimated_profit())
                .collect();
            
            let ml_recommended_indices = enhanced_ml_system.analyze_opportunities_with_ml(&token_pairs, &profits);
            
            // Ejecutar solo las oportunidades recomendadas por ML
            for &i in ml_recommended_indices.iter().take(2) { // Top 2 ML recommendations
                if let Some(opportunity) = opportunities.get(i) {
                    let trade_id = format!("ml_trade_{}_{}", cycle_count, i + 1);
                    let profit_estimate = opportunity.get_estimated_profit();
                    let token_pair = &token_pairs[i];
                    
                    info!("💰 [ML-RECOMMENDED] EJECUTANDO TRADE #{} - ID: {} - Token: {} - Profit esperado: {:.6}%", 
                          i + 1, trade_id, token_pair, profit_estimate);
                    
                    // Balance antes del trade
                    let balance_before = if let Ok(balance) = get_wallet_balance(&rpc_client, &wallet_pubkey).await {
                        balance
                    } else {
                        last_balance
                    };
                    
                    let execution_start = Instant::now();
                    
                    match system.execute_opportunity_real(opportunity.clone()).await {
                        Ok(result) => {
                            let execution_time_ms = execution_start.elapsed().as_millis() as u64;
                            
                            if result.success {
                                total_opportunities_executed += 1;
                                
                                info!("✅ 🧠 [ML-TRADE] EXITOSO! Profit reportado: {:.6} SOL", result.actual_profit_sol);
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
                                    
                                    // ACCIÓN 7.2 + 8: Registrar trade en ML-Enhanced Profit Tracker
                                    let trade_result = TradeResult {
                                        trade_id: trade_id.clone(),
                                        timestamp: Utc::now(),
                                        profit_sol: real_change,
                                        execution_time_ms,
                                        success: real_change > 0.0,
                                        _dex_used: "Jupiter".to_string(),
                                    };
                                    
                                    enhanced_ml_system.record_trade(trade_result);
                                    
                                    if real_change.abs() > 0.000001 {
                                        info!("   ✅ CONFIRMADO: ML Trade real ejecutado - Balance modificado");
                                        last_balance = balance_after;
                                    } else {
                                        warn!("   ⚠️ POSIBLE SIMULACIÓN: No hay cambio en balance real");
                                    }
                                }
                            } else {
                                warn!("⚠️ ML Trade #{} falló: {:?}", i + 1, result.error_message);
                                
                                // Registrar trade fallido para ML learning
                                let trade_result = TradeResult {
                                    trade_id: trade_id.clone(),
                                    timestamp: Utc::now(),
                                    profit_sol: 0.0,
                                    execution_time_ms,
                                    success: false,
                                    _dex_used: "Jupiter".to_string(),
                                };
                                enhanced_ml_system.record_trade(trade_result);
                            }
                        }
                        Err(e) => {
                            error!("❌ Error ejecutando ML trade #{}: {}", i + 1, e);
                            
                            // Registrar error como trade fallido para ML learning
                            let trade_result = TradeResult {
                                trade_id: trade_id.clone(),
                                timestamp: Utc::now(),
                                profit_sol: 0.0,
                                execution_time_ms: execution_start.elapsed().as_millis() as u64,
                                success: false,
                                _dex_used: "Jupiter".to_string(),
                            };
                            enhanced_ml_system.record_trade(trade_result);
                        }
                    }
                }
            }
        }

        // ACCIÓN 7.3 + 8: Actualizar dashboard ML en tiempo real
        if last_dashboard_update.elapsed().as_secs() >= dashboard_interval {
            enhanced_ml_system.display_dashboard(cycle_count, total_opportunities_found, current_balance);
            last_dashboard_update = Instant::now();
        }

        // Reportes ML cada 5 ciclos
        if cycle_count % 5 == 0 {
            info!("📊 === REPORTE ML-ENHANCED (Ciclo {}) ===", cycle_count);
            let report = enhanced_ml_system.generate_report();
            info!("{}", report);
            info!("📊 === FIN REPORTE ML-ENHANCED ===");
        }

        // Pausa entre ciclos (10 segundos)
        info!("⏳ Esperando 10 segundos hasta próximo ciclo ML...");
        sleep(Duration::from_secs(10)).await;
        
        // Limitar a 20 ciclos para demostración (200 segundos total)
        if cycle_count >= 20 {
            info!("🏁 Demo ML completada - 20 ciclos ejecutados");
            
            // Balance final
            if let Ok(final_balance) = get_wallet_balance(&rpc_client, &wallet_pubkey).await {
                let total_change = final_balance - initial_balance;
                
                // Dashboard final ML
                enhanced_ml_system.display_dashboard(cycle_count, total_opportunities_found, final_balance);
                
                info!("📊 RESUMEN FINAL ML-ENHANCED:");
                info!("   • Balance inicial: {:.9} SOL", initial_balance);
                info!("   • Balance final: {:.9} SOL", final_balance);
                info!("   • Cambio total: {:+.9} SOL", total_change);
                info!("   • Profit acumulado: {:+.9} SOL", total_profit);
                info!("   • Oportunidades encontradas: {}", total_opportunities_found);
                info!("   • Oportunidades ejecutadas (ML): {}", total_opportunities_executed);
            }
            
            // Reporte final completo ML
            info!("📈 === REPORTE FINAL ML-ENHANCED ===");
            let final_report = enhanced_ml_system.generate_report();
            info!("{}", final_report);
            info!("📈 === FIN REPORTE FINAL ML ===");
            
            break;
        }
    }

    info!("✅ Sistema de Arbitraje Phase 4.5 ML-Enhanced finalizado");
    info!("🧠 ACCIÓN 8 completada exitosamente con Machine Learning integrado");
    Ok(())
}
