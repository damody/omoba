use std::{net::SocketAddr, sync::Arc, time::Duration};

use tokio::{
    net::UdpSocket,
    sync::{watch, RwLock},
    time::{sleep_until, Instant},
};

use crate::{
    config::{DelayMode, RouteConfig},
    delay::{DelaySampler, Direction, RouteId},
    profile::DelayProfile,
    queue::{DelayQueue, QueueMetrics},
    NetemError, Result,
};

#[derive(Debug)]
pub struct RouteResult {
    pub route: RouteId,
    pub upstream: QueueMetrics,
    pub downstream: QueueMetrics,
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
    let upstream_socket = UdpSocket::bind(config.upstream_bind)
        .await
        .map_err(|e| NetemError::Bind(format!("team {} upstream socket: {e}", config.team_id)))?;
    upstream_socket
        .connect(server_addr)
        .await
        .map_err(|e| NetemError::Route(e.to_string()))?;
    let mut sampler = DelaySampler::new(seed, route);
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
    let mut client_endpoint: Option<SocketAddr> = None;
    let mut client_buffer = vec![0_u8; 65536];
    let mut server_buffer = vec![0_u8; 65536];
    loop {
        let wake = next_wake(upstream.next_deadline(), downstream.next_deadline());
        tokio::select! {
            changed=shutdown.changed()=>{if changed.is_err()||*shutdown.borrow(){break}}
            recv=client_socket.recv_from(&mut client_buffer)=>{
                let(count,peer)=match recv {
                    Ok(value)=>value,
                    Err(error) if error.raw_os_error()==Some(10054)=>continue,
                    Err(error)=>return Err(NetemError::Route(error.to_string())),
                };
                if let Some(expected)=client_endpoint{if peer!=expected{return Err(NetemError::Route(format!("team {} client endpoint changed",config.team_id)))}}else{client_endpoint=Some(peer)}
                let active=profile.read().await.clone();let sample=sampler.sample(Direction::ClientToServer,&active)?;
                upstream.enqueue(Instant::now(),sample.client_to_server_ms,sample.rtt_ms,sample.bucket,client_buffer[..count].to_vec(),None)?;
            }
            recv=upstream_socket.recv(&mut server_buffer)=>{
                let count=match recv {
                    Ok(count)=>count,
                    Err(error) if error.raw_os_error()==Some(10054)=>continue,
                    Err(error)=>return Err(NetemError::Route(error.to_string())),
                };let target=client_endpoint.ok_or_else(||NetemError::Route(format!("team {} server reply before client endpoint",config.team_id)))?;
                let active=profile.read().await.clone();let sample=sampler.sample(Direction::ServerToClient,&active)?;
                downstream.enqueue(Instant::now(),sample.server_to_client_ms,sample.rtt_ms,sample.bucket,server_buffer[..count].to_vec(),Some(target))?;
            }
            _=sleep_until(wake)=>{}
        }
        let now = Instant::now();
        upstream.check_watchdog(now)?;
        downstream.check_watchdog(now)?;
        while let Some(value) = upstream.pop_due(now) {
            upstream_socket
                .send(&value.bytes)
                .await
                .map_err(|e| NetemError::Route(e.to_string()))?;
        }
        while let Some(value) = downstream.pop_due(now) {
            let target = value
                .target
                .ok_or_else(|| NetemError::Route("downstream target missing".into()))?;
            client_socket
                .send_to(&value.bytes, target)
                .await
                .map_err(|e| NetemError::Route(e.to_string()))?;
        }
    }
    let drain_deadline = Instant::now() + watchdog;
    while (!upstream.is_empty() || !downstream.is_empty()) && Instant::now() < drain_deadline {
        let now = Instant::now();
        while let Some(value) = upstream.pop_due(now) {
            upstream_socket
                .send(&value.bytes)
                .await
                .map_err(|e| NetemError::Route(e.to_string()))?;
        }
        while let Some(value) = downstream.pop_due(now) {
            client_socket
                .send_to(
                    &value.bytes,
                    value
                        .target
                        .ok_or_else(|| NetemError::Route("downstream target missing".into()))?,
                )
                .await
                .map_err(|e| NetemError::Route(e.to_string()))?;
        }
        if !upstream.is_empty() || !downstream.is_empty() {
            sleep_until(
                next_wake(upstream.next_deadline(), downstream.next_deadline()).min(drain_deadline),
            )
            .await
        }
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
    })
}

fn next_wake(a: Option<Instant>, b: Option<Instant>) -> Instant {
    match (a, b) {
        (Some(a), Some(b)) => a.min(b),
        (Some(v), None) | (None, Some(v)) => v,
        (None, None) => Instant::now() + Duration::from_secs(3600),
    }
}
