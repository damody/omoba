use omoba_sim::{Fixed64, Vec2 as SimVec2};
use serde::{Deserialize, Serialize};
use vek::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckPoint {
    pub name: String,
    pub class: String,
    pub pos: Vec2<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Path {
    pub check_points: Vec<CheckPoint>,
    #[serde(skip)]
    pub check_points_sim: Vec<SimVec2>,
}

impl Path {
    pub fn new(check_points: Vec<CheckPoint>) -> Self {
        let check_points_sim = check_points
            .iter()
            .map(|cp| SimVec2::new(fixed_from_f32(cp.pos.x), fixed_from_f32(cp.pos.y)))
            .collect();
        Self {
            check_points,
            check_points_sim,
        }
    }
}

#[inline]
fn fixed_from_f32(v: f32) -> Fixed64 {
    Fixed64::from_raw((v * omoba_sim::fixed::SCALE as f32) as i64)
}
