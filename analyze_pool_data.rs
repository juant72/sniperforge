use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() {
    println!("🔍 === ANÁLISIS DETALLADO DE POOLS ===");
    
    // Test con diferentes RPCs
    let rpcs = vec![
        ("Helius Premium", "https://mainnet.helius-rpc.com/?api-key=062bf3dd-23d4-4ffd-99fd-6e397ee59d6c"),
        ("Alchemy Premium", "https://solana-mainnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg"),
        ("Solana Public", "https://api.mainnet-beta.solana.com"),
    ];
    
    for (name, url) in rpcs {
        println!("\n🌐 Testing {}: {}", name, url);
        
        let client = RpcClient::new(url.to_string());
        
        // Test básico de conectividad
        match client.get_slot() {
            Ok(slot) => {
                println!("✅ {}: Slot actual = {}", name, slot);
                
                // Test específico de pool conocido
                let pool_address = "58oQChx4yEynixQWo9ijwWLkZVWDxQZMYpRdCbNZHRGz";
                match Pubkey::from_str(pool_address) {
                    Ok(pubkey) => {
                        match client.get_account_data(&pubkey) {
                            Ok(data) => {
                                println!("✅ {}: Pool data size = {} bytes", name, data.len());
                                
                                // Análisis básico de datos
                                if data.len() > 0 {
                                    println!("   📊 Pool data: {:02x?}...", &data[..std::cmp::min(32, data.len())]);
                                    
                                    // Si es un pool de Raydium, analizar estructura
                                    if data.len() >= 752 {
                                        println!("   🎯 Raydium pool structure detected");
                                        
                                        // Extraer token amounts (aproximado)
                                        let token_a_amount = u64::from_le_bytes([
                                            data[64], data[65], data[66], data[67], 
                                            data[68], data[69], data[70], data[71]
                                        ]);
                                        let token_b_amount = u64::from_le_bytes([
                                            data[72], data[73], data[74], data[75], 
                                            data[76], data[77], data[78], data[79]
                                        ]);
                                        
                                        println!("   💰 Token A: {} lamports", token_a_amount);
                                        println!("   💰 Token B: {} lamports", token_b_amount);
                                        
                                        if token_a_amount > 0 && token_b_amount > 0 {
                                            println!("   🟢 POOL ACTIVO con liquidez real");
                                        } else {
                                            println!("   🔴 POOL VACÍO o con liquidez cero");
                                        }
                                    }
                                } else {
                                    println!("   ⚠️  Pool data está vacío");
                                }
                            }
                            Err(e) => println!("❌ {}: Error obteniendo pool data: {}", name, e),
                        }
                    }
                    Err(e) => println!("❌ {}: Error parsing pool address: {}", name, e),
                }
            }
            Err(e) => println!("❌ {}: Error de conectividad: {}", name, e),
        }
    }
    
    println!("\n🔍 === ANÁLISIS DE CALIDAD DE DATOS ===");
    
    // Verificar Jupiter API
    println!("\n🪐 Testing Jupiter API...");
    let jupiter_url = "https://quote-api.jup.ag/v6/tokens";
    match reqwest::blocking::get(jupiter_url) {
        Ok(response) => {
            println!("✅ Jupiter API: Status {}", response.status());
            if let Ok(text) = response.text() {
                let tokens_count = text.matches("\"symbol\":").count();
                println!("   📊 Jupiter tokens found: {}", tokens_count);
                
                // Verificar si hay tokens mainstream
                if text.contains("\"SOL\"") && text.contains("\"USDC\"") {
                    println!("   🟢 Tokens mainstream detectados (SOL, USDC)");
                } else {
                    println!("   🔴 Tokens mainstream no encontrados");
                }
            }
        }
        Err(e) => println!("❌ Jupiter API error: {}", e),
    }
    
    // Verificar Raydium API
    println!("\n🌊 Testing Raydium API...");
    let raydium_url = "https://api.raydium.io/v2/main/pairs";
    match reqwest::blocking::get(raydium_url) {
        Ok(response) => {
            println!("✅ Raydium API: Status {}", response.status());
            if let Ok(text) = response.text() {
                let pairs_count = text.matches("\"name\":").count();
                println!("   📊 Raydium pairs found: {}", pairs_count);
                
                // Verificar pares con liquidez real
                let liquidity_matches = text.matches("\"liquidity\":").count();
                println!("   💰 Pairs with liquidity data: {}", liquidity_matches);
                
                if text.contains("SOL/USDC") || text.contains("SOL-USDC") {
                    println!("   🟢 Par principal SOL/USDC detectado");
                } else {
                    println!("   🔴 Par principal SOL/USDC no encontrado");
                }
            }
        }
        Err(e) => println!("❌ Raydium API error: {}", e),
    }
    
    println!("\n🎯 === CONCLUSIONES ===");
    println!("1. Si Helius Premium funciona pero pools están vacíos: DATOS REALES");
    println!("2. Si APIs responden con tokens/pairs: DATOS ESTRUCTURADOS");
    println!("3. Si no hay oportunidades: MERCADO EFICIENTE o POOLS OBSOLETOS");
    println!("4. Recomendación: Usar pools más activos y recientes");
}
