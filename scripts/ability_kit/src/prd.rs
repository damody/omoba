//! v3 `chance { mode = "prd" }` (E505, 原作 PseudoRandom / Dota pseudo-random distribution): the
//! chance on the N-th try since the last success is `C × N`, with `C` chosen so the long-run rate
//! is the nominal `p`. The tries since the last success are a unit var of the caster (like a
//! chance's pity counter); `C` is computed in integers, so every world rolls the same.
use crate::codegen::{Cx, Res};
use omb_script_abi::types::Fixed64;

/// Fixed point of the computation: 2^30.
const ONE: u64 = 1 << 30;

/// The long-run success rate (in [`ONE`]) of a PRD with constant `c` (in [`ONE`]).
fn rate_of(c: u64) -> u64 {
    if c == 0 { return 0; }
    // P(first success on try N) = min(1, N c) × P(no success before N); the rate is 1 / E[N].
    let (mut none_yet, mut expected) = (ONE as u128, 0u128);
    let mut n = 1u128;
    while none_yet > 0 {
        let p_n = (n * c as u128).min(ONE as u128);
        let first = none_yet * p_n / ONE as u128;
        expected += n * first;
        none_yet -= first.min(none_yet);
        if p_n == ONE as u128 { break; }
        n += 1;
    }
    if expected == 0 { return ONE; }
    ((ONE as u128 * ONE as u128) / expected).min(ONE as u128) as u64
}

/// The PRD constant (in [`ONE`]) for a nominal chance `p` (in [`ONE`]), by bisection.
fn constant_of(p: u64) -> u64 {
    if p == 0 { return 0; }
    if p >= ONE { return ONE; }
    let (mut low, mut high) = (0u64, p);
    while high - low > 1 {
        let mid = low + (high - low) / 2;
        if rate_of(mid) < p { low = mid; } else { high = mid; }
    }
    high
}

impl Cx<'_, '_> {
    /// One PRD roll of nominal chance `p` whose misses count in the caster's var `key` (the
    /// generator's `__prd:<range>`; `hero`: kept across the hero's death like a pity counter).
    pub fn chance_prd(&mut self, key: &str, p: Fixed64, hero: bool) -> Result<bool, &'static str> {
        let nominal = (p.raw().clamp(0, 1024) as u64) << 20;
        let c = constant_of(nominal);
        let caster = self.caster;
        let tries = self.state_num(caster, key, Fixed64::ZERO) + Fixed64::ONE;
        let n = (tries.raw().max(1024) >> 10) as u64;
        let chance = Fixed64::from_raw((c.saturating_mul(n).min(ONE) >> 20) as i64);
        let hit = self.chance(chance);
        let res: Res = self.set_state_num(caster, key, if hit { Fixed64::ZERO } else { tries }, hero);
        res?;
        Ok(hit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The constants match the known Dota PRD values within a hair, and the long-run rate is the
    /// nominal chance.
    #[test]
    fn prd_constants_give_the_nominal_rate() {
        let at = |pct: u64| constant_of(ONE * pct / 100) as f64 / ONE as f64;
        // Dota 2 published C: 25% -> 0.0847, 40% -> 0.2015, 50% -> 0.3021.
        for (pct, c) in [(25, 0.0847), (40, 0.2015), (50, 0.3021)] {
            assert!((at(pct) - c).abs() < 0.001, "{pct}%: {} vs {c}", at(pct));
            let rate = rate_of(constant_of(ONE * pct / 100)) as f64 / ONE as f64;
            assert!((rate - pct as f64 / 100.0).abs() < 0.0005, "{pct}%: rate {rate}");
        }
        assert_eq!((constant_of(0), constant_of(ONE)), (0, ONE));
    }
}
