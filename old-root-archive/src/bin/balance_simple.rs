// ================================================================
// BALANCE CHECK SIMPLE
// ================================================================

use std::process::Command;

fn main() {
    println!("🔍 VERIFICANDO BALANCE...");
    println!("========================");
    
    let output = Command::new("solana")
        .args(&[
            "balance",
            "JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7",
            "--url",
            "https://api.mainnet-beta.solana.com"
        ])
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                let balance_str = String::from_utf8_lossy(&result.stdout);
                println!("✅ Balance: {}", balance_str.trim());
                
                // Parse SOL amount
                if let Some(sol_part) = balance_str.split_whitespace().next() {
                    if let Ok(balance_f64) = sol_part.parse::<f64>() {
                        println!();
                        if balance_f64 >= 0.1 {
                            println!("🟢 SUFICIENTE para arbitraje (>= 0.1 SOL)");
                            println!("✅ LISTO PARA TRADING REAL");
                        } else if balance_f64 >= 0.01 {
                            println!("🟡 LIMITADO - Solo gas fees (>= 0.01 SOL)");
                            println!("⚠️  Deposita más SOL para arbitraje");
                        } else {
                            println!("🔴 INSUFICIENTE para trading");
                            println!("❌ Necesitas depositar SOL");
                        }
                    }
                }
            } else {
                let error_str = String::from_utf8_lossy(&result.stderr);
                println!("❌ Error: {}", error_str);
            }
        }
        Err(e) => {
            println!("❌ Error ejecutando solana CLI: {}", e);
            println!("💡 Tip: Asegúrate de tener Solana CLI instalado");
        }
    }
    
    println!();
    println!("📊 REQUERIMIENTOS TRADING:");
    println!("• Gas fees: ~0.005 SOL por transacción");
    println!("• Capital mínimo: 0.1 SOL");
    println!("• Recomendado: 1.0+ SOL para trading activo");
}
