//! 傷害管線（behavior-model-v1 §1）。每一筆傷害封包——普攻、投射物、技能、道具、DoT——都在
//! `game_processor::handle_damage` 依序走過這裡的五段，順序就是結算順序：
//!
//! 1. **建立**（[`Packet`]）：來源 `source`（實際造成傷害的單位）、歸屬 `credit`（擊殺與助攻；
//!    目前與來源相同）、目標、三種傷害、封包旗標（`DamageProfile`：類型位元＋`SPELL`、
//!    `REQUIRE_SOURCE_ALIVE`）。[`admit`] 決定封包能不能進入管線：比賽規則（友軍、野怪歸屬）、
//!    replica 上由 authority 揭露生命的目標、`require_source_alive`。
//! 2. **減免前事件**（[`before_mitigation`]）：可以修改封包，依 [`BEFORE_MITIGATION`] 的固定順序。
//!    目前是法術增幅（E188）。結束時剝掉所有非類型旗標，TD 層與呈現只看到類型。
//! 3. **減免**：
//!    - 護甲與魔抗（[`defenses`]，含英雄魔抗池）在串列結算把 `RawDamage` 轉成 `Damage` 時套用，
//!      因為 fact 與 committed vitals 要的是已減免的值；已減免的封包（`Damage`）由產生端負責。
//!    - [`mitigate`]：依類型免傷（魔免使魔法傷害為 0，`eventfordamage.lua:30`）與傷害轉治療
//!      （`damage_immunity`），再乘受到傷害加成，最後乘建築受傷修正（E187，原作塔與大將的
//!      modifier_tower_armor／modifier_unit_armor 與所有建築的 debuff_tower，依攻擊者種類；
//!      `moba_match::structure_damage_factor`，只在依原作規則的比賽裡）。
//!    - [`absorb`]：護盾吸收，只對還活著的目標、正的傷害。
//! 4. **套用與減免後事件**：[`apply`] 扣生命並判定是否致死；[`after_mitigation`] 依序觸發回擊記錄、
//!    打斷回城、商店撤銷失效（命中即觸發），以及打斷前搖與傷害紀錄（實際扣血才觸發）。
//! 5. **致死與歸屬**：由 `handle_damage` 依 [`Applied::died`] 產生 `Death`、塔擊殺數與 TD 賞金；
//!    擊殺與助攻歸屬在第 4 段的傷害紀錄（`record_moba_*_damage`，帶 `first_lethal_hit`）。
//!
//! 管線只改結算的「位置」，不改數值：每一步沿用原本的定點運算與捨入順序。
use omb_script_abi::types::DamageProfile;
use omoba_sim::Fixed64;
use specs::{Entity, World, WorldExt};

use crate::runtime::comp::*;
use crate::runtime::{ability_runtime::UnitStats, BuffStore};

/// ① 一筆傷害。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Packet {
    pub source: Entity,
    /// 擊殺與助攻歸屬。舊的 Rust handler 封包一律是來源本身。
    pub credit: Entity,
    pub target: Entity,
    pub physical: Fixed64,
    pub magical: Fixed64,
    pub pure: Fixed64,
    /// `DamageProfile` 位元：類型＋旗標。
    pub profile: u32,
}

/// The credit a script named for the damage packet being settled (v3 `damage.credit`,
/// `Outcome::ScriptDamageCredit`); `handle_damage` takes it into [`Packet::credit`].
#[derive(Clone, Copy, Debug, Default)]
pub struct DamageCredit(pub Option<Entity>);

impl Packet {
    pub fn new(source: Entity, target: Entity, physical: Fixed64, magical: Fixed64, pure: Fixed64, profile: u32) -> Self {
        Self { source, credit: source, target, physical, magical, pure, profile }
    }

    pub fn requires_source_alive(&self) -> bool {
        self.profile & DamageProfile::REQUIRE_SOURCE_ALIVE.bits() != 0
    }
}

/// 封包旗標以外的類型位元；沒有類型的位元組合原樣保留，由後面的驗證拒絕。
pub const fn type_bits(profile: u32) -> u32 {
    if profile & DamageProfile::TYPE_BITS != 0 { profile & !DamageProfile::FLAG_BITS } else { profile }
}

/// 來源在結算當下是否還活著：實體存在，而且有生命的單位生命大於 0
/// （同一批次稍早被打死、Death 尚未提交的單位也算死亡）。
pub fn source_alive(world: &World, source: Entity) -> bool {
    world.entities().is_alive(source)
        && world.read_storage::<CProperty>().get(source).is_none_or(|property| property.hp > Fixed64::ZERO)
}

/// 封包能不能進入管線。
pub fn admit(world: &World, packet: &Packet) -> bool {
    if !crate::runtime::native::moba_match::moba_damage_from_allowed(world, Some(packet.source), packet.target) {
        return false;
    }
    // replica：生命由 authority 揭露的目標不在本地結算（m2-disclosure-policy.md）。
    if world.try_fetch::<crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets>()
        .is_some_and(|targets| targets.0.contains(&packet.target)) {
        return false;
    }
    // 原作 MODIFIER_STATE_INVULNERABLE：無敵的目標不受任何傷害。
    if world.try_fetch::<BuffStore>().is_some_and(|buffs| super::vocab::invulnerable(&buffs, packet.target)) {
        return false;
    }
    // 原作 AMHC:Damage（amhc.lua:540）：攻擊者已死就不造成傷害。
    !packet.requires_source_alive() || source_alive(world, packet.source)
}

/// ② 減免前事件，依序執行。新的全域規則（例如原作的結界、幻象倍率）加在這裡。
/// `status_damage_pre`：狀態的 on_deal_damage_pre／on_take_damage_pre（v3，產生的技能 Rust）可以改量或取消。
pub const BEFORE_MITIGATION: &[fn(&World, &mut Packet)] = &[spell_amplification, crate::runtime::native::scripting::damage_pre::status_damage_pre];

/// 法術增幅（E188）：帶 SPELL 的封包乘上來源的增幅。
fn spell_amplification(world: &World, packet: &mut Packet) {
    let (physical, magical, pure) = crate::runtime::native::spell_amplify::apply(
        &world.read_resource::<BuffStore>(), packet.source, packet.profile, packet.physical, packet.magical, packet.pure);
    (packet.physical, packet.magical, packet.pure) = (physical, magical, pure);
}

pub fn before_mitigation(world: &World, packet: &mut Packet) {
    for rule in BEFORE_MITIGATION { rule(world, packet); }
    packet.profile = type_bits(packet.profile);
}

/// ③ 物理格擋：普攻的物理傷害先減去目標的格擋（`StatKey::PhysicalConstantBlock`，Dota 在護甲之前），
/// 不低於 0；目標帶 `block_disabled`（v3.1）時不格擋。
pub fn physical_block(buffs: &BuffStore, target: Entity, physical: Fixed64) -> Fixed64 {
    let block = block_amount(buffs, target);
    if physical <= Fixed64::ZERO || block <= Fixed64::ZERO { physical } else { (physical - block).max(Fixed64::ZERO) }
}

/// 目標對普攻的物理格擋量；`block_disabled` 時為 0。
pub fn block_amount(buffs: &BuffStore, target: Entity) -> Fixed64 {
    if buffs.has(target, super::vocab::state::BLOCK_DISABLED) { return Fixed64::ZERO; }
    buffs.sum_add(target, omb_script_abi::stat_keys::StatKey::PhysicalConstantBlock).max(Fixed64::ZERO)
}

/// ③ 格擋、護甲與魔抗：`RawDamage` 轉成已減免封包時的結算。`attack` 表示普攻的封包（格擋只對普攻）。
/// 不合法的封包或已死亡的目標回傳 None。
pub fn defenses(world: &World, target: Entity, physical: Fixed64, magical: Fixed64, pure: Fixed64, profile: u32,
    attack: bool) -> Option<DamageTypes> {
    if [physical, magical, pure].iter().any(|value| *value < Fixed64::ZERO)
        || i128::from(physical.raw()) + i128::from(magical.raw()) + i128::from(pure.raw()) > i128::from(i64::MAX)
        || DamageProfile::from_bits(profile).is_none()
        || !world.entities().is_alive(target) { return None; }
    let properties = world.read_storage::<CProperty>();
    let property = properties.get(target)?;
    if property.hp <= Fixed64::ZERO { return None; }
    let damage = DamageTypes::new(physical, magical, pure);
    if world.read_storage::<Creep>().get(target).is_some_and(|creep| creep.td_layer.is_some()) { return Some(damage); }
    if physical == Fixed64::ZERO && magical == Fixed64::ZERO { return Some(damage); }
    let heroes = world.read_storage::<Hero>();
    let units = world.read_storage::<Unit>();
    let hero = heroes.get(target).is_some();
    let (armor, resistance) = if hero { (property.def_physic, property.def_magic) }
        else { units.get(target).map_or((property.def_physic, property.def_magic), |unit| (unit.base_armor, unit.magic_resistance)) };
    let buffs = world.read_resource::<BuffStore>();
    let stats = UnitStats::from_refs(&buffs, world.read_storage::<IsBuilding>().get(target).is_some());
    let physical = if attack { physical_block(&buffs, target, physical) } else { physical };
    let damage = DamageTypes::new(physical, magical, pure);
    // v3 `ignore_armor`: the physical part as if the target had no armor.
    let ignore_armor = profile & DamageProfile::IGNORE_ARMOR.bits() != 0;
    let armor = if physical > Fixed64::ZERO && !ignore_armor { stats.checked_final_armor(armor, target)? } else { Fixed64::ZERO };
    let resistance = if magical <= Fixed64::ZERO { Fixed64::ZERO } else {
        let base = if hero {
            let rules = world.try_fetch::<crate::runtime::MobaMatch>().and_then(|state| state.config.game_rules);
            super::magic_resist::hero_pool(&buffs, target, resistance, rules.as_ref())
        } else { resistance };
        stats.checked_final_magic_resist(base / Fixed64::from_i32(100), target)? * Fixed64::from_i32(100)
    };
    Some(mitigate_base_defenses(&damage, armor, resistance, false, false))
}

/// ③ 減免的結果（護盾吸收之前）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mitigated {
    /// 三種類型合計，乘過受到傷害加成。
    pub total: Fixed64,
    /// 轉成治療的部分（目標沒死才回復）。
    pub converted_heal: Fixed64,
}

pub fn mitigate(world: &World, packet: &Packet) -> Mitigated {
    let buffs = world.read_resource::<BuffStore>();
    let (physical, magical, pure, converted_heal) = crate::runtime::native::damage_immunity::settle(
        &buffs, packet.target, packet.physical, packet.magical, packet.pure);
    let total = UnitStats::from_refs(&buffs, false).incoming_damage_packet(physical, magical, pure, packet.target);
    // E187 structure defense (MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE, dummy.lua:510-639): an exact
    // rational factor on the authority's structure HP; nothing disclosed changes but the HP itself.
    let total = match crate::runtime::native::moba_match::structure_damage_factor(world, packet.source, packet.target) {
        Some((numerator, denominator)) => Fixed64::from_raw(
            (i128::from(total.raw()) * i128::from(numerator) / i128::from(denominator)) as i64),
        None => total,
    };
    Mitigated { total, converted_heal }
}

/// ③ 護盾吸收。回傳 (命中：正的傷害打在活著的目標上, 吸收後的傷害)。
pub fn absorb(world: &mut World, target: Entity, total: Fixed64) -> (bool, Fixed64) {
    let hit = total > Fixed64::ZERO
        && world.read_storage::<CProperty>().get(target).is_some_and(|property| property.hp > Fixed64::ZERO);
    let dealt = if hit { world.write_resource::<BuffStore>().absorb_damage_with_shield(target, total) } else { total };
    (hit, dealt)
}

/// ④ 扣生命的結果。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Applied {
    pub hp_before: Fixed64,
    pub hp_after: Fixed64,
    pub max_hp: Fixed64,
    /// 生命原本大於 0，而且確實扣了血。
    pub took_damage: bool,
    pub died: bool,
    /// 這一擊把活著的目標打到 0。
    pub first_lethal_hit: bool,
}

/// ④ 扣生命，生命不低於 0。沒有生命的目標回傳 None。
pub fn apply(world: &World, target: Entity, dealt: Fixed64) -> Option<Applied> {
    let mut properties = world.write_storage::<CProperty>();
    let property = properties.get_mut(target)?;
    let hp_before = property.hp;
    property.hp = property.hp - dealt;
    let took_damage = hp_before > Fixed64::ZERO && dealt > Fixed64::ZERO;
    let died = property.hp <= Fixed64::ZERO;
    if died { property.hp = Fixed64::ZERO; }
    Some(Applied { hp_before, hp_after: property.hp, max_hp: property.mhp, took_damage, died,
        first_lethal_hit: died && hp_before > Fixed64::ZERO && dealt > Fixed64::ZERO })
}

/// ④ 減免後事件。`mitigated` 是吸收前的量；`hit` 與 `applied` 來自 [`absorb`] 與 [`apply`]。
pub fn after_mitigation(world: &mut World, packet: &Packet, mitigated: &Mitigated, hit: bool, applied: Option<&Applied>) {
    use crate::runtime::native::moba_match;
    // 閒置的英雄回擊打他的人（retaliation.rs）；時間以 authority 的比賽時鐘為準。
    if mitigated.total > Fixed64::ZERO {
        let now = world.try_fetch::<crate::runtime::MobaMatch>().map(|state| state.elapsed);
        if let Some(now) = now { crate::runtime::native::retaliation::record(world, packet.source, packet.target, now); }
    }
    if hit {
        moba_match::interrupt_moba_recall(world, packet.target);
        // 在護盾把封包吃掉之前就失效；助攻與擊殺歸屬走下面的實際扣血。
        crate::runtime::native::shop::note_shop_undo_combat(world, Some(packet.source), packet.target);
    }
    let Some(applied) = applied.filter(|applied| applied.took_damage) else { return };
    crate::runtime::native::cast_windup::interrupt_for_damage(world, packet.target);
    // OnTakeDamage ends home_aura's speed_up (E187, dummy.lua:763-767).
    moba_match::break_home_speed_up(world, packet.target);
    let (credit, target, lethal) = (Some(packet.credit), packet.target, applied.first_lethal_hit);
    moba_match::record_moba_hero_damage(world, credit, target, lethal);
    moba_match::record_moba_jungle_damage(world, credit, target, lethal);
    moba_match::record_moba_lane_damage(world, credit, target, lethal);
    moba_match::record_moba_building_damage(world, credit, target, lethal);
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn property(hp: i32) -> CProperty {
        CProperty { hp: Fixed64::from_i32(hp), mhp: Fixed64::from_i32(hp.max(1)), msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }
    }

    fn world() -> World {
        crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool())
    }

    #[test]
    fn flags_are_stripped_after_the_events_and_types_survive() {
        let normal = DamageProfile::NORMAL.bits();
        let flagged = normal | DamageProfile::SPELL.bits() | DamageProfile::REQUIRE_SOURCE_ALIVE.bits();
        assert_eq!(type_bits(flagged), normal);
        assert_eq!(type_bits(DamageProfile::REQUIRE_SOURCE_ALIVE.bits()), DamageProfile::REQUIRE_SOURCE_ALIVE.bits(),
            "a packet without a type is left for validation to refuse");
        let mut w = world();
        let source = w.create_entity().with(property(100)).build();
        let target = w.create_entity().with(property(100)).build();
        let mut packet = Packet::new(source, target, Fixed64::from_i32(10), Fixed64::ZERO, Fixed64::ZERO, flagged);
        before_mitigation(&w, &mut packet);
        assert_eq!(packet.profile, normal);
        assert_eq!(packet.physical, Fixed64::from_i32(10), "no amplification without a source amplifier");
    }

    #[test]
    fn an_invulnerable_target_takes_no_damage_of_any_kind() {
        let mut w = world();
        let source = w.create_entity().with(property(100)).build();
        let target = w.create_entity().with(property(100)).build();
        let pure = Packet::new(source, target, Fixed64::ZERO, Fixed64::ZERO, Fixed64::from_i32(10), DamageProfile::NORMAL.bits());
        assert!(admit(&w, &pure));
        w.write_resource::<BuffStore>().add(target, super::super::vocab::state::INVULNERABLE, Fixed64::from_i32(3), serde_json::Value::Null);
        assert!(!admit(&w, &pure), "invulnerable refuses even pure damage");
        let attack = Packet::new(source, target, Fixed64::from_i32(10), Fixed64::ZERO, Fixed64::ZERO, DamageProfile::NORMAL.bits());
        assert!(!admit(&w, &attack));
        w.write_resource::<BuffStore>().remove(target, super::super::vocab::state::INVULNERABLE);
        assert!(admit(&w, &attack), "the state ends with its buff");
    }

    #[test]
    fn require_source_alive_drops_the_packet_of_a_dead_or_removed_source() {
        let mut w = world();
        let source = w.create_entity().with(property(100)).build();
        let target = w.create_entity().with(property(100)).build();
        let alive = DamageProfile::NORMAL.union(DamageProfile::REQUIRE_SOURCE_ALIVE).bits();
        let packet = Packet::new(source, target, Fixed64::from_i32(10), Fixed64::ZERO, Fixed64::ZERO, alive);
        assert!(admit(&w, &packet));
        // Killed earlier in the same batch: hp 0, Death not yet committed.
        w.write_storage::<CProperty>().get_mut(source).unwrap().hp = Fixed64::ZERO;
        assert!(!admit(&w, &packet));
        let unflagged = Packet { profile: DamageProfile::NORMAL.bits(), ..packet };
        assert!(admit(&w, &unflagged), "an ordinary packet still lands after its source died");
        w.delete_entity(source).unwrap();
        w.maintain();
        assert!(!admit(&w, &packet));
        assert!(admit(&w, &unflagged));
    }

    #[test]
    fn apply_clamps_at_zero_and_marks_only_the_first_lethal_hit() {
        let mut w = world();
        let target = w.create_entity().with(property(30)).build();
        let first = apply(&w, target, Fixed64::from_i32(20)).unwrap();
        assert!(first.took_damage && !first.died);
        let lethal = apply(&w, target, Fixed64::from_i32(20)).unwrap();
        assert!(lethal.died && lethal.first_lethal_hit);
        assert_eq!(lethal.hp_after, Fixed64::ZERO);
        let again = apply(&w, target, Fixed64::from_i32(20)).unwrap();
        assert!(again.died && !again.first_lethal_hit && !again.took_damage);
        let none = w.create_entity().build();
        assert!(apply(&w, none, Fixed64::ONE).is_none());
    }

    #[test]
    fn magic_immunity_zeroes_magical_damage_only() {
        let mut w = world();
        let source = w.create_entity().with(property(100)).build();
        let target = w.create_entity().with(property(100)).build();
        w.write_resource::<BuffStore>().add(target, "magic_immune", Fixed64::from_i32(3),
            serde_json::json!({"absolute_no_damage_magical": 1024}));
        let packet = Packet::new(source, target, Fixed64::from_i32(10), Fixed64::from_i32(40), Fixed64::from_i32(5),
            DamageProfile::NORMAL.bits());
        assert_eq!(mitigate(&w, &packet), Mitigated { total: Fixed64::from_i32(15), converted_heal: Fixed64::ZERO });
    }

    #[test]
    fn physical_immunity_zeroes_physical_before_any_bonus() {
        let mut w = world();
        let source = w.create_entity().with(property(100)).build();
        let target = w.create_entity().with(property(100)).build();
        w.write_resource::<BuffStore>().add(target, super::super::vocab::state::PHYSICAL_IMMUNE, Fixed64::from_i32(3), serde_json::Value::Null);
        w.write_resource::<BuffStore>().add(target, "frail", Fixed64::from_i32(3), serde_json::json!({"damage_taken_bonus": 1024}));
        let packet = Packet::new(source, target, Fixed64::from_i32(50), Fixed64::from_i32(10), Fixed64::ZERO,
            DamageProfile::NORMAL.bits());
        assert_eq!(mitigate(&w, &packet).total, Fixed64::from_i32(20), "only the magical 10, doubled");
    }

    #[test]
    fn block_reduces_attack_physical_only_and_can_be_disabled() {
        let mut w = world();
        let target = w.create_entity().with(property(100)).build();
        w.write_resource::<BuffStore>().add(target, "vanguard", Fixed64::from_i32(30), serde_json::json!({"physical_constant_block": 20 * 1024}));
        let normal = DamageProfile::NORMAL.bits();
        let ten = Fixed64::from_i32(10);
        let attack = defenses(&w, target, Fixed64::from_i32(50), ten, Fixed64::ZERO, normal, true).unwrap();
        assert_eq!((attack.physical, attack.magical), (Fixed64::from_i32(30), ten));
        let spell = defenses(&w, target, Fixed64::from_i32(50), ten, Fixed64::ZERO, normal, false).unwrap();
        assert_eq!(spell.physical, Fixed64::from_i32(50), "block is for attacks");
        assert_eq!(physical_block(&w.read_resource::<BuffStore>(), target, Fixed64::from_i32(15)), Fixed64::ZERO);
        w.write_resource::<BuffStore>().add(target, super::super::vocab::state::BLOCK_DISABLED, Fixed64::from_i32(3), serde_json::Value::Null);
        let disabled = defenses(&w, target, Fixed64::from_i32(50), ten, Fixed64::ZERO, normal, true).unwrap();
        assert_eq!(disabled.physical, Fixed64::from_i32(50));
    }
}
