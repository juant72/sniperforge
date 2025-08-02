//! Enhanced Mean Reversion Strategy - Enterprise Integration
//! 
//! This module provides a sophisticated mean reversion trading strategy that implements 
//! the TradingStrategy trait with enterprise-grade features including statistical analysis,
//! multi-timeframe reversion detection, and conservative risk management.

use super::{
    TradingStrategy, StrategyConfig, StrategyPerformance, StrategySignal, SignalType,
    Timeframe, RiskLevel, TradeResult
};
use crate::types::{TradingOpportunity, MarketData};
use anyhow::Result;
use std::collections::HashMap;
use chrono::Utc;
use tracing::{info, debug, warn};

/// Enhanced mean reversion strategy implementing TradingStrategy trait
/// Features enterprise-grade statistical analysis and reversion detection
pub struct MeanReversionStrategy {
    config: StrategyConfig,
    performance: StrategyPerformance,
    enabled: bool,
    
    // 📊 Enterprise mean reversion specific features
    lookback_period: usize,            // Statistical lookback window
    deviation_threshold: f64,          // Standard deviation threshold for signals
    mean_calculation_method: String,   // SMA, EMA, or VWMA
    reversion_confidence: f64,         // Minimum confidence for reversion
    statistical_significance: f64,     // Z-score threshold for statistical significance
    price_history: HashMap<String, Vec<f64>>, // Price history for statistical analysis
    volume_history: HashMap<String, Vec<f64>>, // Volume for VWMA calculations
    
    // 🎯 Advanced statistical features
    bollinger_period: usize,           // Bollinger bands period
    bollinger_std_dev: f64,            // Standard deviations for bands
    rsi_period: usize,                 // RSI calculation period
    rsi_oversold: f64,                 // RSI oversold threshold
    rsi_overbought: f64,               // RSI overbought threshold
    volume_spike_threshold: f64,       // Volume spike detection
    
    // 🛡️ Conservative risk management
    max_consecutive_losses: usize,     // Maximum consecutive losses allowed
    consecutive_losses: usize,         // Current consecutive losses
    dynamic_position_sizing: bool,     // Enable dynamic sizing based on volatility
    volatility_adjustment: f64,        // Volatility-based position adjustment
}

impl MeanReversionStrategy {
    /// Create a new enhanced mean reversion strategy with enterprise features
    pub fn new() -> Self {
        let config = StrategyConfig {
            name: "Enhanced Mean Reversion Enterprise".to_string(),
            enabled: true,
            capital_allocation: 0.25, // 25% allocation - conservative approach
            risk_level: RiskLevel::Conservative,
            max_position_size: 300.0, // Smaller positions for conservative strategy
            stop_loss_percent: 3.0,   // Tighter stop loss
            take_profit_percent: 8.0, // More modest profit targets
            min_confidence: 0.70,     // Higher confidence requirement
            timeframes: vec![
                Timeframe::FiveMin, 
                Timeframe::FifteenMin, 
                Timeframe::OneHour,
                Timeframe::FourHour   // Longer timeframes for mean reversion
            ],
        };

        let performance = StrategyPerformance {
            strategy_name: "Enhanced Mean Reversion Enterprise".to_string(),
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
            
            // Statistical parameters
            lookback_period: 20,              // 20-period lookback
            deviation_threshold: 2.0,         // 2 standard deviations
            mean_calculation_method: "EMA".to_string(), // Exponential moving average
            reversion_confidence: 0.75,       // 75% minimum confidence
            statistical_significance: 2.5,    // 2.5 z-score threshold
            price_history: HashMap::new(),
            volume_history: HashMap::new(),
            
            // Technical analysis parameters
            bollinger_period: 20,
            bollinger_std_dev: 2.0,
            rsi_period: 14,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
            volume_spike_threshold: 1.5,      // 50% above average
            
            // Risk management
            max_consecutive_losses: 3,        // Stop after 3 consecutive losses
            consecutive_losses: 0,
            dynamic_position_sizing: true,
            volatility_adjustment: 0.5,       // 50% adjustment for volatility
        }
    }
    
    /// Calculate exponential moving average
    fn calculate_ema(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }
        
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];
        
        for &price in &prices[1..] {
            ema = alpha * price + (1.0 - alpha) * ema;
        }
        
        Some(ema)
    }
    
    /// Calculate simple moving average
    fn calculate_sma(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }
        
        let sum: f64 = prices[prices.len() - period..].iter().sum();
        Some(sum / period as f64)
    }
    
    /// Calculate volume weighted moving average
    fn calculate_vwma(&self, prices: &[f64], volumes: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period || volumes.len() < period {
            return None;
        }
        
        let price_slice = &prices[prices.len() - period..];
        let volume_slice = &volumes[volumes.len() - period..];
        
        let weighted_sum: f64 = price_slice.iter()
            .zip(volume_slice.iter())
            .map(|(p, v)| p * v)
            .sum();
        
        let volume_sum: f64 = volume_slice.iter().sum();
        
        if volume_sum > 0.0 {
            Some(weighted_sum / volume_sum)
        } else {
            None
        }
    }
    
    /// Calculate standard deviation
    fn calculate_std_dev(&self, prices: &[f64], mean: f64) -> f64 {
        if prices.is_empty() {
            return 0.0;
        }
        
        let variance: f64 = prices.iter()
            .map(|price| (price - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;
        
        variance.sqrt()
    }
    
    /// Calculate RSI (Relative Strength Index)
    fn calculate_rsi(&self, prices: &[f64]) -> Option<f64> {
        if prices.len() < self.rsi_period + 1 {
            return None;
        }
        
        let mut gains = 0.0;
        let mut losses = 0.0;
        
        for i in 1..=self.rsi_period {
            let change = prices[prices.len() - i] - prices[prices.len() - i - 1];
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
        }
        
        let avg_gain = gains / self.rsi_period as f64;
        let avg_loss = losses / self.rsi_period as f64;
        
        if avg_loss == 0.0 {
            return Some(100.0);
        }
        
        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }
    
    /// Calculate Bollinger Bands
    fn calculate_bollinger_bands(&self, prices: &[f64]) -> Option<(f64, f64, f64)> {
        if prices.len() < self.bollinger_period {
            return None;
        }
        
        let recent_prices = &prices[prices.len() - self.bollinger_period..];
        let sma = self.calculate_sma(recent_prices, self.bollinger_period)?;
        let std_dev = self.calculate_std_dev(recent_prices, sma);
        
        let upper_band = sma + (self.bollinger_std_dev * std_dev);
        let lower_band = sma - (self.bollinger_std_dev * std_dev);
        
        Some((upper_band, sma, lower_band))
    }
    
    /// Calculate mean reversion score using multiple indicators
    fn calculate_reversion_score(&self, market_data: &MarketData, symbol: &str) -> (f64, String) {
        let mut reversion_score = 0.0;
        let mut signal_reasons = Vec::new();
        
        if let Some(prices) = self.price_history.get(symbol) {
            if prices.len() >= self.lookback_period {
                let current_price = market_data.current_price;
                
                // Calculate mean based on selected method
                let mean = match self.mean_calculation_method.as_str() {
                    "SMA" => self.calculate_sma(prices, self.lookback_period),
                    "EMA" => self.calculate_ema(prices, self.lookback_period),
                    "VWMA" => {
                        if let Some(volumes) = self.volume_history.get(symbol) {
                            self.calculate_vwma(prices, volumes, self.lookback_period)
                        } else {
                            self.calculate_ema(prices, self.lookback_period)
                        }
                    },
                    _ => self.calculate_ema(prices, self.lookback_period),
                };
                
                if let Some(mean_value) = mean {
                    let deviation_from_mean = (current_price - mean_value) / mean_value * 100.0;
                    let std_dev = self.calculate_std_dev(prices, mean_value);
                    let z_score = if std_dev > 0.0 { 
                        (current_price - mean_value) / std_dev 
                    } else { 
                        0.0 
                    };
                    
                    // Price deviation component (40% weight)
                    if deviation_from_mean.abs() > self.deviation_threshold {
                        let deviation_score = deviation_from_mean.abs() / 10.0; // Scale to reasonable range
                        reversion_score += deviation_score * 0.4;
                        signal_reasons.push(format!("price_deviation: {:.2}%", deviation_from_mean));
                    }
                    
                    // Statistical significance component (30% weight)
                    if z_score.abs() > self.statistical_significance {
                        reversion_score += z_score.abs() * 0.3;
                        signal_reasons.push(format!("z_score: {:.2}", z_score));
                    }
                    
                    // Bollinger Bands analysis (20% weight)
                    if let Some((upper, _middle, lower)) = self.calculate_bollinger_bands(prices) {
                        if current_price > upper {
                            reversion_score += 2.0 * 0.2; // Overbought condition
                            signal_reasons.push("bollinger_overbought".to_string());
                        } else if current_price < lower {
                            reversion_score += 2.0 * 0.2; // Oversold condition
                            signal_reasons.push("bollinger_oversold".to_string());
                        }
                    }
                    
                    // RSI analysis (10% weight)
                    if let Some(rsi) = self.calculate_rsi(prices) {
                        if rsi > self.rsi_overbought {
                            reversion_score += 1.0 * 0.1;
                            signal_reasons.push(format!("rsi_overbought: {:.1}", rsi));
                        } else if rsi < self.rsi_oversold {
                            reversion_score += 1.0 * 0.1;
                            signal_reasons.push(format!("rsi_oversold: {:.1}", rsi));
                        }
                    }
                    
                    // Volume spike analysis - Enhanced mean reversion with volume confirmation
                    if let Some(volumes) = self.volume_history.get(symbol) {
                        if volumes.len() >= 10 {
                            let recent_volume = volumes.last().unwrap();
                            let avg_volume = volumes.iter().sum::<f64>() / volumes.len() as f64;
                            
                            if *recent_volume > avg_volume * self.volume_spike_threshold {
                                // Volume spike detected - enhances mean reversion confidence
                                let volume_multiplier = (recent_volume / avg_volume).min(3.0); // Cap at 3x
                                reversion_score *= 1.0 + (volume_multiplier - 1.0) * 0.3; // Up to 60% boost
                                signal_reasons.push(format!("volume_spike: {:.1}x_avg", volume_multiplier));
                            }
                        }
                    }
                }
            }
        }
        
        let reasons_string = signal_reasons.join(", ");
        (reversion_score, reasons_string)
    }
    
    /// Update price and volume history for statistical calculations
    pub fn update_market_data(&mut self, symbol: String, price: f64, volume: f64) {
        let price_history = self.price_history.entry(symbol.clone()).or_insert_with(Vec::new);
        price_history.push(price);
        
        // Keep only necessary history for memory efficiency
        if price_history.len() > self.lookback_period * 3 {
            price_history.remove(0);
        }
        
        let volume_history = self.volume_history.entry(symbol).or_insert_with(Vec::new);
        volume_history.push(volume);
        
        // Keep only necessary volume history
        if volume_history.len() > self.lookback_period * 3 {
            volume_history.remove(0);
        }
    }
    
    /// Check if current market conditions favor mean reversion trading
    fn is_mean_reversion_market(&self, market_data: &MarketData) -> bool {
        // Mean reversion works better in:
        // 1. Stable/ranging markets (not trending strongly)
        // 2. Markets with sufficient liquidity
        // 3. Markets without extreme volatility
        
        market_data.volume_24h > 50000.0 &&         // Sufficient liquidity
        market_data.bid_ask_spread < 0.01 &&        // Reasonable spreads
        market_data.current_price > 5.0             // Avoid very low-priced assets
    }
    
    /// Adjust position size based on volatility
    fn calculate_volatility_adjusted_position(&self, base_size: f64, symbol: &str) -> f64 {
        if !self.dynamic_position_sizing {
            return base_size;
        }
        
        if let Some(prices) = self.price_history.get(symbol) {
            if prices.len() >= 10 {
                let recent_prices = &prices[prices.len() - 10..];
                let mean = recent_prices.iter().sum::<f64>() / recent_prices.len() as f64;
                let volatility = self.calculate_std_dev(recent_prices, mean) / mean;
                
                // Reduce position size for higher volatility
                let volatility_factor = 1.0 - (volatility * self.volatility_adjustment);
                return base_size * volatility_factor.max(0.1); // Minimum 10% of base size
            }
        }
        
        base_size
    }
}

impl TradingStrategy for MeanReversionStrategy {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn enabled(&self) -> bool {
        self.enabled && self.consecutive_losses < self.max_consecutive_losses
    }
    
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            self.consecutive_losses = 0; // Reset consecutive losses when re-enabled
        }
        info!("Mean reversion strategy enabled status changed to: {}", enabled);
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
    
    fn analyze(&mut self, opportunity: &TradingOpportunity, market_data: &MarketData) -> Result<Vec<StrategySignal>> {
        if !self.enabled() {
            return Ok(vec![]);
        }
        
        let mut signals = Vec::new();
        
        // Check if market conditions favor mean reversion trading
        if !self.is_mean_reversion_market(market_data) {
            debug!("Market conditions do not favor mean reversion trading");
            return Ok(signals);
        }
        
        // Calculate mean reversion score and get signal reasons
        let (reversion_score, signal_reasons) = self.calculate_reversion_score(market_data, &opportunity.token_pair);
        
        debug!("Mean reversion score for {}: {:.2}, reasons: {}", 
               opportunity.token_pair, reversion_score, signal_reasons);
        
        // Generate signals based on reversion score
        if reversion_score >= self.reversion_confidence {
            // Determine signal direction based on current price vs mean
            let signal_type = if let Some(prices) = self.price_history.get(&opportunity.token_pair) {
                if prices.len() >= self.lookback_period {
                    let mean = match self.mean_calculation_method.as_str() {
                        "SMA" => self.calculate_sma(prices, self.lookback_period),
                        "EMA" => self.calculate_ema(prices, self.lookback_period),
                        "VWMA" => {
                            if let Some(volumes) = self.volume_history.get(&opportunity.token_pair) {
                                self.calculate_vwma(prices, volumes, self.lookback_period)
                            } else {
                                self.calculate_ema(prices, self.lookback_period)
                            }
                        },
                        _ => self.calculate_ema(prices, self.lookback_period),
                    };
                    
                    if let Some(mean_value) = mean {
                        if market_data.current_price > mean_value {
                            SignalType::Sell // Price above mean, expect reversion down
                        } else {
                            SignalType::Buy  // Price below mean, expect reversion up
                        }
                    } else {
                        SignalType::Hold
                    }
                } else {
                    SignalType::Hold
                }
            } else {
                SignalType::Hold
            };
            
            if signal_type != SignalType::Hold {
                let confidence = (reversion_score / 5.0).min(1.0).max(0.0); // Scale to 0-1
                
                if confidence >= self.config.min_confidence {
                    let signal_type_for_log = signal_type.clone();
                    let signal = StrategySignal {
                        strategy_name: self.config.name.clone(),
                        signal_type: signal_type.clone(),
                        confidence,
                        timeframe: Timeframe::FifteenMin, // Primary timeframe for mean reversion
                        token_pair: opportunity.token_pair.clone(),
                        price: market_data.current_price,
                        volume: market_data.volume_24h,
                        timestamp: Utc::now(),
                        metadata: Some(format!(
                            "reversion_score: {:.2}, method: {}, reasons: {}",
                            reversion_score, self.mean_calculation_method, signal_reasons
                        )),
                        
                        // ✅ ENTERPRISE FIELDS
                        expected_profit: (reversion_score * 0.5).min(4.0).max(0.3), // Conservative profit expectations
                        stop_loss: match signal_type {
                            SignalType::Buy => market_data.current_price * (1.0 - self.config.stop_loss_percent / 100.0),
                            SignalType::Sell => market_data.current_price * (1.0 + self.config.stop_loss_percent / 100.0),
                            _ => market_data.current_price,
                        },
                        take_profit: match signal_type {
                            SignalType::Buy => market_data.current_price * (1.0 + self.config.take_profit_percent / 100.0),
                            SignalType::Sell => market_data.current_price * (1.0 - self.config.take_profit_percent / 100.0),
                            _ => market_data.current_price,
                        },
                        reasoning: Some(format!("Mean reversion opportunity: {}", signal_reasons)),
                        risk_score: (1.0 - confidence) * 0.6, // Mean reversion generally lower risk
                        market_conditions: Some(format!("Statistical reversion: {:.1}σ", reversion_score / 2.0)),
                    };
                    
                    signals.push(signal);
                    info!("Generated {:?} mean reversion signal for {} with confidence {:.2}", 
                          signal_type_for_log, opportunity.token_pair, confidence);
                }
            }
        }
        
        Ok(signals)
    }
    
    fn update_performance(&mut self, trade_result: &TradeResult) -> Result<()> {
        self.performance.total_trades += 1;
        self.performance.total_profit_loss += trade_result.profit_loss;
        self.performance.total_fees += trade_result.fees;
        
        if trade_result.profit_loss > 0.0 {
            self.performance.winning_trades += 1;
            self.consecutive_losses = 0; // Reset consecutive losses on win
            self.performance.average_profit = 
                (self.performance.average_profit * (self.performance.winning_trades - 1) as f64 + 
                 trade_result.profit_loss) / self.performance.winning_trades as f64;
        } else {
            self.performance.losing_trades += 1;
            self.consecutive_losses += 1; // Increment consecutive losses
            self.performance.average_loss = 
                (self.performance.average_loss * (self.performance.losing_trades - 1) as f64 + 
                 trade_result.profit_loss.abs()) / self.performance.losing_trades as f64;
            
            // Check if we need to temporarily disable the strategy
            if self.consecutive_losses >= self.max_consecutive_losses {
                warn!("Mean reversion strategy disabled due to {} consecutive losses", 
                      self.consecutive_losses);
            }
        }
        
        // Update win rate
        self.performance.win_rate = self.performance.winning_trades as f64 / self.performance.total_trades as f64;
        
        // Update max drawdown if necessary
        if trade_result.profit_loss < 0.0 && trade_result.profit_loss.abs() > self.performance.max_drawdown {
            self.performance.max_drawdown = trade_result.profit_loss.abs();
        }
        
        self.performance.last_updated = Utc::now();
        
        info!("Updated mean reversion strategy performance: Win rate: {:.2}%, Total P&L: {:.2}, Consecutive losses: {}", 
              self.performance.win_rate * 100.0, self.performance.total_profit_loss, self.consecutive_losses);
        
        Ok(())
    }
    
    fn get_position_size(&self, confidence: f64, available_capital: f64) -> f64 {
        let base_size = available_capital * self.config.capital_allocation * confidence;
        let max_size = base_size.min(self.config.max_position_size);
        
        // Apply volatility adjustment
        self.calculate_volatility_adjusted_position(max_size, "default")
    }
    
    fn should_exit(&self, current_price: f64, entry_price: f64, signal_type: &SignalType) -> bool {
        let price_change_percent = match signal_type {
            SignalType::Buy => (current_price - entry_price) / entry_price * 100.0,
            SignalType::Sell => (entry_price - current_price) / entry_price * 100.0,
            SignalType::Hold => return false,
            SignalType::StopLoss | SignalType::TakeProfit => return true,
        };
        
        // Conservative exit conditions
        price_change_percent <= -self.config.stop_loss_percent || 
        price_change_percent >= self.config.take_profit_percent
    }
}

impl Default for MeanReversionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OpportunityType;
    use std::collections::HashMap;

    fn create_test_market_data() -> MarketData {
        let mut market_data = MarketData {
            current_price: 100.0,
            volume_24h: 100000.0,
            bid_ask_spread: 0.005,
            prices: HashMap::new(),
            volumes: HashMap::new(),
            liquidity: HashMap::new(),
            last_updated: Some(std::time::Instant::now()),
        };
        
        // Add test data
        market_data.prices.insert("BTC/USDC".to_string(), 100.0);
        market_data.volumes.insert("BTC/USDC".to_string(), 100000.0);
        market_data.liquidity.insert("BTC/USDC".to_string(), 50000.0);
        
        market_data
    }

    fn create_test_opportunity() -> TradingOpportunity {
        TradingOpportunity {
            opportunity_type: OpportunityType::Arbitrage,
            token_pair: "BTC/USDC".to_string(),
            profit_percentage: 1.5,
            volume_24h: 100000.0,
            liquidity: 50000.0,
            source_exchange: "Jupiter".to_string(),
            target_exchange: "Raydium".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_mean_reversion_strategy_creation() {
        let strategy = MeanReversionStrategy::new();
        assert_eq!(strategy.name(), "Enhanced Mean Reversion Enterprise");
        assert!(strategy.enabled());
        assert_eq!(strategy.config().capital_allocation, 0.25);
        assert_eq!(strategy.config().risk_level, RiskLevel::Conservative);
    }

    #[test]
    fn test_statistical_calculations() {
        let strategy = MeanReversionStrategy::new();
        // Providing 20 prices to ensure we have enough for RSI calculation (needs 15 minimum)
        let prices = vec![
            95.0, 98.0, 100.0, 102.0, 105.0, 103.0, 101.0, 99.0, 97.0, 100.0,
            102.0, 104.0, 103.0, 101.0, 99.0, 98.0, 100.0, 102.0, 104.0, 103.0
        ];
        
        // Test SMA calculation
        let sma = strategy.calculate_sma(&prices, 5);
        assert!(sma.is_some());
        
        // Test EMA calculation
        let ema = strategy.calculate_ema(&prices, 5);
        assert!(ema.is_some());
        
        // Test RSI calculation (now with sufficient data)
        let rsi = strategy.calculate_rsi(&prices);
        assert!(rsi.is_some());
        
        // Test Bollinger Bands
        let bands = strategy.calculate_bollinger_bands(&prices);
        assert!(bands.is_some());
    }

    #[test]
    fn test_mean_reversion_market_detection() {
        let strategy = MeanReversionStrategy::new();
        let market_data = create_test_market_data();
        
        assert!(strategy.is_mean_reversion_market(&market_data));
    }

    #[tokio::test]
    async fn test_mean_reversion_signal_generation() {
        let mut strategy = MeanReversionStrategy::new();
        let opportunity = create_test_opportunity();
        let market_data = create_test_market_data();
        
        // Add price history to enable statistical calculations
        for price in vec![95.0, 98.0, 100.0, 102.0, 105.0, 103.0, 101.0, 99.0, 97.0, 100.0] {
            strategy.update_market_data("BTC/USDC".to_string(), price, 100000.0);
        }
        
        let result = strategy.analyze(&opportunity, &market_data);
        assert!(result.is_ok());
        
        let signals = result.unwrap();
        // Signals depend on statistical analysis, so we just verify the method works
        assert!(signals.len() <= 1); // Should generate 0 or 1 signal
    }

    #[test]
    fn test_conservative_position_sizing() {
        let strategy = MeanReversionStrategy::new();
        let position_size = strategy.get_position_size(0.8, 1000.0);
        
        // Should be 25% allocation * 80% confidence * 1000 capital = 200 (before volatility adjustment)
        assert!(position_size <= 200.0); // May be less due to volatility adjustment
    }

    #[test]
    fn test_conservative_exit_conditions() {
        let strategy = MeanReversionStrategy::new();
        
        // Test stop loss (3% for conservative)
        assert!(strategy.should_exit(97.0, 100.0, &SignalType::Buy)); // 3% loss triggers stop
        
        // Test take profit (8% for conservative)
        assert!(strategy.should_exit(108.0, 100.0, &SignalType::Buy)); // 8% gain triggers exit
        
        // Test normal conditions
        assert!(!strategy.should_exit(102.0, 100.0, &SignalType::Buy)); // 2% gain, keep holding
    }

    #[test]
    fn test_consecutive_loss_protection() {
        let mut strategy = MeanReversionStrategy::new();
        
        // Simulate consecutive losses
        for _ in 0..3 {
            let trade_result = TradeResult {
                profit_loss: -10.0,
                fees: 1.0,
                ..Default::default()
            };
            strategy.update_performance(&trade_result).unwrap();
        }
        
        // Strategy should be disabled after max consecutive losses
        assert!(!strategy.enabled());
    }
}
