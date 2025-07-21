// 🧮 CALCULATIONS MODULE - AMM and arbitrage mathematical functions
use anyhow::{Result, anyhow};

/// Calculate output amount for exact input using constant product formula (x * y = k)
/// with fees applied
pub fn calculate_amm_output_exact(
    reserve_in: u64,
    reserve_out: u64,
    amount_in: u64,
    fee_bps: u64,
) -> Result<u64> {
    if reserve_in == 0 || reserve_out == 0 {
        return Err(anyhow!("Invalid reserves: cannot be zero"));
    }

    if amount_in == 0 {
        return Ok(0);
    }

    // Apply fee (fee is in basis points, so 25 bps = 0.25%)
    let fee_multiplier = 10000 - fee_bps;
    let amount_in_after_fee = (amount_in as u128 * fee_multiplier as u128) / 10000;

    // Constant product formula: (x + dx) * (y - dy) = x * y
    // Solving for dy: dy = (y * dx) / (x + dx)
    let numerator = (reserve_out as u128) * amount_in_after_fee;
    let denominator = (reserve_in as u128) + amount_in_after_fee;

    if denominator == 0 {
        return Err(anyhow!("Division by zero in AMM calculation"));
    }

    let amount_out = numerator / denominator;

    // Ensure we don't exceed available reserves
    if amount_out > reserve_out as u128 {
        return Err(anyhow!("Insufficient liquidity: output exceeds reserves"));
    }

    Ok(amount_out as u64)
}

/// Calculate input amount needed for exact output
pub fn calculate_amm_input_exact(
    reserve_in: u64,
    reserve_out: u64,
    amount_out: u64,
    fee_bps: u64,
) -> Result<u64> {
    if reserve_in == 0 || reserve_out == 0 {
        return Err(anyhow!("Invalid reserves: cannot be zero"));
    }

    if amount_out == 0 {
        return Ok(0);
    }

    if amount_out >= reserve_out {
        return Err(anyhow!("Insufficient liquidity: output exceeds reserves"));
    }

    // Constant product formula: (x + dx) * (y - dy) = x * y
    // Solving for dx: dx = (x * dy) / (y - dy)
    let numerator = (reserve_in as u128) * (amount_out as u128);
    let denominator = (reserve_out as u128) - (amount_out as u128);

    if denominator == 0 {
        return Err(anyhow!("Division by zero in AMM calculation"));
    }

    let amount_in_before_fee = numerator / denominator;

    // Account for fees (add fee back to get required input)
    let fee_multiplier = 10000 - fee_bps;
    let amount_in = (amount_in_before_fee * 10000) / fee_multiplier as u128;

    Ok(amount_in as u64)
}

/// Calculate price impact for a given trade
pub fn calculate_price_impact(
    reserve_in: u64,
    reserve_out: u64,
    amount_in: u64,
    fee_bps: u64,
) -> Result<f64> {
    if reserve_in == 0 || reserve_out == 0 {
        return Err(anyhow!("Invalid reserves for price impact calculation"));
    }

    // Price before trade
    let price_before = (reserve_out as f64) / (reserve_in as f64);

    // Calculate output amount
    let amount_out = calculate_amm_output_exact(reserve_in, reserve_out, amount_in, fee_bps)?;

    // Price after trade (effective price)
    let effective_price = (amount_out as f64) / (amount_in as f64);

    // Price impact as percentage
    let price_impact = ((price_before - effective_price) / price_before).abs() * 100.0;

    Ok(price_impact)
}

/// Calculate optimal arbitrage size based on pool sizes and fees
pub fn calculate_optimal_arbitrage_size(
    pool_a_reserve_x: u64,
    pool_a_reserve_y: u64,
    pool_a_fee: u64,
    pool_b_reserve_x: u64,
    pool_b_reserve_y: u64,
    pool_b_fee: u64,
) -> Result<u64> {
    // This is a simplified calculation
    // In practice, you'd want to solve the derivative to find the optimal point
    
    let min_reserve = pool_a_reserve_x
        .min(pool_a_reserve_y)
        .min(pool_b_reserve_x)
        .min(pool_b_reserve_y);

    // Use a conservative percentage of the smallest reserve
    let max_size = min_reserve / 20; // 5% of smallest reserve

    // Factor in fees - higher fees mean smaller optimal size
    let avg_fee = (pool_a_fee + pool_b_fee) / 2;
    let fee_factor = 1.0 - (avg_fee as f64 / 10000.0);
    
    Ok((max_size as f64 * fee_factor) as u64)
}

/// Calculate slippage for a given trade size
pub fn calculate_slippage(
    reserve_in: u64,
    reserve_out: u64,
    amount_in: u64,
    fee_bps: u64,
) -> Result<f64> {
    if amount_in == 0 {
        return Ok(0.0);
    }

    // Expected output without slippage (linear pricing)
    let linear_price = (reserve_out as f64) / (reserve_in as f64);
    let expected_output_linear = (amount_in as f64) * linear_price;

    // Actual output with AMM curve
    let actual_output = calculate_amm_output_exact(reserve_in, reserve_out, amount_in, fee_bps)? as f64;

    // Slippage percentage
    let slippage = ((expected_output_linear - actual_output) / expected_output_linear) * 100.0;

    Ok(slippage.max(0.0)) // Slippage can't be negative
}

/// Estimate gas cost for arbitrage transaction
pub fn estimate_arbitrage_gas_cost(num_swaps: u32, complexity_factor: f64) -> u64 {
    // Base gas cost per swap
    let base_gas_per_swap = 100_000; // ~0.0001 SOL
    
    // Total gas with complexity factor
    let total_gas = (base_gas_per_swap * num_swaps) as f64 * complexity_factor;
    
    // Add buffer for transaction fees and rent
    let with_buffer = total_gas * 1.2;
    
    with_buffer as u64
}

/// Check if arbitrage is profitable after all costs
pub fn is_arbitrage_profitable(
    amount_in: u64,
    amount_out: u64,
    gas_cost: u64,
    min_profit_bps: u64,
) -> bool {
    if amount_out <= amount_in {
        return false; // No gross profit
    }

    let gross_profit = amount_out - amount_in;
    
    if gross_profit <= gas_cost {
        return false; // Doesn't cover gas
    }

    let net_profit = gross_profit - gas_cost;
    let profit_bps = (net_profit * 10_000) / amount_in;

    profit_bps >= min_profit_bps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amm_output_calculation() {
        // Test basic AMM calculation
        let reserve_in = 1_000_000_000; // 1000 tokens
        let reserve_out = 1_000_000_000; // 1000 tokens
        let amount_in = 100_000_000; // 100 tokens
        let fee_bps = 30; // 0.3%

        let result = calculate_amm_output_exact(reserve_in, reserve_out, amount_in, fee_bps);
        assert!(result.is_ok());
        
        let amount_out = result.unwrap();
        assert!(amount_out > 0);
        assert!(amount_out < amount_in); // Should be less due to slippage and fees
    }

    #[test]
    fn test_price_impact() {
        let reserve_in = 1_000_000_000;
        let reserve_out = 1_000_000_000;
        let amount_in = 50_000_000; // 5% of pool
        let fee_bps = 30;

        let impact = calculate_price_impact(reserve_in, reserve_out, amount_in, fee_bps);
        assert!(impact.is_ok());
        
        let impact_pct = impact.unwrap();
        assert!(impact_pct > 0.0);
        assert!(impact_pct < 100.0);
    }

    #[test]
    fn test_profitability_check() {
        let amount_in = 1_000_000_000; // 1000 tokens
        let amount_out = 1_010_000_000; // 1010 tokens (1% profit)
        let gas_cost = 5_000_000; // 5 tokens gas
        let min_profit_bps = 50; // 0.5%

        let is_profitable = is_arbitrage_profitable(amount_in, amount_out, gas_cost, min_profit_bps);
        assert!(is_profitable);
    }
}
