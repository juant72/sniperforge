use anyhow::Result;
use sniperforge::arbitrage::detector::ArbitrageDetector;
use sniperforge::arbitrage::types::ArbitrageSettings;
use sniperforge::shared::network_config::NetworkConfig;
use std::collections::HashMap;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    info!("🤖 === Bot de Arbitraje - DevNet Test ===");
    info!("==========================================");

    // Create DevNet configuration manually for testing
    let config = create_devnet_config();

    info!("✅ Configuración creada:");
    info!("   Network: {}", config.network);
    info!("   RPC: {}", config.rpc_endpoint);
    info!("   Tokens disponibles: {}", config.token_addresses.len());

    // Show configured tokens
    info!("🔧 Tokens configurados:");
    for (key, token) in &config.token_addresses {
        info!(
            "   {}: {} ({})",
            key.to_uppercase(),
            token.address,
            token.symbol
        );
    }

    // Create arbitrage detector
    info!("\n🔍 Inicializando detector de arbitraje...");
    let detector = ArbitrageDetector::new(config.clone()).await?;
    info!("✅ Detector inicializado correctamente");

    // Test arbitrage detection with real tokens
    info!("\n📊 === Iniciando detección de arbitraje ===");

    // Test 1: SOL -> USDC
    info!("\n📊 Test 1: SOL -> USDC");
    let sol_addr = &config.token_addresses["sol"].address;
    let usdc_addr = &config.token_addresses["usdc"].address;
    test_arbitrage_pair(
        &detector, sol_addr, usdc_addr, 0.001, // 0.001 SOL
        "SOL", "USDC",
    )
    .await?;

    // Test 2: SOL -> RAY
    info!("\n📊 Test 2: SOL -> RAY");
    let ray_addr = &config.token_addresses["ray"].address;
    test_arbitrage_pair(
        &detector, sol_addr, ray_addr, 0.001, // 0.001 SOL
        "SOL", "RAY",
    )
    .await?;

    // Test 3: USDC -> RAY
    info!("\n📊 Test 3: USDC -> RAY");
    test_arbitrage_pair(
        &detector, usdc_addr, ray_addr, 1.0, // 1 USDC
        "USDC", "RAY",
    )
    .await?;

    info!("\n🎯 === Test de arbitraje completado ===");
    Ok(())
}

fn create_devnet_config() -> NetworkConfig {
    use sniperforge::shared::network_config::{NetworkConfig, ProgramIds, TokenInfo};
    use std::str::FromStr;

    let mut token_addresses = HashMap::new();

    token_addresses.insert(
        "sol".to_string(),
        TokenInfo {
            address: "So11111111111111111111111111111111111111112".to_string(),
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            decimals: 9,
            verified: true,
            tradeable: true,
        },
    );

    token_addresses.insert(
        "usdc".to_string(),
        TokenInfo {
            address: "2iEJ8jXQN9xmhA7w9SpVgFGNkCYEsEf8LfEFumKjNRQo".to_string(),
            symbol: "USDC-Test".to_string(),
            name: "USD Coin Test".to_string(),
            decimals: 6,
            verified: true,
            tradeable: true,
        },
    );

    token_addresses.insert(
        "ray".to_string(),
        TokenInfo {
            address: "7QT4DzmTUfE3jTDHcmCPD9AHHRjkLGc9wKNp5JEJiSXW".to_string(),
            symbol: "RAY-Test".to_string(),
            name: "Raydium Test".to_string(),
            decimals: 6,
            verified: true,
            tradeable: true,
        },
    );

    let arbitrage_settings = Some(ArbitrageSettings {
        min_profit_threshold: 0.005,
        max_slippage: 0.01,
        detection_interval_ms: 1000,
        execution_timeout_ms: 30000,
        enabled: true,
    });

    NetworkConfig {
        network: "devnet".to_string(),
        rpc_endpoint: "https://solana-devnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg".to_string(),
        program_ids: ProgramIds {
            system_program: solana_sdk::pubkey::Pubkey::from_str(
                "11111111111111111111111111111111",
            )
            .unwrap(),
            token_program: solana_sdk::pubkey::Pubkey::from_str(
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            )
            .unwrap(),
            associated_token_program: solana_sdk::pubkey::Pubkey::from_str(
                "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
            )
            .unwrap(),
            compute_budget_program: solana_sdk::pubkey::Pubkey::from_str(
                "ComputeBudget111111111111111111111111111111",
            )
            .unwrap(),
            jupiter_program: Some(
                solana_sdk::pubkey::Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4")
                    .unwrap(),
            ),
            orca_whirlpool_program: Some(
                solana_sdk::pubkey::Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc")
                    .unwrap(),
            ),
            raydium_amm_program: None,
            spl_token_swap_program: Some(
                solana_sdk::pubkey::Pubkey::from_str("SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8")
                    .unwrap(),
            ),
        },
        token_addresses,
        arbitrage_settings,
    }
}

async fn test_arbitrage_pair(
    detector: &ArbitrageDetector,
    from_token: &str,
    to_token: &str,
    amount: f64,
    from_symbol: &str,
    to_symbol: &str,
) -> Result<()> {
    info!("   Testing {} {} -> {}", amount, from_symbol, to_symbol);
    info!("   From: {}", from_token);
    info!("   To: {}", to_token);

    match detector
        .detect_opportunities(from_token, to_token, amount)
        .await
    {
        Ok(opportunities) => {
            if opportunities.is_empty() {
                warn!("   ❌ No arbitrage opportunities found");
            } else {
                info!("   ✅ {} opportunities found:", opportunities.len());
                for (i, opp) in opportunities.iter().enumerate() {
                    info!("     [{}] {} -> {}", i + 1, opp.buy_dex, opp.sell_dex);
                    info!(
                        "         Profit: {:.6} SOL ({:.2}%)",
                        opp.profit_amount,
                        opp.profit_percentage * 100.0
                    );
                    info!("         Buy Price: {:.6}", opp.buy_price);
                    info!("         Sell Price: {:.6}", opp.sell_price);
                    info!(
                        "         Spread: {:.4}%",
                        (opp.sell_price - opp.buy_price) / opp.buy_price * 100.0
                    );
                }
            }
        }
        Err(e) => {
            error!("   ❌ Error detecting opportunities: {}", e);
        }
    }

    Ok(())
}
