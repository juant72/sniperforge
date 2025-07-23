// ===== SABER POOLS QUICK TEST =====
// Test rápido para verificar la URL de Saber pools

use std::time::Duration;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🗡️  SABER POOLS QUICK TEST INICIADO");
    println!("=================================");
    
    let client = reqwest::Client::new();
    
    println!("📡 Conectando a Saber Registry...");
    
    let response = client
        .get("https://registry.saber.so/data/pools-info.mainnet.json")
        .timeout(Duration::from_secs(15))
        .send()
        .await?;
    
    println!("📊 Status: {}", response.status());
    
    if response.status().is_success() {
        let text_response = response.text().await?;
        println!("📋 Primeros 500 caracteres de respuesta:");
        println!("{}", &text_response[..text_response.len().min(500)]);
        
        // Intentar parsear como JSON
        match serde_json::from_str::<serde_json::Value>(&text_response) {
            Ok(pools) => {
                if let Some(pools_array) = pools.as_array() {
                    println!("✅ Es un array con {} elementos", pools_array.len());
                } else if let Some(pools_object) = pools.as_object() {
                    println!("📦 Es un objeto con {} propiedades", pools_object.len());
                    println!("🔑 Propiedades: {:?}", pools_object.keys().collect::<Vec<_>>());
                    
                    // Buscar arrays dentro del objeto
                    for (key, value) in pools_object {
                        if value.is_array() {
                            if let Some(arr) = value.as_array() {
                                println!("   - {}: array con {} elementos", key, arr.len());
                            }
                        }
                    }
                } else {
                    println!("🤔 No es ni array ni objeto");
                }
            }
            Err(e) => {
                println!("❌ Error parseando JSON: {}", e);
            }
        }
            println!("✅ Total pools encontrados: {}", pools_array.len());
            
            // Buscar pools con nuestros tokens objetivo
            let target_tokens = vec!["USDC", "USDT", "SOL", "BONK", "RAY"];
            let mut found_pools = Vec::new();
            
            for pool in pools_array.iter().take(50) { // Limitar para test rápido
                if let Some(name) = pool.get("name").and_then(|n| n.as_str()) {
                    for token in &target_tokens {
                        if name.to_uppercase().contains(token) {
                            found_pools.push(name);
                            break;
                        }
                    }
                }
            }
            
            println!("🎯 Pools relevantes encontrados:");
            for (i, pool_name) in found_pools.iter().take(10).enumerate() {
                println!("   {}. {}", i + 1, pool_name);
            }
            
            println!("✅ Test completado exitosamente!");
            println!("📈 Saber Registry conectado correctamente");
            
        } else {
            println!("⚠️  Respuesta no es un array");
        }
    } else {
        println!("❌ Error: {}", response.status());
    }
    
    Ok(())
}
