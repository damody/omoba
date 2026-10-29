//! Script gold grants and charges (v3 `give_gold` / `take_gold`; C06 偷天換日、殺人越貨、豪奪).
//!
//! 原作 `GivePlayerGold_UnReliable`（amhc.lua:509-522）以 `SetGold(id, GetGold(id) + gold, false)`
//! 實作：GetGold 是可靠＋不可靠的總額卻寫回不可靠金錢，玩家有可靠金錢時會被重複計入（FC-129～131）。
//! 本作每位玩家只有一個金錢池（`Gold`），這裡只把金額加上或扣掉：給錢不會多出一份，扣錢不會讓
//! 金錢變成負數（豪奪在資料裡先檢查對方金錢超過扣款）。
//!
//! 封包是一次性的 AddBuff（不存進 BuffStore），和其他 envelope 一樣在 cast 結算時准入、在序列
//! 結算時套用，authority 與隊伍 replica 跑同一個 cast 會得到同樣的結果。
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use super::comp::Gold;

pub const KEY: &str = "__script_gold_v1";
pub const BUFF: &str = "__script_gold";
/// Largest single grant or charge (gold units).
pub const MAX_DELTA: i64 = 1_000_000;

/// A gold change in whole gold units: positive grants, negative charges.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub delta: i64,
}

impl Spec {
    pub fn checked(delta: i64) -> Option<Self> {
        (delta != 0 && delta.abs() <= MAX_DELTA).then_some(Self { delta })
    }
    pub fn payload(self) -> Value {
        json!({ (KEY): { "delta": self.delta } })
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || duration != Fixed64::ONE || payload.as_object()?.len() != 1 {
            return None;
        }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 1 {
            return None;
        }
        Self::checked(row.get("delta")?.as_i64()?)
    }
}

/// The packet is well formed and the target has a gold pool.
pub fn admitted(world: &World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Spec> {
    let spec = Spec::from_payload(buff_id, duration, payload)?;
    world.read_storage::<Gold>().get(target).is_some().then_some(spec)
}

/// The pool after the change: a grant adds, a charge never goes below zero.
pub fn settle(gold: i32, delta: i64) -> i32 {
    (i64::from(gold) + delta).clamp(0, i64::from(i32::MAX)) as i32
}

pub fn apply(world: &mut World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> bool {
    let Some(spec) = admitted(world, target, buff_id, duration, payload) else { return false };
    if let Some(gold) = world.write_storage::<Gold>().get_mut(target) {
        gold.0 = settle(gold.0, spec.delta);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn world_with(gold: i32) -> (World, Entity) {
        let mut world = World::new();
        world.register::<Gold>();
        let unit = world.create_entity().with(Gold(gold)).build();
        (world, unit)
    }

    fn run(world: &mut World, unit: Entity, delta: i64) -> bool {
        apply(world, unit, BUFF, Fixed64::ONE, &Spec::checked(delta).unwrap().payload())
    }

    /// FC-129（偷天換日）／FC-130（殺人越貨）：給錢只加上金額。原作在可靠 300、不可靠 100 時給 50，
    /// 會把 300＋100＋50＝450 寫回不可靠金錢（總額變成 750）；本作只有一個池，400 給 50 是 450。
    #[test]
    fn a_grant_adds_exactly_the_amount() {
        let (mut world, unit) = world_with(400);
        assert!(run(&mut world, unit, 50));
        assert_eq!(world.read_storage::<Gold>().get(unit).unwrap().0, 450);
    }

    /// FC-131（豪奪）：扣 50 只扣 50；扣款超過餘額時歸零，不會變負。
    #[test]
    fn a_charge_takes_the_amount_and_never_goes_negative() {
        let (mut world, unit) = world_with(400);
        assert!(run(&mut world, unit, -50));
        assert_eq!(world.read_storage::<Gold>().get(unit).unwrap().0, 350);
        let (mut world, poor) = world_with(30);
        assert!(run(&mut world, poor, -50));
        assert_eq!(world.read_storage::<Gold>().get(poor).unwrap().0, 0);
    }

    #[test]
    fn malformed_packets_and_units_without_gold_are_refused() {
        let (mut world, unit) = world_with(100);
        let other = world.create_entity().build();
        assert!(!run(&mut world, other, 50), "no gold pool");
        assert!(Spec::checked(0).is_none() && Spec::checked(MAX_DELTA + 1).is_none());
        let payload = Spec::checked(5).unwrap().payload();
        assert!(!apply(&mut world, unit, "other", Fixed64::ONE, &payload));
        assert!(!apply(&mut world, unit, BUFF, Fixed64::from_i32(2), &payload));
        assert_eq!(world.read_storage::<Gold>().get(unit).unwrap().0, 100);
    }
}
