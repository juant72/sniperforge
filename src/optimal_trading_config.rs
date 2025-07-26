// CONFIGURACIÓN OPTIMIZADA PARA GANAR DINERO REAL
// Basado en análisis de fees del sistema actual

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptimalTradingConfig {
    pub min_trade_amounts: MinTradeAmounts,
    pub fee_thresholds: FeeThresholds,
    pub profit_calculations: ProfitCalculations,
    pub opportunity_filters: OpportunityFilters,
    pub dynamic_sizing: DynamicSizing,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MinTradeAmounts {
    // Montos mínimos para cubrir fees fijos
    pub sol_minimum: f64,        // 0.01 SOL mínimo para cubrir fees
    pub usdc_minimum: f64,       // Equivalente en USDC
    pub token_minimum_usd: f64,  // $2 USD mínimo para cualquier token
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeeThresholds {
    // Fees máximos aceptables como % del trade
    pub max_jupiter_fee_percent: f64,    // 0.5% máximo
    pub max_solana_fee_percent: f64,     // 0.1% máximo
    pub max_dex_fee_percent: f64,        // 0.6% máximo
    pub max_total_fee_percent: f64,      // 1.5% total máximo
    pub max_slippage_percent: f64,       // 0.5% slippage máximo
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfitCalculations {
    // Configuración para cálculos de profit real
    pub min_net_profit_bps: u64,        // 25 bps = 0.25% profit neto mínimo
    pub safety_margin_bps: u64,         // 10 bps safety margin adicional
    pub fee_calculation_mode: String,    // "conservative" | "aggressive" | "dynamic"
    pub include_mev_costs: bool,         // Incluir costos MEV en cálculos
    pub profit_target_scaling: bool,     // Escalar target según market volatility
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpportunityFilters {
    // Filtros inteligentes para oportunidades rentables
    pub min_net_profit_sol: f64,        // 0.00025 SOL profit neto mínimo
    pub min_confidence_score: f64,       // 0.3 confidence mínima
    pub max_execution_risk: f64,         // 0.2 riesgo máximo de ejecución
    pub liquidity_depth_check: bool,    // Verificar liquidez suficiente
    pub slippage_impact_check: bool,     // Verificar impacto de slippage
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicSizing {
    // Sizing dinámico basado en oportunidad
    pub base_trade_size_sol: f64,       // 0.01 SOL base
    pub max_trade_size_sol: f64,        // 0.5 SOL máximo
    pub profit_multiplier: f64,         // Aumentar size si profit > threshold
    pub risk_reduction_factor: f64,     // Reducir size si riesgo alto
    pub liquidity_based_sizing: bool,   // Ajustar según liquidez disponible
}

impl Default for OptimalTradingConfig {
    fn default() -> Self {
        Self {
            min_trade_amounts: MinTradeAmounts {
                sol_minimum: 0.01,           // 0.01 SOL = ~$1.85 (suficiente para fees)
                usdc_minimum: 2.0,           // $2 USDC mínimo
                token_minimum_usd: 2.0,      // $2 USD mínimo
            },
            fee_thresholds: FeeThresholds {
                max_jupiter_fee_percent: 0.5,    // 0.5% Jupiter máximo
                max_solana_fee_percent: 0.1,     // 0.1% Solana fees máximo
                max_dex_fee_percent: 0.6,        // 0.6% DEX fees máximo
                max_total_fee_percent: 1.5,      // 1.5% total fees máximo
                max_slippage_percent: 0.5,       // 0.5% slippage máximo
            },
            profit_calculations: ProfitCalculations {
                min_net_profit_bps: 25,          // 0.25% profit neto mínimo
                safety_margin_bps: 10,           // 0.10% safety margin
                fee_calculation_mode: "conservative".to_string(),
                include_mev_costs: true,
                profit_target_scaling: true,
            },
            opportunity_filters: OpportunityFilters {
                min_net_profit_sol: 0.00025,     // 0.00025 SOL = ~$0.046 profit mínimo
                min_confidence_score: 0.3,       // 30% confidence mínima
                max_execution_risk: 0.2,         // 20% riesgo máximo
                liquidity_depth_check: true,
                slippage_impact_check: true,
            },
            dynamic_sizing: DynamicSizing {
                base_trade_size_sol: 0.01,       // Start con 0.01 SOL
                max_trade_size_sol: 0.5,         // Máximo 0.5 SOL
                profit_multiplier: 2.0,          // 2x size si profit > 0.5%
                risk_reduction_factor: 0.5,      // 0.5x size si riesgo alto
                liquidity_based_sizing: true,
            },
        }
    }
}

impl OptimalTradingConfig {
    pub fn load_from_json() -> Result<Self, Box<dyn std::error::Error>> {
        // Intentar cargar desde archivo JSON, usar default si no existe
        match std::fs::read_to_string("optimal_trading_config.json") {
            Ok(content) => {
                let config: OptimalTradingConfig = serde_json::from_str(&content)?;
                Ok(config)
            }
            Err(_) => {
                // Crear archivo default si no existe
                let default_config = Self::default();
                let json = serde_json::to_string_pretty(&default_config)?;
                std::fs::write("optimal_trading_config.json", json)?;
                Ok(default_config)
            }
        }
    }

    pub fn calculate_optimal_trade_size(&self, opportunity_profit_percent: f64, available_liquidity_sol: f64) -> f64 {
        let mut size = self.dynamic_sizing.base_trade_size_sol;
        
        // Aumentar size si profit es alto
        if opportunity_profit_percent > 0.5 {
            size *= self.dynamic_sizing.profit_multiplier;
        }
        
        // Limitar por liquidez disponible
        if self.dynamic_sizing.liquidity_based_sizing {
            size = size.min(available_liquidity_sol * 0.1); // Máximo 10% de liquidez
        }
        
        // Aplicar límites
        size = size.max(self.min_trade_amounts.sol_minimum);
        size = size.min(self.dynamic_sizing.max_trade_size_sol);
        
        size
    }

    pub fn is_opportunity_profitable(&self, gross_profit_sol: f64, total_fees_sol: f64, trade_size_sol: f64) -> bool {
        let net_profit_sol = gross_profit_sol - total_fees_sol;
        let net_profit_percent = (net_profit_sol / trade_size_sol) * 100.0;
        let min_net_profit_percent = self.profit_calculations.min_net_profit_bps as f64 / 100.0;
        
        // Debe tener profit neto mínimo Y profit absoluto mínimo
        net_profit_sol >= self.opportunity_filters.min_net_profit_sol 
            && net_profit_percent >= min_net_profit_percent
    }

    pub fn calculate_recommended_trade_amount(&self, gross_profit_percent: f64) -> f64 {
        // Calcular monto que hace rentable el trade considerando fees
        let estimated_fees_sol = 0.000024; // Fees típicos observados
        let min_gross_profit_sol = estimated_fees_sol + self.opportunity_filters.min_net_profit_sol;
        
        // Si gross profit % conocido, calcular monto necesario
        if gross_profit_percent > 0.0 {
            let required_trade_size = min_gross_profit_sol / (gross_profit_percent / 100.0);
            return required_trade_size.max(self.min_trade_amounts.sol_minimum);
        }
        
        self.dynamic_sizing.base_trade_size_sol
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profitability_calculation() {
        let config = OptimalTradingConfig::default();
        
        // Caso 1: Trade NO rentable (caso actual del sistema)
        let gross_profit = 0.000002; // 0.000002 SOL
        let total_fees = 0.000024;   // 0.000024 SOL
        let trade_size = 0.001;      // 0.001 SOL
        
        assert!(!config.is_opportunity_profitable(gross_profit, total_fees, trade_size));
        
        // Caso 2: Trade RENTABLE con monto mayor
        let gross_profit = 0.0001;   // 0.0001 SOL
        let total_fees = 0.000024;   // 0.000024 SOL  
        let trade_size = 0.01;       // 0.01 SOL
        
        assert!(config.is_opportunity_profitable(gross_profit, total_fees, trade_size));
    }

    #[test]
    fn test_optimal_trade_size_calculation() {
        let config = OptimalTradingConfig::default();
        
        // Oportunidad con 0.5% profit
        let size = config.calculate_optimal_trade_size(0.5, 1.0);
        assert!(size >= config.min_trade_amounts.sol_minimum);
        assert!(size <= config.dynamic_sizing.max_trade_size_sol);
    }

    #[test]
    fn test_recommended_trade_amount() {
        let config = OptimalTradingConfig::default();
        
        // Para 0.23% profit (caso real detectado), calcular monto necesario
        let amount = config.calculate_recommended_trade_amount(0.23);
        println!("Para 0.23% profit, monto recomendado: {} SOL", amount);
        
        // Debería ser mayor que 0.001 SOL para ser rentable
        assert!(amount > 0.01);
    }
}
