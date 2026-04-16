//! Entity rendering system

use fyrox::{
    asset::untyped::ResourceKind,
    core::{
        pool::Handle,
        algebra::Vector3,
        color::Color,
    },
    graph::BaseSceneGraph,
    material::{Material, MaterialResource},
    scene::{
        Scene,
        node::Node,
        base::BaseBuilder,
        dim2::rectangle::RectangleBuilder,
        transform::TransformBuilder,
    },
};
use log::{debug, info};
use std::collections::HashMap;

use omoba_core::{Entity, EntityType};

/// Colors for different entity types
pub struct EntityColors;

impl EntityColors {
    pub const PLAYER_ALLY: Color = Color::from_rgba(100, 149, 237, 255);    // Cornflower blue
    pub const PLAYER_ENEMY: Color = Color::from_rgba(220, 20, 60, 255);     // Crimson
    pub const SUMMON_ALLY: Color = Color::from_rgba(135, 206, 250, 255);    // Light sky blue
    pub const SUMMON_ENEMY: Color = Color::from_rgba(255, 182, 193, 255);   // Light pink
    pub const CREEP_ALLY: Color = Color::from_rgba(144, 238, 144, 255);     // Light green
    pub const CREEP_ENEMY: Color = Color::from_rgba(255, 165, 0, 255);      // Orange
    pub const TOWER: Color = Color::from_rgba(128, 128, 128, 255);          // Gray
    pub const PROJECTILE: Color = Color::from_rgba(255, 255, 0, 255);       // Yellow
    pub const EFFECT: Color = Color::from_rgba(255, 255, 255, 128);         // Semi-transparent white
}

/// Entity sizes (used as scale factors for rectangles)
pub struct EntitySizes;

impl EntitySizes {
    pub const PLAYER: f32 = 32.0;
    pub const SUMMON: f32 = 20.0;
    pub const CREEP: f32 = 16.0;
    pub const TOWER: f32 = 48.0;
    pub const PROJECTILE: f32 = 8.0;
    pub const EFFECT: f32 = 24.0;
}

/// Entity renderer
pub struct EntityRenderer {
    /// Map from entity ID to scene node handle
    entity_nodes: HashMap<u32, Handle<Node>>,
    /// Local player name for determining ally/enemy
    local_player_name: String,
    /// 2D material resource for rendering rectangles
    material: MaterialResource,
}

impl EntityRenderer {
    pub fn new(local_player_name: String) -> Self {
        // Create a standard 2D material for rendering colored rectangles
        let material = Material::standard_2d();
        let material_resource = MaterialResource::new_ok(ResourceKind::Embedded, material);

        Self {
            entity_nodes: HashMap::new(),
            local_player_name,
            material: material_resource,
        }
    }

    /// Update or create entity visual
    pub fn update_entity(&mut self, entity: &Entity, scene: &mut Scene) {
        if let Some(&node_handle) = self.entity_nodes.get(&entity.id) {
            // Update existing node position
            if let Some(node) = scene.graph.try_get_mut(node_handle) {
                let transform = node.local_transform_mut();
                transform.set_position(Vector3::new(
                    entity.position.x,
                    entity.position.y,
                    0.0,
                ));
            }
        } else {
            // Create new node
            let (color, size) = self.get_entity_visual_properties(entity);

            info!(
                "[renderer] Creating visual for entity {} ({:?}) at ({}, {}) with size {}",
                entity.id, entity.entity_type, entity.position.x, entity.position.y, size
            );

            let node = RectangleBuilder::new(
                BaseBuilder::new()
                    .with_local_transform(
                        TransformBuilder::new()
                            .with_local_position(Vector3::new(
                                entity.position.x,
                                entity.position.y,
                                0.0,
                            ))
                            .with_local_scale(Vector3::new(size, size, f32::EPSILON))
                            .build()
                    )
            )
            .with_color(color)
            .with_material(self.material.clone())
            .build(&mut scene.graph);

            self.entity_nodes.insert(entity.id, node);
            debug!("Entity {} visual created, node handle: {:?}", entity.id, node);
        }
    }

    /// Remove entity visual
    pub fn remove_entity(&mut self, entity_id: u32, scene: &mut Scene) {
        if let Some(node_handle) = self.entity_nodes.remove(&entity_id) {
            scene.graph.remove_node(node_handle);
        }
    }

    /// Get visual properties based on entity type
    fn get_entity_visual_properties(&self, entity: &Entity) -> (Color, f32) {
        match &entity.entity_type {
            EntityType::Player(name) => {
                let is_ally = name == &self.local_player_name;
                let color = if is_ally { EntityColors::PLAYER_ALLY } else { EntityColors::PLAYER_ENEMY };
                (color, EntitySizes::PLAYER)
            }
            EntityType::Summon(_) => {
                let is_ally = entity.owner.as_ref().map_or(false, |o| o == &self.local_player_name);
                let color = if is_ally { EntityColors::SUMMON_ALLY } else { EntityColors::SUMMON_ENEMY };
                (color, EntitySizes::SUMMON)
            }
            EntityType::Creep(_) => {
                // For now, assume creeps are enemies unless we have ownership info
                (EntityColors::CREEP_ENEMY, EntitySizes::CREEP)
            }
            EntityType::Tower => {
                (EntityColors::TOWER, EntitySizes::TOWER)
            }
            EntityType::Projectile => {
                (EntityColors::PROJECTILE, EntitySizes::PROJECTILE)
            }
            EntityType::Effect => {
                (EntityColors::EFFECT, EntitySizes::EFFECT)
            }
        }
    }

    /// Clear all entity visuals
    pub fn clear(&mut self, scene: &mut Scene) {
        for (_, node_handle) in self.entity_nodes.drain() {
            scene.graph.remove_node(node_handle);
        }
    }

    /// Sync with game state
    pub fn sync_with_game_state(&mut self, entities: &HashMap<u32, Entity>, scene: &mut Scene) {
        // Update existing entities
        let before_count = self.entity_nodes.len();
        for entity in entities.values() {
            self.update_entity(entity, scene);
        }
        let after_count = self.entity_nodes.len();
        if after_count != before_count {
            info!("[renderer] Entity nodes: {} -> {} (game_state entities: {})", before_count, after_count, entities.len());
        }

        // Remove entities that no longer exist
        let current_ids: std::collections::HashSet<u32> = entities.keys().copied().collect();
        let stale_ids: Vec<u32> = self.entity_nodes.keys()
            .filter(|id| !current_ids.contains(id))
            .copied()
            .collect();

        for id in stale_ids {
            self.remove_entity(id, scene);
        }
    }
}
