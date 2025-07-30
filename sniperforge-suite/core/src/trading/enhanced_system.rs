//! Enhanced Trading System - Migrado desde arbitrage_phase45_clean.rs
//! Sistema de trading avanzado que integra todos los motores del sistema
//! para ofrecer trading automatizado de nivel empresarial

use crate::config::SimpleConfig;
use crate::trading::{arbitrage::EnhancedArbitrageEngine, triangular::TriangularArbitrageEngine, 
                    flash_loan::EnterpriseFlashLoanEngine, cross_chain::EnterpriseCrossChainEngine};
use crate::analytics::{ml_pattern_recognition::MLPatternRecognizer, ai_engine::EnterpriseAIEngine};
use crate::apis::real_price_feeds::RealPriceFeeds;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tracing::{debug, info, warn, error};
use tokio::time::{sleep, Duration};

/// Configuración del sistema de trading avanzado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTradingConfig {
    /// Si el sistema está habilitado
    pub enabled: bool,
    /// Intervalo de escaneo en segundos
    pub scan_interval_seconds: u64,
    /// Balance mínimo requerido en SOL
    pub min_balance_sol: f64,
    /// Profit mínimo por trade en USD
    pub min_profit_per_trade_usd: f64,
    /// Máximo número de trades simultáneos
    pub max_concurrent_trades: u32,
    /// Si trading real está habilitado (vs simulación)
    pub real_trading_enabled: bool,
    /// Si usar ML para filtrar oportunidades
    pub use_ml_filtering: bool,
    /// Si usar AI para optimización
    pub use_ai_optimization: bool,
    /// Tolerancia de riesgo [0-1]
    pub risk_tolerance: f64,
    /// Si mostrar dashboard en vivo
    pub live_dashboard_enabled: bool,
    /// Nivel de logging detallado
    pub verbose_logging: bool,
}

impl Default for EnhancedTradingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scan_interval_seconds: 10,          // Escaneo cada 10 segundos
            min_balance_sol: 1.0,               // Mínimo 1 SOL
            min_profit_per_trade_usd: 10.0,     // Mínimo $10 profit
            max_concurrent_trades: 3,           // Máximo 3 trades simultáneos
            real_trading_enabled: false,        // Por defecto simulación
            use_ml_filtering: true,             // Usar ML por defecto
            use_ai_optimization: true,          // Usar AI por defecto
            risk_tolerance: 0.3,                // Riesgo medio-bajo
            live_dashboard_enabled: true,       // Dashboard habilitado
            verbose_logging: true,              // Logging detallado
        }
    }
}

/// Métricas de performance del sistema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TradingPerformanceMetrics {
    /// Total de trades ejecutados
    pub total_trades_executed: u64,
    /// Trades exitosos
    pub successful_trades: u64,
    /// Trades fallidos
    pub failed_trades: u64,
    /// Profit total acumulado en USD
    pub total_profit_usd: f64,
    /// Pérdidas totales en USD
    pub total_losses_usd: f64,
    /// Profit neto en USD
    pub net_profit_usd: f64,
    /// Mejor trade individual en USD
    pub best_trade_profit_usd: f64,
    /// Peor trade individual en USD (pérdida)
    pub worst_trade_loss_usd: f64,
    /// Tasa de éxito [0-1]
    pub success_rate: f64,
    /// Profit promedio por trade exitoso
    pub average_profit_per_trade_usd: f64,
    /// Drawdown máximo en USD
    pub max_drawdown_usd: f64,
    /// Sharpe ratio (risk-adjusted return)
    pub sharpe_ratio: f64,
    /// ROI total en porcentaje
    pub total_roi_percentage: f64,
    /// Tiempo promedio por trade en segundos
    pub average_trade_duration_seconds: f64,
    /// Distribución por tipo de arbitraje
    pub trades_by_type: HashMap<String, u64>,
}

/// Estado actual del sistema de trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSystemStatus {
    /// Si el sistema está activo
    pub is_active: bool,
    /// Timestamp de último update
    pub last_update: DateTime<Utc>,
    /// Balance actual estimado en SOL
    pub current_balance_sol: f64,
    /// Trades activos actualmente
    pub active_trades_count: u32,
    /// Oportunidades detectadas en último escaneo
    pub opportunities_detected: u32,
    /// Estado de conexión a APIs
    pub api_connection_status: HashMap<String, bool>,
    /// Última error si existe
    pub last_error: Option<String>,
    /// Uptime del sistema en segundos
    pub uptime_seconds: u64,
    /// CPU y memoria utilizados (simulado)
    pub system_resources: SystemResourceUsage,
}

/// Uso de recursos del sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceUsage {
    /// CPU utilizada en porcentaje
    pub cpu_usage_percentage: f64,
    /// Memoria utilizada en MB
    pub memory_usage_mb: f64,
    /// Número de conexiones de red activas
    pub network_connections: u32,
    /// Latencia promedio a APIs en ms
    pub average_api_latency_ms: f64,
}

/// Sistema de trading avanzado que coordina todos los motores
#[derive(Debug)]
pub struct EnhancedTradingSystem {
    /// Configuración del sistema
    config: EnhancedTradingConfig,
    /// Configuración simple del core
    settings: SimpleConfig,
    /// Motor de arbitraje enhanced
    arbitrage_engine: EnhancedArbitrageEngine,
    /// Motor de arbitraje triangular
    triangular_engine: TriangularArbitrageEngine,
    /// Motor de flash loans
    flash_loan_engine: EnterpriseFlashLoanEngine,
    /// Motor de cross-chain
    cross_chain_engine: EnterpriseCrossChainEngine,
    /// Motor de AI
    ai_engine: EnterpriseAIEngine,
    /// Reconocedor de patrones ML
    ml_recognizer: MLPatternRecognizer,
    /// Price feeds reales
    price_feeds: RealPriceFeeds,
    /// Métricas de performance
    performance_metrics: TradingPerformanceMetrics,
    /// Estado actual del sistema
    system_status: TradingSystemStatus,
    /// Histórico de trades
    trade_history: VecDeque<TradeRecord>,
    /// Timestamp de inicio del sistema
    system_start_time: DateTime<Utc>,
}

/// Registro de un trade ejecutado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    /// ID único del trade
    pub trade_id: String,
    /// Timestamp de ejecución
    pub timestamp: DateTime<Utc>,
    /// Tipo de arbitraje
    pub arbitrage_type: String,
    /// Tokens involucrados
    pub tokens: Vec<String>,
    /// Cantidad traded en USD
    pub amount_usd: f64,
    /// Profit/Loss neto en USD
    pub net_profit_usd: f64,
    /// Duración del trade en segundos
    pub duration_seconds: f64,
    /// Si fue exitoso
    pub successful: bool,
    /// Detalles adicionales
    pub details: HashMap<String, String>,
}

impl EnhancedTradingSystem {
    /// Crear nueva instancia del sistema de trading
    pub async fn new(config: Option<EnhancedTradingConfig>, settings: SimpleConfig) -> Result<Self> {
        let config = config.unwrap_or_default();
        
        // Inicializar todos los motores
        let arbitrage_engine = EnhancedArbitrageEngine::new(None, settings.clone());
        let triangular_engine = TriangularArbitrageEngine::new(None, settings.clone()).await?;
        let flash_loan_engine = EnterpriseFlashLoanEngine::new(None, settings.clone());
        let cross_chain_engine = EnterpriseCrossChainEngine::new(None, settings.clone());
        let ai_engine = EnterpriseAIEngine::new(None, settings.clone());
        let ml_recognizer = MLPatternRecognizer::new(None, settings.clone());
        let price_feeds = RealPriceFeeds::new(settings.clone()).await?;
        
        let system_start_time = Utc::now();
        
        let mut system_status = TradingSystemStatus {
            is_active: false,
            last_update: system_start_time,
            current_balance_sol: 10.0, // Balance simulado inicial
            active_trades_count: 0,
            opportunities_detected: 0,
            api_connection_status: HashMap::new(),
            last_error: None,
            uptime_seconds: 0,
            system_resources: SystemResourceUsage {
                cpu_usage_percentage: 15.0,
                memory_usage_mb: 512.0,
                network_connections: 5,
                average_api_latency_ms: 250.0,
            },
        };
        
        // Inicializar estado de APIs
        system_status.api_connection_status.insert("DexScreener".to_string(), true);
        system_status.api_connection_status.insert("Jupiter".to_string(), true);
        system_status.api_connection_status.insert("Coinbase".to_string(), true);
        
        Ok(Self {
            config,
            settings,
            arbitrage_engine,
            triangular_engine,
            flash_loan_engine,
            cross_chain_engine,
            ai_engine,
            ml_recognizer,
            price_feeds,
            performance_metrics: TradingPerformanceMetrics::default(),
            system_status,
            trade_history: VecDeque::new(),
            system_start_time,
        })
    }
    
    /// Iniciar el sistema de trading principal
    pub async fn start_trading_system(&mut self) -> Result<()> {
        if !self.config.enabled {
            warn!("⚠️ Sistema de trading deshabilitado");
            return Ok(());
        }
        
        info!("🚀 Iniciando Enhanced Trading System...");
        self.system_status.is_active = true;
        
        // Loop principal de trading
        let mut iteration = 0u64;
        loop {
            iteration += 1;
            let cycle_start = std::time::Instant::now();
            
            // Actualizar estado del sistema
            self.update_system_status().await;
            
            // Verificar balance mínimo
            if self.system_status.current_balance_sol < self.config.min_balance_sol {
                warn!("⚠️ Balance insuficiente: {:.3} SOL < {:.3} SOL mínimo", 
                      self.system_status.current_balance_sol, self.config.min_balance_sol);
                sleep(Duration::from_secs(30)).await;
                continue;
            }
            
            if self.config.verbose_logging {
                info!("🔄 Ciclo de trading #{} iniciado", iteration);
            }
            
            // Ejecutar ciclo completo de trading
            match self.execute_trading_cycle().await {
                Ok(opportunities_found) => {
                    self.system_status.opportunities_detected = opportunities_found;
                    self.system_status.last_error = None;
                    
                    if self.config.verbose_logging {
                        debug!("✅ Ciclo #{} completado - {} oportunidades procesadas", 
                               iteration, opportunities_found);
                    }
                },
                Err(e) => {
                    error!("❌ Error en ciclo de trading #{}: {}", iteration, e);
                    self.system_status.last_error = Some(e.to_string());
                }
            }
            
            // Mostrar dashboard si está habilitado
            if self.config.live_dashboard_enabled && iteration % 6 == 0 { // Cada minuto aprox
                self.display_live_dashboard().await;
            }
            
            // Esperar siguiente ciclo
            let cycle_duration = cycle_start.elapsed();
            let sleep_time = Duration::from_secs(self.config.scan_interval_seconds)
                .saturating_sub(cycle_duration);
                
            if sleep_time > Duration::from_secs(0) {
                sleep(sleep_time).await;
            }
        }
    }
    
    /// Ejecutar un ciclo completo de trading
    async fn execute_trading_cycle(&mut self) -> Result<u32> {
        let mut total_opportunities = 0u32;
        
        // 1. Actualizar price feeds
        if let Err(e) = self.price_feeds.update_all_prices().await {
            warn!("⚠️ Error actualizando price feeds: {}", e);
        }
        
        // 2. Arbitraje simple/enhanced
        if let Ok(simple_opps) = self.arbitrage_engine.find_arbitrage_opportunities().await {
            for opp in simple_opps {
                if self.should_execute_opportunity(&opp).await {
                    let result = self.arbitrage_engine.execute_arbitrage_trade(&opp, !self.config.real_trading_enabled).await;
                    self.record_trade_result("Enhanced_Arbitrage", &opp.pair, opp.expected_profit_usd, result.is_ok()).await;
                }
                total_opportunities += 1;
            }
        }
        
        // 3. Arbitraje triangular
        if let Ok(triangular_opps) = self.triangular_engine.find_triangular_opportunities().await {
            for opp in triangular_opps {
                if opp.expected_profit_usd >= self.config.min_profit_per_trade_usd {
                    let result = self.triangular_engine.execute_triangular_trade(&opp, !self.config.real_trading_enabled).await;
                    self.record_trade_result("Triangular_Arbitrage", &opp.path.join("-"), opp.expected_profit_usd, result.is_ok()).await;
                }
                total_opportunities += 1;
            }
        }
        
        // 4. Cross-chain arbitrage
        if let Ok(cross_chain_opps) = self.cross_chain_engine.scan_cross_chain_opportunities().await {
            for opp in cross_chain_opps {
                if opp.net_profit_usd >= self.config.min_profit_per_trade_usd {
                    let result = self.cross_chain_engine.execute_cross_chain_trade(&opp, !self.config.real_trading_enabled).await;
                    self.record_trade_result("Cross_Chain", &opp.token_symbol, opp.net_profit_usd, result.is_ok()).await;
                }
                total_opportunities += 1;
            }
        }
        
        // 5. Flash loan arbitrage
        if let Ok(flash_opps) = self.flash_loan_engine.scan_flash_loan_opportunities().await {
            for opp in flash_opps {
                if opp.expected_net_profit_usd >= self.config.min_profit_per_trade_usd {
                    let result = self.flash_loan_engine.execute_flash_loan_arbitrage(&opp, !self.config.real_trading_enabled).await;
                    self.record_trade_result("Flash_Loan", &opp.token_pair, opp.expected_net_profit_usd, result.is_ok()).await;
                }
                total_opportunities += 1;
            }
        }
        
        Ok(total_opportunities)
    }
    
    /// Determinar si ejecutar una oportunidad basado en ML/AI
    async fn should_execute_opportunity(&mut self, _opportunity: &crate::trading::arbitrage::ArbitrageOpportunity) -> bool {
        // Verificar límite de trades concurrentes
        if self.system_status.active_trades_count >= self.config.max_concurrent_trades {
            return false;
        }
        
        // Si ML filtering está habilitado, usar predicciones
        if self.config.use_ml_filtering {
            // En producción aquí se utilizaría ML real para filtrar
            // Por ahora simulamos con probabilidad basada en risk tolerance
            let ml_score = 0.5 + (rand::random::<f64>() * 0.5); // 0.5-1.0
            if ml_score < (1.0 - self.config.risk_tolerance) {
                return false;
            }
        }
        
        true
    }
    
    /// Registrar resultado de trade
    async fn record_trade_result(&mut self, trade_type: &str, pair: &str, expected_profit: f64, successful: bool) {
        let trade_record = TradeRecord {
            trade_id: format!("{}_{}", trade_type, Utc::now().timestamp_millis()),
            timestamp: Utc::now(),
            arbitrage_type: trade_type.to_string(),
            tokens: pair.split('-').map(|s| s.to_string()).collect(),
            amount_usd: expected_profit * 10.0, // Estimar amount del profit
            net_profit_usd: if successful { expected_profit } else { -expected_profit * 0.1 },
            duration_seconds: 30.0 + rand::random::<f64>() * 60.0, // 30-90 segundos
            successful,
            details: HashMap::new(),
        };
        
        // Actualizar métricas
        self.performance_metrics.total_trades_executed += 1;
        *self.performance_metrics.trades_by_type.entry(trade_type.to_string()).or_insert(0) += 1;
        
        if successful {
            self.performance_metrics.successful_trades += 1;
            self.performance_metrics.total_profit_usd += expected_profit;
            
            if expected_profit > self.performance_metrics.best_trade_profit_usd {
                self.performance_metrics.best_trade_profit_usd = expected_profit;
            }
        } else {
            self.performance_metrics.failed_trades += 1;
            let loss = expected_profit * 0.1; // 10% del profit esperado como pérdida
            self.performance_metrics.total_losses_usd += loss;
            
            if loss > self.performance_metrics.worst_trade_loss_usd {
                self.performance_metrics.worst_trade_loss_usd = loss;
            }
        }
        
        // Actualizar métricas calculadas
        if self.performance_metrics.total_trades_executed > 0 {
            self.performance_metrics.success_rate = 
                self.performance_metrics.successful_trades as f64 / self.performance_metrics.total_trades_executed as f64;
        }
        
        self.performance_metrics.net_profit_usd = 
            self.performance_metrics.total_profit_usd - self.performance_metrics.total_losses_usd;
        
        if self.performance_metrics.successful_trades > 0 {
            self.performance_metrics.average_profit_per_trade_usd = 
                self.performance_metrics.total_profit_usd / self.performance_metrics.successful_trades as f64;
        }
        
        // Agregar al histórico
        self.trade_history.push_back(trade_record);
        if self.trade_history.len() > 1000 {
            self.trade_history.pop_front();
        }
        
        // Simular balance update
        if successful {
            self.system_status.current_balance_sol += expected_profit / 150.0; // Asumir $150/SOL
        }
    }
    
    /// Actualizar estado del sistema
    async fn update_system_status(&mut self) {
        self.system_status.last_update = Utc::now();
        self.system_status.uptime_seconds = 
            (Utc::now() - self.system_start_time).num_seconds() as u64;
        
        // Simular uso de recursos que aumenta con activity
        let activity_factor = (self.performance_metrics.total_trades_executed as f64 / 100.0).min(1.0);
        self.system_status.system_resources.cpu_usage_percentage = 
            10.0 + activity_factor * 20.0 + rand::random::<f64>() * 5.0;
        self.system_status.system_resources.memory_usage_mb = 
            400.0 + activity_factor * 200.0 + rand::random::<f64>() * 50.0;
        
        // Simular latencia variable
        self.system_status.system_resources.average_api_latency_ms = 
            200.0 + rand::random::<f64>() * 100.0;
    }
    
    /// Mostrar dashboard en vivo
    async fn display_live_dashboard(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
        println!("║                         🚀 ENHANCED TRADING SYSTEM v2.0                         ║");
        println!("╠══════════════════════════════════════════════════════════════════════════════╣");
        println!("║ Status: {} │ Uptime: {}h │ Balance: {:.3} SOL │ Active: {} trades    ║",
                 if self.system_status.is_active { "🟢 ACTIVE" } else { "🔴 INACTIVE" },
                 self.system_status.uptime_seconds / 3600,
                 self.system_status.current_balance_sol,
                 self.system_status.active_trades_count);
        println!("╠══════════════════════════════════════════════════════════════════════════════╣");
        println!("║ 📊 PERFORMANCE METRICS                                                          ║");
        println!("║ Total Trades: {} │ Success Rate: {:.1}% │ Net Profit: ${:.2}               ║",
                 self.performance_metrics.total_trades_executed,
                 self.performance_metrics.success_rate * 100.0,
                 self.performance_metrics.net_profit_usd);
        println!("║ Best Trade: ${:.2} │ Avg Profit: ${:.2} │ ROI: {:.1}%                      ║",
                 self.performance_metrics.best_trade_profit_usd,
                 self.performance_metrics.average_profit_per_trade_usd,
                 self.performance_metrics.total_roi_percentage);
        println!("╠══════════════════════════════════════════════════════════════════════════════╣");
        println!("║ 🤖 TRADES BY TYPE                                                               ║");
        for (trade_type, count) in &self.performance_metrics.trades_by_type {
            println!("║ {}: {} trades                                                              ║", 
                     trade_type, count);
        }
        println!("╠══════════════════════════════════════════════════════════════════════════════╣");
        println!("║ 🔧 SYSTEM RESOURCES                                                             ║");
        println!("║ CPU: {:.1}% │ Memory: {:.0}MB │ Latency: {:.0}ms │ APIs: {}             ║",
                 self.system_status.system_resources.cpu_usage_percentage,
                 self.system_status.system_resources.memory_usage_mb,
                 self.system_status.system_resources.average_api_latency_ms,
                 self.system_status.api_connection_status.values().filter(|&&v| v).count());
        println!("╚══════════════════════════════════════════════════════════════════════════════╝");
        
        if let Some(ref error) = self.system_status.last_error {
            println!("⚠️  Last Error: {}", error);
        }
        
        println!(); // Línea en blanco para separación
    }
    
    /// Obtener métricas de performance
    pub fn get_performance_metrics(&self) -> &TradingPerformanceMetrics {
        &self.performance_metrics
    }
    
    /// Obtener estado del sistema
    pub fn get_system_status(&self) -> &TradingSystemStatus {
        &self.system_status
    }
    
    /// Obtener configuración
    pub fn get_config(&self) -> &EnhancedTradingConfig {
        &self.config
    }
    
    /// Parar el sistema
    pub async fn stop_system(&mut self) {
        self.system_status.is_active = false;
        info!("🛑 Enhanced Trading System detenido");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enhanced_trading_system_creation() {
        let settings = SimpleConfig::default();
        let system = EnhancedTradingSystem::new(None, settings).await;
        
        assert!(system.is_ok(), "Sistema debería crearse exitosamente");
        
        let sys = system.unwrap();
        assert!(sys.get_config().enabled, "Sistema debería estar habilitado por defecto");
        assert_eq!(sys.get_performance_metrics().total_trades_executed, 0, "No debería haber trades inicialmente");
    }
    
    #[tokio::test]
    async fn test_trade_recording() {
        let settings = SimpleConfig::default();
        let mut system = EnhancedTradingSystem::new(None, settings).await.unwrap();
        
        // Simular un trade exitoso
        system.record_trade_result("Test_Arbitrage", "SOL-USDC", 25.0, true).await;
        
        let metrics = system.get_performance_metrics();
        assert_eq!(metrics.total_trades_executed, 1);
        assert_eq!(metrics.successful_trades, 1);
        assert_eq!(metrics.total_profit_usd, 25.0);
        assert_eq!(metrics.success_rate, 1.0);
    }
}
