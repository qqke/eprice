/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // 当前选中的标签页
    current_tab: Tab,
    // 门店管理相关状态
    store_search_text: String,
    store_filter_distance: f32,
    store_filter_rating: f32,
    stores: Vec<Store>,
    selected_store: Option<usize>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct Store {
    id: usize,
    name: String,
    address: String,
    latitude: f64,
    longitude: f64,
    rating: f32,
    distance: f32,
    opening_hours: String,
    phone: String,
    tags: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
enum Tab {
    Stores,    // 门店管理
    Products,  // 商品比价
    Trends,    // 价格趋势
    Community, // 用户互动
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            current_tab: Tab::Stores,
            store_search_text: String::new(),
            store_filter_distance: 5.0,
            store_filter_rating: 3.0,
            stores: vec![
                Store {
                    id: 1,
                    name: "示例商店1".to_string(),
                    address: "东京都新宿区".to_string(),
                    latitude: 35.6895,
                    longitude: 139.6917,
                    rating: 4.5,
                    distance: 0.5,
                    opening_hours: "10:00-22:00".to_string(),
                    phone: "03-1234-5678".to_string(),
                    tags: vec!["便利店".to_string(), "24小时".to_string()],
                },
                Store {
                    id: 2,
                    name: "示例商店2".to_string(),
                    address: "东京都涩谷区".to_string(),
                    latitude: 35.6580,
                    longitude: 139.7016,
                    rating: 4.0,
                    distance: 1.2,
                    opening_hours: "09:00-21:00".to_string(),
                    phone: "03-8765-4321".to_string(),
                    tags: vec!["超市".to_string(), "生鲜".to_string()],
                },
            ],
            selected_store: None,
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

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn render_stores_tab(&mut self, ui: &mut egui::Ui) {
        // 搜索和筛选区域
        ui.horizontal(|ui| {
            ui.label("搜索：");
            ui.text_edit_singleline(&mut self.store_search_text);
            ui.label("距离(km)：");
            ui.add(egui::Slider::new(
                &mut self.store_filter_distance,
                0.0..=10.0,
            ));
            ui.label("评分：");
            ui.add(egui::Slider::new(&mut self.store_filter_rating, 0.0..=5.0));
        });

        ui.separator();

        // 商店列表和地图区域
        ui.horizontal(|ui| {
            // 左侧商店列表
            ui.vertical(|ui| {
                ui.heading("附近商店");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, store) in self.stores.iter().enumerate() {
                        let is_selected = self.selected_store == Some(index);
                        if ui
                            .selectable_label(
                                is_selected,
                                format!("{} - {}", store.name, store.address),
                            )
                            .clicked()
                        {
                            self.selected_store = Some(index);
                        }
                    }
                });
            });

            ui.separator();

            // 右侧地图和商店详情
            ui.vertical(|ui| {
                // 地图区域（占位）
                let size = egui::vec2(400.0, 300.0);
                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, 0.0, egui::Color32::from_rgb(200, 200, 200));
                ui.label("地图显示区域");

                // 商店详情
                if let Some(selected_index) = self.selected_store {
                    if let Some(store) = self.stores.get(selected_index) {
                        ui.separator();
                        ui.heading(&store.name);
                        ui.label(format!("地址：{}", store.address));
                        ui.label(format!("营业时间：{}", store.opening_hours));
                        ui.label(format!("电话：{}", store.phone));
                        ui.label(format!("评分：{:.1}", store.rating));
                        ui.label(format!("距离：{:.1}km", store.distance));

                        ui.label("标签：");
                        ui.horizontal(|ui| {
                            for tag in &store.tags {
                                ui.label(tag);
                            }
                        });
                    }
                }
            });
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
                egui::widgets::global_theme_preference_buttons(ui);
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
            }
        });
    }
}

// fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
//     ui.horizontal(|ui| {
//         ui.spacing_mut().item_spacing.x = 0.0;
//         ui.label("Powered by ");
//         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
//         ui.label(" and ");
//         ui.hyperlink_to(
//             "eframe",
//             "https://github.com/emilk/egui/tree/master/crates/eframe",
//         );
//         ui.label(".");
//     });
// }
