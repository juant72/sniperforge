use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

// Modo de prueba para verificar cálculos sin ejecutar transacciones reales
const JUPITER_API_BASE: &str = "https://quote-api.jup.ag/v6";
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🧪 === PRUEBA DE CÁLCULOS - REAL ARBITRAGE SYSTEM ===");
    info!("   🔍 VERIFICANDO MATEMÁTICA Y PROTECCIONES");
    info!("   💡 MODO SEGURO - SIN TRANSACCIONES REALES");
    info!("   🛡️ VALIDANDO PREVENCIÓN DE PÉRDIDAS");

    let mut test_system = ArbitrageTestSystem::new().await?;
    test_system.run_calculation_tests().await?;

    Ok(())
}

struct ArbitrageTestSystem {
    http_client: Client,
    rate_limiter: std::time::Instant,
}

#[derive(Debug, Clone)]
struct TestOpportunity {
    input_mint: String,
    output_mint: String,
    intermediate_mint: String,
    amount_in: u64,
    expected_amount_out: u64,
    profit_lamports: u64,
    profit_percentage: f64,
    route: Vec<String>,
    all_fees: u64,
    net_profit: u64,
}

impl ArbitrageTestSystem {
    async fn new() -> Result<Self> {
        let http_client = Client::new();

        info!("✅ Sistema de pruebas inicializado");

        Ok(Self {
            http_client,
            rate_limiter: std::time::Instant::now(),
        })
    }

    async fn run_calculation_tests(&mut self) -> Result<()> {
        info!("🚀 Iniciando pruebas de cálculos...");

        // Test 1: Verificar cálculo de fees
        self.test_fee_calculations().await?;

        // Test 2: Verificar detección de oportunidades
        self.test_opportunity_detection().await?;

        // Test 3: Verificar prevención de pérdidas
        self.test_loss_prevention().await?;

        // Test 4: Verificar cálculos de slippage
        self.test_slippage_calculations().await?;

        // Test 5: Verificar quotes reales de Jupiter
        self.test_real_jupiter_quotes().await?;

        info!("✅ Todas las pruebas completadas exitosamente!");

        Ok(())
    }

    async fn test_fee_calculations(&self) -> Result<()> {
        info!("\n🧮 === TEST 1: CÁLCULO DE FEES ===");

        // Simular datos de quote de Jupiter
        let mock_quote = serde_json::json!({
            "platformFee": {
                "amount": "2500",
                "feeBps": 25
            }
        });

        let jupiter_fee = self.calculate_jupiter_fees(&mock_quote);
        let transaction_fees = 10000u64; // 2 transactions × 5000 lamports
        let priority_fees = 50000u64; // Realistic priority fees for mainnet
        let rent_fees = 4000u64; // Potential token account creation
        let total_fees = transaction_fees + jupiter_fee + priority_fees + rent_fees;

        info!("   📊 DESGLOSE DE FEES:");
        info!("      💰 Transaction fees: {} lamports", transaction_fees);
        info!("      🌟 Jupiter platform fee: {} lamports", jupiter_fee);
        info!("      ⚡ Priority fees: {} lamports", priority_fees);
        info!("      🏠 Rent fees: {} lamports", rent_fees);
        info!(
            "      🔢 TOTAL FEES: {} lamports ({:.6} SOL)",
            total_fees,
            total_fees as f64 / 1_000_000_000.0
        );

        // Verificar que los fees son razonables
        if total_fees > 100000 {
            // > 0.0001 SOL
            warn!("   ⚠️ WARNING: Fees muy altos: {} lamports", total_fees);
        } else {
            info!("   ✅ Fees dentro de rango esperado");
        }

        Ok(())
    }

    async fn test_opportunity_detection(&mut self) -> Result<()> {
        info!("\n🎯 === TEST 2: DETECCIÓN DE OPORTUNIDADES ===");

        let test_amount = 5_000_000u64; // 0.005 SOL
        let routes = vec![
            (SOL_MINT, USDC_MINT, SOL_MINT, "SOL->USDC->SOL"),
            (
                SOL_MINT,
                "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
                SOL_MINT,
                "SOL->RAY->SOL",
            ),
        ];

        let mut opportunities_found = 0;

        for (input_mint, intermediate_mint, output_mint, route_name) in routes {
            info!("   🔍 Probando ruta: {}", route_name);

            match self
                .check_arbitrage_route_test(input_mint, intermediate_mint, output_mint, test_amount)
                .await?
            {
                Some(opportunity) => {
                    opportunities_found += 1;
                    info!("      ✅ OPORTUNIDAD ENCONTRADA:");
                    info!(
                        "         💰 Profit bruto: {} lamports",
                        opportunity.profit_lamports
                    );
                    info!("         💸 Total fees: {} lamports", opportunity.all_fees);
                    info!(
                        "         💎 Profit neto: {} lamports",
                        opportunity.net_profit
                    );
                    info!(
                        "         📈 Porcentaje: {:.4}%",
                        opportunity.profit_percentage
                    );

                    // Verificar que el profit neto sea positivo
                    if opportunity.net_profit > 0 {
                        info!("         ✅ Profit neto POSITIVO");
                    } else {
                        warn!("         ❌ Profit neto NEGATIVO - se rechazaría");
                    }
                }
                None => {
                    info!("      💤 No hay oportunidad profitable en esta ruta");
                }
            }

            // Rate limiting
            sleep(Duration::from_millis(300)).await;
        }

        info!(
            "   📊 RESUMEN: {} oportunidades encontradas",
            opportunities_found
        );

        Ok(())
    }

    async fn test_loss_prevention(&self) -> Result<()> {
        info!("\n🛡️ === TEST 3: PREVENCIÓN DE PÉRDIDAS ===");

        // Simular diferentes escenarios de pérdidas
        let scenarios = vec![
            ("Balance insuficiente", 0.005, 0.01, true), // Balance menor al mínimo
            ("Profit muy pequeño", 0.05, 0.000010, true), // Profit menor que fees
            ("Balance suficiente", 0.05, 0.000020, false), // Escenario normal
        ];

        for (scenario_name, balance, expected_profit_sol, should_prevent) in scenarios {
            info!("   🧪 Escenario: {}", scenario_name);
            info!("      💰 Balance simulado: {:.6} SOL", balance);
            info!("      📈 Profit esperado: {:.6} SOL", expected_profit_sol);

            let would_execute = self.simulate_execution_decision(balance, expected_profit_sol);

            if should_prevent {
                if !would_execute {
                    info!("      ✅ CORRECTO: Ejecución PREVENIDA");
                } else {
                    error!("      ❌ ERROR: Debería prevenir ejecución pero no lo hace");
                }
            } else {
                if would_execute {
                    info!("      ✅ CORRECTO: Ejecución PERMITIDA");
                } else {
                    warn!("      ⚠️ ADVERTENCIA: Ejecución prevenida cuando debería permitir");
                }
            }
        }

        Ok(())
    }

    async fn test_slippage_calculations(&self) -> Result<()> {
        info!("\n📊 === TEST 4: CÁLCULOS DE SLIPPAGE ===");

        let test_cases = vec![
            (1_000_000u64, "SOL/USDC", "Trade pequeño, par líquido"),
            (100_000_000u64, "SOL/RAY", "Trade mediano, buena liquidez"),
            (1_000_000_000u64, "SOL/mSOL", "Trade grande, liquidez menor"),
        ];

        for (amount, token_pair, description) in test_cases {
            let slippage_bps = self.calculate_safe_slippage(amount, token_pair);

            info!("   🔄 {}", description);
            info!(
                "      💰 Amount: {} lamports ({:.6} SOL)",
                amount,
                amount as f64 / 1_000_000_000.0
            );
            info!(
                "      📈 Slippage calculado: {} bps ({:.2}%)",
                slippage_bps,
                slippage_bps as f64 / 100.0
            );

            // Verificar que el slippage esté dentro de límites razonables
            if slippage_bps > 200 {
                error!("      ❌ ERROR: Slippage muy alto: {} bps", slippage_bps);
            } else {
                info!("      ✅ Slippage dentro de límites seguros");
            }
        }

        Ok(())
    }

    async fn test_real_jupiter_quotes(&mut self) -> Result<()> {
        info!("\n🌐 === TEST 5: QUOTES REALES DE JUPITER ===");

        let test_amount = 1_000_000u64; // 0.001 SOL para prueba

        // Test quote SOL -> USDC
        info!("   🔍 Probando quote SOL -> USDC...");
        match self
            .get_jupiter_quote(SOL_MINT, USDC_MINT, test_amount)
            .await?
        {
            Some(quote) => {
                let out_amount: u64 = quote["outAmount"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);

                info!("      ✅ Quote obtenido exitosamente");
                info!("         📥 Input: {} lamports SOL", test_amount);
                info!("         📤 Output: {} USDC tokens", out_amount);

                // Verificar que el quote sea razonable
                if out_amount > 0 {
                    info!("         ✅ Quote válido");
                } else {
                    warn!("         ⚠️ Quote inválido (0 output)");
                }
            }
            None => {
                warn!("      ⚠️ No se pudo obtener quote (posible rate limit o error de API)");
            }
        }

        // Pequeña pausa para rate limiting
        sleep(Duration::from_millis(1000)).await;

        // Test quote USDC -> SOL (ruta inversa)
        info!("   🔍 Probando quote USDC -> SOL...");
        let usdc_test_amount = 1000u64; // $0.001 USDC
        match self
            .get_jupiter_quote(USDC_MINT, SOL_MINT, usdc_test_amount)
            .await?
        {
            Some(quote) => {
                let out_amount: u64 = quote["outAmount"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);

                info!("      ✅ Quote inverso obtenido");
                info!("         📥 Input: {} USDC tokens", usdc_test_amount);
                info!("         📤 Output: {} lamports SOL", out_amount);
            }
            None => {
                warn!("      ⚠️ No se pudo obtener quote inverso");
            }
        }

        Ok(())
    }

    // Función auxiliar para simular decisión de ejecución
    fn simulate_execution_decision(&self, balance: f64, expected_profit_sol: f64) -> bool {
        // Replicar la lógica del sistema real
        if balance < 0.01 {
            return false; // Balance insuficiente
        }

        let expected_profit_lamports = (expected_profit_sol * 1_000_000_000.0) as u64;
        let min_profit_lamports = 15000; // Mismo umbral que el sistema real

        expected_profit_lamports > min_profit_lamports
    }

    // Funciones auxiliares replicadas del sistema principal
    async fn check_arbitrage_route_test(
        &mut self,
        input_mint: &str,
        intermediate_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<Option<TestOpportunity>> {
        // Step 1: Get quote for input -> intermediate
        let quote1 = self
            .get_jupiter_quote(input_mint, intermediate_mint, amount)
            .await?;

        if let Some(quote1_data) = quote1 {
            let intermediate_amount: u64 = quote1_data["outAmount"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);

            if intermediate_amount == 0 {
                return Ok(None);
            }

            // Small delay to avoid rate limits
            sleep(Duration::from_millis(300)).await;

            // Step 2: Get quote for intermediate -> output
            let quote2 = self
                .get_jupiter_quote(intermediate_mint, output_mint, intermediate_amount)
                .await?;

            if let Some(quote2_data) = quote2 {
                let final_amount: u64 = quote2_data["outAmount"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);

                if final_amount > amount {
                    let profit_lamports = final_amount - amount;
                    let profit_percentage = (profit_lamports as f64 / amount as f64) * 100.0;

                    // Calculate ALL fees
                    let transaction_fees = 10000u64;
                    let jupiter_fees = self.calculate_jupiter_fees(&quote1_data)
                        + self.calculate_jupiter_fees(&quote2_data);
                    let priority_fees = 50000u64;
                    let rent_fees = 4000u64;
                    let total_fees = transaction_fees + jupiter_fees + priority_fees + rent_fees;

                    let net_profit = if profit_lamports > total_fees {
                        profit_lamports - total_fees
                    } else {
                        0 // No profit after fees
                    };

                    return Ok(Some(TestOpportunity {
                        input_mint: input_mint.to_string(),
                        output_mint: output_mint.to_string(),
                        intermediate_mint: intermediate_mint.to_string(),
                        amount_in: amount,
                        expected_amount_out: final_amount,
                        profit_lamports,
                        profit_percentage,
                        all_fees: total_fees,
                        net_profit,
                        route: vec![
                            self.get_token_symbol(input_mint),
                            self.get_token_symbol(intermediate_mint),
                            self.get_token_symbol(output_mint),
                        ],
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn get_jupiter_quote(
        &mut self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<Option<Value>> {
        // Enforce rate limiting
        self.enforce_rate_limit().await;

        let slippage_bps =
            self.calculate_safe_slippage(amount, &format!("{}/{}", input_mint, output_mint));

        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            JUPITER_API_BASE, input_mint, output_mint, amount, slippage_bps
        );

        match self.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let quote: Value = response.json().await?;
                    Ok(Some(quote))
                } else {
                    warn!("Jupiter API error: {}", response.status());
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("Failed to get Jupiter quote: {}", e);
                Ok(None)
            }
        }
    }

    async fn enforce_rate_limit(&mut self) {
        let elapsed = self.rate_limiter.elapsed();
        if elapsed < Duration::from_millis(500) {
            let sleep_time = Duration::from_millis(500) - elapsed;
            tokio::time::sleep(sleep_time).await;
        }
        self.rate_limiter = std::time::Instant::now();
    }

    fn calculate_jupiter_fees(&self, quote_data: &serde_json::Value) -> u64 {
        if let Some(platform_fee) = quote_data.get("platformFee") {
            if let Some(amount) = platform_fee.get("amount") {
                if let Some(fee_str) = amount.as_str() {
                    return fee_str.parse::<u64>().unwrap_or(5000);
                } else if let Some(fee_num) = amount.as_u64() {
                    return fee_num;
                }
            }
        }
        5000 // Conservative default
    }

    fn calculate_safe_slippage(&self, amount: u64, token_pair: &str) -> u64 {
        let base_slippage = 50;

        let size_adjustment = if amount > 100_000_000 {
            20
        } else if amount > 1_000_000_000 {
            50
        } else {
            0
        };

        let liquidity_adjustment = match token_pair {
            "SOL/USDC" => 0,
            "SOL/RAY" => 10,
            "SOL/mSOL" => 20,
            _ => 30,
        };

        let total_slippage = base_slippage + size_adjustment + liquidity_adjustment;
        std::cmp::min(total_slippage, 200)
    }

    fn get_token_symbol(&self, mint: &str) -> String {
        match mint {
            SOL_MINT => "SOL".to_string(),
            USDC_MINT => "USDC".to_string(),
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => "RAY".to_string(),
            "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So" => "mSOL".to_string(),
            _ => format!("TOKEN({})", &mint[..8]),
        }
    }
}
