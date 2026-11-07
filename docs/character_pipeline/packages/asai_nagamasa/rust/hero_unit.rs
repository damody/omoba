//! 淺井長政 hero UnitScript（content `unit_script = true`）：把掛點交給各技能 handler。
//! host 以英雄 content id 綁定，所以 unit_id 必須是 hero id。
//! - on_skill_cast（技能施放與物品使用）：活心．魂（No3_living_soul）。
//! - 每個 tick：一刀．連牙斷空的瞬移／刀牙序列（No5_endless_fang）。
use abi_stable::std_types::RStr;
use omb_script_abi::{script::UnitScript,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};

pub struct AsaiNagamasaUnit;

impl UnitScript for AsaiNagamasaUnit {
    fn unit_id(&self)->RStr<'_> {RStr::from_str("asai_nagamasa")}
    fn on_skill_cast(&self,caster:EntityHandle,skill_id:RStr<'_>,_:Target,w:&mut GameWorldDyn<'_>) {
        super::No3_living_soul::on_owner_cast(w,caster,skill_id.as_str());
    }
    fn on_tick(&self,hero:EntityHandle,_dt:Fixed64,w:&mut GameWorldDyn<'_>) {
        super::No5_endless_fang::on_owner_tick(w,hero);
    }
}

pub fn hero_unit_script()->AsaiNagamasaUnit {AsaiNagamasaUnit}
