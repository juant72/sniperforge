//! Professional Portfolio Integration
//!
//! Real-time portfolio management with live market data, blockchain integration,
//! and professional trading capabilities.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::config::Config;
use super::{
    manager::{PortfolioManager, PortfolioPosition},
    real_data_integration::{PortfolioRealDataIntegration, RealPortfolioMetrics},
    real_time_integration::{RealTimePortfolioIntegration, LivePriceData},
    analytics::PortfolioAnalytics,
    risk_manager::PortfolioRiskManager,
};
use crate::shared::{
    jupiter::{Jupiter, JupiterClient},
    wallet_manager::WalletManager,
    real_time_blockchain::RealTimeBlockchainEngine,
    real_trading_engine::RealTradingEngine,
    websocket_price_feed::WebSocketPriceFeed,
    risk_manager::RiskManager,
};

/// Professional portfolio integration with full live data
#[derive(Debug)]
pub struct ProfessionalPortfolioIntegration {
    config: Config,
    portfolio_manager: Arc<PortfolioManager>,
    real_data_integration: Arc<PortfolioRealDataIntegration>,
    real_time_integration: Arc<RealTimePortfolioIntegration>,
    analytics: Arc<RwLock<PortfolioAnalytics>>,
    portfolio_risk_manager: Arc<PortfolioRiskManager>,
    shared_risk_manager: Arc<RiskManager>,
    jupiter_client: Arc<JupiterClient>,
    wallet_manager: Arc<WalletManager>,
    blockchain_engine: Arc<RealTimeBlockchainEngine>,
    trading_engine: Arc<RealTradingEngine>,
    price_feed: Arc<WebSocketPriceFeed>,
    live_prices: Arc<RwLock<HashMap<String, LivePriceData>>>,
}

/// Professional portfolio status with comprehensive metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfessionalPortfolioStatus {
    pub total_value: f64,
    pub total_invested: f64,
    pub total_pnl: f64,
    pub total_return_percent: f64,
    pub daily_pnl: f64,
    pub daily_return_percent: f64,
    pub positions_count: usize,
    pub active_strategies: Vec<String>,
    pub risk_score: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub total_fees_paid: f64,
    pub total_trades: u32,
    pub largest_position: String,
    pub most_profitable_strategy: String,
    pub positions: Vec<PortfolioPosition>,
    pub strategy_performance: HashMap<String, StrategyMetrics>,
    pub real_time_prices: HashMap<String, f64>,
    pub network: String,
    pub last_updated: DateTime<Utc>,
}

/// Strategy performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    pub name: String,
    pub total_value: f64,
    pub total_pnl: f64,
    pub return_percent: f64,
    pub trades_count: u32,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub max_drawdown: f64,
    pub allocation_percent: f64,
    pub risk_adjusted_return: f64,
}

impl ProfessionalPortfolioIntegration {
    /// Create new professional portfolio integration with full live data
    pub async fn new(config: Config) -> Result<Self> {
        info!("🏢 Initializing Professional Portfolio Integration...");
        info!("📊 Network: {}", config.network.name);

        // Initialize core portfolio components
        let portfolio_config = super::PortfolioConfiguration::default();
        let portfolio_manager = Arc::new(PortfolioManager::new(portfolio_config));        // Initialize analytics
        let analytics = Arc::new(RwLock::new(PortfolioAnalytics::new()));

        // Initialize risk managers
        let portfolio_risk_manager = Arc::new(PortfolioRiskManager::new());
        let shared_risk_manager = Arc::new(RiskManager::new(&config).await?);

        // Initialize Jupiter client for real trading
        info!("🚀 Connecting to Jupiter API...");
        let jupiter_config = crate::shared::jupiter::JupiterConfig::from_network_config(&config.network);
        let jupiter_client = Arc::new(JupiterClient::new(&jupiter_config).await?);

        // Initialize wallet manager
        info!("💼 Initializing wallet manager...");
        let wallet_manager = Arc::new(WalletManager::new(&config).await?);

        // Initialize real-time blockchain engine
        info!("⛓️ Connecting to blockchain engine...");
        let blockchain_config = crate::shared::real_time_blockchain::RealTimeBlockchainConfig::from_network_config(&config.network);
        let blockchain_engine = Arc::new(RealTimeBlockchainEngine::new(&blockchain_config).await?);

        // Initialize trading engine
        info!("⚡ Initializing trading engine...");
        let trading_engine = Arc::new(RealTradingEngine::new(&config, jupiter_client.clone()).await?);

        // Initialize WebSocket price feed
        info!("📡 Connecting to price feeds...");
        let price_feed = Arc::new(WebSocketPriceFeed::new(&config).await?);

        // Initialize real-time integration
        info!("🔄 Setting up real-time integration...");
        let real_time_integration = Arc::new(
            RealTimePortfolioIntegration::new(
                config.clone(),
                portfolio_manager.clone(),
                analytics.clone(),
                jupiter_client.clone(),
            ).await?
        );

        // Initialize real data integration
        info!("💾 Setting up real data integration...");
        let real_data_integration = Arc::new(
            PortfolioRealDataIntegration::new(
                config.clone(),
                portfolio_manager.clone(),
                real_time_integration.clone(),
            ).await?
        );

        let integration = Self {
            config,
            portfolio_manager,
            real_data_integration,
            real_time_integration,
            analytics,
            portfolio_risk_manager,
            shared_risk_manager,
            jupiter_client,
            wallet_manager,
            blockchain_engine,
            trading_engine,
            price_feed,
            live_prices: Arc::new(RwLock::new(HashMap::new())),
        };

        info!("✅ Professional Portfolio Integration initialized successfully");
        Ok(integration)
    }

    /// Get comprehensive professional portfolio status
    pub async fn get_professional_status(&self) -> Result<ProfessionalPortfolioStatus> {
        info!("📊 Gathering comprehensive portfolio status...");

        // Get real portfolio metrics
        let real_metrics = self.real_data_integration.get_real_portfolio_metrics().await?;

        // Get current positions
        let positions = self.portfolio_manager.get_all_positions().await?;

        // Get live prices for all positions
        let mut real_time_prices = HashMap::new();
        for position in &positions {
            if let Ok(price_data) = self.real_time_integration.get_live_price(&position.token_mint).await {
                real_time_prices.insert(position.symbol.clone(), price_data.price_usd);
            }
        }

        // Get strategy performance data
        let strategy_performance = self.calculate_strategy_performance(&real_metrics, &positions).await;

        // Determine most profitable strategy
        let most_profitable_strategy = strategy_performance
            .iter()
            .max_by(|a, b| a.1.return_percent.partial_cmp(&b.1.return_percent).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "None".to_string());

        // Find largest position
        let largest_position = positions
            .iter()
            .max_by(|a, b| a.value_usd.partial_cmp(&b.value_usd).unwrap_or(std::cmp::Ordering::Equal))
            .map(|p| format!("{} (${:.2})", p.symbol, p.value_usd))
            .unwrap_or_else(|| "None".to_string());

        // Get active strategies
        let active_strategies: Vec<String> = positions
            .iter()
            .map(|p| p.strategy.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        Ok(ProfessionalPortfolioStatus {
            total_value: real_metrics.total_value,
            total_invested: real_metrics.total_invested,
            total_pnl: real_metrics.total_pnl,
            total_return_percent: real_metrics.total_return_percent,
            daily_pnl: real_metrics.daily_pnl,
            daily_return_percent: real_metrics.daily_return_percent,
            positions_count: positions.len(),
            active_strategies,
            risk_score: self.calculate_portfolio_risk_score(&positions).await,
            max_drawdown: real_metrics.max_drawdown,
            sharpe_ratio: real_metrics.sharpe_ratio,
            win_rate: real_metrics.win_rate,
            profit_factor: real_metrics.profit_factor,
            total_fees_paid: real_metrics.total_fees_paid,
            total_trades: real_metrics.total_trades,
            largest_position,
            most_profitable_strategy,
            positions,
            strategy_performance,
            real_time_prices,
            network: self.config.network.name.clone(),
            last_updated: Utc::now(),
        })
    }

    /// Calculate portfolio risk score based on current positions
    async fn calculate_portfolio_risk_score(&self, positions: &[PortfolioPosition]) -> f64 {
        if positions.is_empty() {
            return 0.0;
        }

        // Calculate position concentration risk
        let total_value: f64 = positions.iter().map(|p| p.value_usd).sum();
        let max_position_pct = positions
            .iter()
            .map(|p| p.value_usd / total_value)
            .fold(0.0, f64::max);

        // Calculate correlation risk (simplified)
        let strategy_count = positions
            .iter()
            .map(|p| &p.strategy)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let diversification_factor = if strategy_count > 1 {
            1.0 / (strategy_count as f64).sqrt()
        } else {
            1.0
        };

        // Calculate volatility risk based on unrealized P&L
        let unrealized_pnl_volatility = if positions.len() > 1 {
            let mean_pnl: f64 = positions.iter().map(|p| p.unrealized_pnl).sum::<f64>() / positions.len() as f64;
            let variance: f64 = positions
                .iter()
                .map(|p| (p.unrealized_pnl - mean_pnl).powi(2))
                .sum::<f64>() / positions.len() as f64;
            variance.sqrt() / total_value
        } else {
            0.1
        };

        // Combine risk factors (0.0 = no risk, 1.0 = maximum risk)
        let concentration_risk = max_position_pct;
        let correlation_risk = diversification_factor * 0.5;
        let volatility_risk = unrealized_pnl_volatility.min(1.0);

        ((concentration_risk + correlation_risk + volatility_risk) / 3.0).min(1.0)
    }

    /// Calculate strategy-specific performance metrics
    async fn calculate_strategy_performance(
        &self,
        real_metrics: &RealPortfolioMetrics,
        positions: &[PortfolioPosition],
    ) -> HashMap<String, StrategyMetrics> {
        let mut strategy_metrics = HashMap::new();

        // Get total portfolio value for allocation calculations
        let total_value: f64 = positions.iter().map(|p| p.value_usd).sum();

        // Group positions by strategy
        let mut strategy_positions: HashMap<String, Vec<&PortfolioPosition>> = HashMap::new();
        for position in positions {
            strategy_positions
                .entry(position.strategy.clone())
                .or_insert_with(Vec::new)
                .push(position);
        }

        // Calculate metrics for each strategy
        for (strategy_name, positions) in strategy_positions {
            let strategy_value: f64 = positions.iter().map(|p| p.value_usd).sum();
            let strategy_pnl: f64 = positions.iter().map(|p| p.unrealized_pnl + p.realized_pnl).sum();
            let allocation_percent = if total_value > 0.0 {
                (strategy_value / total_value) * 100.0
            } else {
                0.0
            };

            let return_percent = if strategy_value > 0.0 {
                (strategy_pnl / (strategy_value - strategy_pnl)) * 100.0
            } else {
                0.0
            };

            // Get strategy-specific metrics from real metrics if available
            let strategy_real_metrics = real_metrics
                .strategy_performance
                .get(&strategy_name)
                .cloned()
                .unwrap_or_else(|| super::real_data_integration::StrategyMetrics {
                    total_value: strategy_value,
                    total_pnl: strategy_pnl,
                    return_percent,
                    trades_count: 0,
                    win_rate: 0.0,
                    profit_factor: 1.0,
                    max_drawdown: 0.0,
                    allocation_percent,
                });

            strategy_metrics.insert(
                strategy_name.clone(),
                StrategyMetrics {
                    name: strategy_name,
                    total_value: strategy_real_metrics.total_value,
                    total_pnl: strategy_real_metrics.total_pnl,
                    return_percent: strategy_real_metrics.return_percent,
                    trades_count: strategy_real_metrics.trades_count,
                    win_rate: strategy_real_metrics.win_rate,
                    profit_factor: strategy_real_metrics.profit_factor,
                    max_drawdown: strategy_real_metrics.max_drawdown,
                    allocation_percent: strategy_real_metrics.allocation_percent,
                    risk_adjusted_return: if strategy_real_metrics.max_drawdown > 0.0 {
                        strategy_real_metrics.return_percent / strategy_real_metrics.max_drawdown
                    } else {
                        strategy_real_metrics.return_percent
                    },
                },
            );
        }

        strategy_metrics
    }

    /// Start real-time monitoring and updates
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("🔄 Starting professional portfolio monitoring...");

        // Start price feed monitoring
        self.price_feed.start().await?;

        // Start blockchain monitoring
        self.blockchain_engine.start().await?;

        // Start real-time integration updates
        self.real_time_integration.start_live_updates().await?;

        info!("✅ Professional portfolio monitoring started");
        Ok(())
    }

    /// Stop all monitoring and cleanup
    pub async fn stop_monitoring(&self) -> Result<()> {
        info!("🛑 Stopping professional portfolio monitoring...");

        // Stop various monitoring services
        // Note: Individual stop methods would need to be implemented in each service
        warn!("⚠️ Graceful shutdown not fully implemented - monitors may continue running");

        info!("✅ Professional portfolio monitoring stop requested");
        Ok(())
    }
}

/// Convenience function to run professional portfolio system with full integration
pub async fn run_professional_portfolio(config: Config) -> Result<()> {
    info!("🏢 Starting Professional Portfolio System...");
    info!("📊 Network: {}", config.network.name);

    let professional_integration = ProfessionalPortfolioIntegration::new(config).await?;

    // Start monitoring systems
    info!("🔄 Starting real-time monitoring...");
    if let Err(e) = professional_integration.start_monitoring().await {
        warn!("⚠️ Some monitoring services failed to start: {}", e);
        info!("📊 Continuing with available data sources...");
    }

    // Get comprehensive portfolio status
    let status = professional_integration.get_professional_status().await?;

    // Display professional portfolio dashboard
    display_professional_dashboard(&status).await;

    // Optional: Keep monitoring running for demo
    info!("🔄 Professional portfolio system running...");
    info!("💡 Press Ctrl+C to stop monitoring");

    // In a real application, this would run indefinitely or until stopped
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Cleanup
    if let Err(e) = professional_integration.stop_monitoring().await {
        warn!("⚠️ Error during monitoring cleanup: {}", e);
    }

    info!("✅ Professional portfolio system completed!");
    Ok(())
}

/// Display comprehensive professional portfolio dashboard
async fn display_professional_dashboard(status: &ProfessionalPortfolioStatus) {
    println!("\n🏢 PROFESSIONAL PORTFOLIO DASHBOARD");
    println!("═══════════════════════════════════════════════════════════");
    println!("🌐 Network: {}", status.network);
    println!("⏰ Last Updated: {}", status.last_updated.format("%Y-%m-%d %H:%M:%S UTC"));

    // Portfolio Overview
    println!("\n📊 PORTFOLIO OVERVIEW");
    println!("─────────────────────────────");
    println!("💰 Total Value: ${:.2}", status.total_value);
    println!("💵 Total Invested: ${:.2}", status.total_invested);
    println!(
        "📈 Total P&L: ${:.2} ({:.2}%)",
        status.total_pnl, status.total_return_percent
    );
    println!(
        "📅 Daily P&L: ${:.2} ({:.2}%)",
        status.daily_pnl, status.daily_return_percent
    );
    println!("🎯 Active Positions: {}", status.positions_count);
    println!("🏷️ Active Strategies: {}", status.active_strategies.join(", "));

    // Risk Metrics
    println!("\n🛡️ RISK ANALYSIS");
    println!("─────────────────────────────");
    println!("⚠️ Risk Score: {:.1}%", status.risk_score * 100.0);
    println!("📉 Max Drawdown: {:.2}%", status.max_drawdown * 100.0);
    println!("📊 Sharpe Ratio: {:.2}", status.sharpe_ratio);
    println!("🎯 Win Rate: {:.1}%", status.win_rate * 100.0);
    println!("💹 Profit Factor: {:.2}", status.profit_factor);

    // Trading Statistics
    println!("\n📈 TRADING STATISTICS");
    println!("─────────────────────────────");
    println!("🔄 Total Trades: {}", status.total_trades);
    println!("💸 Total Fees Paid: ${:.2}", status.total_fees_paid);
    println!("🏆 Top Strategy: {}", status.most_profitable_strategy);
    println!("🎯 Largest Position: {}", status.largest_position);

    // Current Positions
    if !status.positions.is_empty() {
        println!("\n� CURRENT POSITIONS");
        println!("─────────────────────────────");
        for position in &status.positions {
            let pnl_indicator = if position.unrealized_pnl >= 0.0 { "📈" } else { "📉" };
            println!(
                "  {} {} | ${:.2} | {}{:.2} ({:.1}%) | {}",
                pnl_indicator,
                position.symbol,
                position.value_usd,
                if position.unrealized_pnl >= 0.0 { "+" } else { "" },
                position.unrealized_pnl,
                (position.unrealized_pnl / (position.value_usd - position.unrealized_pnl)) * 100.0,
                position.strategy
            );
        }
    }

    // Strategy Performance
    if !status.strategy_performance.is_empty() {
        println!("\n🎯 STRATEGY PERFORMANCE");
        println!("─────────────────────────────");
        for (_, strategy) in &status.strategy_performance {
            let perf_indicator = if strategy.return_percent >= 0.0 { "📈" } else { "📉" };
            println!(
                "  {} {} | {:.1}% allocation | {}{:.2}% return | {:.2} risk-adj",
                perf_indicator,
                strategy.name,
                strategy.allocation_percent,
                if strategy.return_percent >= 0.0 { "+" } else { "" },
                strategy.return_percent,
                strategy.risk_adjusted_return
            );
        }
    }

    // Real-time Prices
    if !status.real_time_prices.is_empty() {
        println!("\n📡 LIVE MARKET PRICES");
        println!("─────────────────────────────");
        for (symbol, price) in &status.real_time_prices {
            println!("  💱 {}: ${:.4}", symbol, price);
        }
    }

    // Professional Insights
    println!("\n💡 PROFESSIONAL INSIGHTS");
    println!("─────────────────────────────");

    if status.risk_score > 0.7 {
        println!("  ⚠️ Portfolio risk is elevated - consider position reduction");
    } else if status.risk_score < 0.3 {
        println!("  ✅ Portfolio risk is well-managed");
    } else {
        println!("  📊 Portfolio risk is moderate - monitor closely");
    }

    if status.total_return_percent > 5.0 {
        println!("  🎉 Strong portfolio performance - consider profit taking");
    } else if status.total_return_percent < -5.0 {
        println!("  📉 Portfolio underperforming - review strategy allocation");
    }

    if status.positions_count < 3 {
        println!("  🎯 Consider diversification across more positions");
    } else if status.positions_count > 10 {
        println!("  📋 Portfolio may be over-diversified - consider consolidation");
    }

    if status.win_rate > 0.6 {
        println!("  🏆 Excellent win rate - strategy execution is strong");
    } else if status.win_rate < 0.4 {
        println!("  🔍 Review trading strategy - win rate below optimal");
    }

    println!("\n═══════════════════════════════════════════════════════════");
}
