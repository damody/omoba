//! Owner HUD derived exclusively from disclosed state and sanitized HUD metrics.
use omoba_core::{
    game_proto::{AbilityHudPresentation, HeroHudPresentation, MobaHudPresentation},
    runtime::*,
};

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
            mana_supported: false,
        })
    });
    Some(MobaHudPresentation {
        schema_version: 1,
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
