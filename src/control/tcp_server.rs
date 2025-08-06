use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;
use tracing::{info, error};

use crate::api::{BotType, BotStatus, BotMetrics, BotConfig, PersistedSystemMetrics};
use crate::control::{BotController, BotSummary, SystemMetrics, SystemStateSummary, MassControlResult, SystemResourceStatus};

pub struct TcpControlServer {
    bot_controller: Arc<BotController>,
    listener: TcpListener,
    port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TcpCommand {
    ListBots,
    CreateBot { bot_type: BotType, config: BotConfig },
    StartBot { bot_id: Uuid, config: BotConfig },
    StopBot { bot_id: Uuid },
    GetBotStatus { bot_id: Uuid },
    GetBotMetrics { bot_id: Uuid },
    GetSystemMetrics,
    GetSystemState,
    GetMetricsHistory { hours: u32 },
    CreateBackup,
    ForceSave,
    StartAllBots,
    StopAllBots,
    GetResourceStatus,
    Ping,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TcpResponse {
    BotList(Vec<BotSummary>),
    BotCreated { bot_id: Uuid },
    BotStarted { bot_id: Uuid },
    BotStopped { bot_id: Uuid },
    BotStatus(BotStatus),
    BotMetrics(BotMetrics),
    SystemMetrics(SystemMetrics),
    SystemState(SystemStateSummary),
    MetricsHistory(Vec<PersistedSystemMetrics>),
    BackupCreated(String),
    MassControlResult(MassControlResult),
    ResourceStatus(SystemResourceStatus),
    Pong,
    Success(String),
    Error(String),
}

impl TcpControlServer {
    pub async fn new(bot_controller: Arc<BotController>, port: u16) -> Result<Self> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        info!("🌐 TCP Control Server listening on port {}", port);
        
        Ok(Self {
            bot_controller,
            listener,
            port,
        })
    }
    
    pub async fn run(&self) -> Result<()> {
        info!("🚀 Starting TCP Control Server on port {}...", self.port);
        
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    info!("📡 New TCP connection from: {}", addr);
                    
                    let controller = self.bot_controller.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, controller).await {
                            error!("❌ TCP connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("❌ TCP accept error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }
    
    async fn handle_connection(
        mut stream: TcpStream, 
        controller: Arc<BotController>
    ) -> Result<()> {
        let mut buffer = [0; 4096];
        
        loop {
            // Read command
            let n = match stream.read(&mut buffer).await {
                Ok(0) => break, // Connection closed
                Ok(n) => n,
                Err(e) => {
                    error!("❌ TCP read error: {}", e);
                    break;
                }
            };
            
            let command_data = &buffer[..n];
            
            // Parse command
            let command: TcpCommand = match serde_json::from_slice(command_data) {
                Ok(cmd) => cmd,
                Err(e) => {
                    let error_response = TcpResponse::Error(format!("Invalid command JSON: {}", e));
                    let response_data = serde_json::to_vec(&error_response)?;
                    if let Err(e) = stream.write_all(&response_data).await {
                        error!("❌ TCP write error: {}", e);
                    }
                    continue;
                }
            };
            
            // Process command
            let response = Self::process_command(command, &controller).await;
            
            // Send response
            let response_data = match serde_json::to_vec(&response) {
                Ok(data) => data,
                Err(e) => {
                    error!("❌ Failed to serialize response: {}", e);
                    let error_response = TcpResponse::Error("Internal server error".to_string());
                    serde_json::to_vec(&error_response)?
                }
            };
            
            if let Err(e) = stream.write_all(&response_data).await {
                error!("❌ TCP write error: {}", e);
                break;
            }
            
            // Handle shutdown command
            if matches!(response, TcpResponse::Success(ref msg) if msg == "Shutdown initiated") {
                info!("🛑 Shutdown command processed, closing connection");
                break;
            }
        }
        
        info!("📡 TCP connection closed");
        Ok(())
    }
    
    async fn process_command(
        command: TcpCommand, 
        controller: &Arc<BotController>
    ) -> TcpResponse {
        // 🔄 HOT-RELOAD AUTOMÁTICO: Recargar configuraciones antes de cada comando CLI
        info!("🔄 Hot-reload: Updating configurations from disk...");
        if let Err(e) = controller.hot_reload_configs().await {
            error!("⚠️ Hot-reload failed: {}", e);
            // No fallar el comando por esto, solo advertir
        } else {
            info!("✅ Hot-reload completed successfully");
        }
        
        match command {
            TcpCommand::ListBots => {
                match controller.list_bots().await {
                    Ok(bots) => {
                        info!("📋 Listing {} bots", bots.len());
                        TcpResponse::BotList(bots)
                    }
                    Err(e) => {
                        error!("❌ Error listing bots: {}", e);
                        TcpResponse::Error(e.to_string())
                    }
                }
            }
            
            TcpCommand::CreateBot { bot_type, config } => {
                match controller.create_bot(bot_type, config).await {
                    Ok(bot_id) => {
                        info!("✅ Created bot: {}", bot_id);
                        TcpResponse::BotCreated { bot_id }
                    }
                    Err(e) => {
                        error!("❌ Error creating bot: {}", e);
                        TcpResponse::Error(e.to_string())
                    }
                }
            }
            
            TcpCommand::StartBot { bot_id, config } => {
                match controller.start_bot(bot_id, config).await {
                    Ok(_) => {
                        info!("🚀 Started bot: {}", bot_id);
                        TcpResponse::BotStarted { bot_id }
                    }
                    Err(e) => {
                        error!("❌ Error starting bot: {}", e);
                        TcpResponse::Error(e.to_string())
                    }
                }
            }
            
            TcpCommand::StopBot { bot_id } => {
                match controller.stop_bot(bot_id).await {
                    Ok(_) => {
                        info!("🛑 Stopped bot: {}", bot_id);
                        TcpResponse::BotStopped { bot_id }
                    }
                    Err(e) => {
                        error!("❌ Error stopping bot: {}", e);
                        TcpResponse::Error(e.to_string())
                    }
                }
            }
            
            TcpCommand::GetBotStatus { bot_id } => {
                match controller.get_bot_status(bot_id).await {
                    Ok(status) => {
                        info!("📊 Bot {} status: {:?}", bot_id, status);
                        TcpResponse::BotStatus(status)
                    }
                    Err(e) => {
                        error!("❌ Error getting bot status: {}", e);
                        TcpResponse::Error(e.to_string())
                    }
                }
            }
            
            TcpCommand::GetBotMetrics { bot_id } => {
                match controller.get_bot_metrics(bot_id).await {
                    Ok(metrics) => {
                        info!("📈 Bot {} metrics retrieved", bot_id);
                        TcpResponse::BotMetrics(metrics)
                    }
                    Err(e) => {
                        error!("❌ Error getting bot metrics: {}", e);
                        TcpResponse::Error(e.to_string())
                    }
                }
            }
            
            TcpCommand::GetSystemMetrics => {
                match controller.get_system_metrics().await {
                    Ok(metrics) => {
                        info!("📊 System metrics: {} bots, {} running", metrics.total_bots, metrics.running_bots);
                        TcpResponse::SystemMetrics(metrics)
                    }
                    Err(e) => {
                        error!("❌ Error getting system metrics: {}", e);
                        TcpResponse::Error(e.to_string())
                    }
                }
            }
            
            TcpCommand::GetSystemState => {
                match controller.get_system_state_summary().await {
                    Ok(state) => TcpResponse::SystemState(state),
                    Err(e) => TcpResponse::Error(e.to_string()),
                }
            }
            
            TcpCommand::GetMetricsHistory { hours } => {
                match controller.get_historical_metrics(hours).await {
                    Ok(history) => TcpResponse::MetricsHistory(history),
                    Err(e) => TcpResponse::Error(e.to_string()),
                }
            }
            
            TcpCommand::CreateBackup => {
                match controller.create_system_backup().await {
                    Ok(backup_path) => TcpResponse::BackupCreated(backup_path),
                    Err(e) => TcpResponse::Error(e.to_string()),
                }
            }
            
            TcpCommand::ForceSave => {
                match controller.force_save_all_state().await {
                    Ok(()) => TcpResponse::Success("All state saved to persistence".to_string()),
                    Err(e) => TcpResponse::Error(e.to_string()),
                }
            }
            
            TcpCommand::StartAllBots => {
                match controller.start_all_bots().await {
                    Ok(result) => TcpResponse::MassControlResult(result),
                    Err(e) => TcpResponse::Error(e.to_string()),
                }
            }
            
            TcpCommand::StopAllBots => {
                match controller.stop_all_bots().await {
                    Ok(result) => TcpResponse::MassControlResult(result),
                    Err(e) => TcpResponse::Error(e.to_string()),
                }
            }
            
            TcpCommand::GetResourceStatus => {
                match controller.get_system_resource_status().await {
                    Ok(status) => TcpResponse::ResourceStatus(status),
                    Err(e) => TcpResponse::Error(e.to_string()),
                }
            }
            
            TcpCommand::Ping => {
                info!("🏓 Ping received");
                TcpResponse::Pong
            }
            
            TcpCommand::Shutdown => {
                info!("🛑 Shutdown command received");
                TcpResponse::Success("Shutdown initiated".to_string())
            }
        }
    }
}
