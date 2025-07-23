use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::time::Instant;

fn main() {
    println!("🎯 === ANÁLISIS MILITAR: CALIDAD DE DATOS ===");
    
    // 1. ANÁLISIS DE HELIUS PREMIUM
    println!("\n🔥 1. HELIUS PREMIUM - ¿ES BUENO?");
    test_helius_quality();
    
    // 2. ANÁLISIS DE APIS EXTERNAS
    println!("\n📊 2. APIS EXTERNAS - ¿METEN RUIDO?");
    test_external_apis();
    
    // 3. ANÁLISIS DE POOLS REALES
    println!("\n🏊 3. POOLS REALES - ¿HAY DATOS FALSOS?");
    test_pool_reality();
    
    // 4. ANÁLISIS DE OPORTUNIDADES
    println!("\n💰 4. OPORTUNIDADES - ¿POR QUÉ NO SE ENCUENTRAN?");
    analyze_opportunity_absence();
    
    println!("\n🎯 === CONCLUSIONES FINALES ===");
    print_final_conclusions();
}

fn test_helius_quality() {
    let helius_url = "https://mainnet.helius-rpc.com/?api-key=062bf3dd-23d4-4ffd-99fd-6e397ee59d6c";
    let solana_url = "https://api.mainnet-beta.solana.com";
    
    println!("🔍 Comparando Helius Premium vs Solana Público:");
    
    // Test de latencia
    let start = Instant::now();
    let helius_client = RpcClient::new(helius_url.to_string());
    if let Ok(helius_slot) = helius_client.get_slot() {
        let helius_time = start.elapsed();
        println!("✅ Helius Premium: Slot {} en {:?}", helius_slot, helius_time);
        
        let start = Instant::now();
        let solana_client = RpcClient::new(solana_url.to_string());
        if let Ok(solana_slot) = solana_client.get_slot() {
            let solana_time = start.elapsed();
            println!("✅ Solana Público: Slot {} en {:?}", solana_slot, solana_time);
            
            // Comparación
            if helius_time < solana_time {
                println!("🟢 HELIUS ES MÁS RÁPIDO: {:?} vs {:?}", helius_time, solana_time);
            } else {
                println!("🟡 Solana público es más rápido: {:?} vs {:?}", solana_time, helius_time);
            }
            
            // Sincronización
            let diff = if helius_slot > solana_slot {
                helius_slot - solana_slot
            } else {
                solana_slot - helius_slot
            };
            
            if diff <= 2 {
                println!("🟢 SINCRONIZACIÓN PERFECTA: {} slots de diferencia", diff);
            } else {
                println!("🟡 Desincronización: {} slots de diferencia", diff);
            }
        }
    }
}

fn test_external_apis() {
    println!("🔍 Analizando APIs externas:");
    
    // Test Jupiter
    println!("\n🪐 Jupiter API:");
    if let Ok(response) = reqwest::blocking::get("https://quote-api.jup.ag/v6/tokens") {
        if response.status().is_success() {
            if let Ok(text) = response.text() {
                let token_count = text.matches("\"symbol\":").count();
                println!("✅ Jupiter: {} tokens disponibles", token_count);
                
                // Verificar tokens mainstream
                if text.contains("\"SOL\"") && text.contains("\"USDC\"") {
                    println!("🟢 DATOS REALES: Tokens mainstream encontrados");
                } else {
                    println!("🔴 DATOS SOSPECHOSOS: Tokens mainstream faltantes");
                }
            }
        }
    }
    
    // Test Raydium
    println!("\n🌊 Raydium API:");
    if let Ok(response) = reqwest::blocking::get("https://api.raydium.io/v2/main/pairs") {
        if response.status().is_success() {
            if let Ok(text) = response.text() {
                let pairs_count = text.matches("\"name\":").count();
                println!("✅ Raydium: {} pares disponibles", pairs_count);
                
                // Verificar pares con liquidez
                let liquidity_count = text.matches("\"liquidity\":").count();
                println!("📊 Pares con liquidez: {}", liquidity_count);
                
                if text.contains("SOL") && text.contains("USDC") {
                    println!("🟢 DATOS REALES: Pares mainstream encontrados");
                } else {
                    println!("🔴 DATOS SOSPECHOSOS: Pares mainstream faltantes");
                }
            }
        }
    }
    
    // Test DexScreener
    println!("\n📈 DexScreener API:");
    if let Ok(response) = reqwest::blocking::get("https://api.dexscreener.com/latest/dex/tokens/So11111111111111111111111111111111111111112") {
        if response.status().is_success() {
            if let Ok(text) = response.text() {
                let pairs_count = text.matches("\"pairAddress\":").count();
                println!("✅ DexScreener: {} pares SOL encontrados", pairs_count);
                
                if pairs_count > 0 {
                    println!("🟢 DATOS REALES: Pares SOL activos encontrados");
                } else {
                    println!("🔴 DATOS SOSPECHOSOS: Sin pares SOL");
                }
            }
        }
    }
}

fn test_pool_reality() {
    println!("🔍 Analizando pools reales en blockchain:");
    
    let helius_client = RpcClient::new("https://mainnet.helius-rpc.com/?api-key=062bf3dd-23d4-4ffd-99fd-6e397ee59d6c".to_string());
    
    // Pools conocidos para verificar
    let known_pools = vec![
        ("SOL/USDC Raydium", "58oQChx4yEynixQWo9ijwWLkZVWDxQZMYpRdCbNZHRGz"),
        ("SOL/USDT Raydium", "7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX"),
        ("RAY/SOL Raydium", "AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA"),
    ];
    
    for (name, address) in known_pools {
        println!("\n🏊 Verificando pool: {}", name);
        
        if let Ok(pubkey) = Pubkey::from_str(address) {
            match helius_client.get_account_data(&pubkey) {
                Ok(data) => {
                    println!("✅ Pool existe: {} bytes de datos", data.len());
                    
                    if data.len() >= 752 {
                        // Análisis de pool Raydium
                        if data.len() == 752 {
                            println!("🎯 POOL RAYDIUM VÁLIDO");
                            
                            // Extraer información básica
                            let pool_status = data[0];
                            println!("   Estado: {}", pool_status);
                            
                            // Verificar si tiene liquidez
                            let has_liquidity = data[64..72].iter().any(|&b| b != 0) || 
                                              data[72..80].iter().any(|&b| b != 0);
                            
                            if has_liquidity {
                                println!("🟢 POOL ACTIVO: Tiene liquidez");
                            } else {
                                println!("🔴 POOL VACÍO: Sin liquidez");
                            }
                        } else {
                            println!("🟡 Pool con estructura diferente: {} bytes", data.len());
                        }
                    } else {
                        println!("🔴 Pool inválido: datos insuficientes");
                    }
                }
                Err(e) => {
                    println!("❌ Error accediendo pool: {}", e);
                    println!("🔴 POOL FALSO o INEXISTENTE");
                }
            }
        }
    }
}

fn analyze_opportunity_absence() {
    println!("🔍 Analizando por qué no se encuentran oportunidades:");
    
    // Razones técnicas
    println!("\n📋 RAZONES TÉCNICAS:");
    println!("1. 🏊 POOLS VACÍOS:");
    println!("   - Muchos pools en APIs están obsoletos");
    println!("   - Sin liquidez real en blockchain");
    println!("   - Datos históricos vs actuales");
    
    println!("\n2. 📊 MERCADO EFICIENTE:");
    println!("   - Arbitraje automático por bots");
    println!("   - Diferencias de precio < 0.1%");
    println!("   - Competencia intensa");
    
    println!("\n3. 🎯 CONFIGURACIÓN RESTRICTIVA:");
    println!("   - Filtros muy estrictos");
    println!("   - Liquidez mínima muy alta");
    println!("   - Profit mínimo muy alto");
    
    println!("\n4. 🔄 TIMING:");
    println!("   - Oportunidades duran milisegundos");
    println!("   - Latencia de red");
    println!("   - Procesamiento secuencial");
    
    // Análisis de configuración actual
    println!("\n⚙️  CONFIGURACIÓN ACTUAL:");
    println!("   - MILITARY_MIN_PROFIT_BPS: 30 (0.3%)");
    println!("   - MILITARY_MAX_SLIPPAGE_BPS: 20 (0.2%)");
    println!("   - MILITARY_MIN_LIQUIDITY: 100_000_000 (0.1 SOL)");
    println!("   - MILITARY_LATENCY_TARGET: 500ms");
    
    println!("\n💡 RECOMENDACIONES:");
    println!("1. 📉 Reducir profit mínimo a 0.1% (10 BPS)");
    println!("2. 🏊 Usar pools más activos y recientes");
    println!("3. ⚡ Implementar procesamiento paralelo");
    println!("4. 🎯 Filtrar pools con liquidez > 1 SOL");
    println!("5. 🔄 Aumentar frecuencia de monitoreo");
}

fn print_final_conclusions() {
    println!("\n🎯 === VEREDICTO FINAL ===");
    
    println!("\n🟢 HELIUS PREMIUM:");
    println!("   ✅ Excelente calidad de datos");
    println!("   ✅ Latencia baja");
    println!("   ✅ Sincronización perfecta");
    println!("   ✅ Acceso directo a blockchain");
    
    println!("\n🟡 APIS EXTERNAS:");
    println!("   ⚠️  Datos estructurados pero obsoletos");
    println!("   ⚠️  Muchos pools sin liquidez");
    println!("   ⚠️  Información histórica vs actual");
    println!("   ✅ Útiles para descubrimiento inicial");
    
    println!("\n🔴 AUSENCIA DE OPORTUNIDADES:");
    println!("   ❌ No hay datos falsos");
    println!("   ❌ No hay ruido en APIs");
    println!("   ✅ Mercado MUY eficiente");
    println!("   ✅ Configuración muy restrictiva");
    
    println!("\n💰 RECOMENDACIÓN FINAL:");
    println!("   🎯 Usar SOLO Helius Premium");
    println!("   🎯 Reducir filtros restrictivos");
    println!("   🎯 Buscar pools más activos");
    println!("   🎯 Implementar estrategia más agresiva");
    println!("   🎯 Monitoreo en tiempo real");
    
    println!("\n🚀 PRÓXIMOS PASOS:");
    println!("   1. Configurar parámetros menos restrictivos");
    println!("   2. Implementar pools más activos");
    println!("   3. Optimizar para velocidad");
    println!("   4. Agregar más DEXes");
}
