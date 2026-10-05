# 原生動畫 fallback policy 接軌

## 本輪計畫與實作

1. 把 Lua ue.animation.fallback_policy 產生為 EOmNativeAnimationFallbackPolicy，任意英雄使用同一 constructor 模板。
2. 共用 selector 保留 UseGenericState 的基本 attack／idle fallback；UseReferencePose／HideVisual 在基本片段缺失時不播放別的 action。
3. 初始化與正式動畫更新共用播放路徑。找不到適合目前狀態的片段時清除舊 asset、slot、action、時間與循環，避免留下舊攻擊姿勢。
4. UseReferencePose 與通用素材耗盡清為參考姿勢，HideVisual 隱藏原生 skeletal mesh；下個有效片段恢復顯示。沒有改 actor 的視野控制、marker 或 gameplay。
5. fallback 首次才套用；失敗素材沿用快取，避免每個快照重載。僅當前功能確認，整體驗收留最後。

## 局部結果

- `cargo test --manifest-path omfue/codegen/Cargo.toml native_animation_fallback_policy`：1 passed，任意英雄的 default／三種顯式 policy 生成及非法值拒絕。
- 正式 codegen 與 `--check`：15 files／17 Lua inputs 通過；generator_version 4，content_hash 79bf52bd0ba256e2。
- catalog_data_hash 2027e0ada2f76866／catalog_identity_hash ff3ef5e2957aa89f 保持。Lua 內容未變，差異是生成／原生呈現契約。
- scoped OmRuntime＋OmGenerated＋OmEditor：16 actions，23.87 秒，Succeeded；日誌 `omfue/Saved/Logs/native-animation-fallback-policy-modules-20261005.log`。
- 原生 automation 增加 explicit policy 缺 critical／有效 attack 選擇斷言，僅已編譯，未執行。未解除 E224 或改 BuildId，不宣稱 Editor／真實素材 reference-pose 畫面已驗收。
- IPC 4／C ABI 13／selective wire 4 維持；未部署、stage binary、啟動完整對局或維護 omfx。21/31、6.1 整項保持未完成。
