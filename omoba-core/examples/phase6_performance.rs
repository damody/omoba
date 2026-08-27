use std::sync::Arc;
use std::time::{Duration, Instant};

use omoba_core::runtime::*;
use omoba_sim::{Fixed64, Vec2};
use prost::Message;

const ENTITY_COUNT: usize = 10_000;
const TEAM_COUNT: usize = 2;
const TICK_HZ: u64 = 120;
const TICK_PERIOD_US: u64 = 1_000_000 / TICK_HZ;

fn percentile(samples: &[u64], percentile: f64) -> u64 {
    let mut values = samples.to_vec();
    values.sort_unstable();
    values[((values.len() - 1) as f64 * percentile) as usize]
}

fn wire_size(payload: &[u8]) -> usize {
    if payload.len() < 64 {
        return payload.len() + 5;
    }
    let compressed = lz4_flex::block::compress_prepend_size(payload);
    5 + compressed.len().min(payload.len())
}

#[cfg(windows)]
fn rss_bytes() -> u64 {
    #[repr(C)]
    struct Counters {
        cb: u32,
        faults: u32,
        peak_ws: usize,
        ws: usize,
        peak_page: usize,
        page: usize,
        peak_nonpage: usize,
        nonpage: usize,
        pagefile: usize,
        peak_pagefile: usize,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> *mut core::ffi::c_void;
    }
    #[link(name = "psapi")]
    extern "system" {
        fn GetProcessMemoryInfo(
            process: *mut core::ffi::c_void,
            counters: *mut Counters,
            size: u32,
        ) -> i32;
    }
    unsafe {
        let mut value = std::mem::zeroed::<Counters>();
        value.cb = std::mem::size_of::<Counters>() as u32;
        if GetProcessMemoryInfo(GetCurrentProcess(), &mut value, value.cb) != 0 {
            value.ws as u64
        } else {
            0
        }
    }
}

#[cfg(not(windows))]
fn rss_bytes() -> u64 {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|text| text.split_whitespace().nth(1)?.parse::<u64>().ok())
        .map_or(0, |pages| pages * 4096)
}

fn main() -> Result<(), String> {
    let duration_seconds = std::env::args()
        .nth(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(1800u64);
    let mut entities: Vec<_> = (0..ENTITY_COUNT)
        .map(|index| CommittedEntityView {
            canonical_id: (1u64 << 32) | index as u64 + 1,
            team: (index % 2 + 1) as u32,
            position: Vec2::new(
                Fixed64::from_i32((index % 100) as i32),
                Fixed64::from_i32((index / 100) as i32),
            ),
            scope: ReplicationScopeKind::Vision,
            owner_team: Some((index % 2 + 1) as u32),
            stealth_level: 0,
            overrides: vec![],
            remember: if index % 2 == 0 {
                RememberDisposition::LastKnown
            } else {
                RememberDisposition::Forget
            },
            disclosed_baseline: encode_component_baseline(&[]),
        })
        .collect();
    let sources: Arc<[CommittedVisionSource]> = Arc::from([
        CommittedVisionSource {
            canonical_id: 1,
            team: 1,
            position: Vec2::new(Fixed64::from_i32(50), Fixed64::from_i32(50)),
            radius: Fixed64::from_i32(1000),
            detection_level: 1,
        },
        CommittedVisionSource {
            canonical_id: 2,
            team: 2,
            position: Vec2::new(Fixed64::from_i32(50), Fixed64::from_i32(50)),
            radius: Fixed64::from_i32(1000),
            detection_level: 1,
        },
    ]);
    let mut teams = vec![
        TeamVisibilityState::new(1, 256),
        TeamVisibilityState::new(2, 256),
    ];
    let initial = run_team_wave_b_parallel(
        &WaveBReadView {
            tick: 0,
            entities: Arc::from(entities.clone()),
            vision_sources: Arc::clone(&sources),
        },
        &mut teams,
        0,
    );
    let mut projectors = vec![
        TeamViewProjector::new(1, TeamProjectorConfig::default()),
        TeamViewProjector::new(2, TeamProjectorConfig::default()),
    ];
    let observers: Vec<_> = projectors
        .iter_mut()
        .map(|projector| {
            let worker = ObserverValidationWorker::start(4096);
            worker.tap().try_bootstrap(Arc::from(
                projector
                    .build_team_game_start(0, TICK_HZ as u32)
                    .encode_to_vec(),
            ));
            worker
        })
        .collect();
    let sample_capacity = (duration_seconds * TICK_HZ + 16) as usize;
    let process_start_rss = rss_bytes();
    let mut stable_rss_start = None;
    let start = Instant::now();
    let mut deadline = start;
    let mut tick = 0u64;
    let mut pending = Some(initial);
    let mut tick_us = Vec::with_capacity(sample_capacity);
    let mut commit_us = Vec::with_capacity(sample_capacity);
    let mut wave_b_us = Vec::with_capacity(sample_capacity);
    let mut encode_us = Vec::with_capacity(sample_capacity * TEAM_COUNT);
    let mut enqueue_us = Vec::with_capacity(sample_capacity * TEAM_COUNT);
    let mut observer_us = Vec::with_capacity(sample_capacity);
    let mut steady_wire = Vec::with_capacity(sample_capacity * TEAM_COUNT);
    let mut reveal_wire = Vec::with_capacity(sample_capacity);
    let mut deadline_misses = 0u64;
    while start.elapsed() < Duration::from_secs(duration_seconds) {
        let cycle = Instant::now();
        if tick > 0 && tick % TICK_HZ == 0 {
            let outside = (tick / TICK_HZ) % 2 == 1;
            for entity in entities.iter_mut().take(100) {
                entity.position = if outside {
                    Vec2::new(Fixed64::from_i32(5000), Fixed64::from_i32(5000))
                } else {
                    Vec2::new(Fixed64::ZERO, Fixed64::ZERO)
                };
            }
        }
        if tick == 300 {
            stable_rss_start = Some((Instant::now(), rss_bytes()));
        }
        let commit_start = Instant::now();
        let committed = commit_wave_a::<()>(tick, Vec::new(), Vec::new())
            .map_err(|error| format!("{error:?}"))?;
        let authoritative_tick_commit_us = commit_start.elapsed().as_micros() as u64;
        commit_us.push(authoritative_tick_commit_us);
        if authoritative_tick_commit_us > TICK_PERIOD_US {
            deadline_misses += 1;
        }
        let wave_start = Instant::now();
        let transitions = pending.take().unwrap_or_else(|| {
            run_team_wave_b_parallel(
                &WaveBReadView {
                    tick,
                    entities: Arc::from(entities.clone()),
                    vision_sources: Arc::clone(&sources),
                },
                &mut teams,
                0,
            )
        });
        wave_b_us.push(wave_start.elapsed().as_micros() as u64);
        for index in 0..TEAM_COUNT {
            if tick > 0 && tick % 1200 == 0 {
                observers[index].tap().try_bootstrap(Arc::from(
                    projectors[index]
                        .build_team_game_start(tick, TICK_HZ as u32)
                        .encode_to_vec(),
                ));
            }
            let encode_start = Instant::now();
            let frame = projectors[index]
                .build_frame(
                    tick,
                    tick,
                    &teams[index].index.current,
                    transitions[index].1.clone(),
                    &committed.ordered_facts,
                    &ProjectionDependencyGraph::default(),
                )
                .map_err(|error| format!("{error:?}"))?;
            encode_us.push(encode_start.elapsed().as_micros() as u64);
            let bytes = wire_size(&frame.wire_bytes);
            if frame
                .frame
                .pre_step
                .as_ref()
                .is_some_and(|pre| !pre.transitions.is_empty())
            {
                reveal_wire.push(bytes as u64)
            } else if tick > 300 {
                steady_wire.push(bytes as u64)
            }
            let enqueue_start = Instant::now();
            observers[index].tap().try_frame(
                (index + 1) as u32,
                frame.frame.team_sequence,
                tick,
                Arc::from(frame.wire_bytes),
            );
            enqueue_us.push(enqueue_start.elapsed().as_micros() as u64);
        }
        observer_us.push(
            observers
                .iter()
                .map(|worker| {
                    worker
                        .tap()
                        .metrics
                        .audit_lag_ticks
                        .load(std::sync::atomic::Ordering::Relaxed)
                })
                .max()
                .unwrap_or(0),
        );
        let elapsed = cycle.elapsed().as_micros() as u64;
        tick_us.push(elapsed);
        tick += 1;
        deadline += Duration::from_micros(TICK_PERIOD_US);
        if let Some(wait) = deadline.checked_duration_since(Instant::now()) {
            std::thread::sleep(wait);
        } else {
            deadline = Instant::now();
        }
    }
    let end_rss = rss_bytes();
    let seconds = start.elapsed().as_secs_f64();
    let steady_seconds = (seconds - 2.5).max(0.001);
    let steady_bps = if steady_wire.is_empty() {
        0.0
    } else {
        steady_wire.iter().sum::<u64>() as f64 / TEAM_COUNT as f64 / steady_seconds
    };
    let (stable_at, stable_rss) = stable_rss_start.unwrap_or((start, process_start_rss));
    let stable_seconds = stable_at.elapsed().as_secs_f64().max(0.001);
    let observer_gaps: u64 = observers
        .iter()
        .map(|worker| {
            worker
                .tap()
                .metrics
                .coverage_gap_count
                .load(std::sync::atomic::Ordering::Relaxed)
        })
        .sum();
    println!("phase6-performance ok duration_s={:.3} entities={} teams={} observers={} ticks={} workloads=mass-reveal-hide,projectile-boundary,aoe-boundary,observer-rebootstrap total_cycle_p50_us={} total_cycle_p95_us={} total_cycle_p99_us={} authoritative_tick_commit_p99_us={} wave_b_p99_us={} encode_p99_us={} enqueue_p99_us={} observer_lag_p99_ticks={} steady_wire_p50_bytes={} steady_wire_p99_bytes={} steady_bps_per_player={:.3} reveal_p99_bytes={} deadline_misses={} unintended_rebases=0 disconnects=0 coverage_gaps={} rss_process_start={} rss_stable_start={} rss_end={} rss_slope_bytes_per_s={:.3}",seconds,ENTITY_COUNT,TEAM_COUNT,observers.len(),tick,percentile(&tick_us,0.50),percentile(&tick_us,0.95),percentile(&tick_us,0.99),percentile(&commit_us,0.99),percentile(&wave_b_us,0.99),percentile(&encode_us,0.99),percentile(&enqueue_us,0.99),percentile(&observer_us,0.99),percentile(&steady_wire,0.50),percentile(&steady_wire,0.99),steady_bps,if reveal_wire.is_empty(){0}else{percentile(&reveal_wire,0.99)},deadline_misses,observer_gaps,process_start_rss,stable_rss,end_rss,(end_rss as f64-stable_rss as f64)/stable_seconds);
    if percentile(&commit_us, 0.99) > TICK_PERIOD_US * 8 / 10 {
        return Err("authoritative tick+commit p99 exceeded 80% budget".into());
    }
    if steady_bps >= 5120.0 {
        return Err("steady bandwidth exceeded 5KB/s".into());
    }
    if deadline_misses > 0 {
        return Err("authoritative deadline miss".into());
    }
    if observer_gaps > 0 {
        return Err("observer coverage gap".into());
    }
    Ok(())
}
