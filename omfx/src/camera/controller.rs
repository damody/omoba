//! Camera controller with edge scrolling

use fyrox::core::algebra::{Vector2, Vector3};
use log::debug;

/// Edge scrolling configuration
#[derive(Debug, Clone)]
pub struct EdgeScrollConfig {
    /// Edge trigger zone in pixels
    pub edge_size: f32,
    /// Maximum scroll speed (units per second)
    pub max_speed: f32,
    /// Zoom speed multiplier
    pub zoom_speed: f32,
    /// Minimum zoom level
    pub min_zoom: f32,
    /// Maximum zoom level
    pub max_zoom: f32,
}

impl Default for EdgeScrollConfig {
    fn default() -> Self {
        Self {
            edge_size: 20.0,
            max_speed: 800.0,
            zoom_speed: 0.1,
            min_zoom: 0.5,
            max_zoom: 3.0,
        }
    }
}

/// Camera controller
pub struct CameraController {
    /// Camera position (center of view)
    pub position: Vector2<f32>,
    /// Zoom level
    pub zoom: f32,
    /// Window size
    pub window_size: Vector2<f32>,
    /// Current mouse position
    pub mouse_position: Vector2<f32>,
    /// Edge scroll configuration
    pub config: EdgeScrollConfig,
    /// Target entity to follow (if any)
    pub follow_target: Option<Vector2<f32>>,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            position: Vector2::new(400.0, 0.0),
            zoom: 1.0,
            window_size: Vector2::new(1920.0, 1080.0),
            mouse_position: Vector2::new(960.0, 540.0), // Center of default 1920x1080 window
            config: EdgeScrollConfig::default(),
            follow_target: None,
        }
    }

    /// Update camera position based on edge scrolling
    pub fn update(&mut self, dt: f32) {
        // If following a target, center on it
        if let Some(target) = self.follow_target {
            self.position = target;
            return;
        }

        // Calculate edge scroll velocity
        let scroll_velocity = self.calculate_edge_scroll_velocity();

        // Apply scroll
        self.position += scroll_velocity * dt;
    }

    /// Calculate scroll velocity based on mouse position at edges
    fn calculate_edge_scroll_velocity(&self) -> Vector2<f32> {
        let mut velocity = Vector2::new(0.0, 0.0);
        let edge = self.config.edge_size;
        let max_speed = self.config.max_speed;

        // Left edge
        if self.mouse_position.x < edge {
            let factor = 1.0 - (self.mouse_position.x / edge);
            velocity.x = -max_speed * factor;
        }
        // Right edge
        else if self.mouse_position.x > self.window_size.x - edge {
            let factor = (self.mouse_position.x - (self.window_size.x - edge)) / edge;
            velocity.x = max_speed * factor;
        }

        // Top edge (note: in screen coords, top is lower y)
        if self.mouse_position.y < edge {
            let factor = 1.0 - (self.mouse_position.y / edge);
            velocity.y = max_speed * factor; // Move up in world coords
        }
        // Bottom edge
        else if self.mouse_position.y > self.window_size.y - edge {
            let factor = (self.mouse_position.y - (self.window_size.y - edge)) / edge;
            velocity.y = -max_speed * factor; // Move down in world coords
        }

        velocity
    }

    /// Handle mouse position update
    pub fn on_mouse_move(&mut self, position: Vector2<f64>) {
        self.mouse_position = Vector2::new(position.x as f32, position.y as f32);
    }

    /// Handle zoom (mouse wheel)
    pub fn on_zoom(&mut self, delta: f32) {
        self.zoom += delta * self.config.zoom_speed;
        self.zoom = self.zoom.clamp(self.config.min_zoom, self.config.max_zoom);
        debug!("Zoom: {:.2}x", self.zoom);
    }

    /// Handle middle mouse button drag
    pub fn on_pan(&mut self, delta: Vector2<f32>) {
        self.position -= delta / self.zoom;
    }

    /// Set follow target
    pub fn set_follow_target(&mut self, target: Option<Vector2<f32>>) {
        self.follow_target = target;
        if target.is_some() {
            debug!("Camera following target");
        } else {
            debug!("Camera free mode");
        }
    }

    /// Focus on a specific position
    pub fn focus_on(&mut self, position: Vector2<f32>) {
        self.follow_target = None;
        self.position = position;
    }

    /// Go to map center
    pub fn go_to_center(&mut self) {
        self.focus_on(Vector2::new(400.0, 0.0));
    }

    /// Update window size
    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.window_size = Vector2::new(width, height);
    }

    /// Get camera transform for Fyrox
    pub fn get_transform(&self) -> Vector3<f32> {
        Vector3::new(self.position.x, self.position.y, 0.0)
    }

    /// Convert screen position to world position
    pub fn screen_to_world(&self, screen_pos: Vector2<f32>) -> Vector2<f32> {
        let world_x = self.position.x + (screen_pos.x - self.window_size.x / 2.0) / self.zoom;
        let world_y = self.position.y - (screen_pos.y - self.window_size.y / 2.0) / self.zoom;

        Vector2::new(world_x, world_y)
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new()
    }
}
