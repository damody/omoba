//! `UnitStats` — 集中「最終屬性」計算 helper。
//!
//! Dota 2 modifier property 系統的 host 端實裝：
//! 所有 tick 系統（creep_tick / hero_tick / tower_tick / damage pipeline）
//! 統一透過這裡取最終數值，避免各自呼 `BuffStore::sum_add` 造成 key 拼寫分歧。
//!
//! 建築物識別：`IsBuilding` component 存在 → 跳過 movespeed / respawn / vision
//! / illusion / bounty 類屬性聚合。
//!
//! 讀 base value：若 entity 有 component 欄位就用那個當 base（TAttack.atk_physic
//! 為 atk base、CProperty.msd 為 move base 等）；其餘未內建欄位的屬性（crit /
//! armor / magic_resist 等）由 spawn 腳本 `on_spawn` 打 duration=∞ 的 base_stats
//! buff 提供基底。

use omb_script_abi::stat_keys::StatKey;
use omb_script_abi::types::DamageKind;
use omoba_sim::Fixed64;
use specs::Entity;

use super::BuffStore;

/// Settlement for an already-authored `Outcome::Damage` packet. Outgoing
/// attack modifiers and accuracy belong to packet creation, not settlement.
/// Keep the fixed-point sum-before-multiply order used by authority; this is
/// deliberately not the separate armor/block `DamageInstance` pipeline.
/// Prediction callers must supply a legitimately disclosed incoming bonus;
/// absence of that observation does not mean a zero bonus.
pub fn settle_damage_packet(
    physical: Fixed64,
    magical: Fixed64,
    pure: Fixed64,
    damage_taken_bonus: Fixed64,
) -> Fixed64 {
    (physical + magical + pure)
        * (Fixed64::ONE + damage_taken_bonus).max(Fixed64::ZERO)
}

/// 單位統計聚合上下文的每個刻度快照。
/// 建置一次，查詢 N 次－便宜（僅儲存引用）。
pub struct UnitStats<'a> {
    pub buffs: &'a BuffStore,
    pub is_building: bool,
}

impl<'a> UnitStats<'a> {
    /// 建一個 `UnitStats`。呼叫端需自行取 `BuffStore` 和 `IsBuilding` 的 borrow，
    /// 再傳進來 — 這樣符合 specs `SystemData` 的 resource lock 流程，
    /// 避免在 System 內部再 `read_resource` 衝突。
    ///
    /// 典型用法（System run 裡）：
    /// 『忽略
    /// 讓 stats = UnitStats::from_refs(&*buffs, is_buildings.get(e).is_some());
    /// 讓 msd = stats.final_move_speed(cp.msd, e);
    /// ```
    pub fn from_refs(buffs: &'a BuffStore, is_building: bool) -> Self {
        Self { buffs, is_building }
    }

    // ================= 位移 =================

    pub fn final_move_speed(&self, base: Fixed64, e: Entity) -> Fixed64 {
        if self.is_building {
            return Fixed64::ZERO;
        }
        if !self.buffs.has_any(e) {
            return base;
        }
        let sums = self.buffs.move_speed_sums(e);
        let effective = if sums.absolute > Fixed64::ZERO {
            sums.absolute
        } else {
            let base_eff = if sums.base_override > Fixed64::ZERO {
                sums.base_override
            } else {
                base
            };
            // Equipment flat（boots、靴類道具）：跟 base 一起被 percentage 縮放
            let bonus_c = sums.bonus_equipment;
            // Percentage（含 ice tower 用的 MoveSpeedBonus，當 -50% 寫進去）
            let pct = sums.percentage;
            // Buff flat post-percentage：不被 slow 削弱、不疊到 base/equipment 上
            let buff_bonus = sums.bonus_buff;
            (base_eff + bonus_c) * (Fixed64::ONE + pct) + buff_bonus
        };
        self.apply_move_clamp_sums(effective, sums)
    }

    fn apply_move_clamp_sums(&self, v: Fixed64, sums: super::buff_store::MoveSpeedSums) -> Fixed64 {
        let min_abs = sums.absolute_min;
        let max_abs = sums.max;
        let limit = sums.limit;
        let mut r = v;
        if min_abs > Fixed64::ZERO && r < min_abs {
            r = min_abs;
        }
        if max_abs > Fixed64::ZERO && r > max_abs {
            r = max_abs;
        }
        if limit > Fixed64::ZERO && r > limit {
            r = limit;
        }
        if r < Fixed64::ZERO {
            Fixed64::ZERO
        } else {
            r
        }
    }

    pub fn turn_rate_mult(&self, e: Entity) -> Fixed64 {
        if self.is_building {
            return Fixed64::ONE;
        }
        Fixed64::ONE + self.buffs.sum_add(e, StatKey::TurnRatePercentage)
    }

    // ================= 攻擊 =================

    pub fn final_atk(&self, base: Fixed64, e: Entity) -> Fixed64 {
        let bonus = self.buffs.sum_add(e, StatKey::PreattackBonusDamage)
            + self.buffs.sum_add(e, StatKey::BaseAttackBonusDamage);
        let pct_total = self
            .buffs
            .sum_add(e, StatKey::TotalDamageOutgoingPercentage);
        let pct_base = self.buffs.sum_add(e, StatKey::BaseDamageOutgoingPercentage)
            + self
                .buffs
                .sum_add(e, StatKey::BaseDamageOutgoingPercentageUnique);
        let mult = Fixed64::ONE + pct_total + pct_base;
        let v = (base + bonus) * mult;
        if v < Fixed64::ZERO {
            Fixed64::ZERO
        } else {
            v
        }
    }

    /// Physical packet before accuracy and target settlement. The armed bonus
    /// is added AFTER outgoing modifiers, matching normal projectile launch.
    /// This is a read-only observation, not consumption or an impact guarantee.
    pub fn normal_attack_physical(&self, base: Fixed64, e: Entity) -> Fixed64 {
        self.final_atk(base, e) + self.buffs.next_attack_bonus(e)
    }

    /// 攻速倍數（乘到 base attack interval 上）。
    /// Dota：每秒有效攻擊數 = 基礎 × (1 + as_bonus / 100)
    /// 簡化：以 bonus/100 當 multiplier 加成；fixed_attack_rate 若設則覆蓋。
    /// 另疊 `ATTACK_SPEED_MULTIPLIER`（專案自訂 product_mult，tower upgrade 用）。
    pub fn final_attack_speed_mult(&self, e: Entity) -> Fixed64 {
        let fixed = self.buffs.sum_add(e, StatKey::FixedAttackRate);
        if fixed > Fixed64::ZERO {
            return fixed;
        }
        let as_bonus = self.buffs.sum_add(e, StatKey::AttackSpeedBonusConstant);
        let hundred = Fixed64::from_i32(100);
        let one_tenth = Fixed64::from_raw(102); // 0.1 in Q22.10 (102/1024 ≈ 0.0996)
        let constant_mult_raw = Fixed64::ONE + as_bonus / hundred;
        let constant_mult = if constant_mult_raw < one_tenth {
            one_tenth
        } else {
            constant_mult_raw
        };
        let extra_mult = self.buffs.product_mult(e, StatKey::AttackSpeedMultiplier);
        let v = constant_mult * extra_mult;
        if v < one_tenth {
            one_tenth
        } else {
            v
        }
    }

    /// 射程 = base + ATTACK_RANGE_BONUS + ATTACK_RANGE_BONUS_UNIQUE，
    /// 再由 MAX_ATTACK_RANGE 上限（若設）。
    pub fn final_attack_range(&self, base: Fixed64, e: Entity) -> Fixed64 {
        let bonus = self.buffs.sum_add(e, StatKey::AttackRangeBonus)
            + self.buffs.sum_add(e, StatKey::AttackRangeBonusUnique);
        let raw = base + bonus;
        let r = if raw < Fixed64::ZERO {
            Fixed64::ZERO
        } else {
            raw
        };
        let max = self.buffs.sum_add(e, StatKey::MaxAttackRange);
        if max > Fixed64::ZERO && r > max {
            max
        } else {
            r
        }
    }

    pub fn final_cast_range(&self, base: Fixed64, e: Entity) -> Fixed64 {
        let v = base
            + self.buffs.sum_add(e, StatKey::CastRangeBonus)
            + self.buffs.sum_add(e, StatKey::CastRangeBonusStacking);
        if v < Fixed64::ZERO {
            Fixed64::ZERO
        } else {
            v
        }
    }

    // ================= 防禦 =================

    pub fn final_armor(&self, base: Fixed64, e: Entity) -> Fixed64 {
        base + self.buffs.sum_add(e, StatKey::PhysicalArmorBonus)
            + self.buffs.sum_add(e, StatKey::PhysicalArmorBonusUnique)
            + self
                .buffs
                .sum_add(e, StatKey::PhysicalArmorBonusUniqueActive)
    }

    /// 魔抗：0..1 = 百分比。direct_modification 若存在 → 覆蓋 base + bonus。
    pub fn final_magic_resist(&self, base: Fixed64, e: Entity) -> Fixed64 {
        let direct = self
            .buffs
            .sum_add(e, StatKey::MagicalResistanceDirectModification);
        if direct > Fixed64::ZERO {
            return clamp_fx(direct, Fixed64::ZERO, Fixed64::ONE);
        }
        let bonus = self.buffs.sum_add(e, StatKey::MagicalResistanceBonus);
        let decrepify = self
            .buffs
            .sum_add(e, StatKey::MagicalResistanceDecrepifyUnique);
        let hundred = Fixed64::from_i32(100);
        // Dota 疊加公式：1 - (1-r1)(1-r2)...
        let combined = Fixed64::ONE
            - (Fixed64::ONE - base)
                * (Fixed64::ONE - bonus / hundred)
                * (Fixed64::ONE - decrepify / hundred);
        clamp_fx(combined, Fixed64::ZERO - Fixed64::ONE, Fixed64::ONE)
    }

    // ================= 命中率 =================

    pub fn evasion_chance(&self, e: Entity) -> Fixed64 {
        let v = self.buffs.sum_add(e, StatKey::EvasionConstant)
            - self.buffs.sum_add(e, StatKey::NegativeEvasionConstant);
        clamp_fx(v, Fixed64::ZERO, Fixed64::ONE)
    }

    pub fn miss_chance(&self, e: Entity) -> Fixed64 {
        clamp_fx(
            self.buffs.sum_add(e, StatKey::MissPercentage),
            Fixed64::ZERO,
            Fixed64::ONE,
        )
    }

    /// 回 (chance, multiplier)；chance 為 0..1；multiplier 預設 1.0（無暴擊）
    pub fn crit(&self, e: Entity) -> (Fixed64, Fixed64) {
        let chance = clamp_fx(
            self.buffs.sum_add(e, StatKey::PreattackCriticalStrike),
            Fixed64::ZERO,
            Fixed64::ONE,
        );
        let mult_raw = self.buffs.sum_add(e, StatKey::CritMultiplier);
        let mult = if mult_raw > Fixed64::ZERO {
            mult_raw
        } else {
            Fixed64::ONE
        };
        (chance, mult)
    }

    // ================= CD / 施法 =================

    /// 冷卻百分比乘數：final_cd = base_cd × (1 + pct + 疊加)
    pub fn cooldown_mult(&self, e: Entity) -> Fixed64 {
        let pct = self.buffs.sum_add(e, StatKey::CooldownPercentage);
        let stacking = self.buffs.sum_add(e, StatKey::CooldownPercentageStacking);
        let one_tenth = Fixed64::from_raw(102);
        let v = Fixed64::ONE + pct + stacking;
        if v < one_tenth {
            one_tenth
        } else {
            v
        }
    }

    pub fn cast_time_mult(&self, e: Entity) -> Fixed64 {
        let one_tenth = Fixed64::from_raw(102);
        let v = Fixed64::ONE + self.buffs.sum_add(e, StatKey::CastTimePercentage);
        if v < one_tenth {
            one_tenth
        } else {
            v
        }
    }

    pub fn mana_cost_mult(&self, e: Entity) -> Fixed64 {
        let v = Fixed64::ONE + self.buffs.sum_add(e, StatKey::ManaCostPercentage);
        if v < Fixed64::ZERO {
            Fixed64::ZERO
        } else {
            v
        }
    }

    // ================= 回復 =================

    pub fn hp_regen(&self, base: Fixed64, e: Entity) -> Fixed64 {
        let half = Fixed64::from_raw(512); // 0.5 in Q22.10
        if self.buffs.has(e, StatKey::DisableHealing.as_str())
            || self.buffs.sum_add(e, StatKey::DisableHealing) > half
        {
            return Fixed64::ZERO;
        }
        let bonus = self.buffs.sum_add(e, StatKey::HealthRegenConstant);
        let pct = self.buffs.sum_add(e, StatKey::HealthRegenPercentage);
        let amp = Fixed64::ONE + self.buffs.sum_add(e, StatKey::HpRegenAmplifyPercentage);
        let v = (base + bonus) * (Fixed64::ONE + pct) * amp;
        if v < Fixed64::ZERO {
            Fixed64::ZERO
        } else {
            v
        }
    }

    pub fn mana_regen(&self, base: Fixed64, e: Entity) -> Fixed64 {
        self.checked_mana_regen(base,e).unwrap_or(Fixed64::ZERO)
    }

    /// Natural recovery only; protected home recovery is added by the match.
    /// Clamp each factor before multiplication: two negative percentages must
    /// never produce positive recovery. Overflow rejects, never wraps/refills.
    pub fn checked_mana_regen(&self, base: Fixed64, e: Entity) -> Option<Fixed64> {
        self.checked_mana_regen_with_flat_bonus(base,e,Fixed64::ZERO)
    }

    /// Read-only prediction for a new, additive, non-family flat regen buff.
    /// Same clamping and rounding as authority recovery; no mutation or refill.
    pub fn checked_mana_regen_with_flat_bonus(&self,base:Fixed64,e:Entity,bonus:Fixed64) -> Option<Fixed64> {
        if base<Fixed64::ZERO {return None;}
        let sum=|key|self.buffs.checked_sum_add(e,key).map(|v|i128::from(v.raw()));
        let base_override = sum(StatKey::BaseManaRegen)?;
        let base_eff = if base_override > 0 {
            base_override
        } else {
            i128::from(base.raw())
        };
        let flat=(base_eff+sum(StatKey::ManaRegenConstant)?+sum(StatKey::ManaRegenConstantUnique)?
            +i128::from(bonus.raw())).max(0);
        let pct=(1024+sum(StatKey::ManaRegenPercentage)?).max(0);
        let total_pct=(1024+sum(StatKey::ManaRegenTotalPercentage)?).max(0);
        let first=flat.checked_mul(pct)?/1024;
        let raw=first.checked_mul(total_pct)?/1024;
        Some(Fixed64::from_raw(i64::try_from(raw).ok()?))
    }

    // ================= HP / Mana 上限 =================

    pub fn max_hp_bonus(&self, e: Entity) -> Fixed64 {
        self.buffs.sum_add(e, StatKey::HealthBonus)
            + self.buffs.sum_add(e, StatKey::ExtraHealthBonus)
    }

    pub fn max_mp_bonus(&self, e: Entity) -> Fixed64 {
        self.buffs.sum_add(e, StatKey::ManaBonus) + self.buffs.sum_add(e, StatKey::ExtraManaBonus)
    }

    /// Authored managed capacity plus flat buffs. Growth does not restore mana;
    /// the pool applies that rule. Invalid modifiers reject instead of granting
    /// an enormous capacity. Negative modifiers may reduce capacity to zero.
    pub fn checked_mana_capacity(&self, base:Fixed64, e:Entity) -> Option<Fixed64> {
        let limit=1_000_000i128*1024;
        if base<Fixed64::ZERO || i128::from(base.raw())>limit {return None;}
        let raw=i128::from(base.raw())
            + i128::from(self.buffs.checked_sum_add(e,StatKey::ManaBonus)?.raw())
            + i128::from(self.buffs.checked_sum_add(e,StatKey::ExtraManaBonus)?.raw());
        if raw>limit {return None;}
        Some(Fixed64::from_raw(raw.max(0) as i64))
    }

    // ================= Damage pipeline 入口 =================

    /// Exact authority packet settlement, shared by normal HP and TD layers.
    pub fn incoming_damage_packet(
        &self,
        physical: Fixed64,
        magical: Fixed64,
        pure: Fixed64,
        victim: Entity,
    ) -> Fixed64 {
        settle_damage_packet(physical, magical, pure,
            self.buffs.sum_add(victim, StatKey::DamageTakenBonus))
    }

    /// 計算 `e`（victim）承受 `raw` damage 後的 final 值（含 block / armor / resist / prevention）。
    /// NOTE: evasion / miss 由呼叫端先 roll，此函式假設攻擊已命中。
    /// Phase 1c.3: damage / armor / resist 仍為 f32（damage pipeline 完整 Fixed64 化是 1c.4）。
    /// 內部 sum_add 回 Fixed64 → 在此 boundary 暫時 to_f32_for_render。
    pub fn apply_incoming_damage(
        &self,
        raw: f32,
        kind: DamageKind,
        e: Entity,
        base_armor: f32,
        base_resist: f32,
    ) -> f32 {
        let half_fx = Fixed64::from_raw(512); // 0.5 in Q22.10
                                              // 1. 絕對免疫
        match kind {
            DamageKind::Physical
                if self.buffs.sum_add(e, StatKey::AbsoluteNoDamagePhysical) > half_fx =>
            {
                return 0.0
            }
            DamageKind::Magical
                if self.buffs.sum_add(e, StatKey::AbsoluteNoDamageMagical) > half_fx =>
            {
                return 0.0
            }
            DamageKind::Pure if self.buffs.sum_add(e, StatKey::AbsoluteNoDamagePure) > half_fx => {
                return 0.0
            }
            _ => {}
        }

        // 2. Block（無法避免、先套）
        let unavoid_block = self
            .buffs
            .sum_add(e, StatKey::TotalConstantBlockUnavoidablePreArmor)
            .to_f32_for_render();
        let after_unavoid = (raw - unavoid_block).max(0.0);

        // 3. 護甲/抵抗
        let after_defense = match kind {
            DamageKind::Physical => {
                let armor = self
                    .final_armor(Fixed64::from_raw((base_armor * 1024.0) as i64), e)
                    .to_f32_for_render();
                after_unavoid * armor_to_mult(armor)
            }
            DamageKind::Magical => {
                let resist = self
                    .final_magic_resist(Fixed64::from_raw((base_resist * 1024.0) as i64), e)
                    .to_f32_for_render();
                after_unavoid * (1.0 - resist)
            }
            DamageKind::Pure => after_unavoid,
        };

        // 4. 類型 block（post-armor）
        let kind_block = self
            .buffs
            .sum_add(
                e,
                match kind {
                    DamageKind::Physical => StatKey::PhysicalConstantBlock,
                    DamageKind::Magical => StatKey::MagicalConstantBlock,
                    _ => StatKey::TotalConstantBlock,
                },
            )
            .to_f32_for_render();
        let after_kind_block = (after_defense - kind_block).max(0.0);

        // 5. 收入百分比
        let pct_all = 1.0
            + self
                .buffs
                .sum_add(e, StatKey::IncomingDamagePercentage)
                .to_f32_for_render();
        let pct_kind = 1.0
            + match kind {
                DamageKind::Physical => self
                    .buffs
                    .sum_add(e, StatKey::IncomingPhysicalDamagePercentage)
                    .to_f32_for_render(),
                _ => 0.0,
            };
        let after_pct = after_kind_block * pct_all * pct_kind;

        // 6.傳入常數
        let k_const = match kind {
            DamageKind::Physical => self
                .buffs
                .sum_add(e, StatKey::IncomingPhysicalDamageConstant)
                .to_f32_for_render(),
            DamageKind::Magical => self
                .buffs
                .sum_add(e, StatKey::IncomingSpellDamageConstant)
                .to_f32_for_render(),
            _ => 0.0,
        };
        (after_pct + k_const).max(0.0)
    }
}

/// Helper：將固定 64 固定到 [min, max]（無 f32 繞行）。
#[inline]
fn clamp_fx(v: Fixed64, lo: Fixed64, hi: Fixed64) -> Fixed64 {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

/// Dota護甲→傷害倍增。
/// armor > 0 → 減傷；armor < 0 → 增傷；armor = 0 → 1.0。
/// 公式：`1 - (0.06 * armor) / (1 + 0.06 * |armor|)`
pub fn armor_to_mult(armor: f32) -> f32 {
    let abs = armor.abs();
    let k = 0.06 * abs;
    if armor >= 0.0 {
        1.0 - (0.06 * armor) / (1.0 + k)
    } else {
        1.0 + (0.06 * abs) / (1.0 + k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn normal_attack_physical_includes_armed_bonus_after_modifiers_without_consuming() {
        let mut world = World::new();
        let owner = world.create_entity().build();
        let other = world.create_entity().build();
        let mut buffs = BuffStore::new();
        buffs.add(owner, "outgoing", fx_huge(), json!({
            (StatKey::PreattackBonusDamage.as_str()): 10 * 1024,
            (StatKey::TotalDamageOutgoingPercentage.as_str()): 512,
        }));
        assert!(buffs.arm_next_attack_bonus(owner, Fixed64::from_i32(60)));
        let stats = UnitStats::from_refs(&buffs, false);
        for _ in 0..2 {
            assert_eq!(stats.normal_attack_physical(Fixed64::from_i32(20), owner),
                Fixed64::from_i32(105));
            assert_eq!(stats.normal_attack_physical(Fixed64::from_i32(20), other),
                Fixed64::from_i32(20));
        }
        assert_eq!(buffs.next_attack_bonus(owner), Fixed64::from_i32(60));
        buffs.consume_next_attack_bonus(owner);
        assert_eq!(UnitStats::from_refs(&buffs, false).normal_attack_physical(
            Fixed64::from_i32(20), owner), Fixed64::from_i32(45));
    }

    #[test]
    fn damage_packet_settlement_preserves_sum_rounding_and_incoming_bonus() {
        for (packet, bonus, expected) in [
            ([1024, 2048, 3072], 0, 6144),
            ([1024, 2048, 3072], 512, 9216),
            ([1024, 2048, 3072], -512, 3072),
            ([1024, 2048, 3072], -1024, 0),
            ([1024, 2048, 3072], -2048, 0),
            // Rounding each channel first would incorrectly produce zero.
            ([1, 1, 1], -512, 1),
            ([0, 0, 0], 1024, 0),
        ] {
            assert_eq!(settle_damage_packet(
                Fixed64::from_raw(packet[0]), Fixed64::from_raw(packet[1]),
                Fixed64::from_raw(packet[2]), Fixed64::from_raw(bonus)).raw(), expected);
        }
        let mut world = World::new();
        let victim = world.create_entity().build();
        let mut buffs = BuffStore::new();
        buffs.add(victim, "incoming", fx_huge(), json!({"damage_taken_bonus":512}));
        let stats = UnitStats::from_refs(&buffs, false);
        assert_eq!(stats.incoming_damage_packet(Fixed64::from_i32(80),
            Fixed64::ZERO, Fixed64::ZERO, victim), Fixed64::from_i32(120));
    }

    fn fx_secs(seconds: f32) -> Fixed64 {
        Fixed64::from_raw((seconds * 1024.0) as i64)
    }

    fn fx_huge() -> Fixed64 {
        // 代表「無窮大」——足夠大，以至於測試視窗中的蜱蟲衰減不會達到 0。
        Fixed64::from_i32(1_000_000)
    }

    #[test]
    fn mana_regeneration_prediction_reuses_clamping_multipliers_and_does_not_mutate() {
        let mut world=World::new();let entity=world.create_entity().build();let mut buffs=BuffStore::new();
        buffs.add(entity,"fixture",fx_huge(),json!({"mana_regen_constant":-6*1024,
            "mana_regen_percentage":512,"mana_regen_total_percentage":1024}));
        let stats=UnitStats::from_refs(&buffs,false);
        assert_eq!(stats.checked_mana_regen(Fixed64::from_i32(5),entity),Some(Fixed64::ZERO));
        assert_eq!(stats.checked_mana_regen_with_flat_bonus(Fixed64::from_i32(5),entity,Fixed64::from_i32(2)),
            Some(Fixed64::from_i32(3)),"flat clamp must happen after the hypothetical additive bonus");
        assert_eq!(stats.checked_mana_regen(Fixed64::from_i32(5),entity),Some(Fixed64::ZERO));
        assert_eq!(buffs.len(),1);
        buffs.add(entity,"broken",fx_huge(),json!({"mana_regen_constant":"invalid"}));
        assert!(UnitStats::from_refs(&buffs,false).checked_mana_regen_with_flat_bonus(
            Fixed64::from_i32(5),entity,Fixed64::from_i32(2)).is_none());
    }

    #[test]
    fn mana_capacity_buffs_are_checked_family_aware_and_never_invent_a_balance() {
        let mut world=World::new();let entity=world.create_entity().build();
        let base=Fixed64::from_i32(280);
        let mut buffs=BuffStore::new();
        buffs.add(entity,"first",fx_huge(),json!({"mana_bonus":100*1024,"__aggregation_family":"capacity"}));
        buffs.add(entity,"second",fx_huge(),json!({"mana_bonus":50*1024,"__aggregation_family":"capacity"}));
        buffs.add(entity,"extra",fx_huge(),json!({"extra_mana_bonus":20*1024}));
        assert_eq!(UnitStats::from_refs(&buffs,false).checked_mana_capacity(base,entity),Some(Fixed64::from_i32(400)));
        buffs.remove(entity,"first");
        assert_eq!(UnitStats::from_refs(&buffs,false).checked_mana_capacity(base,entity),Some(Fixed64::from_i32(350)));
        buffs.add(entity,"negative",fx_huge(),json!({"mana_bonus":-1_000_000*1024i64}));
        assert_eq!(UnitStats::from_refs(&buffs,false).checked_mana_capacity(base,entity),Some(Fixed64::ZERO));
        buffs.remove(entity,"negative");
        for invalid in [json!(i64::MAX),json!("invalid"),json!(1_000_001*1024i64)] {
            buffs.add(entity,"invalid",fx_huge(),json!({"mana_bonus":invalid}));
            assert!(UnitStats::from_refs(&buffs,false).checked_mana_capacity(base,entity).is_none());
        }
    }

    #[test]
    fn mana_buff_recovery_composes_and_clamps_each_factor_without_overflow() {
        let mut world=World::new();let entity=world.create_entity().build();
        let mut buffs=BuffStore::new();
        buffs.add(entity,"regen",fx_huge(),json!({
            "base_mana_regen":10*1024,"mana_regen_constant":3*1024,
            "mana_regen_constant_unique":2*1024,"mana_regen_percentage":512,
            "mana_regen_total_percentage":1024,
        }));
        assert_eq!(UnitStats::from_refs(&buffs,false).checked_mana_regen(Fixed64::from_i32(5),entity),Some(Fixed64::from_i32(45)));
        buffs.add(entity,"regen",fx_huge(),json!({
            "mana_regen_percentage":-2*1024,"mana_regen_total_percentage":-2*1024,
        }));
        assert_eq!(UnitStats::from_refs(&buffs,false).mana_regen(Fixed64::from_i32(5),entity),Fixed64::ZERO);
        buffs.add(entity,"regen",fx_huge(),json!({"mana_regen_constant":i64::MAX,"mana_regen_percentage":i64::MAX,
            "mana_regen_total_percentage":i64::MAX}));
        assert!(UnitStats::from_refs(&buffs,false).checked_mana_regen(Fixed64::from_i32(5),entity).is_none());
        assert_eq!(UnitStats::from_refs(&buffs,false).mana_regen(Fixed64::from_i32(5),entity),Fixed64::ZERO);
        buffs.add(entity,"regen",fx_huge(),json!({"mana_regen_constant":"invalid"}));
        assert!(UnitStats::from_refs(&buffs,false).checked_mana_regen(Fixed64::from_i32(5),entity).is_none());
    }

    #[test]
    fn mana_buff_checked_aggregation_preserves_family_and_order_independent_cancellation() {
        let mut world=World::new();let entity=world.create_entity().build();
        for reversed in [false,true] {
            let mut buffs=BuffStore::new();
            let mut values=vec![i64::MAX,i64::MAX,-i64::MAX];
            if reversed {values.reverse();}
            for (index,value) in values.into_iter().enumerate() {
                buffs.add(entity,&format!("source{index}"),fx_huge(),json!({"mana_regen_constant":value}));
            }
            assert_eq!(buffs.checked_sum_add(entity,StatKey::ManaRegenConstant),Some(Fixed64::from_raw(i64::MAX)));
        }
        let mut buffs=BuffStore::new();
        buffs.add(entity,"weak",fx_huge(),json!({"__aggregation_family":"mana","mana_regen_constant":3*1024}));
        buffs.add(entity,"strong",fx_huge(),json!({"__aggregation_family":"mana","mana_regen_constant":7*1024}));
        assert_eq!(UnitStats::from_refs(&buffs,false).mana_regen(Fixed64::from_i32(5),entity),Fixed64::from_i32(12));
        buffs.remove(entity,"strong");
        assert_eq!(UnitStats::from_refs(&buffs,false).mana_regen(Fixed64::from_i32(5),entity),Fixed64::from_i32(8));
        buffs.add(entity,"min",fx_huge(),json!({"__aggregation_family":"mana","mana_regen_constant":i64::MIN}));
        assert_eq!(buffs.checked_sum_add(entity,StatKey::ManaRegenConstant),Some(Fixed64::from_raw(i64::MIN)));
    }

    #[test]
    // Regression: ice tower percentage slow must affect authoritative movement.
    fn move_speed_bonus_applies_as_percentage_slow() {
        let mut world = World::new();
        let e = world.create_entity().build();
        let mut store = BuffStore::new();
        store.add(
            e,
            "slow_test",
            fx_secs(2.0),
            json!({ StatKey::MoveSpeedBonus.as_str(): -0.5 }),
        );
        let stats = UnitStats::from_refs(&store, false);
        let effective = stats
            .final_move_speed(Fixed64::from_i32(100), e)
            .to_f32_for_render();
        assert!(
            (effective - 50.0).abs() < 1.0,
            "expected ~50.0, got {}",
            effective
        );
    }

    // Dota 順序：equipment bonus（boots 等）跟 base 一起被 percentage 縮放。
    // 基本=300、靴子+90、緩慢-50% → (300+90)*0.5 = 195。
    #[test]
    fn move_speed_equipment_bonus_scales_with_percentage() {
        let mut world = World::new();
        let e = world.create_entity().build();
        let mut store = BuffStore::new();
        store.add(
            e,
            "boots",
            fx_huge(),
            json!({ StatKey::MoveSpeedBonusEquipment.as_str(): 90.0 }),
        );
        store.add(
            e,
            "slow_ice",
            fx_secs(2.0),
            json!({ StatKey::MoveSpeedBonus.as_str(): -0.5 }),
        );
        let stats = UnitStats::from_refs(&store, false);
        let effective = stats
            .final_move_speed(Fixed64::from_i32(300), e)
            .to_f32_for_render();
        assert!(
            (effective - 195.0).abs() < 1.0,
            "expected ~195.0, got {}",
            effective
        );
    }

    // Buff flat post-pct：不被 percentage 縮放，純加在最末端。
    // 基本=300、靴子 +90、減速 -50%、增益 +60 → (300+90)*0.5 + 60 = 255。
    #[test]
    fn move_speed_buff_bonus_is_flat_post_percentage() {
        let mut world = World::new();
        let e = world.create_entity().build();
        let mut store = BuffStore::new();
        store.add(
            e,
            "boots",
            fx_huge(),
            json!({ StatKey::MoveSpeedBonusEquipment.as_str(): 90.0 }),
        );
        store.add(
            e,
            "slow_ice",
            fx_secs(2.0),
            json!({ StatKey::MoveSpeedBonus.as_str(): -0.5 }),
        );
        store.add(
            e,
            "haste_buff",
            fx_secs(5.0),
            json!({ StatKey::MoveSpeedBonusBuff.as_str(): 60.0 }),
        );
        let stats = UnitStats::from_refs(&store, false);
        let effective = stats
            .final_move_speed(Fixed64::from_i32(300), e)
            .to_f32_for_render();
        assert!(
            (effective - 255.0).abs() < 1.0,
            "expected ~255.0, got {}",
            effective
        );
    }
}
