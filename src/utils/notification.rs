use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

/// Notification types
#[derive(Debug, Clone)]
pub enum NotificationType {
    PriceAlert,
    SystemInfo,
    Warning,
    Error,
}

/// Notification structure
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub created_at: DateTime<Utc>,
    pub is_read: bool,
}

impl Notification {
    /// Create a new notification
    pub fn new(title: String, message: String, notification_type: NotificationType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            message,
            notification_type,
            created_at: Utc::now(),
            is_read: false,
        }
    }

    /// Mark notification as read
    pub fn mark_as_read(&mut self) {
        self.is_read = true;
    }
}

/// Notification service for managing in-app notifications
pub struct NotificationService {
    notifications: VecDeque<Notification>,
    max_notifications: usize,
}

impl NotificationService {
    /// Create a new notification service
    pub fn new(max_notifications: usize) -> Self {
        Self {
            notifications: VecDeque::new(),
            max_notifications,
        }
    }

    /// Add a new notification
    pub fn add_notification(&mut self, notification: Notification) {
        self.notifications.push_front(notification);

        // Keep only the most recent notifications
        while self.notifications.len() > self.max_notifications {
            self.notifications.pop_back();
        }
    }

    /// Add a price alert notification
    pub fn add_price_alert(&mut self, product_name: &str, current_price: f64, target_price: f64) {
        let title = "Price Alert!".to_string();
        let message = format!(
            "{} is now ¥{:.2} (target: ¥{:.2})",
            product_name, current_price, target_price
        );

        let notification = Notification::new(title, message, NotificationType::PriceAlert);
        self.add_notification(notification);
    }

    /// Add a system info notification
    pub fn add_info(&mut self, title: String, message: String) {
        let notification = Notification::new(title, message, NotificationType::SystemInfo);
        self.add_notification(notification);
    }

    /// Add a warning notification
    pub fn add_warning(&mut self, title: String, message: String) {
        let notification = Notification::new(title, message, NotificationType::Warning);
        self.add_notification(notification);
    }

    /// Add an error notification
    pub fn add_error(&mut self, title: String, message: String) {
        let notification = Notification::new(title, message, NotificationType::Error);
        self.add_notification(notification);
    }

    /// Get all notifications
    pub fn get_notifications(&self) -> &VecDeque<Notification> {
        &self.notifications
    }

    /// Get unread notifications
    pub fn get_unread_notifications(&self) -> Vec<&Notification> {
        self.notifications.iter().filter(|n| !n.is_read).collect()
    }

    /// Mark notification as read by ID
    pub fn mark_as_read(&mut self, notification_id: &str) -> Result<()> {
        if let Some(notification) = self
            .notifications
            .iter_mut()
            .find(|n| n.id == notification_id)
        {
            notification.mark_as_read();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Notification not found"))
        }
    }

    /// Mark all notifications as read
    pub fn mark_all_as_read(&mut self) {
        for notification in &mut self.notifications {
            notification.mark_as_read();
        }
    }

    /// Clear all notifications
    pub fn clear_all(&mut self) {
        self.notifications.clear();
    }

    /// Get notification count
    pub fn get_notification_count(&self) -> usize {
        self.notifications.len()
    }

    /// Get unread notification count
    pub fn get_unread_count(&self) -> usize {
        self.notifications.iter().filter(|n| !n.is_read).count()
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new(100) // Default to keeping 100 notifications
    }
}
