use crate::auth::session::{
    get_remembered_session, load_remembered_session_from_disk, set_remembered_session,
    GLOBAL_SESSION_MANAGER,
};
use crate::auth::AuthManager;
use crate::auth::SessionManager;
use crate::auth::models::{LoginRequest, RegisterRequest};
use crate::models::User;
use crate::utils::validate_email;
use egui;
use std::sync::Arc;

/// Authentication state for UI management
#[derive(Debug, Clone, Default)]
pub enum AuthState {
    #[default]
    LoggedOut,
    LoggingIn,
    Registering,
    LoggedIn(User),
}

/// Authentication UI component
pub struct AuthUI {
    pub auth_state: AuthState,
    pub session_manager: SessionManager,
    pub current_session_id: Option<String>,
    pub auth_manager: Arc<AuthManager>,
    pub login_email: String,
    pub login_password: String,
    pub login_remember_me: bool,
    pub login_error: Option<String>,
    pub register_username: String,
    pub register_email: String,
    pub register_password: String,
    pub register_password_confirm: String,
    pub register_error: Option<String>,
    pub show_auth_window: bool,
}

impl Default for AuthUI {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthUI {
    pub fn new() -> Self {
        let mut ui = Self {
            auth_state: AuthState::LoggedOut,
            session_manager: SessionManager::new(),
            current_session_id: None,
            auth_manager: Arc::new(AuthManager::new()),
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
        };

        load_remembered_session_from_disk();
        ui.try_restore_session();
        ui
    }

    pub fn show_auth_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_auth_window {
            return;
        }

        egui::Window::new("Account access")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .default_size(egui::vec2(440.0, 560.0))
            .show(ctx, |ui| {
                ui.style_mut().visuals.window_fill = egui::Color32::from_rgb(18, 18, 20);
                ui.style_mut().visuals.window_stroke =
                    egui::Stroke::new(1.0, egui::Color32::from_gray(60));
                self.render_auth_content(ui);
            });
    }

    fn render_auth_content(&mut self, ui: &mut egui::Ui) {
        match self.auth_state.clone() {
            AuthState::LoggedOut => self.render_logged_out_view(ui),
            AuthState::LoggingIn => self.render_login_form(ui),
            AuthState::Registering => self.render_register_form(ui),
            AuthState::LoggedIn(user) => self.render_logged_in_view(ui, &user),
        }
    }

    fn render_logged_out_view(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(16.0);
            ui.heading("eprice");
            ui.label(
                egui::RichText::new("Supabase-ready account system")
                    .color(egui::Color32::from_gray(150)),
            );
            ui.add_space(24.0);
            ui.label("Choose an action");
            ui.add_space(12.0);

            let button_size = egui::vec2(220.0, 38.0);
            if ui
                .add_sized(button_size, egui::Button::new("Sign in"))
                .clicked()
            {
                self.auth_state = AuthState::LoggingIn;
                self.clear_form_errors();
            }
            ui.add_space(8.0);
            if ui
                .add_sized(button_size, egui::Button::new("Create account"))
                .clicked()
            {
                self.auth_state = AuthState::Registering;
                self.clear_form_errors();
            }
            ui.add_space(8.0);
            if ui
                .add_sized(button_size, egui::Button::new("Close"))
                .clicked()
            {
                self.show_auth_window = false;
            }
        });
    }

    fn render_login_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Sign in");
        ui.add_space(12.0);

        ui.label("Email");
        ui.text_edit_singleline(&mut self.login_email);
        ui.add_space(8.0);

        ui.label("Password");
        ui.add(egui::TextEdit::singleline(&mut self.login_password).password(true));
        ui.checkbox(&mut self.login_remember_me, "Remember me");
        ui.add_space(8.0);

        if let Some(error) = &self.login_error {
            ui.colored_label(egui::Color32::from_rgb(220, 90, 90), error);
        }

        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                self.auth_state = AuthState::LoggedOut;
                self.clear_login_form();
            }
            if ui.button("Sign in").clicked() {
                self.handle_login();
            }
        });
    }

    fn render_register_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Create account");
        ui.add_space(12.0);

        ui.label("Username");
        ui.text_edit_singleline(&mut self.register_username);
        ui.label("Email");
        ui.text_edit_singleline(&mut self.register_email);
        ui.label("Password");
        ui.add(egui::TextEdit::singleline(&mut self.register_password).password(true));
        ui.label("Confirm password");
        ui.add(egui::TextEdit::singleline(&mut self.register_password_confirm).password(true));
        ui.add_space(8.0);

        if let Some(error) = &self.register_error {
            ui.colored_label(egui::Color32::from_rgb(220, 90, 90), error);
        }

        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                self.auth_state = AuthState::LoggedOut;
                self.clear_register_form();
            }
            if ui.button("Create").clicked() {
                self.handle_register();
            }
        });
    }

    fn render_logged_in_view(&mut self, ui: &mut egui::Ui, user: &User) {
        ui.heading("Account");
        ui.label(format!("Username: {}", user.username));
        ui.label(format!("Email: {}", user.email));
        ui.label(format!("Reputation: {}", user.reputation_score));
        ui.label(format!(
            "Last login: {}",
            user.last_login
                .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Never".to_string())
        ));

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                self.show_auth_window = false;
            }
            if ui.button("Logout").clicked() {
                self.handle_logout();
            }
        });
    }

    fn handle_login(&mut self) {
        self.login_error = None;

        if self.login_email.trim().is_empty() {
            self.login_error = Some("Please enter an email address".to_string());
            return;
        }
        if self.login_password.is_empty() {
            self.login_error = Some("Please enter a password".to_string());
            return;
        }

        let login_request = LoginRequest {
            email: self.login_email.trim().to_string(),
            password: self.login_password.clone(),
            remember_me: self.login_remember_me,
        };

        match self.auth_manager.login(login_request) {
            Ok(user) => {
                let local_session_id = self
                    .session_manager
                    .create_session(user.clone(), self.login_remember_me);
                self.current_session_id = Some(local_session_id);

                let remembered_session_id = if self.login_remember_me {
                    GLOBAL_SESSION_MANAGER
                        .lock()
                        .ok()
                        .map(|mut global| global.create_session(user.clone(), true))
                } else {
                    None
                };

                if self.login_remember_me {
                    set_remembered_session(remembered_session_id);
                } else {
                    set_remembered_session(None);
                }

                self.auth_state = AuthState::LoggedIn(user);
                self.clear_login_form();
            }
            Err(e) => {
                self.login_error = Some(match e {
                    crate::auth::AuthError::InvalidCredentials => {
                        "Invalid email or password".to_string()
                    }
                    crate::auth::AuthError::UserAlreadyExists => "User already exists".to_string(),
                    crate::auth::AuthError::SessionExpired => "Session expired".to_string(),
                    crate::auth::AuthError::Unauthorized => "Unauthorized access".to_string(),
                    crate::auth::AuthError::PasswordValidation(msg) => msg,
                    crate::auth::AuthError::Database(err) => format!("Auth error: {err}"),
                });
            }
        }
    }

    fn handle_register(&mut self) {
        self.register_error = None;

        let register_request = RegisterRequest {
            username: self.register_username.trim().to_string(),
            email: self.register_email.trim().to_string(),
            password: self.register_password.clone(),
            password_confirm: self.register_password_confirm.clone(),
        };

        if !validate_email(&register_request.email) {
            self.register_error = Some("Invalid email format".to_string());
            return;
        }

        match self.auth_manager.register(register_request) {
            Ok(user) => {
                let local_session_id = self.session_manager.create_session(user.clone(), true);
                self.current_session_id = Some(local_session_id);
                let remembered_session_id = GLOBAL_SESSION_MANAGER
                    .lock()
                    .ok()
                    .map(|mut global| global.create_session(user.clone(), true));
                set_remembered_session(remembered_session_id);
                self.auth_state = AuthState::LoggedIn(user);
                self.clear_register_form();
            }
            Err(e) => {
                self.register_error = Some(match e {
                    crate::auth::AuthError::InvalidCredentials => "Invalid credentials".to_string(),
                    crate::auth::AuthError::UserAlreadyExists => "User already exists".to_string(),
                    crate::auth::AuthError::SessionExpired => "Session expired".to_string(),
                    crate::auth::AuthError::Unauthorized => "Unauthorized access".to_string(),
                    crate::auth::AuthError::PasswordValidation(msg) => msg,
                    crate::auth::AuthError::Database(err) => format!("Auth error: {err}"),
                });
            }
        }
    }

    pub fn handle_logout(&mut self) {
        if let Some(session_id) = &self.current_session_id {
            let _ = self.session_manager.remove_session(session_id);
            if let Ok(mut g) = GLOBAL_SESSION_MANAGER.lock() {
                let _ = g.remove_session(session_id);
            }
        }

        self.current_session_id = None;
        self.auth_state = AuthState::LoggedOut;
        self.show_auth_window = false;
        set_remembered_session(None);
    }

    pub fn get_current_user(&mut self) -> Option<&User> {
        if let Some(session_id) = &self.current_session_id {
            self.session_manager.validate_session(session_id)
        } else if let Some(session_id) = get_remembered_session() {
            if let Ok(mut global) = GLOBAL_SESSION_MANAGER.lock()
                && let Some(user) = global.validate_session(&session_id)
            {
                let new_id = self.session_manager.create_session(user.clone(), true);
                self.current_session_id = Some(new_id);
                return self
                    .session_manager
                    .validate_session(self.current_session_id.as_deref().unwrap());
            }
            None
        } else {
            None
        }
    }

    pub fn is_logged_in(&mut self) -> bool {
        self.get_current_user().is_some()
    }

    pub fn open_auth_window(&mut self) {
        self.show_auth_window = true;
        if let Some(user) = self.get_current_user().cloned() {
            self.auth_state = AuthState::LoggedIn(user);
        } else {
            self.auth_state = AuthState::LoggedOut;
        }
    }

    fn try_restore_session(&mut self) {
        if let Some(session_id) = get_remembered_session()
            && let Ok(mut global) = GLOBAL_SESSION_MANAGER.lock()
            && let Some(user) = global.validate_session(&session_id).cloned()
        {
            self.current_session_id = Some(self.session_manager.create_session(user.clone(), true));
            self.auth_state = AuthState::LoggedIn(user);
            self.show_auth_window = false;
        }
    }

    fn clear_login_form(&mut self) {
        self.login_email.clear();
        self.login_password.clear();
        self.login_remember_me = false;
        self.login_error = None;
    }

    fn clear_register_form(&mut self) {
        self.register_username.clear();
        self.register_email.clear();
        self.register_password.clear();
        self.register_password_confirm.clear();
        self.register_error = None;
    }

    fn clear_form_errors(&mut self) {
        self.login_error = None;
        self.register_error = None;
    }
}
