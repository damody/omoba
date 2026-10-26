//! 玉子 hero UnitScript（content `unit_script = true`）。host 以英雄 content id 綁定，
//! 所以 unit_id 必須是 hero id。普攻命中交給百箭齊發的被動計數
//! （No4_hundred_arrows::on_owner_attack_hit），每第 8 次命中放出 1 支致命箭矢。
use abi_stable::std_types::RStr;
use omb_script_abi::{script::UnitScript,types::EntityHandle,world::GameWorldDyn};

pub struct TamakoUnit;

impl UnitScript for TamakoUnit {
    fn unit_id(&self)->RStr<'_> {RStr::from_str("tamako")}
    fn on_attack_hit(&self,attacker:EntityHandle,victim:EntityHandle,w:&mut GameWorldDyn<'_>) {
        super::No4_hundred_arrows::on_owner_attack_hit(w,attacker,victim);
    }
}

pub fn hero_unit_script()->TamakoUnit {TamakoUnit}
