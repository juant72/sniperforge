use anyhow::Result;
use tracing::{info, warn, error};
use sniperforge::shared::jupiter_client::JupiterClient;
use sniperforge::shared::jupiter_config::JupiterConfig;
use sniperforge::shared::jupiter_types::QuoteRequest;
use solana_sdk::{signature::Keypair, signer::Signer};
use solana_client::rpc_client::RpcClient;
use std::fs;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🚀 === EJECUTOR DE ARBITRAJE REAL SIMPLE ===");
    info!("===============================================");
    
    // Configuración DevNet
    let rpc_url = "https://api.devnet.solana.com";
    let sol_mint = "So11111111111111111111111111111111111111112";
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC que funciona
    let amount = 10_000_000u64; // 0.01 SOL
    
    // Cargar wallet desde variable de entorno (como los comandos binarios)
    info!("🔑 Cargando wallet desde variable de entorno...");
    let private_key = env::var("SOLANA_PRIVATE_KEY")
        .map_err(|_| anyhow::anyhow!("SOLANA_PRIVATE_KEY not found"))?;
    
    let wallet_bytes: Vec<u8> = serde_json::from_str(&private_key)?;
    let keypair = Keypair::from_bytes(&wallet_bytes)?;
    let wallet_pubkey = keypair.pubkey();
    
    info!("💼 Wallet address: {}", wallet_pubkey);
    
    // Cliente RPC y balance inicial
    let rpc_client = RpcClient::new(rpc_url);
    let balance_inicial = rpc_client.get_balance(&wallet_pubkey)?;
    info!("💰 BALANCE INICIAL: {} SOL ({} lamports)", 
          balance_inicial as f64 / 1_000_000_000.0, balance_inicial);
    
    // Configurar Jupiter
    let jupiter_config = JupiterConfig {
        base_url: "https://quote-api.jup.ag".to_string(),
        api_key: None,
        timeout_seconds: 30,
        max_retries: 3,
        rpc_endpoint: rpc_url.to_string(),
        network_name: "devnet".to_string(),
    };
    let jupiter_client = JupiterClient::new(&jupiter_config).await?;
    
    info!("\n📊 === EJECUTANDO SWAP REAL ===");
    info!("🔄 Swap: {} SOL -> USDC -> SOL", amount as f64 / 1_000_000_000.0);
    
    // PASO 1: SOL -> USDC
    info!("🔄 PASO 1: SOL -> USDC");
    let quote_request = QuoteRequest {
        inputMint: sol_mint.to_string(),
        outputMint: usdc_mint.to_string(),
        amount,
        slippageBps: 100,
    };
    
    let quote_response = jupiter_client.get_quote(quote_request).await?;
    let usdc_amount: u64 = quote_response.outAmount.parse().unwrap_or(0);
    info!("✅ Quote obtenido: {} SOL -> {} USDC", 
          amount as f64 / 1_000_000_000.0, usdc_amount as f64 / 1_000_000.0);
    
    // Ejecutar swap REAL SOL -> USDC
    match jupiter_client.execute_swap_with_wallet(
        &quote_response,
        &wallet_pubkey.to_string(),
        Some(&keypair)
    ).await {
        Ok(result) => {
            info!("✅ SWAP 1 EJECUTADO: SOL -> USDC");
            info!("🔗 Resultado: {:?}", result);
            
            // Esperar confirmación
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            // PASO 2: USDC -> SOL
            info!("🔄 PASO 2: USDC -> SOL");
            let quote_request_2 = QuoteRequest {
                inputMint: usdc_mint.to_string(),
                outputMint: sol_mint.to_string(),
                amount: usdc_amount,
                slippageBps: 100,
            };
            
            let quote_response_2 = jupiter_client.get_quote(quote_request_2).await?;
            let final_sol_amount: u64 = quote_response_2.outAmount.parse().unwrap_or(0);
            info!("✅ Quote obtenido: {} USDC -> {} SOL", 
                  usdc_amount as f64 / 1_000_000.0, final_sol_amount as f64 / 1_000_000_000.0);
            
            // Ejecutar swap REAL USDC -> SOL
            match jupiter_client.execute_swap_with_wallet(
                &quote_response_2,
                &wallet_pubkey.to_string(),
                Some(&keypair)
            ).await {
                Ok(result_2) => {
                    info!("✅ SWAP 2 EJECUTADO: USDC -> SOL");
                    info!("🔗 Resultado: {:?}", result_2);
                    
                    // Esperar confirmación final
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    
                    // MEDICIÓN FINAL
                    info!("\n📊 === RESULTADO FINAL ===");
                    let balance_final = rpc_client.get_balance(&wallet_pubkey)?;
                    let cambio = balance_final as i64 - balance_inicial as i64;
                    
                    info!("💰 BALANCE INICIAL:  {} SOL ({} lamports)", 
                          balance_inicial as f64 / 1_000_000_000.0, balance_inicial);
                    info!("💰 BALANCE FINAL:    {} SOL ({} lamports)", 
                          balance_final as f64 / 1_000_000_000.0, balance_final);
                    info!("📊 CAMBIO:           {} lamports", cambio);
                    
                    if cambio > 0 {
                        info!("🎉 ¡ARBITRAJE EXITOSO! +{} lamports (+{:.9} SOL)", 
                              cambio, cambio as f64 / 1_000_000_000.0);
                        info!("💰 GANANCIA REAL DEMOSTRADA");
                    } else if cambio < 0 {
                        warn!("📉 Pérdida: {} lamports ({:.9} SOL)", 
                              cambio, cambio as f64 / 1_000_000_000.0);
                    } else {
                        info!("➖ Sin cambio (empate)");
                    }
                }
                Err(e) => {
                    error!("❌ Error en SWAP 2: {}", e);
                }
            }
        }
        Err(e) => {
            error!("❌ Error en SWAP 1: {}", e);
        }
    }
    
    Ok(())
}
