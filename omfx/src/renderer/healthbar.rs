//! Health bar rendering system

use fyrox::{
    core::{
        pool::Handle,
        algebra::{Vector2, Vector3},
        color::Color,
    },
    graph::BaseSceneGraph,
    scene::{
        Scene,
        node::Node,
        base::BaseBuilder,
        dim2::rectangle::RectangleBuilder,
        transform::TransformBuilder,
    },
};
use std::collections::HashMap;

/// Health bar configuration
#[derive(Debug, Clone)]
pub struct HealthBarConfig {
    /// Bar width in world units
    pub width: f32,
    /// Bar height in world units
    pub height: f32,
    /// Offset above entity center
    pub y_offset: f32,
    /// Background color (dark)
    pub background_color: Color,
    /// Border color
    pub border_color: Color,
    /// High health color (green)
    pub high_health_color: Color,
    /// Medium health color (yellow)
    pub medium_health_color: Color,
    /// Low health color (red)
    pub low_health_color: Color,
    /// Threshold for medium health (percentage)
    pub medium_threshold: f32,
    /// Threshold for low health (percentage)
    pub low_threshold: f32,
}

impl Default for HealthBarConfig {
    fn default() -> Self {
        Self {
            width: 40.0,
            height: 6.0,
            y_offset: 25.0,
            background_color: Color::from_rgba(40, 40, 40, 200),
            border_color: Color::from_rgba(0, 0, 0, 255),
            high_health_color: Color::from_rgba(50, 200, 50, 255),    // Green
            medium_health_color: Color::from_rgba(200, 200, 50, 255), // Yellow
            low_health_color: Color::from_rgba(200, 50, 50, 255),     // Red
            medium_threshold: 0.6,
            low_threshold: 0.3,
        }
    }
}

/// Health bar data for an entity
#[derive(Debug, Clone)]
pub struct HealthBarData {
    pub entity_id: u32,
    pub position: Vector2<f32>,
    pub current_health: f32,
    pub max_health: f32,
    pub show_name: bool,
    pub name: Option<String>,
}

impl HealthBarData {
    pub fn health_percentage(&self) -> f32 {
        if self.max_health <= 0.0 {
            0.0
        } else {
            (self.current_health / self.max_health).clamp(0.0, 1.0)
        }
    }
}

/// Health bar node group (background + foreground)
struct HealthBarNodes {
    background: Handle<Node>,
    foreground: Handle<Node>,
}

/// Health bar renderer
pub struct HealthBarRenderer {
    /// Configuration
    pub config: HealthBarConfig,
    /// Health bar nodes per entity
    health_bar_nodes: HashMap<u32, HealthBarNodes>,
    /// Active health bar data
    health_bars: HashMap<u32, HealthBarData>,
}

impl HealthBarRenderer {
    pub fn new() -> Self {
        Self {
            config: HealthBarConfig::default(),
            health_bar_nodes: HashMap::new(),
            health_bars: HashMap::new(),
        }
    }

    /// Update or add health bar for an entity
    pub fn update_health_bar(&mut self, data: HealthBarData) {
        self.health_bars.insert(data.entity_id, data);
    }

    /// Remove health bar for an entity
    pub fn remove_health_bar(&mut self, entity_id: u32, scene: &mut Scene) {
        self.health_bars.remove(&entity_id);
        if let Some(nodes) = self.health_bar_nodes.remove(&entity_id) {
            scene.graph.remove_node(nodes.background);
            scene.graph.remove_node(nodes.foreground);
        }
    }

    /// Get color based on health percentage
    fn get_health_color(&self, percentage: f32) -> Color {
        if percentage <= self.config.low_threshold {
            self.config.low_health_color
        } else if percentage <= self.config.medium_threshold {
            // Interpolate between low and medium
            let t = (percentage - self.config.low_threshold)
                / (self.config.medium_threshold - self.config.low_threshold);
            Self::lerp_color(self.config.low_health_color, self.config.medium_health_color, t)
        } else {
            // Interpolate between medium and high
            let t = (percentage - self.config.medium_threshold)
                / (1.0 - self.config.medium_threshold);
            Self::lerp_color(self.config.medium_health_color, self.config.high_health_color, t)
        }
    }

    /// Linear interpolation between two colors
    fn lerp_color(a: Color, b: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::from_rgba(
            ((a.r as f32) * (1.0 - t) + (b.r as f32) * t) as u8,
            ((a.g as f32) * (1.0 - t) + (b.g as f32) * t) as u8,
            ((a.b as f32) * (1.0 - t) + (b.b as f32) * t) as u8,
            ((a.a as f32) * (1.0 - t) + (b.a as f32) * t) as u8,
        )
    }

    /// Render all health bars
    pub fn render(&mut self, scene: &mut Scene) {
        for (entity_id, data) in &self.health_bars {
            let percentage = data.health_percentage();
            let health_color = self.get_health_color(percentage);
            let bar_pos = Vector2::new(
                data.position.x,
                data.position.y + self.config.y_offset,
            );

            if let Some(nodes) = self.health_bar_nodes.get(entity_id) {
                // Update existing nodes
                // Update background position
                if let Some(bg_node) = scene.graph.try_get_mut(nodes.background) {
                    bg_node.local_transform_mut().set_position(Vector3::new(
                        bar_pos.x,
                        bar_pos.y,
                        0.3,
                    ));
                }

                // Update foreground position, scale, and color
                if let Some(fg_node) = scene.graph.try_get_mut(nodes.foreground) {
                    let fg_width = self.config.width * percentage;
                    let fg_x = bar_pos.x - (self.config.width - fg_width) / 2.0;

                    fg_node.local_transform_mut().set_position(Vector3::new(
                        fg_x,
                        bar_pos.y,
                        0.31,
                    ));
                    fg_node.local_transform_mut().set_scale(Vector3::new(
                        fg_width.max(0.1),
                        self.config.height - 2.0,
                        1.0,
                    ));
                }
            } else {
                // Create new health bar nodes
                // Background (full width, dark)
                let background = RectangleBuilder::new(
                    BaseBuilder::new()
                        .with_local_transform(
                            TransformBuilder::new()
                                .with_local_position(Vector3::new(bar_pos.x, bar_pos.y, 0.3))
                                .with_local_scale(Vector3::new(
                                    self.config.width,
                                    self.config.height,
                                    1.0,
                                ))
                                .build()
                        )
                )
                .with_color(self.config.background_color)
                .build(&mut scene.graph);

                // Foreground (health portion, colored)
                let fg_width = self.config.width * percentage;
                let fg_x = bar_pos.x - (self.config.width - fg_width) / 2.0;

                let foreground = RectangleBuilder::new(
                    BaseBuilder::new()
                        .with_local_transform(
                            TransformBuilder::new()
                                .with_local_position(Vector3::new(fg_x, bar_pos.y, 0.31))
                                .with_local_scale(Vector3::new(
                                    fg_width.max(0.1),
                                    self.config.height - 2.0,
                                    1.0,
                                ))
                                .build()
                        )
                )
                .with_color(health_color)
                .build(&mut scene.graph);

                self.health_bar_nodes.insert(*entity_id, HealthBarNodes {
                    background,
                    foreground,
                });
            }
        }

        // Remove stale health bar nodes
        let active_ids: std::collections::HashSet<u32> = self.health_bars.keys().copied().collect();
        let stale_ids: Vec<u32> = self.health_bar_nodes
            .keys()
            .filter(|id| !active_ids.contains(id))
            .copied()
            .collect();

        for id in stale_ids {
            if let Some(nodes) = self.health_bar_nodes.remove(&id) {
                scene.graph.remove_node(nodes.background);
                scene.graph.remove_node(nodes.foreground);
            }
        }
    }

    /// Sync health bars with game state entities
    pub fn sync_with_entities<'a, I>(&mut self, entities: I, scene: &mut Scene)
    where
        I: Iterator<Item = (u32, Vector2<f32>, (f32, f32), Option<&'a str>)>,
    {
        // Clear current health bars
        self.health_bars.clear();

        // Add health bars for each entity
        for (entity_id, position, (current, max), name) in entities {
            self.health_bars.insert(entity_id, HealthBarData {
                entity_id,
                position,
                current_health: current,
                max_health: max,
                show_name: name.is_some(),
                name: name.map(|s| s.to_string()),
            });
        }

        self.render(scene);
    }

    /// Clear all health bars
    pub fn clear(&mut self, scene: &mut Scene) {
        for (_, nodes) in self.health_bar_nodes.drain() {
            scene.graph.remove_node(nodes.background);
            scene.graph.remove_node(nodes.foreground);
        }
        self.health_bars.clear();
    }
}

impl Default for HealthBarRenderer {
    fn default() -> Self {
        Self::new()
    }
}
