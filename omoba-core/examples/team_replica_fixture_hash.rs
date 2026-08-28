use omoba_core::runtime::{tick_random_u64, tick_seed};
use sha2::{Digest, Sha256};

fn main() {
    let global_seed = 0x4f4d_4f42_415f_5631_u64;
    let mut fixture = Sha256::new();
    fixture.update(b"omoba/team-replica/cross-platform-fixture/v1");
    for tick in 0..4096_u64 {
        fixture.update(tick.to_be_bytes());
        fixture.update(tick_seed(global_seed, tick));
        for ordinal in 0..8_u64 {
            fixture.update(tick_random_u64(global_seed, tick, ordinal).to_be_bytes());
        }
    }
    println!("{:x}", fixture.finalize());
}
