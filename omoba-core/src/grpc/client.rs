use anyhow::Result;
use log::*;
use tokio::sync::mpsc;

use super::game_proto::game_service_client::GameServiceClient;
use super::game_proto::*;

/// gRPC client for communicating with the omb game server.
pub struct GrpcClient {
    client: GameServiceClient<tonic::transport::Channel>,
    player_name: String,
}

impl GrpcClient {
    /// Connect to the gRPC game server.
    pub async fn connect(addr: &str, player_name: String) -> Result<Self> {
        let client = GameServiceClient::connect(addr.to_string()).await?;
        info!("Connected to gRPC server at {}", addr);
        Ok(Self {
            client,
            player_name,
        })
    }

    /// Send a player command to the server.
    pub async fn send_command(
        &mut self,
        msg_type: &str,
        action: &str,
        data: serde_json::Value,
    ) -> Result<bool> {
        let data_bytes = serde_json::to_vec(&data)?;
        let request = tonic::Request::new(PlayerCommand {
            player_name: self.player_name.clone(),
            msg_type: msg_type.to_string(),
            action: action.to_string(),
            data_json: data_bytes,
        });

        let response = self.client.send_command(request).await?;
        Ok(response.into_inner().ok)
    }

    /// Subscribe to game events from the server.
    /// Returns a receiver channel that yields parsed game events.
    pub async fn subscribe_events(
        &mut self,
    ) -> Result<mpsc::Receiver<GameEventData>> {
        let request = tonic::Request::new(SubscribeRequest {
            player_name: self.player_name.clone(),
        });

        let mut stream = self.client.subscribe_events(request).await?.into_inner();
        let (tx, rx) = mpsc::channel(10000);

        tokio::spawn(async move {
            while let Ok(Some(event)) = stream.message().await {
                let payload_bytes = event.data_json.len();
                let data = if event.data_json.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::from_slice(&event.data_json).unwrap_or(serde_json::Value::Null)
                };

                let parsed = GameEventData {
                    topic: event.topic,
                    msg_type: event.msg_type,
                    action: event.action,
                    data,
                    timestamp_ms: event.timestamp_ms,
                    payload_bytes,
                };

                if tx.send(parsed).await.is_err() {
                    break;
                }
            }
        });

        Ok(rx)
    }
}

/// Parsed game event data for client consumption.
#[derive(Debug, Clone)]
pub struct GameEventData {
    pub topic: String,
    pub msg_type: String,
    pub action: String,
    pub data: serde_json::Value,
    pub timestamp_ms: u64,
    /// 原始 proto data_json bytes 長度；供前端網路吞吐統計用。
    pub payload_bytes: usize,
}
