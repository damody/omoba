# Unreal owner 經濟 HUD（2026-10-04）

## 決定與已實作

1. 沿用 owner-only IPC schema，不新增 UE 玩法計算。bridge 依明確 configured player_id 驗證 Gold、六個唯一 slot、compiled catalog、finite bounded cooldown、最多 64 個唯一權威 receipt；錯誤 owner／future tick／錯誤版本／result 拒絕呈現。
2. C ABI 5 新增 OmOwnerEconomy／OmInventorySlot／OmShopReceipt。指標與 Lua catalog 物品名稱字串由 frame lease 擁有；control frame 清空、不借用 protobuf 臨時資料。header 由既有 cbindgen 流程產生。
3. OmWorldBridgeActor 是共用 framework 程式，不是目前 codegen 每次產生的英雄類別。此次修改共用事件轉接一次，不新增英雄專屬 C++、Blueprint graph 或美術要求。
4. 原生 Slate 增加獨立金錢／六格背包／最新交易回覆，避免 MatchStatus 覆蓋金錢。字串取 Rust 的 compiled Lua catalog，未收到安全狀態時明確 unavailable；完整空 view 重設、control frame 保留 UI。
5. 商店 UI 明確 read-only。權威距離／phase gate 與 secure transport 支援不是同一件事；購買／出售仍 fail closed。沒有新增收入、回城或 initial Gold。

## 驗證

- bridge runtime-driver lib：44 passed、0 failed、1 opt-in real capture ignored。
- 包含 owner 7（不是 team id）的身分隔離、六格／NaN／未知 catalog／負金錢、receipt code／schema／correlation／future tick／duplicate／容量，以及 frame-slot-owned pointer／字串／control 清空測試。
- 真實 publish_snapshot 路徑驗證 compiled Lua 名稱／價格、busy ring 保留資料、empty complete view 清空；不是手填名稱 fixture 代替 catalog 轉換。
- Unreal full build 成功，最新 staged bridge SHA-256 `6a9e2bcb2de55ae6a8a464f922c19d516a544c85e162cb16ba616e18d7075168`；Editor PID 86648、MCP HTTP 30000 ready。原先 PID 61352 已由 scoped restart 停止。
- 同 Editor PID 61352 與最新版 PID 86648：各完成兩輪 8/8、零 failed／skipped／not_run；含擴充 UiOverlaySurface 的 economy reflected payload／delegate。Ghost fixture 有既有 EditorPreview world-context warning，不宣稱零警告。
- PIE smoke 通過並停止自啟動的 PIE；檢查截圖 `omfue/Saved/McpAutomation/pie-smoke.png`，舊模式安全狀態缺席時顯示 Economy unavailable，不捏造 Gold 或背包。此截圖不是正式 MOBA 商店驗收，測試瞬時 FPS 3 不是效能基線通過。
- codegen --check：11 generated files／14 Lua inputs 通過；共用 WorldBridge 手工框架檔並非 codegen 輸出，未覆蓋掉此次修改。
- 真實雙隊 single-lane run `interactive-ue-1791045376` 通過：兩個 UE 正式 presentation adapter 均輸出自己的 authority Gold 0／slots=6，離開基地後 shop 從 1 變 0；每隊四技能 4/4 真實輸入／結算／冷卻通過，三方 hash 114／106 PASS、最後 tick 7800。證據 `target/interactive-runs/interactive-ue-1791045376/unreal-ipc-smoke-report.json` 與兩隊 stdout。
- Gold 0 是目前無收入的啟動規則，不代表收入完成；沒有注入 fixture Gold／物品或 bypass secure shop。真實非空物品／receipt UE 畫面仍待正式交易入口，Rust 的完整 converter／lease／busy retry 回歸不等於其畫面验收。
- 測試五程序已確認退出：server 70948、runtime 88836／63064、UE game clients 93096／78432；Editor 86648 保留供後續開發。staged SHA 最後再核對一致；本輪 scoped diff --check 通過。

## 下一輪實作順序

先補 secure shop 的 catalog 相容與明確交易 terminal-result／重送契約，再開買賣 intent 與通用商店操作；接著以真正權威交易驗收非空六格／拒絕與成功 receipt／renderer 重連，最後補收入／擊殺助攻／回城。不得為了讓畫面有物品而注入測試金錢、直接呼叫交易 kernel 或移除原有 fail-closed gate。

## 仍未完成

- 真正 secure 購買／出售、重送／冪等性／catalog 握手、收入／擊殺助攻／回城。
- 選角、小地圖、計分板、三路／野區、五位置 Bot、多英雄與完整效能／LAN 驗收。
- OpenSpec 4.1／5.3／6.2 維持未完成，不以唯讀 HUD 勾選整個商店或完整框架。
