use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Tag constants for KCP framing protocol
pub const TAG_PLAYER_COMMAND: u8 = 0x01;
pub const TAG_GAME_EVENT: u8 = 0x02;
pub const TAG_COMMAND_ACK: u8 = 0x03;
pub const TAG_SUBSCRIBE_REQUEST: u8 = 0x04;
pub const TAG_GAME_STATE_REQUEST: u8 = 0x05;
pub const TAG_GAME_STATE_RESPONSE: u8 = 0x06;

/// Write a framed message: [1 byte tag][4 bytes len (big-endian)][N bytes payload]
pub async fn write_framed<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    tag: u8,
    payload: &[u8],
) -> std::io::Result<()> {
    let len = payload.len() as u32;
    writer.write_u8(tag).await?;
    writer.write_u32(len).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

/// Write a framed prost message
pub async fn write_framed_msg<W: AsyncWriteExt + Unpin, M: prost::Message>(
    writer: &mut W,
    tag: u8,
    msg: &M,
) -> std::io::Result<()> {
    let payload = msg.encode_to_vec();
    write_framed(writer, tag, &payload).await
}

/// Read a framed message, returns (tag, payload bytes).
/// Returns None on EOF.
pub async fn read_framed<R: AsyncReadExt + Unpin>(
    reader: &mut R,
) -> std::io::Result<Option<(u8, Vec<u8>)>> {
    let tag = match reader.read_u8().await {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    };
    let len = reader.read_u32().await? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(Some((tag, buf)))
}
