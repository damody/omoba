//! Production ECS + generated Lua manifest + formal PlayerInput acceptance.

#[test]
fn sale_undo_formal_60hz_restores_slots_without_hp_rewind_and_binds_retained_life() {
    use omoba_core::runtime::shop::{inspect_owner_undo,ShopUndoKind,ShopError,ShopUndoLedger};
    use omoba_core::game_proto::{ItemBuy,ItemSell,ShopUndo};
    let mut replays=Vec::new();
    for _ in 0..2 {
        let (mut w,mut d)=world(SingleLaneConfig{base_recovery_enabled:false,respawn_delay:Fixed64::ONE,wave_interval:Fixed64::from_i32(10000),..fast_config()},SimulationTickProfile::Production60Hz);
        w.insert(omoba_core::runtime::shop_undo_wire::LockstepHashContract{version:omoba_core::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION});
        d.step(&mut w,[]).unwrap();let [source,target]=hero_pair(&w);
        for e in [source,target] {set_hold(&mut w,e);disarm_attack(&mut w,e);w.write_resource::<BuffStore>().add(e,"no_regen",Fixed64::from_i32(100),serde_json::json!({"health_regen_percentage":-1024}));}
        w.write_storage::<Gold>().get_mut(source).unwrap().0=5000;w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::from_i32(100);
        let buy=|id:&str|PlayerInput{action:Some(PlayerInputEnum::ItemBuy(ItemBuy{item_id:id.into()}))};
        let sell=|slot|PlayerInput{action:Some(PlayerInputEnum::ItemSell(ItemSell{item_slot:slot}))};
        let undo=|sequence|PlayerInput{action:Some(PlayerInputEnum::ShopUndo(ShopUndo{mode:2,sequence}))};
        let newest=|w:&World|{let state=w.read_resource::<MobaMatch>();let a=inspect_owner_undo(w,&state,1);assert!(a.can_undo);a.newest_sequence};
        d.step(&mut w,[(1,buy("moba_guard_charm"))]).unwrap();d.step(&mut w,[(1,buy("moba_armor"))]).unwrap();
        d.step(&mut w,[(1,sell(1))]).unwrap();let sale=newest(&w);assert_eq!(w.read_storage::<Gold>().get(source).unwrap().0,4350);
        let settled=d.step(&mut w,[(1,undo(sale))]).unwrap();
        assert_eq!(w.read_resource::<PendingItemUseQueue>().undo_settlements[0].result,Ok(sale));
        assert_eq!(w.read_storage::<Gold>().get(source).unwrap().0,4200);
        assert_eq!(w.read_storage::<Inventory>().get(source).unwrap().find_item("moba_armor"),Some(1));
        assert_eq!(w.read_storage::<CProperty>().get(source).unwrap().hp,Fixed64::from_i32(100));
        assert!(settled.facts.iter().any(|f|matches!(&f.fact,ObservableFact::OwnerEconomy{state,..}if state.player_id==1&&state.economy.gold==4200&&state.economy.item_ids[1]==2)));
        d.step(&mut w,[(1,undo(sale))]).unwrap();assert_eq!(w.read_resource::<PendingItemUseQueue>().undo_settlements[0].result,Err(ShopError::UndoDuplicate));assert_eq!(w.read_storage::<Gold>().get(source).unwrap().0,4200);
        d.step(&mut w,[(1,sell(0))]).unwrap();let sold_charm=newest(&w);d.step(&mut w,[(1,buy("moba_sword"))]).unwrap();let purchased=newest(&w);
        d.step(&mut w,[(1,undo(purchased))]).unwrap();d.step(&mut w,[(1,undo(sold_charm))]).unwrap();assert_eq!(w.read_storage::<Gold>().get(source).unwrap().0,4200);
        // Death retires live records. A later roster sale is a separate admitted transaction.
        d.step(&mut w,[(1,sell(0))]).unwrap();let live_sale=newest(&w);
        w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::ZERO;let pos=w.read_storage::<Pos>().get(source).unwrap().0;
        w.write_resource::<Vec<Outcome>>().push(Outcome::Death{ent:source,pos});process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();w.maintain();
        assert!(w.read_resource::<MobaMatch>().heroes[0].entity.is_none());
        d.step(&mut w,[(1,sell(1))]).unwrap();let retained_sale=newest(&w);
        assert_ne!(retained_sale,live_sale);
        d.step(&mut w,[(1,undo(retained_sale))]).unwrap();assert_eq!(w.read_resource::<PendingItemUseQueue>().undo_settlements[0].result,Ok(retained_sale));
        assert!(w.read_resource::<MobaMatch>().heroes[0].entity.is_none(),"undo cannot revive the retired body");
        d.step(&mut w,[(1,sell(1))]).unwrap();let pending=newest(&w);
        let mut fresh=None;for _ in 0..90 {d.step(&mut w,[]).unwrap();fresh=w.read_resource::<MobaMatch>().heroes[0].entity;if fresh.is_some(){break}}
        let fresh=fresh.unwrap();assert_ne!(fresh,source);set_hold(&mut w,fresh);disarm_attack(&mut w,fresh);
        let base=w.read_resource::<MobaMatch>().bases[0].unwrap();let pos=w.read_storage::<Pos>().get(base).unwrap().0;w.write_storage::<Pos>().get_mut(fresh).unwrap().0=pos;
        w.write_resource::<BuffStore>().add(fresh,"no_regen",Fixed64::from_i32(100),serde_json::json!({"health_regen_percentage":-1024}));w.write_storage::<CProperty>().get_mut(fresh).unwrap().hp=Fixed64::from_i32(100);
        {let ledger=w.read_resource::<ShopUndoLedger>();let record=ledger.owners.get(&1).unwrap().records.last().unwrap();assert_eq!(record.kind,ShopUndoKind::Sale);assert_eq!(record.sequence,pending);assert_eq!((record.source_index,record.source_generation),(fresh.id(),fresh.gen().id()));}
        d.step(&mut w,[(1,undo(pending))]).unwrap();assert_eq!(w.read_resource::<PendingItemUseQueue>().undo_settlements[0].result,Ok(pending));
        assert_eq!(w.read_storage::<Inventory>().get(fresh).unwrap().find_item("moba_armor"),Some(1));assert_eq!(w.read_storage::<CProperty>().get(fresh).unwrap().hp,Fixed64::from_i32(100));
        assert_eq!(w.read_storage::<Gold>().get(fresh).unwrap().0,4450);
        replays.push(single_lane_replay_digest(&w));
    }
    assert_eq!(replays[0],replays[1]);
}

#[test]
fn equipment_critical_formal_60hz_recipe_undo_cap_leech_settlement_and_new_life() {
    use omoba_core::runtime::shop::{transact_moba_shop,undo_moba_shop,ShopCommand,ShopUndoRequest};
    let (mut w,mut d)=world(SingleLaneConfig{base_recovery_enabled:false,respawn_delay:Fixed64::from_raw(64),wave_interval:Fixed64::from_i32(10000),..fast_config()},SimulationTickProfile::Production60Hz);
    d.step(&mut w,[]).unwrap();let [source,target]=hero_pair(&w);
    for e in [source,target] {set_hold(&mut w,e);disarm_attack(&mut w,e);w.write_resource::<BuffStore>().add(e,"no_regen",Fixed64::from_i32(100),serde_json::json!({"health_regen_percentage":-1024}));}
    w.write_storage::<Gold>().get_mut(source).unwrap().0=30000;
    w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::from_i32(100);
    w.write_storage::<CProperty>().get_mut(target).unwrap().def_physic=Fixed64::ZERO;
    let chance=|w:&World,e|ability_runtime::UnitStats::from_refs(&w.read_resource::<BuffStore>(),false).crit_with_base(e,Fixed64::ZERO,Fixed64::from_i32(2)).0;
    let leech=|w:&World,e|ability_runtime::UnitStats::from_refs(&w.read_resource::<BuffStore>(),false).lifesteal(e);
    let buy=|w:&mut World,id:&str|transact_moba_shop(w,1,ShopCommand::Buy(id.into())).unwrap();
    buy(&mut w,"moba_precision_glove");d.step(&mut w,[]).unwrap();assert_eq!(chance(&w,source),Fixed64::from_raw(128));
    assert_eq!(w.read_storage::<Gold>().get(source).unwrap().0,29400);
    // A pure chance purchase is offensive even when it adds no flat attack.
    w.write_resource::<BuffStore>().add(source,"independent_critical",Fixed64::ONE,serde_json::json!({"preattack_criticalstrike":896}));
    let before=w.read_storage::<CProperty>().get(target).unwrap().hp;
    w.write_resource::<Vec<DamageInstance>>().push(DamageInstance::new_attack(source,target,Fixed64::from_i32(10)));
    let step=d.step(&mut w,[]).unwrap();assert!(step.facts.iter().any(|f|matches!(f.fact,ObservableFact::CriticalHit{target:t}if t==canonical_entity_id(target))));
    assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,before-Fixed64::from_i32(20));
    assert!(undo_moba_shop(&mut w,1,ShopUndoRequest::Newest).is_err(),"pure chance cannot be refunded after combat benefit");
    w.write_resource::<BuffStore>().remove(source,"independent_critical");assert_eq!(chance(&w,source),Fixed64::from_raw(128));
    buy(&mut w,"moba_sword");buy(&mut w,"moba_precision_saber");d.step(&mut w,[]).unwrap();assert_eq!(chance(&w,source),Fixed64::from_raw(256));
    undo_moba_shop(&mut w,1,ShopUndoRequest::Newest).unwrap();d.step(&mut w,[]).unwrap();assert_eq!(chance(&w,source),Fixed64::from_raw(128),"undo restores component not stale upgrade rate");
    buy(&mut w,"moba_precision_saber");
    for _ in 0..3 {buy(&mut w,"moba_precision_glove");buy(&mut w,"moba_sword");buy(&mut w,"moba_precision_saber");}
    buy(&mut w,"moba_sword");buy(&mut w,"moba_leech_blade");d.step(&mut w,[]).unwrap();
    assert_eq!(chance(&w,source),Fixed64::ONE);assert_eq!(leech(&w,source),Fixed64::from_raw(128));
    w.write_resource::<BuffStore>().add(source,"independent_critical",Fixed64::ONE,serde_json::json!({"preattack_criticalstrike":128}));assert_eq!(chance(&w,source),Fixed64::ONE,"shared total caps at100%");w.write_resource::<BuffStore>().remove(source,"independent_critical");
    let before=w.read_storage::<CProperty>().get(target).unwrap().hp;
    w.write_resource::<Vec<DamageInstance>>().push(DamageInstance::new_attack(source,target,Fixed64::from_i32(100)));
    let step=d.step(&mut w,[]).unwrap();assert_eq!(step.facts.iter().filter(|f|matches!(f.fact,ObservableFact::CriticalHit{target:t}if t==canonical_entity_id(target))).count(),1);
    assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,before-Fixed64::from_i32(200));
    assert_eq!(w.read_storage::<CProperty>().get(source).unwrap().hp,Fixed64::from_i32(125),"2x physical critical coexists with12.5% actual leech");
    assert_eq!(w.read_resource::<BuffStore>().iter_for(source).filter(|(id,_)|*id=="__item_passives").count(),1);
    transact_moba_shop(&mut w,1,ShopCommand::Sell(0)).unwrap();d.step(&mut w,[]).unwrap();assert_eq!(chance(&w,source),Fixed64::from_raw(768));assert_eq!(leech(&w,source),Fixed64::from_raw(128),"selling critical preserves lifesteal in shared lease");
    undo_moba_shop(&mut w,1,ShopUndoRequest::Newest).unwrap();d.step(&mut w,[]).unwrap();
    assert_eq!(chance(&w,source),Fixed64::ONE);assert_eq!(leech(&w,source),Fixed64::from_raw(128));
    loop {let slot=w.read_storage::<Inventory>().get(source).unwrap().slots.iter().position(Option::is_some);let Some(slot)=slot else{break};transact_moba_shop(&mut w,1,ShopCommand::Sell(slot)).unwrap();}
    d.step(&mut w,[]).unwrap();assert_eq!(chance(&w,source),Fixed64::ZERO);assert_eq!(leech(&w,source),Fixed64::ZERO);assert!(!w.read_resource::<BuffStore>().has(source,"__item_passives"));
    assert_eq!(w.read_storage::<CProperty>().get(source).unwrap().hp,Fixed64::from_i32(125),"inventory changes never heal");
    buy(&mut w,"moba_precision_glove");buy(&mut w,"moba_sword");buy(&mut w,"moba_precision_saber");d.step(&mut w,[]).unwrap();
    w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::ZERO;let pos=w.read_storage::<Pos>().get(source).unwrap().0;
    w.write_resource::<Vec<Outcome>>().push(Outcome::Death{ent:source,pos});process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();w.maintain();
    for _ in 0..8 {d.step(&mut w,[]).unwrap();}let fresh=w.read_resource::<MobaMatch>().heroes[0].entity.unwrap();assert_ne!(fresh,source);
    assert_eq!(chance(&w,fresh),Fixed64::from_raw(256));assert_eq!(chance(&w,source),Fixed64::ZERO);
}

#[test]
fn critical_feedback_formal_60hz_requires_settled_physical_hp_loss_and_valid_envelope() {
    for mode in 0..14 {
        let (mut w,mut d,source,target)=mana_cast_fixture(90);
        for e in [source,target] {set_hold(&mut w,e);disarm_attack(&mut w,e);w.write_resource::<BuffStore>().add(e,"no_regen",Fixed64::from_i32(100),serde_json::json!({"health_regen_percentage":-1024}));}
        w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::from_i32(100);
        w.write_resource::<BuffStore>().add(source,"critical",Fixed64::from_i32(100),serde_json::json!({"preattack_criticalstrike":1024,"crit_multiplier":if mode==10 {1024}else{3072}}));
        if mode==2 {w.write_resource::<BuffStore>().add(source,"miss",Fixed64::ONE,serde_json::json!({"miss_percentage":1024}));}
        if mode==3 {w.write_resource::<BuffStore>().add(target,"evade",Fixed64::ONE,serde_json::json!({"evasion_constant":1024}));}
        if mode==4||mode==8 {assert!(w.write_resource::<BuffStore>().grant_damage_shield(target,Fixed64::from_i32(if mode==4{1000}else{150}),Fixed64::from_i32(5)));}
        if mode==5 {w.write_storage::<CProperty>().get_mut(target).unwrap().hp=Fixed64::ZERO;}
        if mode==7 {w.write_resource::<BuffStore>().add(source,"leech",Fixed64::ONE,serde_json::json!({"lifesteal":512}));}
        if mode==13 {w.write_resource::<BuffStore>().add(target,"immune",Fixed64::ONE,serde_json::json!({"damage_taken_bonus":-1024}));}
        if mode==11||mode==12 {
            let inner=if mode==11 {Outcome::Heal{target,amount:Fixed64::from_i32(999),pos:omoba_sim::Vec2::ZERO}}
                else{Outcome::DamageWithCritical{damage:Box::new(Outcome::Damage{source,target,pos:omoba_sim::Vec2::ZERO,phys:Fixed64::from_i32(100),magi:Fixed64::ZERO,real:Fixed64::ZERO,damage_profile:1,predeclared:false})}};
            w.write_resource::<Vec<Outcome>>().push(Outcome::DamageWithCritical{damage:Box::new(inner)});
        }else{
            let mut hit=DamageInstance::new_attack(source,target,Fixed64::from_i32(if mode==6||mode==9{0}else{100}));hit.damage_flags.can_crit=mode!=1;
            if mode==6 {hit.damage_types.magical=Fixed64::from_i32(100);}
            w.write_resource::<Vec<DamageInstance>>().push(hit);
        }
        let step=d.step(&mut w,[]).unwrap();let count=step.facts.iter().filter(|f|matches!(f.fact,ObservableFact::CriticalHit{target:t}if t==canonical_entity_id(target))).count();
        assert_eq!(count,usize::from(matches!(mode,0|7|8|10)),"mode{mode}: exactly one actual physical critical, independent of numerical multiplier");
        let loss=[300,100,0,0,0,0,100,300,150,0,100,0,0,0][mode];
        let after=w.read_storage::<CProperty>().get(target).map(|p|p.hp).unwrap_or(Fixed64::ZERO);
        assert_eq!(after,Fixed64::from_i32(if mode==5{0}else{10000-loss}),"mode{mode}: actual admitted HP");
        assert_eq!(w.read_storage::<CProperty>().get(source).unwrap().hp,Fixed64::from_i32(if mode==7{250}else{100}));
    }
}

#[test]
fn equipment_leech_formal_60hz_shop_recipe_undo_sale_actual_healing_and_new_life() {
    use omoba_core::runtime::shop::{transact_moba_shop,undo_moba_shop,ShopCommand,ShopUndoRequest};
    let (mut w,mut d)=world(SingleLaneConfig{base_recovery_enabled:false,respawn_delay:Fixed64::from_raw(64),wave_interval:Fixed64::from_i32(10000),..fast_config()},SimulationTickProfile::Production60Hz);
    d.step(&mut w,[]).unwrap();let [source,target]=hero_pair(&w);
    for e in [source,target] {set_hold(&mut w,e);disarm_attack(&mut w,e);w.write_resource::<BuffStore>().add(e,"no_regen",Fixed64::from_i32(100),serde_json::json!({"health_regen_percentage":-1024}));}
    w.write_storage::<Gold>().get_mut(source).unwrap().0=10000;
    w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::from_i32(100);
    w.write_storage::<CProperty>().get_mut(target).unwrap().def_physic=Fixed64::ZERO;
    let rate=|w:&World,e|ability_runtime::UnitStats::from_refs(&w.read_resource::<BuffStore>(),false).lifesteal(e);
    let blade=|w:&mut World| {transact_moba_shop(w,1,ShopCommand::Buy("moba_sword".into())).unwrap();transact_moba_shop(w,1,ShopCommand::Buy("moba_leech_blade".into())).unwrap();};
    blade(&mut w);d.step(&mut w,[]).unwrap();
    assert_eq!(rate(&w,source),Fixed64::from_raw(128));assert_eq!(w.read_storage::<Gold>().get(source).unwrap().0,9100);
    assert_eq!(w.read_storage::<CProperty>().get(source).unwrap().hp,Fixed64::from_i32(100),"buy never heals existing body");
    let mut hit=DamageInstance::new_attack(source,target,Fixed64::from_i32(100));hit.damage_flags.can_crit=false;
    w.write_resource::<Vec<DamageInstance>>().push(hit);d.step(&mut w,[]).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(source).unwrap().hp,Fixed64::from_raw(115200),"100+12.5 actual physical leech");
    assert!(undo_moba_shop(&mut w,1,ShopUndoRequest::Newest).is_err(),"equal-attack recipe cannot refund after actual lifesteal benefit");
    assert_eq!(w.read_storage::<Gold>().get(source).unwrap().0,9100,"rejected refund preserves wallet");
    assert_eq!(rate(&w,source),Fixed64::from_raw(128));
    blade(&mut w);d.step(&mut w,[]).unwrap();assert_eq!(rate(&w,source),Fixed64::from_raw(256));
    transact_moba_shop(&mut w,1,ShopCommand::Buy("moba_sword".into())).unwrap();
    transact_moba_shop(&mut w,1,ShopCommand::Buy("moba_leech_saber".into())).unwrap();d.step(&mut w,[]).unwrap();assert_eq!(rate(&w,source),Fixed64::from_raw(384),"recipe consumes one component, leaves one, adds upgrade");
    w.write_resource::<BuffStore>().add(source,"independent_transform",Fixed64::ONE,serde_json::json!({"lifesteal":128}));assert_eq!(rate(&w,source),Fixed64::from_raw(512));w.write_resource::<BuffStore>().remove(source,"independent_transform");
    undo_moba_shop(&mut w,1,ShopUndoRequest::Newest).unwrap();d.step(&mut w,[]).unwrap();assert_eq!(rate(&w,source),Fixed64::from_raw(256));
    transact_moba_shop(&mut w,1,ShopCommand::Sell(0)).unwrap();d.step(&mut w,[]).unwrap();assert_eq!(rate(&w,source),Fixed64::from_raw(128));
    undo_moba_shop(&mut w,1,ShopUndoRequest::Newest).unwrap();
    d.step(&mut w,[]).unwrap();assert_eq!(rate(&w,source),Fixed64::from_raw(256),"sale undo restores sold equipment through dirty rebuild");
    loop {let slot=w.read_storage::<Inventory>().get(source).unwrap().slots.iter().position(Option::is_some);let Some(slot)=slot else{break};transact_moba_shop(&mut w,1,ShopCommand::Sell(slot)).unwrap();}
    d.step(&mut w,[]).unwrap();assert_eq!(rate(&w,source),Fixed64::ZERO);assert!(!w.read_resource::<BuffStore>().has(source,"__item_passives"));
    assert_eq!(w.read_storage::<CProperty>().get(source).unwrap().hp,Fixed64::from_raw(115200),"sale/undo never restore health");
    blade(&mut w);transact_moba_shop(&mut w,1,ShopCommand::Buy("moba_sword".into())).unwrap();
    transact_moba_shop(&mut w,1,ShopCommand::Buy("moba_leech_saber".into())).unwrap();d.step(&mut w,[]).unwrap();
    assert_eq!(rate(&w,source),Fixed64::from_raw(256));assert_eq!(w.read_resource::<BuffStore>().iter_for(source).filter(|(id,_)|*id=="__item_passives").count(),1);
    w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::ZERO;let pos=w.read_storage::<Pos>().get(source).unwrap().0;
    w.write_resource::<Vec<Outcome>>().push(Outcome::Death{ent:source,pos});process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();w.maintain();
    for _ in 0..8 {d.step(&mut w,[]).unwrap();}let fresh=w.read_resource::<MobaMatch>().heroes[0].entity.unwrap();assert_ne!(fresh,source);
    assert_eq!(rate(&w,fresh),Fixed64::from_raw(256),"retained equipment restored by production new-life finalization");assert_eq!(rate(&w,source),Fixed64::ZERO);
}

#[test]
fn shared_native_attack_leech_keeps_ability_opt_out_and_explicit_rate_single() {
    for mode in 0..11 {
        let (mut w,mut d,source,target)=mana_cast_fixture(90);
        for e in [source,target] {set_hold(&mut w,e);disarm_attack(&mut w,e);w.write_resource::<BuffStore>().add(e,"no_regen",Fixed64::from_i32(100),serde_json::json!({"health_regen_percentage":-1024}));}
        w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::from_i32(100);
        w.write_resource::<BuffStore>().add(source,"leech",Fixed64::from_i32(100),serde_json::json!({"lifesteal":512}));
        if mode==5||mode==6 {w.write_storage::<IsBuilding>().insert(target,IsBuilding).unwrap();}
        if mode==7 {let own=w.read_storage::<Faction>().get(source).unwrap().clone();*w.write_storage::<Faction>().get_mut(target).unwrap()=own;}
        if mode==8 {*w.write_storage::<Faction>().get_mut(target).unwrap()=Faction::new(FactionType::HostileNeutral,-1);}
        if mode==9 {*w.write_storage::<Faction>().get_mut(target).unwrap()=Faction::new(FactionType::Ally,3);}
        if mode==10 {*w.write_storage::<Faction>().get_mut(target).unwrap()=Faction::new(FactionType::Neutral,-1);}
        let mut hit=DamageInstance::new_attack(source,target,Fixed64::from_i32(100));hit.damage_flags.can_crit=false;
        if mode==1||mode==4 {hit.source.source_type=DamageSourceType::Ability;}
        if mode==2||mode==4||mode==6 {hit.damage_flags.lifesteal=Fixed64::from_raw(512);}
        if mode==3 {hit.damage_flags.lifesteal=Fixed64::ONE;}
        w.write_resource::<Vec<DamageInstance>>().push(hit);d.step(&mut w,[]).unwrap();
        assert_eq!(w.read_storage::<CProperty>().get(source).unwrap().hp,Fixed64::from_i32(100+[50,0,50,100,50,0,50,0,50,0,0][mode]),"mode{mode}: explicit and shared target/rate policies remain independent");
    }
}

#[test]
fn hero_projectile_lifesteal_formal_60hz_freezes_launch_and_settles_actual_hp_loss() {
    for mode in 0..14 {
        let (mut w,mut d,source,target)=mana_cast_fixture(90);
        w.write_storage::<Hero>().get_mut(source).unwrap().experience_to_next=i32::MAX;
        for e in [source,target] {set_hold(&mut w,e);disarm_attack(&mut w,e);w.write_resource::<BuffStore>().add(e,"no_regen",Fixed64::from_i32(100),serde_json::json!({"health_regen_percentage":-1024}));}
        w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::from_i32(100);
        {let mut a=w.write_storage::<TAttack>();let a=a.get_mut(source).unwrap();a.atk_physic.v=Fixed64::from_i32(100);a.bullet_speed=Fixed64::from_i32(300);a.asd.v=Fixed64::ONE;a.asd_count=a.asd.v;}
        {let mut b=w.write_resource::<BuffStore>();b.add(source,"visual",Fixed64::from_i32(100),serde_json::json!({"multi_shot_visual":3072}));
            if mode!=1 {b.add(source,"leech",Fixed64::from_i32(100),serde_json::json!({"lifesteal":if mode==8 {-512}else if mode==9 {2048}else{512}}));}
            if mode==4 {b.add(source,"miss",Fixed64::from_i32(100),serde_json::json!({"miss_percentage":1024}));}}
        run_committed_visibility_wave_b(&mut w,d.tick(),0);
        let input=(1,PlayerInput {action:Some(PlayerInputEnum::AttackTarget(AttackTarget {target_id:target.id(),queued:false}))});let mut shots=Vec::new();
        for step in 0..90 {d.step(&mut w,if step==0 {vec![input.clone()]}else{vec![]}).unwrap();shots=(&w.entities(),&w.read_storage::<Projectile>()).join().filter(|(_,p)|p.owner==source).map(|(e,p)|(e,p.clone())).collect();if !shots.is_empty(){break;}}
        assert_eq!(shots.len(),3);let rate=if matches!(mode,1|4|8){Fixed64::ZERO}else if mode==9 {Fixed64::ONE}else{Fixed64::from_raw(512)};
        {let origins=w.read_resource::<ProjectileOrigins>();for (e,p) in &shots {assert_eq!(origins.0[e].attack_lifesteal,if p.target.is_some(){rate}else{Fixed64::ZERO});}}
        set_hold(&mut w,source);disarm_attack(&mut w,source);
        if mode==10 {w.write_storage::<IsBuilding>().insert(target,IsBuilding).unwrap();}
        if mode==11 {let own=w.read_storage::<Faction>().get(source).unwrap().clone();*w.write_storage::<Faction>().get_mut(target).unwrap()=own;}
        if mode==12 {*w.write_storage::<Faction>().get_mut(target).unwrap()=Faction::new(FactionType::Neutral,-1);}
        if mode==13 {*w.write_storage::<Faction>().get_mut(target).unwrap()=Faction::new(FactionType::HostileNeutral,-1);}
        {let mut b=w.write_resource::<BuffStore>();b.remove(source,"leech");b.add(source,"late_leech",Fixed64::from_i32(100),serde_json::json!({"lifesteal":1024}));
            assert!(b.grant_damage_shield(target,Fixed64::from_i32(40),Fixed64::from_i32(5)));
            if mode==2 {b.add(source,"cut",Fixed64::from_i32(100),serde_json::json!({"heal_received_multiplier":512}));}
            if mode==3 {b.add(target,"evade",Fixed64::from_i32(100),serde_json::json!({"evasion_constant":1024}));}}
        if mode==5 {w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::ZERO;}
        if mode==6 {let pos=w.read_storage::<Pos>().get(source).unwrap().0;w.write_resource::<Vec<Outcome>>().push(Outcome::Death {ent:source,pos});}
        if mode==7 {w.write_storage::<CProperty>().get_mut(target).unwrap().hp=Fixed64::from_i32(30);}
        for _ in 0..90 {d.step(&mut w,[]).unwrap();}
        if !matches!(mode,5|6) {assert_eq!(w.read_storage::<CProperty>().get(source).unwrap().hp,Fixed64::from_i32(100+[30,0,15,0,0,0,0,15,0,60,0,0,0,30][mode]),"mode{mode}: current eligibility and committed physical HP-loss leech");}
        if mode==5 {assert!(w.read_storage::<CProperty>().get(source).is_none_or(|p|p.hp<=Fixed64::ZERO));}
        if mode==6 {assert!(!w.entities().is_alive(source),"formal death retires the exact original source");}
        assert!(shots.iter().all(|(e,_)|!w.entities().is_alive(*e)&&!w.read_resource::<ProjectileOrigins>().0.contains_key(e)));
    }
}

#[test]
fn native_leech_formal_60hz_tracks_post_shield_overkill_immunity_and_healing_cut() {
    for mode in 0..5 {
        let (mut w,mut driver,source,target)=mana_cast_fixture(90);
        w.write_storage::<Hero>().get_mut(source).unwrap().experience_to_next=i32::MAX;
        for e in [source,target] {set_hold(&mut w,e);disarm_attack(&mut w,e);w.write_resource::<BuffStore>().add(e,"no_regen",Fixed64::from_i32(100),serde_json::json!({"health_regen_percentage":-1024}));}
        {let mut p=w.write_storage::<CProperty>();p.get_mut(source).unwrap().hp=Fixed64::from_i32(100);p.get_mut(target).unwrap().hp=Fixed64::from_i32(if mode==1 {30}else{1000});}
        if matches!(mode,0|3) {assert!(w.write_resource::<BuffStore>().grant_damage_shield(target,Fixed64::from_i32(40),Fixed64::from_i32(5)));
            assert_eq!(w.read_resource::<BuffStore>().shield_remaining(target),Fixed64::from_i32(40));}
        if mode==2 {w.write_resource::<BuffStore>().add(target,"immune",Fixed64::from_i32(100),serde_json::json!({"damage_taken_bonus":-1024}));}
        if mode==3 {w.write_resource::<BuffStore>().add(source,"cut",Fixed64::from_i32(100),serde_json::json!({"heal_received_multiplier":512}));}
        if mode==4 {w.write_resource::<BuffStore>().add(target,"evade",Fixed64::from_i32(100),serde_json::json!({"evasion_constant":1024}));}
        let mut hit=DamageInstance::new_attack(source,target,Fixed64::from_i32(100));hit.damage_flags.can_crit=false;hit.damage_flags.lifesteal=Fixed64::ONE;
        w.write_resource::<Vec<DamageInstance>>().push(hit);let result=driver.step(&mut w,[]).unwrap();
        let expected=Fixed64::from_i32(100+[60,30,0,30,0][mode]);assert_eq!(w.read_storage::<CProperty>().get(source).unwrap().hp,expected,"mode{mode}: actual settled leech");
        if matches!(mode,0|1|3) {assert!(result.facts.iter().any(|f|matches!(f.fact,ObservableFact::CommittedVitals {source:s,hp_raw,..}if s==canonical_entity_id(source)&&hp_raw==expected.raw())),"source committed healing reaches presentation facts");}
    }
}

#[test]
fn hero_shared_critical_formal_60hz_commits_amount_once_before_visual_pellets() {
    for early in [false,true] {
        let (mut w,mut d,source,target)=mana_cast_fixture(90);set_hold(&mut w,target);disarm_attack(&mut w,target);
        {let mut attacks=w.write_storage::<TAttack>();let a=attacks.get_mut(source).unwrap();
            a.atk_physic.v=Fixed64::from_i32(100);a.bullet_speed=Fixed64::from_i32(300);a.asd.v=Fixed64::ONE;a.asd_count=a.asd.v;}
        {let mut buffs=w.write_resource::<BuffStore>();buffs.add(source,"visuals",Fixed64::from_i32(100),serde_json::json!({"multi_shot_visual":3072}));
            if early {buffs.add(source,"critical",Fixed64::from_i32(100),serde_json::json!({"preattack_criticalstrike":1024,"crit_multiplier":3072}));}
            assert!(buffs.arm_next_attack_bonus(source,Fixed64::from_i32(20)));}
        run_committed_visibility_wave_b(&mut w,d.tick(),0);let before=w.read_storage::<CProperty>().get(target).unwrap().hp;
        let input=(1,PlayerInput {action:Some(PlayerInputEnum::AttackTarget(AttackTarget {target_id:target.id(),queued:false}))});let mut shots=Vec::new();let mut criticals=0;
        for step in 0..90 {let result=d.step(&mut w,if step==0 {vec![input.clone()]}else{vec![]}).unwrap();criticals+=result.facts.iter().filter(|f|matches!(f.fact,ObservableFact::CriticalHit{target:t}if t==canonical_entity_id(target))).count();
            shots=(&w.entities(),&w.read_storage::<Projectile>()).join().filter(|(_,p)|p.owner==source).map(|(e,p)|(e,p.clone())).collect();if !shots.is_empty(){break;}}
        let expected=Fixed64::from_i32(if early {360}else{120});assert_eq!(shots.len(),3);
        for(e,p)in &shots {assert_eq!(w.read_resource::<ProjectileOrigins>().0[e].attack_critical,early&&p.damage_phys>Fixed64::ZERO,"only the primary launch owns frozen critical provenance");}
        assert_eq!(shots.iter().filter(|(_,p)|p.damage_phys>Fixed64::ZERO).count(),1);
        assert_eq!(shots.iter().fold(Fixed64::ZERO,|s,(_,p)|s+p.damage_phys),expected);
        assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(source),Fixed64::ZERO);
        {let mut buffs=w.write_resource::<BuffStore>();buffs.remove(source,"critical");if !early {buffs.add(source,"late_critical",Fixed64::from_i32(100),serde_json::json!({"preattack_criticalstrike":1024,"crit_multiplier":3072}));}}
        set_hold(&mut w,source);disarm_attack(&mut w,source);for _ in 0..90 {let result=d.step(&mut w,[]).unwrap();criticals+=result.facts.iter().filter(|f|matches!(f.fact,ObservableFact::CriticalHit{target:t}if t==canonical_entity_id(target))).count();}
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,before-expected);
        assert!(shots.iter().all(|(e,_)|!w.entities().is_alive(*e)));assert_eq!(criticals,usize::from(early),"one frozen primary critical, never visual copies or late modifier reroll");
    }
}

#[test]
fn script_attack_accuracy_formal_60hz_commits_miss_samples_evasion_and_retires_after_source() {
    for mode in 0..9 {
        let (mut w,mut driver)=world(SingleLaneConfig {heroes:["saika_magoichi".into(),"training_luminary".into()],
            wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
        let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
        w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
        w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO);
        set_hold(&mut w,caster);set_hold(&mut w,enemy);disarm_attack(&mut w,caster);disarm_attack(&mut w,enemy);
        {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(enemy).unwrap();p.hp=Fixed64::from_i32(10000);p.mhp=p.hp;p.def_physic=Fixed64::ZERO;}
        {let mut units=w.write_storage::<Unit>();units.get_mut(enemy).unwrap().base_armor=Fixed64::ZERO;}
        driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index:1,target_entity:None,target_pos:Some(Vec2I {x:origin.x.raw() as i32,y:origin.y.raw() as i32})}))})]).unwrap();
        let mut summons=Vec::new();for _ in 0..30 {driver.step(&mut w,[]).unwrap();
            summons=(&w.entities(),&w.read_storage::<SummonedUnit>()).join().map(|(e,_)|e).collect();if !summons.is_empty(){break;}}
        assert_eq!(summons.len(),2);
        {let mut buffs=w.write_resource::<BuffStore>();
            for e in &summons {buffs.add(*e,"leech",Fixed64::from_i32(100),serde_json::json!({"lifesteal":512}));}
            if mode==7 {for e in &summons {buffs.add(*e,"critical",Fixed64::from_i32(100),serde_json::json!({"preattack_criticalstrike":1024,"crit_multiplier":3072}));}}
            if mode==0 {for e in &summons {buffs.add(*e,"miss_fixture",Fixed64::from_i32(100),serde_json::json!({"miss_percentage":1024}));}}
            if matches!(mode,1|3|4) {buffs.add(enemy,"evade_fixture",Fixed64::from_i32(100),serde_json::json!({"evasion_constant":1024}));}}
        run_committed_visibility_wave_b(&mut w,driver.tick(),0);
        let before=w.read_storage::<CProperty>().get(enemy).unwrap().hp;let mut shots=Vec::new();
        for _ in 0..60 {driver.step(&mut w,[]).unwrap();shots=(&w.entities(),&w.read_storage::<Projectile>()).join()
            .filter(|(_,p)|summons.contains(&p.owner)).map(|(e,p)|(e,p.clone())).collect();if !shots.is_empty(){break;}}
        assert_eq!(shots.len(),2,"mode{mode}: actual summon ordinary shots");
        let amount=shots.iter().fold(Fixed64::ZERO,|sum,(_,p)|sum+p.damage_phys);
        assert_eq!(amount>Fixed64::ZERO,mode!=0);
        if matches!(mode,7|8) {for (_,p) in &shots {
            let base=w.read_storage::<TAttack>().get(p.owner).unwrap().atk_physic.v;
            let expected=UnitStats::from_refs(&w.read_resource::<BuffStore>(),false).final_atk(base,p.owner)*Fixed64::from_i32(if mode==7 {3}else{1});
            assert_eq!(p.damage_phys,expected,"script ordinary shot shared critical committed at launch");
        }}
        for (shot,_) in &shots {let origins=w.read_resource::<ProjectileOrigins>();let o=&origins.0[shot];
            assert!(!o.normal_attack,"script accuracy never grants hero normal-hit passives");assert_eq!(o.script_attack,mode!=0);assert_eq!(o.attack_missed,mode==0);assert_eq!(o.attack_critical,mode==7);
            assert_eq!(o.attack_lifesteal,if mode==0{Fixed64::ZERO}else{Fixed64::from_raw(512)},"typed ordinary summon launch captures shared leech");}
        {let mut buffs=w.write_resource::<BuffStore>();for e in &summons {buffs.remove(*e,"miss_fixture");
            buffs.remove(*e,"critical");if mode==8 {buffs.add(*e,"late_critical",Fixed64::from_i32(100),serde_json::json!({"preattack_criticalstrike":1024,"crit_multiplier":3072}));}
            if mode==5 {buffs.add(*e,"late_miss",Fixed64::from_i32(100),serde_json::json!({"miss_percentage":1024}));}}
            if mode==3 {buffs.remove(enemy,"evade_fixture");}
            if matches!(mode,2|6) {buffs.add(enemy,"evade_fixture",Fixed64::from_i32(100),serde_json::json!({"evasion_constant":1024}));}
            if mode==4 {buffs.add(enemy,"counter_fixture",Fixed64::from_i32(100),serde_json::json!({"negative_evasion_constant":1024}));}}
        if mode==6 {w.write_resource::<GamePause>().is_paused=true;for _ in 0..3 {driver.step(&mut w,[]).unwrap();}
            assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,before);w.write_resource::<GamePause>().is_paused=false;}
        for summon in &summons {w.write_storage::<SummonedUnit>().get_mut(*summon).unwrap().time_remaining=Some(Fixed64::from_raw(1));}
        driver.step(&mut w,[]).unwrap();assert!(summons.iter().all(|e|!w.entities().is_alive(*e)));
        let mut avoided=0;let mut criticals=0;for _ in 0..50 {let step=driver.step(&mut w,[]).unwrap();criticals+=step.facts.iter().filter(|f|matches!(f.fact,ObservableFact::CriticalHit{target}if target==canonical_entity_id(enemy))).count();avoided+=step.facts.iter().filter(|f|matches!(f.fact,ObservableFact::AttackAvoided {target}if target==canonical_entity_id(enemy))).count();}
        assert_eq!(avoided,if matches!(mode,0|1|2|6){2}else{0},"authority target-local avoidance only");assert_eq!(criticals,if mode==7{2}else{0},"typed ordinary critical retained across source retirement, no late reroll or missed impact");
        assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,before-if matches!(mode,0|1|2|6){Fixed64::ZERO}else{amount},"mode{mode}: actual impact");
        assert!(shots.iter().all(|(e,_)|!w.entities().is_alive(*e)&&!w.read_resource::<ProjectileOrigins>().0.contains_key(e)));
    }
}

#[test]
fn primary_projectile_evasion_formal_60hz_samples_impact_target_state_and_retires() {
    for mode in 0..7 {
        let (mut w,mut driver,source,target)=mana_cast_fixture(90);set_hold(&mut w,target);disarm_attack(&mut w,target);
        {let mut attacks=w.write_storage::<TAttack>();let a=attacks.get_mut(source).unwrap();a.atk_physic.v=Fixed64::from_i32(100);
            a.bullet_speed=Fixed64::from_i32(300);a.asd.v=Fixed64::ONE;a.asd_count=a.asd.v;}
        {let mut buffs=w.write_resource::<BuffStore>();buffs.add(source,"stun_fixture",Fixed64::from_i32(100),
            serde_json::json!({"attack_stun_chance":1.0,"attack_stun_duration":3.0,"multi_shot_visual":3072}));
            assert!(buffs.arm_next_attack_bonus(source,Fixed64::from_i32(20)));
            if matches!(mode,1|2) {buffs.add(target,"evade_fixture",Fixed64::from_i32(100),serde_json::json!({"evasion_constant":1024}));}}
        run_committed_visibility_wave_b(&mut w,driver.tick(),0);let before=w.read_storage::<CProperty>().get(target).unwrap().hp;
        let input=(1,PlayerInput {action:Some(PlayerInputEnum::AttackTarget(AttackTarget {target_id:target.id(),queued:false}))});let mut shots=Vec::new();
        for step in 0..90 {driver.step(&mut w,if step==0 {vec![input.clone()]}else{vec![]}).unwrap();
            shots=(&w.entities(),&w.read_storage::<Projectile>()).join().filter(|(_,p)|p.owner==source).map(|(e,p)|(e,p.clone())).collect();if !shots.is_empty(){break;}}
        assert_eq!(shots.len(),3);assert_eq!(shots.iter().filter(|(_,p)|p.damage_phys>Fixed64::ZERO).count(),1);
        assert_eq!(shots.iter().fold(Fixed64::ZERO,|sum,(_,p)|sum+p.damage_phys),Fixed64::from_i32(120),"launch never samples target evasion");
        assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(source),Fixed64::ZERO);
        {let mut buffs=w.write_resource::<BuffStore>();if mode==1 {buffs.remove(target,"evade_fixture");}
            if matches!(mode,0|3|5|6) {buffs.add(target,"evade_fixture",if mode==5 {Fixed64::from_raw(1)}else{Fixed64::from_i32(100)},serde_json::json!({"evasion_constant":1024}));}
            if mode==3 {buffs.add(target,"counter_fixture",Fixed64::from_i32(100),serde_json::json!({"negative_evasion_constant":1024}));}}
        set_hold(&mut w,source);disarm_attack(&mut w,source);
        if mode==6 {let remaining=w.read_resource::<BuffStore>().get(target,"evade_fixture").unwrap().remaining;
            w.write_resource::<GamePause>().is_paused=true;for _ in 0..3 {driver.step(&mut w,[]).unwrap();}
            assert_eq!(w.read_resource::<BuffStore>().get(target,"evade_fixture").unwrap().remaining,remaining);
            assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,before);w.write_resource::<GamePause>().is_paused=false;}
        for _ in 0..90 {driver.step(&mut w,[]).unwrap();}
        let evaded=matches!(mode,0|2|6);
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,before-if evaded{Fixed64::ZERO}else{Fixed64::from_i32(120)},"mode{mode}: current impact target");
        assert_eq!(w.read_resource::<BuffStore>().is_stunned(target),!evaded,"mode{mode}: no evaded hit stun");
        assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(source),Fixed64::ZERO,"evaded attack does not refund consumed bonus");
        assert!(shots.iter().all(|(e,_)|!w.entities().is_alive(*e)),"all real/visual shots retire");
    }
}

#[test]
fn hero_attack_misses_formal_60hz_clear_stun_consume_once_and_freeze_launch_decisions() {
    for mode in 0..5 {
        let (mut w,mut driver,source,target)=mana_cast_fixture(90);
        set_hold(&mut w,target);disarm_attack(&mut w,target);
        {let mut attacks=w.write_storage::<TAttack>();let a=attacks.get_mut(source).unwrap();
            a.atk_physic.v=Fixed64::from_i32(100);a.bullet_speed=Fixed64::from_i32(300);
            a.asd.v=Fixed64::ONE;a.asd_count=a.asd.v;}
        {let mut buffs=w.write_resource::<BuffStore>();
            buffs.add(source,"stun_fixture",Fixed64::from_i32(100),serde_json::json!({"attack_stun_chance":1.0,"attack_stun_duration":3.0,"multi_shot_visual":3072}));
            if matches!(mode,0|2) {buffs.add(source,"miss_fixture",Fixed64::from_i32(100),serde_json::json!({"miss_percentage":1024}));}
            if matches!(mode,1|2) {buffs.add(source,"legacy_accuracy_fixture",Fixed64::from_i32(100),serde_json::json!({"accuracy_bonus":-1024}));}
            assert!(buffs.arm_next_attack_bonus(source,Fixed64::from_i32(20)));}
        run_committed_visibility_wave_b(&mut w,driver.tick(),0);
        let before=w.read_storage::<CProperty>().get(target).unwrap().hp;
        let input=(1,PlayerInput {action:Some(PlayerInputEnum::AttackTarget(AttackTarget {target_id:target.id(),queued:false}))});
        let mut launched=Vec::new();
        for step in 0..90 {driver.step(&mut w,if step==0 {vec![input.clone()]}else{vec![]}).unwrap();
            launched=(&w.entities(),&w.read_storage::<Projectile>()).join().filter(|(_,p)|p.owner==source).map(|(e,p)|(e,p.clone())).collect();
            if !launched.is_empty(){break;}}
        assert_eq!(launched.len(),3,"mode{mode}: real native commitment emits one shot and two visual pellets");
        let missed=mode<3;let damage:Fixed64=launched.iter().fold(Fixed64::ZERO,|sum,(_,p)|sum+p.damage_phys);
        assert_eq!(damage,if missed{Fixed64::ZERO}else{Fixed64::from_i32(120)});
        assert_eq!(launched.iter().filter(|(_,p)|p.stun_duration>Fixed64::ZERO).count(),if missed{0}else{1});
        for (shot,p) in &launched {assert_eq!(w.read_resource::<ProjectileOrigins>().0[shot].attack_missed,missed&&p.target.is_some(),"only primary launch result, never visual copies");}
        assert_eq!(w.read_resource::<ProjectileOrigins>().0.values().filter(|o|o.source==source&&o.normal_attack).count(),if missed{0}else{1});
        assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(source),Fixed64::ZERO,"attempt consumes once including misses");
        {let mut buffs=w.write_resource::<BuffStore>();buffs.remove(source,"miss_fixture");buffs.remove(source,"legacy_accuracy_fixture");
            if mode==3 {buffs.add(source,"late_miss_fixture",Fixed64::from_i32(100),serde_json::json!({"miss_percentage":1024}));}}
        set_hold(&mut w,source);disarm_attack(&mut w,source);
        let mut avoided=0;for _ in 0..90 {let step=driver.step(&mut w,[]).unwrap();avoided+=step.facts.iter().filter(|f|matches!(f.fact,ObservableFact::AttackAvoided {target:t}if t==canonical_entity_id(target))).count();}
        assert_eq!(avoided,usize::from(missed),"one committed source miss, no visual or late-buff rewrite");
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,before-damage,"mode{mode}: no post-launch accuracy rewrite");
        assert_eq!(w.read_resource::<BuffStore>().is_stunned(target),!missed,"mode{mode}: only admitted hits can attach stun");
        assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(source),Fixed64::ZERO);
        assert!(launched.iter().all(|(e,_)|!w.entities().is_alive(*e)),"all real/visual shots retire");
    }
}

#[test]
fn native_lane_accuracy_formal_60hz_avoids_then_restores_real_ai_attacks() {
    for mode in 0..3 {
        let (mut w,mut driver)=world(SingleLaneConfig {creeps_per_wave:1,wave_interval:Fixed64::from_i32(10000),
            base_recovery_enabled:false,..fast_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let source=lane_creeps(&w,1)[0];let target=lane_creeps(&w,2)[0];
        let origin=omoba_sim::Vec2::new(Fixed64::from_i32(10000),Fixed64::from_i32(10000));
        {let mut positions=w.write_storage::<Pos>();for (e,p)in(&w.entities(),&mut positions).join(){
            p.0=omoba_sim::Vec2::new(Fixed64::from_i32(-50000-e.id() as i32*3000),Fixed64::from_i32(-50000));}
            positions.get_mut(source).unwrap().0=origin;positions.get_mut(target).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::ONE,Fixed64::ZERO);}
        {let mut props=w.write_storage::<CProperty>();for p in (&mut props).join(){p.msd=Fixed64::ZERO;}}
        for entity in [source,target] {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(entity).unwrap();
            p.hp=Fixed64::from_i32(10000);p.mhp=p.hp;p.msd=Fixed64::ZERO;p.def_physic=Fixed64::ZERO;
            let mut units=w.write_storage::<Unit>();units.get_mut(entity).unwrap().base_armor=Fixed64::ZERO;}
        {let mut buffs=w.write_resource::<BuffStore>();
            if mode!=1 {buffs.add(target,"evasion_fixture",Fixed64::from_i32(100),serde_json::json!({"evasion_constant":1024}));}
            if mode==1 {buffs.add(source,"miss_fixture",Fixed64::from_i32(100),serde_json::json!({"miss_percentage":1024}));}
            if mode==2 {buffs.add(target,"counter_fixture",Fixed64::from_i32(100),serde_json::json!({"negative_evasion_constant":1024}));}}
        run_committed_visibility_wave_b(&mut w,driver.tick(),0);
        let before=w.read_storage::<CProperty>().get(target).unwrap().hp;
        let mut avoidance=0;for _ in 0..90 {let step=driver.step(&mut w,[]).unwrap();avoidance+=step.facts.iter()
            .filter(|f|matches!(f.fact,ObservableFact::AttackAvoided {target:t}if t==canonical_entity_id(target))).count();}
        assert_eq!(avoidance>0,mode!=2,"actual native DamageInstance avoidance feedback");
        let protected=w.read_storage::<CProperty>().get(target).unwrap().hp;
        assert_eq!(protected==before,mode!=2,"mode{mode}: real lane AI accuracy");
        {let mut buffs=w.write_resource::<BuffStore>();buffs.remove(target,"evasion_fixture");buffs.remove(target,"counter_fixture");buffs.remove(source,"miss_fixture");}
        for _ in 0..90 {driver.step(&mut w,[]).unwrap();}
        assert!(w.read_storage::<CProperty>().get(target).unwrap().hp<protected,"mode{mode}: attacks resume after removal");
    }
}

use omoba_core::runtime::*;
use omoba_sim::Fixed64;
use specs::{Builder, Join, World, WorldExt};

#[test]
fn effective_mana_cost_formal_60hz_facts_share_checked_authority_and_team_scope() {
    use swept_dash_formal::setup_hero_skill;
    for (value,expected) in [(serde_json::json!(-512),23040),(serde_json::json!(1024),92160),
        (serde_json::json!(-1024),0),(serde_json::json!(i64::MAX),-2)] {
        let (mut w,mut d,e,_)=setup_hero_skill(1,"ranger_patch","training_ranger");
        w.write_resource::<BuffStore>().add(e,"cost_public_fixture",Fixed64::from_i32(100),serde_json::json!({"manacost_percentage":value}));
        let slot=w.read_storage::<Hero>().get(e).unwrap().abilities.iter().position(|id|id=="ranger_patch").unwrap();
        let result=d.step(&mut w,[]).unwrap();
        let read=|ns| {
            let metric=single_lane_player_metric(ns,1);
            let rows:Vec<_>=result.facts.iter().filter_map(|f|match &f.fact {
                ObservableFact::Hud {team,metric_id,value} if *metric_id==metric=>Some((f.audience.clone(),*team,*value)),_=>None,
            }).collect();assert_eq!(rows.len(),1);assert!(matches!(rows[0].0,FactAudience::Team(1)));assert_eq!(rows[0].1,1);rows[0].2
        };
        assert_eq!(read(MANA_COST_RAW[slot]),expected);assert_eq!(read(MANA_COST_TICK),result.tick as i64);
        assert_eq!(read(MANA_COST_BINDING[slot]),(i64::from(omoba_template_ids::ability_by_name("ranger_patch").unwrap().raw())<<8)|1);
    }
}

#[test]
fn effective_cooldown_total_formal_60hz_survives_modifier_drift_pause_and_serde() {
    use swept_dash_formal::{setup_hero_skill,cast_skill};
    for (pct,total) in [(-512,10),(1024,40)] {
        let (mut w,mut d,e,_)=setup_hero_skill(1,"ranger_patch","training_ranger");
        w.write_storage::<CProperty>().get_mut(e).unwrap().hp=Fixed64::ONE;
        w.write_resource::<BuffStore>().add(e,"cd_total_fixture",Fixed64::from_i32(100),serde_json::json!({"cooldown_percentage":pct}));
        cast_skill(&mut w,&mut d,e,"ranger_patch",None,None);
        let check=|w:&World| {
            let h=w.read_storage::<Hero>().get(e).unwrap().clone();
            let copy:Hero=serde_json::from_slice(&serde_json::to_vec(&h).unwrap()).unwrap();
            assert_eq!(copy.cooldown_total("ranger_patch",Fixed64::from_i32(20)),Some(Fixed64::from_i32(total)));
            copy.get_cooldown("ranger_patch")
        };
        assert_eq!(check(&w),Fixed64::from_i32(total));
        w.write_resource::<BuffStore>().remove(e,"cd_total_fixture");
        w.write_resource::<GamePause>().is_paused=true;for _ in 0..3 {d.step(&mut w,[]).unwrap();}assert_eq!(check(&w),Fixed64::from_i32(total));
        w.write_resource::<GamePause>().is_paused=false;d.step(&mut w,[]).unwrap();assert!(check(&w)<Fixed64::from_i32(total));
    }
}

#[test]
fn checked_mana_modifiers_formal_60hz_charge_exactly_and_reject_invalid_without_effects() {
    use swept_dash_formal::{setup_hero_skill,cast_skill,mana};
    for (value,cost) in [(serde_json::json!(-512),Some(23040)),(serde_json::json!(1024),Some(92160)),
        (serde_json::json!(-1024),Some(0)),(serde_json::json!(-9999),Some(0)),
        (serde_json::json!("bad"),None),(serde_json::json!(i64::MAX),None),(serde_json::json!(u64::MAX),None)] {
        let (mut w,mut d,e,_)=setup_hero_skill(1,"ranger_patch","training_ranger");
        w.write_storage::<CProperty>().get_mut(e).unwrap().hp=Fixed64::ONE;
        w.write_resource::<BuffStore>().add(e,"cost_fixture",Fixed64::from_i32(100),serde_json::json!({"manacost_percentage":value}));
        let before=mana(&w,e);cast_skill(&mut w,&mut d,e,"ranger_patch",None,None);
        assert_eq!(mana(&w,e).raw(),before.raw()-cost.unwrap_or(0));
        assert_eq!(w.read_storage::<Hero>().get(e).unwrap().is_on_cooldown("ranger_patch"),cost.is_some());
        let hp=w.read_storage::<CProperty>().get(e).unwrap().hp;
        if cost.is_some(){assert!(hp>Fixed64::ONE);}else{assert_eq!(hp,Fixed64::ONE);}
    }
}

#[test]
fn checked_mana_modifiers_windup_rechecks_release_and_never_charges_preparation() {
    use swept_dash_formal::{setup_hero_skill,cast_skill,mana};
    for invalid in [false,true] {
        let (mut w,mut d,e,origin)=setup_hero_skill(1,"saika_reinforcements","saika_magoichi");
        w.write_resource::<BuffStore>().add(e,"cost_before",Fixed64::from_i32(100),serde_json::json!({"manacost_percentage":1024}));
        let before=mana(&w,e);cast_skill(&mut w,&mut d,e,"saika_reinforcements",Some(origin),None);
        assert!(w.read_resource::<BuffStore>().has(e,"cast_preparation"));assert_eq!(mana(&w,e),before);
        w.write_resource::<BuffStore>().remove(e,"cost_before");
        let value=if invalid{serde_json::json!(i64::MAX)}else{serde_json::json!(-512)};
        w.write_resource::<BuffStore>().add(e,"cost_release",Fixed64::from_i32(100),serde_json::json!({"manacost_percentage":value}));
        for _ in 0..40 {d.step(&mut w,[]).unwrap();if !w.read_resource::<BuffStore>().has(e,"cast_preparation"){break;}}
        assert!(!w.read_resource::<BuffStore>().has(e,"cast_preparation"));
        assert_eq!(mana(&w,e),if invalid{before}else{before-Fixed64::from_i32(45)});
        assert_eq!(w.read_storage::<SummonedUnit>().join().count(),if invalid{0}else{2});
        assert_eq!(w.read_storage::<Hero>().get(e).unwrap().is_on_cooldown("saika_reinforcements"),!invalid);
    }
}

#[test]
fn checked_mana_modifiers_bot_budget_rejects_invalid_instead_of_planning_free_recovery() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::runtime::ability_runtime::ManaPool;
    let (mut w,mut d)=world(SingleLaneConfig {mana_enabled:true,heroes:["training_luminary".into(),"training_ranger".into()],
        wave_interval:Fixed64::from_i32(10000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    let result=d.step(&mut w,[]).unwrap();let [e,_]=hero_pair(&w);
    w.write_storage::<Pos>().get_mut(e).unwrap().0=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
    {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(e).unwrap();p.mhp=Fixed64::from_i32(1000);p.hp=p.mhp;}
    {let mut heroes=w.write_storage::<Hero>();let h=heroes.get_mut(e).unwrap();h.mana_pool=Some(ManaPool::new(Fixed64::from_i32(60),h.moba_mana_capacity().unwrap()).unwrap());}
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Mid,lane:1,escort_player_id:None}],
        think_interval_ticks:1,sustain:None,item_builds:Vec::new(),ability_learning:Vec::new(),ability_policies:vec![BotAbilityPolicy {
            ability:"lumen_touch".into(),intent:BotAbilityIntent::SelfRecovery {below_hp_per_mille:600,below_mana_per_mille:500,restore_key:"mana_restore".into()}}]};
    let has_cast=|inputs:&[(u32,PlayerInput)]|inputs.iter().any(|(_,i)|matches!(i.action,Some(PlayerInputEnum::CastAbility(_))));
    let before=w.read_storage::<Hero>().get(e).unwrap().mana_pool.clone();
    for value in [serde_json::json!("bad"),serde_json::json!(i64::MAX),serde_json::json!(u64::MAX)] {
        w.write_resource::<BuffStore>().add(e,"cost_bot",Fixed64::from_i32(100),serde_json::json!({"manacost_percentage":value}));
        assert!(!has_cast(&role_bot_inputs(&w,&bots).unwrap()));w.write_resource::<BuffStore>().remove(e,"cost_bot");
    }
    w.write_resource::<BuffStore>().add(e,"cost_bot",Fixed64::from_i32(100),serde_json::json!({"manacost_percentage":-768}));
    assert!(has_cast(&role_bot_inputs(&w,&bots).unwrap()));
    assert_eq!(w.read_storage::<Hero>().get(e).unwrap().mana_pool,before,"planning is read-only");
}

#[test]
fn cooldown_modifiers_formal_60hz_apply_once_pause_and_keep_running_duration() {
    use swept_dash_formal::{setup_hero_skill,cast_skill,mana};
    for (pct,stack,raw) in [(-512,0,10240),(1024,0,40960),(-512,256,15360),(-99999,0,2040)] {
        let (mut w,mut d,e,_)=setup_hero_skill(1,"ranger_patch","training_ranger");
        w.write_storage::<CProperty>().get_mut(e).unwrap().hp=Fixed64::ONE;
        w.write_resource::<BuffStore>().add(e,"cd_fixture",Fixed64::from_i32(100),
            serde_json::json!({"cooldown_percentage":pct,"cooldown_percentage_stacking":stack}));
        let before=mana(&w,e);cast_skill(&mut w,&mut d,e,"ranger_patch",None,None);
        let cd=|w:&World|w.read_storage::<Hero>().get(e).unwrap().get_cooldown("ranger_patch").raw();
        assert_eq!(cd(&w),raw);assert_eq!(mana(&w,e),before-Fixed64::from_i32(45));
        w.write_resource::<BuffStore>().remove(e,"cd_fixture");
        w.write_resource::<BuffStore>().add(e,"cd_drift",Fixed64::from_i32(100),serde_json::json!({"cooldown_percentage":1024}));
        w.write_resource::<GamePause>().is_paused=true;for _ in 0..3 {d.step(&mut w,[]).unwrap();}assert_eq!(cd(&w),raw);
        w.write_resource::<GamePause>().is_paused=false;
        let start=w.read_resource::<MobaMatch>().elapsed.raw();d.step(&mut w,[]).unwrap();
        assert_eq!(cd(&w),raw-(w.read_resource::<MobaMatch>().elapsed.raw()-start));
        let before=mana(&w,e);cast_skill(&mut w,&mut d,e,"ranger_patch",None,None);
        assert_eq!(mana(&w,e),before,"rejected repeat must not spend");assert!(cd(&w)<raw);
    }
}

#[test]
fn cooldown_invalid_modifiers_reject_formal_cast_without_healing_mana_or_timer() {
    use swept_dash_formal::{setup_hero_skill,cast_skill,mana};
    for key in ["cooldown_percentage","cooldown_percentage_stacking"] {
        for value in [serde_json::json!("invalid"),serde_json::json!(i64::MAX),serde_json::json!(u64::MAX),serde_json::json!(100_000_000*1024i64)] {
            let (mut w,mut d,e,_)=setup_hero_skill(1,"ranger_patch","training_ranger");
            w.write_storage::<CProperty>().get_mut(e).unwrap().hp=Fixed64::ONE;
            w.write_resource::<BuffStore>().add(e,"bad_cd",Fixed64::from_i32(100),serde_json::json!({key:value}));
            let before=mana(&w,e);cast_skill(&mut w,&mut d,e,"ranger_patch",None,None);
            assert_eq!(mana(&w,e),before);assert_eq!(w.read_storage::<CProperty>().get(e).unwrap().hp,Fixed64::ONE);
            assert!(!w.read_storage::<Hero>().get(e).unwrap().is_on_cooldown("ranger_patch"));
        }
    }
}

#[test]
fn cooldown_windup_uses_release_stats_once_and_invalid_release_has_no_effects() {
    use swept_dash_formal::{setup_hero_skill,cast_skill,mana};
    for invalid in [false,true] {
        let (mut w,mut d,e,origin)=setup_hero_skill(1,"saika_reinforcements","saika_magoichi");
        w.write_resource::<BuffStore>().add(e,"cd_before",Fixed64::from_i32(100),serde_json::json!({"cooldown_percentage":1024}));
        let before=mana(&w,e);cast_skill(&mut w,&mut d,e,"saika_reinforcements",Some(origin),None);
        assert_eq!(mana(&w,e),before);assert_eq!(w.read_storage::<SummonedUnit>().join().count(),0);
        w.write_resource::<BuffStore>().remove(e,"cd_before");
        let value=if invalid {serde_json::json!("bad")}else{serde_json::json!(-512)};
        w.write_resource::<BuffStore>().add(e,"cd_release",Fixed64::from_i32(100),serde_json::json!({"cooldown_percentage":value}));
        for _ in 0..40 {
            d.step(&mut w,[]).unwrap();
            if !w.read_resource::<BuffStore>().has(e,"cast_preparation") {break;}
        }
        assert!(!w.read_resource::<BuffStore>().has(e,"cast_preparation"));
        assert_eq!(w.read_storage::<SummonedUnit>().join().count(),if invalid{0}else{2});
        assert_eq!(mana(&w,e),if invalid{before}else{before-Fixed64::from_i32(90)});
        assert_eq!(w.read_storage::<Hero>().get(e).unwrap().get_cooldown("saika_reinforcements").raw(),if invalid{0}else{13824});
    }
}

#[test]
fn cast_time_modifier_formal_60hz_snapshots_haste_slow_floor_pause_and_single_release() {
    use swept_dash_formal::{setup_hero_skill,cast_skill,mana};
    for (percentage,expected) in [(-512,256),(1024,1024),(-99999,51),(0,512)] {
        let (mut w,mut driver,caster,origin)=setup_hero_skill(1,"saika_reinforcements","saika_magoichi");
        w.write_resource::<BuffStore>().add(caster,"cast_timing_fixture",Fixed64::from_i32(100),
            serde_json::json!({"casttime_percentage":percentage}));
        let before=mana(&w,caster);
        cast_skill(&mut w,&mut driver,caster,"saika_reinforcements",Some(origin),None);
        let start=w.read_resource::<MobaMatch>().elapsed.raw();
        let status=|w:&World|w.read_resource::<BuffStore>().get(caster,"cast_preparation")
            .map(|b|(b.remaining.raw(),b.payload["cast_preparation"]["total_raw"].as_i64().unwrap()));
        assert_eq!(status(&w),Some((expected,expected)));
        assert_eq!(mana(&w,caster),before);assert_eq!(w.read_storage::<SummonedUnit>().join().count(),0);
        // A later modifier, even malformed, cannot retime an admitted instance.
        w.write_resource::<BuffStore>().remove(caster,"cast_timing_fixture");
        w.write_resource::<BuffStore>().add(caster,"cast_drift_fixture",Fixed64::from_i32(100),
            serde_json::json!({"casttime_percentage":"changed_after_admission"}));
        w.write_resource::<GamePause>().is_paused=true;
        for _ in 0..3 {driver.step(&mut w,[]).unwrap();}
        assert_eq!(status(&w),Some((expected,expected)));assert_eq!(w.read_resource::<MobaMatch>().elapsed.raw(),start);
        w.write_resource::<GamePause>().is_paused=false;
        let mut released=false;
        for _ in 0..90 {
            driver.step(&mut w,[]).unwrap();let elapsed=w.read_resource::<MobaMatch>().elapsed.raw()-start;
            if elapsed<expected {
                assert_eq!(status(&w),Some((expected-elapsed,expected)));
                assert_eq!(mana(&w,caster),before);assert_eq!(w.read_storage::<SummonedUnit>().join().count(),0);
            } else {
                assert!(status(&w).is_none());assert_eq!(w.read_storage::<SummonedUnit>().join().count(),2);
                assert_eq!(mana(&w,caster),before-Fixed64::from_i32(90));
                assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("saika_reinforcements"));
                released=true;break;
            }
        }
        assert!(released,"bounded effective duration must release");
        for _ in 0..5 {driver.step(&mut w,[]).unwrap();}
        assert_eq!(w.read_storage::<SummonedUnit>().join().count(),2);assert_eq!(mana(&w,caster),before-Fixed64::from_i32(90));
    }
}

#[test]
fn invalid_cast_time_modifiers_reject_without_resources_but_instant_skills_stay_instant() {
    use swept_dash_formal::{setup_hero_skill,cast_skill,mana};
    for value in [serde_json::json!("invalid"),serde_json::json!(i64::MAX),serde_json::json!(u64::MAX),serde_json::json!(123*1024)] {
        let (mut w,mut driver,caster,origin)=setup_hero_skill(1,"saika_reinforcements","saika_magoichi");
        w.write_resource::<BuffStore>().add(caster,"invalid_cast_timing",Fixed64::from_i32(100),serde_json::json!({"casttime_percentage":value}));
        let before=mana(&w,caster);cast_skill(&mut w,&mut driver,caster,"saika_reinforcements",Some(origin),None);
        assert!(!w.read_resource::<BuffStore>().has(caster,"cast_preparation"));
        assert_eq!(mana(&w,caster),before);assert_eq!(w.read_storage::<SummonedUnit>().join().count(),0);
        assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("saika_reinforcements"));
    }
    let (mut w,mut driver,caster,_)=setup_hero_skill(1,"ranger_patch","training_ranger");
    w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::ONE;
    w.write_resource::<BuffStore>().add(caster,"invalid_cast_timing",Fixed64::from_i32(100),
        serde_json::json!({"casttime_percentage":"invalid"}));
    let before=mana(&w,caster);cast_skill(&mut w,&mut driver,caster,"ranger_patch",None,None);
    assert!(!w.read_resource::<BuffStore>().has(caster,"cast_preparation"));
    assert!(mana(&w,caster)<before);assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_patch"));
    assert!(w.read_storage::<CProperty>().get(caster).unwrap().hp>Fixed64::ONE);
}

#[test]
fn summoner_self_shield_formal_60hz_input_disclosure_pause_absorption_and_expiry() {
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10000),
        summoner_loadouts:std::collections::BTreeMap::from([(2,[Some("training_self_shield_v1".into()),None])]),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [hero,enemy]=hero_pair(&w);
    let y=isolated_skill_origin_y();place_hero(&mut w,hero,0,y);place_hero(&mut w,enemy,500,y);
    for entity in [hero,enemy] {set_hold(&mut w,entity);disarm_attack(&mut w,entity);}
    w.write_resource::<BuffStore>().grant_healing_reduction(enemy,Fixed64::ZERO,Fixed64::from_i32(4));
    let cast=|| (2,PlayerInput {action:Some(PlayerInputEnum::SummonerCast(omoba_core::game_proto::SummonerCast {slot:0,x_raw:None,y_raw:None}))});
    let before=w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    driver.step(&mut w,[cast()]).unwrap();
    assert_eq!(w.read_resource::<BuffStore>().shield_remaining(enemy),Fixed64::from_i32(160));
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,before);
    let snapshot=extract_snapshot(&mut w,driver.tick() as u32,std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),Vec::new(),Vec::new());
    let buff=snapshot.entities.iter().find(|e|e.entity_id==enemy.id()).unwrap().buffs.iter().find(|b|b.buff_id=="damage_shield").unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(&buff.payload_json).unwrap()["shield_remaining_raw"],163840);
    let time=w.read_resource::<BuffStore>().get(enemy,"damage_shield").unwrap().remaining;
    w.write_resource::<GamePause>().is_paused=true;for _ in 0..10{driver.step(&mut w,[]).unwrap();}
    assert_eq!(w.read_resource::<BuffStore>().get(enemy,"damage_shield").unwrap().remaining,time);
    w.write_resource::<GamePause>().is_paused=false;
    driver.step(&mut w,[cast()]).unwrap(); // Cooldown rejects; does not refresh the shield.
    assert!(w.read_resource::<BuffStore>().get(enemy,"damage_shield").unwrap().remaining<time);
    w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage {target:enemy,amount:Fixed64::from_i32(170)});
    driver.step(&mut w,[]).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,before-Fixed64::from_i32(10));
    assert!(!w.read_resource::<BuffStore>().has(enemy,"damage_shield"));
    // A later shield grant follows the same canonical timer expiry path.
    w.write_resource::<BuffStore>().grant_damage_shield(enemy,Fixed64::from_i32(20),Fixed64::from_raw(1));
    driver.step(&mut w,[]).unwrap();assert!(!w.read_resource::<BuffStore>().has(enemy,"damage_shield"));
}

mod recovery_channel_formal {
    use super::*;
    fn fixture()->(World,SimulationDriver,specs::Entity,specs::Entity) {
        let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10000),base_recovery_enabled:false,..fast_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let [hero,enemy]=hero_pair(&w);
        for entity in [hero,enemy] {set_hold(&mut w,entity);disarm_attack(&mut w,entity);}
        w.write_storage::<Gold>().get_mut(hero).unwrap().0=5000;
        driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::ItemBuy(ItemBuy{item_id:"moba_recovery_charm".into()}))})]).unwrap();
        assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().item_id,"moba_recovery_charm");
        let y=isolated_skill_origin_y();place_hero(&mut w,hero,0,y);place_hero(&mut w,enemy,100,y);
        w.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::from_i32(50);
        (w,driver,hero,enemy)
    }
    fn hp(w:&World,e:specs::Entity)->Fixed64 {w.read_storage::<CProperty>().get(e).unwrap().hp}
    fn cd(w:&World,e:specs::Entity)->f32 {w.read_storage::<Inventory>().get(e).unwrap().slots[0].as_ref().unwrap().cooldown_remaining}
    fn active(w:&World,e:specs::Entity)->bool {w.read_resource::<BuffStore>().has(e,"recovery_channel")}
    fn cast(w:&mut World,d:&mut SimulationDriver) {d.step(w,[item_use(0,None,None,None)]).unwrap();}
    #[test]
    fn purchased_channel_pulses_three_times_pauses_and_suppresses_autonomous_attack() {
        let (mut w,mut d,hero,enemy)=fixture();let enemy_hp=hp(&w,enemy);cast(&mut w,&mut d);
        assert!(active(&w,hero));assert_eq!(hp(&w,hero),Fixed64::from_i32(50));assert_eq!(cd(&w,hero),30.0);
        w.write_storage::<TAttack>().get_mut(hero).unwrap().asd_count=Fixed64::from_i32(1000);
        for _ in 0..59 {d.step(&mut w,[]).unwrap();assert_eq!(hp(&w,hero),Fixed64::from_i32(50));}
        d.step(&mut w,[]).unwrap();assert_eq!(hp(&w,hero),Fixed64::from_i32(90));assert_eq!(hp(&w,enemy),enemy_hp);
        let remaining=w.read_resource::<BuffStore>().get(hero,"recovery_channel").unwrap().remaining;let cooldown=cd(&w,hero);
        w.write_resource::<GamePause>().is_paused=true;for _ in 0..10 {d.step(&mut w,[]).unwrap();}
        assert_eq!(cd(&w,hero),cooldown);assert_eq!(w.read_resource::<BuffStore>().get(hero,"recovery_channel").unwrap().remaining,remaining);
        w.write_resource::<GamePause>().is_paused=false;
        for _ in 0..120 {d.step(&mut w,[]).unwrap();}
        assert_eq!(hp(&w,hero),Fixed64::from_i32(170));assert!(!active(&w,hero));assert!(cd(&w,hero)>26.9&&cd(&w,hero)<27.1);
    }
    #[test]
    fn pulse_boundary_hp_damage_cancels_but_shield_only_damage_does_not() {
        for shield in [false,true] {
            let (mut w,mut d,hero,_)=fixture();cast(&mut w,&mut d);for _ in 0..59 {d.step(&mut w,[]).unwrap();}
            if shield {w.write_resource::<BuffStore>().grant_damage_shield(hero,Fixed64::from_i32(20),Fixed64::from_i32(5));}
            w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage{target:hero,amount:Fixed64::from_i32(10)});
            d.step(&mut w,[]).unwrap();
            assert_eq!(hp(&w,hero),Fixed64::from_i32(if shield {90}else{40}));assert_eq!(active(&w,hero),shield);
            for _ in 0..120 {d.step(&mut w,[]).unwrap();}
            assert_eq!(hp(&w,hero),Fixed64::from_i32(if shield {170}else{40}));
        }
    }
    #[test]
    fn controlled_start_is_rejected_without_charge_and_accepted_stop_cancels() {
        let (mut w,mut d,hero,_)=fixture();
        w.write_resource::<BuffStore>().add(hero,"stun",Fixed64::from_i32(5),serde_json::json!({}));cast(&mut w,&mut d);
        assert!(!active(&w,hero));assert_eq!(cd(&w,hero),0.0);assert_eq!(hp(&w,hero),Fixed64::from_i32(50));
        w.write_resource::<BuffStore>().remove(hero,"stun");cast(&mut w,&mut d);assert!(active(&w,hero));
        d.step(&mut w,[(1,PlayerInput{action:Some(PlayerInputEnum::Stop(Default::default()))})]).unwrap();
        assert!(!active(&w,hero));for _ in 0..180 {d.step(&mut w,[]).unwrap();}assert_eq!(hp(&w,hero),Fixed64::from_i32(50));
        assert!(cd(&w,hero)>26.9&&cd(&w,hero)<27.1);
    }
}

#[test]
fn summoner_control_cleanse_formal_60hz_input_clears_public_control_and_pause_preserves_cooldown() {
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10000),
        summoner_loadouts:std::collections::BTreeMap::from([(2,[Some("training_control_cleanse_v1".into()),None])]),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [hero,enemy]=hero_pair(&w);
    let y=isolated_skill_origin_y();place_hero(&mut w,hero,0,y);place_hero(&mut w,enemy,500,y);
    for entity in [hero,enemy] {set_hold(&mut w,entity);disarm_attack(&mut w,entity);}
    {
        let mut buffs=w.write_resource::<BuffStore>();
        for entity in [hero,enemy] {for kind in ["stun","root","silence"] {buffs.add(entity,kind,Fixed64::from_i32(5),serde_json::json!({}));}}
        buffs.grant_damage_shield(enemy,Fixed64::from_i32(30),Fixed64::from_i32(5));
    }
    let cast=|| (2,PlayerInput {action:Some(PlayerInputEnum::SummonerCast(omoba_core::game_proto::SummonerCast {slot:0,x_raw:None,y_raw:None}))});
    driver.step(&mut w,[cast()]).unwrap();
    {let buffs=w.read_resource::<BuffStore>();assert!(!buffs.is_stunned(enemy)&&!buffs.is_rooted(enemy)&&!buffs.is_silenced(enemy));
        assert!(buffs.is_stunned(hero));assert_eq!(buffs.shield_remaining(enemy),Fixed64::from_i32(30));}
    let snapshot=extract_snapshot(&mut w,driver.tick() as u32,std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),Vec::new(),Vec::new());
    let row=snapshot.entities.iter().find(|e|e.entity_id==enemy.id()).unwrap();
    assert!(!row.buffs.iter().any(|b|matches!(b.buff_id.as_str(),"stun"|"root"|"silence")));
    let disclosed_cooldown=|world:&World| {
        let bytes=project_team_summoners(&world.read_resource::<MobaMatch>(),2);
        let rows=omoba_core::runtime::summoner_transport::decode_projected_summoners(&bytes).unwrap();
        let row=rows.iter().find(|row|row.player_id==2).unwrap();assert_eq!(row.slots[0].catalog_id,4);
        Fixed64::from_raw(row.slots[0].cooldown_raw)
    };
    let remaining=disclosed_cooldown(&w);assert!(remaining>Fixed64::from_i32(209)&&remaining<=Fixed64::from_i32(210));
    w.write_resource::<GamePause>().is_paused=true;for _ in 0..5 {driver.step(&mut w,[]).unwrap();}
    assert_eq!(disclosed_cooldown(&w),remaining);
    w.write_resource::<GamePause>().is_paused=false;
    w.write_resource::<BuffStore>().add(enemy,"stun",Fixed64::ONE,serde_json::json!({}));
    driver.step(&mut w,[cast()]).unwrap();assert!(w.read_resource::<BuffStore>().is_stunned(enemy),"cooling cleanse cannot repeat and grants no immunity");
}

#[test]
fn summoner_sprint_formal_60hz_shared_speed_pause_expiry_and_owner_disclosure() {
    use omb_script_abi::stat_keys::StatKey;use omoba_core::runtime::ability_runtime::ItemTimedModifier;
    let (mut w,mut driver)=world(SingleLaneConfig{wave_interval:Fixed64::from_i32(10000),
        summoner_loadouts:std::collections::BTreeMap::from([(2,[Some("training_self_sprint_v1".into()),None])]),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [other,hero]=hero_pair(&w);let y=isolated_skill_origin_y();place_hero(&mut w,hero,0,y);place_hero(&mut w,other,500,y);
    for e in [hero,other]{set_hold(&mut w,e);disarm_attack(&mut w,e);}
    let base=w.read_storage::<CProperty>().get(hero).unwrap().msd;
    {let mut b=w.write_resource::<BuffStore>();b.add(hero,"root",Fixed64::from_i32(2),serde_json::json!({}));assert!(b.grant_item_timed_modifier(hero,"fixture_stride",ItemTimedModifier::Sprint,Fixed64::from_i32(60),Fixed64::from_i32(10)));}
    let cast=|| (2,PlayerInput{action:Some(PlayerInputEnum::SummonerCast(omoba_core::game_proto::SummonerCast{slot:0,x_raw:None,y_raw:None}))});
    driver.step(&mut w,[cast()]).unwrap();
    assert_eq!(w.read_resource::<BuffStore>().sum_add(hero,StatKey::MoveSpeedBonusBuff),Fixed64::from_i32(80));
    assert_eq!(w.read_resource::<BuffStore>().sum_add(other,StatKey::MoveSpeedBonusBuff),Fixed64::ZERO);assert!(w.read_resource::<BuffStore>().is_rooted(hero));
    assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().msd,base);
    assert!(matches!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active.as_ref(),Some(HeroCommand::HoldPosition)));
    let cd=|w:&World|{let bytes=omoba_core::runtime::native::summoner_world::project_team_summoners(&w.read_resource::<MobaMatch>(),2);let rows=omoba_core::runtime::summoner_transport::decode_projected_summoners(&bytes).unwrap();let row=rows.iter().find(|r|r.player_id==2).unwrap();assert_eq!(row.slots[0].catalog_id,5);row.slots[0].cooldown_raw};
    let sprint_time=|w:&World|w.read_resource::<BuffStore>().iter_for(hero).find(|(_,entry)|entry.payload.get(StatKey::MoveSpeedBonusBuff.as_str()).and_then(serde_json::Value::as_i64)==Some(80*1024)).map(|(_,entry)|entry.remaining).unwrap();
    let before=cd(&w);assert!(before>179*1024&&before<=180*1024);let remaining=sprint_time(&w);
    w.write_resource::<GamePause>().is_paused=true;for _ in 0..5 {driver.step(&mut w,[]).unwrap();}assert_eq!(cd(&w),before);assert_eq!(sprint_time(&w),remaining);
    w.write_resource::<GamePause>().is_paused=false;driver.step(&mut w,[cast()]).unwrap();assert!(cd(&w)<before);assert!(sprint_time(&w)<remaining);
    for _ in 0..410{driver.step(&mut w,[]).unwrap();}
    assert_eq!(w.read_resource::<BuffStore>().sum_add(hero,StatKey::MoveSpeedBonusBuff),Fixed64::from_i32(60));
    for _ in 0..275{driver.step(&mut w,[]).unwrap();}assert_eq!(w.read_resource::<BuffStore>().sum_add(hero,StatKey::MoveSpeedBonusBuff),Fixed64::ZERO);
}

#[test]
fn summoner_guard_formal_60hz_settles_damage_shared_expiry_pause_and_owner_cooldown() {
    use omb_script_abi::stat_keys::StatKey;use omoba_core::runtime::ability_runtime::ItemTimedModifier;
    let (mut w,mut driver)=world(SingleLaneConfig{base_recovery_enabled:false,wave_interval:Fixed64::from_i32(10000),
        summoner_loadouts:std::collections::BTreeMap::from([(2,[Some("training_self_guard_v1".into()),None])]),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [other,hero]=hero_pair(&w);let y=isolated_skill_origin_y();place_hero(&mut w,hero,0,y);place_hero(&mut w,other,500,y);
    for e in [hero,other]{set_hold(&mut w,e);disarm_attack(&mut w,e);}
    {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(hero).unwrap();p.hp=Fixed64::from_i32(10000);p.mhp=p.hp;}
    let damage=|w:&mut World,expected:i32|{let before=w.read_storage::<CProperty>().get(hero).unwrap().hp;
        w.write_resource::<Vec<Outcome>>().push(Outcome::Damage{pos:omoba_sim::Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,real:Fixed64::from_i32(160),source:other,target:hero,damage_profile:0,predeclared:false});
        process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();assert_eq!(before-w.read_storage::<CProperty>().get(hero).unwrap().hp,Fixed64::from_i32(expected));};
    damage(&mut w,160);assert!(w.write_resource::<BuffStore>().grant_item_timed_modifier(hero,"weak_guard",ItemTimedModifier::DamageReduction,Fixed64::from_raw(128),Fixed64::from_i32(6)));
    let cast=|| (2,PlayerInput{action:Some(PlayerInputEnum::SummonerCast(omoba_core::game_proto::SummonerCast{slot:0,x_raw:None,y_raw:None}))});driver.step(&mut w,[cast()]).unwrap();
    assert_eq!(w.read_resource::<BuffStore>().sum_add(hero,StatKey::DamageTakenBonus),Fixed64::from_raw(-256));assert_eq!(w.read_resource::<BuffStore>().sum_add(other,StatKey::DamageTakenBonus),Fixed64::ZERO);damage(&mut w,120);
    assert!(matches!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active.as_ref(),Some(HeroCommand::HoldPosition)));
    let cd=|w:&World|{let bytes=omoba_core::runtime::native::summoner_world::project_team_summoners(&w.read_resource::<MobaMatch>(),2);let rows=omoba_core::runtime::summoner_transport::decode_projected_summoners(&bytes).unwrap();let row=rows.iter().find(|r|r.player_id==2).unwrap();assert_eq!(row.slots[0].catalog_id,6);row.slots[0].cooldown_raw};
    let before=cd(&w);let remaining=w.read_resource::<BuffStore>().get(hero,"item_damage_reduce:@summoner:training_self_guard_v1").unwrap().remaining;
    w.write_resource::<GamePause>().is_paused=true;for _ in 0..5{driver.step(&mut w,[]).unwrap();}assert_eq!(cd(&w),before);assert_eq!(w.read_resource::<BuffStore>().get(hero,"item_damage_reduce:@summoner:training_self_guard_v1").unwrap().remaining,remaining);
    w.write_resource::<GamePause>().is_paused=false;driver.step(&mut w,[cast()]).unwrap();assert!(cd(&w)<before);
    for _ in 0..200{driver.step(&mut w,[]).unwrap();}assert_eq!(w.read_resource::<BuffStore>().sum_add(hero,StatKey::DamageTakenBonus),Fixed64::from_raw(-128));damage(&mut w,140);
    for _ in 0..200{driver.step(&mut w,[]).unwrap();}assert_eq!(w.read_resource::<BuffStore>().sum_add(hero,StatKey::DamageTakenBonus),Fixed64::ZERO);damage(&mut w,160);
}

#[test]
fn summon_formation_formal_60hz_all_ranks_charge_once_and_pause_lifetime() {
    use self::swept_dash_formal::{setup_hero_skill,cast_skill,mana};
    for rank in 1..=4 {
        let (mut w,mut driver,caster,origin)=setup_hero_skill(rank,"saika_reinforcements","saika_magoichi");
        w.write_storage::<Facing>().get_mut(caster).unwrap().0=omoba_sim::Angle::ZERO;
        let before=mana(&w,caster);
        cast_skill(&mut w,&mut driver,caster,"saika_reinforcements",Some(origin),None);
        for _ in 0..30 {driver.step(&mut w,[]).unwrap();}
        let summons:Vec<_>=(&w.entities(),&w.read_storage::<SummonedUnit>()).join().map(|(e,s)|(e,s.time_remaining)).collect();
        assert_eq!(summons.len(),rank as usize*2,"authored rank {rank}");
        assert_eq!(mana(&w,caster),before-Fixed64::from_i32(90));
        assert!(w.read_storage::<Hero>().get(caster).unwrap().get_cooldown("saika_reinforcements")>Fixed64::ZERO);
        let duration=Fixed64::from_i32(40+rank*5);
        for (entity,remaining) in &summons {
            assert_eq!(w.read_storage::<SummonedUnit>().get(*entity).unwrap().summoner,caster);
            assert!(remaining.unwrap()<=duration&&remaining.unwrap()>duration-Fixed64::ONE);
        }
        w.write_resource::<GamePause>().is_paused=true;
        for _ in 0..8 {driver.step(&mut w,[]).unwrap();}
        for (entity,remaining) in &summons {assert_eq!(w.read_storage::<SummonedUnit>().get(*entity).unwrap().time_remaining,*remaining);}
        assert_eq!(mana(&w,caster),before-Fixed64::from_i32(90));
    }
}

#[test]
fn summon_cast_formal_input_and_60hz_script_phase_rechecks_terrain_before_charge_or_cue() {
    use self::swept_dash_formal::{setup_hero_skill,cast_skill,mana};
    for blocked in [false,true] {
        let (mut world,mut driver,caster,origin)=setup_hero_skill(2,"saika_reinforcements","saika_magoichi");
        let before=mana(&world,caster);
        cast_skill(&mut world,&mut driver,caster,"saika_reinforcements",Some(origin),None);
        let mut registry=ScriptRegistry::new();registry.insert_manifest(crate::get_manifest());
        // Public input entered the normal driver above. Tap the real serial
        // script phase at 60Hz to insert a terrain change AFTER script success
        // and BEFORE outcome settlement; this is deliberately a phase fixture.
        let mut found=false;
        for tick in 1..=40 {
            world.write_resource::<Tick>().0+=1;
            let current_tick=world.read_resource::<Tick>().0;
            run_script_dispatch(&mut world,&registry,current_tick,
                Fixed64::from_raw(tick*1024/60-(tick-1)*1024/60));
            found=world.read_resource::<Vec<Outcome>>().iter().any(|effect|matches!(effect,Outcome::ScriptCastTransaction {caster:owner,..} if *owner==caster));
            if found {break;}
            process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();world.maintain();
        }
        assert!(found,"real ability handler must produce a native settlement envelope");
        assert_eq!(mana(&world,caster),before,"script success is not a debit");
        assert_eq!((&world.entities(),&world.read_storage::<SummonedUnit>()).join().count(),0);
        if blocked {
            world.write_resource::<BlockedRegions>().0.push(BlockedRegion {name:"phase-fixture-enclosure".into(),points:vec![
                vek::Vec2::new(-100_000.,-100_000.),vek::Vec2::new(100_000.,-100_000.),
                vek::Vec2::new(100_000.,100_000.),vek::Vec2::new(-100_000.,100_000.)]});
        }
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();world.maintain();
        let count=(&world.entities(),&world.read_storage::<SummonedUnit>()).join().count();
        assert_eq!(count,if blocked {0}else{4});
        assert_eq!(mana(&world,caster),if blocked {before}else{before-Fixed64::from_i32(90)});
        assert_eq!(world.read_storage::<Hero>().get(caster).unwrap().get_cooldown("saika_reinforcements")>Fixed64::ZERO,!blocked);
    }
}

mod self_transform_formal {
    use super::*;
    use omoba_sim::Vec2;
    use super::swept_dash_formal::{setup_hero_skill,body,hp,mana,cast_skill};
    #[test]
    fn self_transform_formal_60hz_all_ranks_charge_once_pause_and_keep_single_real_normal_hit() {
        use omoba_template_ids::{ABILITY_MATCHLOCK_GUN,ABILITY_THREE_STAGE_TECHNIQUE};
        for (id,hero,buff,max_rank,visuals) in [
            (ABILITY_MATCHLOCK_GUN,"date_masamune","matchlock_gun",3,1),
            (ABILITY_THREE_STAGE_TECHNIQUE,"saika_magoichi","three_stage_transform",4,3),
        ] {for rank in 1..=max_rank {
            let (mut world,mut driver,source,origin)=setup_hero_skill(rank,id.as_str(),hero);
            // Isolate the transformation from the hero's separately tested passive proc.
            world.write_storage::<Hero>().get_mut(source).unwrap().ability_levels.insert("rain_iron_cannon".into(),0);
            let before=mana(&world,source);let level=omoba_template_ids::active_ability_const(id).unwrap().levels[rank as usize-1];
            cast_skill(&mut world,&mut driver,source,id.as_str(),None,None);
            assert_eq!(mana(&world,source),before-level.mana_cost,"{} rank{rank}",id.as_str());
            assert_eq!(world.read_storage::<Hero>().get(source).unwrap().get_cooldown(id.as_str()),level.cooldown);
            let remaining=world.read_resource::<BuffStore>().get(source,buff).unwrap().remaining;
            let cooldown=world.read_storage::<Hero>().get(source).unwrap().get_cooldown(id.as_str());
            world.write_resource::<GamePause>().is_paused=true;
            for _ in 0..10 {driver.step(&mut world,[]).unwrap();}
            assert_eq!(world.read_resource::<BuffStore>().get(source,buff).unwrap().remaining,remaining);
            assert_eq!(world.read_storage::<Hero>().get(source).unwrap().get_cooldown(id.as_str()),cooldown);
            assert_eq!(mana(&world,source),before-level.mana_cost);
            world.write_resource::<GamePause>().is_paused=false;
            // Cooldown rejection cannot charge again or refresh the existing transform.
            cast_skill(&mut world,&mut driver,source,id.as_str(),None,None);
            assert_eq!(mana(&world,source),before-level.mana_cost);
            assert!(world.read_resource::<BuffStore>().get(source,buff).unwrap().remaining<remaining);
            let victim=body(&mut world,origin+Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO),2);
            world.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::from_i32(100);
            world.write_resource::<BuffStore>().add(source,"transform_no_health_regen",Fixed64::from_i32(100),serde_json::json!({"health_regen_percentage":-1024}));
            let attack=world.read_storage::<TAttack>().get(source).unwrap().atk_physic.v;
            let expected=ability_runtime::UnitStats::from_refs(&world.read_resource::<BuffStore>(),false).final_atk(attack,source);
            let leech=ability_runtime::UnitStats::from_refs(&world.read_resource::<BuffStore>(),false).lifesteal(source);
            let authored=if id==ABILITY_MATCHLOCK_GUN {crate::ability_builder::extra_at_id(id,"lifesteal",rank as u8)}else{Fixed64::ZERO};
            assert_eq!(leech,authored,"shared transform rate uses the actual authored rank");
            assert!(expected>attack);
            let old:std::collections::BTreeSet<_>=world.try_fetch::<ProjectileOrigins>()
                .map(|origins|origins.0.keys().copied().collect()).unwrap_or_default();
            world.write_resource::<Vec<Outcome>>().push(Outcome::ProjectileLine2 {pos:origin,source:Some(source),target:Some(victim)});
            process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();world.maintain();
            {
                let origins=world.read_resource::<ProjectileOrigins>();let projectiles=world.read_storage::<Projectile>();
                let launched:Vec<_>=origins.0.iter().filter(|(e,o)|!old.contains(e)&&o.source==source).collect();
                assert_eq!(launched.len(),visuals);assert_eq!(launched.iter().filter(|(_,o)|o.normal_attack).count(),1);
                for (_,o) in &launched {assert_eq!(o.attack_lifesteal,if o.normal_attack{authored}else{Fixed64::ZERO});}
                assert_eq!(launched.iter().map(|(e,_)|projectiles.get(**e).unwrap().damage_phys).fold(Fixed64::ZERO,|a,b|a+b),expected);
            }
            world.write_resource::<BuffStore>().add(source,"stun",Fixed64::from_i32(100),serde_json::json!({}));
            for _ in 0..60 {driver.step(&mut world,[]).unwrap();}
            assert_eq!(hp(&world,victim),Fixed64::from_i32(1000)-expected,"{} rank{rank}",id.as_str());
            assert_eq!(hp(&world,source),Fixed64::from_i32(100)+expected*authored,"{} rank{rank}: actual physical leech, not natural recovery",id.as_str());
            // Move to the last fixed quantum before expiry, then let the formal tick expire it.
            let remaining=world.read_resource::<BuffStore>().get(source,buff).unwrap().remaining;
            world.write_resource::<BuffStore>().tick(remaining-Fixed64::from_raw(1));
            driver.step(&mut world,[]).unwrap();assert!(!world.read_resource::<BuffStore>().has(source,buff));
            assert_eq!(ability_runtime::UnitStats::from_refs(&world.read_resource::<BuffStore>(),false).final_atk(attack,source),attack);
            assert_eq!(ability_runtime::UnitStats::from_refs(&world.read_resource::<BuffStore>(),false).lifesteal(source),Fixed64::ZERO);
        }}
    }
}

mod passive_cone_formal {
    use super::*;
    use omoba_sim::Vec2;
    use super::swept_dash_formal::{setup_hero_skill,body,hp};
    #[test]
    fn passive_cone_formal_60hz_real_normal_projectile_all_ranks_hits_impact_sector_once_after_stun_and_pause() {
        for rank in 1..=4 {
            let (mut world,mut driver,source,origin)=setup_hero_skill(rank,"rain_iron_cannon","saika_magoichi");
            if let Some(queue)=world.write_storage::<HeroCommandQueue>().get_mut(source) {queue.clear_all();}
            let apex=origin+Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
            let victim=body(&mut world,apex,2);let front=body(&mut world,apex+Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO),2);
            let behind=body(&mut world,apex-Vec2::new(Fixed64::from_i32(10),Fixed64::ZERO),2);
            let outside=body(&mut world,apex+Vec2::new(Fixed64::from_i32(50),Fixed64::from_i32(51)),2);
            let edge=body(&mut world,apex+Vec2::new(Fixed64::from_i32(50),Fixed64::from_i32(50)),2);
            let attack=world.read_storage::<TAttack>().get(source).unwrap().atk_physic.v;
            let attack=ability_runtime::UnitStats::from_refs(&world.read_resource::<BuffStore>(),false).final_atk(attack,source);
            let fraction=crate::passive_cone::resolve(omoba_template_ids::ABILITY_RAIN_IRON_CANNON,rank as u8).unwrap().attack_fraction;
            let expected=attack*fraction;
            world.write_resource::<Vec<Outcome>>().push(Outcome::ProjectileLine2 {pos:origin,source:Some(source),target:Some(victim)});
            process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();world.maintain();
            let projectile=*world.read_resource::<ProjectileOrigins>().0.iter().find(|(_,o)|o.source==source&&o.normal_attack).unwrap().0;
            let primary=world.read_storage::<Projectile>().get(projectile).unwrap().damage_phys;
            world.write_resource::<BuffStore>().add(source,"stun",Fixed64::from_i32(100),serde_json::json!({}));
            world.write_resource::<GamePause>().is_paused=true;
            let position=world.read_storage::<Pos>().get(projectile).unwrap().0;
            for _ in 0..5 {driver.step(&mut world,[]).unwrap();}
            assert_eq!(world.read_storage::<Pos>().get(projectile).unwrap().0,position);
            assert_eq!(hp(&world,victim),Fixed64::from_i32(1000));
            world.write_resource::<GamePause>().is_paused=false;
            for _ in 0..60 {driver.step(&mut world,[]).unwrap();}
            assert_eq!(hp(&world,victim),Fixed64::from_i32(1000)-primary-expected,"rank {rank}");
            for e in [front,edge] {assert_eq!(hp(&world,e),Fixed64::from_i32(1000)-expected,"rank {rank}");}
            for e in [behind,outside] {assert_eq!(hp(&world,e),Fixed64::from_i32(1000),"rank {rank}");}
        }
    }
    #[test]
    fn passive_cone_formal_60hz_active_cast_rejects_without_resources_or_damage() {
        use super::swept_dash_formal::{cast_skill,mana};
        let (mut world,mut driver,source,origin)=setup_hero_skill(4,"rain_iron_cannon","saika_magoichi");
        let target=body(&mut world,origin+Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO),2);
        let before=mana(&world,source);cast_skill(&mut world,&mut driver,source,"rain_iron_cannon",None,None);
        assert_eq!(mana(&world,source),before);assert_eq!(hp(&world,target),Fixed64::from_i32(1000));
        assert!(world.read_storage::<Hero>().get(source).unwrap().ability_cooldowns.get("rain_iron_cannon").is_none_or(|v|*v==Fixed64::ZERO));
    }
}

mod swept_dash_formal {
    use super::*;
    use omoba_sim::Vec2;
    const ID:&str="fire_dash";
    fn setup(rank:i32)->(World,SimulationDriver,specs::Entity,Vec2) {
        setup_skill(rank,ID)
    }
    pub(super) fn setup_skill(rank:i32,id:&str)->(World,SimulationDriver,specs::Entity,Vec2) {
        setup_hero_skill(rank,id,"date_masamune")
    }
    pub(super) fn setup_hero_skill(rank:i32,id:&str,hero:&str)->(World,SimulationDriver,specs::Entity,Vec2) {
        let (mut world,mut driver)=world(SingleLaneConfig {
            heroes:[hero.into(),"training_luminary".into()],mana_enabled:true,
            base_recovery_enabled:false,wave_interval:Fixed64::from_i32(10_000),..fast_config()
        },SimulationTickProfile::Production60Hz);
        driver.step(&mut world,[]).unwrap();let [caster,enemy]=hero_pair(&world);
        let origin=Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
        world.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
        world.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+Vec2::new(Fixed64::from_i32(10_000),Fixed64::ZERO);
        world.write_storage::<MoveTarget>().remove(caster);
        {let mut heroes=world.write_storage::<Hero>();let hero=heroes.get_mut(caster).unwrap();
            hero.level=18;hero.ability_levels.insert(id.into(),rank);
            let max=hero.moba_mana_capacity().unwrap();hero.mana_pool=Some(ability_runtime::ManaPool::new(max,max).unwrap());}
        world.write_resource::<BuffStore>().add(caster,"dash_no_regen_fixture",Fixed64::from_i32(100),
            serde_json::json!({"mana_regen_percentage":-1024}));
        (world,driver,caster,origin)
    }
    pub(super) fn body(world:&mut World,at:Vec2,team:i32)->specs::Entity {
        world.create_entity().with(Pos(at)).with(Faction::new(if team==1 {FactionType::Player}else{FactionType::Enemy},team))
            .with(CProperty {hp:Fixed64::from_i32(1000),mhp:Fixed64::from_i32(1000),msd:Fixed64::ZERO,
                def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build()
    }
    fn cast(world:&mut World,driver:&mut SimulationDriver,caster:specs::Entity,target:Vec2) {
        cast_skill(world,driver,caster,ID,Some(target),None);
    }
    pub(super) fn cast_skill(world:&mut World,driver:&mut SimulationDriver,caster:specs::Entity,id:&str,target:Option<Vec2>,entity:Option<u32>) {
        let slot=world.read_storage::<Hero>().get(caster).unwrap().abilities.iter().position(|ability|ability==id).unwrap() as u32;
        driver.step(world,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index:slot,target_entity:entity,target_pos:target.map(|target|Vec2I {x:target.x.raw() as i32,y:target.y.raw() as i32}),
        }))})]).unwrap();
    }
    pub(super) fn hp(world:&World,entity:specs::Entity)->Fixed64 {world.read_storage::<CProperty>().get(entity).unwrap().hp}
    pub(super) fn mana(world:&World,entity:specs::Entity)->Fixed64 {world.read_storage::<Hero>().get(entity).unwrap().mana_pool.as_ref().unwrap().current()}
    #[test]
    fn fire_dash_formal_60hz_clamps_rank_range_hits_path_and_does_not_repeat_damage() {
        for rank in 1..=4 {
            let (mut world,mut driver,caster,origin)=setup(rank);
            let content=omoba_template_ids::active_ability_const(omoba_template_ids::ABILITY_FIRE_DASH).unwrap();
            let range=content.levels[rank as usize-1].range;let cost=content.levels[rank as usize-1].mana_cost;
            let middle=body(&mut world,origin+Vec2::new(range/Fixed64::from_i32(2),Fixed64::ZERO),2);
            let endpoint=body(&mut world,origin+Vec2::new(range,Fixed64::ZERO),2);
            let outside=body(&mut world,origin+Vec2::new(range/Fixed64::from_i32(2),Fixed64::from_i32(76)),2);
            let beyond=body(&mut world,origin+Vec2::new(range+Fixed64::from_i32(300),Fixed64::ZERO),2);
            let ally=body(&mut world,origin+Vec2::new(range/Fixed64::from_i32(2),Fixed64::ZERO),1);
            let before=mana(&world,caster);
            cast(&mut world,&mut driver,caster,origin+Vec2::new(range+Fixed64::from_i32(500),Fixed64::ZERO));
            assert_eq!(world.read_storage::<Pos>().get(caster).unwrap().0,origin+Vec2::new(range,Fixed64::ZERO));
            let damage=crate::ability_builder::extra_at_id(omoba_template_ids::ABILITY_FIRE_DASH,"damage_per_tick",rank as u8)
                *crate::ability_builder::extra_at_id(omoba_template_ids::ABILITY_FIRE_DASH,"dash_duration",rank as u8)*Fixed64::from_i32(10);
            for target in [middle,endpoint] {assert_eq!(hp(&world,target),Fixed64::from_i32(1000)-damage,"rank {rank}");}
            for target in [outside,beyond,ally] {assert_eq!(hp(&world,target),Fixed64::from_i32(1000));}
            assert_eq!(mana(&world,caster),before-cost);
            assert_eq!(world.read_storage::<Hero>().get(caster).unwrap().get_cooldown(ID),Fixed64::from_i32(30));
            world.write_resource::<BuffStore>().add(caster,"stun",Fixed64::from_i32(10),serde_json::json!({}));
            for _ in 0..30 {driver.step(&mut world,[]).unwrap();}
            for target in [middle,endpoint] {assert_eq!(hp(&world,target),Fixed64::from_i32(1000)-damage);}
        }
    }
    #[test]
    fn fire_dash_formal_60hz_stops_before_thin_wall_and_uses_clipped_damage_path() {
        let (mut world,mut driver,caster,origin)=setup(1);
        world.write_resource::<BlockedRegions>().0.push(isolated_skill_region("fire-dash-thin-wall",300.0,301.0,200.0));
        let middle=body(&mut world,origin+Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO),2);
        let behind=body(&mut world,origin+Vec2::new(Fixed64::from_i32(500),Fixed64::ZERO),2);
        cast(&mut world,&mut driver,caster,origin+Vec2::new(Fixed64::from_i32(900),Fixed64::ZERO));
        let end=world.read_storage::<Pos>().get(caster).unwrap().0;
        assert!(end.x>Fixed64::from_i32(200) && end.x<Fixed64::from_i32(300));assert_eq!(end.y,origin.y);
        assert_eq!(hp(&world,middle),Fixed64::from_i32(750));assert_eq!(hp(&world,behind),Fixed64::from_i32(1000));
        let radius=world.read_storage::<CollisionRadius>().get(caster).unwrap().0;
        let polygon=world.read_resource::<BlockedRegions>().0.last().unwrap().points.iter()
            .map(|p|Vec2::new(Fixed64::from_raw((p.x*1024.0) as i64),Fixed64::from_raw((p.y*1024.0) as i64))).collect::<Vec<_>>();
        assert!(!omoba_sim::terrain::swept_circle_hits_polygon(origin,end,radius,&polygon));
    }
    #[test]
    fn fire_dash_formal_60hz_root_zero_and_enclosure_preserve_mana_and_cooldown() {
        for reason in ["root","zero","enclosed"] {
            let (mut world,mut driver,caster,origin)=setup(1);
            if reason=="root" {world.write_resource::<BuffStore>().add(caster,"root",Fixed64::from_i32(10),serde_json::json!({}));}
            if reason=="enclosed" {world.write_resource::<BlockedRegions>().0.push(isolated_skill_region("fire-dash-enclosed",-1000.0,1000.0,1000.0));}
            let target=body(&mut world,origin+Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO),2);
            let before=mana(&world,caster);
            cast(&mut world,&mut driver,caster,if reason=="zero" {origin}else{origin+Vec2::new(Fixed64::from_i32(900),Fixed64::ZERO)});
            assert_eq!(world.read_storage::<Pos>().get(caster).unwrap().0,origin,"{reason}");
            assert_eq!(mana(&world,caster),before,"{reason}");assert_eq!(hp(&world,target),Fixed64::from_i32(1000));
            assert!(!world.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown(ID),"{reason}");
        }
    }
}

mod point_blast_formal {
    use super::*;
    use super::swept_dash_formal::{setup_skill,body,cast_skill,hp,mana};
    use omoba_sim::Vec2;
    use omoba_template_ids::{AbilityId,ABILITY_FLAME_BLADE,ABILITY_FLAME_ASSAULT};
    fn radius(id:AbilityId,rank:u8)->Fixed64 {crate::ability_builder::extra_at_id(id,if id==ABILITY_FLAME_BLADE {"swipe_radius"}else{"radius"},rank)}
    #[test]
    fn ground_zone_public_status_formal_60hz_cast_pause_countdown_and_death() {
        let id=ABILITY_FLAME_ASSAULT;let (mut world,mut driver,caster,origin)=setup_skill(1,id.as_str());
        disarm_attack(&mut world,caster);set_hold(&mut world,caster);
        cast_skill(&mut world,&mut driver,caster,id.as_str(),Some(origin),None);
        let public=|world:&World|omoba_core::runtime::buff_visual_state::BuffVisualState::from_store(&world.read_resource::<BuffStore>(),caster).render_buffs();
        let first=public(&world);let status=first.iter().find(|row|row.buff_id=="ground_damage_zone").unwrap();
        assert_eq!(status.remaining_secs,3.0);assert!(status.payload_json.is_empty());
        world.write_resource::<GamePause>().is_paused=true;for _ in 0..10 {driver.step(&mut world,[]).unwrap();}
        assert_eq!(public(&world).iter().find(|row|row.buff_id=="ground_damage_zone").unwrap().remaining_secs,3.0);
        world.write_resource::<GamePause>().is_paused=false;for _ in 0..30 {driver.step(&mut world,[]).unwrap();}
        assert_eq!(public(&world).iter().find(|row|row.buff_id=="ground_damage_zone").unwrap().remaining_secs,2.5);
        world.write_resource::<Vec<Outcome>>().push(Outcome::Death{ent:caster,pos:origin});
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
        assert!(!public(&world).iter().any(|row|row.buff_id=="ground_damage_zone"));
    }
    #[test]
    fn ground_zone_formal_60hz_all_ranks_pause_and_complete_six_pulses_after_one_blast() {
        for rank in 1..=4 {
            let id=ABILITY_FLAME_ASSAULT;
            let (mut world,mut driver,caster,origin)=setup_skill(rank,id.as_str());
            disarm_attack(&mut world,caster);set_hold(&mut world,caster);
            let center=origin+Vec2::new(Fixed64::from_i32(400),Fixed64::ZERO);
            let victim=body(&mut world,center,2);let ally=body(&mut world,center,1);
            let before=mana(&world,caster);cast_skill(&mut world,&mut driver,caster,id.as_str(),Some(center),None);
            let blast=crate::ability_builder::extra_at_id(id,"damage",rank as u8);
            let pulse=crate::ability_builder::extra_at_id(id,"zone_damage",rank as u8);
            assert_eq!(hp(&world,victim),Fixed64::from_i32(1000)-blast);
            let key=format!("__ground_damage_zone:{}:{}:{}",id.as_str(),caster.id(),caster.gen().id());
            let frozen=world.read_resource::<BuffStore>().get(caster,&key).unwrap().remaining;
            world.write_resource::<GamePause>().is_paused=true;
            for _ in 0..40 {driver.step(&mut world,[]).unwrap();}
            assert_eq!(hp(&world,victim),Fixed64::from_i32(1000)-blast);
            assert_eq!(world.read_resource::<BuffStore>().get(caster,&key).unwrap().remaining,frozen);
            world.write_resource::<GamePause>().is_paused=false;
            for _ in 0..185 {driver.step(&mut world,[]).unwrap();}
            assert_eq!(hp(&world,victim),Fixed64::from_i32(1000)-blast-pulse*Fixed64::from_i32(6),"rank {rank}");
            assert_eq!(hp(&world,ally),Fixed64::from_i32(1000));assert!(!world.read_resource::<BuffStore>().has(caster,&key));
            assert_eq!(mana(&world,caster),before-omoba_template_ids::active_ability_const(id).unwrap().levels[rank as usize-1].mana_cost);
            for _ in 0..40 {driver.step(&mut world,[]).unwrap();}
            assert_eq!(hp(&world,victim),Fixed64::from_i32(1000)-blast-pulse*Fixed64::from_i32(6));
        }
    }
    #[test]
    fn ground_zone_formal_60hz_rechecks_entry_exit_and_source_death_cancels() {
        let id=ABILITY_FLAME_ASSAULT;let (mut world,mut driver,caster,origin)=setup_skill(1,id.as_str());
        disarm_attack(&mut world,caster);set_hold(&mut world,caster);
        let center=origin+Vec2::new(Fixed64::from_i32(400),Fixed64::ZERO);
        let victim=body(&mut world,center,2);cast_skill(&mut world,&mut driver,caster,id.as_str(),Some(center),None);
        world.write_storage::<Pos>().get_mut(victim).unwrap().0=origin+Vec2::new(Fixed64::from_i32(2000),Fixed64::ZERO);
        for _ in 0..32 {driver.step(&mut world,[]).unwrap();}
        assert_eq!(hp(&world,victim),Fixed64::from_i32(800));
        world.write_storage::<Pos>().get_mut(victim).unwrap().0=center;
        for _ in 0..30 {driver.step(&mut world,[]).unwrap();}
        assert_eq!(hp(&world,victim),Fixed64::from_i32(775));
        world.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::ZERO;
        for _ in 0..40 {driver.step(&mut world,[]).unwrap();}
        assert_eq!(hp(&world,victim),Fixed64::from_i32(775));
        assert!(!world.read_resource::<BuffStore>().entities_with_key(ability_runtime::damage_zone::KEY).any(|source|source==caster));
    }
    #[test]
    fn point_blast_formal_60hz_both_skills_all_ranks_use_center_range_exact_radius_and_control() {
        for id in [ABILITY_FLAME_BLADE,ABILITY_FLAME_ASSAULT] {for rank in 1..=4 {
            let (mut world,mut driver,caster,origin)=setup_skill(rank,id.as_str());
            let content=omoba_template_ids::active_ability_const(id).unwrap();let level=&content.levels[rank as usize-1];
            let r=radius(id,rank as u8);let center=origin+Vec2::new(level.range,Fixed64::ZERO);
            let inside=body(&mut world,center,2);let tangent=body(&mut world,center+Vec2::new(r,Fixed64::ZERO),2);
            let outside=body(&mut world,center+Vec2::new(r+Fixed64::from_raw(1),Fixed64::ZERO),2);
            let ally=body(&mut world,center,1);let dead=body(&mut world,center,2);
            world.write_storage::<CProperty>().get_mut(dead).unwrap().hp=Fixed64::ZERO;
            let before=mana(&world,caster);cast_skill(&mut world,&mut driver,caster,id.as_str(),Some(center),None);
            let damage=crate::ability_builder::extra_at_id(id,"damage",rank as u8);
            for target in [inside,tangent] {
                assert_eq!(hp(&world,target),Fixed64::from_i32(1000)-damage,"{} rank{rank}",id.as_str());
                assert_eq!(world.read_resource::<BuffStore>().is_stunned(target),id==ABILITY_FLAME_ASSAULT);
                if id==ABILITY_FLAME_ASSAULT {let buffs=world.read_resource::<BuffStore>();let time=buffs.get(target,"stun").unwrap().remaining;
                    let authored=crate::ability_builder::extra_at_id(id,"stun_duration",rank as u8);assert!(time>Fixed64::ZERO && time<=authored);}
            }
            for target in [outside,ally] {assert_eq!(hp(&world,target),Fixed64::from_i32(1000));assert!(!world.read_resource::<BuffStore>().is_stunned(target));}
            assert_eq!(mana(&world,caster),before-level.mana_cost);
            assert_eq!(world.read_storage::<Hero>().get(caster).unwrap().get_cooldown(id.as_str()),level.cooldown);
            world.write_resource::<BuffStore>().add(caster,"stun",Fixed64::from_i32(10),serde_json::json!({}));
            for _ in 0..10 {driver.step(&mut world,[]).unwrap();}
            for target in [inside,tangent] {assert_eq!(hp(&world,target),Fixed64::from_i32(1000)-damage,"instant damage does not repeat");}
        }}
    }
    #[test]
    fn point_blast_formal_60hz_outside_wrong_target_and_missing_point_preserve_resources() {
        for id in [ABILITY_FLAME_BLADE,ABILITY_FLAME_ASSAULT] {for reason in ["outside","entity","missing"] {
            let (mut world,mut driver,caster,origin)=setup_skill(1,id.as_str());
            let range=omoba_template_ids::active_ability_const(id).unwrap().levels[0].range;
            let victim=body(&mut world,origin+Vec2::new(range,Fixed64::ZERO),2);let before=mana(&world,caster);
            let point=(reason=="outside").then_some(origin+Vec2::new(range+Fixed64::from_raw(1),Fixed64::ZERO));
            cast_skill(&mut world,&mut driver,caster,id.as_str(),point,(reason=="entity").then_some(victim.id()));
            assert_eq!(mana(&world,caster),before,"{} {reason}",id.as_str());assert_eq!(hp(&world,victim),Fixed64::from_i32(1000));
            assert!(!world.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown(id.as_str()));
            assert!(!world.read_resource::<BuffStore>().is_stunned(victim));
        }}
    }
    #[test]
    fn point_blast_formal_60hz_root_does_not_block_non_movement_circle_skills() {
        for id in [ABILITY_FLAME_BLADE,ABILITY_FLAME_ASSAULT] {
            let (mut world,mut driver,caster,origin)=setup_skill(1,id.as_str());
            world.write_resource::<BuffStore>().add(caster,"root",Fixed64::from_i32(10),serde_json::json!({}));
            let center=origin+Vec2::new(Fixed64::from_i32(400),Fixed64::ZERO);let victim=body(&mut world,center,2);
            cast_skill(&mut world,&mut driver,caster,id.as_str(),Some(center),None);
            assert_eq!(hp(&world,victim),Fixed64::from_i32(800));
            assert_eq!(world.read_storage::<Pos>().get(caster).unwrap().0,origin);
            assert!(world.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown(id.as_str()));
        }
    }
}

fn world(mut config: SingleLaneConfig, profile: SimulationTickProfile) -> (World, SimulationDriver) {
    // Fixtures naming an authored map derive its redundant base scalar just as
    // the production RoleBotMatchPlan does. Explicit non-default overrides stay.
    if config.lane_length==SingleLaneConfig::default().lane_length {
        if let Some(map)=config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name) {
            config.lane_length=Fixed64::from_i32(map.lane_length);
        }
    }
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    world.insert(TowerTemplateRegistry::default());
    populate_tower_upgrade_registry(&mut world);
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

// These skill fixtures intentionally run outside lane/camp combat. Keep the
// complete cast/HP/cooldown assertions independent of an authored map's size.
fn isolated_skill_origin_y()->i32 {
    omoba_template_ids::MOBA_MAPS.iter().flat_map(|map|
        map.lanes.iter().flat_map(|lane|lane.waypoints.iter().copied())
            .chain(map.terrain.iter().flat_map(|region|[region.min,region.max]))
            .chain(map.jungle_camps.iter().flat_map(|camp|camp.home_positions())))
        .map(|(_,y)|y).max().unwrap_or(2400)+2000
}

fn isolated_skill_region(name:&str,min_x:f32,max_x:f32,half_y:f32)->BlockedRegion {
    let y=isolated_skill_origin_y() as f32;
    BlockedRegion {name:name.into(),points:vec![vek::Vec2::new(min_x,y-half_y),
        vek::Vec2::new(max_x,y-half_y),vek::Vec2::new(max_x,y+half_y),vek::Vec2::new(min_x,y+half_y)]}
}

// These navigation fixtures explicitly control the disclosed lane opportunity.
// Nearby idle jungle camps are unrelated candidates on the expanded map.
fn disclose_lane_fixture(world: &mut World, tick: u64, _delay: u64) {
    run_committed_visibility_wave_b(world,tick,0);
    let camps:Vec<_>=world.read_resource::<MobaMatch>().jungle_camps.iter()
        .flat_map(|camp|std::iter::once(camp).chain(camp.members.iter()))
        .filter_map(|camp|camp.entity.map(canonical_entity_id)).collect();
    world.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current
        .retain(|id|!camps.contains(id));
}

#[test]
fn champion_protection_prioritizes_aggressor_and_expires_in_real_lane_ai() {
    for map_id in [None,Some("three_lane_training")] {
      for basic_attack in [false,true] {
        let (mut w,mut driver)=world(SingleLaneConfig { map_id:map_id.map(str::to_owned),
            additional_players:vec![extra_player(3,1)],
            creeps_per_wave:1,wave_interval:Fixed64::from_i32(10_000),..fast_config() },SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();
        let [victim,attacker]=hero_pair(&w);
        let other_victim=w.read_resource::<MobaMatch>().heroes[2].entity.unwrap();
        let defender=lane_creeps(&w,1)[0];let bait=lane_creeps(&w,2)[0];
        let tower=w.read_resource::<MobaMatch>().lane_towers[0][0].unwrap();
        let origin=omoba_sim::Vec2::new(Fixed64::from_i32(10_000),Fixed64::from_i32(10_000));
        {let mut positions=w.write_storage::<Pos>();
            for (_,p) in (&w.entities(),&mut positions).join() {p.0=-origin;}
            for entity in [victim,attacker,defender,tower] {positions.get_mut(entity).unwrap().0=origin;}
            positions.get_mut(bait).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::ONE,Fixed64::ZERO);
        }
        run_committed_visibility_wave_b(&mut w,driver.tick(),0);
        w.write_resource::<DeltaTime>().0=Fixed64::ONE;
        w.write_resource::<Vec<DamageInstance>>().clear();
        assert!(begin_moba_match_tick(&mut w));
        assert!(w.read_resource::<Vec<DamageInstance>>().iter().any(|d|d.source.source_entity==defender && d.target==bait));
        w.write_resource::<Vec<DamageInstance>>().clear();
        if basic_attack {
            w.write_resource::<Vec<Outcome>>().push(Outcome::AttackPhaseCue {entity:attacker,attack_seq:1,is_critical:false,
                target:Some(victim),target_pos:Some(origin),windup_ms:200,backswing_ms:800,dir_rad:0.0});
        } else {
            let mut hit=lethal(attacker,victim);
            if let Outcome::Damage {real,..}=&mut hit {*real=Fixed64::ONE;}
            w.write_resource::<Vec<Outcome>>().push(hit);
        }
        process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
        let mut distant=lethal(attacker,other_victim);
        if let Outcome::Damage {real,..}=&mut distant {*real=Fixed64::ONE;}
        w.write_resource::<Vec<Outcome>>().push(distant);
        process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
        assert!(begin_moba_match_tick(&mut w));
        {let damage=w.read_resource::<Vec<DamageInstance>>();
            assert!(damage.iter().any(|d|d.source.source_entity==tower && d.target==attacker),"tower protects on any champion damage");
            assert!(damage.iter().any(|d|d.source.source_entity==defender && d.target==if basic_attack {attacker} else {bait}),
                "only deliberate attacks trigger minion assistance");
        }
        if basic_attack {
            w.write_resource::<Vec<DamageInstance>>().clear();
            let mut area=lethal(attacker,victim);
            if let Outcome::Damage {real,..}=&mut area {*real=Fixed64::ONE;}
            w.write_resource::<Vec<Outcome>>().push(area);
            process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
            w.write_resource::<DeltaTime>().0=Fixed64::from_i32(2);
            assert!(begin_moba_match_tick(&mut w));
            let damage=w.read_resource::<Vec<DamageInstance>>();
            assert!(damage.iter().any(|d|d.source.source_entity==defender && d.target==bait),"AOE cannot extend minion help");
            assert!(damage.iter().any(|d|d.source.source_entity==tower && d.target==attacker),"new damage can refresh turret retaliation");
        }
        w.write_resource::<Vec<DamageInstance>>().clear();
        w.write_resource::<DeltaTime>().0=Fixed64::from_i32(3);
        assert!(begin_moba_match_tick(&mut w));
        assert!(w.read_resource::<Vec<DamageInstance>>().iter().any(|d|d.source.source_entity==defender && d.target==bait),"expired protection returns to wave");
        let enemy_tower=w.read_resource::<MobaMatch>().lane_towers[0][1].unwrap();
        {let mut positions=w.write_storage::<Pos>();
            for entity in [bait,attacker] {positions.get_mut(entity).unwrap().0=-origin;}
            positions.get_mut(enemy_tower).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(50),Fixed64::ZERO);
        }
        run_committed_visibility_wave_b(&mut w,driver.tick(),0);
        w.write_resource::<Vec<DamageInstance>>().clear();w.write_resource::<DeltaTime>().0=Fixed64::ONE;
        assert!(begin_moba_match_tick(&mut w));
        assert!(w.read_resource::<Vec<DamageInstance>>().iter().any(|d|d.source.source_entity==defender && d.target==enemy_tower));
        w.write_storage::<Pos>().get_mut(attacker).unwrap().0=origin;
        run_committed_visibility_wave_b(&mut w,driver.tick(),0);
        w.write_resource::<Vec<Outcome>>().push(Outcome::AttackPhaseCue {entity:attacker,attack_seq:2,is_critical:false,
            target:Some(victim),target_pos:Some(origin),windup_ms:200,backswing_ms:800,dir_rad:0.0});
        process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
        w.write_resource::<Vec<DamageInstance>>().clear();
        assert!(begin_moba_match_tick(&mut w));
        assert!(w.read_resource::<Vec<DamageInstance>>().iter().any(|d|d.source.source_entity==defender && d.target==enemy_tower),
            "minions already attacking a turret ignore champion calls for help");
      }
    }
}

#[test]
fn authored_minion_waves_spawn_melee_casters_periodic_siege_and_use_actual_range() {
    let (mut w,mut driver)=world(SingleLaneConfig {authored_minion_waves:true,
        wave_interval:Fixed64::from_i32(30),..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let count=|w:&World,id:&str|(&w.entities(),&w.read_storage::<Unit>()).join().filter(|(_,u)|u.id==id).count();
    assert_eq!((count(&w,"lane_melee"),count(&w,"lane_caster"),count(&w,"lane_siege")),(18,18,0));
    w.write_resource::<DeltaTime>().0=Fixed64::from_i32(30);
    assert!(begin_moba_match_tick(&mut w));assert!(begin_moba_match_tick(&mut w));
    assert_eq!((count(&w,"lane_melee"),count(&w,"lane_caster"),count(&w,"lane_siege")),(54,54,6));
    let find=|id:&str,team|(&w.entities(),&w.read_storage::<Unit>(),&w.read_storage::<Faction>()).join()
        .find(|(_,u,f)|u.id==id && f.team_id==team).unwrap().0;
    let caster=find("lane_caster",1);let melee=find("lane_melee",1);let target=find("lane_melee",2);
    let origin=omoba_sim::Vec2::new(Fixed64::from_i32(10_000),Fixed64::from_i32(10_000));
    {let mut positions=w.write_storage::<Pos>();
        for (_,p) in (&w.entities(),&mut positions).join() {p.0=-origin;}
        for e in [caster,melee] {positions.get_mut(e).unwrap().0=origin;}
        positions.get_mut(target).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(400),Fixed64::ZERO);
    }
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    w.write_resource::<Vec<DamageInstance>>().clear();w.write_resource::<DeltaTime>().0=Fixed64::ONE;
    assert!(begin_moba_match_tick(&mut w));
    let damage=w.read_resource::<Vec<DamageInstance>>();
    assert!(damage.iter().any(|d|d.source.source_entity==caster && d.target==target));
    assert!(!damage.iter().any(|d|d.source.source_entity==melee),"melee must move rather than hit at caster range");
    let states=w.read_resource::<MobaMatch>().disclosed_unit_attack_states(false);
    assert_eq!(states[&canonical_entity_id(caster)].phase,2,"committed caster strike has a public recovery animation");
    assert_eq!(states[&canonical_entity_id(melee)].phase,0,"movement does not invent an attack animation");
}

fn mana_cast_fixture(balance: i32) -> (World, SimulationDriver, specs::Entity, specs::Entity) {
    use omoba_core::runtime::ability_runtime::ManaPool;
    let (mut w, mut driver) = world(SingleLaneConfig {
        heroes: ["training_ranger".into(), "training_luminary".into()],
        wave_interval: Fixed64::from_i32(10_000), ..three_lane_config()
    }, SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let [caster, target] = hero_pair(&w);
    let source = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(isolated_skill_origin_y()));
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
fn active_cone_formal_60hz_direction_damage_resources_pause_and_rejection() {
    use omoba_core::runtime::ability_runtime::ManaPool;
    for (aim_x,aim_y,should_hit,should_cast) in [(1,0,true,true),(-1,0,false,true),(0,0,false,false),(701,0,false,false)] {
        let (mut w,mut driver)=world(SingleLaneConfig{heroes:["training_scatter_ranger".into(),"training_luminary".into()],
            wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let [caster,target]=hero_pair(&w);let y=isolated_skill_origin_y();
        place_hero(&mut w,caster,0,y);place_hero(&mut w,target,600,y);
        {let mut props=w.write_storage::<CProperty>();for e in [caster,target]{let p=props.get_mut(e).unwrap();p.hp=Fixed64::from_i32(10_000);p.mhp=p.hp;p.def_physic=Fixed64::ZERO;p.def_magic=Fixed64::ZERO;}}
        w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool=Some(ManaPool::new(Fixed64::from_i32(280),Fixed64::from_i32(280)).unwrap());
        let result=driver.step(&mut w,[(1,PlayerInput{action:Some(PlayerInputEnum::CastAbility(CastAbility{ability_index:2,target_entity:None,
            target_pos:Some(Vec2I{x:aim_x*1024,y:(y+aim_y)*1024})}))})]).unwrap();
        let hp=w.read_storage::<CProperty>().get(target).unwrap().hp;
        assert_eq!(hp,Fixed64::from_i32(if should_hit{9890}else{10_000}),"direction {aim_x},{aim_y}");
        assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_scatter"),should_cast);
        let mana=w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current();
        assert!(mana>=Fixed64::from_i32(if should_cast{235}else{280}));assert!(mana<=Fixed64::from_i32(if should_cast{236}else{280}));
        assert!(!result.facts.iter().any(|f|matches!(f.fact,ObservableFact::AbilityArea{..})),"no circular cue for a sector");
        w.write_resource::<GamePause>().is_paused=true;driver.step(&mut w,[]).unwrap();
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,hp);assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current(),mana);
    }
}

#[test]
fn raw_defenses_formal_60hz_arrow_bleed_pause_shield_and_live_armor() {
    for extra_armor in [false,true] {
        let (mut w,mut driver,caster,target)=mana_cast_fixture(280);
        for entity in [caster,target] {set_hold(&mut w,entity);disarm_attack(&mut w,entity);}
        w.write_storage::<CProperty>().get_mut(target).unwrap().def_physic=Fixed64::from_i32(100);
        driver.step(&mut w,[(1,PlayerInput{action:Some(PlayerInputEnum::CastAbility(CastAbility{ability_index:0,target_entity:Some(target.id()),target_pos:None}))})]).unwrap();
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(9955));
        w.write_resource::<BuffStore>().grant_damage_shield(target,Fixed64::from_i32(10),Fixed64::from_i32(10));
        w.write_resource::<GamePause>().is_paused=true;for _ in 0..10 {driver.step(&mut w,[]).unwrap();}
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(9955));
        w.write_resource::<GamePause>().is_paused=false;
        for _ in 0..120 {driver.step(&mut w,[]).unwrap();}
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(9955));
        assert_eq!(w.read_resource::<BuffStore>().shield_remaining(target),Fixed64::ZERO);
        if extra_armor {w.write_resource::<BuffStore>().add(target,"late_armor_fixture",Fixed64::from_i32(10),serde_json::json!({"physical_armor_bonus":100*1024}));}
        for _ in 0..120 {driver.step(&mut w,[]).unwrap();}
        // At 200 armor: ratio682/1024; 112 seventeen-raw ticks deal57 raw each,
        // eight eighteen-raw ticks deal61 raw each. Preserve per-packet rounding.
        let expected=if extra_armor {Fixed64::from_i32(9955)-Fixed64::from_raw(6872)}else{Fixed64::from_i32(9945)};
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,expected);
        for _ in 0..5 {driver.step(&mut w,[]).unwrap();}
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,expected);
    }
}

#[test]
fn raw_defenses_formal_60hz_authored_magic_resistance_and_direct_override_cap() {
    for override_resistance in [false,true] {
        let (mut w,mut driver)=world(SingleLaneConfig{heroes:["training_luminary".into(),"training_ranger".into()],
            wave_interval:Fixed64::from_i32(10000),..three_lane_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let [caster,target]=hero_pair(&w);
        for (entity,x) in [(caster,0),(target,300)] {place_hero(&mut w,entity,x,isolated_skill_origin_y());set_hold(&mut w,entity);disarm_attack(&mut w,entity);}
        assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().abilities[0],"lumen_bolt");
        {let mut properties=w.write_storage::<CProperty>();let p=properties.get_mut(target).unwrap();p.hp=Fixed64::from_i32(10000);p.mhp=p.hp;p.def_magic=Fixed64::from_i32(25);}
        if override_resistance {w.write_resource::<BuffStore>().add(target,"resist_override_fixture",Fixed64::from_i32(10),serde_json::json!({"magical_resistance_direct_modification":1024}));}
        driver.step(&mut w,[(1,PlayerInput{action:Some(PlayerInputEnum::CastAbility(CastAbility{ability_index:0,target_entity:Some(target.id()),target_pos:None}))})]).unwrap();
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(if override_resistance {9980}else{9940}));
        assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("lumen_bolt"));
    }
}

#[test]
fn authored_bleed_formal_60hz_all_ranks_pause_shield_and_complete_expiry() {
    for rank in 1..=4 {
        let (mut w,mut driver,caster,target)=mana_cast_fixture(280);
        for entity in [caster,target] {set_hold(&mut w,entity);disarm_attack(&mut w,entity);}
        w.write_storage::<Hero>().get_mut(caster).unwrap().ability_levels.insert("ranger_shot".into(),rank);
        let input=(1,PlayerInput{action:Some(PlayerInputEnum::CastAbility(CastAbility{ability_index:0,target_entity:Some(target.id()),target_pos:None}))});
        driver.step(&mut w,[input]).unwrap();
        let key=format!("generic_dot:ranger_shot:{}:{}",caster.id(),caster.gen().id());
        let immediate=45+45*rank;
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(10000-immediate));
        let entry=w.read_resource::<BuffStore>().get(target,&key).unwrap().clone();
        assert_eq!(entry.remaining,Fixed64::from_i32(4));
        assert_eq!(entry.payload["dot_kind"],"physical");
        let public=omoba_core::runtime::buff_visual_state::BuffVisualState::from_store(&w.read_resource::<BuffStore>(),target);
        let bleed_id=omoba_template_ids::buff_by_name("bleed").unwrap().raw();
        let slow_id=omoba_template_ids::buff_by_name("slow").unwrap().raw();
        assert!(public.0.iter().any(|(id,_)|*id==bleed_id));assert!(public.0.iter().any(|(id,_)|*id==slow_id));
        assert!(public.encode().is_some());
        w.write_resource::<BuffStore>().grant_damage_shield(target,Fixed64::from_i32(10),Fixed64::from_i32(10));
        w.write_resource::<GamePause>().is_paused=true;
        for _ in 0..10 {driver.step(&mut w,[]).unwrap();}
        assert_eq!(w.read_resource::<BuffStore>().get(target,&key).unwrap().remaining,entry.remaining);
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(10000-immediate));
        w.write_resource::<GamePause>().is_paused=false;
        for _ in 0..240 {driver.step(&mut w,[]).unwrap();}
        assert!(!w.read_resource::<BuffStore>().has(target,&key));
        assert_eq!(w.read_resource::<BuffStore>().shield_remaining(target),Fixed64::ZERO);
        let expected=Fixed64::from_i32(10000-immediate-4*(5+5*rank)+10);
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,expected);
        for _ in 0..5 {driver.step(&mut w,[]).unwrap();}
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,expected);
    }
}

#[test]
fn authored_bleed_formal_input_rejection_and_explicit_removal_stop_damage() {
    for accepted in [false,true] {
        let (mut w,mut driver,caster,target)=mana_cast_fixture(if accepted {280}else{1});
        for entity in [caster,target] {set_hold(&mut w,entity);disarm_attack(&mut w,entity);}
        let input=(1,PlayerInput{action:Some(PlayerInputEnum::CastAbility(CastAbility{ability_index:0,target_entity:Some(target.id()),target_pos:None}))});
        driver.step(&mut w,[input]).unwrap();
        let key=format!("generic_dot:ranger_shot:{}:{}",caster.id(),caster.gen().id());
        assert_eq!(w.read_resource::<BuffStore>().has(target,&key),accepted);
        for _ in 0..10 {driver.step(&mut w,[]).unwrap();}
        w.write_resource::<BuffStore>().remove(target,&key);
        let hp=w.read_storage::<CProperty>().get(target).unwrap().hp;
        for _ in 0..15 {driver.step(&mut w,[]).unwrap();}
        assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,hp);
        if !accepted {assert_eq!(hp,Fixed64::from_i32(10000));}
    }
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
    // The earlier arrow continues its authored 10 DPS during this cast tick.
    let bleed=Fixed64::from_i32(10)*w.read_resource::<DeltaTime>().0;
    assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp, Fixed64::from_i32(9700)-bleed);
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
        ability_index:2,target_entity:None,target_pos:Some(Vec2I {x:x*1024,y:isolated_skill_origin_y()*1024}),
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
        && (x_raw,y_raw)==(300*1024,i64::from(isolated_skill_origin_y())*1024) && radius_raw>0 && duration_raw==1024));
    for (team,visible) in [(1,true),(1,false),(2,true)] {
        let canonical=canonical_entity_id(caster);
        let visible_set=if visible {std::collections::BTreeSet::from([canonical])} else {Default::default()};
        let reveals=if visible {vec![VisibilityTransition::Reveal {canonical_id:canonical,effective_tick:succeeded.tick,baseline:Vec::new()}]} else {Vec::new()};
        let projected=TeamViewProjector::new(team,TeamProjectorConfig::default()).build_frame(
            succeeded.tick,succeeded.tick,&visible_set,reveals,std::slice::from_ref(fact),&ProjectionDependencyGraph::default()).unwrap();
        let cues:Vec<_>=projected.frame.step.as_ref().unwrap().public_events.iter().filter_map(|event|
            PresentationCue::from_public_event(succeeded.tick,event,|_|Some(1))).collect();
        assert_eq!(cues.len(),usize::from(team==1 && visible));
        if let Some((_,PresentationCue::AbilityArea(area)))=cues.first() {assert_eq!(area.center,(300*1024,i64::from(isolated_skill_origin_y())*1024));}
    }
    assert!(!driver.step(&mut w,[]).unwrap().facts.iter().any(|fact|matches!(fact.fact,ObservableFact::AbilityArea {..})));
}

#[test]
fn successful_cast_facts_keep_unique_order_across_dispatch_drains_in_one_tick() {
    let (mut w, _, caster, target) = mana_cast_fixture(280);
    w.read_resource::<ObservableFactBuffer>().drain_ordered().unwrap();
    w.write_resource::<ScriptVisualEventQueue>().drain();
    let mut registry = ScriptRegistry::new(); registry.insert_manifest(crate::get_manifest());
    let mut committed_visuals=Vec::new();
    for slot in [0, 3] {
        handle_ability_cast_from_input(&mut w, slot, None, Some(target.id()), 1).unwrap();
        run_script_dispatch(&mut w, &registry, 17, Fixed64::from_raw(17));
        assert!(!w.write_resource::<ScriptVisualEventQueue>().drain().iter().any(|event|event.kind==ScriptVisualEventKind::SkillCast));
        process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
        committed_visuals.extend(w.write_resource::<ScriptVisualEventQueue>().drain());
    }
    let facts: Vec<_> = w.read_resource::<ObservableFactBuffer>().drain_ordered().unwrap().into_iter()
        .filter(|fact| fact.key.fact_kind == FactKind::Ability).collect();
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].key.tick, facts[1].key.tick);
    assert_ne!(facts[0].key.local_ordinal, facts[1].key.local_ordinal);
    let visuals = committed_visuals;
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
        if managed {
            assert!(!w.write_resource::<ScriptVisualEventQueue>().drain().iter().any(|event|event.kind==ScriptVisualEventKind::SkillCast),"managed success is provisional before settlement");
            process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
        }
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
                if managed {
                    assert!(!w.write_resource::<ScriptVisualEventQueue>().drain().iter().any(|event|event.kind==ScriptVisualEventKind::SkillCast),"managed rank cue must await settlement");
                }
                process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
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
    let cast_began = authority.read_resource::<MobaMatch>().elapsed;
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
        let elapsed = authority.read_resource::<MobaMatch>().elapsed;
        // The authored rank-one ranger_patch adds 2 mana/sec for 6 seconds.
        // These twelve steps are inside that lifetime; the bootstrap step
        // before the cast only gets the base regeneration rate.
        (elapsed - began).raw() * i64::from(omoba_template_ids::MOBA_MANA_REGEN_PER_SECOND)
            + (elapsed - cast_began).raw() * 2
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
        let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
        w.write_storage::<Pos>().get_mut(caster).unwrap().0=source;
        w.write_storage::<Pos>().get_mut(target).unwrap().0=source+
            omoba_sim::Vec2::new(Fixed64::from_i32(290),Fixed64::ZERO);
        let abilities=w.read_storage::<Hero>().get(caster).unwrap().abilities.clone();
        assert_eq!(abilities.len(),4);
        for slot in 0..4 {
            // Independent target per slot; full timed effects have separate coverage.
            w.write_resource::<BuffStore>().remove_all_for(target);
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
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
    let destination=origin+omoba_sim::Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO);
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(victim).unwrap().0=destination;
    w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::from_i32(100);
    let original=(*w.read_resource::<BlockedRegions>()).clone();
    w.write_resource::<BlockedRegions>().0.push(isolated_skill_region("bot-dash-fixture",100.0,120.0,100.0));
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
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
    *w.write_resource::<BlockedRegions>()=BlockedRegions(vec![isolated_skill_region("dash-wall",100.0,120.0,100.0)]);
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
fn cast_preflight_formal_60hz_rejects_far_unit_and_normalizes_no_target_heal() {
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let [caster,victim]=hero_pair(&w);
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
        // None-target definitions authoritatively ignore the cursor payload.
        w.write_storage::<Hero>().get_mut(caster).unwrap().ability_cooldowns.remove(&abilities[1]);
        w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::from_i32(100);
        driver.step(&mut w,cast(1,entity,point)).unwrap();
        assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(170));
        assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown(&abilities[1]));
    }
    w.write_storage::<Pos>().get_mut(victim).unwrap().0=source+omoba_sim::Vec2::new(range,Fixed64::ZERO);
    driver.step(&mut w,cast(0,Some(victim.id()),None)).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(victim).unwrap().hp,Fixed64::from_i32(920));
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown(&abilities[0]));
    w.write_storage::<Hero>().get_mut(caster).unwrap().ability_cooldowns.remove(&abilities[1]);
    w.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::from_i32(100);
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
        let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
            target:Some(Vec2I {x:600*1024,y:isolated_skill_origin_y()*1024}),queued:false,
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
            let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
        let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
        target:Some(Vec2I {x:600*1024,y:isolated_skill_origin_y()*1024}),queued:false,
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
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
        target:Some(Vec2I {x:600*1024,y:isolated_skill_origin_y()*1024}),queued:false,
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
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
        } else {omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()))};}
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
        let own = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(isolated_skill_origin_y()));
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
        let own=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
            omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));}
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
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
        omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
        omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
        ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),summoner_loadouts:std::collections::BTreeMap::new(),
        ability_policies:vec![BotAbilityPolicy {ability:"lumen_bolt".into(),intent:BotAbilityIntent::EnemyUnit}]};
    let (mut config,bots)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
    config.warmup=Fixed64::ZERO;config.wave_interval=Fixed64::from_i32(10_000);
    let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();
    let slots=w.read_resource::<MobaMatch>().heroes.clone();
    let entity=|id|slots.iter().find(|s|s.player_id==id).unwrap().entity.unwrap();
    let (support,close,carry,guard)=(entity(1),entity(2),entity(3),entity(4));
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
            player(3,1,BotRole::Mid,"training_ranger")],ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),summoner_loadouts:std::collections::BTreeMap::new(),
        ability_policies:vec![BotAbilityPolicy {ability:"vanguard_strike".into(),intent:BotAbilityIntent::EnemyUnit}]};
    let (mut config,bots)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
    config.warmup=Fixed64::ZERO;config.wave_interval=Fixed64::from_i32(10_000);
    let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();let [caster,target]=hero_pair(&w);
    let ally=w.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==3).unwrap().entity.unwrap();
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
fn jungle_cone_60hz_uses_committed_direction_and_authoritative_full_range_damage() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::runtime::ability_runtime::ManaPool;
    use specs::Builder;
    let (mut w,mut driver)=world(SingleLaneConfig {heroes:["training_scatter_ranger".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    let result=driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=source;
    w.write_storage::<Pos>().get_mut(enemy).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(-1500),Fixed64::ZERO);
    let model=w.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
    let unit=w.read_storage::<Unit>().get(model).unwrap().clone();
    let mut property=w.read_storage::<CProperty>().get(model).unwrap().clone();
    property.hp=Fixed64::from_i32(1000);property.mhp=property.hp;
    property.def_physic=Fixed64::ZERO;property.def_magic=Fixed64::ZERO;
    let targets:Vec<_>=[100,600,610].into_iter().map(|x|w.create_entity().with(unit.clone()).with(property.clone())
        .with(Faction::new(FactionType::HostileNeutral,0))
        .with(ReplicationScope {kind:ReplicationScopeKind::Vision,owner_team:None})
        .with(Pos(source+omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO))).build()).collect();
    w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool=
        Some(ManaPool::new(Fixed64::from_i32(280),Fixed64::from_i32(280)).unwrap());
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Jungle,lane:0,
        escort_player_id:None}],think_interval_ticks:1,ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),
        ability_policies:vec![BotAbilityPolicy {ability:"ranger_scatter".into(),intent:BotAbilityIntent::EnemyCone {min_targets:2}}]};
    run_committed_visibility_wave_b(&mut w,result.tick,0);
    let inputs=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&inputs[0].1.action,Some(PlayerInputEnum::CastAbility(cast))
        if cast.ability_index==2 && cast.target_entity.is_none()
        && cast.target_pos==Some(Vec2I {x:100*1024,y:source.y.raw() as i32})));
    let saved:Vec<_>=targets.iter().map(|e|w.read_storage::<Pos>().get(*e).unwrap().0).collect();
    for e in &targets {w.write_storage::<Pos>().get_mut(*e).unwrap().0=source-omoba_sim::Vec2::new(Fixed64::from_i32(900),Fixed64::ZERO);}
    assert_eq!(role_bot_inputs(&w,&bots).unwrap(),inputs,"private moves cannot change disclosed aim");
    for (e,pos) in targets.iter().zip(saved) {w.write_storage::<Pos>().get_mut(*e).unwrap().0=pos;}
    let membership=w.read_resource::<TeamVisibilityRuntime>().teams[&1].index.current.clone();
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current
        .retain(|id|*id==canonical_entity_id(caster)||*id==canonical_entity_id(targets[0]));
    assert!(!role_bot_inputs(&w,&bots).unwrap().iter().any(|(_,i)|matches!(i.action,Some(PlayerInputEnum::CastAbility(_)))),
        "undisclosed cached units cannot meet the cone threshold");
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current=membership;
    driver.step(&mut w,inputs).unwrap();
    for e in targets {assert_eq!(w.read_storage::<CProperty>().get(e).unwrap().hp,Fixed64::from_i32(890),"full source range, not an aim-centered circle");}
    let heroes=w.read_storage::<Hero>();let hero=heroes.get(caster).unwrap();
    assert!(hero.is_on_cooldown("ranger_scatter"));
    let mana=hero.mana_pool.as_ref().unwrap().current();
    assert!(mana>=Fixed64::from_i32(235)&&mana<Fixed64::from_i32(236));
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
            player(3,1,BotRole::Carry,"training_ranger")],ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),summoner_loadouts:std::collections::BTreeMap::new(),
        ability_policies:vec![BotAbilityPolicy {ability:"lumen_aid".into(),intent:BotAbilityIntent::AllyHeal {below_hp_per_mille:600}}]};
    let (mut config,bots)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
    config.warmup=Fixed64::ZERO;config.wave_interval=Fixed64::from_i32(10_000);
    let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
    let ally=w.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==3).unwrap().entity.unwrap();
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
        ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),summoner_loadouts:std::collections::BTreeMap::new(),
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
    let plan=RoleBotMatchPlan {schema_version:1,map_id:"three_lane_training".into(),think_hz:60,mana_enabled:false,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),summoner_loadouts:std::collections::BTreeMap::new(),
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
    let plan=RoleBotMatchPlan {schema_version:1,map_id:"three_lane_training".into(),think_hz:5,mana_enabled:false,players,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),summoner_loadouts:std::collections::BTreeMap::new()};
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
        if player==1 {assert_eq!(current,pos);} else if player==4 || player==9 {
            assert!(w.read_storage::<HeroCommandQueue>().get(entity).unwrap().active.is_some(),"support may guard a nearby carry");
        } else {assert_ne!(current,pos,"player {player}");}
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
        // Enter actual play before queuing damage; warmup damage is rejected.
        ad.step(&mut a,[]).unwrap();bd.step(&mut b,[]).unwrap();
        // Explicit geometry/placement fixture, not a generated Lua map acceptance.
        let rect=|name:&str,x:f32,y:f32| BlockedRegion { name:name.into(),points:vec![
            vek::Vec2::new(x,y-30.0),vek::Vec2::new(x+1.0,y-30.0),
            vek::Vec2::new(x+1.0,y+30.0),vek::Vec2::new(x,y+30.0)] };
        let camp_home=a.read_storage::<Pos>().get(a.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap()).unwrap().0;
        let regions=BlockedRegions(vec![rect("lane-wall",400.0,0.0),rect("camp-wall",camp_home.x.to_f32_for_render()-50.0,camp_home.y.to_f32_for_render())]);
        for w in [&mut a,&mut b] {
            *w.write_resource::<BlockedRegions>()=regions.clone();
            let hero=hero_pair(w)[0];
            w.write_storage::<Pos>().get_mut(hero).unwrap().0=camp_home-omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
            let camp=w.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
            let mut hit=lethal(hero,camp);
            if let Outcome::Damage { real,.. }=&mut hit { *real=Fixed64::ONE; }
            w.write_resource::<Vec<Outcome>>().push(hit);
            process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();
        }
        let first=ad.step(&mut a,[]).unwrap(); bd.step(&mut b,[]).unwrap();
        run_committed_visibility_wave_b(&mut b,bd.tick(),0);project_tick(&mut a,first);
        let creep=(&a.entities(),&a.read_storage::<Unit>(),&a.read_storage::<Pos>()).join()
            .find(|(_,u,p)| u.id=="single_lane_creep" && p.0.y==Fixed64::ZERO && p.0.x<Fixed64::from_i32(500)).unwrap().0;
        let camp=a.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
        let home=camp_home;
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
            run_committed_visibility_wave_b(&mut b,bd.tick(),0);
            assert_eq!(single_lane_replay_digest(&a),single_lane_replay_digest(&b));
            let positions=a.read_storage::<Pos>();
            if let Some(p)=positions.get(creep) { lane_detour |= p.0.y!=Fixed64::ZERO; lane_crossed |= p.0.x>Fixed64::from_i32(450); }
            let pos=positions.get(camp).unwrap().0;
            camp_detour |= pos.y!=home.y;
            camp_crossed |= pos.x<home.x-Fixed64::from_i32(70);
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
        let first=ad.step(&mut a,[]).unwrap(); bd.step(&mut b,[]).unwrap();
        run_committed_visibility_wave_b(&mut b,bd.tick(),0);project_tick(&mut a,first);
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
        let target=if compiled { omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::from_i32(2275)) }
            else { start+omoba_sim::Vec2::new(Fixed64::from_i32(192),Fixed64::ZERO) };
        let approach=omoba_sim::Vec2::new(Fixed64::ZERO,target.y);
        let mut detoured=false;
        for index in 0..if compiled {1200} else {600} {
            let destination=if compiled && index==0 {approach} else {target};
            let inputs=if index==0 || (compiled && index==600) { vec![(1,PlayerInput { action:Some(PlayerInputEnum::MoveTo(MoveTo {
                target:Some(Vec2I { x:destination.x.raw() as i32,y:destination.y.raw() as i32 }),queued:false })) })] } else { vec![] };
            let accepted=inputs.iter().map(|(player,input)| CanonicalAcceptedInput::from_authoritative_acceptance(
                1,*player,ad.tick()+1,1,canonical_entity_id(hero_pair(&a)[0]),None,input.encode_to_vec())).collect::<Vec<_>>();
            let result=ad.step(&mut a,inputs.clone()).unwrap(); bd.step(&mut b,inputs).unwrap();
            run_committed_visibility_wave_b(&mut b,bd.tick(),0);
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
fn death_split_camp_formal_60hz_rewards_snapshots_full_clear_and_determinism() {
    for (map,camp_id) in [("three_lane_training","blue_krugs"),("three_lane_training","red_krugs"),
        ("three_lane_layered_training","blue_krugs"),("three_lane_layered_training","red_krugs")] {
        let run=|| {
            let (mut w,mut driver)=world(SingleLaneConfig {map_id:Some(map.into()),
                wave_interval:Fixed64::from_i32(10000),..three_lane_config()},SimulationTickProfile::Production60Hz);
            driver.step(&mut w,[]).unwrap();let hero=hero_pair(&w)[0];
            let index=w.read_resource::<MobaMatch>().jungle_camps.iter().position(|c|c.definition.id==camp_id).unwrap();
            let initial_hero=w.read_storage::<Hero>().get(hero).unwrap().clone();
            let initial_gold=w.read_storage::<Gold>().get(hero).unwrap().0;
            let mut count=0;let mut gold=0;let mut xp=0;let mut retired=Vec::new();
            for population in [2,4,4] {
                let bodies:Vec<_>={let s=w.read_resource::<MobaMatch>();let c=&s.jungle_camps[index];
                    std::iter::once(c).chain(c.members.iter()).filter_map(|b|b.entity.map(|e|(e,b.definition.gold,b.definition.xp))).collect()};
                assert_eq!(bodies.len(),population);
                for (entity,g,x) in bodies {
                    count+=1;gold+=g;xp+=x;retired.push(entity);
                    w.write_resource::<Vec<Outcome>>().extend([lethal(hero,entity),lethal(hero,entity)]);
                }
                for _ in 0..3 {driver.step(&mut w,[]).unwrap();}
                assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0,initial_gold+gold as i32);
                let snapshot=extract_snapshot(&mut w,driver.tick() as u32,std::sync::Arc::new(Vec::new()),
                    std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),Vec::new(),Vec::new());
                let s=w.read_resource::<MobaMatch>();let c=&s.jungle_camps[index];
                for e in std::iter::once(c).chain(c.members.iter()).filter_map(|b|b.entity) {
                    assert!(!retired.contains(&e));
                    assert!(snapshot.entities.iter().any(|row|row.entity_id==e.id()));
                }
                if population==2 || count<10 {assert!(c.respawn_at.is_none());}
            }
            assert_eq!(count,10);assert_eq!((gold,xp),(124,193));
            let mut expected=initial_hero;expected.add_moba_experience(xp);
            let actual=w.read_storage::<Hero>().get(hero).unwrap().clone();
            assert_eq!((actual.level,actual.experience),(expected.level,expected.experience));
            let deadline=w.read_resource::<MobaMatch>().jungle_camps[index].respawn_at.unwrap();
            w.write_resource::<MobaMatch>().elapsed=deadline;
            driver.step(&mut w,[]).unwrap();
            let s=w.read_resource::<MobaMatch>();let c=&s.jungle_camps[index];
            assert_eq!(c.respawns,1);assert_eq!(c.members.len(),1);
            assert!(c.entity.is_some() && c.members[0].entity.is_some());
            assert!(!retired.contains(&c.entity.unwrap()));drop(s);
            single_lane_replay_digest(&w)
        };
        assert_eq!(run(),run(),"deterministic split lifecycle {map}");
    }
}

#[test]
fn grouped_camp_shares_aggro_and_respawns_only_after_last_member() {
    let run=|| {
        let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10000),..three_lane_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let hero=hero_pair(&w)[0];
        let (index,leader,children,home)= {
            let state=w.read_resource::<MobaMatch>();let (index,camp)=state.jungle_camps.iter().enumerate().find(|(_,c)|c.members.len()==2).unwrap();
            (index,camp.entity.unwrap(),camp.members.iter().map(|m|m.entity.unwrap()).collect::<Vec<_>>(),omoba_sim::Vec2::new(Fixed64::from_i32(camp.definition.position.0),Fixed64::from_i32(camp.definition.position.1)))
        };
        w.write_storage::<Pos>().get_mut(hero).unwrap().0=home;
        w.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::from_i32(10000);
        let mut hit=lethal(hero,children[0]);if let Outcome::Damage{real,..}=&mut hit{*real=Fixed64::ONE;}
        w.write_resource::<Vec<Outcome>>().push(hit);process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
        let hp=w.read_storage::<CProperty>().get(hero).unwrap().hp;
        driver.step(&mut w,[]).unwrap();
        assert!(w.read_storage::<CProperty>().get(hero).unwrap().hp<hp,"a child hit engages the actual group AI");
        let visuals=w.read_resource::<MobaMatch>().disclosed_unit_attack_states(false);
        assert!(std::iter::once(leader).chain(children.iter().copied()).all(|e|visuals.get(&canonical_entity_id(e)).is_some_and(|s|s.sequence>0)));
        let gold=w.read_storage::<Gold>().get(hero).unwrap().0;
        w.write_resource::<Vec<Outcome>>().push(lethal(hero,leader));
        for _ in 0..2{process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();}w.maintain();
        assert!(w.read_resource::<MobaMatch>().jungle_camps[index].respawn_at.is_none(),"leader death does not start camp respawn");
        assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0,gold+60);
        for entity in &children {w.write_resource::<Vec<Outcome>>().extend([lethal(hero,*entity),lethal(hero,*entity)]);}
        for _ in 0..2{process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();}w.maintain();
        assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0,gold+80,"independent member credit is idempotent");
        assert!(w.read_resource::<MobaMatch>().jungle_camps[index].respawn_at.is_some());
        w.write_storage::<Pos>().get_mut(hero).unwrap().0=omoba_sim::Vec2::ZERO;
        for _ in 0..960 {driver.step(&mut w,[]).unwrap();}
        let state=w.read_resource::<MobaMatch>();let camp=&state.jungle_camps[index];
        assert_eq!(camp.respawns,1);assert_ne!(camp.entity,Some(leader));
        assert!(camp.members.iter().zip(&children).all(|(m,old)|m.entity.is_some()&&m.entity!=Some(*old)));
        drop(state);single_lane_replay_digest(&w)
    };
    assert_eq!(run(),run());
}

#[test]
fn jungle_soft_reset_can_reengage_and_hard_reset_refuses_damage() {
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let hero=hero_pair(&w)[0];
    let camp=w.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
    let home=w.read_storage::<Pos>().get(camp).unwrap().0;
    let hit=|amount|Outcome::Damage{pos:home,phys:Fixed64::ZERO,magi:Fixed64::ZERO,real:Fixed64::from_i32(amount),source:hero,target:camp,damage_profile:0,predeclared:false};
    w.write_storage::<Pos>().get_mut(hero).unwrap().0=home;
    w.write_resource::<Vec<Outcome>>().push(hit(100));process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
    w.write_storage::<Pos>().get_mut(hero).unwrap().0=omoba_sim::Vec2::ZERO;
    for _ in 0..140{driver.step(&mut w,[]).unwrap();}
    {let state=w.read_resource::<MobaMatch>();assert!(state.jungle_camps[0].returning&&!state.jungle_camps[0].hard_reset);}
    let hp=w.read_storage::<CProperty>().get(camp).unwrap().hp;
    w.write_resource::<Vec<Outcome>>().push(hit(100));process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(camp).unwrap().hp,hp,"outside attacker cannot damage a soft return");
    w.write_storage::<Pos>().get_mut(hero).unwrap().0=home;
    w.write_resource::<Vec<Outcome>>().push(hit(1));process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
    assert!(!w.read_resource::<MobaMatch>().jungle_camps[0].returning,"in-range hit reengages soft reset");
    w.write_storage::<Pos>().get_mut(hero).unwrap().0=omoba_sim::Vec2::ZERO;
    let far=home+omoba_sim::Vec2::new(Fixed64::from_i32(5000),Fixed64::ZERO);
    w.write_storage::<Pos>().get_mut(camp).unwrap().0=far;
    let mut hard=false;
    for _ in 0..520 {
        driver.step(&mut w,[]).unwrap();
        if w.read_resource::<MobaMatch>().jungle_camps[0].hard_reset {
            hard=true;assert!(!moba_damage_allowed(&w,camp));break;
        }
    }
    assert!(hard);
    let hp=w.read_storage::<CProperty>().get(camp).unwrap().hp;
    w.write_resource::<Vec<Outcome>>().push(lethal(hero,camp));process_outcomes(&mut w,&mut RuntimeEventVecSink::default()).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(camp).unwrap().hp,hp);
    for _ in 0..900{driver.step(&mut w,[]).unwrap();}
    assert_eq!(w.read_storage::<Pos>().get(camp).unwrap().0,home);
    assert_eq!(w.read_storage::<CProperty>().get(camp).unwrap().hp,Fixed64::from_i32(450));
    assert!(!w.read_resource::<MobaMatch>().jungle_camps[0].returning);
}

#[test]
fn jungle_60hz_aggro_leash_immunity_reward_and_respawn() {
    let run = || {
        let (mut w,mut driver) = world(SingleLaneConfig { wave_interval:Fixed64::from_i32(10_000),
            ..three_lane_config() },SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();
        // Legacy zero-patience policy remains supported; the separate soft
        // reset test exercises the authored production reset phases.
        w.write_resource::<MobaMatch>().jungle_camps[0].definition.patience_seconds=0;
        w.write_resource::<MobaMatch>().jungle_camps[0].definition.soft_reset_seconds=0;
        let hero = hero_pair(&w)[0];
        let camp = w.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
        let home = w.read_storage::<Pos>().get(camp).unwrap().0;
        assert!(w.read_storage::<VisionSource>().get(camp).is_none());
        let f = w.read_storage::<Faction>().get(camp).unwrap().clone();
        assert!(f.is_hostile_to(w.read_storage::<Faction>().get(hero).unwrap()));
        assert!(!Faction::new(FactionType::Neutral,0).is_hostile_to(w.read_storage::<Faction>().get(hero).unwrap()));
        // Fixture placement only. Combat uses the actual damage/outcome pipeline.
        w.write_storage::<Pos>().get_mut(hero).unwrap().0 = home + omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(250));
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
            ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),summoner_loadouts:std::collections::BTreeMap::new(),
            players:vec![player(1,1,BotRole::Top),player(2,2,BotRole::Mid),player(3,2,BotRole::Support)]};
        let (mut config,_)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
        config.warmup=Fixed64::ZERO;config.wave_interval=Fixed64::from_i32(10_000);
        let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();
        let slots=w.read_resource::<MobaMatch>().heroes.clone();
        let entity=|id|slots.iter().find(|s|s.player_id==id).unwrap().entity.unwrap();
        let (bot,a,b)=(entity(1),entity(2),entity(3));
        let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
        let block=|x:f32|isolated_skill_region("combat-navigation-fixture",x-40.0,x+40.0,40.0);
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
        let origin=route[0]-omoba_sim::Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO);
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
            let d=w.read_resource::<MobaMatch>().jungle_camps[0].definition;
            w.write_storage::<Pos>().get_mut(hero).unwrap().0 = omoba_sim::Vec2::new(Fixed64::from_i32(d.position.0-200),Fixed64::from_i32(d.position.1));
        }
        let first = ad.step(&mut a,[]).unwrap(); bd.step(&mut b,[]).unwrap();
        run_committed_visibility_wave_b(&mut b,bd.tick(),0);project_tick(&mut a,first);
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
                    target:Some(if index == 90 { Vec2I {x:0,y:0} } else {
                        let d=a.read_resource::<MobaMatch>().jungle_camps[0].definition;
                        Vec2I {x:(d.position.0-200)*1024,y:d.position.1*1024}
                    }),queued:false })) }));
            } else if !died && (index == 0 || index >= 300)
                && {
                    let d=a.read_resource::<MobaMatch>().jungle_camps[0].definition;
                    let home=omoba_sim::Vec2::new(Fixed64::from_i32(d.position.0),Fixed64::from_i32(d.position.1));
                    (a.read_storage::<Pos>().get(hero_pair(&a)[0]).unwrap().0-home).length_squared()
                        <=Fixed64::from_i32(d.leash_radius)*Fixed64::from_i32(d.leash_radius)
                } && camp.is_some_and(|entity|
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
            run_committed_visibility_wave_b(&mut b,bd.tick(),0);
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
            let top=paths["__moba_lane_0__"];let mid=paths["__moba_lane_1__"];let bottom=paths["__moba_lane_2__"];
            assert!(top.y > top.x);
            assert_eq!(mid.y,mid.x);
            assert!(bottom.x > bottom.y);
        }
    }
}

#[test]
fn authored_five_player_waves_60hz_dual_world_and_both_filtered_replicas() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use std::collections::BTreeSet;
    for seed in [1,42,0x20261004] {
        let mut players=Vec::new();
        for side in 0..2 { for (rank,(role,lane)) in [(BotRole::Top,"top"),(BotRole::Mid,"mid"),
            (BotRole::Carry,"bottom"),(BotRole::Support,"bottom"),(BotRole::Jungle,"mid")].into_iter().enumerate() {
            players.push(RoleBotPlayerPlan {player_id:side*5+rank as u32+1,team_id:side+1,
                hero:"training_luminary".into(),role,lane:lane.into(),bot:false});
        } }
        let plan=RoleBotMatchPlan {schema_version:1,map_id:"three_lane_training".into(),think_hz:5,mana_enabled:false,
            players,ability_policies:Vec::new(),ability_learning:Vec::new(),sustain:None,item_builds:Vec::new(),
            summoner_loadouts:std::collections::BTreeMap::new()};
        let (mut config,_)=plan.compile(seed,SimulationTickProfile::Production60Hz).unwrap();
        config.warmup=Fixed64::ZERO;
        assert!(config.authored_minion_waves);assert_eq!(config.wave_interval,Fixed64::from_i32(30));
        let (mut a,mut ad)=world(config.clone(),SimulationTickProfile::Production60Hz);
        let (mut b,mut bd)=world(config,SimulationTickProfile::Production60Hz);
        let first=ad.step(&mut a,[]).unwrap();bd.step(&mut b,[]).unwrap();
        run_committed_visibility_wave_b(&mut b,bd.tick(),0);project_tick(&mut a,first);
        let allow=TeamProjectorConfig::default().component_allowlist;
        let starts=a.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(ad.tick()+1,60,seed);
        let mut replicas:Vec<_>=starts.into_iter().map(|(team,start)| {
            let replica=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),BTreeSet::new()).unwrap();
            let mut stepper=SpecsDisclosedWorldStepper::from_start(&start,allow.clone(),BTreeSet::new());
            stepper.script_registry.insert_manifest(crate::get_manifest());populate_ability_registry(&mut stepper.filtered.world,&stepper.script_registry);
            stepper.bootstrap_membership(replica.world()).unwrap();(team,replica,stepper)
        }).collect();
        let (mut caster_struck,mut siege_spawned,mut siege_struck)=(false,false,false);
        for index in 0..4500 {
            let result=ad.step(&mut a,[]).unwrap();bd.step(&mut b,[]).unwrap();
            run_committed_visibility_wave_b(&mut b,bd.tick(),0);
            assert_eq!(single_lane_replay_digest(&a),single_lane_replay_digest(&b),"wave seed={seed} tick={index}");
            {let state=a.read_resource::<MobaMatch>();let states=state.disclosed_unit_attack_states(false);
                let units=a.read_storage::<Unit>();
                for (entity,unit) in (&a.entities(),&units).join() {
                    let struck=states.get(&canonical_entity_id(entity)).is_some_and(|s|s.phase==2 && s.sequence>0);
                    caster_struck |= unit.id=="lane_caster" && struck;
                    siege_spawned |= unit.id=="lane_siege";siege_struck |= unit.id=="lane_siege" && struck;
                }
            }
            project_tick(&mut a,result);
            let frames=a.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
            let expected=a.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(ad.tick()+1,60,seed);
            for (team,replica,stepper) in &mut replicas {
                let frame=frames[team].frame.clone();
                assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
                assert!(frames[team].wire_bytes.len()<=64*1024);
                assert!(matches!(replica.apply_frame(frame,stepper).unwrap(),FrameApplyResult::Applied {..}));
                let fresh=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team],allow.clone(),BTreeSet::new()).unwrap();
                assert_eq!(replica.canonical_team_hash(),fresh.canonical_team_hash(),"wave seed={seed} team={team} tick={index}");
                assert!(stepper.filtered.world.try_fetch::<MobaMatch>().is_none());
            }
        }
        assert!(caster_struck && siege_spawned && siege_struck,"actual caster and siege strikes seed={seed}");
        assert_eq!(a.read_resource::<MobaMatch>().waves,3);
        println!("authored 5v5 seed={seed} ticks=4500 dual_world_steps=9000 replica_steps=9000 three waves and actual caster/siege strikes passed");
    }
}

#[test]
fn three_lane_base_damage_requires_inhibitor_and_nexus_towers_and_finishes_once() {
    let (mut world,mut driver) = world(three_lane_config(),SimulationTickProfile::Production60Hz);
    driver.step(&mut world,[]).unwrap();
    let source = hero_pair(&world)[0];
    let towers: Vec<_> = world.read_resource::<MobaMatch>().lane_tower_layers.iter().flat_map(|lane|lane.iter().map(|layer|layer[1].unwrap())).collect();
    let tower_count=towers.len();assert_eq!(tower_count,9);
    let base = world.read_resource::<MobaMatch>().bases[1].unwrap();
    let hp = world.read_storage::<CProperty>().get(base).unwrap().hp;
    for (index,tower) in towers.into_iter().take(3).enumerate() {
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
        assert!(!state.base_unlocked(1),"lane clearance alone cannot bypass inhibitors and nexus towers");
        assert_eq!(state.towers[1].is_none(),index + 1 == tower_count);
    }
    destroy_exposed_base_chain(&mut world,source,1);
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
            assert!(!state.base_unlocked(1));
            assert!(!state.base_unlocked(0));
        }
    }
    destroy_exposed_base_chain(&mut world,source,1);
    assert!(moba_damage_allowed(&world,base));
}

fn destroy_exposed_base_chain(world:&mut World,source:specs::Entity,side:usize) {
    let team=world.read_resource::<MobaMatch>().config.teams[side] as i32;
    for id in ["moba_inhibitor","moba_nexus_tower","moba_nexus_tower"] {
        let target=(&world.entities(),&world.read_storage::<Unit>(),&world.read_storage::<Faction>()).join()
            .find(|(e,u,f)|u.id==id && f.team_id==team && moba_damage_allowed(world,*e)).map(|(e,_,_)|e).expect("exposed authored structure");
        world.write_resource::<Vec<Outcome>>().push(lethal(source,target));
        process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();
        if id=="moba_inhibitor" {assert!(!world.read_resource::<MobaMatch>().base_unlocked(side));}
        process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();world.maintain();
    }
    assert!(world.read_resource::<MobaMatch>().base_unlocked(side));
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
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
    w.write_storage::<Pos>().get_mut(owner).unwrap().0=origin;
    let mut points:Vec<_>=w.read_resource::<MobaMatch>().jungle_camps.iter().map(|camp| {
        let (x,y)=camp.definition.position;
        omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))
    }).collect();
    assert_eq!(points.len(),16,"compiled training fixture");
    points.sort_by_key(|point|(*point-origin).length_squared().raw());
    let original=(*w.read_resource::<BlockedRegions>()).clone();
    let block=|point:omoba_sim::Vec2| {
        let x=point.x.to_f32_for_render();let y=point.y.to_f32_for_render();
        BlockedRegion {name:"camp-patrol-fixture".into(),points:vec![
            vek::Vec2::new(x-40.0,y-40.0),vek::Vec2::new(x+40.0,y-40.0),
            vek::Vec2::new(x+40.0,y+40.0),vek::Vec2::new(x-40.0,y+40.0)]}
    };
    for (index,point) in points.iter().enumerate() { if index!=1 {
        w.write_resource::<BlockedRegions>().0.push(block(*point));
    } }
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Jungle,lane:1,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:None,item_builds:Vec::new()};
    disclose_lane_fixture(&mut w,driver.tick(),0);
    let input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&input[0].1.action,Some(PlayerInputEnum::AttackMove(a))
        if a.target.as_ref().is_some_and(|p|i64::from(p.x)==points[1].x.raw() && i64::from(p.y)==points[1].y.raw())),"patrol input={input:?} points={points:?}");
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
        omoba_sim::Vec2::new(Fixed64::from_i32(650),Fixed64::from_i32(-120));
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
    disclose_lane_fixture(&mut w,driver.tick(),0);
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
    let disclose=|w:&mut World,tick| {
        disclose_lane_fixture(w,tick,0);
        let other_towers:Vec<_>=w.read_resource::<MobaMatch>().lane_tower_layers.iter().flatten()
            .flatten().filter_map(|entity|entity.filter(|entity|*entity!=tower).map(canonical_entity_id)).collect();
        w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.retain(|id|!other_towers.contains(id));
    };
    let tower_pos=w.read_storage::<Pos>().get(tower).unwrap().0;
    let own=tower_pos-omoba_sim::Vec2::new(Fixed64::from_i32(650),Fixed64::from_i32(-120));
    w.write_storage::<Pos>().get_mut(owner).unwrap().0=own;
    {let mut health=w.write_storage::<CProperty>();let p=health.get_mut(owner).unwrap();
        p.hp=Fixed64::from_i32(10_000);p.mhp=p.hp;}
    let wave=lane_creeps(&w,1)[0];
    w.write_storage::<Pos>().get_mut(wave).unwrap().0=tower_pos-
        omoba_sim::Vec2::new(Fixed64::from_i32(50),Fixed64::ZERO);
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Jungle,lane:1,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:None,item_builds:Vec::new()};
    disclose(&mut w,driver.tick());
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
    disclose(&mut w,driver.tick());
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"same travel is not resubmitted");
    let mut handed_off=false;
    for _ in 0..180 {
        disclose(&mut w,driver.tick());
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
    disclose_lane_fixture(&mut w,driver.tick(),0);
    let hold=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(hold.len(),1);
    assert!(matches!(&hold[0].1.action,Some(PlayerInputEnum::HoldPosition(h)) if !h.queued));
    driver.step(&mut w,hold).unwrap();
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active,Some(HeroCommand::HoldPosition));
    disclose_lane_fixture(&mut w,driver.tick(),0);
    let wave=lane_creeps(&w,1)[0];
    w.write_storage::<Pos>().get_mut(wave).unwrap().0=tower_pos-
        omoba_sim::Vec2::new(Fixed64::from_i32(50),Fixed64::ZERO);
    assert!(role_bot_inputs(&w,&bots).unwrap().is_empty(),"uncommitted wave cannot release hold");
    disclose_lane_fixture(&mut w,driver.tick(),0);
    let attack=role_bot_inputs(&w,&bots).unwrap();
    assert_eq!(attack.len(),1);
    assert!(matches!(&attack[0].1.action,Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==tower.id()));
    driver.step(&mut w,attack).unwrap();
    assert!(matches!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active,
        Some(HeroCommand::AttackTarget {target,..}) if target==tower));
    disclose_lane_fixture(&mut w,driver.tick(),0);
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
fn siege_wave_yield_60hz_moves_behind_disclosed_creep_and_holds_without_one() {
    use omoba_core::runtime::native::moba_match::bots::*;
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),
        ..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();
    let owner=hero_pair(&w)[0];
    let tower=w.read_resource::<MobaMatch>().lane_towers[1][1].unwrap();
    let tower_pos=w.read_storage::<Pos>().get(tower).unwrap().0;
    let base=w.read_resource::<MobaMatch>().bases[0].unwrap();
    let inward=(tower_pos-w.read_storage::<Pos>().get(base).unwrap().0).normalized();
    let along=|units:i32|tower_pos-inward*Fixed64::from_i32(units);
    let hero_pos=along(800);
    let creep_pos=along(700);
    let stage=along(1400);
    w.write_storage::<Pos>().get_mut(owner).unwrap().0=hero_pos;
    {let mut health=w.write_storage::<CProperty>();let hero=health.get_mut(owner).unwrap();
        hero.hp=Fixed64::from_i32(10_000);hero.mhp=hero.hp;}
    let creeps=lane_creeps(&w,1);
    let wave=creeps[0];
    for creep in &creeps {
        w.write_storage::<Pos>().get_mut(*creep).unwrap().0=if *creep==wave {creep_pos} else {along(-4000)};
        let mut health=w.write_storage::<CProperty>();
        let vital=health.get_mut(*creep).unwrap();
        vital.hp=Fixed64::from_i32(10_000);vital.mhp=vital.hp;
    }
    let bots=RoleBotConfig {assignments:vec![BotAssignment {player_id:1,role:BotRole::Mid,lane:1,
        escort_player_id:None}],think_interval_ticks:1,ability_policies:Vec::new(),ability_learning:Vec::new(),
        sustain:None,item_builds:Vec::new()};
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let yield_input=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&yield_input[0].1.action,Some(PlayerInputEnum::MoveTo(m))
        if !m.queued && m.target.as_ref().is_some_and(|p|
            i64::from(p.x)==stage.x.raw() && i64::from(p.y)==stage.y.raw())),
        "ahead of a disclosed nearby wave must MoveTo behind it: {yield_input:?}");
    assert!(!yield_input.iter().any(|(_,input)|matches!(&input.action,
        Some(PlayerInputEnum::AttackTarget(_)|PlayerInputEnum::AttackMove(_)))));
    driver.step(&mut w,yield_input).unwrap();
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active,Some(HeroCommand::MoveTo {pos:stage}));
    w.write_storage::<Pos>().get_mut(owner).unwrap().0=hero_pos;
    let wave_key=canonical_entity_id(wave);
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.remove(&wave_key);
    let hidden=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&hidden[0].1.action,Some(PlayerInputEnum::HoldPosition(h)) if !h.queued),
        "a private creep position cannot release the tower alert: {hidden:?}");
    w.write_resource::<TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.insert(wave_key);
    w.write_storage::<Pos>().get_mut(wave).unwrap().0=along(1600);
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let far=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&far[0].1.action,Some(PlayerInputEnum::HoldPosition(_))),
        "a disclosed creep outside 700 still holds: {far:?}");
    w.write_storage::<Pos>().get_mut(owner).unwrap().0=along(400);
    w.write_storage::<Pos>().get_mut(wave).unwrap().0=along(50);
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let attack=role_bot_inputs(&w,&bots).unwrap();
    assert!(matches!(&attack[0].1.action,Some(PlayerInputEnum::AttackTarget(a)) if a.target_id==tower.id()),
        "attack only after the disclosed creep is inside 650: {attack:?}");
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
fn tower_plating_formal_60hz_skill_wallet_snapshot_native_defense_and_cutoff() {
    let (mut w,mut driver)=world(SingleLaneConfig {tower_hp:Fixed64::from_i32(1000),tower_plating_rules:Some(TowerPlatingRules::training()),
        wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [hero,_]=hero_pair(&w);let target=w.read_resource::<MobaMatch>().towers[1].unwrap();
    assert_eq!(w.read_resource::<BuffStore>().iter_for(target).find(|(id,_)|*id=="tower_plating_status").unwrap().1.payload["health_segments"]["remaining"],5);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(hero).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(target).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
    {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(target).unwrap();p.hp=Fixed64::from_i32(801);p.def_magic=Fixed64::ZERO;}
    let step=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {ability_index:0,
        target_entity:Some(target.id()),target_pos:None}))})]).unwrap();
    let hp=w.read_storage::<CProperty>().get(target).unwrap().hp;
    let plates=((Fixed64::from_i32(1000)-hp).raw()*5/Fixed64::from_i32(1000).raw()) as i32;
    assert!(plates>0 && plates<5,"generated skill crosses a nonlethal outer plate");
    assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0,125*plates);
    assert!(step.facts.iter().any(|f|matches!(&f.fact,ObservableFact::OwnerEconomy {state,..} if state.player_id==1 && state.economy.gold==125*plates)),
        "same-step authoritative HUD economy includes plate gold");
    let snapshot=extract_snapshot(&mut w,driver.tick() as u32,std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),Vec::new(),Vec::new());
    let buff=snapshot.entities.iter().find(|e|e.entity_id==target.id()).unwrap().buffs.iter().find(|b|b.buff_id=="tower_plating").unwrap();
    assert!(buff.remaining_secs>830. && buff.remaining_secs<=840.);
    assert_eq!(serde_json::from_str::<serde_json::Value>(&buff.payload_json).unwrap()["stacks"],plates);
    let status=snapshot.entities.iter().find(|e|e.entity_id==target.id()).unwrap().buffs.iter().find(|b|b.buff_id=="tower_plating_status").unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(&status.payload_json).unwrap()["health_segments"]["remaining"],5-plates);
    w.write_storage::<Pos>().get_mut(hero).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(1000),Fixed64::ZERO);
    let armor=w.read_storage::<Unit>().get(target).unwrap().base_armor+Fixed64::from_i32(20*plates);
    w.write_resource::<Vec<DamageInstance>>().push(DamageInstance::new_attack(hero,target,Fixed64::from_i32(100)));
    driver.step(&mut w,[]).unwrap();
    let expected=Fixed64::from_i32(100)-Fixed64::from_i32(100)*(armor/(armor+Fixed64::from_i32(100)));
    assert_eq!(hp-w.read_storage::<CProperty>().get(target).unwrap().hp,expected,"next native packet uses earned fortification exactly once");
    let before=w.read_storage::<Gold>().get(hero).unwrap().0;w.write_resource::<MobaMatch>().elapsed=Fixed64::from_i32(840);
    w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage {target,amount:Fixed64::from_i32(200)});
    driver.step(&mut w,[]).unwrap();assert!(!w.read_resource::<BuffStore>().has(target,"tower_plating"));
    assert!(!w.read_resource::<BuffStore>().has(target,"tower_plating_status"));
    assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0,before);
    assert!(w.read_resource::<MobaMatch>().heroes.iter().all(|s|s.kills==0 && s.assists==0));
}

#[test]
fn building_rewards_formal_60hz_generated_skill_updates_team_wallets_without_hero_score() {
    let (mut w,mut driver)=world(SingleLaneConfig {building_reward_rules:BuildingRewardRules::training(),
        additional_players:vec![extra_player(3,1)],wave_interval:Fixed64::from_i32(10_000),
        ..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,_]=hero_pair(&w);
    let target=w.read_resource::<MobaMatch>().towers[1].unwrap();
    let source=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=source;
    w.write_storage::<Pos>().get_mut(target).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
    let helper=w.read_resource::<MobaMatch>().heroes.iter().find(|s|s.player_id==3).unwrap().entity.unwrap();
    w.write_storage::<Pos>().get_mut(helper).unwrap().0=source+omoba_sim::Vec2::new(Fixed64::from_i32(5000),Fixed64::ZERO);
    {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(target).unwrap();p.hp=Fixed64::from_i32(80);p.def_magic=Fixed64::ZERO;}
    let settled=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:0,target_entity:Some(target.id()),target_pos:None,
    }))})]).unwrap();
    assert_eq!(w.read_storage::<Gold>().get(caster).unwrap().0,500,"generated skill pays local pool, team gold and the first tower bonus");
    assert!(settled.facts.iter().any(|fact|matches!(&fact.fact,ObservableFact::OwnerEconomy {state,..}
        if state.player_id==1 && state.economy.gold==500)),"same-step owner economy includes the first tower award");
    assert_eq!(w.read_storage::<Gold>().get(helper).unwrap().0,100,"distant teammate receives global gold");
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("lumen_bolt"));
    assert!(w.read_resource::<MobaMatch>().heroes.iter().all(|s|s.kills==0 && s.assists==0));
}

#[test]
fn objective_buff_formal_60hz_generated_skill_grants_team_stats_and_snapshot() {
    let (mut w,mut driver)=world(SingleLaneConfig {additional_players:vec![extra_player(3,1)],
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
    let target=w.read_resource::<MobaMatch>().jungle_camps.iter().find(|c|c.definition.objective_buff
        .is_some_and(|b|b.lost_on_death)).unwrap().entity.unwrap();
    let home=w.read_storage::<Pos>().get(target).unwrap().0;
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=home;
    {let mut properties=w.write_storage::<CProperty>();let prop=properties.get_mut(target).unwrap();prop.hp=Fixed64::from_i32(80);prop.def_magic=Fixed64::ZERO;}
    driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:0,target_entity:Some(target.id()),target_pos:None,
    }))})]).unwrap();
    let ally=w.read_resource::<MobaMatch>().heroes.iter().find(|s|s.player_id==3).unwrap().entity.unwrap();
    driver.step(&mut w,[]).unwrap(); // commit any queued Death before checking the objective reward
    {let buffs=w.read_resource::<BuffStore>();assert!(buffs.has(caster,"objective_might") && buffs.has(ally,"objective_might"));
        assert!(!buffs.has(enemy,"objective_might"));
        assert_eq!(UnitStats::from_refs(&buffs,false).normal_attack_physical(Fixed64::from_i32(50),caster),Fixed64::from_i32(70));}
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("lumen_bolt"));
    let snapshot=extract_snapshot(&mut w,driver.tick() as u32,std::sync::Arc::new(Vec::new()),
        std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),Vec::new(),Vec::new());
    let rendered=snapshot.entities.iter().find(|entity|entity.entity_id==caster.id()).unwrap();
    let buff=rendered.buffs.iter().find(|buff|buff.buff_id=="objective_might").unwrap();
    assert!(buff.remaining_secs>179.0 && buff.remaining_secs<=180.0);
    assert_eq!(serde_json::from_str::<serde_json::Value>(&buff.payload_json).unwrap()["baseattack_bonus_damage"],20*1024);
}

#[test]
fn jungle_nonhero_credit_formal_60hz_outcomes_split_player_rewards_from_team_objective() {
    for owned in [true,false] {
        let (mut w,mut driver)=world(SingleLaneConfig {additional_players:vec![extra_player(3,1)],
            wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let [hero,enemy]=hero_pair(&w);
        let target=w.read_resource::<MobaMatch>().jungle_camps.iter().find(|c|c.definition.objective_buff
            .is_some_and(|b|!b.lost_on_death)).unwrap().entity.unwrap();
        let home=w.read_storage::<Pos>().get(target).unwrap().0;
        let unit=w.create_entity().with(Unit::new("jungle_credit_fixture".into(),"credit".into(),UnitType::Creep))
            .with(Faction::new(FactionType::Ally,1)).with(Pos(home)).with(CProperty {hp:Fixed64::from_i32(100),
                mhp:Fixed64::from_i32(100),msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        if owned {w.write_storage::<SummonedUnit>().insert(unit,SummonedUnit::new(hero,None,0.0)).unwrap();}
        w.write_resource::<Vec<Outcome>>().push(Outcome::Damage {source:unit,target,pos:home,phys:Fixed64::ZERO,
            magi:Fixed64::ZERO,real:Fixed64::from_i32(100_000),damage_profile:0,predeclared:false});
        driver.step(&mut w,[]).unwrap();driver.step(&mut w,[]).unwrap();
        assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0,if owned {150}else{0});
        assert_eq!((w.read_storage::<Hero>().get(hero).unwrap().level,w.read_storage::<Hero>().get(hero).unwrap().experience),if owned {(3,80)}else{(1,0)});
        let buffs=w.read_resource::<BuffStore>();assert!(buffs.has(hero,"objective_guard"));assert!(!buffs.has(enemy,"objective_guard"));
    }
}

#[test]
fn objective_minion_aura_formal_60hz_changes_real_attack_defense_snapshot_and_death() {
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [hero,_]=hero_pair(&w);
    let camp=w.read_resource::<MobaMatch>().jungle_camps.iter().find(|c|c.definition.objective_buff.is_some_and(|b|b.minion_aura.is_some())).unwrap().entity.unwrap();
    w.write_resource::<Vec<Outcome>>().push(lethal(hero,camp));
    driver.step(&mut w,[]).unwrap();driver.step(&mut w,[]).unwrap();
    let creep=lane_creeps(&w,1)[0];let enemy=lane_creeps(&w,2)[0];
    let origin=omoba_sim::Vec2::new(Fixed64::from_i32(10_000),Fixed64::from_i32(10_000));
    {let mut positions=w.write_storage::<Pos>();for (_,p) in (&w.entities(),&mut positions).join(){p.0=-origin;}
        positions.get_mut(creep).unwrap().0=origin;positions.get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::ONE,Fixed64::ZERO);
        positions.get_mut(hero).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(800),Fixed64::ZERO);}
    {let mut props=w.write_storage::<CProperty>();for entity in [creep,enemy]{props.get_mut(entity).unwrap().msd=Fixed64::ZERO;}}
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    w.write_resource::<DeltaTime>().0=Fixed64::ONE;w.write_resource::<Vec<DamageInstance>>().clear();
    assert!(begin_moba_match_tick(&mut w));
    let base=w.read_storage::<Unit>().get(creep).unwrap().base_damage;
    assert!(w.read_resource::<Vec<DamageInstance>>().iter().any(|d|d.source.source_entity==creep && d.target==enemy
        && d.damage_types.physical==Fixed64::from_i32(base+15)),"real lane AI uses aura attack bonus");
    w.write_resource::<Vec<DamageInstance>>().clear();
    let before=w.read_storage::<CProperty>().get(creep).unwrap().hp;
    let armor=w.read_storage::<Unit>().get(creep).unwrap().base_armor+Fixed64::from_i32(10);
    w.write_resource::<Vec<DamageInstance>>().push(DamageInstance::new_attack(enemy,creep,Fixed64::from_i32(100)));
    driver.step(&mut w,[]).unwrap();
    let expected=Fixed64::from_i32(100)-Fixed64::from_i32(100)*(armor/(armor+Fixed64::from_i32(100)));
    assert_eq!(before-w.read_storage::<CProperty>().get(creep).unwrap().hp,expected,"formal damage system uses aura defense once");
    let snapshot=extract_snapshot(&mut w,driver.tick() as u32,std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),Vec::new(),Vec::new());
    assert!(snapshot.entities.iter().find(|e|e.entity_id==creep.id()).unwrap().buffs.iter().any(|b|b.buff_id=="siege_aura"));
    assert!(!w.read_resource::<BuffStore>().has(enemy,"siege_aura"));
    w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage {target:hero,amount:Fixed64::from_i32(100_000)});
    driver.step(&mut w,[]).unwrap();driver.step(&mut w,[]).unwrap();
    assert!(w.read_resource::<MobaMatch>().heroes[0].entity.is_none());
    assert!(!w.read_resource::<BuffStore>().has(creep,"siege_aura"));
}

#[test]
fn permanent_objective_formal_60hz_multi_reward_snapshot_survives_time_and_respawn() {
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [hero,_]=hero_pair(&w);
    for n in 1..=3 {
        let camp=w.read_resource::<MobaMatch>().jungle_camps.iter().find(|c|!c.definition.additional_objective_buffs.is_empty()).unwrap().entity.unwrap();
        w.write_resource::<Vec<Outcome>>().push(Outcome::Damage {source:hero,target:camp,pos:omoba_sim::Vec2::ZERO,
            phys:Fixed64::ZERO,magi:Fixed64::ZERO,real:Fixed64::from_i32(100_000),damage_profile:0,predeclared:false});
        driver.step(&mut w,[]).unwrap();driver.step(&mut w,[]).unwrap();
        assert!(w.read_resource::<BuffStore>().has(hero,"objective_guard"));
        assert_eq!(UnitStats::from_refs(&w.read_resource::<BuffStore>(),false).normal_attack_physical(Fixed64::from_i32(50),hero),Fixed64::from_i32(50+5*n));
        if n<3 {w.write_resource::<MobaMatch>().elapsed+=Fixed64::from_i32(90);driver.step(&mut w,[]).unwrap();}
    }
    w.write_resource::<MobaMatch>().elapsed=Fixed64::from_i32(1_000_000);driver.step(&mut w,[]).unwrap();
    assert!(!w.read_resource::<BuffStore>().has(hero,"objective_guard"));
    let snapshot=extract_snapshot(&mut w,driver.tick() as u32,std::sync::Arc::new(Vec::new()),
        std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),Vec::new(),Vec::new());
    let b=snapshot.entities.iter().find(|e|e.entity_id==hero.id()).unwrap().buffs.iter().find(|b|b.buff_id=="objective_legacy").unwrap();
    assert_eq!(b.remaining_secs,-1.0);assert_eq!(serde_json::from_str::<serde_json::Value>(&b.payload_json).unwrap()["objective_stacks"],3);
    w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage {target:hero,amount:Fixed64::from_i32(100_000)});
    driver.step(&mut w,[]).unwrap();driver.step(&mut w,[]).unwrap();
    assert!(w.read_resource::<MobaMatch>().heroes[0].entity.is_none());
    w.write_resource::<MobaMatch>().elapsed+=Fixed64::from_i32(60);driver.step(&mut w,[]).unwrap();
    let next=w.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    assert_eq!(UnitStats::from_refs(&w.read_resource::<BuffStore>(),false).normal_attack_physical(Fixed64::from_i32(50),next),Fixed64::from_i32(65));
    assert_eq!(w.read_resource::<BuffStore>().iter_for(next).find(|(id,_)|*id=="objective_legacy").unwrap().1.remaining,
        omoba_core::runtime::ability_runtime::buff_store::PERMANENT_BUFF_DURATION);
}

#[test]
fn personal_buff_formal_60hz_camp_skill_and_hero_kill_preserve_remaining_time() {
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [holder,killer]=hero_pair(&w);
    let camp=w.read_resource::<MobaMatch>().jungle_camps.iter().find(|c|c.definition.personal_buff.is_some()).unwrap().entity.unwrap();
    let home=w.read_storage::<Pos>().get(camp).unwrap().0;w.write_storage::<Pos>().get_mut(holder).unwrap().0=home;
    {let mut p=w.write_storage::<CProperty>();let c=p.get_mut(camp).unwrap();c.hp=Fixed64::from_i32(40);c.def_magic=Fixed64::ZERO;}
    driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:0,target_entity:Some(camp.id()),target_pos:None,
    }))})]).unwrap();driver.step(&mut w,[]).unwrap();
    assert!(w.read_resource::<BuffStore>().has(holder,"jungle_guard"));
    assert!(!w.read_resource::<BuffStore>().has(killer,"jungle_guard"));
    w.write_resource::<MobaMatch>().elapsed+=Fixed64::from_i32(60);
    w.write_resource::<Vec<Outcome>>().push(Outcome::Damage {source:killer,target:holder,pos:home,phys:Fixed64::ZERO,
        magi:Fixed64::ZERO,real:Fixed64::from_i32(100_000),damage_profile:0,predeclared:false});
    driver.step(&mut w,[]).unwrap();driver.step(&mut w,[]).unwrap();
    {let buffs=w.read_resource::<BuffStore>();
        let b=buffs.iter_for(killer).find(|(id,_)|*id=="jungle_guard").unwrap().1;
        assert!(b.remaining>Fixed64::from_i32(59) && b.remaining<Fixed64::from_i32(61));
        assert!(!buffs.has(holder,"jungle_guard"));
        assert_eq!(UnitStats::from_refs(&buffs,false).final_armor(Fixed64::ZERO,killer),Fixed64::from_i32(10));}
    let snapshot=extract_snapshot(&mut w,driver.tick() as u32,std::sync::Arc::new(Vec::new()),
        std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),Vec::new(),Vec::new());
    assert!(snapshot.entities.iter().find(|e|e.entity_id==killer.id()).unwrap().buffs.iter().any(|b|b.buff_id=="jungle_guard" && b.remaining_secs<61.0));
    w.write_resource::<GamePause>().is_paused=true;let elapsed=w.read_resource::<MobaMatch>().elapsed;
    for _ in 0..60 {driver.step(&mut w,[]).unwrap();}assert_eq!(w.read_resource::<MobaMatch>().elapsed,elapsed);
    w.write_resource::<GamePause>().is_paused=false;w.write_resource::<MobaMatch>().elapsed+=Fixed64::from_i32(61);
    driver.step(&mut w,[]).unwrap();assert!(!w.read_resource::<BuffStore>().has(killer,"jungle_guard"));
    let next=w.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    let camp=w.read_resource::<MobaMatch>().jungle_camps.iter().find(|c|c.definition.personal_buff.is_some()).unwrap().entity.unwrap();
    w.write_resource::<Vec<Outcome>>().push(Outcome::Damage {source:next,target:camp,pos:home,phys:Fixed64::ZERO,
        magi:Fixed64::ZERO,real:Fixed64::from_i32(100_000),damage_profile:0,predeclared:false});
    driver.step(&mut w,[]).unwrap();driver.step(&mut w,[]).unwrap();
    assert!(w.read_resource::<BuffStore>().has(next,"jungle_guard"));
    w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage {target:next,amount:Fixed64::from_i32(100_000)});
    driver.step(&mut w,[]).unwrap();driver.step(&mut w,[]).unwrap();
    assert!(w.read_resource::<MobaMatch>().heroes[0].entity.is_none());
    assert!(!w.read_resource::<BuffStore>().has(next,"jungle_guard"));
    assert!(!w.read_resource::<BuffStore>().has(killer,"jungle_guard"));
}

#[test]
fn objective_stacks_formal_60hz_actual_respawns_publish_multiplied_stats_and_expire() {
    let (mut w,mut driver)=world(SingleLaneConfig {additional_players:vec![extra_player(3,1)],
        wave_interval:Fixed64::from_i32(10_000),..three_lane_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [hero,enemy]=hero_pair(&w);
    for n in 1..=3 {
        let target=w.read_resource::<MobaMatch>().jungle_camps.iter().find(|c|c.definition.objective_buff
            .is_some_and(|b|b.buff_id=="objective_guard")).unwrap().entity.unwrap();
        w.write_resource::<Vec<Outcome>>().push(Outcome::Damage {source:hero,target,pos:omoba_sim::Vec2::ZERO,
            phys:Fixed64::ZERO,magi:Fixed64::ZERO,real:Fixed64::from_i32(100_000),damage_profile:0,predeclared:false});
        driver.step(&mut w,[]).unwrap();driver.step(&mut w,[]).unwrap();
        {let buffs=w.read_resource::<BuffStore>();
            assert_eq!(UnitStats::from_refs(&buffs,false).final_armor(Fixed64::ZERO,hero),Fixed64::from_i32(15*n));
            assert!(!buffs.has(enemy,"objective_guard"));}
        if n<3 {w.write_resource::<MobaMatch>().elapsed+=Fixed64::from_i32(90);driver.step(&mut w,[]).unwrap();}
    }
    let snapshot=extract_snapshot(&mut w,driver.tick() as u32,std::sync::Arc::new(Vec::new()),
        std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),Vec::new(),Vec::new());
    let entry=snapshot.entities.iter().find(|e|e.entity_id==hero.id()).unwrap().buffs.iter()
        .find(|b|b.buff_id=="objective_guard").unwrap();
    let payload:serde_json::Value=serde_json::from_str(&entry.payload_json).unwrap();
    assert_eq!(payload["objective_stacks"],3);assert_eq!(payload["physical_armor_bonus"],45*1024);
    w.write_resource::<GamePause>().is_paused=true;let elapsed=w.read_resource::<MobaMatch>().elapsed;
    for _ in 0..60 {driver.step(&mut w,[]).unwrap();}
    assert_eq!(w.read_resource::<MobaMatch>().elapsed,elapsed);
    w.write_resource::<GamePause>().is_paused=false;w.write_resource::<MobaMatch>().elapsed+=Fixed64::from_i32(121);
    driver.step(&mut w,[]).unwrap();assert!(!w.read_resource::<BuffStore>().has(hero,"objective_guard"));
}

#[test]
fn death_streak_formal_60hz_skill_discounts_base_only_and_never_double_pays() {
    let (mut w,mut driver)=world(SingleLaneConfig {death_streak_rules:DeathStreakRules::training(),
        first_kill_bonus_gold:100,kill_bounty_rules:KillBountyRules::training(),
        wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
    {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(enemy).unwrap();p.hp=Fixed64::from_i32(40);p.def_magic=Fixed64::ZERO;}
    {let mut state=w.write_resource::<MobaMatch>();state.heroes[1].unanswered_deaths=7;
        state.heroes[1].kill_streak=3;state.heroes[0].unanswered_deaths=5;}
    driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:0,target_entity:Some(enemy.id()),target_pos:None,
    }))})]).unwrap();
    assert_eq!(w.read_storage::<Gold>().get(caster).unwrap().0,350,"100 discounted base +150 shutdown +100 first kill");
    assert_eq!(w.read_resource::<MobaMatch>().heroes[0].unanswered_deaths,0);
    assert_eq!(w.read_resource::<MobaMatch>().heroes[1].unanswered_deaths,8);
    driver.step(&mut w,[]).unwrap();
    assert!(w.read_resource::<MobaMatch>().heroes[1].entity.is_none());
    assert_eq!(w.read_resource::<MobaMatch>().heroes[1].unanswered_deaths,8);
    assert_eq!(w.read_storage::<Gold>().get(caster).unwrap().0,350);
    w.write_resource::<GamePause>().is_paused=true;
    for _ in 0..60 {driver.step(&mut w,[]).unwrap();}
    assert_eq!(w.read_resource::<MobaMatch>().heroes[1].unanswered_deaths,8);
    assert_eq!(w.read_storage::<Gold>().get(caster).unwrap().0,350);
}

#[test]
fn first_kill_formal_60hz_combines_shutdown_once_and_freezes_public_notice_during_pause() {
    let (mut w,mut driver)=world(SingleLaneConfig {first_kill_bonus_gold:100,kill_bounty_rules:KillBountyRules::training(),
        wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
    let tuple=|result:&SimulationTickResult|[FIRST_KILL_KILLER_METRIC_ID,FIRST_KILL_VICTIM_METRIC_ID,FIRST_KILL_GOLD_METRIC_ID].map(|id|{
        let values:Vec<_>=result.facts.iter().filter_map(|ordered|match ordered.fact{ObservableFact::Hud {team:0,metric_id,value}if metric_id==id=>{
            assert_eq!(ordered.audience,FactAudience::AllPlayers);Some(value)},_=>None}).collect();
        assert_eq!(values.len(),1,"one authority notice tuple per tick including pause");values[0]
    });
    let baseline=driver.step(&mut w,[]).unwrap();assert_eq!(tuple(&baseline),[0;3]);let [caster,enemy]=hero_pair(&w);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
    {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(enemy).unwrap();p.hp=Fixed64::from_i32(40);p.def_magic=Fixed64::ZERO;}
    w.write_resource::<MobaMatch>().heroes[1].kill_streak=3;
    let killed=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:0,target_entity:Some(enemy.id()),target_pos:None,
    }))})]).unwrap();assert_eq!(tuple(&killed),[1,2,100]);
    assert_eq!(w.read_storage::<Gold>().get(caster).unwrap().0,550);
    let retired=driver.step(&mut w,[]).unwrap();assert_eq!(tuple(&retired),[1,2,100]);
    assert!(w.read_resource::<MobaMatch>().heroes[1].entity.is_none());
    assert_eq!(w.read_storage::<Gold>().get(caster).unwrap().0,550);
    w.write_resource::<GamePause>().is_paused=true;let time=w.read_resource::<MobaMatch>().elapsed;
    for _ in 0..360{let paused=driver.step(&mut w,[]).unwrap();assert_eq!(tuple(&paused),[1,2,100]);}
    assert_eq!(w.read_resource::<MobaMatch>().elapsed,time);w.write_resource::<GamePause>().is_paused=false;
    let mut last=retired;for _ in 0..310{last=driver.step(&mut w,[]).unwrap();}
    assert_eq!(tuple(&last),[0;3]);assert_eq!(w.read_storage::<Gold>().get(caster).unwrap().0,550);
}

#[test]
fn shutdown_gold_formal_60hz_generated_skill_retires_public_bounty_once() {
    let (mut w,mut driver)=world(SingleLaneConfig {kill_bounty_rules:KillBountyRules::training(),
        wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
    {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(enemy).unwrap();p.hp=Fixed64::from_i32(40);p.def_magic=Fixed64::ZERO;}
    w.write_resource::<MobaMatch>().heroes[1].kill_streak=3;
    assert_eq!(w.read_resource::<MobaMatch>().shutdown_bounty(2),150);
    let result=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:0,target_entity:Some(enemy.id()),target_pos:None,
    }))})]).unwrap();
    assert_eq!(w.read_storage::<Gold>().get(caster).unwrap().0,450);
    assert!(w.read_storage::<CProperty>().get(enemy).unwrap().hp<=Fixed64::ZERO);
    assert_eq!(w.read_resource::<MobaMatch>().heroes[0].kill_streak,1);
    assert_eq!(w.read_resource::<MobaMatch>().heroes[1].kill_streak,0);
    for player in [1,2] {
        let fact=result.facts.iter().find(|ordered|matches!(ordered.fact,ObservableFact::Hud {team:0,metric_id,value:0}
            if metric_id==single_lane_player_metric(SINGLE_LANE_SCORE_BOUNTY_METRIC_ID,player))).expect("actual public PostStep bounty cleared");
        assert_eq!(fact.audience,FactAudience::AllPlayers);
    }
    driver.step(&mut w,[]).unwrap();assert_eq!(w.read_storage::<Gold>().get(caster).unwrap().0,450);
    assert!(w.read_resource::<MobaMatch>().heroes[1].entity.is_none(),"deferred Death retires the life on the next production step");
}

#[test]
fn cast_progress_formal_60hz_publishes_preparing_cancel_release_and_dead_owner_clear() {
    let (mut w,mut driver)=world(SingleLaneConfig {heroes:["saika_magoichi".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(4000),Fixed64::ZERO);
    let cast=||[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {ability_index:1,
        target_entity:None,target_pos:Some(Vec2I {x:origin.x.raw() as i32,y:origin.y.raw() as i32})}))})];
    let tuple=|result:&SimulationTickResult| {
        [CAST_SLOT,CAST_STATE,CAST_REMAINING,CAST_TOTAL,CAST_INSTANCE].map(|namespace|
            result.facts.iter().find_map(|ordered|match &ordered.fact {
                ObservableFact::Hud {team,metric_id,value} if *team==1 && *metric_id==single_lane_player_metric(namespace,1)=>Some(*value),
                _=>None,
            }).expect("actual PostStep team HUD cast metric"))
    };
    let started=driver.step(&mut w,cast()).unwrap();assert_eq!(tuple(&started),[2,1,512,512,1]);
    let progress=driver.step(&mut w,[]).unwrap();let progress=tuple(&progress);
    assert!(progress[2]>0 && progress[2]<512);
    let stopped=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::Stop(Default::default()))})]).unwrap();
    assert_eq!(tuple(&stopped),[2,2,0,512,1]);
    let restarted=driver.step(&mut w,cast()).unwrap();assert_eq!(tuple(&restarted),[2,1,512,512,2]);
    let mut result=restarted;for _ in 0..30 {result=driver.step(&mut w,[]).unwrap();}
    assert_eq!(tuple(&result),[2,3,0,512,2]);
    w.write_resource::<Vec<Outcome>>().push(lethal(enemy,caster));
    let dead=driver.step(&mut w,[]).unwrap();assert_eq!(tuple(&dead),[0;5]);
}

#[test]
fn cast_windup_formal_60hz_defers_effects_and_cancels_without_resource_debit() {
    for mode in 0..5 {
        let (mut w,mut driver)=world(SingleLaneConfig {
            heroes:["saika_magoichi".into(),"training_luminary".into()],
            wave_interval:Fixed64::from_i32(10_000),..fast_config()
        },SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
        let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
        w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
        w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(4000),Fixed64::ZERO);
        w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool=Some(omoba_core::runtime::ability_runtime::ManaPool::new(
            Fixed64::from_i32(200),Fixed64::from_i32(200)).unwrap());
        let mana=w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current();
        driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index:1,target_entity:None,target_pos:Some(Vec2I {x:origin.x.raw() as i32,y:origin.y.raw() as i32}),
        }))})]).unwrap();
        assert_eq!(w.read_storage::<SummonedUnit>().join().count(),0);
        assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current(),mana);
        assert!(!w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("saika_reinforcements"));
        if mode==1 {
            w.write_resource::<BuffStore>().add(caster,"silence",Fixed64::from_i32(10),serde_json::json!({}));
        } else if mode==2 {
            driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::Stop(Default::default()))})]).unwrap();
        } else if mode==3 {
            w.write_storage::<Pos>().get_mut(caster).unwrap().0.x+=Fixed64::ONE;
        } else if mode==4 {
            w.write_resource::<GamePause>().is_paused=true;
            for _ in 0..40 {driver.step(&mut w,[]).unwrap();}
            assert_eq!(w.read_storage::<SummonedUnit>().join().count(),0);
            w.write_resource::<GamePause>().is_paused=false;
        }
        for _ in 0..29 {driver.step(&mut w,[]).unwrap();}
        assert_eq!(w.read_storage::<SummonedUnit>().join().count(),0,"mode {mode}: no early release");
        driver.step(&mut w,[]).unwrap();
        let success=mode==0 || mode==4;
        assert_eq!(w.read_storage::<SummonedUnit>().join().count(),if success {2}else{0},"mode {mode}");
        let hero=w.read_storage::<Hero>();let hero=hero.get(caster).unwrap();
        assert_eq!(hero.is_on_cooldown("saika_reinforcements"),success);
        if !success {assert_eq!(hero.mana_pool.as_ref().unwrap().current(),mana);}
    }
}

#[test]
fn summon_formal_60hz_skill_creates_owned_combatants_then_expires_without_kill_reward() {
    let (mut w,mut driver)=world(SingleLaneConfig {
        heroes:["saika_magoichi".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..fast_config()
    },SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO);
    w.write_storage::<Facing>().get_mut(caster).unwrap().0=omoba_sim::Angle::ZERO;
    driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:1,target_entity:None,target_pos:Some(Vec2I {x:origin.x.raw() as i32,y:origin.y.raw() as i32}),
    }))})]).unwrap();
    for _ in 0..30 {driver.step(&mut w,[]).unwrap();}
    let summons:Vec<_>=(&w.entities(),&w.read_storage::<SummonedUnit>()).join().map(|(e,_)|e).collect();
    assert_eq!(summons.len(),2,"actual generated handler must create both rank-one summons");
    for entity in &summons {
        assert_eq!(w.read_storage::<PlayerOwner>().get(*entity).unwrap().player_id,1);
        assert_eq!(w.read_storage::<Faction>().get(*entity).unwrap().team_id,1);
        assert_eq!(w.read_storage::<SummonedUnit>().get(*entity).unwrap().summoner,caster);
        assert_eq!(w.read_storage::<CProperty>().get(*entity).unwrap().hp,Fixed64::from_i32(400));
        assert_eq!(w.read_storage::<TAttack>().get(*entity).unwrap().atk_physic.v,Fixed64::from_i32(45));
        let vision=w.read_storage::<VisionSource>();let sight=vision.get(*entity).expect("authored summon supplies sight");
        assert_eq!((sight.team,sight.radius),(1,Fixed64::from_i32(800)));
    }
    // Projectile creation proves the summons' real script hooks attack a hero,
    // rather than merely accepting a manually assembled ownership fixture.
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    driver.step(&mut w,[]).unwrap();
    assert!((&w.entities(),&w.read_storage::<Projectile>()).join().any(|(_,p)|summons.contains(&p.owner)),
        "summon scripts must produce real combat projectiles");
    let before=w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    for _ in 0..30 {driver.step(&mut w,[]).unwrap();}
    assert!(w.read_storage::<CProperty>().get(enemy).unwrap().hp<before);
    let remaining=w.read_storage::<SummonedUnit>().get(summons[0]).unwrap().time_remaining;
    w.write_resource::<GamePause>().is_paused=true;
    driver.step(&mut w,[]).unwrap();
    assert_eq!(w.read_storage::<SummonedUnit>().get(summons[0]).unwrap().time_remaining,remaining);
    w.write_resource::<GamePause>().is_paused=false;
    let enemy_gold=w.read_storage::<Gold>().get(enemy).unwrap().0;
    for entity in &summons {w.write_storage::<SummonedUnit>().get_mut(*entity).unwrap().time_remaining=Some(Fixed64::from_raw(1));}
    driver.step(&mut w,[]).unwrap();
    assert!(summons.iter().all(|e|!w.entities().is_alive(*e)));
    assert_eq!(w.read_storage::<Gold>().get(enemy).unwrap().0,enemy_gold);
}

#[test]
fn summon_navigation_formal_60hz_places_outside_wall_and_returns_to_moving_owner() {
    let run=|| {
        let (mut w,mut driver)=world(SingleLaneConfig {
            heroes:["saika_magoichi".into(),"training_luminary".into()],
            wave_interval:Fixed64::from_i32(10_000),..fast_config()
        },SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
        let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
        w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
        w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(4000),Fixed64::ZERO);
        w.write_storage::<Facing>().get_mut(caster).unwrap().0=omoba_sim::Angle::ZERO;
        w.write_resource::<BlockedRegions>().0.push(BlockedRegion {name:"actual-summon-formation-wall".into(),
            points:vec![vek::Vec2::new(100.0,6960.0),vek::Vec2::new(200.0,6960.0),
                vek::Vec2::new(200.0,7040.0),vek::Vec2::new(100.0,7040.0)]});
        driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index:1,target_entity:None,target_pos:Some(Vec2I {x:origin.x.raw() as i32,y:origin.y.raw() as i32}),
        }))})]).unwrap();
        for _ in 0..30 {driver.step(&mut w,[]).unwrap();}
    let summons:Vec<_>=(&w.entities(),&w.read_storage::<SummonedUnit>()).join().map(|(e,_)|e).collect();
        assert_eq!(summons.len(),2,"real formation must resolve wall-embedded requested positions");
        let legal=|w:&World,entity| {
            let pos=w.read_storage::<Pos>().get(entity).unwrap().0;
            let radius=w.read_storage::<CollisionRadius>().get(entity).unwrap().0;
            let terrain=&w.read_resource::<BlockedRegions>();
            for region in &terrain.0 {
                let polygon:Vec<_>=region.points.iter().map(|p|omoba_sim::Vec2::new(
                    Fixed64::from_raw((f64::from(p.x)*1024.0).round() as i64),
                    Fixed64::from_raw((f64::from(p.y)*1024.0).round() as i64))).collect();
                assert!(!omoba_sim::terrain::swept_circle_hits_polygon(pos,pos,radius,&polygon));
            }
            pos
        };
        for entity in &summons {
            legal(&w,*entity);
            let data=w.read_storage::<SummonedUnit>();let nav=&data.get(*entity).unwrap().navigation;
            assert_eq!((nav.follow_start_distance,nav.follow_stop_distance),(Fixed64::from_i32(750),Fixed64::from_i32(350)));
        }
        let destination=origin+omoba_sim::Vec2::new(Fixed64::from_i32(1000),Fixed64::ZERO);
        w.write_storage::<Pos>().get_mut(caster).unwrap().0=destination;
        run_committed_visibility_wave_b(&mut w,driver.tick(),0);
        let mut trace=Vec::new();
        for _ in 0..240 {
            driver.step(&mut w,[]).unwrap();
            trace.push(summons.iter().map(|e|legal(&w,*e)).collect::<Vec<_>>());
        }
        for entity in &summons {
            assert!((w.read_storage::<Pos>().get(*entity).unwrap().0-destination).length()<=Fixed64::from_i32(351));
            assert!(w.read_storage::<MoveTarget>().get(*entity).is_none());
        }
        (trace,single_lane_replay_digest(&w))
    };
    assert_eq!(run(),run(),"generated skill and host navigation must replay identically");
}

#[test]
fn summon_projectile_formal_60hz_kills_after_gunner_expiry_and_credits_caster() {
    let (mut w,mut driver)=world(SingleLaneConfig {
        heroes:["saika_magoichi".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..fast_config()
    },SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO);
    w.write_storage::<Facing>().get_mut(caster).unwrap().0=omoba_sim::Angle::ZERO;
    driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:1,target_entity:None,target_pos:Some(Vec2I {x:origin.x.raw() as i32,y:origin.y.raw() as i32}),
    }))})]).unwrap();
    for _ in 0..30 {driver.step(&mut w,[]).unwrap();}
    let summons:Vec<_>=(&w.entities(),&w.read_storage::<SummonedUnit>()).join().map(|(e,_)|e).collect();
    assert_eq!(summons.len(),2);
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    driver.step(&mut w,[]).unwrap();
    let shots:Vec<_>=(&w.entities(),&w.read_storage::<Projectile>()).join()
        .filter(|(_,p)|summons.contains(&p.owner)).map(|(e,_)|e).collect();
    assert_eq!(shots.len(),2);
    for shot in &shots {assert_eq!(w.read_resource::<ProjectileOrigins>().0[shot].credit_player,Some(1));}
    w.write_storage::<CProperty>().get_mut(enemy).unwrap().hp=Fixed64::ONE;
    for summon in &summons {w.write_storage::<SummonedUnit>().get_mut(*summon).unwrap().time_remaining=Some(Fixed64::from_raw(1));}
    driver.step(&mut w,[]).unwrap();assert!(summons.iter().all(|e|!w.entities().is_alive(*e)));
    let before=w.read_storage::<Gold>().get(caster).unwrap().0;
    for _ in 0..40 {driver.step(&mut w,[]).unwrap();}
    let state=w.read_resource::<MobaMatch>();
    assert_eq!(state.heroes[0].kills,1,"real expired-summon projectile must credit its player");
    assert_eq!(w.read_storage::<Gold>().get(caster).unwrap().0,before+state.config.hero_kill_gold as i32);
    assert!(shots.iter().all(|e|!w.read_resource::<ProjectileOrigins>().0.contains_key(e)));
}

#[test]
fn summon_combat_formal_60hz_stun_root_silence_and_slow() {
    let (mut w,mut driver)=world(SingleLaneConfig {
        heroes:["saika_magoichi".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..fast_config()
    },SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(7000));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(350),Fixed64::ZERO);
    w.write_storage::<Facing>().get_mut(caster).unwrap().0=omoba_sim::Angle::ZERO;
    driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:1,target_entity:None,target_pos:Some(Vec2I {x:origin.x.raw() as i32,y:origin.y.raw() as i32}),
    }))})]).unwrap();
    for _ in 0..30 {driver.step(&mut w,[]).unwrap();}
    let summons:Vec<_>=(&w.entities(),&w.read_storage::<SummonedUnit>()).join().map(|(e,_)|e).collect();
    assert_eq!(summons.len(),2);
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    driver.step(&mut w,[]).unwrap();
    for control in ["stun","root","silence"] {
        let old:Vec<_>=(&w.entities(),&w.read_storage::<Projectile>()).join().map(|(e,_)|e).collect();
        w.delete_entities(&old).unwrap();w.maintain();
        for entity in &summons {
            let mut attacks=w.write_storage::<TAttack>();let attack=attacks.get_mut(*entity).unwrap();attack.asd_count=attack.asd.v;
            w.write_resource::<BuffStore>().add(*entity,control,Fixed64::from_i32(10),serde_json::json!({}));
        }
        driver.step(&mut w,[]).unwrap();
        let shots=(&w.entities(),&w.read_storage::<Projectile>()).join().filter(|(_,p)|summons.contains(&p.owner)).count();
        assert_eq!(shots,if control=="stun" {0}else{2},"{control} has distinct ordinary-attack semantics");
        for entity in &summons {w.write_resource::<BuffStore>().remove(*entity,control);}
    }
    let summon=summons[0];let start=w.read_storage::<Pos>().get(summon).unwrap().0;
    w.write_storage::<MoveTarget>().insert(summon,MoveTarget(start+omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(100)))).unwrap();
    w.write_resource::<BuffStore>().add(summon,"root",Fixed64::from_i32(10),serde_json::json!({}));
    driver.step(&mut w,[]).unwrap();assert_eq!(w.read_storage::<Pos>().get(summon).unwrap().0,start);
    w.write_resource::<BuffStore>().remove(summon,"root");
    w.write_resource::<BuffStore>().add(summon,"slow_test",Fixed64::from_i32(10),serde_json::json!({"move_speed_bonus":-512}));
    driver.step(&mut w,[]).unwrap();
    let distance=(w.read_storage::<Pos>().get(summon).unwrap().0-start).length();
    let base_step=Fixed64::from_i32(320)*w.read_resource::<DeltaTime>().0;
    assert!(distance>Fixed64::ZERO && distance<base_step,"movement uses the same slowed stat as AI: {distance:?}/{base_step:?}");
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
        let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
        w.write_storage::<Pos>().get_mut(hero).unwrap().0=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
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
fn ground_zone_formal_60hz_earlier_source_death_retires_later_pulse_without_ghost_kill() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::runtime::ability_runtime::damage_zone::DamageZoneSpec;
    use omb_script_abi::types::{DamageKind,DamageProfile};
    for survives in [false,true] {
        let player=|player_id,team_id,role|RoleBotPlayerPlan {player_id,team_id,role,
            hero:"training_luminary".into(),lane:"mid".into(),bot:false};
        let plan=RoleBotMatchPlan {schema_version:1,map_id:"three_lane_training".into(),think_hz:60,mana_enabled:false,
            players:vec![player(1,1,BotRole::Mid),player(2,2,BotRole::Mid),player(3,1,BotRole::Top)],
            ability_learning:Vec::new(),ability_policies:Vec::new(),sustain:None,item_builds:Vec::new(),summoner_loadouts:std::collections::BTreeMap::new()};
        let (mut config,_)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
        config.warmup=Fixed64::ZERO;config.wave_interval=Fixed64::from_i32(10_000);
        let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);driver.step(&mut w,[]).unwrap();
        let entity=|w:&World,id|w.read_resource::<MobaMatch>().heroes.iter().find(|s|s.player_id==id).unwrap().entity.unwrap();
        let protected=entity(&w,1);let later=entity(&w,2);let earlier=entity(&w,3);assert!(earlier.id()>later.id());
        let y=isolated_skill_origin_y();for (e,x) in [(earlier,0),(later,400),(protected,1000)] {
            place_hero(&mut w,e,x,y);disarm_attack(&mut w,e);set_hold(&mut w,e);
        }
        let make=|x|DamageZoneSpec {center:sim_point(x,y),radius:Fixed64::from_i32(100),amount:Fixed64::from_i32(40),
            interval:Fixed64::from_raw(512),kind:DamageKind::Magical,profile:DamageProfile::FIRE.bits()};
        let admit=|w:&mut World,source,key:&str,spec:DamageZoneSpec| {
            w.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff {target:source,buff_id:key.into(),duration:Fixed64::from_i32(3),payload:spec.payload()});
            process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();
        };
        // Explicit native-clock fixture setup: neither zone has pulsed yet.
        admit(&mut w,earlier,"retire-earlier",make(400));
        {let dt=Fixed64::from_raw(4);let mut buffs=w.write_resource::<BuffStore>();assert!(buffs.drain_damage_zones(dt).is_empty());buffs.tick(dt);}
        admit(&mut w,later,"retire-later",make(1000));
        {let dt=Fixed64::from_raw(496);let mut buffs=w.write_resource::<BuffStore>();assert!(buffs.drain_damage_zones(dt).is_empty());buffs.tick(dt);}
        {let mut hp=w.write_storage::<CProperty>();for (e,value) in [(later,if survives {1000}else{10}),(protected,10)] {
            let p=hp.get_mut(e).unwrap();p.hp=Fixed64::from_i32(value);p.mhp=Fixed64::from_i32(1000);p.def_magic=Fixed64::ZERO;}}
        driver.step(&mut w,[1,2,3].into_iter().map(|id|(id,PlayerInput {action:Some(PlayerInputEnum::HoldPosition(Default::default()))}))).unwrap();
        let state=w.read_resource::<MobaMatch>();let score=|id|{let h=state.heroes.iter().find(|h|h.player_id==id).unwrap();(h.kills,h.deaths)};
        if survives {
            assert_eq!(score(1),(0,1));assert_eq!(score(2),(1,0));assert_eq!(score(3),(0,0));
            assert!(w.read_resource::<BuffStore>().has(later,"retire-later"),"living late source remains active");
        } else {
            assert_eq!(score(1),(0,0));assert_eq!(score(2),(0,1));assert_eq!(score(3),(1,0));
            assert!(w.read_storage::<CProperty>().get(protected).unwrap().hp>=Fixed64::from_i32(10),"no posthumous zone hit");
            assert!(!w.read_resource::<BuffStore>().has(later,"retire-later"));
        }
    }
}

#[test]
fn ground_zone_formal_60hz_first_lethal_uses_due_time_then_canonical_source() {
    use omoba_core::runtime::native::moba_match::bots::*;
    use omoba_core::runtime::ability_runtime::damage_zone::DamageZoneSpec;
    use omb_script_abi::types::{DamageKind,DamageProfile};
    for simultaneous in [false,true] {
        let player=|player_id,team_id,role|RoleBotPlayerPlan {player_id,team_id,role,
            hero:"training_luminary".into(),lane:"mid".into(),bot:false};
        let plan=RoleBotMatchPlan {schema_version:1,map_id:"three_lane_training".into(),think_hz:60,mana_enabled:false,
            players:vec![player(1,1,BotRole::Mid),player(2,1,BotRole::Top),player(3,2,BotRole::Mid)],
            ability_learning:Vec::new(),ability_policies:Vec::new(),sustain:None,item_builds:Vec::new(),summoner_loadouts:std::collections::BTreeMap::new()};
        let (mut config,_)=plan.compile(42,SimulationTickProfile::Production60Hz).unwrap();
        config.warmup=Fixed64::ZERO;config.wave_interval=Fixed64::from_i32(10_000);
        let (mut w,mut driver)=world(config,SimulationTickProfile::Production60Hz);driver.step(&mut w,[]).unwrap();
        let entity=|w:&World,id|w.read_resource::<MobaMatch>().heroes.iter().find(|slot|slot.player_id==id).unwrap().entity.unwrap();
        let low=entity(&w,1);let high=entity(&w,2);let victim=entity(&w,3);assert!(low.id()<high.id());
        let y=isolated_skill_origin_y();place_hero(&mut w,low,0,y);place_hero(&mut w,high,0,y);
        place_hero(&mut w,victim,2000,y);for e in [low,high,victim] {disarm_attack(&mut w,e);set_hold(&mut w,e);}
        let center=sim_point(400,y);let make=|interval|DamageZoneSpec {center,radius:Fixed64::from_i32(100),
            amount:Fixed64::from_i32(40),interval:Fixed64::from_raw(interval),kind:DamageKind::Magical,profile:DamageProfile::FIRE.bits()};
        let admit=|w:&mut World,source,key:&str,spec:DamageZoneSpec| {
            w.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff {target:source,buff_id:key.into(),duration:Fixed64::from_i32(3),payload:spec.payload()});
            process_outcomes(w,&mut RuntimeEventVecSink::default()).unwrap();
        };
        // Prime native clocks while the victim is outside the area. The tested
        // first-lethal settlement below runs through the production 60Hz driver.
        admit(&mut w,low,"order-low",make(256));
        {let dt=Fixed64::from_raw(if simultaneous {256}else{252});let mut buffs=w.write_resource::<BuffStore>();
            buffs.drain_damage_zones(dt);buffs.tick(dt);}
        admit(&mut w,high,"order-high",make(512));
        {let dt=Fixed64::from_raw(if simultaneous {496}else{500});let mut buffs=w.write_resource::<BuffStore>();
            buffs.drain_damage_zones(dt);buffs.tick(dt);}
        place_hero(&mut w,victim,400,y);w.write_storage::<CProperty>().get_mut(victim).unwrap().hp=Fixed64::from_i32(10);
        let remaining=w.read_resource::<BuffStore>().get(low,"order-low").unwrap().remaining;
        w.write_resource::<GamePause>().is_paused=true;driver.step(&mut w,[]).unwrap();
        assert_eq!(w.read_resource::<BuffStore>().get(low,"order-low").unwrap().remaining,remaining);
        assert_eq!(w.read_storage::<CProperty>().get(victim).unwrap().hp,Fixed64::from_i32(10));
        w.write_resource::<GamePause>().is_paused=false;
        driver.step(&mut w,[1,2,3].into_iter().map(|id|(id,PlayerInput {action:Some(PlayerInputEnum::HoldPosition(Default::default()))}))).unwrap();
        let state=w.read_resource::<MobaMatch>();let kills=|id|state.heroes.iter().find(|h|h.player_id==id).unwrap().kills;
        assert_eq!((kills(1),kills(2)),if simultaneous {(1,0)}else{(0,1)},"tie={simultaneous}:12raw high precedes16raw low; equal offsets use low ID");
        assert_eq!(state.heroes.iter().find(|h|h.player_id==3).unwrap().deaths,1,"same-tick overkill pays only once");
    }
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
    use omoba_core::runtime::shop::{transact_moba_shop, undo_moba_shop, ShopCommand, ShopUndoRequest};
    let (mut world, mut driver) = world(SingleLaneConfig {
        wave_interval: Fixed64::from_i32(10_000),
        hero_kill_gold: 0,
        hero_assist_gold: 0,
        ..fast_config()
    }, SimulationTickProfile::Coarse15Hz);
    world.insert(ItemRegistry::from_configs(vec![ItemConfig {id: "moba_armor".into(), name: "Armor".into(),
        cost: 100, bonus: ItemBonus {hp: 100.0, armor: 5.0, ..ItemBonus::default()}, active: None,
        cooldown: 0.0, recipe: vec![]} ]));
    driver.step(&mut world, []).unwrap();
    let old = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    world.write_storage::<Gold>().get_mut(old).unwrap().0 = 200;
    world.write_storage::<Hero>().get_mut(old).unwrap().experience = 23;
    let intact = world.read_storage::<CProperty>().get(old).unwrap().clone();
    let damaged = intact.hp - Fixed64::from_i32(80);
    world.write_storage::<CProperty>().get_mut(old).unwrap().hp = damaged;
    transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())).unwrap();
    driver.step(&mut world, []).unwrap();
    let bought = world.read_storage::<CProperty>().get(old).unwrap().clone();
    assert_eq!(bought.mhp, intact.mhp + Fixed64::from_i32(100));
    assert_eq!(bought.def_physic, intact.def_physic + Fixed64::from_i32(5));
    assert!(bought.hp - damaged < Fixed64::from_i32(100));
    assert!(bought.mhp - bought.hp > Fixed64::from_i32(40));
    assert_eq!(world.read_storage::<Hero>().get(old).unwrap().experience, 23);
    assert_eq!(world.read_storage::<ItemEffects>().get(old).unwrap().applied_hp, 100.0);
    undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest).unwrap();
    driver.step(&mut world, []).unwrap();
    let undone = world.read_storage::<CProperty>().get(old).unwrap().clone();
    assert_eq!(undone.mhp, intact.mhp);
    assert_eq!(undone.def_physic, intact.def_physic);
    assert!(undone.hp - damaged < Fixed64::from_i32(100));
    assert!(undone.mhp - undone.hp > Fixed64::from_i32(40));
    assert_eq!(world.read_storage::<Hero>().get(old).unwrap().experience, 23);
    assert_eq!(world.read_storage::<ItemEffects>().get(old).unwrap().applied_hp, 0.0);
    assert!(world.read_storage::<Inventory>().get(old).unwrap().find_item("moba_armor").is_none());
    transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())).unwrap();
    driver.step(&mut world, []).unwrap();
    let rebought = world.read_storage::<CProperty>().get(old).unwrap().clone();
    assert_eq!(rebought.mhp, intact.mhp + Fixed64::from_i32(100));
    assert!(rebought.hp - damaged < Fixed64::from_i32(100));
    assert!(rebought.mhp - rebought.hp > Fixed64::from_i32(40));
    assert_eq!(world.read_storage::<Hero>().get(old).unwrap().experience, 23);
    let pos = world.read_storage::<Pos>().get(old).unwrap().0;
    world.write_resource::<Vec<Outcome>>().push(Outcome::Death { pos, ent: old });
    for _ in 0..40 { driver.step(&mut world, []).unwrap(); }
    let new = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    assert_ne!(canonical_entity_id(new), canonical_entity_id(old));
    assert!(!world.entities().is_alive(old));
    assert_eq!(world.read_storage::<Gold>().get(new).unwrap().0, 100);
    assert_eq!(world.read_storage::<Inventory>().get(new).unwrap().find_item("moba_armor"), Some(0));
    let born = world.read_storage::<CProperty>().get(new).unwrap().clone();
    assert_eq!(born.mhp, rebought.mhp);
    assert_eq!(born.def_physic, rebought.def_physic);
    assert_eq!(born.hp, born.mhp, "respawned loadout starts at its finalized maximum");
    assert_eq!(world.read_storage::<Hero>().get(new).unwrap().experience, 23);
    let born_effects = *world.read_storage::<ItemEffects>().get(new).unwrap();
    assert_eq!((born_effects.applied_hp, born_effects.applied_armor, born_effects.dirty), (100.0, 5.0, false));
    driver.step(&mut world, []).unwrap();
    let second = world.read_storage::<CProperty>().get(new).unwrap().clone();
    let second_effects = *world.read_storage::<ItemEffects>().get(new).unwrap();
    assert_eq!(second.mhp, born.mhp);
    assert_eq!(second.def_physic, born.def_physic);
    assert_eq!((second_effects.applied_hp, second_effects.applied_armor), (100.0, 5.0));
    assert!(second.hp - born.hp < Fixed64::from_i32(100));
}

#[test]
fn single_lane_dead_roster_shop_persists_through_respawn_without_reviving() {
    use omoba_core::runtime::native::economy_projection::{
        SHOP_REASON_AVAILABLE, SHOP_REASON_DEAD_UNCAPTURED, SHOP_REASON_FOREIGN_OWNER,
        SHOP_REASON_MISSING_BASE, SHOP_REASON_NOT_PLAYING, SHOP_REASON_OUTSIDE_SHOP,
        SHOP_REASON_PAUSED, SHOP_REASON_RETIRED, SHOP_REASON_STALE_IDENTITY, SHOP_REASON_UNKNOWN_PLAYER,
    };
    use omoba_core::runtime::shop::{
        owner_shop_availability, transact_moba_shop, undo_moba_shop, ShopCommand, ShopError,
        ShopUndoLedger, ShopUndoRequest, SHOP_UNDO_ROSTER_GENERATION, SHOP_UNDO_ROSTER_INDEX,
    };
    use omoba_core::runtime::shop_receipt::{project_shop_receipts, ShopReceipt};
    use omoba_core::runtime::shop_transport::validate_receipt_replay;
    use prost::Message;
    let config = SingleLaneConfig {
        wave_interval: Fixed64::from_i32(10_000),
        hero_kill_gold: 0,
        hero_assist_gold: 0,
        ..fast_config()
    };
    let catalog = |id: &str| -> u16 {
        omoba_template_ids::MOBA_ITEM_CATALOG.iter().find(|item| item.id == id).unwrap().catalog_id
    };
    let owner_fact = |facts: &[omoba_core::runtime::OrderedFact], player: u32| {
        facts.iter().rev().find_map(|fact| match &fact.fact {
            ObservableFact::OwnerEconomy { state, .. } if state.player_id == player => Some(state.clone()),
            _ => None,
        }).unwrap()
    };
    let availability = |world: &World, player: u32| {
        let state = world.read_resource::<MobaMatch>().clone();
        owner_shop_availability(world, &state, player)
    };
    let no_living_owner = |world: &World, player: u32| {
        !(&world.entities(), &world.read_storage::<PlayerOwner>(), &world.read_storage::<CProperty>()).join()
            .any(|(_, owner, property)| owner.player_id == player && property.hp > Fixed64::ZERO)
    };
    let outside_shop_position = |world: &World, player: u32| {
        let state = world.read_resource::<MobaMatch>();
        let side = state.heroes.iter().find(|slot| slot.player_id == player).unwrap().side;
        let base = state.bases[side].unwrap();
        let origin = world.read_storage::<Pos>().get(base).unwrap().0;
        origin + omoba_sim::Vec2::new(Fixed64::ZERO,
            Fixed64::from_i32(omoba_core::runtime::shop::MOBA_SHOP_RADIUS + 1))
    };
    let registry = || ItemRegistry::from_configs(vec![
        ItemConfig { id: "moba_sword".into(), name: "Sword".into(), cost: 100,
            bonus: ItemBonus { atk: 10.0, ..ItemBonus::default() }, active: None, cooldown: 0.0, recipe: vec![] },
        ItemConfig { id: "moba_armor".into(), name: "Armor".into(), cost: 100,
            bonus: ItemBonus { hp: 100.0, armor: 5.0, ..ItemBonus::default() }, active: None, cooldown: 0.0, recipe: vec![] },
    ]);
    let buy_input = |id: &str| PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: id.into() })) };
    let mut dead_digests = Vec::new();
    let mut respawn_digests = Vec::new();
    for _ in 0..2 {
        let (mut world, mut driver) = world(config.clone(), SimulationTickProfile::Coarse15Hz);
        world.insert(registry());
        driver.step(&mut world, []).unwrap();
        let heroes = hero_pair(&world);
        for hero in heroes { world.write_storage::<Gold>().get_mut(hero).unwrap().0 = 500; }
        let td_before = extract_snapshot(&mut world, 0, std::sync::Arc::new(Vec::new()), std::sync::Arc::new(Vec::new()),
            std::sync::Arc::new(Vec::new()), Vec::new(), Vec::new()).player_gold;
        transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_sword".into())).unwrap();
        let spawn = world.read_storage::<Pos>().get(heroes[0]).unwrap().0;
        let outside = outside_shop_position(&world, 1);
        world.write_storage::<Pos>().get_mut(heroes[0]).unwrap().0 = outside;
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Sell(0)), Err(ShopError::OutsideShop));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_OUTSIDE_SHOP));
        assert_eq!(world.read_storage::<Gold>().get(heroes[0]).unwrap().0, 400);
        world.write_storage::<Pos>().get_mut(heroes[0]).unwrap().0 = spawn;
        world.write_storage::<CProperty>().get_mut(heroes[0]).unwrap().hp = Fixed64::ZERO;
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())), Err(ShopError::HeroUnavailable));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_DEAD_UNCAPTURED));
        assert_eq!(world.read_storage::<CProperty>().get(heroes[0]).unwrap().hp, Fixed64::ZERO);
        world.write_storage::<CProperty>().get_mut(heroes[0]).unwrap().hp = Fixed64::from_i32(500);
        transact_moba_shop(&mut world, 2, ShopCommand::Buy("moba_armor".into())).unwrap();
        world.write_resource::<Vec<Outcome>>().push(Outcome::Death { pos: spawn, ent: heroes[0] });
        let died = driver.step(&mut world, [(1, buy_input("moba_armor"))]).unwrap();
        assert!(world.read_resource::<MobaMatch>().heroes[0].entity.is_none());
        assert!(!world.entities().is_alive(heroes[0]));
        assert!(no_living_owner(&world, 1));
        assert_eq!(world.read_resource::<PendingItemUseQueue>().settlements.len(), 1);
        assert!(world.read_resource::<PendingItemUseQueue>().settlements[0].result.is_ok());
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        let died_economy = owner_fact(&died.facts, 1);
        assert_eq!(died_economy.economy.gold, 300);
        assert_eq!(died_economy.economy.item_ids[0], catalog("moba_sword"));
        assert_eq!(died_economy.economy.item_ids[1], catalog("moba_armor"));
        assert!(died_economy.shop_available);
        assert_eq!(died_economy.shop_reason, SHOP_REASON_AVAILABLE);
        assert_eq!(world.read_storage::<Gold>().get(heroes[1]).unwrap().0, 400);
        {
            let prior = world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().records[0].clone();
            assert!(!prior.eligible);
            assert_eq!(prior.source_index, heroes[0].id());
            assert_ne!(prior.source_index, SHOP_UNDO_ROSTER_INDEX);
        }
        transact_moba_shop(&mut world, 1, ShopCommand::Sell(0)).unwrap();
        let sold = driver.step(&mut world, []).unwrap();
        let sold_economy = owner_fact(&sold.facts, 1);
        assert_eq!(sold_economy.economy.gold, 350);
        assert_eq!(sold_economy.economy.item_ids[0], 0);
        assert_eq!(sold_economy.economy.item_ids[1], catalog("moba_armor"));
        assert!(world.read_resource::<MobaMatch>().heroes[0].entity.is_none());
        let bought = driver.step(&mut world, [(1, buy_input("moba_sword"))]).unwrap();
        assert_eq!(world.read_resource::<PendingItemUseQueue>().settlements[0].result, Ok(()));
        let bought_economy = owner_fact(&bought.facts, 1);
        assert_eq!(bought_economy.economy.gold, 250);
        assert_eq!(bought_economy.economy.item_ids[0], catalog("moba_sword"));
        assert_eq!(bought_economy.economy.item_ids[1], catalog("moba_armor"));
        assert!(bought_economy.shop_available);
        assert_eq!(bought_economy.shop_reason, SHOP_REASON_AVAILABLE);
        assert!(no_living_owner(&world, 1));
        assert_eq!(world.read_storage::<Gold>().get(heroes[1]).unwrap().0, 400);
        let td_after = extract_snapshot(&mut world, driver.tick() as u32, std::sync::Arc::new(Vec::new()),
            std::sync::Arc::new(Vec::new()), std::sync::Arc::new(Vec::new()), Vec::new(), Vec::new()).player_gold;
        assert_eq!(td_after, td_before);
        let settlements = world.read_resource::<PendingItemUseQueue>().settlements.clone();
        let payload = buy_input("moba_sword").encode_to_vec();
        let accepted = CanonicalAcceptedInput::from_authoritative_acceptance(1, 1, 42, 17, 0, None, payload.clone());
        let receipts = project_shop_receipts(1, driver.tick(), &[accepted.clone(), accepted], &settlements).unwrap();
        assert_eq!(receipts.len(), 1);
        let receipt = ShopReceipt::decode(&receipts[0].sanitized_payload).unwrap();
        assert_eq!(receipt.result_code, 0);
        assert_eq!(receipt.catalog_id, u32::from(catalog("moba_sword")));
        let local = CanonicalAcceptedInput::from_authoritative_acceptance(1, 1, 0, 17, 0, None, payload);
        assert!(project_shop_receipts(1, driver.tick(), &[local], &settlements).unwrap().is_empty());
        let replay = omoba_core::game_proto::ShopReceiptReplay {
            schema_version: 1, request_id: 4, player_id: 1, input_id: 42, status: 2, receipt: receipt.encode(),
        };
        let before_replay = single_lane_replay_digest(&world);
        assert_eq!(validate_receipt_replay(&replay, 1), Ok(Some(receipt.clone())));
        assert_eq!(validate_receipt_replay(&replay, 1), Ok(Some(receipt)));
        assert_eq!(single_lane_replay_digest(&world), before_replay);
        let sequence = world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().records.last().unwrap().sequence;
        assert_eq!(world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().records.last().unwrap().source_index, SHOP_UNDO_ROSTER_INDEX);
        assert_eq!(world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().records.last().unwrap().source_generation, SHOP_UNDO_ROSTER_GENERATION);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(sequence)).unwrap(), sequence);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(sequence)), Err(ShopError::UndoDuplicate));
        assert!(no_living_owner(&world, 1));
        let refunded = driver.step(&mut world, []).unwrap();
        let refunded_economy = owner_fact(&refunded.facts, 1);
        assert_eq!(refunded_economy.economy.gold, 350);
        assert_eq!(refunded_economy.economy.item_ids[0], 0);
        assert_eq!(refunded_economy.economy.item_ids[1], catalog("moba_armor"));
        let again = driver.step(&mut world, [(1, buy_input("moba_sword"))]).unwrap();
        assert_eq!(owner_fact(&again.facts, 1).economy.gold, 250);
        let fresh = world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().records.last().unwrap().clone();
        assert!(fresh.eligible);
        assert_eq!(fresh.source_index, SHOP_UNDO_ROSTER_INDEX);
        assert!(world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().records.iter()
            .filter(|record| record.sequence != fresh.sequence && record.kind==omoba_core::runtime::shop::ShopUndoKind::Purchase).all(|record| !record.eligible));
        let base = world.read_resource::<MobaMatch>().bases[0].unwrap();
        let respawn_at = world.read_resource::<MobaMatch>().heroes[0].respawn_at;
        world.write_resource::<MobaMatch>().bases[0] = None;
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_sword".into())), Err(ShopError::MatchUnavailable));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_MISSING_BASE));
        world.write_resource::<MobaMatch>().bases[0] = Some(base);
        world.write_resource::<GamePause>().is_paused = true;
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Sell(1)), Err(ShopError::MatchUnavailable));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_PAUSED));
        world.write_resource::<GamePause>().is_paused = false;
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Finished { winner: Some(0), tick: driver.tick() };
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Sell(1)), Err(ShopError::MatchUnavailable));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_NOT_PLAYING));
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
        for player in [0, 99] {
            assert_eq!(transact_moba_shop(&mut world, player, ShopCommand::Buy("moba_sword".into())), Err(ShopError::UnknownPlayer));
            assert_eq!(availability(&world, player), (false, SHOP_REASON_UNKNOWN_PLAYER));
        }
        world.write_resource::<MobaMatch>().heroes[0].respawn_at = None;
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_sword".into())), Err(ShopError::HeroUnavailable));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_RETIRED));
        world.write_resource::<MobaMatch>().heroes[0].respawn_at = respawn_at;
        world.write_resource::<MobaMatch>().heroes[0].entity = Some(heroes[0]);
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_sword".into())), Err(ShopError::HeroUnavailable));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_STALE_IDENTITY));
        world.write_resource::<MobaMatch>().heroes[0].entity = None;
        world.write_resource::<MobaMatch>().heroes[0].entity = Some(heroes[1]);
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Sell(0)), Err(ShopError::UnknownPlayer));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_FOREIGN_OWNER));
        assert_eq!(world.read_storage::<Gold>().get(heroes[1]).unwrap().0, 400);
        world.write_resource::<MobaMatch>().heroes[0].entity = None;
        let away = world.read_storage::<Pos>().get(heroes[1]).unwrap().0;
        let outside = outside_shop_position(&world, 2);
        world.write_storage::<Pos>().get_mut(heroes[1]).unwrap().0 = outside;
        assert_eq!(transact_moba_shop(&mut world, 2, ShopCommand::Sell(0)), Err(ShopError::OutsideShop));
        assert_eq!(availability(&world, 2), (false, SHOP_REASON_OUTSIDE_SHOP));
        assert_eq!(world.read_storage::<Gold>().get(heroes[1]).unwrap().0, 400);
        world.write_storage::<Pos>().get_mut(heroes[1]).unwrap().0 = away;
        let settled = driver.step(&mut world, []).unwrap();
        assert!(world.read_resource::<MobaMatch>().heroes[0].entity.is_none());
        assert_eq!(owner_fact(&settled.facts, 1).economy.gold, 250);
        assert_eq!(owner_fact(&settled.facts, 1).shop_reason, SHOP_REASON_AVAILABLE);
        assert_eq!(world.read_storage::<Gold>().get(heroes[1]).unwrap().0, 400);
        dead_digests.push(single_lane_replay_digest(&world));
        let mut spawned = None;
        for _ in 0..40 {
            driver.step(&mut world, []).unwrap();
            if let Some(entity) = world.read_resource::<MobaMatch>().heroes[0].entity {
                spawned = Some(entity);
                break;
            }
        }
        let new = spawned.expect("replacement hero");
        assert_ne!(canonical_entity_id(new), canonical_entity_id(heroes[0]));
        assert!(!world.entities().is_alive(heroes[0]));
        assert_eq!(world.read_storage::<Gold>().get(new).unwrap().0, 250);
        assert_eq!(world.read_storage::<Inventory>().get(new).unwrap().find_item("moba_sword"), Some(0));
        assert_eq!(world.read_storage::<Inventory>().get(new).unwrap().find_item("moba_armor"), Some(1));
        assert_eq!(world.read_storage::<Gold>().get(heroes[1]).unwrap().0, 400);
        let born = world.read_storage::<CProperty>().get(new).unwrap().clone();
        assert_eq!(born.hp, born.mhp, "a new life starts at full health including retained/dead-purchased equipment");
        assert_eq!(born.mhp, world.read_storage::<CProperty>().get(heroes[1]).unwrap().mhp);
        let effects = *world.read_storage::<ItemEffects>().get(new).unwrap();
        assert_eq!((effects.applied_hp, effects.applied_armor, effects.applied_atk, effects.dirty), (100.0, 5.0, 10.0, false));
        let atk = world.read_storage::<TAttack>().get(new).unwrap().atk_physic.clone().val();
        assert_eq!(atk, world.read_storage::<TAttack>().get(heroes[1]).unwrap().atk_physic.clone().val() + Fixed64::from_i32(10));
        driver.step(&mut world, []).unwrap();
        let second = world.read_storage::<CProperty>().get(new).unwrap().clone();
        let second_effects = *world.read_storage::<ItemEffects>().get(new).unwrap();
        assert_eq!(second.mhp, born.mhp);
        assert!(second.hp - born.hp < Fixed64::from_i32(100));
        assert_eq!((second_effects.applied_hp, second_effects.applied_armor, second_effects.applied_atk), (100.0, 5.0, 10.0));
        let bound = world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().records.last().unwrap().clone();
        assert!(bound.eligible);
        assert_eq!(bound.source_index, new.id());
        assert_eq!(bound.source_generation, new.gen().id());
        assert!(world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().records.iter()
            .filter(|record| record.sequence != bound.sequence && record.kind==omoba_core::runtime::shop::ShopUndoKind::Purchase).all(|record| !record.eligible));
        respawn_digests.push(single_lane_replay_digest(&world));
        let base_pos = world.read_storage::<Pos>().get(base).unwrap().0;
        world.write_storage::<Pos>().get_mut(new).unwrap().0 = base_pos;
        world.write_resource::<ShopUndoLedger>().owners.get_mut(&1).unwrap().records.last_mut().unwrap().source_generation = 0;
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(world.read_storage::<Gold>().get(new).unwrap().0, 250);
        assert_eq!(world.read_storage::<Inventory>().get(new).unwrap().find_item("moba_sword"), Some(0));
        world.write_resource::<ShopUndoLedger>().owners.get_mut(&1).unwrap().records.last_mut().unwrap().source_generation = new.gen().id();
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(bound.sequence)).unwrap(), bound.sequence);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(bound.sequence)), Err(ShopError::UndoDuplicate));
        assert_eq!(world.read_storage::<Gold>().get(new).unwrap().0, 350);
        assert!(world.read_storage::<Inventory>().get(new).unwrap().find_item("moba_sword").is_none());
        assert_eq!(world.read_storage::<Inventory>().get(new).unwrap().find_item("moba_armor"), Some(1));
        assert_eq!(world.read_storage::<CProperty>().get(new).unwrap().hp, second.hp);
        driver.step(&mut world, []).unwrap();
        let after_undo = *world.read_storage::<ItemEffects>().get(new).unwrap();
        assert_eq!((after_undo.applied_hp, after_undo.applied_armor, after_undo.applied_atk), (100.0, 5.0, 0.0));
        assert_eq!(world.read_storage::<CProperty>().get(new).unwrap().mhp, second.mhp);
        assert!(world.read_storage::<CProperty>().get(new).unwrap().hp - second.hp < Fixed64::from_i32(100));
        assert_eq!(world.read_storage::<TAttack>().get(new).unwrap().atk_physic.clone().val(), atk - Fixed64::from_i32(10));
        assert_eq!(world.read_storage::<Gold>().get(heroes[1]).unwrap().0, 400);
        assert!(world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().records.iter().any(|record| {
            record.item_id == "moba_sword" && !record.eligible && record.source_index == heroes[0].id()
        }));
    }
    assert_eq!(dead_digests[0], dead_digests[1]);
    assert_eq!(respawn_digests[0], respawn_digests[1]);
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
    assert_eq!(authority.read_resource::<PendingItemUseQueue>().settlements[0].result, Err(ShopError::InsufficientGold));
    let owned = result.facts.iter().rev().find_map(|fact| match &fact.fact {
        ObservableFact::OwnerEconomy { state, .. } if state.player_id == 1 => Some(state), _ => None,
    }).unwrap();
    assert_eq!(owned.economy.gold, 0);
    assert_eq!(owned.economy.item_ids, [0; 6]);
    assert!(owned.shop_available);
    assert_eq!(owned.shop_reason, omoba_core::runtime::native::economy_projection::SHOP_REASON_AVAILABLE);
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
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse { item_slot: 0, target_pos: None, target_entity: None, point_explicit: None, target_generation: None, owner_entity: None, owner_generation: None })) }),
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
        item_slot: 0, target_pos: None, target_entity: None, point_explicit: None, target_generation: None, owner_entity: None, owner_generation: None,
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
    assert!(handle_item_use_from_input(&mut w, 0, None, None, 1, false).is_err());
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
            item_slot: 0, target_pos: None, target_entity: None, point_explicit: None, target_generation: None, owner_entity: None, owner_generation: None,
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
            ActiveEffect::ChannelHeal { .. } => panic!("channeled recovery has dedicated formal pulse fixtures"),
            ActiveEffect::SelfControlCleanse {} => panic!("control cleanse has dedicated formal fixtures"),
            ActiveEffect::PointNova { .. } | ActiveEffect::UnitHex { .. } => panic!("targeted training items have their own fixtures"),
        }
        let cooldown = w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining;
        assert!(cooldown > cfg.cooldown - 1.0 && cooldown <= cfg.cooldown);
        assert!(handle_item_use_from_input(&mut w, 0, None, None, 1, false).is_err());
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
        item_slot: 0, target_pos: None, target_entity: None, point_explicit: None, target_generation: None, owner_entity: None, owner_generation: None,
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
    assert!(handle_item_use_from_input(&mut w, 0, None, None, 1, false).is_err());
    assert_eq!(w.read_storage::<Hero>().get(hero).unwrap().mana_pool.as_ref().unwrap(), &pool);
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 0.0;
    assert!(handle_item_use_from_input(&mut w, 0, None, None, 1, false).is_ok());
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
        assert!(handle_item_use_from_input(&mut w, 0, None, None, 1, false).is_err(), "case {case}");
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
        assert!(handle_item_use_from_input(&mut w, 0, None, None, 1, false).is_err());
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
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse { item_slot: 0, target_pos: None, target_entity: None, point_explicit: None, target_generation: None, owner_entity: None, owner_generation: None })) }),
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
    assert!(handle_item_use_from_input(&mut w, 0, None, None, 1, false).is_ok());
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
    let public_shield=|w:&mut World,tick:u32,raw:i64| {
        let snapshot=extract_snapshot(w,tick,std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),
            std::sync::Arc::new(Vec::new()),Vec::new(),Vec::new());
        let shield=snapshot.entities.iter().find(|e|e.entity_id==hero.id()).and_then(|e|e.buffs.iter().find(|b|b.buff_id=="damage_shield"));
        if raw==0 {assert!(shield.is_none());}else {
            let buff=shield.unwrap();assert!(buff.remaining_secs>0. && buff.remaining_secs<=3.);
            assert_eq!(serde_json::from_str::<serde_json::Value>(&buff.payload_json).unwrap()["shield_remaining_raw"],raw);
        }
    };
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
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse { item_slot: 0, target_pos: None, target_entity: None, point_explicit: None, target_generation: None, owner_entity: None, owner_generation: None })) }),
    ]).unwrap();
    check(&result.facts, 100 * 1024);
    public_shield(&mut w,driver.tick() as u32,100*1024);
    w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage { target: hero, amount: Fixed64::from_i32(40) });
    let result = driver.step(&mut w, []).unwrap();
    check(&result.facts, 60 * 1024);
    public_shield(&mut w,driver.tick() as u32,60*1024);
    w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage { target: hero, amount: Fixed64::from_i32(60) });
    let result = driver.step(&mut w, []).unwrap();
    check(&result.facts, 0);
    public_shield(&mut w,driver.tick() as u32,0);
    w.write_resource::<BuffStore>().grant_damage_shield(hero, Fixed64::from_i32(20), Fixed64::from_raw(1));
    let result = driver.step(&mut w, []).unwrap();
    check(&result.facts, 0);
    public_shield(&mut w,driver.tick() as u32,0);
    w.write_resource::<BuffStore>().grant_damage_shield(hero, Fixed64::from_i32(20), Fixed64::from_i32(3));
    let pos = w.read_storage::<Pos>().get(hero).unwrap().0;
    w.write_resource::<Vec<Outcome>>().push(Outcome::Death { pos, ent: hero });
    let result = driver.step(&mut w, []).unwrap();
    check(&result.facts, 0);
    public_shield(&mut w,driver.tick() as u32,0);
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
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse { item_slot: 0, target_pos: None, target_entity: None, point_explicit: None, target_generation: None, owner_entity: None, owner_generation: None })) }),
    ]).unwrap();
    assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(hero), Fixed64::from_i32(60));
    assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(other), Fixed64::ZERO);
    assert_eq!(w.read_storage::<TAttack>().get(hero).unwrap().atk_physic.v, base);
    assert!(handle_item_use_from_input(&mut w, 0, None, None, 1, false).is_err());
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
        item_slot: slot, target_pos: None, target_entity: None, point_explicit: None, target_generation: None, owner_entity: None, owner_generation: None,
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
    assert!(handle_item_use_from_input(&mut w, 1, None, None, 1, false).is_err());
    assert_eq!(w.read_resource::<BuffStore>().get(hero, "item_damage_reduce:moba_sword").unwrap().remaining, remaining);
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[1].as_mut().unwrap().cooldown_remaining = 0.0;
    assert!(handle_item_use_from_input(&mut w, 1, None, None, 1, false).is_ok());
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
        assert!(handle_item_use_from_input(&mut w, 0, None, None, 1, false).is_err());
        assert!(!w.read_resource::<BuffStore>().has(hero, "item_damage_reduce:moba_armor"));
        assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 0.0);
        assert_eq!(format!("{:?}", w.read_storage::<HeroCommandQueue>().get(hero).unwrap()), command);
    }
}

fn raw_point(x: i32, y: i32) -> Vec2I {
    Vec2I { x: Fixed64::from_i32(x).raw() as i32, y: Fixed64::from_i32(y).raw() as i32 }
}

fn sim_point(x: i32, y: i32) -> omoba_sim::Vec2 {
    omoba_sim::Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y))
}

fn place_hero(world: &mut World, entity: specs::Entity, x: i32, y: i32) {
    world.write_storage::<Pos>().get_mut(entity).unwrap().0 = sim_point(x, y);
}

fn set_hold(world: &mut World, entity: specs::Entity) {
    let mut queues = world.write_storage::<HeroCommandQueue>();
    if let Some(queue) = queues.get_mut(entity) {
        queue.replace(HeroCommand::HoldPosition);
    } else {
        queues.insert(entity, HeroCommandQueue { active: Some(HeroCommand::HoldPosition), queued: Vec::new() }).unwrap();
    }
}

fn disarm_attack(world: &mut World, entity: specs::Entity) {
    world.write_storage::<TAttack>().get_mut(entity).unwrap().asd_count = Fixed64::from_i32(-100);
}

fn body(hp: i32) -> CProperty {
    CProperty { hp: Fixed64::from_i32(hp), mhp: Fixed64::from_i32(hp), msd: Fixed64::from_i32(300), def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }
}

fn spawn_body(world: &mut World, x: i32, y: i32, team: i32, hp: i32, hide_from: Option<u32>) -> specs::Entity {
    let entity = world.create_entity()
        .with(Pos(sim_point(x, y)))
        .with(Faction::new(FactionType::Enemy, team))
        .with(body(hp))
        .with(ReplicationScope { kind: ReplicationScopeKind::Vision, owner_team: Some(team.max(0) as u32) })
        .build();
    if let Some(viewer) = hide_from {
        world.write_storage::<VisibilityOverride>().insert(entity, VisibilityOverride {
            team: Some(viewer), kind: VisibilityOverrideKind::ForceHide, priority: 1, stable_rule_id: 1,
        }).unwrap();
    }
    entity
}

fn item_use(slot: u32, point: Option<Vec2I>, entity: Option<u32>, explicit: Option<bool>) -> (u32, PlayerInput) {
    (1, PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse {
        item_slot: slot, target_pos: point, target_entity: entity, point_explicit: explicit, target_generation: None, owner_entity: None, owner_generation: None,
    })) })
}

#[test]
fn item_point_nova_60hz_damages_visible_enemies_at_explicit_point_including_origin() {
    let (mut w, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    assert_eq!(w.read_resource::<MobaMatch>().phase, MobaMatchPhase::Playing);
    let [hero, enemy] = hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 5000;
    driver.step(&mut w, [(1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_training_flare".into() })) })]).unwrap();
    place_hero(&mut w, hero, 0, 7000);
    place_hero(&mut w, enemy, 80, 7000);
    set_hold(&mut w, hero);
    disarm_attack(&mut w, hero);
    disarm_attack(&mut w, enemy);
    let hidden = spawn_body(&mut w, 20, 7000, 2, 300, Some(1));
    let ally = spawn_body(&mut w, 10, 7000, 1, 300, None);
    let neutral = spawn_body(&mut w, 15, 7000, 0, 300, None);
    let owner_hp = w.read_storage::<CProperty>().get(hero).unwrap().hp;
    let enemy_hp = w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    driver.step(&mut w, [item_use(0, Some(raw_point(0, 7000)), None, Some(true))]).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().hp, owner_hp);
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp, enemy_hp - Fixed64::from_i32(40));
    assert_eq!(w.read_storage::<CProperty>().get(hidden).unwrap().hp, Fixed64::from_i32(300));
    assert_eq!(w.read_storage::<CProperty>().get(ally).unwrap().hp, Fixed64::from_i32(300));
    assert_eq!(w.read_storage::<CProperty>().get(neutral).unwrap().hp, Fixed64::from_i32(300));
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active, Some(HeroCommand::HoldPosition));
    let cooldown = w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining;
    assert!(cooldown > 7.0 && cooldown <= 8.0);
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 0.0;
    place_hero(&mut w, hero, 0, 0);
    place_hero(&mut w, enemy, 0, 80);
    let before_origin = w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    assert!(handle_item_use_from_input(&mut w, 0, Some(sim_point(0, 0)), None, 1, true).is_ok());
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp, before_origin - Fixed64::from_i32(40));
    assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().hp, owner_hp);
    assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 8.0);
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active, Some(HeroCommand::HoldPosition));
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 0.0;
    place_hero(&mut w, enemy, 200, 0);
    let after_hit = w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    assert!(handle_item_use_from_input(&mut w, 0, Some(sim_point(0, 0)), None, 1, true).is_ok());
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp, after_hit);
    assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 8.0);
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active, Some(HeroCommand::HoldPosition));
}

#[test]
fn unit_hex_healing_reduction_60hz_actual_cast_healing_pause_snapshot_and_expiry() {
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10000),
        summoner_loadouts:std::collections::BTreeMap::from([(2,[Some("training_self_heal_v1".into()),None])]),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [hero,enemy]=hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0=5000;
    driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::ItemBuy(ItemBuy {item_id:"moba_training_hex".into()}))})]).unwrap();
    let y=isolated_skill_origin_y();place_hero(&mut w,hero,0,y);place_hero(&mut w,enemy,500,y);
    set_hold(&mut w,hero);set_hold(&mut w,enemy);disarm_attack(&mut w,hero);disarm_attack(&mut w,enemy);
    assert!(handle_item_use_from_input(&mut w,0,None,Some(enemy.id()),1,false).is_err());
    assert!(!w.read_resource::<BuffStore>().has(enemy,"healing_reduction"));
    assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining,0.);
    place_hero(&mut w,enemy,400,y);
    driver.step(&mut w,[item_use(0,None,Some(enemy.id()),None)]).unwrap();
    assert!(w.read_resource::<BuffStore>().has(enemy,"healing_reduction"));
    let cast=|| (2,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:1,target_entity:None,target_pos:None,..Default::default()}))});
    w.write_storage::<CProperty>().get_mut(enemy).unwrap().hp=Fixed64::from_i32(100);
    driver.step(&mut w,[cast()]).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,Fixed64::from_i32(135),"actual generated70 heal becomes35 exactly once");
    let snapshot=extract_snapshot(&mut w,driver.tick() as u32,std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),Vec::new(),Vec::new());
    let buff=snapshot.entities.iter().find(|e|e.entity_id==enemy.id()).unwrap().buffs.iter().find(|b|b.buff_id=="healing_reduction").unwrap();
    assert!(buff.remaining_secs>3. && buff.remaining_secs<=4.);
    assert_eq!(serde_json::from_str::<serde_json::Value>(&buff.payload_json).unwrap()["heal_received_multiplier"],512);
    let remaining=w.read_resource::<BuffStore>().get(enemy,"healing_reduction").unwrap().remaining;
    w.write_resource::<GamePause>().is_paused=true;for _ in 0..10 {driver.step(&mut w,[]).unwrap();}
    assert_eq!(w.read_resource::<BuffStore>().get(enemy,"healing_reduction").unwrap().remaining,remaining);
    w.write_resource::<GamePause>().is_paused=false;
    w.write_resource::<Vec<Outcome>>().extend([Outcome::Heal {pos:sim_point(400,y),target:enemy,amount:Fixed64::from_i32(20)},Outcome::ScriptHeal {target:enemy,amount:Fixed64::from_i32(20)}]);
    driver.step(&mut w,[]).unwrap();assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,Fixed64::from_i32(155));
    w.write_storage::<CProperty>().get_mut(enemy).unwrap().hp=Fixed64::from_i32(500);
    driver.step(&mut w,[(2,PlayerInput {action:Some(PlayerInputEnum::SummonerCast(omoba_core::game_proto::SummonerCast {slot:0,x_raw:None,y_raw:None}))})]).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,Fixed64::from_i32(550),"summoner200 halves before50 missing-HP cap");
    for _ in 0..270 {driver.step(&mut w,[]).unwrap();}
    assert!(!w.read_resource::<BuffStore>().has(enemy,"healing_reduction"));
    w.write_storage::<Hero>().get_mut(enemy).unwrap().ability_cooldowns.clear();
    w.write_storage::<CProperty>().get_mut(enemy).unwrap().hp=Fixed64::from_i32(100);
    driver.step(&mut w,[cast()]).unwrap();assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,Fixed64::from_i32(170));
    w.write_resource::<BuffStore>().grant_healing_reduction(enemy,Fixed64::from_raw(512),Fixed64::from_i32(4));
    w.write_resource::<Vec<Outcome>>().push(Outcome::Death {ent:enemy,pos:sim_point(400,y)});driver.step(&mut w,[]).unwrap();
    assert!(!w.read_resource::<BuffStore>().has(enemy,"healing_reduction"));
}

#[test]
fn targeted_control_item_60hz_actual_catalog_root_pause_movement_and_expiry() {
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10000),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [hero,enemy]=hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0=5000;
    driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::ItemBuy(ItemBuy {item_id:"moba_binding_charm".into()}))})]).unwrap();
    assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0,4350);
    let y=isolated_skill_origin_y();place_hero(&mut w,hero,0,y);place_hero(&mut w,enemy,500,y);
    set_hold(&mut w,hero);set_hold(&mut w,enemy);disarm_attack(&mut w,hero);disarm_attack(&mut w,enemy);
    let hp=w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    assert!(handle_item_use_from_input(&mut w,0,None,Some(enemy.id()),1,false).is_err());
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,hp);
    assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining,0.);
    place_hero(&mut w,enemy,400,y);
    driver.step(&mut w,[item_use(0,None,Some(enemy.id()),None)]).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,hp-Fixed64::from_i32(30));
    let remaining=w.read_resource::<BuffStore>().get(enemy,"root").unwrap().remaining;
    assert!(remaining>Fixed64::from_i32(1)&&remaining<=Fixed64::from_raw(1536));
    let public=omoba_core::runtime::buff_visual_state::BuffVisualState::from_store(&w.read_resource::<BuffStore>(),enemy);
    assert!(public.render_buffs().iter().any(|b|b.buff_id=="root"&&b.remaining_secs>1.&&b.remaining_secs<=1.5));
    w.write_resource::<GamePause>().is_paused=true;for _ in 0..10 {driver.step(&mut w,[]).unwrap();}
    assert_eq!(w.read_resource::<BuffStore>().get(enemy,"root").unwrap().remaining,remaining);
    w.write_resource::<GamePause>().is_paused=false;
    let before=w.read_storage::<Pos>().get(enemy).unwrap().0;
    driver.step(&mut w,[(2,PlayerInput {action:Some(PlayerInputEnum::MoveTo(MoveTo {target:Some(Vec2I {x:600*1024,y:y*1024}),queued:false}))})]).unwrap();
    for _ in 0..10 {driver.step(&mut w,[]).unwrap();}
    assert_eq!(w.read_storage::<Pos>().get(enemy).unwrap().0,before);
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active,Some(HeroCommand::HoldPosition));
    for _ in 0..110 {driver.step(&mut w,[]).unwrap();}
    assert!(!w.read_resource::<BuffStore>().has(enemy,"root"));
    assert_ne!(w.read_storage::<Pos>().get(enemy).unwrap().0,before,"pending movement resumes after root expires");
    assert!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining>15.);
}

#[test]
fn self_cleanse_item_60hz_actual_catalog_works_while_controlled_and_preserves_other_effects() {
    let (mut w,mut driver)=world(SingleLaneConfig {wave_interval:Fixed64::from_i32(10000),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [hero,enemy]=hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0=5000;
    driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::ItemBuy(ItemBuy {item_id:"moba_purification_charm".into()}))})]).unwrap();
    assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0,3600);
    let y=isolated_skill_origin_y();place_hero(&mut w,hero,0,y);place_hero(&mut w,enemy,400,y);set_hold(&mut w,hero);set_hold(&mut w,enemy);disarm_attack(&mut w,hero);disarm_attack(&mut w,enemy);
    let hp=w.read_storage::<CProperty>().get(hero).unwrap().hp;
    {let mut buffs=w.write_resource::<BuffStore>();for kind in ["stun","root","silence"] {for entity in [hero,enemy] {buffs.add(entity,kind,Fixed64::from_i32(4),serde_json::json!({"control_probe":1024}));}}
     for kind in ["unrelated_slow","unrelated_dot"] {buffs.add(hero,kind,Fixed64::from_i32(7),serde_json::json!({"keep_probe":512}));}
     buffs.grant_healing_reduction(hero,Fixed64::from_raw(512),Fixed64::from_i32(4));buffs.grant_damage_shield(hero,Fixed64::from_i32(30),Fixed64::from_i32(5));}
    driver.step(&mut w,[item_use(0,None,None,None)]).unwrap();
    {let buffs=w.read_resource::<BuffStore>();for kind in ["stun","root","silence"] {assert!(!buffs.has(hero,kind));assert!(buffs.has(enemy,kind));}
     assert!(!buffs.entities_with_key("control_probe").any(|e|e==hero));assert!(buffs.entities_with_key("control_probe").any(|e|e==enemy));
     for kind in ["unrelated_slow","unrelated_dot"] {let effect=buffs.get(hero,kind).unwrap();assert_eq!(effect.payload["keep_probe"],512);assert!(effect.remaining>Fixed64::from_i32(6));}
     assert!(buffs.has(hero,"healing_reduction"));assert_eq!(buffs.shield_remaining(hero),Fixed64::from_i32(30));}
    assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().hp,hp);
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active,Some(HeroCommand::HoldPosition));
    let public=omoba_core::runtime::buff_visual_state::BuffVisualState::from_store(&w.read_resource::<BuffStore>(),hero);
    assert!(!public.render_buffs().iter().any(|b|["stun","root","silence"].contains(&b.buff_id.as_str())));
    let cooldown=w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining;assert!(cooldown>44.&&cooldown<=45.);
    w.write_resource::<BuffStore>().add(hero,"root",Fixed64::from_i32(2),serde_json::json!({}));
    assert!(handle_item_use_from_input(&mut w,0,None,None,1,false).is_err());assert!(w.read_resource::<BuffStore>().has(hero,"root"));
    driver.step(&mut w,[]).unwrap();assert!(w.read_resource::<BuffStore>().has(hero,"root"),"no immunity or replay cleanse");
}

#[test]
fn self_cleanse_item_rejections_are_atomic_and_empty_use_keeps_commands_and_spends_cooldown() {
    let (mut w,mut driver)=world(fast_config(),SimulationTickProfile::Production60Hz);driver.step(&mut w,[]).unwrap();let [hero,enemy]=hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0=5000;driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::ItemBuy(ItemBuy {item_id:"moba_purification_charm".into()}))})]).unwrap();
    set_hold(&mut w,hero);w.write_resource::<BuffStore>().add(hero,"stun",Fixed64::from_i32(4),serde_json::json!({}));
    let hp=w.read_storage::<CProperty>().get(hero).unwrap().hp;
    let unchanged=|w:&World| {assert!(w.read_resource::<BuffStore>().has(hero,"stun"));assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining,0.);assert_eq!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active,Some(HeroCommand::HoldPosition));};
    for (slot,owner) in [(6,1),(1,1),(0,999)] {assert!(handle_item_use_from_input(&mut w,slot,None,None,owner,false).is_err());unchanged(&w);}
    w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Warmup;assert!(handle_item_use_from_input(&mut w,0,None,None,1,false).is_err());unchanged(&w);w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
    w.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::ZERO;assert!(handle_item_use_from_input(&mut w,0,None,None,1,false).is_err());unchanged(&w);w.write_storage::<CProperty>().get_mut(hero).unwrap().hp=hp;
    {let mut registry=w.write_resource::<ItemRegistry>();std::sync::Arc::make_mut(registry.items.get_mut("moba_purification_charm").unwrap()).cooldown=f32::NAN;}
    assert!(handle_item_use_from_input(&mut w,0,None,None,1,false).is_err());unchanged(&w);
    {let mut registry=w.write_resource::<ItemRegistry>();std::sync::Arc::make_mut(registry.items.get_mut("moba_purification_charm").unwrap()).cooldown=45.;}
    w.write_resource::<BuffStore>().cleanse_controls(hero);
    let destination=sim_point(300,7000);w.write_storage::<HeroCommandQueue>().get_mut(hero).unwrap().active=Some(HeroCommand::MoveTo {pos:destination});
    handle_item_use_from_input(&mut w,0,Some(sim_point(1,1)),Some(enemy.id()),1,true).unwrap();
    assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining,45.);
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active,Some(HeroCommand::MoveTo {pos:destination}));assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().hp,hp);
}

#[test]
fn self_item_active_context_gate_covers_legacy_effect_pause_phase_and_roster() {
    for id in ["moba_guard_charm","moba_purification_charm"] {
        let (mut w,mut driver)=world(fast_config(),SimulationTickProfile::Production60Hz);driver.step(&mut w,[]).unwrap();let [hero,_]=hero_pair(&w);
        w.write_storage::<Gold>().get_mut(hero).unwrap().0=5000;driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::ItemBuy(ItemBuy {item_id:id.into()}))})]).unwrap();
        set_hold(&mut w,hero);w.write_resource::<BuffStore>().add(hero,"stun",Fixed64::from_i32(4),serde_json::json!({}));
        for mode in 0..3 {
            if mode==0 {w.write_resource::<GamePause>().is_paused=true;}
            if mode==1 {w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Warmup;}
            if mode==2 {w.write_resource::<MobaMatch>().heroes[0].entity=None;}
            assert!(handle_item_use_from_input(&mut w,0,None,None,1,false).is_err(),"{id} mode{mode}");
            assert!(w.read_resource::<BuffStore>().has(hero,"stun"));assert_eq!(w.read_resource::<BuffStore>().shield_remaining(hero),Fixed64::ZERO);
            assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining,0.);
            assert_eq!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active,Some(HeroCommand::HoldPosition));
            w.write_resource::<GamePause>().is_paused=false;w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;w.write_resource::<MobaMatch>().heroes[0].entity=Some(hero);
        }
    }
}

#[test]
fn targeted_control_transaction_is_generic_atomic_refreshes_and_skips_dead_targets() {
    use omoba_content_model::ControlEffectKind;
    for kind in [ControlEffectKind::Root,ControlEffectKind::Stun,ControlEffectKind::Silence] {
        let (mut w,mut driver)=world(fast_config(),SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let [hero,enemy]=hero_pair(&w);
        w.write_storage::<Gold>().get_mut(hero).unwrap().0=5000;
        driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::ItemBuy(ItemBuy {item_id:"moba_binding_charm".into()}))})]).unwrap();
        let y=isolated_skill_origin_y();place_hero(&mut w,hero,0,y);place_hero(&mut w,enemy,400,y);set_hold(&mut w,hero);set_hold(&mut w,enemy);
        let set_duration=|w:&mut World,duration| {
            let mut registry=w.write_resource::<ItemRegistry>();let config=std::sync::Arc::make_mut(registry.items.get_mut("moba_binding_charm").unwrap());
            let Some(ActiveEffect::UnitHex {control:Some(control),..})=&mut config.active else {panic!("authored control")};
            control.kind=kind;control.duration=duration;
        };
        let hp=w.read_storage::<CProperty>().get(enemy).unwrap().hp;
        for duration in [Fixed64::ZERO,Fixed64::from_i32(-1),Fixed64::from_i32(61)] {
            set_duration(&mut w,duration);assert!(handle_item_use_from_input(&mut w,0,None,Some(enemy.id()),1,false).is_err());
            assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,hp);
            assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining,0.);
            assert!(!w.read_resource::<BuffStore>().has(enemy,kind.as_str()));
        }
        set_duration(&mut w,Fixed64::from_raw(1536));
        w.write_storage::<IsBuilding>().insert(enemy,IsBuilding).unwrap();
        assert!(handle_item_use_from_input(&mut w,0,None,Some(enemy.id()),1,false).is_err());
        assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp,hp);
        assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining,0.);
        w.write_storage::<IsBuilding>().remove(enemy);
        w.write_resource::<BuffStore>().add(enemy,kind.as_str(),Fixed64::from_i32(4),serde_json::json!({}));
        handle_item_use_from_input(&mut w,0,None,Some(enemy.id()),1,false).unwrap();
        assert_eq!(w.read_resource::<BuffStore>().get(enemy,kind.as_str()).unwrap().remaining,Fixed64::from_i32(4),"shorter refresh never truncates existing control");
        w.write_resource::<BuffStore>().remove(enemy,kind.as_str());
        w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().cooldown_remaining=0.;
        w.write_storage::<CProperty>().get_mut(enemy).unwrap().hp=Fixed64::from_i32(1);
        handle_item_use_from_input(&mut w,0,None,Some(enemy.id()),1,false).unwrap();
        assert!(w.read_storage::<CProperty>().get(enemy).unwrap().hp<=Fixed64::ZERO);
        assert!(!w.read_resource::<BuffStore>().has(enemy,kind.as_str()),"lethal item never adds corpse control");
    }
}

#[test]
fn item_unit_hex_60hz_damages_visible_enemy_and_ignores_legacy_point() {
    let (mut w, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let [hero, enemy] = hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 5000;
    driver.step(&mut w, [(1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_training_hex".into() })) })]).unwrap();
    place_hero(&mut w, hero, 0, 7000);
    place_hero(&mut w, enemy, 400, 7000);
    set_hold(&mut w, hero);
    disarm_attack(&mut w, hero);
    disarm_attack(&mut w, enemy);
    let enemy_hp = w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    let owner_hp = w.read_storage::<CProperty>().get(hero).unwrap().hp;
    driver.step(&mut w, [item_use(0, Some(raw_point(0, 0)), Some(enemy.id()), None)]).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp, enemy_hp - Fixed64::from_i32(55));
    assert_eq!(w.read_storage::<CProperty>().get(hero).unwrap().hp, owner_hp);
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active, Some(HeroCommand::HoldPosition));
    let cooldown = w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining;
    assert!(cooldown > 7.0 && cooldown <= 8.0);
    assert!(handle_item_use_from_input(&mut w, 0, Some(sim_point(0, 0)), Some(enemy.id()), 1, false).is_err());
    assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, cooldown);
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp, enemy_hp - Fixed64::from_i32(55));
}

#[test]
fn item_target_rejections_are_non_mutating_for_shape_range_team_fog_death_owner_and_replacement() {
    let (mut w, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let [hero, enemy] = hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 5000;
    driver.step(&mut w, [
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_training_flare".into() })) }),
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_training_hex".into() })) }),
    ]).unwrap();
    place_hero(&mut w, hero, 0, 7000);
    place_hero(&mut w, enemy, 400, 7000);
    set_hold(&mut w, hero);
    let ally = spawn_body(&mut w, 100, 7000, 1, 250, None);
    let hidden = spawn_body(&mut w, 120, 7000, 2, 250, Some(1));
    let deleted = spawn_body(&mut w, 140, 7000, 2, 250, None);
    let deleted_id = deleted.id();
    assert!(w.delete_entity(deleted).is_ok());
    w.maintain();
    let read = |w: &World| {
        let hero_hp = w.read_storage::<CProperty>().get(hero).unwrap().hp;
        let enemy_hp = w.read_storage::<CProperty>().get(enemy).unwrap().hp;
        let ally_hp = w.read_storage::<CProperty>().get(ally).unwrap().hp;
        let hidden_hp = w.read_storage::<CProperty>().get(hidden).unwrap().hp;
        let gold = w.read_storage::<Gold>().get(hero).unwrap().0;
        let inventory = w.read_storage::<Inventory>();
        let cd0 = inventory.get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining;
        let cd1 = inventory.get(hero).unwrap().slots[1].as_ref().unwrap().cooldown_remaining;
        drop(inventory);
        let command = w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active;
        (hero_hp, enemy_hp, ally_hp, hidden_hp, gold, cd0, cd1, command)
    };
    let before = read(&w);
    let refuse = |w: &mut World, slot, point: Option<omoba_sim::Vec2>, entity, explicit: bool| {
        assert!(handle_item_use_from_input(w, slot, point, entity, 1, explicit).is_err());
    };
    w.write_resource::<MobaMatch>().phase = MobaMatchPhase::Warmup;
    refuse(&mut w, 0, Some(sim_point(0, 7000)), None, true);
    w.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
    w.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::ZERO;
    refuse(&mut w, 0, Some(sim_point(0, 7000)), None, true);
    w.write_storage::<CProperty>().get_mut(hero).unwrap().hp = before.0;
    refuse(&mut w, 0, Some(sim_point(600, 7000)), None, true);
    refuse(&mut w, 0, Some(sim_point(0, 7000)), None, false);
    refuse(&mut w, 0, Some(sim_point(0, 7000)), Some(enemy.id()), true);
    refuse(&mut w, 0, None, None, false);
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().item_id = "moba_training_hex".into();
    refuse(&mut w, 0, Some(sim_point(0, 7000)), None, true);
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().item_id = "moba_sword".into();
    refuse(&mut w, 0, Some(sim_point(0, 7000)), None, true);
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().item_id = "moba_training_flare".into();
    let saved = (*w.read_resource::<ItemRegistry>()).clone();
    w.insert(ItemRegistry::from_configs(vec![ItemConfig {
        id: "moba_armor".into(), name: "Mismatched nova".into(), cost: 100, bonus: ItemBonus::default(),
        active: Some(ActiveEffect::PointNova { damage: Fixed64::from_i32(40), radius: Fixed64::from_i32(80) }), cooldown: 8.0, recipe: vec![],
    }]));
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().item_id = "moba_armor".into();
    refuse(&mut w, 0, Some(sim_point(0, 7000)), None, true);
    w.insert(saved);
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().item_id = "moba_training_flare".into();
    refuse(&mut w, 1, None, Some(ally.id()), false);
    refuse(&mut w, 1, None, Some(hidden.id()), false);
    w.write_storage::<CProperty>().get_mut(enemy).unwrap().hp = Fixed64::ZERO;
    refuse(&mut w, 1, Some(sim_point(0, 0)), Some(enemy.id()), false);
    w.write_storage::<CProperty>().get_mut(enemy).unwrap().hp = before.1;
    refuse(&mut w, 1, None, Some(u32::MAX), false);
    refuse(&mut w, 1, None, Some(deleted_id), false);
    place_hero(&mut w, enemy, 2000, 7000);
    refuse(&mut w, 1, None, Some(enemy.id()), false);
    place_hero(&mut w, enemy, 400, 7000);
    refuse(&mut w, 1, Some(sim_point(0, 7000)), Some(enemy.id()), true);
    refuse(&mut w, 1, None, None, false);
    w.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::ZERO;
    refuse(&mut w, 1, None, Some(enemy.id()), false);
    w.write_storage::<CProperty>().get_mut(hero).unwrap().hp = before.0;
    assert_eq!(read(&w), before);
}

#[test]
fn item_point_shape_rejects_unresolved_entity_presence() {
    let (mut w, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let [hero, enemy] = hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 5000;
    driver.step(&mut w, [
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_training_flare".into() })) }),
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_guard_charm".into() })) }),
    ]).unwrap();
    place_hero(&mut w, hero, 0, 0);
    place_hero(&mut w, enemy, 0, 80);
    set_hold(&mut w, hero);
    disarm_attack(&mut w, hero);
    disarm_attack(&mut w, enemy);
    let read = |w: &World| {
        let hero_hp = w.read_storage::<CProperty>().get(hero).unwrap().hp;
        let enemy_hp = w.read_storage::<CProperty>().get(enemy).unwrap().hp;
        let gold = w.read_storage::<Gold>().get(hero).unwrap().0;
        let inventory = w.read_storage::<Inventory>();
        let flare = inventory.get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining;
        let charm = inventory.get(hero).unwrap().slots[1].as_ref().unwrap().cooldown_remaining;
        drop(inventory);
        let command = w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active;
        (hero_hp, enemy_hp, gold, flare, charm, command)
    };
    let before = read(&w);
    assert_eq!(before.3, 0.0);
    assert_eq!(before.4, 0.0);
    assert_eq!(before.5, Some(HeroCommand::HoldPosition));
    let reject_direct = |w: &mut World, entity: Option<u32>| {
        assert!(handle_item_use_from_input(w, 0, Some(sim_point(0, 0)), entity, 1, true).is_err());
    };
    reject_direct(&mut w, Some(u32::MAX));
    let missing = spawn_body(&mut w, 0, 40, 2, 300, None);
    let missing_id = missing.id();
    let missing_gen = missing.gen().id() as u32;
    assert!(w.delete_entity(missing).is_ok());
    w.maintain();
    reject_direct(&mut w, Some(missing_id));
    reject_direct(&mut w, Some(enemy.id()));
    assert_eq!(read(&w), before);

    let queue_point = |w: &mut World, slot: u32, index: Option<u32>, generation: Option<u32>| {
        let item = ItemUse {
            item_slot: slot, target_pos: Some(raw_point(0, 0)), target_entity: index, point_explicit: Some(true),
            target_generation: generation, owner_entity: None, owner_generation: None,
        };
        w.write_resource::<PendingItemUseQueue>().requests.push(PendingItemAction::Use(
            pending_item_use_from_command(1, &item),
        ));
    };
    queue_point(&mut w, 0, Some(u32::MAX), Some(u32::MAX));
    bind_queued_item_lives(&w);
    match &w.read_resource::<PendingItemUseQueue>().requests[0] {
        PendingItemAction::Use(request) => {
            assert_eq!(request.target_entity, Some(u32::MAX));
            assert_eq!(request.target_generation, Some(u32::MAX));
            assert_eq!(request.target_life, None);
            assert!(request.point_explicit);
            assert_eq!(request.target_pos, Some(sim_point(0, 0)));
        }
        other => panic!("expected the unresolved point shape to stay queued, got {other:?}"),
    }
    drain_pending_item_uses(&mut w);
    assert!(w.read_resource::<PendingItemUseQueue>().requests.is_empty());
    assert_eq!(read(&w), before);
    let stale_gen = (enemy.gen().id() as u32).saturating_add(1);
    for (index, generation) in [
        (Some(u32::MAX), None),
        (Some(missing_id), None),
        (Some(missing_id), Some(missing_gen)),
        (Some(enemy.id()), Some(0)),
        (Some(enemy.id()), Some(stale_gen)),
        (None, Some(enemy.gen().id() as u32)),
        (None, Some(0)),
    ] {
        queue_point(&mut w, 0, index, generation);
        drain_pending_item_uses(&mut w);
        assert_eq!(read(&w), before, "queued point {index:?} gen {generation:?}");
    }
    let accepted = spawn_body(&mut w, 0, 40, 2, 300, None);
    queue_point(&mut w, 0, Some(accepted.id()), Some(accepted.gen().id() as u32));
    assert!(w.delete_entity(accepted).is_ok());
    w.maintain();
    let recycled = spawn_body(&mut w, 0, 40, 2, 300, None);
    assert_eq!(recycled.id(), accepted.id());
    assert_ne!(recycled.gen(), accepted.gen());
    drain_pending_item_uses(&mut w);
    assert_eq!(w.read_storage::<CProperty>().get(recycled).unwrap().hp, Fixed64::from_i32(300));
    assert_eq!(read(&w), before);

    assert!(handle_item_use_from_input(&mut w, 0, Some(sim_point(0, 0)), None, 1, true).is_ok());
    let legal = read(&w);
    assert_eq!(legal.0, before.0);
    assert_eq!(legal.1, before.1 - Fixed64::from_i32(40));
    assert_eq!(legal.2, before.2);
    assert_eq!(legal.3, 8.0);
    assert_eq!(legal.4, 0.0);
    assert_eq!(legal.5, Some(HeroCommand::HoldPosition));
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 0.0;
    queue_point(&mut w, 0, None, None);
    drain_pending_item_uses(&mut w);
    let queued_legal = read(&w);
    assert_eq!(queued_legal.1, legal.1 - Fixed64::from_i32(40));
    assert_eq!(queued_legal.2, before.2);
    assert_eq!(queued_legal.3, 8.0);
    assert_eq!(queued_legal.4, 0.0);
    assert_eq!(queued_legal.5, Some(HeroCommand::HoldPosition));

    assert!(handle_item_use_from_input(&mut w, 1, Some(sim_point(9, 9)), Some(u32::MAX), 1, true).is_ok());
    let self_use = read(&w);
    assert_eq!(self_use.0, before.0);
    assert_eq!(self_use.1, queued_legal.1);
    assert_eq!(self_use.2, before.2);
    assert_eq!(self_use.3, 8.0);
    assert!(self_use.4 > 0.0);
    assert!(self_use.5.is_none());
    assert!(w.read_resource::<BuffStore>().shield_remaining(hero) > Fixed64::ZERO);
    assert!(handle_item_use_from_input(&mut w, 1, Some(sim_point(9, 9)), Some(enemy.id()), 1, true).is_err());
    assert_eq!(read(&w).4, self_use.4);
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[1].as_mut().unwrap().cooldown_remaining = 0.0;
    set_hold(&mut w, hero);
    queue_point(&mut w, 1, Some(u32::MAX), Some(1));
    drain_pending_item_uses(&mut w);
    let queued_self = read(&w);
    assert_eq!(queued_self.1, queued_legal.1);
    assert_eq!(queued_self.2, before.2);
    assert_eq!(queued_self.3, 8.0);
    assert!(queued_self.4 > 0.0);
    assert!(queued_self.5.is_none());
    assert!(w.read_resource::<BuffStore>().shield_remaining(hero) > Fixed64::ZERO);
}

#[test]
fn item_targeted_cooldown_is_independent_and_does_not_clear_commands() {
    let (mut w, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let [hero, enemy] = hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 5000;
    driver.step(&mut w, [
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_training_flare".into() })) }),
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_guard_charm".into() })) }),
    ]).unwrap();
    place_hero(&mut w, hero, 0, 7000);
    place_hero(&mut w, enemy, 80, 7000);
    set_hold(&mut w, hero);
    disarm_attack(&mut w, hero);
    disarm_attack(&mut w, enemy);
    let enemy_before = w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    driver.step(&mut w, [item_use(0, Some(raw_point(0, 7000)), None, Some(true))]).unwrap();
    let flare_cooldown = w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining;
    assert!(flare_cooldown > 7.0 && flare_cooldown <= 8.0);
    assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[1].as_ref().unwrap().cooldown_remaining, 0.0);
    assert_eq!(w.read_resource::<BuffStore>().shield_remaining(hero), Fixed64::ZERO);
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active, Some(HeroCommand::HoldPosition));
    let damaged = w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    assert_eq!(damaged, enemy_before - Fixed64::from_i32(40));
    assert!(handle_item_use_from_input(&mut w, 1, Some(sim_point(0, 7000)), Some(enemy.id()), 1, true).is_ok());
    assert!(w.read_resource::<BuffStore>().shield_remaining(hero) > Fixed64::ZERO);
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp, damaged);
    assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, flare_cooldown);
    assert!(w.read_storage::<Inventory>().get(hero).unwrap().slots[1].as_ref().unwrap().cooldown_remaining > 0.0);
    assert!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active.is_none());
}

#[test]
fn item_target_credit_60hz_records_kill_assist_and_lane_last_hit() {
    let (mut w, mut driver) = world(SingleLaneConfig {
        additional_players: vec![extra_player(3, 1)],
        wave_interval: Fixed64::from_i32(10_000),
        hero_kill_gold: 0, hero_assist_gold: 0, hero_kill_xp: 0, hero_assist_xp: 0,
        lane_creep_gold: 0, lane_creep_xp: 0, ..fast_config()
    }, SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let hero_of = |w: &World, player: u32| w.read_resource::<MobaMatch>().heroes.iter()
        .find(|slot| slot.player_id == player).unwrap().entity.unwrap();
    let [owner, enemy] = hero_pair(&w);
    let ally = hero_of(&w, 3);
    let place_at_own_base = |w: &mut World, player: u32| {
        let state = w.read_resource::<MobaMatch>();
        let slot = state.heroes.iter().find(|slot| slot.player_id == player).unwrap();
        let hero = slot.entity.unwrap();
        let base = state.bases[slot.side].unwrap();
        drop(state);
        let base_pos = w.read_storage::<Pos>().get(base).unwrap().0;
        w.write_storage::<Pos>().get_mut(hero).unwrap().0 = base_pos;
    };
    place_at_own_base(&mut w, 1);
    place_at_own_base(&mut w, 3);
    w.write_storage::<Gold>().get_mut(owner).unwrap().0 = 5000;
    w.write_storage::<Gold>().get_mut(ally).unwrap().0 = 5000;
    driver.step(&mut w, [
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_training_hex".into() })) }),
        (3, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_training_hex".into() })) }),
    ]).unwrap();
    let purchased = |w: &World, entity: specs::Entity| {
        w.read_storage::<Inventory>().get(entity).unwrap().slots[0].as_ref().unwrap().item_id.clone()
    };
    assert_eq!(purchased(&w, owner), "moba_training_hex");
    assert_eq!(purchased(&w, ally), "moba_training_hex");
    assert_eq!(w.read_storage::<Gold>().get(owner).unwrap().0, 5000 - 450);
    assert_eq!(w.read_storage::<Gold>().get(ally).unwrap().0, 5000 - 450);
    for entity in [owner, enemy, ally] {
        place_hero(&mut w, entity, 0, 7000);
        disarm_attack(&mut w, entity);
        set_hold(&mut w, entity);
    }
    place_hero(&mut w, enemy, 400, 7000);
    {
        let mut state = w.write_resource::<MobaMatch>();
        state.config.hero_kill_gold = 40;
        state.config.hero_assist_gold = 15;
        state.config.lane_creep_gold = 20;
    }
    let gold = |w: &World, entity: specs::Entity| w.read_storage::<Gold>().get(entity).unwrap().0;
    let counts = |w: &World, player: u32| {
        let state = w.read_resource::<MobaMatch>();
        let slot = state.heroes.iter().find(|slot| slot.player_id == player).unwrap();
        (slot.kills, slot.assists)
    };
    assert_eq!(gold(&w, owner), 5000 - 450);
    assert_eq!(gold(&w, ally), 5000 - 450);
    assert_eq!(counts(&w, 1), (0, 0));
    assert_eq!(counts(&w, 3), (0, 0));
    w.write_storage::<CProperty>().get_mut(enemy).unwrap().hp = Fixed64::from_i32(800);
    w.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage { target: enemy, amount: Fixed64::from_i32(1) });
    process_outcomes(&mut w, &mut RuntimeEventVecSink::default()).unwrap();
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp, Fixed64::from_i32(799));
    assert_eq!(gold(&w, owner), 5000 - 450, "anonymous script damage does not buy participation");
    assert_eq!(counts(&w, 1), (0, 0));
    assert!(handle_item_use_from_input(&mut w, 0, None, Some(enemy.id()), 1, false).is_ok());
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp, Fixed64::from_i32(799 - 55));
    assert_eq!(gold(&w, owner), 5000 - 450);
    assert_eq!(counts(&w, 1), (0, 0));
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active, Some(HeroCommand::HoldPosition));
    w.write_storage::<CProperty>().get_mut(enemy).unwrap().hp = Fixed64::from_i32(55);
    assert!(handle_item_use_from_input(&mut w, 0, None, Some(enemy.id()), 3, false).is_ok());
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp, Fixed64::ZERO);
    assert_eq!(gold(&w, ally), 5000 - 450 + 40);
    assert_eq!(gold(&w, owner), 5000 - 450 + 15);
    assert_eq!(counts(&w, 3), (1, 0));
    assert_eq!(counts(&w, 1), (0, 1));
    assert_eq!(counts(&w, 2), (0, 0));
    w.write_storage::<Inventory>().get_mut(owner).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 0.0;
    let creep = lane_creeps(&w, 2)[0];
    place_hero(&mut w, creep, 100, 7000);
    w.write_storage::<CProperty>().get_mut(creep).unwrap().hp = Fixed64::from_i32(55);
    assert!(handle_item_use_from_input(&mut w, 0, None, Some(creep.id()), 1, false).is_ok());
    assert_eq!(w.read_storage::<CProperty>().get(creep).unwrap().hp, Fixed64::ZERO);
    assert_eq!(gold(&w, owner), 5000 - 450 + 15 + 20);
    assert_eq!(gold(&w, ally), 5000 - 450 + 40);
    assert_eq!(counts(&w, 1), (0, 1), "lane last hit is not a hero kill");
    assert_eq!(counts(&w, 3), (1, 0));
    w.write_storage::<Inventory>().get_mut(owner).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 0.0;
    let shielded = spawn_body(&mut w, 80, 7000, 2, 200, None);
    assert!(w.write_resource::<BuffStore>().grant_damage_shield(shielded, Fixed64::from_i32(55), Fixed64::from_i32(10)));
    assert!(handle_item_use_from_input(&mut w, 0, None, Some(shielded.id()), 1, false).is_ok());
    assert_eq!(w.read_storage::<CProperty>().get(shielded).unwrap().hp, Fixed64::from_i32(200));
    assert_eq!(w.read_resource::<BuffStore>().shield_remaining(shielded), Fixed64::ZERO);
    assert_eq!(gold(&w, owner), 5000 - 450 + 15 + 20);
    assert_eq!(counts(&w, 1), (0, 1));
    assert_eq!(w.read_storage::<Inventory>().get(owner).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 8.0);
    w.write_storage::<Inventory>().get_mut(owner).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 0.0;
    place_hero(&mut w, shielded, 2000, 7000);
    let command = w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active;
    assert!(handle_item_use_from_input(&mut w, 0, None, Some(shielded.id()), 1, false).is_err());
    assert_eq!(w.read_storage::<CProperty>().get(shielded).unwrap().hp, Fixed64::from_i32(200));
    assert_eq!(w.read_storage::<Inventory>().get(owner).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 0.0);
    assert_eq!(gold(&w, owner), 5000 - 450 + 15 + 20);
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(owner).unwrap().active, command);
}

#[test]
fn item_target_life_rejects_reused_index_and_reassigned_owner() {
    let (mut w, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let [hero, enemy] = hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 5000;
    driver.step(&mut w, [(1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_training_hex".into() })) })]).unwrap();
    place_hero(&mut w, hero, 0, 7000);
    place_hero(&mut w, enemy, 400, 7000);
    set_hold(&mut w, hero);
    disarm_attack(&mut w, hero);
    disarm_attack(&mut w, enemy);
    let queue_use = |w: &mut World, index: Option<u32>, generation: Option<u32>| {
        w.write_resource::<PendingItemUseQueue>().requests.push(PendingItemAction::Use(PendingItemUse {
            item_slot: 0, target_pos: None, point_explicit: false, target_entity: index, target_generation: generation,
            owner_pid: 1, owner_index: None, owner_generation: None, lives_bound: false, owner_entity: None, target_life: None,
        }));
    };
    let snapshot = |w: &World| {
        let enemy_hp = w.read_storage::<CProperty>().get(enemy).unwrap().hp;
        let gold = w.read_storage::<Gold>().get(hero).unwrap().0;
        let cooldown = w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining;
        let command = w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active;
        (enemy_hp, gold, cooldown, command)
    };
    let body = spawn_body(&mut w, 120, 7000, 2, 250, None);
    let before = snapshot(&w);
    queue_use(&mut w, Some(body.id()), Some(body.gen().id() as u32));
    bind_queued_item_lives(&w);
    match &w.read_resource::<PendingItemUseQueue>().requests[0] {
        PendingItemAction::Use(request) => {
            assert_eq!(request.owner_entity, Some(hero));
            assert_eq!(request.target_life, Some(body));
            assert!(request.lives_bound);
        }
        other => panic!("expected a bound item use, got {other:?}"),
    }
    assert!(w.delete_entity(body).is_ok());
    w.maintain();
    let recycled = spawn_body(&mut w, 120, 7000, 2, 250, None);
    assert_eq!(recycled.id(), body.id(), "specs must hand the deleted index back");
    assert_ne!(recycled.gen(), body.gen());
    match &w.read_resource::<PendingItemUseQueue>().requests[0] {
        PendingItemAction::Use(request) => assert_eq!(request.target_life, Some(body)),
        other => panic!("expected the bound life to survive recycle, got {other:?}"),
    }
    drain_pending_item_uses(&mut w);
    assert!(!w.entities().is_alive(body));
    assert_eq!(w.read_storage::<CProperty>().get(recycled).unwrap().hp, Fixed64::from_i32(250));
    assert_eq!(snapshot(&w), before);
    queue_use(&mut w, Some(enemy.id()), Some(enemy.gen().id() as u32));
    bind_queued_item_lives(&w);
    w.write_storage::<PlayerOwner>().get_mut(hero).unwrap().player_id = 99;
    drain_pending_item_uses(&mut w);
    assert_eq!(snapshot(&w), before);
    w.write_storage::<PlayerOwner>().get_mut(hero).unwrap().player_id = 1;
    queue_use(&mut w, Some(enemy.id()), Some(0));
    bind_queued_item_lives(&w);
    match &w.read_resource::<PendingItemUseQueue>().requests[0] {
        PendingItemAction::Use(request) => assert_eq!(request.target_life, None),
        other => panic!("generation 0 is not the current entity, got {other:?}"),
    }
    drain_pending_item_uses(&mut w);
    assert_eq!(snapshot(&w), before);
    queue_use(&mut w, None, Some(enemy.gen().id() as u32));
    bind_queued_item_lives(&w);
    match &w.read_resource::<PendingItemUseQueue>().requests[0] {
        PendingItemAction::Use(request) => {
            assert_eq!(request.target_life, None);
            assert_ne!(request.target_life, Some(w.entities().entity(0)));
        }
        other => panic!("a generation without an index is not a target, got {other:?}"),
    }
    drain_pending_item_uses(&mut w);
    assert_eq!(snapshot(&w), before);
    queue_use(&mut w, Some(enemy.id()), Some(enemy.gen().id() as u32));
    bind_queued_item_lives(&w);
    let hero_life = w.read_storage::<Hero>().get(hero).unwrap().clone();
    let inventory = w.read_storage::<Inventory>().get(hero).unwrap().clone();
    let property = w.read_storage::<CProperty>().get(hero).unwrap().clone();
    let gold = w.read_storage::<Gold>().get(hero).unwrap().0;
    assert!(w.delete_entity(hero).is_ok());
    w.maintain();
    let replaced = w.create_entity()
        .with(hero_life)
        .with(PlayerOwner::new(1))
        .with(property)
        .with(Gold(gold))
        .with(inventory)
        .with(Pos(sim_point(0, 7000)))
        .with(Faction::new(FactionType::Player, 1))
        .with(VisionSource { team: 1, radius: Fixed64::from_i32(1000), detection_level: 0 })
        .build();
    assert_eq!(replaced.id(), hero.id());
    assert_ne!(replaced.gen(), hero.gen());
    drain_pending_item_uses(&mut w);
    assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp, before.0);
    assert_eq!(w.read_storage::<Gold>().get(replaced).unwrap().0, gold);
    assert_eq!(w.read_storage::<Inventory>().get(replaced).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 0.0);
    assert!(!w.entities().is_alive(hero));
}

#[test]
fn item_accepted_owner_life_rejects_replacement_before_first_bind() {
    let (mut w, mut driver) = world(SingleLaneConfig {
        additional_players: vec![extra_player(3, 1)],
        wave_interval: Fixed64::from_i32(10_000),
        ..fast_config()
    }, SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let hero_of = |w: &World, player: u32| w.read_resource::<MobaMatch>().heroes.iter()
        .find(|slot| slot.player_id == player).unwrap().entity.unwrap();
    let [hero, enemy] = hero_pair(&w);
    let ally = hero_of(&w, 3);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 5000;
    w.write_storage::<Gold>().get_mut(ally).unwrap().0 = 5000;
    driver.step(&mut w, [
        (1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_training_hex".into() })) }),
        (3, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_training_hex".into() })) }),
    ]).unwrap();
    for entity in [hero, ally] {
        place_hero(&mut w, entity, 0, 7000);
        set_hold(&mut w, entity);
        disarm_attack(&mut w, entity);
        w.write_storage::<CProperty>().get_mut(entity).unwrap().hp = Fixed64::from_i32(800);
    }
    place_hero(&mut w, enemy, 400, 7000);
    disarm_attack(&mut w, enemy);
    w.write_storage::<CProperty>().get_mut(enemy).unwrap().hp = Fixed64::from_i32(800);
    let cd = |w: &World, entity: specs::Entity| {
        w.read_storage::<Inventory>().get(entity).unwrap().slots[0].as_ref().unwrap().cooldown_remaining
    };
    let gold = |w: &World, entity: specs::Entity| w.read_storage::<Gold>().get(entity).unwrap().0;
    let command = |w: &World, entity: specs::Entity| w.read_storage::<HeroCommandQueue>().get(entity).unwrap().active;
    let enemy_hp = |w: &World| w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    let before_hp = enemy_hp(&w);
    let before_gold = gold(&w, hero);
    let ally_gold = gold(&w, ally);
    let before_command = command(&w, hero);
    assert_eq!(before_command, Some(HeroCommand::HoldPosition));
    assert_eq!(cd(&w, hero), 0.0);
    assert_eq!(cd(&w, ally), 0.0);
    let unchanged = |w: &World| {
        assert_eq!(enemy_hp(w), before_hp);
        assert_eq!(gold(w, hero), before_gold);
        assert_eq!(gold(w, ally), ally_gold);
        assert_eq!(cd(w, hero), 0.0);
        assert_eq!(cd(w, ally), 0.0);
        assert_eq!(command(w, hero), before_command);
        assert_eq!(command(w, ally), Some(HeroCommand::HoldPosition));
    };
    let parts = |input: &PlayerInput| match &input.action {
        Some(PlayerInputEnum::ItemUse(value)) => (
            value.owner_entity, value.owner_generation, value.target_entity, value.target_generation,
        ),
        other => panic!("expected ItemUse, got {other:?}"),
    };
    let mut forged = PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse {
        item_slot: 0, target_pos: None, target_entity: Some(ally.id()), point_explicit: None,
        target_generation: Some(99), owner_entity: Some(ally.id()),
        owner_generation: Some(ally.gen().id() as u32),
    })) };
    assert!(!seal_item_use_owner(&w, 1, &mut forged), "another player's actor is not rewritten onto player 1");
    assert_eq!(parts(&forged).0, Some(ally.id()));
    assert_eq!(parts(&forged).1, Some(ally.gen().id() as u32));
    clear_untrusted_item_life(&mut forged);
    assert_eq!((parts(&forged).0, parts(&forged).1, parts(&forged).3), (None, None, None));
    assert_eq!(parts(&forged).2, Some(ally.id()), "legacy target index is not an owner life");
    assert!(seal_item_use_owner(&w, 1, &mut forged));
    assert_eq!(parts(&forged).0, Some(hero.id()));
    assert_eq!(parts(&forged).1, Some(hero.gen().id() as u32));
    assert_ne!(parts(&forged).0, Some(ally.id()));
    let mut half = PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse {
        item_slot: 0, target_pos: None, target_entity: None, point_explicit: None,
        target_generation: None, owner_entity: Some(hero.id()), owner_generation: None,
    })) };
    assert!(!seal_item_use_owner(&w, 1, &mut half));
    assert_eq!(parts(&half), (Some(hero.id()), None, None, None));

    let mut stolen = ItemUse {
        item_slot: 0, target_pos: None, target_entity: Some(enemy.id()), point_explicit: None,
        target_generation: Some(enemy.gen().id() as u32), owner_entity: Some(999),
        owner_generation: Some(u32::MAX),
    };
    write_local_item_actor(&mut stolen, ally);
    assert_eq!(stolen.owner_entity, Some(ally.id()));
    assert_eq!(stolen.owner_generation, Some(ally.gen().id() as u32));
    assert_ne!(stolen.owner_generation, Some(u32::MAX), "filtered translation uses the local generation");
    assert!(canonical_item_owner(&w, 1, &stolen).is_none());
    assert_eq!(canonical_item_owner(&w, 3, &stolen), Some(canonical_entity_id(ally)));
    w.write_resource::<PendingItemUseQueue>().requests.push(PendingItemAction::Use(
        pending_item_use_from_command(1, &stolen),
    ));
    bind_queued_item_lives(&w);
    match &w.read_resource::<PendingItemUseQueue>().requests[0] {
        PendingItemAction::Use(request) => {
            assert_eq!(request.owner_entity, Some(ally));
            assert_ne!(request.owner_entity, Some(hero));
            assert!(request.lives_bound);
        }
        other => panic!("expected the stamped ally, got {other:?}"),
    }
    drain_pending_item_uses(&mut w);
    unchanged(&w);

    let mut partial = stolen;
    partial.owner_generation = None;
    w.write_resource::<PendingItemUseQueue>().requests.push(PendingItemAction::Use(
        pending_item_use_from_command(1, &partial),
    ));
    bind_queued_item_lives(&w);
    match &w.read_resource::<PendingItemUseQueue>().requests[0] {
        PendingItemAction::Use(request) => assert_eq!(request.owner_entity, None),
        other => panic!("a half owner pair is not the current hero, got {other:?}"),
    }
    drain_pending_item_uses(&mut w);
    unchanged(&w);

    let mut accepted = ItemUse {
        item_slot: 0, target_pos: None, target_entity: Some(enemy.id()), point_explicit: None,
        target_generation: Some(enemy.gen().id() as u32), owner_entity: None, owner_generation: None,
    };
    write_local_item_actor(&mut accepted, hero);
    w.write_resource::<PendingItemUseQueue>().requests.push(PendingItemAction::Use(
        pending_item_use_from_command(1, &accepted),
    ));
    w.write_storage::<PlayerOwner>().get_mut(hero).unwrap().player_id = 99;
    w.write_storage::<PlayerOwner>().get_mut(ally).unwrap().player_id = 1;
    bind_queued_item_lives(&w);
    match &w.read_resource::<PendingItemUseQueue>().requests[0] {
        PendingItemAction::Use(request) => {
            assert_eq!(request.owner_entity, Some(hero));
            assert_ne!(request.owner_entity, Some(ally));
        }
        other => panic!("reassignment must not retarget the queued owner, got {other:?}"),
    }
    drain_pending_item_uses(&mut w);
    assert_eq!(cd(&w, ally), 0.0, "player 1's replacement hero must not spend the queued hex");
    assert_eq!(cd(&w, hero), 0.0);
    assert_eq!(enemy_hp(&w), before_hp);
    assert_eq!(gold(&w, hero), before_gold);
    assert_eq!(gold(&w, ally), ally_gold);
    assert_eq!(command(&w, hero), before_command);
    assert_eq!(command(&w, ally), Some(HeroCommand::HoldPosition));
    w.write_storage::<PlayerOwner>().get_mut(hero).unwrap().player_id = 1;
    w.write_storage::<PlayerOwner>().get_mut(ally).unwrap().player_id = 3;

    let mut stamped = PlayerInput { action: Some(PlayerInputEnum::ItemUse(accepted)) };
    assert!(seal_item_use_owner(&w, 1, &mut stamped));
    assert_eq!(parts(&stamped).0, Some(hero.id()));
    assert_eq!(parts(&stamped).1, Some(hero.gen().id() as u32));
    let stamped_item = match &stamped.action {
        Some(PlayerInputEnum::ItemUse(value)) => *value,
        _ => unreachable!(),
    };
    w.write_resource::<PendingItemUseQueue>().requests.push(PendingItemAction::Use(
        pending_item_use_from_command(1, &stamped_item),
    ));
    let old_gen = hero.gen();
    let hero_life = w.read_storage::<Hero>().get(hero).unwrap().clone();
    let inventory = w.read_storage::<Inventory>().get(hero).unwrap().clone();
    let property = w.read_storage::<CProperty>().get(hero).unwrap().clone();
    let held_gold = w.read_storage::<Gold>().get(hero).unwrap().0;
    assert!(w.delete_entity(hero).is_ok());
    w.maintain();
    let replaced = w.create_entity()
        .with(hero_life)
        .with(PlayerOwner::new(1))
        .with(property)
        .with(Gold(held_gold))
        .with(inventory)
        .with(Pos(sim_point(0, 7000)))
        .with(Faction::new(FactionType::Player, 1))
        .with(VisionSource { team: 1, radius: Fixed64::from_i32(1000), detection_level: 0 })
        .build();
    set_hold(&mut w, replaced);
    assert_eq!(replaced.id(), hero.id(), "specs must hand the deleted owner index back");
    assert_ne!(replaced.gen(), old_gen);
    assert!(!seal_item_use_owner(&w, 1, &mut stamped));
    assert_eq!(parts(&stamped).1, Some(old_gen.id() as u32));
    assert_ne!(parts(&stamped).1, Some(replaced.gen().id() as u32));
    let mut legacy = PlayerInput { action: Some(PlayerInputEnum::ItemUse(ItemUse {
        item_slot: 0, target_pos: None, target_entity: Some(enemy.id()), point_explicit: None,
        target_generation: None, owner_entity: None, owner_generation: None,
    })) };
    assert!(seal_item_use_owner(&w, 1, &mut legacy));
    assert_eq!(parts(&legacy).0, Some(replaced.id()));
    assert_eq!(parts(&legacy).1, Some(replaced.gen().id() as u32));
    assert_ne!(parts(&legacy).1, Some(old_gen.id() as u32));
    bind_queued_item_lives(&w);
    match &w.read_resource::<PendingItemUseQueue>().requests[0] {
        PendingItemAction::Use(request) => {
            assert_eq!(request.owner_entity.map(|entity| (entity.id(), entity.gen().id())), Some((hero.id(), old_gen.id())));
            assert_ne!(request.owner_entity, Some(replaced));
        }
        other => panic!("the first bind must keep the accepted life, got {other:?}"),
    }
    drain_pending_item_uses(&mut w);
    assert_eq!(enemy_hp(&w), before_hp);
    assert_eq!(gold(&w, replaced), held_gold);
    assert_eq!(cd(&w, replaced), 0.0);
    assert_eq!(command(&w, replaced), Some(HeroCommand::HoldPosition));
    assert!(!w.entities().is_alive(hero));
}

#[test]
fn item_target_fixed_damage_keeps_fractional_and_large_raw_values() {
    let (mut w, mut driver) = world(fast_config(), SimulationTickProfile::Production60Hz);
    driver.step(&mut w, []).unwrap();
    let [hero, enemy] = hero_pair(&w);
    w.write_storage::<Gold>().get_mut(hero).unwrap().0 = 5000;
    driver.step(&mut w, [(1, PlayerInput { action: Some(PlayerInputEnum::ItemBuy(ItemBuy { item_id: "moba_training_flare".into() })) })]).unwrap();
    place_hero(&mut w, hero, 0, 7000);
    place_hero(&mut w, enemy, 0, 7000);
    set_hold(&mut w, hero);
    disarm_attack(&mut w, hero);
    disarm_attack(&mut w, enemy);
    w.write_storage::<CProperty>().get_mut(enemy).unwrap().hp = Fixed64::from_i32(20_000);
    w.write_storage::<CProperty>().get_mut(enemy).unwrap().mhp = Fixed64::from_i32(20_000);
    let override_flare = |w: &mut World, damage: Fixed64, radius: Fixed64| {
        w.insert(ItemRegistry::from_configs(vec![ItemConfig {
            id: "moba_training_flare".into(), name: "Precision flare".into(), cost: 450,
            bonus: ItemBonus::default(), active: Some(ActiveEffect::PointNova { damage, radius }),
            cooldown: 8.0, recipe: vec![],
        }]));
    };
    let damage = Fixed64::from_raw((1i64 << 24) + 1);
    let mirrored = Fixed64::from_raw((f64::from(damage.to_f32_for_render()) * 1024.0).round() as i64);
    assert_ne!(damage, mirrored);
    override_flare(&mut w, damage, Fixed64::from_i32(80));
    let before = w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    let gold = w.read_storage::<Gold>().get(hero).unwrap().0;
    assert!(handle_item_use_from_input(&mut w, 0, Some(sim_point(0, 7000)), None, 1, true).is_ok());
    assert_eq!(before - w.read_storage::<CProperty>().get(enemy).unwrap().hp, damage);
    assert_ne!(before - w.read_storage::<CProperty>().get(enemy).unwrap().hp, mirrored);
    assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 8.0);
    assert_eq!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active, Some(HeroCommand::HoldPosition));
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 0.0;
    override_flare(&mut w, Fixed64::from_raw(1), Fixed64::from_i32(80));
    let before_fraction = w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    assert!(handle_item_use_from_input(&mut w, 0, Some(sim_point(0, 7000)), None, 1, true).is_ok());
    assert_eq!(before_fraction - w.read_storage::<CProperty>().get(enemy).unwrap().hp, Fixed64::from_raw(1));
    w.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 0.0;
    let settled = w.read_storage::<CProperty>().get(enemy).unwrap().hp;
    for (amount, radius) in [
        (Fixed64::ZERO, Fixed64::from_i32(80)),
        (Fixed64::from_raw(-1), Fixed64::from_i32(80)),
        (Fixed64::from_raw(1_000_000 * 1024 + 1), Fixed64::from_i32(80)),
        (damage, Fixed64::ZERO),
    ] {
        override_flare(&mut w, amount, radius);
        assert!(handle_item_use_from_input(&mut w, 0, Some(sim_point(0, 7000)), None, 1, true).is_err());
        assert_eq!(w.read_storage::<CProperty>().get(enemy).unwrap().hp, settled);
        assert_eq!(w.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 0.0);
        assert_eq!(w.read_storage::<Gold>().get(hero).unwrap().0, gold);
        assert_eq!(w.read_storage::<HeroCommandQueue>().get(hero).unwrap().active, Some(HeroCommand::HoldPosition));
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
    // This covers input/replay lifecycle, not Push-versus-Guard balance.
    assert!(matches!(phase, MobaMatchPhase::Finished { winner: Some(0|1), .. }),
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
    let advance = |authority: &mut World, driver: &mut SimulationDriver, inputs: Vec<(u32, PlayerInput)>, replicas: &mut [(u32, SelectiveReplicaRuntime, SpecsDisclosedWorldStepper)]| {
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
        for (team, replica, stepper) in replicas.iter_mut() {
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
            let undo_facts: Vec<_> = frame.step.as_ref().unwrap().public_events.iter()
                .filter(|event| event.event_kind == FactKind::OwnerShopUndo as u32)
                .collect();
            assert!(!undo_facts.is_empty(), "shop team {team} tick {} dropped owner undo", driver.tick());
            assert!(undo_facts.iter().all(|event| {
                event.subject.is_none()
                    && omoba_core::runtime::shop_undo_wire::OwnerShopUndoProjection::decode(&event.sanitized_payload)
                        .is_some_and(|projection| projection.player_id == *team)
            }), "other team's owner undo leaked");
            let current = economy.last().unwrap();
            if let Some(hero) = authority.read_resource::<MobaMatch>().heroes[(*team - 1) as usize].entity {
                assert_eq!(current.economy.gold, authority.read_storage::<Gold>().get(hero).unwrap().0);
                let (_, inventory, _) = current.economy.components().unwrap();
                assert_eq!(serde_json::to_value(&inventory).unwrap(),
                    serde_json::to_value(authority.read_storage::<Inventory>().get(hero).unwrap()).unwrap());
            } else if *team == 2 {
                assert_eq!(current.economy.gold, 550);
                assert_eq!(current.economy.item_ids[1], 3);
                assert!(current.shop_available);
                assert_eq!(current.shop_reason, omoba_core::runtime::native::economy_projection::SHOP_REASON_AVAILABLE);
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
    ] { advance(&mut authority, &mut driver, inputs, &mut replicas); }
    assert_eq!(authority.read_storage::<Gold>().get(heroes[0]).unwrap().0, 525);
    assert_eq!(authority.read_storage::<Gold>().get(heroes[1]).unwrap().0, 550);
    // Cover a fresh filtered bootstrap/reconnect after settled transactions.
    let current = authority.write_resource::<TeamProjectionRuntime>()
        .build_team_bootstraps(driver.tick() + 1, 15, fast_config().seed);
    for (team, replica, _) in &replicas {
        let fresh = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&current[team], allow.clone(), BTreeSet::new()).unwrap();
        assert_private(fresh.world(), *team);
        assert_eq!(replica.canonical_team_hash(), fresh.canonical_team_hash(), "shop reconnect team {team}");
    }
    // Keep the boots across death and the new identity reveal. Increase all
    // remaining allied vision so the opponent stays visible while dead.
    for vision in (&mut authority.write_storage::<VisionSource>()).join() { vision.radius = Fixed64::from_i32(5000); }
    let speed_before = authority.read_storage::<CProperty>().get(heroes[1]).unwrap().msd;
    let pos = authority.read_storage::<Pos>().get(heroes[1]).unwrap().0;
    authority.write_resource::<Vec<Outcome>>().push(Outcome::Death { pos, ent: heroes[1] });
    for _ in 0..40 { advance(&mut authority, &mut driver, vec![], &mut replicas); }
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
    // Full authored maps contain all lanes, tower layers and base structures.
    // Their completion budget must cover normal progression without the old
    // generic Unit XP being paid a second time. Keep every lifecycle/hash
    // assertion and a finite, tick-rate-independent 20-minute game-time bound.
    let limit=if map.is_some(){limit.max(fps as usize*20*60)}else{limit};
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
    for step in 0..limit {
        if step%600==0 {
            let state=authority.read_resource::<MobaMatch>();
            println!("lifecycle progress map={map:?} seed={seed} tick={} phase={:?} towers={} deaths={:?}",
                driver.tick(),state.phase,state.lane_tower_layers.iter().flatten().flatten().filter(|t|t.is_some()).count(),
                state.heroes.iter().map(|h|h.deaths).collect::<Vec<_>>());
            if map.is_some() {
                for slot in &state.heroes { if let Some(entity)=slot.entity {
                    println!("lifecycle owner={} pos={:?} hp={:?} command={:?}",slot.player_id,
                        authority.read_storage::<Pos>().get(entity),authority.read_storage::<CProperty>().get(entity),
                        authority.read_storage::<HeroCommandQueue>().get(entity));
                } }
            }
        }
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
                            target: Some({let target=authority.read_storage::<Pos>().get(state.bases[1-side].unwrap()).unwrap().0;
                                Vec2I {x:target.x.raw() as i32,y:target.y.raw() as i32}}),
                            queued: false,
                        })),
                    }))
                }).collect()
            } else {
                let defender=state.config.players[1];
                let defender_alive=state.heroes[1].entity.is_some();
                let phase=state.phase;
                let withdraw=state.bases[1].and_then(|base|authority.read_storage::<Pos>().get(base).map(|p|p.0))
                    .unwrap_or(omoba_sim::Vec2::ZERO)+omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(3000));
                drop(state);
                let mut inputs=single_lane_bot_inputs(&authority,[SingleLaneBotPolicy::Push,SingleLaneBotPolicy::Guard]);
                // This regression tests progression/respawn/projection/finish,
                // not competitive bot balance. XP enables sustainable defence;
                // withdraw the defender using only the ordinary input path.
                inputs.retain(|(player,_)| *player != defender);
                if phase == MobaMatchPhase::Playing && defender_alive {
                    inputs.push((defender,PlayerInput {action:Some(PlayerInputEnum::MoveTo(MoveTo {
                        target:Some(Vec2I {x:withdraw.x.raw() as i32,y:withdraw.y.raw() as i32}),queued:false,
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

#[test]
fn objective_comeback_60hz_actual_skill_tower_wallet_public_status_pause_and_end() {
    let rules=ObjectiveComebackRules {starts_seconds:1,activate_gap:300,release_gap:100,
        tower_gold_per_player:100,epic_gold_per_player:125,activation_pool:200};
    let (mut w,mut driver)=world(SingleLaneConfig {objective_comeback_rules:rules,
        wave_interval:Fixed64::from_i32(10_000),respawn_delay:Fixed64::from_i32(100),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
    let origin=omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(isolated_skill_origin_y()));
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
    set_hold(&mut w,caster);set_hold(&mut w,enemy);disarm_attack(&mut w,caster);disarm_attack(&mut w,enemy);
    {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(enemy).unwrap();p.hp=Fixed64::from_i32(40);p.def_magic=Fixed64::ZERO;}
    driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:0,target_entity:Some(enemy.id()),target_pos:None}))})]).unwrap();
    driver.step(&mut w,[]).unwrap();
    assert_eq!(w.read_resource::<MobaMatch>().team_earned_gold(),[300,0]);
    w.write_storage::<Gold>().get_mut(caster).unwrap().0=0;
    for _ in 0..70 {driver.step(&mut w,[]).unwrap();}
    assert_eq!(w.read_resource::<MobaMatch>().objective_comeback(),Some((1,200)));
    // Respawn through the production lifecycle; the earned ledger survives.
    {let elapsed=w.read_resource::<MobaMatch>().elapsed;w.write_resource::<MobaMatch>().heroes[1].respawn_at=Some(elapsed);}
    driver.step(&mut w,[]).unwrap();let enemy=w.read_resource::<MobaMatch>().heroes[1].entity.unwrap();
    assert!(w.read_resource::<BuffStore>().has(enemy,"objective_comeback"));
    let snapshot=extract_snapshot(&mut w,driver.tick() as u32,std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),std::sync::Arc::new(Vec::new()),Vec::new(),Vec::new());
    let status=snapshot.entities.iter().find(|row|row.entity_id==enemy.id()).unwrap().buffs.iter().find(|buff|buff.buff_id=="objective_comeback").unwrap();
    assert_eq!(status.remaining_secs,-1.0,"public snapshot retains authority permanent eligibility");
    let tower=w.read_resource::<MobaMatch>().towers[0].unwrap();
    {let mut props=w.write_storage::<CProperty>();let p=props.get_mut(tower).unwrap();p.hp=Fixed64::from_i32(40);p.def_magic=Fixed64::ZERO;}
    // Skill damage is actual generated content. Relocate away from lane combat.
    w.write_storage::<Pos>().get_mut(tower).unwrap().0=origin;
    w.write_storage::<Pos>().get_mut(enemy).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(200),Fixed64::ZERO);
    w.write_storage::<Pos>().get_mut(caster).unwrap().0=origin+omoba_sim::Vec2::new(Fixed64::from_i32(2000),Fixed64::ZERO);
    disarm_attack(&mut w,enemy);set_hold(&mut w,enemy);
    let before=w.read_storage::<Gold>().get(enemy).unwrap().0;
    driver.step(&mut w,[(2,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:0,target_entity:Some(tower.id()),target_pos:None}))})]).unwrap();
    driver.step(&mut w,[]).unwrap();
    assert_eq!(w.read_storage::<Gold>().get(enemy).unwrap().0,before+100);
    assert_eq!(w.read_resource::<MobaMatch>().team_earned_gold(),[300,100]);
    assert_eq!(w.read_resource::<MobaMatch>().objective_comeback(),Some((1,100)));
    let before=w.read_resource::<MobaMatch>().team_earned_gold();w.write_resource::<GamePause>().is_paused=true;
    for _ in 0..10 {driver.step(&mut w,[]).unwrap();}
    assert_eq!(w.read_resource::<MobaMatch>().team_earned_gold(),before);
    assert!(w.read_resource::<BuffStore>().has(enemy,"objective_comeback"));w.write_resource::<GamePause>().is_paused=false;
    w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Finished {winner:None,tick:999};driver.step(&mut w,[]).unwrap();
    assert!(!w.read_resource::<BuffStore>().has(enemy,"objective_comeback"));
}

#[test]
fn objective_notice_60hz_generated_tower_and_epic_casts_public_queue_pause_and_terminal() {
    let (mut w,mut driver)=world(SingleLaneConfig {map_id:Some("three_lane_training".into()),
        wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [blue,red]=hero_pair(&w);
    let (tower,epic)={let s=w.read_resource::<MobaMatch>();(s.towers[1].unwrap(),
        s.jungle_camps.iter().find(|c|c.definition.objective_rewards().next().is_some()).unwrap().entity.unwrap())};
    let y=isolated_skill_origin_y();
    for (entity,x) in [(blue,0),(red,400),(tower,200),(epic,600)] {place_hero(&mut w,entity,x,y);}
    for entity in [blue,red] {set_hold(&mut w,entity);disarm_attack(&mut w,entity);}
    {let mut props=w.write_storage::<CProperty>();for target in [tower,epic] {let p=props.get_mut(target).unwrap();p.hp=Fixed64::from_i32(40);p.def_magic=Fixed64::ZERO;}}
    let read=|result:&SimulationTickResult|->[i64;4] {
        [OBJECTIVE_TEAM_METRIC_ID,OBJECTIVE_KIND_METRIC_ID,OBJECTIVE_SEQUENCE_METRIC_ID,OBJECTIVE_REMAINING_METRIC_ID].map(|id|
            result.facts.iter().find_map(|f|match &f.fact {ObservableFact::Hud {team:0,metric_id,value} if *metric_id==id=>Some(*value),_=>None}).expect("complete public objective tuple"))
    };
    let mut result=driver.step(&mut w,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {ability_index:0,target_entity:Some(tower.id()),target_pos:None}))}),
        (2,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {ability_index:0,target_entity:Some(epic.id()),target_pos:None}))})]).unwrap();
    // Camp death commits in the next death phase. Canonical tower identity comes first.
    result=driver.step(&mut w,[]).unwrap();assert_eq!(read(&result)[..3],[1,1,1]);
    w.write_resource::<GamePause>().is_paused=true;let frozen=read(&result);
    for _ in 0..10 {result=driver.step(&mut w,[]).unwrap();assert_eq!(read(&result),frozen);}
    w.write_resource::<GamePause>().is_paused=false;
    for _ in 0..270 {result=driver.step(&mut w,[]).unwrap();if read(&result)[2]==2 {break;}}
    assert_eq!(read(&result)[..3],[2,4,2]);assert!((1..=4096).contains(&read(&result)[3]));
    for _ in 0..270 {result=driver.step(&mut w,[]).unwrap();}
    assert_eq!(read(&result),[0;4]);
    w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Finished {winner:None,tick:999};
    result=driver.step(&mut w,[]).unwrap();assert_eq!(read(&result),[0;4]);
}

#[test]
fn cast_preparation_public_status_formal_60hz_cancels_freezes_and_releases_once() {
    let (mut w,mut driver)=world(SingleLaneConfig {heroes:["saika_magoichi".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
    place_hero(&mut w,caster,0,7000);place_hero(&mut w,enemy,4000,7000);
    w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool=Some(omoba_core::runtime::ability_runtime::ManaPool::new(Fixed64::from_i32(200),Fixed64::from_i32(200)).unwrap());
    let cast=||[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {ability_index:1,target_entity:None,
        target_pos:Some(Vec2I{x:0,y:7000*1024})}))})];
    let status=|w:&World|w.read_resource::<BuffStore>().get(caster,"cast_preparation").map(|b|(b.remaining.raw(),b.payload.clone()));
    driver.step(&mut w,cast()).unwrap();let first=status(&w).expect("public first preparation");
    assert_eq!(first.0,512);assert_eq!(first.1["cast_preparation"]["total_raw"],512);assert_eq!(first.1["cast_preparation"]["instance"],"1");
    w.write_resource::<GamePause>().is_paused=true;for _ in 0..10 {driver.step(&mut w,[]).unwrap();}assert_eq!(status(&w),Some(first));
    w.write_resource::<GamePause>().is_paused=false;
    driver.step(&mut w,[(1,PlayerInput{action:Some(PlayerInputEnum::Stop(Default::default()))})]).unwrap();assert!(status(&w).is_none());
    driver.step(&mut w,cast()).unwrap();assert_eq!(status(&w).unwrap().1["cast_preparation"]["instance"],"2");
    driver.step(&mut w,[]).unwrap();assert!(status(&w).unwrap().0<512);
    w.write_resource::<BuffStore>().add(caster,"stun",Fixed64::ONE,serde_json::json!({}));driver.step(&mut w,[]).unwrap();assert!(status(&w).is_none());
    assert_eq!(w.read_storage::<SummonedUnit>().join().count(),0);
    assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current(),Fixed64::from_i32(200));
    w.write_resource::<BuffStore>().remove(caster,"stun");driver.step(&mut w,cast()).unwrap();
    for _ in 0..30 {driver.step(&mut w,[]).unwrap();}
    assert!(status(&w).is_none());assert_eq!(w.read_storage::<SummonedUnit>().join().count(),2);
    assert!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("saika_reinforcements"));
    for _ in 0..10 {driver.step(&mut w,[]).unwrap();}assert_eq!(w.read_storage::<SummonedUnit>().join().count(),2);
}

#[test]
fn cast_damage_interruption_formal_60hz_distinguishes_hp_loss_shields_and_release() {
    for mode in 0..7 {
        let (mut w,mut driver)=world(SingleLaneConfig {heroes:["saika_magoichi".into(),"training_luminary".into()],
            wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
        driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);
        place_hero(&mut w,caster,0,7000);place_hero(&mut w,enemy,4000,7000);
        w.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool=Some(omoba_core::runtime::ability_runtime::ManaPool::new(Fixed64::from_i32(200),Fixed64::from_i32(200)).unwrap());
        driver.step(&mut w,[(1,PlayerInput{action:Some(PlayerInputEnum::CastAbility(CastAbility{ability_index:1,target_entity:None,target_pos:Some(Vec2I{x:0,y:7000*1024})}))})]).unwrap();
        assert!(w.read_resource::<BuffStore>().get(caster,"cast_preparation").is_some());
        if matches!(mode,2|3|6) {w.write_resource::<BuffStore>().grant_damage_shield(caster,Fixed64::from_i32(if mode==6 {5}else{100}),Fixed64::from_i32(5));}
        let hp=w.read_storage::<CProperty>().get(caster).unwrap().hp;
        let damage=||Outcome::Damage {pos:omoba_sim::Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,real:Fixed64::from_i32(if mode==4 {0}else{10}),source:enemy,target:caster,damage_profile:0,predeclared:false};
        w.write_resource::<Vec<Outcome>>().push(match mode {
            1|3=>Outcome::ScriptDirectDamage {target:caster,amount:Fixed64::from_i32(10)},
            5=>Outcome::Heal {pos:omoba_sim::Vec2::ZERO,target:caster,amount:Fixed64::from_i32(10)},
            _=>damage(),
        });
        driver.step(&mut w,[]).unwrap();let interrupted=matches!(mode,0|1|6);
        assert_eq!(w.read_storage::<CProperty>().get(caster).unwrap().hp<hp,interrupted,"mode {mode}: actual HP debit");
        assert_eq!(w.read_resource::<BuffStore>().get(caster,"cast_preparation").is_none(),interrupted,"mode {mode}: preparation");
        assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current(),Fixed64::from_i32(200));
        for _ in 0..35 {driver.step(&mut w,[]).unwrap();}
        assert_eq!(w.read_storage::<SummonedUnit>().join().count(),if interrupted{0}else{2},"mode {mode}: release exactly once");
        assert_eq!(w.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("saika_reinforcements"),!interrupted);
        w.write_resource::<Vec<Outcome>>().push(damage());driver.step(&mut w,[]).unwrap();
        assert_eq!(w.read_storage::<SummonedUnit>().join().count(),if interrupted{0}else{2},"damage after release cannot roll back summons");
    }
}

#[test]
fn summon_presentation_formal_60hz_keeps_explicit_kind_owner_and_retirement() {
    let (mut w,mut driver)=world(SingleLaneConfig{heroes:["saika_magoichi".into(),"training_luminary".into()],wave_interval:Fixed64::from_i32(10_000),..fast_config()},SimulationTickProfile::Production60Hz);
    driver.step(&mut w,[]).unwrap();let [caster,enemy]=hero_pair(&w);place_hero(&mut w,caster,0,7000);place_hero(&mut w,enemy,4000,7000);
    driver.step(&mut w,[(1,PlayerInput{action:Some(PlayerInputEnum::CastAbility(CastAbility{ability_index:1,target_entity:None,target_pos:Some(Vec2I{x:0,y:7000*1024})}))})]).unwrap();
    for _ in 0..30 {driver.step(&mut w,[]).unwrap();}
    let summons:Vec<_>=(&w.entities(),&w.read_storage::<SummonedUnit>()).join().map(|(e,_)|e).collect();assert_eq!(summons.len(),2);
    let snapshot=omoba_core::runtime::extract_data_for_render(&mut w,driver.tick() as u32,vec![],vec![]);
    for entity in &summons {
        let row=snapshot.entities.iter().find(|r|r.entity_id==entity.id()).unwrap();assert_eq!(row.kind,omoba_core::runtime::EntityKind::Summon);assert_eq!(row.unit_id,"saika_gunner");assert_eq!(row.owner_player_id,Some(1));

    }
    place_hero(&mut w,enemy,350,7000);
    run_committed_visibility_wave_b(&mut w,driver.tick(),0);
    let mut fired=false;for _ in 0..100 {driver.step(&mut w,[]).unwrap();
        if w.read_storage::<Projectile>().join().any(|p|summons.contains(&p.owner)) {
            let combat=omoba_core::runtime::extract_data_for_render(&mut w,driver.tick() as u32,vec![],vec![]);
            assert!(combat.attack_phase_fx.iter().any(|fx|summons.iter().any(|e|e.id()==fx.entity_id && e.gen().id() as u32==fx.entity_gen) && fx.backswing_ms>0),"actual script attack cue must reach public snapshot");fired=true;break;
        }
    }assert!(fired,"actual summon script produces both projectile and public animation state");
    for entity in &summons {w.write_storage::<SummonedUnit>().get_mut(*entity).unwrap().time_remaining=Some(Fixed64::from_raw(1));}
    w.write_resource::<GamePause>().is_paused=true;driver.step(&mut w,[]).unwrap();assert!(summons.iter().all(|e|w.entities().is_alive(*e)));
    w.write_resource::<GamePause>().is_paused=false;driver.step(&mut w,[]).unwrap();assert!(summons.iter().all(|e|!w.entities().is_alive(*e)));
    let retired=omoba_core::runtime::extract_data_for_render(&mut w,driver.tick() as u32,vec![],vec![]);
    assert!(!retired.entities.iter().any(|r|summons.iter().any(|e|e.id()==r.entity_id && e.gen().id() as u32==r.entity_gen)));
}
