# 安全施法 cue 契約進度（2026-10-05）

## 問題與決策

- 成功施法來源 gate 已完成，但正式 renderer IPC 的效果管線只處理 DMG1 傷害。不能把輸入 ACK 當成功施法，也不能直接複製 embedded runtime 的全世界 script visuals。
- `project_fact` 共用條件原本允許目標可見時發布 Ability；施法者不可見時，subject 可能退回目標，讓目標看似施法者，並揭露隱藏施法技能。改為 Ability 必須同時有目前可見施法者與 Disclosed replica mapping，否則整個施法事件不發布。傷害／AOE 的目標限定 sanitizer 維持原契約。
- 現有安全 Ability public event 只有穩定技能 ID（8 bytes）及施法者 subject，沒有目標／座標。新共用 `AbilityPresentationCue` 使用 ABY1＋tick／caster replica ID／disclosure epoch／stable ability ID（固定 36 bytes），不補造未揭露欄位。
- `from_public_event` 僅接受 Ability、精確 payload、非零身分及 live epoch，並拒絕事件 ID 的 tick／ordinal 溢位。`from_effect` 驗證版本／長度／tick／ordinal 及當前 live epoch，不把 remembered ghosts 當即時角色。技能穩定 ID 不是 UE catalog index，後續消費端仍須做 catalog lookup。

## 本批實作範圍

- 共用投影安全 gate、共用編碼／解碼／admission API 及三項指定測試。
- 沒有改 C ABI 13、IPC 4、Lua catalog 或執行期 Lua 設定；只維護 Rust／omfue 方向，不處理 omfx。
- 本批尚未將 ABY1 接入正式 IPC 保留／ACK／重連去重或 Unreal OnAbilityCue。這些是接下來的實作，不是目前已完成能力。不能只把效果加入最新 snapshot，否則會在低 renderer frame rate 下遺失一次性事件。
- 未宣稱目標位置／切換技能 payload／陣形資訊已支援；不得以推測資料觸發錯誤視覺。

## 局部確認

指定指令：`cargo test --manifest-path omoba-core/Cargo.toml --lib ability_cue -- --nocapture`。

最後版本結果：3 passed／0 failed／428 filtered out，exit 0；包含補強 remembered mapping 案例後的重新編譯。既有模板 build script 三項 dead-code warnings 未隱藏。OpenSpec strict validate 與根工作樹 whitespace check 通過；沒有全套重驗、重新部署 DLL 或 Unreal gameplay 驗收。

測試涵蓋：隱藏／remembered／forgotten 施法者對可見目標不發布、可見施法者對隱藏目標只發布技能 ID、無目標且隱藏時不發布、格式截斷／超長／未知版本／零身分、DMG1 與 ABY1 不混讀、stale／缺失 live epoch、非法 event kind／payload／tick／ordinal。

OpenSpec 6.1／4.3 維持未完成，總進度仍為 21/31。完整 Unreal／對局／LAN／效能驗收留到整體實作完成後。

## 防錯

- 查找時猜根 Cargo.toml、replica.rs、team_identity.rs 造成 missing path；改在實際 inventory／型別搜尋定位 selective.rs。PowerShell Skip 誤填 sixty，改整数。這些是工具錯誤，不是程式編譯錯誤。
- 歷史 tasks／搜尋的巨量輸出遭截斷，改限定精確標題與小段，不以截斷內容作驗收證據。
- 編譯期間補強 Disclosed mapping 負向案例，須重新編譯指定 filter，不能以先前三項成功代替最後版本證據。

詳見防錯紀錄 E218。
