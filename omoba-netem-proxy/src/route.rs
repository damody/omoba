use std::{collections::BTreeMap, net::SocketAddr, sync::Arc, time::Duration};

use tokio::{
    net::UdpSocket,
    sync::{mpsc, watch, RwLock},
    task::JoinHandle,
    time::{sleep_until, Instant},
};

use crate::{
    config::{DelayMode, RouteConfig},
    delay::{DelaySampler, Direction, RouteId},
    outage::{OutageSchedule, Transition},
    profile::DelayProfile,
    queue::{DelayQueue, QueueMetrics},
    NetemError, Result,
};

/// A NAT flow that is idle (no client datagram) this long is reclaimed: its
/// upstream socket closes, so the server sees that address go away.
pub const NAT_IDLE_TIMEOUT: Duration = Duration::from_secs(30);

/// A new client endpoint appeared on a route (ms since the route started):
/// `from` was the latest endpoint, `to` got its own upstream socket `upstream`.
#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct EndpointChange {
    pub route_ms: u64,
    pub from: SocketAddr,
    pub to: SocketAddr,
    pub upstream: SocketAddr,
}

/// One NAT mapping: a client endpoint and the upstream socket that carries it.
#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct NatFlowEvidence {
    pub endpoint: SocketAddr,
    pub upstream: SocketAddr,
    pub opened_ms: u64,
    pub reclaimed_ms: Option<u64>,
    pub client_to_server: u64,
    pub server_to_client: u64,
}

#[derive(Debug)]
pub struct RouteResult {
    pub route: RouteId,
    pub upstream: QueueMetrics,
    pub downstream: QueueMetrics,
    /// Profile (loss_ppm) drops; outage drops are counted per window.
    pub lost_client_to_server: u64,
    pub lost_server_to_client: u64,
    pub endpoint_changes: Vec<EndpointChange>,
    pub nat_flows: Vec<NatFlowEvidence>,
}

/// Per client endpoint state. Like a NAT, every endpoint gets its own upstream
/// socket, so a reconnecting client (new socket, new source port) reaches the
/// server from a new address, and the old socket's traffic stays on the old
/// mapping instead of flipping where replies go.
struct NatFlow {
    upstream: Arc<UdpSocket>,
    reader: JoinHandle<()>,
    last_client_datagram: Instant,
    evidence_index: usize,
}

type ServerDatagram = std::result::Result<(SocketAddr, Vec<u8>), String>;

fn spawn_flow_reader(
    endpoint: SocketAddr,
    upstream: Arc<UdpSocket>,
    tx: mpsc::Sender<ServerDatagram>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut buffer = vec![0_u8; 65536];
        loop {
            let item = match upstream.recv(&mut buffer).await {
                Ok(count) => Ok((endpoint, buffer[..count].to_vec())),
                Err(error) if error.raw_os_error() == Some(10054) => continue,
                Err(error) => Err(error.to_string()),
            };
            let failed = item.is_err();
            if tx.send(item).await.is_err() || failed {
                break;
            }
        }
    })
}

pub async fn run_route(
    config: RouteConfig,
    server_addr: SocketAddr,
    mode: DelayMode,
    seed: u64,
    max_datagrams: usize,
    max_bytes: usize,
    watchdog: Duration,
    profile: Arc<RwLock<DelayProfile>>,
    outages: Arc<OutageSchedule>,
    shutdown: watch::Receiver<bool>,
) -> Result<RouteResult> {
    run_route_with_idle(
        config, server_addr, mode, seed, max_datagrams, max_bytes, watchdog, profile, outages,
        NAT_IDLE_TIMEOUT, shutdown,
    )
    .await
}

pub async fn run_route_with_idle(
    config: RouteConfig,
    server_addr: SocketAddr,
    mode: DelayMode,
    seed: u64,
    max_datagrams: usize,
    max_bytes: usize,
    watchdog: Duration,
    profile: Arc<RwLock<DelayProfile>>,
    outages: Arc<OutageSchedule>,
    nat_idle_timeout: Duration,
    mut shutdown: watch::Receiver<bool>,
) -> Result<RouteResult> {
    let route = if config.team_id == 1 {
        RouteId::Team1
    } else {
        RouteId::Team2
    };
    let client_socket = UdpSocket::bind(config.client_bind)
        .await
        .map_err(|e| NetemError::Bind(format!("team {} client socket: {e}", config.team_id)))?;
    // The configured upstream address is bound up front (startup fails fast if
    // it is taken) and serves the first client endpoint; later endpoints get
    // ephemeral ports on the same interface.
    let mut reserved_upstream = Some(
        UdpSocket::bind(config.upstream_bind)
            .await
            .map_err(|e| NetemError::Bind(format!("team {} upstream socket: {e}", config.team_id)))?,
    );
    let mut sampler = DelaySampler::new(seed, route);
    // Lost datagrams are counted per direction and reported when the route ends.
    let mut dropped_upstream = 0_u64;
    let mut dropped_downstream = 0_u64;
    let mut upstream = DelayQueue::new(
        route,
        Direction::ClientToServer,
        mode,
        max_datagrams,
        max_bytes,
        watchdog,
    );
    let mut downstream = DelayQueue::new(
        route,
        Direction::ServerToClient,
        mode,
        max_datagrams,
        max_bytes,
        watchdog,
    );
    let (server_tx, mut server_rx) = mpsc::channel::<ServerDatagram>(1024);
    let mut flows: BTreeMap<SocketAddr, NatFlow> = BTreeMap::new();
    let mut nat_flows: Vec<NatFlowEvidence> = Vec::new();
    let mut latest_endpoint: Option<SocketAddr> = None;
    let mut endpoint_changes = Vec::new();
    let route_started = std::time::Instant::now();
    let route_ms = || route_started.elapsed().as_millis() as u64;
    let mut client_buffer = vec![0_u8; 65536];
    loop {
        let boundary = outages.next_boundary(config.team_id, Instant::now().into_std()).map(Instant::from_std);
        let reclaim = flows.values().map(|flow| flow.last_client_datagram + nat_idle_timeout).min();
        let wake = next_wake(
            next_wake_opt(next_wake_opt(upstream.next_deadline(), downstream.next_deadline()), reclaim),
            boundary,
        );
        tokio::select! {
            changed=shutdown.changed()=>{if changed.is_err()||*shutdown.borrow(){break}}
            recv=client_socket.recv_from(&mut client_buffer)=>{
                let(count,peer)=match recv {
                    Ok(value)=>value,
                    Err(error) if error.raw_os_error()==Some(10054)=>continue,
                    Err(error)=>return Err(NetemError::Route(error.to_string())),
                };
                // Only local clients are routed; a non-loopback peer is never accepted.
                if !peer.ip().is_loopback() {
                    return Err(NetemError::Route(format!("team {} client endpoint {peer} is not loopback", config.team_id)));
                }
                let now = Instant::now();
                if let Some(flow) = flows.get_mut(&peer) {
                    flow.last_client_datagram = now;
                } else {
                    let socket = match reserved_upstream.take() {
                        Some(socket) => socket,
                        None => UdpSocket::bind(SocketAddr::new(config.upstream_bind.ip(), 0))
                            .await
                            .map_err(|e| NetemError::Bind(format!("team {} upstream socket for {peer}: {e}", config.team_id)))?,
                    };
                    socket.connect(server_addr).await.map_err(|e| NetemError::Route(e.to_string()))?;
                    let upstream_addr = socket.local_addr().map_err(|e| NetemError::Route(e.to_string()))?;
                    let socket = Arc::new(socket);
                    let opened_ms = route_ms();
                    if let Some(previous) = latest_endpoint {
                        let change = EndpointChange { route_ms: opened_ms, from: previous, to: peer, upstream: upstream_addr };
                        println!("netem-proxy endpoint change team={} route_ms={} from={} to={} upstream={}",
                            config.team_id, change.route_ms, change.from, change.to, change.upstream);
                        endpoint_changes.push(change);
                    }
                    nat_flows.push(NatFlowEvidence {
                        endpoint: peer, upstream: upstream_addr, opened_ms, reclaimed_ms: None,
                        client_to_server: 0, server_to_client: 0,
                    });
                    flows.insert(peer, NatFlow {
                        reader: spawn_flow_reader(peer, socket.clone(), server_tx.clone()),
                        upstream: socket,
                        last_client_datagram: now,
                        evidence_index: nat_flows.len() - 1,
                    });
                }
                latest_endpoint = Some(peer);
                // Outages cover the whole team: every endpoint of this route.
                if let Some(window)=outages.active(config.team_id,now.into_std()){outages.record_drop(window,Direction::ClientToServer);continue}
                let active=profile.read().await.clone();
                if sampler.drops(Direction::ClientToServer,&active){dropped_upstream+=1;continue}
                let sample=sampler.sample(Direction::ClientToServer,&active)?;
                upstream.enqueue(now,sample.client_to_server_ms,sample.rtt_ms,sample.bucket,client_buffer[..count].to_vec(),Some(peer))?;
            }
            recv=server_rx.recv()=>{
                // `server_tx` stays alive here, so the channel never closes.
                let Some(item)=recv else { continue };
                let (endpoint,bytes)=item.map_err(NetemError::Route)?;
                // A reply on an already reclaimed mapping is dropped like on a real NAT.
                if !flows.contains_key(&endpoint){continue}
                if let Some(window)=outages.active(config.team_id,Instant::now().into_std()){outages.record_drop(window,Direction::ServerToClient);continue}
                let active=profile.read().await.clone();
                if sampler.drops(Direction::ServerToClient,&active){dropped_downstream+=1;continue}
                let sample=sampler.sample(Direction::ServerToClient,&active)?;
                downstream.enqueue(Instant::now(),sample.server_to_client_ms,sample.rtt_ms,sample.bucket,bytes,Some(endpoint))?;
            }
            _=sleep_until(wake)=>{}
        }
        let now = Instant::now();
        for transition in outages.observe(config.team_id, now.into_std()) {
            match transition {
                Transition::Started { window, at_ms } => println!(
                    "netem-proxy outage start team={} window={} at_ms={}", config.team_id, window, at_ms),
                Transition::Ended { window, at_ms, dropped_client_to_server, dropped_server_to_client } => println!(
                    "netem-proxy outage end team={} window={} at_ms={} dropped_client_to_server={} dropped_server_to_client={}",
                    config.team_id, window, at_ms, dropped_client_to_server, dropped_server_to_client),
            }
        }
        // Idle mappings are reclaimed: the old socket of a reconnected client
        // stops sending, so its upstream closes after the idle timeout.
        let idle: Vec<SocketAddr> = flows
            .iter()
            .filter(|(_, flow)| now >= flow.last_client_datagram + nat_idle_timeout)
            .map(|(endpoint, _)| *endpoint)
            .collect();
        for endpoint in idle {
            if let Some(flow) = flows.remove(&endpoint) {
                flow.reader.abort();
                let evidence = &mut nat_flows[flow.evidence_index];
                evidence.reclaimed_ms = Some(route_ms());
                println!("netem-proxy nat reclaim team={} endpoint={} upstream={} route_ms={}",
                    config.team_id, endpoint, evidence.upstream, route_ms());
            }
        }
        // Datagrams queued before an outage are released into it: drop them too.
        let blackout = outages.active(config.team_id, now.into_std());
        upstream.check_watchdog(now)?;
        downstream.check_watchdog(now)?;
        while let Some(value) = upstream.pop_due(now) {
            if let Some(window) = blackout { outages.record_drop(window, Direction::ClientToServer); continue; }
            send_upstream(&mut flows, &mut nat_flows, value.target, &value.bytes).await?;
        }
        while let Some(value) = downstream.pop_due(now) {
            if let Some(window) = blackout { outages.record_drop(window, Direction::ServerToClient); continue; }
            send_downstream(&client_socket, &flows, &mut nat_flows, value.target, &value.bytes).await?;
        }
    }
    println!(
        "netem-proxy route team={} dropped_client_to_server={} dropped_server_to_client={}",
        config.team_id, dropped_upstream, dropped_downstream
    );
    let drain_deadline = Instant::now() + watchdog;
    while (!upstream.is_empty() || !downstream.is_empty()) && Instant::now() < drain_deadline {
        let now = Instant::now();
        while let Some(value) = upstream.pop_due(now) {
            send_upstream(&mut flows, &mut nat_flows, value.target, &value.bytes).await?;
        }
        while let Some(value) = downstream.pop_due(now) {
            send_downstream(&client_socket, &flows, &mut nat_flows, value.target, &value.bytes).await?;
        }
        if !upstream.is_empty() || !downstream.is_empty() {
            sleep_until(
                next_wake(upstream.next_deadline(), downstream.next_deadline()).min(drain_deadline),
            )
            .await
        }
    }
    for flow in flows.values() {
        flow.reader.abort();
    }
    if !upstream.is_empty() || !downstream.is_empty() {
        return Err(NetemError::Watchdog(format!(
            "team {} shutdown drain timeout",
            config.team_id
        )));
    }
    Ok(RouteResult {
        route,
        upstream: upstream.metrics,
        downstream: downstream.metrics,
        lost_client_to_server: dropped_upstream,
        lost_server_to_client: dropped_downstream,
        endpoint_changes,
        nat_flows,
    })
}

/// Sends a client datagram through its endpoint's own upstream socket. A
/// mapping reclaimed while the datagram was delayed drops it.
async fn send_upstream(
    flows: &mut BTreeMap<SocketAddr, NatFlow>,
    nat_flows: &mut [NatFlowEvidence],
    endpoint: Option<SocketAddr>,
    bytes: &[u8],
) -> Result<()> {
    let endpoint = endpoint.ok_or_else(|| NetemError::Route("upstream endpoint missing".into()))?;
    let Some(flow) = flows.get(&endpoint) else { return Ok(()) };
    flow.upstream.send(bytes).await.map_err(|e| NetemError::Route(e.to_string()))?;
    nat_flows[flow.evidence_index].client_to_server += 1;
    Ok(())
}

/// Sends a server datagram back to the endpoint whose mapping received it.
async fn send_downstream(
    client_socket: &UdpSocket,
    flows: &BTreeMap<SocketAddr, NatFlow>,
    nat_flows: &mut [NatFlowEvidence],
    endpoint: Option<SocketAddr>,
    bytes: &[u8],
) -> Result<()> {
    let endpoint = endpoint.ok_or_else(|| NetemError::Route("downstream target missing".into()))?;
    let Some(flow) = flows.get(&endpoint) else { return Ok(()) };
    client_socket.send_to(bytes, endpoint).await.map_err(|e| NetemError::Route(e.to_string()))?;
    nat_flows[flow.evidence_index].server_to_client += 1;
    Ok(())
}

fn next_wake_opt(a: Option<Instant>, b: Option<Instant>) -> Option<Instant> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(v), None) | (None, Some(v)) => Some(v),
        (None, None) => None,
    }
}

fn next_wake(a: Option<Instant>, b: Option<Instant>) -> Instant {
    next_wake_opt(a, b).unwrap_or_else(|| Instant::now() + Duration::from_secs(3600))
}
