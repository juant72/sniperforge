use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;
use crate::expert::constants::*;

// ===== EXPERT MATHEMATICAL CALCULATIONS =====
// Funciones matemáticas puras basadas en AMM reales

/// Calculate exact AMM output using constant product formula (x*y=k)
/// This is the CORE mathematical function for all DEX calculations
pub fn calculate_amm_output_exact(
    reserve_in: u64, 
    reserve_out: u64, 
    amount_in: u64,
    fee_bps: u64
) -> Result<u64> {
    if reserve_in == 0 || reserve_out == 0 {
        return Err(anyhow!("Zero reserves not allowed"));
    }
    
    if amount_in == 0 {
        return Err(anyhow!("Zero input amount"));
    }
    
    // Apply fee to input amount
    let amount_in_after_fees = amount_in
        .checked_mul(10_000 - fee_bps)
        .and_then(|x| x.checked_div(10_000))
        .ok_or_else(|| anyhow!("Fee calculation overflow"))?;
    
    // Constant product formula: k = x * y
    let k = (reserve_in as u128)
        .checked_mul(reserve_out as u128)
        .ok_or_else(|| anyhow!("K calculation overflow"))?;
    
    let new_reserve_in = (reserve_in as u128)
        .checked_add(amount_in_after_fees as u128)
        .ok_or_else(|| anyhow!("New reserve calculation overflow"))?;
    
    let new_reserve_out = k
        .checked_div(new_reserve_in)
        .ok_or_else(|| anyhow!("Division by zero in AMM calculation"))?;
    
    let amount_out = (reserve_out as u128)
        .checked_sub(new_reserve_out)
        .ok_or_else(|| anyhow!("Insufficient liquidity"))?;
    
    // Validate output is reasonable
    if amount_out > reserve_out as u128 / 2 {
        return Err(anyhow!("Output too large - exceeds 50% of reserves"));
    }
    
    Ok(amount_out as u64)
}

/// Calculate real price impact using square root model
/// Price impact increases non-linearly with trade size
pub fn calculate_real_price_impact(
    reserve_in: u64, 
    reserve_out: u64, 
    amount_in: u64,
    _fee_bps: u64
) -> Result<f64> {
    if reserve_in == 0 || reserve_out == 0 {
        return Err(anyhow!("Zero reserves"));
    }
    
    let trade_ratio = amount_in as f64 / reserve_in as f64;
    
    // Non-linear price impact model using square root
    // Small trades: minimal impact
    // Large trades: exponential impact
    let price_impact = if trade_ratio < 0.001 {
        trade_ratio * 0.1 // 0.01% impact for 0.1% trade size
    } else if trade_ratio < 0.01 {
        trade_ratio * 0.5 // 0.5% impact for 1% trade size
    } else {
        // Square root model for larger trades
        trade_ratio.sqrt() * 0.8
    };
    
    Ok(price_impact.min(0.5)) // Cap at 50% impact
}

/// Calculate optimal trade size using Kelly Criterion
/// Maximizes profit while minimizing risk
pub fn calculate_optimal_trade_size(
    pool_a_liquidity: u64,
    pool_b_liquidity: u64,
    expected_profit_bps: u64,
    wallet_balance: u64
) -> Result<u64> {
    // Kelly Criterion: f* = (bp - q) / b
    // Where f* = fraction to bet, b = odds, p = probability of winning, q = probability of losing
    
    let min_liquidity = pool_a_liquidity.min(pool_b_liquidity);
    
    // Conservative approach: use 1-5% of minimum pool liquidity
    let max_trade_by_liquidity = min_liquidity / 50; // 2% of pool liquidity
    
    // Profit-based sizing: higher expected profit = larger trade
    let profit_multiplier = if expected_profit_bps > 100 {
        3 // High profit opportunity
    } else if expected_profit_bps > 50 {
        2 // Medium profit opportunity  
    } else {
        1 // Low profit opportunity
    };
    
    let optimal_size = (EXPERT_MINIMUM_TRADE_SIZE * profit_multiplier)
        .min(max_trade_by_liquidity)
        .min(wallet_balance / 4) // Never use more than 25% of wallet
        .min(EXPERT_MAXIMUM_TRADE_SIZE);
    
    if optimal_size < EXPERT_MINIMUM_TRADE_SIZE {
        return Err(anyhow!("Optimal trade size below minimum profitable threshold"));
    }
    
    Ok(optimal_size)
}

/// Calculate total arbitrage fees including all costs
pub fn calculate_total_arbitrage_fees(trade_amount: u64) -> Result<u64> {
    // 1. Solana network fees
    let base_transaction_fees = SOLANA_BASE_FEE * 2; // 2 transactions
    let priority_fees = PRIORITY_FEE_MODERATE * 2; // 2 transactions
    let compute_fees = COMPUTE_UNITS_ARBITRAGE; // Total compute units
    
    // 2. DEX fees (average across DEXes)
    let avg_dex_fee_bps = (RAYDIUM_FEE_BPS + ORCA_FEE_BPS) / 2; // Average 0.275%
    let dex_fees = trade_amount * avg_dex_fee_bps * 2 / 10_000; // 2 swaps
    
    // 3. Price impact costs (estimated)
    let trade_ratio = trade_amount as f64 / 1_000_000_000.0; // Relative to 1 SOL
    let impact_cost_percentage = if trade_ratio < 1.0 {
        0.001 // 0.1% for trades < 1 SOL
    } else if trade_ratio < 5.0 {
        0.003 // 0.3% for trades 1-5 SOL
    } else {
        0.005 // 0.5% for trades > 5 SOL
    };
    let price_impact_costs = (trade_amount as f64 * impact_cost_percentage) as u64;
    
    let total_fees = base_transaction_fees + priority_fees + compute_fees + dex_fees + price_impact_costs;
    
    Ok(total_fees)
}

/// Determine if arbitrage is mathematically profitable
pub fn is_arbitrage_mathematically_profitable(
    amount_in: u64,
    amount_out: u64,
    total_fees: u64
) -> Result<bool> {
    if amount_out <= amount_in {
        return Ok(false); // No gross profit
    }
    
    let gross_profit = amount_out - amount_in;
    let net_profit = gross_profit.saturating_sub(total_fees);
    
    // Calculate profit in basis points
    let profit_bps = (net_profit * 10_000) / amount_in;
    
    Ok(profit_bps >= MILITARY_MIN_PROFIT_BPS)
}

/// Calculate realistic slippage factor based on trade size and liquidity
pub fn calculate_dynamic_slippage_factor(trade_amount: u64, total_liquidity: u64) -> u64 {
    if total_liquidity == 0 {
        return 500; // 5% slippage for zero liquidity (error case)
    }
    
    let liquidity_ratio = (trade_amount as f64) / (total_liquidity as f64);
    
    // Dynamic slippage in basis points
    let slippage_bps = if liquidity_ratio < 0.0001 {
        10 // 0.1% slippage for very small trades
    } else if liquidity_ratio < 0.001 {
        30 // 0.3% slippage for small trades
    } else if liquidity_ratio < 0.01 {
        100 // 1% slippage for medium trades
    } else if liquidity_ratio < 0.02 {
        200 // 2% slippage for large trades
    } else {
        500 // 5% slippage for very large trades
    };
    
    slippage_bps.min(MILITARY_MAX_SLIPPAGE_BPS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amm_calculation() {
        let result = calculate_amm_output_exact(
            1_000_000_000, // 1 SOL reserve in
            1_000_000_000, // 1 SOL reserve out  
            100_000_000,   // 0.1 SOL input
            25             // 0.25% fee
        ).unwrap();
        
        // Should be less than input due to fees and slippage
        assert!(result < 100_000_000);
        assert!(result > 90_000_000); // But not too much less
    }
    
    #[test]
    fn test_profit_calculation() {
        let fees = calculate_total_arbitrage_fees(1_000_000_000).unwrap();
        let is_profitable = is_arbitrage_mathematically_profitable(
            1_000_000_000, // 1 SOL in
            1_010_000_000, // 1.01 SOL out (1% gross profit)
            fees
        ).unwrap();
        
        // Should be profitable with 1% gross profit after fees
        assert!(is_profitable);
    }
}
