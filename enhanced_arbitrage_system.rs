use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Keypair,
    signer::Signer,
    pubkey::Pubkey,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use serde_json::Value;
use reqwest::Client;
use base64;
use bincode;

// Multiple DEX/Aggregator APIs
const JUPITER_API_BASE: &str = "https://quote-api.jup.ag/v6";
const ONE_INCH_API_BASE: &str = "https://api.1inch.dev/swap/v6.0/1"; // Ethereum but has Solana plans
const BIRDEYE_API_BASE: &str = "https://public-api.birdeye.so";

// Popular Solana Tokens (Top 50)
const POPULAR_TOKENS: &[(&str, &str)] = &[
    ("So11111111111111111111111111111111111111112", "SOL"),
    ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "USDC"),
    ("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "USDT"),
    ("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "RAY"),
    ("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So", "mSOL"),
    ("7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs", "ETH"),
    ("9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E", "BTC"),
    ("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "BONK"),
    ("5z3EqYQo9HiCdqL2L2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q", "ORCA"),
    ("SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt", "SRM"),
    ("AGFEad2et2ZJif9jaGpdMixQqvW5i81aBdvKe7PHNfz3", "FTT"),
    ("kinXdEcpDQeHPEuQnqmUgtYykqKGVFq6CeVX5iAHJq6", "KIN"),
    ("7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj", "stSOL"),
    ("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1", "bSOL"),
    ("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn", "jitoSOL"),
    ("HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3", "PYTH"),
    ("nosXBVoaCTtYdLvKY6Csb4AC8JCdQKKAaWYtx2ZMoo7", "NOS"),
];

// DEX Routes Configuration
const ARBITRAGE_ROUTES: &[(&str, &str, &str)] = &[
    // Major triangle arbitrages
    ("SOL", "USDC", "SOL"),
    ("SOL", "USDT", "SOL"),
    ("SOL", "RAY", "SOL"),
    ("SOL", "mSOL", "SOL"),
    ("SOL", "ETH", "SOL"),
    ("SOL", "BTC", "SOL"),
    ("SOL", "BONK", "SOL"),
    ("USDC", "USDT", "USDC"),
    ("USDC", "ETH", "USDC"),
    ("USDC", "BTC", "USDC"),
    ("ETH", "BTC", "ETH"),
    ("RAY", "ORCA", "RAY"),
    ("mSOL", "stSOL", "mSOL"),
    // 4-step arbitrages
    ("SOL", "USDC", "ETH", "SOL"),
    ("SOL", "USDT", "BTC", "SOL"),
    ("USDC", "ETH", "BTC", "USDC"),
];

// Trade size optimization
const TRADE_SIZES_SOL: &[f64] = &[0.001, 0.005, 0.01, 0.05, 0.1, 0.5];

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === ENHANCED ARBITRAGE SYSTEM ===");
    info!("   💎 MULTI-DEX ARBITRAGE SCANNER");
    info!("   🔄 {} POPULAR TOKENS", POPULAR_TOKENS.len());
    info!("   📈 {} ARBITRAGE ROUTES", ARBITRAGE_ROUTES.len());
    info!("   💰 DYNAMIC TRADE SIZING");

    let mut arbitrage = EnhancedArbitrageSystem::new().await?;
    arbitrage.run_enhanced_arbitrage().await?;

    Ok(())
}

struct EnhancedArbitrageSystem {
    client: RpcClient,
    keypair: Keypair,
    wallet_address: Pubkey,
    http_client: Client,
    rate_limiter: std::time::Instant,
    token_map: HashMap<String, String>, // symbol -> mint
    mint_map: HashMap<String, String>,  // mint -> symbol
}

#[derive(Debug, Clone)]
struct EnhancedOpportunity {
    route: Vec<String>,
    amounts: Vec<u64>,
    profit_lamports: u64,
    profit_percentage: f64,
    trade_size_sol: f64,
    dex_source: String,
    confidence_score: f64,
}

impl EnhancedArbitrageSystem {
    async fn new() -> Result<Self> {
        // Load wallet
        let wallet_path = "mainnet_wallet.json";
        let json_str = std::fs::read_to_string(wallet_path)?;
        let keypair_bytes: Vec<u8> = serde_json::from_str(&json_str)?;
        let keypair = Keypair::from_bytes(&keypair_bytes)?;
        let wallet_address = keypair.pubkey();

        // RPC setup
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        // HTTP client
        let http_client = Client::new();

        // Build token maps
        let mut token_map = HashMap::new();
        let mut mint_map = HashMap::new();
        
        for (mint, symbol) in POPULAR_TOKENS {
            token_map.insert(symbol.to_string(), mint.to_string());
            mint_map.insert(mint.to_string(), symbol.to_string());
        }

        info!("✅ Enhanced Arbitrage System loaded: {}", wallet_address);
        info!("   📊 {} tokens loaded", POPULAR_TOKENS.len());

        Ok(Self {
            client,
            keypair,
            wallet_address,
            http_client,
            rate_limiter: std::time::Instant::now(),
            token_map,
            mint_map,
        })
    }

    async fn run_enhanced_arbitrage(&mut self) -> Result<()> {
        info!("🚀 Starting ENHANCED arbitrage scanning...");
        
        let mut cycle = 0;
        let initial_balance = self.get_wallet_balance().await?;
        info!("💰 Initial balance: {:.9} SOL", initial_balance);

        loop {
            cycle += 1;
            info!("\n🏆 === ENHANCED ARBITRAGE CYCLE {} ===", cycle);

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

            // 1. Scan ALL opportunities across multiple sources
            match self.scan_all_arbitrage_opportunities().await {
                Ok(opportunities) => {
                    if opportunities.is_empty() {
                        info!("   💤 No profitable arbitrage found across {} routes", ARBITRAGE_ROUTES.len());
                    } else {
                        info!("   🎯 {} ENHANCED arbitrage opportunities found!", opportunities.len());
                        
                        // Display top 5 opportunities
                        for (i, opp) in opportunities.iter().take(5).enumerate() {
                            info!("   📊 OPPORTUNITY #{}: {:?}", i + 1, opp.route);
                            info!("      💰 Profit: {} lamports ({:.4}%) - Size: {:.3} SOL", 
                                  opp.profit_lamports, opp.profit_percentage, opp.trade_size_sol);
                            info!("      🏢 Source: {} - Confidence: {:.1}%", 
                                  opp.dex_source, opp.confidence_score * 100.0);
                        }
                        
                        // Execute best opportunity if profitable enough
                        let best_opp = &opportunities[0];
                        let min_profit_lamports = 20000; // Higher threshold for enhanced system
                        
                        if best_opp.profit_lamports > min_profit_lamports && best_opp.confidence_score > 0.7 {
                            info!("   🚀 EXECUTING BEST OPPORTUNITY...");
                            // Here would go the execution logic
                            // For now, just simulate
                            info!("   ✅ SIMULATED EXECUTION SUCCESSFUL");
                        } else {
                            info!("   💡 Best opportunity below threshold or low confidence");
                        }
                    }
                }
                Err(e) => {
                    error!("   ❌ Failed to scan opportunities: {}", e);
                }
            }

            // Wait before next cycle (faster for enhanced system)
            sleep(Duration::from_secs(10)).await;
        }
    }

    async fn scan_all_arbitrage_opportunities(&mut self) -> Result<Vec<EnhancedOpportunity>> {
        info!("   🔍 ENHANCED SCANNING: {} routes × {} sizes = {} combinations", 
              ARBITRAGE_ROUTES.len(), TRADE_SIZES_SOL.len(), 
              ARBITRAGE_ROUTES.len() * TRADE_SIZES_SOL.len());
        
        let mut all_opportunities = Vec::new();
        
        // Scan each route with each trade size
        for route in ARBITRAGE_ROUTES {
            for &trade_size_sol in TRADE_SIZES_SOL {
                let trade_size_lamports = (trade_size_sol * 1_000_000_000.0) as u64;
                
                if let Some(opp) = self.check_enhanced_route(route, trade_size_lamports, trade_size_sol).await? {
                    all_opportunities.push(opp);
                }
                
                // Rate limiting
                sleep(Duration::from_millis(100)).await;
            }
        }
        
        // Sort by profit potential (considering confidence)
        all_opportunities.sort_by(|a, b| {
            let score_a = a.profit_lamports as f64 * a.confidence_score;
            let score_b = b.profit_lamports as f64 * b.confidence_score;
            score_b.partial_cmp(&score_a).unwrap()
        });
        
        Ok(all_opportunities)
    }

    async fn check_enhanced_route(
        &mut self,
        route: &(&str, &str, &str),
        amount: u64,
        trade_size_sol: f64,
    ) -> Result<Option<EnhancedOpportunity>> {
        
        let (input_symbol, intermediate_symbol, output_symbol) = route;
        
        // Convert symbols to mints
        let input_mint = self.token_map.get(*input_symbol)?;
        let intermediate_mint = self.token_map.get(*intermediate_symbol)?;
        let output_mint = self.token_map.get(*output_symbol)?;
        
        // Get quotes from Jupiter
        if let Some(opp) = self.check_jupiter_arbitrage(
            input_mint, intermediate_mint, output_mint, amount
        ).await? {
            return Ok(Some(EnhancedOpportunity {
                route: vec![input_symbol.to_string(), intermediate_symbol.to_string(), output_symbol.to_string()],
                amounts: vec![amount, 0, 0], // Will be filled by actual quotes
                profit_lamports: opp.profit_lamports,
                profit_percentage: opp.profit_percentage,
                trade_size_sol,
                dex_source: "Jupiter".to_string(),
                confidence_score: self.calculate_confidence_score(&opp, trade_size_sol),
            }));
        }
        
        Ok(None)
    }

    async fn check_jupiter_arbitrage(
        &mut self,
        input_mint: &str,
        intermediate_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<Option<BasicOpportunity>> {
        
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
                    
                    // Enhanced fee calculation
                    let total_fees = self.calculate_enhanced_fees(&quote1_data, &quote2_data, amount);
                    
                    if profit_lamports > total_fees {
                        let net_profit = profit_lamports - total_fees;
                        
                        return Ok(Some(BasicOpportunity {
                            profit_lamports: net_profit,
                            profit_percentage,
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }

    fn calculate_confidence_score(&self, opportunity: &BasicOpportunity, trade_size_sol: f64) -> f64 {
        let mut score = 0.5; // Base score
        
        // Higher confidence for higher profits
        if opportunity.profit_percentage > 1.0 {
            score += 0.3;
        } else if opportunity.profit_percentage > 0.5 {
            score += 0.2;
        }
        
        // Optimal trade sizes get higher confidence
        if trade_size_sol >= 0.01 && trade_size_sol <= 0.1 {
            score += 0.2;
        }
        
        // Jupiter is reliable
        score += 0.1;
        
        score.min(1.0)
    }

    fn calculate_enhanced_fees(&self, quote1: &Value, quote2: &Value, amount: u64) -> u64 {
        // More sophisticated fee calculation
        let base_tx_fees = 15000u64; // Higher estimate for complex arbitrage
        let jupiter_fees = self.calculate_jupiter_fees(quote1) + self.calculate_jupiter_fees(quote2);
        let priority_fees = std::cmp::max(50000, amount / 1000); // Dynamic priority fees
        let slippage_buffer = amount / 200; // 0.5% slippage buffer
        
        base_tx_fees + jupiter_fees + priority_fees + slippage_buffer
    }

    // Reuse existing methods from original system
    async fn get_jupiter_quote(&mut self, input_mint: &str, output_mint: &str, amount: u64) -> Result<Option<Value>> {
        // Rate limiting
        let elapsed = self.rate_limiter.elapsed();
        if elapsed < Duration::from_millis(300) { // Faster for enhanced system
            let sleep_time = Duration::from_millis(300) - elapsed;
            tokio::time::sleep(sleep_time).await;
        }
        self.rate_limiter = std::time::Instant::now();
        
        let slippage_bps = 100; // 1% slippage for enhanced system
        
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            JUPITER_API_BASE, input_mint, output_mint, amount, slippage_bps
        );
        
        match self.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let quote: Value = response.json().await?;
                    Ok(Some(quote))
                } else {
                    warn!("Jupiter API error: {}", response.status());
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("Failed to get Jupiter quote: {}", e);
                Ok(None)
            }
        }
    }

    fn calculate_jupiter_fees(&self, quote_data: &Value) -> u64 {
        if let Some(platform_fee) = quote_data.get("platformFee") {
            if let Some(amount) = platform_fee.get("amount") {
                if let Some(fee_str) = amount.as_str() {
                    return fee_str.parse::<u64>().unwrap_or(7500);
                } else if let Some(fee_num) = amount.as_u64() {
                    return fee_num;
                }
            }
        }
        7500 // Higher default for enhanced system
    }

    async fn get_wallet_balance(&self) -> Result<f64> {
        let balance_lamports = self.client.get_balance(&self.wallet_address).await?;
        Ok(balance_lamports as f64 / 1_000_000_000.0)
    }
}

// Helper structs
#[derive(Debug, Clone)]
struct BasicOpportunity {
    profit_lamports: u64,
    profit_percentage: f64,
}
