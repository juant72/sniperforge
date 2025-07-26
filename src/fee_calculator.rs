// ================================================================================
// FEE CALCULATOR - PRINCIPIO 26: CÁLCULO PRECISO DE FEES TOTALES + OPTIMAL SIZING
// ================================================================================
// Sistema para calcular EXACTAMENTE todos los costos involucrados en arbitraje
// Crítico para determinar rentabilidad real de trades con montos óptimos
// ================================================================================

use anyhow::Result;
use serde::{Deserialize, Serialize};
use log::{info, warn, debug};
use crate::optimal_trading_config::OptimalTradingConfig;

/// Breakdown completo de todos los fees involucrados en un arbitrage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageFeeBreakdown {
    // Jupiter fees
    pub jupiter_swap_fee_bps: u16,      // Basis points (25-50 typical)
    pub jupiter_fee_sol: f64,           // Fee en SOL
    
    // Solana network fees
    pub solana_base_fee: f64,           // ~0.000005 SOL por transaction
    pub solana_priority_fee: f64,       // Fee adicional para prioridad
    pub total_solana_fees: f64,         // Suma de todas las TX fees
    
    // DEX-specific fees
    pub dex_a_fee_bps: u16,            // DEX origen fee in bps
    pub dex_b_fee_bps: u16,            // DEX destino fee in bps
    pub dex_fees_sol: f64,             // Total DEX fees en SOL
    
    // Slippage costs
    pub estimated_slippage_pct: f64,   // Slippage estimado %
    pub slippage_cost_sol: f64,        // Costo del slippage en SOL
    
    // MEV protection (opcional)
    pub mev_protection_fee: f64,       // Fee anti-MEV si se usa
    
    // TOTAL COSTS
    pub total_fees_sol: f64,           // Suma de TODOS los fees
    pub total_fees_usd: f64,           // Fees en USD para referencia
    
    // Profit analysis
    pub gross_profit_sol: f64,         // Profit antes de fees
    pub net_profit_sol: f64,           // Profit después de TODOS los fees
    pub profit_margin_pct: f64,        // Margen real de profit
    pub is_profitable: bool,           // true solo si net profit > 0
}

/// Resultado del cálculo óptimo de Flashbots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashbotsOptimalResult {
    pub optimal_amount_sol: f64,        // Monto óptimo a usar
    pub theoretical_optimal: f64,       // Óptimo teórico sin límites de capital
    pub expected_gross_profit: f64,     // Profit esperado antes de fees
    pub capital_efficiency: f64,        // % de eficiencia vs óptimo teórico
    pub is_capital_limited: bool,       // true si limitado por capital disponible
    pub profit_per_sol: f64,           // ROI por SOL invertido
}

impl FlashbotsOptimalResult {
    pub fn no_opportunity() -> Self {
        Self {
            optimal_amount_sol: 0.0,
            theoretical_optimal: 0.0,
            expected_gross_profit: 0.0,
            capital_efficiency: 0.0,
            is_capital_limited: false,
            profit_per_sol: 0.0,
        }
    }
}

/// Configuración de fees por DEX (actualizable vía JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DEXFeeConfig {
    pub dex_name: String,
    pub swap_fee_bps: u16,             // Fee base en basis points
    pub minimum_fee_sol: f64,          // Fee mínimo en SOL
    pub liquidity_dependent: bool,     // Si el fee depende de liquidez
}

/// Calculator principal de fees para arbitrage
pub struct FeeCalculator {
    dex_configs: Vec<DEXFeeConfig>,
    sol_price_usd: f64,                // Precio SOL para conversiones
    base_solana_fee: f64,              // Fee base de Solana (~0.000005)
}

impl FeeCalculator {
    /// Crear nuevo calculator con configuración por defecto
    pub fn new() -> Self {
        Self {
            dex_configs: Self::default_dex_configs(),
            sol_price_usd: 185.0,  // Precio aproximado, debe actualizarse
            base_solana_fee: 0.000005,
        }
    }
    
    /// Calcular breakdown completo de fees para un arbitrage
    pub async fn calculate_arbitrage_fees(
        &self,
        trade_amount_sol: f64,
        dex_a_name: &str,
        dex_b_name: &str,
        gross_profit_sol: f64,
        liquidity_a: f64,
        liquidity_b: f64,
    ) -> Result<ArbitrageFeeBreakdown> {
        
        info!("🧮 Calculando fees completos para arbitrage de {} SOL", trade_amount_sol);
        
        // 1. JUPITER FEES (variable según pool)
        let jupiter_fee_bps = self.estimate_jupiter_fee_bps(trade_amount_sol, liquidity_a.min(liquidity_b)).await?;
        let jupiter_fee_sol = trade_amount_sol * (jupiter_fee_bps as f64 / 10000.0);
        
        // 2. SOLANA NETWORK FEES (multiple transactions)
        let num_transactions = 2; // Típicamente 2 swaps
        let priority_multiplier = 1.5; // Factor para priority fees
        let solana_base_fee = self.base_solana_fee * num_transactions as f64;
        let solana_priority_fee = solana_base_fee * (priority_multiplier - 1.0);
        let total_solana_fees = solana_base_fee + solana_priority_fee;
        
        // 3. DEX-SPECIFIC FEES
        let dex_a_fee_bps = self.get_dex_fee_bps(dex_a_name);
        let dex_b_fee_bps = self.get_dex_fee_bps(dex_b_name);
        let dex_fees_sol = trade_amount_sol * ((dex_a_fee_bps + dex_b_fee_bps) as f64 / 10000.0);
        
        // 4. SLIPPAGE ESTIMATION (crítico para trades grandes)
        let estimated_slippage_pct = self.estimate_slippage_percentage(trade_amount_sol, liquidity_a.min(liquidity_b));
        let slippage_cost_sol = trade_amount_sol * (estimated_slippage_pct / 100.0);
        
        // 5. MEV PROTECTION (opcional pero recomendado)
        let mev_protection_fee = if trade_amount_sol > 0.1 {
            trade_amount_sol * 0.0005 // 0.05% para trades grandes
        } else {
            0.0
        };
        
        // 6. TOTAL CALCULATION
        let total_fees_sol = jupiter_fee_sol + total_solana_fees + dex_fees_sol + slippage_cost_sol + mev_protection_fee;
        let total_fees_usd = total_fees_sol * self.sol_price_usd;
        
        // 7. PROFIT ANALYSIS
        let net_profit_sol = gross_profit_sol - total_fees_sol;
        let profit_margin_pct = if trade_amount_sol > 0.0 {
            (net_profit_sol / trade_amount_sol) * 100.0
        } else {
            0.0
        };
        let is_profitable = net_profit_sol > 0.0;
        
        // Log del breakdown para debugging
        if is_profitable {
            info!("✅ Arbitrage RENTABLE:");
        } else {
            warn!("❌ Arbitrage NO rentable:");
        }
        info!("   💰 Gross Profit: {:.6} SOL", gross_profit_sol);
        info!("   🏦 Jupiter Fee: {:.6} SOL ({} bps)", jupiter_fee_sol, jupiter_fee_bps);
        info!("   ⛓️ Solana Fees: {:.6} SOL", total_solana_fees);
        info!("   🏪 DEX Fees: {:.6} SOL", dex_fees_sol);
        info!("   📉 Slippage: {:.6} SOL ({:.2}%)", slippage_cost_sol, estimated_slippage_pct);
        info!("   🛡️ MEV Protection: {:.6} SOL", mev_protection_fee);
        info!("   💸 TOTAL FEES: {:.6} SOL (${:.2})", total_fees_sol, total_fees_usd);
        info!("   💎 NET PROFIT: {:.6} SOL ({:.2}%)", net_profit_sol, profit_margin_pct);
        
        Ok(ArbitrageFeeBreakdown {
            jupiter_swap_fee_bps: jupiter_fee_bps,
            jupiter_fee_sol,
            solana_base_fee,
            solana_priority_fee,
            total_solana_fees,
            dex_a_fee_bps,
            dex_b_fee_bps,
            dex_fees_sol,
            estimated_slippage_pct,
            slippage_cost_sol,
            mev_protection_fee,
            total_fees_sol,
            total_fees_usd,
            gross_profit_sol,
            net_profit_sol,
            profit_margin_pct,
            is_profitable,
        })
    }
    
    /// Estimar fee de Jupiter basado en cantidad y liquidez
    async fn estimate_jupiter_fee_bps(&self, amount_sol: f64, liquidity_usd: f64) -> Result<u16> {
        // Jupiter fees variables según liquidez y pool
        let fee_bps = if liquidity_usd > 1_000_000.0 {
            25 // 0.25% para pools con alta liquidez
        } else if liquidity_usd > 100_000.0 {
            30 // 0.30% para pools mediana liquidez
        } else if liquidity_usd > 10_000.0 {
            40 // 0.40% para pools baja liquidez
        } else {
            50 // 0.50% para pools muy baja liquidez
        };
        
        // Ajustar por tamaño del trade
        let adjusted_fee = if amount_sol > 1.0 {
            fee_bps + 5 // Penalty por trades grandes
        } else {
            fee_bps
        };
        
        Ok(adjusted_fee)
    }
    
    /// Obtener fee de DEX específico
    fn get_dex_fee_bps(&self, dex_name: &str) -> u16 {
        self.dex_configs
            .iter()
            .find(|config| config.dex_name.to_lowercase() == dex_name.to_lowercase())
            .map(|config| config.swap_fee_bps)
            .unwrap_or(30) // Default 0.30% si no se encuentra
    }
    
    /// Estimar slippage basado en tamaño del trade vs liquidez
    fn estimate_slippage_percentage(&self, trade_amount_sol: f64, liquidity_usd: f64) -> f64 {
        let trade_amount_usd = trade_amount_sol * self.sol_price_usd;
        let trade_to_liquidity_ratio = trade_amount_usd / liquidity_usd.max(1000.0);
        
        // Slippage increases exponentially with trade size
        if trade_to_liquidity_ratio > 0.1 {
            2.0 // 2% slippage para trades muy grandes
        } else if trade_to_liquidity_ratio > 0.05 {
            1.0 // 1% slippage para trades grandes  
        } else if trade_to_liquidity_ratio > 0.01 {
            0.5 // 0.5% slippage para trades medianos
        } else {
            0.1 // 0.1% slippage para trades pequeños
        }
    }
    
    /// Configuración por defecto de fees por DEX
    fn default_dex_configs() -> Vec<DEXFeeConfig> {
        vec![
            DEXFeeConfig {
                dex_name: "Raydium".to_string(),
                swap_fee_bps: 25,
                minimum_fee_sol: 0.000001,
                liquidity_dependent: true,
            },
            DEXFeeConfig {
                dex_name: "Orca".to_string(),
                swap_fee_bps: 30,
                minimum_fee_sol: 0.000001,
                liquidity_dependent: true,
            },
            DEXFeeConfig {
                dex_name: "Whirlpool".to_string(),
                swap_fee_bps: 10,
                minimum_fee_sol: 0.000001,
                liquidity_dependent: true,
            },
            DEXFeeConfig {
                dex_name: "Serum".to_string(),
                swap_fee_bps: 25,
                minimum_fee_sol: 0.000005,
                liquidity_dependent: false,
            },
            DEXFeeConfig {
                dex_name: "Jupiter".to_string(),
                swap_fee_bps: 25,
                minimum_fee_sol: 0.000001,
                liquidity_dependent: true,
            },
        ]
    }
    
    /// Actualizar precio de SOL para cálculos USD
    pub fn update_sol_price(&mut self, new_price_usd: f64) {
        self.sol_price_usd = new_price_usd;
    }
    
    /// FLASHBOTS OPTIMAL TRADE SIZE CALCULATION
    /// Basado en investigación de Flashbots - simple-blind-arbitrage
    /// Calcula el tamaño óptimo de trade para maximizar profit bruto
    pub fn calculate_flashbots_optimal_size(
        &self,
        reserves_a: (f64, f64), // (reserve_in, reserve_out) exchange A
        reserves_b: (f64, f64), // (reserve_in, reserve_out) exchange B  
        available_capital_sol: f64,
        max_capital_ratio: f64, // Máximo % del capital a usar (ej: 0.8 = 80%)
    ) -> Result<FlashbotsOptimalResult> {
        let (r_i_a, r_o_a) = reserves_a;
        let (r_i_b, r_o_b) = reserves_b;
        
        // Constantes de Uniswap V2 (fee structure)
        let fee_constant = 997.0;
        let fee_divisor = 1000.0;
        
        info!("🎯 FLASHBOTS OPTIMAL CALCULATION:");
        info!("   Exchange A reserves: {:.6} / {:.6}", r_i_a, r_o_a);
        info!("   Exchange B reserves: {:.6} / {:.6}", r_i_b, r_o_b);
        info!("   Available capital: {:.6} SOL", available_capital_sol);
        
        // Verificar que tenemos liquidez suficiente
        if r_i_a <= 0.0 || r_o_a <= 0.0 || r_i_b <= 0.0 || r_o_b <= 0.0 {
            warn!("❌ Invalid reserves - cannot calculate optimal size");
            return Ok(FlashbotsOptimalResult::no_opportunity());
        }
        
        // Fórmula de Flashbots para tamaño óptimo:
        // optimal = (sqrt(F² * R_o_A * R_o_B / (R_i_B * R_i_A)) - D) * 
        //           (R_i_B * R_i_A * D) / ((F * R_i_B * D) + (F² * R_o_A))
        
        let numerator_sqrt = (fee_constant * fee_constant * r_o_a * r_o_b) / (r_i_b * r_i_a);
        
        if numerator_sqrt <= 0.0 {
            warn!("❌ Negative sqrt input - no arbitrage opportunity");
            return Ok(FlashbotsOptimalResult::no_opportunity());
        }
        
        let sqrt_result = numerator_sqrt.sqrt();
        let numerator = (sqrt_result - fee_divisor) * (r_i_b * r_i_a * fee_divisor);
        let denominator = (fee_constant * r_i_b * fee_divisor) + (fee_constant * fee_constant * r_o_a);
        
        if denominator <= 0.0 {
            warn!("❌ Invalid denominator - cannot calculate optimal size");
            return Ok(FlashbotsOptimalResult::no_opportunity());
        }
        
        let optimal_amount_raw = numerator / denominator;
        
        info!("   📊 Raw optimal amount: {:.6} SOL", optimal_amount_raw);
        
        // Aplicar límites de capital
        let max_allowed = available_capital_sol * max_capital_ratio;
        let optimal_amount = optimal_amount_raw.min(max_allowed).max(0.001); // Mínimo 0.001 SOL
        
        // Calcular profit esperado con este tamaño
        let gross_profit = self.calculate_gross_profit_for_amount(
            optimal_amount, 
            reserves_a, 
            reserves_b
        )?;
        
        // Calcular eficiencia (qué tan cerca estamos del óptimo teórico)
        let efficiency = if optimal_amount_raw > 0.0 {
            (optimal_amount / optimal_amount_raw).min(1.0)
        } else {
            0.0
        };
        
        info!("   ✅ Final optimal amount: {:.6} SOL (${:.2})", 
              optimal_amount, optimal_amount * self.sol_price_usd);
        info!("   📈 Expected gross profit: {:.6} SOL", gross_profit);
        info!("   🎯 Capital efficiency: {:.1}%", efficiency * 100.0);
        
        Ok(FlashbotsOptimalResult {
            optimal_amount_sol: optimal_amount,
            theoretical_optimal: optimal_amount_raw,
            expected_gross_profit: gross_profit,
            capital_efficiency: efficiency,
            is_capital_limited: optimal_amount_raw > max_allowed,
            profit_per_sol: if optimal_amount > 0.0 { gross_profit / optimal_amount } else { 0.0 },
        })
    }
    
    /// Calcula el profit bruto para un monto específico usando las reservas
    pub fn calculate_gross_profit_for_amount(
        &self,
        amount_in: f64,
        reserves_a: (f64, f64),
        reserves_b: (f64, f64)
    ) -> Result<f64> {
        let (r_i_a, r_o_a) = reserves_a;
        let (r_i_b, r_o_b) = reserves_b;
        
        // Simular el trade A->B->A
        let amount_out_a = self.calculate_uniswap_output(amount_in, r_i_a, r_o_a);
        let amount_out_b = self.calculate_uniswap_output(amount_out_a, r_i_b, r_o_b);
        
        let gross_profit = amount_out_b - amount_in;
        Ok(gross_profit.max(0.0))
    }
    
    /// Calcula output de Uniswap V2 dado input y reservas
    fn calculate_uniswap_output(&self, amount_in: f64, reserve_in: f64, reserve_out: f64) -> f64 {
        if reserve_in <= 0.0 || reserve_out <= 0.0 || amount_in <= 0.0 {
            return 0.0;
        }
        
        let amount_in_with_fee = amount_in * 997.0; // 0.3% fee
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = (reserve_in * 1000.0) + amount_in_with_fee;
        
        numerator / denominator
    }

    /// NUEVO: Calcular monto óptimo de trade para maximizar profit
    pub fn calculate_optimal_trade_amount(
        &self,
        gross_profit_percentage: f64,
        available_liquidity_usd: f64,
        config: &OptimalTradingConfig,
    ) -> Result<f64> {
        // 1. Calcular fees fijos típicos (independientes del monto)
        let fixed_fees_sol = self.calculate_fixed_fees();
        
        // 2. Calcular monto mínimo para ser rentable
        let min_trade_for_profit = if gross_profit_percentage > 0.0 {
            let required_gross_profit = fixed_fees_sol + config.opportunity_filters.min_net_profit_sol;
            required_gross_profit / (gross_profit_percentage / 100.0)
        } else {
            config.min_trade_amounts.sol_minimum
        };
        
        // 3. Considerar liquidez disponible
        let max_by_liquidity = (available_liquidity_usd / self.sol_price_usd) * 0.1; // Max 10% de liquidez
        
        // 4. Aplicar límites del config
        let optimal_amount = min_trade_for_profit
            .max(config.min_trade_amounts.sol_minimum)
            .min(config.dynamic_sizing.max_trade_size_sol)
            .min(max_by_liquidity);
        
        info!("🎯 OPTIMAL TRADE CALCULATION:");
        info!("   📊 Gross profit: {:.3}%", gross_profit_percentage);
        info!("   🔧 Fixed fees: {:.6} SOL", fixed_fees_sol);
        info!("   💰 Min for profit: {:.6} SOL", min_trade_for_profit);
        info!("   🌊 Max by liquidity: {:.6} SOL", max_by_liquidity);
        info!("   ✅ OPTIMAL AMOUNT: {:.6} SOL (${:.2})", optimal_amount, optimal_amount * self.sol_price_usd);
        
        Ok(optimal_amount)
    }
    
    /// Calcular fees fijos (Solana + base Jupiter)
    fn calculate_fixed_fees(&self) -> f64 {
        let solana_base_fee = 0.000005; // Base TX fee
        let solana_priority_fee = 0.00001; // Typical priority fee
        let jupiter_base_fee = 0.000003; // Minimum Jupiter fee
        
        solana_base_fee + solana_priority_fee + jupiter_base_fee
    }

    /// NUEVO: Verificar si un trade es rentable con el monto propuesto
    pub fn is_trade_profitable_at_amount(
        &self,
        trade_amount_sol: f64,
        gross_profit_percentage: f64,
        config: &OptimalTradingConfig,
    ) -> Result<bool> {
        let gross_profit_sol = trade_amount_sol * (gross_profit_percentage / 100.0);
        
        // Calcular fees rápido (estimado)
        let estimated_total_fees = self.estimate_total_fees(trade_amount_sol);
        let net_profit_sol = gross_profit_sol - estimated_total_fees;
        
        Ok(config.is_opportunity_profitable(gross_profit_sol, estimated_total_fees, trade_amount_sol))
    }
    
    /// Estimación rápida de fees totales
    fn estimate_total_fees(&self, trade_amount_sol: f64) -> f64 {
        let jupiter_fee = trade_amount_sol * 0.003; // 0.3% estimado
        let solana_fees = 0.000015; // Fees fijos típicos
        let dex_fees = trade_amount_sol * 0.006; // 0.6% estimado
        let slippage = trade_amount_sol * 0.001; // 0.1% estimado
        
        jupiter_fee + solana_fees + dex_fees + slippage
    }
}

impl Default for FeeCalculator {
    fn default() -> Self {
        Self::new()
    }
}
