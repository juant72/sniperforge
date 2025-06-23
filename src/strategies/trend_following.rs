use super::{TradingStrategy, StrategySignal, StrategyPerformance, StrategyConfig, SignalType, Timeframe, MarketData, TradeResult, RiskLevel};
use crate::shared::pool_detector::{TradingOpportunity, OpportunityType};
use anyhow::Result;
use std::collections::HashMap;

pub struct TrendFollowingStrategy {
    config: StrategyConfig,
    performance: StrategyPerformance,
    enabled: bool,
}

impl TrendFollowingStrategy {
    pub fn new() -> Self {
        let config = StrategyConfig {
            name: "Trend Following".to_string(),
            enabled: true,
            capital_allocation: 0.25, // 25% allocation
            risk_level: RiskLevel::Moderate,
            max_position_size: 500.0,
            stop_loss_percent: 3.0,
            take_profit_percent: 8.0,
            min_confidence: 0.65,
            timeframes: vec![Timeframe::FiveMin, Timeframe::FifteenMin, Timeframe::OneHour],
        };

        let performance = StrategyPerformance {
            strategy_name: "Trend Following".to_string(),
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

    fn calculate_moving_average(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let sum: f64 = prices.iter().rev().take(period).sum();
        Some(sum / period as f64)
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

    fn detect_breakout(&self, prices: &[f64], volume: &[f64]) -> Option<(bool, f64)> {
        if prices.len() < 20 || volume.len() < 20 {
            return None;
        }

        let recent_prices = &prices[prices.len()-20..];
        let recent_volume = &volume[volume.len()-20..];

        // Calculate resistance and support levels
        let max_price = recent_prices.iter().fold(0.0f64, |a, &b| a.max(b));
        let min_price = recent_prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        
        let current_price = prices[prices.len() - 1];
        let current_volume = volume[volume.len() - 1];
        let avg_volume: f64 = recent_volume.iter().sum::<f64>() / recent_volume.len() as f64;

        // Breakout conditions
        let resistance_breakout = current_price > max_price * 1.02; // 2% above resistance
        let support_breakdown = current_price < min_price * 0.98; // 2% below support
        let volume_confirmation = current_volume > avg_volume * 1.5; // 50% above average volume

        if resistance_breakout && volume_confirmation {
            Some((true, (current_price - max_price) / max_price)) // Bullish breakout
        } else if support_breakdown && volume_confirmation {
            Some((false, (min_price - current_price) / min_price)) // Bearish breakdown
        } else {
            None
        }
    }

    fn calculate_momentum(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let current_price = prices[prices.len() - 1];
        let past_price = prices[prices.len() - period];
        
        Some((current_price - past_price) / past_price)
    }

    fn analyze_trend_strength(&self, market_data: &MarketData) -> Result<f64> {
        let prices: Vec<f64> = market_data.price_history.iter().map(|p| p.close).collect();
        let volumes: Vec<f64> = market_data.volume_history.iter().map(|v| v.volume).collect();

        if prices.len() < 20 {
            return Ok(0.0);
        }

        // Calculate multiple indicators
        let ma_short = self.calculate_moving_average(&prices, 5).unwrap_or(0.0);
        let ma_long = self.calculate_moving_average(&prices, 20).unwrap_or(0.0);
        let ema_12 = self.calculate_ema(&prices, 12).unwrap_or(0.0);
        let ema_26 = self.calculate_ema(&prices, 26).unwrap_or(0.0);
        let rsi = self.calculate_rsi(&prices, 14).unwrap_or(50.0);
        let momentum_5 = self.calculate_momentum(&prices, 5).unwrap_or(0.0);
        let momentum_10 = self.calculate_momentum(&prices, 10).unwrap_or(0.0);

        let current_price = market_data.current_price;

        // Trend strength calculation
        let mut trend_score = 0.0;

        // Moving average trend
        if ma_short > ma_long {
            trend_score += 0.2; // Uptrend
        } else {
            trend_score -= 0.2; // Downtrend
        }

        // Price above/below moving averages
        if current_price > ma_short && current_price > ma_long {
            trend_score += 0.15;
        } else if current_price < ma_short && current_price < ma_long {
            trend_score -= 0.15;
        }

        // MACD-like signal
        let macd = ema_12 - ema_26;
        if macd > 0.0 {
            trend_score += 0.15;
        } else {
            trend_score -= 0.15;
        }

        // RSI momentum
        if rsi > 50.0 && rsi < 80.0 {
            trend_score += 0.1; // Bullish but not overbought
        } else if rsi < 50.0 && rsi > 20.0 {
            trend_score -= 0.1; // Bearish but not oversold
        }

        // Price momentum
        if momentum_5 > 0.02 && momentum_10 > 0.01 {
            trend_score += 0.2; // Strong upward momentum
        } else if momentum_5 < -0.02 && momentum_10 < -0.01 {
            trend_score -= 0.2; // Strong downward momentum
        }

        // Volume confirmation
        if volumes.len() >= 5 {
            let recent_avg_volume: f64 = volumes.iter().rev().take(5).sum::<f64>() / 5.0;
            let long_avg_volume: f64 = volumes.iter().sum::<f64>() / volumes.len() as f64;
            
            if recent_avg_volume > long_avg_volume * 1.2 {
                trend_score += 0.1; // Volume confirmation
            }
        }

        // Breakout detection
        if let Some((is_bullish, strength)) = self.detect_breakout(&prices, &volumes) {
            if is_bullish {
                trend_score += 0.1 + (strength * 0.5);
            } else {
                trend_score -= 0.1 + (strength * 0.5);
            }
        }

        // Normalize trend score to [-1, 1] range
        trend_score = trend_score.max(-1.0).min(1.0);

        Ok(trend_score)
    }
}

impl TradingStrategy for TrendFollowingStrategy {
    fn name(&self) -> &str {
        &self.config.name
    }    fn analyze(&self, _opportunity: &TradingOpportunity, market_data: &MarketData) -> Result<Option<StrategySignal>> {
        // Only analyze if we have sufficient data
        if market_data.price_history.len() < 20 {
            return Ok(None);
        }

        // Calculate trend strength
        let trend_strength = self.analyze_trend_strength(market_data)?;
        
        // Calculate confidence based on trend strength and market conditions
        let base_confidence = trend_strength.abs();
        
        // Additional confidence factors
        let liquidity_factor = if market_data.liquidity > 10000.0 { 0.1 } else { 0.0 };
        let volume_factor = if market_data.volume_24h > market_data.current_price * 50000.0 { 0.1 } else { 0.0 };
        let spread_factor = if market_data.bid_ask_spread < 0.01 { 0.05 } else { 0.0 };

        let confidence = (base_confidence + liquidity_factor + volume_factor + spread_factor).min(1.0);

        // Only generate signal if confidence is above threshold
        if confidence < self.config.min_confidence {
            return Ok(None);
        }

        // Determine signal type based on trend direction
        let signal_type = if trend_strength > 0.2 {
            SignalType::Buy
        } else if trend_strength < -0.2 {
            SignalType::Sell
        } else {
            return Ok(None); // No clear trend
        };

        // Calculate position size based on confidence and risk level
        let base_position_size = match self.config.risk_level {
            RiskLevel::Conservative => self.config.max_position_size * 0.5,
            RiskLevel::Moderate => self.config.max_position_size * 0.75,
            RiskLevel::Aggressive => self.config.max_position_size,
        };

        let position_size = base_position_size * confidence;

        // Calculate stop loss and take profit
        let stop_loss = match signal_type {
            SignalType::Buy => Some(market_data.current_price * (1.0 - self.config.stop_loss_percent / 100.0)),
            SignalType::Sell => Some(market_data.current_price * (1.0 + self.config.stop_loss_percent / 100.0)),
            _ => None,
        };

        let take_profit = match signal_type {
            SignalType::Buy => Some(market_data.current_price * (1.0 + self.config.take_profit_percent / 100.0)),
            SignalType::Sell => Some(market_data.current_price * (1.0 - self.config.take_profit_percent / 100.0)),
            _ => None,
        };

        // Create metadata with analysis details
        let mut metadata = HashMap::new();
        metadata.insert("trend_strength".to_string(), format!("{:.4}", trend_strength));
        metadata.insert("liquidity".to_string(), format!("{:.2}", market_data.liquidity));
        metadata.insert("volume_24h".to_string(), format!("{:.2}", market_data.volume_24h));
        metadata.insert("spread".to_string(), format!("{:.6}", market_data.bid_ask_spread));

        let signal = StrategySignal {
            strategy_name: self.config.name.clone(),
            signal_type,
            confidence,
            timeframe: Timeframe::FifteenMin, // Default timeframe
            entry_price: market_data.current_price,
            stop_loss,
            take_profit,
            position_size,
            timestamp: chrono::Utc::now(),
            metadata,
        };

        println!("📈 Trend Following Signal: {:?} with confidence {:.2}% (trend: {:.3})", 
            signal.signal_type, confidence * 100.0, trend_strength);

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
            let risk_free_rate = 0.02; // Assume 2% risk-free rate
            self.performance.sharpe_ratio = (avg_return - risk_free_rate) / (avg_return * 0.1); // Simplified calculation
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
