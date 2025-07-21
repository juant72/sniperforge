use anyhow::Result;
use reqwest;
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
    signature::Keypair, signer::Signer,
};
use std::fs;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === PHASE 4B: ENHANCED JUPITER ARBITRAGE SCANNER ===");
    info!("   🎯 Multi-token scanning for bigger opportunities");
    info!("   💰 Testing: SOL/USDC, SOL/RAY, SOL/BONK");
    info!("   🔍 Amount scaling: 0.005 → 0.05 SOL");
    info!("   ⚠️  REAL MONEY - MAINNET");

    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new_with_timeout_and_commitment(
        rpc_url.to_string(),
        Duration::from_secs(30),
        CommitmentConfig::confirmed(),
    );

    let wallet = load_mainnet_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 MainNet Wallet: {}", user_pubkey);

    let initial_balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance: {:.9} SOL", initial_balance);

    // Test different token pairs and amounts
    let test_pairs = vec![
        (
            "SOL",
            "USDC",
            "So11111111111111111111111111111111111111112",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            1_000_000.0,
        ),
        (
            "SOL",
            "RAY",
            "So11111111111111111111111111111111111111112",
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
            1_000_000.0,
        ),
        (
            "SOL",
            "BONK",
            "So11111111111111111111111111111111111111112",
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            1_000_000_000.0,
        ),
    ];

    let test_amounts_sol = vec![0.005, 0.01, 0.02, 0.03];

    let mut best_opportunity: Option<ArbitrageResult> = None;
    let mut scan_count = 0;

    info!("\n🔍 === COMPREHENSIVE ARBITRAGE SCAN ===");

    for (token_a, token_b, mint_a, mint_b, token_b_decimals) in test_pairs {
        for &amount_sol in &test_amounts_sol {
            if amount_sol > initial_balance - 0.01 {
                continue; // Skip if not enough balance
            }

            scan_count += 1;
            info!(
                "\n🔍 Scan {}: {} ↔ {} ({} SOL)",
                scan_count, token_a, token_b, amount_sol
            );

            match scan_arbitrage_opportunity(mint_a, mint_b, amount_sol, token_b_decimals).await {
                Ok(Some(result)) => {
                    info!("   ✅ Opportunity found!");
                    info!("   💰 Profit: {:.9} SOL", result.profit);
                    info!("   📈 ROI: {:.4}%", result.roi);

                    if result.profit > 0.000015 {
                        info!("   🎯 PROFITABLE! (> fees)");
                        if best_opportunity.is_none()
                            || result.profit > best_opportunity.as_ref().unwrap().profit
                        {
                            best_opportunity = Some(result);
                            info!("   🏆 NEW BEST OPPORTUNITY!");
                        }
                    } else {
                        info!("   ⚠️  Small profit (< fees)");
                    }
                }
                Ok(None) => {
                    info!("   ❌ No opportunity");
                }
                Err(e) => {
                    warn!("   ❌ Scan failed: {}", e);
                }
            }

            // Rate limiting
            sleep(Duration::from_millis(300)).await;
        }
    }

    info!("\n📊 === FINAL SCAN RESULTS ===");
    info!("   🔍 Total scans: {}", scan_count);

    if let Some(best) = best_opportunity {
        info!("🏆 === BEST OPPORTUNITY FOUND ===");
        info!("   💰 Pair: {} ↔ {}", best.token_a, best.token_b);
        info!("   💰 Amount: {} SOL", best.amount_sol);
        info!("   📈 Profit: {:.9} SOL", best.profit);
        info!("   📈 ROI: {:.4}%", best.roi);
        info!("   ⚡ Spread: {:.6}%", best.spread_percent);

        if best.profit > 0.000050 {
            // Significant margin over fees
            info!("🎉 HIGHLY PROFITABLE OPPORTUNITY!");
            info!("   ✅ Recommended for execution");
            info!("   💡 Profit margin: {:.1}x fees", best.profit / 0.000015);
        } else if best.profit > 0.000015 {
            info!("🎯 MARGINAL OPPORTUNITY");
            info!("   ⚠️  Small margin over fees");
            info!("   💡 Risk: Slippage could eat profit");
        }
    } else {
        info!("❌ NO PROFITABLE OPPORTUNITIES");
        info!("   💡 Current market conditions:");
        info!("   📉 Spreads too small vs MainNet fees");
        info!("   🔄 Market efficiency high");
        info!("   ⏰ Try again during volatile periods");
    }

    info!("\n💡 === MARKET ANALYSIS ===");
    info!("   🌍 MainNet efficiency: High (small spreads)");
    info!("   ⚡ Best strategy: Volume + timing");
    info!("   💰 Minimum viable: 0.000050+ SOL profit");
    info!("   🎯 Sweet spot: Volatile periods, larger amounts");

    Ok(())
}

#[derive(Debug, Clone)]
struct ArbitrageResult {
    token_a: String,
    token_b: String,
    amount_sol: f64,
    profit: f64,
    roi: f64,
    spread_percent: f64,
}

async fn scan_arbitrage_opportunity(
    mint_a: &str,
    mint_b: &str,
    amount_sol: f64,
    token_b_decimals: f64,
) -> Result<Option<ArbitrageResult>> {
    let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

    // Route 1: A → B
    let route_1 = get_jupiter_quote(mint_a, mint_b, amount_lamports).await?;

    if let Some(route_1_data) = route_1 {
        let token_b_amount: u64 = route_1_data["outAmount"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        // Small delay
        sleep(Duration::from_millis(200)).await;

        // Route 2: B → A
        let route_2 = get_jupiter_quote(mint_b, mint_a, token_b_amount).await?;

        if let Some(route_2_data) = route_2 {
            let final_amount: u64 = route_2_data["outAmount"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            let final_sol = final_amount as f64 / LAMPORTS_PER_SOL as f64;

            let profit = final_sol - amount_sol;
            let roi = (profit / amount_sol) * 100.0;

            return Ok(Some(ArbitrageResult {
                token_a: "SOL".to_string(),
                token_b: if mint_b.contains("EPjF") {
                    "USDC"
                } else if mint_b.contains("4k3D") {
                    "RAY"
                } else {
                    "BONK"
                }
                .to_string(),
                amount_sol,
                profit,
                roi,
                spread_percent: roi,
            }));
        }
    }

    Ok(None)
}

async fn get_jupiter_quote(
    input_mint: &str,
    output_mint: &str,
    amount: u64,
) -> Result<Option<Value>> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=50",
        input_mint, output_mint, amount
    );

    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Value>().await {
                    Ok(data) => {
                        if data.get("error").is_some() {
                            Ok(None)
                        } else {
                            Ok(Some(data))
                        }
                    }
                    Err(_) => Ok(None),
                }
            } else {
                Ok(None)
            }
        }
        Err(_) => Ok(None),
    }
}

async fn check_sol_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<f64> {
    let balance = client.get_balance(pubkey)?;
    Ok(balance as f64 / LAMPORTS_PER_SOL as f64)
}

async fn load_mainnet_wallet() -> Result<Keypair> {
    let wallet_path = "mainnet-arbitrage-wallet.json";

    if std::path::Path::new(wallet_path).exists() {
        let wallet_data = fs::read_to_string(wallet_path)?;
        let secret_key: Vec<u8> = serde_json::from_str(&wallet_data)?;
        Ok(Keypair::from_bytes(&secret_key)?)
    } else {
        error!("❌ MainNet wallet not found: {}", wallet_path);
        anyhow::bail!("MainNet wallet file not found");
    }
}
