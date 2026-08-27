use std::sync::Arc;
use std::time::Instant;

use omoba_core::runtime::native::scene::import_map::{
    PointJD, VisionOccluderPolygonJD, VisionTreeJD,
};
use omoba_core::runtime::{
    run_team_wave_b_parallel, CommittedEntityView, CommittedVisionSource, RememberDisposition,
    ReplicationScopeKind, TeamVisibilityState, VisionOccluderSet, WaveBReadView,
};
use omoba_sim::{Fixed64, Vec2};

const SAMPLES: usize = 4_000;

fn point(x: f32, y: f32) -> PointJD {
    PointJD { X: x, Y: y }
}

fn build_occluders() -> Arc<[omoba_core::runtime::VisionOccluder]> {
    let trees: Vec<_> = (0..64)
        .map(|index| VisionTreeJD {
            StableId: index + 1,
            X: -875.0 + (index % 8) as f32 * 250.0,
            Y: -875.0 + (index / 8) as f32 * 250.0,
            Radius: if index % 3 == 0 { 82.0 } else { 62.0 },
        })
        .collect();
    let polygons = vec![
        VisionOccluderPolygonJD {
            StableId: 1001,
            Name: "square".into(),
            Points: vec![
                point(-170.0, -150.0),
                point(190.0, -150.0),
                point(190.0, 130.0),
                point(-170.0, 130.0),
            ],
        },
        VisionOccluderPolygonJD {
            StableId: 1002,
            Name: "west".into(),
            Points: vec![
                point(-980.0, 180.0),
                point(-560.0, 180.0),
                point(-560.0, 520.0),
                point(-720.0, 520.0),
                point(-720.0, 340.0),
                point(-980.0, 340.0),
            ],
        },
        VisionOccluderPolygonJD {
            StableId: 1003,
            Name: "east".into(),
            Points: vec![
                point(520.0, -560.0),
                point(980.0, -560.0),
                point(980.0, -360.0),
                point(700.0, -360.0),
                point(700.0, -160.0),
                point(520.0, -160.0),
            ],
        },
    ];
    VisionOccluderSet::from_descriptors(&trees, &polygons)
        .unwrap()
        .0
        .into()
}

fn build_view(occluders: Arc<[omoba_core::runtime::VisionOccluder]>) -> WaveBReadView {
    let entities: Vec<_> = (0..100)
        .map(|index| {
            let row = index / 10;
            let column = index % 10;
            CommittedEntityView {
                canonical_id: index as u64 + 1,
                team: (index % 3) as u32,
                position: Vec2::new(
                    Fixed64::from_i32(-990 + column * 220),
                    Fixed64::from_i32(-990 + row * 220),
                ),
                scope: ReplicationScopeKind::Vision,
                owner_team: None,
                stealth_level: 0,
                overrides: Vec::new(),
                remember: RememberDisposition::Forget,
                disclosed_baseline: vec![0; 25],
            }
        })
        .collect();
    let sources = vec![
        CommittedVisionSource {
            canonical_id: 1001,
            team: 1,
            position: Vec2::new(Fixed64::from_i32(-1320), Fixed64::from_i32(-1100)),
            radius: Fixed64::from_i32(700),
            detection_level: 0,
        },
        CommittedVisionSource {
            canonical_id: 1002,
            team: 2,
            position: Vec2::new(Fixed64::from_i32(1320), Fixed64::from_i32(1100)),
            radius: Fixed64::from_i32(700),
            detection_level: 0,
        },
    ];
    WaveBReadView {
        tick: 1,
        entities: entities.into(),
        vision_sources: sources.into(),
        vision_occluders: occluders,
    }
}

fn measure(view: &WaveBReadView) -> u128 {
    let mut teams = vec![
        TeamVisibilityState::new(1, 8),
        TeamVisibilityState::new(2, 8),
    ];
    for _ in 0..200 {
        let _ = run_team_wave_b_parallel(view, &mut teams, 0);
    }
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let started = Instant::now();
        std::hint::black_box(run_team_wave_b_parallel(view, &mut teams, 0));
        samples.push(started.elapsed().as_nanos());
    }
    samples.sort_unstable();
    samples[SAMPLES * 99 / 100]
}

fn main() {
    let baseline = measure(&build_view(Arc::from([])));
    let occluded = measure(&build_view(build_occluders()));
    println!("{{\"samples\":{SAMPLES},\"entities\":100,\"heroes\":2,\"trees\":64,\"polygons\":3,\"baseline_p99_ns\":{baseline},\"occluded_p99_ns\":{occluded},\"ratio\":{:.3},\"tick_budget_ns\":8333333}}", occluded as f64 / baseline.max(1) as f64);
    assert!(
        occluded <= baseline.saturating_mul(2),
        "occluded Wave B p99 exceeds 2x baseline"
    );
    assert!(
        occluded <= 8_333_333,
        "occluded Wave B p99 exceeds 8.33ms tick budget"
    );
}
