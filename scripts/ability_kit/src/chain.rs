//! v3 `chain` (E528, 原作 a bounce that jumps target to target, each chain with its own hit table:
//! B28 鬼謀．蛇火). One chain is a token; each unit it reaches gets a hidden row
//! `__chainhit:<token>` for the time the chain can still run, so a later jump skips it. The rows
//! live in the host's `BuffStore` like every marker (the DLL keeps nothing), dispels leave `__`
//! rows alone and a row that runs out needs no cleanup. A row written in this run is not read back
//! until the run is committed: a chain's jumps come from later scheduled steps.
use crate::codegen::{Cx, Res};
use omb_script_abi::types::{EntityHandle, Fixed64};

/// Buff id prefix of a chain's hit rows.
pub const HIT_PREFIX: &str = "__chainhit:";

/// How a chain picks its next target among the units left (v3 `chain.order`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainOrder { Nearest, Random, Id }

impl Cx<'_, '_> {
    /// A new chain's token: a positive integer from the host RNG (carried in the chain's schedule
    /// args as a number).
    pub fn chain_token(&mut self) -> Fixed64 {
        let raw = (0..3).fold(0i64, |acc, _| (acc << 10) ^ (self.world.rand_unit().raw() & 0x3ff));
        Fixed64::from_i32(i32::try_from(raw.max(1)).unwrap_or(1))
    }

    fn chain_hit_id(token: Fixed64) -> String { format!("{HIT_PREFIX}{}", token.raw() >> 10) }

    /// `unit` was reached by chain `token`; the mark lasts `life` seconds (the time the chain can
    /// still jump).
    pub fn chain_mark(&mut self, token: Fixed64, unit: EntityHandle, life: Fixed64) -> Res {
        if !unit.is_valid() || life <= Fixed64::ZERO { return Ok(()); }
        self.world.add_stat_buff(unit, abi_stable::std_types::RStr::from_str(&Self::chain_hit_id(token)), life,
            abi_stable::std_types::RStr::from_str("{}"));
        Ok(())
    }

    /// Whether chain `token` has reached `unit` already.
    pub fn chain_hit(&self, token: Fixed64, unit: EntityHandle) -> bool {
        self.world.has_buff(unit, abi_stable::std_types::RStr::from_str(&Self::chain_hit_id(token)))
    }

    /// The next target of chain `token` from `from` among `candidates` (the selector's units, its
    /// `where` already applied): never `from`, and never a unit the chain reached unless
    /// `allow_repeat`; nearest to `from` (then entity id), a random one, or the lowest id.
    pub fn chain_next(&mut self, token: Fixed64, from: EntityHandle, candidates: Vec<EntityHandle>, order: ChainOrder,
        allow_repeat: bool) -> Option<EntityHandle> {
        let mut left: Vec<EntityHandle> = candidates.into_iter()
            .filter(|u| u.is_valid() && (u.id, u.gen) != (from.id, from.gen))
            .filter(|u| allow_repeat || !self.chain_hit(token, *u))
            .collect();
        left.sort_by_key(|u| (u.id, u.gen));
        left.dedup();
        if left.is_empty() { return None; }
        match order {
            ChainOrder::Id => left.first().copied(),
            ChainOrder::Random => {
                let n = Fixed64::from_i32(i32::try_from(left.len()).unwrap_or(i32::MAX));
                let i = usize::try_from((self.world.rand_unit() * n).raw() >> 10).unwrap_or(0).min(left.len() - 1);
                left.get(i).copied()
            }
            ChainOrder::Nearest => {
                let at = self.pos(from).ok()?;
                left.into_iter().filter_map(|u| self.pos(u).ok().map(|p| (u, (p - at).length_squared())))
                    .min_by(|(a, da), (b, db)| da.cmp(db).then((a.id, a.gen).cmp(&(b.id, b.gen)))).map(|(u, _)| u)
            }
        }
    }
}
