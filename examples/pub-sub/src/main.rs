//! Pub/Sub Messaging Example
//! 
//! This example demonstrates how to use shared-redis for real-time messaging
//! between microservices using Redis pub/sub capabilities.

use shared_redis::operations::{broadcasting_data, subscribe_data};
use serde::{Deserialize, Serialize};
use log::{info, warn, error};
use futures::StreamExt;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserEvent {
    user_id: u32,
    event_type: String,
    data: serde_json::Value,
    timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct NotificationMessage {
    recipient_id: u32,
    message: String,
    priority: String,
    created_at: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    info!("Starting shared-redis pub/sub messaging example");
    
    // Set up Redis connection
    std::env::set_var("REDIS_URL", "redis://localhost:6379");
    
    // Spawn publisher task
    let publisher_handle = tokio::spawn(publisher_task());
    
    // Spawn subscriber tasks
    let user_events_subscriber = tokio::spawn(subscribe_to_user_events());
    let notifications_subscriber = tokio::spawn(subscribe_to_notifications());
    
    // Wait for all tasks to complete
    let (publisher_result, user_events_result, notifications_result) = tokio::join!(
        publisher_handle,
        user_events_subscriber,
        notifications_subscriber
    );
    
    // Check results
    if let Err(e) = publisher_result {
        error!("Publisher task failed: {}", e);
    }
    
    if let Err(e) = user_events_result {
        error!("User events subscriber failed: {}", e);
    }
    
    if let Err(e) = notifications_result {
        error!("Notifications subscriber failed: {}", e);
    }
    
    info!("Pub/sub messaging example completed");
    Ok(())
}

async fn publisher_task() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting publisher task");
    
    // Publish user events
    for i in 1..=5 {
        let user_event = UserEvent {
            user_id: i,
            event_type: "login".to_string(),
            data: serde_json::json!({
                "ip_address": "192.168.1.100",
                "user_agent": "Mozilla/5.0..."
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let event_json = serde_json::to_string(&user_event)?;
        
        match broadcasting_data("user_events".to_string(), event_json).await {
            Ok(_) => info!("Published user event for user {}", i),
            Err(e) => warn!("Failed to publish user event: {}", e),
        }
        
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    // Publish notifications
    for i in 1..=3 {
        let notification = NotificationMessage {
            recipient_id: i,
            message: format!("Welcome back, user {}!", i),
            priority: "normal".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        let notification_json = serde_json::to_string(&notification)?;
        
        match broadcasting_data("notifications".to_string(), notification_json).await {
            Ok(_) => info!("Published notification for user {}", i),
            Err(e) => warn!("Failed to publish notification: {}", e),
        }
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    info!("Publisher task completed");
    Ok(())
}

async fn subscribe_to_user_events() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting user events subscriber");
    
    let mut pubsub = subscribe_data("user_events".to_string()).await?;
    
    let mut message_count = 0;
    while let Some(msg) = pubsub.on_message().next().await {
        message_count += 1;
        
        match msg.get_payload::<String>() {
            Ok(payload) => {
                match serde_json::from_str::<UserEvent>(&payload) {
                    Ok(event) => {
                        info!("Received user event: {:?}", event);
                        
                        // Process the event (e.g., update analytics, send notifications)
                        process_user_event(&event).await;
                    }
                    Err(e) => {
                        warn!("Failed to deserialize user event: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get message payload: {}", e);
            }
        }
        
        // Stop after receiving 5 messages
        if message_count >= 5 {
            break;
        }
    }
    
    info!("User events subscriber completed");
    Ok(())
}

async fn subscribe_to_notifications() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting notifications subscriber");
    
    let mut pubsub = subscribe_data("notifications".to_string()).await?;
    
    let mut message_count = 0;
    while let Some(msg) = pubsub.on_message().next().await {
        message_count += 1;
        
        match msg.get_payload::<String>() {
            Ok(payload) => {
                match serde_json::from_str::<NotificationMessage>(&payload) {
                    Ok(notification) => {
                        info!("Received notification: {:?}", notification);
                        
                        // Process the notification (e.g., send email, push notification)
                        process_notification(&notification).await;
                    }
                    Err(e) => {
                        warn!("Failed to deserialize notification: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get message payload: {}", e);
            }
        }
        
        // Stop after receiving 3 messages
        if message_count >= 3 {
            break;
        }
    }
    
    info!("Notifications subscriber completed");
    Ok(())
}

async fn process_user_event(event: &UserEvent) {
    info!("Processing user event for user {}: {}", event.user_id, event.event_type);
    
    // Simulate processing time
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Example processing logic:
    // - Update user session
    // - Log analytics
    // - Trigger notifications
    // - Update user activity
    
    match event.event_type.as_str() {
        "login" => {
            info!("User {} logged in from IP: {}", 
                  event.user_id, 
                  event.data["ip_address"].as_str().unwrap_or("unknown"));
        }
        "logout" => {
            info!("User {} logged out", event.user_id);
        }
        "purchase" => {
            info!("User {} made a purchase", event.user_id);
        }
        _ => {
            info!("Unknown event type: {}", event.event_type);
        }
    }
}

async fn process_notification(notification: &NotificationMessage) {
    info!("Processing notification for user {}: {}", 
          notification.recipient_id, 
          notification.message);
    
    // Simulate processing time
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Example processing logic:
    // - Send email notification
    // - Send push notification
    // - Update notification history
    // - Track delivery status
    
    match notification.priority.as_str() {
        "high" => {
            info!("Sending high priority notification to user {}", notification.recipient_id);
        }
        "normal" => {
            info!("Sending normal priority notification to user {}", notification.recipient_id);
        }
        "low" => {
            info!("Sending low priority notification to user {}", notification.recipient_id);
        }
        _ => {
            warn!("Unknown notification priority: {}", notification.priority);
        }
    }
}
