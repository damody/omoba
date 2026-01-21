//! Debug overlay rendering
//!
//! Provides visual debug overlays for collision boxes, vision ranges,
//! movement paths, and entity labels. Toggled with F1-F4 keys.

use fyrox::{
    core::{algebra::Vector3, color::Color, pool::Handle},
    event::ElementState,
    graph::BaseSceneGraph,
    keyboard::{KeyCode, PhysicalKey},
    scene::{
        base::BaseBuilder, dim2::rectangle::RectangleBuilder, node::Node,
        transform::TransformBuilder, Scene,
    },
};
use std::collections::HashMap;

/// Debug overlay types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverlayType {
    /// F1 - Collision boxes
    CollisionBoxes,
    /// F2 - Vision ranges
    VisionRanges,
    /// F3 - Movement paths/targets
    MovementPaths,
    /// F4 - Entity ID labels
    EntityLabels,
}

/// Debug overlay colors
pub struct OverlayColors;

impl OverlayColors {
    pub const COLLISION_BOX: Color = Color::from_rgba(255, 0, 255, 100); // Magenta
    pub const VISION_RANGE: Color = Color::from_rgba(255, 255, 0, 60); // Yellow
    pub const MOVEMENT_PATH: Color = Color::from_rgba(0, 255, 255, 150); // Cyan
    pub const MOVEMENT_TARGET: Color = Color::from_rgba(255, 100, 0, 200); // Orange
    pub const ENTITY_LABEL_BG: Color = Color::from_rgba(0, 0, 0, 180); // Dark background
}

/// Collision box data
#[derive(Debug, Clone)]
pub struct CollisionBox {
    pub entity_id: u32,
    pub center_x: f32,
    pub center_y: f32,
    pub half_width: f32,
    pub half_height: f32,
}

/// Movement path data
#[derive(Debug, Clone)]
pub struct MovementPath {
    pub entity_id: u32,
    pub current_x: f32,
    pub current_y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub waypoints: Vec<(f32, f32)>,
}

/// Entity label data
#[derive(Debug, Clone)]
pub struct EntityLabel {
    pub entity_id: u32,
    pub position_x: f32,
    pub position_y: f32,
    pub text: String,
}

/// Vision range data
#[derive(Debug, Clone)]
pub struct VisionRange {
    pub entity_id: u32,
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
}

/// Debug overlays manager
pub struct DebugOverlays {
    /// Which overlays are enabled
    enabled: HashMap<OverlayType, bool>,
    /// Collision box nodes
    collision_nodes: HashMap<u32, Handle<Node>>,
    /// Vision range nodes
    vision_nodes: HashMap<u32, Handle<Node>>,
    /// Movement path nodes (entity_id -> (path_nodes, target_node))
    movement_nodes: HashMap<u32, (Vec<Handle<Node>>, Handle<Node>)>,
    /// Entity label nodes
    label_nodes: HashMap<u32, Handle<Node>>,
    /// Current data
    collision_boxes: Vec<CollisionBox>,
    movement_paths: Vec<MovementPath>,
    entity_labels: Vec<EntityLabel>,
    vision_ranges: Vec<VisionRange>,
}

impl DebugOverlays {
    pub fn new() -> Self {
        let mut enabled = HashMap::new();
        enabled.insert(OverlayType::CollisionBoxes, false);
        enabled.insert(OverlayType::VisionRanges, false);
        enabled.insert(OverlayType::MovementPaths, false);
        enabled.insert(OverlayType::EntityLabels, false);

        Self {
            enabled,
            collision_nodes: HashMap::new(),
            vision_nodes: HashMap::new(),
            movement_nodes: HashMap::new(),
            label_nodes: HashMap::new(),
            collision_boxes: Vec::new(),
            movement_paths: Vec::new(),
            entity_labels: Vec::new(),
            vision_ranges: Vec::new(),
        }
    }

    /// Toggle overlay by type
    pub fn toggle(&mut self, overlay_type: OverlayType, scene: &mut Scene) {
        let current = *self.enabled.get(&overlay_type).unwrap_or(&false);
        self.enabled.insert(overlay_type, !current);

        // Clear nodes if disabled
        if current {
            self.clear_overlay_nodes(overlay_type, scene);
        }
    }

    /// Check if overlay is enabled
    pub fn is_enabled(&self, overlay_type: OverlayType) -> bool {
        *self.enabled.get(&overlay_type).unwrap_or(&false)
    }

    /// Handle keyboard input (F1-F4)
    /// Returns true if the key was handled
    pub fn handle_key(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        scene: &mut Scene,
    ) -> bool {
        // Only handle key press, not release
        if state != ElementState::Pressed {
            return false;
        }

        if let PhysicalKey::Code(key_code) = physical_key {
            match key_code {
                KeyCode::F1 => {
                    self.toggle(OverlayType::CollisionBoxes, scene);
                    log::info!(
                        "Collision boxes overlay: {}",
                        if self.is_enabled(OverlayType::CollisionBoxes) {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                    true
                }
                KeyCode::F2 => {
                    self.toggle(OverlayType::VisionRanges, scene);
                    log::info!(
                        "Vision ranges overlay: {}",
                        if self.is_enabled(OverlayType::VisionRanges) {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                    true
                }
                KeyCode::F3 => {
                    self.toggle(OverlayType::MovementPaths, scene);
                    log::info!(
                        "Movement paths overlay: {}",
                        if self.is_enabled(OverlayType::MovementPaths) {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                    true
                }
                KeyCode::F4 => {
                    self.toggle(OverlayType::EntityLabels, scene);
                    log::info!(
                        "Entity labels overlay: {}",
                        if self.is_enabled(OverlayType::EntityLabels) {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Update overlay data
    pub fn set_collision_boxes(&mut self, boxes: Vec<CollisionBox>) {
        self.collision_boxes = boxes;
    }

    pub fn set_movement_paths(&mut self, paths: Vec<MovementPath>) {
        self.movement_paths = paths;
    }

    pub fn set_entity_labels(&mut self, labels: Vec<EntityLabel>) {
        self.entity_labels = labels;
    }

    pub fn set_vision_ranges(&mut self, ranges: Vec<VisionRange>) {
        self.vision_ranges = ranges;
    }

    /// Render all enabled overlays
    pub fn render(&mut self, scene: &mut Scene) {
        if self.is_enabled(OverlayType::CollisionBoxes) {
            self.render_collision_boxes(scene);
        }
        if self.is_enabled(OverlayType::VisionRanges) {
            self.render_vision_ranges(scene);
        }
        if self.is_enabled(OverlayType::MovementPaths) {
            self.render_movement_paths(scene);
        }
        if self.is_enabled(OverlayType::EntityLabels) {
            self.render_entity_labels(scene);
        }
    }

    fn render_collision_boxes(&mut self, scene: &mut Scene) {
        for box_data in &self.collision_boxes {
            if !self.collision_nodes.contains_key(&box_data.entity_id) {
                let node = RectangleBuilder::new(
                    BaseBuilder::new().with_local_transform(
                        TransformBuilder::new()
                            .with_local_position(Vector3::new(
                                box_data.center_x,
                                box_data.center_y,
                                0.6,
                            ))
                            .with_local_scale(Vector3::new(
                                box_data.half_width * 2.0,
                                box_data.half_height * 2.0,
                                1.0,
                            ))
                            .build(),
                    ),
                )
                .with_color(OverlayColors::COLLISION_BOX)
                .build(&mut scene.graph);

                self.collision_nodes.insert(box_data.entity_id, node);
            } else if let Some(&node) = self.collision_nodes.get(&box_data.entity_id) {
                if let Some(n) = scene.graph.try_get_mut(node) {
                    n.local_transform_mut().set_position(Vector3::new(
                        box_data.center_x,
                        box_data.center_y,
                        0.6,
                    ));
                }
            }
        }
    }

    fn render_vision_ranges(&mut self, scene: &mut Scene) {
        for range in &self.vision_ranges {
            if !self.vision_nodes.contains_key(&range.entity_id) {
                let node = RectangleBuilder::new(
                    BaseBuilder::new().with_local_transform(
                        TransformBuilder::new()
                            .with_local_position(Vector3::new(
                                range.center_x,
                                range.center_y,
                                0.55,
                            ))
                            .with_local_scale(Vector3::new(range.radius * 2.0, range.radius * 2.0, 1.0))
                            .build(),
                    ),
                )
                .with_color(OverlayColors::VISION_RANGE)
                .build(&mut scene.graph);

                self.vision_nodes.insert(range.entity_id, node);
            } else if let Some(&node) = self.vision_nodes.get(&range.entity_id) {
                if let Some(n) = scene.graph.try_get_mut(node) {
                    n.local_transform_mut().set_position(Vector3::new(
                        range.center_x,
                        range.center_y,
                        0.55,
                    ));
                }
            }
        }
    }

    fn render_movement_paths(&mut self, scene: &mut Scene) {
        for path in &self.movement_paths {
            // Render target marker
            if !self.movement_nodes.contains_key(&path.entity_id) {
                let target_node = RectangleBuilder::new(
                    BaseBuilder::new().with_local_transform(
                        TransformBuilder::new()
                            .with_local_position(Vector3::new(path.target_x, path.target_y, 0.65))
                            .with_local_scale(Vector3::new(10.0, 10.0, 1.0))
                            .build(),
                    ),
                )
                .with_color(OverlayColors::MOVEMENT_TARGET)
                .build(&mut scene.graph);

                self.movement_nodes
                    .insert(path.entity_id, (Vec::new(), target_node));
            } else if let Some((_, target_node)) = self.movement_nodes.get(&path.entity_id) {
                if let Some(n) = scene.graph.try_get_mut(*target_node) {
                    n.local_transform_mut()
                        .set_position(Vector3::new(path.target_x, path.target_y, 0.65));
                }
            }
        }
    }

    fn render_entity_labels(&mut self, scene: &mut Scene) {
        for label in &self.entity_labels {
            if !self.label_nodes.contains_key(&label.entity_id) {
                // Render label background (text rendering would need UI integration)
                let node = RectangleBuilder::new(
                    BaseBuilder::new().with_local_transform(
                        TransformBuilder::new()
                            .with_local_position(Vector3::new(
                                label.position_x,
                                label.position_y + 35.0,
                                0.7,
                            ))
                            .with_local_scale(Vector3::new(40.0, 15.0, 1.0))
                            .build(),
                    ),
                )
                .with_color(OverlayColors::ENTITY_LABEL_BG)
                .build(&mut scene.graph);

                self.label_nodes.insert(label.entity_id, node);
            } else if let Some(&node) = self.label_nodes.get(&label.entity_id) {
                if let Some(n) = scene.graph.try_get_mut(node) {
                    n.local_transform_mut().set_position(Vector3::new(
                        label.position_x,
                        label.position_y + 35.0,
                        0.7,
                    ));
                }
            }
        }
    }

    fn clear_overlay_nodes(&mut self, overlay_type: OverlayType, scene: &mut Scene) {
        match overlay_type {
            OverlayType::CollisionBoxes => {
                for (_, node) in self.collision_nodes.drain() {
                    scene.graph.remove_node(node);
                }
            }
            OverlayType::VisionRanges => {
                for (_, node) in self.vision_nodes.drain() {
                    scene.graph.remove_node(node);
                }
            }
            OverlayType::MovementPaths => {
                for (_, (path_nodes, target_node)) in self.movement_nodes.drain() {
                    for node in path_nodes {
                        scene.graph.remove_node(node);
                    }
                    scene.graph.remove_node(target_node);
                }
            }
            OverlayType::EntityLabels => {
                for (_, node) in self.label_nodes.drain() {
                    scene.graph.remove_node(node);
                }
            }
        }
    }

    /// Clear all overlays
    pub fn clear_all(&mut self, scene: &mut Scene) {
        self.clear_overlay_nodes(OverlayType::CollisionBoxes, scene);
        self.clear_overlay_nodes(OverlayType::VisionRanges, scene);
        self.clear_overlay_nodes(OverlayType::MovementPaths, scene);
        self.clear_overlay_nodes(OverlayType::EntityLabels, scene);
    }

    /// Get status string for all overlays
    pub fn status_string(&self) -> String {
        format!(
            "F1:Collision[{}] F2:Vision[{}] F3:Path[{}] F4:Labels[{}]",
            if self.is_enabled(OverlayType::CollisionBoxes) {
                "ON"
            } else {
                "off"
            },
            if self.is_enabled(OverlayType::VisionRanges) {
                "ON"
            } else {
                "off"
            },
            if self.is_enabled(OverlayType::MovementPaths) {
                "ON"
            } else {
                "off"
            },
            if self.is_enabled(OverlayType::EntityLabels) {
                "ON"
            } else {
                "off"
            },
        )
    }
}

impl Default for DebugOverlays {
    fn default() -> Self {
        Self::new()
    }
}
