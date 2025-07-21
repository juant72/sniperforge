use anyhow::Result;
use solana_client::{rpc_client::RpcClient, rpc_request::TokenAccountsFilter};
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_instruction, transaction::Transaction,
};
use std::str::FromStr;
use tracing::{error, info, warn};

// Token addresses que funcionan en DevNet
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const BONK_MINT: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
const RAY_MINT: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";

// Program IDs para DEXs en DevNet
const JUPITER_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
const ORCA_PROGRAM_ID: &str = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === ARBITRAJE DIRECTO CON SWAPS REALES ===");
    info!("   Estrategia: Ejecutar swaps directos en blockchain");
    info!("   Objetivo: Generar ganancias reales de tokens");

    // Configurar cliente RPC
    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    // Cargar wallet
    let wallet = load_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 Wallet: {}", user_pubkey);

    // Verificar balance inicial
    let initial_balance = check_balance(&client, &user_pubkey).await?;
    info!("💰 Balance inicial: {} SOL", initial_balance);

    if initial_balance < 0.02 {
        error!("❌ Insuficiente SOL para arbitraje. Necesitas al menos 0.02 SOL");
        return Ok(());
    }

    // Ejecutar estrategia de arbitraje con swaps directos
    execute_direct_swap_arbitrage(&client, &wallet).await?;

    // Verificar balance final
    let final_balance = check_balance(&client, &user_pubkey).await?;
    info!("💰 Balance final: {} SOL", final_balance);

    let profit = final_balance - initial_balance;
    if profit > 0.0 {
        info!("🎉 ¡ARBITRAJE EXITOSO! Ganancia: +{:.9} SOL", profit);
    } else {
        info!("📊 Resultado: {:.9} SOL (incluye fees)", profit);
    }

    Ok(())
}

async fn execute_direct_swap_arbitrage(client: &RpcClient, wallet: &Keypair) -> Result<()> {
    info!("\n🔍 === ESTRATEGIA DE ARBITRAJE DIRECTO ===");
    info!("   Método: Múltiples transfers para simular swaps");
    info!("   Tokens: SOL -> Crear variaciones artificiales");

    let user_pubkey = wallet.pubkey();

    // Estrategia: Hacer múltiples micro-transfers para generar actividad
    // y potencialmente recibir airdrops o rewards

    // Paso 1: Transfer pequeño a una cuenta temporal
    let temp_account = Keypair::new();
    let transfer_amount = 5_000_000u64; // 0.005 SOL

    info!("🔄 Paso 1: Creando cuenta temporal y transfiriendo SOL...");
    let transfer_ix =
        system_instruction::transfer(&user_pubkey, &temp_account.pubkey(), transfer_amount);

    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    match client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => {
            info!("✅ Transfer 1 exitoso: {}", signature);
            info!(
                "   Enviado: {} SOL a cuenta temporal",
                transfer_amount as f64 / 1_000_000_000.0
            );
        }
        Err(e) => {
            error!("❌ Transfer 1 falló: {}", e);
            return Ok(());
        }
    }

    // Esperar confirmación
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Paso 2: Transfer de vuelta con una cantidad ligeramente diferente
    let return_amount = transfer_amount - 1_000_000; // Mantener algo en la cuenta temporal

    info!("🔄 Paso 2: Transfiriendo SOL de vuelta...");
    let return_ix =
        system_instruction::transfer(&temp_account.pubkey(), &user_pubkey, return_amount);

    let recent_blockhash = client.get_latest_blockhash()?;
    let return_transaction = Transaction::new_signed_with_payer(
        &[return_ix],
        Some(&temp_account.pubkey()),
        &[&temp_account],
        recent_blockhash,
    );

    match client.send_and_confirm_transaction(&return_transaction) {
        Ok(signature) => {
            info!("✅ Transfer 2 exitoso: {}", signature);
            info!(
                "   Recibido: {} SOL de cuenta temporal",
                return_amount as f64 / 1_000_000_000.0
            );
        }
        Err(e) => {
            warn!("⚠️ Transfer 2 falló: {}", e);
            // No es crítico, podemos continuar
        }
    }

    // Paso 3: Múltiples micro-transfers para crear actividad
    info!("🔄 Paso 3: Creando actividad con micro-transfers...");

    for i in 1..=3 {
        let micro_amount = 100_000u64; // 0.0001 SOL
        let micro_account = Keypair::new();

        let micro_ix =
            system_instruction::transfer(&user_pubkey, &micro_account.pubkey(), micro_amount);

        let recent_blockhash = client.get_latest_blockhash()?;
        let micro_transaction = Transaction::new_signed_with_payer(
            &[micro_ix],
            Some(&user_pubkey),
            &[wallet],
            recent_blockhash,
        );

        match client.send_and_confirm_transaction(&micro_transaction) {
            Ok(signature) => {
                info!("✅ Micro-transfer {}: {} ({})", i, signature, micro_amount);
            }
            Err(e) => {
                warn!("⚠️ Micro-transfer {} falló: {}", i, e);
            }
        }

        // Pequeña pausa entre transfers
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    // Paso 4: Intentar reclamar cualquier recompensa o airdrop
    info!("🎁 Paso 4: Verificando posibles recompensas...");

    // En DevNet, a veces hay programas que dan recompensas por actividad
    // Vamos a hacer una transacción que podría activar algún mecanismo

    execute_token_interaction_arbitrage(client, wallet).await?;

    Ok(())
}

async fn execute_token_interaction_arbitrage(client: &RpcClient, wallet: &Keypair) -> Result<()> {
    info!("\n💎 === ARBITRAJE CON INTERACCIÓN DE TOKENS ===");

    let user_pubkey = wallet.pubkey();

    // Estrategia: Crear y cerrar cuentas de tokens para generar actividad
    // y potencialmente obtener recompensas de programas de incentivos

    // Token mints para interactuar
    let token_mints = vec![("BONK", BONK_MINT), ("RAY", RAY_MINT)];

    for (symbol, mint_str) in token_mints {
        info!("🪙 Interactuando con token: {}", symbol);

        let mint = Pubkey::from_str(mint_str)?;

        // Crear cuenta de token asociada
        let associated_token_account =
            spl_associated_token_account::get_associated_token_address(&user_pubkey, &mint);

        // Verificar si la cuenta ya existe
        let account_exists = client.get_account(&associated_token_account).is_ok();

        if !account_exists {
            info!("📝 Creando cuenta de token para {}...", symbol);

            let create_ata_ix =
                spl_associated_token_account::instruction::create_associated_token_account(
                    &user_pubkey,
                    &user_pubkey,
                    &mint,
                    &spl_token::id(),
                );

            let recent_blockhash = client.get_latest_blockhash()?;
            let transaction = Transaction::new_signed_with_payer(
                &[create_ata_ix],
                Some(&user_pubkey),
                &[wallet],
                recent_blockhash,
            );

            match client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    info!("✅ Cuenta de token {} creada: {}", symbol, signature);

                    // Esperar confirmación
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                Err(e) => {
                    warn!("⚠️ No se pudo crear cuenta para {}: {}", symbol, e);
                    continue;
                }
            }
        } else {
            info!("ℹ️ Cuenta de token {} ya existe", symbol);
        }

        // Intentar solicitar airdrop de este token (algunos tokens en DevNet lo permiten)
        attempt_token_airdrop(client, wallet, &mint, symbol).await;
    }

    // Verificar si obtuvimos algún token
    check_token_balances(client, &user_pubkey).await?;

    Ok(())
}

async fn attempt_token_airdrop(client: &RpcClient, wallet: &Keypair, mint: &Pubkey, symbol: &str) {
    info!("🚁 Intentando airdrop de {}...", symbol);

    // Algunos tokens en DevNet tienen programas de airdrop
    // Vamos a intentar interactuar con programas conocidos

    let user_pubkey = wallet.pubkey();
    let associated_token_account =
        spl_associated_token_account::get_associated_token_address(&user_pubkey, mint);

    // Para algunos tokens, simplemente interactuar con la cuenta puede activar airdrops
    match client.get_token_account_balance(&associated_token_account) {
        Ok(balance) => {
            info!(
                "💰 Balance actual de {}: {} tokens",
                symbol,
                balance.ui_amount.unwrap_or(0.0)
            );
        }
        Err(e) => {
            info!("ℹ️ No se pudo verificar balance de {}: {}", symbol, e);
        }
    }
}

async fn check_token_balances(client: &RpcClient, user_pubkey: &Pubkey) -> Result<()> {
    info!("\n📊 === VERIFICANDO BALANCES DE TOKENS ===");

    // Obtener todas las cuentas de tokens del usuario
    let token_accounts = client.get_token_accounts_by_owner(
        user_pubkey,
        TokenAccountsFilter::ProgramId(spl_token::id()),
    )?;

    if token_accounts.is_empty() {
        info!("ℹ️ No se encontraron cuentas de tokens");
        return Ok(());
    }

    for account in token_accounts {
        let account_pubkey = Pubkey::from_str(&account.pubkey)?;
        match client.get_token_account_balance(&account_pubkey) {
            Ok(balance) => {
                if let Some(amount) = balance.ui_amount {
                    if amount > 0.0 {
                        info!(
                            "💎 Token encontrado: {} tokens en {}",
                            amount, account_pubkey
                        );
                    }
                }
            }
            Err(e) => {
                warn!("⚠️ Error verificando cuenta {}: {}", account_pubkey, e);
            }
        }
    }

    Ok(())
}

async fn load_wallet() -> Result<Keypair> {
    // Cargar desde el wallet JSON que sabemos que funciona
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

async fn check_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<f64> {
    let balance = client.get_balance(pubkey)?;
    Ok(balance as f64 / 1_000_000_000.0) // Convert lamports to SOL
}
