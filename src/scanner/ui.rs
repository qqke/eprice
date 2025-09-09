use crate::models::Product;
use crate::scanner::{BarcodeType, CameraInfo, ProductMatch, ScanResult, ScannerService};
use eframe::egui;
use std::time::{Duration, Instant};

/// Scanner UI component for barcode scanning interface
pub struct ScannerUI {
    scanner_service: ScannerService,

    // UI State
    is_scanning: bool,
    last_scan_time: Option<Instant>,
    scan_cooldown: Duration,

    // Results
    current_scan: Option<ScanResult>,
    current_product: Option<Product>,
    scan_history: Vec<ScanHistoryItem>,

    // UI Elements
    camera_preview_enabled: bool,
    available_cameras: Vec<CameraInfo>,
    selected_camera: usize,

    // Search functionality
    manual_search_query: String,
    search_results: Vec<ProductMatch>,

    // Status messages
    status_message: String,
    error_message: Option<String>,
}

impl ScannerUI {
    pub fn new() -> Self {
        let available_cameras = ScannerService::default().list_cameras();

        Self {
            scanner_service: ScannerService::new(),
            is_scanning: false,
            last_scan_time: None,
            scan_cooldown: Duration::from_millis(1000), // 1 second cooldown
            current_scan: None,
            current_product: None,
            scan_history: Vec::new(),
            camera_preview_enabled: false,
            available_cameras,
            selected_camera: 0,
            manual_search_query: String::new(),
            search_results: Vec::new(),
            status_message: "Ready to scan".to_string(),
            error_message: None,
        }
    }

    /// Show the scanner UI
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("📱 Barcode Scanner");

        ui.separator();

        // Camera controls section
        self.show_camera_controls(ui);

        ui.separator();

        // Scanning section
        self.show_scanning_section(ctx, ui);

        ui.separator();

        // Results section
        self.show_results_section(ui);

        ui.separator();

        // Manual search section
        self.show_manual_search_section(ui);

        ui.separator();

        // History section
        self.show_history_section(ui);

        // Update scanning state
        self.update_scanning_state();
    }

    /// Show camera controls
    fn show_camera_controls(&mut self, ui: &mut egui::Ui) {
        ui.label("📷 Camera Controls");

        ui.horizontal(|ui| {
            // Camera selection
            ui.label("Camera:");
            egui::ComboBox::from_label("")
                .selected_text(if self.available_cameras.is_empty() {
                    "No cameras available"
                } else if self.selected_camera < self.available_cameras.len() {
                    &self.available_cameras[self.selected_camera].name
                } else {
                    "Select camera"
                })
                .show_ui(ui, |ui| {
                    for (index, camera) in self.available_cameras.iter().enumerate() {
                        ui.selectable_value(&mut self.selected_camera, index, &camera.name);
                    }
                });
        });

        ui.horizontal(|ui| {
            // Start/Stop camera button
            if self.scanner_service.is_camera_running() {
                if ui.button("🛑 Stop Camera").clicked() {
                    if let Err(e) = self.scanner_service.stop_camera() {
                        self.error_message = Some(format!("Failed to stop camera: {}", e));
                    } else {
                        self.status_message = "Camera stopped".to_string();
                        self.is_scanning = false;
                    }
                }
            } else {
                if ui.button("▶️ Start Camera").clicked() {
                    if let Err(e) = self.scanner_service.start_camera() {
                        self.error_message = Some(format!("Failed to start camera: {}", e));
                    } else {
                        self.status_message = "Camera started".to_string();
                        self.error_message = None;
                    }
                }
            }

            // Preview toggle
            ui.checkbox(&mut self.camera_preview_enabled, "📺 Show Preview");
        });

        // Camera preview (placeholder)
        if self.camera_preview_enabled && self.scanner_service.is_camera_running() {
            ui.group(|ui| {
                ui.label("Camera Preview");
                ui.add_sized([200.0, 150.0], egui::widgets::Separator::default());
                ui.label("📹 Live camera feed would appear here");
            });
        }
    }

    /// Show scanning section
    fn show_scanning_section(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.label("🔍 Barcode Scanning");

        ui.horizontal(|ui| {
            // Scan button
            if self.scanner_service.is_camera_running() {
                let can_scan = !self.is_scanning
                    && self
                        .last_scan_time
                        .map_or(true, |t| t.elapsed() >= self.scan_cooldown);

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
                .map_or(true, |t| t.elapsed() >= self.scan_cooldown);

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
struct ScanHistoryItem {
    barcode: String,
    barcode_type: BarcodeType,
    product_name: Option<String>,
    timestamp: Instant,
}
