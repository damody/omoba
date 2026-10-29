//! Serial raw settlement and read-only conditional leech accounting.
use omoba_sim::Fixed64;
use specs::{World,WorldExt,Entity};
use crate::runtime::comp::*;
use crate::runtime::ability_runtime::UnitStats;

/// Read-only serial wrapper around the shared stat policy.
pub(super) fn ordinary_target(world:&World,source:Entity,target:Entity)->bool {
    let factions=world.read_storage::<Faction>();
    UnitStats::ordinary_leech_target(factions.get(source),factions.get(target),world.read_storage::<IsBuilding>().get(target).is_some())
}

pub(super) struct LeechCommit {
    source:Entity,target:Entity,before:Fixed64,components:[Fixed64;3],rates:[Fixed64;2],
}
impl LeechCommit {
    pub(super) fn capture(world:&World,damage:&Outcome,physical:Fixed64,magical:Fixed64,removed:&[Entity])->Option<Self> {
        let Outcome::Damage {source,target,phys,magi,real,..}=damage else {return None;};
        if source==target || removed.contains(source) || removed.contains(target)
            || !world.entities().is_alive(*source) || !world.entities().is_alive(*target)
            || [*phys,*magi,*real,physical,magical].iter().any(|v|*v<Fixed64::ZERO)
            || world.read_storage::<Creep>().get(*target).is_some_and(|c|c.td_layer.is_some()) {return None;}
        let properties=world.read_storage::<CProperty>();
        if properties.get(*source)?.hp<=Fixed64::ZERO {return None;}
        let before=properties.get(*target)?.hp;if before<=Fixed64::ZERO {return None;}
        Some(Self {source:*source,target:*target,before,components:[*phys,*magi,*real],rates:[physical,magical]})
    }
    pub(super) fn healing(self,world:&World)->Option<(Entity,Fixed64)> {
        if !world.entities().is_alive(self.source) {return None;}
        let properties=world.read_storage::<CProperty>();
        if properties.get(self.source)?.hp<=Fixed64::ZERO {return None;}
        let after=properties.get(self.target)?.hp.max(Fixed64::ZERO);
        let loss=(self.before-after).clamp(Fixed64::ZERO,self.before);
        // One rational rounding. Global packet modifiers, shields and overkill
        // reduce each component proportionally; pure damage never earns leech.
        let total=self.components.iter().map(|v|i128::from(v.raw())).sum::<i128>();
        if total<=0 || loss<=Fixed64::ZERO {return None;}
        let weight=i128::from(self.components[0].raw()).checked_mul(i128::from(self.rates[0].raw()))?
            .checked_add(i128::from(self.components[1].raw()).checked_mul(i128::from(self.rates[1].raw()))?)?;
        let raw=i128::from(loss.raw()).checked_mul(weight)?/total.checked_mul(1024)?;
        let amount=Fixed64::from_raw(i64::try_from(raw).ok()?);
        (amount>Fixed64::ZERO).then_some((self.source,amount))
    }
}

pub(super) struct CriticalCommit {target:Entity,before:Fixed64}
impl CriticalCommit {
    pub(super) fn capture(world:&World,target:Entity,physical:Fixed64,pending:&[Entity])->Option<Self> {
        if physical<=Fixed64::ZERO || !world.entities().is_alive(target) || pending.contains(&target)
            || world.read_storage::<Creep>().get(target).is_some_and(|c|c.td_layer.is_some()) {return None;}
        let before=world.read_storage::<CProperty>().get(target)?.hp;
        (before>Fixed64::ZERO).then_some(Self{target,before})
    }
    pub(super) fn emit(self,world:&World,tick:u64,ordinal:u32) {
        let properties=world.read_storage::<CProperty>();
        let Some(after)=properties.get(self.target).map(|p|p.hp.max(Fixed64::ZERO))else{return;};
        if after>=self.before{return;}
        let target=crate::runtime::canonical_entity_id(self.target);
        let Some(facts)=world.try_fetch::<crate::runtime::ObservableFactBuffer>()else{return;};
        let _=facts.emit(crate::runtime::OrderedFact{
            key:crate::runtime::FactOrderingKey{tick,phase:crate::runtime::FactPhase::Step,canonical_source_order:target,local_ordinal:ordinal,fact_kind:crate::runtime::FactKind::CriticalHit},
            audience:crate::runtime::FactAudience::VisibilityPolicy(omb_script_abi::types::projection_policy_ids::PROJECTILE.to_owned()),
            fact:crate::runtime::ObservableFact::CriticalHit{target}});
    }
}

/// Block, armor and resistance of a raw packet (rules::damage::defenses, stage ③ of the
/// pipeline). `attack`: an ordinary attack's packet, which physical block applies to.
pub(super) fn settle(world:&World,target:Entity,phys:Fixed64,magi:Fixed64,real:Fixed64,profile:u32,attack:bool)->Option<DamageTypes> {
    crate::runtime::native::rules::damage::defenses(world,target,phys,magi,real,profile,attack)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::BuffStore;
    use specs::Builder;
    use omb_script_abi::types::DamageProfile;
    use omoba_sim::Vec2;
    fn fixture(armor:i32,resist:i32)->(World,Entity,Entity) {
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
        world.insert(crate::runtime::TowerTemplateRegistry::default());
        let source=world.create_entity().with(Faction::new(FactionType::Player,1)).build();
        let target=world.create_entity().with(Pos(Vec2::ZERO)).with(Faction::new(FactionType::Enemy,2))
            .with(CProperty{hp:Fixed64::from_i32(1000),mhp:Fixed64::from_i32(1000),msd:Fixed64::ZERO,
                def_physic:Fixed64::from_i32(armor),def_magic:Fixed64::from_i32(resist)}).build();
        (world,source,target)
    }
    fn raw(source:Entity,target:Entity,phys:i32,magi:i32,real:i32)->Outcome {
        Outcome::RawDamage{source,target,pos:Vec2::ZERO,phys:Fixed64::from_i32(phys),magi:Fixed64::from_i32(magi),
            real:Fixed64::from_i32(real),damage_profile:DamageProfile::NORMAL.bits(),origin:None}
    }
    fn flush(world:&mut World,rows:Vec<Outcome>) {
        world.write_resource::<Vec<Outcome>>().extend(rows);
        crate::runtime::process_outcomes(world,&mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
    }
    fn hp(world:&World,target:Entity)->Fixed64 {world.read_storage::<CProperty>().get(target).unwrap().hp}
    fn leech(source:Entity,target:Entity)->Outcome {
        Outcome::DamageWithLeech {damage:Box::new(Outcome::Damage {source,target,pos:Vec2::ZERO,
            phys:Fixed64::from_i32(60),magi:Fixed64::from_i32(30),real:Fixed64::from_i32(10),
            damage_profile:DamageProfile::NORMAL.bits(),predeclared:false}),
            physical_leech:Fixed64::from_raw(512),magical_leech:Fixed64::ONE}
    }
    fn leech_body(world:&mut World,source:Entity) {
        world.write_storage::<CProperty>().insert(source,CProperty {hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(1000),
            msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).unwrap();
    }
    #[test]
    fn leech_uses_actual_hp_loss_after_shields_overkill_packet_scaling_and_received_healing() {
        for mode in 0..6 {
            let (mut w,source,target)=fixture(0,0);leech_body(&mut w,source);
            if mode==0 {w.write_resource::<BuffStore>().grant_damage_shield(target,Fixed64::from_i32(20),Fixed64::ONE);}
            if mode==1 {w.write_storage::<CProperty>().get_mut(target).unwrap().hp=Fixed64::from_i32(25);}
            if matches!(mode,2|4) {w.write_resource::<BuffStore>().add(target,"reduce",Fixed64::ONE,serde_json::json!({"damage_taken_bonus":-512}));}
            if mode==3 {w.write_resource::<BuffStore>().grant_damage_shield(target,Fixed64::from_i32(200),Fixed64::ONE);}
            if mode==4 {w.write_resource::<BuffStore>().add(source,"cut",Fixed64::ONE,serde_json::json!({"heal_received_multiplier":512}));}
            if mode==5 {w.write_resource::<BuffStore>().add(target,"immune",Fixed64::ONE,serde_json::json!({"damage_taken_bonus":-1024}));}
            let packet=serde_json::from_str(&serde_json::to_string(&leech(source,target)).unwrap()).unwrap();
            flush(&mut w,vec![packet]);let expected=[48,15,30,0,15,0][mode];
            assert_eq!(hp(&w,source),Fixed64::from_i32(100+expected),"mode{mode}");
        }
    }
    #[test]
    fn leech_rejects_dead_reused_source_stale_target_and_nested_effects() {
        for mode in 0..4 {
            let (mut w,source,target)=fixture(0,0);leech_body(&mut w,source);let mut recipient=source;
            let mut packet=leech(source,target);
            let faction=w.read_storage::<Faction>().get(source).cloned();
            let Outcome::DamageWithLeech {damage,..}=&packet else {unreachable!()};
            assert!(super::super::cast_settlement::admit(&w,source,"probe",0,faction.as_ref(),None,None,&[(**damage).clone()],0));
            assert!(!super::super::cast_settlement::admit(&w,source,"probe",0,faction.as_ref(),None,None,&[packet.clone()],0),"host-only leech cannot enter a managed script cast");
            if mode==0 {w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::ZERO;}
            if mode==1 {w.delete_entity(source).unwrap();w.maintain();recipient=w.create_entity().build();assert_eq!(recipient.id(),source.id());assert_ne!(recipient,source);leech_body(&mut w,recipient);}
            if mode==2 {w.write_storage::<CProperty>().get_mut(target).unwrap().hp=Fixed64::ZERO;}
            if mode==3 {packet=Outcome::DamageWithLeech {damage:Box::new(Outcome::Heal {target:source,amount:Fixed64::from_i32(999),pos:Vec2::ZERO}),physical_leech:Fixed64::ONE,magical_leech:Fixed64::ZERO};}
            let before=hp(&w,recipient);flush(&mut w,vec![packet]);assert_eq!(hp(&w,recipient),before,"mode{mode}");
        }
    }
    #[test]
    fn leech_checked_weight_math_never_wraps_or_earns_from_pure_damage() {
        for pure in [false,true] {
            let (mut w,source,target)=fixture(0,0);leech_body(&mut w,source);
            let mut packet=leech(source,target);if let Outcome::DamageWithLeech {damage,physical_leech,magical_leech}=&mut packet {
                *physical_leech=Fixed64::from_raw(i64::MAX);*magical_leech=*physical_leech;
                if let Outcome::Damage {phys,magi,real,..}=&mut **damage {*phys=if pure {Fixed64::ZERO}else{Fixed64::from_raw(i64::MAX/2)};*magi=Fixed64::ZERO;*real=Fixed64::from_i32(100);}
            }
            let Outcome::DamageWithLeech {damage,physical_leech,magical_leech}=packet else {unreachable!()};
            let commit=LeechCommit::capture(&w,&damage,physical_leech,magical_leech,&[]).unwrap();
            w.write_storage::<CProperty>().get_mut(target).unwrap().hp=Fixed64::ZERO;
            assert!(commit.healing(&w).is_none());
        }
    }
    #[test]
    fn ordinary_projectile_leech_requires_explicit_exact_nonmissed_origin_and_live_source() {
        for mode in 0..14 {
            let (mut w,source,target)=fixture(0,0);leech_body(&mut w,source);
            let other=w.create_entity().build();leech_body(&mut w,other);
            if mode==8 {w.write_storage::<IsBuilding>().insert(target,IsBuilding).unwrap();}
            if mode==9 {*w.write_storage::<Faction>().get_mut(target).unwrap()=Faction::new(FactionType::Player,1);}
            if mode==10 {*w.write_storage::<Faction>().get_mut(target).unwrap()=Faction::new(FactionType::HostileNeutral,-1);}
            if mode==11 {*w.write_storage::<Faction>().get_mut(target).unwrap()=Faction::new(FactionType::Neutral,-1);}
            if mode==12 {w.write_storage::<Faction>().remove(target);}
            if mode==13 {w.write_storage::<Faction>().remove(source);}
            let mut origin=crate::runtime::moba_match::capture_projectile_origin(&w,source).unwrap();
            origin.normal_attack=mode!=1&&mode!=2;origin.script_attack=mode==2;
            origin.attack_lifesteal=Fixed64::from_raw(512);origin.attack_missed=mode==3;
            if mode==4 {origin.source=other;}
            if mode==5 {w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::ZERO;}
            let recipient=if mode==6 {w.delete_entity(source).unwrap();w.maintain();let e=w.create_entity().build();assert_eq!(e.id(),source.id());assert_ne!(e,source);leech_body(&mut w,e);e}else{source};
            if mode==7 {assert!(w.write_resource::<BuffStore>().grant_damage_shield(target,Fixed64::from_i32(100),Fixed64::ONE));}
            let before=hp(&w,recipient);
            let packet=Outcome::ProjectileDamage {origin,pos:Vec2::ZERO,phys:Fixed64::from_i32(60),magi:Fixed64::from_i32(30),real:Fixed64::from_i32(10),source,target,damage_profile:DamageProfile::NORMAL.bits(),predeclared:false};
            flush(&mut w,vec![packet]);assert_eq!(hp(&w,recipient),before+Fixed64::from_i32(if matches!(mode,0|2|10){30}else{0}),"mode{mode}");
            if mode>=8 {assert_eq!(hp(&w,target),Fixed64::from_i32(900),"shared leech policy does not invent damage admission");}
            assert_eq!(hp(&w,other),Fixed64::from_i32(100),"mismatched origin cannot redirect leech");
        }
        // A real primary's callback keeps attribution, but cannot carry its
        // leech entitlement into script-authored damage or another generation.
        let (mut w,source,target)=fixture(0,0);leech_body(&mut w,source);
        let mut origin=crate::runtime::moba_match::capture_projectile_origin(&w,source).unwrap();
        origin.normal_attack=true;origin.attack_lifesteal=Fixed64::from_raw(512);
        flush(&mut w,vec![Outcome::ProjectileContext {origin,outcome:Box::new(Outcome::ProjectileHit {source,target,kind_id:0,generation:0})}]);
        let events=w.write_resource::<crate::runtime::scripting::ScriptEventQueue>().drain();
        assert!(matches!(events.as_slice(),[crate::runtime::scripting::ScriptEvent::ProjectileHit {origin:Some(o),..}]
            if o.source==source && o.normal_attack && o.attack_lifesteal==Fixed64::ZERO));
    }
    #[test]
    fn raw_mixed_and_precomputed_packets_reach_the_same_hp_without_double_defenses() {
        for precomputed in [false,true] {
            let (mut world,source,target)=fixture(100,25);
            let hit=if precomputed {Outcome::Damage{source,target,pos:Vec2::ZERO,phys:Fixed64::from_i32(50),magi:Fixed64::from_i32(75),
                real:Fixed64::from_i32(20),damage_profile:DamageProfile::NORMAL.bits(),predeclared:false}}else{raw(source,target,100,100,20)};
            let hit:Outcome=serde_json::from_str(&serde_json::to_string(&hit).unwrap()).unwrap();
            flush(&mut world,vec![hit]);assert_eq!(hp(&world,target),Fixed64::from_i32(855));
        }
    }
    #[test]
    fn raw_reads_serial_defense_buff_before_one_packet_modifier_and_one_shield() {
        let (mut world,source,target)=fixture(0,0);
        world.write_resource::<BuffStore>().grant_damage_shield(target,Fixed64::from_i32(20),Fixed64::ONE);
        let buff=Outcome::AddBuff{target,buff_id:"defense_fixture".into(),duration:Fixed64::ONE,
            payload:serde_json::json!({"physical_armor_bonus":100*1024,"magical_resistance_bonus":50*1024,"damage_taken_bonus":-512})};
        let hit=raw(source,target,100,100,20);
        assert_eq!(hp(&world,target),Fixed64::from_i32(1000));flush(&mut world,vec![buff,hit]);
        assert_eq!(hp(&world,target),Fixed64::from_i32(960));
        assert_eq!(world.read_resource::<BuffStore>().shield_remaining(target),Fixed64::ZERO);
    }
    #[test]
    fn raw_uses_hero_property_then_unit_then_property_only_bases_and_existing_caps() {
        for kind in 0..3 {
            let (mut world,source,target)=fixture(100,25);
            if kind<2 {let mut unit=Unit::default();unit.base_armor=Fixed64::from_i32(300);unit.magic_resistance=Fixed64::from_i32(90);world.write_storage::<Unit>().insert(target,unit).unwrap();}
            if kind==0 {world.write_storage::<Hero>().insert(target,Hero::new("fixture".into(),"Fixture".into(),String::new())).unwrap();}
            flush(&mut world,vec![raw(source,target,100,100,20)]);
            assert_eq!(hp(&world,target),Fixed64::from_i32(if kind==1 {930}else{855}));
        }
    }
    #[test]
    fn invalid_raw_or_defenses_never_touch_hp_or_shield_and_pure_bypasses_defenses() {
        for case in 0..5 {
            let (mut world,source,target)=fixture(100,25);
            world.write_resource::<BuffStore>().grant_damage_shield(target,Fixed64::from_i32(10),Fixed64::ONE);
            let mut hit=raw(source,target,100,0,0);
            match case {
                0=>if let Outcome::RawDamage{phys,..}=&mut hit {*phys=Fixed64::from_i32(-1);},
                1=>if let Outcome::RawDamage{phys,magi,real,..}=&mut hit {*phys=Fixed64::from_raw(i64::MAX);*magi=*phys;*real=*phys;},
                2=>if let Outcome::RawDamage{damage_profile,..}=&mut hit {*damage_profile=1<<31;},
                _=>{world.write_resource::<BuffStore>().add(target,"invalid_defense",Fixed64::ONE,
                    serde_json::json!({"physical_armor_bonus":"invalid"}));},
            }
            if case==4 {hit=raw(source,target,0,0,100);}
            flush(&mut world,vec![hit]);
            assert_eq!(hp(&world,target),Fixed64::from_i32(if case==4 {910}else{1000}));
            assert_eq!(world.read_resource::<BuffStore>().shield_remaining(target),Fixed64::from_i32(if case==4 {0}else{10}));
        }
    }
    #[test]
    fn raw_projectile_context_keeps_retired_source_and_rejects_forged_origin() {
        for forged in [false,true] {
            let (mut world,source,target)=fixture(100,0);
            let other=world.create_entity().build();
            let origin=ProjectileOrigin{source:if forged {other}else{source},script_attack:false,attack_missed:false,normal_attack:false,attack_critical:false,attack_lifesteal:Fixed64::ZERO,
                launch_position:Some(Vec2::ZERO),credit_team:Some(1),credit_player:None,script:None};
            world.delete_entity(source).unwrap();world.maintain();
            let hit=Outcome::ProjectileContext{origin,outcome:Box::new(raw(source,target,100,0,0))};
            flush(&mut world,vec![hit]);assert_eq!(hp(&world,target),Fixed64::from_i32(if forged {1000}else{950}));
        }
    }
    #[test]
    fn raw_retired_target_in_same_batch_never_consumes_its_shield() {
        let (mut world,source,target)=fixture(100,0);
        world.write_resource::<BuffStore>().grant_damage_shield(target,Fixed64::from_i32(10),Fixed64::ONE);
        flush(&mut world,vec![Outcome::Death{pos:Vec2::ZERO,ent:target},raw(source,target,100,0,0)]);
        // The component entity is retired at the batch boundary; resource-backed
        // shield state must not be consumed by damage after its explicit Death.
        assert_eq!(world.read_resource::<BuffStore>().shield_remaining(target),Fixed64::from_i32(10));
    }
    #[test]
    fn raw_td_packets_keep_layer_immunity_and_skip_moba_base_defenses() {
        let (mut world,source,target)=fixture(1000,75);
        world.write_storage::<Creep>().insert(target,Creep{name:"td_btd_lead".into(),label:None,path:String::new(),pidx:0,
            path_remaining_distance:Fixed64::ZERO,block_tower:None,status:CreepStatus::Walk,
            td_layer:Some(TdLayerState{base_archetype:"lead".into(),current_layer:"lead".into(),properties:0,
                regrow_ceiling:"lead".into(),regrow_elapsed:Fixed64::ZERO,remaining_leak_value:1,spawn_lineage:1})}).unwrap();
        world.write_storage::<CProperty>().get_mut(target).unwrap().hp=Fixed64::ONE;
        let mut hit=raw(source,target,1,0,0);if let Outcome::RawDamage{damage_profile,..}=&mut hit {*damage_profile=DamageProfile::SHARP.bits();}
        assert_eq!(settle(&world,target,Fixed64::ONE,Fixed64::ZERO,Fixed64::ZERO,DamageProfile::SHARP.bits(),false).unwrap().physical,Fixed64::ONE);
        flush(&mut world,vec![hit]);assert_eq!(hp(&world,target),Fixed64::ONE);
        let mut hit=raw(source,target,0,0,1);if let Outcome::RawDamage{damage_profile,..}=&mut hit {*damage_profile=DamageProfile::TRUE.bits();}
        flush(&mut world,vec![hit]);assert!(world.read_storage::<Creep>().get(target).is_none_or(|c|c.td_layer.as_ref().is_none_or(|s|s.current_layer!="lead")));
    }
}
