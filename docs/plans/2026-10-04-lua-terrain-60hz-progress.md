# Lua 地形編譯與公開碰撞接入

## 本輪計畫

- [x] Lua map 增加整數矩形 terrain 與 generated MobaTerrainConst。
- [x] 驗證 id、數量、範圍、非零面積、兵線 corridor、五人出生點及野怪完整 leash。
- [x] 編入完整 map catalog hash／compiled agreement，compiled map 接入 BlockedRegions。
- [x] 固定點 broad phase＋矩形 stack buffer，避免每個 BFS edge 為矩形分配 Vec。
- [x] 真實 Lua 地形正式 MoveTo／60Hz 三seed雙隊逐tickhash與重播。
- [x] 真實三路 KCP server＋雙外部 runtime 60Hz 基線。
- [x] 完整回歸、最後 Unreal build-only／DLL SHA。

## 決策

每個地圖最多32個 `{id,min={x,y},max={x,y}}`，整數座標±100000；預設空保持旧manifest相容。地形 min／max 必須嚴格遞增；只宣告碰撞，不假定同時遮蔽視野。驗證使用 omoba-sim 相同 i128 swept-circle，而不是另一套浮點算法。

選定 compiled map 才以它的地形取代 BlockedRegions；None 不覆蓋既有單路／TD 地形。完整 Lua map catalog JSON／hash原有gate涵蓋terrain，禁止單邊熱更新。後端已經將 BlockedRegions 序列化為公開 bootstrap metadata，filtered builder同路徑載入；沒有傳營地仇恨／timer或完整世界。

NPC尚未有通用detour，所以兵線100-unit corridor、所有兩隊五人出生點20-unit半徑，以及 camp leash＋100-unit margin，必須不受地形阻擋。此限制明確是過渡安全條件，不能當作NPC已實作地形導航。training圖配置north／south island；不增加角色C++或Blueprint graph。

## 驗證中與剩餘

新增 Lua validation 拒絕浮點／未知欄位／錯誤維度／重複ID／反向或零面積／超界／擋兵線／侵入leash，並驗證合法變更會改hash。

- Lua地形正式輸入測試通過：seed1／42／539365380各1200tick，共7200雙隊filtered steps。先正式MoveTo靠近island，再第二次MoveTo跨牆；明確觀察繞路並抵達，兩份權威replay一致，兩隊逐tick對authority projection hash相同／零ComponentRepair／敵方輸入私有／filtered沒有MobaMatch。沒有teleport或注入直接位移。
- 最後測試：omoba-sim69 unit＋8 determinism、core338、template runtime-lua-content39 unit＋23 generated＋8 hero＋2 catalog通過，出生點專項補強後terrain_catalog再次通過。
- 完整base107全部通過（271.10秒，含真正Lua地形與既有手工薄牆／三路完整lifecycle／野區／經濟／技能）；server156＋1benchmark ignored、runtime61＋8opt-in ignored＋3integration、bridge53＋1opt-in ignored＋2integration／1opt-in ignored通過。矩形快路徑前base106为301.43秒，非相同test count／同負載的正式benchmark，不以此宣稱效能比例或60FPS。
- 既有 `scripts/run_moba_runtime_smoke.lua` 新增 opt-in `OMOBA_MOBA_SMOKE_MODE=three_lane`，60Hz且與商店／回城／roster／重連等fixture互斥。預設single_lane不改；只在獨立evidence配置檔設three_lane，不改omb/game.toml。報告kind／gameplay_mode明確，不稱為單路或Unreal測試。

## 實際 KCP 與最後建置

- `OMOBA_MOBA_SMOKE_MODE=three_lane`／FPS60／port58261，`tools/lua/lua.exe scripts/run_moba_runtime_smoke.lua`：run1791112988 exit0／success=true。雙隊正式MoveTo位移，9／8 unique三方checkpoint至1080；獨立讀完整保存JSONL，PASS rows12／8、pre/post parity皆true、零FAIL。大量UNVERIFIED是未到checkpoint的行，不等於每個KCP tick驗hash。
- 報告 `target/interactive-runs/moba-runtime-1791112988/moba-runtime-smoke-report.json` SHA-256 `cb883f239d02aa9e80b61ecd8fb17e47d23747d461899c8e80f55385d9095dd3`。server76400／runtime106140／86944皆由Lua正常清理，報告cleanup_verified=true，另外CIM精確PID核對已不存在。
- 此KCP是三路移動／版本／迷霧／bootstrap基線，不是現場跨island繞障、完整勝負或UE地形畫面；跨牆由上面的正式ECS雙隊逐tick測試驗證，兩者不混稱。
- 最後 `tools/lua/lua.exe scripts/build_ue_moba.lua --build-only` exit0，OmGameEditor Win64 Development `Result: Succeeded`／up-to-date；無Editor／PIE啟動。
- 獨立 `--verify-staged-only` exit0：bridge SHA-256 `094de8152a204243e515e7ed782d5430b491c2477567ff5f5092906a4d181200`。
- script debug build／根scripts/base_content.dll／Unreal stage三份獨立SHA相同：`c5c4d8c6326b07a4dfab840bc3d8f220a123a69d773865bee8623328a30c5564`。不拿舊release headless或omb/scripts DLL當本輪證據。
- 最後codegen `--check`：11files／16Lua inputs、content_hash3b296ff5bdbcd7d9，exit0；OpenSpec strict及scope diff check通過。Lua完整map data hash變更，不把未變的art content hash誤當地形未更新。

整體5.4仍未完成：NPC／野怪通用避障、Unreal三路地圖呈現、完整建築層級未交付。本輪不宣稱60FPS或硬即時tick budget；矩形快路徑不是完整靜態polygon cache，也不需新增有失效風險的第二份collision resource。

錯誤與修正集中在 `docs/plans/unreal-moba-error-register.md` E129。

本輪Lua地形接入增量完成；總進度仍20/30。下一段抽出可共用的靜態尋路供NPC／野怪使用，再擴UE三路公開layout與建築層級；不勾選完整5.4。
