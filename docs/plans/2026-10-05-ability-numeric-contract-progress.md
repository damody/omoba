# 共用技能數值契約（2026-10-05）

## 計畫與決策

1. 延續 5.5，先核對資源與隊伍能力。權威 current_mana 仍讀最大 Mana，spend_mana／restore_mana 無作用；faction_of 仍回 RNone，GameWorld 方法表已有容量限制。本批不假裝 Mana 可用，也不把「不是敵人」當友軍；真正資源／盟友契約仍需後續實作。
2. 已確認的共同缺口是技能等級資料只驗 rank 數與解鎖門檻，沒有完整檢查 cooldown／mana_cost／cast_time／range；效果量原先只要求有限非負，可能接受極大值或被 Q10 量化成零的正值。改在生成／載入邊界拒絕，不靠 Unreal UI 防呆、不在執行器偷偷 clamp。
3. 共用模型對所有非 tombstone 技能、每個 rank 的上述四欄要求零或有限 [1/1024,1000000]。這是內容安全範圍，不是英雄平衡或全部運算的溢位證明；零冷卻／零成本仍合法，缺省欄位保持零。
4. 宣告式效果使用到的 amount extras 套用同一範圍；無關 extras 不強制非負，保留特殊 Rust handler 的資料語意。既有 targeted cast range／area radius 仍受較小 [1/1024,10000] 限制，不放寬前批規則。
5. 固定 Lua FFI 生成器對所有 active catalog 等級資料驗證，包含特殊 Rust handler，不只檢查被通用效果引用的技能；同時保留 shared schema 的 max_level 缺省4與 required_hero_level 缺省1、單調1..25規則。tombstone 不需要完整執行資料。
6. Rust 共用 schema 是 f32、Lua 數字是 double。Lua 先用標準庫 string.pack／unpack 轉成相同 f32 再驗 scalar／cast range／radius，避免邊界只有其中一個生成器拒絕。規則作用於 schema 解析後的數值，不宣稱保留已被 f32 underflow 丟失的原始 double 精度。
7. shared model 已由 template IDs build、runtime content 與 Unreal codegen 共用。本批無新 ID／Lua gameplay 資料／ABI／英雄專屬 C++ 或 Blueprint；生成內容雜湊不變。

## 當前功能確認

- `cargo test --manifest-path omoba-content-model/Cargo.toml --lib authored_numeric_envelope -- --nocapture`：2/2。涵蓋每欄第二 rank、負值、低於Q10、過大值、NaN／Infinity、零與精確上下界、f32邊界捨入、tombstone、執行效果與無關 extras 分離。
- 固定 Lua `scripts/test_ability_numeric_contract.lua`：34/34。包含合法缺省、上下界、f32邊界、四欄非法值、非法效果量、特殊handler與 required_hero_level 型別／門檻；fixture 使用獨立 target 目錄，不改正式內容。
- 固定 Lua `scripts/test_hero_registry_includes.lua`：原 include 5/5通過；正式 `gen_hero_registry.lua scripts/lua_data ids` 成功輸出24項註冊。
- `cargo test --manifest-path scripts/Cargo.toml -p base_content --lib cast_preflight -- --nocapture`：2/2，含正式60Hz距離／治療／HP-CD確認；本批共用validator已參與 template IDs／FFI／core編譯。不是完整對局驗收。
- Unreal codegen `--check`：11檔／17 Lua inputs，content_hash=`5638c23df5bba3c7`，生成檔無變更。

## 剩餘工作

Mana 權威狀態與扣除／恢復／安全投影、盟友技能契約、完整三原型與100場、Unreal最新DLL stage／整合、LAN與效能留待後續。OpenSpec維持20/30，5.5不勾選；本批不跑 Editor／PIE或完整驗收，無提交／推送／清理。防錯記錄見 E170。
