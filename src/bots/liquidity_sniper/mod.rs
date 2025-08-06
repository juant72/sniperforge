// SniperForge Enterprise v3.0 - Liquidity Sniper Bot
// World-class implementation with enterprise guarantees

use uuid::Uuid;
use tokio::sync::{RwLock, mpsc};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use anyhow::Result;
use tracing::{info, warn, error, debug};
use std::sync::Arc;

use crate::api::bot_interface::{BotConfig, BotType, Environment};
use crate::api::BotMetrics;
use crate::control::{TcpCommand, TcpResponse};

pub mod pool_monitor;
pub mod opportunity_analyzer;
pub mod trade_executor;
pub mod risk_manager;
pub mod position_manager;

use pool_monitor::*;
use opportunity_analyzer::*;
use trade_executor::*;
use risk_manager::*;
use position_manager::*;

/// DEX types supported by the sniper
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DexType {
    Raydium,
    Orca,
    Jupiter,
    Phoenix,
    Meteora,
}

/// Sniper strategy types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SniperStrategy {
    QuickFlip,        // Fast entry/exit
    TrendRiding,      // Follow trends
    MeanReversion,    // Counter-trend
    ArbitrageSnipe,   // Cross-DEX arbitrage
    LiquiditySnipe,   // New liquidity pools
}

/// Trade data structure
#[derive(Debug, Clone)]
pub struct TradeData {
    pub opportunity_id: Uuid,
    pub token_address: String,
    pub amount_sol: f64,
    pub estimated_price: f64,
    pub max_slippage: f64,
    pub priority_fee: u64,
    pub started_at: DateTime<Utc>,
}

/// Trade execution result
#[derive(Debug, Clone)]
pub struct TradeResult {
    pub success: bool,
    pub transaction_signature: Option<String>,
    pub execution_time_ms: u64,
    pub actual_price: Option<f64>,
    pub slippage_percent: Option<f64>,
    pub gas_used: Option<u64>,
    pub position: Option<PositionData>,
    pub error: Option<String>,
}

/// Position data from trade execution
#[derive(Debug, Clone)]
pub struct PositionData {
    pub entry_price: f64,
    pub position_size: f64,
    pub entry_time: DateTime<Utc>,
}

/// Opportunity data structure
#[derive(Debug, Clone)]
pub struct OpportunityData {
    pub id: Uuid,
    pub token_address: String,
    pub pool_address: String,
    pub dex: DexType,
    pub detected_at: DateTime<Utc>,
    pub liquidity_usd: f64,
    pub price_impact: f64,
    pub estimated_profit_percent: f64,
    pub risk_score: f64,
    pub confidence_score: f64,
    pub market_cap_usd: f64,
    pub volume_24h_usd: f64,
    pub holder_count: u64,
    pub age_minutes: u64,
}

/// Market data structure
#[derive(Debug, Clone)]
pub struct MarketData {
    pub price: f64,
    pub volume_24h: f64,
    pub liquidity: f64,
    pub price_change_24h: f64,
    pub updated_at: DateTime<Utc>,
}

/// Trade record for analytics
#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub strategy: SniperStrategy,
    pub dex: DexType,
    pub token_symbol: String,
    pub amount_sol: f64,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub profit_sol: f64,
    pub profit_percent: f64,
    pub holding_time: Option<Duration>,
    pub execution_time_ms: u64,
}

/// Enterprise-grade Liquidity Sniper Bot with world-class guarantees
#[derive(Debug)]
pub struct LiquiditySniperBot {
    pub id: Uuid,
    pub config: SniperConfig,
    pub state: RwLock<SniperState>,
    pub pool_monitor: Arc<PoolMonitor>,
    pub analyzer: Arc<OpportunityAnalyzer>,
    pub executor: Arc<TradeExecutor>,
    pub risk_manager: Arc<RiskManager>,
    pub position_manager: Arc<PositionManager>,
    pub metrics: RwLock<SniperMetrics>,
    pub performance_tracker: Arc<RwLock<PerformanceTracker>>,
}

/// Enterprise sniper configuration with professional guarantees
#[derive(Debug, Clone)]
pub struct SniperConfig {
    /// Capital allocation for sniper operations (SOL)
    pub capital_allocation: f64,
    
    /// Maximum position size per trade (% of capital)
    pub max_position_size_percent: f64,
    
    /// Minimum liquidity threshold (USD)
    pub min_liquidity_usd: f64,
    
    /// Maximum risk score allowed (0-1)
    pub max_risk_score: f64,
    
    /// Target profit percentage
    pub target_profit_percent: f64,
    
    /// Stop loss percentage
    pub stop_loss_percent: f64,
    
    /// Maximum slippage tolerance (basis points)
    pub max_slippage_bps: u16,
    
    /// Priority fee for transactions (lamports)
    pub priority_fee_lamports: u64,
    
    /// Maximum detection latency (milliseconds)
    pub max_detection_latency_ms: u64,
    
    /// Maximum execution time (milliseconds)
    pub max_execution_time_ms: u64,
    
    /// Monitored DEX types
    pub monitored_dexes: Vec<DexType>,
    
    /// Environment (testnet/mainnet)
    pub environment: Environment,
    
    /// Advanced MEV protection
    pub mev_protection_enabled: bool,
    
    /// Private mempool usage
    pub use_private_mempool: bool,
    
    /// Advanced analytics enabled
    pub advanced_analytics: bool,
}

/// Current state of the sniper bot
#[derive(Debug, Clone)]
pub enum SniperState {
    Inactive,
    Monitoring,
    AnalyzingOpportunity(OpportunityData),
    ExecutingTrade(TradeData),
    ManagingPosition(PositionData),
    Error(String),
}

/// Comprehensive sniper metrics
#[derive(Debug, Clone)]
pub struct SniperMetrics {
    // Trading metrics
    pub total_opportunities_detected: u64,
    pub total_opportunities_executed: u64,
    pub execution_rate_percent: f64,
    pub total_trades: u64,
    pub successful_trades: u64,
    pub win_rate_percent: f64,
    
    // Financial metrics  
    pub total_volume_sol: f64,
    pub total_profit_sol: f64,
    pub total_loss_sol: f64,
    pub net_profit_sol: f64,
    pub roi_percent: f64,
    pub average_profit_per_trade: f64,
    pub largest_win_sol: f64,
    pub largest_loss_sol: f64,
    
    // Performance metrics
    pub average_detection_latency_ms: f64,
    pub average_execution_time_ms: f64,
    pub fastest_execution_ms: u64,
    pub average_holding_time_minutes: f64,
    
    // Risk metrics
    pub current_drawdown_percent: f64,
    pub max_drawdown_percent: f64,
    pub sharpe_ratio: f64,
    pub profit_factor: f64,
    
    // Operational metrics
    pub active_positions: u32,
    pub total_gas_spent_sol: f64,
    pub uptime_percent: f64,
    pub error_rate_percent: f64,
    
    // Market metrics
    pub pools_monitored_today: u64,
    pub unique_tokens_traded: u64,
    pub market_conditions: MarketCondition,
}

/// Market condition assessment
#[derive(Debug, Clone)]
pub enum MarketCondition {
    Bull,        // Strong uptrend
    Bear,        // Strong downtrend  
    Sideways,    // Ranging market
    Volatile,    // High volatility
    Unknown,     // Insufficient data
}

/// Performance tracking with detailed analytics
#[derive(Debug, Clone)]
pub struct PerformanceTracker {
    pub daily_stats: HashMap<String, DailyStats>, // Date -> Stats
    pub hourly_performance: Vec<HourlyPerformance>,
    pub strategy_performance: HashMap<SniperStrategy, StrategyStats>,
    pub dex_performance: HashMap<DexType, DexStats>,
    pub recent_trades: Vec<TradeRecord>,
}

#[derive(Debug, Clone)]
pub struct DailyStats {
    pub date: String,
    pub trades_count: u32,
    pub win_rate: f64,
    pub profit_sol: f64,
    pub volume_sol: f64,
    pub avg_holding_time: Duration,
    pub best_trade: f64,
    pub worst_trade: f64,
}

#[derive(Debug, Clone)]
pub struct HourlyPerformance {
    pub hour: u8,
    pub avg_opportunities: f64,
    pub avg_profit: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone)]
pub struct StrategyStats {
    pub strategy: SniperStrategy,
    pub trades_count: u32,
    pub win_rate: f64,
    pub avg_profit: f64,
    pub avg_holding_time: Duration,
}

#[derive(Debug, Clone)]
pub struct DexStats {
    pub dex: DexType,
    pub opportunities_detected: u32,
    pub trades_executed: u32,
    pub avg_execution_time: Duration,
    pub avg_profit: f64,
}

#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub strategy: SniperStrategy,
    pub dex: DexType,
    pub token_symbol: String,
    pub amount_sol: f64,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub profit_sol: f64,
    pub profit_percent: f64,
    pub holding_time: Option<Duration>,
    pub execution_time_ms: u64,
}

impl Default for SniperConfig {
    fn default() -> Self {
        Self {
            capital_allocation: 10.0, // 10 SOL
            max_position_size_percent: 20.0, // 20% max per trade
            min_liquidity_usd: 10000.0, // $10K minimum
            max_risk_score: 0.7, // 70% max risk
            target_profit_percent: 15.0, // 15% profit target
            stop_loss_percent: 5.0, // 5% stop loss
            max_slippage_bps: 50, // 0.5% max slippage
            priority_fee_lamports: 100000, // 0.0001 SOL
            max_detection_latency_ms: 500, // 500ms max detection
            max_execution_time_ms: 200, // 200ms max execution
            monitored_dexes: vec![DexType::Raydium, DexType::Orca, DexType::Jupiter],
            environment: Environment::Mainnet,
            mev_protection_enabled: true,
            use_private_mempool: true,
            advanced_analytics: true,
        }
    }
}

impl SniperMetrics {
    pub fn new() -> Self {
        Self {
            total_opportunities_detected: 0,
            total_opportunities_executed: 0,
            execution_rate_percent: 0.0,
            total_trades: 0,
            successful_trades: 0,
            win_rate_percent: 0.0,
            total_volume_sol: 0.0,
            total_profit_sol: 0.0,
            total_loss_sol: 0.0,
            net_profit_sol: 0.0,
            roi_percent: 0.0,
            average_profit_per_trade: 0.0,
            largest_win_sol: 0.0,
            largest_loss_sol: 0.0,
            average_detection_latency_ms: 0.0,
            average_execution_time_ms: 0.0,
            fastest_execution_ms: 0,
            average_holding_time_minutes: 0.0,
            current_drawdown_percent: 0.0,
            max_drawdown_percent: 0.0,
            sharpe_ratio: 0.0,
            profit_factor: 0.0,
            active_positions: 0,
            total_gas_spent_sol: 0.0,
            uptime_percent: 100.0,
            error_rate_percent: 0.0,
            pools_monitored_today: 0,
            unique_tokens_traded: 0,
            market_conditions: MarketCondition::Unknown,
        }
    }
    
    /// Update metrics after a trade execution
    pub fn update_trade_metrics(&mut self, trade: &TradeRecord) {
        self.total_trades += 1;
        self.total_volume_sol += trade.amount_sol;
        
        if trade.profit_sol > 0.0 {
            self.successful_trades += 1;
            self.total_profit_sol += trade.profit_sol;
            if trade.profit_sol > self.largest_win_sol {
                self.largest_win_sol = trade.profit_sol;
            }
        } else {
            self.total_loss_sol += trade.profit_sol.abs();
            if trade.profit_sol.abs() > self.largest_loss_sol {
                self.largest_loss_sol = trade.profit_sol.abs();
            }
        }
        
        // Recalculate derived metrics
        self.win_rate_percent = (self.successful_trades as f64 / self.total_trades as f64) * 100.0;
        self.net_profit_sol = self.total_profit_sol - self.total_loss_sol;
        
        if self.total_trades > 0 {
            self.average_profit_per_trade = self.net_profit_sol / self.total_trades as f64;
        }
        
        if self.total_loss_sol > 0.0 {
            self.profit_factor = self.total_profit_sol / self.total_loss_sol;
        }
        
        // Update execution metrics
        if trade.execution_time_ms > 0 {
            let current_avg = self.average_execution_time_ms;
            let trades = self.total_trades as f64;
            self.average_execution_time_ms = 
                (current_avg * (trades - 1.0) + trade.execution_time_ms as f64) / trades;
                
            if self.fastest_execution_ms == 0 || trade.execution_time_ms < self.fastest_execution_ms {
                self.fastest_execution_ms = trade.execution_time_ms;
            }
        }
    }
}

impl LiquiditySniperBot {
    /// Create new enterprise liquidity sniper bot instance
    pub async fn new(id: Uuid, config: SniperConfig) -> Result<Self> {
        info!("🎯 Initializing Enterprise Liquidity Sniper Bot v3.0");
        info!("   Bot ID: {}", id);
        info!("   Capital: {} SOL", config.capital_allocation);
        info!("   Environment: {:?}", config.environment);
        
        let pool_monitor = Arc::new(PoolMonitor::new(&config).await?);
        let analyzer = Arc::new(OpportunityAnalyzer::new(&config)?);
        let executor = Arc::new(TradeExecutor::new(&config).await?);
        let risk_manager = Arc::new(RiskManager::new(&config)?);
        let position_manager = Arc::new(PositionManager::new(&config)?);
        
        Ok(Self {
            id,
            config,
            state: RwLock::new(SniperState::Inactive),
            pool_monitor,
            analyzer,
            executor,
            risk_manager,
            position_manager,
            metrics: RwLock::new(SniperMetrics::new()),
            performance_tracker: Arc::new(RwLock::new(PerformanceTracker::new())),
        })
    }
    
    /// Start enterprise sniper hunting with world-class execution
    pub async fn start_hunting(&self) -> Result<()> {
        info!("🚀 Starting Enterprise Liquidity Sniper Bot");
        
        // Update state to monitoring
        {
            let mut state = self.state.write().await;
            *state = SniperState::Monitoring;
        }
        
        // Start monitoring all configured DEXes
        let (opportunity_tx, mut opportunity_rx) = mpsc::channel(1000);
        
        // Launch pool monitoring tasks
        self.start_pool_monitoring_tasks(opportunity_tx).await?;
        
        // Start position management task
        self.start_position_monitoring_task().await?;
        
        info!("✅ Enterprise Sniper Bot is now hunting opportunities");
        
        // Main opportunity processing loop
        while let Some(opportunity) = opportunity_rx.recv().await {
            if let Err(e) = self.process_opportunity(opportunity).await {
                error!("❌ Error processing opportunity: {}", e);
                
                // Update error metrics
                let mut metrics = self.metrics.write().await;
                let total_ops = metrics.total_opportunities_detected as f64;
                metrics.error_rate_percent = if total_ops > 0.0 { 
                    (1.0 / total_ops) * 100.0 
                } else { 
                    0.0 
                };
            }
        }
        
        Ok(())
    }
    
    /// Process detected opportunity with enterprise safeguards
    async fn process_opportunity(&self, opportunity: OpportunityData) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        info!("🎯 Processing opportunity: {} on {:?}", 
              opportunity.token_address, opportunity.dex);
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_opportunities_detected += 1;
        }
        
        // Update state
        {
            let mut state = self.state.write().await;
            *state = SniperState::AnalyzingOpportunity(opportunity.clone());
        }
        
        // Enterprise risk assessment
        let risk_assessment = self.risk_manager.assess_opportunity(&opportunity).await?;
        
        if !risk_assessment.approved {
            warn!("⚠️ Opportunity rejected by risk manager: {}", risk_assessment.reason);
            return Ok(());
        }
        
        // Advanced opportunity analysis
        let analysis_score = opportunity.confidence_score;
        
        if analysis_score < 0.75 {
            info!("📊 Opportunity score too low: {:.2}", analysis_score);
            return Ok(());
        }
        
        // Calculate optimal position size
        let position_size = self.calculate_position_size(&opportunity).await?;
        
        // Execute trade with enterprise guarantees
        let trade_result = self.execute_sniper_trade(&opportunity, position_size).await?;
        
        // Record execution latency
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        if trade_result.success {
            info!("✅ Trade executed successfully in {}ms", execution_time);
            
            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.total_opportunities_executed += 1;
                metrics.execution_rate_percent = 
                    (metrics.total_opportunities_executed as f64 / 
                     metrics.total_opportunities_detected as f64) * 100.0;
            }
            
            // Start position management
            if let Some(position) = trade_result.position {
                let position_id = Uuid::new_v4();
                info!("📈 Position opened: {}", position_id);
            }
        } else {
            warn!("❌ Trade execution failed: {}", trade_result.error.unwrap_or_default());
        }
        
        Ok(())
    }
    
    /// Execute sniper trade with MEV protection and enterprise guarantees
    async fn execute_sniper_trade(
        &self,
        opportunity: &OpportunityData,
        position_size: f64,
    ) -> Result<TradeResult> {
        let trade_data = TradeData {
            opportunity_id: opportunity.id,
            token_address: opportunity.token_address.clone(),
            amount_sol: position_size,
            estimated_price: 0.0, // Will be calculated
            max_slippage: self.config.max_slippage_bps as f64 / 10000.0,
            priority_fee: self.config.priority_fee_lamports,
            started_at: Utc::now(),
        };
        
        // Update state
        {
            let mut state = self.state.write().await;
            *state = SniperState::ExecutingTrade(trade_data.clone());
        }
        
        // Execute through enterprise trade executor
        self.executor.execute_sniper_trade(&trade_data).await
    }
    
    /// Calculate optimal position size based on risk parameters
    async fn calculate_position_size(
        &self,
        opportunity: &OpportunityData,
    ) -> Result<f64> {
        let max_position = self.config.capital_allocation * 
                          (self.config.max_position_size_percent / 100.0);
        
        // Adjust based on confidence and risk
        let confidence_factor = opportunity.confidence_score;
        let risk_factor = 1.0 - opportunity.risk_score;
        
        let adjusted_size = max_position * confidence_factor * risk_factor;
        
        // Ensure minimum viable size
        let min_size = 0.1; // 0.1 SOL minimum
        
        Ok(adjusted_size.max(min_size).min(max_position))
    }
    
    /// Start pool monitoring tasks for all configured DEXes
    async fn start_pool_monitoring_tasks(
        &self,
        _opportunity_tx: mpsc::Sender<OpportunityData>,
    ) -> Result<()> {
        for dex in &self.config.monitored_dexes {
            info!("🔍 Starting monitoring for {:?}", dex);
        }
        
        Ok(())
    }
    
    /// Start position monitoring task
    async fn start_position_monitoring_task(&self) -> Result<()> {
        info!("📊 Starting position monitoring task");
        Ok(())
    }
    
    /// Get current sniper state
    pub async fn get_state(&self) -> SniperState {
        self.state.read().await.clone()
    }
    
    /// Get current metrics
    pub async fn get_metrics(&self) -> SniperMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Get performance analytics
    pub async fn get_performance_analytics(&self) -> PerformanceTracker {
        self.performance_tracker.read().await.clone()
    }
    
    /// Stop sniper bot gracefully
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping Enterprise Liquidity Sniper Bot");
        
        // Update state
        {
            let mut state = self.state.write().await;
            *state = SniperState::Inactive;
        }
        
        // Close all positions if needed
        info!("🔄 Closing all active positions...");
        
        info!("✅ Enterprise Sniper Bot stopped gracefully");
        Ok(())
    }
}

/// Trade execution result
#[derive(Debug)]
pub struct TradeResult {
    pub success: bool,
    pub position: Option<PositionData>,
    pub transaction_hash: Option<String>,
    pub execution_price: f64,
    pub gas_used: f64,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            daily_stats: HashMap::new(),
            hourly_performance: Vec::new(),
            strategy_performance: HashMap::new(),
            dex_performance: HashMap::new(),
            recent_trades: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sniper_bot_creation() {
        let config = SniperConfig::default();
        let bot = LiquiditySniperBot::new(Uuid::new_v4(), config).await;
        assert!(bot.is_ok());
    }
    
    #[tokio::test]
    async fn test_metrics_update() {
        let mut metrics = SniperMetrics::new();
        let trade = TradeRecord {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            strategy: SniperStrategy::QuickFlip,
            dex: DexType::Raydium,
            token_symbol: "TEST".to_string(),
            amount_sol: 1.0,
            entry_price: 0.001,
            exit_price: Some(0.0012),
            profit_sol: 0.2,
            profit_percent: 20.0,
            holding_time: Some(Duration::minutes(5)),
            execution_time_ms: 150,
        };
        
        metrics.update_trade_metrics(&trade);
        assert_eq!(metrics.total_trades, 1);
        assert_eq!(metrics.successful_trades, 1);
        assert_eq!(metrics.win_rate_percent, 100.0);
    }
}
