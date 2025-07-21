use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use std::str::FromStr;
use tracing::{error, info, warn};

use sniperforge::shared::orca_client::{OrcaClient, OrcaQuoteRequest, OrcaSwapRequest};

// Tokens para swap real
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const BONK_MINT: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === ORCA REAL SWAP ARBITRAGE - FASE 1 ===");
    info!("   Objetivo: Ejecutar swap real SOL → BONK");
    info!("   Estrategia: Usar Orca client para cambio real de balance");
    info!("   Meta: Recibir tokens BONK verificables");

    // Configurar cliente RPC
    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    // Cargar wallet
    let wallet = load_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 Wallet: {}", user_pubkey);

    // Verificar balance inicial
    let initial_sol_balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance inicial SOL: {} SOL", initial_sol_balance);

    if initial_sol_balance < 0.02 {
        error!("❌ Insuficiente SOL para swap. Necesitas al menos 0.02 SOL");
        return Ok(());
    }

    // Verificar balance inicial de tokens BONK
    let initial_bonk_balance = check_token_balance(&client, &user_pubkey, BONK_MINT).await;
    info!("💰 Balance inicial BONK: {} tokens", initial_bonk_balance);

    // Inicializar cliente Orca
    info!("🌊 Inicializando cliente Orca...");
    let orca_client = OrcaClient::new("devnet");
    info!("✅ Orca client inicializado");

    // PASO 1: Obtener cotización para swap SOL → BONK
    let swap_amount_sol = 0.01; // 0.01 SOL
    let swap_amount_lamports = (swap_amount_sol * 1_000_000_000.0) as u64;

    info!("\n🔍 === OBTENIENDO COTIZACIÓN ORCA ===");
    info!("   Swap: {} SOL → BONK", swap_amount_sol);
    info!("   Cantidad: {} lamports", swap_amount_lamports);

    let quote_request = OrcaQuoteRequest {
        input_mint: SOL_MINT.to_string(),
        output_mint: BONK_MINT.to_string(),
        amount: swap_amount_lamports.to_string(),
        slippage_bps: 100, // 1% slippage
    };

    let quote = match orca_client.get_quote(&quote_request).await {
        Ok(quote) => {
            info!("✅ Cotización Orca obtenida:");
            info!("   Input: {} lamports SOL", swap_amount_lamports);
            info!("   Output: {} tokens BONK", quote.output_amount);
            info!(
                "   Price impact: {:.2}%",
                quote.price_impact_pct.unwrap_or(0.0)
            );
            quote
        }
        Err(e) => {
            error!("❌ Error obteniendo cotización Orca: {}", e);
            return Ok(());
        }
    };

    // PASO 2: Ejecutar swap usando método simplificado
    info!("\n🚀 === EJECUTANDO SWAP SIMPLIFICADO ===");
    info!("   Método: execute_real_swap() de Orca client");
    info!("   Esperado: {} BONK tokens", quote.output_amount);

    // Intentar usar el método execute_real_swap si existe
    match execute_simple_orca_swap(&orca_client, &client, &wallet, &quote).await {
        Ok(signature) => {
            info!("✅ SWAP EJECUTADO EXITOSAMENTE!");
            info!("   Signature: {}", signature);
            info!(
                "   Explorer: https://explorer.solana.com/tx/{}?cluster=devnet",
                signature
            );
        }
        Err(e) => {
            error!("❌ Error ejecutando swap: {}", e);

            // FALLBACK: Crear cuenta de tokens manualmente
            info!("🔧 Intentando fallback: crear cuenta de tokens...");
            match create_token_account(&client, &wallet, BONK_MINT).await {
                Ok(signature) => {
                    info!("✅ Cuenta de tokens BONK creada: {}", signature);
                }
                Err(e) => {
                    warn!("⚠️ Error creando cuenta de tokens: {}", e);
                }
            }
        }
    }

    // Esperar confirmación
    info!("⏳ Esperando confirmación en blockchain...");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // PASO 3: Verificar cambios en balance
    info!("\n📊 === VERIFICANDO RESULTADOS ===");

    let final_sol_balance = check_sol_balance(&client, &user_pubkey).await?;
    let final_bonk_balance = check_token_balance(&client, &user_pubkey, BONK_MINT).await;

    // Calcular cambios
    let sol_change = final_sol_balance - initial_sol_balance;
    let bonk_change = final_bonk_balance - initial_bonk_balance;

    info!("💰 Balance final SOL: {} SOL", final_sol_balance);
    info!("💰 Balance final BONK: {} tokens", final_bonk_balance);
    info!("📈 Cambio SOL: {:.9} SOL", sol_change);
    info!("📈 Cambio BONK: {} tokens", bonk_change);

    // Evaluar resultado
    if bonk_change > 0.0 {
        info!("🎉 ¡SWAP EXITOSO!");
        info!("   ✅ Recibido: {} BONK tokens", bonk_change);
        info!("   ✅ Gastado: ~{:.6} SOL (incluye fees)", -sol_change);
        info!("   ✅ Balance de tokens cambió verificablemente");
    } else if sol_change < 0.0 {
        info!("📊 TRANSACCIÓN EJECUTADA:");
        info!("   ✅ SOL gastado: {:.6} SOL", -sol_change);
        info!("   ⚠️ BONK no recibido (posible delay o error)");
        info!("   💡 Cuenta de tokens podría estar creada");
    } else {
        warn!("⚠️ No se detectaron cambios en balance");
        info!("   Posibles causas:");
        info!("   - Swap no ejecutado");
        info!("   - Error en cotización Orca");
        info!("   - Problema de conectividad");
    }

    info!("\n🎯 === RESUMEN FASE 1 ===");
    info!("   Objetivo: Swap real SOL → BONK");
    info!(
        "   Estado: {}",
        if bonk_change > 0.0 {
            "✅ EXITOSO"
        } else if sol_change < 0.0 {
            "⚠️ PARCIAL"
        } else {
            "❌ REQUIERE REVISIÓN"
        }
    );
    info!("   Próximo paso: Fase 2 - Ciclo completo con ganancia");

    Ok(())
}

async fn execute_simple_orca_swap(
    orca_client: &OrcaClient,
    rpc_client: &RpcClient,
    wallet: &Keypair,
    quote: &sniperforge::shared::orca_client::OrcaQuoteResponse,
) -> Result<Signature> {
    info!("🔄 Intentando ejecutar swap con Orca...");

    // Crear swap request
    let swap_request = OrcaSwapRequest {
        quote: quote.clone(),
        user_public_key: wallet.pubkey().to_string(),
        wrap_unwrap_sol: true,
    };

    // Intentar obtener transacción de swap
    let swap_response = orca_client.get_swap_transaction(&swap_request).await?;

    // Decodificar y enviar transacción
    let transaction_bytes = base64::decode(&swap_response.transaction)?;
    let mut transaction: Transaction = bincode::deserialize(&transaction_bytes)?;

    // Actualizar blockhash y firmar
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.message.recent_blockhash = recent_blockhash;
    transaction.sign(&[wallet], recent_blockhash);

    // Enviar transacción
    info!("📡 Enviando transacción al blockchain...");
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;

    Ok(signature)
}

async fn create_token_account(
    client: &RpcClient,
    wallet: &Keypair,
    mint: &str,
) -> Result<Signature> {
    info!("🏗️ Creando cuenta de token para mint: {}", mint);

    let mint_pubkey = Pubkey::from_str(mint)?;
    let user_pubkey = wallet.pubkey();

    // Crear instrucción para cuenta de token asociada
    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &user_pubkey,
        &user_pubkey,
        &mint_pubkey,
        &spl_token::id(),
    );

    // Crear y enviar transacción
    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[create_ata_ix],
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    let signature = client.send_and_confirm_transaction(&transaction)?;
    Ok(signature)
}

async fn load_wallet() -> Result<Keypair> {
    let wallet_path = "test-cli-arbitrage.json";

    if std::path::Path::new(wallet_path).exists() {
        let wallet_data = std::fs::read_to_string(wallet_path)?;
        let secret_key: Vec<u8> = serde_json::from_str(&wallet_data)?;
        Ok(Keypair::from_bytes(&secret_key)?)
    } else {
        error!("❌ Wallet file not found: {}", wallet_path);
        error!("   Ejecuta primero: cargo run --bin create_test_wallet");
        std::process::exit(1);
    }
}

async fn check_sol_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<f64> {
    let balance = client.get_balance(pubkey)?;
    Ok(balance as f64 / 1_000_000_000.0) // Convert lamports to SOL
}

async fn check_token_balance(client: &RpcClient, owner: &Pubkey, mint: &str) -> f64 {
    let mint_pubkey = match Pubkey::from_str(mint) {
        Ok(pk) => pk,
        Err(_) => return 0.0,
    };

    // Obtener cuenta de token asociada
    let associated_token_account =
        spl_associated_token_account::get_associated_token_address(owner, &mint_pubkey);

    // Verificar balance
    match client.get_token_account_balance(&associated_token_account) {
        Ok(balance) => balance.ui_amount.unwrap_or(0.0),
        Err(_) => 0.0, // Cuenta no existe o sin balance
    }
}
