//! Unified Routing System - Coordinates Strategic and Real-time Route Data
//! Combines historical optimization with live market execution

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use super::route_optimizer::{RouteOptimizationEngine, OptimizedRoute};
use super::realtime_monitor::{RealtimeRouteMonitor, RealtimeRoute};

#[derive(Debug, Clone)]
pub struct UnifiedRoute {
    pub strategic_route: OptimizedRoute,
    pub realtime_data: Option<RealtimeRoute>,
    pub combined_score: f64,
    pub execution_confidence: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub selected_route: UnifiedRoute,
    pub reason: String,
    pub risk_assessment: f64,
    pub profit_estimate: f64,
    pub execution_window_seconds: u32,
}

/// Unified routing system combining strategic and real-time data
#[derive(Debug)]
pub struct UnifiedRoutingSystem {
    strategic_engine: RouteOptimizationEngine,
    realtime_monitor: Arc<RwLock<RealtimeRouteMonitor>>,
    last_decision: Option<RoutingDecision>,
    decision_cache: std::collections::HashMap<String, RoutingDecision>,
    performance_tracker: PerformanceTracker,
}

#[derive(Debug, Clone)]
struct PerformanceTracker {
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_profit: f64,
    pub avg_execution_time_ms: f64,
    pub best_performing_strategy: String,
}

impl UnifiedRoutingSystem {
    /// Create new unified routing system
    pub async fn new() -> Result<Self> {
        let strategic_engine = RouteOptimizationEngine::default();
        let realtime_monitor = Arc::new(RwLock::new(RealtimeRouteMonitor::default()));
        
        // Initial refresh of real-time data
        realtime_monitor.write().await.refresh_routes().await?;
        
        Ok(Self {
            strategic_engine,
            realtime_monitor,
            last_decision: None,
            decision_cache: std::collections::HashMap::new(),
            performance_tracker: PerformanceTracker {
                successful_executions: 0,
                failed_executions: 0,
                total_profit: 0.0,
                avg_execution_time_ms: 0.0,
                best_performing_strategy: "none".to_string(),
            },
        })
    }

    /// Get optimal trading route based on current conditions
    pub async fn get_optimal_route(
        &mut self,
        market_sentiment: f64,
        available_capital: f64,
        risk_tolerance: f64,
        execution_urgency: f64, // 0.0 = patient, 1.0 = immediate
    ) -> Result<Option<RoutingDecision>> {
        // 1. Refresh real-time data
        self.realtime_monitor.write().await.refresh_routes().await?;
        
        // 2. Get strategic routes based on market conditions
        let strategic_routes = self.strategic_engine.get_sentiment_optimized_routes(market_sentiment);
        
        // 3. Get real-time executable routes
        let realtime_routes = {
            let monitor = self.realtime_monitor.read().await;
            monitor.get_executable_routes().into_iter().cloned().collect::<Vec<_>>()
        };
        
        // 4. Create unified routes by matching strategic with real-time
        let unified_routes = self.create_unified_routes(&strategic_routes, &realtime_routes);
        
        // 5. Score and rank routes
        let mut scored_routes = Vec::new();
        for route in unified_routes {
            let score = self.calculate_unified_score(&route, market_sentiment, risk_tolerance, execution_urgency).await;
            scored_routes.push((route, score));
        }
        
        // Sort by score (highest first)
        scored_routes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // 6. Select best route and create decision
        if let Some((best_route, score)) = scored_routes.first() {
            let decision = RoutingDecision {
                selected_route: best_route.clone(),
                reason: self.generate_decision_reason(&best_route, *score).await,
                risk_assessment: best_route.strategic_route.success_rate,
                profit_estimate: (best_route.strategic_route.avg_profit_bps as f64) / 10000.0 * available_capital,
                execution_window_seconds: self.calculate_execution_window(&best_route).await,
            };
            
            self.last_decision = Some(decision.clone());
            
            // Cache decision for quick retrieval
            let cache_key = format!("{}-{:.3}-{:.0}", 
                                   best_route.strategic_route.route.join("-"), 
                                   market_sentiment, 
                                   available_capital);
            self.decision_cache.insert(cache_key, decision.clone());
            
            return Ok(Some(decision));
        }
        
        Ok(None)
    }

    /// Create unified routes by matching strategic with real-time data
    fn create_unified_routes(
        &self,
        strategic_routes: &[OptimizedRoute],
        realtime_routes: &[RealtimeRoute],
    ) -> Vec<UnifiedRoute> {
        let mut unified_routes = Vec::new();
        
        for strategic_route in strategic_routes {
            // Try to find matching real-time route
            let matching_realtime = realtime_routes.iter()
                .find(|rt_route| self.routes_match(&strategic_route.route, &rt_route.tokens))
                .cloned();
            
            // Calculate combined score
            let combined_score = self.calculate_combined_score(strategic_route, &matching_realtime);
            
            // Calculate execution confidence
            let execution_confidence = if matching_realtime.is_some() {
                strategic_route.success_rate * 0.7 + 0.3 // Boost confidence with real-time data
            } else {
                strategic_route.success_rate * 0.5 // Reduce confidence without real-time data
            };
            
            unified_routes.push(UnifiedRoute {
                strategic_route: strategic_route.clone(),
                realtime_data: matching_realtime,
                combined_score,
                execution_confidence,
                last_updated: Utc::now(),
            });
        }
        
        unified_routes
    }

    /// Check if strategic and real-time routes match
    fn routes_match(&self, strategic_tokens: &[String], realtime_tokens: &[String]) -> bool {
        strategic_tokens.len() == realtime_tokens.len() &&
        strategic_tokens.iter().zip(realtime_tokens.iter()).all(|(a, b)| a == b)
    }

    /// Calculate combined score for unified route
    fn calculate_combined_score(&self, strategic: &OptimizedRoute, realtime: &Option<RealtimeRoute>) -> f64 {
        let strategic_score = (strategic.avg_profit_bps as f64 / 100.0) * strategic.success_rate;
        
        if let Some(rt) = realtime {
            // Combine strategic and real-time scores
            let realtime_score = rt.profitability_score * rt.execution_probability;
            (strategic_score * 0.6) + (realtime_score * 0.4)
        } else {
            strategic_score * 0.5 // Penalty for no real-time data
        }
    }

    /// Calculate unified score considering all factors
    async fn calculate_unified_score(
        &self,
        unified_route: &UnifiedRoute,
        market_sentiment: f64,
        risk_tolerance: f64,
        execution_urgency: f64,
    ) -> f64 {
        let mut score = unified_route.combined_score;
        
        // Apply market sentiment multiplier
        if market_sentiment > 0.2 && unified_route.strategic_route.avg_profit_bps > 50 {
            score *= 1.2; // Boost profitable routes in bull market
        } else if market_sentiment < -0.2 {
            score *= 0.8; // Reduce all routes in bear market
        }
        
        // Apply risk tolerance
        let risk_factor = 1.0 - unified_route.strategic_route.success_rate;
        if risk_factor > risk_tolerance {
            score *= 0.5; // Heavy penalty for routes exceeding risk tolerance
        }
        
        // Apply execution urgency
        if execution_urgency > 0.7 {
            // Prefer faster routes when urgent
            if let Some(rt_data) = &unified_route.realtime_data {
                if rt_data.latency_ms < 50 {
                    score *= 1.3;
                } else if rt_data.latency_ms > 100 {
                    score *= 0.7;
                }
            }
        }
        
        // Apply execution confidence
        score *= unified_route.execution_confidence;
        
        score
    }

    /// Generate human-readable decision reason
    async fn generate_decision_reason(&self, route: &UnifiedRoute, score: f64) -> String {
        let mut reasons = Vec::new();
        
        if route.strategic_route.avg_profit_bps > 100 {
            reasons.push(format!("High profit potential: {}bps", route.strategic_route.avg_profit_bps));
        }
        
        if route.strategic_route.success_rate > 0.85 {
            reasons.push("High historical success rate".to_string());
        }
        
        if let Some(rt_data) = &route.realtime_data {
            if rt_data.execution_probability > 0.95 {
                reasons.push("Excellent real-time execution probability".to_string());
            }
            
            if rt_data.latency_ms < 30 {
                reasons.push("Ultra-low latency execution".to_string());
            }
            
            if rt_data.price_impact < 0.005 {
                reasons.push("Minimal price impact".to_string());
            }
        }
        
        if score > 1.0 {
            reasons.push("Superior risk-adjusted returns".to_string());
        }
        
        if reasons.is_empty() {
            "Best available route under current conditions".to_string()
        } else {
            reasons.join("; ")
        }
    }

    /// Calculate execution window in seconds
    async fn calculate_execution_window(&self, route: &UnifiedRoute) -> u32 {
        if let Some(rt_data) = &route.realtime_data {
            // Base window on market volatility and liquidity
            let base_window = 30; // 30 seconds base
            let volatility_multiplier = {
                let monitor = self.realtime_monitor.read().await;
                if let Some(conditions) = monitor.get_market_conditions() {
                    1.0 + conditions.volatility_index
                } else {
                    1.5
                }
            };
            
            (base_window as f64 * volatility_multiplier) as u32
        } else {
            300 // 5 minutes for routes without real-time data
        }
    }

    /// Execute selected route and track performance
    pub async fn execute_route(&mut self, decision: &RoutingDecision) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        
        // Simulate execution (replace with actual trading logic)
        let success = decision.selected_route.execution_confidence > 0.8;
        
        let execution_time_ms = start_time.elapsed().as_millis() as f64;
        
        // Update performance tracking
        if success {
            self.performance_tracker.successful_executions += 1;
            self.performance_tracker.total_profit += decision.profit_estimate;
        } else {
            self.performance_tracker.failed_executions += 1;
        }
        
        // Update average execution time
        let total_executions = self.performance_tracker.successful_executions + self.performance_tracker.failed_executions;
        self.performance_tracker.avg_execution_time_ms = 
            (self.performance_tracker.avg_execution_time_ms * (total_executions - 1) as f64 + execution_time_ms) / total_executions as f64;
        
        Ok(ExecutionResult {
            success,
            execution_time_ms,
            actual_profit: if success { decision.profit_estimate } else { 0.0 },
            route_id: decision.selected_route.strategic_route.route.join("-"),
        })
    }

    /// Get system performance statistics
    pub fn get_performance_stats(&self) -> &PerformanceTracker {
        &self.performance_tracker
    }

    /// Display unified system dashboard
    pub async fn display_unified_dashboard(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
        println!("║                         UNIFIED ROUTING SYSTEM DASHBOARD                        ║");
        println!("╠══════════════════════════════════════════════════════════════════════════════╣");
        
        // Strategic engine stats
        let strategic_settings = self.strategic_engine.get_optimization_settings();
        println!("║ 📊 Strategic Routes: {} │ ML Optimization: {} │ Sentiment: {}        ║",
                 15, // From strategic config
                 if strategic_settings.ml_route_optimization { "✅" } else { "❌" },
                 if strategic_settings.sentiment_based_routing { "✅" } else { "❌" });
        
        // Real-time monitor stats
        {
            let monitor = self.realtime_monitor.read().await;
            if let Some((cache_ratio, calc_time, avg_profit)) = monitor.get_performance_stats() {
                println!("║ ⚡ Real-time: Cache {:.1}% │ Calc: {}ms │ Avg Profit: {:.3}%      ║",
                         cache_ratio * 100.0, calc_time, avg_profit * 100.0);
            }
        }
        
        // Performance tracking
        let total_executions = self.performance_tracker.successful_executions + self.performance_tracker.failed_executions;
        let success_rate = if total_executions > 0 {
            (self.performance_tracker.successful_executions as f64 / total_executions as f64) * 100.0
        } else {
            0.0
        };
        
        println!("║ 🎯 Executions: {} │ Success: {:.1}% │ Total Profit: ${:.2}        ║",
                 total_executions, success_rate, self.performance_tracker.total_profit);
        println!("║ ⏱️ Avg Execution: {:.1}ms │ Cache Size: {}                           ║",
                 self.performance_tracker.avg_execution_time_ms, self.decision_cache.len());
        
        println!("╠══════════════════════════════════════════════════════════════════════════════╣");
        
        // Last decision info
        if let Some(decision) = &self.last_decision {
            println!("║ 🏆 LAST DECISION:                                                           ║");
            println!("║   Route: {}                                           ║",
                     decision.selected_route.strategic_route.route.join(" → "));
            println!("║   Reason: {}                                          ║",
                     &decision.reason[..60.min(decision.reason.len())]);
            println!("║   Risk: {:.1}% │ Profit Est: ${:.2} │ Window: {}s                    ║",
                     (1.0 - decision.risk_assessment) * 100.0,
                     decision.profit_estimate,
                     decision.execution_window_seconds);
        }
        
        println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub execution_time_ms: f64,
    pub actual_profit: f64,
    pub route_id: String,
}

impl Default for UnifiedRoutingSystem {
    fn default() -> Self {
        // This is a simplified default - in practice, use ::new().await.unwrap()
        Self {
            strategic_engine: RouteOptimizationEngine::default(),
            realtime_monitor: Arc::new(RwLock::new(RealtimeRouteMonitor::default())),
            last_decision: None,
            decision_cache: std::collections::HashMap::new(),
            performance_tracker: PerformanceTracker {
                successful_executions: 0,
                failed_executions: 0,
                total_profit: 0.0,
                avg_execution_time_ms: 0.0,
                best_performing_strategy: "none".to_string(),
            },
        }
    }
}
