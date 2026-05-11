use omoba_sim::{Fixed64, Vec2 as SimVec2};
use serde::{Deserialize, Serialize};
use specs::Entity;

use crate::runtime::comp::{CProperty, Creep, Faction, TAttack, TProperty, Unit};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Outcome {
    Damage {
        pos: SimVec2,
        phys: Fixed64,
        magi: Fixed64,
        real: Fixed64,
        source: Entity,
        target: Entity,
        #[serde(default)]
        predeclared: bool,
    },
    ProjectileLine2 {
        pos: SimVec2,
        source: Option<Entity>,
        target: Option<Entity>,
    },
    Death {
        pos: SimVec2,
        ent: Entity,
    },
    Creep {
        cd: CreepData,
    },
    CreepStop {
        source: Entity,
        target: Entity,
    },
    CreepWalk {
        target: Entity,
    },
    Tower {
        pos: SimVec2,
        td: TowerData,
    },
    Heal {
        pos: SimVec2,
        target: Entity,
        amount: Fixed64,
    },
    UpdateAttack {
        target: Entity,
        asd_count: Option<Fixed64>,
        cooldown_reset: bool,
    },
    GainExperience {
        target: Entity,
        amount: i32,
    },
    GainGold {
        target: Entity,
        amount: i32,
    },
    SpawnUnit {
        pos: SimVec2,
        unit: Unit,
        faction: Faction,
        duration: Option<Fixed64>,
    },
    CreepLeaked {
        ent: Entity,
    },
    AddBuff {
        target: Entity,
        buff_id: String,
        duration: Fixed64,
        #[serde(default)]
        payload: serde_json::Value,
    },
    Explosion {
        pos: SimVec2,
        radius: Fixed64,
        duration: Fixed64,
    },
    ProjectileDirectional {
        pos: SimVec2,
        source: Option<Entity>,
        end_pos: SimVec2,
    },
    AttackPhaseCue {
        entity: Entity,
        attack_seq: u32,
        #[serde(default)]
        is_critical: bool,
        target: Option<Entity>,
        target_pos: Option<SimVec2>,
        windup_ms: u32,
        backswing_ms: u32,
        dir_rad: f32,
    },
    EntityRemoved {
        entity: Entity,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreepData {
    pub pos: SimVec2,
    pub creep: Creep,
    pub cdata: CProperty,
    #[serde(default)]
    pub faction_name: String,
    #[serde(default = "default_creep_turn_speed_deg")]
    pub turn_speed_deg: Fixed64,
    #[serde(default = "default_creep_cr")]
    pub collision_radius: Fixed64,
}

fn default_creep_cr() -> Fixed64 {
    Fixed64::from_i32(20)
}

fn default_creep_turn_speed_deg() -> Fixed64 {
    Fixed64::from_i32(90)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TowerData {
    pub tpty: TProperty,
    pub tatk: TAttack,
}
