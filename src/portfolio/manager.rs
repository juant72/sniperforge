//! Portfolio Manager - Core portfolio management functionality
//! 
//! Manages multiple positions across strategies with risk controls

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tokio::sync::RwLock;
use std::sync::Arc;

use super::{
    Position, PortfolioConfiguration, PortfolioMetrics, 
    GlobalRiskLimits, RebalanceFrequency
};

/// Core portfolio manager handling all positions and strategies
#[derive(Debug)]
pub struct PortfolioManager {
    config: PortfolioConfiguration,
    positions: Arc<RwLock<HashMap<Uuid, Position>>>,
    strategy_allocations: HashMap<String, f64>,
    last_rebalance: DateTime<Utc>,
}

/// Portfolio position data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub id: Uuid,
    pub token_mint: String,
    pub symbol: String,
    pub strategy: String,
    pub entry_price: f64,
    pub current_price: f64,
    pub quantity: f64,
    pub value_usd: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub entry_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
}

/// Portfolio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioConfig {
    pub total_capital: f64,
    pub max_positions: u32,
    pub max_correlation: f64,
    pub rebalance_frequency: RebalanceFrequency,
    pub risk_limits: GlobalRiskLimits,
    pub strategy_allocations: HashMap<String, f64>,
}

impl PortfolioManager {
    /// Create new portfolio manager
    pub fn new(config: PortfolioConfiguration) -> Self {
        Self {
            strategy_allocations: config.strategy_allocations.clone(),
            config,
            positions: Arc::new(RwLock::new(HashMap::new())),
            last_rebalance: Utc::now(),
        }
    }

    /// Add a new position to the portfolio
    pub async fn add_position(&self, position: Position) -> Result<()> {
        let mut positions = self.positions.write().await;
        
        // Check position limits
        if positions.len() >= self.config.max_positions as usize {
            return Err(anyhow::anyhow!("Maximum positions limit reached"));
        }

        // Check concentration limits
        let total_value = self.calculate_total_value(&positions).await;
        let position_concentration = position.value_usd / total_value;
        
        if position_concentration > self.config.risk_limits.max_position_concentration {
            return Err(anyhow::anyhow!("Position concentration limit exceeded"));
        }

        positions.insert(position.id, position);
        Ok(())
    }

    /// Remove a position from the portfolio
    pub async fn remove_position(&self, position_id: Uuid) -> Result<Position> {
        let mut positions = self.positions.write().await;
        positions.remove(&position_id)
            .ok_or_else(|| anyhow::anyhow!("Position not found"))
    }

    /// Update position with current market data
    pub async fn update_position(&self, position_id: Uuid, current_price: f64) -> Result<()> {
        let mut positions = self.positions.write().await;
        
        if let Some(position) = positions.get_mut(&position_id) {
            position.current_price = current_price;
            position.value_usd = position.quantity * current_price;
            position.unrealized_pnl = (current_price - position.entry_price) * position.quantity;
            position.last_update = Utc::now();
        }
        
        Ok(())
    }

    /// Get all current positions
    pub async fn get_positions(&self) -> HashMap<Uuid, Position> {
        self.positions.read().await.clone()
    }

    /// Get positions for a specific strategy
    pub async fn get_strategy_positions(&self, strategy: &str) -> Vec<Position> {
        let positions = self.positions.read().await;
        positions.values()
            .filter(|p| p.strategy == strategy)
            .cloned()
            .collect()
    }

    /// Calculate total portfolio value
    pub async fn calculate_total_value(&self, positions: &HashMap<Uuid, Position>) -> f64 {
        positions.values().map(|p| p.value_usd).sum()
    }

    /// Calculate portfolio metrics
    pub async fn calculate_metrics(&self) -> Result<PortfolioMetrics> {
        let positions = self.positions.read().await;
        let total_value = self.calculate_total_value(&positions).await;
        
        let total_unrealized_pnl: f64 = positions.values().map(|p| p.unrealized_pnl).sum();
        let total_realized_pnl: f64 = positions.values().map(|p| p.realized_pnl).sum();
        let total_pnl = total_unrealized_pnl + total_realized_pnl;
        
        let total_return_percent = if self.config.total_capital > 0.0 {
            (total_pnl / self.config.total_capital) * 100.0
        } else {
            0.0
        };

        // Calculate daily PnL (simplified - would need historical data)
        let daily_pnl = total_unrealized_pnl; // Placeholder

        // Calculate basic risk metrics (simplified)
        let sharpe_ratio = if total_return_percent > 0.0 { 1.5 } else { 0.0 }; // Placeholder
        let sortino_ratio = if total_return_percent > 0.0 { 1.8 } else { 0.0 }; // Placeholder
        let max_drawdown = positions.values()
            .map(|p| p.risk_metrics.max_drawdown)
            .fold(0.0, f64::max);

        // Win rate calculation (simplified)
        let profitable_positions = positions.values().filter(|p| p.unrealized_pnl > 0.0).count();
        let total_positions = positions.len();
        let win_rate = if total_positions > 0 {
            (profitable_positions as f64 / total_positions as f64) * 100.0
        } else {
            0.0
        };

        // Average win/loss (simplified)
        let winning_positions: Vec<_> = positions.values().filter(|p| p.unrealized_pnl > 0.0).collect();
        let losing_positions: Vec<_> = positions.values().filter(|p| p.unrealized_pnl < 0.0).collect();
        
        let average_win = if !winning_positions.is_empty() {
            winning_positions.iter().map(|p| p.unrealized_pnl).sum::<f64>() / winning_positions.len() as f64
        } else {
            0.0
        };

        let average_loss = if !losing_positions.is_empty() {
            losing_positions.iter().map(|p| p.unrealized_pnl.abs()).sum::<f64>() / losing_positions.len() as f64
        } else {
            0.0
        };

        Ok(PortfolioMetrics {
            total_value,
            total_pnl,
            daily_pnl,
            total_return_percent,
            sharpe_ratio,
            sortino_ratio,
            max_drawdown,
            win_rate,
            average_win,
            average_loss,
            correlation_score: 0.5, // Placeholder - would need correlation analysis
        })
    }

    /// Check if portfolio needs rebalancing
    pub async fn needs_rebalancing(&self) -> bool {
        match self.config.rebalance_frequency {
            RebalanceFrequency::Never => false,
            RebalanceFrequency::Daily => {
                (Utc::now() - self.last_rebalance).num_hours() >= 24
            },
            RebalanceFrequency::Weekly => {
                (Utc::now() - self.last_rebalance).num_days() >= 7
            },
            RebalanceFrequency::Monthly => {
                (Utc::now() - self.last_rebalance).num_days() >= 30
            },
            RebalanceFrequency::ThresholdBased(threshold) => {
                // Check if any strategy allocation drifted beyond threshold
                self.check_allocation_drift(threshold).await
            }
        }
    }

    /// Check allocation drift for threshold-based rebalancing
    async fn check_allocation_drift(&self, threshold: f64) -> bool {
        let positions = self.positions.read().await;
        let total_value = self.calculate_total_value(&positions).await;
        
        if total_value == 0.0 {
            return false;
        }

        // Calculate current strategy allocations
        let mut current_allocations = HashMap::new();
        for position in positions.values() {
            let strategy_value = current_allocations.entry(position.strategy.clone())
                .or_insert(0.0);
            *strategy_value += position.value_usd;
        }

        // Convert to percentages and compare with target allocations
        for (strategy, target_pct) in &self.strategy_allocations {
            let current_pct = current_allocations.get(strategy).unwrap_or(&0.0) / total_value;
            let drift = (current_pct - target_pct).abs();
            
            if drift > threshold {
                return true;
            }
        }

        false
    }

    /// Get strategy allocation summary
    pub async fn get_strategy_allocations(&self) -> HashMap<String, f64> {
        let positions = self.positions.read().await;
        let total_value = self.calculate_total_value(&positions).await;
        
        if total_value == 0.0 {
            return HashMap::new();
        }

        let mut allocations = HashMap::new();
        for position in positions.values() {
            let strategy_value = allocations.entry(position.strategy.clone())
                .or_insert(0.0);
            *strategy_value += position.value_usd;
        }

        // Convert to percentages
        for value in allocations.values_mut() {
            *value = (*value / total_value) * 100.0;
        }

        allocations
    }

    /// Get portfolio summary for display
    pub async fn get_summary(&self) -> Result<String> {
        let metrics = self.calculate_metrics().await?;
        let allocations = self.get_strategy_allocations().await;
        
        let mut summary = String::new();
        summary.push_str(&format!("📊 Portfolio Summary\n"));
        summary.push_str(&format!("═══════════════════\n"));
        summary.push_str(&format!("💰 Total Value: ${:.2}\n", metrics.total_value));
        summary.push_str(&format!("📈 Total P&L: ${:.2} ({:.2}%)\n", metrics.total_pnl, metrics.total_return_percent));
        summary.push_str(&format!("📊 Daily P&L: ${:.2}\n", metrics.daily_pnl));
        summary.push_str(&format!("📉 Max Drawdown: {:.2}%\n", metrics.max_drawdown));
        summary.push_str(&format!("🎯 Win Rate: {:.1}%\n", metrics.win_rate));
        summary.push_str(&format!("📊 Sharpe Ratio: {:.2}\n", metrics.sharpe_ratio));
        summary.push_str(&format!("\n🎯 Strategy Allocations:\n"));
        
        for (strategy, allocation) in allocations {
            summary.push_str(&format!("  • {}: {:.1}%\n", strategy, allocation));
        }

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio::{PositionRiskMetrics, RebalanceFrequency, GlobalRiskLimits};

    fn create_test_config() -> PortfolioConfiguration {
        let mut strategy_allocations = HashMap::new();
        strategy_allocations.insert("trend".to_string(), 0.4);
        strategy_allocations.insert("momentum".to_string(), 0.3);
        strategy_allocations.insert("mean_reversion".to_string(), 0.3);

        PortfolioConfiguration {
            total_capital: 10000.0,
            max_positions: 10,
            max_correlation: 0.7,
            rebalance_frequency: RebalanceFrequency::Daily,
            risk_limits: GlobalRiskLimits {
                max_portfolio_drawdown: 0.15,
                max_daily_loss: 500.0,
                max_position_concentration: 0.2,
                max_strategy_allocation: 0.5,
                correlation_limit: 0.7,
            },
            strategy_allocations,
        }
    }

    fn create_test_position() -> Position {
        Position {
            id: Uuid::new_v4(),
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: "USDC".to_string(),
            strategy: "trend".to_string(),
            entry_price: 1.0,
            current_price: 1.05,
            quantity: 1000.0,
            value_usd: 1050.0,
            unrealized_pnl: 50.0,
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

    #[tokio::test]
    async fn test_portfolio_creation() {
        let config = create_test_config();
        let manager = PortfolioManager::new(config);
        
        let positions = manager.get_positions().await;
        assert!(positions.is_empty());
    }

    #[tokio::test]
    async fn test_add_position() {
        let config = create_test_config();
        let manager = PortfolioManager::new(config);
        let position = create_test_position();
        
        let result = manager.add_position(position).await;
        assert!(result.is_ok());
        
        let positions = manager.get_positions().await;
        assert_eq!(positions.len(), 1);
    }

    #[tokio::test]
    async fn test_calculate_metrics() {
        let config = create_test_config();
        let manager = PortfolioManager::new(config);
        let position = create_test_position();
        
        manager.add_position(position).await.unwrap();
        
        let metrics = manager.calculate_metrics().await.unwrap();
        assert_eq!(metrics.total_value, 1050.0);
        assert_eq!(metrics.total_pnl, 50.0);
        assert_eq!(metrics.total_return_percent, 0.5);
    }
}
