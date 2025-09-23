use crate::models::Product;
use crate::scanner::{BarcodeType, CameraInfo, ProductMatch, ScanResult, ScannerService};
use crate::utils::{generate_barcode_checksum, validate_barcode};
use eframe::egui;
use std::time::{Duration, Instant};

/// Enhanced Scanner UI component with improved camera controls and user experience
pub struct ScannerUI {
    scanner_service: ScannerService,

    // UI State
    is_scanning: bool,
    last_scan_time: Option<Instant>,
    scan_cooldown: Duration,
    flash_enabled: bool,
    focus_mode: FocusMode,
    resolution: CameraResolution,

    // Results
    current_scan: Option<ScanResult>,
    current_product: Option<Product>,
    scan_history: Vec<ScanHistoryItem>,

    // Enhanced UI Elements
    camera_preview_enabled: bool,
    available_cameras: Vec<CameraInfo>,
    selected_camera: usize,
    camera_zoom: f32,
    brightness: f32,
    contrast: f32,
    exposure_compensation: f32,

    // Advanced scanning features
    scan_area_selection: bool,
    #[allow(dead_code)]
    scan_area: egui::Rect,
    multi_scan_mode: bool,
    vibration_feedback: bool,
    auto_focus_enabled: bool,

    // Search functionality
    manual_search_query: String,
    search_results: Vec<ProductMatch>,
    #[allow(dead_code)]
    search_filters: SearchFilters,

    // Manual barcode input
    manual_barcode_input: String,
    manual_barcode_type: BarcodeType,
    manual_barcode_info: Option<String>,

    // Enhanced status and feedback
    status_message: String,
    error_message: Option<String>,
    #[allow(dead_code)]
    success_animation: bool,
    #[allow(dead_code)]
    scan_feedback_type: FeedbackType,
    scan_count: u32,

    // Tutorial and help
    show_tutorial: bool,
    show_help_overlay: bool,
    first_time_user: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum FocusMode {
    Auto,
    Manual,
    Macro,
    Infinity,
}

#[derive(Debug, Clone, PartialEq)]
enum CameraResolution {
    Low,    // 640x480
    Medium, // 1280x720
    High,   // 1920x1080
    Ultra,  // 4K
}

#[derive(Debug, Clone)]
struct SearchFilters {
    #[allow(dead_code)]
    category: Option<String>,
    #[allow(dead_code)]
    min_price: Option<f64>,
    #[allow(dead_code)]
    max_price: Option<f64>,
    #[allow(dead_code)]
    only_in_stock: bool,
    #[allow(dead_code)]
    only_on_sale: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum FeedbackType {
    #[allow(dead_code)]
    Visual,
    #[allow(dead_code)]
    Audio,
    #[allow(dead_code)]
    Haptic,
    Combined,
}

impl ScannerUI {
    pub fn new() -> Self {
        let available_cameras = ScannerService::default().list_cameras();
        let first_time = true; // In real app, check from settings

        Self {
            scanner_service: ScannerService::new(),
            is_scanning: false,
            last_scan_time: None,
            scan_cooldown: Duration::from_millis(1000),
            flash_enabled: false,
            focus_mode: FocusMode::Auto,
            resolution: CameraResolution::Medium,

            current_scan: None,
            current_product: None,
            scan_history: Vec::new(),

            camera_preview_enabled: true,
            available_cameras,
            selected_camera: 0,
            camera_zoom: 1.0,
            brightness: 0.0,
            contrast: 0.0,
            exposure_compensation: 0.0,

            scan_area_selection: false,
            scan_area: egui::Rect::from_min_size(egui::pos2(50.0, 50.0), egui::vec2(200.0, 150.0)),
            multi_scan_mode: false,
            vibration_feedback: true,
            auto_focus_enabled: true,

            manual_search_query: String::new(),
            search_results: Vec::new(),
            search_filters: SearchFilters {
                category: None,
                min_price: None,
                max_price: None,
                only_in_stock: false,
                only_on_sale: false,
            },

            manual_barcode_input: String::new(),
            manual_barcode_type: BarcodeType::Ean13,
            manual_barcode_info: None,

            status_message: "Ready to scan - Point camera at barcode".to_string(),
            error_message: None,
            success_animation: false,
            scan_feedback_type: FeedbackType::Combined,
            scan_count: 0,

            show_tutorial: first_time,
            show_help_overlay: false,
            first_time_user: first_time,
        }
    }

    /// Show the enhanced scanner UI with improved controls and feedback
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // Show tutorial for first-time users
        if self.show_tutorial {
            self.show_tutorial_overlay(ctx);
        }

        // Help overlay
        if self.show_help_overlay {
            self.show_help_overlay_window(ctx);
        }

        // Main scanner interface
        ui.horizontal(|ui| {
            ui.heading("📱 Enhanced Barcode Scanner");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❓ Help").clicked() {
                    self.show_help_overlay = true;
                }
                if ui.button("⚙️ Settings").clicked() {
                    // Settings would open in a separate window
                }
                // Scan counter
                ui.label(format!("Scans: {}", self.scan_count));
            });
        });

        ui.separator();

        // Enhanced camera controls section
        self.show_enhanced_camera_controls(ui);

        ui.separator();

        // Enhanced scanning section with visual feedback
        self.show_scanning_section(ctx, ui);

        ui.separator();

        // Enhanced results section with animations
        self.show_results_section(ui);

        ui.separator();

        // Advanced search section with filters
        self.show_manual_search_section(ui);

        ui.separator();

        // Manual barcode input and utilities
        self.show_manual_barcode_section(ui);

        ui.separator();

        // Enhanced history section with statistics
        self.show_history_section(ui);

        // Update scanning state and animations
        self.update_scanning_state();
    }

    /// Show enhanced camera controls with advanced settings
    fn show_enhanced_camera_controls(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("📷 Camera Controls");

            // Primary camera controls
            ui.horizontal(|ui| {
                // Camera selection with preview info
                ui.vertical(|ui| {
                    ui.label("Camera:");
                    egui::ComboBox::from_label("")
                        .selected_text(if self.available_cameras.is_empty() {
                            "No cameras available".to_string()
                        } else if self.selected_camera < self.available_cameras.len() {
                            let camera_name = &self.available_cameras[self.selected_camera].name;
                            let resolution = self.resolution_text();
                            format!("{} ({})", camera_name, resolution)
                        } else {
                            "Select camera".to_string()
                        })
                        .show_ui(ui, |ui| {
                            for (index, camera) in self.available_cameras.iter().enumerate() {
                                let text = format!("{} - {}", camera.name, camera.id);
                                ui.selectable_value(&mut self.selected_camera, index, text);
                            }
                        });
                });

                ui.separator();

                // Resolution settings
                ui.vertical(|ui| {
                    ui.label("Resolution:");
                    egui::ComboBox::from_label("")
                        .selected_text(self.resolution_text())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.resolution,
                                CameraResolution::Low,
                                "Low (640x480)",
                            );
                            ui.selectable_value(
                                &mut self.resolution,
                                CameraResolution::Medium,
                                "Medium (1280x720)",
                            );
                            ui.selectable_value(
                                &mut self.resolution,
                                CameraResolution::High,
                                "High (1920x1080)",
                            );
                            ui.selectable_value(
                                &mut self.resolution,
                                CameraResolution::Ultra,
                                "Ultra (4K)",
                            );
                        });
                });
            });

            ui.separator();

            // Camera control buttons with status indicators
            ui.horizontal(|ui| {
                // Start/Stop camera with enhanced feedback
                if self.scanner_service.is_camera_running() {
                    if ui.button("🛑 Stop Camera").clicked() {
                        if let Err(e) = self.scanner_service.stop_camera() {
                            self.error_message = Some(format!("Failed to stop camera: {}", e));
                        } else {
                            self.status_message = "Camera stopped".to_string();
                            self.is_scanning = false;
                        }
                    }
                    ui.colored_label(egui::Color32::GREEN, "● Live");
                } else {
                    if ui.button("▶️ Start Camera").clicked() {
                        if let Err(e) = self.scanner_service.start_camera() {
                            self.error_message = Some(format!("Failed to start camera: {}", e));
                        } else {
                            self.status_message = "Camera started - Ready to scan".to_string();
                            self.error_message = None;
                        }
                    }
                    ui.colored_label(egui::Color32::RED, "● Offline");
                }

                ui.separator();

                // Enhanced toggles
                ui.checkbox(&mut self.camera_preview_enabled, "📺 Preview");
                ui.checkbox(&mut self.flash_enabled, "🔦 Flash");
                ui.checkbox(&mut self.auto_focus_enabled, "🎯 Auto Focus");
            });

            // Advanced camera settings (collapsible)
            ui.collapsing("⚙️ Advanced Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Focus Mode:");
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", self.focus_mode))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.focus_mode, FocusMode::Auto, "Auto");
                                ui.selectable_value(
                                    &mut self.focus_mode,
                                    FocusMode::Manual,
                                    "Manual",
                                );
                                ui.selectable_value(
                                    &mut self.focus_mode,
                                    FocusMode::Macro,
                                    "Macro",
                                );
                                ui.selectable_value(
                                    &mut self.focus_mode,
                                    FocusMode::Infinity,
                                    "Infinity",
                                );
                            });
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.label("Camera Adjustments:");
                        ui.horizontal(|ui| {
                            ui.label("Zoom:");
                            ui.add(egui::Slider::new(&mut self.camera_zoom, 1.0..=5.0).suffix("x"));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Brightness:");
                            ui.add(egui::Slider::new(&mut self.brightness, -100.0..=100.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Contrast:");
                            ui.add(egui::Slider::new(&mut self.contrast, -100.0..=100.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Exposure:");
                            ui.add(egui::Slider::new(
                                &mut self.exposure_compensation,
                                -2.0..=2.0,
                            ));
                        });
                    });
                });

                ui.separator();

                // Scan area controls
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.scan_area_selection, "Custom Scan Area");
                    ui.checkbox(&mut self.multi_scan_mode, "Multi-Scan Mode");
                    ui.checkbox(&mut self.vibration_feedback, "Haptic Feedback");
                });
            });
        });

        // Enhanced camera preview with overlay
        if self.camera_preview_enabled && self.scanner_service.is_camera_running() {
            self.show_enhanced_camera_preview(ui);
        }
    }

    /// Show enhanced camera preview with scan overlay and guides
    fn show_enhanced_camera_preview(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("📹 Live Camera Preview");

            // Preview area with scan guides
            let preview_size = egui::vec2(320.0, 240.0);
            let (rect, _response) =
                ui.allocate_exact_size(preview_size, egui::Sense::click_and_drag());

            // Draw preview background
            ui.painter().rect_filled(
                rect,
                egui::CornerRadius::same(4),
                egui::Color32::from_gray(30),
            );

            // Draw scan area overlay
            if self.scan_area_selection {
                let scan_rect =
                    egui::Rect::from_center_size(rect.center(), egui::vec2(160.0, 120.0));

                // Scan area border - draw lines manually
                let stroke = egui::Stroke::new(2.0, egui::Color32::GREEN);
                ui.painter()
                    .line_segment([scan_rect.left_top(), scan_rect.right_top()], stroke);
                ui.painter()
                    .line_segment([scan_rect.right_top(), scan_rect.right_bottom()], stroke);
                ui.painter()
                    .line_segment([scan_rect.right_bottom(), scan_rect.left_bottom()], stroke);
                ui.painter()
                    .line_segment([scan_rect.left_bottom(), scan_rect.left_top()], stroke);
            }

            // Draw center crosshair
            let center = rect.center();
            let crosshair_size = 20.0;
            ui.painter().line_segment(
                [
                    egui::pos2(center.x - crosshair_size, center.y),
                    egui::pos2(center.x + crosshair_size, center.y),
                ],
                egui::Stroke::new(1.0, egui::Color32::WHITE),
            );

            // Preview controls
            ui.horizontal(|ui| {
                ui.label(format!("Zoom: {:.1}x", self.camera_zoom));
                if ui.small_button("➕").clicked() && self.camera_zoom < 5.0 {
                    self.camera_zoom += 0.5;
                }
                if ui.small_button("➖").clicked() && self.camera_zoom > 1.0 {
                    self.camera_zoom -= 0.5;
                }
            });
        });
    }

    /// Get resolution text for display
    fn resolution_text(&self) -> &'static str {
        match self.resolution {
            CameraResolution::Low => "640x480",
            CameraResolution::Medium => "1280x720",
            CameraResolution::High => "1920x1080",
            CameraResolution::Ultra => "4K",
        }
    }

    /// Show tutorial overlay for first-time users
    fn show_tutorial_overlay(&mut self, ctx: &egui::Context) {
        egui::Window::new("🎓 Scanner Tutorial")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.heading("Welcome to the Enhanced Barcode Scanner!");
                ui.separator();

                ui.label("📷 1. Start your camera using the 'Start Camera' button");
                ui.label("🔍 2. Point the camera at a barcode");
                ui.label("📱 3. Tap 'Scan Now' or enable 'Auto Scan'");
                ui.label("⚙️ 4. Use advanced settings for better scanning");
                ui.label("📊 5. View scan history and results");

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("✓ Got it!").clicked() {
                        self.show_tutorial = false;
                        self.first_time_user = false;
                    }

                    if ui.button("Skip Tutorial").clicked() {
                        self.show_tutorial = false;
                    }
                });
            });
    }

    /// Show help overlay window
    fn show_help_overlay_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("❓ Scanner Help")
            .collapsible(true)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Scanner Help & Tips");
                ui.separator();

                ui.collapsing("📷 Camera Tips", |ui| {
                    ui.label("• Ensure good lighting for better scan results");
                    ui.label("• Hold the camera steady and parallel to the barcode");
                    ui.label("• Use the zoom feature for small barcodes");
                    ui.label("• Enable flash in low-light conditions");
                });

                ui.collapsing("🔍 Scanning Tips", |ui| {
                    ui.label("• Fill the scan area with the barcode");
                    ui.label("• Ensure the barcode is not damaged or dirty");
                    ui.label("• Try different angles if scanning fails");
                    ui.label("• Use manual search if barcode scanning doesn't work");
                });

                ui.collapsing("⚙️ Advanced Features", |ui| {
                    ui.label("• Custom scan area: Define specific area to scan");
                    ui.label("• Multi-scan mode: Scan multiple barcodes quickly");
                    ui.label("• Focus modes: Choose appropriate focus for distance");
                    ui.label("• Resolution: Higher resolution for better accuracy");
                });

                ui.separator();

                if ui.button("✖ Close Help").clicked() {
                    self.show_help_overlay = false;
                }
            });
    }
    fn show_scanning_section(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.label("🔍 Barcode Scanning");

        ui.horizontal(|ui| {
            // Scan button
            if self.scanner_service.is_camera_running() {
                let can_scan = !self.is_scanning
                    && self
                        .last_scan_time
                        .is_none_or(|t| t.elapsed() >= self.scan_cooldown);

                if ui
                    .add_enabled(can_scan, egui::Button::new("📷 Scan Now"))
                    .clicked()
                {
                    self.perform_scan();
                }
            } else {
                ui.add_enabled(false, egui::Button::new("📷 Scan Now"));
                ui.label("(Start camera first)");
            }

            // Auto-scan toggle
            ui.checkbox(&mut self.is_scanning, "🔄 Auto Scan");
        });

        // Status display
        ui.horizontal(|ui| {
            ui.label("Status:");
            if let Some(ref error) = self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            } else {
                ui.label(&self.status_message);
            }
        });

        // Scanning indicator
        if self.is_scanning {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Scanning for barcodes...");
            });
        }

        // Request repaint for auto-scanning
        if self.is_scanning {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    /// Show results section
    fn show_results_section(&mut self, ui: &mut egui::Ui) {
        ui.label("📊 Scan Results");

        if let Some(ref scan_result) = self.current_scan {
            ui.group(|ui| {
                ui.label("🏷️ Barcode Details");
                ui.horizontal(|ui| {
                    ui.label("Barcode:");
                    ui.monospace(&scan_result.barcode);
                });
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ui.label(format!("{:?}", scan_result.barcode_type));
                });
                ui.horizontal(|ui| {
                    ui.label("Confidence:");
                    ui.label(format!("{:.1}%", scan_result.confidence * 100.0));
                });
            });
        }

        if let Some(ref product) = self.current_product {
            ui.group(|ui| {
                ui.label("🏪 Product Information");
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.label(&product.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Category:");
                    ui.label(&product.category);
                });
                ui.horizontal(|ui| {
                    ui.label("Description:");
                    ui.label(&product.description);
                });

                ui.horizontal(|ui| {
                    if ui.button("📋 View Details").clicked() {
                        // In a real app, this would open a product detail view
                        self.status_message = format!("Viewing details for {}", product.name);
                    }

                    if ui.button("⭐ Add to Favorites").clicked() {
                        self.status_message = format!("Added {} to favorites", product.name);
                    }

                    if ui.button("💰 Check Prices").clicked() {
                        self.status_message = format!("Checking prices for {}", product.name);
                    }
                });
            });
        } else if self.current_scan.is_some() {
            ui.group(|ui| {
                ui.label("❌ Product Not Found");
                ui.label("No product information available for this barcode.");

                if ui.button("➕ Add New Product").clicked() {
                    self.status_message = "Opening product creation form...".to_string();
                }
            });
        }
    }

    /// Show manual search section
    fn show_manual_search_section(&mut self, ui: &mut egui::Ui) {
        ui.label("🔍 Manual Product Search");

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.manual_search_query);

            if ui.button("🔍 Search").clicked() {
                self.perform_manual_search();
            }
        });

        // Search results
        if !self.search_results.is_empty() {
            ui.group(|ui| {
                ui.label(format!("📋 Search Results ({})", self.search_results.len()));

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (index, result) in self.search_results.iter().enumerate() {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&result.product.name);
                                    ui.label(format!("({:.1}% match)", result.similarity * 100.0));

                                    if ui.small_button("Select").clicked() {
                                        self.current_product = Some(result.product.clone());
                                        self.status_message =
                                            format!("Selected {}", result.product.name);
                                    }
                                });

                                ui.label(format!("Category: {}", result.product.category));
                                ui.label(format!("Description: {}", result.product.description));
                            });

                            if index < self.search_results.len() - 1 {
                                ui.separator();
                            }
                        }
                    });
            });
        }
    }

    /// Show manual barcode input section
    fn show_manual_barcode_section(&mut self, ui: &mut egui::Ui) {
        ui.label("⌨️ Manual Barcode Input");

        ui.horizontal(|ui| {
            ui.label("Barcode:");
            ui.text_edit_singleline(&mut self.manual_barcode_input);

            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.manual_barcode_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.manual_barcode_type,
                        BarcodeType::Ean13,
                        "EAN-13",
                    );
                    // UPC 未在模型中定义，暂不提供
                    ui.selectable_value(&mut self.manual_barcode_type, BarcodeType::Ean8, "EAN-8");
                    ui.selectable_value(
                        &mut self.manual_barcode_type,
                        BarcodeType::Code128,
                        "Code128",
                    );
                });

            if ui.button("✔ Validate").clicked() {
                let code = self.manual_barcode_input.trim();
                if validate_barcode(code) {
                    // Optionally compute checksum for EAN/UPC
                    if let Some(chk) = generate_barcode_checksum(code) {
                        self.manual_barcode_info = Some(format!("Valid. Checksum={}", chk));
                    } else {
                        self.manual_barcode_info = Some("Valid".to_string());
                    }
                } else {
                    self.manual_barcode_info = Some("Invalid barcode".to_string());
                }
            }

            if ui.button("🔎 Lookup").clicked() {
                let code = self.manual_barcode_input.trim();
                if code.is_empty() {
                    self.error_message = Some("Please enter a barcode".to_string());
                } else {
                    match self.scanner_service.matcher().find_product_by_barcode(code) {
                        Ok(Some(product)) => {
                            self.current_product = Some(product.clone());
                            self.status_message = format!("Product found by barcode: {}", code);
                            self.error_message = None;
                        }
                        Ok(None) => {
                            self.current_product = None;
                            self.status_message = format!("No product for barcode: {}", code);
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Lookup failed: {}", e));
                        }
                    }
                }
            }
        });

        if let Some(info) = &self.manual_barcode_info {
            ui.label(info);
        }
    }

    /// Show scan history section
    fn show_history_section(&mut self, ui: &mut egui::Ui) {
        ui.label("📜 Scan History");

        ui.horizontal(|ui| {
            ui.label(format!("Total scans: {}", self.scan_history.len()));

            if ui.small_button("🗑️ Clear History").clicked() {
                self.scan_history.clear();
                self.status_message = "Scan history cleared".to_string();
            }
        });

        if !self.scan_history.is_empty() {
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    for (index, history_item) in self.scan_history.iter().enumerate().rev() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("#{}", self.scan_history.len() - index));
                                ui.label(&history_item.barcode);
                                ui.label(format!("{:?}", history_item.barcode_type));

                                if let Some(ref product_name) = history_item.product_name {
                                    ui.label(product_name);
                                }

                                if ui.small_button("📋").clicked() {
                                    // Restore this scan as current
                                    self.current_scan = Some(ScanResult {
                                        barcode: history_item.barcode.clone(),
                                        barcode_type: history_item.barcode_type.clone(),
                                        confidence: 1.0,
                                    });

                                    // Try to find the product again
                                    if let Ok(Some(product)) = self
                                        .scanner_service
                                        .matcher()
                                        .find_product_by_barcode(&history_item.barcode)
                                    {
                                        self.current_product = Some(product);
                                    }
                                }
                            });
                        });
                    }
                });
        }
    }

    /// Update scanning state for auto-scan
    fn update_scanning_state(&mut self) {
        if self.is_scanning && self.scanner_service.is_camera_running() {
            let can_scan = self
                .last_scan_time
                .is_none_or(|t| t.elapsed() >= self.scan_cooldown);

            if can_scan {
                self.perform_scan();
            }
        }
    }

    /// Perform a barcode scan
    fn perform_scan(&mut self) {
        self.last_scan_time = Some(Instant::now());

        match self.scanner_service.scan_and_match() {
            Ok(Some(product)) => {
                // Found both barcode and product
                if let Ok(frame) = self.scanner_service.camera().capture_frame() {
                    if let Ok(scan_result) = self.scanner_service.decoder().decode(&frame) {
                        self.current_scan = Some(scan_result.clone());
                        self.current_product = Some(product.clone());

                        // Add to history
                        self.scan_history.push(ScanHistoryItem {
                            barcode: scan_result.barcode,
                            barcode_type: scan_result.barcode_type,
                            product_name: Some(product.name.clone()),
                            timestamp: Instant::now(),
                        });

                        self.status_message = format!("Found product: {}", product.name);
                        self.error_message = None;
                    }
                }
            }
            Ok(None) => {
                // Found barcode but no matching product
                if let Ok(frame) = self.scanner_service.camera().capture_frame() {
                    if let Ok(scan_result) = self.scanner_service.decoder().decode(&frame) {
                        self.current_scan = Some(scan_result.clone());
                        self.current_product = None;

                        // Add to history
                        self.scan_history.push(ScanHistoryItem {
                            barcode: scan_result.barcode.clone(),
                            barcode_type: scan_result.barcode_type,
                            product_name: None,
                            timestamp: Instant::now(),
                        });

                        self.status_message =
                            format!("Barcode found: {} (no product match)", scan_result.barcode);
                        self.error_message = None;
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Scan failed: {}", e));
                self.status_message = "Scan failed".to_string();
            }
        }
    }

    /// Perform manual search
    fn perform_manual_search(&mut self) {
        if self.manual_search_query.trim().is_empty() {
            self.error_message = Some("Please enter a search query".to_string());
            return;
        }

        match self
            .scanner_service
            .matcher()
            .search_products(&self.manual_search_query)
        {
            Ok(results) => {
                self.search_results = results;
                self.status_message = format!("Found {} search results", self.search_results.len());
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Search failed: {}", e));
                self.search_results.clear();
            }
        }
    }
}

impl Default for ScannerUI {
    fn default() -> Self {
        Self::new()
    }
}

/// Scan history item
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ScanHistoryItem {
    barcode: String,
    barcode_type: BarcodeType,
    product_name: Option<String>,
    timestamp: Instant,
}
