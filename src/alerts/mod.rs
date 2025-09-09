pub mod monitor;
pub mod notification;
// pub mod ui; // TODO: Fix string encoding issues

pub use monitor::{MonitoringResult, PriceMonitor};
pub use notification::{Notification, NotificationService, NotificationType};
// pub use ui::AlertUI; // TODO: Fix string encoding issues

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AlertError {
    #[error("Price monitoring failed: {0}")]
    MonitoringFailed(String),
    #[error("Notification failed: {0}")]
    NotificationFailed(String),
    #[error("Alert not found: {0}")]
    AlertNotFound(String),
    #[error("Invalid price threshold: {0}")]
    InvalidThreshold(f64),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

pub type AlertResult<T> = Result<T, AlertError>;

/// Main alert service that coordinates monitoring and notifications
pub struct AlertService {
    monitor: PriceMonitor,
    notification_service: NotificationService,
}

impl AlertService {
    pub fn new() -> Self {
        Self {
            monitor: PriceMonitor::new(),
            notification_service: NotificationService::new(),
        }
    }

    /// Start the price monitoring service
    pub fn start_monitoring(&mut self) -> AlertResult<()> {
        self.monitor.start()
    }

    /// Stop the price monitoring service
    pub fn stop_monitoring(&mut self) -> AlertResult<()> {
        self.monitor.stop()
    }

    /// Add a new price alert
    pub fn add_alert(&mut self, alert: crate::models::PriceAlert) -> AlertResult<()> {
        self.monitor.add_alert(alert)
    }

    /// Remove a price alert
    pub fn remove_alert(&mut self, alert_id: &str) -> AlertResult<()> {
        self.monitor.remove_alert(alert_id)
    }

    /// Check if monitoring is active
    pub fn is_monitoring(&self) -> bool {
        self.monitor.is_running()
    }

    /// Get all active alerts for a user
    pub fn get_user_alerts(&self, user_id: &str) -> AlertResult<Vec<crate::models::PriceAlert>> {
        self.monitor.get_user_alerts(user_id)
    }

    /// Force check all alerts (for testing)
    pub fn check_alerts(&mut self) -> AlertResult<Vec<MonitoringResult>> {
        self.monitor.check_all_alerts()
    }

    /// Access to individual components
    pub fn monitor(&self) -> &PriceMonitor {
        &self.monitor
    }

    pub fn monitor_mut(&mut self) -> &mut PriceMonitor {
        &mut self.monitor
    }

    pub fn notification_service(&self) -> &NotificationService {
        &self.notification_service
    }

    pub fn notification_service_mut(&mut self) -> &mut NotificationService {
        &mut self.notification_service
    }
}

impl Default for AlertService {
    fn default() -> Self {
        Self::new()
    }
}
