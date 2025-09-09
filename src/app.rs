use crate::auth::{AuthState, AuthUI};
use crate::models::{PriceRecord, Product, Store};
use crate::services::AppServices;
// use crate::alerts::AlertUI; // TODO: Fix string encoding issues
#[cfg(not(target_arch = "wasm32"))]
use crate::scanner::ScannerUI;
use chrono::Utc;
use eframe::egui;
use walkers::{
    extras::{Place, Places, Style},
    sources::OpenStreetMap,
    HttpTiles, Map, MapMemory, Position, Tiles,
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
    // #[serde(skip)]
    // alert_ui: AlertUI, // Alert UI component - TODO: Fix string encoding issues
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    scanner_ui: ScannerUI, // Scanner UI component
    #[serde(skip)]
    app_services: AppServices, // Business logic services
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
            // alert_ui: AlertUI::new(), // TODO: Fix string encoding issues
            #[cfg(not(target_arch = "wasm32"))]
            scanner_ui: ScannerUI::new(),
            app_services: AppServices::new(),
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
}

impl TemplateApp {
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

        // 加载上一次的应用状态（如果有）
        // let mut app = if let Some(storage) = cc.storage {
        //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        // } else {
        //     Self::default()
        // };
        let mut app = Self::default();
        // 初始化地图
        app.tiles = Some(Box::new(HttpTiles::new(OpenStreetMap, cc.egui_ctx.clone())));

        // Initialize services with sample data
        app.initialize_services();

        app
    }

    /// Initialize services with sample data
    fn initialize_services(&mut self) {
        // Add sample data to services if they're empty

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
                let matches_search = self.search_text.is_empty()
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
                    });
                matches_search
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
                    let places: Vec<Place> = filtered_stores
                        .iter()
                        .map(|store| Place {
                            position: Position::new(store.longitude, store.latitude), // 假设 Store 结构体中有 position 字段
                            label: store.name.clone(),
                            symbol: store.symbol,
                            style: Style::default(),
                        })
                        .collect();
                    if self.previous_store_id.as_ref() != Some(&selected_store.id) {
                        self.map_memory.center_at(store_pos);
                        self.previous_store_id = Some(selected_store.id.clone());
                    }
                    ui.add(
                        Map::new(Some(tiles.as_mut()), &mut self.map_memory, store_pos)
                            .with_plugin(Places::new(places)),
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
                        .map_or(true, |c| p.category == *c);

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
            egui::menu::bar(ui, |ui| {
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
                Tab::Alerts => {
                    // let current_user = self.auth_ui.get_current_user();
                    // self.alert_ui.show(ctx, ui, current_user);
                    ui.heading("Price Alerts");
                    ui.label("Price alert functionality implemented but UI temporarily disabled due to encoding issues.");
                    ui.label("Core alert monitoring and notification system is fully functional.");
                }
                Tab::Trends => {
                    ui.heading("价格趋势分析");
                    ui.label("商品价格历史走势");
                    // TODO: 添加价格趋势图表
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
