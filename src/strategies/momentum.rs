use super::{TradingStrategy, StrategySignal, StrategyPerformance, StrategyConfig, SignalType, Timeframe, MarketData, TradeResult, RiskLevel};
use crate::shared::pool_detector::{TradingOpportunity, OpportunityType};
use anyhow::Result;
use std::collections::HashMap;

pub struct MomentumStrategy {
    config: StrategyConfig,
    performance: StrategyPerformance,
    enabled: bool,
}

impl MomentumStrategy {
    pub fn new() -> Self {
        let config = StrategyConfig {
            name: "Momentum Trading".to_string(),
            enabled: true,
            capital_allocation: 0.3, // 30% allocation
            risk_level: RiskLevel::Aggressive,
            max_position_size: 400.0,
            stop_loss_percent: 5.0,
            take_profit_percent: 12.0,
            min_confidence: 0.6,
            timeframes: vec![Timeframe::OneMin, Timeframe::FiveMin, Timeframe::FifteenMin],
        };

        let performance = StrategyPerformance {
            strategy_name: "Momentum Trading".to_string(),
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

    fn calculate_price_momentum(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let current_price = prices[prices.len() - 1];
        let past_price = prices[prices.len() - period];
        
        Some((current_price - past_price) / past_price)
    }

    fn calculate_price_velocity(&self, prices: &[f64]) -> Option<f64> {
        if prices.len() < 3 {
            return None;
        }

        let current = prices[prices.len() - 1];
        let prev1 = prices[prices.len() - 2];
        let prev2 = prices[prices.len() - 3];

        let velocity1 = (current - prev1) / prev1;
        let velocity2 = (prev1 - prev2) / prev2;

        Some(velocity1 - velocity2) // Acceleration
    }

    fn calculate_volume_momentum(&self, volumes: &[f64], period: usize) -> Option<f64> {
        if volumes.len() < period {
            return None;
        }

        let recent_avg: f64 = volumes.iter().rev().take(period / 2).sum::<f64>() / (period / 2) as f64;
        let past_avg: f64 = volumes.iter().rev().skip(period / 2).take(period / 2).sum::<f64>() / (period / 2) as f64;

        if past_avg == 0.0 {
            return Some(0.0);
        }

        Some((recent_avg - past_avg) / past_avg)
    }

    fn calculate_macd(&self, prices: &[f64], fast_period: usize, slow_period: usize, signal_period: usize) -> Option<(f64, f64, f64)> {
        if prices.len() < slow_period.max(signal_period) {
            return None;
        }

        let ema_fast = self.calculate_ema(prices, fast_period)?;
        let ema_slow = self.calculate_ema(prices, slow_period)?;
        let macd_line = ema_fast - ema_slow;

        // For simplicity, we'll use a simple moving average for the signal line
        // In practice, this should be an EMA of the MACD line
        let signal_line = macd_line * 0.9; // Simplified

        let histogram = macd_line - signal_line;

        Some((macd_line, signal_line, histogram))
    }

    fn calculate_ema(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];

        for &price in prices.iter().skip(1) {
            ema = alpha * price + (1.0 - alpha) * ema;
        }

        Some(ema)
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

    fn detect_volume_spike(&self, volumes: &[f64], threshold: f64) -> bool {
        if volumes.len() < 10 {
            return false;
        }

        let current_volume = volumes[volumes.len() - 1];
        let avg_volume: f64 = volumes.iter().rev().skip(1).take(9).sum::<f64>() / 9.0;

        current_volume > avg_volume * threshold
    }

    fn detect_price_breakout(&self, prices: &[f64], lookback: usize) -> Option<(bool, f64)> {
        if prices.len() < lookback + 1 {
            return None;
        }

        let current_price = prices[prices.len() - 1];
        let historical_prices = &prices[prices.len() - lookback - 1..prices.len() - 1];
        
        let max_price = historical_prices.iter().fold(0.0f64, |a, &b| a.max(b));
        let min_price = historical_prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        let upward_breakout = current_price > max_price * 1.01; // 1% above historical high
        let downward_breakout = current_price < min_price * 0.99; // 1% below historical low

        if upward_breakout {
            Some((true, (current_price - max_price) / max_price))
        } else if downward_breakout {
            Some((false, (min_price - current_price) / min_price))
        } else {
            None
        }
    }

    fn calculate_momentum_score(&self, market_data: &MarketData) -> Result<f64> {
        let prices: Vec<f64> = market_data.price_history.iter().map(|p| p.close).collect();
        let volumes: Vec<f64> = market_data.volume_history.iter().map(|v| v.volume).collect();

        if prices.len() < 20 {
            return Ok(0.0);
        }

        let mut momentum_score = 0.0;

        // Price momentum (short, medium, long term)
        let momentum_1min = self.calculate_price_momentum(&prices, 1).unwrap_or(0.0);
        let momentum_5min = self.calculate_price_momentum(&prices, 5).unwrap_or(0.0);
        let momentum_15min = self.calculate_price_momentum(&prices, 15).unwrap_or(0.0);

        // Weight recent momentum more heavily
        momentum_score += momentum_1min * 0.4;
        momentum_score += momentum_5min * 0.3;
        momentum_score += momentum_15min * 0.2;

        // Price velocity (acceleration)
        if let Some(velocity) = self.calculate_price_velocity(&prices) {
            momentum_score += velocity * 0.3;
        }

        // Volume momentum
        if let Some(vol_momentum) = self.calculate_volume_momentum(&volumes, 10) {
            momentum_score += vol_momentum * 0.2;
        }

        // MACD momentum
        if let Some((macd_line, signal_line, histogram)) = self.calculate_macd(&prices, 12, 26, 9) {
            if macd_line > signal_line && histogram > 0.0 {
                momentum_score += 0.15; // Bullish MACD
            } else if macd_line < signal_line && histogram < 0.0 {
                momentum_score -= 0.15; // Bearish MACD
            }
        }

        // RSI momentum (avoid extreme overbought/oversold)
        if let Some(rsi) = self.calculate_rsi(&prices, 14) {
            if rsi > 45.0 && rsi < 75.0 {
                momentum_score += 0.1; // Good momentum zone
            } else if rsi < 55.0 && rsi > 25.0 {
                momentum_score -= 0.1; // Bearish momentum zone
            } else if rsi > 80.0 || rsi < 20.0 {
                momentum_score *= 0.5; // Reduce momentum in extreme zones
            }
        }

        // Volume spike confirmation
        if self.detect_volume_spike(&volumes, 2.0) {
            momentum_score *= 1.3; // Volume confirms momentum
        }

        // Breakout confirmation
        if let Some((is_upward, strength)) = self.detect_price_breakout(&prices, 20) {
            if is_upward {
                momentum_score += 0.2 + (strength * 0.5);
            } else {
                momentum_score -= 0.2 + (strength * 0.5);
            }
        }

        // Market conditions factor
        let price_change_24h = market_data.price_change_24h;
        if price_change_24h.abs() > 0.05 {
            momentum_score *= 1.1; // Higher volatility can create momentum opportunities
        }

        // Liquidity factor (momentum works better in liquid markets)
        if market_data.liquidity > 20000.0 {
            momentum_score *= 1.05;
        } else if market_data.liquidity < 5000.0 {
            momentum_score *= 0.9;
        }

        // Normalize to [-1, 1] range
        momentum_score = momentum_score.max(-1.0).min(1.0);

        Ok(momentum_score)
    }
}

impl TradingStrategy for MomentumStrategy {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn analyze(&self, opportunity: &TradingOpportunity, market_data: &MarketData) -> Result<Option<StrategySignal>> {
        // Need sufficient data for momentum analysis
        if market_data.price_history.len() < 20 {
            return Ok(None);
        }

        let prices: Vec<f64> = market_data.price_history.iter().map(|p| p.close).collect();
        let volumes: Vec<f64> = market_data.volume_history.iter().map(|v| v.volume).collect();
        let current_price = market_data.current_price;

        // Calculate momentum score
        let momentum_score = self.calculate_momentum_score(market_data)?;

        // Calculate supporting indicators
        let momentum_5min = self.calculate_price_momentum(&prices, 5).unwrap_or(0.0);
        let momentum_15min = self.calculate_price_momentum(&prices, 15).unwrap_or(0.0);
        let rsi = self.calculate_rsi(&prices, 14).unwrap_or(50.0);
        let volume_spike = self.detect_volume_spike(&volumes, 1.5);

        // Determine signal type based on momentum strength
        let signal_type = if momentum_score > 0.4 && momentum_5min > 0.02 {
            SignalType::Buy // Strong upward momentum
        } else if momentum_score < -0.4 && momentum_5min < -0.02 {
            SignalType::Sell // Strong downward momentum
        } else {
            return Ok(None); // Insufficient momentum
        };

        // Calculate confidence based on multiple momentum factors
        let base_confidence = momentum_score.abs();
        
        // Momentum alignment bonus
        let alignment_bonus = if (momentum_5min > 0.0 && momentum_15min > 0.0 && momentum_score > 0.0) ||
                                 (momentum_5min < 0.0 && momentum_15min < 0.0 && momentum_score < 0.0) {
            0.15 // All timeframes aligned
        } else {
            0.0
        };

        // Volume confirmation bonus
        let volume_bonus = if volume_spike { 0.1 } else { 0.0 };

        // RSI confirmation (avoid extreme zones)
        let rsi_bonus = match signal_type {
            SignalType::Buy => if rsi > 45.0 && rsi < 75.0 { 0.1 } else { 0.0 },
            SignalType::Sell => if rsi < 55.0 && rsi > 25.0 { 0.1 } else { 0.0 },
            _ => 0.0,
        };

        // Liquidity factor
        let liquidity_bonus = if market_data.liquidity > 15000.0 { 0.05 } else { 0.0 };

        let confidence = (base_confidence + alignment_bonus + volume_bonus + rsi_bonus + liquidity_bonus).min(1.0);

        // Only generate signal if confidence meets threshold
        if confidence < self.config.min_confidence {
            return Ok(None);
        }

        // Calculate position size based on momentum strength and confidence
        let base_position_size = match self.config.risk_level {
            RiskLevel::Conservative => self.config.max_position_size * 0.6,
            RiskLevel::Moderate => self.config.max_position_size * 0.8,
            RiskLevel::Aggressive => self.config.max_position_size,
        };

        // Increase position size for stronger momentum
        let momentum_multiplier = 1.0 + (momentum_score.abs() * 0.5);
        let position_size = base_position_size * confidence * momentum_multiplier;

        // Calculate dynamic stop loss based on volatility
        let volatility_factor = if prices.len() >= 10 {
            let recent_prices: Vec<f64> = prices.iter().rev().take(10).cloned().collect();
            let mean: f64 = recent_prices.iter().sum::<f64>() / recent_prices.len() as f64;
            let variance: f64 = recent_prices.iter()
                .map(|price| (price - mean).powi(2))
                .sum::<f64>() / recent_prices.len() as f64;
            variance.sqrt() / mean
        } else {
            0.02 // Default 2% volatility
        };

        let dynamic_stop_loss = (self.config.stop_loss_percent / 100.0).max(volatility_factor * 2.0);
        let dynamic_take_profit = (self.config.take_profit_percent / 100.0).max(volatility_factor * 3.0);

        let stop_loss = match signal_type {
            SignalType::Buy => Some(current_price * (1.0 - dynamic_stop_loss)),
            SignalType::Sell => Some(current_price * (1.0 + dynamic_stop_loss)),
            _ => None,
        };

        let take_profit = match signal_type {
            SignalType::Buy => Some(current_price * (1.0 + dynamic_take_profit)),
            SignalType::Sell => Some(current_price * (1.0 - dynamic_take_profit)),
            _ => None,
        };

        // Create metadata with detailed momentum analysis
        let mut metadata = HashMap::new();
        metadata.insert("momentum_score".to_string(), format!("{:.4}", momentum_score));
        metadata.insert("momentum_5min".to_string(), format!("{:.4}", momentum_5min));
        metadata.insert("momentum_15min".to_string(), format!("{:.4}", momentum_15min));
        metadata.insert("rsi".to_string(), format!("{:.2}", rsi));
        metadata.insert("volume_spike".to_string(), volume_spike.to_string());
        metadata.insert("volatility".to_string(), format!("{:.4}", volatility_factor));
        metadata.insert("alignment_bonus".to_string(), format!("{:.3}", alignment_bonus));

        // Determine appropriate timeframe based on momentum strength
        let timeframe = if momentum_score.abs() > 0.7 {
            Timeframe::OneMin // Very strong momentum - act quickly
        } else if momentum_score.abs() > 0.5 {
            Timeframe::FiveMin // Strong momentum
        } else {
            Timeframe::FifteenMin // Moderate momentum
        };

        let signal = StrategySignal {
            strategy_name: self.config.name.clone(),
            signal_type,
            confidence,
            timeframe,
            entry_price: current_price,
            stop_loss,
            take_profit,
            position_size,
            timestamp: chrono::Utc::now(),
            metadata,
        };

        println!("🚀 Momentum Signal: {:?} with confidence {:.2}% (score: {:.3}, 5min: {:.2}%, vol_spike: {})", 
            signal.signal_type, confidence * 100.0, momentum_score, momentum_5min * 100.0, volume_spike);

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
