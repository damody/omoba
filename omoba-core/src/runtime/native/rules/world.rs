//! 對局層的規則狀態（v3.1 §12.10）。
//!
//! - **暫時夜晚**（`temporary_night { duration }`）：期間 `game.is_night` 為 true。本遊戲目前沒有
//!   日夜循環，所以夜晚只來自這個 action。截止時間是比賽時間（`MobaMatch.elapsed`），重疊時取較晚者。
//!   這是雙方都會讀的對局事實，揭露政策是 `public`（m2-disclosure-policy.md）；目前還沒有消費者
//!   （夜晚視野在 M3 之後接上）。
//! - **尋路飛行**（狀態旗標 `flying_for_pathing`）：只改尋路，不給飛行視野。
use omoba_sim::Fixed64;
use specs::{Entity, World, WorldExt};

use crate::runtime::BuffStore;

/// 暫時夜晚的截止時間（比賽時間）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TemporaryNight {
    pub until: Option<Fixed64>,
}

fn now(world: &World) -> Fixed64 {
    world.try_fetch::<crate::runtime::MobaMatch>().map_or(Fixed64::ZERO, |state| state.elapsed)
}

/// 開始（或延長）全圖暫時夜晚。
pub fn start_temporary_night(world: &mut World, duration: Fixed64) {
    if duration <= Fixed64::ZERO { return; }
    let until = now(world) + duration;
    if world.try_fetch::<TemporaryNight>().is_none() { world.insert(TemporaryNight::default()); }
    let mut night = world.write_resource::<TemporaryNight>();
    night.until = Some(night.until.map_or(until, |old| old.max(until)));
}

/// `game.is_night`。
pub fn is_night(world: &World) -> bool {
    let time = now(world);
    world.try_fetch::<TemporaryNight>().and_then(|night| night.until).is_some_and(|until| time < until)
}

/// 單位尋路時是否當成飛行（越過地形）。
pub fn flying_for_pathing(buffs: &BuffStore, entity: Entity) -> bool {
    buffs.has(entity, super::vocab::state::FLYING_FOR_PATHING)
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    #[test]
    fn temporary_night_lasts_its_duration_and_overlaps_take_the_later_end() {
        let mut world = World::new();
        assert!(!is_night(&world));
        start_temporary_night(&mut world, Fixed64::from_i32(5));
        assert!(is_night(&world));
        start_temporary_night(&mut world, Fixed64::from_i32(2));
        assert_eq!(world.read_resource::<TemporaryNight>().until, Some(Fixed64::from_i32(5)));
        world.write_resource::<TemporaryNight>().until = Some(Fixed64::ZERO);
        assert!(!is_night(&world), "over at its end");
        let unit = world.create_entity().build();
        let mut buffs = BuffStore::new();
        assert!(!flying_for_pathing(&buffs, unit));
        buffs.add(unit, super::super::vocab::state::FLYING_FOR_PATHING, Fixed64::from_i32(3), serde_json::Value::Null);
        assert!(flying_for_pathing(&buffs, unit));
    }
}
