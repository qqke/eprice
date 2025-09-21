use crate::auth::SessionManager;
use crate::auth::models::{LoginRequest, RegisterRequest};
use crate::models::User;
use egui;

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
pub struct AuthUI {
    pub auth_state: AuthState,
    pub session_manager: SessionManager,
    pub current_session_id: Option<String>,

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

impl Default for AuthUI {
    fn default() -> Self {
        Self {
            auth_state: AuthState::default(),
            session_manager: SessionManager::new(),
            current_session_id: None,
            login_email: String::new(),
            login_password: String::new(),
            login_remember_me: false,
            login_error: None,
            register_username: String::new(),
            register_email: String::new(),
            register_password: String::new(),
            register_password_confirm: String::new(),
            register_error: None,
            show_auth_window: false,
        }
    }
}

impl AuthUI {
    pub fn new() -> Self {
        Self::default()
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

        // Simulate authentication (in real app, this would call AuthManager)
        if self.simulate_login(&login_request) {
            let user = User::new(
                "test_user".to_string(),
                self.login_email.clone(),
                "hashed_password".to_string(),
            );

            // Create session
            let session_id = self
                .session_manager
                .create_session(user.clone(), self.login_remember_me);
            self.current_session_id = Some(session_id);

            self.auth_state = AuthState::LoggedIn(user);
            self.clear_login_form();
            self.login_error = None;
        } else {
            self.login_error = Some("邮箱或密码错误".to_string());
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

        // Validate registration
        if let Err(error) = register_request.validate() {
            self.register_error = Some(error);
            return;
        }

        // Simulate registration (in real app, this would call AuthManager)
        if self.simulate_register(&register_request) {
            let user = User::new(
                register_request.username.clone(),
                register_request.email.clone(),
                "hashed_password".to_string(),
            );

            // Create session
            let session_id = self.session_manager.create_session(user.clone(), false);
            self.current_session_id = Some(session_id);

            self.auth_state = AuthState::LoggedIn(user);
            self.clear_register_form();
            self.register_error = None;
        } else {
            self.register_error = Some("注册失败，用户可能已存在".to_string());
        }
    }

    /// Handle logout
    pub fn handle_logout(&mut self) {
        if let Some(session_id) = &self.current_session_id {
            self.session_manager.remove_session(session_id);
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
        self.auth_state = AuthState::LoggedOut;
    }

    /// Close the authentication window
    pub fn close_auth_window(&mut self) {
        self.show_auth_window = false;
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

    // Simulation methods (would be replaced with real AuthManager calls)
    fn simulate_login(&self, _request: &LoginRequest) -> bool {
        // Simple simulation - accept any non-empty credentials
        !_request.email.is_empty() && !_request.password.is_empty()
    }

    fn simulate_register(&self, _request: &RegisterRequest) -> bool {
        // Simple simulation - accept valid registration
        true
    }
}
