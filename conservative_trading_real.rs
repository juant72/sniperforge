// ===== CONFIGURACIÓN TRADING REAL CONSERVADOR =====
// 🎯 SISTEMA PHASE 4.5 - MODO TRADING REAL CON PARÁMETROS SEGUROS
// 💰 OBJETIVO: Profits reales conservadores con capital limitado

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use tokio::sync::Mutex;
use solana_sdk::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;
use reqwest;
use serde_json::{Value, json};

// ===== CONFIGURACIÓN CRYPTO PROVIDER PARA WEBSOCKETS =====
fn setup_crypto_provider() {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
}

// ===== PARÁMETROS CONSERVADORES PARA TRADING REAL =====
const CONSERVATIVE_MIN_PROFIT_BPS: u64 = 10; // 0.1% - Mínimo profit conservador
const CONSERVATIVE_MAX_SLIPPAGE_BPS: u64 = 30; // 0.3% - Slippage muy conservador
const CONSERVATIVE_MAX_TRADE_SOL: f64 = 1.0; // 1 SOL máximo por trade
const CONSERVATIVE_MIN_TRADE_SOL: f64 = 0.1; // 0.1 SOL mínimo
const CONSERVATIVE_API_TIMEOUT_MS: u64 = 10000; // 10 segundos timeout
const CONSERVATIVE_PRIORITY_FEE: u64 = 50_000; // 0.00005 SOL - Bajo fee
const CONSERVATIVE_CYCLE_DELAY_MS: u64 = 15000; // 15 segundos entre ciclos
const CONSERVATIVE_MAX_DAILY_TRADES: usize = 50; // Máximo 50 trades/día
const CONSERVATIVE_STOP_LOSS_SOL: f64 = 0.5; // Stop loss a 0.5 SOL pérdida
const CONSERVATIVE_MIN_CONFIDENCE: f64 = 90.0; // Solo oportunidades 90%+ confianza

// ===== TOKENS SEGUROS PARA TRADING CONSERVADOR =====
const SAFE_TRADING_TOKENS: &[(&str, &str, f64)] = &[
    ("SOL", "So11111111111111111111111111111111111111112", 100.0), // Capital limitado
    ("USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", 500.0), // Stablecoin seguro
    ("USDT", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", 300.0), // Stablecoin respaldo
    ("RAY", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", 50.0), // Major token limitado
];

// ===== CONFIGURACIÓN CONSERVADORA =====
#[derive(Debug, Clone)]
pub struct ConservativeConfig {
    pub max_trade_amount_sol: f64,
    pub min_profit_threshold_bps: u64,
    pub max_slippage_bps: u64,
    pub max_daily_trades: usize,
    pub stop_loss_sol: f64,
    pub min_confidence_score: f64,
    pub cycle_delay_ms: u64,
    pub enable_mev_protection: bool,
    pub enable_real_execution: bool,
    pub dry_run_mode: bool,
}

impl Default for ConservativeConfig {
    fn default() -> Self {
        Self {
            max_trade_amount_sol: CONSERVATIVE_MAX_TRADE_SOL,
            min_profit_threshold_bps: CONSERVATIVE_MIN_PROFIT_BPS,
            max_slippage_bps: CONSERVATIVE_MAX_SLIPPAGE_BPS,
            max_daily_trades: CONSERVATIVE_MAX_DAILY_TRADES,
            stop_loss_sol: CONSERVATIVE_STOP_LOSS_SOL,
            min_confidence_score: CONSERVATIVE_MIN_CONFIDENCE,
            cycle_delay_ms: CONSERVATIVE_CYCLE_DELAY_MS,
            enable_mev_protection: true,
            enable_real_execution: false, // INICIALMENTE FALSE - Paper trading
            dry_run_mode: true, // SAFETY FIRST
        }
    }
}

// ===== SISTEMA TRADING CONSERVADOR =====
pub struct ConservativeArbitrageSystem {
    pub config: ConservativeConfig,
    pub daily_trades_count: AtomicUsize,
    pub daily_profit_sol: Arc<std::sync::Mutex<f64>>,
    pub daily_loss_sol: Arc<std::sync::Mutex<f64>>,
    pub start_time: Instant,
    pub jupiter_client: JupiterAdvancedClient,
    pub mev_protection: MEVProtectionClient,
    pub price_cache: Arc<Mutex<HashMap<String, RealPriceData>>>,
    pub last_api_call: Arc<Mutex<Instant>>,
}

impl ConservativeArbitrageSystem {
    pub async fn new() -> Result<Self> {
        setup_crypto_provider();
        
        let config = ConservativeConfig::default();
        
        info!("🛡️ Iniciando sistema conservador con parámetros seguros:");
        info!("   💰 Max trade: {} SOL", config.max_trade_amount_sol);
        info!("   📊 Min profit: {}% ", config.min_profit_threshold_bps as f64 / 100.0);
        info!("   🛡️ Max slippage: {}%", config.max_slippage_bps as f64 / 100.0);
        info!("   📅 Max trades/día: {}", config.max_daily_trades);
        info!("   🚨 Stop loss: {} SOL", config.stop_loss_sol);
        info!("   🎯 Min confianza: {}%", config.min_confidence_score);
        info!("   🔄 Dry run: {}", config.dry_run_mode);
        
        Ok(Self {
            config,
            daily_trades_count: AtomicUsize::new(0),
            daily_profit_sol: Arc::new(std::sync::Mutex::new(0.0)),
            daily_loss_sol: Arc::new(std::sync::Mutex::new(0.0)),
            start_time: Instant::now(),
            jupiter_client: JupiterAdvancedClient::new().await?,
            mev_protection: MEVProtectionClient::new(),
            price_cache: Arc::new(Mutex::new(HashMap::new())),
            last_api_call: Arc::new(Mutex::new(Instant::now())),
        })
    }
    
    /// 🎯 MAIN LOOP CONSERVADOR
    pub async fn run_conservative_trading(&mut self) -> Result<()> {
        info!("🚀 Iniciando trading conservador - Phase 4.5");
        info!("🛡️ Modo: {}", if self.config.dry_run_mode { "DRY RUN (Paper Trading)" } else { "TRADING REAL" });
        
        let mut cycle = 1;
        
        loop {
            // 1. VERIFICAR LÍMITES DIARIOS
            if !self.check_daily_limits().await? {
                warn!("📊 Límites diarios alcanzados - parando trading");
                break;
            }
            
            // 2. VERIFICAR STOP LOSS
            if !self.check_stop_loss().await? {
                error!("🚨 Stop loss activado - parando trading");
                break;
            }
            
            info!("🔄 Ciclo conservador #{} iniciando...", cycle);
            
            // 3. BUSCAR OPORTUNIDADES CONSERVADORAS
            let opportunities = self.find_conservative_opportunities().await?;
            
            if opportunities.is_empty() {
                info!("📊 No hay oportunidades conservadoras en ciclo #{}", cycle);
            } else {
                info!("💡 Encontradas {} oportunidades conservadoras", opportunities.len());
                
                // 4. EJECUTAR SOLO LAS MEJORES
                for (i, opportunity) in opportunities.iter().take(3).enumerate() { // Máximo 3 por ciclo
                    if self.daily_trades_count.load(Ordering::Relaxed) >= self.config.max_daily_trades {
                        warn!("📊 Máximo trades diarios alcanzado");
                        break;
                    }
                    
                    self.execute_conservative_opportunity(opportunity, i + 1).await?;
                    
                    // Pausa entre trades
                    tokio::time::sleep(Duration::from_millis(2000)).await;
                }
            }
            
            // 5. ESTADÍSTICAS DEL CICLO
            self.log_cycle_stats(cycle).await;
            
            cycle += 1;
            
            // 6. PAUSA ENTRE CICLOS
            info!("⏱️ Pausando {} segundos antes del siguiente ciclo...", self.config.cycle_delay_ms / 1000);
            tokio::time::sleep(Duration::from_millis(self.config.cycle_delay_ms)).await;
        }
        
        Ok(())
    }
    
    /// 🔍 BUSCAR OPORTUNIDADES CONSERVADORAS
    async fn find_conservative_opportunities(&self) -> Result<Vec<ConservativeOpportunity>> {
        let mut opportunities = Vec::new();
        
        // Solo buscar entre tokens seguros
        for (token_name, token_mint, _) in SAFE_TRADING_TOKENS {
            for (other_name, other_mint, _) in SAFE_TRADING_TOKENS {
                if token_name == other_name { continue; }
                
                // Rate limiting
                self.wait_for_rate_limit().await;
                
                // Buscar arbitraje conservador
                if let Ok(opp) = self.find_conservative_arbitrage(token_mint, other_mint, token_name, other_name).await {
                    if opp.meets_conservative_criteria(&self.config) {
                        opportunities.push(opp);
                    }
                }
            }
        }
        
        // Ordenar por profit/risk ratio
        opportunities.sort_by(|a, b| b.risk_adjusted_profit().partial_cmp(&a.risk_adjusted_profit()).unwrap());
        
        Ok(opportunities)
    }
    
    /// 💰 BUSCAR ARBITRAJE ESPECÍFICO CONSERVADOR
    async fn find_conservative_arbitrage(&self, input_mint: &str, output_mint: &str, input_name: &str, output_name: &str) -> Result<ConservativeOpportunity> {
        let trade_amount = (self.config.max_trade_amount_sol * 0.5 * 1_000_000_000.0) as u64; // 50% del máximo
        
        let quote = self.jupiter_client.get_advanced_quote(
            input_mint,
            output_mint,
            trade_amount,
            true
        ).await?;
        
        let profit_bps = ((quote.output_amount as f64 - quote.input_amount as f64) / quote.input_amount as f64) * 10000.0;
        
        Ok(ConservativeOpportunity {
            input_token: input_name.to_string(),
            output_token: output_name.to_string(),
            input_amount: quote.input_amount,
            output_amount: quote.output_amount,
            profit_bps,
            confidence_score: quote.profitability_score,
            estimated_gas_sol: 0.001, // Estimación conservadora
            route_complexity: quote.complexity,
        })
    }
    
    /// ⚡ EJECUTAR OPORTUNIDAD CONSERVADORA
    async fn execute_conservative_opportunity(&self, opportunity: &ConservativeOpportunity, index: usize) -> Result<()> {
        info!("🎯 Ejecutando oportunidad conservadora #{}: {} -> {}", 
            index, opportunity.input_token, opportunity.output_token);
        info!("💰 Profit esperado: {:.4}% ({:.6} SOL)", 
            opportunity.profit_bps / 100.0, 
            opportunity.profit_bps * opportunity.input_amount as f64 / 1_000_000_000.0 / 10000.0);
        
        if self.config.dry_run_mode {
            // PAPER TRADING
            info!("📄 PAPER TRADING - Simulando ejecución");
            
            // Simular éxito/fallo basado en confianza
            let success = opportunity.confidence_score > 85.0;
            
            if success {
                let profit_sol = opportunity.profit_bps * opportunity.input_amount as f64 / 1_000_000_000.0 / 10000.0;
                {
                    let mut daily_profit = self.daily_profit_sol.lock().unwrap();
                    *daily_profit += profit_sol;
                }
                self.daily_trades_count.fetch_add(1, Ordering::Relaxed);
                info!("✅ PAPER TRADE exitoso: +{:.6} SOL profit", profit_sol);
            } else {
                let loss_sol = 0.001; // Fee estimado
                {
                    let mut daily_loss = self.daily_loss_sol.lock().unwrap();
                    *daily_loss += loss_sol;
                }
                warn!("❌ PAPER TRADE falló: -{:.6} SOL (fees)", loss_sol);
            }
        } else {
            // TRADING REAL
            info!("💸 TRADING REAL - Ejecutando con MEV protection");
            
            match self.mev_protection.execute_protected_trade(opportunity).await {
                Ok(signature) => {
                    let profit_sol = opportunity.profit_bps * opportunity.input_amount as f64 / 1_000_000_000.0 / 10000.0;
                    {
                        let mut daily_profit = self.daily_profit_sol.lock().unwrap();
                        *daily_profit += profit_sol;
                    }
                    self.daily_trades_count.fetch_add(1, Ordering::Relaxed);
                    info!("✅ TRADING REAL exitoso: {} - +{:.6} SOL profit", signature, profit_sol);
                },
                Err(e) => {
                    let loss_sol = 0.002; // Fee + slippage estimado
                    {
                        let mut daily_loss = self.daily_loss_sol.lock().unwrap();
                        *daily_loss += loss_sol;
                    }
                    warn!("❌ TRADING REAL falló: {} - -{:.6} SOL", e, loss_sol);
                }
            }
        }
        
        Ok(())
    }
    
    /// 📊 VERIFICAR LÍMITES DIARIOS
    async fn check_daily_limits(&self) -> Result<bool> {
        let trades_today = self.daily_trades_count.load(Ordering::Relaxed);
        
        if trades_today >= self.config.max_daily_trades {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// 🚨 VERIFICAR STOP LOSS
    async fn check_stop_loss(&self) -> Result<bool> {
        let daily_loss = *self.daily_loss_sol.lock().unwrap();
        
        if daily_loss >= self.config.stop_loss_sol {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// 📈 LOG ESTADÍSTICAS DEL CICLO
    async fn log_cycle_stats(&self, cycle: usize) {
        let daily_profit = *self.daily_profit_sol.lock().unwrap();
        let daily_loss = *self.daily_loss_sol.lock().unwrap();
        let net_profit = daily_profit - daily_loss;
        let trades_count = self.daily_trades_count.load(Ordering::Relaxed);
        
        info!("📊 ESTADÍSTICAS CONSERVADORAS (Ciclo #{}):", cycle);
        info!("   💰 Profit diario: {:.6} SOL", daily_profit);
        info!("   📉 Pérdidas diarias: {:.6} SOL", daily_loss);
        info!("   📈 Profit neto: {:.6} SOL", net_profit);
        info!("   📊 Trades ejecutados: {}/{}", trades_count, self.config.max_daily_trades);
        
        if net_profit > 0.0 {
            info!("✅ Sistema RENTABLE - continuando");
        } else if daily_loss > 0.0 {
            warn!("⚠️ Sistema con pérdidas - monitoreando");
        }
    }
    
    /// ⏱️ RATE LIMITING
    async fn wait_for_rate_limit(&self) {
        let mut last_call = self.last_api_call.lock().await;
        let elapsed = last_call.elapsed();
        
        if elapsed < Duration::from_millis(500) { // 2 calls/second max
            tokio::time::sleep(Duration::from_millis(500) - elapsed).await;
        }
        
        *last_call = Instant::now();
    }
}

// ===== ESTRUCTURA OPORTUNIDAD CONSERVADORA =====
#[derive(Debug, Clone)]
pub struct ConservativeOpportunity {
    pub input_token: String,
    pub output_token: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub profit_bps: f64,
    pub confidence_score: f64,
    pub estimated_gas_sol: f64,
    pub route_complexity: usize,
}

impl ConservativeOpportunity {
    /// ✅ VERIFICAR SI CUMPLE CRITERIOS CONSERVADORES
    pub fn meets_conservative_criteria(&self, config: &ConservativeConfig) -> bool {
        // 1. Profit mínimo
        if self.profit_bps < config.min_profit_threshold_bps as f64 {
            return false;
        }
        
        // 2. Confianza mínima
        if self.confidence_score < config.min_confidence_score {
            return false;
        }
        
        // 3. Complejidad baja (menos risk)
        if self.route_complexity > 3 {
            return false;
        }
        
        // 4. Profit neto positivo después de fees
        let net_profit = (self.profit_bps * self.input_amount as f64 / 10000.0 / 1_000_000_000.0) - self.estimated_gas_sol;
        if net_profit <= 0.0 {
            return false;
        }
        
        true
    }
    
    /// 📊 CALCULAR PROFIT AJUSTADO POR RIESGO
    pub fn risk_adjusted_profit(&self) -> f64 {
        let base_profit = self.profit_bps / 100.0;
        let confidence_factor = self.confidence_score / 100.0;
        let complexity_penalty = 1.0 - (self.route_complexity as f64 * 0.1);
        
        base_profit * confidence_factor * complexity_penalty
    }
}

// ===== STUBS PARA COMPATIBILIDAD =====
pub use crate::arbitrage_bot_phase45_final::{
    JupiterAdvancedClient, MEVProtectionClient, RealPriceData,
    JupiterAdvancedQuote, setup_crypto_provider as setup_crypto_provider_base
};

// ===== MAIN FUNCTION CONSERVADORA =====
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("🛡️ SNIPERFORGE ARBITRAGE - MODO CONSERVADOR");
    println!("═══════════════════════════════════════════════");
    println!("🎯 Trading real conservador con parámetros seguros");
    println!("💰 Capital limitado, profits consistentes");
    println!("🚨 Stop loss automático y límites diarios");
    println!();
    
    let mut system = ConservativeArbitrageSystem::new().await?;
    
    println!("Selecciona modo:");
    println!("1. 📄 Paper Trading (Recomendado - Sin riesgo)");
    println!("2. 💸 Trading Real (Conservador - Capital real)");
    println!("3. ⚙️ Configurar parámetros");
    print!("Selecciona opción (1-3): ");
    
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();
    
    match choice {
        "1" => {
            println!("📄 Iniciando Paper Trading - Sin riesgo");
            system.config.dry_run_mode = true;
            system.config.enable_real_execution = false;
        },
        "2" => {
            println!("💸 ¿Estás seguro de trading real? (yes/no): ");
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;
            if confirm.trim().to_lowercase() == "yes" {
                println!("💸 Iniciando Trading Real Conservador");
                system.config.dry_run_mode = false;
                system.config.enable_real_execution = true;
            } else {
                println!("📄 Modo seguro activado - Paper Trading");
                system.config.dry_run_mode = true;
                system.config.enable_real_execution = false;
            }
        },
        "3" => {
            println!("⚙️ Configuración personalizada no implementada");
            println!("📄 Usando Paper Trading por seguridad");
            system.config.dry_run_mode = true;
            system.config.enable_real_execution = false;
        },
        _ => {
            println!("📄 Opción inválida - Usando Paper Trading por seguridad");
            system.config.dry_run_mode = true;
            system.config.enable_real_execution = false;
        }
    }
    
    system.run_conservative_trading().await?;
    
    Ok(())
}
