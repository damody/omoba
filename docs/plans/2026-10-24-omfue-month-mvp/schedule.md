# 工作清單與相依

**依 README §0：每項工作都以「遊戲變得更好玩、更完整」為目標，平日驗證從輕，里程碑時才整理完整證據。**

這份清單沒有日曆。全部項目目前都是 `todo`。執行時從「前置都已完成」的項目中挑最早的一項；一次只保留一個主要 `doing`。項目提早完成就直接接下一項，不等日期。

本月以 omfue 的可玩結果為中心。任何 Rust／schema 工作都要能回答：這讓哪個 UE 操作、來源技能或美術流程變得可用？

「人工」欄標示需要使用者在場或提供外部條件的項目。這些項目要提早預約；條件未到位時先做其他項目，不要空等。

## 1. 工作包總覽

| 工作包 | 內容 | 主要位置 | 前置 | 對應 gate |
|---|---|---|---|---|
| A | 基準、啟動拓撲與環境 | 本計畫、`scripts/moba_role_launch.lua`、`scripts/moba_host_budget.lua` | 無 | G0 |
| B | 規則式 autopilot | omfue Development／Editor 模組 | A3 | G2 |
| C | 可玩流程與操作 | omfue runtime／Editor、既有 session scripts | A3 | G2 |
| D | 來源索引與選角 | `scripts/` Lua、`docs/character_pipeline` | A1 | G1／G3 |
| E | 英雄 package 與作者工具 | `docs/character_pipeline`、`scripts/gen_hero_registry.lua`、base_content、template IDs、omfue/codegen | D1 | G1／G5 |
| F | Editor 美術設定 | OmRuntime／OmEditor、asset recipe／HUD asset registry | E3 | G4 |
| G | 通用技能能力（只做選定 kit 需要的） | script ABI／host／replica／UE presentation | D3、E3 | G3 |
| H | 來源 kit 量產 | authoring／script／bot／UE 綁定 | E3、G 中該 kit 需要的項目 | G3 |
| I | 盲測與作者 UX | 作者工具、錯誤診斷 | H 至少 4 套、F2 | G5 |
| J | 連線、效能、打包 | omfue、client runtime、Lua workflow | A3；打包 smoke 持續做 | G6 |
| K | 發佈與交接 | guides、acceptance index、release output | 所有核心 gate | G6 |

主要路徑：A → D → E → G／F → H → I → K。B、C 與 J 的打包 smoke 從一開始就並行推進，不推遲到最後。

## 2. 里程碑（整合點）

里程碑由工作完成觸發，不按日期。每個里程碑要做三件事：

1. 單機 B＋C＋C 拓撲（test-topology.md）跑一整局：2 個 autopilot＋8 bot，自然結束後進下一局。
2. 使用者 L3h 試玩一局。
3. 打包 smoke 一次。

另外整理該階段的 gate 證據，並且只處理阻擋整合的問題。

| 里程碑 | 觸發條件 | 判定 |
|---|---|---|
| M1 可玩基線 | A、B1–B3、C1–C5、E3 完成 | G1、G2 |
| M2 可重用能力 | F1–F3、H01＋H02 完成 | G3 進度 2/N、G4 中期 |
| M3 內容量 | H 達最低線、F4、I1 完成 | G3、G4 |
| M4 發佈候選 | I2、J 全部、E5 完成 | G5、G6 初判；之後只修 release blocker（K1），凍結新功能 |

未通過時：先補主要斷點，再回到清單。M1 沒過不得開始大量複製英雄。

## 3. 工作清單

### A. 基準、啟動拓撲與環境

| ID | 工作 | 前置 | 人工 | 完成條件 |
|---|---|---|---|---|
| A1 | 建立 `execution-status.md` 與 `acceptance-index.md`。記錄各 repo HEAD、dirty 檔案、工具版本、UE engine（根目錄 README：`D:\UE5.8`）與正式入口（README §0.1）。Rust／UE 建置基準沿用 `2026-10-25-minion-cadence` 並標「沿用」，只在 dirty diff 影響到時重跑相關部分。核對原 11 項缺口與 `handoff.md`、進行中的 OpenSpec changes，避免覆蓋他人工作。 | — | — | 已知道哪個入口能達到哪層驗收；之後不會把 synthetic 當真局。 |
| A2 | 修正同機多個 `--connect` 的 presentation port 衝突。已確認：`moba_role_launch.lua` 第 349–352 行用 `options.port+ordinal`，每次呼叫 ordinal 都從 1 開始。改為用 player ID 推導 port，或加明確的 port 基底參數；權威 port 不變。同時處理 `moba_host_budget` 的 CPU 預算重複計算，可加「同機總程序數」參數。 | — | — | 兩次模式 C 在同一台機器上不衝突、預算不超額。 |
| A3 | 單機 B＋C＋C 跑通到雙方進場。量測 2 UE＋3 Rust 程序的記憶體與 CPU，必要時加 1 個 Editor。依此固定硬體／解析度／畫質基準與效能門檻（README §8）。 | A2 | — | 有可重複的啟動命令與資源基準；owned cleanup 乾淨。 |
| A4 | 找出程式與工具中寫死的局數上限，改成可設定，並記錄改了哪裡。 | A1 | — | L3／L4 不再被舊 `10/10` 預算擋住。 |
| A5 | 第一次打包可行性檢查，提早發現 Editor-only plugin、DLL、asset cook 問題。 | A1 | — | 已知打包阻塞清單。 |
| A6 | 人工與硬體需求清單：第二台機器、Editor 操作時段、OS DPI 切換、可用素材、防火牆。確認已暫停的無人值守排程 `omoba-48` 維持暫停，不與本計畫的對局同時跑。 | A1 | 使用者確認 | 每個人工需求都有取得方式或明確 blocked。 |

### B. 規則式 autopilot

契約見 test-topology.md §4。這是 L3a 證據的基礎，A3 完成後立即開始。

| ID | 工作 | 前置 | 完成條件 |
|---|---|---|---|
| B1 | v0：`UOmAutoPilotComponent`（暫名）與 `-om-autopilot=<profile>`，由 `moba_role_launch` 依玩家傳入。先只做右鍵移動與普攻，走 input-component 入口。每個動作寫 trace：request ID → 權威結果 → UI。Shipping 包不編譯。 | A3 | 一局內產生 trace，可對到權威 ACK。 |
| B2 | v1 戰鬥操作：A、Shift+A、MoveOnly、AttackOnly、S/H、自用技能；依 `bot_usage` metadata 施放技能。 | B1 | 每個操作都有 accepted／rejected 加原因。 |
| B3 | v2 對局循環：買裝、合成、賣出、undo、升級、回城、插眼、ping、聊天、小地圖點擊、計分板、死亡重生、結算 → 下一局。輸出 `coverage.json`。 | B2、C4 | `coverage.json` 全部項目都有結果。 |
| B4 | Slate 模式：在 Slate／Enhanced Input 層合成實體事件，含螢幕座標與 DPI 換算。供 J4 使用。 | B3 | 同一套 profile 可切換兩種輸出模式。 |

### C. 可玩流程與操作

| ID | 工作 | 前置 | 完成條件 |
|---|---|---|---|
| C1 | 移動與戰鬥輸入閉環。逐條驗右鍵、A、Shift+A、MoveOnly、AttackOnly、S/H、英雄優先、auto attack、queue、取消、自用、前搖取消、攻擊後移動、UI 攔截。串 request ID 與權威 accepted／rejected，`submit sent` 不當 ACK。修焦點丟失、按住重複、死亡與切選角後殘留輸入。對應原 3.1／3.3／3.4／3.5／4.3。 | A3 | 真對局可完整操作；無重複命令、無 hidden actor target。 |
| C2 | MOBA Editor 入口：Editor 中提供明確的 MOBA 開發設定或入口，沿現有 PIE flow（`scripts/run_moba_pie_ue.lua`），保留 TD 選項，不改壞 `DefaultGame.ini` 與無參數 `run_ue.bat` 的單機 TD 預設。統一選角、進場、結果畫面的返回行為。 | A1 | 使用者不讀內部文件也能找到英雄與開始入口。 |
| C3 | HUD 收斂：以當前真畫面檢查 720p／1080p，修核心血量、技能、道具、小地圖的裁切、重疊與 tooltip 遮擋。診斷資訊移到可開關區。對應原 4.7 的布局部分。 | C2 | 核心操作列與必要面板完整可達。 |
| C4 | 商店與成長：真對局買、合成、賣、undo，升級技能、回城、死亡重生、ward／trinket，UI 與權威結果一致。對應原 3.9／4.4。 | C1 | 交易與成長全部走權威回覆。 |
| C5 | 對局推進：沿現有三路 map 驗小兵、塔、兵營、基地、野區、視野與 bot 推進。只修阻擋完整遊玩的問題，例如 bot 卡住、不買裝、不放技能。驗結算 → 返回／下一局沒有舊 actor 或 pending command。 | C1 | 只靠正常操作就能推塔到勝負；沒有用測試指令宣布贏家。 |
| C6 | 團戰可讀性：用交付英雄混合組隊打一局，修輪廓、選取、效果疊加、名牌／HUD 問題。 | H 至少 4 套 | 團戰中能分辨敵我、施法方向、命中與狀態。 |

### D. 來源索引與選角

| ID | 工作 | 前置 | 完成條件 |
|---|---|---|---|
| D1 | Lua KV 解析器加全量 provenance 索引。固定來源 commit；處理註解、巢狀、跳脫、BOM、placeholder、rank 陣列、缺值與繁中字串。97 個主檔加變體與 `_old` 全部列入，保留 path、hash、variant。不執行外部 repo 腳本。這一步是機械工作，必須全量完成。 | A1 | 任一索引項都能回到原檔原 token。 |
| D2 | 全量粗分類：用關鍵字／KV 欄位把每招標上可能的能力族；無法判斷的標 `unclassified`，不硬猜。輸出完整來源 ID → 遊戲 ID 對照，並記錄 B02 名稱衝突。 | D1 | 能用能力族篩出候選。 |
| D3 | 候選詳細分類：只對 6 位交付、2 位盲測、約 4 位候補英雄逐招人工閱讀，標 `supported`／`needs_framework`／`ambiguous`／`deferred`。沒讀過的技能不得標 supported。 | D2 | 每位候選都有能力需求表與 slot 映射。 |
| D4 | 選角名單：交付名單須涵蓋射手、近戰、施法者、支援、召喚或切換。B01 雜賀孫市是 H01 第一候選（本地已有 `scripts/base_content/src/heroes/B01_saika_magoichi`，需核對來源版本）。另列 2 位盲測（不先寫專用 handler）與候補，並寫明換人規則。 | D3 | 名單、候補與假設政策已固定。 |

### E. 英雄 package 與作者工具

| ID | 工作 | 前置 | 完成條件 |
|---|---|---|---|
| E1 | 擴充既有 `docs/character_pipeline`：已有 `schema/`、`packages/`（`hero_example`、`hero_smoke_astral_ranger`）與 `tools/validate_package.lua`。在這套 schema 上補 README §4.2 實際需要的欄位，不另起一套平行 package。package 最終落到 `scripts/lua_data/templates/`（heroes、abilities、`moba_archetypes.lua` 等），數值與目標驗證走 `omoba-content-model`。 | D1 | schema 與 validate 能涵蓋 H01 需要的欄位。 |
| E2 | 作者工具三個子命令（入口名稱實作時決定，經 `tools\lua\lua.exe scripts\<tool>.lua` 執行）：`index`（來源解析，複用 D1）、`validate`（package、`omoba-content-model` 與能力支援檢查）、`build`（附加 Lua 宣告 → `gen_hero_registry.lua`／`omoba-template-ids` 生成 → 呼叫既有 `scripts/build_ue_moba.lua` → `test_compiled_content_contract.lua`）。只附加在宣告尾端、不重排 ID。build 失敗時不更新 `scripts/base_content.dll` 與嵌入 catalog。preview／accept 不另做命令，直接開一局。 | E1 | 三個子命令可用，失敗時 exit code 非零並指向具體欄位；server、runtime、DLL、bridge 的 content hash 一致。 |
| E3 | H01 薄切片：來源 → package → Rust handler 或 generic effect → 註冊、tooltip、選角、佔位呈現 → UE 施法。重複生成時 ID 不漂移。 | E2、D4 | 至少一招從 UE 輸入到權威效果與畫面可追；整套 kit 未接受的部分照實保留。整套必要能力接通才算 G1。 |
| E4 | 生成可靠性：測缺 rank、錯 target、重複 ID、無效數字、缺 API、缺資產、錯骨架、版本不一致與 failed build；核對回滾、idempotence 與人工外觀保護。整理生成指南與固定 prompt 模板。 | I1 | 壞 package 不進正式選角、不覆蓋上一版。 |
| E5 | 自由文字入口：直接貼英雄描述，不帶來源 ID，也能走完同一條流程。 | E4 | 原文 → 新英雄 → omfue 可玩，或明確失敗。 |

### F. Editor 美術設定

這部分需要在 Editor 裡操作、存檔、重開與檢查。若執行者無法操作 Editor GUI，要預約使用者在場。

| ID | 工作 | 前置 | 人工 | 完成條件 |
|---|---|---|---|---|
| F1 | 外觀設定最小方案：沿現有 `AOmAnimatedUnitActor` 欄位與 Blueprint，或補小型 DataAsset。實作「人工覆寫 > 生成預設 > fallback」，並提供建立／開啟、Validate、Preview 入口，只操作指定英雄。 | E3 | Editor 驗證 | Save 後重開與 regenerate 都不回復預設；現有英雄相容。 |
| F2 | 解除 omfx 資產依賴：改 `omfue/codegen/src/lib.rs` 的 `recipe_source("omfx", …)` 與 `scripts/tests/ue_art_swap_editor_test.lua` 的替換圖來源。補缺失與 cook／soft reference 診斷，以及 maintained-source 依賴檢查。 | — | — | 正式 build、generate、art acceptance 都不讀 omfx。 |
| F3 | 動畫、技能 VFX、音效槽位：統一 idle、move、attack、cast、channel、death、recall 與技能 cue 映射。施放、飛行、命中、循環、結束、取消都由 instance 管理；AnimNotify 不控制扣血。 | F1 | Editor 驗證 | 換長短不同的動畫後權威命中 tick 不變；取消、死亡、離開視野、PIE stop 後無殘留。 |
| F4 | 兩種不同骨架或模型的 Editor-only 替換驗收：scale、材質槽、socket、retarget、圖示、VFX、音效。逐一走過 Save、重開、regenerate、PIE、cook 與 packaged smoke。產出 `unreal-art-guide.md` 草案。 | F3、H 至少 2 套 | 使用者操作 | 不改 Rust、C++、生成檔或傷害 Blueprint；錯素材給診斷而不 crash。滿足 G4。 |

### G. 通用技能能力

只做 D3 選定 kit 實際需要的能力。開始前先查既有實作，沿現有 ABI、結算與 replica 擴充，不另起框架。根目錄 README 已把「NPC 普攻前搖／彈體、更多追蹤／碰撞／多段技能與召喚控制」列為待完成，G-1～G-3 從這裡接續。新增權威事實或呈現欄位時，依既有規則升 selective（目前 15）／presentation（目前 12）／CABI（目前 36）版本，舊版 fail closed，並更新根目錄 README。

| ID | 工作 | 觸發 | 完成條件 |
|---|---|---|---|
| G-1 | 技能投射物：直線／追蹤、speed、radius、lifetime、穿透／停止、牆與目標失效；固定步進、命中去重、穩定的多目標順序。UE 只讀權威的合法狀態。 | 交付 kit 有飛行技能 | 近距與遠距的飛行時間看得出差別；不以瞬發傷害加假飛行冒充；霧內資料不洩漏。 |
| G-2 | 複合技能狀態：多段、recast、蓄力、toggle，只做必要子集。寫明每階段的扣費、CD、取消、死亡、rank 變更與目標失效；UI 顯示次數、窗口與下一段。 | 交付 kit 有多段技能 | 多按、晚到事件、控制打斷都不多扣費；重連後狀態一致。 |
| G-3 | 召喚控制與額外 action slot：選取、跟隨、移動、攻擊、召回、死亡清理；處理原 W/E/R/T/D/F 的額外技能。 | 交付 kit 有召喚或額外按鍵 | 主人退役、召喚物到期、視野變化、無效命令都正確處理；bot 不偷讀私有位置。 |
| G-4 | 其他能力（隱形／偵測、推拉、擊飛、衝鋒等）：依 D3 需求逐項開項。 | D3 標為交付必需 | 同上原則；無來源需求的能力不做。 |

### H. 來源 kit 量產

本月**最低 4 套，目標 6 套**完整來源 kit。每套的工作相同：由來源描述先寫預期行為與測試，再生成實作；完成 bot usage、tooltip、預覽、動畫、選角，以及每個 rank 的接受。完整 kit 才算數；技能不完整的角色留在候選目錄，不改名充數。

| ID | 定位 | 前置 | 完成條件 |
|---|---|---|---|
| H01 | 射手（E3 的延續） | E3 | 整套必要技能可用。 |
| H02 | 近戰／位移／強化普攻 | H01、需要的 G 項 | 位移與命中不受動畫或 mesh scale 影響。 |
| H03 | 施法者／範圍 | H01、需要的 G 項 | 不靠改顯示名稱複製 H01。 |
| H04 | 支援／控制／護盾 | H01、需要的 G 項 | 治療或護盾被拒絕時不顯示成功。 |
| H05 | 召喚或切換 | G-2 或 G-3 | 召喚 owner 退役、到期正確。 |
| H06 | 覆蓋剩餘能力的特殊型 | 需要的 G 項 | 超出本月範圍時，依 D4 候補規則公開換角，不刪技能。 |

H02–H06 彼此沒有相依，可依 G 項完成順序調換。

### I. 盲測與作者 UX

| ID | 工作 | 前置 | 完成條件 |
|---|---|---|---|
| I1 | 第一位盲測：只交原始描述與來源資料，不給專用 patch。記錄人工干預、共用框架變更、生成次數與缺值。 | H 至少 4 套、F2 | 完整 kit，或明確的 `needs_framework` 阻塞。已標 supported 的能力不需新增 UE hero-name 分支。 |
| I2 | 第二位盲測：機制與第一位不同；兩位中至少一位需要 AI 新寫 Rust handler。 | I1 | 兩份盲測報告。滿足 G5。 |

### J. 連線、效能、打包

| ID | 工作 | 前置 | 人工 | 完成條件 |
|---|---|---|---|---|
| J1 | netem 接到模式 C（test-topology.md §3 第 3 點），加斷線 10 秒重連；檢查狀態補齊、輸入恢復、命中／扣費／購買不重複、視野不洩漏。 | B3 | — | 已定義 profile 下的 L3a-loopback 網路證據。 |
| J2 | 真雙機 LAN：把一個模式 C 移到第二台，逐項驗移動、攻擊、技能、道具、聊天、ping、ward、死亡、勝負。 | J1 | 第二台機器、防火牆 | 真 LAN 與 loopback 分開標示；沒有第二台機器就標 blocked。 |
| J3 | 打包 smoke：每個里程碑做一次。 | A5 | — | Editor 成功但包不能啟動的問題能及早發現。 |
| J4 | DPI 與視窗矩陣：OS 100%／150% DPI、1280×720／1920×1080、windowed／borderless、Alt-Tab、拖移／縮放、chat／shop focus、PIE stop/restart。用 B4 的 Slate 模式。對應原 7.3。 | B4、C3 | OS DPI 切換 | 核心操作在支援矩陣可用；未實測項保留 unknown。 |
| J5 | 固定硬體效能與兩小時 soak：依 A3 鎖定的門檻，記錄 frame、simulation、memory 與 actor／VFX／audio 峰值；由 2 個 autopilot 提供負載，另測集中施法。不以 10000 entity 壓測替代。 | B3、H 至少 4 套 | — | 符合門檻，或公開標為 release blocker；不在最後才改門檻。 |
| J6 | 乾淨打包：正式 cook／stage UE、runtime、backend、script DLL、catalog 與素材，用 Lua 工作流執行。從乾淨目錄啟動，確認無開發目錄依賴、無 omfx 依賴。 | J3、F2 | 無 Editor 測試機（可選） | 至少能從選角走到可操作，不需開 UE Editor 補救。 |

### K. 發佈與交接

| ID | 工作 | 前置 | 完成條件 |
|---|---|---|---|
| K1 | 只修發佈阻塞：crash、無法開局、技能錯誤、生成覆蓋美術、重連、打包。凍結新英雄與非必要功能。 | M4 | 每個修正都有證據；沒有放寬斷言或刪掉失敗情境。 |
| K2 | 交付文件：README §10 列出的 release-notes、player-guide、hero-authoring-guide、unreal-art-guide、source-catalog、acceptance-index、next-month。同步更新根目錄 README 的「目前實作與驗收進度」、Quick Start 與版本敘述，以及 `handoff.md`。 | K1 | 新 agent 能從文件接手；最短重現路徑寫清楚：啟動 → 打一場 → 貼新英雄 → 生成 → Editor 換模型。 |

## 4. 機制工作超時時如何取捨

1. 先保 G2 可玩流程、G4 美術覆寫、G5 描述閉環，避免做出很多技能但仍無法玩。
2. D4 依機制風險準備至少兩位候補英雄。換人要公開；保留原英雄與換人原因，不能用改名的變體冒充新的來源英雄。
3. G 的通用能力只做交付 kit 必要的部分；其他放 backlog，但不得偷偷省略來源的必要技能。
4. M3 之後不新增大架構。未支援的複製技能、地形改造、多形態等延到下一輪，並保持診斷清楚。
5. 可延後：額外地圖、推薦出裝的精緻 UI、更多召喚師技能、符文、觀戰、帳號服務、專屬美術。
6. 不可交換掉：真實命中、資源一致、完整 kit、可見性、人工美術保護、可用啟動、真實證據。

## 5. 交接格式

執行時在 `execution-status.md` 裡，每完成一批工作寫幾行。欄位不適用就留空；首次建立時不得把模板誤當完成紀錄。

```text
日期：
目前工作 ID／狀態：
玩家或作者新增能做到的事：
repo HEAD／本次修改：
來源英雄與 variant：
已執行命令、exit code、build identity：
L1／L2／L3／L4 證據（fresh／沿用／未執行）：
gate 與原 11 主項的變化：
失敗、原因、尚未修的風險：
下一個工作 ID：
人工／硬體需求與是否已取得：
owned process／資源清理結果：
```

平日驗證只做到「改的東西能用、沒弄壞旁邊」：編譯、相關測試、實際開一局看一眼。純文件修改不必重建 UE。不可讓每個兩行修改都觸發全量驗收，結果一直完成不了核心 gate。
