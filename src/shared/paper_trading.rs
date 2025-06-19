/// Paper Trading Engine - Mainnet Data, Zero Risk
/// 
/// Este módulo permite trading simulado usando datos reales de mainnet
/// sin ejecutar transacciones reales ni arriesgar dinero

use anyhow::{Result, anyhow};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::shared::jupiter::client::JupiterClient;
use crate::shared::jupiter::{JupiterConfig, types::JupiterQuote};
use crate::shared::syndica_websocket::{SyndicaWebSocketClient, SyndicaConfig};

/// Configuración para paper trading
#[derive(Debug, Clone)]
pub struct PaperTradingConfig {
    /// Balance inicial simulado en SOL
    pub initial_sol_balance: f64,
    /// Balance inicial simulado en USDC
    pub initial_usdc_balance: f64,
    /// Comisiones simuladas (bps)
    pub simulated_fee_bps: u16,
    /// Slippage simulado (%)
    pub simulated_slippage_percent: f64,
    /// Usar datos reales de mainnet
    pub use_mainnet_data: bool,
    /// Latencia simulada de ejecución (ms)
    pub execution_latency_ms: u64,
}

impl Default for PaperTradingConfig {
    fn default() -> Self {
        Self {
            initial_sol_balance: 10.0,        // 10 SOL inicial
            initial_usdc_balance: 2000.0,     // $2000 USDC inicial
            simulated_fee_bps: 30,            // 0.3% fees (realistic)
            simulated_slippage_percent: 0.1,  // 0.1% slippage
            use_mainnet_data: true,           // Usar datos reales
            execution_latency_ms: 150,        // Latencia realista
        }
    }
}

/// Balance de tokens simulado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperBalance {
    pub token_mint: String,
    pub symbol: String,
    pub amount: f64,
    pub value_usd: f64,
    pub last_updated: u64,
}

/// Resultado de trade simulado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTradeResult {
    pub trade_id: String,
    pub timestamp: u64,
    pub input_token: String,
    pub output_token: String,
    pub input_amount: f64,
    pub output_amount: f64,
    pub input_price: f64,
    pub output_price: f64,
    pub simulated_fee: f64,
    pub simulated_slippage: f64,
    pub total_latency: Duration,
    pub profit_loss_usd: f64,
    pub is_profitable: bool,
}

/// Portfolio simulado
#[derive(Debug, Clone)]
pub struct PaperPortfolio {
    pub balances: HashMap<String, PaperBalance>,
    pub total_value_usd: f64,
    pub initial_value_usd: f64,
    pub profit_loss_usd: f64,
    pub profit_loss_percent: f64,
    pub trade_count: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
}

/// Paper Trading Engine
#[derive(Debug)]
pub struct PaperTradingEngine {
    config: PaperTradingConfig,
    portfolio: PaperPortfolio,
    jupiter_client: JupiterClient,
    syndica_client: Option<SyndicaWebSocketClient>,
    trade_history: Vec<PaperTradeResult>,
}

impl PaperTradingEngine {
    /// Crear nuevo paper trading engine
    pub async fn new(config: PaperTradingConfig) -> Result<Self> {
        info!("📊 Initializing Paper Trading Engine (Mainnet Data)");
        info!("   Initial SOL: {}", config.initial_sol_balance);
        info!("   Initial USDC: ${}", config.initial_usdc_balance);
        info!("   Using mainnet data: {}", config.use_mainnet_data);
          // Configurar Jupiter para mainnet
        let jupiter_config = if config.use_mainnet_data {
            JupiterConfig {
                api_base_url: "https://quote-api.jup.ag/v6".to_string(),
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                timeout_seconds: 5,
                max_retries: 3,
                slippage_bps: 50, // 0.5% slippage
                enable_devnet: false,
                enable_mainnet_paper: true, // Paper trading en mainnet
            }
        } else {
            JupiterConfig::default()
        };
        
        let jupiter_client = JupiterClient::new(&jupiter_config).await?;
        
        // Configurar Syndica para mainnet
        let syndica_config = if config.use_mainnet_data {
            SyndicaConfig {
                endpoint: "wss://solana-mainnet.api.syndica.io".to_string(),
                ..SyndicaConfig::default()
            }
        } else {
            SyndicaConfig::default()
        };
        
        let syndica_client = SyndicaWebSocketClient::new(syndica_config).await?;
        
        // Inicializar portfolio
        let mut balances = HashMap::new();
        
        // SOL balance
        balances.insert("So11111111111111111111111111111111111111112".to_string(), PaperBalance {
            token_mint: "So11111111111111111111111111111111111111112".to_string(),
            symbol: "SOL".to_string(),
            amount: config.initial_sol_balance,
            value_usd: config.initial_sol_balance * 180.0, // Precio estimado
            last_updated: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        });
        
        // USDC balance
        balances.insert("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), PaperBalance {
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: "USDC".to_string(),
            amount: config.initial_usdc_balance,
            value_usd: config.initial_usdc_balance,
            last_updated: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        });
        
        let initial_value = config.initial_sol_balance * 180.0 + config.initial_usdc_balance;
        
        let portfolio = PaperPortfolio {
            balances,
            total_value_usd: initial_value,
            initial_value_usd: initial_value,
            profit_loss_usd: 0.0,
            profit_loss_percent: 0.0,
            trade_count: 0,
            winning_trades: 0,
            losing_trades: 0,
        };
        
        Ok(Self {
            config,
            portfolio,
            jupiter_client,
            syndica_client: Some(syndica_client),
            trade_history: Vec::new(),
        })
    }
    
    /// Conectar a feeds de datos de mainnet
    pub async fn connect_mainnet_feeds(&mut self) -> Result<()> {
        info!("🌐 Connecting to mainnet data feeds...");
        
        if let Some(ref mut syndica) = self.syndica_client {
            syndica.connect().await?;
            info!("✅ Connected to Syndica mainnet WebSocket");
        }
        
        // Test Jupiter mainnet connection
        match self.jupiter_client.get_tokens().await {
            Ok(tokens) => {
                info!("✅ Connected to Jupiter mainnet API ({} tokens available)", tokens.len());
            }
            Err(e) => {
                warn!("⚠️ Jupiter mainnet connection issue: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Obtener precio real de mainnet
    pub async fn get_real_mainnet_price(&self, token_mint: &str) -> Result<f64> {
        debug!("🔍 Getting real mainnet price for {}", token_mint);
        
        // Primero intentar Syndica (más rápido)
        if let Some(ref syndica) = self.syndica_client {
            if let Ok(Some(price)) = syndica.get_price_ultra_safe(token_mint).await {
                debug!("✅ Got price from Syndica: ${:.4}", price);
                return Ok(price);
            }
        }
        
        // Fallback a Jupiter
        match self.jupiter_client.get_token_price_direct(token_mint).await {
            Ok(price) => {
                debug!("✅ Got price from Jupiter: ${:.4}", price);
                Ok(price)
            }
            Err(e) => {
                error!("❌ Failed to get mainnet price: {}", e);
                Err(e)
            }
        }
    }
    
    /// Simular trade con datos reales de mainnet
    pub async fn simulate_trade(
        &mut self,
        input_token: &str,
        output_token: &str,
        input_amount: f64,
    ) -> Result<PaperTradeResult> {
        info!("📊 Simulating trade: {} -> {} (amount: {})", 
              input_token, output_token, input_amount);
        
        let start_time = Instant::now();
        
        // Step 1: Obtener precios reales de mainnet
        let input_price = self.get_real_mainnet_price(input_token).await?;
        let output_price = self.get_real_mainnet_price(output_token).await?;
        
        // Step 2: Obtener quote real de Jupiter
        let input_amount_lamports = (input_amount * 1_000_000_000.0) as u64; // Convert to lamports
        let quote = self.jupiter_client.get_quote_direct(
            input_token,
            output_token,
            input_amount_lamports,
        ).await?;
        
        // Step 3: Calcular output con fees y slippage simulados
        let base_output = quote.out_amount.parse::<u64>()
            .map_err(|_| anyhow!("Invalid output amount"))? as f64 / 1_000_000_000.0;
        
        // Aplicar fees simulados
        let fee_amount = input_amount * (self.config.simulated_fee_bps as f64 / 10000.0);
        
        // Aplicar slippage simulado
        let slippage_amount = base_output * (self.config.simulated_slippage_percent / 100.0);
        let final_output = base_output - slippage_amount;
        
        // Step 4: Simular latencia de ejecución
        tokio::time::sleep(Duration::from_millis(self.config.execution_latency_ms)).await;
        
        let total_latency = start_time.elapsed();
        
        // Step 5: Calcular P&L
        let input_value_usd = input_amount * input_price;
        let output_value_usd = final_output * output_price;
        let profit_loss = output_value_usd - input_value_usd - (fee_amount * input_price);
        
        // Step 6: Crear resultado del trade
        let trade_result = PaperTradeResult {
            trade_id: format!("paper_{}", self.portfolio.trade_count + 1),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            input_token: input_token.to_string(),
            output_token: output_token.to_string(),
            input_amount,
            output_amount: final_output,
            input_price,
            output_price,
            simulated_fee: fee_amount,
            simulated_slippage: slippage_amount,
            total_latency,
            profit_loss_usd: profit_loss,
            is_profitable: profit_loss > 0.0,
        };
        
        info!("💹 Trade simulation completed:");
        info!("   Input: {} {} (${:.2})", input_amount, "TOKEN", input_value_usd);
        info!("   Output: {:.6} {} (${:.2})", final_output, "TOKEN", output_value_usd);
        info!("   P&L: ${:.4} ({})", profit_loss, if profit_loss > 0.0 { "📈 PROFIT" } else { "📉 LOSS" });
        info!("   Latency: {:?}", total_latency);
        
        Ok(trade_result)
    }
    
    /// Ejecutar trade simulado y actualizar portfolio
    pub async fn execute_paper_trade(
        &mut self,
        input_token: &str,
        output_token: &str,
        input_amount: f64,
    ) -> Result<PaperTradeResult> {
        // Validar que tenemos suficiente balance
        if let Some(balance) = self.portfolio.balances.get(input_token) {
            if balance.amount < input_amount {
                return Err(anyhow!("❌ Insufficient balance: have {}, need {}", 
                                   balance.amount, input_amount));
            }
        } else {
            return Err(anyhow!("❌ No balance found for token {}", input_token));
        }
        
        // Simular el trade
        let trade_result = self.simulate_trade(input_token, output_token, input_amount).await?;
        
        // Actualizar balances
        self.update_portfolio_balances(&trade_result).await?;
        
        // Guardar en historial
        self.trade_history.push(trade_result.clone());
        
        Ok(trade_result)
    }
    
    /// Actualizar balances del portfolio
    async fn update_portfolio_balances(&mut self, trade: &PaperTradeResult) -> Result<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        // Reducir balance del token de entrada
        if let Some(balance) = self.portfolio.balances.get_mut(&trade.input_token) {
            balance.amount -= trade.input_amount + trade.simulated_fee;
            balance.value_usd = balance.amount * trade.input_price;
            balance.last_updated = now;
        }
        
        // Aumentar balance del token de salida
        if let Some(balance) = self.portfolio.balances.get_mut(&trade.output_token) {
            balance.amount += trade.output_amount;
            balance.value_usd = balance.amount * trade.output_price;
            balance.last_updated = now;
        } else {
            // Crear nuevo balance si no existe
            self.portfolio.balances.insert(trade.output_token.clone(), PaperBalance {
                token_mint: trade.output_token.clone(),
                symbol: "UNKNOWN".to_string(),
                amount: trade.output_amount,
                value_usd: trade.output_amount * trade.output_price,
                last_updated: now,
            });
        }
        
        // Actualizar estadísticas del portfolio
        self.portfolio.trade_count += 1;
        if trade.is_profitable {
            self.portfolio.winning_trades += 1;
        } else {
            self.portfolio.losing_trades += 1;
        }
        
        // Recalcular valor total
        self.portfolio.total_value_usd = self.portfolio.balances.values()
            .map(|b| b.value_usd)
            .sum();
        
        self.portfolio.profit_loss_usd = self.portfolio.total_value_usd - self.portfolio.initial_value_usd;
        self.portfolio.profit_loss_percent = (self.portfolio.profit_loss_usd / self.portfolio.initial_value_usd) * 100.0;
        
        Ok(())
    }
    
    /// Obtener resumen del portfolio
    pub fn get_portfolio_summary(&self) -> &PaperPortfolio {
        &self.portfolio
    }
    
    /// Obtener historial de trades
    pub fn get_trade_history(&self) -> &[PaperTradeResult] {
        &self.trade_history
    }
    
    /// Imprimir estado del portfolio
    pub fn print_portfolio_status(&self) {
        println!("\n📊 PAPER TRADING PORTFOLIO STATUS");
        println!("==================================");
        println!("💰 Total Value: ${:.2}", self.portfolio.total_value_usd);
        println!("📈 P&L: ${:.2} ({:.2}%)", 
                 self.portfolio.profit_loss_usd, 
                 self.portfolio.profit_loss_percent);
        println!("📊 Trades: {} (✅ {} wins, ❌ {} losses)", 
                 self.portfolio.trade_count,
                 self.portfolio.winning_trades,
                 self.portfolio.losing_trades);
        
        let win_rate = if self.portfolio.trade_count > 0 {
            (self.portfolio.winning_trades as f64 / self.portfolio.trade_count as f64) * 100.0
        } else { 0.0 };
        println!("🎯 Win Rate: {:.1}%", win_rate);
        
        println!("\n💼 Current Balances:");
        for balance in self.portfolio.balances.values() {
            println!("   {} {}: {:.6} (${:.2})", 
                     balance.symbol, 
                     &balance.token_mint[..8],
                     balance.amount, 
                     balance.value_usd);
        }
    }
}

/// Test function para paper trading con mainnet
pub async fn test_paper_trading_mainnet() -> Result<()> {
    println!("📊 PAPER TRADING WITH MAINNET DATA");
    println!("===================================");
    
    let config = PaperTradingConfig::default();
    let mut engine = PaperTradingEngine::new(config).await?;
    
    // Conectar a mainnet
    engine.connect_mainnet_feeds().await?;
    
    // Mostrar portfolio inicial
    engine.print_portfolio_status();
    
    // Test 1: Obtener precio real de SOL
    println!("\n1️⃣ Testing real mainnet price fetching...");
    let sol_mint = "So11111111111111111111111111111111111111112";
    match engine.get_real_mainnet_price(sol_mint).await {
        Ok(price) => {
            println!("✅ Real SOL price: ${:.4}", price);
        }
        Err(e) => {
            println!("❌ Failed to get SOL price: {}", e);
        }
    }
    
    // Test 2: Simular trade SOL -> USDC
    println!("\n2️⃣ Testing paper trade simulation...");
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    match engine.execute_paper_trade(sol_mint, usdc_mint, 1.0).await {
        Ok(trade) => {
            println!("✅ Paper trade executed:");
            println!("   Trade ID: {}", trade.trade_id);
            println!("   Input: {} SOL", trade.input_amount);
            println!("   Output: {:.2} USDC", trade.output_amount);
            println!("   P&L: ${:.4}", trade.profit_loss_usd);
            println!("   Latency: {:?}", trade.total_latency);
        }
        Err(e) => {
            println!("❌ Paper trade failed: {}", e);
        }
    }
    
    // Mostrar portfolio actualizado
    engine.print_portfolio_status();
    
    println!("\n✅ Paper trading test completed!");
    Ok(())
}
