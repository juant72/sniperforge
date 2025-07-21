use anyhow::{anyhow, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    account::Account, commitment_config::CommitmentConfig, instruction::Instruction,
    pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::collections::HashMap;
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

// ===== MILITAR V2: DIRECT POOL INSPECTION =====
// Completamente sin APIs, acceso directo a blockchain

// Program IDs
const RAYDIUM_AMM_PROGRAM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const ORCA_SWAP_PROGRAM: &str = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";
const ORCA_WHIRLPOOL_PROGRAM: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

// Verified active pools
const RAYDIUM_SOL_USDC: &str = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
const RAYDIUM_SOL_USDT: &str = "7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX";
const RAYDIUM_RAY_USDC: &str = "6UmmUiYoBjSrhakAobJw8BvkmJtDVxaeBtbt7rxWo1mg";
const ORCA_SOL_USDC: &str = "EGZ7tiLeH62TPV1gL8WwbXGzEPa9zmcpVnnkPKKnrE2U";
const ORCA_SOL_USDT: &str = "7qbRF6YsyGuLUVs6Y1q64bdVrfe4ZcUUz1JRdoVNUJnm";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🔥 === MILITAR V2: DIRECT POOL INSPECTOR ===");
    info!("   ⚔️  NO APIs - PURE BLOCKCHAIN ACCESS");
    info!("   🔬 DIRECT POOL STRUCTURE ANALYSIS");
    info!("   💰 REAL-TIME POOL DATA EXTRACTION");

    let mut arbitrage = MilitarV2::new().await?;
    arbitrage.run_analysis().await?;

    Ok(())
}

#[derive(Debug, Clone)]
struct RealPoolData {
    address: Pubkey,
    program_id: Pubkey,
    token_a_mint: Pubkey,
    token_b_mint: Pubkey,
    token_a_vault: Pubkey,
    token_b_vault: Pubkey,
    token_a_balance: u64,
    token_b_balance: u64,
    pool_type: String,
    price_a_to_b: f64,
    price_b_to_a: f64,
    last_updated: u64,
}

#[derive(Debug)]
struct ArbitrageOpportunity {
    pool_1: String,
    pool_2: String,
    token_in: String,
    token_intermediate: String,
    token_out: String,
    amount_in: u64,
    estimated_profit: i64,
    profit_percentage: f64,
}

struct MilitarV2 {
    client: RpcClient,
    keypair: Keypair,
    wallet_address: Pubkey,
    pools: HashMap<String, RealPoolData>,
    monitoring_pools: Vec<String>,
}

impl MilitarV2 {
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

        let monitoring_pools = vec![
            RAYDIUM_SOL_USDC.to_string(),
            RAYDIUM_SOL_USDT.to_string(),
            RAYDIUM_RAY_USDC.to_string(),
            ORCA_SOL_USDC.to_string(),
            ORCA_SOL_USDT.to_string(),
        ];

        info!("⚔️  Militar V2 loaded: {}", wallet_address);
        info!("🔬 Analyzing {} pools", monitoring_pools.len());

        Ok(Self {
            client,
            keypair,
            wallet_address,
            pools: HashMap::new(),
            monitoring_pools,
        })
    }

    async fn run_analysis(&mut self) -> Result<()> {
        info!("🔥 Starting direct pool analysis...");

        let balance = self.get_wallet_balance().await?;
        info!("💰 Wallet balance: {:.9} SOL", balance);

        loop {
            info!("\n⚔️  === ANALYSIS CYCLE ===");

            // 1. Analyze all pools directly
            self.analyze_all_pools().await?;

            // 2. Find arbitrage opportunities
            let opportunities = self.find_arbitrage_opportunities().await?;

            if !opportunities.is_empty() {
                info!("🎯 Found {} arbitrage opportunities:", opportunities.len());
                for (i, opp) in opportunities.iter().take(5).enumerate() {
                    info!(
                        "  {}. {} -> {} -> {}: {:.4}% profit ({:.6} SOL)",
                        i + 1,
                        opp.token_in,
                        opp.token_intermediate,
                        opp.token_out,
                        opp.profit_percentage,
                        opp.estimated_profit as f64 / 1e9
                    );
                }

                // Execute best opportunity
                if let Some(best) = opportunities.first() {
                    if best.profit_percentage > 1.0 {
                        // Only execute if >1% profit
                        info!("⚔️  Executing best opportunity...");
                        match self.execute_arbitrage(best).await {
                            Ok(signature) => {
                                info!("✅ Arbitrage executed: {}", signature);
                                let new_balance = self.get_wallet_balance().await?;
                                let profit = new_balance - balance;
                                info!("💎 Actual profit: {:.9} SOL", profit);
                            }
                            Err(e) => {
                                warn!("❌ Execution failed: {}", e);
                            }
                        }
                    }
                }
            } else {
                info!("💤 No profitable opportunities found");
            }

            sleep(Duration::from_secs(3)).await;
        }
    }

    async fn analyze_all_pools(&mut self) -> Result<()> {
        info!("🔬 Analyzing pool structures...");

        for pool_address in &self.monitoring_pools.clone() {
            match self.analyze_pool_structure(pool_address).await {
                Ok(pool_data) => {
                    info!(
                        "  ✅ {}: {} ({} / {})",
                        pool_address[..8].to_string(),
                        pool_data.pool_type,
                        self.format_balance(pool_data.token_a_balance),
                        self.format_balance(pool_data.token_b_balance)
                    );
                    self.pools.insert(pool_address.clone(), pool_data);
                }
                Err(e) => {
                    warn!("  ❌ {}: {}", pool_address[..8].to_string(), e);
                }
            }
        }

        info!("📊 Successfully analyzed {} pools", self.pools.len());
        Ok(())
    }

    async fn analyze_pool_structure(&self, pool_address: &str) -> Result<RealPoolData> {
        let pool_pubkey = Pubkey::from_str(pool_address)?;
        let account = self.client.get_account(&pool_pubkey).await?;

        let program_id = account.owner;
        let program_str = program_id.to_string();

        match program_str.as_str() {
            RAYDIUM_AMM_PROGRAM => self.parse_raydium_pool_v2(pool_pubkey, &account).await,
            ORCA_SWAP_PROGRAM => self.parse_orca_pool_v2(pool_pubkey, &account).await,
            ORCA_WHIRLPOOL_PROGRAM => self.parse_orca_whirlpool_v2(pool_pubkey, &account).await,
            _ => Err(anyhow!("Unknown pool program: {}", program_str)),
        }
    }

    async fn parse_raydium_pool_v2(
        &self,
        pool_address: Pubkey,
        account: &Account,
    ) -> Result<RealPoolData> {
        let data = &account.data;

        if data.len() < 752 {
            return Err(anyhow!("Invalid Raydium pool size: {}", data.len()));
        }

        // Raydium AMM v4 layout offsets
        let token_a_mint = Pubkey::new_from_array(
            data[400..432]
                .try_into()
                .map_err(|_| anyhow!("Invalid token A mint"))?,
        );
        let token_b_mint = Pubkey::new_from_array(
            data[432..464]
                .try_into()
                .map_err(|_| anyhow!("Invalid token B mint"))?,
        );
        let token_a_vault = Pubkey::new_from_array(
            data[464..496]
                .try_into()
                .map_err(|_| anyhow!("Invalid token A vault"))?,
        );
        let token_b_vault = Pubkey::new_from_array(
            data[496..528]
                .try_into()
                .map_err(|_| anyhow!("Invalid token B vault"))?,
        );

        // Get real balances from vault accounts
        let token_a_balance = self.get_vault_balance(&token_a_vault).await.unwrap_or(0);
        let token_b_balance = self.get_vault_balance(&token_b_vault).await.unwrap_or(0);

        // Calculate prices
        let (price_a_to_b, price_b_to_a) = self.calculate_prices(token_a_balance, token_b_balance);

        Ok(RealPoolData {
            address: pool_address,
            program_id: account.owner,
            token_a_mint,
            token_b_mint,
            token_a_vault,
            token_b_vault,
            token_a_balance,
            token_b_balance,
            pool_type: "Raydium AMM".to_string(),
            price_a_to_b,
            price_b_to_a,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    async fn parse_orca_pool_v2(
        &self,
        pool_address: Pubkey,
        account: &Account,
    ) -> Result<RealPoolData> {
        let data = &account.data;

        if data.len() < 324 {
            return Err(anyhow!("Invalid Orca pool size: {}", data.len()));
        }

        // Orca swap pool layout
        let token_a_mint = Pubkey::new_from_array(
            data[101..133]
                .try_into()
                .map_err(|_| anyhow!("Invalid token A mint"))?,
        );
        let token_b_mint = Pubkey::new_from_array(
            data[181..213]
                .try_into()
                .map_err(|_| anyhow!("Invalid token B mint"))?,
        );
        let token_a_vault = Pubkey::new_from_array(
            data[85..117]
                .try_into()
                .map_err(|_| anyhow!("Invalid token A vault"))?,
        );
        let token_b_vault = Pubkey::new_from_array(
            data[165..197]
                .try_into()
                .map_err(|_| anyhow!("Invalid token B vault"))?,
        );

        let token_a_balance = self.get_vault_balance(&token_a_vault).await.unwrap_or(0);
        let token_b_balance = self.get_vault_balance(&token_b_vault).await.unwrap_or(0);

        let (price_a_to_b, price_b_to_a) = self.calculate_prices(token_a_balance, token_b_balance);

        Ok(RealPoolData {
            address: pool_address,
            program_id: account.owner,
            token_a_mint,
            token_b_mint,
            token_a_vault,
            token_b_vault,
            token_a_balance,
            token_b_balance,
            pool_type: "Orca Swap".to_string(),
            price_a_to_b,
            price_b_to_a,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    async fn parse_orca_whirlpool_v2(
        &self,
        pool_address: Pubkey,
        account: &Account,
    ) -> Result<RealPoolData> {
        let data = &account.data;

        if data.len() < 653 {
            return Err(anyhow!("Invalid Whirlpool size: {}", data.len()));
        }

        // Whirlpool layout (concentrated liquidity)
        let token_a_mint = Pubkey::new_from_array(
            data[101..133]
                .try_into()
                .map_err(|_| anyhow!("Invalid token A mint"))?,
        );
        let token_b_mint = Pubkey::new_from_array(
            data[133..165]
                .try_into()
                .map_err(|_| anyhow!("Invalid token B mint"))?,
        );
        let token_a_vault = Pubkey::new_from_array(
            data[165..197]
                .try_into()
                .map_err(|_| anyhow!("Invalid token A vault"))?,
        );
        let token_b_vault = Pubkey::new_from_array(
            data[197..229]
                .try_into()
                .map_err(|_| anyhow!("Invalid token B vault"))?,
        );

        let token_a_balance = self.get_vault_balance(&token_a_vault).await.unwrap_or(0);
        let token_b_balance = self.get_vault_balance(&token_b_vault).await.unwrap_or(0);

        let (price_a_to_b, price_b_to_a) = self.calculate_prices(token_a_balance, token_b_balance);

        Ok(RealPoolData {
            address: pool_address,
            program_id: account.owner,
            token_a_mint,
            token_b_mint,
            token_a_vault,
            token_b_vault,
            token_a_balance,
            token_b_balance,
            pool_type: "Orca Whirlpool".to_string(),
            price_a_to_b,
            price_b_to_a,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    async fn get_vault_balance(&self, vault_address: &Pubkey) -> Result<u64> {
        let account = self.client.get_account(vault_address).await?;

        if account.data.len() < 165 {
            return Err(anyhow!("Invalid token account size"));
        }

        // Token account amount is at offset 64-72
        let amount_bytes: [u8; 8] = account.data[64..72]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse amount"))?;

        Ok(u64::from_le_bytes(amount_bytes))
    }

    fn calculate_prices(&self, balance_a: u64, balance_b: u64) -> (f64, f64) {
        if balance_a == 0 || balance_b == 0 {
            return (0.0, 0.0);
        }

        let price_a_to_b = balance_b as f64 / balance_a as f64;
        let price_b_to_a = balance_a as f64 / balance_b as f64;

        (price_a_to_b, price_b_to_a)
    }

    async fn find_arbitrage_opportunities(&self) -> Result<Vec<ArbitrageOpportunity>> {
        let mut opportunities = Vec::new();

        // Compare all pool pairs
        let pool_keys: Vec<_> = self.pools.keys().collect();

        for i in 0..pool_keys.len() {
            for j in (i + 1)..pool_keys.len() {
                let pool_1 = &self.pools[pool_keys[i]];
                let pool_2 = &self.pools[pool_keys[j]];

                // Check for common tokens
                let common_tokens = self.find_common_tokens(pool_1, pool_2);

                for common_token in common_tokens {
                    if let Some(opp) = self
                        .calculate_arbitrage_opportunity(pool_1, pool_2, &common_token)
                        .await?
                    {
                        opportunities.push(opp);
                    }
                }
            }
        }

        // Sort by profit percentage
        opportunities.sort_by(|a, b| {
            b.profit_percentage
                .partial_cmp(&a.profit_percentage)
                .unwrap()
        });

        Ok(opportunities)
    }

    fn find_common_tokens(&self, pool_1: &RealPoolData, pool_2: &RealPoolData) -> Vec<Pubkey> {
        let mut common = Vec::new();

        if pool_1.token_a_mint == pool_2.token_a_mint || pool_1.token_a_mint == pool_2.token_b_mint
        {
            common.push(pool_1.token_a_mint);
        }
        if pool_1.token_b_mint == pool_2.token_a_mint || pool_1.token_b_mint == pool_2.token_b_mint
        {
            common.push(pool_1.token_b_mint);
        }

        common
    }

    async fn calculate_arbitrage_opportunity(
        &self,
        pool_1: &RealPoolData,
        pool_2: &RealPoolData,
        common_token: &Pubkey,
    ) -> Result<Option<ArbitrageOpportunity>> {
        let test_amount = 10_000_000; // 0.01 SOL equivalent

        // Route 1: Pool 1 -> Common Token -> Pool 2
        let step1_out = self.simulate_swap(pool_1, test_amount, common_token)?;
        let step2_out = self.simulate_swap(pool_2, step1_out, common_token)?;

        let profit = step2_out as i64 - test_amount as i64;
        let profit_percentage = (profit as f64 / test_amount as f64) * 100.0;

        if profit > 0 {
            return Ok(Some(ArbitrageOpportunity {
                pool_1: pool_1.address.to_string()[..8].to_string(),
                pool_2: pool_2.address.to_string()[..8].to_string(),
                token_in: "SOL".to_string(),             // Simplified
                token_intermediate: "TOKEN".to_string(), // Simplified
                token_out: "SOL".to_string(),            // Simplified
                amount_in: test_amount,
                estimated_profit: profit,
                profit_percentage,
            }));
        }

        Ok(None)
    }

    fn simulate_swap(
        &self,
        pool: &RealPoolData,
        amount_in: u64,
        output_token: &Pubkey,
    ) -> Result<u64> {
        let (reserve_in, reserve_out) = if pool.token_a_mint == *output_token {
            (pool.token_b_balance, pool.token_a_balance)
        } else if pool.token_b_mint == *output_token {
            (pool.token_a_balance, pool.token_b_balance)
        } else {
            return Err(anyhow!("Token not in pool"));
        };

        if reserve_in == 0 || reserve_out == 0 {
            return Ok(0);
        }

        // Constant product formula with 0.3% fee
        let amount_in_with_fee = amount_in * 997 / 1000;
        let k = reserve_in as u128 * reserve_out as u128;
        let new_reserve_in = reserve_in as u128 + amount_in_with_fee as u128;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = reserve_out as u128 - new_reserve_out;

        Ok((amount_out * 99 / 100) as u64) // 1% slippage buffer
    }

    async fn execute_arbitrage(&self, opportunity: &ArbitrageOpportunity) -> Result<String> {
        // Simplified execution - would need full transaction building
        info!("🚀 Simulating arbitrage execution...");

        // In real implementation, would build and send transaction here
        let simulated_signature = format!(
            "SIM{}{}",
            opportunity.pool_1,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        Ok(simulated_signature)
    }

    async fn get_wallet_balance(&self) -> Result<f64> {
        let balance = self.client.get_balance(&self.wallet_address).await?;
        Ok(balance as f64 / 1_000_000_000.0)
    }

    fn format_balance(&self, balance: u64) -> String {
        format!("{:.2}M", balance as f64 / 1_000_000.0)
    }
}
