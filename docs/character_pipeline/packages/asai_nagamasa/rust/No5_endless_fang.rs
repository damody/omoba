//! 一刀．連牙斷空（H06，來源 B08D current，D 欄大招；B08.txt:14-140、
//! B08_Asai_Nagamasa.lua:B08D_old:45-108）。施放時（B08D_old 64-66）：
//! - 目標暈眩 target_stun 秒（不檢查魔免）；施法者暈眩 self_stun 秒；
//! - 施法者 modifier_invulnerable guard_time 秒：本遊戲只做「不受傷害」（absolute_no_damage_*），
//!   不可被選取的部分是 needs_framework（見 parity.md）。
//! 序列（相對施放時刻 t，step = step_interval，n = 1..=count+1）由英雄 UnitScript on_tick（[`on_owner_tick`]）
//! 依施法者身上的標記 buff 驅動：
//! - n ≥ 2：從 pos[n−1] 朝目標原位置射出直線刀牙（原作 79-100）：半徑 radius、
//!   速度 speed、飛行 distance、穿透，每個敵人 damage 魔法傷害一次；
//! - n ≤ count：施法者瞬移到 pos[n] = 中心 + ring_radius·(cos θn, sin θn)（原作 68-78）；
//! - n = count+1（return_at）：施法者回到目標原位置（104-107）。
//! 同一時刻先射出上一位置的刀牙再瞬移，所以每顆刀牙都從自己的環上位置出發。
//!
//! GameWorld 不讓腳本讀 buff payload，所以序列狀態放在標記 buff 的剩餘時間：
//! `seq` 的經過時間是序列時鐘；中心座標存成 `cx`／`cy` 與 `seq` 的剩餘時間差
//! （同一批加入、同步倒數，差值精確不變）；每步完成加一個 `step{n}` 標記。全部
//! 只由 has_buff／get_buff_remaining 判定，authority 與施法方隊伍 replica 一致。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},buff_ids::BuffId,stat_keys::StatKey,
    types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target,Vec2},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::ability_projectile as host;
use omoba_template_ids::{ABILITY_B08_ENDLESS_FANG,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B08_ENDLESS_FANG;
/// Coordinate bias of the centre markers (raw), above any map coordinate.
const BIAS_RAW:i64=1<<30;
/// The sequence clock outlives the last step by this margin.
const MARGIN:Fixed64=Fixed64::from_raw(512);
pub(crate) const GUARD_BUFF:&str="b08_endless_fang_invulnerable";

fn marker(name:&str,caster:EntityHandle)->String {format!("b08_endless_fang_{name}:{}:{}",caster.id,caster.gen)}
fn step_marker(n:u32,caster:EntityHandle)->String {marker(&format!("step{n}"),caster)}

#[derive(Clone,Copy,Debug)]
pub(crate) struct Fang {pub range:Fixed64,pub stun:Fixed64,pub count:u32,pub ring:Fixed64,pub return_at:Fixed64,
    pub guard:Fixed64,pub self_stun:Fixed64,pub speed:Fixed64,pub radius:Fixed64,pub distance:Fixed64,pub max_hits:u16,pub damage:Fixed64,
    /// Both teams' plain sight at the target's original position (E185).
    pub vision_radius:Fixed64,pub vision_duration:Fixed64}

impl Fang {
    /// Steps 1..=count teleport, step count+1 returns.
    fn steps(&self)->u32 {self.count+1}
    fn clock(&self)->Fixed64 {self.return_at+MARGIN}
    /// Time of step `n`: n/steps of the return time (the last step is exactly return_at).
    fn at(&self,n:u32)->Fixed64 {Fixed64::from_raw(self.return_at.raw()*i64::from(n)/i64::from(self.steps()))}
    /// Ring point `k` (1-based): 720·k/count degrees around the centre.
    fn ring_point(&self,center:Vec2,k:u32)->Vec2 {
        let angle=omoba_sim::Angle::from_degrees_i32((720*k/self.count) as i32);
        center+Vec2::new(self.ring*omoba_sim::trig::cos(angle),self.ring*omoba_sim::trig::sin(angle))
    }
}

/// Content values (rank 1; the ability has one rank).
pub(crate) fn spec(rank:u8)->Result<Fang,&'static str> {
    let content=active_ability_const(ID).ok_or("missing endless fang content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid endless fang rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid endless fang rank");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing endless fang extra");
    let whole=|v:Fixed64|(v.raw()%1024==0 && v>Fixed64::ZERO).then_some((v.raw()/1024) as u32).ok_or("endless fang count must be whole");
    let hits=whole(extra("max_hits")?)?;
    let spec=Fang {range:content.levels[index].range,stun:extra("target_stun")?,count:whole(extra("count")?)?,
        ring:extra("ring_radius")?,return_at:extra("return_at")?,guard:extra("guard_time")?,
        self_stun:extra("self_stun")?,speed:extra("speed")?,radius:extra("radius")?,
        distance:extra("distance")?,max_hits:u16::try_from(hits).map_err(|_|"endless fang hit budget")?,damage:extra("damage")?,
        vision_radius:extra("vision_radius")?,vision_duration:extra("vision_duration")?};
    // The authored step_interval and return_at must agree (step_interval × (count+1) = return_at, up to Q10 rounding).
    let interval=extra("step_interval")?;
    if spec.count==0 || spec.count>64 || 720%spec.count!=0 || interval<=Fixed64::ZERO
        || (interval.raw()*i64::from(spec.steps())-spec.return_at.raw()).abs()>i64::from(spec.steps())
        || spec.ring<=Fixed64::ZERO || spec.ring>Fixed64::from_i32(2000) || spec.guard<=Fixed64::ZERO || spec.guard>Fixed64::from_i32(10)
        || spec.self_stun<=Fixed64::ZERO || spec.self_stun>spec.guard || spec.stun<=Fixed64::ZERO
        || !omoba_core::runtime::native::reveal_zone::valid_duration(spec.vision_duration) {return Err("invalid endless fang values");}
    Ok(spec)
}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Fang,&'static str> {
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid endless fang metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid endless fang range");}
    Ok(Fang {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),..spec(rank)?})
}

fn projectile(caster:EntityHandle,aim:Vec2,spec:&Fang)->host::Spec {
    let lifetime=Fixed64::from_raw((i128::from(spec.distance.raw())*1024/i128::from(spec.speed.raw().max(1))) as i64)+Fixed64::ONE;
    host::Spec {source_id:caster.id,source_generation:caster.gen,ability:ID.as_str().to_owned(),motion:host::Motion::Linear {aim},
        speed:spec.speed,radius:spec.radius,distance:spec.distance,lifetime,max_hits:spec.max_hits,
        wall:host::WallPolicy::Pass,target_lost:host::TargetLostPolicy::Expire,caster_death:host::CasterDeathPolicy::Continue,
        amount:spec.damage,kind:DamageKind::Magical,profile:DamageProfile::SHARP.union(DamageProfile::SPELL).bits(),slow:None,control:None,dot:None,buildings:false,forced:None}
}

fn clear_markers(world:&mut GameWorldDyn<'_>,caster:EntityHandle,spec:&Fang) {
    for name in ["seq","cx","cy"] {world.remove_buff(caster,marker(name,caster).as_str().into());}
    for n in 1..=spec.steps() {world.remove_buff(caster,step_marker(n,caster).as_str().into());}
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Fang)->Result<(),&'static str> {
    let Target::Entity(victim)=target else {return Err("endless fang requires a unit target");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let at=world.get_pos(victim).into_option().ok_or("missing target position")?;
    if victim==caster || world.is_building(victim) || !world.is_alive(victim) || !world.get_hp(victim).into_option().is_some_and(|hp|hp>Fixed64::ZERO)
        || !world.query_enemies_in_range(at,Fixed64::ZERO,caster).iter().any(|e|*e==victim) {return Err("endless fang target is not a live enemy unit");}
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("endless fang target outside cast range");}
    if !projectile(caster,at,&spec).valid() {return Err("invalid endless fang projectile");}
    // Source B08_Asai_Nagamasa.lua:46: B08D opens with ProjectileManager:ProjectileDodge(caster).
    world.add_stat_buff(caster,omoba_core::runtime::native::ability_projectile::DODGE_BUFF.into(),Fixed64::ONE,
        omoba_core::runtime::native::ability_projectile::dodge_payload(caster.id,caster.gen).to_string().as_str().into());
    // Source 64-66: caster invulnerable guard_time (damage only here), caster stunned self_stun, target stunned target_stun.
    let guard=serde_json::json!({(StatKey::AbsoluteNoDamagePhysical.as_str()):Fixed64::ONE.raw(),
        (StatKey::AbsoluteNoDamageMagical.as_str()):Fixed64::ONE.raw(),(StatKey::AbsoluteNoDamagePure.as_str()):Fixed64::ONE.raw()}).to_string();
    world.add_stat_buff(caster,GUARD_BUFF.into(),spec.guard,guard.as_str().into());
    world.add_buff(caster,BuffId::Stun.as_rstr(),spec.self_stun);
    world.add_buff(victim,BuffId::Stun.as_rstr(),spec.stun);
    // Source 62-63: AddFOWViewer for both teams at the target's original position (plain sight).
    use omoba_core::runtime::native::reveal_zone;
    let sight=reveal_zone::sight_payload(caster.id,caster.gen,at,spec.vision_radius,true,None).ok_or("invalid endless fang sight")?;
    world.add_stat_buff(caster,reveal_zone::BUFF.into(),spec.vision_duration,sight.to_string().as_str().into());
    // Sequence markers: a previous run's markers are cleared first (a refresh keeps the longer duration).
    clear_markers(world,caster,&spec);
    let clock=spec.clock();
    let coordinate=|v:Fixed64|Fixed64::from_raw(clock.raw()+BIAS_RAW+v.raw());
    world.add_stat_buff(caster,marker("seq",caster).as_str().into(),clock,"{}".into());
    world.add_stat_buff(caster,marker("cx",caster).as_str().into(),coordinate(at.x),"{}".into());
    world.add_stat_buff(caster,marker("cy",caster).as_str().into(),coordinate(at.y),"{}".into());
    Ok(())
}

/// Hero tick: runs every due step of an active sequence, in time order.
pub(crate) fn on_owner_tick(world:&mut GameWorldDyn<'_>,caster:EntityHandle) {
    let seq=marker("seq",caster);
    if !world.has_buff(caster,seq.as_str().into()) {return;}
    let Ok(spec)=spec(1) else {return;};
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {clear_markers(world,caster,&spec);return;}
    let left=world.get_buff_remaining(caster,seq.as_str().into());
    let elapsed=spec.clock()-left;
    let decode=|world:&GameWorldDyn<'_>,name:&str|Fixed64::from_raw(world.get_buff_remaining(caster,marker(name,caster).as_str().into()).raw()-left.raw()-BIAS_RAW);
    let center=Vec2::new(decode(world,"cx"),decode(world,"cy"));
    for n in 1..=spec.steps() {
        if elapsed<spec.at(n) {break;}
        let done=step_marker(n,caster);
        if world.has_buff(caster,done.as_str().into()) {continue;}
        if n>=2 {
            // The blade of the previous ring point, fired before the caster leaves it.
            let envelope=projectile(caster,center,&spec);
            let from=spec.ring_point(center,n-1);
            if envelope.valid() && from!=center {
                world.add_stat_buff(caster,host::BUFF.into(),envelope.lifetime,envelope.payload().to_string().as_str().into());
            }
        }
        if n<=spec.count {
            world.set_pos(caster,spec.ring_point(center,n));
            world.add_stat_buff(caster,done.as_str().into(),spec.clock(),"{}".into());
        } else {
            world.set_pos(caster,center);
            clear_markers(world,caster,&spec);
        }
    }
}

pub struct EndlessFangHandler;
impl AbilityScript for EndlessFangHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn b08_endless_fang_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,EndlessFangHandler,vec![
        EffectSpec::Projectile {speed:extra_at_id_f32(ID,"speed",1),on_hit:vec![
            EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical}]},
        EffectSpec::StatusModifier {target:TargetSelector::Target,modifier_type:"stun".into(),value:1.0,
            duration:Some(extra_at_id_f32(ID,"target_stun",1))}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B08_asai_nagamasa::b08_test_support::*;
    use omoba_core::Outcome;
    use omoba_core::runtime::BuffStore;
    use specs::WorldExt;

    fn tick(t:&mut TestWorld)->Vec<Outcome> {
        use omoba_core::runtime::scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter};
        let cache=ParallelAdapterCache::new(&t.world,1);
        let mut adapter=ParallelWorldAdapter::new(&cache,t.source);
        let caster=ParallelWorldAdapter::entity_to_handle(t.source);
        let mut w=omb_script_abi::world::GameWorld_TO::from_ptr(abi_stable::RMut::new(&mut adapter),abi_stable::sabi_trait::prelude::TD_Opaque);
        on_owner_tick(&mut w,caster);
        drop(w);adapter.finish()
    }

    #[test]
    fn content_learns_at_sixteen_with_the_source_numbers() {
        let content=active_ability_const(ID).unwrap();
        assert_eq!((content.max_level,content.levels[0].required_hero_level),(1,16));
        let s=spec(1).unwrap();
        assert_eq!((s.count,s.ring,s.steps()),(18,Fixed64::from_i32(700),19));
        assert_eq!((s.at(1),s.at(19)),(Fixed64::from_raw(102),s.return_at),"0.1 s steps, return at 1.9 s");
        assert_eq!((s.speed,s.radius,s.distance,s.damage),(Fixed64::from_i32(2000),Fixed64::from_i32(175),Fixed64::from_i32(1100),Fixed64::from_i32(100)));
        assert_eq!((s.guard,s.self_stun,s.stun),(Fixed64::from_raw(2253),Fixed64::from_i32(2),Fixed64::from_raw(2253)));
        // pos[18] = 720° = the 0° point; pos[9] = 360° as well: nine places, each twice.
        assert_eq!(s.ring_point(p(0,0),18),p(700,0));
        assert_eq!(s.ring_point(p(0,0),9),s.ring_point(p(0,0),18));
    }

    #[test]
    fn the_cast_stuns_both_guards_the_caster_and_rejects_allies_and_far_targets() {
        let mut t=TestWorld::new();let enemy=t.body(p(800,0),2);
        let outcomes=t.cast(&EndlessFangHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).unwrap();
        let stun=|who:specs::Entity|outcomes.iter().find_map(|o|match o {Outcome::AddBuff {target,buff_id,duration,..} if *target==who && buff_id=="stun"=>Some(*duration),_=>None});
        assert_eq!(stun(enemy),Some(Fixed64::from_raw(2253)),"target 2.2 s");
        assert_eq!(stun(t.source),Some(Fixed64::from_i32(2)),"caster 2.0 s");
        assert!(outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {target,buff_id,payload,..} if *target==t.source && buff_id==GUARD_BUFF
            && payload[StatKey::AbsoluteNoDamageMagical.as_str()]==serde_json::json!(1024))),"2.2 s damage immunity");
        assert!(!outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {..})),"no instant 1800");
        // the_cast_dodges_first: B08_Asai_Nagamasa.lua:46 ProjectileDodge(caster) opens the cast.
        let dodge=omoba_core::runtime::native::ability_projectile::DODGE_BUFF;
        assert!(matches!(outcomes.first(),Some(Outcome::AddBuff {target,buff_id,..}) if *target==t.source && buff_id==dodge),"dodge first");
        let ally=t.body(p(100,0),1);let far=t.body(p(1101,0),2);
        for target in [ally,far] {assert!(t.cast(&EndlessFangHandler,Target::Entity(t.handle(target)),1,&rank_json(ID,1)).is_err());}
    }

    /// 原作時間軸：step_interval·i 瞬移到 pos[i]，step_interval·(i+1) 從 pos[i] 朝中心射出，return_at 回到中心。
    /// （刀牙飛行與命中由 single_lane_match_tests 的整合測試驗證。）
    #[test]
    fn the_sequence_teleports_eighteen_times_fires_from_each_ring_point_and_returns() {
        let mut t=TestWorld::new();
        let center=p(800,0);let enemy=t.body(center,2);
        let outcomes=t.cast(&EndlessFangHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).unwrap();
        t.apply(outcomes);
        let s=spec(1).unwrap();
        let dt=Fixed64::from_raw(17);
        let (mut teleports,mut launches)=(Vec::new(),Vec::new());
        let mut elapsed=Fixed64::ZERO;
        while elapsed<Fixed64::from_i32(4) {
            let outcomes=tick(&mut t);
            let here=t.world.read_storage::<omoba_core::Pos>().get(t.source).unwrap().0;
            for o in &outcomes {match o {
                Outcome::ScriptSetPos {entity,pos} if *entity==t.source=>teleports.push((elapsed,*pos)),
                Outcome::AddBuff {buff_id,duration,payload,..} if buff_id==host::BUFF=>{
                    let spec=host::Spec::from_payload(buff_id,*duration,payload).expect("valid blade");
                    assert_eq!(spec.motion,host::Motion::Linear {aim:center},"aimed at the original centre");
                    launches.push((elapsed,here));
                }
                _=>{}
            }}
            t.apply(outcomes);
            t.world.write_resource::<BuffStore>().tick(dt);
            elapsed+=dt;
        }
        assert_eq!(teleports.len(),19,"eighteen ring points and the return");
        assert_eq!(launches.len(),18,"eighteen blades");
        for k in 1..=18u32 {
            let (at,pos)=teleports[(k-1) as usize];
            assert_eq!(pos,s.ring_point(center,k),"ring point {k}");
            assert!(at>=s.at(k) && at<s.at(k)+dt,"teleport {k} at 0.1·{k}");
            let (fired,from)=launches[(k-1) as usize];
            assert_eq!(from,s.ring_point(center,k),"blade {k} leaves its own ring point");
            assert!(fired>=s.at(k+1) && fired<s.at(k+1)+dt,"blade {k} at 0.1+0.1·{k}");
        }
        assert_eq!(teleports[18].1,center,"returns to the target's original point");
        assert!(teleports[18].0>=s.at(19) && teleports[18].0<s.at(19)+dt,"at 1.9 s");
        assert_eq!(t.world.read_storage::<omoba_core::Pos>().get(t.source).unwrap().0,center);
        assert!(!t.world.read_resource::<BuffStore>().has(t.source,&marker("seq",t.handle(t.source))),"markers cleared");
    }
}
