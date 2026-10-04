# MOBA 商店核心進度

此為前輪核心紀錄；正式權威輸入與 Lua catalog 的後續實作、最新測試及仍未開放的 secure／IPC 邊界，見 `2026-10-03-moba-shop-formal-input-progress.md`。以下「尚未接 PlayerInput／Lua」描述的是前輪狀態。

## 本輪計畫與修正

原先準備接經濟／擊殺／商店；檢查發現正式 PlayerInput 沒有購買／出售、Gold 未進 owner economy 投影，舊 JSON 商店固定基地原點。因此先補可測的權威交易核心與裝備生命週期，再接正式 protocol，避免無法 hash 驗證的旁路直接成為正式玩法。完整 5.3 維持未完成。

## 實作決定

- `runtime/native/shop.rs`：六格原子交易，所有檢查完成才扣錢／消耗材料。成品 cost 為總價，合成需備齊直接材料且扣差價；相同材料 ID 要不同實例。
- 售價整數 cost/2；拒絕 balance overflow、負 balance、非正 cost、非有限／負 bonus、無效 cooldown、未知／缺材料、self recipe、差價為負／加總溢位、空／越界格。完整 catalog cycle／引用生成驗證仍待共用 Lua 模型，不宣稱已有。
- `transact_moba_shop`：只能由權威呼叫並帶已認證 player_id，不接 client 自選 hero。查自己的單路 hero／基地，Playing、未暫停、存活、距基地不超過 300 才交易；拒絕不設定 dirty。
- 單路 spawn/respawn 加入 dirty ItemEffects，以既有 item_tick 套用／移除 bonus。保留既有背包與 Gold；不另寫 Unreal gameplay。
- 舊 TD／JSON 商店保留原狀；新 API 目前沒有 PlayerInput/IPC 路由。新測試用明確 fixture Gold 與 Rust ItemConfig，不宣稱已接 Lua 內容來源或玩家可操作商店。

## 已驗證

- core 最新 311 tests 全通過（7 個新 shop tests）；新 ECS shop test 通過，兩隊自己的基地、遠離基地、未知玩家、死亡、Pause／Warmup／Finished，以及購買／出售 stats 撤銷均覆蓋。額外驗證 i32::MAX 價格的整數退款、材料價加總溢位、負差額與負 balance，拒絕前後狀態完全相同。
- 新 respawn test 通過：保留金錢與裝備、新 generation 身分、HP／armor 不加倍。
- base_content 最新全套 66 pass，包含新 shop／respawn／pause 斷言及 15Hz／120Hz 完整 filtered lifecycle。單路權威 replay digest 加入 ItemEffects，避免裝備重算中間狀態被漏掉；修改 digest 後的完整 Bot 固定輸入逐 tick 雙次重播測試另行通過。
- omobab 139 pass／1 ignored。唯一新編譯錯誤 E0596 已修正並記錄 E084；既有 warnings 不隱藏。
- 修改 digest 後 client-runtime 41 lib＋3 bin pass、bridge 41 pass／1 capture ignored，相關 consumer 無編譯回退；scoped git diff --check 通過。

## 尚待工作

正式購買／出售 PlayerInput 與 IPC／回覆、內容 ID/catalog 版本一致、owner Gold／裝備投影與完整 filtered hash、擊殺／助攻／獎金、回城 channel、Unreal 商店／裝備 UI。此輪沒有重建／重啟 Unreal；前輪的 UE 終局證據只適用前輪 staged binaries。源碼已變動，正式重跑前必須完整重建 consumers／DLL，不能用 skip build 冒充最新。
