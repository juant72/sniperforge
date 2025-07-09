use anyhow::Result;
use sniperforge::shared::jupiter_client::JupiterClient;
use sniperforge::shared::jupiter_config::JupiterConfig;
use tracing::info;
use serde_json::Value;

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

    // Get available tokens on DevNet
    info!("📡 Consultando tokens disponibles en DevNet...");

    match client.get_tokens().await {
        Ok(tokens_response) => {
            info!("✅ Lista de tokens obtenida exitosamente");
            
            // Parse the response to get token information
            if let Value::Array(tokens) = tokens_response {
                info!("📊 Total de tokens encontrados: {}", tokens.len());
                
                let mut devnet_tokens = Vec::new();
                
                for (i, token) in tokens.iter().enumerate() {
                    if i >= 20 { // Limit to first 20 tokens for display
                        break;
                    }
                    
                    if let Value::Object(token_obj) = token {
                        let address = token_obj.get("address")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let symbol = token_obj.get("symbol")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let name = token_obj.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let decimals = token_obj.get("decimals")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        
                        devnet_tokens.push((address, symbol, name, decimals));
                        
                        info!("  [{}] {} ({}) - {} decimals", 
                              i + 1, symbol, name, decimals);
                        info!("      Address: {}", address);
                    }
                }
                
                info!("\n🎯 === TOKENS RECOMENDADOS PARA ARBITRAJE ===");
                
                // Look for common tokens that might have liquidity
                let recommended_tokens: Vec<(&str, &str, &str, u64)> = devnet_tokens.iter()
                    .filter(|(_, symbol, _, _)| {
                        let s = symbol.to_lowercase();
                        s.contains("sol") || s.contains("usdc") || s.contains("usdt") || 
                        s.contains("ray") || s.contains("bonk") || s.contains("test")
                    })
                    .cloned()
                    .collect();
                
                if recommended_tokens.is_empty() {
                    info!("⚠️  No se encontraron tokens comunes en DevNet");
                    info!("💡 Sugerencia: Usar SOL para wrapping/unwrapping como test básico");
                } else {
                    for (address, symbol, name, decimals) in recommended_tokens {
                        info!("✅ {}: {} ({} decimals)", symbol, name, decimals);
                        info!("   Address: {}", address);
                    }
                }
                
            } else {
                info!("⚠️  Formato de respuesta inesperado");
            }
        }
        Err(e) => {
            info!("❌ Error obteniendo tokens: {}", e);
            info!("💡 Esto puede ser normal en DevNet - Jupiter puede tener tokens limitados");
            
            // Let's try to test with known DevNet tokens
            info!("\n🔄 Probando con tokens conocidos de DevNet...");
            test_known_devnet_tokens(&client).await?;
        }
    }

    Ok(())
}

async fn test_known_devnet_tokens(client: &JupiterClient) -> Result<()> {
    info!("🧪 Probando tokens conocidos de DevNet:");
    
    let known_tokens = vec![
        ("So11111111111111111111111111111111111111112", "SOL (Native)"),
        ("So11111111111111111111111111111111111111112", "wSOL (Wrapped)"),
        // DevNet USDC equivalent (if exists)
        ("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU", "USDC (DevNet)"),
        // DevNet test tokens that might exist
        ("GugU1tP7doLeTw9hQP51xRJyS8Da1fWxuiy2rVrnMD2m", "Test Token 1"),
        ("34XMHa3JUPv46ftU4dGHvemZ9oKVjnciRePYMcX3rjEF", "Test Token 2"),
    ];
    
    for (address, name) in known_tokens {
        info!("  🔍 Probando {}: {}", name, address);
        
        // Try to get a quote for 0.001 SOL -> this token
        match client.get_quote(
            "So11111111111111111111111111111111111111112", // SOL
            address,
            0.001,
            100 // 1% slippage
        ).await {
            Ok(_) => {
                info!("    ✅ Token disponible para trading");
            }
            Err(e) => {
                info!("    ❌ Token no disponible: {}", e);
            }
        }
    }
    
    info!("\n💡 === RECOMENDACIONES PARA ARBITRAJE DEVNET ===");
    info!("1. Usar SOL wrapping/unwrapping como test básico");
    info!("2. Buscar pools de Orca DevNet que tengan liquidez real");
    info!("3. Crear pools propios con tokens SPL custom");
    info!("4. Usar simulación para testing mientras se encuentra liquidez real");
    
    Ok(())
}
