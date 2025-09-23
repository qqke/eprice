use crate::auth::models::{LoginRequest, RegisterRequest};
use crate::auth::session::{
    GLOBAL_SESSION_MANAGER, get_remembered_session, load_remembered_session_from_disk,
    set_remembered_session,
};
use crate::auth::{AuthManager, SessionManager};
use crate::database::DatabaseManager;
use crate::models::User;
use crate::utils::validate_email;
use egui;
use std::sync::Arc;

/// Authentication state for UI management
#[derive(Debug, Clone)]
pub enum AuthState {
    LoggedOut,
    LoggingIn,
    Registering,
    LoggedIn(User),
}

impl Default for AuthState {
    fn default() -> Self {
        Self::LoggedOut
    }
}

/// Authentication UI component
#[derive(Default)]
pub struct AuthUI {
    pub auth_state: AuthState,
    pub session_manager: SessionManager,
    pub current_session_id: Option<String>,
    pub auth_manager: Option<Arc<AuthManager>>,

    // Login form fields
    pub login_email: String,
    pub login_password: String,
    pub login_remember_me: bool,
    pub login_error: Option<String>,

    // Registration form fields
    pub register_username: String,
    pub register_email: String,
    pub register_password: String,
    pub register_password_confirm: String,
    pub register_error: Option<String>,

    // UI state
    pub show_auth_window: bool,
}

// Default derive already provided

impl AuthUI {
    pub fn new() -> Self {
        // Attempt restore remembered session at creation
        let mut ui = Self {
            auth_manager: None,
            ..Self::default()
        };
        load_remembered_session_from_disk();
        ui.try_restore_session();
        ui
    }

    /// Initialize the AuthUI with database connection
    pub async fn with_database(
        database_manager: Arc<DatabaseManager>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize database
        let database = crate::database::Database::new(database_manager.pool().clone());
        database.initialize().await?;

        // Create auth manager
        let auth_manager = Arc::new(AuthManager::new(database_manager.pool().clone()));

        let mut ui = Self {
            auth_manager: Some(auth_manager),
            ..Self::default()
        };

        load_remembered_session_from_disk();
        ui.try_restore_session();
        Ok(ui)
    }

    /// Initialize the AuthUI with database connection using a new Tokio runtime
    pub fn with_database_sync(
        database_manager: Arc<DatabaseManager>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(Self::with_database(database_manager))
    }

    /// Show the authentication window
    pub fn show_auth_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_auth_window {
            return;
        }

        egui::Window::new("用户认证")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                self.render_auth_content(ui);
            });
    }

    /// Render the main authentication content
    fn render_auth_content(&mut self, ui: &mut egui::Ui) {
        match &self.auth_state.clone() {
            AuthState::LoggedOut => {
                self.render_logged_out_view(ui);
            }
            AuthState::LoggingIn => {
                self.render_login_form(ui);
            }
            AuthState::Registering => {
                self.render_register_form(ui);
            }
            AuthState::LoggedIn(user) => {
                self.render_logged_in_view(ui, user);
            }
        }
    }

    /// Render the initial view when user is logged out
    fn render_logged_out_view(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("欢迎使用 eprice");
            ui.add_space(20.0);

            ui.label("请选择操作:");
            ui.add_space(10.0);

            if ui.button("登录").clicked() {
                self.auth_state = AuthState::LoggingIn;
                self.clear_form_errors();
            }

            if ui.button("注册").clicked() {
                self.auth_state = AuthState::Registering;
                self.clear_form_errors();
            }

            ui.add_space(10.0);

            if ui.button("游客模式").clicked() {
                self.show_auth_window = false;
            }
        });
    }

    /// Render the login form
    fn render_login_form(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("用户登录");
            ui.add_space(20.0);

            // Email field
            ui.horizontal(|ui| {
                ui.label("邮箱:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.login_email).hint_text("请输入邮箱地址"),
                );
            });

            // Password field
            ui.horizontal(|ui| {
                ui.label("密码:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.login_password)
                        .password(true)
                        .hint_text("请输入密码"),
                );
            });

            // Remember me checkbox
            ui.checkbox(&mut self.login_remember_me, "记住我");

            ui.add_space(10.0);

            // Error message
            if let Some(error) = &self.login_error {
                ui.colored_label(egui::Color32::RED, error);
                ui.add_space(5.0);
            }

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("登录").clicked() {
                    self.handle_login();
                }

                if ui.button("返回").clicked() {
                    self.auth_state = AuthState::LoggedOut;
                    self.clear_login_form();
                }
            });

            ui.add_space(10.0);

            if ui.link("没有账户？点击注册").clicked() {
                self.auth_state = AuthState::Registering;
                self.clear_form_errors();
            }
        });
    }

    /// Render the registration form
    fn render_register_form(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("用户注册");
            ui.add_space(20.0);

            // Username field
            ui.horizontal(|ui| {
                ui.label("用户名:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.register_username)
                        .hint_text("请输入用户名"),
                );
            });

            // Email field
            ui.horizontal(|ui| {
                ui.label("邮箱:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.register_email)
                        .hint_text("请输入邮箱地址"),
                );
            });

            // Password field
            ui.horizontal(|ui| {
                ui.label("密码:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.register_password)
                        .password(true)
                        .hint_text("请输入密码"),
                );
            });

            // Confirm password field
            ui.horizontal(|ui| {
                ui.label("确认密码:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.register_password_confirm)
                        .password(true)
                        .hint_text("请再次输入密码"),
                );
            });

            ui.add_space(10.0);

            // Error message
            if let Some(error) = &self.register_error {
                ui.colored_label(egui::Color32::RED, error);
                ui.add_space(5.0);
            }

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("注册").clicked() {
                    self.handle_register();
                }

                if ui.button("返回").clicked() {
                    self.auth_state = AuthState::LoggedOut;
                    self.clear_register_form();
                }
            });

            ui.add_space(10.0);

            if ui.link("已有账户？点击登录").clicked() {
                self.auth_state = AuthState::LoggingIn;
                self.clear_form_errors();
            }
        });
    }

    /// Render the logged in user view
    fn render_logged_in_view(&mut self, ui: &mut egui::Ui, user: &User) {
        ui.vertical_centered(|ui| {
            ui.heading("用户信息");
            ui.add_space(20.0);

            ui.label(format!("欢迎, {}!", user.username));
            ui.label(format!("邮箱: {}", user.email));
            ui.label(format!("信誉分数: {}", user.reputation_score));

            if let Some(last_login) = user.last_login {
                ui.label(format!(
                    "上次登录: {}",
                    last_login.format("%Y-%m-%d %H:%M:%S")
                ));
            }

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if ui.button("继续使用").clicked() {
                    self.show_auth_window = false;
                }

                if ui.button("退出登录").clicked() {
                    self.handle_logout();
                }
            });
        });
    }

    /// Handle login attempt
    fn handle_login(&mut self) {
        // Validate input
        if self.login_email.is_empty() {
            self.login_error = Some("请输入邮箱地址".to_string());
            return;
        }

        if self.login_password.is_empty() {
            self.login_error = Some("请输入密码".to_string());
            return;
        }

        // Create login request
        let login_request = LoginRequest {
            email: self.login_email.clone(),
            password: self.login_password.clone(),
            remember_me: self.login_remember_me,
        };

        // Use database authentication if available
        if let Some(auth_manager) = &self.auth_manager {
            // Create a new runtime for database operations
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(auth_manager.login(login_request)) {
                Ok(user) => {
                    // Store also in global session manager for cross-UI persistence
                    let session_id = {
                        let mut global = GLOBAL_SESSION_MANAGER.lock().unwrap();
                        global.create_session(user.clone(), self.login_remember_me)
                    };
                    // Mirror to local manager for immediate access
                    self.current_session_id = Some(session_id.clone());
                    self.session_manager
                        .create_session(user.clone(), self.login_remember_me);
                    if self.login_remember_me {
                        set_remembered_session(Some(session_id.clone()));
                    } else {
                        set_remembered_session(None);
                    }

                    self.auth_state = AuthState::LoggedIn(user);
                    self.clear_login_form();
                    self.login_error = None;
                }
                Err(e) => {
                    self.login_error = Some(match e {
                        crate::auth::AuthError::InvalidCredentials => "邮箱或密码错误".to_string(),
                        crate::auth::AuthError::UserAlreadyExists => "用户已存在".to_string(),
                        crate::auth::AuthError::SessionExpired => "会话已过期".to_string(),
                        crate::auth::AuthError::Unauthorized => "未授权访问".to_string(),
                        crate::auth::AuthError::PasswordValidation(msg) => {
                            format!("密码验证失败: {}", msg)
                        }
                        crate::auth::AuthError::Database(_) => "数据库错误".to_string(),
                    });
                }
            }
        } else {
            self.login_error = Some("数据库连接不可用".to_string());
        }
    }

    /// Handle registration attempt
    fn handle_register(&mut self) {
        let register_request = RegisterRequest {
            username: self.register_username.clone(),
            email: self.register_email.clone(),
            password: self.register_password.clone(),
            password_confirm: self.register_password_confirm.clone(),
        };

        // Validate registration (basic UI checks)
        if let Err(error) = register_request.validate() {
            self.register_error = Some(error);
            return;
        }

        if !validate_email(&register_request.email) {
            self.register_error = Some("邮箱格式不正确".to_string());
            return;
        }

        // Use database authentication if available
        if let Some(auth_manager) = &self.auth_manager {
            // Create a new runtime for database operations
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(auth_manager.register(register_request)) {
                Ok(user) => {
                    let session_id = {
                        let mut global = GLOBAL_SESSION_MANAGER.lock().unwrap();
                        global.create_session(user.clone(), false)
                    };
                    self.current_session_id = Some(session_id.clone());
                    self.session_manager.create_session(user.clone(), false);
                    set_remembered_session(Some(session_id.clone()));

                    self.auth_state = AuthState::LoggedIn(user);
                    self.clear_register_form();
                    self.register_error = None;
                }
                Err(e) => {
                    let msg = match e {
                        crate::auth::AuthError::InvalidCredentials => "无效凭据".to_string(),
                        crate::auth::AuthError::UserAlreadyExists => "用户已存在".to_string(),
                        crate::auth::AuthError::SessionExpired => "会话已过期".to_string(),
                        crate::auth::AuthError::Unauthorized => "未授权访问".to_string(),
                        crate::auth::AuthError::PasswordValidation(msg) => {
                            format!("密码验证失败: {}", msg)
                        }
                        crate::auth::AuthError::Database(_) => "数据库错误".to_string(),
                    };
                    self.register_error = Some(msg);
                }
            }
        } else {
            self.register_error = Some("数据库连接不可用".to_string());
        }
    }

    /// Handle logout
    pub fn handle_logout(&mut self) {
        if let Some(session_id) = &self.current_session_id {
            let _ = self.session_manager.remove_session(session_id);
            let _ = GLOBAL_SESSION_MANAGER.lock().map(|mut g| {
                g.remove_session(session_id);
            });
        }

        self.current_session_id = None;
        self.auth_state = AuthState::LoggedOut;
        self.clear_all_forms();
        self.show_auth_window = false;
    }

    /// Get current logged in user
    pub fn get_current_user(&mut self) -> Option<&User> {
        if let Some(session_id) = &self.current_session_id {
            self.session_manager.validate_session(session_id)
        } else if let Some(session_id) = get_remembered_session() {
            // Try global session manager, then mirror into local manager to return a stable reference
            if let Ok(mut global) = GLOBAL_SESSION_MANAGER.lock() {
                if let Some(user) = global.validate_session(&session_id) {
                    let new_id = self.session_manager.create_session(user.clone(), true);
                    self.current_session_id = Some(new_id);
                    return self
                        .session_manager
                        .validate_session(self.current_session_id.as_deref().unwrap());
                }
            }
            None
        } else {
            None
        }
    }

    /// Check if user is logged in
    pub fn is_logged_in(&self) -> bool {
        matches!(self.auth_state, AuthState::LoggedIn(_))
    }

    /// Open the authentication window
    pub fn open_auth_window(&mut self) {
        self.show_auth_window = true;
        if self.get_current_user().is_some() {
            if let Some(id) = &self.current_session_id {
                if let Some(user) = self.session_manager.get_session(id).map(|s| s.user.clone()) {
                    self.auth_state = AuthState::LoggedIn(user);
                }
            }
        } else {
            self.auth_state = AuthState::LoggedOut;
        }
    }

    /// Close the authentication window
    pub fn close_auth_window(&mut self) {
        self.show_auth_window = false;
    }

    fn try_restore_session(&mut self) {
        if let Some(session_id) = get_remembered_session() {
            if let Ok(mut global) = GLOBAL_SESSION_MANAGER.lock() {
                if let Some(user) = global.validate_session(&session_id) {
                    // Mirror into local manager so references are stable within UI
                    let new_id = self.session_manager.create_session(user.clone(), true);
                    self.current_session_id = Some(new_id);
                    self.auth_state = AuthState::LoggedIn(user.clone());
                }
            }
        }
    }

    // Helper methods for form management
    fn clear_login_form(&mut self) {
        self.login_email.clear();
        self.login_password.clear();
        self.login_remember_me = false;
    }

    fn clear_register_form(&mut self) {
        self.register_username.clear();
        self.register_email.clear();
        self.register_password.clear();
        self.register_password_confirm.clear();
    }

    fn clear_all_forms(&mut self) {
        self.clear_login_form();
        self.clear_register_form();
    }

    fn clear_form_errors(&mut self) {
        self.login_error = None;
        self.register_error = None;
    }

    // Simulation methods removed (UserService is used instead)
}
