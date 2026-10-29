//! 偽隨機 `chance { p, pity_after }`（behavior-model-v1 §1）。
//!
//! 原作的爆擊與觸發幾乎都不是引擎的 pseudo-random，而是「`RandomInt` 擲骰＋未觸發計數保底」：
//! - C01R（`C01_Mitsuhide_Akechi.lua:481-519`）：`RandomInt(0,100) <= 18`，沒觸發就計數，計數
//!   超過 5 時下一擊必定觸發（第 7 擊）。
//! - 武尊環（`item_brave_ring.lua:27-42`）：機率 15，計數超過 7 必定觸發。
//!
//! 這裡的定義：`pity_after = n` 表示連續 n 次沒觸發之後，下一次一定觸發；觸發就把計數歸零。
//! 計數器以 [`PityKey`] 分開：持有者加上共用範圍（同一個技能、同一位英雄、同名道具），
//! 由資料宣告範圍，不同技能各自計數。
//!
//! 決定性：擲骰是 `(seed, tick, 擲骰鍵, op)` 的純函式（與 `attack_accuracy` 同一套）。
//! **擲骰鍵**（[`roll_key`]）用雙方都有的身分：英雄用玩家 id（replica 的 ECS id 是隊伍本地的重新編號，
//! 與 authority 不同，m2-disclosure-policy §5.2），其他單位沿用 entity id。計數器是 `team` 狀態；
//! 鍵一致時，己方 replica 的計數與 authority 相同。
use omoba_sim::Fixed64;
use specs::Entity;
use std::collections::BTreeMap;

/// 一個機率宣告。`p` 是 0..=1 的機率；`pity_after` 見模組說明。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chance {
    pub p: Fixed64,
    pub pity_after: Option<u16>,
}

impl Chance {
    pub const fn new(p: Fixed64) -> Self { Self { p, pity_after: None } }
    pub const fn with_pity(p: Fixed64, pity_after: u16) -> Self { Self { p, pity_after: Some(pity_after) } }
}

/// 計數器的共用範圍。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PityScope {
    /// 同一個技能（技能 id）。
    Ability(String),
    /// 同一位英雄的所有來源共用。
    Hero,
    /// 同名道具共用（道具 id）。
    Item(String),
    /// 同一個狀態（狀態 id）。爆擊來源的保底以狀態為範圍。
    Status(String),
}

/// 計數器的鍵：持有者（id 加世代，實體回收後不會沿用舊計數）＋範圍。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PityKey {
    pub holder: (u32, i32),
    pub scope: PityScope,
}

impl PityKey {
    pub fn new(holder: Entity, scope: PityScope) -> Self { Self { holder: (holder.id(), holder.gen().id()), scope } }
}

/// 所有偽隨機計數器。`BTreeMap` 讓走訪與雜湊順序固定。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PityCounters(pub BTreeMap<PityKey, u16>);

impl PityCounters {
    /// 擲一次：`unit_roll` 是 [0, 1) 的均勻值（呼叫端用決定性亂數算出）。
    pub fn roll(&mut self, key: PityKey, chance: Chance, unit_roll: Fixed64) -> bool {
        let misses = self.0.get(&key).copied().unwrap_or(0);
        let forced = chance.pity_after.is_some_and(|after| misses >= after);
        let hit = forced || (chance.p > Fixed64::ZERO && unit_roll < chance.p);
        if chance.pity_after.is_some() {
            if hit { self.0.remove(&key); } else { self.0.insert(key, misses.saturating_add(1)); }
        }
        hit
    }

    /// 持有者消失時移除它的計數器。
    pub fn retain_alive(&mut self, alive: impl Fn((u32, i32)) -> bool) {
        self.0.retain(|key, _| alive(key.holder));
    }
}

/// 英雄擲骰鍵的標記位元：與 entity id 分開（entity id 遠小於 2^31）。
pub const HERO_KEY_BIT: u32 = 1 << 31;

/// 擲骰鍵：英雄（`hero_player` 是它的玩家 id）用玩家 id，其他單位用 entity id。
pub fn roll_key(entity: Entity, hero_player: Option<u32>) -> u64 {
    hero_player.map_or(u64::from(entity.id()), |player| u64::from(HERO_KEY_BIT | player))
}

/// 從世界讀出擲骰鍵：有 `Hero` 與 `PlayerOwner` 的單位是英雄。
pub fn roll_key_of(world: &specs::World, entity: Entity) -> u64 {
    use specs::WorldExt;
    let hero = world.has_value::<specs::storage::MaskedStorage<crate::runtime::comp::Hero>>()
        && world.read_storage::<crate::runtime::comp::Hero>().get(entity).is_some();
    let player = (hero && world.has_value::<specs::storage::MaskedStorage<crate::runtime::comp::PlayerOwner>>())
        .then(|| world.read_storage::<crate::runtime::comp::PlayerOwner>().get(entity).map(|owner| owner.player_id)).flatten();
    roll_key(entity, player)
}

/// `(seed, tick, 擲骰鍵, op)` 的決定性 [0, 1) 值，與 `attack_accuracy` 的擲骰同一套。
pub fn unit_roll(seed: u64, tick: u32, key: u64, op: u16) -> Fixed64 {
    let scale = omoba_sim::fixed::SCALE as u64;
    Fixed64::from_raw((crate::runtime::tick_random_u64(seed, u64::from(tick), (key << 16) | u64::from(op)) % scale) as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn pity_forces_the_hit_after_n_misses_and_resets() {
        let mut world = World::new();
        let hero = world.create_entity().build();
        let mut counters = PityCounters::default();
        let key = || PityKey::new(hero, PityScope::Ability("c01_r".into()));
        // C01R: about 19% with a guaranteed 7th attack after six misses.
        let chance = Chance::with_pity(Fixed64::from_raw(194), 6);
        let never = Fixed64::from_raw(1023);
        for _ in 0..6 { assert!(!counters.roll(key(), chance, never)); }
        assert!(counters.roll(key(), chance, never), "the 7th attack is forced");
        assert!(counters.0.is_empty(), "a hit resets the count");
        assert!(!counters.roll(key(), chance, never));
        assert!(counters.roll(key(), chance, Fixed64::ZERO), "an ordinary hit");
        assert!(counters.0.is_empty());
    }

    #[test]
    fn scopes_count_apart_and_plain_chances_keep_no_state() {
        let mut world = World::new();
        let (a, b) = (world.create_entity().build(), world.create_entity().build());
        let mut counters = PityCounters::default();
        let chance = Chance::with_pity(Fixed64::from_raw(100), 2);
        let never = Fixed64::from_raw(1023);
        counters.roll(PityKey::new(a, PityScope::Item("item_brave_ring".into())), chance, never);
        counters.roll(PityKey::new(a, PityScope::Hero), chance, never);
        counters.roll(PityKey::new(b, PityScope::Item("item_brave_ring".into())), chance, never);
        assert_eq!(counters.0.len(), 3);
        assert!(!counters.roll(PityKey::new(a, PityScope::Hero), Chance::new(Fixed64::from_raw(100)), never));
        assert_eq!(counters.0.get(&PityKey::new(a, PityScope::Hero)), Some(&1), "a plain chance leaves the count alone");
        counters.retain_alive(|holder| holder != (a.id(), a.gen().id()));
        assert_eq!(counters.0.len(), 1);
    }

    #[test]
    fn unit_rolls_are_a_pure_function_of_their_inputs() {
        let mut world = World::new();
        let hero = world.create_entity().build();
        let key = roll_key(hero, None);
        assert_eq!(key, u64::from(hero.id()));
        assert_eq!(unit_roll(7, 100, key, 1), unit_roll(7, 100, key, 1));
        let rolls: std::collections::BTreeSet<_> = (0..64).map(|tick| unit_roll(7, tick, key, 1).raw()).collect();
        assert!(rolls.len() > 32 && rolls.iter().all(|raw| (0..1024).contains(raw)));
        // A hero rolls by its player, whatever its local entity id (authority or team replica).
        let other = world.create_entity().build();
        assert_eq!(roll_key(hero, Some(3)), roll_key(other, Some(3)));
        assert_ne!(roll_key(hero, Some(3)), roll_key(hero, None));
    }
}
