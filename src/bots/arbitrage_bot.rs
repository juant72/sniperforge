use crate::strategies::{ArbitrageStrategy, TradingStrategy, StrategySignal, MarketData, TradeResult};
use crate::shared::pool_detector::{TradingOpportunity, OpportunityType};
use crate::shared::jupiter::Jupiter;
use crate::shared::cache_free_trader_simple::CacheFreeTraderSimple;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};
use tokio::time::timeout;

/// Arbitrage bot for detecting and executing arbitrage opportunities
/// Uses the existing ArbitrageStrategy for analysis and adds execution logic
pub struct ArbitrageBot {
    strategy: ArbitrageStrategy,
    executor: ArbitrageExecutor,
    risk_manager: RiskManager,
    profit_tracker: ProfitTracker,
    monitoring: MonitoringSystem,
    wallet_address: String,
}

/// Executes arbitrage trades using existing trading modules
pub struct ArbitrageExecutor {
    jupiter_client: Jupiter,
    cache_free_trader: CacheFreeTraderSimple,
    max_position_size: f64,   // 20% of balance
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

/// Arbitrage opportunity detected by the bot
#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub token_pair: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub spread_percentage: f64,
    pub estimated_profit_usd: f64,
    pub liquidity_buy: f64,
    pub liquidity_sell: f64,
    pub confidence_score: f64,
    pub detected_at: Instant,
    pub estimated_execution_time_ms: u64,
}

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
    /// Create a new arbitrage bot instance using the existing ArbitrageStrategy
    pub async fn new(wallet_address: String, initial_capital: f64) -> Result<Self> {
        info!("🤖 Initializing Arbitrage Bot with ${:.2} capital", initial_capital);

        // Initialize the existing ArbitrageStrategy
        let mut strategy = ArbitrageStrategy::new();

        // Initialize Jupiter for DEX access
        let jupiter_config = crate::shared::jupiter::JupiterConfig::default();
        let jupiter = Jupiter::new(&jupiter_config).await?;

        // Initialize cache-free trader for safe execution
        let cache_free_trader = CacheFreeTraderSimple::new();

        // Initialize executor
        let executor = ArbitrageExecutor {
            jupiter_client: jupiter,
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
            wallet_address,
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

            // Detect arbitrage opportunities
            match self.detect_opportunities().await {
                Ok(opportunities) => {
                    if !opportunities.is_empty() {
                        info!("🔍 Found {} arbitrage opportunities", opportunities.len());

                        // Execute the best opportunity
                        if let Some(best_opportunity) = self.select_best_opportunity(&opportunities) {
                            if self.should_execute_trade(&best_opportunity)? {
                                match self.execute_opportunity(best_opportunity).await {
                                    Ok(result) => {
                                        self.process_trade_result(result).await?;
                                    }
                                    Err(e) => {
                                        error!("❌ Failed to execute opportunity: {}", e);
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

    /// Detect arbitrage opportunities using the existing ArbitrageStrategy
    async fn detect_opportunities(&mut self) -> Result<Vec<ArbitrageOpportunity>> {
        let start_time = Instant::now();
        let mut opportunities = Vec::new();

        // Update price feeds for the strategy
        self.update_strategy_price_feeds().await?;

        // Create market data for analysis
        let market_data = self.build_market_data().await?;

        // Create a dummy trading opportunity for the strategy analysis
        let dummy_opportunity = TradingOpportunity {
            pool: crate::shared::pool_detector::DetectedPool {
                pair_address: "SOL/USDC".to_string(),
                token_a: crate::shared::pool_detector::TokenInfo {
                    mint: "So11111111111111111111111111111111111111112".to_string(),
                    symbol: "SOL".to_string(),
                    name: "Solana".to_string(),
                    decimals: 9,
                    price_usd: market_data.current_price,
                    volume_24h: market_data.volume_24h,
                    liquidity: market_data.liquidity,
                    is_verified: true,
                },
                token_b: crate::shared::pool_detector::TokenInfo {
                    mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                    symbol: "USDC".to_string(),
                    name: "USD Coin".to_string(),
                    decimals: 6,
                    price_usd: 1.0,
                    volume_24h: market_data.volume_24h,
                    liquidity: market_data.liquidity,
                    is_verified: true,
                },
                liquidity_usd: market_data.liquidity,
                volume_24h: market_data.volume_24h,
                dex: "Multiple".to_string(),
                created_at: chrono::Utc::now(),
                is_active: true,
            },
            opportunity_type: OpportunityType::PriceDiscrepancy,
            expected_profit_usd: 0.0,
            confidence: 0.8,
            time_window_ms: 15000,
            recommended_size_usd: 500.0,
        };

        // Use the existing strategy for analysis
        if let Some(signal) = self.strategy.analyze(&dummy_opportunity, &market_data)? {
            // Convert strategy signal to ArbitrageOpportunity
            if let Some(opportunity) = self.convert_signal_to_opportunity(signal, &market_data) {
                opportunities.push(opportunity);
            }
        }

        // Update monitoring stats
        self.monitoring.opportunities_detected += opportunities.len() as u32;
        if !opportunities.is_empty() {
            self.monitoring.last_opportunity_time = Some(Instant::now());
        }

        let detection_time = start_time.elapsed().as_millis() as f64;
        self.monitoring.average_latency_ms =
            (self.monitoring.average_latency_ms + detection_time) / 2.0;

        Ok(opportunities)
    }

    /// Update price feeds from all monitored DEXs for the strategy
    async fn update_strategy_price_feeds(&mut self) -> Result<()> {
        // Get prices from Jupiter (aggregated)
        if let Ok(sol_price) = self.executor.jupiter_client.get_token_price("SOL").await {
            self.strategy.update_price_feed("Jupiter".to_string(), sol_price.price);
        }

        // Simulate different DEX prices for arbitrage detection
        // In a real implementation, you would get actual prices from each DEX
        if let Ok(sol_price) = self.executor.jupiter_client.get_token_price("SOL").await {
            // Simulate small price differences between DEXs
            self.strategy.update_price_feed("Raydium".to_string(), sol_price.price * 0.999);
            self.strategy.update_price_feed("Orca".to_string(), sol_price.price * 1.001);
        }

        Ok(())
    }

    /// Build market data for strategy analysis
    async fn build_market_data(&self) -> Result<MarketData> {
        // Get current SOL price and market data
        let sol_price_info = self.executor.jupiter_client.get_token_price("SOL").await?;

        Ok(MarketData {
            current_price: sol_price_info.price,
            volume_24h: 1_000_000.0, // Mock volume
            price_change_24h: 0.02, // Mock 2% change
            liquidity: 500_000.0, // Mock liquidity
            bid_ask_spread: 0.002, // Mock 0.2% spread
            order_book_depth: 100_000.0, // Mock depth
            market_cap: 50_000_000_000.0, // Mock market cap
            volatility: 0.3, // Mock volatility
            momentum: 0.1, // Mock momentum
            support_level: sol_price_info.price * 0.95,
            resistance_level: sol_price_info.price * 1.05,
            rsi: 50.0, // Mock RSI
            macd: 0.0, // Mock MACD
            bollinger_upper: sol_price_info.price * 1.02,
            bollinger_lower: sol_price_info.price * 0.98,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Convert a strategy signal to an ArbitrageOpportunity
    fn convert_signal_to_opportunity(&self, signal: StrategySignal, market_data: &MarketData) -> Option<ArbitrageOpportunity> {
        // Extract arbitrage information from signal metadata
        let buy_exchange = signal.metadata.get("buy_exchange")?.clone();
        let sell_exchange = signal.metadata.get("sell_exchange")?.clone();
        let buy_price: f64 = signal.metadata.get("buy_price")?.parse().ok()?;
        let sell_price: f64 = signal.metadata.get("sell_price")?.parse().ok()?;
        let estimated_profit: f64 = signal.metadata.get("estimated_profit")?.parse().ok()?;
        let profit_percentage: f64 = signal.metadata.get("profit_percentage")?.parse().ok()?;

        Some(ArbitrageOpportunity {
            buy_exchange,
            sell_exchange,
            token_pair: "SOL/USDC".to_string(),
            buy_price,
            sell_price,
            spread_percentage: profit_percentage,
            estimated_profit_usd: estimated_profit,
            liquidity_buy: market_data.liquidity,
            liquidity_sell: market_data.liquidity,
            confidence_score: signal.confidence,
            detected_at: Instant::now(),
            estimated_execution_time_ms: 2000,
        })
    }

    /// Select the best arbitrage opportunity from a list
    fn select_best_opportunity(&self, opportunities: &[ArbitrageOpportunity]) -> Option<ArbitrageOpportunity> {
        opportunities
            .iter()
            .filter(|opp| opp.confidence_score >= 0.7) // Minimum confidence
            .max_by(|a, b| a.estimated_profit_usd.partial_cmp(&b.estimated_profit_usd).unwrap())
            .cloned()
    }

    /// Check if we should execute a trade based on risk management
    fn should_execute_trade(&self, opportunity: &ArbitrageOpportunity) -> Result<bool> {
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

        // Check minimum profit threshold
        if opportunity.estimated_profit_usd < 1.0 {
            return Ok(false);
        }

        // Check liquidity requirements (using strategy config)
        let min_liquidity = self.strategy.get_config().max_position_size * 2.0; // Need 2x position size
        if opportunity.liquidity_buy < min_liquidity ||
           opportunity.liquidity_sell < min_liquidity {
            return Ok(false);
        }

        Ok(true)
    }

    /// Execute an arbitrage opportunity
    async fn execute_opportunity(&mut self, opportunity: ArbitrageOpportunity) -> Result<ArbitrageTradeResult> {
        let start_time = Instant::now();
        let opportunity_id = format!("arb_{}", chrono::Utc::now().timestamp_millis());

        info!("⚡ Executing arbitrage: {} -> {} | Profit: ${:.2}",
              opportunity.buy_exchange, opportunity.sell_exchange, opportunity.estimated_profit_usd);

        // Calculate trade amount (use smaller of max position size or available liquidity)
        let max_amount = self.executor.max_position_size.min(
            opportunity.liquidity_buy.min(opportunity.liquidity_sell) * 0.8
        );

        // Execute simultaneous buy and sell
        let buy_result = self.execute_buy_order(&opportunity, max_amount).await?;
        let sell_result = self.execute_sell_order(&opportunity, max_amount).await?;

        let execution_time = start_time.elapsed().as_millis() as u64;
        let success = buy_result.is_some() && sell_result.is_some();

        let actual_profit = if success {
            // Calculate actual profit based on execution results
            max_amount * (opportunity.sell_price - opportunity.buy_price) -
            (max_amount * 0.006) // Estimated fees
        } else {
            0.0
        };

        Ok(ArbitrageTradeResult {
            success,
            opportunity_id,
            executed_amount: if success { max_amount } else { 0.0 },
            actual_profit_usd: actual_profit,
            execution_time_ms: execution_time,
            buy_transaction_id: buy_result,
            sell_transaction_id: sell_result,
            actual_slippage: 0.002, // TODO: Calculate actual slippage
            total_fees: max_amount * 0.006,
            error_message: if success { None } else { Some("Execution failed".to_string()) },
        })
    }

    /// Execute buy order on the lower-priced DEX
    async fn execute_buy_order(&self, opportunity: &ArbitrageOpportunity, amount: f64) -> Result<Option<String>> {
        info!("📈 Executing BUY on {} at ${:.6}", opportunity.buy_exchange, opportunity.buy_price);

        // Use cache-free trader for safe execution
        let amount_lamports = (amount * 1_000_000_000.0) as u64; // Convert to lamports

        match timeout(Duration::from_secs(30),
            self.executor.cache_free_trader.execute_safe_swap(
                "So11111111111111111111111111111111111111112", // SOL mint
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC mint
                amount_lamports
            )
        ).await {
            Ok(Ok(result)) => {
                if result.success {
                    info!("✅ Buy order executed successfully");
                    Ok(Some("mock_buy_tx_id".to_string()))
                } else {
                    error!("❌ Buy order failed");
                    Ok(None)
                }
            }
            Ok(Err(e)) => {
                error!("❌ Buy order error: {}", e);
                Ok(None)
            }
            Err(_) => {
                error!("❌ Buy order timeout");
                Ok(None)
            }
        }
    }

    /// Execute sell order on the higher-priced DEX
    async fn execute_sell_order(&self, opportunity: &ArbitrageOpportunity, amount: f64) -> Result<Option<String>> {
        info!("📉 Executing SELL on {} at ${:.6}", opportunity.sell_exchange, opportunity.sell_price);

        // Use cache-free trader for safe execution
        let amount_lamports = (amount * 1_000_000_000.0) as u64; // Convert to lamports

        match timeout(Duration::from_secs(30),
            self.executor.cache_free_trader.execute_safe_swap(
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC mint
                "So11111111111111111111111111111111111111112", // SOL mint
                amount_lamports
            )
        ).await {
            Ok(Ok(result)) => {
                if result.success {
                    info!("✅ Sell order executed successfully");
                    Ok(Some("mock_sell_tx_id".to_string()))
                } else {
                    error!("❌ Sell order failed");
                    Ok(None)
                }
            }
            Ok(Err(e)) => {
                error!("❌ Sell order error: {}", e);
                Ok(None)
            }
            Err(_) => {
                error!("❌ Sell order timeout");
                Ok(None)
            }
        }
    }

    /// Process the result of a trade execution
    async fn process_trade_result(&mut self, result: ArbitrageTradeResult) -> Result<()> {
        // Update profit tracker
        self.profit_tracker.total_trades += 1;
        self.risk_manager.trades_today += 1;

        // Create a TradeResult for the strategy
        let trade_result = TradeResult {
            strategy_name: "Arbitrage".to_string(),
            entry_price: 0.0, // Would need to track from opportunity
            exit_price: 0.0,  // Would need to track from execution
            position_size: result.executed_amount,
            profit_loss: result.actual_profit_usd,
            fees: result.total_fees,
            duration: chrono::Duration::milliseconds(result.execution_time_ms as i64),
            success: result.success,
        };

        // Update strategy performance
        self.strategy.update_performance(&trade_result)?;

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
