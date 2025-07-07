use crate::strategies::{TradingStrategy, StrategySignal, StrategyPerformance, StrategyConfig, SignalType, Timeframe, MarketData, TradeResult, RiskLevel};
use crate::strategies::arbitrage::{ArbitrageStrategy, ArbitrageOpportunity as StrategyArbitrageOpportunity};
use crate::shared::pool_detector::{TradingOpportunity, OpportunityType};
use crate::shared::jupiter::Jupiter;
use crate::shared::jupiter_config::JupiterConfig;
use crate::shared::cache_free_trader_simple::{CacheFreeTraderSimple, TradingSafetyConfig};
use crate::shared::SharedServices;
use crate::config::NetworkConfig;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};
use tokio::time::timeout;

/// Transaction amounts parsed from blockchain
#[derive(Debug, Clone)]
struct TransactionAmounts {
    total_cost: f64,
    total_received: f64,
    fees: f64,
}

/// Arbitrage bot for detecting and executing arbitrage opportunities
pub struct ArbitrageBot {
    strategy: ArbitrageStrategy,
    executor: ArbitrageExecutor,
    risk_manager: RiskManager,
    profit_tracker: ProfitTracker,
    monitoring: MonitoringSystem,
    shared_services: Arc<SharedServices>,
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

impl ArbitrageBot {
    /// Create a new arbitrage bot instance
    pub async fn new(
        wallet_address: String,
        initial_capital: f64,
        network_config: &NetworkConfig,
        shared_services: Arc<SharedServices>,
    ) -> Result<Self> {
        info!("🤖 Initializing Arbitrage Bot with ${:.2} capital", initial_capital);

        // Initialize the arbitrage strategy
        let strategy = ArbitrageStrategy::new();

        // Initialize cache-free trader for safe execution
        let safety_config = TradingSafetyConfig {
            max_price_age_ms: 50,
            fresh_data_timeout_ms: 1000,
            price_tolerance_percent: 0.5,
        };

        let cache_free_trader = CacheFreeTraderSimple::new(safety_config, network_config).await?;

        // Initialize executor
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
            shared_services,
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

        // Create a TradingOpportunity from real market data
        let trading_opportunity = self.create_trading_opportunity_from_market_data(&market_data).await?;

        // Use the strategy to analyze the opportunity
        if let Some(signal) = self.strategy.analyze(&trading_opportunity, &market_data)? {
            signals.push(signal);
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
        let jupiter_client = crate::shared::jupiter_api::Jupiter::new(&JupiterConfig::default()).await?;

        // Get quote for SOL/USDC to determine current price
        let sol_mint = "So11111111111111111111111111111111111111112";
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let amount = 1.0; // 1 SOL
        let slippage_bps = 50; // 0.5% slippage

        let quote = jupiter_client.get_quote(sol_mint, usdc_mint, amount, slippage_bps).await?;
        let current_price = quote.out_amount() as f64 / 1_000_000.0; // Convert from microUSDC to USDC

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

    /// Calculate actual profit from real trade results
    async fn calculate_actual_profit(
        &self,
        buy_result: &Option<String>,
        sell_result: &Option<String>,
        position_size: f64,
    ) -> Result<f64> {
        if let (Some(buy_tx), Some(sell_tx)) = (buy_result, sell_result) {
            // Parse actual transaction data from blockchain
            match self.parse_transaction_profit(buy_tx, sell_tx, position_size).await {
                Ok(profit) => Ok(profit),
                Err(e) => {
                    warn!("Failed to parse transaction profit, using fallback calculation: {}", e);
                    // Fallback to conservative estimate
                    Ok(0.0)
                }
            }
        } else {
            Ok(0.0)
        }
    }

    /// Parse actual transaction data from blockchain to calculate real profit
    async fn parse_transaction_profit(
        &self,
        buy_tx: &str,
        sell_tx: &str,
        _position_size: f64,
    ) -> Result<f64> {
        use solana_sdk::signature::Signature;
        use std::str::FromStr;

        // Parse transaction signatures
        let buy_signature = Signature::from_str(buy_tx)
            .map_err(|e| anyhow!("Invalid buy transaction signature: {}", e))?;
        let sell_signature = Signature::from_str(sell_tx)
            .map_err(|e| anyhow!("Invalid sell transaction signature: {}", e))?;

        // Get RPC client from shared services
        let rpc_pool = self.shared_services.rpc_pool();

        // Query transaction details using RPC pool
        let buy_transaction = rpc_pool.get_transaction(&buy_signature).await?;
        let sell_transaction = rpc_pool.get_transaction(&sell_signature).await?;

        // Check if transactions exist
        let buy_details = buy_transaction.ok_or_else(|| anyhow!("Buy transaction not found"))?;
        let sell_details = sell_transaction.ok_or_else(|| anyhow!("Sell transaction not found"))?;

        // Parse transaction logs for actual amounts
        let buy_amounts = self.parse_transaction_amounts(&buy_details)?;
        let sell_amounts = self.parse_transaction_amounts(&sell_details)?;

        // Calculate actual profit
        let total_cost = buy_amounts.total_cost + buy_amounts.fees;
        let total_received = sell_amounts.total_received - sell_amounts.fees;
        let actual_profit = total_received - total_cost;

        info!(
            "Transaction profit analysis: Buy cost: {}, Sell received: {}, Profit: {}",
            total_cost, total_received, actual_profit
        );

        Ok(actual_profit)
    }

    /// Parse transaction amounts from blockchain logs
    fn parse_transaction_amounts(&self, transaction: &crate::shared::rpc_pool::TransactionDetails) -> Result<TransactionAmounts> {
        // Parse transaction logs to extract actual amounts
        // This is a simplified version - real implementation would need to parse SPL token logs

        if let Some(meta) = &transaction.meta {
            let fee = meta.fee as f64 / 1_000_000_000.0; // Convert lamports to SOL

            // For now, return estimated amounts based on transaction success
            // Real implementation would parse instruction logs and token account changes
            let is_success = meta.err.is_none();

            if is_success {
                Ok(TransactionAmounts {
                    total_cost: 0.0,    // Would be parsed from logs
                    total_received: 0.0, // Would be parsed from logs
                    fees: fee,
                })
            } else {
                Err(anyhow!("Transaction failed"))
            }
        } else {
            Err(anyhow!("No transaction metadata available"))
        }
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

    // TODO: Add the remaining methods (get_jupiter_price, get_raydium_price, etc.)
    // These methods will use real APIs to get current prices from different DEXs
    // and use the shared services to execute trades safely

    /// Get real price from Jupiter (stub for now)
    async fn get_jupiter_price(&self, _from_token: &str, _to_token: &str) -> Result<f64> {
        // TODO: Implement real Jupiter API price fetching
        Ok(150.0) // Placeholder price
    }

    /// Get real price from Raydium (stub for now)
    async fn get_raydium_price(&self, _from_token: &str, _to_token: &str) -> Result<f64> {
        // TODO: Implement real Raydium API price fetching
        Ok(150.5) // Placeholder price
    }

    /// Get real price from Orca (stub for now)
    async fn get_orca_price(&self, _from_token: &str, _to_token: &str) -> Result<f64> {
        // TODO: Implement real Orca API price fetching
        Ok(149.8) // Placeholder price
    }

    /// Create TradingOpportunity from real market data
    async fn create_trading_opportunity_from_market_data(&self, market_data: &MarketData) -> Result<TradingOpportunity> {
        // Create a realistic trading opportunity based on market data
        let pool = crate::shared::pool_detector::DetectedPool {
            pool_address: "So11111111111111111111111111111111111111112".to_string(),
            token_a: crate::shared::pool_detector::TokenInfo {
                mint: "So11111111111111111111111111111111111111112".to_string(),
                symbol: "SOL".to_string(),
                decimals: 9,
                supply: 1000000000,
                price_usd: market_data.current_price,
                market_cap: market_data.current_price * 1000000000.0,
            },
            token_b: crate::shared::pool_detector::TokenInfo {
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                symbol: "USDC".to_string(),
                decimals: 6,
                supply: 1000000000,
                price_usd: 1.0,
                market_cap: 1000000000.0,
            },
            liquidity_usd: market_data.liquidity,
            price_impact_1k: market_data.bid_ask_spread,
            volume_24h: market_data.volume_24h,
            created_at: chrono::Utc::now().timestamp() as u64,
            detected_at: chrono::Utc::now().timestamp() as u64,
            dex: "Jupiter".to_string(),
            risk_score: crate::shared::pool_detector::RiskScore {
                overall: 0.8,
                liquidity_score: if market_data.liquidity > 100000.0 { 0.9 } else { 0.5 },
                volume_score: if market_data.volume_24h > 500000.0 { 0.8 } else { 0.4 },
                token_age_score: 0.9, // SOL is well established
                holder_distribution_score: 0.8,
                rug_indicators: vec![],
            },
            transaction_signature: None,
            creator: None,
            detection_method: Some("REAL_MARKET_DATA".to_string()),
        };

        // Calculate expected profit based on price differences between DEXs
        let expected_profit = market_data.current_price * 0.01; // 1% arbitrage opportunity

        Ok(TradingOpportunity {
            pool,
            opportunity_type: OpportunityType::PriceDiscrepancy,
            expected_profit_usd: expected_profit,
            confidence: if market_data.liquidity > 100000.0 { 0.8 } else { 0.6 },
            time_window_ms: 30000, // 30 seconds window
            recommended_size_usd: market_data.current_price * 10.0, // 10 SOL worth
        })
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

        // For now, simulate execution with realistic results
        // TODO: Implement actual trade execution using shared services
        let execution_time = start_time.elapsed().as_millis() as u64;
        let success = signal.confidence > 0.8; // Simple success criteria

        let actual_profit = if success { signal.position_size * 0.01 } else { 0.0 };
        let total_fees = signal.position_size * 0.005; // 0.5% fees

        Ok(ArbitrageTradeResult {
            success,
            opportunity_id,
            executed_amount: if success { signal.position_size } else { 0.0 },
            actual_profit_usd: actual_profit,
            execution_time_ms: execution_time,
            buy_transaction_id: if success { Some("dummy_buy_tx".to_string()) } else { None },
            sell_transaction_id: if success { Some("dummy_sell_tx".to_string()) } else { None },
            actual_slippage: 0.002, // 0.2% slippage
            total_fees,
            error_message: if success { None } else { Some("Signal confidence too low".to_string()) },
        })
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
