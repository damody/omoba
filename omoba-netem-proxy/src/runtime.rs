use crate::{
    config::ProxyConfig,
    control::{self, SharedProfiles},
    delay::Direction,
    evidence,
    outage::OutageSchedule,
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
    let outages = Arc::new(OutageSchedule::new(started, config.outages.clone()));
    let profiles: SharedProfiles = Arc::new([
        Arc::new(RwLock::new(config.routes[0].initial_profile.clone())),
        Arc::new(RwLock::new(config.routes[1].initial_profile.clone())),
    ]);
    // Bind control before announcing ready. Route binds happen at task start and are checked by a short readiness grace.
    let control_config = config.clone();
    let control_profiles = profiles.clone();
    let control_evidence = evidence.clone();
    let control_shutdown = shutdown_tx.clone();
    let control_outages = outages.clone();
    let mut control = tokio::spawn(async move {
        control::serve(
            &control_config,
            control_profiles,
            control_evidence,
            control_outages,
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
        outages.clone(),
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
        outages.clone(),
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
        evidence::write(&config, &evidence, &[], &outages.evidence(), &[]).await?;
        return Err(NetemError::Bind(reason));
    }
    // Outage windows count from readiness unless a control `anchor` (e.g. all clients admitted) re-arms them first.
    outages.anchor_if_unset(Instant::now());
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
    let mut metrics: Vec<(crate::delay::RouteId, Direction, QueueMetrics, u64)> = Vec::new();
    let mut endpoint_changes = Vec::new();
    for result in results {
        endpoint_changes.push((result.route, result.endpoint_changes.clone(), result.nat_flows.clone()));
        metrics.push((result.route, Direction::ClientToServer, result.upstream, result.lost_client_to_server));
        metrics.push((result.route, Direction::ServerToClient, result.downstream, result.lost_server_to_client));
    }
    let outage_rows = outages.evidence();
    for row in &outage_rows {
        println!("netem-proxy outage summary team={} scheduled_at_ms={} duration_ms={} anchor_ms={:?} actual_start_ms={:?} actual_end_ms={:?} dropped_client_to_server={} dropped_server_to_client={}",
            row.team_id, row.scheduled_at_ms, row.scheduled_duration_ms, row.anchor_ms, row.actual_start_ms, row.actual_end_ms,
            row.dropped_client_to_server, row.dropped_server_to_client);
    }
    evidence::write(&config, &evidence, &metrics, &outage_rows, &endpoint_changes).await?;
    println!("netem-proxy stopped status={} evidence={}", if failure.is_some() { "FAIL" } else { "PASS" },
        config.evidence_dir.join("proxy-evidence.json").display());
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
