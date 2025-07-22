// ===== ENTERPRISE RISK MANAGER =====
// Gestión profesional de riesgos para operaciones de arbitraje institucional

use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use std::sync::atomic::Ordering;

use crate::types::*;

// ===== ENTERPRISE RISK CONSTANTS =====
const ENTERPRISE_RISK_DAILY_VOLUME: f64 = 1000.0; // SOL - Institutional volume limits
const INSTITUTIONAL_MAX_TRADE_SOL: f64 = 100.0; // Enterprise position sizing
const MILITARY_MIN_TRADE_SOL: f64 = 0.1; // Precision execution minimum

pub struct EnterpriseRiskManager;

impl EnterpriseRiskManager {
    /// INSTITUTIONAL RISK CHECKS - Pre-execution validation
    pub fn execute_institutional_risk_checks(
        risk_metrics: &RiskMetrics,
        emergency_stop: &std::sync::atomic::AtomicBool
    ) -> Result<()> {
        info!("🛡️  EXECUTING INSTITUTIONAL RISK PROTOCOLS");
        
        if risk_metrics.current_exposure_usd > risk_metrics.max_exposure_usd {
            error!("🚨 INSTITUTIONAL ALERT: Risk exposure exceeds enterprise limits");
            return Err(anyhow!("ENTERPRISE RISK LIMIT EXCEEDED - Mission aborted"));
        }
        
        if risk_metrics.daily_pnl < -1000.0 {
            error!("🚨 MILITARY ALERT: Daily loss threshold breached");
            emergency_stop.store(true, Ordering::Relaxed);
            return Err(anyhow!("ENTERPRISE EMERGENCY STOP - Daily loss limit reached"));
        }
        
        info!("✅ INSTITUTIONAL RISK ASSESSMENT: All parameters within enterprise limits");
        Ok(())
    }
    
    /// ENTERPRISE RISK FILTERS - Filter opportunities by institutional criteria
    pub fn apply_enterprise_risk_filters(
        opportunities: Vec<DirectOpportunity>,
        adaptive_config: &AdaptiveConfig
    ) -> Result<Vec<DirectOpportunity>> {
        let original_count = opportunities.len();
        info!("🛡️  APPLYING ENTERPRISE RISK MANAGEMENT FILTERS");
        
        let filtered: Vec<_> = opportunities.into_iter()
            .filter(|opp| {
                let trade_size_sol = opp.amount_in as f64 / 1e9;
                
                // Institutional size requirements
                if trade_size_sol < MILITARY_MIN_TRADE_SOL || trade_size_sol > INSTITUTIONAL_MAX_TRADE_SOL {
                    debug!("❌ Trade size outside institutional parameters: {:.3} SOL", trade_size_sol);
                    return false;
                }
                
                // Military-grade profit threshold with volatility adjustment
                let adjusted_threshold = adaptive_config.min_profit_threshold as f64 * 
                                       adaptive_config.volatility_adjustment;
                let profit_bps = (opp.profit_lamports * 10_000) / opp.amount_in as i64;
                
                if (profit_bps as f64) < adjusted_threshold {
                    debug!("❌ Profit below enterprise threshold: {:.2}% < {:.2}%", 
                           profit_bps as f64 / 100.0, adjusted_threshold / 100.0);
                    return false;
                }
                
                info!("✅ Opportunity passed enterprise filters: {:.2}% profit, {:.3} SOL", 
                      profit_bps as f64 / 100.0, trade_size_sol);
                true
            })
            .collect();
        
        info!("🛡️  ENTERPRISE RISK FILTER RESULTS: {}/{} opportunities cleared", filtered.len(), original_count);
        info!("🎖️  MILITARY STATUS: {} opportunities meet institutional standards", filtered.len());
        Ok(filtered)
    }
    
    /// CALCULATE ENTERPRISE OPPORTUNITY SCORE - Risk-adjusted scoring
    pub fn calculate_enterprise_opportunity_score(
        opportunity: &DirectOpportunity,
        market_metrics: &MarketMetrics,
        adaptive_config: &AdaptiveConfig
    ) -> f64 {
        let base_profit = opportunity.profit_lamports as f64 / 1e9;
        let volatility_factor = 1.0 / (1.0 + market_metrics.volatility_index);
        let institutional_score = base_profit * volatility_factor * adaptive_config.risk_multiplier;
        
        // Enterprise bonus factors
        let enterprise_multiplier = if institutional_score > 0.01 { 1.2 } else { 1.0 }; // Bonus for high profit
        
        institutional_score * enterprise_multiplier
    }
    
    /// SELECT ENTERPRISE OPTIMAL OPPORTUNITY - Risk-adjusted selection
    pub fn select_enterprise_optimal_opportunity(
        opportunities: Vec<DirectOpportunity>,
        market_metrics: &MarketMetrics,
        adaptive_config: &AdaptiveConfig
    ) -> Result<DirectOpportunity> {
        info!("🎯 SELECTING OPTIMAL ENTERPRISE TARGET");
        
        let optimal = opportunities.into_iter()
            .max_by(|a, b| {
                let score_a = Self::calculate_enterprise_opportunity_score(a, market_metrics, adaptive_config);
                let score_b = Self::calculate_enterprise_opportunity_score(b, market_metrics, adaptive_config);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("ENTERPRISE ERROR: No optimal opportunity identified"))?;
        
        info!("✅ ENTERPRISE TARGET SELECTED: Optimal opportunity identified");
        Ok(optimal)
    }
    
    /// UPDATE INSTITUTIONAL PERFORMANCE METRICS - Post-execution analysis
    pub fn update_institutional_performance_metrics(
        performance_metrics: &mut PerformanceMetrics,
        risk_metrics: &mut RiskMetrics,
        opportunity: &DirectOpportunity,
        success: bool
    ) {
        info!("📊 UPDATING INSTITUTIONAL PERFORMANCE METRICS");
        
        if success {
            performance_metrics.successful_trades += 1;
            performance_metrics.total_profit_usd += (opportunity.profit_lamports as f64 / 1e9) * 200.0;
            info!("✅ ENTERPRISE SUCCESS: Trade profit logged - {:.6} SOL", 
                  opportunity.profit_lamports as f64 / 1e9);
        } else {
            warn!("⚠️  INSTITUTIONAL ALERT: Trade unsuccessful - adjusting risk metrics");
        }
        
        performance_metrics.total_trades += 1;
        
        if performance_metrics.total_trades > 0 {
            risk_metrics.success_rate = performance_metrics.successful_trades as f64 / performance_metrics.total_trades as f64;
            info!("📈 ENTERPRISE SUCCESS RATE: {:.2}%", risk_metrics.success_rate * 100.0);
        }
        
        info!("🎖️  MILITARY METRICS: Performance data updated with institutional standards");
    }
    
    /// VALIDATE EXECUTION - Pre-execution safety checks
    pub fn validate_execution(opportunity: &DirectOpportunity, min_profit_sol: f64) -> Result<()> {
        let profit_sol = opportunity.profit_lamports as f64 / 1e9;
        
        if profit_sol < min_profit_sol {
            return Err(anyhow!("Profit below mainnet threshold: {:.6} SOL < {:.6} SOL", 
                              profit_sol, min_profit_sol));
        }
        
        info!("✅ Pre-execution validation passed");
        Ok(())
    }
    
    /// CHECK BALANCE SUFFICIENCY - Ensure adequate funds for execution
    pub fn check_balance_sufficiency(current_balance: f64, required_balance: f64) -> Result<()> {
        if current_balance < required_balance {
            error!("🚨 ENTERPRISE ALERT: Insufficient institutional capital");
            return Err(anyhow!("INSTITUTIONAL CAPITAL SHORTAGE: {:.3} SOL required, {:.3} SOL available", 
                required_balance, current_balance));
        }
        
        info!("✅ ENTERPRISE VALIDATION: Transaction cleared by institutional protocols");
        Ok(())
    }
}
