use crate::{
    config::SimpleConfig,
    types::{ArbitrageOpportunity, ApiResult as Result},
};
use std::time::Duration;
use tracing::{info, warn, error};

/// Risk management for trading operations
#[derive(Clone)]
pub struct RiskManager {
    config: SimpleConfig,
    max_position_size: f64,
    max_daily_loss: f64,
    min_confidence_score: f64,
    max_execution_time: Duration,
}

impl RiskManager {
    /// Create a new risk manager
    pub fn new(config: &SimpleConfig) -> Self {
        Self {
            config: config.clone(),
            max_position_size: config.max_position_size,
            max_daily_loss: 0.05, // 5% max daily loss
            min_confidence_score: 0.7, // Minimum 70% confidence
            max_execution_time: Duration::from_secs(60), // 1 minute max
        }
    }
    
    /// Assess the risk of an arbitrage opportunity
    pub async fn assess_opportunity(&self, opportunity: &ArbitrageOpportunity) -> Result<RiskAssessment> {
        let mut assessment = RiskAssessment::default();
        let mut risk_factors = Vec::new();
        
        // Check profit threshold
        if opportunity.profit_percentage < self.config.min_profit_threshold {
            risk_factors.push(RiskFactor::InsufficientProfit);
            assessment.is_acceptable = false;
        }
        
        // Check confidence score
        if opportunity.confidence_score < self.min_confidence_score {
            risk_factors.push(RiskFactor::LowConfidence);
            assessment.risk_score += 0.3;
        }
        
        // Check position size
        if opportunity.volume_required > self.max_position_size {
            risk_factors.push(RiskFactor::ExcessivePositionSize);
            assessment.is_acceptable = false;
        }
        
        // Check execution time window
        if opportunity.execution_time_window > self.max_execution_time {
            risk_factors.push(RiskFactor::LongExecutionTime);
            assessment.risk_score += 0.2;
        }
        
        // Check gas cost impact
        let gas_cost_impact = opportunity.estimated_gas_cost / opportunity.volume_required;
        if gas_cost_impact > 0.01 { // 1% of volume
            risk_factors.push(RiskFactor::HighGasCost);
            assessment.risk_score += 0.1;
        }
        
        // Check slippage risk
        let potential_slippage = self.estimate_slippage(opportunity);
        if potential_slippage > self.config.max_slippage {
            risk_factors.push(RiskFactor::HighSlippage);
            assessment.is_acceptable = false;
        }
        
        // Final risk assessment
        assessment.risk_factors = risk_factors;
        assessment.estimated_slippage = potential_slippage;
        assessment.max_loss = self.calculate_max_loss(opportunity);
        
        // If risk score is too high, reject
        if assessment.risk_score > 0.7 {
            assessment.is_acceptable = false;
        }
        
        // Log assessment
        if assessment.is_acceptable {
            info!("Risk assessment passed - Risk score: {:.3}", assessment.risk_score);
        } else {
            warn!("Risk assessment failed - Risk factors: {:?}", assessment.risk_factors);
        }
        
        Ok(assessment)
    }
    
    /// Estimate potential slippage
    fn estimate_slippage(&self, opportunity: &ArbitrageOpportunity) -> f64 {
        // Simplified slippage estimation based on volume
        let base_slippage = 0.001; // 0.1% base slippage
        let volume_factor = (opportunity.volume_required / 1000.0).min(1.0);
        
        base_slippage * (1.0 + volume_factor)
    }
    
    /// Calculate maximum potential loss
    fn calculate_max_loss(&self, opportunity: &ArbitrageOpportunity) -> f64 {
        let slippage_loss = self.estimate_slippage(opportunity) * opportunity.volume_required;
        let gas_cost = opportunity.estimated_gas_cost;
        let market_risk = opportunity.volume_required * 0.005; // 0.5% market risk
        
        slippage_loss + gas_cost + market_risk
    }
    
    /// Check if daily loss limit is reached
    pub async fn check_daily_loss_limit(&self, current_loss: f64) -> bool {
        current_loss < self.max_daily_loss
    }
    
    /// Update risk parameters
    pub fn update_parameters(&mut self, new_config: &SimpleConfig) {
        self.config = new_config.clone();
        self.max_position_size = new_config.max_position_size;
    }
}

/// Risk assessment result
#[derive(Debug, Clone, Default)]
pub struct RiskAssessment {
    pub is_acceptable: bool,
    pub risk_score: f64, // 0.0 to 1.0, where 1.0 is highest risk
    pub risk_factors: Vec<RiskFactor>,
    pub estimated_slippage: f64,
    pub max_loss: f64,
    pub recommendations: Vec<String>,
}

/// Risk factors that can affect trading
#[derive(Debug, Clone, PartialEq)]
pub enum RiskFactor {
    InsufficientProfit,
    LowConfidence,
    ExcessivePositionSize,
    LongExecutionTime,
    HighGasCost,
    HighSlippage,
    MarketVolatility,
    LiquidityRisk,
    TechnicalIssue,
}

impl std::fmt::Display for RiskFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskFactor::InsufficientProfit => write!(f, "Insufficient Profit"),
            RiskFactor::LowConfidence => write!(f, "Low Confidence Score"),
            RiskFactor::ExcessivePositionSize => write!(f, "Excessive Position Size"),
            RiskFactor::LongExecutionTime => write!(f, "Long Execution Time"),
            RiskFactor::HighGasCost => write!(f, "High Gas Cost"),
            RiskFactor::HighSlippage => write!(f, "High Slippage Risk"),
            RiskFactor::MarketVolatility => write!(f, "Market Volatility"),
            RiskFactor::LiquidityRisk => write!(f, "Liquidity Risk"),
            RiskFactor::TechnicalIssue => write!(f, "Technical Issue"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ArbitragePair, Token};
    
    fn create_test_config() -> SimpleConfig {
        SimpleConfig {
            solana_rpc_url: "test".to_string(),
            solana_ws_url: "test".to_string(),
            max_slippage: 0.005,
            min_profit_threshold: 0.001,
            max_position_size: 0.1,
            private_key_path: "test".to_string(),
            enable_simulation: true,
            log_level: "info".to_string(),
            dexscreener_base_url: "test".to_string(),
            max_requests_per_second: 10,
            cooldown_period_ms: 100,
        }
    }
    
    fn create_test_opportunity() -> ArbitrageOpportunity {
        ArbitrageOpportunity {
            pair: ArbitragePair {
                base_token: Token {
                    symbol: "SOL".to_string(),
                    mint: "So11111111111111111111111111111111111111112".to_string(),
                    decimals: 9,
                },
                quote_token: Token {
                    symbol: "USDC".to_string(),
                    mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                    decimals: 6,
                },
                pool_address: None,
                fee_rate: 0.003,
            },
            buy_exchange: "Raydium".to_string(),
            sell_exchange: "Orca".to_string(),
            buy_price: 100.0,
            sell_price: 100.5,
            profit_percentage: 0.005,
            volume_required: 10.0,
            estimated_gas_cost: 0.001,
            confidence_score: 0.8,
            timestamp: chrono::Utc::now(),
            execution_time_window: Duration::from_secs(30),
        }
    }
    
    #[tokio::test]
    async fn test_risk_assessment_acceptable() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(&config);
        let opportunity = create_test_opportunity();
        
        let assessment = risk_manager.assess_opportunity(&opportunity).await.unwrap();
        assert!(assessment.is_acceptable);
    }
    
    #[tokio::test]
    async fn test_risk_assessment_insufficient_profit() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(&config);
        let mut opportunity = create_test_opportunity();
        opportunity.profit_percentage = 0.0001; // Below threshold
        
        let assessment = risk_manager.assess_opportunity(&opportunity).await.unwrap();
        assert!(!assessment.is_acceptable);
        assert!(assessment.risk_factors.contains(&RiskFactor::InsufficientProfit));
    }
    
    #[tokio::test]
    async fn test_risk_assessment_excessive_position() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(&config);
        let mut opportunity = create_test_opportunity();
        opportunity.volume_required = 1.0; // Above max position size
        
        let assessment = risk_manager.assess_opportunity(&opportunity).await.unwrap();
        assert!(!assessment.is_acceptable);
        assert!(assessment.risk_factors.contains(&RiskFactor::ExcessivePositionSize));
    }
}
