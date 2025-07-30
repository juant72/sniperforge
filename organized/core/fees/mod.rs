// ================================================================================
// FEES CALCULATION - Cálculo de Fees y Costos
// ================================================================================

use std::collections::HashMap;
use anyhow::Result;
use tracing::{debug, warn};
use serde::{Serialize, Deserialize};

use crate::{CoreResult, CoreError};
use crate::types::RouteInfo;

/// Calculadora de fees
pub struct FeeCalculator {
    config: FeesConfig,
    dex_fees: HashMap<String, DexFeeInfo>,
    gas_tracker: GasTracker,
    fee_history: Vec<FeeRecord>,
}

/// Configuración de fees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeesConfig {
    pub platform_fee_bps: u16,
    pub jupiter_fee_bps: u16,
    pub priority_fee_multiplier: f64,
    pub gas_optimization: bool,
    pub dynamic_fees: bool,
    pub max_fee_percentage: f64,
}

/// Información de fees de DEX
#[derive(Debug, Clone)]
pub struct DexFeeInfo {
    pub name: String,
    pub base_fee_bps: u16,
    pub platform_fee_bps: u16,
    pub gas_cost_lamports: u64,
    pub priority_fee_lamports: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Tracker de gas
#[derive(Debug, Clone, Default)]
pub struct GasTracker {
    pub recent_gas_prices: Vec<GasRecord>,
    pub avg_gas_price: u64,
    pub peak_gas_price: u64,
    pub low_gas_price: u64,
    pub network_congestion: f64,
}

/// Registro de gas
#[derive(Debug, Clone)]
pub struct GasRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub gas_price_lamports: u64,
    pub network_congestion: f64,
    pub transaction_success_rate: f64,
}

/// Registro de fee
#[derive(Debug, Clone)]
pub struct FeeRecord {
    pub transaction_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub breakdown: FeeBreakdown,
    pub actual_cost_sol: f64,
    pub estimated_cost_sol: f64,
    pub accuracy: f64,
}

/// Desglose detallado de fees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeBreakdown {
    pub dex_fees_sol: f64,
    pub jupiter_fees_sol: f64,
    pub platform_fees_sol: f64,
    pub gas_fees_sol: f64,
    pub priority_fees_sol: f64,
    pub slippage_cost_sol: f64,
    pub total_sol: f64,
    pub percentage_of_trade: f64,
}

/// Estimación de fee
#[derive(Debug, Clone)]
pub struct FeeEstimate {
    pub breakdown: FeeBreakdown,
    pub confidence: f64,
    pub valid_until: chrono::DateTime<chrono::Utc>,
    pub recommendations: Vec<FeeRecommendation>,
}

/// Recomendación de fee
#[derive(Debug, Clone)]
pub struct FeeRecommendation {
    pub recommendation_type: RecommendationType,
    pub description: String,
    pub potential_savings_sol: f64,
    pub implementation_difficulty: Difficulty,
}

/// Tipo de recomendación
#[derive(Debug, Clone)]
pub enum RecommendationType {
    ReducePriorityFee,
    OptimizeRoute,
    WaitForLowerGas,
    UseAlternativeDex,
    BatchTransactions,
}

/// Dificultad de implementación
#[derive(Debug, Clone)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl FeeCalculator {
    /// Crear nueva calculadora de fees
    pub fn new(config: &FeesConfig) -> Self {
        let mut dex_fees = HashMap::new();
        
        // Inicializar fees conocidos de DEXs populares
        dex_fees.insert("Raydium".to_string(), DexFeeInfo {
            name: "Raydium".to_string(),
            base_fee_bps: 25,        // 0.25%
            platform_fee_bps: 0,
            gas_cost_lamports: 15000,
            priority_fee_lamports: 10000,
            last_updated: chrono::Utc::now(),
        });
        
        dex_fees.insert("Orca".to_string(), DexFeeInfo {
            name: "Orca".to_string(),
            base_fee_bps: 30,        // 0.30%
            platform_fee_bps: 0,
            gas_cost_lamports: 12000,
            priority_fee_lamports: 8000,
            last_updated: chrono::Utc::now(),
        });
        
        dex_fees.insert("Serum".to_string(), DexFeeInfo {
            name: "Serum".to_string(),
            base_fee_bps: 22,        // 0.22%
            platform_fee_bps: 0,
            gas_cost_lamports: 20000,
            priority_fee_lamports: 12000,
            last_updated: chrono::Utc::now(),
        });
        
        dex_fees.insert("Phoenix".to_string(), DexFeeInfo {
            name: "Phoenix".to_string(),
            base_fee_bps: 20,        // 0.20%
            platform_fee_bps: 0,
            gas_cost_lamports: 18000,
            priority_fee_lamports: 9000,
            last_updated: chrono::Utc::now(),
        });
        
        Self {
            config: config.clone(),
            dex_fees,
            gas_tracker: GasTracker::default(),
            fee_history: Vec::new(),
        }
    }
    
    /// Calcular fees totales para una operación
    pub fn calculate_total_fees(&self, amount_sol: f64, route_info: &RouteInfo) -> CoreResult<FeeBreakdown> {
        debug!("💰 Calculando fees para amount: {:.6} SOL", amount_sol);
        
        let mut breakdown = FeeBreakdown {
            dex_fees_sol: 0.0,
            jupiter_fees_sol: 0.0,
            platform_fees_sol: 0.0,
            gas_fees_sol: 0.0,
            priority_fees_sol: 0.0,
            slippage_cost_sol: 0.0,
            total_sol: 0.0,
            percentage_of_trade: 0.0,
        };
        
        // 1. Calcular fees de DEX
        breakdown.dex_fees_sol = self.calculate_dex_fees(amount_sol, route_info)?;
        
        // 2. Calcular fees de Jupiter
        breakdown.jupiter_fees_sol = self.calculate_jupiter_fees(amount_sol)?;
        
        // 3. Calcular fees de plataforma
        breakdown.platform_fees_sol = self.calculate_platform_fees(amount_sol)?;
        
        // 4. Calcular fees de gas
        breakdown.gas_fees_sol = self.calculate_gas_fees(route_info)?;
        
        // 5. Calcular priority fees
        breakdown.priority_fees_sol = self.calculate_priority_fees(route_info)?;
        
        // 6. Estimar costo de slippage
        breakdown.slippage_cost_sol = self.estimate_slippage_cost(amount_sol, route_info)?;
        
        // 7. Calcular total
        breakdown.total_sol = breakdown.dex_fees_sol
            + breakdown.jupiter_fees_sol
            + breakdown.platform_fees_sol
            + breakdown.gas_fees_sol
            + breakdown.priority_fees_sol
            + breakdown.slippage_cost_sol;
        
        // 8. Calcular porcentaje del trade
        breakdown.percentage_of_trade = if amount_sol > 0.0 {
            (breakdown.total_sol / amount_sol) * 100.0
        } else {
            0.0
        };
        
        // 9. Verificar límites máximos
        if breakdown.percentage_of_trade > self.config.max_fee_percentage {
            return Err(CoreError::Validation(format!(
                "Fee percentage ({:.2}%) exceeds maximum allowed ({:.2}%)",
                breakdown.percentage_of_trade, self.config.max_fee_percentage
            )));
        }
        
        debug!("💰 Fees calculados: {:.6} SOL ({:.2}%)", 
               breakdown.total_sol, breakdown.percentage_of_trade);
        
        Ok(breakdown)
    }
    
    /// Calcular fees de DEX
    fn calculate_dex_fees(&self, amount_sol: f64, route_info: &RouteInfo) -> CoreResult<f64> {
        let mut total_dex_fees = 0.0;
        
        for dex_name in &route_info.route_plan {
            if let Some(dex_info) = self.dex_fees.get(dex_name) {
                let dex_fee_rate = dex_info.base_fee_bps as f64 / 10000.0; // Convert bps to decimal
                let dex_fee = amount_sol * dex_fee_rate;
                total_dex_fees += dex_fee;
                
                debug!("DEX {} fee: {:.6} SOL ({:.2}%)", 
                       dex_name, dex_fee, dex_fee_rate * 100.0);
            } else {
                // Usar fee por defecto si no conocemos el DEX
                let default_fee_rate = 0.0025; // 0.25% default
                let dex_fee = amount_sol * default_fee_rate;
                total_dex_fees += dex_fee;
                
                warn!("Unknown DEX {}, using default fee: {:.6} SOL", dex_name, dex_fee);
            }
        }
        
        Ok(total_dex_fees)
    }
    
    /// Calcular fees de Jupiter
    fn calculate_jupiter_fees(&self, amount_sol: f64) -> CoreResult<f64> {
        let jupiter_fee_rate = self.config.jupiter_fee_bps as f64 / 10000.0;
        Ok(amount_sol * jupiter_fee_rate)
    }
    
    /// Calcular fees de plataforma
    fn calculate_platform_fees(&self, amount_sol: f64) -> CoreResult<f64> {
        let platform_fee_rate = self.config.platform_fee_bps as f64 / 10000.0;
        Ok(amount_sol * platform_fee_rate)
    }
    
    /// Calcular fees de gas
    fn calculate_gas_fees(&self, route_info: &RouteInfo) -> CoreResult<f64> {
        const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
        
        // Gas base por transacción
        let base_gas = route_info.estimated_gas;
        
        // Gas adicional por hop
        let gas_per_hop = 15000u64;
        let additional_gas = (route_info.hops as u64).saturating_sub(1) * gas_per_hop;
        
        let total_gas_lamports = base_gas + additional_gas;
        
        // Aplicar multiplicador si hay optimización de gas habilitada
        let optimized_gas = if self.config.gas_optimization {
            (total_gas_lamports as f64 * 0.85) as u64 // 15% reduction with optimization
        } else {
            total_gas_lamports
        };
        
        Ok(optimized_gas as f64 / LAMPORTS_PER_SOL as f64)
    }
    
    /// Calcular priority fees
    fn calculate_priority_fees(&self, route_info: &RouteInfo) -> CoreResult<f64> {
        const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
        
        // Priority fee base
        let base_priority_fee = 10000u64; // 0.00001 SOL base
        
        // Multiplicador por congestión de red
        let congestion_multiplier = if self.gas_tracker.network_congestion > 0.7 {
            self.config.priority_fee_multiplier * 1.5 // Extra multiplier for high congestion
        } else {
            self.config.priority_fee_multiplier
        };
        
        // Priority fee por hop
        let fee_per_hop = (base_priority_fee as f64 * congestion_multiplier) as u64;
        let total_priority_fee = fee_per_hop * route_info.hops as u64;
        
        Ok(total_priority_fee as f64 / LAMPORTS_PER_SOL as f64)
    }
    
    /// Estimar costo de slippage
    fn estimate_slippage_cost(&self, amount_sol: f64, route_info: &RouteInfo) -> CoreResult<f64> {
        // Slippage estimado basado en price impact y liquidez
        let base_slippage = route_info.price_impact;
        
        // Slippage adicional por múltiples hops
        let hop_slippage = (route_info.hops as f64 - 1.0) * 0.001; // 0.1% per additional hop
        
        let total_slippage = base_slippage + hop_slippage;
        let slippage_cost = amount_sol * total_slippage;
        
        Ok(slippage_cost)
    }
    
    /// Estimar fees con confianza
    pub fn estimate_fees_with_confidence(&self, amount_sol: f64, route_info: &RouteInfo) -> CoreResult<FeeEstimate> {
        let breakdown = self.calculate_total_fees(amount_sol, route_info)?;
        
        // Calcular confianza basada en datos históricos y volatilidad del gas
        let confidence = self.calculate_confidence_score(&breakdown)?;
        
        // Calcular validez (fees son válidos por poco tiempo en mercados volátiles)
        let valid_until = chrono::Utc::now() + chrono::Duration::seconds(30);
        
        // Generar recomendaciones
        let recommendations = self.generate_fee_recommendations(&breakdown, amount_sol)?;
        
        Ok(FeeEstimate {
            breakdown,
            confidence,
            valid_until,
            recommendations,
        })
    }
    
    /// Calcular score de confianza
    fn calculate_confidence_score(&self, breakdown: &FeeBreakdown) -> CoreResult<f64> {
        let mut confidence = 0.8; // Base confidence
        
        // Reducir confianza si los fees son muy altos
        if breakdown.percentage_of_trade > 1.0 { // > 1%
            confidence -= 0.2;
        }
        
        // Reducir confianza si hay alta congestión de red
        if self.gas_tracker.network_congestion > 0.8 {
            confidence -= 0.3;
        }
        
        // Aumentar confianza si tenemos datos históricos recientes
        if !self.fee_history.is_empty() {
            let recent_accuracy: f64 = self.fee_history.iter()
                .rev()
                .take(10)
                .map(|r| r.accuracy)
                .sum::<f64>() / 10.0.min(self.fee_history.len() as f64);
            
            confidence = confidence * 0.7 + recent_accuracy * 0.3;
        }
        
        Ok(confidence.max(0.0).min(1.0))
    }
    
    /// Generar recomendaciones de optimización de fees
    fn generate_fee_recommendations(&self, breakdown: &FeeBreakdown, amount_sol: f64) -> CoreResult<Vec<FeeRecommendation>> {
        let mut recommendations = Vec::new();
        
        // Recomendación 1: Reducir priority fee si es muy alto
        if breakdown.priority_fees_sol > 0.001 { // > 0.001 SOL
            recommendations.push(FeeRecommendation {
                recommendation_type: RecommendationType::ReducePriorityFee,
                description: "Consider reducing priority fee during low congestion".to_string(),
                potential_savings_sol: breakdown.priority_fees_sol * 0.5,
                implementation_difficulty: Difficulty::Easy,
            });
        }
        
        // Recomendación 2: Optimizar ruta si hay muchos hops
        if breakdown.dex_fees_sol > amount_sol * 0.005 { // > 0.5%
            recommendations.push(FeeRecommendation {
                recommendation_type: RecommendationType::OptimizeRoute,
                description: "Route optimization could reduce DEX fees".to_string(),
                potential_savings_sol: breakdown.dex_fees_sol * 0.3,
                implementation_difficulty: Difficulty::Medium,
            });
        }
        
        // Recomendación 3: Esperar gas más bajo
        if self.gas_tracker.network_congestion > 0.7 {
            recommendations.push(FeeRecommendation {
                recommendation_type: RecommendationType::WaitForLowerGas,
                description: "Network congestion is high, consider waiting".to_string(),
                potential_savings_sol: breakdown.gas_fees_sol * 0.4,
                implementation_difficulty: Difficulty::Easy,
            });
        }
        
        // Recomendación 4: Usar DEX alternativo
        if breakdown.percentage_of_trade > 1.5 {
            recommendations.push(FeeRecommendation {
                recommendation_type: RecommendationType::UseAlternativeDex,
                description: "Alternative DEX might offer better rates".to_string(),
                potential_savings_sol: breakdown.total_sol * 0.2,
                implementation_difficulty: Difficulty::Hard,
            });
        }
        
        Ok(recommendations)
    }
    
    /// Actualizar información de gas de la red
    pub fn update_gas_info(&mut self, gas_price: u64, congestion: f64, success_rate: f64) {
        let gas_record = GasRecord {
            timestamp: chrono::Utc::now(),
            gas_price_lamports: gas_price,
            network_congestion: congestion,
            transaction_success_rate: success_rate,
        };
        
        self.gas_tracker.recent_gas_prices.push(gas_record);
        
        // Mantener solo los últimos 100 registros
        if self.gas_tracker.recent_gas_prices.len() > 100 {
            self.gas_tracker.recent_gas_prices.remove(0);
        }
        
        // Actualizar estadísticas agregadas
        self.update_gas_statistics();
    }
    
    /// Actualizar estadísticas de gas
    fn update_gas_statistics(&mut self) {
        if self.gas_tracker.recent_gas_prices.is_empty() {
            return;
        }
        
        let gas_prices: Vec<u64> = self.gas_tracker.recent_gas_prices
            .iter()
            .map(|r| r.gas_price_lamports)
            .collect();
        
        self.gas_tracker.avg_gas_price = gas_prices.iter().sum::<u64>() / gas_prices.len() as u64;
        self.gas_tracker.peak_gas_price = *gas_prices.iter().max().unwrap_or(&0);
        self.gas_tracker.low_gas_price = *gas_prices.iter().min().unwrap_or(&u64::MAX);
        
        // Calcular congestión promedio
        let avg_congestion: f64 = self.gas_tracker.recent_gas_prices
            .iter()
            .map(|r| r.network_congestion)
            .sum::<f64>() / self.gas_tracker.recent_gas_prices.len() as f64;
        
        self.gas_tracker.network_congestion = avg_congestion;
    }
    
    /// Registrar fee real pagado para mejorar precisión
    pub fn record_actual_fee(&mut self, transaction_id: String, breakdown: FeeBreakdown, actual_cost_sol: f64) {
        let estimated_cost = breakdown.total_sol;
        let accuracy = if estimated_cost > 0.0 {
            1.0 - (estimated_cost - actual_cost_sol).abs() / estimated_cost
        } else {
            0.0
        };
        
        let fee_record = FeeRecord {
            transaction_id,
            timestamp: chrono::Utc::now(),
            breakdown,
            actual_cost_sol,
            estimated_cost_sol: estimated_cost,
            accuracy: accuracy.max(0.0).min(1.0),
        };
        
        self.fee_history.push(fee_record);
        
        // Mantener solo los últimos 1000 registros
        if self.fee_history.len() > 1000 {
            self.fee_history.remove(0);
        }
    }
    
    /// Obtener estadísticas de precisión
    pub fn get_accuracy_stats(&self) -> (f64, f64, usize) {
        if self.fee_history.is_empty() {
            return (0.0, 0.0, 0);
        }
        
        let accuracies: Vec<f64> = self.fee_history.iter().map(|r| r.accuracy).collect();
        let avg_accuracy = accuracies.iter().sum::<f64>() / accuracies.len() as f64;
        let recent_accuracy = if accuracies.len() > 10 {
            accuracies.iter().rev().take(10).sum::<f64>() / 10.0
        } else {
            avg_accuracy
        };
        
        (avg_accuracy, recent_accuracy, self.fee_history.len())
    }
}

impl Default for FeesConfig {
    fn default() -> Self {
        Self {
            platform_fee_bps: 0,
            jupiter_fee_bps: 0,
            priority_fee_multiplier: 1.5,
            gas_optimization: true,
            dynamic_fees: true,
            max_fee_percentage: 5.0, // 5% maximum
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RouteInfo;
    
    #[test]
    fn test_fee_calculation() {
        let config = FeesConfig::default();
        let calculator = FeeCalculator::new(&config);
        
        let route_info = RouteInfo {
            hops: 2,
            route_plan: vec!["Raydium".to_string(), "Orca".to_string()],
            estimated_gas: 50000,
            price_impact: 0.001, // 0.1%
        };
        
        let result = calculator.calculate_total_fees(1.0, &route_info);
        assert!(result.is_ok());
        
        let breakdown = result.unwrap();
        assert!(breakdown.total_sol > 0.0);
        assert!(breakdown.percentage_of_trade < 5.0); // Should be under 5%
    }
}
