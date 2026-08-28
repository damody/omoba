use std::time::{SystemTime, UNIX_EPOCH};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{profile::bucket_bounds, profile::DelayProfile, NetemError, Result};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum RouteId {
    Team1,
    Team2,
}

impl RouteId {
    pub fn team_id(self) -> u32 {
        match self {
            Self::Team1 => 1,
            Self::Team2 => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    ClientToServer,
    ServerToClient,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelaySample {
    pub bucket: usize,
    pub rtt_ms: u32,
    pub client_to_server_ms: u32,
    pub server_to_client_ms: u32,
}

pub struct DelaySampler {
    route: RouteId,
    upstream_rng: ChaCha12Rng,
    downstream_rng: ChaCha12Rng,
}

impl DelaySampler {
    pub fn new(test_seed: u64, route: RouteId) -> Self {
        Self {
            route,
            upstream_rng: ChaCha12Rng::from_seed(derive_seed(
                test_seed,
                route,
                Direction::ClientToServer,
            )),
            downstream_rng: ChaCha12Rng::from_seed(derive_seed(
                test_seed,
                route,
                Direction::ServerToClient,
            )),
        }
    }

    pub fn sample(&mut self, direction: Direction, profile: &DelayProfile) -> Result<DelaySample> {
        let rng = match direction {
            Direction::ClientToServer => &mut self.upstream_rng,
            Direction::ServerToClient => &mut self.downstream_rng,
        };
        let draw = rng.random_range(0..profile.total_weight);
        let mut cumulative = 0_u64;
        let bucket = profile
            .weights
            .iter()
            .position(|weight| {
                cumulative += *weight;
                draw < cumulative
            })
            .ok_or_else(|| NetemError::Config("weighted RTT draw failed".into()))?;
        let (low, high) = bucket_bounds(bucket)?;
        let rtt_ms = match profile.name.as_str() {
            "fixed-20" => 20,
            "fixed-60" => 60,
            "fixed-100" => 100,
            _ => rng.random_range(low..=high),
        };
        let ratio_percent = rng.random_range(35_u32..=65);
        let mut client_to_server_ms = rtt_ms.saturating_mul(ratio_percent) / 100;
        client_to_server_ms = client_to_server_ms.clamp(1, rtt_ms - 1);
        let server_to_client_ms = rtt_ms - client_to_server_ms;
        Ok(DelaySample {
            bucket,
            rtt_ms,
            client_to_server_ms,
            server_to_client_ms,
        })
    }

    pub fn route(&self) -> RouteId {
        self.route
    }
}

fn derive_seed(test_seed: u64, route: RouteId, direction: Direction) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"omoba/netem/delay-stream/v1");
    hash.update(test_seed.to_be_bytes());
    hash.update(route.team_id().to_be_bytes());
    hash.update([match direction {
        Direction::ClientToServer => 1,
        Direction::ServerToClient => 2,
    }]);
    hash.finalize().into()
}

pub fn auto_seed() -> u64 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    rand::random::<u64>() ^ time as u64 ^ (time >> 64) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_seed_replays_and_split_is_bounded() {
        let p = DelayProfile::named("uniform-20-100").unwrap();
        let run = || {
            let mut s = DelaySampler::new(77, RouteId::Team1);
            (0..100)
                .map(|_| s.sample(Direction::ClientToServer, &p).unwrap())
                .collect::<Vec<_>>()
        };
        let values = run();
        assert_eq!(values, run());
        for v in values {
            assert!((20..=100).contains(&v.rtt_ms));
            assert_eq!(v.client_to_server_ms + v.server_to_client_ms, v.rtt_ms);
            let ratio = v.client_to_server_ms * 100 / v.rtt_ms;
            assert!((34..=65).contains(&ratio))
        }
    }
    #[test]
    fn route_and_direction_streams_are_isolated() {
        let p = DelayProfile::named("uniform-20-100").unwrap();
        let mut a = DelaySampler::new(9, RouteId::Team1);
        let mut b = DelaySampler::new(9, RouteId::Team2);
        let a0 = a.sample(Direction::ClientToServer, &p).unwrap();
        for _ in 0..10 {
            a.sample(Direction::ServerToClient, &p).unwrap();
        }
        assert_eq!(a.sample(Direction::ClientToServer, &p).unwrap(), {
            let mut fresh = DelaySampler::new(9, RouteId::Team1);
            fresh.sample(Direction::ClientToServer, &p).unwrap();
            fresh.sample(Direction::ClientToServer, &p).unwrap()
        });
        assert_ne!(a0, b.sample(Direction::ClientToServer, &p).unwrap())
    }
}
