//! Owner HUD derived exclusively from disclosed state and sanitized HUD metrics.
use omoba_core::{
    game_proto::{
        AbilityHudPresentation, HeroHudPresentation, MobaHudPresentation, SummonerHudPresentation,
    },
    runtime::*,
};

/// Schema 2 introduced mana; 3 negotiated summoners; 4 current filtered attributes.
pub const MANA_SCHEMA_VERSION: u32 = 2;
pub const SUMMONER_SCHEMA_VERSION: u32 = 3;
pub const SCHEMA_VERSION: u32 = 4;
pub const SUMMONER_SHAPE_UNAVAILABLE: u32 = 0;
pub const SUMMONER_SHAPE_SELF: u32 = 1;
pub const SUMMONER_SHAPE_POINT: u32 = 2;
pub const SUMMONER_SHAPE_ENTITY: u32 = 3;

/// Version 1 never carried mana. Schema 2 and 3 share the same mana rules.
pub fn valid_mana(schema: u32, hero: &HeroHudPresentation) -> bool {
    if hero.mana_supported {
        (MANA_SCHEMA_VERSION..=SCHEMA_VERSION).contains(&schema)
            && (0..=1_000_000 * 1024).contains(&hero.max_mana_raw)
            && (0..=hero.max_mana_raw).contains(&hero.mana_raw)
    } else {
        hero.mana_raw == 0 && hero.max_mana_raw == 0
    }
}

pub fn valid_combat_attributes(schema: u32, hero: &HeroHudPresentation) -> bool {
    hero.combat_attributes.as_ref().is_none_or(|value| schema == SCHEMA_VERSION
        && omoba_core::runtime::hero_attributes::valid_attributes(value))
}

pub fn project(snapshot: &FilteredRenderSnapshot, player_id: u32) -> Option<MobaHudPresentation> {
    let metric = |team: u32, id: u64| {
        snapshot.public_events.iter().find_map(|event| {
            let p = &event.sanitized_payload;
            (event.event_kind == FactKind::Hud as u32
                && p.len() == 20
                && p[0..4] == team.to_le_bytes()
                && p[4..12] == id.to_le_bytes())
            .then(|| i64::from_le_bytes(p[12..20].try_into().unwrap()))
        })
    };
    let phase = metric(0, SINGLE_LANE_PHASE_METRIC_ID)?;
    let elapsed = metric(0, SINGLE_LANE_ELAPSED_METRIC_ID)?;
    let winner = metric(0, SINGLE_LANE_WINNER_METRIC_ID).unwrap_or(0);
    if player_id == 0 || !(0..=2).contains(&phase) || elapsed < 0 || !(0..=2).contains(&winner) {
        return None;
    }
    let hero = snapshot.entities.iter().find_map(|entity| {
        let render =
            decode_demo_render_state(entity.components.get(&DEMO_RENDER_COMPONENT_SCHEMA_ID)?)?;
        if render.kind != 1
            || render.team_id != snapshot.team_id
            || render.owner_player_id != player_id
        {
            return None;
        }
        let hero: Hero =
            serde_json::from_slice(entity.components.get(&DISCLOSED_HERO_COMPONENT_SCHEMA_ID)?)
                .ok()?;
        let p = entity
            .components
            .get(&DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID)?;
        if p.len() != 40 {
            return None;
        }
        let raw = |offset| i64::from_be_bytes(p[offset..offset + 8].try_into().unwrap());
        let abilities = hero
            .abilities
            .iter()
            .take(4)
            .enumerate()
            .map(|(slot, id)| {
                let level = hero.ability_levels.get(id).copied().unwrap_or(0).max(0) as u32;
                let total = omoba_template_ids::ability_by_name(id)
                    .and_then(omoba_template_ids::active_ability_const)
                    .and_then(|def| {
                        level
                            .checked_sub(1)
                            .and_then(|index| def.levels.get(index as usize))
                    })
                    .map_or(0, |data| data.cooldown.raw());
                AbilityHudPresentation {
                    slot: slot as u32,
                    ability_id: id.clone(),
                    level,
                    cooldown_raw: hero
                        .ability_cooldowns
                        .get(id)
                        .map_or(0, |value| value.raw().max(0)),
                    total_cooldown_raw: total,
                }
            })
            .collect();
        let (mana_supported, mana_raw, max_mana_raw) = hero.mana_pool.as_ref()
            .map_or((false, 0, 0), |pool| (true, pool.current().raw(), pool.maximum().raw()));
        Some(HeroHudPresentation {
            render_id: entity.replica_id,
            disclosure_epoch: entity.disclosure_epoch,
            content_id: hero.id,
            hp_raw: raw(0),
            max_hp_raw: raw(8),
            level: hero.level.max(0) as u32,
            experience: hero.experience.max(0) as u32,
            skill_points: hero.skill_points.max(0) as u32,
            abilities,
            mana_supported,
            mana_raw,
            max_mana_raw,
            combat_attributes: snapshot.hero_attributes.get(&(entity.replica_id, entity.disclosure_epoch))
                .filter(|attributes| attributes.player_id == player_id && attributes.replica_tick == snapshot.replica_tick
                    && omoba_core::runtime::hero_attributes::valid_attributes(&attributes.values))
                .map(|attributes| attributes.values.clone()),
        })
    });
    Some(MobaHudPresentation {
        schema_version: SCHEMA_VERSION,
        player_id,
        phase: phase as u32,
        elapsed_raw: elapsed,
        winner_team: u32::try_from(winner).ok()?,
        respawn_remaining_raw: metric(snapshot.team_id, single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID, player_id))
            .unwrap_or(0)
            .max(0),
        hero,
        lane_length_raw: metric(0, SINGLE_LANE_LENGTH_METRIC_ID)
            .filter(|value| (2_000 * 1024..=1_000_000 * 1024).contains(value))
            .unwrap_or(0),
        recall_remaining_raw: metric(snapshot.team_id, single_lane_player_metric(SINGLE_LANE_RECALL_REMAINING_METRIC_ID, player_id))
            .unwrap_or(0).clamp(0, i64::from(omoba_template_ids::MOBA_RECALL_CHANNEL_SECONDS) * 1024),
        recall_protocol_enabled: false,
        scoreboard: crate::scoreboard::project(snapshot),
        score: (|| {
            Some(omoba_core::game_proto::OwnerScorePresentation {
                player_id,
                kills: u32::try_from(metric(snapshot.team_id, single_lane_player_metric(SINGLE_LANE_KILLS_METRIC_ID, player_id))?).ok()?,
                deaths: u32::try_from(metric(snapshot.team_id, single_lane_player_metric(SINGLE_LANE_DEATHS_METRIC_ID, player_id))?).ok()?,
                assists: u32::try_from(metric(snapshot.team_id, single_lane_player_metric(SINGLE_LANE_ASSISTS_METRIC_ID, player_id))?).ok()?,
            })
        })(),
        summoner_protocol_enabled: false,
        summoner_slot0: Some(closed_summoner_slot(0)),
        summoner_slot1: Some(closed_summoner_slot(1)),
    })
}

fn catalog_spell(catalog_id: u32) -> Option<&'static omoba_template_ids::SummonerSpellConst> {
    let id = u16::try_from(catalog_id).ok()?;
    omoba_template_ids::SUMMONER_SPELL_CATALOG
        .iter()
        .find(|spell| spell.catalog_id == id)
}

fn shape_and_range(spell: &omoba_template_ids::SummonerSpellConst) -> (u32, i64) {
    match spell.target {
        omoba_template_ids::SummonerTargetConst::None => (SUMMONER_SHAPE_SELF, 0),
        omoba_template_ids::SummonerTargetConst::Point => {
            let range = match spell.operation {
                omoba_template_ids::SummonerSpellOperationConst::PointBlink { range, .. } => range.raw(),
                omoba_template_ids::SummonerSpellOperationConst::SelfHeal { .. } => 0,
            };
            (SUMMONER_SHAPE_POINT, range)
        }
    }
}

fn closed_summoner_slot(slot: u32) -> SummonerHudPresentation {
    SummonerHudPresentation {
        slot,
        catalog_id: 0,
        shape: SUMMONER_SHAPE_UNAVAILABLE,
        range_raw: 0,
        cooldown_raw: 0,
        total_cooldown_raw: 0,
        spell_id: String::new(),
        owner_player_id: 0,
    }
}

fn empty_owner_slot(slot: u32, player_id: u32) -> SummonerHudPresentation {
    SummonerHudPresentation {
        owner_player_id: player_id,
        ..closed_summoner_slot(slot)
    }
}

fn open_row(slot: u32, player_id: u32, catalog_id: u16, cooldown_raw: i64) -> Option<SummonerHudPresentation> {
    if catalog_id == 0 {
        return (cooldown_raw == 0).then(|| empty_owner_slot(slot, player_id));
    }
    let spell = catalog_spell(u32::from(catalog_id))?;
    let (shape, range_raw) = shape_and_range(spell);
    let total = spell.cooldown.raw();
    (0..=total).contains(&cooldown_raw).then(|| SummonerHudPresentation {
        slot,
        catalog_id: u32::from(catalog_id),
        shape,
        range_raw,
        cooldown_raw,
        total_cooldown_raw: total,
        spell_id: spell.id.to_owned(),
        owner_player_id: player_id,
    })
}

fn row_closed(slot: u32, row: Option<&SummonerHudPresentation>) -> bool {
    row.is_none_or(|row| row == &closed_summoner_slot(slot))
}

fn valid_open_row(player_id: u32, slot: u32, row: &SummonerHudPresentation) -> bool {
    if row.slot != slot || row.owner_player_id != player_id || player_id == 0 || row.shape == SUMMONER_SHAPE_ENTITY {
        return false;
    }
    if row.catalog_id == 0 {
        return row.shape == SUMMONER_SHAPE_UNAVAILABLE
            && row.range_raw == 0
            && row.cooldown_raw == 0
            && row.total_cooldown_raw == 0
            && row.spell_id.is_empty();
    }
    let Some(spell) = catalog_spell(row.catalog_id) else { return false; };
    let (shape, range) = shape_and_range(spell);
    row.shape == shape
        && row.range_raw == range
        && row.total_cooldown_raw == spell.cooldown.raw()
        && row.spell_id == spell.id
        && (0..=row.total_cooldown_raw).contains(&row.cooldown_raw)
}

/// Schema below 3, or a disabled flag, accepts only canonical closed slots.
/// An enabled schema-3 row must match the authored catalog and the owner.
pub fn valid_summoner_rows(hud: &MobaHudPresentation) -> bool {
    let open = (SUMMONER_SCHEMA_VERSION..=SCHEMA_VERSION).contains(&hud.schema_version) && hud.summoner_protocol_enabled;
    if !open {
        return !hud.summoner_protocol_enabled
            && row_closed(0, hud.summoner_slot0.as_ref())
            && row_closed(1, hud.summoner_slot1.as_ref());
    }
    match (&hud.summoner_slot0, &hud.summoner_slot1) {
        (Some(slot0), Some(slot1)) => {
            valid_open_row(hud.player_id, 0, slot0) && valid_open_row(hud.player_id, 1, slot1)
        }
        _ => false,
    }
}

/// Admitted owner rows. `None` means the roster itself is not publishable.
/// A missing owner is canonical empty equipment, not another player's row.
pub fn disclosed_summoner_rows(
    snapshot: &FilteredRenderSnapshot,
    player_id: u32,
) -> Option<(SummonerHudPresentation, SummonerHudPresentation)> {
    if player_id == 0 {
        return Some((closed_summoner_slot(0), closed_summoner_slot(1)));
    }
    if !matches!(snapshot.team_id, 1 | 2) {
        return Some((empty_owner_slot(0, player_id), empty_owner_slot(1, player_id)));
    }
    let mut seen = std::collections::BTreeSet::new();
    for row in &snapshot.disclosed_summoners.players {
        if row.player_id == 0 || !seen.insert(row.player_id) {
            return None;
        }
    }
    let Some(row) = snapshot.disclosed_summoners.players.iter().find(|row| row.player_id == player_id) else {
        return Some((empty_owner_slot(0, player_id), empty_owner_slot(1, player_id)));
    };
    Some((
        open_row(0, player_id, row.slots[0].catalog_id, row.slots[0].cooldown_raw)?,
        open_row(1, player_id, row.slots[1].catalog_id, row.slots[1].cooldown_raw)?,
    ))
}

/// Capability is independent of recall and shop. A bad roster closes the rows
/// and leaves the rest of the HUD in place. Clocks are copied, never advanced.
pub fn gate_summoner_capability(hud: &mut MobaHudPresentation, snapshot: &FilteredRenderSnapshot, enabled: bool) {
    let close = |hud: &mut MobaHudPresentation| {
        hud.summoner_protocol_enabled = false;
        hud.summoner_slot0 = Some(closed_summoner_slot(0));
        hud.summoner_slot1 = Some(closed_summoner_slot(1));
    };
    if !(SUMMONER_SCHEMA_VERSION..=SCHEMA_VERSION).contains(&hud.schema_version) || !enabled {
        close(hud);
        return;
    }
    let Some((slot0, slot1)) = disclosed_summoner_rows(snapshot, hud.player_id) else {
        close(hud);
        return;
    };
    if slot0.owner_player_id == 0 || slot1.owner_player_id == 0 {
        close(hud);
        return;
    }
    hud.summoner_protocol_enabled = true;
    hud.summoner_slot0 = Some(slot0);
    hud.summoner_slot1 = Some(slot1);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mana_hud_is_owner_scoped_and_distinguishes_disabled_from_empty() {
        use omoba_core::runtime::ability_runtime::ManaPool;
        let mut s = snapshot();
        let mut owner = entity(3, 1, 7);
        let mut hero: Hero = serde_json::from_slice(&owner.components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        for pool in [None, Some(ManaPool::new(Fixed64::ZERO, Fixed64::from_i32(280)).unwrap()),
            Some(ManaPool::from_raw_state(45123, 280 * 1024, 17).unwrap())] {
            hero.mana_pool = pool.clone();
            owner.components.insert(DISCLOSED_HERO_COMPONENT_SCHEMA_ID, serde_json::to_vec(&hero).unwrap());
            s.entities = vec![entity(1, 2, 7), entity(2, 1, 8), owner.clone()];
            let hud = project(&s, 7).unwrap();
            assert_eq!(hud.schema_version, SCHEMA_VERSION);
            let projected = hud.hero.unwrap();
            assert_eq!(projected.render_id, 3);
            assert_eq!(projected.mana_supported, pool.is_some());
            assert_eq!((projected.mana_raw, projected.max_mana_raw),
                pool.as_ref().map_or((0,0), |p| (p.current().raw(), p.maximum().raw())));
            assert!(valid_mana(SCHEMA_VERSION, &projected));
            assert!(valid_mana(MANA_SCHEMA_VERSION, &projected));
        }
        s.entities.retain(|e| e.replica_id != 3);
        assert!(project(&s, 7).unwrap().hero.is_none(), "no stale dead/hidden owner mana");
    }

    #[test]
    fn mana_hud_rejects_invalid_ranges_and_legacy_managed_payloads() {
        let mut hero = HeroHudPresentation {mana_supported: true, mana_raw: 0, max_mana_raw: 280*1024, ..Default::default()};
        assert!(valid_mana(SCHEMA_VERSION, &hero));
        assert!(valid_mana(MANA_SCHEMA_VERSION, &hero));
        assert!(!valid_mana(1, &hero));
        for (current, maximum) in [(-1,280*1024),(280*1024+1,280*1024),(0,-1),(0,1_000_000*1024+1)] {
            hero.mana_raw = current; hero.max_mana_raw = maximum;
            assert!(!valid_mana(SCHEMA_VERSION, &hero));
        }
        hero.mana_supported = false; hero.mana_raw = 0; hero.max_mana_raw = 0;
        assert!(valid_mana(1, &hero));
        hero.max_mana_raw = 1;
        assert!(!valid_mana(SCHEMA_VERSION, &hero));
    }
    #[test]
    fn recall_projection_is_owner_team_only_and_legacy_capability_stays_closed() {
        let mut s = snapshot();
        s.public_events.extend([
            metric(2, single_lane_player_metric(SINGLE_LANE_RECALL_REMAINING_METRIC_ID, 7), 7 * 1024),
            metric(1, single_lane_player_metric(SINGLE_LANE_RECALL_REMAINING_METRIC_ID, 7), 3 * 1024),
            metric(1, single_lane_player_metric(SINGLE_LANE_RECALL_REMAINING_METRIC_ID, 8), 5 * 1024),
        ]);
        let hud = project(&s, 7).unwrap();
        assert_eq!(hud.recall_remaining_raw, 3 * 1024);
        assert_eq!(project(&s, 8).unwrap().recall_remaining_raw, 5 * 1024);
        assert!(!hud.recall_protocol_enabled);
        s.public_events.retain(|e| e.sanitized_payload[0..4] != 1_u32.to_le_bytes());
        assert_eq!(project(&s, 7).unwrap().recall_remaining_raw, 0);
    }
    use omoba_core::game_proto::TeamPublicEvent;
    use omoba_sim::Fixed64;
    use std::collections::BTreeMap;

    #[test]
    fn score_is_persistent_player_scoped_and_absent_or_invalid_is_not_zero() {
        let mut s = snapshot();
        assert!(project(&s,7).unwrap().score.is_none());
        for (team,player,values) in [(1,7,[2,3,4]),(1,8,[9,8,7]),(2,7,[99,99,99])] {
            for (ns,value) in [SINGLE_LANE_KILLS_METRIC_ID,SINGLE_LANE_DEATHS_METRIC_ID,SINGLE_LANE_ASSISTS_METRIC_ID].into_iter().zip(values) {
                s.public_events.push(metric(team,single_lane_player_metric(ns,player),value));
            }
        }
        let score = project(&s,7).unwrap().score.unwrap();
        assert_eq!((score.player_id,score.kills,score.deaths,score.assists),(7,2,3,4));
        assert_eq!(project(&s,8).unwrap().score.unwrap().kills,9);
        assert!(project(&s,9).unwrap().score.is_none());
        // No live entity is needed: death and a reconnect baseline retain score.
        assert!(s.entities.is_empty());
        for invalid in [-1,i64::from(u32::MAX)+1] {
            let id = single_lane_player_metric(SINGLE_LANE_KILLS_METRIC_ID,7);
            s.public_events.retain(|e| !(e.sanitized_payload[0..4]==1_u32.to_le_bytes() && e.sanitized_payload[4..12]==id.to_le_bytes()));
            s.public_events.push(metric(1,id,invalid));
            assert!(project(&s,7).unwrap().score.is_none());
        }
    }

    #[test]
    fn teammate_respawn_metrics_never_overwrite_owner_and_namespace_preserves_full_id() {
        let mut s = snapshot();
        s.public_events.extend([
            metric(1,single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID,8),9000),
            metric(1,single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID,7),5000),
            metric(1,single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID,u32::MAX),3000),
        ]);
        assert_eq!(project(&s,7).unwrap().respawn_remaining_raw,5000);
        assert_eq!(project(&s,8).unwrap().respawn_remaining_raw,9000);
        assert_eq!(project(&s,u32::MAX).unwrap().respawn_remaining_raw,3000);
        assert_eq!(project(&s,9).unwrap().respawn_remaining_raw,0);
        for ns in [SINGLE_LANE_RESPAWN_METRIC_ID,SINGLE_LANE_RECALL_ACTIVE_METRIC_ID,SINGLE_LANE_RECALL_REMAINING_METRIC_ID] {
            assert_eq!(single_lane_metric_player(single_lane_player_metric(ns,u32::MAX),ns),Some(u32::MAX));
            assert_eq!(single_lane_metric_player(ns,ns),None);
        }
    }

    fn metric(team: u32, id: u64, value: i64) -> TeamPublicEvent {
        TeamPublicEvent {
            event_kind: FactKind::Hud as u32,
            sanitized_payload: [
                team.to_le_bytes().as_slice(),
                id.to_le_bytes().as_slice(),
                value.to_le_bytes().as_slice(),
            ]
            .concat(),
            ..Default::default()
        }
    }
    fn snapshot() -> FilteredRenderSnapshot {
        FilteredRenderSnapshot {
            team_id: 1,
            replica_tick: 99,
            entities: vec![],
            public_events: vec![
                metric(0, SINGLE_LANE_PHASE_METRIC_ID, 1),
                metric(0, SINGLE_LANE_ELAPSED_METRIC_ID, 10240),
            ],
            external_effects: vec![],
            memory_directives: vec![],
            remembered_presentations: BTreeMap::new(),
            hero_attributes: Default::default(), disclosed_summoners: Default::default(), authoritative_mode: None, combat_preferences: None,
        }
    }
    fn entity(id: u64, team: u32, player: u32) -> FilteredRenderEntity {
        let mut hero = Hero::new("training_hero".into(), "Training".into(), "".into());
        hero.abilities = (0..4).map(|slot| format!("slot_{slot}")).collect();
        for ability in &hero.abilities {
            hero.ability_levels.insert(ability.clone(), 1);
            hero.ability_cooldowns
                .insert(ability.clone(), Fixed64::from_i32(3));
        }
        let property = [
            Fixed64::from_i32(40).raw(),
            Fixed64::from_i32(100).raw(),
            0,
            0,
            0,
        ]
        .into_iter()
        .flat_map(i64::to_be_bytes)
        .collect();
        FilteredRenderEntity {
            replica_id: id,
            disclosure_epoch: 8,
            entity_kind: 1,
            components: BTreeMap::from([
                (
                    DEMO_RENDER_COMPONENT_SCHEMA_ID,
                    encode_demo_render_state(DemoRenderState {
                        x_raw: 0,
                        y_raw: 0,
                        team_id: team,
                        kind: 1,
                        owner_player_id: player,
                    }),
                ),
                (
                    DISCLOSED_HERO_COMPONENT_SCHEMA_ID,
                    serde_json::to_vec(&hero).unwrap(),
                ),
                (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property),
            ]),
        }
    }
    #[test]
    fn owner_hud_preserves_unlearned_rank_and_first_learning_without_prediction() {
        let mut snapshot = snapshot();
        let mut owner = entity(3,1,7);
        let mut hero:Hero = serde_json::from_slice(&owner.components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        for rank in [0,1] {
            hero.ability_levels.insert("slot_0".into(),rank);
            hero.skill_points = 1-rank;
            owner.components.insert(DISCLOSED_HERO_COMPONENT_SCHEMA_ID,serde_json::to_vec(&hero).unwrap());
            snapshot.entities = vec![owner.clone()];
            let hud = project(&snapshot,7).unwrap().hero.unwrap();
            assert_eq!(hud.abilities[0].level,rank as u32);
            assert_eq!(hud.skill_points,(1-rank) as u32);
            assert_eq!(hud.abilities[0].cooldown_raw,3072,"projection does not reset cooldown");
        }
    }

    #[test]
    fn owner_only_hud_ignores_enemy_and_teammate_order() {
        let mut snapshot = snapshot();
        snapshot.entities = vec![entity(1, 2, 7), entity(2, 1, 8), entity(3, 1, 7)];
        let hud = project(&snapshot, 7).unwrap();
        let hero = hud.hero.unwrap();
        assert_eq!((hero.render_id, hero.disclosure_epoch), (3, 8));
        assert_eq!((hero.hp_raw, hero.max_hp_raw), (40960, 102400));
        assert!(!hero.mana_supported);
        assert_eq!(hero.abilities.len(), 4);
        for (slot, ability) in hero.abilities.iter().enumerate() {
            assert_eq!(ability.slot, slot as u32);
            assert_eq!(ability.cooldown_raw, 3072);
        }
        assert!(project(&snapshot, 9).unwrap().hero.is_none());
    }
    #[test]
    fn death_and_finished_states_are_persistent_without_a_live_hero() {
        let mut snapshot = snapshot();
        snapshot
            .public_events
            .push(metric(1, single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID, 7), 5120));
        snapshot
            .public_events
            .push(metric(2, single_lane_player_metric(SINGLE_LANE_RESPAWN_METRIC_ID, 7), 99999));
        let hud = project(&snapshot, 7).unwrap();
        assert!(hud.hero.is_none());
        assert_eq!(hud.respawn_remaining_raw, 5120);
        snapshot.public_events[0] = metric(0, SINGLE_LANE_PHASE_METRIC_ID, 2);
        snapshot
            .public_events
            .push(metric(0, SINGLE_LANE_WINNER_METRIC_ID, 2));
        assert_eq!(project(&snapshot, 7).unwrap().winner_team, 2);
        assert_eq!(project(&snapshot.clone(), 7), project(&snapshot, 7));
    }
    #[test]
    fn current_attributes_require_exact_owner_body_epoch_and_snapshot_tick() {
        use omoba_core::{game_proto::HeroCombatAttributesPresentation, runtime::hero_attributes::DisclosedHeroAttributes};
        let mut source = snapshot(); source.entities.push(entity(3,1,7));
        let values = HeroCombatAttributesPresentation { attack_damage_raw: 50*1024, physical_armor_raw: -3*1024,
            magic_resistance_raw: 256, move_speed_raw: 300*1024, attack_range_raw: 550*1024, attack_interval_raw: 512 };
        source.hero_attributes.insert((3,8),DisclosedHeroAttributes { player_id: 7, replica_tick: source.replica_tick, values: values.clone() });
        assert_eq!(project(&source,7).unwrap().hero.unwrap().combat_attributes,Some(values));
        let mut wrong = source.clone();wrong.hero_attributes.get_mut(&(3,8)).unwrap().player_id=8;
        assert!(project(&wrong,7).unwrap().hero.unwrap().combat_attributes.is_none());
        let mut wrong = source.clone();wrong.replica_tick+=1;
        assert!(project(&wrong,7).unwrap().hero.unwrap().combat_attributes.is_none());
        let mut wrong = source.clone();wrong.entities[0].disclosure_epoch+=1;
        assert!(project(&wrong,7).unwrap().hero.unwrap().combat_attributes.is_none());
        let mut wrong = source.clone();wrong.hero_attributes.get_mut(&(3,8)).unwrap().values.magic_resistance_raw=1025;
        assert!(project(&wrong,7).unwrap().hero.unwrap().combat_attributes.is_none());
        source.entities.clear();assert!(project(&source,7).unwrap().hero.is_none());
    }

    #[test]
    fn attribute_absence_and_explicit_zero_survive_proto_with_legacy_schema_rules() {
        use prost::Message;
        use omoba_core::game_proto::HeroCombatAttributesPresentation;
        let mut hero = HeroHudPresentation::default();
        for version in [1,MANA_SCHEMA_VERSION,SUMMONER_SCHEMA_VERSION,SCHEMA_VERSION] { assert!(valid_combat_attributes(version,&hero)); }
        hero.combat_attributes=Some(HeroCombatAttributesPresentation::default());
        assert!(valid_combat_attributes(SCHEMA_VERSION,&hero));
        for version in [1,MANA_SCHEMA_VERSION,SUMMONER_SCHEMA_VERSION] { assert!(!valid_combat_attributes(version,&hero)); }
        let decoded=HeroHudPresentation::decode(hero.encode_to_vec().as_slice()).unwrap();
        assert!(decoded.combat_attributes.is_some(),"zero is still an explicitly provided message");
        hero.combat_attributes.as_mut().unwrap().attack_interval_raw=-1;
        assert!(!valid_combat_attributes(SCHEMA_VERSION,&hero));
    }

    #[test]
    fn malformed_or_non_moba_control_does_not_invent_hud() {
        let mut snapshot = snapshot();
        assert!(project(&snapshot, 0).is_none());
        snapshot.entities.push(entity(3, 1, 7));
        snapshot.entities[0]
            .components
            .insert(DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, vec![0; 39]);
        assert!(project(&snapshot, 7).unwrap().hero.is_none());
        snapshot.public_events[0].sanitized_payload.truncate(19);
        assert!(project(&snapshot, 7).is_none());
        snapshot.public_events.clear();
        assert!(project(&snapshot, 7).is_none());
    }
    #[test]
    fn static_map_length_is_public_validated_and_independent_of_units() {
        let mut s = snapshot();
        assert_eq!(project(&s, 7).unwrap().lane_length_raw, 0);
        s.public_events.push(metric(2, SINGLE_LANE_LENGTH_METRIC_ID, 9_000 * 1024));
        assert_eq!(project(&s, 7).unwrap().lane_length_raw, 0, "team-private data is not map metadata");
        s.public_events.push(metric(0, SINGLE_LANE_LENGTH_METRIC_ID, 3_200 * 1024));
        assert_eq!(project(&s, 7).unwrap().lane_length_raw, 3_200 * 1024);
        s.entities.push(entity(33, 2, 99));
        assert_eq!(project(&s, 7).unwrap().lane_length_raw, 3_200 * 1024);
        for length in [-1, 0, 1_999 * 1024, 1_000_001 * 1024, i64::MAX] {
            *s.public_events.last_mut().unwrap() = metric(0, SINGLE_LANE_LENGTH_METRIC_ID, length);
            assert_eq!(project(&s, 7).unwrap().lane_length_raw, 0);
        }
    }

    fn projected_player(player: u32, catalog: [u16; 2], cooldown: [i64; 2]) -> omoba_core::runtime::summoner_transport::ProjectedSummonerPlayer {
        use omoba_core::runtime::summoner_transport::{ProjectedSummonerPlayer, ProjectedSummonerSlot};
        ProjectedSummonerPlayer {
            player_id: player,
            slots: [0, 1].map(|index| ProjectedSummonerSlot {
                catalog_id: catalog[index],
                cooldown_raw: cooldown[index],
            }),
        }
    }

    #[test]
    fn summoner_rows_require_schema_3_and_reject_bad_catalog_clock_shape_and_owner() {
        use omoba_core::game_proto::SummonerHudPresentation;
        let blink = omoba_template_ids::summoner_spell_by_name("training_point_blink_v1").unwrap();
        let heal = omoba_template_ids::summoner_spell_by_name("training_self_heal_v1").unwrap();
        let mut snapshot = snapshot();
        let closed = project(&snapshot, 7).unwrap();
        assert_eq!(closed.schema_version, SCHEMA_VERSION);
        assert!(!closed.summoner_protocol_enabled);
        assert!(valid_summoner_rows(&closed));
        let mut legacy = closed.clone();
        legacy.schema_version = MANA_SCHEMA_VERSION;
        assert!(valid_summoner_rows(&legacy));
        legacy.summoner_protocol_enabled = true;
        assert!(!valid_summoner_rows(&legacy));
        legacy.summoner_protocol_enabled = false;
        legacy.summoner_slot0 = Some(SummonerHudPresentation { catalog_id: 1, owner_player_id: 7, ..closed_summoner_slot(0) });
        assert!(!valid_summoner_rows(&legacy));

        snapshot.disclosed_summoners.players = vec![projected_player(7, [1, 2], [0, heal.cooldown.raw()])];
        let mut hud = project(&snapshot, 7).unwrap();
        assert!(hud.hero.is_none());
        gate_summoner_capability(&mut hud, &snapshot, true);
        let again = hud.clone();
        gate_summoner_capability(&mut hud, &snapshot, true);
        assert_eq!(hud, again, "a second projection does not tick the authority clock");
        assert!(hud.summoner_protocol_enabled);
        assert!(valid_summoner_rows(&hud));
        let slot0 = hud.summoner_slot0.as_ref().unwrap();
        let slot1 = hud.summoner_slot1.as_ref().unwrap();
        assert_eq!((slot0.catalog_id, slot0.shape, slot0.spell_id.as_str()), (1, SUMMONER_SHAPE_POINT, blink.id));
        assert_eq!(slot0.range_raw, match blink.operation {
            omoba_template_ids::SummonerSpellOperationConst::PointBlink { range, .. } => range.raw(),
            _ => panic!("blink range"),
        });
        assert_eq!(slot0.total_cooldown_raw, blink.cooldown.raw());
        assert_eq!((slot1.catalog_id, slot1.shape, slot1.range_raw, slot1.cooldown_raw), (2, SUMMONER_SHAPE_SELF, 0, heal.cooldown.raw()));
        assert_eq!(slot1.owner_player_id, 7);

        let mut bad = hud.clone();
        bad.summoner_slot0.as_mut().unwrap().catalog_id = 99;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot1.as_mut().unwrap().slot = 0;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot1.as_mut().unwrap().cooldown_raw = -1;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot1.as_mut().unwrap().cooldown_raw = heal.cooldown.raw() + 1;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot0.as_mut().unwrap().shape = SUMMONER_SHAPE_ENTITY;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot0.as_mut().unwrap().shape = SUMMONER_SHAPE_SELF;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot0.as_mut().unwrap().owner_player_id = 8;
        assert!(!valid_summoner_rows(&bad));
        bad = hud.clone();
        bad.summoner_slot0 = Some(empty_owner_slot(0, 7));
        assert!(valid_summoner_rows(&bad), "catalog 0 is canonical unavailable");

        snapshot.disclosed_summoners.players = vec![projected_player(7, [99, 2], [0, 0])];
        let mut rejected = project(&snapshot, 7).unwrap();
        gate_summoner_capability(&mut rejected, &snapshot, true);
        assert!(!rejected.summoner_protocol_enabled);
        assert_eq!(rejected.phase, 1);
        snapshot.disclosed_summoners.players = vec![
            projected_player(7, [1, 2], [0, 0]),
            projected_player(7, [1, 0], [0, 0]),
        ];
        gate_summoner_capability(&mut rejected, &snapshot, true);
        assert!(!rejected.summoner_protocol_enabled);
        snapshot.disclosed_summoners.players = vec![projected_player(7, [0, 0], [1, 0])];
        gate_summoner_capability(&mut rejected, &snapshot, true);
        assert!(!rejected.summoner_protocol_enabled, "unequipped catalog cannot carry a clock");
    }
}
