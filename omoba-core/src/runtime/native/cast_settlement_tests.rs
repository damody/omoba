use super::*;

const SKILL:&str="saika_reinforcements";
fn fixture()->(World,Entity,Vec<Entity>,Outcome) {
    let pool=crate::runtime::StateInitializer::create_thread_pool();
    let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
    world.write_resource::<Tick>().0=1;
    let mut hero=Hero::new("caster".into(),"Caster".into(),String::new());
    hero.abilities.push(SKILL.into());hero.ability_levels.insert(SKILL.into(),1);
    hero.mana_pool=Some(ManaPool::full(Fixed64::from_i32(300)).unwrap());
    let faction=Faction::new(FactionType::Player,1);
    let caster=world.create_entity().with(hero).with(Pos(SimVec2::ZERO)).with(faction.clone()).with(PlayerOwner::new(7))
        .with(CProperty {hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(100),msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
    let children:Vec<_>=(0..2).map(|_|world.entities().create()).collect();
    let mut effects=vec![Outcome::ScriptManaSpent {entity:caster,amount:Fixed64::from_i32(90),ability_id:SKILL.into()}];
    for (index,entity) in children.iter().enumerate(){
        let pos=SimVec2::new(Fixed64::from_i32(200+index as i32*200),Fixed64::ZERO);
        world.read_resource::<super::super::super::summon_navigation::PlacementReservations>().0.lock().unwrap()
            .insert(*entity,(1,pos,Fixed64::from_i32(16)));
        effects.push(Outcome::SpawnSummoned {entity:*entity,pos,unit_id:"saika_gunner".into(),owner:caster,
            faction:faction.clone(),player_owner:Some(7),duration:Fixed64::from_i32(30)});
    }
    effects.push(Outcome::ScriptStartCooldown {entity:caster,ability_id:SKILL.into(),duration:Fixed64::from_i32(10)});
    effects.push(Outcome::ScriptSetMana {entity:caster,pool:ManaPool::new(Fixed64::from_i32(210),Fixed64::from_i32(300)).unwrap()});
    let batch=Outcome::ScriptCastTransaction {caster,skill_id:SKILL.into(),rank:1,faction:Some(faction),player_owner:Some(7),
        expected_mana:Some(ManaPool::full(Fixed64::from_i32(300)).unwrap()),outcomes:effects,
        cues:vec![CastSettlementCue {target:None,skill_id:Some(SKILL.into()),rank:1,tick:1,relocation:None,area:None}],aggression_target:None};
    (world,caster,children,batch)
}
fn mana(world:&World,caster:Entity)->Fixed64 {world.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current()}
fn flush(world:&mut World,rows:Vec<Outcome>)->Vec<crate::runtime::OrderedFact> {
    world.write_resource::<Vec<Outcome>>().extend(rows);
    process_outcomes(world,&mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
    world.read_resource::<crate::runtime::ObservableFactBuffer>().drain_ordered().unwrap()
}

#[test]
fn cast_transaction_commits_entire_formation_once_with_resources_hooks_and_cue() {
    let (mut world,caster,children,batch)=fixture();
    let restored:Outcome=serde_json::from_str(&serde_json::to_string(&batch).unwrap()).unwrap();
    assert!(world.read_resource::<crate::runtime::ObservableFactBuffer>().drain_ordered().unwrap().is_empty());
    let facts=flush(&mut world,vec![restored]);
    assert_eq!(facts.iter().filter(|f|matches!(f.fact,crate::runtime::ObservableFact::Spawn {..})).count(),2);
    assert_eq!(mana(&world,caster),Fixed64::from_i32(210));
    assert_eq!(world.read_storage::<Hero>().get(caster).unwrap().get_cooldown(SKILL),Fixed64::from_i32(10));
    for child in &children {assert!(world.read_storage::<SummonedUnit>().get(*child).is_some());}
    let hooks=world.write_resource::<ScriptEventQueue>().drain();
    assert_eq!(hooks.iter().filter(|e|matches!(e,ScriptEvent::Spawn {..})).count(),2);
    assert_eq!(hooks.iter().filter(|e|matches!(e,ScriptEvent::SpentMana {..})).count(),1);
    let cues=world.write_resource::<super::super::super::scripting::ScriptVisualEventQueue>().drain();
    assert_eq!(cues.iter().filter(|e|e.kind==super::super::super::scripting::ScriptVisualEventKind::SkillCast).count(),1);
    assert!(flush(&mut world,vec![batch]).is_empty());assert_eq!(mana(&world,caster),Fixed64::from_i32(210));
    assert!(world.write_resource::<ScriptEventQueue>().drain().is_empty());
    assert!(children.iter().all(|e|world.entities().is_alive(*e)));
}

#[test]
fn cast_transaction_later_sibling_or_caster_rejection_leaks_no_spawn_mana_or_success() {
    for case in 0..12 {
        let (mut world,caster,children,mut batch)=fixture();
        match case {
            0..=2=>{if let Outcome::ScriptCastTransaction {outcomes,..}=&mut batch {
                if let Outcome::SpawnSummoned {unit_id,duration,entity,..}=&mut outcomes[2] {
                    if case==0 {*unit_id="missing_summon".into();}else if case==1 {*duration=-Fixed64::ONE;}else{let _=entity;}
                }
                if case==2 {outcomes.push(outcomes[1].clone());}
            }},
            3=>world.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::ZERO,
            4=>world.write_storage::<Faction>().get_mut(caster).unwrap().team_id=2,
            5=>world.write_storage::<PlayerOwner>().get_mut(caster).unwrap().player_id=8,
            6=>world.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool.as_mut().unwrap().spend(Fixed64::ONE).unwrap(),
            7=>{world.write_resource::<BuffStore>().add(caster,"silence",Fixed64::ONE,json!({}));},
            8=>world.write_resource::<GamePause>().is_paused=true,
            9=>{if let Outcome::ScriptCastTransaction {outcomes,..}=&mut batch {outcomes.push(Outcome::ScriptDirectDamage {target:caster,amount:Fixed64::ONE});}},
            10=>world.write_storage::<Hero>().get_mut(caster).unwrap().ability_levels.insert(SKILL.into(),2).map(|_|()).unwrap(),
            _=>world.write_storage::<Hero>().get_mut(caster).unwrap().start_cooldown(SKILL,Fixed64::from_i32(5)),
        }
        let before=mana(&world,caster);let cd=world.read_storage::<Hero>().get(caster).unwrap().get_cooldown(SKILL);
        assert!(flush(&mut world,vec![batch]).is_empty(),"case {case}");
        assert_eq!(mana(&world,caster),before,"case {case}");
        assert_eq!(world.read_storage::<Hero>().get(caster).unwrap().get_cooldown(SKILL),cd);
        assert!(children.iter().all(|e|world.read_storage::<Unit>().get(*e).is_none()));
        assert!(world.write_resource::<ScriptEventQueue>().drain().is_empty());
        assert!(world.write_resource::<super::super::super::scripting::ScriptVisualEventQueue>().drain().is_empty());
        assert!(world.read_resource::<super::super::super::summon_navigation::PlacementReservations>().0.lock().unwrap().is_empty());
    }
}

#[test]
fn cast_transaction_rejected_predecessor_cannot_charge_later_absolute_mana_write() {
    let (mut world,caster,_,mut first)=fixture();
    let mut next=first.clone();
    if let Outcome::ScriptCastTransaction {outcomes,..}=&mut first {
        if let Outcome::SpawnSummoned {unit_id,..}=&mut outcomes[2] {*unit_id="invalid_later_sibling".into();}
    }
    if let Outcome::ScriptCastTransaction {expected_mana,outcomes,cues,..}=&mut next {
        *expected_mana=Some(ManaPool::new(Fixed64::from_i32(210),Fixed64::from_i32(300)).unwrap());
        *outcomes=vec![Outcome::ScriptSetMana {entity:caster,pool:ManaPool::new(Fixed64::from_i32(180),Fixed64::from_i32(300)).unwrap()}];cues.clear();
    }
    assert!(flush(&mut world,vec![first,next.clone()]).is_empty());assert_eq!(mana(&world,caster),Fixed64::from_i32(300));
    if let Outcome::ScriptCastTransaction {expected_mana,outcomes,..}=&mut next {
        *expected_mana=Some(ManaPool::full(Fixed64::from_i32(300)).unwrap());
        *outcomes=vec![Outcome::ScriptSetMana {entity:caster,pool:ManaPool::new(Fixed64::from_i32(270),Fixed64::from_i32(300)).unwrap()}];
    }
    flush(&mut world,vec![next]);assert_eq!(mana(&world,caster),Fixed64::from_i32(270));
}

#[test]
fn cast_transaction_plans_all_friendly_siblings_before_mutation_and_is_deterministic() {
    let run=|| {
        let (mut world,_,children,mut batch)=fixture();
        if let Outcome::ScriptCastTransaction {outcomes,..}=&mut batch {
            if let Outcome::SpawnSummoned {pos,..}=&mut outcomes[2] {*pos=SimVec2::new(Fixed64::from_i32(200),Fixed64::ZERO);}
        }
        flush(&mut world,vec![batch]);
        let positions:Vec<_>=children.iter().map(|e|world.read_storage::<Pos>().get(*e).unwrap().0).collect();
        let radius=world.read_storage::<CollisionRadius>().get(children[0]).unwrap().0;
        assert!((positions[1]-positions[0]).length()>=radius+radius);positions
    };
    assert_eq!(run(),run());
}

#[test]
fn guarded_script_mana_callback_cannot_carry_a_rejected_cast_debit_or_notify() {
    use crate::comp::{ManaSettlementChange,ManaSettlementNotification};
    let (mut world,caster,_,mut batch)=fixture();
    if let Outcome::ScriptCastTransaction {outcomes,..}=&mut batch {
        if let Outcome::SpawnSummoned {unit_id,..}=&mut outcomes[2] {*unit_id="invalid_sibling".into();}
    }
    let callback=Outcome::ScriptManaCommit {
        changes:vec![ManaSettlementChange {entity:caster,
            expected:ManaPool::new(Fixed64::from_i32(210),Fixed64::from_i32(300)).unwrap(),
            pool:ManaPool::new(Fixed64::from_i32(220),Fixed64::from_i32(300)).unwrap()}],
        notifications:vec![ManaSettlementNotification::Gained {entity:caster,amount:Fixed64::from_i32(10)}],
    };
    assert!(flush(&mut world,vec![batch,callback]).is_empty());
    assert_eq!(mana(&world,caster),Fixed64::from_i32(300));
    assert!(world.write_resource::<ScriptEventQueue>().drain().is_empty());
}

#[test]
fn guarded_script_mana_commit_checks_all_owners_before_any_mutation_and_keeps_notice_order() {
    use crate::comp::{ManaSettlementChange,ManaSettlementNotification};
    let (mut world,caster,_,_)=fixture();
    let mut hero=Hero::new("other".into(),"Other".into(),String::new());
    hero.mana_pool=Some(ManaPool::new(Fixed64::from_i32(60),Fixed64::from_i32(300)).unwrap());
    let other=world.create_entity().with(hero).with(CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,
        msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
    let mut commit=Outcome::ScriptManaCommit {
        changes:vec![ManaSettlementChange {entity:caster,expected:ManaPool::full(Fixed64::from_i32(300)).unwrap(),
            pool:ManaPool::new(Fixed64::from_i32(290),Fixed64::from_i32(300)).unwrap()},
            ManaSettlementChange {entity:other,expected:ManaPool::new(Fixed64::from_i32(59),Fixed64::from_i32(300)).unwrap(),
                pool:ManaPool::new(Fixed64::from_i32(70),Fixed64::from_i32(300)).unwrap()}],
        notifications:vec![ManaSettlementNotification::Gained {entity:other,amount:Fixed64::from_i32(10)},
            ManaSettlementNotification::Spent {entity:caster,amount:Fixed64::from_i32(10),ability_id:"callback".into()}],
    };
    flush(&mut world,vec![commit.clone()]);assert_eq!(mana(&world,caster),Fixed64::from_i32(300));
    assert_eq!(mana(&world,other),Fixed64::from_i32(60));assert!(world.write_resource::<ScriptEventQueue>().drain().is_empty());
    if let Outcome::ScriptManaCommit {changes,..}=&mut commit {changes[1].expected=ManaPool::new(Fixed64::from_i32(60),Fixed64::from_i32(300)).unwrap();}
    flush(&mut world,vec![commit]);assert_eq!(mana(&world,caster),Fixed64::from_i32(290));assert_eq!(mana(&world,other),Fixed64::from_i32(70));
    let events=world.write_resource::<ScriptEventQueue>().drain();assert_eq!(events.len(),2);
    assert!(matches!(events[0],ScriptEvent::ManaGained {e,..}if e==other));
    assert!(matches!(events[1],ScriptEvent::SpentMana {caster:source,..}if source==caster));
}
