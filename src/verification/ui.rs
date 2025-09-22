use crate::models::PriceRecord;
use crate::services::AppServices;
use crate::verification::manager::VerificationManager;
use egui::{Color32, RichText};
use std::collections::HashMap;

/// UI component for managing price record verification
pub struct VerificationUI {
    verification_manager: VerificationManager,
    selected_records: HashMap<String, bool>, // price_record_id -> selected
    filter_status: String,                   // "all", "pending", "verified", "rejected"
    search_text: String,
    reason_text: String,
    show_verification_dialog: bool,
    verification_action: VerificationAction,
    bulk_operation_mode: bool,
    current_verifier: String,
}

#[derive(Debug, Clone, PartialEq)]
enum VerificationAction {
    None,
    Verify,
    Reject,
    #[allow(dead_code)]
    Reset,
}

impl VerificationUI {
    pub fn new() -> Self {
        Self {
            verification_manager: VerificationManager::new(),
            selected_records: HashMap::new(),
            filter_status: "pending".to_string(),
            search_text: String::new(),
            reason_text: String::new(),
            show_verification_dialog: false,
            verification_action: VerificationAction::None,
            bulk_operation_mode: false,
            current_verifier: "system".to_string(),
        }
    }

    /// Set the current verifier (usually the logged-in user)
    pub fn set_verifier(&mut self, verifier: &str) {
        self.current_verifier = verifier.to_string();
    }

    /// Show the verification UI
    pub fn show(&mut self, ui: &mut egui::Ui, app_services: &mut AppServices) {
        ui.heading("价格记录验证系统");
        ui.separator();

        // Show verification statistics
        self.render_verification_stats(ui, &*app_services);

        ui.separator();

        // Filter and search controls
        self.render_filter_controls(ui);

        ui.separator();

        // Bulk operation controls
        self.render_bulk_controls(ui, app_services);

        ui.separator();

        // Price records table
        self.render_price_records_table(ui, app_services);

        // Show verification dialog if needed
        if self.show_verification_dialog {
            self.render_verification_dialog(ui, app_services);
        }
    }

    fn render_verification_stats(&mut self, ui: &mut egui::Ui, app_services: &AppServices) {
        ui.group(|ui| {
            ui.label(RichText::new("验证统计").strong());

            if let Ok(stats) = self
                .verification_manager
                .get_verification_stats(&app_services.price_service)
            {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("待验证: {}", stats.total_pending));
                        ui.label(format!("已验证: {}", stats.total_verified));
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label(format!("已拒绝: {}", stats.total_rejected));
                        ui.label(format!("验证率: {:.1}%", stats.verification_rate));
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label(format!("24小时内验证: {}", stats.recent_verifications));

                        // Progress bar for verification rate
                        let progress = stats.verification_rate / 100.0;
                        ui.add(
                            egui::ProgressBar::new(progress as f32)
                                .text(format!("{:.1}%", stats.verification_rate)),
                        );
                    });
                });
            } else {
                ui.label("无法获取验证统计数据");
            }
        });
    }

    fn render_filter_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("筛选:");

            // Status filter
            egui::ComboBox::from_label("状态")
                .selected_text(&self.filter_status)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_status, "all".to_string(), "全部");
                    ui.selectable_value(&mut self.filter_status, "pending".to_string(), "待验证");
                    ui.selectable_value(&mut self.filter_status, "verified".to_string(), "已验证");
                    ui.selectable_value(&mut self.filter_status, "rejected".to_string(), "已拒绝");
                });

            ui.separator();

            // Search box
            ui.label("搜索:");
            ui.add(egui::TextEdit::singleline(&mut self.search_text).hint_text("商品名称或店铺"));

            ui.separator();

            // Bulk operation toggle
            ui.checkbox(&mut self.bulk_operation_mode, "批量操作模式");
        });
    }

    fn render_bulk_controls(&mut self, ui: &mut egui::Ui, _app_services: &mut AppServices) {
        if !self.bulk_operation_mode {
            return;
        }

        ui.group(|ui| {
            ui.label(RichText::new("批量操作").strong());

            ui.horizontal(|ui| {
                let selected_count = self.selected_records.values().filter(|&&v| v).count();
                ui.label(format!("已选择 {} 条记录", selected_count));

                ui.separator();

                if ui.button("全选").clicked() {
                    for (_, selected) in self.selected_records.iter_mut() {
                        *selected = true;
                    }
                }

                if ui.button("取消全选").clicked() {
                    for (_, selected) in self.selected_records.iter_mut() {
                        *selected = false;
                    }
                }

                ui.separator();

                // Bulk action buttons
                if selected_count > 0 {
                    if ui
                        .button(RichText::new("批量验证").color(Color32::GREEN))
                        .clicked()
                    {
                        self.verification_action = VerificationAction::Verify;
                        self.show_verification_dialog = true;
                    }

                    if ui
                        .button(RichText::new("批量拒绝").color(Color32::RED))
                        .clicked()
                    {
                        self.verification_action = VerificationAction::Reject;
                        self.show_verification_dialog = true;
                    }
                }
            });
        });
    }

    fn render_price_records_table(&mut self, ui: &mut egui::Ui, app_services: &mut AppServices) {
        // Get all price records from the service
        let all_records = self.get_filtered_price_records(app_services);

        ui.label(format!("找到 {} 条价格记录", all_records.len()));

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::auto()) // Checkbox (if bulk mode)
                .column(egui_extras::Column::initial(80.0).at_least(60.0)) // Product
                .column(egui_extras::Column::initial(80.0).at_least(60.0)) // Store
                .column(egui_extras::Column::initial(60.0).at_least(50.0)) // Price
                .column(egui_extras::Column::initial(80.0).at_least(70.0)) // Status
                .column(egui_extras::Column::initial(100.0).at_least(80.0)) // Timestamp
                .column(egui_extras::Column::initial(120.0).at_least(100.0)) // Actions
                .header(20.0, |mut header| {
                    if self.bulk_operation_mode {
                        header.col(|ui| {
                            ui.label("选择");
                        });
                    }
                    header.col(|ui| {
                        ui.label("商品");
                    });
                    header.col(|ui| {
                        ui.label("店铺");
                    });
                    header.col(|ui| {
                        ui.label("价格");
                    });
                    header.col(|ui| {
                        ui.label("状态");
                    });
                    header.col(|ui| {
                        ui.label("时间");
                    });
                    header.col(|ui| {
                        ui.label("操作");
                    });
                })
                .body(|mut body| {
                    for (record, product_name, store_name) in all_records {
                        body.row(25.0, |mut row| {
                            // Checkbox for bulk operations
                            if self.bulk_operation_mode {
                                row.col(|ui| {
                                    let record_id = record
                                        .id
                                        .as_ref()
                                        .unwrap_or(&"unknown".to_string())
                                        .clone();
                                    let selected = self
                                        .selected_records
                                        .entry(record_id.clone())
                                        .or_insert(false);
                                    ui.checkbox(selected, "");
                                });
                            }

                            // Product name
                            row.col(|ui| {
                                ui.label(&product_name);
                            });

                            // Store name
                            row.col(|ui| {
                                ui.label(&store_name);
                            });

                            // Price
                            row.col(|ui| {
                                let price_text = if record.is_on_sale {
                                    format!("¥{:.2} 🏷", record.price)
                                } else {
                                    format!("¥{:.2}", record.price)
                                };
                                ui.label(price_text);
                            });

                            // Status
                            row.col(|ui| {
                                let (status_text, status_color) =
                                    match record.verification_status.as_str() {
                                        "verified" => ("已验证", Color32::GREEN),
                                        "rejected" => ("已拒绝", Color32::RED),
                                        "pending" => ("待验证", Color32::YELLOW),
                                        _ => ("未知", Color32::GRAY),
                                    };
                                ui.colored_label(status_color, status_text);
                            });

                            // Timestamp
                            row.col(|ui| {
                                ui.label(record.timestamp.format("%m-%d %H:%M").to_string());
                            });

                            // Actions
                            row.col(|ui| {
                                if !self.bulk_operation_mode {
                                    ui.horizontal(|ui| match record.verification_status.as_str() {
                                        "pending" => {
                                            if ui.small_button("✓").clicked() {
                                                self.verify_single_record(
                                                    record.id.as_ref().unwrap(),
                                                    app_services,
                                                );
                                            }
                                            if ui.small_button("✗").clicked() {
                                                self.reject_single_record(
                                                    record.id.as_ref().unwrap(),
                                                    app_services,
                                                );
                                            }
                                        }
                                        "verified" | "rejected" => {
                                            if ui.small_button("↻").clicked() {
                                                self.reset_single_record(
                                                    record.id.as_ref().unwrap(),
                                                    app_services,
                                                );
                                            }
                                        }
                                        _ => {}
                                    });
                                }
                            });
                        });
                    }
                });
        });
    }

    fn render_verification_dialog(&mut self, ui: &mut egui::Ui, app_services: &mut AppServices) {
        let dialog_title = match self.verification_action {
            VerificationAction::Verify => "验证价格记录",
            VerificationAction::Reject => "拒绝价格记录",
            VerificationAction::Reset => "重置价格记录",
            VerificationAction::None => "操作",
        };

        egui::Window::new(dialog_title)
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("请输入操作原因 (可选):");
                ui.add(egui::TextEdit::multiline(&mut self.reason_text).desired_rows(3));

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("确认").clicked() {
                        self.execute_verification_action(app_services);
                        self.show_verification_dialog = false;
                        self.reason_text.clear();
                    }

                    if ui.button("取消").clicked() {
                        self.show_verification_dialog = false;
                        self.reason_text.clear();
                    }
                });
            });
    }

    fn get_filtered_price_records(
        &self,
        app_services: &AppServices,
    ) -> Vec<(PriceRecord, String, String)> {
        // This is a simplified implementation - in a real app, you'd query the database
        let mut all_records = Vec::new();

        // Get all price records from products (this is for demo purposes)
        for product in &app_services
            .product_service
            .get_all_products()
            .unwrap_or_default()
        {
            for price_record in &product.prices {
                // Apply status filter
                if self.filter_status != "all"
                    && price_record.verification_status != self.filter_status
                {
                    continue;
                }

                // Apply search filter
                if !self.search_text.is_empty() {
                    let search_lower = self.search_text.to_lowercase();
                    if !product.name.to_lowercase().contains(&search_lower) {
                        continue;
                    }
                }

                // Get store name
                let store_name = app_services
                    .store_service
                    .get_store(&price_record.store_id)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|_| "未知店铺".to_string());

                all_records.push((price_record.clone(), product.name.clone(), store_name));
            }
        }

        // Sort by timestamp (newest first)
        all_records.sort_by(|a, b| b.0.timestamp.cmp(&a.0.timestamp));

        all_records
    }

    fn verify_single_record(&mut self, record_id: &str, app_services: &mut AppServices) {
        if let Err(e) = self.verification_manager.verify_price_record(
            &mut app_services.price_service,
            record_id,
            &self.current_verifier,
            None,
        ) {
            log::error!("Failed to verify record {}: {}", record_id, e);
        }
    }

    fn reject_single_record(&mut self, record_id: &str, app_services: &mut AppServices) {
        if let Err(e) = self.verification_manager.reject_price_record(
            &mut app_services.price_service,
            record_id,
            &self.current_verifier,
            None,
        ) {
            log::error!("Failed to reject record {}: {}", record_id, e);
        }
    }

    fn reset_single_record(&mut self, record_id: &str, app_services: &mut AppServices) {
        if let Err(e) = self.verification_manager.reset_to_pending(
            &mut app_services.price_service,
            record_id,
            &self.current_verifier,
            None,
        ) {
            log::error!("Failed to reset record {}: {}", record_id, e);
        }
    }

    fn execute_verification_action(&mut self, app_services: &mut AppServices) {
        let selected_records: Vec<String> = self
            .selected_records
            .iter()
            .filter_map(|(id, &selected)| if selected { Some(id.clone()) } else { None })
            .collect();

        let reason = if self.reason_text.trim().is_empty() {
            None
        } else {
            Some(self.reason_text.clone())
        };

        let result = match self.verification_action {
            VerificationAction::Verify => {
                if self.bulk_operation_mode {
                    self.verification_manager.bulk_verify_records(
                        &mut app_services.price_service,
                        &selected_records,
                        &self.current_verifier,
                        reason,
                    )
                } else {
                    Ok(0)
                }
            }
            VerificationAction::Reject => {
                if self.bulk_operation_mode {
                    self.verification_manager.bulk_reject_records(
                        &mut app_services.price_service,
                        &selected_records,
                        &self.current_verifier,
                        reason,
                    )
                } else {
                    Ok(0)
                }
            }
            _ => Ok(0),
        };

        match result {
            Ok(count) => {
                log::info!("Successfully processed {} records", count);
                // Clear selections after successful operation
                self.selected_records.clear();
            }
            Err(e) => {
                log::error!("Failed to process records: {}", e);
            }
        }

        self.verification_action = VerificationAction::None;
    }
}

impl Default for VerificationUI {
    fn default() -> Self {
        Self::new()
    }
}
