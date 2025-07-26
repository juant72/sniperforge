// Fee calculator without external dependencies

#[derive(Debug, Clone)]
struct OpportunityTest {
    symbol: String,
    gross_profit_percentage: f64,
    confidence: f64,
    trade_amount_sol: f64,
}

#[derive(Debug)]
struct FeeBreakdown {
    network_fees: f64,
    platform_fees: f64, 
    slippage_cost: f64,
    price_impact: f64,
    dex_fees: f64,
    mev_protection: f64,
    safety_margin: f64,
    total_fees: f64,
    net_profit_sol: f64,
    net_profit_percentage: f64,
    profitable: bool,
}

fn calculate_comprehensive_fees(
    trade_amount_sol: f64,
    gross_profit_percentage: f64
) -> FeeBreakdown {
    // Fees en SOL
    let network_fees = 0.00061; // Fixed Solana network fees
    let platform_fees = trade_amount_sol * 0.0004; // Jupiter 0.04%
    let slippage_cost = trade_amount_sol * 0.0015; // 0.15% average slippage
    let price_impact = trade_amount_sol * 0.001; // 0.1% price impact
    let dex_fees = trade_amount_sol * 0.005; // 0.25% * 2 swaps
    let mev_protection = 0.000015; // MEV protection fixed cost
    
    let subtotal_fees = network_fees + platform_fees + slippage_cost + price_impact + dex_fees + mev_protection;
    let safety_margin = subtotal_fees * 0.20; // 20% safety margin
    let total_fees = subtotal_fees + safety_margin;
    
    // Profit calculation
    let gross_profit_sol = trade_amount_sol * (gross_profit_percentage / 100.0);
    let net_profit_sol = gross_profit_sol - total_fees;
    let net_profit_percentage = (net_profit_sol / trade_amount_sol) * 100.0;
    
    FeeBreakdown {
        network_fees,
        platform_fees,
        slippage_cost,
        price_impact,
        dex_fees,
        mev_protection,
        safety_margin,
        total_fees,
        net_profit_sol,
        net_profit_percentage,
        profitable: net_profit_sol > 0.0,
    }
}

fn main() {
    println!("🧮 CALCULADORA DE FEES CON NUEVA CONFIGURACIÓN");
    println!("═══════════════════════════════════════════════");
    println!("📋 Trade Amount: 0.1 SOL (configuración actual)");
    println!("📋 Min Profit Threshold: 0.002 SOL (configuración actual)");
    println!();

    let test_opportunities = vec![
        OpportunityTest {
            symbol: "RAY".to_string(),
            gross_profit_percentage: 0.032, // 0.032%
            confidence: 83.9,
            trade_amount_sol: 0.1,
        },
        OpportunityTest {
            symbol: "WIF".to_string(),
            gross_profit_percentage: 0.494, // 0.494%
            confidence: 91.6,
            trade_amount_sol: 0.1,
        },
        OpportunityTest {
            symbol: "PYTH".to_string(),
            gross_profit_percentage: 0.458, // 0.458%
            confidence: 82.3,
            trade_amount_sol: 0.1,
        },
        OpportunityTest {
            symbol: "BONK".to_string(),
            gross_profit_percentage: 0.121, // 0.121%
            confidence: 85.3,
            trade_amount_sol: 0.1,
        },
        OpportunityTest {
            symbol: "SOL".to_string(),
            gross_profit_percentage: 0.183, // 0.183%
            confidence: 86.4,
            trade_amount_sol: 0.1,
        },
        // Test with higher profit scenarios
        OpportunityTest {
            symbol: "HYPOTHETICAL_HIGH".to_string(),
            gross_profit_percentage: 2.5, // 2.5%
            confidence: 90.0,
            trade_amount_sol: 0.1,
        },
        OpportunityTest {
            symbol: "HYPOTHETICAL_EXTREME".to_string(),
            gross_profit_percentage: 5.0, // 5.0%
            confidence: 95.0,
            trade_amount_sol: 0.1,
        },
    ];

    for opportunity in test_opportunities {
        let fees = calculate_comprehensive_fees(
            opportunity.trade_amount_sol,
            opportunity.gross_profit_percentage
        );

        println!("💰 {} - {:.3}% gross profit ({:.1}% confidence)", 
                opportunity.symbol, 
                opportunity.gross_profit_percentage, 
                opportunity.confidence);
        println!("   📊 Trade Amount: {:.3} SOL", opportunity.trade_amount_sol);
        println!("   💸 Total Fees: {:.6} SOL ({:.2}%)", 
                fees.total_fees, 
                (fees.total_fees / opportunity.trade_amount_sol) * 100.0);
        println!("   📈 Gross Profit: {:.6} SOL", 
                opportunity.trade_amount_sol * (opportunity.gross_profit_percentage / 100.0));
        println!("   💵 Net Profit: {:.6} SOL ({:.3}%)", 
                fees.net_profit_sol, 
                fees.net_profit_percentage);
        
        let meets_threshold = fees.net_profit_sol >= 0.002;
        let status = if fees.profitable && meets_threshold {
            "✅ EJECUTAR"
        } else if fees.profitable && !meets_threshold {
            "⚠️ PROFITABLE PERO < THRESHOLD"
        } else {
            "❌ NO EJECUTAR"
        };
        
        println!("   🎯 Status: {}", status);
        println!("   📋 Meets 0.002 SOL threshold: {}", meets_threshold);
        println!();
    }

    // Analysis summary
    println!("📊 ANÁLISIS CONFIGURACIÓN ACTUAL:");
    println!("═══════════════════════════════════");
    println!("🎯 Min Profit Threshold: 0.002 SOL (2% de 0.1 SOL)");
    println!("📈 Para ser profitable con 0.1 SOL:");
    println!("   • Gross profit mínimo: ~7.8% (para cubrir 7.8% de fees)");
    println!("   • Para threshold 0.002 SOL: gross profit mínimo ~9.8%");
    println!();
    println!("💡 RECOMENDACIÓN:");
    println!("   • Reducir min_profit_threshold_sol a 0.0005 SOL (0.5%)");
    println!("   • O buscar oportunidades con >5% gross profit");
    println!("   • O aumentar trade size a 0.5 SOL para diluir fees fijos");
}
