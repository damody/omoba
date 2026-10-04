//! Public compiled map identity; never contains live units, aggro, or timers.
use crate::game_proto::DeterministicMetadata;
use omoba_template_ids::{MobaMapConst, MOBA_MAP_CATALOG_HASH};

pub const NAMESPACE: &str = "map";
pub const KEY: &str = "moba-layout";

pub fn encode(map: &MobaMapConst) -> DeterministicMetadata {
    DeterministicMetadata {
        namespace: NAMESPACE.into(),
        key: KEY.into(),
        schema_version: 1,
        value: serde_json::to_vec(&(map.id, MOBA_MAP_CATALOG_HASH)).expect("map identity JSON"),
    }
}

/// Empty first-join bootstrap has no ECS yet, but the same immutable compiled
/// geometry must already be installed before the first accepted MoveTo.
pub fn bootstrap_metadata(map: &MobaMapConst) -> Vec<DeterministicMetadata> {
    let regions = compiled_blocked_regions(map);
    vec![
        encode(map),
        DeterministicMetadata {
            namespace: crate::runtime::PUBLIC_BLOCKED_REGIONS_NAMESPACE.into(),
            key: crate::runtime::PUBLIC_BLOCKED_REGIONS_KEY.into(),
            schema_version: 1,
            value: crate::runtime::encode_public_blocked_regions(&regions),
        },
    ]
}

/// The only compiled-map to runtime collision conversion. Authority, first join
/// and replica agreement must share corner order and float boundary conversion.
pub fn compiled_blocked_regions(map: &MobaMapConst) -> crate::runtime::BlockedRegions {
    use crate::runtime::{BlockedRegion, BlockedRegions};
    BlockedRegions(
        map.terrain
            .iter()
            .map(|r| BlockedRegion {
                name: r.id.into(),
                points: vec![
                    vek::Vec2::new(r.min.0 as f32, r.min.1 as f32),
                    vek::Vec2::new(r.max.0 as f32, r.min.1 as f32),
                    vek::Vec2::new(r.max.0 as f32, r.max.1 as f32),
                    vek::Vec2::new(r.min.0 as f32, r.max.1 as f32),
                ],
            })
            .collect(),
    )
}

/// A valid map ID/hash alone does not prove that navigation received the same
/// geometry. Compare canonical bytes before decoding or allocating untrusted
/// counts. Legacy non-compiled maps retain their existing metadata contract.
pub fn validate_metadata(
    metadata: &[DeterministicMetadata],
) -> Result<Option<&'static MobaMapConst>, &'static str> {
    let Some(map) = decode(metadata)? else {
        return Ok(None);
    };
    let mut entries = metadata.iter().filter(|m| {
        m.namespace == crate::runtime::PUBLIC_BLOCKED_REGIONS_NAMESPACE
            && m.key == crate::runtime::PUBLIC_BLOCKED_REGIONS_KEY
    });
    let region = entries.next().ok_or("missing compiled MOBA terrain")?;
    if entries.next().is_some() || region.schema_version != 1 {
        return Err("invalid compiled MOBA terrain metadata");
    }
    if region.value != crate::runtime::encode_public_blocked_regions(&compiled_blocked_regions(map))
    {
        return Err("compiled MOBA terrain mismatch");
    }
    Ok(Some(map))
}

pub fn resolve(id: &str, hash: &str) -> Result<Option<&'static MobaMapConst>, &'static str> {
    if id.is_empty() && hash.is_empty() {
        return Ok(None);
    }
    if id.len() > 64 || hash != MOBA_MAP_CATALOG_HASH {
        return Err("MOBA map catalog mismatch");
    }
    omoba_template_ids::moba_map_by_name(id)
        .map(Some)
        .ok_or("unknown MOBA map")
}

pub fn decode(
    metadata: &[DeterministicMetadata],
) -> Result<Option<&'static MobaMapConst>, &'static str> {
    let mut matches = metadata
        .iter()
        .filter(|m| m.namespace == NAMESPACE && m.key == KEY);
    let Some(m) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() || m.schema_version != 1 || m.value.len() > 256 {
        return Err("invalid public MOBA map metadata");
    }
    let (id, hash): (String, String) =
        serde_json::from_slice(&m.value).map_err(|_| "invalid MOBA map identity")?;
    // A present entry must name a compiled map; omission is the legacy contract.
    resolve(&id, &hash)?
        .map(Some)
        .ok_or("empty public MOBA map identity")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn geometry_conversion_is_data_driven_not_map_name_specific() {
        use omoba_template_ids::MobaTerrainConst;
        let map = MobaMapConst {
            id: "not_a_shipped_map",
            lane_length: 900,
            tower_offset: 100,
            lanes: &[],
            jungle_camps: &[],
            terrain: &[
                MobaTerrainConst {
                    id: "negative",
                    min: (-90, -60),
                    max: (-30, -10),
                },
                MobaTerrainConst {
                    id: "positive",
                    min: (70, 80),
                    max: (100, 120),
                },
            ],
        };
        let regions = compiled_blocked_regions(&map);
        assert_eq!(regions.0.len(), 2);
        assert_eq!(regions.0[0].name, "negative");
        assert_eq!(
            regions.0[0].points,
            vec![
                vek::Vec2::new(-90., -60.),
                vek::Vec2::new(-30., -60.),
                vek::Vec2::new(-30., -10.),
                vek::Vec2::new(-90., -10.)
            ]
        );
        assert_eq!(regions.0[1].points[2], vek::Vec2::new(100., 120.));
        assert!(compiled_blocked_regions(&MobaMapConst {
            terrain: &[],
            ..map
        })
        .0
        .is_empty());
    }
    #[test]
    fn compiled_geometry_agreement_rejects_missing_duplicate_and_mutated_terrain() {
        let map = omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        let metadata = bootstrap_metadata(map);
        assert_eq!(validate_metadata(&metadata).unwrap().unwrap().id, map.id);
        assert!(validate_metadata(&[]).unwrap().is_none());
        assert!(validate_metadata(&metadata[..1]).is_err());
        let mut reordered = metadata.clone();
        reordered.reverse();
        assert!(validate_metadata(&reordered).is_ok());
        let mut duplicate = metadata.clone();
        duplicate.push(metadata[1].clone());
        assert!(validate_metadata(&duplicate).is_err());
        let mut wrong_schema = metadata.clone();
        wrong_schema[1].schema_version = 2;
        assert!(validate_metadata(&wrong_schema).is_err());
        for value in [vec![], u32::MAX.to_be_bytes().to_vec(), vec![0; 4]] {
            let mut corrupt = metadata.clone();
            corrupt[1].value = value;
            assert!(validate_metadata(&corrupt).is_err());
        }
        let mut corrupt = metadata.clone();
        let last = corrupt[1].value.len() - 1;
        corrupt[1].value[last] ^= 1;
        assert!(validate_metadata(&corrupt).is_err());
    }
    #[test]
    fn compiled_map_identity_round_trip_and_fail_closed() {
        let map = omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        let m = encode(map);
        let initial = bootstrap_metadata(map);
        assert_eq!(decode(&initial).unwrap().unwrap().id, map.id);
        let regions = crate::runtime::decode_public_blocked_regions(&initial[1].value).unwrap();
        assert_eq!(regions.0.len(), map.terrain.len());
        assert_eq!(decode(&[m.clone()]).unwrap().unwrap().id, map.id);
        assert!(decode(&[]).unwrap().is_none());
        assert!(resolve("", "").unwrap().is_none());
        assert!(resolve(map.id, "wrong-hash").is_err());
        assert!(resolve("unknown", MOBA_MAP_CATALOG_HASH).is_err());
        assert!(resolve("", MOBA_MAP_CATALOG_HASH).is_err());
        assert!(decode(&[m.clone(), m.clone()]).is_err());
        for value in [b"invalid".to_vec(), vec![0; 257], b"[\"\",\"\"]".to_vec()] {
            assert!(decode(&[DeterministicMetadata { value, ..m.clone() }]).is_err());
        }
        assert!(decode(&[DeterministicMetadata {
            schema_version: 2,
            ..m
        }])
        .is_err());
    }
}
