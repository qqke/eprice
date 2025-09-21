use crate::settings::config::AppConfig;
use egui::{Color32, RichText, Slider, Ui};

/// Settings UI component for managing application configuration
pub struct SettingsUI {
    config: AppConfig,
    show_restart_required: bool,
    show_save_success: bool,
    error_message: Option<String>,
    current_tab: SettingsTab,
    temp_values: TempValues, // For slider values that need validation
}

#[derive(Debug, Clone, PartialEq)]
enum SettingsTab {
    UI,
    Notifications,
    Monitoring,
    Data,
    About,
}

#[derive(Debug, Clone)]
struct TempValues {
    font_size: f32,
    window_transparency: f32,
    notification_frequency: f32,
    price_drop_threshold: f32,
    monitoring_interval: f32,
    backup_frequency: f32,
}

impl SettingsUI {
    pub fn new() -> Self {
        let config = AppConfig::load().unwrap_or_default();
        let temp_values = TempValues {
            font_size: config.ui_settings.font_size,
            window_transparency: config.ui_settings.window_transparency,
            notification_frequency: config.notification_settings.notification_frequency_minutes
                as f32,
            price_drop_threshold: config.notification_settings.price_drop_threshold as f32,
            monitoring_interval: config.monitoring_settings.monitoring_interval_minutes as f32,
            backup_frequency: config.data_settings.backup_frequency_hours as f32,
        };

        Self {
            config,
            show_restart_required: false,
            show_save_success: false,
            error_message: None,
            current_tab: SettingsTab::UI,
            temp_values,
        }
    }

    /// Show the settings UI
    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("应用设置");
        ui.separator();

        // Show status messages
        if self.show_restart_required {
            ui.colored_label(Color32::YELLOW, "⚠ 某些设置需要重启应用后生效");
        }

        if self.show_save_success {
            ui.colored_label(Color32::GREEN, "✓ 设置已保存");
        }

        if let Some(ref error) = self.error_message {
            ui.colored_label(Color32::RED, format!("❌ 错误: {}", error));
        }

        ui.separator();

        // Settings tabs
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.current_tab == SettingsTab::UI, "界面设置")
                .clicked()
            {
                self.current_tab = SettingsTab::UI;
            }
            if ui
                .selectable_label(self.current_tab == SettingsTab::Notifications, "通知设置")
                .clicked()
            {
                self.current_tab = SettingsTab::Notifications;
            }
            if ui
                .selectable_label(self.current_tab == SettingsTab::Monitoring, "监控设置")
                .clicked()
            {
                self.current_tab = SettingsTab::Monitoring;
            }
            if ui
                .selectable_label(self.current_tab == SettingsTab::Data, "数据设置")
                .clicked()
            {
                self.current_tab = SettingsTab::Data;
            }
            if ui
                .selectable_label(self.current_tab == SettingsTab::About, "关于")
                .clicked()
            {
                self.current_tab = SettingsTab::About;
            }
        });

        ui.separator();

        // Settings content
        egui::ScrollArea::vertical().show(ui, |ui| match self.current_tab {
            SettingsTab::UI => self.render_ui_settings(ui),
            SettingsTab::Notifications => self.render_notification_settings(ui),
            SettingsTab::Monitoring => self.render_monitoring_settings(ui),
            SettingsTab::Data => self.render_data_settings(ui),
            SettingsTab::About => self.render_about_tab(ui),
        });

        ui.separator();

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("保存设置").clicked() {
                self.save_settings();
            }

            if ui.button("重置为默认").clicked() {
                self.reset_to_defaults();
            }

            if ui.button("导出设置").clicked() {
                self.export_settings();
            }

            if ui.button("导入设置").clicked() {
                self.import_settings();
            }
        });
    }

    fn render_ui_settings(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("外观设置").strong());

            ui.horizontal(|ui| {
                ui.label("主题:");
                egui::ComboBox::from_label("")
                    .selected_text(&self.config.ui_settings.theme)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.config.ui_settings.theme,
                            "light".to_string(),
                            "浅色",
                        );
                        ui.selectable_value(
                            &mut self.config.ui_settings.theme,
                            "dark".to_string(),
                            "深色",
                        );
                        ui.selectable_value(
                            &mut self.config.ui_settings.theme,
                            "auto".to_string(),
                            "跟随系统",
                        );
                    });
            });

            ui.horizontal(|ui| {
                ui.label("语言:");
                egui::ComboBox::from_label("")
                    .selected_text(match self.config.ui_settings.language.as_str() {
                        "zh" => "中文",
                        "en" => "English",
                        "auto" => "自动",
                        _ => "未知",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.config.ui_settings.language,
                            "zh".to_string(),
                            "中文",
                        );
                        ui.selectable_value(
                            &mut self.config.ui_settings.language,
                            "en".to_string(),
                            "English",
                        );
                        ui.selectable_value(
                            &mut self.config.ui_settings.language,
                            "auto".to_string(),
                            "自动",
                        );
                    });
            });

            ui.horizontal(|ui| {
                ui.label("字体大小:");
                ui.add(Slider::new(&mut self.temp_values.font_size, 8.0..=32.0).suffix("px"));
                if self.temp_values.font_size != self.config.ui_settings.font_size {
                    self.config.ui_settings.font_size = self.temp_values.font_size;
                    self.show_restart_required = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("窗口透明度:");
                ui.add(
                    Slider::new(&mut self.temp_values.window_transparency, 0.1..=1.0).suffix("%"),
                );
                if self.temp_values.window_transparency
                    != self.config.ui_settings.window_transparency
                {
                    self.config.ui_settings.window_transparency =
                        self.temp_values.window_transparency;
                }
            });

            ui.checkbox(&mut self.config.ui_settings.show_animations, "显示动画效果");
            ui.checkbox(&mut self.config.ui_settings.compact_mode, "紧凑模式");
        });
    }

    fn render_notification_settings(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("通知设置").strong());

            ui.checkbox(
                &mut self.config.notification_settings.enable_notifications,
                "启用通知",
            );

            if self.config.notification_settings.enable_notifications {
                ui.checkbox(
                    &mut self.config.notification_settings.enable_sound,
                    "声音提醒",
                );
                ui.checkbox(
                    &mut self.config.notification_settings.enable_popup,
                    "弹窗提醒",
                );
                ui.checkbox(
                    &mut self.config.notification_settings.show_promotion_alerts,
                    "促销提醒",
                );

                ui.horizontal(|ui| {
                    ui.label("通知频率:");
                    ui.add(
                        Slider::new(&mut self.temp_values.notification_frequency, 1.0..=1440.0)
                            .suffix("分钟"),
                    );
                    if self.temp_values.notification_frequency
                        != self
                            .config
                            .notification_settings
                            .notification_frequency_minutes as f32
                    {
                        self.config
                            .notification_settings
                            .notification_frequency_minutes =
                            self.temp_values.notification_frequency as u32;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("价格下降阈值:");
                    ui.add(
                        Slider::new(&mut self.temp_values.price_drop_threshold, 0.1..=50.0)
                            .suffix("%"),
                    );
                    if (self.temp_values.price_drop_threshold as f64
                        - self.config.notification_settings.price_drop_threshold)
                        .abs()
                        > 0.001
                    {
                        self.config.notification_settings.price_drop_threshold =
                            self.temp_values.price_drop_threshold as f64;
                    }
                });
            }
        });
    }

    fn render_monitoring_settings(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("监控设置").strong());

            ui.checkbox(
                &mut self.config.monitoring_settings.enable_auto_monitoring,
                "自动监控价格",
            );

            if self.config.monitoring_settings.enable_auto_monitoring {
                ui.horizontal(|ui| {
                    ui.label("监控间隔:");
                    ui.add(
                        Slider::new(&mut self.temp_values.monitoring_interval, 5.0..=1440.0)
                            .suffix("分钟"),
                    );
                    if self.temp_values.monitoring_interval
                        != self.config.monitoring_settings.monitoring_interval_minutes as f32
                    {
                        self.config.monitoring_settings.monitoring_interval_minutes =
                            self.temp_values.monitoring_interval as u32;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("价格历史保存天数:");
                    ui.add(
                        Slider::new(
                            &mut self.config.monitoring_settings.price_history_days,
                            7..=365,
                        )
                        .suffix("天"),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("每个商品最大价格记录数:");
                    ui.add(Slider::new(
                        &mut self
                            .config
                            .monitoring_settings
                            .max_price_records_per_product,
                        10..=1000,
                    ));
                });

                ui.checkbox(
                    &mut self.config.monitoring_settings.enable_trend_analysis,
                    "启用趋势分析",
                );
            }
        });
    }

    fn render_data_settings(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("数据设置").strong());

            ui.checkbox(&mut self.config.data_settings.auto_backup, "自动备份");

            if self.config.data_settings.auto_backup {
                ui.horizontal(|ui| {
                    ui.label("备份频率:");
                    ui.add(
                        Slider::new(&mut self.temp_values.backup_frequency, 1.0..=168.0)
                            .suffix("小时"),
                    );
                    if self.temp_values.backup_frequency
                        != self.config.data_settings.backup_frequency_hours as f32
                    {
                        self.config.data_settings.backup_frequency_hours =
                            self.temp_values.backup_frequency as u32;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("最大备份文件数:");
                    ui.add(Slider::new(
                        &mut self.config.data_settings.max_backup_files,
                        1..=30,
                    ));
                });
            }

            ui.horizontal(|ui| {
                ui.label("数据保留天数:");
                ui.add(
                    Slider::new(
                        &mut self.config.data_settings.data_retention_days,
                        30..=1095,
                    )
                    .suffix("天"),
                );
            });

            ui.checkbox(
                &mut self.config.data_settings.enable_cloud_sync,
                "云同步 (即将推出)",
            );

            ui.separator();

            ui.label("数据管理:");
            ui.horizontal(|ui| {
                if ui.button("清理缓存").clicked() {
                    self.clear_cache();
                }
                if ui.button("导出数据").clicked() {
                    self.export_data();
                }
                if ui.button("导入数据").clicked() {
                    self.import_data();
                }
            });
        });
    }

    fn render_about_tab(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("关于 ePrice").strong().size(18.0));
            ui.separator();

            ui.label("版本: 1.0.0");
            ui.label("构建日期: 2024-01-01");
            ui.label("Rust 版本: 1.70+");

            ui.separator();

            ui.label("一个现代化的价格比较和监控应用");
            ui.label("帮助您追踪商品价格变化，发现最优惠的购买时机。");

            ui.separator();

            ui.label("功能特色:");
            ui.label("• 实时价格监控");
            ui.label("• 价格趋势分析");
            ui.label("• 多店铺比价");
            ui.label("• 智能提醒系统");
            ui.label("• 条码扫描");
            ui.label("• 用户评价系统");

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("检查更新").clicked() {
                    self.check_for_updates();
                }
                if ui.button("反馈问题").clicked() {
                    self.open_feedback();
                }
                if ui.button("使用帮助").clicked() {
                    self.open_help();
                }
            });
        });
    }

    fn save_settings(&mut self) {
        match self.config.validate() {
            Ok(()) => {
                match self.config.save() {
                    Ok(()) => {
                        self.show_save_success = true;
                        self.error_message = None;
                        // Clear the success message after a few seconds
                        // In a real app, you'd use a timer
                    }
                    Err(e) => {
                        self.error_message = Some(format!("保存失败: {}", e));
                        self.show_save_success = false;
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(e);
                self.show_save_success = false;
            }
        }
    }

    fn reset_to_defaults(&mut self) {
        self.config.reset_to_defaults();
        self.temp_values = TempValues {
            font_size: self.config.ui_settings.font_size,
            window_transparency: self.config.ui_settings.window_transparency,
            notification_frequency: self
                .config
                .notification_settings
                .notification_frequency_minutes as f32,
            price_drop_threshold: self.config.notification_settings.price_drop_threshold as f32,
            monitoring_interval: self.config.monitoring_settings.monitoring_interval_minutes as f32,
            backup_frequency: self.config.data_settings.backup_frequency_hours as f32,
        };
        self.show_restart_required = true;
    }

    fn export_settings(&mut self) {
        // In a real implementation, show file dialog and export settings
        self.show_save_success = true;
    }

    fn import_settings(&mut self) {
        // In a real implementation, show file dialog and import settings
        self.show_save_success = true;
    }

    fn clear_cache(&mut self) {
        // In a real implementation, clear application cache
        self.show_save_success = true;
    }

    fn export_data(&mut self) {
        // In a real implementation, export user data
        self.show_save_success = true;
    }

    fn import_data(&mut self) {
        // In a real implementation, import user data
        self.show_save_success = true;
    }

    fn check_for_updates(&mut self) {
        // In a real implementation, check for app updates
        self.show_save_success = true;
    }

    fn open_feedback(&mut self) {
        // In a real implementation, open feedback form or URL
    }

    fn open_help(&mut self) {
        // In a real implementation, open help documentation
    }

    /// Get current configuration
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    /// Apply configuration changes
    pub fn apply_config(&mut self, config: AppConfig) {
        self.config = config;
    }
}

impl Default for SettingsUI {
    fn default() -> Self {
        Self::new()
    }
}
