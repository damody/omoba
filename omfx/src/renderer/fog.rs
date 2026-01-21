//! Fog of war rendering system

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
use std::collections::HashSet;

/// Fog of war configuration
#[derive(Debug, Clone)]
pub struct FogConfig {
    /// Tile size in world units
    pub tile_size: f32,
    /// Fog color (semi-transparent black)
    pub fog_color: Color,
    /// Explored but not visible color (darker)
    pub explored_color: Color,
    /// Whether to show vision range circles
    pub show_vision_ranges: bool,
    /// Vision range circle color
    pub vision_range_color: Color,
}

impl Default for FogConfig {
    fn default() -> Self {
        Self {
            tile_size: 32.0,
            fog_color: Color::from_rgba(0, 0, 0, 128),         // 50% black
            explored_color: Color::from_rgba(0, 0, 0, 180),    // 70% black for explored but not visible
            show_vision_ranges: false,
            vision_range_color: Color::from_rgba(255, 255, 255, 60), // Faint white
        }
    }
}

/// Vision source (entity that provides vision)
#[derive(Debug, Clone)]
pub struct VisionSource {
    pub entity_id: u32,
    pub position: Vector2<f32>,
    pub vision_range: f32,
}

/// Fog tile state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FogState {
    /// Never seen
    Unexplored,
    /// Seen before but not currently visible
    Explored,
    /// Currently visible
    Visible,
}

/// Fog of war renderer
pub struct FogOfWarRenderer {
    /// Configuration
    pub config: FogConfig,
    /// Fog tile nodes (grid_x, grid_y) -> node handle
    fog_nodes: std::collections::HashMap<(i32, i32), Handle<Node>>,
    /// Vision range circle nodes
    vision_range_nodes: std::collections::HashMap<u32, Handle<Node>>,
    /// Current fog state per tile
    fog_state: std::collections::HashMap<(i32, i32), FogState>,
    /// Currently visible tiles (updated each frame)
    visible_tiles: HashSet<(i32, i32)>,
    /// Vision sources
    vision_sources: Vec<VisionSource>,
    /// Map bounds for rendering
    map_min: Vector2<f32>,
    map_max: Vector2<f32>,
}

impl FogOfWarRenderer {
    pub fn new() -> Self {
        Self {
            config: FogConfig::default(),
            fog_nodes: std::collections::HashMap::new(),
            vision_range_nodes: std::collections::HashMap::new(),
            fog_state: std::collections::HashMap::new(),
            visible_tiles: HashSet::new(),
            vision_sources: Vec::new(),
            map_min: Vector2::new(-1000.0, -1000.0),
            map_max: Vector2::new(1000.0, 1000.0),
        }
    }

    /// Set map bounds
    pub fn set_map_bounds(&mut self, min: Vector2<f32>, max: Vector2<f32>) {
        self.map_min = min;
        self.map_max = max;
    }

    /// Update vision sources
    pub fn set_vision_sources(&mut self, sources: Vec<VisionSource>) {
        self.vision_sources = sources;
    }

    /// Add a vision source
    pub fn add_vision_source(&mut self, entity_id: u32, position: Vector2<f32>, vision_range: f32) {
        // Update existing or add new
        if let Some(source) = self.vision_sources.iter_mut().find(|s| s.entity_id == entity_id) {
            source.position = position;
            source.vision_range = vision_range;
        } else {
            self.vision_sources.push(VisionSource {
                entity_id,
                position,
                vision_range,
            });
        }
    }

    /// Remove a vision source
    pub fn remove_vision_source(&mut self, entity_id: u32, scene: &mut Scene) {
        self.vision_sources.retain(|s| s.entity_id != entity_id);
        if let Some(node) = self.vision_range_nodes.remove(&entity_id) {
            scene.graph.remove_node(node);
        }
    }

    /// Convert world position to grid coordinates
    fn world_to_grid(&self, pos: Vector2<f32>) -> (i32, i32) {
        let grid_x = (pos.x / self.config.tile_size).floor() as i32;
        let grid_y = (pos.y / self.config.tile_size).floor() as i32;
        (grid_x, grid_y)
    }

    /// Convert grid coordinates to world position (center of tile)
    fn grid_to_world(&self, grid_x: i32, grid_y: i32) -> Vector2<f32> {
        Vector2::new(
            (grid_x as f32 + 0.5) * self.config.tile_size,
            (grid_y as f32 + 0.5) * self.config.tile_size,
        )
    }

    /// Calculate which tiles are visible from vision sources
    fn calculate_visible_tiles(&mut self) {
        self.visible_tiles.clear();

        for source in &self.vision_sources {
            let (center_x, center_y) = self.world_to_grid(source.position);
            let range_tiles = (source.vision_range / self.config.tile_size).ceil() as i32;

            // Check all tiles within range
            for dx in -range_tiles..=range_tiles {
                for dy in -range_tiles..=range_tiles {
                    let tile_x = center_x + dx;
                    let tile_y = center_y + dy;
                    let tile_center = self.grid_to_world(tile_x, tile_y);

                    // Check if tile center is within vision range
                    let dist_sq = (tile_center.x - source.position.x).powi(2)
                        + (tile_center.y - source.position.y).powi(2);

                    if dist_sq <= source.vision_range.powi(2) {
                        self.visible_tiles.insert((tile_x, tile_y));
                    }
                }
            }
        }
    }

    /// Update fog state based on visibility
    fn update_fog_state(&mut self) {
        // Mark currently visible tiles
        for &tile in &self.visible_tiles {
            self.fog_state.insert(tile, FogState::Visible);
        }

        // Downgrade previously visible tiles to explored
        let to_downgrade: Vec<(i32, i32)> = self.fog_state
            .iter()
            .filter(|(tile, state)| **state == FogState::Visible && !self.visible_tiles.contains(tile))
            .map(|(tile, _)| *tile)
            .collect();

        for tile in to_downgrade {
            self.fog_state.insert(tile, FogState::Explored);
        }
    }

    /// Update fog rendering
    pub fn update(&mut self, viewport_min: Vector2<f32>, viewport_max: Vector2<f32>, scene: &mut Scene) {
        self.calculate_visible_tiles();
        self.update_fog_state();

        // Determine grid bounds to render
        let (min_grid_x, min_grid_y) = self.world_to_grid(viewport_min);
        let (max_grid_x, max_grid_y) = self.world_to_grid(viewport_max);

        // Remove fog nodes outside viewport
        let to_remove: Vec<(i32, i32)> = self.fog_nodes
            .keys()
            .filter(|(x, y)| *x < min_grid_x - 1 || *x > max_grid_x + 1 || *y < min_grid_y - 1 || *y > max_grid_y + 1)
            .cloned()
            .collect();

        for key in to_remove {
            if let Some(node) = self.fog_nodes.remove(&key) {
                scene.graph.remove_node(node);
            }
        }

        // Render fog tiles in viewport
        for grid_x in min_grid_x..=max_grid_x {
            for grid_y in min_grid_y..=max_grid_y {
                let tile_key = (grid_x, grid_y);
                let state = self.fog_state.get(&tile_key).copied().unwrap_or(FogState::Unexplored);

                // Only render fog for non-visible tiles
                if state == FogState::Visible {
                    // Remove fog node if it exists
                    if let Some(node) = self.fog_nodes.remove(&tile_key) {
                        scene.graph.remove_node(node);
                    }
                    continue;
                }

                let color = match state {
                    FogState::Unexplored => self.config.fog_color,
                    FogState::Explored => self.config.explored_color,
                    FogState::Visible => continue, // Already handled above
                };

                let tile_pos = self.grid_to_world(grid_x, grid_y);

                if self.fog_nodes.contains_key(&tile_key) {
                    // Node already exists, keep it
                    // (For now, we don't update color dynamically)
                } else {
                    // Create new fog tile
                    let node = RectangleBuilder::new(
                        BaseBuilder::new()
                            .with_local_transform(
                                TransformBuilder::new()
                                    .with_local_position(Vector3::new(tile_pos.x, tile_pos.y, 0.5)) // Above other elements
                                    .with_local_scale(Vector3::new(
                                        self.config.tile_size,
                                        self.config.tile_size,
                                        1.0,
                                    ))
                                    .build()
                            )
                    )
                    .with_color(color)
                    .build(&mut scene.graph);

                    self.fog_nodes.insert(tile_key, node);
                }
            }
        }

        // Render vision range circles if enabled
        if self.config.show_vision_ranges {
            self.render_vision_ranges(scene);
        }
    }

    /// Render vision range circles
    fn render_vision_ranges(&mut self, scene: &mut Scene) {
        for source in &self.vision_sources {
            if let Some(&node_handle) = self.vision_range_nodes.get(&source.entity_id) {
                // Update existing node position
                if let Some(node) = scene.graph.try_get_mut(node_handle) {
                    let transform = node.local_transform_mut();
                    transform.set_position(Vector3::new(source.position.x, source.position.y, 0.45));
                }
            } else {
                // Create new vision range circle
                let node = RectangleBuilder::new(
                    BaseBuilder::new()
                        .with_local_transform(
                            TransformBuilder::new()
                                .with_local_position(Vector3::new(source.position.x, source.position.y, 0.45))
                                .with_local_scale(Vector3::new(
                                    source.vision_range * 2.0,
                                    source.vision_range * 2.0,
                                    1.0,
                                ))
                                .build()
                        )
                )
                .with_color(self.config.vision_range_color)
                .build(&mut scene.graph);

                self.vision_range_nodes.insert(source.entity_id, node);
            }
        }

        // Remove vision range circles for removed sources
        let active_ids: HashSet<u32> = self.vision_sources.iter().map(|s| s.entity_id).collect();
        let to_remove: Vec<u32> = self.vision_range_nodes
            .keys()
            .filter(|id| !active_ids.contains(id))
            .cloned()
            .collect();

        for id in to_remove {
            if let Some(node) = self.vision_range_nodes.remove(&id) {
                scene.graph.remove_node(node);
            }
        }
    }

    /// Toggle vision range display
    pub fn toggle_vision_ranges(&mut self) {
        self.config.show_vision_ranges = !self.config.show_vision_ranges;
    }

    /// Set vision range display
    pub fn set_show_vision_ranges(&mut self, show: bool, scene: &mut Scene) {
        self.config.show_vision_ranges = show;
        if !show {
            // Remove all vision range nodes
            for (_, node) in self.vision_range_nodes.drain() {
                scene.graph.remove_node(node);
            }
        }
    }

    /// Clear all fog
    pub fn clear(&mut self, scene: &mut Scene) {
        for (_, node) in self.fog_nodes.drain() {
            scene.graph.remove_node(node);
        }
        for (_, node) in self.vision_range_nodes.drain() {
            scene.graph.remove_node(node);
        }
        self.fog_state.clear();
        self.visible_tiles.clear();
        self.vision_sources.clear();
    }

    /// Reveal entire map (debug feature)
    pub fn reveal_all(&mut self, scene: &mut Scene) {
        // Clear all fog nodes
        for (_, node) in self.fog_nodes.drain() {
            scene.graph.remove_node(node);
        }
        // Mark all current fog state as visible
        for state in self.fog_state.values_mut() {
            *state = FogState::Visible;
        }
    }
}

impl Default for FogOfWarRenderer {
    fn default() -> Self {
        Self::new()
    }
}
