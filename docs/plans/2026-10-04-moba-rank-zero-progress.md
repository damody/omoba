# MOBA rank 0 初次學習增量（2026-10-04）

## 決策與範圍

先保持已通過的 60Hz 工作流，補功能缺口，不重試 E121 已排除的截字假設。此為 OpenSpec 5.3 的增量，完整框架仍是 19/30，不勾選完整經濟或 UI。

英雄 Lua 可選：

```lua
moba_loadout = {
    ranks = { 0, 0, 0, 0 }, -- Q/W/E/R；0 表示未學習
    skill_points = 1,
},
```

未宣告時沿用四槽 rank 1、0 點。現有 shipped Lua 不改，既有四技能 smoke 的前提不變。Story 與 TD 的出生初始化不變。技能第一級可以要求英雄 level 6，此時 rank 0 出生合法，預先學到 rank 1 的配置會被拒絕。

Rust 與 Unreal 生成器都呼叫共用 typed validator；拒絕超過四槽、未知技能、超過 max_level、出生等級未滿以及 loadout 欄位拼錯。沒有技能的舊槽接受預設 rank 1 相容 sentinel 或 0，不建立不存在的技能。rank array 必須恰四個 u8，skill_points 為 u8。

template-ids 生成 MobaLoadoutConst 輕量常數，runtime Lua adapter 轉同一型別；不把 optional content-model dependency 或 serde 強制帶進預設 runtime，也不擴 abi_stable script ABI。loadout 變更進原始完整 Lua catalog hash，單邊 hot reload 與 runtime/compiled 不一致均拒絕，需走既有 Lua build 入口重建所有 peers。

權威 match setup 先驗完整 loadout 再写 Hero，完成全部 roster 檢查才建立實體。只有出生套用，死亡重生保留 ranks/SP。第一次學習沿用正式 UpgradeAbility，按 Lua 第一級門檻驗證且只扣一點；不重設冷卻。

內部 ScriptEvent::SkillCast 額外檢查 MobaMatch 中 hero rank，不讓既有腳本 `max(1)` fallback 將未學習技能當 rank 1 執行。不能用 GameMode::Moba 判斷 opt-in，因為它也是舊 Story 的預設。filtered 正式 cast handler 原本就檢查 learned rank；rank fact 和 HUD 仍使用既有安全投影，不新增角色 graph/C++ 或 renderer 模擬。

## 驗證

- shared model：9 項通過，包含預設、rank 0、晚第一級解鎖、非法陣列／未知欄位。
- template-ids：35 unit + 23 generated + 8 hero + 1 catalog = 67 項通過；runtime adapter 保留四個零與一點，hot reload 與 compiled agreement 拒絕錯配。
- core：334 項通過，包括零 rank 的絕對結算／首次學習不重扣。
- base_content：97 項通過，包含權威 rank 0 → 正式學習 → 80 傷害 → 死亡重生保留、非法出生配置原子拒絕、雙隊首次學習 parity，以及既有完整 60Hz／120Hz 生命周期回歸。
- 雙隊 parity 使用正式 PlayerInput／CanonicalAcceptedInput／ScriptRegistry／filtered ECS，共 40 ticks × 2 隊逐步完整 canonical hash 一致、無 component repair、對手不取得 private upgrade input。初始 rank/SP 是明確 pre-bootstrap fixture，不冒充真實網路出生配置驗收。
- runtime：61 通過／7 opt-in ignored；新增 owner HUD 保留 rank 0、學到 1 及 SP，不預測 rank、不改 cooldown。
- Unreal codegen：25 項通過，包括真實 Lua rank-zero native-only 英雄生成、level-6 技能出生非法 rank 拒絕；shipped --check 11 files／15 Lua inputs，content_hash `ac6ff592a15c8b5e` 不變。
- bridge：52 通過／1 opt-in ignored，沒有修改 C ABI 9。
- 標準 Lua `build_ue_moba.lua --build-only` 兩輪 exit 0；最終 OmGameEditor Result: Succeeded／Target up to date，bridge stage SHA-256 `00fb8bf3d1c102ed10bb8a92b26062571dbe22436393471edb80474550e3dbcd` 核對通過。沒有啟動 Editor 或 game processes，不能把本輪 build-only 稱為新 MCP／PIE 驗收。
- OpenSpec strict validation、scoped git diff --check 通過；只有既有 LF→CRLF 提示與工具／插件 warnings，沒有自動 cleanup、commit 或 push。

## 未完成與下一個驗收

尚未改 shipped hero 的初始 rank，尚無真實 KCP／雙 Unreal rank 0 出生→第一次學習 capture，較晚第一級的 runtime 實戰仍需專用內容。现有原生 HUD 可以呈現 L0，但未新增 unlearned 像素／操作驗收；之前名稱截字仍未修好。這些不是本輪 headless fixture 的替代結論。

下一段應以單一 opt-in Lua hero loadout 做有界 60Hz 端到端驗收，精確對照原始 authority ranks/SP、自己的 IPC/HUD 與雙隊 hash，再補 rank0 無點／等級不足→學習成功的 UI 提示。不要把預設四招已學 smoke 或 CPU GetText 當成新的 rank0／GPU 證據。

錯誤與防止重犯的修正位於 `unreal-moba-error-register.md` E122。
