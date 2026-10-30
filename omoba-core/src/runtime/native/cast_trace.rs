//! Test-only cast trace (conformance): every ability cast the script dispatch judged, with its
//! result. A harness inserts [`CastTrace`] into the world; without it nothing is recorded.
//! PlayerInput carries no correlation, so the owner command journal cannot tell a test why a cast
//! was not accepted; this trace does (rejection code, and the script's own `Err` text).
use specs::{Entity, World, WorldExt};

/// One dispatch's rows: its accepted casts, then the windups it started and the casts it refused;
/// a windup that ends without its release adds an [`CastTraceResult::Interrupted`] row then.
#[derive(Clone, Debug, Default)]
pub struct CastTrace(pub Vec<CastTraceRow>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CastTraceRow {
    pub tick: u64,
    pub caster: Entity,
    pub ability: String,
    pub result: CastTraceResult,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CastTraceResult {
    /// The cast ran and succeeded (serial settlement may still refuse its transaction).
    Accepted,
    /// A cast point started; the release is a later row.
    WindupStarted,
    /// Refused with an owner command code (`command_outcome`, e.g. NOT_ENOUGH_MANA).
    Rejected(&'static str),
    /// The script returned `Err`: its classified code and text.
    ScriptError { code: &'static str, error: String },
    /// A started windup ended without its release (cast_windup): `command` (a new order, channel,
    /// taunt or stop replaced it), `damage`, `dead`, `silenced`, `moved`, `recall`, `rank`,
    /// `target_lost`, `phase` (the match left Playing) or `invalid` (a replica's view disagreed).
    Interrupted(&'static str),
}

pub fn enabled(world: &World) -> bool {
    world.has_value::<CastTrace>()
}

pub(crate) fn record(world: &World, rows: impl IntoIterator<Item = CastTraceRow>) {
    if let Some(mut trace) = world.try_fetch_mut::<CastTrace>() { trace.0.extend(rows); }
}

/// The windups that ended unreleased, with their reasons.
pub(crate) fn interrupted(world: &World, ended: impl IntoIterator<Item = (Entity, String, &'static str)>) {
    if !enabled(world) { return; }
    let tick = world.read_resource::<crate::runtime::Tick>().0;
    record(world, ended.into_iter().map(|(caster, ability, reason)|
        CastTraceRow { tick, caster, ability, result: CastTraceResult::Interrupted(reason) }));
}
