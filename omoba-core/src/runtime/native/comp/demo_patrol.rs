use omoba_sim::Vec2;
use specs::{Component, VecStorage};

#[derive(Clone, Debug)]
pub struct DemoPatrol {
    pub stable_index: u32,
    pub endpoint_a: Vec2,
    pub endpoint_b: Vec2,
    pub target_b: bool,
    pub speed_per_tick: omoba_sim::Fixed64,
}

impl Component for DemoPatrol {
    type Storage = VecStorage<Self>;
}
