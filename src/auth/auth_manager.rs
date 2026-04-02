use crate::auth::models::{LoginRequest, RegisterRequest, User};
use crate::auth::{AuthError, AuthResult};
use crate::utils::{validate_email, validate_password, validate_username};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Default)]
struct AuthStore {
    users: HashMap<String, User>,
    username_to_id: HashMap<String, String>,
    email_to_id: HashMap<String, String>,
    sessions: HashMap<String, String>,
}

/// Authentication manager for handling user login and registration.
///
/// This is a local, Supabase-shaped auth layer:
/// it keeps the same email/password/profile flow, but no longer depends on SQLite.
pub struct AuthManager {
    store: Mutex<AuthStore>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(AuthStore::default()),
        }
    }

    pub fn register(&self, request: RegisterRequest) -> AuthResult<User> {
        request.validate().map_err(AuthError::PasswordValidation)?;
        validate_username(&request.username).map_err(AuthError::PasswordValidation)?;
        if !validate_email(&request.email) {
            return Err(AuthError::PasswordValidation("Invalid email format".to_string()));
        }
        if !validate_password(&request.password) {
            return Err(AuthError::PasswordValidation(
                "Password must be at least 8 characters and include upper, lower, number, and symbol"
                    .to_string(),
            ));
        }

        let mut store = self.store.lock().expect("auth store poisoned");

        if store.username_to_id.contains_key(&request.username) {
            return Err(AuthError::UserAlreadyExists);
        }
        if store.email_to_id.contains_key(&request.email) {
            return Err(AuthError::UserAlreadyExists);
        }

        let password_hash =
            hash(&request.password, DEFAULT_COST).map_err(|e| AuthError::Database(e.into()))?;
        let user = User::new(request.username, request.email, password_hash);
        store.username_to_id.insert(user.username.clone(), user.id.clone());
        store.email_to_id.insert(user.email.clone(), user.id.clone());
        store.users.insert(user.id.clone(), user.clone());

        log::info!("User registered: {} ({})", user.username, user.email);
        Ok(user)
    }

    pub fn login(&self, request: LoginRequest) -> AuthResult<User> {
        let mut store = self.store.lock().expect("auth store poisoned");

        let user_id = store
            .email_to_id
            .get(&request.email)
            .cloned()
            .ok_or(AuthError::InvalidCredentials)?;

        let user = store
            .users
            .get(&user_id)
            .cloned()
            .ok_or(AuthError::InvalidCredentials)?;

        let valid = verify(&request.password, &user.password_hash)
            .map_err(|e| AuthError::Database(e.into()))?;
        if !valid {
            return Err(AuthError::InvalidCredentials);
        }

        let mut user = user;
        user.update_last_login();
        store.users.insert(user.id.clone(), user.clone());
        let session_id = format!("session_{}", user.id);
        store.sessions.insert(session_id, user.id.clone());

        log::info!("User logged in: {} ({})", user.username, user.email);
        Ok(user)
    }

    pub fn get_user_by_id(&self, user_id: &str) -> AuthResult<Option<User>> {
        let store = self.store.lock().expect("auth store poisoned");
        Ok(store.users.get(user_id).cloned())
    }

    pub fn update_profile(
        &self,
        user_id: &str,
        username: Option<String>,
        email: Option<String>,
        reputation_score: Option<i32>,
    ) -> AuthResult<User> {
        let mut store = self.store.lock().expect("auth store poisoned");
        let (old_username, old_email) = {
            let user = store.users.get(user_id).ok_or(AuthError::Unauthorized)?;
            (user.username.clone(), user.email.clone())
        };

        if let Some(new_username) = username {
            validate_username(&new_username).map_err(AuthError::PasswordValidation)?;
            if let Some(existing_id) = store.username_to_id.get(&new_username)
                && existing_id != user_id
            {
                return Err(AuthError::UserAlreadyExists);
            }
            store.username_to_id.remove(&old_username);
            store.username_to_id
                .insert(new_username.clone(), user_id.to_string());
            if let Some(user) = store.users.get_mut(user_id) {
                user.username = new_username;
            }
        }

        if let Some(new_email) = email {
            if !validate_email(&new_email) {
                return Err(AuthError::PasswordValidation("Invalid email format".to_string()));
            }
            if let Some(existing_id) = store.email_to_id.get(&new_email)
                && existing_id != user_id
            {
                return Err(AuthError::UserAlreadyExists);
            }
            store.email_to_id.remove(&old_email);
            store.email_to_id
                .insert(new_email.clone(), user_id.to_string());
            if let Some(user) = store.users.get_mut(user_id) {
                user.email = new_email;
            }
        }

        if let Some(score) = reputation_score {
            if let Some(user) = store.users.get_mut(user_id) {
                user.reputation_score = score;
            }
        }

        store
            .users
            .get(user_id)
            .cloned()
            .ok_or(AuthError::Unauthorized)
    }

    pub fn validate_session(&self, session_id: &str) -> AuthResult<bool> {
        let store = self.store.lock().expect("auth store poisoned");
        Ok(store.sessions.contains_key(session_id))
    }

    pub fn update_last_login(&self, user_id: &str) -> AuthResult<()> {
        let mut store = self.store.lock().expect("auth store poisoned");
        let user = store
            .users
            .get_mut(user_id)
            .ok_or(AuthError::Unauthorized)?;
        user.last_login = Some(Utc::now());
        Ok(())
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
