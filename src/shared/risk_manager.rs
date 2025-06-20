/// Risk Management System for Automated Trading
/// 
/// Provides comprehensive risk assessment and position management

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};

use super::pool_detector::TradingOpportunity;
use super::trade_executor::TradingMode;
use super::automated_trader::AutomatedTradingConfig;

/// Risk assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub approved: bool,
    pub risk_score: f64,
    pub risk_factors: Vec<String>,
    pub recommended_position_size: f64,
    pub max_acceptable_loss: f64,
}

/// Risk manager for trading decisions
pub struct RiskManager {
    config: AutomatedTradingConfig,
    daily_trades: HashMap<String, u32>, // Date -> trade count
    daily_losses: HashMap<String, f64>, // Date -> total loss
    hourly_trades: Vec<(Instant, u32)>, // Sliding window of trades
    blacklisted_tokens: Vec<String>,
    last_reset: Instant,
}

impl RiskManager {
    /// Create new risk manager
    pub async fn new(config: AutomatedTradingConfig) -> Result<Self> {
        info!("🛡️ Initializing Risk Manager");
        info!("   Max daily loss: ${:.2}", config.max_daily_loss);
        info!("   Max trades/hour: {}", config.max_trades_per_hour);
        info!("   Confidence threshold: {:.1}%", config.confidence_threshold);

        // Initialize blacklist with known risky tokens
        let blacklisted_tokens = vec![
            // Add known scam/risky token addresses here
            "So11111111111111111111111111111111111111112".to_string(), // Wrapped SOL (example)
        ];

        Ok(Self {
            config,
            daily_trades: HashMap::new(),
            daily_losses: HashMap::new(),
            hourly_trades: Vec::new(),
            blacklisted_tokens,
            last_reset: Instant::now(),
        })
    }

    /// Validate trading opportunity against risk criteria
    pub async fn validate_opportunity(&self, opportunity: &TradingOpportunity) -> Result<bool> {
        let assessment = self.assess_opportunity_risk(opportunity).await?;
        
        if !assessment.approved {
            debug!("❌ Risk assessment rejected opportunity: {:?}", assessment.risk_factors);
            return Ok(false);
        }

        // Check rate limits
        if !self.check_rate_limits().await? {
            debug!("❌ Rate limits exceeded");
            return Ok(false);
        }

        // Check daily loss limits
        if !self.check_daily_limits().await? {
            warn!("❌ Daily trading limits exceeded");
            return Ok(false);
        }

        debug!("✅ Risk assessment approved opportunity");
        Ok(true)
    }

    /// Comprehensive risk assessment of trading opportunity
    pub async fn assess_opportunity_risk(&self, opportunity: &TradingOpportunity) -> Result<RiskAssessment> {
        let mut risk_factors = Vec::new();
        let mut risk_score = 0.0;

        // 1. Check confidence score
        if opportunity.confidence_score < self.config.confidence_threshold {
            risk_factors.push(format!("Low confidence: {:.1}%", opportunity.confidence_score));
            risk_score += 30.0;
        } else {
            risk_score += (100.0 - opportunity.confidence_score) * 0.2; // Lower is better
        }

        // 2. Check price impact
        if opportunity.price_impact > 5.0 {
            risk_factors.push(format!("High price impact: {:.2}%", opportunity.price_impact));
            risk_score += 25.0;
        } else if opportunity.price_impact > 2.0 {
            risk_factors.push(format!("Moderate price impact: {:.2}%", opportunity.price_impact));
            risk_score += 10.0;
        }

        // 3. Check liquidity
        if opportunity.liquidity < 50000.0 {
            risk_factors.push(format!("Low liquidity: ${:.0}", opportunity.liquidity));
            risk_score += 20.0;
        } else if opportunity.liquidity < 100000.0 {
            risk_factors.push(format!("Moderate liquidity: ${:.0}", opportunity.liquidity));
            risk_score += 10.0;
        }

        // 4. Check for blacklisted tokens
        let token_a_str = opportunity.token_a_mint.to_string();
        let token_b_str = opportunity.token_b_mint.to_string();
        
        if self.blacklisted_tokens.contains(&token_a_str) || self.blacklisted_tokens.contains(&token_b_str) {
            risk_factors.push("Blacklisted token detected".to_string());
            risk_score += 50.0;
        }

        // 5. Check opportunity age (newer pools are riskier)
        if opportunity.pool_age_seconds.unwrap_or(0) < 300 { // Less than 5 minutes old
            risk_factors.push("Very new pool (< 5 min)".to_string());
            risk_score += 15.0;
        } else if opportunity.pool_age_seconds.unwrap_or(0) < 3600 { // Less than 1 hour old
            risk_factors.push("New pool (< 1 hour)".to_string());
            risk_score += 8.0;
        }

        // 6. Check estimated profit vs risk
        let profit_to_risk_ratio = opportunity.estimated_profit / opportunity.price_impact;
        if profit_to_risk_ratio < 10.0 {
            risk_factors.push(format!("Low profit-to-risk ratio: {:.1}", profit_to_risk_ratio));
            risk_score += 15.0;
        }

        // 7. Paper trading mode gets lower risk score
        if matches!(self.config.trading_mode, TradingMode::MainNetPaper | TradingMode::Simulation) {
            risk_score *= 0.5; // Reduce risk score for paper trading
        }

        // Calculate recommended position size based on risk
        let base_position = self.config.max_position_size;
        let risk_multiplier = if risk_score < 20.0 {
            1.0 // Full position for low risk
        } else if risk_score < 40.0 {
            0.7 // Reduced position for moderate risk
        } else if risk_score < 60.0 {
            0.4 // Small position for high risk
        } else {
            0.1 // Minimal position for very high risk
        };

        let recommended_position_size = base_position * risk_multiplier;
        let max_acceptable_loss = recommended_position_size * (self.config.stop_loss_percentage / 100.0);

        // Determine if opportunity is approved
        let approved = risk_score < 70.0 && // Risk score threshold
                      opportunity.confidence_score >= self.config.confidence_threshold &&
                      opportunity.estimated_profit >= self.config.min_profit_target &&
                      !self.blacklisted_tokens.contains(&token_a_str) &&
                      !self.blacklisted_tokens.contains(&token_b_str);

        debug!("🎯 Risk assessment complete:");
        debug!("   Risk score: {:.1}", risk_score);
        debug!("   Approved: {}", approved);
        debug!("   Position size: ${:.2}", recommended_position_size);

        Ok(RiskAssessment {
            approved,
            risk_score,
            risk_factors,
            recommended_position_size,
            max_acceptable_loss,
        })
    }

    /// Check if within hourly rate limits
    async fn check_rate_limits(&self) -> Result<bool> {
        let one_hour_ago = Instant::now() - Duration::from_secs(3600);
        
        // Count trades in the last hour
        let recent_trades: u32 = self.hourly_trades
            .iter()
            .filter(|(timestamp, _)| *timestamp > one_hour_ago)
            .map(|(_, count)| count)
            .sum();

        let within_limits = recent_trades < self.config.max_trades_per_hour;
        
        if !within_limits {
            warn!("⚠️ Hourly rate limit reached: {} trades", recent_trades);
        }

        Ok(within_limits)
    }

    /// Check daily trading limits
    async fn check_daily_limits(&self) -> Result<bool> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        
        // Check daily trade count (if we want to add this limit)
        let daily_trade_count = self.daily_trades.get(&today).unwrap_or(&0);
        
        // Check daily losses
        let daily_loss = self.daily_losses.get(&today).unwrap_or(&0.0);
        
        let within_limits = *daily_loss > -self.config.max_daily_loss;
        
        if !within_limits {
            warn!("⚠️ Daily loss limit reached: ${:.2}", daily_loss);
        }

        debug!("📊 Daily limits check: trades={}, loss=${:.2}, within_limits={}", 
               daily_trade_count, daily_loss, within_limits);

        Ok(within_limits)
    }

    /// Record a trade execution for rate limiting
    pub async fn record_trade(&mut self, profit_loss: f64) -> Result<()> {
        let now = Instant::now();
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

        // Record for hourly rate limiting
        self.hourly_trades.push((now, 1));
        
        // Clean up old entries (older than 1 hour)
        let one_hour_ago = now - Duration::from_secs(3600);
        self.hourly_trades.retain(|(timestamp, _)| *timestamp > one_hour_ago);

        // Record for daily tracking
        *self.daily_trades.entry(today.clone()).or_insert(0) += 1;
        *self.daily_losses.entry(today).or_insert(0.0) += profit_loss;

        debug!("📝 Trade recorded: P&L=${:.2}", profit_loss);

        Ok(())
    }

    /// Add token to blacklist
    pub async fn blacklist_token(&mut self, token_address: String) -> Result<()> {
        if !self.blacklisted_tokens.contains(&token_address) {
            self.blacklisted_tokens.push(token_address.clone());
            warn!("⚫ Token blacklisted: {}", token_address);
        }
        Ok(())
    }

    /// Remove token from blacklist
    pub async fn whitelist_token(&mut self, token_address: String) -> Result<()> {
        self.blacklisted_tokens.retain(|addr| addr != &token_address);
        info!("⚪ Token whitelisted: {}", token_address);
        Ok(())
    }

    /// Get current risk statistics
    pub async fn get_risk_stats(&self) -> Result<String> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let daily_trades = self.daily_trades.get(&today).unwrap_or(&0);
        let daily_loss = self.daily_losses.get(&today).unwrap_or(&0.0);
        
        let one_hour_ago = Instant::now() - Duration::from_secs(3600);
        let hourly_trades: u32 = self.hourly_trades
            .iter()
            .filter(|(timestamp, _)| *timestamp > one_hour_ago)
            .map(|(_, count)| count)
            .sum();

        let stats = format!(
            "🛡️ RISK MANAGEMENT STATS\n\
             📅 Today's trades: {}\n\
             💸 Today's P&L: ${:.2}\n\
             ⏰ Last hour trades: {}\n\
             🚫 Blacklisted tokens: {}\n\
             📊 Max daily loss: ${:.2}\n\
             📈 Max trades/hour: {}",
            daily_trades,
            daily_loss,
            hourly_trades,
            self.blacklisted_tokens.len(),
            self.config.max_daily_loss,
            self.config.max_trades_per_hour
        );

        Ok(stats)
    }

    /// Reset daily counters (called at midnight)
    pub async fn reset_daily_counters(&mut self) -> Result<()> {
        let yesterday = (chrono::Utc::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d").to_string();
        
        // Keep only today's data
        self.daily_trades.retain(|date, _| date != &yesterday);
        self.daily_losses.retain(|date, _| date != &yesterday);
        
        info!("🔄 Daily risk counters reset");
        Ok(())
    }
}
