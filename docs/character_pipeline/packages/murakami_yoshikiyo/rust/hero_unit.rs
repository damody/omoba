//! 村上義清 hero UnitScript（content `unit_script = true`）。host 以英雄 content
//! id 綁定，所以 unit_id 必須是 hero id。每個 tick 把掛點交給震地斧的破軍靈光
//! （No2_quake_axe::on_owner_tick），由它依施法者身上的 rank 標記刷新減甲。
use abi_stable::std_types::RStr;
use omb_script_abi::{script::UnitScript,types::{EntityHandle,Fixed64},world::GameWorldDyn};

pub struct MurakamiYoshikiyoUnit;

impl UnitScript for MurakamiYoshikiyoUnit {
    fn unit_id(&self)->RStr<'_> {RStr::from_str("murakami_yoshikiyo")}
    fn on_tick(&self,hero:EntityHandle,_dt:Fixed64,w:&mut GameWorldDyn<'_>) {
        super::No2_quake_axe::on_owner_tick(w,hero);
        super::No3_morale_boost::on_owner_tick(w,hero);
    }
}

pub fn hero_unit_script()->MurakamiYoshikiyoUnit {MurakamiYoshikiyoUnit}
