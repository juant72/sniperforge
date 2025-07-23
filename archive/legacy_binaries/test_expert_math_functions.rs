// 🚀 EXPERT MATHEMATICAL FUNCTIONS TEST
// Test the new expert arbitrage calculations

use solana_sdk::pubkey::Pubkey;

// Expert constants from the roadmap
const EXPERT_MINIMUM_TRADE_SIZE: u64 = 1_000_000_000; // 1 SOL minimum
const EXPERT_RAYDIUM_FEE_BPS: u64 = 25; // 0.25%
const EXPERT_ORCA_FEE_BPS: u64 = 30; // 0.30%
const EXPERT_SOLANA_TRANSACTION_COST: u64 = 10_000; // 0.00001 SOL
const EXPERT_PRIORITY_FEE_LAMPORTS: u64 = 1_000_000; // 0.001 SOL
const MILITARY_MIN_PROFIT_BPS: u64 = 50; // 0.5% minimum profit

// Mock pool data for testing
struct PoolData {
    token_a_amount: u64,
    token_b_amount: u64,
}

fn main() {
    println!("🚀 TESTING EXPERT MATHEMATICAL FUNCTIONS");
    println!("==========================================");
    
    // Test Case 1: Large liquid pool (like real Raydium SOL/USDC)
    let pool_a = PoolData {
        token_a_amount: 10_000_000_000_000, // 10,000 SOL
        token_b_amount: 2_000_000_000_000,  // 2,000,000 USDC (6 decimals)
    };
    
    // Test Case 2: Another large pool with slight price difference
    let pool_b = PoolData {
        token_a_amount: 8_000_000_000_000,  // 8,000 SOL
        token_b_amount: 1_600_000_000_000,  // 1,600,000 USDC (slightly different ratio)
    };
    
    let trade_amount = EXPERT_MINIMUM_TRADE_SIZE; // 1 SOL
    
    println!("📊 TEST POOLS:");
    println!("   Pool A: {:.0} SOL, {:.0} USDC", 
             pool_a.token_a_amount as f64 / 1e9, 
             pool_a.token_b_amount as f64 / 1e6);
    println!("   Pool B: {:.0} SOL, {:.0} USDC", 
             pool_b.token_a_amount as f64 / 1e9, 
             pool_b.token_b_amount as f64 / 1e6);
    println!("   Trade Amount: {:.3} SOL", trade_amount as f64 / 1e9);
    println!();
    
    // Test AMM calculations
    println!("🧮 TESTING EXPERT AMM CALCULATIONS:");
    
    // Step 1: Buy tokens from Pool A
    let tokens_received = calculate_amm_output_exact(
        pool_a.token_a_amount, 
        pool_a.token_b_amount, 
        trade_amount, 
        EXPERT_RAYDIUM_FEE_BPS
    );
    
    println!("   Step 1 (SOL->USDC): {:.3} SOL -> {:.2} USDC", 
             trade_amount as f64 / 1e9, 
             tokens_received as f64 / 1e6);
    
    // Step 2: Sell tokens to Pool B
    let sol_received = calculate_amm_output_exact(
        pool_b.token_b_amount, 
        pool_b.token_a_amount, 
        tokens_received, 
        EXPERT_ORCA_FEE_BPS
    );
    
    println!("   Step 2 (USDC->SOL): {:.2} USDC -> {:.3} SOL", 
             tokens_received as f64 / 1e6, 
             sol_received as f64 / 1e9);
    
    // Calculate profit
    let gross_profit = if sol_received > trade_amount { 
        sol_received - trade_amount 
    } else { 
        0 
    };
    
    println!();
    println!("💰 PROFIT CALCULATION:");
    println!("   Gross Profit: {:.6} SOL", gross_profit as f64 / 1e9);
    
    // Calculate all costs
    let total_fees = calculate_total_arbitrage_fees(trade_amount);
    let net_profit = if gross_profit > total_fees { 
        gross_profit - total_fees 
    } else { 
        0 
    };
    
    let profit_percentage = if net_profit > 0 {
        (net_profit as f64 / trade_amount as f64) * 100.0
    } else {
        0.0
    };
    
    println!("   Total Fees: {:.6} SOL", total_fees as f64 / 1e9);
    println!("   Net Profit: {:.6} SOL", net_profit as f64 / 1e9);
    println!("   Profit %: {:.4}%", profit_percentage);
    
    // Check if profitable
    let min_profit_threshold = (MILITARY_MIN_PROFIT_BPS as f64 / 100.0);
    let is_profitable = profit_percentage >= min_profit_threshold;
    
    println!();
    println!("🎯 EXPERT VERDICT:");
    println!("   Minimum Required: {:.2}%", min_profit_threshold);
    println!("   Actual Profit: {:.4}%", profit_percentage);
    println!("   PROFITABLE: {}", if is_profitable { "✅ YES" } else { "❌ NO" });
    
    if is_profitable {
        println!();
        println!("🚀 EXPERT RECOMMENDATION: EXECUTE ARBITRAGE");
        println!("   Expected daily profit (100 trades): {:.3} SOL", 
                 net_profit as f64 / 1e9 * 100.0);
    } else {
        println!();
        println!("⚠️  EXPERT RECOMMENDATION: DO NOT EXECUTE");
        println!("   Profit too low or unprofitable");
    }
}

/// Calculate exact AMM output using constant product formula with fees
fn calculate_amm_output_exact(reserve_in: u64, reserve_out: u64, amount_in: u64, fee_bps: u64) -> u64 {
    if reserve_in == 0 || reserve_out == 0 || amount_in == 0 {
        return 0;
    }
    
    // EXPERT FORMULA: x * y = k (constant product)
    let fee_multiplier = 10_000 - fee_bps; // e.g., 9975 for 0.25% fee
    let amount_in_after_fees = (amount_in * fee_multiplier) / 10_000;
    
    // Prevent overflow with careful calculation
    let numerator = amount_in_after_fees as u128 * reserve_out as u128;
    let denominator = reserve_in as u128 + amount_in_after_fees as u128;
    
    if denominator == 0 {
        return 0;
    }
    
    let amount_out = (numerator / denominator) as u64;
    
    // Sanity check: output cannot exceed pool reserves
    if amount_out >= reserve_out {
        return reserve_out - 1; // Leave 1 token in pool
    }
    
    amount_out
}

/// Calculate exact fees for a complete arbitrage round-trip
fn calculate_total_arbitrage_fees(trade_amount: u64) -> u64 {
    // EXPERT CALCULATION: All real fees included
    
    // 1. Solana base transaction fees (2 transactions)
    let solana_base_fees = EXPERT_SOLANA_TRANSACTION_COST;
    
    // 2. Priority fees for MEV protection (crucial for success)
    let priority_fees = EXPERT_PRIORITY_FEE_LAMPORTS;
    
    // 3. DEX fees (0.25% per swap x 2 swaps = 0.5% total)
    let dex_fees_first_swap = (trade_amount * EXPERT_RAYDIUM_FEE_BPS) / 10_000;
    let tokens_after_first_swap = trade_amount - dex_fees_first_swap;
    let dex_fees_second_swap = (tokens_after_first_swap * EXPERT_ORCA_FEE_BPS) / 10_000;
    
    let total_fees = solana_base_fees + priority_fees + dex_fees_first_swap + dex_fees_second_swap;
    
    total_fees
}
