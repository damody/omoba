use anyhow::Result;
use log::*;
use prost::Message;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use tokio_kcp::{KcpConfig, KcpNoDelayConfig, KcpStream};

use super::framing::*;
use super::game_proto::*;
use crate::quant::{facing_dequant, fixed_dequant, pos_dequant};
use omoba_template_ids::{active_creep_display, projectile_id_str, CreepId, ProjectileKindId};

/// P3：英雄靜態元資料的客戶端快取。
///
/// 伺服器現在發出“HeroStatic”（冷，罕見：創建/升級/能力學習）
/// 和「HeroHot」（熱，每 0.3 秒）分別。舊版 omfx 需要一個
/// 合併 `hero.stats` JSON。 shim 快取每個實體 id 的最新靜態，
/// 每次「HeroHot」到達時都會與快取合併並發出一個 Hero.stats JSON
/// 與 P3 之前的線形狀相同，因此 omfx 需要零更改。
///
/// `latest_lives` 追蹤最新的 `GameLives` 事件值並被注入
/// 到合併的 JSON 中（先前伺服器將“lives”填入 Hero.stats 中）。
#[derive(Default)]
struct HeroStatsCache {
    statics: HashMap<u64, HeroStatic>,
    latest_lives: Option<i32>,
}

/// KCP客戶端用於與omb遊戲伺服器通訊。
pub struct KcpClient {
    player_name: String,
    writer: Arc<Mutex<WriteHalf<KcpStream>>>,
    event_rx: Option<mpsc::Receiver<GameEventData>>,
    /// 第 2 階段鎖步：每當讀取器任務
    /// 0x11 / 0x12 / 0x14 / 0x16 幀到達。透過拍攝一次
    /// `subscribe_lockstep`。
    lockstep_rx: Option<mpsc::Receiver<LockstepInbound>>,
    /// 第2階段鎖步：由GameStart分配；需要標記InputSubmit。
    last_player_id: Option<u32>,
    /// 第 2 階段鎖步：為呼叫者快取 master_seed。
    last_master_seed: Option<u64>,
}

/// 階段 2 鎖定步入站幀從 kcp 讀取器顯示客戶端
/// 任務。 `GameStart` 也是透過這個通道傳遞的，因此呼叫者
/// 等待“join_lockstep”可以從同一流接收。
///
/// `wire_bytes` = 實際 UDP 成本（壓縮後 + 5 位元組幀）。
/// `邏輯位元組` = 解壓縮後的 prost 有效負載長度。讓消費者
/// HUD 顯示這兩個數字，因此鎖步頻寬勝過傳統頻寬
/// 每個事件的廣播可以量化。
#[derive(Debug, Clone)]
pub enum LockstepInbound {
    TickBatch {
        msg: TickBatch,
        wire_bytes: usize,
        logical_bytes: usize,
    },
    StateHash {
        msg: StateHash,
        wire_bytes: usize,
        logical_bytes: usize,
    },
    GameStart {
        msg: GameStart,
        wire_bytes: usize,
        logical_bytes: usize,
    },
    SnapshotResp {
        msg: SnapshotResp,
        wire_bytes: usize,
        logical_bytes: usize,
    },
    /// 伺服器回顯 PingRequest。 `rtt_us` = 往返時間（以微秒為單位），
    /// 根據回顯的“client_send_us”相對於客戶端的本機計算
    /// 接收時的單調時鐘。
    Pong {
        rtt_us: u64,
        wire_bytes: usize,
        logical_bytes: usize,
    },
}

/// P6：每會話序列間隙檢查的結果。暴露進行測試。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SeqGapResult {
    /// 有史以來的第一個事件 - 只是種子“last_seq”，沒有檢查。
    InitialSeed,
    /// `event.sequence == last_seq + 1` — 沒有間隙。
    Ok,
    /// `event.sequence < last_seq + 1` — 重複/重新排序/換行？僅記錄。
    Backwards,
    /// `event.sequence > last_seq + 1` — 錯過了 `gap_size` 事件。客戶
    /// 應請求重新同步。
    Gap {
        expected: u64,
        got: u64,
        gap_size: u64,
    },
}

/// 純函數：給定先前最後已知的序列和到達的序列
/// 事件的序列，傳回一個`SeqGapResult`。拆分出來進行單元測試
/// 無需啟動 KCP 連線。
///
/// `last_seq_opt = None` 表示此會話中尚未看到任何事件。
/// `last_seq_opt = Some(n)` 表示最後一個成功事件的序列為 `n`；
/// 下一個期望值是「n + 1」。
pub fn detect_seq_gap(last_seq_opt: Option<u64>, got: u64) -> SeqGapResult {
    match last_seq_opt {
        None => SeqGapResult::InitialSeed,
        Some(last) => {
            let expected = last.wrapping_add(1);
            if got == expected {
                SeqGapResult::Ok
            } else if got < expected {
                // 可能是重新排序（KCP 是可靠的+有序的，所以這應該是
                // 罕見）或重複。僅記錄；不要重新同步。
                SeqGapResult::Backwards
            } else {
                SeqGapResult::Gap {
                    expected,
                    got,
                    gap_size: got - expected,
                }
            }
        }
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
    /// 原始 proto data_json bytes 長度；供前端網路吞吐統計用，
    /// 避免在 hot path 做冗餘 serde_json::to_string。
    /// **舊欄位**：=`logical_bytes`（解壓後 prost payload 長度）。
    /// 保留欄位名為了 source-compat。前端要看真實 UDP 成本請用
    /// `wire_bytes` 欄位（`logical_bytes` 約等於 `wire_bytes` × LZ4 解壓比）。
    pub payload_bytes: usize,
    /// **真實 UDP/KCP wire 成本**（LZ4 壓縮後 + framing 1+4 byte header）。
    /// 對應 server 端 `KcpBytesCounter::record(frame.len())` 計的同一個值。
    /// 前端 HUD 顯示真實 bandwidth 應用此欄位，不是 `payload_bytes`。
    pub wire_bytes: usize,
}

impl KcpClient {
    /// 連接到KCP遊戲伺服器。
    #[tracing::instrument(skip_all, fields(perfetto = true, addr = %addr))]
    pub async fn connect(addr: &str, player_name: String) -> Result<Self> {
        let mut config = KcpConfig::default();
        config.nodelay = KcpNoDelayConfig::fastest();

        let sock_addr: std::net::SocketAddr = addr.parse()?;
        let stream = KcpStream::connect(&config, sock_addr).await?;
        info!("Connected to KCP server at {}", addr);

        let (reader, writer) = tokio::io::split(stream);
        let writer = Arc::new(Mutex::new(writer));

        // 共享單調紀元 — ping 發送者和閱讀者（其中
        // 根據該單一即時計算 PingResponse 上的 RTT）時間戳
        // 所以減法是環繞安全的。
        let epoch = std::time::Instant::now();

        // 立即發送訂閱請求
        {
            let mut w = writer.lock().await;
            let sub = SubscribeRequest {
                player_name: player_name.clone(),
            };
            write_framed_msg(&mut *w, TAG_SUBSCRIBE_REQUEST, &sub).await?;
        }

        // 產生後台閱讀器任務
        let (event_tx, event_rx) = mpsc::channel(10000);
        let (lockstep_tx, lockstep_rx) = mpsc::channel(1024);
        Self::spawn_reader(
            reader,
            event_tx,
            lockstep_tx.clone(),
            writer.clone(),
            player_name.clone(),
            epoch,
        );

        // 產生 ping 循環 — 每 1 秒發送一次 TAG_PING_REQ。讀者處理
        // TAG_PING_RESP 並發出 LockstepInbound::Pong 以及計算出的 RTT。
        Self::spawn_ping_loop(writer.clone(), epoch);

        Ok(Self {
            player_name,
            writer,
            event_rx: Some(event_rx),
            lockstep_rx: Some(lockstep_rx),
            last_player_id: None,
            last_master_seed: None,
        })
    }

    /// 定期發送帶有單調時間戳記的 TAG_PING_REQ。
    /// 伺服器將其回應為 TAG_PING_RESP；讀者任務匯出 RTT
    /// 對抗同一個時代。
    fn spawn_ping_loop(writer: Arc<Mutex<WriteHalf<KcpStream>>>, epoch: std::time::Instant) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            // 第一個蜱立即觸發；跳過它，這樣我們就不會參加比賽
            // 會話開始時訂閱/加入握手。
            interval.tick().await;
            loop {
                interval.tick().await;
                let now_us = epoch.elapsed().as_micros() as i64;
                let req = PingRequest {
                    client_send_us: now_us,
                };
                tracing::trace!(perfetto = true, "omoba_core::kcp::send_ping_request");
                let mut w = writer.lock().await;
                if let Err(e) = write_framed_msg(&mut *w, TAG_PING_REQ, &req).await {
                    warn!("Failed to send PingRequest: {}", e);
                    break;
                }
            }
        });
    }

    fn spawn_reader(
        mut reader: ReadHalf<KcpStream>,
        event_tx: mpsc::Sender<GameEventData>,
        lockstep_tx: mpsc::Sender<LockstepInbound>,
        writer_for_resync: Arc<Mutex<WriteHalf<KcpStream>>>,
        player_name_for_resync: String,
        epoch: std::time::Instant,
    ) {
        tokio::spawn(async move {
            // P3：本機快取位於讀取器任務中，因此不需要鎖定。
            // HeroStatic 到達僅更新快取（不發出）；英雄熱門登場
            // 尋找快取+發出合併的舊版hero.stats JSON。
            let mut hero_cache = HeroStatsCache::default();
            // P6：每會話序列間隙檢測。直到第一次之前都沒有
            // 遊戲事件到來；之後，每個後續事件的順序
            // 必須等於最後 + 1。
            let mut last_seq: Option<u64> = None;
            // 限制重新同步請求，導致病態的間隙氾濫
            // 不會變成 StateReq 流量的洪流。跟踪最後一個
            // 我們請求重新同步的序列；如果我們已經問過，請跳過。
            let mut last_resync_req: Option<u64> = None;
            loop {
                match read_framed(&mut reader).await {
                    Ok(Some((tag, payload, wire_compressed_bytes))) => {
                        tracing::trace!(
                            perfetto = true,
                            tag,
                            wire_bytes = wire_compressed_bytes,
                            logical_bytes = payload.len(),
                            "omoba_core::kcp::frame_received"
                        );
                        match tag {
                            TAG_GAME_EVENT => {
                                match GameEvent::decode(payload.as_slice()) {
                                    Ok(event) => {
                                        // P6：解碼有效負載之前檢查序列。
                                        // `sequence` 是 proto3 uint64 — 預設為 0
                                        // 當伺服器是 P6 之前的版本或尚未標記時
                                        // （gRPC路徑），所以我們只進行間隙檢測
                                        // 一旦我們看到一個非零序列。
                                        let got_seq = event.sequence;
                                        match detect_seq_gap(last_seq, got_seq) {
                                            SeqGapResult::InitialSeed => {
                                                last_seq = Some(got_seq);
                                            }
                                            SeqGapResult::Ok => {
                                                last_seq = Some(got_seq);
                                            }
                                            SeqGapResult::Backwards => {
                                                warn!(
                                                    "⏪ seq backwards (got={}, last_seq={:?}) — keeping state",
                                                    got_seq, last_seq
                                                );
                                                // 不要更新last_seq——保留
                                                // 所見最高。複製/重新排序
                                                // 事件仍然得到處理。
                                            }
                                            SeqGapResult::Gap {
                                                expected,
                                                got,
                                                gap_size,
                                            } => {
                                                warn!(
                                                    "⚠️ seq gap: expected={} got={} (missed {} events)",
                                                    expected, got, gap_size
                                                );
                                                // Debounce：不重新要求
                                                // 作者已經聽過同樣的差距
                                                // 關於。
                                                let should_request = match last_resync_req {
                                                    Some(prev) => prev < expected,
                                                    None => true,
                                                };
                                                if should_request {
                                                    last_resync_req = Some(expected);
                                                    let req = GameStateRequest {
                                                        query_type: "seq-gap".into(),
                                                        // 將最後已知的 seq 編碼為
                                                        // player_name 欄位 —
                                                        // 避免原型模式碰撞
                                                        // （請參閱伺服器存根
                                                        // 匹配解碼端）。
                                                        player_name: expected
                                                            .saturating_sub(1)
                                                            .to_string(),
                                                    };
                                                    let w = writer_for_resync.clone();
                                                    tokio::spawn(async move {
                                                        tracing::trace!(
                                                            perfetto = true,
                                                            expected_seq = expected,
                                                            "omoba_core::kcp::send_seq_gap_state_req"
                                                        );
                                                        let mut w = w.lock().await;
                                                        if let Err(e) = write_framed_msg(
                                                            &mut *w,
                                                            TAG_GAME_STATE_REQUEST,
                                                            &req,
                                                        )
                                                        .await
                                                        {
                                                            warn!("Failed to send seq-gap StateReq: {}", e);
                                                        }
                                                    });
                                                }
                                                // 繼續處理無序的情況
                                                // 事件，這樣我們就不會在頂部雙重下降
                                                // 差距（客戶端邏輯容忍
                                                // 過時的 HP/位置更新）。
                                                last_seq = Some(got_seq);
                                            }
                                        }
                                        // 靜默未使用的 var 警告
                                        // 用於建置的“player_name_for_resync”
                                        // 從未到達 Gap 分支。
                                        let _ = &player_name_for_resync;
                                        // P9 信封條：每個事件都帶有印刷的
                                        // 其中“有效負載”。墊片派生 (msg_type, action)
                                        // 來自變體並為舊版 omfx 重建 JSON。
                                        // `wire_compressed_bytes` = 實際 UDP 成本（如果縮小 + 成幀則為 LZ4）；
                                        // `payload.len()` = 解壓縮的 prost 位元組（邏輯有效負載）。
                                        let logical_bytes = payload.len();
                                        let parsed_opt: Option<GameEventData> =
                                            match event.payload.as_ref() {
                                                Some(p) => translate_typed_payload(
                                                    p,
                                                    wire_compressed_bytes,
                                                    logical_bytes,
                                                    &mut hero_cache,
                                                ),
                                                None => None,
                                            };

                                        if let Some(parsed) = parsed_opt {
                                            // try_send 而非 send().await：當沒有時
                                            // 消費者正在耗盡「event_rx」（例如階段 5.1+
                                            // omfx，僅訂閱鎖步
                                            // 流），阻塞發送填充 10000 槽
                                            // 在大約 10 秒的 TD_STRESS 負載中建立通道，然後
                                            // 停止此讀取器任務 - 防止鎖定步
                                            // 同一套接字上的幀永遠不會
                                            // 發表。完全刪除遺留事件是
                                            // 安全，因為他們的消費者選擇加入。
                                            match event_tx.try_send(parsed) {
                                                Ok(()) => {}
                                                Err(mpsc::error::TrySendError::Full(_)) => {
                                                    // 不會耗盡訂閱者；默默地掉落。
                                                }
                                                Err(mpsc::error::TrySendError::Closed(_)) => {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to decode GameEvent: {}", e);
                                    }
                                }
                            }
                            TAG_COMMAND_ACK => {
                                // CommandAck — 目前被忽略
                            }
                            TAG_GAME_STATE_RESPONSE => {
                                // GameStateResponse — 目前未被客戶端使用
                            }
                            // ===== 第 2 階段鎖步標籤 =====
                            TAG_TICK_BATCH => {
                                let logical_bytes = payload.len();
                                match TickBatch::decode(payload.as_slice()) {
                                    Ok(b) => {
                                        if lockstep_tx
                                            .send(LockstepInbound::TickBatch {
                                                msg: b,
                                                wire_bytes: wire_compressed_bytes,
                                                logical_bytes,
                                            })
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                    Err(e) => warn!("Failed to decode TickBatch: {}", e),
                                }
                            }
                            TAG_STATE_HASH => {
                                let logical_bytes = payload.len();
                                match StateHash::decode(payload.as_slice()) {
                                    Ok(s) => {
                                        if lockstep_tx
                                            .send(LockstepInbound::StateHash {
                                                msg: s,
                                                wire_bytes: wire_compressed_bytes,
                                                logical_bytes,
                                            })
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                    Err(e) => warn!("Failed to decode StateHash: {}", e),
                                }
                            }
                            TAG_GAME_START => {
                                let logical_bytes = payload.len();
                                match GameStart::decode(payload.as_slice()) {
                                    Ok(gs) => {
                                        if lockstep_tx
                                            .send(LockstepInbound::GameStart {
                                                msg: gs,
                                                wire_bytes: wire_compressed_bytes,
                                                logical_bytes,
                                            })
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                    Err(e) => warn!("Failed to decode GameStart: {}", e),
                                }
                            }
                            TAG_SNAPSHOT_RESP => {
                                let logical_bytes = payload.len();
                                match SnapshotResp::decode(payload.as_slice()) {
                                    Ok(s) => {
                                        if lockstep_tx
                                            .send(LockstepInbound::SnapshotResp {
                                                msg: s,
                                                wire_bytes: wire_compressed_bytes,
                                                logical_bytes,
                                            })
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                    Err(e) => warn!("Failed to decode SnapshotResp: {}", e),
                                }
                            }
                            TAG_PING_RESP => {
                                let logical_bytes = payload.len();
                                match PingResponse::decode(payload.as_slice()) {
                                    Ok(resp) => {
                                        let now_us = epoch.elapsed().as_micros() as i64;
                                        let rtt_us = (now_us - resp.client_send_us).max(0) as u64;
                                        if lockstep_tx
                                            .send(LockstepInbound::Pong {
                                                rtt_us,
                                                wire_bytes: wire_compressed_bytes,
                                                logical_bytes,
                                            })
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                    Err(e) => warn!("Failed to decode PingResponse: {}", e),
                                }
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

    /// 向伺服器發送玩家命令。
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

    /// 將視口更新傳送到伺服器以進行空間過濾。
    pub async fn send_viewport_update(&self, cx: f32, cy: f32, hw: f32, hh: f32) -> Result<()> {
        let vp = ViewportUpdate {
            center_x: cx,
            center_y: cy,
            half_width: hw,
            half_height: hh,
        };
        let mut w = self.writer.lock().await;
        write_framed_msg(&mut *w, TAG_VIEWPORT_UPDATE, &vp).await?;
        Ok(())
    }

    /// 從伺服器訂閱遊戲事件。
    /// 傳回一個接收器通道，該通道產生已解析的遊戲事件。
    pub async fn subscribe_events(&mut self) -> Result<mpsc::Receiver<GameEventData>> {
        self.event_rx
            .take()
            .ok_or_else(|| anyhow::anyhow!("subscribe_events can only be called once"))
    }

    // ===== 第 2 階段 Lockstep API =====

    /// 取得同步入站流的所有權。產量TickBatch /
    /// StateHash / GameStart / SnapshotResp 幀從
    /// 伺服器.每個客戶端只能呼叫一次。
    pub fn subscribe_lockstep(&mut self) -> Result<mpsc::Receiver<LockstepInbound>> {
        self.lockstep_rx
            .take()
            .ok_or_else(|| anyhow::anyhow!("subscribe_lockstep can only be called once"))
    }

    /// 發送JoinRequest（標籤0x13）並等待伺服器的GameStart回复
    /// （標籤 0x14）。從 GameStart 傳回指定的 `master_seed`
    /// 呼叫者可以構造確定性 SimRng 流。
    ///
    /// 注意：此方法在內部耗盡鎖定步入站通道
    /// 直到它看到 GameStart，所以不要在 `subscribe_lockstep` 之後呼叫它。
    /// 推薦流程為：
    /// 1.`連線`
    /// 2. `join_lockstep` （從頻道消耗 GameStart）
    /// 3. `subscribe_lockstep` （現在只產生 TickBatch/StateHash/SnapshotResp）
    #[tracing::instrument(skip_all, fields(perfetto = true, observer))]
    pub async fn join_lockstep(&mut self, player_name: String, observer: bool) -> Result<u64> {
        let role = if observer {
            JoinRole::RoleObserver
        } else {
            JoinRole::RolePlayer
        };
        let req = JoinRequest {
            player_name: player_name.clone(),
            role: role as i32,
        };
        {
            let mut w = self.writer.lock().await;
            write_framed_msg(&mut *w, TAG_JOIN_REQUEST, &req).await?;
        }
        // 排空鎖步流，直到 GameStart 到達。
        let rx = self
            .lockstep_rx
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("lockstep_rx already taken"))?;
        loop {
            match rx.recv().await {
                Some(LockstepInbound::GameStart { msg: gs, .. }) => {
                    self.last_player_id = Some(gs.player_id);
                    self.last_master_seed = Some(gs.master_seed);
                    return Ok(gs.master_seed);
                }
                Some(_) => {
                    // 可能的競爭：TickBatch 可能在 GameStart 之前到達
                    // 如果伺服器在 `lockstep_joined=true` 之後觸發它
                    // 已設定。我們故意放棄那些──第二階段的來電者
                    // 應該先加入，然後訂閱。
                    continue;
                }
                None => {
                    anyhow::bail!("lockstep stream closed before GameStart");
                }
            }
        }
    }

    /// 提交針對「target_tick」的玩家輸入。呼叫者必須有
    /// 先呼叫 `join_lockstep` （否則不知道 `player_id`）。
    ///
    /// 返回“(邏輯字節，線路字節)”，以便呼叫者可以饋送網絡
    /// 吞吐量計數器。 InputSubmit 訊息很小（遠低於
    /// `LZ4_THRESHOLD = 128`)，因此它們永遠不會被壓縮並且
    /// `連線 = 1 + 4 + 邏輯`。
    #[tracing::instrument(skip_all, fields(perfetto = true, target_tick, input_id))]
    pub async fn submit_input(
        &mut self,
        target_tick: u32,
        input: PlayerInput,
        input_id: u32,
    ) -> Result<(usize, usize)> {
        let player_id = self
            .last_player_id
            .ok_or_else(|| anyhow::anyhow!("submit_input before join_lockstep"))?;
        let req = InputSubmit {
            player_id,
            target_tick,
            input: Some(input),
            input_id,
        };
        let logical_bytes = req.encoded_len();
        let mut w = self.writer.lock().await;
        write_framed_msg(&mut *w, TAG_INPUT_SUBMIT, &req).await?;
        let wire_bytes = 1 + 4 + logical_bytes;
        Ok((logical_bytes, wire_bytes))
    }

    /// 從伺服器請求快照。回覆如下
    /// 鎖步流上的「LockstepInbound::SnapshotResp」。
    #[tracing::instrument(skip_all, fields(perfetto = true, from_tick))]
    pub async fn request_snapshot(&mut self, from_tick: u32) -> Result<()> {
        let req = SnapshotReq { from_tick };
        let mut w = self.writer.lock().await;
        write_framed_msg(&mut *w, TAG_SNAPSHOT_REQ, &req).await?;
        Ok(())
    }

    /// 最近觀察到的由 GameStart 指派的player_id。
    pub fn lockstep_player_id(&self) -> Option<u32> {
        self.last_player_id
    }

    /// 最近從 GameStart 觀察到的 master_seed。
    pub fn lockstep_master_seed(&self) -> Option<u64> {
        self.last_master_seed
    }
}

/// P9 信封條墊片：從輸入的有效負載衍生（msg_type，action）
/// 變體。編譯時詳盡 - 添加新的原型變體打破了
/// 建在這裡。 `LegacyJson` 帶有它自己的密鑰。
pub fn variant_to_legacy_keys(p: &game_event::Payload) -> (String, String) {
    use game_event::Payload::*;
    match p {
        Heartbeat(_) => ("heartbeat".into(), "tick".into()),
        HeroStatic(_) => ("hero".into(), "static_internal".into()),
        HeroHot(_) => ("hero".into(), "stats".into()),
        HeroCreate(_) => ("hero".into(), "create".into()),
        CreepCreate(_) => ("creep".into(), "create".into()),
        CreepMove(_) => ("creep".into(), "M".into()),
        CreepHp(m) => match super::game_proto::EntityKind::try_from(m.kind)
            .unwrap_or(super::game_proto::EntityKind::Unspecified)
        {
            super::game_proto::EntityKind::Creep => ("creep".into(), "H".into()),
            super::game_proto::EntityKind::Hero => ("hero".into(), "H".into()),
            super::game_proto::EntityKind::Unit => ("unit".into(), "H".into()),
            super::game_proto::EntityKind::Tower => ("tower".into(), "H".into()),
            _ => ("entity".into(), "H".into()),
        },
        CreepSlow(_) => ("creep".into(), "S".into()),
        CreepStall(_) => ("creep".into(), "stall".into()),
        EntityFacing(_) => ("entity".into(), "F".into()),
        EntityDeath(m) => match super::game_proto::EntityKind::try_from(m.kind)
            .unwrap_or(super::game_proto::EntityKind::Unspecified)
        {
            super::game_proto::EntityKind::Creep => ("creep".into(), "D".into()),
            super::game_proto::EntityKind::Tower => ("tower".into(), "D".into()),
            super::game_proto::EntityKind::Hero => ("hero".into(), "D".into()),
            super::game_proto::EntityKind::Unit => ("unit".into(), "D".into()),
            super::game_proto::EntityKind::Projectile => ("projectile".into(), "D".into()),
            _ => ("entity".into(), "D".into()),
        },
        UnitCreate(_) => ("unit".into(), "create".into()),
        ProjectileCreate(_) => ("projectile".into(), "C".into()),
        ProjectileDestroy(_) => ("projectile".into(), "D".into()),
        TowerCreate(_) => ("tower".into(), "create".into()),
        TowerUpgrade(_) => ("tower".into(), "upgrade".into()),
        BuffAdd(_) => ("buff".into(), "buff_add".into()),
        BuffRemove(_) => ("buff".into(), "buff_remove".into()),
        GameRound(_) => ("game".into(), "round".into()),
        GameLives(_) => ("game".into(), "lives".into()),
        GameEnd(_) => ("game".into(), "end".into()),
        GameExplosion(_) => ("game".into(), "explosion".into()),
        LegacyJson(m) => (m.msg_type.clone(), m.action.clone()),
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 將 prost 類型的 Payload 轉換為傳統的 JSON 形狀的“GameEventData”
/// omfx 已經消耗了。 P9：線路不再攜帶 topic/msg_type/
/// action / data_json / timestamp_ms — 我們從 (msg_type, action) 匯出
/// 變體標籤，設定 topic =“td/all/res”（伺服器已路由），並使用
/// timestamp_ms 的客戶端本機時鐘。
fn translate_typed_payload(
    tp: &game_event::Payload,
    wire_bytes: usize,
    logical_bytes: usize,
    hero_cache: &mut HeroStatsCache,
) -> Option<GameEventData> {
    let (msg_type, action) = variant_to_legacy_keys(tp);
    let topic = "td/all/res".to_string();
    let timestamp_ms = now_ms();
    let default = || GameEventData {
        topic: topic.clone(),
        msg_type: msg_type.clone(),
        action: action.clone(),
        data: serde_json::Value::Null,
        timestamp_ms,
        payload_bytes: logical_bytes,
        wire_bytes,
    };

    let out = match tp {
        game_event::Payload::Heartbeat(hb) => {
            let hp_snapshot: Vec<serde_json::Value> = hb
                .hp_snapshot
                .iter()
                .map(|e| {
                    let hp = e.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
                    json!({ "i": e.id as u32, "h": hp })
                })
                .collect();
            // P4：用於客戶端漂移校正的蠕變位置樣本。空的
            // Vec（沒有可見的蠕變）序列化為空數組 - omfx 的
            // 快照邏輯的迭代是無害的。
            let pos_snapshot: Vec<serde_json::Value> = hb
                .pos_snapshot
                .iter()
                .map(|e| {
                    let (x, y) = e
                        .pos
                        .as_ref()
                        .map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q)))
                        .unwrap_or((0.0, 0.0));
                    json!({ "i": e.id as u32, "x": x, "y": y })
                })
                .collect();
            // P7分層：伺服器端權威集依然存在
            // 目標在該玩家的範圍內的預先聲明傷害彈頭
            // 視口。客戶端使用它來保留其pending_pred_dmg映射
            // （proj_id 不在該集合中的條目已解決）。
            let in_flight_projectiles: Vec<serde_json::Value> = hb
                .in_flight_projectiles
                .iter()
                .map(|&id| serde_json::Value::from(id))
                .collect();
            let d = json!({
                "tick": hb.tick,
                "game_time": hb.game_time,
                "entity_count": hb.entity_count,
                "hero_count": hb.hero_count,
                "unit_count": hb.unit_count,
                "creep_count": hb.creep_count,
                "render_delay_ms": hb.render_delay_ms,
                "hp_snapshot": hp_snapshot,
                "pos_snapshot": pos_snapshot,
                "in_flight_projectiles": in_flight_projectiles,
            });
            GameEventData {
                topic: topic.clone(),
                msg_type: "heartbeat".to_string(),
                action: "tick".to_string(),
                data: d,
                timestamp_ms,
                payload_bytes: logical_bytes,
                wire_bytes,
            }
        }
        game_event::Payload::ProjectileCreate(m) => {
            let start = m
                .start_pos
                .as_ref()
                .map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q)))
                .unwrap_or((0.0, 0.0));
            let end = m
                .end_pos
                .as_ref()
                .map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q)))
                .unwrap_or((0.0, 0.0));
            let splash = m
                .splash_radius
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let hit = m
                .hit_radius
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            // P7：非AOE彈丸的預先聲明傷害。 0 當
            // 飛濺 > 0 或未設定。 omfx 讀取此內容以安排樂觀的 HP
            // 在影響刻度時更新。
            let damage = m
                .damage
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            // 反向查找 kind_id（現在是來自 omoba-template-ids 的連續 u16）
            // → 原始標籤字串（「大頭釘」/「炸彈」/等）。未知 ID 回退
            // 到“”，這樣 omfx 的項目符號顏色開關就可以優雅地預設。
            let kind_str = projectile_id_str(ProjectileKindId(m.kind_id as u16));
            let d = json!({
                "id": m.id as u32,
                "target_id": m.target_id as u32,
                "start_pos": { "x": start.0, "y": start.1 },
                "end_pos": { "x": end.0, "y": end.1 },
                "flight_time_ms": m.flight_time_ms,
                "directional": m.directional,
                "splash_radius": splash,
                "hit_radius": hit,
                "kind": kind_str,
                "damage": damage,
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::ProjectileDestroy(m) => {
            let d = json!({ "id": m.id as u32 });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::CreepCreate(m) => {
            let pos = m
                .pos
                .as_ref()
                .map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q)))
                .unwrap_or((0.0, 0.0));
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let max_hp = m
                .max_hp
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let move_speed = m
                .move_speed
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            // 反向查找name_id（順序CreepId u16）→顯示名稱。
            // 未知 id →“”（omfx 回落到實體類型字串）。
            let name_str = active_creep_display(CreepId(m.name_id as u16));
            let d = json!({
                "id": m.id as u32,
                "entity_id": m.id as u32,
                "name": name_str,
                "position": { "x": pos.0, "y": pos.1 },
                "hp": hp,
                "max_hp": max_hp,
                "move_speed": move_speed,
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::CreepMove(m) => {
            let tgt = m
                .target
                .as_ref()
                .map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q)))
                .unwrap_or((0.0, 0.0));
            // P4：重建外推場。 `velocity`/`start_pos`/`start_tick`/`arrival_tick`
            // 對於遺留發射（handle_creep_stop freeze）為零 - omfx 將其視為
            // 「僅 lerp，無推論」。
            let velocity = m
                .velocity
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let start = m
                .start_pos
                .as_ref()
                .map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q)))
                .unwrap_or((0.0, 0.0));
            let d = json!({
                "id": m.id as u32,
                "x": tgt.0,
                "y": tgt.1,
                "facing": facing_dequant(m.facing_q),
                "velocity": velocity,
                "arrival_tick": m.arrival_tick,
                "start_pos": { "x": start.0, "y": start.1 },
                "start_tick": m.start_tick,
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::CreepHp(m) => {
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "hp": hp,
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::CreepSlow(m) => {
            let ms = m
                .move_speed
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "move_speed": ms,
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::CreepStall(m) => {
            let pos = m
                .pos
                .as_ref()
                .map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q)))
                .unwrap_or((0.0, 0.0));
            let d = json!({
                "id": m.id as u32,
                "x": pos.0,
                "y": pos.1,
                "facing": facing_dequant(m.facing_q),
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::EntityFacing(m) => {
            let d = json!({
                "id": m.id as u32,
                "facing": facing_dequant(m.facing_q),
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::EntityDeath(m) => {
            let d = json!({ "id": m.id as u32 });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::TowerCreate(m) => {
            let pos = m
                .pos
                .as_ref()
                .map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q)))
                .unwrap_or((0.0, 0.0));
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let max_hp = m
                .max_hp
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "entity_id": m.id as u32,
                "name": m.name,
                "kind": m.kind,
                "position": { "x": pos.0, "y": pos.1 },
                "hp": hp,
                "max_hp": max_hp,
                "is_base": false,
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::TowerUpgrade(m) => {
            let l0 = m.levels.get(0).copied().unwrap_or(0);
            let l1 = m.levels.get(1).copied().unwrap_or(0);
            let l2 = m.levels.get(2).copied().unwrap_or(0);
            let d = json!({
                "tower_id": m.id as u32,
                "levels": [l0, l1, l2],
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::BuffAdd(m) => {
            // 剩餘_ms=0xFFFF 哨兵 → -1 剩餘（無限/切換）
            let remaining = if m.remaining_ms == 0xFFFF {
                -1.0_f32
            } else {
                m.remaining_ms as f32 / 1000.0
            };
            let payload: serde_json::Value =
                serde_json::from_str(&m.payload_json).unwrap_or(serde_json::Value::Null);
            let d = json!({
                "entity_id": m.entity_id as u32,
                "id": m.entity_id as u32,
                "buff_id": m.buff_id,
                "remaining": remaining,
                "payload": payload,
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::BuffRemove(m) => {
            let d = json!({
                "entity_id": m.entity_id as u32,
                "id": m.entity_id as u32,
                "buff_id": m.buff_id,
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        // P3：HeroStatic → 僅更新快取（不傳送到 omfx）。
        // 升級/能力學習很少發生； omfx 將看到更新
        // 下一次 HeroHot 合併時的名稱/等級/能力（≤ 0.3 秒後）。
        game_event::Payload::HeroStatic(m) => {
            hero_cache.statics.insert(m.id, m.clone());
            return None;
        }
        // P3：HeroHot → 與緩存的 HeroStatic 合併並發出舊形狀
        // Hero.stats JSON 因此 omfx 需要零更改。 First Hero之前很熱
        // 任何 HeroStatic 到達 → 空靜態欄位（omfx 使用 `if let
        // 每個字段都有一些，因此它保留最後已知的值）。
        game_event::Payload::HeroHot(m) => {
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let max_hp = m
                .max_hp
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let attack_damage = m
                .attack_damage
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let armor = m
                .armor
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let magic_resist = m
                .magic_resist
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let move_speed = m
                .move_speed
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let attack_range = m
                .attack_range
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let attack_interval = m
                .attack_interval
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);

            let buffs: Vec<serde_json::Value> = m
                .buffs
                .iter()
                .map(|b| {
                    let remaining = if b.remaining_ms == 0xFFFF {
                        -1.0_f32
                    } else {
                        b.remaining_ms as f32 / 1000.0
                    };
                    let payload: serde_json::Value =
                        serde_json::from_str(&b.payload_json).unwrap_or(serde_json::Value::Null);
                    json!({ "id": b.buff_id, "remaining": remaining, "payload": payload })
                })
                .collect();

            let mut d = json!({
                "id": m.id as u32,
                "gold": m.gold as i32,
                "hp": hp,
                "max_hp": max_hp,
                "attack_damage": attack_damage,
                "armor": armor,
                "magic_resist": magic_resist,
                "move_speed": move_speed,
                "attack_range": attack_range,
                "attack_interval": attack_interval,
                "buffs": buffs,
            });

            // 合併快取的 HeroStatic 欄位（名稱/頭銜/基本統計資料/等級/xp/能力）
            if let Some(st) = hero_cache.statics.get(&m.id) {
                let ability_levels_map: serde_json::Map<String, serde_json::Value> = st
                    .ability_ids
                    .iter()
                    .enumerate()
                    .map(|(i, id)| {
                        let lvl = st.ability_levels.get(i).map(|p| p.cur as i32).unwrap_or(0);
                        (id.clone(), json!(lvl))
                    })
                    .collect();
                // 推斷 primary_attribute：按 HeroStatic 的 base_str/agi/int 取最大；
                // 目前 server Hero.primary_attribute 的判定也是根據角色設計 (strength/agility/
                // intelligence) — 實務上 base 最大的就是 primary。
                let (p_name, p_val) = [
                    ("strength", st.base_str),
                    ("agility", st.base_agi),
                    ("intelligence", st.base_int),
                ]
                .iter()
                .max_by_key(|(_, v)| *v)
                .copied()
                .unwrap_or(("strength", 0));
                let _ = p_val;
                if let Some(obj) = d.as_object_mut() {
                    obj.insert("name".into(), json!(st.name));
                    obj.insert("title".into(), json!(st.title));
                    obj.insert("strength".into(), json!(st.base_str as i32));
                    obj.insert("agility".into(), json!(st.base_agi as i32));
                    obj.insert("intelligence".into(), json!(st.base_int as i32));
                    obj.insert("primary_attribute".into(), json!(p_name));
                    obj.insert("level".into(), json!(st.level as i32));
                    obj.insert("xp".into(), json!(st.xp as i32));
                    obj.insert("xp_next".into(), json!(st.xp_next as i32));
                    obj.insert("skill_points".into(), json!(st.skill_points as i32));
                    obj.insert("abilities".into(), json!(st.ability_ids));
                    obj.insert(
                        "ability_levels".into(),
                        serde_json::Value::Object(ability_levels_map),
                    );
                }
            }
            // 注入最新的生命（從 GameLives 事件追蹤），以便 omfx 的
            // TD 模式偵測 + HUD 生命顯示在 Hero.stats 中保持正常運作。
            if let Some(lives) = hero_cache.latest_lives {
                if let Some(obj) = d.as_object_mut() {
                    obj.insert("lives".into(), json!(lives));
                }
            }

            GameEventData {
                topic: topic.clone(),
                msg_type: "hero".to_string(),
                action: "stats".to_string(),
                data: d,
                timestamp_ms,
                payload_bytes: logical_bytes,
                wire_bytes,
            }
        }
        game_event::Payload::GameRound(m) => {
            let d = json!({
                "round": m.round,
                "total": m.total,
                "is_running": m.is_running,
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::GameLives(m) => {
            // P3：快取以便稍後注入合併的hero.stats。
            hero_cache.latest_lives = Some(m.lives);
            let d = json!({ "lives": m.lives });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::GameEnd(m) => {
            let d = json!({ "winner": m.winner });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::GameExplosion(m) => {
            let pos = m
                .pos
                .as_ref()
                .map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q)))
                .unwrap_or((0.0, 0.0));
            let radius = m
                .radius
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let d = json!({
                "x": pos.0,
                "y": pos.1,
                "radius": radius,
                "duration": m.duration_ms as f32 / 1000.0,
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        // P9：HeroCreate / UnitCreate 可見性差異（佔位符 — omfx
        // 目前從 Creep.create 風格的有效負載中產生；這些會發出
        // 最小化創建 JSON，以便現有調度仍然可以看到它們）。
        game_event::Payload::HeroCreate(m) => {
            let pos = m
                .pos
                .as_ref()
                .map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q)))
                .unwrap_or((0.0, 0.0));
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let max_hp = m
                .max_hp
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "entity_id": m.id as u32,
                "name": m.name,
                "title": m.title,
                "position": { "x": pos.0, "y": pos.1 },
                "hp": hp,
                "max_hp": max_hp,
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        game_event::Payload::UnitCreate(m) => {
            let pos = m
                .pos
                .as_ref()
                .map(|p| (pos_dequant(p.x_q), pos_dequant(p.y_q)))
                .unwrap_or((0.0, 0.0));
            let hp = m.hp.as_ref().map(|f| fixed_dequant(f.v_q)).unwrap_or(0.0);
            let max_hp = m
                .max_hp
                .as_ref()
                .map(|f| fixed_dequant(f.v_q))
                .unwrap_or(0.0);
            let d = json!({
                "id": m.id as u32,
                "entity_id": m.id as u32,
                "name": m.name,
                "position": { "x": pos.0, "y": pos.1 },
                "hp": hp,
                "max_hp": max_hp,
            });
            GameEventData {
                data: d,
                ..default()
            }
        }
        // P9：低頻不規則事件的包羅萬象（init / ack /
        // 拒絕/庫存）。將攜帶的 JSON 位元組解碼回
        // serde_json::Value，以便 omfx 的現有調度程序看到原始數據
        // （訊息類型、操作、資料）形狀。
        game_event::Payload::LegacyJson(m) => {
            let data = if m.data_json.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::from_slice(&m.data_json).unwrap_or(serde_json::Value::Null)
            };
            GameEventData {
                topic: topic.clone(),
                msg_type: m.msg_type.clone(),
                action: m.action.clone(),
                data,
                timestamp_ms,
                payload_bytes: logical_bytes,
                wire_bytes,
            }
        }
    };
    Some(out)
}

#[cfg(test)]
mod seq_gap_tests {
    use super::{detect_seq_gap, SeqGapResult};

    #[test]
    fn initial_event_seeds() {
        // 沒有先前的 seq → 任何傳入的序列只是為追蹤器播種。
        assert_eq!(detect_seq_gap(None, 0), SeqGapResult::InitialSeed);
        assert_eq!(detect_seq_gap(None, 42), SeqGapResult::InitialSeed);
    }

    #[test]
    fn contiguous_is_ok() {
        assert_eq!(detect_seq_gap(Some(0), 1), SeqGapResult::Ok);
        assert_eq!(detect_seq_gap(Some(99), 100), SeqGapResult::Ok);
    }

    #[test]
    fn gap_of_one_detected() {
        // 最後=5，得到=7 → 錯過了#6。
        match detect_seq_gap(Some(5), 7) {
            SeqGapResult::Gap {
                expected,
                got,
                gap_size,
            } => {
                assert_eq!(expected, 6);
                assert_eq!(got, 7);
                assert_eq!(gap_size, 1);
            }
            other => panic!("expected Gap, got {:?}", other),
        }
    }

    #[test]
    fn large_gap_reports_size() {
        match detect_seq_gap(Some(100), 150) {
            SeqGapResult::Gap {
                expected,
                got,
                gap_size,
            } => {
                assert_eq!(expected, 101);
                assert_eq!(got, 150);
                assert_eq!(gap_size, 49);
            }
            other => panic!("expected Gap, got {:?}", other),
        }
    }

    #[test]
    fn backwards_flagged_not_gap() {
        // 重複或無序 — 不請求重新同步。
        assert_eq!(detect_seq_gap(Some(10), 10), SeqGapResult::Backwards);
        assert_eq!(detect_seq_gap(Some(10), 5), SeqGapResult::Backwards);
    }

    #[test]
    fn streaming_sequence_smoke() {
        // 模擬短流並斷言在正確位置檢測到間隙。
        let incoming = [0u64, 1, 2, 3, 5, 6]; // missed #4
        let mut last: Option<u64> = None;
        let mut gap_seen = false;
        for (i, &s) in incoming.iter().enumerate() {
            match detect_seq_gap(last, s) {
                SeqGapResult::InitialSeed => {
                    last = Some(s);
                }
                SeqGapResult::Ok => {
                    last = Some(s);
                }
                SeqGapResult::Backwards => {}
                SeqGapResult::Gap {
                    expected,
                    got,
                    gap_size,
                } => {
                    assert_eq!(i, 4, "gap at index {}", i);
                    assert_eq!(expected, 4);
                    assert_eq!(got, 5);
                    assert_eq!(gap_size, 1);
                    last = Some(s);
                    gap_seen = true;
                }
            }
        }
        assert!(gap_seen, "expected to observe one gap in the stream");
    }
}
