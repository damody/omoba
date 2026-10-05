# 資料驅動 Bot 對局配置與 headless 接線（2026-10-05）

## 計畫與已做決策

1. 新增共用 RoleBotMatchPlan／RoleBotPlayerPlan：schema1、compiled map ID、think_hz、玩家 ID／team／hero／role／named lane／bot 控制權。JSON 嚴格拒絕未知欄位。
2. compile 在建立 World 前原子驗證版本、思考頻率、map／active hero／lane、非零唯一玩家、兩隊各1..5人、同隊角色唯一、Jungle 的公開 camp 存在。依 player ID 排序，不因宣告順序改 roster。路線以 compiled lane.id 解析，不依角色硬編號。
3. think_hz 轉為 ceil(simulation Hz / think_hz) 的整數 tick 間隔，實際思考頻率不超過配置；60Hz／5Hz 得12 ticks，不改 gameplay tick rate。
4. Lua 配方 scripts/lua_data/moba_role_match.lua 目前為兩隊各五名 Bot。任何玩家 bot=false 即不產生其 Bot assignment，可表示一名真人＋九名 Bot；真人與 Bot 皆在同一 roster，不偽造第三隊。
5. scripts/export_moba_role_plan.lua 使用固定 Lua 5.4 與既有 JSON／path 模組匯出；run_moba_headless.lua 接 --role-plan-lua，產生每次獨立名稱的 JSON，避免平行執行覆寫固定配方。Rust 仍是 schema／catalog 驗證者，不把 Lua 匯出成功當作有效對局。
6. moba-headless --role-plan 走 role_bot_inputs→SimulationDriver 正式 PlayerInput，每步 Wave B 提供下一次 Bot 所用的 committed disclosure；舊 Push/Guard 保留。拒絕 role-plan 與 fixture map／defender 混用或重複指定 role-plan。
7. 新模式報告保存實際 plan／bot_mode／skill_coverage_checked=false。不施法的角色基礎策略不能冒充四技能驗收；完整模式仍必須終局唯一、combat facts 非零與 recorded-input replay 一致，不能以配置通過冒充對局成功。
8. --plan-only true 是無 World／無 DLL 的啟動前檢查，Lua wrapper 亦跳過 DLL 建置，報告明確標示 configuration only。尚未接 KCP server；必須連同 Bot 身分不可被 wire client 佔用、正式 acceptance／投影做完整接線，不能只塞 ECS input。

## 本功能確認

- core role_bot_plan 篩選測試：2/2 passed，涵蓋真人＋九 Bot、named lane、排序穩定、schema／catalog／role／team／ID／頻率拒絕。
- base_content role_bot_plan_nine_bots_leave_human_control_untouched_at_60hz：1/1 passed，正式 driver 60 ticks／5Hz Bot，九名 Bot 都提交輸入並位移，真人完全沒有 Bot 命令且位置不變。測試 warmup=0，只確認接線，不是正式預設 warmup 或完整遊戲驗收。
- cargo check --manifest-path omb/Cargo.toml -p omobab --bin moba-headless：exit0。
- 真實 Lua recipe→JSON→moba-headless plan-only 入口 exit0，10 players／10 bots，interval12。固定 Lua wrapper 新 --role-plan-lua／--plan-only 路徑也 exit0，不建立對局或重建 DLL。
- 既有 protoc fallback／td_rounds dead-code warnings 保留；不宣稱零警告。沒有 Unreal／KCP／100場／終局長測或新的 staged DLL。

## 指令

只檢查配置，不跑對局：

```text
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_headless.lua --role-plan-lua scripts\lua_data\moba_role_match.lua --plan-only true --report omb\target\moba-headless\role-plan-check.json
```

最後整合階段才使用同一入口移除 --plan-only true，建置 DLL 並跑完整角色 Bot 對局；本批沒有執行該長測。

## 剩餘工作

- KCP server／單機九 Bot launcher 的 controller ownership、正式 admission／安全 acceptance 接線。
- Carry／Support 專屬策略、公開 HP 的目標退休排除、通用技能與三種完整英雄原型。
- 100場固定種子安全目標／無死局、完整 replay／filtered、Unreal／LAN 最後驗收。

OpenSpec 5.5 仍未勾選；總進度20/30。OpenSpec 技能維持「局部實作與整項驗收分開記錄」，沒有以本批配置與短測冒充5.5完成。
