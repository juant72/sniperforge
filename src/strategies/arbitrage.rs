use super::{TradingStrategy, StrategySignal, StrategyPerformance, StrategyConfig, SignalType, Timeframe, MarketData, TradeResult, RiskLevel};
use crate::shared::pool_detector::{TradingOpportunity, OpportunityType};
use anyhow::Result;
use std::collections::HashMap;

pub struct ArbitrageStrategy {
    config: StrategyConfig,
    performance: StrategyPerformance,
    enabled: bool,
    price_feeds: HashMap<String, f64>, // DEX -> Price mapping
}

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit_percentage: f64,
    pub estimated_profit: f64,
    pub liquidity_buy: f64,
    pub liquidity_sell: f64,
    pub confidence: f64,
}

impl ArbitrageStrategy {
    pub fn new() -> Self {
        let config = StrategyConfig {
            name: "Arbitrage".to_string(),
            enabled: true,
            capital_allocation: 0.15, // 15% allocation
            risk_level: RiskLevel::Conservative,
            max_position_size: 200.0,
            stop_loss_percent: 1.0, // Very tight for arbitrage
            take_profit_percent: 3.0, // Quick profits
            min_confidence: 0.8, // High confidence required
            timeframes: vec![Timeframe::OneMin], // Arbitrage needs immediate execution
        };

        let performance = StrategyPerformance {
            strategy_name: "Arbitrage".to_string(),
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            total_profit_loss: 0.0,
            win_rate: 0.0,
            average_profit: 0.0,
            average_loss: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            total_fees: 0.0,
            last_updated: chrono::Utc::now(),
        };

        Self {
            config,
            performance,
            enabled: true,
            price_feeds: HashMap::new(),
        }
    }

    pub fn with_config(config: StrategyConfig) -> Self {
        let performance = StrategyPerformance {
            strategy_name: config.name.clone(),
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            total_profit_loss: 0.0,
            win_rate: 0.0,
            average_profit: 0.0,
            average_loss: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            total_fees: 0.0,
            last_updated: chrono::Utc::now(),
        };

        Self {
            config,
            performance,
            enabled: true,
            price_feeds: HashMap::new(),
        }
    }

    pub fn update_price_feed(&mut self, exchange: String, price: f64) {
        self.price_feeds.insert(exchange, price);
    }

    pub fn clear_old_prices(&mut self) {
        // In a real implementation, you'd track timestamps and remove old prices
        // For now, we'll keep all prices (they should be refreshed regularly)
    }

    fn calculate_transaction_costs(&self, amount: f64, exchange: &str) -> f64 {
        // Estimate transaction costs (trading fees, slippage, gas fees)
        let base_fee = match exchange {
            "Raydium" => 0.0025,  // 0.25%
            "Orca" => 0.003,      // 0.30%
            "Jupiter" => 0.002,   // 0.20% (aggregator)
            "Serum" => 0.0022,    // 0.22%
            _ => 0.003,           // Default 0.30%
        };

        let slippage = if amount > 1000.0 { 0.001 } else { 0.0005 }; // Estimate slippage
        let gas_fee = 0.00001; // Solana gas fee in SOL terms (very low)

        amount * (base_fee + slippage) + gas_fee
    }

    fn detect_arbitrage_opportunities(&self, market_data: &MarketData) -> Vec<ArbitrageOpportunity> {
        let mut opportunities = Vec::new();

        // Create synthetic price feeds based on current market data
        let mut exchanges = HashMap::new();
        exchanges.insert("Raydium".to_string(), market_data.current_price * (1.0 + (rand::random::<f64>() - 0.5) * 0.002));
        exchanges.insert("Orca".to_string(), market_data.current_price * (1.0 + (rand::random::<f64>() - 0.5) * 0.002));
        exchanges.insert("Jupiter".to_string(), market_data.current_price * (1.0 + (rand::random::<f64>() - 0.5) * 0.002));
        exchanges.insert("Serum".to_string(), market_data.current_price * (1.0 + (rand::random::<f64>() - 0.5) * 0.002));

        // Also use stored price feeds if available
        for (exchange, price) in &self.price_feeds {
            exchanges.insert(exchange.clone(), *price);
        }

        // Find price differences between exchanges
        let exchange_list: Vec<(String, f64)> = exchanges.into_iter().collect();
        
        for i in 0..exchange_list.len() {
            for j in i+1..exchange_list.len() {
                let (ref exchange1, price1) = exchange_list[i];
                let (ref exchange2, price2) = exchange_list[j];

                let price_diff = (price2 - price1).abs();
                let profit_percentage = price_diff / price1.min(price2);

                // Only consider if price difference is significant
                if profit_percentage > 0.005 { // 0.5% minimum
                    let (buy_exchange, buy_price, sell_exchange, sell_price) = 
                        if price1 < price2 {
                            (exchange1.clone(), price1, exchange2.clone(), price2)
                        } else {
                            (exchange2.clone(), price2, exchange1.clone(), price1)
                        };

                    // Estimate liquidity (simplified)
                    let liquidity_buy = market_data.liquidity * 0.8; // Assume 80% available
                    let liquidity_sell = market_data.liquidity * 0.8;

                    // Calculate transaction costs
                    let amount = self.config.max_position_size;
                    let buy_costs = self.calculate_transaction_costs(amount, &buy_exchange);
                    let sell_costs = self.calculate_transaction_costs(amount, &sell_exchange);
                    let total_costs = buy_costs + sell_costs;

                    let gross_profit = amount * profit_percentage;
                    let net_profit = gross_profit - total_costs;

                    if net_profit > 0.0 {
                        // Calculate confidence based on various factors
                        let mut confidence = 0.0;

                        // Price difference factor
                        confidence += (profit_percentage * 20.0).min(0.4);

                        // Liquidity factor
                        if liquidity_buy > amount * 2.0 && liquidity_sell > amount * 2.0 {
                            confidence += 0.3;
                        } else if liquidity_buy > amount && liquidity_sell > amount {
                            confidence += 0.2;
                        } else {
                            confidence += 0.1;
                        }

                        // Market conditions factor
                        if market_data.bid_ask_spread < 0.005 {
                            confidence += 0.1;
                        }

                        // Volume factor
                        if market_data.volume_24h > market_data.current_price * 100000.0 {
                            confidence += 0.1;
                        }

                        // Time sensitivity factor (arbitrage opportunities are time-sensitive)
                        confidence += 0.1;

                        confidence = confidence.min(1.0);

                        opportunities.push(ArbitrageOpportunity {
                            buy_exchange,
                            sell_exchange,
                            buy_price,
                            sell_price,
                            profit_percentage,
                            estimated_profit: net_profit,
                            liquidity_buy,
                            liquidity_sell,
                            confidence,
                        });
                    }
                }
            }
        }

        // Sort by estimated profit (highest first)
        opportunities.sort_by(|a, b| b.estimated_profit.partial_cmp(&a.estimated_profit).unwrap());

        opportunities
    }

    fn detect_triangular_arbitrage(&self, market_data: &MarketData) -> Vec<ArbitrageOpportunity> {
        // Simplified triangular arbitrage detection
        // In a real implementation, this would analyze paths like SOL -> USDC -> Token -> SOL
        let mut opportunities = Vec::new();

        // For demonstration, we'll create a simplified triangular opportunity
        let base_price = market_data.current_price;
        
        // Simulate a triangular path with small price inefficiencies
        let path_efficiency = 0.997; // 0.3% inefficiency across the path
        
        if path_efficiency < 0.998 { // If path has inefficiency
            let profit_percentage = 1.0 - path_efficiency;
            let amount = self.config.max_position_size * 0.5; // Smaller amounts for triangular
            
            let estimated_costs = amount * 0.005; // Estimate total transaction costs
            let gross_profit = amount * profit_percentage;
            let net_profit = gross_profit - estimated_costs;

            if net_profit > 0.0 {
                opportunities.push(ArbitrageOpportunity {
                    buy_exchange: "Path_Start".to_string(),
                    sell_exchange: "Path_End".to_string(),
                    buy_price: base_price,
                    sell_price: base_price * (1.0 + profit_percentage),
                    profit_percentage,
                    estimated_profit: net_profit,
                    liquidity_buy: market_data.liquidity * 0.3,
                    liquidity_sell: market_data.liquidity * 0.3,
                    confidence: 0.6, // Lower confidence for triangular
                });
            }
        }

        opportunities
    }

    fn select_best_arbitrage<'a>(&self, opportunities: &'a [ArbitrageOpportunity]) -> Option<&'a ArbitrageOpportunity> {
        if opportunities.is_empty() {
            return None;
        }

        // Score opportunities based on profit, confidence, and execution feasibility
        let mut best_opportunity: Option<&ArbitrageOpportunity> = None;
        let mut best_score = 0.0;

        for opportunity in opportunities {
            let mut score = 0.0;

            // Profit factor (40%)
            score += opportunity.estimated_profit * 0.4;

            // Confidence factor (30%)
            score += opportunity.confidence * 0.3;

            // Liquidity factor (20%)
            let min_liquidity = opportunity.liquidity_buy.min(opportunity.liquidity_sell);
            if min_liquidity > self.config.max_position_size * 3.0 {
                score += 0.2;
            } else if min_liquidity > self.config.max_position_size * 2.0 {
                score += 0.15;
            } else if min_liquidity > self.config.max_position_size {
                score += 0.1;
            }

            // Speed factor (10%) - prefer direct arbitrage over triangular
            if !opportunity.buy_exchange.contains("Path") {
                score += 0.1;
            }

            if score > best_score {
                best_score = score;
                best_opportunity = Some(opportunity);
            }
        }

        best_opportunity
    }
}

impl TradingStrategy for ArbitrageStrategy {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn analyze(&self, opportunity: &TradingOpportunity, market_data: &MarketData) -> Result<Option<StrategySignal>> {
        // Arbitrage requires real-time price data from multiple sources
        // For this analysis, we'll simulate having multiple exchange prices

        // Detect direct arbitrage opportunities
        let mut all_opportunities = self.detect_arbitrage_opportunities(market_data);
        
        // Detect triangular arbitrage opportunities
        let triangular_opportunities = self.detect_triangular_arbitrage(market_data);
        all_opportunities.extend(triangular_opportunities);

        // Select the best opportunity
        let best_opportunity = match self.select_best_arbitrage(&all_opportunities) {
            Some(opp) => opp,
            None => return Ok(None), // No profitable arbitrage found
        };

        // Only proceed if the opportunity meets our minimum requirements
        if best_opportunity.confidence < self.config.min_confidence {
            return Ok(None);
        }

        if best_opportunity.estimated_profit < 1.0 { // Minimum $1 profit
            return Ok(None);
        }

        // Create arbitrage signal
        // For arbitrage, we typically execute both sides simultaneously
        // This signal represents the "buy low" side of the arbitrage
        let signal_type = SignalType::Buy;

        // Position size should not exceed available liquidity
        let max_liquidity = best_opportunity.liquidity_buy.min(best_opportunity.liquidity_sell);
        let position_size = self.config.max_position_size.min(max_liquidity * 0.8);

        // For arbitrage, stop loss should be very tight (execution failure)
        let stop_loss = Some(best_opportunity.buy_price * (1.0 - self.config.stop_loss_percent / 100.0));
        
        // Take profit is the sell price minus estimated costs
        let take_profit_price = best_opportunity.sell_price * 0.995; // Account for execution slippage
        let take_profit = Some(take_profit_price);

        // Create detailed metadata
        let mut metadata = HashMap::new();
        metadata.insert("arbitrage_type".to_string(), "direct".to_string());
        metadata.insert("buy_exchange".to_string(), best_opportunity.buy_exchange.clone());
        metadata.insert("sell_exchange".to_string(), best_opportunity.sell_exchange.clone());
        metadata.insert("buy_price".to_string(), format!("{:.6}", best_opportunity.buy_price));
        metadata.insert("sell_price".to_string(), format!("{:.6}", best_opportunity.sell_price));
        metadata.insert("profit_percentage".to_string(), format!("{:.4}", best_opportunity.profit_percentage * 100.0));
        metadata.insert("estimated_profit".to_string(), format!("{:.2}", best_opportunity.estimated_profit));
        metadata.insert("liquidity_buy".to_string(), format!("{:.2}", best_opportunity.liquidity_buy));
        metadata.insert("liquidity_sell".to_string(), format!("{:.2}", best_opportunity.liquidity_sell));

        let signal = StrategySignal {
            strategy_name: self.config.name.clone(),
            signal_type,
            confidence: best_opportunity.confidence,
            timeframe: Timeframe::OneMin, // Arbitrage requires immediate execution
            entry_price: best_opportunity.buy_price,
            stop_loss,
            take_profit,
            position_size,
            timestamp: chrono::Utc::now(),
            metadata,
        };

        println!("⚡ Arbitrage Signal: Buy on {} at {:.6}, Sell on {} at {:.6} | Profit: {:.2}% (${:.2})", 
            best_opportunity.buy_exchange,
            best_opportunity.buy_price,
            best_opportunity.sell_exchange,
            best_opportunity.sell_price,
            best_opportunity.profit_percentage * 100.0,
            best_opportunity.estimated_profit);

        Ok(Some(signal))
    }

    fn update_performance(&mut self, trade_result: &TradeResult) -> Result<()> {
        self.performance.total_trades += 1;
        self.performance.total_profit_loss += trade_result.profit_loss;
        self.performance.total_fees += trade_result.fees;

        if trade_result.success {
            self.performance.winning_trades += 1;
            if self.performance.average_profit == 0.0 {
                self.performance.average_profit = trade_result.profit_loss;
            } else {
                self.performance.average_profit = 
                    (self.performance.average_profit * (self.performance.winning_trades - 1) as f64 + trade_result.profit_loss) 
                    / self.performance.winning_trades as f64;
            }
        } else {
            self.performance.losing_trades += 1;
            if self.performance.average_loss == 0.0 {
                self.performance.average_loss = trade_result.profit_loss.abs();
            } else {
                self.performance.average_loss = 
                    (self.performance.average_loss * (self.performance.losing_trades - 1) as f64 + trade_result.profit_loss.abs()) 
                    / self.performance.losing_trades as f64;
            }
        }

        self.performance.win_rate = self.performance.winning_trades as f64 / self.performance.total_trades as f64;
        self.performance.last_updated = chrono::Utc::now();

        // Arbitrage should have a high win rate and consistent profits
        if self.performance.total_trades > 5 {
            let avg_return = self.performance.total_profit_loss / self.performance.total_trades as f64;
            let risk_free_rate = 0.02;
            // Arbitrage typically has lower volatility, so better Sharpe ratios
            self.performance.sharpe_ratio = (avg_return - risk_free_rate) / (avg_return * 0.05);
        }

        Ok(())
    }

    fn get_performance(&self) -> &StrategyPerformance {
        &self.performance
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_config(&self) -> &StrategyConfig {
        &self.config
    }
}
