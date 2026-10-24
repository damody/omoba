//! Rain Iron Cannon: E passive, impact-centered forward sector on normal hits.
use abi_stable::std_types::{RErr,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{EntityHandle,Fixed64,Target,DamageKind,DamageProfile},world::GameWorldDyn};
use omoba_template_ids::ABILITY_RAIN_IRON_CANNON;
use crate::{ability_builder::build_ability_ffi,passive_cone};
const BUFF_ID:&str="rain_iron_cannon_passive";
pub struct RainIronCannonHandler;
impl AbilityScript for RainIronCannonHandler {
 fn ability_id(&self)->RStr<'_> {RStr::from_str(ABILITY_RAIN_IRON_CANNON.as_str())}
 fn execute(&self,_:EntityHandle,_:Target,_:u8,_:RStr<'_>,_:&mut GameWorldDyn<'_>)->RResult<(),RString> {
  RErr("passive ability cannot be cast actively".into())
 }
 fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
  let Some(spec)=passive_cone::resolve(ABILITY_RAIN_IRON_CANNON,rank) else{return;};
  let payload=serde_json::json!({"visual_effect":BUFF_ID,"level":rank,"true_damage_pct":spec.attack_fraction.to_f32_for_render()}).to_string();
  world.add_stat_buff(caster,RStr::from_str(BUFF_ID),Fixed64::from_raw(i64::MAX),payload.as_str().into());
 }
 fn on_attack_hit(&self,owner:EntityHandle,attacker:EntityHandle,victim:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
  if owner!=attacker {return;}
  let Some(spec)=passive_cone::resolve(ABILITY_RAIN_IRON_CANNON,rank) else{return;};
  passive_cone::execute(world,attacker,victim,spec,DamageKind::Pure,DamageProfile::TRUE);
 }
}
pub fn rain_iron_cannon_ffi()->AbilityDefFFI {build_ability_ffi(ABILITY_RAIN_IRON_CANNON,RainIronCannonHandler,Vec::new())}