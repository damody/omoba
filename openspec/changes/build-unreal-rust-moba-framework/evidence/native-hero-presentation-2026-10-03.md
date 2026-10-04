# 通用原生英雄呈現驗收

## 實作範圍

Lua render／ue.native_visual → Rust codegen → generated native C++ constructor → AOmHeroActor 的通用 mesh／材質／動畫槽。

角色作者不需撰寫角色專屬 C++ 或 Blueprint graph。生成的 CDO 僅儲存 soft 引用；BeginPlay 載入。既有 Blueprint 的 skeletal 呈現保留，缺少模型維持 fallback。

## 可重現指令與結果

| 指令 | 結果 |
| --- | --- |
| `tools\lua\lua.exe scripts\build_ue_moba.lua --full --ue-root D:\UE5.8` | exit 0，DLL staging、OmGameEditor 編譯與 MCP ready；本次 Editor PID 57700 |
| `cargo test --manifest-path omfue/codegen/Cargo.toml` | 22/22，完整 object path、clip、非法引用、原生軸向與 Lua override |
| `tools\lua\lua.exe scripts\ue_native_visual_smoke.lua` | 同一 Editor 連續兩輪，每輪 4/4 passed、零 error，非 skipped／not-run |
| `tools\lua\lua.exe scripts\ue_pie_smoke.lua` | 9 步序列、兩個屬性斷言 PASS，native_mesh_rendered=true，截圖後自行停止 PIE |
| `tools\lua\lua.exe scripts\tests\ue_asset_recipe_editor_test.lua` | 10 verified jobs，22 保存套件 SHA-256 不變 |
| `tools\lua\lua.exe scripts\tests\ue_asset_recipe_test.lua` | planner tests passed |
| `openspec validate build-unreal-rust-moba-framework --strict` | valid |
| 主 repo 與 omfue 的 `git diff --check` | 通過；有既有 LF→CRLF 提示，不是 whitespace error |

## 畫面／診斷證據

- `omfue/Saved/McpAutomation/NativeVisual/report.json`：完整 discovery、兩輪 verdict 與 individual test results。
- `omfue/Saved/McpAutomation/pie-smoke-report.json`：生成位置、Lua scale、玩家實際視點、render 回傳與 PIE 停止狀態。
- `omfue/Saved/McpAutomation/pie-native-hero.png`：native 測試英雄位於既有英雄旁，不與舊 fallback 重疊；可見模型、材質與影子。
- `omfue/Saved/McpAutomation/AssetRecipe/report.json` 與 ledger：配方保存與 owned 套件驗證。

Saved 屬執行產物，不提交 binary／log／cache。重跑以上工具可重新產生證據。

## 防錯與邊界

完整建置存在預編譯插件 dependency 宣告與 Rust dead-code 警告，不宣稱整體零 warning。測試曾因固定 transient Blueprint 名稱撞名崩潰，修正 MakeUniqueObjectName 後兩轮重跑通過；原因與防重犯規則在 `docs/plans/unreal-moba-error-register.md` E009–E019。

此驗收不涵蓋技能 cue、完整 fog／地圖、LAN 雙玩家、完整 MOBA 對局規則或 UI。OpenSpec 仍 14/30；6.1 不因 native 動畫基礎通過就勾選完成。
