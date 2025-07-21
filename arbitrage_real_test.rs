use anyhow::Result;
use std::fs;
use std::process::Command;
use std::str;

fn main() -> Result<()> {
    println!("🚀 === PRUEBA REAL DE ARBITRAJE CON MEDICIÓN ===");
    println!("===============================================");

    // PASO 1: Medir balance inicial
    println!("💰 Midiendo balance inicial...");
    let balance_inicial = get_balance()?;
    println!("📊 BALANCE INICIAL: {} SOL", balance_inicial);

    // PASO 2: Verificar oportunidad actual
    println!("\n🔍 Verificando oportunidad de arbitraje...");
    let (spread, profit) = get_arbitrage_opportunity()?;
    println!("📈 Spread actual: {}%", spread);
    println!("💰 Profit estimado: ${}", profit);

    if spread < 10.0 {
        println!("⚠️ Spread muy bajo ({}%), no ejecutando arbitraje", spread);
        return Ok(());
    }

    println!("✅ Oportunidad rentable detectada!");

    // PASO 3: Preguntar confirmación
    println!("\n🎯 ¿Ejecutar arbitraje REAL? (escribir 'SI' para confirmar):");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_uppercase() != "SI" {
        println!("❌ Arbitraje cancelado por usuario");
        return Ok(());
    }

    // PASO 4: Ejecutar arbitraje usando el comando que sabemos funciona
    println!("\n🚀 EJECUTANDO ARBITRAJE REAL...");
    println!("💡 Usando comando: cargo run --bin sniperforge -- arbitrage-execute --wallet test-arbitrage-wallet.json --network devnet --amount 0.01 --confirm");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "sniperforge",
            "--",
            "arbitrage-execute",
            "--wallet",
            "test-arbitrage-wallet.json",
            "--network",
            "devnet",
            "--amount",
            "0.01",
            "--confirm",
        ])
        .output()?;

    let stdout = str::from_utf8(&output.stdout)?;
    let stderr = str::from_utf8(&output.stderr)?;

    println!("📋 Output del arbitraje:");
    println!("{}", stdout);
    if !stderr.is_empty() {
        println!("⚠️ Stderr:");
        println!("{}", stderr);
    }

    // PASO 5: Esperar confirmación y medir balance final
    println!("\n⏳ Esperando 10 segundos para confirmación en blockchain...");
    std::thread::sleep(std::time::Duration::from_secs(10));

    println!("💰 Midiendo balance final...");
    let balance_final = get_balance()?;
    println!("📊 BALANCE FINAL: {} SOL", balance_final);

    // PASO 6: Calcular diferencia
    let diferencia = balance_final - balance_inicial;

    println!("\n🎯 === RESULTADO FINAL ===");
    println!("📊 Balance inicial: {} SOL", balance_inicial);
    println!("📊 Balance final:   {} SOL", balance_final);
    println!("📊 Diferencia:      {:.9} SOL", diferencia);

    if diferencia > 0.0 {
        println!("🎉 ¡ARBITRAJE EXITOSO! Ganancia de +{:.9} SOL", diferencia);
        println!(
            "💰 Incremento del balance: +{:.6}%",
            (diferencia / balance_inicial) * 100.0
        );
        println!("✅ DEMOSTRADO: El arbitraje SÍ funciona y aumenta el balance real");
    } else if diferencia < 0.0 {
        println!("📉 Pérdida en arbitraje: {:.9} SOL", diferencia);
        println!("❌ El arbitraje resultó en pérdida");
    } else {
        println!("➖ Sin cambio en balance");
        println!("🤔 Posible empate o error en medición");
    }

    Ok(())
}

fn get_balance() -> Result<f64> {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "check_devnet_balance"])
        .output()?;

    let stdout = str::from_utf8(&output.stdout)?;

    // Buscar línea con balance
    for line in stdout.lines() {
        if line.contains("SOL Balance:") && line.contains("SOL") {
            // Extraer número de "💰 SOL Balance: 2.000000000 SOL"
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "Balance:" && i + 1 < parts.len() {
                    if let Ok(balance) = parts[i + 1].parse::<f64>() {
                        return Ok(balance);
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!("No se pudo extraer balance"))
}

fn get_arbitrage_opportunity() -> Result<(f64, f64)> {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_arbitrage_cross_dex"])
        .output()?;

    let stdout = str::from_utf8(&output.stdout)?;

    let mut spread = 0.0;
    let mut profit = 0.0;

    for line in stdout.lines() {
        if line.contains("Spread:") && line.contains("%") {
            // Buscar "Spread: 62.78%"
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "Spread:" && i + 1 < parts.len() {
                    let spread_str = parts[i + 1].replace("%", "");
                    if let Ok(s) = spread_str.parse::<f64>() {
                        spread = s;
                    }
                }
            }
        }
        if line.contains("Profit por SOL:") && line.contains("$") {
            // Buscar "Profit por SOL: $62.46"
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "SOL:" && i + 1 < parts.len() {
                    let profit_str = parts[i + 1].replace("$", "");
                    if let Ok(p) = profit_str.parse::<f64>() {
                        profit = p;
                    }
                }
            }
        }
    }

    Ok((spread, profit))
}
