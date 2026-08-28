//! Code-owned projection policy catalogue. Secure matches validate the entire
//! required catalogue before simulation starts; unknown script IDs fail closed.

use std::collections::{BTreeMap, BTreeSet};

use omb_script_abi::types::{projection_policy_ids, ProjectionPolicyId};

pub const REQUIRED_PROJECTION_POLICIES: &[(&str, &str)] = &[
    (
        projection_policy_ids::MOVEMENT,
        "runtime/native/tick/movement",
    ),
    (
        projection_policy_ids::SPAWN,
        "runtime/native/game_processor/spawn",
    ),
    (projection_policy_ids::DEATH, "runtime/native/tick/death"),
    (
        projection_policy_ids::OWNERSHIP,
        "runtime/native/game_processor/ownership",
    ),
    (
        projection_policy_ids::DIRECT_COMBAT,
        "runtime/native/tick/damage",
    ),
    (
        projection_policy_ids::PROJECTILE,
        "runtime/native/tick/projectile",
    ),
    (
        projection_policy_ids::AOE,
        "runtime/native/game_processor/aoe",
    ),
    (
        projection_policy_ids::BUFF_DEBUFF,
        "runtime/native/tick/buff",
    ),
    (
        projection_policy_ids::HERO_ABILITY,
        "runtime/native/tick/hero",
    ),
    (projection_policy_ids::TOWER, "runtime/native/tick/tower"),
    (projection_policy_ids::ITEM, "runtime/native/tick/item"),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionPolicyRegistration {
    pub action_id: String,
    pub source_module_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingProjectionPolicy {
    pub action_id: String,
    pub source_module_path: String,
}

#[derive(Clone, Debug, Default)]
pub struct ProjectionPolicyRegistry {
    registrations: BTreeMap<String, ProjectionPolicyRegistration>,
}

impl ProjectionPolicyRegistry {
    pub fn secure_defaults() -> Self {
        let mut registry = Self::default();
        for (action_id, path) in REQUIRED_PROJECTION_POLICIES {
            registry.register_str(action_id, path);
        }
        registry
    }

    pub fn register_str(&mut self, action_id: &str, source_module_path: &str) {
        self.registrations.insert(
            action_id.to_owned(),
            ProjectionPolicyRegistration {
                action_id: action_id.to_owned(),
                source_module_path: source_module_path.to_owned(),
            },
        );
    }

    /// Script host adapter entry point. ABI version mismatch and unknown IDs
    /// are rejected without partially modifying the registry.
    pub fn register_script_policy(
        &mut self,
        id: &ProjectionPolicyId,
        source_module_path: &str,
    ) -> Result<(), MissingProjectionPolicy> {
        let action_id = id.value.as_str();
        let known: BTreeSet<_> = REQUIRED_PROJECTION_POLICIES
            .iter()
            .map(|(id, _)| *id)
            .collect();
        if id.abi_version != ProjectionPolicyId::ABI_VERSION || !known.contains(action_id) {
            return Err(MissingProjectionPolicy {
                action_id: action_id.to_owned(),
                source_module_path: source_module_path.to_owned(),
            });
        }
        self.register_str(action_id, source_module_path);
        Ok(())
    }

    pub fn validate_complete(&self) -> Result<(), Vec<MissingProjectionPolicy>> {
        let missing: Vec<_> = REQUIRED_PROJECTION_POLICIES
            .iter()
            .filter(|(id, _)| !self.registrations.contains_key(*id))
            .map(|(action_id, source_module_path)| MissingProjectionPolicy {
                action_id: (*action_id).to_owned(),
                source_module_path: (*source_module_path).to_owned(),
            })
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    pub fn contains(&self, action_id: &str) -> bool {
        self.registrations.contains_key(action_id)
    }
}

pub fn validate_secure_match_startup(registry: &ProjectionPolicyRegistry) -> Result<(), String> {
    registry.validate_complete().map_err(|missing| {
        let entries = missing
            .into_iter()
            .map(|entry| format!("{} ({})", entry.action_id, entry.source_module_path))
            .collect::<Vec<_>>()
            .join(", ");
        format!("secure match blocked: missing projection policy: {entries}")
    })
}
