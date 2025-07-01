use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::portfolio::blockchain_analyzer::{TransactionHistory, TransactionType};
use crate::portfolio::price_feed::{PriceFeed, TokenPrice};
use crate::portfolio::wallet_scanner::WalletBalance;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub strategy_name: String,
    pub wallet_address: String,
    pub time_period: String,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: f64,
    pub total_pnl_usd: f64,
    pub total_pnl_percentage: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub profit_factor: f64,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: f64,
    pub max_drawdown_percentage: f64,
    pub total_fees_paid: f64,
    pub roi: f64,
    pub trades_per_day: f64,
    pub best_trade: Option<TradeResult>,
    pub worst_trade: Option<TradeResult>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub trade_id: String,
    pub strategy: String,
    pub entry_time: chrono::DateTime<chrono::Utc>,
    pub exit_time: Option<chrono::DateTime<chrono::Utc>>,
    pub symbol: String,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub quantity: f64,
    pub pnl_usd: f64,
    pub pnl_percentage: f64,
    pub fees: f64,
    pub status: TradeStatus,
    pub transaction_signatures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeStatus {
    Open,
    Closed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAllocation {
    pub strategy_name: String,
    pub allocation_percentage: f64,
    pub current_value_usd: f64,
    pub target_value_usd: f64,
    pub rebalance_needed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallPortfolioMetrics {
    pub total_value_usd: f64,
    pub total_pnl_usd: f64,
    pub total_pnl_percentage: f64,
    pub strategy_allocations: Vec<StrategyAllocation>,
    pub risk_metrics: RiskMetrics,
    pub performance_attribution: Vec<StrategyContribution>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub portfolio_volatility: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub var_95: f64, // Value at Risk 95%
    pub beta: Option<f64>,
    pub correlation_matrix: HashMap<String, HashMap<String, f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyContribution {
    pub strategy_name: String,
    pub contribution_to_return: f64,
    pub contribution_percentage: f64,
    pub alpha: f64,
    pub beta: f64,
}

#[derive(Debug, Clone)]
pub struct StrategyTracker {
    network: String,
}

impl StrategyTracker {
    pub fn new(network: &str) -> Self {
        Self {
            network: network.to_string(),
        }
    }

    pub async fn calculate_strategy_performance(
        &self,
        strategy_name: &str,
        wallet_address: &str,
        transaction_history: &TransactionHistory,
        price_feed: &PriceFeed,
    ) -> Result<StrategyPerformance> {
        println!("📊 Calculating performance for strategy: {}", strategy_name);

        // Filter transactions relevant to this strategy
        let strategy_transactions = self.filter_strategy_transactions(
            &transaction_history.transactions,
            strategy_name,
        );

        let total_trades = strategy_transactions.len();
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut total_pnl = 0.0;
        let mut total_fees = 0.0;
        let mut wins_sum = 0.0;
        let mut losses_sum = 0.0;

        // Analyze individual trades
        let mut trades = Vec::new();
        for (_i, tx) in strategy_transactions.iter().enumerate() {
            let trade_result = self.analyze_trade(tx, price_feed).await?;

            total_fees += tx.fee;

            if trade_result.pnl_usd > 0.0 {
                winning_trades += 1;
                wins_sum += trade_result.pnl_usd;
            } else if trade_result.pnl_usd < 0.0 {
                losing_trades += 1;
                losses_sum += trade_result.pnl_usd.abs();
            }

            total_pnl += trade_result.pnl_usd;
            trades.push(trade_result);
        }

        // Calculate metrics
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };

        let average_win = if winning_trades > 0 {
            wins_sum / winning_trades as f64
        } else {
            0.0
        };

        let average_loss = if losing_trades > 0 {
            losses_sum / losing_trades as f64
        } else {
            0.0
        };

        let profit_factor = if average_loss > 0.0 {
            (wins_sum) / (losses_sum)
        } else {
            if wins_sum > 0.0 { f64::INFINITY } else { 0.0 }
        };

        // Calculate drawdown
        let (max_drawdown, max_drawdown_percentage) = self.calculate_drawdown(&trades);

        // Find best and worst trades
        let best_trade = trades.iter()
            .max_by(|a, b| a.pnl_usd.partial_cmp(&b.pnl_usd).unwrap_or(std::cmp::Ordering::Equal))
            .cloned();

        let worst_trade = trades.iter()
            .min_by(|a, b| a.pnl_usd.partial_cmp(&b.pnl_usd).unwrap_or(std::cmp::Ordering::Equal))
            .cloned();

        // Calculate ROI (simplified)
        let roi = total_pnl; // Would need initial capital for proper ROI calculation

        // Calculate trades per day
        let days_span = if let (Some(first), Some(last)) = (
            strategy_transactions.first().and_then(|tx| tx.block_time),
            strategy_transactions.last().and_then(|tx| tx.block_time)
        ) {
            let duration = (last - first) as f64 / 86400.0; // Convert seconds to days
            if duration > 0.0 { duration } else { 1.0 }
        } else {
            1.0
        };

        let trades_per_day = total_trades as f64 / days_span;

        Ok(StrategyPerformance {
            strategy_name: strategy_name.to_string(),
            wallet_address: wallet_address.to_string(),
            time_period: "30 days".to_string(), // Configurable
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            total_pnl_usd: total_pnl,
            total_pnl_percentage: 0.0, // Would need initial capital
            average_win,
            average_loss,
            profit_factor,
            sharpe_ratio: None, // Would need risk-free rate and volatility calculation
            max_drawdown,
            max_drawdown_percentage,
            total_fees_paid: total_fees,
            roi,
            trades_per_day,
            best_trade,
            worst_trade,
            last_updated: chrono::Utc::now(),
        })
    }

    fn filter_strategy_transactions<'a>(
        &self,
        transactions: &'a [crate::portfolio::blockchain_analyzer::TransactionRecord],
        strategy_name: &str,
    ) -> Vec<&'a crate::portfolio::blockchain_analyzer::TransactionRecord> {
        // This is a simplified filter
        // In a real implementation, we would have strategy-specific transaction tagging
        transactions.iter()
            .filter(|tx| {
                match strategy_name {
                    "jupiter_arbitrage" => tx.programs.iter().any(|p| p.contains("Jupiter")),
                    "raydium_lp" => tx.programs.iter().any(|p| p.contains("Raydium")),
                    "dex_trading" => matches!(tx.transaction_type, TransactionType::TokenSwap),
                    _ => true, // Default: include all transactions
                }
            })
            .collect()
    }

    async fn analyze_trade(
        &self,
        tx: &crate::portfolio::blockchain_analyzer::TransactionRecord,
        price_feed: &PriceFeed,
    ) -> Result<TradeResult> {
        // Simplified trade analysis
        // In a real implementation, we would parse transaction details more thoroughly

        let symbol = if !tx.token_changes.is_empty() {
            tx.token_changes[0].symbol.clone()
        } else {
            "SOL".to_string()
        };

        let entry_time = if let Some(block_time) = tx.block_time {
            chrono::DateTime::from_timestamp(block_time, 0)
                .unwrap_or_else(|| chrono::Utc::now())
        } else {
            chrono::Utc::now()
        };

        // Get current price for P&L calculation (simplified)
        let current_price = if symbol != "SOL" {
            // Try to find the token mint for price lookup
            if let Some(token_change) = tx.token_changes.first() {
                match price_feed.get_token_price(&token_change.mint).await {
                    Ok(price_info) => price_info.price_usd,
                    Err(_) => 0.0,
                }
            } else {
                0.0
            }
        } else {
            match price_feed.get_sol_price().await {
                Ok(price_info) => price_info.price_usd,
                Err(_) => 0.0,
            }
        };

        let pnl_usd = tx.sol_change * current_price; // Simplified calculation

        Ok(TradeResult {
            trade_id: tx.signature.clone(),
            strategy: "auto_detected".to_string(),
            entry_time,
            exit_time: Some(entry_time), // Simplified: assume immediate execution
            symbol,
            entry_price: current_price,
            exit_price: Some(current_price),
            quantity: tx.sol_change.abs(),
            pnl_usd,
            pnl_percentage: 0.0, // Would need entry value
            fees: tx.fee,
            status: match tx.status {
                crate::portfolio::blockchain_analyzer::TransactionStatus::Success => TradeStatus::Closed,
                crate::portfolio::blockchain_analyzer::TransactionStatus::Failed => TradeStatus::Failed,
                _ => TradeStatus::Open,
            },
            transaction_signatures: vec![tx.signature.clone()],
        })
    }

    fn calculate_drawdown(&self, trades: &[TradeResult]) -> (f64, f64) {
        let mut peak = 0.0;
        let mut max_drawdown = 0.0;
        let mut running_pnl = 0.0;

        for trade in trades {
            running_pnl += trade.pnl_usd;
            if running_pnl > peak {
                peak = running_pnl;
            }
            let drawdown = peak - running_pnl;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        let max_drawdown_percentage = if peak > 0.0 {
            (max_drawdown / peak) * 100.0
        } else {
            0.0
        };

        (max_drawdown, max_drawdown_percentage)
    }

    pub async fn calculate_overall_metrics(
        &self,
        wallet_balances: &[WalletBalance],
        strategy_performances: &[StrategyPerformance],
        price_feed: &PriceFeed,
    ) -> Result<OverallPortfolioMetrics> {
        println!("📈 Calculating overall portfolio metrics");

        let mut total_value_usd = 0.0;

        // Calculate total portfolio value
        for balance in wallet_balances {
            // Add SOL value
            if let Ok(sol_price) = price_feed.get_sol_price().await {
                total_value_usd += balance.sol_balance * sol_price.price_usd;
            }

            // Add token values
            for token in &balance.token_balances {
                if let Some(value) = token.value_usd {
                    total_value_usd += value;
                }
            }
        }

        // Calculate total P&L
        let total_pnl_usd: f64 = strategy_performances.iter()
            .map(|sp| sp.total_pnl_usd)
            .sum();

        let total_pnl_percentage = if total_value_usd > 0.0 {
            (total_pnl_usd / total_value_usd) * 100.0
        } else {
            0.0
        };

        // Create strategy allocations
        let strategy_allocations: Vec<StrategyAllocation> = strategy_performances.iter()
            .map(|sp| {
                let allocation_percentage = if total_value_usd > 0.0 {
                    (sp.total_pnl_usd.abs() / total_value_usd) * 100.0
                } else {
                    0.0
                };

                StrategyAllocation {
                    strategy_name: sp.strategy_name.clone(),
                    allocation_percentage,
                    current_value_usd: sp.total_pnl_usd.abs(),
                    target_value_usd: sp.total_pnl_usd.abs(),
                    rebalance_needed: false,
                }
            })
            .collect();

        // Calculate risk metrics (simplified)
        let portfolio_volatility = self.calculate_portfolio_volatility(strategy_performances);
        let sharpe_ratio = self.calculate_sharpe_ratio(&strategy_performances, portfolio_volatility);
        let max_drawdown = strategy_performances.iter()
            .map(|sp| sp.max_drawdown)
            .fold(0.0, f64::max);

        let risk_metrics = RiskMetrics {
            portfolio_volatility,
            sharpe_ratio,
            max_drawdown,
            var_95: 0.0, // Would need historical returns data
            beta: None,
            correlation_matrix: HashMap::new(),
        };

        // Calculate performance attribution
        let performance_attribution: Vec<StrategyContribution> = strategy_performances.iter()
            .map(|sp| {
                let contribution_to_return = if total_pnl_usd != 0.0 {
                    sp.total_pnl_usd / total_pnl_usd
                } else {
                    0.0
                };

                StrategyContribution {
                    strategy_name: sp.strategy_name.clone(),
                    contribution_to_return: sp.total_pnl_usd,
                    contribution_percentage: contribution_to_return * 100.0,
                    alpha: 0.0, // Would need benchmark comparison
                    beta: 1.0,  // Simplified
                }
            })
            .collect();

        Ok(OverallPortfolioMetrics {
            total_value_usd,
            total_pnl_usd,
            total_pnl_percentage,
            strategy_allocations,
            risk_metrics,
            performance_attribution,
            last_updated: chrono::Utc::now(),
        })
    }

    fn calculate_portfolio_volatility(&self, strategy_performances: &[StrategyPerformance]) -> f64 {
        // Simplified volatility calculation
        if strategy_performances.is_empty() {
            return 0.0;
        }

        let returns: Vec<f64> = strategy_performances.iter()
            .map(|sp| sp.total_pnl_percentage)
            .collect();

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;

        variance.sqrt()
    }

    fn calculate_sharpe_ratio(&self, strategy_performances: &[StrategyPerformance], volatility: f64) -> f64 {
        if volatility == 0.0 {
            return 0.0;
        }

        let average_return = strategy_performances.iter()
            .map(|sp| sp.total_pnl_percentage)
            .sum::<f64>() / strategy_performances.len() as f64;

        let risk_free_rate = 2.0; // Assume 2% risk-free rate
        (average_return - risk_free_rate) / volatility
    }

    pub fn get_network(&self) -> &str {
        &self.network
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_tracker_creation() {
        let tracker = StrategyTracker::new("devnet");
        assert_eq!(tracker.get_network(), "devnet");
    }

    #[test]
    fn test_drawdown_calculation() {
        let tracker = StrategyTracker::new("devnet");
        let trades = vec![
            TradeResult {
                trade_id: "1".to_string(),
                strategy: "test".to_string(),
                entry_time: chrono::Utc::now(),
                exit_time: None,
                symbol: "SOL".to_string(),
                entry_price: 100.0,
                exit_price: None,
                quantity: 1.0,
                pnl_usd: 10.0,
                pnl_percentage: 10.0,
                fees: 0.1,
                status: TradeStatus::Closed,
                transaction_signatures: vec![],
            },
            TradeResult {
                trade_id: "2".to_string(),
                strategy: "test".to_string(),
                entry_time: chrono::Utc::now(),
                exit_time: None,
                symbol: "SOL".to_string(),
                entry_price: 100.0,
                exit_price: None,
                quantity: 1.0,
                pnl_usd: -5.0,
                pnl_percentage: -5.0,
                fees: 0.1,
                status: TradeStatus::Closed,
                transaction_signatures: vec![],
            },
        ];

        let (max_drawdown, max_drawdown_percentage) = tracker.calculate_drawdown(&trades);
        assert!(max_drawdown >= 0.0);
        assert!(max_drawdown_percentage >= 0.0);
    }
}
