use crate::models::User;
use crate::services::{ServiceError, ServiceResult};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// User service for managing user registration, authentication, and session management
pub struct UserService {
    /// In-memory user storage (in real app would use database)
    users: HashMap<String, User>,
    /// Username to ID mapping for quick lookups
    username_to_id: HashMap<String, String>,
    /// Email to ID mapping for quick lookups
    email_to_id: HashMap<String, String>,
    /// Active sessions (session_token -> user_id)
    sessions: HashMap<String, String>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            username_to_id: HashMap::new(),
            email_to_id: HashMap::new(),
            sessions: HashMap::new(),
        }
    }

    /// Register a new user
    pub fn register_user(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> ServiceResult<User> {
        // Validate input
        self.validate_username(&username)?;
        self.validate_email(&email)?;
        self.validate_password(&password)?;

        // Check for duplicates
        if self.username_to_id.contains_key(&username) {
            return Err(ServiceError::BusinessRuleViolation(
                "Username already exists".to_string(),
            ));
        }

        if self.email_to_id.contains_key(&email) {
            return Err(ServiceError::BusinessRuleViolation(
                "Email already exists".to_string(),
            ));
        }

        // Create user
        let user = User::new(
            username.clone(),
            email.clone(),
            self.hash_password(&password)?,
        );

        // Store user
        self.users.insert(user.id.clone(), user.clone());
        self.username_to_id.insert(username, user.id.clone());
        self.email_to_id.insert(email, user.id.clone());

        log::info!("User registered: {} ({})", user.username, user.id);
        Ok(user)
    }

    /// Login with username or email
    pub fn login(
        &mut self,
        username_or_email: String,
        password: String,
    ) -> ServiceResult<(User, String)> {
        // Find user by username or email
        let user_id = self
            .username_to_id
            .get(&username_or_email)
            .or_else(|| self.email_to_id.get(&username_or_email))
            .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;

        let user_id = user_id.clone(); // Clone to avoid borrow issues

        // Get user and verify password
        let user = self
            .users
            .get(&user_id)
            .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;

        // Verify password
        if !self.verify_password(&password, &user.password_hash)? {
            return Err(ServiceError::PermissionDenied(
                "Invalid password".to_string(),
            ));
        }

        // Create session
        let session_token = self.create_session_token();

        // Now update last login and create session
        let user = self.users.get_mut(&user_id).unwrap(); // Safe since we checked above
        user.last_login = Some(Utc::now());

        self.sessions.insert(session_token.clone(), user.id.clone());

        log::info!("User logged in: {} ({})", user.username, user.id);
        Ok((user.clone(), session_token))
    }

    /// Logout and invalidate session
    pub fn logout(&mut self, session_token: &str) -> ServiceResult<()> {
        self.sessions
            .remove(session_token)
            .ok_or_else(|| ServiceError::NotFound("Invalid session".to_string()))?;

        log::info!("User logged out with session: {}", session_token);
        Ok(())
    }

    /// Validate session and return user
    pub fn validate_session(&self, session_token: &str) -> ServiceResult<User> {
        let user_id = self
            .sessions
            .get(session_token)
            .ok_or_else(|| ServiceError::PermissionDenied("Invalid session".to_string()))?;

        let user = self
            .users
            .get(user_id)
            .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;

        Ok(user.clone())
    }

    /// Get user by ID
    pub fn get_user(&self, user_id: &str) -> ServiceResult<User> {
        self.users
            .get(user_id)
            .cloned()
            .ok_or_else(|| ServiceError::NotFound(format!("User {} not found", user_id)))
    }

    /// Update user profile
    pub fn update_user_profile(
        &mut self,
        user_id: &str,
        username: Option<String>,
        email: Option<String>,
        reputation_score: Option<i32>,
    ) -> ServiceResult<User> {
        // Validate inputs first
        if let Some(ref new_username) = username {
            self.validate_username(new_username)?;

            // Check if username is already taken by another user
            if let Some(existing_id) = self.username_to_id.get(new_username) {
                if existing_id != user_id {
                    return Err(ServiceError::BusinessRuleViolation(
                        "Username already exists".to_string(),
                    ));
                }
            }
        }

        if let Some(ref new_email) = email {
            self.validate_email(new_email)?;

            // Check if email is already taken by another user
            if let Some(existing_id) = self.email_to_id.get(new_email) {
                if existing_id != user_id {
                    return Err(ServiceError::BusinessRuleViolation(
                        "Email already exists".to_string(),
                    ));
                }
            }
        }

        let user = self
            .users
            .get_mut(user_id)
            .ok_or_else(|| ServiceError::NotFound(format!("User {} not found", user_id)))?;

        // Update username if provided
        if let Some(new_username) = username {
            // Remove old username mapping
            self.username_to_id.remove(&user.username);
            // Add new username mapping
            self.username_to_id
                .insert(new_username.clone(), user_id.to_string());
            // Update user
            user.username = new_username;
        }

        // Update email if provided
        if let Some(new_email) = email {
            // Remove old email mapping
            self.email_to_id.remove(&user.email);
            // Add new email mapping
            self.email_to_id
                .insert(new_email.clone(), user_id.to_string());
            // Update user
            user.email = new_email;
        }

        // Update reputation score if provided
        if let Some(new_score) = reputation_score {
            user.reputation_score = new_score;
        }

        log::info!("User profile updated: {} ({})", user.username, user.id);
        Ok(user.clone())
    }

    /// Change user password
    pub fn change_password(
        &mut self,
        user_id: &str,
        old_password: String,
        new_password: String,
    ) -> ServiceResult<()> {
        // First validate the new password
        self.validate_password(&new_password)?;

        // Generate new hash before getting mutable reference
        let new_hash = self.hash_password(&new_password)?;

        // Get user and verify old password
        let user = self
            .users
            .get(user_id)
            .ok_or_else(|| ServiceError::NotFound(format!("User {} not found", user_id)))?;

        // Verify old password
        if !self.verify_password(&old_password, &user.password_hash)? {
            return Err(ServiceError::PermissionDenied(
                "Invalid old password".to_string(),
            ));
        }

        // Now get mutable reference and update password
        let user = self.users.get_mut(user_id).unwrap(); // Safe since we checked above
        user.password_hash = new_hash;

        log::info!("Password changed for user: {} ({})", user.username, user.id);
        Ok(())
    }

    /// Search users by username or email
    pub fn search_users(&self, query: &str) -> ServiceResult<Vec<User>> {
        let query_lower = query.to_lowercase();

        let users: Vec<User> = self
            .users
            .values()
            .filter(|u| {
                u.username.to_lowercase().contains(&query_lower)
                    || u.email.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();

        Ok(users)
    }

    /// Get user statistics
    pub fn get_user_stats(&self) -> ServiceResult<UserStats> {
        let stats = UserStats {
            total_users: self.users.len(),
            active_sessions: self.sessions.len(),
            average_reputation: if self.users.is_empty() {
                0.0
            } else {
                let total_reputation: i32 = self.users.values().map(|u| u.reputation_score).sum();
                total_reputation as f64 / self.users.len() as f64
            },
        };

        Ok(stats)
    }

    /// Get all active sessions (admin function)
    pub fn get_active_sessions(&self) -> ServiceResult<Vec<SessionInfo>> {
        let sessions: Vec<SessionInfo> = self
            .sessions
            .iter()
            .filter_map(|(token, user_id)| {
                self.users.get(user_id).map(|user| SessionInfo {
                    session_token: token.clone(),
                    user_id: user_id.clone(),
                    username: user.username.clone(),
                })
            })
            .collect();

        Ok(sessions)
    }

    // Helper methods

    fn validate_username(&self, username: &str) -> ServiceResult<()> {
        if username.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Username cannot be empty".to_string(),
            ));
        }

        if username.len() < 3 {
            return Err(ServiceError::ValidationError(
                "Username must be at least 3 characters".to_string(),
            ));
        }

        if username.len() > 30 {
            return Err(ServiceError::ValidationError(
                "Username too long".to_string(),
            ));
        }

        // Simple alphanumeric + underscore validation
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ServiceError::ValidationError(
                "Username can only contain letters, numbers, and underscores".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_email(&self, email: &str) -> ServiceResult<()> {
        if email.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Email cannot be empty".to_string(),
            ));
        }

        // Simple email validation
        if !email.contains('@') || !email.contains('.') {
            return Err(ServiceError::ValidationError(
                "Invalid email format".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_password(&self, password: &str) -> ServiceResult<()> {
        if password.len() < 6 {
            return Err(ServiceError::ValidationError(
                "Password must be at least 6 characters".to_string(),
            ));
        }

        if password.len() > 100 {
            return Err(ServiceError::ValidationError(
                "Password too long".to_string(),
            ));
        }

        Ok(())
    }

    fn hash_password(&self, password: &str) -> ServiceResult<String> {
        // In a real app, use bcrypt or similar
        // For now, simple mock hash
        Ok(format!("hashed_{}", password))
    }

    fn verify_password(&self, password: &str, hash: &str) -> ServiceResult<bool> {
        // In a real app, use bcrypt verification
        // For now, simple mock verification
        Ok(hash == &format!("hashed_{}", password))
    }

    fn create_session_token(&self) -> String {
        // In a real app, use proper session token generation
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or(0)
            .hash(&mut hasher);
        format!("session_{}", hasher.finish())
    }
}

impl Default for UserService {
    fn default() -> Self {
        Self::new()
    }
}

/// User statistics
#[derive(Debug, Clone)]
pub struct UserStats {
    pub total_users: usize,
    pub active_sessions: usize,
    pub average_reputation: f64,
}

/// Session information
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_token: String,
    pub user_id: String,
    pub username: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_registration_success() {
        let mut service = UserService::new();

        let result = service.register_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "password123".to_string(),
        );

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert!(!user.password_hash.is_empty());
    }

    #[test]
    fn test_user_registration_duplicate_username() {
        let mut service = UserService::new();

        // Register first user
        service
            .register_user(
                "testuser".to_string(),
                "test1@example.com".to_string(),
                "password123".to_string(),
            )
            .unwrap();

        // Try to register with same username
        let result = service.register_user(
            "testuser".to_string(),
            "test2@example.com".to_string(),
            "password456".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_user_registration_duplicate_email() {
        let mut service = UserService::new();

        // Register first user
        service
            .register_user(
                "testuser1".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .unwrap();

        // Try to register with same email
        let result = service.register_user(
            "testuser2".to_string(),
            "test@example.com".to_string(),
            "password456".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_user_registration_invalid_input() {
        let mut service = UserService::new();

        // Test empty username
        let result = service.register_user(
            "".to_string(),
            "test@example.com".to_string(),
            "password123".to_string(),
        );
        assert!(result.is_err());

        // Test invalid email
        let result = service.register_user(
            "testuser".to_string(),
            "invalid-email".to_string(),
            "password123".to_string(),
        );
        assert!(result.is_err());

        // Test weak password
        let result = service.register_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "123".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_user_login_success() {
        let mut service = UserService::new();

        // Register user first
        service
            .register_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .unwrap();

        // Login with username
        let result = service.login("testuser".to_string(), "password123".to_string());
        assert!(result.is_ok());

        let (user, session_token) = result.unwrap();
        assert_eq!(user.username, "testuser");
        assert!(!session_token.is_empty());

        // Login with email
        let result = service.login("test@example.com".to_string(), "password123".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_user_login_failure() {
        let mut service = UserService::new();

        // Register user first
        service
            .register_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .unwrap();

        // Wrong password
        let result = service.login("testuser".to_string(), "wrongpassword".to_string());
        assert!(result.is_err());

        // Non-existent user
        let result = service.login("nonexistent".to_string(), "password123".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_session_management() {
        let mut service = UserService::new();

        // Register and login user
        let user = service
            .register_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .unwrap();

        let (_, session_token) = service
            .login("testuser".to_string(), "password123".to_string())
            .unwrap();

        // Validate session
        let result = service.validate_session(&session_token);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, user.id);

        // Logout
        let result = service.logout(&session_token);
        assert!(result.is_ok());

        // Session should be invalid after logout
        let result = service.validate_session(&session_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_user_profile_update() {
        let mut service = UserService::new();

        // Register user
        let user = service
            .register_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .unwrap();

        // Update profile
        let result = service.update_user_profile(
            &user.id,
            Some("newusername".to_string()),
            Some("newemail@example.com".to_string()),
            None,
        );
        assert!(result.is_ok());

        let updated_user = result.unwrap();
        assert_eq!(updated_user.username, "newusername");
        assert_eq!(updated_user.email, "newemail@example.com");
    }

    #[test]
    fn test_password_change() {
        let mut service = UserService::new();

        // Register user
        let user = service
            .register_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
            )
            .unwrap();

        // Change password
        let result = service.change_password(
            &user.id,
            "password123".to_string(),
            "newpassword456".to_string(),
        );
        assert!(result.is_ok());

        // Old password should not work
        let result = service.login("testuser".to_string(), "password123".to_string());
        assert!(result.is_err());

        // New password should work
        let result = service.login("testuser".to_string(), "newpassword456".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_user_stats() {
        let mut service = UserService::new();

        // Register multiple users
        service
            .register_user(
                "user1".to_string(),
                "user1@example.com".to_string(),
                "password123".to_string(),
            )
            .unwrap();
        service
            .register_user(
                "user2".to_string(),
                "user2@example.com".to_string(),
                "password123".to_string(),
            )
            .unwrap();

        // Login one user
        service
            .login("user1".to_string(), "password123".to_string())
            .unwrap();

        let stats = service.get_user_stats().unwrap();
        assert_eq!(stats.total_users, 2);
        assert_eq!(stats.active_sessions, 1);
    }
}
