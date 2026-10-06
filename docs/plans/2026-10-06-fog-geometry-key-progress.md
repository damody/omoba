# 通用霧幾何快取（2026-10-06）

## 問題與決策

舊快取只比較第一視野圓的整格origin與tree／polygon數量，忽略格內位置、其他viewer、半徑及遮擋形狀；無viewer時隱藏plane卻保留鍵，恢復同origin可能永不顯示。

- 新OmBuildFogGeometryKey以長度分隔、逐欄位float bits／u32建精確陣列鍵：全部vision circles、tree位置／半徑、polygon spans／points與WorldUnitsToCm。不讀struct padding、不依sequence、不使用可碰撞hash，也不掃描10000 entities。
- 所有幾何float必須finite、scale為正；第一viewer的10-unit格座標保留90-cell邊界裕量，拒絕無法用int32表示的mask origin。
- 共用OmFrameContract先以uint64驗證每個polygon span不超point_count，正式ProcessFrame任何更新前拒絕越界；不讓霧segment計算讀出陣列。
- SyncPresentationOverlay使用真正完整鍵，無viewer／非法幾何／runtime重設清鍵並隱藏plane。重建前隱藏舊plane，只有成功重新顯示後才保存鍵；缺資源失敗可在下一frame重試，不能提前commit舊mask。
- 新FogGeometryKey native矩陣涵蓋sequence不影響、格內移動／第二viewer／半徑／tree／polygon變形／scale影響、overflow與past-end span拒絕、NaN／超大origin拒絕及恢復。

## 局部確認

- UE限定模組9actions成功，exit0；Saved/Logs/fog-geometry-key.log。
- 固定Lua scoped runner語法與兩repo whitespace通過，新native矩陣僅編譯未執行；未PIE／stage／效能或完整驗收，不宣稱霧畫面已驗收。
- 無新編譯失敗；依舊視野幾何做呈現，不更動Rust權威／安全披露。ABI／wire／IPC與生成hash保持，整體21/31、完整6.1仍未完成。見E291。
