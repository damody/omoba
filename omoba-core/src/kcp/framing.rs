use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Preserve the existing public KCP path while shared policy stays transport independent.
pub use crate::wire_tags::*;

pub use crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION;
pub const SELECTIVE_FRAME_SCHEMA_VERSION: u32 = 1;
pub const FILTERED_SNAPSHOT_SCHEMA_VERSION: u32 = 1;

/// 標籤位元組的高位元 — 當幀有效負載經過 LZ4 壓縮時設定。
/// 基本標籤 0x01~0x07 從不使用該位，因此它始終可以作為標誌自由使用。
pub const COMPRESSION_FLAG: u8 = 0x80;

/// 嘗試壓縮之前的最小有效負載大小。下面這個，LZ4
/// 幀開銷在所有節省中占主導地位，因此我們跳過。
pub const LZ4_THRESHOLD: usize = 64;

/// 嘗試壓縮線路的有效負載。回傳 `Some((out_tag, Vec<u8>))`
/// 當壓縮位元組時，與壓縮標誌或“base_tag”
/// 嚴格小於輸入；否則返回“None”，因此呼叫者
/// 可以回落到原始有效負載。
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

/// 寫入幀訊息：[1 位元組標籤][4 位元組 len (big-endian)][N 位元組有效負載]
///
/// 當有效負載≥“LZ4_THRESHOLD”位元組時，實際上是LZ4壓縮
/// 縮小它，有效負載被替換為大小前置的 LZ4 區塊，並且
/// `COMPRESSION_FLAG` 與標籤進行或運算。
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

/// 寫一封帶框的前列腺訊息
pub async fn write_framed_msg<W: AsyncWriteExt + Unpin, M: prost::Message>(
    writer: &mut W,
    tag: u8,
    msg: &M,
) -> std::io::Result<()> {
    let payload = msg.encode_to_vec();
    write_framed(writer, tag, &payload).await
}

/// Decompressed-byte ceiling for one communication tag. Other tags, including
/// snapshots, have no ceiling here.
pub fn communication_decompressed_limit(base_tag: u8) -> Option<usize> {
    match base_tag & !COMPRESSION_FLAG {
        TAG_COMMUNICATION_HELLO
        | TAG_COMMUNICATION_HELLO_RESULT
        | TAG_COMMUNICATION_REPLAY_QUERY
        | TAG_COMMUNICATION_RECEIPT => Some(96),
        TAG_COMMUNICATION_SUBMIT => Some(512),
        TAG_COMMUNICATION_EVENT => Some(768),
        TAG_COMMUNICATION_REPLAY => Some(24 * 1024),
        _ => None,
    }
}

/// Checked wire size. Communication tags are rejected before the payload
/// buffer is allocated. Legacy tags keep their existing unbounded length.
pub fn communication_wire_bytes(tag_raw: u8, len: u32) -> std::io::Result<usize> {
    let len_usize = len as usize;
    if communication_decompressed_limit(tag_raw).is_some_and(|limit| len_usize > limit) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "communication frame exceeds its wire bound",
        ));
    }
    1usize
        .checked_add(4)
        .and_then(|prefix| prefix.checked_add(len_usize))
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "frame length overflow"))
}

/// Decode one frame body. A communication LZ4 prefix is bounded before
/// `decompress_size_prepended` allocates. A prefix shorter than 4 bytes is
/// truncated. Snapshot tags are not given a new global cap.
pub fn decode_framed_payload(tag_raw: u8, buf: &[u8]) -> std::io::Result<(u8, Vec<u8>)> {
    if tag_raw & COMPRESSION_FLAG != 0 {
        let base = tag_raw & !COMPRESSION_FLAG;
        if buf.len() < 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "truncated lz4 size prefix",
            ));
        }
        let declared = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        if communication_decompressed_limit(base).is_some_and(|limit| declared > limit) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "communication decompressed prefix exceeds its bound",
            ));
        }
        let decompressed = lz4_flex::block::decompress_size_prepended(buf)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        if communication_decompressed_limit(base).is_some_and(|limit| decompressed.len() > limit) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "communication decompressed payload exceeds its bound",
            ));
        }
        Ok((base, decompressed))
    } else if communication_decompressed_limit(tag_raw).is_some_and(|limit| buf.len() > limit) {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "communication payload exceeds its bound",
        ))
    } else {
        Ok((tag_raw, buf.to_vec()))
    }
}

/// 讀取幀訊息，返回（tag，decompressed_pa​​yload，wire_bytes）。
/// EOF 時不回傳任何內容。
///
/// `wire_bytes` 是線路上的實際 UDP/KCP 位元組成本 = 1 (標籤) +
/// 4（長度前綴）+ N（原始幀有效負載，可能經過 LZ4 壓縮）。
/// 讓呼叫者能夠獨立於實際情況報告壓縮的線路成本
/// 用於 HUD/計數器報告的解壓縮邏輯有效負載大小。
///
/// 如果傳入的標籤位元組設定了“COMPRESSION_FLAG”，則有效負載將被解碼
/// 作為 LZ4 大小前置塊，並且返回的標記已剝離標誌
/// （因此無論線路形式如何，呼叫者都會看到原始的 0x01~0x07 標記）。
pub async fn read_framed<R: AsyncReadExt + Unpin>(
    reader: &mut R,
) -> std::io::Result<Option<(u8, Vec<u8>, usize)>> {
    let tag_raw = match reader.read_u8().await {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    };
    let len = reader.read_u32().await?;
    let wire_bytes = communication_wire_bytes(tag_raw, len)?;
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await?;
    let (tag, payload) = decode_framed_payload(tag_raw, &buf)?;
    Ok(Some((tag, payload, wire_bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graceful_session_close_tag_is_pinned_and_not_compressed() {
        assert_eq!(TAG_SESSION_CLOSE, 0x2A);
        assert_eq!(TAG_SESSION_CLOSE & COMPRESSION_FLAG, 0);
        assert_eq!(TAG_OWNER_WARD_ENVELOPE, 0x2D);
        assert_eq!(TAG_OWNER_WARD_RECOVERY_QUERY, 0x2E);
        assert_eq!(TAG_OWNER_WARD_ENVELOPE & COMPRESSION_FLAG, 0);
        assert_eq!(TAG_OWNER_WARD_RECOVERY_QUERY & COMPRESSION_FLAG, 0);
        assert_eq!(TAG_COMMUNICATION_HELLO, 0x2F);
        assert_eq!(TAG_COMMUNICATION_REPLAY, 0x35);
        for tag in [
            TAG_COMMUNICATION_HELLO,
            TAG_COMMUNICATION_HELLO_RESULT,
            TAG_COMMUNICATION_SUBMIT,
            TAG_COMMUNICATION_RECEIPT,
            TAG_COMMUNICATION_EVENT,
            TAG_COMMUNICATION_REPLAY_QUERY,
            TAG_COMMUNICATION_REPLAY,
        ] {
            assert_eq!(tag & COMPRESSION_FLAG, 0);
            assert_ne!(tag, TAG_PING_REQ);
            assert_ne!(tag, TAG_PING_RESP);
            assert_ne!(tag, TAG_SHOP_RECEIPT_QUERY);
            assert_ne!(tag, TAG_SHOP_RECEIPT_REPLAY);
            assert_ne!(tag, TAG_OWNER_WARD_ENVELOPE);
            assert_ne!(tag, TAG_OWNER_WARD_RECOVERY_QUERY);
        }
    }
    use tokio::io::{duplex, AsyncWriteExt};

    #[tokio::test]
    async fn small_payload_not_compressed() {
        // 有效負載低於閾值→標籤必須保持原始狀態，無壓縮位元。
        let (mut a, mut b) = duplex(8192);
        let payload = b"hello world".to_vec();
        assert!(payload.len() < LZ4_THRESHOLD);

        write_framed(&mut a, TAG_GAME_EVENT, &payload)
            .await
            .unwrap();
        a.shutdown().await.ok();

        // 手動查看原始線字節：第一個位元組應該是基本標籤，
        // 不是標籤 |壓縮標誌。
        let mut wire = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut b, &mut wire)
            .await
            .unwrap();
        assert_eq!(
            wire[0], TAG_GAME_EVENT,
            "small payload must not set compression flag"
        );
        assert_eq!(wire[0] & COMPRESSION_FLAG, 0);

        // 並且完整的往返恢復原始位元組+標籤。
        let (mut a2, mut b2) = duplex(8192);
        write_framed(&mut a2, TAG_GAME_EVENT, &payload)
            .await
            .unwrap();
        a2.shutdown().await.ok();
        let (tag, out, _wire_bytes) = read_framed(&mut b2).await.unwrap().unwrap();
        assert_eq!(tag, TAG_GAME_EVENT);
        assert_eq!(out, payload);
    }

    #[tokio::test]
    async fn large_payload_compressed() {
        // 高度冗餘的 1KB 有效負載 — 應壓縮 >>2 倍。
        let payload = vec![0xABu8; 1000];

        // 首先，捕獲原始線路位元組以斷言我們確實節省了空間。
        let (mut a, mut b) = duplex(8192);
        write_framed(&mut a, TAG_GAME_EVENT, &payload)
            .await
            .unwrap();
        a.shutdown().await.ok();
        let mut wire = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut b, &mut wire)
            .await
            .unwrap();

        // 線路格式：[1B 標籤][4B 長度][壓縮有效負載]
        assert_eq!(
            wire[0] & COMPRESSION_FLAG,
            COMPRESSION_FLAG,
            "large redundant payload must set compression flag"
        );
        assert_eq!(wire[0] & 0x7F, TAG_GAME_EVENT);
        // 總線路位元組（標籤+長度+有效負載）應該更小。
        assert!(
            wire.len() < 500,
            "expected <500 wire bytes for 1000 redundant bytes, got {}",
            wire.len()
        );
        // 具體來說是壓縮的有效負載長度字段
        let len_bytes: [u8; 4] = wire[1..5].try_into().unwrap();
        let compressed_len = u32::from_be_bytes(len_bytes) as usize;
        assert!(
            compressed_len < payload.len(),
            "compressed {} vs raw {}",
            compressed_len,
            payload.len()
        );

        // 其次，完整往返：read_framed 透明解壓縮。
        let (mut a2, mut b2) = duplex(8192);
        write_framed(&mut a2, TAG_GAME_EVENT, &payload)
            .await
            .unwrap();
        a2.shutdown().await.ok();
        let (tag, out, _wire_bytes) = read_framed(&mut b2).await.unwrap().unwrap();
        assert_eq!(
            tag, TAG_GAME_EVENT,
            "read_framed must strip COMPRESSION_FLAG"
        );
        assert_eq!(out, payload, "decompressed bytes must equal original");

        eprintln!(
            "lz4 test: 1000 raw bytes → {} compressed bytes ({} total wire bytes)",
            compressed_len,
            wire.len()
        );
    }

    #[tokio::test]
    async fn incompressible_payload_falls_back() {
        // Random-ish / already-compressed data usually fails to shrink.使用
        // LZ4 無法有效壓縮的資料：小+非冗餘。
        // 建立具有高熵的閾值大小的有效負載（Knuth 雜湊）。
        let mut payload = Vec::with_capacity(200);
        for i in 0..200u32 {
            payload.push((i.wrapping_mul(2654435761) & 0xFF) as u8);
        }
        assert!(payload.len() >= LZ4_THRESHOLD);

        // 直接檢查原始線路位元組以證明後備分支
        // 發射 - 高熵 Knuth 序列可能同時壓縮
        // 一兩個字節，我們不能讓僅往返斷言屏蔽
        // 那種情況。
        let (mut a, mut b) = duplex(8192);
        write_framed(&mut a, TAG_GAME_EVENT, &payload)
            .await
            .unwrap();
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

    #[tokio::test]
    async fn communication_frame_rejects_malicious_length_prefix_and_accepts_compressed_boundary() {
        let (mut writer, mut reader) = duplex(64);
        writer.write_u8(TAG_COMMUNICATION_SUBMIT).await.unwrap();
        writer.write_u32(1_000_000).await.unwrap();
        writer.shutdown().await.ok();
        let malicious = read_framed(&mut reader).await;
        assert!(malicious.is_err(), "a communication length above the tag bound is rejected before use");

        let (mut writer, mut reader) = duplex(64);
        writer.write_u8(TAG_COMMUNICATION_EVENT | COMPRESSION_FLAG).await.unwrap();
        let mut prefix = Vec::new();
        prefix.extend_from_slice(&u32::MAX.to_le_bytes());
        prefix.extend_from_slice(&[0, 0, 0, 0]);
        writer.write_u32(prefix.len() as u32).await.unwrap();
        writer.write_all(&prefix).await.unwrap();
        writer.shutdown().await.ok();
        assert!(read_framed(&mut reader).await.is_err(), "huge decompressed prefix is rejected");

        let (mut writer, mut reader) = duplex(32);
        writer.write_u8(TAG_COMMUNICATION_RECEIPT | COMPRESSION_FLAG).await.unwrap();
        writer.write_u32(2).await.unwrap();
        writer.write_all(&[1, 2]).await.unwrap();
        writer.shutdown().await.ok();
        assert!(read_framed(&mut reader).await.is_err(), "a short lz4 prefix is truncated");

        let (mut writer, mut reader) = duplex(32);
        writer.write_u8(TAG_COMMUNICATION_HELLO).await.unwrap();
        writer.write_u32(4).await.unwrap();
        writer.write_all(&[1, 2]).await.unwrap();
        writer.shutdown().await.ok();
        assert!(read_framed(&mut reader).await.is_err(), "a short declared payload is truncated");

        let payload = vec![0x11u8; 768];
        let (mut writer, mut reader) = duplex(4096);
        write_framed(&mut writer, TAG_COMMUNICATION_EVENT, &payload).await.unwrap();
        writer.shutdown().await.ok();
        let (tag, out, _) = read_framed(&mut reader).await.unwrap().unwrap();
        assert_eq!(tag, TAG_COMMUNICATION_EVENT);
        assert_eq!(out, payload);

        let snapshot = vec![0xABu8; 1000];
        let (mut writer, mut reader) = duplex(4096);
        write_framed(&mut writer, TAG_SNAPSHOT_RESP, &snapshot).await.unwrap();
        writer.shutdown().await.ok();
        let (tag, out, _) = read_framed(&mut reader).await.unwrap().unwrap();
        assert_eq!(tag, TAG_SNAPSHOT_RESP);
        assert_eq!(out, snapshot);
    }
}
