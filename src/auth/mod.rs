pub mod auth_manager;
pub mod models;
pub mod session;
pub mod ui;

pub use auth_manager::AuthManager;
pub use models::{LoginRequest, RegisterRequest, User};
pub use session::{SessionManager, UserSession};
pub use ui::{AuthState, AuthUI};

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Session expired")]
    SessionExpired,
    #[error("Unauthorized access")]
    Unauthorized,
    #[error("Password validation failed: {0}")]
    PasswordValidation(String),
    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),
}

pub type AuthResult<T> = Result<T, AuthError>;
