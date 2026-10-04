# Unreal 四槽輸入實作與驗收

## 本輪計畫

1. 補上原生 Q/W/E/R 通用槽位輸入，使用目前 owner HUD 的 generated ID 與 target type，不新增英雄專屬 C++／Blueprint graph。
2. 只有明確 opt-in smoke 才自動移動至單路交戰位置與提交四槽；使用同一正式施法 API、不從 runtime 代送、不查 hidden world。
3. 分別核對 queued input ID、terminal applied result、正值 HUD cooldown；不能把本機 queued、complete banner 或別槽結果當施法成功。
4. 固定 Lua build／stage／双 UE gate，保存失敗與成功證據、檢查三方 hash、清理本次精確 PID。錯誤集中 E076。

## 決定與邊界

- OmRuntime 不能依賴 OmGenerated（後者已依賴前者）；控制器透過 reflected 通用函式呼叫，維持既有單向 module 邊界。
- unit 目標只用當前 live actor 身分，point 要有有效 deprojection；後端仍負責敵友／距離／技能資格。沒有 HUD、死亡、非 Playing、未學或冷卻中不發技能。
- 測試 hero 交戰位置是既有單路 1000／1400，不是正式三路導航驗收；測試只從自身安全 snapshot 選另一個可見 hero，不能推導敵方隱藏狀態。
- 此次 opt-in API 操作不等於物理 Q/W/E/R 按鍵測試，也不等於死亡／重生／完整終局已完成。完整框架仍 17/30。

## 驗證

### 已完成的編譯與回歸

- core 303、base_content 62、server 139（另 1 ignored）、client 41、bridge 41 passed；base_content 的四槽 test 延伸施法後 120 tick，每 tick 核對兩隊完整 canonical hash。
- Lua movement 6／ability 7／parity 3 tests 與工具模組 tests 通過；complete banner、queued、零 cooldown、錯 player／input ID、rejected result、hash mismatch 均不能冒充通過。
- full build session 54205 exit 0，Editor PID 53480／MCP HTTP 30000 ready；本輪 staging bridge SHA-256 `0a3de7bb2f17e75a891321d1e01d72ba050a9d98b2309a04b686136b5a3f41bf`。
- 這個 Editor NativeVisual 兩輪各 7/7 通過，GameplayInputSurface 1/1 且 complete=true；PIE 通過並已停止。RememberedGhost 的既有 world-context warning 仍存在，不宣稱零 warning。
- `cargo check --manifest-path omfx/Cargo.toml --tests` 通過，保留舊前端相容。

### 失敗與決定

- 1791031584：施法中斷 approach move；新增到達位置後才能施法。
- 1791031862：兩個指定敵方技能 status=4；定位 server 把 generation-packed canonical u64 誤轉 u32。保留既有安全驗證後只轉 authority 內部索引；generation 7／index 42 回歸通過。
- 1791032111：第一隊四槽成功、第二隊部分成功；arrival guard 不應每招重新要求起點，改一次性 latch，正常追擊不鎖住角色。
- 1791032353：第二隊 tick 6360 最先 hash FAIL；逐 component 唯一不同為敵方 TAttack clock／phase。加入 typed visible enemy CommittedAttack，13 bytes 不帶 target 或 input，owner 不覆寫。詳見 E078，不降低 hash gate。
- 1791033230：第一隊四槽成功，第二隊 runtime 正式移動已到 1400，但 UE 未呈現到達窗口就死亡／重生；inbound_depth 最大 438。當時並行 PIE 與 Fyrox 編譯，需隔離負載再次驗收；沒有 hash FAIL，不把逾時當成功。詳見 E079。

- 1791033455：獨立負載仍重現，不能歸因於 PIE／cargo。確認 catch-up 即使不發布 latest snapshot，仍重算大型 fog envelope 再丟棄。修正為先判斷 Latest／Critical／不準備；lifecycle 與 input snapshot 仍走必送 FIFO，每 tick damage retention 不變，runtime binary 3 tests 通過。

### 修正後真實雙 Unreal

- `interactive-ue-1791033665` exit 0／success=true，五程序（authority、雙 external runtime、雙 UE -game）完整實際驗收；兩隊各 4/4 技能 status=0，四槽都有對應 input ID 的正值 HUD cooldown。
- 最後一招 team 2 tick 6509；gate 等兩隊 verified checkpoint 到 tick 6600，分別 65／87 PASS。結束檔案實際計數 67／89 PASS、零 FAIL；UNVERIFIED rows 不當成功。
- UE 自己送移動 120→1000／2280→1400，兩隊各自安全 view 與 consumed snapshot 861／1375 有證據；不是 runtime 替 UE 送命令。
- report：`target/interactive-runs/interactive-ue-1791033665/unreal-ipc-smoke-report.json`。所有先前 failed runs 保留。

- 第二次 `interactive-ue-1791033806` exit 0／success=true，同樣兩隊各 4/4 applied 與 cooldown；三方 61／67 PASS、零 FAIL、最後 verified tick 6360。UNVERIFIED 12923／12800 不算通過。safe ticks 6299／6335、consumed sequence 987／1547。
- 兩次各五個本次測試 PID 均已退出。第二次 server 71364、runtimes 18888／64760、UE 65820／55868；Editor 53480 保留且 PIE 已停止。
- 最終 runtime lib 41＋binary 3 passed，Lua 6／7／3 與工具模組 tests 通過；staged-only gate 再次核對同一 SHA。root／omb／omfue 本輪來源 CRLF-aware diff check 通過，不修改無關 manifest 行尾，不 commit／push／清除既有工作樹。

## 重現方式

使用既有 `scripts/run_2player_ue.lua --single-lane`，由固定 `tools/lua/lua.exe` 執行。環境設定 `OMOBA_RELEASE=0`、`OMOBA_SKIP_BUILD=0`、`OMOBA_SKIP_UE_BUILD=1`、`OMOBA_UE_SMOKE_SECONDS=75`、`OMOBA_UE_ABILITY_SMOKE=1`。只有預先 full build／staging 並通過 `scripts/build_ue_moba.lua --verify-staged-only` 後才能 skip UE build；正常互動入口不得設置這兩個 smoke 變數。

## 下一段計畫與未完成界線

1. 正式 UE 操作的物理按鍵／游標目標測試，從同一安全 projection 走死亡、重生、推塔到勝負；目前 API smoke 與 reflected surface 不冒充物理輸入。
2. 全生命週期 HUD／勝負 UI 與單路正式地形呈現；目前 visual fog／NPC wire kind 還有 FOG fixture，不宣稱正式三路場景。
3. current mana、經濟／商店／六格裝備、選角、小地圖／計分板、三路／野區／五位置 Bot 仍未完成。
4. LAN／混版拒絕、完整 cue／重連、美術替換與效能基線封關仍待驗證。OpenSpec 保持 17/30，不因這個垂直片段勾選完整 4.1／6.2。
