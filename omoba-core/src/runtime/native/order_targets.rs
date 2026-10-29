//! Event source of v3 `on_order_targeted` (status_event ORDER_TARGETED, work line A): a unit was
//! made the target of an accepted order — a hero's attack order, or an ability cast on it. The
//! host records the rows of the current tick in [`OrderTargets`]; the status event dispatch takes
//! them ([`take`]). Rows of an earlier tick are dropped when a new one is recorded, so nothing
//! piles up while no dispatch reads them.
use specs::{Entity, World, WorldExt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderKind { Attack, Cast }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrderTargeted {
    pub tick: u64,
    /// The unit the order names.
    pub target: Entity,
    /// The unit that received the order.
    pub orderer: Entity,
    pub kind: OrderKind,
}

#[derive(Clone, Debug, Default)]
pub struct OrderTargets(pub Vec<OrderTargeted>);

fn now(world: &World) -> u64 {
    world.try_fetch::<crate::comp::Tick>().map_or(0, |tick| tick.0)
}

/// Records an accepted order on `target` (an order on oneself is not targeting).
pub fn record(world: &mut World, orderer: Entity, target: Entity, kind: OrderKind) {
    if orderer == target { return; }
    let tick = now(world);
    if world.try_fetch::<OrderTargets>().is_none() { world.insert(OrderTargets::default()); }
    let mut rows = world.write_resource::<OrderTargets>();
    rows.0.retain(|row| row.tick >= tick);
    rows.0.push(OrderTargeted { tick, target, orderer, kind });
}

/// Takes the recorded rows, in the order the orders were accepted.
pub fn take(world: &World) -> Vec<OrderTargeted> {
    world.try_fetch_mut::<OrderTargets>().map(|mut rows| std::mem::take(&mut rows.0)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    #[test]
    fn rows_of_the_current_tick_are_taken_once_and_older_ones_dropped() {
        let mut world = World::new();
        world.insert(crate::comp::Tick(5));
        let (a, b) = (world.create_entity().build(), world.create_entity().build());
        record(&mut world, a, b, OrderKind::Attack);
        record(&mut world, a, a, OrderKind::Cast);
        world.insert(crate::comp::Tick(6));
        record(&mut world, b, a, OrderKind::Cast);
        assert_eq!(take(&world), vec![OrderTargeted { tick: 6, target: a, orderer: b, kind: OrderKind::Cast }]);
        assert!(take(&world).is_empty());
    }
}
