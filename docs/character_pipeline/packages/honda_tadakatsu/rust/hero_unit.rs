//! 本多忠勝 hero UnitScript（content `unit_script = true`）。host 以英雄 content
//! id 綁定，所以 unit_id 必須是 hero id。
//! - 普攻命中：碎擊槍決定下一次是否暴擊（No3_shattering_spear），戰國最強期間濺射
//!   （No4_strongest）。
//! - 每個 tick：赤雷擊施放 unslow_delay 秒後解除自身減速（No2_red_thunder）。
use abi_stable::std_types::RStr;
use omb_script_abi::{script::UnitScript,types::{EntityHandle,Fixed64},world::GameWorldDyn};

pub struct HondaTadakatsuUnit;

impl UnitScript for HondaTadakatsuUnit {
    fn unit_id(&self)->RStr<'_> {RStr::from_str("honda_tadakatsu")}
    fn on_attack_hit(&self,attacker:EntityHandle,victim:EntityHandle,w:&mut GameWorldDyn<'_>) {
        super::No4_strongest::on_owner_attack_hit(w,attacker,victim);
        super::No3_shattering_spear::on_owner_attack_hit(w,attacker);
    }
    fn on_tick(&self,hero:EntityHandle,_dt:Fixed64,w:&mut GameWorldDyn<'_>) {
        super::No2_red_thunder::on_owner_tick(w,hero);
    }
}

pub fn hero_unit_script()->HondaTadakatsuUnit {HondaTadakatsuUnit}
