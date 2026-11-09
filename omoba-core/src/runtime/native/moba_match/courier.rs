//! 信使（E 線信使規格 courier-spec-2026-11-09 §1；原作 npc_dota_courier2「馬」，v3 summon
//! `courier`）：`shop_range_rule` 的比賽裡每位玩家一隻。英雄第一次生成時在英雄的出生點跟著
//! 生成（原作 eventforspawned.lua:38-57），主人是該英雄（SummonedUnit summoner；英雄重生後
//! 改指向新的身體，主人死亡時信使留著）。被殺時的賞金照 summon_bounty（模板 bounty_gold 150），
//! 下一個 tick 就在出生點重生（原作 eventforkill.lua:540-557：沒有重生計時）。送貨、回家等指令
//! 與運送是 B 線的。
use super::*;

/// The v3 summon template of a courier (pilot_b unit template `courier`).
pub(crate) const COURIER_SUMMON: &str = "courier";

/// After `index`'s hero spawned at `spawn`: the player's first courier there, or its living
/// courier now follows the new body.
pub(super) fn on_hero_spawn(state: &mut MobaMatch, world: &mut World, index: usize, spawn: Vec2) {
    if !state.config.shop_range_rule { return; }
    let Some(hero) = state.heroes[index].entity else { return };
    if state.heroes[index].courier_spawn.is_none() { state.heroes[index].courier_spawn = Some(spawn); }
    match state.heroes[index].courier.filter(|courier| alive(world, *courier)) {
        Some(courier) => {
            if let Some(summon) = world.write_storage::<SummonedUnit>().get_mut(courier) { summon.summoner = hero; }
        }
        None => spawn_courier(state, world, index),
    }
}

/// Each tick: a killed (or retired) courier comes back at once at its spawn point.
pub(super) fn tick(world: &mut World, state: &mut MobaMatch) {
    if !state.config.shop_range_rule { return; }
    for index in 0..state.heroes.len() {
        // its hero has not spawned yet: the first courier comes with it
        if state.heroes[index].courier_spawn.is_none() { continue; }
        if state.heroes[index].courier.is_some_and(|courier| alive(world, courier)) { continue; }
        spawn_courier(state, world, index);
    }
}

/// The player's courier, if one stands now.
pub(crate) fn courier_of(world: &World, state: &MobaMatch, player_id: u32) -> Option<Entity> {
    state.heroes.iter().find(|slot| slot.player_id == player_id)
        .and_then(|slot| slot.courier).filter(|courier| alive(world, *courier))
}

fn alive(world: &World, courier: Entity) -> bool {
    world.entities().is_alive(courier)
        && world.read_storage::<CProperty>().get(courier).is_some_and(|props| props.hp > Fixed64::ZERO)
}

fn spawn_courier(state: &mut MobaMatch, world: &mut World, index: usize) {
    let slot = &state.heroes[index];
    // owned by the hero's body, the one that died while it is dead
    let (Some(owner), Some(pos)) = (slot.entity.or(slot.retired_body), slot.courier_spawn) else { return };
    let Some(stats) = omoba_template_ids::summon_by_name(COURIER_SUMMON).and_then(omoba_template_ids::active_summon_stats) else {
        log::error!("shop_range_rule needs the v3 summon '{COURIER_SUMMON}'");
        return;
    };
    let (side, player_id) = (slot.side, slot.player_id);
    let owner_faction = faction(side as u8, state.config.teams[side]);
    let entity = world.create_entity().build();
    match crate::runtime::native::game_processor::commit_summoned_spawn(world, entity, pos, COURIER_SUMMON, owner,
        owner_faction, Some(player_id), Fixed64::ZERO, stats) {
        Ok(()) => state.heroes[index].courier = Some(entity),
        Err(error) => {
            log::error!("courier of player {player_id} not spawned: {error}");
            let _ = world.entities().delete(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::native::moba_match::bots::RoleBotMatchPlan;

    fn world(shop_range_rule: bool) -> (World, crate::runtime::SimulationDriver) {
        let json = serde_json::json!({"schema_version":1,"map_id":"three_lane_training","think_hz":5,"shop_range_rule":shop_range_rule,
            "players":[{"player_id":1,"team_id":1,"hero":"tamako","role":"carry","lane":"bottom","bot":false},
                {"player_id":2,"team_id":2,"hero":"murakami_yoshikiyo","role":"mid","lane":"mid","bot":false}]});
        let plan: RoleBotMatchPlan = serde_json::from_value(json).unwrap();
        let (config, _) = plan.compile(1, crate::runtime::SimulationTickProfile::Production60Hz).unwrap();
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(crate::runtime::TowerTemplateRegistry::default());
        setup_single_lane_match(&mut world, config).unwrap();
        world.insert(crate::runtime::ScriptRegistry::new());
        let driver = crate::runtime::SimulationDriver::from_world(&mut world, crate::runtime::SimulationTickProfile::Production60Hz).unwrap();
        (world, driver)
    }

    fn couriers(world: &World) -> Vec<(Entity, Entity, i32)> {
        use specs::Join;
        let (entities, units, summons, factions) = (world.entities(), world.read_storage::<Unit>(),
            world.read_storage::<SummonedUnit>(), world.read_storage::<Faction>());
        (&entities, &units, &summons, &factions).join()
            .filter(|(entity, unit, _, _)| matches!(unit.unit_type, UnitType::Courier) && alive(world, *entity))
            .map(|(entity, _, summon, faction)| (entity, summon.summoner, faction.team_id)).collect()
    }

    #[test]
    fn each_player_gets_a_courier_only_under_the_shop_range_rule() {
        let (off, _) = world(false);
        assert!(couriers(&off).is_empty(), "no courier without the rule");
        let (on, _) = world(true);
        let state = on.read_resource::<MobaMatch>();
        let found = couriers(&on);
        assert_eq!(found.len(), state.heroes.len(), "one courier per player: {found:?}");
        for slot in &state.heroes {
            let courier = courier_of(&on, &state, slot.player_id).expect("a living courier");
            let (_, owner, team) = found.iter().copied().find(|(entity, _, _)| *entity == courier).unwrap();
            assert_eq!(Some(owner), slot.entity, "the hero owns its courier");
            assert_eq!(i64::from(team), i64::from(state.config.teams[slot.side]), "on the hero's team");
            let props = on.read_storage::<CProperty>();
            let props = props.get(courier).unwrap();
            assert_eq!((props.mhp, props.msd), (Fixed64::from_i32(500), Fixed64::from_i32(550)), "the original's courier body");
            assert!(on.read_resource::<crate::runtime::BuffStore>().get(courier, crate::runtime::native::magic_immunity::BUFF).is_some(),
                "magic immune for life");
            assert_eq!(on.read_storage::<Pos>().get(courier).map(|p| p.0), slot.courier_spawn, "at the hero's spawn");
        }
    }

    #[test]
    fn a_killed_courier_comes_back_at_its_spawn_on_the_next_tick() {
        let (mut world, mut driver) = world(true);
        // past the warmup: the match ticks its play phase
        while world.read_resource::<MobaMatch>().phase == MobaMatchPhase::Warmup { driver.step(&mut world, Vec::new()).unwrap(); }
        let (first, spawn) = {
            let state = world.read_resource::<MobaMatch>();
            (courier_of(&world, &state, 1).unwrap(), state.heroes.iter().find(|s| s.player_id == 1).unwrap().courier_spawn.unwrap())
        };
        world.write_storage::<Pos>().get_mut(first).unwrap().0 = spawn + Vec2::new(Fixed64::from_i32(900), Fixed64::ZERO);
        world.write_storage::<CProperty>().get_mut(first).unwrap().hp = Fixed64::ZERO;
        for _ in 0..3 { driver.step(&mut world, Vec::new()).unwrap(); }
        let state = world.read_resource::<MobaMatch>();
        let next = courier_of(&world, &state, 1).expect("a courier again");
        assert_ne!(next, first, "a new body");
        assert_eq!(world.read_storage::<Pos>().get(next).map(|p| p.0), Some(spawn), "back at its spawn");
        assert_eq!(couriers(&world).iter().filter(|(_, _, team)| i64::from(*team) == i64::from(state.config.teams[0])).count(), 1,
            "still one courier for the player");
    }
    /// The authority and every team's replica (in process, through the server's projection) step
    /// 720 ticks of a shop_range_rule match; player 1's courier is killed at tick 240 and comes back.
    /// Fails at the first tick a replica's team hash differs from the authority's view.
    #[test]
    fn a_courier_spawn_death_and_respawn_keep_every_team_replica_with_the_authority() {
        use crate::runtime::inprocess_observer::{accepted_inputs, project_tick, InProcessObserver};
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world, SingleLaneConfig { warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(1_000_000),
            shop_range_rule: true, ..SingleLaneConfig::default() }).unwrap();
        world.insert(crate::runtime::TowerTemplateRegistry::default());
        world.insert(crate::runtime::ScriptRegistry::new());
        let mut driver = crate::runtime::SimulationDriver::from_world(&mut world, crate::runtime::SimulationTickProfile::Production60Hz).unwrap();
        let step = |world: &mut World, driver: &mut crate::runtime::SimulationDriver| {
            let accepted = accepted_inputs(world, driver.tick() + 1, &[]);
            let result = driver.step(world, Vec::new()).unwrap();
            world.write_resource::<crate::runtime::TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
            project_tick(world, &result).unwrap();
            result.tick
        };
        step(&mut world, &mut driver);
        let seed = world.try_fetch::<crate::runtime::MasterSeed>().map_or(0, |seed| seed.0);
        let mut observer = InProcessObserver::start(&mut world, driver.tick() + 1, seed, 1, 60).unwrap();
        let first = { let state = world.read_resource::<MobaMatch>(); courier_of(&world, &state, 1).unwrap() };
        for at in 0..720 {
            if at == 240 { world.write_storage::<CProperty>().get_mut(first).unwrap().hp = Fixed64::ZERO; }
            let tick = step(&mut world, &mut driver);
            assert_eq!(observer.after_projection(&mut world, tick).unwrap(), None, "tick {tick}");
        }
        let state = world.read_resource::<MobaMatch>();
        assert!(courier_of(&world, &state, 1).is_some_and(|next| next != first), "the killed courier came back as a new body");
    }
}
