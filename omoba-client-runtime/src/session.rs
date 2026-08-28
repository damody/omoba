use omoba_core::{
    game_proto::TeamGameStart,
    kcp::client::{KcpClient, LockstepInbound},
};
use tokio::sync::mpsc;

use crate::{config::ClientRuntimeConfig, ClientRuntimeError};

pub struct SelectiveSession {
    pub client: KcpClient,
    pub inbound: mpsc::Receiver<LockstepInbound>,
    pub start: TeamGameStart,
}

impl SelectiveSession {
    pub async fn connect(config: &ClientRuntimeConfig) -> Result<Self, ClientRuntimeError> {
        let mut client =
            KcpClient::connect(&config.server_addr.to_string(), config.player_name.clone())
                .await
                .map_err(|error| ClientRuntimeError::Session(error.to_string()))?;
        let start = client
            .join_selective_lockstep(config.player_name.clone(), config.player_id)
            .await
            .map_err(|error| ClientRuntimeError::Session(error.to_string()))?;
        if start.team_id != config.team_id {
            return Err(ClientRuntimeError::Session(format!(
                "secure bootstrap team mismatch: configured={} received={}",
                config.team_id, start.team_id
            )));
        }
        if start.protocol_version != config.protocol_version {
            return Err(ClientRuntimeError::Session(
                "secure protocol downgrade rejected".into(),
            ));
        }
        let inbound = client
            .subscribe_lockstep()
            .map_err(|error| ClientRuntimeError::Session(error.to_string()))?;
        Ok(Self {
            client,
            inbound,
            start,
        })
    }
}
