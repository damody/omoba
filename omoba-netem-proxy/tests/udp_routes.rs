use omoba_netem_proxy::{
    config::{DelayMode, RouteConfig},
    profile::DelayProfile,
    route::run_route,
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
async fn route_rejects_a_second_client_endpoint() {
    let server = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let server_addr = server.local_addr().unwrap();
    let client_bind = free_addr().await;
    let upstream = free_addr().await;
    let (_shutdown_tx, shutdown_rx) = watch::channel(false);
    let config = RouteConfig {
        team_id: 1,
        client_bind,
        upstream_bind: upstream,
        initial_profile: DelayProfile::named("fixed-20").unwrap(),
    };
    let route = tokio::spawn(run_route(
        config,
        server_addr,
        DelayMode::OrderedDelay,
        9,
        64,
        1024 * 1024,
        Duration::from_secs(2),
        Arc::new(RwLock::new(DelayProfile::named("fixed-20").unwrap())),
        shutdown_rx,
    ));
    let first = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let second = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    first.send_to(b"first", client_bind).await.unwrap();
    second.send_to(b"second", client_bind).await.unwrap();
    let error = tokio::time::timeout(Duration::from_secs(1), route)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert!(error.to_string().contains("client endpoint changed"));
}
