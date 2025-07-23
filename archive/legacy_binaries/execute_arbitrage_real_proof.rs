use anyhow::Result;
use sniperforge::shared::jupiter_client::JupiterClient;
use sniperforge::shared::jupiter_config::JupiterConfig;
use sniperforge::shared::jupiter_types::QuoteRequest;
use sniperforge::shared::orca_client::OrcaClient;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::fs;
use std::str::FromStr;
use tokio;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("🚀 === EJECUTOR DE ARBITRAJE REAL - PRUEBA DE CONCEPTO ===");
    info!("=========================================================");

    // Configuración
    let rpc_url = "https://api.devnet.solana.com";
    let sol_mint_str = "So11111111111111111111111111111111111111112";
    let usdc_mint_str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC real que funciona en DevNet
    let arbitrage_amount = 10_000_000u64; // 0.01 SOL para prueba segura

    // Cargar wallet
    info!("🔑 Cargando wallet...");
    let wallet_data = fs::read_to_string("test-arbitrage-wallet.json")?;
    let wallet_bytes: Vec<u8> = serde_json::from_str(&wallet_data)?;
    let keypair = Keypair::from_bytes(&wallet_bytes)?;
    let wallet_pubkey = keypair.pubkey();

    info!("💼 Wallet address: {}", wallet_pubkey);

    // Cliente RPC
    let rpc_client = RpcClient::new(rpc_url);

    // PASO 1: Medir balance inicial
    let balance_inicial = rpc_client.get_balance(&wallet_pubkey)?;
    let sol_inicial = balance_inicial as f64 / 1_000_000_000.0;
    info!(
        "💰 BALANCE INICIAL: {} SOL ({} lamports)",
        sol_inicial, balance_inicial
    );

    // PASO 2: Análisis de precios
    info!("\n📊 === ANÁLISIS DE PRECIOS ===");

    let jupiter_config = JupiterConfig {
        base_url: "https://quote-api.jup.ag".to_string(),
        api_key: None,
        timeout_seconds: 30,
        max_retries: 3,
        rpc_endpoint: "https://api.devnet.solana.com".to_string(),
        network_name: "devnet".to_string(),
    };
    let jupiter_client = JupiterClient::new(&jupiter_config).await?;
    let orca_client = OrcaClient::new("devnet");

    // Precio Jupiter
    let quote_request = QuoteRequest {
        inputMint: sol_mint_str.to_string(),
        outputMint: usdc_mint_str.to_string(),
        amount: arbitrage_amount,
        slippageBps: 100,
    };

    let jupiter_quote = jupiter_client.get_quote(quote_request).await?;
    let jupiter_usdc_output: u64 = jupiter_quote.outAmount.parse().unwrap_or(0);
    let jupiter_sol_price =
        jupiter_usdc_output as f64 / 1_000_000.0 / (arbitrage_amount as f64 / 1_000_000_000.0);

    // Precio Orca
    let orca_sol_price_opt = orca_client.get_price(sol_mint_str).await?;
    let orca_sol_price = orca_sol_price_opt.unwrap_or(99.5);

    info!("💵 Jupiter SOL: ${:.2}", jupiter_sol_price);
    info!("💵 Orca SOL: ${:.2}", orca_sol_price);

    let spread = ((jupiter_sol_price - orca_sol_price) / orca_sol_price) * 100.0;
    let profit_estimado = jupiter_sol_price - orca_sol_price;

    info!("📈 Spread: {:.2}%", spread);
    info!("💰 Profit estimado por SOL: ${:.2}", profit_estimado);

    if spread < 5.0 {
        warn!("⚠️ Spread muy bajo ({}%), abortando arbitraje", spread);
        return Ok(());
    }

    info!("✅ Spread rentable detectado, procediendo con arbitraje...");

    // PASO 3: EJECUTAR ARBITRAJE REAL
    info!("\n🎯 === EJECUTANDO ARBITRAJE REAL ===");

    // En DevNet, Orca es mock, así que vamos a simular la "compra" en Orca
    // y hacer la venta REAL en Jupiter para demostrar el concepto

    info!(
        "🛒 PASO 1: [SIMULADO] Comprando {} SOL en Orca a ${:.2}",
        arbitrage_amount as f64 / 1_000_000_000.0,
        orca_sol_price
    );
    info!("✅ [SIMULADO] Compra en Orca completada");

    info!(
        "💰 PASO 2: [REAL] Vendiendo {} SOL en Jupiter a ${:.2}",
        arbitrage_amount as f64 / 1_000_000_000.0,
        jupiter_sol_price
    );

    // Ejecutar swap REAL en Jupiter
    match jupiter_client
        .execute_swap_with_wallet(&jupiter_quote, &wallet_pubkey.to_string(), Some(&keypair))
        .await
    {
        Ok(result) => {
            info!("✅ Swap REAL ejecutado en Jupiter!");
            info!("🔗 Resultado: {:?}", result);

            // Esperar confirmación
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            // PASO 4: Medir balance final
            info!("\n📊 === MEDICIÓN FINAL ===");

            let balance_final = rpc_client.get_balance(&wallet_pubkey)?;
            let sol_final = balance_final as f64 / 1_000_000_000.0;

            info!(
                "💰 BALANCE FINAL: {} SOL ({} lamports)",
                sol_final, balance_final
            );

            let cambio_lamports = balance_final as i64 - balance_inicial as i64;
            let cambio_sol = cambio_lamports as f64 / 1_000_000_000.0;

            info!("\n🎯 === RESULTADO DEL ARBITRAJE ===");
            info!("📊 Balance inicial: {} SOL", sol_inicial);
            info!("📊 Balance final:   {} SOL", sol_final);
            info!(
                "📊 Cambio:          {:.9} SOL ({} lamports)",
                cambio_sol, cambio_lamports
            );

            if cambio_sol > 0.0 {
                info!("🎉 ¡ARBITRAJE EXITOSO! +{:.9} SOL", cambio_sol);
                info!(
                    "💰 Profit real demostrado: +${:.4}",
                    cambio_sol * jupiter_sol_price
                );
            } else if cambio_sol < 0.0 {
                warn!("📉 Pérdida en arbitraje: {:.9} SOL", cambio_sol);
                warn!(
                    "💸 Pérdida en USD: -${:.4}",
                    (-cambio_sol) * jupiter_sol_price
                );
            } else {
                info!("➖ Sin cambio en balance (empate)");
            }
        }
        Err(e) => {
            error!("❌ Error ejecutando swap en Jupiter: {}", e);

            // Verificar balance de todas formas para ver si hubo cambios
            let balance_final = rpc_client.get_balance(&wallet_pubkey)?;
            let cambio = balance_final as i64 - balance_inicial as i64;

            if cambio != 0 {
                warn!("⚠️ Balance cambió a pesar del error: {} lamports", cambio);
            }
        }
    }

    Ok(())
}
