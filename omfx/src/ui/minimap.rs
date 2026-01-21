//! Minimap panel
//!
//! Provides a small overview map showing:
//! - Entity positions as colored markers
//! - Viewport indicator (current camera view)
//! - Click to pan camera to location

use fyrox::{
    core::{
        pool::Handle,
        algebra::Vector2,
        color::Color,
    },
    gui::{
        UiNode,
        widget::WidgetBuilder,
        canvas::CanvasBuilder,
        border::BorderBuilder,
        BuildContext,
        Thickness,
        brush::Brush,
    },
};
use std::collections::HashMap;

/// Minimap entity marker
#[derive(Debug, Clone)]
pub struct MinimapMarker {
    /// Entity ID this marker represents
    pub entity_id: u32,
    /// World position
    pub position: Vector2<f32>,
    /// Marker color
    pub color: Color,
    /// Marker size in pixels
    pub size: f32,
    /// Marker shape (for differentiation)
    pub shape: MarkerShape,
}

/// Marker shapes for different entity types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkerShape {
    /// Circle for players/summons
    #[default]
    Circle,
    /// Square for structures
    Square,
    /// Triangle for projectiles
    Triangle,
    /// Diamond for important entities
    Diamond,
}

impl MinimapMarker {
    /// Create a new minimap marker
    pub fn new(entity_id: u32, position: Vector2<f32>, color: Color) -> Self {
        Self {
            entity_id,
            position,
            color,
            size: 4.0,
            shape: MarkerShape::Circle,
        }
    }

    /// Set marker size
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set marker shape
    pub fn with_shape(mut self, shape: MarkerShape) -> Self {
        self.shape = shape;
        self
    }

    /// Create a player marker (larger, circle)
    pub fn player(entity_id: u32, position: Vector2<f32>, team_color: Color) -> Self {
        Self::new(entity_id, position, team_color)
            .with_size(6.0)
            .with_shape(MarkerShape::Circle)
    }

    /// Create a minion marker (small, circle)
    pub fn minion(entity_id: u32, position: Vector2<f32>, team_color: Color) -> Self {
        Self::new(entity_id, position, team_color)
            .with_size(3.0)
            .with_shape(MarkerShape::Circle)
    }

    /// Create a projectile marker (tiny, triangle)
    pub fn projectile(entity_id: u32, position: Vector2<f32>, color: Color) -> Self {
        Self::new(entity_id, position, color)
            .with_size(2.0)
            .with_shape(MarkerShape::Triangle)
    }

    /// Create a structure marker (medium, square)
    pub fn structure(entity_id: u32, position: Vector2<f32>, team_color: Color) -> Self {
        Self::new(entity_id, position, team_color)
            .with_size(5.0)
            .with_shape(MarkerShape::Square)
    }
}

/// Minimap panel
pub struct MinimapPanel {
    /// Root widget handle
    pub root: Handle<UiNode>,
    /// Canvas for drawing markers
    canvas: Handle<UiNode>,
    /// Map bounds for coordinate conversion (world min)
    pub map_min: Vector2<f32>,
    /// Map bounds for coordinate conversion (world max)
    pub map_max: Vector2<f32>,
    /// Minimap size in pixels
    pub minimap_size: f32,
    /// Entity markers
    markers: HashMap<u32, MinimapMarker>,
    /// Camera viewport rectangle (for showing current view)
    pub viewport: Option<MinimapViewport>,
}

/// Camera viewport indicator on minimap
#[derive(Debug, Clone, Copy)]
pub struct MinimapViewport {
    /// Center position in world coordinates
    pub center: Vector2<f32>,
    /// Width in world units
    pub width: f32,
    /// Height in world units
    pub height: f32,
}

impl MinimapPanel {
    /// Create a new minimap panel
    pub fn new(ctx: &mut BuildContext) -> Self {
        let canvas = CanvasBuilder::new(
            WidgetBuilder::new()
                .with_background(Brush::Solid(Color::from_rgba(20, 40, 20, 200)).into())
        )
        .build(ctx);

        let root = BorderBuilder::new(
            WidgetBuilder::new()
                .with_width(180.0)
                .with_height(180.0)
                .on_column(1)
                .on_row(2)
                .with_child(canvas)
        )
        .with_stroke_thickness(Thickness::uniform(2.0).into())
        .build(ctx);

        Self {
            root,
            canvas,
            map_min: Vector2::new(-1000.0, -1000.0),
            map_max: Vector2::new(1000.0, 1000.0),
            minimap_size: 180.0,
            markers: HashMap::new(),
            viewport: None,
        }
    }

    /// Set the world map bounds for coordinate conversion
    pub fn set_map_bounds(&mut self, min: Vector2<f32>, max: Vector2<f32>) {
        self.map_min = min;
        self.map_max = max;
    }

    /// Convert world position to minimap position
    pub fn world_to_minimap(&self, world_pos: Vector2<f32>) -> Vector2<f32> {
        let map_width = self.map_max.x - self.map_min.x;
        let map_height = self.map_max.y - self.map_min.y;

        // Avoid division by zero
        if map_width <= 0.0 || map_height <= 0.0 {
            return Vector2::new(self.minimap_size / 2.0, self.minimap_size / 2.0);
        }

        let norm_x = (world_pos.x - self.map_min.x) / map_width;
        let norm_y = (world_pos.y - self.map_min.y) / map_height;

        Vector2::new(
            norm_x * self.minimap_size,
            (1.0 - norm_y) * self.minimap_size, // Flip Y for screen coords
        )
    }

    /// Convert minimap position to world position
    pub fn minimap_to_world(&self, minimap_pos: Vector2<f32>) -> Vector2<f32> {
        let map_width = self.map_max.x - self.map_min.x;
        let map_height = self.map_max.y - self.map_min.y;

        let norm_x = minimap_pos.x / self.minimap_size;
        let norm_y = 1.0 - (minimap_pos.y / self.minimap_size); // Flip Y back

        Vector2::new(
            self.map_min.x + norm_x * map_width,
            self.map_min.y + norm_y * map_height,
        )
    }

    /// Update all markers at once
    pub fn update_markers(&mut self, markers: Vec<MinimapMarker>) {
        self.markers.clear();
        for marker in markers {
            self.markers.insert(marker.entity_id, marker);
        }
    }

    /// Add or update a single marker
    pub fn add_marker(&mut self, marker: MinimapMarker) {
        self.markers.insert(marker.entity_id, marker);
    }

    /// Update marker position
    pub fn update_marker_position(&mut self, entity_id: u32, position: Vector2<f32>) {
        if let Some(marker) = self.markers.get_mut(&entity_id) {
            marker.position = position;
        }
    }

    /// Remove a marker
    pub fn remove_marker(&mut self, entity_id: u32) {
        self.markers.remove(&entity_id);
    }

    /// Clear all markers
    pub fn clear_markers(&mut self) {
        self.markers.clear();
    }

    /// Get marker by entity ID
    pub fn get_marker(&self, entity_id: u32) -> Option<&MinimapMarker> {
        self.markers.get(&entity_id)
    }

    /// Get all markers
    pub fn markers(&self) -> impl Iterator<Item = &MinimapMarker> {
        self.markers.values()
    }

    /// Set the camera viewport indicator
    pub fn set_viewport(&mut self, viewport: Option<MinimapViewport>) {
        self.viewport = viewport;
    }

    /// Update viewport from camera position and size
    pub fn update_viewport(&mut self, center: Vector2<f32>, width: f32, height: f32) {
        self.viewport = Some(MinimapViewport {
            center,
            width,
            height,
        });
    }

    /// Check if a point (in minimap coords) is within the minimap bounds
    pub fn contains_point(&self, minimap_pos: Vector2<f32>) -> bool {
        minimap_pos.x >= 0.0
            && minimap_pos.x <= self.minimap_size
            && minimap_pos.y >= 0.0
            && minimap_pos.y <= self.minimap_size
    }

    /// Get canvas handle for custom rendering
    pub fn canvas(&self) -> Handle<UiNode> {
        self.canvas
    }
}

/// Standard colors for minimap markers
pub mod colors {
    use fyrox::core::color::Color;

    /// Blue team color
    pub const BLUE_TEAM: Color = Color::from_rgba(80, 140, 220, 255);
    /// Red team color
    pub const RED_TEAM: Color = Color::from_rgba(220, 80, 80, 255);
    /// Neutral color
    pub const NEUTRAL: Color = Color::from_rgba(180, 180, 100, 255);
    /// Self/Local player highlight
    pub const SELF_PLAYER: Color = Color::from_rgba(100, 255, 100, 255);
    /// Projectile color
    pub const PROJECTILE: Color = Color::from_rgba(255, 200, 100, 255);
    /// Structure color
    pub const STRUCTURE: Color = Color::from_rgba(160, 160, 160, 255);
}
