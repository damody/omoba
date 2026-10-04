# 商店輸入重送與 catalog 握手（2026-10-04）

## 問題與決定

- InputBuffer 原先每次追加同 ID 輸入，跨 tick／drain 的重送可能二次交易。選擇在 wire admission 層補 match-lifetime、player-scoped shop journal，wire ID 不進 gameplay／script ABI；不修改 TD 非商店輸入排序與既有 late grace。
- 同 ID、原始 target tick、Buy／Sell 命令完全相同，只回 DuplicateShop，保留第一次 effective tick，不能再排入；變更命令／tick、零 ID、已逐出的舊 ID 均拒絕。被拒絕的晚到原始請求保持第一次拒絕，即使重送改 grace 也不轉成功。
- 每玩家最近 1024 筆，以 expired-through watermark 保證逐出不重開舊交易；Buy 字串最多 128 bytes。不同玩家可用相同 ID，新交易保留抵達順序而非排序 input ID。
- JoinRequest tags 8／9 與 TeamGameStart tags 19／20 增加 shop_catalog_version／shop_catalog_hash。MOBA_ITEM_CATALOG_HASH 由既有 canonical hash 演算法生成，涵蓋價格／numeric ID／名稱／配方；屬於混版檢查，不是 cryptographic authentication。
- 宣告 catalog 必須是明確 secure V2 player、非零玩家、supported protocols 含 2，版本／hash 精確匹配，失敗在 roster 註冊前拒絕。沒有宣告的舊客戶端仍可用原協議；新版 selective client 必須收到匹配回覆，不靜默降級至無 catalog 檢查的 server。
- 不變更 C ABI 5／presentation IPC 3／secure network 2。generated protobuf fallback 由 vendored protoc 生成；Fyrox web 舊 Join initializer 同步。

## 實作範圍與限制

本輪是「catalog 相容」與「交易輸入不重排」的前提，不是開放購買。secure shop gate 仍 fail closed。

DuplicateShop 不表示交易成功，journal 不保存權威 ShopReceipt，也不重播 receipt。不宣稱 exactly-once settlement、跨 server restart 帳本或 runtime 重連後 ID 恢復已完成。過期／衝突的 transport result 與原始交易結果是不同資料，不能互換。

## 驗證

- core runtime-lua-content：320 passed。
- omobab lib：146 passed、1 ignored。新增 journal 的 drain 前後／跨 tick／retarget／保留 late reject／player 隔離／合成命令順序／Sell 重送／容量 watermark／超長字串；握手版本、hash、未知角色、observer、protocol、capability 與零身分拒絕；匹配 catalog 仍不能繞過 secure buy gate。
- catalog contract：1 passed，價格／ID／名稱／配方任一變動均改變 hash。
- client runtime：45 lib＋3 bin passed；bridge：44 passed、1 opt-in capture ignored。
- base_content：71 passed，含完整 15／120Hz lifecycle 與 shop filtered settlement；Fyrox cargo check --tests 通過。
- 原始未格式化來源與 rustfmt 後的 targeted journal 11 項、握手 1 項皆通過；scoped diff --check 通過。
- Unreal full build／staging／MCP 成功，C ABI header 不變；staged bridge SHA-256 `8c5c40fd690faeb5d20d52e6734c1faeb524daa5fa764eea5fc0eeda965208e2`，Editor PID 78632。最新同 Editor 兩輪各 8/8 passed、零 failed／skipped／not_run。
- 新握手真實雙隊 run `interactive-ue-1791046869` success=true：兩個 runtime 在正式 KCP join 收到 version 1／hash `61a5ab2c358ce0a7`，各自的自己的 economy／HUD／移動與四技能 4/4 通過，三方 hash 120／113 PASS、最後 tick 7920。證據 `target/interactive-runs/interactive-ue-1791046869/unreal-ipc-smoke-report.json` 與 runtime-p1／p2 stderr。這不是實際買賣驗收。
- 五程序退出核對：server 80184、runtimes 96132／12356、UE game clients 53996／17212。Editor 78632 留供開發；本輪不提交或清理既有工作樹變更。

## 接續計畫

1. 保存／重送原始 terminal shop receipt，而非只回入列 ACK。
2. 決定 match／player-scoped ID allocator 的重連恢復與 u32 耗盡契約，不能新 runtime 從 1 重用已完成交易 ID。
3. 在上述條件具備後啟用 secure shop intent、通用 Unreal 商店操作，驗收真實成功／非法拒絕、非空六格與 renderer 重連。
4. 再補正常收入、擊殺助攻、回城；不得以測試注入金錢作正式入口验收。

OpenSpec 4.1／5.3／6.2 保持未完成。錯誤與預防規則見 unreal-moba-error-register.md 的 E090。
