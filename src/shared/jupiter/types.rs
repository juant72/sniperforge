/// Jupiter API Types
/// 
/// Type definitions for Jupiter API requests and responses

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Jupiter quote response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterQuote {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    pub other_amount_threshold: String,
    #[serde(rename = "swapMode")]
    pub swap_mode: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "platformFee")]
    pub platform_fee: Option<PlatformFee>,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: String,
    #[serde(rename = "routePlan")]
    pub route_plan: Vec<RoutePlan>,
    #[serde(rename = "contextSlot")]
    pub context_slot: Option<u64>,
    #[serde(rename = "timeTaken")]
    pub time_taken: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformFee {
    pub amount: String,
    pub fee_bps: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePlan {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    pub percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapInfo {
    #[serde(rename = "ammKey")]
    pub amm_key: String,
    pub label: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "feeAmount")]
    pub fee_amount: String,
    #[serde(rename = "feeMint")]
    pub fee_mint: String,
}

/// Jupiter swap transaction response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterSwapTransaction {
    pub swap_transaction: String, // Base64 encoded transaction
    pub last_valid_block_height: Option<u64>,
    pub priority_fee_lamports: Option<u64>,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterToken {
    pub address: String,
    pub chain_id: u32,
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub logo_uri: Option<String>,
    pub tags: Vec<String>,
}

/// Price information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterPrice {
    pub id: String,
    pub mint_symbol: Option<String>,
    pub vs_token: String,
    pub vs_token_symbol: String,
    pub price: f64,
    pub extra_info: Option<HashMap<String, serde_json::Value>>,
}

/// Quote request parameters
#[derive(Debug, Clone)]
pub struct QuoteRequest {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount: u64,
    pub slippage_bps: Option<u16>,
    pub exclude_dexes: Option<Vec<String>>,
    pub only_direct_routes: Option<bool>,
    pub asset_ledger_filter: Option<String>,
    pub max_accounts: Option<u8>,
}

impl QuoteRequest {
    pub fn new(input_mint: Pubkey, output_mint: Pubkey, amount: u64) -> Self {
        Self {
            input_mint,
            output_mint,
            amount,
            slippage_bps: None,
            exclude_dexes: None,
            only_direct_routes: None,
            asset_ledger_filter: None,
            max_accounts: None,
        }
    }

    pub fn with_slippage(mut self, slippage_bps: u16) -> Self {
        self.slippage_bps = Some(slippage_bps);
        self
    }

    pub fn exclude_dexes(mut self, dexes: Vec<String>) -> Self {
        self.exclude_dexes = Some(dexes);
        self
    }

    pub fn only_direct_routes(mut self) -> Self {
        self.only_direct_routes = Some(true);
        self
    }
}

/// Swap execution result
#[derive(Debug, Clone)]
pub struct SwapResult {
    pub success: bool,
    pub transaction_signature: Option<String>,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact: f64,
    pub slippage: f64,
    pub route_info: RouteInfo,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub dexes_used: Vec<String>,
    pub number_of_routes: usize,
    pub best_route_label: String,
}

/// Well-known token addresses on Solana
pub mod tokens {
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    /// SOL (wrapped SOL)
    pub const WSOL: &str = "So11111111111111111111111111111111111111112";
    
    /// USDC
    pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    
    /// USDT
    pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

    /// Get SOL pubkey
    pub fn sol() -> Pubkey {
        Pubkey::from_str(WSOL).unwrap()
    }

    /// Get USDC pubkey  
    pub fn usdc() -> Pubkey {
        Pubkey::from_str(USDC).unwrap()
    }

    /// Get USDT pubkey
    pub fn usdt() -> Pubkey {
        Pubkey::from_str(USDT).unwrap()
    }
}
