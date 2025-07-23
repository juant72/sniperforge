use anyhow::Result;
use sniperforge::shared::jupiter_client::JupiterClient;
use sniperforge::shared::jupiter_config::JupiterConfig;
use sniperforge::shared::jupiter_types::QuoteRequest;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🔍 === Descubriendo tokens disponibles en Jupiter DevNet ===");
    info!("==========================================================");

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

    // Test known tokens directly
    info!("� Probando tokens conocidos en DevNet...");
    test_known_devnet_tokens(&client).await?;

    Ok(())
}

async fn test_known_devnet_tokens(client: &JupiterClient) -> Result<()> {
    info!("🧪 Probando tokens conocidos de DevNet:");

    let known_tokens = vec![
        (
            "So11111111111111111111111111111111111111112",
            "SOL (Native)",
        ),
        (
            "So11111111111111111111111111111111111111112",
            "wSOL (Wrapped)",
        ),
        // DevNet USDC equivalent (if exists)
        (
            "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
            "USDC (DevNet)",
        ),
        // DevNet test tokens that might exist
        (
            "GugU1tP7doLeTw9hQP51xRJyS8Da1fWxuiy2rVrnMD2m",
            "Test Token 1",
        ),
        (
            "34XMHa3JUPv46ftU4dGHvemZ9oKVjnciRePYMcX3rjEF",
            "Test Token 2",
        ),
    ];

    for (address, name) in known_tokens {
        info!("  🔍 Probando {}: {}", name, address);

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
            }
            Err(e) => {
                info!("    ❌ Token no disponible: {}", e);
            }
        }
    }

    // Test SOL wrapping/unwrapping specifically
    info!("\n🔄 Probando SOL wrapping/unwrapping específicamente:");

    // SOL -> wSOL (should work)
    let wrap_request = QuoteRequest {
        inputMint: "So11111111111111111111111111111111111111112".to_string(),
        outputMint: "So11111111111111111111111111111111111111112".to_string(),
        amount: 1_000_000, // 0.001 SOL
        slippageBps: 0,    // No slippage for wrapping
    };

    match client.get_quote(wrap_request).await {
        Ok(quote) => {
            info!("  ✅ SOL wrapping disponible");
            info!("     Output: {} units", quote.outAmount);
        }
        Err(e) => {
            info!("  ❌ SOL wrapping falló: {}", e);
        }
    }

    info!("\n💡 === RECOMENDACIONES PARA ARBITRAJE DEVNET ===");
    info!("1. Usar SOL wrapping/unwrapping como test básico");
    info!("2. Buscar pools de Orca DevNet que tengan liquidez real");
    info!("3. Crear pools propios con tokens SPL custom");
    info!("4. Usar simulación para testing mientras se encuentra liquidez real");
    info!("\n🎯 === PRÓXIMOS PASOS ===");
    info!("1. Si SOL wrapping funciona, usar como base para arbitraje");
    info!("2. Integrar con Orca para encontrar pools con liquidez");
    info!("3. Crear test de arbitraje real con tokens que funcionan");

    Ok(())
}
