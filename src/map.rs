use egui::{Rect, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MapState {
    pub center_lat: f64,
    pub center_lng: f64,
    pub zoom: f64,
    pub markers: Vec<MapMarker>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MapMarker {
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub is_selected: bool,
}

impl Default for MapState {
    fn default() -> Self {
        Self {
            center_lat: 35.6812, // 东京
            center_lng: 139.7671,
            zoom: 1.0,
            markers: Vec::new(),
        }
    }
}

impl MapState {
    pub fn add_marker(&mut self, marker: MapMarker) {
        self.markers.push(marker);
    }

    pub fn clear_markers(&mut self) {
        self.markers.clear();
    }

    pub fn select_marker(&mut self, id: &str) {
        for marker in &mut self.markers {
            marker.is_selected = marker.id == id;
        }
    }

    pub fn render(&self, ui: &mut egui::Ui, rect: Rect) {
        let html = r#"
        <iframe
            width="800"
            height="600"
            style="border:0"
            loading="lazy"
            allowfullscreen
            referrerpolicy="no-referrer-when-downgrade"
            src="https://www.google.com/maps/embed/v1/view?key=YOUR_API_KEY&center=35.6895,139.6917&zoom=10">
        </iframe>
    "#;

        ui.horizontal(|ui| {
            ui.label("嵌入 Google Maps：");
            ui.add(egui::widgets::TextEdit::multiline(&mut html.to_string()).desired_rows(6));
        });
    }

    pub fn handle_drag(&mut self, delta: Vec2) {
        let scale = 2.0f64.powf(self.zoom);
        self.center_lng += delta.x as f64 / (100.0 * scale);
        self.center_lat -= delta.y as f64 / (100.0 * scale);
    }

    pub fn handle_zoom(&mut self, delta: f32) {
        self.zoom += delta as f64 * 0.1;
        self.zoom = self.zoom.clamp(0.0, 5.0);
    }
}
