use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub buy_dex: String,
    pub sell_dex: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit_amount: f64,
    pub profit_percentage: f64,
    pub confidence: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub liquidity_buy: f64,
    pub liquidity_sell: f64,
    pub estimated_profit: f64,
}

impl ArbitrageOpportunity {
    pub fn new(
        buy_dex: String,
        sell_dex: String,
        buy_price: f64,
        sell_price: f64,
        profit_amount: f64,
        profit_percentage: f64,
        confidence: f64,
        liquidity_buy: f64,
        liquidity_sell: f64,
        estimated_profit: f64,
    ) -> Self {
        Self {
            buy_dex,
            sell_dex,
            buy_price,
            sell_price,
            profit_amount,
            profit_percentage,
            confidence,
            timestamp: chrono::Utc::now(),
            liquidity_buy,
            liquidity_sell,
            estimated_profit,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageSettings {
    pub min_profit_threshold: f64,
    pub max_slippage: f64,
    pub detection_interval_ms: u64,
    pub execution_timeout_ms: u64,
    pub enabled: bool,
}

impl Default for ArbitrageSettings {
    fn default() -> Self {
        Self {
            min_profit_threshold: 0.005,
            max_slippage: 0.01,
            detection_interval_ms: 1000,
            execution_timeout_ms: 30000,
            enabled: true,
        }
    }
}
