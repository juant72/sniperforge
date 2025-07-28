// ================================================================
// WALLET MONITOR - VERIFICACIÓN DE INCREMENTOS DE BALANCE
// ================================================================

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use chrono::Local;

const WALLET_ADDRESS: &str = "JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7";
const RPC_URL: &str = "https://api.mainnet-beta.solana.com";

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 WALLET MONITOR - Sistema de Verificación de Incrementos");
    println!("==========================================================");
    println!("📍 Wallet: {}", WALLET_ADDRESS);
    println!("🌐 RPC: {}", RPC_URL);
    println!("⏰ Timestamp: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
    println!();

    let client = RpcClient::new(RPC_URL);
    let wallet_pubkey = Pubkey::from_str(WALLET_ADDRESS)?;

    // Balance inicial
    let initial_balance = get_balance(&client, &wallet_pubkey).await?;
    println!("💰 Balance Inicial: {:.9} SOL", initial_balance);
    println!();

    // Histórico de balances (simulado con checks cada 10 segundos)
    println!("📊 MONITOREANDO INCREMENTOS...");
    println!("Time              | Balance (SOL)   | Change      | Status");
    println!("------------------|-----------------|-------------|--------");

    let mut previous_balance = initial_balance;
    let mut check_count = 1;

    loop {
        sleep(Duration::from_secs(10)).await; // Check cada 10 segundos
        
        let current_balance = get_balance(&client, &wallet_pubkey).await?;
        let change = current_balance - previous_balance;
        let timestamp = Local::now().format("%H:%M:%S");
        
        let status = if change > 0.0 {
            "🟢 PROFIT+"
        } else if change < 0.0 {
            "🔴 LOSS-"
        } else {
            "🟡 STABLE"
        };

        println!(
            "{} | {:>13.9} | {:>+10.9} | {}",
            timestamp, current_balance, change, status
        );

        // Si hay incremento, mostrar detalles
        if change > 0.0 {
            println!("   ✅ INCREMENTO DETECTADO!");
            println!("   📈 Ganancia: +{:.9} SOL (~${:.2} USD)", 
                change, change * 180.0); // Assuming ~$180 per SOL
            
            // Obtener transacciones recientes
            if let Ok(recent_txs) = get_recent_transactions(&client, &wallet_pubkey).await {
                if !recent_txs.is_empty() {
                    println!("   🔗 Última transacción: {}", recent_txs[0]);
                }
            }
        }

        previous_balance = current_balance;
        check_count += 1;

        // Mostrar resumen cada 60 checks (10 minutos)
        if check_count % 60 == 0 {
            let total_change = current_balance - initial_balance;
            println!();
            println!("📋 RESUMEN ({} checks, {} minutos):", check_count, check_count / 6);
            println!("   💰 Balance Inicial: {:.9} SOL", initial_balance);
            println!("   💰 Balance Actual:  {:.9} SOL", current_balance);
            println!("   📊 Cambio Total:    {:+.9} SOL", total_change);
            if total_change != 0.0 {
                let percentage = (total_change / initial_balance) * 100.0;
                println!("   📈 Porcentaje:      {:+.4}%", percentage);
            }
            println!();
        }

        // Break si se ejecuta por mucho tiempo (opcional)
        if check_count > 360 { // 1 hora (360 checks de 10 segundos)
            println!("⏰ Monitor completado después de 1 hora");
            break;
        }
    }

    println!("✅ Monitoreo finalizado");
    Ok(())
}

async fn get_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<f64> {
    let balance_lamports = client.get_balance(pubkey)?;
    Ok(balance_lamports as f64 / 1_000_000_000.0) // Convert lamports to SOL
}

async fn get_recent_transactions(client: &RpcClient, pubkey: &Pubkey) -> Result<Vec<String>> {
    match client.get_signatures_for_address(pubkey) {
        Ok(signatures) => {
            Ok(signatures.into_iter().take(5).map(|sig| sig.signature).collect())
        }
        Err(_) => Ok(vec![])
    }
}
