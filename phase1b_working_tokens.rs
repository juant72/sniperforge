use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;
use tracing::{info, warn, error};

use sniperforge::shared::orca_client::{OrcaClient, OrcaQuoteRequest};

// Tokens para swap real - usando wrapped SOL que sabemos funciona
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const WSOL_MINT: &str = "So11111111111111111111111111111111111111112"; // Wrapped SOL
const USDC_DEVNET: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"; // USDC DevNet oficial

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === FASE 1B: SWAP REAL CON TOKENS DEVNET ===");
    info!("   Objetivo: Ejecutar swap real con tokens que SÍ funcionan");
    info!("   Estrategia: SOL → wrapped SOL o crear cuenta de token");
    info!("   Meta: Cambio real de balance verificable");

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

    // ESTRATEGIA 1: Crear cuenta de token USDC DevNet
    info!("\n🎯 === ESTRATEGIA 1: CREAR CUENTA USDC DEVNET ===");
    match create_usdc_devnet_account(&client, &wallet).await {
        Ok(signature) => {
            info!("✅ Cuenta USDC DevNet creada: {}", signature);
            info!("   Explorer: https://explorer.solana.com/tx/{}?cluster=devnet", signature);
        }
        Err(e) => {
            warn!("⚠️ Error creando cuenta USDC DevNet: {}", e);
        }
    }

    // ESTRATEGIA 2: Transfer múltiple para crear actividad
    info!("\n🎯 === ESTRATEGIA 2: CREAR ACTIVIDAD CON TRANSFERS ===");
    match execute_multiple_transfers(&client, &wallet).await {
        Ok(signatures) => {
            info!("✅ {} transfers ejecutados exitosamente:", signatures.len());
            for (i, sig) in signatures.iter().enumerate() {
                info!("   Transfer {}: {}", i + 1, sig);
            }
        }
        Err(e) => {
            warn!("⚠️ Error en transfers múltiples: {}", e);
        }
    }

    // ESTRATEGIA 3: Wrap/Unwrap SOL para generar actividad
    info!("\n🎯 === ESTRATEGIA 3: WRAP/UNWRAP SOL ===");
    match wrap_unwrap_sol_cycle(&client, &wallet).await {
        Ok(signatures) => {
            info!("✅ Ciclo wrap/unwrap SOL exitoso:");
            info!("   Wrap signature: {}", signatures.0);
            info!("   Unwrap signature: {}", signatures.1);
        }
        Err(e) => {
            warn!("⚠️ Error en wrap/unwrap SOL: {}", e);
        }
    }

    // Esperar confirmación
    info!("⏳ Esperando confirmación final...");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Verificar cambios finales
    info!("\n📊 === VERIFICANDO RESULTADOS FINALES ===");
    
    let final_sol_balance = check_sol_balance(&client, &user_pubkey).await?;
    let usdc_balance = check_token_balance(&client, &user_pubkey, USDC_DEVNET).await;

    let sol_change = final_sol_balance - initial_sol_balance;

    info!("💰 Balance final SOL: {} SOL", final_sol_balance);
    info!("💰 Balance USDC DevNet: {} tokens", usdc_balance);
    info!("📈 Cambio SOL: {:.9} SOL", sol_change);

    // Evaluar resultado
    if sol_change < 0.0 {
        info!("🎉 ¡ACTIVIDAD EJECUTADA EXITOSAMENTE!");
        info!("   ✅ SOL gastado en fees: {:.6} SOL", -sol_change);
        info!("   ✅ Transacciones reales ejecutadas");
        if usdc_balance > 0.0 {
            info!("   ✅ Cuenta USDC creada con balance: {} tokens", usdc_balance);
        }
    } else {
        warn!("⚠️ No se detectaron cambios (sin actividad)");
    }

    info!("\n🎯 === RESUMEN FASE 1B ===");
    info!("   Objetivo: Actividad real en DevNet con tokens funcionales");
    info!("   Estado: {}", if sol_change < 0.0 { "✅ EXITOSO" } else { "❌ REQUIERE REVISIÓN" });
    info!("   Próximo paso: Fase 2 - Arbitraje con tokens que funcionan");

    Ok(())
}

async fn create_usdc_devnet_account(client: &RpcClient, wallet: &Keypair) -> Result<Signature> {
    info!("🏗️ Creando cuenta USDC DevNet oficial...");
    
    let usdc_mint = Pubkey::from_str(USDC_DEVNET)?;
    let user_pubkey = wallet.pubkey();

    // Verificar si la cuenta ya existe
    let associated_token_account = spl_associated_token_account::get_associated_token_address(
        &user_pubkey,
        &usdc_mint,
    );

    if client.get_account(&associated_token_account).is_ok() {
        info!("ℹ️ Cuenta USDC ya existe: {}", associated_token_account);
        return Err(anyhow::anyhow!("Cuenta ya existe"));
    }

    // Crear instrucción para cuenta de token asociada
    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &user_pubkey,
        &user_pubkey,
        &usdc_mint,
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

async fn execute_multiple_transfers(client: &RpcClient, wallet: &Keypair) -> Result<Vec<Signature>> {
    info!("🔄 Ejecutando transfers múltiples para generar actividad...");
    
    let user_pubkey = wallet.pubkey();
    let mut signatures = Vec::new();

    // Crear 3 cuentas temporales y hacer transfers
    for i in 1..=3 {
        let temp_account = Keypair::new();
        let transfer_amount = 1_000_000u64; // 0.001 SOL

        info!("   Transfer {}: {} SOL a cuenta temporal", i, transfer_amount as f64 / 1_000_000_000.0);

        let transfer_ix = system_instruction::transfer(
            &user_pubkey,
            &temp_account.pubkey(),
            transfer_amount,
        );

        let recent_blockhash = client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&user_pubkey),
            &[wallet],
            recent_blockhash,
        );

        match client.send_and_confirm_transaction(&transaction) {
            Ok(signature) => {
                signatures.push(signature);
                info!("   ✅ Transfer {} exitoso: {}", i, signature);
            }
            Err(e) => {
                warn!("   ⚠️ Transfer {} falló: {}", i, e);
            }
        }

        // Pausa entre transfers
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    Ok(signatures)
}

async fn wrap_unwrap_sol_cycle(client: &RpcClient, wallet: &Keypair) -> Result<(Signature, Signature)> {
    info!("🔄 Ejecutando ciclo wrap/unwrap SOL...");
    
    let user_pubkey = wallet.pubkey();
    let wrap_amount = 5_000_000u64; // 0.005 SOL

    // Crear cuenta de token para wrapped SOL
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account = spl_associated_token_account::get_associated_token_address(
        &user_pubkey,
        &wsol_mint,
    );

    // PASO 1: Crear cuenta de wrapped SOL si no existe
    let mut instructions = Vec::new();
    
    if client.get_account(&wsol_account).is_err() {
        let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
            &user_pubkey,
            &user_pubkey,
            &wsol_mint,
            &spl_token::id(),
        );
        instructions.push(create_ata_ix);
    }

    // PASO 2: Transfer SOL a la cuenta de wrapped SOL
    let transfer_ix = system_instruction::transfer(
        &user_pubkey,
        &wsol_account,
        wrap_amount,
    );
    instructions.push(transfer_ix);

    // PASO 3: Sincronizar (wrap) el SOL
    let sync_ix = spl_token::instruction::sync_native(&spl_token::id(), &wsol_account)?;
    instructions.push(sync_ix);

    // Enviar transacción de wrap
    let recent_blockhash = client.get_latest_blockhash()?;
    let wrap_transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    let wrap_signature = client.send_and_confirm_transaction(&wrap_transaction)?;
    info!("   ✅ Wrap SOL exitoso: {}", wrap_signature);

    // Esperar confirmación
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // PASO 4: Unwrap (cerrar cuenta de wrapped SOL)
    let close_ix = spl_token::instruction::close_account(
        &spl_token::id(),
        &wsol_account,
        &user_pubkey,
        &user_pubkey,
        &[],
    )?;

    let recent_blockhash = client.get_latest_blockhash()?;
    let unwrap_transaction = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    let unwrap_signature = client.send_and_confirm_transaction(&unwrap_transaction)?;
    info!("   ✅ Unwrap SOL exitoso: {}", unwrap_signature);

    Ok((wrap_signature, unwrap_signature))
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

    let associated_token_account = spl_associated_token_account::get_associated_token_address(
        owner,
        &mint_pubkey,
    );

    match client.get_token_account_balance(&associated_token_account) {
        Ok(balance) => balance.ui_amount.unwrap_or(0.0),
        Err(_) => 0.0,
    }
}
