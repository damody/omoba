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
        vec![BotItemBuild {role:BotRole::Carry,items:items.iter().map(|s|(*s).into()).collect(),return_to_shop:None}]
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
