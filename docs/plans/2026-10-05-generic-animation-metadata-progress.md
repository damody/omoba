# 通用動畫生成與共用宣告驗證

## 計畫與決定

1. 移除生成器由 sniper 片段自行加入 Saika 狀態的分支，保留通用 walk 推斷；任何特殊狀態由 Lua 顯式宣告。
2. 共用模型提供 animation_metadata validator，Rust Lua 建置與 Unreal codegen 都在輸出前檢查同一契約。
3. 六種字串映射與 idle list 上限 64 項，拒絕空白名稱、錯誤型別；播放倍率須正值、有限且可表示為 f32，fallback_policy 須為已定義字串值。
4. 僅確認當前生成功能，不啟動完整驗收、不維護 omfx。

## 實作與局部結果

- `cargo test --manifest-path omoba-content-model/Cargo.toml generic_animation_metadata`：2 passed，六種映射、數值溢位／下溢、fallback 型別、六種內容類別與 tombstone。
- `cargo test --manifest-path omfue/codegen/Cargo.toml generic_animation_metadata`：2 passed，任意英雄不再推斷 Saika 狀態、非法映射生成前拒絕且不留下輸出。
- `cargo run --manifest-path omfue/codegen/Cargo.toml -- --content-root scripts/lua_data --out omfue/Plugins/OmRuntime/Source/OmGenerated --check`：15 files／17 Lua inputs 通過，content_hash 6460930eee2ba926 不變；現有顯式 Lua 配置保持，未重寫生成檔。
- Rust template-ids 建置入口已實際隨 codegen 測試重建，使用共用驗證；没有引入正式遊戲 runtime Lua。
- 移除遷移後未使用的舊 array validator；無編譯／斷言失敗，既有 template-ids dead_code warning 保持。
- 2.2b 整項仍未完成，既有 Blueprint typed 相容與完整回歸留最後；21/31 保持。驗證 fallback 宣告不等於所有 fallback 播放行為或完整畫面已驗收。
