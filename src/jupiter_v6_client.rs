use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

pub const JUPITER_V6_QUOTE_API: &str = "https://quote-api.jup.ag/v6";
pub const JUPITER_V6_SWAP_API: &str = "https://quote-api.jup.ag/v6";

#[derive(Serialize, Deserialize, Debug)]
pub struct JupiterV6QuoteRequest {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    pub amount: u64,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "onlyDirectRoutes")]
    pub only_direct_routes: bool,
    #[serde(rename = "asLegacyTransaction")]
    pub as_legacy_transaction: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlatformFee {
    pub amount: String,
    pub feeBps: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoutePlanStep {
    pub swapInfo: SwapInfo,
    pub percent: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwapInfo {
    pub ammKey: String,
    pub label: String,
    pub inputMint: String,
    pub outputMint: String,
    pub inAmount: String,
    pub outAmount: String,
    pub feeAmount: String,
    pub feeMint: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JupiterV6Route {
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
    pub route_plan: Vec<RoutePlanStep>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JupiterV6SwapRequest {
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
    #[serde(rename = "quoteResponse")]
    pub quote_response: JupiterV6Route,
    pub config: SwapConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SwapConfig {
    #[serde(rename = "wrapAndUnwrapSol")]
    pub wrap_and_unwrap_sol: bool,
    #[serde(rename = "feeAccount")]
    pub fee_account: Option<String>,
    #[serde(rename = "computeUnitPriceMicroLamports")]
    pub compute_unit_price_micro_lamports: String,
    #[serde(rename = "prioritizationFeeLamports")]
    pub prioritization_fee_lamports: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JupiterV6SwapResponse {
    #[serde(rename = "swapTransaction")]
    pub swap_transaction: String,
    #[serde(rename = "lastValidBlockHeight")]
    pub last_valid_block_height: u64,
}

pub struct JupiterV6Client {
    client: Client,
    quote_api_url: String,
    swap_api_url: String,
}

impl JupiterV6Client {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            quote_api_url: JUPITER_V6_QUOTE_API.to_string(),
            swap_api_url: JUPITER_V6_SWAP_API.to_string(),
        }
    }

    pub async fn get_quote(&self, request: JupiterV6QuoteRequest) -> Result<JupiterV6Route> {
        let mut params = HashMap::new();
        params.insert("inputMint", request.input_mint);
        params.insert("outputMint", request.output_mint);
        params.insert("amount", request.amount.to_string());
        params.insert("slippageBps", request.slippage_bps.to_string());
        params.insert("onlyDirectRoutes", request.only_direct_routes.to_string());
        params.insert("asLegacyTransaction", request.as_legacy_transaction.to_string());

        let response = self.client
            .get(&format!("{}/quote", self.quote_api_url))
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Jupiter API error {}: {}", status, error_text));
        }

        let route: JupiterV6Route = response.json().await?;
        Ok(route)
    }

    pub async fn get_routes(&self, input_mint: &str, output_mint: &str, amount: u64) -> Result<Vec<JupiterV6Route>> {
        let request = JupiterV6QuoteRequest {
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            amount,
            slippage_bps: 50, // 0.5% slippage
            only_direct_routes: false,
            as_legacy_transaction: false,
        };

        // Jupiter v6 retorna una sola mejor ruta
        let best_route = self.get_quote(request).await?;
        Ok(vec![best_route])
    }

    pub async fn prepare_swap_transaction(&self, route: &JupiterV6Route, user_public_key: &str) -> Result<String> {
        let swap_request = JupiterV6SwapRequest {
            user_public_key: user_public_key.to_string(),
            quote_response: route.clone(),
            config: SwapConfig {
                wrap_and_unwrap_sol: true,
                fee_account: None,
                compute_unit_price_micro_lamports: "auto".to_string(),
                prioritization_fee_lamports: "auto".to_string(),
            },
        };

        let response = self.client
            .post(&format!("{}/swap", self.swap_api_url))
            .json(&swap_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Jupiter swap API error {}: {}", status, error_text));
        }

        let swap_response: JupiterV6SwapResponse = response.json().await?;
        Ok(swap_response.swap_transaction)
    }

    pub async fn test_connectivity(&self) -> Result<bool> {
        // Test simple con SOL -> USDC
        let request = JupiterV6QuoteRequest {
            input_mint: "So11111111111111111111111111111111111111112".to_string(), // SOL
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            amount: 1000000, // 0.001 SOL
            slippage_bps: 50,
            only_direct_routes: false,
            as_legacy_transaction: false,
        };

        match self.get_quote(request).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl Default for JupiterV6Client {
    fn default() -> Self {
        Self::new()
    }
}
