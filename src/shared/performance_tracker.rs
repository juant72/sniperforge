/// Performance Tracking System for Automated Trading
/// 
/// Tracks trading performance metrics, generates reports, and provides analytics

use anyhow::{Result, anyhow};
use std::collections::VecDeque;
use tracing::{info, debug};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Individual trade metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeMetric {
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub profit_loss: f64,
    pub execution_time_ms: u64,
    pub slippage: f64,
    pub gas_fee: f64,
}

/// Performance summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_trades: u32,
    pub successful_trades: u32,
    pub failed_trades: u32,
    pub success_rate: f64,
    pub total_profit_loss: f64,
    pub average_profit: f64,
    pub average_loss: f64,
    pub best_trade: f64,
    pub worst_trade: f64,
    pub total_gas_fees: f64,
    pub average_execution_time: f64,
    pub average_slippage: f64,
}

/// Performance tracker
pub struct PerformanceTracker {
    trade_history: VecDeque<TradeMetric>,
    max_history_size: usize,
    session_start: DateTime<Utc>,
}

impl PerformanceTracker {
    /// Create new performance tracker
    pub fn new() -> Self {
        Self {
            trade_history: VecDeque::new(),
            max_history_size: 1000, // Keep last 1000 trades
            session_start: Utc::now(),
        }
    }

    /// Add a trade metric to tracking
    pub async fn add_trade_metric(&mut self, metric: TradeMetric) -> Result<()> {
        debug!("📊 Recording trade metric: success={}, P&L=${:.2}", 
               metric.success, metric.profit_loss);

        self.trade_history.push_back(metric);

        // Maintain maximum history size
        while self.trade_history.len() > self.max_history_size {
            self.trade_history.pop_front();
        }

        Ok(())
    }

    /// Calculate current performance statistics
    pub async fn calculate_stats(&self) -> Result<PerformanceStats> {
        if self.trade_history.is_empty() {
            return Ok(PerformanceStats {
                total_trades: 0,
                successful_trades: 0,
                failed_trades: 0,
                success_rate: 0.0,
                total_profit_loss: 0.0,
                average_profit: 0.0,
                average_loss: 0.0,
                best_trade: 0.0,
                worst_trade: 0.0,
                total_gas_fees: 0.0,
                average_execution_time: 0.0,
                average_slippage: 0.0,
            });
        }

        let total_trades = self.trade_history.len() as u32;
        let successful_trades = self.trade_history.iter().filter(|t| t.success).count() as u32;
        let failed_trades = total_trades - successful_trades;
        
        let success_rate = if total_trades > 0 {
            (successful_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let total_profit_loss: f64 = self.trade_history.iter().map(|t| t.profit_loss).sum();
        let total_gas_fees: f64 = self.trade_history.iter().map(|t| t.gas_fee).sum();
        
        let profitable_trades: Vec<f64> = self.trade_history
            .iter()
            .filter(|t| t.success && t.profit_loss > 0.0)
            .map(|t| t.profit_loss)
            .collect();
            
        let losing_trades: Vec<f64> = self.trade_history
            .iter()
            .filter(|t| !t.success || t.profit_loss < 0.0)
            .map(|t| t.profit_loss)
            .collect();

        let average_profit = if !profitable_trades.is_empty() {
            profitable_trades.iter().sum::<f64>() / profitable_trades.len() as f64
        } else {
            0.0
        };

        let average_loss = if !losing_trades.is_empty() {
            losing_trades.iter().sum::<f64>() / losing_trades.len() as f64
        } else {
            0.0
        };

        let best_trade = profitable_trades.iter().cloned().fold(0.0, f64::max);
        let worst_trade = losing_trades.iter().cloned().fold(0.0, f64::min);

        let average_execution_time = if total_trades > 0 {
            self.trade_history.iter().map(|t| t.execution_time_ms as f64).sum::<f64>() / total_trades as f64
        } else {
            0.0
        };

        let average_slippage = if total_trades > 0 {
            self.trade_history.iter().map(|t| t.slippage).sum::<f64>() / total_trades as f64
        } else {
            0.0
        };

        Ok(PerformanceStats {
            total_trades,
            successful_trades,
            failed_trades,
            success_rate,
            total_profit_loss,
            average_profit,
            average_loss,
            best_trade,
            worst_trade,
            total_gas_fees,
            average_execution_time,
            average_slippage,
        })
    }

    /// Generate detailed performance summary
    pub async fn generate_summary(&self) -> Result<String> {
        let stats = self.calculate_stats().await?;
        let session_duration = Utc::now().signed_duration_since(self.session_start);
        
        let summary = format!(
            "📊 AUTOMATED TRADING PERFORMANCE SUMMARY\n\
             \n\
             ⏱️  Session Duration: {} hours, {} minutes\n\
             📈 Total Trades: {}\n\
             ✅ Successful: {} ({:.1}%)\n\
             ❌ Failed: {} ({:.1}%)\n\
             \n\
             💰 PROFIT & LOSS\n\
             💵 Total P&L: ${:.2}\n\
             📊 Average Profit: ${:.2}\n\
             📉 Average Loss: ${:.2}\n\
             🏆 Best Trade: ${:.2}\n\
             💸 Worst Trade: ${:.2}\n\
             ⛽ Total Gas Fees: ${:.2}\n\
             💎 Net Profit: ${:.2}\n\
             \n\
             ⚡ EXECUTION METRICS\n\
             🚀 Avg Execution Time: {:.1}ms\n\
             📊 Avg Slippage: {:.3}%\n\
             \n\
             📈 PERFORMANCE ANALYSIS\n\
             🎯 Win Rate: {:.1}%\n\
             💰 Profit Factor: {:.2}\n\
             📊 Trades per Hour: {:.1}\n\
             🔥 Status: {}",
            session_duration.num_hours(),
            session_duration.num_minutes() % 60,
            stats.total_trades,
            stats.successful_trades,
            stats.success_rate,
            stats.failed_trades,
            100.0 - stats.success_rate,
            stats.total_profit_loss,
            stats.average_profit,
            stats.average_loss,
            stats.best_trade,
            stats.worst_trade,
            stats.total_gas_fees,
            stats.total_profit_loss - stats.total_gas_fees,
            stats.average_execution_time,
            stats.average_slippage * 100.0,
            stats.success_rate,
            if stats.average_loss != 0.0 { stats.average_profit / stats.average_loss.abs() } else { 0.0 },
            if session_duration.num_hours() > 0 { 
                stats.total_trades as f64 / session_duration.num_hours() as f64 
            } else { 
                0.0 
            },
            self.get_performance_status(&stats)
        );

        Ok(summary)
    }

    /// Get performance status indicator
    fn get_performance_status(&self, stats: &PerformanceStats) -> &'static str {
        if stats.total_trades == 0 {
            "🔄 INITIALIZING"
        } else if stats.success_rate >= 80.0 && stats.total_profit_loss > 0.0 {
            "🔥 EXCELLENT"
        } else if stats.success_rate >= 60.0 && stats.total_profit_loss > 0.0 {
            "✅ GOOD"
        } else if stats.success_rate >= 40.0 || stats.total_profit_loss > -10.0 {
            "⚠️ AVERAGE"
        } else {
            "❌ NEEDS ATTENTION"
        }
    }

    /// Get recent trade history (last N trades)
    pub async fn get_recent_trades(&self, count: usize) -> Result<Vec<TradeMetric>> {
        let recent_count = count.min(self.trade_history.len());
        let recent_trades: Vec<TradeMetric> = self.trade_history
            .iter()
            .rev()
            .take(recent_count)
            .cloned()
            .collect();

        Ok(recent_trades)
    }

    /// Get trades within time window
    pub async fn get_trades_since(&self, since: DateTime<Utc>) -> Result<Vec<TradeMetric>> {
        let filtered_trades: Vec<TradeMetric> = self.trade_history
            .iter()
            .filter(|t| t.timestamp >= since)
            .cloned()
            .collect();

        Ok(filtered_trades)
    }

    /// Export performance data as JSON
    pub async fn export_data(&self) -> Result<String> {
        let stats = self.calculate_stats().await?;
        let recent_trades = self.get_recent_trades(50).await?; // Last 50 trades

        let export_data = serde_json::json!({
            "session_start": self.session_start,
            "export_time": Utc::now(),
            "performance_stats": stats,
            "recent_trades": recent_trades,
            "total_trade_count": self.trade_history.len()
        });

        Ok(serde_json::to_string_pretty(&export_data)?)
    }

    /// Get real-time performance metrics for monitoring
    pub async fn get_live_metrics(&self) -> Result<String> {
        if self.trade_history.is_empty() {
            return Ok("📊 No trades executed yet".to_string());
        }

        let stats = self.calculate_stats().await?;
        let last_trade = self.trade_history.back().unwrap();
        
        let metrics = format!(
            "📊 LIVE METRICS | Total: {} | Success: {:.0}% | P&L: ${:.2} | Last: ${:.2}",
            stats.total_trades,
            stats.success_rate,
            stats.total_profit_loss,
            last_trade.profit_loss
        );

        Ok(metrics)
    }

    /// Clear all performance data
    pub async fn reset(&mut self) -> Result<()> {
        self.trade_history.clear();
        self.session_start = Utc::now();
        info!("🔄 Performance tracker reset");
        Ok(())
    }

    /// Get profitability trend (last N trades)
    pub async fn get_profitability_trend(&self, window_size: usize) -> Result<Vec<f64>> {
        let window_size = window_size.min(self.trade_history.len());
        let trend: Vec<f64> = self.trade_history
            .iter()
            .rev()
            .take(window_size)
            .map(|t| t.profit_loss)
            .collect();

        Ok(trend)
    }

    /// Calculate Sharpe ratio (risk-adjusted returns)
    pub async fn calculate_sharpe_ratio(&self) -> Result<f64> {
        if self.trade_history.len() < 2 {
            return Ok(0.0);
        }

        let returns: Vec<f64> = self.trade_history.iter().map(|t| t.profit_loss).collect();
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;
        
        let std_dev = variance.sqrt();
        
        let sharpe_ratio = if std_dev > 0.0 {
            mean_return / std_dev
        } else {
            0.0
        };

        Ok(sharpe_ratio)
    }
}
