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
    #[serde(default)]
    pub lifesteal: f32,
    #[serde(default)]
    pub critical_chance: f32,
    #[serde(default)]
    pub unique_passive: Option<omoba_content_model::moba_items::MobaItemUniquePassive>,
}

/// Native active-effect schema. Timed and one-shot modifiers use BuffStore;
/// RestoreMana uses the managed pool. Shield absorbs settled damage, not healing.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActiveEffect {
    SelfControlCleanse {},
    ChannelHeal {amount:Fixed64,duration:Fixed64,interval:Fixed64},
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
    UnitHex { damage: Fixed64,#[serde(default)] healing_reduction:Option<HealingReduction>,
        #[serde(default)] control:Option<ItemControl> },
}

#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemControl {pub kind:omoba_content_model::ControlEffectKind,pub duration:Fixed64}

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub struct HealingReduction {pub multiplier:Fixed64,pub duration:Fixed64}

impl From<omoba_template_ids::MobaItemActiveConst> for ActiveEffect {
    fn from(value: omoba_template_ids::MobaItemActiveConst) -> Self {
        use omoba_template_ids::MobaItemActiveConst as Generated;
        match value {
            Generated::SelfControlCleanse => Self::SelfControlCleanse {},
            Generated::ChannelHeal {amount,duration,interval} => Self::ChannelHeal {amount,duration,interval},
            Generated::Shield { amount, duration } => Self::Shield { amount: amount.to_f32_for_render(), duration: duration.to_f32_for_render() },
            Generated::SprintBuff { ms_bonus, duration } => Self::SprintBuff { ms_bonus: ms_bonus.to_f32_for_render(), duration: duration.to_f32_for_render() },
            Generated::RestoreMana { amount } => Self::RestoreMana { amount: amount.to_f32_for_render() },
            Generated::DamageReduce { percent, duration } => Self::DamageReduce { percent: percent.to_f32_for_render(), duration: duration.to_f32_for_render() },
            Generated::HeadshotNext { bonus_damage } => Self::HeadshotNext { bonus_damage: bonus_damage.to_f32_for_render() },
            Generated::PointNova { damage, radius } => Self::PointNova { damage, radius },
            Generated::UnitHex { damage,healing_reduction,control } => Self::UnitHex { damage,
                healing_reduction:healing_reduction.map(|r|HealingReduction {multiplier:r.multiplier,duration:r.duration}),
                control:control.map(|c|ItemControl {kind:match c.kind {
                    omoba_template_ids::MobaItemControlKindConst::Stun=>omoba_content_model::ControlEffectKind::Stun,
                    omoba_template_ids::MobaItemControlKindConst::Root=>omoba_content_model::ControlEffectKind::Root,
                    omoba_template_ids::MobaItemControlKindConst::Silence=>omoba_content_model::ControlEffectKind::Silence,
                },duration:c.duration}) },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ItemConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purchase_group: Option<omoba_content_model::moba_items::MobaItemPurchaseGroup>,
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
        Self::from_configs(omoba_template_ids::MOBA_ITEM_CATALOG.iter().map(|item| ItemConfig { purchase_group: item.purchase_group.map(|v|omoba_content_model::moba_items::MobaItemPurchaseGroup {id:v.id.into(),name:v.name.into(),limit:v.limit}),
            id: item.id.into(), name: item.name.into(), cost: item.cost,
            bonus: ItemBonus {
                atk: item.atk.to_f32_for_render(), hp: item.hp.to_f32_for_render(),
                ms: item.ms.to_f32_for_render(), armor: item.armor.to_f32_for_render(),
                lifesteal: item.lifesteal.to_f32_for_render(),
                critical_chance: item.critical_chance.to_f32_for_render(),
                unique_passive: item.unique_passive.map(|v|omoba_content_model::moba_items::MobaItemUniquePassive {
                    group:v.group.into(),name:v.name.into(),priority:v.priority,
                    atk:v.atk.to_f32_for_render(),hp:v.hp.to_f32_for_render(),ms:v.ms.to_f32_for_render(),
                    armor:v.armor.to_f32_for_render(),lifesteal:v.lifesteal.to_f32_for_render(),
                    critical_chance:v.critical_chance.to_f32_for_render(),
                }),
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

#[cfg(test)]
mod cleanse_tests {
    use super::*;
    #[test]
    fn generated_cleanse_has_typed_runtime_operation_and_no_hidden_parameters() {
        let row=ItemRegistry::generated_moba().get("moba_purification_charm").unwrap();
        assert_eq!(row.cost,1400);assert_eq!(row.cooldown,45.);
        assert!(matches!(row.active,Some(ActiveEffect::SelfControlCleanse {})));
        let effect:ActiveEffect=serde_json::from_value(serde_json::json!({"kind":"self_control_cleanse"})).unwrap();
        assert_eq!(serde_json::to_value(effect).unwrap(),serde_json::json!({"kind":"self_control_cleanse"}));
    }
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
    pub lifesteal: f32,
    pub critical_chance: f32,
}

impl ItemStatAggregate {
    pub fn from_inventory(inventory: &super::comp::Inventory, registry: &ItemRegistry) -> Self {
        let mut totals = Self::default();
        let mut winners=std::collections::BTreeMap::new();
        for slot in &inventory.slots {
            let Some(inst) = slot else { continue };
            let Some(cfg) = registry.get(&inst.item_id) else { continue };
            totals.atk += cfg.bonus.atk;
            totals.hp += cfg.bonus.hp;
            totals.mp += cfg.bonus.mp;
            totals.ms += cfg.bonus.ms;
            totals.armor += cfg.bonus.armor;
            totals.mp_regen += cfg.bonus.mp_regen;
            totals.lifesteal += cfg.bonus.lifesteal;
            totals.critical_chance += cfg.bonus.critical_chance;
            if let Some(passive)=cfg.bonus.unique_passive.as_ref().filter(|p|p.validate().is_ok()) {
                let replace=winners.get(&passive.group).is_none_or(|(priority,id,_):&(u16,String,omoba_content_model::moba_items::MobaItemUniquePassive)|
                    passive.priority>*priority || (passive.priority==*priority && cfg.id<*id));
                if replace {winners.insert(passive.group.clone(),(passive.priority,cfg.id.clone(),passive.clone()));}
            }
        }
        for (_,(_,_,passive)) in winners {
            totals.atk+=passive.atk;totals.hp+=passive.hp;totals.ms+=passive.ms;totals.armor+=passive.armor;
            totals.lifesteal+=passive.lifesteal;totals.critical_chance+=passive.critical_chance;
        }
        totals
    }
}

/// How a freshly computed aggregate meets an existing body.
/// Inventory owns one permanent shared modifier, rebuilt only on inventory changes/new life.
pub fn sync_item_passives(entity:specs::Entity,totals:ItemStatAggregate,buffs:&mut super::ability_runtime::BuffStore) {
    const KEY:&str="__item_passives";
    buffs.remove(entity,KEY);
    let raw=|rate:f32| if rate.is_finite(){(rate.clamp(0.0,1.0)*1024.0).round() as i64}else{0};
    let lifesteal=raw(totals.lifesteal);let critical=raw(totals.critical_chance);
    if lifesteal>0 || critical>0 {
        buffs.add(entity,KEY,super::ability_runtime::buff_store::PERMANENT_BUFF_DURATION,
            serde_json::json!({"lifesteal":lifesteal,"preattack_criticalstrike":critical}));
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

#[cfg(test)]
#[test]
fn unique_passive_whole_winner_is_slot_independent_and_groups_stack() {
    use super::comp::{Inventory,ItemInstance};
    let make=|id:&str,group:&str,priority:u16,ms:f32,hp:f32|ItemConfig { purchase_group: None,
        id:id.into(),name:id.into(),cost:100,active:None,cooldown:0.,recipe:vec![],
        bonus:ItemBonus {atk:3.,unique_passive:Some(omoba_content_model::moba_items::MobaItemUniquePassive {
            group:group.into(),name:"Unique".into(),priority,ms,hp,atk:0.,armor:0.,lifesteal:0.125,critical_chance:0.25,
        }),..Default::default()},
    };
    let registry=ItemRegistry::from_configs(vec![make("z","group",1,25.,300.),make("b","group",2,45.,10.),
        make("a","group",2,40.,20.),make("other","independent",1,5.,30.)]);
    for order in [["z","b","a","a","other"],["other","a","z","a","b"],["b","a","other","z","a"]] {
        let mut inventory=Inventory::default();for (slot,id) in order.iter().enumerate(){inventory.slots[slot]=Some(ItemInstance {item_id:(*id).into(),cooldown_remaining:0.});}
        let total=ItemStatAggregate::from_inventory(&inventory,&registry);
        assert_eq!(total.ms,45.);assert_eq!(total.hp,50.,"entire a wins, not strongest HP from z");
        assert_eq!(total.atk,15.,"base stats still stack for duplicate copies");
        assert_eq!(total.lifesteal,0.25);assert_eq!(total.critical_chance,0.5);
        inventory.slots.iter_mut().for_each(|s|if s.as_ref().is_some_and(|i|i.item_id=="a"){*s=None});
        assert_eq!(ItemStatAggregate::from_inventory(&inventory,&registry).ms,50.,"removal exposes b");
    }
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
