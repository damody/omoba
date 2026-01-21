//! Skill effect rendering system

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
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Effect colors
pub struct EffectColors;

impl EffectColors {
    pub const SKILL_RANGE: Color = Color::from_rgba(100, 200, 255, 80);     // Light blue, very transparent
    pub const AOE_FRIENDLY: Color = Color::from_rgba(100, 255, 100, 100);   // Green, semi-transparent
    pub const AOE_ENEMY: Color = Color::from_rgba(255, 100, 100, 100);      // Red, semi-transparent
    pub const AOE_NEUTRAL: Color = Color::from_rgba(255, 255, 100, 100);    // Yellow, semi-transparent
    pub const PROJECTILE_TRAIL: Color = Color::from_rgba(255, 200, 50, 150); // Orange-yellow
}

/// Skill range indicator (circle shown during casting)
#[derive(Debug, Clone)]
pub struct SkillRangeIndicator {
    pub id: u32,
    pub center: Vector2<f32>,
    pub radius: f32,
    pub color: Color,
}

/// AOE zone effect
#[derive(Debug, Clone)]
pub struct AoeZone {
    pub id: u32,
    pub center: Vector2<f32>,
    pub radius: f32,
    pub color: Color,
    pub created_at: Instant,
    pub duration: Duration,
}

/// Projectile trail point
#[derive(Debug, Clone)]
pub struct TrailPoint {
    pub position: Vector2<f32>,
    pub timestamp: Instant,
}

/// Projectile trail
#[derive(Debug, Clone)]
pub struct ProjectileTrail {
    pub projectile_id: u32,
    pub points: VecDeque<TrailPoint>,
    pub color: Color,
    pub max_duration: Duration,
}

impl ProjectileTrail {
    pub fn new(projectile_id: u32) -> Self {
        Self {
            projectile_id,
            points: VecDeque::new(),
            color: EffectColors::PROJECTILE_TRAIL,
            max_duration: Duration::from_millis(500),
        }
    }

    /// Add a new point to the trail
    pub fn add_point(&mut self, position: Vector2<f32>) {
        self.points.push_back(TrailPoint {
            position,
            timestamp: Instant::now(),
        });
        self.cleanup_old_points();
    }

    /// Remove points older than max_duration
    fn cleanup_old_points(&mut self) {
        let now = Instant::now();
        while let Some(front) = self.points.front() {
            if now.duration_since(front.timestamp) > self.max_duration {
                self.points.pop_front();
            } else {
                break;
            }
        }
    }
}

/// Effect renderer
pub struct EffectRenderer {
    /// Skill range indicator nodes
    skill_range_nodes: HashMap<u32, Handle<Node>>,
    /// AOE zone nodes
    aoe_nodes: HashMap<u32, Handle<Node>>,
    /// Trail segment nodes (projectile_id -> list of segment nodes)
    trail_nodes: HashMap<u32, Vec<Handle<Node>>>,
    /// Active skill ranges
    skill_ranges: HashMap<u32, SkillRangeIndicator>,
    /// Active AOE zones
    aoe_zones: HashMap<u32, AoeZone>,
    /// Active projectile trails
    projectile_trails: HashMap<u32, ProjectileTrail>,
    /// Next effect ID
    next_id: u32,
}

impl EffectRenderer {
    pub fn new() -> Self {
        Self {
            skill_range_nodes: HashMap::new(),
            aoe_nodes: HashMap::new(),
            trail_nodes: HashMap::new(),
            skill_ranges: HashMap::new(),
            aoe_zones: HashMap::new(),
            projectile_trails: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a skill range indicator
    pub fn add_skill_range(&mut self, center: Vector2<f32>, radius: f32, color: Option<Color>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        self.skill_ranges.insert(id, SkillRangeIndicator {
            id,
            center,
            radius,
            color: color.unwrap_or(EffectColors::SKILL_RANGE),
        });

        id
    }

    /// Remove a skill range indicator
    pub fn remove_skill_range(&mut self, id: u32, scene: &mut Scene) {
        self.skill_ranges.remove(&id);
        if let Some(node) = self.skill_range_nodes.remove(&id) {
            scene.graph.remove_node(node);
        }
    }

    /// Add an AOE zone
    pub fn add_aoe_zone(&mut self, center: Vector2<f32>, radius: f32, duration: Duration, is_friendly: bool) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let color = if is_friendly {
            EffectColors::AOE_FRIENDLY
        } else {
            EffectColors::AOE_ENEMY
        };

        self.aoe_zones.insert(id, AoeZone {
            id,
            center,
            radius,
            color,
            created_at: Instant::now(),
            duration,
        });

        id
    }

    /// Update projectile position (adds to trail)
    pub fn update_projectile_trail(&mut self, projectile_id: u32, position: Vector2<f32>) {
        let trail = self.projectile_trails
            .entry(projectile_id)
            .or_insert_with(|| ProjectileTrail::new(projectile_id));
        trail.add_point(position);
    }

    /// Remove projectile trail
    pub fn remove_projectile_trail(&mut self, projectile_id: u32, scene: &mut Scene) {
        self.projectile_trails.remove(&projectile_id);
        if let Some(nodes) = self.trail_nodes.remove(&projectile_id) {
            for node in nodes {
                scene.graph.remove_node(node);
            }
        }
    }

    /// Update all effects (remove expired, render active)
    pub fn update(&mut self, scene: &mut Scene) {
        self.update_aoe_zones(scene);
        self.update_trails(scene);
        self.render_skill_ranges(scene);
        self.render_aoe_zones(scene);
        self.render_trails(scene);
    }

    /// Remove expired AOE zones
    fn update_aoe_zones(&mut self, scene: &mut Scene) {
        let now = Instant::now();
        let expired: Vec<u32> = self.aoe_zones
            .iter()
            .filter(|(_, zone)| now.duration_since(zone.created_at) > zone.duration)
            .map(|(id, _)| *id)
            .collect();

        for id in expired {
            self.aoe_zones.remove(&id);
            if let Some(node) = self.aoe_nodes.remove(&id) {
                scene.graph.remove_node(node);
            }
        }
    }

    /// Update trail cleanup
    fn update_trails(&mut self, scene: &mut Scene) {
        // Clean up old points in each trail
        for trail in self.projectile_trails.values_mut() {
            trail.cleanup_old_points();
        }

        // Remove empty trails
        let empty_trails: Vec<u32> = self.projectile_trails
            .iter()
            .filter(|(_, trail)| trail.points.is_empty())
            .map(|(id, _)| *id)
            .collect();

        for id in empty_trails {
            self.remove_projectile_trail(id, scene);
        }
    }

    /// Render skill range indicators as rectangles (approximating circles)
    fn render_skill_ranges(&mut self, scene: &mut Scene) {
        for (id, range) in &self.skill_ranges {
            if !self.skill_range_nodes.contains_key(id) {
                // Create new node - using rectangle as placeholder for circle
                // In a full implementation, we'd use a circle mesh or sprite
                let node = RectangleBuilder::new(
                    BaseBuilder::new()
                        .with_local_transform(
                            TransformBuilder::new()
                                .with_local_position(Vector3::new(range.center.x, range.center.y, 0.1))
                                .with_local_scale(Vector3::new(range.radius * 2.0, range.radius * 2.0, 1.0))
                                .build()
                        )
                )
                .with_color(range.color)
                .build(&mut scene.graph);

                self.skill_range_nodes.insert(*id, node);
            } else if let Some(&node_handle) = self.skill_range_nodes.get(id) {
                // Update existing node position
                if let Some(node) = scene.graph.try_get_mut(node_handle) {
                    let transform = node.local_transform_mut();
                    transform.set_position(Vector3::new(range.center.x, range.center.y, 0.1));
                }
            }
        }
    }

    /// Render AOE zones
    fn render_aoe_zones(&mut self, scene: &mut Scene) {
        for (id, zone) in &self.aoe_zones {
            if !self.aoe_nodes.contains_key(id) {
                let node = RectangleBuilder::new(
                    BaseBuilder::new()
                        .with_local_transform(
                            TransformBuilder::new()
                                .with_local_position(Vector3::new(zone.center.x, zone.center.y, 0.05))
                                .with_local_scale(Vector3::new(zone.radius * 2.0, zone.radius * 2.0, 1.0))
                                .build()
                        )
                )
                .with_color(zone.color)
                .build(&mut scene.graph);

                self.aoe_nodes.insert(*id, node);
            }
        }
    }

    /// Render projectile trails as line segments
    fn render_trails(&mut self, scene: &mut Scene) {
        for (projectile_id, trail) in &self.projectile_trails {
            // Clear existing trail nodes for this projectile
            if let Some(old_nodes) = self.trail_nodes.remove(projectile_id) {
                for node in old_nodes {
                    scene.graph.remove_node(node);
                }
            }

            // Create new trail segments
            let mut nodes = Vec::new();
            let points: Vec<_> = trail.points.iter().collect();

            for i in 1..points.len() {
                let p1 = &points[i - 1];
                let p2 = &points[i];

                // Calculate segment properties
                let dx = p2.position.x - p1.position.x;
                let dy = p2.position.y - p1.position.y;
                let length = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);
                let mid_x = (p1.position.x + p2.position.x) / 2.0;
                let mid_y = (p1.position.y + p2.position.y) / 2.0;

                // Calculate alpha based on age (older = more transparent)
                let now = Instant::now();
                let age = now.duration_since(p1.timestamp).as_secs_f32();
                let max_age = trail.max_duration.as_secs_f32();
                let alpha = ((1.0 - age / max_age) * 150.0) as u8;

                let color = Color::from_rgba(
                    trail.color.r,
                    trail.color.g,
                    trail.color.b,
                    alpha,
                );

                // Create line segment as a thin rectangle
                let node = RectangleBuilder::new(
                    BaseBuilder::new()
                        .with_local_transform(
                            TransformBuilder::new()
                                .with_local_position(Vector3::new(mid_x, mid_y, 0.15))
                                .with_local_scale(Vector3::new(length, 2.0, 1.0))
                                .with_local_rotation(
                                    fyrox::core::algebra::UnitQuaternion::from_axis_angle(
                                        &fyrox::core::algebra::Vector3::z_axis(),
                                        angle,
                                    )
                                )
                                .build()
                        )
                )
                .with_color(color)
                .build(&mut scene.graph);

                nodes.push(node);
            }

            if !nodes.is_empty() {
                self.trail_nodes.insert(*projectile_id, nodes);
            }
        }
    }

    /// Clear all effects
    pub fn clear(&mut self, scene: &mut Scene) {
        for (_, node) in self.skill_range_nodes.drain() {
            scene.graph.remove_node(node);
        }
        for (_, node) in self.aoe_nodes.drain() {
            scene.graph.remove_node(node);
        }
        for (_, nodes) in self.trail_nodes.drain() {
            for node in nodes {
                scene.graph.remove_node(node);
            }
        }
        self.skill_ranges.clear();
        self.aoe_zones.clear();
        self.projectile_trails.clear();
    }
}

impl Default for EffectRenderer {
    fn default() -> Self {
        Self::new()
    }
}
