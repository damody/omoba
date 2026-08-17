use crate::runtime::comp::TdLayerState;
use omoba_sim::Fixed64;
use omoba_template_ids::TdLayerMetadataConst;

/// Monotonic deterministic serial assigned at the authoritative commit boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TdLayerCommitSerial(pub u64);

pub const TD_REGROW_INTERVAL: Fixed64 = Fixed64::from_i32(3);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageProfile(pub u32);

impl DamageProfile {
    pub fn new(bits: u32) -> Result<Self, TdLayerResolveError> {
        if bits == 0 || bits & !omoba_template_ids::td_rounds::damage_profile::KNOWN != 0 {
            return Err(TdLayerResolveError::InvalidDamageProfile(bits));
        }
        Ok(Self(bits))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HitProvenance {
    pub source_entity_id: u32,
    pub owner_player_id: Option<u32>,
    pub hit_serial: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoppedLayer {
    pub layer_id: String,
    pub cash: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedTdLayer {
    pub state: TdLayerState,
    pub hp: Fixed64,
    pub max_hp: Fixed64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TdLayerResolutionPlan {
    pub provenance: HitProvenance,
    pub immune_layer: Option<String>,
    pub popped: Vec<PoppedLayer>,
    /// First survivor reuses the original entity.
    pub original: Option<ResolvedTdLayer>,
    /// Remaining survivors are materialized in authored order.
    pub children: Vec<ResolvedTdLayer>,
    pub unapplied_damage: Fixed64,
}

impl TdLayerResolutionPlan {
    pub fn pop_count(&self) -> u32 {
        self.popped.len().try_into().unwrap_or(u32::MAX)
    }

    pub fn cash(&self) -> u32 {
        self.popped
            .iter()
            .fold(0u32, |total, layer| total.saturating_add(layer.cash))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TdLayerResolveError {
    InvalidDamageProfile(u32),
    InvalidDamage(Fixed64),
    MissingLayer(String),
    InvalidCurrentHp {
        layer: String,
        hp: Fixed64,
        max_hp: Fixed64,
    },
}

struct ConsumeResult {
    survivors: Vec<ResolvedTdLayer>,
    popped: Vec<PoppedLayer>,
    remaining_damage: Fixed64,
    immune_layer: Option<String>,
}

pub fn resolve_td_layer_damage(
    catalog: &[TdLayerMetadataConst],
    state: &TdLayerState,
    current_hp: Fixed64,
    damage: Fixed64,
    profile: DamageProfile,
    provenance: HitProvenance,
) -> Result<TdLayerResolutionPlan, TdLayerResolveError> {
    DamageProfile::new(profile.0)?;
    if damage < Fixed64::ZERO {
        return Err(TdLayerResolveError::InvalidDamage(damage));
    }
    let metadata = find_layer(catalog, &state.current_layer)?;
    let max_hp = layer_max_hp(metadata, state.properties);
    if current_hp <= Fixed64::ZERO || current_hp > max_hp {
        return Err(TdLayerResolveError::InvalidCurrentHp {
            layer: state.current_layer.clone(),
            hp: current_hp,
            max_hp,
        });
    }
    if damage == Fixed64::ZERO {
        return Ok(TdLayerResolutionPlan {
            provenance,
            immune_layer: None,
            popped: Vec::new(),
            original: Some(ResolvedTdLayer {
                state: state.clone(),
                hp: current_hp,
                max_hp,
            }),
            children: Vec::new(),
            unapplied_damage: Fixed64::ZERO,
        });
    }

    let consumed = consume_layer(
        catalog,
        state,
        &state.current_layer,
        current_hp,
        damage,
        profile,
        state.spawn_lineage,
    )?;
    let mut survivors = consumed.survivors.into_iter();
    let original = survivors.next();
    Ok(TdLayerResolutionPlan {
        provenance,
        immune_layer: consumed.immune_layer,
        popped: consumed.popped,
        original,
        children: survivors.collect(),
        unapplied_damage: consumed.remaining_damage,
    })
}

fn consume_layer(
    catalog: &[TdLayerMetadataConst],
    root_state: &TdLayerState,
    layer_id: &str,
    hp: Fixed64,
    damage: Fixed64,
    profile: DamageProfile,
    lineage: u64,
) -> Result<ConsumeResult, TdLayerResolveError> {
    let metadata = find_layer(catalog, layer_id)?;
    let properties = inherited_properties(root_state.properties, metadata);
    let max_hp = layer_max_hp(metadata, properties);
    let state = state_for_layer(root_state, metadata, properties, lineage);
    if !is_compatible(metadata, profile) {
        return Ok(ConsumeResult {
            survivors: vec![ResolvedTdLayer { state, hp, max_hp }],
            popped: Vec::new(),
            remaining_damage: damage,
            immune_layer: Some(layer_id.to_string()),
        });
    }
    if damage < hp {
        return Ok(ConsumeResult {
            survivors: vec![ResolvedTdLayer {
                state,
                hp: hp - damage,
                max_hp,
            }],
            popped: Vec::new(),
            remaining_damage: Fixed64::ZERO,
            immune_layer: None,
        });
    }

    let mut remaining_damage = damage - hp;
    let mut popped = vec![PoppedLayer {
        layer_id: layer_id.to_string(),
        cash: metadata.cash,
    }];
    let mut survivors = Vec::new();
    let mut immune_layer = None;
    for (child_index, child_id) in metadata.children.iter().enumerate() {
        let child = find_layer(catalog, child_id)?;
        let child_properties = inherited_properties(properties, child);
        let child_hp = layer_max_hp(child, child_properties);
        let child_lineage = child_lineage(lineage, child_index);
        if remaining_damage <= Fixed64::ZERO || immune_layer.is_some() {
            survivors.push(ResolvedTdLayer {
                state: state_for_layer(root_state, child, child_properties, child_lineage),
                hp: child_hp,
                max_hp: child_hp,
            });
            continue;
        }
        let result = consume_layer(
            catalog,
            root_state,
            child_id,
            child_hp,
            remaining_damage,
            profile,
            child_lineage,
        )?;
        remaining_damage = result.remaining_damage;
        popped.extend(result.popped);
        survivors.extend(result.survivors);
        immune_layer = result.immune_layer;
    }
    Ok(ConsumeResult {
        survivors,
        popped,
        remaining_damage,
        immune_layer,
    })
}

fn find_layer<'a>(
    catalog: &'a [TdLayerMetadataConst],
    id: &str,
) -> Result<&'a TdLayerMetadataConst, TdLayerResolveError> {
    catalog
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| TdLayerResolveError::MissingLayer(id.to_string()))
}

fn layer_max_hp(metadata: &TdLayerMetadataConst, properties: u32) -> Fixed64 {
    let fortified = properties & omoba_template_ids::td_rounds::layer_property::FORTIFIED != 0
        && metadata.fortified_eligible;
    Fixed64::from_i32(metadata.hp.saturating_mul(if fortified { 2 } else { 1 }) as i32)
}

fn inherited_properties(parent: u32, child: &TdLayerMetadataConst) -> u32 {
    let mut properties = child.properties;
    properties |= parent
        & (omoba_template_ids::td_rounds::layer_property::CAMO
            | omoba_template_ids::td_rounds::layer_property::MOAB_CLASS);
    if child.regrow_eligible {
        properties |= parent & omoba_template_ids::td_rounds::layer_property::REGROW;
    }
    if child.fortified_eligible {
        properties |= parent & omoba_template_ids::td_rounds::layer_property::FORTIFIED;
    }
    properties
}

fn state_for_layer(
    root: &TdLayerState,
    metadata: &TdLayerMetadataConst,
    properties: u32,
    lineage: u64,
) -> TdLayerState {
    TdLayerState {
        base_archetype: root.base_archetype.clone(),
        current_layer: metadata.id.to_string(),
        properties,
        regrow_ceiling: root.regrow_ceiling.clone(),
        regrow_elapsed: root.regrow_elapsed,
        remaining_leak_value: metadata.leak_value.saturating_mul(
            if properties & omoba_template_ids::td_rounds::layer_property::FORTIFIED != 0 {
                2
            } else {
                1
            },
        ),
        spawn_lineage: lineage,
    }
}

pub fn resolve_td_regrow_parent(
    catalog: &[TdLayerMetadataConst],
    state: &TdLayerState,
) -> Result<Option<ResolvedTdLayer>, TdLayerResolveError> {
    if state.properties & omoba_template_ids::td_rounds::layer_property::REGROW == 0
        || state.current_layer == state.regrow_ceiling
    {
        return Ok(None);
    }
    let mut path = Vec::new();
    if !find_path(
        catalog,
        &state.regrow_ceiling,
        &state.current_layer,
        &mut path,
    )? || path.len() < 2
    {
        return Ok(None);
    }
    let parent = find_layer(catalog, &path[path.len() - 2])?;
    let properties = inherited_properties(state.properties, parent);
    let mut next_state = state_for_layer(state, parent, properties, state.spawn_lineage);
    next_state.regrow_elapsed = state.regrow_elapsed;
    let max_hp = layer_max_hp(parent, properties);
    Ok(Some(ResolvedTdLayer {
        state: next_state,
        hp: max_hp,
        max_hp,
    }))
}

fn find_path(
    catalog: &[TdLayerMetadataConst],
    from: &str,
    target: &str,
    path: &mut Vec<String>,
) -> Result<bool, TdLayerResolveError> {
    let layer = find_layer(catalog, from)?;
    path.push(from.to_string());
    if from == target {
        return Ok(true);
    }
    for child in layer.children {
        if find_path(catalog, child, target, path)? {
            return Ok(true);
        }
    }
    path.pop();
    Ok(false)
}

fn child_lineage(parent: u64, child_index: usize) -> u64 {
    parent
        .wrapping_mul(0x100000001b3)
        .wrapping_add(child_index as u64 + 1)
}

fn is_compatible(metadata: &TdLayerMetadataConst, profile: DamageProfile) -> bool {
    let true_damage = omoba_template_ids::td_rounds::damage_profile::TRUE;
    profile.0 & true_damage != 0 || profile.0 & metadata.accepted_damage != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_template_ids::td_rounds::{damage_profile, layer_property};

    fn state(layer: &str, properties: u32) -> TdLayerState {
        let metadata = omoba_template_ids::td_layer_by_name(layer).unwrap();
        TdLayerState {
            base_archetype: layer.into(),
            current_layer: layer.into(),
            properties,
            regrow_ceiling: layer.into(),
            regrow_elapsed: Fixed64::ZERO,
            remaining_leak_value: metadata.leak_value,
            spawn_lineage: 42,
        }
    }

    fn hit(
        layer: &str,
        hp: i32,
        damage: i32,
        profile: u32,
    ) -> Result<TdLayerResolutionPlan, TdLayerResolveError> {
        resolve_td_layer_damage(
            omoba_template_ids::td_layer_catalog(),
            &state(layer, 0),
            Fixed64::from_i32(hp),
            Fixed64::from_i32(damage),
            DamageProfile(profile),
            HitProvenance {
                source_entity_id: 7,
                owner_player_id: Some(1),
                hit_serial: 9,
            },
        )
    }

    #[test]
    fn partial_and_exact_pop_have_no_transient_layer() {
        let partial = hit("ceramic", 10, 3, damage_profile::NORMAL).unwrap();
        assert!(partial.popped.is_empty());
        assert_eq!(partial.original.unwrap().hp, Fixed64::from_i32(7));

        let exact = hit("blue", 1, 1, damage_profile::NORMAL).unwrap();
        assert_eq!(
            exact
                .popped
                .iter()
                .map(|p| p.layer_id.as_str())
                .collect::<Vec<_>>(),
            ["blue"]
        );
        let survivor = exact.original.unwrap();
        assert_eq!(survivor.state.current_layer, "red");
        assert_eq!(survivor.hp, Fixed64::from_i32(1));
    }

    #[test]
    fn overkill_consumes_single_child_chain_in_one_plan() {
        let plan = hit("green", 1, 3, damage_profile::NORMAL).unwrap();
        assert_eq!(
            plan.popped
                .iter()
                .map(|p| p.layer_id.as_str())
                .collect::<Vec<_>>(),
            ["green", "blue", "red"]
        );
        assert!(plan.original.is_none());
        assert!(plan.children.is_empty());
        assert_eq!(plan.cash(), 3);
    }

    #[test]
    fn branch_damage_follows_authored_order_and_only_returns_survivors() {
        let exact = hit("black", 1, 1, damage_profile::NORMAL).unwrap();
        assert_eq!(exact.original.as_ref().unwrap().state.current_layer, "pink");
        assert_eq!(exact.children.len(), 1);
        assert_eq!(exact.children[0].state.current_layer, "pink");
        assert_ne!(
            exact.original.as_ref().unwrap().state.spawn_lineage,
            exact.children[0].state.spawn_lineage
        );

        let plan = hit("black", 1, 7, damage_profile::NORMAL).unwrap();
        assert_eq!(
            plan.popped
                .iter()
                .map(|layer| layer.layer_id.as_str())
                .collect::<Vec<_>>(),
            ["black", "pink", "yellow", "green", "blue", "red", "pink"]
        );
        let original = plan.original.as_ref().unwrap();
        assert_eq!(original.state.current_layer, "yellow");
        assert_eq!(original.hp, Fixed64::from_i32(1));
        assert!(plan.children.is_empty());
    }

    #[test]
    fn immune_zero_invalid_and_true_damage_are_explicit() {
        let immune = hit("lead", 1, 5, damage_profile::SHARP).unwrap();
        assert_eq!(immune.immune_layer.as_deref(), Some("lead"));
        assert!(immune.popped.is_empty());
        assert_eq!(immune.original.unwrap().hp, Fixed64::from_i32(1));

        let true_hit = hit("lead", 1, 1, damage_profile::TRUE).unwrap();
        assert_eq!(true_hit.popped[0].layer_id, "lead");

        let zero = hit("red", 1, 0, damage_profile::NORMAL).unwrap();
        assert!(zero.popped.is_empty());
        assert_eq!(zero.original.unwrap().hp, Fixed64::from_i32(1));

        assert_eq!(
            hit("red", 1, 1, 1 << 20),
            Err(TdLayerResolveError::InvalidDamageProfile(1 << 20))
        );
    }

    #[test]
    fn fortified_properties_and_repeat_order_are_deterministic() {
        let root = state("blue", layer_property::CAMO | layer_property::FORTIFIED);
        let run = || {
            resolve_td_layer_damage(
                omoba_template_ids::td_layer_catalog(),
                &root,
                Fixed64::from_i32(2),
                Fixed64::from_i32(2),
                DamageProfile(damage_profile::NORMAL),
                HitProvenance {
                    source_entity_id: 7,
                    owner_player_id: Some(1),
                    hit_serial: 9,
                },
            )
            .unwrap()
        };
        let first = run();
        let second = run();
        assert_eq!(first, second);
        let red = first.original.unwrap();
        assert_ne!(red.state.properties & layer_property::CAMO, 0);
        assert_ne!(red.state.properties & layer_property::FORTIFIED, 0);
        assert_eq!(red.max_hp, Fixed64::from_i32(2));
    }

    #[test]
    fn regrow_climbs_one_authored_parent_without_crossing_ceiling() {
        let mut state = state("green", 0);
        state.base_archetype = "yellow".into();
        state.regrow_ceiling = "yellow".into();
        state.properties = omoba_template_ids::td_rounds::layer_property::REGROW;
        let parent = resolve_td_regrow_parent(omoba_template_ids::td_layer_catalog(), &state)
            .unwrap()
            .expect("green regrows to yellow");
        assert_eq!(parent.state.current_layer, "yellow");
        assert_eq!(parent.hp, Fixed64::ONE);
        assert!(
            resolve_td_regrow_parent(omoba_template_ids::td_layer_catalog(), &parent.state)
                .unwrap()
                .is_none()
        );
    }
}
