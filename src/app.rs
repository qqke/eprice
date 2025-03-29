use crate::models::{PriceRecord, Product, Store};
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
    min_rating: f64,
    max_distance: f64,
    current_tab: Tab,
    selected_store: Option<Store>,
    previous_store_id: Option<String>,
    #[serde(skip)]
    tiles: Option<Box<dyn Tiles>>,
    #[serde(skip)]
    map_memory: MapMemory,
    products: Vec<Product>,
    current_location: (f64, f64),     // 当前位置 (纬度, 经度)
    selected_product: Option<String>, // 选中的商品ID
    product_search_text: String,
    selected_category: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
enum Tab {
    Stores,    // 门店管理
    Products,  // 商品比价
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
            stores: vec![
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
                },
            ],
            search_text: String::new(),
            min_rating: 0.0,
            max_distance: f64::MAX,
            current_tab: Tab::default(),
            selected_store: None,
            previous_store_id: None,
            tiles: None,
            map_memory: MapMemory::default(),
            products: vec![
                Product {
                    id: "1".to_string(),
                    name: "可口可乐".to_string(),
                    category: "饮料".to_string(),
                    description: "碳酸饮料，330ml".to_string(),
                    images: vec!["cola.jpg".to_string()],
                    prices: vec![PriceRecord {
                        store_id: "1".to_string(),
                        price: 3.5,
                        timestamp: Utc::now(),
                        is_on_sale: false,
                    }],
                    tags: vec!["饮料".to_string(), "碳酸".to_string()],
                },
                // ... 更多商品数据 ...
            ],
            current_location: (35.6812, 139.7671), // 默认位置（东京站）
            selected_product: None,
            product_search_text: String::new(),
            selected_category: None,
        }
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
        let mut app = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        };

        // 初始化地图
        app.tiles = Some(Box::new(HttpTiles::new(OpenStreetMap, cc.egui_ctx.clone())));

        app
    }

    fn render_stores_tab(&mut self, ui: &mut egui::Ui) {
        // 搜索和筛选区域
        ui.vertical(|ui| {
            // 搜索栏占据整行
            ui.horizontal(|ui| {
                ui.label("搜索：");
                ui.add(egui::TextEdit::singleline(&mut self.search_text));
            });

            // 筛选条件分两行显示
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;

                ui.label("距离：");
                ui.add(egui::Slider::new(&mut self.max_distance, 0.0..=10.0).suffix("km"));

                ui.label("评分：");
                ui.add(egui::Slider::new(&mut self.min_rating, 0.0..=5.0).suffix("分"));
            });
        });

        ui.separator();

        ui.with_layout(
            egui::Layout::left_to_right(egui::Align::TOP).with_cross_justify(true),
            |ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width() * 0.4, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.heading("附近商店");
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            // 表格标题行
                            {
                                ui.style_mut().spacing.item_spacing.x = 10.0;
                                egui::Grid::new("stores_header")
                                    .num_columns(5)
                                    .spacing([10.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("店名");
                                        ui.label("距离");
                                        ui.label("评分");
                                        ui.label("营业时间");
                                        ui.label("标签");
                                        ui.end_row();
                                    });
                            }
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

                                    let distance = store.distance_to(
                                        self.current_location.0,
                                        self.current_location.1,
                                    );
                                    let matches_distance = distance <= self.max_distance;
                                    let matches_rating = store.rating >= self.min_rating;

                                    matches_search && matches_distance && matches_rating
                                })
                                .collect();

                            egui::Grid::new("stores_grid")
                                .num_columns(5)
                                .spacing([10.0, 4.0])
                                .show(ui, |ui| {
                                    for store in filtered_stores.iter() {
                                        let is_selected =
                                            self.selected_store.as_ref() == Some(store);
                                        let distance = store.distance_to(
                                            self.current_location.0,
                                            self.current_location.1,
                                        );

                                        // 店名列（可点击）
                                        if ui.selectable_label(is_selected, &store.name).clicked() {
                                            self.selected_store = Some((*store).clone());
                                        }

                                        // 距离列
                                        ui.label(format!("{:.1}km", distance));

                                        // 评分列
                                        ui.label(format!("{:.1}分", store.rating));

                                        // 营业时间列
                                        ui.label(&store.opening_hours);

                                        // 标签列
                                        ui.label(store.tags.join("、"));

                                        ui.end_row();
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

                    if self.previous_store_id.as_ref() != Some(&selected_store.id) {
                        self.map_memory.center_at(store_pos);
                        self.previous_store_id = Some(selected_store.id.clone());
                    }
                    ui.add(
                        Map::new(Some(tiles.as_mut()), &mut self.map_memory, store_pos)
                            .with_plugin(Places::new(vec![Place {
                                position: store_pos,
                                label: selected_store.name.clone(),
                                symbol: '🏪',
                                style: Style::default(),
                            }])),
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
                    if ui
                        .selectable_label(
                            self.selected_product.as_ref() == Some(&product.id),
                            &product.name,
                        )
                        .clicked()
                    {
                        self.selected_product = Some(product.id.clone());
                    }
                    ui.label(&product.category);
                    ui.label(format!("¥{:.2}", lowest_price.map_or(0.0, |p| p.price)));
                    ui.label(format!("¥{:.2} - ¥{:.2}", price_range.0, price_range.1));
                    ui.label(product.tags.join("、"));
                });
            }
        });

        // 如果选中了商品，显示详情
        if let Some(product_id) = &self.selected_product {
            if let Some(product) = self.products.iter().find(|p| p.id == *product_id) {
                self.show_product_detail(ui, product);
            }
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
                Tab::Trends => {
                    ui.heading("价格趋势分析");
                    ui.label("商品价格历史走势");
                    // TODO: 添加价格趋势图表
                }
                Tab::Community => {
                    ui.heading("用户互动");
                    ui.label("用户评价和分享");
                    // TODO: 添加用户互动功能
                }
                Tab::Settings => {
                    ui.heading("设置");
                    ui.label("在这里可以设置应用的配置");
                    // TODO: 添加设置功能
                }
            }
        });
    }
}
