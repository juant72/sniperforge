pub mod bot_factory;
pub mod enhanced_arbitrage_bot;
pub mod triangular_arbitrage_bot;
pub mod ml_analytics_bot;
pub mod portfolio_manager_bot;
pub mod dashboard_bot;
pub mod mock_arbitrage_bot; // For testing control system
pub mod liquidity_sniper; // Capital accumulation bot

// Re-export main bot types
pub use bot_factory::{BotFactory, BotRegistry, BotTypeMetadata, ResourceRequirements};
pub use mock_arbitrage_bot::MockArbitrageBot;

// Individual bot implementations will be added as they are migrated
// pub use enhanced_arbitrage_bot::EnhancedArbitrageBot;
// pub use triangular_arbitrage_bot::TriangularArbitrageBot;
// pub use ml_analytics_bot::MLAnalyticsBot;
// pub use portfolio_manager_bot::PortfolioManagerBot;
// pub use dashboard_bot::DashboardBot;
