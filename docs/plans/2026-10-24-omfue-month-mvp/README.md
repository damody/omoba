# omfue：描述驅動英雄製作與可玩 MOBA MVP，一個月執行計畫

建立日期：2026-10-24（Asia/Taipei）。執行者：Grok 或其他具備本機程式碼、建置與 Unreal 操作能力的 coding agent。計畫不排日曆，只依 [schedule.md](schedule.md) 工作清單的相依關係推進；不要求連續跑滿 720 小時。

本計畫的產品是 **omfue**。使用者只提供英雄／技能文字，AI 完成 Rust 玩法、資料、測試與 Unreal 接線；使用者最後只需在 Unreal Editor 指派模型、材質、動畫、圖示、音效與特效。omfx 完全不列入開發、編譯、回歸或驗收；其刪除是未來另一項工作。

這是一份規劃交付，尚未執行下列實作與對局驗收。它也沒有建立 Grok 的排程或啟動一個月自動工作。

## 使用這份交接包

1. [audit.md](audit.md)：目前程式碼證據、與 LOL 類 MOBA 的產品落差、原有 11 項整合缺口。
2. 本文件：MVP 範圍、英雄生成架構、Unreal 美術契約、品質與交付門檻。
3. [schedule.md](schedule.md)：工作清單、相依關係、里程碑、完成條件與取捨規則。
4. [grok-prompt.md](grok-prompt.md)：可直接交給 Grok 的執行指令及跨工作階段續跑格式。
5. [test-topology.md](test-topology.md)：單機 2 client＋3 Rust 程序的測試拓撲、規則式 autopilot 契約、換到雙機的步驟。

先讀根目錄 `AGENTS.md`、根目錄 [README.md](../../../README.md)（目前進度、正式入口、建置與內容管線的現行說明）與 [handoff.md](../../../handoff.md)，再讀這五份文件。不要從舊計畫的歷史「下一步」直接開工。根目錄 README 與 AGENTS.md 說法不一致時，以較新的根目錄 README 與實際腳本為準，並在 execution-status.md 記錄。

## 0. 工作原則（優先於本文其他章節）

**盡快把遊戲做好。** 每一批工作的成果，都應該是遊戲本身變好：多一個能用的操作、少一個卡住對局的問題、多一個能選的完整英雄、多一種能在 Editor 換的外觀。測試數量、驗收文件和開了幾局都不是進度。

- **平日驗證從輕**：只跑和改動有關的測試，再實際開一局看一眼。完整證據只在里程碑（schedule.md §2）整理。
- **測試的目的是防止做好的東西壞掉**。不為純文字或排版修改新增鏡像測試，也不為了湊數重跑舊驗收。
- **證據紀律仍然有效**：L1–L4 分開記錄、fresh／沿用要標明、不拿 synthetic 冒稱真局。這是記帳規則，不是每天的工作目標。§8 的 gate 只在里程碑判定。
- **沒有日曆**：從 schedule.md 中「前置已完成」的項目挑最早的一項做，一次只保留一個主要 `doing`。提早完成就接下一項。
- **普通決策自己做並記錄**，例如命名、佔位、實作選擇。只有未知的技能語意、外部硬體或人工條件，才集中詢問使用者。

### 0.1 根目錄 README 已確立、本計畫必須沿用的現況

以下摘自根目錄 README（2026-10-25）。這些是本月工作的起點，不要重做，也不要寫成本月新增成果：

- **內容現況**：兩張 MOBA map、三路、16 營地、九個英雄配裝、19 項道具、五種召喚師法術與 ward、server 內 Bot。
- **原 MVP 清單**：33／44 完成、11 個父項待驗收。缺的是真權威 ACK、完整對局流程、LAN、實際 OS DPI、實體輸入與固定硬體效能證據。
- **已列為待完成的戰鬥能力**：NPC 普攻前搖／彈體、更多追蹤／碰撞／多段技能與召喚控制。與本計畫的通用技能工作（schedule.md G 包）一致，從這裡接續。
- **最新集中驗收基準**：`2026-10-25-minion-cadence`。九套 Rust 共 2,163 passed／0 failed／9 ignored；UE5.8 建置與 DLL staging 一致。A1 可以把它標成「沿用」，不必為了建立 baseline 重跑全套。
- **協定版本**：selective lockstep 15、presentation 12、Unreal bridge CABI 36。新增權威事實或呈現欄位時，要依既有規則升版，舊版要 fail closed，並同步更新根目錄 README 的版本敘述。
- **正式入口**：`run_ue.bat --moba` → `scripts/unreal_entry.lua` → `scripts/run_moba_session_ue.lua`，再重用 `scripts/run_moba_role_ue.lua`。預設配方是 `scripts/lua_data/moba_single_player.lua`、release、KCP `57061`、story `FOG_2TEAM_DEMO`、`STEP_FPS=60`。全本機真人可在結算後準備下一局。
- **選角**：`--interactive-selection` 支援一位真人（`-om-hero-selection`），兩到十位真人則走 `scripts/moba_shared_selection.lua`。這不是 LAN 共享大廳。
- **compiled-content-only**：正式 server、client runtime、`base_content.dll` 與 Unreal bridge 都不帶 Lua VM，也沒有執行期 Lua 熱重載。Lua 只用於作者資料、建置期生成與開局工具。
- **內容 ID**：Lua 宣告順序就是 ID 順序，ID 0 保留給 `UNSPECIFIED`，既有條目不得重排。驗證在 `omoba-content-model`，typed ID 由 `omoba-template-ids/build.rs` 產生，UE 端由 `omfue/codegen` 與 bridge `build.rs` 嵌入 catalog。
- **建置**：`scripts/build_ue_moba.lua` 會編 `base_content`、stage 到 repo 的 `scripts/base_content.dll`，再經 `omfue/restart` 編 bridge 與 OmGame。不要把 DLL 複製到 `omb/scripts/`。`tools\lua\lua.exe scripts\test_compiled_content_contract.lua` 檢查正式依賴沒有 `mlua`。
- **omfx 資產殘留**：英雄 `portrait` 目前寫成 `data/hero_portraits/...`，實際檔案在 `omfx/data/`。這就是 schedule.md F2 要解除的依賴。
- **無人值守排程**：`omoba-48` 已依使用者要求暫停。本計畫執行時不要自行恢復它，也不要和它同時跑對局。

## 1. 一個月真正要交付什麼

**核心目標：用灰盒美術也能完整玩一場，而且下一位英雄不必手寫 Unreal 玩法。**

| 交付 | 必須達到的結果 |
|---|---|
| 可玩的 omfue | 正式入口 → 選英雄／隊伍 → 讀取 → 三路對局 → 真實勝負 → 返回／再開一局。1 人類席＋9 bot 可玩；雙人類席＋8 bot 有實際連線驗收。人類席的例行驗收由 client 端規則式 autopilot 驅動，使用者在每個里程碑親自試玩一局（見 test-topology.md）。 |
| 玩家操作 | 滑鼠移動、普攻、A、Stop/Hold、技能、自用、升級、道具、回城、死亡重生、鏡頭、小地圖、商店、計分板都能在同一真實對局使用。 |
| 英雄內容管線 | 原始描述／來源 ID → 有版本的技能規格 → Rust script＋Lua 定義 → 生成註冊 → UE 可選、可操作、可看見效果。成功或不支援必須明確，不可悄悄換成別的技能。 |
| 英雄交付量 | 來源全目錄建立可追溯索引；完整可玩的來源英雄 kit 最低 4 套、目標 6 套，涵蓋射手、近戰、施法者、支援與召喚／特殊操作。另以 2 位未參與開發的來源英雄做盲測。配裝變體不算不同來源英雄。 |
| 美術接入 | Editor 中一份明確的外觀設定即可換模型、材質、動畫、技能 VFX、音效與 UI 圖示；重新生成不覆蓋人工設定。使用者不必改 Rust、C++ 或傷害 Blueprint。 |
| 發佈 | 一份可從乾淨輸出目錄啟動的 Windows 可玩包／本機後端組合；需要哪些 runtime、port、工具都列明。Editor 測試通過不能代替打包接受。 |
| 交接 | 操作手冊、英雄製作手冊、美術槽位清單、能力支援表、未支援項目、實際驗收證據、下一輪 backlog。 |

4（最低）／6（目標）＋2 是本月目標，不是聲稱目前已達成；最低線是預先公開的交付下限，不是事後降標。來源的 97 個主英雄檔不是 97 個已可移植英雄；本月先全量索引與分類，再對代表性 kit 建立完整流程。若盲測需要新增通用能力，必須如實記錄，不能將它算成「只靠描述零框架修改」。

### MVP 與完整版的界線

- 保留三路、塔、兵營、主堡、野怪、經濟、成長、裝備與視野這個對戰骨架。復用現有系統，先修玩家能遇到的斷點。
- 本月不承諾 LOL 全部英雄、全部裝備、完整符文與天賦、天梯配對、帳號商店、賽季、觀戰平台、賽事工具或大量新地圖。
- TD 只做被共用改動影響的最低回歸，不是本月產品擴充重點。
- 不要求精緻美術才算可玩，但佔位內容必須能分辨角色、敵我、施法方向、命中、狀態、死亡與重要目標。
- 不照抄某個 LOL 版本的數值。LOL 作為操作與完整對局體驗參照；英雄內容以使用者指定來源與明確改編規格為準。

## 2. 已確認的基礎與最優先問題

目前有 `OmHeroSelectionWidget`、`OmCommandBarWidget`、相機、瞄準、小地圖、商店、名牌、技能動畫、戰鬥特效、選角／下一局管理，以及相當多 Rust 玩法。不是從零重寫 MOBA。

優先順序如下：

1. **完整真實對局與預設入口**：`run_ue.bat --moba` 已存在，且已有真人選角與下一局準備；Editor 的 `DefaultGame.ini` 仍是 `SinglePlayer`／`TD_1`（根目錄 README 也記載無參數的 `run_ue.bat` 是單機 TD）。補成清楚、可重複使用的 MOBA Editor 開發入口，不靠記憶一串參數，也不隨意破壞 TD 入口。
2. **Editor 外觀覆寫契約**：已有 native visual 欄位與資產 recipe，但尚不能由此推定所有新英雄都能只在 Editor 完成美術綁定。
3. **描述到可玩英雄的閉環**：已有 `AbilityScript`／`UnitScript`、Lua metadata 與 registry generator；缺的是來源正規化、支援度判斷、完整 hero package、生成驗證、UE 註冊與回歸的一體流程。
4. **必要的新技能能力**：由來源 kit 的需求決定，先提供可重用的投射物、複合技能狀態、召喚控制等；不要無止境做與本月英雄無關的底層擴充。
5. **真實連線與打包**：必須提早做薄切片，到最後一週才接外部流程風險太高。

## 3. 目標流程與責任邊界

```mermaid
flowchart LR
    A[描述文字或固定版本的來源英雄] --> B[來源解析與技能語意規格]
    B --> C[能力支援檢查及假設紀錄]
    C --> D[AI 生成 Rust script 與 Lua 定義]
    D --> E[靜態檢查／玩法測試／註冊生成]
    E --> F[omfue 灰盒英雄與完整操作]
    F --> G[Editor 外觀設定與人工美術]
    G --> H[PIE／實際對局／打包驗收]
```

- **Rust / script**：技能判定、數值、資源、冷卻、命中、位移、召喚、狀態、死亡歸因，沿既有權威流程。
- **Lua authoring**：ID、數值、等級、目標規則、公開 metadata、AI 選技能提示、預設呈現資料；不得再維護一份互相漂移的技能規格。
- **omfue C++ 通用框架**：輸入、預覽、狀態投影、共用呈現、資產管理、Editor 工具。新增已支援類型的英雄時不加入 hero-name 特判。
- **Blueprint／DataAsset**：外觀與設計者設定。動畫 Notify 可以發聲／特效，不能決定扣血、命中或真正位移；不同動畫長度不得改變玩法。
- **使用者**：補素材、指定骨架／socket／動畫映射、調整呈現與檢視美術結果。程式接線、輸入綁定、生成 registry、技能邏輯都由 AI 完成。

優先沿用現有 `AOmAnimatedUnitActor`、`OmGenerated`、asset recipe、HUD asset registry、combat visual subsystem。只有現有機制缺少穩定覆寫點時，才增加小型通用 presentation profile；不要另建第二套戰鬥框架。

## 4. 來源匯入與技能語意契約

### 4.1 來源基準

盤點時取得的來源 commit：`ca6ee8602473f6015a397cc121dac6baeb70b2e5`。

- 倉庫：[damody/nobugg_official](https://github.com/damody/nobugg_official)。它是信長英雄資訊網站／資料來源，不能假定附有全部遊戲腳本。
- `npc/A_Oda`、`npc/B_Unified`、`npc/C_Neutral` 的 `[ABC][0-9][0-9].txt` 主檔共 97 份，分別為 36／36／25；其他 `_2` 檔與 `_old` 技能須另列版本，不能混算英雄數。
- [read_hero.lua](https://github.com/damody/nobugg_official/blob/ca6ee8602473f6015a397cc121dac6baeb70b2e5/read_hero.lua) 顯示來源使用 W/E/R/T/D/F 及舊版變體；它用於理解來源，不直接作為本專案解析器執行。
- 原始語言 token、KV 數值、敘述都留 provenance；保留原文，編譯輸入使用正規化結果。
- 已讀 [B01.txt](https://github.com/damody/nobugg_official/blob/ca6ee8602473f6015a397cc121dac6baeb70b2e5/npc/B_Unified/B01.txt) 與 [B02.txt](https://github.com/damody/nobugg_official/blob/ca6ee8602473f6015a397cc121dac6baeb70b2e5/npc/B_Unified/B02.txt)。來源 B02 是風魔小太郎，本地 `B02_date_masamune` 是不同識別，禁止按資料夾編號自動配對。
- 來源中有指向 `scripts/vscripts/...` 的呼叫；本次 tree 查詢未列出該目錄。缺失的原實作不能冒稱已還原。

### 4.2 必須建立的正規化資料

以下是**待實作的契約**，不是目前已存在的 API。可整合進既有 `docs/character_pipeline` 與 Lua schema，避免多一套不相容 package。

| 區塊 | 必填內容 |
|---|---|
| provenance | repository、commit、path、source_hero_id、source_ability_id、variant、語言、原文 hash |
| identity | 穩定 game hero／ability ID、顯示名稱、原 ID 對照、依賴 ID、schema version |
| targeting | self／unit／point／direction、敵我與單位類別、距離與形狀、可見性／無敵／地形政策 |
| timing | 前搖、施放、引導、後搖、可取消階段、命中時間、持續／tick／冷卻開始點 |
| resources | 法力或其他資源、charges、扣費與退款時點、升級條件、各 rank 數值 |
| effects | 傷害種類、倍率／基礎值、治療、buff、疊層、驅散、位移、召喚、觸發順序 |
| lifecycle | 死亡／重生、施法者消失、目標失效、離開視野、重連、換隊、取消、到期 |
| input_slots | 原 W/E/R/T/D/F 到本遊戲 action 的明確映射；被動、切換、額外按鍵及換技能組 |
| presentation | 預覽形狀、動畫狀態、施放／飛行／命中／循環／結束 cues、socket、佔位呈現 |
| bot_usage | 自用／盟友／敵人／方向、射程、保留資源、何時施放與取消；只讀合法觀察 |
| verification | 可驗證情境、期望結果、允許誤差、未支援能力、AI 補值與改編差異 |

規則：

1. 文字與 KV 衝突時保留兩者，記錄採用理由；不把臆測寫成來源事實。
2. 缺數值可採明確的 MVP 預設並標 `assumed`；缺「技能到底做什麼」不能悄悄用普通傷害替換。
3. 新舊版本各有獨立鍵值。預設先做來源當前版；若選舊版，整套 kit 明示版本，不能將新技能數值與舊敘述拼接。
4. 不強制所有角色四個按鍵。MVP 常規 Q/W/E/R＋被動；原生額外技能須有通用 extra action／secondary slot 或明確列為該 kit 的阻塞，不可偷刪。來源 D/F 不自動當召喚師技能。
5. 數值／距離轉換集中處理，包含來源距離、Rust 世界單位與 `WorldUnitsToCm`。圖上預覽與權威判定共用幾何資料。
6. 每個能力狀態分為 `supported`、`needs_framework`、`ambiguous`、`deferred`。只有全部必要機制可用且驗收完成的 kit 才能進入正式可選目錄。
7. 每個生成英雄以來源描述推導測試，再檢視程式。禁止用生成程式的行為反向定義「正確描述」。

### 4.3 英雄 package 與執行步驟

建議 package 內容：`source.lua`、`character.lua`、`abilities.lua`、`capabilities.lua`、`presentation.lua`、`assumptions.md`、`acceptance.md`、生成 Rust 檔與測試。一律擴充既有 `docs/character_pipeline`（已有 `schema/`、`packages/` 與 `tools/validate_package.lua`），正式路徑在 schedule.md E1 對齊現有 schema 後固定；這些名字是提案。

對使用者提供兩種等價輸入：來源 hero ID＋variant，或貼上任意英雄／技能描述。自由文字不能硬要求先有網站 ID。

作者工具只做三個確定性子命令：`index`（來源解析與索引）、`validate`（package 與能力支援檢查）、`build`（生成註冊 → 編譯 → 更新 catalog，失敗保留上一版）。由根目錄 `tools\lua\lua.exe scripts\<tool>.lua` 執行。preview／accept 不另做命令，直接開一局驗證。這些子命令目前尚未實作，執行者不能假裝已可呼叫。工具不需要內嵌 Grok API：coding agent 負責產生程式（含正規化規格與 Rust handler），Lua 工具負責確定性處理、驗證與建置。需接外部模型服務時另做可替換 provider，不把 API key 寫入內容檔。

生成必須：

- 沿 `scripts/gen_hero_registry.lua`、`scripts/base_content`、`omoba-template-ids`、`omfue/codegen` 註冊，ID 穩定且 append-only／保留 tombstone。
- 支援既有通用效果的技能直接生成資料；自訂技能產生 `AbilityScript`／`UnitScript` 實作。不要為證明「Rust 生成」複製一堆相同 handler。
- 產出可玩佔位角色、圖示 fallback、tooltip、瞄準、動畫／cue 映射與選角目錄，不僅產生 `.rs` 檔。
- 未知 API、無效 rank、非有限數、無界召喚／循環、缺少 handler、缺資產都在進入正式目錄前診斷。
- failed build 不更新正在用的 `scripts/base_content.dll` 與嵌入 catalog；保留上一版可用內容。遊戲中的 DLL 不隨意替換，先停止本工作擁有的 session。正式路徑是 `compiled-content-only`，沒有執行期 Lua 熱重載；英雄內容變更一律重新生成、重新建置，經 `scripts/build_ue_moba.lua` 進 UE。
- 新英雄、新技能只能附加在 Lua 宣告尾端，不重排既有條目；server、client runtime、DLL 與 UE bridge 必須來自同一次生成（同一個 content hash）。
- 編譯 host／script 使用同一 Rust 1.95.0。ABI 邊界只用 ABI 友善型別，測試與部署使用同一批來源。
- 生成不執行來自來源敘述的命令；Rust script 是受信任的本機原生程式，不是可安全執行任意第三方程式的 sandbox。

## 5. 技能能力交付矩陣

「已有」表示找到程式／接受入口，仍要驗證本月 kit 的真實玩法與 UE 呈現。

| 能力族 | 基礎狀態與本月工作 | UE 必須呈現 |
|---|---|---|
| 指定傷害／治療、資源、護盾 | 復用既有通用效果；補敵我／可見性／死亡與實際結算測試 | 合法目標、拒絕原因、數字、護盾、結果 |
| 普攻強化、被動、暴擊、吸血 | 已有 hooks 與結算；補自訂觸發條件、事件次數、防遞迴 | 觸發 cue、buff、射程／形態變化 |
| 圓形、扇形、直線、位移 | 已有各類基礎；不得把瞬發直線說成飛行 skillshot | 預覽與命中一致、落點／路徑、取消 |
| 持續引導、DoT、地面區域 | 已有相關實作與增量紀錄；逐 kit 驗證 tick、打斷與來源退休 | 引導條、區域邊界、開始／結束、隱藏後清除 |
| 飛行直線／追蹤投射物 | 普攻投射物已有，通用技能是否足夠要查；補 speed、radius、lifetime、碰撞、穿透／停止與 stable hit order | 飛行、命中、消失；可見性裁切，不能穿霧洩漏 |
| 多段／重施放／蓄力／切換 | 有部分 toggle／transform；按來源挑一個共用有限狀態模型 | 次數、第二段按鈕、剩餘窗口、取消、冷卻 |
| 召喚物 | 已有生成、生命期、歸因、導航；玩家選取／命令與失控處理待完善 | 召喚體、歸屬、有限控制、跟隨／攻擊／召回 |
| 隱形／偵測／控制 | 視野與 stun/root/silence 等有基礎；更多控制按 kit 需求補 | 可見狀態、控制提示、不能選取隱藏敵人 |
| 推拉、擊飛、衝鋒、牆體互動 | 支援情況待逐項確認；列出合法落點、持續時間及免疫規則 | 清楚的位移／受控動作，動畫不反向控制位置 |
| 分身、多形態／多技能組 | 一般變身 buff 不等於完整機制；僅需求矩陣選中者納入本月 | 模型／action slot 變化、歸屬、清理 |
| 複製／偷技能、回溯、任意地形改造 | 完整版 backlog；有來源需求即明示 deferred | 不以普通傷害偽裝支援 |

投射物與複合狀態開發要沿現有 ABI／結算／replica 擴充。任何新增玩法事實都先通過合法可見性投影再傳給 omfue；不為方便顯示把敵方私有資料送到客戶端。

## 6. Unreal Editor 美術接入契約

### 設計者最終操作

1. 在 Editor 的英雄目錄／工具中選擇已生成英雄，按「建立或開啟外觀設定」。
2. 指派 Skeletal Mesh、材質、頭像；選骨架或執行 retarget。
3. 對應 idle、move、attack、cast、channel、death、recall 等動畫。技能專屬可覆寫，共用 fallback 保證沒填完仍可玩。
4. 指派手部／武器／彈體出生 socket、技能預覽／施放／飛行／命中／循環／結束特效及音效。
5. 按 Validate，看到缺槽、錯骨架、缺 socket、無效 asset path、cook 遺漏等具體診斷。
6. 按 Preview／Play，用同一英雄測試移動、普攻、四技能／額外 action、受控、死亡重生與取消。
7. 再跑一次英雄生成，人工素材設定仍保留；打包後外觀相同。

具體工具名稱與 UI 是本月待建。不能要求使用者自己用 Blueprint 寫傷害、投射物命中或技能狀態機。

### 覆寫與資產所有權

- 固定優先序：**人工外觀設定 > 生成的預設設定 > 通用佔位**。
- 生成輸出只寫生成區；人工 profile、Blueprint 子類、素材放獨立受保護位置。既有 `/Game/RustBP` 人工內容保留，不能整批重建。
- 需要新 profile 類型時，建議 `UOmHeroPresentationProfile`／技能 presentation profile（名稱待實作）；綁定以穩定 content ID，不靠顯示名稱或資料夾排序。
- 生成 signature 只涵蓋生成擁有的內容；人工美術變動不能被「自動修復」成預設值。
- 驗證從 Editor 變更、Save、關閉重開、PIE、regenerate、cook、packaged run 的全生命期。
- 外觀欄位：模型、scale／rotation／offset、材質槽、裝備附掛、骨架、動畫／播放速率／段落、socket、圖示、色彩、音效、Niagara／可替代通用特效。
- 玩法碰撞與視覺 mesh 尺寸分開；美術 scale 不能意外改射程、命中半徑或移速。必要的可點選範圍可另設但不能越過權威可見性。
- 特效必須由 cast／projectile／buff instance 追蹤；死亡、取消、隱藏、重連、PIE stop 不留下孤兒 emitter／audio／actor。
- 本月不替使用者大量生成素材；以現有素材或簡單佔位驗證程式與替換流程。

## 7. omfue 遊玩體驗工作

| 系統 | MVP 接受要求 |
|---|---|
| 入口與選角 | 可辨識 MOBA 模式、英雄技能說明、隊伍與 bot；載入失敗可讀，返回不殘留輸入。 |
| HUD | 1280×720 與 1920×1080 核心區不裁切；HP／MP、技能等級／CD、道具、金幣、小地圖與比分清楚。較小視窗有明確布局規則。 |
| 輸入 | 右鍵、A、Shift+A、S/H、升級、自用、取消、聊天、商店 focus 行為一致；實際輸入→request→結果→UI 可追溯。 |
| 地圖 | 道路、基地、塔、野區、草叢、牆、敵我單位可辨識；鏡頭邊界與戰爭迷霧不曝露未知敵人。 |
| 回饋 | 成功、失敗、沒藍、CD、距離、無效目標、死亡／控制造成的拒絕清楚且有界。 |
| 商店／成長 | 搜尋、購買、合成、出售、撤銷、升級、死亡金錢與復活狀態走權威結果。 |
| bot | 能走線、補兵／交戰、購物／成長、回補、推塔至勝負；至少能使用交付英雄的重要主動技能，不要求競技級 AI。 |
| 對局流程 | 斷線可恢復或明確退出；結算是真的，返回／下一局沒有舊 actor、pending command 或效應。 |
| 包裝 | 無測試面板、內部 ABI 字詞常駐；診斷可開關。無 Editor 電腦按說明可啟動交付版本。 |

## 8. 驗收層級與量化門檻

### 四層證據分開記錄

1. **L1**：Lua／Rust／codegen 靜態與單元／整合測試。
2. **L2**：Unreal native／Blueprint 測試、synthetic frame、預覽畫面。證明 renderer 或 UI，不代表真對局。
3. **L3**：真 runtime／authority → omfue 的完整遊玩；真實輸入、權威結果、選角到勝負、重開。分兩種：
   - **L3a**：由 omfue 內的規則式 autopilot 經真輸入管線驅動。單機 2 client＋3 Rust 程序時標 `L3a-loopback`。
   - **L3h**：使用者親自試玩，每個里程碑一局，判斷手感、可讀性與美術。
4. **L4**：雙機 LAN／實際 OS DPI／固定硬體效能／packaged build。Loopback 不能代替 LAN；合成 gamepad 不能代替實體手把。

所有日報標示 fresh、沿用、未執行、blocked。既有 33/44 是原 checklist 的歷史進度，不是遊戲完成百分比，也不能拿本月新增測試數量去沖銷。

### Release gates

| Gate | 通過條件 |
|---|---|
| G0 基準 | 實際入口、版本、dirty diff、既有 11 個缺口、工具可用性、對局授權（不限局數）與未完成工作已記錄。 |
| G1 英雄薄切片 | 一套來源 kit 通過描述→script→選角→UE 施法；一筆技能可追到權威效果與畫面。 |
| G2 可玩基線 | 1 人類席（autopilot，L3a）＋9 bot 完整三路對局可自然結束；重開一局可用。autopilot `coverage.json` 證明所有核心操作在真對局驗到；另有一局 L3h。 |
| G3 製作能力 | 至少 4 套（目標 6 套）來源 kit 完整；各 kit 的來源映射、全部必要技能、rank、死亡／取消／視野與呈現通過。 |
| G4 美術交付 | 至少兩種不同骨架／外觀設定完成 Editor-only 替換，regenerate／重開／cook 保留，無玩法程式修改。 |
| G5 泛化能力 | 2 位保留盲測英雄從原始描述產生；至少一位需要自訂 Rust handler。已支援能力不得改共用引擎／UE 玩法程式；不支援者如實計入能力缺口。 |
| G6 發佈 | 單機 B＋C＋C（L3a-loopback）與真雙機連線、斷線恢復、DPI／視窗矩陣、固定硬體效能、乾淨包啟動、手冊及已知問題完成。缺任一必需外部證據，標候選版而非完整 MVP 驗收通過。 |

預設效能目標在 schedule.md A3 寫入硬體／解析度／畫質清單：1080p、10 英雄、正式常態兵線與野區，暖機後連續 20 分鐘，UE p95 frame time ≤ 16.7ms、p99 ≤ 33.3ms；60Hz 模擬步進 p99 ≤ 16.7ms。另測一段 10 英雄集中施法。若基準硬體不合理，在 A3 記錄並固定可達目標，不在最後偷偷換硬體或降低負載。

長時間接受：同一 session 連續 2 小時無 crash／deadlock，重複能力與死亡不造成持續資源增長；暖機後最後 30 分鐘相對前一段 30 分鐘記憶體峰值增長目標 <10%，並報 actor／VFX／audio pool 峰值與上限。不能以重啟 session 分段相加冒稱完成。

網路矩陣：本機雙 client、真雙機 LAN、至少 80ms RTT／20ms jitter／1% loss 的已定義 netem profile；斷線 10 秒重連後不重複扣費／命中／購物、不顯示隱藏敵人、輸入恢復。數字是本計畫目標，不是現況量測。

英雄測試：每個技能每個 rank 至少一個正常情境，外加目標失效、資源不足、冷卻、取消／控制、死亡／重生、重複／過期事件、visibility；有特殊機制再加牆角、穿透、多目標順序、疊層或召喚 owner 退役。不要為純文字或排版修改新增鏡像測試。

## 9. 執行與資源管理

- 每批工作以一個可展示的垂直功能為單位；修復沒有明確玩家效益的邊角問題前，先看它是否阻擋下一個里程碑的 gate。
- 每個 task 須有 ID、相依、狀態、變更檔、命令、結果、證據、下一步。狀態採 `todo / doing / blocked / accepted / deferred`，一次只保留一個主要 doing。
- 限制失敗重試：同一原因連續 3 次先寫根因與替代路徑，避免重跑完整套件碰運氣。與改動無關的未變測試可沿用，但需明記來源一致性。
- 依 schedule.md §2 的里程碑 M1–M4 做整合；M4 之後只修 release blocker，不要把整個月排滿新功能。
- 人工／硬體依賴提早提出：在 schedule.md A6 確認 Editor 操作時段、build tools、第二台機器、OS DPI 切換、可用素材；尚未取得時先做可獨立驗證的工作。第二台電腦到位前，多人驗收一律在單機以 2 client＋3 Rust 程序進行（test-topology.md），到位後只移動一個 client。
- 使用者 2026-10-25 授權：本機對局不限局數，舊 `10/10` 對局預算不再適用於本計畫。需要真對局證據時直接開局，不必等待批准；若程式碼或工具中仍有寫死的局數上限擋住 L3/L4，可以移除或改成可設定，並在 execution-status.md 記錄改了哪裡。仍須遵守：只停止本次自己啟動的程序、每局獨立輸出目錄、失敗時保留證據；同一時間只跑一組單機拓撲，避免互相搶資源污染效能數據。
- 使用 `tools\lua\lua.exe` 與現有 Lua host。不要新增 PowerShell／Python／shell 工作流，不增加根目錄 bat。
- 只清理自己擁有的程序與輸出；保留現有 dirty work。本輪文件建立時主 repo、omb、omfue 都有未提交變更，不能 reset 或整批 commit。
- 未有交付政策時不擅自 push／發布。若獲授權提交，先處理變動 submodule 的 commit/push，再更新主 repo gitlink；建置輸出不進版控。

## 10. 月末交付檔案

- `release-notes.md`：實際完成內容、G0–G6 結果、未解問題及限制。
- `player-guide.md`：從啟動到對局與再開一局的操作。
- `hero-authoring-guide.md`：提供描述到產出英雄；含一個完整範例與失敗診斷。
- `unreal-art-guide.md`：使用者只在 Editor 做美術的圖文步驟與槽位規格。
- `source-catalog.lua`／等價既有格式：全來源目錄與版本、能力分類；本月交付 kit 與 2 位盲測的驗收對照。
- `acceptance-index.md`：各 gate 的證據路徑、build identity、fresh／沿用與未執行項目。
- `next-month.md`：依實際能力缺口排序的後續英雄、多人產品服務與進階技能 backlog。

上述檔名為月末要求，尚未建立。時間不足時先保住可玩與工具閉環，公開調整未交付英雄／進階能力，不能用空殼英雄或改口定義驗收來湊數。
