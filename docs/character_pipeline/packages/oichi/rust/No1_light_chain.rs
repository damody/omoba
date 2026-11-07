//! 光鍊罰則（H04，來源 C17W current，C17.txt:47-160；C17.lua:C17W:212-222、C17W_Enemy:134-210、
//! C17W_Friend:225-265）：在阿市與目標英雄（只有英雄，C17.txt:52）之間牽起 3 秒光鍊。
//! - 對敵：目標身上掛 host tether buff；自然到期時依邊到邊距離 r（最多 1000）結算：
//!   傷害 r × 0.35/0.52/0.69/0.86（＝350..860 × r/1000）、定身
//!   ((1000 − r)/200 + 1) × 0.3/0.5/0.7/0.9（貼身 1.8..5.4、1000 外 0.3..0.9），定身時另受
//!   5 點魔法傷害（modifier_C17W OnCreated，C17.txt:109-114）。阿市先死亡時距離傷害略過
//!   （AMHC:Damage 攻擊者已死），定身與 5 點傷害照常（194）。光鍊是 Lua Timer，不能驅散；
//!   目標先死亡則不結算。
//! - 對友：3 秒內受到傷害 −30/40/50/60%（modifier_C17W2）。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{DamageKind,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::tether;
use omoba_template_ids::{ABILITY_C17_LIGHT_CHAIN,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_C17_LIGHT_CHAIN;

pub(crate) struct Chain {pub range:Fixed64,pub duration:Fixed64,pub reach:Fixed64,pub damage:Fixed64,pub root:Fixed64,
    pub root_far:Fixed64,pub root_damage:Fixed64,pub guard:Fixed64,pub guard_duration:Fixed64,
    /// Both teams' plain sight following Oichi and the target (E185).
    pub vision_radius:Fixed64,pub vision_duration:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Chain,&'static str> {
    let content=active_ability_const(ID).ok_or("missing light chain content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid light chain rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid light chain rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid light chain metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid light chain range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing light chain extra");
    let spec=Chain {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),duration:extra("tether_duration")?,
        reach:extra("max_distance")?,damage:extra("damage")?,root:extra("root")?,
        root_far:extra("root_far")?,root_damage:extra("root_damage")?,
        // Authored in percent (−30 = −30% damage taken); the stat is a fraction.
        guard:Fixed64::from_raw((extra("guard")?.raw()-50)/100),guard_duration:extra("guard_duration")?,
        vision_radius:extra("vision_radius")?,vision_duration:extra("vision_duration")?};
    if spec.duration<=Fixed64::ZERO || spec.duration>Fixed64::from_i32(60) || spec.guard>=Fixed64::ZERO || spec.guard<=-Fixed64::ONE
        || spec.guard_duration<=Fixed64::ZERO || !omoba_core::runtime::native::reveal_zone::valid_duration(spec.vision_duration) {return Err("invalid light chain values");}
    Ok(spec)
}

/// A hero template (MOBA hero id or the legacy `hero_<id>` tag), as B01 雨鐵炮's target classes.
fn is_hero(world:&GameWorldDyn<'_>,e:EntityHandle)->bool {
    !world.is_building(e) && world.unit_id_of(e).into_option().is_some_and(|id| {
        let id=id.as_str();
        omoba_template_ids::hero_by_name(id.strip_prefix("hero_").unwrap_or(id)).is_some()
    })
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Chain)->Result<(),&'static str> {
    let Target::Entity(other)=target else {return Err("light chain requires a unit target");};
    if other==caster || !is_hero(world,other) || !world.is_alive(other)
        || !world.get_hp(other).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("light chain target is not a live hero");}
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let at=world.get_pos(other).into_option().ok_or("missing target position")?;
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("light chain target outside cast range");}
    let enemy=world.query_enemies_in_range(at,Fixed64::ZERO,caster).iter().any(|e|*e==other);
    let ally=!enemy && world.faction_of(caster).into_option().map(|f|f.as_str().to_owned())==world.faction_of(other).into_option().map(|f|f.as_str().to_owned());
    if enemy {
        let link=tether::Spec {root_far:spec.root_far,root_damage:spec.root_damage,edge:true,root_after_source_death:true,
            ..tether::Spec::new(caster.id,caster.gen,spec.damage,DamageKind::Magical,spec.root,spec.reach)}.checked().ok_or("invalid light chain tether")?;
        let id=format!("c17_light_chain:{}:{}",caster.id,caster.gen);
        world.add_stat_buff(other,id.as_str().into(),spec.duration,link.payload().to_string().as_str().into());
    } else if ally {
        let id=format!("c17_light_chain_guard:{}:{}",caster.id,caster.gen);
        let payload=serde_json::json!({(StatKey::IncomingDamagePercentage.as_str()):spec.guard.raw()}).to_string();
        world.add_stat_buff(other,id.as_str().into(),spec.guard_duration,payload.as_str().into());
    } else {
        return Err("light chain target is neither enemy nor ally");
    }
    // Both teams see around Oichi and the target while the chain lasts (following each).
    use omoba_core::runtime::native::reveal_zone;
    for (unit,pos) in [(caster,origin),(other,at)] {
        let sight=reveal_zone::sight_payload(caster.id,caster.gen,pos,spec.vision_radius,true,Some((unit.id,unit.gen))).ok_or("invalid light chain sight")?;
        world.add_stat_buff(caster,reveal_zone::BUFF.into(),spec.vision_duration,sight.to_string().as_str().into());
    }
    Ok(())
}

pub struct LightChainHandler;
impl AbilityScript for LightChainHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn c17_light_chain_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,LightChainHandler,vec![
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical},
        EffectSpec::StatusModifier {target:TargetSelector::Target,modifier_type:"root".into(),value:1.0,
            duration:Some(extra_at_id_f32(ID,"root",1))}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::C17_oichi::c17_test_support::*;
    use omoba_core::Outcome;

    #[test]
    fn enemies_get_a_rank_tether_and_allies_a_damage_guard() {
        for (rank,damage,near,far,guard) in [(1u8,350,1843,307,-307),(2,520,3072,512,-410),(3,690,4301,717,-512),(4,860,5530,922,-614)] {
            let mut t=TestWorld::new();let enemy=t.hero(p(500,0),2);let ally=t.hero(p(300,0),1);
            let link=t.cast(&LightChainHandler,Target::Entity(t.handle(enemy)),rank,&rank_json(ID,rank)).unwrap().iter().find_map(|o|match o {
                Outcome::AddBuff {target,payload,duration,..} if *target==enemy=>Some((tether::Spec::from_payload(payload),*duration)),_=>None}).expect("tether");
            let spec=link.0.expect("strict tether payload");
            assert_eq!(link.1,Fixed64::from_i32(3));
            assert_eq!((spec.amount,spec.range),(Fixed64::from_i32(damage),Fixed64::from_i32(1000)),"rank {rank}: distance capped at 1000");
            assert_eq!((spec.root,spec.root_far),(Fixed64::from_raw(near),Fixed64::from_raw(far)),"rank {rank}: 6k near, k far");
            assert_eq!(spec.root_damage,Fixed64::from_i32(5),"5 magical damage when the root lands");
            assert!(spec.edge && spec.root_after_source_death);
            let rows:Vec<_>=t.cast(&LightChainHandler,Target::Entity(t.handle(ally)),rank,&rank_json(ID,rank)).unwrap().iter().filter_map(|o|match o {
                Outcome::AddBuff {target,payload,duration,..} if *target==ally=>Some((payload[StatKey::IncomingDamagePercentage.as_str()].as_i64(),*duration)),_=>None}).collect();
            assert_eq!(rows,vec![(Some(guard),Fixed64::from_i32(3))],"rank {rank}");
        }
    }

    /// modifier_C17W2：MOBA 傷害結算真的套用受到傷害 −30%（incoming_damage_percentage）。
    #[test]
    fn the_ally_guard_reduces_settled_damage() {
        let mut t=TestWorld::new();let ally=t.hero(p(300,0),1);let source=t.source;
        let outcomes=t.cast(&LightChainHandler,Target::Entity(t.handle(ally)),1,&rank_json(ID,1)).unwrap();
        t.apply(outcomes);
        t.apply(vec![Outcome::RawDamage {pos:p(300,0),phys:Fixed64::ZERO,magi:Fixed64::ZERO,real:Fixed64::from_i32(100),source,target:ally,
            damage_profile:omb_script_abi::types::DamageProfile::TRUE.bits(),origin:None}]);
        let hp={use specs::WorldExt;t.world.read_storage::<omoba_core::CProperty>().get(ally).unwrap().hp};
        assert_eq!(hp,Fixed64::from_raw(1000*1024-100*(1024-307)),"100 pure damage lands as 70");
    }

    #[test]
    fn self_creeps_buildings_and_out_of_range_are_rejected() {
        let mut t=TestWorld::new();let tower=t.building(p(200,0),2);let far=t.hero(p(601,0),2);let creep=t.body(p(300,0),2);
        let ally_creep=t.body(p(300,0),1);let source=t.source;
        for target in [source,tower,far,creep,ally_creep] {assert!(t.cast(&LightChainHandler,Target::Entity(t.handle(target)),1,&rank_json(ID,1)).is_err());}
        assert!(t.cast(&LightChainHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
    }
}
