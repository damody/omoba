//! Cross-platform determinism pin tests.
//!
//! Each `pin_hash` test below hashes the result of a known sequence of operations
//! and asserts that the hash matches a hard-coded constant. The constants are first
//! captured by running the test once with `assert_eq!(actual, 0)`, copying the
//! printed PIN HASH value, then locking it. Any future change to the underlying
//! type's arithmetic (Fixed32 ops, RNG impl, trig LUT, etc.) that alters the
//! sequence will fail this test loudly — preventing silent desync introduction.
//!
//! See `D:\omoba\docs\plans\2026-05-02-server-paced-lockstep-design.md` Phase 0
//! Task 0.5 for the rationale.

use omoba_sim::fixed::Fixed32;
use std::hash::{Hash, Hasher};

#[test]
fn fixed32_arithmetic_pin_hash() {
    let mut h = fxhash::FxHasher64::default();

    // 1000-step mixed-arithmetic walk. Inputs are deliberately chosen
    // to exercise positive/negative/perfect-square/coprime-divisor paths.
    let mut acc = Fixed32::from_raw(1);
    for i in 1..=1000 {
        let v = Fixed32::from_raw(i);
        acc = (acc + v) * Fixed32::from_raw(1003);
        acc = acc / Fixed32::from_raw(997);
        if i % 7 == 0 { acc = acc.sqrt(); }
        if i % 13 == 0 { acc = -acc; }
        acc.raw().hash(&mut h);
    }

    let actual = h.finish();
    println!("PIN HASH = {}", actual);
    // First run: assert_eq!(actual, 0) — capture printed value, then lock.
    assert_eq!(actual, 16173917078359596551u64);
}

#[test]
fn trig_lut_pin_hash() {
    use omoba_sim::trig::{sin, cos, Angle, TAU_TICKS};
    let mut h = fxhash::FxHasher64::default();
    for i in 0..TAU_TICKS {
        let a = Angle::from_ticks(i);
        sin(a).raw().hash(&mut h);
        cos(a).raw().hash(&mut h);
    }
    let actual = h.finish();
    println!("TRIG PIN HASH = {}", actual);
    assert_eq!(actual, 10864827002850446389u64);
}

#[test]
fn rng_sequence_pin_hash() {
    use omoba_sim::rng::SimRng;
    let mut h = fxhash::FxHasher64::default();
    let mut r = SimRng::from_master(0xDEAD_BEEF_CAFE_BABE, 12345);
    for _ in 0..10000 {
        r.next_u64().hash(&mut h);
    }
    let actual = h.finish();
    println!("RNG PIN HASH = {}", actual);
    assert_eq!(actual, 864202779384842305u64);
}

/// Composite interaction pin: RNG → Angle → sin → Fixed32 → Vec2 → hash.
/// Catches cross-module determinism regressions that intra-module pins miss
/// (e.g., someone renaming TAU_TICKS without re-capturing the trig pin).
#[test]
fn composite_rng_trig_fixed_vec2_pin_hash() {
    use omoba_sim::fixed::Fixed32;
    use omoba_sim::trig::{sin, cos, Angle, TAU_TICKS};
    use omoba_sim::rng::SimRng;
    use omoba_sim::vec2::Vec2;

    let mut h = fxhash::FxHasher64::default();
    let mut rng = SimRng::from_master(0x1234_5678_9ABC_DEF0, 7);
    let mut acc = Vec2::ZERO;

    for _ in 0..200 {
        let tick = (rng.next_u32() as i32).rem_euclid(TAU_TICKS);
        let a = Angle::from_ticks(tick);
        let r = Fixed32::from_raw(rng.next_u32() as i32);
        let step = Vec2::new(cos(a) * r, sin(a) * r);
        acc = acc + step;
        acc.x.raw().hash(&mut h);
        acc.y.raw().hash(&mut h);
    }

    let actual = h.finish();
    println!("COMPOSITE PIN HASH = {}", actual);
    assert_eq!(actual, 4431538388923105100u64);
}

/// Snapshot wire format pin: serialize Fixed32 + Angle + Vec2 round-trip.
/// Locks bincode 1.x default config wire format for Phase 5 observer rejoin.
#[test]
fn snapshot_wire_format_pin_hash() {
    use omoba_sim::fixed::Fixed32;
    use omoba_sim::trig::Angle;
    use omoba_sim::vec2::Vec2;
    use omoba_sim::snapshot::serialize;

    #[derive(serde::Serialize)]
    struct Sample {
        f: Fixed32,
        a: Angle,
        v: Vec2,
    }

    let s = Sample {
        f: Fixed32::from_raw(123456),
        a: Angle::from_degrees_i32(73),
        v: Vec2::new(Fixed32::from_i32(-100), Fixed32::from_raw(987654)),
    };

    let bytes = serialize(&s).expect("serialize");
    let mut h = fxhash::FxHasher64::default();
    bytes.hash(&mut h);
    let actual = h.finish();
    println!("SNAPSHOT PIN HASH = {}", actual);
    assert_eq!(actual, 18245341913185412542u64);
}
