use omoba_netem_proxy::{
    config::{DelayMode, RouteConfig},
    outage::{OutageSchedule, OutageWindow},
    profile::DelayProfile,
    route::{run_route, run_route_with_idle},
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::{
    net::UdpSocket,
    sync::{watch, RwLock},
};

async fn round_trip(client: &UdpSocket, route: SocketAddr) -> bool {
    let mut reply = [0_u8; 32];
    client.send_to(b"ping", route).await.unwrap();
    tokio::time::timeout(Duration::from_millis(200), client.recv(&mut reply)).await.is_ok()
}

fn no_outages() -> Arc<OutageSchedule> {
    Arc::new(OutageSchedule::new(std::time::Instant::now(), Vec::new()))
}

async fn free_addr() -> SocketAddr {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    socket.local_addr().unwrap()
}

#[tokio::test]
async fn two_routes_keep_client_endpoints_isolated() {
    let server = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let server_addr = server.local_addr().unwrap();
    let echo = tokio::spawn(async move {
        let mut bytes = [0_u8; 128];
        for _ in 0..2 {
            let (n, peer) = server.recv_from(&mut bytes).await.unwrap();
            server.send_to(&bytes[..n], peer).await.unwrap();
        }
    });
    let client_bind = [free_addr().await, free_addr().await];
    let upstream = [free_addr().await, free_addr().await];
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let mut tasks = Vec::new();
    for index in 0..2 {
        let config = RouteConfig {
            team_id: (index + 1) as u32,
            client_bind: client_bind[index],
            upstream_bind: upstream[index],
            initial_profile: DelayProfile::named("fixed-20").unwrap(),
        };
        tasks.push(tokio::spawn(run_route(
            config,
            server_addr,
            DelayMode::OrderedDelay,
            44,
            64,
            1024 * 1024,
            Duration::from_secs(2),
            Arc::new(RwLock::new(DelayProfile::named("fixed-20").unwrap())),
            no_outages(),
            shutdown_rx.clone(),
        )))
    }
    let clients = [
        UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
            .await
            .unwrap(),
        UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
            .await
            .unwrap(),
    ];
    clients[0]
        .send_to(b"team-one", client_bind[0])
        .await
        .unwrap();
    clients[1]
        .send_to(b"team-two", client_bind[1])
        .await
        .unwrap();
    let mut a = [0_u8; 32];
    let mut b = [0_u8; 32];
    let an = tokio::time::timeout(Duration::from_secs(2), clients[0].recv(&mut a))
        .await
        .unwrap()
        .unwrap();
    let bn = tokio::time::timeout(Duration::from_secs(2), clients[1].recv(&mut b))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&a[..an], b"team-one");
    assert_eq!(&b[..bn], b"team-two");
    shutdown_tx.send(true).unwrap();
    for task in tasks {
        let result = task.await.unwrap().unwrap();
        assert_eq!(result.upstream.released, 1);
        assert_eq!(result.downstream.released, 1)
    }
    echo.await.unwrap();
}

#[tokio::test]
async fn each_client_endpoint_gets_its_own_upstream_mapping_and_idle_mappings_are_reclaimed() {
    // Echo server: replies to every datagram and reports which address sent it.
    let server = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let server_addr = server.local_addr().unwrap();
    let (seen_tx, mut seen_rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut bytes = [0_u8; 128];
        loop {
            let Ok((n, peer)) = server.recv_from(&mut bytes).await else { continue };
            let _ = seen_tx.send(peer);
            let _ = server.send_to(&bytes[..n], peer).await;
        }
    });
    let client_bind = free_addr().await;
    let upstream = free_addr().await;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let config = RouteConfig {
        team_id: 2,
        client_bind,
        upstream_bind: upstream,
        initial_profile: DelayProfile::named("fixed-20").unwrap(),
    };
    let idle = Duration::from_millis(400);
    let route = tokio::spawn(run_route_with_idle(
        config,
        server_addr,
        DelayMode::OrderedDelay,
        9,
        64,
        1024 * 1024,
        Duration::from_secs(2),
        Arc::new(RwLock::new(DelayProfile::named("fixed-20").unwrap())),
        no_outages(),
        idle,
        shutdown_rx,
    ));
    // A reconnecting client binds a new socket (new source port) while the old
    // one may still be sending.
    let first = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let second = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    assert!(round_trip(&first, client_bind).await, "first endpoint is served");
    assert_eq!(seen_rx.recv().await.unwrap(), upstream, "the first mapping uses the configured upstream");
    assert!(round_trip(&second, client_bind).await, "second endpoint is served");
    let second_upstream = seen_rx.recv().await.unwrap();
    assert_ne!(second_upstream, upstream, "a new endpoint reaches the server from a new address");
    // The old endpoint's traffic stays on its own mapping: its reply goes back
    // to it, and the new endpoint receives nothing from it.
    assert!(round_trip(&first, client_bind).await, "old endpoint still answered on its own mapping");
    assert_eq!(seen_rx.recv().await.unwrap(), upstream);
    let mut stray = [0_u8; 32];
    assert!(tokio::time::timeout(Duration::from_millis(150), second.recv(&mut stray)).await.is_err(),
        "old-endpoint replies never reach the new endpoint");
    // The old endpoint goes silent; the new one keeps talking past the idle timeout.
    let quiet_until = tokio::time::Instant::now() + idle * 2;
    while tokio::time::Instant::now() < quiet_until {
        assert!(round_trip(&second, client_bind).await);
        assert_eq!(seen_rx.recv().await.unwrap(), second_upstream);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    shutdown_tx.send(true).unwrap();
    let result = route.await.unwrap().unwrap();
    assert_eq!(result.endpoint_changes.len(), 1);
    assert_eq!(result.endpoint_changes[0].from, first.local_addr().unwrap());
    assert_eq!(result.endpoint_changes[0].to, second.local_addr().unwrap());
    assert_eq!(result.endpoint_changes[0].upstream, second_upstream);
    assert_eq!(result.nat_flows.len(), 2);
    let old = &result.nat_flows[0];
    assert_eq!((old.endpoint, old.upstream), (first.local_addr().unwrap(), upstream));
    assert!(old.reclaimed_ms.is_some(), "the silent mapping is reclaimed");
    assert_eq!((old.client_to_server, old.server_to_client), (2, 2));
    let new = &result.nat_flows[1];
    assert_eq!(new.upstream, second_upstream);
    assert!(new.reclaimed_ms.is_none(), "an active mapping is kept");
}
#[tokio::test]
async fn team_outage_drops_both_directions_only_inside_its_window() {
    // Echo server: replies to every datagram it receives.
    let server = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let server_addr = server.local_addr().unwrap();
    let received = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let counter = received.clone();
    tokio::spawn(async move {
        let mut bytes = [0_u8; 128];
        loop {
            let (n, peer) = server.recv_from(&mut bytes).await.unwrap();
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            server.send_to(&bytes[..n], peer).await.unwrap();
        }
    });
    let client_bind = free_addr().await;
    let upstream = free_addr().await;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let started = std::time::Instant::now();
    let outages = Arc::new(OutageSchedule::new(started, vec![OutageWindow { team_id: 1, at_ms: 300, duration_ms: 600 }]));
    outages.anchor(started).unwrap();
    let route = tokio::spawn(run_route(
        RouteConfig { team_id: 1, client_bind, upstream_bind: upstream, initial_profile: DelayProfile::named("fixed-20").unwrap() },
        server_addr, DelayMode::OrderedDelay, 3, 64, 1024 * 1024, Duration::from_secs(2),
        Arc::new(RwLock::new(DelayProfile::named("fixed-20").unwrap())), outages.clone(), shutdown_rx,
    ));
    let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    assert!(round_trip(&client, client_bind).await, "before the window the route forwards");
    tokio::time::sleep_until(tokio::time::Instant::from_std(started + Duration::from_millis(350))).await;
    let before = received.load(std::sync::atomic::Ordering::SeqCst);
    for _ in 0..3 { assert!(!round_trip(&client, client_bind).await, "blackout drops the request"); }
    assert_eq!(received.load(std::sync::atomic::Ordering::SeqCst), before, "nothing reaches the server");
    tokio::time::sleep_until(tokio::time::Instant::from_std(started + Duration::from_millis(950))).await;
    assert!(round_trip(&client, client_bind).await, "after the window the route forwards again");
    shutdown_tx.send(true).unwrap();
    route.await.unwrap().unwrap();
    let row = &outages.evidence()[0];
    assert_eq!(row.dropped_client_to_server, 3);
    assert!(row.actual_start_ms.is_some_and(|ms| (300..400).contains(&ms)), "{row:?}");
    assert!(row.actual_end_ms.is_some_and(|ms| (900..1000).contains(&ms)), "{row:?}");
}
