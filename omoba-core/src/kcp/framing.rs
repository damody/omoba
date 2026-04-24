use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Tag constants for KCP framing protocol
pub const TAG_PLAYER_COMMAND: u8 = 0x01;
pub const TAG_GAME_EVENT: u8 = 0x02;
pub const TAG_COMMAND_ACK: u8 = 0x03;
pub const TAG_SUBSCRIBE_REQUEST: u8 = 0x04;
pub const TAG_GAME_STATE_REQUEST: u8 = 0x05;
pub const TAG_GAME_STATE_RESPONSE: u8 = 0x06;
pub const TAG_VIEWPORT_UPDATE: u8 = 0x07;

/// High bit of the tag byte — set when the framed payload is LZ4-compressed.
/// Base tags 0x01~0x07 never use this bit so it is always free as a flag.
pub const COMPRESSION_FLAG: u8 = 0x80;

/// Minimum payload size before compression is attempted. Below this, LZ4
/// framing overhead dominates any savings, so we skip.
pub const LZ4_THRESHOLD: usize = 128;

/// Try to compress a payload for the wire. Returns `Some((out_tag, Vec<u8>))`
/// with the compression flag OR'd into `base_tag` when the compressed bytes
/// are strictly smaller than the input; otherwise returns `None` so the caller
/// can fall back to the raw payload.
fn maybe_compress(base_tag: u8, payload: &[u8]) -> Option<(u8, Vec<u8>)> {
    if payload.len() < LZ4_THRESHOLD {
        return None;
    }
    let compressed = lz4_flex::block::compress_prepend_size(payload);
    if compressed.len() < payload.len() {
        Some((base_tag | COMPRESSION_FLAG, compressed))
    } else {
        None
    }
}

/// Write a framed message: [1 byte tag][4 bytes len (big-endian)][N bytes payload]
///
/// When the payload is ≥ `LZ4_THRESHOLD` bytes and LZ4 compression actually
/// shrinks it, the payload is replaced with a size-prepended LZ4 block and
/// `COMPRESSION_FLAG` is OR'd into the tag.
pub async fn write_framed<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    tag: u8,
    payload: &[u8],
) -> std::io::Result<()> {
    debug_assert!(
        tag & COMPRESSION_FLAG == 0,
        "base tag must not use high bit; caller passed 0x{:02x}",
        tag
    );
    let (out_tag, out_payload): (u8, &[u8]);
    let compressed_holder;
    match maybe_compress(tag, payload) {
        Some((t, bytes)) => {
            out_tag = t;
            compressed_holder = bytes;
            out_payload = &compressed_holder;
        }
        None => {
            out_tag = tag;
            out_payload = payload;
        }
    }
    let len = out_payload.len() as u32;
    writer.write_u8(out_tag).await?;
    writer.write_u32(len).await?;
    writer.write_all(out_payload).await?;
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
///
/// If the incoming tag byte has `COMPRESSION_FLAG` set, the payload is decoded
/// as an LZ4 size-prepended block and the returned tag has the flag stripped
/// (so callers see the original 0x01~0x07 tag regardless of the wire form).
pub async fn read_framed<R: AsyncReadExt + Unpin>(
    reader: &mut R,
) -> std::io::Result<Option<(u8, Vec<u8>)>> {
    let tag_raw = match reader.read_u8().await {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    };
    let len = reader.read_u32().await? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    if tag_raw & COMPRESSION_FLAG != 0 {
        let base_tag = tag_raw & 0x7F;
        let decompressed = lz4_flex::block::decompress_size_prepended(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Some((base_tag, decompressed)))
    } else {
        Ok(Some((tag_raw, buf)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncWriteExt, duplex};

    #[tokio::test]
    async fn small_payload_not_compressed() {
        // Payload below threshold → tag must stay raw, no compression bit.
        let (mut a, mut b) = duplex(8192);
        let payload = b"hello world".to_vec();
        assert!(payload.len() < LZ4_THRESHOLD);

        write_framed(&mut a, TAG_GAME_EVENT, &payload).await.unwrap();
        a.shutdown().await.ok();

        // Peek the raw wire bytes manually: first byte should be the base tag,
        // NOT the tag | COMPRESSION_FLAG.
        let mut wire = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut b, &mut wire).await.unwrap();
        assert_eq!(wire[0], TAG_GAME_EVENT, "small payload must not set compression flag");
        assert_eq!(wire[0] & COMPRESSION_FLAG, 0);

        // And a full roundtrip recovers the original bytes + tag.
        let (mut a2, mut b2) = duplex(8192);
        write_framed(&mut a2, TAG_GAME_EVENT, &payload).await.unwrap();
        a2.shutdown().await.ok();
        let (tag, out) = read_framed(&mut b2).await.unwrap().unwrap();
        assert_eq!(tag, TAG_GAME_EVENT);
        assert_eq!(out, payload);
    }

    #[tokio::test]
    async fn large_payload_compressed() {
        // Highly redundant 1KB payload — should compress >>2x.
        let payload = vec![0xABu8; 1000];

        // First, capture the raw wire bytes to assert we actually saved space.
        let (mut a, mut b) = duplex(8192);
        write_framed(&mut a, TAG_GAME_EVENT, &payload).await.unwrap();
        a.shutdown().await.ok();
        let mut wire = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut b, &mut wire).await.unwrap();

        // Wire format: [1B tag][4B len][compressed payload]
        assert_eq!(
            wire[0] & COMPRESSION_FLAG,
            COMPRESSION_FLAG,
            "large redundant payload must set compression flag"
        );
        assert_eq!(wire[0] & 0x7F, TAG_GAME_EVENT);
        // Total wire bytes (tag + len + payload) should be meaningfully smaller.
        assert!(
            wire.len() < 500,
            "expected <500 wire bytes for 1000 redundant bytes, got {}",
            wire.len()
        );
        // Specifically the compressed payload length field
        let len_bytes: [u8; 4] = wire[1..5].try_into().unwrap();
        let compressed_len = u32::from_be_bytes(len_bytes) as usize;
        assert!(
            compressed_len < payload.len(),
            "compressed {} vs raw {}",
            compressed_len,
            payload.len()
        );

        // Second, full roundtrip: read_framed transparently decompresses.
        let (mut a2, mut b2) = duplex(8192);
        write_framed(&mut a2, TAG_GAME_EVENT, &payload).await.unwrap();
        a2.shutdown().await.ok();
        let (tag, out) = read_framed(&mut b2).await.unwrap().unwrap();
        assert_eq!(tag, TAG_GAME_EVENT, "read_framed must strip COMPRESSION_FLAG");
        assert_eq!(out, payload, "decompressed bytes must equal original");

        eprintln!(
            "lz4 test: 1000 raw bytes → {} compressed bytes ({} total wire bytes)",
            compressed_len,
            wire.len()
        );
    }

    #[tokio::test]
    async fn incompressible_payload_falls_back() {
        // Random-ish / already-compressed data usually fails to shrink. Use
        // data that LZ4 cannot meaningfully compress: small + non-redundant.
        // Construct a payload at threshold size with high entropy (Knuth hash).
        let mut payload = Vec::with_capacity(200);
        for i in 0..200u32 {
            payload.push((i.wrapping_mul(2654435761) & 0xFF) as u8);
        }
        assert!(payload.len() >= LZ4_THRESHOLD);

        // Inspect the raw wire bytes directly to prove the fallback branch
        // fired — the high-entropy Knuth sequence COULD coincidentally compress
        // a byte or two, and we must not let roundtrip-only assertions mask
        // that case.
        let (mut a, mut b) = duplex(8192);
        write_framed(&mut a, TAG_GAME_EVENT, &payload).await.unwrap();
        a.shutdown().await.ok();

        let mut wire = Vec::new();
        b.read_to_end(&mut wire).await.unwrap();
        assert_eq!(
            wire[0] & COMPRESSION_FLAG,
            0,
            "fallback path: tag high-bit must be clear when we keep raw payload"
        );
        assert_eq!(wire[0] & 0x7F, TAG_GAME_EVENT);
        let len = u32::from_be_bytes(wire[1..5].try_into().unwrap()) as usize;
        assert_eq!(len, payload.len(), "fallback path keeps original length");
        assert_eq!(&wire[5..], payload.as_slice(), "raw bytes match payload");
    }
}
