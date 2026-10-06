# 遠端選角的最終配方分發

2026-10-06 後續更新：正式 compiled-content-only release moba-config 已重新建置成功，production TCP worker 的真 socket 多席位配方分發／錯 token／host take 後 read 與 re-admission 已通過局部確認。詳見 [TCP 整合紀錄](2026-10-06-selection-tcp-integration-progress.md)。下方「尚未新建 host exe／沒有真 network」保留為 E335 當時的歷史邊界，不代表本次尚未建置；完整 executable host/proxy／UE／雙實機 LAN 與全遊戲驗收仍未完成。

## 實際缺口與決定

遠端已透過私人邀請連到Rust的唯一SelectionRoom，但finalize只有成功提交者回覆帶plan；其他玩家的read／stale回覆只有finalized snapshot，因此選角完成後還需要額外手動傳主機的match-plan.json。

現在由HeroSelectionSession提供只讀finalized plan snapshot，SelectionService在正式finalize之後為所有已綁定席位的回覆附同一份immutable plan。尚未finalized，即使所有人locked，也不得提早取得plan。這不是另一份開局權限：SelectionRoom.take_finalized_plan仍為host-only一次領取，read／EOF／rebind或並發失敗者不重新填入host handoff、不改revision。既有plan欄位使用Some，無新schema欄位、wire版本或ABI變更；不是新的帳號認證，原私人邀請准入與受信任LAN限制保持。

遠端Lua取得terminal receipt後核對protocol／shared-room／owner／compiled hash、所有locked seats與plan roster一致、自己的真人席位存在；以所選profile的原生moba-config --lock-plan再驗整份配方／編譯catalog。native結果必須exit0、同hash且序列化plan完全相同，不能悄悄改規則或normalize成另一份配方。只有驗證及original renderer退役都成功，才發布本機match-plan.json／result.json。沒有計算或拼接follower配方，也不開gameplay或停止remote host。

## 使用方式

主機需重新建置moba-config，正常run_moba_role_ue.lua建置路徑會處理；no-build模式則明確先建指定profile，例如：

```text
cargo build --manifest-path omb/Cargo.toml -p omobab --bin moba-config --features compiled-content-only --release
```

遠端仍使用run_moba_selection_join.lua與自己的私人invite。成功時OUTPUT/match-plan.json就是主機房間發布並驗證的正式配方，不再額外傳配方；使用既有明確步驟進入遊戲：

```text
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_role_ue.lua --connect <主機IPv4> --recipe <遠端選角OUTPUT>/match-plan.json --local-player <自己ID> --output <新的遊戲輸出目錄>
```

舊主機若不分發plan，入口明確拒絕並要求重建，不從seats補造配方或自動切profile；仍不宣稱遠端自動遊戲啟動／服務發現。

## 局部確認與邊界

Rust hero_selection 11項通過，涵蓋finalized前不分發、host一次take後follower／rebind／EOF與stale read取得相同plan、兩個並發finalize只有一個成功mutation／host handoff但兩份相同只讀配方、poison與原拒絕回歸。Lua join17／shared25通過；join新增一真人九Bot canonical recipe的真實debug moba-config生成與再次validation／輸出完全相同，renderer仍注入。沒有重跑整個Rust／Unreal套件、game／場次模擬／LAN或完整驗收，沒有編譯新的主機moba-config.exe／UE。正式core test編譯成功，本批53個不同測試，不重複計算第一次Rust3與join16重跑。

正式建置仍需最後一致更新。完整六项保持待驗收25/31。本輪錯誤及Grok結果保存E335。Grok只讀稽核job run-muwich4j-mnzbqf／thread d652d9ec-5663-4b93-a258-cb4c29ae3d11，5分29秒仍停在讀取交接流程，沒有完整稽核結果，primary取消並確認follower terminal／tracked三PIDnull／OS原47028及68152不存在；stop exit1/taskkill128不單獨當清理證明。沒有接受Grok補丁或把進度文字當功能結論，成本／API時間／根因未知；primary完成上述實作及審查，不改Grok/MCP/auth設定。

另確認：staged-copy一致不是建置來源證明。沒有為遊戲啟動添加作者Lua檔／mtime需求，也沒有建立暫時source freshness fallback；正式執行仍只用Rust／C++編譯內容。
