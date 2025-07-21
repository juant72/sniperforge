//! Real Trade Execution Engine - 100% Real Data Only
//!
//! Handles actual trade execution using Jupiter API with real blockchain data
//! No simulation, mock, virtual, or paper trading - only real trades

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Signature;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::Config;
use crate::shared::jupiter::{Jupiter, QuoteResponse};
use crate::shared::real_data_manager::RealDataManager;
use crate::shared::rpc_pool::RpcConnectionPool;

/// Real trading execution modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealTradingMode {
    DevNet,  // Real trades on Solana DevNet
    MainNet, // Real trades on Solana MainNet
}

/// Real trade request using actual tokens and amounts
#[derive(Debug, Clone)]
pub struct RealTradeRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: f64,
    pub slippage_bps: u16,
    pub wallet_name: String,
    pub max_price_impact: f64,
}

/// Real trade execution result from blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTradeResult {
    pub success: bool,
    pub transaction_signature: Option<String>,
    pub block_height: Option<u64>,
    pub input_amount: f64,
    pub output_amount: f64,
    pub actual_price: f64,
    pub actual_slippage: f64,
    pub network_fee: f64,
    pub execution_time_ms: u64,
    pub timestamp: u64,
    pub error_message: Option<String>,
}

/// Real trade execution engine - 100% blockchain based
#[derive(Debug)]
pub struct RealTradeExecutor {
    jupiter: Jupiter,
    rpc_pool: RpcConnectionPool,
    real_data_manager: RealDataManager,
    trading_mode: RealTradingMode,
    config: Config,
}

impl RealTradeExecutor {
    /// Create new real trade executor
    pub fn new(
        jupiter: Jupiter,
        rpc_pool: RpcConnectionPool,
        real_data_manager: RealDataManager,
        trading_mode: RealTradingMode,
        config: Config,
    ) -> Self {
        info!(
            "🚀 Real Trade Executor initialized - Mode: {:?}",
            trading_mode
        );

        Self {
            jupiter,
            rpc_pool,
            real_data_manager,
            trading_mode,
            config,
        }
    }

    /// Execute real trade on blockchain
    pub async fn execute_real_trade(
        &mut self,
        request: RealTradeRequest,
    ) -> Result<RealTradeResult> {
        let start_time = SystemTime::now();
        let timestamp = start_time.duration_since(UNIX_EPOCH)?.as_secs();

        info!(
            "🎯 Executing REAL trade: {} -> {} (Amount: {:.6})",
            &request.input_mint[..8],
            &request.output_mint[..8],
            request.amount
        );

        // Validate wallet balance using real blockchain data
        self.validate_real_balance(&request).await?;

        // Get real quote from Jupiter
        let quote = self.get_real_quote(&request).await?;

        // Validate quote parameters
        self.validate_quote_safety(&quote, &request)?;

        // Execute real swap on blockchain
        let result = self.execute_blockchain_swap(&quote, &request).await?;

        let execution_time = start_time.elapsed()?.as_millis() as u64;

        let trade_result = RealTradeResult {
            success: result.success,
            transaction_signature: result.transaction_signature,
            block_height: result.block_height,
            input_amount: request.amount,
            output_amount: result.output_amount,
            actual_price: if result.output_amount > 0.0 {
                result.output_amount / request.amount
            } else {
                0.0
            },
            actual_slippage: result.actual_slippage,
            network_fee: result.network_fee,
            execution_time_ms: execution_time,
            timestamp,
            error_message: result.error_message,
        };

        if trade_result.success {
            info!(
                "✅ REAL trade completed successfully: {} in {:.2}s",
                trade_result
                    .transaction_signature
                    .as_ref()
                    .unwrap_or(&"unknown".to_string())[..16]
                    .to_string(),
                execution_time as f64 / 1000.0
            );
        } else {
            error!(
                "❌ REAL trade failed: {}",
                trade_result
                    .error_message
                    .as_ref()
                    .unwrap_or(&"Unknown error".to_string())
            );
        }

        Ok(trade_result)
    }

    /// Validate wallet has sufficient real balance
    async fn validate_real_balance(&mut self, request: &RealTradeRequest) -> Result<()> {
        debug!(
            "💰 Validating REAL wallet balance for: {}",
            &request.wallet_name
        );

        // Get wallet address from config
        // Note: Wallet access should be through wallet_manager, not config directly
        // For now, return an error indicating wallet manager integration needed
        return Err(anyhow!(
            "Wallet '{}' access requires wallet manager integration",
            request.wallet_name
        ));
    }

    /// Get real quote from Jupiter API
    async fn get_real_quote(&self, request: &RealTradeRequest) -> Result<QuoteResponse> {
        debug!("📊 Getting REAL quote from Jupiter API");

        let quote_result = timeout(
            Duration::from_secs(15),
            self.jupiter.get_quote(
                &request.input_mint,
                &request.output_mint,
                request.amount,
                request.slippage_bps,
            ),
        )
        .await;

        match quote_result {
            Ok(Ok(quote)) => {
                info!(
                    "✅ Real quote received: {:.6} -> {:.6} (Price Impact: {:.2}%)",
                    quote.in_amount(),
                    quote.out_amount(),
                    quote.price_impact_pct() * 100.0
                );
                Ok(quote)
            }
            Ok(Err(e)) => {
                error!("❌ Jupiter quote error: {}", e);
                Err(anyhow!("Failed to get real quote: {}", e))
            }
            Err(_) => {
                error!("❌ Jupiter quote timeout");
                Err(anyhow!("Quote request timeout"))
            }
        }
    }

    /// Validate quote safety parameters
    fn validate_quote_safety(
        &self,
        quote: &QuoteResponse,
        request: &RealTradeRequest,
    ) -> Result<()> {
        debug!("🛡️ Validating quote safety parameters");

        // Check price impact
        if quote.price_impact_pct() > request.max_price_impact {
            return Err(anyhow!(
                "Price impact too high: {:.2}% > {:.2}% limit",
                quote.price_impact_pct() * 100.0,
                request.max_price_impact * 100.0
            ));
        }

        // Check minimum output amount
        if quote.out_amount() <= 0.0 {
            return Err(anyhow!("Invalid output amount: {}", quote.out_amount()));
        }

        // Check route exists
        if quote.routePlan.is_empty() {
            return Err(anyhow!("No valid route found for swap"));
        }

        info!(
            "✅ Quote safety validated - Price Impact: {:.2}%",
            quote.price_impact_pct() * 100.0
        );
        Ok(())
    }

    /// Execute real swap on blockchain
    async fn execute_blockchain_swap(
        &self,
        quote: &QuoteResponse,
        request: &RealTradeRequest,
    ) -> Result<BlockchainSwapResult> {
        info!("⚡ Executing REAL swap on blockchain");

        let swap_result = timeout(
            Duration::from_secs(60),
            self.jupiter.execute_swap(quote, &request.wallet_name),
        )
        .await;

        match swap_result {
            Ok(Ok(result)) => {
                // Verify transaction on blockchain
                if let Some(signature) = &result.transaction_signature {
                    self.verify_transaction_on_chain(signature).await?;
                }

                Ok(BlockchainSwapResult {
                    success: result.success,
                    transaction_signature: result.transaction_signature,
                    block_height: None, // Would get from transaction confirmation
                    output_amount: result.output_amount,
                    actual_slippage: result.actual_slippage,
                    network_fee: result.fee_amount,
                    error_message: if result.success {
                        None
                    } else {
                        Some("Swap failed".to_string())
                    },
                })
            }
            Ok(Err(e)) => {
                error!("❌ Blockchain swap error: {}", e);
                Ok(BlockchainSwapResult {
                    success: false,
                    transaction_signature: None,
                    block_height: None,
                    output_amount: 0.0,
                    actual_slippage: 0.0,
                    network_fee: 0.0,
                    error_message: Some(format!("Swap error: {}", e)),
                })
            }
            Err(_) => {
                error!("❌ Blockchain swap timeout");
                Ok(BlockchainSwapResult {
                    success: false,
                    transaction_signature: None,
                    block_height: None,
                    output_amount: 0.0,
                    actual_slippage: 0.0,
                    network_fee: 0.0,
                    error_message: Some("Swap timeout".to_string()),
                })
            }
        }
    }

    /// Verify transaction was actually executed on blockchain
    async fn verify_transaction_on_chain(&self, signature: &str) -> Result<()> {
        debug!(
            "🔍 Verifying transaction on blockchain: {}",
            &signature[..16]
        );

        let signature = signature
            .parse::<Signature>()
            .map_err(|e| anyhow!("Invalid transaction signature: {}", e))?;

        let verification_result = timeout(
            Duration::from_secs(30),
            self.rpc_pool.get_transaction(&signature),
        )
        .await;

        match verification_result {
            Ok(Ok(_transaction)) => {
                info!(
                    "✅ Transaction verified on blockchain: {}",
                    signature.to_string()
                );
                Ok(())
            }
            Ok(Err(e)) => {
                error!("❌ Transaction verification failed: {}", e);
                Err(anyhow!("Failed to verify transaction: {}", e))
            }
            Err(_) => {
                warn!("⚠️ Transaction verification timeout (may still be pending)");
                Ok(()) // Don't fail on verification timeout - transaction may still succeed
            }
        }
    }

    /// Get real trading statistics
    pub async fn get_real_trading_stats(&mut self) -> Result<RealTradingStats> {
        debug!("📊 Calculating REAL trading statistics");

        // Get real portfolio metrics from blockchain
        let wallet_address = "trading"; // Use default trading wallet
        let metrics = self
            .real_data_manager
            .get_real_portfolio_metrics(wallet_address)
            .await?;

        Ok(RealTradingStats {
            total_trades: metrics.total_transactions,
            total_volume_usd: metrics.total_value_usd,
            total_fees_paid: metrics.total_fees_paid,
            win_rate: metrics.actual_win_rate,
            largest_gain: metrics.largest_gain,
            largest_loss: metrics.largest_loss,
            current_balance_usd: metrics.total_value_usd,
            last_updated: metrics.last_updated,
        })
    }

    /// Validate all systems are using real data
    pub async fn validate_real_data_only(&self) -> Result<()> {
        info!("🔍 Validating ALL systems use real data only");

        // Validate Jupiter is configured for real trading
        if !self.jupiter.is_configured() {
            return Err(anyhow!("Jupiter not configured for real trading"));
        }

        // Validate RPC is connected to real network
        if !self.rpc_pool.is_healthy().await? {
            return Err(anyhow!("RPC not connected to real network"));
        }

        // Validate real data manager
        self.real_data_manager.validate_real_data_only().await?;

        info!("✅ ALL systems validated for real data only");
        Ok(())
    }
}

/// Result from blockchain swap execution
#[derive(Debug)]
struct BlockchainSwapResult {
    success: bool,
    transaction_signature: Option<String>,
    block_height: Option<u64>,
    output_amount: f64,
    actual_slippage: f64,
    network_fee: f64,
    error_message: Option<String>,
}

/// Real trading statistics from blockchain
#[derive(Debug, Serialize, Deserialize)]
pub struct RealTradingStats {
    pub total_trades: u32,
    pub total_volume_usd: f64,
    pub total_fees_paid: f64,
    pub win_rate: f64,
    pub largest_gain: f64,
    pub largest_loss: f64,
    pub current_balance_usd: f64,
    pub last_updated: u64,
}
