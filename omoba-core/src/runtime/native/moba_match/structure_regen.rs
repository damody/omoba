//! Structure HP regeneration on the deterministic, pause-aware match clock (E187: the original
//! StatusHealthRegen of npc_units_custom_load.txt: general towers and the main base 20 HP/s).
use super::*;

impl MobaMatch {
    /// Every living structure below its maximum regains its kind's map-authored HP per second for
    /// `dt` of play, up to the maximum. Paused and finished matches regenerate nothing.
    pub(super) fn regenerate_structures(&self, world: &mut World, dt: Fixed64) {
        if self.phase != MobaMatchPhase::Playing || dt <= Fixed64::ZERO
            || world.read_resource::<GamePause>().is_paused { return; }
        let stats = self.structure_stats();
        let mut healed = Vec::new();
        let mut bodies = world.write_storage::<CProperty>();
        for unit in self.units.values() {
            let rate = match unit.role {
                LaneRole::Tower => stats.tower_hp_regen,
                LaneRole::Inhibitor => stats.inhibitor_hp_regen,
                LaneRole::NexusTower => stats.nexus_tower_hp_regen,
                LaneRole::Base => stats.base_hp_regen,
                _ => continue,
            };
            if rate <= 0 { continue; }
            let Some(body) = bodies.get_mut(unit.entity) else { continue };
            if body.hp <= Fixed64::ZERO || body.hp >= body.mhp { continue; }
            body.hp = (body.hp + Fixed64::from_i32(rate) * dt).min(body.mhp);
            healed.push(unit.entity);
        }
        drop(bodies);
        // Regeneration bypasses the outcome pipeline, so it publishes the same committed vitals
        // a heal outcome would; otherwise the team replicas keep the old HP.
        crate::runtime::native::game_processor::emit_committed_vitals(world, healed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(map_id: &str) -> World {
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        let map = omoba_template_ids::moba_map_by_name(map_id).unwrap();
        setup_single_lane_match(&mut world, SingleLaneConfig { map_id: Some(map_id.into()),
            lane_length: Fixed64::from_i32(map.lane_length), warmup: Fixed64::ZERO,
            authored_minion_waves: true, ..SingleLaneConfig::default() }).unwrap();
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
        world
    }
    /// One living structure of each kind for side 1, each set `missing` HP below its maximum.
    fn wound(world: &mut World, missing: i32) -> Vec<(LaneRole, Entity)> {
        let state = world.read_resource::<MobaMatch>();
        let rows: Vec<_> = [LaneRole::Tower, LaneRole::Inhibitor, LaneRole::NexusTower, LaneRole::Base].iter()
            .map(|role| (*role, state.units.values().find(|u| u.team == 1 && u.role == *role).unwrap().entity)).collect();
        drop(state);
        let mut bodies = world.write_storage::<CProperty>();
        for (_, entity) in &rows { let body = bodies.get_mut(*entity).unwrap(); body.hp = body.mhp - Fixed64::from_i32(missing); }
        rows
    }
    fn missing(world: &World, entity: Entity) -> Fixed64 {
        let body = world.read_storage::<CProperty>().get(entity).unwrap().clone();
        body.mhp - body.hp
    }

    #[test]
    fn general_towers_and_the_base_regenerate_the_map_rate_up_to_their_maximum() {
        for map_id in ["three_lane_training", "three_lane_quick_training"] {
            let stats = omoba_template_ids::moba_map_by_name(map_id).unwrap().structure_stats();
            let rate = |role: LaneRole| match role {
                LaneRole::Tower => stats.tower_hp_regen, LaneRole::Inhibitor => stats.inhibitor_hp_regen,
                LaneRole::NexusTower => stats.nexus_tower_hp_regen, _ => stats.base_hp_regen };
            let mut world = fixture(map_id);
            // Less than any quick-map structure's HP.
            let rows = wound(&mut world, 100);
            let state = world.remove::<MobaMatch>().unwrap();
            let _ = world.read_resource::<crate::runtime::ObservableFactBuffer>().drain_ordered();
            // Half a second at a time, on the deterministic clock.
            state.regenerate_structures(&mut world, Fixed64::from_raw(512));
            state.regenerate_structures(&mut world, Fixed64::from_raw(512));
            for (role, entity) in &rows {
                assert_eq!(missing(&world, *entity), Fixed64::from_i32(100 - rate(*role)), "{map_id} {role:?}");
            }
            // Every regenerating structure publishes its new HP like a heal outcome would, so the team
            // replicas follow; structures that do not regenerate publish nothing.
            let facts = world.read_resource::<crate::runtime::ObservableFactBuffer>().drain_ordered().unwrap();
            for (role, entity) in &rows {
                let id = crate::runtime::canonical_entity_id(*entity);
                let hp = world.read_storage::<CProperty>().get(*entity).unwrap().hp.raw();
                let published = facts.iter().filter_map(|fact| match fact.fact {
                    crate::runtime::ObservableFact::CommittedVitals { source, hp_raw, .. } if source == id => Some(hp_raw),
                    _ => None,
                }).last();
                assert_eq!(published, (rate(*role) > 0).then_some(hp), "{map_id} {role:?}");
            }
            // Capped at the maximum.
            state.regenerate_structures(&mut world, Fixed64::from_i32(1000));
            for (role, entity) in &rows {
                assert_eq!(missing(&world, *entity), if rate(*role) > 0 { Fixed64::ZERO } else { Fixed64::from_i32(100) }, "{map_id} {role:?}");
            }
        }
        let stats = omoba_template_ids::moba_map_by_name("three_lane_training").unwrap().structure_stats();
        assert!(stats.nexus_tower_hp_regen > 0 && stats.base_hp_regen > 0 && stats.tower_hp_regen == 0 && stats.inhibitor_hp_regen == 0);
    }

    #[test]
    fn regeneration_runs_on_the_match_tick_and_stops_when_paused_finished_or_dead() {
        let mut world = fixture("three_lane_training");
        let rate = omoba_template_ids::moba_map_by_name("three_lane_training").unwrap().structure_stats().nexus_tower_hp_regen;
        let rows = wound(&mut world, 1000);
        let general = rows.iter().find(|(role, _)| *role == LaneRole::NexusTower).unwrap().1;
        world.write_resource::<DeltaTime>().0 = Fixed64::ONE;
        assert!(begin_moba_match_tick(&mut world));
        assert_eq!(missing(&world, general), Fixed64::from_i32(1000 - rate));
        // Paused: the clock and HP stand still.
        world.write_resource::<GamePause>().is_paused = true;
        assert!(!begin_moba_match_tick(&mut world));
        assert_eq!(missing(&world, general), Fixed64::from_i32(1000 - rate));
        world.write_resource::<GamePause>().is_paused = false;
        // A destroyed structure (0 HP) never comes back by regeneration.
        world.write_storage::<CProperty>().get_mut(general).unwrap().hp = Fixed64::ZERO;
        assert!(begin_moba_match_tick(&mut world));
        assert_eq!(world.read_storage::<CProperty>().get(general).unwrap().hp, Fixed64::ZERO);
        // Finished: nothing regenerates.
        let base = rows.iter().find(|(role, _)| *role == LaneRole::Base).unwrap().1;
        let before = missing(&world, base);
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Finished { winner: Some(0), tick: 1 };
        assert!(!begin_moba_match_tick(&mut world));
        assert_eq!(missing(&world, base), before);
    }
}
