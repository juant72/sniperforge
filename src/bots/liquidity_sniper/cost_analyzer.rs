// SniperForge Enterprise v3.0 - Cost Analysis Module
// Análisis de costos y viabilidad económica para operaciones

use anyhow::Result;
use tracing::{info, warn, debug};

/// Analizador de costos para operaciones de trading
#[derive(Debug, Clone)]
pub struct CostAnalyzer {
    pub config: CostConfig,
}

/// Configuración de costos
#[derive(Debug, Clone)]
pub struct CostConfig {
    pub transaction_cost_sol: f64,
    pub priority_fee_sol: f64,
    pub slippage_cost_percent: f64,
    pub mev_protection_cost_sol: f64,
    pub total_entry_cost_sol: f64,
    pub total_exit_cost_sol: f64,
    pub round_trip_cost_sol: f64,
    pub min_profit_to_breakeven_percent: f64,
    pub recommended_min_position_sol: f64,
    pub cost_efficiency_threshold: f64,
}

/// Resultado del análisis de costos
#[derive(Debug, Clone)]
pub struct CostAnalysis {
    pub position_size_sol: f64,
    pub total_costs_sol: f64,
    pub cost_percentage: f64,
    pub min_profit_needed_percent: f64,
    pub min_profit_needed_sol: f64,
    pub breakeven_price_change_percent: f64,
    pub cost_efficiency_score: f64,
    pub is_economically_viable: bool,
    pub recommendations: Vec<String>,
    pub cost_breakdown: CostBreakdown,
}

/// Desglose detallado de costos
#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub entry_transaction_cost: f64,
    pub entry_priority_fee: f64,
    pub entry_slippage_cost: f64,
    pub mev_protection_cost: f64,
    pub exit_transaction_cost: f64,
    pub exit_priority_fee: f64,
    pub exit_slippage_cost: f64,
    pub total_fixed_costs: f64,
    pub total_variable_costs: f64,
    pub total_all_costs: f64,
}

impl CostAnalyzer {
    /// Crear nuevo analizador de costos
    pub fn new(config: CostConfig) -> Self {
        Self { config }
    }

    /// Analizar viabilidad económica de una posición
    pub fn analyze_position_viability(
        &self,
        position_size_sol: f64,
        expected_profit_percent: f64,
        price_impact_percent: f64,
    ) -> Result<CostAnalysis> {
        info!("💰 Analizando viabilidad económica");
        info!("   Tamaño posición: {} SOL", position_size_sol);
        info!("   Ganancia esperada: {:.1}%", expected_profit_percent);
        
        // Calcular costos detallados
        let cost_breakdown = self.calculate_cost_breakdown(position_size_sol, price_impact_percent)?;
        
        // Costos totales
        let total_costs_sol = cost_breakdown.total_all_costs;
        let cost_percentage = (total_costs_sol / position_size_sol) * 100.0;
        
        // Ganancia mínima necesaria para ser rentable
        let min_profit_needed_sol = total_costs_sol * 1.5; // 50% buffer
        let min_profit_needed_percent = (min_profit_needed_sol / position_size_sol) * 100.0;
        
        // Cambio de precio necesario para breakeven
        let breakeven_price_change_percent = cost_percentage + 1.0; // 1% buffer adicional
        
        // Score de eficiencia de costos (0-1, higher is better)
        let cost_efficiency_score = self.calculate_efficiency_score(
            position_size_sol,
            total_costs_sol,
            expected_profit_percent,
        )?;
        
        // Viabilidad económica
        let is_economically_viable = self.assess_economic_viability(
            expected_profit_percent,
            min_profit_needed_percent,
            cost_efficiency_score,
        )?;
        
        // Recomendaciones
        let recommendations = self.generate_recommendations(
            position_size_sol,
            cost_percentage,
            expected_profit_percent,
            cost_efficiency_score,
        )?;
        
        let analysis = CostAnalysis {
            position_size_sol,
            total_costs_sol,
            cost_percentage,
            min_profit_needed_percent,
            min_profit_needed_sol,
            breakeven_price_change_percent,
            cost_efficiency_score,
            is_economically_viable,
            recommendations,
            cost_breakdown,
        };
        
        // Log resultados
        if is_economically_viable {
            info!("✅ Posición VIABLE - Costos: {:.3} SOL ({:.1}%)", 
                  total_costs_sol, cost_percentage);
        } else {
            warn!("❌ Posición NO VIABLE - Costos demasiado altos: {:.1}%", cost_percentage);
        }
        
        Ok(analysis)
    }
    
    /// Calcular desglose detallado de costos
    fn calculate_cost_breakdown(
        &self,
        position_size_sol: f64,
        price_impact_percent: f64,
    ) -> Result<CostBreakdown> {
        // Costos de entrada
        let entry_transaction_cost = self.config.transaction_cost_sol;
        let entry_priority_fee = self.config.priority_fee_sol;
        let entry_slippage_cost = position_size_sol * (self.config.slippage_cost_percent / 100.0);
        let mev_protection_cost = self.config.mev_protection_cost_sol;
        
        // Costos de salida  
        let exit_transaction_cost = self.config.transaction_cost_sol;
        let exit_priority_fee = self.config.priority_fee_sol;
        
        // Slippage de salida (puede ser mayor si hay price impact)
        let exit_slippage_multiplier = 1.0 + (price_impact_percent / 100.0);
        let exit_slippage_cost = position_size_sol * 
                                (self.config.slippage_cost_percent / 100.0) * 
                                exit_slippage_multiplier;
        
        // Totales
        let total_fixed_costs = entry_transaction_cost + entry_priority_fee + 
                               mev_protection_cost + exit_transaction_cost + exit_priority_fee;
        
        let total_variable_costs = entry_slippage_cost + exit_slippage_cost;
        
        let total_all_costs = total_fixed_costs + total_variable_costs;
        
        Ok(CostBreakdown {
            entry_transaction_cost,
            entry_priority_fee,
            entry_slippage_cost,
            mev_protection_cost,
            exit_transaction_cost,
            exit_priority_fee,
            exit_slippage_cost,
            total_fixed_costs,
            total_variable_costs,
            total_all_costs,
        })
    }
    
    /// Calcular score de eficiencia de costos
    fn calculate_efficiency_score(
        &self,
        position_size_sol: f64,
        total_costs_sol: f64,
        expected_profit_percent: f64,
    ) -> Result<f64> {
        let cost_ratio = total_costs_sol / position_size_sol;
        let profit_ratio = expected_profit_percent / 100.0;
        
        // Score basado en ratio profit/cost
        let profit_cost_ratio = if cost_ratio > 0.0 {
            profit_ratio / cost_ratio
        } else {
            0.0
        };
        
        // Normalizar a 0-1 scale
        let efficiency_score = (profit_cost_ratio / 10.0).min(1.0).max(0.0);
        
        Ok(efficiency_score)
    }
    
    /// Evaluar viabilidad económica
    fn assess_economic_viability(
        &self,
        expected_profit_percent: f64,
        min_profit_needed_percent: f64,
        cost_efficiency_score: f64,
    ) -> Result<bool> {
        let profit_viable = expected_profit_percent >= min_profit_needed_percent;
        let efficiency_viable = cost_efficiency_score >= self.config.cost_efficiency_threshold;
        let minimum_return_viable = expected_profit_percent >= self.config.min_profit_to_breakeven_percent;
        
        Ok(profit_viable && efficiency_viable && minimum_return_viable)
    }
    
    /// Generar recomendaciones
    fn generate_recommendations(
        &self,
        position_size_sol: f64,
        cost_percentage: f64,
        expected_profit_percent: f64,
        cost_efficiency_score: f64,
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();
        
        // Recomendaciones basadas en tamaño de posición
        if position_size_sol < self.config.recommended_min_position_sol {
            recommendations.push(format!(
                "Posición muy pequeña. Recomendado mínimo: {:.2} SOL para eficiencia de costos",
                self.config.recommended_min_position_sol
            ));
        }
        
        // Recomendaciones basadas en costos
        if cost_percentage > 5.0 {
            recommendations.push("Costos muy altos (>5%). Considerar posición mayor o esperar mejor oportunidad".to_string());
        } else if cost_percentage > 3.0 {
            recommendations.push("Costos moderados (3-5%). Monitorear cuidadosamente".to_string());
        } else {
            recommendations.push("Costos aceptables (<3%). Buena eficiencia económica".to_string());
        }
        
        // Recomendaciones basadas en ganancia esperada
        if expected_profit_percent < 10.0 {
            recommendations.push("Ganancia esperada baja. Considerar solo oportunidades >15%".to_string());
        }
        
        // Recomendaciones basadas en eficiencia
        if cost_efficiency_score < 0.5 {
            recommendations.push("Baja eficiencia de costos. Reconsiderar esta oportunidad".to_string());
        } else if cost_efficiency_score > 0.8 {
            recommendations.push("Excelente eficiencia de costos. Oportunidad atractiva".to_string());
        }
        
        // Recomendaciones específicas para capital pequeño
        if position_size_sol < 0.5 {
            recommendations.push("Capital pequeño: Enfocar en oportunidades >20% ganancia".to_string());
            recommendations.push("Usar máximo 1 posición simultánea para optimizar costos".to_string());
            recommendations.push("Considerar acumular más capital antes de trading activo".to_string());
        }
        
        Ok(recommendations)
    }
    
    /// Validar si una oportunidad es económicamente viable
    pub fn validate_opportunity_economics(
        &self,
        position_size_sol: f64,
        expected_profit_percent: f64,
        price_impact_percent: f64,
        liquidity_usd: f64,
    ) -> Result<bool> {
        let analysis = self.analyze_position_viability(
            position_size_sol,
            expected_profit_percent,
            price_impact_percent,
        )?;
        
        // Validaciones adicionales específicas para la oportunidad
        let liquidity_adequate = liquidity_usd >= (position_size_sol * 200.0 * 100.0); // 200x en USD
        let slippage_acceptable = price_impact_percent <= 8.0;
        
        let economically_viable = analysis.is_economically_viable && 
                                 liquidity_adequate && 
                                 slippage_acceptable;
        
        if !economically_viable {
            debug!("❌ Oportunidad rechazada por análisis económico");
            debug!("   Viable: {}", analysis.is_economically_viable);
            debug!("   Liquidez adecuada: {}", liquidity_adequate);
            debug!("   Slippage aceptable: {}", slippage_acceptable);
        }
        
        Ok(economically_viable)
    }
    
    /// Calcular tamaño óptimo de posición basado en costos
    pub fn calculate_optimal_position_size(
        &self,
        available_capital_sol: f64,
        expected_profit_percent: f64,
        target_cost_percentage: f64,
    ) -> Result<f64> {
        let mut optimal_size = available_capital_sol * 0.8; // Start with 80% of capital
        
        // Iteratively find the size that gives target cost percentage
        for _ in 0..10 {
            let analysis = self.analyze_position_viability(optimal_size, expected_profit_percent, 2.0)?;
            
            if analysis.cost_percentage <= target_cost_percentage {
                break;
            }
            
            // Increase size to reduce cost percentage
            optimal_size = optimal_size * 1.1;
            
            if optimal_size > available_capital_sol {
                optimal_size = available_capital_sol;
                break;
            }
        }
        
        Ok(optimal_size.min(available_capital_sol))
    }
}

impl Default for CostConfig {
    fn default() -> Self {
        Self {
            transaction_cost_sol: 0.001,
            priority_fee_sol: 0.00005,
            slippage_cost_percent: 1.5,
            mev_protection_cost_sol: 0.0005,
            total_entry_cost_sol: 0.00155,
            total_exit_cost_sol: 0.00155,
            round_trip_cost_sol: 0.0031,
            min_profit_to_breakeven_percent: 2.5,
            recommended_min_position_sol: 0.15,
            cost_efficiency_threshold: 0.6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cost_analysis_small_position() {
        let config = CostConfig::default();
        let analyzer = CostAnalyzer::new(config);
        
        let analysis = analyzer.analyze_position_viability(0.2, 20.0, 2.0).unwrap();
        
        assert!(analysis.cost_percentage < 10.0);
        assert!(analysis.min_profit_needed_percent > 0.0);
    }
    
    #[test]
    fn test_economic_viability() {
        let config = CostConfig::default();
        let analyzer = CostAnalyzer::new(config);
        
        // Small position should require higher profit
        let viable = analyzer.validate_opportunity_economics(0.1, 30.0, 3.0, 20000.0).unwrap();
        assert!(viable);
        
        // Low profit should not be viable
        let not_viable = analyzer.validate_opportunity_economics(0.1, 5.0, 3.0, 20000.0).unwrap();
        assert!(!not_viable);
    }
}
