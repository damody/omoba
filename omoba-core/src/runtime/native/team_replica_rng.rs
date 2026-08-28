use rand::{RngCore, SeedableRng};
use rand_pcg::Pcg64Mcg;
use sha2::{Digest, Sha256};

/// Derives a fresh stream seed for one lockstep tick. Previous tick cursor
/// consumption cannot influence this value.
pub fn tick_seed(global_seed: u64, tick: u64) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"omoba/team-replica/tick-rng/v1");
    hasher.update(global_seed.to_be_bytes());
    hasher.update(tick.to_be_bytes());
    let digest = hasher.finalize();
    digest[..16].try_into().expect("SHA-256 prefix")
}

pub fn tick_random_u64(global_seed: u64, tick: u64, request_ordinal: u64) -> u64 {
    let mut rng = Pcg64Mcg::from_seed(tick_seed(global_seed, tick));
    rng.advance(u128::from(request_ordinal));
    rng.next_u64()
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RandomRequestKey {
    pub phase_ordinal: u16,
    pub stable_request_ordinal: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RandomRequest {
    pub key: RandomRequestKey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RandomAssignment {
    pub key: RandomRequestKey,
    pub value: u64,
}

/// World-local random barrier. Parallel systems may only append requests;
/// sorting and mutable RNG consumption happen serially at the barrier.
pub struct TickDeterministicRng {
    global_seed: u64,
    tick: u64,
    requests: Vec<RandomRequest>,
    assignments: Vec<RandomAssignment>,
}

impl TickDeterministicRng {
    pub fn new(global_seed: u64) -> Self {
        Self {
            global_seed,
            tick: 0,
            requests: Vec::new(),
            assignments: Vec::new(),
        }
    }

    pub fn begin_tick(&mut self, tick: u64) {
        self.tick = tick;
        self.requests.clear();
        self.assignments.clear();
    }

    pub fn request(&mut self, request: RandomRequest) {
        self.requests.push(request);
    }

    pub fn resolve(&mut self) -> &[RandomAssignment] {
        self.requests
            .sort_by(|left, right| left.key.cmp(&right.key));
        let mut rng = Pcg64Mcg::from_seed(tick_seed(self.global_seed, self.tick));
        self.assignments = self
            .requests
            .iter()
            .map(|request| RandomAssignment {
                key: request.key.clone(),
                value: rng.next_u64(),
            })
            .collect();
        &self.assignments
    }

    pub fn finish_tick(&mut self) {
        self.requests.clear();
        self.assignments.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_tick_and_request_order_produce_same_assignments() {
        let run = || {
            let mut rng = TickDeterministicRng::new(77);
            rng.begin_tick(900);
            rng.request(RandomRequest {
                key: RandomRequestKey {
                    phase_ordinal: 2,
                    stable_request_ordinal: 9,
                },
            });
            rng.request(RandomRequest {
                key: RandomRequestKey {
                    phase_ordinal: 1,
                    stable_request_ordinal: 4,
                },
            });
            rng.resolve().to_vec()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn previous_tick_consumption_does_not_change_next_tick_stream() {
        let mut reused = TickDeterministicRng::new(88);
        reused.begin_tick(1);
        for ordinal in 0..20 {
            reused.request(RandomRequest {
                key: RandomRequestKey {
                    phase_ordinal: 0,
                    stable_request_ordinal: ordinal,
                },
            });
        }
        reused.resolve();
        reused.begin_tick(2);
        reused.request(RandomRequest {
            key: RandomRequestKey {
                phase_ordinal: 0,
                stable_request_ordinal: 0,
            },
        });
        let reused_value = reused.resolve()[0].value;

        let mut fresh = TickDeterministicRng::new(88);
        fresh.begin_tick(2);
        fresh.request(RandomRequest {
            key: RandomRequestKey {
                phase_ordinal: 0,
                stable_request_ordinal: 0,
            },
        });
        assert_eq!(reused_value, fresh.resolve()[0].value);
    }
}
