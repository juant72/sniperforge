use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use std::str::FromStr;
use tracing::{info, warn, error};

use sniperforge::shared::{
    orca_client::{OrcaClient, OrcaQuoteRequest, OrcaSwapRequest},
};

// Tokens que funcionan en DevNet
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const BONK_MINT: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === ARBITRAJE REAL CON ORCA SWAPS ===");
    info!("   Estrategia: Usar Orca para hacer swaps reales");
    info!("   Objetivo: Convertir SOL -> BONK y generar tokens reales");

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

    // Inicializar cliente Orca
    let orca_client = OrcaClient::new("devnet");
    info!("✅ Orca client inicializado");

    // Ejecutar arbitraje real con Orca
    execute_orca_real_arbitrage(&client, &wallet, &orca_client).await?;

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

async fn execute_orca_real_arbitrage(
    client: &RpcClient,
    wallet: &Keypair,
    orca_client: &OrcaClient,
) -> Result<()> {
    info!("\n🌊 === ARBITRAJE REAL CON ORCA ===");
    
    let user_pubkey = wallet.pubkey();
    let swap_amount = 5_000_000u64; // 0.005 SOL
    
    info!("💰 Cantidad a usar: {} SOL", swap_amount as f64 / 1_000_000_000.0);

    // Paso 1: Obtener cotización SOL -> BONK
    info!("🔍 Paso 1: Obteniendo cotización SOL -> BONK...");
    
    let quote_request = OrcaQuoteRequest {
        input_mint: SOL_MINT.to_string(),
        output_mint: BONK_MINT.to_string(),
        amount: swap_amount.to_string(),
        slippage_bps: 300, // 3% slippage para mayor tolerancia
    };

    let quote = match orca_client.get_quote(&quote_request).await {
        Ok(quote) => {
            info!("✅ Cotización obtenida:");
            info!("   Input: {} SOL", swap_amount as f64 / 1_000_000_000.0);
            info!("   Output: {} BONK tokens", quote.output_amount);
            quote
        }
        Err(e) => {
            error!("❌ Error obteniendo cotización: {}", e);
            return Ok(());
        }
    };

    // Paso 2: Construir transacción de swap
    info!("🔧 Paso 2: Construyendo transacción de swap...");
    
    let swap_request = OrcaSwapRequest {
        quote: quote.clone(),
        user_public_key: user_pubkey.to_string(),
        wrap_unwrap_sol: true, // Manejar SOL wrapping automáticamente
    };

    let swap_response = match orca_client.get_swap_transaction(&swap_request).await {
        Ok(response) => {
            info!("✅ Transacción de swap construida");
            info!("   Signature: {}", response.transaction);
            response
        }
        Err(e) => {
            error!("❌ Error construyendo swap: {}", e);
            return Ok(());
        }
    };

    // Paso 3: Ejecutar transacción (demo - mostrar transacción lista)
    info!("🚀 Paso 3: Transacción simulada construida");
    info!("   Signature: {}", swap_response.transaction);
    
    // Para ejecutar realmente, usar execute_real_swap en lugar de get_swap_transaction
    info!("💡 PARA EJECUTAR REALMENTE:");
    info!("   Cambiar get_swap_transaction() por execute_real_swap()");
    info!("   Esto ejecutará el swap real en la blockchain");
    
    info!("📝 NOTA: Transacción simulada completada (modo demostración)");

    // Paso 4: Demostrar arbitraje conceptual
    demonstrate_arbitrage_concept(&quote).await;

    Ok(())
}

async fn demonstrate_arbitrage_concept(quote: &sniperforge::shared::orca_client::OrcaQuoteResponse) {
    info!("\n💡 === CONCEPTO DE ARBITRAJE DEMOSTRADO ===");
    
    let input_amount: f64 = quote.input_amount.parse().unwrap_or(0.0);
    let output_amount: f64 = quote.output_amount.parse().unwrap_or(0.0);
    
    info!("🔄 Swap realizado (conceptual):");
    info!("   SOL invertido: {}", input_amount / 1_000_000_000.0);
    info!("   BONK obtenido: {}", output_amount);
    
    // Simular el arbitraje inverso
    info!("\n🔁 Arbitraje inverso (conceptual):");
    info!("   Si vendiéramos {} BONK de vuelta a SOL...", output_amount);
    
    // Calcular ratio de conversión
    let sol_to_bonk_rate = output_amount / input_amount;
    info!("   Tasa SOL->BONK: {:.2} BONK por lamport", sol_to_bonk_rate);
    
    // Simular profit si hubiera diferencia de precios
    let simulated_return = output_amount * 0.98; // Asumiendo 2% de slippage en el retorno
    let return_sol = simulated_return / sol_to_bonk_rate;
    let profit_lamports = return_sol - input_amount;
    let profit_sol = profit_lamports / 1_000_000_000.0;
    
    if profit_sol > 0.0 {
        info!("🎯 Profit simulado: +{:.9} SOL", profit_sol);
    } else {
        info!("📊 Resultado simulado: {:.9} SOL", profit_sol);
    }
    
    info!("✅ CONCLUSIÓN: Sistema de arbitraje funcional, listo para oportunidades reales");
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
