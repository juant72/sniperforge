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
    account::Account,
};

// 🎯 MILITARY-GRADE POOL MONITOR
// Direct blockchain access, no external dependencies

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🎯 === MILITARY-GRADE POOL MONITOR ===");
    info!("   ⚡ DIRECT BLOCKCHAIN ACCESS");
    info!("   🔥 REAL-TIME POOL ANALYSIS");
    info!("   💰 ZERO API DEPENDENCIES");

    let mut monitor = MilitaryPoolMonitor::new().await?;
    monitor.run_continuous_monitoring().await?;

    Ok(())
}

struct MilitaryPoolMonitor {
    client: RpcClient,
    wallet_address: Pubkey,
    
    // Pool configurations
    monitored_pools: HashMap<String, PoolConfig>,
    live_data: HashMap<String, LivePoolData>,
    
    // Performance metrics
    update_times: Vec<u64>,
    last_arbitrage_check: std::time::Instant,
}

#[derive(Debug, Clone)]
struct PoolConfig {
    address: Pubkey,
    program_id: Pubkey,
    pool_type: PoolType,
    token_a_mint: Pubkey,
    token_b_mint: Pubkey,
    token_a_decimals: u8,
    token_b_decimals: u8,
    fee_rate: f64,
}

#[derive(Debug, Clone)]
enum PoolType {
    RaydiumAMM,
    OrcaWhirlpool,
    SerumDEX,
    Unknown,
}

#[derive(Debug, Clone)]
struct LivePoolData {
    reserve_a: u64,
    reserve_b: u64,
    price_a_to_b: f64,
    price_b_to_a: f64,
    liquidity_usd: f64,
    volume_24h: f64,
    last_update: std::time::Instant,
    update_count: u64,
}

#[derive(Debug, Clone)]
struct ArbitrageAlert {
    pool_pair: (String, String),
    profit_opportunity: f64,
    confidence: f64,
    recommended_amount: u64,
    execution_time_window: Duration,
}

impl MilitaryPoolMonitor {
    async fn new() -> Result<Self> {
        // Load wallet for address reference
        let wallet_path = "mainnet_wallet.json";
        let json_str = std::fs::read_to_string(wallet_path)?;
        let keypair_bytes: Vec<u8> = serde_json::from_str(&json_str)?;
        let keypair = Keypair::from_bytes(&keypair_bytes)?;
        let wallet_address = keypair.pubkey();

        // Ultra-fast RPC connection
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        info!("🎯 Military Monitor initialized: {}", wallet_address);

        let mut monitor = Self {
            client,
            wallet_address,
            monitored_pools: HashMap::new(),
            live_data: HashMap::new(),
            update_times: Vec::new(),
            last_arbitrage_check: std::time::Instant::now(),
        };

        // Initialize pool configurations
        monitor.setup_pool_configurations().await?;

        Ok(monitor)
    }

    async fn setup_pool_configurations(&mut self) -> Result<()> {
        info!("🔧 Setting up pool configurations...");

        // Add major pools for monitoring
        let pools = vec![
            // Raydium SOL/USDC
            ("raydium_sol_usdc", "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2", 
             "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", PoolType::RaydiumAMM),
            
            // Orca SOL/USDC
            ("orca_sol_usdc", "HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ",
             "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc", PoolType::OrcaWhirlpool),
            
            // Raydium RAY/USDC
            ("raydium_ray_usdc", "6UmmUiYoBjSrhakAobJw8BvkmJtDVxaeBtbt7rxWo1mg",
             "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", PoolType::RaydiumAMM),
            
            // Orca mSOL/SOL
            ("orca_msol_sol", "83v8iPyZihDEjDdY8RdZddyZNyUtXngz69Lgo9Kt5d6d",
             "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc", PoolType::OrcaWhirlpool),
        ];

        for (name, pool_addr, program_addr, pool_type) in pools {
            let pool_config = self.analyze_pool_structure(name, pool_addr, program_addr, pool_type).await?;
            self.monitored_pools.insert(name.to_string(), pool_config);
            
            // Initialize live data
            self.live_data.insert(name.to_string(), LivePoolData {
                reserve_a: 0,
                reserve_b: 0,
                price_a_to_b: 0.0,
                price_b_to_a: 0.0,
                liquidity_usd: 0.0,
                volume_24h: 0.0,
                last_update: std::time::Instant::now(),
                update_count: 0,
            });
        }

        info!("✅ {} pools configured for monitoring", self.monitored_pools.len());
        Ok(())
    }

    async fn analyze_pool_structure(&self, name: &str, pool_addr: &str, program_addr: &str, pool_type: PoolType) -> Result<PoolConfig> {
        let pool_pubkey = Pubkey::from_str(pool_addr)?;
        let program_pubkey = Pubkey::from_str(program_addr)?;
        
        // Get pool account to analyze structure
        let account = self.client.get_account(&pool_pubkey).await?;
        
        info!("🔍 Analyzing {} pool structure ({} bytes)", name, account.data.len());
        
        // Parse based on pool type
        let (token_a_mint, token_b_mint, fee_rate) = match pool_type {
            PoolType::RaydiumAMM => self.parse_raydium_structure(&account.data)?,
            PoolType::OrcaWhirlpool => self.parse_orca_structure(&account.data)?,
            _ => return Err(anyhow!("Unsupported pool type")),
        };

        Ok(PoolConfig {
            address: pool_pubkey,
            program_id: program_pubkey,
            pool_type,
            token_a_mint,
            token_b_mint,
            token_a_decimals: 9, // Default to SOL decimals
            token_b_decimals: 6, // Default to USDC decimals  
            fee_rate,
        })
    }

    fn parse_raydium_structure(&self, data: &[u8]) -> Result<(Pubkey, Pubkey, f64)> {
        if data.len() < 752 {
            return Err(anyhow!("Invalid Raydium pool data length: {}", data.len()));
        }

        // Raydium AMM structure offsets
        let token_a_mint = self.extract_pubkey(data, 8)?;
        let token_b_mint = self.extract_pubkey(data, 40)?;
        
        // Raydium typically uses 0.25% fee
        let fee_rate = 0.0025;

        info!("   📊 Raydium pool: {} <-> {}", 
              self.short_pubkey(&token_a_mint), 
              self.short_pubkey(&token_b_mint));

        Ok((token_a_mint, token_b_mint, fee_rate))
    }

    fn parse_orca_structure(&self, data: &[u8]) -> Result<(Pubkey, Pubkey, f64)> {
        if data.len() < 653 {
            return Err(anyhow!("Invalid Orca pool data length: {}", data.len()));
        }

        // Orca Whirlpool structure offsets
        let token_a_mint = self.extract_pubkey(data, 101)?;
        let token_b_mint = self.extract_pubkey(data, 133)?;
        
        // Orca typically uses 0.3% fee
        let fee_rate = 0.003;

        info!("   📊 Orca pool: {} <-> {}", 
              self.short_pubkey(&token_a_mint), 
              self.short_pubkey(&token_b_mint));

        Ok((token_a_mint, token_b_mint, fee_rate))
    }

    async fn run_continuous_monitoring(&mut self) -> Result<()> {
        info!("🚀 Starting continuous pool monitoring...");

        let mut cycle = 0;
        
        loop {
            cycle += 1;
            let cycle_start = std::time::Instant::now();

            info!("\n⚡ === MONITORING CYCLE {} ===", cycle);

            // Update all pools in parallel
            self.update_all_pools_parallel().await?;

            // Analyze arbitrage opportunities
            let arbitrage_alerts = self.detect_arbitrage_opportunities().await?;

            // Report findings
            self.report_cycle_results(cycle, &arbitrage_alerts).await?;

            // Performance tracking
            let cycle_duration = cycle_start.elapsed().as_millis() as u64;
            self.update_times.push(cycle_duration);
            
            // Keep only last 100 measurements
            if self.update_times.len() > 100 {
                self.update_times.remove(0);
            }

            // Ultra-fast military-grade cycle
            sleep(Duration::from_millis(200)).await;
        }
    }

    async fn update_all_pools_parallel(&mut self) -> Result<()> {
        let pool_names: Vec<String> = self.monitored_pools.keys().cloned().collect();
        
        for pool_name in pool_names {
            match self.update_single_pool_data(&pool_name).await {
                Ok(_) => {},
                Err(e) => warn!("Failed to update {}: {}", pool_name, e),
            }
        }

        Ok(())
    }

    async fn update_single_pool_data(&mut self, pool_name: &str) -> Result<()> {
        if let Some(pool_config) = self.monitored_pools.get(pool_name) {
            let account = self.client.get_account(&pool_config.address).await?;
            
            // Extract current reserves
            let (reserve_a, reserve_b) = match pool_config.pool_type {
                PoolType::RaydiumAMM => self.extract_raydium_reserves(&account.data)?,
                PoolType::OrcaWhirlpool => self.extract_orca_reserves(&account.data)?,
                _ => return Err(anyhow!("Unsupported pool type")),
            };

            // Calculate prices
            let price_a_to_b = if reserve_a > 0 && reserve_b > 0 {
                (reserve_b as f64) / (reserve_a as f64)
            } else { 0.0 };
            
            let price_b_to_a = if price_a_to_b > 0.0 { 1.0 / price_a_to_b } else { 0.0 };

            // Estimate liquidity in USD (rough calculation)
            let liquidity_usd = self.estimate_liquidity_usd(reserve_a, reserve_b, pool_config);

            // Update live data
            if let Some(live_data) = self.live_data.get_mut(pool_name) {
                live_data.reserve_a = reserve_a;
                live_data.reserve_b = reserve_b;
                live_data.price_a_to_b = price_a_to_b;
                live_data.price_b_to_a = price_b_to_a;
                live_data.liquidity_usd = liquidity_usd;
                live_data.last_update = std::time::Instant::now();
                live_data.update_count += 1;
            }
        }

        Ok(())
    }

    fn extract_raydium_reserves(&self, data: &[u8]) -> Result<(u64, u64)> {
        // Raydium reserve positions
        let reserve_a = self.extract_u64(data, 504)?;
        let reserve_b = self.extract_u64(data, 512)?;
        Ok((reserve_a, reserve_b))
    }

    fn extract_orca_reserves(&self, data: &[u8]) -> Result<(u64, u64)> {
        // Orca reserve positions (approximate)
        let reserve_a = self.extract_u64(data, 229)?;
        let reserve_b = self.extract_u64(data, 237)?;
        Ok((reserve_a, reserve_b))
    }

    async fn detect_arbitrage_opportunities(&mut self) -> Result<Vec<ArbitrageAlert>> {
        let mut alerts = Vec::new();
        
        // Get all pool combinations
        let pool_names: Vec<String> = self.live_data.keys().cloned().collect();
        
        for i in 0..pool_names.len() {
            for j in (i + 1)..pool_names.len() {
                let pool_a_name = &pool_names[i];
                let pool_b_name = &pool_names[j];
                
                if let Some(alert) = self.check_arbitrage_between_pools(pool_a_name, pool_b_name).await? {
                    alerts.push(alert);
                }
            }
        }

        // Sort by profit potential
        alerts.sort_by(|a, b| {
            (b.profit_opportunity * b.confidence).partial_cmp(&(a.profit_opportunity * a.confidence))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(alerts)
    }

    async fn check_arbitrage_between_pools(&self, pool_a_name: &str, pool_b_name: &str) -> Result<Option<ArbitrageAlert>> {
        if let (Some(data_a), Some(data_b)) = (self.live_data.get(pool_a_name), self.live_data.get(pool_b_name)) {
            
            // Simple price difference check
            let price_diff = (data_a.price_a_to_b - data_b.price_a_to_b).abs();
            let avg_price = (data_a.price_a_to_b + data_b.price_a_to_b) / 2.0;
            
            if avg_price > 0.0 {
                let price_diff_percentage = (price_diff / avg_price) * 100.0;
                
                // Look for significant price differences (> 0.5%)
                if price_diff_percentage > 0.5 {
                    let confidence = self.calculate_arbitrage_confidence(data_a, data_b);
                    
                    // Estimate optimal trade amount
                    let min_liquidity = data_a.liquidity_usd.min(data_b.liquidity_usd);
                    let recommended_amount = (min_liquidity * 0.01) as u64; // 1% of min liquidity
                    
                    return Ok(Some(ArbitrageAlert {
                        pool_pair: (pool_a_name.to_string(), pool_b_name.to_string()),
                        profit_opportunity: price_diff_percentage,
                        confidence,
                        recommended_amount,
                        execution_time_window: Duration::from_secs(5),
                    }));
                }
            }
        }
        
        Ok(None)
    }

    fn calculate_arbitrage_confidence(&self, data_a: &LivePoolData, data_b: &LivePoolData) -> f64 {
        // Data freshness
        let freshness_a = if data_a.last_update.elapsed().as_secs() < 1 { 1.0 } else { 0.5 };
        let freshness_b = if data_b.last_update.elapsed().as_secs() < 1 { 1.0 } else { 0.5 };
        
        // Liquidity score
        let liquidity_score = if data_a.liquidity_usd > 1000.0 && data_b.liquidity_usd > 1000.0 { 1.0 } else { 0.7 };
        
        // Update frequency score
        let freq_score = if data_a.update_count > 10 && data_b.update_count > 10 { 1.0 } else { 0.8 };
        
        freshness_a * freshness_b * liquidity_score * freq_score
    }

    async fn report_cycle_results(&self, cycle: u64, alerts: &[ArbitrageAlert]) -> Result<()> {
        // Performance metrics
        let avg_update_time = if !self.update_times.is_empty() {
            self.update_times.iter().sum::<u64>() as f64 / self.update_times.len() as f64
        } else { 0.0 };

        info!("📊 CYCLE {} RESULTS:", cycle);
        info!("   ⚡ Avg Update Time: {:.2}ms", avg_update_time);
        info!("   🔍 Pools Monitored: {}", self.live_data.len());
        info!("   🎯 Arbitrage Alerts: {}", alerts.len());

        // Show pool status
        for (name, data) in &self.live_data {
            if data.reserve_a > 0 && data.reserve_b > 0 {
                info!("   📈 {}: {:.6} price, ${:.0} liquidity, {} updates",
                      name, data.price_a_to_b, data.liquidity_usd, data.update_count);
            }
        }

        // Show top arbitrage opportunities
        for (i, alert) in alerts.iter().take(3).enumerate() {
            info!("   🚨 ALERT #{}: {:.2}% profit between {} <-> {} (confidence: {:.2})",
                  i + 1, alert.profit_opportunity, alert.pool_pair.0, alert.pool_pair.1, alert.confidence);
        }

        Ok(())
    }

    fn estimate_liquidity_usd(&self, reserve_a: u64, reserve_b: u64, config: &PoolConfig) -> f64 {
        // Rough USD estimation (assuming one token is SOL ~$100, other is USDC ~$1)
        let sol_value = (reserve_a as f64 / 1_000_000_000.0) * 100.0; // SOL at $100
        let usdc_value = reserve_b as f64 / 1_000_000.0; // USDC at $1
        sol_value + usdc_value
    }

    // Helper functions
    fn extract_pubkey(&self, data: &[u8], offset: usize) -> Result<Pubkey> {
        if data.len() < offset + 32 {
            return Err(anyhow!("Data too short for pubkey at offset {}", offset));
        }
        let bytes: [u8; 32] = data[offset..offset + 32].try_into()?;
        Ok(Pubkey::new_from_array(bytes))
    }

    fn extract_u64(&self, data: &[u8], offset: usize) -> Result<u64> {
        if data.len() < offset + 8 {
            return Err(anyhow!("Data too short for u64 at offset {}", offset));
        }
        let bytes: [u8; 8] = data[offset..offset + 8].try_into()?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn short_pubkey(&self, pubkey: &Pubkey) -> String {
        let s = pubkey.to_string();
        format!("{}...{}", &s[..4], &s[s.len()-4..])
    }
}
