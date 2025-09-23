use crate::alerts::AlertUI;
use crate::auth::{AuthState, AuthUI};
use crate::database::DatabaseManager;
use crate::models::{PriceRecord, Product, Store};
#[cfg(not(target_arch = "wasm32"))]
use crate::scanner::ScannerUI;
use crate::services::AppServices;
use chrono::Utc;
use eframe::egui;
use std::sync::Arc;
use walkers::{
    HttpTiles, Map, MapMemory, Position, Tiles,
    extras::{LabeledSymbol, LabeledSymbolStyle, Places, Symbol},
    sources::OpenStreetMap,
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    stores: Vec<Store>,
    search_text: String,
    current_tab: Tab,
    selected_store: Option<Store>,
    previous_store_id: Option<String>,
    #[serde(skip)]
    tiles: Option<Box<dyn Tiles>>,
    #[serde(skip)]
    map_memory: MapMemory,
    products: Vec<Product>,
    current_location: (f64, f64),      // 当前位置 (纬度, 经度)
    selected_product: Option<Product>, // 选中的商品
    product_search_text: String,
    selected_category: Option<String>,
    #[serde(skip)]
    auth_ui: AuthUI, // Authentication UI component
    #[serde(skip)]
    alert_ui: AlertUI, // Alert UI component
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    scanner_ui: ScannerUI, // Scanner UI component
    #[serde(skip)]
    app_services: AppServices, // Business logic services
    #[serde(skip)]
    database_manager: Option<Arc<DatabaseManager>>, // Database connection
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
enum Tab {
    Stores,    // 门店管理
    Products,  // 商品比价
    Scanner,   // 条码扫描
    Alerts,    // 价格提醒
    Trends,    // 价格趋势
    Community, // 用户互动
    Settings,
}

impl Default for Tab {
    fn default() -> Self {
        Self::Stores
    }
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            stores: Self::create_sample_stores(),
            search_text: String::new(),
            current_tab: Tab::default(),
            selected_store: None,
            previous_store_id: None,
            tiles: None,
            map_memory: MapMemory::default(),
            products: Self::create_sample_products(),
            current_location: (35.6812, 139.7671), // 当前位置 (纬度, 经度)
            selected_product: None,                // 选中的商品
            product_search_text: String::new(),
            selected_category: None,
            auth_ui: AuthUI::new(),
            alert_ui: AlertUI::new(),
            #[cfg(not(target_arch = "wasm32"))]
            scanner_ui: ScannerUI::new(),
            app_services: AppServices::new(),
            database_manager: None,
        }
    }
}

impl TemplateApp {
    fn create_sample_stores() -> Vec<Store> {
        vec![
            Store {
                id: "1".to_string(),
                name: "全家便利店 - 东京站店".to_string(),
                address: "东京都千代田区丸の内1-9-1".to_string(),
                latitude: 35.6812,
                longitude: 139.7671,
                rating: 4.5,
                opening_hours: "24小时营业".to_string(),
                phone: "03-1234-5678".to_string(),
                tags: vec!["便利店".to_string(), "24小时".to_string()],
                symbol: '🏪',
                created_at: Utc::now(),
            },
            Store {
                id: "2".to_string(),
                name: "松本清 - 新宿店".to_string(),
                address: "东京都新宿区新宿3-1-1".to_string(),
                latitude: 35.6895,
                longitude: 139.6917,
                rating: 4.2,
                opening_hours: "10:00-22:00".to_string(),
                phone: "03-2345-6789".to_string(),
                tags: vec![
                    "药妆店".to_string(),
                    "化妆品".to_string(),
                    "免税".to_string(),
                ],
                symbol: '🏪',
                created_at: Utc::now(),
            },
            Store {
                id: "3".to_string(),
                name: "唐吉诃德 - 涩谷店".to_string(),
                address: "东京都涩谷区道玄坂2-25-5".to_string(),
                latitude: 35.6580,
                longitude: 139.6994,
                rating: 4.0,
                opening_hours: "24小时营业".to_string(),
                phone: "03-3456-7890".to_string(),
                tags: vec![
                    "综合商店".to_string(),
                    "免税".to_string(),
                    "24小时".to_string(),
                ],
                symbol: '🏪',
                created_at: Utc::now(),
            },
            Store {
                id: "4".to_string(),
                name: "无印良品 - 银座店".to_string(),
                address: "东京都中央区银座3-3-5".to_string(),
                latitude: 35.6721,
                longitude: 139.7636,
                rating: 4.3,
                opening_hours: "11:00-20:00".to_string(),
                phone: "03-4567-8901".to_string(),
                tags: vec![
                    "生活用品".to_string(),
                    "服装".to_string(),
                    "家居".to_string(),
                ],
                symbol: '🏪',
                created_at: Utc::now(),
            },
            Store {
                id: "5".to_string(),
                name: "优衣库 - 原宿店".to_string(),
                address: "东京都涩谷区神宫前1-14-30".to_string(),
                latitude: 35.6716,
                longitude: 139.7031,
                rating: 4.4,
                opening_hours: "10:00-21:00".to_string(),
                phone: "03-5678-9012".to_string(),
                tags: vec!["服装".to_string(), "时尚".to_string()],
                symbol: '🏪',
                created_at: Utc::now(),
            },
        ]
    }

    fn create_sample_products() -> Vec<Product> {
        vec![
            Product {
                id: "1".to_string(),
                name: "可口可乐".to_string(),
                category: "饮料".to_string(),
                description: "碳酸饮料，330ml".to_string(),
                barcode: Some("1234567890123".to_string()),
                images: vec!["cola.jpg".to_string()],
                prices: vec![PriceRecord {
                    id: Some("price1".to_string()),
                    product_id: Some("1".to_string()),
                    store_id: "1".to_string(),
                    user_id: None,
                    price: 3.5,
                    timestamp: Utc::now(),
                    is_on_sale: false,
                    receipt_image: None,
                    verification_status: "verified".to_string(),
                }],
                tags: vec!["饮料".to_string(), "碳酸".to_string()],
                created_at: Utc::now(),
            },
            Product {
                id: "2".to_string(),
                name: "百事可乐".to_string(),
                category: "饮料".to_string(),
                description: "碳酸饮料，330ml".to_string(),
                barcode: Some("1234567890124".to_string()),
                images: vec!["pepsi.jpg".to_string()],
                prices: vec![PriceRecord {
                    id: Some("price2".to_string()),
                    product_id: Some("2".to_string()),
                    store_id: "2".to_string(),
                    user_id: None,
                    price: 3.0,
                    timestamp: Utc::now(),
                    is_on_sale: true,
                    receipt_image: None,
                    verification_status: "verified".to_string(),
                }],
                tags: vec!["饮料".to_string(), "碳酸".to_string()],
                created_at: Utc::now(),
            },
        ]
    }

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 配置字体
        let mut fonts = egui::FontDefinitions::default();
        // 添加中文字体
        fonts.font_data.insert(
            "microsoft_yahei".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/simhei.ttf")).into(),
        );

        // 将中文字体设置为优先字体
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "microsoft_yahei".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        // 使用带默认值的结构体更新，避免后续字段再赋值
        let mut app = Self {
            tiles: Some(Box::new(HttpTiles::new(OpenStreetMap, cc.egui_ctx.clone()))),
            ..Self::default()
        };

        // Initialize database connection
        app.initialize_database();

        // Initialize services with sample data
        app.initialize_services();

        app
    }

    /// Initialize database connection
    fn initialize_database(&mut self) {
        // Try to initialize database connection
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(DatabaseManager::new_default()) {
            Ok(database_manager) => {
                let database_manager = Arc::new(database_manager);
                self.database_manager = Some(database_manager.clone());

                // Initialize AuthUI with database
                match AuthUI::with_database_sync(database_manager) {
                    Ok(auth_ui) => {
                        self.auth_ui = auth_ui;
                        log::info!("Database connection initialized successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to initialize AuthUI with database: {}", e);
                        // Keep default AuthUI without database
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to initialize database: {}", e);
                // Keep default AuthUI without database
            }
        }
    }

    /// Initialize services with sample data
    fn initialize_services(&mut self) {
        // Add sample stores
        for store in &self.stores {
            let _ = self.app_services.store_service.create_store(
                store.name.clone(),
                store.address.clone(),
                store.latitude,
                store.longitude,
                store.opening_hours.clone(),
                store.phone.clone(),
                store.tags.clone(),
                store.symbol,
            );
        }

        // Add sample products
        for product in &self.products {
            let _ = self.app_services.product_service.create_product(
                product.name.clone(),
                product.category.clone(),
                product.description.clone(),
                product.barcode.clone(),
                product.tags.clone(),
            );
        }
    }

    fn render_stores_tab(&mut self, ui: &mut egui::Ui) {
        // 搜索和筛选区域
        ui.vertical(|ui| {
            // 搜索栏占据整行
            ui.horizontal(|ui| {
                ui.label("搜索：");
                ui.add(egui::TextEdit::singleline(&mut self.search_text));
            });
        });

        ui.separator();
        let filtered_stores: Vec<_> = self
            .stores
            .iter()
            .filter(|store| {
                self.search_text.is_empty()
                    || store
                        .name
                        .to_lowercase()
                        .contains(&self.search_text.to_lowercase())
                    || store
                        .address
                        .to_lowercase()
                        .contains(&self.search_text.to_lowercase())
                    || store.tags.iter().any(|tag| {
                        tag.to_lowercase()
                            .contains(&self.search_text.to_lowercase())
                    })
            })
            .collect();
        ui.with_layout(
            egui::Layout::left_to_right(egui::Align::TOP).with_cross_justify(true),
            |ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width() * 0.4, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.heading("附近商店");
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            egui_extras::TableBuilder::new(ui)
                                .striped(true)
                                .column(
                                    egui_extras::Column::initial(100.0)
                                        .at_least(40.0)
                                        .clip(true)
                                        .resizable(true),
                                )
                                .column(
                                    egui_extras::Column::initial(100.0)
                                        .at_least(40.0)
                                        .clip(true)
                                        .resizable(true),
                                )
                                .column(
                                    egui_extras::Column::initial(100.0)
                                        .at_least(40.0)
                                        .clip(true)
                                        .resizable(true),
                                )
                                .column(
                                    egui_extras::Column::initial(100.0)
                                        .at_least(60.0)
                                        .clip(true)
                                        .resizable(true),
                                )
                                .column(
                                    egui_extras::Column::initial(100.0)
                                        .at_least(40.0)
                                        .clip(true)
                                        .resizable(true),
                                )
                                .header(20.0, |mut header| {
                                    header.col(|ui| {
                                        ui.label("店名");
                                    });
                                    header.col(|ui| {
                                        ui.label("距离");
                                    });
                                    header.col(|ui| {
                                        ui.label("评分");
                                    });
                                    header.col(|ui| {
                                        ui.label("营业时间");
                                    });
                                    header.col(|ui| {
                                        ui.label("标签");
                                    });
                                })
                                .body(|mut body| {
                                    for store in filtered_stores.iter() {
                                        let is_selected =
                                            self.selected_store.as_ref() == Some(store);
                                        let distance = store.distance_to(
                                            self.current_location.0,
                                            self.current_location.1,
                                        );
                                        body.row(20.0, |mut row| {
                                            row.col(|ui| {
                                                if ui
                                                    .selectable_label(is_selected, &store.name)
                                                    .clicked()
                                                {
                                                    self.selected_store = Some((*store).clone());
                                                }
                                            });
                                            row.col(|ui| {
                                                ui.label(format!("{:.1}km", distance));
                                            });
                                            row.col(|ui| {
                                                ui.label(format!("{:.1}分", store.rating));
                                            });
                                            row.col(|ui| {
                                                ui.label(&store.opening_hours);
                                            });
                                            row.col(|ui| {
                                                ui.label(store.tags.join("、"));
                                            });
                                        });
                                    }
                                });
                        });
                    },
                );
            },
        );

        // 地图区域
        if let Some(selected_store) = &self.selected_store {
            if let Some(tiles) = &mut self.tiles {
                egui::Window::new("地图").show(ui.ctx(), |ui| {
                    let store_pos =
                        Position::new(selected_store.longitude, selected_store.latitude);
                    let places = Places::new(
                        filtered_stores
                            .iter()
                            .map(|store| LabeledSymbol {
                                position: Position::new(store.longitude, store.latitude),
                                label: store.name.clone(),
                                symbol: Some(Symbol::Circle("🏪".to_string())),
                                style: LabeledSymbolStyle::default(),
                            })
                            .collect(),
                    );
                    if self.previous_store_id.as_ref() != Some(&selected_store.id) {
                        self.map_memory.center_at(store_pos);
                        self.previous_store_id = Some(selected_store.id.clone());
                    }
                    ui.add(
                        Map::new(Some(tiles.as_mut()), &mut self.map_memory, store_pos)
                            .with_plugin(places),
                    );
                    // 在地图右上角添加控制按钮
                    let map_rect = ui.max_rect();
                    let button_size = egui::vec2(32.0, 32.0);
                    let padding = 8.0;
                    // 缩放按钮
                    let zoom_in_rect = egui::Rect::from_min_size(
                        map_rect.right_top() + egui::vec2(-button_size.x - padding, padding),
                        button_size,
                    );
                    let zoom_out_rect = egui::Rect::from_min_size(
                        map_rect.right_top()
                            + egui::vec2(-button_size.x - padding, button_size.y + padding * 2.0),
                        button_size,
                    );
                    // 定位按钮
                    let location_rect = egui::Rect::from_min_size(
                        map_rect.right_top()
                            + egui::vec2(
                                -button_size.x - padding,
                                button_size.y * 2.0 + padding * 3.0,
                            ),
                        button_size,
                    );

                    if ui.put(zoom_in_rect, egui::Button::new("➕")).clicked() {
                        let _ = self.map_memory.zoom_in();
                    }
                    if ui.put(zoom_out_rect, egui::Button::new("➖")).clicked() {
                        let _ = self.map_memory.zoom_out();
                    }
                    if ui.put(location_rect, egui::Button::new("📍")).clicked() {
                        self.map_memory.center_at(store_pos);
                    }
                });
            }
        }
    }

    fn render_products_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("搜索商品：");
            ui.text_edit_singleline(&mut self.product_search_text);

            // 分类过滤
            egui::ComboBox::from_label("分类")
                .selected_text(self.selected_category.as_deref().unwrap_or("全部"))
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(self.selected_category.is_none(), "全部")
                        .clicked()
                    {
                        self.selected_category = None;
                    }
                    let categories: Vec<_> = self
                        .products
                        .iter()
                        .map(|p| p.category.as_str())
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect();
                    for category in categories {
                        if ui
                            .selectable_label(
                                self.selected_category.as_deref() == Some(category),
                                category,
                            )
                            .clicked()
                        {
                            self.selected_category = Some(category.to_string());
                        }
                    }
                });
        });

        ui.separator();

        // 商品列表
        egui::ScrollArea::vertical().show(ui, |ui| {
            // 过滤商品
            let filtered_products: Vec<_> = self
                .products
                .iter()
                .filter(|p| {
                    let matches_search = self.product_search_text.is_empty()
                        || p.name
                            .to_lowercase()
                            .contains(&self.product_search_text.to_lowercase())
                        || p.description
                            .to_lowercase()
                            .contains(&self.product_search_text.to_lowercase())
                        || p.tags.iter().any(|t| {
                            t.to_lowercase()
                                .contains(&self.product_search_text.to_lowercase())
                        });

                    let matches_category = self
                        .selected_category
                        .as_ref()
                        .is_none_or(|c| p.category == *c);

                    matches_search && matches_category
                })
                .collect();

            // 显示商品表格
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing.x = 10.0;
                ui.label("商品名称");
                ui.label("分类");
                ui.label("最低价格");
                ui.label("价格范围");
                ui.label("标签");
            });
            ui.separator();

            for product in filtered_products {
                let lowest_price = product.current_lowest_price();
                let price_range = self.get_price_range(product);

                ui.horizontal(|ui| {
                    let selected_product_id = self.selected_product.as_ref().map(|p| p.id.clone());
                    if ui
                        .selectable_label(
                            selected_product_id.as_ref() == Some(&product.id),
                            &product.name,
                        )
                        .clicked()
                    {
                        self.selected_product = Some(product.clone());
                    }
                    ui.label(&product.category);
                    ui.label(format!("¥{:.2}", lowest_price.map_or(0.0, |p| p.price)));
                    ui.label(format!("¥{:.2} - ¥{:.2}", price_range.0, price_range.1));
                    ui.label(product.tags.join("、"));
                });
            }
        });

        // 如果选中了商品，显示详情
        if let Some(selected_product) = &self.selected_product {
            self.show_product_detail(ui, selected_product);
        }
    }

    fn get_price_range(&self, product: &Product) -> (f64, f64) {
        let prices: Vec<_> = product.prices.iter().map(|p| p.price).collect();
        match (
            prices.iter().min_by(|a, b| a.partial_cmp(b).unwrap()),
            prices.iter().max_by(|a, b| a.partial_cmp(b).unwrap()),
        ) {
            (Some(min), Some(max)) => (*min, *max),
            _ => (0.0, 0.0),
        }
    }

    fn show_product_detail(&self, ui: &mut egui::Ui, product: &Product) {
        egui::Window::new("商品详情").show(ui.ctx(), |ui| {
            ui.heading(&product.name);
            ui.label(&product.description);

            ui.separator();

            // 价格历史
            ui.heading("价格历史");
            let mut prices: Vec<_> = product.prices.iter().collect();
            prices.sort_by_key(|p| p.timestamp);

            for price in prices {
                let store = self
                    .stores
                    .iter()
                    .find(|s| s.id == price.store_id)
                    .map(|s| s.name.as_str())
                    .unwrap_or("未知商店");

                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{} - ¥{:.2} {}",
                        price.timestamp.format("%Y-%m-%d"),
                        price.price,
                        if price.is_on_sale { "[特价]" } else { "" }
                    ));
                    ui.label(store);
                });
            }
        });
    }

    fn render_community_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("用户互动与评价系统");

        if !self.auth_ui.is_logged_in() {
            ui.colored_label(egui::Color32::YELLOW, "请先登录以使用评价功能");
            return;
        }

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("最新评价");

                match self.app_services.review_service.get_recent_reviews(0, 5) {
                    Ok(reviews) => {
                        if reviews.is_empty() {
                            ui.label("暂无评价");
                        } else {
                            for review in reviews {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("⭐ {}/5", review.rating));
                                        ui.label(review.created_at.format("%m-%d").to_string());
                                    });
                                    ui.label(&review.comment);

                                    if let Some(ref store_id) = review.store_id {
                                        if let Some(store) =
                                            self.stores.iter().find(|s| s.id == *store_id)
                                        {
                                            ui.small(format!("店铺: {}", store.name));
                                        }
                                    }

                                    if let Some(ref product_id) = review.product_id {
                                        if let Some(product) =
                                            self.products.iter().find(|p| p.id == *product_id)
                                        {
                                            ui.small(format!("商品: {}", product.name));
                                        }
                                    }
                                });
                                ui.add_space(4.0);
                            }
                        }
                    }
                    Err(_) => {
                        ui.colored_label(egui::Color32::RED, "加载评价失败");
                    }
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("系统统计");

                match self.app_services.review_service.get_review_stats() {
                    Ok(stats) => {
                        ui.label(format!("总评价数: {}", stats.total_reviews));
                        ui.label(format!("店铺评价: {}", stats.store_reviews));
                        ui.label(format!("商品评价: {}", stats.product_reviews));
                        ui.label(format!("平均评分: {:.1}", stats.average_rating));
                        ui.label(format!("活跃用户: {}", stats.unique_reviewers));

                        ui.separator();
                        ui.heading("评分分布");
                        let dist = &stats.rating_distribution;
                        ui.label(format!("⭐⭐⭐⭐⭐: {}", dist.five_star));
                        ui.label(format!("⭐⭐⭐⭐: {}", dist.four_star));
                        ui.label(format!("⭐⭐⭐: {}", dist.three_star));
                        ui.label(format!("⭐⭐: {}", dist.two_star));
                        ui.label(format!("⭐: {}", dist.one_star));
                    }
                    Err(_) => {
                        ui.colored_label(egui::Color32::RED, "加载统计数据失败");
                    }
                }
            });
        });

        ui.separator();

        // Demo review submission (for testing)
        if ui.button("添加测试评价").clicked() {
            if let Some(current_user) = self.auth_ui.get_current_user() {
                if !self.stores.is_empty() {
                    let store = &self.stores[0];
                    let _ = self.app_services.review_service.submit_review(
                        current_user.id.clone(),
                        Some(store.id.clone()),
                        None,
                        4,
                        "这是一个测试评价，服务不错！".to_string(),
                    );
                }
            }
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 顶部导航栏
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("退出").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.add_space(16.0);

                // Authentication menu
                ui.menu_button("用户", |ui| {
                    if self.auth_ui.is_logged_in() {
                        if let Some(user) = self.auth_ui.get_current_user() {
                            ui.label(format!("欢迎, {}!", user.username));
                            ui.separator();
                        }

                        if ui.button("用户信息").clicked() {
                            self.auth_ui.open_auth_window();
                        }

                        if ui.button("退出登录").clicked() {
                            self.auth_ui.handle_logout();
                        }
                    } else {
                        if ui.button("登录").clicked() {
                            self.auth_ui.open_auth_window();
                        }

                        if ui.button("注册").clicked() {
                            self.auth_ui.open_auth_window();
                            self.auth_ui.auth_state = AuthState::Registering;
                        }
                    }
                });

                // Service statistics
                if self.auth_ui.is_logged_in() {
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Show service statistics
                    if let Ok(user_stats) = self.app_services.user_service.get_user_stats() {
                        ui.label(format!("用户: {}", user_stats.total_users));
                    }

                    if let Ok(product_stats) = self.app_services.product_service.get_product_stats()
                    {
                        ui.label(format!("商品: {}", product_stats.total_products));
                    }

                    if let Ok(store_stats) = self.app_services.store_service.get_store_stats() {
                        ui.label(format!("店铺: {}", store_stats.total_stores));
                    }
                }
            });
        });

        // 侧边栏
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("功能导航");
            ui.separator();

            if ui
                .selectable_label(self.current_tab == Tab::Stores, "门店管理")
                .clicked()
            {
                self.current_tab = Tab::Stores;
            }
            if ui
                .selectable_label(self.current_tab == Tab::Products, "商品比价")
                .clicked()
            {
                self.current_tab = Tab::Products;
            }
            #[cfg(not(target_arch = "wasm32"))]
            if ui
                .selectable_label(self.current_tab == Tab::Scanner, "条码扫描")
                .clicked()
            {
                self.current_tab = Tab::Scanner;
            }
            if ui
                .selectable_label(self.current_tab == Tab::Alerts, "价格提醒")
                .clicked()
            {
                self.current_tab = Tab::Alerts;
            }
            if ui
                .selectable_label(self.current_tab == Tab::Trends, "价格趋势")
                .clicked()
            {
                self.current_tab = Tab::Trends;
            }
            if ui
                .selectable_label(self.current_tab == Tab::Community, "用户互动")
                .clicked()
            {
                self.current_tab = Tab::Community;
            }
        });

        // 主内容区
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Stores => self.render_stores_tab(ui),
                Tab::Products => self.render_products_tab(ui),
                #[cfg(not(target_arch = "wasm32"))]
                Tab::Scanner => {
                    self.scanner_ui.show(ctx, ui);
                }
                #[cfg(target_arch = "wasm32")]
                Tab::Scanner => {
                    ui.heading("Scanner");
                    ui.label("Scanner functionality is only available on desktop platforms.");
                }
                Tab::Alerts => {
                    if let Some(current_user) = self.auth_ui.get_current_user() {
                        self.alert_ui.show(ui, &current_user.id);
                    } else {
                        ui.heading("价格提醒");
                        ui.colored_label(egui::Color32::YELLOW, "请先登录以使用价格提醒功能");
                        ui.label("登录后您可以设置价格提醒，当商品价格达到您设定的目标价格时会收到通知。");
                    }
                }
                Tab::Trends => {
                    self.render_trends_tab(ui);
                }
                Tab::Community => {
                    self.render_community_tab(ui);
                }
                Tab::Settings => {
                    ui.heading("设置");
                    ui.label("在这里可以设置应用的配置");
                    // TODO: 添加设置功能
                }
            }
        });

        // Render authentication UI
        self.auth_ui.show_auth_dialog(ctx);
    }
}

impl TemplateApp {
    /// Render the trends tab with price visualization
    fn render_trends_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("价格趋势分析");
        ui.label("商品价格历史走势分析");
        ui.separator();

        // Product selection for trend analysis
        ui.horizontal(|ui| {
            ui.label("选择商品:");
            egui::ComboBox::from_label("")
                .selected_text(
                    self.selected_product
                        .as_ref()
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| "请选择商品".to_string()),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_product, None, "无选择");
                    for product in &self.products {
                        ui.selectable_value(
                            &mut self.selected_product,
                            Some(product.clone()),
                            &product.name,
                        );
                    }
                });
        });

        ui.separator();

        if let Some(selected_product) = &self.selected_product {
            self.render_price_trends_for_product(ui, selected_product);
        } else {
            ui.label("请选择一个商品以查看价格趋势");

            // Show general market trends when no product is selected
            self.render_market_overview(ui);
        }
    }

    /// Render price trends for a specific product
    fn render_price_trends_for_product(&self, ui: &mut egui::Ui, product: &Product) {
        ui.heading(format!("{}的价格趋势", product.name));

        // Price statistics
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("价格统计");

                if let Ok(stats) = self
                    .app_services
                    .price_service
                    .get_price_statistics(&product.id)
                {
                    ui.label(format!("最低价: ¥{:.2}", stats.min_price));
                    ui.label(format!("最高价: ¥{:.2}", stats.max_price));
                    ui.label(format!("平均价: ¥{:.2}", stats.avg_price));
                    ui.label(format!("中位数: ¥{:.2}", stats.median_price));
                    ui.label(format!("价格记录数: {}", stats.total_records));
                    ui.label(format!("覆盖店铺数: {}", stats.stores_count));
                    ui.label(format!("促销比例: {:.1}%", stats.sale_percentage));
                } else {
                    ui.label("暂无价格统计数据");
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label("价格历史");

                // Simple text-based price history visualization
                let mut prices: Vec<_> = product.prices.iter().collect();
                prices.sort_by_key(|p| p.timestamp);

                if prices.is_empty() {
                    ui.label("暂无价格数据");
                } else {
                    // Calculate price change
                    let first_price = prices.first().unwrap().price;
                    let last_price = prices.last().unwrap().price;
                    let price_change = last_price - first_price;
                    let price_change_percent = (price_change / first_price) * 100.0;

                    ui.label(format!(
                        "价格变化: ¥{:.2} ({:+.1}%)",
                        price_change, price_change_percent
                    ));

                    let trend_color = if price_change > 0.0 {
                        egui::Color32::RED
                    } else if price_change < 0.0 {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::GRAY
                    };

                    let trend_text = if price_change > 0.0 {
                        "↗ 上涨趋势"
                    } else if price_change < 0.0 {
                        "↘ 下降趋势"
                    } else {
                        "→ 价格稳定"
                    };

                    ui.colored_label(trend_color, trend_text);
                }
            });
        });

        ui.separator();

        // Price history chart (simplified visualization)
        self.render_price_chart(ui, product);

        ui.separator();

        // Store-wise price comparison
        self.render_store_price_comparison(ui, product);
    }

    /// Render a simple price chart using egui
    fn render_price_chart(&self, ui: &mut egui::Ui, product: &Product) {
        ui.label("价格走势图");

        let mut prices: Vec<_> = product.prices.iter().collect();
        prices.sort_by_key(|p| p.timestamp);

        if prices.is_empty() {
            ui.label("暂无数据可显示");
            return;
        }

        // Create a simple line chart
        let chart_rect = ui.available_rect_before_wrap();
        let chart_rect =
            egui::Rect::from_min_size(chart_rect.min, egui::vec2(chart_rect.width(), 200.0));

        ui.allocate_rect(chart_rect, egui::Sense::hover());

        let painter = ui.ctx().layer_painter(egui::LayerId::new(
            egui::Order::Background,
            egui::Id::new("price_chart"),
        ));

        // Draw chart background
        painter.rect_filled(
            chart_rect,
            egui::CornerRadius::same(4),
            egui::Color32::from_rgb(240, 240, 240),
        );

        if prices.len() > 1 {
            let min_price = prices.iter().map(|p| p.price).fold(f64::INFINITY, f64::min);
            let max_price = prices
                .iter()
                .map(|p| p.price)
                .fold(f64::NEG_INFINITY, f64::max);
            let price_range = (max_price - min_price).max(0.01); // Avoid division by zero

            // Draw price line
            let points: Vec<egui::Pos2> = prices
                .iter()
                .enumerate()
                .map(|(i, price_record)| {
                    let x = chart_rect.min.x
                        + (i as f32 / (prices.len() - 1) as f32) * chart_rect.width();
                    let y = chart_rect.max.y
                        - ((price_record.price - min_price) / price_range) as f32
                            * chart_rect.height();
                    egui::pos2(x, y)
                })
                .collect();

            // Draw the price line
            for i in 1..points.len() {
                painter.line_segment(
                    [points[i - 1], points[i]],
                    egui::Stroke::new(2.0, egui::Color32::BLUE),
                );
            }

            // Draw price points
            for (i, point) in points.iter().enumerate() {
                let color = if prices[i].is_on_sale {
                    egui::Color32::RED // Red for sale prices
                } else {
                    egui::Color32::BLUE // Blue for regular prices
                };

                painter.circle_filled(*point, 3.0, color);
            }

            // Draw price labels
            painter.text(
                egui::pos2(chart_rect.min.x + 5.0, chart_rect.min.y + 5.0),
                egui::Align2::LEFT_TOP,
                format!("最高: ¥{:.2}", max_price),
                egui::FontId::default(),
                egui::Color32::BLACK,
            );

            painter.text(
                egui::pos2(chart_rect.min.x + 5.0, chart_rect.max.y - 20.0),
                egui::Align2::LEFT_BOTTOM,
                format!("最低: ¥{:.2}", min_price),
                egui::FontId::default(),
                egui::Color32::BLACK,
            );
        }

        ui.allocate_space(egui::vec2(0.0, 200.0)); // Reserve space for the chart
    }

    /// Render store-wise price comparison
    fn render_store_price_comparison(&self, ui: &mut egui::Ui, product: &Product) {
        ui.label("各店铺价格对比");

        let mut store_prices: std::collections::HashMap<String, Vec<&PriceRecord>> =
            std::collections::HashMap::new();

        for price in &product.prices {
            store_prices
                .entry(price.store_id.clone())
                .or_default()
                .push(price);
        }

        if store_prices.is_empty() {
            ui.label("暂无价格数据");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (store_id, prices) in store_prices {
                let store_name = self
                    .stores
                    .iter()
                    .find(|s| s.id == store_id)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| format!("未知店铺 ({})", store_id));

                // Get the latest price for this store
                let latest_price = prices.iter().max_by_key(|p| p.timestamp).unwrap();

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(&store_name);
                            ui.label(format!("当前价格: ¥{:.2}", latest_price.price));
                            if latest_price.is_on_sale {
                                ui.colored_label(egui::Color32::RED, "[促销中]");
                            }
                            ui.label(format!(
                                "更新时间: {}",
                                latest_price.timestamp.format("%Y-%m-%d %H:%M")
                            ));
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format!("{} 条记录", prices.len()));
                        });
                    });
                });
            }
        });
    }

    /// Render market overview when no specific product is selected
    fn render_market_overview(&self, ui: &mut egui::Ui) {
        ui.separator();
        ui.heading("市场概览");

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("热门商品");

                // Show trending products
                if let Ok(trending) = self.app_services.price_service.get_trending_prices(5) {
                    for trend in trending {
                        ui.group(|ui| {
                            if let Some(product) =
                                self.products.iter().find(|p| p.id == trend.product_id)
                            {
                                ui.horizontal(|ui| {
                                    ui.label(&product.name);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(format!("¥{:.2}", trend.latest_price));
                                            ui.label(format!("({} 次更新)", trend.activity_count));
                                        },
                                    );
                                });
                            }
                        });
                    }
                } else {
                    ui.label("暂无热门商品数据");
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label("价格动态");

                // Show recent price changes
                let mut all_prices: Vec<&PriceRecord> =
                    self.products.iter().flat_map(|p| &p.prices).collect();

                all_prices.sort_by_key(|p| std::cmp::Reverse(p.timestamp));
                all_prices.truncate(5);

                for price in all_prices {
                    if let Some(product) = self
                        .products
                        .iter()
                        .find(|p| p.id == *price.product_id.as_ref().unwrap_or(&String::new()))
                    {
                        let store_name = self
                            .stores
                            .iter()
                            .find(|s| s.id == price.store_id)
                            .map(|s| s.name.clone())
                            .unwrap_or_else(|| "未知店铺".to_string());

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(&product.name);
                                    ui.label(&store_name);
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(format!("¥{:.2}", price.price));
                                        ui.label(price.timestamp.format("%m-%d %H:%M").to_string());
                                    },
                                );
                            });
                        });
                    }
                }
            });
        });
    }
}
