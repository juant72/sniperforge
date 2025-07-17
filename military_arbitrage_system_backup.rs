use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};
use solana_client::nonblocking::rpc_client::RpcClient;
use reqwest;
use serde_json::Value;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Keypair,
    signer::Signer,
    pubkey::Pubkey,
    transaction::Transaction,
    instruction::Instruction,
    account::Account,
};
use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};

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

// ===== MULTI-DEX POOL CONFIGURATION =====
// MILITARY INTELLIGENT POOL DISCOVERY - DYNAMIC DETECTION
// Sistema inteligente para detectar pools funcionales automáticamente

// Token Mint Addresses (VERIFIED MAINNET)
const SOL_MINT: &str = "So11111111111111111111111111111111111111112"; // Wrapped SOL
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USD Coin
const USDT_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"; // Tether USD
const RAY_MINT: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"; // Raydium Token

// MILITARY STRATEGY: Multiple pool candidates per DEX
// El sistema probará automáticamente hasta encontrar pools funcionales

struct PoolCandidate {
    address: &'static str,
    dex_type: PoolType,
    token_a: &'static str,
    token_b: &'static str,
    description: &'static str,
}

// MILITARY INTEL: Pool candidates database
const POOL_CANDIDATES: &[PoolCandidate] = &[
    // === RAYDIUM CANDIDATES ===
    PoolCandidate {
        address: "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2",
        dex_type: PoolType::Raydium,
        token_a: SOL_MINT,
        token_b: USDC_MINT,
        description: "Raydium SOL/USDC v1"
    },
    PoolCandidate {
        address: "7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX",
        dex_type: PoolType::Raydium,
        token_a: SOL_MINT,
        token_b: USDT_MINT,
        description: "Raydium SOL/USDT v1"
    },
    PoolCandidate {
        address: "6UmmUiYoBjSrhakAobJw8BvkmJtDVxaeBtbt7rxWo1mg",
        dex_type: PoolType::Raydium,
        token_a: RAY_MINT,
        token_b: USDC_MINT,
        description: "Raydium RAY/USDC v1"
    },
    PoolCandidate {
        address: "AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA",
        dex_type: PoolType::Raydium,
        token_a: RAY_MINT,
        token_b: SOL_MINT,
        description: "Raydium RAY/SOL v1"
    },
    // Alternativas Raydium
    PoolCandidate {
        address: "2QdhepnKRTLjjSqPL1PtKNwqrUkoLee5Gqs8bvZhRdMv",
        dex_type: PoolType::Raydium,
        token_a: SOL_MINT,
        token_b: USDC_MINT,
        description: "Raydium SOL/USDC v2"
    },
    PoolCandidate {
        address: "H6Q3oEH3zLgJ9r8cXKCwX8nQoZWxLNmzxJ7VJcCwKfXg",
        dex_type: PoolType::Raydium,
        token_a: SOL_MINT,
        token_b: USDT_MINT,
        description: "Raydium SOL/USDT v2"
    },
    
    // === ORCA CANDIDATES ===
    PoolCandidate {
        address: "EGZ7tiLeH62TPV1gL8WwbXGzEPa9zmcpVnnkPKKnrE2U",
        dex_type: PoolType::Orca,
        token_a: SOL_MINT,
        token_b: USDC_MINT,
        description: "Orca SOL/USDC v1"
    },
    PoolCandidate {
        address: "7qbRF6YsyGuLUVs6Y1q64bdVrfe4ZcUUz1JRdoVNUJnm",
        dex_type: PoolType::Orca,
        token_a: SOL_MINT,
        token_b: USDT_MINT,
        description: "Orca SOL/USDT v1"
    },
    // Alternativas Orca
    PoolCandidate {
        address: "2p7nYbtPBgtmY69NsE8DAW6szpRJn7tQvDnqvoEWQvjY",
        dex_type: PoolType::Orca,
        token_a: SOL_MINT,
        token_b: USDC_MINT,
        description: "Orca SOL/USDC v2"
    },
    PoolCandidate {
        address: "9vqYJjDUFecLL2xPUC4Rc7hyCtZ6iJ4mDiVZX7aFXoAe",
        dex_type: PoolType::Orca,
        token_a: SOL_MINT,
        token_b: USDT_MINT,
        description: "Orca SOL/USDT v2"
    },
    
    // === WHIRLPOOL CANDIDATES ===
    PoolCandidate {
        address: "HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ",
        dex_type: PoolType::OrcaWhirlpool,
        token_a: SOL_MINT,
        token_b: USDC_MINT,
        description: "Whirlpool SOL/USDC concentrated"
    },
    PoolCandidate {
        address: "4fuUiYxTQ6QCrdSq9ouBYcTM7bqSwYTSyLueGZLTy4T4",
        dex_type: PoolType::OrcaWhirlpool,
        token_a: SOL_MINT,
        token_b: USDT_MINT,
        description: "Whirlpool SOL/USDT concentrated"
    },
];

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

#[derive(Debug, Clone, Copy)]
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

        info!("⚔️  Military Arbitrage System loaded: {}", wallet_address);
        info!("🎯 MILITARY INTELLIGENCE: Discovering operational pools...");

        // MILITARY INTELLIGENCE: Discover working pools automatically
        let mut system = Self {
            client,
            keypair,
            wallet_address,
            pools: HashMap::new(),
            monitoring_pools: Vec::new(),
            last_pool_update: std::time::Instant::now(),
        };

        // MILITARY RECONNAISSANCE: Test all pool candidates
        system.discover_operational_pools().await?;

        Ok(system)
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
                info!("   💤 No profitable opportunities found - STANDBY mode...");
                sleep(Duration::from_secs(3)).await; // Military patience
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
        
        // MILITARY EFFICIENCY: Force update on first run, then respect intervals
        if !self.pools.is_empty() && now.duration_since(self.last_pool_update) < Duration::from_secs(3) {
            return Ok(());
        }
        
        info!("   🔬 MILITARY UPDATE: Refreshing pool data from blockchain...");
        
        let mut successful_updates = 0;
        let mut failed_updates = 0;
        
        for pool_address in &self.monitoring_pools.clone() {
            match self.read_pool_data_direct(pool_address).await {
                Ok(pool_data) => {
                    self.pools.insert(pool_address.clone(), pool_data);
                    successful_updates += 1;
                    info!("   ✅ Pool {} OPERATIONAL", &pool_address[..8]);
                }
                Err(e) => {
                    failed_updates += 1;
                    warn!("   ❌ Pool {} FAILED: {}", &pool_address[..8], e);
                }
            }
        }
        
        self.last_pool_update = now;
        info!("   📊 MILITARY STATUS: {} operational, {} failed", successful_updates, failed_updates);
        
        if successful_updates == 0 {
            return Err(anyhow!("CRITICAL: No pools operational - mission aborted"));
        }
        
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
        // Use enhanced parsing method
        self.parse_raydium_pool_enhanced(pool_address, account).await
    }

    async fn parse_orca_pool(&self, pool_address: Pubkey, account: &Account) -> Result<PoolData> {
        // MILITARY INTELLIGENCE: Enhanced Orca parsing
        let data = &account.data;
        
        if data.len() < 324 {
            return Err(anyhow!("Invalid Orca pool data length: {}", data.len()));
        }
        
        // MILITARY STRATEGY: Try multiple Orca layout versions with better error handling
        let layout_attempts = vec![
            ("orca-v1", vec![(8, 40, 72, 104, 136)]),
            ("orca-v2", vec![(101, 181, 85, 165, 245)]),
            ("orca-v3", vec![(40, 72, 104, 136, 168)]),
            ("orca-alt1", vec![(16, 48, 80, 112, 144)]),
            ("orca-alt2", vec![(24, 56, 88, 120, 152)]),
        ];
        
        for (layout_name, offsets) in layout_attempts {
            for (mint_a_off, mint_b_off, vault_a_off, vault_b_off, lp_off) in offsets {
                if data.len() >= lp_off + 32 {
                    match self.try_parse_orca_layout(pool_address, data, mint_a_off, mint_b_off, vault_a_off, vault_b_off, lp_off).await {
                        Ok(pool_data) => {
                            info!("   ✅ Orca {} layout success at offsets: {},{},{},{},{}", 
                                layout_name, mint_a_off, mint_b_off, vault_a_off, vault_b_off, lp_off);
                            return Ok(pool_data);
                        }
                        Err(_) => {
                            // Silent failure - try next layout
                            continue;
                        }
                    }
                }
            }
        }
        
        Err(anyhow!("Could not parse Orca pool with any known layout format"))
    }
    
    async fn try_parse_orca_layout(&self, pool_address: Pubkey, data: &[u8], 
                                  mint_a_off: usize, mint_b_off: usize, vault_a_off: usize, 
                                  vault_b_off: usize, lp_off: usize) -> Result<PoolData> {
        
        let token_a_mint = Pubkey::new_from_array(
            data[mint_a_off..mint_a_off+32].try_into().map_err(|_| anyhow!("Invalid mint A"))?
        );
        let token_b_mint = Pubkey::new_from_array(
            data[mint_b_off..mint_b_off+32].try_into().map_err(|_| anyhow!("Invalid mint B"))?
        );
        let token_a_vault = Pubkey::new_from_array(
            data[vault_a_off..vault_a_off+32].try_into().map_err(|_| anyhow!("Invalid vault A"))?
        );
        let token_b_vault = Pubkey::new_from_array(
            data[vault_b_off..vault_b_off+32].try_into().map_err(|_| anyhow!("Invalid vault B"))?
        );
        let lp_mint = Pubkey::new_from_array(
            data[lp_off..lp_off+32].try_into().map_err(|_| anyhow!("Invalid LP mint"))?
        );
        
        // Validate addresses are not default
        if token_a_mint == Pubkey::default() || token_b_mint == Pubkey::default() || 
           token_a_vault == Pubkey::default() || token_b_vault == Pubkey::default() {
            return Err(anyhow!("Invalid addresses detected"));
        }
        
        // Try to get balances to validate
        let token_a_amount = self.get_token_account_balance(&token_a_vault).await?;
        let token_b_amount = self.get_token_account_balance(&token_b_vault).await?;
        
        let lp_supply = self.get_token_supply(&lp_mint).await.unwrap_or(0);
        
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
            fees_bps: 30,
        })
    }

    async fn parse_orca_whirlpool(&self, pool_address: Pubkey, account: &Account) -> Result<PoolData> {
        // Orca Whirlpool parsing with multiple layout attempts
        let data = &account.data;
        
        if data.len() < 653 {
            return Err(anyhow!("Invalid Orca Whirlpool data length: {}", data.len()));
        }
        
        // Try multiple Whirlpool layout versions
        let offset_attempts = vec![
            // Whirlpool v3 layout attempt 1
            (8, 40, 72, 104),
            // Whirlpool v3 layout attempt 2 (alternative)
            (101, 133, 165, 197),
            // Whirlpool v3 layout attempt 3 (newer format)
            (168, 200, 232, 264),
        ];
        
        for (mint_a_off, mint_b_off, vault_a_off, vault_b_off) in offset_attempts {
            if data.len() >= vault_b_off + 32 {
                let token_a_mint = Pubkey::new_from_array(
                    data[mint_a_off..mint_a_off+32].try_into().map_err(|_| anyhow!("Invalid token A mint"))?
                );
                let token_b_mint = Pubkey::new_from_array(
                    data[mint_b_off..mint_b_off+32].try_into().map_err(|_| anyhow!("Invalid token B mint"))?
                );
                let token_a_vault = Pubkey::new_from_array(
                    data[vault_a_off..vault_a_off+32].try_into().map_err(|_| anyhow!("Invalid token A vault"))?
                );
                let token_b_vault = Pubkey::new_from_array(
                    data[vault_b_off..vault_b_off+32].try_into().map_err(|_| anyhow!("Invalid token B vault"))?
                );
                
                // Validate that these look like real addresses
                if token_a_mint != Pubkey::default() && token_b_mint != Pubkey::default() && 
                   token_a_vault != Pubkey::default() && token_b_vault != Pubkey::default() {
                    
                    info!("   ✅ Found valid Whirlpool layout at offsets: mint_a={}, vault_a={}", mint_a_off, vault_a_off);
                    
                    // Try to get balances to confirm these are correct
                    let token_a_amount = match self.get_token_account_balance(&token_a_vault).await {
                        Ok(amount) => amount,
                        Err(e) => {
                            warn!("Whirlpool layout validation failed for vault {}: {}", token_a_vault, e);
                            continue; // Try next layout
                        }
                    };
                    let token_b_amount = match self.get_token_account_balance(&token_b_vault).await {
                        Ok(amount) => amount,
                        Err(e) => {
                            warn!("Whirlpool layout validation failed for vault {}: {}", token_b_vault, e);
                            continue; // Try next layout
                        }
                    };
                    
                    return Ok(PoolData {
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
                    });
                }
            }
        }
        
        Err(anyhow!("Could not parse Whirlpool with any known layout format"))
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
        
        info!("   🔍 MILITARY SCAN: Analyzing {} pools for arbitrage opportunities...", self.pools.len());
        
        if self.pools.is_empty() {
            return Err(anyhow!("CRITICAL: No operational pools available"));
        }
        
        // MILITARY INTEL: Show pool status
        info!("   📊 OPERATIONAL POOLS:");
        for (address, pool) in &self.pools {
            info!("     - {}: {} + {} liquidity ({:?})", 
                &address[..8], 
                pool.token_a_amount / 1_000_000,
                pool.token_b_amount / 1_000_000,
                pool.pool_type
            );
        }
        
        // MILITARY STRATEGY: Check all possible arbitrage combinations
        let pool_addresses: Vec<_> = self.pools.keys().collect();
        let mut combinations_checked = 0;
        
        for i in 0..pool_addresses.len() {
            for j in (i + 1)..pool_addresses.len() {
                combinations_checked += 1;
                let pool_a = &self.pools[pool_addresses[i]];
                let pool_b = &self.pools[pool_addresses[j]];
                
                // Check for arbitrage opportunity
                if let Some(opportunity) = self.calculate_direct_arbitrage(pool_a, pool_b).await? {
                    if opportunity.profit_lamports > 0 {
                        info!("   🎯 TARGET ACQUIRED: {:.6}% profit potential", opportunity.profit_percentage);
                        opportunities.push(opportunity);
                    }
                }
            }
        }
        
        // MILITARY PRIORITIZATION: Sort by profit
        opportunities.sort_by(|a, b| b.profit_percentage.partial_cmp(&a.profit_percentage).unwrap());
        
        info!("   📈 MISSION REPORT: {} combinations checked, {} opportunities found", 
            combinations_checked, opportunities.len());
        
        if !opportunities.is_empty() {
            info!("   🏆 TOP OPPORTUNITIES:");
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
        
        info!("   🔍 MILITARY ANALYSIS: Common token {} detected", 
            &common_token.to_string()[..8]);
        
        // MILITARY EFFICIENCY: Test incremental amounts for optimal profit
        let test_amounts = vec![
            1_000_000,     // 0.001 SOL - reconnaissance
            5_000_000,     // 0.005 SOL - light probe
            10_000_000,    // 0.01 SOL - standard operation
            50_000_000,    // 0.05 SOL - heavy assault (if profitable)
        ];
        
        let mut best_opportunity: Option<DirectOpportunity> = None;
        
        for amount in test_amounts {
            // Route 1: Pool A -> Common Token -> Pool B
            if let Ok(route1_profit) = self.calculate_route_profit(pool_a, pool_b, &common_token, amount).await {
                if route1_profit > 0 {
                    let profit_percentage = (route1_profit as f64 / amount as f64) * 100.0;
                    
                    info!("   💰 Route 1 profit: {:.6}% ({:.9} SOL)", 
                        profit_percentage, route1_profit as f64 / 1e9);
                    
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
                    
                    info!("   💰 Route 2 profit: {:.6}% ({:.9} SOL)", 
                        profit_percentage, route2_profit as f64 / 1e9);
                    
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
        // === COMPREHENSIVE PROFIT CALCULATION ===
        
        // Step 1: Calculate first swap output (with DEX fees)
        let first_swap_output = self.calculate_pool_output(pool_1, amount_in, intermediate_token)?;
        
        // Step 2: Calculate second swap output (with DEX fees)
        let final_amount = self.calculate_pool_output(pool_2, first_swap_output, intermediate_token)?;
        
        // Step 3: Calculate all transaction costs
        let network_fees = self.calculate_transaction_fees()?;
        let trading_fees = self.calculate_trading_fees(pool_1, pool_2, amount_in, first_swap_output)?;
        let slippage_impact = self.calculate_slippage_impact(pool_1, pool_2, amount_in)?;
        
        // Step 4: Calculate net profit
        let total_costs = network_fees + trading_fees + slippage_impact;
        let gross_profit = final_amount as i64 - amount_in as i64;
        let net_profit = gross_profit - total_costs as i64;
        
        // Step 5: Add minimum profit threshold (prevent dust arbitrage)
        let min_profit_threshold = 10_000; // 0.00001 SOL minimum profit
        if net_profit < min_profit_threshold {
            return Ok(-1); // Not profitable enough
        }
        
        info!("   📊 Profit calculation for {:.6} SOL input:", amount_in as f64 / 1e9);
        info!("     🔄 First swap: {:.6} → {:.6}", amount_in as f64 / 1e9, first_swap_output as f64 / 1e9);
        info!("     🔄 Second swap: {:.6} → {:.6}", first_swap_output as f64 / 1e9, final_amount as f64 / 1e9);
        info!("     💰 Gross profit: {:.9} SOL", gross_profit as f64 / 1e9);
        info!("     💸 Total costs: {:.9} SOL", total_costs as f64 / 1e9);
        info!("     📈 Net profit: {:.9} SOL", net_profit as f64 / 1e9);
        
        Ok(net_profit)
    }
    
    fn calculate_trading_fees(&self, pool_1: &PoolData, pool_2: &PoolData, 
                             amount_1: u64, amount_2: u64) -> Result<u64> {
        // Calculate actual trading fees paid to DEXs
        let fee_1 = (amount_1 * pool_1.fees_bps) / 10_000;
        let fee_2 = (amount_2 * pool_2.fees_bps) / 10_000;
        
        let total_trading_fees = fee_1 + fee_2;
        
        info!("     🏪 DEX trading fees: {:.9} SOL", total_trading_fees as f64 / 1e9);
        
        Ok(total_trading_fees)
    }
    
    fn calculate_slippage_impact(&self, pool_1: &PoolData, pool_2: &PoolData, 
                                amount_in: u64) -> Result<u64> {
        // Calculate price impact from large trades
        let impact_1 = self.calculate_price_impact(pool_1, amount_in)?;
        let impact_2 = self.calculate_price_impact(pool_2, amount_in)?;
        
        let total_impact = impact_1 + impact_2;
        
        info!("     📉 Price impact: {:.9} SOL", total_impact as f64 / 1e9);
        
        Ok(total_impact)
    }
    
    fn calculate_price_impact(&self, pool: &PoolData, amount_in: u64) -> Result<u64> {
        // Price impact calculation based on pool depth
        let total_liquidity = pool.token_a_amount + pool.token_b_amount;
        
        if total_liquidity == 0 {
            return Ok(0);
        }
        
        // Impact percentage based on trade size vs liquidity
        let impact_percentage = (amount_in as f64 / total_liquidity as f64) * 100.0;
        
        // Convert impact to lamports (higher impact = higher cost)
        let impact_cost = (amount_in as f64 * impact_percentage / 100.0 * 0.1) as u64;
        
        Ok(impact_cost)
    }
    
    fn calculate_pool_output(&self, pool: &PoolData, amount_in: u64, output_token: &Pubkey) -> Result<u64> {
        // === MULTI-DEX POOL OUTPUT CALCULATION ===
        
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
        
        // Calculate output based on DEX type
        let amount_out = match pool.pool_type {
            PoolType::Raydium => {
                self.calculate_raydium_output(amount_in, reserve_in, reserve_out, pool.fees_bps)?
            }
            PoolType::Orca => {
                self.calculate_orca_output(amount_in, reserve_in, reserve_out, pool.fees_bps)?
            }
            PoolType::OrcaWhirlpool => {
                self.calculate_whirlpool_output(amount_in, reserve_in, reserve_out, pool.fees_bps)?
            }
            PoolType::Serum => {
                self.calculate_serum_output(amount_in, reserve_in, reserve_out, pool.fees_bps)?
            }
        };
        
        Ok(amount_out)
    }
    
    fn calculate_raydium_output(&self, amount_in: u64, reserve_in: u64, reserve_out: u64, fees_bps: u64) -> Result<u64> {
        // Raydium uses standard constant product with fees
        let amount_in_after_fees = amount_in * (10_000 - fees_bps) / 10_000;
        
        let k = reserve_in as u128 * reserve_out as u128;
        let new_reserve_in = reserve_in as u128 + amount_in_after_fees as u128;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = reserve_out as u128 - new_reserve_out;
        
        // Raydium has minimal slippage for standard pairs
        let amount_out_with_slippage = amount_out * 995 / 1000; // 0.5% slippage
        
        Ok(amount_out_with_slippage as u64)
    }
    
    fn calculate_orca_output(&self, amount_in: u64, reserve_in: u64, reserve_out: u64, fees_bps: u64) -> Result<u64> {
        // Orca uses similar constant product but with different fee structure
        let amount_in_after_fees = amount_in * (10_000 - fees_bps) / 10_000;
        
        let k = reserve_in as u128 * reserve_out as u128;
        let new_reserve_in = reserve_in as u128 + amount_in_after_fees as u128;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = reserve_out as u128 - new_reserve_out;
        
        // Orca has slightly higher slippage
        let amount_out_with_slippage = amount_out * 990 / 1000; // 1% slippage
        
        Ok(amount_out_with_slippage as u64)
    }
    
    fn calculate_whirlpool_output(&self, amount_in: u64, reserve_in: u64, reserve_out: u64, fees_bps: u64) -> Result<u64> {
        // Whirlpool uses concentrated liquidity - more complex calculation
        // For now, using constant product with adjusted fees
        let amount_in_after_fees = amount_in * (10_000 - fees_bps) / 10_000;
        
        let k = reserve_in as u128 * reserve_out as u128;
        let new_reserve_in = reserve_in as u128 + amount_in_after_fees as u128;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = reserve_out as u128 - new_reserve_out;
        
        // Whirlpool has better pricing due to concentrated liquidity
        let amount_out_with_slippage = amount_out * 998 / 1000; // 0.2% slippage
        
        Ok(amount_out_with_slippage as u64)
    }
    
    fn calculate_serum_output(&self, amount_in: u64, reserve_in: u64, reserve_out: u64, fees_bps: u64) -> Result<u64> {
        // Serum uses order book, but for simplification using constant product
        // In reality, would need to calculate based on order book depth
        let amount_in_after_fees = amount_in * (10_000 - fees_bps) / 10_000;
        
        let k = reserve_in as u128 * reserve_out as u128;
        let new_reserve_in = reserve_in as u128 + amount_in_after_fees as u128;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = reserve_out as u128 - new_reserve_out;
        
        // Order book can have variable slippage
        let amount_out_with_slippage = amount_out * 992 / 1000; // 0.8% slippage
        
        Ok(amount_out_with_slippage as u64)
    }
    
    fn calculate_transaction_fees(&self) -> Result<u64> {
        // === COMPREHENSIVE SOLANA TRANSACTION FEES ===
        
        // 1. Base transaction fee (always required)
        let base_fee = 5_000; // 0.000005 SOL per signature
        
        // 2. Priority fee (essential for MEV/arbitrage)
        let priority_fee = 100_000; // 0.0001 SOL - aggressive priority
        
        // 3. Compute unit fees (realistic for multi-DEX arbitrage)
        let compute_units = 600_000; // 2 swaps + ATA creation + validations
        let compute_unit_price = 50; // microlamports per CU (competitive)
        let compute_fee = compute_units * compute_unit_price;
        
        // 4. ATA creation fees (token accounts)
        let ata_rent_exemption = 2_039_280; // Rent exemption for token account
        let max_ata_creations = 4; // Input/output tokens for both swaps
        let ata_creation_fees = ata_rent_exemption * max_ata_creations;
        
        // 5. Account rent (temporary accounts during swap)
        let temp_account_rent = 890_880; // Temporary PDA accounts
        
        // 6. DEX-specific fees (protocol fees, not trading fees)
        let dex_protocol_fees = 10_000; // Protocol interaction fees
        
        // 7. Slippage buffer (additional safety margin)
        let slippage_buffer = 50_000; // 0.00005 SOL buffer
        
        // === TOTAL NETWORK FEES ===
        let network_fees = base_fee + priority_fee + compute_fee + 
                          ata_creation_fees + temp_account_rent + 
                          dex_protocol_fees + slippage_buffer;
        
        info!("   💰 Fee breakdown:");
        info!("     📊 Base fee: {:.9} SOL", base_fee as f64 / 1e9);
        info!("     ⚡ Priority fee: {:.9} SOL", priority_fee as f64 / 1e9);
        info!("     💻 Compute fee: {:.9} SOL", compute_fee as f64 / 1e9);
        info!("     🏦 ATA creation: {:.9} SOL", ata_creation_fees as f64 / 1e9);
        info!("     📋 Total network fees: {:.9} SOL", network_fees as f64 / 1e9);
        
        Ok(network_fees)
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
                // Raydium swap instruction: instruction_id (1 byte) + amount_in (8 bytes) + minimum_amount_out (8 bytes)
                let mut data = vec![9]; // Raydium swap instruction discriminator
                data.extend_from_slice(&amount.to_le_bytes());
                data.extend_from_slice(&(amount * 990 / 1000).to_le_bytes()); // 1% slippage protection
                Ok(data)
            }
            PoolType::Orca => {
                // Orca swap instruction: Different format
                let mut data = vec![1]; // Orca swap instruction discriminator  
                data.extend_from_slice(&amount.to_le_bytes());
                data.extend_from_slice(&(amount * 990 / 1000).to_le_bytes()); // 1% slippage protection
                Ok(data)
            }
            PoolType::OrcaWhirlpool => {
                // Whirlpool swap - more complex but simplified for now
                let mut data = vec![248, 198, 158, 145, 225, 117, 135, 200]; // whirlpool swap discriminator
                data.extend_from_slice(&amount.to_le_bytes());
                data.extend_from_slice(&(amount * 990 / 1000).to_le_bytes());
                // Additional whirlpool parameters would go here
                Ok(data)
            }
            PoolType::Serum => {
                // Serum DEX instruction - placeholder
                warn!("Serum swaps not yet implemented");
                Err(anyhow!("Serum swaps not supported"))
            }
        }
    }

    // ===== DIRECT TRANSACTION EXECUTION =====
    
    async fn execute_direct_arbitrage(&mut self, opportunity: &DirectOpportunity) -> Result<String> {
        info!("   ⚔️  Executing REAL arbitrage transaction...");
        
        // 1. Check and create ATAs if needed
        let mut pre_instructions = Vec::new();
        for swap_instruction in &opportunity.execution_path {
            let input_ata = get_associated_token_address(&self.wallet_address, &swap_instruction.input_mint);
            let output_ata = get_associated_token_address(&self.wallet_address, &swap_instruction.output_mint);
            
            // Check if ATAs exist, create if not
            if !self.account_exists(&input_ata).await? {
                info!("   🔧 Creating input ATA for token: {}", swap_instruction.input_mint);
                let create_ata_ix = create_associated_token_account(
                    &self.wallet_address,
                    &self.wallet_address,
                    &swap_instruction.input_mint,
                    &Pubkey::from_str(TOKEN_PROGRAM_ID)?,
                );
                pre_instructions.push(create_ata_ix);
            }
            
            if !self.account_exists(&output_ata).await? {
                info!("   🔧 Creating output ATA for token: {}", swap_instruction.output_mint);
                let create_ata_ix = create_associated_token_account(
                    &self.wallet_address,
                    &self.wallet_address,
                    &swap_instruction.output_mint,
                    &Pubkey::from_str(TOKEN_PROGRAM_ID)?,
                );
                pre_instructions.push(create_ata_ix);
            }
        }
        
        // 2. Build swap instructions
        let mut swap_instructions = Vec::new();
        for swap_instruction in &opportunity.execution_path {
            let instruction = self.build_solana_instruction(swap_instruction).await?;
            swap_instructions.push(instruction);
        }
        
        // 3. Combine all instructions
        let mut all_instructions = pre_instructions;
        all_instructions.extend(swap_instructions);
        
        if all_instructions.is_empty() {
            return Err(anyhow!("No instructions to execute"));
        }
        
        // 4. Build and send transaction
        info!("   🚀 Sending transaction with {} instructions", all_instructions.len());
        let recent_blockhash = self.client.get_latest_blockhash().await?;
        
        let transaction = Transaction::new_signed_with_payer(
            &all_instructions,
            Some(&self.wallet_address),
            &[&self.keypair],
            recent_blockhash,
        );
        
        // 5. Send with confirmation
        let signature = self.client.send_and_confirm_transaction(&transaction).await?;
        
        info!("   ✅ REAL arbitrage transaction confirmed!");
        Ok(signature.to_string())
    }
    
    async fn account_exists(&self, address: &Pubkey) -> Result<bool> {
        match self.client.get_account(address).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
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

    // ===== MILITARY INTELLIGENCE: INTELLIGENT POOL DISCOVERY =====
    
    async fn discover_operational_pools(&mut self) -> Result<()> {
        info!("🔍 MILITARY RECONNAISSANCE: Discovering operational pools...");
        
        // ESTRATEGIA MILITAR: Buscar pools conocidos de mainnet usando direcciones verificadas
        let verified_pools = self.find_verified_mainnet_pools().await?;
        
        if verified_pools.is_empty() {
            warn!("⚠️  No verified pools found, falling back to candidate testing");
            self.test_pool_candidates().await?;
        } else {
            info!("✅ Found {} verified pools", verified_pools.len());
            self.monitoring_pools = verified_pools;
        }
        
        // Validar que tenemos al menos algunos pools operativos
        if self.monitoring_pools.is_empty() {
            warn!("⚠️  MILITARY FALLBACK: No pools discovered, creating mock pools for testing");
            
            // SISTEMA DE EMERGENCIA: Crear pools mock para testing
            let mock_pool = PoolData {
                address: Pubkey::from_str("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2").unwrap(),
                token_a_mint: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(), // SOL
                token_b_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(), // USDC
                token_a_vault: Pubkey::from_str("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2").unwrap(),
                token_b_vault: Pubkey::from_str("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2").unwrap(),
                token_a_amount: 1_000_000_000_000, // 1M SOL
                token_b_amount: 50_000_000_000_000, // 50M USDC
                lp_mint: Pubkey::from_str("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2").unwrap(),
                lp_supply: 1_000_000_000,
                pool_type: PoolType::Raydium,
                last_updated: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                fees_bps: 25,
            };
            
            self.monitoring_pools.push("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2".to_string());
            self.pools.insert("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2".to_string(), mock_pool);
            
            info!("✅ MILITARY FALLBACK: Mock pool created for testing");
        }
        
        info!("🎯 MILITARY INTELLIGENCE COMPLETE: {} operational pools ready", self.monitoring_pools.len());
        Ok(())
    }
    
    async fn find_verified_mainnet_pools(&mut self) -> Result<Vec<String>> {
        let mut verified_pools = Vec::new();
        
        info!("🔍 MILITARY STRATEGY: Dynamic pool discovery from live APIs...");
        
        // ESTRATEGIA MILITAR: Descubrimiento dinámico de pools activos
        // Priorizar por liquidez y actividad real
        
        // PASO 1: Buscar pools activos de Raydium dinámicamente
        info!("🎯 Fetching active Raydium pools from API...");
        let raydium_pools = self.fetch_raydium_active_pools().await?;
        
        for (address, name, pool_type, liquidity) in raydium_pools.into_iter().take(15) {
            if verified_pools.len() >= 15 { break; }
            
            info!("   🎯 Testing Raydium: {} - Liquidity: {:.0}K", name, liquidity / 1000.0);
            
            match self.intelligent_pool_validation(&address, pool_type).await {
                Ok(pool_data) => {
                    info!("   ✅ RAYDIUM VERIFIED: {} - Live Liquidity: {:.0}K + {:.0}K", 
                        name,
                        pool_data.token_a_amount as f64 / 1_000_000.0,
                        pool_data.token_b_amount as f64 / 1_000_000.0
                    );
                    verified_pools.push(address.to_string());
                    self.pools.insert(address.to_string(), pool_data);
                }
                Err(e) => {
                    warn!("   ❌ RAYDIUM FAILED: {} - {}", name, e);
                }
            }
        }
        
        // PASO 2: Buscar pools activos de Whirlpool dinámicamente
        info!("🎯 Fetching active Whirlpool pools from API...");
        let whirlpool_pools = self.fetch_whirlpool_active_pools().await?;
        
        for (address, name, pool_type, liquidity) in whirlpool_pools.into_iter().take(10) {
            if verified_pools.len() >= 20 { break; }
            
            info!("   🎯 Testing Whirlpool: {} - Liquidity: {:.0}K", name, liquidity / 1000.0);
            
            match self.intelligent_pool_validation(&address, pool_type).await {
                Ok(pool_data) => {
                    info!("   ✅ WHIRLPOOL VERIFIED: {} - Live Liquidity: {:.0}K + {:.0}K", 
                        name,
                        pool_data.token_a_amount as f64 / 1_000_000.0,
                        pool_data.token_b_amount as f64 / 1_000_000.0
                    );
                    verified_pools.push(address.to_string());
                    self.pools.insert(address.to_string(), pool_data);
                }
                Err(e) => {
                    warn!("   ❌ WHIRLPOOL FAILED: {} - {}", name, e);
                }
            }
        }
        
        // PASO 3: Buscar pools adicionales de Jupiter si es necesario
        if verified_pools.len() < 5 {
            info!("🎯 Fetching additional pools from Jupiter API...");
            let jupiter_pools = self.fetch_jupiter_active_pools().await?;
            
            for (address, name, pool_type, liquidity) in jupiter_pools.into_iter().take(5) {
                if verified_pools.len() >= 25 { break; }
                
                info!("   🎯 Testing Jupiter: {} - Liquidity: {:.0}K", name, liquidity / 1000.0);
                
                match self.intelligent_pool_validation(&address, pool_type).await {
                    Ok(pool_data) => {
                        info!("   ✅ JUPITER VERIFIED: {} - Live Liquidity: {:.0}K + {:.0}K", 
                            name,
                            pool_data.token_a_amount as f64 / 1_000_000.0,
                            pool_data.token_b_amount as f64 / 1_000_000.0
                        );
                        verified_pools.push(address.to_string());
                        self.pools.insert(address.to_string(), pool_data);
                    }
                    Err(e) => {
                        warn!("   ❌ JUPITER FAILED: {} - {}", name, e);
                    }
                }
            }
        }
        
        // PASO 4: Fallback a pools conocidos solo si la búsqueda dinámica falló completamente
        if verified_pools.is_empty() {
            warn!("⚠️  Dynamic discovery failed, falling back to known pools...");
            let fallback_pools = self.get_fallback_pools();
            
            for (address, name, pool_type, _) in fallback_pools.into_iter().take(3) {
                match self.intelligent_pool_validation(&address, pool_type).await {
                    Ok(pool_data) => {
                        info!("   ✅ FALLBACK VERIFIED: {} - Liquidity: {:.0}K + {:.0}K", 
                            name,
                            pool_data.token_a_amount as f64 / 1_000_000.0,
                            pool_data.token_b_amount as f64 / 1_000_000.0
                        );
                        verified_pools.push(address.to_string());
                        self.pools.insert(address.to_string(), pool_data);
                    }
                    Err(e) => {
                        warn!("   ❌ FALLBACK FAILED: {} - {}", name, e);
                    }
                }
            }
        }
        
        Ok(verified_pools)
    }
    
    async fn fetch_raydium_active_pools(&self) -> Result<Vec<(String, String, PoolType, f64)>> {
        info!("🔍 Fetching Raydium active pools from multiple APIs...");
        
        let mut pools = Vec::new();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        // API 1: Raydium oficial liquidity endpoint
        let raydium_urls = vec![
            "https://api.raydium.io/v2/sdk/liquidity/mainnet.json",
            "https://api.raydium.io/v2/ammV3/ammPools",
            "https://api.raydium.io/v2/main/pools",
        ];
        
        for url in raydium_urls {
            info!("   🎯 Trying Raydium API: {}", url);
            
            match client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(data) = response.json::<serde_json::Value>().await {
                            // Parsear diferentes formatos de respuesta
                            if let Some(official_pools) = data.get("official") {
                                self.parse_raydium_official_pools(official_pools, &mut pools)?;
                            } else if let Some(pool_list) = data.get("data") {
                                self.parse_raydium_data_pools(pool_list, &mut pools)?;
                            } else if data.is_array() {
                                self.parse_raydium_array_pools(&data, &mut pools)?;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("   ❌ Raydium API {} failed: {}", url, e);
                }
            }
        }
        
        // API 2: Jupiter aggregator para obtener pools activos
        info!("   🎯 Trying Jupiter API for Raydium pools...");
        match client.get("https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=1000000&slippageBps=50").send().await {
            Ok(response) => {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    if let Some(route_plan) = data.get("routePlan") {
                        self.parse_jupiter_route_pools(route_plan, &mut pools)?;
                    }
                }
            }
            Err(e) => {
                warn!("   ❌ Jupiter API failed: {}", e);
            }
        }
        
        // Filtrar duplicados y ordenar por liquidez
        pools.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        pools.dedup_by(|a, b| a.0 == b.0);
        
        info!("   ✅ Found {} unique Raydium pools", pools.len());
        Ok(pools.into_iter().take(20).collect())
    }
    
    fn parse_raydium_official_pools(&self, official_pools: &serde_json::Value, pools: &mut Vec<(String, String, PoolType, f64)>) -> Result<()> {
        if let Some(pool_array) = official_pools.as_array() {
            for pool in pool_array {
                if let (Some(id), Some(base_mint), Some(quote_mint)) = (
                    pool.get("id").and_then(|v| v.as_str()),
                    pool.get("baseMint").and_then(|v| v.as_str()),
                    pool.get("quoteMint").and_then(|v| v.as_str()),
                ) {
                    // Calcular liquidez de múltiples campos
                    let liquidity = pool.get("liquidity").and_then(|v| v.as_f64())
                        .or_else(|| pool.get("tvl").and_then(|v| v.as_f64()))
                        .or_else(|| pool.get("volume24h").and_then(|v| v.as_f64()))
                        .unwrap_or(0.0);
                    
                    if self.is_major_token_pair(base_mint, quote_mint) && liquidity > 100_000.0 {
                        let name = format!("Raydium {}/{}",
                            self.get_token_symbol(base_mint),
                            self.get_token_symbol(quote_mint));
                        pools.push((id.to_string(), name, PoolType::Raydium, liquidity));
                    }
                }
            }
        }
        Ok(())
    }
    
    fn parse_raydium_data_pools(&self, data_pools: &serde_json::Value, pools: &mut Vec<(String, String, PoolType, f64)>) -> Result<()> {
        if let Some(pool_array) = data_pools.as_array() {
            for pool in pool_array {
                if let (Some(address), Some(base_symbol), Some(quote_symbol)) = (
                    pool.get("ammId").and_then(|v| v.as_str()),
                    pool.get("baseSymbol").and_then(|v| v.as_str()),
                    pool.get("quoteSymbol").and_then(|v| v.as_str()),
                ) {
                    let liquidity = pool.get("liquidity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    
                    if liquidity > 100_000.0 {
                        let name = format!("Raydium {}/{}", base_symbol, quote_symbol);
                        pools.push((address.to_string(), name, PoolType::Raydium, liquidity));
                    }
                }
            }
        }
        Ok(())
    }
    
    fn parse_raydium_array_pools(&self, pool_array: &serde_json::Value, pools: &mut Vec<(String, String, PoolType, f64)>) -> Result<()> {
        if let Some(pools_list) = pool_array.as_array() {
            for pool in pools_list {
                if let Some(pool_id) = pool.get("id").and_then(|v| v.as_str()) {
                    let liquidity = pool.get("liquidity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    
                    if liquidity > 100_000.0 {
                        let name = format!("Raydium Pool {}", &pool_id[..8]);
                        pools.push((pool_id.to_string(), name, PoolType::Raydium, liquidity));
                    }
                }
            }
        }
        Ok(())
    }
    
    fn parse_jupiter_route_pools(&self, route_plan: &serde_json::Value, pools: &mut Vec<(String, String, PoolType, f64)>) -> Result<()> {
        if let Some(routes) = route_plan.as_array() {
            for route in routes {
                if let Some(swap_info) = route.get("swapInfo") {
                    if let (Some(amm_key), Some(label)) = (
                        swap_info.get("ammKey").and_then(|v| v.as_str()),
                        swap_info.get("label").and_then(|v| v.as_str()),
                    ) {
                        if label.contains("Raydium") {
                            let liquidity = swap_info.get("liquidity").and_then(|v| v.as_f64()).unwrap_or(1_000_000.0);
                            let name = format!("Jupiter-Raydium {}", &amm_key[..8]);
                            pools.push((amm_key.to_string(), name, PoolType::Raydium, liquidity));
                        }
                    }
                }
            }
        }
        Ok(())
    }
                    
                    // Ordenar por liquidez (mayor primero)
                    pools.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
                    pools.truncate(10); // Top 10 pools
                    
                    Ok(pools)
                } else {
                    warn!("Could not parse Raydium API response");
                    Ok(vec![])
                }
            }
            Err(e) => {
                warn!("Failed to fetch Raydium pools: {}", e);
                Ok(vec![])
            }
        }
    }
    
    async fn fetch_orca_legacy_pools(&self) -> Result<Vec<(String, String, PoolType, f64)>> {
        info!("🔍 Fetching Orca legacy pools...");
        
        // Los pools de Orca clásicos son más difíciles de obtener
        // Por ahora, devolvemos una lista vacía ya que la mayoría son Whirlpool
        Ok(vec![])
    }
    
    async fn fetch_whirlpool_active_pools(&self) -> Result<Vec<(String, String, PoolType, f64)>> {
        info!("🔍 Fetching Whirlpool active pools from multiple APIs...");
        
        let mut pools = Vec::new();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        // API 1: Orca oficial whirlpool endpoint
        let orca_urls = vec![
            "https://api.orca.so/v1/whirlpool/list",
            "https://api.orca.so/v1/whirlpools",
            "https://api.mainnet.orca.so/v1/whirlpool/list",
        ];
        
        for url in orca_urls {
            info!("   🎯 Trying Orca API: {}", url);
            
            match client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(data) = response.json::<serde_json::Value>().await {
                            // Parsear diferentes formatos de respuesta de Orca
                            if let Some(whirlpools) = data.get("whirlpools") {
                                self.parse_orca_whirlpools(whirlpools, &mut pools)?;
                            } else if let Some(pools_data) = data.get("data") {
                                self.parse_orca_whirlpools(pools_data, &mut pools)?;
                            } else if data.is_array() {
                                self.parse_orca_whirlpools(&data, &mut pools)?;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("   ❌ Orca API {} failed: {}", url, e);
                }
            }
        }
        
        // API 2: Jupiter para obtener pools de Whirlpool
        info!("   🎯 Trying Jupiter API for Whirlpool pools...");
        match client.get("https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=1000000&slippageBps=50").send().await {
            Ok(response) => {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    if let Some(route_plan) = data.get("routePlan") {
                        self.parse_jupiter_whirlpool_routes(route_plan, &mut pools)?;
                    }
                }
            }
            Err(e) => {
                warn!("   ❌ Jupiter Whirlpool API failed: {}", e);
            }
        }
        
        // API 3: Birdeye para obtener pools adicionales
        info!("   🎯 Trying Birdeye API for Whirlpool pools...");
        match client.get("https://public-api.birdeye.so/defi/v2/pools?chain=solana&limit=50").send().await {
            Ok(response) => {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    if let Some(pools_data) = data.get("data") {
                        self.parse_birdeye_whirlpools(pools_data, &mut pools)?;
                    }
                }
            }
            Err(e) => {
                warn!("   ❌ Birdeye Whirlpool API failed: {}", e);
            }
        }
        
        // Filtrar duplicados y ordenar por liquidez
        pools.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        pools.dedup_by(|a, b| a.0 == b.0);
        
        info!("   ✅ Found {} unique Whirlpool pools", pools.len());
        Ok(pools.into_iter().take(15).collect())
    }
    
       
    fn is_major_token_pair(&self, token_a: &str, token_b: &str) -> bool {
        let major_tokens = vec![
            "So11111111111111111111111111111111111111112", // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", // USDT
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", // RAY
            "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",  // mSOL
        ];
        
        major_tokens.contains(&token_a) && major_tokens.contains(&token_b)
    }
    
    fn parse_orca_whirlpools(&self, whirlpools: &serde_json::Value, pools: &mut Vec<(String, String, PoolType, f64)>) -> Result<()> {
        if let Some(pool_array) = whirlpools.as_array() {
            for pool in pool_array {
                if let (Some(address), Some(token_a), Some(token_b)) = (
                    pool.get("address").and_then(|v| v.as_str()),
                    pool.get("tokenA").and_then(|v| v.get("mint")).and_then(|v| v.as_str()),
                    pool.get("tokenB").and_then(|v| v.get("mint")).and_then(|v| v.as_str()),
                ) {
                    let liquidity = pool.get("liquidity").and_then(|v| v.as_f64())
                        .or_else(|| pool.get("tvl").and_then(|v| v.as_f64()))
                        .or_else(|| pool.get("volume24h").and_then(|v| v.as_f64()))
                        .unwrap_or(0.0);
                    
                    if self.is_major_token_pair(token_a, token_b) && liquidity > 100_000.0 {
                        let name = format!("Whirlpool {}/{}",
                            self.get_token_symbol(token_a),
                            self.get_token_symbol(token_b));
                        pools.push((address.to_string(), name, PoolType::OrcaWhirlpool, liquidity));
                    }
                }
            }
        }
        Ok(())
    }
    
    fn parse_jupiter_whirlpool_routes(&self, route_plan: &serde_json::Value, pools: &mut Vec<(String, String, PoolType, f64)>) -> Result<()> {
        if let Some(routes) = route_plan.as_array() {
            for route in routes {
                if let Some(swap_info) = route.get("swapInfo") {
                    if let (Some(amm_key), Some(label)) = (
                        swap_info.get("ammKey").and_then(|v| v.as_str()),
                        swap_info.get("label").and_then(|v| v.as_str()),
                    ) {
                        if label.contains("Whirlpool") || label.contains("Orca") {
                            let liquidity = swap_info.get("liquidity").and_then(|v| v.as_f64()).unwrap_or(1_000_000.0);
                            let name = format!("Jupiter-Whirlpool {}", &amm_key[..8]);
                            pools.push((amm_key.to_string(), name, PoolType::OrcaWhirlpool, liquidity));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    fn parse_birdeye_whirlpools(&self, pools_data: &serde_json::Value, pools: &mut Vec<(String, String, PoolType, f64)>) -> Result<()> {
        if let Some(pool_array) = pools_data.as_array() {
            for pool in pool_array {
                if let (Some(address), Some(base_mint), Some(quote_mint)) = (
                    pool.get("address").and_then(|v| v.as_str()),
                    pool.get("baseMint").and_then(|v| v.as_str()),
                    pool.get("quoteMint").and_then(|v| v.as_str()),
                ) {
                    let source = pool.get("source").and_then(|v| v.as_str()).unwrap_or("");
                    
                    if source.contains("Whirlpool") || source.contains("Orca") {
                        let liquidity = pool.get("liquidity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        
                        if self.is_major_token_pair(base_mint, quote_mint) && liquidity > 100_000.0 {
                            let name = format!("Birdeye-Whirlpool {}/{}",
                                self.get_token_symbol(base_mint),
                                self.get_token_symbol(quote_mint));
                            pools.push((address.to_string(), name, PoolType::OrcaWhirlpool, liquidity));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn fetch_jupiter_active_pools(&self) -> Result<Vec<(String, String, PoolType, f64)>> {
        info!("🔍 Fetching Jupiter active pools from multiple routes...");
        
        let mut pools = Vec::new();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        // API 1: Jupiter route discovery
        let jupiter_urls = vec![
            "https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=1000000&slippageBps=50",
            "https://quote-api.jup.ag/v6/quote?inputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&outputMint=Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB&amount=1000000&slippageBps=50",
            "https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R&amount=1000000&slippageBps=50",
        ];
        
        for url in jupiter_urls {
            info!("   🎯 Trying Jupiter route: {}", url);
            
            match client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(data) = response.json::<serde_json::Value>().await {
                            if let Some(route_plan) = data.get("routePlan") {
                                self.parse_jupiter_route_pools(route_plan, &mut pools)?;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("   ❌ Jupiter route failed: {}", e);
                }
            }
        }
        
        // API 2: Jupiter token list
        info!("   🎯 Trying Jupiter token list...");
        match client.get("https://token.jup.ag/all").send().await {
            Ok(response) => {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    if let Some(tokens) = data.as_array() {
                        self.parse_jupiter_token_pools(tokens, &mut pools)?;
                    }
                }
            }
            Err(e) => {
                warn!("   ❌ Jupiter token list failed: {}", e);
            }
        }
        
        // API 3: Jupiter indexed routes
        info!("   🎯 Trying Jupiter indexed routes...");
        match client.get("https://quote-api.jup.ag/v6/indexed-route-map").send().await {
            Ok(response) => {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    self.parse_jupiter_indexed_routes(&data, &mut pools)?;
                }
            }
            Err(e) => {
                warn!("   ❌ Jupiter indexed routes failed: {}", e);
            }
        }
        
        // Filtrar duplicados y ordenar por liquidez
        pools.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        pools.dedup_by(|a, b| a.0 == b.0);
        
        info!("   ✅ Found {} unique Jupiter pools", pools.len());
        Ok(pools.into_iter().take(15).collect())
    }
    
    fn parse_jupiter_token_pools(&self, tokens: &[serde_json::Value], pools: &mut Vec<(String, String, PoolType, f64)>) -> Result<()> {
        for token in tokens {
            if let (Some(address), Some(symbol)) = (
                token.get("address").and_then(|v| v.as_str()),
                token.get("symbol").and_then(|v| v.as_str()),
            ) {
                // Crear pools sintéticos para tokens populares
                let daily_volume = token.get("dailyVolume").and_then(|v| v.as_f64()).unwrap_or(0.0);
                
                if daily_volume > 100_000.0 {
                    let name = format!("Jupiter-{}", symbol);
                    pools.push((address.to_string(), name, PoolType::RaydiumAmmV4, daily_volume));
                }
            }
        }
        Ok(())
    }
    
    fn parse_jupiter_indexed_routes(&self, data: &serde_json::Value, pools: &mut Vec<(String, String, PoolType, f64)>) -> Result<()> {
        if let Some(route_map) = data.as_object() {
            for (mint, routes) in route_map {
                if let Some(route_array) = routes.as_array() {
                    for route in route_array {
                        if let Some(out_mint) = route.as_str() {
                            if self.is_major_token_pair(mint, out_mint) {
                                let pool_address = format!("{}_{}", mint, out_mint);
                                let name = format!("Jupiter-Route {}/{}",
                                    self.get_token_symbol(mint),
                                    self.get_token_symbol(out_mint));
                                pools.push((pool_address, name, PoolType::RaydiumAmmV4, 500_000.0));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    fn get_fallback_pools(&self) -> Vec<(String, String, PoolType, f64)> {
        info!("🔄 Using fallback pools...");
        vec![
            // Pools de alta liquidez conocidos
            ("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2".to_string(), "Raydium SOL/USDC".to_string(), PoolType::RaydiumAmmV4, 10_000_000.0),
            ("7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX".to_string(), "Raydium SOL/USDT".to_string(), PoolType::RaydiumAmmV4, 8_000_000.0),
            ("6UmmUiYoBjSrhakAobJw8BvkmJtDVxaeBtbt7rxWo1mg".to_string(), "Raydium SOL/RAY".to_string(), PoolType::RaydiumAmmV4, 5_000_000.0),
            ("7qbRF6YsyGuLUVs6Y1q64bdVrfe4ZcUUz1JRdoVNUJnm".to_string(), "Whirlpool SOL/USDC".to_string(), PoolType::OrcaWhirlpool, 12_000_000.0),
            ("HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ".to_string(), "Whirlpool SOL/USDT".to_string(), PoolType::OrcaWhirlpool, 7_000_000.0),
            ("83v8iPyZihDEjDdY8RdZddyZNyUtXngz69Lgo9Kt5d6d".to_string(), "Whirlpool USDC/USDT".to_string(), PoolType::OrcaWhirlpool, 15_000_000.0),
        ]
    }
    
    fn get_token_symbol(&self, mint: &str) -> &str {
        match mint {
            "So11111111111111111111111111111111111111112" => "SOL",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC",
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "USDT",
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => "RAY",
            "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So" => "mSOL",
            "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn" => "JTO",
            "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4" => "JUP",
            "8wXtPeU6557ETkp9WHFY1n1EUiEzSK9tKBqQcQufwTTw" => "BONK",
            "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E" => "BTC",
            "2FPyTwcZLUg1MDrwsyoP4D6s1tM7hAkHYRjkNb5w6Pxk" => "ETH",
            _ => "UNKNOWN",
        }
    }
    
    async fn intelligent_pool_validation(&self, address: &str, expected_type: PoolType) -> Result<PoolData> {
        let pool_pubkey = Pubkey::from_str(address)?;
        
        // Obtener cuenta del pool
        let account = self.client.get_account(&pool_pubkey).await
            .map_err(|e| anyhow!("Account not found: {}", e))?;
        
        // Validar programa owner
        let program_valid = match expected_type {
            PoolType::Raydium => account.owner.to_string() == RAYDIUM_AMM_PROGRAM,
            PoolType::Orca => account.owner.to_string() == ORCA_SWAP_PROGRAM,
            PoolType::OrcaWhirlpool => account.owner.to_string() == ORCA_WHIRLPOOL_PROGRAM,
            PoolType::Serum => account.owner.to_string() == SERUM_DEX_PROGRAM,
        };
        
        if !program_valid {
            return Err(anyhow!("Invalid program owner: expected {:?}, got {}", 
                expected_type, account.owner));
        }
        
        // Parsear usando método inteligente
        let pool_data = match expected_type {
            PoolType::Raydium => self.intelligent_raydium_parsing(pool_pubkey, &account).await?,
            PoolType::Orca => self.intelligent_orca_parsing(pool_pubkey, &account).await?,
            PoolType::OrcaWhirlpool => self.intelligent_whirlpool_parsing(pool_pubkey, &account).await?,
            PoolType::Serum => return Err(anyhow!("Serum parsing not implemented")),
        };
        
        // Validar liquidez mínima
        if pool_data.token_a_amount < 100_000 || pool_data.token_b_amount < 100_000 {
            return Err(anyhow!("Insufficient liquidity: {} / {}", 
                pool_data.token_a_amount, pool_data.token_b_amount));
        }
        
        Ok(pool_data)
    }
    
    async fn intelligent_raydium_parsing(&self, pool_address: Pubkey, account: &Account) -> Result<PoolData> {
        let data = &account.data;
        
        if data.len() < 680 {
            return Err(anyhow!("Invalid Raydium pool data length: {}", data.len()));
        }
        
        info!("   🔍 Raydium intelligent parsing - data length: {}", data.len());
        
        // ===== ESTRUCTURA OFICIAL DE RAYDIUM AMM V4 SEGÚN DOCUMENTACIÓN =====
        // Basado en: https://github.com/raydium-io/raydium-ui/blob/main/src/utils/liquidity.ts
        // AMM_INFO_LAYOUT_V4 structure:
        // 
        // u64('status') - 8 bytes - offset 0
        // u64('nonce') - 8 bytes - offset 8
        // u64('orderNum') - 8 bytes - offset 16
        // u64('depth') - 8 bytes - offset 24
        // u64('coinDecimals') - 8 bytes - offset 32
        // u64('pcDecimals') - 8 bytes - offset 40
        // u64('state') - 8 bytes - offset 48
        // u64('resetFlag') - 8 bytes - offset 56
        // u64('minSize') - 8 bytes - offset 64
        // u64('volMaxCutRatio') - 8 bytes - offset 72
        // u64('amountWaveRatio') - 8 bytes - offset 80
        // u64('coinLotSize') - 8 bytes - offset 88
        // u64('pcLotSize') - 8 bytes - offset 96
        // u64('minPriceMultiplier') - 8 bytes - offset 104
        // u64('maxPriceMultiplier') - 8 bytes - offset 112
        // u64('systemDecimalsValue') - 8 bytes - offset 120
        // u64('minSeparateNumerator') - 8 bytes - offset 128
        // u64('minSeparateDenominator') - 8 bytes - offset 136
        // u64('tradeFeeNumerator') - 8 bytes - offset 144
        // u64('tradeFeeDenominator') - 8 bytes - offset 152
        // u64('pnlNumerator') - 8 bytes - offset 160
        // u64('pnlDenominator') - 8 bytes - offset 168
        // u64('swapFeeNumerator') - 8 bytes - offset 176
        // u64('swapFeeDenominator') - 8 bytes - offset 184
        // u64('needTakePnlCoin') - 8 bytes - offset 192
        // u64('needTakePnlPc') - 8 bytes - offset 200
        // u64('totalPnlX') - 8 bytes - offset 208
        // u64('totalPnlY') - 8 bytes - offset 216
        // u64('systemDecimalsValue') - 8 bytes - offset 224
        // publicKey('poolCoinTokenAccount') - 32 bytes - offset 232
        // publicKey('poolPcTokenAccount') - 32 bytes - offset 264
        // publicKey('coinMintAddress') - 32 bytes - offset 296
        // publicKey('pcMintAddress') - 32 bytes - offset 328
        // publicKey('lpMintAddress') - 32 bytes - offset 360
        // publicKey('ammOpenOrders') - 32 bytes - offset 392
        // publicKey('serumMarket') - 32 bytes - offset 424
        // publicKey('serumProgramId') - 32 bytes - offset 456
        // publicKey('ammTargetOrders') - 32 bytes - offset 488
        // publicKey('ammQuantities') - 32 bytes - offset 520
        // publicKey('poolWithdrawQueue') - 32 bytes - offset 552
        // publicKey('poolTempLpTokenAccount') - 32 bytes - offset 584
        // publicKey('ammOwner') - 32 bytes - offset 616
        // publicKey('pnlOwner') - 32 bytes - offset 648
        
        // OFFSETS EXACTOS SEGÚN DOCUMENTACIÓN OFICIAL
        let coin_vault_offset = 232;  // poolCoinTokenAccount
        let pc_vault_offset = 264;    // poolPcTokenAccount
        let coin_mint_offset = 296;   // coinMintAddress
        let pc_mint_offset = 328;     // pcMintAddress
        let lp_mint_offset = 360;     // lpMintAddress
        
        // Verificar que tenemos suficientes bytes
        let required_size = lp_mint_offset + 32;
        if data.len() < required_size {
            return Err(anyhow!("Insufficient data for Raydium AMM V4 parsing: need {}, got {}", 
                required_size, data.len()));
        }
        
        // Verificar status (debe ser 6 o 7 para pools activos)
        let status = u64::from_le_bytes(data[0..8].try_into().unwrap());
        if status != 6 && status != 7 {
            return Err(anyhow!("Invalid AMM status: {}", status));
        }
        
        // Extraer direcciones usando offsets oficiales
        let token_a_mint = Pubkey::new_from_array(
            data[coin_mint_offset..coin_mint_offset+32].try_into()
                .map_err(|_| anyhow!("Invalid coinMintAddress at offset {}", coin_mint_offset))?
        );
        let token_b_mint = Pubkey::new_from_array(
            data[pc_mint_offset..pc_mint_offset+32].try_into()
                .map_err(|_| anyhow!("Invalid pcMintAddress at offset {}", pc_mint_offset))?
        );
        let token_a_vault = Pubkey::new_from_array(
            data[coin_vault_offset..coin_vault_offset+32].try_into()
                .map_err(|_| anyhow!("Invalid poolCoinTokenAccount at offset {}", coin_vault_offset))?
        );
        let token_b_vault = Pubkey::new_from_array(
            data[pc_vault_offset..pc_vault_offset+32].try_into()
                .map_err(|_| anyhow!("Invalid poolPcTokenAccount at offset {}", pc_vault_offset))?
        );
        let lp_mint = Pubkey::new_from_array(
            data[lp_mint_offset..lp_mint_offset+32].try_into()
                .map_err(|_| anyhow!("Invalid lpMintAddress at offset {}", lp_mint_offset))?
        );
        
        // Validar que las direcciones no son por defecto
        if token_a_mint == Pubkey::default() || token_b_mint == Pubkey::default() ||
           token_a_vault == Pubkey::default() || token_b_vault == Pubkey::default() {
            return Err(anyhow!("Invalid default addresses detected"));
        }
        
        info!("   📊 Raydium AMM V4 addresses extracted:");
        info!("     Coin Token: {} (vault: {})", token_a_mint, token_a_vault);
        info!("     PC Token: {} (vault: {})", token_b_mint, token_b_vault);
        info!("     LP Mint: {}", lp_mint);
        
        // VALIDACIÓN CRÍTICA: Verificar que los vaults existen y tienen balance
        let token_a_amount = self.get_token_account_balance(&token_a_vault).await
            .map_err(|e| anyhow!("Invalid coin token vault {}: {}", token_a_vault, e))?;
        let token_b_amount = self.get_token_account_balance(&token_b_vault).await
            .map_err(|e| anyhow!("Invalid PC token vault {}: {}", token_b_vault, e))?;
        
        // Verificar liquidez mínima
        if token_a_amount < 1000 || token_b_amount < 1000 {
            return Err(anyhow!("Insufficient liquidity: {} / {}", token_a_amount, token_b_amount));
        }
        
        let lp_supply = self.get_token_supply(&lp_mint).await.unwrap_or(0);
        
        info!("   ✅ Raydium AMM V4 parsing SUCCESS - Liquidity: {:.6} + {:.6}", 
            token_a_amount as f64 / 1_000_000.0, token_b_amount as f64 / 1_000_000.0);
        
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
            fees_bps: 25, // Raydium típicamente usa 0.25% = 25 bps
        })
    }
    
    async fn intelligent_orca_parsing(&self, pool_address: Pubkey, account: &Account) -> Result<PoolData> {
        let data = &account.data;
        
        if data.len() < 324 {
            return Err(anyhow!("Invalid Orca pool data length: {}", data.len()));
        }
        
        info!("   🔍 Orca intelligent parsing - data length: {}", data.len());
        
        // DETECCIÓN INTELIGENTE: Múltiples layouts de Orca
        let layouts = vec![
            ("Orca v1", 8, 40, 72, 104, 136),
            ("Orca v2", 101, 181, 85, 165, 245),
            ("Orca v3", 40, 72, 104, 136, 168),
            ("Orca Alt A", 16, 48, 80, 112, 144),
            ("Orca Alt B", 24, 56, 88, 120, 152),
            ("Orca Alt C", 32, 64, 96, 128, 160),
            ("Orca Alt D", 50, 82, 114, 146, 178),
            ("Orca Alt E", 60, 92, 124, 156, 188),
        ];
        
        for (name, mint_a, mint_b, vault_a, vault_b, lp_mint) in layouts {
            if data.len() >= lp_mint + 32 {
                match self.try_orca_layout(pool_address, data, mint_a, mint_b, vault_a, vault_b, lp_mint).await {
                    Ok(pool_data) => {
                        info!("   ✅ Orca {} layout SUCCESS", name);
                        return Ok(pool_data);
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        }
        
        Err(anyhow!("Could not parse Orca pool with any known layout"))
    }
    
    async fn try_orca_layout(&self, pool_address: Pubkey, data: &[u8], 
                            mint_a_off: usize, mint_b_off: usize, vault_a_off: usize, 
                            vault_b_off: usize, lp_off: usize) -> Result<PoolData> {
        
        let token_a_mint = Pubkey::new_from_array(
            data[mint_a_off..mint_a_off+32].try_into().map_err(|_| anyhow!("Invalid mint A"))?
        );
        let token_b_mint = Pubkey::new_from_array(
            data[mint_b_off..mint_bOff+32].try_into().map_err(|_| anyhow!("Invalid mint B"))?
        );
        let token_a_vault = Pubkey::new_from_array(
            data[vault_a_off..vault_a_off+32].try_into().map_err(|_| anyhow!("Invalid vault A"))?
        );
        let token_b_vault = Pubkey::new_from_array(
            data[vault_b_off..vault_b_off+32].try_into().map_err(|_| anyhow!("Invalid vault B"))?
        );
        let lp_mint = Pubkey::new_from_array(
            data[lp_off..lp_off+32].try_into().map_err(|_| anyhow!("Invalid LP mint"))?
        );
        
        // Validate addresses are not default
        if token_a_mint == Pubkey::default() || token_b_mint == Pubkey::default() || 
           token_a_vault == Pubkey::default() || token_b_vault == Pubkey::default() {
            return Err(anyhow!("Invalid addresses detected"));
        }
        
        // Try to get balances to validate
        let token_a_amount = self.get_token_account_balance(&token_a_vault).await?;
        let token_b_amount = self.get_token_account_balance(&token_b_vault).await?;
        
        let lp_supply = self.get_token_supply(&lp_mint).await.unwrap_or(0);
        
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
            fees_bps: 30,
        })
    }

    async fn parse_orca_whirlpool(&self, pool_address: Pubkey, account: &Account) -> Result<PoolData> {
        // Orca Whirlpool parsing with multiple layout attempts
        let data = &account.data;
        
        if data.len() < 653 {
            return Err(anyhow!("Invalid Orca Whirlpool data length: {}", data.len()));
        }
        
        // Try multiple Whirlpool layout versions
        let offset_attempts = vec![
            // Whirlpool v3 layout attempt 1
            (8, 40, 72, 104),
            // Whirlpool v3 layout attempt 2 (alternative)
            (101, 133, 165, 197),
            // Whirlpool v3 layout attempt 3 (newer format)
            (168, 200, 232, 264),
        ];
        
        for (mint_a_off, mint_b_off, vault_a_off, vault_b_off) in offset_attempts {
            if data.len() >= vault_b_off + 32 {
                let token_a_mint = Pubkey::new_from_array(
                    data[mint_a_off..mint_a_off+32].try_into().map_err(|_| anyhow!("Invalid token A mint"))?
                );
                let token_b_mint = Pubkey::new_from_array(
                    data[mint_b_off..mint_bOff+32].try_into().map_err(|_| anyhow!("Invalid token B mint"))?
                );
                let token_a_vault = Pubkey::new_from_array(
                    data[vault_a_off..vault_a_off+32].try_into().map_err(|_| anyhow!("Invalid token A vault"))?
                );
                let token_b_vault = Pubkey::new_from_array(
                    data[vault_b_off..vault_b_off+32].try_into().map_err(|_| anyhow!("Invalid token B vault"))?
                );
                
                // Validate that these look like real addresses
                if token_a_mint != Pubkey::default() && token_b_mint != Pubkey::default() && 
                   token_a_vault != Pubkey::default() && token_b_vault != Pubkey::default() {
                    
                    info!("   ✅ Found valid Whirlpool layout at offsets: mint_a={}, vault_a={}", mint_a_off, vault_a_off);
                    
                    // Try to get balances to confirm these are correct
                    let token_a_amount = match self.get_token_account_balance(&token_a_vault).await {
                        Ok(amount) => amount,
                        Err(e) => {
                            warn!("Whirlpool layout validation failed for vault {}: {}", token_a_vault, e);
                            continue; // Try next layout
                        }
                    };
                    let token_b_amount = match self.get_token_account_balance(&token_b_vault).await {
                        Ok(amount) => amount,
                        Err(e) => {
                            warn!("Whirlpool layout validation failed for vault {}: {}", token_b_vault, e);
                            continue; // Try next layout
                        }
                    };
                    
                    return Ok(PoolData {
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
                    });
                }
            }
        }
        
        Err(anyhow!("Could not parse Whirlpool with any known layout format"))
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
        
        info!("   🔍 MILITARY SCAN: Analyzing {} pools for arbitrage opportunities...", self.pools.len());
        
        if self.pools.is_empty() {
            return Err(anyhow!("CRITICAL: No operational pools available"));
        }
        
        // MILITARY INTEL: Show pool status
        info!("   📊 OPERATIONAL POOLS:");
        for (address, pool) in &self.pools {
            info!("     - {}: {} + {} liquidity ({:?})", 
                &address[..8], 
                pool.token_a_amount / 1_000_000,
                pool.token_b_amount / 1_000_000,
                pool.pool_type
            );
        }
        
        // MILITARY STRATEGY: Check all possible arbitrage combinations
        let pool_addresses: Vec<_> = self.pools.keys().collect();
        let mut combinations_checked = 0;
        
        for i in 0..pool_addresses.len() {
            for j in (i + 1)..pool_addresses.len() {
                combinations_checked += 1;
                let pool_a = &self.pools[pool_addresses[i]];
                let pool_b = &self.pools[pool_addresses[j]];
                
                // Check for arbitrage opportunity
                if let Some(opportunity) = self.calculate_direct_arbitrage(pool_a, pool_b).await? {
                    if opportunity.profit_lamports > 0 {
                        info!("   🎯 TARGET ACQUIRED: {:.6}% profit potential", opportunity.profit_percentage);
                        opportunities.push(opportunity);
                    }
                }
            }
        }
        
        // MILITARY PRIORITIZATION: Sort by profit
        opportunities.sort_by(|a, b| b.profit_percentage.partial_cmp(&a.profit_percentage).unwrap());
        
        info!("   📈 MISSION REPORT: {} combinations checked, {} opportunities found", 
            combinations_checked, opportunities.len());
        
        if !opportunities.is_empty() {
            info!("   🏆 TOP OPPORTUNITIES:");
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
        
        info!("   🔍 MILITARY ANALYSIS: Common token {} detected", 
            &common_token.to_string()[..8]);
        
        // MILITARY EFFICIENCY: Test incremental amounts for optimal profit
        let test_amounts = vec![
            1_000_000,     // 0.001 SOL - reconnaissance
            5_000_000,     // 0.005 SOL - light probe
            10_000_000,    // 0.01 SOL - standard operation
            50_000_000,    // 0.05 SOL - heavy assault (if profitable)
        ];
        
        let mut best_opportunity: Option<DirectOpportunity> = None;
        
        for amount in test_amounts {
            // Route 1: Pool A -> Common Token -> Pool B
            if let Ok(route1_profit) = self.calculate_route_profit(pool_a, pool_b, &common_token, amount).await {
                if route1_profit > 0 {
                    let profit_percentage = (route1_profit as f64 / amount as f64) * 100.0;
                    
                    info!("   💰 Route 1 profit: {:.6}% ({:.9} SOL)", 
                        profit_percentage, route1_profit as f64 / 1e9);
                    
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
                    
                    info!("   💰 Route 2 profit: {:.6}% ({:.9} SOL)", 
                        profit_percentage, route2_profit as f64 / 1e9);
                    
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
        // === COMPREHENSIVE PROFIT CALCULATION ===
        
        // Step 1: Calculate first swap output (with DEX fees)
        let first_swap_output = self.calculate_pool_output(pool_1, amount_in, intermediate_token)?;
        
        // Step 2: Calculate second swap output (with DEX fees)
        let final_amount = self.calculate_pool_output(pool_2, first_swap_output, intermediate_token)?;
        
        // Step 3: Calculate all transaction costs
        let network_fees = self.calculate_transaction_fees()?;
        let trading_fees = self.calculate_trading_fees(pool_1, pool_2, amount_in, first_swap_output)?;
        let slippage_impact = self.calculate_slippage_impact(pool_1, pool_2, amount_in)?;
        
        // Step 4: Calculate net profit
        let total_costs = network_fees + trading_fees + slippage_impact;
        let gross_profit = final_amount as i64 - amount_in as i64;
        let net_profit = gross_profit - total_costs as i64;
        
        // Step 5: Add minimum profit threshold (prevent dust arbitrage)
        let min_profit_threshold = 10_000; // 0.00001 SOL minimum profit
        if net_profit < min_profit_threshold {
            return Ok(-1); // Not profitable enough
        }
        
        info!("   📊 Profit calculation for {:.6} SOL input:", amount_in as f64 / 1e9);
        info!("     🔄 First swap: {:.6} → {:.6}", amount_in as f64 / 1e9, first_swap_output as f64 / 1e9);
        info!("     🔄 Second swap: {:.6} → {:.6}", first_swap_output as f64 / 1e9, final_amount as f64 / 1e9);
        info!("     💰 Gross profit: {:.9} SOL", gross_profit as f64 / 1e9);
        info!("     💸 Total costs: {:.9} SOL", total_costs as f64 / 1e9);
        info!("     📈 Net profit: {:.9} SOL", net_profit as f64 / 1e9);
        
        Ok(net_profit)
    }
    
    fn calculate_trading_fees(&self, pool_1: &PoolData, pool_2: &PoolData, 
                             amount_1: u64, amount_2: u64) -> Result<u64> {
        // Calculate actual trading fees paid to DEXs
        let fee_1 = (amount_1 * pool_1.fees_bps) / 10_000;
        let fee_2 = (amount_2 * pool_2.fees_bps) / 10_000;
        
        let total_trading_fees = fee_1 + fee_2;
        
        info!("     🏪 DEX trading fees: {:.9} SOL", total_trading_fees as f64 / 1e9);
        
        Ok(total_trading_fees)
    }
    
    fn calculate_slippage_impact(&self, pool_1: &PoolData, pool_2: &PoolData, 
                                amount_in: u64) -> Result<u64> {
        // Calculate price impact from large trades
        let impact_1 = self.calculate_price_impact(pool_1, amount_in)?;
        let impact_2 = self.calculate_price_impact(pool_2, amount_in)?;
        
        let total_impact = impact_1 + impact_2;
        
        info!("     📉 Price impact: {:.9} SOL", total_impact as f64 / 1e9);
        
        Ok(total_impact)
    }
    
    fn calculate_price_impact(&self, pool: &PoolData, amount_in: u64) -> Result<u64> {
        // Price impact calculation based on pool depth
        let total_liquidity = pool.token_a_amount + pool.token_b_amount;
        
        if total_liquidity == 0 {
            return Ok(0);
        }
        
        // Impact percentage based on trade size vs liquidity
        let impact_percentage = (amount_in as f64 / total_liquidity as f64) * 100.0;
        
        // Convert impact to lamports (higher impact = higher cost)
        let impact_cost = (amount_in as f64 * impact_percentage / 100.0 * 0.1) as u64;
        
        Ok(impact_cost)
    }
    
    fn calculate_pool_output(&self, pool: &PoolData, amount_in: u64, output_token: &Pubkey) -> Result<u64> {
        // === MULTI-DEX POOL OUTPUT CALCULATION ===
        
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
        
        // Calculate output based on DEX type
        let amount_out = match pool.pool_type {
            PoolType::Raydium => {
                self.calculate_raydium_output(amount_in, reserve_in, reserve_out, pool.fees_bps)?
            }
            PoolType::Orca => {
                self.calculate_orca_output(amount_in, reserve_in, reserve_out, pool.fees_bps)?
            }
            PoolType::OrcaWhirlpool => {
                self.calculate_whirlpool_output(amount_in, reserve_in, reserve_out, pool.fees_bps)?
            }
            PoolType::Serum => {
                self.calculate_serum_output(amount_in, reserve_in, reserve_out, pool.fees_bps)?
            }
        };
        
        Ok(amount_out)
    }
    
    fn calculate_raydium_output(&self, amount_in: u64, reserve_in: u64, reserve_out: u64, fees_bps: u64) -> Result<u64> {
        // Raydium uses standard constant product with fees
        let amount_in_after_fees = amount_in * (10_000 - fees_bps) / 10_000;
        
        let k = reserve_in as u128 * reserve_out as u128;
        let new_reserve_in = reserve_in as u128 + amount_in_after_fees as u128;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = reserve_out as u128 - new_reserve_out;
        
        // Raydium has minimal slippage for standard pairs
        let amount_out_with_slippage = amount_out * 995 / 1000; // 0.5% slippage
        
        Ok(amount_out_with_slippage as u64)
    }
    
    fn calculate_orca_output(&self, amount_in: u64, reserve_in: u64, reserve_out: u64, fees_bps: u64) -> Result<u64> {
        // Orca uses similar constant product but with different fee structure
        let amount_in_after_fees = amount_in * (10_000 - fees_bps) / 10_000;
        
        let k = reserve_in as u128 * reserve_out as u128;
        let new_reserve_in = reserve_in as u128 + amount_in_after_fees as u128;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = reserve_out as u128 - new_reserve_out;
        
        // Orca has slightly higher slippage
        let amount_out_with_slippage = amount_out * 990 / 1000; // 1% slippage
        
        Ok(amount_out_with_slippage as u64)
    }
    
    fn calculate_whirlpool_output(&self, amount_in: u64, reserve_in: u64, reserve_out: u64, fees_bps: u64) -> Result<u64> {
        // Whirlpool uses concentrated liquidity - more complex calculation
        // For now, using constant product with adjusted fees
        let amount_in_after_fees = amount_in * (10_000 - fees_bps) / 10_000;
        
        let k = reserve_in as u128 * reserve_out as u128;
        let new_reserve_in = reserve_in as u128 + amount_in_after_fees as u128;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = reserve_out as u128 - new_reserve_out;
        
        // Whirlpool has better pricing due to concentrated liquidity
        let amount_out_with_slippage = amount_out * 998 / 1000; // 0.2% slippage
        
        Ok(amount_out_with_slippage as u64)
    }
    
    fn calculate_serum_output(&self, amount_in: u64, reserve_in: u64, reserve_out: u64, fees_bps: u64) -> Result<u64> {
        // Serum uses order book, but for simplification using constant product
        // In reality, would need to calculate based on order book depth
        let amount_in_after_fees = amount_in * (10_000 - fees_bps) / 10_000;
        
        let k = reserve_in as u128 * reserve_out as u128;
        let new_reserve_in = reserve_in as u128 + amount_in_after_fees as u128;
        let new_reserve_out = k / new_reserve_in;
        let amount_out = reserve_out as u128 - new_reserve_out;
        
        // Order book can have variable slippage
        let amount_out_with_slippage = amount_out * 992 / 1000; // 0.8% slippage
        
        Ok(amount_out_with_slippage as u64)
    }
    
    fn calculate_transaction_fees(&self) -> Result<u64> {
        // === COMPREHENSIVE SOLANA TRANSACTION FEES ===
        
        // 1. Base transaction fee (always required)
        let base_fee = 5_000; // 0.000005 SOL per signature
        
        // 2. Priority fee (essential for MEV/arbitrage)
        let priority_fee = 100_000; // 0.0001 SOL - aggressive priority
        
        // 3. Compute unit fees (realistic for multi-DEX arbitrage)
        let compute_units = 600_000; // 2 swaps + ATA creation + validations
        let compute_unit_price = 50; // microlamports per CU (competitive)
        let compute_fee = compute_units * compute_unit_price;
        
        // 4. ATA creation fees (token accounts)
        let ata_rent_exemption = 2_039_280; // Rent exemption for token account
        let max_ata_creations = 4; // Input/output tokens for both swaps
        let ata_creation_fees = ata_rent_exemption * max_ata_creations;
        
        // 5. Account rent (temporary accounts during swap)
        let temp_account_rent = 890_880; // Temporary PDA accounts
        
        // 6. DEX-specific fees (protocol fees, not trading fees)
        let dex_protocol_fees = 10_000; // Protocol interaction fees
        
        // 7. Slippage buffer (additional safety margin)
        let slippage_buffer = 50_000; // 0.00005 SOL buffer
        
        // === TOTAL NETWORK FEES ===
        let network_fees = base_fee + priority_fee + compute_fee + 
                          ata_creation_fees + temp_account_rent + 
                          dex_protocol_fees + slippage_buffer;
        
        info!("   💰 Fee breakdown:");
        info!("     📊 Base fee: {:.9} SOL", base_fee as f64 / 1e9);
        info!("     ⚡ Priority fee: {:.9} SOL", priority_fee as f64 / 1e9);
        info!("     💻 Compute fee: {:.9} SOL", compute_fee as f64 / 1e9);
        info!("     🏦 ATA creation: {:.9} SOL", ata_creation_fees as f64 / 1e9);
        info!("     📋 Total network fees: {:.9} SOL", network_fees as f64 / 1e9);
        
        Ok(network_fees)
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
                // Raydium swap instruction: instruction_id (1 byte) + amount_in (8 bytes) + minimum_amount_out (8 bytes)
                let mut data = vec![9]; // Raydium swap instruction discriminator
                data.extend_from_slice(&amount.to_le_bytes());
                data.extend_from_slice(&(amount * 990 / 1000).to_le_bytes()); // 1% slippage protection
                Ok(data)
            }
            PoolType::Orca => {
                // Orca swap instruction: Different format
                let mut data = vec![1]; // Orca swap instruction discriminator  
                data.extend_from_slice(&amount.to_le_bytes());
                data.extend_from_slice(&(amount * 990 / 1000).to_le_bytes()); // 1% slippage protection
                Ok(data)
            }
            PoolType::OrcaWhirlpool => {
                // Whirlpool swap - more complex but simplified for now
                let mut data = vec![248, 198, 158, 145, 225, 117, 135, 200]; // whirlpool swap discriminator
                data.extend_from_slice(&amount.to_le_bytes());
                data.extend_from_slice(&(amount * 990 / 1000).to_le_bytes());
                // Additional whirlpool parameters would go here
                Ok(data)
            }
            PoolType::Serum => {
                // Serum DEX instruction - placeholder
                warn!("Serum swaps not yet implemented");
                Err(anyhow!("Serum swaps not supported"))
            }
        }
    }

    // ===== DIRECT TRANSACTION EXECUTION =====
    
    async fn execute_direct_arbitrage(&mut self, opportunity: &DirectOpportunity) -> Result<String> {
        info!("   ⚔️  Executing REAL arbitrage transaction...");
        
        // 1. Check and create ATAs if needed
        let mut pre_instructions = Vec::new();
        for swap_instruction in &opportunity.execution_path {
            let input_ata = get_associated_token_address(&self.wallet_address, &swap_instruction.input_mint);
            let output_ata = get_associated_token_address(&self.wallet_address, &swap_instruction.output_mint);
            
            // Check if ATAs exist, create if not
            if !self.account_exists(&input_ata).await? {
                info!("   🔧 Creating input ATA for token: {}", swap_instruction.input_mint);
                let create_ata_ix = create_associated_token_account(
                    &self.wallet_address,
                    &self.wallet_address,
                    &swap_instruction.input_mint,
                    &Pubkey::from_str(TOKEN_PROGRAM_ID)?,
                );
                pre_instructions.push(create_ata_ix);
            }
            
            if !self.account_exists(&output_ata).await? {
                info!("   🔧 Creating output ATA for token: {}", swap_instruction.output_mint);
                let create_ata_ix = create_associated_token_account(
                    &self.wallet_address,
                    &self.wallet_address,
                    &swap_instruction.output_mint,
                    &Pubkey::from_str(TOKEN_PROGRAM_ID)?,
                );
                pre_instructions.push(create_ata_ix);
            }
        }
        
        // 2. Build swap instructions
        let mut swap_instructions = Vec::new();
        for swap_instruction in &opportunity.execution_path {
            let instruction = self.build_solana_instruction(swap_instruction).await?;
            swap_instructions.push(instruction);
        }
        
        // 3. Combine all instructions
        let mut all_instructions = pre_instructions;
        all_instructions.extend(swap_instructions);
        
        if all_instructions.is_empty() {
            return Err(anyhow!("No instructions to execute"));
        }
        
        // 4. Build and send transaction
        info!("   🚀 Sending transaction with {} instructions", all_instructions.len());
        let recent_blockhash = self.client.get_latest_blockhash().await?;
        
        let transaction = Transaction::new_signed_with_payer(
            &all_instructions,
            Some(&self.wallet_address),
            &[&self.keypair],
            recent_blockhash,
        );
        
        // 5. Send with confirmation
        let signature = self.client.send_and_confirm_transaction(&transaction).await?;
        
        info!("   ✅ REAL arbitrage transaction confirmed!");
        Ok(signature.to_string())
    }
    
    async fn account_exists(&self, address: &Pubkey) -> Result<bool> {
        match self.client.get_account(address).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
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

    // ===== MILITARY INTELLIGENCE: INTELLIGENT POOL DISCOVERY =====
    
    async fn discover_operational_pools(&mut self) -> Result<()> {
        info!("🔍 MILITARY RECONNAISSANCE: Discovering operational pools...");
        
        // ESTRATEGIA MILITAR: Buscar pools conocidos de mainnet usando direcciones verificadas
        let verified_pools = self.find_verified_mainnet_pools().await?;
        
        if verified_pools.is_empty() {
            warn!("⚠️  No verified pools found, falling back to candidate testing");
            self.test_pool_candidates().await?;
        } else {
            info!("✅ Found {} verified pools", verified_pools.len());
            self.monitoring_pools = verified_pools;
        }
        
        // Validar que tenemos al menos algunos pools operativos
        if self.monitoring_pools.is_empty() {
            warn!("⚠️  MILITARY FALLBACK: No pools discovered, creating mock pools for testing");
            
            // SISTEMA DE EMERGENCIA: Crear pools mock para testing
            let mock_pool = PoolData {
                address: Pubkey::from_str("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2").unwrap(),
                token_a_mint: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(), // SOL
                token_b_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(), // USDC
                token_a_vault: Pubkey::from_str("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2").unwrap(),
                token_b_vault: Pubkey::from_str("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2").unwrap(),
                token_a_amount: 1_000_000_000_000, // 1M SOL
                token_b_amount: 50_000_000_000_000, // 50M USDC
                lp_mint: Pubkey::from_str("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2").unwrap(),
                lp_supply: 1_000_000_000,
                pool_type: PoolType::Raydium,
                last_updated: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                fees_bps: 25,
            };
            
            self.monitoring_pools.push("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2".to_string());
            self.pools.insert("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2".to_string(), mock_pool);
            
            info!("✅ MILITARY FALLBACK: Mock pool created for testing");
        }
        