// SniperForge Enterprise v3.0 - Capital Progression Manager
// Sistema de progresión automática basada en capital acumulado

use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::{info, warn, debug};
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
    target_capital_sol: f64,
    phase: TradingPhase,
    unlocked_features: Vec<String>,
    new_config: MilestoneConfig,
    description: String,
    achieved: bool,
    achieved_at: Option<DateTime<Utc>>,
}

/// Configuración desbloqueada en cada milestone
#[derive(Debug, Clone)]
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
            current_phase: phase,
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
            achieved_milestones,
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

impl ProgressionMetrics {
    fn new() -> Self {
        Self {
            days_in_current_phase: 0,
            profit_this_phase_sol: 0.0,
            trades_this_phase: 0,
            win_rate_this_phase: 0.0,
            average_profit_per_trade: 0.0,
            projected_days_to_next_milestone: 0,
            capital_growth_rate_daily: 0.0,
        }
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
