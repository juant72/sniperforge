use anyhow::Result;

// ===== EXPERT MATHEMATICAL CONSTANTS =====
// Constantes optimizadas basadas en análisis real de mainnet

/// EXPERT TRADE SIZING - Based on real profitability analysis
pub const EXPERT_MINIMUM_TRADE_SIZE: u64 = 1_000_000_000; // 1.0 SOL minimum (PROFITABLE)
pub const EXPERT_MAXIMUM_TRADE_SIZE: u64 = 10_000_000_000; // 10.0 SOL maximum (SAFE)

/// EXPERT PROFIT REQUIREMENTS - Based on real trading costs
pub const MILITARY_MIN_PROFIT_BPS: u64 = 50; // 0.5% minimum profit (REALISTIC)
pub const EXPERT_MIN_POOL_TVL_USD: f64 = 10_000.0; // $10K minimum TVL

/// EXPERT LIQUIDITY REQUIREMENTS - Based on slippage analysis
pub const MILITARY_MIN_LIQUIDITY: u64 = 10_000_000; // 0.01 SOL minimum (REALISTIC)
pub const MILITARY_MAX_SLIPPAGE_BPS: u64 = 200; // 2% maximum slippage

/// EXPERT TIMING AND RATE LIMITS - Production optimized
pub const MILITARY_RATE_LIMIT_DELAY: u64 = 150; // 150ms between requests
pub const MILITARY_TIMEOUT_SECONDS: u64 = 10; // 10s timeout
pub const MILITARY_MAX_POOLS_PER_DEX: usize = 50; // Limit per DEX

/// EXPERT FEE CALCULATIONS - Real network costs
pub const SOLANA_BASE_FEE: u64 = 5_000; // 0.000005 SOL base fee
pub const PRIORITY_FEE_MODERATE: u64 = 10_000; // 0.00001 SOL priority
pub const COMPUTE_UNITS_ARBITRAGE: u64 = 200_000; // Realistic compute units

/// DEX-SPECIFIC CONSTANTS
pub const RAYDIUM_FEE_BPS: u64 = 25; // 0.25% fee
pub const ORCA_FEE_BPS: u64 = 30; // 0.30% fee  
pub const WHIRLPOOL_FEE_BPS: u64 = 5; // 0.05% fee (concentrated liquidity)

/// EXPERT VALIDATION THRESHOLDS
pub const MAX_REALISTIC_LIQUIDITY: u64 = 10_000_000_000_000_000; // 10M SOL
pub const MIN_TOKEN_RATIO: f64 = 0.000001;
pub const MAX_TOKEN_RATIO: f64 = 1_000_000.0;

/// ADAPTIVE CONFIGURATION
#[derive(Debug, Clone)]
pub struct AdaptiveConfig {
    pub current_batch_size: usize,
    pub success_rate: f64,
    pub last_update: std::time::Instant,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            current_batch_size: 3,
            success_rate: 0.0,
            last_update: std::time::Instant::now(),
        }
    }
}

impl AdaptiveConfig {
    pub fn adjust_batch_size(&mut self, success_rate: f64) {
        self.success_rate = success_rate;
        
        if success_rate > 0.9 {
            self.current_batch_size = (self.current_batch_size + 1).min(5);
        } else if success_rate < 0.7 {
            self.current_batch_size = (self.current_batch_size.saturating_sub(1)).max(1);
        }
        
        self.last_update = std::time::Instant::now();
    }
}
