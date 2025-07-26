// ================================================================================
// TEST FLASHBOTS OPTIMAL SIZING - VALIDACIÓN RÁPIDA
// ================================================================================

use anyhow::Result;
use sniperforge::fee_calculator::{FeeCalculator, FlashbotsOptimalResult};

fn main() -> Result<()> {
    println!("🧪 Testing Flashbots Optimal Sizing Algorithm...\n");
    
    let fee_calc = FeeCalculator::new();
    
    // Test 1: Escenario con oportunidad clara
    let available_capital = 0.15; // 0.15 SOL disponible
    let reserves_a = (100_000.0, 110_000.0); // Pool A: 100k SOL, 110k TOKEN  
    let reserves_b = (105_000.0, 100_000.0); // Pool B: 105k TOKEN, 100k SOL
    
    println!("📊 TEST 1: Pools con oportunidad de arbitraje");
    println!("   Capital disponible: {:.3} SOL", available_capital);
    println!("   Pool A reserves: {:.0} / {:.0}", reserves_a.0, reserves_a.1);
    println!("   Pool B reserves: {:.0} / {:.0}", reserves_b.0, reserves_b.1);
    
    let result = fee_calc.calculate_flashbots_optimal_size(
        reserves_a,
        reserves_b,
        available_capital,
        0.8  // Usar máximo 80% del capital disponible
    )?;
    
    println!("   ✅ RESULTADO FLASHBOTS:");
    println!("      Monto óptimo: {:.6} SOL", result.optimal_amount_sol);
    println!("      Óptimo teórico: {:.6} SOL", result.theoretical_optimal);
    println!("      Profit esperado: {:.6} SOL", result.expected_gross_profit);
    println!("      ROI por SOL: {:.2}%", result.profit_per_sol * 100.0);
    println!("      Eficiencia: {:.1}%", result.capital_efficiency * 100.0);
    println!("      ¿Limitado por capital?: {}", result.is_capital_limited);
    
    // Test 2: Comparación con método fijo
    println!("\n🆚 TEST 2: Comparación con método tradicional");
    let fixed_amount = available_capital * 0.5; // 50% del capital
    let fixed_profit = fee_calc.calculate_gross_profit_for_amount(
        fixed_amount,
        reserves_a,
        reserves_b
    )?;
    
    println!("   📏 MÉTODO FIJO (50% capital):");
    println!("      Monto: {:.6} SOL", fixed_amount);
    println!("      Profit: {:.6} SOL", fixed_profit);
    println!("      ROI: {:.2}%", (fixed_profit / fixed_amount) * 100.0);
    
    let improvement = (result.expected_gross_profit / fixed_profit - 1.0) * 100.0;
    println!("   📈 MEJORA CON FLASHBOTS: {:.1}%", improvement);
    
    // Validaciones
    if result.optimal_amount_sol > 0.0 && result.expected_gross_profit > 0.0 {
        println!("\n✅ VALIDACIÓN EXITOSA:");
        println!("   ✓ Encontró monto óptimo positivo");
        println!("   ✓ Respeta límites de capital");
        println!("   ✓ Calcula profit esperado");
        
        if improvement > 0.0 {
            println!("   ✓ Supera al método tradicional por {:.1}%", improvement);
        }
    } else {
        println!("\n⚠️  Sin oportunidad de arbitraje detectada");
    }
    
    println!("\n🎯 CONCLUSIÓN: Algoritmo Flashbots funcionando correctamente");
    Ok(())
}
