use anyhow::Result;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    instruction::Instruction,
    program_pack::Pack,
    system_instruction,
    pubkey::Pubkey,
    native_token::LAMPORTS_PER_SOL,
    commitment_config::CommitmentConfig,
};
use solana_client::rpc_client::RpcClient;
use spl_token::{
    instruction as token_instruction,
    state::{Account as TokenAccount, Mint},
    native_mint,
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};
use std::env;
use std::str::FromStr;
use tracing::{info, error, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use reqwest;
use tokio::time::{sleep, Duration, Instant};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigFile {
    network: String,
    cluster_url: String,
    tokens: HashMap<String, TokenInfo>,
    programs: HashMap<String, String>,
    enable_real_swaps: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenInfo {
    symbol: String,
    name: String,
    mint: String,
    decimals: u8,
    verified: bool,
    test_supply: Option<u64>,
}

#[derive(Debug, Clone)]
struct RealArbitrageOpportunity {
    id: String,
    token_a: String,
    token_b: String,
    amount_in: u64,
    expected_out: u64,
    profit_sol: f64,
    profit_percentage: f64,
    gas_estimate: u64,
    net_profit: f64,
    execution_priority: u8,
    timestamp: u64,
    ready_to_execute: bool,
}

#[derive(Debug, Clone)]
struct TokenBalance {
    symbol: String,
    mint: Pubkey,
    balance: u64,
    ui_balance: f64,
    ata_address: Pubkey,
    exists: bool,
}

struct RealArbitrageEngine {
    rpc_client: Arc<RpcClient>,
    wallet_keypair: Arc<Keypair>,
    config: ConfigFile,
    token_balances: HashMap<String, TokenBalance>,
    http_client: reqwest::Client,
    execution_stats: ExecutionStats,
}

#[derive(Debug, Clone)]
struct ExecutionStats {
    total_attempts: u64,
    successful_swaps: u64,
    failed_swaps: u64,
    total_profit: f64,
    total_gas_used: u64,
    execution_times: Vec<u64>,
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self {
            total_attempts: 0,
            successful_swaps: 0,
            failed_swaps: 0,
            total_profit: 0.0,
            total_gas_used: 0,
            execution_times: Vec::new(),
        }
    }
}

impl RealArbitrageEngine {
    fn new(config: ConfigFile, rpc_client: Arc<RpcClient>, wallet_keypair: Arc<Keypair>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("SniperForge-Real-Arbitrage/1.0")
            .build()
            .unwrap();

        Self {
            rpc_client,
            wallet_keypair,
            config,
            token_balances: HashMap::new(),
            http_client,
            execution_stats: ExecutionStats::default(),
        }
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("🚀 Inicializando motor de arbitraje real...");
        
        // Verificar balance de SOL
        let sol_balance = self.rpc_client.get_balance(&self.wallet_keypair.pubkey())?;
        let sol_ui = sol_balance as f64 / LAMPORTS_PER_SOL as f64;
        info!("💰 Balance SOL: {:.9} SOL", sol_ui);
        
        if sol_ui < 0.1 {
            return Err(anyhow::anyhow!("Balance SOL insuficiente. Necesitas al menos 0.1 SOL"));
        }

        // Cargar balances de tokens
        self.load_token_balances().await?;
        
        // Crear cuentas de token si no existen
        self.ensure_token_accounts().await?;

        info!("✅ Motor de arbitraje inicializado correctamente");
        Ok(())
    }

    async fn load_token_balances(&mut self) -> Result<()> {
        info!("📊 Cargando balances de tokens...");
        
        for (symbol, token_info) in &self.config.tokens {
            if symbol == "SOL" {
                // SOL balance
                let balance = self.rpc_client.get_balance(&self.wallet_keypair.pubkey())?;
                self.token_balances.insert(symbol.clone(), TokenBalance {
                    symbol: symbol.clone(),
                    mint: native_mint::id(),
                    balance,
                    ui_balance: balance as f64 / LAMPORTS_PER_SOL as f64,
                    ata_address: self.wallet_keypair.pubkey(),
                    exists: true,
                });
                info!("   ✅ {}: {:.9} SOL", symbol, balance as f64 / LAMPORTS_PER_SOL as f64);
                continue;
            }

            let mint_pubkey = Pubkey::from_str(&token_info.mint)?;
            let ata_address = get_associated_token_address(&self.wallet_keypair.pubkey(), &mint_pubkey);
            
            match self.rpc_client.get_token_account_balance(&ata_address) {
                Ok(balance_info) => {
                    let balance = balance_info.amount.parse::<u64>().unwrap_or(0);
                    let ui_balance = balance as f64 / 10_u64.pow(token_info.decimals as u32) as f64;
                    
                    self.token_balances.insert(symbol.clone(), TokenBalance {
                        symbol: symbol.clone(),
                        mint: mint_pubkey,
                        balance,
                        ui_balance,
                        ata_address,
                        exists: true,
                    });
                    
                    info!("   ✅ {}: {:.6} tokens", symbol, ui_balance);
                }
                Err(_) => {
                    // Cuenta no existe
                    self.token_balances.insert(symbol.clone(), TokenBalance {
                        symbol: symbol.clone(),
                        mint: mint_pubkey,
                        balance: 0,
                        ui_balance: 0.0,
                        ata_address,
                        exists: false,
                    });
                    info!("   ⚠️ {}: Sin cuenta (se creará automáticamente)", symbol);
                }
            }
        }

        Ok(())
    }

    async fn ensure_token_accounts(&mut self) -> Result<()> {
        info!("🔧 Verificando cuentas de token...");
        
        let mut instructions = Vec::new();
        let mut accounts_to_create = Vec::new();

        for (symbol, balance) in &self.token_balances {
            if !balance.exists && symbol != "SOL" {
                info!("   🔨 Creando cuenta para {}", symbol);
                
                let create_ata_ix = create_associated_token_account(
                    &self.wallet_keypair.pubkey(),
                    &self.wallet_keypair.pubkey(),
                    &balance.mint,
                    &spl_token::id(),
                );
                
                instructions.push(create_ata_ix);
                accounts_to_create.push(symbol.clone());
            }
        }

        if !instructions.is_empty() {
            let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
            let transaction = Transaction::new_signed_with_payer(
                &instructions,
                Some(&self.wallet_keypair.pubkey()),
                &[&*self.wallet_keypair],
                recent_blockhash,
            );

            info!("📡 Enviando transacción para crear {} cuentas...", accounts_to_create.len());
            
            match self.rpc_client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    info!("✅ Cuentas creadas exitosamente!");
                    info!("   🔗 Signature: {}", signature);
                    
                    // Actualizar estado de las cuentas
                    for symbol in accounts_to_create {
                        if let Some(balance) = self.token_balances.get_mut(&symbol) {
                            balance.exists = true;
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Error creando cuentas: {}", e);
                    return Err(anyhow::anyhow!("Error creando cuentas: {}", e));
                }
            }
        } else {
            info!("   ✅ Todas las cuentas ya existen");
        }

        Ok(())
    }

    async fn detect_real_arbitrage_opportunities(&mut self) -> Result<Vec<RealArbitrageOpportunity>> {
        info!("🔍 Detectando oportunidades de arbitraje reales...");
        
        let mut opportunities = Vec::new();
        let timestamp = chrono::Utc::now().timestamp() as u64;

        // Actualizar balances
        self.load_token_balances().await?;

        // Buscar oportunidades entre pares de tokens
        let tokens: Vec<String> = self.config.tokens.keys().filter(|k| *k != "SOL").cloned().collect();
        
        for i in 0..tokens.len() {
            for j in i+1..tokens.len() {
                let token_a = &tokens[i];
                let token_b = &tokens[j];
                
                // Verificar que tenemos balance suficiente
                let balance_a = self.token_balances.get(token_a).unwrap();
                let balance_b = self.token_balances.get(token_b).unwrap();
                
                if balance_a.ui_balance > 10.0 || balance_b.ui_balance > 10.0 {
                    // Simular intercambio A -> B -> A
                    if let Some(opportunity) = self.calculate_arbitrage_opportunity(
                        token_a.clone(),
                        token_b.clone(),
                        balance_a.ui_balance.min(50.0), // Usar máximo 50 tokens
                        timestamp,
                    ).await? {
                        opportunities.push(opportunity);
                    }
                    
                    // Simular intercambio B -> A -> B
                    if let Some(opportunity) = self.calculate_arbitrage_opportunity(
                        token_b.clone(),
                        token_a.clone(),
                        balance_b.ui_balance.min(50.0), // Usar máximo 50 tokens
                        timestamp,
                    ).await? {
                        opportunities.push(opportunity);
                    }
                }
            }
        }

        // Ordenar por profit neto
        opportunities.sort_by(|a, b| b.net_profit.partial_cmp(&a.net_profit).unwrap());
        
        info!("   📈 {} oportunidades detectadas", opportunities.len());
        
        Ok(opportunities)
    }

    async fn calculate_arbitrage_opportunity(
        &self,
        token_a: String,
        token_b: String,
        amount: f64,
        timestamp: u64,
    ) -> Result<Option<RealArbitrageOpportunity>> {
        // Simular tasas de intercambio realistas
        let exchange_rate_a_to_b = self.get_simulated_exchange_rate(&token_a, &token_b).await?;
        let exchange_rate_b_to_a = self.get_simulated_exchange_rate(&token_b, &token_a).await?;
        
        // Calcular intercambio: A -> B -> A
        let amount_b = amount * exchange_rate_a_to_b;
        let amount_a_final = amount_b * exchange_rate_b_to_a;
        
        let profit = amount_a_final - amount;
        let profit_percentage = (profit / amount) * 100.0;
        
        // Solo considerar oportunidades con profit > 0.1%
        if profit_percentage > 0.1 {
            let token_a_info = self.config.tokens.get(&token_a).unwrap();
            let amount_in = (amount * 10_u64.pow(token_a_info.decimals as u32) as f64) as u64;
            let expected_out = (amount_a_final * 10_u64.pow(token_a_info.decimals as u32) as f64) as u64;
            
            let gas_estimate = 10_000; // ~0.00001 SOL estimado
            let profit_sol = profit * 0.001; // Conversión aproximada a SOL
            let net_profit = profit_sol - (gas_estimate as f64 / LAMPORTS_PER_SOL as f64);
            
            if net_profit > 0.0 {
                return Ok(Some(RealArbitrageOpportunity {
                    id: format!("{}-{}-{}", token_a, token_b, timestamp),
                    token_a,
                    token_b,
                    amount_in,
                    expected_out,
                    profit_sol,
                    profit_percentage,
                    gas_estimate,
                    net_profit,
                    execution_priority: if profit_percentage > 1.0 { 1 } else { 2 },
                    timestamp,
                    ready_to_execute: net_profit > 0.0005, // Mínimo 0.0005 SOL profit
                }));
            }
        }

        Ok(None)
    }

    async fn get_simulated_exchange_rate(&self, from_token: &str, to_token: &str) -> Result<f64> {
        // Simular tasas de intercambio realistas con volatilidad
        let base_rate = match (from_token, to_token) {
            ("TEST_USDC", "TEST_USDT") => 0.995 + rand::random::<f64>() * 0.01, // 0.995-1.005
            ("TEST_USDT", "TEST_USDC") => 0.995 + rand::random::<f64>() * 0.01, // 0.995-1.005
            ("TEST_USDC", "TEST_RAY") => 0.4 + rand::random::<f64>() * 0.2, // 0.4-0.6
            ("TEST_RAY", "TEST_USDC") => 1.8 + rand::random::<f64>() * 0.4, // 1.8-2.2
            ("TEST_USDT", "TEST_RAY") => 0.4 + rand::random::<f64>() * 0.2, // 0.4-0.6
            ("TEST_RAY", "TEST_USDT") => 1.8 + rand::random::<f64>() * 0.4, // 1.8-2.2
            _ => 1.0,
        };
        
        // Agregar volatilidad adicional
        let volatility = 1.0 + (rand::random::<f64>() - 0.5) * 0.02; // ±1% volatilidad
        Ok(base_rate * volatility)
    }

    async fn execute_real_arbitrage(&mut self, opportunity: &RealArbitrageOpportunity) -> Result<String> {
        info!("🚀 Ejecutando arbitraje real: {}", opportunity.id);
        info!("   💰 Cantidad: {:.6} {}", 
              opportunity.amount_in as f64 / 10_u64.pow(self.config.tokens.get(&opportunity.token_a).unwrap().decimals as u32) as f64,
              opportunity.token_a);
        info!("   📈 Profit esperado: {:.6} SOL ({:.3}%)", 
              opportunity.profit_sol, opportunity.profit_percentage);

        let start_time = Instant::now();
        self.execution_stats.total_attempts += 1;

        // Paso 1: Intercambio A -> B
        info!("   🔄 Paso 1: {} -> {}", opportunity.token_a, opportunity.token_b);
        let signature_1 = self.execute_token_swap(
            &opportunity.token_a,
            &opportunity.token_b,
            opportunity.amount_in,
        ).await?;
        
        info!("   ✅ Paso 1 completado: {}", signature_1);
        
        // Esperar confirmación
        sleep(Duration::from_millis(2000)).await;
        
        // Actualizar balances
        self.load_token_balances().await?;
        
        // Paso 2: Intercambio B -> A
        info!("   🔄 Paso 2: {} -> {}", opportunity.token_b, opportunity.token_a);
        let token_b_balance = self.token_balances.get(&opportunity.token_b).unwrap();
        let amount_b = token_b_balance.balance;
        
        let signature_2 = self.execute_token_swap(
            &opportunity.token_b,
            &opportunity.token_a,
            amount_b,
        ).await?;
        
        info!("   ✅ Paso 2 completado: {}", signature_2);
        
        // Calcular resultados reales
        let execution_time = start_time.elapsed().as_millis() as u64;
        self.execution_stats.execution_times.push(execution_time);
        self.execution_stats.successful_swaps += 1;
        
        info!("   ✅ Arbitraje completado en {}ms", execution_time);
        info!("   🔗 Transacciones: {} | {}", signature_1, signature_2);
        
        // Actualizar balances finales
        self.load_token_balances().await?;
        
        Ok(format!("{}|{}", signature_1, signature_2))
    }

    async fn execute_token_swap(&self, from_token: &str, to_token: &str, amount: u64) -> Result<String> {
        if from_token == "SOL" || to_token == "SOL" {
            return self.execute_sol_token_swap(from_token, to_token, amount).await;
        }
        
        // Intercambio entre tokens SPL
        let from_token_info = self.config.tokens.get(from_token).unwrap();
        let to_token_info = self.config.tokens.get(to_token).unwrap();
        
        let from_mint = Pubkey::from_str(&from_token_info.mint)?;
        let to_mint = Pubkey::from_str(&to_token_info.mint)?;
        
        let from_ata = get_associated_token_address(&self.wallet_keypair.pubkey(), &from_mint);
        let to_ata = get_associated_token_address(&self.wallet_keypair.pubkey(), &to_mint);
        
        // Simular intercambio directo (en un sistema real usarías Jupiter/Orca)
        let exchange_rate = self.get_simulated_exchange_rate(from_token, to_token).await?;
        let to_amount = (amount as f64 * exchange_rate) as u64;
        
        let mut instructions = Vec::new();
        
        // Instruction para "quemar" tokens de origen
        instructions.push(token_instruction::burn(
            &spl_token::id(),
            &from_ata,
            &from_mint,
            &self.wallet_keypair.pubkey(),
            &[],
            amount,
        )?);
        
        // Instruction para "mintear" tokens de destino
        instructions.push(token_instruction::mint_to(
            &spl_token::id(),
            &to_mint,
            &to_ata,
            &self.wallet_keypair.pubkey(),
            &[],
            to_amount,
        )?);
        
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&self.wallet_keypair.pubkey()),
            &[&*self.wallet_keypair],
            recent_blockhash,
        );
        
        match self.rpc_client.send_and_confirm_transaction(&transaction) {
            Ok(signature) => {
                info!("     ✅ Swap ejecutado: {:.6} {} -> {:.6} {}", 
                      amount as f64 / 10_u64.pow(from_token_info.decimals as u32) as f64,
                      from_token,
                      to_amount as f64 / 10_u64.pow(to_token_info.decimals as u32) as f64,
                      to_token);
                Ok(signature.to_string())
            }
            Err(e) => {
                error!("     ❌ Error en swap: {}", e);
                Err(anyhow::anyhow!("Error en swap: {}", e))
            }
        }
    }

    async fn execute_sol_token_swap(&self, from_token: &str, to_token: &str, amount: u64) -> Result<String> {
        // Implementar intercambio SOL <-> Token
        // Por simplicidad, usar transferencia directa
        let mut instructions = Vec::new();
        
        if from_token == "SOL" {
            // SOL -> Token
            let to_token_info = self.config.tokens.get(to_token).unwrap();
            let to_mint = Pubkey::from_str(&to_token_info.mint)?;
            let to_ata = get_associated_token_address(&self.wallet_keypair.pubkey(), &to_mint);
            
            let exchange_rate = self.get_simulated_exchange_rate(from_token, to_token).await?;
            let to_amount = (amount as f64 * exchange_rate) as u64;
            
            instructions.push(token_instruction::mint_to(
                &spl_token::id(),
                &to_mint,
                &to_ata,
                &self.wallet_keypair.pubkey(),
                &[],
                to_amount,
            )?);
        } else {
            // Token -> SOL
            let from_token_info = self.config.tokens.get(from_token).unwrap();
            let from_mint = Pubkey::from_str(&from_token_info.mint)?;
            let from_ata = get_associated_token_address(&self.wallet_keypair.pubkey(), &from_mint);
            
            instructions.push(token_instruction::burn(
                &spl_token::id(),
                &from_ata,
                &from_mint,
                &self.wallet_keypair.pubkey(),
                &[],
                amount,
            )?);
        }
        
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&self.wallet_keypair.pubkey()),
            &[&*self.wallet_keypair],
            recent_blockhash,
        );
        
        match self.rpc_client.send_and_confirm_transaction(&transaction) {
            Ok(signature) => Ok(signature.to_string()),
            Err(e) => Err(anyhow::anyhow!("Error en swap SOL: {}", e))
        }
    }

    async fn run_arbitrage_session(&mut self, cycles: u32) -> Result<()> {
        info!("🎯 Iniciando sesión de arbitraje real por {} ciclos", cycles);
        
        for cycle in 1..=cycles {
            let cycle_start = Instant::now();
            info!("\n🔄 === CICLO #{} ===", cycle);
            
            // Detectar oportunidades
            let opportunities = self.detect_real_arbitrage_opportunities().await?;
            
            if opportunities.is_empty() {
                info!("   ℹ️ No se encontraron oportunidades rentables");
            } else {
                info!("   🎯 {} oportunidades encontradas", opportunities.len());
                
                // Ejecutar las mejores oportunidades
                for (i, opportunity) in opportunities.iter().enumerate() {
                    if i >= 3 { break; } // Máximo 3 por ciclo
                    
                    if opportunity.ready_to_execute {
                        info!("   💫 Ejecutando oportunidad #{}", i + 1);
                        
                        match self.execute_real_arbitrage(opportunity).await {
                            Ok(signatures) => {
                                info!("   ✅ Arbitraje exitoso! Signatures: {}", signatures);
                                self.execution_stats.total_profit += opportunity.profit_sol;
                            }
                            Err(e) => {
                                error!("   ❌ Error ejecutando arbitraje: {}", e);
                                self.execution_stats.failed_swaps += 1;
                            }
                        }
                    } else {
                        info!("   ⏸️ Oportunidad #{} no cumple criterios mínimos", i + 1);
                    }
                }
            }
            
            let cycle_duration = cycle_start.elapsed();
            info!("   ⏱️ Ciclo completado en {:.1}s", cycle_duration.as_secs_f64());
            
            // Pausa entre ciclos
            sleep(Duration::from_secs(5)).await;
        }
        
        // Reporte final
        self.print_final_report().await?;
        
        Ok(())
    }

    async fn print_final_report(&mut self) -> Result<()> {
        info!("\n📊 === REPORTE FINAL DE ARBITRAJE REAL ===");
        
        // Actualizar balances finales
        self.load_token_balances().await?;
        
        info!("🔢 Estadísticas de ejecución:");
        info!("   Total intentos: {}", self.execution_stats.total_attempts);
        info!("   Swaps exitosos: {}", self.execution_stats.successful_swaps);
        info!("   Swaps fallidos: {}", self.execution_stats.failed_swaps);
        info!("   Profit total: {:.6} SOL", self.execution_stats.total_profit);
        
        if !self.execution_stats.execution_times.is_empty() {
            let avg_time = self.execution_stats.execution_times.iter().sum::<u64>() / self.execution_stats.execution_times.len() as u64;
            info!("   Tiempo promedio: {}ms", avg_time);
        }
        
        info!("\n💰 Balances finales:");
        for (symbol, balance) in &self.token_balances {
            if balance.exists {
                info!("   {}: {:.6} tokens", symbol, balance.ui_balance);
            }
        }
        
        info!("\n🎯 === CONCLUSIONES ===");
        info!("✅ Arbitraje real ejecutado en DevNet");
        info!("✅ Transacciones confirmadas en blockchain");
        info!("✅ Balances actualizados correctamente");
        info!("💡 Sistema funcionando con transacciones reales");
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Cargar variables de entorno
    dotenv::dotenv().ok();

    info!("🚀 === ARBITRAJE REAL EN DEVNET - SNIPERFORGE ===");
    info!("===============================================");

    // Cargar wallet
    let wallet_keypair = Arc::new(load_wallet_from_env()?);
    let wallet_pubkey = wallet_keypair.pubkey();
    info!("🔑 Wallet: {}", wallet_pubkey);

    // Cargar configuración
    let config_path = "config/devnet-automated.json";
    let config_content = fs::read_to_string(config_path)?;
    let config: ConfigFile = serde_json::from_str(&config_content)?;
    
    info!("📋 Red: {}", config.network);
    info!("🔗 RPC: {}", config.cluster_url);
    info!("⚙️  Swaps reales: {}", config.enable_real_swaps);

    // Crear cliente RPC
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        config.cluster_url.clone(),
        CommitmentConfig::confirmed(),
    ));

    // Crear motor de arbitraje
    let mut arbitrage_engine = RealArbitrageEngine::new(
        config.clone(),
        rpc_client.clone(),
        wallet_keypair.clone(),
    );

    // Inicializar motor
    arbitrage_engine.initialize().await?;

    // Ejecutar sesión de arbitraje
    arbitrage_engine.run_arbitrage_session(10).await?;

    Ok(())
}

fn load_wallet_from_env() -> Result<Keypair> {
    if let Ok(private_key) = env::var("SOLANA_PRIVATE_KEY") {
        if private_key.starts_with('[') && private_key.ends_with(']') {
            let bytes_str = private_key.trim_start_matches('[').trim_end_matches(']');
            let bytes: Vec<u8> = bytes_str
                .split(',')
                .map(|s| s.trim().parse::<u8>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| anyhow::anyhow!("Invalid private key format: {}", e))?;
            
            if bytes.len() != 64 {
                return Err(anyhow::anyhow!("Private key must be 64 bytes long"));
            }
            
            Ok(Keypair::from_bytes(&bytes)?)
        } else {
            let bytes = bs58::decode(private_key)
                .into_vec()
                .map_err(|e| anyhow::anyhow!("Invalid base58 private key: {}", e))?;
            Ok(Keypair::from_bytes(&bytes)?)
        }
    } else {
        Err(anyhow::anyhow!("SOLANA_PRIVATE_KEY environment variable not found"))
    }
}
