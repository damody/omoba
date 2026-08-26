use omoba_sim::Fixed64;
use specs::{Component, VecStorage};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplicationScopeKind { ServerOnly, Public, OwnerTeam, Vision }

#[derive(Component, Clone, Copy, Debug)]
#[storage(VecStorage)]
pub struct ReplicationScope {
    pub kind: ReplicationScopeKind,
    pub owner_team: Option<u32>,
}

#[derive(Component, Clone, Copy, Debug)]
#[storage(VecStorage)]
pub struct VisionSource {
    pub team: u32,
    pub radius: Fixed64,
    pub detection_level: u16,
}

#[derive(Component, Clone, Copy, Debug, Default)]
#[storage(VecStorage)]
pub struct StealthProfile { pub stealth_level: u16 }

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VisibilityOverrideKind { ForceHide, ForceShow }

#[derive(Component, Clone, Copy, Debug)]
#[storage(VecStorage)]
pub struct VisibilityOverride {
    pub team: Option<u32>,
    pub kind: VisibilityOverrideKind,
    pub priority: i16,
    pub stable_rule_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RememberDisposition { Forget, LastKnown, Silhouette }

#[derive(Component, Clone, Copy, Debug)]
#[storage(VecStorage)]
pub struct RememberPolicy { pub disposition: RememberDisposition }
