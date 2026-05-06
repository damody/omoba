//! 相機和螢幕邊界的視窗管理

use vek::Vec2;

/// 代表可見區域的視窗
#[derive(Debug, Clone)]
pub struct Viewport {
    pub center: Vec2<f32>,
    pub width: f32,
    pub height: f32,
    pub zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            center: Vec2::zero(),
            width: 1920.0,
            height: 1080.0,
            zoom: 1.0,
        }
    }
}

impl Viewport {
    /// 取得視口邊界（最小、最大）
    pub fn get_bounds(&self) -> (Vec2<f32>, Vec2<f32>) {
        let half_width = self.width / (2.0 * self.zoom);
        let half_height = self.height / (2.0 * self.zoom);

        let min = Vec2::new(
            self.center.x - half_width,
            self.center.y - half_height,
        );
        let max = Vec2::new(
            self.center.x + half_width,
            self.center.y + half_height,
        );

        (min, max)
    }

    /// 跟隨玩家位置
    pub fn follow_player(&mut self, player_pos: Vec2<f32>) {
        self.center = player_pos;
    }

    /// 設定縮放等級（限制為 0.5 - 3.0）
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.5, 3.0);
    }

    /// 設定視口大小
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// 檢查某點在視窗中是否可見
    pub fn contains(&self, point: Vec2<f32>) -> bool {
        let (min, max) = self.get_bounds();
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }

    /// 按 delta 平移視窗
    pub fn pan(&mut self, delta: Vec2<f32>) {
        self.center += delta;
    }
}
