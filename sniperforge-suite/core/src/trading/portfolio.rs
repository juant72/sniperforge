use crate::{
    config::SimpleConfig,
    types::{ApiResult as Result, Token},
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Instant,
};
use tokio::sync::RwLock;
use tracing::{info, debug};

/// Portfolio manager for tracking positions and balances
#[derive(Clone)]
pub struct PortfolioManager {
    config: SimpleConfig,
    positions: Arc<RwLock<HashMap<String, Position>>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    last_update: Arc<RwLock<Instant>>,
}

impl PortfolioManager {
    /// Create a new portfolio manager
    pub fn new(config: SimpleConfig) -> Self {
        Self {
            config,
            positions: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }
    
    /// Update position for a token
    pub async fn update_position(&self, token: &Token, amount: f64, price: f64) -> Result<()> {
        let mut positions = self.positions.write().await;
        let position = positions.entry(token.symbol.clone())
            .or_insert_with(|| Position::new(token.clone()));
        
        position.update(amount, price);
        *self.last_update.write().await = Instant::now();
        
        debug!("Updated position for {}: {} @ ${}", token.symbol, amount, price);
        Ok(())
    }
    
    /// Get current position for a token
    pub async fn get_position(&self, symbol: &str) -> Option<Position> {
        let positions = self.positions.read().await;
        positions.get(symbol).cloned()
    }
    
    /// Get all positions
    pub async fn get_all_positions(&self) -> HashMap<String, Position> {
        self.positions.read().await.clone()
    }
    
    /// Calculate total portfolio value
    pub async fn calculate_total_value(&self, current_prices: &HashMap<String, f64>) -> f64 {
        let positions = self.positions.read().await;
        let mut total_value = 0.0;
        
        for (symbol, position) in positions.iter() {
            if let Some(current_price) = current_prices.get(symbol) {
                total_value += position.amount * current_price;
            }
        }
        
        total_value
    }
    
    /// Record a trade execution
    pub async fn record_trade(&self, trade: TradeRecord) -> Result<()> {
        let mut metrics = self.performance_metrics.write().await;
        
        metrics.total_trades += 1;
        metrics.total_volume += trade.volume;
        
        if trade.profit > 0.0 {
            metrics.profitable_trades += 1;
            metrics.total_profit += trade.profit;
        } else {
            metrics.losing_trades += 1;
            metrics.total_loss += trade.profit.abs();
        }
        
        metrics.trades.push(trade);
        
        // Keep only last 1000 trades
        if metrics.trades.len() > 1000 {
            let trades_to_remove = metrics.trades.len() - 1000;
            metrics.trades.drain(0..trades_to_remove);
        }
        
        info!("Recorded trade - Total trades: {}, Win rate: {:.2}%", 
              metrics.total_trades, 
              metrics.get_win_rate() * 100.0);
        
        Ok(())
    }
    
    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }
    
    /// Calculate portfolio risk metrics
    pub async fn calculate_risk_metrics(&self, current_prices: &HashMap<String, f64>) -> RiskMetrics {
        let positions = self.positions.read().await;
        let mut total_value = 0.0;
        let mut max_single_position: f64 = 0.0;
        let mut diversification_score = 0.0;
        
        for (symbol, position) in positions.iter() {
            if let Some(current_price) = current_prices.get(symbol) {
                let position_value = position.amount * current_price;
                total_value += position_value;
                max_single_position = max_single_position.max(position_value);
            }
        }
        
        // Calculate concentration risk
        let concentration_risk = if total_value > 0.0 {
            max_single_position / total_value
        } else {
            0.0
        };
        
        // Simple diversification score (number of positions)
        diversification_score = positions.len() as f64;
        
        RiskMetrics {
            total_value,
            concentration_risk,
            diversification_score,
            max_single_position_ratio: concentration_risk,
            position_count: positions.len(),
        }
    }
    
    /// Get portfolio summary
    pub async fn get_portfolio_summary(&self, current_prices: &HashMap<String, f64>) -> PortfolioSummary {
        let positions = self.get_all_positions().await;
        let performance = self.get_performance_metrics().await;
        let risk_metrics = self.calculate_risk_metrics(current_prices).await;
        let total_value = self.calculate_total_value(current_prices).await;
        
        PortfolioSummary {
            total_value,
            position_count: positions.len(),
            total_trades: performance.total_trades,
            win_rate: performance.get_win_rate(),
            total_pnl: performance.get_net_pnl(),
            risk_metrics,
            last_update: *self.last_update.read().await,
        }
    }
}

/// Individual position in the portfolio
#[derive(Debug, Clone)]
pub struct Position {
    pub token: Token,
    pub amount: f64,
    pub average_price: f64,
    pub last_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub last_updated: Instant,
}

impl Position {
    pub fn new(token: Token) -> Self {
        Self {
            token,
            amount: 0.0,
            average_price: 0.0,
            last_price: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            last_updated: Instant::now(),
        }
    }
    
    pub fn update(&mut self, amount_change: f64, price: f64) {
        if amount_change > 0.0 {
            // Buying - update average price
            let total_cost = self.amount * self.average_price + amount_change * price;
            self.amount += amount_change;
            if self.amount > 0.0 {
                self.average_price = total_cost / self.amount;
            }
        } else {
            // Selling - realize PnL
            let sold_amount = amount_change.abs();
            let realized_gain = sold_amount * (price - self.average_price);
            self.realized_pnl += realized_gain;
            self.amount += amount_change; // amount_change is negative
        }
        
        self.last_price = price;
        self.update_unrealized_pnl();
        self.last_updated = Instant::now();
    }
    
    fn update_unrealized_pnl(&mut self) {
        self.unrealized_pnl = self.amount * (self.last_price - self.average_price);
    }
    
    pub fn get_total_pnl(&self) -> f64 {
        self.realized_pnl + self.unrealized_pnl
    }
    
    pub fn get_market_value(&self) -> f64 {
        self.amount * self.last_price
    }
}

/// Performance tracking metrics
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub total_trades: u64,
    pub profitable_trades: u64,
    pub losing_trades: u64,
    pub total_profit: f64,
    pub total_loss: f64,
    pub total_volume: f64,
    pub trades: Vec<TradeRecord>,
}

impl PerformanceMetrics {
    pub fn get_win_rate(&self) -> f64 {
        if self.total_trades > 0 {
            self.profitable_trades as f64 / self.total_trades as f64
        } else {
            0.0
        }
    }
    
    pub fn get_net_pnl(&self) -> f64 {
        self.total_profit - self.total_loss
    }
    
    pub fn get_average_profit(&self) -> f64 {
        if self.profitable_trades > 0 {
            self.total_profit / self.profitable_trades as f64
        } else {
            0.0
        }
    }
    
    pub fn get_average_loss(&self) -> f64 {
        if self.losing_trades > 0 {
            self.total_loss / self.losing_trades as f64
        } else {
            0.0
        }
    }
    
    pub fn get_profit_factor(&self) -> f64 {
        if self.total_loss > 0.0 {
            self.total_profit / self.total_loss
        } else if self.total_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        }
    }
}

/// Individual trade record
#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub symbol: String,
    pub side: TradeSide,
    pub amount: f64,
    pub price: f64,
    pub volume: f64,
    pub profit: f64,
    pub gas_cost: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub trade_id: String,
}

#[derive(Debug, Clone)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// Risk metrics for the portfolio
#[derive(Debug, Clone)]
pub struct RiskMetrics {
    pub total_value: f64,
    pub concentration_risk: f64,
    pub diversification_score: f64,
    pub max_single_position_ratio: f64,
    pub position_count: usize,
}

/// Portfolio summary
#[derive(Debug, Clone)]
pub struct PortfolioSummary {
    pub total_value: f64,
    pub position_count: usize,
    pub total_trades: u64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub risk_metrics: RiskMetrics,
    pub last_update: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_config() -> SimpleConfig {
        SimpleConfig {
            solana_rpc_url: "test".to_string(),
            solana_ws_url: "test".to_string(),
            max_slippage: 0.005,
            min_profit_threshold: 0.001,
            max_position_size: 0.1,
            private_key_path: "test".to_string(),
            enable_simulation: true,
            log_level: "info".to_string(),
            dexscreener_base_url: "test".to_string(),
            max_requests_per_second: 10,
            cooldown_period_ms: 100,
        }
    }
    
    fn create_test_token() -> Token {
        Token {
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            decimals: 9,
        }
    }
    
    #[tokio::test]
    async fn test_portfolio_position_update() {
        let config = create_test_config();
        let portfolio = PortfolioManager::new(config);
        let token = create_test_token();
        
        // Buy 10 SOL at $100
        portfolio.update_position(&token, 10.0, 100.0).await.unwrap();
        
        let position = portfolio.get_position("SOL").await.unwrap();
        assert_eq!(position.amount, 10.0);
        assert_eq!(position.average_price, 100.0);
    }
    
    #[tokio::test]
    async fn test_portfolio_pnl_calculation() {
        let config = create_test_config();
        let portfolio = PortfolioManager::new(config);
        let token = create_test_token();
        
        // Buy 10 SOL at $100
        portfolio.update_position(&token, 10.0, 100.0).await.unwrap();
        
        // Price goes to $110
        portfolio.update_position(&token, 0.0, 110.0).await.unwrap();
        
        let position = portfolio.get_position("SOL").await.unwrap();
        assert_eq!(position.unrealized_pnl, 100.0); // 10 SOL * ($110 - $100)
    }
    
    #[tokio::test]
    async fn test_trade_recording() {
        let config = create_test_config();
        let portfolio = PortfolioManager::new(config);
        
        let trade = TradeRecord {
            symbol: "SOL".to_string(),
            side: TradeSide::Buy,
            amount: 10.0,
            price: 100.0,
            volume: 1000.0,
            profit: 50.0,
            gas_cost: 0.001,
            timestamp: chrono::Utc::now(),
            trade_id: "trade_1".to_string(),
        };
        
        portfolio.record_trade(trade).await.unwrap();
        
        let metrics = portfolio.get_performance_metrics().await;
        assert_eq!(metrics.total_trades, 1);
        assert_eq!(metrics.profitable_trades, 1);
        assert_eq!(metrics.total_profit, 50.0);
    }
}
