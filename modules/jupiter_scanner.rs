// ===== JUPITER SCANNER MODULE =====
// Módulo para escaneo comprehensivo basado en phase4b_jupiter_scanner.rs exitoso
// Implementa detección de oportunidades reales documentadas

use anyhow::{Result, anyhow};
use tracing::{info, debug};
use std::collections::HashMap;
use reqwest;
use serde_json::Value;
use tokio::time::{Duration, sleep};
use chrono::{Utc, DateTime};

#[derive(Debug, Clone, serde::Serialize)]
pub struct OpportunityResult {
    pub timestamp: DateTime<Utc>,
    pub token_pair: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: f64,
    pub estimated_profit: f64,
    pub profit_percentage: f64,
    pub confidence_score: f64,
    pub execution_priority: Priority,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Priority {
    High,    // >5x fees, immediate execution
    Medium,  // >3x fees, good execution
    Low,     // >1.5x fees, cautious execution
    Monitor, // >fees but < 1.5x, monitor only
}

pub struct JupiterScanner {
    jupiter_url: String,
    fee_threshold_lamports: u64,
    scan_amounts: Vec<f64>,
    supported_tokens: HashMap<String, String>,
}

impl JupiterScanner {
    pub fn new() -> Self {
        let mut supported_tokens = HashMap::new();
        
        // Tokens documentados como exitosos en el histórico
        supported_tokens.insert("SOL".to_string(), "So11111111111111111111111111111111111111112".to_string());
        supported_tokens.insert("USDC".to_string(), "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string());
        supported_tokens.insert("RAY".to_string(), "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string());
        supported_tokens.insert("BONK".to_string(), "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string());

        Self {
            jupiter_url: "https://quote-api.jup.ag/v6".to_string(),
            fee_threshold_lamports: 15000, // 0.000015 SOL - documented cost
            scan_amounts: vec![0.005, 0.01, 0.03, 0.05], // Amounts from successful documentation
            supported_tokens,
        }
    }

    /// Comprehensive market scan - basado en documentación exitosa
    pub async fn scan_all_opportunities(&self) -> Result<Vec<OpportunityResult>> {
        info!("🔍 Iniciando Jupiter Scanner - Búsqueda comprehensiva");
        info!("📊 Escaneando {} tokens en {} amounts diferentes", 
              self.supported_tokens.len(), 
              self.scan_amounts.len());

        let mut all_opportunities = Vec::new();
        let scan_start = Utc::now();

        // Scan all documented successful pairs
        for amount in &self.scan_amounts {
            let opportunities = self.scan_amount_level(*amount).await?;
            all_opportunities.extend(opportunities);
        }

        // Sort by profit potential
        all_opportunities.sort_by(|a, b| b.estimated_profit.partial_cmp(&a.estimated_profit).unwrap());

        let scan_duration = Utc::now().signed_duration_since(scan_start);
        self.log_scan_summary(&all_opportunities, scan_duration.num_milliseconds());

        Ok(all_opportunities)
    }

    /// Scan specific amount level across all pairs
    async fn scan_amount_level(&self, amount: f64) -> Result<Vec<OpportunityResult>> {
        debug!("📈 Escaneando amount level: {} SOL", amount);
        let mut opportunities = Vec::new();

        // Define documented successful trading pairs
        let trading_pairs = vec![
            ("SOL", "BONK"),
            ("SOL", "RAY"),
            ("SOL", "USDC"),
            ("USDC", "RAY"),
            ("RAY", "BONK"),
        ];

        for (token_a, token_b) in trading_pairs {
            if let Ok(opportunity) = self.analyze_arbitrage_pair(token_a, token_b, amount).await {
                if opportunity.estimated_profit > 0.0 {
                    opportunities.push(opportunity);
                }
            }
            
            // Rate limiting to avoid API overload
            sleep(Duration::from_millis(200)).await;
        }

        Ok(opportunities)
    }

    /// Analyze specific arbitrage pair - implementación real
    async fn analyze_arbitrage_pair(&self, token_a: &str, token_b: &str, amount: f64) -> Result<OpportunityResult> {
        let mint_a = self.supported_tokens.get(token_a)
            .ok_or_else(|| anyhow!("Unsupported token: {}", token_a))?;
        let mint_b = self.supported_tokens.get(token_b)
            .ok_or_else(|| anyhow!("Unsupported token: {}", token_b))?;

        // Execute A -> B -> A cycle for real profit calculation
        let step1_result = self.get_jupiter_quote_real(mint_a, mint_b, amount).await?;
        let intermediate_amount = step1_result / self.get_token_decimals_multiplier(mint_b);
        
        let step2_result = self.get_jupiter_quote_real(mint_b, mint_a, intermediate_amount).await?;
        let final_amount = step2_result / self.get_token_decimals_multiplier(mint_a);

        let profit = final_amount - amount;
        let profit_percentage = (profit / amount) * 100.0;
        let profit_lamports = (profit * 1_000_000_000.0) as i64;

        // Calculate confidence and priority based on documented thresholds
        let fee_multiplier = profit_lamports as f64 / self.fee_threshold_lamports as f64;
        let confidence_score = self.calculate_confidence_score(fee_multiplier, token_a, token_b);
        let priority = self.determine_priority(fee_multiplier);

        Ok(OpportunityResult {
            timestamp: Utc::now(),
            token_pair: format!("{}/{}", token_a, token_b),
            input_mint: mint_a.clone(),
            output_mint: mint_b.clone(),
            input_amount: amount,
            estimated_profit: profit,
            profit_percentage,
            confidence_score,
            execution_priority: priority,
        })
    }

    /// Get real Jupiter quote with proper error handling
    async fn get_jupiter_quote_real(&self, input_mint: &str, output_mint: &str, amount: f64) -> Result<f64> {
        let amount_lamports = (amount * self.get_token_decimals_multiplier(input_mint)) as u64;
        
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps=50",
            self.jupiter_url, input_mint, output_mint, amount_lamports
        );

        let client = reqwest::Client::new();
        let response = client.get(&url)
            .timeout(Duration::from_secs(15))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Jupiter API error: {} for pair {}->{}", 
                              response.status(), input_mint, output_mint));
        }

        let json: Value = response.json().await?;
        
        let out_amount_str = json["outAmount"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing outAmount in Jupiter response"))?;

        out_amount_str.parse::<f64>()
            .map_err(|_| anyhow!("Invalid outAmount format: {}", out_amount_str))
    }

    /// Calculate confidence score based on historical success patterns
    fn calculate_confidence_score(&self, fee_multiplier: f64, token_a: &str, token_b: &str) -> f64 {
        let base_score = (fee_multiplier * 20.0).min(100.0); // Base score from profit ratio
        
        // Adjust based on historical success patterns from documentation
        let pair_bonus = match (token_a, token_b) {
            ("SOL", "BONK") => 15.0, // Historically most successful
            ("SOL", "RAY") => 10.0,  // Good historical performance
            ("SOL", "USDC") => 5.0,  // Most stable but smaller spreads
            _ => 0.0,
        };

        (base_score + pair_bonus).min(100.0)
    }

    /// Determine execution priority based on documented thresholds
    fn determine_priority(&self, fee_multiplier: f64) -> Priority {
        if fee_multiplier >= 5.0 {
            Priority::High
        } else if fee_multiplier >= 3.0 {
            Priority::Medium
        } else if fee_multiplier >= 1.5 {
            Priority::Low
        } else {
            Priority::Monitor
        }
    }

    /// Get token decimals multiplier for accurate calculations
    fn get_token_decimals_multiplier(&self, mint: &str) -> f64 {
        match mint {
            "So11111111111111111111111111111111111111112" => 1_000_000_000.0, // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => 1_000_000.0,     // USDC
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => 1_000_000.0,     // RAY
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => 100_000.0,       // BONK (5 decimals)
            _ => 1_000_000_000.0, // Default to 9 decimals
        }
    }

    /// Log comprehensive scan summary
    fn log_scan_summary(&self, opportunities: &[OpportunityResult], duration_ms: i64) {
        let high_priority = opportunities.iter().filter(|o| o.execution_priority == Priority::High).count();
        let medium_priority = opportunities.iter().filter(|o| o.execution_priority == Priority::Medium).count();
        let low_priority = opportunities.iter().filter(|o| o.execution_priority == Priority::Low).count();
        let monitor_only = opportunities.iter().filter(|o| o.execution_priority == Priority::Monitor).count();

        info!("📊 JUPITER SCANNER RESULTS:");
        info!("   Scan completado en: {}ms", duration_ms);
        info!("   Total oportunidades: {}", opportunities.len());
        info!("   🔴 Alta prioridad (>5x fees): {}", high_priority);
        info!("   🟡 Media prioridad (>3x fees): {}", medium_priority);
        info!("   🟢 Baja prioridad (>1.5x fees): {}", low_priority);
        info!("   ⚪ Solo monitoreo (>fees): {}", monitor_only);

        // Log top 3 opportunities
        for (i, opp) in opportunities.iter().take(3).enumerate() {
            let priority_icon = match opp.execution_priority {
                Priority::High => "🔴",
                Priority::Medium => "🟡",
                Priority::Low => "🟢",
                Priority::Monitor => "⚪",
            };

            info!("   {}#{} {} ({:.3} SOL): +{:.9} SOL ({:.2}%, conf: {:.1}%)",
                priority_icon,
                i + 1,
                opp.token_pair,
                opp.input_amount,
                opp.estimated_profit,
                opp.profit_percentage,
                opp.confidence_score
            );
        }
    }

    /// Quick scan for immediate opportunities (faster version)
    pub async fn quick_scan(&self) -> Result<Vec<OpportunityResult>> {
        info!("⚡ Quick scan - Oportunidades inmediatas");
        
        // Only scan most promising pairs with optimal amounts
        let quick_pairs = vec![
            ("SOL", "BONK", 0.03), // Best historical performer
            ("SOL", "RAY", 0.005),  // Good volatility
            ("SOL", "USDC", 0.01),  // Most liquid
        ];

        let mut opportunities = Vec::new();
        
        for (token_a, token_b, amount) in quick_pairs {
            if let Ok(opp) = self.analyze_arbitrage_pair(token_a, token_b, amount).await {
                if matches!(opp.execution_priority, Priority::High | Priority::Medium) {
                    opportunities.push(opp);
                }
            }
        }

        opportunities.sort_by(|a, b| b.estimated_profit.partial_cmp(&a.estimated_profit).unwrap());
        Ok(opportunities)
    }
}

/// Public function for easy integration
pub async fn execute_comprehensive_scan() -> Result<Vec<OpportunityResult>> {
    let scanner = JupiterScanner::new();
    scanner.scan_all_opportunities().await
}

/// Public function for quick checks
pub async fn execute_quick_scan() -> Result<Vec<OpportunityResult>> {
    let scanner = JupiterScanner::new();
    scanner.quick_scan().await
}
