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
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct Store {
    pub id: String,
    pub name: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub rating: f64,
    pub distance: f64,
    pub opening_hours: String,
    pub phone: String,
    pub tags: Vec<String>,
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
                    distance: 0.2,
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
                    distance: 0.5,
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
                    distance: 1.2,
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
                    distance: 0.8,
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
                    distance: 1.5,
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

                let matches_distance = store.distance <= self.max_distance;
                let matches_rating = store.rating >= self.min_rating;

                matches_search && matches_distance && matches_rating
            })
            .collect();

        egui::Grid::new("my_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.heading("附近商店");
                ui.end_row();
                for store in filtered_stores.iter() {
                    let is_selected = self.selected_store.as_ref() == Some(store);
                    let response = ui.selectable_label(
                        is_selected,
                        format!(
                            "{} - {:.1}km - {:.1}分",
                            store.name, store.distance, store.rating
                        ),
                    );
                    if response.clicked() {
                        self.selected_store = Some((*store).clone());
                    }

                    response.on_hover_text(format!(
                        "地址：{}\n营业时间：{}\n电话：{}\n评分：{:.1}\n距离：{:.1}km\n标签：{}",
                        store.address,
                        store.opening_hours,
                        store.phone,
                        store.rating,
                        store.distance,
                        store.tags.join("、")
                    ));
                    ui.end_row();
                }
            });
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
                Tab::Products => {
                    ui.heading("商品比价");
                    ui.label("商品搜索和价格对比");
                    // TODO: 添加商品搜索和价格对比功能
                }
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
