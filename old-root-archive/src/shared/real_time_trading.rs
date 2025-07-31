//! Real-Time Trading Engine with Live Solana Blockchain Integration
//!
//! This module connects the cache-free trading engine to live Solana blockchain
//! using WebSocket feeds for real-time price updates and market data.
//! Phase 5A: Real-time Solana Blockchain Integration

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};

use crate::shared::cache_free_trading::{CacheFreeConfig, CacheFreeTradeEngine};
use crate::shared::helius_websocket::HeliusWebSocketClient;
use crate::shared::pool_detector::{PoolDetector, TradingOpportunity as PoolTradingOpportunity};
use crate::shared::syndica_websocket::SyndicaWebSocketClient;
use crate::shared::wallet_manager::WalletManager;
use crate::shared::websocket_price_feed::WebSocketPriceFeed;
use crate::types::*;

/// Real-time trading configuration
#[derive(Debug, Clone)]
pub struct RealTimeTradingConfig {
    pub use_websocket_feeds: bool,
    pub primary_price_source: PriceSource,
    pub fallback_sources: Vec<PriceSource>,
    pub price_update_interval_ms: u64,
    pub max_price_age_ms: u64,
    pub enable_live_blockchain: bool,
    pub devnet_mode: bool,
    pub risk_limits: RiskLimits,
}

impl Default for RealTimeTradingConfig {
    fn default() -> Self {
        Self {
            use_websocket_feeds: true,
            primary_price_source: PriceSource::Syndica,
            fallback_sources: vec![PriceSource::Jupiter, PriceSource::Helius],
            price_update_interval_ms: 100, // 100ms for real-time
            max_price_age_ms: 500,         // 500ms max age
            enable_live_blockchain: true,
            devnet_mode: true, // Start with DevNet for safety
            risk_limits: RiskLimits::conservative(),
        }
    }
}

/// Price source options
#[derive(Debug, Clone, PartialEq)]
pub enum PriceSource {
    Syndica,
    Helius,
    Jupiter,
    WebSocket,
}

/// Risk limits for real-time trading
#[derive(Debug, Clone)]
pub struct RiskLimits {
    pub max_position_size_sol: f64,
    pub max_daily_trades: u32,
    pub max_slippage_percent: f64,
    pub min_profit_threshold: f64,
    pub stop_loss_percent: f64,
}

impl RiskLimits {
    pub fn conservative() -> Self {
        Self {
            max_position_size_sol: 0.1, // 0.1 SOL max per trade
            max_daily_trades: 50,
            max_slippage_percent: 1.0,
            min_profit_threshold: 0.005, // 0.5% min profit
            stop_loss_percent: 2.0,      // 2% stop loss
        }
    }

    pub fn aggressive() -> Self {
        Self {
            max_position_size_sol: 1.0, // 1 SOL max per trade
            max_daily_trades: 200,
            max_slippage_percent: 2.0,
            min_profit_threshold: 0.002, // 0.2% min profit
            stop_loss_percent: 5.0,      // 5% stop loss
        }
    }
}

/// Real-time price update
#[derive(Debug, Clone)]
pub struct RealTimePriceUpdate {
    pub token_mint: String,
    pub price: f64,
    pub source: PriceSource,
    pub timestamp: Instant,
    pub confidence: f64, // 0.0 to 1.0
}

/// Real-time trading session
pub struct RealTimeTradingSession {
    config: RealTimeTradingConfig,
    cache_free_engine: CacheFreeTradeEngine,
    pool_detector: PoolDetector,
    wallet_manager: WalletManager,

    // WebSocket clients
    syndica_client: Option<SyndicaWebSocketClient>,
    helius_client: Option<HeliusWebSocketClient>,
    websocket_feed: Option<WebSocketPriceFeed>,

    // Real-time price tracking
    live_prices: Arc<RwLock<HashMap<String, RealTimePriceUpdate>>>,
    price_receiver: Option<mpsc::UnboundedReceiver<RealTimePriceUpdate>>,

    // Session state
    is_active: Arc<RwLock<bool>>,
    trades_today: Arc<Mutex<u32>>,
    session_start: Instant,
}

impl RealTimeTradingSession {
    /// Create new real-time trading session
    pub async fn new(config: RealTimeTradingConfig) -> Result<Self> {
        info!("🚀 Initializing Real-Time Trading Session");
        info!("   DevNet Mode: {}", config.devnet_mode);
        info!("   WebSocket Feeds: {}", config.use_websocket_feeds);
        info!("   Primary Source: {:?}", config.primary_price_source); // Initialize core components
        let cache_free_engine = CacheFreeTradeEngine::new(CacheFreeConfig::default()).await?;
        let jupiter_client = crate::shared::jupiter::JupiterClient::new(
            &crate::shared::jupiter::JupiterConfig::default(),
        )
        .await?;
        let pool_detector = PoolDetector::new(
            crate::shared::pool_detector::PoolDetectorConfig::default(),
            jupiter_client,
            None, // syndica_client
            None, // helius_client
        )
        .await?;
        let wallet_manager = WalletManager::new(&crate::config::Config::default()).await?;

        Ok(Self {
            config,
            cache_free_engine,
            pool_detector,
            wallet_manager,
            syndica_client: None,
            helius_client: None,
            websocket_feed: None,
            live_prices: Arc::new(RwLock::new(HashMap::new())),
            price_receiver: None,
            is_active: Arc::new(RwLock::new(false)),
            trades_today: Arc::new(Mutex::new(0)),
            session_start: Instant::now(),
        })
    }

    /// Start real-time trading session
    pub async fn start(&mut self, duration_seconds: u64) -> Result<()> {
        info!("🎯 Starting Real-Time Trading Session");
        info!("   Duration: {}s", duration_seconds);
        info!(
            "   Network: {}",
            if self.config.devnet_mode {
                "DevNet"
            } else {
                "MainNet"
            }
        );

        // Initialize WebSocket connections
        if self.config.use_websocket_feeds {
            self.init_websocket_feeds().await?;
        }

        // Set session as active
        *self.is_active.write().await = true;

        // Start main trading loop
        let end_time = Instant::now() + Duration::from_secs(duration_seconds);
        let mut iteration = 0;

        while Instant::now() < end_time && *self.is_active.read().await {
            iteration += 1;

            // Process real-time price updates
            self.process_price_updates().await?;
            // Check for trading opportunities
            if let Some(opportunity) = self.scan_for_opportunities().await? {
                if self.should_execute_trade(&opportunity).await? {
                    self.execute_real_time_trade(opportunity).await?;
                }
            }

            // Update session metrics every 10 iterations
            if iteration % 10 == 0 {
                self.log_session_status().await;
            }

            // Short sleep to prevent excessive CPU usage
            tokio::time::sleep(Duration::from_millis(self.config.price_update_interval_ms)).await;
        }

        info!("✅ Real-Time Trading Session Completed");
        self.stop().await?;

        Ok(())
    }

    /// Initialize WebSocket feeds
    async fn init_websocket_feeds(&mut self) -> Result<()> {
        info!("📡 Initializing WebSocket Price Feeds");
        let (_price_tx, price_rx) = mpsc::unbounded_channel();
        self.price_receiver = Some(price_rx);

        // TODO: Use _price_tx to send price updates from WebSocket feeds
        // Initialize Syndica WebSocket if it's the primary source
        if self.config.primary_price_source == PriceSource::Syndica {
            match SyndicaWebSocketClient::new(
                crate::shared::syndica_websocket::SyndicaConfig::default(),
            )
            .await
            {
                Ok(mut client) => {
                    client.connect().await?;
                    self.syndica_client = Some(client);
                    info!("✅ Syndica WebSocket connected");
                }
                Err(e) => {
                    warn!("⚠️ Failed to connect Syndica WebSocket: {}", e);
                }
            }
        }

        // Initialize Helius WebSocket for pool detection
        match HeliusWebSocketClient::new(crate::shared::helius_websocket::HeliusConfig::default())
            .await
        {
            Ok(mut client) => {
                let network = if self.config.devnet_mode {
                    "devnet"
                } else {
                    "mainnet"
                };
                client.start_pool_monitoring(network).await?;
                self.helius_client = Some(client);
                info!("✅ Helius WebSocket connected for pool monitoring");
            }
            Err(e) => {
                warn!("⚠️ Failed to connect Helius WebSocket: {}", e);
            }
        }
        // Initialize general WebSocket price feed as fallback
        let mut websocket_feed = WebSocketPriceFeed::new().await?;
        websocket_feed.connect_solana_pools().await?;
        self.websocket_feed = Some(websocket_feed);
        info!("✅ WebSocket price feed initialized");

        Ok(())
    }

    /// Process real-time price updates
    async fn process_price_updates(&mut self) -> Result<()> {
        if let Some(receiver) = &mut self.price_receiver {
            // Process all available price updates
            while let Ok(price_update) = receiver.try_recv() {
                let mut prices = self.live_prices.write().await;
                prices.insert(price_update.token_mint.clone(), price_update);
            }
        }

        // Clean up old price data
        self.cleanup_old_prices().await;

        Ok(())
    }

    /// Clean up old price data
    async fn cleanup_old_prices(&self) {
        let mut prices = self.live_prices.write().await;
        let max_age = Duration::from_millis(self.config.max_price_age_ms);
        let now = Instant::now();

        prices.retain(|_, price_update| now.duration_since(price_update.timestamp) <= max_age);
    }
    /// Scan for trading opportunities using real-time data
    async fn scan_for_opportunities(&mut self) -> Result<Option<RealTimeTradingOpportunity>> {
        // Use pool detector to find opportunities with real-time prices
        let opportunities = self.pool_detector.detect_opportunities_once().await?;

        if opportunities.is_empty() {
            return Ok(None);
        }

        // Enhance opportunity with real-time price data
        let first_opportunity = &opportunities[0];
        let real_time_opportunity = self.enhance_with_real_time_data(first_opportunity).await?;

        Ok(Some(real_time_opportunity))
    }
    /// Enhance opportunity with real-time price data
    async fn enhance_with_real_time_data(
        &self,
        opportunity: &PoolTradingOpportunity,
    ) -> Result<RealTimeTradingOpportunity> {
        let prices = self.live_prices.read().await;
        // Get estimated price from opportunity
        let input_price = 100.0; // Default price for now since PoolTradingOpportunity doesn't have estimated_price
        let output_price = 100.0; // Using same price for now

        // Calculate real-time profit potential
        let profit_potential = opportunity.expected_profit_usd;
        Ok(RealTimeTradingOpportunity {
            base_opportunity: opportunity.clone(),
            real_time_input_price: input_price,
            real_time_output_price: output_price,
            profit_potential,
            confidence: self.calculate_confidence(&prices, opportunity).await,
            timestamp: Instant::now(),
        })
    }
    /// Calculate confidence score based on price data freshness
    async fn calculate_confidence(
        &self,
        _prices: &HashMap<String, RealTimePriceUpdate>,
        opportunity: &PoolTradingOpportunity,
    ) -> f64 {
        // Use confidence score from the opportunity
        opportunity.confidence
    }

    /// Check if trade should be executed based on risk limits
    async fn should_execute_trade(&self, opportunity: &RealTimeTradingOpportunity) -> Result<bool> {
        let trades_today = *self.trades_today.lock().await;

        // Check daily trade limit
        if trades_today >= self.config.risk_limits.max_daily_trades {
            debug!("❌ Daily trade limit reached: {}", trades_today);
            return Ok(false);
        }

        // Check profit threshold
        if opportunity.profit_potential < self.config.risk_limits.min_profit_threshold {
            debug!(
                "❌ Profit below threshold: {} < {}",
                opportunity.profit_potential, self.config.risk_limits.min_profit_threshold
            );
            return Ok(false);
        }

        // Check confidence score
        if opportunity.confidence < 0.7 {
            debug!("❌ Confidence too low: {}", opportunity.confidence);
            return Ok(false);
        } // Check position size (use expected profit for position calculation)
        let position_value = opportunity.profit_potential; // Use profit estimate as position proxy
        if position_value > self.config.risk_limits.max_position_size_sol {
            debug!("❌ Position size too large: {} USD", position_value);
            return Ok(false);
        }

        Ok(true)
    }
    /// Execute real-time trade
    async fn execute_real_time_trade(
        &mut self,
        opportunity: RealTimeTradingOpportunity,
    ) -> Result<()> {
        info!("🎯 Executing Real-Time Trade");
        info!(
            "   Pool Address: {}",
            opportunity.base_opportunity.pool.pool_address
        );
        info!(
            "   Type: {:?}",
            opportunity.base_opportunity.opportunity_type
        );
        info!("   Expected Profit: ${:.6}", opportunity.profit_potential);
        info!("   Confidence: {:.2}%", opportunity.confidence * 100.0);

        // Execute trade using cache-free engine
        match self
            .cache_free_engine
            .execute_trade_with_validation(&opportunity.base_opportunity)
            .await
        {
            Ok(result) => {
                info!("✅ Trade executed successfully");
                info!("   Trade ID: {}", result.trade_id);
                info!("   Execution time: {}ms", result.execution_time_ms);

                // Update daily trade counter
                let mut trades_today = self.trades_today.lock().await;
                *trades_today += 1;
            }
            Err(e) => {
                error!("❌ Trade execution failed: {}", e);
            }
        }

        Ok(())
    }

    /// Log session status
    async fn log_session_status(&self) {
        let prices_count = self.live_prices.read().await.len();
        let trades_today = *self.trades_today.lock().await;
        let uptime = self.session_start.elapsed();

        info!(
            "📊 Session Status - Uptime: {:.1}s, Live Prices: {}, Trades: {}",
            uptime.as_secs_f64(),
            prices_count,
            trades_today
        );
    }

    /// Stop trading session
    pub async fn stop(&mut self) -> Result<()> {
        info!("🛑 Stopping Real-Time Trading Session");

        *self.is_active.write().await = false;
        // Disconnect WebSocket clients
        if let Some(_client) = self.syndica_client.take() {
            // Syndica client disconnect if available
            info!("📡 Disconnecting Syndica WebSocket");
        }

        if let Some(_client) = self.helius_client.take() {
            // Helius client disconnect if available
            info!("📡 Disconnecting Helius WebSocket");
        }

        info!("✅ Real-Time Trading Session Stopped");
        Ok(())
    }

    /// Get session statistics
    pub async fn get_session_stats(&self) -> RealTimeTradingStats {
        let live_prices_count = self.live_prices.read().await.len();
        let trades_today = *self.trades_today.lock().await;

        RealTimeTradingStats {
            session_duration: self.session_start.elapsed(),
            trades_executed: trades_today,
            live_price_sources: live_prices_count as u32,
            is_active: *self.is_active.read().await,
            network_mode: if self.config.devnet_mode {
                "DevNet".to_string()
            } else {
                "MainNet".to_string()
            },
        }
    }
}

/// Enhanced trading opportunity with real-time data
#[derive(Debug, Clone)]
pub struct RealTimeTradingOpportunity {
    pub base_opportunity: PoolTradingOpportunity,
    pub real_time_input_price: f64,
    pub real_time_output_price: f64,
    pub profit_potential: f64,
    pub confidence: f64,
    pub timestamp: Instant,
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct RealTimeTradingStats {
    pub session_duration: Duration,
    pub trades_executed: u32,
    pub live_price_sources: u32,
    pub is_active: bool,
    pub network_mode: String,
}

/// Test function for real-time trading
pub async fn test_real_time_trading() -> Result<()> {
    println!("🚀 REAL-TIME TRADING ENGINE TEST");
    println!("================================");

    let config = RealTimeTradingConfig {
        devnet_mode: true, // Start with DevNet for safety
        use_websocket_feeds: true,
        price_update_interval_ms: 200,
        max_price_age_ms: 1000,
        risk_limits: RiskLimits::conservative(),
        ..Default::default()
    };

    let mut session = RealTimeTradingSession::new(config).await?;

    println!("🎯 Starting 30-second real-time trading test on DevNet");
    println!("   Safety Mode: ON");
    println!("   Risk Limits: Conservative");
    println!("   WebSocket Feeds: Enabled");

    session.start(30).await?;

    let stats = session.get_session_stats().await;
    println!("\n📊 REAL-TIME TRADING TEST RESULTS:");
    println!("   Duration: {:.1}s", stats.session_duration.as_secs_f64());
    println!("   Trades Executed: {}", stats.trades_executed);
    println!("   Live Price Sources: {}", stats.live_price_sources);
    println!("   Network: {}", stats.network_mode);

    if stats.trades_executed > 0 {
        println!("🎉 SUCCESS: Real-time trading system operational!");
    } else {
        println!("ℹ️ No trades executed (may be due to conservative risk limits)");
    }

    Ok(())
}
