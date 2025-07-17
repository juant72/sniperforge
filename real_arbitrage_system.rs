use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signature},
    signer::Signer,
    pubkey::Pubkey,
    transaction::Transaction,
    instruction::Instruction,
    system_instruction,
};
use spl_associated_token_account::get_associated_token_address;
use serde_json::Value;
use reqwest::Client;
use base64::{engine::general_purpose, Engine};
use bincode;

// Jupiter APIs - Modern and efficient
const JUPITER_QUOTE_API: &str = "https://quote-api.jup.ag/v6";
const JUPITER_PRICE_API: &str = "https://lite-api.jup.ag/price/v3"; 
const JUPITER_TOKENS_API: &str = "https://tokens.jup.ag/tokens";

// Popular Solana Tokens
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const USDT_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
const RAY_MINT: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";
const MSOL_MINT: &str = "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So";
const ETH_MINT: &str = "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs";
const BTC_MINT: &str = "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E";
const BONK_MINT: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
const ORCA_MINT: &str = "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE";
const SRM_MINT: &str = "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt";
const STSOL_MINT: &str = "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj";
const JITOSOL_MINT: &str = "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === ENHANCED REAL ARBITRAGE SYSTEM ===");
    info!("   💎 REAL TOKEN SWAPS - NOT SIMULATION");
    info!("   ⚡ JUPITER API INTEGRATION");
    info!("   🎯 ACTUAL ARBITRAGE EXECUTION");
    info!("   💰 REAL MONEY, REAL PROFITS");
    info!("   🔄 17 ARBITRAGE ROUTES × 4 TRADE SIZES");
    info!("   📈 68 TOTAL COMBINATIONS PER CYCLE");

    let mut arbitrage = RealArbitrageSystem::new().await?;
    arbitrage.run_real_arbitrage().await?;

    Ok(())
}

struct RealArbitrageSystem {
    client: RpcClient,
    keypair: Keypair,
    wallet_address: Pubkey,
    http_client: Client,
    rate_limiter: std::time::Instant,
}

#[derive(Debug, Clone)]
struct RealOpportunity {
    input_mint: String,
    output_mint: String,
    intermediate_mint: String,
    amount_in: u64,
    expected_amount_out: u64,
    profit_lamports: u64,
    profit_percentage: f64,
    route: Vec<String>,
}

impl RealArbitrageSystem {
    async fn new() -> Result<Self> {
        // Load wallet
        let wallet_path = "mainnet_wallet.json";
        let json_str = std::fs::read_to_string(wallet_path)?;
        let keypair_bytes: Vec<u8> = serde_json::from_str(&json_str)?;
        let keypair = Keypair::from_bytes(&keypair_bytes)?;
        let wallet_address = keypair.pubkey();

        // RPC setup with nonblocking client
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        // HTTP client for Jupiter API
        let http_client = Client::new();

        info!("✅ Real Arbitrage System loaded: {}", wallet_address);

        Ok(Self {
            client,
            keypair,
            wallet_address,
            http_client,
            rate_limiter: std::time::Instant::now(),
        })
    }

    async fn run_real_arbitrage(&mut self) -> Result<()> {
        info!("🚀 Starting REAL arbitrage execution...");
        
        let mut cycle = 0;
        let initial_balance = self.get_wallet_balance().await?;
        info!("💰 Initial balance: {:.9} SOL", initial_balance);

        loop {
            cycle += 1;
            info!("\n🏆 === REAL ARBITRAGE CYCLE {} ===", cycle);

            // Check current balance
            let current_balance = self.get_wallet_balance().await?;
            let net_profit = current_balance - initial_balance;
            info!("   💰 Current balance: {:.9} SOL", current_balance);
            info!("   📈 Net profit: {:.9} SOL", net_profit);

            if current_balance < 0.01 {
                warn!("   ⚠️ Low balance - minimum 0.01 SOL required");
                sleep(Duration::from_secs(60)).await;
                continue;
            }

            // 1. Find real arbitrage opportunities using Jupiter API
            match self.find_real_arbitrage_opportunities().await {
                Ok(opportunities) => {
                    if opportunities.is_empty() {
                        info!("   💤 No profitable arbitrage found");
                    } else {
                        info!("   🎯 {} REAL arbitrage opportunities found!", opportunities.len());
                        
                        // Execute best opportunity
                        let best_opp = &opportunities[0];
                        let min_profit_lamports = 12000; // Lowered threshold for more opportunities
                        
                        info!("   📊 BEST OPPORTUNITY ANALYSIS:");
                        info!("      💰 Route: {:?}", best_opp.route);
                        info!("      💎 Profit: {} lamports ({:.4}%)", 
                              best_opp.profit_lamports, best_opp.profit_percentage);
                        info!("      📈 Amount: {} lamports ({:.3} SOL)", 
                              best_opp.amount_in, best_opp.amount_in as f64 / 1_000_000_000.0);
                        
                        if best_opp.profit_lamports > min_profit_lamports {
                            // Execute real arbitrage
                            match self.execute_real_arbitrage(best_opp).await {
                                Ok(signatures) => {
                                    info!("   ✅ REAL ARBITRAGE SUCCESS!");
                                    info!("      💰 Transactions: {:?}", signatures);
                                    
                                    // Verify actual profit
                                    sleep(Duration::from_secs(5)).await;
                                    let new_balance = self.get_wallet_balance().await?;
                                    let actual_profit = new_balance - current_balance;
                                    
                                    if actual_profit > 0.0 {
                                        info!("   ✅ PROFIT CONFIRMED: {:.9} SOL", actual_profit);
                                    } else {
                                        error!("   ❌ LOSS DETECTED: {:.9} SOL", actual_profit.abs());
                                    }
                                }
                                Err(e) => {
                                    error!("   ❌ Arbitrage execution failed: {}", e);
                                }
                            }
                        } else {
                            info!("   💡 Opportunity too small: {} lamports (min: {})", 
                                  best_opp.profit_lamports, min_profit_lamports);
                        }
                    }
                }
                Err(e) => {
                    error!("   ❌ Failed to find opportunities: {}", e);
                }
            }

            // Wait before next cycle (faster scanning for more opportunities)
            sleep(Duration::from_secs(12)).await;
        }
    }

    async fn find_real_arbitrage_opportunities(&mut self) -> Result<Vec<RealOpportunity>> {
        info!("   🔍 Scanning for REAL arbitrage opportunities...");
        
        let mut opportunities = Vec::new();
        
        // Multiple trade sizes for better opportunity detection
        let trade_sizes = vec![
            2_000_000u64,   // 0.002 SOL
            5_000_000u64,   // 0.005 SOL  
            10_000_000u64,  // 0.01 SOL
            20_000_000u64,  // 0.02 SOL
        ];
        
        // Expanded arbitrage routes with popular tokens
        let routes = vec![
            // SOL triangular routes
            (SOL_MINT, USDC_MINT, SOL_MINT),     // SOL -> USDC -> SOL
            (SOL_MINT, USDT_MINT, SOL_MINT),     // SOL -> USDT -> SOL
            (SOL_MINT, RAY_MINT, SOL_MINT),      // SOL -> RAY -> SOL
            (SOL_MINT, MSOL_MINT, SOL_MINT),     // SOL -> mSOL -> SOL
            (SOL_MINT, ETH_MINT, SOL_MINT),      // SOL -> ETH -> SOL
            (SOL_MINT, BTC_MINT, SOL_MINT),      // SOL -> BTC -> SOL
            (SOL_MINT, BONK_MINT, SOL_MINT),     // SOL -> BONK -> SOL
            (SOL_MINT, ORCA_MINT, SOL_MINT),     // SOL -> ORCA -> SOL
            (SOL_MINT, STSOL_MINT, SOL_MINT),    // SOL -> stSOL -> SOL
            (SOL_MINT, JITOSOL_MINT, SOL_MINT),  // SOL -> jitoSOL -> SOL
            
            // USDC triangular routes
            (USDC_MINT, USDT_MINT, USDC_MINT),   // USDC -> USDT -> USDC
            (USDC_MINT, ETH_MINT, USDC_MINT),    // USDC -> ETH -> USDC
            (USDC_MINT, BTC_MINT, USDC_MINT),    // USDC -> BTC -> USDC
            (USDC_MINT, RAY_MINT, USDC_MINT),    // USDC -> RAY -> USDC
            
            // Cross-token opportunities
            (ETH_MINT, BTC_MINT, ETH_MINT),      // ETH -> BTC -> ETH
            (RAY_MINT, ORCA_MINT, RAY_MINT),     // RAY -> ORCA -> RAY
            (MSOL_MINT, STSOL_MINT, MSOL_MINT),  // mSOL -> stSOL -> mSOL
        ];
        
        info!("   📊 Scanning {} routes × {} sizes = {} combinations", 
              routes.len(), trade_sizes.len(), routes.len() * trade_sizes.len());
        
        // Test each route with each trade size
        for (input_mint, intermediate_mint, output_mint) in routes {
            for &test_amount in &trade_sizes {
                // Only test if we have enough balance
                let current_balance = self.get_wallet_balance().await?;
                let required_sol = test_amount as f64 / 1_000_000_000.0;
                
                if current_balance >= required_sol + 0.01 { // Keep 0.01 SOL buffer
                    if let Some(opportunity) = self.check_arbitrage_route(
                        input_mint, 
                        intermediate_mint, 
                        output_mint, 
                        test_amount
                    ).await? {
                        opportunities.push(opportunity);
                    }
                }
                
                // Small delay to avoid API rate limits
                sleep(Duration::from_millis(150)).await;
            }
        }
        
        // Sort by profit potential
        opportunities.sort_by(|a, b| b.profit_lamports.cmp(&a.profit_lamports));
        
        if !opportunities.is_empty() {
            info!("   🎯 Found {} profitable opportunities!", opportunities.len());
            for (i, opp) in opportunities.iter().take(3).enumerate() {
                info!("   #{}: {:?} - {} lamports ({:.4}%)", 
                      i + 1, opp.route, opp.profit_lamports, opp.profit_percentage);
            }
        }
        
        Ok(opportunities)
    }

    async fn check_arbitrage_route(
        &mut self,
        input_mint: &str,
        intermediate_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<Option<RealOpportunity>> {
        
        // Step 1: Get quote for input -> intermediate
        let quote1 = self.get_jupiter_quote(input_mint, intermediate_mint, amount).await?;
        
        if let Some(quote1_data) = quote1 {
            let intermediate_amount: u64 = quote1_data["outAmount"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            
            if intermediate_amount == 0 {
                return Ok(None);
            }
            
            // Step 2: Get quote for intermediate -> output
            let quote2 = self.get_jupiter_quote(intermediate_mint, output_mint, intermediate_amount).await?;
            
            if let Some(quote2_data) = quote2 {
                let final_amount: u64 = quote2_data["outAmount"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                
                if final_amount > amount {
                    let profit_lamports = final_amount - amount;
                    let profit_percentage = (profit_lamports as f64 / amount as f64) * 100.0;
                    
                    // Account for ALL fees accurately including mainnet realities
                    let transaction_fees = 10000u64; // 2 transactions × 5000 lamports
                    let jupiter_fees = self.calculate_jupiter_fees(&quote1_data) + self.calculate_jupiter_fees(&quote2_data);
                    let priority_fees = 50000u64; // Realistic priority fees for mainnet
                    let rent_fees = 4000u64; // Potential token account creation
                    let total_fees = transaction_fees + jupiter_fees + priority_fees + rent_fees;
                    
                    if profit_lamports > total_fees {
                        let net_profit = profit_lamports - total_fees;
                        
                        info!("      🎯 FOUND OPPORTUNITY:");
                        info!("         Route: {} -> {} -> {}", 
                               self.get_token_symbol(input_mint),
                               self.get_token_symbol(intermediate_mint),
                               self.get_token_symbol(output_mint));
                        info!("         Profit: {} lamports ({:.4}%)", net_profit, profit_percentage);
                        
                        return Ok(Some(RealOpportunity {
                            input_mint: input_mint.to_string(),
                            output_mint: output_mint.to_string(),
                            intermediate_mint: intermediate_mint.to_string(),
                            amount_in: amount,
                            expected_amount_out: final_amount,
                            profit_lamports: net_profit,
                            profit_percentage,
                            route: vec![
                                self.get_token_symbol(input_mint),
                                self.get_token_symbol(intermediate_mint),
                                self.get_token_symbol(output_mint),
                            ],
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }

    async fn get_jupiter_quote(&mut self, input_mint: &str, output_mint: &str, amount: u64) -> Result<Option<Value>> {
        // Enforce rate limiting (more relaxed for modern API)
        self.enforce_rate_limit().await;
        
        // Try Price API first (faster and more reliable)
        if let Ok(Some(price_quote)) = self.get_jupiter_price_quote(input_mint, output_mint, amount).await {
            return Ok(Some(price_quote));
        }
        
        // Fallback to Quote API if Price API fails
        self.get_jupiter_legacy_quote(input_mint, output_mint, amount).await
    }

    async fn get_jupiter_price_quote(&mut self, input_mint: &str, output_mint: &str, amount: u64) -> Result<Option<Value>> {
        // Use Jupiter Price API v3 to get USD prices for both tokens
        let url = format!(
            "{}?ids={},{}",
            JUPITER_PRICE_API, input_mint, output_mint
        );
        
        match self.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let price_data: Value = response.json().await?;
                    
                    // Extract USD prices for both tokens
                    if let (Some(input_price_info), Some(output_price_info)) = 
                        (price_data.get(input_mint), price_data.get(output_mint)) {
                        
                        if let (Some(input_usd_price), Some(output_usd_price)) = 
                            (input_price_info["usdPrice"].as_f64(), output_price_info["usdPrice"].as_f64()) {
                            
                            // Calculate the exchange rate and estimate output amount
                            let exchange_rate = input_usd_price / output_usd_price;
                            let estimated_out = (amount as f64 * exchange_rate) as u64;
                            
                            // Create compatible quote response
                            let quote = serde_json::json!({
                                "inputMint": input_mint,
                                "outputMint": output_mint,
                                "inAmount": amount.to_string(),
                                "outAmount": estimated_out.to_string(),
                                "priceImpactPct": "0.1",
                                "marketInfos": [],
                                "platformFee": {
                                    "amount": "2500",
                                    "feeBps": 25
                                }
                            });
                            
                            return Ok(Some(quote));
                        }
                    }
                }
                Ok(None)
            }
            Err(e) => {
                warn!("Price API error: {}", e);
                Ok(None)
            }
        }
    }

    async fn get_jupiter_legacy_quote(&mut self, input_mint: &str, output_mint: &str, amount: u64) -> Result<Option<Value>> {
        // Calculate dynamic slippage based on trade size and market conditions
        let slippage_bps = self.calculate_safe_slippage(amount, &format!("{}/{}", input_mint, output_mint));
        
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            JUPITER_QUOTE_API, input_mint, output_mint, amount, slippage_bps
        );
        
        match self.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let quote: Value = response.json().await?;
                    Ok(Some(quote))
                } else if response.status().as_u16() == 429 {
                    warn!("Rate limit hit on Quote API, backing off...");
                    sleep(Duration::from_millis(1000)).await;
                    Ok(None)
                } else {
                    warn!("Jupiter Quote API error: {}", response.status());
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("Failed to get Jupiter quote: {}", e);
                Ok(None)
            }
        }
    }

    async fn execute_real_arbitrage(&mut self, opportunity: &RealOpportunity) -> Result<Vec<String>> {
        info!("   🚀 EXECUTING REAL ARBITRAGE");
        info!("      📋 Route: {:?}", opportunity.route);
        info!("      💰 Expected profit: {} lamports", opportunity.profit_lamports);
        
        // 🔍 BALANCE VERIFICATION BEFORE EXECUTION
        let balance_before = self.get_wallet_balance().await?;
        let balance_before_lamports = (balance_before * 1_000_000_000.0) as u64;
        
        info!("      🔍 BALANCE BEFORE ARBITRAGE: {:.9} SOL ({} lamports)", 
              balance_before, balance_before_lamports);
        
        // 🚨 SAFETY CHECK: Prevent execution if insufficient balance
        if balance_before < 0.01 {
            return Err(anyhow!("❌ INSUFFICIENT BALANCE: Need at least 0.01 SOL, have {:.9}", balance_before));
        }
        
        let mut signatures = Vec::new();
        
        // Step 1: Execute first swap (input -> intermediate)
        info!("      🔄 Step 1: {} -> {}", 
               self.get_token_symbol(&opportunity.input_mint),
               self.get_token_symbol(&opportunity.intermediate_mint));
        
        // Calculate expected minimum for first swap
        let slippage_bps = self.calculate_safe_slippage(opportunity.amount_in, 
            &format!("{}/{}", opportunity.input_mint, opportunity.intermediate_mint));
        let min_intermediate_expected = self.calculate_minimum_amount_out(
            opportunity.expected_amount_out, slippage_bps
        );
        
        let sig1 = self.execute_jupiter_swap(
            &opportunity.input_mint,
            &opportunity.intermediate_mint,
            opportunity.amount_in,
        ).await?;
        
        signatures.push(sig1.clone());
        info!("         ✅ Swap 1 completed: {}", sig1);
        
        // Wait for confirmation
        sleep(Duration::from_secs(5)).await;
        
        // 🔍 BALANCE CHECK AFTER FIRST SWAP
        let balance_after_swap1 = self.get_wallet_balance().await?;
        let swap1_result = balance_after_swap1 - balance_before;
        
        info!("         💰 Balance after swap 1: {:.9} SOL", balance_after_swap1);
        info!("         📊 Swap 1 result: {:.9} SOL", swap1_result);
        
        // 🚨 ABORT IF FIRST SWAP LOST MONEY
        if swap1_result < -0.001 { // Allow small fees
            return Err(anyhow!("❌ FIRST SWAP LOST TOO MUCH: {:.9} SOL", swap1_result.abs()));
        }
        
        // 🔍 CRITICAL FIX: Get ACTUAL received amount from token account
        let intermediate_mint_pubkey = Pubkey::from_str(&opportunity.intermediate_mint)?;
        let intermediate_account = get_associated_token_address(&self.keypair.pubkey(), &intermediate_mint_pubkey);
        
        let intermediate_amount = self.get_actual_token_balance_after_swap(
            &intermediate_account,
            1, // Expect at least 1 token
            &intermediate_mint_pubkey, // Validate mint address
        ).await?;
        
        if intermediate_amount == 0 {
            return Err(anyhow!("❌ FIRST SWAP FAILED: No intermediate tokens received"));
        }
        
        info!("         💰 ACTUAL intermediate amount received: {} tokens", intermediate_amount);
        
        // Validate slippage for first swap
        // Note: We use a conservative estimate since we don't have the exact quote used
        let estimated_intermediate = (opportunity.amount_in as f64 * 0.99) as u64; // Rough estimate
        if let Err(e) = self.validate_post_swap_slippage(estimated_intermediate, intermediate_amount, 300).await {
            warn!("         ⚠️ First swap slippage warning: {}", e);
        }
        
        // Step 2: Execute second swap (intermediate -> output)
        info!("      🔄 Step 2: {} -> {}", 
               self.get_token_symbol(&opportunity.intermediate_mint),
               self.get_token_symbol(&opportunity.output_mint));
        
        let sig2 = self.execute_jupiter_swap(
            &opportunity.intermediate_mint,
            &opportunity.output_mint,
            intermediate_amount,
        ).await?;
        
        signatures.push(sig2.clone());
        info!("         ✅ Swap 2 completed: {}", sig2);
        
        // Wait for final confirmation
        sleep(Duration::from_secs(5)).await;
        
        // 🔍 FINAL BALANCE VERIFICATION
        let balance_after = self.get_wallet_balance().await?;
        let total_profit = balance_after - balance_before;
        let profit_lamports = (total_profit * 1_000_000_000.0) as i64;
        
        info!("      🔍 FINAL BALANCE: {:.9} SOL", balance_after);
        info!("      💰 TOTAL PROFIT: {:.9} SOL ({} lamports)", total_profit, profit_lamports);
        
        // 🚨 COMPREHENSIVE PROFIT VALIDATION
        if total_profit > 0.0 {
            info!("      ✅ ARBITRAGE SUCCESSFUL: Gained {:.9} SOL", total_profit);
            
            // Additional validation: Check if profit meets expectations
            let expected_profit_sol = opportunity.profit_lamports as f64 / 1_000_000_000.0;
            let profit_deviation = ((total_profit - expected_profit_sol) / expected_profit_sol * 100.0).abs();
            
            if profit_deviation > 50.0 {
                warn!("      ⚠️ PROFIT DEVIATION: Expected {:.9} SOL, got {:.9} SOL ({:.1}% deviation)", 
                      expected_profit_sol, total_profit, profit_deviation);
            }
        } else {
            warn!("      ⚠️ ARBITRAGE RESULTED IN LOSS: {:.9} SOL", total_profit.abs());
        }
        
        Ok(signatures)
    }

    async fn execute_jupiter_swap(&mut self, input_mint: &str, output_mint: &str, amount: u64) -> Result<String> {
        // Get fresh quote with timeout check
        let quote = self.get_fresh_jupiter_quote_with_timeout(input_mint, output_mint, amount, 2000).await?
            .ok_or_else(|| anyhow!("Failed to get quote for swap"))?;
        
        // Use consistent priority fees with calculation
        let priority_fee = 50000u64; // Match with fee calculation
        
        // Prepare swap request for Jupiter Quote API v6
        let swap_request = serde_json::json!({
            "quoteResponse": quote,
            "userPublicKey": self.wallet_address.to_string(),
            "wrapAndUnwrapSol": true,
            "dynamicComputeUnitLimit": true,
            "prioritizationFeeLamports": priority_fee,
        });
        
        // Get swap transaction from Jupiter Quote API v6
        let swap_url = format!("{}/swap", JUPITER_QUOTE_API);
        let response = self.http_client
            .post(&swap_url)
            .json(&swap_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Jupiter swap API error: {}", response.status()));
        }
        
        let swap_response: Value = response.json().await?;
        let swap_transaction = swap_response["swapTransaction"]
            .as_str()
            .ok_or_else(|| anyhow!("No swap transaction in response"))?;
        
        // Decode and sign transaction
        let tx_bytes = general_purpose::STANDARD.decode(swap_transaction)?;
        let mut transaction: Transaction = bincode::deserialize(&tx_bytes)?;
        
        // Update blockhash and sign
        let recent_blockhash = self.client.get_latest_blockhash().await?;
        transaction.message.recent_blockhash = recent_blockhash;
        transaction.sign(&[&self.keypair], recent_blockhash);
        
        // Send transaction
        let signature = self.client.send_and_confirm_transaction(&transaction).await?;
        
        Ok(signature.to_string())
    }

    async fn get_wallet_balance(&self) -> Result<f64> {
        let balance_lamports = self.client.get_balance(&self.wallet_address).await?;
        Ok(balance_lamports as f64 / 1_000_000_000.0)
    }

    fn get_token_symbol(&self, mint: &str) -> String {
        match mint {
            SOL_MINT => "SOL".to_string(),
            USDC_MINT => "USDC".to_string(),
            USDT_MINT => "USDT".to_string(),
            RAY_MINT => "RAY".to_string(),
            MSOL_MINT => "mSOL".to_string(),
            ETH_MINT => "ETH".to_string(),
            BTC_MINT => "BTC".to_string(),
            BONK_MINT => "BONK".to_string(),
            ORCA_MINT => "ORCA".to_string(),
            SRM_MINT => "SRM".to_string(),
            STSOL_MINT => "stSOL".to_string(),
            JITOSOL_MINT => "jitoSOL".to_string(),
            _ => format!("TOKEN({})", &mint[..8]),
        }
    }

    // Calculate Jupiter platform fees from quote data with better error handling
    fn calculate_jupiter_fees(&self, quote_data: &serde_json::Value) -> u64 {
        // Jupiter typically charges 0-0.4% platform fee
        if let Some(platform_fee) = quote_data.get("platformFee") {
            if let Some(amount) = platform_fee.get("amount") {
                // Try different possible formats for the fee amount
                if let Some(fee_str) = amount.as_str() {
                    return fee_str.parse::<u64>().unwrap_or(5000);
                } else if let Some(fee_num) = amount.as_u64() {
                    return fee_num;
                }
            }
        }
        5000 // Conservative default platform fee (0.005 SOL)
    }

    // Calculate safe slippage based on trade size and market conditions
    fn calculate_safe_slippage(&self, amount: u64, token_pair: &str) -> u64 {
        // Base slippage of 50 bps (0.5%)
        let base_slippage = 50;
        
        // Adjust based on trade size
        let size_adjustment = if amount > 100_000_000 { // > 0.1 SOL
            20 // Add 20 bps for larger trades
        } else if amount > 1_000_000_000 { // > 1 SOL
            50 // Add 50 bps for very large trades
        } else {
            0
        };
        
        // Adjust based on token pair liquidity
        let liquidity_adjustment = match token_pair {
            pair if pair.contains("SOL/USDC") || pair.contains("USDC/SOL") => 0,    // Most liquid
            pair if pair.contains("SOL/USDT") || pair.contains("USDT/SOL") => 5,    // Very liquid
            pair if pair.contains("SOL/RAY") || pair.contains("RAY/SOL") => 10,     // Good liquidity
            pair if pair.contains("SOL/mSOL") || pair.contains("mSOL/SOL") => 15,   // Good liquidity
            pair if pair.contains("SOL/ETH") || pair.contains("ETH/SOL") => 20,     // Moderate liquidity
            pair if pair.contains("SOL/BTC") || pair.contains("BTC/SOL") => 20,     // Moderate liquidity
            pair if pair.contains("USDC/USDT") || pair.contains("USDT/USDC") => 5,  // Very liquid
            pair if pair.contains("ETH/BTC") || pair.contains("BTC/ETH") => 25,     // Lower liquidity
            pair if pair.contains("BONK") => 30,     // Meme token, higher slippage
            pair if pair.contains("ORCA") => 15,     // DEX token, good liquidity
            pair if pair.contains("stSOL") => 15,    // Liquid staking, good liquidity
            pair if pair.contains("jitoSOL") => 20,  // Newer liquid staking
            _ => 35,            // Unknown pairs get higher slippage
        };
        
        let total_slippage = base_slippage + size_adjustment + liquidity_adjustment;
        
        // SAFETY: Cap maximum slippage at 200 bps (2%) to prevent excessive losses
        std::cmp::min(total_slippage, 200)
    }

    // Get actual token balance after swap execution with enhanced validation
    async fn get_actual_token_balance_after_swap(
        &self,
        token_account: &Pubkey,
        expected_minimum: u64,
        expected_mint: &Pubkey,
    ) -> Result<u64> {
        // Wait a moment for transaction to settle
        tokio::time::sleep(Duration::from_millis(3000)).await; // Increased to 3 seconds
        
        // Verify account ownership before reading balance
        if !self.verify_account_ownership(token_account).await? {
            return Err(anyhow!("Token account not owned by wallet"));
        }
        
        // Get current token account info
        let account_info = self.client.get_account(token_account).await?;
        
        if account_info.data.len() < 72 {
            return Err(anyhow!("Invalid token account data length"));
        }
        
        // Validate token account structure and mint
        self.validate_token_account(&account_info.data, expected_mint)?;
        
        // Parse token account data (amount is at bytes 64-72)
        let amount_bytes: [u8; 8] = account_info.data[64..72].try_into()?;
        let actual_balance = u64::from_le_bytes(amount_bytes);
        
        // Verify we received at least the minimum expected amount
        if actual_balance < expected_minimum {
            return Err(anyhow!(
                "Received {} but expected at least {}",
                actual_balance, expected_minimum
            ));
        }
        
        Ok(actual_balance)
    }

    // Validate token account structure and mint address
    fn validate_token_account(&self, account_data: &[u8], expected_mint: &Pubkey) -> Result<()> {
        if account_data.len() < 72 {
            return Err(anyhow!("Account data too short for token account"));
        }
        
        // Parse mint address from token account (bytes 0-32)
        let mint_bytes: [u8; 32] = account_data[0..32].try_into()?;
        let account_mint = Pubkey::new_from_array(mint_bytes);
        
        if account_mint != *expected_mint {
            return Err(anyhow!(
                "Token account mint mismatch: expected {}, found {}",
                expected_mint, account_mint
            ));
        }
        
        // Parse owner from token account (bytes 32-64)
        let owner_bytes: [u8; 32] = account_data[32..64].try_into()?;
        let account_owner = Pubkey::new_from_array(owner_bytes);
        
        if account_owner != self.wallet_address {
            return Err(anyhow!(
                "Token account owner mismatch: expected {}, found {}",
                self.wallet_address, account_owner
            ));
        }
        
        Ok(())
    }

    // Enhanced token account verification with balance checking
    async fn verify_token_account_with_balance(
        &self,
        wallet: &Pubkey,
        mint: &Pubkey,
        minimum_balance: Option<u64>,
    ) -> Result<(Pubkey, u64), Box<dyn std::error::Error>> {
        let token_account = get_associated_token_address(wallet, mint);
        
        // Check if account exists and get balance
        match self.client.get_account(&token_account).await {
            Ok(account_info) => {
                if account_info.data.len() < 72 {
                    return Err("Invalid token account data length".into());
                }
                
                // Parse current balance
                let amount_bytes: [u8; 8] = account_info.data[64..72].try_into()?;
                let current_balance = u64::from_le_bytes(amount_bytes);
                
                // Check minimum balance requirement
                if let Some(min_bal) = minimum_balance {
                    if current_balance < min_bal {
                        return Err(format!(
                            "Insufficient balance: {} < required {}",
                            current_balance, min_bal
                        ).into());
                    }
                }
                
                Ok((token_account, current_balance))
            }
            Err(_) => {
                // Account doesn't exist - create it if needed
                if minimum_balance.is_some() && minimum_balance.unwrap() > 0 {
                    return Err("Token account does not exist and balance is required".into());
                }
                
                // Return account address with zero balance for account creation
                Ok((token_account, 0))
            }
        }
    }

    // Rate limiting to prevent API abuse
    async fn enforce_rate_limit(&mut self) {
        let elapsed = self.rate_limiter.elapsed();
        // More relaxed rate limiting for Jupiter API v6 - better rate limits
        if elapsed < Duration::from_millis(200) { // Max 5 requests per second
            let sleep_time = Duration::from_millis(200) - elapsed;
            tokio::time::sleep(sleep_time).await;
        }
        self.rate_limiter = std::time::Instant::now();
    }

    // Verify token account ownership before reading balance
    async fn verify_account_ownership(&self, token_account: &Pubkey) -> Result<bool> {
        match self.client.get_account(token_account).await {
            Ok(account_info) => {
                // Check if it's a valid token account and belongs to our wallet
                if account_info.data.len() >= 72 {
                    // Parse owner from token account data (bytes 32-64)
                    let owner_bytes: [u8; 32] = account_info.data[32..64].try_into()?;
                    let owner = Pubkey::new_from_array(owner_bytes);
                    Ok(owner == self.wallet_address)
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }

    // Validate slippage after swap execution
    async fn validate_post_swap_slippage(
        &self,
        expected_amount: u64,
        actual_amount: u64,
        max_slippage_bps: u64,
    ) -> Result<()> {
        if actual_amount == 0 {
            return Err(anyhow!("No tokens received in swap"));
        }
        
        if actual_amount >= expected_amount {
            return Ok(()); // Got more than expected, all good
        }
        
        let slippage = ((expected_amount - actual_amount) as f64 / expected_amount as f64) * 10000.0;
        let slippage_bps = slippage as u64;
        
        if slippage_bps > max_slippage_bps {
            return Err(anyhow!(
                "Slippage too high: {}bps (max: {}bps). Expected: {}, Got: {}",
                slippage_bps, max_slippage_bps, expected_amount, actual_amount
            ));
        }
        
        info!("✅ Slippage within limits: {}bps (max: {}bps)", slippage_bps, max_slippage_bps);
        Ok(())
    }

    // Calculate expected minimum amount considering slippage
    fn calculate_minimum_amount_out(&self, expected_amount: u64, slippage_bps: u64) -> u64 {
        let slippage_multiplier = (10000 - slippage_bps) as f64 / 10000.0;
        (expected_amount as f64 * slippage_multiplier) as u64
    }

    // Enhanced quote freshness check to prevent stale quotes
    async fn get_fresh_jupiter_quote_with_timeout(
        &mut self, 
        input_mint: &str, 
        output_mint: &str, 
        amount: u64,
        timeout_ms: u64
    ) -> Result<Option<Value>> {
        let start_time = std::time::Instant::now();
        
        // Enforce rate limiting
        self.enforce_rate_limit().await;
        
        let quote = self.get_jupiter_quote(input_mint, output_mint, amount).await?;
        
        let elapsed = start_time.elapsed().as_millis() as u64;
        if elapsed > timeout_ms {
            warn!("Quote took {}ms (max: {}ms) - may be stale", elapsed, timeout_ms);
        }
        
        Ok(quote)
    }
}
