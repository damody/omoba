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
                // P9: GameEvent envelope is gone — server now wraps legacy
                // grpc-side payloads in `LegacyJson`. Decode and recover
                // (msg_type, action, data) from the variant.
                let Some(payload) = event.payload else { continue };
                let (msg_type, action, data, payload_bytes) = match payload {
                    game_event::Payload::LegacyJson(m) => {
                        let pb = m.data_json.len();
                        let d = if m.data_json.is_empty() {
                            serde_json::Value::Null
                        } else {
                            serde_json::from_slice(&m.data_json).unwrap_or(serde_json::Value::Null)
                        };
                        (m.msg_type, m.action, d, pb)
                    }
                    // gRPC path uses LegacyJson exclusively today; if a future
                    // server wire-up adds typed variants here, fall back to a
                    // best-effort empty payload.
                    _ => (String::new(), String::new(), serde_json::Value::Null, 0),
                };
                let timestamp_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);

                let parsed = GameEventData {
                    topic: "td/all/res".to_string(),
                    msg_type,
                    action,
                    data,
                    timestamp_ms,
                    payload_bytes,
                    // gRPC 路徑沒有 LZ4 壓縮層，wire bytes ≈ logical bytes。
                    wire_bytes: payload_bytes,
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
    /// Logical (decompressed) payload bytes — 應用層 size。
    pub payload_bytes: usize,
    /// 真實 wire bytes — gRPC 路徑無 LZ4，wire ≈ logical。
    pub wire_bytes: usize,
}
