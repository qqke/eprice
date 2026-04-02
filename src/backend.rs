use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SupabaseConfig {
    pub url: String,
    pub anon_key: String,
    pub project_ref: String,
}

impl SupabaseConfig {
    pub fn from_env() -> Self {
        let url = env::var("SUPABASE_URL").unwrap_or_default();
        let anon_key = env::var("SUPABASE_ANON_KEY").unwrap_or_default();
        let project_ref = env::var("SUPABASE_PROJECT_REF").unwrap_or_default();

        Self {
            url,
            anon_key,
            project_ref,
        }
    }

    pub fn from_settings(settings: &crate::settings::config::BackendSettings) -> Self {
        Self {
            url: settings.supabase_url.clone(),
            anon_key: settings.supabase_anon_key.clone(),
            project_ref: settings.supabase_project_ref.clone(),
        }
    }

    pub fn from_env_or_settings(settings: &crate::settings::config::BackendSettings) -> Self {
        let env_config = Self::from_env();
        if env_config.is_configured() {
            env_config
        } else {
            Self::from_settings(settings)
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.url.trim().is_empty() && !self.anon_key.trim().is_empty()
    }

    pub fn status_label(&self) -> &'static str {
        if self.is_configured() {
            "Supabase connected"
        } else {
            "Supabase not configured"
        }
    }

    pub fn missing_fields(&self) -> Vec<String> {
        let mut missing = Vec::new();
        if self.url.trim().is_empty() {
            missing.push("SUPABASE_URL".to_string());
        }
        if self.anon_key.trim().is_empty() {
            missing.push("SUPABASE_ANON_KEY".to_string());
        }
        if self.project_ref.trim().is_empty() {
            missing.push("SUPABASE_PROJECT_REF".to_string());
        }
        missing
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendHealth {
    Ready,
    MissingConfig(Vec<String>),
}

impl BackendHealth {
    pub fn from_config(config: &SupabaseConfig) -> Self {
        if config.is_configured() {
            BackendHealth::Ready
        } else {
            BackendHealth::MissingConfig(config.missing_fields())
        }
    }

    pub fn label(&self) -> String {
        match self {
            BackendHealth::Ready => "Supabase ready".to_string(),
            BackendHealth::MissingConfig(fields) => {
                format!("Missing config: {}", fields.join(", "))
            }
        }
    }
}
