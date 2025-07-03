//! Multi-DEX Fallback Manager
//! 
//! Provides fallback logic for trading across multiple DEXs on Solana DevNet/Mainnet
//! Primary: Orca -> Fallback: SPL Token-Swap -> Final: Jupiter

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};

use crate::shared::orca_client::{OrcaClient, OrcaQuoteRequest, OrcaSwapRequest};
use crate::shared::jupiter::{JupiterClient, QuoteRequest as JupiterQuoteRequest};
use crate::types::PlatformError;

/// Supported DEX providers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DexProvider {
    Orca,
    Raydium,
    SplSwap,
    Jupiter,
}

impl DexProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            DexProvider::Orca => "orca",
            DexProvider::Raydium => "raydium", 
            DexProvider::SplSwap => "spl-swap",
            DexProvider::Jupiter => "jupiter",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "orca" => Some(DexProvider::Orca),
            "raydium" => Some(DexProvider::Raydium),
            "spl-swap" | "spl_swap" => Some(DexProvider::SplSwap),
            "jupiter" => Some(DexProvider::Jupiter),
            _ => None,
        }
    }
}

/// Unified quote request across all DEXs
#[derive(Debug, Clone)]
pub struct UnifiedQuoteRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub slippage_bps: u16,
}

/// Unified quote response
#[derive(Debug, Clone)]
pub struct UnifiedQuoteResponse {
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: u64,
    pub out_amount: u64,
    pub slippage_bps: u16,
    pub price_impact_pct: f64,
    pub fee_mint: String,
    pub fee_amount: u64,
    pub dex_provider: DexProvider,
    pub route_plan: Vec<String>,
    pub quote_data: serde_json::Value, // Raw quote data for swap execution
}

/// Unified swap request
#[derive(Debug, Clone)]
pub struct UnifiedSwapRequest {
    pub quote: UnifiedQuoteResponse,
    pub user_public_key: String,
    pub wrap_unwrap_sol: bool,
    pub compute_unit_price_micro_lamports: Option<u64>,
}

/// Unified swap response
#[derive(Debug, Clone)]
pub struct UnifiedSwapResponse {
    pub swap_transaction: String, // Base64 encoded transaction
    pub last_valid_block_height: u64,
    pub prioritization_fee_lamports: u64,
    pub compute_unit_limit: u32,
    pub dex_provider: DexProvider,
}

/// Trait for DEX client abstraction
#[async_trait]
pub trait DexClient: Send + Sync {
    async fn get_quote(&self, request: UnifiedQuoteRequest) -> Result<UnifiedQuoteResponse>;
    async fn build_swap(&self, request: UnifiedSwapRequest) -> Result<UnifiedSwapResponse>;
    async fn health_check(&self) -> Result<bool>;
    fn get_provider(&self) -> DexProvider;
}

/// Multi-DEX fallback manager
pub struct DexFallbackManager {
    orca_client: Option<OrcaClient>,
    jupiter_client: Option<JupiterClient>,
    fallback_chain: Vec<DexProvider>,
    enable_fallback: bool,
    max_retries: u32,
}

impl DexFallbackManager {
    /// Create new fallback manager with optional clients
    pub fn new(
        orca_client: Option<OrcaClient>,
        jupiter_client: Option<JupiterClient>,
        fallback_chain: Vec<DexProvider>,
        enable_fallback: bool,
        max_retries: u32,
    ) -> Self {
        Self {
            orca_client,
            jupiter_client,
            fallback_chain,
            enable_fallback,
            max_retries,
        }
    }

    /// Get quote with fallback logic
    pub async fn get_quote_with_fallback(&self, request: UnifiedQuoteRequest) -> Result<UnifiedQuoteResponse> {
        info!("🔄 Getting quote with fallback: {} {} -> {} {}", 
              request.amount, request.input_mint, request.output_mint, request.slippage_bps);

        let mut last_error = None;
        
        for (attempt, &dex) in self.fallback_chain.iter().enumerate() {
            debug!("🎯 Attempt {} using DEX: {:?}", attempt + 1, dex);
            
            match self.try_get_quote(&request, dex).await {
                Ok(quote) => {
                    info!("✅ Quote successful from {}: {} -> {} (price impact: {:.4}%)", 
                          dex.as_str(), quote.in_amount, quote.out_amount, quote.price_impact_pct);
                    return Ok(quote);
                }
                Err(e) => {
                    warn!("❌ Quote failed from {}: {}", dex.as_str(), e);
                    last_error = Some(e);
                    
                    if !self.enable_fallback {
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| 
            anyhow!("All DEX providers failed and no fallback enabled")))
    }

    /// Get swap with fallback logic  
    pub async fn build_swap_with_fallback(&self, request: UnifiedSwapRequest) -> Result<UnifiedSwapResponse> {
        info!("🔄 Building swap with fallback for DEX: {:?}", request.quote.dex_provider);

        // Try to use the same DEX that provided the quote first
        match self.try_build_swap(&request, request.quote.dex_provider).await {
            Ok(swap) => {
                info!("✅ Swap built successfully with {}", request.quote.dex_provider.as_str());
                return Ok(swap);
            }
            Err(e) => {
                warn!("❌ Swap failed with {}: {}", request.quote.dex_provider.as_str(), e);
                
                if !self.enable_fallback {
                    return Err(e);
                }
            }
        }

        // Try fallback DEXs for swap
        let mut last_error = None;
        for &dex in &self.fallback_chain {
            if dex == request.quote.dex_provider {
                continue; // Already tried this one
            }
            
            debug!("🎯 Fallback swap attempt using DEX: {:?}", dex);
            
            // First get a new quote from this DEX
            let quote_request = UnifiedQuoteRequest {
                input_mint: request.quote.input_mint.clone(),
                output_mint: request.quote.output_mint.clone(),
                amount: request.quote.in_amount,
                slippage_bps: request.quote.slippage_bps,
            };
            
            match self.try_get_quote(&quote_request, dex).await {
                Ok(fallback_quote) => {
                    let fallback_swap_request = UnifiedSwapRequest {
                        quote: fallback_quote,
                        user_public_key: request.user_public_key.clone(),
                        wrap_unwrap_sol: request.wrap_unwrap_sol,
                        compute_unit_price_micro_lamports: request.compute_unit_price_micro_lamports,
                    };
                    
                    match self.try_build_swap(&fallback_swap_request, dex).await {
                        Ok(swap) => {
                            info!("✅ Fallback swap successful with {}", dex.as_str());
                            return Ok(swap);
                        }
                        Err(e) => {
                            warn!("❌ Fallback swap failed with {}: {}", dex.as_str(), e);
                            last_error = Some(e);
                        }
                    }
                }
                Err(e) => {
                    warn!("❌ Fallback quote failed with {}: {}", dex.as_str(), e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| 
            anyhow!("All DEX swap fallbacks failed")))
    }

    /// Try to get quote from specific DEX
    async fn try_get_quote(&self, request: &UnifiedQuoteRequest, dex: DexProvider) -> Result<UnifiedQuoteResponse> {
        match dex {
            DexProvider::Orca => {
                if let Some(orca) = &self.orca_client {
                    let orca_request = OrcaQuoteRequest {
                        input_mint: request.input_mint.clone(),
                        output_mint: request.output_mint.clone(),
                        amount: request.amount,
                        slippage_bps: request.slippage_bps,
                    };
                    
                    let orca_quote = orca.get_quote(orca_request).await?;
                    
                    Ok(UnifiedQuoteResponse {
                        input_mint: request.input_mint.clone(),
                        output_mint: request.output_mint.clone(),
                        in_amount: orca_quote.in_amount,
                        out_amount: orca_quote.out_amount,
                        slippage_bps: request.slippage_bps,
                        price_impact_pct: orca_quote.price_impact_pct,
                        fee_mint: orca_quote.fee_mint,
                        fee_amount: orca_quote.fee_amount,
                        dex_provider: DexProvider::Orca,
                        route_plan: orca_quote.route_plan,
                        quote_data: serde_json::to_value(&orca_quote)?,
                    })
                } else {
                    Err(anyhow!("Orca client not available"))
                }
            }
            DexProvider::Jupiter => {
                if let Some(jupiter) = &self.jupiter_client {
                    let jupiter_request = JupiterQuoteRequest {
                        inputMint: request.input_mint.clone(),
                        outputMint: request.output_mint.clone(),
                        amount: request.amount,
                        slippageBps: request.slippage_bps,
                    };
                    
                    let jupiter_quote = jupiter.get_quote(jupiter_request).await?;
                    
                    Ok(UnifiedQuoteResponse {
                        input_mint: request.input_mint.clone(),
                        output_mint: request.output_mint.clone(),
                        in_amount: jupiter_quote.in_amount(),
                        out_amount: jupiter_quote.out_amount(),
                        slippage_bps: request.slippage_bps,
                        price_impact_pct: jupiter_quote.price_impact_pct.unwrap_or(0.0),
                        fee_mint: "So11111111111111111111111111111111111111112".to_string(), // SOL
                        fee_amount: 5000, // Default Solana fee
                        dex_provider: DexProvider::Jupiter,
                        route_plan: jupiter_quote.route_plan.iter()
                            .map(|r| format!("{}%", r.swap_info.label))
                            .collect(),
                        quote_data: serde_json::to_value(&jupiter_quote)?,
                    })
                } else {
                    Err(anyhow!("Jupiter client not available"))
                }
            }
            DexProvider::Raydium => {
                // TODO: Implement Raydium client
                Err(anyhow!("Raydium client not yet implemented"))
            }
            DexProvider::SplSwap => {
                // TODO: Implement SPL Token-Swap client
                Err(anyhow!("SPL Token-Swap client not yet implemented"))
            }
        }
    }

    /// Try to build swap from specific DEX
    async fn try_build_swap(&self, request: &UnifiedSwapRequest, dex: DexProvider) -> Result<UnifiedSwapResponse> {
        match dex {
            DexProvider::Orca => {
                if let Some(orca) = &self.orca_client {
                    // Convert back to Orca quote
                    let orca_quote = serde_json::from_value(request.quote.quote_data.clone())?;
                    
                    let orca_swap_request = OrcaSwapRequest {
                        quote: orca_quote,
                        user_public_key: request.user_public_key.clone(),
                        wrap_unwrap_sol: request.wrap_unwrap_sol,
                        compute_unit_price_micro_lamports: request.compute_unit_price_micro_lamports,
                    };
                    
                    let orca_swap = orca.build_swap(orca_swap_request).await?;
                    
                    Ok(UnifiedSwapResponse {
                        swap_transaction: orca_swap.swap_transaction,
                        last_valid_block_height: orca_swap.last_valid_block_height,
                        prioritization_fee_lamports: orca_swap.prioritization_fee_lamports,
                        compute_unit_limit: orca_swap.compute_unit_limit,
                        dex_provider: DexProvider::Orca,
                    })
                } else {
                    Err(anyhow!("Orca client not available"))
                }
            }
            DexProvider::Jupiter => {
                if let Some(jupiter) = &self.jupiter_client {
                    // Convert back to Jupiter quote
                    let jupiter_quote = serde_json::from_value(request.quote.quote_data.clone())?;
                    
                    let jupiter_swap_request = crate::shared::jupiter::SwapRequest {
                        quoteResponse: jupiter_quote,
                        userPublicKey: request.user_public_key.clone(),
                        wrapAndUnwrapSol: Some(request.wrap_unwrap_sol),
                        computeUnitPriceMicroLamports: request.compute_unit_price_micro_lamports,
                        asLegacyTransaction: Some(false),
                        useSharedAccounts: Some(true),
                        feeAccount: None,
                        trackingAccount: None,
                        allowOptimizedWrappedSolTokenAccount: Some(true),
                        skipUserAccountsRpcCalls: Some(false),
                    };
                    
                    let jupiter_swap = jupiter.build_swap_transaction(jupiter_swap_request).await?;
                    
                    Ok(UnifiedSwapResponse {
                        swap_transaction: jupiter_swap.swapTransaction,
                        last_valid_block_height: jupiter_swap.lastValidBlockHeight,
                        prioritization_fee_lamports: jupiter_swap.prioritizationFeeLamports.unwrap_or(0),
                        compute_unit_limit: jupiter_swap.computeUnitLimit.unwrap_or(200_000),
                        dex_provider: DexProvider::Jupiter,
                    })
                } else {
                    Err(anyhow!("Jupiter client not available"))
                }
            }
            DexProvider::Raydium => {
                Err(anyhow!("Raydium swap not yet implemented"))
            }
            DexProvider::SplSwap => {
                Err(anyhow!("SPL Token-Swap not yet implemented"))
            }
        }
    }

    /// Health check all available DEXs
    pub async fn health_check_all(&self) -> HashMap<DexProvider, bool> {
        let mut results = HashMap::new();
        
        for &dex in &self.fallback_chain {
            let health = match dex {
                DexProvider::Orca => {
                    if let Some(orca) = &self.orca_client {
                        orca.health_check().await.unwrap_or(false)
                    } else {
                        false
                    }
                }
                DexProvider::Jupiter => {
                    if let Some(jupiter) = &self.jupiter_client {
                        // Jupiter doesn't have explicit health check, try a price call
                        jupiter.get_price("So11111111111111111111111111111111111111112").await.is_ok()
                    } else {
                        false
                    }
                }
                DexProvider::Raydium | DexProvider::SplSwap => false, // Not implemented yet
            };
            
            results.insert(dex, health);
        }
        
        results
    }

    /// Get best quote from all available DEXs
    pub async fn get_best_quote(&self, request: UnifiedQuoteRequest) -> Result<UnifiedQuoteResponse> {
        info!("🏆 Finding best quote across all DEXs");
        
        let mut quotes = Vec::new();
        let mut errors = Vec::new();
        
        for &dex in &self.fallback_chain {
            match self.try_get_quote(&request, dex).await {
                Ok(quote) => {
                    info!("💰 Quote from {}: {} out for {} in (impact: {:.4}%)", 
                          dex.as_str(), quote.out_amount, quote.in_amount, quote.price_impact_pct);
                    quotes.push(quote);
                }
                Err(e) => {
                    warn!("❌ Quote failed from {}: {}", dex.as_str(), e);
                    errors.push((dex, e));
                }
            }
        }
        
        if quotes.is_empty() {
            return Err(anyhow!("No quotes available from any DEX: {:?}", errors));
        }
        
        // Find best quote (highest output amount with reasonable price impact)
        let best_quote = quotes.into_iter()
            .filter(|q| q.price_impact_pct < 5.0) // Filter out high price impact
            .max_by(|a, b| a.out_amount.cmp(&b.out_amount))
            .unwrap_or_else(|| {
                // If all have high price impact, just take the highest output
                quotes.into_iter().max_by(|a, b| a.out_amount.cmp(&b.out_amount)).unwrap()
            });
        
        info!("🏆 Best quote selected from {}: {} -> {} (impact: {:.4}%)", 
              best_quote.dex_provider.as_str(), best_quote.in_amount, 
              best_quote.out_amount, best_quote.price_impact_pct);
        
        Ok(best_quote)
    }
}

impl Default for DexFallbackManager {
    fn default() -> Self {
        Self::new(
            None,
            None,
            vec![DexProvider::Orca, DexProvider::SplSwap, DexProvider::Jupiter],
            true,
            3,
        )
    }
}
