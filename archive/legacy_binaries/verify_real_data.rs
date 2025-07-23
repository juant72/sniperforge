use reqwest;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 VERIFICACIÓN DE DATOS REALES - MÚLTIPLES LLAMADAS");
    println!("📊 Probando que los datos cambian en tiempo real...\n");

    // Hacer 5 llamadas con intervalos de 3 segundos para ver variaciones
    for i in 1..=5 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        println!("🔄 === LLAMADA {} - Timestamp: {} ===", i, timestamp);
        
        // 1. Precio SOL desde CoinGecko
        match get_coingecko_price().await {
            Ok(price) => println!("💰 CoinGecko SOL: ${:.4}", price),
            Err(e) => println!("❌ CoinGecko error: {}", e),
        }
        
        // 2. Datos de Jupiter
        match get_jupiter_data().await {
            Ok(count) => println!("🚀 Jupiter tokens activos: {}", count),
            Err(e) => println!("❌ Jupiter error: {}", e),
        }
        
        // 3. Datos de DexScreener
        match get_dexscreener_data().await {
            Ok(pairs) => println!("📊 DexScreener pairs encontrados: {}", pairs),
            Err(e) => println!("❌ DexScreener error: {}", e),
        }
        
        // 4. Blockchain data - último bloque
        match get_solana_latest_block().await {
            Ok(slot) => println!("⛓️  Último slot de Solana: {}", slot),
            Err(e) => println!("❌ Solana RPC error: {}", e),
        }
        
        println!("⏰ Esperando 3 segundos...\n");
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
    
    println!("✅ PRUEBA COMPLETADA");
    println!("🎯 Si ves DIFERENTES valores en cada llamada = DATOS REALES");
    println!("❌ Si ves MISMOS valores siempre = DATOS FALSOS");
    
    Ok(())
}

async fn get_coingecko_price() -> Result<f64, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd")
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    let json: Value = response.json().await?;
    let price = json["solana"]["usd"].as_f64()
        .ok_or("No se pudo obtener precio")?;
    
    Ok(price)
}

async fn get_jupiter_data() -> Result<usize, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://quote-api.jup.ag/v6/tokens")
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    let json: Value = response.json().await?;
    let tokens = json.as_array()
        .ok_or("Respuesta no es array")?;
    
    Ok(tokens.len())
}

async fn get_dexscreener_data() -> Result<usize, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.dexscreener.com/latest/dex/tokens/So11111111111111111111111111111111111111112")
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    let json: Value = response.json().await?;
    let pairs = json["pairs"].as_array()
        .ok_or("No hay pairs")?;
    
    Ok(pairs.len())
}

async fn get_solana_latest_block() -> Result<u64, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getSlot"
    });
    
    let response = client
        .post("https://api.mainnet-beta.solana.com")
        .json(&payload)
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    let json: Value = response.json().await?;
    let slot = json["result"].as_u64()
        .ok_or("No se pudo obtener slot")?;
    
    Ok(slot)
}
