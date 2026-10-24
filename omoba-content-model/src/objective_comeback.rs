//! Shared, integer-only objective compensation policy. All-zero preserves legacy rules.
use serde::{Deserialize,Serialize};

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectiveComebackRules {
    pub starts_seconds:u32,
    pub activate_gap:u32,
    pub release_gap:u32,
    pub tower_gold_per_player:u32,
    pub epic_gold_per_player:u32,
    pub activation_pool:u32,
}
impl ObjectiveComebackRules {
    pub fn is_valid(self)->bool {
        self==Self::default() || ((1..=3600).contains(&self.starts_seconds)
            && (1..=1_000_000).contains(&self.activate_gap) && self.release_gap<self.activate_gap
            && self.tower_gold_per_player<=1_000_000 && self.epic_gold_per_player<=1_000_000
            && self.tower_gold_per_player.max(self.epic_gold_per_player)>0
            && (1..=1_000_000).contains(&self.activation_pool))
    }
}
