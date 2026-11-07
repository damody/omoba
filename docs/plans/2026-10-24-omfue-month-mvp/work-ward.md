# work-ward：owner ward envelope 被 client 判為 NONCANONICAL

```text
日期：2026-10-25
目前工作 ID／狀態：ward 投影拒絕（影響 checklist 3.9 ward）／程式修正完成，待主 session 重建與 L3 實機驗證
玩家或作者新增能做到的事：client runtime 不再持續丟棄 owner ward 投影；ward 充能、冷卻、可放置狀態能持續送到 UE。
repo HEAD／本次修改：HEAD b1f00b9（未 commit）；
  omoba-core/src/runtime/ward_transport.rs（新增 seal_stored_owner_ward_envelope、拒絕碼細分、重現測試）
  omoba-core/src/runtime/selective_replica.rs（同 tick／revision 不同 bytes 改回 NONCANONICAL_OWNER_WARD_SAME_TICK_BYTES）
  omb/src/transport/kcp_transport.rs（continuing／rebase／recovery 三條路徑都以 store tick 封裝；移除 last_owner_ward_tick）
  omb/src/state/core.rs（StoredTeamBootstrap 註解）
來源英雄與 variant：不適用
已執行命令、exit code、build identity：見下方「測試」
L1／L2／L3／L4 證據（fresh／沿用／未執行）：L1 fresh（單元測試）；L3 未執行（依指示由主 session 執行）
gate 與原 11 主項的變化：3.9 ward 的「投影被拒」阻斷已移除，需 L3 重驗
失敗、原因、尚未修的風險：見下方「風險」
下一個工作 ID：主 session 重建 omobab + omoba-client-runtime 後重跑 B+C+C
人工／硬體需求與是否已取得：無
owned process／資源清理結果：未啟動任何 process；cargo 只寫 target/session-ward
```

## 實際觸發的分支

`runtime-p1.stderr.log` 的 6834 次拒絕全部來自 `selective_replica.rs` 的
「同 authoritative_tick、同 revision、bytes 不同」分支，不是 `decode_owner_ward_private` 的欄位規則。
依據：server 的 `project_private_owner_ward` 經 `encode_owner_ward_private` 產生 bytes，
而 encode 本身會先跑同一個 decode，欄位不 canonical 時會回空 bytes（= Clear），不會送出能被 decode 拒絕的投影。
重現測試在修正前的失敗訊息與 log 一致：`left: Err("NONCANONICAL_OWNER_WARD")`。

## 根因

server 封裝 envelope 的 tick 與 bytes 不是同一個狀態：

- `omb/src/state/core.rs` 每個 server step 重建 `StoredTeamBootstrap`，`owner_wards` 由當下 world 投影，
  bytes 內含 `elapsed_raw`，因此每個 step 都不同；同一份 store 的 `start` tick 是 `local_tick + 1`。
- `omb/src/transport/kcp_transport.rs` 的 continuing dispatch 卻用「觸發這次 dispatch 的 team frame 的
  `replica_tick`」封裝 store 裡最新的 bytes。projection barrier 未到時，`latest_frames` 不更新，
  同一個 replica_tick 每個 step 都被重送；frame tick 也落後 live world。
- 於是 client 在同一個 (tick, revision) 下收到一串不同的 bytes。client 規則「同 tick+revision 必須同 bytes」
  是既有設計（`verified_owner_ward_receiver_rejects_unverified_order_and_restores_by_recovery` 明確測 twin 被拒），
  錯的是 server。
- recovery 用 `max(store tick, 上次送出的 tick)` 配上「現在的」bytes，同樣可能產生 twin；
  rebase manifest dispatch 用 manifest tick，也有相同問題。

## 修正

- 新增 `ward_transport::seal_stored_owner_ward_envelope(start, player, session, bytes)`：
  envelope tick 一律是該 bytes 所屬 store 的 bootstrap tick（`owner_ward_bootstrap_tick(start)`）。
- omb 三條路徑（每 frame continuing、rebase 後強制重送、recovery query）都改用 store tick；
  `OwnerWardDispatchRow` 只多帶 `view_epoch` 和 `tick`（不 clone 整個 bootstrap，避免熱路徑成本）。
  `ClientSession.last_owner_ward_tick` 已無讀者，移除。
- store tick 每 step 單調遞增，且 tick 與 bytes 在同一個 mutex 下整份替換，所以同一 tick 只會有一種 bytes；
  client 驗證規則未放寬。
- 拒絕原因細分（皆保留 `NONCANONICAL_OWNER_WARD_` 前綴）：
  `_PLAYER`、`_ELAPSED`、`_RECHARGE_NEGATIVE`、`_CAPABLE_FLAG`、`_UNEQUIPPED_FIELDS`、`_CHARGES`、
  `_RECHARGE_RANGE`、`_FULL_RECHARGE`、`_CAPABLE_EMPTY`、`_CLOCK_OVERFLOW`、`_OUTCOME_HOLE`、
  `_OUTCOME_RECORD`、`_OUTCOME_ORDER`，以及 replica 的 `_SAME_TICK_BYTES`。

## 重現／回歸測試

`omoba-core/src/runtime/ward_transport.rs`
`continuing_owner_ward_envelopes_from_the_live_store_stay_canonical_while_frame_ticks_repeat`：
用真實 `setup_single_lane_match` + `set_authoritative_trinket_loadout` + `project_private_owner_ward`
產生 bytes，join 後推進 elapsed、frame tick 固定，經 client `apply_verified_owner_ward_envelope` 驗證。
修正前（以 frame tick 封裝）在 server tick 11 失敗；修正後每次都接受，同一 store 的 recovery 回覆被視為同一份，
舊的 frame-tick 封裝變成 `OWNER_WARD_STALE`，真正的 twin 仍是 `NONCANONICAL_OWNER_WARD_SAME_TICK_BYTES`。

（原本想放在 `scripts/base_content/src/ward_transport_tests.rs`，但該 crate 的 test 目前因另一個 session
進行中的 `generic_effects.rs`（Projectile 未窮舉、`div_ceil` unstable）無法編譯，因此改放 omoba-core。）

## 測試

所有命令皆設 `CARGO_TARGET_DIR=D:\omoba\target\session-ward`。

| 命令 | exit | 結果 |
|---|---|---|
| `cargo test --manifest-path omoba-core/Cargo.toml --lib continuing_owner_ward_envelopes`（修正前） | 101 | 重現：server tick 11 回 `Err("NONCANONICAL_OWNER_WARD")` |
| `cargo test --manifest-path omoba-core/Cargo.toml --lib --tests` | 101 | lib 1109 passed／0 failed；`tests/td_autoplay_100.rs` 3 個失敗，原因是 `no scripts loaded from ../scripts/target/release`（需要 release DLL，與本修正無關，未改動） |
| `cargo test --manifest-path omoba-client-runtime/Cargo.toml` | 0 | 160 passed、8 ignored，其餘 test binary 全過 |
| `cargo test --manifest-path omb/Cargo.toml -p omobab ward` | 0 | 5 passed（omb 編譯通過，含 kcp_transport 變更） |

`cargo test -p omoba-core` 不加 `--lib --tests` 時，examples 因既有的 `WaveBReadView.brush_regions` 缺欄位而編不過（與本修正無關）。

建置凍結（`target/BUILD_FREEZE`）是在 Rust 修改完成並編譯通過後才出現；之後沒有再改任何原始碼。主 session 這次一致性重建若在本修改之後取得原始碼，就已經包含此修正。

## 主 session 需要重建

- `omobab`（server：dispatch／recovery 封裝 tick 改變）
- `omoba-client-runtime`（拒絕碼字串改變；驗證規則不變）
- `base_content.dll` 不需要重建（未改 scripts）。UE／bridge 不需要重建（未改 bridge 契約）。
- 協定版本未改；新 server 送出的 envelope 舊 client 也能接受，反之舊 server 仍會觸發舊問題。

## 風險

- omb submodule 有變更但未 commit（依指示）；commit 時需先在 omb commit／push 再 bump gitlink。
- L3 尚未驗證：預期 `owner ward envelope rejected` 不再出現；若仍出現，新 log 會帶細分的原因碼。
