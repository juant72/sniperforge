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
    program_pack::Pack,
    account::Account,
};
use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};
use spl_token::{instruction as token_instruction, state::Account as TokenAccount};
use bincode;

// ===== MILITARY-GRADE DIRECT POOL ACCESS =====
// No APIs, no rate limits, pure blockchain access

// Raydium Program IDs
const RAYDIUM_AMM_PROGRAM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const RAYDIUM_AUTHORITY: &str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";

// Orca Program IDs  
const ORCA_WHIRLPOOL_PROGRAM: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
const ORCA_SWAP_PROGRAM: &str = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";

// Serum/OpenBook Program IDs
const SERUM_DEX_PROGRAM: &str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";

// Token Program
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

// Popular Solana Tokens with decimals info
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

// Known Pool Addresses - Updated with real mainnet pools
const RAYDIUM_SOL_USDC: &str = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
const RAYDIUM_SOL_USDT: &str = "7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX";
const RAYDIUM_RAY_SOL: &str = "AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA";
const RAYDIUM_RAY_USDC: &str = "6UmmUiYoBjSrhakAobJw8BvkmJtDVxaeBtbt7rxWo1mg";

const ORCA_SOL_USDC: &str = "HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ";
const ORCA_SOL_USDT: &str = "4fuUiYxTQ6QCrdSq9ouBYcTM7bqSwYTSyLueGZLTy4T4";
const ORCA_ETH_SOL: &str = "71FymgN2ZUf7VvVTLE8jYEnjP3jSK1Frp2XT1nHs8Hob";
const ORCA_BTC_SOL: &str = "2AuTTzpTiLntFe4dRMnwoo5cZMr8nxvkEbFqjmMkwQzP";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🔥 === MILITARY-GRADE DIRECT ARBITRAGE SYSTEM ===");
    info!("   ⚔️  DIRECT POOL ACCESS - NO API LIMITS");
    info!("   🎯 BLOCKCHAIN-NATIVE CALCULATIONS");
    info!("   💀 MANUAL TRANSACTION CONSTRUCTION");
    info!("   ⚡ MILITARY PRECISION & SPEED");
    info!("   🔬 DIRECT RAYDIUM/ORCA POOL MONITORING");
    info!("   💰 MAXIMUM PROFIT EXTRACTION");

    let mut arbitrage = MilitaryArbitrageSystem::new().await?;
    arbitrage.run_direct_arbitrage().await?;

    Ok(())
}

// Military-grade pool data structure
#[derive(Debug, Clone)]
struct PoolData {
    address: Pubkey,
    token_a_mint: Pubkey,
    token_b_mint: Pubkey,
    token_a_vault: Pubkey,
    token_b_vault: Pubkey,
    token_a_amount: u64,
    token_b_amount: u64,
    lp_mint: Pubkey,
    lp_supply: u64,
    pool_type: PoolType,
    last_updated: u64,
    fees_bps: u64,
}

#[derive(Debug, Clone)]
enum PoolType {
    Raydium,
    Orca,
    OrcaWhirlpool,
    Serum,
}

#[derive(Debug, Clone)]
struct DirectOpportunity {
    pool_a: PoolData,
    pool_b: PoolData,
    intermediate_token: Pubkey,
    amount_in: u64,
    estimated_amount_out: u64,
    profit_lamports: i64,
    profit_percentage: f64,
    execution_path: Vec<SwapInstruction>,
}

#[derive(Debug, Clone)]
struct SwapInstruction {
    program_id: Pubkey,
    pool_address: Pubkey,
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount_in: u64,
    minimum_amount_out: u64,
    instruction_data: Vec<u8>,
}

struct MilitaryArbitrageSystem {
    client: RpcClient,
    keypair: Keypair,
    wallet_address: Pubkey,
    pools: HashMap<String, PoolData>,
    monitoring_pools: Vec<String>,
    last_pool_update: std::time::Instant,
}

impl MilitaryArbitrageSystem {
    async fn new() -> Result<Self> {
        // Load wallet
        let wallet_path = "mainnet_wallet.json";
        let json_str = std::fs::read_to_string(wallet_path)?;
        let keypair_bytes: Vec<u8> = serde_json::from_str(&json_str)?;
        let keypair = Keypair::from_bytes(&keypair_bytes)?;
        let wallet_address = keypair.pubkey();

        // RPC setup - use fastest endpoint for military operations
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::finalized());

        // Initialize monitoring pools - major liquidity pools only
        let monitoring_pools = vec![
            RAYDIUM_SOL_USDC.to_string(),
            RAYDIUM_SOL_USDT.to_string(),
            RAYDIUM_RAY_SOL.to_string(),
            RAYDIUM_RAY_USDC.to_string(),
            ORCA_SOL_USDC.to_string(),
            ORCA_SOL_USDT.to_string(),
            ORCA_ETH_SOL.to_string(),
            ORCA_BTC_SOL.to_string(),
        ];

        info!("⚔️  Military Arbitrage System loaded: {}", wallet_address);
        info!("🔬 Monitoring {} critical pools", monitoring_pools.len());

        Ok(Self {
            client,
            keypair,
            wallet_address,
            pools: HashMap::new(),
            monitoring_pools,
            last_pool_update: std::time::Instant::now(),
        })
    }

    async fn run_direct_arbitrage(&mut self) -> Result<()> {
        info!("🔥 Starting military-grade direct arbitrage execution...");
        
        let initial_balance = self.get_wallet_balance().await?;
        info!("💰 Initial balance: {:.9} SOL", initial_balance);
        
        let mut cycle = 1;
        let mut total_profit = 0.0;

        loop {
            info!("\n⚔️  === MILITARY ARBITRAGE CYCLE {} ===", cycle);
            
            let balance_before = self.get_wallet_balance().await?;
            info!("   💰 Current balance: {:.9} SOL", balance_before);
            info!("   📈 Total profit: {:.9} SOL", total_profit);
            
            // 1. Update pool data directly from blockchain
            self.update_all_pools().await?;
            
            // 2. Scan for arbitrage opportunities using direct calculations
            info!("   🔍 Scanning pools for direct arbitrage opportunities...");
            let opportunities = self.find_direct_arbitrage_opportunities().await?;
            
            if opportunities.is_empty() {
                info!("   💤 No profitable opportunities found - waiting...");
                sleep(Duration::from_secs(5)).await;
                continue;
            }

            // 3. Execute the most profitable opportunity
            let best_opportunity = &opportunities[0];
            info!("   🎯 Executing opportunity: {:.6}% profit", best_opportunity.profit_percentage);
            
            match self.execute_direct_arbitrage(best_opportunity).await {
                Ok(signature) => {
                    info!("   ✅ Arbitrage executed: {}", signature);
                    
                    let balance_after = self.get_wallet_balance().await?;
                    let cycle_profit = balance_after - balance_before;
                    total_profit += cycle_profit;
                    
                    info!("   💎 Cycle profit: {:.9} SOL", cycle_profit);
                    info!("   📊 New balance: {:.9} SOL", balance_after);
                }
                Err(e) => {
                    warn!("   ❌ Arbitrage execution failed: {}", e);
                }
            }
            
            cycle += 1;
            sleep(Duration::from_millis(500)).await; // Military speed
        }
    }

    // ===== DIRECT POOL DATA ACCESS =====
    
    async fn update_all_pools(&mut self) -> Result<()> {
        let now = std::time::Instant::now();
        
        // Only update if more than 2 seconds have passed (military efficiency)
        if now.duration_since(self.last_pool_update) < Duration::from_secs(2) {
            return Ok(());
        }
        
        info!("   🔬 Updating pool data from blockchain...");
        
        for pool_address in &self.monitoring_pools.clone() {
            match self.read_pool_data_direct(pool_address).await {
                Ok(pool_data) => {
                    self.pools.insert(pool_address.clone(), pool_data);
                }
                Err(e) => {
                    warn!("Failed to read pool {}: {}", pool_address, e);
                }
            }
        }
        
        self.last_pool_update = now;
        info!("   ✅ Updated {} pools", self.pools.len());
        Ok(())
    }

    async fn read_pool_data_direct(&self, pool_address: &str) -> Result<PoolData> {
        let pool_pubkey = Pubkey::from_str(pool_address)?;
        let account = self.client.get_account(&pool_pubkey).await?;
        
        // Determine pool type and parse accordingly
        if account.owner.to_string() == RAYDIUM_AMM_PROGRAM {
            self.parse_raydium_pool(pool_pubkey, &account).await
        } else if account.owner.to_string() == ORCA_SWAP_PROGRAM {
            self.parse_orca_pool(pool_pubkey, &account).await
        } else if account.owner.to_string() == ORCA_WHIRLPOOL_PROGRAM {
            self.parse_orca_whirlpool(pool_pubkey, &account).await
        } else {
            Err(anyhow!("Unknown pool program: {}", account.owner))
        }
    }

    async fn parse_raydium_pool(&self, pool_address: Pubkey, account: &Account) -> Result<PoolData> {
        // Raydium AMM pool structure parsing
        let data = &account.data;
        
        if data.len() < 752 {
            return Err(anyhow!("Invalid Raydium pool data length"));
        }
        
        // Parse Raydium pool data structure (offsets from Raydium SDK)
        let token_a_mint = Pubkey::new_from_array(
            data[400..432].try_into().map_err(|_| anyhow!("Invalid token A mint"))?
        );
        let token_b_mint = Pubkey::new_from_array(
            data[432..464].try_into().map_err(|_| anyhow!("Invalid token B mint"))?
        );
        let token_a_vault = Pubkey::new_from_array(
            data[464..496].try_into().map_err(|_| anyhow!("Invalid token A vault"))?
        );
        let token_b_vault = Pubkey::new_from_array(
            data[496..528].try_into().map_err(|_| anyhow!("Invalid token B vault"))?
        );
        let lp_mint = Pubkey::new_from_array(
            data[528..560].try_into().map_err(|_| anyhow!("Invalid LP mint"))?
        );
        
        // Get actual token amounts from vaults
        let token_a_amount = self.get_token_account_balance(&token_a_vault).await?;
        let token_b_amount = self.get_token_account_balance(&token_b_vault).await?;
        let lp_supply = self.get_token_supply(&lp_mint).await?;
        
        Ok(PoolData {
            address: pool_address,
            token_a_mint,
            token_b_mint,
            token_a_vault,
            token_b_vault,
            token_a_amount,
            token_b_amount,
            lp_mint,
            lp_supply,
            pool_type: PoolType::Raydium,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            fees_bps: 25, // Raydium typical fee: 0.25%
        })
    }

    async fn parse_orca_pool(&self, pool_address: Pubkey, account: &Account) -> Result<PoolData> {
        // Orca pool structure parsing
        let data = &account.data;
        
        if data.len() < 324 {
            return Err(anyhow!("Invalid Orca pool data length"));
        }
        
        // Parse Orca pool data structure
        let token_a_mint = Pubkey::new_from_array(
            data[101..133].try_into().map_err(|_| anyhow!("Invalid token A mint"))?
        );
        let token_b_mint = Pubkey::new_from_array(
            data[181..213].try_into().map_err(|_| anyhow!("Invalid token B mint"))?
        );
        let token_a_vault = Pubkey::new_from_array(
            data[85..117].try_into().map_err(|_| anyhow!("Invalid token A vault"))?
        );
        let token_b_vault = Pubkey::new_from_array(
            data[165..197].try_into().map_err(|_| anyhow!("Invalid token B vault"))?
        );
        let lp_mint = Pubkey::new_from_array(
            data[245..277].try_into().map_err(|_| anyhow!("Invalid LP mint"))?
        );
        
        let token_a_amount = self.get_token_account_balance(&token_a_vault).await?;
        let token_b_amount = self.get_token_account_balance(&token_b_vault).await?;
        let lp_supply = self.get_token_supply(&lp_mint).await?;
        
        Ok(PoolData {
            address: pool_address,
            token_a_mint,
            token_b_mint,
            token_a_vault,
            token_b_vault,
            token_a_amount,
            token_b_amount,
            lp_mint,
            lp_supply,
            pool_type: PoolType::Orca,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            fees_bps: 30, // Orca typical fee: 0.30%
        })
    }

    async fn parse_orca_whirlpool(&self, pool_address: Pubkey, account: &Account) -> Result<PoolData> {
        // Orca Whirlpool parsing - more complex concentrated liquidity
        let data = &account.data;
        
        if data.len() < 653 {
            return Err(anyhow!("Invalid Orca Whirlpool data length"));
        }
        
        // Simplified parsing for now - would need full Whirlpool SDK implementation
        let token_a_mint = Pubkey::new_from_array(
            data[101..133].try_into().map_err(|_| anyhow!("Invalid token A mint"))?
        );
        let token_b_mint = Pubkey::new_from_array(
            data[133..165].try_into().map_err(|_| anyhow!("Invalid token B mint"))?
        );
        let token_a_vault = Pubkey::new_from_array(
            data[165..197].try_into().map_err(|_| anyhow!("Invalid token A vault"))?
        );
        let token_b_vault = Pubkey::new_from_array(
            data[197..229].try_into().map_err(|_| anyhow!("Invalid token B vault"))?
        );
        
        let token_a_amount = self.get_token_account_balance(&token_a_vault).await?;
        let token_b_amount = self.get_token_account_balance(&token_b_vault).await?;
        
        Ok(PoolData {
            address: pool_address,
            token_a_mint,
            token_b_mint,
            token_a_vault,
            token_b_vault,
            token_a_amount,
            token_b_amount,
            lp_mint: Pubkey::default(), // Whirlpools don't use traditional LP tokens
            lp_supply: 0,
            pool_type: PoolType::OrcaWhirlpool,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            fees_bps: 30, // Variable, but typical
        })
    }

    // ===== HELPER FUNCTIONS FOR DIRECT POOL ACCESS =====
    
    async fn get_token_account_balance(&self, token_account: &Pubkey) -> Result<u64> {
        let account = self.client.get_account(token_account).await?;
        
        if account.data.len() < 165 {
            return Err(anyhow!("Invalid token account data"));
        }
        
        // Parse token account amount (bytes 64-72)
        let amount_bytes: [u8; 8] = account.data[64..72]
            .try_into()
            .map_err(|_| anyhow!("Failed to parse token amount"))?;
        
        Ok(u64::from_le_bytes(amount_bytes))
    }
    
    async fn get_token_supply(&self, mint: &Pubkey) -> Result<u64> {
        let supply = self.client.get_token_supply(mint).await?;
        Ok(supply.amount.parse()?)
    }

    // ===== DIRECT ARBITRAGE CALCULATION =====
    
    async fn find_direct_arbitrage_opportunities(&self) -> Result<Vec<DirectOpportunity>> {
        let mut opportunities = Vec::new();
        
        // Check all pool pairs for arbitrage opportunities
        let pool_addresses: Vec<_> = self.pools.keys().collect();
        
        for i in 0..pool_addresses.len() {
            for j in (i + 1)..pool_addresses.len() {
                let pool_a = &self.pools[pool_addresses[i]];
                let pool_b = &self.pools[pool_addresses[j]];
                
                // Check if pools share a common token for triangular arbitrage
                if let Some(opportunity) = self.calculate_direct_arbitrage(pool_a, pool_b).await? {
                    if opportunity.profit_lamports > 0 {
                        opportunities.push(opportunity);
                    }
                }
            }
        }
        
        // Sort by profit percentage
        opportunities.sort_by(|a, b| b.profit_percentage.partial_cmp(&a.profit_percentage).unwrap());
        
        if !opportunities.is_empty() {
            info!("   🎯 Found {} profitable opportunities", opportunities.len());
            for (i, opp) in opportunities.iter().take(3).enumerate() {
                info!("     {}. {:.4}% profit ({:.6} SOL)", 
                    i + 1, opp.profit_percentage, opp.profit_lamports as f64 / 1e9);
            }
        }
        
        Ok(opportunities)
    }
    
    async fn calculate_direct_arbitrage(&self, pool_a: &PoolData, pool_b: &PoolData) -> Result<Option<DirectOpportunity>> {
        // Find common token between pools
        let common_token = if pool_a.token_a_mint == pool_b.token_a_mint || 
                             pool_a.token_a_mint == pool_b.token_b_mint {
            pool_a.token_a_mint
        } else if pool_a.token_b_mint == pool_b.token_a_mint || 
                  pool_a.token_b_mint == pool_b.token_b_mint {
            pool_a.token_b_mint
        } else {
            return Ok(None); // No common token
        };
        
        // Calculate potential arbitrage for different amounts
        let test_amounts = vec![
            1_000_000,     // 0.001 SOL equivalent
            5_000_000,     // 0.005 SOL equivalent  
            10_000_000,    // 0.01 SOL equivalent
            50_000_000,    // 0.05 SOL equivalent
        ];
        
        let mut best_opportunity: Option<DirectOpportunity> = None;
        
        for amount in test_amounts {
            // Route 1: Pool A -> Common Token -> Pool B
            if let Ok(route1_profit) = self.calculate_route_profit(pool_a, pool_b, &common_token, amount).await {
                if route1_profit > 0 {
                    let profit_percentage = (route1_profit as f64 / amount as f64) * 100.0;
                    
                    if best_opportunity.is_none() || 
                       profit_percentage > best_opportunity.as_ref().unwrap().profit_percentage {
                        
                        best_opportunity = Some(DirectOpportunity {
                            pool_a: pool_a.clone(),
                            pool_b: pool_b.clone(),
                            intermediate_token: common_token,
                            amount_in: amount,
                            estimated_amount_out: (amount as i64 + route1_profit) as u64,
                            profit_lamports: route1_profit,
                            profit_percentage,
                            execution_path: self.build_execution_path(pool_a, pool_b, &common_token, amount).await?,
                        });
                    }
                }
            }
            
            // Route 2: Pool B -> Common Token -> Pool A  
            if let Ok(route2_profit) = self.calculate_route_profit(pool_b, pool_a, &common_token, amount).await {
                if route2_profit > 0 {
                    let profit_percentage = (route2_profit as f64 / amount as f64) * 100.0;
                    
                    if best_opportunity.is_none() || 
                       profit_percentage > best_opportunity.as_ref().unwrap().profit_percentage {
                        
                        best_opportunity = Some(DirectOpportunity {
                            pool_a: pool_b.clone(),
                            pool_b: pool_a.clone(),
                            intermediate_token: common_token,
                            amount_in: amount,
                            estimated_amount_out: (amount as i64 + route2_profit) as u64,
                            profit_lamports: route2_profit,
                            profit_percentage,
                            execution_path: self.build_execution_path(pool_b, pool_a, &common_token, amount).await?,
                        });
                    }
                }
            }
        }
        
        Ok(best_opportunity)
    }
    
    async fn calculate_route_profit(&self, pool_1: &PoolData, pool_2: &PoolData, 
                                   intermediate_token: &Pubkey, amount_in: u64) -> Result<i64> {
        // Step 1: Calculate output from first pool
        let intermediate_amount = self.calculate_pool_output(pool_1, amount_in, intermediate_token)?;
        
        // Step 2: Calculate output from second pool  
        let final_amount = self.calculate_pool_output(pool_2, intermediate_amount, intermediate_token)?;
        
        // Step 3: Calculate profit (minus estimated fees)
        let estimated_fees = self.calculate_transaction_fees()?;
        let profit = final_amount as i64 - amount_in as i64 - estimated_fees as i64;
        
        Ok(profit)
    }
    
    fn calculate_pool_output(&self, pool: &PoolData, amount_in: u64, output_token: &Pubkey) -> Result<u64> {
        // Constant product formula: x * y = k
        // For swap: new_x = x + amount_in, new_y = k / new_x
        // amount_out = y - new_y
        
        let (reserve_in, reserve_out) = if pool.token_a_mint == *output_token {
            (pool.token_b_amount, pool.token_a_amount)
        } else if pool.token_b_mint == *output_token {
            (pool.token_a_amount, pool.token_b_amount)
        } else {
            return Err(anyhow!("Token not found in pool"));
        };
        
        if reserve_in == 0 || reserve_out == 0 {
            return Err(anyhow!("Pool has no liquidity"));
        }
        
        // Apply fees (subtract from input)
        let amount_in_after_fees = amount_in * (10000 - pool.fees_bps) / 10000;
        
        // Constant product calculation
        let k = reserve_in as u128 * reserve_out as u128;
        let new_reserve_in = reserve_in as u128 + amount_in_after_fees as u128;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = reserve_out as u128 - new_reserve_out;
        
        // Add slippage protection
        let amount_out_with_slippage = amount_out * 990 / 1000; // 1% slippage buffer
        
        Ok(amount_out_with_slippage as u64)
    }
    
    fn calculate_transaction_fees(&self) -> Result<u64> {
        // Base transaction fee + priority fee + account rent
        let base_fee = 5_000; // 0.000005 SOL
        let priority_fee = 10_000; // 0.00001 SOL for speed
        let rent_exemption = 2_039_280; // For temporary accounts
        
        Ok(base_fee + priority_fee + rent_exemption)
    }
    
    async fn build_execution_path(&self, pool_a: &PoolData, pool_b: &PoolData, 
                                 intermediate_token: &Pubkey, amount: u64) -> Result<Vec<SwapInstruction>> {
        let mut instructions = Vec::new();
        
        // Build first swap instruction
        let first_swap = SwapInstruction {
            program_id: self.get_pool_program_id(&pool_a.pool_type)?,
            pool_address: pool_a.address,
            input_mint: if pool_a.token_a_mint == *intermediate_token { 
                pool_a.token_b_mint 
            } else { 
                pool_a.token_a_mint 
            },
            output_mint: *intermediate_token,
            amount_in: amount,
            minimum_amount_out: self.calculate_pool_output(pool_a, amount, intermediate_token)? * 99 / 100, // 1% slippage
            instruction_data: self.build_swap_instruction_data(&pool_a.pool_type, amount)?,
        };
        instructions.push(first_swap);
        
        // Build second swap instruction
        let intermediate_amount = self.calculate_pool_output(pool_a, amount, intermediate_token)?;
        let second_swap = SwapInstruction {
            program_id: self.get_pool_program_id(&pool_b.pool_type)?,
            pool_address: pool_b.address,
            input_mint: *intermediate_token,
            output_mint: if pool_b.token_a_mint == *intermediate_token { 
                pool_b.token_b_mint 
            } else { 
                pool_b.token_a_mint 
            },
            amount_in: intermediate_amount,
            minimum_amount_out: self.calculate_pool_output(pool_b, intermediate_amount, intermediate_token)? * 99 / 100,
            instruction_data: self.build_swap_instruction_data(&pool_b.pool_type, intermediate_amount)?,
        };
        instructions.push(second_swap);
        
        Ok(instructions)
    }
    
    fn get_pool_program_id(&self, pool_type: &PoolType) -> Result<Pubkey> {
        match pool_type {
            PoolType::Raydium => Pubkey::from_str(RAYDIUM_AMM_PROGRAM),
            PoolType::Orca => Pubkey::from_str(ORCA_SWAP_PROGRAM),
            PoolType::OrcaWhirlpool => Pubkey::from_str(ORCA_WHIRLPOOL_PROGRAM),
            PoolType::Serum => Pubkey::from_str(SERUM_DEX_PROGRAM),
        }.map_err(|e| anyhow!("Invalid program ID: {}", e))
    }
    
    fn build_swap_instruction_data(&self, pool_type: &PoolType, amount: u64) -> Result<Vec<u8>> {
        match pool_type {
            PoolType::Raydium => {
                // Raydium swap instruction: [9, amount_in, minimum_amount_out]
                let mut data = vec![9]; // Swap instruction discriminator
                data.extend_from_slice(&amount.to_le_bytes());
                data.extend_from_slice(&(amount * 99 / 100).to_le_bytes()); // 1% slippage
                Ok(data)
            }
            PoolType::Orca => {
                // Orca swap instruction: [1, amount_in, minimum_amount_out]  
                let mut data = vec![1]; // Swap instruction discriminator
                data.extend_from_slice(&amount.to_le_bytes());
                data.extend_from_slice(&(amount * 99 / 100).to_le_bytes());
                Ok(data)
            }
            PoolType::OrcaWhirlpool => {
                // Whirlpool swap is more complex - simplified for now
                let mut data = vec![248, 198, 158, 145, 225, 117, 135, 200]; // swap discriminator
                data.extend_from_slice(&amount.to_le_bytes());
                Ok(data)
            }
            PoolType::Serum => {
                // Serum DEX instruction
                Ok(vec![0]) // Placeholder
            }
        }
    }

    // ===== DIRECT TRANSACTION EXECUTION =====
    
    async fn execute_direct_arbitrage(&mut self, opportunity: &DirectOpportunity) -> Result<String> {
        info!("   ⚔️  Executing direct arbitrage transaction...");
        
        // Build complete transaction manually
        let mut instructions = Vec::new();
        
        // Add swap instructions from execution path
        for swap_instruction in &opportunity.execution_path {
            let instruction = self.build_solana_instruction(swap_instruction).await?;
            instructions.push(instruction);
        }
        
        // Build and send transaction
        let recent_blockhash = self.client.get_latest_blockhash().await?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&self.wallet_address),
            &[&self.keypair],
            recent_blockhash,
        );
        
        // Send with confirmation
        let signature = self.client.send_and_confirm_transaction(&transaction).await?;
        
        Ok(signature.to_string())
    }
    
    async fn build_solana_instruction(&self, swap_instruction: &SwapInstruction) -> Result<Instruction> {
        // Build accounts vector based on pool type
        let accounts = match swap_instruction.program_id.to_string().as_str() {
            RAYDIUM_AMM_PROGRAM => {
                self.build_raydium_accounts(&swap_instruction).await?
            }
            ORCA_SWAP_PROGRAM => {
                self.build_orca_accounts(&swap_instruction).await?
            }
            ORCA_WHIRLPOOL_PROGRAM => {
                self.build_whirlpool_accounts(&swap_instruction).await?
            }
            _ => {
                return Err(anyhow!("Unsupported pool program"));
            }
        };
        
        Ok(Instruction {
            program_id: swap_instruction.program_id,
            accounts,
            data: swap_instruction.instruction_data.clone(),
        })
    }
    
    async fn build_raydium_accounts(&self, swap_instruction: &SwapInstruction) -> Result<Vec<solana_sdk::instruction::AccountMeta>> {
        use solana_sdk::instruction::AccountMeta;
        
        // Get pool data
        let pool_key = swap_instruction.pool_address.to_string();
        let pool = self.pools.get(&pool_key)
            .ok_or_else(|| anyhow!("Pool not found"))?;
        
        // User token accounts
        let user_input_ata = get_associated_token_address(&self.wallet_address, &swap_instruction.input_mint);
        let user_output_ata = get_associated_token_address(&self.wallet_address, &swap_instruction.output_mint);
        
        Ok(vec![
            AccountMeta::new_readonly(Pubkey::from_str(TOKEN_PROGRAM_ID)?, false),
            AccountMeta::new(swap_instruction.pool_address, false),
            AccountMeta::new_readonly(Pubkey::from_str(RAYDIUM_AUTHORITY)?, false),
            AccountMeta::new(user_input_ata, false),
            AccountMeta::new(user_output_ata, false),
            AccountMeta::new(pool.token_a_vault, false),
            AccountMeta::new(pool.token_b_vault, false),
            AccountMeta::new_readonly(self.wallet_address, true),
        ])
    }
    
    async fn build_orca_accounts(&self, swap_instruction: &SwapInstruction) -> Result<Vec<solana_sdk::instruction::AccountMeta>> {
        use solana_sdk::instruction::AccountMeta;
        
        let pool_key = swap_instruction.pool_address.to_string();
        let pool = self.pools.get(&pool_key)
            .ok_or_else(|| anyhow!("Pool not found"))?;
        
        let user_input_ata = get_associated_token_address(&self.wallet_address, &swap_instruction.input_mint);
        let user_output_ata = get_associated_token_address(&self.wallet_address, &swap_instruction.output_mint);
        
        Ok(vec![
            AccountMeta::new_readonly(Pubkey::from_str(TOKEN_PROGRAM_ID)?, false),
            AccountMeta::new(swap_instruction.pool_address, false),
            AccountMeta::new_readonly(self.wallet_address, true),
            AccountMeta::new(user_input_ata, false),
            AccountMeta::new(user_output_ata, false),
            AccountMeta::new(pool.token_a_vault, false),
            AccountMeta::new(pool.token_b_vault, false),
            AccountMeta::new_readonly(pool.lp_mint, false),
        ])
    }
    
    async fn build_whirlpool_accounts(&self, swap_instruction: &SwapInstruction) -> Result<Vec<solana_sdk::instruction::AccountMeta>> {
        // Simplified Whirlpool accounts - would need full implementation
        self.build_orca_accounts(swap_instruction).await
    }

    async fn get_wallet_balance(&self) -> Result<f64> {
        let balance_lamports = self.client.get_balance(&self.wallet_address).await?;
        Ok(balance_lamports as f64 / 1_000_000_000.0)
    }
}
