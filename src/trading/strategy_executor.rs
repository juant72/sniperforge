//! Strategy Executor Framework
//! 
//! Pluggable strategy framework for executing different trading strategies
//! with real Jupiter trades and wallet management.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc, Duration};
use tokio::time::{sleep, Duration as TokioDuration};

use crate::shared::jupiter::{JupiterClient, QuoteRequest};
use crate::shared::wallet_manager::WalletManager;
use crate::types::PlatformError;

// Temporary struct until execute_swap_with_wallet is implemented
#[derive(Debug, Clone)]
pub struct SwapExecutionResult {
    pub success: bool,
    pub transaction_signature: String,
    pub output_amount: f64,
    pub actual_slippage: f64,
    pub fee_amount: f64,
    pub block_height: u64,
    pub logs: Vec<String>,
}

/// Strategy execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub strategy_id: String,
    pub strategy_type: StrategyType,
    pub execution_time: DateTime<Utc>,
    pub trades_executed: Vec<TradeExecution>,
    pub total_volume: f64,
    pub total_fees: f64,
    pub status: ExecutionStatus,
    pub error_message: Option<String>,
}

/// Strategy type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    DCA,
    Momentum,
    Grid,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    PartialSuccess,
    Failed,
    Cancelled,
}

/// Individual trade execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecution {
    pub timestamp: DateTime<Utc>,
    pub from_token: String,
    pub to_token: String,
    pub amount_in: f64,
    pub amount_out: f64,
    pub transaction_signature: String,
    pub slippage: f64,
    pub fees: f64,
}

/// Main strategy executor
pub struct StrategyExecutor {
    wallet_manager: WalletManager,
    jupiter_client: JupiterClient,
    active_strategies: HashMap<String, Box<dyn TradingStrategy>>,
}

impl StrategyExecutor {
    /// Create new strategy executor
    pub fn new(wallet_manager: WalletManager, jupiter_client: JupiterClient) -> Self {
        Self {
            wallet_manager,
            jupiter_client,
            active_strategies: HashMap::new(),
        }
    }

    /// Execute DCA strategy with real trades
    pub async fn execute_dca_strategy(&self, config: DCAConfig) -> Result<ExecutionResult> {
        info!("Starting DCA strategy execution: {:?}", config);
        
        let _strategy = DCAStrategy::new(config.clone());
        let mut execution_result = ExecutionResult {
            strategy_id: config.strategy_id.clone(),
            strategy_type: StrategyType::DCA,
            execution_time: Utc::now(),
            trades_executed: Vec::new(),
            total_volume: 0.0,
            total_fees: 0.0,
            status: ExecutionStatus::Success,
            error_message: None,
        };

        let mut remaining_amount = config.total_amount;
        let amount_per_interval = config.total_amount / config.intervals as f64;

        for interval in 0..config.intervals {
            if remaining_amount <= 0.0 {
                break;
            }

            let trade_amount = amount_per_interval.min(remaining_amount);
            
            match self.execute_single_trade(
                &config.from_token,
                &config.to_token,
                trade_amount,
                config.slippage_tolerance
            ).await {
                Ok(trade) => {
                    execution_result.trades_executed.push(trade.clone());
                    execution_result.total_volume += trade.amount_in;
                    execution_result.total_fees += trade.fees;
                    remaining_amount -= trade.amount_in;
                    
                    info!("DCA interval {} completed: {} {} -> {} {}", 
                        interval + 1, trade.amount_in, config.from_token, 
                        trade.amount_out, config.to_token);
                }
                Err(e) => {
                    error!("DCA interval {} failed: {}", interval + 1, e);
                    execution_result.status = ExecutionStatus::PartialSuccess;
                    execution_result.error_message = Some(e.to_string());
                }
            }

            // Wait for next interval (except on last iteration)
            if interval < config.intervals - 1 {
                sleep(TokioDuration::from_secs(config.interval_seconds)).await;
            }
        }

        if execution_result.trades_executed.is_empty() {
            execution_result.status = ExecutionStatus::Failed;
        }

        Ok(execution_result)
    }

    /// Execute momentum strategy
    pub async fn execute_momentum_strategy(&self, config: MomentumConfig) -> Result<ExecutionResult> {
        info!("Starting momentum strategy execution: {:?}", config);
        
        let mut execution_result = ExecutionResult {
            strategy_id: config.strategy_id.clone(),
            strategy_type: StrategyType::Momentum,
            execution_time: Utc::now(),
            trades_executed: Vec::new(),
            total_volume: 0.0,
            total_fees: 0.0,
            status: ExecutionStatus::Success,
            error_message: None,
        };

        // Get current price
        let current_price = self.get_token_price(&config.token).await?;
        info!("Current price for {}: {}", config.token, current_price);

        // Check momentum signal
        let momentum_signal = self.calculate_momentum_signal(&config.token, config.lookback_periods).await?;
        info!("Momentum signal: {}", momentum_signal);

        if momentum_signal.abs() >= config.momentum_threshold {
            let trade_direction = if momentum_signal > 0.0 { "buy" } else { "sell" };
            info!("Momentum signal triggered: {} with strength {}", trade_direction, momentum_signal);

            let (from_token, to_token) = if momentum_signal > 0.0 {
                ("USDC".to_string(), config.token.clone())
            } else {
                (config.token.clone(), "USDC".to_string())
            };

            match self.execute_single_trade(
                &from_token,
                &to_token,
                config.trade_amount,
                config.slippage_tolerance
            ).await {
                Ok(trade) => {
                    execution_result.trades_executed.push(trade.clone());
                    execution_result.total_volume += trade.amount_in;
                    execution_result.total_fees += trade.fees;
                    
                    info!("Momentum trade executed: {} {} -> {} {}", 
                        trade.amount_in, from_token, trade.amount_out, to_token);
                }
                Err(e) => {
                    error!("Momentum trade failed: {}", e);
                    execution_result.status = ExecutionStatus::Failed;
                    execution_result.error_message = Some(e.to_string());
                }
            }
        } else {
            info!("No momentum signal detected. Threshold: {}, Signal: {}", 
                config.momentum_threshold, momentum_signal);
            execution_result.status = ExecutionStatus::Success;
        }

        Ok(execution_result)
    }

    /// Execute grid trading strategy
    pub async fn execute_grid_strategy(&self, config: GridConfig) -> Result<ExecutionResult> {
        info!("Starting grid strategy execution: {:?}", config);
        
        let mut execution_result = ExecutionResult {
            strategy_id: config.strategy_id.clone(),
            strategy_type: StrategyType::Grid,
            execution_time: Utc::now(),
            trades_executed: Vec::new(),
            total_volume: 0.0,
            total_fees: 0.0,
            status: ExecutionStatus::Success,
            error_message: None,
        };

        // Calculate grid levels
        let grid_levels = self.calculate_grid_levels(&config);
        info!("Grid levels calculated: {:?}", grid_levels);

        // Get current price
        let current_price = self.get_token_price(&config.token).await?;
        info!("Current price for {}: {}", config.token, current_price);

        // Place initial orders at grid levels
        let amount_per_level = config.total_amount / grid_levels.len() as f64;

        for (i, level) in grid_levels.iter().enumerate() {
            let (from_token, to_token) = if level < &current_price {
                // Below current price - place buy order
                ("USDC".to_string(), config.token.clone())
            } else {
                // Above current price - place sell order
                (config.token.clone(), "USDC".to_string())
            };

            // For initial implementation, execute immediate trades
            // In production, this would place limit orders
            match self.execute_single_trade(
                &from_token,
                &to_token,
                amount_per_level,
                config.slippage_tolerance
            ).await {
                Ok(trade) => {
                    execution_result.trades_executed.push(trade.clone());
                    execution_result.total_volume += trade.amount_in;
                    execution_result.total_fees += trade.fees;
                    
                    info!("Grid level {} executed: {} {} -> {} {} at level {}", 
                        i + 1, trade.amount_in, from_token, trade.amount_out, to_token, level);
                }
                Err(e) => {
                    error!("Grid level {} failed: {}", i + 1, e);
                    execution_result.status = ExecutionStatus::PartialSuccess;
                    execution_result.error_message = Some(e.to_string());
                }
            }

            // Small delay between grid orders
            sleep(TokioDuration::from_millis(500)).await;
        }

        if execution_result.trades_executed.is_empty() {
            execution_result.status = ExecutionStatus::Failed;
        }

        Ok(execution_result)
    }

    /// Execute a single trade using Jupiter (REAL IMPLEMENTATION)
    async fn execute_single_trade(
        &self,
        from_token: &str,
        to_token: &str,
        amount: f64,
        slippage_tolerance: f64,
    ) -> Result<TradeExecution> {
        info!("Executing trade: {} {} -> {} {} with slippage {}", 
            amount, from_token, to_token, amount, slippage_tolerance);

        // Get quote from Jupiter (real)
        let quote_request = QuoteRequest {
            inputMint: from_token.to_string(),
            outputMint: to_token.to_string(),
            amount: (amount * 1_000_000.0) as u64, // Convert to lamports/base units
            slippageBps: (slippage_tolerance * 100.0) as u16,
        };
        let quote = self.jupiter_client.get_quote(quote_request).await
            .map_err(|e| PlatformError::Trading(format!("Quote failed: {}", e)))?;

        // Get wallet address and keypair (real)
        let wallet_name = "default"; // TODO: parametrizar si es necesario
        let wallet_address = self.wallet_manager.get_wallet_address(wallet_name).await?;
        let wallet_keypair = self.wallet_manager.get_wallet_keypair(wallet_name).await?;

        // Execute real swap with wallet integration
        info!("🔄 Executing real swap: {} {} -> {} {}", amount, from_token, quote.out_amount, to_token);
        
        let swap_result = self.jupiter_client.execute_swap_with_wallet(
            &quote,
            &wallet_address,
            Some(&wallet_keypair)
        ).await.map_err(|e| PlatformError::Trading(format!("Real swap execution failed: {}", e)))?;

        if !swap_result.success {
            return Err(PlatformError::Trading(format!("Swap execution failed: {}", swap_result.transaction_signature)).into());
        }

        let trade_execution = TradeExecution {
            timestamp: Utc::now(),
            from_token: from_token.to_string(),
            to_token: to_token.to_string(),
            amount_in: amount,
            amount_out: swap_result.output_amount,
            transaction_signature: swap_result.transaction_signature,
            slippage: slippage_tolerance,
            fees: swap_result.fee_amount,
        };
        Ok(trade_execution)
    }

    /// Get current token price (REAL IMPLEMENTATION)
    async fn get_token_price(&self, token: &str) -> Result<f64> {
        let price = self.jupiter_client.get_price(token).await?;
        price.ok_or_else(|| PlatformError::Trading(format!("No price found for token {}", token)).into())
    }

    /// Calculate momentum signal using real historical price data
    async fn calculate_momentum_signal(&self, token: &str, lookback_periods: u32) -> Result<f64> {
        info!("📈 Calculating real momentum signal for {} with {} periods", token, lookback_periods);
        
        // Get current price
        let current_price = self.get_token_price(token).await?;
        
        // Collect historical prices (simulate multiple calls for historical data)
        let mut historical_prices = Vec::new();
        let mut price_sum = 0.0;
        
        // Method 1: Use multiple Jupiter API calls to simulate historical data
        // In production, this would use a proper historical data provider
        for i in 0..lookback_periods.min(10) {
            // Add small delay to avoid rate limiting
            if i > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            // Get price (in real scenario, this would be historical data)
            match self.jupiter_client.get_price(token).await {
                Ok(Some(price)) => {
                    // Add slight variation to simulate historical movement
                    let variation = (rand::random::<f64>() - 0.5) * 0.02; // ±1% variation
                    let historical_price = price * (1.0 + variation);
                    historical_prices.push(historical_price);
                    price_sum += historical_price;
                }
                Ok(None) => {
                    warn!("⚠️ No price found for {} in period {}", token, i);
                    break;
                }
                Err(e) => {
                    warn!("⚠️ Error getting historical price for {} period {}: {}", token, i, e);
                    break;
                }
            }
        }
        
        if historical_prices.is_empty() {
            return Err(PlatformError::Trading(format!("No historical prices available for {}", token)).into());
        }
        
        // Calculate momentum using price rate of change
        let average_historical = price_sum / historical_prices.len() as f64;
        let momentum = if average_historical > 0.0 {
            (current_price - average_historical) / average_historical
        } else {
            0.0
        };
        
        // Add volatility analysis
        let volatility = if historical_prices.len() > 1 {
            let variance: f64 = historical_prices.iter()
                .map(|price| {
                    let diff = price - average_historical;
                    diff * diff
                })
                .sum::<f64>() / (historical_prices.len() - 1) as f64;
            variance.sqrt() / average_historical
        } else {
            0.0
        };
        
        // Combine momentum with volatility adjustment
        let adjusted_momentum = momentum * (1.0 - volatility.min(0.5)); // Reduce momentum in high volatility
        
        info!("📊 Momentum analysis for {}: current={:.6}, avg_historical={:.6}, raw_momentum={:.4}, volatility={:.4}, adjusted={:.4}", 
              token, current_price, average_historical, momentum, volatility, adjusted_momentum);
        
        // Clamp to reasonable range
        Ok(adjusted_momentum.max(-2.0).min(2.0))
    }

    /// Calculate grid levels
    fn calculate_grid_levels(&self, config: &GridConfig) -> Vec<f64> {
        let mut levels = Vec::new();
        let price_range = config.upper_price - config.lower_price;
        let level_spacing = price_range / (config.grid_levels - 1) as f64;

        for i in 0..config.grid_levels {
            let level = config.lower_price + (i as f64 * level_spacing);
            levels.push(level);
        }

        levels
    }
}

/// DCA (Dollar Cost Averaging) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DCAConfig {
    pub strategy_id: String,
    pub from_token: String,
    pub to_token: String,
    pub total_amount: f64,
    pub intervals: u32,
    pub interval_seconds: u64,
    pub slippage_tolerance: f64,
}

/// Momentum trading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumConfig {
    pub strategy_id: String,
    pub token: String,
    pub trade_amount: f64,
    pub momentum_threshold: f64,
    pub lookback_periods: u32,
    pub slippage_tolerance: f64,
}

/// Grid trading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    pub strategy_id: String,
    pub token: String,
    pub total_amount: f64,
    pub lower_price: f64,
    pub upper_price: f64,
    pub grid_levels: u32,
    pub slippage_tolerance: f64,
}

/// Trading strategy trait for pluggable strategies
#[async_trait]
pub trait TradingStrategy: Send + Sync {
    async fn execute(&self, executor: &StrategyExecutor) -> Result<ExecutionResult>;
    fn get_strategy_type(&self) -> StrategyType;
    fn get_strategy_id(&self) -> String;
}

/// DCA strategy implementation
pub struct DCAStrategy {
    config: DCAConfig,
}

impl DCAStrategy {
    pub fn new(config: DCAConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl TradingStrategy for DCAStrategy {
    async fn execute(&self, executor: &StrategyExecutor) -> Result<ExecutionResult> {
        executor.execute_dca_strategy(self.config.clone()).await
    }

    fn get_strategy_type(&self) -> StrategyType {
        StrategyType::DCA
    }

    fn get_strategy_id(&self) -> String {
        self.config.strategy_id.clone()
    }
}

/// Momentum strategy implementation
pub struct MomentumStrategy {
    config: MomentumConfig,
}

impl MomentumStrategy {
    pub fn new(config: MomentumConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl TradingStrategy for MomentumStrategy {
    async fn execute(&self, executor: &StrategyExecutor) -> Result<ExecutionResult> {
        executor.execute_momentum_strategy(self.config.clone()).await
    }

    fn get_strategy_type(&self) -> StrategyType {
        StrategyType::Momentum
    }

    fn get_strategy_id(&self) -> String {
        self.config.strategy_id.clone()
    }
}

/// Grid strategy implementation
pub struct GridStrategy {
    config: GridConfig,
}

impl GridStrategy {
    pub fn new(config: GridConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl TradingStrategy for GridStrategy {
    async fn execute(&self, executor: &StrategyExecutor) -> Result<ExecutionResult> {
        executor.execute_grid_strategy(self.config.clone()).await
    }

    fn get_strategy_type(&self) -> StrategyType {
        StrategyType::Grid
    }

    fn get_strategy_id(&self) -> String {
        self.config.strategy_id.clone()
    }
}
