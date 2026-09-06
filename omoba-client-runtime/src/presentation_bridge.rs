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
        PresentationRenderEntity, RenderLifecycleBatch, RenderLifecycleEvent,
        RenderLifecycleForget, RenderLifecycleHide, RenderLifecycleResetView, RendererInput,
        RendererIpcEnvelope, RuntimeReadyPresentation, TeamPresentationSnapshot,
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

pub const PRESENTATION_MAGIC: u32 = 0x4f4d_5254;
pub const PRESENTATION_PROTOCOL_VERSION: u32 = 2;
pub const MAX_PRESENTATION_FRAME_BYTES: usize = 8 * 1024 * 1024;

pub struct PresentationHub {
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
        tokio::spawn(serve_connections(
            listener,
            latest_rx,
            Arc::new(Mutex::new(critical_rx)),
            input_tx.clone(),
            connected.clone(),
            disconnected_at_ms.clone(),
        ));
        Ok(Self {
            latest_tx,
            critical_tx,
            input_rx,
            input_tx,
            connected,
            disconnected_at_ms,
        })
    }

    pub fn publish_latest(&self, envelope: RendererIpcEnvelope) {
        self.latest_tx.send_replace(Some(Arc::new(envelope)));
    }

    pub async fn publish_critical(
        &self,
        envelope: RendererIpcEnvelope,
    ) -> Result<(), ClientRuntimeError> {
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
    latest_rx: watch::Receiver<Option<Arc<RendererIpcEnvelope>>>,
    critical_rx: Arc<Mutex<mpsc::Receiver<RendererIpcEnvelope>>>,
    input_tx: mpsc::Sender<RendererInput>,
    connected: Arc<AtomicBool>,
    disconnected_at_ms: Arc<AtomicU64>,
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
        connected.store(true, Ordering::Relaxed);
        let connection_flag = connected.clone();
        let disconnected = disconnected_at_ms.clone();
        tokio::spawn(async move {
            if let Err(error) = serve_renderer(stream, latest, critical, inputs).await {
                log::warn!("renderer IPC disconnected: {error}");
            }
            connection_flag.store(false, Ordering::Relaxed);
            disconnected.store(now_ms(), Ordering::Relaxed);
        });
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

async fn serve_renderer(
    stream: TcpStream,
    mut latest_rx: watch::Receiver<Option<Arc<RendererIpcEnvelope>>>,
    critical_rx: Arc<Mutex<mpsc::Receiver<RendererIpcEnvelope>>>,
    input_tx: mpsc::Sender<RendererInput>,
) -> Result<(), ClientRuntimeError> {
    let (mut reader, mut writer) = stream.into_split();
    let initial = { latest_rx.borrow_and_update().clone() };
    if let Some(latest) = initial {
        write_envelope(&mut writer, &latest).await?;
    }
    loop {
        tokio::select! {
            biased;
            // Critical frames are assigned their sequence before later watch
            // snapshots. Drain them first so a coalesced newer snapshot cannot
            // overtake an input result and make the renderer reject the stream.
            critical = async { critical_rx.lock().await.recv().await } => {
                let Some(critical) = critical else { return Ok(()); };
                write_envelope(&mut writer, &critical).await?;
            }
            read = read_envelope(&mut reader) => {
                let envelope = read?;
                match envelope.payload {
                    Some(renderer_ipc_envelope::Payload::RendererInput(input)) => {
                        input_tx.send(input).await.map_err(|_| ClientRuntimeError::Ipc("input bridge closed".into()))?;
                    }
                    Some(renderer_ipc_envelope::Payload::RendererShutdown(_)) => return Ok(()),
                    Some(renderer_ipc_envelope::Payload::RendererReady(_)) |
                    Some(renderer_ipc_envelope::Payload::RendererConsumed(_)) => {}
                    _ => return Err(ClientRuntimeError::Ipc("renderer sent server-only payload".into())),
                }
            }
            changed = latest_rx.changed() => {
                changed.map_err(|_| ClientRuntimeError::Ipc("presentation source closed".into()))?;
                let latest = latest_rx.borrow().clone();
                if let Some(latest) = latest { write_envelope(&mut writer, &latest).await?; }
            }
        }
    }
}

pub async fn read_envelope(
    reader: &mut (impl AsyncReadExt + Unpin),
) -> Result<RendererIpcEnvelope, ClientRuntimeError> {
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
    Ok(envelope)
}

pub async fn write_envelope(
    writer: &mut (impl AsyncWriteExt + Unpin),
    envelope: &RendererIpcEnvelope,
) -> Result<(), ClientRuntimeError> {
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
    Ok(())
}

pub fn ready_envelope(
    sequence: u64,
    config: &ClientRuntimeConfig,
    server_tick: u64,
    replica_tick: u64,
) -> RendererIpcEnvelope {
    envelope(
        sequence,
        renderer_ipc_envelope::Payload::RuntimeReady(RuntimeReadyPresentation {
            player_id: config.player_id,
            team_id: config.team_id,
            authoritative_tick: server_tick,
            replica_tick,
            content_hash: config.content_hash.clone(),
        }),
    )
}

pub fn snapshot_envelope(
    sequence: u64,
    authoritative_tick: u64,
    view_epoch: u64,
    snapshot: FilteredRenderSnapshot,
    runtime_rtt_us: u64,
) -> RendererIpcEnvelope {
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
        derive_demo_fog(&entities, snapshot.team_id);
    envelope(
        sequence,
        renderer_ipc_envelope::Payload::Snapshot(TeamPresentationSnapshot {
            team_id: snapshot.team_id,
            authoritative_tick,
            replica_tick: snapshot.replica_tick,
            visibility_digest,
            entities,
            removed_render_ids: Vec::new(),
            remembered_ghosts: Vec::new(),
            fog_tiles,
            vision_circles,
            tree_occluders,
            polygon_occluders,
            effects: Vec::new(),
            audio_cues: Vec::new(),
            view_epoch,
            runtime_rtt_us,
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

fn derive_demo_fog(
    entities: &[PresentationRenderEntity],
    team_id: u32,
) -> (
    Vec<FogTilePresentation>,
    Vec<VisionCirclePresentation>,
    Vec<TreeOccluderPresentation>,
    Vec<PolygonOccluderPresentation>,
) {
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
        };
        let bytes = snapshot_envelope(1, 2, 1, snapshot, 0).encode_to_vec();
        let decoded = RendererIpcEnvelope::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.magic, PRESENTATION_MAGIC);
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
}
