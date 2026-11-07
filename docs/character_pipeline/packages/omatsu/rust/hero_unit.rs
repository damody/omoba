//! 阿松 hero UnitScript（content `unit_script = true`）。host 以英雄 content id 綁定，
//! 所以 unit_id 必須是 hero id。急速冷卻主動期間（modifier_A34T OnAbilityExecuted →
//! Mana_regen，A34_Matsu.lua:120-123）每施放一個技能回復法力（v3.lua 急速冷卻的 mana_on_cast）。
//! on_skill_cast 在技能 execute 之前、扣費之後於同一施法交易內執行；急速冷卻本身施放
//! 時主動 buff 尚未掛上，所以不回魔。
use abi_stable::std_types::RStr;
use omb_script_abi::{script::UnitScript,types::{EntityHandle,Target},world::GameWorldDyn};

pub struct OmatsuUnit;

impl UnitScript for OmatsuUnit {
    fn unit_id(&self)->RStr<'_> {RStr::from_str("omatsu")}
    fn on_skill_cast(&self,caster:EntityHandle,_skill_id:RStr<'_>,_target:Target,w:&mut GameWorldDyn<'_>) {
        if let Some(amount)=super::No4_rapid_cooling::mana_on_cast(w,caster) {w.restore_mana(caster,amount);}
    }
}

pub fn hero_unit_script()->OmatsuUnit {OmatsuUnit}
