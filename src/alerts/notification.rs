use crate::alerts::{AlertError, AlertResult};
use crate::models::{PriceAlert, User};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Notification service for sending alerts to users
pub struct NotificationService {
    /// Notification queue
    notification_queue: Arc<Mutex<VecDeque<Notification>>>,
    /// Notification history
    notification_history: Arc<Mutex<Vec<Notification>>>,
    /// Service configuration
    config: NotificationConfig,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            notification_queue: Arc::new(Mutex::new(VecDeque::new())),
            notification_history: Arc::new(Mutex::new(Vec::new())),
            config: NotificationConfig::default(),
        }
    }

    /// Send a price alert notification
    pub fn send_price_alert(
        &self,
        user: &User,
        alert: &PriceAlert,
        current_price: f64,
    ) -> AlertResult<()> {
        let notification = Notification {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            notification_type: NotificationType::PriceAlert,
            title: "Price Alert: Target Reached!".to_string(),
            message: format!(
                "Your price alert for product {} has been triggered! Current price: ¥{:.2}, Target: ¥{:.2}",
                alert.product_id, current_price, alert.target_price
            ),
            data: Some(serde_json::json!({
                "alert_id": alert.id,
                "product_id": alert.product_id,
                "current_price": current_price,
                "target_price": alert.target_price
            })),
            created_at: Utc::now(),
            sent_at: None,
            read_at: None,
            status: NotificationStatus::Pending,
        };

        self.queue_notification(notification)
    }

    /// Send a general notification
    pub fn send_notification(
        &self,
        user_id: &str,
        notification_type: NotificationType,
        title: String,
        message: String,
        data: Option<serde_json::Value>,
    ) -> AlertResult<()> {
        let notification = Notification {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            notification_type,
            title,
            message,
            data,
            created_at: Utc::now(),
            sent_at: None,
            read_at: None,
            status: NotificationStatus::Pending,
        };

        self.queue_notification(notification)
    }

    /// Queue a notification for sending
    fn queue_notification(&self, notification: Notification) -> AlertResult<()> {
        let mut queue = self.notification_queue.lock().map_err(|e| {
            AlertError::NotificationFailed(format!("Failed to acquire queue lock: {}", e))
        })?;

        log::info!("Queueing notification: {}", notification.title);
        queue.push_back(notification);

        // Process the queue
        self.process_queue()?;

        Ok(())
    }

    /// Process pending notifications
    fn process_queue(&self) -> AlertResult<()> {
        let mut queue = self.notification_queue.lock().map_err(|e| {
            AlertError::NotificationFailed(format!("Failed to acquire queue lock: {}", e))
        })?;

        let mut history = self.notification_history.lock().map_err(|e| {
            AlertError::NotificationFailed(format!("Failed to acquire history lock: {}", e))
        })?;

        while let Some(mut notification) = queue.pop_front() {
            match self.send_notification_internal(&mut notification) {
                Ok(_) => {
                    notification.status = NotificationStatus::Sent;
                    notification.sent_at = Some(Utc::now());
                    log::info!("Successfully sent notification: {}", notification.id);
                }
                Err(e) => {
                    notification.status = NotificationStatus::Failed;
                    log::error!("Failed to send notification {}: {}", notification.id, e);
                }
            }

            history.push(notification);
        }

        Ok(())
    }

    /// Internal notification sending implementation
    fn send_notification_internal(
        &self,
        notification: &mut Notification,
    ) -> Result<(), AlertError> {
        match notification.notification_type {
            NotificationType::PriceAlert => self.send_price_alert_internal(notification),
            NotificationType::SystemAlert => self.send_system_alert_internal(notification),
            NotificationType::ProductUpdate => self.send_product_update_internal(notification),
            NotificationType::UserMessage => self.send_user_message_internal(notification),
        }
    }

    /// Send price alert notification
    fn send_price_alert_internal(&self, notification: &Notification) -> Result<(), AlertError> {
        // In a real implementation, this would send email, push notification, etc.
        log::info!("Price Alert Notification: {}", notification.message);

        // Simulate sending via different channels
        if self.config.email_enabled {
            self.send_email_notification(notification)?;
        }

        if self.config.push_enabled {
            self.send_push_notification(notification)?;
        }

        if self.config.in_app_enabled {
            self.send_in_app_notification(notification)?;
        }

        Ok(())
    }

    /// Send system alert notification
    fn send_system_alert_internal(&self, notification: &Notification) -> Result<(), AlertError> {
        log::info!("System Alert: {}", notification.message);

        // System alerts are typically high priority
        if self.config.push_enabled {
            self.send_push_notification(notification)?;
        }

        self.send_in_app_notification(notification)?;

        Ok(())
    }

    /// Send product update notification
    fn send_product_update_internal(&self, notification: &Notification) -> Result<(), AlertError> {
        log::info!("Product Update: {}", notification.message);

        // Product updates are usually lower priority
        if self.config.in_app_enabled {
            self.send_in_app_notification(notification)?;
        }

        Ok(())
    }

    /// Send user message notification
    fn send_user_message_internal(&self, notification: &Notification) -> Result<(), AlertError> {
        log::info!("User Message: {}", notification.message);

        if self.config.email_enabled {
            self.send_email_notification(notification)?;
        }

        if self.config.in_app_enabled {
            self.send_in_app_notification(notification)?;
        }

        Ok(())
    }

    /// Mock email notification sending
    fn send_email_notification(&self, notification: &Notification) -> Result<(), AlertError> {
        // Mock implementation - in real app would integrate with email service
        log::info!(
            "📧 Email sent to user {}: {}",
            notification.user_id,
            notification.title
        );
        std::thread::sleep(std::time::Duration::from_millis(100)); // Simulate network delay
        Ok(())
    }

    /// Mock push notification sending
    fn send_push_notification(&self, notification: &Notification) -> Result<(), AlertError> {
        // Mock implementation - in real app would integrate with push service
        log::info!(
            "📱 Push notification sent to user {}: {}",
            notification.user_id,
            notification.title
        );
        std::thread::sleep(std::time::Duration::from_millis(50)); // Simulate network delay
        Ok(())
    }

    /// Mock in-app notification
    fn send_in_app_notification(&self, notification: &Notification) -> Result<(), AlertError> {
        // Mock implementation - in real app would store in local notification center
        log::info!(
            "🔔 In-app notification for user {}: {}",
            notification.user_id,
            notification.title
        );
        Ok(())
    }

    /// Get notification history for a user
    pub fn get_user_notifications(&self, user_id: &str) -> AlertResult<Vec<Notification>> {
        let history = self.notification_history.lock().map_err(|e| {
            AlertError::NotificationFailed(format!("Failed to acquire history lock: {}", e))
        })?;

        let user_notifications: Vec<Notification> = history
            .iter()
            .filter(|n| n.user_id == user_id)
            .cloned()
            .collect();

        Ok(user_notifications)
    }

    /// Mark notification as read
    pub fn mark_as_read(&self, notification_id: &str) -> AlertResult<()> {
        let mut history = self.notification_history.lock().map_err(|e| {
            AlertError::NotificationFailed(format!("Failed to acquire history lock: {}", e))
        })?;

        if let Some(notification) = history.iter_mut().find(|n| n.id == notification_id) {
            notification.read_at = Some(Utc::now());
            log::info!("Marked notification {} as read", notification_id);
            Ok(())
        } else {
            Err(AlertError::NotificationFailed(format!(
                "Notification {} not found",
                notification_id
            )))
        }
    }

    /// Get unread notification count for a user
    pub fn get_unread_count(&self, user_id: &str) -> AlertResult<usize> {
        let history = self.notification_history.lock().map_err(|e| {
            AlertError::NotificationFailed(format!("Failed to acquire history lock: {}", e))
        })?;

        let count = history
            .iter()
            .filter(|n| n.user_id == user_id && n.read_at.is_none())
            .count();

        Ok(count)
    }

    /// Clear old notifications
    pub fn clear_old_notifications(&self, days: i64) -> AlertResult<usize> {
        let mut history = self.notification_history.lock().map_err(|e| {
            AlertError::NotificationFailed(format!("Failed to acquire history lock: {}", e))
        })?;

        let cutoff_date = Utc::now() - chrono::Duration::days(days);
        let initial_count = history.len();

        history.retain(|n| n.created_at > cutoff_date);

        let removed_count = initial_count - history.len();
        log::info!("Cleared {} old notifications", removed_count);

        Ok(removed_count)
    }

    /// Update notification configuration
    pub fn update_config(&mut self, config: NotificationConfig) {
        self.config = config;
        log::info!("Updated notification configuration");
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

/// Notification configuration
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub in_app_enabled: bool,
    pub max_notifications_per_day: usize,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            email_enabled: true,
            push_enabled: true,
            in_app_enabled: true,
            max_notifications_per_day: 50,
        }
    }
}

/// Individual notification
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub status: NotificationStatus,
}

/// Types of notifications
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    PriceAlert,
    SystemAlert,
    ProductUpdate,
    UserMessage,
}

/// Notification status
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Read,
}
