#[derive(Clone, Debug)]
pub struct ExplosionFx {
    pub pos_x: f32,
    pub pos_y: f32,
    pub radius: f32,
    pub duration_ms: u32,
    pub spawn_tick: u32,
}

#[derive(Default)]
pub struct ExplosionFxQueue {
    pub pending: Vec<ExplosionFx>,
}

#[derive(Clone, Debug)]
pub struct TowerFireFx {
    pub entity_id: u32,
    pub entity_gen: u32,
    pub spawn_tick: u32,
    pub dir_rad: f32,
}

#[derive(Default)]
pub struct TowerFireFxQueue {
    pub pending: Vec<TowerFireFx>,
}

#[derive(Clone, Debug)]
pub struct AttackPhaseFx {
    pub entity_id: u32,
    pub entity_gen: u32,
    pub spawn_tick: u32,
    pub attack_seq: u32,
    pub is_critical: bool,
    pub windup_ms: u32,
    pub impact_at_ms: u32,
    pub backswing_ms: u32,
    pub dir_rad: f32,
    pub target_entity_id: Option<u32>,
    pub target_pos_x: Option<f32>,
    pub target_pos_y: Option<f32>,
}

#[derive(Default)]
pub struct AttackPhaseFxQueue {
    pub pending: Vec<AttackPhaseFx>,
    pub next_seq: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackCancelPhase {
    Windup,
    Backswing,
}

#[derive(Clone, Debug)]
pub struct AttackCancelFx {
    pub entity_id: u32,
    pub entity_gen: u32,
    pub spawn_tick: u32,
    pub attack_seq: u32,
    pub phase: AttackCancelPhase,
    pub impact_committed: bool,
}

#[derive(Default)]
pub struct AttackCancelFxQueue {
    pub pending: Vec<AttackCancelFx>,
}

#[derive(Default)]
pub struct RemovedEntitiesQueue {
    pub pending: Vec<u32>,
}
