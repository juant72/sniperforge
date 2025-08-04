//! API Gateway Module
//! 
//! This module provides the main REST API gateway for the SniperForge bot ecosystem.
//! It handles HTTP requests for bot management, configuration, and monitoring.

use actix_web::{web, App, HttpServer, HttpResponse, Result, middleware::Logger};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::api::bot_interface::{BotConfig, BotType, BotStatus};
use crate::bots::bot_factory::{BotFactory, BotRegistry};

/// API Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub rate_limit_per_minute: u32,
    pub max_concurrent_requests: usize,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            cors_enabled: true,
            rate_limit_per_minute: 1000,
            max_concurrent_requests: 100,
        }
    }
}

/// API request to create a new bot
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBotRequest {
    pub bot_type: BotType,
    pub config: BotConfig,
    pub name: Option<String>,
}

/// API response for bot creation
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBotResponse {
    pub bot_id: Uuid,
    pub status: String,
    pub message: String,
}

/// API request to update bot configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBotConfigRequest {
    pub config: BotConfig,
}

/// API response for bot operations
#[derive(Debug, Serialize, Deserialize)]
pub struct BotOperationResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Shared application state
pub struct AppState {
    pub bot_factory: Arc<RwLock<BotFactory>>,
    pub bot_registry: Arc<RwLock<BotRegistry>>,
}

/// API Gateway struct
pub struct ApiGateway {
    config: GatewayConfig,
    state: Arc<AppState>,
}

impl ApiGateway {
    /// Create a new API Gateway instance
    pub fn new(config: GatewayConfig) -> Self {
        let state = Arc::new(AppState {
            bot_factory: Arc::new(RwLock::new(BotFactory::new())),
            bot_registry: Arc::new(RwLock::new(BotRegistry::new())),
        });

        Self { config, state }
    }

    /// Start the API Gateway server
    pub async fn start(&self) -> std::io::Result<()> {
        let bind_address = format!("{}:{}", self.config.host, self.config.port);
        println!("🚀 SniperForge API Gateway starting on {}", bind_address);

        HttpServer::new({
            let state = self.state.clone();
            move || {
                App::new()
                    .app_data(web::Data::new(state.clone()))
                    .wrap(Logger::default())
                    .configure(configure_routes)
            }
        })
        .bind(&bind_address)?
        .run()
        .await
    }
}

/// Configure API routes
fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(
                web::scope("/bots")
                    .route("", web::get().to(list_bots))
                    .route("", web::post().to(create_bot))
                    .route("/{bot_id}", web::get().to(get_bot))
                    .route("/{bot_id}", web::delete().to(delete_bot))
                    .route("/{bot_id}/start", web::post().to(start_bot))
                    .route("/{bot_id}/stop", web::post().to(stop_bot))
                    .route("/{bot_id}/configure", web::put().to(configure_bot))
                    .route("/{bot_id}/status", web::get().to(get_bot_status))
                    .route("/{bot_id}/metrics", web::get().to(get_bot_metrics))
                    .route("/{bot_id}/health", web::get().to(get_bot_health))
                    .route("/types", web::get().to(get_bot_types))
            )
            .service(
                web::scope("/system")
                    .route("/health", web::get().to(system_health))
                    .route("/metrics", web::get().to(system_metrics))
                    .route("/status", web::get().to(system_status))
            )
    );
}

/// List all bots
async fn list_bots(state: web::Data<Arc<AppState>>) -> Result<HttpResponse> {
    let registry = state.bot_registry.read().await;
    let bots = registry.list_bots().await;
    
    Ok(HttpResponse::Ok().json(BotOperationResponse {
        success: true,
        message: "Bots listed successfully".to_string(),
        data: Some(serde_json::to_value(bots).unwrap()),
    }))
}

/// Create a new bot
async fn create_bot(
    state: web::Data<Arc<AppState>>,
    req: web::Json<CreateBotRequest>,
) -> Result<HttpResponse> {
    let factory = state.bot_factory.write().await;
    
    match factory.create_bot(req.bot_type.clone(), req.config.clone()).await {
        Ok(bot_id) => {
            let mut registry = state.bot_registry.write().await;
            registry.register_bot(
                bot_id,
                req.bot_type.clone(),
                req.name.clone().unwrap_or_else(|| format!("{:?}-{}", req.bot_type, bot_id)),
            ).await;
            
            Ok(HttpResponse::Created().json(CreateBotResponse {
                bot_id,
                status: "created".to_string(),
                message: "Bot created successfully".to_string(),
            }))
        },
        Err(e) => Ok(HttpResponse::BadRequest().json(BotOperationResponse {
            success: false,
            message: format!("Failed to create bot: {}", e),
            data: None,
        }))
    }
}

/// Get bot details
async fn get_bot(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let bot_id = path.into_inner();
    let registry = state.bot_registry.read().await;
    
    if let Some(bot_info) = registry.get_bot_info(bot_id).await {
        Ok(HttpResponse::Ok().json(BotOperationResponse {
            success: true,
            message: "Bot found".to_string(),
            data: Some(serde_json::to_value(bot_info).unwrap()),
        }))
    } else {
        Ok(HttpResponse::NotFound().json(BotOperationResponse {
            success: false,
            message: "Bot not found".to_string(),
            data: None,
        }))
    }
}

/// Delete a bot
async fn delete_bot(
    state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let bot_id = path.into_inner();
    let mut registry = state.bot_registry.write().await;
    
    registry.unregister_bot(bot_id).await;
    
    Ok(HttpResponse::Ok().json(BotOperationResponse {
        success: true,
        message: "Bot deleted successfully".to_string(),
        data: None,
    }))
}

/// Start a bot
async fn start_bot(
    _state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let _bot_id = path.into_inner();
    // TODO: Implement actual bot starting logic
    
    Ok(HttpResponse::Ok().json(BotOperationResponse {
        success: true,
        message: "Bot start command sent".to_string(),
        data: None,
    }))
}

/// Stop a bot
async fn stop_bot(
    _state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let _bot_id = path.into_inner();
    // TODO: Implement actual bot stopping logic
    
    Ok(HttpResponse::Ok().json(BotOperationResponse {
        success: true,
        message: "Bot stop command sent".to_string(),
        data: None,
    }))
}

/// Configure a bot
async fn configure_bot(
    _state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
    _req: web::Json<UpdateBotConfigRequest>,
) -> Result<HttpResponse> {
    let _bot_id = path.into_inner();
    // TODO: Implement actual bot configuration logic
    
    Ok(HttpResponse::Ok().json(BotOperationResponse {
        success: true,
        message: "Bot configuration updated".to_string(),
        data: None,
    }))
}

/// Get bot status
async fn get_bot_status(
    _state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let _bot_id = path.into_inner();
    // TODO: Implement actual bot status retrieval
    
    let status = BotStatus::Stopped; // Placeholder
    
    Ok(HttpResponse::Ok().json(BotOperationResponse {
        success: true,
        message: "Bot status retrieved".to_string(),
        data: Some(serde_json::to_value(status).unwrap()),
    }))
}

/// Get bot metrics
async fn get_bot_metrics(
    _state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let _bot_id = path.into_inner();
    // TODO: Implement actual bot metrics retrieval
    
    Ok(HttpResponse::Ok().json(BotOperationResponse {
        success: true,
        message: "Bot metrics retrieved".to_string(),
        data: None,
    }))
}

/// Get bot health
async fn get_bot_health(
    _state: web::Data<Arc<AppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let _bot_id = path.into_inner();
    // TODO: Implement actual bot health check
    
    let health = crate::api::bot_interface::HealthStatus {
        status: crate::api::bot_interface::HealthLevel::Healthy,
        checks: Vec::new(),
        timestamp: chrono::Utc::now(),
        details: std::collections::HashMap::new(),
    }; // System health check
    
    Ok(HttpResponse::Ok().json(BotOperationResponse {
        success: true,
        message: "Bot health status retrieved".to_string(),
        data: Some(serde_json::to_value(health).unwrap()),
    }))
}

/// Get available bot types
async fn get_bot_types(state: web::Data<Arc<AppState>>) -> Result<HttpResponse> {
    let factory = state.bot_factory.read().await;
    let types = factory.get_bot_types();
    
    Ok(HttpResponse::Ok().json(BotOperationResponse {
        success: true,
        message: "Bot types retrieved".to_string(),
        data: Some(serde_json::to_value(types).unwrap()),
    }))
}

/// System health check
async fn system_health(_state: web::Data<Arc<AppState>>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(BotOperationResponse {
        success: true,
        message: "System is healthy".to_string(),
        data: None,
    }))
}

/// System metrics
async fn system_metrics(_state: web::Data<Arc<AppState>>) -> Result<HttpResponse> {
    // TODO: Implement actual system metrics collection
    Ok(HttpResponse::Ok().json(BotOperationResponse {
        success: true,
        message: "System metrics retrieved".to_string(),
        data: None,
    }))
}

/// System status
async fn system_status(_state: web::Data<Arc<AppState>>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(BotOperationResponse {
        success: true,
        message: "System status retrieved".to_string(),
        data: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_config_default() {
        let config = GatewayConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(config.cors_enabled);
    }

    #[tokio::test]
    async fn test_gateway_creation() {
        let config = GatewayConfig::default();
        let gateway = ApiGateway::new(config);
        
        // Test that the gateway was created successfully
        assert_eq!(gateway.config.port, 8080);
    }
}
