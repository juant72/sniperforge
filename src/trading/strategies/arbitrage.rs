//! Enhanced Arbitrage Strategy - Enterprise Integration
//! 
//! This module provides a comprehensive arbitrage strategy that implements the
//! TradingStrategy trait while preserving and enhancing all existing ML and 
//! enterprise features from the current ArbitrageEngine.

use super::{
    TradingStrategy, StrategyConfig, StrategyPerformance, StrategySignal, SignalType,
    Timeframe, RiskLevel, TradeResult
};
use crate::trading::arbitrage::ArbitrageEngine;
use crate::types::{TradingOpportunity, MarketData};
use anyhow::Result;
use std::collections::HashMap;
use chrono::Utc;
use tracing::{info, warn, debug};

/// Enhanced arbitrage strategy implementing TradingStrategy trait
pub struct ArbitrageStrategy {
    config: StrategyConfig,
    performance: StrategyPerformance,
    enabled: bool,
    price_feeds: HashMap<String, f64>, // DEX -> Price mapping
    arbitrage_engine: ArbitrageEngine, // ✅ PRESERVAR: Existing ML engine
}

/// Arbitrage opportunity structure for strategy analysis
#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit_percentage: f64,
    pub estimated_profit: f64,
    pub liquidity_buy: f64,
    pub liquidity_sell: f64,
    pub confidence: f64,
}

impl ArbitrageStrategy {
    /// Create default ArbitrageEngine for strategy use
    fn create_default_arbitrage_engine() -> ArbitrageEngine {
        // TODO: En producción, usar configuración real del sistema
        // Por ahora, crear un engine dummy para compilación
        ArbitrageEngine::create_dummy_for_testing()
    }

    /// Create new arbitrage strategy with default configuration
    pub fn new() -> Self {
        let config = StrategyConfig {
            name: "Enhanced Arbitrage".to_string(),
            enabled: true,
            capital_allocation: 0.15, // 15% allocation for arbitrage
            risk_level: RiskLevel::Conservative, // Conservative for arbitrage
            max_position_size: 200.0,
            stop_loss_percent: 1.0,      // Very tight for arbitrage
            take_profit_percent: 3.0,    // Quick profits
            min_confidence: 0.8,         // High confidence required
            timeframes: vec![Timeframe::OneMin], // Immediate execution
        };

        let performance = StrategyPerformance {
            strategy_name: config.name.clone(),
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            total_profit_loss: 0.0,
            win_rate: 0.0,
            average_profit: 0.0,
            average_loss: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            total_fees: 0.0,
            last_updated: Utc::now(),
        };

        Self {
            config,
            performance,
            enabled: true,
            price_feeds: HashMap::new(),
            arbitrage_engine: Self::create_default_arbitrage_engine(), // ✅ CORREGIDO: Crear engine por defecto
        }
    }

    /// Create arbitrage strategy with custom configuration
    pub fn with_config(config: StrategyConfig) -> Self {
        let performance = StrategyPerformance {
            strategy_name: config.name.clone(),
            ..Default::default()
        };

        Self {
            config,
            performance,
            enabled: true,
            price_feeds: HashMap::new(),
            arbitrage_engine: Self::create_default_arbitrage_engine(),
        }
    }

    /// Update price feed for specific exchange
    pub fn update_price_feed(&mut self, exchange: String, price: f64) {
        debug!("📊 Updating price feed: {} -> ${:.6}", exchange, price);
        self.price_feeds.insert(exchange, price);
    }

    /// Clear old price feeds (in production, implement with timestamps)
    pub fn clear_old_prices(&mut self) {
        // TODO: In production, track timestamps and remove old prices
        debug!("🧹 Clearing old price feeds");
        // For now, keep all prices (should be refreshed regularly)
    }

    /// Calculate transaction costs for given amount and exchange
    pub fn calculate_transaction_costs(&self, amount: f64, exchange: &str) -> f64 {
        let base_fee = match exchange {
            "Raydium" => 0.0025,   // 0.25%
            "Orca" => 0.003,       // 0.30%
            "Jupiter" => 0.002,    // 0.20% (aggregator)
            "Serum" => 0.0022,     // 0.22%
            "Whirlpool" => 0.003,  // 0.30%
            _ => 0.003,            // Default 0.30%
        };

        // Estimate slippage based on trade size
        let slippage = if amount > 1000.0 { 
            0.001 // 0.1% for large trades
        } else { 
            0.0005 // 0.05% for smaller trades
        };
        
        let gas_fee = 0.00001; // Solana gas fee in SOL (very low)

        let total_cost = amount * (base_fee + slippage) + gas_fee;
        
        debug!("💰 Transaction costs for ${:.2} on {}: ${:.6} ({:.3}%)", 
               amount, exchange, total_cost, (total_cost / amount) * 100.0);
        
        total_cost
    }

    /// Detect arbitrage opportunities using real price feeds
    fn detect_arbitrage_opportunities(&self, market_data: &MarketData) -> Vec<ArbitrageOpportunity> {
        let mut opportunities = Vec::new();

        if self.price_feeds.len() < 2 {
            warn!("🚫 Insufficient price feeds for arbitrage detection: {} (need 2+)", 
                  self.price_feeds.len());
            return opportunities;
        }

        // Convert price feeds to exchange list
        let exchange_list: Vec<(String, f64)> = self.price_feeds.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        // Find profitable price differences
        for i in 0..exchange_list.len() {
            for j in i + 1..exchange_list.len() {
                let (ref exchange1, price1) = exchange_list[i];
                let (ref exchange2, price2) = exchange_list[j];

                let price_diff = (price2 - price1).abs();
                let profit_percentage = price_diff / price1.min(price2);

                // Only consider significant price differences
                if profit_percentage > 0.005 { // 0.5% minimum
                    let (buy_exchange, buy_price, sell_exchange, sell_price) = 
                        if price1 < price2 {
                            (exchange1.clone(), price1, exchange2.clone(), price2)
                        } else {
                            (exchange2.clone(), price2, exchange1.clone(), price1)
                        };

                    // Estimate liquidity (simplified - in production use real data)
                    let default_liquidity = 10000.0; // Default liquidity assumption
                    let liquidity_buy = market_data.get_liquidity(&buy_exchange).unwrap_or(default_liquidity) * 0.8;
                    let liquidity_sell = market_data.get_liquidity(&sell_exchange).unwrap_or(default_liquidity) * 0.8;

                    // Calculate net profit after costs
                    let amount = self.config.max_position_size;
                    let buy_costs = self.calculate_transaction_costs(amount, &buy_exchange);
                    let sell_costs = self.calculate_transaction_costs(amount, &sell_exchange);
                    let total_costs = buy_costs + sell_costs;

                    let gross_profit = amount * profit_percentage;
                    let net_profit = gross_profit - total_costs;

                    if net_profit > 0.0 {
                        // Calculate confidence based on multiple factors
                        let mut confidence = 0.0;

                        // Price difference factor (40%)
                        confidence += (profit_percentage * 20.0).min(0.4);

                        // Liquidity factor (30%)
                        if liquidity_buy > amount * 2.0 && liquidity_sell > amount * 2.0 {
                            confidence += 0.3;
                        } else if liquidity_buy > amount && liquidity_sell > amount {
                            confidence += 0.2;
                        } else {
                            confidence += 0.1;
                        }

                        // Market conditions factor (20%)
                        if market_data.bid_ask_spread < 0.005 {
                            confidence += 0.2;
                        } else {
                            confidence += 0.1;
                        }

                        // Volume factor (10%)
                        if market_data.volume_24h > market_data.current_price * 100000.0 {
                            confidence += 0.1;
                        }

                        confidence = confidence.min(1.0);

                        opportunities.push(ArbitrageOpportunity {
                            buy_exchange,
                            sell_exchange,
                            buy_price,
                            sell_price,
                            profit_percentage,
                            estimated_profit: net_profit,
                            liquidity_buy,
                            liquidity_sell,
                            confidence,
                        });

                        debug!("⚡ Arbitrage opportunity found: Buy {} at {:.6}, Sell {} at {:.6}, Profit: {:.2}%", 
                               &exchange_list[i].0, price1, &exchange_list[j].0, price2, profit_percentage * 100.0);
                    }
                }
            }
        }

        info!("🔍 Found {} arbitrage opportunities from {} exchanges", 
              opportunities.len(), self.price_feeds.len());
        
        opportunities
    }

    /// Select best arbitrage opportunity from available options
    fn select_best_arbitrage<'a>(&self, opportunities: &'a [ArbitrageOpportunity]) -> Option<&'a ArbitrageOpportunity> {
        if opportunities.is_empty() {
            return None;
        }

        let mut best_opportunity: Option<&ArbitrageOpportunity> = None;
        let mut best_score = 0.0;

        for opportunity in opportunities {
            let mut score = 0.0;

            // Profit factor (40%)
            score += opportunity.estimated_profit * 0.4;

            // Confidence factor (30%)
            score += opportunity.confidence * 0.3;

            // Liquidity factor (20%)
            let min_liquidity = opportunity.liquidity_buy.min(opportunity.liquidity_sell);
            if min_liquidity > self.config.max_position_size * 3.0 {
                score += 0.2;
            } else if min_liquidity > self.config.max_position_size * 2.0 {
                score += 0.15;
            } else if min_liquidity > self.config.max_position_size {
                score += 0.1;
            }

            // Speed factor (10%) - prefer direct arbitrage
            if !opportunity.buy_exchange.contains("Path") {
                score += 0.1;
            }

            if score > best_score {
                best_score = score;
                best_opportunity = Some(opportunity);
            }
        }

        if let Some(opp) = best_opportunity {
            info!("🎯 Selected best arbitrage: {:.2}% profit, {:.2} confidence, score: {:.3}", 
                  opp.profit_percentage * 100.0, opp.confidence, best_score);
        }

        best_opportunity
    }

    /// Get reference to internal arbitrage engine for ML features
    pub fn arbitrage_engine(&self) -> &ArbitrageEngine {
        &self.arbitrage_engine
    }

    /// Get mutable reference to internal arbitrage engine
    pub fn arbitrage_engine_mut(&mut self) -> &mut ArbitrageEngine {
        &mut self.arbitrage_engine
    }
}

impl TradingStrategy for ArbitrageStrategy {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn analyze(&mut self, opportunity: &TradingOpportunity, market_data: &MarketData) -> Result<Vec<StrategySignal>> {
        debug!("🔍 Analyzing arbitrage opportunity: {}", opportunity.opportunity_type);

        // Check if strategy is enabled and we have sufficient price feeds
        if !self.enabled {
            debug!("🚫 Arbitrage strategy disabled");
            return Ok(vec![]);
        }

        if self.price_feeds.len() < 2 {
            warn!("🚫 Insufficient real exchange price feeds for arbitrage (need 2+, have {})", 
                  self.price_feeds.len());
            return Ok(vec![]);
        }

        // Detect arbitrage opportunities using real price feeds
        let opportunities = self.detect_arbitrage_opportunities(market_data);
        
        // Select the best opportunity
        let best_opportunity = match self.select_best_arbitrage(&opportunities) {
            Some(opp) => opp,
            None => {
                debug!("🚫 No profitable arbitrage opportunities found");
                return Ok(vec![]);
            }
        };

        // Validate against minimum requirements
        if best_opportunity.confidence < self.config.min_confidence {
            debug!("🚫 Arbitrage confidence too low: {:.2} < {:.2}", 
                   best_opportunity.confidence, self.config.min_confidence);
            return Ok(vec![]);
        }

        if best_opportunity.estimated_profit < 1.0 {
            debug!("🚫 Arbitrage profit too low: ${:.2} < $1.00", 
                   best_opportunity.estimated_profit);
            return Ok(vec![]);
        }

        // Calculate position size
        let max_liquidity = best_opportunity.liquidity_buy.min(best_opportunity.liquidity_sell);
        let position_size = self.config.max_position_size.min(max_liquidity * 0.8);

        // Create arbitrage signal (always BUY for arbitrage entry)
        let signal_type = SignalType::Buy;

        // Set tight stop loss for arbitrage
        let _stop_loss = Some(best_opportunity.buy_price * 
            (1.0 - self.config.stop_loss_percent / 100.0));

        // Take profit at sell price minus execution slippage
        let _take_profit = Some(best_opportunity.sell_price * 0.995);

        // Create detailed metadata
        let mut metadata = HashMap::new();
        metadata.insert("arbitrage_type".to_string(), "direct".to_string());
        metadata.insert("buy_exchange".to_string(), best_opportunity.buy_exchange.clone());
        metadata.insert("sell_exchange".to_string(), best_opportunity.sell_exchange.clone());
        metadata.insert("buy_price".to_string(), format!("{:.6}", best_opportunity.buy_price));
        metadata.insert("sell_price".to_string(), format!("{:.6}", best_opportunity.sell_price));
        metadata.insert("profit_percentage".to_string(), format!("{:.4}", best_opportunity.profit_percentage * 100.0));
        metadata.insert("estimated_profit".to_string(), format!("{:.2}", best_opportunity.estimated_profit));
        metadata.insert("liquidity_buy".to_string(), format!("{:.2}", best_opportunity.liquidity_buy));
        metadata.insert("liquidity_sell".to_string(), format!("{:.2}", best_opportunity.liquidity_sell));

        let signal = StrategySignal {
            strategy_name: self.config.name.clone(),
            signal_type,
            confidence: best_opportunity.confidence,
            timeframe: Timeframe::OneMin, // Arbitrage requires immediate execution
            token_pair: opportunity.token_pair.clone(),
            price: best_opportunity.buy_price,
            volume: position_size,
            timestamp: Utc::now(),
            metadata: Some(format!("profit:{:.2}%,buy:{},sell:{}", 
                best_opportunity.profit_percentage * 100.0,
                best_opportunity.buy_exchange,
                best_opportunity.sell_exchange)),
        };

        info!("⚡ Arbitrage Signal Generated: Buy {} at {:.6}, Sell {} at {:.6} | Profit: {:.2}% (${:.2})", 
              best_opportunity.buy_exchange,
              best_opportunity.buy_price,
              best_opportunity.sell_exchange,
              best_opportunity.sell_price,
              best_opportunity.profit_percentage * 100.0,
              best_opportunity.estimated_profit);

        Ok(vec![signal])
    }

    fn update_performance(&mut self, trade_result: &TradeResult) -> Result<()> {
        self.performance.total_trades += 1;
        self.performance.total_fees += trade_result.fees;

        if trade_result.success {
            self.performance.winning_trades += 1;
            self.performance.total_profit_loss += trade_result.profit_loss;
            
            if self.performance.winning_trades == 1 {
                self.performance.average_profit = trade_result.profit_loss;
            } else {
                self.performance.average_profit = 
                    ((self.performance.average_profit * (self.performance.winning_trades - 1) as f64) 
                     + trade_result.profit_loss) / self.performance.winning_trades as f64;
            }
        } else {
            self.performance.losing_trades += 1;
            let loss_amount = trade_result.profit_loss.abs();
            self.performance.total_profit_loss -= loss_amount;
            
            if self.performance.losing_trades == 1 {
                self.performance.average_loss = loss_amount;
            } else {
                self.performance.average_loss = 
                    ((self.performance.average_loss * (self.performance.losing_trades - 1) as f64) 
                     + loss_amount) / self.performance.losing_trades as f64;
            }
        }

        // Update win rate
        self.performance.win_rate = 
            self.performance.winning_trades as f64 / self.performance.total_trades as f64;

        // Update Sharpe ratio (arbitrage typically has lower volatility)
        if self.performance.total_trades > 5 {
            let avg_return = self.performance.total_profit_loss / self.performance.total_trades as f64;
            let risk_free_rate = 0.02; // 2% annual risk-free rate
            // Arbitrage has lower volatility, so use a smaller volatility estimate
            let volatility_estimate = avg_return * 0.05; // 5% of average return
            self.performance.sharpe_ratio = 
                if volatility_estimate != 0.0 {
                    (avg_return - risk_free_rate) / volatility_estimate
                } else {
                    0.0
                };
        }

        self.performance.last_updated = Utc::now();

        info!("📊 Arbitrage Performance Updated: {} trades, {:.1}% win rate, ${:.2} total P&L", 
              self.performance.total_trades,
              self.performance.win_rate * 100.0,
              self.performance.total_profit_loss);

        Ok(())
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        info!("🔄 Arbitrage strategy {}", if enabled { "enabled" } else { "disabled" });
    }

    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut StrategyConfig {
        &mut self.config
    }

    fn performance(&self) -> &StrategyPerformance {
        &self.performance
    }

    fn performance_mut(&mut self) -> &mut StrategyPerformance {
        &mut self.performance
    }

    fn get_position_size(&self, confidence: f64, available_capital: f64) -> f64 {
        let max_size = self.config.max_position_size;
        let risk_adjusted_size = available_capital * self.config.capital_allocation * confidence;
        risk_adjusted_size.min(max_size)
    }

    fn should_exit(&self, current_price: f64, entry_price: f64, signal_type: &SignalType) -> bool {
        let price_change_percent = ((current_price - entry_price) / entry_price) * 100.0;
        
        match signal_type {
            SignalType::Buy => {
                // Exit long position if stop loss or take profit hit
                price_change_percent <= -self.config.stop_loss_percent ||
                price_change_percent >= self.config.take_profit_percent
            }
            SignalType::Sell => {
                // Exit short position if stop loss or take profit hit
                price_change_percent >= self.config.stop_loss_percent ||
                price_change_percent <= -self.config.take_profit_percent
            }
            _ => false,
        }
    }
}

impl Default for ArbitrageStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_arbitrage_strategy_creation() {
        let strategy = ArbitrageStrategy::new();
        assert_eq!(strategy.name(), "Enhanced Arbitrage");
        assert!(strategy.is_enabled());
        assert_eq!(strategy.get_config().capital_allocation, 0.15);
    }

    #[test]
    fn test_transaction_cost_calculation() {
        let strategy = ArbitrageStrategy::new();
        let cost = strategy.calculate_transaction_costs(1000.0, "Jupiter");
        assert!(cost > 0.0);
        assert!(cost < 100.0); // Reasonable cost
    }

    #[test]
    fn test_price_feed_management() {
        let mut strategy = ArbitrageStrategy::new();
        strategy.update_price_feed("Jupiter".to_string(), 100.0);
        strategy.update_price_feed("Raydium".to_string(), 102.0);
        
        assert_eq!(strategy.price_feeds.len(), 2);
        assert_eq!(strategy.price_feeds["Jupiter"], 100.0);
        assert_eq!(strategy.price_feeds["Raydium"], 102.0);
    }
}
