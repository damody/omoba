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

/// Same authored wall bounds as collision and presentation. IDs are local to
/// this immutable map and assigned by name, independent of authoring order.
pub fn compiled_vision_occluders(map: &MobaMapConst) -> crate::runtime::VisionOccluderSet {
    use crate::runtime::{VisionAabb, VisionOccluder, VisionOccluderSet, VisionTerrainPolygon};
    use omoba_sim::{Fixed64, Vec2};
    let point=|(x,y)| Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
    let mut walls:Vec<_>=map.terrain.iter().collect();
    walls.sort_by_key(|wall|wall.id);
    VisionOccluderSet(walls.into_iter().enumerate().map(|(index,wall)| {
        VisionOccluder::Terrain(VisionTerrainPolygon {
            stable_id:index as u64+1,name:wall.id.into(),
            vertices:vec![point(wall.min),point((wall.max.0,wall.min.1)),point(wall.max),point((wall.min.0,wall.max.1))],
            aabb:VisionAabb {min:point(wall.min),max:point(wall.max)},
        })
    }).collect())
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
    /// E070: every compiled map's public lane routes are walkable by a hero under
    /// the exact runtime collision and the server's own MoveTo planner — each
    /// point stands clear and each next point is reachable from the previous one.
    #[test]
    fn every_compiled_lane_route_is_reachable_by_a_hero() {
        use omoba_sim::{Fixed64,Vec2};
        let radius=Fixed64::from_i32(30);
        let p=|(x,y):(i32,i32)|Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        assert!(omoba_template_ids::MOBA_MAPS.len()>=3,"training, layered and quick maps are compiled");
        for map in omoba_template_ids::MOBA_MAPS {
            let regions=compiled_blocked_regions(map);
            for lane in map.lanes {
                for (index,point) in lane.waypoints.iter().enumerate() {
                    assert!(!crate::runtime::native::tick::hero_move_tick::path_hits_regions(p(*point),p(*point),radius,&regions),
                        "map {} lane {} point {} {:?} is inside terrain",map.id,lane.id,index,point);
                }
                for (index,edge) in lane.waypoints.windows(2).enumerate() {
                    assert!(crate::runtime::native::tick::hero_command_tick::static_next_waypoint(p(edge[0]),p(edge[1]),radius,&regions).is_some(),
                        "map {} lane {} segment {} {:?}->{:?} is unreachable for a hero",map.id,lane.id,index,edge[0],edge[1]);
                }
            }
        }
        // The check is real: a wall on a route point is caught.
        let map=&omoba_template_ids::MOBA_MAPS[0];
        let (x,y)=map.lanes[0].waypoints[1];
        const WALL:[omoba_template_ids::MobaTerrainConst;1]=[omoba_template_ids::MobaTerrainConst {id:"probe",min:(0,0),max:(0,0)}];
        let mut walled=WALL;walled[0].min=(x-50,y-50);walled[0].max=(x+50,y+50);
        let leaked:&'static [omoba_template_ids::MobaTerrainConst]=Box::leak(Box::new(walled));
        let regions=compiled_blocked_regions(&MobaMapConst {terrain:leaked,..*map});
        assert!(crate::runtime::native::tick::hero_move_tick::path_hits_regions(p((x,y)),p((x,y)),radius,&regions));
    }
    #[test]
    fn compiled_wall_vision_matches_collision_is_order_independent_and_blocks_segments() {
        use crate::runtime::{LosResult,VisionOccluder};
        use omoba_sim::{Fixed64,Vec2};
        use omoba_template_ids::MobaTerrainConst;
        const WALLS:[MobaTerrainConst;2]=[MobaTerrainConst {id:"east",min:(20,-10),max:(30,10)},
            MobaTerrainConst {id:"west",min:(-30,-10),max:(-20,10)}];
        let map=MobaMapConst {id:"custom_geometry",terrain:&WALLS,..omoba_template_ids::MOBA_MAPS[0]};
        let set=compiled_vision_occluders(&map);
        const REVERSED:[MobaTerrainConst;2]=[WALLS[1],WALLS[0]];
        assert_eq!(set,compiled_vision_occluders(&MobaMapConst {terrain:&REVERSED,..map}));
        let p=|x,y|Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        for occluder in &set.0 {
            let VisionOccluder::Terrain(poly)=occluder else {panic!("compiled wall must be terrain")};
            let collision=compiled_blocked_regions(&map).0.into_iter().find(|r|r.name==poly.name).unwrap();
            assert_eq!(poly.vertices.iter().map(|p|(p.x.to_f32_for_render(),p.y.to_f32_for_render())).collect::<Vec<_>>(),
                collision.points.iter().map(|p|(p.x,p.y)).collect::<Vec<_>>());
        }
        assert_eq!(set.line_of_sight(p(0,0),p(40,0)),LosResult::Blocked);
        assert_eq!(set.line_of_sight(p(0,10),p(40,10)),LosResult::Blocked,"tangent edge is closed");
        assert_eq!(set.line_of_sight(p(0,11),p(40,11)),LosResult::Clear);
        assert_eq!(set.line_of_sight(p(0,0),p(10,0)),LosResult::Clear);
        assert!(compiled_vision_occluders(&MobaMapConst {terrain:&[],..map}).0.is_empty());
    }

    #[test]
    fn compiled_map_boot_installs_own_vision_and_preserves_legacy_campaign_policy() {
        use specs::WorldExt;
        use crate::runtime::{StateInitializer,VisionOccluderSet};
        use crate::runtime::native::moba_match::{setup_single_lane_match,SingleLaneConfig};
        let pool=StateInitializer::create_thread_pool();
        for map in omoba_template_ids::MOBA_MAPS {
            let mut world=StateInitializer::setup_campaign_ecs_world(&pool);
            setup_single_lane_match(&mut world,SingleLaneConfig {map_id:Some(map.id.into()),
                lane_length:omoba_sim::Fixed64::from_i32(map.lane_length),..SingleLaneConfig::default()}).unwrap();
            assert_eq!(*world.read_resource::<VisionOccluderSet>(),compiled_vision_occluders(map));
            let view=crate::runtime::build_wave_b_read_view(&world,1);
            assert_eq!(view.vision_occluders.len(),map.terrain.len());
        }
        let map=&omoba_template_ids::MOBA_MAPS[0];
        let mut legacy=StateInitializer::setup_campaign_ecs_world(&pool);
        let authored=compiled_vision_occluders(map);legacy.insert(authored.clone());
        setup_single_lane_match(&mut legacy,SingleLaneConfig::default()).unwrap();
        assert_eq!(*legacy.read_resource::<VisionOccluderSet>(),authored,"legacy keeps descriptor geometry");
    }
    #[test]
    fn geometry_conversion_is_data_driven_not_map_name_specific() {
        use omoba_template_ids::MobaTerrainConst;
        let map = MobaMapConst {
            base_structures: None, base_unlock: omoba_template_ids::MobaBaseUnlockConst::AllLaneTowers,
            minion_wave: &[],wave_interval_seconds:12,
            id: "not_a_shipped_map",
            lane_length: 900,
            tower_hp: 1200,
            base_hp: 1600,
            tower_offset: 100,
            tower_layers: &[100],
            lanes: &[],
            jungle_camps: &[],
            presentation_surfaces: &[],
            brushes: &[],
            bounds: None, world_bounds: None, minimap_bounds: None,
            towers: &[], barracks: &[], general_towers: &[], bases: &[], fountains: &[], hero_spawns: &[],
            shops: &[], shop_radius: 0,
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
            brushes: &[],
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
