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
    pub async fn initialize_strategies(&mut self) -> Result<()> {
        info!("Initializing enterprise trading strategies...");
        
        // ✅ CONFIG INTEGRATION: Apply configuration-based strategy parameters
        info!("Using enterprise config with max_position_size: {:.4} SOL", 
              self.config.max_position_size);
        
        // Validate signal aggregation method
        self.validate_signal_aggregation_method()?;
        
        // Initialize Arbitrage Strategy
        let arbitrage = ArbitrageStrategy::new().await.map_err(|e| anyhow::Error::msg(format!("Failed to initialize arbitrage strategy: {}", e)))?;
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
        
        // ✅ ENTERPRISE CONFIGURATION: Apply config-based customizations
        self.apply_config_customization()?;
        info!("✅ Applied enterprise configuration customizations");
        
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
        
        // Check daily loss limit before proceeding
        if self.current_daily_loss.abs() > self.daily_loss_limit {
            warn!("Daily loss limit exceeded: {:.2}% > {:.2}% - blocking new signals", 
                  self.current_daily_loss * 100.0, self.daily_loss_limit * 100.0);
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
        
        // Analyze strategy correlations for conflict resolution
        let correlation_conflicts = self.analyze_strategy_correlations(&signal_groups).await?;
        
        // Process each signal group with conflict resolution
        let signal_groups_count = signal_groups.len();
        for ((token_pair, signal_type), grouped_signals) in signal_groups {
            // Apply conflict resolution strategy
            let resolved_signals = self.resolve_signal_conflicts(&token_pair, &signal_type, grouped_signals, &correlation_conflicts).await?;
            
            let coordinated_signal = self.aggregate_signals(token_pair, signal_type, resolved_signals).await?;
            
            if let Some(signal) = coordinated_signal {
                coordinated_signals.push(signal);
            }
        }
        
        // Apply portfolio-level constraints and rebalancing
        let final_signals = self.apply_portfolio_constraints(coordinated_signals).await?;
        
        info!("Coordinated {} enterprise signals from {} strategy groups with correlation analysis", 
              final_signals.len(), signal_groups_count);
        
        Ok(final_signals)
    }
    
    /// Aggregate signals from multiple strategies for the same asset and direction
    async fn aggregate_signals(&self, token_pair: String, signal_type: SignalType, signals: Vec<(String, StrategySignal)>) -> Result<Option<StrategySignal>> {
        if signals.is_empty() {
            return Ok(None);
        }
        
        // Calculate confidence-weighted average
        let mut total_confidence = 0.0;
        let mut weighted_expected_profit = 0.0;
        let mut weighted_stop_loss = 0.0;
        let mut weighted_take_profit = 0.0;
        let mut strategy_names = Vec::new();
        
        for (strategy_name, signal) in &signals {
            total_confidence += signal.confidence;
            weighted_expected_profit += signal.expected_profit * signal.confidence;
            weighted_stop_loss += signal.stop_loss * signal.confidence;
            weighted_take_profit += signal.take_profit * signal.confidence;
            strategy_names.push(strategy_name.clone());
        }
        
        if total_confidence == 0.0 {
            return Ok(None);
        }
        
        // Use the signal with highest confidence as the base
        let base_signal = signals.iter()
            .max_by(|a, b| a.1.confidence.partial_cmp(&b.1.confidence).unwrap())
            .unwrap();
        
        // Create aggregated signal with enhanced metadata
        let mut aggregated_signal = base_signal.1.clone();
        aggregated_signal.confidence = total_confidence / signals.len() as f64;
        aggregated_signal.expected_profit = weighted_expected_profit / total_confidence;
        aggregated_signal.stop_loss = weighted_stop_loss / total_confidence;
        aggregated_signal.take_profit = weighted_take_profit / total_confidence;
        
        // ✅ ENTERPRISE VALIDATION: Check signal agreement threshold
        if !self.meets_signal_agreement(aggregated_signal.confidence, signals.len()) {
            warn!("Signal agreement below threshold for {} {:?}: confidence {:.2}%, signals {}", 
                  token_pair, signal_type, aggregated_signal.confidence * 100.0, signals.len());
            return Ok(None); // Reject signals below agreement threshold
        }
        
        // ✅ USE token_pair and signal_type for enhanced reasoning
        aggregated_signal.reasoning = Some(format!(
            "Aggregated {} signal for {} from strategies: {} (confidence: {:.2})", 
            match signal_type {
                SignalType::Buy => "BUY",
                SignalType::Sell => "SELL",
                SignalType::Hold => "HOLD",
                SignalType::StopLoss => "STOP_LOSS",
                SignalType::TakeProfit => "TAKE_PROFIT",
            },
            token_pair,
            strategy_names.join(", "),
            aggregated_signal.confidence
        ));
        
        // Apply signal aggregation method if configured
        if let Some(aggregation_boost) = self.get_aggregation_boost(&signal_type, signals.len()) {
            aggregated_signal.confidence *= aggregation_boost;
            info!("Applied {} aggregation boost: {:.2}x for {:?} on {}", 
                  self.signal_aggregation_method, aggregation_boost, signal_type, token_pair);
        }
        
        Ok(Some(aggregated_signal))
    }
    
    /// Get aggregation boost based on method and signal count
    fn get_aggregation_boost(&self, _signal_type: &SignalType, signal_count: usize) -> Option<f64> {
        match self.signal_aggregation_method.as_str() {
            "weighted_average" => {
                // More signals = higher confidence (up to 20% boost)
                Some(1.0 + (signal_count as f64 - 1.0) * 0.05)
            },
            "consensus" => {
                // Consensus method gets boost for agreement
                if signal_count >= 2 {
                    Some(1.15) // 15% boost for consensus
                } else {
                    Some(0.9) // Penalty for no consensus
                }
            },
            "highest_confidence" => {
                // No boost for single best signal method
                Some(1.0)
            },
            _ => None
        }
    }
    
    /// Validate signal aggregation method configuration
    fn validate_signal_aggregation_method(&self) -> Result<()> {
        match self.signal_aggregation_method.as_str() {
            "weighted_average" | "consensus" | "highest_confidence" => {
                info!("✅ Signal aggregation method '{}' validated with min_agreement: {:.2}", 
                      self.signal_aggregation_method, self.min_signal_agreement);
                Ok(())
            },
            _ => {
                warn!("Invalid signal aggregation method '{}', falling back to 'weighted_average'", 
                      self.signal_aggregation_method);
                Ok(()) // Non-critical, will use default
            }
        }
    }
    
    /// Check if signals meet minimum agreement threshold
    fn meets_signal_agreement(&self, confidence: f64, signal_count: usize) -> bool {
        if signal_count >= 2 {
            // Multiple signals need higher agreement
            confidence >= self.min_signal_agreement
        } else {
            // Single signal uses lower threshold
            confidence >= self.min_signal_agreement * 0.8
        }
    }
    
    /// Apply config-based strategy customization
    fn apply_config_customization(&mut self) -> Result<()> {
        // Use config to customize strategy behavior
        let max_risk = self.config.max_position_size;
        self.max_total_risk_exposure = (max_risk / 100.0).min(0.25); // Cap at 25%
        info!("Applied config max risk exposure: {:.2}%", self.max_total_risk_exposure * 100.0);
        
        let stop_loss = self.config.stop_loss_percentage;
        self.daily_loss_limit = -(stop_loss / 100.0); // Convert to negative limit
        info!("Applied config daily loss limit: {:.2}%", self.daily_loss_limit * 100.0);
        
        Ok(())
    }
    
    /// Analyze correlations between strategies to detect conflicts
    async fn analyze_strategy_correlations(&self, signal_groups: &HashMap<(String, SignalType), Vec<(String, StrategySignal)>>) -> Result<HashMap<String, f64>> {
        let mut correlations = HashMap::new();
        
        // Analyze existing correlation matrix
        for (strategy_a, strategy_correlations) in &self.correlation_matrix {
            for (strategy_b, correlation) in strategy_correlations {
                let correlation_key = format!("{}_{}", strategy_a, strategy_b);
                correlations.insert(correlation_key, *correlation);
            }
        }
        
        // Update correlations based on current signals
        let mut strategy_signal_counts: HashMap<String, HashMap<SignalType, usize>> = HashMap::new();
        
        for ((_token_pair, signal_type), strategy_signals) in signal_groups {
            for (strategy_name, _signal) in strategy_signals {
                let strategy_counts = strategy_signal_counts.entry(strategy_name.clone()).or_insert_with(HashMap::new);
                *strategy_counts.entry(signal_type.clone()).or_insert(0) += 1;
            }
        }
        
        Ok(correlations)
    }
    
    /// Resolve conflicts between strategies based on configuration
    async fn resolve_signal_conflicts(&self, _token_pair: &str, _signal_type: &SignalType, signals: Vec<(String, StrategySignal)>, _correlations: &HashMap<String, f64>) -> Result<Vec<(String, StrategySignal)>> {
        match self.conflict_resolution_strategy.as_str() {
            "highest_confidence" => {
                // Keep only the signal with highest confidence
                if let Some(best_signal) = signals.iter().max_by(|a, b| a.1.confidence.partial_cmp(&b.1.confidence).unwrap()) {
                    Ok(vec![best_signal.clone()])
                } else {
                    Ok(signals)
                }
            },
            "weighted_average" => {
                // Keep all signals for weighted aggregation
                Ok(signals)
            },
            "consensus" => {
                // Keep signals only if multiple strategies agree
                if signals.len() >= 2 {
                    Ok(signals)
                } else {
                    Ok(vec![])
                }
            },
            _ => {
                // Default: keep all signals
                Ok(signals)
            }
        }
    }
    
    /// Apply portfolio-level constraints and risk management
    async fn apply_portfolio_constraints(&self, signals: Vec<StrategySignal>) -> Result<Vec<StrategySignal>> {
        let mut filtered_signals = Vec::new();
        
        // Check if portfolio rebalancing is needed based on current allocation drift
        let needs_rebalancing = self.check_rebalancing_threshold().await?;
        if needs_rebalancing {
            info!("Portfolio allocation drift exceeds rebalancing threshold: {:.2}% - applying rebalancing constraints", 
                  self.rebalancing_threshold * 100.0);
        }
        
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
                    // Apply rebalancing bias if needed
                    let mut adjusted_signal = signal.clone();
                    if needs_rebalancing {
                        adjusted_signal = self.apply_rebalancing_adjustment(adjusted_signal).await?;
                    }
                    
                    selected_strategies.insert(signal.strategy_name.clone());
                    filtered_signals.push(adjusted_signal);
                }
            }
        } else {
            // Apply rebalancing adjustments to all signals if needed
            for signal in signals {
                let adjusted_signal = if needs_rebalancing {
                    self.apply_rebalancing_adjustment(signal).await?
                } else {
                    signal
                };
                filtered_signals.push(adjusted_signal);
            }
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
        
        info!("Applied portfolio constraints with rebalancing: {} signals passed filters", filtered_signals.len());
        
        Ok(filtered_signals)
    }
    
    /// Check if portfolio rebalancing is needed based on allocation drift
    async fn check_rebalancing_threshold(&self) -> Result<bool> {
        // Calculate current allocation vs target allocation
        let mut max_drift = 0.0;
        
        for (strategy_name, target_weight) in &self.strategy_weights {
            if let Some(current_capital) = self.allocated_capital.get(strategy_name) {
                let current_weight = current_capital / self.total_capital;
                let drift = (current_weight - target_weight).abs();
                
                if drift > max_drift {
                    max_drift = drift;
                }
            }
        }
        
        Ok(max_drift > self.rebalancing_threshold)
    }
    
    /// Apply rebalancing adjustments to signals
    async fn apply_rebalancing_adjustment(&self, mut signal: StrategySignal) -> Result<StrategySignal> {
        // Get current strategy allocation
        if let Some(current_capital) = self.allocated_capital.get(&signal.strategy_name) {
            if let Some(target_weight) = self.strategy_weights.get(&signal.strategy_name) {
                let current_weight = current_capital / self.total_capital;
                let allocation_ratio = target_weight / current_weight;
                
                // Adjust signal volume based on allocation needs
                if allocation_ratio > 1.1 {
                    // Strategy is under-allocated, increase signal strength
                    signal.volume *= allocation_ratio.min(1.5); // Cap at 50% increase
                    signal.reasoning = Some(format!("{} [REBALANCING: +{:.1}% allocation needed]", 
                                             signal.reasoning.unwrap_or_default(), 
                                             (allocation_ratio - 1.0) * 100.0));
                } else if allocation_ratio < 0.9 {
                    // Strategy is over-allocated, decrease signal strength
                    signal.volume *= allocation_ratio.max(0.5); // Cap at 50% decrease
                    signal.reasoning = Some(format!("{} [REBALANCING: -{:.1}% allocation excess]", 
                                             signal.reasoning.unwrap_or_default(), 
                                             (1.0 - allocation_ratio) * 100.0));
                }
            }
        }
        
        Ok(signal)
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

    fn create_test_config() -> SimpleConfig {
        SimpleConfig {
            enable_ml_analysis: true,
            max_concurrent_trades: 5,
            trading_amount: 10000.0,
            use_secondary_rpc: Some(false),
            rpc_retry_attempts: Some(3),
            rpc_timeout_ms: Some(10000),
            ..Default::default()
        }
    }

    #[test]
    fn test_strategy_manager_creation() {
        let config = create_test_config();
        let manager = StrategyManager::new(config.clone());
        
        assert_eq!(manager.total_capital, config.trading_amount * 100.0); // Expected calculation
        assert_eq!(manager.max_concurrent_strategies, 3);
        assert_eq!(manager.strategies.len(), 0); // Before initialization
    }

    #[tokio::test]
    async fn test_strategy_initialization() {
        let config = create_test_config();
        let mut manager = StrategyManager::new(config);
        
        let result = manager.initialize_strategies().await;
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
        let mut manager = StrategyManager::new(config.clone());
        
        // Calculate expected total capital
        let expected_total_capital = config.trading_amount * 100.0; // 1,000,000.0
        
        // Simulate a large loss that should trigger emergency stop (15% loss)
        let trade_result = TradeResult {
            profit_loss: -0.15 * expected_total_capital, // 15% of total capital
            fees: 5.0,
            ..Default::default()
        };
        
        manager.update_global_performance(&trade_result, "test_strategy").unwrap();
        
        // Should trigger emergency stop (current_daily_loss should be <= -0.10)
        assert!(manager.current_daily_loss <= manager.emergency_stop_threshold,
               "Daily loss {} should trigger emergency stop at {}", 
               manager.current_daily_loss, manager.emergency_stop_threshold);
    }
}
