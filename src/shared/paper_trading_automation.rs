// Paper Trading Automation Engine
// Phase 3 Implementation - Automated Paper Trading with Real Market Data

use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use uuid::Uuid;

use crate::shared::pool_detector::{DetectedPool, TradingOpportunity, OpportunityType};

/// Configuration for paper trading automation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTradingConfig {
    /// Starting virtual balance in USD
    pub initial_balance_usd: f64,
    /// Maximum position size as percentage of portfolio
    pub max_position_size_pct: f64,
    /// Minimum confidence score to trigger a trade
    pub min_confidence_threshold: f64,
    /// Maximum price impact allowed for execution
    pub max_price_impact_pct: f64,
    /// Stop loss percentage
    pub stop_loss_pct: f64,
    /// Take profit percentage
    pub take_profit_pct: f64,
    /// Maximum number of concurrent positions
    pub max_concurrent_positions: usize,
    /// Minimum liquidity required for trading
    pub min_liquidity_usd: f64,
}

impl Default for PaperTradingConfig {
    fn default() -> Self {
        Self {
            initial_balance_usd: 10000.0, // $10K starting capital
            max_position_size_pct: 5.0,   // Max 5% per trade
            min_confidence_threshold: 0.7, // 70% confidence minimum
            max_price_impact_pct: 3.0,     // Max 3% price impact
            stop_loss_pct: 15.0,           // 15% stop loss
            take_profit_pct: 25.0,         // 25% take profit
            max_concurrent_positions: 5,   // Max 5 positions
            min_liquidity_usd: 25000.0,    // Min $25K liquidity
        }
    }
}

/// Represents a paper trading position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperPosition {
    pub id: String,
    pub pool_address: String,
    pub opportunity_type: OpportunityType,
    pub entry_time: chrono::DateTime<chrono::Utc>,
    pub entry_price_usd: f64,
    pub position_size_usd: f64,
    pub expected_profit_usd: f64,
    pub confidence_score: f64,
    pub stop_loss_price: f64,
    pub take_profit_price: f64,
    pub status: PositionStatus,
    pub current_pnl_usd: f64,
    pub current_price_usd: f64,
    pub execution_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PositionStatus {
    Open,
    ClosedProfit,
    ClosedLoss,
    ClosedManual,
    ClosedExpired,
}

/// Paper trading execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTradeResult {
    pub position_id: String,
    pub success: bool,
    pub action: TradeAction,
    pub execution_time: chrono::DateTime<chrono::Utc>,
    pub price_usd: f64,
    pub size_usd: f64,
    pub pnl_usd: f64,
    pub reason: String,
    pub portfolio_value_after: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeAction {
    Open,
    Close,
    StopLoss,
    TakeProfit,
}

/// Portfolio statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioStats {
    pub total_value_usd: f64,
    pub available_cash_usd: f64,
    pub invested_amount_usd: f64,
    pub unrealized_pnl_usd: f64,
    pub realized_pnl_usd: f64,
    pub total_pnl_usd: f64,
    pub total_return_pct: f64,
    pub win_rate_pct: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub largest_win_usd: f64,
    pub largest_loss_usd: f64,
    pub average_trade_pnl: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown_pct: f64,
    pub active_positions: usize,
}

/// Paper Trading Automation Engine
pub struct PaperTradingEngine {
    config: PaperTradingConfig,
    positions: HashMap<String, PaperPosition>,
    trade_history: Vec<PaperTradeResult>,
    portfolio_value_history: Vec<(chrono::DateTime<chrono::Utc>, f64)>,
    session_start: chrono::DateTime<chrono::Utc>,
    session_id: String,
    total_realized_pnl: f64,
    peak_portfolio_value: f64,
}

impl PaperTradingEngine {
    /// Create a new paper trading engine
    pub fn new(config: PaperTradingConfig) -> Self {
        let session_start = chrono::Utc::now();
        let session_id = Uuid::new_v4().to_string();
        
        Self {
            config: config.clone(),
            positions: HashMap::new(),
            trade_history: Vec::new(),
            portfolio_value_history: vec![(session_start, config.initial_balance_usd)],
            session_start,
            session_id,
            total_realized_pnl: 0.0,
            peak_portfolio_value: config.initial_balance_usd,
        }
    }

    /// Evaluate if an opportunity should trigger a trade
    pub fn should_trade(&self, opportunity: &TradingOpportunity) -> bool {
        // Check confidence threshold
        if opportunity.confidence < self.config.min_confidence_threshold {
            return false;
        }

        // Check price impact
        if opportunity.pool.price_impact_1k > self.config.max_price_impact_pct {
            return false;
        }

        // Check liquidity requirement
        if opportunity.pool.liquidity_usd < self.config.min_liquidity_usd {
            return false;
        }

        // Check if we already have a position in this pool
        if self.positions.values().any(|p| p.pool_address == opportunity.pool.pool_address && p.status == PositionStatus::Open) {
            return false;
        }

        // Check maximum concurrent positions
        let active_positions = self.positions.values().filter(|p| p.status == PositionStatus::Open).count();
        if active_positions >= self.config.max_concurrent_positions {
            return false;
        }

        // Check available capital for position sizing
        let stats = self.get_portfolio_stats();
        let position_size = self.calculate_position_size(&stats, opportunity);
        if position_size < 100.0 { // Minimum $100 position
            return false;
        }

        true
    }

    /// Execute a paper trade based on an opportunity
    pub fn execute_trade(&mut self, opportunity: &TradingOpportunity) -> Result<PaperTradeResult> {
        if !self.should_trade(opportunity) {
            return Err(anyhow::anyhow!("Trade criteria not met"));
        }

        let position_id = Uuid::new_v4().to_string();
        let stats = self.get_portfolio_stats();
        let position_size = self.calculate_position_size(&stats, opportunity);
        
        // Calculate entry price (simulate slippage)
        let entry_price = opportunity.pool.token_a.price_usd * (1.0 + opportunity.pool.price_impact_1k / 100.0 * 0.5);
        
        // Calculate stop loss and take profit levels
        let stop_loss_price = entry_price * (1.0 - self.config.stop_loss_pct / 100.0);
        let take_profit_price = entry_price * (1.0 + self.config.take_profit_pct / 100.0);

        // Create position
        let position = PaperPosition {
            id: position_id.clone(),
            pool_address: opportunity.pool.pool_address.clone(),
            opportunity_type: opportunity.opportunity_type.clone(),
            entry_time: chrono::Utc::now(),
            entry_price_usd: entry_price,
            position_size_usd: position_size,
            expected_profit_usd: opportunity.expected_profit_usd,
            confidence_score: opportunity.confidence,
            stop_loss_price,
            take_profit_price,
            status: PositionStatus::Open,
            current_pnl_usd: 0.0,
            current_price_usd: entry_price,
            execution_notes: vec![format!("Auto-executed based on {} opportunity", 
                match opportunity.opportunity_type {
                    OpportunityType::NewPoolSnipe => "new pool snipe",
                    OpportunityType::PriceDiscrepancy => "price arbitrage",
                    OpportunityType::LiquidityImbalance => "low slippage",
                    OpportunityType::VolumeSpike => "volume spike",
                })
            ],
        };

        // Add position to portfolio
        self.positions.insert(position_id.clone(), position);

        // Create trade result
        let result = PaperTradeResult {
            position_id: position_id.clone(),
            success: true,
            action: TradeAction::Open,
            execution_time: chrono::Utc::now(),
            price_usd: entry_price,
            size_usd: position_size,
            pnl_usd: 0.0,
            reason: format!("Opened {} position - Confidence: {:.1}%", 
                match opportunity.opportunity_type {
                    OpportunityType::NewPoolSnipe => "New Pool",
                    OpportunityType::PriceDiscrepancy => "Arbitrage",
                    OpportunityType::LiquidityImbalance => "Low Slippage",
                    OpportunityType::VolumeSpike => "Volume Spike",
                },
                opportunity.confidence * 100.0
            ),
            portfolio_value_after: self.get_portfolio_stats().total_value_usd,
        };

        // Add to trade history
        self.trade_history.push(result.clone());

        // Update portfolio value history
        let current_value = self.get_portfolio_stats().total_value_usd;
        self.portfolio_value_history.push((chrono::Utc::now(), current_value));

        if current_value > self.peak_portfolio_value {
            self.peak_portfolio_value = current_value;
        }

        Ok(result)
    }

    /// Update positions with current market data
    pub fn update_positions(&mut self, pools: &[DetectedPool]) -> Vec<PaperTradeResult> {
        let mut results = Vec::new();
        let mut positions_to_update = Vec::new();

        // Collect positions that need updates
        for (id, position) in &self.positions {
            if position.status == PositionStatus::Open {
                positions_to_update.push((id.clone(), position.clone()));
            }
        }

        for (position_id, mut position) in positions_to_update {
            // Find current pool data
            if let Some(current_pool) = pools.iter().find(|p| p.pool_address == position.pool_address) {
                // Update current price (simulate market movement)
                let price_change = (rand::thread_rng().gen::<f64>() - 0.5) * 0.1; // ±5% random movement
                position.current_price_usd = current_pool.token_a.price_usd * (1.0 + price_change);
                
                // Calculate current P&L
                let price_change_pct = (position.current_price_usd - position.entry_price_usd) / position.entry_price_usd;
                position.current_pnl_usd = position.position_size_usd * price_change_pct;

                // Check for stop loss
                if position.current_price_usd <= position.stop_loss_price {
                    position.status = PositionStatus::ClosedLoss;
                    self.total_realized_pnl += position.current_pnl_usd;
                    
                    let result = PaperTradeResult {
                        position_id: position_id.clone(),
                        success: false,
                        action: TradeAction::StopLoss,
                        execution_time: chrono::Utc::now(),
                        price_usd: position.current_price_usd,
                        size_usd: position.position_size_usd,
                        pnl_usd: position.current_pnl_usd,
                        reason: format!("Stop loss triggered at ${:.4}", position.current_price_usd),
                        portfolio_value_after: self.get_portfolio_stats().total_value_usd,
                    };
                    results.push(result);
                }
                // Check for take profit
                else if position.current_price_usd >= position.take_profit_price {
                    position.status = PositionStatus::ClosedProfit;
                    self.total_realized_pnl += position.current_pnl_usd;
                    
                    let result = PaperTradeResult {
                        position_id: position_id.clone(),
                        success: true,
                        action: TradeAction::TakeProfit,
                        execution_time: chrono::Utc::now(),
                        price_usd: position.current_price_usd,
                        size_usd: position.position_size_usd,
                        pnl_usd: position.current_pnl_usd,
                        reason: format!("Take profit triggered at ${:.4}", position.current_price_usd),
                        portfolio_value_after: self.get_portfolio_stats().total_value_usd,
                    };
                    results.push(result);
                }

                // Update position in HashMap
                self.positions.insert(position_id, position);
            }
        }

        // Add results to history
        self.trade_history.extend(results.clone());

        // Update portfolio value history
        let current_value = self.get_portfolio_stats().total_value_usd;
        self.portfolio_value_history.push((chrono::Utc::now(), current_value));

        if current_value > self.peak_portfolio_value {
            self.peak_portfolio_value = current_value;
        }

        results
    }

    /// Calculate appropriate position size for an opportunity
    fn calculate_position_size(&self, stats: &PortfolioStats, opportunity: &TradingOpportunity) -> f64 {
        // Base position size as percentage of portfolio
        let base_size = stats.available_cash_usd * (self.config.max_position_size_pct / 100.0);
        
        // Adjust based on confidence (higher confidence = larger position)
        let confidence_multiplier = opportunity.confidence;
        
        // Adjust based on expected profit ratio
        let profit_ratio = opportunity.expected_profit_usd / opportunity.recommended_size_usd.max(1.0);
        let profit_multiplier = (1.0 + profit_ratio / 10.0).min(2.0); // Cap at 2x
        
        let adjusted_size = base_size * confidence_multiplier * profit_multiplier;
        
        // Ensure we don't exceed available cash
        adjusted_size.min(stats.available_cash_usd * 0.8) // Leave 20% cash buffer
    }

    /// Get current portfolio statistics
    pub fn get_portfolio_stats(&self) -> PortfolioStats {
        let open_positions: Vec<&PaperPosition> = self.positions.values()
            .filter(|p| p.status == PositionStatus::Open)
            .collect();
        
        let invested_amount = open_positions.iter()
            .map(|p| p.position_size_usd)
            .sum::<f64>();
            
        let unrealized_pnl = open_positions.iter()
            .map(|p| p.current_pnl_usd)
            .sum::<f64>();
            
        let total_value = self.config.initial_balance_usd + self.total_realized_pnl + unrealized_pnl;
        let available_cash = total_value - invested_amount;
        
        // Calculate trade statistics
        let closed_trades: Vec<&PaperPosition> = self.positions.values()
            .filter(|p| p.status != PositionStatus::Open)
            .collect();
            
        let winning_trades = closed_trades.iter()
            .filter(|p| p.current_pnl_usd > 0.0)
            .count() as u32;
            
        let losing_trades = closed_trades.iter()
            .filter(|p| p.current_pnl_usd < 0.0)
            .count() as u32;
            
        let total_trades = closed_trades.len() as u32;
        
        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };
        
        let largest_win = closed_trades.iter()
            .map(|p| p.current_pnl_usd)
            .fold(0.0, f64::max);
            
        let largest_loss = closed_trades.iter()
            .map(|p| p.current_pnl_usd)
            .fold(0.0, f64::min);
            
        let average_trade_pnl = if total_trades > 0 {
            self.total_realized_pnl / total_trades as f64
        } else {
            0.0
        };
        
        // Calculate returns and drawdown
        let total_pnl = self.total_realized_pnl + unrealized_pnl;
        let total_return = (total_pnl / self.config.initial_balance_usd) * 100.0;
        
        let max_drawdown = if self.peak_portfolio_value > self.config.initial_balance_usd {
            ((self.peak_portfolio_value - total_value) / self.peak_portfolio_value) * 100.0
        } else {
            0.0
        };
        
        // Simple Sharpe ratio calculation (simplified)
        let sharpe_ratio = if total_trades > 3 {
            let returns: Vec<f64> = self.trade_history.iter()
                .map(|t| t.pnl_usd / self.config.initial_balance_usd)
                .collect();
            let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance = returns.iter()
                .map(|r| (r - avg_return).powi(2))
                .sum::<f64>() / returns.len() as f64;
            let std_dev = variance.sqrt();
            if std_dev > 0.0 { avg_return / std_dev } else { 0.0 }
        } else {
            0.0
        };

        PortfolioStats {
            total_value_usd: total_value,
            available_cash_usd: available_cash,
            invested_amount_usd: invested_amount,
            unrealized_pnl_usd: unrealized_pnl,
            realized_pnl_usd: self.total_realized_pnl,
            total_pnl_usd: total_pnl,
            total_return_pct: total_return,
            win_rate_pct: win_rate,
            total_trades,
            winning_trades,
            losing_trades,
            largest_win_usd: largest_win,
            largest_loss_usd: largest_loss,
            average_trade_pnl,
            sharpe_ratio,
            max_drawdown_pct: max_drawdown,
            active_positions: open_positions.len(),
        }
    }

    /// Generate a comprehensive performance report
    pub fn generate_report(&self) -> String {
        let stats = self.get_portfolio_stats();
        let session_duration = chrono::Utc::now().signed_duration_since(self.session_start);
        
        format!(
            r#"
📊 PAPER TRADING PERFORMANCE REPORT
=====================================
Session ID: {}
Duration: {:.1} minutes
Start Time: {}
Initial Capital: ${:.2}

💰 PORTFOLIO PERFORMANCE
=======================
Current Value: ${:.2}
Total P&L: ${:.2} ({:+.2}%)
Realized P&L: ${:.2}
Unrealized P&L: ${:.2}
Available Cash: ${:.2}
Invested Amount: ${:.2}

📈 TRADING STATISTICS
====================
Total Trades: {}
Winning Trades: {} ({:.1}%)
Losing Trades: {} ({:.1}%)
Win Rate: {:.1}%
Average Trade P&L: ${:.2}
Largest Win: ${:.2}
Largest Loss: ${:.2}

⚡ RISK METRICS
==============
Maximum Drawdown: {:.2}%
Sharpe Ratio: {:.3}
Active Positions: {}
Peak Portfolio Value: ${:.2}

🎯 POSITION BREAKDOWN
====================
{}

💡 RECENT TRADES
================
{}

📊 RECOMMENDATIONS
=================
{}
"#,
            self.session_id,
            session_duration.num_seconds() as f64 / 60.0,
            self.session_start.format("%Y-%m-%d %H:%M:%S UTC"),
            self.config.initial_balance_usd,
            stats.total_value_usd,
            stats.total_pnl_usd,
            stats.total_return_pct,
            stats.realized_pnl_usd,
            stats.unrealized_pnl_usd,
            stats.available_cash_usd,
            stats.invested_amount_usd,
            stats.total_trades,
            stats.winning_trades,
            stats.win_rate_pct,
            stats.losing_trades,
            100.0 - stats.win_rate_pct,
            stats.win_rate_pct,
            stats.average_trade_pnl,
            stats.largest_win_usd,
            stats.largest_loss_usd,
            stats.max_drawdown_pct,
            stats.sharpe_ratio,
            stats.active_positions,
            self.peak_portfolio_value,
            self.format_positions(),
            self.format_recent_trades(),
            self.generate_recommendations(&stats)
        )
    }

    /// Format active positions for display
    fn format_positions(&self) -> String {
        let open_positions: Vec<&PaperPosition> = self.positions.values()
            .filter(|p| p.status == PositionStatus::Open)
            .collect();
            
        if open_positions.is_empty() {
            return "No active positions".to_string();
        }
        
        let mut result = String::new();
        for (i, position) in open_positions.iter().enumerate() {
            let pnl_pct = (position.current_pnl_usd / position.position_size_usd) * 100.0;
            result.push_str(&format!(
                "{}. {} - ${:.0} ({:+.1}%) - Confidence: {:.0}%\n",
                i + 1,
                match position.opportunity_type {
                    OpportunityType::NewPoolSnipe => "New Pool",
                    OpportunityType::PriceDiscrepancy => "Arbitrage",
                    OpportunityType::LiquidityImbalance => "Low Slippage",
                    OpportunityType::VolumeSpike => "Volume Spike",
                },
                position.position_size_usd,
                pnl_pct,
                position.confidence_score * 100.0
            ));
        }
        result
    }

    /// Format recent trades for display
    fn format_recent_trades(&self) -> String {
        let recent_trades: Vec<&PaperTradeResult> = self.trade_history.iter()
            .rev()
            .take(5)
            .collect();
            
        if recent_trades.is_empty() {
            return "No trades executed yet".to_string();
        }
        
        let mut result = String::new();
        for trade in recent_trades {
            let action_str = match trade.action {
                TradeAction::Open => "OPEN",
                TradeAction::Close => "CLOSE",
                TradeAction::StopLoss => "STOP",
                TradeAction::TakeProfit => "PROFIT",
            };
            
            result.push_str(&format!(
                "{} {} - ${:.0} ({:+.1}%) - {}\n",
                trade.execution_time.format("%H:%M:%S"),
                action_str,
                trade.size_usd,
                (trade.pnl_usd / trade.size_usd) * 100.0,
                trade.reason
            ));
        }
        result
    }

    /// Generate performance-based recommendations
    fn generate_recommendations(&self, stats: &PortfolioStats) -> String {
        let mut recommendations = Vec::new();
        
        if stats.win_rate_pct < 50.0 && stats.total_trades > 5 {
            recommendations.push("• Consider tightening entry criteria - win rate below 50%");
        }
        
        if stats.max_drawdown_pct > 20.0 {
            recommendations.push("• Reduce position sizes - high drawdown detected");
        }
        
        if stats.active_positions == 0 && stats.available_cash_usd > stats.total_value_usd * 0.8 {
            recommendations.push("• Consider more aggressive opportunity targeting");
        }
        
        if stats.average_trade_pnl < 0.0 {
            recommendations.push("• Review exit strategy - average trade P&L negative");
        }
        
        if stats.total_trades < 3 {
            recommendations.push("• Continue monitoring for more trading opportunities");
        }
        
        if recommendations.is_empty() {
            recommendations.push("• Performance looks solid - continue current strategy");
        }
        
        recommendations.join("\n")
    }

    /// Get session statistics for external use
    pub fn get_session_stats(&self) -> (String, chrono::DateTime<chrono::Utc>, u32, u32) {
        let stats = self.get_portfolio_stats();
        (
            self.session_id.clone(),
            self.session_start,
            stats.total_trades,
            stats.active_positions as u32
        )
    }

    /// Export portfolio data to JSON
    pub fn export_to_json(&self) -> Result<String> {
        #[derive(Serialize)]
        struct ExportData {
            session_id: String,
            session_start: chrono::DateTime<chrono::Utc>,
            config: PaperTradingConfig,
            stats: PortfolioStats,
            positions: Vec<PaperPosition>,
            trade_history: Vec<PaperTradeResult>,
            portfolio_value_history: Vec<(chrono::DateTime<chrono::Utc>, f64)>,
        }
        
        let export_data = ExportData {
            session_id: self.session_id.clone(),
            session_start: self.session_start,
            config: self.config.clone(),
            stats: self.get_portfolio_stats(),
            positions: self.positions.values().cloned().collect(),
            trade_history: self.trade_history.clone(),
            portfolio_value_history: self.portfolio_value_history.clone(),
        };
        
        Ok(serde_json::to_string_pretty(&export_data)?)
    }
}
