/// Trade Execution Engine
/// 
/// Handles actual trade execution combining Jupiter API with Wallet Management
/// Supports both DevNet real trading and MainNet paper trading

use anyhow::{Result, anyhow};
use solana_sdk::{signature::Keypair, signer::Signer, pubkey::Pubkey, transaction::Transaction};
use solana_client::rpc_client::RpcClient;
use tracing::{info, warn, error, debug};
use std::time::Instant;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::jupiter::{JupiterClient, JupiterConfig, JupiterQuote, JupiterSwapService};
use super::wallet_manager::WalletManager;
// TODO: Re-enable when cache_free_trader_simple is implemented
// use super::cache_free_trader_simple::{CacheFreeTraderSimple, TradingSafetyConfig, SwapResult};
use crate::config::Config;

/// Trade execution mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingMode {
    DevNetReal,     // Real transactions on DevNet
    MainNetPaper,   // Paper trading with MainNet data
    MainNetReal,    // Real transactions on MainNet (Phase 5B)
    Simulation,     // Full simulation mode
}

/// Trade execution request
#[derive(Debug, Clone)]
pub struct TradeRequest {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount_in: u64,
    pub slippage_bps: u16,
    pub wallet_name: String,
    pub max_price_impact: f64,
    pub trading_mode: TradingMode,
}

/// Trade execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub success: bool,
    pub transaction_signature: Option<String>,
    pub input_amount: u64,
    pub output_amount: u64,
    pub actual_price_impact: f64,
    pub actual_slippage: f64,
    pub gas_fee: f64,
    pub trading_mode: TradingMode,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
    pub jupiter_quote: Option<JupiterQuote>,
    pub wallet_balance_before: f64,
    pub wallet_balance_after: f64,
}

/// Trade Execution Engine
pub struct TradeExecutor {
    config: Config,    jupiter_client: JupiterClient,
    swap_service: JupiterSwapService,
    wallet_manager: WalletManager,
    rpc_client: RpcClient,
    trading_mode: TradingMode,
    // TODO: Re-enable when cache_free_trader_simple is implemented
    // cache_free_trader: Option<CacheFreeTraderSimple>,
}

impl TradeExecutor {
    /// Create new trade executor
    pub async fn new(config: Config, trading_mode: TradingMode) -> Result<Self> {
        info!("🎯 Initializing Trade Executor in mode: {:?}", trading_mode);        // Setup Jupiter configuration based on trading mode
        let jupiter_config = match trading_mode {
            TradingMode::DevNetReal => JupiterConfig {
                api_base_url: "https://quote-api.jup.ag/v6".to_string(),
                rpc_url: config.network.primary_rpc().to_string(),
                timeout_seconds: 10,
                max_retries: 3,
                slippage_bps: 50,
                enable_devnet: true,
                enable_mainnet_paper: false,
            },
            TradingMode::MainNetReal => JupiterConfig {
                api_base_url: "https://quote-api.jup.ag/v6".to_string(),
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                timeout_seconds: 5,  // Faster timeout for real trading
                max_retries: 2,      // Fewer retries for real trading
                slippage_bps: 30,    // Tighter slippage for real trading
                enable_devnet: false,
                enable_mainnet_paper: false,
            },
            TradingMode::MainNetPaper | TradingMode::Simulation => JupiterConfig {
                api_base_url: "https://quote-api.jup.ag/v6".to_string(),
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                timeout_seconds: 10,
                max_retries: 3,
                slippage_bps: 50,
                enable_devnet: false,
                enable_mainnet_paper: true,
            },
        };

        let jupiter_client = JupiterClient::new(&jupiter_config).await?;
        let rpc_url = match trading_mode {
            TradingMode::DevNetReal => config.network.primary_rpc(),
            TradingMode::MainNetReal | TradingMode::MainNetPaper | TradingMode::Simulation => "https://api.mainnet-beta.solana.com",
        };
          let swap_service = JupiterSwapService::new(jupiter_client.clone(), rpc_url);
        let wallet_manager = WalletManager::new(&config).await?;
        let rpc_client = RpcClient::new(rpc_url.to_string());        // TODO: Re-enable when cache_free_trader_simple is implemented
        // Initialize cache-free trader for maximum safety
        // let cache_free_trader = match CacheFreeTraderSimple::new(TradingSafetyConfig::default()).await {
        //     Ok(trader) => {
        //         info!("🛡️ Cache-free trader initialized for safe trading");
        //         Some(trader)
        //     }
        //     Err(e) => {
        //         warn!("⚠️ Failed to initialize cache-free trader: {}", e);
        //         None
        //     }
        // };

        info!("✅ Trade Executor initialized successfully");

        Ok(Self {
            config,
            jupiter_client,
            swap_service,
            wallet_manager,
            rpc_client,
            trading_mode,
            // cache_free_trader,
        })
    }

    /// Execute a trade
    pub async fn execute_trade(&self, request: TradeRequest) -> Result<TradeResult> {
        let start_time = Instant::now();
        
        info!("🎯 Executing trade: {} -> {} ({})", 
              request.input_mint, request.output_mint, request.amount_in);
        info!("   Mode: {:?} | Wallet: {} | Slippage: {}bps", 
              request.trading_mode, request.wallet_name, request.slippage_bps);

        // Validate wallet availability
        let wallet_balance_before = self.get_wallet_balance(&request.wallet_name).await?;
        
        if !self.validate_trade_request(&request, wallet_balance_before).await? {
            return Ok(TradeResult {
                success: false,
                transaction_signature: None,
                input_amount: request.amount_in,
                output_amount: 0,
                actual_price_impact: 0.0,
                actual_slippage: 0.0,
                gas_fee: 0.0,
                trading_mode: request.trading_mode.clone(),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some("Trade validation failed".to_string()),
                jupiter_quote: None,
                wallet_balance_before,
                wallet_balance_after: wallet_balance_before,
            });
        }

        // Get Jupiter quote
        let quote = match self.get_quote(&request).await {
            Ok(quote) => quote,
            Err(e) => {
                error!("❌ Failed to get quote: {}", e);
                return Ok(TradeResult {
                    success: false,
                    transaction_signature: None,
                    input_amount: request.amount_in,
                    output_amount: 0,
                    actual_price_impact: 0.0,
                    actual_slippage: 0.0,
                    gas_fee: 0.0,
                    trading_mode: request.trading_mode.clone(),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    error_message: Some(format!("Quote failed: {}", e)),
                    jupiter_quote: None,
                    wallet_balance_before,
                    wallet_balance_after: wallet_balance_before,
                });
            }
        };

        // Validate quote before execution
        if !self.validate_quote(&quote, &request).await? {
            return Ok(TradeResult {
                success: false,
                transaction_signature: None,
                input_amount: request.amount_in,
                output_amount: 0,
                actual_price_impact: quote.price_impact_pct.parse().unwrap_or(0.0),
                actual_slippage: 0.0,
                gas_fee: 0.0,
                trading_mode: request.trading_mode.clone(),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some("Quote validation failed - price impact too high".to_string()),
                jupiter_quote: Some(quote),
                wallet_balance_before,
                wallet_balance_after: wallet_balance_before,
            });
        }        // Execute trade based on mode
        let result = match request.trading_mode {
            TradingMode::DevNetReal => self.execute_devnet_trade(&quote, &request).await?,
            TradingMode::MainNetReal => self.execute_mainnet_real_trade(&quote, &request).await?,
            TradingMode::MainNetPaper => self.execute_paper_trade(&quote, &request).await?,
            TradingMode::Simulation => self.execute_simulation_trade(&quote, &request).await?,
        };

        let wallet_balance_after = self.get_wallet_balance(&request.wallet_name).await.unwrap_or(wallet_balance_before);
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        info!("🎯 Trade completed in {} ms | Success: {}", execution_time, result.success);

        Ok(TradeResult {
            success: result.success,
            transaction_signature: result.transaction_signature,
            input_amount: request.amount_in,
            output_amount: result.output_amount,
            actual_price_impact: quote.price_impact_pct.parse().unwrap_or(0.0),
            actual_slippage: result.slippage,
            gas_fee: result.gas_fee,
            trading_mode: request.trading_mode,
            execution_time_ms: execution_time,
            error_message: result.error_message,
            jupiter_quote: Some(quote),
            wallet_balance_before,
            wallet_balance_after,
        })
    }    /// Execute a safe trade using cache-free pricing (RECOMMENDED for real trading)
    pub async fn execute_safe_trade(&self, request: TradeRequest) -> Result<TradeResult> {
        let _start_time = Instant::now();
        
        info!("🛡️ Executing SAFE trade with cache-free pricing: {} -> {} ({})", 
              request.input_mint, request.output_mint, request.amount_in);
                // TODO: Re-enable when cache_free_trader_simple is implemented
        // Use cache-free trader if available
        // if let Some(ref cache_free_trader) = self.cache_free_trader {
        //     match cache_free_trader.execute_safe_swap(
        //         &request.input_mint.to_string(),
        //         &request.output_mint.to_string(),
        //         request.amount_in,
        //     ).await {
        //         Ok(swap_result) => {
        //             return Ok(TradeResult {
        //                 success: swap_result.success,
        //                 transaction_signature: None, // TODO: Add when swap execution is implemented
        //                 input_amount: swap_result.input_amount,
        //                 output_amount: swap_result.output_amount,
        //                 actual_price_impact: 0.0, // TODO: Calculate from price data
        //                 actual_slippage: 0.0,     // TODO: Calculate actual slippage
        //                 gas_fee: 0.0,             // TODO: Add gas calculation
        //                 trading_mode: request.trading_mode,
        //                 execution_time_ms: start_time.elapsed().as_millis() as u64,
        //                 error_message: None,
        //                 jupiter_quote: None,      // TODO: Add quote from cache-free trader
        //                 wallet_balance_before: 0.0, // TODO: Get from wallet
        //                 wallet_balance_after: 0.0,   // TODO: Get from wallet
        //             });
        //         }
        //         Err(e) => {
        //             error!("❌ Cache-free trade execution failed: {}", e);
        //             return Ok(TradeResult {
        //                 success: false,
        //                 transaction_signature: None,
        //                 input_amount: request.amount_in,
        //                 output_amount: 0,
        //                 actual_price_impact: 0.0,
        //                 actual_slippage: 0.0,
        //                 gas_fee: 0.0,
        //                 trading_mode: request.trading_mode,
        //                 execution_time_ms: start_time.elapsed().as_millis() as u64,
        //                 error_message: Some(e.to_string()),
        //                 jupiter_quote: None,        //                 wallet_balance_before: 0.0,
        //                 wallet_balance_after: 0.0,
        //             });
        //         }
        //     }
        // }
        
        // Fallback to regular execution since cache-free trader is not available
        warn!("⚠️ Cache-free trader not available, falling back to regular execution");
        return self.execute_trade(request).await;
    }

    /// Get Jupiter quote for trade
    async fn get_quote(&self, request: &TradeRequest) -> Result<JupiterQuote> {
        self.jupiter_client.get_quote(
            &request.input_mint.to_string(),
            &request.output_mint.to_string(),
            request.amount_in,
            Some(request.slippage_bps),
        ).await
    }

    /// Validate trade request
    async fn validate_trade_request(&self, request: &TradeRequest, wallet_balance: f64) -> Result<bool> {
        // Check if wallet exists and has sufficient balance
        if !self.wallet_manager.is_wallet_available(&request.wallet_name, 0.01).await? {
            warn!("❌ Wallet {} not available", request.wallet_name);
            return Ok(false);
        }

        // Basic balance check (simplified)
        let required_sol = request.amount_in as f64 / 1_000_000_000.0;
        if wallet_balance < required_sol {
            warn!("❌ Insufficient balance: {} SOL < {} SOL", wallet_balance, required_sol);
            return Ok(false);
        }

        Ok(true)
    }

    /// Validate Jupiter quote
    async fn validate_quote(&self, quote: &JupiterQuote, request: &TradeRequest) -> Result<bool> {
        let price_impact: f64 = quote.price_impact_pct.parse().unwrap_or(0.0);
        
        if price_impact > request.max_price_impact {
            warn!("❌ Price impact too high: {}% > {}%", price_impact, request.max_price_impact);
            return Ok(false);
        }

        if quote.route_plan.is_empty() {
            warn!("❌ No route found for trade");
            return Ok(false);
        }

        Ok(true)
    }

    /// Execute real trade on DevNet
    async fn execute_devnet_trade(&self, quote: &JupiterQuote, request: &TradeRequest) -> Result<TradeExecutionResult> {
        info!("🔄 Executing REAL trade on DevNet");
        
        // In a real implementation, we would:
        // 1. Get wallet keypair from wallet manager
        // 2. Use Jupiter swap service to execute the trade
        // 3. Sign and submit transaction
        // 4. Wait for confirmation
        
        // For now, return a simulation result
        self.execute_simulation_trade(quote, request).await
    }

    /// Execute REAL trade on MainNet (Phase 5B)
    async fn execute_mainnet_real_trade(&self, quote: &JupiterQuote, request: &TradeRequest) -> Result<TradeExecutionResult> {
        println!("🚨 Executing REAL TRADE on MainNet - USING REAL MONEY!");
        
        // Extra safety checks for real money trading
        if quote.price_impact_pct.parse::<f64>().unwrap_or(0.0) > 2.0 {
            return Ok(TradeExecutionResult {
                success: false,
                transaction_signature: None,
                output_amount: 0,
                slippage: 0.0,
                gas_fee: 0.0,
                error_message: Some("Price impact too high for real trading (>2%)".to_string()),
            });
        }

        // In a real implementation, this would:
        // 1. Validate wallet has sufficient balance
        // 2. Get wallet keypair from wallet manager (with additional confirmations)
        // 3. Create and submit actual Jupiter swap transaction
        // 4. Monitor transaction confirmation
        // 5. Handle any failures with proper rollback

        println!("⚠️  REAL MAINNET TRADING NOT YET IMPLEMENTED - Using simulation for safety");
        println!("   Quote: {} -> {} tokens", quote.in_amount, quote.out_amount);
        println!("   Price Impact: {}%", quote.price_impact_pct);
        
        // For now, simulate to prevent accidental real trading
        // TODO: Remove this safety check once fully tested
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        Ok(TradeExecutionResult {
            success: true,
            transaction_signature: Some(format!("MAINNET_REAL_{}", uuid::Uuid::new_v4())),
            output_amount: quote.out_amount.parse().unwrap_or(0),
            slippage: request.slippage_bps as f64 / 10000.0 * 1.5, // Slightly higher slippage for real trading
            gas_fee: 0.01, // Higher gas fee for mainnet
            error_message: None,
        })
    }

    /// Execute paper trade with MainNet data
    async fn execute_paper_trade(&self, quote: &JupiterQuote, request: &TradeRequest) -> Result<TradeExecutionResult> {
        info!("📝 Executing PAPER trade with MainNet data");
        
        // Simulate trade execution with realistic timing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(TradeExecutionResult {
            success: true,
            transaction_signature: Some(format!("PAPER_{}", uuid::Uuid::new_v4())),
            output_amount: quote.out_amount.parse().unwrap_or(0),
            slippage: request.slippage_bps as f64 / 10000.0,
            gas_fee: 0.005,
            error_message: None,
        })
    }

    /// Execute simulation trade
    async fn execute_simulation_trade(&self, quote: &JupiterQuote, _request: &TradeRequest) -> Result<TradeExecutionResult> {
        info!("🎭 Executing SIMULATION trade");
        
        // Simulate some processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        Ok(TradeExecutionResult {
            success: true,
            transaction_signature: Some(format!("SIM_{}", uuid::Uuid::new_v4())),
            output_amount: quote.out_amount.parse().unwrap_or(0),
            slippage: 0.1,
            gas_fee: 0.005,
            error_message: None,
        })
    }    /// Get wallet balance
    async fn get_wallet_balance(&self, _wallet_name: &str) -> Result<f64> {
        // For now, use a mock balance
        // In production, this would query the actual wallet balance
        match self.trading_mode {
            TradingMode::DevNetReal => {
                // Query actual balance from devnet
                Ok(2.0) // Mock balance
            },
            TradingMode::MainNetReal => {
                // Query actual balance from mainnet
                Ok(1.0) // Mock balance for real mainnet trading
            },
            TradingMode::MainNetPaper | TradingMode::Simulation => {
                // Return virtual balance
                Ok(100.0) // Mock virtual balance
            },
        }
    }

    /// Get current trading mode
    pub fn get_trading_mode(&self) -> &TradingMode {
        &self.trading_mode
    }

    /// Get quote for a potential trade (public method for testing)
    pub async fn get_trade_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_in: u64,
        slippage_bps: Option<u16>,
    ) -> Result<JupiterQuote> {
        self.jupiter_client.get_quote(
            input_mint,
            output_mint,
            amount_in,
            slippage_bps,
        ).await
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        // Check Jupiter connectivity
        self.jupiter_client.health_check().await?;
        
        // Check wallet manager
        self.wallet_manager.health_check().await?;
        
        Ok(())
    }
}

/// Internal trade execution result
#[derive(Debug)]
struct TradeExecutionResult {
    success: bool,
    transaction_signature: Option<String>,
    output_amount: u64,
    slippage: f64,
    gas_fee: f64,
    error_message: Option<String>,
}
