"""
Generator for scripts/script-abi/src/stat_keys.rs.

Reads stat_keys.rs.backup (snapshot of the const-based version),
parses all `pub const FOO: &str = "...";` entries and the
BUILDING_EXCLUDED_KEYS array, then produces the enum-based stat_keys.rs.

Rules:
- SECTION 1/2/3 comment banners determine default StatSection
  (All / NonBuilding / Visual).
- Keys listed in BUILDING_EXCLUDED_KEYS are forced to NonBuilding
  (override of whatever section they were declared in).
- Aggregation is classified by the SNAKE_CASE wire-string suffix:
    _percentage (incl. _percentage_unique, _percentage_unique_2) -> SumAddThenMul1Plus
    _multiplier -> ProductMult
    _chance -> Chance
    _bonus / _constant / _stacking / _unique / _absolute / _amplify / _flat -> SumAdd
    everything else -> PassThrough
- PascalCase is produced by greedy longest-match on a curated dictionary
  (see WORDS), with the Dota 2 prefixes preattack/procattack preserved
  as single tokens.

After migration, 7 new variants are appended to cover tower/hero scripts
that currently use magic strings.

Usage:
    python docs/tools/gen_stat_keys.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

BACKUP_PATH = Path("D:/omoba/scripts/script-abi/src/stat_keys.rs.backup")
OUTPUT_PATH = Path("D:/omoba/scripts/script-abi/src/stat_keys.rs")

# Longer entries must come first for greedy matching.
WORDS = sorted(
    [
        # Dota prefixes (preserved as single token) — per spec Task 2 polish.
        # Only these two compound prefixes stay as one word; all other compound
        # SNAKE tokens split on word boundary via the dictionary below.
        "preattack",
        "procattack",
        # Clearly single English words
        "cooldown",
        "reincarnation",
        "invisibility",
        "invincible",
        "invulnerable",
        "autoattack",
        # Compound-ish
        "illusion",
        "multiplier",
        "percentage",
        "constant",
        "stacking",
        "absolute",
        "amplify",
        "feedback",
        "override",
        "negative",
        "physical",
        "magical",
        "strength",
        "agility",
        "intellect",
        "decrepify",
        "accuracy",
        "projectile",
        "evasion",
        "lifesteal",
        "healing",
        "reflect",
        "absorb",
        "disable",
        "respawn",
        "resistance",
        "modification",
        "direct",
        "ignore",
        "target",
        "incoming",
        "outgoing",
        "animation",
        "weight",
        "splash",
        "chance",
        "bonus",
        "unique",
        "active",
        "special",
        "extra",
        "total",
        "limit",
        "always",
        "allow",
        "needs",
        "refresh",
        "slow",
        "stun",
        "duration",
        "factor",
        "received",
        "effect",
        "cost",
        "boost",
        "rate",
        "angle",
        "turning",
        "turn",
        "vision",
        "fixed",
        "night",
        "day",
        "bounty",
        "creep",
        "other",
        "post",
        "crit",
        "proc",
        "pre",
        "base",
        "max",
        "min",
        "stats",
        "mana",
        "health",
        "hp",
        "mp",
        "regen",
        "cast",
        "attack",
        "speed",
        "damage",
        "armor",
        "block",
        "spell",
        "pure",
        "miss",
        "avoid",
        "level",
        "label",
        "super",
        "ultimate",
        "with",
        "reduction",
        "time",
        "point",
        "range",
        "unit",
        "persistent",
        "pipeline",
        "no",
        "is",
        "buff",
        "id",
        "root",
        "silence",
        "invisible",
        "exp",
        "xp",
        "experience",
        "kill",
        "gold",
        "minimap",
        # Added for splitting CASTTIME, RESPAWNTIME, DEATHGOLDCOST,
        # MAGICDAMAGEOUTGOING, MOVESPEED, PREATTACK_CRITICALSTRIKE, etc.
        "death",
        "magic",
        "move",
        "critical",
        "strike",
        # Added to silence to_pascal fallback warnings (已存在於 wire 中但字典漏列)
        "unavoidable",
        "heal",
    ],
    key=lambda w: (-len(w), w),
)


def to_pascal(snake: str) -> str:
    """Greedy-longest-match word split, then PascalCase join."""
    s = snake.lower()
    out: list[str] = []
    i = 0
    n = len(s)
    while i < n:
        if s[i] == "_":
            i += 1
            continue
        # Trailing digit like "_2"? Consume digits as their own token.
        if s[i].isdigit():
            j = i
            while j < n and s[j].isdigit():
                j += 1
            out.append(s[i:j])
            i = j
            continue
        matched = False
        for w in WORDS:
            if s.startswith(w, i):
                # Greedy longest-match: accept regardless of following char
                # (WORDS is sorted longest-first, so "preattack" wins over
                # "pre"+"attack" and "magic" can split "magicdamage"). This
                # relies on the dictionary being curated — no word should be
                # a harmful prefix of another wire segment.
                out.append(w)
                i += len(w)
                matched = True
                break
        if not matched:
            # Fallback: take until next `_` or digit
            j = i
            while j < n and s[j].isalpha():
                j += 1
            chunk = s[i:j]
            if not chunk:
                raise RuntimeError(f"Cannot tokenize {snake!r} at {i}")
            # 未知 chunk：WORDS 字典沒涵蓋 → 印 warning 提示維護者擴字典，
            # 否則會 silent 以 literal chunk 命名導致誤產 Pascal 名。
            print(
                f"warn: no dictionary match for chunk {chunk!r} in snake {snake!r}",
                file=sys.stderr,
            )
            out.append(chunk)
            i = j
    return "".join(p[:1].upper() + p[1:] for p in out)


def classify_aggregation(wire: str) -> str:
    """Return one of: SumAdd, SumAddThenMul1Plus, ProductMult, Chance, PassThrough.

    Rules (priority order on the SNAKE-underscored segments):
      1. Contains `multiplier` segment          -> ProductMult
      2. Contains `chance` segment              -> Chance
      3. Contains `percentage` segment          -> SumAddThenMul1Plus
      4. Contains `bonus`/`constant`/`stacking`/
         `absolute`/`amplify`/`flat` segment    -> SumAdd
      5. Else                                   -> PassThrough

    Note: Dota keys sometimes place the aggregation suffix mid-word (e.g.
    `preattack_bonus_damage`), so we test whole-word segment membership rather
    than end-match.
    """
    segs = set(wire.split("_"))
    if "multiplier" in segs:
        return "ProductMult"
    if "chance" in segs:
        return "Chance"
    if "percentage" in segs:
        return "SumAddThenMul1Plus"
    sum_add = {"bonus", "constant", "stacking", "absolute", "amplify", "flat"}
    if segs & sum_add:
        return "SumAdd"
    return "PassThrough"


# Dota wire format 不含典型後綴但語義已知的特例覆寫。
# 若 classify_aggregation 的 suffix rule 無法正確處理某個 wire，在此加入覆寫。
AGGREGATION_OVERRIDES: dict[str, str] = {
    # 受攻擊反擊傷害，多個 buff 疊加（Dota 2 語義）
    "procattack_feedback": "SumAdd",
    # 視覺多段射擊計數（無 _bonus suffix 但需疊加）
    "multi_shot_visual": "SumAdd",
    # DoT tick 傷害，多個 dot buff 疊加傷害
    "dot_damage": "SumAdd",
}


CONST_RE = re.compile(
    r'^pub const ([A-Z][A-Z0-9_]*): &str = "([^"]+)";', re.MULTILINE
)
BUILDING_EXCLUDED_ARRAY_RE = re.compile(
    r"pub const BUILDING_EXCLUDED_KEYS: &\[&str\] = &\[(.*?)\];",
    re.DOTALL,
)
SECTION_RE = re.compile(r"^// SECTION (\d+) —", re.MULTILINE)


def parse_backup(text: str):
    """Return entries: list of (const_name, wire, section_num) in file order.

    BUFF_ID_* consts from older backups are silently skipped — they are now
    modelled as the `BuffId` enum in `scripts/script-abi/src/buff_ids.rs`,
    which is hand-maintained and not generated from this script.
    """
    # Find SECTION banner positions to classify each const by file offset
    section_banners = [
        (m.start(), int(m.group(1))) for m in SECTION_RE.finditer(text)
    ]
    # Default section = 1 for anything before SECTION 1 banner
    # (Dota pre-SECTION content). In practice SECTION 1 banner is the first.
    def section_of(pos: int) -> int:
        cur = 1
        for start, num in section_banners:
            if pos >= start:
                cur = num
            else:
                break
        return cur

    entries: list[tuple[str, str, int]] = []
    for m in CONST_RE.finditer(text):
        name, wire = m.group(1), m.group(2)
        if name == "BUILDING_EXCLUDED_KEYS":
            continue
        if name.startswith("BUFF_ID_"):
            # Buff identifier 已遷至 BuffId enum（buff_ids.rs），這裡不產出。
            continue
        entries.append((name, wire, section_of(m.start())))
    return entries


def parse_excluded_wires(text: str, name_to_wire: dict[str, str]) -> set[str]:
    m = BUILDING_EXCLUDED_ARRAY_RE.search(text)
    if not m:
        return set()
    body = m.group(1)
    # Items inside are const names (identifiers), one per line
    names = re.findall(r"\b([A-Z][A-Z0-9_]*)\b", body)
    out: set[str] = set()
    for n in names:
        if n in name_to_wire:
            out.add(name_to_wire[n])
    return out


# New variants appended at tail.
# Section 選擇規則（對照現有同族 variant）：
#   - MoveSpeedBonus 同 MoveSpeedBonusEquipment 等 → NonBuilding
#   - DamageTakenBonus 同 IncomingDamage* 系 → All（建築物也受傷）
#   - MultiShotVisual 視覺 pass-through → Visual
#   - DotDamage 遊戲邏輯（DoT 傷害 tick）→ All
NEW_VARIANTS: list[tuple[str, str, str, str]] = [
    # (pascal, wire, section, aggregation)
    ("CritChance", "crit_chance", "All", "Chance"),
    ("CritBonus", "crit_bonus", "All", "SumAdd"),
    ("SplashBonus", "splash_bonus", "All", "SumAdd"),
    ("SlowFactorOverride", "slow_factor_override", "All", "PassThrough"),
    ("SlowDurationBonus", "slow_duration_bonus", "All", "SumAdd"),
    ("AttackStunChance", "attack_stun_chance", "All", "Chance"),
    ("AttackStunDuration", "attack_stun_duration", "All", "SumAdd"),
    # Task: enum-ify 4 previously private payload keys (remove sum_add_str backdoor)
    ("MoveSpeedBonus", "move_speed_bonus", "NonBuilding", "SumAdd"),
    ("DamageTakenBonus", "damage_taken_bonus", "All", "SumAdd"),
    ("MultiShotVisual", "multi_shot_visual", "Visual", "SumAdd"),
    ("DotDamage", "dot_damage", "All", "SumAdd"),
]


FILE_HEADER = '''//! 共用 stat key 枚舉 — 對應 Dota 2 MODIFIER_PROPERTY_*（~140 項）。
//!
//! 所有 DLL 腳本透過 `world.add_stat_buff(..., modifiers_json)` 寫入屬性修改；
//! host 端 `UnitStats` helper 聚合並套用到 tick 系統 / damage pipeline。
//!
//! 命名慣例（與 BuffStore 聚合對應）：
//! * `*Bonus` / `*Constant` / `*Stacking` → 加法聚合（`sum_add`）；空為 0
//! * `*Percentage` → 加法聚合後當倍率（乘到 `1 + sum`）
//! * `*Multiplier` → 乘法聚合（`product_mult`）；空為 1
//! * `*Chance` → 觸發機率（0..=1）
//!
//! 分三大 section（見 `StatSection`）：
//! 1. **All**：全單位通用（包含建築物）
//! 2. **NonBuilding**：僅非建築物
//! 3. **Visual**：純視覺 / 前端 pass-through
//!
//! 此檔由 `docs/tools/gen_stat_keys.py` 產生；請勿手改。
//! 新增 variant 請改 generator 後重跑並 commit 兩個檔案。

use abi_stable::StableAbi;

/// StatKey 的三分章節分類（對應 Dota 2 modifier property 的通用性）。
///
/// - `All`：全單位通用（包含建築物）
/// - `NonBuilding`：僅非建築物（有 IsBuilding 標記的實體會跳過）
/// - `Visual`：純視覺／前端 pass-through（不影響遊戲邏輯）
#[repr(u8)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StatSection {
    All = 0,
    NonBuilding = 1,
    Visual = 2,
}

/// StatKey 的聚合方式（對應 BuffStore 的 payload 數值合併規則）。
///
/// - `SumAdd`：`_Bonus` / `_Constant` / `_Stacking` — 多個 buff 的數值直接相加（預設 0）
/// - `SumAddThenMul1Plus`：`_Percentage` — 相加後當倍率係數套用 `(1.0 + sum)`
/// - `ProductMult`：`_Multiplier` — 多個 buff 的數值相乘（預設 1.0）
/// - `Chance`：`_Chance` — 觸發機率 `[0.0, 1.0]`，通常只取最大值或 OR 邏輯
/// - `PassThrough`：視覺 / 覆蓋型（例 `*_override`）— host 直接轉發原值不聚合
#[repr(u8)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Aggregation {
    SumAdd = 0,
    SumAddThenMul1Plus = 1,
    ProductMult = 2,
    Chance = 3,
    PassThrough = 4,
}

'''


def build_enum_block(entries: list[tuple[str, str, str, str, int]]) -> str:
    """entries: list of (pascal, wire, section_enum, aggregation_enum, discriminant)."""
    lines: list[str] = []
    lines.append("/// Script ABI 的 stat key 枚舉。")
    lines.append("///")
    lines.append("/// # SAFETY")
    lines.append("/// Variant 順序 = FFI ABI 契約：新增只能 **追加到尾端**，絕不可在中間 insert")
    lines.append("/// 或更動 discriminant 值，否則 host 與 script DLL 版本不同步會 UB。")
    lines.append("/// **刪除亦禁止**：廢棄 variant 請保留佔位並標 `#[deprecated]`，")
    lines.append("/// 避免後續 discriminant 重編號錯位。")
    lines.append("/// 每個 variant 顯式寫 `= N` 以鎖定值。")
    lines.append("// u16 而非 u8：目前 variant 數 < 256 但留擴充空間（未來若對齊 Dota 2 完整 modifier list 可能超 256）。")
    lines.append("#[repr(u16)]")
    lines.append("#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]")
    lines.append("pub enum StatKey {")
    for pascal, wire, _sec, _agg, disc in entries:
        lines.append(f"    {pascal} = {disc},")
    lines.append("}")
    return "\n".join(lines)


def build_impl_block(entries: list[tuple[str, str, str, str, int]]) -> str:
    lines: list[str] = []
    lines.append("impl StatKey {")
    # as_str
    lines.append("    /// 回傳對應的 wire-format 字串（與 Dota 2 modifier property 名稱一致）。")
    lines.append("    pub const fn as_str(self) -> &'static str {")
    lines.append("        match self {")
    for pascal, wire, _sec, _agg, _disc in entries:
        lines.append(f'            StatKey::{pascal} => "{wire}",')
    lines.append("        }")
    lines.append("    }")
    lines.append("")
    # from_str_key
    lines.append("    /// 將 wire-format 字串解析回 `StatKey`，未知值回 `None`。")
    lines.append("    pub fn from_str_key(s: &str) -> Option<StatKey> {")
    lines.append("        match s {")
    for pascal, wire, _sec, _agg, _disc in entries:
        lines.append(f'            "{wire}" => Some(StatKey::{pascal}),')
    lines.append("            _ => None,")
    lines.append("        }")
    lines.append("    }")
    lines.append("")
    # section
    lines.append("    /// 屬於哪個 `StatSection`（影響建築物 / 前端是否套用）。")
    lines.append("    pub const fn section(self) -> StatSection {")
    lines.append("        match self {")
    for pascal, _wire, sec, _agg, _disc in entries:
        lines.append(f"            StatKey::{pascal} => StatSection::{sec},")
    lines.append("        }")
    lines.append("    }")
    lines.append("")
    # aggregation
    lines.append("    /// 對應的 BuffStore 聚合規則。")
    lines.append("    pub const fn aggregation(self) -> Aggregation {")
    lines.append("        match self {")
    for pascal, _wire, _sec, agg, _disc in entries:
        lines.append(f"            StatKey::{pascal} => Aggregation::{agg},")
    lines.append("        }")
    lines.append("    }")
    lines.append("")
    # is_building_excluded
    lines.append("    /// 建築物（IsBuilding component）是否跳過這個 stat key。")
    lines.append("    ///")
    lines.append("    /// 等同 `section() == StatSection::NonBuilding`。取代舊 `BUILDING_EXCLUDED_KEYS`")
    lines.append("    /// 陣列掃描，host 端 UnitStats 在對建築物計算時用這個 method 快速判定。")
    lines.append("    pub const fn is_building_excluded(self) -> bool {")
    lines.append("        matches!(self.section(), StatSection::NonBuilding)")
    lines.append("    }")
    lines.append("}")
    return "\n".join(lines)


def build_all_array(entries: list[tuple[str, str, str, str, int]]) -> str:
    n = len(entries)
    lines: list[str] = []
    lines.append(f"/// 按 discriminant 排序的所有 `StatKey` variant — 供 gen-docs / 遍歷測試使用。")
    lines.append(f"pub const ALL: &[StatKey; {n}] = &[")
    for pascal, _wire, _sec, _agg, _disc in entries:
        lines.append(f"    StatKey::{pascal},")
    lines.append("];")
    return "\n".join(lines)


def build_tests_block(total: int) -> str:
    return f"""#[cfg(test)]
mod tests {{
    use super::{{StatKey, StatSection, Aggregation, ALL}};

    #[test]
    fn all_array_length_is_{total}() {{
        assert_eq!(ALL.len(), {total});
    }}

    #[test]
    fn as_str_to_from_str_roundtrip_all() {{
        for &k in ALL {{
            assert_eq!(
                StatKey::from_str_key(k.as_str()),
                Some(k),
                "round-trip failed for {{:?}}",
                k
            );
        }}
    }}

    #[test]
    fn from_str_unknown_returns_none() {{
        assert_eq!(StatKey::from_str_key("not_a_real_key"), None);
    }}

    #[test]
    fn discriminants_are_sequential() {{
        for (i, &k) in ALL.iter().enumerate() {{
            assert_eq!(k as u16, i as u16, "discriminant mismatch at {{}}", i);
        }}
    }}

    #[test]
    fn section_and_aggregation_spot_checks() {{
        // SECTION 1 通用 + _bonus → SumAdd
        assert_eq!(StatKey::PreattackBonusDamage.section(), StatSection::All);
        assert_eq!(
            StatKey::PreattackBonusDamage.aggregation(),
            Aggregation::SumAdd
        );
        // SECTION 2 NonBuilding + _percentage → SumAddThenMul1Plus
        assert_eq!(
            StatKey::MoveSpeedBonusPercentage.section(),
            StatSection::NonBuilding
        );
        assert_eq!(
            StatKey::MoveSpeedBonusPercentage.aggregation(),
            Aggregation::SumAddThenMul1Plus
        );
        // SECTION 3 Visual
        assert_eq!(StatKey::PreAttack.section(), StatSection::Visual);
        // _multiplier → ProductMult
        assert_eq!(
            StatKey::AttackSpeedMultiplier.aggregation(),
            Aggregation::ProductMult
        );
        // _chance → Chance
        assert_eq!(
            StatKey::CritChance.aggregation(),
            Aggregation::Chance
        );
    }}

    #[test]
    fn new_tower_variants_exist() {{
        // 7 個塔/英雄新 variant 都可以 round-trip
        let new_keys = [
            StatKey::CritChance,
            StatKey::CritBonus,
            StatKey::SplashBonus,
            StatKey::SlowFactorOverride,
            StatKey::SlowDurationBonus,
            StatKey::AttackStunChance,
            StatKey::AttackStunDuration,
        ];
        for k in new_keys {{
            assert_eq!(StatKey::from_str_key(k.as_str()), Some(k));
        }}
    }}

    #[test]
    fn enum_ified_private_payload_keys() {{
        // 4 個前私有 payload key（原本走 sum_add_str 後門）已納入 StatKey
        assert_eq!(StatKey::MoveSpeedBonus.as_str(), "move_speed_bonus");
        assert_eq!(StatKey::DamageTakenBonus.as_str(), "damage_taken_bonus");
        assert_eq!(StatKey::MultiShotVisual.as_str(), "multi_shot_visual");
        assert_eq!(StatKey::DotDamage.as_str(), "dot_damage");
        // Section 分配
        assert_eq!(StatKey::MoveSpeedBonus.section(), StatSection::NonBuilding);
        assert_eq!(StatKey::DamageTakenBonus.section(), StatSection::All);
        assert_eq!(StatKey::MultiShotVisual.section(), StatSection::Visual);
        assert_eq!(StatKey::DotDamage.section(), StatSection::All);
        // Aggregation 都應 SumAdd（override 後）
        for k in [
            StatKey::MoveSpeedBonus,
            StatKey::DamageTakenBonus,
            StatKey::MultiShotVisual,
            StatKey::DotDamage,
        ] {{
            assert_eq!(k.aggregation(), Aggregation::SumAdd, "{{:?}}", k);
        }}
    }}

    #[test]
    fn is_building_excluded_matches_non_building_section() {{
        // Property：`is_building_excluded()` 應等同 `section() == NonBuilding`
        // 取代舊 BUILDING_EXCLUDED_KEYS 陣列掃描。
        for &k in ALL {{
            let expected = matches!(k.section(), StatSection::NonBuilding);
            assert_eq!(
                k.is_building_excluded(),
                expected,
                "variant {{:?}} section={{:?}} but is_building_excluded={{}}",
                k, k.section(), k.is_building_excluded()
            );
        }}
    }}
}}
"""


def main() -> int:
    if not BACKUP_PATH.exists():
        print(f"ERROR: backup not found at {BACKUP_PATH}", file=sys.stderr)
        return 2
    text = BACKUP_PATH.read_text(encoding="utf-8")

    raw = parse_backup(text)
    name_to_wire = {name: wire for (name, wire, _s) in raw}
    excluded_wires = parse_excluded_wires(text, name_to_wire)

    # Build migrated entries
    entries: list[tuple[str, str, str, str, int]] = []
    used_pascal: dict[str, str] = {}
    disc = 0
    section_num_to_name = {1: "All", 2: "NonBuilding", 3: "Visual"}
    for name, wire, sec_num in raw:
        pascal = to_pascal(name)
        if pascal in used_pascal:
            raise RuntimeError(
                f"Pascal collision: {name!r} -> {pascal!r} (already from {used_pascal[pascal]!r})"
            )
        used_pascal[pascal] = name
        sec = section_num_to_name[sec_num]
        # Override if listed in BUILDING_EXCLUDED_KEYS and currently All
        if wire in excluded_wires and sec == "All":
            sec = "NonBuilding"
        agg = AGGREGATION_OVERRIDES.get(wire, classify_aggregation(wire))
        entries.append((pascal, wire, sec, agg, disc))
        disc += 1

    migrated_count = len(entries)

    # Append new variants
    for pascal, wire, sec, agg in NEW_VARIANTS:
        if pascal in used_pascal:
            # e.g. CritBonus / SplashBonus / SlowFactorOverride / SlowDurationBonus
            # already exist in the old const list → skip (treat new 7 as
            # "ensure these exist" rather than "append duplicates"). Log.
            print(
                f"note: new variant {pascal!r} ({wire!r}) already present from "
                f"const {used_pascal[pascal]!r}; skipping append.",
                file=sys.stderr,
            )
            continue
        used_pascal[pascal] = "(new)"
        entries.append((pascal, wire, sec, agg, disc))
        disc += 1

    total = len(entries)
    new_appended = total - migrated_count

    # Emit
    parts: list[str] = [FILE_HEADER]
    parts.append(build_enum_block(entries))
    parts.append("")
    parts.append(build_impl_block(entries))
    parts.append("")
    parts.append(build_all_array(entries))
    parts.append("")
    parts.append(build_tests_block(total))

    OUTPUT_PATH.write_text("\n".join(parts), encoding="utf-8")

    print(
        f"Generated {total} variants "
        f"({migrated_count} migrated + {new_appended} new)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
