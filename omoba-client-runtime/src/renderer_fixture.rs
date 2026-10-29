//! Synthetic filtered renderer publisher.
//!
//! Binds the production [`crate::presentation_bridge::PresentationHub`] on
//! loopback port 0. It never calls `SelectiveSession::connect`, `ReplicaHost`,
//! or the gameplay launcher. Observed `RendererInput` values are local transport
//! observations. This module never emits `CriticalInputResult`.

use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::SocketAddr,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
use std::collections::VecDeque;

use omoba_core::{
    game_proto::{
        renderer_input, renderer_ipc_envelope, RendererInput, RendererIpcEnvelope, TeamGameStart,
        CommunicationHelloResult, CommunicationPresentation,
        TeamPresentationSnapshot, TeamPublicEvent,
    },
    runtime::{
        decode_demo_render_state, decode_disclosed_combat_relation, decode_disclosed_health,
        encode_demo_render_state, encode_disclosed_combat_relation, encode_disclosed_property,
        DemoRenderState, DisclosedCombatRelation, FilteredRenderEntity, FilteredRenderSnapshot,
        CProperty, FactKind, DEMO_RENDER_COMPONENT_SCHEMA_ID,
        DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, DISCLOSED_HERO_COMPONENT_SCHEMA_ID,
        DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, SINGLE_LANE_ELAPSED_METRIC_ID,
        SINGLE_LANE_LENGTH_METRIC_ID, SINGLE_LANE_PHASE_METRIC_ID,
        SINGLE_LANE_RECALL_REMAINING_METRIC_ID, single_lane_player_metric,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    config::ClientRuntimeConfig,
    presentation_bridge::{self, PresentationHub, PRESENTATION_MAGIC, PRESENTATION_PROTOCOL_VERSION},
    ClientRuntimeError,
};

pub const CONTROL_SCHEMA: &str = "omoba-renderer-fixture-control/1";
pub const CONTROL_ACK_SCHEMA: &str = "omoba-renderer-fixture-control-ack/1";
pub const STARTUP_SCHEMA: &str = "omoba-renderer-fixture-startup/1";
pub const CLEANUP_SCHEMA: &str = "omoba-renderer-fixture-cleanup/1";
pub const INTENT_SCHEMA: &str = "omoba-renderer-fixture-intent/1";
pub const ERROR_SCHEMA: &str = "omoba-renderer-fixture-error/1";
pub const FIXTURE_MAP_ID: &str = "three_lane_training";
pub const FIXTURE_HERO_ID: &str = "training_luminary";
pub const OWNER_PLAYER_ID: u32 = 1;
pub const OWNER_TEAM_ID: u32 = 1;
pub const ENEMY_PLAYER_ID: u32 = 2;
pub const ENEMY_TEAM_ID: u32 = 2;
/// Nonzero team that is not the owned source team. A Neutral on this team is
/// nonhostile and is still not an ally.
pub const NEUTRAL_OTHER_TEAM_ID: u32 = 3;
/// Presentation render ids. Native entity id 0 is a different namespace and is
/// never published here. `disclosed_attack_target_admitted` rejects render id 0.
pub const SOURCE_RENDER_ID: u64 = 11;
pub const REPLACEMENT_SOURCE_RENDER_ID: u64 = 12;
pub const TARGET_RENDER_ID: u64 = 21;
pub const REPLACEMENT_TARGET_RENDER_ID: u64 = 22;
pub const DEFAULT_LIFETIME_SECONDS: u64 = 900;
pub const MIN_LIFETIME_SECONDS: u64 = 1;
pub const MAX_LIFETIME_SECONDS: u64 = 1800;
pub const MAX_CONTROL_BYTES: usize = 4096;
/// `ErrorKind::Interrupted` retries inside one open or read. There is no sleep.
/// An interrupt that is still present after this many tries counts as one poll.
const MAX_CONTROL_READ_INTERRUPTS_PER_POLL: u32 = 8;
/// Consecutive Windows sharing denials (raw 5, 32, 33) the existing 50ms control
/// poll may observe. The 300th consecutive denial fails. At an uninterrupted
/// 50ms cadence this is nominally 15s; intervening input and scheduling may delay polls.
/// UE's independent frame-based ACK timeout may expire earlier. The existing
/// process/lifetime watchdogs remain authoritative. NotFound clears this count.
pub const MAX_CONTROL_READ_TRANSIENTS: u32 = 300;
const TICK_RATE_HZ: u32 = 60;
const RELEASE_PROBE: Duration = Duration::from_secs(2);

const STAGES: &[&str] = &[
    "input_point", "input_keyboard", "input_locked",
    "combat_defaults", "combat_cursor_projection", "combat_idle_off_projection",
    // Explicit synthetic negotiation only. Never an authority receipt or gameplay ACK.
    "communication_ready",
    "communication_hud",
    "communication_received",
    "communication_signals",
    "communication_signals_expired",
    "communication_retire",
    "baseline",
    "remove_target",
    "replace_target",
    "source_removal",
    "source_replacement",
    "source_death",
    "death",
    "disclosure_epoch",
    "hud_cooldown",
    // Later owner disclosures. They do not bump a disclosure epoch.
    "owner_cast_ready",
    "owner_upgrade_point",
    "owner_short_mana",
    "owner_cooldown",
    "owner_hide_target",
    "owner_far_unit",
    "owner_apprentice",
    "owner_point_ready",
    "owner_support_ready",
    "owner_support_team_cards",
    "owner_support_team_cards_dead",
    "owner_support_enemy",
    "owner_support_conceal",
    "owner_support_dead",
    "owner_support_neutral",
    // Catalog-valid summoner projections. They do not bump a disclosure epoch
    // and they do not rewrite QWER ranks. The protocol flag is a separate contract.
    "owner_summoner_ready",
    "owner_summoner_cooldown",
    "owner_summoner_unavailable",
    "owner_summoner_malformed",
    "owner_summoner_unsupported",
    // Authored recall and six-slot inventory. They do not bump a disclosure epoch.
    // Shop protocol stays closed. Catalog identity is fixture observation, not a wire field.
    "owner_recall_ready",
    "owner_recall_unavailable",
    "owner_recall_channel",
    "owner_item_ready",
    "owner_item_cooldown",
    "owner_item_empty",
    "owner_item_replaced",
    "owner_item_far",
    // Typed presentation. These do not infer a mode from lives, hero presence, or map text.
    "mode_tower_defense",
    "mode_td_defeat",
    "mode_td_no_hero",
    "mode_moba_dead",
    "mode_unavailable",
    "mode_td_epoch",
    // Moba shop after an explicit baseline restore. Same six slots and the same gold 0.
    // The negotiated protocol flag is separate from those item stages. No epoch bump and no receipt.
    "owner_shop_ready",
    // Labeled controlled owner projection. These stages are not an authority ACK,
    // a vision settlement, or a charge debit. Malformed bytes clear the private channel.
    "controlled_owner_ward_projection",
    "controlled_owner_ward_dead",
    "controlled_owner_ward_malformed",
    "controlled_owner_ward_settled",
    "controlled_owner_ward_rejected",
    "controlled_owner_ward_uncertain",
    "controlled_owner_ward_clear",
    "controlled_owner_ward_forwarded",
    "controlled_owner_ward_uncertain_mapped",
    "controlled_owner_ward_settled_mapped",
    "controlled_owner_ward_rejected_mapped",
    "controlled_owner_ward_local_rejected",
    "controlled_owner_ward_same_epoch_source",
    // Shop on the current source. These stages do not restore source 11 or credit gold.
    "controlled_owner_shop_current",
    "controlled_owner_shop_admitted",
    "controlled_owner_shop_uncertain",
    "controlled_owner_shop_settled",
    "controlled_owner_shop_rejected",
    "controlled_owner_shop_stale",
    "controlled_owner_shop_epoch",
    "controlled_owner_shop_full",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureOptions {
    pub run_id: String,
    pub evidence_dir: PathBuf,
    pub lifetime_seconds: u64,
    pub shutdown_file: PathBuf,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CompiledContentIdentity {
    pub content_hash: String,
    pub content_catalog_identity_hash: String,
    pub moba_map_id: String,
    pub moba_map_catalog_hash: String,
    pub hero_content_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct StartupRecord {
    pub schema: String,
    pub run_id: String,
    pub address: String,
    pub player_id: u32,
    pub team_id: u32,
    pub lifetime_seconds: u64,
    pub started_unix_ms: u64,
    pub deadline_unix_ms: u64,
    pub content_hash: String,
    pub content_catalog_identity_hash: String,
    pub moba_map_id: String,
    pub moba_map_catalog_hash: String,
    pub hero_content_id: String,
    pub source_render_id: u64,
    pub source_disclosure_epoch: u64,
    pub target_render_id: u64,
    pub target_disclosure_epoch: u64,
    pub owner_player_id: u32,
    pub ready_sequence: u64,
    pub published_sequence: u64,
    pub authority: bool,
    pub critical_input_result: &'static str,
    pub presentation_render_id_zero: &'static str,
    pub native_entity_id_zero: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct ControlAck {
    pub schema: String,
    pub run_id: String,
    pub sequence: u64,
    pub stage: String,
    pub accepted: bool,
    pub reason: String,
    pub published_sequence: u64,
    pub content_hash: String,
    pub moba_map_id: String,
    pub moba_map_catalog_hash: String,
    pub source_render_id: Option<u64>,
    pub source_disclosure_epoch: Option<u64>,
    pub target_render_id: Option<u64>,
    pub target_disclosure_epoch: Option<u64>,
    pub authority_ack: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct CleanupRecord {
    pub schema: String,
    pub run_id: String,
    pub reason: String,
    pub address: String,
    pub listener_released: bool,
    pub authority: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlCommand {
    schema: String,
    run_id: String,
    sequence: u64,
    stage: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TargetSpan {
    End,
    Near,
    Far,
}

#[derive(Clone)]
struct Actor {
    render_id: u64,
    epoch: u64,
    owner_player_id: u32,
    team_id: u32,
    alive: bool,
    attackable: bool,
    /// Disclosed combat faction. 1 is Player, the previous hardcoded default.
    faction_kind: u8,
}

/// Synthetic fixture capability. This is not a Connected boolean and not an authority ACK.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum SummonerContract {
    #[default]
    Closed,
    Ready,
    Cooldown,
    Unavailable,
    Malformed,
    Unsupported,
}

impl SummonerContract {
    fn requests_protocol(self) -> bool {
        !matches!(self, Self::Closed | Self::Unsupported)
    }
}

/// Recall admission is the negotiated protocol plus a living owner. Remaining time is disclosed, not a local refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum RecallContract {
    #[default]
    Closed,
    Ready,
    Unavailable,
    Channel,
}

impl RecallContract {
    fn requests_protocol(self) -> bool {
        matches!(self, Self::Ready | Self::Channel)
    }
}

/// Six catalog slots. Closed publishes no OwnerEconomy event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum ItemContract {
    #[default]
    Closed,
    Ready,
    Cooldown,
    Empty,
    Replaced,
    Far,
}

const READY_ITEM_CATALOG: [u16; 6] = [5, 10, 11, 1, 6, 7];

/// Shop admission for one restored owner. Closed keeps the existing item stages read-only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum ShopContract {
    #[default]
    Closed,
    Ready,
}

/// Labeled private owner projection for this fixture. Not an authority receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum WardContract {
    #[default]
    Closed,
    Ready,
    Dead,
    Malformed,
    Settled,
    Rejected,
    Uncertain,
    Forwarded,
    UncertainMapped,
    SettledMapped,
    RejectedMapped,
    LocalRejected,
}

/// One-shot buy/sell correlation for the next snapshot. Closed publishes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum ShopTxnContract {
    #[default]
    Closed,
    Current,
    Admitted,
    Uncertain,
    Settled,
    Rejected,
    Stale,
    EpochClear,
    /// Sixty-four admitted buys and no receipt. This stage does not arm a renderer request.
    Full,
}

#[derive(Clone)]
struct FixtureWorld {
    combat_preferences: Option<(u32, bool)>,
    input_scheme: Option<omoba_core::game_proto::InputSchemeOwnerState>,
    communication_enabled: bool,
    team_cards_disclosed: bool,
    communication_received: bool,
    communication_signals: u8,
    birth: omoba_core::runtime::Hero,
    base_hp_raw: i64,
    active: omoba_core::runtime::Hero,
    active_hp_raw: i64,
    lane: Vec<(i32, i32)>,
    source_xy: (i64, i64),
    lane_end_xy: (i64, i64),
    target_xy: (i64, i64),
    target_span: TargetSpan,
    hide_target: bool,
    conceal_target: bool,
    lane_length_raw: i64,
    epoch_floor: u64,
    source: Option<Actor>,
    target: Option<Actor>,
    hud_cooldown: bool,
    summoner_contract: SummonerContract,
    recall_contract: RecallContract,
    item_contract: ItemContract,
    shop_contract: ShopContract,
    ward_contract: WardContract,
    last_ward_renderer_request: u64,
    shop_txn: ShopTxnContract,
    last_shop_renderer_request: u64,
    last_shop_action_kind: u32,
    presentation_mode: Option<omoba_core::runtime::native::comp::GameMode>,
    td_defeat: bool,
    suppress_owner_hero: bool,
    replica_tick: u64,
    published_sequence: u64,
    control_sequence: u64,
}

pub struct RendererFixture {
    hub: PresentationHub,
    pending_communication: Option<CommunicationPresentation>,
    world: FixtureWorld,
    identity: CompiledContentIdentity,
    run_id: String,
    evidence_dir: PathBuf,
    shutdown_file: PathBuf,
    lifetime_seconds: u64,
    deadline: tokio::time::Instant,
    observations: u64,
    last_control_token: String,
    /// Consecutive sharing denials since the last absent or completed read.
    control_read_streak: u32,
    /// Scripted bodies for tests. Each one is handled by `accept_control_body`.
    #[cfg(test)]
    pending_control_reads: VecDeque<ControlFileBody>,
    last_snapshot: RendererIpcEnvelope,
    ready: RendererIpcEnvelope,
}

pub fn run() -> i32 {
    let parsed = match parse_args(std::env::args().skip(1)) {
        Ok(Parsed::Help) => {
            println!("{}", usage());
            return 0;
        }
        Ok(Parsed::Run(options)) => options,
        Err(error) => {
            eprintln!("renderer fixture: {error}");
            return 2;
        }
    };
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("renderer fixture: tokio runtime: {error}");
            return 1;
        }
    };
    runtime.block_on(run_options(parsed))
}

pub async fn run_options(options: FixtureOptions) -> i32 {
    let fixture = match RendererFixture::start(options).await {
        Ok(fixture) => fixture,
        Err(error) => {
            eprintln!("renderer fixture: {error}");
            return if error.to_string().contains("startup identity was not published") {
                3
            } else {
                2
            };
        }
    };
    let code = fixture.run_until_deadline().await;
    code
}

pub fn parse_lifetime(value: Option<&str>) -> Result<u64, String> {
    let Some(value) = value else {
        return Ok(DEFAULT_LIFETIME_SECONDS);
    };
    let parsed: u64 = value
        .parse()
        .map_err(|_| format!("lifetime seconds must be an integer {MIN_LIFETIME_SECONDS}..={MAX_LIFETIME_SECONDS}"))?;
    if (MIN_LIFETIME_SECONDS..=MAX_LIFETIME_SECONDS).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(format!(
            "lifetime seconds must be {MIN_LIFETIME_SECONDS}..={MAX_LIFETIME_SECONDS}"
        ))
    }
}

pub fn valid_run_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=64).contains(&bytes.len())
        && bytes[0].is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[derive(Debug)]
enum Parsed {
    Help,
    Run(FixtureOptions),
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Parsed, String> {
    let mut run_id = None;
    let mut evidence_dir = None;
    let mut lifetime = None;
    let mut shutdown_file = None;
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        if flag == "--help" || flag == "-h" {
            return Ok(Parsed::Help);
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--run-id" => run_id = Some(value),
            "--evidence-dir" => evidence_dir = Some(PathBuf::from(value)),
            "--lifetime-seconds" => lifetime = Some(value),
            "--shutdown-file" => shutdown_file = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown argument {flag}")),
        }
    }
    let run_id = run_id.ok_or("missing --run-id")?;
    if !valid_run_id(&run_id) {
        return Err("run id must match [A-Za-z0-9][A-Za-z0-9._-]{0,63}".to_owned());
    }
    let evidence_dir = evidence_dir.ok_or("missing --evidence-dir")?;
    if evidence_dir.file_name().and_then(|name| name.to_str()) != Some(run_id.as_str()) {
        return Err("evidence directory name must equal --run-id".to_owned());
    }
    let lifetime_seconds = parse_lifetime(lifetime.as_deref())?;
    let shutdown_file = shutdown_file.unwrap_or_else(|| evidence_dir.join("shutdown.signal"));
    Ok(Parsed::Run(FixtureOptions {
        run_id,
        evidence_dir,
        lifetime_seconds,
        shutdown_file,
    }))
}

fn usage() -> &'static str {
    "omoba-renderer-fixture --run-id ID --evidence-dir DIR/ID [--lifetime-seconds 1..1800] [--shutdown-file PATH]\n\
     Default lifetime is 900 seconds. The process does not extend it.\n\
     Stages: baseline, remove_target, replace_target, source_removal, source_replacement, source_death, death, disclosure_epoch, hud_cooldown, owner_cast_ready, owner_upgrade_point, owner_short_mana, owner_cooldown, owner_hide_target, owner_far_unit, owner_apprentice, owner_point_ready, owner_support_ready, owner_support_enemy, owner_support_conceal, owner_support_dead, owner_support_neutral, owner_summoner_ready, owner_summoner_cooldown, owner_summoner_unavailable, owner_summoner_malformed, owner_summoner_unsupported, owner_recall_ready, owner_recall_unavailable, owner_recall_channel, owner_item_ready, owner_item_cooldown, owner_item_empty, owner_item_replaced, owner_item_far, mode_tower_defense, mode_td_defeat, mode_td_no_hero, mode_moba_dead, mode_unavailable, mode_td_epoch, owner_shop_ready, controlled_owner_ward_projection, controlled_owner_ward_dead, controlled_owner_ward_malformed, controlled_owner_ward_settled, controlled_owner_ward_rejected, controlled_owner_ward_uncertain, controlled_owner_ward_clear, controlled_owner_ward_forwarded, controlled_owner_ward_uncertain_mapped, controlled_owner_ward_settled_mapped, controlled_owner_ward_rejected_mapped, controlled_owner_ward_local_rejected, controlled_owner_ward_same_epoch_source, controlled_owner_shop_current, controlled_owner_shop_admitted, controlled_owner_shop_uncertain, controlled_owner_shop_settled, controlled_owner_shop_rejected, controlled_owner_shop_stale, controlled_owner_shop_epoch, controlled_owner_shop_full"
}

pub fn compiled_content_identity() -> Result<CompiledContentIdentity, ClientRuntimeError> {
    let map = omoba_template_ids::moba_map_by_name(FIXTURE_MAP_ID).ok_or_else(|| {
        ClientRuntimeError::Config(format!("compiled map {FIXTURE_MAP_ID} is missing"))
    })?;
    if map.id != FIXTURE_MAP_ID {
        return Err(ClientRuntimeError::Config(
            "compiled map id does not match the requested catalog entry".into(),
        ));
    }
    let hero_id = omoba_template_ids::hero_by_name(FIXTURE_HERO_ID).ok_or_else(|| {
        ClientRuntimeError::Config(format!("compiled hero {FIXTURE_HERO_ID} is missing"))
    })?;
    if omoba_template_ids::active_hero_stats(hero_id).is_none() {
        return Err(ClientRuntimeError::Config(format!(
            "compiled hero {FIXTURE_HERO_ID} has no stats"
        )));
    }
    Ok(CompiledContentIdentity {
        content_hash: omoba_template_ids::CONTENT_CATALOG_DATA_HASH.to_owned(),
        content_catalog_identity_hash: omoba_template_ids::CONTENT_CATALOG_IDENTITY_HASH.to_owned(),
        moba_map_id: map.id.to_owned(),
        moba_map_catalog_hash: omoba_template_ids::MOBA_MAP_CATALOG_HASH.to_owned(),
        hero_content_id: FIXTURE_HERO_ID.to_owned(),
    })
}

impl RendererFixture {
    pub async fn start(options: FixtureOptions) -> Result<Self, ClientRuntimeError> {
        if !valid_run_id(&options.run_id) {
            return Err(ClientRuntimeError::Config("invalid run id".into()));
        }
        if !(MIN_LIFETIME_SECONDS..=MAX_LIFETIME_SECONDS).contains(&options.lifetime_seconds) {
            return Err(ClientRuntimeError::Config("invalid lifetime".into()));
        }
        if options.evidence_dir.file_name().and_then(|name| name.to_str()) != Some(options.run_id.as_str())
        {
            return Err(ClientRuntimeError::Config(
                "evidence directory name must equal run id".into(),
            ));
        }
        fs::create_dir(&options.evidence_dir).map_err(|error| {
            ClientRuntimeError::Config(format!(
                "evidence directory must be a fresh path: {error}"
            ))
        })?;
        let identity = match compiled_content_identity() {
            Ok(identity) => identity,
            Err(error) => {
                write_error(&options.evidence_dir, &options.run_id, &error.to_string());
                return Err(error);
            }
        };
        let config = fixture_config(&options.evidence_dir, &identity);
        let hub = match PresentationHub::bind(&config).await {
            Ok(hub) => hub,
            Err(error) => {
                write_error(
                    &options.evidence_dir,
                    &options.run_id,
                    &format!("bind failed: {error}"),
                );
                return Err(ClientRuntimeError::Ipc(format!(
                    "startup identity was not published: {error}"
                )));
            }
        };
        let started = RendererFixture::finish_start(hub, config, options, identity).await;
        if let Err(error) = &started {
            // finish_start shuts the hub down before returning.
            let _ = error;
        }
        started
    }

    async fn finish_start(
        hub: PresentationHub,
        config: ClientRuntimeConfig,
        options: FixtureOptions,
        identity: CompiledContentIdentity,
    ) -> Result<Self, ClientRuntimeError> {
        let world = match FixtureWorld::load() {
            Ok(world) => world,
            Err(error) => {
                return Err(startup_rejected(
                    &hub,
                    &options.evidence_dir,
                    &options.run_id,
                    error,
                ))
            }
        };
        let ready = match ready_frame(&config, &identity) {
            Ok(ready) => ready,
            Err(error) => {
                return Err(startup_rejected(
                    &hub,
                    &options.evidence_dir,
                    &options.run_id,
                    error.to_string(),
                ))
            }
        };
        if !ready_matches(&ready, &identity) {
            return Err(startup_rejected(
                &hub,
                &options.evidence_dir,
                &options.run_id,
                "ready content or map hash disagrees with the compiled catalog".into(),
            ));
        }
        hub.publish_latest(ready.clone());
        let mut fixture = Self {
            hub,
            pending_communication: None,
            world,
            identity,
            run_id: options.run_id,
            evidence_dir: options.evidence_dir,
            shutdown_file: options.shutdown_file,
            lifetime_seconds: options.lifetime_seconds,
            deadline: tokio::time::Instant::now() + Duration::from_secs(options.lifetime_seconds),
            observations: 0,
            last_control_token: String::new(),
            control_read_streak: 0,
            #[cfg(test)]
            pending_control_reads: VecDeque::new(),
            last_snapshot: ready.clone(),
            ready,
        };
        if let Err(error) = fixture.publish_snapshot() {
            let message = error.to_string();
            fixture.hub.shutdown();
            write_error(&fixture.evidence_dir, &fixture.run_id, &message);
            return Err(ClientRuntimeError::Ipc(format!(
                "startup identity was not published: {message}"
            )));
        }
        let startup = fixture.startup_record();
        if startup.source_render_id == 0 || startup.target_render_id == 0 {
            let message = "startup requires nonzero source and target presentation render ids";
            fixture.hub.shutdown();
            write_error(&fixture.evidence_dir, &fixture.run_id, message);
            return Err(ClientRuntimeError::Ipc(format!(
                "startup identity was not published: {message}"
            )));
        }
        if let Err(error) = write_json(&fixture.evidence_dir.join("startup.json"), &startup) {
            let message = format!("startup file: {error}");
            fixture.hub.shutdown();
            write_error(&fixture.evidence_dir, &fixture.run_id, &message);
            return Err(ClientRuntimeError::Ipc(format!(
                "startup identity was not published: {message}"
            )));
        }
        println!("{}", serde_json::to_string(&startup).unwrap_or_default());
        Ok(fixture)
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.hub.local_addr()
    }

    pub fn startup_record(&self) -> StartupRecord {
        let now_ms = unix_ms();
        let (source_id, source_epoch) = actor_identity(&self.world.source);
        let (target_id, target_epoch) = actor_identity(&self.world.target);
        StartupRecord {
            schema: STARTUP_SCHEMA.into(),
            run_id: self.run_id.clone(),
            address: self.local_addr().to_string(),
            player_id: OWNER_PLAYER_ID,
            team_id: OWNER_TEAM_ID,
            lifetime_seconds: self.lifetime_seconds,
            started_unix_ms: now_ms,
            deadline_unix_ms: now_ms.saturating_add(self.lifetime_seconds.saturating_mul(1000)),
            content_hash: self.identity.content_hash.clone(),
            content_catalog_identity_hash: self.identity.content_catalog_identity_hash.clone(),
            moba_map_id: self.identity.moba_map_id.clone(),
            moba_map_catalog_hash: self.identity.moba_map_catalog_hash.clone(),
            hero_content_id: self.identity.hero_content_id.clone(),
            source_render_id: source_id.unwrap_or(0),
            source_disclosure_epoch: source_epoch.unwrap_or(0),
            target_render_id: target_id.unwrap_or(0),
            target_disclosure_epoch: target_epoch.unwrap_or(0),
            owner_player_id: OWNER_PLAYER_ID,
            ready_sequence: self.ready.sequence,
            published_sequence: self.world.published_sequence,
            authority: false,
            critical_input_result: "never",
            presentation_render_id_zero: "invalid",
            native_entity_id_zero: "different namespace; not a presentation render id",
        }
    }

    pub fn ready_envelope(&self) -> &RendererIpcEnvelope {
        &self.ready
    }

    pub fn last_snapshot(&self) -> &RendererIpcEnvelope {
        &self.last_snapshot
    }

    pub fn apply_control_bytes(&mut self, bytes: &[u8]) -> ControlAck {
        if bytes.len() > MAX_CONTROL_BYTES {
            return self.ack(0, "", false, "oversized");
        }
        let command: ControlCommand = match serde_json::from_slice(bytes) {
            Ok(command) => command,
            Err(_) => return self.ack(0, "", false, "malformed"),
        };
        if command.schema != CONTROL_SCHEMA {
            return self.ack(command.sequence, &command.stage, false, "unknown_schema");
        }
        if command.run_id != self.run_id {
            return self.ack(command.sequence, &command.stage, false, "run_mismatch");
        }
        if command.sequence == 0 || command.sequence <= self.world.control_sequence {
            return self.ack(command.sequence, &command.stage, false, "stale_sequence");
        }
        if command.sequence != self.world.control_sequence.saturating_add(1) {
            return self.ack(command.sequence, &command.stage, false, "sequence_not_next");
        }
        if !STAGES.contains(&command.stage.as_str()) {
            return self.ack(command.sequence, &command.stage, false, "unknown_stage");
        }
        let mut next = self.world.clone();
        if let Err(reason) = next.apply_stage(&command.stage) {
            return self.ack(command.sequence, &command.stage, false, reason);
        }
        if next.to_snapshot().is_err() {
            return self.ack(command.sequence, &command.stage, false, "malformed");
        }
        let previous = self.world.clone();
        self.world = next;
        self.world.control_sequence = command.sequence;
        if let Err(error) = self.publish_snapshot() {
            self.world = previous;
            return self.ack(command.sequence, &command.stage, false, &error.to_string());
        }
        self.ack(command.sequence, &command.stage, true, "")
    }

    pub fn record_input(&mut self, input: RendererInput) -> Result<serde_json::Value, ClientRuntimeError> {
        self.observations = self.observations.saturating_add(1);
        if matches!(input.intent, Some(omoba_core::game_proto::renderer_input::Intent::WardPlacement(_))) && input.request_id != 0 {
            self.world.last_ward_renderer_request = input.request_id;
        }
        if let Some(omoba_core::game_proto::renderer_input::Intent::ItemBuy(_)) = &input.intent {
            if input.request_id != 0 {
                self.world.last_shop_renderer_request = input.request_id;
                self.world.last_shop_action_kind = 17;
            }
        }
        if let Some(omoba_core::game_proto::renderer_input::Intent::ItemSell(_)) = &input.intent {
            if input.request_id != 0 {
                self.world.last_shop_renderer_request = input.request_id;
                self.world.last_shop_action_kind = 18;
            }
        }
        let (source_id, source_epoch) = actor_identity(&self.world.source);
        let (target_id, target_epoch) = actor_identity(&self.world.target);
        let record = json!({
            "schema": INTENT_SCHEMA,
            "run_id": self.run_id,
            "observation_ordinal": self.observations,
            // This legacy field identifies the disclosed snapshot behind the observed input.
            // Communication has its own intervening IPC envelopes, recorded separately.
            "published_sequence": self.last_snapshot.sequence,
            "ipc_sequence": self.world.published_sequence,
            "local_request_id": input.request_id,
            "player_id": input.player_id,
            "disclosure_epoch": input.disclosure_epoch,
            "intent": intent_value(&self.world, input.intent),
            "content_hash": self.identity.content_hash,
            "content_catalog_identity_hash": self.identity.content_catalog_identity_hash,
            "moba_map_id": self.identity.moba_map_id,
            "moba_map_catalog_hash": self.identity.moba_map_catalog_hash,
            "source_render_id": source_id,
            "source_disclosure_epoch": source_epoch,
            "target_render_id": target_id,
            "target_disclosure_epoch": target_epoch,
            "entity_epoch_floor": self.world.epoch_floor,
            "authority_ack": false,
        });
        let line = serde_json::to_string(&record)
            .map_err(|error| ClientRuntimeError::Ipc(error.to_string()))?;
        append_line(&self.evidence_dir.join("intents.jsonl"), &line)?;
        println!("{line}");
        Ok(record)
    }

    pub async fn run_until_deadline(mut self) -> i32 {
        let mut reason = "lifetime";
        loop {
            if tokio::time::Instant::now() >= self.deadline {
                break;
            }
            tokio::select! {
                _ = tokio::time::sleep_until(self.deadline) => {
                    reason = "lifetime";
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(50)) => {
                    if self.shutdown_file.exists() {
                        reason = "shutdown_file";
                        break;
                    }
                    if let Err(error) = self.poll_control_file() {
                        write_error(&self.evidence_dir, &self.run_id, &error.to_string());
                        self.finish("control_poll_failed").await;
                        return 1;
                    }
                    if let Err(error) = self.publish_pending_communication().await {
                        write_error(&self.evidence_dir, &self.run_id, &error.to_string());
                        self.finish("communication_publish_failed").await;
                        return 1;
                    }
                }
                input = self.hub.recv_input() => {
                    let Some(input) = input else {
                        write_error(&self.evidence_dir, &self.run_id, "input channel closed");
                        self.finish("input_channel_closed").await;
                        return 1;
                    };
                    if let Err(error) = self.record_input(input) {
                        write_error(&self.evidence_dir, &self.run_id, &error.to_string());
                        self.finish("intent_log_failed").await;
                        return 1;
                    }
                }
            }
        }
        let (cleanup, persisted) = self.finish(reason).await;
        if !cleanup.listener_released {
            return 4;
        }
        if !persisted {
            return 1;
        }
        0
    }

    async fn finish(self, reason: &str) -> (CleanupRecord, bool) {
        let address = self.local_addr();
        let run_id = self.run_id.clone();
        let evidence_dir = self.evidence_dir.clone();
        self.hub.shutdown();
        let listener_released = probe_listener_released(address).await;
        let record = CleanupRecord {
            schema: CLEANUP_SCHEMA.into(),
            run_id,
            reason: reason.to_owned(),
            address: address.to_string(),
            listener_released,
            authority: false,
        };
        let persisted = match write_json(&evidence_dir.join("cleanup.json"), &record) {
            Ok(()) => true,
            Err(error) => {
                note_evidence_error(
                    &evidence_dir,
                    &record.run_id,
                    &format!("cleanup file: {error}"),
                    Some(reason),
                    Some(listener_released),
                );
                false
            }
        };
        (record, persisted)
    }

    fn poll_control_file(&mut self) -> Result<(), ClientRuntimeError> {
        let path = self.evidence_dir.join("control.json");
        #[cfg(test)]
        let body = self
            .pending_control_reads
            .pop_front()
            .unwrap_or_else(|| read_control_file(&path));
        #[cfg(not(test))]
        let body = read_control_file(&path);
        self.accept_control_body(body)
    }

    fn accept_control_body(&mut self, body: ControlFileBody) -> Result<(), ClientRuntimeError> {
        match body {
            ControlFileBody::Absent => {
                self.control_read_streak = 0;
                Ok(())
            }
            ControlFileBody::Failed { operation, error } => self.note_control_failure(operation, &error),
            ControlFileBody::Oversized { observed_len } => self.accept_oversized_control(observed_len),
            ControlFileBody::Bytes(bytes) if bytes.len() > MAX_CONTROL_BYTES => {
                self.accept_oversized_control(bytes.len() as u64)
            }
            ControlFileBody::Bytes(bytes) => self.accept_control_bytes(bytes),
        }
    }

    fn note_control_failure(&mut self, operation: &str, error: &std::io::Error) -> Result<(), ClientRuntimeError> {
        match classify_control_io(error) {
            ControlIoClass::Absent => {
                self.control_read_streak = 0;
                Ok(())
            }
            ControlIoClass::Transient | ControlIoClass::Interrupted => {
                self.control_read_streak = self.control_read_streak.saturating_add(1);
                if self.control_read_streak < MAX_CONTROL_READ_TRANSIENTS {
                    return Ok(());
                }
                Err(control_read_failure(
                    operation,
                    &self.control_path(),
                    error,
                    self.control_read_streak,
                ))
            }
            ControlIoClass::Permanent => Err(control_read_failure(
                operation,
                &self.control_path(),
                error,
                self.control_read_streak,
            )),
        }
    }

    fn accept_oversized_control(&mut self, observed_len: u64) -> Result<(), ClientRuntimeError> {
        self.control_read_streak = 0;
        let token = format!("oversized:{observed_len}");
        if token == self.last_control_token {
            return Ok(());
        }
        self.last_control_token = token;
        let ack = self.ack(0, "", false, "oversized");
        write_json(&self.evidence_dir.join("control.ack.json"), &ack)
    }

    fn accept_control_bytes(&mut self, bytes: Vec<u8>) -> Result<(), ClientRuntimeError> {
        self.control_read_streak = 0;
        let token = hex::encode(Sha256::digest(&bytes));
        if token == self.last_control_token {
            return Ok(());
        }
        let ack = self.apply_control_bytes(&bytes);
        self.last_control_token = token;
        write_json(&self.evidence_dir.join("control.ack.json"), &ack)
    }

    fn control_path(&self) -> PathBuf {
        self.evidence_dir.join("control.json")
    }

    fn publish_snapshot(&mut self) -> Result<(), ClientRuntimeError> {
        self.prepare_communication();
        let enabled = fixture_summoner_protocol_enabled(
            self.world.summoner_contract.requests_protocol(),
            &self.identity.content_hash,
        );
        self.hub.set_summoner_protocol_enabled(enabled);
        let recall_enabled = fixture_recall_protocol_enabled(
            self.world.recall_contract.requests_protocol(),
            &self.identity.content_hash,
        );
        self.hub.set_recall_protocol_enabled(recall_enabled);
        let shop_enabled = fixture_shop_protocol_enabled(
            self.world.shop_contract == ShopContract::Ready,
            &self.identity.content_hash,
        );
        self.hub.set_shop_protocol_enabled(shop_enabled);
        let (ward_protocol, ward_bytes) = self.world.owner_ward_bytes()
            .map_err(|reason| ClientRuntimeError::Ipc(reason.into()))?;
        self.hub.set_owner_ward_private(self.world.epoch_floor, ward_protocol, &ward_bytes);
        let correlations = self.world.ward_correlation_drafts()
            .map_err(|reason| ClientRuntimeError::Ipc(reason.into()))?;
        self.hub.replace_ward_correlations(self.world.epoch_floor, correlations);
        let snapshot = self.world.to_snapshot()?;
        self.publish_shop_transaction()?;
        self.world.replica_tick = self.world.replica_tick.saturating_add(1);
        self.world.published_sequence = self.world.published_sequence.saturating_add(1);
        let envelope = self.hub.snapshot_envelope(
            self.world.published_sequence,
            self.world.replica_tick,
            self.world.epoch_floor,
            snapshot,
            0,
        );
        if matches!(
            envelope.payload,
            Some(renderer_ipc_envelope::Payload::CriticalInputResult(_))
        ) {
            return Err(ClientRuntimeError::Ipc(
                "fixture refused to publish CriticalInputResult".into(),
            ));
        }
        self.hub.publish_latest(envelope.clone());
        self.last_snapshot = envelope;
        Ok(())
    }

    fn prepare_communication(&mut self) {
        use omoba_core::runtime::communication_transport as wire;
        let result = if self.world.communication_enabled {
            self.hub.note_communication_transition(self.world.epoch_floor, self.world.replica_tick + 1);
            Some(CommunicationHelloResult {
                schema_version: wire::COMMUNICATION_SCHEMA_VERSION,
                protocol_version: wire::COMMUNICATION_PROTOCOL_VERSION,
                agreed: true, refusal: wire::REFUSAL_ADMITTED, room_id: 1, run_id: 1,
                player_id: OWNER_PLAYER_ID, team_id: OWNER_TEAM_ID,
                view_epoch: self.world.epoch_floor, request_floor: 0,
            })
        } else if self.hub.current_communication_presentation().is_some() {
            Some(CommunicationHelloResult {
                schema_version: wire::COMMUNICATION_SCHEMA_VERSION,
                refusal: wire::REFUSAL_NOT_NEGOTIATED, ..Default::default()
            })
        } else { None };
        if let Some(result) = result {
            self.pending_communication = self.hub.apply_communication_hello(&result);
        }
        if self.world.communication_received {
            // Explicit inbound fixture rows, unrelated to renderer requests. Never fabricate
            // an accepted input receipt or a critical gameplay ACK from these observations.
            for (sender, team, audience, channel, text) in [
                (OWNER_PLAYER_ID, OWNER_TEAM_ID, wire::AUDIENCE_TEAM, wire::CHANNEL_TEAM, "隊伍公告：一起守住中路"),
                (2, 2, wire::AUDIENCE_ALL, wire::CHANNEL_ALL, "全體公告：準備出發"),
            ] {
                let event = omoba_core::game_proto::CommunicationWireEvent {
                    schema_version: wire::COMMUNICATION_SCHEMA_VERSION, room_id: 1, run_id: 1,
                    sender_id: sender, sender_team: team, sequence: 1,
                    view_epoch: self.world.epoch_floor, audience, kind: wire::KIND_CHAT,
                    chat_channel: channel, text: text.into(), ..Default::default()
                };
                if let Some(batch) = self.hub.apply_communication_event(&event) {
                    self.pending_communication = Some(batch);
                }
            }
        }
        if self.world.communication_signals != 0 {
            // Public inbound observations only. No renderer request or accepted receipt.
            let base = |sequence, kind| omoba_core::game_proto::CommunicationWireEvent {
                schema_version: wire::COMMUNICATION_SCHEMA_VERSION, room_id: 1, run_id: 1,
                sender_id: OWNER_PLAYER_ID, sender_team: OWNER_TEAM_ID, sequence,
                view_epoch: self.world.epoch_floor, audience: wire::AUDIENCE_TEAM, kind,
                admitted_at_raw: 1024, ..Default::default()
            };
            let ping = omoba_core::game_proto::CommunicationWireEvent {
                has_point: true, semantic: 5, point_x_raw: 0, point_y_raw: 6 * 1024,
                expires_at_raw: 1024 + omoba_core::runtime::communication::PING_MARKER_TTL_RAW,
                ..base(10, wire::KIND_PING)
            };
            let objective = omoba_core::game_proto::CommunicationWireEvent {
                objective_id: "middle_gate".into(), objective_x_raw: -8 * 1024, objective_y_raw: 8 * 1024,
                expires_at_raw: 1024 + omoba_core::runtime::communication::OBJECTIVE_VOTE_TTL_RAW,
                ..base(11, wire::KIND_OBJECTIVE)
            };
            let row = omoba_template_ids::communication_content(101).expect("canonical wheel fixture content");
            let emote = omoba_core::game_proto::CommunicationWireEvent {
                content_id: 101, expires_at_raw: 1024 + i64::from(row.duration_raw), ..base(12, wire::KIND_EMOTE)
            };
            for event in [ping, objective, emote] {
                if let Some(batch) = self.hub.apply_communication_event(&event) { self.pending_communication = Some(batch); }
            }
            if self.world.communication_signals == 2 {
                let later = omoba_core::game_proto::CommunicationWireEvent {
                    admitted_at_raw: 32768, chat_channel: wire::CHANNEL_TEAM,
                    text: "訊號期限已結束".into(), ..base(13, wire::KIND_CHAT)
                };
                if let Some(batch) = self.hub.apply_communication_event(&later) { self.pending_communication = Some(batch); }
            }
        }
    }

    async fn publish_pending_communication(&mut self) -> Result<(), ClientRuntimeError> {
        if let Some(batch) = self.pending_communication.take() {
            self.world.published_sequence = self.world.published_sequence.checked_add(1)
                .ok_or_else(|| ClientRuntimeError::Ipc("fixture IPC sequence exhausted".into()))?;
            self.hub.publish_critical(presentation_bridge::communication_envelope(self.world.published_sequence, batch)).await?;
        }
        Ok(())
    }

    /// Binds the last renderer request, then retains a canonical receipt when the stage says so.
    /// The receipt tick is the snapshot tick this publish is about to assign.
    fn publish_shop_transaction(&mut self) -> Result<(), ClientRuntimeError> {
        let contract = self.world.shop_txn;
        self.world.shop_txn = ShopTxnContract::Closed;
        if matches!(contract, ShopTxnContract::Closed | ShopTxnContract::Current) {
            return Ok(());
        }
        let epoch = self.world.epoch_floor;
        let request = self.world.last_shop_renderer_request;
        let tick = self.world.replica_tick.saturating_add(1);
        let fail = |reason: &str| Err(ClientRuntimeError::Ipc(reason.into()));
        if request == 0 && !matches!(contract, ShopTxnContract::EpochClear | ShopTxnContract::Full) {
            return fail("shop_request_unseen");
        }
        match contract {
            ShopTxnContract::Closed | ShopTxnContract::Current => Ok(()),
            ShopTxnContract::Admitted => {
                if !self.hub.note_shop_request(epoch, OWNER_PLAYER_ID, 801, request, 17).keeps_pair() {
                    return fail("shop_pending_capacity");
                }
                Ok(())
            }
            ShopTxnContract::Uncertain => {
                if !self.hub.note_shop_uncertain(epoch, OWNER_PLAYER_ID, request) {
                    return fail("shop_request_unseen");
                }
                Ok(())
            }
            ShopTxnContract::Settled => {
                let event = shop_receipt_event(OWNER_PLAYER_ID, 801, tick, 17, 4, 0, 0);
                self.hub.retain_shop_receipts(epoch, tick, &[event]);
                Ok(())
            }
            ShopTxnContract::Rejected => {
                if !self.hub.note_shop_request(epoch, OWNER_PLAYER_ID, 802, request, 18).keeps_pair() {
                    return fail("shop_pending_capacity");
                }
                let event = shop_receipt_event(OWNER_PLAYER_ID, 802, tick, 18, 0, 0, 6);
                self.hub.retain_shop_receipts(epoch, tick, &[event]);
                Ok(())
            }
            ShopTxnContract::Stale => {
                if !self.hub.note_shop_request(epoch, OWNER_PLAYER_ID, 803, request, 17).keeps_pair() {
                    return fail("shop_pending_capacity");
                }
                let foreign = shop_receipt_event(8, 803, tick, 17, 4, 0, 0);
                let unbound = shop_receipt_event(OWNER_PLAYER_ID, 999, tick, 17, 4, 0, 0);
                self.hub.retain_shop_receipts(epoch, tick, &[foreign, unbound]);
                Ok(())
            }
            ShopTxnContract::EpochClear => {
                self.hub.retain_shop_receipts(epoch, tick, &[]);
                Ok(())
            }
            ShopTxnContract::Full => {
                for authority in 1u32..=64 {
                    let request_id = 100 + u64::from(authority);
                    if !self.hub.note_shop_request(epoch, OWNER_PLAYER_ID, authority, request_id, 17).keeps_pair() {
                        return fail("shop_pending_capacity");
                    }
                }
                Ok(())
            }
        }
    }

    fn ack(&self, sequence: u64, stage: &str, accepted: bool, reason: &str) -> ControlAck {
        let (source_id, source_epoch) = actor_identity(&self.world.source);
        let (target_id, target_epoch) = actor_identity(&self.world.target);
        ControlAck {
            schema: CONTROL_ACK_SCHEMA.into(),
            run_id: self.run_id.clone(),
            sequence,
            stage: stage.to_owned(),
            accepted,
            reason: reason.to_owned(),
            published_sequence: self.world.published_sequence,
            content_hash: self.identity.content_hash.clone(),
            moba_map_id: self.identity.moba_map_id.clone(),
            moba_map_catalog_hash: self.identity.moba_map_catalog_hash.clone(),
            source_render_id: source_id,
            source_disclosure_epoch: source_epoch,
            target_render_id: target_id,
            target_disclosure_epoch: target_epoch,
            authority_ack: false,
        }
    }
}

impl Drop for RendererFixture {
    fn drop(&mut self) {
        self.hub.shutdown();
    }
}

impl FixtureWorld {
    fn load() -> Result<Self, String> {
        let identity_map = omoba_template_ids::moba_map_by_name(FIXTURE_MAP_ID)
            .ok_or_else(|| format!("compiled map {FIXTURE_MAP_ID} is missing"))?;
        let lane = identity_map
            .lanes
            .first()
            .filter(|lane| !lane.waypoints.is_empty())
            .ok_or("compiled map has no lane waypoints")?;
        let source = lane.waypoints[0];
        let target = *lane.waypoints.last().unwrap();
        let lane_length_raw = omoba_template_ids::Fixed64::from_i32(identity_map.lane_length).raw();
        if !(2_000 * 1024..=1_000_000 * 1024).contains(&lane_length_raw) {
            return Err("compiled map lane length is outside the HUD contract".into());
        }
        let hero_id = omoba_template_ids::hero_by_name(FIXTURE_HERO_ID)
            .ok_or_else(|| format!("compiled hero {FIXTURE_HERO_ID} is missing"))?;
        let stats = omoba_template_ids::active_hero_stats(hero_id)
            .ok_or_else(|| format!("compiled hero {FIXTURE_HERO_ID} has no stats"))?;
        if stats.base_hp <= 0 {
            return Err("compiled hero base hp is not positive".into());
        }
        let (birth, base_hp_raw) = load_birth(FIXTURE_HERO_ID)?;
        if birth.id != FIXTURE_HERO_ID {
            return Err("compiled hero id changed during load".into());
        }
        let source_xy = fixed_xy(source);
        let lane_end_xy = fixed_xy(target);
        let mut world = Self {
            communication_enabled: false,
            team_cards_disclosed: false,
            communication_received: false,
            communication_signals: 0,
            birth: birth.clone(),
            base_hp_raw,
            active: birth,
            active_hp_raw: base_hp_raw,
            lane: lane.waypoints.to_vec(),
            source_xy,
            lane_end_xy,
            target_xy: lane_end_xy,
            target_span: TargetSpan::End,
            hide_target: false,
            conceal_target: false,
            lane_length_raw,
            epoch_floor: 1,
            source: None,
            target: None,
            hud_cooldown: false,
            summoner_contract: SummonerContract::Closed,
            recall_contract: RecallContract::Closed,
            item_contract: ItemContract::Closed,
            shop_contract: ShopContract::Closed,
            ward_contract: WardContract::Closed,
            combat_preferences: None,
            input_scheme: None,
            last_ward_renderer_request: 0,
            shop_txn: ShopTxnContract::Closed,
            last_shop_renderer_request: 0,
            last_shop_action_kind: 0,
            presentation_mode: Some(omoba_core::runtime::native::comp::GameMode::Moba),
            td_defeat: false,
            suppress_owner_hero: false,
            replica_tick: 0,
            published_sequence: 1,
            control_sequence: 0,
        };
        world.apply_stage("baseline")?;
        Ok(world)
    }

    fn apply_stage(&mut self, stage: &str) -> Result<(), &'static str> {
        match stage {
            "communication_ready" => {
                self.apply_stage("baseline")?;
                self.communication_enabled = true;
            }
            "communication_hud" => {
                self.apply_stage("communication_ready")?;
                self.project_item(ItemContract::Ready)?;
            }
            "communication_received" => {
                self.apply_stage("communication_ready")?;
                self.communication_received = true;
            }
            "communication_signals" | "communication_signals_expired" => {
                self.apply_stage("communication_ready")?;
                self.communication_signals = if stage == "communication_signals" { 1 } else { 2 };
            }
            "communication_retire" => {
                self.bump_epoch();
                self.apply_stage("baseline")?;
            }
            "input_point" | "input_keyboard" | "input_locked" => {
                let preferences = self.combat_preferences;
                self.apply_stage("baseline")?;
                self.combat_preferences = preferences;
                self.input_scheme = Some(omoba_core::game_proto::InputSchemeOwnerState {
                    schema_version: 1, scheme: if stage == "input_point" { 0 } else { 1 },
                    hero_content_id: "training_luminary".to_owned(),
                    playing_elapsed_raw: if stage == "input_locked" { 600 * 1024 } else { 1024 },
                    lock_after_raw: 600 * 1024, direction_lease_raw: 307,
                    last_press_id: 0, active_press_id: 0, direction_step: None, direction_protocol: 1,
                });
            }
            "combat_defaults" => {
                self.apply_stage("baseline")?;
                self.combat_preferences = Some((0, true));
            }
            "combat_cursor_projection" => self.combat_preferences = Some((1, true)),
            "combat_idle_off_projection" => self.combat_preferences = Some((1, false)),
            "baseline" => {
                self.combat_preferences = None;
                self.input_scheme = None;
                self.communication_enabled = false;
                self.team_cards_disclosed = false;
                self.communication_received = false;
                self.communication_signals = 0;
                self.close_summoner();
                self.close_item_and_recall();
                self.ward_contract = WardContract::Closed;
                self.shop_txn = ShopTxnContract::Closed;
                self.presentation_mode = Some(omoba_core::runtime::native::comp::GameMode::Moba);
                self.td_defeat = false;
                self.suppress_owner_hero = false;
                self.hud_cooldown = false;
                self.hide_target = false;
                self.conceal_target = false;
                self.active = self.birth.clone();
                self.active.mana_pool = None;
                self.active.ability_cooldowns.clear();
                self.active_hp_raw = self.base_hp_raw;
                self.place_target(TargetSpan::End)?;
                self.source = Some(self.actor(SOURCE_RENDER_ID, OWNER_PLAYER_ID, OWNER_TEAM_ID, true, true));
                self.target = Some(self.actor(TARGET_RENDER_ID, ENEMY_PLAYER_ID, ENEMY_TEAM_ID, true, true));
            }
            "remove_target" => {
                self.close_summoner();
                self.target = None;
            }
            "replace_target" => {
                self.close_summoner();
                self.bump_epoch();
                let render_id = if self.target.as_ref().map(|actor| actor.render_id) == Some(TARGET_RENDER_ID)
                {
                    REPLACEMENT_TARGET_RENDER_ID
                } else {
                    TARGET_RENDER_ID
                };
                self.target = Some(self.actor(render_id, ENEMY_PLAYER_ID, ENEMY_TEAM_ID, true, true));
            }
            "source_removal" => self.source = None,
            "source_replacement" => {
                self.bump_epoch();
                let render_id = if self.source.as_ref().map(|actor| actor.render_id) == Some(SOURCE_RENDER_ID)
                {
                    REPLACEMENT_SOURCE_RENDER_ID
                } else {
                    SOURCE_RENDER_ID
                };
                self.source = Some(self.actor(render_id, OWNER_PLAYER_ID, OWNER_TEAM_ID, true, true));
            }
            "death" => {
                let target = self.target.as_mut().ok_or("no_target")?;
                target.alive = false;
                target.attackable = false;
            }
            "source_death" => {
                let source = self.source.as_mut().ok_or("no_source")?;
                source.alive = false;
                source.attackable = false;
            }
            "disclosure_epoch" => {
                if self.source.is_none() && self.target.is_none() {
                    return Err("no_entity");
                }
                self.bump_epoch();
                if let Some(source) = &mut self.source {
                    source.epoch = self.epoch_floor;
                }
                if let Some(target) = &mut self.target {
                    target.epoch = self.epoch_floor;
                }
            }
            "hud_cooldown" => {
                self.close_summoner();
                if self.source.is_none() {
                    return Err("no_source");
                }
                catalog_cooldown(&mut self.active.clone()).map_err(|_| "cooldown_unavailable")?;
                self.hud_cooldown = true;
            }
            // Disclosed later owner state. Luminary birth ranks stay [1, 1, 1, 1].
            "owner_cast_ready" => self.disclose_owner(FIXTURE_HERO_ID, ManaKind::Full, TargetSpan::Near, false, false, ENEMY_TEAM_ID, false)?,
            "owner_upgrade_point" => {
                self.disclose_owner(FIXTURE_HERO_ID, ManaKind::Full, TargetSpan::Near, false, false, ENEMY_TEAM_ID, false)?;
                self.grant_one_birth_level()?;
            }
            "owner_short_mana" => self.disclose_owner(FIXTURE_HERO_ID, ManaKind::Short, TargetSpan::Near, false, false, ENEMY_TEAM_ID, false)?,
            "owner_cooldown" => self.disclose_owner(FIXTURE_HERO_ID, ManaKind::Full, TargetSpan::Near, false, true, ENEMY_TEAM_ID, false)?,
            "owner_hide_target" => self.disclose_owner(FIXTURE_HERO_ID, ManaKind::Full, TargetSpan::Near, true, false, ENEMY_TEAM_ID, false)?,
            "owner_far_unit" => self.disclose_owner(FIXTURE_HERO_ID, ManaKind::Full, TargetSpan::Far, false, false, ENEMY_TEAM_ID, false)?,
            "owner_apprentice" => self.disclose_owner("training_apprentice", ManaKind::Full, TargetSpan::Near, false, false, ENEMY_TEAM_ID, false)?,
            "owner_point_ready" => self.disclose_owner("training_vanguard", ManaKind::Full, TargetSpan::Near, false, false, ENEMY_TEAM_ID, false)?,
            "owner_support_ready" => self.disclose_owner("training_support", ManaKind::Full, TargetSpan::Near, false, false, OWNER_TEAM_ID, false)?,
            "owner_support_team_cards" | "owner_support_team_cards_dead" => {
                self.disclose_owner("training_support", ManaKind::Full, TargetSpan::Near, false, false, OWNER_TEAM_ID, false)?;
                self.team_cards_disclosed = true;
                if stage == "owner_support_team_cards_dead" {
                    let target = self.target.as_mut().ok_or("no_target")?;
                    target.alive = false;
                    target.attackable = false;
                }
            }
            "owner_support_enemy" => self.disclose_owner("training_support", ManaKind::Full, TargetSpan::Near, false, false, ENEMY_TEAM_ID, false)?,
            "owner_support_conceal" => self.disclose_owner("training_support", ManaKind::Full, TargetSpan::Near, false, false, OWNER_TEAM_ID, true)?,
            "owner_support_dead" => {
                self.disclose_owner("training_support", ManaKind::Full, TargetSpan::Near, false, false, OWNER_TEAM_ID, false)?;
                let target = self.target.as_mut().ok_or("no_target")?;
                target.alive = false;
                target.attackable = false;
            }
            "owner_support_neutral" => {
                self.disclose_owner(
                    "training_support",
                    ManaKind::Full,
                    TargetSpan::Near,
                    false,
                    false,
                    NEUTRAL_OTHER_TEAM_ID,
                    false,
                )?;
                let target = self.target.as_mut().ok_or("no_target")?;
                target.faction_kind = omoba_core::runtime::faction_kind_discriminant(
                    &omoba_core::runtime::FactionType::Neutral,
                )
                .ok_or("neutral_faction")?;
                target.alive = true;
                target.attackable = true;
            }
            "owner_summoner_ready" => self.project_summoner(SummonerContract::Ready)?,
            "owner_summoner_cooldown" => self.project_summoner(SummonerContract::Cooldown)?,
            "owner_summoner_unavailable" => self.project_summoner(SummonerContract::Unavailable)?,
            "owner_summoner_malformed" => self.project_summoner(SummonerContract::Malformed)?,
            "owner_summoner_unsupported" => self.project_summoner(SummonerContract::Unsupported)?,
            "owner_recall_ready" => self.project_recall(RecallContract::Ready)?,
            "owner_recall_unavailable" => self.project_recall(RecallContract::Unavailable)?,
            "owner_recall_channel" => self.project_recall(RecallContract::Channel)?,
            "owner_item_ready" => self.project_item(ItemContract::Ready)?,
            "owner_item_cooldown" => self.project_item(ItemContract::Cooldown)?,
            "owner_item_empty" => self.project_item(ItemContract::Empty)?,
            "owner_item_replaced" => self.project_item(ItemContract::Replaced)?,
            "owner_item_far" => self.project_item(ItemContract::Far)?,
            "mode_tower_defense" => self.project_mode(Some(omoba_core::runtime::native::comp::GameMode::TowerDefense), false, false, true)?,
            "mode_td_defeat" => self.project_mode(Some(omoba_core::runtime::native::comp::GameMode::TowerDefense), true, false, true)?,
            "mode_td_no_hero" => self.project_mode(Some(omoba_core::runtime::native::comp::GameMode::TowerDefense), false, true, false)?,
            "mode_moba_dead" => {
                self.project_mode(Some(omoba_core::runtime::native::comp::GameMode::Moba), false, false, false)?;
                let source = self.source.as_mut().ok_or("no_source")?;
                source.alive = false;
                source.attackable = false;
            }
            "mode_unavailable" => self.project_mode(None, false, false, true)?,
            "mode_td_epoch" => {
                self.project_mode(Some(omoba_core::runtime::native::comp::GameMode::TowerDefense), false, false, true)?;
                if self.source.is_none() && self.target.is_none() {
                    return Err("no_entity");
                }
                self.bump_epoch();
                if let Some(source) = &mut self.source {
                    source.epoch = self.epoch_floor;
                }
                if let Some(target) = &mut self.target {
                    target.epoch = self.epoch_floor;
                }
            }
            "owner_shop_ready" => {
                self.project_item(ItemContract::Ready)?;
                if self.presentation_mode != Some(omoba_core::runtime::native::comp::GameMode::Moba) {
                    return Err("shop_mode");
                }
                if self.source.as_ref().map(|actor| actor.render_id) != Some(SOURCE_RENDER_ID) {
                    return Err("shop_source");
                }
                self.shop_contract = ShopContract::Ready;
            }
            "controlled_owner_ward_projection" => self.project_ward(WardContract::Ready)?,
            "controlled_owner_ward_dead" => self.project_ward(WardContract::Dead)?,
            "controlled_owner_ward_malformed" => self.project_ward(WardContract::Malformed)?,
            "controlled_owner_ward_settled" => self.project_ward(WardContract::Settled)?,
            "controlled_owner_ward_rejected" => self.project_ward(WardContract::Rejected)?,
            "controlled_owner_ward_uncertain" => self.project_ward(WardContract::Uncertain)?,
            "controlled_owner_ward_clear" => self.project_ward(WardContract::Closed)?,
            "controlled_owner_ward_forwarded" => self.project_ward(WardContract::Forwarded)?,
            "controlled_owner_ward_uncertain_mapped" => self.project_ward(WardContract::UncertainMapped)?,
            "controlled_owner_ward_settled_mapped" => self.project_ward(WardContract::SettledMapped)?,
            "controlled_owner_ward_rejected_mapped" => self.project_ward(WardContract::RejectedMapped)?,
            "controlled_owner_ward_local_rejected" => self.project_ward(WardContract::LocalRejected)?,
            "controlled_owner_ward_same_epoch_source" => {
                if self.source.is_none() {
                    return Err("no_source");
                }
                let render_id = if self.source.as_ref().map(|actor| actor.render_id) == Some(SOURCE_RENDER_ID) {
                    REPLACEMENT_SOURCE_RENDER_ID
                } else {
                    SOURCE_RENDER_ID
                };
                let mut actor = self.actor(render_id, OWNER_PLAYER_ID, OWNER_TEAM_ID, true, true);
                actor.epoch = self.epoch_floor.saturating_add(64);
                self.source = Some(actor);
            }
            "controlled_owner_shop_current" => {
                self.project_shop_current()?;
                self.shop_txn = ShopTxnContract::Current;
            }
            "controlled_owner_shop_admitted" => {
                self.project_shop_current()?;
                self.arm_shop_txn(ShopTxnContract::Admitted)?;
            }
            "controlled_owner_shop_uncertain" => {
                self.project_shop_current()?;
                self.arm_shop_txn(ShopTxnContract::Uncertain)?;
            }
            "controlled_owner_shop_settled" => {
                self.project_shop_current()?;
                self.arm_shop_txn(ShopTxnContract::Settled)?;
            }
            "controlled_owner_shop_rejected" => {
                self.project_shop_current()?;
                self.arm_shop_txn(ShopTxnContract::Rejected)?;
            }
            "controlled_owner_shop_stale" => {
                self.project_shop_current()?;
                self.arm_shop_txn(ShopTxnContract::Stale)?;
            }
            "controlled_owner_shop_epoch" => {
                self.project_shop_current()?;
                self.bump_epoch();
                if let Some(source) = &mut self.source {
                    source.epoch = self.epoch_floor;
                }
                if let Some(target) = &mut self.target {
                    target.epoch = self.epoch_floor;
                }
                self.shop_txn = ShopTxnContract::EpochClear;
            }
            "controlled_owner_shop_full" => {
                self.project_shop_current()?;
                self.shop_txn = ShopTxnContract::Full;
            }
            _ => return Err("unknown_stage"),
        }
        Ok(())
    }

    fn disclose_owner(
        &mut self,
        hero_id: &str,
        mana: ManaKind,
        span: TargetSpan,
        hide_target: bool,
        cooldown: bool,
        target_team: u32,
        conceal_target: bool,
    ) -> Result<(), &'static str> {
        self.close_summoner();
        self.close_item_and_recall();
        self.team_cards_disclosed = false;
        if self.source.is_none() || self.target.is_none() {
            return Err("no_source");
        }
        let (mut hero, hp_raw) = if hero_id == FIXTURE_HERO_ID {
            (self.birth.clone(), self.base_hp_raw)
        } else {
            load_birth(hero_id).map_err(|_| "hero_unavailable")?
        };
        hero.ability_cooldowns.clear();
        match mana {
            ManaKind::Absent => hero.mana_pool = None,
            ManaKind::Full => set_full_mana(&mut hero)?,
            ManaKind::Short => set_short_mana(&mut hero)?,
        }
        if cooldown {
            catalog_cooldown(&mut hero).map_err(|_| "cooldown_unavailable")?;
        }
        self.active = hero;
        if let Some(state) = self.input_scheme.as_mut() { state.hero_content_id = hero_id.to_owned(); }
        self.active_hp_raw = hp_raw;
        self.hud_cooldown = cooldown;
        self.hide_target = hide_target;
        self.conceal_target = conceal_target;
        self.place_target(span)?;
        if let Some(source) = &mut self.source {
            source.alive = true;
            source.attackable = true;
        }
        if let Some(target) = &mut self.target {
            target.alive = true;
            target.attackable = !hide_target;
            target.team_id = target_team;
            target.owner_player_id = ENEMY_PLAYER_ID;
            target.faction_kind = omoba_core::runtime::faction_kind_discriminant(
                &omoba_core::runtime::FactionType::Player,
            )
            .ok_or("player_faction")?;
        }
        Ok(())
    }

    /// One legal level from the hero's current `experience_to_next`. Birth stays rank 1 with zero points.
    fn grant_one_birth_level(&mut self) -> Result<(), &'static str> {
        if self.active.level != 1 || self.active.experience != 0 || self.active.skill_points != 0 {
            return Err("upgrade_point_birth");
        }
        let ranks = self.active.ability_levels.clone();
        let threshold = u32::try_from(self.active.experience_to_next).map_err(|_| "experience_threshold")?;
        // The fixture keeps the legacy experience curve (no game rules).
        if threshold == 0 || !self.active.add_moba_experience(threshold, None) {
            return Err("experience_did_not_level");
        }
        if self.active.level != 2
            || self.active.experience != 0
            || self.active.skill_points != 1
            || self.active.ability_levels != ranks
        {
            return Err("experience_grant");
        }
        Ok(())
    }

    fn place_target(&mut self, span: TargetSpan) -> Result<(), &'static str> {
        self.target_span = span;
        self.target_xy = match span {
            TargetSpan::End => self.lane_end_xy,
            TargetSpan::Near => point_at_separation(&self.lane, 200).ok_or("lane_too_short")?,
            TargetSpan::Far => point_at_separation(&self.lane, 900).ok_or("lane_too_short")?,
        };
        Ok(())
    }

    fn actor(&self, render_id: u64, owner_player_id: u32, team_id: u32, alive: bool, attackable: bool) -> Actor {
        Actor {
            render_id,
            epoch: self.epoch_floor,
            owner_player_id,
            team_id,
            alive,
            attackable,
            faction_kind: 1,
        }
    }

    fn bump_epoch(&mut self) {
        self.epoch_floor = self.epoch_floor.saturating_add(1);
    }

    fn close_summoner(&mut self) {
        self.summoner_contract = SummonerContract::Closed;
    }

    fn close_item_and_recall(&mut self) {
        self.item_contract = ItemContract::Closed;
        self.recall_contract = RecallContract::Closed;
        self.shop_contract = ShopContract::Closed;
    }

    /// Keeps the current source identity. Dead retains the same charge projection and drops life.
    fn project_ward(&mut self, contract: WardContract) -> Result<(), &'static str> {
        if self.source.is_none() {
            return Err("no_source");
        }
        if contract == WardContract::Dead {
            let source = self.source.as_mut().ok_or("no_source")?;
            source.alive = false;
            source.attackable = false;
        } else if let Some(source) = &mut self.source {
            source.alive = true;
            source.attackable = true;
        }
        if !matches!(contract, WardContract::Closed | WardContract::Malformed) {
            let row = omoba_template_ids::trinket_by_name("training_scout_lantern_v1").ok_or("ward_catalog")?;
            if row.catalog_id == 0 || row.initial_charges == 0 || row.max_charges < row.initial_charges {
                return Err("ward_catalog");
            }
        }
        self.ward_contract = contract;
        Ok(())
    }

    /// Opens the shop on whatever source is current. It does not restore render id 11,
    /// bump the view, close the ward projection, or credit gold.
    fn project_shop_current(&mut self) -> Result<(), &'static str> {
        if self.source.is_none() {
            return Err("no_source");
        }
        if self.presentation_mode != Some(omoba_core::runtime::native::comp::GameMode::Moba) {
            return Err("shop_mode");
        }
        if self.item_contract == ItemContract::Closed {
            let _ = item_inventory(ItemContract::Ready)?;
            self.item_contract = ItemContract::Ready;
        }
        self.shop_contract = ShopContract::Ready;
        Ok(())
    }

    fn arm_shop_txn(&mut self, contract: ShopTxnContract) -> Result<(), &'static str> {
        if self.last_shop_renderer_request == 0 {
            return Err("shop_request_unseen");
        }
        match contract {
            ShopTxnContract::Admitted | ShopTxnContract::Uncertain | ShopTxnContract::Settled
                if self.last_shop_action_kind != 17 =>
            {
                return Err("shop_request_unseen");
            }
            ShopTxnContract::Rejected if self.last_shop_action_kind != 18 => return Err("shop_request_unseen"),
            ShopTxnContract::Stale if self.last_shop_action_kind != 17 => return Err("shop_request_unseen"),
            _ => {}
        }
        self.shop_txn = contract;
        Ok(())
    }

    /// Bytes are rebuilt from the current epoch so a later bump cannot keep a stale epoch.
    fn owner_ward_bytes(&self) -> Result<(u32, Vec<u8>), &'static str> {
        use omoba_core::runtime::ward_transport::{
            encode_owner_ward_private, OwnerWardOutcome, OwnerWardOutcomeBlock, OwnerWardProjection,
            OWNER_WARD_OUTCOME_REJECTED, OWNER_WARD_OUTCOME_SETTLED, WARD_PROTOCOL_VERSION,
        };
        match self.ward_contract {
            WardContract::Closed => Ok((0, Vec::new())),
            WardContract::Malformed => Ok((WARD_PROTOCOL_VERSION, vec![2, 0, 0, 0])),
            WardContract::Ready
            | WardContract::Dead
            | WardContract::Settled
            | WardContract::Rejected
            | WardContract::Uncertain
            | WardContract::Forwarded
            | WardContract::UncertainMapped
            | WardContract::SettledMapped
            | WardContract::RejectedMapped
            | WardContract::LocalRejected => {
                let row = omoba_template_ids::trinket_by_name("training_scout_lantern_v1").ok_or("ward_catalog")?;
                let projection = OwnerWardProjection {
                    player_id: OWNER_PLAYER_ID,
                    view_epoch: self.epoch_floor,
                    catalog_id: row.catalog_id,
                    charges: row.initial_charges,
                    max_charges: row.max_charges,
                    recharge_remaining_raw: 0,
                    revision: 1,
                    placement_capable: true,
                    elapsed_raw: 0,
                };
                let mut outcomes = OwnerWardOutcomeBlock::default();
                match self.ward_contract {
                    WardContract::Settled => {
                        outcomes.count = 1;
                        outcomes.rows[0] = OwnerWardOutcome {
                            admitted_correlation: 7,
                            actor_generation: 3,
                            stage: OWNER_WARD_OUTCOME_SETTLED,
                            slot: 0,
                        };
                    }
                    WardContract::Rejected => {
                        outcomes.count = 1;
                        outcomes.rows[0] = OwnerWardOutcome {
                            admitted_correlation: 7,
                            actor_generation: 3,
                            stage: OWNER_WARD_OUTCOME_REJECTED,
                            slot: 0,
                        };
                    }
                    WardContract::Uncertain => outcomes.uncertain_through = 7,
                    WardContract::SettledMapped | WardContract::RejectedMapped => {
                        let actor = self.source.as_ref().map(|actor| actor.epoch as u32).unwrap_or(0);
                        outcomes.count = 1;
                        outcomes.rows[0] = OwnerWardOutcome {
                            admitted_correlation: 901,
                            actor_generation: actor,
                            stage: if self.ward_contract == WardContract::SettledMapped {
                                OWNER_WARD_OUTCOME_SETTLED
                            } else {
                                OWNER_WARD_OUTCOME_REJECTED
                            },
                            slot: 0,
                        };
                    }
                    _ => {}
                }
                let bytes = encode_owner_ward_private(&projection, &outcomes).map_err(|_| "ward_encode")?;
                Ok((WARD_PROTOCOL_VERSION, bytes))
            }
        }
    }

    /// Maps the last renderer request onto authority 901. A mapped stage with no request fails closed.
    fn ward_correlation_drafts(&self) -> Result<Vec<omoba_core::runtime::ward_transport::WardCorrelationDraft>, &'static str> {
        use omoba_core::runtime::ward_transport::{
            WardCorrelationDraft, WARD_PHASE_APPLIED, WARD_PHASE_FORWARDED, WARD_PHASE_REJECTED, WARD_PHASE_UNCERTAIN,
        };
        let (authority, phase) = match self.ward_contract {
            WardContract::Forwarded => (901, WARD_PHASE_FORWARDED),
            WardContract::UncertainMapped => (901, WARD_PHASE_UNCERTAIN),
            WardContract::SettledMapped | WardContract::RejectedMapped => (901, WARD_PHASE_APPLIED),
            WardContract::LocalRejected => (0, WARD_PHASE_REJECTED),
            _ => return Ok(Vec::new()),
        };
        if self.last_ward_renderer_request == 0 {
            return Err("ward_request_unseen");
        }
        Ok(vec![WardCorrelationDraft {
            player_id: OWNER_PLAYER_ID,
            view_epoch: self.epoch_floor,
            actor_generation: self.source.as_ref().map(|actor| actor.epoch as u32).unwrap_or(0),
            renderer_request: self.last_ward_renderer_request,
            authority_input: authority,
            phase,
        }])
    }

    /// Ready revives the current source without a new disclosure epoch or a QWER rewrite.
    fn project_summoner(&mut self, contract: SummonerContract) -> Result<(), &'static str> {
        if self.source.is_none() {
            return Err("no_source");
        }
        self.close_item_and_recall();
        let _ = catalog_summoner_pair()?;
        if contract == SummonerContract::Ready {
            let source = self.source.as_mut().ok_or("no_source")?;
            source.alive = true;
            source.attackable = true;
        }
        self.summoner_contract = contract;
        Ok(())
    }

    fn summoner_slots(
        &self,
    ) -> Result<Option<[omoba_core::runtime::summoner_transport::ProjectedSummonerSlot; 2]>, ClientRuntimeError>
    {
        if self.summoner_contract == SummonerContract::Closed {
            return Ok(None);
        }
        let (blink, heal) = catalog_summoner_pair().map_err(|reason| ClientRuntimeError::Ipc(reason.into()))?;
        let slot = |catalog_id, cooldown_raw| {
            omoba_core::runtime::summoner_transport::ProjectedSummonerSlot { catalog_id, cooldown_raw }
        };
        let half = blink.cooldown.raw() / 2;
        let slots = match self.summoner_contract {
            SummonerContract::Closed => return Ok(None),
            SummonerContract::Ready | SummonerContract::Unsupported => [slot(blink.catalog_id, 0), slot(heal.catalog_id, 0)],
            SummonerContract::Cooldown => [slot(blink.catalog_id, half), slot(heal.catalog_id, 0)],
            SummonerContract::Unavailable => [slot(0, 0), slot(0, 0)],
            SummonerContract::Malformed => [slot(99, 0), slot(heal.catalog_id, 0)],
        };
        Ok(Some(slots))
    }

    /// Revive the current source. Ready and Channel negotiate protocol 2. Remaining is a HUD metric only.
    fn project_recall(&mut self, contract: RecallContract) -> Result<(), &'static str> {
        if self.source.is_none() {
            return Err("no_source");
        }
        if contract.requests_protocol() {
            if !fixture_recall_protocol_enabled(true, omoba_template_ids::CONTENT_CATALOG_DATA_HASH) {
                return Err("recall_protocol");
            }
            let source = self.source.as_mut().ok_or("no_source")?;
            source.alive = true;
            source.attackable = true;
        }
        if contract == RecallContract::Channel {
            let remaining = 3 * 1024;
            let max = i64::from(omoba_template_ids::MOBA_RECALL_CHANNEL_SECONDS) * 1024;
            if !(1..=max).contains(&remaining) {
                return Err("recall_remaining");
            }
        }
        self.recall_contract = contract;
        Ok(())
    }

    /// Six authored slots. Far keeps the same catalog and moves the disclosed enemy out of item range.
    fn project_item(&mut self, contract: ItemContract) -> Result<(), &'static str> {
        if self.source.is_none() || self.target.is_none() {
            return Err("no_source");
        }
        let _ = item_inventory(contract)?;
        self.hide_target = false;
        self.conceal_target = false;
        self.place_target(if contract == ItemContract::Far { TargetSpan::Far } else { TargetSpan::Near })?;
        if let Some(source) = &mut self.source {
            source.alive = true;
            source.attackable = true;
        }
        if let Some(target) = &mut self.target {
            target.alive = true;
            target.attackable = true;
            target.team_id = ENEMY_TEAM_ID;
            target.owner_player_id = ENEMY_PLAYER_ID;
            target.faction_kind = omoba_core::runtime::faction_kind_discriminant(
                &omoba_core::runtime::FactionType::Player,
            )
            .ok_or("player_faction")?;
        }
        self.item_contract = contract;
        self.shop_contract = ShopContract::Closed;
        Ok(())
    }

    /// Typed fixture mode. `td_defeat` is a marker only and does not change the snapshot mode.
    /// Suppressing the owner hero omits that entity and leaves the actor in place for the control ack.
    fn project_mode(
        &mut self,
        mode: Option<omoba_core::runtime::native::comp::GameMode>,
        td_defeat: bool,
        suppress_owner_hero: bool,
        revive_source: bool,
    ) -> Result<(), &'static str> {
        self.presentation_mode = mode;
        self.td_defeat = td_defeat;
        self.suppress_owner_hero = suppress_owner_hero;
        self.shop_contract = ShopContract::Closed;
        if revive_source {
            if let Some(source) = &mut self.source {
                source.alive = true;
                source.attackable = true;
            }
        }
        Ok(())
    }

    fn summoner_projection(
        &self,
    ) -> Result<omoba_core::runtime::summoner_transport::DisclosedSummonerProjection, ClientRuntimeError>
    {
        let Some(slots) = self.summoner_slots()? else {
            return Ok(omoba_core::runtime::summoner_transport::DisclosedSummonerProjection::default());
        };
        Ok(omoba_core::runtime::summoner_transport::DisclosedSummonerProjection {
            players: vec![omoba_core::runtime::summoner_transport::ProjectedSummonerPlayer {
                player_id: OWNER_PLAYER_ID,
                slots,
            }],
        })
    }

    fn to_snapshot(&self) -> Result<FilteredRenderSnapshot, ClientRuntimeError> {
        let mut entities = Vec::new();
        if let Some(source) = &self.source {
            if !self.suppress_owner_hero {
                entities.push(self.entity(source, self.source_xy)?);
            }
        }
        if let Some(target) = &self.target {
            entities.push(self.entity(target, self.target_xy)?);
        }
        if entities.iter().any(|entity| entity.replica_id == 0) {
            return Err(ClientRuntimeError::Ipc(
                "presentation render id 0 is invalid".into(),
            ));
        }
        Ok(FilteredRenderSnapshot {
            team_id: OWNER_TEAM_ID,
            replica_tick: self.replica_tick.saturating_add(1),
            entities,
            public_events: vec![
                metric(0, SINGLE_LANE_PHASE_METRIC_ID, 1),
                metric(0, SINGLE_LANE_ELAPSED_METRIC_ID, 1024),
                metric(0, SINGLE_LANE_LENGTH_METRIC_ID, self.lane_length_raw),
            ],
            external_effects: Vec::new(),
            memory_directives: Vec::new(),
            remembered_presentations: BTreeMap::new(),
            hero_attributes: Default::default(), disclosed_summoners: self.summoner_projection()?,
            combat_preferences: self.combat_preferences.or_else(|| self.input_scheme.as_ref().map(|_| (0, true))).map(|(acquisition, idle_auto_attack)|
                omoba_core::game_proto::CombatPreferencesProjection { schema_version: 1, view_epoch: self.epoch_floor,
                    rows: vec![omoba_core::game_proto::CombatPreferencesRow {
                        player_id: OWNER_PLAYER_ID, acquisition, idle_auto_attack,
                        input_scheme: self.input_scheme.clone().map(|mut state| {
                            // Explicit synthetic readiness for this exact snapshot tick.
                            // Neutral state never acknowledges a renderer press or lease.
                            if state.scheme == 1 {
                                state.direction_step = Some(omoba_core::game_proto::InputSchemeDirectionStep {
                                    tick: self.replica_tick.saturating_add(1), scheme: 1,
                                    direction_x_raw: 0, direction_y_raw: 0, press_id: 0,
                                    navigation_reset: false, attack_reset: false,
                                });
                            }
                            state
                        }) }] }),
            authoritative_mode: match self.presentation_mode {
                Some(mode) => omoba_core::runtime::authoritative_mode::presentation(mode, self.epoch_floor),
                None => None,
            },
        })
        .map(|mut snapshot| {
            if self.communication_enabled || self.team_cards_disclosed {
                // Explicit synthetic PUBLIC roster, not inferred from visible actors or chat.
                // This is disclosed fixture state, never an input receipt or authority ACK.
                use omoba_core::runtime::{single_lane_player_metric, SINGLE_LANE_SCORE_COUNT_METRIC_ID,
                    SINGLE_LANE_SCORE_TEAM_METRIC_ID, SINGLE_LANE_SCORE_KILLS_METRIC_ID,
                    SINGLE_LANE_SCORE_DEATHS_METRIC_ID, SINGLE_LANE_SCORE_ASSISTS_METRIC_ID};
                let roster: &[(u32, i64)] = if self.team_cards_disclosed { &[(1, 1), (2, 1), (3, 2)] }
                    else { &[(1, 1), (2, 2)] };
                snapshot.public_events.push(metric(0, SINGLE_LANE_SCORE_COUNT_METRIC_ID, roster.len() as i64));
                for &(player, team) in roster {
                    for (namespace, value) in [(SINGLE_LANE_SCORE_TEAM_METRIC_ID, team),
                        (SINGLE_LANE_SCORE_KILLS_METRIC_ID, 0), (SINGLE_LANE_SCORE_DEATHS_METRIC_ID, 0),
                        (SINGLE_LANE_SCORE_ASSISTS_METRIC_ID, 0)] {
                        snapshot.public_events.push(metric(0, single_lane_player_metric(namespace, player), value));
                    }
                }
            }
            if let Some(event) = self.owner_economy_event() {
                snapshot.public_events.push(event);
            }
            if let Some(event) = self.recall_metric_event() {
                snapshot.public_events.push(event);
            }
            let tick = snapshot.replica_tick;
            snapshot.public_events.extend(self.owner_mana_cost_events(tick));
            snapshot
        })
    }

    /// Team-scoped current ability prices, the same rows the authority publishes in
    /// `emit_owner_mana_costs`. The owner HUD only trusts a price bound to this exact tick,
    /// ability and rank; without these rows every slot reads "price unknown" and a short-mana
    /// stage could never be refused before the wire. The fixture has no buffs, so the price is
    /// the compiled rank cost. An absent or dead owner publishes binding 0 and cost -1.
    fn owner_mana_cost_events(&self, replica_tick: u64) -> Vec<TeamPublicEvent> {
        use omoba_core::runtime::{HERO_ABILITY_SLOTS, MANA_COST_BINDING, MANA_COST_RAW, MANA_COST_TICK};
        let row = |namespace, value| metric(OWNER_TEAM_ID, single_lane_player_metric(namespace, OWNER_PLAYER_ID), value);
        let live = self.source.as_ref().is_some_and(|source| source.alive)
            && !self.suppress_owner_hero
            && self.active.mana_pool.is_some();
        let mut events = vec![row(MANA_COST_TICK, i64::try_from(replica_tick).unwrap_or(-1))];
        for slot in 0..self.active.abilities.len().clamp(4, HERO_ABILITY_SLOTS) {
            let priced = live.then(|| self.active.abilities.get(slot)).flatten().and_then(|id| {
                let rank = u8::try_from(self.active.ability_levels.get(id).copied().unwrap_or(0)).ok().filter(|rank| *rank >= 1)?;
                let catalog = omoba_template_ids::ability_by_name(id)?;
                let cost = omoba_template_ids::active_ability_const(catalog)?
                    .levels
                    .get(usize::from(rank) - 1)?
                    .mana_cost
                    .raw();
                Some(((i64::from(catalog.raw()) << 8) | i64::from(rank), cost))
            });
            let (binding, cost) = priced.unwrap_or((0, -1));
            events.push(row(MANA_COST_BINDING[slot], binding));
            events.push(row(MANA_COST_RAW[slot], cost));
        }
        events
    }

    fn owner_economy_event(&self) -> Option<TeamPublicEvent> {
        let (item_ids, cooldown_bits) = item_inventory(self.item_contract).ok()?;
        let shop_open = self.shop_contract == ShopContract::Ready;
        // The synthetic owner's funds stay 0. Opening the shop does not credit or debit gold.
        let state = omoba_core::runtime::native::economy_projection::OwnerEconomyState {
            player_id: OWNER_PLAYER_ID,
            shop_available: shop_open,
            shop_reason: if shop_open {
                omoba_core::runtime::native::economy_projection::SHOP_REASON_AVAILABLE
            } else {
                omoba_core::runtime::native::economy_projection::SHOP_REASON_HERO_ABSENT
            },
            shield_remaining_raw: 0,
            economy: omoba_core::runtime::native::economy_projection::CommittedEconomyState {
                gold: 0,
                item_ids,
                cooldown_bits,
                effect_bits: [0; 10],
                dirty: false,
            },
        };
        let sanitized_payload = state.encode();
        omoba_core::runtime::native::economy_projection::OwnerEconomyState::decode(&sanitized_payload)?;
        Some(TeamPublicEvent {
            event_kind: FactKind::OwnerEconomy as u32,
            sanitized_payload,
            ..Default::default()
        })
    }

    fn recall_metric_event(&self) -> Option<TeamPublicEvent> {
        if self.recall_contract != RecallContract::Channel {
            return None;
        }
        Some(metric(
            OWNER_TEAM_ID,
            single_lane_player_metric(SINGLE_LANE_RECALL_REMAINING_METRIC_ID, OWNER_PLAYER_ID),
            3 * 1024,
        ))
    }

    fn entity(&self, actor: &Actor, xy: (i64, i64)) -> Result<FilteredRenderEntity, ClientRuntimeError> {
        let owner = actor.owner_player_id == OWNER_PLAYER_ID;
        let mut hero = if owner { self.active.clone() } else { self.birth.clone() };
        if self.hud_cooldown && owner {
            catalog_cooldown(&mut hero).map_err(ClientRuntimeError::Ipc)?;
        }
        let max_hp = if owner { self.active_hp_raw } else { self.base_hp_raw };
        let hp = if actor.alive { max_hp } else { 0 };
        let property = encode_disclosed_property(&CProperty {
            hp: omoba_template_ids::Fixed64::from_raw(hp),
            mhp: omoba_template_ids::Fixed64::from_raw(max_hp),
            msd: omoba_template_ids::Fixed64::ZERO,
            def_physic: omoba_template_ids::Fixed64::ZERO,
            def_magic: omoba_template_ids::Fixed64::ZERO,
        });
        if property.len() != 40 {
            return Err(ClientRuntimeError::Ipc("health codec is not 40 bytes".into()));
        }
        let render = encode_demo_render_state(DemoRenderState {
            x_raw: xy.0,
            y_raw: xy.1,
            team_id: actor.team_id,
            kind: 1,
            owner_player_id: actor.owner_player_id,
        });
        if render.len() != 25 {
            return Err(ClientRuntimeError::Ipc("render codec is not 25 bytes".into()));
        }
        let hero_bytes = serde_json::to_vec(&hero)
            .map_err(|error| ClientRuntimeError::Ipc(error.to_string()))?;
        let mut components = BTreeMap::new();
        components.insert(DEMO_RENDER_COMPONENT_SCHEMA_ID, render);
        components.insert(DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property);
        if owner || !self.conceal_target {
            let combat = encode_disclosed_combat_relation(DisclosedCombatRelation {
                faction_kind: actor.faction_kind,
                team_id: i32::try_from(actor.team_id).unwrap_or(i32::MAX),
                attackable: actor.alive && actor.attackable,
                alive: actor.alive,
            })
            .ok_or_else(|| ClientRuntimeError::Ipc("combat relation codec rejected the row".into()))?;
            components.insert(DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, combat);
        }
        components.insert(DISCLOSED_HERO_COMPONENT_SCHEMA_ID, hero_bytes);
        Ok(FilteredRenderEntity {
            replica_id: actor.render_id,
            disclosure_epoch: actor.epoch,
            entity_kind: 1,
            components,
        })
    }
}

enum ManaKind {
    Absent,
    Full,
    Short,
}

fn fixed_xy(point: (i32, i32)) -> (i64, i64) {
    (
        omoba_template_ids::Fixed64::from_i32(point.0).raw(),
        omoba_template_ids::Fixed64::from_i32(point.1).raw(),
    )
}

fn load_birth(hero_id: &str) -> Result<(omoba_core::runtime::Hero, i64), String> {
    let id = omoba_template_ids::hero_by_name(hero_id)
        .ok_or_else(|| format!("compiled hero {hero_id} is missing"))?;
    let stats = omoba_template_ids::active_hero_stats(id)
        .ok_or_else(|| format!("compiled hero {hero_id} has no stats"))?;
    if stats.base_hp <= 0 {
        return Err(format!("compiled hero {hero_id} base hp is not positive"));
    }
    let mut hero = omoba_core::runtime::Hero::from_campaign_data(
        &omoba_core::runtime::native::scene::import_campaign::HeroJD {
            id: hero_id.to_owned(),
            abilities: Vec::new(),
        },
    );
    omoba_core::runtime::apply_moba_birth_loadout(&mut hero, stats.moba_loadout)
        .map_err(|error| format!("compiled birth loadout: {error}"))?;
    Ok((hero, omoba_template_ids::Fixed64::from_i32(stats.base_hp).raw()))
}

fn set_full_mana(hero: &mut omoba_core::runtime::Hero) -> Result<(), &'static str> {
    let capacity = hero.moba_mana_capacity().map_err(|_| "mana_capacity")?;
    hero.mana_pool = Some(
        omoba_core::runtime::ability_runtime::ManaPool::full(capacity).map_err(|_| "mana_pool")?,
    );
    Ok(())
}

fn set_short_mana(hero: &mut omoba_core::runtime::Hero) -> Result<(), &'static str> {
    let maximum = hero.moba_mana_capacity().map_err(|_| "mana_capacity")?.raw();
    let ability_id = hero.abilities.first().ok_or("no_ability")?;
    let rank = hero.ability_levels.get(ability_id).copied().unwrap_or(0);
    if rank < 1 {
        return Err("unlearned_mana");
    }
    let cost = omoba_template_ids::ability_by_name(ability_id)
        .and_then(omoba_template_ids::active_ability_const)
        .and_then(|definition| definition.levels.get((rank as usize) - 1))
        .map(|level| level.mana_cost.raw())
        .ok_or("mana_cost_missing")?;
    if !(0 < cost && cost < maximum) {
        return Err("mana_cost_unusable");
    }
    hero.mana_pool = Some(
        omoba_core::runtime::ability_runtime::ManaPool::from_raw_state(cost - 1, maximum, 0)
            .map_err(|_| "mana_pool")?,
    );
    Ok(())
}

/// First lane point whose straight-line separation from the first waypoint reaches `separation_world`.
/// The point stays on the compiled polyline. A lane that never reaches that separation fails closed.
fn point_at_separation(lane: &[(i32, i32)], separation_world: i32) -> Option<(i64, i64)> {
    if lane.is_empty() || separation_world < 0 {
        return None;
    }
    let origin = fixed_xy(lane[0]);
    let need = i128::from(separation_world) * 1024;
    let need_sq = need * need;
    let mut prev = origin;
    for point in lane.iter().skip(1) {
        let next = fixed_xy(*point);
        if separation_sq(origin, next) >= need_sq {
            return Some(segment_separation(origin, prev, next, need_sq));
        }
        prev = next;
    }
    None
}

fn separation_sq(origin: (i64, i64), point: (i64, i64)) -> i128 {
    let dx = i128::from(point.0) - i128::from(origin.0);
    let dy = i128::from(point.1) - i128::from(origin.1);
    dx * dx + dy * dy
}

fn segment_separation(origin: (i64, i64), prev: (i64, i64), next: (i64, i64), need_sq: i128) -> (i64, i64) {
    let mut lo = 0i128;
    let mut hi = 1_000_000i128;
    let dx = i128::from(next.0) - i128::from(prev.0);
    let dy = i128::from(next.1) - i128::from(prev.1);
    while lo < hi {
        let mid = (lo + hi) / 2;
        let x = i128::from(prev.0) + dx * mid / 1_000_000;
        let y = i128::from(prev.1) + dy * mid / 1_000_000;
        let sx = x - i128::from(origin.0);
        let sy = y - i128::from(origin.1);
        if sx * sx + sy * sy >= need_sq {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    (
        (i128::from(prev.0) + dx * lo / 1_000_000) as i64,
        (i128::from(prev.1) + dy * lo / 1_000_000) as i64,
    )
}

fn catalog_cooldown(hero: &mut omoba_core::runtime::Hero) -> Result<(), String> {
    let ability_id = hero
        .abilities
        .first()
        .cloned()
        .ok_or_else(|| "compiled hero has no ability".to_owned())?;
    let definition = omoba_template_ids::ability_by_name(&ability_id)
        .and_then(omoba_template_ids::active_ability_const)
        .ok_or_else(|| format!("missing compiled ability {ability_id}"))?;
    let current = hero.ability_levels.get(&ability_id).copied().unwrap_or(0).max(0) as u32;
    let rank = if current >= 1 { current } else { 1 };
    if rank == 0 || rank > u32::from(definition.max_level) {
        return Err(format!("compiled ability {ability_id} has no usable rank"));
    }
    let level = definition
        .levels
        .get((rank - 1) as usize)
        .ok_or_else(|| format!("compiled ability {ability_id} is missing rank {rank}"))?;
    let total = level.cooldown.raw();
    if total <= 0 {
        return Err(format!("compiled ability {ability_id} cooldown is not positive"));
    }
    let cooled = (total / 2).clamp(1, total);
    hero.ability_levels.insert(ability_id.clone(), i32::try_from(rank).unwrap_or(1));
    hero.ability_cooldowns
        .insert(ability_id, omoba_template_ids::Fixed64::from_raw(cooled));
    Ok(())
}

fn ready_frame(
    config: &ClientRuntimeConfig,
    identity: &CompiledContentIdentity,
) -> Result<RendererIpcEnvelope, ClientRuntimeError> {
    let map = omoba_template_ids::moba_map_by_name(&identity.moba_map_id)
        .ok_or_else(|| ClientRuntimeError::Config("compiled map missing".into()))?;
    let start = TeamGameStart {
        server_tick: 0,
        replica_start_tick: 0,
        tick_rate_hz: TICK_RATE_HZ,
        public_metadata: omoba_core::runtime::moba_map_layout::bootstrap_metadata(map),
        ..Default::default()
    };
    presentation_bridge::ready_envelope_for_start(1, config, &start)
}

fn ready_matches(ready: &RendererIpcEnvelope, identity: &CompiledContentIdentity) -> bool {
    let Some(renderer_ipc_envelope::Payload::RuntimeReady(ready)) = &ready.payload else {
        return false;
    };
    ready.content_hash == identity.content_hash
        && ready.moba_map_id == identity.moba_map_id
        && ready.moba_map_catalog_hash == identity.moba_map_catalog_hash
        && ready.player_id == OWNER_PLAYER_ID
        && ready.team_id == OWNER_TEAM_ID
}

fn fixture_config(evidence_dir: &Path, identity: &CompiledContentIdentity) -> ClientRuntimeConfig {
    ClientRuntimeConfig {
        player_id: OWNER_PLAYER_ID,
        team_id: OWNER_TEAM_ID,
        player_name: "fixture-owner".into(),
        server_addr: "127.0.0.1:9".parse().expect("loopback placeholder"),
        presentation_bind: "127.0.0.1:0".parse().expect("loopback ephemeral"),
        presentation_hz: 60,
        evidence_dir: Some(evidence_dir.to_owned()),
        test_mode: false,
        protocol_version: omoba_core::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION,
        content_hash: identity.content_hash.clone(),
        scripted_move_tick: None,
        scripted_move_interval_ticks: None,
        scripted_stall_tick: None,
        scripted_stall_ms: None,
        scripted_hidden_target_tick: None,
        screenshot_tick: None,
        fault_tick: None,
        rebase_probe_tick: None,
        shutdown_file: None,
    }
}

fn shop_receipt_event(player: u32, input: u64, tick: u64, action: u32, catalog: u32, slot: u32, result: u32) -> TeamPublicEvent {
    TeamPublicEvent {
        event_kind: FactKind::ShopReceipt as u32,
        sanitized_payload: omoba_core::runtime::shop_receipt::ShopReceipt {
            player_id: player,
            input_id: input,
            tick,
            action_kind: action,
            catalog_id: catalog,
            slot,
            result_code: result,
        }
        .encode(),
        ..Default::default()
    }
}

fn metric(team: u32, id: u64, value: i64) -> TeamPublicEvent {
    TeamPublicEvent {
        event_kind: FactKind::Hud as u32,
        sanitized_payload: [
            team.to_le_bytes().as_slice(),
            id.to_le_bytes().as_slice(),
            value.to_le_bytes().as_slice(),
        ]
        .concat(),
        ..Default::default()
    }
}

fn actor_identity(actor: &Option<Actor>) -> (Option<u64>, Option<u64>) {
    actor
        .as_ref()
        .map(|actor| (Some(actor.render_id), Some(actor.epoch)))
        .unwrap_or((None, None))
}

fn intent_value(world: &FixtureWorld, intent: Option<renderer_input::Intent>) -> serde_json::Value {
    let Some(intent) = intent else {
        return json!({"kind": "none"});
    };
    match intent {
        renderer_input::Intent::SummonCommand(value) => {
            let id=|r:Option<&omoba_core::game_proto::SecureReplicaTarget>|r.and_then(|r|r.replica_entity_id.as_ref()).map(|r|r.value.to_string());
            let epoch=|r:Option<&omoba_core::game_proto::SecureReplicaTarget>|r.and_then(|r|r.disclosure_epoch.as_ref()).map(|r|r.value.to_string());
            json!({"kind":"SummonCommand","operation":value.operation,
                "owner_render_id":id(value.owner.as_ref()),"owner_disclosure_epoch":epoch(value.owner.as_ref()),
                "summon_render_id":id(value.summon.as_ref()),"summon_disclosure_epoch":epoch(value.summon.as_ref()),
                "has_point":value.point.is_some(),"x_raw":value.point.map(|p|p.x),"y_raw":value.point.map(|p|p.y)})
        }
        renderer_input::Intent::Communication(value) => json!({
            "kind": "Communication", "communication_kind": value.kind,
            "semantic": value.semantic, "content_id": value.content_id,
            "source_generation": value.source_generation,
            "presentation_epoch": value.presentation_epoch,
            "chat_channel": value.chat_channel,
            "text": value.text, "has_point": value.has_point,
            "point_x_raw": value.point_x_raw, "point_y_raw": value.point_y_raw,
            "target_render_id": value.target_render_id,
            "target_disclosure_epoch": value.target_disclosure_epoch,
            "hud_kind": value.hud_kind, "hud_index": value.hud_index,
            "hud_player_id": value.hud_player_id,
            "mute_sender_id": value.mute_sender_id, "muted": value.muted,
            "objective_id": value.objective_id,
        }),
        renderer_input::Intent::MoveTo(value) => json!({"kind": "MoveTo", "x_raw": value.x_raw, "y_raw": value.y_raw, "queued": value.queued}),
        renderer_input::Intent::AttackMove(value) => json!({"kind": "AttackMove", "x_raw": value.x_raw, "y_raw": value.y_raw, "queued": value.queued}),
        renderer_input::Intent::AbilityCast(value) => {
            let (catalog_id, ability_id, target_kind) = observed_ability(world, value.ability_index);
            json!({
                "kind": "AbilityCast",
                "ability_index": value.ability_index,
                "target_render_id": value.target_render_id,
                "x_raw": value.x_raw,
                "y_raw": value.y_raw,
                "catalog_id": catalog_id,
                "ability_id": ability_id,
                "target_kind": target_kind,
            })
        }
        renderer_input::Intent::ItemUse(value) => {
            let (catalog_id, item_id) = observed_item(world, value.item_slot);
            json!({
                "kind": "ItemUse",
                "item_slot": value.item_slot,
                "target_render_id": value.target_render_id,
                "x_raw": value.x_raw,
                "y_raw": value.y_raw,
                "explicit_x_raw": value.explicit_x_raw,
                "explicit_y_raw": value.explicit_y_raw,
                "catalog_id": catalog_id,
                "item_id": item_id,
            })
        }
        renderer_input::Intent::TowerAction(value) => json!({"kind": "TowerAction", "action_kind": value.action_kind, "tower_render_id": value.tower_render_id, "tower_kind_id": value.tower_kind_id, "path": value.path, "level": value.level, "x_raw": value.x_raw, "y_raw": value.y_raw}),
        renderer_input::Intent::ItemBuy(value) => json!({"kind": "ItemBuy", "catalog_id": value.catalog_id}),
        renderer_input::Intent::ItemSell(value) => json!({"kind": "ItemSell", "item_slot": value.item_slot}),
        renderer_input::Intent::AttackTarget(value) => json!({"kind": "AttackTarget", "target_render_id": value.target_render_id, "queued": value.queued}),
        renderer_input::Intent::Recall(_) => json!({"kind": "Recall"}),
        renderer_input::Intent::AbilityUpgrade(value) => {
            let (catalog_id, ability_id, _) = observed_ability(world, value.ability_index);
            json!({
                "kind": "AbilityUpgrade",
                "ability_index": value.ability_index,
                "catalog_id": catalog_id,
                "ability_id": ability_id,
            })
        }
        renderer_input::Intent::HoldPosition(value) => json!({"kind": "HoldPosition", "queued": value.queued}),
        renderer_input::Intent::Stop(_) => json!({"kind": "Stop"}),
        renderer_input::Intent::CombatPreferences(value) => json!({"kind": "CombatPreferences",
            "schema_version": value.schema_version, "acquisition": value.acquisition, "idle_auto_attack": value.idle_auto_attack}),
        renderer_input::Intent::InputScheme(value) => json!({"kind":"InputScheme","schema_version":value.schema_version,"scheme":value.scheme}),
        renderer_input::Intent::Direction(value) => json!({"kind":"Direction","schema_version":value.schema_version,
            "source_render_id":value.source_render_id.to_string(),"press_id":value.press_id.to_string(),"phase":value.phase,"x":value.x,"y":value.y}),
        renderer_input::Intent::SummonerCast(value) => {
            let (catalog_id, spell_id, target_kind) = observed_summoner(world, value.slot);
            json!({
                "kind": "SummonerCast",
                "slot": value.slot,
                "x_raw": value.x_raw,
                "y_raw": value.y_raw,
                "catalog_id": catalog_id,
                "spell_id": spell_id,
                "target_kind": target_kind,
            })
        }
        renderer_input::Intent::ShopUndo(value) => json!({
            "kind": "ShopUndo",
            "sequence": value.sequence,
        }),
        renderer_input::Intent::WardPlacement(value) => json!({
            "kind": "WardPlacement",
            "slot": value.slot,
            "x_raw": value.x_raw,
            "y_raw": value.y_raw,
        }),
    }
}

/// Observation fields only. The wire `AbilityCastIntent` still has no catalog id or target kind.
fn catalog_summoner_pair() -> Result<
    (
        &'static omoba_template_ids::SummonerSpellConst,
        &'static omoba_template_ids::SummonerSpellConst,
    ),
    &'static str,
> {
    let blink = omoba_template_ids::summoner_spell_by_name("training_point_blink_v1").ok_or("summoner_catalog")?;
    let heal = omoba_template_ids::summoner_spell_by_name("training_self_heal_v1").ok_or("summoner_catalog")?;
    if blink.catalog_id == 0 || heal.catalog_id == 0 || blink.catalog_id == heal.catalog_id {
        return Err("summoner_catalog");
    }
    let total = blink.cooldown.raw();
    if total <= 1 || total % 2 != 0 {
        return Err("summoner_cooldown");
    }
    Ok((blink, heal))
}

/// The hub flag is the explicit fixture contract plus the maintained rules hash.
/// A hash mismatch stays closed. This does not invent an accepted ACK.
fn fixture_recall_protocol_enabled(requested: bool, rules_hash: &str) -> bool {
    requested
        && omoba_core::runtime::recall_transport::negotiate_recall_protocol(
            omoba_core::runtime::recall_transport::RECALL_PROTOCOL_VERSION,
            rules_hash,
        )
        .unwrap_or(false)
}

fn item_inventory(contract: ItemContract) -> Result<([u16; 6], [u32; 6]), &'static str> {
    if contract == ItemContract::Closed {
        return Err("item_closed");
    }
    let item_ids = match contract {
        ItemContract::Closed => return Err("item_closed"),
        ItemContract::Empty => [0; 6],
        ItemContract::Replaced => {
            let mut ids = READY_ITEM_CATALOG;
            ids[1] = 6;
            ids
        }
        ItemContract::Ready | ItemContract::Cooldown | ItemContract::Far => READY_ITEM_CATALOG,
    };
    for id in item_ids {
        if id != 0 && !omoba_template_ids::MOBA_ITEM_CATALOG.iter().any(|item| item.catalog_id == id) {
            return Err("item_catalog");
        }
    }
    let mut cooldown_bits = [0u32; 6];
    if contract == ItemContract::Cooldown {
        cooldown_bits[0] = 4.0f32.to_bits();
    }
    omoba_core::runtime::native::economy_projection::CommittedEconomyState {
        gold: 0,
        item_ids,
        cooldown_bits,
        effect_bits: [0; 10],
        dirty: false,
    }
    .components()
    .map_err(|_| "item_economy")?;
    Ok((item_ids, cooldown_bits))
}

/// Observation fields only. The wire `ItemUseIntent` still has no catalog id.
fn observed_item(world: &FixtureWorld, slot: u32) -> (Option<u32>, Option<String>) {
    let Ok((ids, _)) = item_inventory(world.item_contract) else {
        return (None, None);
    };
    let Some(id) = ids.get(slot as usize).copied() else {
        return (None, None);
    };
    if id == 0 {
        return (Some(0), None);
    }
    let Some(item) = omoba_template_ids::MOBA_ITEM_CATALOG.iter().find(|item| item.catalog_id == id) else {
        return (Some(u32::from(id)), None);
    };
    (Some(u32::from(item.catalog_id)), Some(item.id.to_owned()))
}

/// Version 1 buy/sell admission. A hash mismatch stays closed. This does not enable undo or emit a receipt.
fn fixture_shop_protocol_enabled(requested: bool, rules_hash: &str) -> bool {
    requested
        && omoba_core::runtime::shop_transport::negotiate_shop_protocol(
            omoba_core::runtime::shop_transport::SHOP_PROTOCOL_VERSION,
            rules_hash,
            true,
        )
        .unwrap_or(false)
}

fn fixture_summoner_protocol_enabled(requested: bool, rules_hash: &str) -> bool {
    requested
        && omoba_core::runtime::summoner_transport::negotiate_summoner_protocol(
            omoba_core::runtime::summoner_transport::SUMMONER_PROTOCOL_VERSION,
            rules_hash,
        )
        .unwrap_or(false)
}

/// Observation fields only. The wire `SummonerCastIntent` still has no catalog id.
fn observed_summoner(world: &FixtureWorld, slot: u32) -> (Option<u32>, Option<String>, Option<&'static str>) {
    let Ok(Some(rows)) = world.summoner_slots() else {
        return (None, None, None);
    };
    let Some(row) = rows.get(slot as usize) else {
        return (None, None, None);
    };
    if row.catalog_id == 0 {
        return (Some(0), None, None);
    }
    let Some(spell) = omoba_template_ids::SUMMONER_SPELL_CATALOG
        .iter()
        .find(|spell| spell.catalog_id == row.catalog_id)
    else {
        return (Some(u32::from(row.catalog_id)), None, None);
    };
    let kind = match spell.target {
        omoba_template_ids::SummonerTargetConst::None => "none",
        omoba_template_ids::SummonerTargetConst::Point => "point",
    };
    (Some(u32::from(spell.catalog_id)), Some(spell.id.to_owned()), Some(kind))
}

fn observed_ability(world: &FixtureWorld, slot: u32) -> (Option<u16>, Option<String>, Option<&'static str>) {
    let Some(ability_id) = world.active.abilities.get(slot as usize) else {
        return (None, None, None);
    };
    let Some(id) = omoba_template_ids::ability_by_name(ability_id) else {
        return (None, Some(ability_id.clone()), None);
    };
    let kind = omoba_template_ids::active_ability_const(id).map(|definition| match definition.target_type {
        omoba_template_ids::TargetTypeC::None => "none",
        omoba_template_ids::TargetTypeC::Point => "point",
        omoba_template_ids::TargetTypeC::Unit => "unit",
    });
    (Some(id.raw()), Some(ability_id.clone()), kind)
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug)]
enum ControlIoClass {
    Absent,
    Transient,
    Interrupted,
    Permanent,
}

enum ControlFileBody {
    Bytes(Vec<u8>),
    Absent,
    Oversized { observed_len: u64 },
    Failed { operation: &'static str, error: std::io::Error },
}

fn classify_control_io(error: &std::io::Error) -> ControlIoClass {
    if error.kind() == std::io::ErrorKind::Interrupted {
        return ControlIoClass::Interrupted;
    }
    match error.raw_os_error() {
        Some(2) => ControlIoClass::Absent,
        Some(code) if cfg!(windows) && matches!(code, 5 | 32 | 33) => ControlIoClass::Transient,
        Some(_) => ControlIoClass::Permanent,
        None if error.kind() == std::io::ErrorKind::NotFound => ControlIoClass::Absent,
        None => ControlIoClass::Permanent,
    }
}

fn control_read_failure(
    operation: &str,
    path: &Path,
    error: &std::io::Error,
    transient_streak: u32,
) -> ClientRuntimeError {
    let code = match error.raw_os_error() {
        Some(code) => code.to_string(),
        None => "none".to_owned(),
    };
    ClientRuntimeError::Ipc(format!(
        "control file {operation} {} failed: {error} (raw_os_error {code}) transient_streak {transient_streak}",
        path.display()
    ))
}

fn read_control_file(path: &Path) -> ControlFileBody {
    let mut file = match open_control_file(path) {
        Ok(file) => file,
        Err(body) => return body,
    };
    let mut buf = vec![0u8; MAX_CONTROL_BYTES + 1];
    let mut filled = 0usize;
    let mut interrupts = 0u32;
    loop {
        if filled == buf.len() {
            return ControlFileBody::Oversized {
                observed_len: filled as u64,
            };
        }
        match file.read(&mut buf[filled..]) {
            Ok(0) => {
                buf.truncate(filled);
                return ControlFileBody::Bytes(buf);
            }
            Ok(n) => filled += n,
            Err(error) => match classify_control_io(&error) {
                ControlIoClass::Interrupted => {
                    interrupts = interrupts.saturating_add(1);
                    if interrupts > MAX_CONTROL_READ_INTERRUPTS_PER_POLL {
                        return ControlFileBody::Failed {
                            operation: "read",
                            error,
                        };
                    }
                }
                ControlIoClass::Absent => return ControlFileBody::Absent,
                ControlIoClass::Transient | ControlIoClass::Permanent => {
                    return ControlFileBody::Failed {
                        operation: "read",
                        error,
                    };
                }
            },
        }
    }
}

fn open_control_file(path: &Path) -> Result<File, ControlFileBody> {
    let mut interrupts = 0u32;
    loop {
        match File::open(path) {
            Ok(file) => return Ok(file),
            Err(error) => match classify_control_io(&error) {
                ControlIoClass::Interrupted => {
                    interrupts = interrupts.saturating_add(1);
                    if interrupts > MAX_CONTROL_READ_INTERRUPTS_PER_POLL {
                        return Err(ControlFileBody::Failed {
                            operation: "open",
                            error,
                        });
                    }
                }
                ControlIoClass::Absent => return Err(ControlFileBody::Absent),
                ControlIoClass::Transient | ControlIoClass::Permanent => {
                    return Err(ControlFileBody::Failed {
                        operation: "open",
                        error,
                    });
                }
            },
        }
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ClientRuntimeError> {
    let mut bytes = serde_json::to_vec(value).map_err(|error| ClientRuntimeError::Ipc(error.to_string()))?;
    bytes.push(b'\n');
    write_atomic(path, &bytes).map_err(|error| ClientRuntimeError::Ipc(error.to_string()))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("partial");
    {
        let mut file = File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    let _ = fs::remove_file(path);
    fs::rename(&tmp, path)
}

fn append_line(path: &Path, line: &str) -> Result<(), ClientRuntimeError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| ClientRuntimeError::Ipc(error.to_string()))?;
    writeln!(file, "{line}").map_err(|error| ClientRuntimeError::Ipc(error.to_string()))?;
    file.sync_all()
        .map_err(|error| ClientRuntimeError::Ipc(error.to_string()))
}

fn startup_rejected(
    hub: &PresentationHub,
    evidence_dir: &Path,
    run_id: &str,
    message: String,
) -> ClientRuntimeError {
    hub.shutdown();
    write_error(evidence_dir, run_id, &message);
    ClientRuntimeError::Ipc(format!("startup identity was not published: {message}"))
}

fn write_error(dir: &Path, run_id: &str, message: &str) {
    note_evidence_error(dir, run_id, message, None, None);
}

fn note_evidence_error(
    dir: &Path,
    run_id: &str,
    message: &str,
    cleanup_reason: Option<&str>,
    listener_released: Option<bool>,
) {
    let path = dir.join("error.json");
    let mut messages = Vec::new();
    let mut reason = cleanup_reason.map(str::to_owned);
    let mut released = listener_released;
    if let Ok(bytes) = fs::read(&path) {
        if let Ok(existing) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(prior) = existing.get("messages").and_then(serde_json::Value::as_array) {
                for item in prior {
                    if let Some(text) = item.as_str() {
                        push_unique(&mut messages, text);
                    }
                }
            } else if let Some(text) = existing.get("message").and_then(serde_json::Value::as_str) {
                push_unique(&mut messages, text);
            }
            if reason.is_none() {
                reason = existing
                    .get("cleanup_reason")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned);
            }
            if released.is_none() {
                released = existing.get("listener_released").and_then(serde_json::Value::as_bool);
            }
        }
    }
    push_unique(&mut messages, message);
    let mut body = json!({
        "schema": ERROR_SCHEMA,
        "run_id": run_id,
        "message": messages.join("\n"),
        "messages": messages,
        "authority": false,
    });
    if let Some(reason) = reason {
        body["cleanup_reason"] = json!(reason);
    }
    if let Some(released) = released {
        body["listener_released"] = json!(released);
    }
    let _ = write_json(&path, &body);
}

fn push_unique(messages: &mut Vec<String>, text: &str) {
    if !text.is_empty() && messages.iter().all(|have| have != text) {
        messages.push(text.to_owned());
    }
}

async fn probe_listener_released(address: SocketAddr) -> bool {
    let deadline = tokio::time::Instant::now() + RELEASE_PROBE;
    while tokio::time::Instant::now() < deadline {
        if let Ok(listener) = tokio::net::TcpListener::bind(address).await {
            drop(listener);
            return true;
        }
        tokio::task::yield_now().await;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation_bridge::{read_envelope, write_envelope};
    use omoba_core::game_proto::{renderer_input::Intent, MoveToIntent, RendererReady};
    use omoba_core::runtime::disclosed_attack_target_admitted;
    use prost::Message;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tokio::net::TcpStream;

    static RUNS: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn ally_cards_require_explicit_public_roster_and_retire_with_the_fixture() {
        use omoba_core::runtime::{single_lane_player_metric, SINGLE_LANE_SCORE_COUNT_METRIC_ID,
            SINGLE_LANE_SCORE_TEAM_METRIC_ID};
        let mut world = FixtureWorld::load().unwrap();
        assert!(!world.team_cards_disclosed);
        world.apply_stage("owner_support_team_cards").unwrap();
        assert!(!world.communication_enabled);
        let snapshot = world.to_snapshot().unwrap();
        assert!(snapshot.public_events.contains(&metric(0, SINGLE_LANE_SCORE_COUNT_METRIC_ID, 3)));
        assert!(snapshot.public_events.contains(&metric(0, single_lane_player_metric(SINGLE_LANE_SCORE_TEAM_METRIC_ID, 2), 1)));
        assert!(snapshot.public_events.contains(&metric(0, single_lane_player_metric(SINGLE_LANE_SCORE_TEAM_METRIC_ID, 3), 2)));
        assert!(world.target.as_ref().unwrap().alive);
        world.apply_stage("owner_support_team_cards_dead").unwrap();
        assert!(world.team_cards_disclosed);
        assert!(!world.target.as_ref().unwrap().alive);
        world.apply_stage("owner_support_ready").unwrap();
        assert!(!world.team_cards_disclosed);
        assert!(!world.to_snapshot().unwrap().public_events.contains(&metric(0, SINGLE_LANE_SCORE_COUNT_METRIC_ID, 3)));
        world.apply_stage("owner_support_team_cards").unwrap();
        world.apply_stage("baseline").unwrap();
        assert!(!world.team_cards_disclosed);
        assert!(!world.to_snapshot().unwrap().public_events.contains(&metric(0, SINGLE_LANE_SCORE_COUNT_METRIC_ID, 3)));
    }

    #[test]
    fn input_scheme_fixture_is_explicit_and_never_confirms_an_observed_request() {
        use omoba_core::runtime::input_scheme_transport::decode;
        let mut world = FixtureWorld::load().unwrap();
        assert!(world.to_snapshot().unwrap().combat_preferences.is_none());
        world.apply_stage("combat_idle_off_projection").unwrap();
        for (stage, scheme, locked) in [("input_point", 0, false), ("input_keyboard", 1, false), ("input_locked", 1, true)] {
            world.apply_stage(stage).unwrap();
            let snapshot = world.to_snapshot().unwrap();
            let projection = snapshot.combat_preferences.unwrap();
            let state = projection.rows[0].input_scheme.as_ref().unwrap();
            assert_eq!((projection.rows[0].acquisition, projection.rows[0].idle_auto_attack), (1, false));
            assert_eq!(state.scheme, scheme);
            assert_eq!(state.hero_content_id, "training_luminary");
            assert_eq!((state.last_press_id, state.active_press_id), (0, 0));
            if scheme == 1 {
                let step = state.direction_step.as_ref().unwrap();
                assert_eq!(step.tick, world.replica_tick.saturating_add(1));
                assert_eq!((step.scheme, step.press_id, step.direction_x_raw, step.direction_y_raw), (1, 0, 0, 0));
                assert!(!step.navigation_reset && !step.attack_reset);
            } else { assert!(state.direction_step.is_none()); }
            assert_eq!(decode(state).unwrap().locked(), locked);
            world.apply_stage("source_death").unwrap();
            assert_eq!(world.to_snapshot().unwrap().combat_preferences.unwrap().rows[0].input_scheme.as_ref().unwrap().hero_content_id, "training_luminary");
        }
        world.apply_stage("owner_point_ready").unwrap();
        assert_eq!(world.to_snapshot().unwrap().combat_preferences.unwrap().rows[0].input_scheme.as_ref().unwrap().hero_content_id, "training_vanguard");
        world.apply_stage("owner_upgrade_point").unwrap();
        assert_eq!(world.to_snapshot().unwrap().combat_preferences.unwrap().rows[0].input_scheme.as_ref().unwrap().hero_content_id, "training_luminary");
        world.apply_stage("baseline").unwrap();
        assert!(world.to_snapshot().unwrap().combat_preferences.is_none());
    }

    #[test]
    fn combat_wire_fixture_projects_explicit_settings_without_inventing_input_results() {
        let mut world = FixtureWorld::load().unwrap();
        world.apply_stage("combat_defaults").unwrap();
        for (stage, acquisition, idle) in [("combat_defaults", 0, true),
            ("combat_cursor_projection", 1, true), ("combat_idle_off_projection", 1, false)] {
            world.apply_stage(stage).unwrap();
            let snapshot = world.to_snapshot().unwrap();
            let projection = snapshot.combat_preferences.unwrap();
            assert_eq!(projection.view_epoch, world.epoch_floor);
            assert_eq!(projection.rows.len(), 1);
            assert_eq!(projection.rows[0].player_id, OWNER_PLAYER_ID);
            assert_eq!((projection.rows[0].acquisition, projection.rows[0].idle_auto_attack), (acquisition, idle));
        }
        world.apply_stage("source_death").unwrap();
        assert!(!world.to_snapshot().unwrap().combat_preferences.unwrap().rows[0].idle_auto_attack);
        world.apply_stage("disclosure_epoch").unwrap();
        assert_eq!(world.to_snapshot().unwrap().combat_preferences.unwrap().view_epoch, world.epoch_floor);
        world.apply_stage("baseline").unwrap();
        assert!(world.to_snapshot().unwrap().combat_preferences.is_none());
    }

    #[test]
    fn disconnected_fixture_publishes_explicit_moba_mode_without_defaulting_absence() {
        use omoba_core::runtime::authoritative_mode::MOBA;
        let mut world = FixtureWorld::load().expect("fixture catalog");
        let snapshot = world.to_snapshot().unwrap();
        let mode = snapshot.authoritative_mode.as_ref().unwrap();
        assert_eq!((mode.schema_version, mode.mode, mode.view_epoch), (1, MOBA, world.epoch_floor));
        world.apply_stage("source_death").unwrap();
        assert_eq!(world.to_snapshot().unwrap().authoritative_mode.as_ref().unwrap().mode, MOBA);
        world.apply_stage("source_removal").unwrap();
        let removed = world.to_snapshot().unwrap();
        assert_eq!(removed.authoritative_mode.as_ref().unwrap().mode, MOBA);
        assert!(removed.entities.iter().all(|entity| entity.replica_id != 0));
        world.apply_stage("disclosure_epoch").unwrap();
        let bumped = world.to_snapshot().unwrap();
        assert_eq!(bumped.authoritative_mode.as_ref().unwrap().view_epoch, world.epoch_floor);
        assert!(world.epoch_floor > 1);
        let absent = FilteredRenderSnapshot {
            team_id: 1, replica_tick: 1, entities: vec![], public_events: vec![],
            external_effects: vec![], memory_directives: vec![],
            remembered_presentations: Default::default(), hero_attributes: Default::default(), disclosed_summoners: Default::default(),
            authoritative_mode: None, combat_preferences: None,
        };
        let envelope = crate::presentation_bridge::snapshot_envelope(1, 1, world.epoch_floor, absent, 0);
        let Some(omoba_core::game_proto::renderer_ipc_envelope::Payload::Snapshot(snapshot)) = envelope.payload else {
            panic!("snapshot");
        };
        assert!(snapshot.authoritative_mode.is_none());
    }

    #[test]
    fn fixture_mode_stages_keep_identity_and_typed_mode() {
        use omoba_core::runtime::authoritative_mode::{MOBA, TOWER_DEFENSE};
        use omoba_core::runtime::native::comp::GameMode;
        let mut world = FixtureWorld::load().expect("fixture catalog");
        world.apply_stage("mode_tower_defense").unwrap();
        assert_eq!(world.to_snapshot().unwrap().authoritative_mode.as_ref().unwrap().mode, TOWER_DEFENSE);
        world.apply_stage("mode_td_defeat").unwrap();
        assert!(world.td_defeat);
        assert_eq!(world.to_snapshot().unwrap().authoritative_mode.as_ref().unwrap().mode, TOWER_DEFENSE);
        let before = world.epoch_floor;
        world.apply_stage("mode_td_no_hero").unwrap();
        let omitted = world.to_snapshot().unwrap();
        assert_eq!(omitted.authoritative_mode.as_ref().unwrap().mode, TOWER_DEFENSE);
        assert!(world.source.as_ref().is_some_and(|actor| actor.render_id == SOURCE_RENDER_ID));
        assert!(omitted.entities.iter().all(|entity| entity.replica_id != SOURCE_RENDER_ID));
        world.apply_stage("mode_moba_dead").unwrap();
        let dead = world.to_snapshot().unwrap();
        assert_eq!(dead.authoritative_mode.as_ref().unwrap().mode, MOBA);
        assert!(dead.entities.iter().any(|entity| entity.replica_id == SOURCE_RENDER_ID));
        assert!(world.source.as_ref().is_some_and(|actor| !actor.alive));
        world.apply_stage("mode_unavailable").unwrap();
        assert!(world.to_snapshot().unwrap().authoritative_mode.is_none());
        world.apply_stage("baseline").unwrap();
        assert_eq!(world.presentation_mode, Some(GameMode::Moba));
        assert!(!world.td_defeat && !world.suppress_owner_hero);
        assert!(world.source.as_ref().is_some_and(|actor| actor.alive));
        world.apply_stage("mode_td_epoch").unwrap();
        assert!(world.epoch_floor > before);
        let bumped = world.to_snapshot().unwrap();
        assert_eq!(bumped.authoritative_mode.as_ref().unwrap().mode, TOWER_DEFENSE);
        assert_eq!(bumped.authoritative_mode.as_ref().unwrap().view_epoch, world.epoch_floor);
    }

    fn options() -> FixtureOptions {
        let n = RUNS.fetch_add(1, Ordering::Relaxed);
        let run_id = format!("fx{}n{n}", std::process::id());
        let evidence_dir = std::env::temp_dir().join(&run_id);
        let _ = fs::remove_dir_all(&evidence_dir);
        FixtureOptions {
            shutdown_file: evidence_dir.join("shutdown.signal"),
            evidence_dir,
            run_id,
            lifetime_seconds: 30,
        }
    }

    fn snapshot_payload(envelope: &RendererIpcEnvelope) -> TeamPresentationSnapshot {
        match &envelope.payload {
            Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) => snapshot.clone(),
            _ => panic!("expected snapshot"),
        }
    }

    fn rows(snapshot: &TeamPresentationSnapshot) -> Vec<(u64, BTreeMap<u32, Vec<u8>>)> {
        snapshot
            .entities
            .iter()
            .map(|entity| {
                (
                    entity.render_id,
                    entity
                        .components
                        .iter()
                        .map(|component| (component.schema_id, component.safe_payload.clone()))
                        .collect(),
                )
            })
            .collect()
    }

    fn admitted(snapshot: &TeamPresentationSnapshot, target: u64) -> bool {
        let owned = rows(snapshot);
        let view: Vec<_> = owned.iter().map(|(id, components)| (*id, components)).collect();
        disclosed_attack_target_admitted(&view, OWNER_PLAYER_ID, target)
    }

    fn has_component(snapshot: &TeamPresentationSnapshot, render_id: u64, schema: u32) -> bool {
        snapshot.entities.iter().any(|entity| {
            entity.render_id == render_id && entity.components.iter().any(|component| component.schema_id == schema)
        })
    }

    fn component<'a>(snapshot: &'a TeamPresentationSnapshot, render_id: u64, schema: u32) -> &'a [u8] {
        snapshot
            .entities
            .iter()
            .find(|entity| entity.render_id == render_id)
            .unwrap()
            .components
            .iter()
            .find(|component| component.schema_id == schema)
            .unwrap()
            .safe_payload
            .as_slice()
    }

    async fn connect(fixture: &RendererFixture) -> TcpStream {
        let mut client = TcpStream::connect(fixture.local_addr()).await.unwrap();
        write_envelope(
            &mut client,
            &RendererIpcEnvelope {
                magic: PRESENTATION_MAGIC,
                protocol_version: PRESENTATION_PROTOCOL_VERSION,
                sequence: 0,
                payload: Some(renderer_ipc_envelope::Payload::RendererReady(RendererReady {
                    player_id: OWNER_PLAYER_ID,
                    team_id: OWNER_TEAM_ID,
                    latest_snapshot_sequence: 0,
                })),
            },
        )
        .await
        .unwrap();
        client
    }

    #[test]
    fn lifetime_and_run_id_gates_are_finite() {
        assert_eq!(parse_lifetime(None).unwrap(), 900);
        assert_eq!(parse_lifetime(Some("1")).unwrap(), 1);
        assert_eq!(parse_lifetime(Some("1800")).unwrap(), 1800);
        assert!(parse_lifetime(Some("0")).is_err());
        assert!(parse_lifetime(Some("1801")).is_err());
        assert!(parse_lifetime(Some("30s")).is_err());
        assert!(valid_run_id("fx1"));
        assert!(!valid_run_id(""));
        assert!(!valid_run_id("-bad"));
        let err = parse_args(["--run-id".into(), "ab".into()]).unwrap_err();
        assert!(err.contains("evidence"));
    }

    #[test]
    fn communication_fixture_discloses_public_roster_only_in_negotiated_stages() {
        let mut world = FixtureWorld::load().unwrap();
        assert!(crate::scoreboard::project(&world.to_snapshot().unwrap()).is_none());
        world.apply_stage("communication_ready").unwrap();
        let board = crate::scoreboard::project(&world.to_snapshot().unwrap()).unwrap();
        assert_eq!(board.rows.iter().map(|row| (row.player_id, row.team_id)).collect::<Vec<_>>(), vec![(1,1),(2,2)]);
        world.apply_stage("communication_received").unwrap();
        assert_eq!(crate::scoreboard::project(&world.to_snapshot().unwrap()).unwrap(), board);
        world.apply_stage("communication_hud").unwrap();
        assert_eq!(crate::scoreboard::project(&world.to_snapshot().unwrap()).unwrap(), board);
        assert!(world.owner_economy_event().is_some(), "HUD status fixture publishes actual owner item state");
        world.apply_stage("communication_retire").unwrap();
        assert!(crate::scoreboard::project(&world.to_snapshot().unwrap()).is_none());
        world.apply_stage("communication_ready").unwrap();
        world.suppress_owner_hero = true;
        assert_eq!(crate::scoreboard::project(&world.to_snapshot().unwrap()).unwrap(), board,
            "public roster is independent of received messages and live owner actors");
    }

    #[tokio::test]
    async fn received_chat_fixture_uses_real_tcp_and_no_input_receipt_or_echo() {
        use omoba_core::runtime::communication_transport as wire;
        tokio::time::timeout(Duration::from_secs(5), async {
            let options = options();
            let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
            let mut client = connect(&fixture).await;
            assert!(matches!(read_envelope(&mut client).await.unwrap().payload, Some(renderer_ipc_envelope::Payload::RuntimeReady(_))));
            assert!(matches!(read_envelope(&mut client).await.unwrap().payload, Some(renderer_ipc_envelope::Payload::Snapshot(_))));
            for sequence in 1..=2 {
                let control = serde_json::to_vec(&json!({"schema": CONTROL_SCHEMA, "run_id": options.run_id,
                    "sequence": sequence, "stage": "communication_received"})).unwrap();
                assert!(fixture.apply_control_bytes(&control).accepted);
                fixture.publish_pending_communication().await.unwrap();
                loop {
                    let message = read_envelope(&mut client).await.unwrap();
                    match message.payload {
                        Some(renderer_ipc_envelope::Payload::Snapshot(_)) => continue,
                        Some(renderer_ipc_envelope::Payload::TeamCommunication(batch)) => {
                            assert!(batch.negotiated && batch.input_result.is_none());
                            assert!(batch.receipt.as_ref().is_none_or(|receipt| receipt.request_id == 0));
                            assert_eq!(batch.events.len(), 2, "repeated fixture stage must deduplicate inbound events");
                            assert_eq!(batch.events[0].audience, wire::AUDIENCE_TEAM);
                            assert_eq!(batch.events[0].text, "隊伍公告：一起守住中路");
                            assert_eq!(batch.events[1].audience, wire::AUDIENCE_ALL);
                            assert_eq!(batch.events[1].text, "全體公告：準備出發");
                            assert!(batch.events.iter().all(|event| event.receipt_request_id == 0 && !event.has_point && !event.has_target));
                            break;
                        }
                        other => panic!("received fixture fabricated an unrelated response: {other:?}"),
                    }
                }
            }
            assert_eq!(fixture.observations, 0);
            drop(client);
            let (cleanup, persisted) = fixture.finish("received_chat_test").await;
            assert!(persisted && cleanup.listener_released);
            let _ = fs::remove_dir_all(options.evidence_dir);
        }).await.unwrap();
    }

    #[tokio::test]
    async fn received_signals_fixture_uses_real_tcp_and_public_clock_expiry_without_echo() {
        use omoba_core::runtime::communication_transport as wire;
        tokio::time::timeout(Duration::from_secs(5), async {
            let options = options(); let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
            let mut client = connect(&fixture).await;
            assert!(matches!(read_envelope(&mut client).await.unwrap().payload, Some(renderer_ipc_envelope::Payload::RuntimeReady(_))));
            assert!(matches!(read_envelope(&mut client).await.unwrap().payload, Some(renderer_ipc_envelope::Payload::Snapshot(_))));
            for (sequence, stage, count, clock) in [(1,"communication_signals",3,1024),
                (2,"communication_signals",3,1024),(3,"communication_signals_expired",4,32768)] {
                let control=serde_json::to_vec(&json!({"schema":CONTROL_SCHEMA,"run_id":options.run_id,"sequence":sequence,"stage":stage})).unwrap();
                assert!(fixture.apply_control_bytes(&control).accepted);
                fixture.publish_pending_communication().await.unwrap();
                loop {
                    match read_envelope(&mut client).await.unwrap().payload {
                        Some(renderer_ipc_envelope::Payload::Snapshot(_)) => continue,
                        Some(renderer_ipc_envelope::Payload::TeamCommunication(batch)) => {
                            assert!(batch.negotiated && batch.input_result.is_none());
                            assert!(batch.receipt.as_ref().is_none_or(|r|r.request_id==0));
                            assert_eq!(batch.events.len(),count); assert_eq!(batch.elapsed_raw,clock);
                            let ping=&batch.events[0]; assert_eq!(ping.kind,wire::KIND_PING);
                            assert!(ping.has_point && !ping.has_target); assert_eq!((ping.point_x_raw,ping.point_y_raw),(0,6*1024));
                            assert_eq!(batch.events[1].objective_id,"middle_gate");
                            assert_eq!(batch.events[2].content_id,101);
                            assert!(batch.events.iter().all(|e|e.receipt_request_id==0));
                            assert_eq!(batch.events.iter().filter(|e|e.expires_at_raw>batch.elapsed_raw).count(),if clock==1024 {3}else{0});
                            break;
                        }
                        other=>panic!("signals fixture fabricated unrelated response: {other:?}"),
                    }
                }
            }
            assert_eq!(fixture.observations,0);drop(client);
            let (cleanup,persisted)=fixture.finish("received_signals_test").await;
            assert!(persisted && cleanup.listener_released);let _=fs::remove_dir_all(options.evidence_dir);
        }).await.unwrap();
    }

    #[tokio::test]
    async fn communication_fixture_routes_real_tcp_intent_and_retires_without_authority_receipt() {
        use omoba_core::{game_proto::CommunicationIntent, runtime::communication_transport as wire};
        tokio::time::timeout(Duration::from_secs(5), async {
            let options = options();
            let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
            let mut client = connect(&fixture).await;
            assert!(matches!(read_envelope(&mut client).await.unwrap().payload, Some(renderer_ipc_envelope::Payload::RuntimeReady(_))));
            assert!(matches!(read_envelope(&mut client).await.unwrap().payload, Some(renderer_ipc_envelope::Payload::Snapshot(_))));
            let control = |sequence, stage| serde_json::to_vec(&json!({
                "schema": CONTROL_SCHEMA, "run_id": options.run_id, "sequence": sequence, "stage": stage,
            })).unwrap();
            let ack = fixture.apply_control_bytes(&control(1, "communication_ready"));
            assert!(ack.accepted);
            fixture.publish_pending_communication().await.unwrap();
            let batch = loop {
                let envelope = read_envelope(&mut client).await.unwrap();
                match envelope.payload {
                    Some(renderer_ipc_envelope::Payload::TeamCommunication(batch)) => break batch,
                    Some(renderer_ipc_envelope::Payload::Snapshot(_)) => {},
                    other => panic!("unexpected fixture envelope: {other:?}"),
                }
            };
            assert!(batch.negotiated);
            assert!(batch.source_generation > 0 && batch.presentation_epoch > 0);
            assert!(batch.events.is_empty() && batch.input_result.is_none());
            assert_eq!(batch.receipt.as_ref().unwrap().request_id, 0, "only explicit synthetic hello, never an accepted input");
            write_envelope(&mut client, &RendererIpcEnvelope {
                magic: PRESENTATION_MAGIC, protocol_version: PRESENTATION_PROTOCOL_VERSION, sequence: 0,
                payload: Some(renderer_ipc_envelope::Payload::RendererInput(RendererInput {
                    request_id: 88, player_id: OWNER_PLAYER_ID, disclosure_epoch: batch.view_epoch,
                    intent: Some(Intent::Communication(CommunicationIntent {
                        source_generation: batch.source_generation, presentation_epoch: batch.presentation_epoch,
                        kind: wire::KIND_CHAT, chat_channel: 1, text: "集合".into(), ..Default::default()
                    })),
                })),
            }).await.unwrap();
            let observed = fixture.hub.recv_input().await.unwrap();
            let Some(Intent::Communication(intent)) = observed.intent.as_ref() else { panic!("typed communication expected") };
            assert!(intent.renderer_connection_id > 0, "real renderer session owns the stamp");
            let record = fixture.record_input(observed).unwrap();
            assert_eq!(record["intent"]["text"], "集合");
            assert_eq!(record["intent"]["chat_channel"], 1);
            assert_eq!(record["intent"]["has_point"], false);
            assert_eq!(record["published_sequence"], ack.published_sequence);
            assert!(record["ipc_sequence"].as_u64().unwrap() > ack.published_sequence);
            assert_eq!(record["authority_ack"], false);
            // Inputs are observed, never converted into synthetic accepted receipts.
            loop {
                match tokio::time::timeout(Duration::from_millis(50), read_envelope(&mut client)).await {
                    Err(_) => break,
                    Ok(Ok(RendererIpcEnvelope { payload: Some(renderer_ipc_envelope::Payload::Snapshot(_)), .. })) => {},
                    other => panic!("fixture must not acknowledge observed input: {other:?}"),
                }
            }
            assert!(fixture.apply_control_bytes(&control(2, "communication_retire")).accepted);
            fixture.publish_pending_communication().await.unwrap();
            loop {
                let envelope = read_envelope(&mut client).await.unwrap();
                match envelope.payload {
                    Some(renderer_ipc_envelope::Payload::TeamCommunication(batch)) => {
                        assert!(wire::communication_presentation_cleared(&batch));
                        assert!(batch.events.is_empty() && batch.input_result.is_none());
                        break;
                    },
                    Some(renderer_ipc_envelope::Payload::Snapshot(_)) => {},
                    other => panic!("unexpected retirement envelope: {other:?}"),
                }
            }
            let (cleanup, persisted) = fixture.finish("test").await;
            assert!(persisted && cleanup.listener_released);
            let _ = fs::remove_dir_all(options.evidence_dir);
        }).await.unwrap();
    }

    #[tokio::test]
    async fn compiled_rows_use_current_codecs_content_and_legal_source() {
        let options = options();
        let fixture = RendererFixture::start(options.clone()).await.unwrap();
        let startup: serde_json::Value =
            serde_json::from_slice(&fs::read(options.evidence_dir.join("startup.json")).unwrap()).unwrap();
        assert_eq!(startup["address"], fixture.local_addr().to_string());
        assert_eq!(startup["content_hash"], omoba_template_ids::CONTENT_CATALOG_DATA_HASH);
        assert_eq!(startup["moba_map_catalog_hash"], omoba_template_ids::MOBA_MAP_CATALOG_HASH);
        assert_eq!(startup["hero_content_id"], FIXTURE_HERO_ID);
        assert_eq!(startup["authority"], false);
        let snapshot = snapshot_payload(fixture.last_snapshot());
        assert!(snapshot.entities.iter().all(|entity| entity.render_id != 0));
        assert!(!admitted(&snapshot, 0));
        assert!(admitted(&snapshot, TARGET_RENDER_ID));
        let render = decode_demo_render_state(component(&snapshot, SOURCE_RENDER_ID, DEMO_RENDER_COMPONENT_SCHEMA_ID)).unwrap();
        assert_eq!(render.kind, 1);
        assert_eq!(render.owner_player_id, OWNER_PLAYER_ID);
        assert_eq!(component(&snapshot, SOURCE_RENDER_ID, DEMO_RENDER_COMPONENT_SCHEMA_ID).len(), 25);
        assert_eq!(component(&snapshot, SOURCE_RENDER_ID, DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID).len(), 40);
        assert_eq!(
            component(&snapshot, TARGET_RENDER_ID, DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID).len(),
            8
        );
        let combat = decode_disclosed_combat_relation(component(
            &snapshot,
            TARGET_RENDER_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
        ))
        .unwrap();
        assert!(combat.alive && combat.attackable);
        assert_eq!(combat.faction_kind, 1);
        assert!(decode_disclosed_health(component(
            &snapshot,
            SOURCE_RENDER_ID,
            DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID
        ))
        .unwrap()
        .0 > 0);
        let hero: omoba_core::runtime::Hero = serde_json::from_slice(component(
            &snapshot,
            SOURCE_RENDER_ID,
            DISCLOSED_HERO_COMPONENT_SCHEMA_ID,
        ))
        .unwrap();
        assert_eq!(hero.id, FIXTURE_HERO_ID);
        let hud = snapshot.moba_hud.unwrap().hero.unwrap();
        assert_eq!(hud.content_id, FIXTURE_HERO_ID);
        assert_eq!(hud.render_id, SOURCE_RENDER_ID);
        assert_eq!(
            fixture.ready_envelope().encode_to_vec().is_empty(),
            false
        );
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[tokio::test]
    async fn loopback_ready_snapshot_and_public_input_use_production_hub() {
        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        assert!(fixture.local_addr().ip().is_loopback());
        assert_ne!(fixture.local_addr().port(), 0);
        let mut client = connect(&fixture).await;
        let ready = read_envelope(&mut client).await.unwrap();
        let snapshot = read_envelope(&mut client).await.unwrap();
        assert!(matches!(ready.payload, Some(renderer_ipc_envelope::Payload::RuntimeReady(_))));
        assert!(matches!(snapshot.payload, Some(renderer_ipc_envelope::Payload::Snapshot(_))));
        assert!(!matches!(ready.payload, Some(renderer_ipc_envelope::Payload::CriticalInputResult(_))));
        assert!(!matches!(snapshot.payload, Some(renderer_ipc_envelope::Payload::CriticalInputResult(_))));
        assert_eq!(ready, fixture.ready_envelope().clone());
        write_envelope(
            &mut client,
            &RendererIpcEnvelope {
                magic: PRESENTATION_MAGIC,
                protocol_version: PRESENTATION_PROTOCOL_VERSION,
                sequence: 3,
                payload: Some(renderer_ipc_envelope::Payload::RendererInput(RendererInput {
                    request_id: 44,
                    player_id: OWNER_PLAYER_ID,
                    disclosure_epoch: 1,
                    intent: Some(Intent::MoveTo(MoveToIntent { x_raw: 1024, y_raw: 2048, queued: false })),
                })),
            },
        )
        .await
        .unwrap();
        let observed = tokio::time::timeout(Duration::from_secs(2), fixture.hub.recv_input())
            .await
            .unwrap()
            .unwrap();
        let record = fixture.record_input(observed).unwrap();
        assert_eq!(record["local_request_id"], 44);
        assert_eq!(record["intent"]["kind"], "MoveTo");
        assert_eq!(record["intent"]["x_raw"], 1024);
        assert_eq!(record["authority_ack"], false);
        assert_eq!(record["content_hash"], omoba_template_ids::CONTENT_CATALOG_DATA_HASH);
        let third = tokio::time::timeout(Duration::from_millis(200), read_envelope(&mut client)).await;
        assert!(third.is_err() || third.unwrap().is_err());
        let address = fixture.local_addr();
        let (cleanup, persisted) = fixture.finish("test").await;
        assert!(persisted);
        assert!(cleanup.listener_released);
        assert!(tokio::net::TcpListener::bind(address).await.is_ok());
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[tokio::test]
    async fn reconnect_retains_ready_metadata_without_authority_ack() {
        let options = options();
        let fixture = RendererFixture::start(options.clone()).await.unwrap();
        let mut first = connect(&fixture).await;
        let ready = read_envelope(&mut first).await.unwrap();
        let snapshot = read_envelope(&mut first).await.unwrap();
        write_envelope(
            &mut first,
            &RendererIpcEnvelope {
                magic: PRESENTATION_MAGIC,
                protocol_version: PRESENTATION_PROTOCOL_VERSION,
                sequence: 0,
                payload: Some(renderer_ipc_envelope::Payload::RendererShutdown(Default::default())),
            },
        )
        .await
        .unwrap();
        drop(first);
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        let mut restored = None;
        while tokio::time::Instant::now() < deadline {
            if let Ok(mut client) = TcpStream::connect(fixture.local_addr()).await {
                let wrote = write_envelope(
                    &mut client,
                    &RendererIpcEnvelope {
                        magic: PRESENTATION_MAGIC,
                        protocol_version: PRESENTATION_PROTOCOL_VERSION,
                        sequence: 0,
                        payload: Some(renderer_ipc_envelope::Payload::RendererReady(RendererReady {
                            player_id: OWNER_PLAYER_ID,
                            team_id: OWNER_TEAM_ID,
                            latest_snapshot_sequence: 0,
                        })),
                    },
                )
                .await;
                if wrote.is_ok() {
                    if let Ok(Ok(again)) = tokio::time::timeout(Duration::from_millis(500), read_envelope(&mut client)).await {
                        restored = Some((again, client));
                        break;
                    }
                }
            }
            tokio::task::yield_now().await;
        }
        let (again, mut client) = restored.expect("reconnect");
        let again_snapshot = read_envelope(&mut client).await.unwrap();
        assert_eq!(again, ready);
        assert_eq!(again_snapshot, snapshot);
        match &again.payload {
            Some(renderer_ipc_envelope::Payload::RuntimeReady(metadata)) => {
                assert_eq!(metadata.content_hash, omoba_template_ids::CONTENT_CATALOG_DATA_HASH);
                assert_eq!(metadata.moba_map_catalog_hash, omoba_template_ids::MOBA_MAP_CATALOG_HASH);
                assert_eq!(metadata.moba_map_id, FIXTURE_MAP_ID);
            }
            _ => panic!("retained ready"),
        }
        assert!(!matches!(again.payload, Some(renderer_ipc_envelope::Payload::CriticalInputResult(_))));
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[tokio::test]
    async fn control_rejects_run_sequence_and_applies_source_target_stages() {
        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let run_id = fixture.run_id.clone();
        let reject = |fixture: &mut RendererFixture, value: serde_json::Value| {
            fixture.apply_control_bytes(&serde_json::to_vec(&value).unwrap())
        };
        assert_eq!(reject(&mut fixture, json!({"schema": CONTROL_SCHEMA, "run_id": run_id, "sequence": 2, "stage": "baseline"})).reason, "sequence_not_next");
        assert_eq!(reject(&mut fixture, json!({"schema": CONTROL_SCHEMA, "run_id": run_id, "sequence": 0, "stage": "baseline"})).reason, "stale_sequence");
        assert_eq!(reject(&mut fixture, json!({"schema": CONTROL_SCHEMA, "run_id": "other", "sequence": 1, "stage": "baseline"})).reason, "run_mismatch");
        assert_eq!(reject(&mut fixture, json!({"schema": CONTROL_SCHEMA, "run_id": run_id, "sequence": 1, "stage": "apply_ack"})).reason, "unknown_stage");
        assert_eq!(reject(&mut fixture, json!({"schema": "nope", "run_id": run_id, "sequence": 1, "stage": "baseline"})).reason, "unknown_schema");
        assert_eq!(reject(&mut fixture, json!({"schema": CONTROL_SCHEMA, "run_id": run_id, "sequence": 1, "stage": "baseline", "extra": 1})).reason, "malformed");
        assert_eq!(fixture.apply_control_bytes(&vec![0; MAX_CONTROL_BYTES + 1]).reason, "oversized");
        assert_eq!(fixture.world.control_sequence, 0);

        let mut sequence = 1u64;
        let apply = |fixture: &mut RendererFixture, stage: &str, sequence: &mut u64| {
            let run_id = fixture.run_id.clone();
            let ack = fixture.apply_control_bytes(&serde_json::to_vec(&json!({
                "schema": CONTROL_SCHEMA,
                "run_id": run_id,
                "sequence": *sequence,
                "stage": stage,
            })).unwrap());
            assert!(ack.accepted, "{stage} {}", ack.reason);
            assert!(!ack.authority_ack);
            *sequence += 1;
            snapshot_payload(fixture.last_snapshot())
        };
        let baseline = apply(&mut fixture, "baseline", &mut sequence);
        assert!(admitted(&baseline, TARGET_RENDER_ID));
        assert_eq!(baseline.moba_hud.as_ref().unwrap().hero.as_ref().unwrap().render_id, SOURCE_RENDER_ID);
        let replaced_target = apply(&mut fixture, "replace_target", &mut sequence);
        assert!(admitted(&replaced_target, REPLACEMENT_TARGET_RENDER_ID));
        assert!(replaced_target.entities.iter().all(|entity| entity.render_id != TARGET_RENDER_ID));
        let replaced_source = apply(&mut fixture, "source_replacement", &mut sequence);
        assert_eq!(replaced_source.moba_hud.as_ref().unwrap().hero.as_ref().unwrap().render_id, REPLACEMENT_SOURCE_RENDER_ID);
        assert!(admitted(&replaced_source, REPLACEMENT_TARGET_RENDER_ID));
        let removed_target = apply(&mut fixture, "remove_target", &mut sequence);
        assert!(removed_target.entities.iter().all(|entity| entity.render_id != REPLACEMENT_TARGET_RENDER_ID));
        assert!(!admitted(&removed_target, REPLACEMENT_TARGET_RENDER_ID));
        let restored_target = apply(&mut fixture, "replace_target", &mut sequence);
        assert!(admitted(&restored_target, TARGET_RENDER_ID));
        assert!(restored_target.entities.iter().all(|entity| entity.render_id != REPLACEMENT_TARGET_RENDER_ID));
        let disclosed = apply(&mut fixture, "disclosure_epoch", &mut sequence);
        assert!(disclosed.entities.iter().all(|entity| entity.disclosure_epoch > 1));
        assert!(admitted(&disclosed, TARGET_RENDER_ID));
        let removed_source = apply(&mut fixture, "source_removal", &mut sequence);
        assert!(removed_source.moba_hud.as_ref().unwrap().hero.is_none());
        assert!(!admitted(&removed_source, TARGET_RENDER_ID));
        let restored_source = apply(&mut fixture, "source_replacement", &mut sequence);
        assert_eq!(restored_source.moba_hud.as_ref().unwrap().hero.as_ref().unwrap().render_id, SOURCE_RENDER_ID);
        assert!(admitted(&restored_source, TARGET_RENDER_ID));
        let cooled = apply(&mut fixture, "hud_cooldown", &mut sequence);
        let ability = &cooled.moba_hud.as_ref().unwrap().hero.as_ref().unwrap().abilities[0];
        assert!(ability.cooldown_raw > 0);
        assert!(ability.cooldown_raw <= ability.total_cooldown_raw);
        let source_dead = apply(&mut fixture, "source_death", &mut sequence);
        assert_source_dead(&source_dead);
        let restored = apply(&mut fixture, "baseline", &mut sequence);
        assert_source_living(&restored);
        let dead = apply(&mut fixture, "death", &mut sequence);
        assert!(!admitted(&dead, TARGET_RENDER_ID));
        assert!(!decode_disclosed_combat_relation(component(&dead, TARGET_RENDER_ID, DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID)).unwrap().alive);
        assert_eq!(decode_disclosed_health(component(&dead, TARGET_RENDER_ID, DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID)).unwrap().0, 0);
        let run_id = fixture.run_id.clone();
        let stale = fixture.apply_control_bytes(&serde_json::to_vec(&json!({
            "schema": CONTROL_SCHEMA, "run_id": run_id, "sequence": 1, "stage": "baseline"
        })).unwrap());
        assert_eq!(stale.reason, "stale_sequence");
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[tokio::test]
    async fn source_death_blocks_target_admission_until_baseline_restores_living_source() {
        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let run_id = fixture.run_id.clone();
        let mut sequence = 1u64;
        let apply = |fixture: &mut RendererFixture, stage: &str, sequence: &mut u64| {
            let ack = fixture.apply_control_bytes(&serde_json::to_vec(&json!({
                "schema": CONTROL_SCHEMA,
                "run_id": run_id,
                "sequence": *sequence,
                "stage": stage,
            })).unwrap());
            assert!(ack.accepted, "{stage} {}", ack.reason);
            assert!(!ack.authority_ack);
            *sequence += 1;
            snapshot_payload(fixture.last_snapshot())
        };
        let dead = apply(&mut fixture, "source_death", &mut sequence);
        assert_source_dead(&dead);
        let living = apply(&mut fixture, "baseline", &mut sequence);
        assert_source_living(&living);
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    fn assert_source_dead(snapshot: &TeamPresentationSnapshot) {
        let hero = snapshot.moba_hud.as_ref().unwrap().hero.as_ref().unwrap();
        assert_eq!(hero.render_id, SOURCE_RENDER_ID);
        assert_eq!(hero.hp_raw, 0);
        let combat = decode_disclosed_combat_relation(component(
            snapshot,
            SOURCE_RENDER_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
        ))
        .unwrap();
        assert!(!combat.alive && !combat.attackable);
        assert_eq!(
            decode_disclosed_health(component(
                snapshot,
                SOURCE_RENDER_ID,
                DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID
            ))
            .unwrap()
            .0,
            0
        );
        let target = decode_disclosed_combat_relation(component(
            snapshot,
            TARGET_RENDER_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
        ))
        .unwrap();
        assert!(target.alive && target.attackable);
        assert!(!admitted(snapshot, TARGET_RENDER_ID));
    }

    fn assert_source_living(snapshot: &TeamPresentationSnapshot) {
        let hero = snapshot.moba_hud.as_ref().unwrap().hero.as_ref().unwrap();
        assert_eq!(hero.render_id, SOURCE_RENDER_ID);
        assert!(hero.hp_raw > 0);
        assert!(decode_disclosed_combat_relation(component(
            snapshot,
            SOURCE_RENDER_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
        ))
        .unwrap()
        .alive);
        assert!(admitted(snapshot, TARGET_RENDER_ID));
    }

    #[tokio::test]
    async fn owner_stages_disclose_compiled_state_and_ability_observations_have_no_authority_ack() {
        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let run_id = fixture.run_id.clone();
        let mut sequence = 1u64;
        let apply = |fixture: &mut RendererFixture, stage: &str, sequence: &mut u64| {
            let ack = fixture.apply_control_bytes(&serde_json::to_vec(&json!({
                "schema": CONTROL_SCHEMA,
                "run_id": run_id,
                "sequence": *sequence,
                "stage": stage,
            })).unwrap());
            assert!(ack.accepted, "{stage} {}", ack.reason);
            assert!(!ack.authority_ack);
            *sequence += 1;
            snapshot_payload(fixture.last_snapshot())
        };
        let startup_epoch = snapshot_payload(fixture.last_snapshot()).entities[0].disclosure_epoch;
        let ready = apply(&mut fixture, "owner_cast_ready", &mut sequence);
        assert!(ready.entities.iter().all(|entity| entity.disclosure_epoch == startup_epoch));
        let hero = ready.moba_hud.as_ref().unwrap().hero.as_ref().unwrap();
        assert_eq!(hero.content_id, FIXTURE_HERO_ID);
        assert!(hero.mana_supported);
        assert_eq!(hero.mana_raw, hero.max_mana_raw);
        assert!(hero.mana_raw > 0);
        assert_eq!(hero.skill_points, 0);
        assert!(hero.abilities.iter().all(|ability| ability.level == 1));
        assert!(hero.abilities.iter().all(|ability| ability.cooldown_raw == 0));
        assert!(!ready.moba_hud.as_ref().unwrap().summoner_protocol_enabled);
        let near = separation_raw(&ready);
        assert!(near + 2048 >= 200 * 1024 && near < 450 * 1024, "near {near}");
        assert!(admitted(&ready, TARGET_RENDER_ID));

        let cast = fixture.record_input(RendererInput {
            request_id: 7,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: startup_epoch,
            intent: Some(Intent::AbilityCast(omoba_core::game_proto::AbilityCastIntent {
                ability_index: 0,
                target_render_id: TARGET_RENDER_ID,
                x_raw: 0,
                y_raw: 0,
            })),
        }).unwrap();
        assert_eq!(cast["authority_ack"], false);
        assert_eq!(cast["intent"]["kind"], "AbilityCast");
        assert_eq!(cast["intent"]["ability_id"], "lumen_bolt");
        assert_eq!(cast["intent"]["target_kind"], "unit");
        assert_eq!(cast["intent"]["catalog_id"], omoba_template_ids::ability_by_name("lumen_bolt").unwrap().raw());
        assert!(cast["intent"].get("queued").is_none());

        let upgraded = apply(&mut fixture, "owner_upgrade_point", &mut sequence);
        let hero = upgraded.moba_hud.as_ref().unwrap().hero.as_ref().unwrap();
        assert_eq!(hero.content_id, FIXTURE_HERO_ID);
        assert_eq!(hero.level, 2);
        assert_eq!(hero.experience, 0);
        assert_eq!(hero.skill_points, 1);
        assert!(hero.abilities.iter().all(|ability| ability.level == 1));
        assert_eq!(upgraded.entities[0].disclosure_epoch, startup_epoch);
        let upgrade = fixture.record_input(RendererInput {
            request_id: 8,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: startup_epoch,
            intent: Some(Intent::AbilityUpgrade(omoba_core::game_proto::AbilityUpgradeIntent {
                ability_index: 0,
            })),
        }).unwrap();
        assert_eq!(upgrade["authority_ack"], false);
        assert_eq!(upgrade["intent"]["kind"], "AbilityUpgrade");
        assert_eq!(upgrade["intent"]["ability_id"], "lumen_bolt");
        assert_eq!(upgrade["intent"]["catalog_id"], cast["intent"]["catalog_id"]);
        assert!(upgrade["intent"].get("queued").is_none());
        assert!(upgrade["intent"].get("x_raw").is_none());
        assert!(upgrade["intent"].get("target_kind").is_none());

        let short = apply(&mut fixture, "owner_short_mana", &mut sequence);
        let hero = short.moba_hud.as_ref().unwrap().hero.as_ref().unwrap();
        let cost = omoba_template_ids::active_ability_const(omoba_template_ids::ability_by_name("lumen_bolt").unwrap())
            .unwrap()
            .levels[0]
            .mana_cost
            .raw();
        assert!(hero.mana_supported);
        assert_eq!(hero.mana_raw, cost - 1);
        assert!(hero.mana_raw < cost);
        assert!(hero.max_mana_raw > cost);
        // The owner HUD must see the bound current price, or the frontend cannot refuse the cast.
        assert_eq!((hero.abilities[0].mana_cost_state, hero.abilities[0].effective_mana_cost_raw), (2, cost));

        let cooled = apply(&mut fixture, "owner_cooldown", &mut sequence);
        let hero = cooled.moba_hud.as_ref().unwrap().hero.as_ref().unwrap();
        assert!(hero.mana_raw == hero.max_mana_raw && hero.mana_raw > 0);
        assert_eq!((hero.abilities[0].mana_cost_state, hero.abilities[0].effective_mana_cost_raw), (1, cost));
        assert!(hero.abilities[0].cooldown_raw > 0);
        assert!(hero.abilities[0].cooldown_raw <= hero.abilities[0].total_cooldown_raw);

        let hidden = apply(&mut fixture, "owner_hide_target", &mut sequence);
        let combat = decode_disclosed_combat_relation(component(
            &hidden,
            TARGET_RENDER_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
        )).unwrap();
        assert!(combat.alive && !combat.attackable);
        assert!(decode_disclosed_health(component(
            &hidden,
            TARGET_RENDER_ID,
            DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,
        )).unwrap().0 > 0);
        assert!(!admitted(&hidden, TARGET_RENDER_ID));

        let far = apply(&mut fixture, "owner_far_unit", &mut sequence);
        let far_sep = separation_raw(&far);
        assert!(far_sep + 2048 >= 900 * 1024 && far_sep > 700 * 1024, "far {far_sep}");
        assert!(admitted(&far, TARGET_RENDER_ID));

        let apprentice = apply(&mut fixture, "owner_apprentice", &mut sequence);
        let hero = apprentice.moba_hud.as_ref().unwrap().hero.as_ref().unwrap();
        assert_eq!(hero.content_id, "training_apprentice");
        assert_eq!(hero.skill_points, 1);
        assert_eq!(hero.abilities[0].ability_id, "apprentice_bolt");
        assert_eq!(hero.abilities[0].level, 0);
        assert_eq!(hero.abilities[2].level, 0);
        assert_eq!(apprentice.entities[0].disclosure_epoch, startup_epoch);

        let point = apply(&mut fixture, "owner_point_ready", &mut sequence);
        let hero = point.moba_hud.as_ref().unwrap().hero.as_ref().unwrap();
        assert_eq!(hero.content_id, "training_vanguard");
        assert_eq!(hero.abilities[3].ability_id, "vanguard_resolve");
        assert_eq!(hero.abilities[3].level, 1);
        assert!(hero.mana_supported && hero.mana_raw == hero.max_mana_raw);
        let point_cast = fixture.record_input(RendererInput {
            request_id: 9,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: startup_epoch,
            intent: Some(Intent::AbilityCast(omoba_core::game_proto::AbilityCastIntent {
                ability_index: 3,
                target_render_id: 0,
                x_raw: 1024,
                y_raw: 2048,
            })),
        }).unwrap();
        assert_eq!(point_cast["intent"]["ability_id"], "vanguard_resolve");
        assert_eq!(point_cast["intent"]["target_kind"], "point");
        assert_eq!(point_cast["intent"]["x_raw"], 1024);
        assert_eq!(point_cast["authority_ack"], false);
        assert!(point_cast["intent"].get("queued").is_none());

        let support = apply(&mut fixture, "owner_support_ready", &mut sequence);
        let hero = support.moba_hud.as_ref().unwrap().hero.as_ref().unwrap();
        assert_eq!(hero.content_id, "training_support");
        assert_eq!(hero.level, 1);
        assert_eq!(hero.skill_points, 0);
        assert_eq!(hero.abilities[3].ability_id, "lumen_aid");
        assert_eq!(hero.abilities[3].level, 1);
        assert!(hero.mana_supported && hero.mana_raw == hero.max_mana_raw);
        let ally = decode_disclosed_combat_relation(component(
            &support,
            TARGET_RENDER_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
        )).unwrap();
        assert!(ally.alive && ally.attackable);
        assert_eq!(ally.faction_kind, 1);
        assert_eq!(ally.team_id, i32::try_from(OWNER_TEAM_ID).unwrap());
        assert!(!admitted(&support, TARGET_RENDER_ID));
        assert_eq!(support.entities[0].disclosure_epoch, startup_epoch);
        let aid = fixture.record_input(RendererInput {
            request_id: 10,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: startup_epoch,
            intent: Some(Intent::AbilityCast(omoba_core::game_proto::AbilityCastIntent {
                ability_index: 3,
                target_render_id: TARGET_RENDER_ID,
                x_raw: 0,
                y_raw: 0,
            })),
        }).unwrap();
        assert_eq!(aid["authority_ack"], false);
        assert_eq!(aid["intent"]["ability_id"], "lumen_aid");
        assert_eq!(aid["intent"]["target_kind"], "unit");
        assert_eq!(aid["intent"]["target_render_id"], TARGET_RENDER_ID);
        assert_eq!(aid["intent"]["catalog_id"], omoba_template_ids::ability_by_name("lumen_aid").unwrap().raw());
        assert!(aid["intent"].get("queued").is_none());
        let aid_self = fixture.record_input(RendererInput {
            request_id: 11,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: startup_epoch,
            intent: Some(Intent::AbilityCast(omoba_core::game_proto::AbilityCastIntent {
                ability_index: 3,
                target_render_id: SOURCE_RENDER_ID,
                x_raw: 0,
                y_raw: 0,
            })),
        }).unwrap();
        assert_eq!(aid_self["intent"]["target_render_id"], SOURCE_RENDER_ID);
        assert_eq!(aid_self["intent"]["ability_id"], "lumen_aid");
        assert_eq!(aid_self["authority_ack"], false);
        assert!(aid_self["intent"].get("queued").is_none());

        let enemy = apply(&mut fixture, "owner_support_enemy", &mut sequence);
        let enemy_combat = decode_disclosed_combat_relation(component(
            &enemy,
            TARGET_RENDER_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
        )).unwrap();
        assert!(enemy_combat.alive && enemy_combat.attackable);
        assert_eq!(enemy_combat.team_id, i32::try_from(ENEMY_TEAM_ID).unwrap());
        assert!(admitted(&enemy, TARGET_RENDER_ID));
        assert_eq!(enemy.moba_hud.as_ref().unwrap().hero.as_ref().unwrap().content_id, "training_support");

        let concealed = apply(&mut fixture, "owner_support_conceal", &mut sequence);
        assert!(has_component(&concealed, SOURCE_RENDER_ID, DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID));
        assert!(!has_component(&concealed, TARGET_RENDER_ID, DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID));
        assert!(has_component(&concealed, TARGET_RENDER_ID, DEMO_RENDER_COMPONENT_SCHEMA_ID));
        assert!(!admitted(&concealed, TARGET_RENDER_ID));
        assert_eq!(concealed.entities.iter().find(|entity| entity.render_id == TARGET_RENDER_ID).unwrap().disclosure_epoch, startup_epoch);

        let dead = apply(&mut fixture, "owner_support_dead", &mut sequence);
        let dead_combat = decode_disclosed_combat_relation(component(
            &dead,
            TARGET_RENDER_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
        )).unwrap();
        assert!(!dead_combat.alive && !dead_combat.attackable);
        assert_eq!(dead_combat.team_id, i32::try_from(OWNER_TEAM_ID).unwrap());
        assert_eq!(decode_disclosed_health(component(
            &dead,
            TARGET_RENDER_ID,
            DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,
        )).unwrap().0, 0);
        assert!(!admitted(&dead, TARGET_RENDER_ID));

        let neutral = apply(&mut fixture, "owner_support_neutral", &mut sequence);
        let neutral_kind = omoba_core::runtime::faction_kind_discriminant(
            &omoba_core::runtime::FactionType::Neutral,
        )
        .unwrap();
        assert_eq!(neutral_kind, 3);
        let source_render = decode_demo_render_state(component(
            &neutral,
            SOURCE_RENDER_ID,
            DEMO_RENDER_COMPONENT_SCHEMA_ID,
        ))
        .unwrap();
        let source_combat = decode_disclosed_combat_relation(component(
            &neutral,
            SOURCE_RENDER_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
        ))
        .unwrap();
        assert_eq!(source_render.owner_player_id, OWNER_PLAYER_ID);
        assert_eq!(source_render.team_id, OWNER_TEAM_ID);
        assert_eq!(source_combat.faction_kind, 1);
        assert!(source_combat.alive);
        let target_render = decode_demo_render_state(component(
            &neutral,
            TARGET_RENDER_ID,
            DEMO_RENDER_COMPONENT_SCHEMA_ID,
        ))
        .unwrap();
        let neutral_combat = decode_disclosed_combat_relation(component(
            &neutral,
            TARGET_RENDER_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
        ))
        .unwrap();
        assert_eq!(target_render.owner_player_id, ENEMY_PLAYER_ID);
        assert_eq!(target_render.team_id, NEUTRAL_OTHER_TEAM_ID);
        assert_eq!(neutral_combat.faction_kind, neutral_kind);
        assert_ne!(neutral_combat.faction_kind, 1);
        assert!(neutral_combat.alive && neutral_combat.attackable);
        assert_eq!(neutral_combat.team_id, i32::try_from(NEUTRAL_OTHER_TEAM_ID).unwrap());
        assert_ne!(neutral_combat.team_id, source_combat.team_id);
        assert!(decode_disclosed_health(component(
            &neutral,
            TARGET_RENDER_ID,
            DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,
        ))
        .unwrap()
        .0 > 0);
        let source_faction = omoba_core::runtime::faction_from_disclosed_kind(
            source_combat.faction_kind,
            source_combat.team_id,
        )
        .unwrap();
        let target_faction = omoba_core::runtime::faction_from_disclosed_kind(
            neutral_combat.faction_kind,
            neutral_combat.team_id,
        )
        .unwrap();
        assert!(!source_faction.is_hostile_to(&target_faction));
        assert!(!admitted(&neutral, TARGET_RENDER_ID));
        assert_eq!(
            neutral.moba_hud.as_ref().unwrap().hero.as_ref().unwrap().content_id,
            "training_support"
        );
        assert_eq!(neutral.entities[0].disclosure_epoch, startup_epoch);

        let restored = apply(&mut fixture, "baseline", &mut sequence);
        let hero = restored.moba_hud.as_ref().unwrap().hero.as_ref().unwrap();
        assert_eq!(hero.content_id, FIXTURE_HERO_ID);
        assert!(!hero.mana_supported);
        assert_eq!(hero.mana_raw, 0);
        assert_eq!(hero.skill_points, 0);
        assert!(separation_raw(&restored) > 900 * 1024);
        assert!(admitted(&restored, TARGET_RENDER_ID));
        let restored_target = decode_disclosed_combat_relation(component(
            &restored,
            TARGET_RENDER_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
        )).unwrap();
        assert_eq!(restored_target.team_id, i32::try_from(ENEMY_TEAM_ID).unwrap());
        assert_eq!(restored_target.faction_kind, 1);
        assert!(restored_target.alive && restored_target.attackable);
        assert_eq!(restored.entities[0].disclosure_epoch, startup_epoch);
        assert!(!restored.moba_hud.as_ref().unwrap().summoner_protocol_enabled);
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[tokio::test]
    async fn owner_summoner_contract_gates_catalog_rows_without_authority_ack() {
        let hash = omoba_template_ids::CONTENT_CATALOG_DATA_HASH;
        assert!(!fixture_summoner_protocol_enabled(false, hash));
        assert!(!fixture_summoner_protocol_enabled(true, ""));
        assert!(!fixture_summoner_protocol_enabled(true, "not-the-catalog"));
        assert!(fixture_summoner_protocol_enabled(true, hash));

        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let run_id = fixture.run_id.clone();
        let mut sequence = 1u64;
        let apply = |fixture: &mut RendererFixture, stage: &str, sequence: &mut u64| {
            let ack = fixture.apply_control_bytes(&serde_json::to_vec(&json!({
                "schema": CONTROL_SCHEMA,
                "run_id": run_id,
                "sequence": *sequence,
                "stage": stage,
            })).unwrap());
            assert!(ack.accepted, "{stage} {}", ack.reason);
            assert!(!ack.authority_ack);
            *sequence += 1;
            snapshot_payload(fixture.last_snapshot())
        };
        let blink = omoba_template_ids::summoner_spell_by_name("training_point_blink_v1").unwrap();
        let heal = omoba_template_ids::summoner_spell_by_name("training_self_heal_v1").unwrap();
        let half = blink.cooldown.raw() / 2;
        let ready_ranks = apply(&mut fixture, "owner_cast_ready", &mut sequence);
        let ready_hero = ready_ranks.moba_hud.as_ref().unwrap().hero.as_ref().unwrap();
        assert_eq!(ready_hero.content_id, FIXTURE_HERO_ID);
        assert!(ready_hero.abilities.iter().all(|ability| ability.level == 1));
        assert!(!ready_ranks.moba_hud.as_ref().unwrap().summoner_protocol_enabled);
        let epoch = ready_ranks.entities[0].disclosure_epoch;

        let opened = apply(&mut fixture, "owner_summoner_ready", &mut sequence);
        let again = apply(&mut fixture, "owner_summoner_ready", &mut sequence);
        for snapshot in [&opened, &again] {
            let hud = snapshot.moba_hud.as_ref().unwrap();
            assert!(hud.summoner_protocol_enabled);
            assert_eq!(hud.schema_version, crate::moba_hud::SCHEMA_VERSION);
            let hero = hud.hero.as_ref().unwrap();
            assert_eq!(hero.content_id, FIXTURE_HERO_ID);
            assert_eq!(hero.render_id, SOURCE_RENDER_ID);
            assert!(hero.hp_raw > 0);
            assert!(hero.abilities.iter().all(|ability| ability.level == 1 && ability.cooldown_raw == 0));
            assert!(snapshot.entities.iter().all(|entity| entity.disclosure_epoch == epoch));
            let slot0 = hud.summoner_slot0.as_ref().unwrap();
            let slot1 = hud.summoner_slot1.as_ref().unwrap();
            assert_eq!(slot0.catalog_id, u32::from(blink.catalog_id));
            assert_eq!(slot0.spell_id, blink.id);
            assert_eq!(slot0.shape, crate::moba_hud::SUMMONER_SHAPE_POINT);
            assert_eq!(slot0.cooldown_raw, 0);
            assert_eq!(slot0.total_cooldown_raw, blink.cooldown.raw());
            assert_eq!(slot0.owner_player_id, OWNER_PLAYER_ID);
            assert_eq!(slot1.catalog_id, u32::from(heal.catalog_id));
            assert_eq!(slot1.spell_id, heal.id);
            assert_eq!(slot1.shape, crate::moba_hud::SUMMONER_SHAPE_SELF);
            assert_eq!(slot1.cooldown_raw, 0);
            assert_eq!(slot1.total_cooldown_raw, heal.cooldown.raw());
            assert_eq!(slot1.owner_player_id, OWNER_PLAYER_ID);
        }

        let point = fixture.record_input(RendererInput {
            request_id: 21,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: epoch,
            intent: Some(Intent::SummonerCast(omoba_core::game_proto::SummonerCastIntent {
                slot: 0,
                x_raw: Some(4096),
                y_raw: Some(8192),
            })),
        }).unwrap();
        assert_eq!(point["authority_ack"], false);
        assert_eq!(point["intent"]["kind"], "SummonerCast");
        assert_eq!(point["intent"]["slot"], 0);
        assert_eq!(point["intent"]["catalog_id"], i64::from(blink.catalog_id));
        assert_eq!(point["intent"]["spell_id"], "training_point_blink_v1");
        assert_eq!(point["intent"]["target_kind"], "point");
        assert_eq!(point["intent"]["x_raw"], 4096);
        assert_eq!(point["intent"]["y_raw"], 8192);
        assert!(point["intent"].get("queued").is_none());
        assert!(point["intent"].get("ability_index").is_none());

        let heal_cast = fixture.record_input(RendererInput {
            request_id: 22,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: epoch,
            intent: Some(Intent::SummonerCast(omoba_core::game_proto::SummonerCastIntent {
                slot: 1,
                x_raw: None,
                y_raw: None,
            })),
        }).unwrap();
        assert_eq!(heal_cast["authority_ack"], false);
        assert_eq!(heal_cast["intent"]["slot"], 1);
        assert_eq!(heal_cast["intent"]["catalog_id"], i64::from(heal.catalog_id));
        assert_eq!(heal_cast["intent"]["spell_id"], "training_self_heal_v1");
        assert_eq!(heal_cast["intent"]["target_kind"], "none");
        assert!(heal_cast["intent"]["x_raw"].is_null());
        assert!(heal_cast["intent"]["y_raw"].is_null());

        let cooled = apply(&mut fixture, "owner_summoner_cooldown", &mut sequence);
        let cooled_again = apply(&mut fixture, "owner_summoner_cooldown", &mut sequence);
        for snapshot in [&cooled, &cooled_again] {
            let hud = snapshot.moba_hud.as_ref().unwrap();
            assert!(hud.summoner_protocol_enabled);
            let slot0 = hud.summoner_slot0.as_ref().unwrap();
            assert_eq!(slot0.cooldown_raw, half);
            assert_eq!(slot0.total_cooldown_raw, blink.cooldown.raw());
            assert!(half > 0 && half < blink.cooldown.raw());
            assert_eq!(hud.summoner_slot1.as_ref().unwrap().cooldown_raw, 0);
            assert_eq!(hud.hero.as_ref().unwrap().abilities[0].level, 1);
            assert!(snapshot.entities.iter().all(|entity| entity.disclosure_epoch == epoch));
        }

        let unavailable = apply(&mut fixture, "owner_summoner_unavailable", &mut sequence);
        let open_empty = unavailable.moba_hud.as_ref().unwrap();
        assert!(open_empty.summoner_protocol_enabled);
        for slot in [open_empty.summoner_slot0.as_ref().unwrap(), open_empty.summoner_slot1.as_ref().unwrap()] {
            assert_eq!(slot.catalog_id, 0);
            assert_eq!(slot.cooldown_raw, 0);
            assert_eq!(slot.owner_player_id, OWNER_PLAYER_ID);
            assert!(slot.spell_id.is_empty());
        }

        let malformed = apply(&mut fixture, "owner_summoner_malformed", &mut sequence);
        let closed = malformed.moba_hud.as_ref().unwrap();
        assert!(!closed.summoner_protocol_enabled);
        assert_eq!(closed.summoner_slot0.as_ref().unwrap().owner_player_id, 0);
        assert_eq!(closed.summoner_slot1.as_ref().unwrap().owner_player_id, 0);
        let bad = fixture.record_input(RendererInput {
            request_id: 23,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: epoch,
            intent: Some(Intent::SummonerCast(omoba_core::game_proto::SummonerCastIntent {
                slot: 0,
                x_raw: Some(1),
                y_raw: Some(1),
            })),
        }).unwrap();
        assert_eq!(bad["authority_ack"], false);
        assert_eq!(bad["intent"]["catalog_id"], 99);
        assert!(bad["intent"]["spell_id"].is_null());
        assert!(bad["intent"]["target_kind"].is_null());

        let unsupported = apply(&mut fixture, "owner_summoner_unsupported", &mut sequence);
        assert!(!unsupported.moba_hud.as_ref().unwrap().summoner_protocol_enabled);
        let hidden = fixture.record_input(RendererInput {
            request_id: 24,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: epoch,
            intent: Some(Intent::SummonerCast(omoba_core::game_proto::SummonerCastIntent {
                slot: 0,
                x_raw: Some(1),
                y_raw: Some(1),
            })),
        }).unwrap();
        assert_eq!(hidden["intent"]["spell_id"], "training_point_blink_v1");
        assert_eq!(hidden["intent"]["catalog_id"], i64::from(blink.catalog_id));
        assert_eq!(hidden["authority_ack"], false);

        let armed = apply(&mut fixture, "owner_summoner_ready", &mut sequence);
        assert!(armed.moba_hud.as_ref().unwrap().summoner_protocol_enabled);
        let dead = apply(&mut fixture, "source_death", &mut sequence);
        let dead_hud = dead.moba_hud.as_ref().unwrap();
        assert!(dead_hud.summoner_protocol_enabled);
        assert_eq!(dead_hud.hero.as_ref().unwrap().render_id, SOURCE_RENDER_ID);
        assert_eq!(dead_hud.hero.as_ref().unwrap().hp_raw, 0);
        assert_eq!(dead_hud.summoner_slot0.as_ref().unwrap().spell_id, blink.id);
        assert!(dead.entities.iter().all(|entity| entity.disclosure_epoch == epoch));

        let replaced = apply(&mut fixture, "source_replacement", &mut sequence);
        let replaced_hud = replaced.moba_hud.as_ref().unwrap();
        assert!(replaced_hud.summoner_protocol_enabled);
        assert_eq!(replaced_hud.hero.as_ref().unwrap().render_id, REPLACEMENT_SOURCE_RENDER_ID);
        assert!(replaced_hud.hero.as_ref().unwrap().hp_raw > 0);
        assert_eq!(replaced_hud.summoner_slot0.as_ref().unwrap().owner_player_id, OWNER_PLAYER_ID);
        let replaced_source = replaced.entities.iter().find(|entity| entity.render_id == REPLACEMENT_SOURCE_RENDER_ID).unwrap();
        assert_eq!(replaced_source.disclosure_epoch, epoch + 1);
        let replaced_target = replaced.entities.iter().find(|entity| entity.render_id == TARGET_RENDER_ID).unwrap();
        assert_eq!(replaced_target.disclosure_epoch, epoch);

        let restored = apply(&mut fixture, "baseline", &mut sequence);
        let restored_hud = restored.moba_hud.as_ref().unwrap();
        assert!(!restored_hud.summoner_protocol_enabled);
        assert_eq!(restored_hud.summoner_slot0.as_ref().unwrap().owner_player_id, 0);
        let restored_hero = restored_hud.hero.as_ref().unwrap();
        assert_eq!(restored_hero.content_id, FIXTURE_HERO_ID);
        assert_eq!(restored_hero.render_id, SOURCE_RENDER_ID);
        assert_eq!(restored_hero.skill_points, 0);
        assert!(restored_hero.abilities.iter().all(|ability| ability.level == 1));
        assert_eq!(restored.entities[0].disclosure_epoch, epoch + 1);
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[tokio::test]
    async fn owner_item_and_recall_contracts_publish_without_authority_ack() {
        let hash = omoba_template_ids::CONTENT_CATALOG_DATA_HASH;
        assert!(!fixture_recall_protocol_enabled(false, hash));
        assert!(!fixture_recall_protocol_enabled(true, ""));
        assert!(!fixture_recall_protocol_enabled(true, "not-the-catalog"));
        assert!(fixture_recall_protocol_enabled(true, hash));
        assert!(item_inventory(ItemContract::Closed).is_err());
        let (ready_ids, ready_bits) = item_inventory(ItemContract::Ready).unwrap();
        assert_eq!(ready_ids, READY_ITEM_CATALOG);
        assert_eq!(ready_bits, [0; 6]);
        let (cooled_ids, cooled_bits) = item_inventory(ItemContract::Cooldown).unwrap();
        assert_eq!(cooled_ids, READY_ITEM_CATALOG);
        assert_eq!(f32::from_bits(cooled_bits[0]), 4.0);
        assert!(cooled_bits[1..].iter().all(|bits| *bits == 0));
        let (empty_ids, empty_bits) = item_inventory(ItemContract::Empty).unwrap();
        assert_eq!(empty_ids, [0; 6]);
        assert_eq!(empty_bits, [0; 6]);
        let (replaced_ids, _) = item_inventory(ItemContract::Replaced).unwrap();
        assert_eq!(replaced_ids[1], 6);

        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let run_id = fixture.run_id.clone();
        let mut sequence = 1u64;
        let apply = |fixture: &mut RendererFixture, stage: &str, sequence: &mut u64| {
            let ack = fixture.apply_control_bytes(&serde_json::to_vec(&json!({
                "schema": CONTROL_SCHEMA,
                "run_id": run_id,
                "sequence": *sequence,
                "stage": stage,
            })).unwrap());
            assert!(ack.accepted, "{stage} {}", ack.reason);
            assert!(!ack.authority_ack);
            *sequence += 1;
            snapshot_payload(fixture.last_snapshot())
        };
        let baseline = apply(&mut fixture, "baseline", &mut sequence);
        let epoch = baseline.entities[0].disclosure_epoch;
        assert!(baseline.owner_economy.is_none());
        assert!(!baseline.moba_hud.as_ref().unwrap().recall_protocol_enabled);
        assert_eq!(baseline.moba_hud.as_ref().unwrap().recall_remaining_raw, 0);
        assert!(!baseline.moba_hud.as_ref().unwrap().summoner_protocol_enabled);

        let ready = apply(&mut fixture, "owner_recall_ready", &mut sequence);
        let ready_hud = ready.moba_hud.as_ref().unwrap();
        assert!(ready_hud.recall_protocol_enabled);
        assert_eq!(ready_hud.recall_remaining_raw, 0);
        assert!(ready.owner_economy.is_none());
        assert!(ready.entities.iter().all(|entity| entity.disclosure_epoch == epoch));
        assert!(!ready_hud.summoner_protocol_enabled);

        let channel = apply(&mut fixture, "owner_recall_channel", &mut sequence);
        let channel_hud = channel.moba_hud.as_ref().unwrap();
        assert!(channel_hud.recall_protocol_enabled);
        assert_eq!(channel_hud.recall_remaining_raw, 3 * 1024);
        assert!(channel.entities.iter().all(|entity| entity.disclosure_epoch == epoch));

        let unavailable = apply(&mut fixture, "owner_recall_unavailable", &mut sequence);
        assert!(!unavailable.moba_hud.as_ref().unwrap().recall_protocol_enabled);
        assert_eq!(unavailable.moba_hud.as_ref().unwrap().recall_remaining_raw, 0);

        let armed = apply(&mut fixture, "owner_recall_ready", &mut sequence);
        assert!(armed.moba_hud.as_ref().unwrap().recall_protocol_enabled);
        let dead = apply(&mut fixture, "source_death", &mut sequence);
        assert!(dead.moba_hud.as_ref().unwrap().recall_protocol_enabled);
        assert_eq!(dead.moba_hud.as_ref().unwrap().hero.as_ref().unwrap().hp_raw, 0);
        assert!(dead.entities.iter().all(|entity| entity.disclosure_epoch == epoch));

        let closed = apply(&mut fixture, "baseline", &mut sequence);
        assert!(!closed.moba_hud.as_ref().unwrap().recall_protocol_enabled);
        assert!(closed.owner_economy.is_none());
        let floor = closed.entities[0].disclosure_epoch;

        let items = apply(&mut fixture, "owner_item_ready", &mut sequence);
        let economy = items.owner_economy.as_ref().unwrap();
        assert!(!economy.shop_protocol_enabled);
        assert!(!economy.shop_available);
        assert_eq!(economy.gold, 0);
        assert_eq!(economy.player_id, OWNER_PLAYER_ID);
        assert_eq!(economy.slots.len(), 6);
        for (index, catalog_id) in READY_ITEM_CATALOG.iter().enumerate() {
            assert_eq!(economy.slots[index].catalog_id, u32::from(*catalog_id));
            assert_eq!(economy.slots[index].cooldown_seconds, 0.0);
            assert_eq!(economy.slots[index].slot, index as u32);
        }
        assert!(!items.moba_hud.as_ref().unwrap().recall_protocol_enabled);
        assert!(items.entities.iter().all(|entity| entity.disclosure_epoch == floor));
        let flare = omoba_template_ids::MOBA_ITEM_CATALOG.iter().find(|item| item.catalog_id == 10).unwrap();
        let point = fixture.record_input(RendererInput {
            request_id: 31,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: floor,
            intent: Some(Intent::ItemUse(omoba_core::game_proto::ItemUseIntent {
                item_slot: 1,
                target_render_id: 0,
                x_raw: 4096,
                y_raw: 8192,
                explicit_x_raw: Some(4096),
                explicit_y_raw: Some(8192),
            })),
        }).unwrap();
        assert_eq!(point["authority_ack"], false);
        assert_eq!(point["intent"]["kind"], "ItemUse");
        assert_eq!(point["intent"]["item_slot"], 1);
        assert_eq!(point["intent"]["catalog_id"], 10);
        assert_eq!(point["intent"]["item_id"], flare.id);
        assert_eq!(point["intent"]["explicit_x_raw"], 4096);
        assert_eq!(point["intent"]["explicit_y_raw"], 8192);
        assert_eq!(point["intent"]["x_raw"], 4096);
        assert_eq!(point["intent"]["y_raw"], 8192);
        assert_eq!(point["intent"]["target_render_id"], 0);
        assert!(point["intent"].get("queued").is_none());

        let self_use = fixture.record_input(RendererInput {
            request_id: 32,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: floor,
            intent: Some(Intent::ItemUse(omoba_core::game_proto::ItemUseIntent {
                item_slot: 0,
                target_render_id: 0,
                x_raw: 0,
                y_raw: 0,
                explicit_x_raw: None,
                explicit_y_raw: None,
            })),
        }).unwrap();
        assert_eq!(self_use["intent"]["catalog_id"], 5);
        assert!(self_use["intent"]["explicit_x_raw"].is_null());
        assert!(self_use["intent"]["explicit_y_raw"].is_null());
        assert_eq!(self_use["intent"]["x_raw"], 0);
        assert_eq!(self_use["intent"]["y_raw"], 0);

        let cooled = apply(&mut fixture, "owner_item_cooldown", &mut sequence);
        assert_eq!(cooled.owner_economy.as_ref().unwrap().slots[0].cooldown_seconds, 4.0);
        assert_eq!(cooled.owner_economy.as_ref().unwrap().slots[0].catalog_id, 5);
        let empty = apply(&mut fixture, "owner_item_empty", &mut sequence);
        assert!(empty.owner_economy.as_ref().unwrap().slots.iter().all(|slot| slot.catalog_id == 0 && slot.cooldown_seconds == 0.0));
        let replaced = apply(&mut fixture, "owner_item_replaced", &mut sequence);
        assert_eq!(replaced.owner_economy.as_ref().unwrap().slots[1].catalog_id, 6);
        assert_eq!(replaced.owner_economy.as_ref().unwrap().slots[0].catalog_id, 5);
        let stride = fixture.record_input(RendererInput {
            request_id: 33,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: floor,
            intent: Some(Intent::ItemUse(omoba_core::game_proto::ItemUseIntent {
                item_slot: 1,
                target_render_id: 0,
                x_raw: 0,
                y_raw: 0,
                explicit_x_raw: None,
                explicit_y_raw: None,
            })),
        }).unwrap();
        assert_eq!(stride["intent"]["catalog_id"], 6);
        assert_eq!(stride["intent"]["item_id"], "moba_stride_charm");
        let far = apply(&mut fixture, "owner_item_far", &mut sequence);
        assert_eq!(far.owner_economy.as_ref().unwrap().slots[1].catalog_id, 10);
        assert!(separation_raw(&far) >= 900 * 1024);
        assert!(far.entities.iter().all(|entity| entity.disclosure_epoch == floor));

        let summoner = apply(&mut fixture, "owner_summoner_ready", &mut sequence);
        assert!(summoner.owner_economy.is_none());
        assert!(!summoner.moba_hud.as_ref().unwrap().recall_protocol_enabled);
        assert!(summoner.moba_hud.as_ref().unwrap().summoner_protocol_enabled);
        let restored = apply(&mut fixture, "baseline", &mut sequence);
        assert!(restored.owner_economy.is_none());
        assert!(!restored.moba_hud.as_ref().unwrap().recall_protocol_enabled);
        assert!(!restored.moba_hud.as_ref().unwrap().summoner_protocol_enabled);
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[tokio::test]
    async fn owner_shop_ready_keeps_epoch_slots_and_catalog_only_buy_wire() {
        use omoba_core::game_proto::ItemBuyIntent;
        use omoba_core::runtime::authoritative_mode::{MOBA, TOWER_DEFENSE};
        let hash = omoba_template_ids::CONTENT_CATALOG_DATA_HASH;
        assert!(!fixture_shop_protocol_enabled(false, hash));
        assert!(!fixture_shop_protocol_enabled(true, ""));
        assert!(fixture_shop_protocol_enabled(true, hash));
        let sources: Vec<_> = omoba_template_ids::MOBA_ITEM_CATALOG
            .iter()
            .map(|item| omoba_core::runtime::native::shop::RecipeSource {
                id: item.id,
                catalog_id: u32::from(item.catalog_id),
                cost: item.cost,
                recipe: item.recipe,
            })
            .collect();
        let projected = omoba_core::runtime::native::shop::project_catalog_recipes(&sources).unwrap();
        let repeated: Vec<_> = projected
            .iter()
            .filter(|recipe| recipe.components.iter().any(|component| component.count >= 2))
            .collect();
        assert_eq!(repeated.len(), 1);
        assert_eq!(repeated[0].catalog_id, 4);
        assert_eq!(repeated[0].components.len(), 1);
        assert_eq!(repeated[0].components[0].count, 2);
        let sword_id = u16::try_from(repeated[0].components[0].catalog_id).unwrap();
        let sword = omoba_template_ids::MOBA_ITEM_CATALOG
            .iter()
            .find(|item| item.catalog_id == sword_id)
            .unwrap();
        assert_eq!(sword.id, "moba_sword");
        assert_eq!(READY_ITEM_CATALOG.iter().filter(|id| **id == sword_id).count(), 1);

        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let run_id = fixture.run_id.clone();
        let mut sequence = 1u64;
        let apply = |fixture: &mut RendererFixture, stage: &str, sequence: &mut u64| {
            let ack = fixture.apply_control_bytes(
                &serde_json::to_vec(&json!({
                    "schema": CONTROL_SCHEMA,
                    "run_id": run_id,
                    "sequence": *sequence,
                    "stage": stage,
                }))
                .unwrap(),
            );
            assert!(ack.accepted, "{stage} {}", ack.reason);
            assert!(!ack.authority_ack);
            *sequence += 1;
            snapshot_payload(fixture.last_snapshot())
        };
        let _ = apply(&mut fixture, "baseline", &mut sequence);
        let td = apply(&mut fixture, "mode_td_epoch", &mut sequence);
        let floor = td.entities.iter().find(|entity| entity.render_id == SOURCE_RENDER_ID).unwrap().disclosure_epoch;
        assert_eq!(td.authoritative_mode.as_ref().unwrap().mode, TOWER_DEFENSE);
        assert_eq!(td.authoritative_mode.as_ref().unwrap().view_epoch, floor);
        let closed = apply(&mut fixture, "owner_item_ready", &mut sequence);
        let closed_economy = closed.owner_economy.as_ref().unwrap();
        assert!(!closed_economy.shop_protocol_enabled);
        assert!(!closed_economy.shop_available);
        assert_eq!(closed_economy.gold, 0);
        assert!(closed.entities.iter().all(|entity| entity.disclosure_epoch == floor));
        let restored = apply(&mut fixture, "baseline", &mut sequence);
        assert!(restored.owner_economy.is_none());
        assert_eq!(restored.authoritative_mode.as_ref().unwrap().mode, MOBA);
        assert_eq!(restored.authoritative_mode.as_ref().unwrap().view_epoch, floor);
        assert!(restored.entities.iter().any(|entity| entity.render_id == SOURCE_RENDER_ID && entity.disclosure_epoch == floor));
        let shop = apply(&mut fixture, "owner_shop_ready", &mut sequence);
        let economy = shop.owner_economy.as_ref().unwrap();
        assert!(economy.shop_protocol_enabled);
        assert!(economy.shop_available);
        assert_eq!(economy.shop_reason, 0);
        assert_eq!(economy.gold, 0);
        assert_eq!(economy.player_id, OWNER_PLAYER_ID);
        assert_eq!(economy.slots.len(), 6);
        for (index, catalog_id) in READY_ITEM_CATALOG.iter().enumerate() {
            assert_eq!(economy.slots[index].catalog_id, u32::from(*catalog_id));
        }
        assert_eq!(shop.authoritative_mode.as_ref().unwrap().mode, MOBA);
        assert_eq!(shop.authoritative_mode.as_ref().unwrap().view_epoch, floor);
        assert!(shop.entities.iter().any(|entity| entity.render_id == SOURCE_RENDER_ID && entity.disclosure_epoch == floor));
        assert!(shop.shop_receipts.is_empty());
        assert!(shop.shop_undo_receipts.is_empty());
        let buy = fixture
            .record_input(RendererInput {
                request_id: 41,
                player_id: OWNER_PLAYER_ID,
                disclosure_epoch: floor,
                intent: Some(Intent::ItemBuy(ItemBuyIntent { catalog_id: repeated[0].catalog_id })),
            })
            .unwrap();
        assert_eq!(buy["authority_ack"], false);
        assert_eq!(buy["intent"]["kind"], "ItemBuy");
        assert_eq!(buy["intent"]["catalog_id"], 4);
        assert!(buy["intent"].get("item_id").is_none());
        assert!(buy["intent"].get("item_slot").is_none());
        assert!(buy["intent"].get("queued").is_none());
        assert!(buy["intent"].get("x_raw").is_none());
        assert!(buy["intent"].get("y_raw").is_none());
        assert!(buy.get("receipt").is_none());
        let after = snapshot_payload(fixture.last_snapshot());
        assert_eq!(after.owner_economy.as_ref().unwrap().gold, 0);
        assert!(after.shop_receipts.is_empty());
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[tokio::test]
    async fn shop_transaction_correlation_follows_the_current_source_and_keeps_gold() {
        use omoba_core::game_proto::{ItemBuyIntent, ItemSellIntent};
        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let run_id = fixture.run_id.clone();
        let mut sequence = 1u64;
        let apply = |fixture: &mut RendererFixture, stage: &str, sequence: &mut u64| {
            let ack = fixture.apply_control_bytes(
                &serde_json::to_vec(&json!({
                    "schema": CONTROL_SCHEMA,
                    "run_id": run_id,
                    "sequence": *sequence,
                    "stage": stage,
                }))
                .unwrap(),
            );
            assert!(ack.accepted, "{stage} {}", ack.reason);
            assert!(!ack.authority_ack);
            *sequence += 1;
            snapshot_payload(fixture.last_snapshot())
        };
        let _ = apply(&mut fixture, "baseline", &mut sequence);
        let shop = apply(&mut fixture, "owner_shop_ready", &mut sequence);
        let floor = shop.view_epoch;
        assert_eq!(shop.owner_economy.as_ref().unwrap().gold, 0);
        assert!(shop.entities.iter().any(|entity| entity.render_id == SOURCE_RENDER_ID));
        let replaced = apply(&mut fixture, "controlled_owner_ward_same_epoch_source", &mut sequence);
        assert!(replaced.entities.iter().any(|entity| entity.render_id == REPLACEMENT_SOURCE_RENDER_ID && entity.disclosure_epoch == floor + 64));
        assert_eq!(replaced.view_epoch, floor);
        let current = apply(&mut fixture, "controlled_owner_shop_current", &mut sequence);
        let economy = current.owner_economy.as_ref().unwrap();
        assert_eq!(economy.gold, 0);
        assert!(economy.shop_available && economy.shop_protocol_enabled);
        assert!(current.entities.iter().any(|entity| entity.render_id == REPLACEMENT_SOURCE_RENDER_ID && entity.disclosure_epoch == floor + 64));
        assert_eq!(current.view_epoch, floor);
        assert!(current.shop_receipts.is_empty() && current.shop_transaction_correlations.is_empty());
        assert!(current.owner_ward_private.is_empty(), "opening the shop does not invent a ward projection");
        fixture.record_input(RendererInput {
            request_id: 41,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: floor,
            intent: Some(Intent::ItemBuy(ItemBuyIntent { catalog_id: 4 })),
        }).unwrap();
        let admitted = apply(&mut fixture, "controlled_owner_shop_admitted", &mut sequence);
        assert_eq!(admitted.owner_economy.as_ref().unwrap().gold, 0);
        assert!(admitted.shop_receipts.is_empty());
        assert_eq!(admitted.shop_transaction_correlations.len(), 1);
        assert_eq!(admitted.shop_transaction_correlations[0].request_id, 41);
        assert_eq!(admitted.shop_transaction_correlations[0].authority_input, 801);
        assert_eq!(admitted.shop_transaction_correlations[0].phase, crate::shop_presentation::SHOP_TXN_PHASE_ADMITTED);
        assert_eq!(admitted.shop_transaction_correlations[0].view_epoch, floor);
        let uncertain = apply(&mut fixture, "controlled_owner_shop_uncertain", &mut sequence);
        assert!(uncertain.shop_receipts.is_empty());
        assert_eq!(uncertain.shop_transaction_correlations[0].phase, crate::shop_presentation::SHOP_TXN_PHASE_UNCERTAIN);
        assert_eq!(uncertain.shop_transaction_correlations[0].request_id, 41);
        let settled = apply(&mut fixture, "controlled_owner_shop_settled", &mut sequence);
        assert!(settled.shop_transaction_correlations.is_empty());
        assert_eq!(settled.shop_receipts.len(), 1);
        assert_eq!(settled.shop_receipts[0].request_id, 41);
        assert_eq!(settled.shop_receipts[0].view_epoch, floor);
        assert_eq!(settled.shop_receipts[0].result_code, 0);
        assert_eq!(settled.owner_economy.as_ref().unwrap().gold, 0);
        let again = apply(&mut fixture, "controlled_owner_shop_settled", &mut sequence);
        assert_eq!(again.shop_receipts.len(), 1, "a repeated receipt settles once");
        fixture.record_input(RendererInput {
            request_id: 42,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: floor,
            intent: Some(Intent::ItemSell(ItemSellIntent { item_slot: 0 })),
        }).unwrap();
        let rejected = apply(&mut fixture, "controlled_owner_shop_rejected", &mut sequence);
        assert_eq!(rejected.owner_economy.as_ref().unwrap().gold, 0);
        let sell = rejected.shop_receipts.iter().find(|row| row.input_id == 802).unwrap();
        assert_eq!((sell.request_id, sell.result_code, sell.action_kind), (42, 6, 18));
        assert!(rejected.shop_transaction_correlations.is_empty());
        fixture.record_input(RendererInput {
            request_id: 43,
            player_id: OWNER_PLAYER_ID,
            disclosure_epoch: floor,
            intent: Some(Intent::ItemBuy(ItemBuyIntent { catalog_id: 4 })),
        }).unwrap();
        let stale = apply(&mut fixture, "controlled_owner_shop_stale", &mut sequence);
        assert_eq!(stale.owner_economy.as_ref().unwrap().gold, 0);
        assert_eq!(stale.shop_transaction_correlations.len(), 1);
        assert_eq!(stale.shop_transaction_correlations[0].request_id, 43);
        assert_eq!(stale.shop_transaction_correlations[0].authority_input, 803);
        assert!(stale.shop_receipts.iter().any(|row| row.input_id == 999 && row.request_id == 0));
        assert!(stale.shop_receipts.iter().all(|row| row.request_id != 43));
        let advanced = apply(&mut fixture, "controlled_owner_shop_epoch", &mut sequence);
        assert!(advanced.shop_transaction_correlations.is_empty());
        assert!(advanced.shop_receipts.is_empty());
        assert_eq!(advanced.view_epoch, floor + 1);
        assert_eq!(advanced.owner_economy.as_ref().unwrap().gold, 0);
        assert!(advanced.entities.iter().any(|entity| entity.render_id == REPLACEMENT_SOURCE_RENDER_ID && entity.disclosure_epoch == floor + 1));
        let full = apply(&mut fixture, "controlled_owner_shop_full", &mut sequence);
        assert_eq!(full.view_epoch, floor + 1);
        assert_eq!(full.owner_economy.as_ref().unwrap().gold, 0);
        assert!(full.shop_receipts.is_empty());
        assert_eq!(full.shop_transaction_correlations.len(), 64);
        assert_eq!(full.shop_transaction_correlations[0].request_id, 101);
        assert_eq!(full.shop_transaction_correlations[0].authority_input, 1);
        assert_eq!(full.shop_transaction_correlations[63].request_id, 164);
        assert_eq!(full.shop_transaction_correlations[63].authority_input, 64);
        assert!(full.shop_transaction_correlations.iter().all(|row| row.action_kind == 17 && row.phase == crate::shop_presentation::SHOP_TXN_PHASE_ADMITTED));
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[tokio::test]
    async fn controlled_owner_ward_projection_is_not_an_authority_ack() {
        use omoba_core::game_proto::WardPlacementIntent;
        use omoba_core::runtime::ward_transport::{
            decode_owner_ward_private, OWNER_WARD_OUTCOME_REJECTED, OWNER_WARD_OUTCOME_SETTLED,
            WARD_PHASE_APPLIED, WARD_PHASE_FORWARDED, WARD_PHASE_REJECTED, WARD_PHASE_UNCERTAIN,
            WARD_PROTOCOL_VERSION,
        };
        let row = omoba_template_ids::trinket_by_name("training_scout_lantern_v1").unwrap();
        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let run_id = fixture.run_id.clone();
        let mut sequence = 1u64;
        let apply = |fixture: &mut RendererFixture, stage: &str, sequence: &mut u64| {
            let ack = fixture.apply_control_bytes(
                &serde_json::to_vec(&json!({
                    "schema": CONTROL_SCHEMA,
                    "run_id": run_id,
                    "sequence": *sequence,
                    "stage": stage,
                }))
                .unwrap(),
            );
            assert!(ack.accepted, "{stage} {}", ack.reason);
            assert!(!ack.authority_ack);
            *sequence += 1;
            snapshot_payload(fixture.last_snapshot())
        };
        let ready = apply(&mut fixture, "controlled_owner_ward_projection", &mut sequence);
        assert_eq!(ready.owner_ward_protocol, WARD_PROTOCOL_VERSION);
        let (projection, outcomes) = decode_owner_ward_private(&ready.owner_ward_private).unwrap();
        assert_eq!(projection.player_id, OWNER_PLAYER_ID);
        assert_eq!(projection.view_epoch, ready.view_epoch);
        assert_eq!(projection.catalog_id, row.catalog_id);
        assert_eq!(projection.charges, row.initial_charges);
        assert_eq!(projection.max_charges, row.max_charges);
        assert!(projection.placement_capable);
        assert_eq!(outcomes.count, 0);
        assert_eq!(outcomes.uncertain_through, 0);
        let malformed = apply(&mut fixture, "controlled_owner_ward_malformed", &mut sequence);
        assert_eq!(malformed.owner_ward_protocol, 0);
        assert!(malformed.owner_ward_private.is_empty());
        let settled = apply(&mut fixture, "controlled_owner_ward_settled", &mut sequence);
        let (_, settled_outcomes) = decode_owner_ward_private(&settled.owner_ward_private).unwrap();
        assert_eq!(settled.owner_ward_protocol, WARD_PROTOCOL_VERSION);
        assert_eq!(settled_outcomes.count, 1);
        assert_eq!(settled_outcomes.rows[0].admitted_correlation, 7);
        assert_eq!(settled_outcomes.rows[0].actor_generation, 3);
        assert_eq!(settled_outcomes.rows[0].stage, OWNER_WARD_OUTCOME_SETTLED);
        let rejected = apply(&mut fixture, "controlled_owner_ward_rejected", &mut sequence);
        let (_, rejected_outcomes) = decode_owner_ward_private(&rejected.owner_ward_private).unwrap();
        assert_eq!(rejected_outcomes.rows[0].stage, OWNER_WARD_OUTCOME_REJECTED);
        let uncertain = apply(&mut fixture, "controlled_owner_ward_uncertain", &mut sequence);
        let (uncertain_projection, uncertain_outcomes) = decode_owner_ward_private(&uncertain.owner_ward_private).unwrap();
        assert_eq!(uncertain_outcomes.uncertain_through, 7);
        assert_eq!(uncertain_outcomes.count, 0);
        assert_eq!(uncertain_projection.charges, row.initial_charges);
        let cleared = apply(&mut fixture, "controlled_owner_ward_clear", &mut sequence);
        assert_eq!(cleared.owner_ward_protocol, 0);
        assert!(cleared.owner_ward_private.is_empty());
        let again = apply(&mut fixture, "controlled_owner_ward_projection", &mut sequence);
        let placed = fixture
            .record_input(RendererInput {
                request_id: 44,
                player_id: OWNER_PLAYER_ID,
                disclosure_epoch: again.view_epoch,
                intent: Some(Intent::WardPlacement(WardPlacementIntent {
                    slot: 0,
                    x_raw: Some(3),
                    y_raw: Some(4),
                })),
            })
            .unwrap();
        assert_eq!(placed["authority_ack"], false);
        assert_eq!(placed["intent"]["kind"], "WardPlacement");
        assert_eq!(placed["intent"]["slot"], 0);
        assert_eq!(placed["intent"]["x_raw"], 3);
        assert_eq!(placed["intent"]["y_raw"], 4);
        assert!(placed["intent"].get("queued").is_none());
        assert!(placed.get("receipt").is_none());
        let after = snapshot_payload(fixture.last_snapshot());
        let (after_projection, _) = decode_owner_ward_private(&after.owner_ward_private).unwrap();
        assert_eq!(after.owner_ward_protocol, WARD_PROTOCOL_VERSION);
        assert_eq!(after_projection.charges, row.initial_charges);
        assert_eq!(after_projection.max_charges, row.max_charges);
        let forwarded = apply(&mut fixture, "controlled_owner_ward_forwarded", &mut sequence);
        assert_eq!(forwarded.ward_input_correlations.len(), 1);
        let mapped = &forwarded.ward_input_correlations[0];
        assert_eq!(mapped.renderer_request, 44);
        assert_eq!(mapped.authority_input, 901);
        assert_ne!(mapped.renderer_request, u64::from(mapped.authority_input));
        assert_eq!(mapped.phase, WARD_PHASE_FORWARDED);
        assert_eq!(mapped.view_epoch, forwarded.view_epoch);
        let actor = forwarded
            .entities
            .iter()
            .find(|entity| entity.render_id == SOURCE_RENDER_ID)
            .unwrap()
            .disclosure_epoch as u32;
        assert_eq!(mapped.actor_generation, actor);
        let (_, forwarded_outcomes) = decode_owner_ward_private(&forwarded.owner_ward_private).unwrap();
        assert_eq!(forwarded_outcomes.count, 0);
        let uncertain_mapped = apply(&mut fixture, "controlled_owner_ward_uncertain_mapped", &mut sequence);
        assert_eq!(uncertain_mapped.ward_input_correlations[0].renderer_request, 44);
        assert_eq!(uncertain_mapped.ward_input_correlations[0].authority_input, 901);
        assert_eq!(uncertain_mapped.ward_input_correlations[0].phase, WARD_PHASE_UNCERTAIN);
        let settled_mapped = apply(&mut fixture, "controlled_owner_ward_settled_mapped", &mut sequence);
        let (_, settled_mapped_outcomes) = decode_owner_ward_private(&settled_mapped.owner_ward_private).unwrap();
        assert_eq!(settled_mapped_outcomes.rows[0].admitted_correlation, 901);
        assert_eq!(settled_mapped_outcomes.rows[0].actor_generation, actor);
        assert_eq!(settled_mapped_outcomes.rows[0].stage, OWNER_WARD_OUTCOME_SETTLED);
        assert_ne!(settled_mapped_outcomes.rows[0].admitted_correlation, 44);
        assert_eq!(settled_mapped.view_epoch, forwarded.view_epoch);
        assert_eq!(settled_mapped.ward_input_correlations[0].phase, WARD_PHASE_APPLIED);
        assert_eq!(settled_mapped.ward_input_correlations[0].authority_input, 901);
        let rejected_mapped = apply(&mut fixture, "controlled_owner_ward_rejected_mapped", &mut sequence);
        let (_, rejected_mapped_outcomes) = decode_owner_ward_private(&rejected_mapped.owner_ward_private).unwrap();
        assert_eq!(rejected_mapped_outcomes.rows[0].admitted_correlation, 901);
        assert_eq!(rejected_mapped_outcomes.rows[0].stage, OWNER_WARD_OUTCOME_REJECTED);
        assert_eq!(rejected_mapped.ward_input_correlations[0].renderer_request, 44);
        let local = apply(&mut fixture, "controlled_owner_ward_local_rejected", &mut sequence);
        assert_eq!(local.ward_input_correlations[0].authority_input, 0);
        assert_eq!(local.ward_input_correlations[0].phase, WARD_PHASE_REJECTED);
        assert_eq!(local.ward_input_correlations[0].renderer_request, 44);
        let (_, local_outcomes) = decode_owner_ward_private(&local.owner_ward_private).unwrap();
        assert_eq!(local_outcomes.count, 0);
        let replaced = apply(&mut fixture, "controlled_owner_ward_same_epoch_source", &mut sequence);
        assert_eq!(replaced.view_epoch, forwarded.view_epoch);
        assert_eq!(replaced.ward_input_correlations[0].view_epoch, forwarded.view_epoch);
        assert_eq!(replaced.ward_input_correlations[0].actor_generation, (forwarded.view_epoch + 64) as u32);
        assert!(replaced.entities.iter().any(|entity| {
            entity.render_id == REPLACEMENT_SOURCE_RENDER_ID && entity.disclosure_epoch == forwarded.view_epoch + 64
        }));
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    fn control_document(run_id: &str, sequence: u64, stage: &str) -> Vec<u8> {
        let mut bytes = serde_json::to_vec(&json!({
            "schema": CONTROL_SCHEMA,
            "run_id": run_id,
            "sequence": sequence,
            "stage": stage,
        }))
        .unwrap();
        bytes.push(b'\n');
        bytes
    }

    fn push_control_error(fixture: &mut RendererFixture, operation: &'static str, error: std::io::Error) {
        fixture.pending_control_reads.push_back(ControlFileBody::Failed { operation, error });
    }

    #[test]
    fn control_io_classification_is_platform_narrow() {
        assert!(matches!(
            classify_control_io(&std::io::Error::from_raw_os_error(2)),
            ControlIoClass::Absent
        ));
        assert!(matches!(
            classify_control_io(&std::io::Error::new(std::io::ErrorKind::NotFound, "missing")),
            ControlIoClass::Absent
        ));
        assert!(matches!(
            classify_control_io(&std::io::Error::new(std::io::ErrorKind::Interrupted, "again")),
            ControlIoClass::Interrupted
        ));
        assert!(matches!(
            classify_control_io(&std::io::Error::from_raw_os_error(3)),
            ControlIoClass::Permanent
        ));
        assert!(matches!(
            classify_control_io(&std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied")),
            ControlIoClass::Permanent
        ));
        for code in [5, 32, 33] {
            let class = classify_control_io(&std::io::Error::from_raw_os_error(code));
            if cfg!(windows) {
                assert!(matches!(class, ControlIoClass::Transient), "{code}");
            } else {
                assert!(matches!(class, ControlIoClass::Permanent), "{code}");
            }
        }
    }

    #[test]
    fn control_file_read_is_one_bounded_observation() {
        let dir = std::env::temp_dir().join(format!("omoba-control-read-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        let path = dir.join("control.json");
        assert!(matches!(read_control_file(&path), ControlFileBody::Absent));
        fs::write(&path, b"{}").unwrap();
        match read_control_file(&path) {
            ControlFileBody::Bytes(bytes) => assert_eq!(bytes, b"{}"),
            ControlFileBody::Absent | ControlFileBody::Oversized { .. } | ControlFileBody::Failed { .. } => {
                panic!("expected the bounded file bytes");
            }
        }
        fs::write(&path, vec![b'x'; MAX_CONTROL_BYTES + 50]).unwrap();
        match read_control_file(&path) {
            ControlFileBody::Oversized { observed_len } => {
                assert_eq!(observed_len, (MAX_CONTROL_BYTES + 1) as u64);
            }
            _ => panic!("expected the capped oversized observation"),
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn control_read_missing_during_replacement_then_next_sequence() {
        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let path = fixture.control_path();
        let ack_path = options.evidence_dir.join("control.ack.json");
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.world.control_sequence, 0);
        assert_eq!(fixture.control_read_streak, 0);
        assert!(!ack_path.exists());
        let published = fixture.world.published_sequence;

        let partial = path.with_extension("partial");
        fs::write(&partial, control_document(&fixture.run_id, 1, "baseline")).unwrap();
        let _ = fs::remove_file(&path);
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.world.control_sequence, 0);
        assert_eq!(fixture.world.published_sequence, published);
        assert!(fixture.last_control_token.is_empty());
        assert!(!ack_path.exists());

        fs::rename(&partial, &path).unwrap();
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.world.control_sequence, 1);
        assert_eq!(fixture.world.published_sequence, published + 1);
        let ack: serde_json::Value = serde_json::from_slice(&fs::read(&ack_path).unwrap()).unwrap();
        assert_eq!(ack["accepted"], true);
        assert_eq!(ack["sequence"], 1);
        assert_eq!(ack["stage"], "baseline");
        assert_eq!(ack["authority_ack"], false);
        let ack_bytes = fs::read(&ack_path).unwrap();
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.world.control_sequence, 1);
        assert_eq!(fixture.world.published_sequence, published + 1);
        assert_eq!(fs::read(&ack_path).unwrap(), ack_bytes);

        fs::remove_file(&path).unwrap();
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.world.control_sequence, 1);
        assert_eq!(fs::read(&ack_path).unwrap(), ack_bytes);

        write_atomic(&path, &control_document(&fixture.run_id, 2, "remove_target")).unwrap();
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.world.control_sequence, 2);
        assert_eq!(fixture.world.published_sequence, published + 2);
        let next_ack: serde_json::Value = serde_json::from_slice(&fs::read(&ack_path).unwrap()).unwrap();
        assert_eq!(next_ack["accepted"], true);
        assert_eq!(next_ack["sequence"], 2);
        assert_eq!(next_ack["authority_ack"], false);
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.world.control_sequence, 2);
        assert_eq!(fixture.world.published_sequence, published + 2);
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[tokio::test]
    async fn control_read_malformed_and_oversized_leave_state_and_ack() {
        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let path = fixture.control_path();
        let ack_path = options.evidence_dir.join("control.ack.json");
        let published = fixture.world.published_sequence;
        let snapshot = fixture.last_snapshot().sequence;
        fs::write(&path, b"{").unwrap();
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.world.control_sequence, 0);
        assert_eq!(fixture.world.published_sequence, published);
        assert_eq!(fixture.last_snapshot().sequence, snapshot);
        let ack: serde_json::Value = serde_json::from_slice(&fs::read(&ack_path).unwrap()).unwrap();
        assert_eq!(ack["accepted"], false);
        assert_eq!(ack["reason"], "malformed");
        assert_eq!(ack["sequence"], 0);
        assert_eq!(ack["authority_ack"], false);
        let malformed_ack = fs::read(&ack_path).unwrap();
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fs::read(&ack_path).unwrap(), malformed_ack);
        assert_eq!(fixture.world.control_sequence, 0);
        assert_eq!(fixture.world.published_sequence, published);

        fs::write(&path, vec![b'x'; MAX_CONTROL_BYTES + 1]).unwrap();
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.world.control_sequence, 0);
        assert_eq!(fixture.world.published_sequence, published);
        assert_eq!(fixture.last_snapshot().sequence, snapshot);
        assert_eq!(
            fixture.last_control_token,
            format!("oversized:{}", MAX_CONTROL_BYTES as u64 + 1)
        );
        let oversized: serde_json::Value = serde_json::from_slice(&fs::read(&ack_path).unwrap()).unwrap();
        assert_eq!(oversized["accepted"], false);
        assert_eq!(oversized["reason"], "oversized");
        assert_eq!(oversized["authority_ack"], false);
        let oversized_ack = fs::read(&ack_path).unwrap();
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fs::read(&ack_path).unwrap(), oversized_ack);
        assert_eq!(fixture.world.control_sequence, 0);

        push_control_error(&mut fixture, "read", std::io::Error::from_raw_os_error(5));
        let denied = fixture.poll_control_file();
        assert_eq!(fs::read(&ack_path).unwrap(), oversized_ack);
        assert_eq!(fixture.world.control_sequence, 0);
        assert_eq!(fixture.world.published_sequence, published);
        assert_eq!(fixture.last_snapshot().sequence, snapshot);
        if cfg!(windows) {
            assert!(denied.is_ok());
            assert_eq!(fixture.control_read_streak, 1);
        } else {
            let text = denied.unwrap_err().to_string();
            assert!(text.contains("control file read"));
            assert!(text.contains("raw_os_error 5"));
            assert_eq!(fixture.control_read_streak, 0);
        }
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[tokio::test]
    async fn control_read_permanent_error_fails_immediately() {
        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let published = fixture.world.published_sequence;
        push_control_error(&mut fixture, "open", std::io::Error::from_raw_os_error(3));
        let text = fixture.poll_control_file().unwrap_err().to_string();
        assert!(text.contains("control file open"));
        assert!(text.contains(&fixture.control_path().display().to_string()));
        assert!(text.contains("raw_os_error 3"));
        assert!(text.contains("transient_streak 0"));
        assert_eq!(fixture.control_read_streak, 0);
        assert_eq!(fixture.world.control_sequence, 0);
        assert_eq!(fixture.world.published_sequence, published);
        assert!(fixture.last_control_token.is_empty());
        assert!(!options.evidence_dir.join("control.ack.json").exists());

        push_control_error(
            &mut fixture,
            "read",
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        );
        let text = fixture.poll_control_file().unwrap_err().to_string();
        assert!(text.contains("control file read"));
        assert!(text.contains("raw_os_error none"));
        assert!(text.contains("transient_streak 0"));
        assert_eq!(fixture.control_read_streak, 0);
        assert_eq!(fixture.world.control_sequence, 0);
        assert!(!options.evidence_dir.join("control.ack.json").exists());
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn control_read_transient_before_complete_applies_once() {
        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let ack_path = options.evidence_dir.join("control.ack.json");
        let published = fixture.world.published_sequence;
        push_control_error(&mut fixture, "read", std::io::Error::from_raw_os_error(5));
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.control_read_streak, 1);
        assert_eq!(fixture.world.control_sequence, 0);
        assert_eq!(fixture.world.published_sequence, published);
        assert!(fixture.last_control_token.is_empty());
        assert!(!ack_path.exists());

        fixture.pending_control_reads.push_back(ControlFileBody::Absent);
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.control_read_streak, 0);
        assert_eq!(fixture.world.control_sequence, 0);
        assert!(!ack_path.exists());

        let bytes = control_document(&fixture.run_id, 1, "baseline");
        fixture.pending_control_reads.push_back(ControlFileBody::Bytes(bytes.clone()));
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.control_read_streak, 0);
        assert_eq!(fixture.world.control_sequence, 1);
        assert_eq!(fixture.world.published_sequence, published + 1);
        let ack: serde_json::Value = serde_json::from_slice(&fs::read(&ack_path).unwrap()).unwrap();
        assert_eq!(ack["accepted"], true);
        assert_eq!(ack["sequence"], 1);
        assert_eq!(ack["authority_ack"], false);
        let ack_bytes = fs::read(&ack_path).unwrap();

        fixture.pending_control_reads.push_back(ControlFileBody::Bytes(bytes));
        assert!(fixture.poll_control_file().is_ok());
        assert_eq!(fixture.world.control_sequence, 1);
        assert_eq!(fixture.world.published_sequence, published + 1);
        assert_eq!(fs::read(&ack_path).unwrap(), ack_bytes);
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn control_read_persistent_transient_exhaustion_fails() {
        let options = options();
        let mut fixture = RendererFixture::start(options.clone()).await.unwrap();
        let published = fixture.world.published_sequence;
        let codes = [32i32, 33, 5];
        for index in 0..(MAX_CONTROL_READ_TRANSIENTS - 1) {
            let code = codes[(index as usize) % codes.len()];
            push_control_error(&mut fixture, "read", std::io::Error::from_raw_os_error(code));
            if let Err(error) = fixture.poll_control_file() {
                panic!("poll {index}: {error}");
            }
        }
        assert_eq!(fixture.control_read_streak, MAX_CONTROL_READ_TRANSIENTS - 1);
        assert_eq!(fixture.world.control_sequence, 0);
        assert_eq!(fixture.world.published_sequence, published);
        assert!(fixture.last_control_token.is_empty());
        assert!(!options.evidence_dir.join("control.ack.json").exists());

        push_control_error(&mut fixture, "open", std::io::Error::from_raw_os_error(5));
        let text = fixture.poll_control_file().unwrap_err().to_string();
        assert!(text.starts_with("ipc: control file open "));
        assert!(text.contains(&fixture.control_path().display().to_string()));
        assert!(text.contains("raw_os_error 5"));
        assert!(text.contains(&format!("transient_streak {MAX_CONTROL_READ_TRANSIENTS}")));
        assert_eq!(fixture.control_read_streak, MAX_CONTROL_READ_TRANSIENTS);
        assert_eq!(fixture.world.control_sequence, 0);
        assert_eq!(fixture.world.published_sequence, published);
        assert!(fixture.last_control_token.is_empty());
        assert!(!options.evidence_dir.join("control.ack.json").exists());
        drop(fixture);
        let _ = fs::remove_dir_all(options.evidence_dir);
    }

    fn separation_raw(snapshot: &TeamPresentationSnapshot) -> i128 {
        let source = decode_demo_render_state(component(snapshot, SOURCE_RENDER_ID, DEMO_RENDER_COMPONENT_SCHEMA_ID)).unwrap();
        let target = decode_demo_render_state(component(snapshot, TARGET_RENDER_ID, DEMO_RENDER_COMPONENT_SCHEMA_ID)).unwrap();
        let dx = i128::from(source.x_raw) - i128::from(target.x_raw);
        let dy = i128::from(source.y_raw) - i128::from(target.y_raw);
        integer_sqrt(dx * dx + dy * dy)
    }

    fn integer_sqrt(value: i128) -> i128 {
        if value <= 0 {
            return 0;
        }
        let mut x = value;
        let mut y = (x + 1) / 2;
        while y < x {
            x = y;
            y = (x + value / x) / 2;
        }
        x
    }
}
