use crate::alerts::{AlertService, Notification, NotificationType};
use crate::models::PriceAlert;
use eframe::egui;

/// Alert management UI component
#[derive(Default)]
pub struct AlertUI {
    alert_service: AlertService,
    new_alert_product_id: String,
    new_alert_target_price: String,
    selected_alert_id: Option<String>,
    show_add_alert_dialog: bool,
    show_notification_panel: bool,
    notifications: Vec<Notification>,
    error_message: Option<String>,
    unread_count: usize,
    check_interval_secs: u64,
}

impl AlertUI {
    /// Create a new AlertUI instance
    pub fn new() -> Self {
        Self {
            alert_service: AlertService::new(),
            new_alert_product_id: String::new(),
            new_alert_target_price: String::new(),
            selected_alert_id: None,
            show_add_alert_dialog: false,
            show_notification_panel: false,
            notifications: Vec::new(),
            error_message: None,
            unread_count: 0,
            check_interval_secs: 300,
        }
    }

    /// Render the alerts UI tab
    pub fn show(&mut self, ui: &mut egui::Ui, user_id: &str) {
        ui.heading("价格提醒管理");

        // Error message display
        if let Some(error) = &self.error_message {
            ui.colored_label(egui::Color32::RED, format!("错误: {}", error));
        }

        // Control buttons
        ui.horizontal(|ui| {
            if ui.button("添加新提醒").clicked() {
                self.show_add_alert_dialog = true;
            }

            if ui.button("刷新通知").clicked() {
                self.refresh_notifications(user_id);
                self.refresh_unread_count(user_id);
            }

            ui.separator();

            let monitoring_text = if self.alert_service.is_monitoring() {
                "停止监控"
            } else {
                "开始监控"
            };

            if ui.button(monitoring_text).clicked() {
                self.toggle_monitoring();
            }

            ui.label(format!(
                "监控状态: {}",
                if self.alert_service.is_monitoring() {
                    "运行中"
                } else {
                    "已停止"
                }
            ));

            ui.separator();

            ui.label(format!("未读通知: {}", self.unread_count));
            if ui.button("标记全部为已读").clicked() {
                for n in self
                    .alert_service
                    .notification_service()
                    .get_user_notifications(user_id)
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|n| n.read_at.is_none())
                {
                    let _ = self
                        .alert_service
                        .notification_service()
                        .mark_as_read(&n.id);
                }
                self.refresh_notifications(user_id);
                self.refresh_unread_count(user_id);
            }

            if ui.button("清理30天前通知").clicked() {
                let _ = self
                    .alert_service
                    .notification_service()
                    .clear_old_notifications(30);
                self.refresh_notifications(user_id);
                self.refresh_unread_count(user_id);
            }
        });

        ui.horizontal(|ui| {
            ui.label("监控间隔(秒):");
            ui.add(egui::widgets::DragValue::new(&mut self.check_interval_secs).range(30..=3600));
            if ui.button("更新间隔").clicked() {
                self.alert_service
                    .monitor_mut()
                    .set_check_interval(std::time::Duration::from_secs(self.check_interval_secs));
            }
            if ui.button("立即检查提醒").clicked() {
                match self.alert_service.check_alerts() {
                    Ok(results) => {
                        let triggered = results.iter().filter(|r| r.triggered).count();
                        self.error_message = Some(format!("已检查提醒，共触发 {} 项", triggered));
                    }
                    Err(e) => self.error_message = Some(format!("检查提醒失败: {}", e)),
                }
            }
        });

        ui.separator();

        // Alerts list
        self.show_alerts_list(ui, user_id);

        // Add alert dialog
        if self.show_add_alert_dialog {
            self.show_add_alert_dialog(ui, user_id);
        }

        // Notifications panel
        if self.show_notification_panel {
            self.show_notifications_panel(ui);
        }
    }

    /// Display the list of active alerts
    fn show_alerts_list(&mut self, ui: &mut egui::Ui, user_id: &str) {
        ui.heading("当前提醒");

        match self.alert_service.get_user_alerts(user_id) {
            Ok(alerts) => {
                if alerts.is_empty() {
                    ui.label("暂无价格提醒");
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for alert in &alerts {
                            self.show_alert_item(ui, alert);
                        }
                    });
                }
            }
            Err(e) => {
                self.error_message = Some(format!("获取提醒列表失败: {}", e));
            }
        }
    }

    /// Display a single alert item
    fn show_alert_item(&mut self, ui: &mut egui::Ui, alert: &PriceAlert) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(format!("商品ID: {}", alert.product_id));
                    ui.label(format!("目标价格: ¥{:.2}", alert.target_price));
                    ui.label(format!(
                        "状态: {}",
                        if alert.is_active { "激活" } else { "暂停" }
                    ));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("删除").clicked() {
                        if let Err(e) = self.alert_service.remove_alert(&alert.id) {
                            self.error_message = Some(format!("删除提醒失败: {}", e));
                        }
                    }

                    let toggle_text = if alert.is_active { "暂停" } else { "激活" };
                    if ui.button(toggle_text).clicked() {
                        if let Err(e) = self
                            .alert_service
                            .monitor_mut()
                            .update_alert_active(&alert.id, !alert.is_active)
                        {
                            self.error_message = Some(format!("更新提醒状态失败: {}", e));
                        }
                    }

                    if ui.button("编辑").clicked() {
                        self.selected_alert_id = Some(alert.id.clone());
                        self.new_alert_product_id = alert.product_id.clone();
                        self.new_alert_target_price = alert.target_price.to_string();
                        self.show_add_alert_dialog = true;
                    }
                });
            });
        });
    }

    /// Show the add/edit alert dialog
    fn show_add_alert_dialog(&mut self, ui: &mut egui::Ui, user_id: &str) {
        let mut dialog_open = self.show_add_alert_dialog;
        egui::Window::new("添加价格提醒")
            .open(&mut dialog_open)
            .show(ui.ctx(), |ui| {
                ui.label("商品ID:");
                ui.text_edit_singleline(&mut self.new_alert_product_id);

                ui.label("目标价格:");
                ui.text_edit_singleline(&mut self.new_alert_target_price);

                ui.horizontal(|ui| {
                    if ui.button("确认").clicked() {
                        self.add_new_alert(user_id);
                    }

                    if ui.button("取消").clicked() {
                        self.cancel_add_alert();
                    }
                });
            });
        self.show_add_alert_dialog = dialog_open;
    }

    /// Show notifications panel
    fn show_notifications_panel(&mut self, ui: &mut egui::Ui) {
        let mut panel_open = self.show_notification_panel;
        egui::Window::new("通知")
            .open(&mut panel_open)
            .show(ui.ctx(), |ui| {
                if self.notifications.is_empty() {
                    ui.label("暂无通知");
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let notifications = self.notifications.clone();
                        for notification in &notifications {
                            self.show_notification_item(ui, notification);
                        }
                    });
                }
            });
        self.show_notification_panel = panel_open;
    }

    /// Display a single notification item
    fn show_notification_item(&self, ui: &mut egui::Ui, notification: &Notification) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(&notification.title);
                ui.label(&notification.message);
                ui.label(format!(
                    "时间: {}",
                    notification.created_at.format("%Y-%m-%d %H:%M:%S")
                ));

                let type_text = match notification.notification_type {
                    NotificationType::PriceAlert => "价格提醒",
                    NotificationType::SystemAlert => "系统通知",
                    NotificationType::ProductUpdate => "商品更新",
                    NotificationType::UserMessage => "用户消息",
                };
                ui.label(format!("类型: {}", type_text));
            });
        });
    }

    /// Add a new alert
    fn add_new_alert(&mut self, user_id: &str) {
        if self.new_alert_product_id.trim().is_empty() {
            self.error_message = Some("商品ID不能为空".to_string());
            return;
        }
        if let Ok(target_price) = self.new_alert_target_price.parse::<f64>() {
            if target_price <= 0.0 {
                self.error_message = Some("目标价格必须大于0".to_string());
                return;
            }
            // 如果是编辑，复用 id；否则新建
            let id = self
                .selected_alert_id
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            let alert = PriceAlert {
                id: id.clone(),
                user_id: user_id.to_string(),
                product_id: self.new_alert_product_id.clone(),
                target_price,
                is_active: true,
                created_at: chrono::Utc::now(),
            };

            let res = if self.selected_alert_id.is_some() {
                self.alert_service.monitor().update_alert(alert)
            } else {
                self.alert_service.add_alert(alert)
            };

            match res {
                Ok(_) => {
                    self.cancel_add_alert();
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("添加提醒失败: {}", e));
                }
            }
        } else {
            self.error_message = Some("价格格式不正确".to_string());
        }
    }

    /// Cancel adding alert
    fn cancel_add_alert(&mut self) {
        self.show_add_alert_dialog = false;
        self.new_alert_product_id.clear();
        self.new_alert_target_price.clear();
        self.selected_alert_id = None;
    }

    /// Toggle monitoring state
    fn toggle_monitoring(&mut self) {
        if self.alert_service.is_monitoring() {
            if let Err(e) = self.alert_service.stop_monitoring() {
                self.error_message = Some(format!("停止监控失败: {}", e));
            }
        } else if let Err(e) = self.alert_service.start_monitoring() {
            self.error_message = Some(format!("启动监控失败: {}", e));
        }
    }

    /// Refresh notifications
    fn refresh_notifications(&mut self, user_id: &str) {
        // Get notifications from notification service
        let notifications = self
            .alert_service
            .notification_service()
            .get_user_notifications(user_id)
            .unwrap_or_default();

        self.notifications = notifications;
        self.show_notification_panel = true;
    }

    fn refresh_unread_count(&mut self, user_id: &str) {
        self.unread_count = self
            .alert_service
            .notification_service()
            .get_unread_count(user_id)
            .unwrap_or(0);
    }

    /// Get alert service reference
    pub fn alert_service(&self) -> &AlertService {
        &self.alert_service
    }

    /// Get mutable alert service reference
    pub fn alert_service_mut(&mut self) -> &mut AlertService {
        &mut self.alert_service
    }
}
