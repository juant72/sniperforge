//! Jupiter API Types and Data Structures
//! 
//! All Jupiter API request/response types, configurations, and data structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Jupiter price response structure for V3 API
#[derive(Debug, Deserialize)]
pub struct JupiterPriceResponse {
    #[serde(flatten)]
    pub data: HashMap<String, PriceDataV3>,
}

/// Price data structure for V3 API
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct PriceDataV3 {
    pub id: String,
    #[serde(rename = "mintSymbol")]
    pub mintSymbol: String,
    #[serde(rename = "vsToken")]
    pub vsToken: String,
    #[serde(rename = "vsTokenSymbol")]
    pub vsTokenSymbol: String,
    pub price: f64,
}

/// Jupiter V6 Quote Request
#[derive(Debug, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct QuoteRequest {
    #[serde(rename = "inputMint")]
    pub inputMint: String,
    #[serde(rename = "outputMint")]
    pub outputMint: String,
    pub amount: u64,
    #[serde(rename = "slippageBps")]
    pub slippageBps: u16,
}

/// Jupiter V6 Quote Response
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct QuoteResponse {
    #[serde(rename = "inputMint")]
    pub inputMint: String,
    #[serde(rename = "inAmount")]
    pub inAmount: String,
    #[serde(rename = "outputMint")]
    pub outputMint: String,
    #[serde(rename = "outAmount")]
    pub outAmount: String,
    #[serde(rename = "otherAmountThreshold")]
    pub otherAmountThreshold: String,
    #[serde(rename = "swapMode")]
    pub swapMode: String,
    #[serde(rename = "slippageBps")]
    pub slippageBps: u16,
    #[serde(rename = "platformFee")]
    pub platformFee: Option<PlatformFee>,
    #[serde(rename = "priceImpactPct")]
    pub priceImpactPct: String,
    #[serde(rename = "routePlan")]
    pub routePlan: Vec<RoutePlan>,
    #[serde(rename = "contextSlot")]
    pub contextSlot: u64,
    #[serde(rename = "timeTaken")]
    pub timeTaken: f64,
    
    // Computed fields (not from API)
    #[serde(rename = "simplerRouteUsed")]
    pub simplerRouteUsed: bool,
}

impl QuoteResponse {
    /// Get input amount as f64 (converted from lamports to SOL)
    pub fn in_amount_f64(&self) -> f64 {
        self.inAmount.parse::<f64>().unwrap_or(0.0) / 1_000_000_000.0
    }

    /// Get output amount as f64 (converted from base units)
    pub fn out_amount_f64(&self) -> f64 {
        self.outAmount.parse::<f64>().unwrap_or(0.0) / 1_000_000.0
    }

    /// Get price impact as f64 percentage
    pub fn price_impact_pct_f64(&self) -> f64 {
        self.priceImpactPct.parse().unwrap_or(0.0)
    }

    /// Get input amount as u64 (lamports)
    pub fn in_amount_lamports(&self) -> u64 {
        self.inAmount.parse().unwrap_or(0)
    }

    /// Get output amount as u64 (base units)  
    pub fn out_amount_units(&self) -> u64 {
        self.outAmount.parse().unwrap_or(0)
    }

    // Compatibility fields for existing code
    pub fn in_amount(&self) -> f64 {
        self.in_amount_f64()
    }

    pub fn out_amount(&self) -> f64 {
        self.out_amount_f64()
    }

    pub fn price_impact_pct(&self) -> f64 {
        self.price_impact_pct_f64()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct PlatformFee {
    pub amount: String,
    #[serde(rename = "feeBps")]
    pub feeBps: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct RoutePlan {
    #[serde(rename = "swapInfo")]
    pub swapInfo: SwapInfo,
    pub percent: u8,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct SwapInfo {
    #[serde(rename = "ammKey")]
    pub ammKey: String,
    pub label: String,
    #[serde(rename = "inputMint")]
    pub inputMint: String,
    #[serde(rename = "outputMint")]
    pub outputMint: String,
    #[serde(rename = "inAmount")]
    pub inAmount: String,
    #[serde(rename = "outAmount")]
    pub outAmount: String,
    #[serde(rename = "feeAmount")]
    pub feeAmount: String,
    #[serde(rename = "feeMint")]
    pub feeMint: String,
}

/// Swap execution result
#[derive(Debug, Clone)]
pub struct SwapExecutionResult {
    pub success: bool,
    pub transaction_signature: String,
    pub output_amount: f64,
    pub actual_slippage: f64,
    pub fee_amount: f64,
    pub block_height: u64,
    pub logs: Vec<String>,
}

/// Swap request for Jupiter V6 API
#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct SwapRequest {
    #[serde(rename = "quoteResponse")]
    pub quoteResponse: QuoteResponse,
    #[serde(rename = "userPublicKey")]
    pub userPublicKey: String,
    #[serde(rename = "dynamicComputeUnitLimit")]
    pub dynamicComputeUnitLimit: Option<bool>,
    #[serde(rename = "dynamicSlippage")]
    pub dynamicSlippage: Option<bool>,
    #[serde(rename = "prioritizationFeeLamports")]
    pub prioritizationFeeLamports: Option<PrioritizationFee>,
    #[serde(rename = "asLegacyTransaction")]
    pub asLegacyTransaction: Option<bool>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct PrioritizationFee {
    #[serde(rename = "priorityLevelWithMaxLamports")]
    pub priorityLevelWithMaxLamports: PriorityLevelConfig,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct PriorityLevelConfig {
    #[serde(rename = "maxLamports")]
    pub maxLamports: u64,
    #[serde(rename = "priorityLevel")]
    pub priorityLevel: String,
}

/// Swap response from Jupiter V6 API
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct SwapResponse {
    #[serde(rename = "swapTransaction")]
    pub swapTransaction: String,
    #[serde(rename = "lastValidBlockHeight")]
    pub lastValidBlockHeight: u64,
    #[serde(rename = "prioritizationFeeLamports")]
    pub prioritizationFeeLamports: u64,
    #[serde(rename = "computeUnitLimit")]
    pub computeUnitLimit: u64,
}

/// Simple swap result (for transaction building only)
#[derive(Debug, Clone)]
pub struct SwapResult {
    pub success: bool,
    pub transaction_signature: Option<String>,
    pub output_amount: f64,
    pub actual_slippage: f64,
    pub fee_amount: f64,
}

/// Token price information
#[derive(Debug, Clone)]
pub struct TokenPrice {
    pub price: f64,
    pub volume_24h: Option<f64>,
    pub market_cap: Option<f64>,
}

/// Transaction confirmation configuration
#[derive(Debug, Clone)]
pub struct ConfirmationConfig {
    pub commitment: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl Default for ConfirmationConfig {
    fn default() -> Self {
        Self {
            commitment: "confirmed".to_string(),
            timeout_seconds: 60,
            max_retries: 5,
            retry_delay_ms: 1000,
        }
    }
}

/// Transaction confirmation result
#[derive(Debug, Clone)]
pub struct TransactionConfirmation {
    pub signature: String,
    pub confirmed: bool,
    pub slot: u64,
    pub confirmation_time: DateTime<Utc>,
    pub error: Option<String>,
}

impl TransactionConfirmation {
    pub fn success(signature: String, slot: u64) -> Self {
        Self {
            signature,
            confirmed: true,
            slot,
            confirmation_time: Utc::now(),
            error: None,
        }
    }
    
    pub fn failed(signature: String, error: String) -> Self {
        Self {
            signature,
            confirmed: false,
            slot: 0,
            confirmation_time: Utc::now(),
            error: Some(error),
        }
    }
}
