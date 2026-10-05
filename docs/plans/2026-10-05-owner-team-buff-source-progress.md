# 隊友英雄持續 Buff 安全來源（2026-10-05）

## 計畫與決定

1. Lua 僅作生成輸入；正式權威 Rust BuffStore 產生 typed 視覺狀態，接既有 Unreal active_buffs 與 OnBuffState，不新增 Lua runtime、角色專屬 C++ 或 Blueprint graph。
2. 只對同隊、有非零玩家 owner、當前 Disclosed 的英雄披露。只傳 catalog Buff ID 與剩餘時間，不傳 payload、來源、動態 Mana 名稱或隱藏敵方資訊。
3. BVS1：6-byte header（magic＋u16 count），每項 u16 ID＋i64 Fixed64 raw；至多 64 個、嚴格排序／已註冊 ID、正時間或 -1 無限。完整空集合可清除；解碼嚴格拒絕截斷、尾資料、重複、未知 ID 與非法時間。
4. Team projector 每 tick 從當前 committed components 發布 OwnerBuffVisual；filtered replica 必須已有合法 baseline 才接受更新。Reveal／rebase 的 component 契約與安全 allowlist 一起接通。
5. Bridge 只轉為無 payload 的 BuffSnapshot；presentation-only 清除由 baseline 推導的歷史事件，不重播 Added／toggle。現有合法 typed Ability cues 不受影響。
6. Selective 線路版本提升至 3；transport 提供唯一常數，framing 重新匯出，server／projector／replica／fixture 共用。保留 SelectiveV2 等模式名稱，不保留舊 wire 2 的相容假象。

## 局部確認與剩餘範圍

- core 新增／更新 6 個 buff_visual_state 測試通過；包含來源安全、每 tick 倒數與清空、缺 baseline 拒絕及協定版本拒絕。
- 最後整合版本 bridge 2/2 通過；server 新 wire gate 1/1 通過，既有 command capability 局部檢查也通過；client config 2/2 通過。共 11 個不同的當前局部測試，非完整驗收。
- 發現 client config 與 Lua launcher 也硬編碼 2：client default／驗證改共用 Rust 常數，七個共用啟動工具移除固定 --protocol-version，由 runtime 自行選擇編譯版本。固定 Lua 5.4 loadfile 語法確認七檔通過，未執行工作流或啟動舊 DLL。
- OpenSpec strict 與主 repo／omfue whitespace 檢查通過。
- 不執行完整對局／全套驗收，不重啟 E224 BuildId 基線失配的 Editor。C ABI 13 與原生來源此批不變，不 stage DLL。
- 不維護 omfx。OpenSpec 21/31 保持；敵方可見 Buff 的額外披露政策、UI 完整列表退役、宣告式美術與完整呈現驗收仍待完成。
- 防錯與修正記錄：unreal-moba-error-register.md E230。
