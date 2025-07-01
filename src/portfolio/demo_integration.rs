//! Portfolio Integration Demo
//!
//! Demonstrates minimal functional integration of portfolio management
//! with real trading data, prices, and transactions.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use super::{
    manager::{PortfolioManager, PortfolioPosition},
    real_data_integration::{PortfolioRealDataIntegration, PortfolioTransaction},
    real_time_integration::{LivePriceData, PriceSource, RealTimePortfolioIntegration},
    PortfolioConfiguration, Position, PositionRiskMetrics,
};
use crate::config::Config;

/// Minimal portfolio integration demo
#[derive(Debug)]
pub struct PortfolioIntegrationDemo {
    portfolio_manager: Arc<PortfolioManager>,
    mock_prices: Arc<RwLock<HashMap<String, f64>>>,
    demo_positions: Arc<RwLock<Vec<Position>>>,
    config: Config,
}

/// Demo trade execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoTradeResult {
    pub trade_id: Uuid,
    pub symbol: String,
    pub entry_price: f64,
    pub quantity: f64,
    pub strategy: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Demo portfolio metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoPortfolioSnapshot {
    pub timestamp: DateTime<Utc>,
    pub total_value: f64,
    pub total_pnl: f64,
    pub positions_count: usize,
    pub top_performers: Vec<DemoPositionSummary>,
    pub strategy_allocation: HashMap<String, f64>,
}

/// Simplified position summary for demo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoPositionSummary {
    pub symbol: String,
    pub pnl_percent: f64,
    pub value_usd: f64,
    pub strategy: String,
}

impl PortfolioIntegrationDemo {
    /// Create new demo instance
    pub fn new(config: Config) -> Self {
        let portfolio_config = PortfolioConfiguration::default();
        let portfolio_manager = Arc::new(PortfolioManager::new(portfolio_config));

        Self {
            portfolio_manager,
            mock_prices: Arc::new(RwLock::new(HashMap::new())),
            demo_positions: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Initialize demo with sample data
    pub async fn initialize_demo(&self) -> Result<()> {
        info!("🚀 Initializing Portfolio Integration Demo");

        // Set up mock price data for common tokens
        self.setup_mock_prices().await?;

        // Create sample positions
        self.create_sample_positions().await?;

        // Display initial portfolio state
        self.display_portfolio_summary().await?;

        info!("✅ Demo initialization complete");
        Ok(())
    }

    /// Setup mock price data for demo tokens
    async fn setup_mock_prices(&self) -> Result<()> {
        let mut prices = self.mock_prices.write().await;

        // Popular Solana tokens with realistic prices
        prices.insert(
            "So11111111111111111111111111111111111111112".to_string(),
            180.50,
        ); // SOL
        prices.insert(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1.00,
        ); // USDC
        prices.insert(
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
            1.00,
        ); // USDT
        prices.insert(
            "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So".to_string(),
            195.30,
        ); // mSOL
        prices.insert(
            "7kbnvuGBxxj8AG9qp8Scn56muWGaRaFqxg1FsRp3PaFT".to_string(),
            0.15,
        ); // UXD
        prices.insert(
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
            85.40,
        ); // BONK (scaled)

        info!("📊 Mock prices initialized for {} tokens", prices.len());
        Ok(())
    }

    /// Create sample positions for demonstration
    async fn create_sample_positions(&self) -> Result<()> {
        let prices = self.mock_prices.read().await;
        let mut demo_positions = self.demo_positions.write().await;

        // Position 1: SOL trend following (adjusted to avoid concentration limit)
        let sol_position = Position {
            id: Uuid::new_v4(),
            token_mint: "So11111111111111111111111111111111111111112".to_string(),
            symbol: "SOL".to_string(),
            strategy: "trend_following".to_string(),
            entry_price: 175.00,
            current_price: *prices
                .get("So11111111111111111111111111111111111111112")
                .unwrap_or(&180.50),
            quantity: 1.5, // Reduced from 5.0 to 1.5
            value_usd: 1.5 * 180.50,
            unrealized_pnl: 1.5 * (180.50 - 175.00),
            realized_pnl: 0.0,
            entry_time: Utc::now() - chrono::Duration::hours(6),
            last_update: Utc::now(),
            risk_metrics: PositionRiskMetrics {
                var_95: 13.5, // Adjusted proportionally
                var_99: 21.6,
                volatility: 0.35,
                beta: 1.0,
                max_drawdown: 0.0,
            },
        };

        // Position 2: mSOL momentum (adjusted)
        let msol_position = Position {
            id: Uuid::new_v4(),
            token_mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So".to_string(),
            symbol: "mSOL".to_string(),
            strategy: "momentum".to_string(),
            entry_price: 190.00,
            current_price: *prices
                .get("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So")
                .unwrap_or(&195.30),
            quantity: 1.2, // Reduced from 2.5 to 1.2
            value_usd: 1.2 * 195.30,
            unrealized_pnl: 1.2 * (195.30 - 190.00),
            realized_pnl: 0.0,
            entry_time: Utc::now() - chrono::Duration::hours(4),
            last_update: Utc::now(),
            risk_metrics: PositionRiskMetrics {
                var_95: 11.5, // Adjusted proportionally
                var_99: 18.7,
                volatility: 0.28,
                beta: 0.95,
                max_drawdown: 0.0,
            },
        };

        // Position 3: USDC arbitrage (reduced to maintain balance)
        let usdc_position = Position {
            id: Uuid::new_v4(),
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: "USDC".to_string(),
            strategy: "arbitrage".to_string(),
            entry_price: 1.00,
            current_price: *prices
                .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                .unwrap_or(&1.00),
            quantity: 1200.0, // Increased to balance portfolio
            value_usd: 1200.0 * 1.00,
            unrealized_pnl: 0.0,
            realized_pnl: 12.5, // Some realized profit from arbitrage
            entry_time: Utc::now() - chrono::Duration::hours(2),
            last_update: Utc::now(),
            risk_metrics: PositionRiskMetrics {
                var_95: 2.5,
                var_99: 5.0,
                volatility: 0.01,
                beta: 0.01,
                max_drawdown: 0.0,
            },
        };

        // Add positions to portfolio
        self.portfolio_manager
            .add_position(sol_position.clone())
            .await?;
        self.portfolio_manager
            .add_position(msol_position.clone())
            .await?;
        self.portfolio_manager
            .add_position(usdc_position.clone())
            .await?;

        demo_positions.push(sol_position);
        demo_positions.push(msol_position);
        demo_positions.push(usdc_position);

        info!("📈 Created {} sample positions", demo_positions.len());
        Ok(())
    }

    /// Simulate a live trade execution
    pub async fn simulate_trade(
        &self,
        symbol: &str,
        strategy: &str,
        amount_usd: f64,
    ) -> Result<DemoTradeResult> {
        info!(
            "🔄 Simulating trade: {} ${} with {}",
            symbol, amount_usd, strategy
        );

        let prices = self.mock_prices.read().await;
        let token_price = match symbol {
            "SOL" => *prices
                .get("So11111111111111111111111111111111111111112")
                .unwrap_or(&180.0),
            "mSOL" => *prices
                .get("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So")
                .unwrap_or(&195.0),
            "USDC" => *prices
                .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                .unwrap_or(&1.0),
            _ => return Err(anyhow::anyhow!("Unsupported token for demo: {}", symbol)),
        };

        let quantity = amount_usd / token_price;
        let trade_id = Uuid::new_v4();

        // Simulate trade execution (always succeeds in demo)
        let trade_result = DemoTradeResult {
            trade_id,
            symbol: symbol.to_string(),
            entry_price: token_price,
            quantity,
            strategy: strategy.to_string(),
            timestamp: Utc::now(),
            success: true,
            error_message: None,
        };

        // Create new position from the trade
        let token_mint = match symbol {
            "SOL" => "So11111111111111111111111111111111111111112",
            "mSOL" => "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
            "USDC" => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            _ => return Err(anyhow::anyhow!("Unknown token mapping")),
        };

        let new_position = Position {
            id: Uuid::new_v4(),
            token_mint: token_mint.to_string(),
            symbol: symbol.to_string(),
            strategy: strategy.to_string(),
            entry_price: token_price,
            current_price: token_price,
            quantity,
            value_usd: amount_usd,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            entry_time: Utc::now(),
            last_update: Utc::now(),
            risk_metrics: PositionRiskMetrics {
                var_95: amount_usd * 0.05,
                var_99: amount_usd * 0.08,
                volatility: 0.25,
                beta: 1.0,
                max_drawdown: 0.0,
            },
        };

        // Add to portfolio
        self.portfolio_manager.add_position(new_position).await?;

        info!(
            "✅ Trade executed successfully: {} {} at ${}",
            quantity, symbol, token_price
        );
        Ok(trade_result)
    }

    /// Simulate price updates
    pub async fn simulate_price_update(&self, symbol: &str, new_price: f64) -> Result<()> {
        info!("📊 Updating price for {}: ${}", symbol, new_price);

        let mut prices = self.mock_prices.write().await;
        let token_mint = match symbol {
            "SOL" => "So11111111111111111111111111111111111111112",
            "mSOL" => "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
            "USDC" => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            _ => return Err(anyhow::anyhow!("Unsupported token: {}", symbol)),
        };

        prices.insert(token_mint.to_string(), new_price);

        // Update all positions with this token
        let positions = self.portfolio_manager.get_positions().await;
        for (position_id, position) in positions {
            if position.symbol == symbol {
                self.portfolio_manager
                    .update_position(position_id, new_price)
                    .await?;
            }
        }

        info!("✅ Price updated and positions refreshed for {}", symbol);
        Ok(())
    }

    /// Get current portfolio snapshot
    pub async fn get_portfolio_snapshot(&self) -> Result<DemoPortfolioSnapshot> {
        let metrics = self.portfolio_manager.calculate_metrics().await?;
        let positions = self.portfolio_manager.get_positions().await;
        let strategy_allocations = self.portfolio_manager.get_strategy_allocations().await;

        // Get top performing positions
        let mut position_summaries: Vec<DemoPositionSummary> = positions
            .values()
            .map(|p| {
                let pnl_percent = if p.entry_price > 0.0 {
                    ((p.current_price - p.entry_price) / p.entry_price) * 100.0
                } else {
                    0.0
                };

                DemoPositionSummary {
                    symbol: p.symbol.clone(),
                    pnl_percent,
                    value_usd: p.value_usd,
                    strategy: p.strategy.clone(),
                }
            })
            .collect();

        // Sort by P&L percentage (descending)
        position_summaries.sort_by(|a, b| b.pnl_percent.partial_cmp(&a.pnl_percent).unwrap());

        // Take top 3 performers
        position_summaries.truncate(3);

        Ok(DemoPortfolioSnapshot {
            timestamp: Utc::now(),
            total_value: metrics.total_value,
            total_pnl: metrics.total_pnl,
            positions_count: positions.len(),
            top_performers: position_summaries,
            strategy_allocation: strategy_allocations,
        })
    }

    /// Display portfolio summary to console
    pub async fn display_portfolio_summary(&self) -> Result<()> {
        let summary = self.portfolio_manager.get_summary().await?;
        let snapshot = self.get_portfolio_snapshot().await?;

        println!("\n{}", summary);
        println!("🏆 Top Performers:");
        for performer in &snapshot.top_performers {
            println!(
                "  • {}: {:.2}% (${:.2}) - {}",
                performer.symbol, performer.pnl_percent, performer.value_usd, performer.strategy
            );
        }
        println!();

        Ok(())
    }

    /// Run a complete demo workflow
    pub async fn run_complete_demo(&self) -> Result<()> {
        info!("🎯 Starting Complete Portfolio Integration Demo");

        // 1. Initialize demo
        self.initialize_demo().await?;

        // 2. Display initial state
        println!("📊 Initial Portfolio State:");
        self.display_portfolio_summary().await?;

        // 3. Simulate some market movements
        info!("📈 Simulating market movements...");
        self.simulate_price_update("SOL", 185.75).await?;
        self.simulate_price_update("mSOL", 198.50).await?;

        // 4. Execute a new trade
        info!("💼 Executing new trade...");
        let trade_result = self.simulate_trade("SOL", "momentum", 250.0).await?;
        info!("Trade executed: {:?}", trade_result);

        // 5. Show updated portfolio
        println!("📊 Updated Portfolio State (after trade and price movements):");
        self.display_portfolio_summary().await?;

        // 6. Demonstrate position tracking
        let snapshot = self.get_portfolio_snapshot().await?;
        info!(
            "📸 Portfolio Snapshot: {} positions, Total Value: ${:.2}, Total P&L: ${:.2}",
            snapshot.positions_count, snapshot.total_value, snapshot.total_pnl
        );

        // 7. Show strategy performance breakdown
        println!("🎯 Strategy Performance Breakdown:");
        for (strategy, allocation) in &snapshot.strategy_allocation {
            println!("  • {}: {:.1}% allocation", strategy, allocation);
        }

        info!("✅ Complete demo workflow finished successfully");
        Ok(())
    }
}

/// Convenience function to run a quick demo
pub async fn run_portfolio_demo(config: Config) -> Result<()> {
    let demo = PortfolioIntegrationDemo::new(config);
    demo.run_complete_demo().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_initialization() {
        let config = Config::load("config/devnet.toml").expect("Failed to load test config");
        let demo = PortfolioIntegrationDemo::new(config);

        let result = demo.initialize_demo().await;
        assert!(result.is_ok());

        let snapshot = demo.get_portfolio_snapshot().await.unwrap();
        assert_eq!(snapshot.positions_count, 3);
        assert!(snapshot.total_value > 0.0);
    }

    #[tokio::test]
    async fn test_simulate_trade() {
        let config = Config::load("config/devnet.toml").expect("Failed to load test config");
        let demo = PortfolioIntegrationDemo::new(config);

        demo.initialize_demo().await.unwrap();

        let trade_result = demo.simulate_trade("SOL", "momentum", 100.0).await.unwrap();
        assert!(trade_result.success);
        assert_eq!(trade_result.symbol, "SOL");
        assert_eq!(trade_result.strategy, "momentum");
    }

    #[tokio::test]
    async fn test_price_update() {
        let config = Config::load("config/devnet.toml").expect("Failed to load test config");
        let demo = PortfolioIntegrationDemo::new(config);

        demo.initialize_demo().await.unwrap();

        let result = demo.simulate_price_update("SOL", 200.0).await;
        assert!(result.is_ok());

        // Verify the price was updated
        let prices = demo.mock_prices.read().await;
        assert_eq!(
            *prices
                .get("So11111111111111111111111111111111111111112")
                .unwrap(),
            200.0
        );
    }

    #[tokio::test]
    async fn test_portfolio_snapshot() {
        let config = Config::load("config/devnet.toml").expect("Failed to load test config");
        let demo = PortfolioIntegrationDemo::new(config);

        demo.initialize_demo().await.unwrap();

        let snapshot = demo.get_portfolio_snapshot().await.unwrap();
        assert_eq!(snapshot.positions_count, 3);
        assert!(!snapshot.top_performers.is_empty());
        assert!(!snapshot.strategy_allocation.is_empty());
    }
}
