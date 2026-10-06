//! Bounded read-only counters for one dispatcher invocation.
//! Job body times may overlap and omit fetch/scheduling/metrics bookkeeping.
//! They must never be subtracted from dispatcher wall time as "wait time".
use specs::{World, WorldExt};
use super::{System, TickProfile, tick_profile::VariantStat};
use crate::tick::*;

pub const JOB_NAMES: [&str; 18] = [
    <player_input_tick::Sys as System>::NAME, <nearby_tick::Sys as System>::NAME,
    <player_tick::Sys as System>::NAME, <demo_patrol_tick::Sys as System>::NAME,
    <projectile_tick::Sys as System>::NAME, <tower_tick::Sys as System>::NAME,
    <hero_command_tick::Sys as System>::NAME, <hero_move_tick::Sys as System>::NAME,
    <hero_tick::Sys as System>::NAME, <item_tick::Sys as System>::NAME,
    <buff_tick::Sys as System>::NAME, <td_regrow_tick::Sys as System>::NAME,
    <regen_tick::Sys as System>::NAME, <summon_tick::Sys as System>::NAME,
    <creep_tick::Sys as System>::NAME, <creep_wave::Sys as System>::NAME,
    <damage_tick::Sys as System>::NAME, <death_tick::Sys as System>::NAME,
];

#[derive(Clone, Copy)]
pub struct JobSnapshot([VariantStat; JOB_NAMES.len()]);

impl JobSnapshot {
    pub fn capture(world: &World) -> Option<Self> {
        let profile = world.read_resource::<TickProfile>();
        let stats = profile.system_stats.lock().ok()?;
        // New/untracked systems make this diagnostic explicitly unavailable,
        // rather than silently treating an incomplete table as all jobs.
        if stats.keys().any(|key| !JOB_NAMES.contains(key)) { return None; }
        let mut rows = [VariantStat::default(); JOB_NAMES.len()];
        for (i, name) in JOB_NAMES.iter().enumerate() {
            rows[i] = stats.get(name).copied().unwrap_or_default();
        }
        Some(Self(rows))
    }

    pub fn delta(self, after: Self) -> Option<JobBodySummary> {
        let mut summary = JobBodySummary::default();
        for i in 0..JOB_NAMES.len() {
            let count = after.0[i].count.checked_sub(self.0[i].count)?;
            let ns = after.0[i].ns.checked_sub(self.0[i].ns)?;
            if count > 1 || (count == 0 && ns != 0) { return None; }
            if count == 1 {
                summary.observed_jobs += 1;
                summary.body_sum_ns = summary.body_sum_ns.checked_add(ns)?;
                if summary.slowest_name.is_none() || ns > summary.slowest_body_ns {
                    summary.slowest_name = Some(JOB_NAMES[i]);
                    summary.slowest_body_ns = ns;
                }
            }
        }
        Some(summary)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JobBodySummary {
    pub observed_jobs: u32,
    pub body_sum_ns: u128,
    pub slowest_name: Option<&'static str>,
    pub slowest_body_ns: u128,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DispatcherDetail {
    pub input_ns: u128,
    pub build_ns: u128,
    pub execute_ns: u128,
    pub jobs: Option<JobBodySummary>,
}

impl DispatcherDetail {
    pub fn residual_ns(&self, wall_ns: u128) -> Option<u128> {
        wall_ns.checked_sub(self.input_ns.checked_add(self.build_ns)?.checked_add(self.execute_ns)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dispatcher_detail_checked_delta_preserves_parallel_overlap_and_ties() {
        let before = JobSnapshot([VariantStat::default(); JOB_NAMES.len()]);
        let mut after = before;
        after.0[1] = VariantStat { count: 1, ns: 100 };
        after.0[2] = VariantStat { count: 1, ns: 100 };
        let summary = before.delta(after).unwrap();
        assert_eq!(summary.observed_jobs, 2);
        assert_eq!(summary.body_sum_ns, 200);
        assert_eq!(summary.slowest_name, Some(JOB_NAMES[1]));
        assert_eq!(before.delta(before).unwrap(), JobBodySummary::default());
        after.0[2].count = 2;
        assert!(before.delta(after).is_none());
        assert!(after.delta(before).is_none());
        let detail = DispatcherDetail { input_ns: 2, build_ns: 3, execute_ns: 4,
            jobs: Some(summary) };
        assert_eq!(detail.residual_ns(10), Some(1)); // sum of parallel bodies excluded.
        assert_eq!(detail.residual_ns(8), None);
    }

    #[test]
    fn dispatcher_detail_snapshot_rejects_unknown_systems_without_clearing_counters() {
        let mut world = World::new();
        world.insert(TickProfile::default());
        let before = JobSnapshot::capture(&world).unwrap();
        world.read_resource::<TickProfile>().record_system(JOB_NAMES[1], 42);
        let after = JobSnapshot::capture(&world).unwrap();
        assert_eq!(before.delta(after).unwrap().slowest_body_ns, 42);
        assert_eq!(JOB_NAMES.iter().collect::<std::collections::BTreeSet<_>>().len(), JOB_NAMES.len());
        world.read_resource::<TickProfile>().record_system("untracked", 99);
        assert!(JobSnapshot::capture(&world).is_none());
        assert_eq!(world.read_resource::<TickProfile>().system_stats.lock().unwrap()[JOB_NAMES[1]].ns, 42);
    }
}
