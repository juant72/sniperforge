//! Desired State Reconciler
//! 
//! 🎯 DECLARATIVE SYSTEM: Implementa reconciliación automática de estado deseado
//! 
//! Este módulo es el corazón del sistema declarativo de SniperForge.
//! Su responsabilidad es:
//! 1. 📖 Leer el estado deseado desde YAML
//! 2. 🔍 Comparar con el estado actual del sistema 
//! 3. 🔄 Ejecutar acciones para converger al estado deseado
//! 4. 🔁 Reconciliar continuamente para mantener el estado

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use uuid::Uuid;
use tracing::{info, warn, error, debug};
use anyhow::Result;

use crate::api::{
    DesiredStateConfig, DesiredBotState, DesiredBotStatus, DesiredBotConfig,
    ReconciliationConfig, BotType, BotStatus, BotConfig, YamlConfigManager
};
use crate::control::{BotController, BotSummary};

/// 🎯 Sistema de Reconciliación de Estado Deseado
/// 
/// Este es el controlador principal que implementa el paradigma declarativo:
/// - Lee configuración YAML declarativa
/// - Compara estado actual vs estado deseado  
/// - Ejecuta acciones de reconciliación automática
/// - Mantiene convergencia continua
pub struct DesiredStateReconciler {
    /// Controlador de bots para operaciones del sistema
    bot_controller: Arc<BotController>,
    
    /// Manager de configuración YAML
    yaml_manager: Arc<YamlConfigManager>,
    
    /// Estado deseado actual cargado desde YAML
    desired_state: Arc<RwLock<DesiredStateConfig>>,
    
    /// Canal para eventos de reconciliación
    reconciliation_tx: mpsc::Sender<ReconciliationEvent>,
    reconciliation_rx: Arc<RwLock<Option<mpsc::Receiver<ReconciliationEvent>>>>,
    
    /// Estadísticas de reconciliación
    stats: Arc<RwLock<ReconciliationStats>>,
    
    /// Flag para parar el reconciliador
    should_stop: Arc<RwLock<bool>>,
}

/// Eventos de reconciliación
#[derive(Debug, Clone)]
pub enum ReconciliationEvent {
    /// Solicitar reconciliación manual
    TriggerReconciliation,
    
    /// Estado deseado ha cambiado
    DesiredStateChanged,
    
    /// Bot ha cambiado de estado
    BotStateChanged { bot_id: String, new_status: BotStatus },
    
    /// Error de reconciliación
    ReconciliationError { error: String },
    
    /// Reconciliación completada exitosamente
    ReconciliationCompleted { duration_ms: u64, actions_taken: u32 },
}

/// Estadísticas de reconciliación
#[derive(Debug, Clone, Default)]
pub struct ReconciliationStats {
    /// Número total de reconciliaciones ejecutadas
    pub total_reconciliations: u64,
    
    /// Reconciliaciones exitosas
    pub successful_reconciliations: u64,
    
    /// Reconciliaciones fallidas
    pub failed_reconciliations: u64,
    
    /// Tiempo promedio de reconciliación (ms)
    pub avg_reconciliation_time_ms: f64,
    
    /// Última reconciliación exitosa
    pub last_successful_reconciliation: Option<Instant>,
    
    /// Acciones totales ejecutadas
    pub total_actions_executed: u64,
    
    /// Drift detectado en la última reconciliación
    pub last_drift_detected: bool,
}

/// Acción de reconciliación a ejecutar
#[derive(Debug, Clone)]
pub enum ReconciliationAction {
    /// Crear un nuevo bot
    CreateBot {
        id: String,
        bot_type: BotType,
        config: BotConfig,
    },
    
    /// Iniciar un bot existente
    StartBot {
        bot_id: Uuid,
        config: BotConfig,
    },
    
    /// Detener un bot
    StopBot {
        bot_id: Uuid,
    },
    
    /// Pausar un bot
    PauseBot {
        bot_id: Uuid,
    },
    
    /// Actualizar configuración de bot
    UpdateBotConfig {
        bot_id: Uuid,
        new_config: BotConfig,
    },
    
    /// Eliminar un bot
    DeleteBot {
        bot_id: Uuid,
    },
}

/// Resultado de reconciliación
#[derive(Debug)]
pub struct ReconciliationResult {
    /// Acciones ejecutadas exitosamente
    pub successful_actions: Vec<ReconciliationAction>,
    
    /// Acciones que fallaron
    pub failed_actions: Vec<(ReconciliationAction, String)>,
    
    /// Drift detectado entre estado actual y deseado
    pub drift_detected: bool,
    
    /// Tiempo total de reconciliación
    pub duration: Duration,
    
    /// Estado del sistema después de reconciliación
    pub post_reconciliation_summary: String,
}

impl DesiredStateReconciler {
    /// Crear nuevo reconciliador de estado deseado
    pub async fn new(
        bot_controller: Arc<BotController>,
        yaml_manager: Arc<YamlConfigManager>,
    ) -> Result<Self> {
        let (reconciliation_tx, reconciliation_rx) = mpsc::channel(100);
        
        // Cargar estado deseado inicial desde YAML
        let desired_state = match yaml_manager.load_desired_state().await {
            Ok(state) => Arc::new(RwLock::new(state)),
            Err(e) => {
                warn!("⚠️ No se pudo cargar estado deseado inicial: {}. Usando configuración vacía.", e);
                Arc::new(RwLock::new(DesiredStateConfig::new()))
            }
        };
        
        Ok(Self {
            bot_controller,
            yaml_manager,
            desired_state,
            reconciliation_tx,
            reconciliation_rx: Arc::new(RwLock::new(Some(reconciliation_rx))),
            stats: Arc::new(RwLock::new(ReconciliationStats::default())),
            should_stop: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Iniciar el bucle de reconciliación automática
    pub async fn start_reconciliation_loop(&self) -> Result<()> {
        let desired_state = self.desired_state.clone();
        let bot_controller = self.bot_controller.clone();
        let yaml_manager = self.yaml_manager.clone();
        let stats = self.stats.clone();
        let should_stop = self.should_stop.clone();
        let _reconciliation_tx = self.reconciliation_tx.clone();
        
        // Tomar el receiver del canal
        let mut reconciliation_rx = {
            let mut rx_guard = self.reconciliation_rx.write().await;
            rx_guard.take().ok_or_else(|| {
                anyhow::anyhow!("Reconciliation loop already started")
            })?
        };
        
        info!("🎯 Iniciando bucle de reconciliación de estado deseado...");
        
        tokio::spawn(async move {
            // Leer configuración de reconciliación
            let reconciliation_config = {
                let state = desired_state.read().await;
                state.reconciliation.clone()
            };
            
            if !reconciliation_config.enabled {
                info!("⏸️ Reconciliación automática deshabilitada");
                return;
            }
            
            let mut reconciliation_interval = interval(Duration::from_secs(reconciliation_config.interval_seconds));
            
            loop {
                tokio::select! {
                    // Reconciliación periódica automática
                    _ = reconciliation_interval.tick() => {
                        if *should_stop.read().await {
                            info!("🛑 Deteniendo bucle de reconciliación");
                            break;
                        }
                        
                        debug!("⏰ Ejecutando reconciliación automática...");
                        let _ = Self::execute_reconciliation_cycle(
                            &desired_state,
                            &bot_controller,
                            &yaml_manager,
                            &stats,
                            &reconciliation_config,
                        ).await;
                    }
                    
                    // Eventos de reconciliación manual
                    Some(event) = reconciliation_rx.recv() => {
                        match event {
                            ReconciliationEvent::TriggerReconciliation => {
                                info!("🔄 Ejecutando reconciliación manual...");
                                let _ = Self::execute_reconciliation_cycle(
                                    &desired_state,
                                    &bot_controller,
                                    &yaml_manager,
                                    &stats,
                                    &reconciliation_config,
                                ).await;
                            }
                            
                            ReconciliationEvent::DesiredStateChanged => {
                                info!("📝 Estado deseado cambió, recargando configuración...");
                                if let Err(e) = Self::reload_desired_state(&desired_state, &yaml_manager).await {
                                    error!("❌ Error recargando estado deseado: {}", e);
                                } else {
                                    let _ = Self::execute_reconciliation_cycle(
                                        &desired_state,
                                        &bot_controller,
                                        &yaml_manager,
                                        &stats,
                                        &reconciliation_config,
                                    ).await;
                                }
                            }
                            
                            ReconciliationEvent::BotStateChanged { bot_id, new_status } => {
                                debug!("🤖 Bot {} cambió a estado: {:?}", bot_id, new_status);
                                // Verificar si necesitamos reconciliación inmediata
                                let needs_reconciliation = Self::check_if_reconciliation_needed(
                                    &desired_state,
                                    &bot_controller,
                                ).await;
                                
                                if needs_reconciliation {
                                    let _ = Self::execute_reconciliation_cycle(
                                        &desired_state,
                                        &bot_controller,
                                        &yaml_manager,
                                        &stats,
                                        &reconciliation_config,
                                    ).await;
                                }
                            }
                            
                            ReconciliationEvent::ReconciliationError { error } => {
                                error!("❌ Error en reconciliación: {}", error);
                            }
                            
                            ReconciliationEvent::ReconciliationCompleted { duration_ms, actions_taken } => {
                                info!("✅ Reconciliación completada en {}ms, {} acciones ejecutadas", 
                                     duration_ms, actions_taken);
                            }
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Ejecutar un ciclo completo de reconciliación
    async fn execute_reconciliation_cycle(
        desired_state: &Arc<RwLock<DesiredStateConfig>>,
        bot_controller: &Arc<BotController>,
        yaml_manager: &Arc<YamlConfigManager>,
        stats: &Arc<RwLock<ReconciliationStats>>,
        config: &ReconciliationConfig,
    ) -> Result<ReconciliationResult> {
        let start_time = Instant::now();
        
        // 1. 📖 Recargar estado deseado desde YAML (hot-reload)
        if let Err(e) = Self::reload_desired_state(desired_state, yaml_manager).await {
            warn!("⚠️ No se pudo recargar estado deseado: {}", e);
        }
        
        // 2. 🔍 Analizar diferencias entre estado actual y deseado
        let analysis_result = Self::analyze_state_drift(desired_state, bot_controller).await?;
        
        // 3. 📋 Generar plan de reconciliación
        let reconciliation_plan = Self::generate_reconciliation_plan(analysis_result).await?;
        
        // 4. ⚡ Ejecutar acciones de reconciliación
        let execution_result = Self::execute_reconciliation_plan(
            reconciliation_plan,
            bot_controller,
            config,
        ).await?;
        
        // 5. 📊 Actualizar estadísticas
        Self::update_reconciliation_stats(stats, &execution_result, start_time).await;
        
        let duration = start_time.elapsed();
        info!("🎯 Ciclo de reconciliación completado en {:?}", duration);
        
        Ok(execution_result)
    }
    
    /// Recargar estado deseado desde YAML
    async fn reload_desired_state(
        desired_state: &Arc<RwLock<DesiredStateConfig>>,
        yaml_manager: &Arc<YamlConfigManager>,
    ) -> Result<()> {
        match yaml_manager.load_desired_state().await {
            Ok(new_state) => {
                let mut state_guard = desired_state.write().await;
                *state_guard = new_state;
                debug!("✅ Estado deseado recargado desde YAML");
                Ok(())
            }
            Err(e) => {
                error!("❌ Error recargando estado deseado: {}", e);
                Err(e.into())
            }
        }
    }
    
    /// Verificar si se necesita reconciliación inmediata
    async fn check_if_reconciliation_needed(
        desired_state: &Arc<RwLock<DesiredStateConfig>>,
        bot_controller: &Arc<BotController>,
    ) -> bool {
        // Lógica simplificada para verificar drift
        match Self::analyze_state_drift(desired_state, bot_controller).await {
            Ok(analysis) => analysis.drift_detected,
            Err(_) => true, // Si no podemos analizar, asumimos que necesitamos reconciliación
        }
    }
    
    /// Analizar drift entre estado actual y deseado
    async fn analyze_state_drift(
        desired_state: &Arc<RwLock<DesiredStateConfig>>,
        bot_controller: &Arc<BotController>,
    ) -> Result<StateDriftAnalysis> {
        let desired = desired_state.read().await;
        let current_bots = bot_controller.list_bots().await?;
        
        let mut analysis = StateDriftAnalysis {
            drift_detected: false,
            missing_bots: Vec::new(),
            extra_bots: Vec::new(),
            status_mismatches: Vec::new(),
            config_drifts: Vec::new(),
        };
        
        // Crear mapas para análisis eficiente
        let current_bots_map: HashMap<String, &BotSummary> = current_bots
            .iter()
            .map(|bot| (bot.id.to_string(), bot))
            .collect();
        
        let desired_bots_map: HashMap<&String, &DesiredBotState> = desired
            .bots
            .iter()
            .map(|bot| (&bot.id, bot))
            .collect();
        
        // 🔍 Encontrar bots faltantes (en desired pero no en current)
        for desired_bot in &desired.bots {
            if !current_bots_map.contains_key(&desired_bot.id) {
                analysis.missing_bots.push(desired_bot.clone());
                analysis.drift_detected = true;
            }
        }
        
        // 🔍 Encontrar bots extra (en current pero no en desired)
        for (bot_id, current_bot) in &current_bots_map {
            if !desired_bots_map.contains_key(bot_id) {
                analysis.extra_bots.push((*current_bot).clone());
                analysis.drift_detected = true;
            }
        }
        
        // 🔍 Encontrar diferencias de estado
        for (bot_id, current_bot) in &current_bots_map {
            if let Some(desired_bot) = desired_bots_map.get(bot_id) {
                let desired_status = Self::map_desired_to_actual_status(&desired_bot.desired_status);
                
                if current_bot.status != desired_status {
                    analysis.status_mismatches.push(StatusMismatch {
                        bot_id: bot_id.clone(),
                        current_status: current_bot.status.clone(),
                        desired_status: desired_bot.desired_status.clone(),
                    });
                    analysis.drift_detected = true;
                }
                
                // TODO: Comparar configuraciones para detectar config drift
            }
        }
        
        if analysis.drift_detected {
            info!("🔍 Drift detectado: {} bots faltantes, {} bots extra, {} diferencias de estado", 
                 analysis.missing_bots.len(), 
                 analysis.extra_bots.len(), 
                 analysis.status_mismatches.len());
        } else {
            debug!("✅ No se detectó drift en el estado");
        }
        
        Ok(analysis)
    }
    
    /// Mapear estado deseado a estado actual del bot
    fn map_desired_to_actual_status(desired: &DesiredBotStatus) -> BotStatus {
        match desired {
            DesiredBotStatus::Running => BotStatus::Running,
            DesiredBotStatus::Stopped => BotStatus::Stopped,
            DesiredBotStatus::Paused => BotStatus::Stopped, // No tenemos estado pausado, mapeamos a stopped
            DesiredBotStatus::Maintenance => BotStatus::Stopped,
        }
    }
    
    /// Generar plan de reconciliación basado en análisis de drift
    async fn generate_reconciliation_plan(
        analysis: StateDriftAnalysis,
    ) -> Result<Vec<ReconciliationAction>> {
        let mut actions = Vec::new();
        
        // 🆕 Crear bots faltantes
        for missing_bot in analysis.missing_bots {
            let bot_config = Self::convert_desired_config_to_bot_config(&missing_bot.config);
            
            actions.push(ReconciliationAction::CreateBot {
                id: missing_bot.id.clone(),
                bot_type: missing_bot.bot_type,
                config: bot_config,
            });
            
            // Si el bot deseado debe estar corriendo, también agregamos acción de start
            if missing_bot.desired_status == DesiredBotStatus::Running {
                // Nota: Necesitaremos el UUID después de crear el bot
                // Por ahora, asumimos que la creación y el start serán manejados secuencialmente
            }
        }
        
        // 🗑️ Eliminar bots extra (opcional - puede ser peligroso en producción)
        // for extra_bot in analysis.extra_bots {
        //     actions.push(ReconciliationAction::DeleteBot {
        //         bot_id: extra_bot.id,
        //     });
        // }
        
        // 🔄 Corregir diferencias de estado
        for mismatch in analysis.status_mismatches {
            let bot_id = Uuid::parse_str(&mismatch.bot_id).map_err(|e| {
                anyhow::anyhow!("Invalid bot ID format: {}", e)
            })?;
            
            match mismatch.desired_status {
                DesiredBotStatus::Running => {
                    actions.push(ReconciliationAction::StartBot {
                        bot_id,
                        config: BotConfig::default_for_id(Uuid::new_v4()), // TODO: Usar configuración real
                    });
                }
                DesiredBotStatus::Stopped | DesiredBotStatus::Paused | DesiredBotStatus::Maintenance => {
                    actions.push(ReconciliationAction::StopBot { bot_id });
                }
            }
        }
        
        info!("📋 Plan de reconciliación generado: {} acciones", actions.len());
        Ok(actions)
    }
    
    /// Convertir configuración deseada a configuración de bot
    fn convert_desired_config_to_bot_config(_desired_config: &DesiredBotConfig) -> BotConfig {
        // TODO: Implementar conversión completa basada en los campos disponibles
        BotConfig::default_for_id(Uuid::new_v4())
    }
    
    /// Ejecutar plan de reconciliación
    async fn execute_reconciliation_plan(
        plan: Vec<ReconciliationAction>,
        bot_controller: &Arc<BotController>,
        config: &ReconciliationConfig,
    ) -> Result<ReconciliationResult> {
        let start_time = Instant::now();
        let mut successful_actions = Vec::new();
        let mut failed_actions = Vec::new();
        
        info!("⚡ Ejecutando plan de reconciliación con {} acciones", plan.len());
        
        for (i, action) in plan.iter().enumerate() {
            info!("🔧 Ejecutando acción {}/{}: {:?}", i + 1, plan.len(), action);
            
            let result = Self::execute_single_action(action, bot_controller).await;
            
            match result {
                Ok(_) => {
                    successful_actions.push(action.clone());
                    info!("✅ Acción ejecutada exitosamente");
                }
                Err(e) => {
                    failed_actions.push((action.clone(), e.to_string()));
                    error!("❌ Acción falló: {}", e);
                    
                    // Verificar si debemos continuar o abortar
                    if failed_actions.len() as u32 >= config.max_retries {
                        warn!("⚠️ Máximo número de fallos alcanzado, abortando reconciliación");
                        break;
                    }
                }
            }
        }
        
        let duration = start_time.elapsed();
        let drift_detected = !failed_actions.is_empty();
        
        let summary = format!(
            "Reconciliación completada: {}/{} acciones exitosas en {:?}",
            successful_actions.len(),
            plan.len(),
            duration
        );
        
        Ok(ReconciliationResult {
            successful_actions,
            failed_actions,
            drift_detected,
            duration,
            post_reconciliation_summary: summary,
        })
    }
    
    /// Ejecutar una sola acción de reconciliación
    async fn execute_single_action(
        action: &ReconciliationAction,
        bot_controller: &Arc<BotController>,
    ) -> Result<()> {
        match action {
            ReconciliationAction::CreateBot { id, bot_type, config } => {
                let bot_id = bot_controller.create_bot(bot_type.clone(), config.clone()).await?;
                info!("🆕 Bot creado: {} -> {}", id, bot_id);
                Ok(())
            }
            
            ReconciliationAction::StartBot { bot_id, config } => {
                bot_controller.start_bot(*bot_id, config.clone()).await?;
                info!("🚀 Bot iniciado: {}", bot_id);
                Ok(())
            }
            
            ReconciliationAction::StopBot { bot_id } => {
                bot_controller.stop_bot(*bot_id).await?;
                info!("🛑 Bot detenido: {}", bot_id);
                Ok(())
            }
            
            ReconciliationAction::PauseBot { bot_id } => {
                // Por ahora, pausar = detener
                bot_controller.stop_bot(*bot_id).await?;
                info!("⏸️ Bot pausado: {}", bot_id);
                Ok(())
            }
            
            ReconciliationAction::UpdateBotConfig { bot_id, new_config: _ } => {
                // TODO: Implementar actualización de configuración
                warn!("⚠️ Actualización de configuración no implementada aún para bot: {}", bot_id);
                Ok(())
            }
            
            ReconciliationAction::DeleteBot { bot_id } => {
                // TODO: Implementar eliminación de bot
                warn!("⚠️ Eliminación de bot no implementada aún para bot: {}", bot_id);
                Ok(())
            }
        }
    }
    
    /// Actualizar estadísticas de reconciliación
    async fn update_reconciliation_stats(
        stats: &Arc<RwLock<ReconciliationStats>>,
        result: &ReconciliationResult,
        start_time: Instant,
    ) {
        let mut stats_guard = stats.write().await;
        
        stats_guard.total_reconciliations += 1;
        
        if result.failed_actions.is_empty() {
            stats_guard.successful_reconciliations += 1;
            stats_guard.last_successful_reconciliation = Some(start_time);
        } else {
            stats_guard.failed_reconciliations += 1;
        }
        
        stats_guard.total_actions_executed += result.successful_actions.len() as u64;
        stats_guard.last_drift_detected = result.drift_detected;
        
        // Calcular tiempo promedio
        let duration_ms = result.duration.as_millis() as f64;
        stats_guard.avg_reconciliation_time_ms = 
            (stats_guard.avg_reconciliation_time_ms * (stats_guard.total_reconciliations - 1) as f64 + duration_ms) 
            / stats_guard.total_reconciliations as f64;
    }
    
    /// Obtener estadísticas de reconciliación
    pub async fn get_reconciliation_stats(&self) -> ReconciliationStats {
        self.stats.read().await.clone()
    }
    
    /// Trigger manual reconciliation
    pub async fn trigger_reconciliation(&self) -> Result<()> {
        self.reconciliation_tx
            .send(ReconciliationEvent::TriggerReconciliation)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to trigger reconciliation: {}", e))
    }
    
    /// Notificar cambio en estado deseado
    pub async fn notify_desired_state_changed(&self) -> Result<()> {
        self.reconciliation_tx
            .send(ReconciliationEvent::DesiredStateChanged)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to notify desired state change: {}", e))
    }
    
    /// Parar el reconciliador
    pub async fn stop(&self) {
        let mut should_stop = self.should_stop.write().await;
        *should_stop = true;
        info!("🛑 Reconciliador de estado deseado detenido");
    }
}

/// Resultado del análisis de drift del estado
#[derive(Debug)]
pub struct StateDriftAnalysis {
    /// Se detectó drift entre estado actual y deseado
    pub drift_detected: bool,
    
    /// Bots que faltan (están en desired pero no en current)
    pub missing_bots: Vec<DesiredBotState>,
    
    /// Bots extra (están en current pero no en desired)
    pub extra_bots: Vec<BotSummary>,
    
    /// Diferencias de estado entre bots existentes
    pub status_mismatches: Vec<StatusMismatch>,
    
    /// Diferencias de configuración
    pub config_drifts: Vec<ConfigDrift>,
}

/// Diferencia de estado de un bot
#[derive(Debug)]
pub struct StatusMismatch {
    pub bot_id: String,
    pub current_status: BotStatus,
    pub desired_status: DesiredBotStatus,
}

/// Diferencia de configuración de un bot
#[derive(Debug)]
pub struct ConfigDrift {
    pub bot_id: String,
    pub field: String,
    pub current_value: String,
    pub desired_value: String,
}
