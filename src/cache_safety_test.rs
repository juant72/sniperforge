/// Test de seguridad de cache - Demuestra diferentes niveles de seguridad
/// 
/// Ejecutar con: cargo run -- test cache-safety

use anyhow::Result;
use std::time::Instant;
use crate::shared::syndica_websocket::{SyndicaConfig, SyndicaWebSocketClient};

pub async fn test_cache_safety_levels() -> Result<()> {
    println!("\n🛡️ TESTING CACHE SAFETY LEVELS");
    println!("==================================");
    
    let config = SyndicaConfig::default();
    let mut client = SyndicaWebSocketClient::new(config).await?;
    let token = "So11111111111111111111111111111111111111112"; // SOL
    
    // Connect first
    print!("🔗 Connecting to Syndica... ");
    match client.connect().await {
        Ok(()) => {
            println!("✅ Connected");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        Err(_) => {
            println!("❌ Failed, using simulation for demo");
            client.start_price_simulation().await?;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }
    
    println!("\n📊 CACHE SAFETY COMPARISON:");
    println!("===========================");
    
    // 1. Método normal con cache (RIESGOSO para trading)
    println!("\n1️⃣ MÉTODO NORMAL (CON CACHE) - ⚠️ RIESGOSO");
    let start = Instant::now();
    match client.get_price_ultra_fast(token).await {
        Ok(Some(price)) => {
            println!("   💰 Price: ${:.4} in {}μs", price, start.elapsed().as_micros());
            println!("   ⚠️  WARNING: Puede usar datos cached (stale data risk)");
        }
        Ok(None) => {
            println!("   ❌ No cached data available");
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }
    
    // 2. Método ultra-safe (solo datos < 10ms)
    println!("\n2️⃣ ULTRA-SAFE METHOD - 🛡️ SEGURO");
    let start = Instant::now();
    match client.get_price_ultra_safe(token).await {
        Ok(Some(price)) => {
            println!("   💰 Price: ${:.4} in {}μs", price, start.elapsed().as_micros());
            println!("   ✅ SAFE: Solo datos ultra-frescos (< 10ms)");
        }
        Ok(None) => {
            println!("   📡 No ultra-fresh data available (FORCED EXTERNAL FETCH)");
            println!("   ✅ SAFE: Forcing external fetch for fresh data");
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }
    
    // 3. Método directo sin cache
    println!("\n3️⃣ DIRECT NO-CACHE METHOD - 🔥 MÁXIMA SEGURIDAD");
    let start = Instant::now();
    match client.get_price_direct_no_cache(token).await {
        Ok(Some(price)) => {
            println!("   💰 Price: ${:.4} in {}μs", price, start.elapsed().as_micros());
            println!("   ✅ MAXIMUM SAFETY: Zero cache risk");
        }
        Ok(None) => {
            println!("   📡 Forcing external HTTP fetch (NO CACHE EVER)");
            println!("   ✅ MAXIMUM SAFETY: Always fresh data");
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }
    
    // 4. Método de producción (balanceado)
    println!("\n4️⃣ PRODUCTION-SAFE METHOD - 🎯 BALANCEADO");
    let start = Instant::now();
    match client.get_price_production_safe(token).await {
        Ok(Some(price)) => {
            println!("   💰 Price: ${:.4} in {}μs", price, start.elapsed().as_micros());
            println!("   ✅ BALANCED: Ultra-safe cache + direct fallback");
        }
        Ok(None) => {
            println!("   📡 No fresh data available from any source");
            println!("   ⚠️  Requires external HTTP client for fallback");
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }
    
    // 5. Deshabilitar cache completamente
    println!("\n5️⃣ DISABLE CACHE COMPLETELY - 🚨 ZERO RISK");
    println!("   🔥 Disabling all cache for maximum trading safety...");
    client.disable_cache_completely().await?;
    
    let cache_active = client.is_cache_active().await;
    println!("   📊 Cache status: {}", 
        if cache_active { "❌ ACTIVE (risky)" } else { "✅ DISABLED (safe)" }
    );
    
    if !cache_active {
        println!("   ✅ SUCCESS: Cache completely disabled");
        println!("   🛡️ Now all price requests will force fresh fetches");
    }
    
    println!("\n🎯 RECOMENDACIONES FINALES:");
    println!("===========================");
    println!("💰 Para TRADING REAL:");
    println!("   ✅ Usar get_price_direct_no_cache() o deshabilitar cache");
    println!("   ✅ Verificar precios con múltiples fuentes");
    println!("   ✅ Nunca confiar en datos > 10ms para arbitrage");
    println!("");
    println!("📊 Para MONITORING:");
    println!("   ✅ get_price_ultra_fast() es OK (acepta cache)");
    println!("   ✅ Cache puede mejorar performance para dashboards");
    println!("");
    println!("⚡ Para HIGH-FREQUENCY:");
    println!("   🚨 NUNCA usar cache - solo WebSocket directo");
    println!("   🚨 Latencia < 1ms requerida");
    
    println!("\n✅ Cache safety test completed!");
    Ok(())
}
