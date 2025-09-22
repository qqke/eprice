use crate::search::filters::{
    AvailabilityFilter, PriceRange, PromotionFilter, SearchFilters, SortField,
};
use crate::search::{SearchEngine, SearchQuery, SearchResult, SearchResultItem};
use crate::services::AppServices;
use egui::{Color32, RichText, Ui};
use std::collections::HashSet;

/// Advanced search UI with filters, sorting, and intelligent suggestions
pub struct AdvancedSearchUI {
    search_engine: SearchEngine,

    // Search state
    search_query: String,
    current_filters: SearchFilters,
    search_results: Option<SearchResult>,
    is_searching: bool,

    // UI state
    show_filters: bool,
    show_suggestions: bool,
    selected_categories: HashSet<String>,
    selected_stores: HashSet<String>,

    // Filter UI state
    min_price_text: String,
    max_price_text: String,
    search_history: Vec<String>,
    quick_filters: Vec<QuickFilter>,

    // Advanced features
    voice_search_enabled: bool,
    auto_complete_enabled: bool,
    save_search_enabled: bool,
    search_analytics: SearchAnalytics,
}

#[derive(Debug, Clone)]
struct QuickFilter {
    name: String,
    #[allow(dead_code)]
    description: String,
    filters: SearchFilters,
    icon: String,
}

#[derive(Debug, Clone)]
struct SearchAnalytics {
    total_searches: u32,
    popular_terms: Vec<(String, u32)>,
    avg_results_count: f32,
    #[allow(dead_code)]
    last_search_time: std::time::Instant,
}

impl AdvancedSearchUI {
    pub fn new() -> Self {
        let mut ui = Self {
            search_engine: SearchEngine::new(),
            search_query: String::new(),
            current_filters: SearchFilters::default(),
            search_results: None,
            is_searching: false,
            show_filters: false,
            show_suggestions: true,
            selected_categories: HashSet::new(),
            selected_stores: HashSet::new(),
            min_price_text: String::new(),
            max_price_text: String::new(),
            search_history: Vec::new(),
            quick_filters: Vec::new(),
            voice_search_enabled: false,
            auto_complete_enabled: true,
            save_search_enabled: true,
            search_analytics: SearchAnalytics {
                total_searches: 0,
                popular_terms: Vec::new(),
                avg_results_count: 0.0,
                last_search_time: std::time::Instant::now(),
            },
        };

        ui.initialize_quick_filters();
        ui
    }

    /// Show the advanced search interface
    pub fn show(&mut self, ui: &mut Ui, app_services: &mut AppServices) {
        ui.heading("🔍 Advanced Search");
        ui.separator();

        // Search analytics summary
        self.show_search_analytics(ui);

        // Main search bar with suggestions
        self.show_search_bar(ui);

        // Quick filters row
        if self.show_quick_filters(ui) {
            self.perform_search();
        }

        // Advanced filters (collapsible)
        self.show_advanced_filters(ui);

        ui.separator();

        // Search results
        self.show_search_results(ui, app_services);

        // Auto-complete suggestions overlay
        if self.auto_complete_enabled && !self.search_query.is_empty() && self.show_suggestions {
            self.show_autocomplete_suggestions(ui);
        }
    }

    fn show_search_analytics(&mut self, ui: &mut Ui) {
        ui.collapsing("📊 Search Analytics", |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Total searches: {}",
                    self.search_analytics.total_searches
                ));
                ui.separator();
                ui.label(format!(
                    "Avg results: {:.1}",
                    self.search_analytics.avg_results_count
                ));
                ui.separator();
                if !self.search_analytics.popular_terms.is_empty() {
                    ui.label(format!(
                        "Top term: {}",
                        self.search_analytics.popular_terms[0].0
                    ));
                }
            });
        });
    }

    fn show_search_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Search input with enhanced features
            let search_response = ui.add_sized(
                [ui.available_width() - 200.0, 32.0],
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search products, categories, or stores...")
                    .desired_width(f32::INFINITY),
            );

            // Voice search button (placeholder)
            if ui
                .add_enabled(self.voice_search_enabled, egui::Button::new("🎤"))
                .clicked()
            {
                self.start_voice_search();
            }

            // Camera search button
            if ui.button("📷").clicked() {
                self.start_camera_search();
            }

            // Search button
            if ui.button("🔍 Search").clicked()
                || search_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
            {
                self.perform_search();
            }

            // Advanced filters toggle
            if ui.button("⚙️ Filters").clicked() {
                self.show_filters = !self.show_filters;
            }
        });

        // Search history dropdown
        if !self.search_history.is_empty() {
            ui.horizontal(|ui| {
                ui.label("Recent:");
                let history_items: Vec<String> =
                    self.search_history.iter().take(3).cloned().collect();
                for (i, history_item) in history_items.iter().enumerate() {
                    if ui.small_button(history_item).clicked() {
                        self.search_query = history_item.clone();
                        self.perform_search();
                    }
                    if i < 2 {
                        ui.label("•");
                    }
                }
            });
        }
    }

    fn show_quick_filters(&mut self, ui: &mut Ui) -> bool {
        let mut should_search = false;
        ui.horizontal_wrapped(|ui| {
            ui.label("Quick filters:");

            let quick_filters: Vec<_> = self.quick_filters.iter().collect();
            for quick_filter in quick_filters {
                if ui
                    .small_button(format!("{} {}", quick_filter.icon, quick_filter.name))
                    .clicked()
                {
                    self.current_filters = quick_filter.filters.clone();
                    should_search = true;
                }
            }

            // Clear filters
            if ui.small_button("❌ Clear").clicked() {
                self.current_filters = SearchFilters::default();
                self.selected_categories.clear();
                self.selected_stores.clear();
                self.min_price_text.clear();
                self.max_price_text.clear();
                should_search = true;
            }
        });
        should_search
    }

    fn show_advanced_filters(&mut self, ui: &mut Ui) {
        if !self.show_filters {
            return;
        }

        ui.group(|ui| {
            ui.label(RichText::new("🎛️ Advanced Filters").strong());

            ui.horizontal(|ui| {
                // Price range filter
                ui.vertical(|ui| {
                    ui.label("Price Range:");
                    ui.horizontal(|ui| {
                        ui.label("Min:");
                        ui.add_sized(
                            [60.0, 20.0],
                            egui::TextEdit::singleline(&mut self.min_price_text).hint_text("0"),
                        );
                        ui.label("Max:");
                        ui.add_sized(
                            [60.0, 20.0],
                            egui::TextEdit::singleline(&mut self.max_price_text).hint_text("∞"),
                        );
                    });
                });

                ui.separator();

                // Category filter
                ui.vertical(|ui| {
                    ui.label("Categories:");
                    ui.horizontal_wrapped(|ui| {
                        let categories = vec!["饮料", "食品", "电子产品", "日用品", "服装"];
                        for category in categories {
                            let mut selected = self.selected_categories.contains(category);
                            if ui.checkbox(&mut selected, category).changed() {
                                if selected {
                                    self.selected_categories.insert(category.to_string());
                                } else {
                                    self.selected_categories.remove(category);
                                }
                            }
                        }
                    });
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                // Availability filter
                ui.vertical(|ui| {
                    ui.label("Availability:");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.current_filters.availability))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.current_filters.availability,
                                AvailabilityFilter::All,
                                "All",
                            );
                            ui.selectable_value(
                                &mut self.current_filters.availability,
                                AvailabilityFilter::InStock,
                                "In Stock",
                            );
                            ui.selectable_value(
                                &mut self.current_filters.availability,
                                AvailabilityFilter::OutOfStock,
                                "Out of Stock",
                            );
                        });
                });

                ui.separator();

                // Promotion filter
                ui.vertical(|ui| {
                    ui.label("Promotions:");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.current_filters.promotion_filter))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.current_filters.promotion_filter,
                                PromotionFilter::All,
                                "All",
                            );
                            ui.selectable_value(
                                &mut self.current_filters.promotion_filter,
                                PromotionFilter::OnSale,
                                "On Sale",
                            );
                            ui.selectable_value(
                                &mut self.current_filters.promotion_filter,
                                PromotionFilter::HasCoupon,
                                "Has Coupon",
                            );
                        });
                });

                ui.separator();

                // Sort options
                ui.vertical(|ui| {
                    ui.label("Sort by:");
                    egui::ComboBox::from_label("")
                        .selected_text(format!(
                            "{:?}",
                            self.current_filters.sort_options.primary_sort
                        ))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.current_filters.sort_options.primary_sort,
                                SortField::Relevance,
                                "Relevance",
                            );
                            ui.selectable_value(
                                &mut self.current_filters.sort_options.primary_sort,
                                SortField::Price,
                                "Price",
                            );
                            ui.selectable_value(
                                &mut self.current_filters.sort_options.primary_sort,
                                SortField::Name,
                                "Name",
                            );
                            ui.selectable_value(
                                &mut self.current_filters.sort_options.primary_sort,
                                SortField::Rating,
                                "Rating",
                            );
                        });
                });
            });

            // Apply filters button
            ui.horizontal(|ui| {
                if ui.button("🔍 Apply Filters").clicked() {
                    self.apply_filters();
                    self.perform_search();
                }

                if ui.button("💾 Save Search").clicked() && self.save_search_enabled {
                    self.save_current_search();
                }

                if ui.button("📤 Share Search").clicked() {
                    self.share_search();
                }
            });
        });
    }

    fn show_search_results(&mut self, ui: &mut Ui, _app_services: &mut AppServices) {
        if let Some(ref results) = self.search_results {
            let suggestions: Vec<String> = results.suggestions.clone();
            let total_count = results.total_count;
            let search_time_ms = results.search_time_ms;
            let items: Vec<_> = results.items.iter().collect();

            let mut should_search = false;
            let mut new_query = String::new();

            ui.horizontal(|ui| {
                ui.label(format!("Found {} results", total_count));
                ui.separator();
                ui.label(format!("Search time: {}ms", search_time_ms));

                if !suggestions.is_empty() {
                    ui.separator();
                    ui.label("Suggestions:");
                    for suggestion in &suggestions {
                        if ui.small_button(suggestion).clicked() {
                            new_query = suggestion.clone();
                            should_search = true;
                        }
                    }
                }
            });

            ui.separator();

            // Results grid
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for item in items {
                        self.show_search_result_item(ui, item);
                        ui.separator();
                    }
                });

            // Facets sidebar
            self.show_search_facets(ui, results);

            // Perform search after UI updates
            if should_search {
                self.search_query = new_query;
                self.perform_search();
            }
        } else if self.is_searching {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Searching...");
            });
        } else if !self.search_query.is_empty() {
            ui.label("No results found. Try adjusting your search terms or filters.");
        }
    }

    fn show_search_result_item(&self, ui: &mut Ui, item: &SearchResultItem) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                // Product info
                ui.vertical(|ui| {
                    ui.label(RichText::new(&item.product.name).strong());
                    ui.label(format!("Category: {}", item.product.category));
                    ui.label(&item.product.description);

                    // Match reasons
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Matches:");
                        for reason in &item.match_reasons {
                            match reason {
                                crate::search::engine::MatchReason::NameMatch(score) => {
                                    ui.colored_label(
                                        Color32::GREEN,
                                        format!("Name ({:.1}%)", score * 100.0),
                                    );
                                }
                                crate::search::engine::MatchReason::CategoryMatch(score) => {
                                    ui.colored_label(
                                        Color32::BLUE,
                                        format!("Category ({:.1}%)", score * 100.0),
                                    );
                                }
                                crate::search::engine::MatchReason::TagMatch(tag, score) => {
                                    ui.colored_label(
                                        Color32::YELLOW,
                                        format!("{} ({:.1}%)", tag, score * 100.0),
                                    );
                                }
                                _ => {}
                            }
                        }
                    });
                });

                ui.separator();

                // Price info
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(format!("¥{:.2}", item.price_trend.current_price)).strong(),
                    );

                    if let Some(change_24h) = item.price_trend.price_change_24h {
                        let color = if change_24h > 0.0 {
                            Color32::RED
                        } else {
                            Color32::GREEN
                        };
                        let symbol = if change_24h > 0.0 { "↗" } else { "↘" };
                        ui.colored_label(
                            color,
                            format!("{} ¥{:.2} (24h)", symbol, change_24h.abs()),
                        );
                    }

                    if item.price_trend.is_trending_up {
                        ui.colored_label(Color32::ORANGE, "📈 Trending Up");
                    }
                });

                ui.separator();

                // Availability
                ui.vertical(|ui| {
                    let (status, color) = match item.availability_info.stock_level {
                        crate::search::engine::StockLevel::High => ("✅ In Stock", Color32::GREEN),
                        crate::search::engine::StockLevel::Medium => {
                            ("⚠️ Limited", Color32::YELLOW)
                        }
                        crate::search::engine::StockLevel::Low => ("🔶 Low Stock", Color32::ORANGE),
                        crate::search::engine::StockLevel::OutOfStock => {
                            ("❌ Out of Stock", Color32::RED)
                        }
                        crate::search::engine::StockLevel::Unknown => ("❓ Unknown", Color32::GRAY),
                    };
                    ui.colored_label(color, status);
                    ui.label(format!(
                        "Available at {} stores",
                        item.availability_info.store_count
                    ));
                });

                // Action buttons
                ui.vertical(|ui| {
                    if ui.button("📋 View Details").clicked() {
                        // Open product details
                    }
                    if ui.button("💰 Compare Prices").clicked() {
                        // Open price comparison
                    }
                    if ui.button("⭐ Add to Favorites").clicked() {
                        // Add to favorites
                    }
                });
            });
        });
    }

    fn show_search_facets(&self, ui: &mut Ui, results: &SearchResult) {
        ui.collapsing("🏷️ Refine Results", |ui| {
            // Category facets
            if !results.facets.categories.is_empty() {
                ui.label("Categories:");
                for facet in &results.facets.categories {
                    ui.horizontal(|ui| {
                        let mut selected = facet.selected;
                        ui.checkbox(&mut selected, &facet.name);
                        ui.label(format!("({})", facet.count));
                    });
                }
                ui.separator();
            }

            // Price range facets
            if !results.facets.price_ranges.is_empty() {
                ui.label("Price Ranges:");
                for facet in &results.facets.price_ranges {
                    ui.horizontal(|ui| {
                        if ui.button(&facet.label).clicked() {
                            // Apply price range filter
                        }
                        ui.label(format!("({})", facet.count));
                    });
                }
            }
        });
    }

    fn show_autocomplete_suggestions(&mut self, ui: &mut Ui) {
        if self.search_query.len() > 1 {
            let suggestions = self.search_engine.get_suggestions(&self.search_query, 5);
            if !suggestions.is_empty() {
                ui.group(|ui| {
                    ui.label("Suggestions:");
                    for suggestion in suggestions {
                        if ui.small_button(&suggestion).clicked() {
                            self.search_query = suggestion;
                            self.perform_search();
                        }
                    }
                });
            }
        }
    }

    fn initialize_quick_filters(&mut self) {
        self.quick_filters = vec![
            QuickFilter {
                name: "On Sale".to_string(),
                description: "Products currently on sale".to_string(),
                filters: {
                    SearchFilters {
                        promotion_filter: PromotionFilter::OnSale,
                        ..SearchFilters::default()
                    }
                },
                icon: "🏷️".to_string(),
            },
            QuickFilter {
                name: "Under ¥50".to_string(),
                description: "Products under 50 yuan".to_string(),
                filters: SearchFilters::with_price_range(None, Some(50.0)),
                icon: "💰".to_string(),
            },
            QuickFilter {
                name: "Electronics".to_string(),
                description: "Electronic products".to_string(),
                filters: {
                    let mut filters = SearchFilters::default();
                    filters.add_category(
                        "electronics".to_string(),
                        "Electronics".to_string(),
                        true,
                    );
                    filters
                },
                icon: "📱".to_string(),
            },
        ];
    }

    fn perform_search(&mut self) {
        if self.search_query.trim().is_empty() {
            return;
        }

        self.is_searching = true;

        // Add to search history
        if !self.search_history.contains(&self.search_query) {
            self.search_history.insert(0, self.search_query.clone());
            self.search_history.truncate(10);
        }

        // Create search query
        let query = SearchQuery {
            text: self.search_query.clone(),
            filters: self.current_filters.clone(),
            fuzzy_matching: true,
            include_suggestions: true,
            max_results: 50,
            min_relevance_score: 0.1,
        };

        // Perform search (in real app, this would be async)
        match self.search_engine.search(query) {
            Ok(results) => {
                self.search_results = Some(results);
                self.update_search_analytics();
            }
            Err(_e) => {
                // Handle search error
                self.search_results = None;
            }
        }

        self.is_searching = false;
    }

    fn apply_filters(&mut self) {
        // Apply price range
        if !self.min_price_text.is_empty() || !self.max_price_text.is_empty() {
            let min_price = self.min_price_text.parse().ok();
            let max_price = self.max_price_text.parse().ok();

            self.current_filters.price_range = Some(PriceRange {
                min_price,
                max_price,
                include_sale_prices: true,
                currency: "CNY".to_string(),
            });
        }

        // Apply category filters
        self.current_filters.categories.clear();
        for category in &self.selected_categories {
            self.current_filters
                .add_category(category.to_lowercase(), category.clone(), true);
        }
    }

    fn start_voice_search(&mut self) {
        // Placeholder for voice search functionality
        self.search_query = "Voice search activated".to_string();
    }

    fn start_camera_search(&mut self) {
        // Placeholder for camera search functionality
        self.search_query = "Camera search activated".to_string();
    }

    fn save_current_search(&mut self) {
        // Placeholder for saving search functionality
    }

    fn share_search(&mut self) {
        // Placeholder for sharing search functionality
    }

    fn update_search_analytics(&mut self) {
        self.search_analytics.total_searches += 1;
        if let Some(ref results) = self.search_results {
            self.search_analytics.avg_results_count = (self.search_analytics.avg_results_count
                * (self.search_analytics.total_searches - 1) as f32
                + results.total_count as f32)
                / self.search_analytics.total_searches as f32;
        }
    }
}

impl Default for AdvancedSearchUI {
    fn default() -> Self {
        Self::new()
    }
}
