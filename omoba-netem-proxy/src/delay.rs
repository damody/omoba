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
    // Separate streams: enabling loss never changes the delay sequence.
    upstream_loss_rng: ChaCha12Rng,
    downstream_loss_rng: ChaCha12Rng,
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
            upstream_loss_rng: ChaCha12Rng::from_seed(derive_loss_seed(test_seed, route, Direction::ClientToServer)),
            downstream_loss_rng: ChaCha12Rng::from_seed(derive_loss_seed(test_seed, route, Direction::ServerToClient)),
        }
    }

    /// True when this datagram is lost under the profile's loss rate.
    pub fn drops(&mut self, direction: Direction, profile: &DelayProfile) -> bool {
        if profile.loss_ppm == 0 {
            return false;
        }
        let rng = match direction {
            Direction::ClientToServer => &mut self.upstream_loss_rng,
            Direction::ServerToClient => &mut self.downstream_loss_rng,
        };
        rng.random_range(0..1_000_000_u32) < profile.loss_ppm
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

fn derive_loss_seed(test_seed: u64, route: RouteId, direction: Direction) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"omoba/netem/loss-stream/v1");
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
    fn loss_is_seeded_bounded_and_independent_of_delay() {
        let lossy = DelayProfile::named("wan-80-20-loss1").unwrap();
        assert_eq!(lossy.loss_ppm, 10_000);
        let count = |seed: u64| {
            let mut s = DelaySampler::new(seed, RouteId::Team1);
            (0..100_000).filter(|_| s.drops(Direction::ClientToServer, &lossy)).count()
        };
        let dropped = count(5);
        assert_eq!(dropped, count(5));
        assert!((800..=1_200).contains(&dropped), "dropped {dropped}");
        for _ in 0..1000 {
            let sample = DelaySampler::new(3, RouteId::Team2).sample(Direction::ServerToClient, &lossy).unwrap();
            assert!((60..=100).contains(&sample.rtt_ms));
        }
        // Delay draws are identical whether or not loss draws were taken.
        let mut with_loss = DelaySampler::new(11, RouteId::Team1);
        let mut without = DelaySampler::new(11, RouteId::Team1);
        for _ in 0..50 {
            with_loss.drops(Direction::ClientToServer, &lossy);
            assert_eq!(with_loss.sample(Direction::ClientToServer, &lossy).unwrap(),
                without.sample(Direction::ClientToServer, &lossy).unwrap());
        }
        let lossless = DelayProfile::named("uniform-20-100").unwrap();
        let mut s = DelaySampler::new(1, RouteId::Team1);
        assert!((0..10_000).all(|_| !s.drops(Direction::ServerToClient, &lossless)));
        assert!(DelayProfile::named("fixed-20").unwrap().with_loss_ppm(200_001).is_err());
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
