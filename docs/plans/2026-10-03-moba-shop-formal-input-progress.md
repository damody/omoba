# MOBA 正式商店輸入與 Lua catalog

後續已完成 owner-team 經濟安全結算與裝備死亡重生的 filtered hash 驗證；最新進度見 `2026-10-03-moba-economy-projection-progress.md`。以下尚待工作描述的是本階段完成時的狀態；receipt／IPC／UE 商店仍未完成。

## 計畫與本輪實作

1. 將已驗證的原子商店 kernel 接入正式 PlayerInput，而非 JSON 命令旁路。
2. 保持買／賣／使用的同 tick 輸入順序，回歸拒絕不扣錢、暫停／終局、雙隊與逐 tick 重播。
3. 將 MOBA passive 物品納入既有 Lua → Rust catalog 生成，驗證配方與 compiled/runtime 相容限制。
4. 驗證依賴 consumers；記錄錯誤與尚未完成的網路／UI 邊界。

已實作以上程式變更；不是完整 5.3／6.2 驗收。

## 問題與決定

- PlayerInput 新增 ItemBuy（tag 17）與 ItemSell（tag 18），不重新配置舊 tag。購買只有 item_id，出售只有 slot；actor、價格、金錢、基地權限全部由權威查詢。fallback 用 core build.rs 的 vendored protoc 生成。
- PendingItemUseQueue 改為 PendingItemAction::Use／Shop 有序 enum，沿用同一 deterministic phase，避免先處理所有 use 再處理 buy/sell。正式權威 drain 呼叫既有 transact_moba_shop，失敗不修改金錢／背包。
- item_tick 先於 item phase；本 tick 交易後 ItemEffects.dirty=true，下一個 dispatch 套用／撤銷加成。測試明確覆蓋此順序，不自行補另一套 stats 計算。
- 新 templates/moba_items.lua 編譯生成 MOBA_ITEM_CATALOG：劍、甲、靴、兩把劍合成大劍；完整 templates data hash 已包含物品。單路 setup 安裝 generated_moba registry，舊 TD JSON loader 不變。
- 僅開放實際支援的 passive atk／hp／ms／armor；未支援 mana／active 等欄位以 deny_unknown_fields 拒絕。數值有限非負、有上限，配方材料最多六個，可重複材料；每條依賴價格嚴格增加（因此無 cycle），材料總價不能超過成品。
- 生成時把 bonus 量化至 Fixed64；Rust adapter轉為現有 ItemBonus 格式，不需要角色 C++、Blueprint 或獨立 runtime JSON。
- runtime Lua 值與 compiled catalog 不同時拒絕 MOBA 啟動；物品變更不允許 dev hot reload，需重建所有 peers。避免 hash 變了但 world ItemRegistry 沒變。
- secure V2 入口仍 fail closed 拒絕 shop，新增回歸防止意外開放。完整 owner 經濟結算、client 安裝／重播交易、結果 receipt、IPC 意圖與 UE 商店仍未完成；此輪不將 accepted-input 回覆視為交易成功。

## 驗證

- base_content 全套 69 pass，包含正式雙隊交易與 replay、buy→use→sell 順序、真正 Lua catalog 合成差價 250、stats 下個 tick 生效／撤銷、未知／非法／餘額不足原子拒絕、Pause／Finished。
- 15Hz 完整 filtered lifecycle：1618 ticks／3234 雙隊 steps，Finished tick 1604；120Hz：10698 ticks／21394 steps，Finished tick 10684。兩種 profile 各 team 的 deaths／respawns 都是 [7,1]，逐 tick 完整 hash 一致，沒有 ComponentRepair。
- core（runtime-lua-content）311 pass；template-ids 27 lib＋23／8 integration pass，含 catalog validation、compiled 不符拒絕、hot reload 限制及物品變更影響完整 data hash。script-abi 13 pass。
- omobab 140 pass／1 ignored；client-runtime 41 lib＋3 bin pass；Unreal bridge 41 pass／1 capture ignored。ignored 未算通過，既有 dead_code warnings 保留。
- 已修改 tracked 檔案的 root／omb scoped git diff --check 通過。沒有假造 UE runtime 商店驗收。

## 尚待完成與執行狀態

下一步是 owner Gold／Inventory／ItemEffects 的 typed committed 結算與視野安全 hash，之後才允許 secure shop 與 IPC／UE 操作。尚有經濟收入、擊殺助攻、回城、三路、完整 UI／LAN／效能驗收；OpenSpec 維持 17/30，不將部分實作勾成完成。

此輪沒有重建、stage 或重新跑 Unreal；catalog 改變後，舊 staged binaries 與前輪 UE 成功紀錄不代表目前 source。正式 UE 重跑需完整重建 consumers／DLL，禁止 skip-build。沒有 commit／push／cleanup 或新增 root wrapper。

錯誤與防重犯決定：`docs/plans/unreal-moba-error-register.md` E085／E086。
