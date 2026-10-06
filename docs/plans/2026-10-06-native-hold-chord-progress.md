# 原生守住操作的明確佇列語意

## 計畫與決定

1. 檢查發現 H 已有初版，不再重複新增相同功能。修正兩個 chord 共用回呼後讀實體 Shift 的時序依賴。
2. H→HoldNow(false)、Shift+H→QueueHold(true)，共用 HoldOwnedHero(bool)。不依賴 IsInputKeyDown，也不查游標或敵方位置。
3. 共用 BuildOwnedHoldInput 重設整份事件，非法 owner 明確設 PlayerId=0，合法事件為 configured owner／Hotkey／NoTarget HoldPosition，保留 queued。
4. 沿用 HUD／選角攔截；必須有已啟動、診斷 Connected 的 bridge，直接讀 GetConfiguredLocalPlayerId，不使用含歷史 player-one fallback 的 UI helper。
5. 經既有 SubmitGameplayInputEvent→Rust renderer intent→權威 HoldPosition。queued 只是正式命令佇列語意；提交接受不代表已執行守住。
6. non-shipping automation dispatcher 只執行精確、唯一、已綁定的 chord delegate，確認一次 callback 與提交結果；不是 OS 鍵盤注入。

## 當前確認

- 限定 OmRuntime／OmGenerated／OmEditor 編譯成功，10 actions、11.77秒；日誌 `omfue/Saved/Logs/native-hold-chord-modules-20261006.log`。
- NativeAbilityUpgradeBinding 增加兩個 H chord 唯一性／HUD拒絕／queued明確參數／owner7與非法owner清零斷言。只已編譯，沒有執行 Editor automation。
- 既有 Rust renderer codec 的 HoldPosition false／true roundtrip 指定測試通過，1項，沒有跑完整Rust測試。
- 沒有修改 Rust gameplay、ABI／wire／IPC、生成內容或 Blueprint graph；只維護 omfue。

重現局部Rust確認：

```text
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only shared_renderer_codec_preserves_point_target_and_queue_semantics -- --nocapture
```

## 問題與預防 E268

- 既有 H 綁定不是缺失，而是 queued 依賴實體按鍵取樣；後續先檢查已存在的正式入口，再決定補功能或修契約。
- 不將編譯的 native assertions 當作已執行，PIE／真人按鍵／真正網路權威佇列操作留最後整合。
- 既有 plugin dependency warnings 保留，沒有新編譯失敗或為此放寬 NoEngineChanges。

整體21/31、完整6.2仍未完成。下一步持續補齊剩餘框架功能，最後才做統一部署與完整驗收。
