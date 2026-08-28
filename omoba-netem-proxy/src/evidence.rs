use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

use crate::{
    config::{ProxyConfig, RouteConfig},
    delay::{Direction, RouteId},
    profile::DelayProfile,
    queue::QueueMetrics,
    NetemError, Result, TOOL_VERSION,
};

#[derive(Clone, Debug, Serialize)]
pub struct ProfileSwitch {
    pub team_id: u32,
    pub profile: String,
    pub weights: [u64; 20],
    pub monotonic_ms: u64,
    pub authoritative_tick: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct RouteEvidence {
    pub team_id: u32,
    pub profile: String,
    pub weights: [u64; 20],
    pub client_to_server: DirectionEvidence,
    pub server_to_client: DirectionEvidence,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct DirectionEvidence {
    pub observed_histogram: [u64; 20],
    pub scheduled_delay_p50_ms: u64,
    pub scheduled_delay_p95_ms: u64,
    pub scheduled_delay_p99_ms: u64,
    pub scheduled_rtt_p50_ms: u64,
    pub scheduled_rtt_p95_ms: u64,
    pub scheduled_rtt_p99_ms: u64,
    pub scheduled_rtt_min_ms: u64,
    pub scheduled_rtt_max_ms: u64,
    pub release_lateness_p50_us: u64,
    pub release_lateness_p95_us: u64,
    pub release_lateness_p99_us: u64,
    pub reordered_datagrams: u64,
    pub released_datagrams: u64,
    pub packets_high_watermark: usize,
    pub bytes_high_watermark: usize,
}

#[derive(Debug, Serialize)]
pub struct ProxyEvidence {
    pub schema_version: u32,
    pub tool_version: String,
    pub rust_version: String,
    pub pid: u32,
    pub binary_path: PathBuf,
    pub binary_sha256: String,
    pub status: String,
    pub failure: Option<String>,
    pub config: serde_json::Value,
    pub routes: Vec<RouteEvidence>,
    pub profile_switches: Vec<ProfileSwitch>,
}

pub struct EvidenceState {
    pub switches: Vec<ProfileSwitch>,
    pub failure: Option<String>,
}
pub type SharedEvidence = Arc<Mutex<EvidenceState>>;

pub fn shared() -> SharedEvidence {
    Arc::new(Mutex::new(EvidenceState {
        switches: Vec::new(),
        failure: None,
    }))
}

pub async fn write(
    config: &ProxyConfig,
    state: &SharedEvidence,
    metrics: &[(RouteId, Direction, QueueMetrics)],
) -> Result<()> {
    fs::create_dir_all(&config.evidence_dir)
        .map_err(|error| NetemError::Evidence(error.to_string()))?;
    let binary_path =
        std::env::current_exe().map_err(|error| NetemError::Evidence(error.to_string()))?;
    let binary_sha256 = hash_file(&binary_path)?;
    let state = state.lock().await;
    let routes = config
        .routes
        .iter()
        .map(|route| route_evidence(route, metrics))
        .collect();
    let value = ProxyEvidence {
        schema_version: 1,
        tool_version: TOOL_VERSION.into(),
        rust_version: rust_version(),
        pid: std::process::id(),
        binary_path,
        binary_sha256,
        status: if state.failure.is_some() {
            "FAIL"
        } else {
            "PASS"
        }
        .into(),
        failure: state.failure.clone(),
        config: serde_json::to_value(config.sanitized())
            .map_err(|error| NetemError::Evidence(error.to_string()))?,
        routes,
        profile_switches: state.switches.clone(),
    };
    atomic_json(&config.evidence_dir.join("proxy-evidence.json"), &value)
}

fn route_evidence(
    route: &RouteConfig,
    metrics: &[(RouteId, Direction, QueueMetrics)],
) -> RouteEvidence {
    let route_id = if route.team_id == 1 {
        RouteId::Team1
    } else {
        RouteId::Team2
    };
    let get = |direction| {
        metrics
            .iter()
            .find(|(id, dir, _)| *id == route_id && *dir == direction)
            .map(|(_, _, m)| direction_evidence(m))
            .unwrap_or_default()
    };
    RouteEvidence {
        team_id: route.team_id,
        profile: route.initial_profile.name.clone(),
        weights: route.initial_profile.weights,
        client_to_server: get(Direction::ClientToServer),
        server_to_client: get(Direction::ServerToClient),
    }
}

fn direction_evidence(metrics: &QueueMetrics) -> DirectionEvidence {
    DirectionEvidence {
        observed_histogram: metrics.histogram,
        scheduled_delay_p50_ms: percentile(&metrics.scheduled_delay_ms, 50),
        scheduled_delay_p95_ms: percentile(&metrics.scheduled_delay_ms, 95),
        scheduled_delay_p99_ms: percentile(&metrics.scheduled_delay_ms, 99),
        scheduled_rtt_p50_ms: percentile(&metrics.scheduled_rtt_ms, 50),
        scheduled_rtt_p95_ms: percentile(&metrics.scheduled_rtt_ms, 95),
        scheduled_rtt_p99_ms: percentile(&metrics.scheduled_rtt_ms, 99),
        scheduled_rtt_min_ms: metrics
            .scheduled_rtt_ms
            .iter()
            .copied()
            .min()
            .map_or(0, u64::from),
        scheduled_rtt_max_ms: metrics
            .scheduled_rtt_ms
            .iter()
            .copied()
            .max()
            .map_or(0, u64::from),
        release_lateness_p50_us: percentile(&metrics.release_lateness_us, 50),
        release_lateness_p95_us: percentile(&metrics.release_lateness_us, 95),
        release_lateness_p99_us: percentile(&metrics.release_lateness_us, 99),
        reordered_datagrams: metrics.reordered,
        released_datagrams: metrics.released,
        packets_high_watermark: metrics.packets_high_watermark,
        bytes_high_watermark: metrics.bytes_high_watermark,
    }
}
fn percentile<T: Copy + Ord + Into<u64>>(values: &[T], pct: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[((sorted.len() - 1) * pct / 100).min(sorted.len() - 1)].into()
}
fn atomic_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let temp = path.with_extension("json.tmp");
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|e| NetemError::Evidence(e.to_string()))?;
    fs::write(&temp, bytes).map_err(|e| NetemError::Evidence(e.to_string()))?;
    fs::rename(temp, path).map_err(|e| NetemError::Evidence(e.to_string()))
}
fn hash_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|e| NetemError::Evidence(e.to_string()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn rust_version() -> String {
    option_env!("RUSTC_VERSION").unwrap_or("rust-1.95.0").into()
}

pub async fn record_switch(
    state: &SharedEvidence,
    team_id: u32,
    profile: &DelayProfile,
    monotonic_ms: u64,
    authoritative_tick: u64,
) {
    state.lock().await.switches.push(ProfileSwitch {
        team_id,
        profile: profile.name.clone(),
        weights: profile.weights,
        monotonic_ms,
        authoritative_tick,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn percentile_is_stable() {
        assert_eq!(percentile(&[9_u32, 1, 5, 3], 50), 3);
        assert_eq!(percentile(&[9_u32, 1, 5, 3], 99), 5);
        assert_eq!(percentile::<u32>(&[], 99), 0)
    }
    #[test]
    fn direction_evidence_contains_statistics_not_payload() {
        let mut metrics = QueueMetrics::default();
        metrics.scheduled_rtt_ms = vec![20, 100];
        metrics.scheduled_delay_ms = vec![10, 50];
        metrics.release_lateness_us = vec![1, 2];
        metrics.histogram[0] = 1;
        metrics.histogram[19] = 1;
        let json = serde_json::to_string(&direction_evidence(&metrics)).unwrap();
        assert!(json.contains("observed_histogram"));
        assert!(!json.contains("payload"))
    }
}
