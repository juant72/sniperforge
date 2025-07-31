// ================================================================================
// REAL-TIME TRADING DASHBOARD - ACCIÓN 7.3
// ================================================================================
// Dashboard en tiempo real para monitoreo del sistema arbitrage_phase45_clean
// Métricas en vivo, alertas y visualización de performance
// ================================================================================

use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};
use tokio::time::{Duration, Instant, interval};
use tracing::{info, warn, error, debug};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::advanced_profit_tracker::{TradingStats, TradeResult};
use crate::advanced_performance_optimizer::PerformanceMetrics;

/// Configuración del dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub update_interval_seconds: u64,
    pub max_log_entries: usize,
    pub enable_console_output: bool,
    pub enable_web_dashboard: bool,
    pub web_port: u16,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub min_success_rate: f64,
    pub max_loss_streak: u32,
    pub min_profit_per_hour: f64,
    pub max_execution_time_ms: u64,
    pub min_api_uptime: f64,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            update_interval_seconds: 5,
            max_log_entries: 100,
            enable_console_output: true,
            enable_web_dashboard: false,
            web_port: 8080,
            alert_thresholds: AlertThresholds {
                min_success_rate: 80.0,
                max_loss_streak: 3,
                min_profit_per_hour: 0.001, // 0.001 SOL/hora mínimo
                max_execution_time_ms: 2000,
                min_api_uptime: 95.0,
            },
        }
    }
}

/// Estado del sistema en tiempo real
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub timestamp: DateTime<Utc>,
    pub is_running: bool,
    pub current_balance: f64,
    pub total_profit: f64,
    pub opportunities_found: u64,
    pub opportunities_executed: u64,
    pub success_rate: f64,
    pub uptime_seconds: u64,
    pub cycles_completed: u64,
    pub last_error: Option<String>,
    pub api_status: HashMap<String, ApiStatus>,
    pub active_integrators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiStatus {
    pub name: String,
    pub is_online: bool,
    pub response_time_ms: u64,
    pub success_rate: f64,
    pub last_error: Option<String>,
    pub uptime_percentage: f64,
}

/// Entrada de log del dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Success,
    Debug,
}

/// Datos para gráficos en tiempo real
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub profit_over_time: VecDeque<(DateTime<Utc>, f64)>,
    pub balance_over_time: VecDeque<(DateTime<Utc>, f64)>,
    pub opportunities_over_time: VecDeque<(DateTime<Utc>, u64)>,
    pub success_rate_over_time: VecDeque<(DateTime<Utc>, f64)>,
    pub api_response_times: HashMap<String, VecDeque<(DateTime<Utc>, u64)>>,
}

impl ChartData {
    fn new() -> Self {
        Self {
            profit_over_time: VecDeque::new(),
            balance_over_time: VecDeque::new(),
            opportunities_over_time: VecDeque::new(),
            success_rate_over_time: VecDeque::new(),
            api_response_times: HashMap::new(),
        }
    }

    fn cleanup_old_data(&mut self, cutoff_time: DateTime<Utc>) {
        self.profit_over_time.retain(|(time, _)| *time >= cutoff_time);
        self.balance_over_time.retain(|(time, _)| *time >= cutoff_time);
        self.opportunities_over_time.retain(|(time, _)| *time >= cutoff_time);
        self.success_rate_over_time.retain(|(time, _)| *time >= cutoff_time);
        
        for (_, data) in self.api_response_times.iter_mut() {
            data.retain(|(time, _)| *time >= cutoff_time);
        }
    }
}

/// Dashboard principal en tiempo real
pub struct RealTimeTradingDashboard {
    config: DashboardConfig,
    system_status: Arc<Mutex<SystemStatus>>,
    log_entries: Arc<Mutex<VecDeque<DashboardLogEntry>>>,
    chart_data: Arc<Mutex<ChartData>>,
    start_time: Instant,
    is_running: Arc<Mutex<bool>>,
    alert_history: VecDeque<DashboardLogEntry>,
}

impl RealTimeTradingDashboard {
    /// Crear nuevo dashboard
    pub fn new(config: DashboardConfig, initial_balance: f64) -> Self {
        info!("📊 Inicializando Real-Time Trading Dashboard");
        info!("   • Update interval: {}s", config.update_interval_seconds);
        info!("   • Console output: {}", config.enable_console_output);
        info!("   • Web dashboard: {} (port: {})", config.enable_web_dashboard, config.web_port);

        let system_status = SystemStatus {
            timestamp: Utc::now(),
            is_running: true,
            current_balance: initial_balance,
            total_profit: 0.0,
            opportunities_found: 0,
            opportunities_executed: 0,
            success_rate: 0.0,
            uptime_seconds: 0,
            cycles_completed: 0,
            last_error: None,
            api_status: HashMap::new(),
            active_integrators: vec![
                "Jupiter Advanced".to_string(),
                "MEV Protection".to_string(),
                "DEX Specialization".to_string(),
            ],
        };

        Self {
            config,
            system_status: Arc::new(Mutex::new(system_status)),
            log_entries: Arc::new(Mutex::new(VecDeque::new())),
            chart_data: Arc::new(Mutex::new(ChartData::new())),
            start_time: Instant::now(),
            is_running: Arc::new(Mutex::new(true)),
            alert_history: VecDeque::new(),
        }
    }

    /// Iniciar el dashboard
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Iniciando Real-Time Trading Dashboard");

        // Clonar referencias para las tareas asíncronas
        let config = self.config.clone();
        let system_status = Arc::clone(&self.system_status);
        let log_entries = Arc::clone(&self.log_entries);
        let chart_data = Arc::clone(&self.chart_data);
        let is_running = Arc::clone(&self.is_running);
        let start_time = self.start_time;

        // Tarea de actualización del dashboard
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.update_interval_seconds));
            
            while *is_running.lock().unwrap() {
                interval.tick().await;
                
                // Actualizar uptime
                {
                    let mut status = system_status.lock().unwrap();
                    status.uptime_seconds = start_time.elapsed().as_secs();
                    status.timestamp = Utc::now();
                }

                // Cleanup de datos antiguos (mantener últimas 24 horas)
                let cutoff_time = Utc::now() - chrono::Duration::hours(24);
                {
                    let mut chart = chart_data.lock().unwrap();
                    chart.cleanup_old_data(cutoff_time);
                }

                // Cleanup de logs antiguos
                {
                    let mut logs = log_entries.lock().unwrap();
                    while logs.len() > config.max_log_entries {
                        logs.pop_front();
                    }
                }

                // Mostrar dashboard en consola si está habilitado
                if config.enable_console_output {
                    Self::print_console_dashboard(&system_status, &log_entries);
                }
            }
        });

        Ok(())
    }

    /// Actualizar estado del sistema
    pub fn update_system_status(&self, 
        balance: f64, 
        total_profit: f64, 
        opportunities_found: u64,
        opportunities_executed: u64,
        cycles_completed: u64,
    ) {
        let mut status = self.system_status.lock().unwrap();
        status.current_balance = balance;
        status.total_profit = total_profit;
        status.opportunities_found = opportunities_found;
        status.opportunities_executed = opportunities_executed;
        status.cycles_completed = cycles_completed;
        
        // Calcular success rate
        if opportunities_found > 0 {
            status.success_rate = (opportunities_executed as f64 / opportunities_found as f64) * 100.0;
        }

        // Actualizar chart data
        let now = Utc::now();
        let mut chart = self.chart_data.lock().unwrap();
        chart.profit_over_time.push_back((now, total_profit));
        chart.balance_over_time.push_back((now, balance));
        chart.opportunities_over_time.push_back((now, opportunities_found));
        chart.success_rate_over_time.push_back((now, status.success_rate));

        // Verificar alertas
        self.check_alerts(&status);
    }

    /// Actualizar estado de APIs
    pub fn update_api_status(&self, api_name: String, is_online: bool, response_time_ms: u64, error: Option<String>) {
        let mut status = self.system_status.lock().unwrap();
        
        let api_status = status.api_status.entry(api_name.clone()).or_insert_with(|| ApiStatus {
            name: api_name.clone(),
            is_online: false,
            response_time_ms: 0,
            success_rate: 100.0,
            last_error: None,
            uptime_percentage: 100.0,
        });

        api_status.is_online = is_online;
        api_status.response_time_ms = response_time_ms;
        api_status.last_error = error.clone();

        // Actualizar chart data para API response times
        let now = Utc::now();
        let mut chart = self.chart_data.lock().unwrap();
        chart.api_response_times
            .entry(api_name.clone())
            .or_insert_with(VecDeque::new)
            .push_back((now, response_time_ms));

        // Log del estado de la API
        let level = if is_online { LogLevel::Info } else { LogLevel::Warning };
        let message = if is_online {
            format!("API {} online ({}ms)", api_name, response_time_ms)
        } else {
            format!("API {} offline: {:?}", api_name, error)
        };

        self.add_log_entry(level, message, "API Monitor".to_string(), None);
    }

    /// Registrar un trade completado
    pub fn record_trade(&self, trade: &TradeResult) {
        let level = if trade.success { LogLevel::Success } else { LogLevel::Error };
        let message = format!(
            "Trade {}: {:.6} SOL profit ({:.1}%)", 
            trade.trade_id, 
            trade.profit_sol, 
            trade.profit_percentage
        );

        let data = serde_json::json!({
            "trade_id": trade.trade_id,
            "token_pair": trade.token_pair,
            "profit_sol": trade.profit_sol,
            "profit_percentage": trade.profit_percentage,
            "dex_used": trade.dex_used,
            "execution_time_ms": trade.execution_time_ms
        });

        self.add_log_entry(level, message, "Trading Engine".to_string(), Some(data));
    }

    /// Registrar error del sistema
    pub fn record_error(&self, component: String, error_message: String) {
        {
            let mut status = self.system_status.lock().unwrap();
            status.last_error = Some(error_message.clone());
        }

        self.add_log_entry(LogLevel::Error, error_message, component, None);
    }

    /// Añadir entrada de log
    pub fn add_log_entry(&self, level: LogLevel, message: String, component: String, data: Option<serde_json::Value>) {
        let entry = DashboardLogEntry {
            timestamp: Utc::now(),
            level: level.clone(),
            message: message.clone(),
            component,
            data,
        };

        {
            let mut logs = self.log_entries.lock().unwrap();
            logs.push_back(entry.clone());
            
            // Limitar tamaño
            while logs.len() > self.config.max_log_entries {
                logs.pop_front();
            }
        }

        // Log en consola según el nivel
        match level {
            LogLevel::Info => info!("📊 [{}] {}", entry.component, message),
            LogLevel::Warning => warn!("⚠️ [{}] {}", entry.component, message),
            LogLevel::Error => error!("❌ [{}] {}", entry.component, message),
            LogLevel::Success => info!("✅ [{}] {}", entry.component, message),
            LogLevel::Debug => debug!("🔧 [{}] {}", entry.component, message),
        }
    }

    /// Verificar alertas basadas en thresholds
    fn check_alerts(&self, status: &SystemStatus) {
        let thresholds = &self.config.alert_thresholds;

        // Alert: Success rate bajo
        if status.success_rate < thresholds.min_success_rate && status.opportunities_found > 10 {
            self.add_log_entry(
                LogLevel::Warning,
                format!("Success rate bajo: {:.1}% (min: {:.1}%)", status.success_rate, thresholds.min_success_rate),
                "Alert System".to_string(),
                None
            );
        }

        // Alert: Profit por hora bajo
        let hours_running = status.uptime_seconds as f64 / 3600.0;
        if hours_running > 1.0 {
            let profit_per_hour = status.total_profit / hours_running;
            if profit_per_hour < thresholds.min_profit_per_hour {
                self.add_log_entry(
                    LogLevel::Warning,
                    format!("Profit/hora bajo: {:.6} SOL (min: {:.6} SOL)", profit_per_hour, thresholds.min_profit_per_hour),
                    "Alert System".to_string(),
                    None
                );
            }
        }

        // Alert: APIs offline
        for (api_name, api_status) in &status.api_status {
            if !api_status.is_online {
                self.add_log_entry(
                    LogLevel::Error,
                    format!("API {} está offline", api_name),
                    "Alert System".to_string(),
                    None
                );
            }
        }
    }

    /// Mostrar dashboard en consola
    fn print_console_dashboard(
        system_status: &Arc<Mutex<SystemStatus>>,
        log_entries: &Arc<Mutex<VecDeque<DashboardLogEntry>>>,
    ) {
        let status = system_status.lock().unwrap();
        let logs = log_entries.lock().unwrap();

        // Clear screen y mostrar dashboard
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        
        println!("📊 ═══════════════════════════════════════════════════════════════");
        println!("📊   SNIPERFORGE ARBITRAGE - REAL-TIME DASHBOARD");
        println!("📊 ═══════════════════════════════════════════════════════════════");
        println!();
        
        // Estado general
        println!("💰 BALANCE & PROFIT:");
        println!("   • Current Balance: {:.9} SOL", status.current_balance);
        println!("   • Total Profit: {:+.9} SOL", status.total_profit);
        println!("   • Uptime: {}h {}m {}s", 
                 status.uptime_seconds / 3600,
                 (status.uptime_seconds % 3600) / 60,
                 status.uptime_seconds % 60);
        println!();

        // Estadísticas de trading
        println!("📈 TRADING STATS:");
        println!("   • Cycles Completed: {}", status.cycles_completed);
        println!("   • Opportunities Found: {}", status.opportunities_found);
        println!("   • Opportunities Executed: {}", status.opportunities_executed);
        println!("   • Success Rate: {:.1}%", status.success_rate);
        println!();

        // Estado de APIs
        println!("📡 API STATUS:");
        for (name, api) in &status.api_status {
            let status_emoji = if api.is_online { "✅" } else { "❌" };
            println!("   • {}: {} {}ms", name, status_emoji, api.response_time_ms);
        }
        println!();

        // Integradores activos
        println!("🔧 ACTIVE INTEGRATORS:");
        for integrator in &status.active_integrators {
            println!("   • ✅ {}", integrator);
        }
        println!();

        // Últimos logs (5 más recientes)
        println!("📝 RECENT ACTIVITY:");
        for entry in logs.iter().rev().take(5) {
            let emoji = match entry.level {
                LogLevel::Info => "ℹ️",
                LogLevel::Warning => "⚠️",
                LogLevel::Error => "❌",
                LogLevel::Success => "✅",
                LogLevel::Debug => "🔧",
            };
            println!("   {} [{}] {} - {}", 
                     emoji,
                     entry.timestamp.format("%H:%M:%S"),
                     entry.component,
                     entry.message);
        }
        
        println!();
        println!("📊 ═══════════════════════════════════════════════════════════════");
        println!("    Last Update: {} | Status: {} RUNNING", 
                 status.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                 if status.is_running { "🟢" } else { "🔴" });
        println!("📊 ═══════════════════════════════════════════════════════════════");
    }

    /// Obtener estado actual del sistema
    pub fn get_system_status(&self) -> SystemStatus {
        self.system_status.lock().unwrap().clone()
    }

    /// Obtener logs recientes
    pub fn get_recent_logs(&self, count: usize) -> Vec<DashboardLogEntry> {
        let logs = self.log_entries.lock().unwrap();
        logs.iter().rev().take(count).cloned().collect()
    }

    /// Generar reporte de dashboard
    pub fn generate_dashboard_report(&self) -> String {
        let status = self.system_status.lock().unwrap();
        let logs = self.log_entries.lock().unwrap();
        
        let mut report = String::new();
        
        report.push_str("📊 DASHBOARD REPORT\n");
        report.push_str("==================\n\n");
        
        report.push_str(&format!("Timestamp: {}\n", status.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        report.push_str(&format!("Uptime: {}s\n", status.uptime_seconds));
        report.push_str(&format!("Balance: {:.9} SOL\n", status.current_balance));
        report.push_str(&format!("Total Profit: {:+.9} SOL\n", status.total_profit));
        report.push_str(&format!("Success Rate: {:.1}%\n", status.success_rate));
        
        report.push_str(&format!("\nAPI Status ({} APIs):\n", status.api_status.len()));
        for (name, api) in &status.api_status {
            report.push_str(&format!("• {}: {} ({}ms)\n", 
                            name, 
                            if api.is_online { "ONLINE" } else { "OFFLINE" }, 
                            api.response_time_ms));
        }
        
        report.push_str(&format!("\nRecent Logs ({} entries):\n", logs.len()));
        for entry in logs.iter().rev().take(10) {
            report.push_str(&format!("[{}] {}: {}\n", 
                            entry.timestamp.format("%H:%M:%S"),
                            entry.component,
                            entry.message));
        }
        
        report
    }

    /// Detener el dashboard
    pub fn stop(&self) {
        info!("🛑 Deteniendo Real-Time Trading Dashboard");
        *self.is_running.lock().unwrap() = false;
        
        let mut status = self.system_status.lock().unwrap();
        status.is_running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let config = DashboardConfig::default();
        let dashboard = RealTimeTradingDashboard::new(config, 1.0);
        
        let status = dashboard.get_system_status();
        assert_eq!(status.current_balance, 1.0);
        assert!(status.is_running);
    }

    #[test]
    fn test_log_entry() {
        let config = DashboardConfig::default();
        let dashboard = RealTimeTradingDashboard::new(config, 1.0);
        
        dashboard.add_log_entry(
            LogLevel::Info, 
            "Test message".to_string(), 
            "Test Component".to_string(), 
            None
        );
        
        let logs = dashboard.get_recent_logs(1);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "Test message");
    }
}
