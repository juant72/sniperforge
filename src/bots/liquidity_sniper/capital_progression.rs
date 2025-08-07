// SniperForge Enterprise v3.0 - Capital Progression Manager
// Sistema de progresión automática basada en capital acumulado

use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use tracing::{info, debug};
use crate::trading::portfolio::PerformanceMetrics;
use std::collections::HashMap;

/// Gestor de progresión de capital
#[derive(Debug)]
pub struct CapitalProgressionManager {
    current_phase: TradingPhase,
    capital_history: Vec<CapitalSnapshot>,
    milestones: Vec<Milestone>,
    unlocked_features: Vec<String>,
    performance_metrics: ProgressionMetrics,
}

/// Fases de trading basadas en capital
#[derive(Debug, Clone, PartialEq)]
pub enum TradingPhase {
    Accumulation,      // 0.0 - 0.5 SOL
    BasicTrading,      // 0.5 - 1.0 SOL  
    IntermediateTrading, // 1.0 - 2.0 SOL
    AdvancedTrading,   // 2.0+ SOL
}

/// Snapshot de capital en un momento dado
#[derive(Debug, Clone)]
pub struct CapitalSnapshot {
    timestamp: DateTime<Utc>,
    total_capital_sol: f64,
    trading_capital_sol: f64,
    reserved_capital_sol: f64,
    profit_since_last: f64,
    trades_since_last: u32,
    phase: TradingPhase,
}

/// Milestone de progresión
#[derive(Debug, Clone)]
pub struct Milestone {
    pub target_capital_sol: f64,
    phase: TradingPhase,
    unlocked_features: Vec<String>,
    new_config: MilestoneConfig,
    description: String,
    achieved: bool,
    achieved_at: Option<DateTime<Utc>>,
}

/// Configuración desbloqueada en cada milestone
#[derive(Debug, Clone, Default)]
pub struct MilestoneConfig {
    max_positions: u32,
    position_size_percent: f64,
    max_risk_score: f64,
    min_profit_target: f64,
    available_strategies: Vec<String>,
    advanced_exits: bool,
    scaling_allowed: bool,
}

/// Métricas de progresión
#[derive(Debug, Clone)]
pub struct ProgressionMetrics {
    days_in_current_phase: u32,
    profit_this_phase_sol: f64,
    trades_this_phase: u32,
    win_rate_this_phase: f64,
    average_profit_per_trade: f64,
    projected_days_to_next_milestone: u32,
    capital_growth_rate_daily: f64,
}

impl CapitalProgressionManager {
    /// Crear nuevo gestor de progresión
    pub fn new(initial_capital_sol: f64) -> Result<Self> {
        info!("🎯 Inicializando Capital Progression Manager");
        info!("   Capital inicial: {} SOL", initial_capital_sol);
        
        let phase = Self::determine_phase(initial_capital_sol);
        let milestones = Self::create_milestones();
        
        info!("   Fase inicial: {:?}", phase);
        
        Ok(Self {
            current_phase: phase.clone(),
            capital_history: vec![CapitalSnapshot {
                timestamp: Utc::now(),
                total_capital_sol: initial_capital_sol,
                trading_capital_sol: initial_capital_sol * 0.8,
                reserved_capital_sol: initial_capital_sol * 0.2,
                profit_since_last: 0.0,
                trades_since_last: 0,
                phase: phase.clone(),
            }],
            milestones,
            unlocked_features: Self::get_initial_features(&phase),
            performance_metrics: ProgressionMetrics::new(),
        })
    }
    
    /// Actualizar capital y verificar progresión
    pub fn update_capital(&mut self, new_capital_sol: f64, profit_from_trade: f64) -> Result<ProgressionUpdate> {
        debug!("💰 Actualizando capital: {} SOL (profit: {})", new_capital_sol, profit_from_trade);
        
        let previous_phase = self.current_phase.clone();
        let new_phase = Self::determine_phase(new_capital_sol);
        
        // Crear snapshot
        let snapshot = CapitalSnapshot {
            timestamp: Utc::now(),
            total_capital_sol: new_capital_sol,
            trading_capital_sol: new_capital_sol * 0.8,
            reserved_capital_sol: new_capital_sol * 0.2,
            profit_since_last: profit_from_trade,
            trades_since_last: 1,
            phase: new_phase.clone(),
        };
        
        self.capital_history.push(snapshot);
        
        // Verificar milestones
        let mut achieved_milestones = Vec::new();
        for milestone in &mut self.milestones {
            if !milestone.achieved && new_capital_sol >= milestone.target_capital_sol {
                milestone.achieved = true;
                milestone.achieved_at = Some(Utc::now());
                achieved_milestones.push(milestone.clone());
                
                // Desbloquear features
                for feature in &milestone.unlocked_features {
                    if !self.unlocked_features.contains(feature) {
                        self.unlocked_features.push(feature.clone());
                    }
                }
                
                info!("🏆 MILESTONE ACHIEVED: {} SOL - {}", 
                      milestone.target_capital_sol, milestone.description);
            }
        }
        
        // Actualizar fase si cambió
        let phase_changed = previous_phase != new_phase;
        if phase_changed {
            self.current_phase = new_phase.clone();
            info!("📈 PHASE UPGRADE: {:?} → {:?}", previous_phase, new_phase);
        }
        
        // Actualizar métricas
        self.update_metrics()?;
        
        Ok(ProgressionUpdate {
            previous_capital: self.get_previous_capital(),
            new_capital: new_capital_sol,
            previous_phase,
            new_phase,
            phase_changed,
            achieved_milestones: achieved_milestones.clone(),
            newly_unlocked_features: self.get_newly_unlocked_features(&achieved_milestones),
            updated_config: self.get_current_config(),
            progress_metrics: self.performance_metrics.clone(),
        })
    }
    
    /// Determinar fase basada en capital
    fn determine_phase(capital_sol: f64) -> TradingPhase {
        match capital_sol {
            x if x < 0.5 => TradingPhase::Accumulation,
            x if x < 1.0 => TradingPhase::BasicTrading,
            x if x < 2.0 => TradingPhase::IntermediateTrading,
            _ => TradingPhase::AdvancedTrading,
        }
    }
    
    /// Crear milestones de progresión
    fn create_milestones() -> Vec<Milestone> {
        vec![
            Milestone {
                target_capital_sol: 0.5,
                phase: TradingPhase::BasicTrading,
                unlocked_features: vec![
                    "trailing_stops".to_string(),
                    "position_scaling".to_string(),
                ],
                new_config: MilestoneConfig {
                    max_positions: 1,
                    position_size_percent: 60.0,
                    max_risk_score: 0.65,
                    min_profit_target: 20.0,
                    available_strategies: vec!["quick_flip".to_string()],
                    advanced_exits: true,
                    scaling_allowed: false,
                },
                description: "Basic features unlocked".to_string(),
                achieved: false,
                achieved_at: None,
            },
            Milestone {
                target_capital_sol: 1.0,
                phase: TradingPhase::IntermediateTrading,
                unlocked_features: vec![
                    "multiple_positions".to_string(),
                    "partial_exits".to_string(),
                    "advanced_risk_management".to_string(),
                ],
                new_config: MilestoneConfig {
                    max_positions: 2,
                    position_size_percent: 40.0,
                    max_risk_score: 0.7,
                    min_profit_target: 15.0,
                    available_strategies: vec![
                        "quick_flip".to_string(),
                        "trend_riding".to_string(),
                    ],
                    advanced_exits: true,
                    scaling_allowed: true,
                },
                description: "Multi-position trading enabled".to_string(),
                achieved: false,
                achieved_at: None,
            },
            Milestone {
                target_capital_sol: 2.0,
                phase: TradingPhase::AdvancedTrading,
                unlocked_features: vec![
                    "full_strategy_suite".to_string(),
                    "leverage_trading".to_string(),
                    "complex_exits".to_string(),
                    "correlation_analysis".to_string(),
                ],
                new_config: MilestoneConfig {
                    max_positions: 3,
                    position_size_percent: 30.0,
                    max_risk_score: 0.75,
                    min_profit_target: 12.0,
                    available_strategies: vec![
                        "quick_flip".to_string(),
                        "trend_riding".to_string(),
                        "mean_reversion".to_string(),
                        "arbitrage_snipe".to_string(),
                    ],
                    advanced_exits: true,
                    scaling_allowed: true,
                },
                description: "Full advanced trading unlocked".to_string(),
                achieved: false,
                achieved_at: None,
            },
        ]
    }
    
    /// Obtener features iniciales por fase
    fn get_initial_features(phase: &TradingPhase) -> Vec<String> {
        match phase {
            TradingPhase::Accumulation => vec![
                "basic_trading".to_string(),
                "single_position".to_string(),
                "hard_stops".to_string(),
            ],
            _ => vec![], // Features se desbloquean al alcanzar milestones
        }
    }
    
    /// Obtener capital anterior
    fn get_previous_capital(&self) -> f64 {
        if self.capital_history.len() > 1 {
            self.capital_history[self.capital_history.len() - 2].total_capital_sol
        } else {
            0.0
        }
    }
    
    /// Obtener features recién desbloqueadas
    fn get_newly_unlocked_features(&self, achieved_milestones: &[Milestone]) -> Vec<String> {
        achieved_milestones.iter()
            .flat_map(|m| &m.unlocked_features)
            .cloned()
            .collect()
    }
    
    /// Obtener configuración actual
    fn get_current_config(&self) -> Option<MilestoneConfig> {
        // Obtener la config del último milestone alcanzado
        self.milestones.iter()
            .filter(|m| m.achieved)
            .last()
            .map(|m| m.new_config.clone())
    }
    
    /// Actualizar métricas de rendimiento
    fn update_metrics(&mut self) -> Result<()> {
        if self.capital_history.len() < 2 {
            return Ok(());
        }
        
        let recent_snapshots: Vec<_> = self.capital_history.iter()
            .filter(|s| s.phase == self.current_phase)
            .collect();
        
        if recent_snapshots.is_empty() {
            return Ok(());
        }
        
        let start_capital = recent_snapshots[0].total_capital_sol;
        let current_capital = recent_snapshots.last().unwrap().total_capital_sol;
        
        self.performance_metrics.profit_this_phase_sol = current_capital - start_capital;
        self.performance_metrics.trades_this_phase = recent_snapshots.len() as u32;
        
        // Calcular tasa de crecimiento diaria
        let days_in_phase = (Utc::now() - recent_snapshots[0].timestamp).num_days() as f64;
        if days_in_phase > 0.0 {
            self.performance_metrics.capital_growth_rate_daily = 
                self.performance_metrics.profit_this_phase_sol / days_in_phase;
        }
        
        Ok(())
    }
    
    /// Obtener estado actual de progresión
    pub fn get_progression_status(&self) -> ProgressionStatus {
        let current_capital = self.capital_history.last()
            .map(|s| s.total_capital_sol)
            .unwrap_or(0.0);
        
        let next_milestone = self.milestones.iter()
            .find(|m| !m.achieved)
            .cloned();
        
        let progress_to_next = if let Some(ref milestone) = next_milestone {
            (current_capital / milestone.target_capital_sol * 100.0).min(100.0)
        } else {
            100.0
        };
        
        ProgressionStatus {
            current_phase: self.current_phase.clone(),
            current_capital_sol: current_capital,
            unlocked_features: self.unlocked_features.clone(),
            next_milestone,
            progress_to_next_percent: progress_to_next,
            achievements: self.milestones.iter()
                .filter(|m| m.achieved)
                .cloned()
                .collect(),
            performance_metrics: self.performance_metrics.clone(),
        }
    }

    /// 🚀 ENRIQUECIMIENTO: Utiliza campos específicos de CapitalSnapshot para análisis detallado
    pub fn analyze_capital_allocation_efficiency(&self) -> Result<AllocationAnalysis> {
        debug!("📊 Analyzing capital allocation efficiency using detailed snapshot data");
        
        let mut analysis = AllocationAnalysis {
            trading_capital_utilization: 0.0,
            reserved_capital_efficiency: 0.0,
            profit_per_trade_trend: Vec::new(),
            capital_allocation_recommendations: Vec::new(),
        };

        if self.capital_history.len() < 2 {
            return Ok(analysis);
        }

        // Analizar utilización del trading_capital_sol
        let recent_snapshots: Vec<&CapitalSnapshot> = self.capital_history.iter()
            .rev()
            .take(10)
            .collect();
        
        let total_trading_capital: f64 = recent_snapshots.iter()
            .map(|s| s.trading_capital_sol)
            .sum();
        let total_profits: f64 = recent_snapshots.iter()
            .map(|s| s.profit_since_last)
            .sum();
        
        // Utilización = profit generado / capital trading promedio
        let avg_trading_capital = total_trading_capital / recent_snapshots.len() as f64;
        analysis.trading_capital_utilization = if avg_trading_capital > 0.0 {
            total_profits / avg_trading_capital
        } else {
            0.0
        };

        // Analizar eficiencia del reserved_capital_sol
        let avg_reserved = recent_snapshots.iter()
            .map(|s| s.reserved_capital_sol)
            .sum::<f64>() / recent_snapshots.len() as f64;
        let total_capital = avg_trading_capital + avg_reserved;
        analysis.reserved_capital_efficiency = avg_reserved / total_capital;

        // Tendencia de profit_since_last
        for snapshot in recent_snapshots.iter().rev() {
            if snapshot.trades_since_last > 0 {
                let profit_per_trade = snapshot.profit_since_last / snapshot.trades_since_last as f64;
                analysis.profit_per_trade_trend.push(profit_per_trade);
            }
        }

        // Generar recomendaciones basadas en análisis
        if analysis.trading_capital_utilization < 0.02 { // <2% return
            analysis.capital_allocation_recommendations.push(
                "Consider increasing trading capital allocation or improving strategy efficiency".to_string()
            );
        }
        
        if analysis.reserved_capital_efficiency > 0.3 { // >30% reserved
            analysis.capital_allocation_recommendations.push(
                "High reserved capital - consider increasing trading allocation".to_string()
            );
        }

        debug!("📈 Capital allocation analysis complete: utilization={:.2}%, reserved={:.1}%", 
               analysis.trading_capital_utilization * 100.0, analysis.reserved_capital_efficiency * 100.0);
        
        Ok(analysis)
    }

    /// Calcula la volatilidad de las ganancias para optimización de riesgo
    fn calculate_profit_volatility(&self) -> Result<f64> {
        if self.capital_history.len() < 3 {
            return Ok(0.0);
        }
        
        let profits: Vec<f64> = self.capital_history.iter()
            .map(|s| s.profit_since_last)
            .collect();
        
        if profits.is_empty() {
            return Ok(0.0);
        }
        
        let mean = profits.iter().sum::<f64>() / profits.len() as f64;
        let variance = profits.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / profits.len() as f64;
        
        Ok(variance.sqrt())
    }

    /// 🚀 ENRIQUECIMIENTO: Utiliza campos específicos de MilestoneConfig para optimización
    pub fn optimize_milestone_config(&mut self, current_performance: &PerformanceMetrics) -> Result<ConfigOptimization> {
        debug!("⚙️ Optimizing milestone configuration based on performance");
        
        let current_config = self.get_current_config().unwrap_or_default();
        let optimized_config = current_config.clone();
        
        // Calcular volatilidad antes del borrowing mutable
        let profit_volatility = self.calculate_profit_volatility()?;
        
        let mut optimization = ConfigOptimization {
            current_config,
            optimized_config,
            changes_made: Vec::new(),
            expected_improvement: 0.0,
            confidence: 0.0,
        };

        // Obtener configuración actual del milestone
        if let Some(current_milestone) = self.milestones.iter_mut()
            .find(|m| m.phase == self.current_phase && !m.achieved) {
            
            let config = &mut current_milestone.new_config;
            let mut changes = Vec::new();

            // Optimizar max_positions basado en win rate
            if current_performance.get_win_rate() > 0.7 && config.max_positions < 3 {
                config.max_positions += 1;
                changes.push(format!("Increased max_positions to {} (high win rate)", config.max_positions));
            } else if current_performance.get_win_rate() < 0.4 && config.max_positions > 1 {
                config.max_positions -= 1;
                changes.push(format!("Decreased max_positions to {} (low win rate)", config.max_positions));
            }

            // Optimizar position_size_percent basado en profit promedio
            let avg_profit = if current_performance.total_trades > 0 {
                current_performance.total_profit / current_performance.total_trades as f64
            } else {
                0.0
            };
            if avg_profit > 0.15 && config.position_size_percent < 80.0 {
                config.position_size_percent += 10.0;
                changes.push(format!("Increased position size to {:.1}% (high profit per trade)", 
                                   config.position_size_percent));
            } else if avg_profit < 0.05 && config.position_size_percent > 40.0 {
                config.position_size_percent -= 10.0;
                changes.push(format!("Decreased position size to {:.1}% (low profit per trade)", 
                                   config.position_size_percent));
            }

            // Optimizar max_risk_score basado en volatilidad de returns
            if profit_volatility < 0.1 && config.max_risk_score < 0.8 {
                config.max_risk_score += 0.05;
                changes.push(format!("Increased risk tolerance to {:.2} (low volatility)", config.max_risk_score));
            } else if profit_volatility > 0.3 && config.max_risk_score > 0.3 {
                config.max_risk_score -= 0.05;
                changes.push(format!("Decreased risk tolerance to {:.2} (high volatility)", config.max_risk_score));
            }

            // Optimizar min_profit_target basado en win_rate_this_phase
            if self.performance_metrics.win_rate_this_phase > 0.8 && config.min_profit_target < 25.0 {
                config.min_profit_target += 2.0;
                changes.push(format!("Increased profit target to {:.1}% (high win rate)", 
                                   config.min_profit_target));
            }

            // Activar advanced_exits si el performance es bueno
            if !config.advanced_exits && avg_profit > 0.1 {
                config.advanced_exits = true;
                changes.push("Enabled advanced exits (good performance)".to_string());
            }

            // Activar scaling_allowed para fases avanzadas con buen rendimiento
            if !config.scaling_allowed && 
               matches!(self.current_phase, TradingPhase::IntermediateTrading | TradingPhase::AdvancedTrading) &&
               current_performance.get_win_rate() > 0.6 {
                config.scaling_allowed = true;
                changes.push("Enabled position scaling (advanced phase + good win rate)".to_string());
            }

            // Expandir available_strategies basado en el progreso
            if config.available_strategies.len() == 1 && current_performance.get_win_rate() > 0.6 {
                config.available_strategies.push("momentum_follow".to_string());
                changes.push("Added momentum_follow strategy".to_string());
            }

            optimization.optimized_config = current_milestone.new_config.clone();
            optimization.changes_made = changes.clone();
            optimization.expected_improvement = changes.len() as f64 * 0.05; // 5% per change
            optimization.confidence = if changes.len() > 0 { 0.75 } else { 1.0 };

            debug!("🎯 Config optimization complete: {} changes, {:.1}% expected improvement", 
                   changes.len(), optimization.expected_improvement * 100.0);
        }

        Ok(optimization)
    }

    /// 🚀 ENRIQUECIMIENTO: Utiliza campos específicos de ProgressionMetrics para predicciones
    pub fn predict_milestone_achievement(&self) -> Result<MilestonePrediction> {
        debug!("🔮 Predicting milestone achievement using detailed metrics");
        
        let mut prediction = MilestonePrediction {
            next_milestone_target: 0.0,
            predicted_achievement_date: None,
            projected_days_to_achievement: 0,
            confidence_level: 0.0,
            factors_analysis: HashMap::new(),
        };

        // Encontrar próximo milestone
        if let Some(next_milestone) = self.milestones.iter().find(|m| !m.achieved) {
            prediction.next_milestone_target = next_milestone.target_capital_sol;
            
            let current_capital = self.capital_history.last()
                .map(|s| s.total_capital_sol)
                .unwrap_or(0.0);
            
            let capital_needed = next_milestone.target_capital_sol - current_capital;
            
            // Usar days_in_current_phase para calcular velocidad de crecimiento
            let days_in_phase = self.performance_metrics.days_in_current_phase as f64;
            let win_rate = self.performance_metrics.win_rate_this_phase;
            let avg_profit = self.performance_metrics.average_profit_per_trade;
            
            // Usar projected_days_to_next_milestone como base
            let base_projection = self.performance_metrics.projected_days_to_next_milestone as f64;
            
            // Factores de análisis usando campos específicos
            prediction.factors_analysis.insert("current_win_rate".to_string(), win_rate);
            prediction.factors_analysis.insert("avg_profit_per_trade".to_string(), avg_profit);
            prediction.factors_analysis.insert("days_in_current_phase".to_string(), days_in_phase);
            prediction.factors_analysis.insert("capital_growth_rate_daily".to_string(), 
                                             self.performance_metrics.capital_growth_rate_daily);
            
            // Calcular predicción refinada
            if self.performance_metrics.capital_growth_rate_daily > 0.0 {
                let days_needed = capital_needed / self.performance_metrics.capital_growth_rate_daily;
                prediction.projected_days_to_achievement = days_needed.max(1.0) as u32;
                
                // Ajustar por win rate
                let win_rate_adjustment = if win_rate > 0.6 { 0.8 } else { 1.2 };
                prediction.projected_days_to_achievement = 
                    (prediction.projected_days_to_achievement as f64 * win_rate_adjustment) as u32;
                
                prediction.predicted_achievement_date = Some(
                    Utc::now() + Duration::days(prediction.projected_days_to_achievement as i64)
                );
                
                // Calcular confianza basada en consistencia
                let consistency_factor = if days_in_phase > 7.0 { 0.8 } else { 0.5 };
                let growth_factor = if self.performance_metrics.capital_growth_rate_daily > 0.01 { 0.9 } else { 0.6 };
                prediction.confidence_level = consistency_factor * growth_factor * win_rate;
            } else {
                prediction.projected_days_to_achievement = base_projection as u32;
                prediction.confidence_level = 0.3; // Baja confianza sin crecimiento
            }

            debug!("🎯 Milestone prediction: {} days to {:.2} SOL (confidence: {:.1}%)", 
                   prediction.projected_days_to_achievement, 
                   prediction.next_milestone_target,
                   prediction.confidence_level * 100.0);
        }

        Ok(prediction)
    }
}

/// Resultado de actualización de progresión
#[derive(Debug)]
pub struct ProgressionUpdate {
    pub previous_capital: f64,
    pub new_capital: f64,
    pub previous_phase: TradingPhase,
    pub new_phase: TradingPhase,
    pub phase_changed: bool,
    pub achieved_milestones: Vec<Milestone>,
    pub newly_unlocked_features: Vec<String>,
    pub updated_config: Option<MilestoneConfig>,
    pub progress_metrics: ProgressionMetrics,
}

/// Estado actual de progresión
#[derive(Debug)]
pub struct ProgressionStatus {
    pub current_phase: TradingPhase,
    pub current_capital_sol: f64,
    pub unlocked_features: Vec<String>,
    pub next_milestone: Option<Milestone>,
    pub progress_to_next_percent: f64,
    pub achievements: Vec<Milestone>,
    pub performance_metrics: ProgressionMetrics,
}

/// 🚀 ENRIQUECIMIENTO: Estructuras para análisis avanzado de capital progression
#[derive(Debug, Clone)]
pub struct AllocationAnalysis {
    pub trading_capital_utilization: f64,
    pub reserved_capital_efficiency: f64,
    pub profit_per_trade_trend: Vec<f64>,
    pub capital_allocation_recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ConfigOptimization {
    pub current_config: MilestoneConfig,
    pub optimized_config: MilestoneConfig,
    pub changes_made: Vec<String>,
    pub expected_improvement: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct MilestonePrediction {
    pub next_milestone_target: f64,
    pub predicted_achievement_date: Option<DateTime<Utc>>,
    pub projected_days_to_achievement: u32,
    pub confidence_level: f64,
    pub factors_analysis: HashMap<String, f64>,
}

/// 🚀 ENRIQUECIMIENTO: Implementación para ProgressionMetrics utilizando todos los campos
impl ProgressionMetrics {
    pub fn new() -> Self {
        Self {
            days_in_current_phase: 0,
            trades_this_phase: 0,
            win_rate_this_phase: 0.0,
            average_profit_per_trade: 0.0,
            projected_days_to_next_milestone: 0,
            capital_growth_rate_daily: 0.0,
            profit_this_phase_sol: 0.0,
        }
    }

    /// Actualiza todas las métricas usando todos los campos
    pub fn update_comprehensive_metrics(&mut self, 
                                      snapshots: &[CapitalSnapshot], 
                                      current_phase: &TradingPhase) -> Result<()> {
        // Calcular days_in_current_phase
        let phase_snapshots: Vec<&CapitalSnapshot> = snapshots.iter()
            .filter(|s| std::mem::discriminant(&s.phase) == std::mem::discriminant(current_phase))
            .collect();
        
        if let Some(first_phase_snapshot) = phase_snapshots.first() {
            self.days_in_current_phase = (Utc::now() - first_phase_snapshot.timestamp).num_days() as u32;
        }

        // Calcular trades_this_phase sumando trades_since_last
        self.trades_this_phase = phase_snapshots.iter()
            .map(|s| s.trades_since_last)
            .sum();

        // Calcular win_rate_this_phase basado en profit_since_last
        if !phase_snapshots.is_empty() {
            let winning_trades = phase_snapshots.iter()
                .filter(|s| s.profit_since_last > 0.0)
                .count();
            self.win_rate_this_phase = winning_trades as f64 / phase_snapshots.len() as f64;
        }

        // Calcular average_profit_per_trade
        let total_profit: f64 = phase_snapshots.iter()
            .map(|s| s.profit_since_last)
            .sum();
        self.average_profit_per_trade = if self.trades_this_phase > 0 {
            total_profit / self.trades_this_phase as f64
        } else {
            0.0
        };

        // Calcular capital_growth_rate_daily
        if let (Some(first), Some(last)) = (phase_snapshots.first(), phase_snapshots.last()) {
            let days = (last.timestamp - first.timestamp).num_days() as f64;
            if days > 0.0 {
                let capital_growth = last.total_capital_sol - first.total_capital_sol;
                self.capital_growth_rate_daily = capital_growth / days;
            }
        }

        // Estimar projected_days_to_next_milestone
        if self.capital_growth_rate_daily > 0.0 {
            // Asumir que necesitamos 0.5 SOL más para el próximo milestone
            let capital_needed = 0.5; // placeholder
            self.projected_days_to_next_milestone = (capital_needed / self.capital_growth_rate_daily) as u32;
        }

        Ok(())
    }

    /// Genera reporte detallado usando todos los campos
    pub fn generate_detailed_report(&self) -> String {
        format!(
            "📊 PROGRESSION METRICS REPORT:\n\
             📅 Days in Current Phase: {}\n\
             📈 Trades This Phase: {}\n\
             🎯 Win Rate This Phase: {:.1}%\n\
             💰 Average Profit per Trade: {:.4} SOL\n\
             🔮 Projected Days to Next Milestone: {}\n\
             📊 Capital Growth Rate (Daily): {:.4} SOL",
            self.days_in_current_phase,
            self.trades_this_phase,
            self.win_rate_this_phase * 100.0,
            self.average_profit_per_trade,
            self.projected_days_to_next_milestone,
            self.capital_growth_rate_daily
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_phase_determination() {
        assert_eq!(CapitalProgressionManager::determine_phase(0.3), TradingPhase::Accumulation);
        assert_eq!(CapitalProgressionManager::determine_phase(0.7), TradingPhase::BasicTrading);
        assert_eq!(CapitalProgressionManager::determine_phase(1.5), TradingPhase::IntermediateTrading);
        assert_eq!(CapitalProgressionManager::determine_phase(3.0), TradingPhase::AdvancedTrading);
    }
    
    #[test]
    fn test_milestone_achievement() {
        let mut manager = CapitalProgressionManager::new(0.3).unwrap();
        let update = manager.update_capital(0.6, 0.3).unwrap();
        
        assert!(update.phase_changed);
        assert_eq!(update.new_phase, TradingPhase::BasicTrading);
        assert!(!update.achieved_milestones.is_empty());
    }
}
