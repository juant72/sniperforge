use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use anyhow::Result;

pub mod trend_following;
pub mod mean_reversion;
pub mod arbitrage;
pub mod momentum;

use crate::shared::pool_detector::{TradingOpportunity, OpportunityType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub name: String,
    pub enabled: bool,
    pub capital_allocation: f64, // Percentage of total capital (0.0 to 1.0)
    pub risk_level: RiskLevel,
    pub max_position_size: f64,
    pub stop_loss_percent: f64,
    pub take_profit_percent: f64,
    pub min_confidence: f64,
    pub timeframes: Vec<Timeframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Conservative,
    Moderate,
    Aggressive,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum Timeframe {
    OneMin,
    FiveMin,
    FifteenMin,
    OneHour,
    FourHour,
    OneDay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySignal {
    pub strategy_name: String,
    pub signal_type: SignalType,
    pub confidence: f64,
    pub timeframe: Timeframe,
    pub entry_price: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub position_size: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalType {
    Buy,
    Sell,
    Hold,
    StopLoss,
    TakeProfit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub strategy_name: String,
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub total_profit_loss: f64,
    pub win_rate: f64,
    pub average_profit: f64,
    pub average_loss: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub total_fees: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

pub trait TradingStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn analyze(&self, opportunity: &TradingOpportunity, market_data: &MarketData) -> Result<Option<StrategySignal>>;
    fn update_performance(&mut self, trade_result: &TradeResult) -> Result<()>;
    fn get_performance(&self) -> &StrategyPerformance;
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
    fn get_config(&self) -> &StrategyConfig;
}

#[derive(Debug, Clone)]
pub struct MarketData {
    pub current_price: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub liquidity: f64,
    pub bid_ask_spread: f64,
    pub order_book_depth: f64,
    pub price_history: Vec<PricePoint>,
    pub volume_history: Vec<VolumePoint>,
}

#[derive(Debug, Clone)]
pub struct PricePoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone)]
pub struct VolumePoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub volume: f64,
}

#[derive(Debug, Clone)]
pub struct TradeResult {
    pub strategy_name: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub position_size: f64,
    pub profit_loss: f64,
    pub fees: f64,
    pub duration: chrono::Duration,
    pub success: bool,
}

pub struct MultiStrategyEngine {
    strategies: Vec<Box<dyn TradingStrategy>>,
    performance_tracker: Arc<RwLock<HashMap<String, StrategyPerformance>>>,
    active_signals: Arc<RwLock<Vec<StrategySignal>>>,
    config: MultiStrategyConfig,
}

#[derive(Debug, Clone)]
pub struct MultiStrategyConfig {
    pub max_concurrent_strategies: usize,
    pub max_total_capital_allocation: f64,
    pub rebalance_frequency: chrono::Duration,
    pub performance_tracking_enabled: bool,
    pub signal_aggregation_method: SignalAggregationMethod,
}

#[derive(Debug, Clone)]
pub enum SignalAggregationMethod {
    Consensus,      // Require majority agreement
    WeightedAverage, // Weight by confidence and past performance
    BestPerformer,  // Use signal from best performing strategy
    DiversifiedRisk, // Diversify across multiple signals
}

impl MultiStrategyEngine {
    pub fn new(config: MultiStrategyConfig) -> Self {
        Self {
            strategies: Vec::new(),
            performance_tracker: Arc::new(RwLock::new(HashMap::new())),
            active_signals: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn TradingStrategy>) -> Result<()> {
        if self.strategies.len() >= self.config.max_concurrent_strategies {
            return Err(anyhow::anyhow!("Maximum number of strategies reached"));
        }

        let strategy_name = strategy.name().to_string();
        self.strategies.push(strategy);
        
        println!("✅ Added strategy: {}", strategy_name);
        Ok(())
    }

    pub async fn analyze_opportunity(&self, opportunity: &TradingOpportunity, market_data: &MarketData) -> Result<Vec<StrategySignal>> {
        let mut signals = Vec::new();
        
        for strategy in &self.strategies {
            if !strategy.is_enabled() {
                continue;
            }

            match strategy.analyze(opportunity, market_data) {
                Ok(Some(signal)) => {
                    println!("📊 Strategy '{}' generated signal: {:?}", strategy.name(), signal.signal_type);
                    signals.push(signal);
                }
                Ok(None) => {
                    // No signal generated, continue
                }
                Err(e) => {
                    eprintln!("⚠️ Error in strategy '{}': {}", strategy.name(), e);
                }
            }
        }

        // Store active signals
        let mut active_signals = self.active_signals.write().await;
        active_signals.extend(signals.clone());
        
        // Keep only recent signals (last hour)
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(1);
        active_signals.retain(|s| s.timestamp > cutoff);

        Ok(signals)
    }

    pub async fn aggregate_signals(&self, signals: &[StrategySignal]) -> Result<Option<StrategySignal>> {
        if signals.is_empty() {
            return Ok(None);
        }

        match self.config.signal_aggregation_method {
            SignalAggregationMethod::Consensus => self.consensus_aggregation(signals).await,
            SignalAggregationMethod::WeightedAverage => self.weighted_average_aggregation(signals).await,
            SignalAggregationMethod::BestPerformer => self.best_performer_aggregation(signals).await,
            SignalAggregationMethod::DiversifiedRisk => self.diversified_risk_aggregation(signals).await,
        }
    }

    async fn consensus_aggregation(&self, signals: &[StrategySignal]) -> Result<Option<StrategySignal>> {
        let mut buy_count = 0;
        let mut sell_count = 0;
        let mut total_confidence = 0.0;

        for signal in signals {
            match signal.signal_type {
                SignalType::Buy => buy_count += 1,
                SignalType::Sell => sell_count += 1,
                _ => {}
            }
            total_confidence += signal.confidence;
        }

        let majority_threshold = signals.len() / 2 + 1;
        
        if buy_count >= majority_threshold {
            Ok(Some(StrategySignal {
                strategy_name: "CONSENSUS".to_string(),
                signal_type: SignalType::Buy,
                confidence: total_confidence / signals.len() as f64,
                timeframe: signals[0].timeframe,
                entry_price: signals.iter().map(|s| s.entry_price).sum::<f64>() / signals.len() as f64,
                stop_loss: None,
                take_profit: None,
                position_size: signals.iter().map(|s| s.position_size).sum::<f64>() / signals.len() as f64,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            }))
        } else if sell_count >= majority_threshold {
            Ok(Some(StrategySignal {
                strategy_name: "CONSENSUS".to_string(),
                signal_type: SignalType::Sell,
                confidence: total_confidence / signals.len() as f64,
                timeframe: signals[0].timeframe,
                entry_price: signals.iter().map(|s| s.entry_price).sum::<f64>() / signals.len() as f64,
                stop_loss: None,
                take_profit: None,
                position_size: signals.iter().map(|s| s.position_size).sum::<f64>() / signals.len() as f64,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            }))
        } else {
            Ok(None) // No consensus
        }
    }

    async fn weighted_average_aggregation(&self, signals: &[StrategySignal]) -> Result<Option<StrategySignal>> {
        let performance_tracker = self.performance_tracker.read().await;
        let mut weighted_signals: Vec<(f64, &StrategySignal)> = Vec::new();

        for signal in signals {
            let weight = if let Some(perf) = performance_tracker.get(&signal.strategy_name) {
                perf.win_rate * signal.confidence
            } else {
                signal.confidence // Default weight if no performance data
            };
            weighted_signals.push((weight, signal));
        }

        if weighted_signals.is_empty() {
            return Ok(None);
        }

        // Sort by weight (highest first)
        weighted_signals.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        // Use the highest weighted signal
        let best_signal = weighted_signals[0].1;
        Ok(Some(best_signal.clone()))
    }

    async fn best_performer_aggregation(&self, signals: &[StrategySignal]) -> Result<Option<StrategySignal>> {
        let performance_tracker = self.performance_tracker.read().await;
        
        let mut best_signal: Option<&StrategySignal> = None;
        let mut best_performance = 0.0;

        for signal in signals {
            let performance_score = if let Some(perf) = performance_tracker.get(&signal.strategy_name) {
                perf.win_rate * perf.sharpe_ratio.max(0.0)
            } else {
                signal.confidence // Fallback to confidence if no performance data
            };

            if performance_score > best_performance {
                best_performance = performance_score;
                best_signal = Some(signal);
            }
        }

        Ok(best_signal.cloned())
    }

    async fn diversified_risk_aggregation(&self, signals: &[StrategySignal]) -> Result<Option<StrategySignal>> {
        // Select a mix of signals to diversify risk
        if signals.len() <= 2 {
            return self.weighted_average_aggregation(signals).await;
        }

        // Take signals from different strategy types
        let mut selected_signals = Vec::new();
        let mut used_strategies = std::collections::HashSet::new();

        for signal in signals {
            if !used_strategies.contains(&signal.strategy_name) && selected_signals.len() < 3 {
                selected_signals.push(signal);
                used_strategies.insert(signal.strategy_name.clone());
            }
        }

        if selected_signals.is_empty() {
            return Ok(None);
        }

        // Create averaged signal from selected strategies
        let avg_confidence = selected_signals.iter().map(|s| s.confidence).sum::<f64>() / selected_signals.len() as f64;
        let avg_price = selected_signals.iter().map(|s| s.entry_price).sum::<f64>() / selected_signals.len() as f64;
        let avg_position = selected_signals.iter().map(|s| s.position_size).sum::<f64>() / selected_signals.len() as f64;

        Ok(Some(StrategySignal {
            strategy_name: "DIVERSIFIED".to_string(),
            signal_type: selected_signals[0].signal_type.clone(),
            confidence: avg_confidence,
            timeframe: selected_signals[0].timeframe,
            entry_price: avg_price,
            stop_loss: None,
            take_profit: None,
            position_size: avg_position,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }))
    }

    pub async fn get_strategy_performance(&self, strategy_name: &str) -> Option<StrategyPerformance> {
        let performance_tracker = self.performance_tracker.read().await;
        performance_tracker.get(strategy_name).cloned()
    }

    pub async fn get_all_performance(&self) -> HashMap<String, StrategyPerformance> {
        let performance_tracker = self.performance_tracker.read().await;
        performance_tracker.clone()
    }

    pub async fn update_strategy_performance(&self, strategy_name: &str, trade_result: &TradeResult) -> Result<()> {
        let mut performance_tracker = self.performance_tracker.write().await;
        
        let performance = performance_tracker
            .entry(strategy_name.to_string())
            .or_insert(StrategyPerformance {
                strategy_name: strategy_name.to_string(),
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
            });

        performance.total_trades += 1;
        performance.total_profit_loss += trade_result.profit_loss;
        performance.total_fees += trade_result.fees;

        if trade_result.success {
            performance.winning_trades += 1;
        } else {
            performance.losing_trades += 1;
        }

        performance.win_rate = performance.winning_trades as f64 / performance.total_trades as f64;
        performance.last_updated = chrono::Utc::now();

        println!("📈 Updated performance for {}: Win Rate: {:.2}%, Total P&L: ${:.2}", 
            strategy_name, performance.win_rate * 100.0, performance.total_profit_loss);

        Ok(())
    }

    pub fn get_enabled_strategies(&self) -> Vec<String> {
        self.strategies
            .iter()
            .filter(|s| s.is_enabled())
            .map(|s| s.name().to_string())
            .collect()
    }

    pub fn get_strategy_count(&self) -> usize {
        self.strategies.len()
    }

    pub fn get_enabled_strategy_count(&self) -> usize {
        self.strategies.iter().filter(|s| s.is_enabled()).count()
    }
}

impl Default for MultiStrategyConfig {
    fn default() -> Self {
        Self {
            max_concurrent_strategies: 10,
            max_total_capital_allocation: 1.0,
            rebalance_frequency: chrono::Duration::hours(1),
            performance_tracking_enabled: true,
            signal_aggregation_method: SignalAggregationMethod::WeightedAverage,
        }
    }
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            name: "Default Strategy".to_string(),
            enabled: true,
            capital_allocation: 0.2, // 20% of total capital
            risk_level: RiskLevel::Moderate,
            max_position_size: 1000.0,
            stop_loss_percent: 5.0,
            take_profit_percent: 10.0,
            min_confidence: 0.6,
            timeframes: vec![Timeframe::FiveMin, Timeframe::FifteenMin],
        }
    }
}
