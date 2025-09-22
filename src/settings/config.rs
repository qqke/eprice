use serde::{Deserialize, Serialize};

/// Application configuration settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub ui_settings: UISettings,
    pub notification_settings: NotificationSettings,
    pub monitoring_settings: MonitoringSettings,
    pub data_settings: DataSettings,
}

/// UI display and interaction settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    pub theme: String,    // "light", "dark", "auto"
    pub language: String, // "zh", "en", "auto"
    pub font_size: f32,
    pub show_animations: bool,
    pub compact_mode: bool,
    pub window_transparency: f32,
}

/// Notification and alert settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub enable_notifications: bool,
    pub enable_sound: bool,
    pub enable_popup: bool,
    pub notification_frequency_minutes: u32,
    pub price_drop_threshold: f64, // Percentage
    pub show_promotion_alerts: bool,
}

/// Price monitoring settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    pub enable_auto_monitoring: bool,
    pub monitoring_interval_minutes: u32,
    pub max_price_records_per_product: u32,
    pub enable_trend_analysis: bool,
    pub price_history_days: u32,
}

/// Data storage and sync settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSettings {
    pub auto_backup: bool,
    pub backup_frequency_hours: u32,
    pub max_backup_files: u32,
    pub enable_cloud_sync: bool,
    pub data_retention_days: u32,
}

// (removed duplicate AppConfig redefinition)

impl Default for UISettings {
    fn default() -> Self {
        Self {
            theme: "auto".to_string(),
            language: "zh".to_string(),
            font_size: 14.0,
            show_animations: true,
            compact_mode: false,
            window_transparency: 1.0,
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enable_notifications: true,
            enable_sound: true,
            enable_popup: true,
            notification_frequency_minutes: 60,
            price_drop_threshold: 5.0,
            show_promotion_alerts: true,
        }
    }
}

impl Default for MonitoringSettings {
    fn default() -> Self {
        Self {
            enable_auto_monitoring: true,
            monitoring_interval_minutes: 30,
            max_price_records_per_product: 100,
            enable_trend_analysis: true,
            price_history_days: 30,
        }
    }
}

impl Default for DataSettings {
    fn default() -> Self {
        Self {
            auto_backup: true,
            backup_frequency_hours: 24,
            max_backup_files: 7,
            enable_cloud_sync: false,
            data_retention_days: 365,
        }
    }
}

impl AppConfig {
    /// Load configuration from file
    pub fn load() -> std::io::Result<Self> {
        // In a real implementation, load from config file
        Ok(Self::default())
    }

    /// Save configuration to file
    pub fn save(&self) -> std::io::Result<()> {
        // In a real implementation, save to config file
        Ok(())
    }

    /// Reset to default settings
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }

    /// Validate configuration settings
    pub fn validate(&self) -> Result<(), String> {
        // Validate UI settings
        if self.ui_settings.font_size < 8.0 || self.ui_settings.font_size > 32.0 {
            return Err("Font size must be between 8 and 32".to_string());
        }

        if self.ui_settings.window_transparency < 0.1 || self.ui_settings.window_transparency > 1.0
        {
            return Err("Window transparency must be between 0.1 and 1.0".to_string());
        }

        // Validate notification settings
        if self.notification_settings.notification_frequency_minutes < 1 {
            return Err("Notification frequency must be at least 1 minute".to_string());
        }

        if self.notification_settings.price_drop_threshold < 0.1
            || self.notification_settings.price_drop_threshold > 50.0
        {
            return Err("Price drop threshold must be between 0.1% and 50%".to_string());
        }

        // Validate monitoring settings
        if self.monitoring_settings.monitoring_interval_minutes < 5 {
            return Err("Monitoring interval must be at least 5 minutes".to_string());
        }

        if self.monitoring_settings.max_price_records_per_product < 10 {
            return Err("Maximum price records must be at least 10".to_string());
        }

        // Validate data settings
        if self.data_settings.backup_frequency_hours < 1 {
            return Err("Backup frequency must be at least 1 hour".to_string());
        }

        if self.data_settings.max_backup_files < 1 {
            return Err("Must keep at least 1 backup file".to_string());
        }

        Ok(())
    }
}
