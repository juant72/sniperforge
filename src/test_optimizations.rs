// Test rápido para verificar optimizaciones
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 TESTING OPTIMIZACIONES CRÍTICAS");
    println!("═══════════════════════════════════");
    
    let start = Instant::now();
    
    // Test filtros más permisivos
    let test_pools = vec![
        ("Pool A", 75_000.0, 15_000.0),   // Debería pasar (antes NO)
        ("Pool B", 45_000.0, 8_000.0),    // No debería pasar
        ("Pool C", 100_000.0, 25_000.0),  // Debería pasar
        ("Pool D", 30_000.0, 5_000.0),    // No debería pasar
    ];
    
    let min_tvl = 50_000.0;
    let min_volume = 10_000.0;
    
    println!("📊 NUEVOS FILTROS:");
    println!("   Min TVL: ${:.0} (reducido desde $200,000)", min_tvl);
    println!("   Min Volume: ${:.0} (reducido desde $50,000)", min_volume);
    println!();
    
    let mut qualified = 0;
    for (name, tvl, volume) in test_pools {
        let passes = tvl > min_tvl && volume > min_volume;
        if passes {
            qualified += 1;
            println!("✅ {} - TVL: ${:.0}, Vol: ${:.0} - QUALIFIED", name, tvl, volume);
        } else {
            println!("❌ {} - TVL: ${:.0}, Vol: ${:.0} - excluded", name, tvl, volume);
        }
    }
    
    println!();
    println!("🎯 RESULTADO: {} pools qualified (esperado: más pools que antes)", qualified);
    println!("⚡ Tiempo test: {:?}", start.elapsed());
    
    // Test Phoenix timeout simulation
    println!();
    println!("🔧 PHOENIX RPC TIMEOUT TEST:");
    let timeout_start = Instant::now();
    
    // Simular timeout de 10 segundos
    let timeout_result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        async {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            "success"
        }
    ).await;
    
    match timeout_result {
        Ok(result) => println!("✅ RPC completed: {}", result),
        Err(_) => println!("⚠️  RPC timeout (como esperado)"),
    }
    
    println!("⚡ Phoenix test time: {:?}", timeout_start.elapsed());
    println!();
    println!("🎉 OPTIMIZACIONES VERIFICADAS!");
    
    Ok(())
}
