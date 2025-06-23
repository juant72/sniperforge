use super::{TradingStrategy, StrategySignal, StrategyPerformance, StrategyConfig, SignalType, Timeframe, MarketData, TradeResult, RiskLevel};
use crate::shared::pool_detector::{TradingOpportunity, OpportunityType};
use anyhow::Result;
use std::collections::HashMap;

pub struct MeanReversionStrategy {
    config: StrategyConfig,
    performance: StrategyPerformance,
    enabled: bool,
}

impl MeanReversionStrategy {
    pub fn new() -> Self {
        let config = StrategyConfig {
            name: "Mean Reversion".to_string(),
            enabled: true,
            capital_allocation: 0.2, // 20% allocation
            risk_level: RiskLevel::Conservative,
            max_position_size: 300.0,
            stop_loss_percent: 4.0,
            take_profit_percent: 6.0,
            min_confidence: 0.7,
            timeframes: vec![Timeframe::OneMin, Timeframe::FiveMin, Timeframe::FifteenMin],
        };

        let performance = StrategyPerformance {
            strategy_name: "Mean Reversion".to_string(),
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
            last_updated: chrono::Utc::now(),
        };

        Self {
            config,
            performance,
            enabled: true,
        }
    }

    pub fn with_config(config: StrategyConfig) -> Self {
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
            last_updated: chrono::Utc::now(),
        };

        Self {
            config,
            performance,
            enabled: true,
        }
    }

    fn calculate_bollinger_bands(&self, prices: &[f64], period: usize, std_dev: f64) -> Option<(f64, f64, f64)> {
        if prices.len() < period {
            return None;
        }

        let recent_prices: Vec<f64> = prices.iter().rev().take(period).cloned().collect();
        let mean: f64 = recent_prices.iter().sum::<f64>() / period as f64;
        
        let variance: f64 = recent_prices.iter()
            .map(|price| (price - mean).powi(2))
            .sum::<f64>() / period as f64;
        
        let std_deviation = variance.sqrt();
        
        let upper_band = mean + (std_dev * std_deviation);
        let lower_band = mean - (std_dev * std_deviation);
        
        Some((lower_band, mean, upper_band))
    }

    fn calculate_rsi(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period + 1 {
            return None;
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        if gains.len() < period {
            return None;
        }

        let avg_gain: f64 = gains.iter().rev().take(period).sum::<f64>() / period as f64;
        let avg_loss: f64 = losses.iter().rev().take(period).sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Some(rsi)
    }

    fn calculate_stochastic(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let recent_prices: Vec<f64> = prices.iter().rev().take(period).cloned().collect();
        let highest = recent_prices.iter().fold(0.0f64, |a, &b| a.max(b));
        let lowest = recent_prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let current = prices[prices.len() - 1];

        if highest == lowest {
            return Some(50.0);
        }

        let stoch_k = ((current - lowest) / (highest - lowest)) * 100.0;
        Some(stoch_k)
    }    fn detect_support_resistance(&self, prices: &[f64], _volume: &[f64]) -> Option<(f64, f64)> {
        if prices.len() < 30 {
            return None;
        }

        let window_size = 10;
        let mut support_levels = Vec::new();
        let mut resistance_levels = Vec::new();

        // Find local minima (support) and maxima (resistance)
        for i in window_size..prices.len()-window_size {
            let mut is_local_min = true;
            let mut is_local_max = true;

            for j in 1..=window_size {
                if prices[i] > prices[i-j] || prices[i] > prices[i+j] {
                    is_local_min = false;
                }
                if prices[i] < prices[i-j] || prices[i] < prices[i+j] {
                    is_local_max = false;
                }
            }

            if is_local_min {
                support_levels.push(prices[i]);
            }
            if is_local_max {
                resistance_levels.push(prices[i]);
            }
        }

        // Find the most relevant support and resistance levels
        let current_price = prices[prices.len() - 1];
        
        let support = support_levels.iter()
            .filter(|&&s| s < current_price)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(current_price * 0.95);

        let resistance = resistance_levels.iter()
            .filter(|&&r| r > current_price)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(current_price * 1.05);

        Some((support, resistance))
    }

    fn calculate_price_deviation(&self, current_price: f64, mean_price: f64) -> f64 {
        (current_price - mean_price) / mean_price
    }

    fn is_oversold(&self, rsi: f64, stoch: f64, bb_position: f64) -> bool {
        rsi < 30.0 && stoch < 20.0 && bb_position < -0.8
    }

    fn is_overbought(&self, rsi: f64, stoch: f64, bb_position: f64) -> bool {
        rsi > 70.0 && stoch > 80.0 && bb_position > 0.8
    }

    fn analyze_mean_reversion_opportunity(&self, market_data: &MarketData) -> Result<f64> {
        let prices: Vec<f64> = market_data.price_history.iter().map(|p| p.close).collect();
        let volumes: Vec<f64> = market_data.volume_history.iter().map(|v| v.volume).collect();

        if prices.len() < 30 {
            return Ok(0.0);
        }

        let current_price = market_data.current_price;

        // Calculate technical indicators
        let rsi = self.calculate_rsi(&prices, 14).unwrap_or(50.0);
        let stoch = self.calculate_stochastic(&prices, 14).unwrap_or(50.0);
        
        // Bollinger Bands
        let (lower_band, middle_band, upper_band) = self.calculate_bollinger_bands(&prices, 20, 2.0)
            .unwrap_or((current_price * 0.98, current_price, current_price * 1.02));

        // Calculate position relative to Bollinger Bands
        let bb_width = upper_band - lower_band;
        let bb_position = if bb_width > 0.0 {
            (current_price - middle_band) / (bb_width / 2.0)
        } else {
            0.0
        };

        // Support and Resistance levels
        let (support, resistance) = self.detect_support_resistance(&prices, &volumes)
            .unwrap_or((current_price * 0.95, current_price * 1.05));

        // Calculate mean reversion score
        let mut reversion_score = 0.0;

        // RSI signals
        if rsi < 25.0 {
            reversion_score += 0.3; // Strong oversold condition
        } else if rsi < 35.0 {
            reversion_score += 0.2; // Moderate oversold
        } else if rsi > 75.0 {
            reversion_score -= 0.3; // Strong overbought condition
        } else if rsi > 65.0 {
            reversion_score -= 0.2; // Moderate overbought
        }

        // Stochastic signals
        if stoch < 15.0 {
            reversion_score += 0.2;
        } else if stoch > 85.0 {
            reversion_score -= 0.2;
        }

        // Bollinger Band signals
        if bb_position < -0.9 {
            reversion_score += 0.25; // Price near lower band
        } else if bb_position > 0.9 {
            reversion_score -= 0.25; // Price near upper band
        }

        // Support/Resistance signals
        let distance_to_support = (current_price - support) / support;
        let distance_to_resistance = (resistance - current_price) / current_price;

        if distance_to_support < 0.02 {
            reversion_score += 0.15; // Near support level
        } else if distance_to_resistance < 0.02 {
            reversion_score -= 0.15; // Near resistance level
        }

        // Volume confirmation
        if volumes.len() >= 5 {
            let recent_volume: f64 = volumes.iter().rev().take(5).sum::<f64>() / 5.0;
            let avg_volume: f64 = volumes.iter().sum::<f64>() / volumes.len() as f64;
            
            if recent_volume > avg_volume * 1.3 {
                reversion_score *= 1.1; // Volume confirmation
            }
        }

        // Price velocity (rapid moves are more likely to revert)
        if prices.len() >= 5 {
            let price_change_5min = (current_price - prices[prices.len() - 5]) / prices[prices.len() - 5];
            if price_change_5min.abs() > 0.03 {
                reversion_score += 0.1 * price_change_5min.abs(); // Rapid move increases reversion probability
            }
        }

        // Normalize score to [-1, 1] range
        reversion_score = reversion_score.max(-1.0).min(1.0);

        Ok(reversion_score)
    }
}

impl TradingStrategy for MeanReversionStrategy {
    fn name(&self) -> &str {
        &self.config.name
    }    fn analyze(&self, _opportunity: &TradingOpportunity, market_data: &MarketData) -> Result<Option<StrategySignal>> {
        // Need sufficient price history for mean reversion analysis
        if market_data.price_history.len() < 30 {
            return Ok(None);
        }

        let prices: Vec<f64> = market_data.price_history.iter().map(|p| p.close).collect();
        let current_price = market_data.current_price;

        // Calculate mean reversion opportunity score
        let reversion_score = self.analyze_mean_reversion_opportunity(market_data)?;
        
        // Calculate technical indicators for confirmation
        let rsi = self.calculate_rsi(&prices, 14).unwrap_or(50.0);
        let stoch = self.calculate_stochastic(&prices, 14).unwrap_or(50.0);
        
        let (lower_band, middle_band, upper_band) = self.calculate_bollinger_bands(&prices, 20, 2.0)
            .unwrap_or((current_price * 0.98, current_price, current_price * 1.02));

        let bb_position = if upper_band != lower_band {
            (current_price - middle_band) / ((upper_band - lower_band) / 2.0)
        } else {
            0.0
        };

        // Determine signal based on mean reversion conditions
        let signal_type = if self.is_oversold(rsi, stoch, bb_position) && reversion_score > 0.3 {
            SignalType::Buy // Expect price to revert upward
        } else if self.is_overbought(rsi, stoch, bb_position) && reversion_score < -0.3 {
            SignalType::Sell // Expect price to revert downward
        } else {
            return Ok(None); // No clear mean reversion opportunity
        };

        // Calculate confidence based on strength of reversion signals
        let base_confidence = reversion_score.abs();
        
        // Additional confidence factors
        let rsi_factor = match signal_type {
            SignalType::Buy => if rsi < 25.0 { 0.15 } else if rsi < 35.0 { 0.1 } else { 0.0 },
            SignalType::Sell => if rsi > 75.0 { 0.15 } else if rsi > 65.0 { 0.1 } else { 0.0 },
            _ => 0.0,
        };

        let bb_factor = match signal_type {
            SignalType::Buy => if bb_position < -0.8 { 0.1 } else { 0.0 },
            SignalType::Sell => if bb_position > 0.8 { 0.1 } else { 0.0 },
            _ => 0.0,
        };

        let liquidity_factor = if market_data.liquidity > 5000.0 { 0.05 } else { 0.0 };
        
        let confidence = (base_confidence + rsi_factor + bb_factor + liquidity_factor).min(1.0);

        // Only generate signal if confidence is above threshold
        if confidence < self.config.min_confidence {
            return Ok(None);
        }

        // Calculate position size based on confidence and risk level
        let base_position_size = match self.config.risk_level {
            RiskLevel::Conservative => self.config.max_position_size * 0.4,
            RiskLevel::Moderate => self.config.max_position_size * 0.6,
            RiskLevel::Aggressive => self.config.max_position_size * 0.8,
        };

        let position_size = base_position_size * confidence;

        // Calculate stop loss and take profit based on Bollinger Bands
        let stop_loss = match signal_type {
            SignalType::Buy => Some((lower_band * 0.995).min(current_price * (1.0 - self.config.stop_loss_percent / 100.0))),
            SignalType::Sell => Some((upper_band * 1.005).max(current_price * (1.0 + self.config.stop_loss_percent / 100.0))),
            _ => None,
        };

        let take_profit = match signal_type {
            SignalType::Buy => Some(middle_band.min(current_price * (1.0 + self.config.take_profit_percent / 100.0))),
            SignalType::Sell => Some(middle_band.max(current_price * (1.0 - self.config.take_profit_percent / 100.0))),
            _ => None,
        };

        // Create metadata with analysis details
        let mut metadata = HashMap::new();
        metadata.insert("reversion_score".to_string(), format!("{:.4}", reversion_score));
        metadata.insert("rsi".to_string(), format!("{:.2}", rsi));
        metadata.insert("stochastic".to_string(), format!("{:.2}", stoch));
        metadata.insert("bb_position".to_string(), format!("{:.3}", bb_position));
        metadata.insert("lower_band".to_string(), format!("{:.6}", lower_band));
        metadata.insert("upper_band".to_string(), format!("{:.6}", upper_band));
        metadata.insert("middle_band".to_string(), format!("{:.6}", middle_band));

        let signal = StrategySignal {
            strategy_name: self.config.name.clone(),
            signal_type,
            confidence,
            timeframe: Timeframe::FiveMin, // Mean reversion works well on shorter timeframes
            entry_price: current_price,
            stop_loss,
            take_profit,
            position_size,
            timestamp: chrono::Utc::now(),
            metadata,
        };

        println!("🔄 Mean Reversion Signal: {:?} with confidence {:.2}% (RSI: {:.1}, Stoch: {:.1}, BB: {:.2})", 
            signal.signal_type, confidence * 100.0, rsi, stoch, bb_position);

        Ok(Some(signal))
    }

    fn update_performance(&mut self, trade_result: &TradeResult) -> Result<()> {
        self.performance.total_trades += 1;
        self.performance.total_profit_loss += trade_result.profit_loss;
        self.performance.total_fees += trade_result.fees;

        if trade_result.success {
            self.performance.winning_trades += 1;
            if self.performance.average_profit == 0.0 {
                self.performance.average_profit = trade_result.profit_loss;
            } else {
                self.performance.average_profit = 
                    (self.performance.average_profit * (self.performance.winning_trades - 1) as f64 + trade_result.profit_loss) 
                    / self.performance.winning_trades as f64;
            }
        } else {
            self.performance.losing_trades += 1;
            if self.performance.average_loss == 0.0 {
                self.performance.average_loss = trade_result.profit_loss.abs();
            } else {
                self.performance.average_loss = 
                    (self.performance.average_loss * (self.performance.losing_trades - 1) as f64 + trade_result.profit_loss.abs()) 
                    / self.performance.losing_trades as f64;
            }
        }

        self.performance.win_rate = self.performance.winning_trades as f64 / self.performance.total_trades as f64;
        self.performance.last_updated = chrono::Utc::now();

        // Calculate Sharpe ratio (simplified)
        if self.performance.total_trades > 5 {
            let avg_return = self.performance.total_profit_loss / self.performance.total_trades as f64;
            let risk_free_rate = 0.02;
            self.performance.sharpe_ratio = (avg_return - risk_free_rate) / (avg_return * 0.1);
        }

        Ok(())
    }

    fn get_performance(&self) -> &StrategyPerformance {
        &self.performance
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_config(&self) -> &StrategyConfig {
        &self.config
    }
}
