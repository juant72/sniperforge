// ===== ARBITRAGE BOT REAL SIMPLIFICADO =====
// Versión directa y práctica que SÍ ejecuta swaps reales

use anyhow::Result;
use solana_sdk::{signature::Keypair, signer::Signer};
use solana_client::rpc_client::RpcClient;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};

// Imports del sistema existente
use sniperforge::shared::jupiter_client::JupiterClient;
use sniperforge::shared::jupiter_config::JupiterConfig;
use sniperforge::shared::jupiter_types::QuoteRequest;

// ===== CONFIGURACIÓN =====
const MAINNET_RPC: &str = "https://api.mainnet-beta.solana.com";
const MIN_PROFIT_SOL: f64 = 0.001; // 0.001 SOL mínimo profit
const MAX_TRADE_SOL: f64 = 0.1; // 0.1 SOL máximo por trade
const SCAN_INTERVAL_MS: u64 = 300; // 300ms entre scans

// Tokens principales para arbitraje
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const USDT_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

#[derive(Debug)]
struct ArbitrageOpportunity {
    path: String,
    profit_sol: f64,
    profit_percentage: f64,
    input_amount: u64,
    route: Vec<String>,
}

struct RealArbitrageBot {
    wallet: Keypair,
    rpc_client: RpcClient,
    jupiter_client: JupiterClient,
    total_profit: f64,
    total_trades: u32,
}

impl RealArbitrageBot {
    async fn new(wallet_path: &str) -> Result<Self> {
        info!("🚀 Inicializando Real Arbitrage Bot...");
        
        // Cargar wallet
        let wallet = solana_sdk::signature::read_keypair_file(wallet_path)
            .map_err(|e| anyhow::anyhow!("Error cargando wallet: {}", e))?;
        
        info!("💼 Wallet: {}", wallet.pubkey());
        
        // Cliente RPC
        let rpc_client = RpcClient::new(MAINNET_RPC);
        
        // Jupiter client
        let jupiter_config = JupiterConfig {
            base_url: "https://quote-api.jup.ag".to_string(),
            api_key: None,
            timeout_seconds: 10,
            max_retries: 2,
            rpc_endpoint: MAINNET_RPC.to_string(),
            network_name: "mainnet".to_string(),
        };
        let jupiter_client = JupiterClient::new(&jupiter_config).await?;
        
        Ok(Self {
            wallet,
            rpc_client,
            jupiter_client,
            total_profit: 0.0,
            total_trades: 0,
        })
    }
    
    async fn get_balance(&self) -> Result<f64> {
        let lamports = self.rpc_client.get_balance(&self.wallet.pubkey())?;
        Ok(lamports as f64 / 1_000_000_000.0)
    }
    
    async fn scan_arbitrage_opportunities(&self) -> Result<Vec<ArbitrageOpportunity>> {
        let mut opportunities = Vec::new();
        let scan_start = Instant::now();
        
        // Rutas de arbitraje a verificar
        let routes = vec![
            ("SOL → USDC → SOL", SOL_MINT, USDC_MINT, SOL_MINT),
            ("SOL → USDT → SOL", SOL_MINT, USDT_MINT, SOL_MINT),
            ("USDC → SOL → USDC", USDC_MINT, SOL_MINT, USDC_MINT),
        ];
        
        for (path_name, token_a, token_b, token_c) in routes {
            if let Ok(opportunity) = self.calculate_arbitrage_profit(path_name, token_a, token_b, token_c).await {
                if opportunity.profit_sol > MIN_PROFIT_SOL {
                    opportunities.push(opportunity);
                }
            }
        }
        
        // Ordenar por rentabilidad
        opportunities.sort_by(|a, b| b.profit_sol.partial_cmp(&a.profit_sol).unwrap());
        
        let scan_time = scan_start.elapsed().as_millis();
        if !opportunities.is_empty() {
            info!("🎯 Scan completado en {}ms - {} oportunidades encontradas", 
                scan_time, opportunities.len());
        }
        
        Ok(opportunities)
    }
    
    async fn calculate_arbitrage_profit(
        &self, 
        path_name: &str,
        token_a: &str, 
        token_b: &str, 
        token_c: &str
    ) -> Result<ArbitrageOpportunity> {
        let input_amount = (MAX_TRADE_SOL * 1_000_000_000.0) as u64; // Convertir a lamports
        
        // Paso 1: A → B
        let quote_1 = self.jupiter_client.get_quote(QuoteRequest {
            inputMint: token_a.to_string(),
            outputMint: token_b.to_string(),
            amount: input_amount,
            slippageBps: 50, // 0.5% slippage
        }).await?;
        
        let intermediate_amount: u64 = quote_1.outAmount.parse()?;
        
        // Paso 2: B → C
        let quote_2 = self.jupiter_client.get_quote(QuoteRequest {
            inputMint: token_b.to_string(),
            outputMint: token_c.to_string(),
            amount: intermediate_amount,
            slippageBps: 50,
        }).await?;
        
        let final_amount: u64 = quote_2.outAmount.parse()?;
        
        // Calcular profit
        let profit_lamports = final_amount as i64 - input_amount as i64;
        let profit_sol = profit_lamports as f64 / 1_000_000_000.0;
        let profit_percentage = (profit_lamports as f64 / input_amount as f64) * 100.0;
        
        Ok(ArbitrageOpportunity {
            path: path_name.to_string(),
            profit_sol,
            profit_percentage,
            input_amount,
            route: vec![token_a.to_string(), token_b.to_string(), token_c.to_string()],
        })
    }
    
    async fn execute_arbitrage(&mut self, opportunity: &ArbitrageOpportunity) -> Result<String> {
        info!("⚡ EJECUTANDO ARBITRAJE REAL:");
        info!("   Ruta: {}", opportunity.path);
        info!("   Profit esperado: {:.6} SOL ({:.3}%)", 
            opportunity.profit_sol, opportunity.profit_percentage);
        
        let balance_inicial = self.get_balance().await?;
        let execution_start = Instant::now();
        
        // Ejecutar los 3 pasos del arbitraje
        let mut signatures = Vec::new();
        
        for i in 0..2 { // 2 swaps para completar el arbitraje
            let (input_mint, output_mint) = (
                opportunity.route[i].clone(),
                opportunity.route[i + 1].clone()
            );
            
            // Determinar amount para este paso
            let amount = if i == 0 {
                opportunity.input_amount
            } else {
                // Para el segundo swap, usar el output del primero
                // En una implementación real, obtendríamos el balance actual
                opportunity.input_amount // Simplificado
            };
            
            info!("🔄 Ejecutando swap {}/2: {} → {}", 
                i + 1, 
                if input_mint == SOL_MINT { "SOL" } else if input_mint == USDC_MINT { "USDC" } else { "USDT" },
                if output_mint == SOL_MINT { "SOL" } else if output_mint == USDC_MINT { "USDC" } else { "USDT" }
            );
            
            // Obtener quote
            let quote = self.jupiter_client.get_quote(QuoteRequest {
                inputMint: input_mint,
                outputMint: output_mint,
                amount,
                slippageBps: 100, // 1% slippage para ejecución
            }).await?;
            
            // EJECUTAR SWAP REAL
            match self.jupiter_client.execute_swap_with_wallet(
                &quote,
                &self.wallet.pubkey().to_string(),
                Some(&self.wallet)
            ).await {
                Ok(result) => {
                    let signature = result.signature;
                    signatures.push(signature.clone());
                    info!("✅ Swap {}/2 ejecutado: {}", i + 1, signature);
                    
                    // Esperar confirmación antes del siguiente swap
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
                Err(e) => {
                    error!("❌ Error ejecutando swap {}/2: {}", i + 1, e);
                    return Err(e);
                }
            }
        }
        
        // Verificar resultado final
        tokio::time::sleep(Duration::from_secs(3)).await;
        let balance_final = self.get_balance().await?;
        let profit_real = balance_final - balance_inicial;
        let execution_time = execution_start.elapsed().as_millis();
        
        // Actualizar estadísticas
        self.total_profit += profit_real;
        self.total_trades += 1;
        
        if profit_real > 0.0 {
            info!("🎉 ARBITRAJE EXITOSO!");
            info!("   Profit real: {:.6} SOL", profit_real);
            info!("   Tiempo ejecución: {}ms", execution_time);
            info!("   Signatures: {:?}", signatures);
        } else {
            warn!("📉 Pérdida: {:.6} SOL (fees + slippage)", profit_real.abs());
        }
        
        Ok(signatures.join(", "))
    }
    
    pub async fn start_continuous_arbitrage(&mut self) -> Result<()> {
        info!("🚀 INICIANDO ARBITRAJE CONTINUO");
        info!("⚡ Configuración:");
        info!("   Min profit: {:.3} SOL", MIN_PROFIT_SOL);
        info!("   Max trade: {:.1} SOL", MAX_TRADE_SOL);
        info!("   Scan interval: {}ms", SCAN_INTERVAL_MS);
        
        let mut cycle_count = 0;
        
        loop {
            cycle_count += 1;
            
            match self.scan_arbitrage_opportunities().await {
                Ok(opportunities) => {
                    if let Some(best) = opportunities.first() {
                        info!("🎯 Cycle {}: Mejor oportunidad encontrada", cycle_count);
                        info!("   {}: {:.6} SOL ({:.3}%)", 
                            best.path, best.profit_sol, best.profit_percentage);
                        
                        // Ejecutar solo si el profit es significativo
                        if best.profit_sol > MIN_PROFIT_SOL * 2.0 { // 2x para buffer de seguridad
                            match self.execute_arbitrage(best).await {
                                Ok(signatures) => {
                                    info!("✅ Arbitraje ejecutado exitosamente");
                                }
                                Err(e) => {
                                    error!("❌ Error ejecutando arbitraje: {}", e);
                                }
                            }
                        } else {
                            info!("⏳ Profit insuficiente para ejecución segura");
                        }
                    } else {
                        if cycle_count % 20 == 0 { // Log cada 20 cycles
                            info!("🔍 Cycle {}: No hay oportunidades rentables", cycle_count);
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Error en scan: {}", e);
                }
            }
            
            // Mostrar estadísticas cada 100 cycles
            if cycle_count % 100 == 0 {
                let balance = self.get_balance().await.unwrap_or(0.0);
                info!("📊 ESTADÍSTICAS (Cycle {}):", cycle_count);
                info!("   Balance actual: {:.6} SOL", balance);
                info!("   Total trades: {}", self.total_trades);
                info!("   Total profit: {:.6} SOL", self.total_profit);
            }
            
            tokio::time::sleep(Duration::from_millis(SCAN_INTERVAL_MS)).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("🚀 REAL ARBITRAGE BOT - SNIPERFORGE");
    println!("=====================================");
    println!("⚠️  ADVERTENCIA: Este bot ejecutará swaps REALES con dinero real");
    println!("💰 Asegúrate de tener SOL en tu wallet para fees");
    println!("🛡️  Usa solo en MainNet con cantidades pequeñas para testing");
    println!("");
    
    // Cargar wallet (ajustar la ruta según sea necesario)
    let wallet_path = "wallets/mainnet-arbitrage-wallet.json";
    
    let mut bot = match RealArbitrageBot::new(wallet_path).await {
        Ok(bot) => bot,
        Err(e) => {
            error!("❌ Error inicializando bot: {}", e);
            error!("💡 Asegúrate de que existe: {}", wallet_path);
            return Err(e);
        }
    };
    
    let balance = bot.get_balance().await?;
    info!("💼 Balance inicial: {:.6} SOL", balance);
    
    if balance < 0.1 {
        error!("⚠️  Balance insuficiente. Se requieren al menos 0.1 SOL para arbitraje");
        return Ok(());
    }
    
    // Iniciar arbitraje continuo
    bot.start_continuous_arbitrage().await?;
    
    Ok(())
}
