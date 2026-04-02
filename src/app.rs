use crate::alerts::AlertUI;
use crate::auth::AuthUI;
use crate::backend::{BackendHealth, SupabaseConfig};
use crate::models::{Product, Store};
use crate::settings::config::AppConfig;
#[cfg(not(target_arch = "wasm32"))]
use crate::scanner::ScannerUI;
use crate::services::AppServices;
use chrono::Utc;
use eframe::egui;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    current_tab: WorkspaceTab,
    discover_query: String,
    discover_category: Option<String>,
    selected_product_id: Option<String>,
    selected_store_id: Option<String>,
    contribution_product_id: Option<String>,
    contribution_store_id: Option<String>,
    contribution_price: String,
    contribution_is_on_sale: bool,
    contribution_comment: String,
    backend_config: SupabaseConfig,
    #[serde(skip)]
    auth_ui: AuthUI,
    #[serde(skip)]
    alert_ui: AlertUI,
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    scanner_ui: ScannerUI,
    #[serde(skip)]
    app_services: AppServices,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Default)]
enum WorkspaceTab {
    #[default]
    Discover,
    Contribute,
    Account,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let app_services = AppServices::new();
        let app_config = AppConfig::load().unwrap_or_default();
        let discover_defaults = Self::bootstrap_discover_selection(&app_services);

        Self {
            current_tab: WorkspaceTab::default(),
            discover_query: String::new(),
            discover_category: None,
            selected_product_id: discover_defaults.0,
            selected_store_id: discover_defaults.1,
            contribution_product_id: None,
            contribution_store_id: None,
            contribution_price: String::new(),
            contribution_is_on_sale: false,
            contribution_comment: String::new(),
            backend_config: SupabaseConfig::from_env_or_settings(&app_config.backend_settings),
            auth_ui: AuthUI::new(),
            alert_ui: AlertUI::new(),
            #[cfg(not(target_arch = "wasm32"))]
            scanner_ui: ScannerUI::new(),
            app_services,
        }
    }
}

impl TemplateApp {
    fn bootstrap_discover_selection(app_services: &AppServices) -> (Option<String>, Option<String>) {
        let selected_product_id = app_services
            .product_service
            .get_all_products()
            .ok()
            .and_then(|mut products| products.drain(..).next())
            .map(|product| product.id);

        let selected_store_id = app_services
            .store_service
            .list_stores(0, 1)
            .ok()
            .and_then(|mut stores| stores.drain(..).next())
            .map(|store| store.id);

        (selected_product_id, selected_store_id)
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "local_sans".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/simhei.ttf")).into(),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "local_sans".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        Self::default()
    }

    fn backend_health(&self) -> BackendHealth {
        BackendHealth::from_config(&self.backend_config)
    }

    fn render_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.heading("eprice");
            ui.add_space(8.0);
            ui.label("Local price comparison workspace");
            ui.separator();
            ui.label(self.backend_health().label());
            ui.separator();

            let auth_label = if self.auth_ui.is_logged_in() {
                "Account"
            } else {
                "Sign in"
            };
            if ui.button(auth_label).clicked() {
                self.auth_ui.open_auth_window();
            }
        });

        if !self.backend_config.is_configured() {
            ui.add_space(8.0);
            ui.group(|ui| {
                ui.colored_label(egui::Color32::YELLOW, "Supabase is not configured yet.");
                ui.label("Set SUPABASE_URL, SUPABASE_ANON_KEY, and SUPABASE_PROJECT_REF to enable cloud auth and sync.");
                for field in self.backend_config.missing_fields() {
                    ui.small(field);
                }
            });
        }
    }

    fn render_workspace_tabs(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if ui
                .selectable_label(self.current_tab == WorkspaceTab::Discover, "Discover / Compare")
                .clicked()
            {
                self.current_tab = WorkspaceTab::Discover;
            }
            if ui
                .selectable_label(self.current_tab == WorkspaceTab::Contribute, "Contribute")
                .clicked()
            {
                self.current_tab = WorkspaceTab::Contribute;
            }
            if ui
                .selectable_label(self.current_tab == WorkspaceTab::Account, "Account / Alerts")
                .clicked()
            {
                self.current_tab = WorkspaceTab::Account;
            }
            ui.separator();
            let stats = self.app_services.product_service.get_product_stats().ok();
            let store_stats = self.app_services.store_service.get_store_stats().ok();
            if let Some(stats) = stats {
                ui.label(format!("Products: {}", stats.total_products));
            }
            if let Some(stats) = store_stats {
                ui.label(format!("Stores: {}", stats.total_stores));
            }
            ui.label(format!("Generated at {}", Utc::now().format("%H:%M")));
        });
    }

    fn render_discover_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Discover / Compare");
        ui.label("Search products and stores, then inspect price history.");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Search");
            ui.add_sized(
                [260.0, 28.0],
                egui::TextEdit::singleline(&mut self.discover_query).hint_text("Product, store, or tag"),
            );
        });

        let categories: Vec<String> = self
            .app_services
            .product_service
            .get_categories()
            .into_iter()
            .collect();
        ui.horizontal_wrapped(|ui| {
            if ui
                .selectable_label(self.discover_category.is_none(), "All")
                .clicked()
            {
                self.discover_category = None;
            }
            for category in categories {
                let selected = self.discover_category.as_deref() == Some(category.as_str());
                if ui.selectable_label(selected, &category).clicked() {
                    self.discover_category = Some(category);
                }
            }
        });

        ui.separator();

        let query = self.discover_query.trim();
        let products = self
            .app_services
            .product_service
            .search_products(query, self.discover_category.as_deref())
            .unwrap_or_default();
        let stores = self
            .app_services
            .store_service
            .search_stores(query)
            .unwrap_or_default();

        ui.columns(2, |columns| {
            columns[0].heading("Products");
            egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                for product in &products {
                    let selected = self.selected_product_id.as_deref() == Some(product.id.as_str());
                    if ui
                        .selectable_label(selected, format!("{}  [{}]", product.name, product.category))
                        .clicked()
                    {
                        self.selected_product_id = Some(product.id.clone());
                    }
                    ui.small(product.description.clone());
                    ui.separator();
                }
            });

            columns[1].heading("Stores");
            egui::ScrollArea::vertical().show(&mut columns[1], |ui| {
                for store in &stores {
                    let selected = self.selected_store_id.as_deref() == Some(store.id.as_str());
                    if ui
                        .selectable_label(selected, &store.name)
                        .clicked()
                    {
                        self.selected_store_id = Some(store.id.clone());
                    }
                    ui.small(format!("{}  •  {:.1} km", store.address, store.distance_to(35.6812, 139.7671)));
                    ui.separator();
                }
            });
        });

        ui.separator();
        ui.columns(2, |columns| {
            columns[0].heading("Product detail");
            if let Some(product) = self.selected_product(query, &products) {
                self.render_product_detail(&mut columns[0], &product);
            } else {
                columns[0].label("Select a product to inspect its price story.");
            }

            columns[1].heading("Store detail");
            if let Some(store) = self.selected_store(query, &stores) {
                self.render_store_detail(&mut columns[1], &store);
            } else {
                columns[1].label("Select a store to inspect location and tags.");
            }
        });
    }

    fn render_product_detail(&self, ui: &mut egui::Ui, product: &Product) {
        ui.label(&product.description);
        ui.label(format!("Barcode: {}", product.barcode.clone().unwrap_or_else(|| "None".to_string())));
        ui.label(format!("Tags: {}", product.tags.join(", ")));

        if let Some(lowest) = product.current_lowest_price() {
            ui.label(format!("Lowest verified price: {:.2}", lowest.price));
        }
        if let Some(avg) = product.average_price() {
            ui.label(format!("Average verified price: {:.2}", avg));
        }

        ui.add_space(8.0);
        ui.label("Price history");
        let mut prices = product.prices.clone();
        prices.sort_by_key(|price| price.timestamp);
        if prices.is_empty() {
            ui.label("No price records yet.");
        } else {
            for price in prices {
                ui.horizontal(|ui| {
                    ui.label(price.timestamp.format("%Y-%m-%d").to_string());
                    ui.label(format!("{:.2}", price.price));
                    ui.label(if price.is_on_sale { "sale" } else { "" });
                });
            }
        }
    }

    fn render_store_detail(&self, ui: &mut egui::Ui, store: &Store) {
        ui.label(&store.address);
        ui.label(&store.opening_hours);
        ui.label(&store.phone);
        ui.label(format!("Rating: {:.1}", store.rating));
        ui.label(format!("Tags: {}", store.tags.join(", ")));
    }

    fn render_contribute_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Contribute");
        ui.label("Submit price records, verify findings, and leave review notes.");
        ui.add_space(8.0);

        let current_user = self.auth_ui.get_current_user().cloned();
        if current_user.is_none() {
            ui.colored_label(egui::Color32::YELLOW, "Sign in to contribute.");
            if ui.button("Open sign in").clicked() {
                self.auth_ui.open_auth_window();
            }
            return;
        }

        let current_user = current_user.unwrap();
        ui.label(format!("Posting as {}", current_user.username));
        ui.separator();

        let products = self.app_services.product_service.get_all_products().unwrap_or_default();
        let stores = self.app_services.store_service.list_stores(0, 500).unwrap_or_default();

        ui.horizontal(|ui| {
            ui.label("Product");
            egui::ComboBox::from_id_salt("contribute_product")
                .selected_text(
                    self.contribution_product_id
                        .as_ref()
                        .and_then(|id| products.iter().find(|p| &p.id == id))
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| "Select".to_string()),
                )
                .show_ui(ui, |ui| {
                    for product in &products {
                        ui.selectable_value(
                            &mut self.contribution_product_id,
                            Some(product.id.clone()),
                            &product.name,
                        );
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("Store");
            egui::ComboBox::from_id_salt("contribute_store")
                .selected_text(
                    self.contribution_store_id
                        .as_ref()
                        .and_then(|id| stores.iter().find(|s| &s.id == id))
                        .map(|s| s.name.clone())
                        .unwrap_or_else(|| "Select".to_string()),
                )
                .show_ui(ui, |ui| {
                    for store in &stores {
                        ui.selectable_value(
                            &mut self.contribution_store_id,
                            Some(store.id.clone()),
                            &store.name,
                        );
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("Price");
            ui.text_edit_singleline(&mut self.contribution_price);
            ui.checkbox(&mut self.contribution_is_on_sale, "On sale");
        });

        ui.label("Note");
        ui.add(egui::TextEdit::multiline(&mut self.contribution_comment).desired_rows(4));

        if ui.button("Submit price").clicked() {
            self.submit_price_contribution(&current_user);
        }

        ui.add_space(8.0);
        ui.separator();
        ui.heading("Scanner / OCR");
        #[cfg(not(target_arch = "wasm32"))]
        {
            let ctx = ui.ctx().clone();
            self.scanner_ui.show(&ctx, ui);
        }
        #[cfg(target_arch = "wasm32")]
        {
            ui.label("Scanner is available on desktop builds only.");
        }
    }

    fn submit_price_contribution(&mut self, current_user: &crate::models::User) {
        let product_id = match self.contribution_product_id.clone() {
            Some(id) => id,
            None => return,
        };
        let store_id = match self.contribution_store_id.clone() {
            Some(id) => id,
            None => return,
        };

        let price: f64 = match self.contribution_price.trim().parse() {
            Ok(price) => price,
            Err(_) => return,
        };

        if let Ok(price_record) = self.app_services.price_service.submit_price(
            product_id.clone(),
            store_id.clone(),
            Some(current_user.id.clone()),
            price,
            self.contribution_is_on_sale,
            None,
        ) && let Ok(product) = self.app_services.product_service.get_product(&product_id) {
            let _ = self
                .app_services
                .product_service
                .add_price_record(&product.id, price_record);
        }

        if !self.contribution_comment.trim().is_empty() {
            let _ = self.app_services.review_service.submit_review(
                current_user.id.clone(),
                Some(store_id),
                Some(product_id),
                5,
                self.contribution_comment.clone(),
            );
        }

        self.contribution_price.clear();
        self.contribution_comment.clear();
        self.contribution_is_on_sale = false;
    }

    fn render_account_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Account / Alerts");
        ui.label("Your profile, notifications, and backend status.");
        ui.add_space(8.0);

        match self.auth_ui.get_current_user().cloned() {
            Some(user) => {
                ui.group(|ui| {
                    ui.label(format!("Signed in as {}", user.username));
                    ui.label(format!("Email: {}", user.email));
                    ui.label(format!("Reputation: {}", user.reputation_score));
                    if ui.button("Open account panel").clicked() {
                        self.auth_ui.open_auth_window();
                    }
                    if ui.button("Logout").clicked() {
                        self.auth_ui.handle_logout();
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                self.alert_ui.show(ui, &user.id);
            }
            None => {
                ui.colored_label(egui::Color32::YELLOW, "You are not signed in.");
                if ui.button("Open sign in").clicked() {
                    self.auth_ui.open_auth_window();
                }
            }
        }

        ui.add_space(12.0);
        ui.separator();
        ui.heading("Supabase setup");
        ui.label(format!("Status: {}", self.backend_config.status_label()));
        for field in self.backend_config.missing_fields() {
            ui.small(format!("Missing: {}", field));
        }
        ui.small("The app is now wired for a Supabase-first workflow.");
    }

    fn selected_product(&self, query: &str, products: &[Product]) -> Option<Product> {
        if let Some(product_id) = &self.selected_product_id {
            products.iter().find(|product| &product.id == product_id).cloned()
        } else if query.is_empty() {
            products.first().cloned()
        } else {
            None
        }
    }

    fn selected_store(&self, query: &str, stores: &[Store]) -> Option<Store> {
        if let Some(store_id) = &self.selected_store_id {
            stores.iter().find(|store| &store.id == store_id).cloned()
        } else if query.is_empty() {
            stores.first().cloned()
        } else {
            None
        }
    }
}

impl eframe::App for TemplateApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    #[allow(deprecated)]
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        egui::TopBottomPanel::top("top_bar").show_inside(ui, |ui| {
            self.render_top_bar(ui);
        });

        egui::SidePanel::left("workspace_nav")
            .resizable(false)
            .min_width(180.0)
            .show_inside(ui, |ui| {
                self.render_workspace_tabs(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match self.current_tab {
                WorkspaceTab::Discover => self.render_discover_tab(ui),
                WorkspaceTab::Contribute => self.render_contribute_tab(ui),
                WorkspaceTab::Account => self.render_account_tab(ui),
            });
        });

        self.auth_ui.show_auth_dialog(&ctx);
    }
}
