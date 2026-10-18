//! 階段 3.4：鎖步輸入管道擁有的 ECS 資源。
//!
//! 位於“comp::”（不是“lockstep::”），因為：
//! 1. authoritative runtime dispatcher 在每個 tick 都會消耗它們，無論
//! kcp 鎖步傳輸是否處於作用中狀態。
//! 2. local lockstep replica worker 也寫入它們（將 `TickBatch`
//! 原始輸入轉成 shared runtime `PlayerInput` 類型），
//! 所以它們必須存在於以 *all* 功能編譯的模組中
//! 配置。 `lockstep::` 位於 `#[cfg(feature = "kcp")]` 後面。
//!
//! `PlayerInput`（prost 產生的原型類型從
//! `lockstep::PlayerInput`) 僅在 `feature = "kcp"` 下可用。到
//! 保持此模組始終編譯，我們將輸入儲存為不透明的
//! `serde_json::Value` 這裡是不可能的（有損+序列化一個空的
//! oneof 很尷尬）。相反，該資源擁有一個功能門控類型
//! 有效負載和消費者係統也具有門功能。非 kcp 建置獲取
//! 沒有任何內容寫入或讀取的空資源。

use std::collections::BTreeMap;

#[cfg(feature = "kcp")]
use crate::runtime::input::PlayerInput;

use super::TowerTargetPriority;
use omoba_sim::Vec2 as SimVec2;
use specs::Entity;

/// 從最新解碼的玩家輸入的每個刻度集合
/// `TickBatch`。每個刻度由消費者係統清除。 ‘勾選’紀錄
/// 這些輸入目標的鎖步刻度數 - 僅目前使用
/// 用於診斷日誌記錄/非同步追蹤。
#[cfg(feature = "kcp")]
#[derive(Default)]
pub struct PendingPlayerInputs {
    /// `(player_id, PlayerInput)` list for the current lockstep tick.
    /// Multiple inputs from the same player can land on one server tick when
    /// the client and server phases are close; preserving every entry avoids
    /// dropping input ids before the runtime applies them in arrival order.
    pub inputs: Vec<(u32, PlayerInput)>,
    /// 鎖步勾選輸入目標。消費者係統使用這個
    /// 僅適用於日誌上下文 - 實際副作用針對任何刻度
    /// 調度程序目前正在運行。
    pub tick: u32,
}

/// 非 kcp 建置：空標記，以便調度程式/系統程式碼可以讀/寫
/// 到處都沒有編譯時功能門的資源。
#[cfg(not(feature = "kcp"))]
#[derive(Default)]
pub struct PendingPlayerInputs;

/// 階段 2.1：延遲的塔生成請求源自
/// `PlayerInputEnum::TowerPlace`。僅鎖定“player_input_tick::Sys”
/// 具有系統資料存取權限（無“&mut World”），但需要“spawn_td_tower”
/// `&mut World` 查詢 `TowerTemplateRegistry` + 建立實體 + 推送
/// `ScriptEvent::Spawn`。所以我們在這裡排隊，並在 dispatcher 後由
/// authoritative runtime / local replica 使用相同 boundary drain。
///
/// 不變式：authoritative/local replica 每個 tick 都必須 drain，因此兩者
/// sims 保持 deterministic equivalence。`comp::GameProcessor::drain_pending_tower_spawns`
/// 是單一共用 drain 入口點。
#[derive(Default)]
pub struct PendingTowerSpawnQueue {
    pub requests: Vec<PendingTowerSpawn>,
}

#[derive(Clone, Debug)]
pub struct PendingTowerSpawn {
    pub kind_id: u32,
    pub pos: SimVec2,
    pub owner_pid: u32,
}

/// 階段 2.2：延遲的塔樓銷售請求源自
/// `PlayerInputEnum::TowerSell`。與“PendingTowerSpawnQueue”的基本原則相同：
/// 鎖定步驟 `player_input_tick::Sys` 僅有 SystemData 存取權限，但是
/// 出售塔需要 `&mut World`（讀範本註冊表以取得退款、寫 Hero Gold、
/// 刪除實體、清除 BuffStore）。所以我們在這裡排隊，並在 dispatcher
/// 後由 authoritative runtime / local replica 使用相同 boundary drain。
///
/// 不變式：authoritative/local replica 每個 tick 都必須 drain：
/// `comp::GameProcessor::drain_pending_tower_sells`。
#[derive(Default)]
pub struct PendingTowerSellQueue {
    pub requests: Vec<PendingTowerSell>,
}

#[derive(Clone, Debug)]
pub struct PendingTowerSell {
    pub tower_entity_id: u32,
    pub owner_pid: u32,
}

/// 沙箱/測試：延遲的 debug creep 生成請求，源自
/// `PlayerInputEnum::DebugSpawnCreep`（BTD6 沙箱「發送氣球」對應）。
/// 由 `creep_wave::Sys` 在每 tick 開頭 drain（該系統已有
/// creep_emiters / paths / outcomes 存取權限），主機與副本走同一條
/// lockstep 輸入路徑，維持 deterministic equivalence。
#[derive(Default)]
pub struct PendingDebugCreepSpawnQueue {
    pub requests: Vec<PendingDebugCreepSpawn>,
}

#[derive(Clone, Debug)]
pub struct PendingDebugCreepSpawn {
    pub emitter_index: u32,
    pub count: u32,
}

/// 階段 2.3：延遲的塔升級請求源自
/// `PlayerInputEnum::TowerUpgrade`。與「PendingTowerSellQueue」的基本原則相同：
/// 鎖定步驟 `player_input_tick::Sys` 僅有 SystemData 存取權限，但是
/// 升級塔需要 `&mut World`（讀 TowerUpgradeRegistry，寫
/// `Gold` / `Tower` / `BuffStore`，透過 tower_upgrade_rules 驗證）。所以我們
/// 在這裡排隊，並在 dispatcher 後由 authoritative runtime / local replica
/// 使用相同 boundary drain。
///
/// 不變式：authoritative/local replica 每個 tick 都必須 drain：
/// `comp::GameProcessor::drain_pending_tower_upgrades`。
#[derive(Default)]
pub struct PendingTowerUpgradeQueue {
    pub requests: Vec<PendingTowerUpgrade>,
}

#[derive(Clone, Debug)]
pub struct PendingTowerUpgrade {
    pub tower_entity_id: u32,
    pub path: u8,
    /// 目標等級（升級後）。階段 2.3：客戶端可以發送“0”，如果
    /// 尚未透過快照觀察到 `Tower.upgrade_levels` — omb
    /// 處理程序根據實體的當前計算實際目標
    /// 在這種情況下為「upgrade_levels[path] + 1」。
    pub level: u8,
    pub owner_pid: u32,
}

/// 來自 `PlayerInputEnum::UpgradeAbility` 的延遲英雄技能升級請求。
/// 更新 `Hero.ability_levels`、消耗 `skill_points`、讀取 `AbilityRegistry`，
/// 以及推送 `ScriptEvent::SkillLearn` 都需要 `&mut World`，所以 input system
/// 先在這裡排隊，再由 `GameProcessor` 於 dispatch 後 drain。
///
/// 不變式：authoritative/local replica 每個 tick 都必須透過
/// `comp::GameProcessor::drain_pending_ability_upgrades` drain。
#[derive(Default)]
pub struct PendingAbilityUpgradeQueue {
    pub requests: Vec<PendingAbilityUpgrade>,
}

#[derive(Clone, Debug)]
pub struct PendingAbilityUpgrade {
    pub ability_index: u32,
    pub owner_pid: u32,
}

/// 來自 `PlayerInputEnum::CastAbility` 的延遲英雄施法請求。
/// 解析 caster、驗證已學習/冷卻狀態，以及推送 `ScriptEvent::SkillCast`
/// 都需要 `&mut World`，所以施法沿用和升級相同的 queue + drain pattern。
#[derive(Default)]
pub struct PendingAbilityCastQueue {
    pub requests: Vec<PendingAbilityCast>,
}

#[derive(Clone, Debug)]
pub struct PendingAbilityCast {
    pub ability_index: u32,
    pub target_pos: Option<SimVec2>,
    pub target_entity: Option<u32>,
    pub owner_pid: u32,
}

/// Slot-only summoner requests admitted by the wire gate. Spell identity stays
/// on the roster. Receipts record admission separately from settlement.
#[derive(Default)]
pub struct PendingSummonerCastQueue {
    pub tick: Option<u32>,
    pub requests: Vec<PendingSummonerCast>,
    pub receipts: Vec<SummonerCastReceipt>,
}

#[derive(Clone, Debug)]
pub struct PendingSummonerCast {
    pub owner_pid: u32,
    pub request: crate::runtime::summoner_runtime::SummonerCastRequest,
}

/// Slot-0 ward requests admitted by the wire gate. Catalog stays on the server.
/// `source_generation` is the actor life captured at admission. Drain does not
/// rebind it, so a replacement cannot spend.
#[derive(Default)]
pub struct PendingWardPlacementQueue {
    pub tick: Option<u32>,
    pub requests: Vec<PendingWardPlacement>,
    pub receipts: Vec<WardCommandReceipt>,
}

#[derive(Clone, Debug)]
pub struct PendingWardPlacement {
    pub owner_pid: u32,
    pub local_slot: u8,
    pub x_raw: i64,
    pub y_raw: i64,
    pub source_generation: Option<u32>,
    /// Lockstep acceptance correlation. Zero is not an admitted command.
    pub admitted_correlation: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WardCommandReceipt {
    pub player_id: u32,
    pub slot: u32,
    pub admitted_correlation: u32,
    pub stage: crate::runtime::ward_transport::WardReceiptStage,
    pub code: &'static str,
    /// Actor life captured at admission. `None` stays generation 0 on the wire.
    pub actor_generation: Option<u32>,
}

/// Owner-and-epoch journal of at most eight explicit ward outcomes.
/// A ninth correlation retires the oldest into `uncertain_through`.
/// An epoch change drops that owner's rows. This is not a second command route.
#[derive(Clone, Debug, Default)]
pub struct OwnerWardOutcomeJournal {
    owners: std::collections::BTreeMap<u32, OwnerWardJournalRow>,
}

#[derive(Clone, Debug)]
struct OwnerWardJournalRow {
    epoch: u64,
    uncertain_through: u32,
    rows: Vec<crate::runtime::ward_transport::OwnerWardOutcome>,
}

impl OwnerWardOutcomeJournal {
    pub fn note(&mut self, player: u32, epoch: u64, outcome: crate::runtime::ward_transport::OwnerWardOutcome) {
        use crate::runtime::ward_transport::{
            OWNER_WARD_OUTCOME_ADMITTED, OWNER_WARD_OUTCOME_CAPACITY, OWNER_WARD_OUTCOME_REJECTED, OWNER_WARD_OUTCOME_SETTLED,
        };
        if player == 0 || epoch == 0 || outcome.admitted_correlation == 0 || outcome.slot != 0 {
            return;
        }
        if !matches!(outcome.stage, OWNER_WARD_OUTCOME_ADMITTED | OWNER_WARD_OUTCOME_SETTLED | OWNER_WARD_OUTCOME_REJECTED) {
            return;
        }
        let row = self.owners.entry(player).or_insert_with(|| OwnerWardJournalRow {
            epoch,
            uncertain_through: 0,
            rows: Vec::new(),
        });
        if row.epoch != epoch {
            *row = OwnerWardJournalRow { epoch, uncertain_through: 0, rows: Vec::new() };
        }
        if outcome.admitted_correlation <= row.uncertain_through {
            return;
        }
        if let Some(existing) = row.rows.iter_mut().find(|item| item.admitted_correlation == outcome.admitted_correlation) {
            if existing.stage == OWNER_WARD_OUTCOME_ADMITTED && outcome.stage != OWNER_WARD_OUTCOME_ADMITTED {
                *existing = outcome;
            }
            return;
        }
        row.rows.push(outcome);
        row.rows.sort_by_key(|item| item.admitted_correlation);
        while row.rows.len() > OWNER_WARD_OUTCOME_CAPACITY {
            let dropped = row.rows.remove(0);
            row.uncertain_through = row.uncertain_through.max(dropped.admitted_correlation);
        }
    }

    pub fn block_for(&self, player: u32, epoch: u64) -> crate::runtime::ward_transport::OwnerWardOutcomeBlock {
        use crate::runtime::ward_transport::OwnerWardOutcomeBlock;
        let Some(row) = self.owners.get(&player) else {
            return OwnerWardOutcomeBlock::default();
        };
        if row.epoch != epoch {
            return OwnerWardOutcomeBlock::default();
        }
        let mut block = OwnerWardOutcomeBlock {
            uncertain_through: row.uncertain_through,
            count: row.rows.len() as u8,
            rows: [crate::runtime::ward_transport::OwnerWardOutcome::default(); crate::runtime::ward_transport::OWNER_WARD_OUTCOME_CAPACITY],
        };
        for (index, item) in row.rows.iter().enumerate() {
            block.rows[index] = *item;
        }
        block
    }
}

/// One admitted command's trusted actor life, aligned with `PendingPlayerInputs`.
#[derive(Clone, Copy, Debug, Default)]
pub struct AdmittedCommandIdentity {
    pub player_id: u32,
    pub correlation: u32,
    pub actor_generation: Option<u32>,
}

#[derive(Default)]
pub struct AdmittedCommandIdentities {
    pub rows: Vec<AdmittedCommandIdentity>,
}

/// Per-owner view epoch used to retire ward command retention.
#[derive(Default)]
pub struct WardAdmissionEpoch {
    pub by_player: std::collections::BTreeMap<u32, u64>,
}

const WARD_COMMAND_RETENTION: usize = 256;

#[derive(Clone)]
struct OwnerWardRetention {
    epoch: u64,
    expired_through: u32,
    order: std::collections::VecDeque<u32>,
    closed: std::collections::BTreeSet<u32>,
}

impl Default for OwnerWardRetention {
    fn default() -> Self {
        Self {
            epoch: 0,
            expired_through: 0,
            order: std::collections::VecDeque::new(),
            closed: std::collections::BTreeSet::new(),
        }
    }
}

/// Bounded closed command correlations. A second settle does not spend.
/// Changing the owner's epoch drops that owner's retention.
#[derive(Default)]
pub struct WardSettledInputIds {
    owners: std::collections::BTreeMap<u32, OwnerWardRetention>,
}

impl WardSettledInputIds {
    pub fn observe_epoch(&mut self, player: u32, epoch: u64) {
        if let Some(row) = self.owners.get_mut(&player) {
            if row.epoch != epoch {
                *row = OwnerWardRetention { epoch, ..OwnerWardRetention::default() };
            }
        }
    }

    pub fn is_closed(&self, player: u32, epoch: u64, correlation: u32) -> bool {
        let Some(row) = self.owners.get(&player) else {
            return false;
        };
        row.epoch == epoch
            && correlation != 0
            && (correlation <= row.expired_through || row.closed.contains(&correlation))
    }

    pub fn close(&mut self, player: u32, epoch: u64, correlation: u32) {
        if player == 0 || correlation == 0 {
            return;
        }
        let row = self.owners.entry(player).or_default();
        if row.epoch != epoch {
            *row = OwnerWardRetention { epoch, ..OwnerWardRetention::default() };
        }
        if row.closed.insert(correlation) {
            row.order.push_back(correlation);
            while row.order.len() > WARD_COMMAND_RETENTION {
                if let Some(expired) = row.order.pop_front() {
                    row.closed.remove(&expired);
                    row.expired_through = row.expired_through.max(expired);
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummonerCastReceipt {
    pub player_id: u32,
    pub slot: u32,
    pub stage: crate::runtime::summoner_transport::SummonerReceiptStage,
    pub code: &'static str,
    pub effect_class: Option<&'static str>,
}

/// Deterministic tower active-ability pulse opportunities awaiting script
/// dispatch. Dispatch drains these records before the tower scheduler advances
/// the same tower on the next authoritative tick.
#[derive(Default)]
pub struct PendingTowerAbilityPulseQueue {
    pub requests: Vec<PendingTowerAbilityPulse>,
}

#[derive(Clone, Debug)]
pub struct PendingTowerAbilityPulse {
    pub entity: Entity,
    pub ability_id: String,
    pub activation_serial: u32,
    pub pulse_index: u16,
}

/// Lockstep tower active-ability requests routed by `player_input_tick::Sys`.
/// The queue is drained at the same deterministic boundary on host and replica.
#[derive(Default)]
pub struct PendingTowerAbilityCastQueue {
    pub requests: Vec<PendingTowerAbilityCast>,
}

#[derive(Clone, Debug)]
pub struct PendingTowerAbilityCast {
    pub tower_entity_id: u32,
    pub ability_id: String,
    pub owner_pid: u32,
}

/// Accepted activations waiting for Task 5's script ABI dispatch.
#[derive(Default)]
pub struct PendingTowerAbilityActivationQueue {
    pub requests: Vec<PendingTowerAbilityActivation>,
}

#[derive(Clone, Debug)]
pub struct PendingTowerAbilityActivation {
    pub entity: Entity,
    pub ability_id: String,
    pub activation_serial: u32,
}

/// Latest authoritative outcome of a processed tower ability cast.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TowerAbilityCastResult {
    pub player_id: u32,
    pub tower_entity_id: u32,
    pub ability_id: String,
    pub accepted: bool,
    pub reason: String,
    pub result_serial: u32,
}

/// Latest authoritative tower ability cast outcome for every player.
///
/// `BTreeMap` makes iteration and render snapshot extraction deterministic on
/// both the authoritative server and local lockstep replicas.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TowerAbilityCastResults {
    pub latest_by_player: BTreeMap<u32, TowerAbilityCastResult>,
}

/// ItemUse / ItemBuy / ItemSell share one ordered item-processing phase.
/// 與其他待處理隊列的基本原理相同：
/// `use_item` 讀取 `ItemRegistry` 資源 + 寫入 `Inventory` + 寫入
/// `CProperty` + 查詢 `Hero`/`Faction` 存儲，其中沒有一個
/// 可從規格“System”SystemData 存取。
///
/// 不變式：authoritative/local replica 每個 tick 都必須 drain：
/// `comp::GameProcessor::drain_pending_item_uses`。
#[derive(Default)]
pub struct PendingItemUseQueue {
    /// Preserve uses and shop transactions in formal input order.
    pub requests: Vec<PendingItemAction>,
    /// Tick-local authority results, consumed by the team projector.
    pub settlements: Vec<crate::runtime::shop::ShopSettlement>,
    /// Undo results. Separate from the buy/sell FIFO.
    pub undo_settlements: Vec<crate::runtime::shop::ShopUndoSettlement>,
}

#[derive(Clone, Debug)]
pub enum PendingItemAction {
    Use(PendingItemUse),
    Shop {
        owner_pid: u32,
        command: crate::runtime::shop::ShopCommand,
    },
    Undo {
        owner_pid: u32,
        request: crate::runtime::shop::ShopUndoRequest,
    },
}

#[derive(Clone, Debug)]
pub struct PendingItemUse {
    pub item_slot: u32,
    pub target_pos: Option<SimVec2>,
    /// Legacy scalar coordinates are not point presence. True only for an explicit point.
    pub point_explicit: bool,
    /// Command index only. It is not a life and it is not an origin.
    pub target_entity: Option<u32>,
    /// Local generation supplied by secure rewrite or filtered injection.
    /// Absent means the in-process command did not name a life. Zero is rejected
    /// rather than resolved to whatever entity now owns the index.
    pub target_generation: Option<u32>,
    pub owner_pid: u32,
    /// Trusted owner index stamped at acceptance. Not an ECS `Entity` and not a canonical id.
    pub owner_index: Option<u32>,
    /// Paired with `owner_index`. The first bind must use this pair. Zero is not a life.
    pub owner_generation: Option<u32>,
    /// Set by `bind_queued_item_lives`. Drain must not resolve the index again.
    pub lives_bound: bool,
    pub owner_entity: Option<Entity>,
    pub target_life: Option<Entity>,
}

/// Hero command requests from MoveTo / AttackMove / AttackTarget. They are
/// routed through one queue so Shift-appended commands can share validation,
/// replacement, and the 16-command cap.
///
/// 不變式：authoritative/local replica 每個 tick 都必須 drain：
/// `comp::GameProcessor::drain_pending_moves`。
#[derive(Default)]
pub struct PendingMoveQueue {
    pub requests: Vec<PendingHeroCommand>,
}

#[derive(Clone, Debug)]
pub struct PendingHeroCommand {
    pub owner_pid: u32,
    pub queued: bool,
    pub kind: PendingHeroCommandKind,
}

#[derive(Clone, Debug)]
pub enum PendingHeroCommandKind {
    HoldPosition,
    /// One-shot clear. It never becomes a persistent `HeroCommand`.
    Stop,
    MoveTo { pos: SimVec2 },
    AttackMove { pos: SimVec2 },
    AttackTarget { target_entity_id: u32 },
}

#[derive(Default)]
pub struct PendingHeroCommandClearQueue {
    pub requests: Vec<u32>,
}

#[derive(Default)]
pub struct PendingTowerTargetPriorityQueue {
    pub requests: Vec<PendingTowerTargetPriority>,
}

#[derive(Clone, Debug)]
pub struct PendingTowerTargetPriority {
    pub tower_entity_id: u32,
    pub priority: TowerTargetPriority,
    pub owner_pid: u32,
}

/// 階段 5.3：觀察者重新加入的最新序列化世界快照。
///
/// 更新每個“SNAPSHOT_INTERVAL_TICKS”調度程序刻度（= 30 s @ 120 Hz）。
/// 由 KCP 傳輸的 0x16 SnapshotResp 處理程序用於引導
/// 觀察者客戶端在遊戲中期連接；觀察者將位元組應用到
/// 然後它的 sim_runner 透過後續的 TickBatches 向前播放。
///
/// `bytes` 透過 `omoba_sim::snapshot::serialize` 進行二進位碼序列化
/// 元件的穩定子集（`id` + `pos` + `vel` + `faceing` + `hp`/`mhp`
/// +「種類」）。架構透過「WorldSnapshot::schema_version」固定在內部
/// `lockstep::snapshot_ Producer` — 改變線上格式需要
/// 協調兩端。
///
/// 空位元組（`tick = 0`）表示尚未儲存快照； KCP 處理程序
/// 按原樣返回它，觀察者會從“current_tick”開始播放。
#[derive(Default)]
pub struct SnapshotStore {
    /// 勾選快照的拍攝地點。 `0` = 還沒有快照。
    pub tick: u32,
    /// bincode 序列化的「WorldSnapshot」（實體 + Pos / Vel / Facing /
    /// CProperty子集+master_seed+tick+schema_version）。
    pub bytes: Vec<u8>,
}
