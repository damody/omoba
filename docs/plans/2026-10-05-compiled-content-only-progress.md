# 正式 MOBA 編譯內容與 Lua 邊界修正

## 修正後的實作計畫

1. Lua 僅負責作者資料、建置生成與開局工具；生成 Rust 常數／handler／FFI 與 Unreal C++。
2. server、client runtime、base_content DLL、Unreal bridge 採 compiled-content-only；與 runtime-lua-content 編譯互斥。
3. 生成器保留 authoring 預設給 CLI，但 bridge 正常依賴關閉 authoring，只使用 metadata；build.rs 的 host 生成器把完整 catalog 嵌入 bridge。遊戲不再依 content_root 求值 Lua。
4. 正式開局工具將 Lua recipe 轉成普通 TOML／JSON；選角保留 server-owned 位置／隊伍，遊戲只使用一般配方資料。環境明確 OMB_LUA_CONTENT=0／HOT_RELOAD=0，避免繼承開關；雙 UE、role UE、headless 與既有 UE wrapper 同步。
5. 繼續 MOBA 6.2：目前已有開局前真人選角資料入口，下一段為共用選角畫面與鎖定流程；其後收斂動畫／cue／重連、三路 Bot 與 LAN。共享 UE 引擎 E177 未解除前，繼續不依賴引擎建置的部分，不修改使用者引擎。
6. 最後才做完整生成／建置／部署、無 Lua 作者檔執行、選角到結算、100 場與效能驗收；歷史驗收不代表本次產物已部署。

## 當前功能確認

- Lua 選角／設定局部測試 7/7 成功；真實 Rust preflight 接納 training_ranger，未知英雄拒絕，來源設定不改。
- 四個元件正常依賴 Lua VM gate 4/4 成功：`tools/lua/lua.exe scripts/test_compiled_content_contract.lua`。build dependencies 不算遊戲依賴；作者期 mlua 仍允許。
- bridge check／base_content compiled-content-only check／server omobab check 成功。bridge check 同時編譯 client runtime。
- 新 bridge compiled_catalog 測試 1/1 成功：不存在作者目錄、flag 要求 reload 仍 ReloadDisabled，內嵌 catalog 有 unit／ability，租約釋放與 destroy 正常。
- 互斥 feature gate 確實編譯失敗並指出 compiled-content-only forbids runtime-lua-content；檢查流程確認是預期拒絕。
- 既有正式 `jungle_farm_focus_60hz_area_cast_and_attack_share_disclosed_neutral_target` 在 base_content compiled-content-only 下 1/1 成功，包含正常生成 handler 的範圍傷害／冷卻與普攻同一披露目標；非完整對局驗收。
- 選角進一步保存經 Rust 驗證的 `match-plan.json`，`--recipe FILE.json` 可重新選 training_vanguard，原 JSON 中 training_ranger 不改；最後 Lua 局部 7/7 成功。這是 6.2 的普通資料入口，不是 UI 完成。
- root／omfue／omb whitespace diff check 成功；沒有執行完整 Unreal 驗收或提交。

## 限制與決策

- DEV runtime-lua-content 原能力保留為明確非正式建置（bridge 必須 no-default-features），沒有删除或假裝重新驗證舊 DEV 功能。
- Lua 是 declarative source，不是任意 Lua 程式全文轉譯器；既有通用 effect 生成 Rust、特殊行為寫 Rust，Unreal 共用／生成 C++。
- 尚未重新 stage DLL／完整 OmGame build／PIE／無作者檔部署測試；E177 保持。
- E209 記錄根因與本批實際錯誤。只新增本次架構修正任務，不勾選其他未完成 MOBA 功能。
