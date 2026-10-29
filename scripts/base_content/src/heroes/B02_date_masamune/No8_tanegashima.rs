//! 種子島（H02，來源 B04T current，B04.txt:274-440、B04.lua:B04T_OnAttack:157-211）：
//! 限時換上鐵砲：射程 +range_bonus、基礎攻擊 +attack_bonus；普攻發射時 stun_chance 機率
//! 使目標暈眩 stun_time 秒（host 攻擊暈眩：發射時套用、不看命中、排除建築）。
//! 每次普攻發射（host 普攻發射 hook AbilityScript::on_attack_launch，帶當前 rank）沿施法者→目標方向
//! 射出一發直線穿透子彈（bullet_damage 魔法、bullet_distance、bullet_radius、bullet_speed，只打敵方英雄
//! 與一般單位；數值見 v3.lua），另兩次各 extra_shot_hits/extra_shot_outcomes 機率在 extra_delay_first／extra_delay_second 秒後從同一發射點與方向追加一發
//! （G-1 發射延遲）。隨機用 host 決定性 rand。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_core::runtime::native::{ability_projectile as host,attack_stun};
use omoba_template_ids::{ABILITY_B04_TANEGASHIMA,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};
use crate::self_transform::{Encoding,Field};

const ID:omoba_template_ids::AbilityId=ABILITY_B04_TANEGASHIMA;
pub(crate) const BUFF_ID:&str="b04_tanegashima";
const FIELDS:&[Field]=&[
    Field {key:"range_bonus",stat:StatKey::AttackRangeBonus,maximum:10_000,encoding:Encoding::Raw},
    Field {key:"attack_bonus",stat:StatKey::BaseAttackBonusDamage,maximum:1_000_000,encoding:Encoding::Raw},
    Field {key:"stun_chance",stat:StatKey::AttackStunChance,maximum:1,encoding:Encoding::Normalized},
    Field {key:"stun_time",stat:StatKey::AttackStunDuration,maximum:60,encoding:Encoding::Normalized},
];

fn extra(key:&str,rank:u8)->Option<Fixed64> {
    let content=active_ability_const(ID)?;
    let index=usize::from(rank.checked_sub(1)?);
    content.extras.iter().find(|(name,_)|*name==key)?.1.get(index).copied()
}

/// Rank-resolved bullet values (custom extras of B04T).
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) struct Bullets {
    pub damage:Fixed64,pub distance:Fixed64,pub radius:Fixed64,pub speed:Fixed64,pub lifetime:Fixed64,
    pub extra_hits:i64,pub extra_outcomes:i64,pub delays:[Fixed64;2],
}

pub(crate) fn bullets(rank:u8)->Option<Bullets> {
    let whole=|key:&str|extra(key,rank).filter(|v|v.raw()>0 && v.raw()%1024==0).map(|v|v.raw()/1024);
    let spec=Bullets {
        damage:extra("bullet_damage",rank)?,distance:extra("bullet_distance",rank)?,
        radius:extra("bullet_radius",rank)?,speed:extra("bullet_speed",rank)?,
        lifetime:extra("bullet_lifetime",rank)?,
        extra_hits:whole("extra_shot_hits")?,extra_outcomes:whole("extra_shot_outcomes")?,
        delays:[extra("extra_delay_first",rank)?,extra("extra_delay_second",rank)?],
    };
    (spec.extra_hits<=spec.extra_outcomes && spec.delays.iter().all(|d|*d>Fixed64::ZERO && *d<=host::MAX_LAUNCH_DELAY))
        .then_some(spec)
}

fn envelope(caster:EntityHandle,aim:omb_script_abi::types::Vec2,spec:&Bullets)->host::Spec {
    host::Spec {
        source_id:caster.id,source_generation:caster.gen,ability:ID.as_str().to_owned(),
        motion:host::Motion::Linear {aim},speed:spec.speed,radius:spec.radius,distance:spec.distance,
        lifetime:spec.lifetime,max_hits:host::MAX_HITS,wall:host::WallPolicy::Pass,
        target_lost:host::TargetLostPolicy::Expire,caster_death:host::CasterDeathPolicy::Continue,
        amount:spec.damage,kind:DamageKind::Magical,profile:DamageProfile::ENERGY.bits(),
        slow:None,control:None,dot:None,buildings:false,forced:None,
    }
}

/// B04T_OnAttack: one bullet now, plus each of two independent
/// `RandomInt(0,100) <= 25` rolls (extra_shot_hits / extra_shot_outcomes) delays another by extra_delay_first / extra_delay_second.
/// Returns how many bullets were launched.
/// Values follow the current rank (GetSpecialValueFor at attack time).
pub(crate) fn on_owner_attack_launch(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:EntityHandle,rank:u8)->usize {
    if !world.has_buff(caster,BUFF_ID.into()) {return 0;}
    let Some(spec)=bullets(rank) else {return 0;};
    let (Some(origin),Some(aim))=(world.get_pos(caster).into_option(),world.get_pos(target).into_option()) else {return 0;};
    if origin==aim {return 0;}
    let row=envelope(caster,aim,&spec);
    if !row.valid() {return 0;}
    let mut launched=0;
    let mut fire=|world:&mut GameWorldDyn<'_>,delay:Fixed64| {
        let payload=if delay>Fixed64::ZERO {host::with_delay(row.payload(),delay)} else {row.payload()};
        world.add_stat_buff(caster,host::BUFF.into(),spec.lifetime,payload.to_string().as_str().into());
        launched+=1;
    };
    fire(world,Fixed64::ZERO);
    for delay in spec.delays {
        // rand_unit is in [0, 1): floor(r * extra_shot_outcomes) < extra_shot_hits.
        let roll=world.rand_unit().raw().clamp(0,1023)*spec.extra_outcomes/1024;
        if roll<spec.extra_hits {fire(world,delay);}
    }
    launched
}

pub struct TanegashimaHandler;
impl AbilityScript for TanegashimaHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        let result=crate::self_transform::resolve_keyed(ID,level,json.as_str(),FIELDS,None,"duration",true)
            .and_then(|spec| {
                // B04.lua:171-175: the stun is rolled and applied when the attack launches,
                // hit or miss, and only on heroes and creeps (never buildings).
                let on_launch=extra("stun_on_launch",level).is_some_and(|v|v>Fixed64::ZERO);
                let skip=extra("stun_skip_buildings",level).is_some_and(|v|v>Fixed64::ZERO);
                // B04.txt ENEMIES_YES: the stun ignores magic immunity (content pierces_magic_immunity, E184).
                let pierce=omoba_core::runtime::native::magic_immunity::ability_pierces(ID.as_str());
                let spec=spec.with_flag(attack_stun::ON_LAUNCH_KEY,on_launch).with_flag(attack_stun::SKIP_BUILDINGS_KEY,skip)
                    .with_flag(omoba_core::runtime::native::magic_immunity::PIERCE_KEY,pierce);
                crate::self_transform::execute(world,caster,target,BUFF_ID,spec)
            });
        match result {Ok(())=>ROk(()),Err(error)=>RErr(error.into())}
    }
    fn on_attack_launch(&self,attacker:EntityHandle,target:EntityHandle,level:u8,world:&mut GameWorldDyn<'_>) {
        on_owner_attack_launch(world,attacker,target,level);
    }
}

pub fn b04_tanegashima_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    for field in FIELDS {modifiers.insert(field.stat.as_str().into(),extra_at_id_f32(ID,field.key,1));}
    build_ability_ffi(ID,TanegashimaHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,
        duration:Some(extra_at_id_f32(ID,"duration",1)),modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B02_date_masamune::h02_test_support::*;
    use omoba_core::Outcome;
    use specs::WorldExt;

    #[test]
    fn every_rank_equips_the_gun_for_its_duration_with_final_range_800() {
        for (rank,duration,damage) in [(1u8,28,50),(2,32,70),(3,36,90)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&TanegashimaHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();
            let buffs:Vec<_>=outcomes.iter().filter_map(|o|match o {
                Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==BUFF_ID=>Some((payload.clone(),*duration)),_=>None}).collect();
            assert_eq!(buffs.len(),1,"rank {rank}");
            let (payload,got)=&buffs[0];
            assert_eq!(*got,Fixed64::from_i32(duration),"rank {rank}");
            // B04.txt:296: source melee base + range_bonus = the described final range.
            assert_eq!(payload[StatKey::AttackRangeBonus.as_str()].as_i64(),Some(672*1024),"rank {rank}");
            assert_eq!(payload[StatKey::BaseAttackBonusDamage.as_str()].as_i64(),Some(i64::from(damage)*1024),"rank {rank}");
            let chance=payload[StatKey::AttackStunChance.as_str()].as_f64().unwrap();
            let stun=payload[StatKey::AttackStunDuration.as_str()].as_f64().unwrap();
            assert!((chance-0.38).abs()<1.0/1024.0 && (stun-0.2).abs()<1.0/1024.0,"rank {rank}: {chance} {stun}");
            // B04.lua:171-175: launch-time stun that skips buildings.
            assert_eq!(payload[attack_stun::ON_LAUNCH_KEY].as_i64(),Some(1),"rank {rank}");
            assert_eq!(payload[omoba_core::runtime::native::magic_immunity::PIERCE_KEY].as_i64(),Some(1),"rank {rank}: the stun pierces magic immunity");
            assert_eq!(payload[attack_stun::SKIP_BUILDINGS_KEY].as_i64(),Some(1),"rank {rank}");
            assert!(payload.get(StatKey::Lifesteal.as_str()).is_none(),"source has no lifesteal");
        }
    }

    #[test]
    fn every_rank_reads_the_source_bullet() {
        for (rank,damage) in [(1u8,35),(2,50),(3,65)] {
            let b=bullets(rank).expect("bullet values");
            assert_eq!(b.damage,Fixed64::from_i32(damage),"rank {rank}: B04.txt:321");
            assert_eq!((b.distance,b.radius,b.speed),(Fixed64::from_i32(1200),Fixed64::from_i32(100),Fixed64::from_i32(2000)));
            assert_eq!((b.extra_hits,b.extra_outcomes),(26,101),"RandomInt(0,100) <= 25");
            assert_eq!(b.delays,[Fixed64::from_raw(51),Fixed64::from_raw(102)],"0.05 / 0.1 s");
        }
        assert!(bullets(0).is_none() && bullets(4).is_none());
    }

    /// Runs the launch hook in a fresh adapter; returns the projectile launch rows.
    fn launch(t:&mut TestWorld,target:specs::Entity)->Vec<(serde_json::Value,Fixed64)> {
        use abi_stable::{sabi_trait::prelude::TD_Opaque,RMut};
        use omoba_core::runtime::scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter};
        let cache=ParallelAdapterCache::new(&t.world,1);
        let mut adapter=ParallelWorldAdapter::new(&cache,t.source);
        let mut dynworld=omb_script_abi::world::GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
        let fired=on_owner_attack_launch(&mut dynworld,t.handle(t.source),t.handle(target),2);
        drop(dynworld);
        let rows:Vec<_>=adapter.finish().into_iter().filter_map(|o|match o {
            Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==host::BUFF=>Some((payload,duration)),_=>None}).collect();
        assert_eq!(rows.len(),fired);rows
    }

    #[test]
    fn each_attack_launch_fires_a_piercing_bullet_and_rolls_two_delayed_extras() {
        let mut t=TestWorld::new();
        let enemy=t.body(p(400,0),2);
        assert!(launch(&mut t,enemy).is_empty(),"no gun, no bullets");
        let source=t.source;
        t.world.write_resource::<omoba_core::runtime::BuffStore>().add(source,BUFF_ID,Fixed64::from_i32(30),serde_json::json!({}));
        let rows=launch(&mut t,enemy);
        assert!((1..=3).contains(&rows.len()));
        let first=host::Spec::from_payload(host::BUFF,rows[0].1,&rows[0].0).expect("valid linear bullet");
        assert_eq!(first.motion,host::Motion::Linear {aim:p(400,0)});
        assert_eq!((first.amount,first.distance,first.max_hits,first.buildings),(Fixed64::from_i32(50),Fixed64::from_i32(1200),host::MAX_HITS,false));
        assert_eq!(first.kind,DamageKind::Magical);
        assert_eq!(host::delay_of(&rows[0].0),Fixed64::ZERO,"the first bullet leaves with the attack");
        for (payload,_) in &rows[1..] {assert!(host::delay_of(payload)>Fixed64::ZERO,"extras are delayed");}
        // Over many attacks both extras appear, at about 26/101 each.
        let mut extras=0;let attacks=400;
        for _ in 0..attacks {t.world.write_resource::<omoba_core::Tick>().0+=1;extras+=launch(&mut t,enemy).len()-1;}
        assert!(extras>attacks*2*18/100 && extras<attacks*2*34/100,"extras {extras} of {}",attacks*2);
    }

    #[test]
    fn windup_metadata_is_accepted_but_targets_and_ranks_are_strict() {
        let mut t=TestWorld::new();
        assert!(t.cast(&TanegashimaHandler,Target::None,1,&rank_json(ID,1)).is_ok());
        assert!(t.cast(&TanegashimaHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&TanegashimaHandler,Target::None,4,&rank_json(ID,3)).is_err());
        let ranged=serde_json::json!({"cooldown":110,"mana_cost":150,"cast_time":0.1,"range":800}).to_string();
        assert!(t.cast(&TanegashimaHandler,Target::None,1,&ranged).is_err(),"self transform keeps rejecting a cast range");
    }
}
