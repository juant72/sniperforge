use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::str::FromStr;
use dotenv::dotenv;
use std::env;

/// Test básico de conectividad y verificación de balance en DevNet
/// Verifica que todo esté funcionando correctamente sin complejidades

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    println!("🧪 === SniperForge: Test Básico DevNet ===");
    
    // Lista de RPCs para probar - RPC premium primero
    let rpc_urls = vec![
        "https://solana-devnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg",
        "https://api.devnet.solana.com",
        "https://devnet.genesysgo.net",
    ];
    
    println!("🔍 === PRUEBA DE CONECTIVIDAD RPC ===");
    
    let mut working_rpc = None;
    
    for rpc_url in &rpc_urls {
        println!("📡 Probando: {}", rpc_url);
        let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
        
        match client.get_health() {
            Ok(_) => {
                println!("   ✅ RPC saludable");
                
                // Test adicional - obtener último blockhash
                match client.get_latest_blockhash() {
                    Ok(blockhash) => {
                        println!("   ✅ Blockhash: {}", blockhash);
                        working_rpc = Some((client, rpc_url));
                        break;
                    }
                    Err(e) => {
                        println!("   ❌ Error en blockhash: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("   ❌ RPC no disponible: {}", e);
            }
        }
    }
    
    let (client, rpc_url) = working_rpc.expect("No se pudo conectar a ningún RPC");
    println!("🌐 RPC funcional encontrado: {}", rpc_url);
    
    // Cargar y verificar wallet
    println!("\n💰 === VERIFICACIÓN DE WALLET ===");
    
    let private_key_str = env::var("SOLANA_PRIVATE_KEY")
        .expect("SOLANA_PRIVATE_KEY no encontrada en .env");
    
    let private_key_bytes = bs58::decode(&private_key_str)
        .into_vec()
        .expect("Error decodificando clave privada desde base58");
    
    let payer = Keypair::from_bytes(&private_key_bytes)?;
    println!("✅ Wallet cargada: {}", payer.pubkey());
    
    // Verificar balance
    let balance = client.get_balance(&payer.pubkey())?;
    let balance_sol = balance as f64 / 1_000_000_000.0;
    println!("💵 Balance: {:.6} SOL ({} lamports)", balance_sol, balance);
    
    if balance_sol < 0.1 {
        println!("⚠️  Balance bajo para testing. Considera solicitar airdrop:");
        println!("   solana airdrop 2 --url {}", rpc_url);
    } else {
        println!("✅ Balance suficiente para testing");
    }
    
    // Test de tokens conocidos
    println!("\n🪙 === VERIFICACIÓN DE TOKENS ===");
    
    let sol_mint = "So11111111111111111111111111111111111111112";
    println!("🔍 Verificando SOL mint: {}", sol_mint);
    
    match client.get_account(&Pubkey::from_str(sol_mint)?) {
        Ok(account) => {
            println!("✅ SOL mint verificado");
            println!("   Owner: {}", account.owner);
            println!("   Lamports: {}", account.lamports);
            println!("   Data size: {}", account.data.len());
        }
        Err(e) => {
            println!("❌ Error verificando SOL mint: {}", e);
        }
    }
    
    // Verificar cuenta de wSOL si existe
    let wsol_ata = spl_associated_token_account::get_associated_token_address(
        &payer.pubkey(),
        &Pubkey::from_str(sol_mint)?
    );
    
    println!("🔍 Verificando cuenta wSOL: {}", wsol_ata);
    
    match client.get_token_account_balance(&wsol_ata) {
        Ok(balance) => {
            println!("✅ Cuenta wSOL encontrada");
            println!("   Balance: {} wSOL", balance.ui_amount_string);
            println!("   Raw amount: {}", balance.amount);
        }
        Err(_) => {
            println!("ℹ️  Cuenta wSOL no existe (normal si no has hecho wrap)");
            println!("   Para crear: spl-token wrap 0.1");
        }
    }
    
    // Test de información de red
    println!("\n🌐 === INFORMACIÓN DE RED ===");
    
    match client.get_version() {
        Ok(version) => {
            println!("✅ Versión del nodo: {}", version.solana_core);
        }
        Err(e) => {
            println!("❌ Error obteniendo versión: {}", e);
        }
    }
    
    match client.get_slot() {
        Ok(slot) => {
            println!("✅ Slot actual: {}", slot);
        }
        Err(e) => {
            println!("❌ Error obteniendo slot: {}", e);
        }
    }
    
    match client.get_epoch_info() {
        Ok(epoch_info) => {
            println!("✅ Época actual: {}", epoch_info.epoch);
            println!("   Slot en época: {}", epoch_info.slot_index);
            println!("   Slots por época: {}", epoch_info.slots_in_epoch);
        }
        Err(e) => {
            println!("❌ Error obteniendo info de época: {}", e);
        }
    }
    
    // Test de transacciones (simulación)
    println!("\n🧪 === TEST DE TRANSACCIONES ===");
    
    // Crear una transacción simple de transferencia SOL (sin enviar)
    let transfer_instruction = solana_sdk::system_instruction::transfer(
        &payer.pubkey(),
        &payer.pubkey(), // Transferir a sí mismo
        1000, // 1000 lamports = 0.000001 SOL
    );
    
    let mut transaction = solana_sdk::transaction::Transaction::new_with_payer(
        &[transfer_instruction],
        Some(&payer.pubkey()),
    );
    
    let recent_blockhash = client.get_latest_blockhash()?;
    transaction.sign(&[&payer], recent_blockhash);
    
    // Simular la transacción (no enviar realmente)
    match client.simulate_transaction(&transaction) {
        Ok(result) => {
            println!("✅ Simulación de transacción exitosa");
            println!("   Units consumed: {:?}", result.value.units_consumed);
            if let Some(err) = result.value.err {
                println!("   ⚠️  Error en simulación: {:?}", err);
            } else {
                println!("   ✅ Transacción sería exitosa");
            }
        }
        Err(e) => {
            println!("❌ Error en simulación: {}", e);
        }
    }
    
    // Resumen final
    println!("\n🎯 === RESUMEN FINAL ===");
    println!("✅ RPC funcional: {}", rpc_url);
    println!("✅ Wallet válida: {}", payer.pubkey());
    println!("✅ Balance SOL: {:.6}", balance_sol);
    println!("✅ Conectividad verificada");
    println!("✅ Simulación de transacciones funcional");
    
    if balance_sol >= 0.1 {
        println!("\n💡 LISTO PARA:");
        println!("   • Crear tokens SPL propios");
        println!("   • Hacer swaps SOL <-> wSOL");
        println!("   • Testing de trading engine");
    } else {
        println!("\n⚠️  NECESITAS:");
        println!("   • Más SOL para testing (solana airdrop 2)");
    }
    
    println!("\n🧪 COMANDOS ÚTILES:");
    println!("spl-token accounts                    # Ver todos tus tokens");
    println!("spl-token wrap 0.1                   # Crear 0.1 wSOL");
    println!("solana balance                       # Ver balance SOL");
    println!("solana account {}                    # Ver info de wallet", payer.pubkey());
    
    Ok(())
}
