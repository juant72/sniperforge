//! Real-time Route Monitor for High-Frequency Trading
//! Handles live market data with sub-second refresh rates

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeRoute {
    pub id: String,
    pub tokens: Vec<String>,
    pub input_amount: String,
    pub estimated_output: String,
    pub path: Vec<DexStep>,
    pub total_fee: f64,
    pub price_impact: f64,
    pub liquidity_score: f64,
    pub latency_ms: u32,
    pub priority: u32,
    pub profitability_score: f64,
    pub risk_score: f64,
    pub execution_probability: f64,
    pub gas_estimate: String,
    pub slippage_tolerance: f64,
    pub min_received: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexStep {
    pub protocol: String,
    pub market: String,
    pub pool_address: String,
    pub fee: f64,
    #[serde(rename = "type")]
    pub pool_type: String,
    pub reserves: TokenReserves,
    pub price_impact: f64,
    pub liquidity_usd: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick_spacing: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amplification: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenReserves {
    pub token_a: String,
    pub token_b: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditions {
    pub overall_sentiment: f64,
    pub volatility_index: f64,
    pub liquidity_index: f64,
    pub gas_price_gwei: f64,
    pub network_congestion: f64,
    pub arbitrage_competition: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexStatus {
    pub status: String,
    pub latency_ms: u32,
    pub success_rate: f64,
    pub total_pools: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionParams {
    pub max_slippage: f64,
    pub max_price_impact: f64,
    pub min_liquidity_usd: f64,
    pub max_latency_ms: u32,
    pub min_execution_probability: f64,
    pub gas_limit_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteAlert {
    #[serde(rename = "type")]
    pub alert_type: String,
    pub route_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liquidity_drop: Option<f64>,
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeAnalytics {
    pub avg_profit_per_route: f64,
    pub best_performing_dex: String,
    pub most_liquid_pair: String,
    pub fastest_route_id: String,
    pub highest_yield_route_id: String,
    pub total_tvl_monitored: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMetadata {
    pub data_sources: Vec<String>,
    pub calculation_time_ms: u32,
    pub cache_hit_ratio: f64,
    pub api_calls_per_update: u32,
    pub next_refresh: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeRouteData {
    pub version: String,
    pub timestamp: u64,
    pub last_update: String,
    pub refresh_rate_ms: u32,
    pub best_route_index: u32,
    pub total_routes: u32,
    pub routes: Vec<RealtimeRoute>,
    pub market_conditions: MarketConditions,
    pub dex_status: HashMap<String, DexStatus>,
    pub execution_params: ExecutionParams,
    pub analytics: RealtimeAnalytics,
    pub alerts: Vec<RouteAlert>,
    pub metadata: RouteMetadata,
}

/// Real-time route monitor with high-frequency updates
#[derive(Debug, Clone)]
pub struct RealtimeRouteMonitor {
    file_path: String,
    data: Option<RealtimeRouteData>,
    last_refresh: Instant,
    refresh_interval: Duration,
    performance_cache: HashMap<String, f64>,
    alert_history: Vec<RouteAlert>,
}

impl RealtimeRouteMonitor {
    /// Create new real-time route monitor
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            data: None,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_millis(500),
            performance_cache: HashMap::new(),
            alert_history: Vec::new(),
        }
    }

    /// Load and refresh route data from file
    pub async fn refresh_routes(&mut self) -> Result<()> {
        let now = Instant::now();
        
        // Check if refresh is needed
        if now.duration_since(self.last_refresh) < self.refresh_interval {
            return Ok(());
        }

        // Load fresh data
        let file_content = tokio::fs::read_to_string(&self.file_path).await?;
        let new_data: RealtimeRouteData = serde_json::from_str(&file_content)?;
        
        // Process alerts
        self.process_new_alerts(&new_data.alerts);
        
        // Update data and timestamp
        self.data = Some(new_data);
        self.last_refresh = now;
        
        Ok(())
    }

    /// Get best route based on current conditions
    pub fn get_best_route(&self) -> Option<&RealtimeRoute> {
        if let Some(data) = &self.data {
            if data.best_route_index < data.routes.len() as u32 {
                return data.routes.get(data.best_route_index as usize);
            }
        }
        None
    }

    /// Get routes filtered by execution parameters
    pub fn get_executable_routes(&self) -> Vec<&RealtimeRoute> {
        if let Some(data) = &self.data {
            data.routes.iter()
                .filter(|route| {
                    route.execution_probability >= data.execution_params.min_execution_probability &&
                    route.price_impact <= data.execution_params.max_price_impact &&
                    route.latency_ms <= data.execution_params.max_latency_ms
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get routes by profitability threshold
    pub fn get_profitable_routes(&self, min_profit_score: f64) -> Vec<&RealtimeRoute> {
        if let Some(data) = &self.data {
            data.routes.iter()
                .filter(|route| route.profitability_score >= min_profit_score)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get routes for specific DEX
    pub fn get_routes_by_dex(&self, dex_name: &str) -> Vec<&RealtimeRoute> {
        if let Some(data) = &self.data {
            data.routes.iter()
                .filter(|route| {
                    route.path.iter().any(|step| step.protocol.to_lowercase() == dex_name.to_lowercase())
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get fastest routes (lowest latency)
    pub fn get_fastest_routes(&self, count: usize) -> Vec<&RealtimeRoute> {
        if let Some(data) = &self.data {
            let mut routes = data.routes.iter().collect::<Vec<_>>();
            routes.sort_by(|a, b| a.latency_ms.cmp(&b.latency_ms));
            routes.into_iter().take(count).collect()
        } else {
            Vec::new()
        }
    }

    /// Get current market conditions
    pub fn get_market_conditions(&self) -> Option<&MarketConditions> {
        self.data.as_ref().map(|d| &d.market_conditions)
    }

    /// Get DEX status information
    pub fn get_dex_status(&self, dex_name: &str) -> Option<&DexStatus> {
        self.data.as_ref()?.dex_status.get(dex_name)
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<&RouteAlert> {
        if let Some(data) = &self.data {
            let current_time = chrono::Utc::now().timestamp() as u64 * 1000;
            data.alerts.iter()
                .filter(|alert| {
                    alert.expires_at.map_or(true, |expires| expires > current_time)
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get analytics data
    pub fn get_analytics(&self) -> Option<&RealtimeAnalytics> {
        self.data.as_ref().map(|d| &d.analytics)
    }

    /// Calculate route efficiency score
    pub fn calculate_route_efficiency(&self, route: &RealtimeRoute) -> f64 {
        let profit_weight = 0.4;
        let risk_weight = 0.3;
        let speed_weight = 0.2;
        let execution_weight = 0.1;

        let profit_score = route.profitability_score;
        let risk_score = 1.0 - route.risk_score; // Invert risk (lower risk = higher score)
        let speed_score = 1.0 - (route.latency_ms as f64 / 100.0).min(1.0); // Normalize latency
        let execution_score = route.execution_probability;

        (profit_score * profit_weight) +
        (risk_score * risk_weight) +
        (speed_score * speed_weight) +
        (execution_score * execution_weight)
    }

    /// Process new alerts and store in history
    fn process_new_alerts(&mut self, new_alerts: &[RouteAlert]) {
        for alert in new_alerts {
            // Check if this is a new alert
            if !self.alert_history.iter().any(|a| a.route_id == alert.route_id && a.timestamp == alert.timestamp) {
                self.alert_history.push(alert.clone());
                
                // Log important alerts
                match alert.alert_type.as_str() {
                    "high_opportunity" => {
                        if let Some(profit) = alert.profit_percentage {
                            println!("🚨 HIGH OPPORTUNITY ALERT: Route {} - {:.2}% profit potential", 
                                     alert.route_id, profit);
                        }
                    },
                    "low_liquidity" => {
                        if let Some(drop) = alert.liquidity_drop {
                            println!("⚠️ LOW LIQUIDITY ALERT: Route {} - {:.1}% liquidity drop", 
                                     alert.route_id, drop * 100.0);
                        }
                    },
                    _ => {}
                }
            }
        }

        // Keep only recent alerts (last 1 hour)
        let one_hour_ago = (chrono::Utc::now().timestamp() as u64 - 3600) * 1000;
        self.alert_history.retain(|alert| alert.timestamp > one_hour_ago);
    }

    /// Display real-time route dashboard
    pub fn display_realtime_dashboard(&self) {
        if let Some(data) = &self.data {
            println!("\n╔══════════════════════════════════════════════════════════════════╗");
            println!("║                    REAL-TIME ROUTES DASHBOARD                   ║");
            println!("╠══════════════════════════════════════════════════════════════════╣");
            println!("║ Routes: {} │ Best: #{} │ Refresh: {}ms │ Age: {}ms       ║",
                     data.total_routes,
                     data.best_route_index,
                     data.refresh_rate_ms,
                     (chrono::Utc::now().timestamp_millis() - data.timestamp as i64));
            
            if let Some(best_route) = self.get_best_route() {
                println!("║ 🏆 BEST: {} │ Profit: {:.3} │ Risk: {:.3} │ Exec: {:.1}%   ║",
                         best_route.id,
                         best_route.profitability_score,
                         best_route.risk_score,
                         best_route.execution_probability * 100.0);
            }
            
            println!("╠══════════════════════════════════════════════════════════════════╣");
            println!("║ 📊 Market: Sentiment {:.3} │ Volatility {:.3} │ Liquidity {:.3} ║",
                     data.market_conditions.overall_sentiment,
                     data.market_conditions.volatility_index,
                     data.market_conditions.liquidity_index);
            
            // Active alerts
            let active_alerts = self.get_active_alerts();
            if !active_alerts.is_empty() {
                println!("║ 🚨 ACTIVE ALERTS: {}                                           ║", active_alerts.len());
                for alert in active_alerts.iter().take(2) {
                    let alert_emoji = match alert.alert_type.as_str() {
                        "high_opportunity" => "💰",
                        "low_liquidity" => "⚠️",
                        _ => "ℹ️",
                    };
                    println!("║   {} {} (Route: {})                                  ║",
                             alert_emoji, alert.alert_type, &alert.route_id[..12.min(alert.route_id.len())]);
                }
            }
            
            println!("╚══════════════════════════════════════════════════════════════════╝");
        } else {
            println!("⚠️ No real-time route data available");
        }
    }

    /// Get data age in milliseconds
    pub fn get_data_age_ms(&self) -> Option<u64> {
        self.data.as_ref().map(|data| {
            (chrono::Utc::now().timestamp_millis() - data.timestamp as i64) as u64
        })
    }

    /// Check if data is stale
    pub fn is_data_stale(&self, max_age_ms: u64) -> bool {
        self.get_data_age_ms().map_or(true, |age| age > max_age_ms)
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> Option<(f64, u32, f64)> {
        self.data.as_ref().map(|data| {
            (
                data.metadata.cache_hit_ratio,
                data.metadata.calculation_time_ms,
                data.analytics.avg_profit_per_route,
            )
        })
    }
}

impl Default for RealtimeRouteMonitor {
    fn default() -> Self {
        Self::new("data/realtime_routes.json")
    }
}
