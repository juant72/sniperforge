/// Virtual Portfolio Manager - Advanced MainNet Paper Trading
/// 
/// Sistema robusto para gestionar portfolios virtuales con datos reales de MainNet

use anyhow::{Result, anyhow};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Virtual token balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualBalance {
    pub token_mint: String,
    pub symbol: String,
    pub amount: f64,
    pub average_buy_price: f64,
    pub current_price: f64,
    pub value_usd: f64,
    pub unrealized_pnl: f64,
    pub last_updated: u64,
}

/// Virtual trade record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualTrade {
    pub trade_id: String,
    pub timestamp: u64,
    pub trade_type: TradeType,
    pub input_token: String,
    pub output_token: String,
    pub input_amount: f64,
    pub output_amount: f64,
    pub input_price: f64,
    pub output_price: f64,
    pub simulated_fee: f64,
    pub simulated_slippage: f64,
    pub realized_pnl: f64,
    pub portfolio_value_before: f64,
    pub portfolio_value_after: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeType {
    Buy,
    Sell,
    Swap,
}

/// Portfolio performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMetrics {
    pub total_value_usd: f64,
    pub initial_value_usd: f64,
    pub total_pnl: f64,
    pub total_pnl_percent: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub average_trade_pnl: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
}

/// Virtual Portfolio Manager
#[derive(Debug)]
pub struct VirtualPortfolioManager {
    balances: HashMap<String, VirtualBalance>,
    trade_history: Vec<VirtualTrade>,
    initial_value: f64,
    performance_history: Vec<(u64, f64)>, // timestamp, portfolio_value
    settings: PortfolioSettings,
}

#[derive(Debug, Clone)]
pub struct PortfolioSettings {
    pub fee_bps: u16,           // Trading fees in basis points
    pub slippage_percent: f64,  // Simulated slippage
    pub min_trade_size: f64,    // Minimum trade size in USD
    pub max_position_size: f64, // Max position as % of portfolio
}

impl Default for PortfolioSettings {
    fn default() -> Self {
        Self {
            fee_bps: 30,              // 0.3% realistic fees
            slippage_percent: 0.1,    // 0.1% slippage
            min_trade_size: 10.0,     // $10 minimum
            max_position_size: 0.25,  // 25% max position
        }
    }
}

impl VirtualPortfolioManager {
    /// Create new virtual portfolio
    pub fn new(initial_sol: f64, initial_usdc: f64, settings: PortfolioSettings) -> Self {
        let mut balances = HashMap::new();
        
        // Add initial SOL balance
        balances.insert("So11111111111111111111111111111111111111112".to_string(), VirtualBalance {
            token_mint: "So11111111111111111111111111111111111111112".to_string(),
            symbol: "SOL".to_string(),
            amount: initial_sol,
            average_buy_price: 180.0, // Estimate initial price
            current_price: 180.0,
            value_usd: initial_sol * 180.0,
            unrealized_pnl: 0.0,
            last_updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        });
        
        // Add initial USDC balance
        balances.insert("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), VirtualBalance {
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: "USDC".to_string(),
            amount: initial_usdc,
            average_buy_price: 1.0,
            current_price: 1.0,
            value_usd: initial_usdc,
            unrealized_pnl: 0.0,
            last_updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        });
        
        let initial_value = initial_sol * 180.0 + initial_usdc;
        
        Self {
            balances,
            trade_history: Vec::new(),
            initial_value,
            performance_history: vec![(
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                initial_value
            )],
            settings,
        }
    }
    
    /// Update token price
    pub fn update_price(&mut self, token_mint: &str, new_price: f64) -> Result<()> {
        if let Some(balance) = self.balances.get_mut(token_mint) {
            balance.current_price = new_price;
            balance.value_usd = balance.amount * new_price;
            balance.unrealized_pnl = (new_price - balance.average_buy_price) * balance.amount;
            balance.last_updated = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            
            debug!("📊 Updated price: {} = ${:.4}", balance.symbol, new_price);
        }
        
        // Update performance history
        let current_value = self.calculate_total_value();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.performance_history.push((timestamp, current_value));
        
        // Keep only last 1000 entries
        if self.performance_history.len() > 1000 {
            self.performance_history.remove(0);
        }
        
        Ok(())
    }
    
    /// Execute virtual trade
    pub fn execute_virtual_trade(
        &mut self,
        input_token: &str,
        output_token: &str,
        input_amount: f64,
        input_price: f64,
        output_price: f64,
    ) -> Result<VirtualTrade> {
        // Validate trade
        self.validate_trade(input_token, input_amount)?;
        
        // Calculate output amount with fees and slippage
        let gross_output = (input_amount * input_price) / output_price;
        let fee_amount = gross_output * (self.settings.fee_bps as f64 / 10000.0);
        let slippage_amount = gross_output * (self.settings.slippage_percent / 100.0);
        let net_output = gross_output - fee_amount - slippage_amount;
        
        let portfolio_value_before = self.calculate_total_value();
        
        // Update input token balance
        if let Some(input_balance) = self.balances.get_mut(input_token) {
            input_balance.amount -= input_amount;
            input_balance.value_usd = input_balance.amount * input_price;
        }
          // Update or create output token balance
        let output_symbol = self.get_token_symbol(output_token); // Get symbol before borrowing
        self.balances.entry(output_token.to_string())
            .and_modify(|balance| {
                // Weighted average price
                let total_value = balance.value_usd + (net_output * output_price);
                let total_amount = balance.amount + net_output;
                balance.average_buy_price = if total_amount > 0.0 {
                    total_value / total_amount
                } else {
                    output_price
                };
                balance.amount = total_amount;
                balance.current_price = output_price;
                balance.value_usd = total_amount * output_price;
            })
            .or_insert_with(|| VirtualBalance {
                token_mint: output_token.to_string(),
                symbol: output_symbol,
                amount: net_output,
                average_buy_price: output_price,
                current_price: output_price,
                value_usd: net_output * output_price,
                unrealized_pnl: 0.0,
                last_updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            });
        
        let portfolio_value_after = self.calculate_total_value();
        let realized_pnl = portfolio_value_after - portfolio_value_before;
        
        // Create trade record
        let trade = VirtualTrade {
            trade_id: format!("trade_{}", self.trade_history.len() + 1),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            trade_type: TradeType::Swap,
            input_token: input_token.to_string(),
            output_token: output_token.to_string(),
            input_amount,
            output_amount: net_output,
            input_price,
            output_price,
            simulated_fee: fee_amount,
            simulated_slippage: slippage_amount,
            realized_pnl,
            portfolio_value_before,
            portfolio_value_after,
        };
        
        self.trade_history.push(trade.clone());
        
        info!("💰 Virtual trade executed: {:.6} {} -> {:.6} {} (PnL: ${:.2})",
              input_amount, self.get_token_symbol(input_token),
              net_output, self.get_token_symbol(output_token),
              realized_pnl);
        
        Ok(trade)
    }
    
    /// Calculate portfolio metrics
    pub fn get_metrics(&self) -> PortfolioMetrics {
        let total_value = self.calculate_total_value();
        let total_pnl = total_value - self.initial_value;
        let total_pnl_percent = (total_pnl / self.initial_value) * 100.0;
        
        let realized_pnl: f64 = self.trade_history.iter().map(|t| t.realized_pnl).sum();
        let unrealized_pnl: f64 = self.balances.values().map(|b| b.unrealized_pnl).sum();
        
        let winning_trades = self.trade_history.iter().filter(|t| t.realized_pnl > 0.0).count() as u32;
        let losing_trades = self.trade_history.iter().filter(|t| t.realized_pnl < 0.0).count() as u32;
        let total_trades = self.trade_history.len() as u32;
        
        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else { 0.0 };
        
        let largest_win = self.trade_history.iter()
            .map(|t| t.realized_pnl)
            .fold(0.0, f64::max);
            
        let largest_loss = self.trade_history.iter()
            .map(|t| t.realized_pnl)
            .fold(0.0, f64::min);
            
        let average_trade_pnl = if total_trades > 0 {
            realized_pnl / total_trades as f64
        } else { 0.0 };
        
        // Calculate max drawdown from performance history
        let max_drawdown = self.calculate_max_drawdown();
        
        PortfolioMetrics {
            total_value_usd: total_value,
            initial_value_usd: self.initial_value,
            total_pnl,
            total_pnl_percent,
            realized_pnl,
            unrealized_pnl,
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            largest_win,
            largest_loss,
            average_trade_pnl,
            sharpe_ratio: 0.0, // TODO: Implement Sharpe ratio calculation
            max_drawdown,
        }
    }
    
    /// Get current balances
    pub fn get_balances(&self) -> &HashMap<String, VirtualBalance> {
        &self.balances
    }
    
    /// Get trade history
    pub fn get_trade_history(&self) -> &[VirtualTrade] {
        &self.trade_history
    }
    
    // Private helper methods
    
    fn calculate_total_value(&self) -> f64 {
        self.balances.values().map(|b| b.value_usd).sum()
    }
    
    fn validate_trade(&self, input_token: &str, input_amount: f64) -> Result<()> {
        if let Some(balance) = self.balances.get(input_token) {
            if balance.amount < input_amount {
                return Err(anyhow!("Insufficient balance: {} < {}", balance.amount, input_amount));
            }
        } else {
            return Err(anyhow!("Token not found in portfolio: {}", input_token));
        }
        
        let trade_value = input_amount * self.balances.get(input_token).unwrap().current_price;
        if trade_value < self.settings.min_trade_size {
            return Err(anyhow!("Trade size too small: ${} < ${}", trade_value, self.settings.min_trade_size));
        }
        
        Ok(())
    }
    
    fn get_token_symbol(&self, token_mint: &str) -> String {
        match token_mint {
            "So11111111111111111111111111111111111111112" => "SOL".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC".to_string(),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "USDT".to_string(),
            _ => format!("{}...", &token_mint[..8]),
        }
    }
    
    fn calculate_max_drawdown(&self) -> f64 {
        if self.performance_history.len() < 2 {
            return 0.0;
        }
        
        let mut max_value = self.performance_history[0].1;
        let mut max_drawdown = 0.0;
        
        for (_, value) in &self.performance_history {
            if *value > max_value {
                max_value = *value;
            }
            
            let drawdown = (max_value - value) / max_value;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
        
        max_drawdown * 100.0 // Return as percentage
    }
}

/// Print portfolio status in a nice format
impl VirtualPortfolioManager {
    pub fn print_portfolio_status(&self) {
        let metrics = self.get_metrics();
        
        println!("\n🏦 VIRTUAL PORTFOLIO STATUS (MainNet Data)");
        println!("===============================================");
        println!("💰 Total Value: ${:.2}", metrics.total_value_usd);
        println!("📈 Total P&L: ${:.2} ({:.2}%)", metrics.total_pnl, metrics.total_pnl_percent);
        println!("💵 Realized P&L: ${:.2}", metrics.realized_pnl);
        println!("📊 Unrealized P&L: ${:.2}", metrics.unrealized_pnl);
        println!("🎯 Trades: {} (✅ {} wins, ❌ {} losses)", 
                 metrics.total_trades, metrics.winning_trades, metrics.losing_trades);
        println!("🏆 Win Rate: {:.1}%", metrics.win_rate);
        
        if metrics.total_trades > 0 {
            println!("📊 Best Trade: ${:.2}", metrics.largest_win);
            println!("📉 Worst Trade: ${:.2}", metrics.largest_loss);
            println!("📈 Avg Trade: ${:.2}", metrics.average_trade_pnl);
            println!("⬇️ Max Drawdown: {:.2}%", metrics.max_drawdown);
        }
        
        println!("\n💼 Current Holdings:");
        for balance in self.balances.values() {
            if balance.amount > 0.0001 { // Only show meaningful balances
                let pnl_emoji = if balance.unrealized_pnl > 0.0 { "📈" } else if balance.unrealized_pnl < 0.0 { "📉" } else { "➡️" };
                println!("   {} {}: {:.6} (${:.2}) {} ${:.2}",
                         pnl_emoji,
                         balance.symbol,
                         balance.amount,
                         balance.value_usd,
                         if balance.unrealized_pnl != 0.0 { "P&L:" } else { "" },
                         balance.unrealized_pnl);
            }
        }
    }
}
