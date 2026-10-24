# 召喚戰鬥錯誤紀錄

- C01：原召喚查詢只過濾距離與牆，未接 committed disclosure，且沒有 VisionSource；不能把上輪生成通過當作草叢索敵正確。
- C02：本輪第一次搜尋誤猜 hero_attack_tick.rs；實際為 hero_tick.rs。後續用 rg --files 查實際檔名，不重複猜路徑。
- C03：OpenSpec CLI 前輪已確認不可用，依既有技能維護 proposal/design/tasks；使用者已授權自主決策，不重複確認。
- C04：patch 的同檔 hunks 必須按原碼順序，且同一檔不可列兩個 Update。工具已原子拒絕錯序／重複操作；重排後套用，不重複猜 anchor。
- C05：RustV1 兩個正式召喚案例拒絕開火：fixture只有SimulationDriver gameplay，沒跑正式外層Wave B視野提交。嚴格committed disclosure拒絕是正確行為；改fixture呼叫真實run_committed_visibility_wave_b，不注入可見名單、不改authority fail-closed。V1失敗原始log保留，V2等V1 worker與watcher退休後才重連結。
- C06：WorldV1 33成功，4筆既有preview-world DestroyActor context警告保留。這是本機公開場景fixture回歸，不冒稱真對局召喚／草叢畫面或權威ACK。
