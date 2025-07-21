use anyhow::Result;
use reqwest;
use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
    signature::Keypair, signer::Signer,
};
use std::fs;
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === PHASE 4: REAL JUPITER ARBITRAGE MAINNET ===");
    info!("   🎯 Objetivo: Real arbitrage usando spreads verdaderos");
    info!("   💰 Técnica: Jupiter routing cross-DEX");
    info!("   🎯 Target: Profit > 0.000015 SOL (fees breakeven)");
    info!("   ⚠️  REAL MONEY - MAINNET");

    let rpc_url = "https://api.mainnet-beta.solana.com";
    info!("🌐 Connecting to MainNet: {}", rpc_url);

    let client = RpcClient::new_with_timeout_and_commitment(
        rpc_url.to_string(),
        Duration::from_secs(30),
        CommitmentConfig::confirmed(),
    );

    // Test connection
    info!("🔍 Testing MainNet connection...");
    match client.get_health() {
        Ok(_) => info!("✅ MainNet RPC health: OK"),
        Err(e) => {
            error!("❌ MainNet RPC health failed: {}", e);
            return Ok(());
        }
    }

    let wallet = load_mainnet_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 MainNet Wallet: {}", user_pubkey);

    let initial_balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance inicial: {:.9} SOL", initial_balance);

    if initial_balance < 0.01 {
        error!("❌ Necesitas al menos 0.01 SOL para arbitrage real");
        error!("   💰 Current: {:.9} SOL", initial_balance);
        return Ok(());
    }

    info!("\n🎯 === REAL JUPITER ARBITRAGE SCAN ===");
    info!("   📚 Método: SOL → USDC → SOL cross-DEX routing");
    info!("   🔍 Buscando spreads reales entre rutas Jupiter");
    info!("   💡 Aprovechando diferencias temporales de liquidez");

    match execute_jupiter_arbitrage_scan(&user_pubkey).await {
        Ok(opportunity) => {
            if let Some(arb) = opportunity {
                info!("🎯 ¡OPORTUNIDAD DETECTADA!");
                info!("   💰 Expected profit: {:.9} SOL", arb.expected_profit);
                info!("   📈 ROI: {:.4}%", arb.roi);
                info!("   ⚡ Spread: {:.6}%", arb.spread_percent);

                if arb.expected_profit > 0.000020 {
                    // Más que fees + buffer
                    info!("✅ EJECUTANDO ARBITRAGE REAL...");
                    match execute_real_jupiter_arbitrage(&client, &wallet, &arb).await {
                        Ok(_) => {
                            sleep(Duration::from_secs(3)).await;
                            let final_balance = check_sol_balance(&client, &user_pubkey).await?;
                            let actual_profit = final_balance - initial_balance;

                            info!("🏁 === REAL JUPITER ARBITRAGE RESULTS ===");
                            info!("   💰 Balance inicial: {:.9} SOL", initial_balance);
                            info!("   💰 Balance final: {:.9} SOL", final_balance);
                            info!("   📈 Profit real: {:.9} SOL", actual_profit);

                            if actual_profit > 0.0 {
                                info!("🎉 ¡REAL MAINNET ARBITRAGE PROFIT!");
                                info!("   ✅ Expected: {:.9} SOL", arb.expected_profit);
                                info!("   ✅ Actual: {:.9} SOL", actual_profit);
                                let efficiency = (actual_profit / arb.expected_profit) * 100.0;
                                info!("   📊 Efficiency: {:.2}%", efficiency);
                            } else {
                                warn!("⚠️  Loss: {:.9} SOL", actual_profit);
                                info!("   💡 Spread disappeared or slippage too high");
                            }
                        }
                        Err(e) => error!("❌ Real arbitrage execution failed: {}", e),
                    }
                } else {
                    info!("⚠️  Profit too small: {:.9} SOL", arb.expected_profit);
                    info!("   💡 Need bigger spread to cover fees + slippage");
                }
            } else {
                info!("⚠️  No profitable opportunities found");
                info!("   💡 Spreads currently < fees + slippage");
            }
        }
        Err(e) => {
            error!("❌ Jupiter arbitrage scan failed: {}", e);
        }
    }

    Ok(())
}

#[derive(Debug)]
struct ArbitrageOpportunity {
    amount_sol: f64,
    route_1_output: f64, // SOL → USDC amount
    route_2_output: f64, // USDC → SOL amount
    expected_profit: f64,
    roi: f64,
    spread_percent: f64,
    route_1_data: Value,
    route_2_data: Value,
}

async fn execute_jupiter_arbitrage_scan(
    user_pubkey: &Pubkey,
) -> Result<Option<ArbitrageOpportunity>> {
    info!("🔍 === SCANNING JUPITER ROUTES ===");

    // Test amount for arbitrage
    let amount_sol = 0.005; // Start small
    let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

    info!("   💰 Test amount: {} SOL", amount_sol);

    // Token addresses (MainNet)
    let sol_mint = "So11111111111111111111111111111111111111112";
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    info!("   🔄 Route 1: SOL → USDC");
    let route_1 = get_jupiter_quote(sol_mint, usdc_mint, amount_lamports).await?;

    if let Some(route_1_data) = route_1 {
        let usdc_amount: u64 = route_1_data["outAmount"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);
        let usdc_amount_scaled = usdc_amount as f64 / 1_000_000.0; // USDC has 6 decimals

        info!(
            "   ✅ SOL → USDC: {} SOL → {} USDC",
            amount_sol, usdc_amount_scaled
        );

        // Small delay to avoid rate limiting
        sleep(Duration::from_millis(500)).await;

        info!("   🔄 Route 2: USDC → SOL");
        let route_2 = get_jupiter_quote(usdc_mint, sol_mint, usdc_amount).await?;

        if let Some(route_2_data) = route_2 {
            let final_sol_amount: u64 = route_2_data["outAmount"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            let final_sol_scaled = final_sol_amount as f64 / LAMPORTS_PER_SOL as f64;

            info!(
                "   ✅ USDC → SOL: {} USDC → {} SOL",
                usdc_amount_scaled, final_sol_scaled
            );

            // Calculate arbitrage metrics
            let profit = final_sol_scaled - amount_sol;
            let roi = (profit / amount_sol) * 100.0;
            let spread = (profit / amount_sol) * 100.0;

            info!("📊 === ARBITRAGE ANALYSIS ===");
            info!("   💰 Input: {} SOL", amount_sol);
            info!("   💰 Output: {} SOL", final_sol_scaled);
            info!("   📈 Profit: {:.9} SOL", profit);
            info!("   📈 ROI: {:.6}%", roi);
            info!("   ⚡ Spread: {:.6}%", spread);

            if profit > 0.000015 {
                // Must beat fees
                return Ok(Some(ArbitrageOpportunity {
                    amount_sol,
                    route_1_output: usdc_amount_scaled,
                    route_2_output: final_sol_scaled,
                    expected_profit: profit,
                    roi,
                    spread_percent: spread,
                    route_1_data,
                    route_2_data,
                }));
            } else {
                info!(
                    "⚠️  Spread too small: {:.9} SOL (< 0.000015 SOL fees)",
                    profit
                );
            }
        } else {
            warn!("⚠️  Route 2 (USDC → SOL) failed");
        }
    } else {
        warn!("⚠️  Route 1 (SOL → USDC) failed");
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

    info!(
        "   🌐 Jupiter quote: {} → {} (amount: {})",
        input_mint, output_mint, amount
    );

    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Value>().await {
                    Ok(data) => {
                        if data.get("error").is_some() {
                            warn!("   ❌ Jupiter error: {}", data["error"]);
                            Ok(None)
                        } else {
                            info!("   ✅ Quote successful");
                            Ok(Some(data))
                        }
                    }
                    Err(e) => {
                        warn!("   ❌ Parse error: {}", e);
                        Ok(None)
                    }
                }
            } else {
                warn!("   ❌ HTTP error: {}", response.status());
                Ok(None)
            }
        }
        Err(e) => {
            warn!("   ❌ Request error: {}", e);
            Ok(None)
        }
    }
}

async fn execute_real_jupiter_arbitrage(
    _client: &RpcClient,
    _wallet: &Keypair,
    opportunity: &ArbitrageOpportunity,
) -> Result<()> {
    info!("🚀 === EXECUTING REAL JUPITER ARBITRAGE ===");
    info!(
        "   💰 Expected profit: {:.9} SOL",
        opportunity.expected_profit
    );
    info!("   📈 ROI: {:.4}%", opportunity.roi);

    // TODO: Implement actual Jupiter swap execution
    // This would involve:
    // 1. Get swap transaction from Jupiter API
    // 2. Sign and send transaction
    // 3. Wait for confirmation
    // 4. Execute second swap
    // 5. Final confirmation

    info!("⚠️  REAL EXECUTION NOT IMPLEMENTED YET");
    info!("   📚 This scan confirms profitable opportunities exist");
    info!("   🎯 Next step: Implement Jupiter swap execution");

    Ok(())
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
        error!("   📝 Run first: cargo run --bin setup_mainnet_wallet");
        anyhow::bail!("MainNet wallet file not found");
    }
}
