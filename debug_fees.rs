use anyhow::Result;
use sniperforge::expert::{calculations::*, constants::*};

fn main() -> Result<()> {
    println!("🔍 DEBUGGING TEST FEE CALCULATION");
    
    let trade_amount = 1_000_000_000; // 1 SOL
    let gross_output = 1_010_000_000; // 1.01 SOL (1% gross profit)
    
    println!("📊 INPUT:");
    println!("   Trade amount: {} lamports (1 SOL)", trade_amount);
    println!("   Gross output: {} lamports (1.01 SOL)", gross_output);
    println!("   Gross profit: {} lamports (0.01 SOL)", gross_output - trade_amount);
    
    let fees = calculate_total_arbitrage_fees(trade_amount)?;
    println!("\n💰 FEES BREAKDOWN:");
    println!("   Total fees: {} lamports", fees);
    println!("   Fees in SOL: {:.6}", fees as f64 / 1_000_000_000.0);
    
    let gross_profit = gross_output - trade_amount;
    let net_profit = gross_profit.saturating_sub(fees);
    
    println!("\n🎯 PROFIT ANALYSIS:");
    println!("   Gross profit: {} lamports ({:.4} SOL)", gross_profit, gross_profit as f64 / 1_000_000_000.0);
    println!("   Net profit: {} lamports ({:.6} SOL)", net_profit, net_profit as f64 / 1_000_000_000.0);
    
    let profit_bps = (net_profit * 10_000) / trade_amount;
    println!("   Profit BPS: {} ({}%)", profit_bps, profit_bps as f64 / 100.0);
    println!("   Required min BPS: {} ({}%)", MILITARY_MIN_PROFIT_BPS, MILITARY_MIN_PROFIT_BPS as f64 / 100.0);
    
    let is_profitable = is_arbitrage_mathematically_profitable(
        trade_amount,
        gross_output,
        fees
    )?;
    
    println!("\n✅ RESULT:");
    println!("   Is profitable: {}", is_profitable);
    
    if !is_profitable {
        println!("   ❌ FAIL REASON: Net profit {} BPS < required {} BPS", profit_bps, MILITARY_MIN_PROFIT_BPS);
    }
    
    Ok(())
}
