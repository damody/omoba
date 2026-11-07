//! 真田幸村 hero UnitScript（content `unit_script = true`）。host 以英雄 content id 綁定，
//! 所以 unit_id 必須是 hero id。日本第一武士的全軍光環改由通用光環發射器（E173）刷新，
//! tick 目前沒有工作。
use abi_stable::std_types::RStr;
use omb_script_abi::{script::UnitScript,types::{EntityHandle,Fixed64},world::GameWorldDyn};

pub struct SanadaYukimuraUnit;

impl UnitScript for SanadaYukimuraUnit {
    fn unit_id(&self)->RStr<'_> {RStr::from_str("sanada_yukimura")}
    fn on_tick(&self,_hero:EntityHandle,_dt:Fixed64,_w:&mut GameWorldDyn<'_>) {}
}

pub fn hero_unit_script()->SanadaYukimuraUnit {SanadaYukimuraUnit}
