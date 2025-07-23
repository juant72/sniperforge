use anyhow::Result;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use sniperforge::shared::jupiter_api::Jupiter;
use sniperforge::shared::jupiter_config::JupiterConfig;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_transaction_status::UiTransactionEncoding;
use std::env;
use std::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    info!("🚀 === ARBITRAJE REAL EN DEVNET - USING WORKING TOKENS ===");
    info!("=========================================================");

    // Load wallet from environment
    let wallet_keypair = load_wallet_from_env()?;
    let wallet_pubkey = wallet_keypair.pubkey();
    info!("✅ Wallet cargado: {}", wallet_pubkey);

    // Create RPC client
    let rpc_url =
        env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let rpc_client = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());

    // Check wallet balance
    info!("💰 Verificando balance del wallet...");
    let balance = rpc_client.get_balance(&wallet_pubkey)?;
    let balance_sol = balance as f64 / 1_000_000_000.0;
    info!("   Balance: {:.9} SOL", balance_sol);

    if balance_sol < 0.05 {
        error!("❌ Balance insuficiente para arbitraje. Necesitas al menos 0.05 SOL");
        return Ok(());
    }

    // Create Jupiter client
    let jupiter_config = JupiterConfig {
        base_url: "https://quote-api.jup.ag".to_string(),
        api_key: None,
        timeout_seconds: 60,
        max_retries: 3,
        rpc_endpoint: rpc_url.clone(),
        network_name: "devnet".to_string(),
    };

    let jupiter = Jupiter::new(&jupiter_config).await?;
    info!("✅ Jupiter client inicializado");

    // Execute REAL arbitrage cycle using working tokens
    info!("\n🎯 === EJECUTANDO ARBITRAJE REAL ===");

    // Strategy: SOL -> BONK -> SOL (Cycle with working tokens)
    let sol_mint = "So11111111111111111111111111111111111111112";
    let bonk_mint = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"; // Working BONK token

    let arbitrage_amount = 0.01; // 0.01 SOL

    info!(
        "💰 Arbitraje circular: {} SOL -> BONK -> SOL",
        arbitrage_amount
    );

    // Get initial balance
    let initial_balance = balance_sol;
    info!("🏦 Balance inicial: {:.9} SOL", initial_balance);

    // STEP 1: SOL -> BONK
    info!("\n🔄 PASO 1: SOL -> BONK");
    let bonk_amount = match execute_real_swap(
        &jupiter,
        &wallet_keypair,
        &rpc_client,
        sol_mint,
        bonk_mint,
        arbitrage_amount,
        "SOL",
        "BONK",
        9,
        5,
    )
    .await
    {
        Ok(amount) => {
            info!(
                "✅ SWAP 1 EXITOSO: {} SOL -> {} BONK",
                arbitrage_amount, amount
            );
            amount
        }
        Err(e) => {
            error!("❌ SWAP 1 FALLÓ: {}", e);
            return Ok(());
        }
    };

    // Wait for confirmation
    info!("⏱️ Esperando confirmación...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // STEP 2: BONK -> SOL
    info!("\n🔄 PASO 2: BONK -> SOL");
    let final_sol_amount = match execute_real_swap(
        &jupiter,
        &wallet_keypair,
        &rpc_client,
        bonk_mint,
        sol_mint,
        bonk_amount,
        "BONK",
        "SOL",
        5,
        9,
    )
    .await
    {
        Ok(amount) => {
            info!("✅ SWAP 2 EXITOSO: {} BONK -> {} SOL", bonk_amount, amount);
            amount
        }
        Err(e) => {
            error!("❌ SWAP 2 FALLÓ: {}", e);
            info!("⚠️ Nota: Es posible que tengamos BONK pero el swap de vuelta falló");
            return Ok(());
        }
    };

    // Wait for final confirmation
    info!("⏱️ Esperando confirmación final...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Check final balance
    info!("\n💰 === RESULTADO FINAL ===");
    let final_balance = rpc_client.get_balance(&wallet_pubkey)?;
    let final_balance_sol = final_balance as f64 / 1_000_000_000.0;

    let profit = final_balance_sol - initial_balance;
    let profit_percentage = (profit / arbitrage_amount) * 100.0;

    info!("🏦 Balance inicial:  {:.9} SOL", initial_balance);
    info!("🏦 Balance final:    {:.9} SOL", final_balance_sol);
    info!("💰 Cambio neto:      {:.9} SOL", profit);
    info!("📊 Porcentaje:       {:.2}%", profit_percentage);

    if profit > 0.0 {
        info!("🎉 ¡ARBITRAJE EXITOSO! Ganancia: {:.9} SOL", profit);
    } else if profit == 0.0 {
        info!("⚖️ ARBITRAJE NEUTRAL - Sin ganancias ni pérdidas");
    } else {
        info!("📉 PÉRDIDA: {:.9} SOL (incluye fees)", profit);
    }

    info!("\n🎯 === CONCLUSIONES ===");
    info!("✅ Arbitraje REAL ejecutado en DevNet");
    info!("✅ Transacciones procesadas correctamente");
    info!("✅ Sistema de arbitraje circular funcionando");
    info!("💡 Próximo paso: Optimizar strategy para profits consistentes");

    Ok(())
}

fn load_wallet_from_env() -> Result<Keypair> {
    let private_key = env::var("SOLANA_PRIVATE_KEY")
        .or_else(|_| env::var("DEVNET_PRIVATE_KEY"))
        .map_err(|_| {
            anyhow::anyhow!("SOLANA_PRIVATE_KEY or DEVNET_PRIVATE_KEY environment variable not set")
        })?;

    let private_key_bytes = bs58::decode(&private_key)
        .into_vec()
        .map_err(|e| anyhow::anyhow!("Failed to decode private key: {}", e))?;

    let keypair = Keypair::from_bytes(&private_key_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to create keypair from bytes: {}", e))?;

    Ok(keypair)
}

async fn execute_real_swap(
    jupiter: &Jupiter,
    wallet_keypair: &Keypair,
    rpc_client: &RpcClient,
    input_mint: &str,
    output_mint: &str,
    amount: f64,
    input_symbol: &str,
    output_symbol: &str,
    input_decimals: u8,
    output_decimals: u8,
) -> Result<f64> {
    info!(
        "🔄 Ejecutando swap real: {} {} -> {}",
        amount, input_symbol, output_symbol
    );

    // Step 1: Get quote from Jupiter
    info!("  1️⃣ Obteniendo quote de Jupiter...");
    let quote = match jupiter
        .get_quote(input_mint, output_mint, amount, 100)
        .await
    {
        Ok(quote) => {
            let output_amount = quote.outAmount.parse::<u64>().unwrap_or(0);
            let output_tokens = output_amount as f64 / 10_u64.pow(output_decimals as u32) as f64;

            info!("    ✅ Quote obtenido:");
            info!("       Input: {} {}", amount, input_symbol);
            info!("       Output: {:.6} {}", output_tokens, output_symbol);
            info!(
                "       Price Impact: {:.2}%",
                quote.priceImpactPct.parse::<f64>().unwrap_or(0.0) * 100.0
            );

            quote
        }
        Err(e) => {
            error!("    ❌ Error obteniendo quote: {}", e);
            return Err(e);
        }
    };

    // Step 2: Build swap transaction using Jupiter client directly
    info!("  2️⃣ Construyendo transacción...");

    let wallet_address = wallet_keypair.pubkey().to_string();

    // Create swap request
    let swap_request = sniperforge::shared::jupiter_types::SwapRequest {
        quoteResponse: quote.clone(),
        userPublicKey: wallet_address.clone(),
        dynamicComputeUnitLimit: Some(true),
        dynamicSlippage: Some(true),
        prioritizationFeeLamports: Some(sniperforge::shared::jupiter_types::PrioritizationFee {
            priorityLevelWithMaxLamports: sniperforge::shared::jupiter_types::PriorityLevelConfig {
                maxLamports: 1000000, // 0.001 SOL max priority fee for devnet
                priorityLevel: "medium".to_string(),
            },
        }),
        asLegacyTransaction: Some(true),
    };

    let swap_response = match jupiter.client.build_swap_transaction(swap_request).await {
        Ok(response) => {
            info!("    ✅ Transacción construida exitosamente");
            response
        }
        Err(e) => {
            error!("    ❌ Error construyendo transacción: {}", e);
            return Err(e);
        }
    };

    // Step 3: Prepare and sign transaction
    info!("  3️⃣ Firmando transacción...");

    // Decode the transaction from base64
    let tx_data = match BASE64.decode(&swap_response.swapTransaction) {
        Ok(data) => data,
        Err(e) => {
            error!("    ❌ Error decodificando transacción: {}", e);
            return Err(anyhow::anyhow!("Failed to decode transaction: {}", e));
        }
    };

    // Deserialize the transaction
    let mut transaction: Transaction = match bincode::deserialize(&tx_data) {
        Ok(tx) => tx,
        Err(e) => {
            error!("    ❌ Error deserializando transacción: {}", e);
            return Err(anyhow::anyhow!("Failed to deserialize transaction: {}", e));
        }
    };

    // Get recent blockhash
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.message.recent_blockhash = recent_blockhash;

    // Sign the transaction
    transaction.sign(&[wallet_keypair], recent_blockhash);

    info!("    ✅ Transacción firmada exitosamente");

    // Step 4: Send transaction
    info!("  4️⃣ Enviando transacción a la blockchain...");

    match rpc_client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => {
            info!("    ✅ TRANSACCIÓN CONFIRMADA!");
            info!("       Signature: {}", signature);
            info!(
                "       Explorer: https://explorer.solana.com/tx/{}?cluster=devnet",
                signature
            );

            // Return the expected output amount
            let output_amount = quote.outAmount.parse::<u64>().unwrap_or(0);
            let output_tokens = output_amount as f64 / 10_u64.pow(output_decimals as u32) as f64;

            Ok(output_tokens)
        }
        Err(e) => {
            error!("    ❌ Error enviando transacción: {}", e);
            Err(anyhow::anyhow!("Failed to send transaction: {}", e))
        }
    }
}
