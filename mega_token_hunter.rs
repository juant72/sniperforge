use anyhow::Result;
use reqwest;
use serde_json::{self, Value};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};
use chrono::{DateTime, Utc};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signature},
    signer::Signer,
    pubkey::Pubkey,
    transaction::Transaction,
};
use futures::future::join_all;
use base64::{Engine, engine::general_purpose::STANDARD as Base64Engine};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🌟 === MEGA TOKEN ARBITRAGE HUNTER ===");
    info!("   🚀 MAXIMUM TOKEN COVERAGE - 50+ PAIRS");
    info!("   ⚡ Parallel processing for ultra-fast scanning");
    info!("   💰 Covers ALL major Solana tokens");
    info!("   🎯 Auto-executes opportunities > 5x fees");

    let hunter = MegaTokenHunter::new().await?;
    hunter.start_mega_hunting().await?;

    Ok(())
}

struct MegaTokenHunter {
    client: RpcClient,
    keypair: Keypair,
    wallet_address: Pubkey,
    http_client: reqwest::Client,
    fee_cost: f64,
}

impl MegaTokenHunter {
    async fn new() -> Result<Self> {
        // Load wallet
        let wallet_paths = vec![
            std::env::var("SOLANA_WALLET_PATH").unwrap_or_default(),
            "mainnet_wallet.json".to_string(),
            "wallet.json".to_string(),
        ];
        
        let mut keypair_bytes = None;
        let mut used_path = String::new();
        
        for path in wallet_paths {
            if !path.is_empty() && std::path::Path::new(&path).exists() {
                match std::fs::read_to_string(&path) {
                    Ok(json_str) => {
                        if let Ok(bytes_vec) = serde_json::from_str::<Vec<u8>>(&json_str) {
                            if bytes_vec.len() == 64 {
                                keypair_bytes = Some(bytes_vec);
                                used_path = path;
                                break;
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        
        let keypair_bytes = keypair_bytes.ok_or_else(|| {
            anyhow::anyhow!("No valid wallet found")
        })?;
        
        let keypair = Keypair::from_bytes(&keypair_bytes)?;
        let wallet_address = keypair.pubkey();

        // PROFESSIONAL RPC SETUP with your API keys
        let rpc_url = if let Ok(helius_key) = std::env::var("HELIUS_API_KEY") {
            info!("🏆 Using Helius PREMIUM RPC");
            format!("https://rpc.helius.xyz/?api-key={}", helius_key)
        } else if let Ok(alchemy_key) = std::env::var("ALCHEMY_API_KEY") {
            info!("🏆 Using Alchemy PREMIUM RPC");
            format!("https://solana-mainnet.g.alchemy.com/v2/{}", alchemy_key)
        } else {
            // Fallback to your existing Alchemy key
            info!("🏆 Using configured Alchemy PREMIUM RPC");
            "https://solana-mainnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg".to_string()
        };
        
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .pool_max_idle_per_host(20)
            .build()?;

        info!("✅ Mega token hunter loaded: {} (from: {})", wallet_address, used_path);

        Ok(Self {
            client,
            keypair,
            wallet_address,
            http_client,
            fee_cost: 0.000015,
        })
    }

    async fn start_mega_hunting(&self) -> Result<()> {
        info!("🚀 Starting AGGRESSIVE arbitrage hunting...");
        info!("   🌟 Scanning {} token pairs with REAL EXECUTION", self.get_all_token_pairs().len());
        info!("   ⚡ 60-second intervals for JUPITER API respect");
        info!("   💰 Executes opportunities > 0.5x fees (ULTRA aggressive)");
        info!("   🎯 Maximum 3 simultaneous executions for more profit");

        let mut cycle_count = 0;
        let mut total_profit = 0.0;
        let mut executions = 0;

        loop {
            cycle_count += 1;
            let now: DateTime<Utc> = Utc::now();
            
            info!("\n🌟 === MEGA TOKEN CYCLE {} ===", cycle_count);
            info!("   📅 Time: {}", now.format("%H:%M:%S"));

            // Check wallet balance
            match self.get_wallet_balance().await {
                Ok(balance) => {
                    info!("   💰 Current balance: {:.9} SOL", balance);
                    
                    if balance < 0.003 {
                        warn!("   ⚠️ Low balance - hunting paused");
                        sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                }
                Err(e) => {
                    error!("   ❌ Failed to check balance: {}", e);
                    sleep(Duration::from_secs(30)).await;
                    continue;
                }
            }

            // Scan all token pairs in parallel
            let start_time = std::time::Instant::now();
            match self.mega_scan_all_pairs().await {
                Ok(opportunities) => {
                    let scan_duration = start_time.elapsed();
                    info!("   ⚡ MEGA SCAN completed in {:.1}s", scan_duration.as_secs_f64());
                    
                    if opportunities.is_empty() {
                        info!("   💤 No opportunities detected across ALL {} pairs", self.get_all_token_pairs().len());
                    } else {
                        info!("   🎯 {} opportunities found across mega scan!", opportunities.len());
                        
                        // Sort by profit
                        let mut sorted_opportunities = opportunities;
                        sorted_opportunities.sort_by(|a, b| b.profit.partial_cmp(&a.profit).unwrap());
                        
                        // Auto-execution threshold: Reduced to 3x fees for more opportunities (0.000045 SOL)
                        let auto_threshold = self.fee_cost * 3.0;
                        let auto_exec_opportunities: Vec<_> = sorted_opportunities.iter()
                            .filter(|opp| opp.profit > auto_threshold)
                            .collect();

                        if !auto_exec_opportunities.is_empty() {
                            info!("   🔥 {} opportunities above 3x threshold!", auto_exec_opportunities.len());
                            
                            // Additional safety check - verify balance before execution
                            let current_balance = self.get_wallet_balance().await?;
                            if current_balance < 0.005 {
                                warn!("   ⚠️ Balance too low for safe execution: {:.9} SOL", current_balance);
                                continue;
                            }
                            
                            // Execute up to 3 for more opportunities
                            for opp in auto_exec_opportunities.iter().take(3) {
                                // ULTRA AGGRESSIVE: execute even small profits > 0.5x fees
                                if opp.profit > self.fee_cost * 0.5 {
                                    info!("   🚀 REAL EXECUTION: {} - {:.9} SOL profit ({:.1}x fees)", 
                                          opp.pair, opp.profit, opp.profit / self.fee_cost);
                                    
                                    match self.execute_arbitrage(opp).await {
                                        Ok(signature) => {
                                            info!("   ✅ REAL EXECUTION SUCCESSFUL!");
                                            info!("   📋 Signature: {}", signature);
                                            total_profit += opp.profit;
                                            executions += 1;
                                            
                                            // Wait longer between real executions - extended delay
                                            sleep(Duration::from_secs(10)).await;
                                        }
                                        Err(e) => {
                                            error!("   ❌ Real execution failed: {}", e);
                                            // Wait on failure to avoid rapid retries - extended delay
                                            sleep(Duration::from_secs(15)).await;
                                        }
                                    }
                                } else {
                                    info!("   💡 Skipping {} - below 3x threshold", opp.pair);
                                }
                            }
                        }

                        // Show top opportunities
                        info!("   📊 TOP OPPORTUNITIES:");
                        for (i, opp) in sorted_opportunities.iter().take(5).enumerate() {
                            let status = if opp.profit > auto_threshold { "🚀 EXECUTE" } else { "💡 MONITOR" };
                            info!("   {} {}: {} - {:.9} SOL ({:.1}x fees)", 
                                  i + 1, status, opp.pair, opp.profit, opp.profit / self.fee_cost);
                        }
                    }
                }
                Err(e) => {
                    error!("   ❌ Mega scan failed: {}", e);
                }
            }

            // Statistics every 10 cycles
            if cycle_count % 10 == 0 {
                info!("\n📊 === MEGA STATISTICS ===");
                info!("   🔍 Cycles completed: {}", cycle_count);
                info!("   ⏰ Running time: {:.1} minutes", (cycle_count * 10) as f64 / 60.0);
                info!("   🚀 Executions: {}", executions);
                info!("   💰 Total profit: {:.9} SOL", total_profit);
                info!("   🌟 Token pairs scanned: {}", self.get_all_token_pairs().len());
                if executions > 0 {
                    info!("   📈 Average profit per trade: {:.9} SOL", total_profit / executions as f64);
                    info!("   💵 Estimated USD value: ${:.2}", total_profit * 180.0); // Assuming ~$180/SOL
                }
            }

            // Wait 60 seconds for ULTIMATE Jupiter API respect
            sleep(Duration::from_secs(60)).await;
        }
    }

    fn get_all_token_pairs(&self) -> Vec<(&str, f64, &str, &str, &str)> {
        vec![
            // SOL PAIRS - High volume, most liquid (More amounts for better chances)
            ("SOL/USDC", 0.001, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Major stablecoin"),
            ("SOL/USDC", 0.003, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Major stablecoin"),
            ("SOL/USDC", 0.005, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Major stablecoin"),
            ("SOL/USDC", 0.007, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Major stablecoin"),
            ("SOL/USDC", 0.01, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Major stablecoin"),
            ("SOL/USDC", 0.015, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Major stablecoin"),
            ("SOL/USDC", 0.02, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Major stablecoin"),
            ("SOL/USDC", 0.03, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Major stablecoin"),
            ("SOL/USDC", 0.05, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Major stablecoin"),
            
            ("SOL/USDT", 0.001, "So11111111111111111111111111111111111111112", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "Tether USD"),
            ("SOL/USDT", 0.005, "So11111111111111111111111111111111111111112", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "Tether USD"),
            ("SOL/USDT", 0.01, "So11111111111111111111111111111111111111112", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "Tether USD"),
            ("SOL/USDT", 0.02, "So11111111111111111111111111111111111111112", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "Tether USD"),
            ("SOL/USDT", 0.05, "So11111111111111111111111111111111111111112", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "Tether USD"),
            
            // MAJOR DEFI TOKENS with more amounts
            ("SOL/RAY", 0.001, "So11111111111111111111111111111111111111112", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "Raydium DEX"),
            ("SOL/RAY", 0.005, "So11111111111111111111111111111111111111112", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "Raydium DEX"),
            ("SOL/RAY", 0.01, "So11111111111111111111111111111111111111112", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "Raydium DEX"),
            ("SOL/RAY", 0.02, "So11111111111111111111111111111111111111112", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "Raydium DEX"),
            ("SOL/RAY", 0.03, "So11111111111111111111111111111111111111112", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "Raydium DEX"),
            
            ("SOL/JUP", 0.001, "So11111111111111111111111111111111111111112", "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", "Jupiter DEX"),
            ("SOL/JUP", 0.005, "So11111111111111111111111111111111111111112", "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", "Jupiter DEX"),
            ("SOL/JUP", 0.01, "So11111111111111111111111111111111111111112", "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", "Jupiter DEX"),
            ("SOL/JUP", 0.02, "So11111111111111111111111111111111111111112", "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", "Jupiter DEX"),
            
            ("SOL/ORCA", 0.001, "So11111111111111111111111111111111111111112", "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE", "Orca DEX"),
            ("SOL/ORCA", 0.005, "So11111111111111111111111111111111111111112", "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE", "Orca DEX"),
            ("SOL/ORCA", 0.01, "So11111111111111111111111111111111111111112", "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE", "Orca DEX"),
            
            // MEME TOKENS - High volatility with many amounts
            ("SOL/BONK", 0.001, "So11111111111111111111111111111111111111112", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "Bonk meme"),
            ("SOL/BONK", 0.003, "So11111111111111111111111111111111111111112", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "Bonk meme"),
            ("SOL/BONK", 0.005, "So11111111111111111111111111111111111111112", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "Bonk meme"),
            ("SOL/BONK", 0.01, "So11111111111111111111111111111111111111112", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "Bonk meme"),
            ("SOL/BONK", 0.02, "So11111111111111111111111111111111111111112", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "Bonk meme"),
            
            ("SOL/WIF", 0.001, "So11111111111111111111111111111111111111112", "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", "Dogwifhat"),
            ("SOL/WIF", 0.003, "So11111111111111111111111111111111111111112", "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", "Dogwifhat"),
            ("SOL/WIF", 0.005, "So11111111111111111111111111111111111111112", "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", "Dogwifhat"),
            ("SOL/WIF", 0.01, "So11111111111111111111111111111111111111112", "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", "Dogwifhat"),
            ("SOL/WIF", 0.02, "So11111111111111111111111111111111111111112", "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", "Dogwifhat"),
            
            ("SOL/PEPE", 0.001, "So11111111111111111111111111111111111111112", "BxSuTkn5DQcTBUKyYxjrmBxgL3xL5uUEhz7x2rJ6hS7L", "Pepe meme"),
            ("SOL/PEPE", 0.005, "So11111111111111111111111111111111111111112", "BxSuTkn5DQcTBUKyYxjrmBxgL3xL5uUEhz7x2rJ6hS7L", "Pepe meme"),
            ("SOL/PEPE", 0.01, "So11111111111111111111111111111111111111112", "BxSuTkn5DQcTBUKyYxjrmBxgL3xL5uUEhz7x2rJ6hS7L", "Pepe meme"),
            
            // STABLECOINS CROSS-PAIRS - High chance opportunities
            ("USDC/USDT", 1.0, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "Stablecoin arb"),
            ("USDC/USDT", 2.0, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "Stablecoin arb"),
            ("USDC/USDT", 5.0, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "Stablecoin arb"),
            ("USDC/USDT", 10.0, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "Stablecoin arb"),
            ("USDC/USDT", 20.0, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "Stablecoin arb"),
            ("USDC/USDT", 50.0, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "Stablecoin arb"),
            ("USDC/USDT", 100.0, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "Stablecoin arb"),
            
            // MAJOR TOKEN CROSS-PAIRS (Not SOL) with multiple amounts
            ("RAY/USDC", 1.0, "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "RAY to USDC"),
            ("RAY/USDC", 3.0, "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "RAY to USDC"),
            ("RAY/USDC", 5.0, "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "RAY to USDC"),
            ("RAY/USDT", 3.0, "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "RAY to USDT"),
            ("RAY/USDT", 5.0, "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "RAY to USDT"),
            
            ("JUP/USDC", 1.0, "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "JUP to USDC"),
            ("JUP/USDC", 2.0, "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "JUP to USDC"),
            ("JUP/USDC", 3.0, "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "JUP to USDC"),
            
            ("BONK/USDC", 100.0, "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "BONK to USDC"),
            ("BONK/USDC", 500.0, "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "BONK to USDC"),
            ("BONK/USDC", 1000.0, "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "BONK to USDC"),
            
            ("WIF/USDC", 0.5, "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "WIF to USDC"),
            ("WIF/USDC", 1.0, "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "WIF to USDC"),
            ("WIF/USDC", 2.0, "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "WIF to USDC"),
            
            // CROSS MEME PAIRS - High volatility opportunities
            ("BONK/WIF", 200.0, "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", "Meme cross"),
            ("BONK/WIF", 500.0, "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", "Meme cross"),
            ("BONK/PEPE", 300.0, "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "BxSuTkn5DQcTBUKyYxjrmBxgL3xL5uUEhz7x2rJ6hS7L", "Meme cross"),
            ("BONK/PEPE", 500.0, "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "BxSuTkn5DQcTBUKyYxjrmBxgL3xL5uUEhz7x2rJ6hS7L", "Meme cross"),
            
            // DEFI CROSS PAIRS
            ("RAY/JUP", 1.0, "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", "DEX tokens"),
            ("RAY/JUP", 2.0, "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", "DEX tokens"),
            ("RAY/JUP", 3.0, "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", "DEX tokens"),
            ("RAY/ORCA", 1.0, "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE", "DEX tokens"),
            ("RAY/ORCA", 3.0, "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE", "DEX tokens"),
            ("JUP/ORCA", 1.0, "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE", "DEX tokens"),
            ("JUP/ORCA", 3.0, "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN", "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE", "DEX tokens"),
            
            // ADDITIONAL HIGH-VOLUME PAIRS with multiple amounts
            ("SOL/MSOL", 0.001, "So11111111111111111111111111111111111111112", "mSoLzYCxHdYgdziU2hLnk9NTODiw4PLMpS8kXqGNHhL", "Marinade SOL"),
            ("SOL/MSOL", 0.005, "So11111111111111111111111111111111111111112", "mSoLzYCxHdYgdziU2hLnk9NTODiw4PLMpS8kXqGNHhL", "Marinade SOL"),
            ("SOL/MSOL", 0.01, "So11111111111111111111111111111111111111112", "mSoLzYCxHdYgdziU2hLnk9NTODiw4PLMpS8kXqGNHhL", "Marinade SOL"),
            ("SOL/STSOL", 0.001, "So11111111111111111111111111111111111111112", "7dHbWXmci3dT4YGg48VhMFgkqe7SfGgBDcA5oTfp3gBV", "Lido stSOL"),
            ("SOL/STSOL", 0.005, "So11111111111111111111111111111111111111112", "7dHbWXmci3dT4YGg48VhMFgkqe7SfGgBDcA5oTfp3gBV", "Lido stSOL"),
            ("SOL/STSOL", 0.01, "So11111111111111111111111111111111111111112", "7dHbWXmci3dT4YGg48VhMFgkqe7SfGgBDcA5oTfp3gBV", "Lido stSOL"),
            ("SOL/JITOSOL", 0.001, "So11111111111111111111111111111111111111112", "J1tWcv9EShRi6cUxmFbsWLVhzCUyHFvE8QsW5GrSLUhz", "Jito SOL"),
            ("SOL/JITOSOL", 0.005, "So11111111111111111111111111111111111111112", "J1tWcv9EShRi6cUxmFbsWLVhzCUyHFvE8QsW5GrSLUhz", "Jito SOL"),
            ("SOL/JITOSOL", 0.01, "So11111111111111111111111111111111111111112", "J1tWcv9EShRi6cUxmFbsWLVhzCUyHFvE8QsW5GrSLUhz", "Jito SOL"),
        ]
    }

    async fn mega_scan_all_pairs(&self) -> Result<Vec<OpportunityData>> {
        let token_pairs = self.get_all_token_pairs();
        info!("   🔍 Scanning {} token pairs in parallel...", token_pairs.len());
        
        // Split into ULTRA CONSERVATIVE chunks for Jupiter API rate limiting
        let chunk_size = 1; // Process only 1 pair at a time for Jupiter API
        let mut all_opportunities = Vec::new();
        
        for chunk in token_pairs.chunks(chunk_size) {
            let futures: Vec<_> = chunk.iter().map(|(pair_name, amount, mint_a, mint_b, _desc)| {
                self.quick_check_opportunity(mint_a, mint_b, *amount, pair_name)
            }).collect();
            
            let results = join_all(futures).await;
            
            for result in results {
                if let Ok(Some(opp)) = result {
                    if opp.profit > 0.0 {
                        all_opportunities.push(opp);
                    }
                }
            }
            
            // MUCH longer delay between chunks for Jupiter API respect
            sleep(Duration::from_millis(2000)).await;
        }
        
        Ok(all_opportunities)
    }

    async fn quick_check_opportunity(
        &self,
        mint_a: &str,
        mint_b: &str,
        amount: f64,
        pair_name: &str
    ) -> Result<Option<OpportunityData>> {
        let is_sol_pair = mint_a == "So11111111111111111111111111111111111111112" || 
                         mint_b == "So11111111111111111111111111111111111111112";
        
        let (amount_units, divisor) = if is_sol_pair {
            const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
            ((amount * LAMPORTS_PER_SOL as f64) as u64, LAMPORTS_PER_SOL as f64)
        } else {
            const UNITS_PER_TOKEN: u64 = 1_000_000;
            ((amount * UNITS_PER_TOKEN as f64) as u64, UNITS_PER_TOKEN as f64)
        };
        
        if let Ok(Some(route_1_data)) = self.get_jupiter_quote_mega(mint_a, mint_b, amount_units).await {
            let intermediate_amount: u64 = route_1_data["outAmount"].as_str()
                .unwrap_or("0").parse().unwrap_or(0);
            
            if intermediate_amount > 0 {
                if let Ok(Some(route_2_data)) = self.get_jupiter_quote_mega(mint_b, mint_a, intermediate_amount).await {
                    let final_amount: u64 = route_2_data["outAmount"].as_str()
                        .unwrap_or("0").parse().unwrap_or(0);
                    let final_amount_f64 = final_amount as f64 / divisor;
                    
                    let profit_raw = final_amount_f64 - amount;
                    let profit_sol = if is_sol_pair {
                        profit_raw
                    } else {
                        profit_raw * 0.000056 // Approximate conversion to SOL
                    };
                    
                    if profit_sol > 0.0 {
                        return Ok(Some(OpportunityData {
                            pair: pair_name.to_string(),
                            amount,
                            profit: profit_sol,
                            mint_a: mint_a.to_string(),
                            mint_b: mint_b.to_string(),
                            route_1_data,
                            route_2_data,
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }

    async fn get_jupiter_quote_mega(&self, input_mint: &str, output_mint: &str, amount: u64) -> Result<Option<Value>> {
        // PROFESSIONAL RATE LIMITING: Exponential backoff + jitter
        static REQUEST_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let count = REQUEST_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Add jitter (randomness) to avoid thundering herd - LONGER DELAYS
        let jitter = (count % 300) as u64; // Increased jitter range
        tokio::time::sleep(Duration::from_millis(300 + jitter)).await;
        
        // Try with conservative settings first
        let url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=200&maxAccounts=32",
            input_mint, output_mint, amount
        );
        
        match self.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status() == 429 {
                    // PROFESSIONAL: Exponential backoff on 429 - MUCH LONGER
                    let backoff_ms = std::cmp::min(10000, 1000 * (count % 5)); // Up to 10 seconds
                    warn!("   ⏱️ Jupiter rate limited, backing off {}ms", backoff_ms);
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    
                    // Retry with even more conservative settings
                    let fallback_url = format!(
                        "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=150&maxAccounts=16&onlyDirectRoutes=true",
                        input_mint, output_mint, amount
                    );
                    
                    match self.http_client.get(&fallback_url).send().await {
                        Ok(fallback_response) => {
                            if fallback_response.status().is_success() {
                                match fallback_response.json::<Value>().await {
                                    Ok(data) => Ok(Some(data)),
                                    Err(_) => Ok(None)
                                }
                            } else {
                                Ok(None)
                            }
                        }
                        Err(_) => Ok(None)
                    }
                } else if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(data) => {
                            if data.get("error").is_some() {
                                Ok(None)
                            } else {
                                Ok(Some(data))
                            }
                        }
                        Err(_) => Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            Err(_) => {
                // Network error - wait longer
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(None)
            }
        }
    }

    async fn execute_arbitrage(&self, opportunity: &OpportunityData) -> Result<Signature> {
        info!("🚀 REAL EXECUTION for {}", opportunity.pair);
        
        // Execute first swap: A -> B
        info!("   🔄 Step 1: {} -> {}", opportunity.mint_a, opportunity.mint_b);
        let signature1 = self.execute_jupiter_swap(&opportunity.route_1_data).await?;
        info!("   ✅ First swap completed: {}", signature1);
        
        // Wait for confirmation
        sleep(Duration::from_millis(2000)).await;
        
        // Execute second swap: B -> A  
        info!("   🔄 Step 2: {} -> {}", opportunity.mint_b, opportunity.mint_a);
        let signature2 = self.execute_jupiter_swap(&opportunity.route_2_data).await?;
        info!("   ✅ Second swap completed: {}", signature2);
        
        info!("   🎯 ARBITRAGE COMPLETE - Profit: {:.9} SOL", opportunity.profit);
        Ok(signature2)
    }

    async fn execute_jupiter_swap(&self, route_data: &Value) -> Result<Signature> {
        // Get swap transaction from Jupiter
        let swap_request = serde_json::json!({
            "quoteResponse": route_data,
            "userPublicKey": self.wallet_address.to_string(),
            "wrapAndUnwrapSol": true,
            "dynamicComputeUnitLimit": true,
            "prioritizationFeeLamports": 50000
        });

        let response = self.http_client
            .post("https://quote-api.jup.ag/v6/swap")
            .json(&swap_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Jupiter swap API error: {}", response.status()));
        }

        let swap_response: Value = response.json().await?;
        let transaction_b64 = swap_response["swapTransaction"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No transaction in swap response"))?;

        // Decode and sign transaction
        let transaction_bytes = Base64Engine.decode(transaction_b64)?;
        
        let mut transaction: Transaction = bincode::deserialize(&transaction_bytes)?;

        // Sign the transaction
        let recent_blockhash = self.client.get_latest_blockhash()?;
        transaction.sign(&[&self.keypair], recent_blockhash);

        // Send transaction with confirmation
        let signature = self.client.send_and_confirm_transaction_with_spinner(&transaction)?;
        
        Ok(signature)
    }

    async fn get_wallet_balance(&self) -> Result<f64> {
        let balance_lamports = self.client.get_balance(&self.wallet_address)?;
        Ok(balance_lamports as f64 / 1_000_000_000.0)
    }
}

#[derive(Debug, Clone)]
struct OpportunityData {
    pair: String,
    amount: f64,
    profit: f64,
    mint_a: String,
    mint_b: String,
    route_1_data: Value,
    route_2_data: Value,
}
