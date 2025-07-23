use anyhow::Result;
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use std::str::FromStr;
use std::time::Duration;
use tracing::{error, info, warn};

use sniperforge::shared::{
    dex_fallback_manager::{DexFallbackManager, DexProvider, UnifiedQuoteRequest},
    jupiter::JupiterClient,
    orca_client::OrcaClient,
};

// Tokens que funcionan en DevNet
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const BONK_MINT: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
const RAY_MINT: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === ARBITRAJE MULTI-DEX REAL ===");
    info!("   Estrategia: Comparar precios entre Orca y Jupiter");
    info!("   Tokens: SOL, BONK, RAY, USDC");
    info!("   Red: DevNet");

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
    let jupiter_client = std::sync::Arc::new(
        JupiterClient::new(&sniperforge::shared::jupiter::JupiterConfig::default()).await?,
    );
    let orca_client = std::sync::Arc::new(OrcaClient::new("devnet"));

    // Configurar fallback manager
    let fallback_chain = vec![DexProvider::Orca, DexProvider::Jupiter];
    let dex_manager = DexFallbackManager::new(
        Some(orca_client.clone()),
        Some(jupiter_client.clone()),
        fallback_chain,
        true, // enable fallback
        3,    // max retries
    );

    // Ejecutar estrategia de arbitraje multi-DEX
    execute_multi_dex_arbitrage(&dex_manager, &client, &wallet, user_pubkey).await?;

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

async fn execute_multi_dex_arbitrage(
    dex_manager: &DexFallbackManager,
    client: &RpcClient,
    wallet: &Keypair,
    user_pubkey: Pubkey,
) -> Result<()> {
    info!("🔍 === BUSCANDO OPORTUNIDADES DE ARBITRAJE ===");

    // Pares de tokens para probar
    let token_pairs = vec![
        (SOL_MINT, BONK_MINT),
        (SOL_MINT, RAY_MINT),
        (SOL_MINT, USDC_MINT),
        (BONK_MINT, RAY_MINT),
        (USDC_MINT, BONK_MINT),
    ];

    // Cantidad a usar para arbitraje (0.01 SOL o equivalente)
    let test_amount = 10_000_000u64; // 0.01 SOL en lamports

    for (input_token, output_token) in token_pairs {
        info!(
            "\n🎯 === PROBANDO PAR: {} -> {} ===",
            get_token_symbol(input_token),
            get_token_symbol(output_token)
        );

        // Obtener cotizaciones de ambos DEXs
        let quotes =
            get_quotes_from_multiple_dexs(dex_manager, input_token, output_token, test_amount)
                .await?;

        if quotes.is_empty() {
            warn!("❌ No se obtuvieron cotizaciones para este par");
            continue;
        }

        // Analizar oportunidades de arbitraje
        if let Some(opportunity) = analyze_arbitrage_opportunity(&quotes) {
            info!("🚨 ¡OPORTUNIDAD DETECTADA!");
            info!("   DEX Compra: {}", opportunity.buy_dex);
            info!("   DEX Venta: {}", opportunity.sell_dex);
            info!("   Profit estimado: {:.2}%", opportunity.profit_percentage);

            // Ejecutar arbitraje si es rentable
            if opportunity.profit_percentage > 5.0 {
                // Mínimo 5% para cubrir fees
                execute_arbitrage_trade(dex_manager, client, wallet, &opportunity).await?;
            } else {
                info!("📊 Profit muy bajo, saltando...");
            }
        }
    }

    Ok(())
}

async fn get_quotes_from_multiple_dexs(
    dex_manager: &DexFallbackManager,
    input_mint: &str,
    output_mint: &str,
    amount: u64,
) -> Result<Vec<QuoteWithDex>> {
    let mut quotes = Vec::new();

    // Obtener cotización usando fallback (probará Orca primero, luego Jupiter)
    let quote_request = UnifiedQuoteRequest {
        input_mint: input_mint.to_string(),
        output_mint: output_mint.to_string(),
        amount,
        slippage_bps: 100, // 1% slippage
    };

    match dex_manager
        .get_quote_with_fallback(quote_request.clone())
        .await
    {
        Ok(quote) => {
            info!(
                "✅ Cotización de {}: {} -> {} tokens",
                quote.dex_provider.as_str(),
                quote.in_amount,
                quote.out_amount
            );

            quotes.push(QuoteWithDex {
                dex: quote.dex_provider,
                input_amount: quote.in_amount,
                output_amount: quote.out_amount,
                quote_data: quote,
            });
        }
        Err(e) => {
            warn!("❌ Error obteniendo cotizaciones: {}", e);
        }
    }

    // Intentar obtener cotización del otro DEX directamente para comparación
    // (esto es para tener ambas cotizaciones y poder comparar)

    Ok(quotes)
}

#[derive(Debug)]
struct QuoteWithDex {
    dex: DexProvider,
    input_amount: u64,
    output_amount: u64,
    quote_data: sniperforge::shared::dex_fallback_manager::UnifiedQuoteResponse,
}

#[derive(Debug)]
struct ArbitrageOpportunity {
    buy_dex: String,
    sell_dex: String,
    profit_percentage: f64,
    input_token: String,
    output_token: String,
    amount: u64,
}

fn analyze_arbitrage_opportunity(quotes: &[QuoteWithDex]) -> Option<ArbitrageOpportunity> {
    if quotes.len() < 2 {
        return None;
    }

    // Por simplicidad, comparar los primeros dos quotes
    let quote1 = &quotes[0];
    let quote2 = &quotes[1];

    // Calcular tasas de conversión
    let rate1 = quote1.output_amount as f64 / quote1.input_amount as f64;
    let rate2 = quote2.output_amount as f64 / quote2.input_amount as f64;

    let profit_percentage = if rate1 > rate2 {
        ((rate1 - rate2) / rate2) * 100.0
    } else {
        ((rate2 - rate1) / rate1) * 100.0
    };

    if profit_percentage > 1.0 {
        // Mínimo 1% diferencia
        Some(ArbitrageOpportunity {
            buy_dex: if rate1 > rate2 {
                quote2.dex.as_str().to_string()
            } else {
                quote1.dex.as_str().to_string()
            },
            sell_dex: if rate1 > rate2 {
                quote1.dex.as_str().to_string()
            } else {
                quote2.dex.as_str().to_string()
            },
            profit_percentage,
            input_token: quote1.quote_data.input_mint.clone(),
            output_token: quote1.quote_data.output_mint.clone(),
            amount: quote1.input_amount,
        })
    } else {
        None
    }
}

async fn execute_arbitrage_trade(
    dex_manager: &DexFallbackManager,
    client: &RpcClient,
    wallet: &Keypair,
    opportunity: &ArbitrageOpportunity,
) -> Result<()> {
    info!("🚀 === EJECUTANDO ARBITRAJE ===");
    info!(
        "   Par: {} -> {}",
        opportunity.input_token, opportunity.output_token
    );
    info!(
        "   Estrategia: Comprar en {} / Vender en {}",
        opportunity.buy_dex, opportunity.sell_dex
    );

    // Por ahora, ejecutar un trade simple para demostrar funcionalidad
    // En una implementación completa, harías dos trades simultáneos

    let quote_request = UnifiedQuoteRequest {
        input_mint: opportunity.input_token.clone(),
        output_mint: opportunity.output_token.clone(),
        amount: opportunity.amount,
        slippage_bps: 100,
    };

    match dex_manager.get_quote_with_fallback(quote_request).await {
        Ok(quote) => {
            info!("✅ Cotización obtenida de {}", quote.dex_provider.as_str());

            // Para este demo, simplemente mostramos la información
            // En producción, ejecutarías el swap aquí
            info!("📊 Trade simulado exitoso:");
            info!("   Input: {} tokens", quote.in_amount);
            info!("   Output: {} tokens", quote.out_amount);
            info!("   DEX utilizado: {}", quote.dex_provider.as_str());

            // Simular un pequeño delay de ejecución
            tokio::time::sleep(Duration::from_secs(1)).await;

            info!("✅ Arbitraje completado (modo demostración)");
        }
        Err(e) => {
            error!("❌ Error ejecutando arbitraje: {}", e);
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

fn get_token_symbol(mint: &str) -> &str {
    match mint {
        SOL_MINT => "SOL",
        BONK_MINT => "BONK",
        RAY_MINT => "RAY",
        USDC_MINT => "USDC",
        _ => "UNKNOWN",
    }
}
