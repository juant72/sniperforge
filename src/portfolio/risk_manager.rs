//! Portfolio Risk Manager - Advanced risk management for multi-strategy portfolios
//! 
//! Provides comprehensive risk controls including VaR, position limits, and portfolio-level risk monitoring

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

use super::{Position, PortfolioMetrics, GlobalRiskLimits};

/// Portfolio-level risk manager
#[derive(Debug)]
pub struct PortfolioRiskManager {
    limits: GlobalRiskLimits,
    daily_losses: Vec<(DateTime<Utc>, f64)>,
    max_drawdown_history: Vec<f64>,
}

/// Risk limits for the portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_portfolio_drawdown: f64,
    pub max_daily_loss: f64,
    pub max_position_concentration: f64,
    pub max_strategy_allocation: f64,
    pub correlation_limit: f64,
    pub var_limit_95: f64,
    pub var_limit_99: f64,
}

/// Value at Risk calculator
#[derive(Debug)]
pub struct VaRCalculator {
    confidence_levels: Vec<f64>,
    historical_returns: Vec<f64>,
}

/// Risk assessment result
#[derive(Debug, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_score: f64,
    pub var_95: f64,
    pub var_99: f64,
    pub portfolio_beta: f64,
    pub concentration_risk: f64,
    pub correlation_risk: f64,
    pub liquidity_risk: f64,
    pub risk_violations: Vec<RiskViolation>,
    pub recommendations: Vec<String>,
}

/// Risk violation details
#[derive(Debug, Serialize, Deserialize)]
pub struct RiskViolation {
    pub violation_type: RiskViolationType,
    pub current_value: f64,
    pub limit_value: f64,
    pub severity: RiskSeverity,
    pub description: String,
}

/// Types of risk violations
#[derive(Debug, Serialize, Deserialize)]
pub enum RiskViolationType {
    PositionConcentration,
    StrategyAllocation,
    PortfolioDrawdown,
    DailyLoss,
    VaRExceeded,
    CorrelationLimit,
    LiquidityRisk,
}

/// Severity of risk violations
#[derive(Debug, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl PortfolioRiskManager {
    /// Create new portfolio risk manager
    pub fn new(limits: GlobalRiskLimits) -> Self {
        Self {
            limits,
            daily_losses: Vec::new(),
            max_drawdown_history: Vec::new(),
        }
    }

    /// Assess overall portfolio risk
    pub async fn assess_risk(&mut self, positions: &HashMap<uuid::Uuid, Position>, total_capital: f64) -> Result<RiskAssessment> {
        let mut violations = Vec::new();
        let mut recommendations = Vec::new();

        // Calculate total portfolio value
        let total_value: f64 = positions.values().map(|p| p.value_usd).sum();
        let _total_pnl: f64 = positions.values().map(|p| p.unrealized_pnl + p.realized_pnl).sum();

        // 1. Check position concentration
        let concentration_risk = self.check_position_concentration(positions, total_value, &mut violations);

        // 2. Check strategy allocation
        self.check_strategy_allocation(positions, total_value, &mut violations);

        // 3. Check portfolio drawdown
        let current_drawdown = if total_capital > 0.0 {
            (total_capital - total_value) / total_capital
        } else {
            0.0
        };
        
        if current_drawdown > self.limits.max_portfolio_drawdown {
            violations.push(RiskViolation {
                violation_type: RiskViolationType::PortfolioDrawdown,
                current_value: current_drawdown,
                limit_value: self.limits.max_portfolio_drawdown,
                severity: RiskSeverity::High,
                description: "Portfolio drawdown exceeds maximum limit".to_string(),
            });
        }

        // 4. Check daily loss
        let daily_loss = self.calculate_daily_loss(positions);
        if daily_loss > self.limits.max_daily_loss {
            violations.push(RiskViolation {
                violation_type: RiskViolationType::DailyLoss,
                current_value: daily_loss,
                limit_value: self.limits.max_daily_loss,
                severity: RiskSeverity::Critical,
                description: "Daily loss exceeds maximum limit".to_string(),
            });
        }

        // 5. Calculate VaR
        let var_calculator = VaRCalculator::new(positions);
        let var_95 = var_calculator.calculate_var(0.95);
        let var_99 = var_calculator.calculate_var(0.99);

        // 6. Calculate portfolio beta (simplified)
        let portfolio_beta = self.calculate_portfolio_beta(positions);

        // 7. Calculate correlation risk
        let correlation_risk = self.calculate_correlation_risk(positions);

        // 8. Calculate liquidity risk
        let liquidity_risk = self.calculate_liquidity_risk(positions);

        // Generate recommendations
        if concentration_risk > 0.8 {
            recommendations.push("Consider diversifying large positions to reduce concentration risk".to_string());
        }
        
        if correlation_risk > 0.7 {
            recommendations.push("High correlation detected between positions - consider reducing correlated exposure".to_string());
        }

        if violations.len() > 2 {
            recommendations.push("Multiple risk violations detected - immediate risk reduction recommended".to_string());
        }

        // Calculate overall risk score (0-1, where 1 is highest risk)
        let overall_risk_score = self.calculate_overall_risk_score(
            concentration_risk, correlation_risk, liquidity_risk, violations.len()
        );

        Ok(RiskAssessment {
            overall_risk_score,
            var_95,
            var_99,
            portfolio_beta,
            concentration_risk,
            correlation_risk,
            liquidity_risk,
            risk_violations: violations,
            recommendations,
        })
    }

    /// Check position concentration risk
    fn check_position_concentration(&self, positions: &HashMap<uuid::Uuid, Position>, total_value: f64, violations: &mut Vec<RiskViolation>) -> f64 {
        if total_value == 0.0 {
            return 0.0;
        }

        let mut max_concentration: f64 = 0.0;
        
        for position in positions.values() {
            let concentration = position.value_usd / total_value;
            max_concentration = max_concentration.max(concentration);
            
            if concentration > self.limits.max_position_concentration {
                violations.push(RiskViolation {
                    violation_type: RiskViolationType::PositionConcentration,
                    current_value: concentration,
                    limit_value: self.limits.max_position_concentration,
                    severity: if concentration > self.limits.max_position_concentration * 1.5 {
                        RiskSeverity::High
                    } else {
                        RiskSeverity::Medium
                    },
                    description: format!("Position {} exceeds concentration limit", position.symbol),
                });
            }
        }

        max_concentration
    }

    /// Check strategy allocation limits
    fn check_strategy_allocation(&self, positions: &HashMap<uuid::Uuid, Position>, total_value: f64, violations: &mut Vec<RiskViolation>) {
        if total_value == 0.0 {
            return;
        }

        let mut strategy_allocations = HashMap::new();
        
        for position in positions.values() {
            let allocation = strategy_allocations.entry(position.strategy.clone()).or_insert(0.0);
            *allocation += position.value_usd;
        }

        for (strategy, allocation) in strategy_allocations {
            let allocation_pct = allocation / total_value;
            
            if allocation_pct > self.limits.max_strategy_allocation {
                violations.push(RiskViolation {
                    violation_type: RiskViolationType::StrategyAllocation,
                    current_value: allocation_pct,
                    limit_value: self.limits.max_strategy_allocation,
                    severity: RiskSeverity::Medium,
                    description: format!("Strategy {} allocation exceeds limit", strategy),
                });
            }
        }
    }

    /// Calculate daily loss
    fn calculate_daily_loss(&mut self, positions: &HashMap<uuid::Uuid, Position>) -> f64 {
        let now = Utc::now();
        let daily_pnl: f64 = positions.values()
            .filter(|p| (now - p.last_update).num_hours() < 24)
            .map(|p| if p.unrealized_pnl < 0.0 { p.unrealized_pnl.abs() } else { 0.0 })
            .sum();

        // Store daily loss for history
        self.daily_losses.push((now, daily_pnl));
        
        // Keep only last 30 days
        self.daily_losses.retain(|(date, _)| (now - *date).num_days() <= 30);

        daily_pnl
    }

    /// Calculate portfolio beta
    fn calculate_portfolio_beta(&self, positions: &HashMap<uuid::Uuid, Position>) -> f64 {
        if positions.is_empty() {
            return 1.0;
        }

        let total_value: f64 = positions.values().map(|p| p.value_usd).sum();
        
        if total_value == 0.0 {
            return 1.0;
        }

        let weighted_beta: f64 = positions.values()
            .map(|p| (p.value_usd / total_value) * p.risk_metrics.beta)
            .sum();

        weighted_beta
    }

    /// Calculate correlation risk
    fn calculate_correlation_risk(&self, positions: &HashMap<uuid::Uuid, Position>) -> f64 {
        if positions.len() < 2 {
            return 0.0;
        }

        // Simplified correlation calculation
        // In a real implementation, this would use historical price data
        let symbols: Vec<_> = positions.values().map(|p| &p.symbol).collect();
        let mut high_correlations = 0;
        let mut total_pairs = 0;

        for i in 0..symbols.len() {
            for j in (i + 1)..symbols.len() {
                total_pairs += 1;
                // Simulate correlation check (in real implementation, calculate from price data)
                if symbols[i] == symbols[j] || 
                   (symbols[i].contains("SOL") && symbols[j].contains("SOL")) {
                    high_correlations += 1;
                }
            }
        }

        if total_pairs == 0 {
            0.0
        } else {
            high_correlations as f64 / total_pairs as f64
        }
    }

    /// Calculate liquidity risk
    fn calculate_liquidity_risk(&self, positions: &HashMap<uuid::Uuid, Position>) -> f64 {
        if positions.is_empty() {
            return 0.0;
        }

        // Simplified liquidity risk calculation
        // In a real implementation, this would consider:
        // - Trading volume
        // - Bid-ask spreads
        // - Market depth
        // - Token liquidity on DEXs

        let illiquid_positions = positions.values()
            .filter(|p| p.value_usd > 10000.0) // Large positions are harder to exit
            .count();

        illiquid_positions as f64 / positions.len() as f64
    }

    /// Calculate overall risk score
    fn calculate_overall_risk_score(&self, concentration_risk: f64, correlation_risk: f64, liquidity_risk: f64, violation_count: usize) -> f64 {
        let base_risk = (concentration_risk + correlation_risk + liquidity_risk) / 3.0;
        let violation_penalty = (violation_count as f64 * 0.1).min(0.5);
        
        (base_risk + violation_penalty).min(1.0)
    }

    /// Check if trading should be halted due to risk
    pub fn should_halt_trading(&self, assessment: &RiskAssessment) -> bool {
        assessment.risk_violations.iter().any(|v| {
            matches!(v.severity, RiskSeverity::Critical) ||
            matches!(v.violation_type, RiskViolationType::DailyLoss)
        })
    }

    /// Get risk summary for display
    pub fn get_risk_summary(&self, assessment: &RiskAssessment) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("🛡️ Risk Assessment\n"));
        summary.push_str(&format!("═══════════════════\n"));
        summary.push_str(&format!("📊 Overall Risk Score: {:.2}/1.0\n", assessment.overall_risk_score));
        summary.push_str(&format!("📉 VaR (95%): ${:.2}\n", assessment.var_95));
        summary.push_str(&format!("📉 VaR (99%): ${:.2}\n", assessment.var_99));
        summary.push_str(&format!("📈 Portfolio Beta: {:.2}\n", assessment.portfolio_beta));
        summary.push_str(&format!("🎯 Concentration Risk: {:.2}\n", assessment.concentration_risk));
        summary.push_str(&format!("🔗 Correlation Risk: {:.2}\n", assessment.correlation_risk));
        summary.push_str(&format!("💧 Liquidity Risk: {:.2}\n", assessment.liquidity_risk));
        
        if !assessment.risk_violations.is_empty() {
            summary.push_str(&format!("\n⚠️ Risk Violations ({}):\n", assessment.risk_violations.len()));
            for violation in &assessment.risk_violations {
                let severity_emoji = match violation.severity {
                    RiskSeverity::Low => "🟡",
                    RiskSeverity::Medium => "🟠",
                    RiskSeverity::High => "🔴",
                    RiskSeverity::Critical => "🚨",
                };
                summary.push_str(&format!("  {} {}\n", severity_emoji, violation.description));
            }
        }

        if !assessment.recommendations.is_empty() {
            summary.push_str(&format!("\n💡 Recommendations:\n"));
            for rec in &assessment.recommendations {
                summary.push_str(&format!("  • {}\n", rec));
            }
        }

        summary
    }
}

impl VaRCalculator {
    /// Create new VaR calculator from positions
    pub fn new(positions: &HashMap<uuid::Uuid, Position>) -> Self {
        // Simplified VaR calculation using position volatilities
        let historical_returns: Vec<f64> = positions.values()
            .map(|p| p.risk_metrics.volatility)
            .collect();

        Self {
            confidence_levels: vec![0.95, 0.99],
            historical_returns,
        }
    }

    /// Calculate Value at Risk for given confidence level
    pub fn calculate_var(&self, confidence_level: f64) -> f64 {
        if self.historical_returns.is_empty() {
            return 0.0;
        }

        // Simplified VaR using normal distribution approximation
        let mean_return = self.historical_returns.iter().sum::<f64>() / self.historical_returns.len() as f64;
        let variance = self.historical_returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / self.historical_returns.len() as f64;
        let std_dev = variance.sqrt();

        // Z-score for confidence level (approximation)
        let z_score = if confidence_level >= 0.99 {
            2.33
        } else if confidence_level >= 0.95 {
            1.65
        } else {
            1.0
        };

        // VaR = mean - (z_score * std_dev) * portfolio_value
        // For simplicity, assume portfolio value of 1000
        let portfolio_value = 1000.0;
        (mean_return - z_score * std_dev) * portfolio_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio::{PositionRiskMetrics, GlobalRiskLimits};
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_limits() -> GlobalRiskLimits {
        GlobalRiskLimits {
            max_portfolio_drawdown: 0.15,
            max_daily_loss: 500.0,
            max_position_concentration: 0.2,
            max_strategy_allocation: 0.5,
            correlation_limit: 0.7,
        }
    }

    fn create_test_position(symbol: &str, strategy: &str, value: f64) -> Position {
        Position {
            id: Uuid::new_v4(),
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: symbol.to_string(),
            strategy: strategy.to_string(),
            entry_price: 1.0,
            current_price: 1.0,
            quantity: value,
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
    async fn test_risk_manager_creation() {
        let limits = create_test_limits();
        let risk_manager = PortfolioRiskManager::new(limits);
        
        assert!(risk_manager.daily_losses.is_empty());
    }

    #[tokio::test]
    async fn test_position_concentration_check() {
        let limits = create_test_limits();
        let mut risk_manager = PortfolioRiskManager::new(limits);
        
        let mut positions = HashMap::new();
        // Create a position that exceeds concentration limit (30% when limit is 20%)
        positions.insert(Uuid::new_v4(), create_test_position("SOL", "trend", 3000.0));
        positions.insert(Uuid::new_v4(), create_test_position("ETH", "momentum", 7000.0));

        let assessment = risk_manager.assess_risk(&positions, 10000.0).await.unwrap();
        
        // Should have concentration violation
        assert!(!assessment.risk_violations.is_empty());
        assert!(assessment.risk_violations.iter().any(|v| 
            matches!(v.violation_type, RiskViolationType::PositionConcentration)
        ));
    }

    #[tokio::test]
    async fn test_var_calculation() {
        // Create a VaR calculator with diverse historical returns for realistic testing
        let historical_returns = vec![-0.05, 0.03, -0.02, 0.01, -0.08, 0.04, -0.01, 0.02];
        let var_calculator = VaRCalculator {
            confidence_levels: vec![0.95, 0.99],
            historical_returns,
        };
        
        let var_95 = var_calculator.calculate_var(0.95);
        let var_99 = var_calculator.calculate_var(0.99);
        
        // VaR 99% should be lower (more negative) than VaR 95% due to higher confidence level
        assert!(var_99 < var_95, "VaR 99% ({}) should be more negative than VaR 95% ({})", var_99, var_95);
    }
}
