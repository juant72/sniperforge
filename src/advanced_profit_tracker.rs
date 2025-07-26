// ================================================================================
// ADVANCED PROFIT TRACKER - ACCIÓN 7.2
// ================================================================================
// Sistema avanzado de tracking de profits para arbitrage_phase45_clean
// Métricas detalladas, análisis de tendencias y reporting en tiempo real
// ================================================================================

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, Instant};
use tracing::{info, warn, error, debug};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Resultado de una transacción de trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub trade_id: String,
    pub timestamp: DateTime<Utc>,
    pub token_pair: String,
    pub input_amount: f64,
    pub output_amount: f64,
    pub profit_sol: f64,
    pub profit_percentage: f64,
    pub gas_cost: f64,
    pub net_profit: f64,
    pub execution_time_ms: u64,
    pub success: bool,
    pub dex_used: String,
    pub slippage: f64,
}

impl TradeResult {
    pub fn new(
        trade_id: String,
        token_pair: String,
        input_amount: f64,
        output_amount: f64,
        gas_cost: f64,
        execution_time_ms: u64,
        dex_used: String,
    ) -> Self {
        let profit_sol = output_amount - input_amount - gas_cost;
        let profit_percentage = if input_amount > 0.0 {
            (profit_sol / input_amount) * 100.0
        } else { 0.0 };
        
        Self {
            trade_id,
            timestamp: Utc::now(),
            token_pair,
            input_amount,
            output_amount,
            profit_sol,
            profit_percentage,
            gas_cost,
            net_profit: profit_sol,
            execution_time_ms,
            success: profit_sol > 0.0,
            dex_used,
            slippage: 0.0, // Se calcula después
        }
    }
}

/// Estadísticas de performance del trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStats {
    pub total_trades: u64,
    pub successful_trades: u64,
    pub failed_trades: u64,
    pub total_profit_sol: f64,
    pub total_gas_costs: f64,
    pub net_profit_sol: f64,
    pub average_profit_per_trade: f64,
    pub success_rate: f64,
    pub best_trade_profit: f64,
    pub worst_trade_loss: f64,
    pub average_execution_time_ms: f64,
    pub total_volume_sol: f64,
    pub roi_percentage: f64,
}

impl Default for TradingStats {
    fn default() -> Self {
        Self {
            total_trades: 0,
            successful_trades: 0,
            failed_trades: 0,
            total_profit_sol: 0.0,
            total_gas_costs: 0.0,
            net_profit_sol: 0.0,
            average_profit_per_trade: 0.0,
            success_rate: 0.0,
            best_trade_profit: 0.0,
            worst_trade_loss: 0.0,
            average_execution_time_ms: 0.0,
            total_volume_sol: 0.0,
            roi_percentage: 0.0,
        }
    }
}

/// Configuración del profit tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitTrackerConfig {
    pub max_history_size: usize,
    pub reporting_interval_seconds: u64,
    pub alert_loss_threshold: f64,
    pub alert_profit_threshold: f64,
    pub performance_window_minutes: u64,
    pub save_to_file: bool,
    pub file_path: String,
}

impl Default for ProfitTrackerConfig {
    fn default() -> Self {
        Self {
            max_history_size: 1000,
            reporting_interval_seconds: 60,
            alert_loss_threshold: -0.01, // Alert si pérdida > 0.01 SOL
            alert_profit_threshold: 0.1,  // Alert si profit > 0.1 SOL
            performance_window_minutes: 30,
            save_to_file: true,
            file_path: "trading_results.json".to_string(),
        }
    }
}

/// Análisis de tendencias de profit
#[derive(Debug, Clone)]
pub struct ProfitTrendAnalysis {
    pub hourly_profits: Vec<f64>,
    pub moving_average_5min: f64,
    pub moving_average_15min: f64,
    pub moving_average_1hour: f64,
    pub trend_direction: TrendDirection,
    pub volatility_score: f64,
    pub consistency_score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    StronglyUp,
    Up,
    Stable,
    Down,
    StronglyDown,
}

/// Tracker principal de profits
pub struct AdvancedProfitTracker {
    config: ProfitTrackerConfig,
    trade_history: VecDeque<TradeResult>,
    stats: TradingStats,
    hourly_profits: VecDeque<(DateTime<Utc>, f64)>,
    dex_performance: HashMap<String, TradingStats>,
    token_pair_performance: HashMap<String, TradingStats>,
    daily_totals: HashMap<String, f64>, // YYYY-MM-DD -> profit
    start_time: Instant,
    initial_balance: f64,
    current_balance: f64,
}

impl AdvancedProfitTracker {
    /// Crear nuevo profit tracker
    pub fn new(config: ProfitTrackerConfig, initial_balance: f64) -> Self {
        info!("💰 Inicializando Advanced Profit Tracker");
        info!("   • History size: {}", config.max_history_size);
        info!("   • Reporting interval: {}s", config.reporting_interval_seconds);
        info!("   • Initial balance: {:.9} SOL", initial_balance);

        Self {
            config,
            trade_history: VecDeque::new(),
            stats: TradingStats::default(),
            hourly_profits: VecDeque::new(),
            dex_performance: HashMap::new(),
            token_pair_performance: HashMap::new(),
            daily_totals: HashMap::new(),
            start_time: Instant::now(),
            initial_balance,
            current_balance: initial_balance,
        }
    }

    /// Registrar un nuevo trade result
    pub fn record_trade(&mut self, trade: TradeResult) -> Result<()> {
        info!("📊 Registrando trade: {} - Profit: {:.6} SOL", trade.trade_id, trade.profit_sol);

        // Actualizar balance actual
        self.current_balance += trade.net_profit;

        // Actualizar estadísticas generales
        self.update_general_stats(&trade);

        // Actualizar estadísticas por DEX
        self.update_dex_stats(&trade);

        // Actualizar estadísticas por token pair
        self.update_token_pair_stats(&trade);

        // Actualizar profits por hora
        self.update_hourly_profits(&trade);

        // Actualizar totales diarios
        self.update_daily_totals(&trade);

        // Agregar a historial
        self.trade_history.push_back(trade.clone());

        // Limitar tamaño del historial
        if self.trade_history.len() > self.config.max_history_size {
            self.trade_history.pop_front();
        }

        // Alertas por thresholds
        self.check_profit_alerts(&trade);

        // Guardar a archivo si está configurado
        if self.config.save_to_file {
            self.save_to_file()?;
        }

        Ok(())
    }

    /// Actualizar estadísticas generales
    fn update_general_stats(&mut self, trade: &TradeResult) {
        self.stats.total_trades += 1;
        
        if trade.success {
            self.stats.successful_trades += 1;
        } else {
            self.stats.failed_trades += 1;
        }

        self.stats.total_profit_sol += trade.profit_sol;
        self.stats.total_gas_costs += trade.gas_cost;
        self.stats.net_profit_sol += trade.net_profit;
        self.stats.total_volume_sol += trade.input_amount;

        // Success rate
        self.stats.success_rate = (self.stats.successful_trades as f64 / self.stats.total_trades as f64) * 100.0;

        // Average profit per trade
        self.stats.average_profit_per_trade = self.stats.net_profit_sol / self.stats.total_trades as f64;

        // Best/worst trades
        if trade.profit_sol > self.stats.best_trade_profit {
            self.stats.best_trade_profit = trade.profit_sol;
        }
        if trade.profit_sol < self.stats.worst_trade_loss {
            self.stats.worst_trade_loss = trade.profit_sol;
        }

        // Average execution time
        let total_execution_time: u64 = self.trade_history.iter().map(|t| t.execution_time_ms).sum();
        self.stats.average_execution_time_ms = total_execution_time as f64 / self.trade_history.len() as f64;

        // ROI
        if self.initial_balance > 0.0 {
            self.stats.roi_percentage = (self.stats.net_profit_sol / self.initial_balance) * 100.0;
        }
    }

    /// Actualizar estadísticas por DEX
    fn update_dex_stats(&mut self, trade: &TradeResult) {
        let dex_stats = self.dex_performance.entry(trade.dex_used.clone()).or_insert_with(TradingStats::default);
        
        dex_stats.total_trades += 1;
        if trade.success {
            dex_stats.successful_trades += 1;
        } else {
            dex_stats.failed_trades += 1;
        }
        
        dex_stats.total_profit_sol += trade.profit_sol;
        dex_stats.net_profit_sol += trade.net_profit;
        dex_stats.total_volume_sol += trade.input_amount;
        dex_stats.success_rate = (dex_stats.successful_trades as f64 / dex_stats.total_trades as f64) * 100.0;
        dex_stats.average_profit_per_trade = dex_stats.net_profit_sol / dex_stats.total_trades as f64;
    }

    /// Actualizar estadísticas por token pair
    fn update_token_pair_stats(&mut self, trade: &TradeResult) {
        let pair_stats = self.token_pair_performance.entry(trade.token_pair.clone()).or_insert_with(TradingStats::default);
        
        pair_stats.total_trades += 1;
        if trade.success {
            pair_stats.successful_trades += 1;
        } else {
            pair_stats.failed_trades += 1;
        }
        
        pair_stats.total_profit_sol += trade.profit_sol;
        pair_stats.net_profit_sol += trade.net_profit;
        pair_stats.total_volume_sol += trade.input_amount;
        pair_stats.success_rate = (pair_stats.successful_trades as f64 / pair_stats.total_trades as f64) * 100.0;
        pair_stats.average_profit_per_trade = pair_stats.net_profit_sol / pair_stats.total_trades as f64;
    }

    /// Actualizar profits por hora
    fn update_hourly_profits(&mut self, trade: &TradeResult) {
        let _hour_key = trade.timestamp.format("%Y-%m-%d %H:00").to_string();
        self.hourly_profits.push_back((trade.timestamp, trade.profit_sol));
        
        // Mantener solo las últimas 24 horas
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        while let Some((timestamp, _)) = self.hourly_profits.front() {
            if *timestamp < cutoff {
                self.hourly_profits.pop_front();
            } else {
                break;
            }
        }
    }

    /// Actualizar totales diarios
    fn update_daily_totals(&mut self, trade: &TradeResult) {
        let day_key = trade.timestamp.format("%Y-%m-%d").to_string();
        *self.daily_totals.entry(day_key).or_insert(0.0) += trade.profit_sol;
    }

    /// Verificar alertas de profit/loss
    fn check_profit_alerts(&self, trade: &TradeResult) {
        if trade.profit_sol >= self.config.alert_profit_threshold {
            info!("🎉 ALERTA PROFIT ALTO: Trade {} generó {:.6} SOL (≥{:.3})", 
                  trade.trade_id, trade.profit_sol, self.config.alert_profit_threshold);
        }
        
        if trade.profit_sol <= self.config.alert_loss_threshold {
            warn!("⚠️ ALERTA PÉRDIDA: Trade {} perdió {:.6} SOL (≤{:.3})", 
                  trade.trade_id, trade.profit_sol, self.config.alert_loss_threshold);
        }
    }

    /// Obtener estadísticas actuales
    pub fn get_stats(&self) -> &TradingStats {
        &self.stats
    }

    /// Obtener performance por DEX
    pub fn get_dex_performance(&self) -> &HashMap<String, TradingStats> {
        &self.dex_performance
    }

    /// Obtener performance por token pair
    pub fn get_token_pair_performance(&self) -> &HashMap<String, TradingStats> {
        &self.token_pair_performance
    }

    /// Análisis de tendencias
    pub fn analyze_trends(&self) -> ProfitTrendAnalysis {
        let recent_profits: Vec<f64> = self.hourly_profits.iter()
            .map(|(_, profit)| *profit)
            .collect();

        let moving_avg_5min = self.calculate_moving_average(&recent_profits, 5);
        let moving_avg_15min = self.calculate_moving_average(&recent_profits, 15);
        let moving_avg_1hour = self.calculate_moving_average(&recent_profits, 60);

        let trend_direction = self.determine_trend(&recent_profits);
        let volatility_score = self.calculate_volatility(&recent_profits);
        let consistency_score = self.calculate_consistency(&recent_profits);

        ProfitTrendAnalysis {
            hourly_profits: recent_profits,
            moving_average_5min: moving_avg_5min,
            moving_average_15min: moving_avg_15min,
            moving_average_1hour: moving_avg_1hour,
            trend_direction,
            volatility_score,
            consistency_score,
        }
    }

    /// Calcular promedio móvil
    fn calculate_moving_average(&self, data: &[f64], window: usize) -> f64 {
        if data.is_empty() || window == 0 {
            return 0.0;
        }
        
        let window_size = std::cmp::min(window, data.len());
        let recent_data = &data[data.len().saturating_sub(window_size)..];
        
        recent_data.iter().sum::<f64>() / recent_data.len() as f64
    }

    /// Determinar dirección de tendencia
    fn determine_trend(&self, data: &[f64]) -> TrendDirection {
        if data.len() < 3 {
            return TrendDirection::Stable;
        }

        let recent_avg = self.calculate_moving_average(data, 5);
        let older_avg = self.calculate_moving_average(&data[..data.len()/2], 5);
        
        let change_percentage = if older_avg != 0.0 {
            ((recent_avg - older_avg) / older_avg.abs()) * 100.0
        } else { 0.0 };

        match change_percentage {
            x if x > 10.0 => TrendDirection::StronglyUp,
            x if x > 2.0 => TrendDirection::Up,
            x if x < -10.0 => TrendDirection::StronglyDown,
            x if x < -2.0 => TrendDirection::Down,
            _ => TrendDirection::Stable,
        }
    }

    /// Calcular volatilidad
    fn calculate_volatility(&self, data: &[f64]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        variance.sqrt()
    }

    /// Calcular consistencia (0-100)
    fn calculate_consistency(&self, data: &[f64]) -> f64 {
        if data.len() < 2 {
            return 100.0;
        }

        let positive_trades = data.iter().filter(|&&x| x > 0.0).count();
        (positive_trades as f64 / data.len() as f64) * 100.0
    }

    /// Generar reporte completo
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        let uptime = self.start_time.elapsed();
        
        report.push_str("💰 REPORTE AVANZADO DE PROFIT TRACKING\n");
        report.push_str("=====================================\n\n");
        
        // Estadísticas generales
        report.push_str("📊 ESTADÍSTICAS GENERALES:\n");
        report.push_str(&format!("• Uptime: {:.1} minutos\n", uptime.as_secs_f64() / 60.0));
        report.push_str(&format!("• Total trades: {}\n", self.stats.total_trades));
        report.push_str(&format!("• Success rate: {:.1}%\n", self.stats.success_rate));
        report.push_str(&format!("• Net profit: {:.6} SOL\n", self.stats.net_profit_sol));
        report.push_str(&format!("• ROI: {:.2}%\n", self.stats.roi_percentage));
        report.push_str(&format!("• Avg profit/trade: {:.6} SOL\n", self.stats.average_profit_per_trade));
        report.push_str(&format!("• Best trade: {:.6} SOL\n", self.stats.best_trade_profit));
        report.push_str(&format!("• Worst trade: {:.6} SOL\n", self.stats.worst_trade_loss));
        
        // Balance tracking
        report.push_str(&format!("• Initial balance: {:.6} SOL\n", self.initial_balance));
        report.push_str(&format!("• Current balance: {:.6} SOL\n", self.current_balance));
        report.push_str(&format!("• Total change: {:.6} SOL\n", self.current_balance - self.initial_balance));
        
        // Performance por DEX
        if !self.dex_performance.is_empty() {
            report.push_str("\n📈 PERFORMANCE POR DEX:\n");
            for (dex, stats) in &self.dex_performance {
                report.push_str(&format!("• {}: {} trades, {:.2}% success, {:.6} SOL profit\n", 
                                dex, stats.total_trades, stats.success_rate, stats.net_profit_sol));
            }
        }
        
        // Performance por token pair
        if !self.token_pair_performance.is_empty() {
            report.push_str("\n🪙 TOP TOKEN PAIRS:\n");
            let mut sorted_pairs: Vec<_> = self.token_pair_performance.iter().collect();
            sorted_pairs.sort_by(|a, b| b.1.net_profit_sol.partial_cmp(&a.1.net_profit_sol).unwrap());
            
            for (pair, stats) in sorted_pairs.iter().take(5) {
                report.push_str(&format!("• {}: {:.6} SOL ({} trades)\n", 
                                pair, stats.net_profit_sol, stats.total_trades));
            }
        }
        
        // Tendencias
        let trends = self.analyze_trends();
        report.push_str(&format!("\n📊 ANÁLISIS DE TENDENCIAS:\n"));
        report.push_str(&format!("• Dirección: {:?}\n", trends.trend_direction));
        report.push_str(&format!("• Volatilidad: {:.4}\n", trends.volatility_score));
        report.push_str(&format!("• Consistencia: {:.1}%\n", trends.consistency_score));
        
        report
    }

    /// Guardar datos a archivo
    fn save_to_file(&self) -> Result<()> {
        let data = serde_json::json!({
            "stats": self.stats,
            "dex_performance": self.dex_performance,
            "token_pair_performance": self.token_pair_performance,
            "daily_totals": self.daily_totals,
            "recent_trades": self.trade_history.iter().rev().take(50).collect::<Vec<_>>()
        });
        
        std::fs::write(&self.config.file_path, serde_json::to_string_pretty(&data)?)?;
        debug!("💾 Datos guardados en {}", self.config.file_path);
        Ok(())
    }

    /// Actualizar balance actual manualmente
    pub fn update_balance(&mut self, new_balance: f64) {
        let change = new_balance - self.current_balance;
        self.current_balance = new_balance;
        
        if change.abs() > 0.000001 {
            info!("💰 Balance actualizado: {:.9} SOL (cambio: {:+.9} SOL)", 
                  new_balance, change);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profit_tracker_creation() {
        let config = ProfitTrackerConfig::default();
        let tracker = AdvancedProfitTracker::new(config, 1.0);
        assert_eq!(tracker.initial_balance, 1.0);
        assert_eq!(tracker.current_balance, 1.0);
    }

    #[test]
    fn test_trade_recording() {
        let config = ProfitTrackerConfig::default();
        let mut tracker = AdvancedProfitTracker::new(config, 1.0);
        
        let trade = TradeResult::new(
            "test_1".to_string(),
            "SOL/USDC".to_string(),
            0.1,
            0.105,
            0.001,
            150,
            "Jupiter".to_string()
        );
        
        let result = tracker.record_trade(trade);
        assert!(result.is_ok());
        assert_eq!(tracker.stats.total_trades, 1);
    }
}
