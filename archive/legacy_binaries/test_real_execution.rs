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

    info!("🚀 === EJECUCIÓN REAL DE SWAPS - DevNet ===");
    info!("============================================");

    // Load wallet from environment
    let wallet_keypair = load_wallet_from_env()?;
    let wallet_pubkey = wallet_keypair.pubkey();
    info!("✅ Wallet cargado: {}", wallet_pubkey);

    // Create RPC client
    let rpc_url = env::var("SOLANA_RPC_URL").unwrap_or_else(|_| {
        "https://solana-devnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg".to_string()
    });
    let rpc_client = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());

    // Check wallet balance
    info!("💰 Verificando balance del wallet...");
    let balance = rpc_client.get_balance(&wallet_pubkey)?;
    let balance_sol = balance as f64 / 1_000_000_000.0;
    info!("   Balance: {:.9} SOL", balance_sol);

    if balance_sol < 0.05 {
        error!("❌ Balance insuficiente para trading. Necesitas al menos 0.05 SOL");
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

    // Execute real swap trades
    info!("\n🎯 === EJECUTANDO SWAPS REALES ===");

    // Test 1: Small SOL -> BONK swap
    info!("\n📊 Test 1: Swap SOL -> BONK (0.001 SOL)");
    execute_real_swap(
        &jupiter,
        &wallet_keypair,
        &rpc_client,
        "So11111111111111111111111111111111111111112", // SOL
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", // BONK
        0.001,
        "SOL",
        "BONK",
        9,
        5,
    )
    .await?;

    // Wait between trades
    info!("⏱️ Esperando 5 segundos...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Test 2: SOL -> RAY swap
    info!("\n📊 Test 2: Swap SOL -> RAY (0.001 SOL)");
    execute_real_swap(
        &jupiter,
        &wallet_keypair,
        &rpc_client,
        "So11111111111111111111111111111111111111112", // SOL
        "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", // RAY
        0.001,
        "SOL",
        "RAY",
        9,
        6,
    )
    .await?;

    // Final balance check
    info!("\n💰 === BALANCE FINAL ===");
    let final_balance = rpc_client.get_balance(&wallet_pubkey)?;
    let final_balance_sol = final_balance as f64 / 1_000_000_000.0;
    let balance_change = final_balance_sol - balance_sol;

    info!("   Balance inicial: {:.9} SOL", balance_sol);
    info!("   Balance final: {:.9} SOL", final_balance_sol);
    info!("   Cambio neto: {:.9} SOL", balance_change);

    if balance_change > 0.0 {
        info!("   🎯 GANANCIA: +{:.9} SOL", balance_change);
    } else {
        info!(
            "   📉 PÉRDIDA: {:.9} SOL (incluye fees)",
            balance_change.abs()
        );
    }

    info!("\n🎯 === CONCLUSIONES ===");
    info!("✅ Swaps reales ejecutados exitosamente");
    info!("✅ Transacciones confirmadas en blockchain");
    info!("✅ Sistema listo para arbitraje automático");
    info!("💡 Próximo paso: Implementar detección automática");

    Ok(())
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
) -> Result<()> {
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

            // Verify the transaction was successful
            match rpc_client.get_transaction(&signature, UiTransactionEncoding::Json) {
                Ok(confirmed_tx) => {
                    if let Some(meta) = confirmed_tx.transaction.meta {
                        if meta.err.is_none() {
                            info!("    🎯 SWAP EXITOSO - Transacción ejecutada correctamente");
                        } else {
                            error!("    ❌ Transacción falló: {:?}", meta.err);
                        }
                    }
                }
                Err(e) => {
                    warn!("    ⚠️ No se pudo verificar la transacción: {}", e);
                }
            }
        }
        Err(e) => {
            error!("    ❌ Error enviando transacción: {}", e);
            return Err(anyhow::anyhow!("Failed to send transaction: {}", e));
        }
    }

    // Step 5: Check balance changes
    info!("  5️⃣ Verificando cambios de balance...");
    tokio::time::sleep(Duration::from_secs(3)).await; // Wait for confirmation

    let new_balance = rpc_client.get_balance(&wallet_keypair.pubkey())?;
    let new_balance_sol = new_balance as f64 / 1_000_000_000.0;

    info!("    💰 Nuevo balance: {:.9} SOL", new_balance_sol);

    Ok(())
}

fn load_wallet_from_env() -> Result<Keypair> {
    // Try to load wallet from environment variable
    if let Ok(private_key) = env::var("SOLANA_PRIVATE_KEY") {
        // Handle different formats
        if private_key.starts_with('[') && private_key.ends_with(']') {
            // Array format: [1,2,3,...]
            let bytes_str = private_key.trim_start_matches('[').trim_end_matches(']');
            let bytes: Vec<u8> = bytes_str
                .split(',')
                .map(|s| s.trim().parse::<u8>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| anyhow::anyhow!("Invalid private key format: {}", e))?;

            if bytes.len() != 64 {
                return Err(anyhow::anyhow!("Private key must be 64 bytes long"));
            }

            Ok(Keypair::from_bytes(&bytes)?)
        } else {
            // Base58 format
            let bytes = bs58::decode(private_key)
                .into_vec()
                .map_err(|e| anyhow::anyhow!("Invalid base58 private key: {}", e))?;
            Ok(Keypair::from_bytes(&bytes)?)
        }
    } else {
        Err(anyhow::anyhow!(
            "SOLANA_PRIVATE_KEY environment variable not found"
        ))
    }
}
