//! Portfolio Rebalancer - Automated portfolio rebalancing system
//!
//! Handles automatic rebalancing based on strategy performance and allocation targets

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{Position, RebalanceFrequency};

/// Portfolio rebalancer for maintaining target allocations
#[derive(Debug)]
pub struct PortfolioRebalancer {
    config: RebalanceConfig,
    last_rebalance: DateTime<Utc>,
    rebalance_history: Vec<RebalanceEvent>,
}

/// Configuration for rebalancing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceConfig {
    pub target_allocations: HashMap<String, f64>, // Strategy -> target percentage
    pub rebalance_threshold: f64,                 // Threshold for triggering rebalance
    pub min_trade_size: f64,                      // Minimum trade size for rebalancing
    pub max_rebalance_frequency: RebalanceFrequency,
    pub enabled: bool,
}

/// Rebalancing action to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceAction {
    pub action_type: ActionType,
    pub position_id: Option<Uuid>,
    pub strategy: String,
    pub symbol: String,
    pub target_amount: f64,
    pub current_amount: f64,
    pub trade_amount: f64,
    pub priority: RebalancePriority,
    pub reasoning: String,
}

/// Type of rebalance action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Increase,  // Increase position size
    Decrease,  // Decrease position size
    Close,     // Close position entirely
    Open,      // Open new position
    Rebalance, // Partial rebalance
}

/// Priority of rebalance action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RebalancePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Historical rebalance event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceEvent {
    pub timestamp: DateTime<Utc>,
    pub trigger_reason: String,
    pub actions_executed: Vec<RebalanceAction>,
    pub pre_rebalance_allocations: HashMap<String, f64>,
    pub post_rebalance_allocations: HashMap<String, f64>,
    pub total_trades: usize,
    pub total_volume: f64,
    pub success: bool,
}

/// Rebalance analysis result
#[derive(Debug, Serialize, Deserialize)]
pub struct RebalanceAnalysis {
    pub needs_rebalancing: bool,
    pub drift_analysis: HashMap<String, AllocationDrift>,
    pub recommended_actions: Vec<RebalanceAction>,
    pub total_trades_needed: usize,
    pub estimated_volume: f64,
    pub estimated_costs: f64,
    pub risk_impact: RiskImpact,
}

/// Allocation drift for a strategy
#[derive(Debug, Serialize, Deserialize)]
pub struct AllocationDrift {
    pub target_allocation: f64,
    pub current_allocation: f64,
    pub drift_percentage: f64,
    pub drift_amount: f64,
    pub requires_action: bool,
}

/// Impact of rebalancing on risk metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct RiskImpact {
    pub correlation_change: f64,
    pub concentration_change: f64,
    pub volatility_change: f64,
    pub expected_risk_reduction: f64,
}

impl PortfolioRebalancer {
    /// Create new portfolio rebalancer
    pub fn new(config: RebalanceConfig) -> Self {
        Self {
            config,
            last_rebalance: Utc::now(),
            rebalance_history: Vec::new(),
        }
    }

    /// Analyze if portfolio needs rebalancing
    pub async fn analyze_rebalance_needs(
        &self,
        positions: &HashMap<Uuid, Position>,
        _total_capital: f64,
    ) -> Result<RebalanceAnalysis> {
        if !self.config.enabled {
            return Ok(RebalanceAnalysis {
                needs_rebalancing: false,
                drift_analysis: HashMap::new(),
                recommended_actions: Vec::new(),
                total_trades_needed: 0,
                estimated_volume: 0.0,
                estimated_costs: 0.0,
                risk_impact: RiskImpact {
                    correlation_change: 0.0,
                    concentration_change: 0.0,
                    volatility_change: 0.0,
                    expected_risk_reduction: 0.0,
                },
            });
        }

        // Calculate current allocations
        let current_allocations = self.calculate_current_allocations(positions);
        let total_value: f64 = positions.values().map(|p| p.value_usd).sum();

        // Analyze drift for each strategy
        let mut drift_analysis = HashMap::new();
        let mut needs_rebalancing = false;

        for (strategy, &target_pct) in &self.config.target_allocations {
            let current_pct = current_allocations.get(strategy).unwrap_or(&0.0);
            let drift_pct = (current_pct - target_pct).abs();
            let drift_amount = drift_pct * total_value;

            let requires_action = drift_pct > self.config.rebalance_threshold;
            if requires_action {
                needs_rebalancing = true;
            }

            drift_analysis.insert(
                strategy.clone(),
                AllocationDrift {
                    target_allocation: target_pct,
                    current_allocation: *current_pct,
                    drift_percentage: drift_pct,
                    drift_amount,
                    requires_action,
                },
            );
        }

        // Generate recommended actions
        let recommended_actions = if needs_rebalancing {
            self.generate_rebalance_actions(&drift_analysis, positions, total_value)
                .await?
        } else {
            Vec::new()
        };

        // Calculate metrics
        let total_trades_needed = recommended_actions.len();
        let estimated_volume: f64 = recommended_actions
            .iter()
            .map(|a| a.trade_amount.abs())
            .sum();
        let estimated_costs = estimated_volume * 0.001; // Assume 0.1% trading costs

        // Calculate risk impact (simplified)
        let risk_impact = self.calculate_risk_impact(&recommended_actions, positions);

        Ok(RebalanceAnalysis {
            needs_rebalancing,
            drift_analysis,
            recommended_actions,
            total_trades_needed,
            estimated_volume,
            estimated_costs,
            risk_impact,
        })
    }

    /// Generate specific rebalance actions
    async fn generate_rebalance_actions(
        &self,
        drift_analysis: &HashMap<String, AllocationDrift>,
        positions: &HashMap<Uuid, Position>,
        total_value: f64,
    ) -> Result<Vec<RebalanceAction>> {
        let mut actions = Vec::new();

        for (strategy, drift) in drift_analysis {
            if !drift.requires_action {
                continue;
            }

            let target_value = drift.target_allocation * total_value;
            let current_value = drift.current_allocation * total_value;
            let adjustment_needed = target_value - current_value;

            if adjustment_needed.abs() < self.config.min_trade_size {
                continue;
            }

            // Get positions for this strategy
            let strategy_positions: Vec<_> = positions
                .values()
                .filter(|p| p.strategy == *strategy)
                .collect();

            if adjustment_needed > 0.0 {
                // Need to increase allocation - either increase existing positions or open new ones
                if !strategy_positions.is_empty() {
                    // Increase existing positions proportionally
                    let strategy_len = strategy_positions.len() as f64;
                    for position in &strategy_positions {
                        let proportional_increase = adjustment_needed / strategy_len;

                        actions.push(RebalanceAction {
                            action_type: ActionType::Increase,
                            position_id: Some(position.id),
                            strategy: strategy.clone(),
                            symbol: position.symbol.clone(),
                            target_amount: position.value_usd + proportional_increase,
                            current_amount: position.value_usd,
                            trade_amount: proportional_increase,
                            priority: self.determine_priority(drift.drift_percentage),
                            reasoning: format!(
                                "Increase {} allocation by ${:.2}",
                                strategy, proportional_increase
                            ),
                        });
                    }
                } else {
                    // Need to open new position for this strategy
                    actions.push(RebalanceAction {
                        action_type: ActionType::Open,
                        position_id: None,
                        strategy: strategy.clone(),
                        symbol: "SOL".to_string(), // Default to SOL for new positions
                        target_amount: adjustment_needed,
                        current_amount: 0.0,
                        trade_amount: adjustment_needed,
                        priority: self.determine_priority(drift.drift_percentage),
                        reasoning: format!(
                            "Open new {} position for ${:.2}",
                            strategy, adjustment_needed
                        ),
                    });
                }
            } else {
                // Need to decrease allocation
                let reduction_needed = adjustment_needed.abs();

                if strategy_positions.is_empty() {
                    continue;
                }

                // Reduce positions proportionally, starting with the worst performing
                let mut sorted_positions = strategy_positions;
                sorted_positions
                    .sort_by(|a, b| a.unrealized_pnl.partial_cmp(&b.unrealized_pnl).unwrap());

                let mut remaining_reduction = reduction_needed;

                for position in sorted_positions {
                    if remaining_reduction <= 0.0 {
                        break;
                    }

                    let position_reduction = remaining_reduction.min(position.value_usd);

                    let action_type = if position_reduction >= position.value_usd * 0.9 {
                        ActionType::Close
                    } else {
                        ActionType::Decrease
                    };

                    actions.push(RebalanceAction {
                        action_type,
                        position_id: Some(position.id),
                        strategy: strategy.clone(),
                        symbol: position.symbol.clone(),
                        target_amount: position.value_usd - position_reduction,
                        current_amount: position.value_usd,
                        trade_amount: -position_reduction,
                        priority: self.determine_priority(drift.drift_percentage),
                        reasoning: format!(
                            "Reduce {} allocation by ${:.2}",
                            strategy, position_reduction
                        ),
                    });

                    remaining_reduction -= position_reduction;
                }
            }
        }

        // Sort actions by priority
        actions.sort_by(|a, b| {
            let priority_order = |p: &RebalancePriority| match p {
                RebalancePriority::Critical => 0,
                RebalancePriority::High => 1,
                RebalancePriority::Medium => 2,
                RebalancePriority::Low => 3,
            };
            priority_order(&a.priority).cmp(&priority_order(&b.priority))
        });

        Ok(actions)
    }

    /// Calculate current strategy allocations
    fn calculate_current_allocations(
        &self,
        positions: &HashMap<Uuid, Position>,
    ) -> HashMap<String, f64> {
        let total_value: f64 = positions.values().map(|p| p.value_usd).sum();

        if total_value == 0.0 {
            return HashMap::new();
        }

        let mut allocations = HashMap::new();

        for position in positions.values() {
            let allocation = allocations.entry(position.strategy.clone()).or_insert(0.0);
            *allocation += position.value_usd;
        }

        // Convert to percentages
        for allocation in allocations.values_mut() {
            *allocation /= total_value;
        }

        allocations
    }

    /// Determine priority based on drift percentage
    fn determine_priority(&self, drift_percentage: f64) -> RebalancePriority {
        if drift_percentage > 0.2 {
            RebalancePriority::Critical
        } else if drift_percentage > 0.1 {
            RebalancePriority::High
        } else if drift_percentage > 0.05 {
            RebalancePriority::Medium
        } else {
            RebalancePriority::Low
        }
    }

    /// Calculate risk impact of rebalancing
    fn calculate_risk_impact(
        &self,
        actions: &[RebalanceAction],
        _positions: &HashMap<Uuid, Position>,
    ) -> RiskImpact {
        // Simplified risk impact calculation
        let volume_impact = actions.iter().map(|a| a.trade_amount.abs()).sum::<f64>();

        // Estimate that rebalancing generally reduces risk
        let expected_risk_reduction = (volume_impact / 10000.0).min(0.1); // Max 10% risk reduction

        RiskImpact {
            correlation_change: -0.05,   // Assume rebalancing reduces correlation
            concentration_change: -0.03, // Assume rebalancing reduces concentration
            volatility_change: -0.02,    // Assume rebalancing reduces volatility
            expected_risk_reduction,
        }
    }

    /// Execute rebalancing actions
    pub async fn execute_rebalance(
        &mut self,
        actions: Vec<RebalanceAction>,
        reason: String,
    ) -> Result<RebalanceEvent> {
        let timestamp = Utc::now();
        let pre_allocations = HashMap::new(); // Would be calculated from current positions

        // In a real implementation, this would execute the trades
        // For now, we'll just simulate success
        let success = true;
        let total_volume: f64 = actions.iter().map(|a| a.trade_amount.abs()).sum();

        let event = RebalanceEvent {
            timestamp,
            trigger_reason: reason,
            actions_executed: actions.clone(),
            pre_rebalance_allocations: pre_allocations.clone(),
            post_rebalance_allocations: pre_allocations, // Would be updated after execution
            total_trades: actions.len(),
            total_volume,
            success,
        };

        self.rebalance_history.push(event.clone());
        self.last_rebalance = timestamp;

        // Keep only last 100 rebalance events
        if self.rebalance_history.len() > 100 {
            self.rebalance_history.remove(0);
        }

        Ok(event)
    }

    /// Check if enough time has passed for rebalancing
    pub fn can_rebalance(&self) -> bool {
        let time_since_last = Utc::now() - self.last_rebalance;

        match self.config.max_rebalance_frequency {
            RebalanceFrequency::Never => false,
            RebalanceFrequency::Daily => time_since_last.num_hours() >= 24,
            RebalanceFrequency::Weekly => time_since_last.num_days() >= 7,
            RebalanceFrequency::Monthly => time_since_last.num_days() >= 30,
            RebalanceFrequency::ThresholdBased(_) => true, // Always allow threshold-based
        }
    }

    /// Get rebalancing summary
    pub fn get_rebalance_summary(&self, analysis: &RebalanceAnalysis) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("⚖️ Rebalance Analysis\n"));
        summary.push_str(&format!("═══════════════════\n"));
        summary.push_str(&format!(
            "🎯 Needs Rebalancing: {}\n",
            if analysis.needs_rebalancing {
                "Yes"
            } else {
                "No"
            }
        ));
        summary.push_str(&format!(
            "📊 Trades Needed: {}\n",
            analysis.total_trades_needed
        ));
        summary.push_str(&format!(
            "💰 Estimated Volume: ${:.2}\n",
            analysis.estimated_volume
        ));
        summary.push_str(&format!(
            "💸 Estimated Costs: ${:.2}\n",
            analysis.estimated_costs
        ));

        if !analysis.drift_analysis.is_empty() {
            summary.push_str(&format!("\n📈 Strategy Drift Analysis:\n"));
            for (strategy, drift) in &analysis.drift_analysis {
                let status = if drift.requires_action {
                    "⚠️"
                } else {
                    "✅"
                };
                summary.push_str(&format!(
                    "  {} {}: {:.1}% → {:.1}% (drift: {:.1}%)\n",
                    status,
                    strategy,
                    drift.current_allocation * 100.0,
                    drift.target_allocation * 100.0,
                    drift.drift_percentage * 100.0
                ));
            }
        }

        if !analysis.recommended_actions.is_empty() {
            summary.push_str(&format!("\n🔧 Recommended Actions:\n"));
            for action in &analysis.recommended_actions {
                let action_emoji = match action.action_type {
                    ActionType::Increase => "📈",
                    ActionType::Decrease => "📉",
                    ActionType::Close => "❌",
                    ActionType::Open => "🆕",
                    ActionType::Rebalance => "⚖️",
                };
                summary.push_str(&format!("  {} {}\n", action_emoji, action.reasoning));
            }
        }

        summary
    }

    /// Get rebalance history
    pub fn get_history(&self) -> &[RebalanceEvent] {
        &self.rebalance_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio::{PositionRiskMetrics, RebalanceFrequency};
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_config() -> RebalanceConfig {
        let mut target_allocations = HashMap::new();
        target_allocations.insert("trend".to_string(), 0.4);
        target_allocations.insert("momentum".to_string(), 0.3);
        target_allocations.insert("mean_reversion".to_string(), 0.3);

        RebalanceConfig {
            target_allocations,
            rebalance_threshold: 0.05, // 5%
            min_trade_size: 100.0,
            max_rebalance_frequency: RebalanceFrequency::Daily,
            enabled: true,
        }
    }

    fn create_test_position(strategy: &str, value: f64) -> Position {
        Position {
            id: Uuid::new_v4(),
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: "SOL".to_string(),
            strategy: strategy.to_string(),
            entry_price: 100.0,
            current_price: 100.0,
            quantity: value / 100.0,
            value_usd: value,
            unrealized_pnl: 0.0,
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
    async fn test_rebalancer_creation() {
        let config = create_test_config();
        let rebalancer = PortfolioRebalancer::new(config);

        assert!(rebalancer.rebalance_history.is_empty());
    }

    #[tokio::test]
    async fn test_balanced_portfolio() {
        let config = create_test_config();
        let rebalancer = PortfolioRebalancer::new(config);

        let mut positions = HashMap::new();
        positions.insert(Uuid::new_v4(), create_test_position("trend", 4000.0));
        positions.insert(Uuid::new_v4(), create_test_position("momentum", 3000.0));
        positions.insert(
            Uuid::new_v4(),
            create_test_position("mean_reversion", 3000.0),
        );

        let analysis = rebalancer
            .analyze_rebalance_needs(&positions, 10000.0)
            .await
            .unwrap();

        // Should not need rebalancing as allocations are correct
        assert!(!analysis.needs_rebalancing);
        assert_eq!(analysis.total_trades_needed, 0);
    }

    #[tokio::test]
    async fn test_unbalanced_portfolio() {
        let config = create_test_config();
        let rebalancer = PortfolioRebalancer::new(config);

        let mut positions = HashMap::new();
        // Over-allocated to trend (70% instead of 40%)
        positions.insert(Uuid::new_v4(), create_test_position("trend", 7000.0));
        positions.insert(Uuid::new_v4(), create_test_position("momentum", 2000.0));
        positions.insert(
            Uuid::new_v4(),
            create_test_position("mean_reversion", 1000.0),
        );

        let analysis = rebalancer
            .analyze_rebalance_needs(&positions, 10000.0)
            .await
            .unwrap();

        // Should need rebalancing due to large drift
        assert!(analysis.needs_rebalancing);
        assert!(analysis.total_trades_needed > 0);
    }

    #[tokio::test]
    async fn test_priority_determination() {
        let config = create_test_config();
        let rebalancer = PortfolioRebalancer::new(config);

        assert!(matches!(
            rebalancer.determine_priority(0.25),
            RebalancePriority::Critical
        ));
        assert!(matches!(
            rebalancer.determine_priority(0.15),
            RebalancePriority::High
        ));
        assert!(matches!(
            rebalancer.determine_priority(0.08),
            RebalancePriority::Medium
        ));
        assert!(matches!(
            rebalancer.determine_priority(0.03),
            RebalancePriority::Low
        ));
    }
}
