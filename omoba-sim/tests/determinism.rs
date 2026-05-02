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
