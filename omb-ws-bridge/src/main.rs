use std::env;
use std::io;
use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tokio_kcp::{KcpConfig, KcpNoDelayConfig, KcpStream};

const DEFAULT_WS_ADDR: &str = "127.0.0.1:50062";
const DEFAULT_KCP_ADDR: &str = "127.0.0.1:50061";
const COMPRESSION_FLAG: u8 = 0x80;

#[tokio::main]
async fn main() -> io::Result<()> {
    let (ws_addr, kcp_addr) = config_from_env_or_args()?;
    let listener = TcpListener::bind(ws_addr).await?;
    eprintln!("omb-ws-bridge listening on ws://{} -> kcp://{}", ws_addr, kcp_addr);

    loop {
        let (stream, peer) = listener.accept().await?;
        let kcp_addr = kcp_addr;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer, kcp_addr).await {
                eprintln!("bridge connection {} ended: {}", peer, e);
            }
        });
    }
}

fn config_from_env_or_args() -> io::Result<(SocketAddr, SocketAddr)> {
    let mut args = env::args().skip(1);
    let ws = args
        .next()
        .or_else(|| env::var("OMBOBA_WS_ADDR").ok())
        .unwrap_or_else(|| DEFAULT_WS_ADDR.to_string());
    let kcp = args
        .next()
        .or_else(|| env::var("OMB_KCP_ADDR").ok())
        .unwrap_or_else(|| DEFAULT_KCP_ADDR.to_string());
    let ws_addr = ws.parse::<SocketAddr>().map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidInput, format!("invalid WebSocket addr {ws}: {e}"))
    })?;
    let kcp_addr = kcp.parse::<SocketAddr>().map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidInput, format!("invalid KCP addr {kcp}: {e}"))
    })?;
    Ok((ws_addr, kcp_addr))
}

async fn handle_connection(
    tcp_stream: tokio::net::TcpStream,
    peer: SocketAddr,
    kcp_addr: SocketAddr,
) -> io::Result<()> {
    let ws = tokio_tungstenite::accept_async(tcp_stream)
        .await
        .map_err(ws_error)?;
    eprintln!("browser WebSocket connected from {}", peer);

    let mut config = KcpConfig::default();
    config.nodelay = KcpNoDelayConfig::fastest();
    let kcp = KcpStream::connect(&config, kcp_addr).await?;
    eprintln!("connected bridge session {} to KCP {}", peer, kcp_addr);

    let (mut ws_tx, mut ws_rx) = ws.split();
    let (mut kcp_rx, mut kcp_tx) = tokio::io::split(kcp);

    loop {
        tokio::select! {
            ws_msg = ws_rx.next() => {
                let Some(ws_msg) = ws_msg else { break; };
                let ws_msg = ws_msg.map_err(ws_error)?;
                match ws_msg {
                    Message::Binary(bytes) => {
                        validate_framed_bytes(&bytes)?;
                        kcp_tx.write_all(&bytes).await?;
                        kcp_tx.flush().await?;
                    }
                    Message::Close(_) => break,
                    Message::Ping(bytes) => {
                        ws_tx.send(Message::Pong(bytes)).await.map_err(ws_error)?;
                    }
                    Message::Pong(_) => {}
                    Message::Text(text) => {
                        eprintln!("ignoring text frame from {}: {}", peer, text);
                    }
                    Message::Frame(_) => {}
                }
            }
            frame = read_framed_for_websocket(&mut kcp_rx) => {
                let Some(frame) = frame? else { break; };
                ws_tx.send(Message::Binary(frame)).await.map_err(ws_error)?;
            }
        }
    }

    eprintln!("browser WebSocket disconnected from {}", peer);
    Ok(())
}

async fn read_framed_for_websocket<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let tag_raw = match reader.read_u8().await {
        Ok(tag) => tag,
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    };
    let len = reader.read_u32().await? as usize;
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;

    let (tag, payload) = if tag_raw & COMPRESSION_FLAG != 0 {
        let tag = tag_raw & 0x7f;
        let decompressed = lz4_flex::block::decompress_size_prepended(&payload)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        (tag, decompressed)
    } else {
        (tag_raw, payload)
    };

    let mut out = Vec::with_capacity(5 + payload.len());
    out.push(tag);
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(&payload);
    Ok(Some(out))
}

fn validate_framed_bytes(bytes: &[u8]) -> io::Result<()> {
    if bytes.len() < 5 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("short frame: {} bytes", bytes.len()),
        ));
    }
    let len = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
    if bytes.len() != 5 + len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("bad frame length header={} actual={}", len, bytes.len().saturating_sub(5)),
        ));
    }
    Ok(())
}

fn ws_error<E: std::fmt::Display>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e.to_string())
}
