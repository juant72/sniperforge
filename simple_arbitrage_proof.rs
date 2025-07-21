use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use std::env;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    info!("🚀 === ARBITRAJE REAL SIMPLE EN DEVNET ===");
    info!("==========================================");

    // Load wallet from CLI format file directly
    let wallet_data = std::fs::read_to_string("test-cli-arbitrage.json")?;
    let wallet_bytes: Vec<u8> = serde_json::from_str(&wallet_data)?;
    let wallet_keypair = Keypair::from_bytes(&wallet_bytes)?;
    let wallet_pubkey = wallet_keypair.pubkey();
    info!("✅ Wallet cargado: {}", wallet_pubkey);

    // Create RPC client
    let rpc_url = "https://api.devnet.solana.com";
    let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    // Check initial balance
    info!("💰 Verificando balance inicial...");
    let initial_balance = rpc_client.get_balance(&wallet_pubkey)?;
    let initial_balance_sol = initial_balance as f64 / 1_000_000_000.0;
    info!("   Balance inicial: {:.9} SOL", initial_balance_sol);

    if initial_balance_sol < 0.02 {
        error!("❌ Balance insuficiente para arbitraje. Necesitas al menos 0.02 SOL");
        return Ok(());
    }

    // Simulate arbitrage by doing a real transfer loop to demonstrate real blockchain transactions
    info!("\n🎯 === EJECUTANDO ARBITRAJE SIMULADO REAL ===");
    info!("NOTA: Usando transferencias SOL reales para demostrar capacidad de transacciones");

    let arbitrage_amount = 10_000; // 0.00001 SOL - very small for demonstration

    // Step 1: Transfer to self as "buy" simulation
    info!("\n🔄 PASO 1: Ejecutando 'compra' simulada (transferencia real SOL)");
    let transfer_signature_1 = execute_real_transfer(
        &rpc_client,
        &wallet_keypair,
        &wallet_pubkey, // Transfer to self
        arbitrage_amount,
        "Simulando compra en DEX ficticio",
    )
    .await?;

    info!("✅ TRANSACCIÓN 1 CONFIRMADA: {}", transfer_signature_1);
    info!(
        "🔗 Explorer: https://explorer.solana.com/tx/{}?cluster=devnet",
        transfer_signature_1
    );

    // Wait for confirmation
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Step 2: Transfer to self as "sell" simulation
    info!("\n🔄 PASO 2: Ejecutando 'venta' simulada (transferencia real SOL)");
    let transfer_signature_2 = execute_real_transfer(
        &rpc_client,
        &wallet_keypair,
        &wallet_pubkey, // Transfer to self
        arbitrage_amount,
        "Simulando venta en DEX ficticio",
    )
    .await?;

    info!("✅ TRANSACCIÓN 2 CONFIRMADA: {}", transfer_signature_2);
    info!(
        "🔗 Explorer: https://explorer.solana.com/tx/{}?cluster=devnet",
        transfer_signature_2
    );

    // Wait for final confirmation
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Check final balance
    info!("\n💰 === RESULTADO FINAL ===");
    let final_balance = rpc_client.get_balance(&wallet_pubkey)?;
    let final_balance_sol = final_balance as f64 / 1_000_000_000.0;

    let net_change = final_balance as i64 - initial_balance as i64;
    let fees_paid = -(net_change as f64) / 1_000_000_000.0; // Negative because we paid fees

    info!("🏦 Balance inicial:  {:.9} SOL", initial_balance_sol);
    info!("🏦 Balance final:    {:.9} SOL", final_balance_sol);
    info!(
        "💰 Cambio neto:      {:.9} SOL",
        net_change as f64 / 1_000_000_000.0
    );
    info!("💸 Fees pagados:     {:.9} SOL", fees_paid);

    info!("\n🎯 === DEMOSTRACIÓN COMPLETADA ===");
    info!("✅ 2 transacciones REALES ejecutadas en DevNet blockchain");
    info!("✅ Sistema de transacciones funcionando correctamente");
    info!("✅ Wallet y RPC client operativos");
    info!("📊 Fees típicos: ~0.000005 SOL por transacción");
    info!("🚀 LISTO para implementar arbitraje real con DEXs que funcionen");

    info!("\n🎯 === PRÓXIMOS PASOS ===");
    info!("1. 🔍 Usar Jupiter solo para QUOTES (no para ejecución)");
    info!("2. 🔧 Implementar swaps directos con contratos de DEX");
    info!("3. 💰 Usar tokens que funcionan: BONK, RAY");
    info!("4. 🚀 O migrar a MainNet donde Jupiter tiene liquidez real");

    Ok(())
}

async fn execute_real_transfer(
    rpc_client: &RpcClient,
    from_keypair: &Keypair,
    to_pubkey: &solana_sdk::pubkey::Pubkey,
    amount_lamports: u64,
    memo: &str,
) -> Result<String> {
    info!(
        "   📤 Ejecutando transferencia real: {} lamports",
        amount_lamports
    );
    info!("   📝 Memo: {}", memo);

    // Create transfer instruction
    let transfer_instruction =
        system_instruction::transfer(&from_keypair.pubkey(), to_pubkey, amount_lamports);

    // Get recent blockhash
    let recent_blockhash = rpc_client.get_latest_blockhash()?;

    // Create and sign transaction
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_instruction],
        Some(&from_keypair.pubkey()),
        &[from_keypair],
        recent_blockhash,
    );

    // Send transaction to DevNet blockchain
    match rpc_client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => {
            info!("   ✅ Transferencia confirmada en blockchain");
            Ok(signature.to_string())
        }
        Err(e) => {
            error!("   ❌ Error en transferencia: {}", e);
            Err(anyhow::anyhow!("Transfer failed: {}", e))
        }
    }
}
