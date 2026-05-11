use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::Component;
use vek::Vec2;

#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct CircularVision {
    pub range: f32,
    pub height: f32,
    pub precision: u32,
    pub true_sight: bool,
    pub vision_result: Option<VisionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionResult {
    pub observer_pos: Vec2<f32>,
    pub range: f32,
    pub visible_area: Vec<Vec2<f32>>,
    pub shadows: Vec<ShadowArea>,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowArea {
    pub shadow_type: ShadowType,
    pub blocker_id: Option<String>,
    pub geometry: ShadowGeometry,
    pub depth: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShadowType {
    Object,
    Building,
    Sector,
    Trapezoid,
    Terrain,
    Temporary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShadowGeometry {
    Sector {
        center: Vec2<f32>,
        start_angle: f32,
        end_angle: f32,
        radius: f32,
    },
    Trapezoid {
        vertices: [Vec2<f32>; 4],
    },
    Polygon {
        vertices: Vec<Vec2<f32>>,
    },
}

#[derive(Debug, Clone)]
pub struct ObstacleInfo {
    pub position: Vec2<f32>,
    pub obstacle_type: ObstacleType,
    pub height: f32,
    pub properties: ObstacleProperties,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObstacleType {
    Circular { radius: f32 },
    Rectangle {
        width: f32,
        height: f32,
        rotation: f32,
    },
    Terrain { elevation: f32 },
}

#[derive(Debug, Clone)]
pub struct ObstacleProperties {
    pub blocks_completely: bool,
    pub opacity: f32,
    pub shadow_multiplier: f32,
}

impl CircularVision {
    pub fn new(range: f32, height: f32) -> Self {
        Self {
            range,
            height,
            precision: 360,
            true_sight: false,
            vision_result: None,
        }
    }

    pub fn with_precision(mut self, precision: u32) -> Self {
        self.precision = precision;
        self
    }

    pub fn with_true_sight(mut self) -> Self {
        self.true_sight = true;
        self
    }
}
