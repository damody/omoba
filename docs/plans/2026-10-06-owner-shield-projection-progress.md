# 護盾安全投影與原生 HUD

## 計畫與決定

1. 使用既有 owner economy 的持續狀態管線，不從物品 metadata 猜盾量、不公開 Buff payload、來源或動態名稱。
2. 權威僅從存活英雄的 BuffStore 取得剩餘吸收量；死亡或沒有英雄時發布零。新增 Q10 i64 `shield_remaining_raw`，合法範圍 0–1024000000。
3. OwnerEconomyState 定長由 86 改為 94 bytes，decode 拒絕舊長度、負量、過大值。既有 team audience 保持；client 與 bridge 仍只選 configured owner。
4. IPC OwnerEconomyPresentation schema 2，selective wire 6、IPC 5、C ABI 15；不能混用舊元件，最後統一建置部署，不手改 BuildId 或放寬握手。
5. 共用 Unreal economy payload／原生文字呈現剩餘盾量；完整快照取代狀態、缺 owner 與停止清除、control-only 保留。數值為唯讀呈現，不回寫 gameplay。

## 當前功能確認

- 正式生成商品 `moba_guard_charm` 的 60Hz 測試通過：正式購買與使用、100→60→0、到期、死亡與另一 owner 零值；不替換 ItemRegistry。
- bridge configured owner／非法 raw 邊界／舊 schema 拒絕測試通過。
- Unreal OmRuntime／OmGenerated／OmEditor 限定編譯成功，14 actions，11.68 秒。日誌：`omfue/Saved/Logs/owner-shield-modules-20261006.log`。
- core 定長 codec 與 client owner 投影各一項局部測試通過；加上正式60Hz與bridge，本輪四項指定Rust測試全部通過，沒有跑全套。

重現指令：

```text
cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only generated_shield_60hz_owner_projection -- --nocapture
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only owner_codec_is_bounded -- --nocapture
cargo test --manifest-path omoba-client-runtime/Cargo.toml --features compiled-content-only dead_owner_economy -- --nocapture
cargo test --manifest-path omfue/bridge/Cargo.toml economy_and_receipts_require_configured_owner -- --nocapture
```

## 尚未驗收

沒有 PIE、OS 按鍵或畫面驗收；release server／client／bridge／script DLL 尚未統一重建與部署。完整 4.1／6.2 等大項仍不勾選，21/31 保持。內容雜湊維持上一輪 data `2df5b6b1e02d95d7`、presentation `3634f807282b0515`；本輪變更的是傳輸／ABI 契約。

## 問題與避免重犯

E266：Windows `rg` 路徑不能使用未展開的 `transport*` 字面 glob；改用已知 `omoba-core/src` 配合檔案 glob。大段文件與合併輸出截斷不當作完整審閱。UBT 等待既有 Build.bat 自然完成，沒有終止未知程序。既有 td_rounds／plugin 警告不冒充本輪新失敗。

文件 patch 曾在 EOF 新增多餘空行，diff --check 發現後移除；後續文件更新使用明確上下文，避免空 context 插入位置不明。
