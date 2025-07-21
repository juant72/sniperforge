// Verificación profunda de tokens nativos de DevNet
// Investigar si los tokens son realmente de DevNet o cross-network

use reqwest;
use serde_json::Value;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 DEEP INVESTIGATION: DevNet Native vs Cross-Network Tokens");
    println!("===============================================================");

    let client = reqwest::Client::new();

    // Test tokens que encontramos "funcionando"
    let test_scenarios = vec![
        (
            "MainNet USDC on DevNet",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "This is MainNet USDC",
        ),
        (
            "MainNet USDT on DevNet",
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            "This is MainNet USDT",
        ),
        (
            "MainNet RAY on DevNet",
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
            "This is MainNet RAY",
        ),
        (
            "DevNet USDC (old)",
            "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
            "Supposedly DevNet-specific USDC",
        ),
        (
            "Fake DevNet token",
            "11111111111111111111111111111111",
            "System Program (invalid)",
        ),
    ];

    let sol_mint = "So11111111111111111111111111111111111111112";
    let test_amount = 100_000; // 0.0001 SOL

    for (name, token_address, description) in test_scenarios.iter() {
        println!("\n🧪 Testing: {}", name);
        println!("   Address: {}", token_address);
        println!("   Description: {}", description);

        // Test 1: Jupiter Quote API
        let quote_url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}",
            sol_mint, token_address, test_amount
        );

        print!("   📊 Jupiter Quote API: ");
        match client.get(&quote_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(json) => {
                            if json.get("error").is_some() {
                                println!("❌ Error: {}", json);
                            } else {
                                let out_amount = json
                                    .get("outAmount")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("0");
                                println!("✅ Quote: {} output tokens", out_amount);

                                // Analizar la ruta para ver qué DEXes se usan
                                if let Some(route_plan) = json.get("routePlan") {
                                    if let Some(route_array) = route_plan.as_array() {
                                        println!("      Route ({} steps):", route_array.len());
                                        for step in route_array.iter() {
                                            if let Some(swap_info) = step.get("swapInfo") {
                                                let label = swap_info
                                                    .get("label")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("Unknown");
                                                let amm_key = swap_info
                                                    .get("ammKey")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("Unknown");
                                                println!("        • {} ({})", label, &amm_key[..8]);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => println!("❌ Failed to parse response"),
                    }
                } else {
                    println!("❌ HTTP {}", response.status());
                }
            }
            Err(_) => println!("❌ Request failed"),
        }

        // Test 2: Check if token exists on DevNet RPC
        print!("   🌐 DevNet RPC Check: ");
        let rpc_url = "https://api.devnet.solana.com";
        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAccountInfo",
            "params": [
                token_address,
                {
                    "encoding": "jsonParsed"
                }
            ]
        });

        match client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .json(&rpc_request)
            .send()
            .await
        {
            Ok(response) => {
                match response.json::<Value>().await {
                    Ok(json) => {
                        if let Some(result) = json.get("result") {
                            if let Some(value) = result.get("value") {
                                if value.is_null() {
                                    println!("❌ Account does not exist on DevNet");
                                } else {
                                    println!("✅ Account exists on DevNet");

                                    // Try to get token info
                                    if let Some(data) = value.get("data") {
                                        if let Some(parsed) = data.get("parsed") {
                                            if let Some(info) = parsed.get("info") {
                                                if let Some(supply) = info.get("supply") {
                                                    println!("        Supply: {}", supply);
                                                }
                                                if let Some(decimals) = info.get("decimals") {
                                                    println!("        Decimals: {}", decimals);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => println!("❌ Failed to parse RPC response"),
                }
            }
            Err(_) => println!("❌ RPC request failed"),
        }

        println!("   {}", "─".repeat(50));
    }

    println!("\n🎯 ANALYSIS SUMMARY:");
    println!("====================================");
    println!("✅ = Token works in Jupiter and exists on DevNet RPC");
    println!("⚠️  = Token works in Jupiter but may be cross-network");
    println!("❌ = Token doesn't work or doesn't exist");
    println!("");
    println!("💡 RECOMMENDATION:");
    println!("• Use tokens that are ✅ on BOTH Jupiter and DevNet RPC");
    println!("• Avoid cross-network tokens for real DevNet trading");
    println!("• Look for DevNet-specific token programs and mints");

    Ok(())
}
