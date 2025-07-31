//! Enhanced Trading System - Sistema de trading simplificado
//! Demuestra la integración de componentes migrados

use crate::config::SimpleConfig;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Configuración del sistema de trading simplificado
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
            scan_interval_seconds: 10,
            min_balance_sol: 1.0,
            min_profit_per_trade_usd: 10.0,
            max_concurrent_trades: 3,
            real_trading_enabled: false,
            use_ml_filtering: true,
            use_ai_optimization: true,
            risk_tolerance: 0.3,
            live_dashboard_enabled: true,
            verbose_logging: true,
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
}

/// Sistema de trading simplificado
pub struct EnhancedTradingSystem {
    /// Configuración del sistema
    config: EnhancedTradingConfig,
    /// Configuración simple del core
    _settings: SimpleConfig,
    /// Métricas de performance
    performance_metrics: TradingPerformanceMetrics,
    /// Estado actual del sistema
    system_status: TradingSystemStatus,
    /// Timestamp de inicio del sistema
    system_start_time: DateTime<Utc>,
}

impl EnhancedTradingSystem {
    /// Crear nueva instancia del sistema de trading
    pub async fn new(config: Option<EnhancedTradingConfig>, settings: SimpleConfig) -> Result<Self> {
        let config = config.unwrap_or_default();
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
        };
        
        // Inicializar estado de APIs
        system_status.api_connection_status.insert("DexScreener".to_string(), true);
        system_status.api_connection_status.insert("Jupiter".to_string(), true);
        system_status.api_connection_status.insert("Coinbase".to_string(), true);
        
        Ok(Self {
            config,
            _settings: settings,
            performance_metrics: TradingPerformanceMetrics::default(),
            system_status,
            system_start_time,
        })
    }
    
    /// Simular ciclo de trading
    pub async fn simulate_trading_cycle(&mut self) -> Result<u32> {
        info!("🔄 Ejecutando ciclo de trading simulado...");
        
        // Simular detección de oportunidades
        let opportunities_found = 3 + (rand::random::<u32>() % 8); // 3-10 oportunidades
        
        // Simular ejecución de trades
        for i in 0..opportunities_found.min(3) {
            let profit = 15.0 + (rand::random::<f64>() * 25.0); // $15-40 profit
            let successful = rand::random::<f64>() > 0.2; // 80% éxito
            
            self.record_trade_result(&format!("SimulatedTrade_{}", i), "SOL-USDC", profit, successful).await;
        }
        
        self.system_status.opportunities_detected = opportunities_found;
        Ok(opportunities_found)
    }
    
    /// Registrar resultado de trade
    async fn record_trade_result(&mut self, trade_type: &str, _pair: &str, expected_profit: f64, successful: bool) {
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
        
        // Simular balance update
        if successful {
            self.system_status.current_balance_sol += expected_profit / 150.0; // Asumir $150/SOL
        }
    }
    
    /// Actualizar estado del sistema
    pub async fn update_system_status(&mut self) {
        self.system_status.last_update = Utc::now();
        self.system_status.uptime_seconds = 
            (Utc::now() - self.system_start_time).num_seconds() as u64;
    }
    
    /// Mostrar dashboard simplificado
    pub async fn display_dashboard(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
        println!("║                         🚀 ENHANCED TRADING SYSTEM v2.0                         ║");
        println!("╠══════════════════════════════════════════════════════════════════════════════╣");
        println!("║ Status: {} │ Uptime: {}h │ Balance: {:.3} SOL               ║",
                 if self.system_status.is_active { "🟢 ACTIVE" } else { "🔴 INACTIVE" },
                 self.system_status.uptime_seconds / 3600,
                 self.system_status.current_balance_sol);
        println!("╠══════════════════════════════════════════════════════════════════════════════╣");
        println!("║ 📊 PERFORMANCE METRICS                                                          ║");
        println!("║ Total Trades: {} │ Success Rate: {:.1}% │ Net Profit: ${:.2}               ║",
                 self.performance_metrics.total_trades_executed,
                 self.performance_metrics.success_rate * 100.0,
                 self.performance_metrics.net_profit_usd);
        println!("║ Best Trade: ${:.2} │ Avg Profit: ${:.2}                                    ║",
                 self.performance_metrics.best_trade_profit_usd,
                 self.performance_metrics.average_profit_per_trade_usd);
        println!("╚══════════════════════════════════════════════════════════════════════════════╝");
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
    async fn test_trading_simulation() {
        let settings = SimpleConfig::default();
        let mut system = EnhancedTradingSystem::new(None, settings).await.unwrap();
        
        // Simular un ciclo de trading
        let opportunities = system.simulate_trading_cycle().await.unwrap();
        assert!(opportunities > 0, "Debería generar oportunidades");
        
        let metrics = system.get_performance_metrics();
        assert!(metrics.total_trades_executed > 0, "Debería haber ejecutado trades");
    }
}
