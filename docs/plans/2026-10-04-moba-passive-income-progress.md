# MOBA 被動收入與合法購買前提（2026-10-04）

## 問題與決定

正式單路對局從 Gold 0 開始，之前沒有收入。商店 kernel 的注入餘額 fixture 不能證明正式遊戲可購買，因此先補收入；不以 debug 金錢指令開啟商店。

- `scripts/lua_data/templates/moba_economy.lua` 是規則來源，預設每 active 秒 2 Gold、出生仍是 0。這是原型平衡設定，之後可改 Lua 並重建，不是最終競技平衡。
- `omoba-template-ids` 校驗整數／上限 10000、生成共享 Rust 常數。全模板資料雜湊包含收入，runtime hot reload 拒絕單邊變更。
- MobaMatch 使用 Fixed64 active 時間，每滿一秒給固定整數；保留秒內餘數，暖場跨 tick 只計 playing 部分。不是以 frame 數、浮點或牆鐘發錢。
- begin 保存本 tick delta、finish 在 gameplay 後取走一次並結算；重複 commit 不重付。存活改 ECS Gold，死亡改 persistent slot Gold；重生沿用原餘額。金額飽和到 i32::MAX。
- paused／Warmup／Finished 不發錢。私有收入計時加入 authority replay digest，不加入 replica gameplay resource。client 透過已有 CommittedEconomy／OwnerEconomy 與 IPC 得到結果，不預測收入。
- 舊商店／戰鬥隔離 fixtures 明確關閉收入；新增收入測試與兩種 profile 的完整 filtered lifecycle 使用正式收入。

## 驗收

- base_content 全部 72 項通過，包含 15Hz／120Hz 完整 filtered lifecycle 的死亡重生、終局凍結與逐 tick hash。新增合法購買測試之後單獨通過（目前共 73 項；新增測試後未再次全跑）。
- 新收入測試兩 profile 均驗證完整暖場、tick 內暖場跨界、暫停、死亡期間收入、重生、重複 commit、金額飽和與終局凍結。
- 正式 PlayerInput 經濟 fixture：0 Gold 購買拒絕 → 175 秒正式收入得 350 → 買 compiled Lua moba_sword 得 0／一件 → 出售得 175／空格。無寫入 Gold、沒有替換 catalog；此是 headless ECS 驗收，不冒稱真實 KCP 買賣。
- template tests 28＋23＋8＋1 通過；core 321 通過；runtime 46 lib＋3 bin 通過，真實 capture opt-in 另通過；bridge 44 通過、1 external capture opt-in ignored。
- 真實三程序 run `moba-runtime-1791049560` success=true、cleanup_verified=true；兩隊三方 hash 各 10／8 checkpoints，最後核對 tick 1200／1080，無 FAIL／repair。
- 使用原始 protobuf `presentation.capture` 驗收自己的 player／team、六格空、無 receipt、每筆 Gold 精確等於 floor(max(0, elapsed_raw - 2048) / 1024) × 2：team 1 共 571 snapshots，最後 tick 1371、18 Gold；team 2 共 224 snapshots，最後 tick 1205、16 Gold。這是實際 authority → KCP → filtered runtime → presentation IPC，不是人工 JSON。
- 重跑 capture：設定 `OMOBA_INCOME_CAPTURE_ROOT` 為上述 run 的絕對路徑，再執行 runtime 的 `real_single_lane_owner_income_capture -- --ignored --nocapture`。此測試限定 shipped 2 秒暖場／每秒 2 Gold、無 pause／交易的 smoke；修改平衡後須同步調整驗收規則。
- Unreal codegen 生成／check 均通過：11 檔、15 Lua inputs，identity content_hash `de9c7fcfc98d6479` 保持不變；catalog_data_hash 更新為 `7efdcb44ba406fa1`。兩者用途不同。
- full build／橋接 staging 通過，SHA-256 `a6978590f720087819d2422190b86f28c0c08aa3cfe45c3310bd124e61bff63d`；Editor PID 28056、MCP HTTP 30000 ready。同 Editor 兩輪 native automation 各 8/8 通過。
- 重建後 PIE native／ghost 呈現 smoke success=true，已確認 pie-after=false；再次 verify-staged-only SHA 一致。這是 renderer 基線回歸，沒有將 fixture screenshot 當成真實經濟 HUD。
- build-only 曾被開啟的舊 Editor 擋住；MCP 保存資產成功後改 full。restart 正常 taskkill 等待 10 秒失敗，自動 force 結束已驗證舊專案 PID 78632 與其子程序，再重建／重開；不能記成 graceful 成功。沒有停止其它專案 Editor。

## 仍待完成

整體 OpenSpec 保持 17/30，5.3／6.2 不勾選。secure KCP 買賣仍關閉；下一步是獨立 shop capability／查詢節流、正式 input intents、pending receipt 恢復與真實 terminal 買賣，然後開啟通用 native shop UI。擊殺／助攻金錢、回城、三路、選角與完整 UI 亦尚未完成。

本輪 Unreal build／Editor automation 不等於真實雙 Unreal 顯示收入；已有真實 IPC 收入擷取，Unreal 實際 income HUD 尚需後續雙客戶端驗收。問題集中記於 `unreal-moba-error-register.md` 的 E093。
