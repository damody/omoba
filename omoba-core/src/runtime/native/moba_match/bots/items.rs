//! Desired final inventories, derived from owner data without purchase cursors.
use super::BotRole;
use crate::runtime::native::{comp::{Gold, Inventory, INVENTORY_SLOTS}, item::ItemRegistry,
    shop::preview_buy_item};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotItemBuild {
    pub role: BotRole,
    /// Ordered final goals, not an imperative list of component purchases.
    pub items: Vec<String>,
    #[serde(default)]
    pub return_to_shop: Option<BotShopReturnPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_use: Option<BotActiveItemPolicy>,
}

/// Opt-in, owner-only thresholds. Zero disables the corresponding resource rule.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotActiveItemPolicy {
    pub combat_radius: u32,
    pub defend_below_hp_per_mille: u16,
    pub restore_below_mana_per_mille: u16,
}

pub(super) struct ActiveItemObservation<'a> {
    pub health: &'a crate::runtime::CProperty,
    pub mana: Option<&'a crate::runtime::ability_runtime::ManaPool>,
    pub shield_raw: i64,
    pub next_attack_bonus_raw: i64,
    pub reduction_active: bool,
    pub sprint_active: bool,
    pub immobilized: bool,
    pub attack_windup: bool,
    pub attack_range: omoba_sim::Fixed64,
}

// ItemUse replaces the current order. Emergency defense may interrupt an
// attack; routine resource recovery and attack preparation must not do so.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ActiveItemPriority { Shield, Reduction, Escape, Recovery, AttackPreparation }

/// Returns a normal input slot, never mutates resources or simulates a use.
pub(super) fn choose_active(role: BotRole, builds: &[BotItemBuild], registry: &ItemRegistry,
    inventory: &Inventory, own: omoba_sim::Vec2, team: u32, seen: &[super::SeenUnit],
    observation: &ActiveItemObservation<'_>) -> Option<usize>
{
    use crate::runtime::native::item::ActiveEffect;
    use omoba_sim::Fixed64;
    let policy=builds.iter().find(|b|b.role==role)?.active_use?;
    if !(1..=10_000).contains(&policy.combat_radius) || policy.defend_below_hp_per_mille>1000
        || policy.restore_below_mana_per_mille>1000 || team==0
        || observation.health.hp<=Fixed64::ZERO || observation.health.mhp<=Fixed64::ZERO {return None;}
    let threatened=super::disclosed_threat(own,team,seen,policy.combat_radius);
    let defend=threatened && i128::from(observation.health.hp.raw())*1000
        < i128::from(observation.health.mhp.raw())*i128::from(policy.defend_below_hp_per_mille);
    let attack_radius=observation.attack_range.min(Fixed64::from_i32(policy.combat_radius as i32));
    let attack_opportunity=attack_radius>Fixed64::ZERO && seen.iter().any(|u|
        matches!(u.kind,1|2|3) && u.team!=team && u.hp_raw>0
        && (u.position-own).length_squared()<=attack_radius*attack_radius);
    inventory.items().filter_map(|(slot,item)| {
        if !item.cooldown_remaining.is_finite() || item.cooldown_remaining!=0.0 {return None;}
        let config=registry.get(&item.item_id)?;
        let effect=config.active.as_ref()?;
        let priority=match effect {
            ActiveEffect::RestoreMana {..} if !observation.attack_windup && observation.mana.is_some_and(|pool|
                pool.maximum()>Fixed64::ZERO && i128::from(pool.current().raw())*1000
                    < i128::from(pool.maximum().raw())*i128::from(policy.restore_below_mana_per_mille))=>ActiveItemPriority::Recovery,
            ActiveEffect::Shield {..} if defend && observation.shield_raw==0=>ActiveItemPriority::Shield,
            ActiveEffect::DamageReduce {..} if defend && !observation.reduction_active=>ActiveItemPriority::Reduction,
            ActiveEffect::SprintBuff {..} if defend && !observation.immobilized && !observation.sprint_active=>ActiveItemPriority::Escape,
            ActiveEffect::HeadshotNext {..} if attack_opportunity && !observation.attack_windup
                && observation.next_attack_bonus_raw==0=>ActiveItemPriority::AttackPreparation,
            _=>return None,
        };
        Some((priority,slot))
    }).min().map(|(_,slot)|slot)
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotShopReturnPolicy {
    pub min_gold: u32,
    pub threat_radius: u32,
}

pub(super) fn validate(builds: &[BotItemBuild]) -> Result<(), &'static str> {
    if builds.len()>5 {return Err("at most five bot item builds");}
    let mut roles=BTreeSet::new();
    for build in builds {
        if build.active_use.is_some_and(|p| !(1..=10_000).contains(&p.combat_radius)
            || p.defend_below_hp_per_mille>1000 || p.restore_below_mana_per_mille>1000) {
            return Err("bot active item policy requires radius 1..10000 and thresholds 0..1000");
        }
        if build.return_to_shop.is_some_and(|p|p.min_gold==0 || p.min_gold>i32::MAX as u32
            || !(1..=10_000).contains(&p.threat_radius)) {
            return Err("bot shop return requires positive i32 balance threshold and threat radius 1..10000");
        }
        if !roles.insert(build.role) {return Err("duplicate bot item build role");}
        if build.items.is_empty() || build.items.len()>INVENTORY_SLOTS {
            return Err("bot item build requires one to six final items");
        }
        if build.items.iter().any(|id| !omoba_template_ids::MOBA_ITEM_CATALOG.iter().any(|i|i.id==id)) {
            return Err("unknown compiled bot item goal");
        }
    }
    Ok(())
}

/// Pre-match only: reject declared priorities that exhaust six slots before a
/// recipe can be assembled. Do not run this simulation on the Bot hot path.
pub(super) fn validate_feasible(builds: &[BotItemBuild]) -> Result<(), &'static str> {
    validate(builds)?;
    let registry=ItemRegistry::generated_moba();
    for build in builds {
        let mut inventory=Inventory::new();
        let mut complete=false;
        for _ in 0..4096 {
            let mut available=BTreeMap::new();
            for (_,item) in inventory.items() {*available.entry(item.item_id.clone()).or_insert(0)+=1;}
            if build.items.iter().all(|id|take(&mut available,id)) {complete=true;break;}
            let mut gold=Gold(i32::MAX);
            let id=choose_purchase(build.role,std::slice::from_ref(build),&registry,&inventory,&gold)
                .ok_or("bot item priorities cannot assemble final inventory in six slots")?;
            crate::runtime::native::shop::buy_item(&registry,&mut inventory,&mut gold,&id)
                .map_err(|_|"bot item build cannot be purchased")?;
        }
        if !complete {return Err("bot item build exceeds bounded purchase planning budget");}
    }
    Ok(())
}

fn take(available: &mut BTreeMap<String,usize>, id: &str) -> bool {
    if let Some(count)=available.get_mut(id) {
        if *count>0 {*count-=1;return true;}
    }
    false
}

/// Caller must also have an affordable pending purchase. Unknown opponents are
/// never looked up: absence of disclosed threats is not a safety guarantee.
pub(super) fn may_recall(role:BotRole,builds:&[BotItemBuild],gold:&Gold,
    own:omoba_sim::Vec2,home:omoba_sim::Vec2,team:u32,seen:&[super::SeenUnit]) -> bool {
    let Some(policy)=builds.iter().find(|b|b.role==role).and_then(|b|b.return_to_shop) else {return false;};
    let Ok(threshold)=i32::try_from(policy.min_gold) else {return false;};
    let radius=omoba_sim::Fixed64::from_i32(crate::runtime::native::shop::MOBA_SHOP_RADIUS);
    gold.0>=threshold && (own-home).length_squared()>radius*radius
        && !super::disclosed_threat(own,team,seen,policy.threat_radius)
}

fn missing_purchase(registry: &ItemRegistry, available: &mut BTreeMap<String,usize>,
    id: &str, visiting: &mut BTreeSet<String>) -> Option<String>
{
    // Defensive cycle rejection also bounds recursion by catalog size.
    if !visiting.insert(id.into()) {return None;}
    let item=registry.get(id)?;
    for component in &item.recipe {
        if !take(available,component) {
            return missing_purchase(registry,available,component,visiting);
        }
    }
    Some(id.into())
}

pub(super) fn choose_purchase(role: BotRole, builds: &[BotItemBuild], registry: &ItemRegistry,
    inventory: &Inventory, gold: &Gold) -> Option<String>
{
    let build=builds.iter().find(|b|b.role==role)?;
    let mut available=BTreeMap::new();
    for (_,item) in inventory.items() {*available.entry(item.item_id.clone()).or_insert(0)+=1;}
    // Reserve ALL already satisfied final goals, including later ones. Recipe
    // components must not repeatedly consume and repurchase completed goals.
    let mut missing=Vec::new();
    for goal in &build.items {if !take(&mut available,goal) {missing.push(goal);}}
    let goal=missing.first()?;
    let purchase=missing_purchase(registry,&mut available,goal,&mut BTreeSet::new())?;
    // Respect declared priority: insufficient balance/capacity never triggers
    // sales or skipping to a cheaper later goal. Formal authority may still deny.
    preview_buy_item(registry,inventory,gold,&purchase).ok()?;
    Some(purchase)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::native::{comp::ItemInstance, shop::buy_item};
    fn build(items: &[&str]) -> Vec<BotItemBuild> {
        vec![BotItemBuild {role:BotRole::Carry,items:items.iter().map(|s|(*s).into()).collect(),return_to_shop:None,active_use:None}]
    }
    #[test]
    fn role_bot_active_items_generated_kinds_owner_thresholds_and_safe_skip() {
        use omoba_sim::{Fixed64,Vec2};
        use crate::runtime::{CProperty,ability_runtime::ManaPool};
        let registry=ItemRegistry::generated_moba();
        let health=CProperty {hp:Fixed64::from_i32(20),mhp:Fixed64::from_i32(100),
            msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO};
        let mana=ManaPool::new(Fixed64::from_i32(10),Fixed64::from_i32(100)).unwrap();
        let enemy=super::super::SeenUnit {canonical_id:2,position:Vec2::ZERO,team:2,kind:1,
            owner_player_id:2,hp_raw:1024,max_hp_raw:1024};
        let mut observation=ActiveItemObservation {health:&health,mana:Some(&mana),shield_raw:0,
            next_attack_bonus_raw:0,reduction_active:false,sprint_active:false,immobilized:false,
            attack_windup:false,attack_range:Fixed64::from_i32(500)};
        for id in ["moba_guard_charm","moba_stride_charm","moba_mana_charm","moba_ward_charm","moba_strike_charm"] {
            let mut builds=build(&[id]);
            let mut inventory=Inventory::new();
            buy_item(&registry,&mut inventory,&mut Gold(2000),id).unwrap();
            assert!(choose_active(BotRole::Carry,&builds,&registry,&inventory,Vec2::ZERO,1,&[enemy],&observation).is_none());
            builds[0].active_use=Some(BotActiveItemPolicy {combat_radius:1000,
                defend_below_hp_per_mille:500,restore_below_mana_per_mille:300});
            assert_eq!(choose_active(BotRole::Carry,&builds,&registry,&inventory,Vec2::ZERO,1,&[enemy],&observation),Some(0));
            if id!="moba_mana_charm" {
                assert!(choose_active(BotRole::Carry,&builds,&registry,&inventory,Vec2::ZERO,1,&[],&observation).is_none());
                assert!(choose_active(BotRole::Carry,&builds,&registry,&inventory,Vec2::ZERO,1,&[super::super::SeenUnit {hp_raw:0,..enemy}],&observation).is_none());
            }
            for cooldown in [1.0,-1.0,f32::NAN,f32::INFINITY] {
                inventory.slots[0].as_mut().unwrap().cooldown_remaining=cooldown;
                assert!(choose_active(BotRole::Carry,&builds,&registry,&inventory,Vec2::ZERO,1,&[enemy],&observation).is_none());
            }
            inventory.slots[0].as_mut().unwrap().cooldown_remaining=0.0;
            observation.shield_raw=1;observation.next_attack_bonus_raw=1;
            observation.reduction_active=true;observation.sprint_active=true;observation.mana=None;
            assert!(choose_active(BotRole::Carry,&builds,&registry,&inventory,Vec2::ZERO,1,&[enemy],&observation).is_none());
            observation.shield_raw=0;observation.next_attack_bonus_raw=0;
            observation.reduction_active=false;observation.sprint_active=false;observation.mana=Some(&mana);
            if id=="moba_stride_charm" {observation.immobilized=true;}
            if matches!(id,"moba_strike_charm"|"moba_mana_charm") {observation.attack_windup=true;}
            if observation.immobilized || observation.attack_windup {
                assert!(choose_active(BotRole::Carry,&builds,&registry,&inventory,Vec2::ZERO,1,&[enemy],&observation).is_none());
            }
            observation.immobilized=false;observation.attack_windup=false;
            builds[0].active_use.as_mut().unwrap().combat_radius=0;
            assert!(validate(&builds).is_err());
        }
    }
    #[test]
    fn role_bot_active_items_defense_precedes_recovery_and_windup_preserves_routine_order() {
        use omoba_sim::{Fixed64,Vec2};
        use crate::runtime::{CProperty,ability_runtime::ManaPool};
        let registry=ItemRegistry::generated_moba();
        let health=CProperty {hp:Fixed64::from_i32(20),mhp:Fixed64::from_i32(100),
            msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO};
        let mana=ManaPool::new(Fixed64::from_i32(10),Fixed64::from_i32(100)).unwrap();
        let enemy=super::super::SeenUnit {canonical_id:2,position:Vec2::ZERO,team:2,kind:1,
            owner_player_id:2,hp_raw:1024,max_hp_raw:1024};
        for ids in [vec!["moba_mana_charm","moba_guard_charm","moba_ward_charm","moba_stride_charm"],
            vec!["moba_stride_charm","moba_ward_charm","moba_guard_charm","moba_mana_charm"]] {
            let mut builds=build(&ids);
            builds[0].active_use=Some(BotActiveItemPolicy {combat_radius:700,
                defend_below_hp_per_mille:500,restore_below_mana_per_mille:300});
            let mut inventory=Inventory::new();
            for id in &ids {buy_item(&registry,&mut inventory,&mut Gold(2000),id).unwrap();}
            let before=serde_json::to_value(&inventory).unwrap();
            let mut observation=ActiveItemObservation {health:&health,mana:Some(&mana),shield_raw:0,
                next_attack_bonus_raw:0,reduction_active:false,sprint_active:false,immobilized:false,
                attack_windup:true,attack_range:Fixed64::from_i32(500)};
            let choose=|obs:&ActiveItemObservation<'_>,seen:&[super::super::SeenUnit]| {
                choose_active(BotRole::Carry,&builds,&registry,&inventory,Vec2::ZERO,1,seen,obs)
                    .map(|slot|inventory.slots[slot].as_ref().unwrap().item_id.as_str())
            };
            assert_eq!(choose(&observation,&[enemy]),Some("moba_guard_charm"));
            observation.shield_raw=1;
            assert_eq!(choose(&observation,&[enemy]),Some("moba_ward_charm"));
            observation.reduction_active=true;
            assert_eq!(choose(&observation,&[enemy]),Some("moba_stride_charm"));
            observation.sprint_active=true;
            assert_eq!(choose(&observation,&[enemy]),None);
            observation.attack_windup=false;
            assert_eq!(choose(&observation,&[enemy]),Some("moba_mana_charm"));
            assert_eq!(choose(&observation,&[]),Some("moba_mana_charm"));
            observation.attack_windup=true;
            assert_eq!(choose(&observation,&[]),None);
            assert_eq!(serde_json::to_value(&inventory).unwrap(),before);
        }
    }

    #[test]
    fn role_bot_items_buy_duplicate_materials_combine_and_stop_without_cursor() {
        let registry=ItemRegistry::generated_moba();
        let builds=build(&["moba_greatsword","moba_boots"]);
        let mut inventory=Inventory::new(); let mut gold=Gold(1250);
        for (id,cost) in [("moba_sword",350),("moba_sword",350),("moba_greatsword",250),("moba_boots",300)] {
            let before=serde_json::to_string(&inventory).unwrap();let balance=gold.0;
            assert_eq!(choose_purchase(BotRole::Carry,&builds,&registry,&inventory,&gold).as_deref(),Some(id));
            assert_eq!(serde_json::to_string(&inventory).unwrap(),before);assert_eq!(gold.0,balance);
            buy_item(&registry,&mut inventory,&mut gold,id).unwrap();assert_eq!(gold.0,balance-cost);
        }
        assert_eq!(inventory.items().count(),2);assert_eq!(gold.0,0);
        assert!(choose_purchase(BotRole::Carry,&builds,&registry,&inventory,&Gold(9999)).is_none());
        // Multi-level recipes use catalog edges, never item-specific Rust branches.
        let mut registry=registry;
        let mut nested=(*registry.get("moba_greatsword").unwrap()).clone();
        nested.id="test_nested".into();nested.cost=2000;
        nested.recipe=vec!["moba_greatsword".into(),"moba_greatsword".into()];
        registry.items.insert(nested.id.clone(),std::sync::Arc::new(nested));
        let builds=build(&["test_nested"]);
        let mut inventory=Inventory::new();let mut gold=Gold(2000);
        for id in ["moba_sword","moba_sword","moba_greatsword","moba_sword","moba_sword","moba_greatsword","test_nested"] {
            assert_eq!(choose_purchase(BotRole::Carry,&builds,&registry,&inventory,&gold).as_deref(),Some(id));
            buy_item(&registry,&mut inventory,&mut gold,id).unwrap();
        }
        assert_eq!(gold.0,0);assert_eq!(inventory.items().count(),1);
        assert!(choose_purchase(BotRole::Carry,&builds,&registry,&inventory,&Gold(9999)).is_none());
    }
    #[test]
    fn role_bot_items_reserve_final_goals_and_fail_closed_on_balance_capacity_and_cycle() {
        let mut registry=ItemRegistry::generated_moba();
        let builds=build(&["moba_greatsword","moba_sword"]);
        let mut inventory=Inventory::new();
        buy_item(&registry,&mut inventory,&mut Gold(350),"moba_sword").unwrap();
        assert_eq!(choose_purchase(BotRole::Carry,&builds,&registry,&inventory,&Gold(350)).as_deref(),Some("moba_sword"));
        assert!(choose_purchase(BotRole::Carry,&builds,&registry,&inventory,&Gold(349)).is_none());
        assert!(choose_purchase(BotRole::Carry,&builds,&registry,&inventory,&Gold(-1)).is_none());
        for slot in &mut inventory.slots {*slot=Some(ItemInstance {item_id:"moba_armor".into(),cooldown_remaining:0.0});}
        assert!(choose_purchase(BotRole::Carry,&builds,&registry,&inventory,&Gold(9999)).is_none());
        // Full inventory can combine: use the transaction kernel, not free-slot heuristics.
        for slot in &mut inventory.slots[..2] {*slot=Some(ItemInstance {item_id:"moba_sword".into(),cooldown_remaining:0.0});}
        assert_eq!(choose_purchase(BotRole::Carry,&build(&["moba_greatsword"]),&registry,&inventory,&Gold(250)).as_deref(),Some("moba_greatsword"));
        let mut cyclic=(*registry.get("moba_sword").unwrap()).clone();cyclic.recipe=vec!["moba_greatsword".into()];
        registry.items.insert(cyclic.id.clone(),std::sync::Arc::new(cyclic));
        assert!(choose_purchase(BotRole::Carry,&builds,&registry,&Inventory::new(),&Gold(9999)).is_none());
        assert!(validate(&build(&["unknown"])).is_err());
        assert!(validate(&build(&["moba_sword";7])).is_err());
        let mut duplicate=build(&["moba_sword"]);duplicate.extend(build(&["moba_boots"]));
        assert!(validate(&duplicate).is_err());
        assert!(validate(&build(&["moba_sword","moba_sword"])).is_ok());
        assert!(validate_feasible(&build(&["moba_armor","moba_armor","moba_armor","moba_armor","moba_armor","moba_greatsword"])).is_err());
        assert!(validate_feasible(&build(&["moba_greatsword","moba_armor","moba_armor","moba_armor","moba_armor","moba_armor"])).is_ok());
        assert!(validate_feasible(&builds).is_ok());
    }

    #[test]
    fn role_bot_items_shop_recall_is_opt_in_and_only_uses_disclosed_threats() {
        use omoba_sim::{Vec2,Fixed64};
        let home=Vec2::ZERO;let own=Vec2::new(Fixed64::from_i32(2000),Fixed64::ZERO);
        let mut builds=build(&["moba_greatsword"]);
        assert!(!may_recall(BotRole::Carry,&builds,&Gold(9999),own,home,1,&[]));
        builds[0].return_to_shop=Some(BotShopReturnPolicy {min_gold:950,threat_radius:1000});
        assert!(may_recall(BotRole::Carry,&builds,&Gold(950),own,home,1,&[]));
        assert!(!may_recall(BotRole::Carry,&builds,&Gold(949),own,home,1,&[]));
        assert!(!may_recall(BotRole::Carry,&builds,&Gold(950),home,home,1,&[]));
        let enemy=super::super::SeenUnit {canonical_id:2,position:own,team:2,kind:1,owner_player_id:2,hp_raw:1,max_hp_raw:100};
        assert!(!may_recall(BotRole::Carry,&builds,&Gold(950),own,home,1,&[enemy]));
        assert!(!may_recall(BotRole::Carry,&builds,&Gold(950),own,home,1,&[super::super::SeenUnit {team:0,kind:3,..enemy}]));
        assert!(may_recall(BotRole::Carry,&builds,&Gold(950),own,home,1,&[super::super::SeenUnit {hp_raw:0,..enemy}]));
        assert!(may_recall(BotRole::Carry,&builds,&Gold(950),own,home,1,&[super::super::SeenUnit {team:1,..enemy}]));
        builds[0].return_to_shop.as_mut().unwrap().min_gold=0;
        assert!(validate(&builds).is_err());
    }
}
