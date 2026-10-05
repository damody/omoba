//! Deterministic, atomic inventory transactions. Network authorization and
//! shop proximity must be checked by the authority before invoking this kernel.
use super::comp::{Gold, Inventory, ItemInstance};
use super::item::ItemRegistry;

/// Shared admission radius for authority transactions and owner-side planning.
pub const MOBA_SHOP_RADIUS: i32 = 300;

/// Read-only preview using the exact atomic transaction kernel. This is not
/// authority admission: phase, owner identity and proximity are rechecked there.
pub fn preview_buy_item(registry: &ItemRegistry, inventory: &Inventory, gold: &Gold,
    item_id: &str) -> Result<usize, ShopError>
{
    let mut next_inventory=inventory.clone();
    let mut next_gold=*gold;
    buy_item(registry, &mut next_inventory, &mut next_gold, item_id)
}

fn valid_item(item: &super::item::ItemConfig) -> bool {
    item.cost > 0
        && item.cooldown.is_finite()
        && item.cooldown >= 0.0
        && [
            item.bonus.atk,
            item.bonus.hp,
            item.bonus.mp,
            item.bonus.ms,
            item.bonus.armor,
            item.bonus.mp_regen,
        ]
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShopError {
    UnknownItem,
    InvalidCatalog,
    InvalidBalance,
    MissingComponent,
    InventoryFull,
    InsufficientGold,
    InvalidSlot,
    BalanceOverflow,
    MatchUnavailable,
    UnknownPlayer,
    HeroUnavailable,
    OutsideShop,
    MissingComponents,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShopCommand {
    Buy(String),
    Sell(usize),
}

/// Tick-local output only. Correlation belongs to the projection layer, not
/// the gameplay command or any replayed component.
#[derive(Clone, Debug)]
pub struct ShopSettlement {
    pub player_id: u32,
    pub command: ShopCommand,
    pub result: Result<(), ShopError>,
}

impl ShopError {
    /// Stable receipt codes. Zero is reserved for successful settlement.
    pub fn receipt_code(&self) -> u32 {
        match self {
            Self::UnknownItem => 1,
            Self::InvalidCatalog => 2,
            Self::InvalidBalance => 3,
            Self::MissingComponent => 4,
            Self::InventoryFull => 5,
            Self::InsufficientGold => 6,
            Self::InvalidSlot => 7,
            Self::BalanceOverflow => 8,
            Self::MatchUnavailable => 9,
            Self::UnknownPlayer => 10,
            Self::HeroUnavailable => 11,
            Self::OutsideShop => 12,
            Self::MissingComponents => 13,
        }
    }
}

/// The caller must supply its authenticated player ID, never a client-selected
/// hero. Formal PlayerInput drains call this authority-only API; secure network
/// and renderer IPC shop commands remain gated pending economy projection.
pub fn transact_moba_shop(
    world: &mut specs::World,
    player_id: u32,
    command: ShopCommand,
) -> Result<(), ShopError> {
    use super::comp::{ItemEffects, PlayerOwner, Pos};
    use super::moba_match::{MobaMatch, MobaMatchPhase};
    use omoba_sim::Fixed64;
    use specs::WorldExt;
    if world
        .try_fetch::<super::comp::GamePause>()
        .is_none_or(|pause| pause.is_paused)
    {
        return Err(ShopError::MatchUnavailable);
    }
    let (hero, base) = {
        let state = world
            .try_fetch::<MobaMatch>()
            .ok_or(ShopError::MatchUnavailable)?;
        if state.phase != MobaMatchPhase::Playing {
            return Err(ShopError::MatchUnavailable);
        }
        let side = state
            .heroes
            .iter()
            .position(|slot| slot.player_id == player_id)
            .ok_or(ShopError::UnknownPlayer)?;
        (
            state.heroes[side]
                .entity
                .ok_or(ShopError::HeroUnavailable)?,
            state.bases[state.heroes[side].side].ok_or(ShopError::MatchUnavailable)?,
        )
    };
    if !world.entities().is_alive(hero) {
        return Err(ShopError::HeroUnavailable);
    }
    if world
        .read_storage::<PlayerOwner>()
        .get(hero)
        .is_none_or(|owner| owner.player_id != player_id)
    {
        return Err(ShopError::UnknownPlayer);
    }
    {
        let properties = world.read_storage::<super::comp::CProperty>();
        if properties
            .get(hero)
            .is_none_or(|property| property.hp <= Fixed64::ZERO)
        {
            return Err(ShopError::HeroUnavailable);
        }
        let positions = world.read_storage::<Pos>();
        let hero_pos = positions.get(hero).ok_or(ShopError::HeroUnavailable)?.0;
        let base_pos = positions.get(base).ok_or(ShopError::MatchUnavailable)?.0;
    let radius = Fixed64::from_i32(MOBA_SHOP_RADIUS);
        if (hero_pos - base_pos).length_squared() > radius * radius {
            return Err(ShopError::OutsideShop);
        }
    }
    let mut inventories = world.write_storage::<Inventory>();
    let mut balances = world.write_storage::<Gold>();
    let mut effects = world.write_storage::<ItemEffects>();
    let inventory = inventories
        .get_mut(hero)
        .ok_or(ShopError::MissingComponents)?;
    let gold = balances.get_mut(hero).ok_or(ShopError::MissingComponents)?;
    let effect = effects.get_mut(hero).ok_or(ShopError::MissingComponents)?;
    let registry = world.read_resource::<ItemRegistry>();
    match command {
        ShopCommand::Buy(id) => {
            if !omoba_template_ids::MOBA_ITEM_CATALOG
                .iter()
                .any(|item| item.id == id)
            {
                return Err(ShopError::UnknownItem);
            }
            buy_item(&registry, inventory, gold, &id)?;
        }
        ShopCommand::Sell(slot) => {
            sell_item(&registry, inventory, gold, slot)?;
        }
    }
    effect.dirty = true;
    Ok(())
}

/// Item cost is the complete price, not the recipe's additional price. All
/// direct components are required; duplicate IDs consume distinct instances.
pub fn buy_item(
    registry: &ItemRegistry,
    inventory: &mut Inventory,
    gold: &mut Gold,
    item_id: &str,
) -> Result<usize, ShopError> {
    if gold.0 < 0 {
        return Err(ShopError::InvalidBalance);
    }
    let item = registry.get(item_id).ok_or(ShopError::UnknownItem)?;
    if !valid_item(&item) {
        return Err(ShopError::InvalidCatalog);
    }
    let mut next = inventory.clone();
    let mut component_cost = 0_i32;
    for id in &item.recipe {
        if id == item_id {
            return Err(ShopError::InvalidCatalog);
        }
        let component = registry.get(id).ok_or(ShopError::InvalidCatalog)?;
        if !valid_item(&component) {
            return Err(ShopError::InvalidCatalog);
        }
        component_cost = component_cost
            .checked_add(component.cost)
            .ok_or(ShopError::InvalidCatalog)?;
        let slot = next.find_item(id).ok_or(ShopError::MissingComponent)?;
        next.slots[slot] = None;
    }
    let price = item
        .cost
        .checked_sub(component_cost)
        .filter(|cost| *cost >= 0)
        .ok_or(ShopError::InvalidCatalog)?;
    if gold.0 < price {
        return Err(ShopError::InsufficientGold);
    }
    let slot = next.first_free_slot().ok_or(ShopError::InventoryFull)?;
    next.slots[slot] = Some(ItemInstance {
        item_id: item_id.to_owned(),
        cooldown_remaining: 0.0,
    });
    *inventory = next;
    gold.0 -= price;
    Ok(slot)
}

/// Integer floor avoids floating point rounding for large prices. Sales cannot
/// overflow the balance or destroy an item before validation succeeds.
pub fn sell_item(
    registry: &ItemRegistry,
    inventory: &mut Inventory,
    gold: &mut Gold,
    slot: usize,
) -> Result<i32, ShopError> {
    if gold.0 < 0 {
        return Err(ShopError::InvalidBalance);
    }
    let instance = inventory
        .slots
        .get(slot)
        .and_then(Option::as_ref)
        .ok_or(ShopError::InvalidSlot)?;
    let item = registry
        .get(&instance.item_id)
        .ok_or(ShopError::InvalidCatalog)?;
    if !valid_item(&item) {
        return Err(ShopError::InvalidCatalog);
    }
    let refund = item.cost / 2;
    let balance = gold
        .0
        .checked_add(refund)
        .ok_or(ShopError::BalanceOverflow)?;
    inventory.slots[slot] = None;
    gold.0 = balance;
    Ok(refund)
}

#[cfg(test)]
mod tests {
    use super::super::item::{ItemBonus, ItemConfig};
    use super::*;
    fn registry() -> ItemRegistry {
        ItemRegistry::from_configs(vec![
            ItemConfig {
                id: "part".into(),
                name: "Part".into(),
                cost: 31,
                bonus: ItemBonus::default(),
                active: None,
                cooldown: 0.0,
                recipe: vec![],
            },
            ItemConfig {
                id: "upgrade".into(),
                name: "Upgrade".into(),
                cost: 100,
                bonus: ItemBonus::default(),
                active: None,
                cooldown: 0.0,
                recipe: vec!["part".into(), "part".into()],
            },
        ])
    }
    fn snapshot(inventory: &Inventory, gold: &Gold) -> String {
        serde_json::to_string(&(inventory, gold.0)).unwrap()
    }
    #[test]
    fn purchase_sell_and_recipe_charge_exact_integer_prices() {
        let registry = registry();
        let mut inventory = Inventory::default();
        let mut gold = Gold(200);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "part"),
            Ok(0)
        );
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "part"),
            Ok(1)
        );
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "upgrade"),
            Ok(0)
        );
        assert_eq!(gold.0, 100);
        assert_eq!(inventory.items().count(), 1);
        assert_eq!(sell_item(&registry, &mut inventory, &mut gold, 0), Ok(50));
        assert_eq!(gold.0, 150);
        assert_eq!(inventory.items().count(), 0);
    }
    #[test]
    fn full_inventory_can_upgrade_but_cannot_buy_seventh_item() {
        let registry = registry();
        let mut inventory = Inventory::default();
        let mut gold = Gold(1000);
        for index in 0..6 {
            assert_eq!(
                buy_item(&registry, &mut inventory, &mut gold, "part"),
                Ok(index)
            );
        }
        let before = snapshot(&inventory, &gold);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "part"),
            Err(ShopError::InventoryFull)
        );
        assert_eq!(snapshot(&inventory, &gold), before);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "upgrade"),
            Ok(0)
        );
        assert_eq!(inventory.items().count(), 5);
    }
    #[test]
    fn every_rejection_preserves_both_money_and_items() {
        let registry = registry();
        let mut inventory = Inventory::default();
        let mut gold = Gold(31);
        buy_item(&registry, &mut inventory, &mut gold, "part").unwrap();
        let before = snapshot(&inventory, &gold);
        for (id, error) in [
            ("unknown", ShopError::UnknownItem),
            ("upgrade", ShopError::MissingComponent),
            ("part", ShopError::InsufficientGold),
        ] {
            assert_eq!(
                buy_item(&registry, &mut inventory, &mut gold, id),
                Err(error)
            );
            assert_eq!(snapshot(&inventory, &gold), before);
        }
        for slot in [1, 6, usize::MAX] {
            assert_eq!(
                sell_item(&registry, &mut inventory, &mut gold, slot),
                Err(ShopError::InvalidSlot)
            );
            assert_eq!(snapshot(&inventory, &gold), before);
        }
        gold.0 = i32::MAX;
        let before = snapshot(&inventory, &gold);
        assert_eq!(
            sell_item(&registry, &mut inventory, &mut gold, 0),
            Err(ShopError::BalanceOverflow)
        );
        assert_eq!(snapshot(&inventory, &gold), before);
    }
    #[test]
    fn insufficient_recipe_difference_does_not_consume_materials() {
        let registry = registry();
        let mut inventory = Inventory::default();
        let mut gold = Gold(62);
        for _ in 0..2 {
            buy_item(&registry, &mut inventory, &mut gold, "part").unwrap();
        }
        let before = snapshot(&inventory, &gold);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "upgrade"),
            Err(ShopError::InsufficientGold)
        );
        assert_eq!(snapshot(&inventory, &gold), before);
    }
    #[test]
    fn invalid_catalog_never_mutates_inventory() {
        for invalid in 0..7 {
            let mut registry = registry();
            let config = std::sync::Arc::make_mut(registry.items.get_mut("part").unwrap());
            match invalid {
                0 => config.cost = -1,
                1 => config.cost = 0,
                2 => config.bonus.atk = f32::NAN,
                3 => config.cooldown = f32::INFINITY,
                4 => config.recipe = vec!["part".into()],
                5 => config.recipe = vec!["missing".into()],
                _ => config.bonus.hp = -1.0,
            }
            let mut inventory = Inventory::default();
            let mut gold = Gold(1000);
            let before = snapshot(&inventory, &gold);
            assert_eq!(
                buy_item(&registry, &mut inventory, &mut gold, "part"),
                Err(ShopError::InvalidCatalog)
            );
            assert_eq!(snapshot(&inventory, &gold), before);
        }
    }

    #[test]
    fn large_prices_use_exact_integer_refunds_and_recipe_overflow_is_atomic() {
        let mut registry = registry();
        std::sync::Arc::make_mut(registry.items.get_mut("part").unwrap()).cost = i32::MAX;
        let mut inventory = Inventory::default();
        let mut gold = Gold(i32::MAX);
        buy_item(&registry, &mut inventory, &mut gold, "part").unwrap();
        let before = snapshot(&inventory, &gold);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "upgrade"),
            Err(ShopError::InvalidCatalog)
        );
        assert_eq!(snapshot(&inventory, &gold), before);
        assert_eq!(
            sell_item(&registry, &mut inventory, &mut gold, 0),
            Ok(i32::MAX / 2)
        );
        assert_eq!(gold.0, 1_073_741_823);
    }

    #[test]
    fn negative_recipe_difference_and_negative_balance_are_rejected() {
        let mut registry = registry();
        std::sync::Arc::make_mut(registry.items.get_mut("part").unwrap()).cost = 60;
        let mut inventory = Inventory::default();
        let mut gold = Gold(120);
        for _ in 0..2 {
            buy_item(&registry, &mut inventory, &mut gold, "part").unwrap();
        }
        let before = snapshot(&inventory, &gold);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "upgrade"),
            Err(ShopError::InvalidCatalog)
        );
        assert_eq!(snapshot(&inventory, &gold), before);
        gold.0 = -1;
        let before = snapshot(&inventory, &gold);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "part"),
            Err(ShopError::InvalidBalance)
        );
        assert_eq!(
            sell_item(&registry, &mut inventory, &mut gold, 0),
            Err(ShopError::InvalidBalance)
        );
        assert_eq!(snapshot(&inventory, &gold), before);
    }
}
