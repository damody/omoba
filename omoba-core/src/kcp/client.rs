use anyhow::Result;
use log::*;
use prost::Message;
use tokio::sync::mpsc;
use tokio::io::{ReadHalf, WriteHalf};
use std::sync::Arc;
use tokio::sync::Mutex;

use tokio_kcp::{KcpConfig, KcpStream, KcpNoDelayConfig};

use super::framing::*;
use super::game_proto::*;

/// KCP client for communicating with the omb game server.
pub struct KcpClient {
    player_name: String,
    writer: Arc<Mutex<WriteHalf<KcpStream>>>,
    event_rx: Option<mpsc::Receiver<GameEventData>>,
}

/// Parsed game event data for client consumption.
#[derive(Debug, Clone)]
pub struct GameEventData {
    pub topic: String,
    pub msg_type: String,
    pub action: String,
    pub data: serde_json::Value,
    pub timestamp_ms: u64,
}

impl KcpClient {
    /// Connect to the KCP game server.
    pub async fn connect(addr: &str, player_name: String) -> Result<Self> {
        let mut config = KcpConfig::default();
        config.nodelay = KcpNoDelayConfig::fastest();

        let sock_addr: std::net::SocketAddr = addr.parse()?;
        let stream = KcpStream::connect(&config, sock_addr).await?;
        info!("Connected to KCP server at {}", addr);

        let (reader, writer) = tokio::io::split(stream);
        let writer = Arc::new(Mutex::new(writer));

        // Send subscribe request immediately
        {
            let mut w = writer.lock().await;
            let sub = SubscribeRequest {
                player_name: player_name.clone(),
            };
            write_framed_msg(&mut *w, TAG_SUBSCRIBE_REQUEST, &sub).await?;
        }

        // Spawn background reader task
        let (event_tx, event_rx) = mpsc::channel(10000);
        Self::spawn_reader(reader, event_tx);

        Ok(Self {
            player_name,
            writer,
            event_rx: Some(event_rx),
        })
    }

    fn spawn_reader(
        mut reader: ReadHalf<KcpStream>,
        event_tx: mpsc::Sender<GameEventData>,
    ) {
        tokio::spawn(async move {
            loop {
                match read_framed(&mut reader).await {
                    Ok(Some((tag, payload))) => {
                        match tag {
                            TAG_GAME_EVENT => {
                                match GameEvent::decode(payload.as_slice()) {
                                    Ok(event) => {
                                        let data = if event.data_json.is_empty() {
                                            serde_json::Value::Null
                                        } else {
                                            serde_json::from_slice(&event.data_json)
                                                .unwrap_or(serde_json::Value::Null)
                                        };

                                        let parsed = GameEventData {
                                            topic: event.topic,
                                            msg_type: event.msg_type,
                                            action: event.action,
                                            data,
                                            timestamp_ms: event.timestamp_ms,
                                        };

                                        if event_tx.send(parsed).await.is_err() {
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to decode GameEvent: {}", e);
                                    }
                                }
                            }
                            TAG_COMMAND_ACK => {
                                // CommandAck — currently ignored
                            }
                            TAG_GAME_STATE_RESPONSE => {
                                // GameStateResponse — currently not used by client
                            }
                            _ => {
                                warn!("Unknown tag from server: 0x{:02x}", tag);
                            }
                        }
                    }
                    Ok(None) => {
                        info!("KCP connection closed by server");
                        break;
                    }
                    Err(e) => {
                        error!("KCP read error: {}", e);
                        break;
                    }
                }
            }
        });
    }

    /// Send a player command to the server.
    pub async fn send_command(
        &mut self,
        msg_type: &str,
        action: &str,
        data: serde_json::Value,
    ) -> Result<bool> {
        let data_bytes = serde_json::to_vec(&data)?;
        let cmd = PlayerCommand {
            player_name: self.player_name.clone(),
            msg_type: msg_type.to_string(),
            action: action.to_string(),
            data_json: data_bytes,
        };

        let mut w = self.writer.lock().await;
        write_framed_msg(&mut *w, TAG_PLAYER_COMMAND, &cmd).await?;
        Ok(true)
    }

    /// Subscribe to game events from the server.
    /// Returns a receiver channel that yields parsed game events.
    pub async fn subscribe_events(
        &mut self,
    ) -> Result<mpsc::Receiver<GameEventData>> {
        self.event_rx
            .take()
            .ok_or_else(|| anyhow::anyhow!("subscribe_events can only be called once"))
    }
}
