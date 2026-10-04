# MOBA 商店結算回覆與持續性 IPC 進度

## 本輪計畫與結果

沿用 `build-unreal-rust-moba-framework`，spec-driven，17／30 項完成。本輪補 5.3／4.1／6.2 的共同資料邊界，不把完整商店或完整 UI 勾選完成。

1. 已完成：權威交易結果與原始 correlation 對應，區分收件、輸入套用與交易成功。
2. 已完成：owner 金錢／六格背包的持續性 IPC、死亡保存、隊伍／玩家隔離、TCP 重連與有界交易歷史。
3. 尚未完成：C ABI／通用 Unreal HUD／商店輸入介面與實際雙 UE 驗收；正式 secure 網路買賣仍關閉。

## 問題與決定

- 不把 input ID 加進 PendingPlayerInputs 或 gameplay／scripts。ordered item drain 只產生 `ShopSettlement { player_id, command, result }`，每 tick 清空；投影層依同玩家原始命令順序精確核對，再附 CanonicalAcceptedInput correlation。相同命令也逐筆配對，不依 input ID 大小排序。命令不符即失敗，缺結果不假造成功。
- `ShopReceipt` kind 23：40 bytes、schema 1，player、correlation、settled tick、buy／sell、numeric catalog ID／slot、固定結果碼 0～13；沒有 entity／canonical source／價格／client 金額。未知商品拒絕使用 catalog ID 0；同一隊伍可收多玩家結果，但 IPC 僅保留配置玩家。
- Warmup／Pause／Finished 跳過 dispatcher 前先產生 MatchUnavailable；死者正常交易路徑產生 HeroUnavailable。server 可以保留 actor 0 的 shop acceptance 作 receipt correlation，但不發布不存在的 actor 或重演交易。非 roster 玩家仍過濾，不替未知玩家建立 actor。
- `OwnerEconomy` kind 24 使用 86-byte bounded state，與 live hero 身分分離。live hero 查權威 component；死者用 MobaHeroSlot 保存的 Gold／Inventory，不以 client cache 猜餘額。owner-team audience 與 fact team 必須相同；IPC 再按 player 選自己的 state。
- begin 的 PreStep 與 commit 的 PostStep 各可發布 economy；順序保證 IPC 取最後結算值，不顯示交易前餘額。shop_available 僅表示 Playing／未暫停／存活／自己基地 ≤300，不代表買得起。終局為 false。
- `TeamPresentationSnapshot` 新增 typed `owner_economy` 與 `shop_receipts`。保留現行 IPC v3 作可選、唯讀擴充；舊 consumer 可忽略，不能據此開放 shop command。未改 C ABI 4。
- runtime 在每個 Applied frame 收 receipt，早於 presentation divisor／catch-up 丟棄畫面。最近 64 筆結果跨最新 snapshot 覆蓋保留、同 tick／correlation 去重；新 view epoch 清空，超量淘汰最舊一筆並記診斷。這是有界近期狀態，不是永久帳本、交易 exactly-once 或伺服器跨重連歷史。
- 商店 accepted input 不再生成 APPLIED_TO_PRESENTATION 成功回覆；只有真正 receipt 才生成 SHOP_SETTLED 或 SHOP_INSUFFICIENT_GOLD 等具體失敗。非 shop 的舊回覆不变。結果-bearing snapshot 走 Critical，且不受畫面 divisor 擋下，先呈現狀態再回終態。
- renderer baseline 保留 economy／receipt，不把它們當成應清除的歷史 VFX。真實 TCP 兩次連線在沒有新模擬 frame 下取得同一筆金錢、六格與結果。

## 本輪驗證

- `cargo test --manifest-path omoba-core/Cargo.toml --features runtime-lua-content --lib`：319 passed。
- `cargo test --manifest-path scripts/Cargo.toml -p base_content --lib`：71 passed，含 15Hz／120Hz 完整 filtered 對局、逐 tick canonical hash、終局凍結、商店／合成／出售／非法拒絕與裝備死亡重生；沒有 ComponentRepair。
- `cargo test --manifest-path omoba-client-runtime/Cargo.toml`：45 lib＋3 bin passed。
- `cargo test --manifest-path omb/Cargo.toml -p omobab --lib`：140 passed／1 ignored，input ID 與 phase metadata guard 保持原門檻。
- `cargo test --manifest-path omfue/bridge/Cargo.toml --features runtime-driver --lib`：41 passed／1 ignored；此處僅證明現行 bridge 相容，未表示 C ABI 已輸出新 economy。
- scoped `git diff --check` 通過。新 Rust 模組使用 rustfmt，protobuf fallback 使用現有 vendored protoc build.rs 正式更新。

具體新增／擴充：core receipt 非單調 correlation／跨隊／命令不符／缺結果／非法 code 測試，零 tick 合法與無 actor 的死亡拒絕、owner audience 必須精確相符、owner codec 邊界，scripts inactive／dead／unknown player／stale report 清理，雙隊每 tick economy 私密性及死者 boots／Gold 550，runtime history 64 筆／去重／epoch／錯玩家／錯 tick，hub 未發布 step 仍保存結果，protobuf round trip，以及既有真實 TCP 重連 fixture 的 economy／receipt。

## 錯誤與限制

路徑猜錯、Inventory 無 PartialEq 的 E0369、scripts 測試讀 wire ID 觸發 metadata_guard，均已修正且記於 `unreal-moba-error-register.md` E088。没有放寬 hash／來源 guard，也沒有刪除失敗紀錄。

本輪沒有 stage DLL、重建 OmGame、重啟 Editor 或實際 UE 商店操作。現有 Gold 初始值仍為 0，起始金錢／被動收入／擊殺助攻／回城尚待規則實作。跨 runtime 重啟的永久 receipt、交易重送的 server exactly-once，以及完整能力／版本協商尚未封關。測試手動給 Gold 1000，不是正式遊戲起始金錢。

## 下一個實作順序

1. C ABI 明確升版並提供 typed 六格 economy／receipt lease；bridge 核對 configured player、catalog ID／slot／finite cooldown，不直接信任 protobuf。
2. 同步 header、codegen 共用 WorldBridge 模板、native 通用 HUD／shop 控制；從 Lua generated catalog 呈現商品，不新增角色專屬 C++ 或 Blueprint graph。
3. renderer shop intent／secure capability gate／catalog 一致性／重送防護接齊後，才開放正式網路買賣。死亡／Warmup／Pause／Finished 的拒絕必須到 UE。
4. 全建置、stage hash gate、同 Editor tests／PIE、雙 UE 合法購買／不足金錢／出售／死亡重生／畫面重連，保存新證據；不能挪用 2026-10-03 的 UE 成功。
5. 起始金錢與確定性收入、擊殺／助攻、回城；再扩完整 UI 與三路。不因本輪 IPC 子項通過把 5.3／6.2 整項勾選。
