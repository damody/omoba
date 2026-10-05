# 共用 Rust 選角、鎖定與可選英雄目錄

## 本批計畫與決定

1. 6.2 選角規則放進共用 Rust kernel，不放 Unreal、Lua 執行期或新角色程式。
2. HeroSelectionSession 開始前使用正式 RoleBotMatchPlan validator；固定配方的 player／team／role／lane／Bot／策略不由請求更動。
3. 真人可選同一 compiled active catalog 的英雄並鎖定；Bot 起始已鎖定。沒有另加禁重複英雄／禁跨隊鏡像規則，因目前配方本來容許。
4. 所有操作使用全局 revision，過期請求拒絕並應重新讀 snapshot；拒絕不更動，u64 溢位拒絕。鎖定後不能換角／解鎖；所有真人鎖定才 finalize，之後不能再次提交或 finalize。
5. 請求不含玩家 ID，host 必須從已接納連線取得身分；kernel 不承擔網路認證或直接開 socket。
6. 生成 HERO_CATALOG_IDS 保留穩定 numeric ID 並排除 tombstone；Rust 導出同一 compiled hero／metadata／技能目錄，供後續共用 UI 使用。
7. moba-config --lock-plan FILE.json 在本機主機準備模式鎖定所有真人，輸出最終配方／snapshot／hero_catalog；正式 Lua launcher 接入，保存 candidate、lock、catalog 與 final JSON，再沿原設定 preflight，沒有啟動第二個 World 或 Lua VM。

## 當前確認

- Rust 三項狀態機測試首輪 3/3 成功：選角鎖定→finalize、拒絕原子性／控制身分／溢位、全 Bot／配方順序穩定。
- 初次 Lua 流程被防覆寫拒絕，已分離 candidate／final 檔；修正後 7/7 成功。
- 最終 Rust 4/4 成功，新增 compiled catalog 每個可選項皆通過正式准入、numeric ID 遞增／技能引用合法；Lua 7/7 成功，檢查十席全部鎖定、同 hash catalog、JSON 重新選角、來源不改。沒有啟動 server／client runtime／Unreal 對局程序（生命周期部分使用 mock）。

## 尚未完成

- 不是 Unreal 選角畫面、真人點擊、LAN 大廳、跨 client 同意、Ban／Pick 或斷線逾時計畫。
- --lock-plan 只適用可信本機主機準備；遠端入口必須逐真人呼叫同一 kernel，不能把 CLI 自動鎖定當玩家同意。
- 尚未部署新 DLL／重建 OmGame／PIE；E177 保持。只做當前功能確認，完整驗收留最後。
- 6.2 全項不勾選，整體仍 21/31；E210 記錄錯誤與預防。
