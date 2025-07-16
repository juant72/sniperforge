use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
    native_token::LAMPORTS_PER_SOL,
};
use std::str::FromStr;
use tracing::{info, error};

// Token mints
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === FASE 2B: ARBITRAJE OPTIMIZADO ===");
    info!("   Objetivo: Ciclo profitable con rent handling");
    info!("   Estrategia: Múltiples micro-operaciones optimizadas");

    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let wallet = load_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 Wallet: {}", user_pubkey);

    let initial_balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance inicial: {} SOL", initial_balance);

    if initial_balance < 0.05 {
        error!("❌ Necesitas al menos 0.05 SOL para arbitraje optimizado");
        return Ok(());
    }

    info!("\n🎯 === ARBITRAJE ESTRATÉGICO ===");
    info!("   Método: Profit através de timing y fees optimization");

    match execute_profit_strategy(&client, &wallet).await {
        Ok(total_profit) => {
            let final_balance = check_sol_balance(&client, &user_pubkey).await?;
            let real_profit = final_balance - initial_balance;

            info!("\n📊 === RESULTADO FINAL ===");
            info!("💰 Balance inicial: {} SOL", initial_balance);
            info!("💰 Balance final: {} SOL", final_balance);
            info!("📈 Profit calculado: {:.9} SOL", total_profit);
            info!("📈 Profit real: {:.9} SOL", real_profit);

            if real_profit > 0.0 {
                info!("🎉 ¡ARBITRAJE PROFITABLE LOGRADO!");
                info!("   ✅ Ganancia neta: +{:.9} SOL", real_profit);
                info!("   ✅ ROI: {:.4}%", (real_profit / 0.01) * 100.0);
            } else {
                info!("⚠️ Fees superaron profit: {:.9} SOL", real_profit);
            }
        }
        Err(e) => error!("❌ Error: {}", e),
    }

    Ok(())
}

async fn execute_profit_strategy(client: &RpcClient, wallet: &Keypair) -> Result<f64> {
    let user_pubkey = wallet.pubkey();
    let mut total_profit = 0.0;

    info!("🔧 === ESTRATEGIA MULTI-OPERACIÓN ===");

    // OPERACIÓN 1: Dust collection optimization
    info!("\n💫 OPERACIÓN 1: Optimización de dust...");
    let dust_profit = execute_dust_optimization(client, wallet).await?;
    total_profit += dust_profit;
    info!("   ✅ Dust profit: {:.9} SOL", dust_profit);

    // OPERACIÓN 2: Timing arbitrage con wrapped SOL
    info!("\n🔄 OPERACIÓN 2: Timing arbitrage...");
    let timing_profit = execute_timing_arbitrage(client, wallet).await?;
    total_profit += timing_profit;
    info!("   ✅ Timing profit: {:.9} SOL", timing_profit);

    // OPERACIÓN 3: Fee optimization cycle
    info!("\n⚡ OPERACIÓN 3: Fee optimization...");
    let fee_profit = execute_fee_optimization(client, wallet).await?;
    total_profit += fee_profit;
    info!("   ✅ Fee profit: {:.9} SOL", fee_profit);

    info!("\n📊 Total profit acumulado: {:.9} SOL", total_profit);
    Ok(total_profit)
}

async fn execute_dust_optimization(client: &RpcClient, wallet: &Keypair) -> Result<f64> {
    let user_pubkey = wallet.pubkey();
    let initial_balance = check_sol_balance(client, &user_pubkey).await?;

    // Crear múltiples cuentas temporales con micro amounts
    let temp_accounts: Vec<Keypair> = (0..3).map(|_| Keypair::new()).collect();
    let micro_amount = 100_000u64; // 0.0001 SOL

    let mut instructions = Vec::new();

    // Distribuir pequeñas cantidades
    for temp_account in &temp_accounts {
        let transfer_ix = system_instruction::transfer(
            &user_pubkey,
            &temp_account.pubkey(),
            micro_amount,
        );
        instructions.push(transfer_ix);
    }

    // Enviar transacción de distribución
    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    let distribute_sig = client.send_and_confirm_transaction(&transaction)?;
    info!("   📤 Distribución: {}", distribute_sig);

    // Esperar y recoger (rent reclaim)
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Los fondos en cuentas no-rent-exempt regresan automáticamente
    let final_balance = check_sol_balance(client, &user_pubkey).await?;
    let dust_profit = final_balance - initial_balance;

    Ok(dust_profit)
}

async fn execute_timing_arbitrage(client: &RpcClient, wallet: &Keypair) -> Result<f64> {
    let user_pubkey = wallet.pubkey();
    let initial_balance = check_sol_balance(client, &user_pubkey).await?;

    // Crear cuenta temporal con rent-exempt amount
    let temp_account = Keypair::new();
    let rent_exempt_amount = client.get_minimum_balance_for_rent_exemption(0)?;
    
    info!("   💰 Rent exempt amount: {} lamports", rent_exempt_amount);

    // FASE 1: Transfer con rent exempt
    let transfer_ix = system_instruction::transfer(
        &user_pubkey,
        &temp_account.pubkey(),
        rent_exempt_amount + 1_000_000, // Rent + extra
    );

    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    let transfer_sig = client.send_and_confirm_transaction(&transaction)?;
    info!("   📤 Transfer timing: {}", transfer_sig);

    // FASE 2: Optimización de timing
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // FASE 3: Reclaim con estrategia optimizada
    let reclaim_ix = system_instruction::transfer(
        &temp_account.pubkey(),
        &user_pubkey,
        1_000_000, // Solo el extra, manteniendo rent exempt
    );

    let recent_blockhash = client.get_latest_blockhash()?;
    let reclaim_transaction = Transaction::new_signed_with_payer(
        &[reclaim_ix],
        Some(&temp_account.pubkey()),
        &[&temp_account],
        recent_blockhash,
    );

    let reclaim_sig = client.send_and_confirm_transaction(&reclaim_transaction)?;
    info!("   📥 Reclaim timing: {}", reclaim_sig);

    let final_balance = check_sol_balance(client, &user_pubkey).await?;
    let timing_profit = final_balance - initial_balance;

    Ok(timing_profit)
}

async fn execute_fee_optimization(client: &RpcClient, wallet: &Keypair) -> Result<f64> {
    let user_pubkey = wallet.pubkey();
    let initial_balance = check_sol_balance(client, &user_pubkey).await?;

    // Crear wrapped SOL account para fee optimization
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account = spl_associated_token_account::get_associated_token_address(
        &user_pubkey,
        &wsol_mint,
    );

    let wrap_amount = 5_000_000u64; // 0.005 SOL

    // Solo crear ATA si no existe
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

    // Wrap pequeña cantidad
    instructions.push(system_instruction::transfer(&user_pubkey, &wsol_account, wrap_amount));
    instructions.push(spl_token::instruction::sync_native(&spl_token::id(), &wsol_account)?);

    let recent_blockhash = client.get_latest_blockhash()?;
    let wrap_transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    let wrap_sig = client.send_and_confirm_transaction(&wrap_transaction)?;
    info!("   💫 Wrap optimizado: {}", wrap_sig);

    // Unwrap inmediato para profit
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let close_ix = spl_token::instruction::close_account(
        &spl_token::id(),
        &wsol_account,
        &user_pubkey,
        &user_pubkey,
        &[],
    )?;

    let recent_blockhash = client.get_latest_blockhash()?;
    let close_transaction = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    let close_sig = client.send_and_confirm_transaction(&close_transaction)?;
    info!("   🔄 Unwrap optimizado: {}", close_sig);

    let final_balance = check_sol_balance(client, &user_pubkey).await?;
    let fee_profit = final_balance - initial_balance;

    Ok(fee_profit)
}

async fn load_wallet() -> Result<Keypair> {
    let wallet_path = "test-cli-arbitrage.json";
    
    if std::path::Path::new(wallet_path).exists() {
        let wallet_data = std::fs::read_to_string(wallet_path)?;
        let secret_key: Vec<u8> = serde_json::from_str(&wallet_data)?;
        Ok(Keypair::from_bytes(&secret_key)?)
    } else {
        error!("❌ Wallet file not found: {}", wallet_path);
        std::process::exit(1);
    }
}

async fn check_sol_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<f64> {
    let balance = client.get_balance(pubkey)?;
    Ok(balance as f64 / LAMPORTS_PER_SOL as f64)
}
