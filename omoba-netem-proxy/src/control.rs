use crate::{
    config::ProxyConfig,
    evidence::{record_switch, SharedEvidence},
    profile::DelayProfile,
    NetemError, Result,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Instant};
use tokio::{
    net::UdpSocket,
    sync::{watch, RwLock},
};

pub const CONTROL_VERSION: u32 = 1;
pub type SharedProfiles = Arc<[Arc<RwLock<DelayProfile>>; 2]>;

#[derive(Debug, Deserialize, Serialize)]
pub struct ControlMessage {
    pub version: u32,
    pub action: String,
    pub team_id: Option<u32>,
    pub profile: Option<String>,
    pub weights: Option<Vec<u64>>,
    pub authoritative_tick: Option<u64>,
}

pub async fn serve(
    config: &ProxyConfig,
    profiles: SharedProfiles,
    evidence: SharedEvidence,
    started: Instant,
    shutdown: watch::Sender<bool>,
) -> Result<()> {
    let socket = UdpSocket::bind(config.control_bind)
        .await
        .map_err(|e| NetemError::Bind(e.to_string()))?;
    let mut buffer = vec![0; 8192];
    loop {
        let (count, peer) = socket
            .recv_from(&mut buffer)
            .await
            .map_err(|e| NetemError::Route(e.to_string()))?;
        if !peer.ip().is_loopback() {
            continue;
        }
        let message: ControlMessage = serde_json::from_slice(&buffer[..count])
            .map_err(|e| NetemError::Config(e.to_string()))?;
        if message.version != CONTROL_VERSION {
            return Err(NetemError::Config("control version mismatch".into()));
        }
        if message.action == "shutdown" {
            let _ = shutdown.send(true);
            return Ok(());
        }
        if message.action != "profile" {
            return Err(NetemError::Config("unknown control action".into()));
        }
        let team = message
            .team_id
            .filter(|v| matches!(v, 1 | 2))
            .ok_or_else(|| NetemError::Config("control team must be 1 or 2".into()))?;
        let name = message
            .profile
            .ok_or_else(|| NetemError::Config("control profile missing".into()))?;
        let profile = if name == "custom-20-bin" {
            DelayProfile::new(
                name,
                message
                    .weights
                    .ok_or_else(|| NetemError::Config("custom weights missing".into()))?,
            )?
        } else {
            DelayProfile::named(&name)?
        };
        *profiles[(team - 1) as usize].write().await = profile.clone();
        record_switch(
            &evidence,
            team,
            &profile,
            started.elapsed().as_millis() as u64,
            message.authoritative_tick.unwrap_or(0),
        )
        .await;
    }
}
