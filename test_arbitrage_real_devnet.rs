use anyhow::Result;
use sniperforge::shared::jupiter_client::JupiterClient;
use sniperforge::shared::jupiter_config::JupiterConfig;
use sniperforge::shared::jupiter_types::QuoteRequest;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === Probando Arbitraje Real en DevNet ===");
    info!("===========================================");

    // Create Jupiter config for DevNet
    let config = JupiterConfig {
        base_url: "https://quote-api.jup.ag".to_string(),
        api_key: None,
        timeout_seconds: 30,
        max_retries: 3,
        rpc_endpoint: "https://solana-devnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg".to_string(),
        network_name: "devnet".to_string(),
    };

    let client = JupiterClient::new(&config).await?;

    // Test well-known DevNet tokens that might have liquidity
    info!("🔍 Probando tokens conocidos de DevNet que podrían tener liquidez...");

    let devnet_tokens = vec![
        // Well-known DevNet tokens
        ("So11111111111111111111111111111111111111112", "SOL"),
        (
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "USDC-MainNet",
        ), // Might exist in DevNet
        (
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
            "RAY-MainNet",
        ), // Might exist in DevNet
        (
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            "BONK-MainNet",
        ), // Might exist in DevNet
        // DevNet specific test tokens
        (
            "2iEJ8jXQN9xmhA7w9SpVgFGNkCYEsEf8LfEFumKjNRQo",
            "Custom-USDC",
        ),
        ("7QT4DzmTUfE3jTDHcmCPD9AHHRjkLGc9wKNp5JEJiSXW", "Custom-RAY"),
        (
            "8J9FV7yMbKrJ8jKpfzBvFUNn5jGNZjhRCfbhVoMCqgbZ",
            "Custom-Test",
        ),
    ];

    let mut tradable_tokens = Vec::new();

    for (address, symbol) in devnet_tokens {
        if address == "So11111111111111111111111111111111111111112" {
            // Skip SOL as we know it can't trade with itself
            continue;
        }

        info!("  🔍 Probando {}: {}", symbol, address);

        // Try to get a quote for 0.001 SOL -> this token
        let quote_request = QuoteRequest {
            inputMint: "So11111111111111111111111111111111111111112".to_string(),
            outputMint: address.to_string(),
            amount: 1_000_000, // 0.001 SOL (9 decimals)
            slippageBps: 100,  // 1% slippage
        };

        match client.get_quote(quote_request).await {
            Ok(quote) => {
                info!("    ✅ Token disponible para trading");
                info!("       Output: {} units", quote.outAmount);
                tradable_tokens.push((address, symbol, quote.outAmount.clone()));
            }
            Err(e) => {
                info!("    ❌ Token no disponible: {}", e);
            }
        }
    }

    if tradable_tokens.is_empty() {
        warn!("❌ No se encontraron tokens comerciables en DevNet");
        info!("💡 Recomendación: Usar Orca DevNet para crear pools con liquidez");
        return Ok(());
    }

    info!("\n🎯 === TOKENS COMERCIABLES ENCONTRADOS ===");
    for (address, symbol, output) in &tradable_tokens {
        info!("✅ {}: {} (Output: {})", symbol, address, output);
    }

    // Now test potential arbitrage between found tokens
    info!("\n🔄 === PROBANDO ARBITRAJE POTENCIAL ===");

    if tradable_tokens.len() >= 2 {
        let token1 = &tradable_tokens[0];
        let token2 = &tradable_tokens[1];

        info!("🔄 Probando arbitraje: {} -> {} -> SOL", token1.1, token2.1);

        // Step 1: SOL -> Token1
        let quote1 = QuoteRequest {
            inputMint: "So11111111111111111111111111111111111111112".to_string(),
            outputMint: token1.0.to_string(),
            amount: 10_000_000, // 0.01 SOL
            slippageBps: 100,
        };

        match client.get_quote(quote1).await {
            Ok(quote_response1) => {
                info!("  ✅ Paso 1: SOL -> {} exitoso", token1.1);

                // Step 2: Token1 -> Token2
                let token1_amount: u64 = quote_response1.outAmount.parse().unwrap_or(0);
                let quote2 = QuoteRequest {
                    inputMint: token1.0.to_string(),
                    outputMint: token2.0.to_string(),
                    amount: token1_amount,
                    slippageBps: 100,
                };

                match client.get_quote(quote2).await {
                    Ok(quote_response2) => {
                        info!("  ✅ Paso 2: {} -> {} exitoso", token1.1, token2.1);

                        // Step 3: Token2 -> SOL
                        let token2_amount: u64 = quote_response2.outAmount.parse().unwrap_or(0);
                        let quote3 = QuoteRequest {
                            inputMint: token2.0.to_string(),
                            outputMint: "So11111111111111111111111111111111111111112".to_string(),
                            amount: token2_amount,
                            slippageBps: 100,
                        };

                        match client.get_quote(quote3).await {
                            Ok(quote_response3) => {
                                let final_sol: u64 = quote_response3.outAmount.parse().unwrap_or(0);
                                let initial_sol = 10_000_000u64;

                                info!("  ✅ Paso 3: {} -> SOL exitoso", token2.1);
                                info!("  📊 Resultado del arbitraje:");
                                info!("     SOL inicial: {} lamports", initial_sol);
                                info!("     SOL final:   {} lamports", final_sol);

                                if final_sol > initial_sol {
                                    let profit = final_sol - initial_sol;
                                    info!(
                                        "     💰 GANANCIA: {} lamports ({:.6} SOL)",
                                        profit,
                                        profit as f64 / 1_000_000_000.0
                                    );
                                } else {
                                    let loss = initial_sol - final_sol;
                                    info!(
                                        "     💸 PÉRDIDA: {} lamports ({:.6} SOL)",
                                        loss,
                                        loss as f64 / 1_000_000_000.0
                                    );
                                }
                            }
                            Err(e) => {
                                info!("  ❌ Paso 3 falló: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        info!("  ❌ Paso 2 falló: {}", e);
                    }
                }
            }
            Err(e) => {
                info!("  ❌ Paso 1 falló: {}", e);
            }
        }
    }

    info!("\n💡 === CONCLUSIONES ===");
    info!(
        "1. Tokens comerciables encontrados: {}",
        tradable_tokens.len()
    );
    info!("2. Para arbitraje real, necesitamos al menos 2 tokens con liquidez");
    info!("3. Próximo paso: Integrar con Orca para encontrar más oportunidades");

    Ok(())
}
