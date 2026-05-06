use tokio::io::{AsyncReadExt, AsyncWriteExt};

// KCP 成幀協定的標記常數
pub const TAG_PLAYER_COMMAND: u8 = 0x01;
pub const TAG_GAME_EVENT: u8 = 0x02;
pub const TAG_COMMAND_ACK: u8 = 0x03;
pub const TAG_SUBSCRIBE_REQUEST: u8 = 0x04;
pub const TAG_GAME_STATE_REQUEST: u8 = 0x05;
pub const TAG_GAME_STATE_RESPONSE: u8 = 0x06;
pub const TAG_VIEWPORT_UPDATE: u8 = 0x07;

// 第 2 階段鎖步標籤。範圍 0x10..=0x18。 COMPRESSION_FLAG (0x80) 位
// 這些也未使用，因此相同的 write_framed / read_framed 路徑可以工作。
pub const TAG_INPUT_SUBMIT: u8 = 0x10;  // C→S
pub const TAG_TICK_BATCH: u8 = 0x11;    // S→C broadcast (lockstep_joined sessions)
pub const TAG_STATE_HASH: u8 = 0x12;    // S→C broadcast (lockstep_joined sessions)
pub const TAG_JOIN_REQUEST: u8 = 0x13;  // C→S
pub const TAG_GAME_START: u8 = 0x14;    // S→C unicast (reply to JoinRequest)
pub const TAG_SNAPSHOT_REQ: u8 = 0x15;  // C→S
pub const TAG_SNAPSHOT_RESP: u8 = 0x16; // S→C unicast
pub const TAG_PING_REQ: u8 = 0x17;      // C→S RTT probe
pub const TAG_PING_RESP: u8 = 0x18;     // S→C echo

/// 標籤位元組的高位元 — 當幀有效負載經過 LZ4 壓縮時設定。
/// 基本標籤 0x01~0x07 從不使用該位，因此它始終可以作為標誌自由使用。
pub const COMPRESSION_FLAG: u8 = 0x80;

/// 嘗試壓縮之前的最小有效負載大小。下面這個，LZ4
/// 幀開銷在所有節省中占主導地位，因此我們跳過。
pub const LZ4_THRESHOLD: usize = 128;

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
    let len = reader.read_u32().await? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    let wire_bytes = 1 + 4 + len;
    if tag_raw & COMPRESSION_FLAG != 0 {
        let base_tag = tag_raw & 0x7F;
        let decompressed = lz4_flex::block::decompress_size_prepended(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Some((base_tag, decompressed, wire_bytes)))
    } else {
        Ok(Some((tag_raw, buf, wire_bytes)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncWriteExt, duplex};

    #[tokio::test]
    async fn small_payload_not_compressed() {
        // 有效負載低於閾值→標籤必須保持原始狀態，無壓縮位元。
        let (mut a, mut b) = duplex(8192);
        let payload = b"hello world".to_vec();
        assert!(payload.len() < LZ4_THRESHOLD);

        write_framed(&mut a, TAG_GAME_EVENT, &payload).await.unwrap();
        a.shutdown().await.ok();

        // 手動查看原始線字節：第一個位元組應該是基本標籤，
        // 不是標籤 |壓縮標誌。
        let mut wire = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut b, &mut wire).await.unwrap();
        assert_eq!(wire[0], TAG_GAME_EVENT, "small payload must not set compression flag");
        assert_eq!(wire[0] & COMPRESSION_FLAG, 0);

        // 並且完整的往返恢復原始位元組+標籤。
        let (mut a2, mut b2) = duplex(8192);
        write_framed(&mut a2, TAG_GAME_EVENT, &payload).await.unwrap();
        a2.shutdown().await.ok();
        let (tag, out) = read_framed(&mut b2).await.unwrap().unwrap();
        assert_eq!(tag, TAG_GAME_EVENT);
        assert_eq!(out, payload);
    }

    #[tokio::test]
    async fn large_payload_compressed() {
        // 高度冗餘的 1KB 有效負載 — 應壓縮 >>2 倍。
        let payload = vec![0xABu8; 1000];

        // 首先，捕獲原始線路位元組以斷言我們確實節省了空間。
        let (mut a, mut b) = duplex(8192);
        write_framed(&mut a, TAG_GAME_EVENT, &payload).await.unwrap();
        a.shutdown().await.ok();
        let mut wire = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut b, &mut wire).await.unwrap();

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
