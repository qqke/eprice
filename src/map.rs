use egui::{Color32, Pos2, Rect, Vec2};
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
        // 绘制地图背景
        ui.painter()
            .rect_filled(rect, 0.0, Color32::from_rgb(200, 200, 200));

        // 计算缩放比例
        let scale = 2.0f64.powf(self.zoom);

        // 绘制标记
        for marker in &self.markers {
            let x = (marker.lng - self.center_lng) * scale * 100.0 + rect.width() as f64 / 2.0;
            let y = (self.center_lat - marker.lat) * scale * 100.0 + rect.height() as f64 / 2.0;

            let pos = Pos2::new(x as f32, y as f32);
            let color = if marker.is_selected {
                Color32::from_rgb(255, 0, 0)
            } else {
                Color32::from_rgb(0, 0, 255)
            };

            // 绘制标记点
            ui.painter().circle_filled(pos, 5.0, color);

            // 绘制商店名称
            ui.painter().text(
                Pos2::new(pos.x + 10.0, pos.y),
                egui::Align2::LEFT_CENTER,
                &marker.name,
                egui::FontId::proportional(12.0),
                Color32::BLACK,
            );
        }
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
