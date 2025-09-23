use crate::auth::models::{LoginRequest, RegisterRequest, User};
use crate::auth::{AuthError, AuthResult};
#[cfg(not(target_arch = "wasm32"))]
use crate::database::{UserRepository, repository::Repository};
use crate::utils::crypto::{hash_password, verify_password};
use crate::utils::{validate_email, validate_password, validate_username};
#[cfg(not(target_arch = "wasm32"))]
use sqlx::Pool;
#[cfg(not(target_arch = "wasm32"))]
use sqlx::Sqlite;

/// Authentication manager for handling user login and registration
#[cfg(not(target_arch = "wasm32"))]
pub struct AuthManager {
    user_repository: UserRepository,
}

impl AuthManager {
    /// Create a new authentication manager
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            user_repository: UserRepository::new(pool),
        }
    }

    /// Register a new user
    pub async fn register(&self, request: RegisterRequest) -> AuthResult<User> {
        // Validate the registration request
        request.validate().map_err(AuthError::PasswordValidation)?;

        // Additional validations
        if !validate_email(&request.email) {
            return Err(AuthError::PasswordValidation(
                "Invalid email format".to_string(),
            ));
        }

        validate_username(&request.username).map_err(AuthError::PasswordValidation)?;

        if !validate_password(&request.password) {
            return Err(AuthError::PasswordValidation("密码不符合要求".to_string()));
        }

        // Check if user already exists
        if self
            .user_repository
            .find_by_email(&request.email)
            .await?
            .is_some()
        {
            return Err(AuthError::UserAlreadyExists);
        }

        if self
            .user_repository
            .find_by_username(&request.username)
            .await?
            .is_some()
        {
            return Err(AuthError::UserAlreadyExists);
        }

        // Hash the password
        let password_hash = hash_password(&request.password).map_err(|e| {
            AuthError::PasswordValidation(format!("Password hashing failed: {}", e))
        })?;

        // Create new user
        let user = User::new(request.username, request.email, password_hash);

        // Save to database
        self.user_repository.create(&user).await?;

        log::info!("New user registered: {} ({})", user.username, user.email);
        Ok(user)
    }

    /// Login a user
    pub async fn login(&self, request: LoginRequest) -> AuthResult<User> {
        // Find user by email
        let mut user = self
            .user_repository
            .find_by_email(&request.email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        let is_valid = verify_password(&request.password, &user.password_hash)
            .map_err(|_e| AuthError::InvalidCredentials)?;

        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        // Update last login
        user.update_last_login();
        self.user_repository.update_last_login(&user.id).await?;

        log::info!("User logged in: {} ({})", user.username, user.email);
        Ok(user)
    }

    /// Change user password
    pub async fn change_password(
        &self,
        user_id: &str,
        old_password: &str,
        new_password: &str,
    ) -> AuthResult<()> {
        // Find user
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify old password
        let is_valid = verify_password(old_password, &user.password_hash)
            .map_err(|_e| AuthError::InvalidCredentials)?;

        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        // Validate new password
        if !validate_password(new_password) {
            return Err(AuthError::PasswordValidation(
                "新密码不符合要求".to_string(),
            ));
        }

        // Hash new password
        let new_password_hash = hash_password(new_password).map_err(|e| {
            AuthError::PasswordValidation(format!("Password hashing failed: {}", e))
        })?;

        // Update user
        let mut updated_user = user;
        updated_user.password_hash = new_password_hash;
        self.user_repository.update(&updated_user).await?;

        log::info!(
            "Password changed for user: {} ({})",
            updated_user.username,
            updated_user.email
        );
        Ok(())
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: &str) -> AuthResult<Option<User>> {
        let user = self.user_repository.find_by_id(user_id).await?;
        Ok(user)
    }

    /// Update user profile
    pub async fn update_user_profile(
        &self,
        user_id: &str,
        username: Option<String>,
        email: Option<String>,
    ) -> AuthResult<User> {
        // Find user
        let mut user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Update username if provided
        if let Some(new_username) = username {
            validate_username(&new_username).map_err(AuthError::PasswordValidation)?;

            // Check if username is already taken
            if let Some(existing_user) =
                self.user_repository.find_by_username(&new_username).await?
            {
                if existing_user.id != user.id {
                    return Err(AuthError::UserAlreadyExists);
                }
            }

            user.username = new_username;
        }

        // Update email if provided
        if let Some(new_email) = email {
            if !validate_email(&new_email) {
                return Err(AuthError::PasswordValidation(
                    "Invalid email format".to_string(),
                ));
            }

            // Check if email is already taken
            if let Some(existing_user) = self.user_repository.find_by_email(&new_email).await? {
                if existing_user.id != user.id {
                    return Err(AuthError::UserAlreadyExists);
                }
            }

            user.email = new_email;
        }

        // Save changes
        self.user_repository.update(&user).await?;

        log::info!("User profile updated: {} ({})", user.username, user.email);
        Ok(user)
    }

    /// Validate user session (placeholder for session-based authentication)
    pub async fn validate_session(&self, user_id: &str) -> AuthResult<bool> {
        // Simple implementation - just check if user exists
        let user = self.user_repository.find_by_id(user_id).await?;
        Ok(user.is_some())
    }
}
