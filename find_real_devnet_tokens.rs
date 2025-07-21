// Búsqueda de tokens REALES de DevNet
// Usar programas conocidos de DevNet y tokens de prueba

use reqwest;
use serde_json::Value;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 SEARCHING FOR REAL DEVNET TOKENS");
    println!("====================================");

    let client = reqwest::Client::new();
    let sol_mint = "So11111111111111111111111111111111111111112";

    // Tokens conocidos específicos de DevNet o que deberían existir
    let devnet_candidates = vec![
        // Wrapped SOL es universal
        (
            "Wrapped SOL",
            "So11111111111111111111111111111111111111112",
            "SOL wrapper - should exist everywhere",
        ),
        // Test tokens comunes de DevNet (estos son direcciones reales de test tokens)
        (
            "DevNet Test Token 1",
            "GWKHQnbFNxSU9bZo1DhKgFjMhgxcP5H9F5eCYP3KmH7M",
            "Common DevNet test token",
        ),
        (
            "DevNet Test Token 2",
            "HuKfLEvGS6WfZa3ZJHXFJQkPKrn8xBDyNJ7Bn9XsHU8H",
            "Another DevNet test token",
        ),
        // Buscar en known DevNet programs
        (
            "DevNet USDC Mock",
            "FNuFazLh8o1vH1W1VB1X5GgV5V8Zo9mJ5YQ5XhQ1mZXe",
            "Mock USDC for DevNet",
        ),
        // Jupiter's own test tokens on DevNet
        (
            "Jupiter Test A",
            "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "Jupiter test token A",
        ),
        (
            "Jupiter Test B",
            "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
            "Jupiter test token B",
        ),
    ];

    let test_amount = 100_000; // 0.0001 SOL

    for (name, token_address, description) in devnet_candidates.iter() {
        println!("\n🧪 Testing: {}", name);
        println!("   Address: {}", token_address);
        println!("   Description: {}", description);

        // Test 1: DevNet RPC Check FIRST
        print!("   🌐 DevNet RPC: ");
        let rpc_url = "https://api.devnet.solana.com";
        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAccountInfo",
            "params": [
                token_address,
                {"encoding": "jsonParsed"}
            ]
        });

        let rpc_exists = match client
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
                                    println!("❌ Does not exist");
                                    false
                                } else {
                                    println!("✅ EXISTS!");

                                    // Print token details
                                    if let Some(data) = value.get("data") {
                                        if let Some(parsed) = data.get("parsed") {
                                            if let Some(info) = parsed.get("info") {
                                                if let Some(supply) = info.get("supply") {
                                                    println!("        Supply: {}", supply);
                                                }
                                                if let Some(decimals) = info.get("decimals") {
                                                    println!("        Decimals: {}", decimals);
                                                }
                                                if let Some(mint_authority) =
                                                    info.get("mintAuthority")
                                                {
                                                    println!(
                                                        "        Mint Authority: {}",
                                                        mint_authority
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    true
                                }
                            } else {
                                println!("❌ No result value");
                                false
                            }
                        } else {
                            println!("❌ No result field");
                            false
                        }
                    }
                    Err(_) => {
                        println!("❌ RPC parse error");
                        false
                    }
                }
            }
            Err(_) => {
                println!("❌ RPC request failed");
                false
            }
        };

        // Test 2: Jupiter Quote ONLY if token exists on DevNet
        if rpc_exists {
            print!("   📊 Jupiter Quote: ");
            let quote_url = format!(
                "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}",
                sol_mint, token_address, test_amount
            );

            match client.get(&quote_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<Value>().await {
                            Ok(json) => {
                                if json.get("error").is_some() {
                                    println!("❌ Jupiter Error");
                                } else {
                                    let out_amount = json
                                        .get("outAmount")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("0");
                                    println!("✅ Quote: {} tokens", out_amount);

                                    println!(
                                        "   🎯 ** THIS TOKEN IS PERFECT FOR DEVNET TRADING **"
                                    );
                                }
                            }
                            Err(_) => println!("❌ Parse error"),
                        }
                    } else {
                        println!("❌ HTTP {}", response.status());
                    }
                }
                Err(_) => println!("❌ Request failed"),
            }
        } else {
            println!("   📊 Jupiter Quote: Skipped (token doesn't exist on DevNet)");
        }

        println!("   {}", "─".repeat(50));
    }

    println!("\n🎯 FINAL RECOMMENDATION:");
    println!("====================================");
    println!("✅ Use ONLY tokens that:");
    println!("   1. ✅ Exist on DevNet RPC");
    println!("   2. ✅ Have Jupiter quotes available");
    println!("");
    println!("❌ AVOID tokens that only work in Jupiter but don't exist on DevNet RPC");
    println!("   (These cause 'IncorrectProgramId' errors during real execution)");

    Ok(())
}
