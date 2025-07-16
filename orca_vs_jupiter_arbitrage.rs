use std::time::Duration;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
};
use tracing::{info, warn, error};

use sniperforge::shared::{
    jupiter::{JupiterClient, JupiterConfig, QuoteRequest as JupiterQuoteRequest},
    orca_client::{OrcaClient, OrcaQuoteRequest},
};

// Tokens que funcionan en DevNet
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const BONK_MINT: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
const RAY_MINT: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === ARBITRAJE REAL ORCA vs JUPITER ===");
    info!("   Estrategia: Comparar precios directamente entre DEXs");
    info!("   Objetivo: Encontrar y ejecutar oportunidades reales");

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

    if initial_balance < 0.01 {
        error!("❌ Insuficiente SOL para arbitraje. Necesitas al menos 0.01 SOL");
        return Ok(());
    }

    // Inicializar clientes DEX
    info!("🔧 Inicializando clientes DEX...");
    let jupiter_client = JupiterClient::new(&JupiterConfig::default()).await?;
    let orca_client = OrcaClient::new("devnet");
    
    info!("✅ Jupiter client inicializado");
    info!("✅ Orca client inicializado");

    // Ejecutar análisis de arbitraje
    execute_arbitrage_analysis(&jupiter_client, &orca_client).await?;

    // Verificar balance final
    let final_balance = check_balance(&client, &user_pubkey).await?;
    info!("💰 Balance final: {} SOL", final_balance);
    
    let profit = final_balance - initial_balance;
    if profit > 0.0 {
        info!("🎉 Resultado: +{:.9} SOL", profit);
    } else {
        info!("📊 Resultado: {:.9} SOL (incluye fees)", profit);
    }

    Ok(())
}

async fn execute_arbitrage_analysis(
    jupiter_client: &JupiterClient,
    orca_client: &OrcaClient,
) -> Result<()> {
    info!("\n🔍 === ANÁLISIS DE PRECIOS MULTI-DEX ===");

    // Pares de tokens para analizar
    let token_pairs = vec![
        ("SOL", SOL_MINT, "BONK", BONK_MINT),
        ("SOL", SOL_MINT, "RAY", RAY_MINT),
        ("BONK", BONK_MINT, "RAY", RAY_MINT),
    ];

    // Cantidad para cotización (0.01 SOL = 10M lamports)
    let quote_amount = 10_000_000u64;

    for (from_symbol, from_mint, to_symbol, to_mint) in token_pairs {
        info!("\n🎯 === ANALIZANDO: {} -> {} ===", from_symbol, to_symbol);
        
        // Obtener cotización de Jupiter
        let jupiter_quote = get_jupiter_quote(
            jupiter_client, from_mint, to_mint, quote_amount
        ).await;

        // Obtener cotización de Orca
        let orca_quote = get_orca_quote(
            orca_client, from_mint, to_mint, quote_amount
        ).await;

        // Comparar y mostrar resultados
        compare_quotes(from_symbol, to_symbol, quote_amount, jupiter_quote, orca_quote).await;
        
        // Esperar un poco entre consultas
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}

async fn get_jupiter_quote(
    client: &JupiterClient,
    input_mint: &str,
    output_mint: &str,
    amount: u64,
) -> Option<JupiterQuoteResult> {
    info!("📡 Consultando Jupiter...");
    
    let request = JupiterQuoteRequest {
        inputMint: input_mint.to_string(),
        outputMint: output_mint.to_string(),
        amount,
        slippageBps: 100, // 1% slippage
    };

    match client.get_quote(request).await {
        Ok(quote) => {
            info!("✅ Jupiter: {} -> {} tokens", amount, quote.out_amount_units());
            Some(JupiterQuoteResult {
                input_amount: amount,
                output_amount: quote.out_amount_units(),
                price_impact: Some(quote.price_impact_pct()),
            })
        }
        Err(e) => {
            warn!("❌ Jupiter error: {}", e);
            None
        }
    }
}

async fn get_orca_quote(
    client: &OrcaClient,
    input_mint: &str,
    output_mint: &str,
    amount: u64,
) -> Option<OrcaQuoteResult> {
    info!("🌊 Consultando Orca...");
    
    let request = OrcaQuoteRequest {
        input_mint: input_mint.to_string(),
        output_mint: output_mint.to_string(),
        amount: amount.to_string(),
        slippage_bps: 100, // 1% slippage
    };

    match client.get_quote(&request).await {
        Ok(quote) => {
            let output_amount = quote.output_amount.parse::<u64>().unwrap_or(0);
            info!("✅ Orca: {} -> {} tokens", amount, output_amount);
            Some(OrcaQuoteResult {
                input_amount: amount,
                output_amount,
                price_impact: quote.price_impact_pct,
            })
        }
        Err(e) => {
            warn!("❌ Orca error: {}", e);
            None
        }
    }
}

#[derive(Debug)]
struct JupiterQuoteResult {
    input_amount: u64,
    output_amount: u64,
    price_impact: Option<f64>,
}

#[derive(Debug)]
struct OrcaQuoteResult {
    input_amount: u64,
    output_amount: u64,
    price_impact: Option<f64>,
}

async fn compare_quotes(
    from_symbol: &str,
    to_symbol: &str,
    input_amount: u64,
    jupiter_quote: Option<JupiterQuoteResult>,
    orca_quote: Option<OrcaQuoteResult>,
) {
    info!("\n📊 === COMPARACIÓN DE PRECIOS ===");
    info!("   Par: {} -> {}", from_symbol, to_symbol);
    info!("   Cantidad: {} tokens", input_amount);

    match (jupiter_quote, orca_quote) {
        (Some(jupiter), Some(orca)) => {
            info!("🔹 Jupiter: {} -> {} tokens", jupiter.input_amount, jupiter.output_amount);
            info!("🔹 Orca:    {} -> {} tokens", orca.input_amount, orca.output_amount);

            // Calcular diferencia de precios
            let jupiter_rate = jupiter.output_amount as f64 / jupiter.input_amount as f64;
            let orca_rate = orca.output_amount as f64 / orca.input_amount as f64;
            
            let difference_pct = if jupiter_rate > orca_rate {
                ((jupiter_rate - orca_rate) / orca_rate) * 100.0
            } else {
                ((orca_rate - jupiter_rate) / jupiter_rate) * 100.0
            };

            if difference_pct > 1.0 {
                let better_dex = if jupiter_rate > orca_rate { "Jupiter" } else { "Orca" };
                info!("🚨 ¡OPORTUNIDAD DE ARBITRAJE!");
                info!("   Mejor precio en: {}", better_dex);
                info!("   Diferencia: {:.2}%", difference_pct);
                
                if difference_pct > 5.0 {
                    info!("💰 ¡ARBITRAJE MUY RENTABLE! (+{:.2}%)", difference_pct);
                    
                    // Aquí ejecutarías el arbitraje real
                    execute_simple_arbitrage_demo(from_symbol, to_symbol, better_dex, difference_pct).await;
                }
            } else {
                info!("📈 Precios similares (diferencia: {:.2}%)", difference_pct);
            }
        }
        (Some(jupiter), None) => {
            info!("✅ Solo Jupiter disponible: {} -> {} tokens", jupiter.input_amount, jupiter.output_amount);
        }
        (None, Some(orca)) => {
            info!("✅ Solo Orca disponible: {} -> {} tokens", orca.input_amount, orca.output_amount);
        }
        (None, None) => {
            warn!("❌ No se pudieron obtener cotizaciones de ningún DEX");
        }
    }
}

async fn execute_simple_arbitrage_demo(
    from_symbol: &str,
    to_symbol: &str,
    better_dex: &str,
    profit_pct: f64,
) {
    info!("\n🚀 === EJECUTANDO ARBITRAJE DEMO ===");
    info!("   Par: {} -> {}", from_symbol, to_symbol);
    info!("   DEX óptimo: {}", better_dex);
    info!("   Profit esperado: {:.2}%", profit_pct);
    
    // Simulación de arbitraje
    info!("📝 Paso 1: Preparando transacción...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    info!("🔄 Paso 2: Ejecutando swap en {}...", better_dex);
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    info!("✅ Paso 3: Arbitraje completado (DEMO)");
    info!("💡 En producción, esto ejecutaría swaps reales");
    
    // Mostrar resultado simulado
    let simulated_profit = 0.001 * (profit_pct / 100.0); // Profit en SOL
    info!("📊 Profit simulado: +{:.6} SOL", simulated_profit);
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
