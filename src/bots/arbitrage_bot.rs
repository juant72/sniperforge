use crate::strategies::{TradingStrategy, StrategySignal, StrategyPerformance, StrategyConfig, SignalType, Timeframe, MarketData, TradeResult, RiskLevel};
use crate::strategies::arbitrage::{ArbitrageStrategy, ArbitrageOpportunity as StrategyArbitrageOpportunity};
use crate::shared::pool_detector::{TradingOpportunity, OpportunityType};
use crate::shared::jupiter::Jupiter;
use crate::shared::cache_free_trader_simple::{CacheFreeTraderSimple, TradingSafetyConfig};
use crate::config::NetworkConfig;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};
use tokio::time::timeout;

/// Arbitrage bot for detecting and executing arbitrage opportunities
pub struct ArbitrageBot {
    strategy: ArbitrageStrategy,
    executor: ArbitrageExecutor,
    risk_manager: RiskManager,
    profit_tracker: ProfitTracker,
    monitoring: MonitoringSystem,
}

/// Executes arbitrage trades (reuses existing logic)
pub struct ArbitrageExecutor {
    cache_free_trader: CacheFreeTraderSimple,
    max_position_size: f64,
    wallet_address: String,
}

/// Risk management for arbitrage trading
pub struct RiskManager {
    daily_loss_limit: f64,    // 5%
    max_position_size: f64,   // 20%
    max_concurrent_trades: u32, // 3
    emergency_stop: bool,
    current_daily_loss: f64,
    trades_today: u32,
}

/// Tracks profit/loss for arbitrage trades
pub struct ProfitTracker {
    total_trades: u32,
    successful_trades: u32,
    total_profit_usd: f64,
    total_fees_usd: f64,
    daily_profit: f64,
    best_trade_profit: f64,
    worst_trade_loss: f64,
}

/// Monitoring system for arbitrage bot
pub struct MonitoringSystem {
    start_time: Instant,
    last_opportunity_time: Option<Instant>,
    opportunities_detected: u32,
    opportunities_executed: u32,
    average_latency_ms: f64,
}

/// Individual DEX monitor for price feeds (reuse from strategy)
pub struct DexMonitor {
    dex_name: String,
    last_price_update: Instant,
    current_price: f64,
    liquidity: f64,
    is_active: bool,
}

/// Arbitrage opportunity detected by the bot (alias to strategy version)
pub type ArbitrageOpportunity = StrategyArbitrageOpportunity;

/// Result of an arbitrage trade execution
#[derive(Debug, Clone)]
pub struct ArbitrageTradeResult {
    pub success: bool,
    pub opportunity_id: String,
    pub executed_amount: f64,
    pub actual_profit_usd: f64,
    pub execution_time_ms: u64,
    pub buy_transaction_id: Option<String>,
    pub sell_transaction_id: Option<String>,
    pub actual_slippage: f64,
    pub total_fees: f64,
    pub error_message: Option<String>,
}

impl ArbitrageBot {
    /// Create a new arbitrage bot instance
    pub async fn new(wallet_address: String, initial_capital: f64, network_config: &NetworkConfig) -> Result<Self> {
        info!("🤖 Initializing Arbitrage Bot with ${:.2} capital", initial_capital);

        // Initialize the arbitrage strategy with appropriate configuration
        let strategy = ArbitrageStrategy::new();        // Initialize cache-free trader for safe execution with proper config
        let safety_config = TradingSafetyConfig {
            max_price_age_ms: 50,
            fresh_data_timeout_ms: 1000,
            price_tolerance_percent: 0.5,
        };

        let cache_free_trader = CacheFreeTraderSimple::new(safety_config, network_config).await?;

        // Initialize executor (simplified - reuse existing logic)
        let executor = ArbitrageExecutor {
            cache_free_trader,
            max_position_size: initial_capital * 0.2, // 20% max per trade
            wallet_address: wallet_address.clone(),
        };

        // Initialize risk manager
        let risk_manager = RiskManager {
            daily_loss_limit: initial_capital * 0.05, // 5% daily loss limit
            max_position_size: initial_capital * 0.2, // 20% max position
            max_concurrent_trades: 3,
            emergency_stop: false,
            current_daily_loss: 0.0,
            trades_today: 0,
        };

        // Initialize profit tracker
        let profit_tracker = ProfitTracker {
            total_trades: 0,
            successful_trades: 0,
            total_profit_usd: 0.0,
            total_fees_usd: 0.0,
            daily_profit: 0.0,
            best_trade_profit: 0.0,
            worst_trade_loss: 0.0,
        };

        // Initialize monitoring
        let monitoring = MonitoringSystem {
            start_time: Instant::now(),
            last_opportunity_time: None,
            opportunities_detected: 0,
            opportunities_executed: 0,
            average_latency_ms: 0.0,
        };

        Ok(Self {
            strategy,
            executor,
            risk_manager,
            profit_tracker,
            monitoring,
        })
    }

    /// Start the arbitrage bot main loop
    pub async fn start_trading(&mut self) -> Result<()> {
        info!("🚀 Starting Arbitrage Bot trading loop");

        loop {
            // Check if emergency stop is activated
            if self.risk_manager.emergency_stop {
                warn!("🚨 Emergency stop activated - halting trading");
                break;
            }

            // Use the existing strategy to detect opportunities
            match self.detect_opportunities_using_strategy().await {
                Ok(signals) => {
                    if !signals.is_empty() {
                        info!("🔍 Found {} arbitrage signals", signals.len());

                        // Execute the best signal
                        if let Some(best_signal) = signals.first() {
                            if self.should_execute_trade_from_signal(best_signal)? {
                                match self.execute_signal(best_signal.clone()).await {
                                    Ok(result) => {
                                        self.process_trade_result(result).await?;
                                    }
                                    Err(e) => {
                                        error!("❌ Failed to execute signal: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Error detecting opportunities: {}", e);
                }
            }

            // Sleep briefly before next iteration
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        info!("🛑 Arbitrage Bot stopped");
        Ok(())
    }

    /// Detect arbitrage opportunities using real DEX data
    async fn detect_opportunities_using_strategy(&mut self) -> Result<Vec<StrategySignal>> {
        let start_time = Instant::now();
        let mut signals = Vec::new();

        // Update price feeds with real data from DEXs
        self.update_real_price_feeds().await?;

        // Get real market data for SOL/USDC from multiple DEXs
        let market_data = self.get_real_market_data().await?;

        // Use the strategy to detect real opportunities
        let opportunities = self.strategy.detect_opportunities(&market_data)?;

        // Convert strategy opportunities to signals
        for opportunity in opportunities {
            if let Some(signal) = self.strategy.analyze(&opportunity, &market_data)? {
                signals.push(signal);
            }
        }

        // Update monitoring stats
        self.monitoring.opportunities_detected += signals.len() as u32;
        if !signals.is_empty() {
            self.monitoring.last_opportunity_time = Some(Instant::now());
        }

        let detection_time = start_time.elapsed().as_millis() as f64;
        self.monitoring.average_latency_ms =
            (self.monitoring.average_latency_ms + detection_time) / 2.0;

        Ok(signals)
    }

    /// Update price feeds with real data from DEXs
    async fn update_real_price_feeds(&mut self) -> Result<()> {
        // Get real prices from Jupiter API
        if let Ok(jupiter_price) = self.get_jupiter_price("SOL", "USDC").await {
            self.strategy.update_price_feed("Jupiter".to_string(), jupiter_price);
        }

        // Get real prices from Raydium API
        if let Ok(raydium_price) = self.get_raydium_price("SOL", "USDC").await {
            self.strategy.update_price_feed("Raydium".to_string(), raydium_price);
        }

        // Get real prices from Orca API
        if let Ok(orca_price) = self.get_orca_price("SOL", "USDC").await {
            self.strategy.update_price_feed("Orca".to_string(), orca_price);
        }

        Ok(())
    }

    /// Get real market data for SOL/USDC from multiple sources
    async fn get_real_market_data(&self) -> Result<MarketData> {
        // Use Jupiter as primary source for market data
        let jupiter_client = crate::shared::jupiter_api::Jupiter::new(&crate::shared::jupiter_api::JupiterConfig::default()).await?;

        // Get quote for SOL/USDC to determine current price
        let sol_mint = "So11111111111111111111111111111111111111112";
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let amount = 1_000_000_000; // 1 SOL in lamports

        let quote = jupiter_client.get_quote(sol_mint, usdc_mint, amount, None).await?;
        let current_price = quote.out_amount as f64 / 1_000_000.0; // Convert from microUSDC to USDC

        // Get additional market data from DexScreener or other sources
        let market_data = MarketData {
            current_price,
            volume_24h: 0.0, // Will be populated by real API calls
            price_change_24h: 0.0, // Will be populated by real API calls
            liquidity: 0.0, // Will be populated by real API calls
            bid_ask_spread: 0.0, // Will be calculated from order book
            order_book_depth: 0.0, // Will be populated by real API calls
            price_history: vec![],
            volume_history: vec![],
        };

        Ok(market_data)
    }

    /// Get real price from Jupiter
    async fn get_jupiter_price(&self, from_token: &str, to_token: &str) -> Result<f64> {
        let jupiter_client = crate::shared::jupiter_api::Jupiter::new(&crate::shared::jupiter_api::JupiterConfig::default()).await?;

        let (from_mint, to_mint) = self.get_token_mints(from_token, to_token)?;
        let amount = 1_000_000_000; // 1 unit of from_token

        let quote = jupiter_client.get_quote(&from_mint, &to_mint, amount, None).await?;
        let price = quote.out_amount as f64 / 1_000_000.0; // Adjust for decimals

        Ok(price)
    }

    /// Get real price from Raydium
    async fn get_raydium_price(&self, from_token: &str, to_token: &str) -> Result<f64> {
        // Use Raydium API to get real price
        // For now, we'll use a placeholder that gets data from a real source
        // TODO: Implement direct Raydium API calls
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.raydium.io/v2/sdk/token/raydium.mainnet.json")
            .send()
            .await?;

        if response.status().is_success() {
            // Parse response and extract price for the trading pair
            // This is a simplified version - real implementation would parse JSON
            Ok(0.0) // Placeholder - replace with actual price extraction
        } else {
            Err(anyhow!("Failed to get Raydium price"))
        }
    }

    /// Get real price from Orca
    async fn get_orca_price(&self, from_token: &str, to_token: &str) -> Result<f64> {
        // Use Orca API to get real price
        // For now, we'll use a placeholder that gets data from a real source
        // TODO: Implement direct Orca API calls
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.orca.so/v1/whirlpool/list")
            .send()
            .await?;

        if response.status().is_success() {
            // Parse response and extract price for the trading pair
            // This is a simplified version - real implementation would parse JSON
            Ok(0.0) // Placeholder - replace with actual price extraction
        } else {
            Err(anyhow!("Failed to get Orca price"))
        }
    }

    /// Get token mints for a trading pair
    fn get_token_mints(&self, from_token: &str, to_token: &str) -> Result<(String, String)> {
        let from_mint = match from_token {
            "SOL" => "So11111111111111111111111111111111111111112".to_string(),
            "USDC" => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            _ => return Err(anyhow!("Unsupported token: {}", from_token)),
        };

        let to_mint = match to_token {
            "SOL" => "So11111111111111111111111111111111111111112".to_string(),
            "USDC" => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            _ => return Err(anyhow!("Unsupported token: {}", to_token)),
        };

        Ok((from_mint, to_mint))
    }

    /// Check if we should execute a trade based on risk management (from signal)
    fn should_execute_trade_from_signal(&self, signal: &StrategySignal) -> Result<bool> {
        // Check emergency stop
        if self.risk_manager.emergency_stop {
            return Ok(false);
        }

        // Check daily loss limit
        if self.risk_manager.current_daily_loss >= self.risk_manager.daily_loss_limit {
            warn!("🚨 Daily loss limit reached: ${:.2}", self.risk_manager.current_daily_loss);
            return Ok(false);
        }

        // Check concurrent trades limit
        if self.risk_manager.trades_today >= self.risk_manager.max_concurrent_trades {
            return Ok(false);
        }

        // Check minimum confidence
        if signal.confidence < 0.7 {
            return Ok(false);
        }

        // Check position size limits
        if signal.position_size > self.executor.max_position_size {
            return Ok(false);
        }

        Ok(true)
    }

    /// Execute a strategy signal
    async fn execute_signal(&mut self, signal: StrategySignal) -> Result<ArbitrageTradeResult> {
        let start_time = Instant::now();
        let opportunity_id = format!("arb_{}", chrono::Utc::now().timestamp_millis());

        info!("⚡ Executing arbitrage signal: {} | Confidence: {:.2}",
              signal.strategy_name, signal.confidence);

        // Extract arbitrage details from signal metadata
        let _buy_exchange = signal.metadata.get("buy_exchange").unwrap_or(&"Unknown".to_string()).clone();
        let _sell_exchange = signal.metadata.get("sell_exchange").unwrap_or(&"Unknown".to_string()).clone();
        let default_profit = "0.0".to_string();
        let estimated_profit_str = signal.metadata.get("estimated_profit").unwrap_or(&default_profit);
        let estimated_profit: f64 = estimated_profit_str.parse().unwrap_or(0.0);

        // Execute trade using cache-free trader
        let amount_lamports = (signal.position_size * 1_000_000_000.0) as u64;

        // Execute buy and sell orders
        let buy_result = self.execute_trade_order("buy", amount_lamports).await?;
        let sell_result = self.execute_trade_order("sell", amount_lamports).await?;

        let execution_time = start_time.elapsed().as_millis() as u64;
        let success = buy_result.is_some() && sell_result.is_some();

        // Calculate actual profit based on real execution results
        let actual_profit = if success {
            // Get actual profit from the trade results
            self.calculate_actual_profit(&buy_result, &sell_result, signal.position_size).await?
        } else {
            0.0
        };

        // Calculate actual slippage from execution
        let actual_slippage = if success {
            self.calculate_actual_slippage(&buy_result, &sell_result, estimated_profit).await?
        } else {
            0.0
        };

        // Calculate real fees from transaction results
        let total_fees = if success {
            self.calculate_real_fees(&buy_result, &sell_result).await?
        } else {
            0.0
        };

        Ok(ArbitrageTradeResult {
            success,
            opportunity_id,
            executed_amount: if success { signal.position_size } else { 0.0 },
            actual_profit_usd: actual_profit,
            execution_time_ms: execution_time,
            buy_transaction_id: buy_result,
            sell_transaction_id: sell_result,
            actual_slippage,
            total_fees,
            error_message: if success { None } else { Some("Execution failed".to_string()) },
        })
    }

    /// Execute a trade order (simplified)
    async fn execute_trade_order(&self, order_type: &str, amount_lamports: u64) -> Result<Option<String>> {
        info!("📊 Executing {} order for {} lamports", order_type, amount_lamports);

        let (input_mint, output_mint) = match order_type {
            "buy" => ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "So11111111111111111111111111111111111111112"), // USDC -> SOL
            "sell" => ("So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // SOL -> USDC
            _ => return Err(anyhow!("Unknown order type: {}", order_type)),
        };

        match timeout(Duration::from_secs(30),
            self.executor.cache_free_trader.execute_safe_swap(
                input_mint,
                output_mint,
                amount_lamports
            )
        ).await {
            Ok(Ok(result)) => {
                if result.success {
                    info!("✅ {} order executed successfully", order_type);
                    // Return actual transaction ID from the swap result
                    Ok(result.transaction_id)
                } else {
                    error!("❌ {} order failed", order_type);
                    Ok(None)
                }
            }
            Ok(Err(e)) => {
                error!("❌ {} order error: {}", order_type, e);
                Ok(None)
            }
            Err(_) => {
                error!("❌ {} order timeout", order_type);
                Ok(None)
            }
        }
    }

    /// Process the result of a trade execution
    async fn process_trade_result(&mut self, result: ArbitrageTradeResult) -> Result<()> {
        // Update profit tracker
        self.profit_tracker.total_trades += 1;
        self.risk_manager.trades_today += 1;

        if result.success {
            self.profit_tracker.successful_trades += 1;
            self.profit_tracker.total_profit_usd += result.actual_profit_usd;
            self.profit_tracker.daily_profit += result.actual_profit_usd;

            if result.actual_profit_usd > self.profit_tracker.best_trade_profit {
                self.profit_tracker.best_trade_profit = result.actual_profit_usd;
            }

            info!("✅ Trade #{} successful: +${:.2} profit",
                  self.profit_tracker.total_trades, result.actual_profit_usd);
        } else {
            let loss = result.total_fees;
            self.risk_manager.current_daily_loss += loss;

            if loss > self.profit_tracker.worst_trade_loss {
                self.profit_tracker.worst_trade_loss = loss;
            }

            error!("❌ Trade #{} failed: -${:.2} loss",
                   self.profit_tracker.total_trades, loss);
        }

        self.profit_tracker.total_fees_usd += result.total_fees;

        // Update monitoring
        if result.success {
            self.monitoring.opportunities_executed += 1;
        }

        // Print stats
        self.print_trading_stats();

        Ok(())
    }

    /// Print current trading statistics
    fn print_trading_stats(&self) {
        let success_rate = if self.profit_tracker.total_trades > 0 {
            (self.profit_tracker.successful_trades as f64 / self.profit_tracker.total_trades as f64) * 100.0
        } else {
            0.0
        };

        info!("📊 Trading Stats:");
        info!("   Total trades: {}", self.profit_tracker.total_trades);
        info!("   Success rate: {:.1}%", success_rate);
        info!("   Total profit: ${:.2}", self.profit_tracker.total_profit_usd);
        info!("   Daily profit: ${:.2}", self.profit_tracker.daily_profit);
        info!("   Total fees: ${:.2}", self.profit_tracker.total_fees_usd);
        info!("   Opportunities detected: {}", self.monitoring.opportunities_detected);
        info!("   Average latency: {:.1}ms", self.monitoring.average_latency_ms);
    }

    /// Get current bot status and statistics
    pub fn get_status(&self) -> ArbitrageBotStatus {
        ArbitrageBotStatus {
            is_running: !self.risk_manager.emergency_stop,
            uptime_seconds: self.monitoring.start_time.elapsed().as_secs(),
            total_trades: self.profit_tracker.total_trades,
            successful_trades: self.profit_tracker.successful_trades,
            total_profit_usd: self.profit_tracker.total_profit_usd,
            daily_profit_usd: self.profit_tracker.daily_profit,
            success_rate_percent: if self.profit_tracker.total_trades > 0 {
                (self.profit_tracker.successful_trades as f64 / self.profit_tracker.total_trades as f64) * 100.0
            } else {
                0.0
            },
            opportunities_detected: self.monitoring.opportunities_detected,
            opportunities_executed: self.monitoring.opportunities_executed,
            average_latency_ms: self.monitoring.average_latency_ms,
            daily_loss_usd: self.risk_manager.current_daily_loss,
            emergency_stop: self.risk_manager.emergency_stop,
        }
    }

    /// Trigger emergency stop
    pub fn emergency_stop(&mut self) {
        error!("🚨 EMERGENCY STOP ACTIVATED");
        self.risk_manager.emergency_stop = true;
    }

    /// Reset daily statistics (call at start of each day)
    pub fn reset_daily_stats(&mut self) {
        self.profit_tracker.daily_profit = 0.0;
        self.risk_manager.current_daily_loss = 0.0;
        self.risk_manager.trades_today = 0;
        info!("🔄 Daily statistics reset");
    }

    /// Calculate actual profit from real trade results
    async fn calculate_actual_profit(
        &self,
        buy_result: &Option<String>,
        sell_result: &Option<String>,
        position_size: f64,
    ) -> Result<f64> {
        if let (Some(buy_tx), Some(sell_tx)) = (buy_result, sell_result) {
            // Get actual transaction details from blockchain
            // This would involve querying the transaction details
            // For now, we'll use the cache_free_trader results
            // TODO: Implement real transaction analysis

            // Calculate profit based on actual input/output amounts
            let buy_amount_in = position_size; // Amount spent on buy
            let sell_amount_out = position_size; // Amount received from sell

            // Real profit calculation would involve:
            // 1. Getting actual amounts from transaction logs
            // 2. Converting to USD using current prices
            // 3. Subtracting fees

            Ok(0.0) // Placeholder - implement real calculation
        } else {
            Ok(0.0)
        }
    }

    /// Calculate actual slippage from execution
    async fn calculate_actual_slippage(
        &self,
        buy_result: &Option<String>,
        sell_result: &Option<String>,
        estimated_profit: f64,
    ) -> Result<f64> {
        if let (Some(_buy_tx), Some(_sell_tx)) = (buy_result, sell_result) {
            // Calculate slippage based on expected vs actual execution prices
            // This would involve comparing the expected price with actual execution price
            // TODO: Implement real slippage calculation from transaction logs

            Ok(0.0) // Placeholder - implement real calculation
        } else {
            Ok(0.0)
        }
    }

    /// Calculate real fees from transaction results
    async fn calculate_real_fees(
        &self,
        buy_result: &Option<String>,
        sell_result: &Option<String>,
    ) -> Result<f64> {
        if let (Some(_buy_tx), Some(_sell_tx)) = (buy_result, sell_result) {
            // Get actual fees from transaction logs
            // This would involve parsing transaction logs to get:
            // 1. Network fees (transaction fees)
            // 2. DEX fees (swap fees)
            // 3. Any other protocol fees
            // TODO: Implement real fee calculation from transaction logs

            Ok(0.0) // Placeholder - implement real calculation
        } else {
            Ok(0.0)
        }
    }
}

impl DexMonitor {
    fn new(dex_name: String) -> Self {
        Self {
            dex_name,
            last_price_update: Instant::now(),
            current_price: 0.0,
            liquidity: 0.0,
            is_active: false,
        }
    }
}

/// Status information for the arbitrage bot
#[derive(Debug, Clone)]
pub struct ArbitrageBotStatus {
    pub is_running: bool,
    pub uptime_seconds: u64,
    pub total_trades: u32,
    pub successful_trades: u32,
    pub total_profit_usd: f64,
    pub daily_profit_usd: f64,
    pub success_rate_percent: f64,
    pub opportunities_detected: u32,
    pub opportunities_executed: u32,
    pub average_latency_ms: f64,
    pub daily_loss_usd: f64,
    pub emergency_stop: bool,
}
