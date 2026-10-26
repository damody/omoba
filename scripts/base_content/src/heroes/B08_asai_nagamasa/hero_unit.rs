//! 淺井長政 hero UnitScript（content `unit_script = true`）：把施法掛點交給
//! 各被動 handler。host 以英雄 content id 綁定，所以 unit_id 必須是 hero id。
use abi_stable::std_types::RStr;
use omb_script_abi::{script::UnitScript,types::{EntityHandle,Target},world::GameWorldDyn};

pub struct AsaiNagamasaUnit;

impl UnitScript for AsaiNagamasaUnit {
    fn unit_id(&self)->RStr<'_> {RStr::from_str("asai_nagamasa")}
    fn on_skill_cast(&self,caster:EntityHandle,skill_id:RStr<'_>,_:Target,w:&mut GameWorldDyn<'_>) {
        super::No3_living_soul::on_owner_cast(w,caster,skill_id.as_str());
    }
}

pub fn hero_unit_script()->AsaiNagamasaUnit {AsaiNagamasaUnit}
