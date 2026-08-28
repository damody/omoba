use crate::{
    config::ProxyConfig,
    control::{self, SharedProfiles},
    delay::Direction,
    evidence,
    queue::QueueMetrics,
    route::{run_route, RouteResult},
    NetemError, Result,
};
use std::{sync::Arc, time::Instant};
use tokio::sync::{watch, RwLock};

pub async fn run(config: ProxyConfig) -> Result<()> {
    let started = Instant::now();
    let evidence = evidence::shared();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let profiles: SharedProfiles = Arc::new([
        Arc::new(RwLock::new(config.routes[0].initial_profile.clone())),
        Arc::new(RwLock::new(config.routes[1].initial_profile.clone())),
    ]);
    // Bind control before announcing ready. Route binds happen at task start and are checked by a short readiness grace.
    let control_config = config.clone();
    let control_profiles = profiles.clone();
    let control_evidence = evidence.clone();
    let control_shutdown = shutdown_tx.clone();
    let mut control = tokio::spawn(async move {
        control::serve(
            &control_config,
            control_profiles,
            control_evidence,
            started,
            control_shutdown,
        )
        .await
    });
    let mut route1 = tokio::spawn(run_route(
        config.routes[0].clone(),
        config.server_addr,
        config.mode,
        config.seed,
        config.max_datagrams,
        config.max_bytes,
        config.watchdog,
        profiles[0].clone(),
        shutdown_rx.clone(),
    ));
    let mut route2 = tokio::spawn(run_route(
        config.routes[1].clone(),
        config.server_addr,
        config.mode,
        config.seed,
        config.max_datagrams,
        config.max_bytes,
        config.watchdog,
        profiles[1].clone(),
        shutdown_rx.clone(),
    ));
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    if control.is_finished() || route1.is_finished() || route2.is_finished() {
        let reason = "proxy socket task exited before readiness".to_owned();
        evidence.lock().await.failure = Some(reason.clone());
        let _ = shutdown_tx.send(true);
        control.abort();
        route1.abort();
        route2.abort();
        evidence::write(&config, &evidence, &[]).await?;
        return Err(NetemError::Bind(reason));
    }
    println!(
        "netem-proxy ready team1={} team2={} control={} mode={:?} seed={}",
        config.routes[0].client_bind,
        config.routes[1].client_bind,
        config.control_bind,
        config.mode,
        config.seed
    );
    let mut failure = None;
    let results: Vec<RouteResult> = tokio::select! {
        signal=tokio::signal::ctrl_c()=>{signal.map_err(|e|NetemError::Route(e.to_string()))?;let _=shutdown_tx.send(true);control.abort();join_routes(route1,route2).await?}
        value=&mut control=>{match value{Ok(Ok(()))=>{},Ok(Err(e))=>failure=Some(e.to_string()),Err(e)=>failure=Some(e.to_string())};let _=shutdown_tx.send(true);join_routes(route1,route2).await?}
        value=&mut route1=>{failure=Some(route_failure(1,value));let _=shutdown_tx.send(true);control.abort();let mut out=Vec::new();if let Ok(Ok(result))=route2.await{out.push(result)}out}
        value=&mut route2=>{failure=Some(route_failure(2,value));let _=shutdown_tx.send(true);control.abort();let mut out=Vec::new();if let Ok(Ok(result))=route1.await{out.push(result)}out}
    };
    if let Some(reason) = failure.clone() {
        evidence.lock().await.failure = Some(reason)
    }
    let mut metrics: Vec<(crate::delay::RouteId, Direction, QueueMetrics)> = Vec::new();
    for result in results {
        metrics.push((result.route, Direction::ClientToServer, result.upstream));
        metrics.push((result.route, Direction::ServerToClient, result.downstream));
    }
    evidence::write(&config, &evidence, &metrics).await?;
    if let Some(reason) = failure {
        return Err(NetemError::Route(reason));
    }
    Ok(())
}

async fn join_routes(
    a: tokio::task::JoinHandle<Result<RouteResult>>,
    b: tokio::task::JoinHandle<Result<RouteResult>>,
) -> Result<Vec<RouteResult>> {
    let a = a.await.map_err(|e| NetemError::Route(e.to_string()))??;
    let b = b.await.map_err(|e| NetemError::Route(e.to_string()))??;
    Ok(vec![a, b])
}
fn route_failure(
    team: u32,
    value: std::result::Result<Result<RouteResult>, tokio::task::JoinError>,
) -> String {
    match value {
        Ok(Ok(_)) => format!("team {team} route exited before shutdown"),
        Ok(Err(e)) => e.to_string(),
        Err(e) => e.to_string(),
    }
}
