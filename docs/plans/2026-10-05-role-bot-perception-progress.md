# 五位置 Bot 安全資訊邊界（2026-10-05）

## 本批計畫與決策

1. 保留 single_lane_bot_inputs Push/Guard 作為舊 fixture；它直接讀取敵人 authority state，不冒充正式五位置 Bot。
2. 新增 moba_match::bots 的 BotRole／BotAssignment／RoleBotConfig／role_bot_inputs。玩家 ID、角色、lane index 與思考間隔可配置，驗 roster、同隊角色唯一、player 唯一、lane 存在、jungle camp 存在與非零間隔。
3. planner 不取得 World／MobaMatch，只收到當隊 index.current 與 committed render payload 衍生的資料。敵人記憶、隱藏 HP／位置、camp respawn／aggro 不參與決策。缺少或落後超過一 tick 的 Wave B snapshot 停用，不退回全知 Bot。
4. route 是公開 compiled 地圖資訊；野區巡邏依公開營地座標及 tick，並非依私有 camp 存活狀態。四個 lane role 目前共用路線與普攻基礎策略，Jungle 僅選披露的 neutral kind3。角色專屬戰術尚未完成。
5. 只產生正式 PlayerInput AttackMove／AttackTarget；不修改 HP／Pos／orders。讀自己健康與 command queue 去重，死亡／Recall／pause／非 Playing 不行動。目標 generation admission 檢查不讀其私有 components；kind0 建築不作直接選取，仍由正式 AttackMove 與權威建築 gate 處理。
6. 不猜技能槽位用途，不為各英雄新增 C++／Blueprint，也不改 ABI。本批不重啟 Unreal、不 build/stage DLL、不跑完整驗收。

## 當前功能確認

- cargo test --manifest-path omb/Cargo.toml -p omoba-core role_bots --lib：3/3 passed，exit0。涵蓋 hidden cache 位置改變不影響目標、Hide 即不選取、跨隊白名單、友軍／未知 kind 排除、Jungle neutral 選取、固定 tie-break／輸入順序獨立、canonical generation layout。
- cargo test --manifest-path scripts/Cargo.toml -p base_content role_bots_five_positions -- --nocapture：1/1 passed，exit0。兩隊各五名角色、10 人 roster，60Hz 正式 driver／60 ticks，全部英雄透過正式輸入位移；配置錯誤拒絕、缺視野不行動、未提交的 authority 位置改變不影響計畫。
- 第一版整合測試 0 inputs／expected10 失敗，不計為成功；ID layout 修正後通過，詳見 E158。
- 有既有 td_rounds dead-code warnings；不是零警告建置。本批沒有完整 replay、filtered replica、終局、100 場、UE 或 LAN 驗收。

## 尚未完成／接續

- role_bot_inputs 目前是 opt-in 共用 API，測試已經用正式 driver 接入；production server／單機九名 Bot launcher 尚未接線。舊 headless CLI 仍使用 fixture，不能宣稱它已改為安全五位置 Bot。
- 由 Lua／compiled catalog 提供 roster、位置／路線與策略配置，取代呼叫端手動組裝；思考間隔應依 profile 配置，不把測試用每 tick 思考当效能基線。
- Carry 補刀、Support 保護與跟隨、Jungle 有限記憶／gank，及公開 HP 的目標存活篩選。當前已公開但死亡未退休的目標仍可能提交後被權威拒絕，不能宣稱零非法目標。
- 通用技能 target/type／range／cooldown 決策與三種完整英雄原型，避免 hero ID 或 slot 常數。
- 完成以上實作後才跑 100 場固定種子完整 headless／安全輸入與死局驗收，再做最後 Unreal 整體驗收。

OpenSpec 5.5 維持未勾選；整體 20/30 不變。
