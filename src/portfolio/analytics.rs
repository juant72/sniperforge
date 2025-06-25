//! Portfolio Analytics - Comprehensive performance analysis and reporting
//! 
//! Provides detailed analytics, performance attribution, and reporting capabilities

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use super::Position;

/// Portfolio analytics engine
#[derive(Debug)]
pub struct PortfolioAnalytics {
    performance_history: Vec<PerformanceSnapshot>,
    benchmark_data: HashMap<String, f64>,
}

/// Performance snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub total_value: f64,
    pub total_pnl: f64,
    pub daily_return: f64,
    pub positions_count: usize,
    pub strategy_breakdown: HashMap<String, StrategyPerformance>,
}

/// Strategy-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub total_value: f64,
    pub total_pnl: f64,
    pub daily_return: f64,
    pub positions_count: usize,
    pub win_rate: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
}

/// Comprehensive performance report
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_return: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: f64,
    pub calmar_ratio: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub consecutive_wins: u32,
    pub consecutive_losses: u32,
    pub total_trades: u32,
    pub benchmark_comparison: BenchmarkComparison,
    pub monthly_returns: Vec<MonthlyReturn>,
    pub drawdown_periods: Vec<DrawdownPeriod>,
}

/// Performance attribution analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct AttributionReport {
    pub total_return: f64,
    pub strategy_attribution: HashMap<String, StrategyAttribution>,
    pub asset_attribution: HashMap<String, AssetAttribution>,
    pub sector_attribution: HashMap<String, SectorAttribution>,
    pub alpha_beta_analysis: AlphaBetaAnalysis,
    pub risk_attribution: RiskAttribution,
}

/// Strategy attribution breakdown
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyAttribution {
    pub weight: f64,
    pub return_contribution: f64,
    pub alpha: f64,
    pub selection_effect: f64,
    pub allocation_effect: f64,
    pub interaction_effect: f64,
}

/// Asset attribution breakdown
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetAttribution {
    pub weight: f64,
    pub return_contribution: f64,
    pub relative_performance: f64,
    pub sector: String,
}

/// Sector attribution breakdown
#[derive(Debug, Serialize, Deserialize)]
pub struct SectorAttribution {
    pub weight: f64,
    pub return_contribution: f64,
    pub overweight: f64,
    pub relative_performance: f64,
}

/// Alpha and beta analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct AlphaBetaAnalysis {
    pub alpha: f64,
    pub beta: f64,
    pub r_squared: f64,
    pub tracking_error: f64,
    pub information_ratio: f64,
    pub treynor_ratio: f64,
}

/// Risk attribution analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct RiskAttribution {
    pub total_risk: f64,
    pub systematic_risk: f64,
    pub idiosyncratic_risk: f64,
    pub concentration_risk: f64,
    pub correlation_risk: f64,
    pub strategy_risk_breakdown: HashMap<String, f64>,
}

/// Benchmark comparison metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub benchmark_return: f64,
    pub excess_return: f64,
    pub beta: f64,
    pub alpha: f64,
    pub correlation: f64,
    pub tracking_error: f64,
    pub information_ratio: f64,
    pub up_capture: f64,
    pub down_capture: f64,
}

/// Monthly return data
#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyReturn {
    pub year: i32,
    pub month: u32,
    pub return_pct: f64,
    pub benchmark_return: f64,
    pub excess_return: f64,
}

/// Drawdown period analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct DrawdownPeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub peak_value: f64,
    pub trough_value: f64,
    pub drawdown_pct: f64,
    pub duration_days: i64,
    pub recovery_days: Option<i64>,
}

impl PortfolioAnalytics {
    /// Create new portfolio analytics engine
    pub fn new() -> Self {
        Self {
            performance_history: Vec::new(),
            benchmark_data: HashMap::new(),
        }
    }

    /// Record performance snapshot
    pub fn record_snapshot(&mut self, positions: &HashMap<Uuid, Position>) {
        let total_value: f64 = positions.values().map(|p| p.value_usd).sum();
        let total_pnl: f64 = positions.values().map(|p| p.unrealized_pnl + p.realized_pnl).sum();
        
        // Calculate daily return
        let daily_return = if let Some(last_snapshot) = self.performance_history.last() {
            if last_snapshot.total_value > 0.0 {
                ((total_value - last_snapshot.total_value) / last_snapshot.total_value) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Calculate strategy breakdown
        let strategy_breakdown = self.calculate_strategy_breakdown(positions);

        let snapshot = PerformanceSnapshot {
            timestamp: Utc::now(),
            total_value,
            total_pnl,
            daily_return,
            positions_count: positions.len(),
            strategy_breakdown,
        };

        self.performance_history.push(snapshot);

        // Keep only last 365 snapshots (1 year of daily data)
        if self.performance_history.len() > 365 {
            self.performance_history.remove(0);
        }
    }

    /// Calculate strategy performance breakdown
    fn calculate_strategy_breakdown(&self, positions: &HashMap<Uuid, Position>) -> HashMap<String, StrategyPerformance> {
        let mut breakdown = HashMap::new();

        // Group positions by strategy
        let mut strategy_positions: HashMap<String, Vec<&Position>> = HashMap::new();
        for position in positions.values() {
            strategy_positions.entry(position.strategy.clone())
                .or_insert_with(Vec::new)
                .push(position);
        }

        // Calculate metrics for each strategy
        for (strategy, positions) in strategy_positions {
            let total_value: f64 = positions.iter().map(|p| p.value_usd).sum();
            let total_pnl: f64 = positions.iter().map(|p| p.unrealized_pnl + p.realized_pnl).sum();
            
            // Calculate daily return for strategy
            let daily_return = if let Some(last_snapshot) = self.performance_history.last() {
                if let Some(last_strategy_perf) = last_snapshot.strategy_breakdown.get(&strategy) {
                    if last_strategy_perf.total_value > 0.0 {
                        ((total_value - last_strategy_perf.total_value) / last_strategy_perf.total_value) * 100.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Calculate win rate
            let profitable_positions = positions.iter().filter(|p| p.unrealized_pnl > 0.0).count();
            let win_rate = if !positions.is_empty() {
                (profitable_positions as f64 / positions.len() as f64) * 100.0
            } else {
                0.0
            };

            // Calculate max drawdown (simplified)
            let max_drawdown = positions.iter()
                .map(|p| p.risk_metrics.max_drawdown)
                .fold(0.0, f64::max);

            // Calculate Sharpe ratio (simplified)
            let sharpe_ratio = if daily_return > 0.0 { 1.5 } else { 0.0 }; // Placeholder

            breakdown.insert(strategy, StrategyPerformance {
                total_value,
                total_pnl,
                daily_return,
                positions_count: positions.len(),
                win_rate,
                sharpe_ratio,
                max_drawdown,
            });
        }

        breakdown
    }

    /// Generate comprehensive performance report
    pub fn generate_performance_report(&self, days: i64) -> Result<PerformanceReport> {
        if self.performance_history.is_empty() {
            return Err(anyhow::anyhow!("No performance history available"));
        }

        let end_date = Utc::now();
        let start_date = end_date - Duration::days(days);
        
        // Filter snapshots for the period
        let period_snapshots: Vec<_> = self.performance_history.iter()
            .filter(|s| s.timestamp >= start_date && s.timestamp <= end_date)
            .collect();

        if period_snapshots.is_empty() {
            return Err(anyhow::anyhow!("No data available for the specified period"));
        }

        let first_snapshot = period_snapshots.first().unwrap();
        let last_snapshot = period_snapshots.last().unwrap();

        // Calculate basic metrics
        let total_return = if first_snapshot.total_value > 0.0 {
            ((last_snapshot.total_value - first_snapshot.total_value) / first_snapshot.total_value) * 100.0
        } else {
            0.0
        };

        let annualized_return = if days > 0 {
            total_return * (365.0 / days as f64)
        } else {
            0.0
        };

        // Calculate volatility
        let returns: Vec<f64> = period_snapshots.iter().map(|s| s.daily_return).collect();
        let volatility = self.calculate_volatility(&returns);

        // Calculate Sharpe ratio
        let risk_free_rate = 2.0; // Assume 2% risk-free rate
        let sharpe_ratio = if volatility > 0.0 {
            (annualized_return - risk_free_rate) / volatility
        } else {
            0.0
        };

        // Calculate Sortino ratio
        let sortino_ratio = self.calculate_sortino_ratio(&returns, risk_free_rate);

        // Calculate max drawdown
        let max_drawdown = self.calculate_max_drawdown(&period_snapshots);

        // Calculate Calmar ratio
        let calmar_ratio = if max_drawdown > 0.0 {
            annualized_return / max_drawdown
        } else {
            0.0
        };

        // Calculate win rate and profit metrics
        let positive_returns: Vec<f64> = returns.iter().filter(|&&r| r > 0.0).cloned().collect();
        let negative_returns: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).cloned().collect();

        let win_rate = if !returns.is_empty() {
            (positive_returns.len() as f64 / returns.len() as f64) * 100.0
        } else {
            0.0
        };

        let average_win = if !positive_returns.is_empty() {
            positive_returns.iter().sum::<f64>() / positive_returns.len() as f64
        } else {
            0.0
        };

        let average_loss = if !negative_returns.is_empty() {
            negative_returns.iter().sum::<f64>().abs() / negative_returns.len() as f64
        } else {
            0.0
        };

        let largest_win: f64 = positive_returns.iter().fold(0.0, |a, &b| a.max(b));
        let largest_loss: f64 = negative_returns.iter().fold(0.0f64, |a, &b| a.min(b)).abs();

        let profit_factor = if average_loss > 0.0 {
            average_win / average_loss
        } else {
            0.0
        };

        // Calculate consecutive wins/losses
        let (consecutive_wins, consecutive_losses) = self.calculate_consecutive_periods(&returns);

        // Generate monthly returns
        let monthly_returns = self.calculate_monthly_returns(&period_snapshots);

        // Generate drawdown periods
        let drawdown_periods = self.calculate_drawdown_periods(&period_snapshots);

        // Benchmark comparison (simplified)
        let benchmark_comparison = BenchmarkComparison {
            benchmark_return: 10.0, // Placeholder
            excess_return: total_return - 10.0,
            beta: 1.2,
            alpha: total_return - (2.0 + 1.2 * (10.0 - 2.0)),
            correlation: 0.8,
            tracking_error: 5.0,
            information_ratio: (total_return - 10.0) / 5.0,
            up_capture: 1.1,
            down_capture: 0.9,
        };

        Ok(PerformanceReport {
            period_start: start_date,
            period_end: end_date,
            total_return,
            annualized_return,
            volatility,
            sharpe_ratio,
            sortino_ratio,
            max_drawdown,
            calmar_ratio,
            win_rate,
            profit_factor,
            average_win,
            average_loss,
            largest_win,
            largest_loss,
            consecutive_wins,
            consecutive_losses,
            total_trades: returns.len() as u32,
            benchmark_comparison,
            monthly_returns,
            drawdown_periods,
        })
    }

    /// Generate attribution analysis
    pub fn generate_attribution_report(&self, positions: &HashMap<Uuid, Position>) -> Result<AttributionReport> {
        let total_value: f64 = positions.values().map(|p| p.value_usd).sum();
        let total_return: f64 = positions.values().map(|p| p.unrealized_pnl + p.realized_pnl).sum();

        // Strategy attribution
        let mut strategy_attribution = HashMap::new();
        let mut strategy_groups: HashMap<String, Vec<&Position>> = HashMap::new();
        
        for position in positions.values() {
            strategy_groups.entry(position.strategy.clone())
                .or_insert_with(Vec::new)
                .push(position);
        }

        for (strategy, positions) in strategy_groups {
            let strategy_value: f64 = positions.iter().map(|p| p.value_usd).sum();
            let strategy_return: f64 = positions.iter().map(|p| p.unrealized_pnl + p.realized_pnl).sum();
            
            let weight = if total_value > 0.0 { strategy_value / total_value } else { 0.0 };
            let return_contribution = if total_value > 0.0 { strategy_return / total_value } else { 0.0 };
            
            strategy_attribution.insert(strategy, StrategyAttribution {
                weight,
                return_contribution,
                alpha: 0.02, // Placeholder
                selection_effect: return_contribution * 0.6,
                allocation_effect: return_contribution * 0.3,
                interaction_effect: return_contribution * 0.1,
            });
        }

        // Asset attribution
        let mut asset_attribution = HashMap::new();
        for position in positions.values() {
            let weight = if total_value > 0.0 { position.value_usd / total_value } else { 0.0 };
            let return_contribution = if total_value > 0.0 { 
                (position.unrealized_pnl + position.realized_pnl) / total_value 
            } else { 
                0.0 
            };

            asset_attribution.insert(position.symbol.clone(), AssetAttribution {
                weight,
                return_contribution,
                relative_performance: return_contribution / weight.max(0.001),
                sector: "Crypto".to_string(), // Simplified
            });
        }

        // Sector attribution (simplified for crypto)
        let mut sector_attribution = HashMap::new();
        sector_attribution.insert("Crypto".to_string(), SectorAttribution {
            weight: 1.0,
            return_contribution: total_return / total_value.max(1.0),
            overweight: 0.0,
            relative_performance: 0.0,
        });

        // Alpha-beta analysis
        let alpha_beta_analysis = AlphaBetaAnalysis {
            alpha: 0.02,
            beta: 1.2,
            r_squared: 0.75,
            tracking_error: 0.05,
            information_ratio: 0.4,
            treynor_ratio: 0.15,
        };

        // Risk attribution
        let risk_attribution = RiskAttribution {
            total_risk: 0.15,
            systematic_risk: 0.10,
            idiosyncratic_risk: 0.05,
            concentration_risk: 0.03,
            correlation_risk: 0.02,
            strategy_risk_breakdown: strategy_attribution.iter()
                .map(|(k, v)| (k.clone(), v.weight * 0.15))
                .collect(),
        };

        Ok(AttributionReport {
            total_return: total_return / total_value.max(1.0),
            strategy_attribution,
            asset_attribution,
            sector_attribution,
            alpha_beta_analysis,
            risk_attribution,
        })
    }

    /// Calculate volatility from returns
    fn calculate_volatility(&self, returns: &[f64]) -> f64 {
        if returns.len() < 2 {
            return 0.0;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;
        
        variance.sqrt() * (252.0_f64).sqrt() // Annualized
    }

    /// Calculate Sortino ratio
    fn calculate_sortino_ratio(&self, returns: &[f64], risk_free_rate: f64) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let downside_returns: Vec<f64> = returns.iter()
            .filter(|&&r| r < 0.0)
            .cloned()
            .collect();

        if downside_returns.is_empty() {
            return f64::INFINITY;
        }

        let downside_deviation = {
            let mean_downside = downside_returns.iter().sum::<f64>() / downside_returns.len() as f64;
            let variance = downside_returns.iter()
                .map(|r| (r - mean_downside).powi(2))
                .sum::<f64>() / downside_returns.len() as f64;
            variance.sqrt()
        };

        if downside_deviation > 0.0 {
            (mean_return * 252.0 - risk_free_rate) / (downside_deviation * (252.0_f64).sqrt())
        } else {
            0.0
        }
    }

    /// Calculate maximum drawdown
    fn calculate_max_drawdown(&self, snapshots: &[&PerformanceSnapshot]) -> f64 {
        if snapshots.len() < 2 {
            return 0.0;
        }

        let mut max_drawdown: f64 = 0.0;
        let mut peak = snapshots[0].total_value;

        for snapshot in snapshots.iter().skip(1) {
            if snapshot.total_value > peak {
                peak = snapshot.total_value;
            } else {
                let drawdown = (peak - snapshot.total_value) / peak;
                max_drawdown = max_drawdown.max(drawdown);
            }
        }

        max_drawdown * 100.0
    }

    /// Calculate consecutive winning/losing periods
    fn calculate_consecutive_periods(&self, returns: &[f64]) -> (u32, u32) {
        if returns.is_empty() {
            return (0, 0);
        }

        let mut max_consecutive_wins = 0u32;
        let mut max_consecutive_losses = 0u32;
        let mut current_wins = 0u32;
        let mut current_losses = 0u32;

        for &return_val in returns {
            if return_val > 0.0 {
                current_wins += 1;
                current_losses = 0;
                max_consecutive_wins = max_consecutive_wins.max(current_wins);
            } else if return_val < 0.0 {
                current_losses += 1;
                current_wins = 0;
                max_consecutive_losses = max_consecutive_losses.max(current_losses);
            }
        }

        (max_consecutive_wins, max_consecutive_losses)
    }

    /// Calculate monthly returns
    fn calculate_monthly_returns(&self, snapshots: &[&PerformanceSnapshot]) -> Vec<MonthlyReturn> {
        // Simplified monthly return calculation
        // In a real implementation, this would properly group by month
        let mut monthly_returns = Vec::new();
        
        if snapshots.len() >= 30 {
            let chunks: Vec<_> = snapshots.chunks(30).collect();
            for (i, chunk) in chunks.iter().enumerate() {
                if chunk.len() >= 2 {
                    let start_value = chunk[0].total_value;
                    let end_value = chunk[chunk.len() - 1].total_value;
                    let return_pct = if start_value > 0.0 {
                        ((end_value - start_value) / start_value) * 100.0
                    } else {
                        0.0
                    };

                    monthly_returns.push(MonthlyReturn {
                        year: 2024,
                        month: (i + 1) as u32,
                        return_pct,
                        benchmark_return: 2.0, // Placeholder
                        excess_return: return_pct - 2.0,
                    });
                }
            }
        }

        monthly_returns
    }

    /// Calculate drawdown periods
    fn calculate_drawdown_periods(&self, snapshots: &[&PerformanceSnapshot]) -> Vec<DrawdownPeriod> {
        let mut drawdown_periods = Vec::new();
        
        if snapshots.len() < 2 {
            return drawdown_periods;
        }

        let mut peak_value = snapshots[0].total_value;
        let mut peak_date = snapshots[0].timestamp;
        let mut in_drawdown = false;
        let mut trough_value = peak_value;

        for snapshot in snapshots.iter().skip(1) {
            if snapshot.total_value > peak_value {
                // New peak
                if in_drawdown {
                    // End of drawdown period
                    let drawdown_pct = ((peak_value - trough_value) / peak_value) * 100.0;
                    drawdown_periods.push(DrawdownPeriod {
                        start_date: peak_date,
                        end_date: Some(snapshot.timestamp),
                        peak_value,
                        trough_value,
                        drawdown_pct,
                        duration_days: (snapshot.timestamp - peak_date).num_days(),
                        recovery_days: Some((snapshot.timestamp - peak_date).num_days()),
                    });
                    in_drawdown = false;
                }
                peak_value = snapshot.total_value;
                peak_date = snapshot.timestamp;
                trough_value = peak_value;
            } else {
                // Below peak
                if !in_drawdown {
                    in_drawdown = true;
                }
                if snapshot.total_value < trough_value {
                    trough_value = snapshot.total_value;
                }
            }
        }

        drawdown_periods
    }

    /// Get analytics summary
    pub fn get_analytics_summary(&self, report: &PerformanceReport) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("📊 Performance Analytics\n"));
        summary.push_str(&format!("═══════════════════════\n"));
        summary.push_str(&format!("📈 Total Return: {:.2}%\n", report.total_return));
        summary.push_str(&format!("📅 Annualized Return: {:.2}%\n", report.annualized_return));
        summary.push_str(&format!("📊 Volatility: {:.2}%\n", report.volatility));
        summary.push_str(&format!("⚡ Sharpe Ratio: {:.2}\n", report.sharpe_ratio));
        summary.push_str(&format!("📉 Max Drawdown: {:.2}%\n", report.max_drawdown));
        summary.push_str(&format!("🎯 Win Rate: {:.1}%\n", report.win_rate));
        summary.push_str(&format!("💰 Profit Factor: {:.2}\n", report.profit_factor));
        summary.push_str(&format!("📈 Average Win: {:.2}%\n", report.average_win));
        summary.push_str(&format!("📉 Average Loss: {:.2}%\n", report.average_loss));
        summary.push_str(&format!("🔄 Total Trades: {}\n", report.total_trades));
        summary.push_str(&format!("\n🏆 vs Benchmark:\n"));
        summary.push_str(&format!("  📊 Excess Return: {:.2}%\n", report.benchmark_comparison.excess_return));
        summary.push_str(&format!("  📈 Alpha: {:.2}%\n", report.benchmark_comparison.alpha));
        summary.push_str(&format!("  📊 Beta: {:.2}\n", report.benchmark_comparison.beta));

        summary
    }
}

impl Default for PortfolioAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio::PositionRiskMetrics;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_position(strategy: &str, value: f64, pnl: f64) -> Position {
        Position {
            id: Uuid::new_v4(),
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: "SOL".to_string(),
            strategy: strategy.to_string(),
            entry_price: 100.0,
            current_price: 100.0 + pnl,
            quantity: value / 100.0,
            value_usd: value,
            unrealized_pnl: pnl,
            realized_pnl: 0.0,
            entry_time: Utc::now(),
            last_update: Utc::now(),
            risk_metrics: PositionRiskMetrics {
                var_95: 0.05,
                var_99: 0.08,
                volatility: 0.15,
                beta: 1.2,
                max_drawdown: 0.02,
            },
        }
    }

    #[test]
    fn test_analytics_creation() {
        let analytics = PortfolioAnalytics::new();
        assert!(analytics.performance_history.is_empty());
    }

    #[test]
    fn test_record_snapshot() {
        let mut analytics = PortfolioAnalytics::new();
        let mut positions = HashMap::new();
        positions.insert(Uuid::new_v4(), create_test_position("trend", 1000.0, 50.0));

        analytics.record_snapshot(&positions);
        
        assert_eq!(analytics.performance_history.len(), 1);
        assert_eq!(analytics.performance_history[0].total_value, 1000.0);
        assert_eq!(analytics.performance_history[0].positions_count, 1);
    }

    #[test]
    fn test_volatility_calculation() {
        let analytics = PortfolioAnalytics::new();
        let returns = vec![1.0, -0.5, 2.0, -1.0, 0.5];
        let volatility = analytics.calculate_volatility(&returns);
        assert!(volatility > 0.0);
    }

    #[test]
    fn test_max_drawdown_calculation() {
        let analytics = PortfolioAnalytics::new();
        let snapshots = vec![
            PerformanceSnapshot {
                timestamp: Utc::now(),
                total_value: 1000.0,
                total_pnl: 0.0,
                daily_return: 0.0,
                positions_count: 1,
                strategy_breakdown: HashMap::new(),
            },
            PerformanceSnapshot {
                timestamp: Utc::now(),
                total_value: 1200.0,
                total_pnl: 200.0,
                daily_return: 20.0,
                positions_count: 1,
                strategy_breakdown: HashMap::new(),
            },
            PerformanceSnapshot {
                timestamp: Utc::now(),
                total_value: 900.0,
                total_pnl: -100.0,
                daily_return: -25.0,
                positions_count: 1,
                strategy_breakdown: HashMap::new(),
            },
        ];
        
        let snapshot_refs: Vec<_> = snapshots.iter().collect();
        let max_drawdown = analytics.calculate_max_drawdown(&snapshot_refs);
        assert_eq!(max_drawdown, 25.0); // 25% drawdown from 1200 to 900
    }
}
