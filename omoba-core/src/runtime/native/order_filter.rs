//! Order filters across the team views (status_event ORDER_FILTER, 原作 ExecuteOrderFilter).
//!
//! An order filter runs when an enemy's unit order on its holder is about to execute (scripting
//! dispatch, before the windup or the item use) and may refuse it. Its inputs live in two places
//! no team replica holds together: the order is the caster's team's input, the filtering statuses
//! sit on the holder, whose item and status state only the holder's team holds. So the authority
//! alone runs them, and publishes each order they heard, refused or not
//! ([`crate::runtime::ObservableFact::OrderFiltered`]): the caster's team replica refuses a refused
//! cast at the same point of the same tick, and the holder's team replica runs the same hooks on
//! its holder there for every order, so the holder's own effects (a status gone, a shield) match
//! whether or not the order went ahead. What the hooks do to the caster (its cooldown, its mana) reaches
//! its team through the committed cooldown and mana facts, as any other authority-only change.
//! A team that does not see the caster or the holder learns nothing of the order.
use specs::{Entity, World};

const MAGIC: &[u8; 4] = b"ORF1";
/// The longest ability id an order names (an ability or `item:<id>`).
const MAX_ABILITY: usize = 96;

/// The payload one team receives: both units' replica ids for that team (0 for one it does not
/// see), whether the order was refused (1) or went ahead (0), then the ability.
pub fn encode(caster: u64, holder: u64, ability: &str, refused: bool) -> Vec<u8> {
    let mut payload = MAGIC.to_vec();
    payload.extend(caster.to_le_bytes());
    payload.extend(holder.to_le_bytes());
    payload.push(u8::from(refused));
    payload.extend(ability.as_bytes());
    payload
}

/// (caster, holder, ability, refused); `None` for a malformed payload.
pub fn decode(payload: &[u8]) -> Option<(u64, u64, String, bool)> {
    if payload.len() < 21 || payload.len() > 21 + MAX_ABILITY || &payload[..4] != MAGIC { return None; }
    let id = |at: usize| u64::from_le_bytes(payload[at..at + 8].try_into().unwrap());
    let refused = match payload[20] { 0 => false, 1 => true, _ => return None };
    let ability = std::str::from_utf8(&payload[21..]).ok().filter(|ability| !ability.is_empty())?;
    Some((id(4), id(12), ability.to_owned(), refused))
}

/// A team replica's order filters this tick: casts its own casters must not make, and hooks to
/// run on the holders it holds for every order heard (caster `None` when this team does not see
/// the caster).
#[derive(Default)]
pub struct OrderFilterInjections {
    pub refused: Vec<(Entity, String)>,
    pub holders: Vec<(Entity, Option<Entity>, String)>,
}

/// The orders the authority's filters heard this step, published once the dispatch settles:
/// (caster, holder, ability, refused).
pub(crate) fn publish(world: &World, heard: &[(Entity, Entity, String, bool)]) {
    use crate::runtime::{FactAudience, FactKind, FactOrderingKey, FactPhase, ObservableFact, OrderedFact};
    if heard.is_empty() || world.try_fetch::<crate::runtime::MobaMatch>().is_none() { return; }
    let Some(facts) = world.try_fetch::<crate::runtime::ObservableFactBuffer>() else { return };
    let tick = world.try_fetch::<super::comp::Tick>().map_or(0, |tick| tick.0);
    for (ordinal, (caster, holder, ability, refused)) in heard.iter().enumerate() {
        let caster = crate::runtime::canonical_entity_id(*caster);
        facts.emit(OrderedFact {
            key: FactOrderingKey { tick, phase: FactPhase::PostStep, canonical_source_order: caster,
                local_ordinal: u32::try_from(ordinal).unwrap_or(u32::MAX), fact_kind: FactKind::OrderFiltered },
            audience: FactAudience::AllPlayers,
            fact: ObservableFact::OrderFiltered { caster, holder: crate::runtime::canonical_entity_id(*holder), ability: ability.clone(), refused: *refused },
        }).expect("valid order filter fact");
    }
}

/// The order filters in one frame's public events, as this replica's entities (`entity` maps a replica id).
pub(crate) fn injections(events: &[crate::game_proto::TeamPublicEvent], entity: impl Fn(u64) -> Option<Entity>) -> Option<OrderFilterInjections> {
    let mut out = OrderFilterInjections::default();
    for event in events.iter().filter(|event| event.event_kind == crate::runtime::FactKind::OrderFiltered as u32) {
        let (caster, holder, ability, refused) = decode(&event.sanitized_payload)?;
        let caster = (caster != 0).then(|| entity(caster)).flatten();
        if let Some(caster) = caster.filter(|_| refused) { out.refused.push((caster, ability.clone())); }
        if let Some(holder) = (holder != 0).then(|| entity(holder)).flatten() { out.holders.push((holder, caster, ability)); }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_round_trips_and_rejects_malformed_bytes() {
        let bytes = encode(7, 0, "item:item_blink", true);
        assert_eq!(decode(&bytes), Some((7, 0, "item:item_blink".to_owned(), true)));
        assert_eq!(decode(&encode(7, 3, "a01q", false)), Some((7, 3, "a01q".to_owned(), false)));
        assert!(decode(&bytes[..21]).is_none(), "no ability");
        let mut flag = bytes.clone(); flag[20] = 2;
        assert!(decode(&flag).is_none(), "the flag is 0 or 1");
        assert!(decode(&[b"ORF2".as_slice(), &bytes[4..]].concat()).is_none());
        assert!(decode(&encode(1, 2, &"x".repeat(MAX_ABILITY + 1), false)).is_none());
    }

    /// A refused order: the caster's team refuses it and the holder's team runs its hooks. One
    /// that went ahead: only the holder's hooks run.
    #[test]
    fn a_replica_refuses_only_refused_orders_and_reruns_every_heard_one() {
        use crate::game_proto::TeamPublicEvent;
        use specs::{Builder, WorldExt};
        let mut world = World::new();
        let (caster, holder) = (world.create_entity().build(), world.create_entity().build());
        let event = |refused: bool| TeamPublicEvent { event_kind: crate::runtime::FactKind::OrderFiltered as u32, subject: None,
            sanitized_payload: encode(1, 2, "a01q", refused), stable_sub_index: 0 };
        let entity = |id: u64| match id { 1 => Some(caster), 2 => Some(holder), _ => None };
        let rows = injections(&[event(true), event(false)], entity).unwrap();
        assert_eq!(rows.refused, vec![(caster, "a01q".to_owned())]);
        assert_eq!(rows.holders, vec![(holder, Some(caster), "a01q".to_owned()), (holder, Some(caster), "a01q".to_owned())]);
    }
}
