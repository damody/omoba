use omoba_sim::Fixed64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ItemBonus {
    #[serde(default)]
    pub atk: f32,
    #[serde(default)]
    pub hp: f32,
    #[serde(default)]
    pub mp: f32,
    #[serde(default)]
    pub ms: f32,
    #[serde(default)]
    pub armor: f32,
    #[serde(default)]
    pub mp_regen: f32,
}

/// Native active-effect schema. Timed and one-shot modifiers use BuffStore;
/// RestoreMana uses the managed pool. Shield absorbs settled damage, not healing.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActiveEffect {
    /// Arm a one-shot flat physical bonus consumed by the next normal attack launch.
    HeadshotNext { bonus_damage: f32 },
    /// Timed damage absorption; remaining capacity is distinct from HP.
    Shield { amount: f32, duration: f32 },
    /// Restore managed mana, clamped to the current checked capacity.
    RestoreMana { amount: f32 },
    /// 短時間增加移速
    SprintBuff { ms_bonus: f32, duration: f32 },
    /// Timed all-packet reduction through the shared incoming damage modifier.
    DamageReduce { percent: f32, duration: f32 },
    /// Flat damage to visible enemies inside the radius of an explicit point.
    /// Amounts stay in authoritative fixed units. Float conversion is preview only.
    PointNova { damage: Fixed64, radius: Fixed64 },
    /// Flat damage to one disclosed living enemy.
    UnitHex { damage: Fixed64 },
}

impl From<omoba_template_ids::MobaItemActiveConst> for ActiveEffect {
    fn from(value: omoba_template_ids::MobaItemActiveConst) -> Self {
        use omoba_template_ids::MobaItemActiveConst as Generated;
        match value {
            Generated::Shield { amount, duration } => Self::Shield { amount: amount.to_f32_for_render(), duration: duration.to_f32_for_render() },
            Generated::SprintBuff { ms_bonus, duration } => Self::SprintBuff { ms_bonus: ms_bonus.to_f32_for_render(), duration: duration.to_f32_for_render() },
            Generated::RestoreMana { amount } => Self::RestoreMana { amount: amount.to_f32_for_render() },
            Generated::DamageReduce { percent, duration } => Self::DamageReduce { percent: percent.to_f32_for_render(), duration: duration.to_f32_for_render() },
            Generated::HeadshotNext { bonus_damage } => Self::HeadshotNext { bonus_damage: bonus_damage.to_f32_for_render() },
            Generated::PointNova { damage, radius } => Self::PointNova { damage, radius },
            Generated::UnitHex { damage } => Self::UnitHex { damage },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ItemConfig {
    pub id: String,
    pub name: String,
    pub cost: i32,
    #[serde(default)]
    pub bonus: ItemBonus,
    #[serde(default)]
    pub active: Option<ActiveEffect>,
    #[serde(default)]
    pub cooldown: f32,
    /// 升級所需組件 item_id（可為空）。購買時會從背包消耗這些組件
    #[serde(default)]
    pub recipe: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ItemRegistry {
    pub items: HashMap<String, Arc<ItemConfig>>,
}

impl ItemRegistry {
    /// No runtime file IO or independent JSON authoring for MOBA equipment.
    pub fn generated_moba() -> Self {
        Self::from_configs(omoba_template_ids::MOBA_ITEM_CATALOG.iter().map(|item| ItemConfig {
            id: item.id.into(), name: item.name.into(), cost: item.cost,
            bonus: ItemBonus {
                atk: item.atk.to_f32_for_render(), hp: item.hp.to_f32_for_render(),
                ms: item.ms.to_f32_for_render(), armor: item.armor.to_f32_for_render(),
                ..ItemBonus::default()
            },
            active: item.active.map(ActiveEffect::from), cooldown: item.cooldown.to_f32_for_render(),
            recipe: item.recipe.iter().map(|id| (*id).into()).collect(),
        }).collect())
    }

    pub fn from_configs(list: Vec<ItemConfig>) -> Self {
        let mut items = HashMap::new();
        for cfg in list {
            items.insert(cfg.id.clone(), Arc::new(cfg));
        }
        ItemRegistry { items }
    }

    pub fn get(&self, id: &str) -> Option<Arc<ItemConfig>> {
        self.items.get(id).cloned()
    }
}

/// 回收售價（50%）
pub fn sell_price(cost: i32) -> i32 {
    (cost as f32 * 0.5) as i32
}

/// Sum of catalog bonuses currently in one inventory. Unknown ids add nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ItemStatAggregate {
    pub atk: f32,
    pub hp: f32,
    pub mp: f32,
    pub ms: f32,
    pub armor: f32,
    pub mp_regen: f32,
}

impl ItemStatAggregate {
    pub fn from_inventory(inventory: &super::comp::Inventory, registry: &ItemRegistry) -> Self {
        let mut totals = Self::default();
        for slot in &inventory.slots {
            let Some(inst) = slot else { continue };
            let Some(cfg) = registry.get(&inst.item_id) else { continue };
            totals.atk += cfg.bonus.atk;
            totals.hp += cfg.bonus.hp;
            totals.mp += cfg.bonus.mp;
            totals.ms += cfg.bonus.ms;
            totals.armor += cfg.bonus.armor;
            totals.mp_regen += cfg.bonus.mp_regen;
        }
        totals
    }
}

/// How a freshly computed aggregate meets an existing body.
/// `NewLife` is a call argument for a body constructed in this spawn, never a stored heal flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemStatApplication {
    /// Buy, sell, undo, or any later dirty rebuild. Current HP is never raised.
    ExistingBody,
    /// The life constructed by this call. Current HP becomes the finalized maximum.
    NewLife,
}

fn f32_bonus_to_fx(v: f32) -> Fixed64 {
    Fixed64::from_raw((v * omoba_sim::fixed::SCALE as f32) as i64)
}

/// Replace previously applied item deltas with `totals`.
/// Returns the effects record the caller stores. Does not tick cooldowns or touch mana or XP.
pub fn apply_item_stats(
    totals: ItemStatAggregate,
    applied: &super::comp::ItemEffects,
    property: Option<&mut super::comp::CProperty>,
    attack: Option<&mut super::comp::TAttack>,
    application: ItemStatApplication,
) -> super::comp::ItemEffects {
    use super::comp::Vf32;
    if let Some(prop) = property {
        prop.mhp = prop.mhp - f32_bonus_to_fx(applied.applied_hp) + f32_bonus_to_fx(totals.hp);
        prop.msd = prop.msd - f32_bonus_to_fx(applied.applied_ms) + f32_bonus_to_fx(totals.ms);
        prop.def_physic = prop.def_physic - f32_bonus_to_fx(applied.applied_armor) + f32_bonus_to_fx(totals.armor);
        match application {
            ItemStatApplication::NewLife => prop.hp = prop.mhp,
            ItemStatApplication::ExistingBody => {
                // A non-positive body stays dead, including a negative value.
                // A living body keeps its current HP, including a positive fraction,
                // and is only clamped down when the new maximum is lower.
                if prop.hp > Fixed64::ZERO && prop.hp > prop.mhp {
                    prop.hp = prop.mhp;
                }
            }
        }
    }
    if let Some(atk) = attack {
        let cur = atk.atk_physic.val();
        atk.atk_physic = Vf32::new(cur - f32_bonus_to_fx(applied.applied_atk) + f32_bonus_to_fx(totals.atk));
    }
    let mut next = *applied;
    next.bonus_atk = totals.atk;
    next.bonus_hp = totals.hp;
    next.bonus_mp = totals.mp;
    next.bonus_ms = totals.ms;
    next.bonus_armor = totals.armor;
    next.bonus_mp_regen = totals.mp_regen;
    next.applied_atk = totals.atk;
    next.applied_hp = totals.hp;
    next.applied_ms = totals.ms;
    next.applied_armor = totals.armor;
    next.dirty = false;
    next
}
