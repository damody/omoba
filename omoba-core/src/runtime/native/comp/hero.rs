use crate::runtime::ability_runtime::{ManaPool, ManaPoolError};
use omoba_sim::Fixed64;
use serde::{Deserialize, Serialize};
use specs::storage::VecStorage;
use specs::Component;
use std::collections::HashMap;

/// Hero ability slots: Q, W, E, R, plus the optional extra action slot D (index 4).
/// A hero with only four abilities never touches the extra slot.
pub const HERO_ABILITY_SLOTS: usize = omoba_template_ids::HERO_ABILITY_SLOTS;
pub const LEGACY_HERO_ABILITY_SLOTS: usize = 4;

/// 英雄組件 - 包含英雄的基礎屬性和成長數據
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Hero {
    pub id: String,
    pub name: String,
    pub title: String,
    pub background: String,

    // 基礎屬性
    pub strength: i32,
    pub agility: i32,
    pub intelligence: i32,
    pub primary_attribute: AttributeType,

    // 當前等級和經驗
    pub level: i32,
    pub experience: i32,
    pub experience_to_next: i32,

    // 技能相關
    pub abilities: Vec<String>,               // ability IDs
    pub ability_levels: HashMap<String, i32>, // 技能等級
    pub skill_points: i32,                    // 可用技能點
    /// 技能剩餘冷卻秒數；key = ability_id。>0 表示仍在 CD 中，host 會拒絕 SkillCast。
    /// 不寫進 serde 預設值時自動補空 map，相容舊 savegame。
    #[serde(default)]
    pub ability_cooldowns: HashMap<String, Fixed64>,
    /// Actual duration of each running cooldown; old saves may omit this map.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub ability_cooldown_totals: HashMap<String, Fixed64>,
    /// Explicitly enabled by the authoritative ruleset. Legacy heroes/savegames
    /// remain unsupported rather than silently acquiring a full mana balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mana_pool: Option<ManaPool>,
    /// Stored uses for charge abilities (G-2), keyed by ability id. Absent for
    /// ordinary abilities. Persists and keeps recharging through death.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub ability_charges: std::collections::BTreeMap<String, AbilityChargeState>,

    // 升級數據
    pub level_growth: LevelGrowth,
    /// Max HP the attributes have added to this life's `CProperty.mhp` (attributes.rs). Kept on
    /// the hero so a disclosed hero carries it with its max HP and no replica adds it again.
    /// A new life starts at zero.
    #[serde(default)]
    pub attribute_hp: Fixed64,
}

/// One charge ability. `remaining` counts down to the next restored use and is
/// zero exactly when `charges == max`. `recharge` is the per-use duration of
/// the current rank, snapshotted when a cycle starts.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AbilityChargeState {
    pub charges: u16,
    pub max: u16,
    pub recharge: Fixed64,
    pub remaining: Fixed64,
}

/// Content-declared charge behaviour when a rank raises the maximum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChargeRankUp {
    /// Default: the extra uses are granted at once; a running cycle continues.
    GrantNewUses,
    /// Source B01E (B01.lua:49-59, modifier_charges.lua:15-22): the stored
    /// count is kept and, when below the new maximum, the recharge cycle
    /// restarts at the full per-use time.
    KeepCountRestart,
}

impl ChargeRankUp {
    /// Generated extras key (`custom` effect `charges_rank_up_keep_count`, per rank).
    pub const KEEP_COUNT_KEY: &'static str = "custom_charges_rank_up_keep_count";

    pub fn authored(content: &omoba_template_ids::AbilityConst, rank: u8) -> Self {
        let keep = usize::from(rank).checked_sub(1).and_then(|index|
            content.extras.iter().find(|(name, _)| *name == Self::KEEP_COUNT_KEY)?.1.get(index).copied())
            .is_some_and(|value| value > Fixed64::ZERO);
        if keep { Self::KeepCountRestart } else { Self::GrantNewUses }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum AttributeType {
    Strength,
    Agility,
    Intelligence,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LevelGrowth {
    pub strength_per_level: Fixed64,
    pub agility_per_level: Fixed64,
    pub intelligence_per_level: Fixed64,
    pub damage_per_level: Fixed64,
    pub hp_per_level: Fixed64,
    pub mana_per_level: Fixed64,
}

impl Component for Hero {
    type Storage = VecStorage<Self>;
}

impl Hero {
    /// 創建新的英雄實例
    pub fn new(id: String, name: String, title: String) -> Self {
        Hero {
            id,
            name,
            title,
            background: String::new(),
            // No template: no attributes (attributes.rs would otherwise grant HP, armor, ...).
            strength: 0,
            agility: 0,
            intelligence: 0,
            primary_attribute: AttributeType::Strength,
            level: 1,
            experience: 0,
            experience_to_next: 100,
            abilities: Vec::new(),
            ability_levels: HashMap::new(),
            skill_points: 0,
            ability_cooldowns: HashMap::new(),
            ability_cooldown_totals: HashMap::new(),
            ability_charges: std::collections::BTreeMap::new(),
            mana_pool: None,
            level_growth: LevelGrowth::default(),
            attribute_hp: Fixed64::ZERO,
        }
    }

    /// 從戰役資料創建英雄。
    ///
    /// **單一來源**：所有 intrinsic stats（strength/agility/base_hp/level_growth/...）
    /// 都從 `omoba_template_ids::hero_stats(id)` const lookup 取，
    /// `HeroJD`（generated story）只剩 `id` 拿來查 generated templates。
    pub fn from_campaign_data(hero_data: &crate::runtime::scene::import_campaign::HeroJD) -> Self {
        use omoba_template_ids::{
            active_hero_abilities, active_hero_display, active_hero_stats, active_hero_title,
            hero_by_name,
        };
        let id = hero_by_name(&hero_data.id)
            .unwrap_or_else(|| panic!("hero id '{}' not in generated templates", hero_data.id));
        let s = active_hero_stats(id).unwrap_or_else(|| {
            panic!(
                "hero '{}' has no stats in generated templates",
                hero_data.id
            )
        });
        let primary_attribute = match s.primary_attribute {
            0 => AttributeType::Strength,
            1 => AttributeType::Agility,
            2 => AttributeType::Intelligence,
            _ => AttributeType::Strength,
        };
        let abilities: Vec<String> = active_hero_abilities(id)
            .iter()
            .map(|aid| aid.as_str().to_string())
            .collect();
        let mut ability_levels = HashMap::new();
        for ability_id in &abilities {
            ability_levels.insert(ability_id.clone(), 0);
        }

        Hero {
            id: hero_data.id.clone(),
            name: active_hero_display(id).to_string(),
            title: active_hero_title(id).to_string(),
            background: String::new(), // background 不再放 ECS Hero component（templates.lua 內部用）
            strength: s.strength,
            agility: s.agility,
            intelligence: s.intelligence,
            primary_attribute,
            level: 1,
            experience: 0,
            experience_to_next: 100,
            abilities,
            ability_levels,
            skill_points: 8, // 初始技能點（playtest 方便把所有 ability 點起來）
            ability_cooldowns: HashMap::new(),
            ability_cooldown_totals: HashMap::new(),
            ability_charges: std::collections::BTreeMap::new(),
            mana_pool: None,
            level_growth: LevelGrowth {
                strength_per_level: s.level_growth.strength_per_level,
                agility_per_level: s.level_growth.agility_per_level,
                intelligence_per_level: s.level_growth.intelligence_per_level,
                damage_per_level: s.level_growth.damage_per_level,
                hp_per_level: s.level_growth.hp_per_level,
                mana_per_level: s.level_growth.mana_per_level,
            },
            attribute_hp: Fixed64::ZERO,
        }
    }

    /// 獲取當前總屬性值（基礎 + 等級成長）
    pub fn get_total_strength(&self) -> Fixed64 {
        Fixed64::from_i32(self.strength) + self.level_growth.strength_per_level * (self.level - 1)
    }

    pub fn get_total_agility(&self) -> Fixed64 {
        Fixed64::from_i32(self.agility) + self.level_growth.agility_per_level * (self.level - 1)
    }

    pub fn get_total_intelligence(&self) -> Fixed64 {
        Fixed64::from_i32(self.intelligence)
            + self.level_growth.intelligence_per_level * (self.level - 1)
    }

    /// 獲取主屬性值
    pub fn get_primary_attribute_value(&self) -> Fixed64 {
        match self.primary_attribute {
            AttributeType::Strength => self.get_total_strength(),
            AttributeType::Agility => self.get_total_agility(),
            AttributeType::Intelligence => self.get_total_intelligence(),
        }
    }

    /// 計算基礎攻擊力
    pub fn get_base_damage(&self) -> Fixed64 {
        let base = self.get_primary_attribute_value();
        let level_bonus = self.level_growth.damage_per_level * (self.level - 1);
        base + level_bonus
    }

    /// 計算最大生命值
    pub fn get_max_hp(&self) -> Fixed64 {
        let str_bonus = self.get_total_strength() * Fixed64::from_i32(22); // 每點力量 +22 HP
        let level_bonus = self.level_growth.hp_per_level * (self.level - 1);
        Fixed64::from_i32(200) + str_bonus + level_bonus // 基礎 200 HP
    }

    /// 計算最大法力值
    pub fn get_max_mana(&self) -> Fixed64 {
        let int_bonus = self.get_total_intelligence() * Fixed64::from_i32(13); // 每點智力 +13 MP
        let level_bonus = self.level_growth.mana_per_level * (self.level - 1);
        Fixed64::from_i32(75) + int_bonus + level_bonus // 基礎 75 MP
    }

    /// Authored MOBA capacity, independent of the legacy intelligence formula.
    pub fn moba_mana_capacity(&self) -> Result<Fixed64, ManaPoolError> {
        let stats = omoba_template_ids::hero_by_name(&self.id)
            .and_then(omoba_template_ids::active_hero_stats)
            .ok_or(ManaPoolError::InvalidState)?;
        if !(1..=25).contains(&self.level) {
            return Err(ManaPoolError::InvalidState);
        }
        let raw = i128::from(stats.base_mana) * 1024
            + i128::from(self.level_growth.mana_per_level.raw()) * i128::from(self.level - 1);
        if !(0..=1_000_000 * 1024).contains(&raw) {
            return Err(ManaPoolError::InvalidState);
        }
        Ok(Fixed64::from_raw(raw as i64))
    }

    /// Initialize once; repeated initialization must never refill a spent pool.
    /// Activation is not implicit in GameMode::Moba (legacy stories use it too).
    pub fn initialize_mana_pool(&mut self) -> Result<(), ManaPoolError> {
        if self.mana_pool.is_none() {
            self.mana_pool = Some(ManaPool::full(self.get_max_mana())?);
        }
        Ok(())
    }

    /// Recompute capacity after authoritative stat changes without restoring mana.
    pub fn refresh_mana_capacity(&mut self) -> Result<(), ManaPoolError> {
        let maximum = self.get_max_mana();
        if let Some(pool) = &mut self.mana_pool {
            pool.set_maximum(maximum)?;
        }
        Ok(())
    }

    /// 計算攻擊速度倍數
    pub fn get_attack_speed_multiplier(&self) -> Fixed64 {
        // agi_bonus = total_agility * 0.01；每點敏捷 +1% 攻速
        let agi_bonus = self.get_total_agility() / Fixed64::from_i32(100);
        Fixed64::ONE + agi_bonus
    }

    /// 計算移動速度
    pub fn get_move_speed(&self) -> Fixed64 {
        // agi_bonus_pct = total_agility * 0.05；移速 = 300 * (1 + agi_bonus_pct / 100)
        let agi_bonus_pct = self.get_total_agility() * Fixed64::from_raw(51); // 0.05 ≈ 51/1024
        let mult = Fixed64::ONE + agi_bonus_pct / Fixed64::from_i32(100);
        Fixed64::from_i32(300) * mult
    }

    /// 計算暴擊率
    pub fn get_crit_chance(&self) -> Fixed64 {
        // 雜賀孫市作為敏捷英雄，敏捷提供暴擊率
        // agi_crit = total_agility * 0.001；每點敏捷 +0.1% 暴擊率
        let agi_crit = self.get_total_agility() / Fixed64::from_i32(1000);
        let base_crit = Fixed64::from_raw(51); // ≈ 0.05 (5%)
        let cap = Fixed64::from_raw(768); // 0.75
        let total = base_crit + agi_crit;
        if total > cap {
            cap
        } else {
            total
        }
    }

    /// 計算閃避率
    pub fn get_dodge_chance(&self) -> Fixed64 {
        // 高敏捷英雄有少量閃避率：超過20敏捷後每點 +0.05% 閃避
        let agi = self.get_total_agility();
        let threshold = Fixed64::from_i32(20);
        let excess = if agi > threshold {
            agi - threshold
        } else {
            Fixed64::ZERO
        };
        let agi_dodge = excess * Fixed64::from_raw(1); // ≈ 0.001 (about 0.0005 raw=1; using raw=1 ≈ 0.000976)
        let cap = Fixed64::from_raw(256); // 0.25 (25%)
        if agi_dodge > cap {
            cap
        } else {
            agi_dodge
        }
    }

    /// 增加經驗值
    /// MOBA authority rewards use bounded integer progression; legacy TD is unchanged.
    /// `rules`: the match's game rules (templates/game_rules.lua: the original game's table and
    /// level cap); `None` keeps the legacy 100 x 1.2^(level-1) curve and a cap of 25.
    pub fn add_moba_experience(&mut self, exp: u32, rules: Option<&omoba_template_ids::GameRulesConst>) -> bool {
        let cap = rules.map_or(25, |rules| (rules.experience_max_level as i32).clamp(1, 25));
        if exp == 0 || self.level >= cap { return false; }
        self.experience = self.experience.max(0).saturating_add(exp.min(i32::MAX as u32) as i32);
        let before = self.level;
        // At most 24 iterations, even for a saturated reward. Never use floats.
        while self.level < cap && self.experience >= self.experience_to_next.max(1) {
            self.experience -= self.experience_to_next.max(1);
            self.level += 1;
            self.skill_points = self.skill_points.saturating_add(1);
            self.experience_to_next = Self::moba_experience_to_next(self.level, rules);
        }
        if self.level == cap { self.experience = 0; }
        self.level != before
    }

    /// Experience from `level` to the next level under `rules` (see add_moba_experience).
    pub fn moba_experience_to_next(level: i32, rules: Option<&omoba_template_ids::GameRulesConst>) -> i32 {
        match rules {
            Some(rules) => rules.experience_to_next(level).unwrap_or(i32::MAX),
            None => {
                let exponent = (level - 1).clamp(0, 24) as u32;
                (100u128 * 6u128.pow(exponent) / 5u128.pow(exponent)) as i32
            }
        }
    }

    /// Legacy progression retained for existing TD/script callers.
    pub fn add_experience(&mut self, exp: i32) -> bool {
        self.experience += exp;

        if self.experience >= self.experience_to_next {
            self.level_up();
            true
        } else {
            false
        }
    }

    /// 升級處理
    pub fn level_up(&mut self) {
        if self.level < 25 {
            // 最大等級限制
            self.level += 1;
            self.skill_points += 1;
            self.experience -= self.experience_to_next;

            // 計算下一級所需經驗 (指數增長)
            self.experience_to_next = (100.0 * (1.2_f32.powi(self.level - 1))) as i32;
        }
    }

    /// 學習技能
    pub fn learn_ability(&mut self, ability_id: &str) -> Result<(), String> {
        if self.skill_points <= 0 {
            return Err("No skill points available".to_string());
        }

        if !self.abilities.contains(&ability_id.to_string()) {
            return Err("Hero doesn't have this ability".to_string());
        }

        let current_level = *self.ability_levels.get(ability_id).unwrap_or(&0);
        if current_level >= 4 {
            // 最大技能等級
            return Err("Ability is already at maximum level".to_string());
        }

        // 終極技能等級限制
        if ability_id.starts_with("R") && current_level >= (self.level / 6).max(1) {
            return Err("Hero level too low for ultimate upgrade".to_string());
        }

        self.ability_levels
            .insert(ability_id.to_string(), current_level + 1);
        self.skill_points -= 1;

        Ok(())
    }

    /// 獲取技能等級
    pub fn get_ability_level(&self, ability_id: &str) -> i32 {
        *self.ability_levels.get(ability_id).unwrap_or(&0)
    }

    /// 檢查是否可以使用技能
    pub fn can_use_ability(&self, ability_id: &str) -> bool {
        self.get_ability_level(ability_id) > 0
    }

    /// 該技能是否仍在 CD 中（>0 秒）。
    pub fn is_on_cooldown(&self, ability_id: &str) -> bool {
        self.ability_cooldowns
            .get(ability_id)
            .copied()
            .unwrap_or(Fixed64::ZERO)
            > Fixed64::ZERO
    }

    /// 取該技能剩餘冷卻秒；0 代表可施放。
    pub fn get_cooldown(&self, ability_id: &str) -> Fixed64 {
        let v = self
            .ability_cooldowns
            .get(ability_id)
            .copied()
            .unwrap_or(Fixed64::ZERO);
        if v < Fixed64::ZERO {
            Fixed64::ZERO
        } else {
            v
        }
    }

    /// 啟動技能冷卻。duration <= 0 視為無 CD（直接清除）。
    pub fn start_cooldown(&mut self, ability_id: &str, duration: Fixed64) {
        if duration > Fixed64::ZERO {
            self.ability_cooldowns
                .insert(ability_id.to_string(), duration);
            self.ability_cooldown_totals.insert(ability_id.to_string(), duration);
        } else {
            self.ability_cooldowns.remove(ability_id);
            self.ability_cooldown_totals.remove(ability_id);
        }
    }

    /// Shorten a running cooldown by `by` seconds, keeping its admitted total;
    /// reaching zero ends it. Returns whether anything changed.
    pub fn reduce_cooldown(&mut self, ability_id: &str, by: Fixed64) -> bool {
        let remaining = self.get_cooldown(ability_id);
        if remaining <= Fixed64::ZERO || by <= Fixed64::ZERO { return false; }
        let next = Fixed64::from_raw(remaining.raw().saturating_sub(by.raw()));
        if next > Fixed64::ZERO {
            self.ability_cooldowns.insert(ability_id.to_string(), next);
        } else {
            self.ability_cooldowns.remove(ability_id);
            self.ability_cooldown_totals.remove(ability_id);
        }
        true
    }

    /// Apply an authority-settled absolute remaining. A shorter value for a
    /// running cooldown is a reduction and keeps the admitted total; anything
    /// else (no cooldown running, or a longer value) is a fresh start.
    pub fn commit_cooldown(&mut self, ability_id: &str, remaining: Fixed64) {
        let current = self.get_cooldown(ability_id);
        let total = self.ability_cooldown_totals.get(ability_id).copied();
        if remaining > Fixed64::ZERO && current > Fixed64::ZERO && remaining <= current && total.is_some_and(|t| t >= current) {
            self.ability_cooldowns.insert(ability_id.to_string(), remaining);
        } else {
            self.start_cooldown(ability_id, remaining);
        }
    }

    /// Present a running instance's admitted duration, never today's modifiers.
    /// Legacy saves lack that snapshot: use a conservative base/remaining bound.
    pub fn cooldown_total(&self, ability_id: &str, base: Fixed64) -> Option<Fixed64> {
        let limit = Fixed64::from_i32(1_000_000);
        let remaining = self.ability_cooldowns.get(ability_id).copied().unwrap_or(Fixed64::ZERO);
        if base < Fixed64::ZERO || base > limit || remaining < Fixed64::ZERO || remaining > limit { return None; }
        if remaining == Fixed64::ZERO { return Some(base); }
        match self.ability_cooldown_totals.get(ability_id).copied() {
            Some(total) if total >= remaining && total <= limit => Some(total),
            Some(_) => None,
            None => Some(base.max(remaining)),
        }
    }

    /// 每個 tick 扣 dt；遞減到 0 以下的 entry 自動清除。
    pub fn tick_cooldowns(&mut self, dt: Fixed64) {
        if dt <= Fixed64::ZERO { return; }
        self.ability_cooldowns.retain(|_, remaining| {
            *remaining = Fixed64::from_raw(remaining.raw().saturating_sub(dt.raw()));
            *remaining > Fixed64::ZERO
        });
        self.ability_cooldown_totals.retain(|id, _| self.ability_cooldowns.contains_key(id));
        for state in self.ability_charges.values_mut() {
            if state.charges >= state.max || state.recharge <= Fixed64::ZERO { state.remaining = Fixed64::ZERO; continue; }
            // Carry overshoot into the next cycle so long ticks never lose time.
            state.remaining = Fixed64::from_raw(state.remaining.raw().saturating_sub(dt.raw()));
            while state.remaining <= Fixed64::ZERO && state.charges < state.max {
                state.charges += 1;
                state.remaining = if state.charges < state.max { state.remaining + state.recharge } else { Fixed64::ZERO };
            }
        }
    }

    /// Re-derive every charge ability from its generated rank data. Cheap: four
    /// slots, no allocation unless an ability actually declares charges.
    pub fn sync_authored_charges(&mut self) {
        for slot in 0..self.abilities.len() {
            let rank = self.get_ability_level(&self.abilities[slot]);
            let content = u8::try_from(rank).ok().filter(|rank| *rank > 0).and_then(|rank|
                omoba_template_ids::ability_by_name(&self.abilities[slot])
                    .and_then(omoba_template_ids::active_ability_const).map(|content| (content, rank)));
            let spec = content.and_then(|(content, rank)| content.charges_at(rank));
            let policy = content.map_or(ChargeRankUp::GrantNewUses, |(content, rank)| ChargeRankUp::authored(content, rank));
            if spec.is_some() || self.ability_charges.contains_key(&self.abilities[slot]) {
                let id = self.abilities[slot].clone();
                self.sync_charges_with(&id, spec, policy);
            }
        }
    }

    /// Ready when off cooldown and, for a charge ability, holding a use.
    pub fn cast_ready(&self, ability_id: &str) -> bool {
        !self.is_on_cooldown(ability_id) && self.has_charge(ability_id)
    }

    /// Ordinary abilities always "have" a charge; charge abilities need one stored.
    pub fn has_charge(&self, ability_id: &str) -> bool {
        self.ability_charges.get(ability_id).is_none_or(|state| state.charges > 0)
    }

    /// Spend one stored use; arms the recharge cycle if it was full. False if none.
    pub fn consume_charge(&mut self, ability_id: &str) -> bool {
        let Some(state) = self.ability_charges.get_mut(ability_id) else { return true; };
        if state.charges == 0 { return false; }
        state.charges -= 1;
        if state.remaining <= Fixed64::ZERO && state.charges < state.max { state.remaining = state.recharge; }
        true
    }

    /// Align stored uses with the learned rank. First learn starts full; a rank
    /// that raises the maximum grants the extra uses immediately; a lower
    /// maximum clamps. A new per-use duration applies from the next cycle;
    /// a running cycle never exceeds it. Unlearned (rank 0) drops the state.
    pub fn sync_charges(&mut self, ability_id: &str, spec: Option<(u16, Fixed64)>) {
        self.sync_charges_with(ability_id, spec, ChargeRankUp::GrantNewUses);
    }

    /// [`Self::sync_charges`] with the ability's authored rank-up policy.
    /// `KeepCountRestart`: a higher maximum adds no uses; when not full the
    /// recharge cycle restarts at the full per-use time (source B01E).
    pub fn sync_charges_with(&mut self, ability_id: &str, spec: Option<(u16, Fixed64)>, policy: ChargeRankUp) {
        let Some((max, recharge)) = spec.filter(|(max, recharge)| *max > 0 && *recharge > Fixed64::ZERO) else {
            self.ability_charges.remove(ability_id);
            return;
        };
        let state = self.ability_charges.entry(ability_id.to_owned())
            .or_insert(AbilityChargeState { charges: max, max, recharge, remaining: Fixed64::ZERO });
        let raised = max > state.max;
        if raised && policy == ChargeRankUp::GrantNewUses { state.charges = state.charges.saturating_add(max - state.max); }
        state.max = max;
        state.charges = state.charges.min(max);
        state.recharge = recharge;
        if state.charges >= max {
            state.remaining = Fixed64::ZERO;
        } else if (raised && policy == ChargeRankUp::KeepCountRestart)
            || state.remaining <= Fixed64::ZERO || state.remaining > recharge {
            state.remaining = recharge;
        }
    }
}

impl Default for LevelGrowth {
    fn default() -> Self {
        LevelGrowth {
            strength_per_level: Fixed64::from_raw(2867), // 2.8  (2.8 * 1024 ≈ 2867)
            agility_per_level: Fixed64::from_raw(2458),  // 2.4
            intelligence_per_level: Fixed64::from_i32(2), // 2.0
            damage_per_level: Fixed64::from_raw(2560),   // 2.5
            hp_per_level: Fixed64::from_i32(60),
            mana_per_level: Fixed64::from_i32(26),
        }
    }
}

impl Default for Hero {
    fn default() -> Self {
        Hero::new(
            "unknown".to_string(),
            "Unknown Hero".to_string(),
            "The Nameless".to_string(),
        )
    }
}

#[cfg(test)]
mod moba_progression_tests {
    use super::Hero;
    use omoba_sim::Fixed64;

    #[test]
    fn cooldown_instance_total_survives_tick_refresh_clear_and_expiry() {
        let mut h=Hero::default();let base=Fixed64::from_i32(20);
        h.start_cooldown("a",Fixed64::from_i32(40));h.tick_cooldowns(Fixed64::from_i32(10));
        assert_eq!(h.cooldown_total("a",base),Some(Fixed64::from_i32(40)));
        h.tick_cooldowns(Fixed64::ZERO);h.tick_cooldowns(Fixed64::from_raw(-1));
        assert_eq!(h.get_cooldown("a"),Fixed64::from_i32(30));
        h.start_cooldown("a",Fixed64::from_i32(10));assert_eq!(h.cooldown_total("a",base),Some(Fixed64::from_i32(10)));
        h.start_cooldown("a",Fixed64::ZERO);assert!(h.ability_cooldown_totals.is_empty());
        h.start_cooldown("a",Fixed64::ONE);h.tick_cooldowns(Fixed64::from_raw(i64::MAX));
        assert!(h.ability_cooldowns.is_empty()&&h.ability_cooldown_totals.is_empty());
    }

    #[test]
    fn cooldown_instance_serde_supports_old_saves_and_preserves_admitted_total() {
        let mut h=Hero::default();h.start_cooldown("a",Fixed64::from_i32(40));h.tick_cooldowns(Fixed64::ONE);
        let json=serde_json::to_value(&h).unwrap();let copy:Hero=serde_json::from_value(json.clone()).unwrap();
        assert_eq!(copy.cooldown_total("a",Fixed64::from_i32(20)),Some(Fixed64::from_i32(40)));
        let mut legacy=json;legacy.as_object_mut().unwrap().remove("ability_cooldown_totals");
        let copy:Hero=serde_json::from_value(legacy).unwrap();
        assert_eq!(copy.cooldown_total("a",Fixed64::from_i32(20)),Some(Fixed64::from_i32(39)));
    }

    #[test]
    fn cooldown_instance_total_rejects_invalid_present_bounds_and_ignores_retired_values() {
        let mut h=Hero::default();h.start_cooldown("a",Fixed64::from_i32(40));
        for total in [-1,1,1_024_000_001] {
            h.ability_cooldown_totals.insert("a".into(),Fixed64::from_raw(total));
            assert!(h.cooldown_total("a",Fixed64::from_i32(20)).is_none());
        }
        h.ability_cooldowns.clear();assert_eq!(h.cooldown_total("a",Fixed64::from_i32(20)),Some(Fixed64::from_i32(20)));
        h.tick_cooldowns(Fixed64::ONE);assert!(h.ability_cooldown_totals.is_empty());
    }

    #[test]
    fn integer_moba_xp_crosses_multiple_levels_and_caps_without_overflow() {
        let mut hero = Hero::default();
        assert!(!hero.add_moba_experience(99, None));
        assert_eq!((hero.level, hero.experience, hero.skill_points), (1, 99, 0));
        assert!(hero.add_moba_experience(266, None)); // 100 + 120 + 144; one XP remains.
        assert_eq!((hero.level, hero.experience, hero.experience_to_next, hero.skill_points), (4, 1, 172, 3));
        assert!(hero.add_moba_experience(u32::MAX, None));
        assert_eq!((hero.level, hero.experience, hero.skill_points), (25, 0, 24));
        assert!(!hero.add_moba_experience(u32::MAX, None));
        assert_eq!((hero.level, hero.experience, hero.skill_points), (25, 0, 24));
    }

    #[test]
    fn game_rules_xp_follows_the_original_table() {
        // templates/game_rules.lua: 220, 265, 310, ... to the next level.
        let rules = omoba_template_ids::GAME_RULES.expect("the catalog carries game_rules");
        let mut hero = Hero::default();
        hero.experience_to_next = Hero::moba_experience_to_next(1, Some(&rules));
        assert!(!hero.add_moba_experience(219, Some(&rules)));
        assert!(hero.add_moba_experience(267, Some(&rules))); // 220 + 265; one XP remains.
        assert_eq!((hero.level, hero.experience, hero.experience_to_next, hero.skill_points), (3, 1, 310, 2));
        assert!(hero.add_moba_experience(u32::MAX, Some(&rules)));
        assert_eq!((hero.level, hero.experience), (rules.experience_max_level as i32, 0));
    }

    #[test]
    fn zero_xp_does_not_change_progression_and_legacy_remains_single_level() {
        let mut hero = Hero::default();
        assert!(!hero.add_moba_experience(0, None));
        assert_eq!((hero.level, hero.experience), (1, 0));
        assert!(hero.add_experience(365));
        assert_eq!((hero.level, hero.experience), (2, 265));
    }
}

#[cfg(test)]
mod mana_pool_tests {
    use super::*;

    #[test]
    fn mana_lifecycle_capacity_uses_authored_base_and_checked_growth() {
        let mut hero = Hero::from_campaign_data(&crate::runtime::scene::import_campaign::HeroJD {
            id: "training_ranger".into(), abilities: Vec::new(),
        });
        assert_eq!(hero.moba_mana_capacity().unwrap(), Fixed64::from_i32(280));
        hero.intelligence = i32::MAX;
        assert_eq!(hero.moba_mana_capacity().unwrap(), Fixed64::from_i32(280));
        hero.level = 25;
        assert_eq!(hero.moba_mana_capacity().unwrap(), Fixed64::from_i32(280)
            + hero.level_growth.mana_per_level * 24);
        hero.level_growth.mana_per_level = Fixed64::from_raw(i64::MAX);
        assert!(hero.moba_mana_capacity().is_err());
        hero.level = 0;
        assert!(hero.moba_mana_capacity().is_err());
        hero.level = 26;
        assert!(hero.moba_mana_capacity().is_err());
        hero.level = 1;
        hero.id = "unknown".into();
        assert!(hero.moba_mana_capacity().is_err());
    }

    #[test]
    fn mana_pool_hero_initialization_is_opt_in_idempotent_and_growth_preserves_balance() {
        let mut hero = Hero::default();
        assert!(hero.mana_pool.is_none());
        hero.initialize_mana_pool().unwrap();
        let maximum = hero.get_max_mana();
        hero.mana_pool.as_mut().unwrap().spend(Fixed64::from_i32(20)).unwrap();
        hero.initialize_mana_pool().unwrap();
        assert_eq!(hero.mana_pool.as_ref().unwrap().current(), maximum - Fixed64::from_i32(20));
        hero.level = 2;
        hero.refresh_mana_capacity().unwrap();
        assert_eq!(hero.mana_pool.as_ref().unwrap().maximum(), hero.get_max_mana());
        assert_eq!(hero.mana_pool.as_ref().unwrap().current(), maximum - Fixed64::from_i32(20));
        let restored: Hero = serde_json::from_str(&serde_json::to_string(&hero).unwrap()).unwrap();
        assert_eq!(restored.mana_pool, hero.mana_pool);
    }

    #[test]
    fn mana_pool_legacy_save_does_not_invent_balance() {
        let hero = Hero::default();
        let value = serde_json::to_value(&hero).unwrap();
        assert!(value.get("mana_pool").is_none());
        let restored: Hero = serde_json::from_value(value).unwrap();
        assert!(restored.mana_pool.is_none());
    }
}

#[cfg(test)]
mod charge_tests {
    use super::*;
    fn s(seconds: i32) -> Fixed64 { Fixed64::from_i32(seconds) }
    fn hero() -> Hero { let mut h = Hero::new("h".into(), "h".into(), "".into()); h.sync_charges("b", Some((2, s(15)))); h }

    #[test]
    fn first_learn_is_full_and_spending_arms_one_recharge_cycle() {
        let mut h = hero();
        assert_eq!(h.ability_charges["b"], AbilityChargeState { charges: 2, max: 2, recharge: s(15), remaining: Fixed64::ZERO });
        assert!(h.cast_ready("b") && h.consume_charge("b") && h.consume_charge("b"));
        assert!(!h.has_charge("b") && !h.cast_ready("b") && !h.consume_charge("b"), "never negative");
        assert_eq!(h.ability_charges["b"].remaining, s(15), "second spend keeps the running cycle");
        assert!(h.cast_ready("other") && h.consume_charge("other"), "ordinary abilities are unaffected");
    }

    #[test]
    fn recharge_restores_one_per_cycle_and_carries_overshoot() {
        let mut h = hero();
        h.consume_charge("b"); h.consume_charge("b");
        h.tick_cooldowns(s(14));
        assert_eq!(h.ability_charges["b"].charges, 0);
        h.tick_cooldowns(s(2));
        assert_eq!((h.ability_charges["b"].charges, h.ability_charges["b"].remaining), (1, s(14)), "1 s overshoot carried");
        // One long step (e.g. accumulated while dead) restores several uses, then stops when full.
        h.tick_cooldowns(s(100));
        assert_eq!(h.ability_charges["b"], AbilityChargeState { charges: 2, max: 2, recharge: s(15), remaining: Fixed64::ZERO });
        h.tick_cooldowns(Fixed64::ZERO);
        assert_eq!(h.ability_charges["b"].charges, 2);
    }

    #[test]
    fn rank_changes_grant_or_clamp_and_unlearn_drops_state() {
        let mut h = hero();
        h.consume_charge("b");
        h.tick_cooldowns(s(5));
        h.sync_charges("b", Some((3, s(10))));
        let state = h.ability_charges["b"];
        assert_eq!((state.charges, state.max, state.recharge, state.remaining), (2, 3, s(10), s(10)), "extra use granted; cycle capped by new duration");
        h.sync_charges("b", Some((1, s(10))));
        assert_eq!((h.ability_charges["b"].charges, h.ability_charges["b"].remaining), (1, Fixed64::ZERO), "lower max clamps and stops the cycle");
        h.sync_charges("b", None);
        assert!(!h.ability_charges.contains_key("b") && h.has_charge("b"));
    }

    #[test]
    fn keep_count_policy_adds_no_uses_and_restarts_an_unfilled_cycle() {
        let keep = ChargeRankUp::KeepCountRestart;
        // Full at rank 1 (2/2): rank 2 raises the cap to 3 without a use; a cycle starts.
        let mut h = hero();
        h.sync_charges_with("b", Some((3, s(15))), keep);
        assert_eq!(h.ability_charges["b"], AbilityChargeState { charges: 2, max: 3, recharge: s(15), remaining: s(15) });
        // Mid-cycle: the progress is lost and the cycle starts over at 15 s.
        h.tick_cooldowns(s(10));
        assert_eq!(h.ability_charges["b"].remaining, s(5));
        h.sync_charges_with("b", Some((4, s(15))), keep);
        assert_eq!(h.ability_charges["b"], AbilityChargeState { charges: 2, max: 4, recharge: s(15), remaining: s(15) });
        // Re-syncing the same rank every tick is idempotent (the cycle keeps running).
        h.tick_cooldowns(s(4));
        h.sync_charges_with("b", Some((4, s(15))), keep);
        assert_eq!(h.ability_charges["b"].remaining, s(11));
        // A lower maximum still clamps; first learn is still full.
        h.sync_charges_with("b", Some((1, s(15))), keep);
        assert_eq!((h.ability_charges["b"].charges, h.ability_charges["b"].remaining), (1, Fixed64::ZERO));
        h.sync_charges_with("fresh", Some((2, s(15))), keep);
        assert_eq!(h.ability_charges["fresh"].charges, 2);
    }

    #[test]
    fn authored_policy_comes_from_content_extras_and_defaults_to_granting() {
        let saika = omoba_template_ids::ability_by_name("b01_saika_corps").and_then(omoba_template_ids::active_ability_const).unwrap();
        for rank in 1..=4 { assert_eq!(ChargeRankUp::authored(saika, rank), ChargeRankUp::KeepCountRestart, "rank {rank}"); }
        assert_eq!(ChargeRankUp::authored(saika, 0), ChargeRankUp::GrantNewUses);
        let other = omoba_template_ids::ability_by_name("a02_wind_prayer").and_then(omoba_template_ids::active_ability_const).unwrap();
        assert_eq!(ChargeRankUp::authored(other, 1), ChargeRankUp::GrantNewUses);
    }

    #[test]
    fn charges_survive_serde_and_absent_field_defaults_empty() {
        let mut h = hero();
        h.consume_charge("b");
        let value = serde_json::to_value(&h).unwrap();
        let restored: Hero = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(restored.ability_charges, h.ability_charges);
        let mut legacy = value; legacy.as_object_mut().unwrap().remove("ability_charges");
        assert!(serde_json::from_value::<Hero>(legacy).unwrap().ability_charges.is_empty());
        assert!(serde_json::to_value(Hero::new("x".into(), "x".into(), "".into())).unwrap().get("ability_charges").is_none());
    }
}
