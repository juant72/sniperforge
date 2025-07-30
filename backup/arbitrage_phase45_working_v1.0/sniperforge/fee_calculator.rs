// ===== FEE CALCULATOR MODULE =====
// Simplified fee calculation for arbitrage operations

use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeBreakdown {
    pub jupiter_fee: f64,
    pub slippage_fee: f64,
    pub network_fee: f64,
    pub total_fee: f64,
}

#[derive(Debug, Clone)]
pub struct FeeCalculator {
    jupiter_fee_bps: u16,  // Jupiter fee in basis points
    network_fee_sol: f64,  // Network fee in SOL
    enabled: bool,
}

impl FeeCalculator {
    pub fn new() -> Self {
        Self {
            jupiter_fee_bps: 25,    // 0.25% Jupiter fee
            network_fee_sol: 0.001, // ~0.001 SOL network fee
            enabled: true,
        }
    }
    
    pub fn calculate_total_fees(&self, amount_sol: f64, slippage_bps: u16) -> Result<FeeBreakdown> {
        if !self.enabled {
            return Ok(FeeBreakdown {
                jupiter_fee: 0.0,
                slippage_fee: 0.0,
                network_fee: 0.0,
                total_fee: 0.0,
            });
        }
        
        let jupiter_fee = amount_sol * (self.jupiter_fee_bps as f64 / 10000.0);
        let slippage_fee = amount_sol * (slippage_bps as f64 / 10000.0);
        let network_fee = self.network_fee_sol;
        let total_fee = jupiter_fee + slippage_fee + network_fee;
        
        Ok(FeeBreakdown {
            jupiter_fee,
            slippage_fee,
            network_fee,
            total_fee,
        })
    }
    
    pub fn calculate_flashbots_optimal_size(
        &self,
        _base_amount: f64,
        _price_difference: f64,
        _liquidity_a: f64,
        _liquidity_b: f64,
    ) -> Result<f64> {
        // Simplified Flashbots optimal sizing
        // In production, this would use complex mathematical optimization
        let optimal_multiplier = 1.2; // 20% increase from base
        Ok(_base_amount * optimal_multiplier)
    }
    
    pub fn estimate_profit_after_fees(&self, amount_sol: f64, profit_percentage: f64, slippage_bps: u16) -> Result<f64> {
        let gross_profit = amount_sol * (profit_percentage / 100.0);
        let fees = self.calculate_total_fees(amount_sol, slippage_bps)?;
        let net_profit = gross_profit - fees.total_fee;
        Ok(net_profit)
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
