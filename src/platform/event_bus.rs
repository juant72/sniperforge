use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::types::PlatformError;

/// Event types that can be published and subscribed to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    PoolCreated,
    PoolUpdated,
    PriceAlert,
    TradingOpportunity,
    BotStatusChange,
    SystemMetrics,
    Error,
    Custom(String),
}

/// Platform event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformEvent {
    pub id: Uuid,
    pub event_type: EventType,
    pub source: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub priority: EventPriority,
}

/// Event priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Event subscriber information
#[derive(Debug, Clone)]
pub struct EventSubscriber {
    pub id: Uuid,
    pub name: String,
    pub event_types: Vec<EventType>,
    pub sender: broadcast::Sender<PlatformEvent>,
}

/// Event bus for platform-wide event handling
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<Uuid, EventSubscriber>>>,
    event_buffer: Arc<RwLock<Vec<PlatformEvent>>>,
    buffer_size: usize,
    shutdown_tx: mpsc::Sender<()>,
}

impl EventBus {
    pub fn new(buffer_size: Option<usize>) -> Self {
        let (shutdown_tx, _) = mpsc::channel(1);

        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            event_buffer: Arc::new(RwLock::new(Vec::new())),
            buffer_size: buffer_size.unwrap_or(1000),
            shutdown_tx,
        }
    }
    /// Start the event bus
    pub async fn start(&self) -> Result<()> {
        info!("Starting Event Bus");

        // Start event processing loop
        // TODO: Implement event loop with interior mutability
        // self.run_event_loop().await?;

        Ok(())
    }

    /// Stop the event bus
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Event Bus");

        // Send shutdown signal
        let _ = self.shutdown_tx.send(()).await;

        // Clear subscribers
        let mut subscribers = self.subscribers.write().await;
        subscribers.clear();

        Ok(())
    }

    /// Subscribe to specific event types
    pub async fn subscribe(
        &self,
        subscriber_name: String,
        event_types: Vec<EventType>,
        buffer_size: Option<usize>,
    ) -> Result<(Uuid, broadcast::Receiver<PlatformEvent>)> {
        let subscriber_id = Uuid::new_v4();
        let (tx, rx) = broadcast::channel(buffer_size.unwrap_or(100));

        let subscriber = EventSubscriber {
            id: subscriber_id,
            name: subscriber_name.clone(),
            event_types,
            sender: tx,
        };

        let mut subscribers = self.subscribers.write().await;
        subscribers.insert(subscriber_id, subscriber);

        info!(
            "Registered event subscriber: {} ({})",
            subscriber_name, subscriber_id
        );
        Ok((subscriber_id, rx))
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscriber_id: Uuid) -> Result<()> {
        let mut subscribers = self.subscribers.write().await;

        if let Some(subscriber) = subscribers.remove(&subscriber_id) {
            info!(
                "Unregistered event subscriber: {} ({})",
                subscriber.name, subscriber_id
            );
            Ok(())
        } else {
            Err(PlatformError::EventBus("Subscriber not found".to_string()).into())
        }
    }

    /// Publish an event to all relevant subscribers
    pub async fn publish(&self, event: PlatformEvent) -> Result<()> {
        debug!(
            "Publishing event: {:?} from {}",
            event.event_type, event.source
        );

        // Add to buffer for persistence
        self.add_to_buffer(event.clone()).await;

        // Send to subscribers
        let subscribers = self.subscribers.read().await;
        let mut delivery_count = 0;

        for subscriber in subscribers.values() {
            // Check if subscriber is interested in this event type
            if self.subscriber_interested(&subscriber.event_types, &event.event_type) {
                match subscriber.sender.send(event.clone()) {
                    Ok(_) => {
                        delivery_count += 1;
                        debug!("Event delivered to subscriber: {}", subscriber.name);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to deliver event to subscriber {}: {}",
                            subscriber.name, e
                        );
                    }
                }
            }
        }

        debug!("Event delivered to {} subscribers", delivery_count);
        Ok(())
    }

    /// Publish a simple event with minimal data
    pub async fn publish_simple(
        &self,
        event_type: EventType,
        source: String,
        data: serde_json::Value,
        priority: Option<EventPriority>,
    ) -> Result<()> {
        let event = PlatformEvent {
            id: Uuid::new_v4(),
            event_type,
            source,
            data,
            timestamp: chrono::Utc::now(),
            priority: priority.unwrap_or(EventPriority::Normal),
        };

        self.publish(event).await
    }

    /// Get recent events from buffer
    pub async fn get_recent_events(&self, limit: Option<usize>) -> Vec<PlatformEvent> {
        let buffer = self.event_buffer.read().await;
        let limit = limit.unwrap_or(100).min(buffer.len());

        buffer.iter().rev().take(limit).cloned().collect()
    }

    /// Get events by type from buffer
    pub async fn get_events_by_type(
        &self,
        event_type: EventType,
        limit: Option<usize>,
    ) -> Vec<PlatformEvent> {
        let buffer = self.event_buffer.read().await;
        let limit = limit.unwrap_or(100);

        buffer
            .iter()
            .rev()
            .filter(|event| {
                std::mem::discriminant(&event.event_type) == std::mem::discriminant(&event_type)
            })
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get subscriber information
    pub async fn get_subscribers(&self) -> Vec<(Uuid, String, Vec<EventType>)> {
        let subscribers = self.subscribers.read().await;
        subscribers
            .values()
            .map(|sub| (sub.id, sub.name.clone(), sub.event_types.clone()))
            .collect()
    }

    /// Check if subscriber is interested in event type
    fn subscriber_interested(
        &self,
        subscriber_types: &[EventType],
        event_type: &EventType,
    ) -> bool {
        subscriber_types.iter().any(|sub_type| {
            match (sub_type, event_type) {
                // Exact match
                (a, b) if std::mem::discriminant(a) == std::mem::discriminant(b) => true,
                // Custom events match by string
                (EventType::Custom(a), EventType::Custom(b)) => a == b,
                _ => false,
            }
        })
    }

    /// Add event to buffer with size management
    async fn add_to_buffer(&self, event: PlatformEvent) {
        let mut buffer = self.event_buffer.write().await;

        buffer.push(event);

        // Keep buffer size under limit
        if buffer.len() > self.buffer_size {
            let remove_count = buffer.len() - self.buffer_size;
            buffer.drain(0..remove_count);
        }
    }

    /// Main event processing loop
    async fn run_event_loop(&mut self) -> Result<()> {
        // For now, this is primarily passive
        // In the future, we could add event processing, filtering, or aggregation here

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;
                // Periodic cleanup or maintenance tasks
                debug!("Event bus heartbeat");
            }
        });

        Ok(())
    }

    /// Get event statistics
    pub async fn get_stats(&self) -> EventBusStats {
        let subscribers = self.subscribers.read().await;
        let buffer = self.event_buffer.read().await;

        EventBusStats {
            subscriber_count: subscribers.len(),
            buffered_events: buffer.len(),
            buffer_capacity: self.buffer_size,
        }
    }
}

/// Event bus statistics
#[derive(Debug, Serialize)]
pub struct EventBusStats {
    pub subscriber_count: usize,
    pub buffered_events: usize,
    pub buffer_capacity: usize,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(None)
    }
}

/// Helper macro for creating events
#[macro_export]
macro_rules! create_event {
    ($event_type:expr, $source:expr, $data:expr) => {
        PlatformEvent {
            id: uuid::Uuid::new_v4(),
            event_type: $event_type,
            source: $source.to_string(),
            data: serde_json::to_value($data).unwrap_or(serde_json::Value::Null),
            timestamp: chrono::Utc::now(),
            priority: EventPriority::Normal,
        }
    };
    ($event_type:expr, $source:expr, $data:expr, $priority:expr) => {
        PlatformEvent {
            id: uuid::Uuid::new_v4(),
            event_type: $event_type,
            source: $source.to_string(),
            data: serde_json::to_value($data).unwrap_or(serde_json::Value::Null),
            timestamp: chrono::Utc::now(),
            priority: $priority,
        }
    };
}
