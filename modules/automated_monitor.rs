// ===== AUTOMATED MONITOR MODULE =====
// Sistema de monitoreo automático - Opción C implementada modularmente
// Combina safe testing + jupiter scanner + alertas + ejecución automática

use anyhow::Result;
use tracing::{info, warn, error, debug};
use tokio::time::{Duration, interval, Instant};
use chrono::{Utc, DateTime};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};

use super::safe_testing::{SafeTester, SafeTestResult, RiskLevel};
use super::jupiter_scanner::{JupiterScanner, OpportunityResult, Priority};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub scan_interval_minutes: u64,
    pub quick_scan_interval_minutes: u64,
    pub auto_execute_enabled: bool,
    pub min_confidence_score: f64,
    pub min_profit_threshold: f64,
    pub max_daily_executions: u32,
    pub alert_webhook_url: Option<String>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            scan_interval_minutes: 60,        // Full scan every hour
            quick_scan_interval_minutes: 15,  // Quick scan every 15 minutes
            auto_execute_enabled: false,      // Manual approval required by default
            min_confidence_score: 75.0,       // Minimum 75% confidence
            min_profit_threshold: 0.000045,   // 3x fees minimum (documented safe threshold)
            max_daily_executions: 5,          // Conservative daily limit
            alert_webhook_url: None,          // Optional Discord/Slack webhook
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MonitorAlert {
    pub timestamp: DateTime<Utc>,
    pub alert_type: AlertType,
    pub message: String,
    pub opportunity: Option<OpportunityResult>,
    pub action_required: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum AlertType {
    HighPriorityOpportunity,
    SafeExecutionReady,
    MarketConditionsChanged,
    SystemError,
    DailyLimitReached,
}

pub struct AutomatedMonitor {
    config: MonitorConfig,
    safe_tester: SafeTester,
    jupiter_scanner: JupiterScanner,
    execution_count: Arc<Mutex<u32>>,
    last_scan_results: Arc<Mutex<Vec<OpportunityResult>>>,
    alert_history: Arc<Mutex<Vec<MonitorAlert>>>,
}

impl AutomatedMonitor {
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            safe_tester: SafeTester::new(),
            jupiter_scanner: JupiterScanner::new(),
            execution_count: Arc::new(Mutex::new(0)),
            last_scan_results: Arc::new(Mutex::new(Vec::new())),
            alert_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create new monitor with real data validation
    pub async fn new_with_real_validation(config: MonitorConfig) -> Result<Self> {
        info!("🔧 Inicializando componentes con validación real...");
        
        // Initialize components with real data validation
        let safe_tester = SafeTester::new_with_real_validation().await?;
        let jupiter_scanner = JupiterScanner::new_with_real_validation().await?;
        
        info!("✅ Componentes inicializados exitosamente");
        
        Ok(Self {
            config,
            safe_tester,
            jupiter_scanner,
            execution_count: Arc::new(Mutex::new(0)),
            last_scan_results: Arc::new(Mutex::new(Vec::new())),
            alert_history: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Start automated monitoring system (FIXED: Non-blocking with user control)
    pub async fn start_monitoring(self: Arc<Self>) -> Result<()> {
        info!("🤖 Iniciando Sistema de Monitoreo Automático");
        info!("📊 Configuración:");
        info!("   Scan completo: cada {} minutos", self.config.scan_interval_minutes);
        info!("   Quick scan: cada {} minutos", self.config.quick_scan_interval_minutes);
        info!("   Auto-ejecución: {}", if self.config.auto_execute_enabled { "HABILITADA" } else { "MANUAL" });
        info!("   Threshold profit: {:.9} SOL", self.config.min_profit_threshold);
        info!("   Límite diario: {} ejecuciones", self.config.max_daily_executions);
        info!("");
        info!("🎯 CONTROL INTERACTIVO:");
        info!("   Press 'q' + Enter to quit monitoring");
        info!("   Press 's' + Enter to show status");
        info!("   Press Enter to force immediate scan");

        // Reset daily counter
        self.reset_daily_counter().await;

        // Start monitoring loops in background
        let full_scan_handle = {
            let monitor = Arc::clone(&self);
            tokio::spawn(async move {
                monitor.start_full_scan_loop_with_control().await
            })
        };
        
        let quick_scan_handle = {
            let monitor = Arc::clone(&self);
            tokio::spawn(async move {
                monitor.start_quick_scan_loop_with_control().await
            })
        };
        
        let health_monitor_handle = {
            let monitor = Arc::clone(&self);
            tokio::spawn(async move {
                monitor.start_health_monitor_with_control().await
            })
        };

        // Interactive control loop
        info!("🤖 Monitor iniciado. Comandos disponibles:");
        info!("   q = quit/salir");
        info!("   s = status/estado");
        info!("   Enter = scan inmediato");
        println!();
        
        loop {
            use std::io::{self, Write};
            print!("Monitor> ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let command = input.trim().to_lowercase();
            
            match command.as_str() {
                "q" | "quit" | "exit" => {
                    println!("🛑 Deteniendo monitoreo automático...");
                    // Cancel all background tasks
                    full_scan_handle.abort();
                    quick_scan_handle.abort();
                    health_monitor_handle.abort();
                    println!("✅ Monitoreo detenido exitosamente");
                    break;
                },
                "s" | "status" => {
                    self.show_monitoring_status().await;
                },
                "" => {
                    match self.execute_quick_monitoring_cycle().await {
                        Ok(_) => { /* Success message is handled inside the function */ },
                        Err(e) => {
                            println!("❌ SCAN INMEDIATO FALLÓ:");
                            eprintln!("   Error: {}", e);
                            println!("💡 Tip: Verificar conexión a internet y APIs");
                            println!();
                        },
                    }
                },
                "h" | "help" | "?" => {
                    println!();
                    println!("💡 Comandos disponibles:");
                    println!("   q = quit/salir");
                    println!("   s = status/estado");
                    println!("   h = help/ayuda");
                    println!("   Enter = scan inmediato");
                    println!();
                },
                _ => {
                    println!();
                    println!("❓ Comando desconocido: '{}'", command);
                    println!("💡 Comandos disponibles:");
                    println!("   q = quit/salir");
                    println!("   s = status/estado");
                    println!("   h = help/ayuda");
                    println!("   Enter = scan inmediato");
                    println!();
                }
            }
        }

        Ok(())
    }

    /// Full comprehensive scan loop
    async fn start_full_scan_loop(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(self.config.scan_interval_minutes * 60));
        
        loop {
            interval.tick().await;
            
            match self.execute_full_monitoring_cycle().await {
                Ok(_) => debug!("✅ Full scan cycle completed successfully"),
                Err(e) => {
                    error!("❌ Full scan cycle failed: {}", e);
                    self.send_alert(AlertType::SystemError, 
                                  format!("Full scan failed: {}", e), None, false).await;
                }
            }
        }
    }

    /// Full comprehensive scan loop with control (for background execution)
    async fn start_full_scan_loop_with_control(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(self.config.scan_interval_minutes * 60));
        let mut scan_count = 0;
        
        loop {
            interval.tick().await;
            scan_count += 1;
            
            info!("🔍 Iniciando scan completo #{}", scan_count);
            match self.execute_full_monitoring_cycle().await {
                Ok(_) => {
                    info!("✅ Scan completo #{} finalizado exitosamente", scan_count);
                },
                Err(e) => {
                    error!("❌ Scan completo #{} falló: {}", scan_count, e);
                    self.send_alert(AlertType::SystemError, 
                                  format!("Full scan #{} failed: {}", scan_count, e), None, false).await;
                }
            }
            
            // Show next scan time
            let next_scan_mins = self.config.scan_interval_minutes;
            info!("⏰ Próximo scan completo en {} minutos", next_scan_mins);
        }
    }

    /// Quick scan loop for immediate opportunities
    async fn start_quick_scan_loop(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(self.config.quick_scan_interval_minutes * 60));
        
        loop {
            interval.tick().await;
            
            match self.execute_quick_monitoring_cycle().await {
                Ok(_) => debug!("✅ Quick scan cycle completed"),
                Err(e) => {
                    warn!("⚠️ Quick scan cycle failed: {}", e);
                }
            }
        }
    }

    /// Quick scan loop with control (for background execution)
    async fn start_quick_scan_loop_with_control(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(self.config.quick_scan_interval_minutes * 60));
        let mut quick_scan_count = 0;
        
        loop {
            interval.tick().await;
            quick_scan_count += 1;
            
            info!("⚡ Quick scan #{}", quick_scan_count);
            match self.execute_quick_monitoring_cycle().await {
                Ok(_) => {
                    debug!("✅ Quick scan #{} completado", quick_scan_count);
                },
                Err(e) => {
                    warn!("⚠️ Quick scan #{} falló: {}", quick_scan_count, e);
                }
            }
        }
    }

    /// System health monitor
    async fn start_health_monitor(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes
        
        loop {
            interval.tick().await;
            self.check_system_health().await;
        }
    }

    /// System health monitor with control (for background execution)
    async fn start_health_monitor_with_control(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes
        let mut health_check_count = 0;
        
        loop {
            interval.tick().await;
            health_check_count += 1;
            
            debug!("🏥 Health check #{}", health_check_count);
            self.check_system_health().await;
        }
    }

    /// Execute full monitoring cycle (comprehensive analysis)
    async fn execute_full_monitoring_cycle(&self) -> Result<()> {
        info!("🔍 Ejecutando ciclo completo de monitoreo");
        let cycle_start = Instant::now();

        // Step 1: Safe testing validation with real verified pairs
        let verified_pairs = self.safe_tester.get_verified_mainnet_pairs().await?;
        let safe_results = self.safe_tester.execute_safe_test(verified_pairs).await?;

        // Step 2: Comprehensive Jupiter scan
        let opportunities = self.jupiter_scanner.scan_all_opportunities().await?;

        // Step 3: Cross-validate results
        let validated_opportunities = self.cross_validate_opportunities(&safe_results, &opportunities).await?;

        // Step 4: Update stored results
        {
            let mut last_results = self.last_scan_results.lock().await;
            *last_results = validated_opportunities.clone();
        }

        // Step 5: Process high-priority opportunities
        for opportunity in &validated_opportunities {
            if matches!(opportunity.execution_priority, Priority::High) {
                self.process_high_priority_opportunity(opportunity).await?;
            }
        }

        let cycle_duration = cycle_start.elapsed();
        info!("✅ Ciclo completo finalizado en {:.2}s", cycle_duration.as_secs_f64());

        Ok(())
    }

    /// Execute quick monitoring cycle (fast check)
    async fn execute_quick_monitoring_cycle(&self) -> Result<()> {
        println!("⚡ SCAN INMEDIATO - Verificación rápida de oportunidades");
        println!("═══════════════════════════════════════════════════════");
        
        // Quick scan most promising opportunities
        println!("🔍 Ejecutando quick scan con Jupiter...");
        let opportunities = match self.jupiter_scanner.quick_scan().await {
            Ok(opps) => {
                println!("✅ Jupiter scan completado: {} oportunidades encontradas", opps.len());
                opps
            },
            Err(e) => {
                println!("❌ Jupiter scan falló: {}", e);
                return Err(e);
            }
        };

        // Store results for status display
        {
            let mut last_results = self.last_scan_results.lock().await;
            last_results.clear();
            
            // Convert opportunities to OpportunityResult format for display
            for opportunity in &opportunities {
                last_results.push(OpportunityResult {
                    timestamp: chrono::Utc::now(),
                    token_pair: opportunity.token_pair.clone(),
                    input_mint: "SOL".to_string(), // Simplified for now
                    output_mint: "USDC".to_string(), // Simplified for now
                    input_amount: opportunity.input_amount,
                    estimated_profit: opportunity.estimated_profit,
                    profit_percentage: (opportunity.estimated_profit / opportunity.input_amount) * 100.0,
                    confidence_score: opportunity.confidence_score,
                    execution_priority: if opportunity.confidence_score >= 80.0 {
                        Priority::High
                    } else if opportunity.confidence_score >= 60.0 {
                        Priority::Medium
                    } else {
                        Priority::Low
                    },
                });
            }
        }

        if opportunities.is_empty() {
            println!("📊 Resultado: No hay oportunidades inmediatas disponibles");
            println!("💡 Nota: El mercado puede estar en condiciones normales");
        } else {
            println!("📊 RESULTADOS DEL SCAN INMEDIATO:");
            
            // Show all opportunities found
            for (i, opportunity) in opportunities.iter().enumerate() {
                let profit_pct = (opportunity.estimated_profit / opportunity.input_amount) * 100.0;
                let status_icon = match opportunity.execution_priority {
                    Priority::High => "🔴",
                    Priority::Medium => "🟡", 
                    Priority::Low => "🟢",
                    Priority::Monitor => "👁️",
                };
                
                println!("   {}{}. {} ({:.3} SOL): +{:.9} SOL ({:.3}%, conf: {:.1}%)",
                    status_icon,
                    i + 1,
                    opportunity.token_pair,
                    opportunity.input_amount,
                    opportunity.estimated_profit,
                    profit_pct,
                    opportunity.confidence_score
                );
            }
            
            // Check for immediate high-priority opportunities
            let high_priority_count = opportunities.iter()
                .filter(|o| matches!(o.execution_priority, Priority::High) && 
                           o.confidence_score >= self.config.min_confidence_score)
                .count();
                
            if high_priority_count > 0 {
                println!();
                println!("🚨 OPORTUNIDADES DE ALTA PRIORIDAD DETECTADAS: {}", high_priority_count);
                
                for opportunity in &opportunities {
                    if matches!(opportunity.execution_priority, Priority::High) &&
                       opportunity.confidence_score >= self.config.min_confidence_score {
                        
                        println!("🔴 INMEDIATA: {} - Profit: {:.9} SOL (conf: {:.1}%)", 
                               opportunity.token_pair, opportunity.estimated_profit, opportunity.confidence_score);
                        
                        // Validate with safe test before alert
                        let is_safe = self.validate_opportunity_safety(opportunity).await?;
                        
                        if is_safe {
                            self.send_alert(
                                AlertType::HighPriorityOpportunity,
                                format!("OPORTUNIDAD INMEDIATA: {} - Profit: {:.9} SOL", 
                                       opportunity.token_pair, opportunity.estimated_profit),
                                Some(opportunity.clone()),
                                true
                            ).await;
                            println!("   ✅ Oportunidad validada y alerta enviada");
                        } else {
                            println!("   ⚠️ Oportunidad no pasó validación de seguridad");
                        }
                    }
                }
            } else {
                println!();
                println!("📊 No hay oportunidades de alta prioridad en este momento");
                println!("💡 Tip: Las oportunidades de prioridad media/baja siguen siendo monitoreadas");
            }
        }
        
        println!("═══════════════════════════════════════════════════════");
        println!();

        Ok(())
    }

    /// Cross-validate opportunities between safe test and scanner
    async fn cross_validate_opportunities(&self, safe_results: &[SafeTestResult], 
                                        opportunities: &[OpportunityResult]) -> Result<Vec<OpportunityResult>> {
        let mut validated = Vec::new();

        for opportunity in opportunities {
            // Find corresponding safe test result
            if let Some(safe_result) = safe_results.iter().find(|sr| {
                sr.token_pair == opportunity.token_pair && 
                (sr.input_amount - opportunity.input_amount).abs() < 0.001
            }) {
                // Only include if safe test confirms profitability
                if matches!(safe_result.risk_level, RiskLevel::Safe | RiskLevel::Moderate) &&
                   opportunity.estimated_profit >= self.config.min_profit_threshold {
                    validated.push(opportunity.clone());
                }
            }
        }

        info!("🔬 Cross-validation: {}/{} oportunidades validadas", 
              validated.len(), opportunities.len());

        Ok(validated)
    }

    /// Process high-priority opportunity
    async fn process_high_priority_opportunity(&self, opportunity: &OpportunityResult) -> Result<()> {
        info!("🚨 PROCESANDO OPORTUNIDAD DE ALTA PRIORIDAD:");
        info!("   Par: {}", opportunity.token_pair);
        info!("   Profit estimado: {:.9} SOL", opportunity.estimated_profit);
        info!("   Confianza: {:.1}%", opportunity.confidence_score);

        // Check daily execution limit
        let current_count = *self.execution_count.lock().await;
        if current_count >= self.config.max_daily_executions {
            self.send_alert(AlertType::DailyLimitReached,
                          "Límite diario de ejecuciones alcanzado".to_string(),
                          Some(opportunity.clone()), false).await;
            return Ok(());
        }

        // Final safety validation
        let is_safe = self.validate_opportunity_safety(opportunity).await?;
        
        if !is_safe {
            warn!("⚠️ Oportunidad no pasó validación final de seguridad");
            return Ok(());
        }

        if self.config.auto_execute_enabled {
            info!("🎯 Auto-ejecución habilitada - preparando transacción");
            // Here would go the real execution logic
            // For now, just simulate and alert
            self.simulate_execution(opportunity).await?;
        } else {
            self.send_alert(AlertType::SafeExecutionReady,
                          format!("Oportunidad validada y lista para ejecución manual: {}", 
                                 opportunity.token_pair),
                          Some(opportunity.clone()), true).await;
        }

        Ok(())
    }

    /// Validate opportunity safety with final checks
    async fn validate_opportunity_safety(&self, opportunity: &OpportunityResult) -> Result<bool> {
        // Re-test the specific opportunity with current market conditions
        let token_pair = opportunity.token_pair.split('/').collect::<Vec<&str>>();
        if token_pair.len() != 2 {
            return Ok(false);
        }

        // Get fresh quote to confirm opportunity is still valid
        match self.safe_tester.execute_safe_test(vec![(
            opportunity.input_mint.clone(),
            opportunity.output_mint.clone(),
            opportunity.input_amount
        )]).await {
            Ok(results) => {
                if let Some(result) = results.first() {
                    Ok(matches!(result.risk_level, RiskLevel::Safe | RiskLevel::Moderate) &&
                       result.estimated_profit >= self.config.min_profit_threshold)
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false)
        }
    }

    /// Simulate execution (placeholder for real execution)
    async fn simulate_execution(&self, opportunity: &OpportunityResult) -> Result<()> {
        info!("🎬 SIMULANDO EJECUCIÓN:");
        info!("   Input: {:.6} tokens ({})", opportunity.input_amount, opportunity.token_pair.split('/').next().unwrap());
        info!("   Expected profit: {:.9} SOL", opportunity.estimated_profit);
        info!("   Execution priority: {:?}", opportunity.execution_priority);

        // Increment execution counter
        {
            let mut count = self.execution_count.lock().await;
            *count += 1;
        }

        // In real implementation, this would:
        // 1. Execute the actual arbitrage transaction
        // 2. Monitor transaction confirmation
        // 3. Calculate real profit
        // 4. Update success/failure statistics

        self.send_alert(AlertType::SafeExecutionReady,
                      format!("EJECUCIÓN SIMULADA: {} - Profit esperado: {:.9} SOL", 
                             opportunity.token_pair, opportunity.estimated_profit),
                      Some(opportunity.clone()), false).await;

        Ok(())
    }

    /// Show current monitoring status
    async fn show_monitoring_status(&self) {
        println!();
        println!("📊 MONITORING STATUS REPORT");
        println!("═══════════════════════════════════════");
        
        // Configuration status
        println!("🤖 Configuración actual:");
        println!("   Scan completo: cada {} minutos", self.config.scan_interval_minutes);
        println!("   Quick scan: cada {} minutos", self.config.quick_scan_interval_minutes);
        println!("   Auto-ejecución: {}", if self.config.auto_execute_enabled { "HABILITADA" } else { "MANUAL" });
        println!("   Min profit: {:.9} SOL", self.config.min_profit_threshold);
        println!("   Límite diario: {} ejecuciones", self.config.max_daily_executions);
        
        // Daily execution counter (FIXED: usar execution_count en lugar de daily_execution_count)
        let current_count = {
            let count = self.execution_count.lock().await;
            *count
        };
        println!("📈 Estadísticas hoy:");
        println!("   Ejecuciones realizadas: {}/{}", current_count, self.config.max_daily_executions);
        
        // Last scan results
        {
            let last_results = self.last_scan_results.lock().await;
            if last_results.is_empty() {
                println!("🔍 Último scan: Sin oportunidades detectadas");
            } else {
                println!("🔍 Último scan: {} oportunidades encontradas", last_results.len());
                
                // Show top 3 opportunities
                for (i, opportunity) in last_results.iter().take(3).enumerate() {
                    println!("   {}. {} ({:.3} SOL) -> +{:.9} SOL ({:.2}%)",
                        i + 1,
                        opportunity.token_pair,
                        opportunity.input_amount,
                        opportunity.estimated_profit,
                        opportunity.estimated_profit / opportunity.input_amount * 100.0
                    );
                }
            }
        }
        
        // Recent alerts (FIXED: usar alert_history en lugar de recent_alerts)
        {
            let alert_history = self.alert_history.lock().await;
            println!("🚨 Alertas recientes ({}):", alert_history.len());
            if alert_history.is_empty() {
                println!("   (Sin alertas)");
            } else {
                for alert in alert_history.iter().rev().take(3) {
                    let icon = match alert.alert_type {
                        AlertType::HighPriorityOpportunity => "🔴",
                        AlertType::SafeExecutionReady => "✅",
                        AlertType::MarketConditionsChanged => "📊",
                        AlertType::SystemError => "❌",
                        AlertType::DailyLimitReached => "⏰",
                    };
                    println!("   {} {} - {}", icon, alert.timestamp.format("%H:%M:%S"), alert.message);
                }
            }
        }
        
        println!("═══════════════════════════════════════");
        println!();
    }

    /// Send monitoring alert
    async fn send_alert(&self, alert_type: AlertType, message: String, 
                       opportunity: Option<OpportunityResult>, action_required: bool) {
        let alert = MonitorAlert {
            timestamp: Utc::now(),
            alert_type: alert_type.clone(),
            message: message.clone(),
            opportunity,
            action_required,
        };

        // Store in history
        {
            let mut history = self.alert_history.lock().await;
            history.push(alert.clone());
            
            // Keep only last 100 alerts
            if history.len() > 100 {
                let excess = history.len() - 100;
                history.drain(0..excess);
            }
        }

        // Log alert
        let icon = match alert_type {
            AlertType::HighPriorityOpportunity => "🔴",
            AlertType::SafeExecutionReady => "✅",
            AlertType::MarketConditionsChanged => "📊",
            AlertType::SystemError => "❌",
            AlertType::DailyLimitReached => "⏰",
        };

        if action_required {
            warn!("{} ALERTA: {}", icon, message);
        } else {
            info!("{} {}", icon, message);
        }

        // Send webhook if configured
        if let Some(webhook_url) = &self.config.alert_webhook_url {
            if let Err(e) = self.send_webhook_alert(webhook_url, &alert).await {
                warn!("Failed to send webhook alert: {}", e);
            }
        }
    }

    /// Send webhook alert (Discord/Slack integration)
    async fn send_webhook_alert(&self, webhook_url: &str, alert: &MonitorAlert) -> Result<()> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "content": format!("**SniperForge Alert** - {}\n```{}```", 
                             alert.message, 
                             alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC"))
        });

        client.post(webhook_url)
            .json(&payload)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        Ok(())
    }

    /// Check system health
    async fn check_system_health(&self) {
        // Check if monitoring is working properly
        let last_results = self.last_scan_results.lock().await;
        let alerts = self.alert_history.lock().await;
        
        debug!("💓 System health check:");
        debug!("   Last scan results: {} opportunities", last_results.len());
        debug!("   Alert history: {} alerts", alerts.len());
        debug!("   Daily executions: {}", *self.execution_count.lock().await);
    }

    /// Reset daily execution counter
    async fn reset_daily_counter(&self) {
        let mut count = self.execution_count.lock().await;
        *count = 0;
        info!("🔄 Daily execution counter reset");
    }

    /// Get monitoring status
    pub async fn get_status(&self) -> MonitoringStatus {
        let last_results = self.last_scan_results.lock().await;
        let alerts = self.alert_history.lock().await;
        let execution_count = *self.execution_count.lock().await;

        MonitoringStatus {
            is_active: true,
            last_scan_timestamp: last_results.first().map(|r| r.timestamp),
            total_opportunities: last_results.len(),
            high_priority_opportunities: last_results.iter()
                .filter(|o| matches!(o.execution_priority, Priority::High))
                .count(),
            daily_executions: execution_count,
            recent_alerts: alerts.iter().rev().take(5).cloned().collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MonitoringStatus {
    pub is_active: bool,
    pub last_scan_timestamp: Option<DateTime<Utc>>,
    pub total_opportunities: usize,
    pub high_priority_opportunities: usize,
    pub daily_executions: u32,
    pub recent_alerts: Vec<MonitorAlert>,
}

/// Public function to start monitoring with default config
pub async fn start_automated_monitoring() -> Result<()> {
    let config = MonitorConfig::default();
    let monitor = AutomatedMonitor::new(config);
    
    // Start monitoring loop with Arc wrapper
    let monitor_arc = Arc::new(monitor);
    monitor_arc.start_monitoring().await
}

/// Public function to start monitoring with custom config
pub async fn start_automated_monitoring_with_config(config: MonitorConfig) -> Result<()> {
    info!("🤖 INICIANDO AUTOMATED MONITORING - CONFIGURACIÓN PERSONALIZADA");
    info!("📊 Configuración aplicada:");
    info!("   Scan interval: {} minutos", config.scan_interval_minutes);
    info!("   Quick scan: {} minutos", config.quick_scan_interval_minutes);
    info!("   Auto-execute: {}", if config.auto_execute_enabled { "SÍ" } else { "NO" });
    info!("   Min profit: {:.9} SOL", config.min_profit_threshold);
    info!("   Daily limit: {} ejecuciones", config.max_daily_executions);
    
    // Initialize monitor with real validation
    let monitor = AutomatedMonitor::new_with_real_validation(config).await?;
    
    // Start monitoring loop with Arc wrapper
    let monitor_arc = Arc::new(monitor);
    monitor_arc.start_monitoring().await
}
