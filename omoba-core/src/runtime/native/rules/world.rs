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

/// 夜晚剩餘時間的公開 HUD 指標（每個 tick 都送，白天為 0），呈現層用它調暗地圖光源。
pub const NIGHT_METRIC_ID: u64 = 0x4d4e_4947_4854_3031;

/// 比賽時間 `now` 時夜晚還剩多久；白天為 0。
pub fn night_remaining(world: &World, now: Fixed64) -> Fixed64 {
    world.try_fetch::<TemporaryNight>().and_then(|night| night.until).map_or(Fixed64::ZERO, |until| (until - now).max(Fixed64::ZERO))
}

/// `game.is_night`。
pub fn is_night(world: &World) -> bool {
    let time = now(world);
    world.try_fetch::<TemporaryNight>().and_then(|night| night.until).is_some_and(|until| time < until)
}

/// v3 `set_time_of_day { mode, duration }`（E453）：一次性的 host 封包（施法者身上，不儲存）。
/// `night`：全圖夜晚 `duration` 秒（與 [`start_temporary_night`] 相同，重疊取較晚者）；
/// `day`：立刻結束暫時夜晚。
pub const TIME_OF_DAY_BUFF: &str = "__time_of_day";
pub const TIME_OF_DAY_KEY: &str = "__time_of_day_v1";

pub fn time_of_day_payload(night: bool, duration: Fixed64) -> serde_json::Value {
    serde_json::json!({ (TIME_OF_DAY_KEY): { "night": night, "duration_raw": duration.raw() } })
}

/// 結算一個 [`TIME_OF_DAY_BUFF`] 封包；不是這個封包時回傳 false（格式錯誤的封包也算處理過，不做事）。
pub fn settle_time_of_day(world: &mut World, buff_id: &str, payload: &serde_json::Value) -> bool {
    if buff_id != TIME_OF_DAY_BUFF && payload.get(TIME_OF_DAY_KEY).is_none() { return false; }
    let row = payload.as_object().filter(|row| row.len() == 1).and_then(|row| row.get(TIME_OF_DAY_KEY))
        .and_then(serde_json::Value::as_object).filter(|row| row.len() == 2 && buff_id == TIME_OF_DAY_BUFF);
    let Some(row) = row else { return true };
    match (row.get("night").and_then(serde_json::Value::as_bool), row.get("duration_raw").and_then(serde_json::Value::as_i64)) {
        (Some(true), Some(raw)) if raw > 0 && raw <= Fixed64::from_i32(3600).raw() => start_temporary_night(world, Fixed64::from_raw(raw)),
        (Some(false), Some(_)) => if let Some(mut night) = world.try_fetch_mut::<TemporaryNight>() { night.until = None; },
        _ => {}
    }
    true
}

/// v3 stat `when = game.is_night` / `not game.is_night`（E508）：kit 把這種狀態列標上 [`WHEN_KEY`]，
/// 無條件的數值放在 `base`，只在夜晚／白天成立的放在 `night`／`day`；列的頂層數值是 base 加上
/// 現在成立的那一半（`at`）。kit 套上時就算好；夜晚開始或結束時由 [`sync_when_rows`] 改寫。
pub const WHEN_KEY: &str = "__when_v1";

/// 依 `night` 算出 `payload` 頂層的數值並記下 `at`；沒有 [`WHEN_KEY`] 或格式不對時不動，回傳 false。
pub fn resolve_when(payload: &mut serde_json::Map<String, serde_json::Value>, night: bool) -> bool {
    let Some(row) = payload.get(WHEN_KEY).and_then(serde_json::Value::as_object) else { return false };
    let half = |name: &str| row.get(name).and_then(serde_json::Value::as_object).cloned().unwrap_or_default();
    let (base, active, other) = if night { (half("base"), half("night"), half("day")) } else { (half("base"), half("day"), half("night")) };
    let mut totals: Vec<(String, i64)> = Vec::new();
    for key in base.keys().chain(active.keys()).chain(other.keys()) {
        if totals.iter().any(|(k, _)| k == key) { continue; }
        let raw = |half: &serde_json::Map<String, serde_json::Value>| half.get(key).and_then(serde_json::Value::as_i64).unwrap_or(0);
        totals.push((key.clone(), raw(&base).saturating_add(raw(&active))));
    }
    for (key, raw) in totals { payload.insert(key, raw.into()); }
    if let Some(row) = payload.get_mut(WHEN_KEY).and_then(serde_json::Value::as_object_mut) {
        row.insert("at".into(), (if night { "night" } else { "day" }).into());
    }
    true
}

/// 夜晚開始或結束後，把 `at` 與現在不同的 [`WHEN_KEY`] 列改寫成現在的數值（比賽時間 `now`；
/// moba_match 每個進行中的 tick 呼叫，此時 MobaMatch 不在 world 裡，所以由呼叫端給時間）。
pub fn sync_when_rows(world: &World, now: Fixed64) {
    let night = world.try_fetch::<TemporaryNight>().and_then(|night| night.until).is_some_and(|until| now < until);
    let Some(mut buffs) = world.try_fetch_mut::<BuffStore>() else { return };
    let at = if night { "night" } else { "day" };
    let mut holders: Vec<Entity> = buffs.entities_with_key(WHEN_KEY).collect();
    holders.sort_by_key(|entity| (entity.id(), entity.gen().id()));
    for entity in holders {
        let stale: Vec<(String, serde_json::Value)> = buffs.iter_for(entity)
            .filter(|(_, entry)| entry.payload.get(WHEN_KEY).and_then(|row| row.get("at")).and_then(serde_json::Value::as_str) != Some(at))
            .filter(|(_, entry)| entry.payload.get(WHEN_KEY).is_some())
            .map(|(id, entry)| (id.to_owned(), entry.payload.clone())).collect();
        for (id, payload) in stale {
            let serde_json::Value::Object(mut payload) = payload else { continue };
            if resolve_when(&mut payload, night) { buffs.rewrite_payload(entity, &id, serde_json::Value::Object(payload)); }
        }
    }
}

/// Replica 每一步開始：採用這一步公開的夜晚剩餘時間（[`NIGHT_METRIC_ID`]，每個 tick 都送，白天 0），
/// 讓 `game.is_night` 在每一端都一樣（E453）。沒有這個指標時不動。
pub fn apply_disclosed(world: &mut World, payloads: &[&[u8]]) {
    let Some(remaining) = payloads.iter().filter(|bytes| bytes.len() == 20 && bytes[0..4] == 0_u32.to_le_bytes()
            && bytes[4..12] == NIGHT_METRIC_ID.to_le_bytes())
        .map(|bytes| Fixed64::from_raw(i64::from_le_bytes(bytes[12..20].try_into().unwrap()))).last() else { return };
    let until = (remaining > Fixed64::ZERO).then(|| now(world) + remaining);
    if world.try_fetch::<TemporaryNight>().is_none() { world.insert(TemporaryNight::default()); }
    world.write_resource::<TemporaryNight>().until = until;
}

/// 單位尋路與移動時是否當成飛行（越過地形與樹）：`flying_for_pathing` 或 `flying`（E397）。
pub fn flying_for_pathing(buffs: &BuffStore, entity: Entity) -> bool {
    buffs.has(entity, super::vocab::state::FLYING_FOR_PATHING) || buffs.has(entity, super::vocab::state::FLYING)
}

/// 飛行單位的視野越過地形與樹（只有 `flying`，`flying_for_pathing` 不給）。
pub fn flying_sight(buffs: &BuffStore, entity: Entity) -> bool {
    buffs.has(entity, super::vocab::state::FLYING)
}

/// What blocks `entity`'s movement: the map's terrain, or nothing while it flies (E397).
pub fn movement_regions<'a>(buffs: &BuffStore, entity: Entity, regions: &'a crate::comp::BlockedRegions)
    -> &'a crate::comp::BlockedRegions {
    static OPEN: crate::comp::BlockedRegions = crate::comp::BlockedRegions(Vec::new());
    if flying_for_pathing(buffs, entity) { &OPEN } else { regions }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    /// E397: `flying` lifts terrain from movement and sight; `flying_for_pathing` only from movement.
    #[test]
    fn a_flyer_moves_and_sees_over_terrain() {
        let mut world = World::new();
        let unit = world.create_entity().build();
        let mut buffs = BuffStore::new();
        let walls = crate::comp::BlockedRegions(vec![crate::comp::BlockedRegion { name: "wall".into(), points: Vec::new() }]);
        assert_eq!(movement_regions(&buffs, unit, &walls).0.len(), 1);
        buffs.add(unit, super::super::vocab::state::FLYING_FOR_PATHING, Fixed64::from_i32(3), serde_json::Value::Null);
        assert!(movement_regions(&buffs, unit, &walls).0.is_empty() && !flying_sight(&buffs, unit));
        buffs.remove(unit, super::super::vocab::state::FLYING_FOR_PATHING);
        buffs.add(unit, super::super::vocab::state::FLYING, Fixed64::from_i32(3), serde_json::Value::Null);
        assert!(movement_regions(&buffs, unit, &walls).0.is_empty() && flying_sight(&buffs, unit));
    }

    /// E453: the set_time_of_day packet starts or ends the night; a replica adopts the disclosed night.
    #[test]
    fn a_time_of_day_packet_sets_the_night_and_replicas_adopt_the_disclosed_one() {
        let mut world = World::new();
        assert!(!settle_time_of_day(&mut world, "other", &serde_json::json!({})));
        assert!(settle_time_of_day(&mut world, TIME_OF_DAY_BUFF, &time_of_day_payload(true, Fixed64::from_i32(30))));
        assert_eq!(night_remaining(&world, Fixed64::ZERO), Fixed64::from_i32(30));
        assert!(settle_time_of_day(&mut world, TIME_OF_DAY_BUFF, &serde_json::json!({ (TIME_OF_DAY_KEY): { "night": true } })));
        assert_eq!(night_remaining(&world, Fixed64::ZERO), Fixed64::from_i32(30), "a malformed packet does nothing");
        assert!(settle_time_of_day(&mut world, TIME_OF_DAY_BUFF, &time_of_day_payload(false, Fixed64::ZERO)));
        assert!(!is_night(&world), "day ends the night");
        let hud = |remaining: i64| [0_u32.to_le_bytes().as_slice(), NIGHT_METRIC_ID.to_le_bytes().as_slice(), remaining.to_le_bytes().as_slice()].concat();
        let mut replica = World::new();
        apply_disclosed(&mut replica, &[hud(Fixed64::from_i32(12).raw()).as_slice()]);
        assert!(is_night(&replica) && night_remaining(&replica, Fixed64::ZERO) == Fixed64::from_i32(12));
        apply_disclosed(&mut replica, &[]);
        assert!(is_night(&replica), "no fact, no change");
        apply_disclosed(&mut replica, &[hud(0).as_slice()]);
        assert!(!is_night(&replica));
    }

    /// E508：when 列在夜晚開始與結束時改寫頂層數值，不成立那一半的 key 留在頂層為 0。
    #[test]
    fn when_rows_follow_the_night() {
        use omb_script_abi::stat_keys::StatKey;
        let mut world = World::new();
        let unit = world.create_entity().build();
        let raw = |n: i32| i64::from(n) * 1024;
        let mut payload = serde_json::json!({ (WHEN_KEY): { "base": { "physical_armor_bonus": raw(2) },
            "night": { "physical_armor_bonus": raw(3), "preattack_bonus_damage": raw(10) }, "day": { "move_speed_bonus": raw(20) } } })
            .as_object().unwrap().clone();
        assert!(resolve_when(&mut payload, false));
        let mut buffs = BuffStore::new();
        buffs.add(unit, "dusk", Fixed64::from_i32(60), serde_json::Value::Object(payload));
        world.insert(buffs);
        let stat = |world: &World, key: StatKey| world.read_resource::<BuffStore>().sum_add(unit, key);
        assert_eq!((stat(&world, StatKey::PhysicalArmorBonus), stat(&world, StatKey::PreattackBonusDamage), stat(&world, StatKey::MoveSpeedBonus)),
            (Fixed64::from_i32(2), Fixed64::ZERO, Fixed64::from_i32(20)), "day as applied");
        world.insert(TemporaryNight { until: Some(Fixed64::from_i32(10)) });
        sync_when_rows(&world, Fixed64::from_i32(1));
        assert_eq!((stat(&world, StatKey::PhysicalArmorBonus), stat(&world, StatKey::PreattackBonusDamage), stat(&world, StatKey::MoveSpeedBonus)),
            (Fixed64::from_i32(5), Fixed64::from_i32(10), Fixed64::ZERO), "night");
        assert!(world.read_resource::<BuffStore>().get(unit, "dusk").unwrap().remaining == Fixed64::from_i32(60), "duration kept");
        sync_when_rows(&world, Fixed64::from_i32(10));
        assert_eq!(stat(&world, StatKey::PhysicalArmorBonus), Fixed64::from_i32(2), "the night ran out");
        assert_eq!(stat(&world, StatKey::MoveSpeedBonus), Fixed64::from_i32(20));
        let rewritten = world.read_resource::<BuffStore>().get(unit, "dusk").unwrap().payload.clone();
        sync_when_rows(&world, Fixed64::from_i32(11));
        assert_eq!(world.read_resource::<BuffStore>().get(unit, "dusk").unwrap().payload, rewritten, "nothing changed, nothing rewritten");
    }

    /// E453：施法送出的 time_of_day 封包由 process_outcomes 結算成夜晚，不留在 BuffStore。
    #[test]
    fn process_outcomes_settles_a_time_of_day_packet_without_storing_it() {
        use crate::runtime::{SingleLaneConfig, StateInitializer};
        let pool = StateInitializer::create_thread_pool();
        let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
        crate::runtime::setup_single_lane_match(&mut world, SingleLaneConfig::default()).unwrap();
        let hero = world.read_resource::<crate::runtime::MobaMatch>().heroes.iter().find_map(|slot| slot.entity).unwrap();
        assert!(!is_night(&world));
        world.write_resource::<Vec<crate::comp::Outcome>>().push(crate::comp::Outcome::AddBuff { target: hero,
            buff_id: TIME_OF_DAY_BUFF.into(), duration: Fixed64::ONE, payload: time_of_day_payload(true, Fixed64::from_i32(30)) });
        let mut events = crate::runtime::native::events::RuntimeEventVecSink::default();
        crate::runtime::native::game_processor::process_outcomes(&mut world, &mut events).unwrap();
        assert!(is_night(&world));
        assert!(!world.read_resource::<crate::runtime::BuffStore>().has(hero, TIME_OF_DAY_BUFF));
    }

    #[test]
    fn temporary_night_lasts_its_duration_and_overlaps_take_the_later_end() {
        let mut world = World::new();
        assert!(!is_night(&world));
        start_temporary_night(&mut world, Fixed64::from_i32(5));
        assert!(is_night(&world));
        assert_eq!(night_remaining(&world, Fixed64::from_i32(1)), Fixed64::from_i32(4));
        assert_eq!(night_remaining(&world, Fixed64::from_i32(9)), Fixed64::ZERO, "never negative");
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
