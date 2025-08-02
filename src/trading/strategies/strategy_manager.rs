//! Strategy Manager - Enterprise Coordination System
//! 
//! This module provides centralized management and coordination of all trading strategies
//! with enterprise-grade features including portfolio balancing, risk management,
//! and performance monitoring across multiple strategies.

use super::{
    TradingStrategy, StrategyPerformance, StrategySignal, SignalType,
    ArbitrageStrategy, MomentumStrategy, MeanReversionStrategy, TradeResult
};
use crate::types::{TradingOpportunity, MarketData};
use crate::config::SimpleConfig;
use anyhow::Result;
use std::collections::HashMap;
use chrono::Utc;
use tracing::{info, warn, debug, error};

/// Enterprise strategy manager coordinating multiple trading strategies
pub struct StrategyManager {
    strategies: HashMap<String, Box<dyn TradingStrategy>>,
    config: SimpleConfig,
    total_capital: f64,
    allocated_capital: HashMap<String, f64>,
    
    // 🎯 Enterprise portfolio management
    max_total_risk_exposure: f64,    // Maximum total portfolio risk
    correlation_matrix: HashMap<String, HashMap<String, f64>>, // Strategy correlation tracking
    rebalancing_threshold: f64,      // Threshold for portfolio rebalancing
    
    // 📊 Performance tracking
    global_performance: StrategyPerformance,
    strategy_weights: HashMap<String, f64>, // Dynamic strategy weights
    
    // 🛡️ Risk management
    max_concurrent_strategies: usize,        // Maximum strategies active simultaneously
    emergency_stop_threshold: f64,           // Emergency stop loss threshold
    daily_loss_limit: f64,                   // Daily loss limit
    current_daily_loss: f64,                 // Current daily loss tracking
    
    // 🔄 Strategy coordination
    signal_aggregation_method: String,       // Method for aggregating signals
    min_signal_agreement: f64,               // Minimum agreement between strategies
    conflict_resolution_strategy: String,    // How to resolve conflicting signals
}

impl StrategyManager {
    /// Create a new enterprise strategy manager
    pub fn new(config: SimpleConfig) -> Self {
        let total_capital = config.trading_amount * 100.0; // Assume 100x trading amount as portfolio
        
        let global_performance = StrategyPerformance {
            strategy_name: "Portfolio Manager".to_string(),
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
            strategies: HashMap::new(),
            config,
            total_capital,
            allocated_capital: HashMap::new(),
            
            // Portfolio management parameters
            max_total_risk_exposure: 0.15,     // 15% max total portfolio risk
            correlation_matrix: HashMap::new(),
            rebalancing_threshold: 0.05,       // 5% rebalancing threshold
            
            // Performance tracking
            global_performance,
            strategy_weights: HashMap::new(),
            
            // Risk management
            max_concurrent_strategies: 3,      // Maximum 3 strategies active
            emergency_stop_threshold: -0.10,  // 10% emergency stop
            daily_loss_limit: -0.05,          // 5% daily loss limit
            current_daily_loss: 0.0,
            
            // Strategy coordination
            signal_aggregation_method: "weighted_average".to_string(),
            min_signal_agreement: 0.6,         // 60% minimum agreement
            conflict_resolution_strategy: "highest_confidence".to_string(),
        }
    }
    
    /// Initialize and register all available strategies
    pub fn initialize_strategies(&mut self) -> Result<()> {
        info!("Initializing enterprise trading strategies...");
        
        // Initialize Arbitrage Strategy
        let arbitrage = ArbitrageStrategy::new();
        let arbitrage_allocation = 0.40; // 40% allocation
        self.allocated_capital.insert("arbitrage".to_string(), self.total_capital * arbitrage_allocation);
        self.strategy_weights.insert("arbitrage".to_string(), arbitrage_allocation);
        self.strategies.insert("arbitrage".to_string(), Box::new(arbitrage));
        
        // Initialize Momentum Strategy
        let momentum = MomentumStrategy::new();
        let momentum_allocation = 0.35; // 35% allocation
        self.allocated_capital.insert("momentum".to_string(), self.total_capital * momentum_allocation);
        self.strategy_weights.insert("momentum".to_string(), momentum_allocation);
        self.strategies.insert("momentum".to_string(), Box::new(momentum));
        
        // Initialize Mean Reversion Strategy
        let mean_reversion = MeanReversionStrategy::new();
        let mean_reversion_allocation = 0.25; // 25% allocation
        self.allocated_capital.insert("mean_reversion".to_string(), self.total_capital * mean_reversion_allocation);
        self.strategy_weights.insert("mean_reversion".to_string(), mean_reversion_allocation);
        self.strategies.insert("mean_reversion".to_string(), Box::new(mean_reversion));
        
        info!("✅ Initialized {} enterprise strategies with total allocation: {:.1}%", 
              self.strategies.len(), 
              (arbitrage_allocation + momentum_allocation + mean_reversion_allocation) * 100.0);
        
        Ok(())
    }
    
    /// Analyze market opportunities across all strategies
    pub async fn analyze_opportunity(&mut self, opportunity: &TradingOpportunity, market_data: &MarketData) -> Result<Vec<StrategySignal>> {
        if self.current_daily_loss <= self.emergency_stop_threshold {
            warn!("Emergency stop triggered - daily loss limit exceeded: {:.2}%", 
                  self.current_daily_loss * 100.0);
            return Ok(vec![]);
        }
        
        let mut all_signals = Vec::new();
        let mut strategy_signals: HashMap<String, Vec<StrategySignal>> = HashMap::new();
        
        // Collect signals from all enabled strategies
        for (strategy_name, strategy) in &mut self.strategies {
            if strategy.enabled() {
                match strategy.analyze(opportunity, market_data) {
                    Ok(signals) => {
                        if !signals.is_empty() {
                            debug!("Strategy '{}' generated {} signals", strategy_name, signals.len());
                            strategy_signals.insert(strategy_name.clone(), signals.clone());
                            all_signals.extend(signals);
                        }
                    },
                    Err(e) => {
                        error!("Error analyzing opportunity with strategy '{}': {}", strategy_name, e);
                    }
                }
            }
        }
        
        // Apply signal coordination and filtering
        let coordinated_signals = self.coordinate_signals(strategy_signals, market_data).await?;
        
        Ok(coordinated_signals)
    }
    
    /// Coordinate signals from multiple strategies using enterprise logic
    async fn coordinate_signals(&self, strategy_signals: HashMap<String, Vec<StrategySignal>>, _market_data: &MarketData) -> Result<Vec<StrategySignal>> {
        let mut coordinated_signals = Vec::new();
        
        if strategy_signals.is_empty() {
            return Ok(coordinated_signals);
        }
        
        // Group signals by token pair and signal type
        let mut signal_groups: HashMap<(String, SignalType), Vec<(String, StrategySignal)>> = HashMap::new();
        
        for (strategy_name, signals) in strategy_signals {
            for signal in signals {
                let key = (signal.token_pair.clone(), signal.signal_type.clone());
                signal_groups.entry(key).or_insert_with(|| Vec::new())
                    .push((strategy_name.clone(), signal));
            }
        }
        
        // Process each signal group
        let signal_groups_count = signal_groups.len();
        for ((token_pair, signal_type), grouped_signals) in signal_groups {
            let coordinated_signal = self.aggregate_signals(token_pair, signal_type, grouped_signals).await?;
            
            if let Some(signal) = coordinated_signal {
                coordinated_signals.push(signal);
            }
        }
        
        // Apply portfolio-level constraints
        let final_signals = self.apply_portfolio_constraints(coordinated_signals).await?;
        
        info!("Coordinated {} enterprise signals from {} strategy groups", 
              final_signals.len(), signal_groups_count);
        
        Ok(final_signals)
    }
    
    /// Aggregate signals from multiple strategies for the same asset and direction
    async fn aggregate_signals(&self, token_pair: String, signal_type: SignalType, signals: Vec<(String, StrategySignal)>) -> Result<Option<StrategySignal>> {
        if signals.is_empty() {
            return Ok(None);
        }
        
        if signals.len() == 1 {
            // Single signal - return as is
            return Ok(Some(signals[0].1.clone()));
        }
        
        // Multiple signals - apply aggregation method
        match self.signal_aggregation_method.as_str() {
            "weighted_average" => {
                let mut total_weight = 0.0;
                let mut weighted_confidence = 0.0;
                let mut total_volume = 0.0;
                let mut latest_timestamp = signals[0].1.timestamp;
                
                for (strategy_name, signal) in &signals {
                    if let Some(&weight) = self.strategy_weights.get(strategy_name) {
                        total_weight += weight;
                        weighted_confidence += signal.confidence * weight;
                        total_volume += signal.volume;
                        
                        if signal.timestamp > latest_timestamp {
                            latest_timestamp = signal.timestamp;
                        }
                    }
                }
                
                if total_weight > 0.0 {
                    let final_confidence = weighted_confidence / total_weight;
                    
                    // Check if aggregated confidence meets minimum agreement
                    if final_confidence >= self.min_signal_agreement {
                        let aggregated_signal = StrategySignal {
                            strategy_name: "Portfolio Manager".to_string(),
                            signal_type,
                            confidence: final_confidence,
                            timeframe: signals[0].1.timeframe.clone(), // Use first signal's timeframe
                            token_pair,
                            price: signals[0].1.price, // Use first signal's price
                            volume: total_volume / signals.len() as f64, // Average volume
                            timestamp: latest_timestamp,
                            metadata: Some(format!(
                                "aggregated from {} strategies, weighted_confidence: {:.2}",
                                signals.len(), final_confidence
                            )),
                        };
                        
                        return Ok(Some(aggregated_signal));
                    }
                }
            },
            "highest_confidence" => {
                // Return signal with highest confidence
                let best_signal = signals.iter()
                    .max_by(|a, b| a.1.confidence.partial_cmp(&b.1.confidence).unwrap_or(std::cmp::Ordering::Equal));
                
                if let Some((_, signal)) = best_signal {
                    if signal.confidence >= self.min_signal_agreement {
                        return Ok(Some(signal.clone()));
                    }
                }
            },
            "consensus" => {
                // Require consensus from multiple strategies
                if signals.len() >= 2 {
                    let avg_confidence = signals.iter().map(|(_, s)| s.confidence).sum::<f64>() / signals.len() as f64;
                    
                    if avg_confidence >= self.min_signal_agreement {
                        let consensus_signal = StrategySignal {
                            strategy_name: "Portfolio Manager (Consensus)".to_string(),
                            signal_type,
                            confidence: avg_confidence,
                            timeframe: signals[0].1.timeframe.clone(),
                            token_pair,
                            price: signals[0].1.price,
                            volume: signals.iter().map(|(_, s)| s.volume).sum::<f64>() / signals.len() as f64,
                            timestamp: signals.iter().map(|(_, s)| s.timestamp).max().unwrap_or(Utc::now()),
                            metadata: Some(format!(
                                "consensus from {} strategies, avg_confidence: {:.2}",
                                signals.len(), avg_confidence
                            )),
                        };
                        
                        return Ok(Some(consensus_signal));
                    }
                }
            },
            _ => {
                warn!("Unknown signal aggregation method: {}", self.signal_aggregation_method);
                return Ok(Some(signals[0].1.clone())); // Fallback to first signal
            }
        }
        
        Ok(None)
    }
    
    /// Apply portfolio-level constraints and risk management
    async fn apply_portfolio_constraints(&self, signals: Vec<StrategySignal>) -> Result<Vec<StrategySignal>> {
        let mut filtered_signals = Vec::new();
        
        // Check maximum concurrent strategies limit
        let active_strategies: std::collections::HashSet<_> = signals.iter()
            .map(|s| s.strategy_name.as_str())
            .collect();
        
        if active_strategies.len() > self.max_concurrent_strategies {
            // Sort by confidence and take top strategies
            let mut sorted_signals = signals;
            sorted_signals.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
            
            let mut selected_strategies = std::collections::HashSet::new();
            for signal in sorted_signals {
                if selected_strategies.len() < self.max_concurrent_strategies {
                    selected_strategies.insert(signal.strategy_name.clone());
                    filtered_signals.push(signal);
                }
            }
        } else {
            filtered_signals = signals;
        }
        
        // Apply additional risk constraints
        let total_risk_exposure = filtered_signals.len() as f64 / self.max_concurrent_strategies as f64;
        if total_risk_exposure > self.max_total_risk_exposure {
            warn!("Total risk exposure {:.2}% exceeds maximum {:.2}% - filtering signals", 
                  total_risk_exposure * 100.0, self.max_total_risk_exposure * 100.0);
            
            // Reduce signals to meet risk exposure limits
            let max_signals = (self.max_total_risk_exposure * self.max_concurrent_strategies as f64) as usize;
            filtered_signals.truncate(max_signals);
        }
        
        info!("Applied portfolio constraints: {} signals passed filters", filtered_signals.len());
        
        Ok(filtered_signals)
    }
    
    /// Update performance across all strategies
    pub fn update_global_performance(&mut self, trade_result: &TradeResult, strategy_name: &str) -> Result<()> {
        // Update individual strategy performance
        if let Some(strategy) = self.strategies.get_mut(strategy_name) {
            strategy.update_performance(trade_result)?;
        }
        
        // Update global performance
        self.global_performance.total_trades += 1;
        self.global_performance.total_profit_loss += trade_result.profit_loss;
        self.global_performance.total_fees += trade_result.fees;
        
        // Update daily loss tracking
        if trade_result.profit_loss < 0.0 {
            self.current_daily_loss += trade_result.profit_loss / self.total_capital;
        }
        
        if trade_result.profit_loss > 0.0 {
            self.global_performance.winning_trades += 1;
        } else {
            self.global_performance.losing_trades += 1;
        }
        
        // Calculate metrics
        self.global_performance.win_rate = self.global_performance.winning_trades as f64 / self.global_performance.total_trades as f64;
        self.global_performance.last_updated = Utc::now();
        
        info!("Updated global performance: {} total trades, {:.2}% win rate, {:.2} total P&L", 
              self.global_performance.total_trades, 
              self.global_performance.win_rate * 100.0,
              self.global_performance.total_profit_loss);
        
        Ok(())
    }
    
    /// Get comprehensive performance report
    pub fn get_performance_report(&self) -> HashMap<String, StrategyPerformance> {
        let mut report = HashMap::new();
        
        // Add global performance
        report.insert("portfolio_manager".to_string(), self.global_performance.clone());
        
        // Add individual strategy performance
        for (name, strategy) in &self.strategies {
            report.insert(name.clone(), strategy.performance().clone());
        }
        
        report
    }
    
    /// Reset daily loss tracking (should be called daily)
    pub fn reset_daily_tracking(&mut self) {
        self.current_daily_loss = 0.0;
        info!("Reset daily loss tracking for new trading day");
    }
    
    /// Get strategy allocations
    pub fn get_strategy_allocations(&self) -> &HashMap<String, f64> {
        &self.allocated_capital
    }
    
    /// Rebalance portfolio based on performance
    pub async fn rebalance_portfolio(&mut self) -> Result<()> {
        info!("Initiating portfolio rebalancing...");
        
        // Calculate performance-based weights
        let mut new_weights = HashMap::new();
        let mut total_performance_score = 0.0;
        
        for (strategy_name, strategy) in &self.strategies {
            let performance = strategy.performance();
            
            // Calculate performance score (combination of win rate and profit)
            let performance_score = if performance.total_trades > 10 {
                performance.win_rate * 0.6 + 
                (performance.total_profit_loss / self.total_capital).max(-1.0).min(1.0) * 0.4
            } else {
                0.33 // Default equal weight for new strategies
            };
            
            new_weights.insert(strategy_name.clone(), performance_score.max(0.1)); // Minimum 10% weight
            total_performance_score += performance_score.max(0.1);
        }
        
        // Normalize weights
        for (strategy_name, weight) in &mut new_weights {
            *weight /= total_performance_score;
            
            // Update allocations
            self.strategy_weights.insert(strategy_name.clone(), *weight);
            self.allocated_capital.insert(strategy_name.clone(), self.total_capital * (*weight));
        }
        
        info!("✅ Portfolio rebalanced with new allocations: {:?}", self.strategy_weights);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimpleConfig;
    use std::collections::HashMap;

    fn create_test_config() -> SimpleConfig {
        SimpleConfig {
            portfolio_value: Some(10000.0),
            enable_ml_analysis: Some(true),
            max_concurrent_trades: Some(5),
            ..Default::default()
        }
    }

    #[test]
    fn test_strategy_manager_creation() {
        let config = create_test_config();
        let manager = StrategyManager::new(config);
        
        assert_eq!(manager.total_capital, 10000.0);
        assert_eq!(manager.max_concurrent_strategies, 3);
        assert_eq!(manager.strategies.len(), 0); // Before initialization
    }

    #[tokio::test]
    async fn test_strategy_initialization() {
        let config = create_test_config();
        let mut manager = StrategyManager::new(config);
        
        let result = manager.initialize_strategies();
        assert!(result.is_ok());
        assert_eq!(manager.strategies.len(), 3); // Should have 3 strategies
        
        // Verify allocations sum to 100%
        let total_allocation: f64 = manager.strategy_weights.values().sum();
        assert!((total_allocation - 1.0).abs() < 0.01); // Should be close to 1.0
    }

    #[test]
    fn test_performance_tracking() {
        let config = create_test_config();
        let mut manager = StrategyManager::new(config);
        
        let trade_result = TradeResult {
            profit_loss: 100.0,
            fees: 5.0,
            ..Default::default()
        };
        
        let result = manager.update_global_performance(&trade_result, "test_strategy");
        assert!(result.is_ok());
        
        assert_eq!(manager.global_performance.total_trades, 1);
        assert_eq!(manager.global_performance.winning_trades, 1);
        assert_eq!(manager.global_performance.total_profit_loss, 100.0);
    }

    #[test]
    fn test_daily_loss_tracking() {
        let config = create_test_config();
        let mut manager = StrategyManager::new(config);
        
        // Simulate a large loss
        let trade_result = TradeResult {
            profit_loss: -1500.0, // 15% loss
            fees: 5.0,
            ..Default::default()
        };
        
        manager.update_global_performance(&trade_result, "test_strategy").unwrap();
        
        // Should trigger emergency stop
        assert!(manager.current_daily_loss <= manager.emergency_stop_threshold);
    }
}
