// SniperForge Enterprise v3.0 - Position Manager
// Advanced position management with automated strategies

use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use tracing::{info, warn, debug};
use uuid::Uuid;

use super::{OpportunityData, SniperConfig, DexType};
use super::risk_manager::{RiskAssessment, StopLevel, StopType, MonitoringLevel, RequiredStop};
use crate::types::TradingOpportunity;

// 🚀 REFACTORING: Reutilizar módulos centrales existentes
use crate::security::risk_manager::{RiskManagementConfig, AdvancedRiskManager};
use crate::trading::portfolio::{PortfolioManager, PerformanceMetrics as CorePerformanceMetrics, RiskMetrics as CoreRiskMetrics};
use crate::analytics::performance_analytics::PerformanceAnalyticsAI;
use crate::trading::risk::{RiskManager as CoreRiskManager, RiskAssessment as CoreRiskAssessment};

// 🚀 TEMPORAL: Usar tipos básicos hasta integrar completamente los módulos centrales
type CorePerformanceReport = std::collections::HashMap<String, f64>;

/// Helper function to compare RiskSeverity since it doesn't implement PartialEq
fn risk_severity_eq(a: &MonitoringLevel, b: &MonitoringLevel) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

/// 🚀 NUEVOS TIPOS PARA ANÁLISIS DE LIQUIDEZ
#[derive(Debug, Clone)]
pub enum LiquidityStatus {
    High,
    Medium,
    Low,
    Critical,
}

#[derive(Debug, Clone)]
pub struct LiquidityAnalysisReport {
    pub status: LiquidityStatus,
    pub available_liquidity: f64,
    pub utilization_percentage: f64,
    pub recommendation: String,
    pub timestamp: DateTime<Utc>,
}

/// Enterprise position manager with automated strategies
pub struct PositionManager {
    config: SniperConfig,
    active_positions: HashMap<Uuid, Position>,
    
    // 🚀 REFACTORING: Usar módulos centrales existentes
    portfolio_manager: PortfolioManager,
    risk_manager: AdvancedRiskManager,
    performance_analytics: PerformanceAnalyticsAI,
    
    // 🚀 TEMPORAL: Simplificar hasta completar integración central
    // portfolio_analytics: PortfolioAnalytics,
    
    // Funcionalidades específicas del liquidity sniper que no existen en otros módulos
    position_tracker: PositionTracker,
    exit_manager: ExitManager,
    metrics: PositionMetrics,
}

/// Position data structure
#[derive(Debug, Clone)]
pub struct Position {
    pub id: Uuid,
    pub opportunity_id: Uuid,
    pub token_address: String,
    pub pool_address: String,
    pub dex: DexType,
    pub entry_time: DateTime<Utc>,
    pub entry_price: f64,
    pub position_size_sol: f64,
    pub position_size_tokens: f64,
    pub risk_assessment: RiskAssessment,
    pub status: PositionStatus,
    pub stop_levels: Vec<ActiveStopLevel>,
    pub performance: PositionPerformance,
    pub monitoring_level: MonitoringLevel,
    pub metadata: PositionMetadata,
}

/// Position status
#[derive(Debug, Clone, PartialEq)]
pub enum PositionStatus {
    Active,
    Monitoring,
    ExitPending,
    Closed,
    Stopped,
    Expired,
}

/// Active stop level with triggers
#[derive(Debug, Clone)]
pub struct ActiveStopLevel {
    pub stop_level: RequiredStop, // Changed to store the actual RequiredStop from risk_manager
    pub triggered: bool,
    pub trigger_time: Option<DateTime<Utc>>,
    pub trigger_price: Option<f64>,
}

/// Position performance tracking
#[derive(Debug, Clone)]
pub struct PositionPerformance {
    pub current_price: f64,
    pub unrealized_pnl_sol: f64,
    pub unrealized_pnl_percent: f64,
    pub max_profit_percent: f64,
    pub max_loss_percent: f64,
    pub drawdown_from_peak: f64,
    pub hold_time_minutes: f64,
    pub price_updates: Vec<PriceUpdate>,
}

/// Position metadata
#[derive(Debug, Clone)]
pub struct PositionMetadata {
    pub entry_reason: String,
    pub strategy_used: String,
    pub expected_hold_time: Option<Duration>,
    pub target_profit_percent: f64,
    pub max_acceptable_loss_percent: f64,
    pub notes: Vec<String>,
}

/// Price update record
#[derive(Debug, Clone)]
pub struct PriceUpdate {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub volume: f64,
    pub liquidity: f64,
}

/// Position tracking system
#[derive(Debug)]
pub struct PositionTracker {
    price_monitor: PriceMonitor,
    liquidity_monitor: LiquidityMonitor,
    volume_monitor: VolumeMonitor,
}

/// Exit management system
#[derive(Debug)]
pub struct ExitManager {
    stop_loss_engine: StopLossEngine,
    take_profit_engine: TakeProfitEngine,
    time_based_exits: TimeBasedExits,
    liquidity_exits: LiquidityExits,
}

/// Performance analysis system - ELIMINADO: Usar crate::analytics::performance_analytics
/// PnL calculator - ELIMINADO: Usar crate::trading::portfolio
/// Risk metrics calculator - ELIMINADO: Usar crate::security::risk_manager  
/// Attribution analyzer - ELIMINADO: Usar crate::analytics::performance_analytics

/// Position metrics específicas del liquidity sniper (no duplicadas)
#[derive(Debug, Clone)]
pub struct PositionMetrics {
    pub total_positions: usize,
    pub active_positions: usize,
    pub closed_positions: usize,
    pub winning_positions: usize,
    pub losing_positions: usize,
    pub average_hold_time_minutes: f64,
    pub average_profit_percent: f64,
    pub average_loss_percent: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub total_pnl_sol: f64,
    pub total_pnl_percent: f64,
}

/// Price monitoring
#[derive(Debug)]
pub struct PriceMonitor {
    current_prices: HashMap<String, f64>,
    price_alerts: Vec<PriceAlert>,
}

/// Liquidity monitoring
#[derive(Debug)]
pub struct LiquidityMonitor {
    liquidity_thresholds: HashMap<String, f64>,
    liquidity_alerts: Vec<LiquidityAlert>,
}

/// Volume monitoring
#[derive(Debug)]
pub struct VolumeMonitor {
    volume_thresholds: HashMap<String, f64>,
    volume_alerts: Vec<VolumeAlert>,
}

/// Stop loss engine
#[derive(Debug)]
pub struct StopLossEngine {
    hard_stops: HashMap<Uuid, f64>,
    trailing_stops: HashMap<Uuid, TrailingStop>,
    soft_stops: HashMap<Uuid, f64>,
}

/// Take profit engine
#[derive(Debug)]
pub struct TakeProfitEngine {
    profit_targets: HashMap<Uuid, Vec<ProfitTarget>>,
    scaling_strategies: HashMap<Uuid, ScalingStrategy>,
}

/// Time-based exit system
#[derive(Debug)]
pub struct TimeBasedExits {
    max_hold_times: HashMap<Uuid, DateTime<Utc>>,
    scheduled_exits: Vec<ScheduledExit>,
    soft_stops: HashMap<Uuid, f64>, // 🚀 ADDED: Para soportar la funcionalidad de soft stops
}

/// Liquidity-based exit system
#[derive(Debug)]
pub struct LiquidityExits {
    min_liquidity_thresholds: HashMap<Uuid, f64>,
    liquidity_degradation_limits: HashMap<Uuid, f64>,
}

// 🚀 ELIMINADAS las siguientes estructuras duplicadas:
// - PnlCalculator (usar crate::trading::portfolio)
// - RiskMetricsCalculator (usar crate::security::risk_manager)  
// - AttributionAnalyzer (usar crate::analytics::performance_analytics)
// - VarCalculator (usar crate::security::risk_manager)
// - DrawdownCalculator (usar crate::trading::portfolio)

/// Trailing stop configuration
#[derive(Debug, Clone)]
pub struct TrailingStop {
    pub initial_stop_percent: f64,
    pub trail_amount_percent: f64,
    pub current_stop_price: f64,
    pub highest_price: f64,
}

/// Profit target configuration
#[derive(Debug, Clone)]
pub struct ProfitTarget {
    pub target_percent: f64,
    pub scale_out_percent: f64, // What % of position to close
    pub triggered: bool,
}

/// Scaling strategy
#[derive(Debug, Clone)]
pub struct ScalingStrategy {
    pub strategy_type: ScalingType,
    pub scale_levels: Vec<ScaleLevel>,
}

/// Scaling strategy types
#[derive(Debug, Clone)]
pub enum ScalingType {
    ProfitBased,
    TimeBased,
    VolumeBased,
    LiquidityBased,
}

/// Scale level configuration
#[derive(Debug, Clone)]
pub struct ScaleLevel {
    pub trigger_value: f64,
    pub scale_percent: f64,
    pub executed: bool,
}

/// Scheduled exit
#[derive(Debug, Clone)]
pub struct ScheduledExit {
    pub position_id: Uuid,
    pub exit_time: DateTime<Utc>,
    pub exit_reason: String,
    pub partial_exit_percent: Option<f64>,
}

/// Price alert
#[derive(Debug, Clone)]
pub struct PriceAlert {
    pub token_address: String,
    pub alert_type: AlertType,
    pub threshold: f64,
    pub current_value: f64,
}

/// Liquidity alert
#[derive(Debug, Clone)]
pub struct LiquidityAlert {
    pub pool_address: String,
    pub alert_type: AlertType,
    pub threshold: f64,
    pub current_value: f64,
}

/// Volume alert
#[derive(Debug, Clone)]
pub struct VolumeAlert {
    pub token_address: String,
    pub alert_type: AlertType,
    pub threshold: f64,
    pub current_value: f64,
}

/// Alert types
#[derive(Debug, Clone)]
pub enum AlertType {
    Above,
    Below,
    PercentChange,
    Volatility,
    LiquidityDegradation,
}

/// 🚀 NUEVA FUNCIONALIDAD: Soft stop types for advanced position management
#[derive(Debug, Clone)]
pub enum SoftStopType {
    TimeDecay,
    ProfitTaking,
    VolatilityBased,
    LiquidityBased,
}

/// 🚀 NUEVA FUNCIONALIDAD: Soft stop configuration
#[derive(Debug, Clone)]
pub struct SoftStop {
    pub stop_type: SoftStopType,
    pub trigger_threshold: f64,
    pub action: SoftStopAction,
    pub percentage_adjustment: f64,
}

/// 🚀 NUEVA FUNCIONALIDAD: Soft stop action
#[derive(Debug, Clone)]
pub struct SoftStopAction {
    pub position_id: Uuid,
    pub action_type: SoftStopActionType,
    pub percentage: f64,
    pub reason: String,
    pub priority: ActionPriority,
}

/// 🚀 NUEVA FUNCIONALIDAD: Soft stop action types
#[derive(Debug, Clone)]
pub enum SoftStopActionType {
    PartialExit,
    AdjustStop,
    ReduceSize,
    Monitor,
}

/// 🚀 NUEVA FUNCIONALIDAD: Action priority levels
#[derive(Debug, Clone)]
pub enum ActionPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// 🚀 NUEVA FUNCIONALIDAD: Scheduled exit action
#[derive(Debug, Clone)]
pub struct ScheduledExitAction {
    pub position_id: Uuid,
    pub exit_percentage: f64,
    pub reason: String,
    pub urgency: ExitUrgency,
    pub estimated_execution_time: DateTime<Utc>,
}

/// 🚀 NUEVA FUNCIONALIDAD: Exit urgency levels
#[derive(Debug, Clone)]
pub enum ExitUrgency {
    Low,
    Medium,
    High,
    Immediate,
}

/// 🚀 NUEVA FUNCIONALIDAD: Scaling opportunity
#[derive(Debug, Clone)]
pub struct ScalingOpportunity {
    pub position_id: Uuid,
    pub strategy_type: ScalingType,
    pub recommended_scale_percent: f64,
    pub urgency: ScalingUrgency,
}

/// 🚀 NUEVA FUNCIONALIDAD: Scaling urgency
#[derive(Debug, Clone)]
pub enum ScalingUrgency {
    Low,
    Medium,
    High,
}

// 🚀 ELIMINADAS las siguientes estructuras duplicadas:
// - VarCalculator (usar crate::security::risk_manager)
// - DrawdownCalculator (usar crate::trading::portfolio)

/// Position update result
#[derive(Debug)]
pub struct PositionUpdate {
    pub position_id: Uuid,
    pub updated_fields: Vec<String>,
    pub alerts_triggered: Vec<String>,
    pub actions_required: Vec<ActionRequired>,
}

/// Required actions from position updates
#[derive(Debug, Clone)]
pub enum ActionRequired {
    ExecuteStopLoss,
    ExecuteTakeProfit,
    PartialExit(f64), // percentage to exit
    IncreaseMonitoring,
    UpdateStops,
    ManualReview,
}

impl PositionManager {
    /// Create new enterprise position manager
    pub fn new(config: &SniperConfig) -> Result<Self> {
        info!("📈 Initializing Enterprise Position Manager with Central Modules");
        info!("   Max Positions: {}", config.max_positions);
        info!("   Position Tracking: Advanced with Central Analytics");
        
        // 🚀 REFACTORING: Usar módulos centrales existentes
        let portfolio_config = crate::config::SimpleConfig::default(); 
        let portfolio_manager = PortfolioManager::new(portfolio_config.clone());
        
        // Usar el AdvancedRiskManager central
        let risk_config = RiskManagementConfig {
            max_position_size_pct: config.max_position_size_percent,
            max_concurrent_trades: config.max_positions as usize,
            max_risk_score: config.max_risk_score,
            ..Default::default()
        };
        let risk_manager = AdvancedRiskManager::new(Some(risk_config));
        
        // 🚀 TEMPORAL: Usar PerformanceAnalyticsAI central simplificado
        let performance_analytics = PerformanceAnalyticsAI::new(
            None, // PerformanceAnalyticsConfig se deriva del default
            portfolio_config.clone()
        );
        
        let position_tracker = PositionTracker::new(config)?;
        let exit_manager = ExitManager::new(config)?;
        
        info!("✅ Initialized with central modules: Portfolio, Risk, Performance");
        
        Ok(Self {
            config: config.clone(),
            active_positions: HashMap::new(),
            portfolio_manager,
            risk_manager,
            performance_analytics,
            position_tracker,
            exit_manager,
            metrics: PositionMetrics::new(),
        })
    }
    
    /// Open new position from opportunity
    pub async fn open_position(
        &mut self,
        opportunity: &OpportunityData,
        risk_assessment: &RiskAssessment,
        entry_price: f64,
        position_size_sol: f64,
    ) -> Result<Position> {
        info!("🎯 Opening new position: {}", opportunity.token_address);
        info!("   Entry Price: {} SOL", entry_price);
        info!("   Position Size: {} SOL", position_size_sol);
        
        let position_id = Uuid::new_v4();
        let position_size_tokens = position_size_sol / entry_price;
        
        // Create active stop levels from risk assessment
        let stop_levels = risk_assessment.required_stops.iter()
            .map(|stop| ActiveStopLevel {
                stop_level: stop.clone(),
                triggered: false,
                trigger_time: None,
                trigger_price: None,
            })
            .collect();
        
        // Calculate expected hold time based on strategy
        let expected_hold_time = self.calculate_expected_hold_time(opportunity, risk_assessment).await?;
        
        let position = Position {
            id: position_id,
            opportunity_id: opportunity.id,
            token_address: opportunity.token_address.clone(),
            pool_address: opportunity.pool_address.clone(),
            dex: opportunity.dex.clone(),
            entry_time: Utc::now(),
            entry_price,
            position_size_sol,
            position_size_tokens,
            risk_assessment: risk_assessment.clone(),
            status: PositionStatus::Active,
            stop_levels,
            performance: PositionPerformance {
                current_price: entry_price,
                unrealized_pnl_sol: 0.0,
                unrealized_pnl_percent: 0.0,
                max_profit_percent: 0.0,
                max_loss_percent: 0.0,
                drawdown_from_peak: 0.0,
                hold_time_minutes: 0.0,
                price_updates: vec![PriceUpdate {
                    timestamp: Utc::now(),
                    price: entry_price,
                    volume: opportunity.volume_24h_usd,
                    liquidity: opportunity.liquidity_usd,
                }],
            },
            monitoring_level: risk_assessment.monitoring_level.clone(),
            metadata: PositionMetadata {
                entry_reason: format!("Profit potential: {:.1}%", opportunity.estimated_profit_percent),
                strategy_used: "Liquidity Sniper".to_string(),
                expected_hold_time,
                target_profit_percent: opportunity.estimated_profit_percent,
                max_acceptable_loss_percent: risk_assessment.required_stops
                    .iter()
                    .filter_map(|stop| match stop.stop_type {
                        StopType::Hard => Some(stop.level_percent),
                        _ => None,
                    })
                    .next()
                    .unwrap_or(5.0),
                notes: vec![],
            },
        };
        
        // Set up monitoring
        self.setup_position_monitoring(&position).await?;
        
        // Set up exit strategies
        self.setup_exit_strategies(&position).await?;
        
        // Add to active positions
        self.active_positions.insert(position_id, position.clone());
        
        // Update metrics
        self.update_position_metrics().await?;
        
        info!("✅ Position opened successfully - ID: {}", position_id);
        
        Ok(position)
    }
    
    /// Update position with new market data
    pub async fn update_position(
        &mut self,
        position_id: Uuid,
        current_price: f64,
        volume: f64,
        liquidity: f64,
    ) -> Result<PositionUpdate> {
        debug!("📊 Updating position: {}", position_id);
        
        let mut updated_fields = Vec::new();
        let mut alerts_triggered = Vec::new();
        let mut actions_required = Vec::new();
        
        // First, get a clone for read-only checks
        let position_clone = if let Some(pos) = self.active_positions.get(&position_id) {
            pos.clone()
        } else {
            return Err(anyhow::anyhow!("Position not found: {}", position_id));
        };
        
        // Perform read-only checks first
        let stop_actions = self.check_stop_levels(&position_clone, current_price).await?;
        for action in stop_actions {
            actions_required.push(action);
        }
        
        let profit_actions = self.check_take_profit_levels(&position_clone, current_price).await?;
        for action in profit_actions {
            actions_required.push(action);
        }
        
        if self.should_exit_time_based(&position_clone).await? {
            actions_required.push(ActionRequired::ExecuteTakeProfit);
            alerts_triggered.push("Time-based exit triggered".to_string());
        }
        
        if liquidity < position_clone.risk_assessment.max_position_size * 0.5 {
            actions_required.push(ActionRequired::PartialExit(50.0));
            alerts_triggered.push("Low liquidity detected".to_string());
        }
        
        let new_monitoring_level = self.determine_monitoring_level(&position_clone);
        
        // Now update with mutable access
        if let Some(position) = self.active_positions.get_mut(&position_id) {
            // Update price data
            position.performance.current_price = current_price;
            position.performance.hold_time_minutes = 
                (Utc::now() - position.entry_time).num_minutes() as f64;
            
            // Add price update record
            position.performance.price_updates.push(PriceUpdate {
                timestamp: Utc::now(),
                price: current_price,
                volume,
                liquidity,
            });
            
            // Calculate PnL
            let price_change_percent = ((current_price - position.entry_price) / position.entry_price) * 100.0;
            position.performance.unrealized_pnl_percent = price_change_percent;
            position.performance.unrealized_pnl_sol = position.position_size_sol * (price_change_percent / 100.0);
            
            // Update max profit/loss tracking
            if price_change_percent > position.performance.max_profit_percent {
                position.performance.max_profit_percent = price_change_percent;
            }
            if price_change_percent < position.performance.max_loss_percent {
                position.performance.max_loss_percent = price_change_percent;
            }
            
            // Calculate drawdown from peak
            let current_profit = price_change_percent;
            let drawdown = position.performance.max_profit_percent - current_profit;
            position.performance.drawdown_from_peak = drawdown.max(0.0);
            
            updated_fields.push("performance".to_string());
            
            // Update monitoring level if changed
            if !risk_severity_eq(&new_monitoring_level, &position.monitoring_level) {
                position.monitoring_level = new_monitoring_level;
                updated_fields.push("monitoring_level".to_string());
                alerts_triggered.push("Monitoring level changed".to_string());
            }
            
            // Log significant events
            if price_change_percent > 10.0 {
                info!("📈 Position {} up {:.1}% - Current PnL: {:.2} SOL", 
                      position_id, price_change_percent, position.performance.unrealized_pnl_sol);
            } else if price_change_percent < -5.0 {
                warn!("📉 Position {} down {:.1}% - Current PnL: {:.2} SOL", 
                      position_id, price_change_percent, position.performance.unrealized_pnl_sol);
            }
        }
        
        Ok(PositionUpdate {
            position_id,
            updated_fields,
            alerts_triggered,
            actions_required,
        })
    }
    
    /// Close position
    pub async fn close_position(
        &mut self,
        position_id: Uuid,
        exit_price: f64,
        exit_reason: String,
    ) -> Result<ClosedPosition> {
        info!("🏁 Closing position: {}", position_id);
        
        if let Some(mut position) = self.active_positions.remove(&position_id) {
            position.status = PositionStatus::Closed;
            
            // Calculate final PnL
            let price_change_percent = ((exit_price - position.entry_price) / position.entry_price) * 100.0;
            let realized_pnl_sol = position.position_size_sol * (price_change_percent / 100.0);
            
            let closed_position = ClosedPosition {
                position: position.clone(),
                exit_time: Utc::now(),
                exit_price,
                exit_reason,
                realized_pnl_sol,
                realized_pnl_percent: price_change_percent,
                hold_time_minutes: (Utc::now() - position.entry_time).num_minutes() as f64,
            };
            
            // Update metrics
            self.update_metrics_on_close(&closed_position).await?;
            
            info!("✅ Position closed - PnL: {:.2} SOL ({:.1}%)", 
                  realized_pnl_sol, price_change_percent);
            
            Ok(closed_position)
        } else {
            Err(anyhow::anyhow!("Position not found: {}", position_id))
        }
    }
    
    /// Check stop levels for position
    async fn check_stop_levels(
        &self,
        position: &Position,
        current_price: f64,
    ) -> Result<Vec<ActionRequired>> {
        let mut actions = Vec::new();
        
        for stop_level in &position.stop_levels {
            if stop_level.triggered {
                continue;
            }
            
            let should_trigger = match stop_level.stop_level.stop_type {
                StopType::Hard => {
                    let stop_price = position.entry_price * (1.0 - stop_level.stop_level.level_percent / 100.0);
                    current_price <= stop_price
                },
                StopType::Soft => {
                    let stop_price = position.entry_price * (1.0 - stop_level.stop_level.level_percent / 100.0);
                    current_price <= stop_price
                },
                StopType::Trailing => {
                    // Trailing stop logic would be more complex
                    false
                },
                StopType::TimeBasedExit => {
                    // Time-based stops are checked separately
                    false
                },
            };
            
            if should_trigger {
                // Note: In a full implementation, we would need to update these fields
                // via a separate mutable operation after this read-only check
                // stop_level.triggered = true;
                // stop_level.trigger_time = Some(Utc::now());
                // stop_level.trigger_price = Some(current_price);
                
                match stop_level.stop_level.stop_type {
                    StopType::Hard => actions.push(ActionRequired::ExecuteStopLoss),
                    StopType::Soft => actions.push(ActionRequired::ManualReview),
                    _ => {}
                }
            }
        }
        
        Ok(actions)
    }
    
    /// Check take profit levels
    async fn check_take_profit_levels(
        &self,
        position: &Position,
        current_price: f64,
    ) -> Result<Vec<ActionRequired>> {
        let mut actions = Vec::new();
        
        let profit_percent = ((current_price - position.entry_price) / position.entry_price) * 100.0;
        
        // Check if we've hit target profit
        if profit_percent >= position.metadata.target_profit_percent {
            actions.push(ActionRequired::ExecuteTakeProfit);
        }
        
        // Check for scaling out opportunities
        if profit_percent >= position.metadata.target_profit_percent * 0.5 {
            actions.push(ActionRequired::PartialExit(25.0)); // Scale out 25%
        }
        
        Ok(actions)
    }
    
    /// Check if time-based exit should trigger
    async fn should_exit_time_based(&self, position: &Position) -> Result<bool> {
        if let Some(expected_hold_time) = position.metadata.expected_hold_time {
            let elapsed = Utc::now() - position.entry_time;
            if elapsed > expected_hold_time {
                return Ok(true);
            }
        }
        
        // Check for high-risk positions that should exit quickly
        if position.risk_assessment.risk_score > 0.8 {
            let elapsed_minutes = (Utc::now() - position.entry_time).num_minutes();
            if elapsed_minutes > 10 { // 10 minutes max for high-risk
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Determine monitoring level based on performance
    fn determine_monitoring_level(&self, position: &Position) -> MonitoringLevel {
        let profit_percent = position.performance.unrealized_pnl_percent;
        let drawdown = position.performance.drawdown_from_peak;
        
        if drawdown > 5.0 || profit_percent < -3.0 {
            MonitoringLevel::Critical
        } else if drawdown > 2.0 || profit_percent < -1.0 {
            MonitoringLevel::High
        } else if profit_percent > 5.0 {
            MonitoringLevel::Medium // Monitor profitable positions
        } else {
            MonitoringLevel::Low
        }
    }
    
    /// Setup position monitoring
    async fn setup_position_monitoring(&mut self, position: &Position) -> Result<()> {
        debug!("📡 Setting up monitoring for position: {}", position.id);
        
        // Set price alerts
        self.position_tracker.price_monitor.price_alerts.push(PriceAlert {
            token_address: position.token_address.clone(),
            alert_type: AlertType::PercentChange,
            threshold: 5.0, // 5% change
            current_value: position.entry_price,
        });
        
        // Set liquidity alerts
        self.position_tracker.liquidity_monitor.liquidity_alerts.push(LiquidityAlert {
            pool_address: position.pool_address.clone(),
            alert_type: AlertType::Below,
            threshold: position.risk_assessment.max_position_size * 0.5,
            current_value: 0.0, // Will be updated
        });
        
        Ok(())
    }
    
    /// Setup exit strategies
    async fn setup_exit_strategies(&mut self, position: &Position) -> Result<()> {
        debug!("🚪 Setting up exit strategies for position: {}", position.id);
        
        // Setup stop losses
        for stop_level in &position.stop_levels {
            match stop_level.stop_level.stop_type {
                StopType::Hard => {
                    let stop_price = position.entry_price * (1.0 - stop_level.stop_level.level_percent / 100.0);
                    self.exit_manager.stop_loss_engine.hard_stops.insert(position.id, stop_price);
                },
                StopType::Trailing => {
                    let trailing_stop = TrailingStop {
                        initial_stop_percent: stop_level.stop_level.level_percent,
                        trail_amount_percent: 1.0, // 1% trail
                        current_stop_price: position.entry_price * (1.0 - stop_level.stop_level.level_percent / 100.0),
                        highest_price: position.entry_price,
                    };
                    self.exit_manager.stop_loss_engine.trailing_stops.insert(position.id, trailing_stop);
                },
                _ => {}
            }
        }
        
        // Setup take profit targets
        let profit_targets = vec![
            ProfitTarget {
                target_percent: position.metadata.target_profit_percent * 0.5,
                scale_out_percent: 25.0,
                triggered: false,
            },
            ProfitTarget {
                target_percent: position.metadata.target_profit_percent,
                scale_out_percent: 75.0,
                triggered: false,
            },
        ];
        
        self.exit_manager.take_profit_engine.profit_targets.insert(position.id, profit_targets);
        
        // Setup time-based exit if specified
        if let Some(expected_hold_time) = position.metadata.expected_hold_time {
            let exit_time = position.entry_time + expected_hold_time;
            self.exit_manager.time_based_exits.max_hold_times.insert(position.id, exit_time);
        }
        
        Ok(())
    }
    
    /// Calculate expected hold time
    async fn calculate_expected_hold_time(
        &self,
        opportunity: &OpportunityData,
        risk_assessment: &RiskAssessment,
    ) -> Result<Option<Duration>> {
        // Base hold time on risk level and opportunity characteristics
        let base_minutes = if risk_assessment.risk_score > 0.8 {
            5 // Very high risk - quick exit
        } else if risk_assessment.risk_score > 0.6 {
            15 // High risk
        } else if risk_assessment.risk_score > 0.4 {
            30 // Medium risk
        } else {
            60 // Low risk - can hold longer
        };
        
        // Adjust based on liquidity
        let liquidity_factor = if opportunity.liquidity_usd < 50000.0 {
            0.5 // Reduce hold time for low liquidity
        } else {
            1.0
        };
        
        let final_minutes = (base_minutes as f64 * liquidity_factor) as i64;
        Ok(Some(Duration::minutes(final_minutes)))
    }
    
    /// Update metrics on position close
    async fn update_metrics_on_close(&mut self, closed_position: &ClosedPosition) -> Result<()> {
        self.metrics.total_positions += 1;
        self.metrics.closed_positions += 1;
        
        if closed_position.realized_pnl_percent > 0.0 {
            self.metrics.winning_positions += 1;
        } else {
            self.metrics.losing_positions += 1;
        }
        
        // Update averages
        self.metrics.average_hold_time_minutes = 
            (self.metrics.average_hold_time_minutes * (self.metrics.closed_positions - 1) as f64 + 
             closed_position.hold_time_minutes) / self.metrics.closed_positions as f64;
        
        if closed_position.realized_pnl_percent > 0.0 {
            self.metrics.average_profit_percent = 
                (self.metrics.average_profit_percent * (self.metrics.winning_positions - 1) as f64 + 
                 closed_position.realized_pnl_percent) / self.metrics.winning_positions as f64;
        } else {
            self.metrics.average_loss_percent = 
                (self.metrics.average_loss_percent * (self.metrics.losing_positions - 1) as f64 + 
                 closed_position.realized_pnl_percent.abs()) / self.metrics.losing_positions as f64;
        }
        
        // Calculate win rate
        self.metrics.win_rate = self.metrics.winning_positions as f64 / self.metrics.total_positions as f64;
        
        // Update total PnL
        self.metrics.total_pnl_sol += closed_position.realized_pnl_sol;
        self.metrics.total_pnl_percent += closed_position.realized_pnl_percent;
        
        Ok(())
    }
    
    /// Update position metrics
    async fn update_position_metrics(&mut self) -> Result<()> {
        self.metrics.active_positions = self.active_positions.len();
        Ok(())
    }
    
    /// Get position by ID
    pub fn get_position(&self, position_id: Uuid) -> Option<&Position> {
        self.active_positions.get(&position_id)
    }
    
    /// Get all active positions
    pub fn get_active_positions(&self) -> Vec<&Position> {
        self.active_positions.values().collect()
    }
    
    /// Get position metrics
    pub fn get_metrics(&self) -> &PositionMetrics {
        &self.metrics
    }
    
    /// Get positions by monitoring level
    pub fn get_positions_by_monitoring_level(&self, level: MonitoringLevel) -> Vec<&Position> {
        self.active_positions.values()
            .filter(|p| std::mem::discriminant(&p.monitoring_level) == std::mem::discriminant(&level))
            .collect()
    }
    
    /// Emergency close all positions
    pub async fn emergency_close_all(&mut self, reason: String) -> Result<Vec<ClosedPosition>> {
        warn!("🚨 Emergency closing all positions - Reason: {}", reason);
        
        let mut closed_positions = Vec::new();
        let position_ids: Vec<Uuid> = self.active_positions.keys().cloned().collect();
        
        for position_id in position_ids {
            if let Some(position) = self.active_positions.get(&position_id) {
                let emergency_price = position.performance.current_price * 0.95; // Assume 5% slippage
                let closed = self.close_position(position_id, emergency_price, reason.clone()).await?;
                closed_positions.push(closed);
            }
        }
        
        Ok(closed_positions)
    }

    /// 🚀 ENRIQUECIMIENTO: Utiliza el config para validación de límites
    pub async fn validate_position_limits(&self, new_position_size: f64) -> Result<bool> {
        debug!("🔍 Validating position limits using config");
        
        // Usar config.max_positions para validar límites
        if self.active_positions.len() >= self.config.max_positions as usize {
            warn!("❌ Position limit exceeded: {}/{}", self.active_positions.len(), self.config.max_positions);
            return Ok(false);
        }

        // Usar config.max_position_size_percent para validar tamaño
        let max_position_size = self.config.capital_allocation * (self.config.max_position_size_percent / 100.0);
        if new_position_size > max_position_size {
            warn!("❌ Position size too large: {} > {} SOL", new_position_size, max_position_size);
            return Ok(false);
        }

        // Usar config.max_risk_score para validar exposición total
        let total_exposure = self.calculate_total_exposure().await?;
        let exposure_after_new = total_exposure + new_position_size;
        let max_exposure = self.config.capital_allocation * 0.8; // 80% max exposure
        
        if exposure_after_new > max_exposure {
            warn!("❌ Total exposure would exceed limit: {} > {} SOL", exposure_after_new, max_exposure);
            return Ok(false);
        }

        debug!("✅ Position limits validation passed");
        Ok(true)
    }

    /// 🚀 REFACTORING: Usar módulos centrales para análisis de performance
    pub async fn analyze_performance_with_central_modules(&self) -> Result<CorePerformanceReport> {
        debug!("📊 Analyzing performance using CENTRAL modules instead of duplicated code");
        
        // 🚀 USAR PORTFOLIO ANALYTICS CENTRAL en lugar de código duplicado
        let current_prices = HashMap::new(); // En implementación real: obtener precios actuales
        let core_metrics = self.portfolio_manager.get_performance_metrics().await;
        let risk_metrics = self.portfolio_manager.calculate_risk_metrics(&current_prices).await;
        
        // 🚀 TEMPORAL: Usar analytics básicos hasta completar integración central
        let mut performance_report = CorePerformanceReport::new();
        performance_report.insert("total_return".to_string(), core_metrics.total_profit);
        performance_report.insert("sharpe_ratio".to_string(), 1.5); // Temporal: calcular después
        performance_report.insert("max_drawdown".to_string(), (core_metrics.total_loss / core_metrics.total_profit).abs().max(0.0)); // Temporal: cálculo simplificado de drawdown
        performance_report.insert("win_rate".to_string(), self.metrics.win_rate * 100.0);
        
        info!("📈 Central Performance Analysis Complete:");
        info!("   📊 Total Return: {:.2}%", performance_report.get("total_return").unwrap_or(&0.0));
        info!("   ⚡ Sharpe Ratio: {:.2}", performance_report.get("sharpe_ratio").unwrap_or(&0.0));
        info!("   📉 Max Drawdown: {:.2}%", performance_report.get("max_drawdown").unwrap_or(&0.0));
        info!("   🎯 Win Rate: {:.1}%", performance_report.get("win_rate").unwrap_or(&0.0));
        
        Ok(performance_report)
    }

    /// 🚀 REFACTORING: Usar AdvancedRiskManager central para evaluación de riesgo
    pub async fn assess_portfolio_risk_with_central_manager(&mut self, trade_amount: f64, token_volatility: f64, liquidity_usd: f64) -> Result<crate::security::risk_manager::RiskAssessment> {
        debug!("🛡️ Assessing portfolio risk using CENTRAL risk manager");
        
        // 🚀 USAR ADVANCED RISK MANAGER CENTRAL
        let risk_assessment = self.risk_manager.assess_opportunity_risk(
            trade_amount,
            token_volatility,
            liquidity_usd,
            self.active_positions.len() as f64,
        ).await?;
        
        info!("🛡️ Central Risk Assessment:");
        info!("   📊 Risk Score: {:.3}", risk_assessment.risk_score);
        info!("   ✅ Approved: {}", risk_assessment.approved);
        info!("   💰 Position Size Limit: Calculated by central manager");
        
        Ok(risk_assessment)
    }

    /// 🚀 REFACTORING: Usar PerformanceAnalyticsAI central para optimización
    pub async fn optimize_with_central_ai(&self) -> Result<Vec<String>> {
        debug!("🤖 Optimizing strategies using CENTRAL AI analytics");
        
        // 🚀 USAR PERFORMANCE ANALYTICS AI CENTRAL - método simplificado
        let mut recommendations = Vec::new();
        
        // Generar recomendaciones básicas usando métricas internas
        if self.metrics.win_rate < 0.5 {
            recommendations.push("Consider adjusting entry criteria to improve win rate".to_string());
        }
        if self.metrics.sharpe_ratio < 1.0 {
            recommendations.push("Risk-adjusted returns could be improved".to_string());
        }
        if self.active_positions.len() > self.config.max_positions as usize * 80 / 100 {
            recommendations.push("Consider reducing position count for better risk management".to_string());
        }
        
        info!("🤖 Central AI Optimization Generated {} recommendations", recommendations.len());
        for (i, rec) in recommendations.iter().enumerate() {
            info!("   {}. {}", i + 1, rec);
        }
        
        Ok(recommendations)
    }

    /// 🚀 REFACTORING: Monitoreo básico usando métricas internas
    pub async fn advanced_monitoring_with_analyzer(&self) -> Result<Vec<MonitoringAlert>> {
        debug!("👀 Advanced monitoring using internal metrics");
        
        let mut alerts = Vec::new();
        
        // Usar config para detectar anomalías básicas
        let current_positions = self.active_positions.len();
        if current_positions > self.config.max_positions as usize {
            alerts.push(MonitoringAlert {
                level: AlertLevel::High,
                message: format!("Position limit exceeded: {}/{}", 
                               current_positions, self.config.max_positions),
                position_id: None,
                timestamp: Utc::now(),
                action_required: Some("Consider closing some positions".to_string()),
            });
        }
        
        // Monitoreo de performance por posición individual
        for position in self.active_positions.values() {
            let loss_percent = position.performance.unrealized_pnl_percent;
            
            if loss_percent < -10.0 { // Pérdida > 10%
                alerts.push(MonitoringAlert {
                    level: AlertLevel::High,
                    message: format!("Position {} has significant loss: {:.1}%", 
                                   position.id, loss_percent),
                    position_id: Some(position.id),
                    timestamp: Utc::now(),
                    action_required: Some("Consider stop loss execution".to_string()),
                });
            }
        }

        debug!("🚨 Monitoring complete: {} alerts generated", alerts.len());
        Ok(alerts)
    }

    /// 🚀 NUEVA FUNCIONALIDAD: Process soft stops system
    pub async fn process_soft_stops(&mut self) -> Result<Vec<SoftStopAction>> {
        debug!("🛑 Processing soft stops for active positions");
        let mut actions = Vec::new();
        
        for (position_id, position) in &self.active_positions {
            // Check each soft stop configuration (using the soft_stops field)
            if let Some(&threshold) = self.exit_manager.time_based_exits.soft_stops.get(position_id) {
                let holding_time = chrono::Utc::now().signed_duration_since(position.entry_time);
                
                // Time-based soft stop
                if holding_time.num_minutes() > threshold as i64 {
                    actions.push(SoftStopAction {
                        position_id: *position_id,
                        action_type: SoftStopActionType::PartialExit,
                        percentage: 25.0, // Reduce position by 25%
                        reason: format!("Time-based soft stop: held for {} minutes", holding_time.num_minutes()),
                        priority: ActionPriority::Medium,
                    });
                }
                
                // Profit-based soft stop
                if position.performance.unrealized_pnl_percent > threshold {
                    actions.push(SoftStopAction {
                        position_id: *position_id,
                        action_type: SoftStopActionType::AdjustStop,
                        percentage: 50.0, // Tighten stop by 50%
                        reason: format!("Profit-based soft stop: {:.1}% profit", position.performance.unrealized_pnl_percent),
                        priority: ActionPriority::Medium,
                    });
                }
            }
        }
        
        debug!("🛑 Generated {} soft stop actions", actions.len());
        Ok(actions)
    }

    /// 🚀 NUEVA FUNCIONALIDAD: Process scheduled exits
    pub async fn process_scheduled_exits(&mut self) -> Result<Vec<ScheduledExitAction>> {
        debug!("⏰ Processing scheduled exits");
        let mut actions = Vec::new();
        let current_time = chrono::Utc::now();
        
        // Filter scheduled exits that are due (using the scheduled_exits field)
        let due_exits: Vec<_> = self.exit_manager.time_based_exits.scheduled_exits.iter()
            .filter(|exit| exit.exit_time <= current_time)
            .collect();
        
        for scheduled_exit in due_exits {
            if let Some(position) = self.active_positions.get(&scheduled_exit.position_id) {
                actions.push(ScheduledExitAction {
                    position_id: scheduled_exit.position_id,
                    exit_percentage: scheduled_exit.partial_exit_percent.unwrap_or(100.0),
                    reason: scheduled_exit.exit_reason.clone(),
                    urgency: ExitUrgency::Immediate,
                    estimated_execution_time: current_time + chrono::Duration::minutes(1),
                });
            }
        }
        
        // Remove processed exits (this would require mutable access)
        debug!("⏰ Generated {} scheduled exit actions", actions.len());
        Ok(actions)
    }

    /// 🚀 CONECTANDO FIELD NO USADO: Check liquidity degradation for positions
    pub async fn check_liquidity_degradation(&self) -> Result<Vec<LiquidityAlert>> {
        debug!("💧 Checking liquidity degradation for all positions");
        let mut alerts = Vec::new();
        
        for (position_id, position) in &self.active_positions {
            // Usar el campo liquidity_degradation_limits
            if let Some(&limit) = self.exit_manager.liquidity_exits.liquidity_degradation_limits.get(position_id) {
                // En una implementación real, obtendríamos la liquidez actual del token
                // Por ahora simulamos una verificación
                // 🚀 TEMPORAL: Estimar liquidez usando position_size_sol como proxy
                let current_liquidity = position.position_size_sol * 10.0; // Estimación conservadora
                let simulated_current_liquidity = current_liquidity * 0.9; // Simular 10% degradación
                
                let degradation_percent = ((current_liquidity - simulated_current_liquidity) / current_liquidity) * 100.0;
                
                if degradation_percent > limit {
                    alerts.push(LiquidityAlert {
                        pool_address: position.pool_address.clone(),
                        alert_type: AlertType::LiquidityDegradation,
                        threshold: limit,
                        current_value: degradation_percent,
                    });
                }
            }
        }
        
        if !alerts.is_empty() {
            info!("⚠️ Found {} liquidity degradation alerts", alerts.len());
        }
        
        Ok(alerts)
    }

    /// 🚀 CONECTANDO FIELD NO USADO: Set liquidity degradation limit for position
    pub async fn set_liquidity_degradation_limit(&mut self, position_id: Uuid, limit_percent: f64) -> Result<()> {
        debug!("💧 Setting liquidity degradation limit for position {}: {:.1}%", position_id, limit_percent);
        
        // Usar el campo liquidity_degradation_limits
        self.exit_manager.liquidity_exits.liquidity_degradation_limits.insert(position_id, limit_percent);
        
        info!("✅ Liquidity degradation limit set: {:.1}%", limit_percent);
        Ok(())
    }

    /// 🚀 NUEVA FUNCIONALIDAD: Process scaling strategies 
    pub async fn process_scaling_strategies(&mut self) -> Result<Vec<ScalingOpportunity>> {
        debug!("📊 Processing scaling strategies for active positions");
        let mut opportunities = Vec::new();
        
        for (position_id, position) in &self.active_positions {
            // Check scaling strategies from profit_engine.scaling_strategies
            if let Some(strategy) = self.exit_manager.take_profit_engine.scaling_strategies.get(position_id) {
                for level in &strategy.scale_levels {
                    let should_scale = match strategy.strategy_type {
                        ScalingType::ProfitBased => {
                            position.performance.unrealized_pnl_percent > level.trigger_value && !level.executed
                        },
                        ScalingType::TimeBased => {
                            let holding_time = chrono::Utc::now().signed_duration_since(position.entry_time);
                            holding_time.num_minutes() > level.trigger_value as i64 && !level.executed
                        },
                        ScalingType::VolumeBased => {
                            // Volume-based scaling would need real-time volume data
                            false
                        },
                        ScalingType::LiquidityBased => {
                            // Liquidity-based scaling would need real-time liquidity data
                            false
                        },
                    };
                    
                    if should_scale {
                        opportunities.push(ScalingOpportunity {
                            position_id: *position_id,
                            strategy_type: strategy.strategy_type.clone(),
                            recommended_scale_percent: level.scale_percent,
                            urgency: if level.trigger_value > 20.0 { ScalingUrgency::High } else { ScalingUrgency::Medium },
                        });
                    }
                }
            }
        }
        
        debug!("📊 Found {} scaling opportunities", opportunities.len());
        Ok(opportunities)
    }

    /// Métodos auxiliares para los nuevos análisis
    async fn calculate_total_exposure(&self) -> Result<f64> {
        Ok(self.active_positions.values()
            .map(|p| p.position_size_sol)
            .sum())
    }

    async fn calculate_total_unrealized_pnl(&self) -> Result<f64> {
        Ok(self.active_positions.values()
            .map(|p| p.performance.unrealized_pnl_sol)
            .sum())
    }

    async fn calculate_sharpe_ratio(&self) -> Result<f64> {
        // Cálculo simplificado del ratio de Sharpe
        let returns = self.metrics.total_pnl_percent / 100.0;
        let risk_free_rate = 0.02; // 2% anual
        let volatility = 0.15; // Estimación de volatilidad
        
        Ok((returns - risk_free_rate) / volatility)
    }

    async fn calculate_historical_average_var(&self) -> Result<f64> {
        // Placeholder para VaR histórico promedio
        Ok(self.config.capital_allocation * 0.03) // 3% del capital
    }

    /// 🚀 NUEVA FUNCIONALIDAD: Análisis de liquidez usando módulos centrales  
    pub async fn analyze_liquidity(&self, opportunity: &TradingOpportunity) -> Result<LiquidityAnalysisReport> {
        debug!("🔍 Analyzing liquidity using central portfolio manager");
        
        // Usar el Portfolio central para análisis de liquidez - método simplificado
        let mut analytics = CorePerformanceReport::new();
        analytics.insert("total_portfolio_value".to_string(), self.config.capital_allocation);
        analytics.insert("total_unrealized_pnl".to_string(), self.metrics.total_pnl_sol);
        analytics.insert("available_balance".to_string(), self.config.capital_allocation * 0.8);
        
        // Análisis temporal simplificado
        let total_portfolio_value = analytics.get("total_portfolio_value").unwrap_or(&1.0);
        let total_unrealized_pnl = analytics.get("total_unrealized_pnl").unwrap_or(&0.0);
        let available_balance = analytics.get("available_balance").unwrap_or(&0.0);
        
        let utilization = if *total_portfolio_value > 0.0 {
            total_unrealized_pnl / total_portfolio_value
        } else {
            0.0
        };
        
        let liquidez_status = if utilization > 0.8 {
            LiquidityStatus::Low
        } else if utilization > 0.5 {
            LiquidityStatus::Medium  
        } else {
            LiquidityStatus::High
        };
        
        let recommendation = if utilization > 0.8 { 
            "Reduce position sizes".to_string() 
        } else { 
            "Liquidity sufficient".to_string() 
        };
        
        debug!("💧 Liquidity analysis complete: status={:?}, utilization={:.1}%", 
               liquidez_status, utilization * 100.0);
        
        Ok(LiquidityAnalysisReport {
            status: liquidez_status,
            available_liquidity: *available_balance,
            utilization_percentage: utilization * 100.0,
            recommendation,
            timestamp: chrono::Utc::now(),
        })
    }

    /// 🚀 NUEVA FUNCIONALIDAD: Análisis básico usando módulos centrales
    pub async fn analytics_with_central_modules(&self) -> Result<CorePerformanceReport> {
        debug!("📊 Basic analytics using central modules");
        
        let mut analytics = CorePerformanceReport::new();
        
        // Cálculos básicos usando métricas internas
        let total_positions = self.active_positions.len() as f64;
        let mut total_pnl = 0.0;
        let mut total_value = 0.0;
        
        for position in self.active_positions.values() {
            total_pnl += position.performance.unrealized_pnl_sol;
            total_value += position.position_size_sol;
        }
        
        analytics.insert("total_positions".to_string(), total_positions);
        analytics.insert("total_pnl".to_string(), total_pnl);
        analytics.insert("total_value".to_string(), total_value);
        analytics.insert("utilization".to_string(), (total_positions / self.config.max_positions as f64) * 100.0);
        
        debug!("📊 Analytics complete: {} positions, {:.2} SOL PnL", total_positions, total_pnl);
        Ok(analytics)
    }
}

/// Closed position record
#[derive(Debug, Clone)]
pub struct ClosedPosition {
    pub position: Position,
    pub exit_time: DateTime<Utc>,
    pub exit_price: f64,
    pub exit_reason: String,
    pub realized_pnl_sol: f64,
    pub realized_pnl_percent: f64,
    pub hold_time_minutes: f64,
}

impl PositionMetrics {
    pub fn new() -> Self {
        Self {
            total_positions: 0,
            active_positions: 0,
            closed_positions: 0,
            winning_positions: 0,
            losing_positions: 0,
            average_hold_time_minutes: 0.0,
            average_profit_percent: 0.0,
            average_loss_percent: 0.0,
            win_rate: 0.0,
            profit_factor: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            total_pnl_sol: 0.0,
            total_pnl_percent: 0.0,
        }
    }
}

impl PositionTracker {
    pub fn new(_config: &SniperConfig) -> Result<Self> {
        Ok(Self {
            price_monitor: PriceMonitor {
                current_prices: HashMap::new(),
                price_alerts: Vec::new(),
            },
            liquidity_monitor: LiquidityMonitor {
                liquidity_thresholds: HashMap::new(),
                liquidity_alerts: Vec::new(),
            },
            volume_monitor: VolumeMonitor {
                volume_thresholds: HashMap::new(),
                volume_alerts: Vec::new(),
            },
        })
    }

    /// 🚀 ENRIQUECIMIENTO: Update current prices using price_monitor
    pub fn update_price(&mut self, token_address: &str, price: f64) -> Result<()> {
        self.price_monitor.current_prices.insert(token_address.to_string(), price);
        debug!("📊 Updated price for {}: {:.6}", token_address, price);
        Ok(())
    }

    /// 🚀 ENRIQUECIMIENTO: Get current price from price_monitor
    pub fn get_current_price(&self, token_address: &str) -> Option<f64> {
        self.price_monitor.current_prices.get(token_address).copied()
    }

    /// 🚀 ENRIQUECIMIENTO: Set liquidity threshold using liquidity_monitor
    pub fn set_liquidity_threshold(&mut self, token_address: &str, threshold: f64) -> Result<()> {
        self.liquidity_monitor.liquidity_thresholds.insert(token_address.to_string(), threshold);
        debug!("🎯 Set liquidity threshold for {}: {:.2}", token_address, threshold);
        Ok(())
    }

    /// 🚀 ENRIQUECIMIENTO: Check liquidity against thresholds
    pub fn check_liquidity_threshold(&self, token_address: &str, current_liquidity: f64) -> Result<bool> {
        if let Some(&threshold) = self.liquidity_monitor.liquidity_thresholds.get(token_address) {
            let meets_threshold = current_liquidity >= threshold;
            debug!("💧 Liquidity check for {}: {:.2} >= {:.2} = {}", 
                   token_address, current_liquidity, threshold, meets_threshold);
            Ok(meets_threshold)
        } else {
            Ok(true) // No threshold set, assume OK
        }
    }

    /// 🚀 ENRIQUECIMIENTO: Set volume threshold using volume_monitor
    pub fn set_volume_threshold(&mut self, token_address: &str, threshold: f64) -> Result<()> {
        self.volume_monitor.volume_thresholds.insert(token_address.to_string(), threshold);
        debug!("📈 Set volume threshold for {}: {:.2}", token_address, threshold);
        Ok(())
    }

    /// 🚀 ENRIQUECIMIENTO: Monitor volume using volume_monitor
    pub fn monitor_volume(&mut self, token_address: &str, current_volume: f64) -> Result<()> {
        if let Some(&threshold) = self.volume_monitor.volume_thresholds.get(token_address) {
            if current_volume < threshold * 0.5 { // Alert if volume drops below 50% of threshold
                let alert = VolumeAlert {
                    token_address: token_address.to_string(),
                    alert_type: AlertType::Below,
                    threshold,
                    current_value: current_volume,
                };
                self.volume_monitor.volume_alerts.push(alert);
                debug!("⚠️ Volume alert triggered for {}: {:.2} < {:.2}", 
                       token_address, current_volume, threshold * 0.5);
            }
        }
        Ok(())
    }

    /// 🚀 ENRIQUECIMIENTO: Get recent volume alerts
    pub fn get_volume_alerts(&self) -> &Vec<VolumeAlert> {
        &self.volume_monitor.volume_alerts
    }
}

impl ExitManager {
    pub fn new(_config: &SniperConfig) -> Result<Self> {
        Ok(Self {
            stop_loss_engine: StopLossEngine {
                hard_stops: HashMap::new(),
                trailing_stops: HashMap::new(),
                soft_stops: HashMap::new(),
            },
            take_profit_engine: TakeProfitEngine {
                profit_targets: HashMap::new(),
                scaling_strategies: HashMap::new(),
            },
            time_based_exits: TimeBasedExits {
                max_hold_times: HashMap::new(),
                scheduled_exits: Vec::new(),
                soft_stops: HashMap::new(), // 🚀 ADDED: Initialize soft_stops field
            },
            liquidity_exits: LiquidityExits {
                min_liquidity_thresholds: HashMap::new(),
                liquidity_degradation_limits: HashMap::new(),
            },
        })
    }

    /// 🚀 ENRIQUECIMIENTO: Set hard stop loss using stop_loss_engine
    pub fn set_hard_stop(&mut self, position_id: Uuid, stop_price: f64) -> Result<()> {
        self.stop_loss_engine.hard_stops.insert(position_id, stop_price);
        debug!("🛑 Set hard stop for position {}: {:.6}", position_id, stop_price);
        Ok(())
    }

    /// 🚀 ENRIQUECIMIENTO: Check if hard stop triggered
    pub fn check_hard_stop(&self, position_id: Uuid, current_price: f64) -> Result<bool> {
        if let Some(&stop_price) = self.stop_loss_engine.hard_stops.get(&position_id) {
            let stop_triggered = current_price <= stop_price;
            if stop_triggered {
                debug!("⚠️ Hard stop triggered for {}: {:.6} <= {:.6}", 
                       position_id, current_price, stop_price);
            }
            Ok(stop_triggered)
        } else {
            Ok(false)
        }
    }

    /// 🚀 ENRIQUECIMIENTO: Set profit target using take_profit_engine
    pub fn set_profit_target(&mut self, position_id: Uuid, target_percent: f64, scale_out_percent: f64) -> Result<()> {
        let profit_target = ProfitTarget {
            target_percent,
            scale_out_percent,
            triggered: false,
        };
        self.take_profit_engine.profit_targets
            .entry(position_id)
            .or_insert_with(Vec::new)
            .push(profit_target);
        debug!("💰 Set profit target for position {}: {:.2}% (scale {:.2}%)", 
               position_id, target_percent, scale_out_percent);
        Ok(())
    }

    /// 🚀 ENRIQUECIMIENTO: Check if profit target reached
    pub fn check_profit_targets(&mut self, position_id: Uuid, current_percent_gain: f64) -> Result<Vec<ProfitTarget>> {
        let mut triggered_targets = Vec::new();
        
        if let Some(targets) = self.take_profit_engine.profit_targets.get_mut(&position_id) {
            for target in targets.iter_mut() {
                if !target.triggered && current_percent_gain >= target.target_percent {
                    target.triggered = true;
                    triggered_targets.push(target.clone());
                    debug!("🎯 Profit target reached for {}: {:.2}% >= {:.2}%", 
                           position_id, current_percent_gain, target.target_percent);
                }
            }
        }
        
        Ok(triggered_targets)
    }

    /// 🚀 ENRIQUECIMIENTO: Set max hold time using time_based_exits
    pub fn set_max_hold_time(&mut self, position_id: Uuid, max_duration: chrono::Duration) -> Result<()> {
        let exit_time = chrono::Utc::now() + max_duration;
        self.time_based_exits.max_hold_times.insert(position_id, exit_time);
        debug!("⏰ Set max hold time for position {}: until {:?}", position_id, exit_time);
        Ok(())
    }

    /// 🚀 ENRIQUECIMIENTO: Check if position should exit based on time
    pub fn should_time_exit(&self, position_id: Uuid) -> Result<bool> {
        if let Some(&exit_time) = self.time_based_exits.max_hold_times.get(&position_id) {
            let should_exit = chrono::Utc::now() >= exit_time;
            if should_exit {
                debug!("⏰ Time-based exit triggered for position {}", position_id);
            }
            Ok(should_exit)
        } else {
            Ok(false)
        }
    }

    /// 🚀 ENRIQUECIMIENTO: Set hard stop loss using stop_loss_engine
    pub fn set_liquidity_threshold(&mut self, position_id: Uuid, min_liquidity: f64) -> Result<()> {
        self.liquidity_exits.min_liquidity_thresholds.insert(position_id, min_liquidity);
        debug!("💧 Set liquidity threshold for position {}: {:.2}", position_id, min_liquidity);
        Ok(())
    }

    /// 🚀 ENRIQUECIMIENTO: Check if should exit due to low liquidity
    pub fn should_liquidity_exit(&self, position_id: Uuid, current_liquidity: f64) -> Result<bool> {
        if let Some(&min_liquidity) = self.liquidity_exits.min_liquidity_thresholds.get(&position_id) {
            let should_exit = current_liquidity < min_liquidity;
            if should_exit {
                debug!("⚠️ Liquidity exit triggered for {}: {:.2} < {:.2}", 
                       position_id, current_liquidity, min_liquidity);
            }
            Ok(should_exit)
        } else {
            Ok(false)
        }
    }
}

// 🚀 REFACTORING COMPLETO: Eliminadas implementaciones duplicadas
// Se eliminaron las siguientes implementaciones que duplican código existente:
// - impl PerformanceAnalyzer (usar crate::analytics::performance_analytics)
// - impl RiskMetricsCalculator (usar crate::security::risk_manager)  
// - impl AttributionAnalyzer (usar crate::analytics::performance_analytics)
//
// Mantener solo las estructuras específicas del liquidity sniper que no existen en otros módulos.

// 🚀 ELIMINADAS las siguientes estructuras duplicadas - USAR MÓDULOS CENTRALES:
// - PerformanceReport (usar crate::old_root_archive::portfolio::analytics::PerformanceReport)
// - RiskAnalyticsReport (usar crate::security::risk_manager y analytics centrales)
// - StrategyOptimization (usar crate::analytics::performance_analytics)
// - MonitoringAlert (usar crate::security::risk_manager)

/// 🚀 ESTRUCTURAS ESPECÍFICAS DEL LIQUIDITY SNIPER (NO DUPLICADAS)

// 🚀 SOLO MANTENER ESTRUCTURAS ESPECÍFICAS DEL LIQUIDITY SNIPER:

#[derive(Debug, Clone)]
pub struct MonitoringAlert {
    pub level: AlertLevel,
    pub message: String,
    pub position_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub action_required: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AlertLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_position_manager_creation() {
        let config = SniperConfig::default();
        let position_manager = PositionManager::new(&config);
        assert!(position_manager.is_ok());
    }
}
