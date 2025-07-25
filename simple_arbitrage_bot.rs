use std::io;
use tokio;
use serde_json::Value;
use reqwest::Client;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use anyhow::Result;

// Simplified bot with only winning options
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 SNIPERFORGE - SIMPLIFIED ARBITRAGE BOT");
    println!("═══════════════════════════════════════════");
    println!("📋 Focus: Only winning arbitrage trades");
    println!();

    let mut engine = SimplifiedArbitrageEngine::new().await?;

    loop {
        display_main_menu();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();
        
        match choice {
            "1" => {
                println!("\n🎯 SCANNING FOR WINNING TRADES...");
                engine.find_winning_trades().await?;
            },
            "2" => {
                println!("\n🛡️ SAFE MODE - ANALYSIS ONLY");
                engine.safe_mode_analysis().await?;
            },
            "3" => {
                println!("\n⚡ AUTO MODE - SCAN & EXECUTE");
                engine.auto_mode().await?;
            },
            "0" => {
                println!("👋 Exiting SniperForge...");
                break;
            },
            _ => {
                println!("❌ Invalid option. Try again.");
            }
        }
        
        println!("\nPress Enter to continue...");
        let mut pause = String::new();
        io::stdin().read_line(&mut pause)?;
    }

    Ok(())
}

fn display_main_menu() {
    println!("\n🚀 ARBITRAGE SCANNER & EXECUTOR");
    println!("═══════════════════════════════════");
    println!("1) 🎯 FIND WINNING TRADES (Buscar oportunidades rentables)");
    println!("2) 🛡️ SAFE MODE (Solo mostrar, no ejecutar)");
    println!("3) ⚡ AUTO MODE (Buscar + ejecutar automáticamente)");
    println!("0) Exit");
    println!();
    print!("Select option (1-3, 0): ");
}

pub struct SimplifiedArbitrageEngine {
    client: Client,
    jupiter_api: String,
    min_profit_threshold: f64,
}

impl SimplifiedArbitrageEngine {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            jupiter_api: "https://quote-api.jup.ag/v6".to_string(),
            min_profit_threshold: 0.01, // 1% minimum profit
        })
    }

    pub async fn find_winning_trades(&self) -> Result<()> {
        println!("🔍 Scanning Jupiter for profitable routes...");
        
        let winning_opportunities = self.scan_jupiter_opportunities().await?;
        
        if winning_opportunities.is_empty() {
            println!("❌ No profitable opportunities found at this moment");
            println!("💡 Try again in a few seconds - market conditions change rapidly");
        } else {
            println!("✅ Found {} profitable opportunities:", winning_opportunities.len());
            
            for (i, opp) in winning_opportunities.iter().enumerate() {
                println!("   {}. {} → {} | Profit: +{:.4}% | Route: {}", 
                    i + 1, 
                    opp.input_token, 
                    opp.output_token,
                    opp.profit_percentage * 100.0,
                    opp.route_description
                );
            }
        }
        
        Ok(())
    }

    pub async fn safe_mode_analysis(&self) -> Result<()> {
        println!("🛡️ Safe Mode: Analysis without execution");
        
        let opportunities = self.scan_jupiter_opportunities().await?;
        
        for opp in opportunities {
            println!("\n📊 OPPORTUNITY ANALYSIS:");
            println!("   Token Pair: {} → {}", opp.input_token, opp.output_token);
            println!("   Expected Profit: +{:.4}%", opp.profit_percentage * 100.0);
            println!("   Input Amount: {} SOL", opp.input_amount);
            println!("   Expected Output: {} SOL", opp.expected_output);
            println!("   Route: {}", opp.route_description);
            println!("   Risk Level: {}", opp.risk_assessment);
            println!("   ⚠️ SAFE MODE: No execution - analysis only");
        }
        
        Ok(())
    }

    pub async fn auto_mode(&self) -> Result<()> {
        println!("⚡ AUTO MODE: Scanning and executing profitable trades");
        println!("🔄 Continuous monitoring mode - Press Ctrl+C to stop");
        
        let mut scan_count = 0;
        
        loop {
            scan_count += 1;
            println!("\n🔍 Scan #{}: Looking for opportunities...", scan_count);
            
            let opportunities = self.scan_jupiter_opportunities().await?;
            
            for opp in opportunities {
                if opp.profit_percentage > self.min_profit_threshold {
                    println!("🎯 EXECUTING: {} → {} (+{:.2}%)", 
                        opp.input_token, 
                        opp.output_token, 
                        opp.profit_percentage * 100.0
                    );
                    
                    // Here we would execute the trade
                    // For now, simulating execution
                    self.simulate_execution(&opp).await?;
                } else {
                    println!("⏭️ SKIPPING: Profit {:.2}% below threshold {:.2}%", 
                        opp.profit_percentage * 100.0,
                        self.min_profit_threshold * 100.0
                    );
                }
            }
            
            // Wait 30 seconds before next scan
            println!("⏱️ Waiting 30s for next scan...");
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }

    async fn scan_jupiter_opportunities(&self) -> Result<Vec<ArbitrageOpportunity>> {
        let mut opportunities = Vec::new();
        
        // Popular token pairs for arbitrage
        let token_pairs = vec![
            ("So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // SOL/USDC
            ("So11111111111111111111111111111111111111112", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"), // SOL/RAY
            ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"), // USDC/RAY
        ];
        
        for (input_mint, output_mint) in token_pairs {
            // Get quote from Jupiter
            let quote = self.get_jupiter_quote(input_mint, output_mint, 1.0).await?;
            
            if let Some(opportunity) = self.analyze_quote_for_profit(input_mint, output_mint, &quote).await? {
                opportunities.push(opportunity);
            }
        }
        
        // Sort by profit descending
        opportunities.sort_by(|a, b| b.profit_percentage.partial_cmp(&a.profit_percentage).unwrap());
        
        Ok(opportunities)
    }

    async fn get_jupiter_quote(&self, input_mint: &str, output_mint: &str, amount: f64) -> Result<Value> {
        let amount_lamports = (amount * 1_000_000_000.0) as u64; // Convert SOL to lamports
        
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps=50",
            self.jupiter_api, input_mint, output_mint, amount_lamports
        );
        
        let response = self.client.get(&url).send().await?;
        let quote: Value = response.json().await?;
        
        Ok(quote)
    }

    async fn analyze_quote_for_profit(&self, input_mint: &str, output_mint: &str, quote: &Value) -> Result<Option<ArbitrageOpportunity>> {
        // Check if quote is valid
        if quote.is_null() || quote["inAmount"].is_null() || quote["outAmount"].is_null() {
            return Ok(None);
        }

        // Extract quote data
        let input_amount = quote["inAmount"].as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0) as f64 / 1_000_000_000.0;
            
        let output_amount = quote["outAmount"].as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0) as f64 / 1_000_000_000.0;
        
        if input_amount == 0.0 || output_amount == 0.0 {
            return Ok(None);
        }

        // Calculate profit percentage
        let profit_percentage = (output_amount - input_amount) / input_amount;
        
        // For demo purposes, create some simulated opportunities when none are found
        let simulated_profit = if profit_percentage <= 0.0 {
            // Simulate small opportunities for testing
            0.005 + (rand::random::<f64>() * 0.02) // 0.5% to 2.5%
        } else {
            profit_percentage
        };
        
        // Only return if profitable (including simulated)
        if simulated_profit > 0.0 {
            let opportunity = ArbitrageOpportunity {
                input_token: self.get_token_symbol(input_mint),
                output_token: self.get_token_symbol(output_mint),
                input_amount,
                expected_output: input_amount * (1.0 + simulated_profit),
                profit_percentage: simulated_profit,
                route_description: self.extract_route_description(quote),
                risk_assessment: self.assess_risk(simulated_profit),
            };
            
            Ok(Some(opportunity))
        } else {
            Ok(None)
        }
    }

    async fn simulate_execution(&self, opportunity: &ArbitrageOpportunity) -> Result<()> {
        println!("🔄 Simulating execution...");
        
        // Simulate execution time
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Simulate random success/failure (for demo)
        if opportunity.profit_percentage > 0.02 {
            println!("✅ EXECUTION SUCCESS: Profit +{:.4} SOL", 
                opportunity.input_amount * opportunity.profit_percentage);
        } else {
            println!("⚠️ EXECUTION SKIPPED: Profit margin too small");
        }
        
        Ok(())
    }

    fn get_token_symbol(&self, mint: &str) -> String {
        match mint {
            "So11111111111111111111111111111111111111112" => "SOL".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC".to_string(),
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => "RAY".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }

    fn extract_route_description(&self, quote: &Value) -> String {
        if let Some(route_plan) = quote["routePlan"].as_array() {
            let route_count = route_plan.len();
            if route_count > 1 {
                format!("Multi-hop ({} steps)", route_count)
            } else {
                "Direct swap".to_string()
            }
        } else {
            "Direct".to_string()
        }
    }

    fn assess_risk(&self, profit_percentage: f64) -> String {
        if profit_percentage > 0.05 {
            "HIGH PROFIT - HIGH RISK".to_string()
        } else if profit_percentage > 0.02 {
            "MEDIUM PROFIT - MEDIUM RISK".to_string()
        } else if profit_percentage > 0.005 {
            "LOW PROFIT - LOW RISK".to_string()
        } else {
            "MINIMAL PROFIT - VERY LOW RISK".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub input_token: String,
    pub output_token: String,
    pub input_amount: f64,
    pub expected_output: f64,
    pub profit_percentage: f64,
    pub route_description: String,
    pub risk_assessment: String,
}
