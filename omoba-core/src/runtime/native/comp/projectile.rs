use omoba_sim::{Fixed64, Vec2 as SimVec2};
use serde::{Deserialize, Serialize};
use specs::{Component, DenseVecStorage, Entity};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Projectile {
    #[serde(default)]
    pub damage_stage: super::DamageStage,
    pub time_left: Fixed64,
    pub owner: Entity,
    // 如果有target就是指定技 不然就是指向技
    pub target: Option<Entity>,
    pub tpos: SimVec2,
    pub radius: Fixed64,
    pub msd: Fixed64,
    // 傷害值資訊
    pub damage_phys: Fixed64, // 物理傷害
    pub damage_magi: Fixed64, // 魔法傷害
    pub damage_real: Fixed64, // 真實傷害
    #[serde(default)]
    pub damage_profile: u32,
    /// 命中後套用的減速乘數（0.0 表不減速，0.5 表減速到 50%）
    #[serde(default)]
    pub slow_factor: Fixed64,
    /// 減速持續秒數
    #[serde(default)]
    pub slow_duration: Fixed64,
    /// 沿路 hit-test 半徑（無 target 方向性子彈用；0 = 使用預設）
    #[serde(default)]
    pub hit_radius: Fixed64,
    /// 命中後對目標施加的 stun 持續秒數（0 = 不暈眩）。
    /// 由 handle_projectile 在發射時擲骰決定（例：matchlock_gun 的 attack_stun_chance）。
    #[serde(default)]
    pub stun_duration: Fixed64,
    /// Script-visible projectile kind and bounded child generation.
    #[serde(default)]
    pub kind_id: u16,
    #[serde(default)]
    pub generation: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectileData {
    pub id: u32,
    pub time_left: f32,
    pub owner: u32,
    // 如果有target就是指定技 不然就是指向技
    pub target: u32, // 0 就是沒有目標
    pub radius: f32,
    pub msd: f32,
    pub pos: vek::Vec2<f32>,
}

impl Component for Projectile {
    type Storage = DenseVecStorage<Self>;
}

/// Native authority provenance. Never included in public projectile disclosure.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectileOrigin {
    pub source: Entity,
    /// Only the damaging native normal-attack projectile. Never inherited by
    /// visual pellets or script children, even with the NORMAL damage profile.
    #[serde(default)]
    pub normal_attack: bool,
    /// Explicit script ordinary attack accuracy, independent of hero hit-passives.
    #[serde(default)]
    pub script_attack: bool,
    /// Immutable launch miss for a primary ordinary attack; never a zero-damage inference.
    #[serde(default)]
    pub attack_missed: bool,
    /// Primary attack launch critical result; never inherited by visual/child shots.
    #[serde(default)]
    pub attack_critical: bool,
    /// Shared ordinary physical leech committed only on a primary launch.
    #[serde(default)]
    pub attack_lifesteal: Fixed64,
    pub launch_position: Option<omoba_sim::Vec2>,
    pub credit_team: Option<u32>,
    pub credit_player: Option<u32>,
    #[serde(default)]
    pub script: Option<std::sync::Arc<ProjectileScriptSource>>,
}

/// Launch-time fallback for read-only script queries after the exact source life
/// retires. This is authority-only data; it never makes the source alive again.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectileScriptSource {
    pub unit_id: String,
    pub pos: Option<SimVec2>,
    pub facing: Option<super::Facing>,
    pub attack: Option<super::TAttack>,
    pub property: Option<super::CProperty>,
    pub unit: Option<super::Unit>,
    pub faction: Option<super::Faction>,
    pub tower: Option<super::Tower>,
    pub building: bool,
    pub buffs: Vec<(String, Fixed64, serde_json::Value)>,
    #[serde(default)]
    pub ability_levels: Vec<(String,u8)>,
}

impl ProjectileScriptSource {
    pub(crate) fn capture(world: &specs::World, source: Entity) -> Option<std::sync::Arc<Self>> {
        use specs::WorldExt;
        world.try_fetch::<specs::storage::MaskedStorage<crate::runtime::ScriptUnitTag>>()?;
        let unit_id = world.read_storage::<crate::runtime::ScriptUnitTag>().get(source).map(|tag|tag.unit_id.clone()).unwrap_or_default();
        let heroes=if world.try_fetch::<specs::storage::MaskedStorage<super::Hero>>().is_some() {
            Some(world.read_storage::<super::Hero>())
        }else{None};
        let hero=heroes.as_ref().and_then(|heroes|heroes.get(source));
        if unit_id.is_empty()&&hero.is_none() {return None;}
        let mut ability_levels:Vec<_>=hero.filter(|h|h.ability_levels.len()<=64).into_iter()
            .flat_map(|h|h.ability_levels.iter()).filter_map(|(id,rank)|
                u8::try_from(*rank).ok().filter(|rank|*rank>0).map(|rank|(id.clone(),rank))).collect();
        ability_levels.sort();
        // Do not copy the tower's transient spatial lists into every projectile.
        let tower = world.read_storage::<super::Tower>().get(source).map(|tower| {
            let mut snapshot = super::Tower::new();
            snapshot.upgrade_levels = tower.upgrade_levels;
            snapshot.upgrade_flags = tower.upgrade_flags.clone();
            snapshot.target_priority = tower.target_priority;
            snapshot
        });
        let mut buffs: Vec<_> = world.read_resource::<crate::runtime::BuffStore>().iter_for(source)
            .map(|(id, entry)| (id.to_owned(), entry.remaining, entry.payload.clone())).collect();
        buffs.sort_by(|a,b| a.0.cmp(&b.0));
        Some(std::sync::Arc::new(Self { unit_id, tower, buffs, ability_levels,
            pos: world.read_storage::<super::Pos>().get(source).map(|p|p.0),
            facing: world.read_storage::<super::Facing>().get(source).copied(),
            attack: world.read_storage::<super::TAttack>().get(source).copied(),
            property: world.read_storage::<super::CProperty>().get(source).cloned(),
            unit: world.read_storage::<super::Unit>().get(source).cloned(),
            faction: world.read_storage::<super::Faction>().get(source).cloned(),
            building: world.read_storage::<super::IsBuilding>().get(source).is_some(),
        }))
    }
}

#[derive(Default)]
pub struct ProjectileOrigins(pub std::collections::BTreeMap<Entity,ProjectileOrigin>);

/// Exists only while one captured projectile damage outcome is settled.
pub(crate) struct ProjectileImpactOrigin(pub ProjectileOrigin);

#[cfg(test)]
mod miss_origin_tests {
    use super::*;
    use specs::{Builder,WorldExt};
    #[test]
    fn launch_miss_roundtrips_and_legacy_origins_default_to_no_miss() {
        let mut world=specs::World::new();let source=world.create_entity().build();
        let origin=ProjectileOrigin {source,normal_attack:false,script_attack:false,attack_missed:true,attack_critical:true,attack_lifesteal:Fixed64::from_raw(512),
            launch_position:Some(SimVec2::ZERO),credit_team:Some(1),credit_player:Some(7),script:None};
        let mut value=serde_json::to_value(&origin).unwrap();
        let copy:ProjectileOrigin=serde_json::from_value(value.clone()).unwrap();
        assert!(copy.attack_missed);assert!(copy.attack_critical);assert_eq!(copy.attack_lifesteal,Fixed64::from_raw(512));assert_eq!(copy.source,source);assert_eq!(copy.credit_player,Some(7));
        value.as_object_mut().unwrap().remove("attack_missed");
        value.as_object_mut().unwrap().remove("attack_lifesteal");value.as_object_mut().unwrap().remove("attack_critical");
        let legacy:ProjectileOrigin=serde_json::from_value(value).unwrap();
        assert!(!legacy.attack_missed);assert!(!legacy.attack_critical);assert_eq!(legacy.attack_lifesteal,Fixed64::ZERO);assert_eq!(legacy.source,source);assert_eq!(legacy.launch_position,Some(SimVec2::ZERO));
    }
}
