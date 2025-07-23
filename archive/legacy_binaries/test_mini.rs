use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, trace};
use solana_client::nonblocking::rpc_client::RpcClient;
use reqwest;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Keypair,
    signer::Signer,
    pubkey::Pubkey,
    transaction::Transaction,
    instruction::Instruction,
    account::Account,
    hash::Hash,
};
use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};
use std::time::Instant;
use futures_util::future::join_all;

// ===== MILITARY ARBITRAGE SYSTEM V3.0 =====
// 
// 🚫 STRICT RULE: NO FAKE DATA ALLOWED - ONLY REAL BLOCKCHAIN DATA
// 
// This enhanced system provides:
// ✅ 15+ DEX Support with dynamic discovery
// ✅ Parallel pool processing for maximum speed
// ✅ Real-time price feeds with CoinGecko integration
// ✅ Unknown program detection and classification
// ✅ Professional-grade output with detailed analytics
// ✅ Advanced opportunity detection with risk analysis
// 
// Maximum allowed profit: 30% per trade (realistic)
// Minimum pool liquidity: 0.5 SOL per token (accessible)
// Maximum trade impact: 1% of pool liquidity (safe)
// Network latency target: <200ms (premium RPCs)
// 
// ===== MILITARY-GRADE STRATEGIC CONSTANTS =====

// 🎯 STRATEGIC PARAMETERS (OPTIMIZED FOR REAL MARKET CONDITIONS)
const MILITARY_LATENCY_TARGET: u64 = 200; // < 200ms end-to-end (ENHANCED)
const MILITARY_MIN_PROFIT_BPS: u64 = 3; // ≥ 0.03% real profit (ULTRA AGGRESSIVE)
const MILITARY_MAX_SLIPPAGE_BPS: u64 = 50; // Max 0.5% slippage
const MILITARY_PRICE_WATCH_INTERVAL: u64 = 500; // 500ms price monitoring (ULTRA FAST)
const MILITARY_RETRY_ATTEMPTS: u8 = 3; // Reduced retry logic to avoid rate limits
const MILITARY_MIN_LIQUIDITY: u64 = 5_000_000; // 0.005 SOL minimum (ULTRA LOW)
const MILITARY_MAX_PARALLEL_POOLS: usize = 5; // Ultra-reduced to avoid overload
const MILITARY_POOL_REFRESH_INTERVAL: u64 = 30; // 30s pool data refresh
const MILITARY_RATE_LIMIT_DELAY: u64 = 300; // Increased to 300ms delay between requests
const MILITARY_BATCH_SIZE: usize = 2; // Ultra-small batches for stability
const MILITARY_TIMEOUT_SECONDS: u64 = 8; // Reduced timeout to fail faster

// 🔍 ENHANCED DISCOVERY PARAMETERS
const MILITARY_MICRO_BATCH_SIZE: usize = 2; // Process only 2 pools at a time
const MILITARY_INTER_BATCH_DELAY: u64 = 500; // 500ms between micro-batches
const MILITARY_MAX_POOLS_PER_DEX: usize = 10; // Limit pools per DEX to avoid overload
const MILITARY_VALIDATION_TIMEOUT: u64 = 3; // Quick validation timeout

// 🎯 DEX-SPECIFIC DATA SIZES (for precise filtering)
const RAYDIUM_POOL_SIZE: usize = 752; // Exact Raydium AMM pool account size
const ORCA_POOL_SIZE: usize = 324; // Exact Orca pool account size  
const WHIRLPOOL_SIZE: usize = 653; // Exact Whirlpool account size
const SERUM_MARKET_SIZE: usize = 3228; // Exact Serum market account size
const METEORA_POOL_SIZE: usize = 1200; // Meteora DLMM pool size

// 🔐 PREMIUM RPC ENDPOINTS (Helius Premium - Fastest Available)
const HELIUS_PREMIUM_RPC: &str = "https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY";
const PREMIUM_RPC_ENDPOINTS: &[&str] = &[
    "https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY", // Helius Premium - Primary
    "https://solana-mainnet.g.alchemy.com/v2/YOUR_API_KEY", // Alchemy Premium
    "https://rpc.ankr.com/solana", // Ankr Premium
    "https://api.mainnet-beta.solana.com", // Public fallback
];

// 🧰 ENHANCED API INTEGRATIONS
const JUPITER_PRICE_API: &str = "https://price.jup.ag/v4/price";
const JUPITER_QUOTE_API: &str = "https://quote-api.jup.ag/v6/quote";
const COINGECKO_API: &str = "https://api.coingecko.com/api/v3/simple/price";
const BIRDEYE_API: &str = "https://public-api.birdeye.so/public/price";

// 🚀 EXPANDED MULTI-DEX INTEGRATION (15+ DEXes)
const SUPPORTED_DEXES: &[&str] = &[
    "Raydium", "Orca", "OrcaWhirlpool", "Serum", "Meteora", "SolFi", 
    "Jupiter", "Lifinity", "Aldrin", "Saber", "Mercurial", "Cropper", 
    "GoonDex", "SwapNyd", "Unknown9H6"
];

// ===== MILITARY-GRADE DIRECT POOL ACCESS =====
// No APIs, no rate limits, pure blockchain access

// Raydium Program IDs
const RAYDIUM_AMM_PROGRAM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const RAYDIUM_AUTHORITY: &str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";

// Orca Program IDs  
const ORCA_WHIRLPOOL_PROGRAM: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
const ORCA_SWAP_PROGRAM: &str = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";

