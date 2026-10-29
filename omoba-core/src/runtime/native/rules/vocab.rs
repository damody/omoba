//! v3 詞彙（`docs/character_pipeline/tools/v3_vocab.lua` 的 `STATS`、`STATES`）在規則層的落點。
//!
//! - IR 編譯器（Lua，建置期）照名稱輸出 stat 與狀態旗標；執行期在載入 catalog 時用這裡解析，
//!   不認得的名稱拒絕載入（fail closed）。對照表只寫在這裡一份。
//! - v3 的 `pct` 數值在運算式裡是比例（v3.md §12.1：IR 編譯器除以 100），所以每個 stat 要宣告
//!   執行期的單位：[`Unit::Fraction`] 直接用比例；[`Unit::PercentPoints`] 執行期以百分點計
//!   （例如魔抗類），載入時乘 100。
//! - M2 先放規則層已實作的部分（含 v3.1 §12.10 的追加）；其餘 stat 由 M3 的 IR 載入補齊。
use omb_script_abi::stat_keys::StatKey;

/// 執行期 StatKey 的單位。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// 與運算式的值相同（比例或絕對數值）。
    Fraction,
    /// 執行期以百分點計：運算式的比例 × 100。
    PercentPoints,
}

/// v3 stat 名稱 → (執行期 StatKey, 單位)。
pub const STATS: &[(&str, StatKey, Unit)] = &[
    ("magic_resist", StatKey::MagicalResistanceBonus, Unit::PercentPoints),
    // v3.1：nobu 的英雄魔抗池（rules::magic_resist）。
    ("magic_resist_pool", StatKey::MagicResistancePool, Unit::PercentPoints),
    ("armor", StatKey::PhysicalArmorBonus, Unit::Fraction),
    ("move_speed", StatKey::MoveSpeedBonusBuff, Unit::Fraction),
    // v3.1：覆寫基礎移速，加成照算（BuffStore 的 `movespeed_base_override`）。
    ("base_move_speed", StatKey::MoveSpeedBaseOverride, Unit::Fraction),
    ("attack_speed", StatKey::AttackSpeedBonusConstant, Unit::Fraction),
    // Attribute points; the host attributes settle converts them (attributes.rs).
    ("str", StatKey::StatsStrengthBonus, Unit::Fraction),
    ("agi", StatKey::StatsAgilityBonus, Unit::Fraction),
    ("int", StatKey::StatsIntellectBonus, Unit::Fraction),
    ("evasion_pct", StatKey::EvasionConstant, Unit::Fraction),
    // v3.1：攻擊落空（致盲）。
    ("miss_chance_pct", StatKey::MissPercentage, Unit::Fraction),
    ("spell_amp_pct", StatKey::SpellAmplifyPercentage, Unit::Fraction),
    ("incoming_damage_pct", StatKey::IncomingDamagePercentage, Unit::Fraction),
    ("hp_regen", StatKey::HealthRegenConstant, Unit::Fraction),
    ("mana_regen", StatKey::ManaRegenConstant, Unit::Fraction),
    // v3.1：每秒回復最大值的比例，與回復量增幅（rules::regen）。
    ("hp_regen_pct", StatKey::HealthRegenMaxPercentage, Unit::Fraction),
    ("mana_regen_pct", StatKey::ManaRegenMaxPercentage, Unit::Fraction),
    ("hp_regen_amp_pct", StatKey::HpRegenAmplifyPercentage, Unit::Fraction),
    ("mana_regen_amp_pct", StatKey::ManaRegenAmplifyPercentage, Unit::Fraction),
    ("physical_block", StatKey::PhysicalConstantBlock, Unit::Fraction),
    ("max_hp", StatKey::HealthBonus, Unit::Fraction),
    // 白字攻擊（原作 BASEATTACK_BONUSDAMAGE）、攻擊距離、基礎傷害百分比（BASEDAMAGEOUTGOING_PERCENTAGE）。
    ("base_attack_damage", StatKey::BaseAttackBonusDamage, Unit::Fraction),
    ("attack_range", StatKey::AttackRangeBonus, Unit::Fraction),
    ("base_attack_damage_pct", StatKey::BaseDamageOutgoingPercentage, Unit::Fraction),
    // 現行 handler（C17E 勝利祈願）把攻擊力百分比寫在同一個引擎 stat；乘白字或總攻擊由 M1 決定（kit quirk）。
    ("attack_damage_pct", StatKey::BaseDamageOutgoingPercentage, Unit::Fraction),
    ("max_mana", StatKey::ManaBonus, Unit::Fraction),
    // 工作線 A（2026-10-29，B 的分工）：handler 用的 StatKey，規則層都有讀。
    // 攻擊力加成（B06Q、B08E handler 的 PreattackBonusDamage）。
    ("attack_damage", StatKey::PreattackBonusDamage, Unit::Fraction),
    // 移速百分比（B06T handler 的 MoveSpeedBonusPercentage，比例）。
    ("move_speed_pct", StatKey::MoveSpeedBonusPercentage, Unit::Fraction),
    // 受到物理傷害百分比（unit_stats 的 IncomingPhysicalDamagePercentage）。
    ("incoming_physical_pct", StatKey::IncomingPhysicalDamagePercentage, Unit::Fraction),
    // 吸血比例（unit_stats 的 Lifesteal，夾在 0～1）。
    ("lifesteal_pct", StatKey::Lifesteal, Unit::Fraction),
];

/// 規則層直接讀的狀態旗標（BuffStore 的狀態 id 就是 v3 名稱）。
pub mod state {
    /// v3.1：物理傷害在減免段歸零，先於所有增減傷百分比（rules::damage、damage_immunity）。
    pub const PHYSICAL_IMMUNE: &str = "physical_immune";
    /// v3.1：強制近戰（原作 SetAttackCapability）；`is_ranged` 跟著反映（rules::attack）。
    pub const MELEE_ATTACK: &str = "melee_attack";
    /// v3.1：強制遠程。
    pub const RANGED_ATTACK: &str = "ranged_attack";
    /// v3.1：物理格擋失效（rules::damage::physical_block）。
    pub const BLOCK_DISABLED: &str = "block_disabled";
    /// v3.1：只改尋路，不給飛行視野（rules::world::flying_for_pathing）。
    pub const FLYING_FOR_PATHING: &str = "flying_for_pathing";
    /// 原作 MODIFIER_STATE_INVULNERABLE：不受任何傷害（rules::damage::admit），技能與道具的範圍選取、
    /// 英雄的自動普攻都選不到（selector 要明確 `include_invulnerable` 才包含）。
    pub const INVULNERABLE: &str = "invulnerable";
    /// 原作 MODIFIER_STATE_COMMAND_RESTRICTED：擁有者的新指令一律拒絕（command_restriction.rs，BUFF 同名）。
    pub const COMMAND_RESTRICTED: &str = "command_restricted";
    /// 嘲諷／強制攻擊（taunt.rs）：payload `taunt_target` 指名對象；期間拒絕新指令。
    pub const TAUNT: &str = "taunt";
}

/// 單位是否無敵（[`state::INVULNERABLE`]）。
pub fn invulnerable(buffs: &crate::runtime::BuffStore, entity: specs::Entity) -> bool {
    buffs.has(entity, state::INVULNERABLE)
}

/// v3 stat 名稱的執行期落點；不認得回傳 None。
pub fn stat(name: &str) -> Option<(StatKey, Unit)> {
    STATS.iter().find(|(vocab, _, _)| *vocab == name).map(|(_, key, unit)| (*key, *unit))
}

/// `op = "set_absolute"` 的 stat（取代基礎值，不是加成）：v3 名稱 → 執行期 StatKey。
/// `base_attack_time`：原作 modifier 的 base_attack_time，host 以最小的覆寫值取代單位的基礎攻擊間隔
/// （`UnitStats::base_attack_interval`）。
const ABSOLUTE_STATS: &[(&str, StatKey, Unit)] = &[
    ("base_attack_time", StatKey::BaseAttackTimeConstant, Unit::Fraction),
];

/// 狀態旗標（v3 `states`）在 host 的落點。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateEffect {
    /// 狀態本身的 payload 帶這個 StatKey 與 raw（例如隱形等級）。
    Payload(StatKey, i64),
    /// host 以獨立的狀態 buff 表達（同一時長，payload 為空）。
    Buff(&'static str),
}

/// v3 狀態旗標 → host 落點。
const STATE_EFFECTS: &[(&str, StateEffect)] = &[
    // 隱形（攻擊不現形；BuffStore::stealth_level 讀等級）。
    ("invisible", StateEffect::Payload(StatKey::InvisibilityLevel, 1024)),
    // 無單位碰撞（BuffStore::ignores_unit_collision）。
    ("no_unit_collision", StateEffect::Buff(crate::runtime::native::ability_runtime::NO_UNIT_COLLISION)),
    // Host controls (BuffStore::is_stunned / is_rooted / is_silenced, status resistance, magic immunity).
    ("stun", StateEffect::Buff("stun")),
    ("root", StateEffect::Buff("root")),
    ("silence", StateEffect::Buff("silence")),
];

/// 狀態旗標的 host 落點；不認得回傳 None（載入失敗）。
pub fn state(name: &str) -> Option<StateEffect> {
    STATE_EFFECTS.iter().find(|(vocab, _)| *vocab == name).map(|(_, effect)| *effect)
}

/// `set_absolute` stat 的執行期落點；不認得回傳 None。
pub fn absolute_stat(name: &str) -> Option<(StatKey, Unit)> {
    ABSOLUTE_STATS.iter().find(|(vocab, _, _)| *vocab == name).map(|(_, key, unit)| (*key, *unit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_unique_and_keys_round_trip_through_the_wire_names() {
        let mut names: Vec<_> = STATS.iter().map(|(name, _, _)| *name).collect();
        names.sort();
        let before = names.len();
        names.dedup();
        assert_eq!(names.len(), before, "a vocabulary name maps once");
        for (name, key, _) in STATS {
            assert_eq!(StatKey::from_str_key(key.as_str()), Some(*key), "{name}");
        }
        assert_eq!(stat("magic_resist_pool"), Some((StatKey::MagicResistancePool, Unit::PercentPoints)));
        assert_eq!(stat("not_a_stat"), None);
        assert_eq!(absolute_stat("base_attack_time"), Some((StatKey::BaseAttackTimeConstant, Unit::Fraction)));
        assert_eq!(stat("base_attack_time"), None, "only as an absolute value");
        assert_eq!(state("invisible"), Some(StateEffect::Payload(StatKey::InvisibilityLevel, 1024)));
        assert_eq!(state("no_unit_collision"), Some(StateEffect::Buff("no_unit_collision")));
        assert_eq!(state("not_a_state"), None);
    }
}
