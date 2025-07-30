// ================================================================================
// CORE JUPITER - JUPITER V6 INTEGRATION LAYER
// ================================================================================
// Cliente Jupiter V6 modular y reutilizable para todos los bots
// Manejo de quotes, swaps, y optimizaciones de rutas
// ================================================================================

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use solana_sdk::pubkey::Pubkey;
use tokio::time::timeout;
use tracing::{info, warn, debug, error};

use crate::{CoreResult, CoreError, TokenInfo};

/// Error específico de Jupiter
#[derive(Debug, thiserror::Error)]
pub enum JupiterError {
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Quote error: {0}")]
    QuoteError(String),
    
    #[error("Swap error: {0}")]
    SwapError(String),
    
    #[error("Route not found")]
    RouteNotFound,
    
    #[error("Insufficient liquidity")]
    InsufficientLiquidity,
    
    #[error("Timeout error")]
    Timeout,
}

/// Cliente Jupiter V6 principal
#[derive(Clone)]
pub struct JupiterClient {
    http_client: reqwest::Client,
    quote_url: String,
    price_url: String,
    swap_url: String,
    timeout: Duration,
    max_retries: u32,
}

impl JupiterClient {
    /// Crear nuevo cliente Jupiter
    pub fn new(config: &JupiterConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            http_client,
            quote_url: config.quote_url.clone(),
            price_url: config.price_url.clone(),
            swap_url: config.swap_url.clone(),
            timeout: Duration::from_secs(config.timeout_seconds),
            max_retries: config.max_retries,
        }
    }
    
    /// Obtener quote para swap
    pub async fn get_quote(&self, request: &QuoteRequest) -> CoreResult<SwapQuote> {
        debug!("🔍 Solicitando quote Jupiter: {} {} → {}", 
               request.amount, request.input_mint, request.output_mint);
        
        let mut params = vec![
            ("inputMint", request.input_mint.to_string()),
            ("outputMint", request.output_mint.to_string()),
            ("amount", request.amount.to_string()),
            ("slippageBps", request.slippage_bps.to_string()),
        ];
        
        if let Some(ref platform_fee) = request.platform_fee_bps {
            params.push(("platformFeeBps", platform_fee.to_string()));
        }
        
        if request.only_direct_routes {
            params.push(("onlyDirectRoutes", "true".to_string()));
        }
        
        if let Some(max_accounts) = request.max_accounts {
            params.push(("maxAccounts", max_accounts.to_string()));
        }
        
        for retry in 0..=self.max_retries {
            match self.execute_quote_request(&params).await {
                Ok(quote) => {
                    debug!("✅ Quote obtenido: {} {} esperados", quote.out_amount, request.output_mint);
                    return Ok(quote);
                }
                Err(e) if retry < self.max_retries => {
                    warn!("⚠️ Intento {}/{} falló: {}", retry + 1, self.max_retries + 1, e);
                    tokio::time::sleep(Duration::from_millis(500 * (retry + 1) as u64)).await;
                }
                Err(e) => return Err(CoreError::Jupiter(JupiterError::QuoteError(e.to_string()))),
            }
        }
        
        unreachable!()
    }
    
    /// Ejecutar request de quote
    async fn execute_quote_request(&self, params: &[(String, String)]) -> Result<SwapQuote> {
        let url = format!("{}?{}", self.quote_url, 
                         params.iter()
                               .map(|(k, v)| format!("{}={}", k, v))
                               .collect::<Vec<_>>()
                               .join("&"));
        
        let response = timeout(self.timeout, self.http_client.get(&url).send()).await
            .map_err(|_| anyhow!("Request timeout"))?
            .map_err(|e| anyhow!("Network error: {}", e))?;
            
        if !response.status().is_success() {
            return Err(anyhow!("API error: {}", response.status()));
        }
        
        let data: Value = response.json().await
            .map_err(|e| anyhow!("JSON parse error: {}", e))?;
            
        self.parse_quote_response(data)
    }
    
    /// Parse respuesta de quote
    fn parse_quote_response(&self, data: Value) -> Result<SwapQuote> {
        let routes = data["data"].as_array()
            .ok_or_else(|| anyhow!("No routes found"))?;
            
        if routes.is_empty() {
            return Err(anyhow!("No routes available"));
        }
        
        let best_route = &routes[0];
        
        Ok(SwapQuote {
            input_mint: best_route["inputMint"].as_str()
                .ok_or_else(|| anyhow!("Missing inputMint"))?
                .parse()
                .map_err(|e| anyhow!("Invalid inputMint: {}", e))?,
            in_amount: best_route["inAmount"].as_str()
                .ok_or_else(|| anyhow!("Missing inAmount"))?
                .parse()
                .map_err(|e| anyhow!("Invalid inAmount: {}", e))?,
            output_mint: best_route["outputMint"].as_str()
                .ok_or_else(|| anyhow!("Missing outputMint"))?
                .parse()
                .map_err(|e| anyhow!("Invalid outputMint: {}", e))?,
            out_amount: best_route["outAmount"].as_str()
                .ok_or_else(|| anyhow!("Missing outAmount"))?
                .parse()
                .map_err(|e| anyhow!("Invalid outAmount: {}", e))?,
            price_impact_pct: best_route["priceImpactPct"].as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0),
            route_plan: self.parse_route_plan(&best_route["routePlan"])?,
            slippage_bps: 0, // Will be set from request
            platform_fee_bps: None,
            route_data: best_route.clone(),
        })
    }
    
    /// Parse plan de ruta
    fn parse_route_plan(&self, route_plan: &Value) -> Result<Vec<RouteStep>> {
        let steps = route_plan.as_array()
            .ok_or_else(|| anyhow!("Invalid route plan"))?;
            
        let mut result = Vec::new();
        for step in steps {
            result.push(RouteStep {
                swap_info: step["swapInfo"]["label"].as_str()
                    .unwrap_or("Unknown")
                    .to_string(),
                input_mint: step["swapInfo"]["inputMint"].as_str()
                    .unwrap_or("")
                    .parse()
                    .unwrap_or(Pubkey::default()),
                output_mint: step["swapInfo"]["outputMint"].as_str()
                    .unwrap_or("")
                    .parse()
                    .unwrap_or(Pubkey::default()),
                fee_amount: step["swapInfo"]["feeAmount"].as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0),
                fee_mint: step["swapInfo"]["feeMint"].as_str()
                    .unwrap_or("")
                    .parse()
                    .unwrap_or(Pubkey::default()),
            });
        }
        
        Ok(result)
    }
    
    /// Obtener precio de token
    pub async fn get_price(&self, mint: &Pubkey) -> CoreResult<TokenPrice> {
        debug!("💰 Obteniendo precio para: {}", mint);
        
        let url = format!("{}?ids={}", self.price_url, mint);
        
        for retry in 0..=self.max_retries {
            match self.execute_price_request(&url).await {
                Ok(price) => {
                    debug!("✅ Precio obtenido: ${:.6}", price.price_usd);
                    return Ok(price);
                }
                Err(e) if retry < self.max_retries => {
                    warn!("⚠️ Precio intento {}/{} falló: {}", retry + 1, self.max_retries + 1, e);
                    tokio::time::sleep(Duration::from_millis(200 * (retry + 1) as u64)).await;
                }
                Err(e) => return Err(CoreError::Jupiter(JupiterError::ApiError(e.to_string()))),
            }
        }
        
        unreachable!()
    }
    
    /// Ejecutar request de precio
    async fn execute_price_request(&self, url: &str) -> Result<TokenPrice> {
        let response = timeout(self.timeout, self.http_client.get(url).send()).await
            .map_err(|_| anyhow!("Request timeout"))?
            .map_err(|e| anyhow!("Network error: {}", e))?;
            
        if !response.status().is_success() {
            return Err(anyhow!("API error: {}", response.status()));
        }
        
        let data: Value = response.json().await
            .map_err(|e| anyhow!("JSON parse error: {}", e))?;
            
        self.parse_price_response(data)
    }
    
    /// Parse respuesta de precio
    fn parse_price_response(&self, data: Value) -> Result<TokenPrice> {
        let prices = data["data"].as_object()
            .ok_or_else(|| anyhow!("No price data"))?;
            
        if prices.is_empty() {
            return Err(anyhow!("No prices found"));
        }
        
        let (mint_str, price_info) = prices.iter().next().unwrap();
        let price_usd = price_info["price"].as_f64()
            .ok_or_else(|| anyhow!("Invalid price"))?;
            
        Ok(TokenPrice {
            mint: mint_str.parse()
                .map_err(|e| anyhow!("Invalid mint: {}", e))?,
            price_usd,
            price_sol: price_usd / 180.0, // Aproximado, se debería obtener el precio real de SOL
            volume_24h: price_info["vol24h"].as_f64().unwrap_or(0.0),
            timestamp: crate::current_timestamp(),
            source: "Jupiter".to_string(),
        })
    }
    
    /// Crear transacción de swap
    pub async fn create_swap_transaction(&self, quote: &SwapQuote, wallet_pubkey: &Pubkey) -> CoreResult<SwapTransaction> {
        debug!("🔨 Creando transacción de swap");
        
        let request_body = json!({
            "userPublicKey": wallet_pubkey.to_string(),
            "quoteResponse": quote.route_data,
            "config": {
                "wrapAndUnwrapSol": true,
                "skipUserAccountsRpcCalls": true
            }
        });
        
        for retry in 0..=self.max_retries {
            match self.execute_swap_request(&request_body).await {
                Ok(transaction) => {
                    debug!("✅ Transacción creada exitosamente");
                    return Ok(transaction);
                }
                Err(e) if retry < self.max_retries => {
                    warn!("⚠️ Swap intento {}/{} falló: {}", retry + 1, self.max_retries + 1, e);
                    tokio::time::sleep(Duration::from_millis(1000 * (retry + 1) as u64)).await;
                }
                Err(e) => return Err(CoreError::Jupiter(JupiterError::SwapError(e.to_string()))),
            }
        }
        
        unreachable!()
    }
    
    /// Ejecutar request de swap
    async fn execute_swap_request(&self, request_body: &Value) -> Result<SwapTransaction> {
        let response = timeout(
            self.timeout,
            self.http_client.post(&self.swap_url)
                .json(request_body)
                .send()
        ).await
            .map_err(|_| anyhow!("Request timeout"))?
            .map_err(|e| anyhow!("Network error: {}", e))?;
            
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API error {}: {}", response.status(), error_text));
        }
        
        let data: Value = response.json().await
            .map_err(|e| anyhow!("JSON parse error: {}", e))?;
            
        Ok(SwapTransaction {
            transaction: data["swapTransaction"].as_str()
                .ok_or_else(|| anyhow!("Missing transaction"))?
                .to_string(),
            last_valid_block_height: data["lastValidBlockHeight"].as_u64(),
        })
    }
}

/// Configuración de Jupiter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterConfig {
    pub quote_url: String,
    pub price_url: String,
    pub swap_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            quote_url: "https://quote-api.jup.ag/v6/quote".to_string(),
            price_url: "https://api.jup.ag/price/v2".to_string(),
            swap_url: "https://quote-api.jup.ag/v6/swap".to_string(),
            timeout_seconds: 10,
            max_retries: 3,
        }
    }
}

/// Request para obtener quote
#[derive(Debug, Clone)]
pub struct QuoteRequest {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount: u64,
    pub slippage_bps: u16,
    pub platform_fee_bps: Option<u16>,
    pub only_direct_routes: bool,
    pub max_accounts: Option<u32>,
}

/// Respuesta de quote
#[derive(Debug, Clone)]
pub struct SwapQuote {
    pub input_mint: Pubkey,
    pub in_amount: u64,
    pub output_mint: Pubkey,
    pub out_amount: u64,
    pub price_impact_pct: f64,
    pub route_plan: Vec<RouteStep>,
    pub slippage_bps: u16,
    pub platform_fee_bps: Option<u16>,
    pub route_data: Value, // Raw data para crear transaction
}

/// Paso en la ruta de swap
#[derive(Debug, Clone)]
pub struct RouteStep {
    pub swap_info: String,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub fee_amount: u64,
    pub fee_mint: Pubkey,
}

/// Precio de token
#[derive(Debug, Clone)]
pub struct TokenPrice {
    pub mint: Pubkey,
    pub price_usd: f64,
    pub price_sol: f64,
    pub volume_24h: f64,
    pub timestamp: u64,
    pub source: String,
}

/// Transacción de swap preparada
#[derive(Debug, Clone)]
pub struct SwapTransaction {
    pub transaction: String, // Base64 encoded transaction
    pub last_valid_block_height: Option<u64>,
}

/// Resultado de swap ejecutado
#[derive(Debug, Clone)]
pub struct SwapResult {
    pub success: bool,
    pub transaction_id: Option<String>,
    pub input_amount: u64,
    pub output_amount: u64,
    pub execution_time: Duration,
    pub gas_used: u64,
    pub error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    
    #[tokio::test]
    async fn test_jupiter_client_creation() {
        let config = JupiterConfig::default();
        let client = JupiterClient::new(&config);
        
        // Basic assertions
        assert_eq!(client.quote_url, config.quote_url);
        assert_eq!(client.max_retries, config.max_retries);
    }
    
    #[test]
    fn test_quote_request() {
        let request = QuoteRequest {
            input_mint: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
            output_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
            amount: 1_000_000_000, // 1 SOL
            slippage_bps: 100, // 1%
            platform_fee_bps: None,
            only_direct_routes: false,
            max_accounts: Some(20),
        };
        
        assert_eq!(request.amount, 1_000_000_000);
        assert_eq!(request.slippage_bps, 100);
    }
}
