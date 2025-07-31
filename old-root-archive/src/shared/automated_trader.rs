//! Automated Paper Trading System
//!
//! Connects pool detection to automated paper trading with full end-to-end execution
//! Phase 3: Paper Trading Automation

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::performance_tracker::{PerformanceTracker, TradeMetric};
use super::pool_detector::{PoolDetector, TradingOpportunity};
use super::risk_manager::RiskManager;
use super::trade_executor::{TradeExecutor, TradeRequest, TradeResult, TradingMode};
use crate::config::Config;

/// Automated trading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedTradingConfig {
    pub enabled: bool,
    pub max_trades_per_hour: u32,
    pub max_daily_loss: f64,
    pub min_profit_target: f64,
    pub max_position_size: f64,
    pub confidence_threshold: f64,
    pub stop_loss_percentage: f64,
    pub max_slippage_bps: u16,
    pub trading_mode: TradingMode,
    pub monitor_duration_seconds: u64,
}

impl Default for AutomatedTradingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_trades_per_hour: 10,
            max_daily_loss: 100.0,      // $100 max daily loss
            min_profit_target: 25.0,    // $25 minimum profit target
            max_position_size: 50.0,    // $50 max position
            confidence_threshold: 65.0, // 65% minimum confidence
            stop_loss_percentage: 5.0,  // 5% stop loss
            max_slippage_bps: 150,      // 1.5% max slippage
            trading_mode: TradingMode::DevNet,
            monitor_duration_seconds: 3600, // 1 hour default
        }
    }
}

/// Automated trading session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSession {
    pub session_id: String,
    pub start_time: u64, // Unix timestamp instead of Instant for serialization
    pub config: AutomatedTradingConfig,
    pub opportunities_detected: u32,
    pub trades_executed: u32,
    pub successful_trades: u32,
    pub total_profit_loss: f64,
    pub active: bool,
}

impl TradingSession {
    pub fn new(config: AutomatedTradingConfig) -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            config,
            opportunities_detected: 0,
            trades_executed: 0,
            successful_trades: 0,
            total_profit_loss: 0.0,
            active: true,
        }
    }

    pub fn elapsed_minutes(&self) -> f64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        (now - self.start_time) as f64 / 60.0
    }
}

/// Main Automated Trader
pub struct AutomatedTrader {
    detector: Arc<Mutex<PoolDetector>>,
    executor: Arc<TradeExecutor>,
    risk_manager: Arc<RiskManager>,
    performance_tracker: Arc<Mutex<PerformanceTracker>>,
    config: AutomatedTradingConfig,
    current_session: Arc<Mutex<Option<TradingSession>>>,
}

impl AutomatedTrader {
    /// Create new automated trader
    pub async fn new(config: Config, trading_config: AutomatedTradingConfig) -> Result<Self> {
        info!("🤖 Initializing Automated Paper Trader");
        info!("   Mode: {:?}", trading_config.trading_mode);
        info!("   Max trades/hour: {}", trading_config.max_trades_per_hour);
        info!("   Min profit: ${}", trading_config.min_profit_target); // Initialize components
        let jupiter_config = super::jupiter::JupiterConfig::default();
        let jupiter_client = crate::shared::jupiter::JupiterClient::new(&jupiter_config).await?;
        let detector = Arc::new(Mutex::new(
            PoolDetector::new(
                super::pool_detector::PoolDetectorConfig::default(),
                jupiter_client,
                None, // No Syndica client for now
                None, // No Helius client for now
            )
            .await?,
        ));
        let executor = Arc::new(
            TradeExecutor::new(config.clone(), trading_config.trading_mode.clone()).await?,
        );
        let risk_manager = Arc::new(RiskManager::new(trading_config.clone()).await?);
        let performance_tracker = Arc::new(Mutex::new(PerformanceTracker::new()));

        info!("✅ Automated Trader initialized successfully");

        Ok(Self {
            detector,
            executor,
            risk_manager,
            performance_tracker,
            config: trading_config,
            current_session: Arc::new(Mutex::new(None)),
        })
    }

    /// Start automated paper trading session
    pub async fn start_automated_trading(&self) -> Result<()> {
        let session = TradingSession::new(self.config.clone());
        info!(
            "🚀 Starting automated trading session: {}",
            session.session_id
        );
        info!(
            "   Duration: {} seconds",
            self.config.monitor_duration_seconds
        );
        info!("   Trading mode: {:?}", self.config.trading_mode);

        // Update current session
        {
            let mut current = self.current_session.lock().await;
            *current = Some(session.clone());
        } // Enable detector for trading session
        {
            let _detector = self.detector.lock().await;
            info!("🔥 Detector ready for trading session");
        }

        // Start the trading loop
        let result = self.run_trading_loop(session).await;

        // Clean up session
        {
            let mut current = self.current_session.lock().await;
            if let Some(ref mut session) = current.as_mut() {
                session.active = false;
            }
        }

        result
    }

    /// Main trading loop with pool detection and automated execution
    async fn run_trading_loop(&self, mut session: TradingSession) -> Result<()> {
        info!("🔄 Starting automated trading loop");
        let start_time = Instant::now();
        let duration = Duration::from_secs(self.config.monitor_duration_seconds);

        while start_time.elapsed() < duration && session.active {
            // Check for new opportunities
            match self.detect_trading_opportunities().await {
                Ok(opportunities) => {
                    if !opportunities.is_empty() {
                        session.opportunities_detected += opportunities.len() as u32;
                        info!("📊 Detected {} new opportunities", opportunities.len());

                        // Process each opportunity
                        for opportunity in opportunities {
                            if self.should_execute_trade(&opportunity, &session).await? {
                                match self
                                    .execute_automated_trade(opportunity, &mut session)
                                    .await
                                {
                                    Ok(trade_result) => {
                                        session.trades_executed += 1;
                                        if trade_result.success {
                                            session.successful_trades += 1;
                                            session.total_profit_loss +=
                                                self.calculate_profit(&trade_result);
                                            info!(
                                                "✅ Trade executed successfully | Profit: ${:.2}",
                                                self.calculate_profit(&trade_result)
                                            );
                                        } else {
                                            warn!(
                                                "❌ Trade failed: {:?}",
                                                trade_result.error_message
                                            );
                                        }

                                        // Track performance
                                        self.track_trade_performance(&trade_result).await?;
                                    }
                                    Err(e) => {
                                        error!("💥 Trade execution error: {}", e);
                                    }
                                }

                                // Check rate limits
                                if !self.check_rate_limits(&session).await? {
                                    warn!("⚠️ Rate limit reached, pausing trading");
                                    sleep(Duration::from_secs(60)).await;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("🔍 Pool detection error: {}", e);
                    sleep(Duration::from_secs(5)).await;
                }
            }

            // Brief pause between detection cycles
            sleep(Duration::from_millis(500)).await; // Update session in shared state
            {
                let mut current = self.current_session.lock().await;
                *current = Some(session.clone());
            }

            // Log progress every 60 seconds
            if start_time.elapsed().as_secs() % 60 == 0 && start_time.elapsed().as_secs() > 0 {
                self.log_session_progress(&session).await;
            }
        }

        // Final session summary
        self.log_session_summary(&session).await;
        Ok(())
    }

    /// Detect trading opportunities using pool detector
    async fn detect_trading_opportunities(&self) -> Result<Vec<TradingOpportunity>> {
        let mut detector = self.detector.lock().await;

        // Get opportunities
        let opportunities = detector.detect_opportunities_once().await?;

        // Filter for high-quality opportunities
        let filtered_opportunities: Vec<TradingOpportunity> = opportunities
            .into_iter()
            .filter(|opp| {
                opp.confidence >= self.config.confidence_threshold
                    && opp.expected_profit_usd >= self.config.min_profit_target
            })
            .collect();

        debug!(
            "🎯 Filtered {} high-quality opportunities",
            filtered_opportunities.len()
        );
        Ok(filtered_opportunities)
    }

    /// Determine if we should execute a trade for this opportunity
    async fn should_execute_trade(
        &self,
        opportunity: &TradingOpportunity,
        session: &TradingSession,
    ) -> Result<bool> {
        // Check risk management constraints
        if !self.risk_manager.validate_opportunity(opportunity).await? {
            debug!("❌ Risk manager rejected opportunity");
            return Ok(false);
        }

        // Check session limits
        if session.trades_executed >= self.config.max_trades_per_hour {
            debug!("❌ Hourly trade limit reached");
            return Ok(false);
        }

        // Check daily loss limits
        if session.total_profit_loss <= -self.config.max_daily_loss {
            warn!(
                "❌ Daily loss limit reached: ${:.2}",
                session.total_profit_loss
            );
            return Ok(false);
        }

        // Check opportunity quality
        if opportunity.confidence < self.config.confidence_threshold {
            debug!(
                "❌ Opportunity confidence too low: {:.1}%",
                opportunity.confidence
            );
            return Ok(false);
        }

        if opportunity.expected_profit_usd < self.config.min_profit_target {
            debug!(
                "❌ Estimated profit too low: ${:.2}",
                opportunity.expected_profit_usd
            );
            return Ok(false);
        }
        debug!("✅ Trade approved for execution");
        Ok(true)
    }

    /// Execute automated trade for approved opportunity
    async fn execute_automated_trade(
        &self,
        opportunity: TradingOpportunity,
        _session: &mut TradingSession,
    ) -> Result<TradeResult> {
        info!(
            "🎯 Executing automated trade for opportunity: {}",
            opportunity.pool.pool_address
        );

        // Create trade request
        let trade_request = TradeRequest {
            input_mint: opportunity
                .pool
                .token_a
                .mint
                .parse()
                .map_err(|e| anyhow!("Invalid input mint address: {}", e))?,
            output_mint: opportunity
                .pool
                .token_b
                .mint
                .parse()
                .map_err(|e| anyhow!("Invalid output mint address: {}", e))?,
            amount_in: self.calculate_position_size(&opportunity)?,
            slippage_bps: self.config.max_slippage_bps,
            wallet_name: "trading_wallet".to_string(), // Default trading wallet
            max_price_impact: opportunity.pool.price_impact_1k,
            trading_mode: self.config.trading_mode.clone(),
        };

        // Execute the trade
        let trade_result = self.executor.execute_trade(trade_request).await?;

        info!(
            "📊 Trade result: {} | Profit: ${:.2}",
            if trade_result.success {
                "SUCCESS"
            } else {
                "FAILED"
            },
            self.calculate_profit(&trade_result)
        );

        Ok(trade_result)
    }

    /// Calculate appropriate position size based on opportunity and risk management
    fn calculate_position_size(&self, opportunity: &TradingOpportunity) -> Result<u64> {
        // Use confidence score to adjust position size
        let confidence_factor = opportunity.confidence / 100.0;
        let base_size = self.config.max_position_size * confidence_factor;

        // Convert to lamports (assuming SOL-based trades)
        let position_size_lamports = (base_size * 1_000_000_000.0) as u64;

        debug!(
            "💰 Calculated position size: {} lamports (${:.2})",
            position_size_lamports, base_size
        );

        Ok(position_size_lamports)
    }

    /// Calculate profit from trade result
    fn calculate_profit(&self, trade_result: &TradeResult) -> f64 {
        if !trade_result.success {
            return 0.0;
        }
        // Simple profit calculation for paper trading
        let input_value = trade_result.input_amount as f64 / 1_000_000_000.0; // Convert from lamports
        let output_value = trade_result.output_amount as f64 / 1_000_000_000.0;

        output_value - input_value - trade_result.gas_fee
    }

    /// Track trade performance metrics
    async fn track_trade_performance(&self, trade_result: &TradeResult) -> Result<()> {
        let metric = TradeMetric {
            timestamp: chrono::Utc::now(),
            success: trade_result.success,
            profit_loss: self.calculate_profit(trade_result),
            execution_time_ms: trade_result.execution_time_ms,
            slippage: trade_result.actual_slippage,
            gas_fee: trade_result.gas_fee,
        };

        let mut tracker = self.performance_tracker.lock().await;
        tracker.add_trade_metric(metric).await?;

        Ok(())
    }

    /// Check if we're within rate limits
    async fn check_rate_limits(&self, session: &TradingSession) -> Result<bool> {
        let elapsed_hours = session.elapsed_minutes() / 60.0;
        let trades_per_hour = if elapsed_hours > 0.0 {
            session.trades_executed as f64 / elapsed_hours
        } else {
            0.0
        };

        Ok(trades_per_hour < self.config.max_trades_per_hour as f64)
    }

    /// Log session progress
    async fn log_session_progress(&self, session: &TradingSession) {
        let elapsed_minutes = session.elapsed_minutes();
        let success_rate = if session.trades_executed > 0 {
            (session.successful_trades as f64 / session.trades_executed as f64) * 100.0
        } else {
            0.0
        };

        info!("📊 SESSION PROGRESS ({:.1} min)", elapsed_minutes);
        info!(
            "   🎯 Opportunities: {} detected",
            session.opportunities_detected
        );
        info!(
            "   📈 Trades: {} executed, {} successful ({:.1}%)",
            session.trades_executed, session.successful_trades, success_rate
        );
        info!("   💰 P&L: ${:.2}", session.total_profit_loss);
    }

    /// Log final session summary
    async fn log_session_summary(&self, session: &TradingSession) {
        let total_minutes = session.elapsed_minutes();
        let success_rate = if session.trades_executed > 0 {
            (session.successful_trades as f64 / session.trades_executed as f64) * 100.0
        } else {
            0.0
        };

        info!("🎉 AUTOMATED TRADING SESSION COMPLETE");
        info!("   ⏱️ Duration: {:.1} minutes", total_minutes);
        info!(
            "   🎯 Opportunities detected: {}",
            session.opportunities_detected
        );
        info!("   📈 Trades executed: {}", session.trades_executed);
        info!(
            "   ✅ Successful trades: {} ({:.1}%)",
            session.successful_trades, success_rate
        );
        info!("   💰 Total P&L: ${:.2}", session.total_profit_loss);
        info!(
            "   📊 Avg profit per trade: ${:.2}",
            if session.successful_trades > 0 {
                session.total_profit_loss / session.successful_trades as f64
            } else {
                0.0
            }
        );
    }

    /// Get current session status
    pub async fn get_session_status(&self) -> Option<TradingSession> {
        let current = self.current_session.lock().await;
        current.clone()
    }

    /// Stop current trading session
    pub async fn stop_trading_session(&self) -> Result<()> {
        let mut current = self.current_session.lock().await;
        if let Some(ref mut session) = current.as_mut() {
            session.active = false;
            info!("🛑 Trading session stopped: {}", session.session_id);
        }
        Ok(())
    }

    /// Get performance summary
    pub async fn get_performance_summary(&self) -> Result<String> {
        let tracker = self.performance_tracker.lock().await;
        tracker.generate_summary().await
    }
}
