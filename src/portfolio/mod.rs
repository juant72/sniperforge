//! Portfolio Management Module for Phase 6C
//!
//! This module provides comprehensive portfolio management including:
//! - Multi-asset portfolio tracking and optimization
//! - Advanced risk management with portfolio-level controls
//! - Automated rebalancing based on strategy performance
//! - Performance attribution and detailed analytics
//! - Cross-correlation analysis between positions
//! - Real-time wallet scanning and blockchain analysis
//! - Live price feeds and market data integration
//! - Strategy performance tracking and attribution

pub mod analytics;
pub mod blockchain_analyzer;
pub mod correlation;
pub mod demo_integration;
pub mod manager;
pub mod price_feed;
pub mod professional_integration;
pub mod real_data_integration;
pub mod real_time_integration;
pub mod rebalancer;
pub mod risk_manager;
pub mod strategy_tracker;
pub mod transaction_analyzer;
pub mod wallet_scanner;

pub use analytics::{AttributionReport, PerformanceReport, PortfolioAnalytics};
pub use correlation::{CorrelationAnalyzer, CorrelationMatrix, DiversificationMetrics};
pub use manager::{PortfolioConfig, PortfolioManager, PortfolioPosition};
pub use rebalancer::{PortfolioRebalancer, RebalanceAction, RebalanceConfig};
pub use risk_manager::{PortfolioRiskManager, RiskLimits, VaRCalculator};

// Re-exports for convenience
pub use blockchain_analyzer::BlockchainAnalyzer;
pub use price_feed::{PriceFeed, TokenPrice};
pub use professional_integration::{ProfessionalPortfolioIntegration, ProfessionalPortfolioStatus};
pub use strategy_tracker::{StrategyPerformance, StrategyTracker};
pub use transaction_analyzer::{
    SolanaTransaction, TransactionAnalysis, TransactionAnalyzer, TransactionType,
};
pub use wallet_scanner::{TokenBalance, WalletBalance, WalletScanner};

// Legacy re-exports
pub use blockchain_analyzer::TransactionHistory;
pub use strategy_tracker::OverallPortfolioMetrics;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Portfolio configuration for multi-strategy trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioConfiguration {
    pub total_capital: f64,
    pub max_positions: u32,
    pub max_correlation: f64,
    pub rebalance_frequency: RebalanceFrequency,
    pub risk_limits: GlobalRiskLimits,
    pub strategy_allocations: HashMap<String, f64>,
}

/// Frequency for portfolio rebalancing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RebalanceFrequency {
    Never,
    Daily,
    Weekly,
    Monthly,
    ThresholdBased(f64), // Rebalance when drift exceeds percentage
}

/// Global portfolio risk limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalRiskLimits {
    pub max_portfolio_drawdown: f64,     // Maximum portfolio drawdown %
    pub max_daily_loss: f64,             // Maximum daily loss in USD
    pub max_position_concentration: f64, // Max % in single position
    pub max_strategy_allocation: f64,    // Max % to single strategy
    pub correlation_limit: f64,          // Max correlation between positions
}

/// Portfolio position with full tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
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
    pub risk_metrics: PositionRiskMetrics,
}

/// Risk metrics for individual positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRiskMetrics {
    pub var_95: f64,       // Value at Risk 95%
    pub var_99: f64,       // Value at Risk 99%
    pub volatility: f64,   // Historical volatility
    pub beta: f64,         // Beta vs SOL
    pub max_drawdown: f64, // Max drawdown since entry
}

/// Portfolio performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMetrics {
    pub total_value: f64,
    pub total_pnl: f64,
    pub daily_pnl: f64,
    pub total_return_percent: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub correlation_score: f64, // Portfolio diversification score
    pub var_95: f64,
    pub var_99: f64,
}

/// Default portfolio configuration
impl Default for PortfolioConfiguration {
    fn default() -> Self {
        let mut strategy_allocations = HashMap::new();
        strategy_allocations.insert("trend_following".to_string(), 0.3);
        strategy_allocations.insert("momentum".to_string(), 0.25);
        strategy_allocations.insert("mean_reversion".to_string(), 0.25);
        strategy_allocations.insert("arbitrage".to_string(), 0.2);

        Self {
            total_capital: 1000.0, // $1000 default
            max_positions: 20,
            max_correlation: 0.7,
            rebalance_frequency: RebalanceFrequency::ThresholdBased(0.1), // 10% drift
            risk_limits: GlobalRiskLimits {
                max_portfolio_drawdown: 0.15,    // 15%
                max_daily_loss: 100.0,           // $100
                max_position_concentration: 0.2, // 20%
                max_strategy_allocation: 0.4,    // 40%
                correlation_limit: 0.8,          // 80%
            },
            strategy_allocations,
        }
    }
}

/// Portfolio management events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortfolioEvent {
    PositionOpened {
        position_id: Uuid,
        strategy: String,
        symbol: String,
        value: f64,
    },
    PositionClosed {
        position_id: Uuid,
        pnl: f64,
        duration_hours: u64,
    },
    RebalanceTriggered {
        reason: String,
        actions: Vec<String>,
    },
    RiskLimitBreached {
        limit_type: String,
        current_value: f64,
        limit_value: f64,
    },
    CorrelationWarning {
        correlation: f64,
        positions: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_portfolio_config() {
        let config = PortfolioConfiguration::default();
        assert_eq!(config.total_capital, 1000.0);
        assert_eq!(config.max_positions, 20);
        assert!(config.strategy_allocations.len() == 4);

        // Verify allocations sum to 1.0
        let total: f64 = config.strategy_allocations.values().sum();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_position_creation() {
        let position = Position {
            id: Uuid::new_v4(),
            token_mint: "So11111111111111111111111111111111111111112".to_string(),
            symbol: "SOL".to_string(),
            strategy: "trend_following".to_string(),
            entry_price: 180.0,
            current_price: 185.0,
            quantity: 5.0,
            value_usd: 925.0,
            unrealized_pnl: 25.0,
            realized_pnl: 0.0,
            entry_time: Utc::now(),
            last_update: Utc::now(),
            risk_metrics: PositionRiskMetrics {
                var_95: 15.0,
                var_99: 25.0,
                volatility: 0.35,
                beta: 1.0,
                max_drawdown: 0.0,
            },
        };

        assert_eq!(position.symbol, "SOL");
        assert_eq!(position.unrealized_pnl, 25.0);
    }
}
