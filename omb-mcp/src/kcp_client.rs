use anyhow::Result;
use prost::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_kcp::{KcpConfig, KcpStream, KcpNoDelayConfig};

pub mod game_proto {
    include!(concat!(env!("OUT_DIR"), "/game.rs"));
}

use game_proto::{GameStateRequest, GameStateResponse};

const TAG_GAME_STATE_REQUEST: u8 = 0x05;
const TAG_GAME_STATE_RESPONSE: u8 = 0x06;

pub struct GameClient {
    stream: KcpStream,
}

impl GameClient {
    /// 連線到 KCP 遊戲伺服器（僅查詢，不訂閱事件）。
    pub async fn connect(addr: &str) -> Result<Self> {
        let mut config = KcpConfig::default();
        config.nodelay = KcpNoDelayConfig::fastest();

        let sock_addr: std::net::SocketAddr = addr.parse()?;
        let stream = KcpStream::connect(&config, sock_addr).await?;
        Ok(Self { stream })
    }

    pub async fn query_state(&mut self, query_type: &str, player_name: &str) -> Result<QueryResponse> {
        let req = GameStateRequest {
            query_type: query_type.to_string(),
            player_name: player_name.to_string(),
        };

        // 寫入封包化 request
        let payload = req.encode_to_vec();
        self.stream.write_u8(TAG_GAME_STATE_REQUEST).await?;
        self.stream.write_u32(payload.len() as u32).await?;
        self.stream.write_all(&payload).await?;
        self.stream.flush().await?;

        // 讀取封包化 response（query-only 連線不會接收 events）
        let tag = self.stream.read_u8().await?;
        let len = self.stream.read_u32().await? as usize;
        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf).await?;

        if tag != TAG_GAME_STATE_RESPONSE {
            anyhow::bail!("Unexpected response tag: 0x{:02x}", tag);
        }

        let resp = GameStateResponse::decode(buf.as_slice())?;
        Ok(QueryResponse {
            success: resp.success,
            error: resp.error,
            data_json: resp.data_json,
        })
    }
}

pub struct QueryResponse {
    pub success: bool,
    pub error: String,
    pub data_json: Vec<u8>,
}
