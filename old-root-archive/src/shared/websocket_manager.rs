//! WebSocket Manager for Low-Latency Solana Communication
//!
//! This module provides:
//! - Real-time account updates
//! - Transaction confirmations
//! - Pool state changes
//! - Price feeds
//!
//! Optimized for minimal latency in trading operations

use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use url::Url;

use crate::config::Config;

/// WebSocket connection configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub rpc_ws_url: String,
    pub connection_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub max_reconnect_attempts: u32,
    pub buffer_size: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            rpc_ws_url: "wss://api.devnet.solana.com".to_string(),
            connection_timeout_ms: 5000,
            heartbeat_interval_ms: 30000,
            max_reconnect_attempts: 10,
            buffer_size: 1000,
        }
    }
}

/// Types of subscriptions available
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubscriptionType {
    AccountUpdate(Pubkey),
    ProgramUpdate(Pubkey),
    SlotUpdate,
    RootUpdate,
    LogsUpdate(String), // Program ID filter
}

/// WebSocket event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketEvent {
    pub subscription_type: String,
    pub data: Value,
    pub timestamp: u64,
    pub slot: Option<u64>,
}

/// WebSocket manager for real-time Solana data
pub struct WebSocketManager {
    config: WebSocketConfig,
    subscriptions: Arc<RwLock<HashMap<u64, SubscriptionType>>>,
    event_sender: broadcast::Sender<WebSocketEvent>,
    command_sender: mpsc::UnboundedSender<WebSocketCommand>,
    is_connected: Arc<RwLock<bool>>,
    subscription_counter: Arc<RwLock<u64>>,
    rpc_pool: Option<Arc<crate::shared::rpc_pool::RpcConnectionPool>>, // NEW: Optional RPC pool for premium WebSocket URLs
}

/// Internal command for WebSocket operations
#[derive(Debug)]
enum WebSocketCommand {
    Subscribe(SubscriptionType),
    Unsubscribe(u64),
    Reconnect,
    Close,
}

impl WebSocketManager {
    /// Create new WebSocket manager
    pub async fn new(config: &Config) -> Result<Self> {
        Self::new_with_rpc_pool(config, None).await
    }

    /// Create new WebSocket manager with optional RPC pool for premium WebSocket URLs
    pub async fn new_with_rpc_pool(
        config: &Config,
        rpc_pool: Option<Arc<crate::shared::rpc_pool::RpcConnectionPool>>,
    ) -> Result<Self> {
        info!("🌐 Initializing WebSocket manager");

        info!(
            "🔧 Loading config for network: {}",
            config.network.environment
        );
        info!("🔧 Primary RPC: {}", config.network.primary_rpc());

        // Try to get the best WebSocket URL (premium if available)
        let websocket_url = if let Some(ref pool) = rpc_pool {
            if let Some(premium_ws_url) = pool.get_best_websocket_url().await {
                info!("🌟 Using premium WebSocket endpoint");
                premium_ws_url
            } else {
                info!("💡 No premium WebSocket available, using configured endpoint");
                config.network.websocket_url().to_string()
            }
        } else {
            config.network.websocket_url().to_string()
        };

        info!("🔧 WebSocket URL: {}", websocket_url);

        let ws_config = WebSocketConfig {
            rpc_ws_url: websocket_url,
            ..WebSocketConfig::default()
        };

        let (event_sender, _) = broadcast::channel(ws_config.buffer_size);
        let (command_sender, command_receiver) = mpsc::unbounded_channel();

        let manager = Self {
            config: ws_config,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            command_sender,
            is_connected: Arc::new(RwLock::new(false)),
            subscription_counter: Arc::new(RwLock::new(0)),
            rpc_pool,
        };

        // Start the WebSocket task
        manager.start_websocket_task(command_receiver).await?;

        info!("✅ WebSocket manager initialized");
        Ok(manager)
    }

    /// Subscribe to account updates
    pub async fn subscribe_account(&self, pubkey: Pubkey) -> Result<u64> {
        let subscription_id = {
            let mut counter = self.subscription_counter.write().await;
            *counter += 1;
            *counter
        };

        let subscription_type = SubscriptionType::AccountUpdate(pubkey);

        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id, subscription_type.clone());
        }

        self.command_sender
            .send(WebSocketCommand::Subscribe(subscription_type))?;

        info!(
            "📡 Subscribed to account updates: {} (ID: {})",
            pubkey, subscription_id
        );
        Ok(subscription_id)
    }

    /// Subscribe to program updates
    pub async fn subscribe_program(&self, program_id: Pubkey) -> Result<u64> {
        let subscription_id = {
            let mut counter = self.subscription_counter.write().await;
            *counter += 1;
            *counter
        };

        let subscription_type = SubscriptionType::ProgramUpdate(program_id);

        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id, subscription_type.clone());
        }

        self.command_sender
            .send(WebSocketCommand::Subscribe(subscription_type))?;

        info!(
            "📡 Subscribed to program updates: {} (ID: {})",
            program_id, subscription_id
        );
        Ok(subscription_id)
    }

    /// Subscribe to slot updates
    pub async fn subscribe_slots(&self) -> Result<u64> {
        let subscription_id = {
            let mut counter = self.subscription_counter.write().await;
            *counter += 1;
            *counter
        };

        let subscription_type = SubscriptionType::SlotUpdate;

        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id, subscription_type.clone());
        }

        self.command_sender
            .send(WebSocketCommand::Subscribe(subscription_type))?;

        info!("📡 Subscribed to slot updates (ID: {})", subscription_id);
        Ok(subscription_id)
    }

    /// Subscribe to logs (useful for MEV detection)
    pub async fn subscribe_logs(&self, program_filter: &str) -> Result<u64> {
        let subscription_id = {
            let mut counter = self.subscription_counter.write().await;
            *counter += 1;
            *counter
        };

        let subscription_type = SubscriptionType::LogsUpdate(program_filter.to_string());

        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id, subscription_type.clone());
        }

        self.command_sender
            .send(WebSocketCommand::Subscribe(subscription_type))?;

        info!(
            "📡 Subscribed to logs: {} (ID: {})",
            program_filter, subscription_id
        );
        Ok(subscription_id)
    }

    /// Unsubscribe from updates
    pub async fn unsubscribe(&self, subscription_id: u64) -> Result<()> {
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.remove(&subscription_id);
        }

        self.command_sender
            .send(WebSocketCommand::Unsubscribe(subscription_id))?;

        info!("📴 Unsubscribed from updates (ID: {})", subscription_id);
        Ok(())
    }

    /// Get event receiver for listening to WebSocket events
    pub fn get_event_receiver(&self) -> broadcast::Receiver<WebSocketEvent> {
        self.event_sender.subscribe()
    }

    /// Check if WebSocket is connected
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }

    /// Force reconnection
    pub async fn reconnect(&self) -> Result<()> {
        self.command_sender.send(WebSocketCommand::Reconnect)?;
        Ok(())
    }

    /// Upgrade to premium WebSocket if available
    pub async fn upgrade_to_premium_websocket(&mut self) -> Result<()> {
        if let Some(ref pool) = self.rpc_pool {
            if let Some(premium_ws_url) = pool.get_best_websocket_url().await {
                info!("🚀 Upgrading to premium WebSocket: {}", premium_ws_url);

                // Update configuration
                self.config.rpc_ws_url = premium_ws_url;

                // Force reconnection with new URL
                self.command_sender.send(WebSocketCommand::Reconnect)?;

                info!("✅ WebSocket upgraded to premium endpoint");
                return Ok(());
            }
        }

        info!("💡 No premium WebSocket upgrade available");
        Ok(())
    }

    /// Start the WebSocket task
    async fn start_websocket_task(
        &self,
        mut command_receiver: mpsc::UnboundedReceiver<WebSocketCommand>,
    ) -> Result<()> {
        let config = self.config.clone();
        let subscriptions = self.subscriptions.clone();
        let event_sender = self.event_sender.clone();
        let is_connected = self.is_connected.clone();

        tokio::spawn(async move {
            let mut reconnect_attempts = 0;

            loop {
                match Self::websocket_connection_loop(
                    &config,
                    &mut command_receiver,
                    &subscriptions,
                    &event_sender,
                    &is_connected,
                )
                .await
                {
                    Ok(()) => {
                        info!("🔌 WebSocket connection closed gracefully");
                        break;
                    }
                    Err(e) => {
                        error!("❌ WebSocket connection error: {}", e);

                        {
                            let mut connected = is_connected.write().await;
                            *connected = false;
                        }

                        reconnect_attempts += 1;
                        if reconnect_attempts >= config.max_reconnect_attempts {
                            error!("💥 Max reconnection attempts reached, giving up");
                            break;
                        }

                        let delay = std::cmp::min(1000 * reconnect_attempts as u64, 30000);
                        warn!(
                            "🔄 Reconnecting in {}ms (attempt {}/{})",
                            delay, reconnect_attempts, config.max_reconnect_attempts
                        );

                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// Main WebSocket connection loop
    async fn websocket_connection_loop(
        config: &WebSocketConfig,
        command_receiver: &mut mpsc::UnboundedReceiver<WebSocketCommand>,
        _subscriptions: &Arc<RwLock<HashMap<u64, SubscriptionType>>>,
        event_sender: &broadcast::Sender<WebSocketEvent>,
        is_connected: &Arc<RwLock<bool>>,
    ) -> Result<()> {
        info!("🔗 Connecting to WebSocket: {}", config.rpc_ws_url);

        let url = Url::parse(&config.rpc_ws_url)?;

        // Add connection timeout to avoid hanging
        let connect_timeout = tokio::time::Duration::from_millis(config.connection_timeout_ms);
        let connection_result =
            tokio::time::timeout(connect_timeout, connect_async(url.as_str())).await;

        let (ws_stream, _) = match connection_result {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => return Err(anyhow!("WebSocket connection failed: {}", e)),
            Err(_) => {
                return Err(anyhow!(
                    "WebSocket connection timeout after {}ms",
                    config.connection_timeout_ms
                ))
            }
        };

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        {
            let mut connected = is_connected.write().await;
            *connected = true;
        }

        info!("✅ WebSocket connected successfully");

        // Main event loop
        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Err(e) = Self::handle_websocket_message(&text, event_sender).await {
                                warn!("⚠️  Error handling WebSocket message: {}", e);
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("🔌 WebSocket closed by server");
                            break;
                        }
                        Some(Err(e)) => {
                            error!("❌ WebSocket receive error: {}", e);
                            break;
                        }
                        None => {
                            warn!("⚠️  WebSocket stream ended");
                            break;
                        }
                        _ => {} // Ignore other message types
                    }
                }

                // Handle internal commands
                cmd = command_receiver.recv() => {
                    match cmd {
                        Some(WebSocketCommand::Subscribe(sub_type)) => {
                            if let Err(e) = Self::send_subscribe_message(&mut ws_sender, &sub_type).await {
                                error!("❌ Failed to send subscribe message: {}", e);
                            }
                        }
                        Some(WebSocketCommand::Unsubscribe(id)) => {
                            if let Err(e) = Self::send_unsubscribe_message(&mut ws_sender, id).await {
                                error!("❌ Failed to send unsubscribe message: {}", e);
                            }
                        }
                        Some(WebSocketCommand::Reconnect) => {
                            info!("🔄 Reconnection requested");
                            break;
                        }
                        Some(WebSocketCommand::Close) => {
                            info!("🛑 Close requested");
                            let _ = ws_sender.close().await;
                            break;
                        }
                        None => {
                            debug!("Command channel closed - WebSocket shutting down gracefully");
                            break;
                        }
                    }
                }
            }
        }

        {
            let mut connected = is_connected.write().await;
            *connected = false;
        }

        Ok(())
    }

    /// Handle incoming WebSocket message
    async fn handle_websocket_message(
        text: &str,
        event_sender: &broadcast::Sender<WebSocketEvent>,
    ) -> Result<()> {
        let json: Value = serde_json::from_str(text)?;

        // Check if it's a notification (has "method" field)
        if let Some(method) = json.get("method").and_then(|m| m.as_str()) {
            let event = WebSocketEvent {
                subscription_type: method.to_string(),
                data: json.clone(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                slot: json
                    .get("params")
                    .and_then(|p| p.get("result"))
                    .and_then(|r| r.get("context"))
                    .and_then(|c| c.get("slot"))
                    .and_then(|s| s.as_u64()),
            };

            if let Err(e) = event_sender.send(event) {
                debug!("📡 No active receivers for WebSocket event: {}", e);
            }
        }

        Ok(())
    }

    /// Send subscribe message
    async fn send_subscribe_message(
        ws_sender: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
        subscription_type: &SubscriptionType,
    ) -> Result<()> {
        let message = match subscription_type {
            SubscriptionType::AccountUpdate(pubkey) => {
                json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "accountSubscribe",
                    "params": [pubkey.to_string(), {"commitment": "confirmed"}]
                })
            }
            SubscriptionType::ProgramUpdate(program_id) => {
                json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "programSubscribe",
                    "params": [program_id.to_string(), {"commitment": "confirmed"}]
                })
            }
            SubscriptionType::SlotUpdate => {
                json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "slotSubscribe"
                })
            }
            SubscriptionType::RootUpdate => {
                json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "rootSubscribe"
                })
            }
            SubscriptionType::LogsUpdate(filter) => {
                json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "logsSubscribe",
                    "params": [{"mentions": [filter]}, {"commitment": "confirmed"}]
                })
            }
        };

        ws_sender
            .send(Message::Text(message.to_string().into()))
            .await?;
        Ok(())
    }

    /// Send unsubscribe message
    async fn send_unsubscribe_message(
        ws_sender: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
        subscription_id: u64,
    ) -> Result<()> {
        let message = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "accountUnsubscribe",
            "params": [subscription_id]
        });
        ws_sender
            .send(Message::Text(message.to_string().into()))
            .await?;
        Ok(())
    }
}

/// Utility functions for common WebSocket patterns
impl WebSocketManager {
    /// Monitor a specific account for balance changes
    pub async fn monitor_account_balance(
        &self,
        pubkey: Pubkey,
    ) -> Result<broadcast::Receiver<f64>> {
        let _subscription_id = self.subscribe_account(pubkey).await?;
        let mut event_receiver = self.get_event_receiver();
        let (balance_sender, balance_receiver) = broadcast::channel(100);

        tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                if event.subscription_type == "accountNotification" {
                    if let Some(lamports) = event
                        .data
                        .get("params")
                        .and_then(|p| p.get("result"))
                        .and_then(|r| r.get("value"))
                        .and_then(|v| v.get("lamports"))
                        .and_then(|l| l.as_u64())
                    {
                        let sol_balance = lamports as f64 / 1_000_000_000.0;
                        let _ = balance_sender.send(sol_balance);
                    }
                }
            }
        });

        Ok(balance_receiver)
    }

    /// Monitor slot updates for timing trades
    pub async fn monitor_slots(&self) -> Result<broadcast::Receiver<u64>> {
        let _subscription_id = self.subscribe_slots().await?;
        let mut event_receiver = self.get_event_receiver();
        let (slot_sender, slot_receiver) = broadcast::channel(100);

        tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                if event.subscription_type == "slotNotification" {
                    if let Some(slot) = event.slot {
                        let _ = slot_sender.send(slot);
                    }
                }
            }
        });

        Ok(slot_receiver)
    }
}
