// Test para encontrar rutas de trading que funcionen en DevNet
// Probamos diferentes tokens y cantidades

use dotenv::dotenv;
use std::env;
use reqwest;
use serde_json::Value;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    
    println!("🔍 Finding working trading routes on DevNet...");
    
    let client = reqwest::Client::new();
    let sol_mint = "So11111111111111111111111111111111111111112";
    
    // Different test scenarios
    let test_cases = vec![
        ("Small SOL->USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", 10_000), // 0.00001 SOL
        ("Medium SOL->USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", 100_000), // 0.0001 SOL  
        ("Large SOL->USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", 1_000_000), // 0.001 SOL
        ("SOL->NATIVE", "11111111111111111111111111111111", 10_000), // SOL to System Program (invalid, but let's see)
        ("Small SOL->USDT", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", 100_000), // USDT if available
        ("Small SOL->RAY", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", 100_000), // RAY token
    ];
    
    for (name, output_mint, amount) in test_cases.iter() {
        if *output_mint == sol_mint {
            continue; // Skip SOL to SOL
        }
        
        println!("\n📊 Testing {} - {} lamports", name, amount);
        println!("   From: {} (SOL)", sol_mint);
        println!("   To: {}", output_mint);
        
        let quote_url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=300",
            sol_mint, output_mint, amount
        );
        
        match client.get(&quote_url).send().await {
            Ok(response) => {
                let status = response.status();
                println!("   Status: {}", status);
                
                if status.is_success() {
                    match response.json::<Value>().await {
                        Ok(json) => {
                            if json.get("error").is_some() {
                                println!("   ❌ API Error: {}", json);
                            } else {
                                let out_amount = json.get("outAmount").and_then(|v| v.as_str()).unwrap_or("unknown");
                                let price_impact = json.get("priceImpactPct").and_then(|v| v.as_str()).unwrap_or("0");
                                
                                println!("   ✅ ROUTE FOUND!");
                                println!("      Output: {} tokens", out_amount);
                                println!("      Price Impact: {}%", price_impact);
                                
                                // Ver la ruta
                                if let Some(route_plan) = json.get("routePlan") {
                                    if let Some(route_array) = route_plan.as_array() {
                                        println!("      Route: {} steps", route_array.len());
                                        for (i, step) in route_array.iter().enumerate() {
                                            if let Some(swap_info) = step.get("swapInfo") {
                                                let amm_key = swap_info.get("ammKey").and_then(|v| v.as_str()).unwrap_or("unknown");
                                                let label = swap_info.get("label").and_then(|v| v.as_str()).unwrap_or("unknown");
                                                println!("        Step {}: {} ({})", i+1, label, amm_key);
                                            }
                                        }
                                    }
                                }
                                
                                println!("   🎯 This route should work for real swaps!");
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
    
    println!("\n🎯 Summary:");
    println!("Look for routes marked with ✅ ROUTE FOUND! - those should work for real swaps.");
    println!("Use the exact token addresses and amounts that successfully return quotes.");
    
    Ok(())
}
