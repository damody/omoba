use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use omoba_core::{
    game_proto::{
        render_lifecycle_event, renderer_ipc_envelope, FogTilePresentation,
        PolygonOccluderPresentation, PolygonPointPresentation, PresentationComponent,
        PresentationRenderEntity, RememberedGhostPresentation, RenderLifecycleBatch,
        RenderLifecycleEvent, RenderLifecycleForget, RenderLifecycleHide, RenderLifecycleResetView,
        RendererInput, RendererIpcEnvelope, RuntimeReadyPresentation, TeamPresentationSnapshot,
        TreeOccluderPresentation, VisionCirclePresentation,
    },
    runtime::{decode_demo_render_state, FilteredRenderSnapshot, RenderMemoryDirective},
};
use prost::Message;
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc, watch, Mutex},
};

use crate::{config::ClientRuntimeConfig, ClientRuntimeError};
use omoba_core::comp::perf_window::PresentationIpcMeter;

pub use omoba_core::renderer_protocol::{
    PRESENTATION_MAGIC, PRESENTATION_PROTOCOL_VERSION, MAX_PRESENTATION_FRAME_BYTES,
};
type SharedCues = Arc<std::sync::Mutex<crate::cue_retention::CueRetention>>;
type SharedReady = Arc<std::sync::Mutex<Option<Arc<RendererIpcEnvelope>>>>;
static RENDERER_CONNECTION: AtomicU64 = AtomicU64::new(1);

pub struct PresentationHub {
    player_id: u32,
    ready: SharedReady,
    damage: SharedCues,
    shop_receipts: crate::shop_presentation::ShopReceiptHistory,
    shop_pending: crate::shop_recovery::PendingShopQueries,
    shop_clock: std::time::Instant,
    shop_smoke_stage: u8,
    shop_smoke_query: Option<u32>,
    shop_protocol_enabled: bool,
    recall_protocol_enabled: bool,
    recall_smoke_stage: u8,
    combat_smoke_stage: u8,
    combat_smoke_score: Option<(u32, u32, u32)>,
    upgrade_smoke_stage: u8,
    first_learn_smoke_stage: u8,
    first_learn_smoke_tick: u64,
    fog_cache: DemoFogCache,
    latest_tx: watch::Sender<Option<Arc<RendererIpcEnvelope>>>,
    critical_tx: mpsc::Sender<RendererIpcEnvelope>,
    input_rx: mpsc::Receiver<RendererInput>,
    input_tx: mpsc::Sender<RendererInput>,
    connected: Arc<AtomicBool>,
    disconnected_at_ms: Arc<AtomicU64>,
}

impl PresentationHub {
    pub async fn bind(config: &ClientRuntimeConfig) -> Result<Self, ClientRuntimeError> {
        let listener = TcpListener::bind(config.presentation_bind)
            .await
            .map_err(|error| ClientRuntimeError::Ipc(error.to_string()))?;
        let (latest_tx, latest_rx) = watch::channel(None);
        let (critical_tx, critical_rx) = mpsc::channel(256);
        let (input_tx, input_rx) = mpsc::channel(256);
        let connected = Arc::new(AtomicBool::new(false));
        let disconnected_at_ms = Arc::new(AtomicU64::new(now_ms()));
        let damage = Arc::new(std::sync::Mutex::new(crate::cue_retention::CueRetention::default()));
        let ready = Arc::new(std::sync::Mutex::new(None));
        tokio::spawn(serve_connections(
            listener,
            config.player_id,
            config.team_id,
            latest_rx,
            Arc::new(Mutex::new(critical_rx)),
            input_tx.clone(),
            connected.clone(),
            disconnected_at_ms.clone(),
            damage.clone(),
            ready.clone(),
        ));
        Ok(Self {
            player_id: config.player_id,
            ready,
            damage,
            shop_receipts: Default::default(),
            shop_pending: Default::default(),
            shop_clock: std::time::Instant::now(),
            shop_smoke_stage: 0,
            shop_smoke_query: None,
            shop_protocol_enabled: false,
            recall_protocol_enabled: false,
            recall_smoke_stage: 0,
            combat_smoke_stage: 0,
            combat_smoke_score: None,
            upgrade_smoke_stage: 0,
            first_learn_smoke_stage: 0,
            first_learn_smoke_tick: 0,
            fog_cache: DemoFogCache::default(),
            latest_tx,
            critical_tx,
            input_rx,
            input_tx,
            connected,
            disconnected_at_ms,
        })
    }

    pub fn publish_latest(&self, envelope: RendererIpcEnvelope) {
        if matches!(envelope.payload, Some(renderer_ipc_envelope::Payload::RuntimeReady(_))) {
            *self.ready.lock().expect("ready retention poisoned") = Some(Arc::new(envelope.clone()));
        }
        self.latest_tx.send_replace(Some(Arc::new(envelope)));
    }

    pub fn snapshot_envelope(
        &mut self,
        sequence: u64,
        authoritative_tick: u64,
        view_epoch: u64,
        snapshot: FilteredRenderSnapshot,
        runtime_rtt_us: u64,
    ) -> RendererIpcEnvelope {
        let mut hud = crate::moba_hud::project(&snapshot, self.player_id);
        if let Some(hud) = &mut hud { hud.recall_protocol_enabled = self.recall_protocol_enabled; }
        let mut economy = crate::shop_presentation::economy(&snapshot, self.player_id);
        if let Some(economy) = &mut economy {
            economy.shop_protocol_enabled = self.shop_protocol_enabled;
        }
        let mut envelope = snapshot_envelope_cached(
            sequence,
            authoritative_tick,
            view_epoch,
            snapshot,
            runtime_rtt_us,
            &mut self.fog_cache,
        );
        if let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = &mut envelope.payload {
            snapshot.moba_hud = hud;
            snapshot.owner_economy = economy;
            snapshot.shop_receipts = self.shop_receipts.project(view_epoch);
            self.damage.lock().expect("damage retention poisoned").project(snapshot);
        }
        envelope
    }

    pub fn retain_effects(&self, view_epoch: u64, tick: u64, effects: Vec<omoba_core::game_proto::PresentationEffect>) {
        self.damage.lock().expect("damage retention poisoned").capture(view_epoch, tick, effects);
    }

    pub fn retain_shop_receipts(&mut self, view_epoch: u64, tick: u64, events: &[omoba_core::game_proto::TeamPublicEvent]) {
        for event in events {
            if event.event_kind != omoba_core::runtime::FactKind::ShopReceipt as u32 || event.subject.is_some() { continue; }
            if let Some(receipt) = omoba_core::runtime::shop_receipt::ShopReceipt::decode(&event.sanitized_payload) {
                if receipt.player_id == self.player_id && receipt.tick == tick {
                    if let Ok(id) = u32::try_from(receipt.input_id) { self.shop_pending.finish(id); }
                }
            }
        }
        self.shop_receipts.capture(view_epoch, tick, self.player_id, events);
    }

    pub fn set_shop_protocol_enabled(&mut self, enabled: bool) { self.shop_protocol_enabled = enabled; }
    pub fn set_recall_protocol_enabled(&mut self, enabled: bool) { self.recall_protocol_enabled = enabled; }
    pub fn can_track_shop(&self) -> bool { self.shop_pending.can_track() }
    pub fn track_shop(&mut self, id: u32) { self.shop_pending.track(id, self.shop_clock.elapsed().as_millis() as u64); }
    pub fn due_shop_queries(&mut self) -> Vec<u32> { self.shop_pending.due(self.shop_clock.elapsed().as_millis() as u64) }
    pub fn take_shop_smoke_query(&mut self) -> Option<u32> { self.shop_smoke_query.take() }

    /// Opt-in network fixture: only disclosed state and ordinary player intents.
    pub fn inject_combat_smoke(&mut self, snapshot: &FilteredRenderSnapshot, epoch: u64)
        -> Result<(), ClientRuntimeError> {
        use omoba_core::game_proto::{renderer_input::Intent, AttackTargetIntent, MoveToIntent};
        let Some(hud) = crate::moba_hud::project(snapshot, self.player_id) else { return Ok(()); };
        if let Some(score) = &hud.score {
            let value = (score.kills, score.deaths, score.assists);
            if self.combat_smoke_score != Some(value) {
                log::info!("combat smoke score player={} tick={} kills={} deaths={} assists={}",
                    self.player_id, snapshot.replica_tick, value.0, value.1, value.2);
                self.combat_smoke_score = Some(value);
            }
        }
        let Some(hero) = &hud.hero else { return Ok(()); };
        if hud.phase != 1 || snapshot.replica_tick < 360 { return Ok(()); }
        let decode = |e: &omoba_core::runtime::FilteredRenderEntity| {
            e.components.get(&omoba_core::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
                .and_then(|bytes| decode_demo_render_state(bytes))
        };
        let Some(pose) = snapshot.entities.iter().find(|e| e.replica_id == hero.render_id).and_then(decode)
            else { return Ok(()); };
        let intent = match self.combat_smoke_stage {
            0 => Some(Intent::MoveTo(MoveToIntent { x_raw: hud.lane_length_raw / 2,
                y_raw: 900 * 1024, queued: false })),
            1 if self.player_id != 2 && (pose.x_raw-hud.lane_length_raw/2).abs() <= 75*1024
                && (pose.y_raw-900*1024).abs() <= 75*1024 => {
                snapshot.entities.iter().find(|e| decode(e).is_some_and(|target|
                    target.kind == 1 && target.team_id != snapshot.team_id && target.owner_player_id == 2))
                    .map(|target| Intent::AttackTarget(AttackTargetIntent {
                        target_render_id: target.replica_id, queued: false }))
            }
            _ => None,
        };
        if let Some(intent) = intent {
            let request_id = 0x434f_0000 + u64::from(self.combat_smoke_stage);
            self.inject_test_input(RendererInput { request_id, player_id: self.player_id,
                disclosure_epoch: epoch, intent: Some(intent) })?;
            log::info!("combat smoke submitted player={} stage={} request={} tick={}",
                self.player_id, self.combat_smoke_stage, request_id, snapshot.replica_tick);
            self.combat_smoke_stage += 1;
        }
        Ok(())
    }

    /// Explicit test-mode renderer intents, never writes simulation state.
    pub fn inject_first_learn_smoke(&mut self, snapshot: &FilteredRenderSnapshot, epoch: u64)
        -> Result<(), ClientRuntimeError> {
        use omoba_core::game_proto::{renderer_input::Intent, AbilityCastIntent, AbilityUpgradeIntent};
        let Some(hud) = crate::moba_hud::project(snapshot,self.player_id) else { return Ok(()); };
        let Some(hero) = &hud.hero else { return Ok(()); };
        if hud.phase != 1 || hero.hp_raw <= 0 || hero.abilities.len() != 4 { return Ok(()); }
        let player_id = self.player_id;
        let request = |ordinal:u64, intent| RendererInput { request_id:0x464c_0000u64+ordinal,
            player_id,disclosure_epoch:epoch,intent:Some(intent) };
        let cast = || Intent::AbilityCast(AbilityCastIntent {ability_index:3,target_render_id:0,x_raw:0,y_raw:0});
        match self.first_learn_smoke_stage {
            0 => {
                if hero.level != 1 || hero.skill_points != 1 || hero.abilities.iter().any(|a|a.level != 0) {
                    return Err(ClientRuntimeError::Ipc("first learn fixture requires Lua rank-zero birth and one point".into()));
                }
                self.inject_test_input(request(0,cast()))?;
                self.inject_test_input(request(1,Intent::AbilityUpgrade(AbilityUpgradeIntent {ability_index:2})))?;
                self.first_learn_smoke_tick=snapshot.replica_tick;
                self.first_learn_smoke_stage=1;
                log::info!("first learn smoke rejected-candidates player={} tick={}",self.player_id,snapshot.replica_tick);
            }
            1 if snapshot.replica_tick >= self.first_learn_smoke_tick+60 => {
                if hero.skill_points != 1 || hero.abilities.iter().any(|a|a.level != 0 || a.cooldown_raw != 0) {
                    return Err(ClientRuntimeError::Ipc("unlearned cast or level-gated upgrade changed birth state".into()));
                }
                let input=request(2,Intent::AbilityUpgrade(AbilityUpgradeIntent {ability_index:3}));
                self.inject_test_input(input.clone())?; self.inject_test_input(input)?;
                self.first_learn_smoke_stage=2;
                log::info!("first learn smoke submitted player={} tick={} rank=0 points=1",self.player_id,snapshot.replica_tick);
            }
            2 if hero.abilities[3].level == 1 => {
                if hero.skill_points != 0 {return Err(ClientRuntimeError::Ipc("first learning did not spend exactly one point".into()));}
                self.inject_test_input(request(3,cast()))?;
                self.first_learn_smoke_stage=3;
                log::info!("first learn smoke learned player={} tick={} rank=1 points=0",self.player_id,snapshot.replica_tick);
            }
            3 if hero.abilities[3].cooldown_raw > 0 => {
                log::info!("first learn smoke completed player={} tick={} rank=1 points={} cooldown={}",
                    self.player_id,snapshot.replica_tick,hero.skill_points,hero.abilities[3].cooldown_raw);
                self.first_learn_smoke_stage=4;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn inject_upgrade_smoke(&mut self, snapshot: &FilteredRenderSnapshot, epoch: u64)
        -> Result<(), ClientRuntimeError> {
        if self.player_id == 2 { return Ok(()); }
        let Some(hud) = crate::moba_hud::project(snapshot, self.player_id) else { return Ok(()); };
        let Some(hero) = &hud.hero else { return Ok(()); };
        let Some(ability) = hero.abilities.first() else { return Ok(()); };
        if hud.phase != 1 || hero.hp_raw <= 0 { return Ok(()); }
        match self.upgrade_smoke_stage {
            0 if hero.skill_points > 0 && ability.level == 1
                && hud.score.as_ref().is_some_and(|s| s.kills + s.assists > 0) => {
                let request = RendererInput {
                    request_id: 0x5550_0000 + u64::from(self.player_id), player_id: self.player_id,
                    disclosure_epoch: epoch,
                    intent: Some(omoba_core::game_proto::renderer_input::Intent::AbilityUpgrade(
                        omoba_core::game_proto::AbilityUpgradeIntent { ability_index: 0 })),
                };
                // Same renderer request must not allocate a second authority input.
                self.inject_test_input(request.clone())?;
                self.inject_test_input(request)?;
                log::info!("upgrade smoke submitted player={} tick={} rank={} points={}", self.player_id, snapshot.replica_tick, ability.level, hero.skill_points);
                self.upgrade_smoke_stage = 1;
            }
            1 if ability.level == 2 => {
                log::info!("upgrade smoke completed player={} tick={} rank={} points={}", self.player_id, snapshot.replica_tick, ability.level, hero.skill_points);
                self.upgrade_smoke_stage = 2;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn inject_recall_smoke(&mut self, snapshot: &FilteredRenderSnapshot, epoch: u64, after_tick: u64)
        -> Result<(), ClientRuntimeError> {
        let Some(hud) = crate::moba_hud::project(snapshot, self.player_id) else { return Ok(()); };
        let Some(hero) = &hud.hero else { return Ok(()); };
        let Some(pose) = snapshot.entities.iter().find(|e| e.replica_id == hero.render_id)
            .and_then(|e| e.components.get(&omoba_core::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID))
            .and_then(|bytes| decode_demo_render_state(bytes)) else { return Ok(()); };
        match self.recall_smoke_stage {
            0 if snapshot.replica_tick >= after_tick && hud.phase == 1 && self.recall_protocol_enabled => {
                self.inject_test_input(RendererInput {
                    request_id: 0x5243_0000 + u64::from(self.player_id), player_id: self.player_id,
                    disclosure_epoch: epoch,
                    intent: Some(omoba_core::game_proto::renderer_input::Intent::Recall(omoba_core::game_proto::RecallIntent {})),
                })?;
                log::info!("recall smoke submitted player={} tick={} x={} y={}", self.player_id, snapshot.replica_tick, pose.x_raw, pose.y_raw);
                self.recall_smoke_stage = 1;
            }
            1 if hud.recall_remaining_raw > 0 => {
                log::info!("recall smoke active player={} tick={} remaining={}", self.player_id, snapshot.replica_tick, hud.recall_remaining_raw);
                self.recall_smoke_stage = 2;
            }
            2 if hud.recall_remaining_raw == 0 => {
                let base_x = if snapshot.team_id == 1 { 0 } else { hud.lane_length_raw };
                if hud.phase != 1 || pose.x_raw != base_x || pose.y_raw != 0 {
                    // A legitimate interruption is not a corrupt ordered frame.
                    // Record failure for the external verifier, never trigger rebase.
                    log::warn!("recall smoke canceled player={} tick={} x={} y={} hp={}", self.player_id, snapshot.replica_tick, pose.x_raw, pose.y_raw, hero.hp_raw);
                    self.recall_smoke_stage = 255;
                    return Ok(());
                }
                log::info!("recall smoke completed player={} tick={} x={} y={} base_x={}", self.player_id, snapshot.replica_tick, pose.x_raw, pose.y_raw, base_x);
                self.recall_smoke_stage = 3;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn inject_shop_smoke(&mut self, snapshot: &FilteredRenderSnapshot, epoch: u64) -> Result<(), ClientRuntimeError> {
        use omoba_core::game_proto::{renderer_input::Intent, ItemBuyIntent, ItemSellIntent};
        let Some(economy) = crate::shop_presentation::economy(snapshot, self.player_id) else { return Ok(()); };
        let receipts = self.shop_receipts.project(epoch);
        let intent = match self.shop_smoke_stage {
            0 if economy.shop_available => Some(Intent::ItemBuy(ItemBuyIntent { catalog_id: 1 })),
            1 if receipts.iter().any(|r| r.result_code == 6) && economy.gold >= 350 && economy.shop_available =>
                Some(Intent::ItemBuy(ItemBuyIntent { catalog_id: 1 })),
            2 if economy.slots.iter().any(|s| s.catalog_id == 1) && receipts.iter().any(|r| r.action_kind == 17 && r.result_code == 0) =>
                Some(Intent::ItemSell(ItemSellIntent { item_slot: 0 })),
            _ => None,
        };
        if let Some(intent) = intent {
            if self.shop_smoke_stage == 2 {
                self.shop_smoke_query = receipts.iter().find(|r| r.action_kind == 17 && r.result_code == 0)
                    .and_then(|r| u32::try_from(r.input_id).ok());
            }
            self.inject_test_input(RendererInput { request_id: 0x5348_0000 + u64::from(self.shop_smoke_stage),
                player_id: self.player_id, disclosure_epoch: epoch, intent: Some(intent) })?;
            self.shop_smoke_stage += 1;
        }
        Ok(())
    }

    pub fn retain_shop_replay(&mut self, view_epoch: u64, reply: &omoba_core::game_proto::ShopReceiptReplay)
        -> Result<bool, &'static str> {
        omoba_core::runtime::shop_transport::validate_receipt_replay(reply, self.player_id)?;
        if matches!(reply.status, 2 | 3 | 4) { self.shop_pending.finish(reply.input_id); }
        let Some(receipt) = omoba_core::runtime::shop_transport::validate_receipt_replay(reply, self.player_id)? else { return Ok(false); };
        self.retain_shop_receipts(view_epoch, receipt.tick, &[omoba_core::game_proto::TeamPublicEvent {
            event_kind: omoba_core::runtime::FactKind::ShopReceipt as u32,
            sanitized_payload: receipt.encode(), ..Default::default()
        }]);
        Ok(true)
    }

    pub async fn publish_critical(
        &self,
        envelope: RendererIpcEnvelope,
    ) -> Result<(), ClientRuntimeError> {
        if let Some(renderer_ipc_envelope::Payload::Lifecycle(batch)) = &envelope.payload {
            let mut damage = self.damage.lock().expect("damage retention poisoned");
            for event in &batch.events {
                match &event.action {
                    Some(render_lifecycle_event::Action::ResetView(_)) => damage.reset(),
                    Some(render_lifecycle_event::Action::Hide(_)) |
                    Some(render_lifecycle_event::Action::Forget(_)) => damage.invalidate(event.replica_id, event.disclosure_epoch),
                    None => {}
                }
            }
        }
        self.critical_tx
            .send(envelope)
            .await
            .map_err(|_| ClientRuntimeError::Ipc("critical presentation queue closed".into()))
    }

    pub async fn recv_input(&mut self) -> Option<RendererInput> {
        self.input_rx.recv().await
    }
    pub fn has_pending_input(&self) -> bool {
        !self.input_rx.is_empty()
    }
    pub fn inject_test_input(&self, input: RendererInput) -> Result<(), ClientRuntimeError> {
        self.input_tx
            .try_send(input)
            .map_err(|_| ClientRuntimeError::Ipc("test input queue full".into()))
    }
    pub fn presentation_enabled(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
            || now_ms().saturating_sub(self.disconnected_at_ms.load(Ordering::Relaxed)) <= 30_000
    }
}

async fn serve_connections(
    listener: TcpListener,
    player_id: u32,
    team_id: u32,
    latest_rx: watch::Receiver<Option<Arc<RendererIpcEnvelope>>>,
    critical_rx: Arc<Mutex<mpsc::Receiver<RendererIpcEnvelope>>>,
    input_tx: mpsc::Sender<RendererInput>,
    connected: Arc<AtomicBool>,
    disconnected_at_ms: Arc<AtomicU64>,
    damage: SharedCues,
    ready: SharedReady,
) {
    loop {
        let Ok((stream, peer)) = listener.accept().await else {
            break;
        };
        if !peer.ip().is_loopback() {
            continue;
        }
        let _ = stream.set_nodelay(true);
        let latest = latest_rx.clone();
        let critical = Arc::clone(&critical_rx);
        let inputs = input_tx.clone();
        let connection_flag = connected.clone();
        let disconnected = disconnected_at_ms.clone();
        let retained = damage.clone();
        let retained_ready = ready.clone();
        tokio::spawn(async move {
            if let Err(error) = serve_renderer_retained(
                stream,
                player_id,
                team_id,
                latest,
                critical,
                inputs,
                connection_flag,
                disconnected,
                Some(retained),
                Some(retained_ready),
            )
            .await
            {
                log::warn!("renderer IPC disconnected: {error}");
            }
        });
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

struct RendererLease {
    connected: Arc<AtomicBool>,
    disconnected_at_ms: Arc<AtomicU64>,
}

impl Drop for RendererLease {
    fn drop(&mut self) {
        self.disconnected_at_ms.store(now_ms(), Ordering::Relaxed);
        self.connected.store(false, Ordering::Release);
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
async fn serve_renderer(
    stream: TcpStream,
    player_id: u32,
    team_id: u32,
    latest_rx: watch::Receiver<Option<Arc<RendererIpcEnvelope>>>,
    critical_rx: Arc<Mutex<mpsc::Receiver<RendererIpcEnvelope>>>,
    input_tx: mpsc::Sender<RendererInput>,
    connected: Arc<AtomicBool>,
    disconnected_at_ms: Arc<AtomicU64>,
) -> Result<(), ClientRuntimeError> {
    serve_renderer_retained(stream, player_id, team_id, latest_rx, critical_rx, input_tx,
        connected, disconnected_at_ms, None, None).await
}

#[allow(clippy::too_many_arguments)]
async fn serve_renderer_retained(
    stream: TcpStream, player_id: u32, team_id: u32,
    mut latest_rx: watch::Receiver<Option<Arc<RendererIpcEnvelope>>>,
    critical_rx: Arc<Mutex<mpsc::Receiver<RendererIpcEnvelope>>>,
    input_tx: mpsc::Sender<RendererInput>, connected: Arc<AtomicBool>,
    disconnected_at_ms: Arc<AtomicU64>, damage: Option<SharedCues>,
    ready_metadata: Option<SharedReady>,
) -> Result<(), ClientRuntimeError> {
    let connection = RENDERER_CONNECTION.fetch_add(1, Ordering::Relaxed);
    let (mut reader, mut writer) = stream.into_split();
    let (first, handshake_wire) = tokio::time::timeout(Duration::from_secs(5), read_envelope_measured(&mut reader))
        .await
        .map_err(|_| ClientRuntimeError::Ipc("renderer handshake timed out".into()))??;
    let Some(renderer_ipc_envelope::Payload::RendererReady(ready)) = first.payload else {
        return Err(ClientRuntimeError::Ipc(
            "renderer handshake required".into(),
        ));
    };
    validate_renderer_ready(&ready, player_id, team_id)?;
    connected
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map_err(|_| ClientRuntimeError::Ipc("renderer already connected".into()))?;
    let _lease = RendererLease {
        connected,
        disconnected_at_ms,
    };
    let meter = Arc::new(Mutex::new(PresentationIpcMeter::new(player_id, team_id, connection)));
    log::info!("{}", meter.lock().await.enable_line());
    if let Some(line) = meter.lock().await.observe_receive(handshake_wire) {
        log::info!("{line}");
    }
    let initial = { latest_rx.borrow_and_update().clone() };
    // Startup/reconnect metadata is persistent, unlike replaceable snapshots.
    // Send it only after the renderer's player/team handshake is validated.
    let retained_ready = ready_metadata.and_then(|ready| ready.lock().expect("ready retention poisoned").clone());
    if !matches!(initial.as_ref().and_then(|frame| frame.payload.as_ref()), Some(renderer_ipc_envelope::Payload::RuntimeReady(_))) {
        if let Some(ready) = retained_ready { write_measured(&mut writer, &ready, meter.as_ref()).await?; }
    }
    let last_sent_snapshot = AtomicU64::new(0);
    let last_consumed_snapshot = AtomicU64::new(0);
    let mut covered_snapshot = 0;
    let mut has_baseline = false;
    let mut baseline_tick = 0;
    if let Some(latest) = initial {
        write_measured(&mut writer, &renderer_baseline(&latest), meter.as_ref()).await?;
        if matches!(
            latest.payload,
            Some(renderer_ipc_envelope::Payload::Snapshot(_))
        ) {
            covered_snapshot = latest.sequence;
            last_sent_snapshot.store(latest.sequence, Ordering::Release);
            has_baseline = true;
            if let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = &latest.payload {
                baseline_tick = snapshot.replica_tick;
                if let Some(damage) = &damage { baseline_tick = damage.lock().expect("damage retention poisoned").baseline(baseline_tick); }
            }
        }
    }
    let receive_meter = Arc::clone(&meter);
    let send_meter = Arc::clone(&meter);
    // Keep the read future alive across every outgoing frame. Cancelling
    // read_envelope after it consumes a prefix would corrupt TCP framing.
    let receive = async {
        loop {
            let (envelope, wire_bytes) = read_envelope_measured(&mut reader).await?;
            match envelope.payload {
                Some(renderer_ipc_envelope::Payload::RendererInput(input)) => {
                    if let Some(line) = receive_meter.lock().await.observe_receive(wire_bytes) {
                        log::info!("{line}");
                    }
                    input_tx
                        .send(input)
                        .await
                        .map_err(|_| ClientRuntimeError::Ipc("input bridge closed".into()))?;
                }
                Some(renderer_ipc_envelope::Payload::RendererShutdown(_)) => {
                    if let Some(line) = receive_meter.lock().await.observe_receive(wire_bytes) {
                        log::info!("{line}");
                    }
                    return Ok(());
                }
                Some(renderer_ipc_envelope::Payload::RendererConsumed(consumed)) => {
                    if consumed.snapshot_sequence > last_sent_snapshot.load(Ordering::Acquire) {
                        return Err(ClientRuntimeError::Ipc(
                            "renderer consumed unsent snapshot".into(),
                        ));
                    }
                    // Older leases may finish after newer ones; acknowledgements
                    // are idempotent, never allowed to move the cursor backward.
                    let previous = last_consumed_snapshot.fetch_max(consumed.snapshot_sequence, Ordering::AcqRel);
                    if let Some(damage) = &damage {
                        damage.lock().expect("damage retention poisoned").consumed(connection, consumed.snapshot_sequence);
                    }
                    if previous == 0 && consumed.snapshot_sequence > 0 {
                        log::info!("renderer first consumed snapshot player={} team={} sequence={}",
                            player_id, team_id, consumed.snapshot_sequence);
                    }
                    if let Some(line) = receive_meter.lock().await.observe_receive(wire_bytes) {
                        log::info!("{line}");
                    }
                }
                _ => {
                    return Err(ClientRuntimeError::Ipc(
                        "renderer sent unexpected payload".into(),
                    ))
                }
            }
        }
    };
    let send = async {
        let mut critical_rx = critical_rx.lock().await;
        loop {
            tokio::select! {
                biased;
                critical = critical_rx.recv() => {
                let Some(critical) = critical else { return Ok(()); };
                if state_is_covered(&critical, covered_snapshot) { continue; }
                if !has_baseline {
                    write_measured(&mut writer, &renderer_baseline(&critical), send_meter.as_ref()).await?;
                } else {
                    let outgoing = renderer_retained_frame(&critical, baseline_tick, damage.as_ref());
                    write_measured(&mut writer, &outgoing, send_meter.as_ref()).await?;
                    record_damage_sent(damage.as_ref(), connection, &outgoing);
                }
                if !has_baseline {
                    if let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = &critical.payload {
                        baseline_tick = snapshot.replica_tick;
                        if let Some(damage) = &damage { baseline_tick = damage.lock().expect("damage retention poisoned").baseline(baseline_tick); }
                    }
                }
                if matches!(critical.payload, Some(renderer_ipc_envelope::Payload::Snapshot(_))) {
                    covered_snapshot = critical.sequence;
                    last_sent_snapshot.store(critical.sequence, Ordering::Release);
                    has_baseline = true;
                }
                }
                changed = latest_rx.changed() => {
                changed.map_err(|_| ClientRuntimeError::Ipc("presentation source closed".into()))?;
                let latest = latest_rx.borrow().clone();
                if let Some(latest) = latest {
                    if state_is_covered(&latest, covered_snapshot) { continue; }
                    if !has_baseline {
                        write_measured(&mut writer, &renderer_baseline(&latest), send_meter.as_ref()).await?;
                    } else {
                        let outgoing = renderer_retained_frame(&latest, baseline_tick, damage.as_ref());
                        write_measured(&mut writer, &outgoing, send_meter.as_ref()).await?;
                        record_damage_sent(damage.as_ref(), connection, &outgoing);
                    }
                    if !has_baseline {
                        if let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = &latest.payload {
                            baseline_tick = snapshot.replica_tick;
                            if let Some(damage) = &damage { baseline_tick = damage.lock().expect("damage retention poisoned").baseline(baseline_tick); }
                        }
                    }
                    if matches!(latest.payload, Some(renderer_ipc_envelope::Payload::Snapshot(_))) {
                        covered_snapshot = latest.sequence;
                        last_sent_snapshot.store(latest.sequence, Ordering::Release);
                        has_baseline = true;
                    }
                }
                }
            }
        }
    };
    tokio::select! {
        result = receive => result,
        result = send => result,
    }
}

// A renderer's first full view restores persistent state, not historical
// one-shots. This also applies when the first snapshot arrives after Ready.
// Never mutate the shared latest snapshot used by an already-connected client.
fn renderer_baseline(envelope: &RendererIpcEnvelope) -> RendererIpcEnvelope {
    let mut baseline = envelope.clone();
    if let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = &mut baseline.payload {
        snapshot.effects.clear();
        snapshot.audio_cues.clear();
    }
    baseline
}

fn renderer_retained_frame<'a>(envelope: &'a RendererIpcEnvelope, baseline_tick: u64, damage: Option<&SharedCues>) -> std::borrow::Cow<'a, RendererIpcEnvelope> {
    use omoba_core::runtime::presentation_cue::PresentationCue;
    let retained = damage.map(|damage| damage.lock().expect("damage retention poisoned"));
    let old_damage = |effect: &omoba_core::game_proto::PresentationEffect| {
        PresentationCue::decode(&effect.safe_payload).is_some_and(|cue| cue.tick() <= baseline_tick
            || retained.as_ref().is_some_and(|retained| !retained.contains(effect.effect_id)))
    };
    if let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = &envelope.payload {
        if snapshot.effects.iter().any(old_damage) {
            let mut filtered = envelope.clone();
            if let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = &mut filtered.payload {
                snapshot.effects.retain(|effect| !old_damage(effect));
            }
            return std::borrow::Cow::Owned(filtered);
        }
    }
    std::borrow::Cow::Borrowed(envelope)
}

fn record_damage_sent(damage: Option<&SharedCues>, connection: u64, envelope: &RendererIpcEnvelope) {
    if let (Some(damage), Some(renderer_ipc_envelope::Payload::Snapshot(snapshot))) = (damage, &envelope.payload) {
        damage.lock().expect("damage retention poisoned").sent(connection, envelope.sequence, snapshot);
    }
}

fn state_is_covered(envelope: &RendererIpcEnvelope, snapshot_sequence: u64) -> bool {
    envelope.sequence <= snapshot_sequence
        && matches!(
            envelope.payload,
            Some(renderer_ipc_envelope::Payload::Snapshot(_))
                | Some(renderer_ipc_envelope::Payload::Lifecycle(_))
        )
}

fn validate_renderer_ready(
    ready: &omoba_core::game_proto::RendererReady,
    player_id: u32,
    team_id: u32,
) -> Result<(), ClientRuntimeError> {
    if ready.player_id != player_id || ready.team_id != team_id {
        return Err(ClientRuntimeError::Ipc(
            "renderer player/team mismatch".into(),
        ));
    }
    Ok(())
}

pub async fn read_envelope(
    reader: &mut (impl AsyncReadExt + Unpin),
) -> Result<RendererIpcEnvelope, ClientRuntimeError> {
    read_envelope_measured(reader)
        .await
        .map(|(envelope, _wire_bytes)| envelope)
}

pub async fn read_envelope_measured(
    reader: &mut (impl AsyncReadExt + Unpin),
) -> Result<(RendererIpcEnvelope, u64), ClientRuntimeError> {
    let length = reader
        .read_u32()
        .await
        .map_err(|error| ClientRuntimeError::Ipc(error.to_string()))? as usize;
    if length == 0 || length > MAX_PRESENTATION_FRAME_BYTES {
        return Err(ClientRuntimeError::Ipc(
            "invalid presentation frame length".into(),
        ));
    }
    let mut bytes = vec![0; length];
    reader
        .read_exact(&mut bytes)
        .await
        .map_err(|error| ClientRuntimeError::Ipc(error.to_string()))?;
    let envelope = RendererIpcEnvelope::decode(bytes.as_slice())
        .map_err(|_| ClientRuntimeError::Ipc("invalid presentation protobuf".into()))?;
    if envelope.magic != PRESENTATION_MAGIC
        || envelope.protocol_version != PRESENTATION_PROTOCOL_VERSION
    {
        return Err(ClientRuntimeError::Ipc(
            "presentation protocol mismatch".into(),
        ));
    }
    Ok((envelope, (length as u64).saturating_add(4)))
}

pub async fn write_envelope(
    writer: &mut (impl AsyncWriteExt + Unpin),
    envelope: &RendererIpcEnvelope,
) -> Result<u64, ClientRuntimeError> {
    let bytes = envelope.encode_to_vec();
    if bytes.len() > MAX_PRESENTATION_FRAME_BYTES {
        return Err(ClientRuntimeError::Ipc(
            "presentation frame exceeds limit".into(),
        ));
    }
    writer
        .write_u32(bytes.len() as u32)
        .await
        .map_err(|error| ClientRuntimeError::Ipc(error.to_string()))?;
    writer
        .write_all(&bytes)
        .await
        .map_err(|error| ClientRuntimeError::Ipc(error.to_string()))?;
    Ok((bytes.len() as u64).saturating_add(4))
}

async fn write_measured(
    writer: &mut (impl AsyncWriteExt + Unpin),
    envelope: &RendererIpcEnvelope,
    meter: &Mutex<PresentationIpcMeter>,
) -> Result<(), ClientRuntimeError> {
    let wire_bytes = write_envelope(writer, envelope).await?;
    if let Some(line) = meter.lock().await.observe_send(wire_bytes) {
        log::info!("{line}");
    }
    Ok(())
}

pub fn ready_envelope(
    sequence: u64,
    config: &ClientRuntimeConfig,
    server_tick: u64,
    replica_tick: u64,
    tick_rate_hz: u32,
) -> RendererIpcEnvelope {
    envelope(
        sequence,
        renderer_ipc_envelope::Payload::RuntimeReady(RuntimeReadyPresentation {
            player_id: config.player_id,
            team_id: config.team_id,
            authoritative_tick: server_tick,
            replica_tick,
            content_hash: config.content_hash.clone(),
            tick_rate_hz,
            ..Default::default()
        }),
    )
}

pub fn ready_envelope_for_start(sequence: u64, config: &ClientRuntimeConfig,
    start: &omoba_core::game_proto::TeamGameStart) -> Result<RendererIpcEnvelope, ClientRuntimeError> {
    let map = omoba_core::runtime::moba_map_layout::validate_metadata(&start.public_metadata)
        .map_err(|error| ClientRuntimeError::Replica(error.into()))?;
    let mut ready = ready_envelope(sequence,config,start.server_tick,start.replica_start_tick,start.tick_rate_hz);
    if let Some(map) = map {
        if let Some(renderer_ipc_envelope::Payload::RuntimeReady(metadata)) = &mut ready.payload {
            metadata.moba_map_id = map.id.into();
            metadata.moba_map_catalog_hash = omoba_template_ids::MOBA_MAP_CATALOG_HASH.into();
        }
    }
    Ok(ready)
}

pub fn snapshot_envelope(
    sequence: u64,
    authoritative_tick: u64,
    view_epoch: u64,
    snapshot: FilteredRenderSnapshot,
    runtime_rtt_us: u64,
) -> RendererIpcEnvelope {
    snapshot_envelope_cached(
        sequence,
        authoritative_tick,
        view_epoch,
        snapshot,
        runtime_rtt_us,
        &mut DemoFogCache::default(),
    )
}

fn snapshot_envelope_cached(
    sequence: u64,
    authoritative_tick: u64,
    view_epoch: u64,
    snapshot: FilteredRenderSnapshot,
    runtime_rtt_us: u64,
    fog_cache: &mut DemoFogCache,
) -> RendererIpcEnvelope {
    let mut fog = omoba_core::runtime::fog_grid::FogGridRetention::default();
    let fog_grid = fog.ingest_events(&snapshot.public_events, snapshot.team_id, view_epoch, snapshot.replica_tick)
        .ok().and_then(|_| fog.latest().map(|grid| grid.to_presentation()));
    let formal_moba = snapshot.public_events.iter().any(|event| {
        let p = &event.sanitized_payload;
        event.event_kind == omoba_core::runtime::fog_grid::FOG_GRID_EVENT_KIND
            || (event.event_kind == omoba_core::runtime::FactKind::Hud as u32 && p.len() == 20
            && p[..4] == 0_u32.to_le_bytes()
            && p[4..12] == omoba_core::runtime::SINGLE_LANE_PHASE_METRIC_ID.to_le_bytes())
    });
    let effects = snapshot.external_effects.iter().filter_map(|effect| {
        use omoba_core::runtime::presentation_cue::{DamagePresentationCue, presentation_effect_id};
        if effect.effect_kind != omoba_core::runtime::FactKind::DirectCombat as u32 { return None; }
        let target_id = effect.visible_target.as_ref()?.value;
        let target = snapshot.entities.iter().find(|entity| entity.replica_id == target_id)?;
        if target_id == 0 || target.disclosure_epoch == 0 { return None; }
        let amount_milli = i64::from_le_bytes(effect.sanitized_payload.get(..8)?.try_into().ok()?);
        if amount_milli <= 0 { return None; }
        Some(omoba_core::game_proto::PresentationEffect {
            effect_id: presentation_effect_id(snapshot.replica_tick, effect.stable_sub_index)?,
            safe_payload: DamagePresentationCue {tick: snapshot.replica_tick, target_id,
                disclosure_epoch: target.disclosure_epoch, amount_milli}.encode(),
        })
    }).chain(snapshot.public_events.iter().filter_map(|event| {
        use omoba_core::runtime::presentation_cue::PresentationCue;
        let (effect_id, cue) = PresentationCue::from_public_event(snapshot.replica_tick, event, |id| {
            snapshot.entities.iter().find(|entity| entity.replica_id == id).map(|entity| entity.disclosure_epoch)
        })?;
        Some(omoba_core::game_proto::PresentationEffect {effect_id, safe_payload: cue.encode()})
    })).collect();
    let entities: Vec<_> = snapshot
        .entities
        .into_iter()
        .map(|entity| PresentationRenderEntity {
            render_id: entity.replica_id,
            disclosure_epoch: entity.disclosure_epoch,
            entity_kind: entity.entity_kind,
            components: entity
                .components
                .into_iter()
                .map(|(schema_id, safe_payload)| PresentationComponent {
                    schema_id,
                    safe_payload,
                })
                .collect(),
        })
        .collect();
    let mut digest = Sha256::new();
    for entity in &entities {
        digest.update(entity.render_id.to_be_bytes());
        digest.update(entity.disclosure_epoch.to_be_bytes());
    }
    let visibility_digest =
        u64::from_be_bytes(digest.finalize()[..8].try_into().expect("digest prefix"));
    let (fog_tiles, vision_circles, tree_occluders, polygon_occluders) =
        fog_cache.derive_for_mode(&entities, snapshot.team_id, formal_moba);
    envelope(
        sequence,
        renderer_ipc_envelope::Payload::Snapshot(TeamPresentationSnapshot {
            team_id: snapshot.team_id,
            authoritative_tick,
            replica_tick: snapshot.replica_tick,
            visibility_digest,
            entities,
            removed_render_ids: Vec::new(),
            remembered_ghosts: snapshot
                .remembered_presentations
                .into_iter()
                .filter(|(_, presentation)| !presentation.is_empty())
                .map(|((render_id, disclosure_epoch), sanitized_presentation)| {
                    RememberedGhostPresentation {
                        render_id,
                        disclosure_epoch,
                        sanitized_presentation,
                    }
                })
                .collect(),
            fog_tiles,
            vision_circles,
            tree_occluders,
            polygon_occluders,
            effects,
            audio_cues: Vec::new(),
            view_epoch,
            runtime_rtt_us,
            moba_hud: None,
            owner_economy: None,
            shop_receipts: Vec::new(),
            fog_grid,
        }),
    )
}

pub fn lifecycle_envelope(
    sequence: u64,
    team_id: u32,
    authoritative_tick: u64,
    replica_tick: u64,
    view_epoch: u64,
    directives: Vec<RenderMemoryDirective>,
) -> Option<RendererIpcEnvelope> {
    let events = directives
        .into_iter()
        .map(render_lifecycle_event)
        .collect::<Vec<_>>();
    (!events.is_empty()).then(|| {
        envelope(
            sequence,
            renderer_ipc_envelope::Payload::Lifecycle(RenderLifecycleBatch {
                team_id,
                authoritative_tick,
                replica_tick,
                view_epoch,
                events,
            }),
        )
    })
}

pub fn reset_view_envelope(
    sequence: u64,
    team_id: u32,
    authoritative_tick: u64,
    replica_tick: u64,
    view_epoch: u64,
) -> RendererIpcEnvelope {
    envelope(
        sequence,
        renderer_ipc_envelope::Payload::Lifecycle(RenderLifecycleBatch {
            team_id,
            authoritative_tick,
            replica_tick,
            view_epoch,
            events: vec![RenderLifecycleEvent {
                replica_id: 0,
                disclosure_epoch: 0,
                action: Some(render_lifecycle_event::Action::ResetView(
                    RenderLifecycleResetView {},
                )),
            }],
        }),
    )
}

fn render_lifecycle_event(directive: RenderMemoryDirective) -> RenderLifecycleEvent {
    match directive {
        RenderMemoryDirective::Hide {
            replica_id,
            disclosure_epoch,
            remember_policy,
            sanitized_presentation,
        } => RenderLifecycleEvent {
            replica_id,
            disclosure_epoch,
            action: Some(render_lifecycle_event::Action::Hide(RenderLifecycleHide {
                remember_policy,
                sanitized_presentation,
            })),
        },
        RenderMemoryDirective::Forget {
            replica_id,
            disclosure_epoch,
        } => RenderLifecycleEvent {
            replica_id,
            disclosure_epoch,
            action: Some(render_lifecycle_event::Action::Forget(
                RenderLifecycleForget {},
            )),
        },
    }
}

type DemoFog = (
    Vec<FogTilePresentation>,
    Vec<VisionCirclePresentation>,
    Vec<TreeOccluderPresentation>,
    Vec<PolygonOccluderPresentation>,
);

#[derive(Default)]
struct DemoFogCache {
    formal_moba_seen: bool,
    latest: Option<(u32, Vec<(i64, i64)>, DemoFog)>,
    #[cfg(test)]
    rebuilds: usize,
}

impl DemoFogCache {
    fn derive_for_mode(&mut self, entities: &[PresentationRenderEntity], team_id: u32, formal_moba: bool) -> DemoFog {
        self.formal_moba_seen |= formal_moba;
        if self.formal_moba_seen {
            // No authority grid/exploration contract yet: never substitute demo
            // visibility for formal gameplay fog, including after view reset.
            self.latest = None;
            return Default::default();
        }
        self.derive(entities, team_id)
    }
    fn derive(&mut self, entities: &[PresentationRenderEntity], team_id: u32) -> DemoFog {
        let centers = demo_visible_centers(entities, team_id);
        if let Some((cached_team, cached_centers, fog)) = &self.latest {
            if *cached_team == team_id && *cached_centers == centers {
                return fog.clone();
            }
        }
        let fog = derive_demo_fog_at_centers(centers.clone());
        #[cfg(test)]
        {
            self.rebuilds += 1;
        }
        self.latest = Some((team_id, centers, fog.clone()));
        fog
    }
}

#[cfg(test)]
fn derive_demo_fog(entities: &[PresentationRenderEntity], team_id: u32) -> DemoFog {
    derive_demo_fog_at_centers(demo_visible_centers(entities, team_id))
}

fn demo_visible_centers(entities: &[PresentationRenderEntity], team_id: u32) -> Vec<(i64, i64)> {
    let mut visible_centers = Vec::new();
    for entity in entities {
        let Some(component) = entity
            .components
            .iter()
            .find(|value| value.schema_id == omoba_core::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
        else {
            continue;
        };
        if let Some(render) = decode_demo_render_state(&component.safe_payload) {
            if render.team_id == team_id && render.kind == 1 {
                visible_centers.push((render.x_raw, render.y_raw));
            }
        }
    }
    visible_centers
}

fn derive_demo_fog_at_centers(visible_centers: Vec<(i64, i64)>) -> DemoFog {
    let tile_raw = 10_i64 * 1024;
    let vision_raw = 700_i64 * 1024;
    let vision_squared = i128::from(vision_raw) * i128::from(vision_raw);
    let (trees, polygons) = demo_occluders();
    let mut tiles = Vec::new();
    for &(cx, cy) in &visible_centers {
        let center_col = (cx.div_euclid(tile_raw)) as i32;
        let center_row = (cy.div_euclid(tile_raw)) as i32;
        for row in (center_row - 70)..=(center_row + 70) {
            for column in (center_col - 70)..=(center_col + 70) {
                let x = i64::from(column) * tile_raw + tile_raw / 2;
                let y = i64::from(row) * tile_raw + tile_raw / 2;
                let dx = i128::from(x - cx);
                let dy = i128::from(y - cy);
                if dx * dx + dy * dy <= vision_squared
                    && !demo_segment_blocked((cx, cy), (x, y), &trees, &polygons)
                {
                    tiles.push(FogTilePresentation {
                        column,
                        row,
                        visible: true,
                    });
                }
            }
        }
    }
    tiles.sort_by_key(|tile| (tile.row, tile.column));
    tiles.dedup_by_key(|tile| (tile.row, tile.column));
    let circles = visible_centers
        .into_iter()
        .map(|(x_raw, y_raw)| VisionCirclePresentation {
            x_raw,
            y_raw,
            radius_raw: vision_raw,
        })
        .collect();
    let tree_messages = trees
        .iter()
        .map(|&(x_raw, y_raw, radius_raw)| TreeOccluderPresentation {
            x_raw,
            y_raw,
            radius_raw,
        })
        .collect();
    let polygon_messages = polygons
        .iter()
        .map(|points| PolygonOccluderPresentation {
            points: points
                .iter()
                .map(|&(x_raw, y_raw)| PolygonPointPresentation { x_raw, y_raw })
                .collect(),
        })
        .collect();
    (tiles, circles, tree_messages, polygon_messages)
}

fn demo_occluders() -> (Vec<(i64, i64, i64)>, Vec<Vec<(i64, i64)>>) {
    let scale = 1024_i64;
    let mut trees = Vec::with_capacity(64);
    for row in 0..8 {
        for column in 0..8 {
            let id = row * 8 + column + 1;
            trees.push((
                (-875 + column * 250) * scale,
                (-875 + row * 250) * scale,
                (if id % 3 == 0 { 82 } else { 62 }) * scale,
            ));
        }
    }
    let poly = |points: &[(i64, i64)]| {
        points
            .iter()
            .map(|&(x, y)| (x * scale, y * scale))
            .collect()
    };
    let polygons = vec![
        poly(&[(-170, -150), (190, -150), (190, 130), (-170, 130)]),
        poly(&[
            (-980, 180),
            (-560, 180),
            (-560, 520),
            (-720, 520),
            (-720, 340),
            (-980, 340),
        ]),
        poly(&[
            (520, -560),
            (980, -560),
            (980, -360),
            (700, -360),
            (700, -160),
            (520, -160),
        ]),
    ];
    (trees, polygons)
}

fn demo_segment_blocked(
    a: (i64, i64),
    b: (i64, i64),
    trees: &[(i64, i64, i64)],
    polygons: &[Vec<(i64, i64)>],
) -> bool {
    let (ax, ay) = (a.0 as f64, a.1 as f64);
    let (bx, by) = (b.0 as f64, b.1 as f64);
    let dx = bx - ax;
    let dy = by - ay;
    let len2 = dx * dx + dy * dy;
    if trees.iter().any(|&(x, y, r)| {
        let t = if len2 == 0.0 {
            0.0
        } else {
            (((x as f64 - ax) * dx + (y as f64 - ay) * dy) / len2).clamp(0.0, 1.0)
        };
        let ex = ax + t * dx - x as f64;
        let ey = ay + t * dy - y as f64;
        ex * ex + ey * ey <= (r as f64) * (r as f64)
    }) {
        return true;
    }
    polygons.iter().any(|points| {
        points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(points.len())
            .any(|(&p, &q)| segments_intersect(a, b, p, q))
    })
}

fn segments_intersect(a: (i64, i64), b: (i64, i64), c: (i64, i64), d: (i64, i64)) -> bool {
    fn cross(a: (i64, i64), b: (i64, i64), c: (i64, i64)) -> i128 {
        i128::from(b.0 - a.0) * i128::from(c.1 - a.1)
            - i128::from(b.1 - a.1) * i128::from(c.0 - a.0)
    }
    let (ab_c, ab_d, cd_a, cd_b) = (
        cross(a, b, c),
        cross(a, b, d),
        cross(c, d, a),
        cross(c, d, b),
    );
    (ab_c == 0 || ab_d == 0 || ab_c.signum() != ab_d.signum())
        && (cd_a == 0 || cd_b == 0 || cd_a.signum() != cd_b.signum())
}

fn envelope(sequence: u64, payload: renderer_ipc_envelope::Payload) -> RendererIpcEnvelope {
    RendererIpcEnvelope {
        magic: PRESENTATION_MAGIC,
        protocol_version: PRESENTATION_PROTOCOL_VERSION,
        sequence,
        payload: Some(payload),
    }
}

pub fn cadence_period(hz: u32) -> Duration {
    Duration::from_nanos(1_000_000_000 / u64::from(hz.max(1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ready_roundtrip_preserves_authority_rate_not_presentation_cadence() {
        use prost::Message;
        let config = ClientRuntimeConfig::parse([
            "--player-id", "7", "--team-id", "2", "--server", "127.0.0.1:50061",
            "--presentation-bind", "127.0.0.1:50161", "--presentation-hz", "30",
        ].into_iter().map(str::to_owned)).unwrap();
        for rate in [60, 90, 120] {
            let encoded = ready_envelope(3, &config, 120, 119, rate).encode_to_vec();
            let decoded = RendererIpcEnvelope::decode(encoded.as_slice()).unwrap();
            let Some(renderer_ipc_envelope::Payload::RuntimeReady(ready)) = decoded.payload else {
                panic!("missing runtime ready");
            };
            assert_eq!((ready.player_id, ready.team_id), (7, 2));
            assert_eq!(ready.tick_rate_hz, rate);
            assert_eq!((ready.authoritative_tick, ready.replica_tick), (120, 119));
        }
    }

    #[tokio::test]
    async fn late_renderer_and_reconnect_receive_retained_ready_before_latest_snapshot() {
        tokio::time::timeout(Duration::from_secs(3), async {
            let reservation = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = reservation.local_addr().unwrap();
            drop(reservation);
            let config = ClientRuntimeConfig::parse([
                "--player-id".to_owned(), "7".to_owned(), "--team-id".to_owned(), "2".to_owned(),
                "--server".to_owned(), "127.0.0.1:50061".to_owned(),
                "--presentation-bind".to_owned(), address.to_string(),
            ]).unwrap();
            let hub = PresentationHub::bind(&config).await.unwrap();
            let map = omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
            let ready = ready_envelope_for_start(1,&config,&omoba_core::game_proto::TeamGameStart {
                server_tick:10,replica_start_tick:10,tick_rate_hz:60,
                public_metadata:omoba_core::runtime::moba_map_layout::bootstrap_metadata(map),..Default::default()
            }).unwrap();
            hub.publish_latest(ready.clone());
            let latest = envelope(10, renderer_ipc_envelope::Payload::Snapshot(TeamPresentationSnapshot {
                team_id: 2, replica_tick: 20, ..Default::default()
            }));
            hub.publish_latest(latest.clone());
            for _ in 0..2 {
                let mut client = TcpStream::connect(address).await.unwrap();
                write_envelope(&mut client, &envelope(0, renderer_ipc_envelope::Payload::RendererReady(
                    omoba_core::game_proto::RendererReady {player_id: 7, team_id: 2, latest_snapshot_sequence: 0}
                ))).await.unwrap();
                assert_eq!(read_envelope(&mut client).await.unwrap(), ready);
                assert_eq!(read_envelope(&mut client).await.unwrap(), renderer_baseline(&latest));
                write_envelope(&mut client, &envelope(0, renderer_ipc_envelope::Payload::RendererShutdown(Default::default()))).await.unwrap();
                while hub.connected.load(Ordering::Acquire) { tokio::task::yield_now().await; }
            }
            let mut wrong_team = TcpStream::connect(address).await.unwrap();
            write_envelope(&mut wrong_team, &envelope(0, renderer_ipc_envelope::Payload::RendererReady(
                omoba_core::game_proto::RendererReady {player_id: 7, team_id: 1, latest_snapshot_sequence: 0}
            ))).await.unwrap();
            assert!(read_envelope(&mut wrong_team).await.is_err(), "wrong team must not receive retained metadata");
        }).await.unwrap();
    }
    use omoba_core::runtime::{
        encode_demo_render_state, DemoRenderState, DEMO_RENDER_COMPONENT_SCHEMA_ID,
    };

    fn hero() -> PresentationRenderEntity {
        PresentationRenderEntity {
            render_id: 7,
            disclosure_epoch: 3,
            entity_kind: 1,
            components: vec![PresentationComponent {
                schema_id: DEMO_RENDER_COMPONENT_SCHEMA_ID,
                safe_payload: encode_demo_render_state(DemoRenderState {
                    x_raw: -1320 * 1024,
                    y_raw: -1100 * 1024,
                    team_id: 1,
                    kind: 1,
                    owner_player_id: 1,
                }),
            }],
        }
    }

    #[test]
    fn fog_cache_reuses_only_exact_safe_centers_and_team() {
        let mut cache = DemoFogCache::default();
        let original = hero();
        let first = cache.derive(&[original.clone()], 1);
        assert_eq!(first, derive_demo_fog(&[original.clone()], 1));
        assert_eq!(first, cache.derive(&[original.clone()], 1));
        assert_eq!(cache.rebuilds, 1);
        let mut moved = original.clone();
        moved.components[0].safe_payload = encode_demo_render_state(DemoRenderState {
            x_raw: -1320 * 1024 + 1,
            y_raw: -1100 * 1024,
            team_id: 1,
            kind: 1,
            owner_player_id: 1,
        });
        let changed = cache.derive(&[moved.clone()], 1);
        assert_eq!(changed, derive_demo_fog(&[moved], 1));
        assert_eq!(
            cache.rebuilds, 2,
            "even a sub-cell movement invalidates exact fog"
        );
        assert!(cache.derive(&[original], 2).0.is_empty());
        assert!(cache.derive(&[], 1).0.is_empty());
        assert_eq!(
            cache.rebuilds, 4,
            "team change and reset must invalidate fog"
        );
    }

    #[test]
    fn formal_moba_never_falls_back_to_demo_fog() {
        let mut cache = DemoFogCache::default();
        let entities = vec![hero()];
        assert!(!cache.derive_for_mode(&entities, 1, false).0.is_empty());
        let rebuilds = cache.rebuilds;
        let formal = cache.derive_for_mode(&entities, 1, true);
        assert!(formal.0.is_empty() && formal.1.is_empty() && formal.2.is_empty() && formal.3.is_empty());
        assert!(cache.latest.is_none());
        assert!(cache.derive_for_mode(&entities, 1, false).0.is_empty(), "missing HUD after reset cannot restore demo fog");
        assert_eq!(cache.rebuilds, rebuilds);
        let mut payload = Vec::new();
        payload.extend(0_u32.to_le_bytes());
        payload.extend(omoba_core::runtime::SINGLE_LANE_PHASE_METRIC_ID.to_le_bytes());
        payload.extend(1_i64.to_le_bytes());
        let source = FilteredRenderSnapshot { team_id: 1, replica_tick: 5, entities: vec![],
            public_events: vec![omoba_core::game_proto::TeamPublicEvent {
                event_kind: omoba_core::runtime::FactKind::Hud as u32,
                sanitized_payload: payload, ..Default::default()
            }], external_effects: vec![], memory_directives: vec![], remembered_presentations: Default::default() };
        let mut fresh = DemoFogCache::default();
        snapshot_envelope_cached(1, 5, 1, source, 0, &mut fresh);
        assert!(fresh.formal_moba_seen, "production envelope uses authority phase marker");
    }

    #[test]
    fn authority_fog_ipc_is_typed_bound_and_never_demo() {
        use omoba_core::runtime::fog_grid::{FogGridSnapshot, FogGridGeometry};
        let grid = FogGridSnapshot { team: 1, view_epoch: 7, tick: 10,
            geometry: FogGridGeometry { origin_x_raw: -2048, origin_y_raw: 1024,
                cell_size_raw: 1024, columns: 3, rows: 1 }, cells: vec![0,1,2] };
        let source = |events| FilteredRenderSnapshot { team_id: 1, replica_tick: 11,
            entities: vec![], public_events: events, external_effects: vec![],
            memory_directives: vec![], remembered_presentations: Default::default() };
        let mut cache = DemoFogCache::default();
        let good = snapshot_envelope_cached(1, 10, 7, source(vec![grid.presentation_event().unwrap()]), 0, &mut cache);
        let decoded = RendererIpcEnvelope::decode(good.encode_to_vec().as_slice()).unwrap();
        let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = decoded.payload else { panic!("snapshot"); };
        assert_eq!(snapshot.fog_grid, Some(grid.to_presentation()));
        assert!(snapshot.fog_tiles.is_empty() && snapshot.vision_circles.is_empty());
        assert!(snapshot.tree_occluders.is_empty() && snapshot.polygon_occluders.is_empty());
        assert!(cache.formal_moba_seen);
        let bad = snapshot_envelope_cached(2, 10, 8, source(vec![grid.presentation_event().unwrap()]), 0, &mut cache);
        let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = bad.payload else { panic!("snapshot"); };
        assert!(snapshot.fog_grid.is_none());
        assert!(snapshot.fog_tiles.is_empty());
        let absent = snapshot_envelope_cached(3, 11, 8, source(vec![]), 0, &mut cache);
        let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = absent.payload else { panic!("snapshot"); };
        assert!(snapshot.fog_grid.is_none() && snapshot.vision_circles.is_empty());
    }

    #[test]
    fn fog_fixture_uses_ten_by_ten_cells_and_safe_occluders() {
        let (tiles, circles, trees, polygons) = derive_demo_fog(&[hero()], 1);
        assert!(!tiles.is_empty());
        assert!(tiles.iter().all(|tile| tile.visible));
        assert_eq!(circles[0].radius_raw, 700 * 1024);
        assert_eq!(trees.len(), 64);
        assert_eq!(polygons.len(), 3);
        assert!(tiles
            .windows(2)
            .all(|pair| (pair[0].row, pair[0].column) < (pair[1].row, pair[1].column)));
    }

    #[test]
    fn presentation_schema_has_no_canonical_identity_field() {
        let snapshot = FilteredRenderSnapshot {
            team_id: 1,
            replica_tick: 2,
            entities: Vec::new(),
            public_events: Vec::new(),
            external_effects: Vec::new(),
            memory_directives: Vec::new(),
            remembered_presentations: Default::default(),
        };
        let bytes = snapshot_envelope(1, 2, 1, snapshot, 0).encode_to_vec();
        let decoded = RendererIpcEnvelope::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.magic, PRESENTATION_MAGIC);
    }

    #[test]
    fn sanitized_damage_has_stable_identity_and_no_hidden_source_or_state_cues() {
        use omoba_core::runtime::presentation_cue::DamagePresentationCue;
        use omoba_core::game_proto::{SanitizedExternalEffect, ReplicaEntityId};
        let mut snapshot = FilteredRenderSnapshot {
            team_id: 1, replica_tick: 17,
            entities: vec![omoba_core::runtime::FilteredRenderEntity {
                replica_id: 4, disclosure_epoch: 3, entity_kind: 1, components: Default::default(),
            }], public_events: vec![], external_effects: vec![],
            memory_directives: vec![], remembered_presentations: Default::default(),
        };
        for (kind, target, index) in [(4,4,0), (4,4,1), (7,4,2), (4,99,3)] {
            snapshot.external_effects.push(SanitizedExternalEffect {
                effect_kind: kind, visible_target: Some(ReplicaEntityId {value: target}),
                sanitized_payload: 12000i64.to_le_bytes().to_vec(), stable_sub_index: index,
            });
        }
        let first = snapshot_envelope(1, 17, 5, snapshot.clone(), 0);
        let second = snapshot_envelope(99, 17, 5, snapshot, 0);
        let Some(renderer_ipc_envelope::Payload::Snapshot(first)) = first.payload else {panic!()};
        let Some(renderer_ipc_envelope::Payload::Snapshot(second)) = second.payload else {panic!()};
        assert_eq!(first.effects, second.effects, "IPC envelope sequence is not cue identity");
        assert_eq!(first.effects.len(), 2, "Buff and undisclosed targets cannot become one-shots");
        assert_ne!(first.effects[0].effect_id, first.effects[1].effect_id);
        assert_eq!(DamagePresentationCue::decode(&first.effects[0].safe_payload), Some(DamagePresentationCue {
            tick: 17, target_id: 4, disclosure_epoch: 3, amount_milli: 12000,
        }));
        assert!(first.audio_cues.is_empty());
    }

    #[test]
    fn ability_cues_project_only_live_casters_and_share_stream_identity() {
        use omoba_core::game_proto::{TeamPublicEvent, ReplicaEntityId, SanitizedExternalEffect};
        use omoba_core::runtime::presentation_cue::{AbilityPresentationCue, DamagePresentationCue, ProjectileImpactPresentationCue};
        let mut source = FilteredRenderSnapshot {
            team_id: 1, replica_tick: 17,
            entities: vec![omoba_core::runtime::FilteredRenderEntity {
                replica_id: 4, disclosure_epoch: 3, entity_kind: 1, components: Default::default(),
            }],
            public_events: vec![], external_effects: vec![SanitizedExternalEffect {
                effect_kind: omoba_core::runtime::FactKind::DirectCombat as u32,
                visible_target: Some(ReplicaEntityId {value: 4}),
                sanitized_payload: 1000_i64.to_le_bytes().to_vec(), stable_sub_index: 0,
            }], memory_directives: vec![], remembered_presentations: Default::default(),
        };
        for (kind, caster, index, payload) in [
            (8, 4, 1, AbilityPresentationCue::public_payload(123, 3)),
            (8, 99, 2, 123_u64.to_le_bytes().to_vec()),
            (8, 4, 3, vec![0; 9]),
            (7, 4, 4, 123_u64.to_le_bytes().to_vec()),
            (omoba_core::runtime::FactKind::ProjectileImpact as u32, 4, 5, b"HIT1".to_vec()),
            (omoba_core::runtime::FactKind::ProjectileImpact as u32, 99, 6, b"HIT1".to_vec()),
        ] {
            source.public_events.push(TeamPublicEvent {
                event_kind: kind, subject: Some(ReplicaEntityId {value: caster}),
                stable_sub_index: index, sanitized_payload: payload,
            });
        }
        let render = |source| {
            let envelope = snapshot_envelope(1, 17, 7, source, 0);
            let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = envelope.payload else {panic!()};
            snapshot
        };
        let snapshot = render(source.clone());
        assert_eq!(snapshot.effects.len(), 3);
        assert_ne!(snapshot.effects[0].effect_id, snapshot.effects[1].effect_id);
        assert!(DamagePresentationCue::decode(&snapshot.effects[0].safe_payload).is_some());
        assert_eq!(AbilityPresentationCue::decode(&snapshot.effects[1].safe_payload), Some(AbilityPresentationCue {
            tick: 17, caster_id: 4, disclosure_epoch: 3, ability_id: 123, rank: 3, caster_relocation: None,
        }));
        assert_eq!(ProjectileImpactPresentationCue::decode(&snapshot.effects[2].safe_payload),
            Some(ProjectileImpactPresentationCue {tick:17,target_id:4,disclosure_epoch:3}));
        source.entities.clear();
        assert!(render(source).effects.is_empty());
    }

    #[test]
    fn lifecycle_round_trip_preserves_disclosure_epoch() {
        let envelope = lifecycle_envelope(
            9,
            2,
            100,
            99,
            7,
            vec![RenderMemoryDirective::Forget {
                replica_id: 148,
                disclosure_epoch: 23,
            }],
        )
        .unwrap();
        let decoded = RendererIpcEnvelope::decode(envelope.encode_to_vec().as_slice()).unwrap();
        let Some(renderer_ipc_envelope::Payload::Lifecycle(batch)) = decoded.payload else {
            panic!("expected lifecycle payload");
        };
        assert_eq!(batch.view_epoch, 7);
        assert_eq!(batch.events[0].replica_id, 148);
        assert_eq!(batch.events[0].disclosure_epoch, 23);
        assert!(matches!(
            batch.events[0].action,
            Some(render_lifecycle_event::Action::Forget(_))
        ));
    }

    #[test]
    fn state_snapshot_does_not_carry_lifecycle_edges() {
        let snapshot = FilteredRenderSnapshot {
            team_id: 1,
            replica_tick: 2,
            entities: Vec::new(),
            public_events: Vec::new(),
            external_effects: Vec::new(),
            memory_directives: vec![RenderMemoryDirective::Forget {
                replica_id: 9,
                disclosure_epoch: 4,
            }],
            remembered_presentations: Default::default(),
        };
        let decoded = RendererIpcEnvelope::decode(
            snapshot_envelope(1, 2, 1, snapshot, 0)
                .encode_to_vec()
                .as_slice(),
        )
        .unwrap();
        let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = decoded.payload else {
            panic!("expected snapshot payload");
        };
        assert!(snapshot.removed_render_ids.is_empty());
        assert!(snapshot.remembered_ghosts.is_empty());
    }

    #[test]
    fn snapshot_envelope_carries_runtime_rtt() {
        let snapshot = FilteredRenderSnapshot {
            team_id: 1,
            replica_tick: 2,
            entities: Vec::new(),
            public_events: Vec::new(),
            external_effects: Vec::new(),
            memory_directives: Vec::new(),
            remembered_presentations: Default::default(),
        };
        let decoded = RendererIpcEnvelope::decode(
            snapshot_envelope(1, 2, 1, snapshot, 1_500)
                .encode_to_vec()
                .as_slice(),
        )
        .unwrap();
        let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = decoded.payload else {
            panic!("expected snapshot payload");
        };
        assert_eq!(snapshot.runtime_rtt_us, 1_500);
    }

    #[test]
    fn reconnect_snapshot_carries_frozen_memory_separately_from_live_entities() {
        let source = FilteredRenderSnapshot {
            team_id: 2,
            replica_tick: 90,
            entities: Vec::new(),
            public_events: Vec::new(),
            external_effects: Vec::new(),
            memory_directives: Vec::new(),
            remembered_presentations: std::collections::BTreeMap::from([
                ((8, 3), b"server-sanitized-last-known".to_vec()),
                ((9, 4), Vec::new()),
            ]),
        };
        let message = snapshot_envelope(20, 91, 7, source, 0);
        let decoded = RendererIpcEnvelope::decode(message.encode_to_vec().as_slice()).unwrap();
        let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = decoded.payload else {
            panic!("expected snapshot");
        };
        assert!(snapshot.entities.is_empty());
        assert!(snapshot.removed_render_ids.is_empty());
        assert_eq!(snapshot.remembered_ghosts.len(), 1);
        let ghost = &snapshot.remembered_ghosts[0];
        assert_eq!((ghost.render_id, ghost.disclosure_epoch), (8, 3));
        assert_eq!(ghost.sanitized_presentation, b"server-sanitized-last-known");
    }

    #[tokio::test]
    async fn framing_rejects_wrong_version_and_oversized_length() {
        let (mut writer, mut reader) = tokio::io::duplex(64);
        writer
            .write_u32((MAX_PRESENTATION_FRAME_BYTES + 1) as u32)
            .await
            .unwrap();
        assert!(read_envelope(&mut reader).await.is_err());
        let wrong = RendererIpcEnvelope {
            magic: PRESENTATION_MAGIC,
            protocol_version: 99,
            sequence: 0,
            payload: None,
        };
        let (mut writer, mut reader) = tokio::io::duplex(128);
        let bytes = wrong.encode_to_vec();
        writer.write_u32(bytes.len() as u32).await.unwrap();
        writer.write_all(&bytes).await.unwrap();
        assert!(read_envelope(&mut reader).await.is_err());
    }

    #[tokio::test]
    async fn point_queue_rejects_version_three_before_admitting_input() {
        let old = RendererIpcEnvelope {
            magic: PRESENTATION_MAGIC, protocol_version: 3, sequence: 1,
            payload: Some(renderer_ipc_envelope::Payload::RendererInput(RendererInput {
                request_id: 1, player_id: 7, disclosure_epoch: 1,
                intent: Some(omoba_core::game_proto::renderer_input::Intent::MoveTo(
                    omoba_core::game_proto::MoveToIntent { x_raw: 1, y_raw: 2, queued: true })),
            })),
        };
        let bytes = old.encode_to_vec();
        let (mut writer, mut reader) = tokio::io::duplex(256);
        writer.write_u32(bytes.len() as u32).await.unwrap();
        writer.write_all(&bytes).await.unwrap();
        assert!(read_envelope(&mut reader).await.is_err());
        assert_eq!(PRESENTATION_PROTOCOL_VERSION, 5);
    }

    #[tokio::test]
    async fn renderer_handshake_rejects_wrong_team_before_disclosing_snapshot() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (latest_tx, latest_rx) = watch::channel(Some(Arc::new(envelope(
            1,
            renderer_ipc_envelope::Payload::RuntimeReady(RuntimeReadyPresentation {
                player_id: 7,
                team_id: 1,
                authoritative_tick: 1,
                replica_tick: 1,
                content_hash: "private-team-state".into(),
                tick_rate_hz: 60,
                ..Default::default()
            }),
        ))));
        let (_critical_tx, critical_rx) = mpsc::channel(1);
        let (input_tx, _input_rx) = mpsc::channel(1);
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            serve_renderer(
                stream,
                7,
                1,
                latest_rx,
                Arc::new(Mutex::new(critical_rx)),
                input_tx,
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicU64::new(0)),
            )
            .await
        });
        let mut client = TcpStream::connect(address).await.unwrap();
        write_envelope(
            &mut client,
            &envelope(
                0,
                renderer_ipc_envelope::Payload::RendererReady(
                    omoba_core::game_proto::RendererReady {
                        latest_snapshot_sequence: 0,
                        player_id: 7,
                        team_id: 2,
                    },
                ),
            ),
        )
        .await
        .unwrap();
        let rejected = server.await.unwrap();
        assert!(matches!(rejected, Err(ClientRuntimeError::Ipc(_))));
        assert!(read_envelope(&mut client).await.is_err());
        drop(latest_tx);
    }

    #[tokio::test]
    async fn socket_reconnect_restores_frozen_memory_without_new_simulation_frame() {
        tokio::time::timeout(Duration::from_secs(3), async {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let source = FilteredRenderSnapshot {
                team_id: 1,
                replica_tick: 90,
                entities: Vec::new(),
                public_events: Vec::new(),
                external_effects: Vec::new(),
                memory_directives: Vec::new(),
                remembered_presentations: std::collections::BTreeMap::from([(
                    (8, 3),
                    b"frozen-before-disconnect".to_vec(),
                )]),
            };
            let mut retained = snapshot_envelope(20, 91, 7, source, 0);
            if let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = &mut retained.payload
            {
                snapshot
                    .effects
                    .push(omoba_core::game_proto::PresentationEffect {
                        effect_id: 100,
                        safe_payload: b"consumed-vfx".to_vec(),
                    });
                snapshot.owner_economy = Some(omoba_core::game_proto::OwnerEconomyPresentation {
                    schema_version: 2, player_id: 7, gold: 550, shield_remaining_raw: 102400,
                    slots: (0..6).map(|slot| omoba_core::game_proto::InventorySlotPresentation {
                        slot, catalog_id: if slot == 1 { 3 } else { 0 }, cooldown_seconds: 0.0 }).collect(),
                    shop_available: false,
                    shop_protocol_enabled: false,
                });
                snapshot.shop_receipts = vec![omoba_core::game_proto::ShopTransactionReceipt {
                    schema_version: 1, player_id: 7, input_id: 42, settled_tick: 89,
                    action_kind: 17, catalog_id: 3, result_code: 0, ..Default::default()
                }];
                snapshot
                    .audio_cues
                    .push(omoba_core::game_proto::PresentationEffect {
                        effect_id: 101,
                        safe_payload: b"consumed-audio".to_vec(),
                    });
            }
            let expected = renderer_baseline(&retained);
            let (latest_tx, latest_rx) = watch::channel(Some(Arc::new(retained.clone())));
            let (_critical_tx, critical_rx) = mpsc::channel(1);
            let critical_rx = Arc::new(Mutex::new(critical_rx));
            let (input_tx, mut input_rx) = mpsc::channel(1);
            let connected = Arc::new(AtomicBool::new(false));
            let flag = connected.clone();
            let server = tokio::spawn(async move {
                for _ in 0..2 {
                    let (stream, _) = listener.accept().await.unwrap();
                    let _ = serve_renderer(
                        stream,
                        7,
                        1,
                        latest_rx.clone(),
                        critical_rx.clone(),
                        input_tx.clone(),
                        flag.clone(),
                        Arc::new(AtomicU64::new(0)),
                    )
                    .await;
                }
            });
            for _ in 0..2 {
                let mut client = TcpStream::connect(address).await.unwrap();
                write_envelope(
                    &mut client,
                    &envelope(
                        0,
                        renderer_ipc_envelope::Payload::RendererReady(
                            omoba_core::game_proto::RendererReady {
                                latest_snapshot_sequence: 0,
                                player_id: 7,
                                team_id: 1,
                            },
                        ),
                    ),
                )
                .await
                .unwrap();
                let received = read_envelope(&mut client).await.unwrap();
                assert_eq!(received, expected);
                drop(client);
            }
            server.await.unwrap();
            assert_eq!(
                *latest_tx.borrow().as_ref().unwrap().as_ref(),
                retained,
                "baseline must not mutate shared safe source"
            );
            assert!(!connected.load(Ordering::Acquire));
            assert!(input_rx.try_recv().is_err());
        })
        .await
        .expect("renderer reconnect test timed out");
    }

    #[test]
    fn baseline_preserves_terminal_results_and_all_persistent_state() {
        let original = envelope(
            9,
            renderer_ipc_envelope::Payload::Snapshot(TeamPresentationSnapshot {
                team_id: 2,
                replica_tick: 99,
                authoritative_tick: 100,
                view_epoch: 3,
                moba_hud: Some(omoba_core::game_proto::MobaHudPresentation {
                    schema_version: 1, player_id: 7, phase: 2, winner_team: 2,
                    respawn_remaining_raw: 1024,
                    scoreboard:Some(omoba_core::game_proto::ScoreboardPresentation {schema_version:1,rows:vec![
                        omoba_core::game_proto::ScoreboardRowPresentation {player_id:9,team_id:1,kills:0,deaths:1,assists:0},
                        omoba_core::game_proto::ScoreboardRowPresentation {player_id:7,team_id:2,kills:1,deaths:2,assists:3},
                    ]}),
                    score:Some(omoba_core::game_proto::OwnerScorePresentation {player_id:7,kills:1,deaths:2,assists:3}),
                    ..Default::default()
                }),
                effects: vec![omoba_core::game_proto::PresentationEffect {
                    effect_id: 7,
                    safe_payload: vec![1],
                }],
                audio_cues: vec![omoba_core::game_proto::PresentationEffect {
                    effect_id: 8,
                    safe_payload: vec![2],
                }],
                fog_tiles: vec![FogTilePresentation {
                    row: 4,
                    column: 5,
                    visible: true,
                }],
                ..Default::default()
            }),
        );
        let mut expected = original.clone();
        if let Some(renderer_ipc_envelope::Payload::Snapshot(s)) = &mut expected.payload {
            s.effects.clear();
            s.audio_cues.clear();
        }
        assert_eq!(renderer_baseline(&original), expected);
        let result = envelope(
            10,
            renderer_ipc_envelope::Payload::CriticalInputResult(
                omoba_core::game_proto::CriticalInputResult {
                    request_id: 55,
                    accepted: true,
                    result_code: "APPLIED".into(),
                    ..Default::default()
                },
            ),
        );
        assert_eq!(renderer_baseline(&result), result);
    }

    #[tokio::test]
    async fn hub_projects_owner_economy_and_retains_receipts_from_unpublished_steps() {
        use omoba_core::runtime::{FactKind, native::economy_projection::{OwnerEconomyState, CommittedEconomyState},
            shop_receipt::ShopReceipt};
        let config = ClientRuntimeConfig::parse(["--player-id", "7", "--team", "2", "--server", "127.0.0.1:7777",
            "--presentation-bind", "127.0.0.1:0"].into_iter().map(str::to_owned)).unwrap();
        let mut hub = PresentationHub::bind(&config).await.unwrap();
        let receipt = |player| omoba_core::game_proto::TeamPublicEvent {
            event_kind: FactKind::ShopReceipt as u32, sanitized_payload: ShopReceipt {
                player_id: player, input_id: 42, tick: 89, action_kind: 17, catalog_id: 3,
                slot: 0, result_code: 6 }.encode(), ..Default::default() };
        hub.retain_shop_receipts(4, 89, &[receipt(7), receipt(8)]);
        let replay = omoba_core::game_proto::ShopReceiptReplay { schema_version: 1,
            request_id: 10, player_id: 7, input_id: 43, status: 2,
            receipt: ShopReceipt { player_id: 7, input_id: 43, tick: 70, action_kind: 18,
                catalog_id: 0, slot: 1, result_code: 7 }.encode() };
        assert_eq!(hub.retain_shop_replay(4, &replay), Ok(true));
        assert_eq!(hub.retain_shop_replay(4, &replay), Ok(true));
        let mut wrong = replay.clone(); wrong.player_id = 8;
        assert!(hub.retain_shop_replay(4, &wrong).is_err());
        hub.retain_shop_receipts(4, 90, &[]); // discarded intermediate presentation
        let owner = OwnerEconomyState { player_id: 7, shop_available: false, shield_remaining_raw: 0,
            economy: CommittedEconomyState { gold: 550, item_ids: [0, 3, 0, 0, 0, 0],
                cooldown_bits: [0; 6], effect_bits: [0; 10], dirty: false } };
        let source = FilteredRenderSnapshot { team_id: 2, replica_tick: 91, entities: vec![],
            public_events: vec![omoba_core::game_proto::TeamPublicEvent { event_kind: FactKind::OwnerEconomy as u32,
                sanitized_payload: owner.encode(), ..Default::default() }], external_effects: vec![],
            memory_directives: vec![], remembered_presentations: Default::default() };
        let closed = hub.snapshot_envelope(7, 91, 4, source.clone(), 0);
        let Some(renderer_ipc_envelope::Payload::Snapshot(closed)) = closed.payload else { panic!("snapshot"); };
        assert!(!closed.owner_economy.unwrap().shop_protocol_enabled);
        hub.set_shop_protocol_enabled(true);
        let envelope = hub.snapshot_envelope(8, 91, 4, source, 0);
        let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = &envelope.payload else { panic!("snapshot"); };
        assert_eq!(snapshot.owner_economy.as_ref().unwrap().gold, 550);
        assert!(snapshot.owner_economy.as_ref().unwrap().shop_protocol_enabled);
        assert_eq!(snapshot.owner_economy.as_ref().unwrap().slots[1].catalog_id, 3);
        assert_eq!(snapshot.shop_receipts.len(), 2);
        assert_eq!(snapshot.shop_receipts[1].settled_tick, 70, "recovery never fabricates a new settlement tick");
        assert_eq!(snapshot.shop_receipts[1].result_code, 7);
        assert_eq!((snapshot.shop_receipts[0].player_id, snapshot.shop_receipts[0].input_id,
            snapshot.shop_receipts[0].result_code), (7, 42, 6));
        assert_eq!(RendererIpcEnvelope::decode(envelope.encode_to_vec().as_slice()).unwrap(), envelope);
        assert_eq!(renderer_baseline(&envelope), envelope, "persistent results are not historical VFX");
    }

    #[tokio::test]
    async fn retained_cues_survive_watch_overwrite_and_ack_filter_already_prepared_frames() {
        use crate::cue_retention::{CueRetention, tests::{effect, ability_effect, ranked_ability_effect, relocation_effect, area_effect, impact_effect, snapshot}};
        tokio::time::timeout(Duration::from_secs(3), async {
            let variants: [fn(u64, u32) -> omoba_core::game_proto::PresentationEffect; 6] = [effect, ability_effect, ranked_ability_effect, relocation_effect, area_effect, impact_effect];
            for effect in variants {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let ledger = Arc::new(std::sync::Mutex::new(CueRetention::default()));
            ledger.lock().unwrap().capture(7, 2, vec![effect(2, 0)]);
            let first = envelope(1, renderer_ipc_envelope::Payload::Snapshot(snapshot(1)));
            let (latest_tx, latest_rx) = watch::channel(Some(Arc::new(first.clone())));
            let (_critical_tx, critical_rx) = mpsc::channel(8);
            let (input_tx, _input_rx) = mpsc::channel(1);
            let shared = ledger.clone();
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                serve_renderer_retained(stream, 7, 1, latest_rx, Arc::new(Mutex::new(critical_rx)),
                    input_tx, Arc::new(AtomicBool::new(false)), Arc::new(AtomicU64::new(0)), Some(shared), None).await
            });
            let mut client = TcpStream::connect(address).await.unwrap();
            write_envelope(&mut client, &envelope(0, renderer_ipc_envelope::Payload::RendererReady(
                omoba_core::game_proto::RendererReady {player_id: 7, team_id: 1, latest_snapshot_sequence: 0}))).await.unwrap();
            assert_eq!(read_envelope(&mut client).await.unwrap(), first);
            assert!(!ledger.lock().unwrap().contains(effect(2, 0).effect_id));
            ledger.lock().unwrap().capture(7, 3, vec![effect(3, 0)]);
            let mut skipped = snapshot(3);
            ledger.lock().unwrap().project(&mut skipped);
            latest_tx.send_replace(Some(Arc::new(envelope(2, renderer_ipc_envelope::Payload::Snapshot(skipped)))));
            ledger.lock().unwrap().capture(7, 4, vec![effect(4, 0)]);
            let mut live = snapshot(4);
            ledger.lock().unwrap().project(&mut live);
            let mut prepared = envelope(3, renderer_ipc_envelope::Payload::Snapshot(live));
            latest_tx.send_replace(Some(Arc::new(prepared.clone())));
            let received = read_envelope(&mut client).await.unwrap();
            assert_eq!(received, prepared);
            write_envelope(&mut client, &envelope(0, renderer_ipc_envelope::Payload::RendererConsumed(
                omoba_core::game_proto::RendererConsumed {snapshot_sequence: 3}))).await.unwrap();
            while ledger.lock().unwrap().contains(effect(3, 0).effect_id) {
                tokio::task::yield_now().await;
            }
            assert!(!ledger.lock().unwrap().contains(effect(4, 0).effect_id));
            // Recollecting a successfully consumed event cannot make a
            // previously prepared frame eligible for delivery again.
            ledger.lock().unwrap().capture(7,4,vec![effect(3,0),effect(4,0)]);
            assert!(!ledger.lock().unwrap().contains(effect(3,0).effect_id));
            assert!(!ledger.lock().unwrap().contains(effect(4,0).effect_id));
            prepared.sequence = 4; // Created before ACK, delivered after ACK.
            latest_tx.send_replace(Some(Arc::new(prepared)));
            let received = read_envelope(&mut client).await.unwrap();
            let Some(renderer_ipc_envelope::Payload::Snapshot(view)) = received.payload else {panic!("expected snapshot")};
            assert!(view.effects.is_empty());
            ledger.lock().unwrap().capture(7,5,vec![effect(5,0)]);
            let mut fresh=snapshot(5);
            ledger.lock().unwrap().project(&mut fresh);
            assert_eq!(fresh.effects,vec![effect(5,0)]);
            latest_tx.send_replace(Some(Arc::new(envelope(5,renderer_ipc_envelope::Payload::Snapshot(fresh.clone())))));
            assert_eq!(read_envelope(&mut client).await.unwrap(),
                envelope(5,renderer_ipc_envelope::Payload::Snapshot(fresh)));
            write_envelope(&mut client, &envelope(0, renderer_ipc_envelope::Payload::RendererShutdown(Default::default()))).await.unwrap();
            assert!(server.await.unwrap().is_ok());
            }
        }).await.unwrap();
    }

    #[tokio::test]
    async fn all_retained_cue_kinds_survive_real_tcp_reconnect_without_history_replay() {
        use crate::cue_retention::{CueRetention, tests::{effect, ability_effect, ranked_ability_effect, relocation_effect, area_effect, impact_effect, snapshot}};
        tokio::time::timeout(Duration::from_secs(5), async {
            let variants: [fn(u64, u32) -> omoba_core::game_proto::PresentationEffect; 6] =
                [effect, ability_effect, ranked_ability_effect, relocation_effect, area_effect, impact_effect];
            let cues = |tick| variants.iter().enumerate().map(|(i, make)| make(tick, i as u32)).collect::<Vec<_>>();
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let ledger = Arc::new(std::sync::Mutex::new(CueRetention::default()));
            let mut initial = snapshot(10);
            initial.effects = cues(10);
            ledger.lock().unwrap().capture(7, 10, initial.effects.clone());
            let (latest_tx, latest_rx) = watch::channel(Some(Arc::new(envelope(10,
                renderer_ipc_envelope::Payload::Snapshot(initial)))));
            let (_critical_tx, critical_rx) = mpsc::channel(8);
            let critical = Arc::new(Mutex::new(critical_rx));
            let (input_tx, _input_rx) = mpsc::channel(1);
            let shared = ledger.clone();
            let (closed_tx, mut closed_rx) = mpsc::channel(2);
            let server = tokio::spawn(async move {
                let consumed = Arc::new(AtomicU64::new(0));
                let shutdown = Arc::new(AtomicBool::new(false));
                for session in 0..2 {
                    let (stream, _) = listener.accept().await.unwrap();
                    serve_renderer_retained(stream, 7, 1, latest_rx.clone(), critical.clone(),
                        input_tx.clone(), shutdown.clone(), consumed.clone(), Some(shared.clone()), None)
                        .await.unwrap_or_else(|e| panic!("renderer session {session}: {e}"));
                    closed_tx.send(session).await.unwrap();
                }
            });
            for (baseline_tick, live_tick) in [(10, 11), (12, 13)] {
                let mut client = TcpStream::connect(address).await.unwrap();
                write_envelope(&mut client, &envelope(0, renderer_ipc_envelope::Payload::RendererReady(
                    omoba_core::game_proto::RendererReady {player_id: 7, team_id: 1, latest_snapshot_sequence: 0})))
                    .await.unwrap_or_else(|e| panic!("ready at baseline {baseline_tick}: {e}"));
                let baseline = read_envelope(&mut client).await.unwrap();
                let Some(renderer_ipc_envelope::Payload::Snapshot(view)) = baseline.payload else {panic!("expected baseline")};
                assert_eq!(view.replica_tick, baseline_tick);
                assert!(view.effects.is_empty(), "reconnect baseline must not replay old or offline cues");
                assert!(cues(baseline_tick).iter().all(|cue| !ledger.lock().unwrap().contains(cue.effect_id)));
                let fresh = cues(live_tick);
                ledger.lock().unwrap().capture(7, live_tick, fresh.clone());
                let mut live = snapshot(live_tick);
                live.effects = cues(10); // A frame prepared before consumption can retain history.
                live.effects.extend(cues(12).into_iter().filter(|_| live_tick > 12));
                ledger.lock().unwrap().project(&mut live);
                latest_tx.send_replace(Some(Arc::new(envelope(live_tick,
                    renderer_ipc_envelope::Payload::Snapshot(live)))));
                let received = read_envelope(&mut client).await.unwrap();
                let Some(renderer_ipc_envelope::Payload::Snapshot(view)) = received.payload else {panic!("expected live snapshot")};
                assert_eq!(view.effects, fresh, "all six fresh cue kinds delivered exactly once");
                write_envelope(&mut client, &envelope(0, renderer_ipc_envelope::Payload::RendererConsumed(
                    omoba_core::game_proto::RendererConsumed {snapshot_sequence: live_tick}))).await.unwrap();
                while fresh.iter().any(|cue| ledger.lock().unwrap().contains(cue.effect_id)) {
                    tokio::task::yield_now().await;
                }
                write_envelope(&mut client, &envelope(0,
                    renderer_ipc_envelope::Payload::RendererShutdown(Default::default()))).await.unwrap();
                assert_eq!(closed_rx.recv().await, Some(if live_tick == 11 { 0 } else { 1 }));
                drop(client);
                if live_tick == 11 {
                    let offline = cues(12);
                    ledger.lock().unwrap().capture(7, 12, offline.clone());
                    let mut view = snapshot(12);
                    view.effects = fresh;
                    view.effects.extend(offline);
                    latest_tx.send_replace(Some(Arc::new(envelope(12,
                        renderer_ipc_envelope::Payload::Snapshot(view)))));
                }
            }
            server.await.unwrap();
        }).await.unwrap();
    }

    #[tokio::test]
    async fn first_snapshot_after_ready_is_baseline_but_new_live_effects_survive() {
        tokio::time::timeout(Duration::from_secs(3), async {
            for first_is_critical in [false, true] {
                let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                let address = listener.local_addr().unwrap();
                let ready = envelope(
                    1,
                    renderer_ipc_envelope::Payload::RuntimeReady(Default::default()),
                );
                let (latest_tx, latest_rx) = watch::channel(Some(Arc::new(ready.clone())));
                let (critical_tx, critical_rx) = mpsc::channel(8);
                let (input_tx, _input_rx) = mpsc::channel(1);
                let server = tokio::spawn(async move {
                    let (stream, _) = listener.accept().await.unwrap();
                    serve_renderer(
                        stream,
                        7,
                        1,
                        latest_rx,
                        Arc::new(Mutex::new(critical_rx)),
                        input_tx,
                        Arc::new(AtomicBool::new(false)),
                        Arc::new(AtomicU64::new(0)),
                    )
                    .await
                });
                let mut client = TcpStream::connect(address).await.unwrap();
                write_envelope(
                    &mut client,
                    &envelope(
                        0,
                        renderer_ipc_envelope::Payload::RendererReady(
                            omoba_core::game_proto::RendererReady {
                                player_id: 7,
                                team_id: 1,
                                latest_snapshot_sequence: 0,
                            },
                        ),
                    ),
                )
                .await
                .unwrap();
                assert_eq!(read_envelope(&mut client).await.unwrap(), ready);
                let first = envelope(
                    2,
                    renderer_ipc_envelope::Payload::Snapshot(TeamPresentationSnapshot {
                        team_id: 1,
                        replica_tick: 3,
                        effects: vec![omoba_core::game_proto::PresentationEffect {
                            effect_id: 42,
                            safe_payload: vec![5],
                        }],
                        audio_cues: vec![omoba_core::game_proto::PresentationEffect {
                            effect_id: 43,
                            safe_payload: vec![6],
                        }],
                        ..Default::default()
                    }),
                );
                if first_is_critical {
                    critical_tx.send(first.clone()).await.unwrap();
                } else {
                    latest_tx.send_replace(Some(Arc::new(first.clone())));
                }
                assert_eq!(
                    read_envelope(&mut client).await.unwrap(),
                    renderer_baseline(&first)
                );
                for sequence in [2, 2, 0] {
                    write_envelope(
                        &mut client,
                        &envelope(
                            0,
                            renderer_ipc_envelope::Payload::RendererConsumed(
                                omoba_core::game_proto::RendererConsumed {
                                    snapshot_sequence: sequence,
                                },
                            ),
                        ),
                    )
                    .await
                    .unwrap();
                }
                let mut live = first.clone();
                live.sequence = 3;
                if let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = &mut live.payload
                {
                    snapshot.effects[0].effect_id = 44;
                    snapshot.audio_cues[0].effect_id = 45;
                }
                // A subsequent live frame still carries fresh one-shots.
                critical_tx.send(live.clone()).await.unwrap();
                assert_eq!(read_envelope(&mut client).await.unwrap(), live);
                // Retained typed damage at/before the reconnect baseline must
                // also be stripped from later snapshots, not just the first.
                live.sequence = 4;
                if let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = &mut live.payload {
                    use omoba_core::runtime::presentation_cue::{DamagePresentationCue, presentation_effect_id};
                    snapshot.replica_tick = 4;
                    for tick in [3, 4] {
                        snapshot.effects.push(omoba_core::game_proto::PresentationEffect {
                            effect_id: presentation_effect_id(tick, 0).unwrap(),
                            safe_payload: DamagePresentationCue {tick, target_id: 4, disclosure_epoch: 3, amount_milli: 1000}.encode(),
                        });
                    }
                }
                let mut expected = live.clone();
                if let Some(renderer_ipc_envelope::Payload::Snapshot(snapshot)) = &mut expected.payload {
                    snapshot.effects.remove(1); // baseline tick 3, preserve fresh tick 4
                }
                if first_is_critical {latest_tx.send(Some(Arc::new(live))).unwrap();}
                else {critical_tx.send(live).await.unwrap();}
                assert_eq!(read_envelope(&mut client).await.unwrap(), expected);
                write_envelope(
                    &mut client,
                    &envelope(
                        0,
                        renderer_ipc_envelope::Payload::RendererConsumed(
                            omoba_core::game_proto::RendererConsumed {
                                snapshot_sequence: 999,
                            },
                        ),
                    ),
                )
                .await
                .unwrap();
                let error = server.await.unwrap().unwrap_err();
                assert!(error
                    .to_string()
                    .contains("renderer consumed unsent snapshot"));
                assert!(read_envelope(&mut client).await.is_err());
            }
        })
        .await
        .expect("late baseline socket test timed out");
    }

    #[test]
    fn renderer_handshake_requires_matching_player_and_team() {
        let ready = omoba_core::game_proto::RendererReady {
            latest_snapshot_sequence: 0,
            player_id: 7,
            team_id: 1,
        };
        assert!(validate_renderer_ready(&ready, 7, 1).is_ok());
        assert!(validate_renderer_ready(&ready, 8, 1).is_err());
        assert!(validate_renderer_ready(&ready, 7, 2).is_err());
    }

    #[tokio::test]
    async fn fragmented_input_survives_outgoing_frames_and_exclusive_reconnect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (latest_tx, latest_rx) = watch::channel(None);
        let (critical_tx, critical_rx) = mpsc::channel(8);
        let critical_rx = Arc::new(Mutex::new(critical_rx));
        let (input_tx, mut input_rx) = mpsc::channel(8);
        let connected = Arc::new(AtomicBool::new(false));
        let disconnected = Arc::new(AtomicU64::new(0));
        let flag = connected.clone();
        let server = tokio::spawn(async move {
            let mut sessions = Vec::new();
            for _ in 0..3 {
                let (stream, _) = listener.accept().await.unwrap();
                sessions.push(tokio::spawn(serve_renderer(
                    stream,
                    7,
                    1,
                    latest_rx.clone(),
                    critical_rx.clone(),
                    input_tx.clone(),
                    flag.clone(),
                    disconnected.clone(),
                )));
            }
            for session in sessions {
                let _ = session.await.unwrap();
            }
        });
        let ready = envelope(
            0,
            renderer_ipc_envelope::Payload::RendererReady(omoba_core::game_proto::RendererReady {
                player_id: 7,
                team_id: 1,
                latest_snapshot_sequence: 0,
            }),
        );
        let mut first = TcpStream::connect(address).await.unwrap();
        write_envelope(&mut first, &ready).await.unwrap();
        latest_tx.send_replace(Some(Arc::new(envelope(
            1,
            renderer_ipc_envelope::Payload::RuntimeReady(RuntimeReadyPresentation::default()),
        ))));
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), read_envelope(&mut first))
                .await
                .unwrap()
                .unwrap()
                .sequence,
            1
        );
        assert!(connected.load(Ordering::Acquire));

        // A second renderer cannot consume the first renderer's input results.
        let mut second = TcpStream::connect(address).await.unwrap();
        write_envelope(&mut second, &ready).await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_secs(2), read_envelope(&mut second))
                .await
                .unwrap()
                .is_err()
        );
        assert!(connected.load(Ordering::Acquire));

        let input = RendererInput {
            request_id: 91,
            player_id: 7,
            ..RendererInput::default()
        };
        let bytes = envelope(
            2,
            renderer_ipc_envelope::Payload::RendererInput(input.clone()),
        )
        .encode_to_vec();
        first.write_u32(bytes.len() as u32).await.unwrap();
        first.write_all(&bytes[..2]).await.unwrap();
        // Sending and receiving this frame ensures the server runs its writer
        // while the input reader is waiting for the rest of the body.
        critical_tx
            .send(envelope(
                2,
                renderer_ipc_envelope::Payload::CriticalInputResult(
                    omoba_core::game_proto::CriticalInputResult {
                        request_id: 90,
                        input_id: 1,
                        accepted: true,
                        result_code: "FORWARDED".into(),
                        authoritative_tick: 8,
                    },
                ),
            ))
            .await
            .unwrap();
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), read_envelope(&mut first))
                .await
                .unwrap()
                .unwrap()
                .sequence,
            2
        );
        latest_tx.send_replace(Some(Arc::new(envelope(
            3,
            renderer_ipc_envelope::Payload::RuntimeReady(RuntimeReadyPresentation::default()),
        ))));
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), read_envelope(&mut first))
                .await
                .unwrap()
                .unwrap()
                .sequence,
            3
        );
        first.write_all(&bytes[2..]).await.unwrap();
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), input_rx.recv())
                .await
                .unwrap()
                .unwrap(),
            input
        );

        // A graceful disconnect releases the renderer lease without touching
        // the ongoing game or latest presentation state.
        write_envelope(
            &mut first,
            &envelope(
                4,
                renderer_ipc_envelope::Payload::RendererShutdown(
                    omoba_core::game_proto::RendererShutdown { graceful: true },
                ),
            ),
        )
        .await
        .unwrap();
        assert!(
            tokio::time::timeout(Duration::from_secs(2), read_envelope(&mut first))
                .await
                .unwrap()
                .is_err()
        );
        assert!(!connected.load(Ordering::Acquire));
        let snapshot = |sequence| {
            envelope(
                sequence,
                renderer_ipc_envelope::Payload::Snapshot(TeamPresentationSnapshot::default()),
            )
        };
        let result = |sequence| {
            envelope(
                sequence,
                renderer_ipc_envelope::Payload::CriticalInputResult(
                    omoba_core::game_proto::CriticalInputResult {
                        request_id: 91,
                        accepted: true,
                        result_code: "APPLIED_TO_PRESENTATION".into(),
                        ..Default::default()
                    },
                ),
            )
        };
        latest_tx.send_replace(Some(Arc::new(snapshot(50))));
        critical_tx
            .send(reset_view_envelope(40, 1, 1, 1, 1))
            .await
            .unwrap();
        critical_tx.send(result(41)).await.unwrap();
        let mut third = TcpStream::connect(address).await.unwrap();
        write_envelope(&mut third, &ready).await.unwrap();
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), read_envelope(&mut third))
                .await
                .unwrap()
                .unwrap()
                .sequence,
            50
        );
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), read_envelope(&mut third))
                .await
                .unwrap()
                .unwrap()
                .sequence,
            41,
            "covered lifecycle must be dropped, but pending input results must survive reconnect"
        );
        critical_tx.send(snapshot(60)).await.unwrap();
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), read_envelope(&mut third))
                .await
                .unwrap()
                .unwrap()
                .sequence,
            60
        );
        latest_tx.send_replace(Some(Arc::new(snapshot(55))));
        critical_tx.send(result(61)).await.unwrap();
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), read_envelope(&mut third))
                .await
                .unwrap()
                .unwrap()
                .sequence,
            61,
            "old watched snapshots must not rewind a newer critical snapshot"
        );
        drop(third);
        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .unwrap()
            .unwrap();
        assert!(!connected.load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn latest_snapshot_is_overwritten_but_critical_queue_is_ordered() {
        let (latest_tx, latest_rx) = watch::channel(None::<u64>);
        latest_tx.send_replace(Some(1));
        latest_tx.send_replace(Some(2));
        assert_eq!(*latest_rx.borrow(), Some(2));
        let (critical_tx, mut critical_rx) = mpsc::channel(2);
        critical_tx.send(10).await.unwrap();
        critical_tx.send(11).await.unwrap();
        assert_eq!(critical_rx.recv().await, Some(10));
        assert_eq!(critical_rx.recv().await, Some(11));
    }

    #[tokio::test]
    async fn formal_perf_incomplete_transfer_not_counted() {
        use std::pin::Pin;
        use std::task::{Context, Poll};
        use tokio::io::AsyncWrite;

        struct FailWrite;
        impl AsyncWrite for FailWrite {
            fn poll_write(
                self: Pin<&mut Self>,
                _: &mut Context<'_>,
                _: &[u8],
            ) -> Poll<std::io::Result<usize>> {
                Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "incomplete",
                )))
            }
            fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }
            fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }
        }

        let envelope = envelope(1, renderer_ipc_envelope::Payload::RendererShutdown(Default::default()));
        let mut server = PresentationIpcMeter::new(1, 1, 4);
        let failed = write_envelope(&mut FailWrite, &envelope).await;
        assert!(failed.is_err());
        assert!(server.observe_send_result(failed).is_none());
        assert_eq!(server.send_samples(), 0);

        let (mut reader, writer) = tokio::io::duplex(16);
        drop(writer);
        let failed_read = read_envelope_measured(&mut reader).await;
        assert!(failed_read.is_err());
        assert!(server.observe_receive_result(failed_read.map(|(_, bytes)| bytes)).is_none());
        assert_eq!(server.receive_samples(), 0);

        let (mut reader, mut writer) = tokio::io::duplex(16);
        writer.write_all(&[0x00, 0x01]).await.unwrap();
        drop(writer);
        assert!(read_envelope_measured(&mut reader).await.is_err());
        assert_eq!(server.receive_samples(), 0);
    }

    #[tokio::test]
    async fn formal_perf_client_write_does_not_pollute_server_meter() {
        let (mut client, mut server_reader) = tokio::io::duplex(4096);
        let envelope = envelope(1, renderer_ipc_envelope::Payload::RendererShutdown(Default::default()));
        let mut client_meter = PresentationIpcMeter::new(7, 2, 10);
        let mut server_meter = PresentationIpcMeter::new(7, 2, 11);
        let wire = write_envelope(&mut client, &envelope).await.unwrap();
        assert!(wire >= 4);
        client_meter.observe_send(wire);
        assert_eq!(client_meter.send_samples(), 1);
        assert_eq!(server_meter.send_samples(), 0);
        assert_eq!(server_meter.receive_samples(), 0);
        let (received, read_wire) = read_envelope_measured(&mut server_reader).await.unwrap();
        assert_eq!(received.sequence, 1);
        assert_eq!(read_wire, wire);
        server_meter.observe_receive(read_wire);
        assert_eq!(server_meter.receive_samples(), 1);
        assert_eq!(client_meter.receive_samples(), 0);
        assert_eq!(server_meter.send_samples(), 0);
    }
}
