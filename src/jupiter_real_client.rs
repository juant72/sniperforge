// ================================================================================
// JUPITER REAL SWAP IMPLEMENTATION
// ================================================================================
// Implementación real de swaps usando Jupiter V6 API
// ================================================================================

use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    transaction::Transaction,
    signature::{Keypair, Signature},
    pubkey::Pubkey,
    commitment_config::CommitmentConfig,
};
use std::str::FromStr;
use tracing::{info, error, warn};

/// Configuración para Jupiter real swaps
#[derive(Debug, Clone)]
pub struct JupiterRealConfig {
    pub api_base_url: String,
    pub slippage_bps: u16,
    pub compute_unit_price_micro_lamports: Option<u64>,
    pub priority_fee_lamports: Option<u64>,
}

impl Default for JupiterRealConfig {
    fn default() -> Self {
        Self {
            api_base_url: "https://quote-api.jup.ag/v6".to_string(),
            slippage_bps: 50, // 0.5%
            compute_unit_price_micro_lamports: Some(1000), // 1000 micro lamports
            priority_fee_lamports: Some(5000), // 5000 lamports priority fee
        }
    }
}

/// Request para Jupiter quote API
#[derive(Debug, Serialize)]
pub struct JupiterQuoteRequest {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    pub amount: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
}

/// Response de Jupiter quote API
#[derive(Debug, Deserialize)]
pub struct JupiterQuoteResponse {
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
}

#[derive(Debug, Deserialize)]
pub struct PlatformFee {
    pub amount: String,
    #[serde(rename = "feeBps")]
    pub fee_bps: u16,
}

#[derive(Debug, Deserialize)]
pub struct RoutePlan {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    pub percent: u8,
}

#[derive(Debug, Deserialize)]
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

/// Request para Jupiter swap API
#[derive(Debug, Serialize)]
pub struct JupiterSwapRequest {
    #[serde(rename = "quoteResponse")]
    pub quote_response: JupiterQuoteResponse,
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
    #[serde(rename = "wrapAndUnwrapSol")]
    pub wrap_and_unwrap_sol: bool,
    #[serde(rename = "computeUnitPriceMicroLamports")]
    pub compute_unit_price_micro_lamports: Option<u64>,
    #[serde(rename = "prioritizationFeeLamports")]
    pub prioritization_fee_lamports: Option<u64>,
}

/// Response de Jupiter swap API
#[derive(Debug, Deserialize)]
pub struct JupiterSwapResponse {
    #[serde(rename = "swapTransaction")]
    pub swap_transaction: String, // Base64 encoded transaction
}

/// Cliente para ejecutar swaps reales usando Jupiter
pub struct JupiterRealClient {
    http_client: Client,
    rpc_client: RpcClient,
    config: JupiterRealConfig,
}

impl JupiterRealClient {
    /// Crear nuevo cliente Jupiter real
    pub fn new(rpc_endpoint: String, config: Option<JupiterRealConfig>) -> Result<Self> {
        let http_client = Client::new();
        let rpc_client = RpcClient::new_with_commitment(
            rpc_endpoint,
            CommitmentConfig::confirmed(),
        );
        let config = config.unwrap_or_default();

        Ok(Self {
            http_client,
            rpc_client,
            config,
        })
    }

    /// Obtener quote de Jupiter para un swap
    pub async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_lamports: u64,
    ) -> Result<JupiterQuoteResponse> {
        let url = format!("{}/quote", self.config.api_base_url);
        
        let request = JupiterQuoteRequest {
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            amount: amount_lamports.to_string(),
            slippage_bps: self.config.slippage_bps,
        };

        info!("🔍 Obteniendo quote de Jupiter: {} {} → {} {}", 
              amount_lamports, input_mint, "?", output_mint);

        let response = self.http_client
            .get(&url)
            .query(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Jupiter quote failed: {} - {}", response.status(), error_text));
        }

        let quote: JupiterQuoteResponse = response.json().await?;
        
        info!("✅ Quote obtenido: {} {} → {} {}", 
              quote.in_amount, input_mint, quote.out_amount, output_mint);
        info!("   💰 Price impact: {}%", quote.price_impact_pct);

        Ok(quote)
    }

    /// Ejecutar swap real usando Jupiter
    pub async fn execute_swap(
        &self,
        quote: JupiterQuoteResponse,
        user_keypair: &Keypair,
    ) -> Result<Signature> {
        info!("🚀 Ejecutando swap real con Jupiter...");
        
        // 1. Obtener transaction de Jupiter
        let transaction = self.get_swap_transaction(quote, &user_keypair.pubkey()).await?;
        
        // 2. Firmar transaction
        let signed_transaction = self.sign_transaction(transaction, user_keypair)?;
        
        // 3. Enviar a blockchain
        let signature = self.send_transaction(signed_transaction).await?;
        
        info!("✅ Swap real ejecutado exitosamente!");
        info!("   📝 Transaction signature: {}", signature);
        
        Ok(signature)
    }

    /// Obtener transaction serializada de Jupiter
    async fn get_swap_transaction(
        &self,
        quote: JupiterQuoteResponse,
        user_pubkey: &Pubkey,
    ) -> Result<Transaction> {
        let url = format!("{}/swap", self.config.api_base_url);
        
        let request = JupiterSwapRequest {
            quote_response: quote,
            user_public_key: user_pubkey.to_string(),
            wrap_and_unwrap_sol: true,
            compute_unit_price_micro_lamports: self.config.compute_unit_price_micro_lamports,
            prioritization_fee_lamports: self.config.priority_fee_lamports,
        };

        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Jupiter swap API failed: {} - {}", response.status(), error_text));
        }

        let swap_response: JupiterSwapResponse = response.json().await?;
        
        // Decodificar transaction de base64
        let transaction_bytes = general_purpose::STANDARD
            .decode(swap_response.swap_transaction)
            .map_err(|e| anyhow!("Failed to decode transaction: {}", e))?;
        
        let transaction: Transaction = bincode::deserialize(&transaction_bytes)
            .map_err(|e| anyhow!("Failed to deserialize transaction: {}", e))?;

        info!("✅ Transaction obtenida de Jupiter API");
        
        Ok(transaction)
    }

    /// Firmar transaction con keypair del usuario
    fn sign_transaction(&self, mut transaction: Transaction, keypair: &Keypair) -> Result<Transaction> {
        info!("🔐 Firmando transaction...");
        
        // Obtener recent blockhash
        let recent_blockhash = self.rpc_client.get_latest_blockhash()
            .map_err(|e| anyhow!("Failed to get recent blockhash: {}", e))?;
        
        transaction.message.recent_blockhash = recent_blockhash;
        
        // Firmar transaction
        transaction.sign(&[keypair], recent_blockhash);
        
        info!("✅ Transaction firmada");
        
        Ok(transaction)
    }

    /// Enviar transaction firmada a blockchain
    async fn send_transaction(&self, transaction: Transaction) -> Result<Signature> {
        info!("📡 Enviando transaction a blockchain...");
        
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)
            .map_err(|e| anyhow!("Failed to send transaction: {}", e))?;
        
        info!("✅ Transaction confirmada en blockchain");
        
        Ok(signature)
    }
}

/// Mints comunes para testing en mainnet
pub mod common_mints {
    pub const SOL: &str = "So11111111111111111111111111111111111111112"; // Wrapped SOL
    pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC
    pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"; // USDT
    pub const RAY: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"; // Raydium
    pub const BONK: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"; // BONK
}
