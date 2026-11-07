//! 阿市 hero UnitScript（content `unit_script = true`）：施法掛點（技能施放與物品使用）
//! 交給 D 欄被動自我奉獻。host 以英雄 content id 綁定，所以 unit_id 必須是 hero id。
use abi_stable::std_types::RStr;
use omb_script_abi::{script::UnitScript,types::{EntityHandle,Target},world::GameWorldDyn};

pub struct OichiUnit;

impl UnitScript for OichiUnit {
    fn unit_id(&self)->RStr<'_> {RStr::from_str("oichi")}
    fn on_skill_cast(&self,caster:EntityHandle,skill_id:RStr<'_>,_:Target,w:&mut GameWorldDyn<'_>) {
        super::No5_devotion::on_owner_cast(w,caster,skill_id.as_str());
    }
}

pub fn hero_unit_script()->OichiUnit {OichiUnit}
