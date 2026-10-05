# 開局前真人選角入口

## 計畫與決定

1. 補既有互動啟動的選角資料缺口，先建共用入口，不再只改 Bot。
2. `moba_role_launch.select_heroes` 複製 Lua 配方，僅修改既有真人玩家 hero；保留隊伍／位置／Bot 策略。
3. `--hero PLAYER_ID=HERO` 支援不同玩家重複指定，重複玩家／非法 ID／Bot／不存在玩家直接拒絕。英雄合法性只使用 Rust 正式目錄，不維護第二份白名單。
4. 選擇寫入生成設定的權威配方，Rust preflight 成功後才生成 launch-plan，並保存 human_heroes。來源 Lua／game.toml 不修改。

## 使用

```text
tools/lua/lua.exe scripts/run_moba_role_ue.lua --prepare-only --hero 1=training_ranger
```

移除 `--prepare-only` 即沿既有建置／互動啟動流程執行。多真人需先在 server-owned recipe 宣告 bot=false，再各自指定 --hero。

## 當前確認

- `tools/lua/lua.exe scripts/test_moba_role_launch.lua`：7/7 通過。包含配方不變、兩真人獨立選擇、所有權拒絕、Rust 真正接納 training_ranger 覆寫原 training_luminary、未知英雄拒絕，以及既有 readiness／PID 清理。
- 原 source game.toml 位元組未改；CLI help 已顯示新參數。`git -c core.safecrlf=false -c core.whitespace=cr-at-eol diff --check` 通過。
- 未啟動 server、client runtime 或 Unreal；完整驗收留到最後。
- 尚非 UE 選角 UI、網路大廳、英雄鎖定協定或完整 6.2。整體維持 20/30。
- 操作錯誤與預防紀錄 E208；共享引擎 E177 不在本批修改。
