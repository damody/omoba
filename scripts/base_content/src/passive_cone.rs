//! Reusable impact-centered on-hit sector. Uses committed source queries when
//! supplied by the host; source retirement must not erase a launched attack.
use abi_stable::std_types::RSome;
use omb_script_abi::{types::{EntityHandle,Fixed64,DamageKind,DamageProfile},world::GameWorldDyn};
use omoba_template_ids::{AbilityId,active_ability_const};

pub(crate) struct PassiveCone {pub radius:Fixed64,pub half_ticks:u16,pub attack_fraction:Fixed64}
pub(crate) fn resolve(id:AbilityId,rank:u8)->Option<PassiveCone> {
 let content=active_ability_const(id)?;let index=usize::from(rank.checked_sub(1)?);
 if index>=content.levels.len() {return None;}
 let value=|key:&str|content.extras.iter().find(|(name,_)|*name==key)?.1.get(index).copied();
 let radius=value("aoe_radius")?;let fraction=value("true_damage_pct")?;
 let half_ticks=omoba_sim::sector::half_angle_ticks(value("arc_half_angle_rad")?)?;
 if radius<=Fixed64::ZERO||radius>Fixed64::from_i32(10_000)||half_ticks==0
  ||fraction<=Fixed64::ZERO||fraction>Fixed64::from_i32(4) {return None;}
 Some(PassiveCone {radius,half_ticks,attack_fraction:fraction})
}
pub(crate) fn execute(world:&mut GameWorldDyn<'_>,attacker:EntityHandle,victim:EntityHandle,
 spec:PassiveCone,kind:DamageKind,profile:DamageProfile)->bool {
 if attacker==victim||spec.radius<=Fixed64::ZERO||spec.radius>Fixed64::from_i32(10_000)
  ||spec.half_ticks==0||spec.half_ticks>2048||spec.attack_fraction<=Fixed64::ZERO||spec.attack_fraction>Fixed64::from_i32(4) {return false;}
 let supported=|p:omb_script_abi::types::Vec2|p.x.raw().unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64
  &&p.y.raw().unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64;
 let Some(origin)=world.get_pos(attacker).into_option().filter(|p|supported(*p)) else{return false;};
 let Some(apex)=world.get_pos(victim).into_option().filter(|p|supported(*p)) else{return false;};
 if origin==apex {return false;}
 let attack=world.get_final_atk(attacker);
 if attack<=Fixed64::ZERO||attack>Fixed64::from_i32(1_000_000) {return false;}
 let raw=i128::from(attack.raw())*i128::from(spec.attack_fraction.raw())/1024;
 if raw<=0||raw>i128::from(1_000_000*1024) {return false;}
 let damage=Fixed64::from_raw(raw as i64);
 let delta=apex-origin;let axis=omoba_sim::trig::atan2(delta.y,delta.x);
 let mut hits:Vec<_>=world.query_enemies_in_range(apex,spec.radius,attacker).into_iter()
  .filter(|e|*e!=attacker&&world.is_alive(*e)&&world.get_hp(*e).into_option().is_some_and(|hp|hp>Fixed64::ZERO))
  .filter(|e|world.get_pos(*e).into_option().is_some_and(|p|omoba_sim::sector::point_in_sector(p,apex,axis,spec.radius,spec.half_ticks))).collect();
 hits.sort_by_key(|e|(e.id,e.gen));hits.dedup();
 for target in hits {world.deal_damage(target,damage,kind,profile,RSome(attacker));}
 true
}
