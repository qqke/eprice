use crate::auth::models::User;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

/// User session information
#[derive(Debug, Clone)]
pub struct UserSession {
    pub user: User,
    pub login_time: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub remember_me: bool,
}

impl UserSession {
    /// Create a new user session
    pub fn new(user: User, remember_me: bool) -> Self {
        let now = Utc::now();
        Self {
            user,
            login_time: now,
            last_activity: now,
            remember_me,
        }
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let session_duration = if self.remember_me {
            Duration::days(30) // 30 days for "remember me"
        } else {
            Duration::hours(24) // 24 hours for regular session
        };

        now.signed_duration_since(self.last_activity) > session_duration
    }

    /// Get session duration in minutes
    pub fn session_duration_minutes(&self) -> i64 {
        let now = Utc::now();
        now.signed_duration_since(self.login_time).num_minutes()
    }
}

/// Session manager for handling user sessions
pub struct SessionManager {
    sessions: HashMap<String, UserSession>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Create a new session for a user
    pub fn create_session(&mut self, user: User, remember_me: bool) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = UserSession::new(user, remember_me);
        self.sessions.insert(session_id.clone(), session);

        log::info!("Session created: {}", session_id);
        session_id
    }

    /// Get user session by session ID
    pub fn get_session(&self, session_id: &str) -> Option<&UserSession> {
        self.sessions.get(session_id)
    }

    /// Get mutable user session by session ID
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut UserSession> {
        self.sessions.get_mut(session_id)
    }

    /// Update session activity
    pub fn update_session_activity(&mut self, session_id: &str) -> bool {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.update_activity();
            true
        } else {
            false
        }
    }

    /// Validate session and return user if valid
    pub fn validate_session(&mut self, session_id: &str) -> Option<&User> {
        let is_expired = if let Some(session) = self.sessions.get(session_id) {
            session.is_expired()
        } else {
            return None;
        };

        if is_expired {
            self.sessions.remove(session_id);
            log::info!("Session expired and removed: {}", session_id);
            None
        } else if let Some(session) = self.sessions.get_mut(session_id) {
            session.update_activity();
            Some(&session.user)
        } else {
            None
        }
    }

    /// Remove a session (logout)
    pub fn remove_session(&mut self, session_id: &str) -> bool {
        if self.sessions.remove(session_id).is_some() {
            log::info!("Session removed: {}", session_id);
            true
        } else {
            false
        }
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        let expired_sessions: Vec<String> = self
            .sessions
            .iter()
            .filter(|(_, session)| session.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        for session_id in expired_sessions {
            self.sessions.remove(&session_id);
            log::info!("Expired session cleaned up: {}", session_id);
        }
    }

    /// Get active session count
    pub fn active_session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get all sessions for a specific user
    pub fn get_user_sessions(&self, user_id: &str) -> Vec<(&String, &UserSession)> {
        self.sessions
            .iter()
            .filter(|(_, session)| session.user.id == user_id)
            .collect()
    }

    /// Remove all sessions for a specific user
    pub fn remove_user_sessions(&mut self, user_id: &str) {
        let user_session_ids: Vec<String> = self
            .sessions
            .iter()
            .filter(|(_, session)| session.user.id == user_id)
            .map(|(id, _)| id.clone())
            .collect();

        for session_id in user_session_ids {
            self.sessions.remove(&session_id);
            log::info!("User session removed: {}", session_id);
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
