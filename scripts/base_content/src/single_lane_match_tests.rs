//! Production ECS + generated Lua manifest + formal PlayerInput acceptance.

use omoba_core::runtime::*;
use omoba_sim::Fixed64;
use specs::{Join, World, WorldExt};

fn world(config: SingleLaneConfig, profile: SimulationTickProfile) -> (World, SimulationDriver) {
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    world.insert(TowerTemplateRegistry::default());
    let mut scripts = ScriptRegistry::new();
    scripts.insert_manifest(crate::get_manifest());
    populate_ability_registry(&mut world, &scripts);
    world.insert(scripts);
    setup_single_lane_match(&mut world, config).expect("match setup");
    let driver = SimulationDriver::from_world(&mut world, profile).unwrap();
    (world, driver)
}

fn fast_config() -> SingleLaneConfig {
    SingleLaneConfig {
        seed: 0x20261003,
        warmup: Fixed64::ZERO,
        tower_hp: Fixed64::from_i32(360),
        base_hp: Fixed64::from_i32(500),
        respawn_delay: Fixed64::from_i32(2),
        // These fixtures isolate combat / transactions from the income clock.
        passive_gold_per_second: 0,
        lane_creep_gold: 0,
        ..SingleLaneConfig::default()
    }
}

fn three_lane_config() -> SingleLaneConfig {
    SingleLaneConfig { map_id: Some("three_lane_training".into()), ..fast_config() }
}

fn mana_cast_fixture(balance: i32) -> (World, SimulationDriver, specs::Entity, specs::Entity) {
    use omoba_core::runtime::ability_runtime::ManaPool;
    let (mut w, mut driver) = world(SingleLaneConfig {
        heroes: ["training_ranger".into(), "training_luminary".into()],
        wave_interval: Fixed64::from_i32(10_000), ..three_lane_config()
    }, SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let [caster, target] = hero_pair(&w);
    let source = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(2500));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0 = source;
    w.write_storage::<Pos>().get_mut(target).unwrap().0 = source
        + omoba_sim::Vec2::new(Fixed64::from_i32(300), Fixed64::ZERO);
    w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool = Some(
        ManaPool::new(Fixed64::from_i32(balance), Fixed64::from_i32(280)).unwrap());
    let mut props = w.write_storage::<CProperty>();
    for entity in [caster, target] {
        let prop = props.get_mut(entity).unwrap();
        prop.hp = Fixed64::from_i32(10_000); prop.mhp = prop.hp;
        prop.def_physic = Fixed64::ZERO; prop.def_magic = Fixed64::ZERO;
    }
    drop(props);
    (w, driver, caster, target)
}

#[test]
fn attack_visual_state_authority_60hz_uses_resolved_speed_and_publishes_visible_phase() {
    use omoba_core::runtime::attack_visual_state::AttackVisualState;
    use omoba_core::runtime::native::tick::attack_phase::attack_phase_durations;
    use omb_script_abi::stat_keys::StatKey;
    let (mut w, mut driver, caster, target) = mana_cast_fixture(90);
    let mut payload = serde_json::Map::new();
    payload.insert(StatKey::AttackSpeedBonusConstant.as_str().into(), serde_json::json!(100 * 1024));
    w.write_resource::<BuffStore>().add(caster, "private_animation_speed_fixture", Fixed64::from_i32(30), payload.into());
    let base = w.read_storage::<TAttack>().get(caster).unwrap().asd.v;
    w.write_storage::<TAttack>().get_mut(caster).unwrap().asd_count = Fixed64::from_i32(10);
    let input = (1, PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: target.id(), queued: false })) });
    let mut phases = std::collections::BTreeSet::new();
    for tick in 0..120 {
        let result = driver.step(&mut w, if tick == 0 { vec![input.clone()] } else { vec![] }).unwrap();
        project_tick(&mut w, result);
        let attack = *w.read_storage::<TAttack>().get(caster).unwrap();
        let state = AttackVisualState::from_attack(&attack);
        if state.phase == 0 { continue; }
        assert_eq!(attack.animation_timing, Some(AttackAnimationTiming::new(attack_phase_durations(base / Fixed64::from_i32(2)), false)));
        phases.insert(state.phase);
        let bytes = state.encode().unwrap();
        let frames = &w.read_resource::<TeamProjectionRuntime>().latest_frames;
        assert!(!frames.is_empty());
        assert!(frames.values().any(|frame| frame.frame.step.as_ref().is_some_and(|step| step.public_events.iter().any(|event|
            event.event_kind == FactKind::AttackVisual as u32 && event.sanitized_payload == bytes))),
            "actual visible authority phase must be emitted, not derived from base ASD");
    }
    assert_eq!(phases, std::collections::BTreeSet::from([1, 2]));
}

#[test]
fn attack_visual_state_stun_60hz_freezes_and_resumes_authority() {
    use omoba_core::runtime::attack_visual_state::AttackVisualState;
    let (mut w, mut driver, caster, target) = mana_cast_fixture(90);
    let base = w.read_storage::<TAttack>().get(caster).unwrap().asd.v;
    w.write_storage::<TAttack>().get_mut(caster).unwrap().asd_count = base;
    let input = (1, PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: target.id(), queued: false })) });
    let mut started = None;
    for tick in 0..90 {
        let result = driver.step(&mut w, if tick == 0 { vec![input.clone()] } else { vec![] }).unwrap();
        project_tick(&mut w, result);
        let state = AttackVisualState::from_attack(w.read_storage::<TAttack>().get(caster).unwrap());
        if state.phase == 1 && state.elapsed_raw < state.duration_raw { started = Some(state); break; }
    }
    let initial = started.expect("actual attack windup started");
    w.write_resource::<BuffStore>().add(caster, "stun", Fixed64::from_i32(10), serde_json::json!({}));
    for _ in 0..3 {
        let result = driver.step(&mut w, []).unwrap(); project_tick(&mut w, result);
        let state = AttackVisualState::from_attack(w.read_storage::<TAttack>().get(caster).unwrap());
        assert!(state.paused);
        assert_eq!((state.sequence, state.phase, state.elapsed_raw), (initial.sequence, initial.phase, initial.elapsed_raw));
        assert!(w.read_resource::<TeamProjectionRuntime>().latest_frames.values().any(|frame|
            frame.frame.step.as_ref().is_some_and(|step| step.public_events.iter().any(|event|
                event.event_kind == FactKind::AttackVisual as u32 && event.sanitized_payload == state.encode().unwrap()))));
    }
    w.write_resource::<BuffStore>().remove(caster, "stun");
    driver.step(&mut w, []).unwrap();
    let resumed = AttackVisualState::from_attack(w.read_storage::<TAttack>().get(caster).unwrap());
    assert!(!resumed.paused);
    assert_eq!(resumed.sequence, initial.sequence);
    assert!(resumed.elapsed_raw > initial.elapsed_raw);
}

#[test]
fn mana_cast_formal_60hz_success_debits_and_rejection_preserves_balance_and_cooldown() {
    let (mut w, mut driver, caster, target) = mana_cast_fixture(90);
    let cast = |slot, target| (1, PlayerInput { action: Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index: slot, target_entity: Some(target), target_pos: None,
    })) });
    driver.step(&mut w, [cast(0, caster.id())]).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current(), Fixed64::from_i32(90));
    assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_shot"));
    driver.step(&mut w, [cast(0, target.id())]).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp, Fixed64::from_i32(9910));
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current(), Fixed64::from_i32(45));
    driver.step(&mut w, [cast(3, target.id())]).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp, Fixed64::from_i32(9700));
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current(), Fixed64::ZERO);
    driver.step(&mut w, [cast(1, caster.id())]).unwrap();
    assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_patch"));
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current(), Fixed64::ZERO);
}

#[test]
fn successful_cast_facts_60hz_source_and_team_projection_not_input_ack() {
    use omoba_core::runtime::presentation_cue::{AbilityPresentationCue, stable_content_fact_id};
    let (mut w, mut driver, caster, target) = mana_cast_fixture(90);
    w.write_storage::<Hero>().get_mut(caster).unwrap().ability_levels.insert("ranger_shot".into(), 3);
    let cast = |target| (1, PlayerInput {action: Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index: 0, target_entity: Some(target), target_pos: None,
    }))});
    let count = |result: &SimulationTickResult| result.facts.iter()
        .filter(|fact| matches!(fact.fact, ObservableFact::Ability {source, ..} if source == canonical_entity_id(caster))).count();
    let rejected = driver.step(&mut w, [cast(caster.id())]).unwrap();
    assert_eq!(count(&rejected), 0, "accepted friendly request is not successful cast");
    let succeeded = driver.step(&mut w, [cast(target.id())]).unwrap();
    assert_eq!(count(&succeeded), 1);
    let fact = succeeded.facts.iter().find(|fact| matches!(fact.fact,
        ObservableFact::Ability {source, ..} if source == canonical_entity_id(caster))).unwrap();
    assert_eq!(fact.audience, FactAudience::VisibilityPolicy(
        omb_script_abi::types::projection_policy_ids::HERO_ABILITY.into()));
    assert!(matches!(fact.fact, ObservableFact::Ability {ability_id, rank: 3, ..}
        if ability_id == stable_content_fact_id("ranger_shot")));
    // Later HUD/hero state must not rewrite the earlier successful invocation.
    w.write_storage::<Hero>().get_mut(caster).unwrap().ability_levels.insert("ranger_shot".into(), 4);
    let canonical = canonical_entity_id(caster);
    let visible = std::collections::BTreeSet::from([canonical]);
    let mut projector = TeamViewProjector::new(1, TeamProjectorConfig::default());
    let projected = projector.build_frame(succeeded.tick, succeeded.tick, &visible,
        vec![VisibilityTransition::Reveal {canonical_id: canonical, effective_tick: succeeded.tick, baseline: Vec::new()}],
        std::slice::from_ref(fact), &ProjectionDependencyGraph::default()).unwrap();
    let frame = &projected.frame;
    let step = frame.step.as_ref().unwrap();
    let events: Vec<_> = step.public_events.iter().filter(|event| event.event_kind == FactKind::Ability as u32).collect();
    assert_eq!(events.len(), 1, "real successful cast passes common safety projector");
    let caster_replica = events[0].subject.as_ref().unwrap().value;
    let (id, cue) = AbilityPresentationCue::from_public_event(frame.replica_tick, events[0], |id|
        (id == caster_replica).then_some(1)).unwrap();
    assert_eq!(cue.ability_id, stable_content_fact_id("ranger_shot"));
    assert_eq!(cue.rank, 3);
    assert_eq!(id >> 32, frame.replica_tick);
    let hidden = TeamViewProjector::new(2, TeamProjectorConfig::default())
        .build_frame(succeeded.tick, succeeded.tick, &Default::default(), vec![],
            std::slice::from_ref(fact), &ProjectionDependencyGraph::default()).unwrap();
    assert!(!hidden.frame.step.as_ref().unwrap().public_events.iter()
        .any(|event| event.event_kind == FactKind::Ability as u32));
    assert_eq!(count(&driver.step(&mut w, [cast(target.id())]).unwrap()), 0,
        "cooldown rejection cannot republish cast");
    assert_eq!(count(&driver.step(&mut w, []).unwrap()), 0,
        "latest visual is not replayed on an empty step");
}

#[test]
fn successful_cast_facts_insufficient_mana_never_publishes() {
    let (mut w, mut driver, caster, target) = mana_cast_fixture(0);
    let result = driver.step(&mut w, [(1, PlayerInput {action: Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index: 0, target_entity: Some(target.id()), target_pos: None,
    }))})]).unwrap();
    assert!(!result.facts.iter().any(|fact| fact.key.fact_kind == FactKind::Ability));
    assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_shot"));
}

#[test]
fn successful_area_cue_formal_60hz_has_resolved_center_and_team_scope() {
    use omoba_core::runtime::presentation_cue::{PresentationCue,stable_content_fact_id};
    let (mut w,mut driver,caster,_) = mana_cast_fixture(280);
    let cast=|x| [(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:2,target_entity:None,target_pos:Some(Vec2I {x:x*1024,y:2500*1024}),
    }))})];
    let rejected=driver.step(&mut w,cast(9999)).unwrap();
    assert!(!rejected.facts.iter().any(|fact|matches!(fact.fact,ObservableFact::AbilityArea {..})));
    let succeeded=driver.step(&mut w,cast(300)).unwrap();
    let areas:Vec<_>=succeeded.facts.iter().filter(|fact|matches!(fact.fact,ObservableFact::AbilityArea {..})).collect();
    assert_eq!(areas.len(),1);
    let fact=areas[0];
    assert_eq!(fact.audience,FactAudience::Team(1));
    assert!(matches!(fact.fact,ObservableFact::AbilityArea {source,team:1,ability_id,x_raw,y_raw,radius_raw,duration_raw,..}
        if source==canonical_entity_id(caster) && ability_id==stable_content_fact_id("ranger_volley")
        && (x_raw,y_raw)==(300*1024,2500*1024) && radius_raw>0 && duration_raw==1024));
    for (team,visible) in [(1,true),(1,false),(2,true)] {
        let canonical=canonical_entity_id(caster);
        let visible_set=if visible {std::collections::BTreeSet::from([canonical])} else {Default::default()};
        let reveals=if visible {vec![VisibilityTransition::Reveal {canonical_id:canonical,effective_tick:succeeded.tick,baseline:Vec::new()}]} else {Vec::new()};
        let projected=TeamViewProjector::new(team,TeamProjectorConfig::default()).build_frame(
            succeeded.tick,succeeded.tick,&visible_set,reveals,std::slice::from_ref(fact),&ProjectionDependencyGraph::default()).unwrap();
        let cues:Vec<_>=projected.frame.step.as_ref().unwrap().public_events.iter().filter_map(|event|
            PresentationCue::from_public_event(succeeded.tick,event,|_|Some(1))).collect();
        assert_eq!(cues.len(),usize::from(team==1 && visible));
        if let Some((_,PresentationCue::AbilityArea(area)))=cues.first() {assert_eq!(area.center,(300*1024,2500*1024));}
    }
    assert!(!driver.step(&mut w,[]).unwrap().facts.iter().any(|fact|matches!(fact.fact,ObservableFact::AbilityArea {..})));
}

#[test]
fn successful_cast_facts_keep_unique_order_across_dispatch_drains_in_one_tick() {
    let (mut w, _, caster, target) = mana_cast_fixture(90);
    w.read_resource::<ObservableFactBuffer>().drain_ordered().unwrap();
    w.write_resource::<ScriptVisualEventQueue>().drain();
    let mut registry = ScriptRegistry::new(); registry.insert_manifest(crate::get_manifest());
    for slot in [0, 3] {
        handle_ability_cast_from_input(&mut w, slot, None, Some(target.id()), 1).unwrap();
        run_script_dispatch(&mut w, &registry, 17, Fixed64::from_raw(17));
    }
    let facts: Vec<_> = w.read_resource::<ObservableFactBuffer>().drain_ordered().unwrap().into_iter()
        .filter(|fact| fact.key.fact_kind == FactKind::Ability).collect();
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].key.tick, facts[1].key.tick);
    assert_ne!(facts[0].key.local_ordinal, facts[1].key.local_ordinal);
    let visuals = w.write_resource::<ScriptVisualEventQueue>().drain();
    assert_eq!(visuals.iter().filter(|event| event.kind == ScriptVisualEventKind::SkillCast
        && event.projection_policy_id.as_ref().is_some_and(|policy|
            policy.value.as_str() == omb_script_abi::types::projection_policy_ids::HERO_ABILITY)).count(), 2);
}

#[test]
fn cast_visuals_publish_only_success_for_managed_and_legacy_handlers() {
    for managed in [false, true] {
        let (mut w, _, caster, target) = mana_cast_fixture(90);
        if !managed { w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool = None; }
        let mut registry = ScriptRegistry::new();
        registry.insert_manifest(crate::get_manifest());
        w.write_resource::<ScriptVisualEventQueue>().drain();
        // Same formal input adapter, but inspect the queue before the normal
        // publication barrier drains it. The generated handler rejects allies.
        handle_ability_cast_from_input(&mut w, 0, None, Some(caster.id()), 1).unwrap();
        run_script_dispatch(&mut w, &registry, 7, Fixed64::from_raw(17));
        assert!(!w.write_resource::<ScriptVisualEventQueue>().drain().iter()
            .any(|event| event.kind == ScriptVisualEventKind::SkillCast), "managed={managed}: failed handler cue");
        assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_shot"));
        handle_ability_cast_from_input(&mut w, 0, None, Some(target.id()), 1).unwrap();
        handle_ability_cast_from_input(&mut w, 3, None, Some(caster.id()), 1).unwrap();
        run_script_dispatch(&mut w, &registry, 8, Fixed64::from_raw(17));
        let events = w.write_resource::<ScriptVisualEventQueue>().drain();
        let casts: Vec<_> = events.iter().filter(|event| event.kind == ScriptVisualEventKind::SkillCast).collect();
        assert_eq!(casts.len(), 1, "managed={managed}: successful handler cue");
        assert_eq!(casts[0].primary, caster);
        assert_eq!(casts[0].secondary, Some(target));
        assert_eq!(casts[0].skill_id.as_deref(), Some("ranger_shot"));
        process_outcomes(&mut w, &mut RuntimeEventVecSink::default()).unwrap();
        assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_shot"));
        assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_finisher"));
        // Queued internal events still pass the dispatcher cooldown gate.
        w.write_resource::<ScriptEventQueue>().push(ScriptEvent::SkillCast {
            caster, skill_id:"ranger_shot".into(), target:SkillTarget::Entity(target),
        });
        run_script_dispatch(&mut w, &registry, 9, Fixed64::from_raw(17));
        assert!(!w.write_resource::<ScriptVisualEventQueue>().drain().iter()
            .any(|event| event.kind == ScriptVisualEventKind::SkillCast), "managed={managed}: cooldown cue");
    }
}

#[test]
fn checked_cast_rank_rejects_overflow_and_catalog_bounds_for_every_resource_path() {
    for formal in [false, true] {
        for managed in [false, true] {
            for rank in [-1, 0, 1, 3, 5, 255, 256, 257, i32::MAX] {
                let (mut w, _, caster, target) = mana_cast_fixture(90);
                if !formal { w.remove::<MobaMatch>(); }
                {
                    let mut heroes = w.write_storage::<Hero>();
                    let hero = heroes.get_mut(caster).unwrap();
                    if !managed { hero.mana_pool = None; }
                    hero.ability_levels.insert("ranger_shot".into(), rank);
                }
                let mut registry = ScriptRegistry::new();
                registry.insert_manifest(crate::get_manifest());
                assert_eq!(registry.get_ability("ranger_shot").unwrap().0.max_level, 4);
                w.write_resource::<ScriptVisualEventQueue>().drain();
                w.read_resource::<ObservableFactBuffer>().drain_ordered().unwrap();
                w.write_resource::<ScriptEventQueue>().push(ScriptEvent::SkillCast {
                    caster, skill_id: "ranger_shot".into(), target: SkillTarget::Entity(target),
                });
                run_script_dispatch(&mut w, &registry, 11, Fixed64::from_raw(17));
                let expected = (1..=4).contains(&rank) || (!formal && !managed && rank <= 0);
                let events = w.write_resource::<ScriptVisualEventQueue>().drain();
                let casts: Vec<_> = events.iter().filter(|event| event.kind == ScriptVisualEventKind::SkillCast).collect();
                let label = format!("formal={formal} managed={managed} rank={rank}");
                assert_eq!(casts.len(), usize::from(expected), "{label}");
                let facts = w.read_resource::<ObservableFactBuffer>().drain_ordered().unwrap();
                let ability: Vec<_> = facts.iter().filter(|fact| fact.key.fact_kind == FactKind::Ability).collect();
                assert_eq!(ability.len(), usize::from(expected), "{label}");
                if expected {
                    let published = rank.max(0) as u32;
                    assert_eq!(casts[0].ability_rank, published, "{label}");
                    assert!(matches!(ability[0].fact, ObservableFact::Ability {rank, ..} if rank == published), "{label}");
                }
                process_outcomes(&mut w, &mut RuntimeEventVecSink::default()).unwrap();
                let heroes = w.read_storage::<Hero>();
                let hero = heroes.get(caster).unwrap();
                assert_eq!(hero.is_on_cooldown("ranger_shot"), expected, "{label}");
                if !expected && managed {
                    assert_eq!(hero.mana_pool.as_ref().unwrap().current(), Fixed64::from_i32(90), "{label}");
                }
            }
        }
    }
}

#[test]
fn missing_ability_handler_never_publishes_cast_visual() {
    let (mut w, _, caster, target) = mana_cast_fixture(90);
    w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool = None;
    w.write_resource::<ScriptVisualEventQueue>().drain();
    w.read_resource::<ObservableFactBuffer>().drain_ordered().unwrap();
    handle_ability_cast_from_input(&mut w, 0, None, Some(target.id()), 1).unwrap();
    run_script_dispatch(&mut w, &ScriptRegistry::new(), 7, Fixed64::from_raw(17));
    assert!(!w.write_resource::<ScriptVisualEventQueue>().drain().iter()
        .any(|event| event.kind == ScriptVisualEventKind::SkillCast));
    assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_shot"));
    assert!(!w.read_resource::<ObservableFactBuffer>().drain_ordered().unwrap().iter()
        .any(|fact| fact.key.fact_kind == FactKind::Ability));
}

#[test]
fn mana_archetype_content_60hz_uses_generated_handlers_for_three_resource_identities() {
    for (hero_id,skill,heal,cost,restore,bonus,rate) in [
        ("training_vanguard","vanguard_recover",110,45,0,60,5),
        ("training_ranger","ranger_patch",55,45,0,0,7),
        ("training_luminary","lumen_touch",70,55,20,0,5),
    ] {
        let (mut w,mut driver)=world(SingleLaneConfig {
            mana_enabled:true,base_recovery_enabled:false,
            heroes:[hero_id.into(),"training_luminary".into()],
            wave_interval:Fixed64::from_i32(10_000),..fast_config()
        },SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();
        let caster=hero_pair(&w)[0];
        let base=w.read_storage::<Hero>().get(caster).unwrap().moba_mana_capacity().unwrap();
        w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool=Some(
            ability_runtime::ManaPool::new(Fixed64::from_i32(90),base).unwrap());
        {let mut props=w.write_storage::<CProperty>();let health=props.get_mut(caster).unwrap();
            health.hp=Fixed64::from_i32(100);health.mhp=Fixed64::from_i32(10_000);}
        let before=w.read_resource::<MobaMatch>().elapsed;
        driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index:1,target_entity:None,target_pos:None}))})]).unwrap();
        let delta=w.read_resource::<MobaMatch>().elapsed-before;
        let mut expected=ability_runtime::ManaPool::new(Fixed64::from_i32(90-cost+restore),base+Fixed64::from_i32(bonus)).unwrap();
        expected.regenerate(Fixed64::from_i32(rate),delta).unwrap();
        assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected,"{hero_id}");
        assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown(skill));
        assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(100+heal));
        if bonus>0 || rate>5 {
            let stat=if bonus>0 {"mana_bonus"} else {"mana_regen_constant"};
            let key=format!("generic_mana:{skill}:{stat}:{}:{}",caster.id(),caster.gen().id());
            let buffs=w.read_resource::<BuffStore>();let buff=buffs.get(caster,&key).unwrap();
            assert_eq!(buff.payload[stat],if bonus>0 {60*1024} else {2*1024});
            assert_eq!(buff.remaining,Fixed64::from_i32(6));
            let visual=omoba_core::runtime::buff_visual_state::BuffVisualState::from_store(&buffs,caster).render_buffs();
            let public_id=if bonus>0 {"mana_capacity"} else {"mana_regeneration"};
            assert_eq!(visual.len(),1);
            assert_eq!(visual[0].buff_id,public_id);
            assert_eq!(visual[0].remaining_secs,6.0);
            assert!(visual[0].payload_json.is_empty());
        }
        let events=w.write_resource::<ScriptEventQueue>().drain();
        let costs:Vec<_>=events.iter().filter_map(|event|match event {
            ScriptEvent::SpentMana {caster:e,cost,..} if *e==caster=>Some(cost.raw()),_=>None}).collect();
        assert_eq!(costs,vec![i64::from(cost)*1024]);
        if restore>0 {assert!(events.iter().any(|event|matches!(event,
            ScriptEvent::ManaGained {e,amount} if *e==caster && *amount==Fixed64::from_i32(restore))));}
    }
}

#[test]
fn mana_capacity_60hz_growth_expiry_precedes_cast_and_invalid_bonus_fails_closed() {
    let (mut w,mut driver)=world(SingleLaneConfig {
        mana_enabled:true,base_recovery_enabled:false,
        heroes:["training_ranger".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..fast_config()
    },SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let caster=hero_pair(&w)[0];
    w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool.as_mut().unwrap().spend(Fixed64::from_i32(80)).unwrap();
    {
        let mut buffs=w.write_resource::<BuffStore>();
        buffs.add(caster,"no_regen",Fixed64::from_i32(100),serde_json::json!({"mana_regen_percentage":-1024}));
        buffs.add(caster,"capacity",Fixed64::from_i32(10),serde_json::json!({"mana_bonus":100*1024,"extra_mana_bonus":20*1024}));
    }
    driver.step(&mut w,[]).unwrap();
    let pool=w.read_storage::<Hero>().get(caster).unwrap().mana_pool.clone().unwrap();
    assert_eq!(pool.raw_state(),(200*1024,400*1024,0),"capacity growth must not refill");
    w.write_resource::<GamePause>().is_paused=true;
    driver.step(&mut w,[]).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&pool);
    w.write_resource::<GamePause>().is_paused=false;
    w.write_storage::<Hero>().get_mut(caster).unwrap().level=2;
    let base=w.read_storage::<Hero>().get(caster).unwrap().moba_mana_capacity().unwrap();
    driver.step(&mut w,[]).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().raw_state(),(200*1024,base.raw()+120*1024,0));
    w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool=Some(
        ability_runtime::ManaPool::new(base+Fixed64::from_i32(100),base+Fixed64::from_i32(120)).unwrap());
    {
        let mut buffs=w.write_resource::<BuffStore>();
        buffs.remove(caster,"capacity");
        buffs.add(caster,"expiring_capacity",Fixed64::from_raw(1),serde_json::json!({"mana_bonus":120*1024}));
    }
    driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:1,target_entity:None,target_pos:None}))})]).unwrap();
    assert!(!w.read_resource::<BuffStore>().has(caster,"expiring_capacity"));
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().raw_state(),
        (base.raw()-45*1024,base.raw(),0),"expiry clamps before admission, not after spending old excess");
    w.write_resource::<BuffStore>().add(caster,"negative_capacity",Fixed64::from_i32(10),serde_json::json!({"mana_bonus":-1_000_000*1024i64}));
    driver.step(&mut w,[]).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().raw_state(),(0,0,0));
    {
        let mut buffs=w.write_resource::<BuffStore>();
        buffs.remove(caster,"negative_capacity");
        buffs.add(caster,"invalid_capacity",Fixed64::from_i32(10),serde_json::json!({"mana_bonus":i64::MAX}));
    }
    driver.step(&mut w,[]).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().raw_state(),(0,base.raw(),0));
}

#[test]
fn mana_buff_formal_60hz_recovery_expiration_pause_and_home_bonus_share_one_pool() {
    let (mut w,mut driver)=world(SingleLaneConfig {
        mana_enabled:true,base_recovery_enabled:true,
        heroes:["training_ranger".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..fast_config()
    },SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let caster=hero_pair(&w)[0];
    let home=w.read_resource::<MobaMatch>().bases[0].unwrap();
    let position=w.read_storage::<Pos>().get(home).unwrap().0;
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=position;
    w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool.as_mut().unwrap()
        .spend(Fixed64::from_i32(100)).unwrap();
    w.write_resource::<BuffStore>().add(caster,"mana_recovery",Fixed64::from_i32(10),serde_json::json!({
        "mana_regen_constant":3*1024,"mana_regen_percentage":512,"mana_regen_total_percentage":1024,
    }));
    let mut expected=w.read_storage::<Hero>().get(caster).unwrap().mana_pool.clone().unwrap();
    let before=w.read_resource::<MobaMatch>().elapsed;
    driver.step(&mut w,[]).unwrap();
    let delta=w.read_resource::<MobaMatch>().elapsed-before;
    let home_rate=Fixed64::from_i32(omoba_template_ids::MOBA_BASE_RECOVERY_MANA_PER_SECOND as i32);
    expected.regenerate(Fixed64::from_i32(24)+home_rate,delta).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    w.write_resource::<GamePause>().is_paused=true;
    driver.step(&mut w,[]).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    w.write_resource::<GamePause>().is_paused=false;
    // A duration shorter than the next tick expires in the normal buff system.
    w.write_resource::<BuffStore>().remove(caster,"mana_recovery");
    w.write_resource::<BuffStore>().add(caster,"expiring",Fixed64::from_raw(1),serde_json::json!({"mana_regen_constant":100*1024}));
    let before=w.read_resource::<MobaMatch>().elapsed;
    driver.step(&mut w,[]).unwrap();
    let delta=w.read_resource::<MobaMatch>().elapsed-before;
    assert!(!w.read_resource::<BuffStore>().has(caster,"expiring"));
    expected.regenerate(Fixed64::from_i32(omoba_template_ids::MOBA_MANA_REGEN_PER_SECOND as i32)+home_rate,delta).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    w.write_resource::<BuffStore>().add(caster,"invalid",Fixed64::from_i32(10),serde_json::json!({
        "mana_regen_constant":i64::MAX,"mana_regen_percentage":i64::MAX,"mana_regen_total_percentage":i64::MAX,
    }));
    let before=w.read_resource::<MobaMatch>().elapsed;
    driver.step(&mut w,[]).unwrap();
    expected.regenerate(home_rate,w.read_resource::<MobaMatch>().elapsed-before).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
}

#[test]
fn mana_lifecycle_60hz_birth_active_regeneration_pause_growth_and_finished_freeze() {
    let (mut w, mut driver) = world(SingleLaneConfig {
        mana_enabled: true, heroes: ["training_ranger".into(), "training_luminary".into()],
        wave_interval: Fixed64::from_i32(10_000), ..three_lane_config()
    }, SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let caster = hero_pair(&w)[0];
    let initial = w.read_storage::<Hero>().get(caster).unwrap().mana_pool.clone().unwrap();
    assert_eq!(initial.maximum(), Fixed64::from_i32(280));
    assert_eq!(initial.current(), initial.maximum());
    w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool.as_mut().unwrap()
        .spend(Fixed64::from_i32(50)).unwrap();
    let before = w.read_resource::<MobaMatch>().elapsed;
    let mut expected = w.read_storage::<Hero>().get(caster).unwrap().mana_pool.clone().unwrap();
    for _ in 0..12 { driver.step(&mut w, []).unwrap(); }
    let delta = w.read_resource::<MobaMatch>().elapsed - before;
    expected.regenerate(Fixed64::from_i32(omoba_template_ids::MOBA_MANA_REGEN_PER_SECOND as i32), delta).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(), &expected);
    w.write_resource::<GamePause>().is_paused = true;
    driver.step(&mut w, []).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(), &expected);
    w.write_resource::<GamePause>().is_paused = false;
    w.write_storage::<Hero>().get_mut(caster).unwrap().level = 2;
    let maximum = w.read_storage::<Hero>().get(caster).unwrap().moba_mana_capacity().unwrap();
    assert!(maximum > initial.maximum());
    expected.set_maximum(maximum).unwrap();
    let before = w.read_resource::<MobaMatch>().elapsed;
    driver.step(&mut w, []).unwrap();
    expected.regenerate(Fixed64::from_i32(omoba_template_ids::MOBA_MANA_REGEN_PER_SECOND as i32),
        w.read_resource::<MobaMatch>().elapsed - before).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(), &expected);
    w.write_resource::<MobaMatch>().phase = MobaMatchPhase::Finished { winner: None, tick: driver.tick() };
    driver.step(&mut w, []).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(), &expected);
}

#[test]
fn mana_agreement_60hz_disabled_match_omits_new_fact_and_enabled_match_commits_pools() {
    for enabled in [false, true] {
        let (mut w, mut driver) = world(SingleLaneConfig {
            mana_enabled: enabled, wave_interval: Fixed64::from_i32(10_000), ..fast_config()
        }, SimulationTickProfile::Production60Hz);
        for _ in 0..3 {
            let result = driver.step(&mut w, []).unwrap();
            let pools: Vec<_> = result.facts.iter().filter_map(|fact| match &fact.fact {
                ObservableFact::CommittedMana {state, ..} => Some(state), _ => None,
            }).collect();
            assert_eq!(pools.len(), if enabled {2} else {0});
            assert!(pools.iter().all(|state| state.0.is_some()));
        }
    }
}

#[test]
fn mana_lifecycle_60hz_warmup_opt_in_death_and_respawn_start_new_full_life() {
    for enabled in [false, true] {
        let (mut w, mut driver) = world(SingleLaneConfig {
            mana_enabled: enabled, warmup: Fixed64::from_i32(1),
            respawn_delay: Fixed64::from_raw(64), wave_interval: Fixed64::from_i32(10_000),
            heroes: ["training_ranger".into(), "training_luminary".into()], ..three_lane_config()
        }, SimulationTickProfile::Production60Hz);
        driver.step(&mut w, []).unwrap();
        let original = hero_pair(&w)[0];
        assert_eq!(w.read_storage::<Hero>().get(original).unwrap().mana_pool.is_some(), enabled);
        if !enabled { continue; }
        w.write_storage::<Hero>().get_mut(original).unwrap().mana_pool.as_mut().unwrap()
            .spend(Fixed64::from_i32(50)).unwrap();
        let expected = w.read_storage::<Hero>().get(original).unwrap().mana_pool.clone().unwrap();
        for _ in 0..4 { driver.step(&mut w, []).unwrap(); }
        assert_eq!(w.read_storage::<Hero>().get(original).unwrap().mana_pool.as_ref().unwrap(), &expected);
        w.write_resource::<MobaMatch>().config.warmup = Fixed64::ZERO;
        driver.step(&mut w, []).unwrap();
        w.write_storage::<Hero>().get_mut(original).unwrap().level = 3;
        w.write_storage::<CProperty>().get_mut(original).unwrap().hp = Fixed64::ZERO;
        let pos = w.read_storage::<Pos>().get(original).unwrap().0;
        w.write_resource::<Vec<Outcome>>().push(Outcome::Death {pos, ent: original});
        process_outcomes(&mut w, &mut RuntimeEventVecSink::default()).unwrap();
        w.maintain();
        w.write_resource::<GamePause>().is_paused = true;
        driver.step(&mut w, []).unwrap();
        assert!(w.read_resource::<MobaMatch>().heroes[0].entity.is_none());
        w.write_resource::<GamePause>().is_paused = false;
        for _ in 0..5 { driver.step(&mut w, []).unwrap(); }
        let respawned = w.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
        assert_ne!(canonical_entity_id(original), canonical_entity_id(respawned));
        let heroes = w.read_storage::<Hero>();
        let hero = heroes.get(respawned).unwrap();
        let pool = hero.mana_pool.as_ref().unwrap();
        assert_eq!(hero.level, 3);
        assert_eq!(pool.current(), hero.moba_mana_capacity().unwrap());
        assert_eq!(pool.raw_state().2, 0);
    }
}

#[test]
fn mana_cast_same_60hz_batch_uses_ordered_balance_and_cooldown_not_cached_state() {
    use omoba_core::runtime::native::scripting::event::{ScriptEvent, ScriptEventQueue, SkillTarget};
    for (balance, expected_hp, expected_mana) in [(45, 9910, 0), (135, 9700, 45)] {
        let (mut w, mut driver, caster, target) = mana_cast_fixture(balance);
        {
            let mut queue = w.write_resource::<ScriptEventQueue>();
            for skill in ["ranger_shot", "ranger_finisher", "ranger_shot"] {
                queue.push(ScriptEvent::SkillCast { caster, skill_id: skill.into(), target: SkillTarget::Entity(target) });
            }
        }
        driver.step(&mut w, []).unwrap();
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp, Fixed64::from_i32(expected_hp));
        assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current(), Fixed64::from_i32(expected_mana));
        assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_shot"));
        assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_finisher"), balance == 135);
    }
}

#[test]
fn mana_projection_60hz_filtered_cast_and_fractional_state_match_fresh_bootstrap_without_repairs() {
    check_mana_projection_fixture(false);
}

#[test]
fn mana_lifecycle_60hz_filtered_regeneration_matches_authority_without_repairs() {
    check_mana_projection_fixture(true);
}

fn check_mana_projection_fixture(regeneration: bool) {
    use std::collections::BTreeSet;
    use prost::Message;
    use omoba_core::runtime::ability_runtime::ManaPool;
    let (mut authority, mut driver, caster) = if regeneration {
        let (mut w, mut driver) = world(SingleLaneConfig {
            mana_enabled: true, heroes: ["training_ranger".into(), "training_luminary".into()],
            wave_interval: Fixed64::from_i32(10_000), ..three_lane_config()
        }, SimulationTickProfile::Production60Hz);
        driver.step(&mut w, []).unwrap();
        let caster = hero_pair(&w)[0];
        (w, driver, caster)
    } else {
        let (w, driver, caster, _) = mana_cast_fixture(90);
        (w, driver, caster)
    };
    let seed = authority.read_resource::<MobaMatch>().config.seed;
    // A nonzero regeneration remainder must survive baseline and committed state.
    authority.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool = Some(
        ManaPool::from_raw_state(90 * 1024, 280 * 1024, 17).unwrap());
    let began = authority.read_resource::<MobaMatch>().elapsed;
    let first = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, first);
    let allow = TeamProjectorConfig::default().component_allowlist;
    let starts = authority.write_resource::<TeamProjectionRuntime>()
        .build_team_bootstraps(driver.tick() + 1, 60, seed);
    let mut replicas: Vec<_> = starts.into_iter().map(|(team, start)| {
        let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, allow.clone(), BTreeSet::new()).unwrap();
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world, &stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();
        (team, replica, stepper)
    }).collect();
    for tick in 0..12 {
        let inputs = if tick == 0 { vec![(1, PlayerInput {action: Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index: 1, target_entity: None, target_pos: None,
        }))})] } else { vec![] };
        let accepted: Vec<_> = inputs.iter().map(|(player, input)|
            CanonicalAcceptedInput::from_authoritative_acceptance(1, *player, 1, 4,
                canonical_entity_id(caster), None, input.encode_to_vec())).collect();
        let result = driver.step(&mut authority, inputs).unwrap();
        authority.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
        project_tick(&mut authority, result);
        let frames = authority.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected = authority.write_resource::<TeamProjectionRuntime>()
            .build_team_bootstraps(driver.tick() + 1, 60, seed);
        for (team, replica, stepper) in &mut replicas {
            let frame = frames[team].frame.clone();
            assert_eq!(frame.step.as_ref().unwrap().accepted_inputs.len(), usize::from(tick == 0 && *team == 1));
            assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            assert!(matches!(replica.apply_frame(frame, stepper).unwrap(), FrameApplyResult::Applied {..}));
            let expected = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team], allow.clone(), BTreeSet::new()).unwrap();
            if replica.canonical_team_hash() != expected.canonical_team_hash() {
                for (id, entity) in &expected.world().entities {
                    for (schema, bytes) in &entity.components {
                        let actual = replica.world().entities.get(id).and_then(|value| value.components.get(schema));
                        if actual != Some(bytes) {
                            eprintln!("Mana mismatch entity={id} schema={schema:x} actual={:?} expected={:?}",
                                actual.map(|bytes| String::from_utf8_lossy(bytes).to_string()), String::from_utf8_lossy(bytes));
                        }
                    }
                }
            }
            assert_eq!(replica.canonical_team_hash(), expected.canonical_team_hash(), "Mana team {team} tick {}", driver.tick());
        }
    }
    let regenerated = if regeneration {
        (authority.read_resource::<MobaMatch>().elapsed - began).raw()
            * i64::from(omoba_template_ids::MOBA_MANA_REGEN_PER_SECOND)
    } else { 0 };
    assert_eq!(authority.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().raw_state(),
        (45 * 1024 + regenerated, 280 * 1024, 17));
}

#[test]
fn moba_archetypes_four_skills_through_formal_60hz_inputs() {
    for (hero_name, amounts, heals) in [
        ("training_vanguard", [65,110,155,0], [false,true,false,false]),
        ("training_ranger", [90,55,110,210], [false,true,false,false]),
        ("training_luminary", [80,70,180,140], [false,true,false,true]),
    ] {
        let config=SingleLaneConfig {heroes:[hero_name.into(),"training_luminary".into()],
            wave_interval:Fixed64::from_i32(10_000),..three_lane_config()};
        let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
        let mut result=driver.step(&mut w,[]).unwrap();
        let [caster,target]=hero_pair(&w);
        // Public position fixture outside towers/camps; four steps are shorter
        // than an autoattack windup. Enemy defenses are explicit zero fixtures.
        let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
        w.write_storage::<Pos>().get_mut(caster).unwrap().0=source;
        w.write_storage::<Pos>().get_mut(target).unwrap().0=source+
            omoba_sim::Vec2::new(Fixed64::from_i32(290),Fixed64::ZERO);
        let abilities=w.read_storage::<Hero>().get(caster).unwrap().abilities.clone();
        assert_eq!(abilities.len(),4);
        for slot in 0..4 {
            {
                let mut props=w.write_storage::<CProperty>();
                props.get_mut(caster).unwrap().hp=Fixed64::from_i32(100);
                let target_prop=props.get_mut(target).unwrap();
                target_prop.hp=Fixed64::from_i32(1000);
                target_prop.mhp=Fixed64::from_i32(1000);
                target_prop.def_physic=Fixed64::ZERO;
                target_prop.def_magic=Fixed64::ZERO;
            }
            run_committed_visibility_wave_b(&mut w,result.tick,0);
            let point=omoba_template_ids::ability_by_name(&abilities[slot])
                .and_then(omoba_template_ids::active_ability_const).unwrap().target_type==omoba_template_ids::TargetTypeC::Point;
            let target_position=w.read_storage::<Pos>().get(target).unwrap().0;
            result=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
                ability_index:slot as u32,target_entity:(!heals[slot] && !point).then_some(target.id()),
                target_pos:point.then_some(Vec2I {x:target_position.x.raw() as i32,y:target_position.y.raw() as i32}),
            }))})]).unwrap();
            let props=w.read_storage::<CProperty>();
            assert_eq!(props.get(target).unwrap().hp,Fixed64::from_i32(
                if heals[slot] {1000} else {1000-amounts[slot]}),"{hero_name} slot {slot} target HP");
            assert_eq!(props.get(caster).unwrap().hp,Fixed64::from_i32(
                if heals[slot] {100+amounts[slot]} else {100}),"{hero_name} slot {slot} caster HP");
            assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown(&abilities[slot]),
                "{hero_name} slot {slot} must enter cooldown");
        }
    }
}

#[test]
fn area_effects_bot_formal_60hz_hits_multiple_enemies_not_allies_or_outside_radius() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {heroes:["training_ranger".into(),"training_luminary".into()],
        additional_players:vec![
            SingleLanePlayerConfig {player_id:3,team_id:2,hero:"training_luminary".into()},
            SingleLanePlayerConfig {player_id:4,team_id:1,hero:"training_vanguard".into()},
            SingleLanePlayerConfig {player_id:5,team_id:2,hero:"training_luminary".into()},
        ],wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    let mut result=driver.step(&mut w,[]).unwrap();
    let entries:Vec<_>=w.read_resource::<MobaMatch>().heroes.iter().map(|h|(h.player_id,h.entity.unwrap())).collect();
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    for &(player,entity) in &entries {
        // Player3 is beyond the 700 cast range but inside the valid center's
        // radius. Unit range enforcement must not incorrectly reject AoE hits.
        let x=match player {1=>0,2=>590,3=>790,4=>640,5=>1000,_=>unreachable!()};
        w.write_storage::<Pos>().get_mut(entity).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO);
        let mut properties=w.write_storage::<CProperty>();let p=properties.get_mut(entity).unwrap();
        p.hp=Fixed64::from_i32(1000);p.mhp=p.hp;p.def_physic=Fixed64::ZERO;p.def_magic=Fixed64::ZERO;
    }
    let buyer=entries.iter().find(|(p,_)|*p==1).unwrap().1;
    // The effect itself rejects out-of-range points without spending cooldown.
    result=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:2,target_entity:None,target_pos:Some(Vec2I {x:Fixed64::from_i32(701).raw() as i32,y:source.y.raw() as i32}),
    }))})]).unwrap();
    assert!(!w.read_storage::<Hero>().get(buyer).unwrap().is_on_cooldown("ranger_volley"));
    for &(_,entity) in &entries {assert_eq!(w.read_storage::<CProperty>().get(entity).unwrap().hp,Fixed64::from_i32(1000));}
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Carry,lane:2,escort_player_id:None}],
        think_interval_ticks:1,ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),
        ability_policies:vec![BotAbilityPolicy {ability:"ranger_volley".into(),intent:BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:2}}]};
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(inputs.len(),1);
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::CastAbility(c)) if c.ability_index==2 && c.target_entity.is_none() && c.target_pos.is_some()));
    driver.step(&mut w,inputs).unwrap();
    for &(player,entity) in &entries {
        assert_eq!(w.read_storage::<CProperty>().get(entity).unwrap().hp,Fixed64::from_i32(if player==2 || player==3 {890} else {1000}),"player {player}");
    }
    assert!(w.read_storage::<Hero>().get(buyer).unwrap().is_on_cooldown("ranger_volley"));
}

#[test]
fn dash_bot_public_terrain_60hz_skips_blocked_cast_for_recovery_then_relocates() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {heroes:["training_vanguard".into(),"training_luminary".into()],
        mana_enabled:true,wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let [caster,victim]=hero_pair(&w);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    let destination=origin+omoba_sim::Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO);
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(victim).unwrap().0=destination;
    w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::from_i32(100);
    let original=(*w.read_resource::<BlockedRegions>()).clone();
    w.write_resource::<BlockedRegions>().0.push(BlockedRegion {name:"bot-dash-fixture".into(),points:vec![
        vek::Vec2::new(100.0,2400.0),vek::Vec2::new(120.0,2400.0),
        vek::Vec2::new(120.0,2600.0),vek::Vec2::new(100.0,2600.0)]});
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Top,lane:0,escort_player_id:None}],
        think_interval_ticks:1,ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),ability_policies:vec![
            BotAbilityPolicy {ability:"vanguard_resolve".into(),intent:BotAbilityIntent::ApproachEnemyPoint {min_distance:300}},
            BotAbilityPolicy {ability:"vanguard_recover".into(),intent:BotAbilityIntent::SelfHeal {below_hp_per_mille:600}},
        ]};
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let mana=w.read_storage::<Hero>().get(caster).unwrap().mana_pool.clone();
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::CastAbility(c)) if c.ability_index==1));
    assert_eq!(w.read_storage::<Pos>().get(caster).unwrap().0,origin);
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool,mana);
    assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("vanguard_recover"));
    driver.step(&mut w,input).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(210));
    assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("vanguard_resolve"));
    *w.write_resource::<BlockedRegions>()=original;
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let key=canonical_entity_id(victim);
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.remove(&key);
    assert!(!role_bot_inputs(&w,&bots).unwrap().iter().any(|(_,i)|matches!(&i.action,
        Some(PlayerInputEnum::CastAbility(c)) if c.ability_index==3)),"cached target cannot authorize relocation");
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.insert(key);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::CastAbility(c)) if c.ability_index==3));
    driver.step(&mut w,input).unwrap();
    assert_eq!(w.read_storage::<Pos>().get(caster).unwrap().0,destination);
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("vanguard_resolve"));
}

#[test]
fn dash_effect_root_60hz_rejects_without_cost_then_bot_recovers_and_relocates() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {heroes:["training_vanguard".into(),"training_luminary".into()],
        mana_enabled:true,wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let [caster,victim]=hero_pair(&w);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    let destination=origin+omoba_sim::Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO);
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(victim).unwrap().0=destination;
    w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::from_i32(100);
    w.write_resource::<BuffStore>().add(caster,"root",Fixed64::from_raw(256),serde_json::json!({}));
    let mana=w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current();
    let result=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:3,target_entity:None,target_pos:Some(Vec2I {x:destination.x.raw() as i32,y:destination.y.raw() as i32}),
    }))})]).unwrap();
    assert_eq!(w.read_storage::<Pos>().get(caster).unwrap().0,origin);
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current(),mana,
        "full initial pool: failed relocation must roll back reserved cost");
    assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("vanguard_resolve"));
    assert!(!result.facts.iter().any(|fact|matches!(fact.fact,ObservableFact::Ability {source,..}
        if source==canonical_entity_id(caster))),"failed movement emits no successful cast cue");
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Top,lane:0,escort_player_id:None}],
        think_interval_ticks:1,ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),ability_policies:vec![
            BotAbilityPolicy {ability:"vanguard_resolve".into(),intent:BotAbilityIntent::ApproachEnemyPoint {min_distance:300}},
            BotAbilityPolicy {ability:"vanguard_recover".into(),intent:BotAbilityIntent::SelfHeal {below_hp_per_mille:600}},
        ]};
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::CastAbility(cast)) if cast.ability_index==1),
        "skip blocked approach and keep legitimate recovery");
    driver.step(&mut w,input).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(210));
    assert!(w.read_resource::<BuffStore>().is_rooted(caster));
    let mut result=None;
    for _ in 0..25 {result=Some(driver.step(&mut w,[]).unwrap());}
    assert!(!w.read_resource::<BuffStore>().is_rooted(caster));
    run_committed_visibility_wave_b(&mut w,result.unwrap().tick,0);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::CastAbility(cast)) if cast.ability_index==3));
    driver.step(&mut w,input).unwrap();
    assert_eq!(w.read_storage::<Pos>().get(caster).unwrap().0,destination);
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("vanguard_resolve"));
}

#[test]
fn dash_effect_formal_60hz_rejects_wall_and_range_then_bot_relocates() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {heroes:["training_vanguard".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    let mut result=driver.step(&mut w,[]).unwrap();
    let [caster,victim]=hero_pair(&w);
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    let target=source+omoba_sim::Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO);
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=source;
    w.write_storage::<Pos>().get_mut(victim).unwrap().0=target;
    let start_hp=w.read_storage::<CProperty>().get(caster).unwrap().hp;
    let cast=|entity,point| [(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:3,target_entity:entity,target_pos:point,
    }))})];
    for (entity,point) in [(None,None),(Some(victim.id()),None),
        (None,Some(Vec2I {x:0,y:source.y.raw() as i32})),
        (None,Some(Vec2I {x:Fixed64::from_i32(450).raw() as i32+1,y:source.y.raw() as i32}))] {
        result=driver.step(&mut w,cast(entity,point)).unwrap();
        assert_eq!(w.read_storage::<Pos>().get(caster).unwrap().0,source);
        assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("vanguard_resolve"));
    }
    let original=(*w.read_resource::<BlockedRegions>()).clone();
    *w.write_resource::<BlockedRegions>()=BlockedRegions(vec![BlockedRegion {name:"dash-wall".into(),
        points:vec![vek::Vec2::new(100.0,2400.0),vek::Vec2::new(120.0,2400.0),
            vek::Vec2::new(120.0,2600.0),vek::Vec2::new(100.0,2600.0)]}]);
    result=driver.step(&mut w,cast(None,Some(Vec2I {x:target.x.raw() as i32,y:target.y.raw() as i32}))).unwrap();
    assert_eq!(w.read_storage::<Pos>().get(caster).unwrap().0,source,"must not cross a thin wall even with a legal destination");
    assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("vanguard_resolve"));
    *w.write_resource::<BlockedRegions>()=original;
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Top,lane:0,escort_player_id:None}],
        think_interval_ticks:1,ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),
        ability_policies:vec![BotAbilityPolicy {ability:"vanguard_resolve".into(),
            intent:BotAbilityIntent::ApproachEnemyPoint {min_distance:300}}]};
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::CastAbility(c)) if c.ability_index==3 && c.target_entity.is_none() && c.target_pos.is_some()));
    assert_eq!(w.read_storage::<Pos>().get(caster).unwrap().0,source,"Bot must not move the entity itself");
    let relocation = driver.step(&mut w,inputs).unwrap();
    assert_eq!(w.read_storage::<Pos>().get(caster).unwrap().0,target);
    let fact = relocation.facts.iter().find(|fact| matches!(fact.fact,
        ObservableFact::Ability {source, ..} if source == canonical_entity_id(caster))).unwrap();
    assert!(matches!(fact.fact, ObservableFact::Ability {caster_relocation: Some((1, x, y)), ..}
        if (x, y) == (target.x.raw(), target.y.raw())));
    // The same visible caster yields identity for opponents, but only its own
    // team receives this historical relocation point. No hidden target lookup.
    for team in [1, 2] {
        let canonical = canonical_entity_id(caster);
        let projected = TeamViewProjector::new(team, TeamProjectorConfig::default())
            .build_frame(relocation.tick, relocation.tick, &std::collections::BTreeSet::from([canonical]),
                vec![VisibilityTransition::Reveal {canonical_id: canonical, effective_tick: relocation.tick, baseline: Vec::new()}],
                std::slice::from_ref(fact), &ProjectionDependencyGraph::default()).unwrap();
        let events = &projected.frame.step.as_ref().unwrap().public_events;
        let event = events.iter().find(|event| event.event_kind == FactKind::Ability as u32).unwrap();
        let (_, cue) = omoba_core::runtime::presentation_cue::AbilityPresentationCue::from_public_event(
            relocation.tick, event, |_| Some(1)).unwrap();
        assert_eq!(cue.caster_relocation, (team == 1).then_some((target.x.raw(), target.y.raw())));
    }
    assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,start_hp);
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("vanguard_resolve"));
}

#[test]
fn cast_preflight_formal_60hz_rejects_far_unit_and_mistargeted_heal_without_cooldown() {
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let [caster,victim]=hero_pair(&w);
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=source;
    let abilities=w.read_storage::<Hero>().get(caster).unwrap().abilities.clone();
    let range=omoba_template_ids::active_ability_const(omoba_template_ids::ability_by_name(&abilities[0]).unwrap()).unwrap().levels[0].range;
    w.write_storage::<Pos>().get_mut(victim).unwrap().0=source+
        omoba_sim::Vec2::new(Fixed64::from_raw(range.raw()+1),Fixed64::ZERO);
    {
        let mut properties=w.write_storage::<CProperty>();
        properties.get_mut(caster).unwrap().hp=Fixed64::from_i32(100);
        let enemy=properties.get_mut(victim).unwrap();enemy.hp=Fixed64::from_i32(1000);enemy.mhp=enemy.hp;enemy.def_magic=Fixed64::ZERO;
    }
    let cast=|slot,target_entity,target_pos| [(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:slot,target_entity,target_pos,
    }))})];
    driver.step(&mut w,cast(0,Some(victim.id()),None)).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(victim).unwrap().hp,Fixed64::from_i32(1000));
    assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown(&abilities[0]));
    for (entity,point) in [(Some(victim.id()),None),(None,Some(Vec2I {x:source.x.raw() as i32,y:source.y.raw() as i32}))] {
        driver.step(&mut w,cast(1,entity,point)).unwrap();
        assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(100));
        assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown(&abilities[1]));
    }
    w.write_storage::<Pos>().get_mut(victim).unwrap().0=source+omoba_sim::Vec2::new(range,Fixed64::ZERO);
    driver.step(&mut w,cast(0,Some(victim.id()),None)).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(victim).unwrap().hp,Fixed64::from_i32(920));
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown(&abilities[0]));
    driver.step(&mut w,cast(1,None,None)).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(170));
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown(&abilities[1]));
}

#[test]
fn control_effect_root_and_silence_through_formal_60hz_generated_skills() {
    for (hero,slot,status,damage) in [("training_vanguard",2,"root",155),("training_ranger",3,"silence",210)] {
        let (mut w,mut driver)=world(SingleLaneConfig {heroes:[hero.into(),"training_luminary".into()],
            wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();
        let [caster,victim]=hero_pair(&w);
        let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
        for (entity,x) in [(caster,0),(victim,250)] {
            w.write_storage::<Pos>().get_mut(entity).unwrap().0=source+
                omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO);
            let mut props=w.write_storage::<CProperty>();let prop=props.get_mut(entity).unwrap();
            prop.hp=Fixed64::from_i32(10_000);prop.mhp=prop.hp;prop.def_physic=Fixed64::ZERO;
        }
        driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index:slot,target_entity:Some(victim.id()),target_pos:None,
        }))})]).unwrap();
        assert_eq!(w.read_storage::<CProperty>().get(victim).unwrap().hp,Fixed64::from_i32(10_000-damage));
        assert!(w.read_resource::<BuffStore>().has(victim,status));
        assert!(!w.read_resource::<BuffStore>().is_stunned(victim));
        assert_eq!(w.read_resource::<BuffStore>().is_rooted(victim),status=="root");
        assert_eq!(w.read_resource::<BuffStore>().is_silenced(victim),status=="silence");
        let public=omoba_core::runtime::buff_visual_state::BuffVisualState::from_store(&w.read_resource::<BuffStore>(),victim);
        assert!(public.render_buffs().iter().any(|buff|buff.buff_id==status && buff.payload_json.is_empty()));
        let before=w.read_storage::<Pos>().get(victim).unwrap().0;
        let clock=w.read_storage::<TAttack>().get(victim).unwrap().asd_count;
        let move_input=||[(2,PlayerInput {action:Some(PlayerInputEnum::MoveTo(MoveTo {
            target:Some(Vec2I {x:600*1024,y:2500*1024}),queued:false,
        }))})];
        driver.step(&mut w,move_input()).unwrap();
        for _ in 0..5 {driver.step(&mut w,[]).unwrap();}
        let moved=w.read_storage::<Pos>().get(victim).unwrap().0!=before;
        assert_eq!(moved,status=="silence","root alone stops movement; silence must not");
        assert_ne!(w.read_storage::<TAttack>().get(victim).unwrap().asd_count,clock,"neither status freezes attack clock");
        let cast=||[(2,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index:1,target_entity:None,target_pos:None,
        }))})];
        let hp=w.read_storage::<CProperty>().get(victim).unwrap().hp;
        driver.step(&mut w,cast()).unwrap();
        let permitted=status=="root";
        assert_eq!(w.read_storage::<Hero>().get(victim).unwrap().is_on_cooldown("lumen_touch"),permitted);
        assert_eq!(w.read_storage::<CProperty>().get(victim).unwrap().hp,hp+if permitted {Fixed64::from_i32(70)} else {Fixed64::ZERO});
        // Current life and standard status expire through the normal tick, not a test removal.
        for _ in 0..65 {driver.step(&mut w,[]).unwrap();}
        assert!(!w.read_resource::<BuffStore>().has(victim,status));
        let before=w.read_storage::<Pos>().get(victim).unwrap().0;
        driver.step(&mut w,move_input()).unwrap();
        for _ in 0..3 {driver.step(&mut w,[]).unwrap();}
        if status=="root" {assert_ne!(w.read_storage::<Pos>().get(victim).unwrap().0,before);}
        if status=="silence" {
            let hp=w.read_storage::<CProperty>().get(victim).unwrap().hp;
            driver.step(&mut w,cast()).unwrap();
            assert_eq!(w.read_storage::<CProperty>().get(victim).unwrap().hp,hp+Fixed64::from_i32(70));
            assert!(w.read_storage::<Hero>().get(victim).unwrap().is_on_cooldown("lumen_touch"));
        }
    }
}

#[test]
fn role_bot_sustain_navigation_60hz_all_roles_defend_hold_and_resume() {
    use omoba_core::runtime::native::moba_match::bots::*;
    for role in [BotRole::Top,BotRole::Mid,BotRole::Carry,BotRole::Support,BotRole::Jungle] {
        for healing in [false,true] {
            let (mut w,mut driver)=world(SingleLaneConfig {base_recovery_enabled:true,
                heroes:["training_vanguard".into(),"training_luminary".into()],
                wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
            driver.step(&mut w,[]).unwrap();
            let [bot,enemy]=hero_pair(&w);
            let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
            w.write_storage::<Pos>().get_mut(bot).unwrap().0=origin;
            w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+
                omoba_sim::Vec2::new(Fixed64::from_i32(250),Fixed64::ZERO);
            w.write_storage::<CProperty>().get_mut(bot).unwrap().hp=Fixed64::from_i32(100);
            let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role,lane:0,escort_player_id:None}],
                think_interval_ticks:1,ability_learning:Vec::new(),item_builds:Vec::new(),
                sustain:Some(BotSustainPolicy {recall_below_hp_per_mille:350,leave_base_at_hp_per_mille:850,threat_radius:1000,mana:None}),
                ability_policies:if healing {vec![
                    BotAbilityPolicy {ability:"vanguard_strike".into(),intent:BotAbilityIntent::EnemyUnit},
                    BotAbilityPolicy {ability:"vanguard_recover".into(),intent:BotAbilityIntent::SelfHeal {below_hp_per_mille:600}},
                ]} else {Vec::new()}};
            run_committed_visibility_wave_b(&mut w,driver.tick(),0);
            let escape=role_bot_inputs(&w,&bots).unwrap();
            let Some(PlayerInputEnum::MoveTo(m))=&escape[0].1.action else {panic!("{role:?}: reachable escape");};
            let target=m.target.as_ref().unwrap();
            let home=omoba_sim::Vec2::new(Fixed64::from_raw(i64::from(target.x)),Fixed64::from_raw(i64::from(target.y)));
            let original=(*w.read_resource::<BlockedRegions>()).clone();
            let x=home.x.to_f32_for_render();let y=home.y.to_f32_for_render();
            w.write_resource::<BlockedRegions>().0.push(BlockedRegion {name:"sustain-home-fixture".into(),points:vec![
                vek::Vec2::new(x-40.0,y-40.0),vek::Vec2::new(x+40.0,y-40.0),
                vek::Vec2::new(x+40.0,y+40.0),vek::Vec2::new(x-40.0,y+40.0)]});
            // An old offensive order must not survive a failed escape forever.
            w.write_storage::<HeroCommandQueue>().insert(bot,HeroCommandQueue {
                active:Some(HeroCommand::AttackTarget {target:enemy,chase_origin:None}),
                queued:vec![HeroCommand::AttackMove {pos:origin}],
            }).unwrap();
            if healing {
                let input=role_bot_inputs(&w,&bots).unwrap();
                assert_eq!(input.len(),1);
                assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::CastAbility(c)) if c.ability_index==1),"{role:?}: recovery before hold, never offense");
                driver.step(&mut w,input).unwrap();
                assert!(w.read_storage::<CProperty>().get(bot).unwrap().hp>Fixed64::from_i32(100));
                assert!(!w.read_storage::<Hero>().get(bot).unwrap().is_on_cooldown("vanguard_strike"));
                run_committed_visibility_wave_b(&mut w,driver.tick(),0);
            }
            let input=role_bot_inputs(&w,&bots).unwrap();
            assert_eq!(input.len(),1);
            assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::HoldPosition(_))),"{role:?}: unreachable home stops stale aggression");
            driver.step(&mut w,input).unwrap();
            let held=w.read_storage::<Pos>().get(bot).unwrap().0;
            for _ in 0..3 {
                run_committed_visibility_wave_b(&mut w,driver.tick(),0);
                assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"{role:?}: hold must not spam");
                driver.step(&mut w,[]).unwrap();
                assert_eq!(w.read_storage::<Pos>().get(bot).unwrap().0,held);
            }
            *w.write_resource::<BlockedRegions>()=original;
            run_committed_visibility_wave_b(&mut w,driver.tick(),0);
            let input=role_bot_inputs(&w,&bots).unwrap();
            assert_eq!(input.len(),1);
            assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::MoveTo(m))
                if m.target.as_ref().is_some_and(|p|i64::from(p.x)==home.x.raw() && i64::from(p.y)==home.y.raw())));
            driver.step(&mut w,input).unwrap();
            driver.step(&mut w,[]).unwrap();
            assert_ne!(w.read_storage::<Pos>().get(bot).unwrap().0,held,"{role:?}: ordinary movement resumes, not teleport");
        }
    }
}

#[test]
fn role_bot_rooted_sustain_60hz_casts_recovery_instead_of_blocked_escape() {
    use omoba_core::runtime::native::moba_match::bots::*;
    for threatened in [false,true] {
        let (mut w,mut driver)=world(SingleLaneConfig {base_recovery_enabled:true,
            heroes:["training_vanguard".into(),"training_luminary".into()],
            wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
        let result=driver.step(&mut w,[]).unwrap();
        let [bot,enemy]=hero_pair(&w);
        let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
        w.write_storage::<Pos>().get_mut(bot).unwrap().0=origin;
        w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+
            omoba_sim::Vec2::new(Fixed64::from_i32(if threatened {250} else {2000}),Fixed64::ZERO);
        w.write_storage::<CProperty>().get_mut(bot).unwrap().hp=Fixed64::from_i32(100);
        let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Top,lane:0,escort_player_id:None}],
            think_interval_ticks:1,ability_learning:Vec::new(),item_builds:Vec::new(),
            sustain:Some(BotSustainPolicy {recall_below_hp_per_mille:350,leave_base_at_hp_per_mille:850,threat_radius:1000,mana:None}),
            ability_policies:vec![
                BotAbilityPolicy {ability:"vanguard_strike".into(),intent:BotAbilityIntent::EnemyUnit},
                BotAbilityPolicy {ability:"vanguard_recover".into(),intent:BotAbilityIntent::SelfHeal {below_hp_per_mille:600}},
            ]};
        run_committed_visibility_wave_b(&mut w,result.tick,0);
        let escape=role_bot_inputs(&w,&bots).unwrap();
        assert!(matches!(&escape[0].1.action,Some(PlayerInputEnum::MoveTo(_)))==threatened);
        assert!(matches!(&escape[0].1.action,Some(PlayerInputEnum::Recall(_)))==!threatened);
        w.write_resource::<BuffStore>().add(bot,"root",Fixed64::from_raw(256),serde_json::json!({}));
        let input=role_bot_inputs(&w,&bots).unwrap();
        assert_eq!(input.len(),1);
        assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::CastAbility(c)) if c.ability_index==1),
            "blocked sustain must allow self recovery, never the earlier offensive policy");
        let result=driver.step(&mut w,input).unwrap();
        run_committed_visibility_wave_b(&mut w,result.tick,0);
        assert_eq!(w.read_storage::<CProperty>().get(bot).unwrap().hp,Fixed64::from_i32(210));
        assert_eq!(w.read_storage::<Pos>().get(bot).unwrap().0,origin);
        assert!(w.read_storage::<Hero>().get(bot).unwrap().is_on_cooldown("vanguard_recover"));
        assert!(!w.read_storage::<Hero>().get(bot).unwrap().is_on_cooldown("vanguard_strike"));
        for _ in 0..4 {
            let input=role_bot_inputs(&w,&bots).unwrap();
            assert!(input.is_empty(),"cooling recovery must not turn blocked sustain into attack/movement");
            let result=driver.step(&mut w,input).unwrap();
            run_committed_visibility_wave_b(&mut w,result.tick,0);
            assert_eq!(w.read_storage::<Pos>().get(bot).unwrap().0,origin);
        }
        for _ in 0..25 {
            let result=driver.step(&mut w,[]).unwrap();run_committed_visibility_wave_b(&mut w,result.tick,0);
        }
        assert!(!w.read_resource::<BuffStore>().is_rooted(bot));
        let input=role_bot_inputs(&w,&bots).unwrap();
        assert_eq!(input.len(),1);
        assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::MoveTo(_)))==threatened);
        assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::Recall(_)))==!threatened);
        driver.step(&mut w,input).unwrap();
        if threatened {assert!(matches!(w.read_storage::<HeroCommandQueue>().get(bot).unwrap().active,
            Some(HeroCommand::MoveTo {..})));}
        else {assert!(w.read_resource::<MobaMatch>().is_recalling(bot));}
    }
}

#[test]
fn role_bot_rooted_travel_60hz_waits_without_replacing_orders_then_resumes() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {
        additional_players:vec![SingleLanePlayerConfig {player_id:3,team_id:1,hero:"training_luminary".into()}],
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();
    let slots=w.read_resource::<MobaMatch>().heroes.clone();
    let bot=slots.iter().find(|s|s.player_id==1).unwrap().entity.unwrap();
    let ally=slots.iter().find(|s|s.player_id==3).unwrap().entity.unwrap();
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    w.write_storage::<Pos>().get_mut(bot).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(ally).unwrap().0=origin+
        omoba_sim::Vec2::new(Fixed64::from_i32(400),Fixed64::ZERO);
    let mut bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Top,lane:0,escort_player_id:None}],
        think_interval_ticks:1,ability_learning:Vec::new(),item_builds:Vec::new(),ability_policies:Vec::new(),sustain:None};
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    // A freshly spawned hero need not have a command queue yet. Establish
    // a real existing order through the formal input path before testing preservation.
    let result=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::HoldPosition(
        omoba_core::game_proto::HoldPosition {queued:false}))})]).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let original=w.read_storage::<HeroCommandQueue>().get(bot).unwrap().active;
    w.write_resource::<BuffStore>().add(bot,"root",Fixed64::from_raw(128),serde_json::json!({}));
    for role in [BotRole::Top,BotRole::Mid,BotRole::Carry,BotRole::Support,BotRole::Jungle] {
        bots.assignments[0].role=role;
        bots.assignments[0].escort_player_id=(role==BotRole::Support).then_some(3);
        assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"{role:?} must not request new travel while rooted");
        assert_eq!(w.read_storage::<HeroCommandQueue>().get(bot).unwrap().active,original);
    }
    for _ in 0..14 {
        let result=driver.step(&mut w,[]).unwrap();run_committed_visibility_wave_b(&mut w,result.tick,0);
    }
    assert!(!w.read_resource::<BuffStore>().is_rooted(bot));
    for role in [BotRole::Top,BotRole::Mid,BotRole::Carry,BotRole::Support,BotRole::Jungle] {
        bots.assignments[0].role=role;
        bots.assignments[0].escort_player_id=(role==BotRole::Support).then_some(3);
        let inputs=role_bot_inputs(&w,&bots).unwrap();
        assert_eq!(inputs.len(),1,"{role:?} resumes from current disclosure");
        assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::MoveTo(_)|PlayerInputEnum::AttackMove(_))));
    }
}

#[test]
fn role_bot_control_state_waits_and_resumes_through_formal_60hz() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {
        heroes:["training_vanguard".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()
    },SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();
    let [caster,bot]=hero_pair(&w);
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    for (entity,x) in [(caster,0),(bot,250)] {
        w.write_storage::<Pos>().get_mut(entity).unwrap().0=source+
            omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO);
        let mut props=w.write_storage::<CProperty>();let prop=props.get_mut(entity).unwrap();
        prop.hp=Fixed64::from_i32(10_000);prop.mhp=prop.hp;prop.def_physic=Fixed64::ZERO;
    }
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let mut bots=RoleBotConfig {assignments:vec![BotAssignment {
        player_id:2,role:BotRole::Mid,lane:1,escort_player_id:None,
    }],think_interval_ticks:1,sustain:None,item_builds:Vec::new(),ability_learning:Vec::new(),
        ability_policies:vec![BotAbilityPolicy {ability:"lumen_bolt".into(),intent:BotAbilityIntent::EnemyUnit}]};
    assert!(matches!(&role_bot_inputs(&w,&bots).unwrap()[0].1.action,
        Some(PlayerInputEnum::CastAbility(cast)) if cast.target_entity==Some(caster.id())));
    // Real generated enemy spell applies control; the bot never reads enemy Buffs.
    let result=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:0,target_entity:Some(bot.id()),target_pos:None,
    }))})]).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(w.read_resource::<BuffStore>().is_stunned(bot));
    let frozen=w.read_storage::<Pos>().get(bot).unwrap().0;
    let clock=w.read_storage::<TAttack>().get(bot).unwrap().asd_count;
    // Applies to every configured role, including Jungle with compiled camps.
    for role in [BotRole::Top,BotRole::Mid,BotRole::Carry,BotRole::Support,BotRole::Jungle] {
        bots.assignments[0].role=role;
        assert!(role_bot_inputs(&w,&bots).unwrap().is_empty());
    }
    bots.assignments[0].role=BotRole::Mid;
    let mut waits=0;
    while w.read_resource::<BuffStore>().is_stunned(bot) {
        let input=role_bot_inputs(&w,&bots).unwrap();
        assert!(input.is_empty(),"control must not emit rejected spell or replace orders");
        assert_eq!(w.read_storage::<Pos>().get(bot).unwrap().0,frozen);
        assert_eq!(w.read_storage::<TAttack>().get(bot).unwrap().asd_count,clock);
        let result=driver.step(&mut w,input).unwrap();
        run_committed_visibility_wave_b(&mut w,result.tick,0);
        waits+=1;assert!(waits<90,"real stun must expire");
    }
    assert!(waits>=30);
    // Silence alone permits an ordinary attack, then expiry restores spells.
    w.write_resource::<BuffStore>().add(bot,"silence",Fixed64::from_raw(128),serde_json::json!({}));
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::AttackTarget(attack)) if attack.target_id==caster.id()));
    let result=driver.step(&mut w,input).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(!w.read_storage::<Hero>().get(bot).unwrap().is_on_cooldown("lumen_bolt"));
    for _ in 0..10 {
        let input=role_bot_inputs(&w,&bots).unwrap();
        if w.read_resource::<BuffStore>().is_silenced(bot) {
            assert!(!input.iter().any(|(_,input)|matches!(input.action,Some(PlayerInputEnum::CastAbility(_)))));
        }
        let result=driver.step(&mut w,[]).unwrap();
        run_committed_visibility_wave_b(&mut w,result.tick,0);
    }
    assert!(!w.read_resource::<BuffStore>().is_silenced(bot));
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::CastAbility(cast)) if cast.target_entity==Some(caster.id())));
    let hp=w.read_storage::<CProperty>().get(caster).unwrap().hp;
    driver.step(&mut w,input).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,hp-Fixed64::from_i32(80));
    assert!(w.read_storage::<Hero>().get(bot).unwrap().is_on_cooldown("lumen_bolt"));
}

#[test]
fn stun_effect_formal_60hz_blocks_actions_expires_and_preserves_longer_control() {
    let (mut w,mut driver)=world(SingleLaneConfig {heroes:["training_vanguard".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let [caster,victim]=hero_pair(&w);
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    for (entity,x) in [(caster,0),(victim,301)] {
        w.write_storage::<Pos>().get_mut(entity).unwrap().0=source+
            omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO);
        let mut props=w.write_storage::<CProperty>();let prop=props.get_mut(entity).unwrap();
        prop.hp=Fixed64::from_i32(10_000);prop.mhp=prop.hp;prop.def_physic=Fixed64::ZERO;
    }
    let strike=||[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:0,target_entity:Some(victim.id()),target_pos:None,
    }))})];
    driver.step(&mut w,strike()).unwrap();
    assert!(!w.read_resource::<BuffStore>().is_stunned(victim));
    assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("vanguard_strike"));
    w.write_storage::<Pos>().get_mut(victim).unwrap().0=source+
        omoba_sim::Vec2::new(Fixed64::from_i32(250),Fixed64::ZERO);
    driver.step(&mut w,strike()).unwrap();
    assert!(w.read_resource::<BuffStore>().is_stunned(victim));
    assert_eq!(w.read_storage::<CProperty>().get(victim).unwrap().hp,Fixed64::from_i32(9935));
    let public=omoba_core::runtime::buff_visual_state::BuffVisualState::from_store(&w.read_resource::<BuffStore>(),victim);
    assert!(public.render_buffs().iter().any(|buff|buff.buff_id=="stun" && buff.payload_json.is_empty()));
    driver.step(&mut w,[(2,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:1,target_entity:None,target_pos:None,
    }))})]).unwrap();
    assert!(!w.read_storage::<Hero>().get(victim).unwrap().is_on_cooldown("lumen_touch"));
    assert_eq!(w.read_storage::<CProperty>().get(victim).unwrap().hp,Fixed64::from_i32(9935));
    driver.step(&mut w,[(2,PlayerInput {action:Some(PlayerInputEnum::MoveTo(MoveTo {
        target:Some(Vec2I {x:600*1024,y:2500*1024}),queued:false,
    }))})]).unwrap();
    let frozen=w.read_storage::<Pos>().get(victim).unwrap().0;
    let attack=w.read_storage::<TAttack>().get(victim).unwrap().asd_count;
    for _ in 0..15 {
        driver.step(&mut w,[]).unwrap();
        assert_eq!(w.read_storage::<Pos>().get(victim).unwrap().0,frozen);
        assert_eq!(w.read_storage::<TAttack>().get(victim).unwrap().asd_count,attack);
    }
    for _ in 0..65 {driver.step(&mut w,[]).unwrap();}
    assert!(!w.read_resource::<BuffStore>().is_stunned(victim));
    assert_ne!(w.read_storage::<Pos>().get(victim).unwrap().0,frozen);
    w.write_storage::<Pos>().get_mut(victim).unwrap().0=frozen;
    w.write_storage::<Hero>().get_mut(caster).unwrap().ability_cooldowns.clear();
    w.write_resource::<BuffStore>().add(victim,"stun",Fixed64::from_i32(3),serde_json::json!({}));
    let before=w.read_storage::<CProperty>().get(victim).unwrap().hp;
    driver.step(&mut w,strike()).unwrap();
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("vanguard_strike"));
    assert_eq!(w.read_storage::<CProperty>().get(victim).unwrap().hp,before-Fixed64::from_i32(65));
    assert!(w.read_resource::<BuffStore>().iter_for(victim).any(|(id,entry)|
        id=="stun" && entry.remaining>Fixed64::from_i32(2)));
}

#[test]
fn slow_effect_formal_60hz_uses_strongest_source_and_expires_in_real_movement() {
    use omoba_core::runtime::ability_runtime::{BuffStore,UnitStats};
    let (mut w,mut driver)=world(SingleLaneConfig {heroes:["training_ranger".into(),"training_luminary".into()],
        additional_players:vec![SingleLanePlayerConfig {player_id:3,team_id:1,hero:"training_ranger".into()}],
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let entries:Vec<_>=w.read_resource::<MobaMatch>().heroes.iter().map(|h|(h.player_id,h.entity.unwrap())).collect();
    let hero=|player|entries.iter().find(|(id,_)|*id==player).unwrap().1;
    let (caster,victim,ally)=(hero(1),hero(2),hero(3));
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    for &(player,entity) in &entries {
        w.write_storage::<Pos>().get_mut(entity).unwrap().0=source+
            omoba_sim::Vec2::new(Fixed64::from_i32(if player==2 {751} else {0}),Fixed64::ZERO);
        let mut props=w.write_storage::<CProperty>();let property=props.get_mut(entity).unwrap();
        property.hp=Fixed64::from_i32(10_000);property.mhp=property.hp;property.def_physic=Fixed64::ZERO;
    }
    let cast=|player,target| [(player,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:0,target_entity:Some(target),target_pos:None,
    }))})];
    for target in [ally.id(),victim.id()] {
        driver.step(&mut w,cast(1,target)).unwrap();
        assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_shot"));
        assert!(!w.read_resource::<BuffStore>().has_any(victim));
        assert_eq!(w.read_storage::<CProperty>().get(victim).unwrap().hp,Fixed64::from_i32(10_000));
    }
    w.write_storage::<Pos>().get_mut(victim).unwrap().0=source+
        omoba_sim::Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO);
    driver.step(&mut w,cast(1,victim.id())).unwrap();
    let base=w.read_storage::<CProperty>().get(victim).unwrap().msd;
    let speed=|w:&World|UnitStats::from_refs(&*w.read_resource::<BuffStore>(),false).final_move_speed(base,victim);
    assert_eq!(speed(&w),base*Fixed64::from_raw(768));
    w.write_storage::<Hero>().get_mut(ally).unwrap().ability_levels.insert("ranger_shot".into(),4);
    driver.step(&mut w,cast(3,victim.id())).unwrap();
    let ability=omoba_template_ids::active_ability_const(omoba_template_ids::ability_by_name("ranger_shot").unwrap()).unwrap();
    let rank4=ability.extras.iter().find(|(key,_)|*key=="slow_reduction").unwrap().1[3];
    let reduced=base*(Fixed64::ONE-rank4); // Use the generator's Q10 rounding.
    assert_eq!(speed(&w),reduced,"two sources must not add to a 65% slow");
    let buffs=w.read_resource::<BuffStore>();
    assert_eq!(buffs.iter_for(victim).filter(|(id,_)|id.starts_with("generic_slow:")).count(),2);
    let visual=omoba_core::runtime::buff_visual_state::BuffVisualState::from_store(&buffs,victim);
    assert!(visual.encode().is_some());
    let public_slow:Vec<_>=visual.render_buffs().into_iter().filter(|buff|buff.buff_id=="slow").collect();
    assert_eq!(public_slow.len(),1,"multiple real sources share one public identity");
    assert!(public_slow[0].payload_json.is_empty());
    for (id,entry) in buffs.iter_for(victim) {
        if id.starts_with("generic_slow:") {assert!(entry.payload["move_speed_bonus"].is_i64());}
    }
    drop(buffs);
    driver.step(&mut w,[(2,PlayerInput {action:Some(PlayerInputEnum::MoveTo(MoveTo {
        target:Some(Vec2I {x:600*1024,y:2500*1024}),queued:false,
    }))})]).unwrap();
    let before=w.read_storage::<Pos>().get(victim).unwrap().0;
    driver.step(&mut w,[]).unwrap();
    let after=w.read_storage::<Pos>().get(victim).unwrap().0;
    // Navigation may choose a local waypoint rather than the final input goal.
    // Check the actual waypoint used by this step, without bypassing the planner.
    let waypoint=w.read_storage::<MoveTarget>().get(victim).unwrap().0;
    let expected=(waypoint-before).normalized()*(reduced*w.read_resource::<DeltaTime>().0);
    assert_eq!(after-before,expected,"formal movement must consume the reduced speed toward its real navigation waypoint");
    for _ in 0..200 {driver.step(&mut w,[]).unwrap();}
    assert_eq!(speed(&w),base);
    assert!(!w.read_resource::<BuffStore>().iter_for(victim).any(|(id,_)|id.starts_with("generic_slow:")));
}

#[test]
fn role_bot_sustain_recall_and_authoritative_base_recovery_at_60hz() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {base_recovery_enabled:true,
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    let mut result=driver.step(&mut w,[]).unwrap();
    let [caster,_]=hero_pair(&w);
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=source;
    w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::from_i32(100);
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Mid,lane:1,escort_player_id:None}],
        think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),item_builds:Vec::new(),
        sustain:Some(BotSustainPolicy {recall_below_hp_per_mille:350,leave_base_at_hp_per_mille:850,threat_radius:1000,mana:None})};
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::Recall(_))));
    assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(100));
    assert_eq!(w.read_storage::<Pos>().get(caster).unwrap().0,source,"planner must not teleport");
    result=driver.step(&mut w,inputs).unwrap();
    assert!(w.read_resource::<MobaMatch>().is_recalling(caster));
    for _ in 0..600 {
        run_committed_visibility_wave_b(&mut w,result.tick,0);
        assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"do not interrupt own Recall");
        result=driver.step(&mut w,[]).unwrap();
        if !w.read_resource::<MobaMatch>().is_recalling(caster) {break;}
    }
    assert!(!w.read_resource::<MobaMatch>().is_recalling(caster),"formal channel completes");
    let base=w.read_resource::<MobaMatch>().bases[0].unwrap();
    let home=w.read_storage::<Pos>().get(base).unwrap().0;
    assert_eq!(w.read_storage::<Pos>().get(caster).unwrap().0,home);
    let initial=w.read_storage::<CProperty>().get(caster).unwrap().hp;
    assert!(initial>Fixed64::from_i32(100));
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let hold=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&hold[0].1.action,Some(PlayerInputEnum::HoldPosition(h)) if !h.queued));
    result=driver.step(&mut w,hold).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"persistent recovery Hold is not resent");
    let before=w.read_storage::<CProperty>().get(caster).unwrap().hp;
    w.write_resource::<GamePause>().is_paused=true;
    result=driver.step(&mut w,[]).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,before,"pause cannot heal");
    w.write_resource::<GamePause>().is_paused=false;
    for _ in 0..220 {result=driver.step(&mut w,[]).unwrap();}
    let (hp,mhp)={let props=w.read_storage::<CProperty>();let health=props.get(caster).unwrap();(health.hp,health.mhp)};
    assert!(hp>=mhp*Fixed64::from_raw(871));
    assert!(hp<=mhp);
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(role_bot_inputs(&w,&bots).unwrap().iter().any(|(_,i)|matches!(i.action,Some(PlayerInputEnum::AttackMove(_)))),
        "healthy Bot returns to its role");
    {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(caster).unwrap();p.hp=p.mhp-Fixed64::ONE;}
    driver.step(&mut w,[]).unwrap();
    {let props=w.read_storage::<CProperty>();let p=props.get(caster).unwrap();assert_eq!(p.hp,p.mhp,"recovery caps at max HP");}
    // Zero HP never becomes a resurrection through the recovery rule.
    w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::ZERO;
    driver.step(&mut w,[]).unwrap();
    assert!(w.read_storage::<CProperty>().get(caster).is_none_or(|p|p.hp==Fixed64::ZERO));
}

#[test]
fn mana_sustain_bot_formal_recall_recovery_hold_and_leave_at_60hz() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::runtime::ability_runtime::ManaPool;
    let (mut w,mut driver)=world(SingleLaneConfig {mana_enabled:true,base_recovery_enabled:true,
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    let mut result=driver.step(&mut w,[]).unwrap();
    let [caster,_]=hero_pair(&w);
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=source;
    {let mut heroes=w.write_storage::<Hero>();let own=heroes.get_mut(caster).unwrap();
        own.mana_pool=Some(ManaPool::new(Fixed64::ZERO,own.moba_mana_capacity().unwrap()).unwrap());}
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Mid,lane:1,escort_player_id:None}],
        think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),item_builds:Vec::new(),
        sustain:Some(BotSustainPolicy {recall_below_hp_per_mille:350,leave_base_at_hp_per_mille:850,threat_radius:1000,
            mana:Some(BotManaSustainPolicy {recall_below_per_mille:200,leave_base_at_per_mille:850})})};
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::Recall(_))));
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current(),Fixed64::ZERO);
    result=driver.step(&mut w,inputs).unwrap();
    assert!(w.read_resource::<MobaMatch>().is_recalling(caster));
    for _ in 0..600 {
        run_committed_visibility_wave_b(&mut w,result.tick,0);
        assert!(role_bot_inputs(&w,&bots).unwrap().is_empty());
        result=driver.step(&mut w,[]).unwrap();
        if !w.read_resource::<MobaMatch>().is_recalling(caster) {break;}
    }
    assert!(!w.read_resource::<MobaMatch>().is_recalling(caster));
    let base=w.read_resource::<MobaMatch>().bases[0].unwrap();
    let home=w.read_storage::<Pos>().get(base).unwrap().0;
    assert_eq!(w.read_storage::<Pos>().get(caster).unwrap().0,home);
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let hold=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&hold[0].1.action,Some(PlayerInputEnum::HoldPosition(h)) if !h.queued));
    result=driver.step(&mut w,hold).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"low mana holds even with full HP without resending");
    let mut expected=w.read_storage::<Hero>().get(caster).unwrap().mana_pool.clone().unwrap();
    w.write_resource::<GamePause>().is_paused=true;
    driver.step(&mut w,[]).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    w.write_resource::<GamePause>().is_paused=false;
    let rate=Fixed64::from_i32((omoba_template_ids::MOBA_MANA_REGEN_PER_SECOND
        +omoba_template_ids::MOBA_BASE_RECOVERY_MANA_PER_SECOND) as i32);
    for _ in 0..1000 {
        let before=w.read_resource::<MobaMatch>().elapsed;
        result=driver.step(&mut w,[]).unwrap();
        expected.regenerate(rate,w.read_resource::<MobaMatch>().elapsed-before).unwrap();
        assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
        if i128::from(expected.current().raw())*1000>=i128::from(expected.maximum().raw())*850 {break;}
    }
    assert!(i128::from(expected.current().raw())*1000>=i128::from(expected.maximum().raw())*850);
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(role_bot_inputs(&w,&bots).unwrap().iter().any(|(_,input)|matches!(input.action,Some(PlayerInputEnum::AttackMove(_)))));
}

#[test]
fn mana_sustain_base_bonus_is_opt_in_own_home_only_and_caps_at_maximum() {
    use omoba_core::runtime::ability_runtime::ManaPool;
    for (enabled,location) in [(false,0),(true,0),(true,1),(true,2)] {
        let (mut w,mut driver)=world(SingleLaneConfig {mana_enabled:true,base_recovery_enabled:enabled,
            wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();
        let [caster,_]=hero_pair(&w);
        let bases=w.read_resource::<MobaMatch>().bases;
        if location!=0 {w.write_storage::<Pos>().get_mut(caster).unwrap().0=if location==1 {
            w.read_storage::<Pos>().get(bases[1].unwrap()).unwrap().0
        } else {omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500))};}
        let maximum=w.read_storage::<Hero>().get(caster).unwrap().moba_mana_capacity().unwrap();
        let mut expected=ManaPool::new(Fixed64::ZERO,maximum).unwrap();
        w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool=Some(expected.clone());
        let before=w.read_resource::<MobaMatch>().elapsed;
        driver.step(&mut w,[]).unwrap();
        let rate=omoba_template_ids::MOBA_MANA_REGEN_PER_SECOND
            +if enabled && location==0 {omoba_template_ids::MOBA_BASE_RECOVERY_MANA_PER_SECOND} else {0};
        expected.regenerate(Fixed64::from_i32(rate as i32),w.read_resource::<MobaMatch>().elapsed-before).unwrap();
        assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
        if enabled && location==0 {
            w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool=
                Some(ManaPool::new(maximum-Fixed64::from_raw(1),maximum).unwrap());
            driver.step(&mut w,[]).unwrap();
            assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().raw_state(),(maximum.raw(),maximum.raw(),0));
        }
    }
}

#[test]
fn role_bot_items_formal_shop_at_60hz_is_owner_only_and_non_mutating() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {base_recovery_enabled:true,
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    let mut result=driver.step(&mut w,[]).unwrap();
    let [buyer,enemy]=hero_pair(&w);
    let base=w.read_resource::<MobaMatch>().bases[0].unwrap();
    let home=w.read_storage::<Pos>().get(base).unwrap().0;
    w.write_storage::<Pos>().get_mut(buyer).unwrap().0=home;
    // Explicit fixture balance, never supplied or modified by the Bot planner.
    w.write_storage::<Gold>().get_mut(buyer).unwrap().0=1250;
    w.write_storage::<CProperty>().get_mut(buyer).unwrap().hp=Fixed64::from_i32(100);
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Carry,lane:2,escort_player_id:None}],
        think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:Some(BotSustainPolicy {recall_below_hp_per_mille:350,leave_base_at_hp_per_mille:850,threat_radius:1000,mana:None}),
        item_builds:vec![BotItemBuild {role:BotRole::Carry,items:vec!["moba_greatsword".into(),"moba_boots".into()],return_to_shop:None,active_use:None}]};
    let enemy_gold=w.read_storage::<Gold>().get(enemy).unwrap().0;
    let enemy_items=serde_json::to_value(w.read_storage::<Inventory>().get(enemy).unwrap()).unwrap();
    for (id,balance) in [("moba_sword",900),("moba_sword",550),("moba_greatsword",300),("moba_boots",0)] {
        run_committed_visibility_wave_b(&mut w,result.tick,0);
        let old_balance=w.read_storage::<Gold>().get(buyer).unwrap().0;
        let old_items=serde_json::to_value(w.read_storage::<Inventory>().get(buyer).unwrap()).unwrap();
        let inputs=role_bot_inputs(&w,&bots).unwrap();
        assert_eq!(inputs.len(),1,"purchases precede low-HP base hold");
        assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::ItemBuy(buy)) if buy.item_id==id));
        assert_eq!(w.read_storage::<Gold>().get(buyer).unwrap().0,old_balance);
        assert_eq!(serde_json::to_value(w.read_storage::<Inventory>().get(buyer).unwrap()).unwrap(),old_items);
        result=driver.step(&mut w,inputs).unwrap();
        assert_eq!(w.read_storage::<Gold>().get(buyer).unwrap().0,balance,"formal dispatcher settles cost");
        assert!(w.read_storage::<Inventory>().get(buyer).unwrap().find_item(id).is_some());
        assert_eq!(w.read_storage::<Gold>().get(enemy).unwrap().0,enemy_gold);
        assert_eq!(serde_json::to_value(w.read_storage::<Inventory>().get(enemy).unwrap()).unwrap(),enemy_items);
    }
    assert_eq!(w.read_storage::<Inventory>().get(buyer).unwrap().items().count(),2);
    w.write_storage::<Gold>().get_mut(buyer).unwrap().0=9999;
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(!role_bot_inputs(&w,&bots).unwrap().iter().any(|(_,i)|matches!(i.action,Some(PlayerInputEnum::ItemBuy(_)))),"completed build must not repurchase consumed swords");
    w.write_storage::<Inventory>().insert(buyer,Inventory::new()).unwrap();
    w.write_storage::<Pos>().get_mut(buyer).unwrap().0=home+omoba_sim::Vec2::new(Fixed64::from_i32(301),Fixed64::ZERO);
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(!role_bot_inputs(&w,&bots).unwrap().iter().any(|(_,i)|matches!(i.action,Some(PlayerInputEnum::ItemBuy(_)))),"outside shop never buys");
    w.write_storage::<Pos>().get_mut(buyer).unwrap().0=home;
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    w.write_resource::<GamePause>().is_paused=true;
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty());
    w.write_resource::<GamePause>().is_paused=false;
    w.write_storage::<CProperty>().get_mut(buyer).unwrap().hp=Fixed64::ZERO;
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"dead owner never shops");
}

#[test]
fn role_bot_active_item_priority_60hz_defends_but_defers_mana_during_windup() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::runtime::ability_runtime::ManaPool;
    for with_shield in [false, true] {
        let (mut w, mut driver) = world(SingleLaneConfig {mana_enabled: true,
            wave_interval: Fixed64::from_i32(10_000), ..three_lane_config()
        }, SimulationTickProfile::Production60Hz);
        driver.step(&mut w, []).unwrap();
        let [owner, enemy] = hero_pair(&w);
        w.write_storage::<Gold>().get_mut(owner).unwrap().0 = 2000;
        let mut ids = vec!["moba_mana_charm"];
        if with_shield {ids.push("moba_guard_charm");}
        let mut result = None;
        for id in &ids {
            result = Some(driver.step(&mut w, [(1, PlayerInput {action: Some(
                PlayerInputEnum::ItemBuy(ItemBuy {item_id: (*id).into()})
            )})]).unwrap());
        }
        assert_eq!(w.read_storage::<Inventory>().get(owner).unwrap().items().count(), ids.len());
        let own = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(2500));
        w.write_storage::<Pos>().get_mut(owner).unwrap().0 = own;
        w.write_storage::<Pos>().get_mut(enemy).unwrap().0 = own
            + omoba_sim::Vec2::new(Fixed64::from_i32(300), Fixed64::ZERO);
        w.write_storage::<CProperty>().get_mut(owner).unwrap().hp = Fixed64::from_i32(100);
        let capacity = w.read_storage::<Hero>().get(owner).unwrap().moba_mana_capacity().unwrap();
        w.write_storage::<Hero>().get_mut(owner).unwrap().mana_pool = Some(
            ManaPool::new(Fixed64::from_i32(10), capacity).unwrap());
        {
            let mut attacks = w.write_storage::<TAttack>();
            let attack = attacks.get_mut(owner).unwrap();
            attack.range.v = Fixed64::from_i32(550);
            attack.attack_phase = AttackSequencePhase::Windup;
            attack.asd_count = Fixed64::from_raw(-100);
        }
        let command = HeroCommand::AttackTarget {target: enemy, chase_origin: None};
        w.write_storage::<HeroCommandQueue>().insert(owner, HeroCommandQueue {
            active: Some(command), queued: Vec::new(),
        }).unwrap();
        let bots = RoleBotConfig {assignments: vec![BotAssignment {
            player_id: 1, role: BotRole::Carry, lane: 2, escort_player_id: None,
        }], think_interval_ticks: 1, ability_policies: Vec::new(), ability_learning: Vec::new(),
            sustain: None, item_builds: vec![BotItemBuild {role: BotRole::Carry,
                items: ids.iter().map(|id| (*id).into()).collect(), return_to_shop: None,
                active_use: Some(BotActiveItemPolicy {combat_radius: 700,
                    defend_below_hp_per_mille: 500, restore_below_mana_per_mille: 300})}]};
        run_committed_visibility_wave_b(&mut w, result.unwrap().tick, 0);
        let mut inputs = role_bot_inputs(&w, &bots).unwrap();
        assert_eq!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active, Some(command));
        if with_shield {
            assert_eq!(inputs.len(), 1);
            assert!(matches!(&inputs[0].1.action, Some(PlayerInputEnum::ItemUse(i)) if i.item_slot == 1));
        } else {
            assert!(inputs.is_empty(), "routine mana recovery must not cancel the existing windup");
            w.write_storage::<TAttack>().get_mut(owner).unwrap().attack_phase = AttackSequencePhase::Backswing;
            inputs = role_bot_inputs(&w, &bots).unwrap();
            assert_eq!(inputs.len(), 1);
            assert!(matches!(&inputs[0].1.action, Some(PlayerInputEnum::ItemUse(i)) if i.item_slot == 0));
        }
        driver.step(&mut w, inputs).unwrap();
        let inventories = w.read_storage::<Inventory>();
        let inventory = inventories.get(owner).unwrap();
        if with_shield {
            assert!(w.read_resource::<BuffStore>().shield_remaining(owner) > Fixed64::ZERO);
            assert!(inventory.slots[1].as_ref().unwrap().cooldown_remaining > 0.0);
            assert_eq!(inventory.slots[0].as_ref().unwrap().cooldown_remaining, 0.0);
        } else {
            assert!(w.read_storage::<Hero>().get(owner).unwrap().mana_pool.as_ref().unwrap().current()
                > Fixed64::from_i32(10));
            assert!(inventory.slots[0].as_ref().unwrap().cooldown_remaining > 0.0);
        }
    }
}

#[test]
fn role_bot_active_items_generated_shop_and_formal_use_at_60hz() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::runtime::ability_runtime::ManaPool;
    for id in ["moba_guard_charm","moba_stride_charm","moba_mana_charm","moba_ward_charm","moba_strike_charm"] {
        let (mut w,mut driver)=world(SingleLaneConfig {mana_enabled:true,
            wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();
        let [hero,enemy]=hero_pair(&w);
        let own=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
        w.write_storage::<Gold>().get_mut(hero).unwrap().0=2000;
        w.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::from_i32(100);
        w.write_storage::<TAttack>().get_mut(hero).unwrap().range.v=Fixed64::from_i32(500);
        let cap=w.read_storage::<Hero>().get(hero).unwrap().moba_mana_capacity().unwrap();
        w.write_storage::<Hero>().get_mut(hero).unwrap().mana_pool=Some(ManaPool::new(Fixed64::from_i32(10),cap).unwrap());
        let result=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::ItemBuy(ItemBuy {item_id:id.into()}))})]).unwrap();
        assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().items().count(),1,"formal base purchase {id}");
        w.write_storage::<Pos>().get_mut(hero).unwrap().0=own;
        w.write_storage::<Pos>().get_mut(enemy).unwrap().0=own+omoba_sim::Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO);
        let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Carry,lane:2,escort_player_id:None}],
            think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,
            item_builds:vec![BotItemBuild {role:BotRole::Carry,items:vec![id.into()],return_to_shop:None,
                active_use:Some(BotActiveItemPolicy {combat_radius:700,defend_below_hp_per_mille:500,restore_below_mana_per_mille:300})}]};
        run_committed_visibility_wave_b(&mut w,result.tick,0);
        let before=serde_json::to_value(w.read_storage::<Inventory>().get(hero).unwrap()).unwrap();
        let inputs=role_bot_inputs(&w,&bots).unwrap();
        assert_eq!(inputs.len(),1,"{id}");
        assert_eq!(inputs[0].0,1);
        assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::ItemUse(value)) if value.item_slot==0 && value.target_pos.is_none() && value.target_entity.is_none()),"{id}");
        assert_eq!(serde_json::to_value(w.read_storage::<Inventory>().get(hero).unwrap()).unwrap(),before);
        let result=driver.step(&mut w,inputs).unwrap();
        assert!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining>0.0);
        match id {
            "moba_guard_charm"=>assert_eq!(w.read_resource::<BuffStore>().shield_remaining(hero),Fixed64::from_i32(100)),
            "moba_stride_charm"=>assert!(w.read_resource::<BuffStore>().sum_add(hero,omb_script_abi::stat_keys::StatKey::MoveSpeedBonusBuff)>Fixed64::ZERO),
            "moba_mana_charm"=>assert!(w.read_storage::<Hero>().get(hero).unwrap().mana_pool.as_ref().unwrap().current()>=Fixed64::from_i32(110)),
            "moba_ward_charm"=>assert!(w.read_resource::<BuffStore>().sum_add(hero,omb_script_abi::stat_keys::StatKey::DamageTakenBonus)<Fixed64::ZERO),
            _=>assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(hero),Fixed64::from_i32(60)),
        }
        assert_eq!(w.read_resource::<BuffStore>().shield_remaining(enemy),Fixed64::ZERO);
        assert_eq!(w.read_storage::<Inventory>().get(enemy).unwrap().items().count(),0);
        run_committed_visibility_wave_b(&mut w,result.tick,0);
        assert!(!role_bot_inputs(&w,&bots).unwrap().iter().any(|(_,input)|matches!(input.action,Some(PlayerInputEnum::ItemUse(_)))));
    }
}

#[test]
fn role_bot_items_economic_recall_channel_then_shop_at_60hz() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..three_lane_config()},SimulationTickProfile::Production60Hz);
    let mut result=driver.step(&mut w,[]).unwrap();
    let [buyer,_]=hero_pair(&w);
    let base=w.read_resource::<MobaMatch>().bases[0].unwrap();
    let home=w.read_storage::<Pos>().get(base).unwrap().0;
    w.write_storage::<Pos>().get_mut(buyer).unwrap().0=home-
        omoba_sim::Vec2::new(Fixed64::from_i32(2000),Fixed64::ZERO);
    w.write_storage::<Gold>().get_mut(buyer).unwrap().0=950;
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Carry,lane:2,escort_player_id:None}],
        think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,
        item_builds:vec![BotItemBuild {role:BotRole::Carry,items:vec!["moba_greatsword".into()],active_use:None,
            return_to_shop:Some(BotShopReturnPolicy {min_gold:950,threat_radius:1000})}]};
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(inputs.len(),1);
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::Recall(_))));
    result=driver.step(&mut w,inputs).unwrap();
    assert!(w.read_resource::<MobaMatch>().is_recalling(buyer));
    for _ in 0..600 {
        run_committed_visibility_wave_b(&mut w,result.tick,0);
        assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"economy must not interrupt channel");
        result=driver.step(&mut w,[]).unwrap();
        if !w.read_resource::<MobaMatch>().is_recalling(buyer) {break;}
    }
    assert!(!w.read_resource::<MobaMatch>().is_recalling(buyer));
    assert_eq!(w.read_storage::<Pos>().get(buyer).unwrap().0,home);
    for id in ["moba_sword","moba_sword","moba_greatsword"] {
        run_committed_visibility_wave_b(&mut w,result.tick,0);
        let inputs=role_bot_inputs(&w,&bots).unwrap();
        assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::ItemBuy(buy)) if buy.item_id==id));
        result=driver.step(&mut w,inputs).unwrap();
    }
    assert_eq!(w.read_storage::<Gold>().get(buyer).unwrap().0,0);
    assert_eq!(w.read_storage::<Inventory>().get(buyer).unwrap().items().count(),1);
    w.write_storage::<Gold>().get_mut(buyer).unwrap().0=9999;
    w.write_storage::<Pos>().get_mut(buyer).unwrap().0=home-
        omoba_sim::Vec2::new(Fixed64::from_i32(2000),Fixed64::ZERO);
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(!role_bot_inputs(&w,&bots).unwrap().iter().any(|(_,i)|matches!(i.action,Some(PlayerInputEnum::Recall(_)))),
        "finished goals do not cause economic recall loops");
}

#[test]
fn base_recovery_is_opt_in_and_excludes_warmup_and_outside_radius() {
    for (enabled,warmup,outside) in [(false,Fixed64::ZERO,false),
        (true,Fixed64::from_i32(10),false),(true,Fixed64::ZERO,true)] {
        let (mut w,mut driver)=world(SingleLaneConfig {base_recovery_enabled:enabled,warmup,
            wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
        let [caster,_]=hero_pair(&w);
        w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::from_i32(100);
        if outside {w.write_storage::<Pos>().get_mut(caster).unwrap().0=
            omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));}
        driver.step(&mut w,[]).unwrap();
        assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(100));
    }
}

#[test]
fn base_recovery_60hz_filtered_settlement_matches_authority_without_repairs() {
    use std::collections::BTreeSet;
    let config=SingleLaneConfig {base_recovery_enabled:true,wave_interval:Fixed64::from_i32(10_000),..fast_config()};
    let seed=config.seed;
    let (mut authority,mut driver)=world(config,SimulationTickProfile::Production60Hz);
    let [hero,_]=hero_pair(&authority);
    authority.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::from_i32(100);
    let first=driver.step(&mut authority,[]).unwrap();project_tick(&mut authority,first);
    let allow=TeamProjectorConfig::default().component_allowlist;
    let starts=authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick()+1,60,seed);
    let mut replicas:Vec<_>=starts.into_iter().map(|(team,start)| {
        let replica=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),BTreeSet::new()).unwrap();
        let mut stepper=SpecsDisclosedWorldStepper::from_start(&start,allow.clone(),BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world,&stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();(team,replica,stepper)
    }).collect();
    let initial=authority.read_storage::<CProperty>().get(hero).unwrap().hp;
    for _ in 0..20 {
        let result=driver.step(&mut authority,[]).unwrap();project_tick(&mut authority,result);
        let frames=authority.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected=authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick()+1,60,seed);
        for (team,replica,stepper) in &mut replicas {
            let frame=frames[team].frame.clone();
            assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            assert!(matches!(replica.apply_frame(frame,stepper).unwrap(),FrameApplyResult::Applied {..}));
            let expected=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team],allow.clone(),BTreeSet::new()).unwrap();
            assert_eq!(replica.canonical_team_hash(),expected.canonical_team_hash(),"team {team} tick {}",driver.tick());
        }
    }
    assert!(authority.read_storage::<CProperty>().get(hero).unwrap().hp>initial);
}

#[test]
fn role_bot_learning_rank_zero_through_formal_60hz_upgrade_and_cast() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {heroes:["training_apprentice".into(),"training_apprentice".into()],
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();
    let [caster,target]=hero_pair(&w);
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=source;
    w.write_storage::<Pos>().get_mut(target).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(590),Fixed64::ZERO);
    w.write_storage::<Hero>().get_mut(caster).unwrap().abilities.swap(0,3);
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Mid,lane:1,escort_player_id:None}],
        think_interval_ticks:1,sustain:None,item_builds:Vec::new(),
        ability_learning:vec![BotAbilityLearningStep {ability:"apprentice_lance".into(),rank:1},
            BotAbilityLearningStep {ability:"apprentice_bolt".into(),rank:1}],
        ability_policies:vec![BotAbilityPolicy {ability:"apprentice_bolt".into(),intent:BotAbilityIntent::EnemyUnit}]};
    {
        let heroes=w.read_storage::<Hero>();let hero=heroes.get(caster).unwrap();
        assert_eq!((hero.level,hero.skill_points,hero.get_ability_level("apprentice_bolt")),(1,1,0));
    }
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(inputs.len(),1);
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::UpgradeAbility(upgrade)) if upgrade.ability_index==3));
    // Planning itself spends nothing. A gated first goal must not deadlock the
    // later legal goal; authority handles SkillLearn and actual point spending.
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().skill_points,1);
    let result=driver.step(&mut w,inputs).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    {
        let heroes=w.read_storage::<Hero>();let hero=heroes.get(caster).unwrap();
        assert_eq!((hero.get_ability_level("apprentice_bolt"),hero.get_ability_level("apprentice_lance"),hero.skill_points),(1,0,0));
    }
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::CastAbility(cast)) if cast.ability_index==3 && cast.target_entity==Some(target.id())));
    let hp=w.read_storage::<CProperty>().get(target).unwrap().hp;
    let result=driver.step(&mut w,inputs).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert_eq!(hp-w.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(80));
    // Explicit level/point fixture tests a newly eligible goal, not XP rewards.
    {let mut heroes=w.write_storage::<Hero>();let hero=heroes.get_mut(caster).unwrap();hero.level=6;hero.skill_points=1;}
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::UpgradeAbility(upgrade)) if upgrade.ability_index==2));
    driver.step(&mut w,inputs).unwrap();
    let heroes=w.read_storage::<Hero>();let hero=heroes.get(caster).unwrap();
    assert_eq!((hero.get_ability_level("apprentice_lance"),hero.skill_points),(1,0));
}

#[test]
fn role_bot_abilities_cast_damage_and_heal_through_formal_60hz_pipeline() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let config=SingleLaneConfig {creeps_per_wave:1,wave_interval:Fixed64::from_i32(10_000),..three_lane_config()};
    let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();
    let [caster,target]=hero_pair(&w);
    // Position fixture outside public tower/camp ranges, not component removal:
    // MOBA NPC combat owns private attack state independently of TAttack.
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=source;
    w.write_storage::<Pos>().get_mut(target).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(590),Fixed64::ZERO);
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    // Move the same learned ability to another slot; policy must resolve it
    // from this owner's actual loadout instead of a fixed Q/W/E/R number.
    w.write_storage::<Hero>().get_mut(caster).unwrap().abilities.swap(0,2);
    let player=w.read_resource::<MobaMatch>().heroes[0].player_id;
    let mut bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:player,role:BotRole::Mid,lane:1,escort_player_id:None}],
        think_interval_ticks:1,sustain:None,item_builds:Vec::new(),ability_learning:Vec::new(),ability_policies:vec![BotAbilityPolicy {ability:"lumen_bolt".into(),intent:BotAbilityIntent::EnemyUnit}]};
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(inputs.len(),1);
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::CastAbility(cast)) if cast.ability_index==2 && cast.target_entity==Some(target.id())));
    let hp=w.read_storage::<CProperty>().get(target).unwrap().hp;
    let result=driver.step(&mut w,inputs).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert_eq!(hp-w.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(80));
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("lumen_bolt"));
    assert!(role_bot_inputs(&w,&bots).unwrap().iter().all(|(_,input)|!matches!(input.action,Some(PlayerInputEnum::CastAbility(_)))));
    // Hide even a living, in-range opponent: no target can be reconstructed
    // from authority positions or cooldown/HP components.
    w.write_storage::<Hero>().get_mut(caster).unwrap().ability_cooldowns.clear();
    let canonical=((target.gen().id() as u32 as u64)<<32)|u64::from(target.id());
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.remove(&canonical);
    assert!(role_bot_inputs(&w,&bots).unwrap().iter().all(|(_,input)|!matches!(input.action,Some(PlayerInputEnum::CastAbility(_)))));
    bots.ability_policies=vec![BotAbilityPolicy {ability:"lumen_touch".into(),intent:BotAbilityIntent::SelfHeal {below_hp_per_mille:600}}];
    w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::from_i32(100);
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::CastAbility(cast)) if cast.ability_index==1 && cast.target_entity.is_none()));
    let result=driver.step(&mut w,inputs).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(170));
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("lumen_touch"));
}

#[test]
fn mana_budget_bot_uses_script_cost_and_formal_60hz_debit_without_reservation() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::runtime::ability_runtime::{ManaPool,BuffStore};
    let (mut w,mut driver)=world(SingleLaneConfig {
        mana_enabled:true, heroes:["training_ranger".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000), ..three_lane_config()
    },SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();
    let [caster,target]=hero_pair(&w);
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=source;
    w.write_storage::<Pos>().get_mut(target).unwrap().0=source
        + omoba_sim::Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO);
    {let mut props=w.write_storage::<CProperty>();let own=props.get_mut(caster).unwrap();
        own.mhp=Fixed64::from_i32(1000);own.hp=Fixed64::from_i32(100);}
    {let mut heroes=w.write_storage::<Hero>();let own=heroes.get_mut(caster).unwrap();
        // Existing archetypes share rank-one cost 45; the higher learned rank
        // supplies an actual authored cost 60 without changing game content.
        own.ability_levels.insert("ranger_volley".into(),4);
        own.mana_pool=Some(ManaPool::new(Fixed64::from_i32(45),own.moba_mana_capacity().unwrap()).unwrap());}
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Carry,lane:1,escort_player_id:None}],
        think_interval_ticks:1,sustain:None,item_builds:Vec::new(),ability_learning:Vec::new(),ability_policies:vec![
            BotAbilityPolicy {ability:"ranger_volley".into(),intent:BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:1}},
            BotAbilityPolicy {ability:"ranger_patch".into(),intent:BotAbilityIntent::SelfHeal {below_hp_per_mille:600}}]};
    let mut expected=w.read_storage::<Hero>().get(caster).unwrap().mana_pool.clone().unwrap();
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(inputs.len(),1);
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::CastAbility(cast)) if cast.ability_index==1 && cast.target_entity.is_none()),"budget decision: {:?}",inputs);
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    expected.spend(Fixed64::from_i32(45)).unwrap();
    let before=w.read_resource::<MobaMatch>().elapsed;
    let result=driver.step(&mut w,inputs).unwrap();
    // Patch now installs its authored regen buff through the generated handler.
    let rate=UnitStats::from_refs(&w.read_resource::<BuffStore>(),false).mana_regen(
        Fixed64::from_i32(omoba_template_ids::MOBA_MANA_REGEN_PER_SECOND as i32),caster);
    assert_eq!(rate,Fixed64::from_i32(7));
    expected.regenerate(rate,
        w.read_resource::<MobaMatch>().elapsed-before).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_patch"));
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(role_bot_inputs(&w,&bots).unwrap().iter().all(|(_,input)|!matches!(input.action,Some(PlayerInputEnum::CastAbility(_)))));
    // Explicit free-cost buff is read from this owner only; priority can now
    // choose the still-ready expensive skill with almost no mana.
    w.write_resource::<BuffStore>().add(caster,"fixture_free_mana",Fixed64::from_i32(10),
        serde_json::json!({"manacost_percentage":-1024}));
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::CastAbility(cast)) if cast.ability_index==2 && cast.target_pos.is_some()));
    let before=w.read_resource::<MobaMatch>().elapsed;
    driver.step(&mut w,inputs).unwrap();
    expected.regenerate(rate,
        w.read_resource::<MobaMatch>().elapsed-before).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_volley"));
}

#[test]
fn mana_recovery_bot_60hz_full_health_uses_discounted_formal_cast() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::runtime::ability_runtime::{ManaPool,BuffStore,checked_mana_cost};
    let (mut w,mut driver)=world(SingleLaneConfig {
        mana_enabled:true, heroes:["training_luminary".into(),"training_ranger".into()],
        wave_interval:Fixed64::from_i32(10_000), ..three_lane_config()
    },SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();
    let [caster,_]=hero_pair(&w);
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=
        omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    {let mut props=w.write_storage::<CProperty>();let own=props.get_mut(caster).unwrap();
        own.mhp=Fixed64::from_i32(1000);own.hp=own.mhp;}
    {let mut heroes=w.write_storage::<Hero>();let own=heroes.get_mut(caster).unwrap();
        own.mana_pool=Some(ManaPool::new(Fixed64::from_i32(60),own.moba_mana_capacity().unwrap()).unwrap());}
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Mid,lane:1,escort_player_id:None}],
        think_interval_ticks:1,sustain:None,item_builds:Vec::new(),ability_learning:Vec::new(),ability_policies:vec![
            BotAbilityPolicy {ability:"lumen_touch".into(),intent:BotAbilityIntent::SelfRecovery {
                below_hp_per_mille:600,below_mana_per_mille:500,restore_key:"mana_restore".into()}}]};
    assert!(role_bot_inputs(&w,&bots).unwrap().iter().all(|(_,input)|
        !matches!(input.action,Some(PlayerInputEnum::CastAbility(_)))),"55 cost / 20 restore loses mana");
    w.write_resource::<BuffStore>().add(caster,"fixture_discount_mana",Fixed64::from_i32(10),
        serde_json::json!({"manacost_percentage":-768}));
    let mut expected=w.read_storage::<Hero>().get(caster).unwrap().mana_pool.clone().unwrap();
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(inputs.len(),1);
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::CastAbility(cast))
        if cast.ability_index==1 && cast.target_entity.is_none() && cast.target_pos.is_none()));
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    expected.spend(checked_mana_cost(55.0,Fixed64::from_raw(256)).unwrap()).unwrap();
    expected.restore(Fixed64::from_i32(20)).unwrap();
    let before=w.read_resource::<MobaMatch>().elapsed;
    let result=driver.step(&mut w,inputs).unwrap();
    expected.regenerate(Fixed64::from_i32(5),w.read_resource::<MobaMatch>().elapsed-before).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(1000));
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("lumen_touch"));
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(role_bot_inputs(&w,&bots).unwrap().iter().all(|(_,input)|
        !matches!(input.action,Some(PlayerInputEnum::CastAbility(_)))));
}

#[test]
fn mana_regeneration_bot_60hz_generated_buff_blocks_recast_until_removed() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::runtime::ability_runtime::{ManaPool,BuffStore,checked_mana_cost};
    let (mut w,mut driver)=world(SingleLaneConfig {
        mana_enabled:true, heroes:["training_ranger".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000), ..three_lane_config()
    },SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();let [caster,_]=hero_pair(&w);
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=
        omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    {let mut props=w.write_storage::<CProperty>();let own=props.get_mut(caster).unwrap();
        own.mhp=Fixed64::from_i32(1000);own.hp=own.mhp;}
    {let mut heroes=w.write_storage::<Hero>();let own=heroes.get_mut(caster).unwrap();
        own.mana_pool=Some(ManaPool::new(Fixed64::from_i32(100),own.moba_mana_capacity().unwrap()).unwrap());}
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Carry,lane:1,escort_player_id:None}],
        think_interval_ticks:1,sustain:None,item_builds:Vec::new(),ability_learning:Vec::new(),ability_policies:vec![
            BotAbilityPolicy {ability:"ranger_patch".into(),intent:BotAbilityIntent::SelfManaRegeneration {
                below_hp_per_mille:600,below_mana_per_mille:500,
                rate_key:"mana_buff_value".into(),duration_key:"mana_buff_duration".into()}}]};
    let has_cast=|inputs:&[(u32,PlayerInput)]|inputs.iter().any(|(_,input)|matches!(input.action,Some(PlayerInputEnum::CastAbility(_))));
    assert!(!has_cast(&role_bot_inputs(&w,&bots).unwrap()),"12 marginal recovery cannot pay 45 cost");
    w.write_resource::<BuffStore>().add(caster,"fixture_discount_mana",Fixed64::from_i32(10),
        serde_json::json!({"manacost_percentage":-768}));
    let mut expected=w.read_storage::<Hero>().get(caster).unwrap().mana_pool.clone().unwrap();
    let inputs=role_bot_inputs(&w,&bots).unwrap();assert!(has_cast(&inputs));
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::CastAbility(cast))
        if cast.ability_index==1 && cast.target_entity.is_none()));
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    expected.spend(checked_mana_cost(45.0,Fixed64::from_raw(256)).unwrap()).unwrap();
    let before=w.read_resource::<MobaMatch>().elapsed;let result=driver.step(&mut w,inputs).unwrap();
    expected.regenerate(Fixed64::from_i32(7),w.read_resource::<MobaMatch>().elapsed-before).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(1000));
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_patch"));
    let key=omb_script_abi::buff_ids::generic_mana_buff_id("ranger_patch","mana_regen_constant",
        omb_script_abi::types::EntityHandle {id:caster.id(),gen:caster.gen().id() as u32});
    assert!(w.read_resource::<BuffStore>().get(caster,&key).unwrap().remaining>Fixed64::ZERO);
    // Clear only this cooldown to isolate the active-source guard from CD.
    w.write_storage::<Hero>().get_mut(caster).unwrap().start_cooldown("ranger_patch",Fixed64::ZERO);
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(!has_cast(&role_bot_inputs(&w,&bots).unwrap()));
    // Explicitly remove the fixture's source; the planner now considers a NEW
    // buff again. Natural expiry is covered by the authority recovery tests.
    w.write_resource::<BuffStore>().remove(caster,&key);
    assert!(has_cast(&role_bot_inputs(&w,&bots).unwrap()));
}

#[test]
fn support_guard_focus_60hz_shares_formal_attack_and_skill_without_hidden_escort_reads() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::runtime::ability_runtime::ManaPool;
    let player=|player_id,team_id,role,hero:&str|RoleBotPlayerPlan {player_id,team_id,role,
        hero:hero.into(),lane:"mid".into(),bot:player_id==1};
    let plan=RoleBotMatchPlan {schema_version:1,map_id:"three_lane_training".into(),think_hz:60,mana_enabled:true,
        players:vec![player(1,1,BotRole::Support,"training_support"),player(2,2,BotRole::Mid,"training_luminary"),
            player(3,1,BotRole::Carry,"training_ranger"),player(4,2,BotRole::Top,"training_vanguard")],
        ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),
        ability_policies:vec![BotAbilityPolicy {ability:"lumen_bolt".into(),intent:BotAbilityIntent::EnemyUnit}]};
    let (mut config,bots)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
    config.warmup=Fixed64::ZERO;config.wave_interval=Fixed64::from_i32(10_000);
    let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();
    let slots=w.read_resource::<MobaMatch>().heroes.clone();
    let entity=|id|slots.iter().find(|s|s.player_id==id).unwrap().entity.unwrap();
    let (support,close,carry,guard)=(entity(1),entity(2),entity(3),entity(4));
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    {let mut poses=w.write_storage::<Pos>();for (e,x) in [(support,0),(close,50),(carry,250),(guard,400)] {
        poses.get_mut(e).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO);
    }}
    {let mut props=w.write_storage::<CProperty>();for e in [support,close,carry,guard] {
        let p=props.get_mut(e).unwrap();p.hp=Fixed64::from_i32(10_000);p.mhp=p.hp;
        p.def_physic=Fixed64::ZERO;p.def_magic=Fixed64::ZERO;
    }}
    let capacity=w.read_storage::<Hero>().get(support).unwrap().mana_pool.as_ref().unwrap().maximum();
    w.write_storage::<Hero>().get_mut(support).unwrap().mana_pool=Some(
        ManaPool::new(Fixed64::from_i32(200),capacity).unwrap());
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let first=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&first[0].1.action,Some(PlayerInputEnum::CastAbility(cast))
        if cast.ability_index==0 && cast.target_entity==Some(guard.id())));
    let old=w.read_storage::<Pos>().get(carry).unwrap().0;
    w.write_storage::<Pos>().get_mut(carry).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(5000),Fixed64::ZERO);
    assert_eq!(role_bot_inputs(&w,&bots).unwrap(),first,"uncommitted carry pose cannot redirect protection");
    w.write_storage::<Pos>().get_mut(carry).unwrap().0=old;
    let canonical=((carry.gen().id() as u32 as u64)<<32)|u64::from(carry.id());
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.remove(&canonical);
    let fallback=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&fallback[0].1.action,Some(PlayerInputEnum::CastAbility(cast))
        if cast.target_entity==Some(close.id())),"cached hidden carry cannot supply guard focus");
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.insert(canonical);
    let before=w.read_resource::<MobaMatch>().elapsed;
    let mut expected=w.read_storage::<Hero>().get(support).unwrap().mana_pool.clone().unwrap();
    expected.spend(Fixed64::from_i32(45)).unwrap();
    let result=driver.step(&mut w,first).unwrap();
    expected.regenerate(Fixed64::from_i32(5),w.read_resource::<MobaMatch>().elapsed-before).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(support).unwrap().mana_pool.as_ref(),Some(&expected));
    assert_eq!(w.read_storage::<CProperty>().get(guard).unwrap().hp,Fixed64::from_i32(9920));
    assert!(w.read_storage::<Hero>().get(support).unwrap().is_on_cooldown("lumen_bolt"));
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let mut attacks=bots.clone();attacks.ability_policies.clear();
    let input=role_bot_inputs(&w,&attacks).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::AttackTarget(attack)) if attack.target_id==guard.id()));
}

#[test]
fn jungle_assist_60hz_uses_committed_allies_and_formal_skill_and_attack_inputs() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let player=|player_id,team_id,role,hero:&str|RoleBotPlayerPlan {player_id,team_id,role,
        hero:hero.into(),lane:"mid".into(),bot:player_id==1};
    let plan=RoleBotMatchPlan {schema_version:1,map_id:"three_lane_training".into(),think_hz:60,mana_enabled:false,
        players:vec![player(1,1,BotRole::Jungle,"training_vanguard"),player(2,2,BotRole::Mid,"training_luminary"),
            player(3,1,BotRole::Mid,"training_ranger")],ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),
        ability_policies:vec![BotAbilityPolicy {ability:"vanguard_strike".into(),intent:BotAbilityIntent::EnemyUnit}]};
    let (mut config,bots)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
    config.warmup=Fixed64::ZERO;config.wave_interval=Fixed64::from_i32(10_000);
    let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();let [caster,target]=hero_pair(&w);
    let ally=w.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==3).unwrap().entity.unwrap();
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    {let mut poses=w.write_storage::<Pos>();poses.get_mut(caster).unwrap().0=source;
        poses.get_mut(target).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(250),Fixed64::ZERO);
        poses.get_mut(ally).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);}
    {let mut props=w.write_storage::<CProperty>();for entity in [caster,target,ally] {
        let prop=props.get_mut(entity).unwrap();prop.hp=Fixed64::from_i32(10_000);prop.mhp=prop.hp;
        prop.def_physic=Fixed64::ZERO;prop.def_magic=Fixed64::ZERO;}}
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let canonical=((ally.gen().id() as u32 as u64)<<32)|u64::from(ally.id());
    let first=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&first[0].1.action,Some(PlayerInputEnum::CastAbility(cast))
        if cast.ability_index==0 && cast.target_entity==Some(target.id())));
    let old=w.read_storage::<Pos>().get(ally).unwrap().0;
    w.write_storage::<Pos>().get_mut(ally).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(5000),Fixed64::ZERO);
    assert_eq!(role_bot_inputs(&w,&bots).unwrap(),first,"uncommitted authority ally pose cannot change plan");
    w.write_storage::<Pos>().get_mut(ally).unwrap().0=old;
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.remove(&canonical);
    assert!(role_bot_inputs(&w,&bots).unwrap().iter().all(|(_,input)|!matches!(input.action,Some(PlayerInputEnum::CastAbility(_)))),
        "hidden ally cache must not authorize assistance");
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.insert(canonical);
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    let result=driver.step(&mut w,inputs).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(9935));
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("vanguard_strike"));
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let mut attack_only=bots.clone();attack_only.ability_policies.clear();
    let inputs=role_bot_inputs(&w,&attack_only).unwrap();
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::AttackTarget(attack)) if attack.target_id==target.id()));
}

#[test]
fn jungle_farm_focus_60hz_area_cast_and_attack_share_disclosed_neutral_target() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use specs::Builder;
    let (mut w,mut driver)=world(SingleLaneConfig {heroes:["training_ranger".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();let caster=hero_pair(&w)[0];
    let (near,far)={let state=w.read_resource::<MobaMatch>();
        (state.jungle_camps[0].entity.unwrap(),state.jungle_camps[1].entity.unwrap())};
    let origin=w.read_storage::<Pos>().get(near).unwrap().0-
        omoba_sim::Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO);
    let unit=w.read_storage::<Unit>().get(far).unwrap().clone();
    let property=w.read_storage::<CProperty>().get(far).unwrap().clone();
    let cluster=w.create_entity().with(unit).with(property)
        .with(Faction::new(FactionType::HostileNeutral,0))
        .with(ReplicationScope {kind:ReplicationScopeKind::Vision,owner_team:None})
        .with(Pos(origin+omoba_sim::Vec2::new(Fixed64::from_i32(610),Fixed64::ZERO))).build();
    {let mut poses=w.write_storage::<Pos>();poses.get_mut(caster).unwrap().0=origin;
        poses.get_mut(far).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(600),Fixed64::ZERO);}
    {let mut props=w.write_storage::<CProperty>();for e in [near,far,cluster] {
        let p=props.get_mut(e).unwrap();p.hp=Fixed64::from_i32(1000);p.mhp=p.hp;
        p.def_physic=Fixed64::ZERO;p.def_magic=Fixed64::ZERO;}}
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Jungle,lane:0,
        escort_player_id:None}],think_interval_ticks:1,ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),
        ability_policies:vec![BotAbilityPolicy {ability:"ranger_volley".into(),
            intent:BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:1}}]};
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::CastAbility(cast))
        if cast.ability_index==2 && cast.target_entity.is_none()
        && cast.target_pos==Some(Vec2I {x:(origin.x+Fixed64::from_i32(100)).raw() as i32,y:origin.y.raw() as i32})));
    let result=driver.step(&mut w,inputs).unwrap();
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_volley"));
    assert!(w.read_storage::<CProperty>().get(near).unwrap().hp<Fixed64::from_i32(1000));
    for e in [far,cluster] {assert_eq!(w.read_storage::<CProperty>().get(e).unwrap().hp,Fixed64::from_i32(1000));}
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let mut attack=bots.clone();attack.ability_policies.clear();
    assert!(role_bot_inputs(&w,&attack).unwrap().iter().any(|(_,i)|matches!(&i.action,
        Some(PlayerInputEnum::AttackTarget(value)) if value.target_id==near.id())));
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.remove(&canonical_entity_id(near));
    assert!(!role_bot_inputs(&w,&attack).unwrap().iter().any(|(_,i)|matches!(&i.action,
        Some(PlayerInputEnum::AttackTarget(value)) if value.target_id==near.id())),"hidden neutral cache cannot supply focus");
}

#[test]
fn ally_heal_60hz_generated_support_rejects_enemy_and_heals_disclosed_teammate() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::runtime::ability_runtime::ManaPool;
    let player=|player_id,team_id,role,hero:&str|RoleBotPlayerPlan {player_id,team_id,role,
        hero:hero.into(),lane:"mid".into(),bot:player_id==1};
    let plan=RoleBotMatchPlan {schema_version:1,map_id:"three_lane_training".into(),think_hz:60,mana_enabled:true,
        players:vec![player(1,1,BotRole::Support,"training_support"),player(2,2,BotRole::Mid,"training_luminary"),
            player(3,1,BotRole::Carry,"training_ranger")],ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),
        ability_policies:vec![BotAbilityPolicy {ability:"lumen_aid".into(),intent:BotAbilityIntent::AllyHeal {below_hp_per_mille:600}}]};
    let (mut config,bots)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
    config.warmup=Fixed64::ZERO;config.wave_interval=Fixed64::from_i32(10_000);
    let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
    let ally=w.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==3).unwrap().entity.unwrap();
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    {let mut poses=w.write_storage::<Pos>();poses.get_mut(caster).unwrap().0=source;
        poses.get_mut(enemy).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO);
        poses.get_mut(ally).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);}
    {let mut props=w.write_storage::<CProperty>();for entity in [caster,enemy,ally] {
        let prop=props.get_mut(entity).unwrap();prop.mhp=Fixed64::from_i32(1000);
        prop.hp=Fixed64::from_i32(if entity==caster {1000} else {100});}}
    {let mut heroes=w.write_storage::<Hero>();let own=heroes.get_mut(caster).unwrap();
        own.mana_pool=Some(ManaPool::new(Fixed64::from_i32(200),own.moba_mana_capacity().unwrap()).unwrap());}
    let mut expected=w.read_storage::<Hero>().get(caster).unwrap().mana_pool.clone().unwrap();
    let before=w.read_resource::<MobaMatch>().elapsed;
    let result=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:3,target_entity:Some(enemy.id()),target_pos:None}))})]).unwrap();
    expected.regenerate(Fixed64::from_i32(5),w.read_resource::<MobaMatch>().elapsed-before).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("lumen_aid"));
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,Fixed64::from_i32(100));
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::CastAbility(cast))
        if cast.ability_index==3 && cast.target_entity==Some(ally.id())));
    expected.spend(Fixed64::from_i32(90)).unwrap();let before=w.read_resource::<MobaMatch>().elapsed;
    driver.step(&mut w,inputs).unwrap();
    expected.regenerate(Fixed64::from_i32(5),w.read_resource::<MobaMatch>().elapsed-before).unwrap();
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&expected);
    assert_eq!(w.read_storage::<CProperty>().get(ally).unwrap().hp,Fixed64::from_i32(240));
    assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(1000));
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("lumen_aid"));
}

#[test]
fn support_escort_navigation_60hz_holds_on_blocked_disclosure_and_resumes() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let player=|player_id,team_id,role,bot|RoleBotPlayerPlan {player_id,team_id,role,bot,
        hero:"training_luminary".into(),lane:"bottom".into()};
    let plan=RoleBotMatchPlan {schema_version:1,map_id:"three_lane_training".into(),think_hz:60,mana_enabled:false,
        ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),
        players:vec![player(1,1,BotRole::Carry,false),player(2,2,BotRole::Mid,false),player(3,1,BotRole::Support,true)]};
    let (mut config,bots)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
    config.warmup=Fixed64::ZERO;config.wave_interval=Fixed64::from_i32(10_000);
    let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let slots=w.read_resource::<MobaMatch>().heroes.clone();
    let carry=slots.iter().find(|s|s.player_id==1).unwrap().entity.unwrap();
    let support=slots.iter().find(|s|s.player_id==3).unwrap().entity.unwrap();
    let destination=omoba_sim::Vec2::new(Fixed64::from_i32(600),Fixed64::from_i32(100));
    w.write_storage::<Pos>().get_mut(carry).unwrap().0=destination;
    let disclose=|w:&mut World,tick| {
        run_committed_visibility_wave_b(w,tick,0);
        w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current
            .retain(|id|*id==canonical_entity_id(carry) || *id==canonical_entity_id(support));
    };
    disclose(&mut w,driver.tick());
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(input.as_slice(),[(3,PlayerInput {action:Some(PlayerInputEnum::MoveTo(_))})]));
    driver.step(&mut w,input).unwrap();
    let original=(*w.read_resource::<BlockedRegions>()).clone();
    w.write_resource::<BlockedRegions>().0.push(BlockedRegion {name:"escort-destination-fixture".into(),points:vec![
        vek::Vec2::new(560.0,60.0),vek::Vec2::new(640.0,60.0),
        vek::Vec2::new(640.0,140.0),vek::Vec2::new(560.0,140.0)]});
    // A private, uncommitted move does not authorize a different escort target.
    w.write_storage::<Pos>().get_mut(carry).unwrap().0=omoba_sim::Vec2::new(Fixed64::from_i32(900),Fixed64::ZERO);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(input.as_slice(),[(3,PlayerInput {action:Some(PlayerInputEnum::HoldPosition(_))})]));
    w.write_storage::<Pos>().get_mut(carry).unwrap().0=destination;
    driver.step(&mut w,input).unwrap();
    let held=w.read_storage::<Pos>().get(support).unwrap().0;
    for _ in 0..3 {
        disclose(&mut w,driver.tick());
        assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"blocked escort hold is deduplicated");
        driver.step(&mut w,[]).unwrap();
        assert_eq!(w.read_storage::<Pos>().get(support).unwrap().0,held);
    }
    *w.write_resource::<BlockedRegions>()=original;
    disclose(&mut w,driver.tick());
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(input.as_slice(),[(3,PlayerInput {action:Some(PlayerInputEnum::MoveTo(m))})]
        if m.target.as_ref().is_some_and(|p|i64::from(p.x)==destination.x.raw() && i64::from(p.y)==destination.y.raw())));
    driver.step(&mut w,input).unwrap();driver.step(&mut w,[]).unwrap();
    assert_ne!(w.read_storage::<Pos>().get(support).unwrap().0,held,"normal escort movement resumes");
}

#[test]
fn role_bots_support_escorts_human_using_committed_position_at_60hz() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let player=|player_id,team_id,role,bot| RoleBotPlayerPlan {player_id,team_id,role,bot,
        hero:"training_luminary".into(),lane:"bottom".into()};
    let plan=RoleBotMatchPlan {schema_version:1,map_id:"three_lane_training".into(),think_hz:60,mana_enabled:false,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),
        players:vec![player(1,1,BotRole::Carry,false),player(2,2,BotRole::Mid,false),player(3,1,BotRole::Support,true)]};
    let (mut config,bots)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
    config.warmup=Fixed64::ZERO;
    let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();
    let slots=w.read_resource::<MobaMatch>().heroes.clone();
    let carry=slots.iter().find(|s|s.player_id==1).unwrap().entity.unwrap();
    let support=slots.iter().find(|s|s.player_id==3).unwrap().entity.unwrap();
    let destination=omoba_sim::Vec2::new(Fixed64::from_i32(600),Fixed64::from_i32(100));
    w.write_storage::<Pos>().get_mut(carry).unwrap().0=destination;
    let start=w.read_storage::<Pos>().get(support).unwrap().0;
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(inputs.len(),1); assert_eq!(inputs[0].0,3);
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::MoveTo(MoveTo {target:Some(pos),queued:false}))
        if pos.x==destination.x.raw() as i32 && pos.y==destination.y.raw() as i32));
    // A new authority position without Wave B commit cannot redirect the bot.
    w.write_storage::<Pos>().get_mut(carry).unwrap().0=omoba_sim::Vec2::new(Fixed64::from_i32(900),Fixed64::ZERO);
    assert_eq!(role_bot_inputs(&w,&bots).unwrap(),inputs);
    w.write_storage::<Pos>().get_mut(carry).unwrap().0=destination;
    for _ in 0..30 {
        let inputs=role_bot_inputs(&w,&bots).unwrap();
        assert!(inputs.iter().all(|(player,_)|*player==3));
        let result=driver.step(&mut w,inputs).unwrap();
        run_committed_visibility_wave_b(&mut w,result.tick,0);
    }
    assert!((w.read_storage::<Pos>().get(support).unwrap().0-destination).length_squared()
        < (start-destination).length_squared());
    // A stale pursuit must be cancelled via a normal replacement input, even
    // when the escort is already within the follow radius.
    let result=driver.step(&mut w,[(3,PlayerInput {action:Some(PlayerInputEnum::AttackMove(AttackMove {
        target:Some(Vec2I {x:900*1024,y:0}),queued:false,
    }))})]).unwrap();
    let here=w.read_storage::<Pos>().get(support).unwrap().0;
    w.write_storage::<Pos>().get_mut(carry).unwrap().0=here;
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let stop=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(stop.len(),1);
    assert!(matches!(&stop[0].1.action,Some(PlayerInputEnum::HoldPosition(h)) if !h.queued));
    let result=driver.step(&mut w,stop).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(support).unwrap().active,Some(HeroCommand::HoldPosition));
    let held_position=w.read_storage::<Pos>().get(support).unwrap().0;
    // A persistent hold must not be replaced by another one-tick own pose.
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty());
    let result=driver.step(&mut w,[]).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert_eq!(w.read_storage::<Pos>().get(support).unwrap().0,held_position);
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty());
    // Follow resumes only from the newly committed ally position, not from
    // an authority-only update or a private lookup.
    let next=held_position+omoba_sim::Vec2::new(Fixed64::from_i32(500),Fixed64::ZERO);
    w.write_storage::<Pos>().get_mut(carry).unwrap().0=next;
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty());
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    assert!(role_bot_inputs(&w,&bots).unwrap().iter().any(|(_,input)|
        matches!(&input.action,Some(PlayerInputEnum::MoveTo(MoveTo {target:Some(pos),queued:false}))
            if pos.x==next.x.raw() as i32 && pos.y==next.y.raw() as i32)));
}

#[test]
fn role_bot_plan_nine_bots_leave_human_control_untouched_at_60hz() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let mut players=Vec::new();
    for team in 1..=2 {
        for (index,(role,lane)) in [(BotRole::Top,"top"),(BotRole::Mid,"mid"),
            (BotRole::Carry,"bottom"),(BotRole::Support,"bottom"),(BotRole::Jungle,"mid")].into_iter().enumerate() {
            let player_id=(team-1)*5+index as u32+1;
            players.push(RoleBotPlayerPlan {player_id,team_id:team,hero:"training_luminary".into(),
                role,lane:lane.into(),bot:player_id != 1});
        }
    }
    let plan=RoleBotMatchPlan {schema_version:1,map_id:"three_lane_training".into(),think_hz:5,mana_enabled:false,players,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new()};
    let (mut config,bots)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
    config.warmup=Fixed64::ZERO;
    let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let start:Vec<_>=w.read_resource::<MobaMatch>().heroes.iter().map(|slot| {
        let entity=slot.entity.unwrap();
        (slot.player_id,entity,w.read_storage::<Pos>().get(entity).unwrap().0)
    }).collect();
    let mut issued=std::collections::BTreeSet::new();
    for _ in 0..60 {
        let inputs=role_bot_inputs(&w,&bots).unwrap();
        for (player,_) in &inputs { assert_ne!(*player,1); issued.insert(*player); }
        let result=driver.step(&mut w,inputs).unwrap();
        run_committed_visibility_wave_b(&mut w,result.tick,0);
    }
    assert_eq!(issued.len(),9);
    for (player,entity,pos) in start {
        let current=w.read_storage::<Pos>().get(entity).unwrap().0;
        if player==1 {assert_eq!(current,pos);} else {assert_ne!(current,pos);}
    }
}

#[test]
fn role_bots_five_positions_use_committed_vision_and_formal_60hz_inputs() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let mut config = three_lane_config();
    let roles = [BotRole::Top, BotRole::Mid, BotRole::Carry, BotRole::Support, BotRole::Jungle];
    let mut assignments = Vec::new();
    for side in 0..2 {
        for (index, role) in roles.into_iter().enumerate() {
            let player_id = if index == 0 { config.players[side] } else { 10 + side as u32 * 10 + index as u32 };
            if index != 0 { config.additional_players.push(SingleLanePlayerConfig {
                player_id, team_id:config.teams[side],hero:config.heroes[side].clone(),
            }); }
            assignments.push(BotAssignment { player_id,role,escort_player_id:None,lane:match role {
                BotRole::Top => 0, BotRole::Mid => 1, _ => 2,
            }});
        }
    }
    let bots = RoleBotConfig { assignments,think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new() };
    let (mut w,mut driver) = world(config,SimulationTickProfile::Production60Hz);
    bots.validate(&w.read_resource::<MobaMatch>()).unwrap();
    let mut bad = bots.clone(); bad.think_interval_ticks = 0;
    assert!(role_bot_inputs(&w,&bad).is_err());
    bad = bots.clone(); bad.assignments.push(bad.assignments[0]);
    assert!(role_bot_inputs(&w,&bad).is_err());
    bad = bots.clone(); bad.assignments[0].lane = 99;
    assert!(role_bot_inputs(&w,&bad).is_err());
    bad = bots.clone(); bad.assignments[0].player_id = 999;
    assert!(role_bot_inputs(&w,&bad).is_err());
    bad = bots.clone(); bad.assignments[1].role = BotRole::Top;
    assert!(role_bot_inputs(&w,&bad).is_err());
    bad = bots.clone(); bad.assignments[3].escort_player_id = Some(999);
    assert!(role_bot_inputs(&w,&bad).is_err());
    bad = bots.clone(); bad.assignments[3].escort_player_id = Some(2);
    assert!(role_bot_inputs(&w,&bad).is_err()); // opponent cannot be an escort
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty()); // no committed view
    let first = driver.step(&mut w,[]).unwrap();
    run_committed_visibility_wave_b(&mut w,first.tick,0);
    let start:Vec<_> = w.read_resource::<MobaMatch>().heroes.iter().map(|slot| {
        let entity = slot.entity.unwrap();
        (entity,w.read_storage::<Pos>().get(entity).unwrap().0)
    }).collect();
    let inputs = role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(inputs.len(),10);
    assert!(inputs.iter().all(|(_,input)| matches!(input.action,Some(PlayerInputEnum::AttackMove(_)))));
    // Mutating undisclosed authority positions cannot influence current plans.
    let enemy = start[1].0;
    let old = w.read_storage::<Pos>().get(enemy).unwrap().0;
    w.write_storage::<Pos>().get_mut(enemy).unwrap().0 = omoba_sim::Vec2::new(Fixed64::from_i32(500),Fixed64::from_i32(900));
    assert_eq!(inputs,role_bot_inputs(&w,&bots).unwrap());
    w.write_storage::<Pos>().get_mut(enemy).unwrap().0 = old;
    for _ in 0..60 {
        let inputs = role_bot_inputs(&w,&bots).unwrap();
        assert!(inputs.iter().all(|(player, input)| w.read_resource::<MobaMatch>().allows_player_input(*player,input)));
        let result = driver.step(&mut w,inputs).unwrap();
        run_committed_visibility_wave_b(&mut w,result.tick,0);
    }
    assert!(start.iter().all(|(entity,position)| w.read_storage::<Pos>().get(*entity).unwrap().0 != *position));
    w.write_resource::<TeamVisibilityRuntime>().latest_read_view = None;
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty());
}

#[test]
fn public_terrain_60hz_formal_move_dual_replica_replay() {
    terrain_formal_move_dual_replica_replay(false);
}

#[test]
fn npc_terrain_60hz_lane_and_camp_dual_replica_replay() {
    use std::collections::BTreeSet;
    use prost::Message;
    for seed in [1,42,0x20261004] {
        let config=SingleLaneConfig { seed,creeps_per_wave:1,wave_interval:Fixed64::from_i32(10_000),..three_lane_config() };
        let (mut a,mut ad)=world(config.clone(),SimulationTickProfile::Production60Hz);
        let (mut b,mut bd)=world(config,SimulationTickProfile::Production60Hz);
        // Explicit geometry/placement fixture, not a generated Lua map acceptance.
        let rect=|name:&str,x:f32,y:f32| BlockedRegion { name:name.into(),points:vec![
            vek::Vec2::new(x,y-30.0),vek::Vec2::new(x+1.0,y-30.0),
            vek::Vec2::new(x+1.0,y+30.0),vek::Vec2::new(x,y+30.0)] };
        let regions=BlockedRegions(vec![rect("lane-wall",400.0,0.0),rect("camp-wall",750.0,700.0)]);
        for w in [&mut a,&mut b] {
            *w.write_resource::<BlockedRegions>()=regions.clone();
            let hero=hero_pair(w)[0];
            w.write_storage::<Pos>().get_mut(hero).unwrap().0=omoba_sim::Vec2::new(Fixed64::from_i32(600),Fixed64::from_i32(700));
            let camp=w.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
            let mut hit=lethal(hero,camp);
            if let Outcome::Damage { real,.. }=&mut hit { *real=Fixed64::ONE; }
            w.write_resource::<Vec<Outcome>>().push(hit);
            process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();
        }
        let first=ad.step(&mut a,[]).unwrap(); bd.step(&mut b,[]).unwrap(); project_tick(&mut a,first);
        let creep=(&a.entities(),&a.read_storage::<Unit>(),&a.read_storage::<Pos>()).join()
            .find(|(_,u,p)| u.id=="single_lane_creep" && p.0.y==Fixed64::ZERO && p.0.x<Fixed64::from_i32(500)).unwrap().0;
        let camp=a.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
        let home=a.read_storage::<Pos>().get(camp).unwrap().0;
        let allow=TeamProjectorConfig::default().component_allowlist;
        let mut starts=a.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(ad.tick()+1,60,seed);
        for start in starts.values_mut() { start.public_metadata.push(omoba_core::game_proto::DeterministicMetadata {
            namespace:PUBLIC_BLOCKED_REGIONS_NAMESPACE.into(),key:PUBLIC_BLOCKED_REGIONS_KEY.into(),schema_version:1,
            value:encode_public_blocked_regions(&regions) }); }
        let mut replicas:Vec<_>=starts.into_iter().map(|(team,start)| {
            let replica=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),BTreeSet::new()).unwrap();
            let mut stepper=SpecsDisclosedWorldStepper::from_start(&start,allow.clone(),BTreeSet::new());
            stepper.script_registry.insert_manifest(crate::get_manifest());
            populate_ability_registry(&mut stepper.filtered.world,&stepper.script_registry);
            stepper.bootstrap_membership(replica.world()).unwrap();
            (team,replica,stepper)
        }).collect();
        let (mut lane_detour,mut lane_crossed,mut camp_detour,mut camp_crossed,mut returning)=(false,false,false,false,false);
        let polygons:Vec<Vec<_>>=regions.0.iter().map(|r|r.points.iter().map(|p|omoba_sim::Vec2::new(
            Fixed64::from_i32(p.x as i32),Fixed64::from_i32(p.y as i32))).collect()).collect();
        for index in 0..900 {
            let inputs=if index==180 { vec![(1,PlayerInput { action:Some(PlayerInputEnum::MoveTo(MoveTo {
                target:Some(Vec2I {x:120*1024,y:0}),queued:false })) })] } else {vec![]};
            let accepted=inputs.iter().map(|(player,input)|CanonicalAcceptedInput::from_authoritative_acceptance(
                1,*player,ad.tick()+1,1,canonical_entity_id(hero_pair(&a)[0]),None,input.encode_to_vec())).collect::<Vec<_>>();
            let result=ad.step(&mut a,inputs.clone()).unwrap(); bd.step(&mut b,inputs).unwrap();
            assert_eq!(single_lane_replay_digest(&a),single_lane_replay_digest(&b));
            let positions=a.read_storage::<Pos>();
            if let Some(p)=positions.get(creep) { lane_detour |= p.0.y!=Fixed64::ZERO; lane_crossed |= p.0.x>Fixed64::from_i32(450); }
            let pos=positions.get(camp).unwrap().0;
            camp_detour |= pos.y!=home.y;
            camp_crossed |= pos.x<Fixed64::from_i32(730);
            for entity in [creep,camp] { if let Some(p)=positions.get(entity) { for polygon in &polygons {
                assert!(!omoba_sim::terrain::swept_circle_hits_polygon(p.0,p.0,Fixed64::from_i32(20),polygon));
            } } }
            drop(positions);
            returning |= a.read_resource::<MobaMatch>().jungle_camps[0].returning;
            a.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
            project_tick(&mut a,result);
            let frames=a.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
            let expected=a.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(ad.tick()+1,60,seed);
            for (team,replica,stepper) in &mut replicas {
                let frame=frames[team].frame.clone();
                assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
                assert!(frame.step.as_ref().unwrap().accepted_inputs.iter().all(|input|input.player_id==*team));
                assert!(matches!(replica.apply_frame(frame,stepper).unwrap(),FrameApplyResult::Applied {..}));
                let fresh=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team],allow.clone(),BTreeSet::new()).unwrap();
                assert_eq!(replica.canonical_team_hash(),fresh.canonical_team_hash(),"NPC seed={seed} team={team} index={index}");
                assert!(stepper.filtered.world.try_fetch::<MobaMatch>().is_none());
            }
        }
        assert!(lane_detour && lane_crossed && camp_detour && camp_crossed && returning,
            "seed={seed} lane={lane_detour}/{lane_crossed} camp={camp_detour}/{camp_crossed} return={returning}");
        assert_eq!(a.read_storage::<Pos>().get(camp).unwrap().0,home);
        assert_eq!(a.read_storage::<CProperty>().get(camp).unwrap().hp,Fixed64::from_i32(450));
        assert!(!a.read_resource::<MobaMatch>().jungle_camps[0].returning);
        println!("NPC terrain seed={seed} ticks=900 dual_steps=1800 lane/camp detour and healed return passed");
    }
}

#[test]
fn lua_terrain_60hz_formal_move_dual_replica_replay() {
    terrain_formal_move_dual_replica_replay(true);
}

fn terrain_formal_move_dual_replica_replay(compiled: bool) {
    use std::collections::BTreeSet;
    use prost::Message;
    for seed in [1,42,0x20261004] {
        let config=SingleLaneConfig { seed,creeps_per_wave:1,wave_interval:Fixed64::from_i32(10_000),..three_lane_config() };
        let (mut a,mut ad)=world(config.clone(),SimulationTickProfile::Production60Hz);
        let (mut b,mut bd)=world(config,SimulationTickProfile::Production60Hz);
        // Public terrain fixture; does not inject gameplay position or movement.
        let start=a.read_storage::<Pos>().get(hero_pair(&a)[0]).unwrap().0;
        let offset=start.x.raw()/1024;
        let regions=if compiled { (*a.read_resource::<BlockedRegions>()).clone() } else { BlockedRegions(vec![BlockedRegion { name:"thin-wall".into(),
            points:vec![vek::Vec2::new((offset+50) as f32,-30.0),vek::Vec2::new((offset+51) as f32,-30.0),
                vek::Vec2::new((offset+51) as f32,30.0),vek::Vec2::new((offset+50) as f32,30.0)] }]) };
        *a.write_resource::<BlockedRegions>()=regions.clone();
        *b.write_resource::<BlockedRegions>()=regions.clone();
        let first=ad.step(&mut a,[]).unwrap(); bd.step(&mut b,[]).unwrap(); project_tick(&mut a,first);
        let allow=TeamProjectorConfig::default().component_allowlist;
        let mut starts=a.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(ad.tick()+1,60,seed);
        for start in starts.values_mut() {
            start.public_metadata.push(omoba_core::game_proto::DeterministicMetadata {
                namespace:PUBLIC_BLOCKED_REGIONS_NAMESPACE.into(),key:PUBLIC_BLOCKED_REGIONS_KEY.into(),
                schema_version:1,value:encode_public_blocked_regions(&regions),
            });
        }
        let mut replicas:Vec<_>=starts.into_iter().map(|(team,start)| {
            let replica=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),BTreeSet::new()).unwrap();
            let mut stepper=SpecsDisclosedWorldStepper::from_start(&start,allow.clone(),BTreeSet::new());
            stepper.script_registry.insert_manifest(crate::get_manifest());
            populate_ability_registry(&mut stepper.filtered.world,&stepper.script_registry);
            stepper.bootstrap_membership(replica.world()).unwrap();
            assert_eq!(stepper.filtered.world.read_resource::<BlockedRegions>().0.len(),regions.0.len());
            (team,replica,stepper)
        }).collect();
        let target=if compiled { omoba_sim::Vec2::new(Fixed64::from_i32(1350),Fixed64::from_i32(1225)) }
            else { start+omoba_sim::Vec2::new(Fixed64::from_i32(192),Fixed64::ZERO) };
        let approach=omoba_sim::Vec2::new(Fixed64::from_i32(1050),target.y);
        let mut detoured=false;
        for index in 0..if compiled {1200} else {600} {
            let destination=if compiled && index==0 {approach} else {target};
            let inputs=if index==0 || (compiled && index==600) { vec![(1,PlayerInput { action:Some(PlayerInputEnum::MoveTo(MoveTo {
                target:Some(Vec2I { x:destination.x.raw() as i32,y:destination.y.raw() as i32 }),queued:false })) })] } else { vec![] };
            let accepted=inputs.iter().map(|(player,input)| CanonicalAcceptedInput::from_authoritative_acceptance(
                1,*player,ad.tick()+1,1,canonical_entity_id(hero_pair(&a)[0]),None,input.encode_to_vec())).collect::<Vec<_>>();
            let result=ad.step(&mut a,inputs.clone()).unwrap(); bd.step(&mut b,inputs).unwrap();
            assert_eq!(single_lane_replay_digest(&a),single_lane_replay_digest(&b));
            let pos=a.read_storage::<Pos>().get(hero_pair(&a)[0]).unwrap().0;
            if !compiled || index>600 { detoured |= pos.y!=if compiled {target.y} else {start.y}; }
            if compiled && index==599 { assert!((pos-approach).length()<Fixed64::ONE,"did not approach Lua wall: {pos:?}"); }
            let radius=a.read_storage::<CollisionRadius>().get(hero_pair(&a)[0]).unwrap().0;
            for region in &regions.0 {
            let polygon:Vec<_>=region.points.iter().map(|p|omoba_sim::Vec2::new(
                Fixed64::from_i32(p.x as i32),Fixed64::from_i32(p.y as i32))).collect();
            assert!(!omoba_sim::terrain::swept_circle_hits_polygon(pos,pos,radius,&polygon));
            }
            a.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
            project_tick(&mut a,result);
            let frames=a.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
            let expected=a.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(ad.tick()+1,60,seed);
            for (team,replica,stepper) in &mut replicas {
                let frame=frames[team].frame.clone();
                assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
                assert!(frame.step.as_ref().unwrap().accepted_inputs.iter().all(|input|input.player_id==*team));
                assert!(matches!(replica.apply_frame(frame,stepper).unwrap(),FrameApplyResult::Applied {..}));
                let fresh=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team],allow.clone(),BTreeSet::new()).unwrap();
                assert_eq!(replica.canonical_team_hash(),fresh.canonical_team_hash(),"terrain seed={seed} team={team} index={index}");
                assert!(stepper.filtered.world.try_fetch::<MobaMatch>().is_none());
            }
        }
        let final_pos=a.read_storage::<Pos>().get(hero_pair(&a)[0]).unwrap().0;
        assert!(detoured && (final_pos-target).length()<Fixed64::ONE,"terrain route unfinished: {final_pos:?}");
    }
}

#[test]
fn jungle_60hz_aggro_leash_immunity_reward_and_respawn() {
    let run = || {
        let (mut w,mut driver) = world(SingleLaneConfig { wave_interval:Fixed64::from_i32(10_000),
            ..three_lane_config() },SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();
        let hero = hero_pair(&w)[0];
        let camp = w.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
        let home = w.read_storage::<Pos>().get(camp).unwrap().0;
        assert!(w.read_storage::<VisionSource>().get(camp).is_none());
        let f = w.read_storage::<Faction>().get(camp).unwrap().clone();
        assert!(f.is_hostile_to(w.read_storage::<Faction>().get(hero).unwrap()));
        assert!(!Faction::new(FactionType::Neutral,0).is_hostile_to(w.read_storage::<Faction>().get(hero).unwrap()));
        // Fixture placement only. Combat uses the actual damage/outcome pipeline.
        w.write_storage::<Pos>().get_mut(hero).unwrap().0 = home + omoba_sim::Vec2::new(Fixed64::from_i32(250),Fixed64::ZERO);
        let mut hit = lethal(hero,camp);
        if let Outcome::Damage { real,.. } = &mut hit { *real = Fixed64::from_i32(10); }
        w.write_resource::<Vec<Outcome>>().push(hit);
        process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
        for _ in 0..20 { driver.step(&mut w,[]).unwrap(); }
        assert_ne!(w.read_storage::<Pos>().get(camp).unwrap().0,home);
        w.write_storage::<Pos>().get_mut(hero).unwrap().0 = omoba_sim::Vec2::ZERO;
        driver.step(&mut w,[]).unwrap();
        assert!(w.read_resource::<MobaMatch>().jungle_camps[0].returning);
        assert!(!moba_damage_allowed(&w,camp));
        let hp = w.read_storage::<CProperty>().get(camp).unwrap().hp;
        w.write_resource::<Vec<Outcome>>().extend([lethal(hero,camp),Outcome::ScriptDirectDamage { target:camp,amount:Fixed64::from_i32(10_000) }]);
        process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
        assert_eq!(w.read_storage::<CProperty>().get(camp).unwrap().hp,hp);
        for _ in 0..30 { driver.step(&mut w,[]).unwrap(); }
        assert_eq!(w.read_storage::<Pos>().get(camp).unwrap().0,home);
        assert_eq!(w.read_storage::<CProperty>().get(camp).unwrap().hp,Fixed64::from_i32(450));
        let before = w.read_storage::<Gold>().get(hero).unwrap().0;
        let xp = w.read_storage::<Hero>().get(hero).unwrap().experience;
        w.write_resource::<Vec<Outcome>>().extend([lethal(hero,camp),lethal(hero,camp)]);
        process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap(); w.maintain();
        assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0,before+60);
        assert_eq!(w.read_storage::<Hero>().get(hero).unwrap().experience,xp+90);
        assert_eq!(w.read_resource::<MobaMatch>().heroes[0].kills,0);
        assert!(w.read_resource::<MobaMatch>().jungle_camps[0].entity.is_none());
        let elapsed = w.read_resource::<MobaMatch>().elapsed;
        w.write_resource::<GamePause>().is_paused = true;
        for _ in 0..60 { driver.step(&mut w,[]).unwrap(); }
        assert_eq!(w.read_resource::<MobaMatch>().elapsed,elapsed);
        w.write_resource::<GamePause>().is_paused = false;
        for _ in 0..960 { driver.step(&mut w,[]).unwrap(); }
        let next = w.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
        assert_ne!(next,camp); assert_eq!(w.read_resource::<MobaMatch>().jungle_camps[0].respawns,1);
        assert_eq!(w.read_storage::<CProperty>().get(next).unwrap().hp,Fixed64::from_i32(450));
        single_lane_replay_digest(&w)
    };
    assert_eq!(run(),run());
}

#[test]
fn role_bot_jungle_patrol_60hz_keeps_destination_until_disclosed_arrival() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..three_lane_config()},SimulationTickProfile::Production60Hz);
    let first=driver.step(&mut w,[]).unwrap();
    let hero=hero_pair(&w)[0];
    let points:Vec<_>=w.read_resource::<MobaMatch>().jungle_camps.iter().map(|camp| {
        let (x,y)=camp.definition.position;
        omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))
    }).collect();
    assert!(points.len()>1);
    let start=omoba_sim::Vec2::new(Fixed64::from_i32(-10_000),Fixed64::from_i32(-10_000));
    let index=points.iter().enumerate().min_by_key(|(i,p)|((**p-start).length_squared().raw(),*i)).unwrap().0;
    w.write_storage::<Pos>().get_mut(hero).unwrap().0=start;
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Jungle,lane:0,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:None,item_builds:Vec::new()};
    let disclose_owner=|w:&mut World,tick| {
        run_committed_visibility_wave_b(w,tick,0);
        w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current
            .retain(|id|*id==canonical_entity_id(hero));
    };
    disclose_owner(&mut w,first.tick);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(input.as_slice(),[(1,PlayerInput {action:Some(PlayerInputEnum::AttackMove(value))})]
        if value.target==Some(Vec2I {x:points[index].x.raw() as i32,y:points[index].y.raw() as i32})));
    driver.step(&mut w,input).unwrap();
    for _ in 0..105 {
        let tick=driver.tick();
        disclose_owner(&mut w,tick);
        assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"no clock-based diversion or repeated travel input");
        driver.step(&mut w,[]).unwrap();
    }
    assert_ne!(w.read_storage::<Pos>().get(hero).unwrap().0,start,"formal input must really move");
    assert!(matches!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active,
        Some(HeroCommand::AttackMove {pos}) if pos==points[index]));
    // Fixture arrival. The planner must use the next committed owner pose,
    // not this uncommitted ECS mutation or any camp alive/respawn state.
    w.write_storage::<Pos>().get_mut(hero).unwrap().0=points[index];
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty());
    let arrival=driver.step(&mut w,[]).unwrap();
    disclose_owner(&mut w,arrival.tick);
    let next=points[(index+1)%points.len()];
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(input.as_slice(),[(1,PlayerInput {action:Some(PlayerInputEnum::AttackMove(value))})]
        if value.target==Some(Vec2I {x:next.x.raw() as i32,y:next.y.raw() as i32})));
    driver.step(&mut w,input).unwrap();
    assert!(matches!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active,
        Some(HeroCommand::AttackMove {pos}) if pos==next));
}

#[test]
fn combat_navigation_60hz_lane_roles_skip_failed_chases_hold_and_reconsider() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let player=|player_id,team_id,role|RoleBotPlayerPlan {player_id,team_id,role,bot:false,
        hero:"training_vanguard".into(),lane:"top".into()};
    for role in [BotRole::Top,BotRole::Mid,BotRole::Carry,BotRole::Support] {
        let plan=RoleBotMatchPlan {schema_version:1,map_id:"three_lane_training".into(),think_hz:60,mana_enabled:false,
            ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),
            players:vec![player(1,1,BotRole::Top),player(2,2,BotRole::Mid),player(3,2,BotRole::Support)]};
        let (mut config,_)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
        config.warmup=Fixed64::ZERO;config.wave_interval=Fixed64::from_i32(10_000);
        let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();
        let slots=w.read_resource::<MobaMatch>().heroes.clone();
        let entity=|id|slots.iter().find(|s|s.player_id==id).unwrap().entity.unwrap();
        let (bot,a,b)=(entity(1),entity(2),entity(3));
        let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
        w.write_storage::<Pos>().get_mut(bot).unwrap().0=origin;
        w.write_storage::<Pos>().get_mut(a).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO);
        w.write_storage::<Pos>().get_mut(b).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(450),Fixed64::ZERO);
        let disclose=|w:&mut World,tick| {
            run_committed_visibility_wave_b(w,tick,0);
            w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current
                .retain(|id|[bot,a,b].iter().any(|e|*id==canonical_entity_id(*e)));
        };
        let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role,lane:0,escort_player_id:None}],
            think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new()};
        let targets=|input:&[(u32,PlayerInput)],target:specs::Entity|matches!(input,
            [(1,PlayerInput {action:Some(PlayerInputEnum::AttackTarget(t))})] if t.target_id==target.id());
        disclose(&mut w,driver.tick());
        assert!(targets(&role_bot_inputs(&w,&bots).unwrap(),a),"{role:?}: original priority");
        let original=(*w.read_resource::<BlockedRegions>()).clone();
        let block=|x:f32|BlockedRegion {name:"combat-navigation-fixture".into(),points:vec![
            vek::Vec2::new(x-40.0,2460.0),vek::Vec2::new(x+40.0,2460.0),
            vek::Vec2::new(x+40.0,2540.0),vek::Vec2::new(x-40.0,2540.0)]};
        w.write_resource::<BlockedRegions>().0.push(block(300.0));
        let input=role_bot_inputs(&w,&bots).unwrap();
        assert!(targets(&input,b),"{role:?}: select another disclosed reachable target");
        driver.step(&mut w,input).unwrap();
        w.write_resource::<BlockedRegions>().0.push(block(450.0));
        disclose(&mut w,driver.tick());
        let input=role_bot_inputs(&w,&bots).unwrap();
        assert!(matches!(input.as_slice(),[(1,PlayerInput {action:Some(PlayerInputEnum::HoldPosition(_))})]));
        driver.step(&mut w,input).unwrap();
        let held=w.read_storage::<Pos>().get(bot).unwrap().0;
        for _ in 0..3 {
            disclose(&mut w,driver.tick());
            assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"{role:?}: hold deduplication");
            driver.step(&mut w,[]).unwrap();
            assert_eq!(w.read_storage::<Pos>().get(bot).unwrap().0,held);
        }
        *w.write_resource::<BlockedRegions>()=original;
        disclose(&mut w,driver.tick());
        let input=role_bot_inputs(&w,&bots).unwrap();
        assert!(targets(&input,a),"{role:?}: failed candidate is not permanently blacklisted");
        driver.step(&mut w,input).unwrap();driver.step(&mut w,[]).unwrap();
        assert_ne!(w.read_storage::<Pos>().get(bot).unwrap().0,held);
    }
}

#[test]
fn lane_navigation_60hz_all_lane_roles_skip_blocked_waypoints_hold_and_resume() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let route:Vec<_>=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap()
        .lanes[0].waypoints.iter().map(|&(x,y)|omoba_sim::Vec2::new(
            Fixed64::from_i32(x),Fixed64::from_i32(y))).collect();
    let block=|point:omoba_sim::Vec2| {
        let x=point.x.to_f32_for_render();let y=point.y.to_f32_for_render();
        BlockedRegion {name:"lane-navigation-fixture".into(),points:vec![
            vek::Vec2::new(x-40.0,y-40.0),vek::Vec2::new(x+40.0,y-40.0),
            vek::Vec2::new(x+40.0,y+40.0),vek::Vec2::new(x-40.0,y+40.0)]}
    };
    for role in [BotRole::Top,BotRole::Mid,BotRole::Carry,BotRole::Support] {
        let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
            ..three_lane_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let owner=hero_pair(&w)[0];
        let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
        w.write_storage::<Pos>().get_mut(owner).unwrap().0=origin;
        let disclose=|w:&mut World,tick| {
            run_committed_visibility_wave_b(w,tick,0);
            w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current
                .retain(|id|*id==canonical_entity_id(owner));
        };
        let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role,lane:0,escort_player_id:None}],
            think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new()};
        let targets=|inputs:&[(u32,PlayerInput)],point:omoba_sim::Vec2|matches!(inputs,
            [(1,PlayerInput {action:Some(PlayerInputEnum::AttackMove(value))})]
            if value.target==Some(Vec2I {x:point.x.raw() as i32,y:point.y.raw() as i32}));
        disclose(&mut w,driver.tick());
        assert!(targets(&role_bot_inputs(&w,&bots).unwrap(),route[1]),"{role:?}: ordinary bend first");
        let original=(*w.read_resource::<BlockedRegions>()).clone();
        w.write_resource::<BlockedRegions>().0.push(block(route[1]));
        let input=role_bot_inputs(&w,&bots).unwrap();
        assert!(targets(&input,route[2]),"{role:?}: next reachable forward point");
        driver.step(&mut w,input).unwrap();
        disclose(&mut w,driver.tick());
        assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"{role:?}: retain reachable admitted destination");
        for point in &route[2..] {w.write_resource::<BlockedRegions>().0.push(block(*point));}
        let input=role_bot_inputs(&w,&bots).unwrap();
        assert!(matches!(input.as_slice(),[(1,PlayerInput {action:Some(PlayerInputEnum::HoldPosition(_))})]),"{role:?}: no reachable forward point");
        driver.step(&mut w,input).unwrap();
        let held=w.read_storage::<Pos>().get(owner).unwrap().0;
        for _ in 0..3 {
            disclose(&mut w,driver.tick());
            assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"{role:?}: no repeated hold");
            driver.step(&mut w,[]).unwrap();
            assert_eq!(w.read_storage::<Pos>().get(owner).unwrap().0,held);
        }
        *w.write_resource::<BlockedRegions>()=original;
        disclose(&mut w,driver.tick());
        let input=role_bot_inputs(&w,&bots).unwrap();
        assert!(targets(&input,route[1]),"{role:?}: no permanent blacklist");
        driver.step(&mut w,input).unwrap();
        driver.step(&mut w,[]).unwrap();
        let moved=w.read_storage::<Pos>().get(owner).unwrap().0;
        assert_ne!(moved,held,"{role:?}: normal movement resumed");
        assert_ne!(moved,route[1],"{role:?}: no teleport");
    }
}

#[test]
fn role_bot_lane_route_60hz_preserves_bend_until_committed_arrival() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..three_lane_config()},SimulationTickProfile::Production60Hz);
    let first=driver.step(&mut w,[]).unwrap();
    let hero=hero_pair(&w)[0];
    let route:Vec<_>=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap()
        .lanes[0].waypoints.iter().map(|&(x,y)|omoba_sim::Vec2::new(
            Fixed64::from_i32(x),Fixed64::from_i32(y))).collect();
    assert!(route.len()>=3);
    let start=route[0]+(route[1]-route[0])*Fixed64::from_raw(614);
    w.write_storage::<Pos>().get_mut(hero).unwrap().0=start;
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Top,lane:0,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:None,item_builds:Vec::new()};
    let disclose_owner=|w:&mut World,tick| {
        run_committed_visibility_wave_b(w,tick,0);
        w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current
            .retain(|id|*id==canonical_entity_id(hero));
    };
    let targets=|inputs:&[(u32,PlayerInput)],point:omoba_sim::Vec2|matches!(inputs,
        [(1,PlayerInput {action:Some(PlayerInputEnum::AttackMove(value))})]
        if value.target==Some(Vec2I {x:point.x.raw() as i32,y:point.y.raw() as i32}));
    disclose_owner(&mut w,first.tick);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(targets(&input,route[1]),"must not skip the nearest bend to route[2]");
    driver.step(&mut w,input).unwrap();
    for _ in 0..40 {
        let tick=driver.tick();disclose_owner(&mut w,tick);
        assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"preserve admitted waypoint without resending");
        driver.step(&mut w,[]).unwrap();
    }
    assert_ne!(w.read_storage::<Pos>().get(hero).unwrap().0,start);
    w.write_storage::<Pos>().get_mut(hero).unwrap().0=route[1];
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"uncommitted pose cannot advance route");
    let arrived=driver.step(&mut w,[]).unwrap();disclose_owner(&mut w,arrived.tick);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(targets(&input,route[2]));
    driver.step(&mut w,input).unwrap();
    assert!(matches!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active,
        Some(HeroCommand::AttackMove {pos}) if pos==route[2]));
}

#[test]
fn jungle_60hz_formal_inputs_dual_replica_and_every_tick_replay() {
    use std::collections::BTreeSet;
    use prost::Message;
    for seed in [1,42,0x20261004] {
        let config = SingleLaneConfig { seed,wave_interval:Fixed64::from_i32(10_000),..three_lane_config() };
        let (mut a,mut ad) = world(config.clone(),SimulationTickProfile::Production60Hz);
        let (mut b,mut bd) = world(config,SimulationTickProfile::Production60Hz);
        for w in [&mut a,&mut b] {
            let hero = hero_pair(w)[0];
            w.write_storage::<Pos>().get_mut(hero).unwrap().0 = omoba_sim::Vec2::new(Fixed64::from_i32(600),Fixed64::from_i32(700));
        }
        let first = ad.step(&mut a,[]).unwrap(); bd.step(&mut b,[]).unwrap(); project_tick(&mut a,first);
        let allow = TeamProjectorConfig::default().component_allowlist;
        let starts = a.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(ad.tick()+1,60,seed);
        let mut replicas: Vec<_> = starts.into_iter().map(|(team,start)| {
            let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),BTreeSet::new()).unwrap();
            let mut stepper = SpecsDisclosedWorldStepper::from_start(&start,allow.clone(),BTreeSet::new());
            stepper.script_registry.insert_manifest(crate::get_manifest());
            populate_ability_registry(&mut stepper.filtered.world,&stepper.script_registry);
            stepper.bootstrap_membership(replica.world()).unwrap();
            (team,replica,stepper)
        }).collect();
        let original = a.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
        let mut died = false;
        let mut returned = false;
        let mut damage_facts = 0;
        let mut applied_steps = 0;
        let mut withdrew_after_kill = false;
        for index in 0..3000 {
            let camp = a.read_resource::<MobaMatch>().jungle_camps[0].entity;
            died |= camp != Some(original);
            returned |= a.read_resource::<MobaMatch>().jungle_camps[0].returning;
            let mut inputs = Vec::new();
            if died && !withdrew_after_kill {
                withdrew_after_kill = true;
                inputs.push((1,PlayerInput { action:Some(PlayerInputEnum::MoveTo(MoveTo {
                    target:Some(Vec2I {x:0,y:0}),queued:false })) }));
            } else if index == 90 || index == 300 {
                inputs.push((1,PlayerInput { action:Some(PlayerInputEnum::MoveTo(MoveTo {
                    target:Some(if index == 90 { Vec2I {x:0,y:0} } else { Vec2I {x:600*1024,y:700*1024} }),queued:false })) }));
            } else if !died && (index == 0 || index >= 300) && camp.is_some_and(|entity|
                a.read_resource::<TeamVisibilityRuntime>().teams[&1].index.current.contains(&canonical_entity_id(entity))
                && (a.read_storage::<Pos>().get(entity).unwrap().0-a.read_storage::<Pos>().get(hero_pair(&a)[0]).unwrap().0).length_squared()
                    <= Fixed64::from_i32(250)*Fixed64::from_i32(250))
                && !matches!(a.read_storage::<HeroCommandQueue>().get(hero_pair(&a)[0]).and_then(|q|q.active.as_ref()),
                    Some(HeroCommand::AttackTarget { target,.. }) if *target == original) {
                inputs.push((1,PlayerInput { action:Some(PlayerInputEnum::AttackTarget(omoba_core::game_proto::AttackTarget {
                    target_id:original.id(),queued:false })) }));
            }
            let accepted = inputs.iter().map(|(player,input)| {
                let actor = hero_pair(&a)[0]; let mut safe = input.clone();
                let (kind,target) = match safe.action.as_mut().unwrap() {
                    PlayerInputEnum::AttackTarget(attack) => { attack.target_id = 0; (3,Some(canonical_entity_id(original))) },
                    PlayerInputEnum::MoveTo(_) => (1,None), _ => unreachable!(),
                };
                CanonicalAcceptedInput::from_authoritative_acceptance(1,*player,ad.tick()+1,kind,
                    canonical_entity_id(actor),target,safe.encode_to_vec())
            }).collect::<Vec<_>>();
            let result = ad.step(&mut a,inputs.clone()).unwrap(); bd.step(&mut b,inputs).unwrap();
            assert_eq!(single_lane_replay_digest(&a),single_lane_replay_digest(&b),"seed={seed} index={index}");
            a.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
            project_tick(&mut a,result);
            let frames = a.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
            let expected = a.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(ad.tick()+1,60,seed);
            for (team,replica,stepper) in &mut replicas {
                let frame = frames[team].frame.clone();
                assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
                damage_facts += frame.step.as_ref().unwrap().external_effects.len();
                assert!(frame.step.as_ref().unwrap().accepted_inputs.iter().all(|input|input.player_id == *team));
                assert!(matches!(replica.apply_frame(frame,stepper).unwrap(),FrameApplyResult::Applied {..}));
                let view = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team],allow.clone(),BTreeSet::new()).unwrap();
                assert_eq!(replica.canonical_team_hash(),view.canonical_team_hash(),"camp team={team} seed={seed} index={index}");
                assert!(stepper.filtered.world.try_fetch::<MobaMatch>().is_none());
                applied_steps += 1;
            }
        }
        let state = a.read_resource::<MobaMatch>();
        assert!(died && returned && damage_facts > 0,"seed={seed} died={died} returned={returned} damage={damage_facts} camp={:?} hp={:?} hero={:?}",state.jungle_camps[0],state.jungle_camps[0].entity.and_then(|e|a.read_storage::<CProperty>().get(e).cloned()),a.read_storage::<Hero>().get(hero_pair(&a)[0]));
        assert_eq!(state.jungle_camps[0].respawns,1);
        assert_eq!(state.heroes[0].kills,0);
        let hero = state.heroes[0].entity.unwrap();
        assert!(a.read_storage::<Gold>().get(hero).unwrap().0 >= 60);
        println!("jungle seed={seed} ticks=3000 dual_steps={applied_steps} external_damage={damage_facts} returned={returned} respawns=1");
    }
}

#[test]
fn jungle_rewards_require_lethal_hero_provenance_and_survive_death() {
    for mode in 0..3 {
        let (mut w,mut driver) = world(three_lane_config(),SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();
        let camp = w.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
        let other = w.read_resource::<MobaMatch>().jungle_camps[1].entity.unwrap();
        let outcome = match mode {
            0 => Outcome::ScriptDirectDamage { target:camp,amount:Fixed64::from_i32(10_000) },
            1 => lethal(other,camp),
            _ => Outcome::Death { pos:omoba_sim::Vec2::ZERO,ent:camp },
        };
        w.write_resource::<Vec<Outcome>>().push(outcome);
        for _ in 0..3 { driver.step(&mut w,[]).unwrap(); }
        assert!(w.read_resource::<MobaMatch>().jungle_camps[0].entity.is_none(),"mode={mode}");
        for hero in hero_pair(&w) { assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0,0); }
        assert!(w.read_resource::<MobaMatch>().heroes.iter().all(|s|s.kills==0));
    }
    let (mut w,mut driver) = world(three_lane_config(),SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let [hero,enemy] = hero_pair(&w);
    let camp = w.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
    w.write_resource::<Vec<Outcome>>().extend([lethal(hero,camp),lethal(enemy,hero)]);
    for _ in 0..3 { driver.step(&mut w,[]).unwrap(); }
    assert!(w.read_resource::<MobaMatch>().heroes[0].entity.is_none());
    for _ in 0..130 { driver.step(&mut w,[]).unwrap(); }
    let reborn = w.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    assert_eq!(w.read_storage::<Gold>().get(reborn).unwrap().0,60);
    assert_eq!(w.read_storage::<Hero>().get(reborn).unwrap().experience,90);
    assert_eq!(w.read_resource::<MobaMatch>().heroes[0].kills,0);
}

#[test]
fn three_lane_lua_map_spawns_each_route_and_replays_at_60hz() {
    for seed in [1,42,0x20261004] {
        let config = SingleLaneConfig { seed, creeps_per_wave: 1, ..three_lane_config() };
        let (mut a,mut ad) = world(config.clone(),SimulationTickProfile::Production60Hz);
        let (mut b,mut bd) = world(config,SimulationTickProfile::Production60Hz);
        for tick in 0..180 {
            ad.step(&mut a,[]).unwrap(); bd.step(&mut b,[]).unwrap();
            assert_eq!(single_lane_replay_digest(&a),single_lane_replay_digest(&b),"seed={seed} tick={tick}");
        }
        let state = a.read_resource::<MobaMatch>();
        assert_eq!(state.lane_towers.len(),3);
        assert!(state.lane_towers.iter().flatten().all(Option::is_some));
        assert!(!state.base_unlocked(0) && !state.base_unlocked(1));
        let positions = a.read_storage::<Pos>();
        let creeps = a.read_storage::<Creep>();
        let factions = a.read_storage::<Faction>();
        for team in [1,2] {
            let paths: std::collections::BTreeMap<_,_> = (&a.entities(),&creeps,&positions,&factions)
                .join().filter(|(_,_,_,f)| f.team_id == team)
                .map(|(_,c,p,_)| (c.path.as_str(),p.0)).collect();
            assert_eq!(paths.len(),3);
            assert!(paths["__moba_lane_0__"].y > Fixed64::from_i32(500));
            assert_eq!(paths["__moba_lane_1__"].y,Fixed64::ZERO);
            assert!(paths["__moba_lane_2__"].y < Fixed64::from_i32(-500));
        }
    }
}

#[test]
fn three_lane_base_damage_requires_all_lane_towers_and_finishes_once() {
    let (mut world,mut driver) = world(three_lane_config(),SimulationTickProfile::Production60Hz);
    driver.step(&mut world,[]).unwrap();
    let source = hero_pair(&world)[0];
    let towers: Vec<_> = world.read_resource::<MobaMatch>().lane_towers.iter().map(|lane|lane[1].unwrap()).collect();
    let base = world.read_resource::<MobaMatch>().bases[1].unwrap();
    let hp = world.read_storage::<CProperty>().get(base).unwrap().hp;
    for (index,tower) in towers.into_iter().enumerate() {
        assert!(!moba_damage_allowed(&world,base));
        world.write_resource::<Vec<Outcome>>().push(lethal(source,base));
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
        assert_eq!(world.read_storage::<CProperty>().get(base).unwrap().hp,hp);
        world.write_resource::<Vec<Outcome>>().push(lethal(source,tower));
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
        // Damage queues Death for the next outcome drain; unlock follows actual
        // death retirement, not HP reaching zero or a submitted attack.
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
        world.maintain();
        let state = world.read_resource::<MobaMatch>();
        assert_eq!(state.base_unlocked(1),index == 2);
        assert_eq!(state.towers[1].is_none(),index == 2);
    }
    assert!(moba_damage_allowed(&world,base));
    world.write_resource::<Vec<Outcome>>().extend([lethal(source,base),lethal(source,base)]);
    let result = driver.step(&mut world,[]).unwrap();
    assert_eq!(result.events.iter().filter(|event|event.topic == "game.end").count(),1);
    assert!(matches!(world.read_resource::<MobaMatch>().phase,MobaMatchPhase::Finished {winner:Some(0),..}));
    let digest = single_lane_replay_digest(&world);
    for _ in 0..15 {
        let result = driver.step(&mut world,[]).unwrap();
        assert!(!result.events.iter().any(|event|event.topic == "game.end"));
        assert_eq!(single_lane_replay_digest(&world),digest);
    }
}

#[test]
fn three_lane_unknown_map_and_mismatched_bases_reject_atomically() {
    for config in [SingleLaneConfig {map_id:Some("missing".into()),..fast_config()},
        SingleLaneConfig {lane_length:Fixed64::from_i32(3000),..three_lane_config()}] {
        let pool = StateInitializer::create_thread_pool();
        let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
        assert!(setup_single_lane_match(&mut world,config).is_err());
        assert!(world.try_fetch::<MobaMatch>().is_none());
        assert_eq!((&world.entities(),&world.read_storage::<Unit>()).join().count(),0);
    }
}

#[test]
fn layered_lane_towers_unlock_only_after_authoritative_retirement() {
    let config = SingleLaneConfig { map_id: Some("three_lane_layered_training".into()), ..three_lane_config() };
    let (mut world,mut driver) = world(config,SimulationTickProfile::Production60Hz);
    driver.step(&mut world,[]).unwrap();
    let source = hero_pair(&world)[0];
    let layers = world.read_resource::<MobaMatch>().lane_tower_layers.clone();
    assert_eq!(layers.len(),3);
    assert!(layers.iter().all(|lane| lane.len()==3));
    let base = world.read_resource::<MobaMatch>().bases[1].unwrap();
    for (lane_index,lane) in layers.iter().enumerate() {
        for (rank,layer) in lane.iter().enumerate() {
            let tower = layer[1].unwrap();
            assert!(moba_damage_allowed(&world,tower));
            if let Some(next) = lane.get(rank+1).and_then(|layer|layer[1]) {
                assert!(!moba_damage_allowed(&world,next));
                let hp = world.read_storage::<CProperty>().get(next).unwrap().hp;
                world.write_resource::<Vec<Outcome>>().push(lethal(source,next));
                process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
                assert_eq!(world.read_storage::<CProperty>().get(next).unwrap().hp,hp);
            }
            assert!(!moba_damage_allowed(&world,base));
            world.write_resource::<Vec<Outcome>>().push(lethal(source,tower));
            process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
            if let Some(next) = lane.get(rank+1).and_then(|layer|layer[1]) {
                assert!(!moba_damage_allowed(&world,next), "HP zero alone is not retirement");
            }
            process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
            world.maintain();
            let state = world.read_resource::<MobaMatch>();
            assert_eq!(state.lane_towers[lane_index][1],lane.get(rank+1).and_then(|layer|layer[1]));
            assert_eq!(state.base_unlocked(1),lane_index==2 && rank==2);
            assert!(!state.base_unlocked(0));
        }
    }
    assert!(moba_damage_allowed(&world,base));
}

#[test]
fn layered_lane_towers_replay_deterministically_at_60hz() {
    for seed in [1,42,0x20261005] {
        let config = SingleLaneConfig { seed,map_id:Some("three_lane_layered_training".into()), ..three_lane_config() };
        let (mut first,mut first_driver) = world(config.clone(),SimulationTickProfile::Production60Hz);
        let (mut second,mut second_driver) = world(config,SimulationTickProfile::Production60Hz);
        for tick in 0..180 {
            first_driver.step(&mut first,[]).unwrap();
            second_driver.step(&mut second,[]).unwrap();
            assert_eq!(single_lane_replay_digest(&first),single_lane_replay_digest(&second),"seed={seed} tick={tick}");
        }
    }
}

#[test]
fn three_lane_60hz_full_filtered_match_lifecycle() {
    for seed in [1,42,0x20261004] {
        full_match_filtered_lifecycle_with_map(SimulationTickProfile::Production60Hz,60,24_000,
            Some("three_lane_training"),seed);
    }
}

fn recall() -> PlayerInput {
    PlayerInput { action: Some(PlayerInputEnum::Recall(omoba_core::game_proto::Recall {})) }
}

fn upgrade(slot:u32) -> PlayerInput {
    PlayerInput {action:Some(PlayerInputEnum::UpgradeAbility(omoba_core::game_proto::UpgradeAbility {ability_index:slot}))}
}

#[test]
fn single_lane_lua_apprentice_birth_and_late_first_learning_at_60hz() {
    let (mut world,mut driver)=world(SingleLaneConfig {heroes:["training_apprentice".into(),"training_apprentice".into()],
        wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut world,[]).unwrap();
    let hero=hero_pair(&world)[0];
    {
        let heroes=world.read_storage::<Hero>(); let h=heroes.get(hero).unwrap();
        assert_eq!(h.id,"training_apprentice");
        assert_eq!(h.skill_points,1);
        assert!(h.abilities.iter().all(|id|h.get_ability_level(id)==0));
    }
    assert!(handle_ability_upgrade_from_input(&mut world,2,1).unwrap_err().to_string().contains("requires hero level 6"));
    assert_eq!(world.read_storage::<Hero>().get(hero).unwrap().skill_points,1);
    world.write_storage::<Hero>().get_mut(hero).unwrap().level=6;
    driver.step(&mut world,[(1,upgrade(2))]).unwrap();
    let heroes=world.read_storage::<Hero>(); let h=heroes.get(hero).unwrap();
    assert_eq!((h.get_ability_level("apprentice_lance"),h.skill_points),(1,0));
}

#[test]
fn single_lane_rank_zero_first_learning_cast_and_respawn_at_60hz() {
    let run = || {
        let (mut world, mut driver) = world(SingleLaneConfig {
            wave_interval: Fixed64::from_i32(10_000), ..fast_config()
        }, SimulationTickProfile::Production60Hz);
        driver.step(&mut world, []).unwrap();
        let [source,target] = hero_pair(&world);
        {
            let mut heroes = world.write_storage::<Hero>();
            apply_moba_birth_loadout(heroes.get_mut(source).unwrap(),
                omoba_template_ids::MobaLoadoutConst { ranks: [0;4], skill_points: 1 }).unwrap();
        }
        assert!(handle_ability_cast_from_input(&mut world,0,None,Some(target.id()),1)
            .unwrap_err().to_string().contains("not learned"));
        // Bypass input routing deliberately: queued script events must not
        // promote an unlearned hero skill to rank one either.
        world.write_resource::<ScriptEventQueue>().push(ScriptEvent::SkillCast {
            caster: source, skill_id: "lumen_bolt".into(), target: SkillTarget::Entity(target),
        });
        let before = world.read_storage::<CProperty>().get(target).unwrap().hp;
        let mut registry = ScriptRegistry::new(); registry.insert_manifest(crate::get_manifest());
        run_script_dispatch(&mut world,&registry,1,Fixed64::from_raw(17));
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
        assert_eq!(world.read_storage::<CProperty>().get(target).unwrap().hp,before);
        assert!(!world.read_storage::<Hero>().get(source).unwrap().is_on_cooldown("lumen_bolt"));
        driver.step(&mut world,[(1,upgrade(0)),(1,upgrade(0))]).unwrap();
        {
            let heroes = world.read_storage::<Hero>(); let hero = heroes.get(source).unwrap();
            assert_eq!((hero.get_ability_level("lumen_bolt"),hero.skill_points),(1,0));
            assert_eq!(hero.get_ability_level("lumen_touch"),0);
        }
        for (entity,x) in [(source,1100),(target,1200)] {
            world.write_storage::<Pos>().get_mut(entity).unwrap().0 = omoba_sim::Vec2::new(
                Fixed64::from_i32(x),Fixed64::from_i32(700));
        }
        world.write_storage::<CProperty>().get_mut(target).unwrap().def_magic = Fixed64::ZERO;
        let hp = world.read_storage::<CProperty>().get(target).unwrap().hp;
        driver.step(&mut world,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index:0,target_pos:None,target_entity:Some(target.id()),
        }))})]).unwrap();
        assert_eq!(hp-world.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(80));
        let tower = world.read_resource::<MobaMatch>().towers[1].unwrap();
        world.write_resource::<Vec<Outcome>>().push(lethal(tower,source));
        driver.step(&mut world,[]).unwrap();
        for _ in 0..130 {driver.step(&mut world,[]).unwrap();}
        let respawn = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
        assert_ne!(canonical_entity_id(source),canonical_entity_id(respawn));
        let heroes = world.read_storage::<Hero>(); let hero = heroes.get(respawn).unwrap();
        assert_eq!((hero.get_ability_level("lumen_bolt"),hero.skill_points),(1,0));
        assert_eq!(hero.get_ability_level("lumen_touch"),0);
        drop(heroes);
        single_lane_replay_digest(&world)
    };
    assert_eq!(run(),run());
}

#[test]
fn single_lane_birth_loadout_rejections_do_not_partially_mutate_hero() {
    let (world,_) = world(fast_config(),SimulationTickProfile::Production60Hz);
    let mut hero = world.read_storage::<Hero>().get(hero_pair(&world)[0]).unwrap().clone();
    for ranks in [[0,0,2,0],[0,0,0,255]] {
        let before = serde_json::to_value(&hero).unwrap();
        assert!(apply_moba_birth_loadout(&mut hero,omoba_template_ids::MobaLoadoutConst {
            ranks,skill_points:1,
        }).is_err());
        assert_eq!(before,serde_json::to_value(&hero).unwrap());
    }
}

#[test]
fn single_lane_upgrade_requires_lua_hero_level_before_spending_points() {
    for (rank, required) in [(1,6),(2,11),(3,16)] {
        let (mut world,mut driver)=world(fast_config(),SimulationTickProfile::Production60Hz);
        driver.step(&mut world,[]).unwrap(); let hero=hero_pair(&world)[0];
        { let mut heroes=world.write_storage::<Hero>(); let h=heroes.get_mut(hero).unwrap();
          h.level=required-1; h.skill_points=1; h.ability_levels.insert("lumen_lance".into(),rank); }
        let before=serde_json::to_value(world.read_storage::<Hero>().get(hero).unwrap()).unwrap();
        let error=handle_ability_upgrade_from_input(&mut world,2,1).unwrap_err();
        assert!(error.to_string().contains("requires hero level"));
        assert_eq!(before,serde_json::to_value(world.read_storage::<Hero>().get(hero).unwrap()).unwrap());
        world.write_storage::<Hero>().get_mut(hero).unwrap().level=required;
        driver.step(&mut world,[(1,upgrade(2))]).unwrap();
        let heroes=world.read_storage::<Hero>(); let h=heroes.get(hero).unwrap();
        assert_eq!((h.get_ability_level("lumen_lance"),h.skill_points),(rank+1,0));
    }
}

#[test]
fn single_lane_upgraded_bolt_uses_rank_damage_and_cooldown_at_60hz() {
    let run = |rank: i32| {
        let (mut world,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut world,[]).unwrap(); let [source,target]=hero_pair(&world);
        for (hero,x) in [(source,1100),(target,1200)] {
            world.write_storage::<Pos>().get_mut(hero).unwrap().0=omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(700));
        }
        world.write_storage::<CProperty>().get_mut(target).unwrap().def_magic=Fixed64::ZERO;
        if rank==2 {
            world.write_storage::<Hero>().get_mut(source).unwrap().skill_points=1;
            driver.step(&mut world,[(1,upgrade(0))]).unwrap();
        }
        let before=world.read_storage::<CProperty>().get(target).unwrap().hp;
        driver.step(&mut world,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index:0,target_pos:None,target_entity:Some(target.id()),
        }))})]).unwrap();
        let damage=before-world.read_storage::<CProperty>().get(target).unwrap().hp;
        let cooldown=world.read_storage::<Hero>().get(source).unwrap().get_cooldown("lumen_bolt");
        let definition=omoba_template_ids::ability_const(omoba_template_ids::ability_by_name("lumen_bolt").unwrap()).unwrap();
        assert!(cooldown<=definition.levels[(rank-1) as usize].cooldown && cooldown>definition.levels[(rank-1) as usize].cooldown-Fixed64::from_raw(40));
        damage
    };
    assert_eq!(run(1),Fixed64::from_i32(80));
    assert_eq!(run(2),Fixed64::from_i32(125));
}

#[test]
fn single_lane_formal_upgrade_spends_earned_point_once_and_survives_respawn() {
    let run=|| {
        let (mut world,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),..fast_config()},
            SimulationTickProfile::Production60Hz);
        driver.step(&mut world,[]).unwrap();
        let [a,b]=hero_pair(&world);
        world.write_resource::<Vec<Outcome>>().push(lethal(a,b));
        driver.step(&mut world,[]).unwrap();
        driver.step(&mut world,[]).unwrap();
        assert_eq!(world.read_storage::<Hero>().get(a).unwrap().skill_points,1);
        world.write_storage::<Hero>().get_mut(a).unwrap().ability_cooldowns.insert("lumen_bolt".into(),Fixed64::from_i32(7));
        driver.step(&mut world,[(1,upgrade(0)),(1,upgrade(0))]).unwrap();
        let heroes=world.read_storage::<Hero>();let h=heroes.get(a).unwrap();
        assert_eq!(h.get_ability_level("lumen_bolt"),2);
        assert_eq!(h.skill_points,0);
        assert!(h.get_cooldown("lumen_bolt")>Fixed64::from_i32(6),"upgrade must not reset cooldown");
        assert_eq!(h.get_ability_level("lumen_touch"),1);
        drop(heroes);
        let source=world.read_resource::<MobaMatch>().towers[1].unwrap();
        world.write_resource::<Vec<Outcome>>().push(lethal(source,a));
        driver.step(&mut world,[]).unwrap();
        for _ in 0..130 {driver.step(&mut world,[]).unwrap();}
        let new=world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
        assert_ne!(canonical_entity_id(a),canonical_entity_id(new));
        let heroes=world.read_storage::<Hero>();let h=heroes.get(new).unwrap();
        assert_eq!((h.get_ability_level("lumen_bolt"),h.skill_points),(2,0));
        drop(heroes);
        single_lane_replay_digest(&world)
    };
    assert_eq!(run(),run(),"60Hz upgrade replay");
}

#[test]
fn single_lane_upgrade_rejections_are_atomic() {
    for case in 0..11 {
        let (mut world,mut driver)=world(fast_config(),SimulationTickProfile::Production60Hz);
        driver.step(&mut world,[]).unwrap();
        let hero=hero_pair(&world)[0];
        world.write_storage::<Hero>().get_mut(hero).unwrap().skill_points=1;
        let mut slot=0;let mut owner=1;
        match case {
            0=>world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Warmup,
            1=>world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Finished {winner:Some(0),tick:1},
            2=>world.write_resource::<GamePause>().is_paused=true,
            3=>world.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::ZERO,
            4=>owner=999,
            5=>slot=4,
            6=>world.write_storage::<Hero>().get_mut(hero).unwrap().skill_points=0,
            7=>{world.write_storage::<Hero>().get_mut(hero).unwrap().ability_levels.insert("lumen_bolt".into(),4);},
            8=>{world.write_storage::<Hero>().get_mut(hero).unwrap().ability_levels.insert("lumen_bolt".into(),-1);},
            9=>world.insert(AbilityRegistry::default()),
            10=>world.write_storage::<Hero>().get_mut(hero).unwrap().abilities[0]="".into(),
            _=>unreachable!(),
        }
        let before=serde_json::to_value(world.read_storage::<Hero>().get(hero).unwrap()).unwrap();
        assert!(handle_ability_upgrade_from_input(&mut world,slot,owner).is_err(),"case {case}");
        assert_eq!(before,serde_json::to_value(world.read_storage::<Hero>().get(hero).unwrap()).unwrap(),"case {case}");
    }
}

#[test]
fn single_lane_60hz_upgrade_ranks_follow_visible_enemy_without_disclosing_inputs() {
    assert_upgrade_filtered_parity(1);
}

#[test]
fn single_lane_60hz_first_learning_preserves_both_team_hashes_and_private_inputs() {
    assert_upgrade_filtered_parity(0);
}

fn assert_upgrade_filtered_parity(initial_rank: u8) {
    use std::collections::BTreeSet;
    use prost::Message;
    let (mut authority,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),..fast_config()},
        SimulationTickProfile::Production60Hz);
    let [a,b]=hero_pair(&authority);
    for (index,hero) in [a,b].into_iter().enumerate() {
        apply_moba_birth_loadout(authority.write_storage::<Hero>().get_mut(hero).unwrap(),
            omoba_template_ids::MobaLoadoutConst {ranks:[initial_rank;4],skill_points:2}).unwrap();
        authority.write_storage::<Pos>().get_mut(hero).unwrap().0=omoba_sim::Vec2::new(
            Fixed64::from_i32(1100+index as i32*100),Fixed64::from_i32(700));
        // Precondition fixture supplies earned-point state before bootstrap;
        // the separate production-kill test validates earning and persistence.
        authority.write_storage::<Hero>().get_mut(hero).unwrap().skill_points=2;
        authority.write_storage::<Hero>().get_mut(hero).unwrap().level=6;
    }
    let first=driver.step(&mut authority,[]).unwrap();project_tick(&mut authority,first);
    let allow=TeamProjectorConfig::default().component_allowlist;
    let starts=authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick()+1,60,fast_config().seed);
    let mut replicas:Vec<_>=starts.into_iter().map(|(team,start)| {
        let replica=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),BTreeSet::new()).unwrap();
        let mut stepper=SpecsDisclosedWorldStepper::from_start(&start,allow.clone(),BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world,&stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();(team,replica,stepper)
    }).collect();
    for step in 0..40 {
        let inputs=match step {0=>vec![(1,upgrade(0)),(2,upgrade(3))],1=>vec![(1,upgrade(2)),(2,upgrade(3))],
            2=>vec![(1,upgrade(2)),(2,upgrade(3))],_=>vec![]};
        let accepted:Vec<_>=inputs.iter().map(|(player,input)| CanonicalAcceptedInput::from_authoritative_acceptance(
            *player,*player,step+1,10,canonical_entity_id(if *player==1 {a}else{b}),None,input.encode_to_vec())).collect();
        let result=driver.step(&mut authority,inputs).unwrap();
        authority.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
        project_tick(&mut authority,result);
        let frames=authority.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected=authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick()+1,60,fast_config().seed);
        for (team,replica,stepper) in &mut replicas {
            let frame=frames[team].frame.clone();
            assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            assert!(frame.step.as_ref().unwrap().accepted_inputs.iter().all(|i|i.player_id==*team));
            assert_eq!(frame.step.as_ref().unwrap().public_events.iter().filter(|e|e.event_kind==FactKind::CommittedAbilityRanks as u32).count(),2);
            assert!(matches!(replica.apply_frame(frame,stepper).unwrap(),FrameApplyResult::Applied {..}));
            let expected=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team],allow.clone(),BTreeSet::new()).unwrap();
            assert_eq!(replica.canonical_team_hash(),expected.canonical_team_hash(),"team {team} tick {}",driver.tick());
        }
    }
    let heroes=authority.read_storage::<Hero>();
    assert_eq!((heroes.get(a).unwrap().get_ability_level("lumen_bolt"),heroes.get(a).unwrap().skill_points),(i32::from(initial_rank)+1,0));
    assert_eq!(heroes.get(a).unwrap().get_ability_level("lumen_lance"),i32::from(initial_rank)+1);
    assert_eq!((heroes.get(b).unwrap().get_ability_level("lumen_mend"),heroes.get(b).unwrap().skill_points),(i32::from(initial_rank)+2,0));
}

fn hero_pair(world: &World) -> [specs::Entity; 2] {
    let state = world.read_resource::<MobaMatch>();
    std::array::from_fn(|index| state.heroes[index].entity.unwrap())
}

fn extra_player(player_id: u32, team_id: u32) -> SingleLanePlayerConfig {
    SingleLanePlayerConfig { player_id, team_id, hero: "training_luminary".into() }
}

fn lane_creeps(world: &World, team: i32) -> Vec<specs::Entity> {
    (&world.entities(), &world.read_storage::<Creep>(), &world.read_storage::<Faction>())
        .join().filter(|(_,_,f)| f.team_id == team)
        .map(|(e,_,_)| e).collect()
}

#[test]
fn hold_position_filtered_60hz_uses_formal_inputs_and_safe_priority_replay() {
    use std::collections::BTreeSet;
    use prost::Message;
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..fast_config()},SimulationTickProfile::Production60Hz);
    let first=driver.step(&mut w,[]).unwrap();project_tick(&mut w,first);
    let [a,b]=hero_pair(&w);let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(a).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(b).unwrap().0=origin+
        omoba_sim::Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO);
    let second=driver.step(&mut w,[]).unwrap();project_tick(&mut w,second);
    let allow=TeamProjectorConfig::default().component_allowlist;
    let starts=w.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick()+1,60,fast_config().seed);
    let mut replicas:Vec<_>=starts.into_iter().map(|(team,start)| {
        let replica=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),BTreeSet::new()).unwrap();
        let mut stepper=SpecsDisclosedWorldStepper::from_start(&start,allow.clone(),BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world,&stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();(team,replica,stepper)
    }).collect();
    for step in 0..6 {
        let inputs=match step {
            0=>vec![(1,PlayerInput {action:Some(PlayerInputEnum::HoldPosition(omoba_core::game_proto::HoldPosition {queued:false}))})],
            3=>vec![(1,PlayerInput {action:Some(PlayerInputEnum::MoveTo(MoveTo {target:Some(Vec2I {
                x:200*1024,y:7000*1024}),queued:false}))})],_=>vec![],
        };
        let accepted:Vec<_>=inputs.iter().map(|(player,input)|CanonicalAcceptedInput::from_authoritative_acceptance(
            *player,*player,step+1,10,canonical_entity_id(a),None,input.encode_to_vec())).collect();
        let result=driver.step(&mut w,inputs).unwrap();
        w.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
        project_tick(&mut w,result);
        if step<3 {assert_eq!(w.read_storage::<HeroCommandQueue>().get(a).unwrap().active,Some(HeroCommand::HoldPosition));}
        if step==3 {assert!(matches!(w.read_storage::<HeroCommandQueue>().get(a).unwrap().active,Some(HeroCommand::MoveTo {..})));}
        let frames=w.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected=w.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick()+1,60,fast_config().seed);
        for (team,replica,stepper) in &mut replicas {
            let frame=frames[team].frame.clone();
            assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            assert!(frame.step.as_ref().unwrap().accepted_inputs.iter().all(|i|i.player_id==*team));
            replica.apply_frame(frame,stepper).unwrap();
            let expected=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team],allow.clone(),BTreeSet::new()).unwrap();
            assert_eq!(replica.canonical_team_hash(),expected.canonical_team_hash(),"team {team} step {step}");
        }
    }
}

#[test]
fn point_queue_formal_60hz_appends_behind_hold_and_immediate_move_replaces() {
    let (mut w, mut driver) = world(SingleLaneConfig {
        wave_interval: Fixed64::from_i32(10_000), ..fast_config()
    }, SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let [owner, _] = hero_pair(&w);
    let origin = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(owner).unwrap().0 = origin;
    driver.step(&mut w, [(1, PlayerInput { action: Some(PlayerInputEnum::HoldPosition(
        omoba_core::game_proto::HoldPosition { queued: false })) })]).unwrap();
    let move_pos = omoba_sim::Vec2::new(Fixed64::from_i32(200), Fixed64::from_i32(7000));
    let attack_pos = omoba_sim::Vec2::new(Fixed64::from_i32(400), Fixed64::from_i32(7000));
    driver.step(&mut w, [
        (1, PlayerInput { action: Some(PlayerInputEnum::MoveTo(MoveTo {
            target: Some(Vec2I { x: 200 * 1024, y: 7000 * 1024 }), queued: true })) }),
        (1, PlayerInput { action: Some(PlayerInputEnum::AttackMove(omoba_core::game_proto::AttackMove {
            target: Some(Vec2I { x: 400 * 1024, y: 7000 * 1024 }), queued: true })) }),
    ]).unwrap();
    {
        let commands = w.read_storage::<HeroCommandQueue>();
        let commands = commands.get(owner).unwrap();
        assert_eq!(commands.active, Some(HeroCommand::HoldPosition));
        assert_eq!(commands.queued, vec![HeroCommand::MoveTo { pos: move_pos }, HeroCommand::AttackMove { pos: attack_pos }]);
    }
    assert_eq!(w.read_storage::<Pos>().get(owner).unwrap().0, origin);
    driver.step(&mut w, [(1, PlayerInput { action: Some(PlayerInputEnum::MoveTo(MoveTo {
        target: Some(Vec2I { x: 200 * 1024, y: 7000 * 1024 }), queued: false })) })]).unwrap();
    {
        let commands = w.read_storage::<HeroCommandQueue>();
        let commands = commands.get(owner).unwrap();
        assert_eq!(commands.active, Some(HeroCommand::MoveTo { pos: move_pos }));
        assert!(commands.queued.is_empty());
    }
    for _ in 0..20 { driver.step(&mut w, []).unwrap(); }
    assert_ne!(w.read_storage::<Pos>().get(owner).unwrap().0, origin);
}

#[test]
fn jungle_patrol_navigation_60hz_skips_blocked_camp_holds_then_resumes() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let owner=hero_pair(&w)[0];
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    w.write_storage::<Pos>().get_mut(owner).unwrap().0=origin;
    let points:Vec<_>=w.read_resource::<MobaMatch>().jungle_camps.iter().map(|camp| {
        let (x,y)=camp.definition.position;
        omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))
    }).collect();
    assert_eq!(points.len(),2,"compiled training fixture");
    let original=(*w.read_resource::<BlockedRegions>()).clone();
    let block=|point:omoba_sim::Vec2| {
        let x=point.x.to_f32_for_render();let y=point.y.to_f32_for_render();
        BlockedRegion {name:"camp-patrol-fixture".into(),points:vec![
            vek::Vec2::new(x-40.0,y-40.0),vek::Vec2::new(x+40.0,y-40.0),
            vek::Vec2::new(x+40.0,y+40.0),vek::Vec2::new(x-40.0,y+40.0)]}
    };
    w.write_resource::<BlockedRegions>().0.push(block(points[0]));
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Jungle,lane:1,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:None,item_builds:Vec::new()};
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::AttackMove(a))
        if a.target.as_ref().is_some_and(|p|i64::from(p.x)==points[1].x.raw() && i64::from(p.y)==points[1].y.raw())));
    driver.step(&mut w,input).unwrap();
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active,
        Some(HeroCommand::AttackMove {pos:points[1]}));
    w.write_resource::<BlockedRegions>().0.push(block(points[1]));
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::HoldPosition(_))));
    driver.step(&mut w,input).unwrap();
    let held=w.read_storage::<Pos>().get(owner).unwrap().0;
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"unreachable patrol must not spam commands");
    driver.step(&mut w,[]).unwrap();
    assert_eq!(w.read_storage::<Pos>().get(owner).unwrap().0,held);
    *w.write_resource::<BlockedRegions>()=original;
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::AttackMove(a))
        if a.target.as_ref().is_some_and(|p|i64::from(p.x)==points[0].x.raw() && i64::from(p.y)==points[0].y.raw())));
    driver.step(&mut w,input).unwrap();
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active,
        Some(HeroCommand::AttackMove {pos:points[0]}));
    // Acceptance creates the command after the movement phase of this tick.
    driver.step(&mut w,[]).unwrap();
    assert_ne!(w.read_storage::<Pos>().get(owner).unwrap().0,held,"normal travel resumes without teleport");
}

#[test]
fn jungle_siege_route_60hz_skips_blocked_candidate_and_reconsiders_public_terrain() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let owner=hero_pair(&w)[0];
    let (near,far)={let state=w.read_resource::<MobaMatch>();
        (state.lane_towers[1][1].unwrap(),state.lane_towers[0][1].unwrap())};
    let near_pos=w.read_storage::<Pos>().get(near).unwrap().0;
    let far_pos=w.read_storage::<Pos>().get(far).unwrap().0;
    w.write_storage::<Pos>().get_mut(owner).unwrap().0=near_pos-
        omoba_sim::Vec2::new(Fixed64::from_i32(700),Fixed64::ZERO);
    let waves=lane_creeps(&w,1);
    for (wave,point) in [(waves[0],near_pos),(waves[1],far_pos)] {
        w.write_storage::<Pos>().get_mut(wave).unwrap().0=point-
            omoba_sim::Vec2::new(Fixed64::from_i32(50),Fixed64::ZERO);
    }
    let original=(*w.read_resource::<BlockedRegions>()).clone();
    let x=near_pos.x.to_f32_for_render();let y=near_pos.y.to_f32_for_render();
    w.write_resource::<BlockedRegions>().0.push(BlockedRegion {name:"siege-route-fixture".into(),
        points:vec![vek::Vec2::new(x-30.0,y-30.0),vek::Vec2::new(x+30.0,y-30.0),
            vek::Vec2::new(x+30.0,y+30.0),vek::Vec2::new(x-30.0,y+30.0)]});
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Jungle,lane:1,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:None,item_builds:Vec::new()};
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let travel_point=|inputs:Vec<(u32,PlayerInput)>|inputs.into_iter().find_map(|(_,i)|match i.action {
        Some(PlayerInputEnum::MoveTo(m))=>m.target.map(|p|(i64::from(p.x),i64::from(p.y))),_=>None});
    assert_eq!(travel_point(role_bot_inputs(&w,&bots).unwrap()),Some((far_pos.x.raw(),far_pos.y.raw())),
        "nearest disclosed tower is occupied by public terrain; use the next reachable one");
    let far_key=canonical_entity_id(far);
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.remove(&far_key);
    assert_eq!(travel_point(role_bot_inputs(&w,&bots).unwrap()),None,
        "hidden reachable target cannot bypass the blocked current candidate");
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.insert(far_key);
    *w.write_resource::<BlockedRegions>()=original;
    let travel=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(travel_point(travel.clone()),Some((near_pos.x.raw(),near_pos.y.raw())),
        "no persistent rejection cache after terrain changes");
    driver.step(&mut w,travel).unwrap();
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active,
        Some(HeroCommand::MoveTo {pos:near_pos}));
}

#[test]
fn jungle_siege_travel_60hz_moves_without_autoaggression_then_hands_off_to_attack() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let owner=hero_pair(&w)[0];
    let tower=w.read_resource::<MobaMatch>().lane_towers[1][1].unwrap();
    let tower_pos=w.read_storage::<Pos>().get(tower).unwrap().0;
    let own=tower_pos-omoba_sim::Vec2::new(Fixed64::from_i32(700),Fixed64::ZERO);
    w.write_storage::<Pos>().get_mut(owner).unwrap().0=own;
    {let mut health=w.write_storage::<CProperty>();let p=health.get_mut(owner).unwrap();
        p.hp=Fixed64::from_i32(10_000);p.mhp=p.hp;}
    let wave=lane_creeps(&w,1)[0];
    w.write_storage::<Pos>().get_mut(wave).unwrap().0=tower_pos-
        omoba_sim::Vec2::new(Fixed64::from_i32(50),Fixed64::ZERO);
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Jungle,lane:1,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:None,item_builds:Vec::new()};
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let tower_key=canonical_entity_id(tower);
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.remove(&tower_key);
    assert!(!role_bot_inputs(&w,&bots).unwrap().iter().any(|(_,i)|matches!(&i.action,
        Some(PlayerInputEnum::MoveTo(m)) if m.target.as_ref().is_some_and(|p|
            i64::from(p.x)==tower_pos.x.raw() && i64::from(p.y)==tower_pos.y.raw()))),
        "cached hidden tower must not create new siege travel");
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.insert(tower_key);
    let wave_key=canonical_entity_id(wave);
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.remove(&wave_key);
    assert!(matches!(&role_bot_inputs(&w,&bots).unwrap()[0].1.action,
        Some(PlayerInputEnum::HoldPosition(_))),"cached hidden wave cannot authorize approach");
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.insert(wave_key);
    let travel=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(travel.len(),1);
    assert!(matches!(&travel[0].1.action,Some(PlayerInputEnum::MoveTo(m))
        if !m.queued && m.target.as_ref().is_some_and(|p| i64::from(p.x)==tower_pos.x.raw()
            && i64::from(p.y)==tower_pos.y.raw())),"siege travel must not auto-acquire lane targets");
    driver.step(&mut w,travel).unwrap();
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active,
        Some(HeroCommand::MoveTo {pos:tower_pos}));
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"same travel is not resubmitted");
    let mut handed_off=false;
    for _ in 0..180 {
        run_committed_visibility_wave_b(&mut w,driver.tick(),0);
        let inputs=role_bot_inputs(&w,&bots).unwrap();
        if inputs.iter().any(|(_,i)|matches!(&i.action,
            Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==tower.id())) {
            driver.step(&mut w,inputs).unwrap();handed_off=true;break;
        }
        driver.step(&mut w,inputs).unwrap();
    }
    assert!(handed_off,"normal movement must reach local siege selection");
    assert_ne!(w.read_storage::<Pos>().get(owner).unwrap().0,own);
    assert!(matches!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active,
        Some(HeroCommand::AttackTarget {target,..}) if target==tower));
}

#[test]
fn jungle_siege_60hz_waits_for_disclosed_wave_then_submits_formal_attack() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let owner=hero_pair(&w)[0];
    let tower=w.read_resource::<MobaMatch>().lane_towers[1][1].unwrap();
    let tower_pos=w.read_storage::<Pos>().get(tower).unwrap().0;
    let own=tower_pos-omoba_sim::Vec2::new(Fixed64::from_i32(400),Fixed64::ZERO);
    w.write_storage::<Pos>().get_mut(owner).unwrap().0=own;
    {let mut health=w.write_storage::<CProperty>();let p=health.get_mut(owner).unwrap();
        p.hp=Fixed64::from_i32(10_000);p.mhp=p.hp;}
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Jungle,lane:1,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:None,item_builds:Vec::new()};
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let hold=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(hold.len(),1);
    assert!(matches!(&hold[0].1.action,Some(PlayerInputEnum::HoldPosition(h)) if !h.queued));
    driver.step(&mut w,hold).unwrap();
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active,Some(HeroCommand::HoldPosition));
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let wave=lane_creeps(&w,1)[0];
    w.write_storage::<Pos>().get_mut(wave).unwrap().0=tower_pos-
        omoba_sim::Vec2::new(Fixed64::from_i32(50),Fixed64::ZERO);
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"uncommitted wave cannot release hold");
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let attack=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(attack.len(),1);
    assert!(matches!(&attack[0].1.action,Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==tower.id()));
    driver.step(&mut w,attack).unwrap();
    assert!(matches!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active,
        Some(HeroCommand::AttackTarget {target,..}) if target==tower));
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current
        .remove(&canonical_entity_id(tower));
    assert!(!role_bot_inputs(&w,&bots).unwrap().iter().any(|(_,input)|matches!(&input.action,
        Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==tower.id())),
        "a cached hidden structure cannot supply a siege target");
}

#[test]
fn siege_wave_hold_position_60hz_persists_without_autoattack_and_releases_to_wave() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let caster=hero_pair(&w)[0];
    let tower=w.read_resource::<MobaMatch>().towers[1].unwrap();let wave=lane_creeps(&w,1)[0];
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(tower).unwrap().0=origin+
        omoba_sim::Vec2::new(Fixed64::from_i32(400),Fixed64::ZERO);
    w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::from_i32(10_000);
    w.write_storage::<CProperty>().get_mut(caster).unwrap().mhp=Fixed64::from_i32(10_000);
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Mid,lane:0,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:None,item_builds:Vec::new()};
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::HoldPosition(h)) if !h.queued));
    driver.step(&mut w,input).unwrap();
    let hp=w.read_storage::<CProperty>().get(tower).unwrap().hp;
    for _ in 0..65 {
        driver.step(&mut w,[]).unwrap();
        assert_eq!(w.read_storage::<HeroCommandQueue>().get(caster).unwrap().active,Some(HeroCommand::HoldPosition));
        assert!(w.read_storage::<MoveTarget>().get(caster).is_none());
    }
    assert_eq!(w.read_storage::<Pos>().get(caster).unwrap().0,origin);
    assert_eq!(w.read_storage::<CProperty>().get(tower).unwrap().hp,hp,"persistent hold must suppress autoattack");
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"do not resend the hold each think");
    w.write_storage::<Pos>().get_mut(wave).unwrap().0=origin+
        omoba_sim::Vec2::new(Fixed64::from_i32(450),Fixed64::ZERO);
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"uncommitted wave must not release hold");
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let attack=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&attack[0].1.action,Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==tower.id()));
    driver.step(&mut w,attack).unwrap();
    assert!(matches!(w.read_storage::<HeroCommandQueue>().get(caster).unwrap().active,
        Some(HeroCommand::AttackTarget {target,..}) if target==tower));
}

#[test]
fn disclosed_structure_state_60hz_layer_retirement_updates_bots_and_filtered_hashes() {
    use std::collections::BTreeSet;
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {map_id:Some("three_lane_layered_training".into()),
        wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
    let first=driver.step(&mut w,[]).unwrap();project_tick(&mut w,first);
    let caster=hero_pair(&w)[0];let layers=w.read_resource::<MobaMatch>().lane_tower_layers.clone();
    let (outer,inner)=(layers[0][0][1].unwrap(),layers[0][1][1].unwrap());
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    for (e,x) in [(outer,400),(inner,100)] {
        w.write_storage::<Pos>().get_mut(e).unwrap().0=origin+
            omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO);
    }
    let wave=lane_creeps(&w,1)[0];
    w.write_storage::<Pos>().get_mut(wave).unwrap().0=origin+
        omoba_sim::Vec2::new(Fixed64::from_i32(450),Fixed64::ZERO);
    {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(wave).unwrap();
        p.hp=Fixed64::from_i32(10_000);p.mhp=p.hp;}
    let second=driver.step(&mut w,[]).unwrap();project_tick(&mut w,second);
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Carry,lane:0,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:None,item_builds:Vec::new()};
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==outer.id()),
        "nearer locked layer must not outrank the disclosed frontier: input={:?} wave_pos={:?} wave_hp={:?}",
        input,w.read_storage::<Pos>().get(wave),w.read_storage::<CProperty>().get(wave));
    let allow=TeamProjectorConfig::default().component_allowlist;
    let starts=w.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick()+1,60,fast_config().seed);
    let mut replicas:Vec<_>=starts.into_iter().map(|(team,start)| {
        let replica=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),BTreeSet::new()).unwrap();
        let mut stepper=SpecsDisclosedWorldStepper::from_start(&start,allow.clone(),BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world,&stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();(team,replica,stepper)
    }).collect();
    w.write_resource::<Vec<Outcome>>().push(lethal(caster,outer));
    process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
    assert!(!moba_damage_allowed(&w,inner),"HP zero alone must not unlock");
    process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();w.maintain();
    assert!(moba_damage_allowed(&w,inner));
    // Hide/reveal the newly unlocked inner layer; cache is never an audience.
    for step in 0..4 {
        if step==1 {w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin+
            omoba_sim::Vec2::new(Fixed64::from_i32(6000),Fixed64::ZERO);}
        if step==2 {w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;}
        if step==3 {
            // Isolate terminal observation timing, not a legal locked-base kill.
            let base=w.read_resource::<MobaMatch>().bases[1].unwrap();
            let pos=w.read_storage::<Pos>().get(base).unwrap().0;
            w.write_resource::<Vec<Outcome>>().push(Outcome::Death {ent:base,pos});
        }
        let result=driver.step(&mut w,[]).unwrap();project_tick(&mut w,result);
        if step==0 {
            let next=role_bot_inputs(&w,&bots).unwrap();
            assert!(matches!(&next[0].1.action,Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==inner.id()));
        }
        if step==1 {
            assert!(!role_bot_inputs(&w,&bots).unwrap().iter().any(|(_,i)|
                matches!(&i.action,Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==inner.id())));
        }
        if step==3 {
            assert!(matches!(w.read_resource::<MobaMatch>().phase,MobaMatchPhase::Finished {..}));
            assert!(role_bot_inputs(&w,&bots).unwrap().is_empty());
        }
        let frames=w.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected=w.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick()+1,60,fast_config().seed);
        for (team,replica,stepper) in &mut replicas {
            let frame=frames[team].frame.clone();
            assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            replica.apply_frame(frame,stepper).unwrap();
            let expected=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team],allow.clone(),BTreeSet::new()).unwrap();
            assert_eq!(replica.canonical_team_hash(),expected.canonical_team_hash(),"team {team} step {step}");
        }
    }
}

#[test]
fn carry_attack_priority_60hz_preserves_farming_windup_but_not_recovery_or_backswing() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..fast_config()},SimulationTickProfile::Production60Hz);
    let first=driver.step(&mut w,[]).unwrap();
    let caster=hero_pair(&w)[0];let creep=lane_creeps(&w,2)[0];
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(creep).unwrap().0=origin+
        omoba_sim::Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO);
    w.write_storage::<CProperty>().get_mut(creep).unwrap().hp=Fixed64::from_i32(20);
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Carry,lane:0,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:vec![
            BotAbilityPolicy {ability:"lumen_bolt".into(),intent:BotAbilityIntent::EnemyUnit},
            BotAbilityPolicy {ability:"lumen_mend".into(),intent:BotAbilityIntent::SelfHeal {below_hp_per_mille:600}},
        ],ability_learning:Vec::new(),sustain:None,item_builds:Vec::new()};
    {let mut attacks=w.write_storage::<TAttack>();let a=attacks.get_mut(caster).unwrap();
        a.asd_count=Fixed64::from_i32(10);a.attack_phase=AttackSequencePhase::Idle;}
    run_committed_visibility_wave_b(&mut w,first.tick,0);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==creep.id()),
        "ready last hit must precede an available offensive ability");
    let result=driver.step(&mut w,input).unwrap();
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    {let mut attacks=w.write_storage::<TAttack>();let a=attacks.get_mut(caster).unwrap();
        a.asd_count=Fixed64::from_raw(-100);a.attack_phase=AttackSequencePhase::Windup;}
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"keep the admitted attack, no repeated command or offense");
    let healthy=w.read_storage::<CProperty>().get(caster).unwrap().hp;
    w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::from_i32(20);
    let heal=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&heal[0].1.action,Some(PlayerInputEnum::CastAbility(c)) if c.ability_index==3),
        "recovery remains eligible even after offensive-first author policy");
    let before=w.read_storage::<CProperty>().get(caster).unwrap().hp;
    driver.step(&mut w,heal).unwrap();
    assert!(w.read_storage::<CProperty>().get(caster).unwrap().hp>before,"normal generated recovery executed");
    w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=healthy;
    {let mut attacks=w.write_storage::<TAttack>();let a=attacks.get_mut(caster).unwrap();
        a.asd_count=Fixed64::from_raw(400);a.attack_phase=AttackSequencePhase::Backswing;}
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let cast=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&cast[0].1.action,Some(PlayerInputEnum::CastAbility(c)) if c.ability_index==0),
        "launched attack backswing must not starve offensive casting");
}

#[test]
fn incoming_damage_observation_60hz_filtered_updates_without_component_repair() {
    use std::collections::BTreeSet;
    let (mut authority,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..fast_config()},SimulationTickProfile::Production60Hz);
    let first=driver.step(&mut authority,[]).unwrap();project_tick(&mut authority,first);
    let [a,b]=hero_pair(&authority);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    authority.write_storage::<Pos>().get_mut(a).unwrap().0=origin;
    authority.write_storage::<Pos>().get_mut(b).unwrap().0=origin+
        omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
    let second=driver.step(&mut authority,[]).unwrap();project_tick(&mut authority,second);
    let allow=TeamProjectorConfig::default().component_allowlist;
    let starts=authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick()+1,60,fast_config().seed);
    let mut replicas:Vec<_>=starts.into_iter().map(|(team,start)| {
        let replica=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),BTreeSet::new()).unwrap();
        let mut stepper=SpecsDisclosedWorldStepper::from_start(&start,allow.clone(),BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world,&stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();(team,replica,stepper)
    }).collect();
    for (index,raw) in [512i64,-512,-1024,0,512,-512].into_iter().enumerate() {
        if index==4 {
            authority.write_storage::<Pos>().get_mut(b).unwrap().0=origin+
                omoba_sim::Vec2::new(Fixed64::from_i32(6000),Fixed64::ZERO);
        } else if index==5 {
            authority.write_storage::<Pos>().get_mut(b).unwrap().0=origin+
                omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
        }
        authority.write_resource::<BuffStore>().add(b,"incoming_projection_fixture",Fixed64::from_i32(10),
            serde_json::json!({"damage_taken_bonus":raw}));
        let result=driver.step(&mut authority,[]).unwrap();project_tick(&mut authority,result);
        let frames=authority.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected=authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick()+1,60,fast_config().seed);
        for (team,replica,stepper) in &mut replicas {
            let frame=frames[team].frame.clone();
            assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            if index<5 {
                assert_eq!(frame.step.as_ref().unwrap().public_events.iter().filter(|e|
                    e.event_kind==FactKind::CommittedIncomingDamage as u32).count(),
                    usize::from(index!=4 || *team==2),"hidden updates must not leak");
            }
            assert!(matches!(replica.apply_frame(frame,stepper).unwrap(),FrameApplyResult::Applied {..}));
            let expected=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team],allow.clone(),BTreeSet::new()).unwrap();
            assert_eq!(replica.canonical_team_hash(),expected.canonical_team_hash(),"team {team} bonus {raw}");
        }
    }
}

#[test]
fn carry_armed_attack_bonus_60hz_uses_owner_packet_without_consuming_observation() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w, mut driver) = world(SingleLaneConfig {
        wave_interval: Fixed64::from_i32(10_000), ..fast_config()
    }, SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let [owner, other] = hero_pair(&w);
    let creeps = lane_creeps(&w, 2);
    let (immune, killable) = (creeps[0], creeps[1]);
    let origin = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(7000));
    {
        let mut poses = w.write_storage::<Pos>();
        poses.get_mut(owner).unwrap().0 = origin;
        for (entity, x) in [(immune, 50), (killable, 100)] {
            poses.get_mut(entity).unwrap().0 = origin
                + omoba_sim::Vec2::new(Fixed64::from_i32(x), Fixed64::ZERO);
        }
    }
    {
        let mut health = w.write_storage::<CProperty>();
        health.get_mut(immune).unwrap().hp = Fixed64::ONE;
        health.get_mut(killable).unwrap().hp = Fixed64::from_i32(80);
    }
    {
        let mut attacks = w.write_storage::<TAttack>();
        let attack = attacks.get_mut(owner).unwrap();
        attack.atk_physic.v = Fixed64::from_i32(30);
        attack.range.v = Fixed64::from_i32(550);
        attack.asd_count = Fixed64::from_i32(10);
    }
    w.write_resource::<BuffStore>().add(immune, "incoming_fixture", Fixed64::from_i32(10),
        serde_json::json!({"damage_taken_bonus": -1024}));
    let step = driver.step(&mut w, []).unwrap();
    run_committed_visibility_wave_b(&mut w, step.tick, 0);
    let bots = RoleBotConfig {assignments: vec![BotAssignment {
        player_id: 1, role: BotRole::Carry, lane: 0, escort_player_id: None,
    }], think_interval_ticks: 1, ability_policies: Vec::new(),
        ability_learning: Vec::new(), sustain: None, item_builds: Vec::new()};
    // An opponent's armed bonus cannot make our own packet stronger.
    w.write_resource::<BuffStore>().arm_next_attack_bonus(other, Fixed64::from_i32(1000));
    let before = role_bot_inputs(&w, &bots).unwrap();
    assert!(!before.iter().any(|(_, input)| matches!(&input.action,
        Some(PlayerInputEnum::AttackTarget(a)) if a.target_id == killable.id())));
    assert!(w.write_resource::<BuffStore>().arm_next_attack_bonus(owner, Fixed64::from_i32(60)));
    let inputs = role_bot_inputs(&w, &bots).unwrap();
    assert_eq!(inputs.len(), 1);
    assert!(matches!(&inputs[0].1.action,
        Some(PlayerInputEnum::AttackTarget(a)) if a.target_id == killable.id()));
    assert_eq!(role_bot_inputs(&w, &bots).unwrap(), inputs);
    assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(owner), Fixed64::from_i32(60));
    let canonical = canonical_entity_id(killable);
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.remove(&canonical);
    assert!(!role_bot_inputs(&w, &bots).unwrap().iter().any(|(_, input)| matches!(&input.action,
        Some(PlayerInputEnum::AttackTarget(a)) if a.target_id == killable.id())),
        "armed damage cannot turn a cached hidden target into current perception");
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.insert(canonical);
    driver.step(&mut w, inputs).unwrap();
    for _ in 0..120 {
        if w.read_resource::<BuffStore>().next_attack_bonus(owner) == Fixed64::ZERO {break;}
        driver.step(&mut w, []).unwrap();
    }
    assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(owner), Fixed64::ZERO,
        "only the actual normal projectile launch consumes the owner's bonus");
    assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(other), Fixed64::from_i32(1000));
}

#[test]
fn incoming_damage_observation_carry_60hz_uses_committed_bonus_and_formal_input() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let caster=hero_pair(&w)[0];let creeps=lane_creeps(&w,2);
    let (immune,killable)=(creeps[0],creeps[1]);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    {let mut poses=w.write_storage::<Pos>();poses.get_mut(caster).unwrap().0=origin;
        for (e,x) in [(immune,50),(killable,100)] {
            poses.get_mut(e).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO);
        }}
    {let mut hp=w.write_storage::<CProperty>();hp.get_mut(immune).unwrap().hp=Fixed64::from_i32(1);
        hp.get_mut(killable).unwrap().hp=Fixed64::from_i32(20);}
    w.write_resource::<BuffStore>().add(immune,"damage_immunity_fixture",Fixed64::from_i32(10),
        serde_json::json!({"damage_taken_bonus":-1024}));
    let first=driver.step(&mut w,[]).unwrap();
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Carry,lane:0,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:None,item_builds:Vec::new()};
    run_committed_visibility_wave_b(&mut w,first.tick,0);
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==killable.id()));
    // A private mutation cannot change the committed observation mid-think.
    w.write_resource::<BuffStore>().remove(immune,"damage_immunity_fixture");
    assert_eq!(role_bot_inputs(&w,&bots).unwrap(),inputs);
    let result=driver.step(&mut w,inputs).unwrap();
    assert!(matches!(w.read_storage::<HeroCommandQueue>().get(caster).unwrap().active,
        Some(HeroCommand::AttackTarget {target,..}) if target==killable));
    assert!(result.facts.iter().any(|f|matches!(f.fact,ObservableFact::CommittedIncomingDamage {source,bonus_raw:0}
        if source==canonical_entity_id(immune))));
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let after=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&after[0].1.action,Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==immune.id()));
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current
        .remove(&canonical_entity_id(immune));
    let hidden=role_bot_inputs(&w,&bots).unwrap();
    assert!(!hidden.iter().any(|(_,input)|matches!(&input.action,Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==immune.id())));
}

#[test]
fn damage_packet_settlement_formal_60hz_skill_uses_authority_incoming_bonus() {
    for (bonus, expected_damage) in [(512,120),(-512,40),(-2048,0)] {
        let (mut w,mut driver)=world(SingleLaneConfig {
            wave_interval:Fixed64::from_i32(10_000),..fast_config()
        },SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();
        let caster=hero_pair(&w)[0];let target=lane_creeps(&w,2)[0];
        let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
        w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
        w.write_storage::<Pos>().get_mut(target).unwrap().0=origin+
            omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
        w.write_storage::<CProperty>().get_mut(target).unwrap().hp=Fixed64::from_i32(300);
        w.write_resource::<BuffStore>().add(target,"incoming_fixture",Fixed64::from_i32(10),
            serde_json::json!({"damage_taken_bonus":bonus}));
        driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index:0,target_entity:Some(target.id()),target_pos:None,
        }))})]).unwrap();
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,
            Fixed64::from_i32(300-expected_damage),"incoming bonus {bonus}");
        assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("lumen_bolt"));
    }
}

#[test]
fn lane_last_hit_gold_formal_60hz_skill_pays_only_killer_and_survives_respawn() {
    let (mut w,mut driver)=world(SingleLaneConfig {lane_creep_gold:20,
        additional_players:vec![extra_player(3,1)],wave_interval:Fixed64::from_i32(10_000),
        ..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,_]=hero_pair(&w);
    let creep=lane_creeps(&w,2)[0];
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=source;
    w.write_storage::<Pos>().get_mut(creep).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
    {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(creep).unwrap();
        p.hp=Fixed64::from_i32(80);p.def_magic=Fixed64::ZERO;}
    w.write_storage::<Bounty>().insert(creep,Bounty {gold:999,exp:999}).unwrap();
    driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:0,target_entity:Some(creep.id()),target_pos:None,
    }))})]).unwrap();
    assert_eq!(w.read_storage::<Gold>().get(caster).unwrap().0,20,"formal generated skill last hit, not legacy bounty");
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("lumen_bolt"));
    for slot in &w.read_resource::<MobaMatch>().heroes {
        if slot.player_id!=1 {assert_eq!(w.read_storage::<Gold>().get(slot.entity.unwrap()).unwrap().0,0,"no shared gold");}
        assert_eq!(slot.kills,0,"creep gold must not count as hero kill");
    }
    w.write_resource::<Vec<Outcome>>().push(Outcome::Death {pos:source,ent:caster});
    process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();w.maintain();
    for _ in 0..125 {driver.step(&mut w,[]).unwrap();}
    let new=w.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    assert_ne!(canonical_entity_id(new),canonical_entity_id(caster));
    assert_eq!(w.read_storage::<Gold>().get(new).unwrap().0,20,"last-hit income persists across new hero life");
}

#[test]
fn lane_last_hit_gold_retires_anonymous_npc_and_duplicate_lethals_and_bounds_reward() {
    for case in 0..8 {
        let (mut w,mut driver)=world(SingleLaneConfig {lane_creep_gold:20,..fast_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let [hero,enemy]=hero_pair(&w);let creep=lane_creeps(&w,2)[0];
        let first=match case {
            0=>lethal(hero,creep),
            1=>Outcome::ScriptDirectDamage {target:creep,amount:Fixed64::from_i32(10_000)},
            2=>lethal(w.read_resource::<MobaMatch>().towers[0].unwrap(),creep),
            3=>lethal(enemy,creep),
            4=>Outcome::Death {ent:creep,pos:omoba_sim::Vec2::ZERO},
            _=>lethal(hero,creep),
        };
        match case {
            5=>w.write_resource::<GamePause>().is_paused=true,
            6=>w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Finished {winner:Some(0),tick:1},
            7=>w.write_storage::<Gold>().get_mut(hero).unwrap().0=i32::MAX-1,
            _=>{},
        }
        w.write_resource::<Vec<Outcome>>().push(first);
        if case!=4 {w.write_resource::<Vec<Outcome>>().extend([
            Outcome::Heal {pos:omoba_sim::Vec2::ZERO,target:creep,amount:Fixed64::from_i32(500)},
            lethal(hero,creep),lethal(hero,creep),
        ]);}
        process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
        assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0,match case {0=>20,7=>i32::MAX,_=>0},"case {case}");
    }
    let pool=StateInitializer::create_thread_pool();let mut w=StateInitializer::setup_campaign_ecs_world(&pool);
    assert!(setup_single_lane_match(&mut w,SingleLaneConfig {lane_creep_gold:1_000_001,..fast_config()}).is_err());
}

#[test]
fn single_lane_creep_xp_60hz_is_shared_once_from_npc_kill_without_legacy_bounty() {
    let run = || {
        let (mut world, mut driver) = world(SingleLaneConfig {
            additional_players: vec![extra_player(3,1)], wave_interval: Fixed64::from_i32(10_000),
            ..fast_config()
        }, SimulationTickProfile::Production60Hz);
        driver.step(&mut world, []).unwrap();
        let heroes: Vec<_> = world.read_resource::<MobaMatch>().heroes.iter().map(|s| s.entity.unwrap()).collect();
        let creeps = lane_creeps(&world,2);
        let origin = omoba_sim::Vec2::new(Fixed64::from_i32(3000),Fixed64::from_i32(7000));
        for entity in &creeps { world.write_storage::<Pos>().get_mut(*entity).unwrap().0 = origin; }
        for hero in &heroes { world.write_storage::<Pos>().get_mut(*hero).unwrap().0 = origin; }
        // Inclusive boundary, two recipients: floor(25 / 2), no tie-order bonus.
        world.write_storage::<Pos>().get_mut(heroes[0]).unwrap().0.x += Fixed64::from_i32(1200);
        let source = world.read_resource::<MobaMatch>().towers[0].unwrap();
        world.write_storage::<Bounty>().insert(creeps[0], Bounty {gold:999,exp:999}).unwrap();
        world.write_resource::<Vec<Outcome>>().extend([lethal(source,creeps[0]),lethal(source,creeps[0])]);
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        record_moba_death(&mut world,creeps[0]);
        assert_eq!(world.read_storage::<Hero>().get(heroes[0]).unwrap().experience,12);
        assert_eq!(world.read_storage::<Hero>().get(heroes[2]).unwrap().experience,12);
        assert_eq!(world.read_storage::<Hero>().get(heroes[1]).unwrap().experience,0,"same team as dead creep");
        assert_eq!(world.read_storage::<Gold>().get(heroes[0]).unwrap().0,0,"legacy bounty bypassed");
        // Exercise the next production tick for the second creep death.
        driver.step(&mut world,[]).unwrap();
        world.write_storage::<Pos>().get_mut(creeps[1]).unwrap().0 = origin;
        // Pending-deletion HP0 must neither receive XP nor dilute the live share.
        world.write_storage::<CProperty>().get_mut(heroes[2]).unwrap().hp = Fixed64::ZERO;
        world.write_resource::<Vec<Outcome>>().push(lethal(source,creeps[1]));
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        assert_eq!(world.read_storage::<Hero>().get(heroes[0]).unwrap().experience,37);
        assert_eq!(world.read_storage::<Hero>().get(heroes[2]).unwrap().experience,12);
        let enemy_source=world.read_resource::<MobaMatch>().towers[1].unwrap();
        world.write_resource::<Vec<Outcome>>().push(lethal(enemy_source,heroes[0]));
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
        world.maintain();
        for _ in 0..125 {driver.step(&mut world,[]).unwrap();}
        let respawned=world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
        assert_ne!(canonical_entity_id(respawned),canonical_entity_id(heroes[0]));
        assert_eq!(world.read_storage::<Hero>().get(respawned).unwrap().experience,37,"lane XP survives respawn");
        single_lane_replay_digest(&world)
    };
    assert_eq!(run(),run(),"shared lane XP 60Hz replay");
}

#[test]
fn single_lane_creep_xp_rejects_inactive_dead_outside_and_untracked_units() {
    for case in 0..9 {
        let (mut world, mut driver) = world(fast_config(),SimulationTickProfile::Production60Hz);
        driver.step(&mut world, []).unwrap();
        let hero = hero_pair(&world)[0];
        let creep = lane_creeps(&world,2)[0];
        let origin = omoba_sim::Vec2::new(Fixed64::from_i32(3000),Fixed64::from_i32(7000));
        world.write_storage::<Pos>().get_mut(hero).unwrap().0 = origin;
        world.write_storage::<Pos>().get_mut(creep).unwrap().0 = origin;
        world.write_storage::<CProperty>().get_mut(creep).unwrap().hp = Fixed64::ZERO;
        match case {
            0 => world.write_resource::<GamePause>().is_paused = true,
            1 => world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Warmup,
            2 => world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Finished {winner:Some(0),tick:1},
            3 => {world.write_storage::<Pos>().remove(creep);},
            4 => world.write_storage::<CProperty>().get_mut(creep).unwrap().hp = Fixed64::ONE,
            5 => world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::ZERO,
            6 => world.write_storage::<Pos>().get_mut(hero).unwrap().0.x += Fixed64::from_i32(1200) + Fixed64::from_raw(1),
            7 => {
                let own_creep = lane_creeps(&world,1)[0];
                world.write_storage::<Pos>().get_mut(own_creep).unwrap().0 = origin;
                world.write_storage::<CProperty>().get_mut(own_creep).unwrap().hp = Fixed64::ZERO;
                record_moba_death(&mut world,own_creep);
                assert_eq!(world.read_storage::<Hero>().get(hero).unwrap().experience,0);
                continue;
            },
            8 => {record_moba_death(&mut world,creep); world.write_storage::<Hero>().get_mut(hero).unwrap().experience=0;},
            _ => unreachable!(),
        }
        record_moba_death(&mut world,creep);
        assert_eq!(world.read_storage::<Hero>().get(hero).unwrap().experience,0,"case {case}");
    }
}

#[test]
fn single_lane_creep_xp_integer_multilevel_zero_share_and_rules_validation() {
    for (reward,count,expected) in [(365,1,(4,1,3)),(1,2,(1,0,0)),(0,1,(1,0,0))] {
        let (mut world, mut driver) = world(SingleLaneConfig {lane_creep_xp:reward,
            additional_players:if count==2 {vec![extra_player(3,1)]} else {vec![]},..fast_config()},
            SimulationTickProfile::Production60Hz);
        driver.step(&mut world,[]).unwrap();
        let creep = lane_creeps(&world,2)[0];
        let origin = omoba_sim::Vec2::new(Fixed64::from_i32(3000),Fixed64::from_i32(7000));
        let heroes: Vec<_> = world.read_resource::<MobaMatch>().heroes.iter().filter(|s| s.side==0).map(|s| s.entity.unwrap()).collect();
        for hero in &heroes {world.write_storage::<Pos>().get_mut(*hero).unwrap().0=origin;}
        world.write_storage::<Pos>().get_mut(creep).unwrap().0=origin;
        world.write_storage::<CProperty>().get_mut(creep).unwrap().hp=Fixed64::ZERO;
        record_moba_death(&mut world,creep);
        for hero in heroes {
            let h=world.read_storage::<Hero>(); let h=h.get(hero).unwrap();
            assert_eq!((h.level,h.experience,h.skill_points),expected);
        }
    }
    for (xp,radius) in [(1_000_001,1200),(25,0),(25,10_001)] {
        let pool=StateInitializer::create_thread_pool();
        let mut w=StateInitializer::setup_campaign_ecs_world(&pool);
        assert!(setup_single_lane_match(&mut w,SingleLaneConfig {lane_creep_xp:xp,lane_xp_radius:radius,..fast_config()}).is_err());
        assert!(w.try_fetch::<MobaMatch>().is_none());
    }
}

#[test]
fn single_lane_real_assist_pays_each_helper_once_and_preserves_dead_helper_gold() {
    let run = || {
        let (mut world, mut driver) = world(SingleLaneConfig {
            additional_players: vec![extra_player(3, 1), extra_player(4, 1)],
            wave_interval: Fixed64::from_i32(10_000), ..fast_config()
        }, SimulationTickProfile::Production60Hz);
        driver.step(&mut world, []).unwrap();
        let entities: Vec<_> = world.read_resource::<MobaMatch>().heroes.iter().map(|s| s.entity.unwrap()).collect();
        let [killer, victim, live_helper, dead_helper] = entities.as_slice() else { panic!("four heroes"); };
        for source in [*live_helper, *live_helper, *dead_helper] {
            world.write_resource::<Vec<Outcome>>().push(Outcome::Damage {
                pos: omoba_sim::Vec2::ZERO, phys: Fixed64::ZERO, magi: Fixed64::ZERO, real: Fixed64::ONE,
                source, target: *victim, damage_profile: 0, predeclared: false,
            });
            process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        }
        world.write_resource::<Vec<Outcome>>().push(Outcome::Death { ent: *dead_helper, pos: omoba_sim::Vec2::ZERO });
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        world.maintain();
        world.write_resource::<Vec<Outcome>>().extend([lethal(*killer, *victim), lethal(*killer, *victim)]);
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        world.maintain();
        let state = world.read_resource::<MobaMatch>();
        assert_eq!(state.heroes.iter().map(|s| (s.kills, s.assists, s.deaths)).collect::<Vec<_>>(),
            vec![(1,0,0),(0,0,1),(0,1,0),(0,1,1)]);
        assert_eq!(world.read_storage::<Gold>().get(*killer).unwrap().0, 300);
        assert_eq!(world.read_storage::<Gold>().get(*live_helper).unwrap().0, 100);
        let heroes = world.read_storage::<Hero>();
        assert_eq!((heroes.get(*killer).unwrap().level, heroes.get(*killer).unwrap().experience,
            heroes.get(*killer).unwrap().skill_points), (2, 0, 1));
        assert_eq!(heroes.get(*live_helper).unwrap().experience, 50);
        drop(heroes);
        drop(state);
        for _ in 0..125 { driver.step(&mut world, []).unwrap(); }
        let state = world.read_resource::<MobaMatch>();
        for index in [1, 3] { assert_eq!(state.heroes[index].respawns, 1); }
        assert_eq!(world.read_storage::<Gold>().get(state.heroes[3].entity.unwrap()).unwrap().0, 100);
        assert_eq!(state.heroes[0].kills, 1);
        assert_eq!(state.heroes[3].assists, 1);
        assert_eq!(world.read_storage::<Hero>().get(state.heroes[3].entity.unwrap()).unwrap().experience, 50);
        drop(state);
        single_lane_replay_digest(&world)
    };
    assert_eq!(run(), run(), "60Hz assist replay");
}

#[test]
fn single_lane_roster_validation_is_atomic_and_accepts_five_per_team() {
    let unknown = SingleLaneConfig { additional_players: vec![SingleLanePlayerConfig {
        hero:"missing_hero".into(), ..extra_player(3,1)
    }], ..fast_config() };
    let mut scripts = ScriptRegistry::new();
    scripts.insert_manifest(crate::get_manifest());
    assert!(validate_single_lane_scripts(&unknown, &scripts).unwrap_err().to_string().contains("missing_hero"));
    for additional in [vec![extra_player(0,1)], vec![extra_player(1,1)], vec![extra_player(3,9)],
        vec![extra_player(3,1),extra_player(3,2)],
        (3..=7).map(|p| extra_player(p,1)).collect(),
        vec![SingleLanePlayerConfig {hero:"missing_hero".into(), ..extra_player(3,1)}]] {
        let pool = StateInitializer::create_thread_pool();
        let mut w = StateInitializer::setup_campaign_ecs_world(&pool);
        assert!(setup_single_lane_match(&mut w, SingleLaneConfig {additional_players:additional, ..fast_config()}).is_err());
        assert!(w.try_fetch::<MobaMatch>().is_none());
        assert_eq!((&w.entities(), &w.read_storage::<Unit>()).join().count(), 0);
    }
    let (w, _) = world(SingleLaneConfig {
        additional_players:(3..=10).map(|p| extra_player(p, if p % 2 == 1 {1} else {2})).collect(), ..fast_config()
    }, SimulationTickProfile::Production60Hz);
    let state = w.read_resource::<MobaMatch>();
    assert_eq!(state.heroes.len(), 10);
    for slot in &state.heroes {
        let ent = slot.entity.unwrap();
        assert_eq!(w.read_storage::<PlayerOwner>().get(ent).unwrap().player_id, slot.player_id);
        assert_eq!(w.read_storage::<Faction>().get(ent).unwrap().team_id as u32, state.config.teams[slot.side]);
    }
}

#[test]
fn single_lane_zero_hp_entity_cannot_execute_generic_cast_or_start_cooldown() {
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    driver.step(&mut world, []).unwrap();
    let [source, target] = hero_pair(&world);
    world.write_storage::<CProperty>().get_mut(target).unwrap().hp = Fixed64::ZERO;
    assert!(world.entities().is_alive(target), "pending deletion is not health");
    handle_ability_cast_from_input(&mut world, 0, None, Some(target.id()), 1).unwrap();
    let mut registry = ScriptRegistry::new();
    registry.insert_manifest(crate::get_manifest());
    run_script_dispatch(&mut world, &registry, 1, Fixed64::from_raw(17));
    process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
    assert!(!world.read_storage::<Hero>().get(source).unwrap().is_on_cooldown("lumen_bolt"));
    assert_eq!(world.read_resource::<MobaMatch>().heroes[0].kills, 0);
}

#[test]
fn single_lane_xp_level_growth_preserves_hp_and_item_bonus_until_respawn() {
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    driver.step(&mut world, []).unwrap();
    let [source, target] = hero_pair(&world);
    // Stand-in already-applied item bonus: level delta must not rebuild base stats.
    world.write_storage::<CProperty>().get_mut(source).unwrap().mhp += Fixed64::from_i32(200);
    let hp = world.read_storage::<CProperty>().get(source).unwrap().hp;
    let mhp = world.read_storage::<CProperty>().get(source).unwrap().mhp;
    let atk = world.read_storage::<TAttack>().get(source).unwrap().atk_physic.clone().val();
    let growth = world.read_storage::<Hero>().get(source).unwrap().level_growth.clone();
    world.write_resource::<Vec<Outcome>>().extend([lethal(source,target),lethal(source,target)]);
    process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
    assert_eq!(world.read_storage::<CProperty>().get(source).unwrap().hp, hp, "level up is not a heal");
    assert_eq!(world.read_storage::<CProperty>().get(source).unwrap().mhp, mhp + growth.hp_per_level);
    assert_eq!(world.read_storage::<TAttack>().get(source).unwrap().atk_physic.clone().val(), atk + growth.damage_per_level);
    let hero = world.read_storage::<Hero>();
    assert_eq!((hero.get(source).unwrap().level,hero.get(source).unwrap().experience,hero.get(source).unwrap().skill_points),(2,0,1));
}

#[test]
fn single_lane_three_heroes_60hz_filtered_recall_assist_and_respawn_are_player_scoped() {
    use std::collections::BTreeSet;
    use prost::Message;
    use omoba_core::runtime::native::economy_projection::OwnerEconomyState;
    let config = SingleLaneConfig { additional_players:vec![extra_player(3,1)],
        wave_interval:Fixed64::from_i32(10_000), ..fast_config() };
    let (mut authority, mut driver) = world(config, SimulationTickProfile::Production60Hz);
    let (a,b,c) = {
        let state = authority.read_resource::<MobaMatch>();
        (state.heroes[0].entity.unwrap(),state.heroes[1].entity.unwrap(),state.heroes[2].entity.unwrap())
    };
    for (index,entity) in [a,b,c].into_iter().enumerate() {
        authority.write_storage::<Pos>().get_mut(entity).unwrap().0 = omoba_sim::Vec2::new(
            Fixed64::from_i32(if index == 1 {1600} else {700 + index as i32 * 60}), Fixed64::from_i32(700));
    }
    let baseline = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, baseline);
    let allow = TeamProjectorConfig::default().component_allowlist;
    let starts = authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick()+1,60,fast_config().seed);
    let mut replicas: Vec<_> = starts.into_iter().map(|(team,start)| {
        let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),BTreeSet::new()).unwrap();
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start,allow.clone(),BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world,&stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();
        (team,replica,stepper)
    }).collect();
    let move_input = PlayerInput { action:Some(PlayerInputEnum::MoveTo(MoveTo {
        target:Some(Vec2I {x:950*1024,y:700*1024}),queued:false,
    })) };
    let origin_a = authority.read_storage::<Pos>().get(a).unwrap().0;
    for step in 0..640 {
        if step == 0 {
            authority.write_resource::<Vec<Outcome>>().push(Outcome::Damage {
                pos:omoba_sim::Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,real:Fixed64::ONE,
                source:c,target:b,damage_profile:0,predeclared:false,
            });
        }
        if step == 508 {
            authority.write_resource::<Vec<Outcome>>().push(Outcome::Death {ent:c,pos:omoba_sim::Vec2::ZERO});
        }
        if step == 510 { authority.write_resource::<Vec<Outcome>>().push(lethal(a,b)); }
        let inputs = match step {2 => vec![(3,recall())],3 => vec![(1,move_input.clone())],_=>vec![]};
        let accepted:Vec<_> = inputs.iter().map(|(player,input)| CanonicalAcceptedInput::from_authoritative_acceptance(
            1,*player,step+1,if *player == 3 {19} else {2},canonical_entity_id(if *player == 3 {c} else {a}),None,input.encode_to_vec()
        )).collect();
        let result = driver.step(&mut authority,inputs).unwrap();
        authority.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
        project_tick(&mut authority,result);
        let frames = authority.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected = authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick()+1,60,fast_config().seed);
        for (team,replica,stepper) in &mut replicas {
            let frame = frames[team].frame.clone();
            assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            let events = &frame.step.as_ref().unwrap().public_events;
            for event in events.iter().filter(|e| e.event_kind == FactKind::Hud as u32 && e.sanitized_payload.len()==20) {
                let p = &event.sanitized_payload;
                let metric = u64::from_le_bytes(p[4..12].try_into().unwrap());
                for namespace in [SINGLE_LANE_RECALL_ACTIVE_METRIC_ID,SINGLE_LANE_RECALL_REMAINING_METRIC_ID,SINGLE_LANE_RESPAWN_METRIC_ID] {
                    if let Some(player) = single_lane_metric_player(metric,namespace) {
                        assert_eq!(p[0..4],team.to_le_bytes());
                        assert!(if *team==1 {[1,3].contains(&player)} else {player==2},"enemy player metric");
                    }
                }
            }
            if step == 100 && *team == 1 {
                for (player,want_active) in [(1,false),(3,true)] {
                    let id = single_lane_player_metric(SINGLE_LANE_RECALL_ACTIVE_METRIC_ID,player);
                    let fact = events.iter().find(|e| e.event_kind==FactKind::Hud as u32 && e.sanitized_payload[4..12]==id.to_le_bytes()).unwrap();
                    assert_eq!(fact.sanitized_payload[12..20],i64::from(want_active).to_le_bytes(), "player {player} step {step}");
                }
                assert_ne!(authority.read_storage::<Pos>().get(a).unwrap().0,origin_a,"ally Recall must not stop player's Move");
            }
            if step >= 510 && step < 630 {
                let economies:Vec<_> = events.iter().filter(|e| e.event_kind==FactKind::OwnerEconomy as u32)
                    .map(|e| OwnerEconomyState::decode(&e.sanitized_payload).unwrap()).collect();
                if *team==1 {
                    assert!(economies.iter().any(|s| s.player_id==1 && s.economy.gold==300));
                    assert!(economies.iter().any(|s| s.player_id==3 && s.economy.gold==100));
                } else { assert!(economies.iter().all(|s| s.player_id==2 && s.economy.gold==0)); }
            }
            assert!(matches!(replica.apply_frame(frame,stepper).unwrap(),FrameApplyResult::Applied{..}),"step {step} team {team}");
            let rendered = replica.extract_filtered_render_snapshot();
            // Both teams receive public scores for all players, including dead
            // and hidden heroes, but never broaden private metric audiences.
            for (player,side,kills,deaths,assists) in [
                (1,1,u32::from(step>=510),0,0),
                (2,2,0,u32::from(step>=510),0),
                (3,1,0,u32::from(step>=508),u32::from(step>=510)),
            ] {
                for (ns,want) in [(SINGLE_LANE_SCORE_TEAM_METRIC_ID,side),
                    (SINGLE_LANE_SCORE_KILLS_METRIC_ID,kills),(SINGLE_LANE_SCORE_DEATHS_METRIC_ID,deaths),
                    (SINGLE_LANE_SCORE_ASSISTS_METRIC_ID,assists)] {
                    let id=single_lane_player_metric(ns,player);
                    let samples:Vec<_> = rendered.public_events.iter().filter(|e|e.event_kind==FactKind::Hud as u32
                        && e.sanitized_payload.len()==20 && e.sanitized_payload[4..12]==id.to_le_bytes()).collect();
                    assert_eq!(samples.len(),1);
                    assert_eq!(samples[0].sanitized_payload[..4],0u32.to_le_bytes());
                    assert_eq!(samples[0].sanitized_payload[12..20],i64::from(want).to_le_bytes());
                }
            }
            for (player,kills,deaths,assists) in if *team==1 {
                vec![(1,u32::from(step>=510),0,0),(3,0,u32::from(step>=508),u32::from(step>=510))]
            } else { vec![(2,0,u32::from(step>=510),0)] } {
                for (namespace,want) in [(SINGLE_LANE_KILLS_METRIC_ID,kills),(SINGLE_LANE_DEATHS_METRIC_ID,deaths),(SINGLE_LANE_ASSISTS_METRIC_ID,assists)] {
                    let id = single_lane_player_metric(namespace,player);
                    let fact = rendered.public_events.iter().find(|e| e.event_kind==FactKind::Hud as u32
                        && e.sanitized_payload.len()==20 && e.sanitized_payload[0..4]==team.to_le_bytes()
                        && e.sanitized_payload[4..12]==id.to_le_bytes()).expect("persistent score metric");
                    assert_eq!(fact.sanitized_payload[12..20],i64::from(want).to_le_bytes(),"score player {player} step {step}");
                }
            }
            let fresh = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team],allow.clone(),BTreeSet::new()).unwrap();
            assert_eq!(replica.canonical_team_hash(),fresh.canonical_team_hash(),"step {step} team {team}");
        }
        if step == 500 {
            let base = authority.read_resource::<MobaMatch>().bases[0].unwrap();
            assert_eq!(authority.read_storage::<Pos>().get(c).unwrap().0,authority.read_storage::<Pos>().get(base).unwrap().0);
        }
    }
    let state = authority.read_resource::<MobaMatch>();
    assert_eq!(state.heroes[0].kills,1);
    assert_eq!(state.heroes[2].assists,1);
    assert_eq!(state.heroes[1].respawns,1);
    assert_eq!(state.heroes[2].respawns,1);
    assert_eq!(authority.read_storage::<Gold>().get(state.heroes[2].entity.unwrap()).unwrap().0,100);
}

#[test]
fn single_lane_recall_control_60hz_rejects_start_and_cancels_damage_free_final_tick() {
    for status in ["stun","root","silence"] {
        let (mut w,mut driver)=world(SingleLaneConfig {
            wave_interval:Fixed64::from_i32(10_000),..fast_config()
        },SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();
        let [hero,_]=hero_pair(&w);
        let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
        w.write_storage::<Pos>().get_mut(hero).unwrap().0=origin;
        w.write_resource::<BuffStore>().add(hero,status,Fixed64::from_raw(128),serde_json::json!({}));
        driver.step(&mut w,[(1,recall())]).unwrap();
        assert!(!w.read_resource::<MobaMatch>().is_recalling(hero),"{status} blocks admission");
        for _ in 0..10 {driver.step(&mut w,[]).unwrap();}
        assert!(!w.read_resource::<BuffStore>().has(hero,status));
        assert!(!w.read_resource::<MobaMatch>().is_recalling(hero),"expiry never auto-starts recall");
        driver.step(&mut w,[(1,recall())]).unwrap();
        assert!(w.read_resource::<MobaMatch>().is_recalling(hero));
        let ticks=omoba_template_ids::MOBA_RECALL_CHANNEL_SECONDS*60;
        for _ in 1..ticks-1 {driver.step(&mut w,[]).unwrap();}
        assert_eq!(w.read_storage::<Pos>().get(hero).unwrap().0,origin);
        let hp=w.read_storage::<CProperty>().get(hero).unwrap().hp;
        // Use the production deferred AddBuff path, with no damage or movement:
        // this isolates control cancellation from the existing damage interrupt.
        w.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff {target:hero,
            buff_id:status.into(),duration:Fixed64::from_raw(128),payload:serde_json::json!({})});
        driver.step(&mut w,[]).unwrap();
        assert!(w.read_resource::<BuffStore>().has(hero,status));
        assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().hp,hp,"control is not damage");
        assert!(!w.read_resource::<MobaMatch>().is_recalling(hero),"{status} must win over completion");
        assert_eq!(w.read_storage::<Pos>().get(hero).unwrap().0,origin,"must not teleport on interrupted deadline");
        for _ in 0..10 {driver.step(&mut w,[]).unwrap();}
        assert!(!w.read_resource::<MobaMatch>().is_recalling(hero));
        driver.step(&mut w,[(1,recall())]).unwrap();
        assert!(w.read_resource::<MobaMatch>().is_recalling(hero),"fresh request after control expires");
        for _ in 1..ticks {driver.step(&mut w,[]).unwrap();}
        let base=w.read_resource::<MobaMatch>().bases[0].unwrap();
        assert_eq!(w.read_storage::<Pos>().get(hero).unwrap().0,w.read_storage::<Pos>().get(base).unwrap().0);
        assert!(!w.read_resource::<MobaMatch>().is_recalling(hero));
    }
}

#[test]
fn role_bot_recall_control_60hz_does_not_spam_blocked_sustain() {
    use omoba_core::runtime::native::moba_match::bots::*;
    for status in ["stun","root","silence"] {
        let (mut w,mut driver)=world(SingleLaneConfig {base_recovery_enabled:true,
            wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
        let result=driver.step(&mut w,[]).unwrap();
        let [hero,_]=hero_pair(&w);
        w.write_storage::<Pos>().get_mut(hero).unwrap().0=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
        w.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::from_i32(100);
        let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Mid,lane:1,escort_player_id:None}],
            think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),item_builds:Vec::new(),
            sustain:Some(BotSustainPolicy {recall_below_hp_per_mille:350,leave_base_at_hp_per_mille:850,threat_radius:1000,mana:None})};
        run_committed_visibility_wave_b(&mut w,result.tick,0);
        assert!(matches!(&role_bot_inputs(&w,&bots).unwrap()[0].1.action,Some(PlayerInputEnum::Recall(_))));
        w.write_resource::<BuffStore>().add(hero,status,Fixed64::from_raw(128),serde_json::json!({}));
        for _ in 0..4 {
            let inputs=role_bot_inputs(&w,&bots).unwrap();
            assert!(inputs.is_empty(),"{status} must not submit blocked recall");
            let result=driver.step(&mut w,inputs).unwrap();
            run_committed_visibility_wave_b(&mut w,result.tick,0);
            assert!(!w.read_resource::<MobaMatch>().is_recalling(hero));
        }
        for _ in 0..10 {
            let result=driver.step(&mut w,[]).unwrap();run_committed_visibility_wave_b(&mut w,result.tick,0);
        }
        let inputs=role_bot_inputs(&w,&bots).unwrap();
        assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::Recall(_))));
        driver.step(&mut w,inputs).unwrap();
        assert!(w.read_resource::<MobaMatch>().is_recalling(hero));
    }
}

#[test]
fn single_lane_recall_60hz_exact_time_pause_replay_and_no_free_heal() {
    let run = || {
        let (mut world, mut driver) = world(SingleLaneConfig { wave_interval: Fixed64::from_i32(10_000), ..fast_config() }, SimulationTickProfile::Production60Hz);
        driver.step(&mut world, []).unwrap();
        let hero = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
        let origin = omoba_sim::Vec2::new(Fixed64::from_i32(400), Fixed64::from_i32(400));
        world.write_storage::<Pos>().get_mut(hero).unwrap().0 = origin;
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(200);
        driver.step(&mut world, [(1, recall())]).unwrap();
        assert!(world.read_resource::<MobaMatch>().is_recalling(hero));
        let health = world.read_storage::<CProperty>().get(hero).unwrap().hp;
        world.write_resource::<GamePause>().is_paused = true;
        for _ in 0..120 { driver.step(&mut world, []).unwrap(); }
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0, origin);
        world.write_resource::<GamePause>().is_paused = false;
        let ticks = omoba_template_ids::MOBA_RECALL_CHANNEL_SECONDS * 60;
        for _ in 1..ticks - 1 { driver.step(&mut world, [(1, recall())]).unwrap(); }
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0, origin, "not before full channel");
        driver.step(&mut world, []).unwrap();
        let base = world.read_resource::<MobaMatch>().bases[0].unwrap();
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0, world.read_storage::<Pos>().get(base).unwrap().0);
        assert!(!world.read_resource::<MobaMatch>().is_recalling(hero));
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, health, "recall is not a heal");
        assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0, 0);
        single_lane_replay_digest(&world)
    };
    assert_eq!(run(), run());
}

#[test]
fn single_lane_recall_rejects_invalid_owner_and_cancels_on_command_damage_death() {
    for case in 0..8 {
        let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
        driver.step(&mut world, []).unwrap();
    let [hero, enemy] = hero_pair(&world);
        driver.step(&mut world, [(99, recall())]).unwrap();
        assert!(!world.read_resource::<MobaMatch>().is_recalling(hero));
        assert!(world.read_resource::<MobaMatch>().allows_player_input(1, &recall()));
        assert!(!world.read_resource::<MobaMatch>().allows_player_input(99, &recall()));
        driver.step(&mut world, [(1, recall())]).unwrap();
        assert!(world.read_resource::<MobaMatch>().is_recalling(hero));
        match case {
            0 => { driver.step(&mut world, [(1, PlayerInput { action: Some(PlayerInputEnum::MoveTo(MoveTo { target: Some(Vec2I { x: 400 * 1024, y: 400 * 1024 }), queued: false })) })]).unwrap(); }
            1 => { world.write_storage::<Pos>().get_mut(hero).unwrap().0.x += Fixed64::ONE; driver.step(&mut world, []).unwrap(); }
            2 | 3 => {
                let ticks = omoba_template_ids::MOBA_RECALL_CHANNEL_SECONDS * 60;
                for _ in 1..ticks - 1 { driver.step(&mut world, []).unwrap(); }
                world.write_resource::<Vec<Outcome>>().push(Outcome::Damage {
                    pos: omoba_sim::Vec2::ZERO, phys: Fixed64::ZERO, magi: Fixed64::ZERO,
                    real: Fixed64::from_i32(if case == 2 { 1 } else { 10000 }), source: enemy,
                    target: hero, damage_profile: 0, predeclared: false });
                driver.step(&mut world, []).unwrap();
            }
            4 => { world.write_resource::<MobaMatch>().bases[0] = None; driver.step(&mut world, []).unwrap(); }
            5 => { driver.step(&mut world, [(1, PlayerInput { action: Some(PlayerInputEnum::AttackMove(omoba_core::game_proto::AttackMove { target: Some(Vec2I { x: 400 * 1024, y: 0 }), queued: false })) })]).unwrap(); }
            6 => { driver.step(&mut world, [(1, PlayerInput { action: Some(PlayerInputEnum::AttackMove(omoba_core::game_proto::AttackMove { target: Some(Vec2I { x: 400 * 1024, y: 0 }), queued: false })) }), (1, recall())]).unwrap(); }
            7 => { world.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage { target: hero, amount: Fixed64::ONE }); driver.step(&mut world, []).unwrap(); }
            _ => unreachable!(),
        }
        assert!(!world.read_resource::<MobaMatch>().is_recalling(hero), "case {case}");
    }
}

fn lethal(source: specs::Entity, target: specs::Entity) -> Outcome {
    Outcome::Damage { pos: omoba_sim::Vec2::ZERO, phys: Fixed64::ZERO,
        magi: Fixed64::ZERO, real: Fixed64::from_i32(10_000), source, target,
        damage_profile: 0, predeclared: false }
}

#[test]
fn single_lane_60hz_hero_kill_is_once_and_survives_killer_respawn() {
    let run = || {
        let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
        driver.step(&mut world, []).unwrap();
    let [a, b] = hero_pair(&world);
        // Same batch overkill must not pay a second bounty.
        world.write_resource::<Vec<Outcome>>().extend([lethal(a, b),
            Outcome::Heal { pos: omoba_sim::Vec2::ZERO, target: b, amount: Fixed64::from_i32(500) },
            lethal(a, b), lethal(a, b)]);
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        world.maintain();
        assert_eq!(world.read_storage::<Gold>().get(a).unwrap().0, omoba_template_ids::MOBA_HERO_KILL_GOLD as i32);
        assert_eq!(world.read_resource::<MobaMatch>().heroes[0].kills, 1);
        assert_eq!(world.read_resource::<MobaMatch>().heroes[1].deaths, 1);
        let pos = world.read_storage::<Pos>().get(a).unwrap().0;
        world.write_resource::<Vec<Outcome>>().push(Outcome::Death { pos, ent: a });
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        world.maintain();
        for _ in 0..120 { driver.step(&mut world, []).unwrap(); }
        let state = world.read_resource::<MobaMatch>();
        let new = state.heroes[0].entity.unwrap();
        assert_ne!(new, a);
        assert_eq!(state.heroes[0].kills, 1);
        assert_eq!(world.read_storage::<Gold>().get(new).unwrap().0, omoba_template_ids::MOBA_HERO_KILL_GOLD as i32);
        let victim = state.heroes[1].entity.unwrap();
        drop(state);
        world.write_resource::<Vec<Outcome>>().push(lethal(new, victim));
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        world.maintain();
        assert_eq!(world.read_resource::<MobaMatch>().heroes[0].kills, 2, "new life pays once again");
        assert_eq!(world.read_storage::<Gold>().get(new).unwrap().0, 2 * omoba_template_ids::MOBA_HERO_KILL_GOLD as i32);
        single_lane_replay_digest(&world)
    };
    assert_eq!(run(), run(), "fixed 60Hz replay");
}

#[test]
fn single_lane_kill_credit_rejects_self_npc_inactive_and_saturates() {
    for case in 0..8 {
        let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
        driver.step(&mut world, []).unwrap();
    let [a, b] = hero_pair(&world);
        let (source, target) = match case {
            0 => (a, a),
            1 | 7 => (world.read_resource::<MobaMatch>().towers[0].unwrap(), b),
            2 => (a, world.read_resource::<MobaMatch>().towers[1].unwrap()),
            _ => (a, b),
        };
        match case {
            3 => world.write_resource::<GamePause>().is_paused = true,
            4 => world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Warmup,
            5 => world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Finished { winner: Some(0), tick: 1 },
            6 => world.write_storage::<Gold>().get_mut(a).unwrap().0 = i32::MAX - 1,
            _ => (),
        }
        world.write_resource::<Vec<Outcome>>().push(lethal(source, target));
        if case == 7 {
            world.write_resource::<Vec<Outcome>>().extend([
                Outcome::Heal { pos: omoba_sim::Vec2::ZERO, target: b, amount: Fixed64::from_i32(500) },
                lethal(a, b),
            ]);
        }
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        assert_eq!(world.read_resource::<MobaMatch>().heroes[0].kills, u32::from(case == 6), "case {case}");
        assert_eq!(world.read_storage::<Gold>().get(a).unwrap().0, if case == 6 { i32::MAX } else { 0 }, "case {case}");
        assert_eq!(world.read_storage::<Hero>().get(a).unwrap().level, if case == 6 {2} else {1}, "XP gate case {case}");
    }
}

#[test]
fn single_lane_full_filtered_match_lifecycle_60hz() {
    full_match_filtered_lifecycle(SimulationTickProfile::Production60Hz, 60, 18_400);
}

#[test]
fn single_lane_anonymous_lethal_then_heal_cannot_claim_kill_or_assist() {
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    driver.step(&mut world, []).unwrap();
    let [source, target] = hero_pair(&world);
    world.write_resource::<Vec<Outcome>>().extend([
        Outcome::ScriptDirectDamage { target, amount: Fixed64::from_i32(10000) },
        Outcome::ScriptHeal { target, amount: Fixed64::from_i32(500) },
        lethal(source, target),
    ]);
    process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
    let state = world.read_resource::<MobaMatch>();
    assert_eq!(state.heroes[0].kills, 0);
    assert!(state.heroes.iter().all(|s| s.assists == 0));
    assert_eq!(world.read_storage::<Gold>().get(source).unwrap().0, 0);
}

#[test]
fn single_lane_repeated_positive_damage_only_pays_killer_not_fake_assist() {
    let run = || {
        let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
        driver.step(&mut world, []).unwrap();
    let [source, target] = hero_pair(&world);
        for real in [Fixed64::ZERO, Fixed64::ONE, Fixed64::ONE, Fixed64::from_i32(10000)] {
            world.write_resource::<Vec<Outcome>>().push(Outcome::Damage { pos: omoba_sim::Vec2::ZERO,
                phys: Fixed64::ZERO, magi: Fixed64::ZERO, real, source, target, damage_profile: 0, predeclared: false });
            process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        }
        assert_eq!(world.read_resource::<MobaMatch>().heroes[0].kills, 1);
        assert!(world.read_resource::<MobaMatch>().heroes.iter().all(|s| s.assists == 0));
        assert_eq!(world.read_storage::<Gold>().get(source).unwrap().0, omoba_template_ids::MOBA_HERO_KILL_GOLD as i32);
        single_lane_replay_digest(&world)
    };
    assert_eq!(run(), run());
}

#[test]
fn single_lane_60hz_kill_reward_reaches_only_owner_filtered_economy() {
    use std::collections::BTreeSet;
    use prost::Message;
    use omoba_core::runtime::native::economy_projection::OwnerEconomyState;
    let (mut authority, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    let baseline = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, baseline);
    let allow = TeamProjectorConfig::default().component_allowlist;
    let starts = authority.write_resource::<TeamProjectionRuntime>()
        .build_team_bootstraps(driver.tick() + 1, 60, fast_config().seed);
    let mut replicas: Vec<_> = starts.into_iter().map(|(team, start)| {
        let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, allow.clone(), BTreeSet::new()).unwrap();
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world, &stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();
        (team, replica, stepper)
    }).collect();
    let [a, b] = hero_pair(&authority);
    authority.write_resource::<Vec<Outcome>>().extend([lethal(a, b), lethal(a, b)]);
    for step in 0..525 {
        let inputs = if step == 3 { vec![(1, PlayerInput { action: Some(PlayerInputEnum::MoveTo(MoveTo {
            target: Some(Vec2I { x: 900 * 1024, y: 700 * 1024 }), queued: false,
        })) })] } else if step == 10 { vec![(1, recall())] } else { vec![] };
        let accepted: Vec<_> = inputs.iter().map(|(player, input)| {
            CanonicalAcceptedInput::from_authoritative_acceptance(1, *player, step + 1,
                if step == 3 { 2 } else { 19 }, canonical_entity_id(a), None, input.encode_to_vec())
        }).collect();
        let result = driver.step(&mut authority, inputs).unwrap();
        authority.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
        project_tick(&mut authority, result);
        let frames = authority.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected = authority.write_resource::<TeamProjectionRuntime>()
            .build_team_bootstraps(driver.tick() + 1, 60, fast_config().seed);
        for (team, replica, stepper) in &mut replicas {
            let frame = frames[team].frame.clone();
            assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            let recall_events: Vec<_> = frame.step.as_ref().unwrap().public_events.iter()
                .filter(|event| event.event_kind == FactKind::Hud as u32
                    && event.sanitized_payload.len() == 20
                    && [SINGLE_LANE_RECALL_ACTIVE_METRIC_ID, SINGLE_LANE_RECALL_REMAINING_METRIC_ID]
                        .iter().any(|id| single_lane_metric_player(u64::from_le_bytes(event.sanitized_payload[4..12].try_into().unwrap()), *id).is_some()))
                .collect();
            assert!(!recall_events.is_empty(), "recall state absent");
            assert!(recall_events.iter().all(|event| event.sanitized_payload[0..4] == team.to_le_bytes()), "enemy recall leaked");
            assert!(frame.step.as_ref().unwrap().accepted_inputs.iter().all(|input| input.player_id == *team));
            let economy: Vec<_> = frame.step.as_ref().unwrap().public_events.iter()
                .filter(|event| event.event_kind == FactKind::OwnerEconomy as u32)
                .map(|event| OwnerEconomyState::decode(&event.sanitized_payload).unwrap()).collect();
            assert!(economy.iter().all(|state| state.player_id == *team), "enemy economy leaked");
            assert!(economy.iter().any(|state| state.economy.gold == if *team == 1 { omoba_template_ids::MOBA_HERO_KILL_GOLD as i32 } else { 0 }), "step {step} owner reward absent");
            assert!(matches!(replica.apply_frame(frame, stepper).unwrap(), FrameApplyResult::Applied { .. }));
            let fresh = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team], allow.clone(), BTreeSet::new()).unwrap();
            assert_eq!(replica.canonical_team_hash(), fresh.canonical_team_hash(), "step {step} team {team}");
        }
    }
    assert_eq!(authority.read_resource::<MobaMatch>().heroes[0].kills, 1);
    assert_eq!(authority.read_resource::<MobaMatch>().heroes[1].respawns, 1);
    let base = authority.read_resource::<MobaMatch>().bases[0].unwrap();
    assert_eq!(authority.read_storage::<Pos>().get(a).unwrap().0, authority.read_storage::<Pos>().get(base).unwrap().0, "committed recall teleport");
}

#[test]
fn single_lane_passive_income_uses_active_fixed_time_and_persists_through_death() {
    for (profile, fps) in [(SimulationTickProfile::Coarse15Hz, 15),
        (SimulationTickProfile::Production120Hz, 120)] {
        let config = SingleLaneConfig { warmup: Fixed64::from_i32(2),
            passive_gold_per_second: omoba_template_ids::MOBA_PASSIVE_GOLD_PER_SECOND,
            ..fast_config() };
        let (mut world, mut driver) = world(config, profile);
        let hero = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
        for _ in 0..fps * 2 { driver.step(&mut world, []).unwrap(); }
        assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0, 0, "warmup income");
        for _ in 0..fps { driver.step(&mut world, []).unwrap(); }
        let rate = omoba_template_ids::MOBA_PASSIVE_GOLD_PER_SECOND as i32;
        assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0, rate);
        let digest = single_lane_replay_digest(&world);
        finish_moba_match_tick(&mut world);
        assert_eq!(single_lane_replay_digest(&world), digest, "repeat commit must not pay twice");
        world.write_resource::<GamePause>().is_paused = true;
        for _ in 0..fps { driver.step(&mut world, []).unwrap(); }
        assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0, rate);
        world.write_resource::<GamePause>().is_paused = false;
        let pos = world.read_storage::<Pos>().get(hero).unwrap().0;
        world.write_resource::<Vec<Outcome>>().push(Outcome::Death { pos, ent: hero });
        process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
        world.maintain();
        for _ in 0..fps { driver.step(&mut world, []).unwrap(); }
        assert!(world.read_resource::<MobaMatch>().heroes[0].entity.is_none());
        for _ in 0..fps { driver.step(&mut world, []).unwrap(); }
        let new = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
        assert_eq!(world.read_storage::<Gold>().get(new).unwrap().0, rate * 3,
            "dead income must survive respawn exactly once");
        world.write_storage::<Gold>().get_mut(new).unwrap().0 = i32::MAX - 1;
        for _ in 0..fps { driver.step(&mut world, []).unwrap(); }
        assert_eq!(world.read_storage::<Gold>().get(new).unwrap().0, i32::MAX);
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Finished {
            winner: Some(0), tick: driver.tick() };
        let digest = single_lane_replay_digest(&world);
        for _ in 0..fps { driver.step(&mut world, []).unwrap(); }
        assert_eq!(single_lane_replay_digest(&world), digest, "terminal match freezes income");

        // Warmup ends inside a tick: do not pay for its pre-playing portion.
        let (mut crossed, mut driver) = world_with_profile_for_income_crossing(profile);
        for _ in 0..fps * 2 { driver.step(&mut crossed, []).unwrap(); }
        let hero = crossed.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
        assert_eq!(crossed.read_storage::<Gold>().get(hero).unwrap().0, rate);
    }
}

fn world_with_profile_for_income_crossing(profile: SimulationTickProfile) -> (World, SimulationDriver) {
    world(SingleLaneConfig { warmup: Fixed64::from_raw(1001),
        passive_gold_per_second: omoba_template_ids::MOBA_PASSIVE_GOLD_PER_SECOND,
        ..fast_config() }, profile)
}

#[test]
fn single_lane_lua_item_purchase_is_funded_by_income_not_injected_gold() {
    use omoba_core::game_proto::{ItemBuy, ItemSell};
    let (mut world, mut driver) = world(SingleLaneConfig {
        passive_gold_per_second: omoba_template_ids::MOBA_PASSIVE_GOLD_PER_SECOND,
        // Isolate the economic acceptance, not a full combat match fixture.
        wave_interval: Fixed64::from_i32(10_000), ..fast_config()
    }, SimulationTickProfile::Coarse15Hz);
    let hero = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    let buy = || PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_sword".into() })) };
    assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0, 0);
    driver.step(&mut world, [(1, buy())]).unwrap();
    assert!(world.read_storage::<Inventory>().get(hero).unwrap().find_item("moba_sword").is_none());
    for _ in 1..175 * 15 { driver.step(&mut world, []).unwrap(); }
    assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0, 350);
    driver.step(&mut world, [(1, buy())]).unwrap();
    assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0, 0);
    assert_eq!(world.read_storage::<Inventory>().get(hero).unwrap().find_item("moba_sword"), Some(0));
    driver.step(&mut world, [(1, PlayerInput { action: Some(PlayerInputEnum::ItemSell(ItemSell { item_slot: 0 })) })]).unwrap();
    assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0, 175);
    assert!(world.read_storage::<Inventory>().get(hero).unwrap().find_item("moba_sword").is_none());
}

#[test]
fn single_lane_shop_checks_both_bases_and_applies_reversible_equipment_stats() {
    use omoba_core::runtime::item::{ItemRegistry, ItemConfig, ItemBonus};
    use omoba_core::runtime::shop::{transact_moba_shop, ShopCommand, ShopError};
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Coarse15Hz);
    world.insert(ItemRegistry::from_configs(vec![ItemConfig {id: "moba_sword".into(), name: "Sword".into(),
        cost: 100, bonus: ItemBonus {atk: 10.0, hp: 100.0, ..ItemBonus::default()}, active: None,
        cooldown: 0.0, recipe: vec![]} ]));
    driver.step(&mut world, []).unwrap();
    let heroes = hero_pair(&world);
    for hero in heroes { world.write_storage::<Gold>().get_mut(hero).unwrap().0 = 500; }
    let before_hp = world.read_storage::<CProperty>().get(heroes[1]).unwrap().mhp;
    let before_atk = world.read_storage::<TAttack>().get(heroes[1]).unwrap().atk_physic.clone().val();
    world.write_resource::<GamePause>().is_paused = true;
    assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_sword".into())), Err(ShopError::MatchUnavailable));
    world.write_resource::<GamePause>().is_paused = false;
    assert_eq!(transact_moba_shop(&mut world, 99, ShopCommand::Buy("moba_sword".into())), Err(ShopError::UnknownPlayer));
    for player in [1, 2] { transact_moba_shop(&mut world, player, ShopCommand::Buy("moba_sword".into())).unwrap(); }
    driver.step(&mut world, []).unwrap();
    assert_eq!(world.read_storage::<Gold>().get(heroes[1]).unwrap().0, 400);
    assert_eq!(world.read_storage::<CProperty>().get(heroes[1]).unwrap().mhp, before_hp + Fixed64::from_i32(100));
    assert_eq!(world.read_storage::<TAttack>().get(heroes[1]).unwrap().atk_physic.clone().val(), before_atk + Fixed64::from_i32(10));
    let spawn = world.read_storage::<Pos>().get(heroes[1]).unwrap().0;
    world.write_storage::<Pos>().get_mut(heroes[1]).unwrap().0 = omoba_sim::Vec2::ZERO;
    assert_eq!(transact_moba_shop(&mut world, 2, ShopCommand::Sell(0)), Err(ShopError::OutsideShop));
    assert_eq!(world.read_storage::<Gold>().get(heroes[1]).unwrap().0, 400);
    world.write_storage::<Pos>().get_mut(heroes[1]).unwrap().0 = spawn;
    transact_moba_shop(&mut world, 2, ShopCommand::Sell(0)).unwrap();
    driver.step(&mut world, []).unwrap();
    assert_eq!(world.read_storage::<Gold>().get(heroes[1]).unwrap().0, 450);
    assert_eq!(world.read_storage::<CProperty>().get(heroes[1]).unwrap().mhp, before_hp);
    assert_eq!(world.read_storage::<TAttack>().get(heroes[1]).unwrap().atk_physic.clone().val(), before_atk);
    world.write_storage::<CProperty>().get_mut(heroes[1]).unwrap().hp = Fixed64::ZERO;
    assert_eq!(transact_moba_shop(&mut world, 2, ShopCommand::Buy("moba_sword".into())), Err(ShopError::HeroUnavailable));
    for phase in [MobaMatchPhase::Warmup, MobaMatchPhase::Finished { winner: Some(0), tick: driver.tick() }] {
        world.write_resource::<MobaMatch>().phase = phase;
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Sell(0)), Err(ShopError::MatchUnavailable));
        assert_eq!(world.read_storage::<Gold>().get(heroes[0]).unwrap().0, 400);
    }
}

#[test]
fn single_lane_formal_shop_inputs_are_ordered_atomic_and_replayable() {
    use omoba_core::game_proto::{ItemBuy, ItemSell};
    fn fixture() -> (World, SimulationDriver) {
        let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Coarse15Hz);
        world.insert(ItemRegistry::from_configs(vec![ItemConfig {
            id: "moba_sword".into(), name: "Sword".into(), cost: 100,
            bonus: ItemBonus { atk: 10.0, ..ItemBonus::default() },
            active: None, cooldown: 0.0, recipe: vec![],
        }]));
        driver.step(&mut world, []).unwrap();
    let heroes = hero_pair(&world);
        for hero in heroes { world.write_storage::<Gold>().get_mut(hero).unwrap().0 = 200; }
        (world, driver)
    }
    fn buy(id: &str) -> PlayerInput {
        PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: id.into() })) }
    }
    fn sell(slot: u32) -> PlayerInput {
        PlayerInput { action: Some(PlayerInputEnum::ItemSell(ItemSell { item_slot: slot })) }
    }
    let (mut authority, mut driver) = fixture();
    let (mut replay, mut replay_driver) = fixture();
    let recording = vec![
        vec![(1, buy("moba_sword")), (1, sell(0)), (2, sell(0)), (2, buy("moba_sword"))],
        vec![(1, buy("missing")), (2, sell(u32::MAX)), (99, buy("moba_sword"))],
        vec![(1, buy("moba_sword")), (1, buy("moba_sword")), (2, sell(0))],
        vec![],
    ];
    for (index, inputs) in recording.into_iter().enumerate() {
        driver.step(&mut authority, inputs.clone()).unwrap();
        replay_driver.step(&mut replay, inputs).unwrap();
        assert_eq!(single_lane_replay_digest(&authority), single_lane_replay_digest(&replay), "shop replay tick {index}");
        assert!(authority.read_resource::<PendingItemUseQueue>().requests.is_empty());
    let heroes = hero_pair(&authority);
        let balances = heroes.map(|hero| authority.read_storage::<Gold>().get(hero).unwrap().0);
        assert_eq!(balances, if index < 2 { [150, 100] } else { [50, 150] });
        let inventories = authority.read_storage::<Inventory>();
        assert_eq!(inventories.get(heroes[0]).unwrap().find_item("moba_sword").is_some(), index >= 2);
        assert_eq!(inventories.get(heroes[1]).unwrap().find_item("moba_sword").is_some(), index < 2);
    }
    // Phase/pause checks must still be applied to formal commands, not only the direct API.
    authority.write_resource::<GamePause>().is_paused = true;
    driver.step(&mut authority, [(1, sell(0))]).unwrap();
    assert_eq!(authority.read_resource::<PendingItemUseQueue>().settlements.len(), 1);
    assert_eq!(authority.read_resource::<PendingItemUseQueue>().settlements[0].result,
        Err(omoba_core::runtime::shop::ShopError::MatchUnavailable));
    let hero = authority.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    assert_eq!(authority.read_storage::<Gold>().get(hero).unwrap().0, 50);
    assert_eq!(authority.read_storage::<Inventory>().get(hero).unwrap().find_item("moba_sword"), Some(0));
    authority.write_resource::<GamePause>().is_paused = false;
    authority.write_resource::<MobaMatch>().phase = MobaMatchPhase::Finished { winner: Some(0), tick: driver.tick() };
    driver.step(&mut authority, [(1, sell(0))]).unwrap();
    assert_eq!(authority.read_storage::<Gold>().get(hero).unwrap().0, 50);
    assert_eq!(authority.read_resource::<PendingItemUseQueue>().settlements[0].result,
        Err(omoba_core::runtime::shop::ShopError::MatchUnavailable));
    driver.step(&mut authority, []).unwrap();
    assert!(authority.read_resource::<PendingItemUseQueue>().settlements.is_empty(), "inactive tick leaked an old receipt");
}

#[test]
fn single_lane_equipment_and_balance_survive_respawn_without_double_bonus() {
    use omoba_core::runtime::item::{ItemRegistry, ItemConfig, ItemBonus};
    use omoba_core::runtime::shop::{transact_moba_shop, ShopCommand};
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Coarse15Hz);
    world.insert(ItemRegistry::from_configs(vec![ItemConfig {id: "moba_armor".into(), name: "Armor".into(),
        cost: 100, bonus: ItemBonus {hp: 100.0, armor: 5.0, ..ItemBonus::default()}, active: None,
        cooldown: 0.0, recipe: vec![]} ]));
    driver.step(&mut world, []).unwrap();
    let old = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    world.write_storage::<Gold>().get_mut(old).unwrap().0 = 200;
    transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())).unwrap();
    driver.step(&mut world, []).unwrap();
    let hp = world.read_storage::<CProperty>().get(old).unwrap().mhp;
    let armor = world.read_storage::<CProperty>().get(old).unwrap().def_physic;
    let pos = world.read_storage::<Pos>().get(old).unwrap().0;
    world.write_resource::<Vec<Outcome>>().push(Outcome::Death { pos, ent: old });
    for _ in 0..40 { driver.step(&mut world, []).unwrap(); }
    let new = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    assert_ne!(canonical_entity_id(new), canonical_entity_id(old));
    assert_eq!(world.read_storage::<Gold>().get(new).unwrap().0, 100);
    assert_eq!(world.read_storage::<Inventory>().get(new).unwrap().find_item("moba_armor"), Some(0));
    assert_eq!(world.read_storage::<CProperty>().get(new).unwrap().mhp, hp);
    assert_eq!(world.read_storage::<CProperty>().get(new).unwrap().def_physic, armor);
}

#[test]
fn single_lane_shop_warmup_and_dead_owner_produce_explicit_tick_local_rejections() {
    use omoba_core::runtime::shop::ShopError;
    let (mut authority, mut driver) = world(SingleLaneConfig {
        warmup: Fixed64::from_i32(2), ..fast_config()
    }, SimulationTickProfile::Coarse15Hz);
    let buy = || PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_sword".into() })) };
    driver.step(&mut authority, [(1, buy())]).unwrap();
    assert_eq!(authority.read_resource::<PendingItemUseQueue>().settlements[0].result, Err(ShopError::MatchUnavailable));
    driver.step(&mut authority, []).unwrap();
    assert!(authority.read_resource::<PendingItemUseQueue>().settlements.is_empty());
    authority.write_resource::<MobaMatch>().config.warmup = Fixed64::ZERO;
    let hero = authority.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    let pos = authority.read_storage::<Pos>().get(hero).unwrap().0;
    authority.write_resource::<Vec<Outcome>>().push(Outcome::Death { pos, ent: hero });
    driver.step(&mut authority, []).unwrap();
    assert!(authority.read_resource::<MobaMatch>().heroes[0].entity.is_none());
    let result = driver.step(&mut authority, [(1, buy()), (99, buy())]).unwrap();
    assert_eq!(authority.read_resource::<PendingItemUseQueue>().settlements.len(), 1);
    assert_eq!(authority.read_resource::<PendingItemUseQueue>().settlements[0].result, Err(ShopError::HeroUnavailable));
    let owned = result.facts.iter().rev().find_map(|fact| match &fact.fact {
        ObservableFact::OwnerEconomy { state, .. } if state.player_id == 1 => Some(state), _ => None,
    }).unwrap();
    assert_eq!(owned.economy.gold, 0);
    assert_eq!(owned.economy.item_ids, [0; 6]);
    assert!(!owned.shop_available);
}

#[test]
fn single_lane_lua_generated_shop_recipe_uses_formal_input_and_total_price() {
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Coarse15Hz);
    driver.step(&mut world, []).unwrap();
    let hero = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    world.write_storage::<Gold>().get_mut(hero).unwrap().0 = 1000;
    let before_atk = world.read_storage::<TAttack>().get(hero).unwrap().atk_physic.clone().val();
    let buy = |id: &str| (1, PlayerInput {
        action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: id.into() })),
    });
    driver.step(&mut world, [buy("moba_sword"), buy("moba_sword"), buy("moba_greatsword")]).unwrap();
    assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0, 50);
    let inventory = world.read_storage::<Inventory>().get(hero).unwrap().clone();
    assert_eq!(inventory.find_item("moba_sword"), None);
    assert_eq!(inventory.find_item("moba_greatsword"), Some(0));
    assert_eq!(inventory.slots.iter().flatten().count(), 1);
    // Item phase follows item_tick: dirty equipment is applied next dispatch.
    driver.step(&mut world, []).unwrap();
    assert_eq!(world.read_storage::<TAttack>().get(hero).unwrap().atk_physic.clone().val(), before_atk + Fixed64::from_i32(25));
    driver.step(&mut world, [(1, PlayerInput {
        action: Some(PlayerInputEnum::ItemSell(ItemSell { item_slot: 0 })),
    })]).unwrap();
    assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0, 525);
    driver.step(&mut world, []).unwrap();
    assert_eq!(world.read_storage::<TAttack>().get(hero).unwrap().atk_physic.clone().val(), before_atk);
}

#[test]
fn single_lane_buy_use_sell_keeps_mixed_item_input_order() {
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Coarse15Hz);
    world.insert(ItemRegistry::from_configs(vec![ItemConfig {
        id: "moba_armor".into(), name: "Fixture".into(), cost: 100,
        bonus: ItemBonus::default(), active: Some(ActiveEffect::Shield { amount: 50.0, duration: 1.0 }),
        cooldown: 5.0, recipe: vec![],
    }]));
    driver.step(&mut world, []).unwrap();
    let hero = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    world.write_storage::<Gold>().get_mut(hero).unwrap().0 = 200;
    world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(100);
    driver.step(&mut world, [
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_armor".into() })) }),
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse { item_slot: 0, target_pos: None, target_entity: None })) }),
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemSell(ItemSell { item_slot: 0 })) }),
    ]).unwrap();
    assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0, 150);
    assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(100));
    assert_eq!(world.read_resource::<BuffStore>().shield_remaining(hero), Fixed64::from_i32(50));
    assert!(world.read_storage::<Inventory>().get(hero).unwrap().slots.iter().all(Option::is_none));
}

#[test]
fn item_sprint_60hz_uses_timed_buff_without_mutating_base_speed() {
    use omb_script_abi::stat_keys::StatKey;
    let (mut w, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    w.insert(ItemRegistry::from_configs(vec![ItemConfig {
        id: "moba_armor".into(), name: "Sprint fixture".into(), cost: 100,
        bonus: ItemBonus::default(),
        active: Some(ActiveEffect::SprintBuff { ms_bonus: 60.0, duration: 0.25 }),
        cooldown: 5.0, recipe: vec![],
    }]));
    driver.step(&mut w, []).unwrap();
    let hero = hero_pair(&w)[0];
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 200;
    let base = w.read_storage::<CProperty>().get(hero).unwrap().msd;
    let baseline = UnitStats::from_refs(&w.read_resource::<BuffStore>(), false).final_move_speed(base, hero);
    let use_item = || (1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse {
        item_slot: 0, target_pos: None, target_entity: None,
    })) });
    driver.step(&mut w, [
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_armor".into() })) }),
        use_item(),
    ]).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().msd, base);
    assert_eq!(w.read_resource::<BuffStore>().sum_add(hero, StatKey::MoveSpeedBonusBuff), Fixed64::from_i32(60));
    assert!(UnitStats::from_refs(&w.read_resource::<BuffStore>(), false).final_move_speed(base, hero) > baseline);
    assert!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining > 4.0);
    // Cooldown rejection must not refresh duration or stack another copy.
    let remaining = w.read_resource::<BuffStore>().get(hero, "item_sprint:moba_armor").unwrap().remaining;
    assert!(handle_item_use_from_input(&mut w, 0, None, None, 1).is_err());
    assert_eq!(w.read_resource::<BuffStore>().get(hero, "item_sprint:moba_armor").unwrap().remaining, remaining);
    driver.step(&mut w, []).unwrap();
    // Reset only the fixture cooldown to exercise a legal refresh before expiry.
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 0.0;
    driver.step(&mut w, [use_item()]).unwrap();
    assert_eq!(w.read_resource::<BuffStore>().sum_add(hero, StatKey::MoveSpeedBonusBuff), Fixed64::from_i32(60));
    for _ in 0..20 { driver.step(&mut w, []).unwrap(); }
    assert!(!w.read_resource::<BuffStore>().has(hero, "item_sprint:moba_armor"));
    assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().msd, base);
    assert_eq!(UnitStats::from_refs(&w.read_resource::<BuffStore>(), false).final_move_speed(base, hero), baseline);
}

#[test]
fn generated_active_shop_items_60hz_buy_use_real_catalog_without_effect_overrides() {
    use ability_runtime::ManaPool;
    use omb_script_abi::stat_keys::StatKey;
    let fixed = |value: f32| Fixed64::from_raw((f64::from(value) * 1024.0).round() as i64);
    for id in ["moba_guard_charm", "moba_stride_charm", "moba_mana_charm", "moba_ward_charm", "moba_strike_charm"] {
        let (mut w, mut driver) = world(SingleLaneConfig {
            mana_enabled: true, base_recovery_enabled: false, ..fast_config()
        }, SimulationTickProfile::Production60Hz);
        // Use the production-generated registry installed by match setup.
        // No replacement registry, item instance or effect configuration.
        let cfg = w.read_resource::<ItemRegistry>().get(id).expect("Lua authored active item");
        driver.step(&mut w, []).unwrap();
        let [hero, other] = hero_pair(&w);
        w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 2000;
        w.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(100);
        let base_speed = w.read_storage::<CProperty>().get(hero).unwrap().msd;
        let capacity = w.read_storage::<Hero>().get(hero).unwrap().moba_mana_capacity().unwrap();
        w.write_storage::<Hero>().get_mut(hero).unwrap().mana_pool = Some(ManaPool::new(Fixed64::from_i32(10), capacity).unwrap());
        let other_mana = w.read_storage::<Hero>().get(other).unwrap().mana_pool.clone();
        driver.step(&mut w, [(1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: id.into() })) })]).unwrap();
        assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0, 2000 - cfg.cost);
        assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().item_id, id);
        driver.step(&mut w, [(1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse {
            item_slot: 0, target_pos: None, target_entity: None,
        })) })]).unwrap();
        assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(100), "shield is not healing");
        assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().msd, base_speed);
        assert_eq!(w.read_storage::<Hero>().get(other).unwrap().mana_pool, other_mana);
        assert_eq!(w.read_resource::<BuffStore>().shield_remaining(other), Fixed64::ZERO);
        assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(other), Fixed64::ZERO);
        assert_eq!(w.read_resource::<BuffStore>().sum_add(other, StatKey::MoveSpeedBonusBuff), Fixed64::ZERO);
        match cfg.active.as_ref().unwrap() {
            ActiveEffect::Shield { amount, .. } => assert_eq!(w.read_resource::<BuffStore>().shield_remaining(hero), fixed(*amount)),
            ActiveEffect::SprintBuff { ms_bonus, .. } => assert_eq!(w.read_resource::<BuffStore>().sum_add(hero, StatKey::MoveSpeedBonusBuff), fixed(*ms_bonus)),
            ActiveEffect::RestoreMana { amount } => {
                // Production ticks also regenerate mana. Verify the item's exact
                // authority fact rather than pretending regeneration was disabled.
                let events = w.write_resource::<ScriptVisualEventQueue>().drain();
                let gained: Vec<_> = events.iter().filter(|event|
                    event.kind == ScriptVisualEventKind::ManaGained && event.primary == hero)
                    .map(|event| event.amount).collect();
                assert_eq!(gained, vec![fixed(*amount)]);
                let current = w.read_storage::<Hero>().get(hero).unwrap().mana_pool.as_ref().unwrap().current();
                assert!(current >= Fixed64::from_i32(10) + fixed(*amount) && current <= capacity);
            },
            ActiveEffect::DamageReduce { percent, .. } => assert_eq!(w.read_resource::<BuffStore>().sum_add(hero, StatKey::DamageTakenBonus), -fixed(*percent)),
            ActiveEffect::HeadshotNext { bonus_damage } => assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(hero), fixed(*bonus_damage)),
        }
        let cooldown = w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining;
        assert!(cooldown > cfg.cooldown - 1.0 && cooldown <= cfg.cooldown);
        assert!(handle_item_use_from_input(&mut w, 0, None, None, 1).is_err());
        assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, cooldown);
    }
}

#[test]
fn item_restore_mana_60hz_clamps_owner_pool_and_emits_actual_gain() {
    use ability_runtime::ManaPool;
    use omb_script_abi::stat_keys::StatKey;
    let (mut w, mut driver) = world(SingleLaneConfig {
        mana_enabled: true, base_recovery_enabled: false, ..fast_config()
    }, SimulationTickProfile::Production60Hz);
    w.insert(ItemRegistry::from_configs(vec![ItemConfig {
        id: "moba_armor".into(), name: "Mana fixture".into(), cost: 100,
        bonus: ItemBonus::default(), active: Some(ActiveEffect::RestoreMana { amount: 1_000_000.0 }),
        cooldown: 5.0, recipe: vec![],
    }]));
    driver.step(&mut w, []).unwrap();
    let [hero, other] = hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 200;
    driver.step(&mut w, [(1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_armor".into() })) })]).unwrap();
    let base = w.read_storage::<Hero>().get(hero).unwrap().moba_mana_capacity().unwrap();
    let other_before = w.read_storage::<Hero>().get(other).unwrap().mana_pool.clone().unwrap();
    w.write_storage::<Hero>().get_mut(hero).unwrap().mana_pool = Some(ManaPool::new(Fixed64::from_i32(10), base).unwrap());
    w.write_resource::<BuffStore>().add(hero, "fixture_mana_item_capacity", Fixed64::from_i32(10),
        serde_json::json!({ StatKey::ManaBonus.as_str(): 60 * 1024 }));
    w.write_resource::<ScriptEventQueue>().drain();
    w.write_resource::<ScriptVisualEventQueue>().drain();
    driver.step(&mut w, [(1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse {
        item_slot: 0, target_pos: None, target_entity: None,
    })) })]).unwrap();
    let maximum = base + Fixed64::from_i32(60);
    let pool = w.read_storage::<Hero>().get(hero).unwrap().mana_pool.clone().unwrap();
    assert_eq!(pool.current(), maximum);
    assert_eq!(pool.maximum(), maximum);
    assert_eq!(w.read_storage::<Hero>().get(other).unwrap().mana_pool.as_ref().unwrap(), &other_before);
    let events = w.write_resource::<ScriptVisualEventQueue>().drain();
    let gained: Vec<_> = events.iter().filter(|event|
        event.kind == ScriptVisualEventKind::ManaGained && event.primary == hero)
        .map(|event| event.amount).collect();
    assert_eq!(gained, vec![maximum - Fixed64::from_i32(10)]);
    assert!(handle_item_use_from_input(&mut w, 0, None, None, 1).is_err());
    assert_eq!(w.read_storage::<Hero>().get(hero).unwrap().mana_pool.as_ref().unwrap(), &pool);
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 0.0;
    assert!(handle_item_use_from_input(&mut w, 0, None, None, 1).is_ok());
    assert!(!w.write_resource::<ScriptEventQueue>().drain().iter().any(|event| matches!(event, ScriptEvent::ManaGained { .. })));
    assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 5.0);
}

#[test]
fn item_restore_mana_preflight_rejects_invalid_amount_pool_or_capacity_atomically() {
    use ability_runtime::ManaPool;
    use omb_script_abi::stat_keys::StatKey;
    for case in 0..8 {
        let (mut w, mut driver) = world(SingleLaneConfig {
            mana_enabled: true, base_recovery_enabled: false, ..fast_config()
        }, SimulationTickProfile::Production60Hz);
        let amount = match case { 0 => f32::NAN, 1 => f32::INFINITY, 2 => -1.0,
            3 => 0.0, 4 => 1.0 / 2048.0, 5 => 1_000_001.0, _ => 20.0 };
        w.insert(ItemRegistry::from_configs(vec![ItemConfig {
            id: "moba_armor".into(), name: "Mana rejection fixture".into(), cost: 100,
            bonus: ItemBonus::default(), active: Some(ActiveEffect::RestoreMana { amount }),
            cooldown: 5.0, recipe: vec![],
        }]));
        driver.step(&mut w, []).unwrap();
        let hero = hero_pair(&w)[0];
        w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 200;
        driver.step(&mut w, [
            (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_armor".into() })) }),
            (1, PlayerInput { action: Some(PlayerInputEnum::HoldPosition(omoba_core::game_proto::HoldPosition { queued: false })) }),
        ]).unwrap();
        let base = w.read_storage::<Hero>().get(hero).unwrap().moba_mana_capacity().unwrap();
        w.write_storage::<Hero>().get_mut(hero).unwrap().mana_pool = if case == 6 { None }
            else { Some(ManaPool::new(Fixed64::from_i32(10), base).unwrap()) };
        if case == 7 { w.write_resource::<BuffStore>().add(hero, "fixture_invalid_capacity", Fixed64::from_i32(10),
            serde_json::json!({ StatKey::ManaBonus.as_str(): 1_000_001 * 1024i64 })); }
        let pool = w.read_storage::<Hero>().get(hero).unwrap().mana_pool.clone();
        let command = format!("{:?}", w.read_storage::<HeroCommandQueue>().get(hero).unwrap());
        w.write_resource::<ScriptEventQueue>().drain();
        assert!(handle_item_use_from_input(&mut w, 0, None, None, 1).is_err(), "case {case}");
        assert_eq!(w.read_storage::<Hero>().get(hero).unwrap().mana_pool, pool);
        assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 0.0);
        assert_eq!(format!("{:?}", w.read_storage::<HeroCommandQueue>().get(hero).unwrap()), command);
        assert!(w.write_resource::<ScriptEventQueue>().drain().is_empty());
    }
}

#[test]
fn item_active_rejection_is_non_mutating_for_invalid_or_unimplemented_effects() {
    for (active, cooldown) in [
        (ActiveEffect::SprintBuff { ms_bonus: f32::NAN, duration: 1.0 }, 5.0),
        (ActiveEffect::SprintBuff { ms_bonus: 60.0, duration: 0.0 }, 5.0),
        (ActiveEffect::SprintBuff { ms_bonus: 60.0, duration: f32::INFINITY }, 5.0),
        (ActiveEffect::SprintBuff { ms_bonus: 60.0, duration: 1.0 }, f32::NAN),
        (ActiveEffect::RestoreMana { amount: f32::NAN }, 5.0),
        (ActiveEffect::DamageReduce { percent: f32::NAN, duration: 1.0 }, 5.0),
        (ActiveEffect::HeadshotNext { bonus_damage: f32::NAN }, 5.0),
        (ActiveEffect::Shield { amount: f32::NAN, duration: 1.0 }, 5.0),
        (ActiveEffect::Shield { amount: -1.0, duration: 1.0 }, 5.0),
        (ActiveEffect::Shield { amount: 100.0, duration: 0.0 }, 5.0),
        (ActiveEffect::Shield { amount: 100.0, duration: f32::INFINITY }, 5.0),
    ] {
        let (mut w, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
        w.insert(ItemRegistry::from_configs(vec![ItemConfig {
            id: "moba_armor".into(), name: "Invalid fixture".into(), cost: 100,
            bonus: ItemBonus::default(), active: Some(active.clone()), cooldown: 5.0, recipe: vec![],
        }]));
        driver.step(&mut w, []).unwrap();
        let hero = hero_pair(&w)[0];
        w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 200;
        driver.step(&mut w, [
            (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_armor".into() })) }),
            (1, PlayerInput { action: Some(PlayerInputEnum::HoldPosition(omoba_core::game_proto::HoldPosition { queued: false })) }),
        ]).unwrap();
        let before = format!("{:?}", w.read_storage::<HeroCommandQueue>().get(hero).unwrap());
        // Shop rightly rejects non-finite cooldowns. Replace only the fixture registry
        // after a valid purchase to exercise item-use's independent preflight.
        w.insert(ItemRegistry::from_configs(vec![ItemConfig {
            id: "moba_armor".into(), name: "Invalid fixture".into(), cost: 100,
            bonus: ItemBonus::default(), active: Some(active), cooldown, recipe: vec![],
        }]));
        let base = w.read_storage::<CProperty>().get(hero).unwrap().msd;
        assert!(handle_item_use_from_input(&mut w, 0, None, None, 1).is_err());
        assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 0.0);
        assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().msd, base);
        assert!(!w.read_resource::<BuffStore>().has(hero, "item_sprint:moba_armor"));
        assert_eq!(w.read_resource::<BuffStore>().shield_remaining(hero), Fixed64::ZERO);
        assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(hero), Fixed64::ZERO);
        assert_eq!(format!("{:?}", w.read_storage::<HeroCommandQueue>().get(hero).unwrap()), before);
    }
}

#[test]
fn item_shield_60hz_absorbs_after_reduction_and_expires_without_healing() {
    use omb_script_abi::stat_keys::StatKey;
    let (mut w, mut driver) = world(SingleLaneConfig { base_recovery_enabled: false,
        ..fast_config() }, SimulationTickProfile::Production60Hz);
    w.insert(ItemRegistry::from_configs(vec![ItemConfig {
        id: "moba_armor".into(), name: "Shield fixture".into(), cost: 100,
        bonus: ItemBonus::default(), active: Some(ActiveEffect::Shield { amount: 100.0, duration: 0.25 }),
        cooldown: 5.0, recipe: vec![],
    }]));
    driver.step(&mut w, []).unwrap();
    let [hero, source] = hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 200;
    w.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(100);
    driver.step(&mut w, [
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_armor".into() })) }),
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse { item_slot: 0, target_pos: None, target_entity: None })) }),
    ]).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(100));
    assert_eq!(w.read_resource::<BuffStore>().shield_remaining(hero), Fixed64::from_i32(100));
    assert_eq!(w.read_resource::<BuffStore>().shield_remaining(source), Fixed64::ZERO);
    w.write_resource::<BuffStore>().add(hero, "fixture_reduction", Fixed64::from_i32(10),
        serde_json::json!({ StatKey::DamageTakenBonus.as_str(): -512 }));
    driver.step(&mut w, [(1, recall())]).unwrap();
    assert!(w.read_resource::<MobaMatch>().is_recalling(hero));
    let hit = |w: &mut World, amount: i32| {
        let mut damage = lethal(source, hero);
        if let Outcome::Damage { real, .. } = &mut damage { *real = Fixed64::from_i32(amount); }
        w.write_resource::<Vec<Outcome>>().push(damage);
        process_outcomes(w, &mut RuntimeEventVecSink::default()).unwrap();
    };
    hit(&mut w, 120);
    assert!(!w.read_resource::<MobaMatch>().is_recalling(hero));
    assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(100));
    assert_eq!(w.read_resource::<BuffStore>().shield_remaining(hero), Fixed64::from_i32(40));
    w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage { target: hero, amount: Fixed64::from_i32(50) });
    process_outcomes(&mut w, &mut RuntimeEventVecSink::default()).unwrap();
    assert_eq!(w.read_resource::<BuffStore>().shield_remaining(hero), Fixed64::ZERO);
    assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(90));
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 0.0;
    assert!(handle_item_use_from_input(&mut w, 0, None, None, 1).is_ok());
    assert!(w.write_resource::<BuffStore>().grant_damage_shield(hero, Fixed64::from_i32(20), Fixed64::from_raw(1)));
    assert_eq!(w.read_resource::<BuffStore>().shield_remaining(hero), Fixed64::from_i32(100));
    for _ in 0..20 { driver.step(&mut w, []).unwrap(); }
    assert_eq!(w.read_resource::<BuffStore>().shield_remaining(hero), Fixed64::ZERO);
    assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(90));
    hit(&mut w, 20);
    assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(80));
}

#[test]
fn generated_shield_60hz_owner_projection_tracks_absorption_expiry_and_death() {
    use omoba_core::runtime::native::economy_projection::OwnerEconomyState;
    let (mut w, mut driver) = world(SingleLaneConfig { base_recovery_enabled: false,
        ..fast_config() }, SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let [hero, _] = hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 2000;
    let check = |facts: &[omoba_core::runtime::OrderedFact], raw: i64| {
        let owner = facts.iter().rev().find_map(|fact| match &fact.fact {
            ObservableFact::OwnerEconomy { state, .. } if state.player_id == 1 => Some(state), _ => None,
        }).unwrap();
        assert_eq!(owner.shield_remaining_raw, raw);
        assert_eq!(OwnerEconomyState::decode(&owner.encode()).unwrap(), *owner);
        let other = facts.iter().rev().find_map(|fact| match &fact.fact {
            ObservableFact::OwnerEconomy { state, .. } if state.player_id == 2 => Some(state), _ => None,
        }).unwrap();
        assert_eq!(other.shield_remaining_raw, 0);
    };
    let result = driver.step(&mut w, [
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_guard_charm".into() })) }),
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse { item_slot: 0, target_pos: None, target_entity: None })) }),
    ]).unwrap();
    check(&result.facts, 100 * 1024);
    w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage { target: hero, amount: Fixed64::from_i32(40) });
    let result = driver.step(&mut w, []).unwrap();
    check(&result.facts, 60 * 1024);
    w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage { target: hero, amount: Fixed64::from_i32(60) });
    let result = driver.step(&mut w, []).unwrap();
    check(&result.facts, 0);
    w.write_resource::<BuffStore>().grant_damage_shield(hero, Fixed64::from_i32(20), Fixed64::from_raw(1));
    let result = driver.step(&mut w, []).unwrap();
    check(&result.facts, 0);
    w.write_resource::<BuffStore>().grant_damage_shield(hero, Fixed64::from_i32(20), Fixed64::from_i32(3));
    let pos = w.read_storage::<Pos>().get(hero).unwrap().0;
    w.write_resource::<Vec<Outcome>>().push(Outcome::Death { pos, ent: hero });
    let result = driver.step(&mut w, []).unwrap();
    check(&result.facts, 0);
}

#[test]
fn item_next_attack_bonus_60hz_formal_use_arms_owner_without_base_stat_change() {
    let (mut w, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    w.insert(ItemRegistry::from_configs(vec![ItemConfig {
        id: "moba_armor".into(), name: "Next attack fixture".into(), cost: 100,
        bonus: ItemBonus::default(), active: Some(ActiveEffect::HeadshotNext { bonus_damage: 60.0 }),
        cooldown: 5.0, recipe: vec![],
    }]));
    driver.step(&mut w, []).unwrap();
    let [hero, other] = hero_pair(&w);
    let base = w.read_storage::<TAttack>().get(hero).unwrap().atk_physic.v;
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 200;
    driver.step(&mut w, [
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_armor".into() })) }),
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse { item_slot: 0, target_pos: None, target_entity: None })) }),
    ]).unwrap();
    assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(hero), Fixed64::from_i32(60));
    assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(other), Fixed64::ZERO);
    assert_eq!(w.read_storage::<TAttack>().get(hero).unwrap().atk_physic.v, base);
    assert!(handle_item_use_from_input(&mut w, 0, None, None, 1).is_err());
    assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(hero), Fixed64::from_i32(60));
}

#[test]
fn item_damage_reduce_60hz_settles_packets_and_expires_without_stacking() {
    use omb_script_abi::stat_keys::StatKey;
    let (mut w, mut driver) = world(SingleLaneConfig {
        base_recovery_enabled: false, ..fast_config()
    }, SimulationTickProfile::Production60Hz);
    w.insert(ItemRegistry::from_configs(vec![
        ItemConfig { id: "moba_armor".into(), name: "Weak reduction".into(), cost: 100,
            bonus: ItemBonus::default(), active: Some(ActiveEffect::DamageReduce { percent: 0.25, duration: 0.25 }),
            cooldown: 5.0, recipe: vec![] },
        ItemConfig { id: "moba_sword".into(), name: "Strong reduction".into(), cost: 100,
            bonus: ItemBonus::default(), active: Some(ActiveEffect::DamageReduce { percent: 0.5, duration: 0.125 }),
            cooldown: 5.0, recipe: vec![] },
    ]));
    driver.step(&mut w, []).unwrap();
    let [hero, source] = hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 500;
    { let mut props = w.write_storage::<CProperty>(); let p = props.get_mut(hero).unwrap();
        p.hp = Fixed64::from_i32(10_000); p.mhp = p.hp; }
    let use_slot = |slot| (1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse {
        item_slot: slot, target_pos: None, target_entity: None,
    })) });
    driver.step(&mut w, [
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_armor".into() })) }),
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_sword".into() })) }),
        use_slot(0), use_slot(1),
    ]).unwrap();
    let damage = |w: &mut World, expected: i32| {
        let before = w.read_storage::<CProperty>().get(hero).unwrap().hp;
        w.write_resource::<Vec<Outcome>>().push(Outcome::Damage {
            pos: omoba_sim::Vec2::ZERO, phys: Fixed64::from_i32(80),
            magi: Fixed64::from_i32(40), real: Fixed64::from_i32(40),
            source, target: hero, damage_profile: 0, predeclared: false,
        });
        process_outcomes(w, &mut RuntimeEventVecSink::default()).unwrap();
        assert_eq!(before - w.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(expected));
    };
    assert_eq!(w.read_resource::<BuffStore>().sum_add(hero, StatKey::DamageTakenBonus), Fixed64::from_raw(-512));
    assert_eq!(w.read_resource::<BuffStore>().sum_add(source, StatKey::DamageTakenBonus), Fixed64::ZERO);
    damage(&mut w, 80);
    let remaining = w.read_resource::<BuffStore>().get(hero, "item_damage_reduce:moba_sword").unwrap().remaining;
    assert!(handle_item_use_from_input(&mut w, 1, None, None, 1).is_err());
    assert_eq!(w.read_resource::<BuffStore>().get(hero, "item_damage_reduce:moba_sword").unwrap().remaining, remaining);
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[1].as_mut().unwrap().cooldown_remaining = 0.0;
    assert!(handle_item_use_from_input(&mut w, 1, None, None, 1).is_ok());
    assert_eq!(w.read_resource::<BuffStore>().sum_add(hero, StatKey::DamageTakenBonus), Fixed64::from_raw(-512));
    for _ in 0..9 { driver.step(&mut w, []).unwrap(); }
    assert!(!w.read_resource::<BuffStore>().has(hero, "item_damage_reduce:moba_sword"));
    assert_eq!(w.read_resource::<BuffStore>().sum_add(hero, StatKey::DamageTakenBonus), Fixed64::from_raw(-256));
    damage(&mut w, 120);
    for _ in 0..12 { driver.step(&mut w, []).unwrap(); }
    assert_eq!(w.read_resource::<BuffStore>().sum_add(hero, StatKey::DamageTakenBonus), Fixed64::ZERO);
    damage(&mut w, 160);
}

#[test]
fn item_damage_reduce_preflight_rejects_invalid_effect_without_mutation() {
    for (percent, duration) in [(f32::NAN, 1.0), (f32::INFINITY, 1.0), (-0.1, 1.0),
        (0.0, 1.0), (1.1, 1.0), (1.0 / 2048.0, 1.0), (0.25, 0.0),
        (0.25, -1.0), (0.25, f32::NAN), (0.25, f32::INFINITY), (0.25, 61.0)] {
        let (mut w, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
        w.insert(ItemRegistry::from_configs(vec![ItemConfig {
            id: "moba_armor".into(), name: "Rejected reduction".into(), cost: 100,
            bonus: ItemBonus::default(), active: Some(ActiveEffect::DamageReduce { percent, duration }),
            cooldown: 5.0, recipe: vec![],
        }]));
        driver.step(&mut w, []).unwrap();
        let hero = hero_pair(&w)[0];
        w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 200;
        driver.step(&mut w, [
            (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_armor".into() })) }),
            (1, PlayerInput { action: Some(PlayerInputEnum::HoldPosition(omoba_core::game_proto::HoldPosition { queued: false })) }),
        ]).unwrap();
        let command = format!("{:?}", w.read_storage::<HeroCommandQueue>().get(hero).unwrap());
        assert!(handle_item_use_from_input(&mut w, 0, None, None, 1).is_err());
        assert!(!w.read_resource::<BuffStore>().has(hero, "item_damage_reduce:moba_armor"));
        assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 0.0);
        assert_eq!(format!("{:?}", w.read_storage::<HeroCommandQueue>().get(hero).unwrap()), command);
    }
}

#[test]
fn single_lane_bot_match_finishes_and_replays_every_tick() {
    let profile = SimulationTickProfile::Coarse15Hz;
    let config = fast_config();
    let (mut world, mut driver) = world(config.clone(), profile);
    let mut recording = Vec::new();
    let mut digests = Vec::new();
    let mut end_events = 0;
    let mut cast_inputs = [0u32; 4];
    for _ in 0..4_500 {
        let inputs = single_lane_bot_inputs(
            &world,
            [SingleLaneBotPolicy::Push, SingleLaneBotPolicy::Guard],
        );
        for (_, input) in &inputs {
            if let Some(PlayerInputEnum::CastAbility(cast)) = &input.action {
                cast_inputs[cast.ability_index as usize] += 1;
            }
        }
        let result = driver.step(&mut world, inputs.clone()).unwrap();
        end_events += result
            .events
            .iter()
            .filter(|e| e.topic == "game.end")
            .count();
        recording.push(inputs);
        digests.push(single_lane_replay_digest(&world));
        if matches!(
            world.read_resource::<MobaMatch>().phase,
            MobaMatchPhase::Finished { .. }
        ) {
            break;
        }
    }
    let (phase, waves, deaths, respawns) = {
        let state = world.read_resource::<MobaMatch>();
        (
            state.phase,
            state.waves,
            state.heroes.iter().map(|h| h.deaths).sum::<u32>(),
            state.heroes.iter().map(|h| h.respawns).sum::<u32>(),
        )
    };
    if !matches!(phase, MobaMatchPhase::Finished { .. }) {
        let entities = world.entities();
        let units = world.read_storage::<Unit>();
        let positions = world.read_storage::<Pos>();
        let hp = world.read_storage::<CProperty>();
        let faction = world.read_storage::<Faction>();
        let commands = world.read_storage::<HeroCommandQueue>();
        for (e, unit, pos, hp, faction) in (&entities, &units, &positions, &hp, &faction).join() {
            println!(
                "stalled entity={e:?} {} team={} x={:?} hp={:?} command={:?}",
                unit.id,
                faction.team_id,
                pos.0.x,
                hp.hp,
                commands.get(e)
            );
        }
    }
    assert!(matches!(phase, MobaMatchPhase::Finished { winner: Some(0), .. }),
        "match did not finish: phase={phase:?}, waves={waves}, deaths={deaths}, respawns={respawns}");
    assert!(waves > 1);
    assert!(
        deaths > 0 && respawns > 0,
        "must exercise death and respawn"
    );
    assert_eq!(end_events, 1);
    assert!(
        !cast_inputs.contains(&0),
        "four learned skills must traverse formal input: {cast_inputs:?}"
    );
    let (mut replay, mut replay_driver) = world_for_replay(config, profile);
    for (index, (inputs, digest)) in recording.into_iter().zip(digests).enumerate() {
        replay_driver.step(&mut replay, inputs).unwrap();
        assert_eq!(
            single_lane_replay_digest(&replay),
            digest,
            "replay diverged at tick {}",
            index + 1
        );
    }
    assert_eq!(replay.read_resource::<MobaMatch>().phase, phase);
    let digest = single_lane_replay_digest(&world);
    for _ in 0..5 {
        let result = driver
            .step(
                &mut world,
                [(
                    1,
                    PlayerInput {
                        action: Some(PlayerInputEnum::CastAbility(CastAbility {
                            ability_index: 0,
                            target_entity: None,
                            target_pos: None,
                        })),
                    },
                )],
            )
            .unwrap();
        assert!(result.events.is_empty());
        assert_eq!(
            single_lane_replay_digest(&world),
            digest,
            "finished gameplay must stay frozen"
        );
    }
    println!("single-lane acceptance: {phase:?}, waves={waves}, deaths={deaths}, respawns={respawns}, digest={digest}");
}

fn world_for_replay(
    config: SingleLaneConfig,
    profile: SimulationTickProfile,
) -> (World, SimulationDriver) {
    world(config, profile)
}

#[test]
fn single_lane_warmup_discards_inputs_and_pause_freezes_rules() {
    let (mut world, mut driver) = world(
        SingleLaneConfig::default(),
        SimulationTickProfile::Coarse15Hz,
    );
    let initial = single_lane_replay_digest(&world);
    let caster = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    let start = world.read_storage::<Pos>().get(caster).unwrap().0;
    driver
        .step(
            &mut world,
            [(
                1,
                PlayerInput {
                    action: Some(PlayerInputEnum::AttackMove(AttackMove {
                        target: Some(Vec2I {
                            x: 1_800 * 1024,
                            y: 0,
                        }),
                        queued: false,
                    })),
                },
            )],
        )
        .unwrap();
    assert_eq!(
        world.read_resource::<MobaMatch>().phase,
        MobaMatchPhase::Warmup
    );
    assert_eq!(world.read_storage::<Pos>().get(caster).unwrap().0, start);
    assert!(world
        .read_resource::<PendingPlayerInputs>()
        .inputs
        .is_empty());
    assert!(world
        .read_resource::<PendingMoveQueue>()
        .requests
        .is_empty());
    assert_ne!(
        single_lane_replay_digest(&world),
        initial,
        "warmup timer advanced"
    );
    world.write_resource::<GamePause>().is_paused = true;
    let frozen = single_lane_replay_digest(&world);
    for _ in 0..4 {
        driver.step(&mut world, []).unwrap();
    }
    assert_eq!(single_lane_replay_digest(&world), frozen);
}

#[test]
fn single_lane_base_shield_is_enforced_in_damage_settlement() {
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Coarse15Hz);
    driver.step(&mut world, []).unwrap();
    let (source, base, tower) = {
        let state = world.read_resource::<MobaMatch>();
        (
            state.heroes[0].entity.unwrap(),
            state.bases[1].unwrap(),
            state.towers[1].unwrap(),
        )
    };
    let damage = |target| Outcome::Damage {
        pos: omoba_sim::Vec2::ZERO,
        phys: Fixed64::ZERO,
        magi: Fixed64::ZERO,
        real: Fixed64::from_i32(10_000),
        source,
        target,
        damage_profile: 0,
        predeclared: false,
    };
    let before = world.read_storage::<CProperty>().get(base).unwrap().hp;
    world.write_resource::<Vec<Outcome>>().push(damage(base));
    process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
    assert_eq!(
        world.read_storage::<CProperty>().get(base).unwrap().hp,
        before
    );
    world.write_resource::<Vec<Outcome>>().push(damage(tower));
    process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
    process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
    world.maintain();
    assert!(world.read_resource::<MobaMatch>().towers[1].is_none());
    world.write_resource::<Vec<Outcome>>().push(damage(base));
    process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
    process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
    world.maintain();
    finish_moba_match_tick(&mut world);
    assert!(matches!(
        world.read_resource::<MobaMatch>().phase,
        MobaMatchPhase::Finished {
            winner: Some(0),
            ..
        }
    ));
}

#[test]
fn single_lane_duplicate_death_keeps_progression_and_one_respawn() {
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Coarse15Hz);
    driver.step(&mut world, []).unwrap();
    let original = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    world
        .write_storage::<Hero>()
        .get_mut(original)
        .unwrap()
        .level = 7;
    world.write_storage::<Gold>().get_mut(original).unwrap().0 = 123;
    world
        .write_storage::<CProperty>()
        .get_mut(original)
        .unwrap()
        .hp = Fixed64::ZERO;
    let pos = world.read_storage::<Pos>().get(original).unwrap().0;
    world.write_resource::<Vec<Outcome>>().extend([
        Outcome::Death { pos, ent: original },
        Outcome::Death { pos, ent: original },
    ]);
    process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
    world.maintain();
    assert_eq!(world.read_resource::<MobaMatch>().heroes[0].deaths, 1);
    assert!(handle_ability_cast_from_input(&mut world, 0, None, None, 1).is_err());
    for _ in 0..31 {
        driver.step(&mut world, []).unwrap();
    }
    let state = world.read_resource::<MobaMatch>();
    let respawned = state.heroes[0].entity.expect("respawned");
    assert_ne!(original, respawned, "new generation/identity");
    assert_eq!(state.heroes[0].respawns, 1);
    assert_eq!(
        world.read_storage::<Hero>().get(respawned).unwrap().level,
        7
    );
    assert_eq!(world.read_storage::<Gold>().get(respawned).unwrap().0, 123);
    let owner = world.read_storage::<PlayerOwner>();
    assert_eq!(
        (&world.entities(), &owner)
            .join()
            .filter(|(_, o)| o.player_id == 1)
            .count(),
        1
    );
}

#[test]
fn single_lane_invalid_config_is_atomic() {
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    let bad = SingleLaneConfig {
        heroes: ["training_luminary".into(), "missing_hero".into()],
        ..SingleLaneConfig::default()
    };
    assert!(setup_single_lane_match(&mut world, bad).is_err());
    assert!(world.try_fetch::<MobaMatch>().is_none());
    for (gold, window) in [(1_000_001,10),(100,0),(100,61)] {
        let config = SingleLaneConfig {hero_assist_gold:gold,assist_window_seconds:window,..SingleLaneConfig::default()};
        assert!(setup_single_lane_match(&mut world, config).is_err());
        assert!(world.try_fetch::<MobaMatch>().is_none());
    }
    assert_eq!(
        (&world.entities(), &world.read_storage::<Unit>())
            .join()
            .count(),
        0
    );
    setup_single_lane_match(&mut world, SingleLaneConfig::default()).unwrap();
    assert!(setup_single_lane_match(&mut world, SingleLaneConfig::default()).is_err());
}

#[test]
fn single_lane_same_tick_base_deaths_are_a_draw_and_emit_once() {
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Coarse15Hz);
    driver.step(&mut world, []).unwrap();
    let (towers, bases) = {
        let state = world.read_resource::<MobaMatch>();
        (state.towers, state.bases)
    };
    for entity in towers.into_iter().chain(bases).flatten() {
        let pos = world.read_storage::<Pos>().get(entity).unwrap().0;
        world
            .write_resource::<Vec<Outcome>>()
            .push(Outcome::Death { ent: entity, pos });
    }
    process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
    world.maintain();
    finish_moba_match_tick(&mut world);
    finish_moba_match_tick(&mut world);
    assert!(matches!(
        world.read_resource::<MobaMatch>().phase,
        MobaMatchPhase::Finished { winner: None, .. }
    ));
    assert_eq!(
        world
            .read_resource::<RuntimeEvents>()
            .iter()
            .filter(|e| e.topic == "game.end")
            .count(),
        1
    );
}

#[test]
fn single_lane_bootstrap_rejects_missing_script_handlers() {
    assert!(create_single_lane_world(SingleLaneConfig::default(), ScriptRegistry::new()).is_err());
}

#[test]
fn single_lane_four_skills_use_formal_input_and_filtered_hp_settlement() {
    use std::collections::BTreeSet;
    use prost::Message;
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Coarse15Hz);
    driver.step(&mut world, []).unwrap();
    let caster = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    let target = world.read_resource::<MobaMatch>().heroes[1].entity.unwrap();
    // Isolate the cast fixture from lane combat; Bot acceptance never does this.
    world.write_storage::<Pos>().get_mut(caster).unwrap().0 =
        omoba_sim::Vec2::new(Fixed64::from_i32(-1000), Fixed64::ZERO);
    world.write_storage::<Pos>().get_mut(target).unwrap().0 =
        omoba_sim::Vec2::new(Fixed64::from_i32(-990), Fixed64::ZERO);
    world
        .write_storage::<CProperty>()
        .get_mut(caster)
        .unwrap()
        .hp = Fixed64::from_i32(400);
    world
        .write_storage::<CProperty>()
        .get_mut(target)
        .unwrap()
        .hp = Fixed64::from_i32(1000);
    world
        .write_storage::<CProperty>()
        .get_mut(target)
        .unwrap()
        .mhp = Fixed64::from_i32(1000);
    let baseline = driver.step(&mut world, []).unwrap();
    project_tick(&mut world, baseline);
    let allow = TeamProjectorConfig::default().component_allowlist;
    let starts = world.write_resource::<TeamProjectionRuntime>()
        .build_team_bootstraps(driver.tick() + 1, 15, fast_config().seed);
    let mut replicas: Vec<_> = starts.into_iter().map(|(team, start)| {
        let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, allow.clone(), BTreeSet::new()).unwrap();
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world, &stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();
        (team, replica, stepper)
    }).collect();
    let mut cast = |world: &mut World, slot, target_entity| {
        let result = driver
            .step(
                world,
                [(
                    1,
                    PlayerInput {
                        action: Some(PlayerInputEnum::CastAbility(CastAbility {
                            ability_index: slot,
                            target_entity,
                            target_pos: None,
                        })),
                    },
                )],
            )
            .unwrap();
        // Match the server acceptance boundary: payload has no canonical
        // target ID; the projector supplies this team's disclosed reference.
        let sanitized = omoba_core::game_proto::PlayerInput {
            action: Some(omoba_core::game_proto::player_input::Action::CastAbility(
                omoba_core::game_proto::CastAbility { ability_index: slot, target_pos: None, target_entity: None }
            )),
        };
        world.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.push(
            CanonicalAcceptedInput::from_authoritative_acceptance(1, 1, u64::from(slot) + 1, 4,
                canonical_entity_id(caster), target_entity.map(|_| canonical_entity_id(target)), sanitized.encode_to_vec())
        );
        project_tick(world, result);
        let frames = world.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected = world.write_resource::<TeamProjectionRuntime>()
            .build_team_bootstraps(driver.tick() + 1, 15, fast_config().seed);
        for (team, replica, stepper) in &mut replicas {
            let step = frames[team].frame.step.as_ref().unwrap();
            assert_eq!(step.accepted_inputs.len(), usize::from(*team == 1), "enemy inputs must stay private");
            if *team == 1 {
                assert!(!step.public_events.iter().any(|event| event.event_kind == FactKind::CommittedCooldown as u32), "owner cooldown must be predicted from accepted input, not repaired");
                // Automatic attack clocks settle against authority NPC/lifecycle
                // outcomes; cooldown still proves the owner's accepted input.
            }
            assert!(matches!(replica.apply_frame(frames[team].frame.clone(), stepper).unwrap(), FrameApplyResult::Applied { .. }));
            let expected_view = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team], allow.clone(), BTreeSet::new()).unwrap();
            if replica.canonical_team_hash() != expected_view.canonical_team_hash() {
                for (id, expected_entity) in &expected_view.world().entities {
                    let actual = &replica.world().entities[id];
                    for (schema, bytes) in &expected_entity.components {
                        if actual.components.get(schema) != Some(bytes) {
                            println!("skill mismatch slot={slot} team={team} entity={id} schema={schema} expected={} actual={}", String::from_utf8_lossy(bytes), actual.components.get(schema).map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default());
                        }
                    }
                }
            }
            assert_eq!(replica.canonical_team_hash(), expected_view.canonical_team_hash(), "skill slot {slot}, team {team}");
            assert!(stepper.filtered.world.try_fetch::<MobaMatch>().is_none());
        }
    };
    cast(&mut world, 0, Some(target.id()));
    assert_eq!(
        world.read_storage::<CProperty>().get(target).unwrap().hp,
        Fixed64::from_i32(920)
    );
    cast(&mut world, 1, None);
    assert_eq!(
        world.read_storage::<CProperty>().get(caster).unwrap().hp,
        Fixed64::from_i32(470)
    );
    cast(&mut world, 2, Some(target.id()));
    assert_eq!(
        world.read_storage::<CProperty>().get(target).unwrap().hp,
        Fixed64::from_i32(740)
    );
    cast(&mut world, 3, None);
    assert_eq!(
        world.read_storage::<CProperty>().get(caster).unwrap().hp,
        Fixed64::from_i32(550)
    );
    assert_eq!(
        world
            .read_storage::<Hero>()
            .get(caster)
            .unwrap()
            .ability_cooldowns
            .len(),
        4
    );
    drop(cast);
    // Continue past the immediate skill result into real attack phases. The
    // earlier four single-tick casts did not cover enemy Windup/Backswing.
    for _ in 0..120 {
        let result = driver.step(&mut world, []).unwrap();
        project_tick(&mut world, result);
        let frames = world.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected = world.write_resource::<TeamProjectionRuntime>()
            .build_team_bootstraps(driver.tick() + 1, 15, fast_config().seed);
        for (team, replica, stepper) in &mut replicas {
            assert!(frames[team].frame.step.as_ref().unwrap().accepted_inputs.is_empty());
            assert!(frames[team].frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            replica.apply_frame(frames[team].frame.clone(), stepper).unwrap();
            let expected_view = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
                &expected[team], allow.clone(), BTreeSet::new()).unwrap();
            assert_eq!(replica.canonical_team_hash(), expected_view.canonical_team_hash(),
                "post-cast attack phase team {team} tick {}", driver.tick());
        }
    }
}

#[test]
fn single_lane_attacks_use_armor_and_emit_committed_combat_facts() {
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Coarse15Hz);
    driver.step(&mut world, []).unwrap();
    let source = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    let target = world.read_resource::<MobaMatch>().heroes[1].entity.unwrap();
    let hp = world.read_storage::<CProperty>().get(target).unwrap().hp;
    world
        .write_resource::<Vec<DamageInstance>>()
        .push(DamageInstance::new_attack(
            source,
            target,
            Fixed64::from_i32(100),
        ));
    let result = driver.step(&mut world, []).unwrap();
    let after = world.read_storage::<CProperty>().get(target).unwrap().hp;
    assert!(
        after > hp - Fixed64::from_i32(100) && after < hp,
        "generated armor applies"
    );
    assert!(result
        .facts
        .iter()
        .any(|fact| matches!(fact.fact, ObservableFact::DirectCombat { .. })));
    assert!(world
        .read_resource::<ObservableFactBuffer>()
        .drain_ordered()
        .unwrap()
        .is_empty());
}

#[test]
fn single_lane_rejects_non_roster_players_and_td_controls() {
    let (mut world, mut driver) = world(fast_config(), SimulationTickProfile::Coarse15Hz);
    driver.step(&mut world, []).unwrap();
    let caster = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    let start = world.read_storage::<Pos>().get(caster).unwrap().0;
    driver
        .step(
            &mut world,
            [
                (
                    999,
                    PlayerInput {
                        action: Some(PlayerInputEnum::CastAbility(CastAbility {
                            ability_index: 1,
                            target_pos: None,
                            target_entity: None,
                        })),
                    },
                ),
                (
                    1,
                    PlayerInput {
                        action: Some(PlayerInputEnum::StartRound(StartRound {})),
                    },
                ),
                (
                    1,
                    PlayerInput {
                        action: Some(PlayerInputEnum::ToggleGameSpeed(ToggleGameSpeed {})),
                    },
                ),
            ],
        )
        .unwrap();
    assert_eq!(world.read_storage::<Pos>().get(caster).unwrap().0, start);
    assert!(world
        .read_storage::<Hero>()
        .get(caster)
        .unwrap()
        .ability_cooldowns
        .is_empty());
    assert!(!world.read_resource::<CurrentCreepWave>().is_running);
    assert_eq!(world.read_resource::<GameSpeed>().multiplier(), 1);
}

fn project_tick(world: &mut World, result: SimulationTickResult) {
    *world.write_resource::<CommittedProjectionBatch>() = CommittedProjectionBatch {
        tick: result.tick,
        facts: result.facts,
        barrier_reached: true,
        ..Default::default()
    };
    run_committed_visibility_wave_b(world, result.tick, 0);
    run_team_projection_after_wave_b(world, result.tick).unwrap();
}

#[test]
fn single_lane_shop_filtered_settlement_is_private_atomic_and_survives_respawn() {
    use std::collections::BTreeSet;
    use prost::Message;
    fn assert_private(view: &DisclosedReplicaWorld, team: u32) -> usize {
        let mut heroes = 0;
        for entity in view.entities.values() {
            let render = entity.components.get(&DEMO_RENDER_COMPONENT_SCHEMA_ID)
                .and_then(|bytes| decode_demo_render_state(bytes)).unwrap();
            if render.kind != 1 { continue; }
            heroes += 1;
            for schema in [DISCLOSED_GOLD_COMPONENT_SCHEMA_ID, DISCLOSED_INVENTORY_COMPONENT_SCHEMA_ID,
                DISCLOSED_ITEM_EFFECTS_COMPONENT_SCHEMA_ID] {
                assert_eq!(entity.components.contains_key(&schema), render.team_id == team,
                    "private schema leaked to team {team}");
            }
        }
        heroes
    }
    let (mut authority, mut driver) = world(fast_config(), SimulationTickProfile::Coarse15Hz);
    let heroes = hero_pair(&authority);
    for hero in heroes {
        authority.write_storage::<Gold>().get_mut(hero).unwrap().0 = 1000;
        authority.write_storage::<VisionSource>().get_mut(hero).unwrap().radius = Fixed64::from_i32(5000);
    }
    let result = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, result);
    let allow = TeamProjectorConfig::default().component_allowlist;
    let starts = authority.write_resource::<TeamProjectionRuntime>()
        .build_team_bootstraps(driver.tick() + 1, 15, fast_config().seed);
    let mut replicas: Vec<_> = starts.into_iter().map(|(team, start)| {
        let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, allow.clone(), BTreeSet::new()).unwrap();
        assert_eq!(assert_private(replica.world(), team), 2, "both heroes must be visible for privacy verification");
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world, &stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();
        (team, replica, stepper)
    }).collect();
    let buy = |player, id: &str| (player, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: id.into() })) });
    let sell = |player, slot| (player, PlayerInput { action: Some(PlayerInputEnum::ItemSell(ItemSell { item_slot: slot })) });
    let mut advance = |authority: &mut World, driver: &mut SimulationDriver, inputs: Vec<(u32, PlayerInput)>| {
        let accepted: Vec<_> = inputs.iter().enumerate().map(|(ordinal, (player, input))| {
            let state = authority.read_resource::<MobaMatch>();
            let side = state.config.players.iter().position(|id| id == player).unwrap();
            let actor = state.heroes[side].entity.unwrap();
            let kind = if matches!(input.action, Some(PlayerInputEnum::ItemBuy(_))) { 17 } else { 18 };
            CanonicalAcceptedInput::from_authoritative_acceptance(state.config.teams[side], *player,
                (driver.tick() + 1) * 16 + ordinal as u64, kind, canonical_entity_id(actor), None, input.encode_to_vec())
        }).collect();
        let result = driver.step(authority, inputs).unwrap();
        authority.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted.clone());
        project_tick(authority, result);
        let frames = authority.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let starts = authority.write_resource::<TeamProjectionRuntime>()
            .build_team_bootstraps(driver.tick() + 1, 15, fast_config().seed);
        for (team, replica, stepper) in &mut replicas {
            let frame = frames[team].frame.clone();
            assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            assert!(frame.step.as_ref().unwrap().accepted_inputs.iter().all(|input| input.player_id == *team));
            let receipts: Vec<_> = frame.step.as_ref().unwrap().public_events.iter()
                .filter(|event| event.event_kind == FactKind::ShopReceipt as u32)
                .map(|event| omoba_core::runtime::shop_receipt::ShopReceipt::decode(&event.sanitized_payload).unwrap())
                .collect();
            let own: Vec<_> = accepted.iter().filter(|input| input.team_id == *team).collect();
            assert_eq!(receipts.len(), own.len());
            for (receipt, input) in receipts.iter().zip(own) {
                // Exact wire correlation is asserted in core/shop_receipt;
                // script sources deliberately never inspect transport IDs.
                assert_eq!((receipt.player_id, receipt.tick), (input.player_id, driver.tick()));
                let expected_error = if *team == 1 && driver.tick() == 6 { 6 } else { 0 };
                assert_eq!(receipt.result_code, expected_error);
            }
            let economy: Vec<_> = frame.step.as_ref().unwrap().public_events.iter()
                .filter(|event| event.event_kind == FactKind::OwnerEconomy as u32)
                .map(|event| omoba_core::runtime::native::economy_projection::OwnerEconomyState::decode(&event.sanitized_payload).unwrap())
                .collect();
            assert!(!economy.is_empty());
            assert!(economy.iter().all(|state| state.player_id == *team), "other team's persistent economy leaked");
            let current = economy.last().unwrap();
            if let Some(hero) = authority.read_resource::<MobaMatch>().heroes[(*team - 1) as usize].entity {
                assert_eq!(current.economy.gold, authority.read_storage::<Gold>().get(hero).unwrap().0);
                let (_, inventory, _) = current.economy.components().unwrap();
                assert_eq!(serde_json::to_value(&inventory).unwrap(),
                    serde_json::to_value(authority.read_storage::<Inventory>().get(hero).unwrap()).unwrap());
            } else if *team == 2 {
                assert_eq!(current.economy.gold, 550);
                assert_eq!(current.economy.item_ids[1], 3);
                assert!(!current.shop_available);
            }
            assert!(matches!(replica.apply_frame(frame, stepper).unwrap(), FrameApplyResult::Applied { .. }));
            let expected = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&starts[team], allow.clone(), BTreeSet::new()).unwrap();
            if replica.canonical_team_hash() != expected.canonical_team_hash() {
                for (id, entity) in &expected.world().entities {
                    for (schema, bytes) in &entity.components {
                        let actual = replica.world().entities.get(id).and_then(|entity| entity.components.get(schema));
                        if actual != Some(bytes) {
                            println!("shop mismatch team={team} tick={} entity={id} schema={schema:x} expected={bytes:?} actual={actual:?}", driver.tick());
                        }
                    }
                }
            }
            assert_eq!(replica.canonical_team_hash(), expected.canonical_team_hash(), "shop team {team} tick {}", driver.tick());
            assert_private(replica.world(), *team);
            let entities = stepper.filtered.world.entities();
            let heroes = stepper.filtered.world.read_storage::<Hero>();
            let owners = stepper.filtered.world.read_storage::<PlayerOwner>();
            let gold = stepper.filtered.world.read_storage::<Gold>();
            let inventory = stepper.filtered.world.read_storage::<Inventory>();
            let effects = stepper.filtered.world.read_storage::<ItemEffects>();
            for (entity, _, owner) in (&entities, &heroes, &owners).join() {
                let owned = owner.player_id == *team;
                assert_eq!(gold.get(entity).is_some(), owned);
                assert_eq!(inventory.get(entity).is_some(), owned);
                assert_eq!(effects.get(entity).is_some(), owned);
            }
            assert!(stepper.filtered.world.try_fetch::<MobaMatch>().is_none());
        }
    };
    for inputs in [
        vec![buy(1, "moba_sword"), buy(2, "moba_armor")], vec![],
        vec![buy(1, "moba_sword"), buy(1, "moba_greatsword"), buy(2, "moba_boots")], vec![],
        vec![buy(1, "moba_armor"), sell(2, 0)], vec![], vec![sell(1, 0)], vec![],
    ] { advance(&mut authority, &mut driver, inputs); }
    assert_eq!(authority.read_storage::<Gold>().get(heroes[0]).unwrap().0, 525);
    assert_eq!(authority.read_storage::<Gold>().get(heroes[1]).unwrap().0, 550);
    // Cover a fresh filtered bootstrap/reconnect after settled transactions.
    let current = authority.write_resource::<TeamProjectionRuntime>()
        .build_team_bootstraps(driver.tick() + 1, 15, fast_config().seed);
    for (team, start) in current {
        let fresh = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, allow.clone(), BTreeSet::new()).unwrap();
        assert_private(fresh.world(), team);
    }
    // Keep the boots across death and the new identity reveal. Increase all
    // remaining allied vision so the opponent stays visible while dead.
    for vision in (&mut authority.write_storage::<VisionSource>()).join() { vision.radius = Fixed64::from_i32(5000); }
    let speed_before = authority.read_storage::<CProperty>().get(heroes[1]).unwrap().msd;
    let pos = authority.read_storage::<Pos>().get(heroes[1]).unwrap().0;
    authority.write_resource::<Vec<Outcome>>().push(Outcome::Death { pos, ent: heroes[1] });
    for _ in 0..40 { advance(&mut authority, &mut driver, vec![]); }
    let new = authority.read_resource::<MobaMatch>().heroes[1].entity.unwrap();
    assert_ne!(canonical_entity_id(new), canonical_entity_id(heroes[1]));
    assert_eq!(authority.read_storage::<Gold>().get(new).unwrap().0, 550);
    assert_eq!(authority.read_storage::<Inventory>().get(new).unwrap().find_item("moba_boots"), Some(1));
    assert_eq!(authority.read_storage::<CProperty>().get(new).unwrap().msd, speed_before);
}

#[test]
fn single_lane_filtered_replicas_follow_npc_movement_and_damage_without_hidden_ai() {
    verify_filtered_single_lane(false);
}

#[test]
fn single_lane_filtered_enemy_movement_preserves_attack_priority() {
    verify_filtered_single_lane(true);
}

fn verify_filtered_single_lane(move_heroes: bool) {
    use std::collections::BTreeSet;
    let config = SingleLaneConfig {
        warmup: Fixed64::from_i32(2),
        ..fast_config()
    };
    let (mut authority, mut driver) = world(config, SimulationTickProfile::Production120Hz);
    let first = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, first);
    let starts = authority
        .write_resource::<TeamProjectionRuntime>()
        .build_team_bootstraps(2, 120, fast_config().seed);
    let allow = TeamProjectorConfig::default().component_allowlist;
    let mut replicas: Vec<_> = starts
        .into_iter()
        .map(|(team, start)| {
            let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
                &start,
                allow.clone(),
                BTreeSet::new(),
            )
            .unwrap();
            let mut stepper =
                SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), BTreeSet::new());
            stepper
                .script_registry
                .insert_manifest(crate::get_manifest());
            populate_ability_registry(&mut stepper.filtered.world, &stepper.script_registry);
            stepper.bootstrap_membership(replica.world()).unwrap();
            assert!(stepper.filtered.world.try_fetch::<MobaMatch>().is_none());
            // Opponent's starting hero is outside this team's actual vision.
            assert_eq!(
                replica
                    .world()
                    .entities
                    .values()
                    .filter(|entity| {
                        entity
                            .components
                            .get(&DEMO_RENDER_COMPONENT_SCHEMA_ID)
                            .and_then(|bytes| decode_demo_render_state(bytes))
                            .is_some_and(|state| state.kind == 1)
                    })
                    .count(),
                1
            );
            (team, replica, stepper)
        })
        .collect();
    let mut external_damage = 0;
    let mut movement = 0;
    let mut checkpoints = 0;
    // Warmup consumes two seconds before lane NPCs can meet and fight.
    for index in 0..if move_heroes { 2400 } else { 900 } {
        authority.write_resource::<GamePause>().is_paused = (300..320).contains(&index);
        let inputs = if move_heroes && index == 360 {
            vec![
                (
                    1,
                    PlayerInput {
                        action: Some(PlayerInputEnum::MoveTo(MoveTo {
                            target: Some(Vec2I {
                                x: 900 * 1024,
                                y: 700 * 1024,
                            }),
                            queued: false,
                        })),
                    },
                ),
                (
                    2,
                    PlayerInput {
                        action: Some(PlayerInputEnum::MoveTo(MoveTo {
                            target: Some(Vec2I {
                                x: -900 * 1024,
                                y: -700 * 1024,
                            }),
                            queued: false,
                        })),
                    },
                ),
            ]
        } else {
            vec![]
        };
        let result = driver.step(&mut authority, inputs).unwrap();
        project_tick(&mut authority, result);
        let frames = authority
            .read_resource::<TeamProjectionRuntime>()
            .latest_frames
            .clone();
        for (team, replica, stepper) in &mut replicas {
            let frame = frames[team].frame.clone();
            let expected = frame
                .post_step
                .as_ref()
                .and_then(|post| post.hash_checkpoint.as_ref())
                .map(|checkpoint| checkpoint.canonical_team_hash.clone());
            for step in &frame.step {
                external_damage += step.external_effects.len();
                movement += step
                    .public_events
                    .iter()
                    .filter(|event| {
                        event.event_kind == FactKind::Movement as u32
                            || event.event_kind == FactKind::PreStepMovement as u32
                    })
                    .count();
            }
            let applied = replica.apply_frame(frame, stepper).unwrap();
            assert!(
                matches!(applied, FrameApplyResult::Applied { .. }),
                "team {team}, tick {}: {applied:?}",
                driver.tick()
            );
            if let Some(expected) = expected {
                if replica.canonical_team_hash().as_slice() != expected.as_slice() {
                    let current = authority
                        .write_resource::<TeamProjectionRuntime>()
                        .build_team_bootstraps(driver.tick() + 1, 120, fast_config().seed);
                    let expected_view = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
                        &current[team],
                        allow.clone(),
                        BTreeSet::new(),
                    )
                    .unwrap();
                    for (id, expected_entity) in &expected_view.world().entities {
                        let actual = replica
                            .world()
                            .entities
                            .get(id)
                            .expect("expected visible membership");
                        for (schema, value) in &expected_entity.components {
                            if actual.components.get(schema) != Some(value) {
                                println!("mismatch team={team} replica={id} schema={schema}: expected={value:?}, actual={:?}", actual.components.get(schema));
                            }
                        }
                        for schema in actual.components.keys() {
                            if !expected_entity.components.contains_key(schema) {
                                println!("unexpected team={team} replica={id} schema={schema}");
                            }
                        }
                    }
                }
                assert_eq!(
                    replica.canonical_team_hash().as_slice(),
                    expected.as_slice(),
                    "team {team}, tick {}",
                    driver.tick()
                );
                checkpoints += 1;
            }
        }
    }
    assert!(
        (move_heroes || external_damage > 0) && movement > 0 && checkpoints >= 10,
        "external_damage={external_damage}, movement={movement}, checkpoints={checkpoints}"
    );
}

#[test]
fn single_lane_visibility_retires_death_and_reveals_new_respawn_identity() {
    let config = SingleLaneConfig {
        respawn_delay: Fixed64::from_raw(128),
        ..fast_config()
    };
    let (mut authority, mut driver) = world(config, SimulationTickProfile::Coarse15Hz);
    let first = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, first);
    let hero = authority.read_resource::<MobaMatch>().heroes[0]
        .entity
        .unwrap();
    let old_id = canonical_entity_id(hero);
    let pos = authority.read_storage::<Pos>().get(hero).unwrap().0;
    authority
        .write_resource::<Vec<Outcome>>()
        .push(Outcome::Death { pos, ent: hero });
    let death = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, death);
    let visibility = authority.read_resource::<TeamVisibilityRuntime>();
    assert!(!visibility.teams[&1].index.current.contains(&old_id));
    assert!(visibility.last_transitions[&1].iter().any(|transition| matches!(transition,
        VisibilityTransition::Hide { canonical_id, disposition: RememberDisposition::Forget, .. } if *canonical_id == old_id)));
    drop(visibility);
    for _ in 0..3 {
        let result = driver.step(&mut authority, []).unwrap();
        project_tick(&mut authority, result);
    }
    let respawn = authority.read_resource::<MobaMatch>().heroes[0]
        .entity
        .unwrap();
    assert_ne!(canonical_entity_id(respawn), old_id);
    assert!(authority.read_resource::<TeamVisibilityRuntime>().teams[&1]
        .index
        .current
        .contains(&canonical_entity_id(respawn)));
    assert_eq!(
        authority
            .read_storage::<VisionSource>()
            .get(respawn)
            .unwrap()
            .team,
        1
    );
}

#[test]
fn single_lane_full_match_filtered_replay_covers_respawn_and_finished_freeze() {
    full_match_filtered_lifecycle(SimulationTickProfile::Coarse15Hz, 15, 4_600);
}

#[test]
fn single_lane_production_full_match_filtered_lifecycle() {
    full_match_filtered_lifecycle(SimulationTickProfile::Production120Hz, 120, 36_800);
}

fn full_match_filtered_lifecycle(profile: SimulationTickProfile, fps: u32, limit: usize) {
    full_match_filtered_lifecycle_with_map(profile,fps,limit,None,fast_config().seed);
}

fn full_match_filtered_lifecycle_with_map(profile: SimulationTickProfile, fps: u32, limit: usize, map: Option<&str>, seed: u64) {
    use std::collections::BTreeSet;
    let config = SingleLaneConfig { warmup: Fixed64::from_i32(2),
        map_id: map.map(str::to_owned),
        seed,
        passive_gold_per_second: omoba_template_ids::MOBA_PASSIVE_GOLD_PER_SECOND, ..fast_config() };
    let seed = config.seed;
    let (mut authority, mut driver) = world(config, profile);
    let baseline = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, baseline);
    let allow = TeamProjectorConfig::default().component_allowlist;
    let starts = authority.write_resource::<TeamProjectionRuntime>()
        .build_team_bootstraps(driver.tick() + 1, fps, seed);
    let mut replicas: Vec<_> = starts.into_iter().map(|(team, start)| {
        let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
            &start, allow.clone(), BTreeSet::new()).unwrap();
        let mut stepper = SpecsDisclosedWorldStepper::from_start(
            &start, allow.clone(), BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world, &stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();
        (team, replica, stepper)
    }).collect();
    let mut end_events = 0;
    let mut finished_ticks = 0;
    let mut frozen_digest = None;
    let mut applied_steps = 0;
    for _ in 0..limit {
        // This is the existing headless fixture policy, not a production vision bot.
        let inputs = {
            let state = authority.read_resource::<MobaMatch>();
            if state.heroes.iter().any(|hero| hero.respawns == 0) {
                // Exercise death/rebirth before the faster explicit-target push
                // can finish. Only ordinary MoveTo inputs; no HP/phase edits.
                state.heroes.iter().enumerate().filter_map(|(side, hero)| {
                    hero.entity?;
                    if hero.respawns > 0 { return None; }
                    Some((state.config.players[side], PlayerInput {
                        action: Some(PlayerInputEnum::MoveTo(MoveTo {
                            target: Some(Vec2I { x: if side == 0 { 2400 * 1024 } else { 0 }, y: 0 }),
                            queued: false,
                        })),
                    }))
                }).collect()
            } else {
                let defender=state.config.players[1];
                let phase=state.phase;
                let lane_length=state.config.lane_length;
                drop(state);
                let mut inputs=single_lane_bot_inputs(&authority,[SingleLaneBotPolicy::Push,SingleLaneBotPolicy::Guard]);
                // This regression tests progression/respawn/projection/finish,
                // not competitive bot balance. XP enables sustainable defence;
                // withdraw the defender using only the ordinary input path.
                inputs.retain(|(player,_)| *player != defender);
                if phase == MobaMatchPhase::Playing {
                    inputs.push((defender,PlayerInput {action:Some(PlayerInputEnum::MoveTo(MoveTo {
                        target:Some(Vec2I {x:lane_length.raw() as i32,y:3000*1024}),queued:false,
                    }))}));
                }
                inputs
            }
        };
        // The convenience bot reads authority state, unlike a network player.
        // Do not admit its unit-target commands when that team cannot see the
        // target; the replica must receive the same legal input dependencies.
        let inputs: Vec<_> = inputs.into_iter().filter(|(player, input)| {
            let target = match input.action.as_ref() {
                Some(PlayerInputEnum::CastAbility(cast)) => cast.target_entity,
                Some(PlayerInputEnum::AttackTarget(attack)) => Some(attack.target_id),
                _ => None,
            };
            target.is_none_or(|id| {
                let state = authority.read_resource::<MobaMatch>();
                let side = state.config.players.iter().position(|id| id == player).unwrap();
                authority.read_resource::<TeamVisibilityRuntime>().teams[&state.config.teams[side]]
                    .index.current.contains(&canonical_entity_id(authority.entities().entity(id)))
            })
        }).collect();
        let accepted: Vec<_> = inputs.iter().map(|(player, input)| {
            use prost::Message;
            let state = authority.read_resource::<MobaMatch>();
            let side = state.config.players.iter().position(|id| id == player).unwrap();
            let actor = state.heroes[side].entity.unwrap();
            let mut sanitized = input.clone();
            let (kind, target) = match sanitized.action.as_mut().unwrap() {
                PlayerInputEnum::CastAbility(cast) => (4, cast.target_entity.take()),
                PlayerInputEnum::AttackTarget(attack) => {
                    let target = attack.target_id;
                    attack.target_id = 0;
                    (3, Some(target))
                }
                PlayerInputEnum::AttackMove(_) => (11, None),
                PlayerInputEnum::MoveTo(_) => (1, None),
                other => panic!("unhandled fixture input: {other:?}"),
            };
            CanonicalAcceptedInput::from_authoritative_acceptance(
                state.config.teams[side], *player, (driver.tick() + 1) * 2 + side as u64,
                kind, canonical_entity_id(actor), target.map(|id|
                    canonical_entity_id(authority.entities().entity(id))), sanitized.encode_to_vec())
        }).collect();
        let result = driver.step(&mut authority, inputs).unwrap();
        authority.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
        end_events += result.events.iter().filter(|event| event.topic == "game.end").count();
        project_tick(&mut authority, result);
        let frames = authority.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected = authority.write_resource::<TeamProjectionRuntime>()
            .build_team_bootstraps(driver.tick() + 1, fps, seed);
        for (team, replica, stepper) in &mut replicas {
            let frame = frames[team].frame.clone();
            assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            assert!(frame.step.as_ref().unwrap().accepted_inputs.iter()
                .all(|input| input.player_id == *team), "enemy input disclosed");
            let applied = replica.apply_frame(frame, stepper).unwrap();
            assert!(matches!(applied, FrameApplyResult::Applied { .. }),
                "team {team} tick {}: {applied:?}", driver.tick());
            assert!(stepper.filtered.world.try_fetch::<MobaMatch>().is_none());
            let expected_view = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
                &expected[team], allow.clone(), BTreeSet::new()).unwrap();
            if replica.canonical_team_hash() != expected_view.canonical_team_hash() {
                for (id, entity) in &expected_view.world().entities {
                    for (schema, bytes) in &entity.components {
                        let actual = replica.world().entities.get(id)
                            .and_then(|entity| entity.components.get(schema));
                        if actual != Some(bytes) {
                            println!("lifecycle subject {:?}", entity.components.get(&DEMO_RENDER_COMPONENT_SCHEMA_ID).and_then(|b| decode_demo_render_state(b)));
                            if *schema == DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID {
                                println!("property raw expected={:?} actual={:?}",
                                    bytes.chunks_exact(8).map(|b| i64::from_be_bytes(b.try_into().unwrap())).collect::<Vec<_>>(),
                                    actual.map(|b| b.chunks_exact(8).map(|b| i64::from_be_bytes(b.try_into().unwrap())).collect::<Vec<_>>()));
                            }
                            println!("lifecycle mismatch team={team} tick={} replica={id} schema={schema} expected={} actual={}",
                                driver.tick(), String::from_utf8_lossy(bytes),
                                actual.map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_else(|| "MISSING".into()));
                        }
                    }
                }
            }
            assert_eq!(replica.canonical_team_hash(), expected_view.canonical_team_hash(),
                "full lifecycle team {team} tick {}", driver.tick());
            applied_steps += 1;
        }
        if matches!(authority.read_resource::<MobaMatch>().phase, MobaMatchPhase::Finished { .. }) {
            let digest = single_lane_replay_digest(&authority);
            if let Some(expected) = &frozen_digest { assert_eq!(&digest, expected); }
            else { frozen_digest = Some(digest); }
            finished_ticks += 1;
            if finished_ticks >= 15 { break; }
        }
    }
    let state = authority.read_resource::<MobaMatch>();
    assert!(matches!(state.phase, MobaMatchPhase::Finished { winner: Some(0), .. }),
        "full lifecycle phase={:?} tick={} deaths={:?} respawns={:?}",state.phase,driver.tick(),
        state.heroes.iter().map(|h|h.deaths).collect::<Vec<_>>(),state.heroes.iter().map(|h|h.respawns).collect::<Vec<_>>());
    assert!(state.heroes.iter().all(|hero| hero.deaths > 0 && hero.respawns > 0));
    assert_eq!(end_events, 1);
    assert_eq!(finished_ticks, 15);
    println!("full filtered lifecycle map={map:?} seed={seed} ticks={} applied_steps={applied_steps} phase={:?} deaths={:?} respawns={:?}",
    driver.tick(), state.phase, state.heroes.iter().map(|hero| hero.deaths).collect::<Vec<_>>(),
    state.heroes.iter().map(|hero| hero.respawns).collect::<Vec<_>>());
}
