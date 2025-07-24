// ===== ADVANCED ARBITRAGE SYSTEM - INTEGRACIÓN PHASE 1 + 2 =====
// Sistema unificado que combina Jupiter Advanced + MEV Protection

use anyhow::Result;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::sync::Arc;
use tracing::{info, warn, error};

// Import our implemented modules
mod modules;
use modules::{
    JupiterAdvancedEngine, JupiterAdvancedConfig, JupiterAdvancedOpportunity,
    MEVProtectionEngine, MEVProtectionConfig, BundleStatus,
    MEVRiskLevel, RecommendedAction
};

/// Advanced Arbitrage System - Integrates Phase 1 & 2
#[derive(Debug)]
pub struct AdvancedArbitrageSystem {
    jupiter_engine: JupiterAdvancedEngine,
    mev_protection: MEVProtectionEngine,
    config: SystemConfig,
    wallet_keypair: Arc<Keypair>,
    stats: SystemStats,
}

#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub min_profit_threshold: u64,
    pub max_slippage_bps: u16,
    pub priority_fee_lamports: u64,
    pub enable_mev_protection: bool,
    pub target_tokens: Vec<Pubkey>,
    pub max_opportunities_per_scan: usize,
}

#[derive(Debug, Default)]
pub struct SystemStats {
    pub total_scans: u64,
    pub opportunities_found: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_profit: f64,
    pub mev_protections_used: u64,
}

#[derive(Debug, Clone)]
pub struct IntegratedOpportunity {
    pub jupiter_opportunity: JupiterAdvancedOpportunity,
    pub mev_risk_level: MEVRiskLevel,
    pub recommended_action: RecommendedAction,
    pub estimated_profit: u64,
    pub execution_complexity: u8,
    pub priority_score: f64,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            min_profit_threshold: 100_000, // 0.0001 SOL minimum profit
            max_slippage_bps: 50,          // 0.5% max slippage
            priority_fee_lamports: 10_000,  // 0.00001 SOL priority fee
            enable_mev_protection: true,
            target_tokens: Vec::new(),
            max_opportunities_per_scan: 10,
        }
    }
}

impl AdvancedArbitrageSystem {
    /// Initialize the integrated arbitrage system
    pub async fn new(wallet_keypair: Keypair, config: Option<SystemConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();
        
        info!("🚀 Initializing Advanced Arbitrage System");
        info!("   MEV Protection: {}", if config.enable_mev_protection { "ENABLED" } else { "DISABLED" });
        info!("   Min Profit Threshold: {} lamports", config.min_profit_threshold);
        info!("   Target Tokens: {} configured", config.target_tokens.len());

        // Initialize Jupiter Advanced Engine (Phase 1)
        let jupiter_config = JupiterAdvancedConfig {
            api_endpoint: "https://quote-api.jup.ag/v6".to_string(),
            swap_endpoint: "https://quote-api.jup.ag/v6/swap".to_string(),
            max_accounts: 16,
            restrict_intermediate_tokens: true,
            intermediate_tokens: vec![
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".parse()?, // USDC
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".parse()?, // USDT
            ],
            dynamic_slippage: true,
            min_profit_threshold_bps: 10, // 0.1%
            max_route_complexity: 3,
            timeout_seconds: 30,
        };

        let jupiter_engine = JupiterAdvancedEngine::new(Some(jupiter_config)).await?;
        info!("✅ Jupiter Advanced Engine initialized");

        // Initialize MEV Protection Engine (Phase 2)
        let mev_config = MEVProtectionConfig {
            jito_url: "https://mainnet.block-engine.jito.wtf".to_string(),
            max_priority_fee: 1_000_000, // 0.001 SOL max
            min_bundle_tip: 100_000,     // 0.0001 SOL min tip
            max_bundle_wait_ms: 30_000,  // 30 seconds
            enable_sandwich_detection: true,
            enable_frontrun_protection: true,
            max_bundle_retries: 3,
            congestion_multiplier: 2.0,
        };

        let mev_protection = MEVProtectionEngine::new(Some(mev_config)).await?;
        info!("✅ MEV Protection Engine initialized");

        Ok(Self {
            jupiter_engine,
            mev_protection,
            config,
            wallet_keypair: Arc::new(wallet_keypair),
            stats: SystemStats::default(),
        })
    }

    /// Main discovery and execution loop
    pub async fn start_integrated_arbitrage(&mut self) -> Result<()> {
        info!("🎯 Starting Advanced Arbitrage System");
        info!("   Phase 1: Jupiter Auto-routing ACTIVE");
        info!("   Phase 2: MEV Protection ACTIVE");
        
        loop {
            match self.execute_arbitrage_cycle().await {
                Ok(results) => {
                    self.stats.total_scans += 1;
                    info!("✅ Scan #{}: {} opportunities processed", 
                        self.stats.total_scans, results.len());
                }
                Err(e) => {
                    error!("❌ Arbitrage cycle failed: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }

            // Wait before next scan
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }

    /// Execute complete arbitrage cycle: Discovery -> Analysis -> Execution
    async fn execute_arbitrage_cycle(&mut self) -> Result<Vec<String>> {
        let mut results = Vec::new();

        // Step 1: Discover opportunities using Jupiter Advanced (Phase 1)
        let jupiter_opportunities = self.discover_jupiter_opportunities().await?;
        
        if jupiter_opportunities.is_empty() {
            info!("📊 No Jupiter opportunities found this cycle");
            return Ok(results);
        }

        info!("🔍 Found {} Jupiter opportunities", jupiter_opportunities.len());

        // Step 2: Analyze with MEV Protection (Phase 2)
        let integrated_opportunities = self.analyze_with_mev_protection(jupiter_opportunities).await?;

        // Step 3: Execute profitable and safe opportunities
        for opportunity in integrated_opportunities {
            match self.execute_integrated_opportunity(&opportunity).await {
                Ok(signature) => {
                    results.push(signature);
                    self.stats.successful_executions += 1;
                    self.stats.total_profit += opportunity.estimated_profit as f64 / 1_000_000_000.0;
                    info!("✅ Executed opportunity: profit={} SOL", 
                        opportunity.estimated_profit as f64 / 1_000_000_000.0);
                }
                Err(e) => {
                    self.stats.failed_executions += 1;
                    warn!("⚠️ Failed to execute opportunity: {}", e);
                }
            }
        }

        // Log cycle summary
        self.log_cycle_summary(&results).await;

        Ok(results)
    }

    /// Discover opportunities using Jupiter Advanced Engine
    async fn discover_jupiter_opportunities(&mut self) -> Result<Vec<JupiterAdvancedOpportunity>> {
        let base_amount = 1_000_000_000; // 1 SOL in lamports

        // Use Jupiter Advanced auto-routing to find opportunities
        let opportunities = self.jupiter_engine
            .find_auto_routed_opportunities(base_amount)
            .await?;

        self.stats.opportunities_found += opportunities.len() as u64;

        Ok(opportunities)
    }

    /// Analyze Jupiter opportunities with MEV protection
    async fn analyze_with_mev_protection(
        &self, 
        jupiter_opportunities: Vec<JupiterAdvancedOpportunity>
    ) -> Result<Vec<IntegratedOpportunity>> {
        let mut integrated_opportunities = Vec::new();

        for jupiter_opp in jupiter_opportunities {
            // Analyze MEV risk
            let mev_analysis = self.mev_protection
                .analyze_opportunity_risk(&jupiter_opp)
                .await?;

            // Only proceed with low to medium risk opportunities
            let should_proceed = match mev_analysis.risk_level {
                MEVRiskLevel::Low | MEVRiskLevel::Medium => true,
                MEVRiskLevel::High | MEVRiskLevel::Critical => {
                    warn!("🛡️ Skipping high-risk opportunity due to MEV concerns");
                    false
                }
            };

            if should_proceed && jupiter_opp.estimated_profit > self.config.min_profit_threshold {
                let priority_score = self.calculate_priority_score(&jupiter_opp, &mev_analysis);
                
                integrated_opportunities.push(IntegratedOpportunity {
                    jupiter_opportunity: jupiter_opp.clone(),
                    mev_risk_level: mev_analysis.risk_level,
                    recommended_action: mev_analysis.recommended_action,
                    estimated_profit: jupiter_opp.estimated_profit,
                    execution_complexity: jupiter_opp.route_complexity as u8,
                    priority_score,
                });
            }
        }

        // Sort by priority score (highest first)
        integrated_opportunities.sort_by(|a, b| {
            b.priority_score.partial_cmp(&a.priority_score).unwrap()
        });

        // Limit to max opportunities per scan
        integrated_opportunities.truncate(self.config.max_opportunities_per_scan);

        Ok(integrated_opportunities)
    }

    /// Execute integrated opportunity with MEV protection
    async fn execute_integrated_opportunity(&mut self, opportunity: &IntegratedOpportunity) -> Result<String> {
        info!("🎯 Executing opportunity:");
        info!("   Profit: {} SOL", opportunity.estimated_profit as f64 / 1_000_000_000.0);
        info!("   MEV Risk: {:?}", opportunity.mev_risk_level);
        info!("   Priority Score: {:.2}", opportunity.priority_score);

        if self.config.enable_mev_protection {
            // Use MEV protection for execution
            self.stats.mev_protections_used += 1;
            let signature = self.mev_protection
                .execute_protected_transaction(&opportunity.jupiter_opportunity)
                .await?;
            
            info!("🛡️ Executed with MEV protection: {}", signature);
            Ok(signature)
        } else {
            // Direct execution without MEV protection (for testing)
            let signature = self.execute_direct_transaction(&opportunity.jupiter_opportunity).await?;
            info!("⚡ Executed directly: {}", signature);
            Ok(signature)
        }
    }

    /// Calculate priority score for opportunity ranking
    fn calculate_priority_score(
        &self, 
        jupiter_opp: &JupiterAdvancedOpportunity, 
        mev_analysis: &crate::modules::MEVAnalysisResult
    ) -> f64 {
        let profit_score = jupiter_opp.estimated_profit as f64 / 1_000_000_000.0; // SOL
        let complexity_penalty = jupiter_opp.route_complexity as f64 * 0.1;
        
        let risk_multiplier = match mev_analysis.risk_level {
            MEVRiskLevel::Low => 1.0,
            MEVRiskLevel::Medium => 0.8,
            MEVRiskLevel::High => 0.5,
            MEVRiskLevel::Critical => 0.1,
        };

        (profit_score - complexity_penalty) * risk_multiplier
    }

    /// Direct transaction execution (fallback)
    async fn execute_direct_transaction(&self, opportunity: &JupiterAdvancedOpportunity) -> Result<String> {
        // Implement direct execution logic here
        // This would use the Jupiter swap API directly
        Ok("DIRECT_EXECUTION_SIGNATURE".to_string())
    }

    /// Log cycle summary
    async fn log_cycle_summary(&self, results: &[String]) {
        info!("\n📊 CYCLE SUMMARY:");
        info!("   Executions: {}", results.len());
        info!("   Success Rate: {:.1}%", 
            if self.stats.total_scans > 0 {
                (self.stats.successful_executions as f64 / 
                 (self.stats.successful_executions + self.stats.failed_executions) as f64) * 100.0
            } else { 0.0 }
        );
        info!("   Total Profit: {:.4} SOL", self.stats.total_profit);
        info!("   MEV Protections Used: {}", self.stats.mev_protections_used);
    }

    /// Get system statistics
    pub fn get_stats(&self) -> &SystemStats {
        &self.stats
    }

    /// Update system configuration
    pub fn update_config(&mut self, new_config: SystemConfig) {
        self.config = new_config;
        info!("🔧 System configuration updated");
    }
}

/// Create default system for quick testing
pub async fn create_default_advanced_system() -> Result<AdvancedArbitrageSystem> {
    let wallet_keypair = Keypair::new();
    let config = SystemConfig::default();
    
    AdvancedArbitrageSystem::new(wallet_keypair, Some(config)).await
}

/// Create system with custom tokens
pub async fn create_system_with_tokens(target_tokens: Vec<&str>) -> Result<AdvancedArbitrageSystem> {
    let wallet_keypair = Keypair::new();
    
    let tokens: Result<Vec<Pubkey>> = target_tokens
        .into_iter()
        .map(|t| t.parse())
        .collect();
    
    let mut config = SystemConfig::default();
    config.target_tokens = tokens?;
    config.max_opportunities_per_scan = 15;
    
    AdvancedArbitrageSystem::new(wallet_keypair, Some(config)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_initialization() {
        let result = create_default_advanced_system().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_system_with_custom_tokens() {
        let tokens = vec![
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", // RAY
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        ];
        
        let result = create_system_with_tokens(tokens).await;
        assert!(result.is_ok());
    }
}
