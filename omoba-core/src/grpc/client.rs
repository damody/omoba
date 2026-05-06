use anyhow::Result;
use log::*;
use tokio::sync::mpsc;

use super::game_proto::game_service_client::GameServiceClient;
use super::game_proto::*;

/// 用於與 omb 遊戲伺服器通訊的 gRPC 用戶端。
pub struct GrpcClient {
    client: GameServiceClient<tonic::transport::Channel>,
    player_name: String,
}

impl GrpcClient {
    /// 連接到 gRPC 遊戲伺服器。
    pub async fn connect(addr: &str, player_name: String) -> Result<Self> {
        let client = GameServiceClient::connect(addr.to_string()).await?;
        info!("Connected to gRPC server at {}", addr);
        Ok(Self {
            client,
            player_name,
        })
    }

    /// 向伺服器發送玩家命令。
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

    /// 從伺服器訂閱遊戲事件。
    /// 傳回一個接收器通道，該通道產生已解析的遊戲事件。
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
                // P9：GameEvent 信封已消失 - 伺服器現在包裝舊版
                // `LegacyJson` 中的 grpc 端有效負載。解碼並恢復
                // 來自變體的（msg_type、操作、資料）。
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
                    // gRPC 路徑目前僅使用 LegacyJson；如果有未來
                    // 伺服器接線在此處新增類型變體，回退到
                    // 盡最大努力空有效負載。
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

/// 解析遊戲事件資料供客戶端使用。
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
