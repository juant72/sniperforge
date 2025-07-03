// Test para verificar qué tokens están disponibles en DevNet
// y encontrar un par de trading que funcione realmente

use reqwest;
use serde_json::Value;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Investigating available tokens on DevNet...");
    
    // Test 1: Ver qué tokens devuelve Jupiter en DevNet para SOL
    let client = reqwest::Client::new();
    
    // Probar diferentes tokens de salida para SOL
    let test_tokens = vec![
        ("USDC-DevNet", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        ("USDC-Test", "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"), // DevNet USDC conocido
        ("RAY-MainNet", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"),
        ("SOL", "So11111111111111111111111111111111111111112"),
    ];
    
    let sol_mint = "So11111111111111111111111111111111111111112";
    let amount = 10000; // 0.00001 SOL
    
    for (name, token_address) in test_tokens.iter() {
        if *token_address == sol_mint {
            continue; // Skip SOL to SOL
        }
        
        println!("\n📊 Testing {} ({})", name, token_address);
        
        let quote_url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=300",
            sol_mint, token_address, amount
        );
        
        match client.get(&quote_url).send().await {
            Ok(response) => {
                let status = response.status();
                println!("   Status: {}", status);
                
                if status.is_success() {
                    match response.json::<Value>().await {
                        Ok(json) => {
                            if json.get("error").is_some() {
                                println!("   ❌ Error: {}", json.get("error").unwrap());
                            } else {
                                let out_amount = json.get("outAmount").and_then(|v| v.as_str()).unwrap_or("unknown");
                                println!("   ✅ Quote available! Output: {}", out_amount);
                                
                                // Ver la ruta
                                if let Some(route_plan) = json.get("routePlan") {
                                    println!("   📍 Route: {} steps", route_plan.as_array().map(|a| a.len()).unwrap_or(0));
                                }
                            }
                        }
                        Err(e) => println!("   ❌ Failed to parse JSON: {}", e),
                    }
                } else {
                    let text = response.text().await.unwrap_or_else(|_| "unknown".to_string());
                    println!("   ❌ HTTP Error: {}", text);
                }
            }
            Err(e) => println!("   ❌ Request failed: {}", e),
        }
    }
    
    println!("\n🎯 Recommendation:");
    println!("Use a token that has ✅ Quote available for real swaps on DevNet");
    
    Ok(())
}
