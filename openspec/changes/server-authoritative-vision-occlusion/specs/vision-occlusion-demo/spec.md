## ADDED Requirements

### Requirement: 雙玩家 demo 提供可重現的遮蔽場景

`FOG_2TEAM_DEMO` SHALL 包含 10×10 的 100 個普通單位、另計兩位不同隊英雄、至少 64 棵不同半徑樹木、至少一個凸多邊形及兩個凹多邊形。兩位英雄起點 SHALL 顯示不同的 team disclosure，且每隊附近 SHALL 同時存在可見、樹後遮擋與地形後遮擋的觀察目標。

#### Scenario: 啟動固定 demo
- **WHEN** 啟動一個 server 與兩個獨立 omfx process，分別連線為 team 1 與 team 2
- **THEN** 每個前端都看得到自己的可控制英雄
- **AND** 兩個前端看到的普通單位集合不同
- **AND** 場景可辨識樹木、多邊形與己方視野邊界

### Requirement: 玩家可移動英雄驗證 Reveal 與 Forget

每個 omfx process SHALL 將右鍵地圖位置送成自身英雄的合法 lockstep move input。英雄繞過遮蔽物取得無遮擋 LOS 時，server SHALL 以目前 authoritative step 揭露目標；英雄移回陰影後，server SHALL 送出 Forget，前端 SHALL 立即移除目標。

#### Scenario: 繞過樹木揭露即時單位
- **WHEN** 玩家右鍵移動己方英雄，使樹後目標進入無遮擋 LOS
- **THEN** 該玩家收到目標目前 authoritative step 的 Reveal/baseline
- **AND** 另一隊不因這次移動取得同一資訊

#### Scenario: 回到陰影後不留殘影
- **WHEN** 已揭露目標再次落入該隊所有視野來源的遮蔽陰影
- **THEN** server 對該隊送出 Forget
- **AND** omfx 不再顯示該單位、名稱、血條、選取或 remembered identity

### Requirement: 最終驗收保存雙隊證據

所有程式實作完成後 SHALL 集中啟動完整測試與雙 process 人工驗收，保存兩隊在遮蔽前、Reveal 後及再次 Forget 後的畫面和摘要。測試不得以減少單位、遮蔽物或 tick rate 規避失敗。

#### Scenario: 完成最終人工驗收
- **WHEN** 自動測試、determinism 與效能 gate 全部通過後執行 demo 驗收
- **THEN** evidence 包含 team 1 與 team 2 的 before/reveal/forget 畫面
- **AND** 摘要記錄兩隊 visible count、remembered count、replica 驗算與操作步驟
