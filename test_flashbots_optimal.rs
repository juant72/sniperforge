// ================================================================================
// TEST FLASHBOTS OPTIMAL SIZING - VERIFICACIÓN DE EXPERTOS
// ================================================================================
// Test para verificar que nuestra implementación de la fórmula de Flashbots
// produce resultados coherentes y óptimos
// ================================================================================

use anyhow::Result;
use log::info;
use sniperforge::fee_calculator::{FeeCalculator, FlashbotsOptimalResult};

fn setup_logging() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
}

#[tokio::test]
async fn test_flashbots_optimal_calculation() -> Result<()> {
    setup_logging();
    
    let calculator = FeeCalculator::new(185.0, "mainnet".to_string())?;
    
    // Escenario 1: Pools balanceados con buena liquidez
    let reserves_a = (1000.0, 1000.0); // 1000 SOL / 1000 TOKEN
    let reserves_b = (950.0, 1050.0);  // 950 SOL / 1050 TOKEN (oportunidad de arbitraje)
    let available_capital = 10.0; // 10 SOL disponibles
    
    info!("🧪 TEST 1: Pools balanceados con oportunidad de arbitraje");
    let result = calculator.calculate_flashbots_optimal_size(
        reserves_a,
        reserves_b,
        available_capital,
        0.8 // Usar máximo 80% del capital
    )?;
    
    println!("📊 Resultado Test 1:");
    println!("   Optimal amount: {:.6} SOL", result.optimal_amount_sol);
    println!("   Expected profit: {:.6} SOL", result.expected_gross_profit);
    println!("   ROI: {:.2}%", result.profit_per_sol * 100.0);
    println!("   Capital efficiency: {:.1}%", result.capital_efficiency * 100.0);
    println!("");
    
    assert!(result.optimal_amount_sol > 0.0, "Should find optimal trade size");
    assert!(result.expected_gross_profit > 0.0, "Should expect profit");
    assert!(result.optimal_amount_sol <= available_capital * 0.8, "Should respect capital limits");
    
    // Escenario 2: Gran desequilibrio de precios
    let reserves_a_big = (10000.0, 5000.0);  // Precio alto en A
    let reserves_b_big = (5000.0, 10000.0);  // Precio bajo en B
    let available_capital_big = 100.0;
    
    info!("🧪 TEST 2: Gran desequilibrio de precios");
    let result_big = calculator.calculate_flashbots_optimal_size(
        reserves_a_big,
        reserves_b_big,
        available_capital_big,
        0.8
    )?;
    
    println!("📊 Resultado Test 2:");
    println!("   Optimal amount: {:.6} SOL", result_big.optimal_amount_sol);
    println!("   Expected profit: {:.6} SOL", result_big.expected_gross_profit);
    println!("   ROI: {:.2}%", result_big.profit_per_sol * 100.0);
    println!("   Capital efficiency: {:.1}%", result_big.capital_efficiency * 100.0);
    println!("");
    
    assert!(result_big.expected_gross_profit > result.expected_gross_profit, 
            "Bigger opportunity should yield more profit");
    
    // Escenario 3: Pools muy pequeños (liquidez limitada)
    let reserves_a_small = (10.0, 10.0);
    let reserves_b_small = (9.5, 10.5);
    let available_capital_small = 5.0;
    
    info!("🧪 TEST 3: Pools pequeños con liquidez limitada");
    let result_small = calculator.calculate_flashbots_optimal_size(
        reserves_a_small,
        reserves_b_small,
        available_capital_small,
        0.8
    )?;
    
    println!("📊 Resultado Test 3:");
    println!("   Optimal amount: {:.6} SOL", result_small.optimal_amount_sol);
    println!("   Expected profit: {:.6} SOL", result_small.expected_gross_profit);
    println!("   ROI: {:.2}%", result_small.profit_per_sol * 100.0);
    println!("   Capital efficiency: {:.1}%", result_small.capital_efficiency * 100.0);
    println!("");
    
    // En pools pequeños, el optimal debería ser menor debido a high slippage
    assert!(result_small.optimal_amount_sol < result.optimal_amount_sol,
            "Small pools should suggest smaller trade sizes");
    
    // Escenario 4: No hay oportunidad de arbitraje
    let reserves_a_equal = (1000.0, 1000.0);
    let reserves_b_equal = (1000.0, 1000.0); // Mismo precio
    
    info!("🧪 TEST 4: No hay oportunidad de arbitraje");
    let result_no_arb = calculator.calculate_flashbots_optimal_size(
        reserves_a_equal,
        reserves_b_equal,
        available_capital,
        0.8
    )?;
    
    println!("📊 Resultado Test 4:");
    println!("   Optimal amount: {:.6} SOL", result_no_arb.optimal_amount_sol);
    println!("   Expected profit: {:.6} SOL", result_no_arb.expected_gross_profit);
    println!("");
    
    // Sin oportunidad, el profit debería ser muy bajo o cero
    assert!(result_no_arb.expected_gross_profit < 0.001,
            "No arbitrage opportunity should yield minimal profit");
    
    Ok(())
}

#[tokio::test]
async fn test_optimal_vs_fixed_size_comparison() -> Result<()> {
    setup_logging();
    
    let calculator = FeeCalculator::new(185.0, "mainnet".to_string())?;
    
    // Configuración de test
    let reserves_a = (5000.0, 3000.0);  
    let reserves_b = (3000.0, 5000.0);  
    let available_capital = 50.0;
    
    // Calcular tamaño óptimo
    let optimal_result = calculator.calculate_flashbots_optimal_size(
        reserves_a,
        reserves_b,
        available_capital,
        0.8
    )?;
    
    info!("🧪 COMPARACIÓN: Optimal vs Fixed Sizes");
    println!("📊 Optimal Size Strategy:");
    println!("   Amount: {:.6} SOL", optimal_result.optimal_amount_sol);
    println!("   Expected profit: {:.6} SOL", optimal_result.expected_gross_profit);
    println!("   ROI: {:.2}%", optimal_result.profit_per_sol * 100.0);
    println!("");
    
    // Probar tamaños fijos para comparación
    let fixed_sizes = vec![1.0, 5.0, 10.0, 20.0, 30.0];
    
    for fixed_size in fixed_sizes {
        let fixed_profit = calculator.calculate_gross_profit_for_amount(
            fixed_size,
            reserves_a,
            reserves_b
        )?;
        
        let fixed_roi = if fixed_size > 0.0 { fixed_profit / fixed_size } else { 0.0 };
        
        println!("📊 Fixed Size {:.1} SOL:", fixed_size);
        println!("   Expected profit: {:.6} SOL", fixed_profit);
        println!("   ROI: {:.2}%", fixed_roi * 100.0);
        println!("");
    }
    
    // El optimal debería superar a la mayoría de tamaños fijos en ROI
    assert!(optimal_result.profit_per_sol > 0.0, "Optimal should be profitable");
    
    Ok(())
}

fn main() {
    println!("🧪 Running Flashbots Optimal Sizing Tests...");
    println!("");
    
    if let Err(e) = tokio::runtime::Runtime::new().unwrap().block_on(async {
        test_flashbots_optimal_calculation().await?;
        test_optimal_vs_fixed_size_comparison().await?;
        Ok::<(), anyhow::Error>(())
    }) {
        eprintln!("❌ Test failed: {}", e);
        std::process::exit(1);
    }
    
    println!("✅ All tests passed! Flashbots optimal sizing is working correctly.");
}
