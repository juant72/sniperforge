// SniperForge Enterprise v3.0 - Position Manager
// Advanced position management with automated strategies

use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use super::{OpportunityData, SniperConfig, RiskAssessment, StopLevel, StopType, MonitoringLevel};

/// Enterprise position manager with automated strategies
#[derive(Debug)]
pub struct PositionManager {
    config: SniperConfig,
    active_positions: HashMap<Uuid, Position>,
    position_tracker: PositionTracker,
    exit_manager: ExitManager,
    performance_analyzer: PerformanceAnalyzer,
    metrics: PositionMetrics,
}

/// Position data structure
#[derive(Debug, Clone)]
pub struct Position {
    pub id: Uuid,
    pub opportunity_id: Uuid,
    pub token_address: String,
    pub pool_address: String,
    pub dex: super::super::DexType,
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
    pub stop_level: StopLevel,
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

/// Performance analysis system
#[derive(Debug)]
pub struct PerformanceAnalyzer {
    pnl_calculator: PnlCalculator,
    risk_metrics: RiskMetricsCalculator,
    attribution_analyzer: AttributionAnalyzer,
}

/// Position metrics
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
}

/// Liquidity-based exit system
#[derive(Debug)]
pub struct LiquidityExits {
    min_liquidity_thresholds: HashMap<Uuid, f64>,
    liquidity_degradation_limits: HashMap<Uuid, f64>,
}

/// PnL calculator
#[derive(Debug)]
pub struct PnlCalculator {
    realized_pnl: f64,
    unrealized_pnl: f64,
}

/// Risk metrics calculator
#[derive(Debug)]
pub struct RiskMetricsCalculator {
    var_calculator: VarCalculator,
    drawdown_calculator: DrawdownCalculator,
}

/// Attribution analyzer
#[derive(Debug)]
pub struct AttributionAnalyzer {
    factor_contributions: HashMap<String, f64>,
    strategy_performance: HashMap<String, f64>,
}

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
}

/// VaR calculator for positions
#[derive(Debug)]
pub struct VarCalculator {
    confidence_level: f64,
    lookback_periods: usize,
}

/// Drawdown calculator
#[derive(Debug)]
pub struct DrawdownCalculator {
    peak_values: HashMap<Uuid, f64>,
    current_drawdowns: HashMap<Uuid, f64>,
}

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
        info!("📈 Initializing Enterprise Position Manager");
        info!("   Max Positions: {}", config.max_positions);
        info!("   Position Tracking: Advanced");
        
        let position_tracker = PositionTracker::new(config)?;
        let exit_manager = ExitManager::new(config)?;
        let performance_analyzer = PerformanceAnalyzer::new(config)?;
        
        Ok(Self {
            config: config.clone(),
            active_positions: HashMap::new(),
            position_tracker,
            exit_manager,
            performance_analyzer,
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
            
            // Check stop levels
            let stop_actions = self.check_stop_levels(position, current_price).await?;
            for action in stop_actions {
                actions_required.push(action);
            }
            
            // Check take profit levels
            let profit_actions = self.check_take_profit_levels(position, current_price).await?;
            for action in profit_actions {
                actions_required.push(action);
            }
            
            // Check time-based exits
            if self.should_exit_time_based(position).await? {
                actions_required.push(ActionRequired::ExecuteTakeProfit);
                alerts_triggered.push("Time-based exit triggered".to_string());
            }
            
            // Check liquidity-based exits
            if liquidity < position.risk_assessment.max_position_size * 0.5 {
                actions_required.push(ActionRequired::PartialExit(50.0));
                alerts_triggered.push("Low liquidity detected".to_string());
            }
            
            // Update monitoring level based on performance
            let new_monitoring_level = self.determine_monitoring_level(position);
            if new_monitoring_level != position.monitoring_level {
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
        position: &mut Position,
        current_price: f64,
    ) -> Result<Vec<ActionRequired>> {
        let mut actions = Vec::new();
        
        for stop_level in &mut position.stop_levels {
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
                stop_level.triggered = true;
                stop_level.trigger_time = Some(Utc::now());
                stop_level.trigger_price = Some(current_price);
                
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
            },
            liquidity_exits: LiquidityExits {
                min_liquidity_thresholds: HashMap::new(),
                liquidity_degradation_limits: HashMap::new(),
            },
        })
    }
}

impl PerformanceAnalyzer {
    pub fn new(_config: &SniperConfig) -> Result<Self> {
        Ok(Self {
            pnl_calculator: PnlCalculator {
                realized_pnl: 0.0,
                unrealized_pnl: 0.0,
            },
            risk_metrics: RiskMetricsCalculator {
                var_calculator: VarCalculator {
                    confidence_level: 0.95,
                    lookback_periods: 100,
                },
                drawdown_calculator: DrawdownCalculator {
                    peak_values: HashMap::new(),
                    current_drawdowns: HashMap::new(),
                },
            },
            attribution_analyzer: AttributionAnalyzer {
                factor_contributions: HashMap::new(),
                strategy_performance: HashMap::new(),
            },
        })
    }
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
    
    #[tokio::test]
    async fn test_position_opening() {
        let config = SniperConfig::default();
        let mut position_manager = PositionManager::new(&config).unwrap();
        
        let opportunity = OpportunityData {
            id: Uuid::new_v4(),
            token_address: "test_token".to_string(),
            pool_address: "test_pool".to_string(),
            dex: super::super::DexType::Raydium,
            detected_at: Utc::now(),
            liquidity_usd: 100000.0,
            price_impact: 1.0,
            estimated_profit_percent: 15.0,
            risk_score: 0.3,
            confidence_score: 0.9,
            market_cap_usd: 1000000.0,
            volume_24h_usd: 50000.0,
            holder_count: 200,
            age_minutes: 15,
        };
        
        let risk_assessment = RiskAssessment {
            approved: true,
            risk_score: 0.3,
            confidence: 0.9,
            reason: "Good opportunity".to_string(),
            recommendations: vec![],
            max_position_size: 10.0,
            required_stops: vec![],
            monitoring_level: MonitoringLevel::Medium,
        };
        
        let position = position_manager.open_position(
            &opportunity,
            &risk_assessment,
            0.5, // entry price
            5.0, // position size
        ).await;
        
        assert!(position.is_ok());
        assert_eq!(position_manager.get_active_positions().len(), 1);
    }
}
