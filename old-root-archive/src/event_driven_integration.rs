// ===== EVENT-DRIVEN INTEGRATION =====
// Integración del Event-Driven Engine con el sistema principal
// Implementa Phase 4 del roadmap: Event-driven architecture + Parallel execution

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use tracing::{info, warn, debug, error};
use tokio::sync::{Mutex, RwLock, mpsc, broadcast};
use serde_json::{Value, json};
use solana_sdk::{
    pubkey::Pubkey,
    commitment_config::CommitmentConfig,
};
use solana_client::rpc_client::RpcClient;

use crate::phase4::event_driven_engine::{
    EventDrivenEngine,
    PriceEvent,
    EventType,
    EventProcessor,
    EventQueue,
    EventMetrics,
};
use crate::phase4::parallel_execution::{
    ParallelExecutionEngine,
    ExecutionTask,
    TaskPriority,
    TaskResult,
    ExecutionPool,
};
use crate::phase4::real_time_monitoring::{
    RealTimeMonitor,
    SystemMetrics,
    PerformanceAlert,
    MonitoringConfig,
};
use crate::unified_config::UnifiedPhase45Config;

/// Event procesado con contexto completo
#[derive(Debug, Clone)]
pub struct ProcessedEvent {
    pub id: String,
    pub event_type: EventType,
    pub source_data: Value,
    pub processing_time_ms: u64,
    pub opportunity_detected: bool,
    pub action_required: Option<EventAction>,
    pub priority_score: f64,
    pub confidence_level: f64,
    pub created_at: Instant,
    pub processed_at: Instant,
}

/// Acción derivada de un evento
#[derive(Debug, Clone)]
pub enum EventAction {
    ExecuteArbitrage {
        opportunity_id: String,
        estimated_profit: f64,
        urgency: ActionUrgency,
    },
    UpdatePrices {
        token_pairs: Vec<(Pubkey, Pubkey)>,
        price_delta: f64,
    },
    AdjustParameters {
        parameter: String,
        new_value: Value,
        reason: String,
    },
    SendAlert {
        alert_type: AlertType,
        message: String,
        severity: AlertSeverity,
    },
}

/// Urgencia de la acción
#[derive(Debug, Clone)]
pub enum ActionUrgency {
    Low,      // >60 segundos para actuar
    Medium,   // 30-60 segundos para actuar
    High,     // 10-30 segundos para actuar
    Critical, // <10 segundos para actuar
}

/// Tipo de alerta
#[derive(Debug, Clone)]
pub enum AlertType {
    OpportunityDetected,
    SystemPerformance,
    MarketCondition,
    ExecutionFailure,
    NetworkCongestion,
}

/// Severidad de alerta
#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Task de ejecución paralela con contexto
#[derive(Debug, Clone)]
pub struct EnhancedExecutionTask {
    pub id: String,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub base_task: ExecutionTask,
    pub estimated_duration_ms: u64,
    pub resource_requirements: ResourceRequirements,
    pub dependencies: Vec<String>,
    pub retry_config: RetryConfig,
    pub created_at: Instant,
    pub deadline: Option<Instant>,
}

/// Tipo de task
#[derive(Debug, Clone)]
pub enum TaskType {
    ArbitrageExecution,
    PriceDataCollection,
    ParameterOptimization,
    SystemMaintenance,
    AlertProcessing,
}

/// Requerimientos de recursos
#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub cpu_usage_pct: u8,
    pub memory_mb: u32,
    pub network_bandwidth_kbps: u32,
    pub max_concurrent_connections: u8,
}

/// Configuración de retry
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u8,
    pub initial_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub max_delay_ms: u64,
}

/// Resultado de ejecución mejorado
#[derive(Debug, Clone)]
pub struct EnhancedExecutionResult {
    pub task_id: String,
    pub success: bool,
    pub base_result: TaskResult,
    pub actual_duration: Duration,
    pub resource_usage: ActualResourceUsage,
    pub retry_attempts: u8,
    pub performance_metrics: TaskPerformanceMetrics,
    pub error_details: Option<String>,
    pub completed_at: Instant,
}

/// Uso real de recursos
#[derive(Debug, Clone)]
pub struct ActualResourceUsage {
    pub peak_cpu_pct: u8,
    pub peak_memory_mb: u32,
    pub total_network_kb: u32,
    pub concurrent_connections_used: u8,
}

/// Métricas de performance de task
#[derive(Debug, Clone)]
pub struct TaskPerformanceMetrics {
    pub throughput_ops_per_sec: f64,
    pub latency_percentiles: LatencyPercentiles,
    pub error_rate_pct: f64,
    pub efficiency_score: f64, // 0.0-1.0
}

/// Percentiles de latencia
#[derive(Debug, Clone)]
pub struct LatencyPercentiles {
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
}

/// Integrador principal para Event-Driven + Parallel Execution
pub struct EventDrivenIntegrator {
    event_engine: Arc<EventDrivenEngine>,
    parallel_engine: Arc<ParallelExecutionEngine>,
    real_time_monitor: Arc<RealTimeMonitor>,
    config: UnifiedPhase45Config,
    rpc_client: Arc<RpcClient>,
    
    // Event processing
    event_queue: Arc<Mutex<VecDeque<ProcessedEvent>>>,
    event_processor_tx: mpsc::UnboundedSender<PriceEvent>,
    event_processor_rx: Arc<Mutex<mpsc::UnboundedReceiver<PriceEvent>>>,
    
    // Parallel execution
    task_queue: Arc<Mutex<VecDeque<EnhancedExecutionTask>>>,
    task_results: Arc<RwLock<HashMap<String, EnhancedExecutionResult>>>,
    active_tasks: Arc<RwLock<HashMap<String, Instant>>>,
    
    // Broadcasting
    alert_tx: broadcast::Sender<ProcessedEvent>,
    
    // Performance tracking
    system_metrics: Arc<RwLock<SystemMetrics>>,
    performance_history: Arc<RwLock<VecDeque<PerformanceSnapshot>>>,
}

/// Snapshot de performance del sistema
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub timestamp: Instant,
    pub events_processed_per_sec: f64,
    pub tasks_completed_per_sec: f64,
    pub average_event_latency_ms: u64,
    pub average_task_latency_ms: u64,
    pub system_cpu_usage_pct: u8,
    pub system_memory_usage_mb: u32,
    pub active_tasks_count: usize,
    pub queue_length: usize,
}

impl EventDrivenIntegrator {
    /// Crear nuevo integrador Event-Driven
    pub async fn new(config: UnifiedPhase45Config, rpc_client: Arc<RpcClient>) -> Result<Self> {
        info!("⚡ Inicializando Event-Driven Integrator");
        
        // Crear configuraciones específicas
        let event_config = Self::create_event_config(&config)?;
        let parallel_config = Self::create_parallel_config(&config)?;
        let monitoring_config = Self::create_monitoring_config(&config)?;
        
        // Inicializar engines
        let event_engine = Arc::new(EventDrivenEngine::new(event_config).await?);
        let parallel_engine = Arc::new(ParallelExecutionEngine::new(parallel_config).await?);
        let real_time_monitor = Arc::new(RealTimeMonitor::new(monitoring_config).await?);
        
        // Crear channels para comunicación
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (alert_tx, _alert_rx) = broadcast::channel(1000);
        
        info!("✅ Event-Driven Engine inicializado");
        info!("   ⚡ Event processing: {}", if config.event_driven_enabled { "✅" } else { "❌" });
        info!("   🔄 Parallel execution: {}", if config.parallel_execution_enabled { "✅" } else { "❌" });
        info!("   📊 Real-time monitoring: {}", if config.real_time_monitoring_enabled { "✅" } else { "❌" });
        info!("   🎯 Max concurrent tasks: {}", config.max_concurrent_arbitrage_tasks);
        info!("   ⏱️ Event processing target: <{}ms", config.target_event_processing_time_ms);
        
        let integrator = Self {
            event_engine,
            parallel_engine,
            real_time_monitor,
            config,
            rpc_client,
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
            event_processor_tx: event_tx,
            event_processor_rx: Arc::new(Mutex::new(event_rx)),
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            task_results: Arc::new(RwLock::new(HashMap::new())),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            alert_tx,
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            performance_history: Arc::new(RwLock::new(VecDeque::new())),
        };
        
        // Iniciar background tasks
        integrator.start_background_processors().await?;
        
        Ok(integrator)
    }
    
    /// Crear configuración para Event Engine
    fn create_event_config(config: &UnifiedPhase45Config) -> Result<crate::phase4::event_driven_engine::EventDrivenConfig> {
        Ok(crate::phase4::event_driven_engine::EventDrivenConfig {
            max_events_per_second: 1000,
            event_buffer_size: 10000,
            price_change_threshold_bps: 10, // 0.1% price change threshold
            processing_timeout_ms: config.target_event_processing_time_ms,
            enable_event_replay: true,
            max_replay_history: 1000,
        })
    }
    
    /// Crear configuración para Parallel Engine
    fn create_parallel_config(config: &UnifiedPhase45Config) -> Result<crate::phase4::parallel_execution::ParallelConfig> {
        Ok(crate::phase4::parallel_execution::ParallelConfig {
            max_concurrent_tasks: config.max_concurrent_arbitrage_tasks,
            max_queue_size: 1000,
            task_timeout_seconds: 30,
            enable_task_prioritization: true,
            resource_management_enabled: true,
            max_cpu_usage_pct: 80,
            max_memory_usage_mb: 1024,
        })
    }
    
    /// Crear configuración para Monitoring
    fn create_monitoring_config(config: &UnifiedPhase45Config) -> Result<MonitoringConfig> {
        Ok(MonitoringConfig {
            metrics_collection_interval_ms: 1000,
            performance_alert_thresholds: json!({
                "max_event_latency_ms": config.target_event_processing_time_ms * 2,
                "max_task_latency_ms": 10000,
                "max_cpu_usage_pct": 85,
                "max_memory_usage_mb": 1024,
                "min_success_rate_pct": 70
            }),
            enable_real_time_alerts: config.real_time_monitoring_enabled,
            alert_cooldown_seconds: 60,
        })
    }
    
    /// Iniciar procesadores en background
    async fn start_background_processors(&self) -> Result<()> {
        info!("🚀 Iniciando procesadores en background");
        
        if self.config.event_driven_enabled {
            self.start_event_processor().await?;
        }
        
        if self.config.parallel_execution_enabled {
            self.start_task_executor().await?;
        }
        
        if self.config.real_time_monitoring_enabled {
            self.start_performance_monitor().await?;
        }
        
        Ok(())
    }
    
    /// Iniciar procesador de eventos
    async fn start_event_processor(&self) -> Result<()> {
        info!("⚡ Iniciando Event Processor");
        
        let event_rx = self.event_processor_rx.clone();
        let event_queue = self.event_queue.clone();
        let alert_tx = self.alert_tx.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut rx = event_rx.lock().await;
            let mut processed_count = 0u64;
            let mut last_stats_time = Instant::now();
            
            while let Some(event) = rx.recv().await {
                let start_time = Instant::now();
                
                // Procesar evento
                match Self::process_single_event(event, &config).await {
                    Ok(processed_event) => {
                        // Agregar a queue
                        {
                            let mut queue = event_queue.lock().await;
                            queue.push_back(processed_event.clone());
                            
                            // Mantener queue limitado
                            if queue.len() > 1000 {
                                queue.pop_front();
                            }
                        }
                        
                        // Enviar alerta si es necesario
                        if processed_event.opportunity_detected {
                            let _ = alert_tx.send(processed_event);
                        }
                        
                        processed_count += 1;
                    },
                    Err(e) => {
                        error!("Error procesando evento: {}", e);
                    }
                }
                
                // Stats cada 10 segundos
                if last_stats_time.elapsed() > Duration::from_secs(10) {
                    let events_per_sec = processed_count as f64 / 10.0;
                    debug!("📊 Event processing: {:.1} events/sec", events_per_sec);
                    processed_count = 0;
                    last_stats_time = Instant::now();
                }
            }
        });
        
        Ok(())
    }
    
    /// Procesar un evento individual
    async fn process_single_event(
        event: PriceEvent, 
        config: &UnifiedPhase45Config
    ) -> Result<ProcessedEvent> {
        let start_time = Instant::now();
        
        // Simular procesamiento de evento (en implementación real: análisis complejo)
        let processing_delay = match event.event_type {
            EventType::PriceUpdate => 5 + (rand::random::<u64>() % 10), // 5-15ms
            EventType::LiquidityChange => 10 + (rand::random::<u64>() % 20), // 10-30ms
            EventType::TradeExecution => 3 + (rand::random::<u64>() % 5), // 3-8ms
            _ => 8 + (rand::random::<u64>() % 10), // 8-18ms
        };
        
        tokio::time::sleep(Duration::from_millis(processing_delay)).await;
        
        // Determinar si se detectó oportunidad
        let opportunity_detected = match event.event_type {
            EventType::PriceUpdate => {
                // Simular detección basada en cambio de precio
                event.price_change.abs() > 0.005 && rand::random::<f64>() > 0.7
            },
            EventType::LiquidityChange => {
                // Cambios de liquidez pueden crear oportunidades
                rand::random::<f64>() > 0.85
            },
            _ => rand::random::<f64>() > 0.9, // Otros eventos rara vez crean oportunidades
        };
        
        // Calcular scores
        let priority_score = if opportunity_detected {
            0.7 + (rand::random::<f64>() * 0.3) // 0.7-1.0 para oportunidades
        } else {
            0.1 + (rand::random::<f64>() * 0.4) // 0.1-0.5 para eventos normales
        };
        
        let confidence_level = if opportunity_detected {
            0.6 + (rand::random::<f64>() * 0.4) // 0.6-1.0 para oportunidades
        } else {
            0.3 + (rand::random::<f64>() * 0.4) // 0.3-0.7 para eventos normales
        };
        
        // Determinar acción requerida
        let action_required = if opportunity_detected {
            Some(EventAction::ExecuteArbitrage {
                opportunity_id: format!("EVENT_OPP_{}", start_time.elapsed().as_millis()),
                estimated_profit: 0.001 + (rand::random::<f64>() * 0.019), // 0.001-0.02 SOL
                urgency: if priority_score > 0.9 {
                    ActionUrgency::Critical
                } else if priority_score > 0.8 {
                    ActionUrgency::High
                } else {
                    ActionUrgency::Medium
                },
            })
        } else {
            None
        };
        
        let processed = ProcessedEvent {
            id: format!("EVENT_{}", start_time.elapsed().as_millis()),
            event_type: event.event_type,
            source_data: json!({
                "token_a": event.token_a.to_string(),
                "token_b": event.token_b.to_string(),
                "price_change": event.price_change,
                "volume": event.volume_24h,
                "timestamp": event.timestamp
            }),
            processing_time_ms: processing_delay,
            opportunity_detected,
            action_required,
            priority_score,
            confidence_level,
            created_at: start_time,
            processed_at: Instant::now(),
        };
        
        if opportunity_detected {
            debug!("🎯 Oportunidad detectada en evento: {} (priority: {:.2}, confidence: {:.2})", 
                   processed.id, priority_score, confidence_level);
        }
        
        Ok(processed)
    }
    
    /// Iniciar ejecutor de tasks paralelas
    async fn start_task_executor(&self) -> Result<()> {
        info!("🔄 Iniciando Parallel Task Executor");
        
        let task_queue = self.task_queue.clone();
        let task_results = self.task_results.clone();
        let active_tasks = self.active_tasks.clone();
        let config = self.config.clone();
        
        // Spawn múltiples workers para paralelismo
        let worker_count = config.max_concurrent_arbitrage_tasks.min(8); // Máximo 8 workers
        
        for worker_id in 0..worker_count {
            let queue = task_queue.clone();
            let results = task_results.clone();
            let active = active_tasks.clone();
            let cfg = config.clone();
            
            tokio::spawn(async move {
                info!("👷 Worker {} iniciado", worker_id);
                
                loop {
                    // Obtener próxima task
                    let task = {
                        let mut queue = queue.lock().await;
                        queue.pop_front()
                    };
                    
                    if let Some(task) = task {
                        // Marcar como activa
                        {
                            let mut active = active.write().await;
                            active.insert(task.id.clone(), Instant::now());
                        }
                        
                        // Ejecutar task
                        let result = Self::execute_single_task(task, worker_id, &cfg).await;
                        
                        // Guardar resultado
                        {
                            let mut results = results.write().await;
                            results.insert(result.task_id.clone(), result.clone());
                            
                            // Limpiar resultados antiguos (mantener últimos 1000)
                            if results.len() > 1000 {
                                // Remover el más antiguo
                                if let Some((oldest_id, _)) = results.iter()
                                    .min_by_key(|(_, r)| r.completed_at) {
                                    let oldest_id = oldest_id.clone();
                                    results.remove(&oldest_id);
                                }
                            }
                        }
                        
                        // Remover de activas
                        {
                            let mut active = active.write().await;
                            active.remove(&result.task_id);
                        }
                        
                        if result.success {
                            debug!("✅ Worker {} completó task: {} en {:?}", 
                                   worker_id, result.task_id, result.actual_duration);
                        } else {
                            warn!("❌ Worker {} falló task: {} - {}", 
                                  worker_id, result.task_id, 
                                  result.error_details.as_deref().unwrap_or("Unknown error"));
                        }
                    } else {
                        // No hay tasks, esperar un poco
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            });
        }
        
        info!("✅ {} workers paralelos iniciados", worker_count);
        Ok(())
    }
    
    /// Ejecutar una task individual
    async fn execute_single_task(
        task: EnhancedExecutionTask, 
        worker_id: usize,
        config: &UnifiedPhase45Config
    ) -> EnhancedExecutionResult {
        let start_time = Instant::now();
        
        debug!("🔄 Worker {} ejecutando task: {} (tipo: {:?}, prioridad: {:?})", 
               worker_id, task.id, task.task_type, task.priority);
        
        // Simular ejecución de task
        let execution_result = Self::simulate_task_execution(&task, config).await;
        
        let actual_duration = start_time.elapsed();
        
        // Calcular métricas de performance
        let performance_metrics = TaskPerformanceMetrics {
            throughput_ops_per_sec: if execution_result.success { 
                1000.0 / actual_duration.as_millis() as f64 
            } else { 
                0.0 
            },
            latency_percentiles: LatencyPercentiles {
                p50_ms: actual_duration.as_millis() as u64,
                p95_ms: (actual_duration.as_millis() as f64 * 1.2) as u64,
                p99_ms: (actual_duration.as_millis() as f64 * 1.5) as u64,
            },
            error_rate_pct: if execution_result.success { 0.0 } else { 100.0 },
            efficiency_score: if execution_result.success {
                (task.estimated_duration_ms as f64 / actual_duration.as_millis() as f64).min(1.0)
            } else {
                0.0
            },
        };
        
        EnhancedExecutionResult {
            task_id: task.id,
            success: execution_result.success,
            base_result: execution_result,
            actual_duration,
            resource_usage: ActualResourceUsage {
                peak_cpu_pct: task.resource_requirements.cpu_usage_pct,
                peak_memory_mb: task.resource_requirements.memory_mb,
                total_network_kb: task.resource_requirements.network_bandwidth_kbps / 8, // Convert to KB
                concurrent_connections_used: task.resource_requirements.max_concurrent_connections,
            },
            retry_attempts: 0, // Por simplicidad, no implementamos retry aquí
            performance_metrics,
            error_details: if !execution_result.success {
                Some("Simulated task execution failure".to_string())
            } else {
                None
            },
            completed_at: Instant::now(),
        }
    }
    
    /// Simular ejecución de task
    async fn simulate_task_execution(task: &EnhancedExecutionTask, _config: &UnifiedPhase45Config) -> TaskResult {
        // Simular tiempo de ejecución
        let execution_time = task.estimated_duration_ms + 
                            (rand::random::<u64>() % (task.estimated_duration_ms / 2)); // ±25% variación
        
        tokio::time::sleep(Duration::from_millis(execution_time)).await;
        
        // Simular success rate basado en tipo de task
        let success_rate = match task.task_type {
            TaskType::ArbitrageExecution => 0.85, // 85% success para arbitraje
            TaskType::PriceDataCollection => 0.95, // 95% success para datos
            TaskType::ParameterOptimization => 0.90, // 90% success para optimización
            TaskType::SystemMaintenance => 0.98, // 98% success para mantenimiento
            TaskType::AlertProcessing => 0.99, // 99% success para alertas
        };
        
        let success = rand::random::<f64>() < success_rate;
        
        TaskResult {
            task_id: task.id.clone(),
            success,
            output: if success {
                Some(json!({
                    "task_type": format!("{:?}", task.task_type),
                    "execution_time_ms": execution_time,
                    "result": "success"
                }))
            } else {
                None
            },
            error: if !success {
                Some(format!("Task execution failed: {:?}", task.task_type))
            } else {
                None
            },
            duration: Duration::from_millis(execution_time),
        }
    }
    
    /// Iniciar monitor de performance
    async fn start_performance_monitor(&self) -> Result<()> {
        info!("📊 Iniciando Performance Monitor");
        
        let system_metrics = self.system_metrics.clone();
        let performance_history = self.performance_history.clone();
        let active_tasks = self.active_tasks.clone();
        let task_queue = self.task_queue.clone();
        let event_queue = self.event_queue.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5)); // Cada 5 segundos
            
            loop {
                interval.tick().await;
                
                // Recolectar métricas actuales
                let snapshot = Self::collect_performance_snapshot(
                    &active_tasks,
                    &task_queue,
                    &event_queue
                ).await;
                
                // Actualizar métricas del sistema
                {
                    let mut metrics = system_metrics.write().await;
                    *metrics = Self::update_system_metrics(&snapshot);
                }
                
                // Agregar al historial
                {
                    let mut history = performance_history.write().await;
                    history.push_back(snapshot);
                    
                    // Mantener solo últimas 720 muestras (1 hora con muestras cada 5 seg)
                    if history.len() > 720 {
                        history.pop_front();
                    }
                }
                
                // Log stats cada minuto
                if rand::random::<u8>() % 12 == 0 { // ~1/12 probabilidad = cada ~1 minuto
                    debug!("📊 Performance: {:.1} events/sec, {:.1} tasks/sec, {} active tasks", 
                           snapshot.events_processed_per_sec,
                           snapshot.tasks_completed_per_sec,
                           snapshot.active_tasks_count);
                }
            }
        });
        
        Ok(())
    }
    
    /// Recolectar snapshot de performance
    async fn collect_performance_snapshot(
        active_tasks: &Arc<RwLock<HashMap<String, Instant>>>,
        task_queue: &Arc<Mutex<VecDeque<EnhancedExecutionTask>>>,
        event_queue: &Arc<Mutex<VecDeque<ProcessedEvent>>>,
    ) -> PerformanceSnapshot {
        
        let active_count = {
            let tasks = active_tasks.read().await;
            tasks.len()
        };
        
        let queue_length = {
            let queue = task_queue.lock().await;
            queue.len()
        };
        
        // Simular métricas (en implementación real vendrían del sistema)
        let events_per_sec = 5.0 + (rand::random::<f64>() * 15.0); // 5-20 events/sec
        let tasks_per_sec = 1.0 + (rand::random::<f64>() * 4.0); // 1-5 tasks/sec
        let event_latency = 10 + (rand::random::<u64>() % 20); // 10-30ms
        let task_latency = 1000 + (rand::random::<u64>() % 2000); // 1-3 segundos
        let cpu_usage = 20 + (rand::random::<u8>() % 40); // 20-60% CPU
        let memory_usage = 100 + (rand::random::<u32>() % 300); // 100-400 MB
        
        PerformanceSnapshot {
            timestamp: Instant::now(),
            events_processed_per_sec: events_per_sec,
            tasks_completed_per_sec: tasks_per_sec,
            average_event_latency_ms: event_latency,
            average_task_latency_ms: task_latency,
            system_cpu_usage_pct: cpu_usage,
            system_memory_usage_mb: memory_usage,
            active_tasks_count: active_count,
            queue_length,
        }
    }
    
    /// Actualizar métricas del sistema
    fn update_system_metrics(snapshot: &PerformanceSnapshot) -> SystemMetrics {
        SystemMetrics {
            cpu_usage_percent: snapshot.system_cpu_usage_pct as f64,
            memory_usage_mb: snapshot.system_memory_usage_mb as f64,
            network_throughput_mbps: 10.0 + (rand::random::<f64>() * 20.0), // Simulated
            active_connections: snapshot.active_tasks_count as u32,
            uptime_seconds: 3600, // Simulated 1 hour uptime
            last_updated: std::time::SystemTime::now(),
        }
    }
    
    /// Procesar evento y generar task si es necesario
    pub async fn process_event(&self, event: PriceEvent) -> Result<Option<String>> {
        if !self.config.event_driven_enabled {
            debug!("Event-driven processing deshabilitado");
            return Ok(None);
        }
        
        // Enviar evento al procesador
        self.event_processor_tx.send(event)?;
        
        // Intentar obtener el evento procesado (con timeout)
        let processed_event = {
            let start_time = Instant::now();
            loop {
                {
                    let mut queue = self.event_queue.lock().await;
                    if let Some(event) = queue.pop_front() {
                        break Some(event);
                    }
                }
                
                if start_time.elapsed() > Duration::from_millis(self.config.target_event_processing_time_ms) {
                    break None;
                }
                
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        };
        
        if let Some(processed) = processed_event {
            // Si hay acción requerida, crear task
            if let Some(action) = processed.action_required {
                let task_id = self.create_task_from_action(action, &processed).await?;
                return Ok(Some(task_id));
            }
        }
        
        Ok(None)
    }
    
    /// Crear task de ejecución desde acción
    async fn create_task_from_action(
        &self, 
        action: EventAction, 
        event: &ProcessedEvent
    ) -> Result<String> {
        
        match action {
            EventAction::ExecuteArbitrage { opportunity_id, estimated_profit, urgency } => {
                let priority = match urgency {
                    ActionUrgency::Critical => TaskPriority::Critical,
                    ActionUrgency::High => TaskPriority::High,
                    ActionUrgency::Medium => TaskPriority::Medium,
                    ActionUrgency::Low => TaskPriority::Low,
                };
                
                let estimated_duration = match urgency {
                    ActionUrgency::Critical => 500,  // 0.5 segundos
                    ActionUrgency::High => 1000,     // 1 segundo
                    ActionUrgency::Medium => 2000,   // 2 segundos
                    ActionUrgency::Low => 5000,      // 5 segundos
                };
                
                let task = EnhancedExecutionTask {
                    id: format!("ARBITRAGE_TASK_{}", opportunity_id),
                    task_type: TaskType::ArbitrageExecution,
                    priority,
                    base_task: ExecutionTask {
                        id: opportunity_id.clone(),
                        function: format!("execute_arbitrage_{}", opportunity_id),
                        parameters: json!({
                            "opportunity_id": opportunity_id,
                            "estimated_profit": estimated_profit,
                            "urgency": format!("{:?}", urgency),
                            "event_id": event.id
                        }),
                        priority,
                    },
                    estimated_duration_ms: estimated_duration,
                    resource_requirements: ResourceRequirements {
                        cpu_usage_pct: 15,
                        memory_mb: 50,
                        network_bandwidth_kbps: 100,
                        max_concurrent_connections: 3,
                    },
                    dependencies: Vec::new(),
                    retry_config: RetryConfig {
                        max_retries: 2,
                        initial_delay_ms: 100,
                        backoff_multiplier: 2.0,
                        max_delay_ms: 1000,
                    },
                    created_at: Instant::now(),
                    deadline: Some(Instant::now() + Duration::from_secs(30)),
                };
                
                // Agregar a queue
                {
                    let mut queue = self.task_queue.lock().await;
                    queue.push_back(task.clone());
                }
                
                info!("📋 Task de arbitraje creada: {} (urgencia: {:?}, profit: {:.6} SOL)", 
                      task.id, urgency, estimated_profit);
                
                Ok(task.id)
            },
            _ => {
                // Otros tipos de acciones se implementarían aquí
                Err(anyhow!("Action type not implemented yet"))
            }
        }
    }
    
    /// Obtener estadísticas del sistema
    pub async fn get_system_stats(&self) -> EventDrivenStats {
        let performance_history = self.performance_history.read().await;
        let task_results = self.task_results.read().await;
        let active_tasks = self.active_tasks.read().await;
        
        // Calcular métricas agregadas
        let total_tasks = task_results.len();
        let successful_tasks = task_results.values().filter(|r| r.success).count();
        let success_rate = if total_tasks > 0 {
            (successful_tasks as f64 / total_tasks as f64) * 100.0
        } else {
            0.0
        };
        
        let average_event_latency = if !performance_history.is_empty() {
            performance_history.iter()
                .map(|s| s.average_event_latency_ms)
                .sum::<u64>() / performance_history.len() as u64
        } else {
            0
        };
        
        let average_task_latency = if !performance_history.is_empty() {
            performance_history.iter()
                .map(|s| s.average_task_latency_ms)
                .sum::<u64>() / performance_history.len() as u64
        } else {
            0
        };
        
        let current_throughput = performance_history.back()
            .map(|s| s.events_processed_per_sec + s.tasks_completed_per_sec)
            .unwrap_or(0.0);
        
        EventDrivenStats {
            total_events_processed: total_tasks as u64 * 2, // Aproximación
            total_tasks_executed: total_tasks,
            successful_tasks,
            success_rate_pct: success_rate,
            average_event_latency_ms: average_event_latency,
            average_task_latency_ms: average_task_latency,
            current_throughput_ops_sec: current_throughput,
            active_tasks_count: active_tasks.len(),
            queue_length: 0, // Se obtendría del queue actual
            system_uptime_seconds: 3600, // Simulado
        }
    }
    
    /// Forzar ejecución de task específica
    pub async fn execute_task_immediately(&self, task_id: &str) -> Result<EnhancedExecutionResult> {
        // Buscar task en queue
        let task = {
            let mut queue = self.task_queue.lock().await;
            let mut found_task = None;
            let mut index = 0;
            
            for (i, task) in queue.iter().enumerate() {
                if task.id == task_id {
                    found_task = Some(queue.remove(i).unwrap());
                    break;
                }
            }
            
            found_task
        };
        
        if let Some(task) = task {
            info!("⚡ Ejecutando task inmediatamente: {}", task_id);
            
            // Ejecutar directamente
            let result = Self::execute_single_task(task, 999, &self.config).await; // Worker ID 999 para immediate
            
            // Guardar resultado
            {
                let mut results = self.task_results.write().await;
                results.insert(result.task_id.clone(), result.clone());
            }
            
            Ok(result)
        } else {
            Err(anyhow!("Task not found in queue: {}", task_id))
        }
    }
}

/// Estadísticas del sistema Event-Driven
#[derive(Debug, Clone)]
pub struct EventDrivenStats {
    pub total_events_processed: u64,
    pub total_tasks_executed: usize,
    pub successful_tasks: usize,
    pub success_rate_pct: f64,
    pub average_event_latency_ms: u64,
    pub average_task_latency_ms: u64,
    pub current_throughput_ops_sec: f64,
    pub active_tasks_count: usize,
    pub queue_length: usize,
    pub system_uptime_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_event_processing() {
        let config = UnifiedPhase45Config::default();
        let rpc_client = Arc::new(RpcClient::new("test"));
        
        // Test básico de inicialización
        let integrator = EventDrivenIntegrator::new(config, rpc_client).await;
        assert!(integrator.is_ok());
    }
}
