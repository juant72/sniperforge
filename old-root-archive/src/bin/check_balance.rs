// ================================================================
// VERIFICADOR DE BALANCE RÁPIDO
// ================================================================

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

const WALLET_ADDRESS: &str = "JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7";
const RPC_URL: &str = "https://api.mainnet-beta.solana.com";

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 VERIFICADOR DE BALANCE");
    println!("========================");
    println!("📍 Wallet: {}", WALLET_ADDRESS);
    println!("🌐 RPC: {}", RPC_URL);
    println!();

    let client = RpcClient::new(RPC_URL);
    let wallet_pubkey = Pubkey::from_str(WALLET_ADDRESS)?;

    println!("⏳ Consultando balance...");
    
    match client.get_balance(&wallet_pubkey) {
        Ok(balance_lamports) => {
            let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
            println!("✅ BALANCE ACTUAL: {:.9} SOL", balance_sol);
            println!("💰 Equivalente: {} lamports", balance_lamports);
            
            // Verificar si es suficiente para trading
            if balance_sol >= 0.1 {
                println!("🟢 SUFICIENTE para arbitraje (>= 0.1 SOL)");
            } else if balance_sol >= 0.01 {
                println!("🟡 LIMITADO - Solo para gas fees (>= 0.01 SOL)");
                println!("⚠️  Recomendado: Depositar más SOL para arbitraje");
            } else {
                println!("🔴 INSUFICIENTE - Necesitas más SOL");
                println!("⚠️  Mínimo recomendado: 0.01 SOL para gas + 0.1 SOL para arbitraje");
            }
        }
        Err(e) => {
            println!("❌ Error consultando balance: {}", e);
        }
    }

    println!("\n📊 ANÁLISIS DE TRADING:");
    println!("• Gas fee por transacción: ~0.005 SOL");
    println!("• Mínimo para arbitraje: 0.1 SOL");
    println!("• Recomendado para trading activo: 1.0+ SOL");

    Ok(())
}
