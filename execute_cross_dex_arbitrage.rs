use anyhow::Result;
use tracing::{info, warn, error};
use sniperforge::shared::jupiter_client::{JupiterClient, JupiterConfig};
use sniperforge::shared::orca_client::OrcaClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use solana_client::rpc_client::RpcClient;
use std::str::FromStr;
use std::fs;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🚀 === EJECUTOR DE ARBITRAJE CROSS-DEX REAL ===");
    info!("===============================================");
    
    // Configuración
    let rpc_url = "https://api.devnet.solana.com";
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let usdc_mint = Pubkey::from_str("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU")?; // DevNet USDC
    let arbitrage_amount = 100_000_000u64; // 0.1 SOL para empezar seguro
    
    // Cargar wallet
    info!("🔑 Cargando wallet...");
    let wallet_data = fs::read_to_string("test-arbitrage-wallet.json")
        .or_else(|_| {
            warn!("No se encontró test-arbitrage-wallet.json, intentando crear wallet...");
            // Crear wallet dinámicamente si no existe
            let new_keypair = Keypair::new();
            let wallet_bytes: Vec<u8> = new_keypair.to_bytes().to_vec();
            let wallet_json = serde_json::to_string(&wallet_bytes)?;
            fs::write("test-arbitrage-wallet.json", &wallet_json)?;
            info!("✅ Wallet creada automáticamente");
            Ok(wallet_json)
        })?;
    
    let wallet_bytes: Vec<u8> = serde_json::from_str(&wallet_data)?;
    let keypair = Keypair::from_bytes(&wallet_bytes)?;
    let wallet_pubkey = keypair.pubkey();
    
    info!("💼 Wallet address: {}", wallet_pubkey);
    
    // Verificar balance inicial
    let rpc_client = RpcClient::new(rpc_url);
    let initial_balance = rpc_client.get_balance(&wallet_pubkey)?;
    info!("💰 Balance inicial: {} SOL", initial_balance as f64 / 1_000_000_000.0);
    
    if initial_balance < arbitrage_amount {
        error!("❌ Balance insuficiente. Necesitas al menos {} SOL", arbitrage_amount as f64 / 1_000_000_000.0);
        info!("💡 Ejecuta: cargo run --bin request_devnet_airdrop");
        return Ok(());
    }
    
    // Inicializar clientes
    info!("🔧 Inicializando clientes Jupiter y Orca...");
    let jupiter_config = JupiterConfig::mainnet(); // Usar mainnet para precios reales
    let jupiter_client = JupiterClient::new(&jupiter_config).await?;
    let orca_client = OrcaClient::new("devnet");
    info!("✅ Clientes inicializados");
    
    loop {
        info!("\n🔍 === CICLO DE ARBITRAJE ===");
        
        // PASO 1: Análisis de precios
        info!("📊 Analizando precios en ambos DEXs...");
        
        // Precio en Jupiter (SOL -> USDC)
        let jupiter_quote = match jupiter_client.get_quote(&sol_mint, &usdc_mint, arbitrage_amount).await {
            Ok(quote) => quote,
            Err(e) => {
                warn!("⚠️ Error obteniendo precio Jupiter: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                continue;
            }
        };
        
        let jupiter_usdc_output = jupiter_quote.out_amount;
        let jupiter_sol_price = jupiter_usdc_output as f64 / 1_000_000.0 / (arbitrage_amount as f64 / 1_000_000_000.0);
        
        // Precio en Orca (SOL -> USDC)
        let orca_sol_price = match orca_client.get_price(&sol_mint).await {
            Ok(price) => price,
            Err(e) => {
                warn!("⚠️ Error obteniendo precio Orca: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                continue;
            }
        };
        
        info!("💵 Jupiter SOL: ${:.2}", jupiter_sol_price);
        info!("💵 Orca SOL: ${:.2}", orca_sol_price);
        
        // PASO 2: Calcular spread
        let spread_percent = ((jupiter_sol_price - orca_sol_price) / orca_sol_price) * 100.0;
        let profit_per_sol = jupiter_sol_price - orca_sol_price;
        let estimated_profit = profit_per_sol * (arbitrage_amount as f64 / 1_000_000_000.0);
        
        info!("📈 Spread: {:.2}%", spread_percent);
        info!("💰 Profit estimado: ${:.2}", estimated_profit);
        
        // PASO 3: Decidir si ejecutar arbitraje
        if spread_percent > 5.0 && profit_per_sol > 1.0 {
            info!("🎯 EJECUTANDO ARBITRAJE - Spread rentable detectado!");
            
            // PASO 3A: Comprar SOL en Orca (el más barato)
            info!("🛒 PASO 1: Comprando SOL en Orca a ${:.2}...", orca_sol_price);
            
            // Para DevNet, Orca es mock, así que solo simulamos la compra
            info!("✅ [SIMULADO] Compra en Orca completada");
            
            // PASO 3B: Vender SOL en Jupiter (el más caro)
            info!("💰 PASO 2: Vendiendo SOL en Jupiter a ${:.2}...", jupiter_sol_price);
            
            match jupiter_client.execute_swap(
                &keypair,
                &rpc_client,
                &sol_mint,
                &usdc_mint,
                arbitrage_amount,
                jupiter_quote.out_amount,
                100 // 1% slippage
            ).await {
                Ok(signature) => {
                    info!("✅ Swap ejecutado en Jupiter: {}", signature);
                    info!("🎉 ARBITRAJE COMPLETADO!");
                    
                    // Verificar balance final
                    let final_balance = rpc_client.get_balance(&wallet_pubkey)?;
                    let balance_change = final_balance as i64 - initial_balance as i64;
                    info!("💰 Balance final: {} SOL", final_balance as f64 / 1_000_000_000.0);
                    info!("📊 Cambio en balance: {} lamports ({:.6} SOL)", balance_change, balance_change as f64 / 1_000_000_000.0);
                    
                    if balance_change > 0 {
                        info!("🎉 ¡PROFIT REALIZADO! +{:.6} SOL", balance_change as f64 / 1_000_000_000.0);
                    } else {
                        warn!("📉 Pérdida detectada: {:.6} SOL", balance_change as f64 / 1_000_000_000.0);
                    }
                },
                Err(e) => {
                    error!("❌ Error ejecutando swap: {}", e);
                }
            }
            
            // Pausa después de ejecutar arbitraje
            info!("⏸️ Pausando 30 segundos después del arbitraje...");
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            
        } else {
            info!("⏳ Spread no rentable ({}%) o profit muy bajo (${:.2})", spread_percent, profit_per_sol);
            info!("💡 Esperando mejor oportunidad...");
        }
        
        // Pausa entre ciclos
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
    }
}
