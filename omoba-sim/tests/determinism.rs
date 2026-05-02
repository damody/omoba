//! Cross-platform determinism pin tests.
//!
//! Each `pin_hash` test below hashes the result of a known sequence of operations
//! and asserts that the hash matches a hard-coded constant. The constants are first
//! captured by running the test once with `assert_eq!(actual, 0)`, copying the
//! printed PIN HASH value, then locking it. Any future change to the underlying
//! type's arithmetic (Fixed64 ops, RNG impl, trig LUT, etc.) that alters the
//! sequence will fail this test loudly — preventing silent desync introduction.
//!
//! See `D:\omoba\docs\plans\2026-05-02-server-paced-lockstep-design.md` Phase 0
//! Task 0.5 for the rationale.

use omoba_sim::fixed::Fixed64;
use std::hash::{Hash, Hasher};

#[test]
fn fixed64_arithmetic_pin_hash() {
    let mut h = fxhash::FxHasher64::default();

    // 1000-step mixed-arithmetic walk. Inputs are deliberately chosen
    // to exercise positive/negative/perfect-square/coprime-divisor paths.
    let mut acc = Fixed64::from_raw(1);
    for i in 1..=1000 {
        let v = Fixed64::from_raw(i);
        acc = (acc + v) * Fixed64::from_raw(1003);
        acc = acc / Fixed64::from_raw(997);
        if i % 7 == 0 { acc = acc.sqrt(); }
        if i % 13 == 0 { acc = -acc; }
        acc.raw().hash(&mut h);
    }

    let actual = h.finish();
    println!("PIN HASH = {}", actual);
    // First run: assert_eq!(actual, 0) — capture printed value, then lock.
    // Re-baselined after Fixed64 backing widened from i32 → i64 (see fixed.rs header).
    assert_eq!(actual, 16455672772097321992u64);
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
    // Re-baselined after Fixed64 backing widened from i32 → i64.
    assert_eq!(actual, 7637377004829148672u64);
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

/// Composite interaction pin: RNG → Angle → sin → Fixed64 → Vec2 → hash.
/// Catches cross-module determinism regressions that intra-module pins miss
/// (e.g., someone renaming TAU_TICKS without re-capturing the trig pin).
#[test]
fn composite_rng_trig_fixed_vec2_pin_hash() {
    use omoba_sim::fixed::Fixed64;
    use omoba_sim::trig::{sin, cos, Angle, TAU_TICKS};
    use omoba_sim::rng::SimRng;
    use omoba_sim::vec2::Vec2;

    let mut h = fxhash::FxHasher64::default();
    let mut rng = SimRng::from_master(0x1234_5678_9ABC_DEF0, 7);
    let mut acc = Vec2::ZERO;

    for _ in 0..200 {
        let tick = (rng.next_u32() as i32).rem_euclid(TAU_TICKS);
        let a = Angle::from_ticks(tick);
        let r = Fixed64::from_raw(rng.next_u32() as i64);
        let step = Vec2::new(cos(a) * r, sin(a) * r);
        acc = acc + step;
        acc.x.raw().hash(&mut h);
        acc.y.raw().hash(&mut h);
    }

    let actual = h.finish();
    println!("COMPOSITE PIN HASH = {}", actual);
    // Re-baselined after Fixed64 backing widened from i32 → i64.
    assert_eq!(actual, 7681105393870857992u64);
}

#[test]
fn atan2_pin_hash() {
    use omoba_sim::fixed::Fixed64;
    use omoba_sim::trig::atan2;
    let mut h = fxhash::FxHasher64::default();

    // Sample atan2 across all 8 octants and a fine grid in Q1.
    let cardinals = [
        (0, 1), (1, 1), (1, 0), (1, -1),
        (0, -1), (-1, -1), (-1, 0), (-1, 1),
    ];
    for &(y, x) in cardinals.iter() {
        atan2(Fixed64::from_i32(y), Fixed64::from_i32(x)).ticks().hash(&mut h);
    }

    // Q1 grid: y ∈ [1, 100], x ∈ [1, 100] step 7 — 196 samples
    for y in (1..=100).step_by(7) {
        for x in (1..=100).step_by(7) {
            atan2(Fixed64::from_i32(y), Fixed64::from_i32(x)).ticks().hash(&mut h);
        }
    }

    let actual = h.finish();
    println!("ATAN2 PIN HASH = {}", actual);
    assert_eq!(actual, 14253043781236842372u64);
}

/// Snapshot wire format pin: serialize Fixed64 + Angle + Vec2 round-trip.
/// Locks bincode 1.x default config wire format for Phase 5 observer rejoin.
#[test]
fn snapshot_wire_format_pin_hash() {
    use omoba_sim::fixed::Fixed64;
    use omoba_sim::trig::Angle;
    use omoba_sim::vec2::Vec2;
    use omoba_sim::snapshot::serialize;

    #[derive(serde::Serialize)]
    struct Sample {
        f: Fixed64,
        a: Angle,
        v: Vec2,
    }

    let s = Sample {
        f: Fixed64::from_raw(123456),
        a: Angle::from_degrees_i32(73),
        v: Vec2::new(Fixed64::from_i32(-100), Fixed64::from_raw(987654)),
    };

    let bytes = serialize(&s).expect("serialize");
    let mut h = fxhash::FxHasher64::default();
    bytes.hash(&mut h);
    let actual = h.finish();
    println!("SNAPSHOT PIN HASH = {}", actual);
    // Re-baselined after Fixed64 backing widened from i32 → i64 (wire size doubled).
    assert_eq!(actual, 591207636033033545u64);
}

#[test]
fn angle_rotate_toward_pin_hash() {
    use omoba_sim::trig::{angle_rotate_toward, Angle, TAU_TICKS};
    let mut h = fxhash::FxHasher64::default();
    // 8 starting angles × 8 targets × 3 step sizes = 192 samples
    let starts = [0, TAU_TICKS / 8, TAU_TICKS / 4, 3 * TAU_TICKS / 8,
                  TAU_TICKS / 2, 5 * TAU_TICKS / 8, 3 * TAU_TICKS / 4, 7 * TAU_TICKS / 8];
    for &c in &starts {
        for &t in &starts {
            for &step in &[10, 100, 1000] {
                angle_rotate_toward(Angle::from_ticks(c), Angle::from_ticks(t), step).ticks().hash(&mut h);
            }
        }
    }
    let actual = h.finish();
    println!("ANGLE_ROTATE_TOWARD PIN HASH = {}", actual);
    assert_eq!(actual, 15173432368798260831u64);
}

#[test]
fn vec2_normalize_pin_hash() {
    use omoba_sim::vec2::Vec2;
    use omoba_sim::fixed::Fixed64;
    let mut h = fxhash::FxHasher64::default();
    // 15x15 = 225 sample points across the cardinal + off-axis grid
    for x in (-50..=50).step_by(7) {
        for y in (-50..=50).step_by(7) {
            let v = Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y));
            let n = v.normalized();
            n.x.raw().hash(&mut h);
            n.y.raw().hash(&mut h);
        }
    }
    let actual = h.finish();
    println!("VEC2_NORMALIZE PIN HASH = {}", actual);
    // Re-baselined after Fixed64 backing widened from i32 → i64.
    assert_eq!(actual, 9586711281260338778u64);
}
