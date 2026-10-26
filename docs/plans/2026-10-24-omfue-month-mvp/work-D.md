# 工作紀錄：D 包（來源索引與選角草案）

格式依 schedule.md §5。本紀錄由支線 session 寫，不修改 `execution-status.md`；主 session 合併時再抄入。

```text
日期：2026-10-25
目前工作 ID／狀態：D1 accepted；D2 accepted；D3／D4 候選草案已出（todo，待逐招人工判定與定案）
玩家或作者新增能做到的事：
  作者可用 `tools\lua\lua.exe scripts\hero_source_index.lua index --source <checkout>` 從固定 commit 的來源產生全量索引，
  用 `verify` 子命令把每一筆 ability／unit／tooltip 回讀原檔比對原 token 與 hash；可依 README §5 能力族篩候選。
repo HEAD／本次修改：
  主 repo HEAD b1f00b9（未 commit；本次只新增下列檔案，另在 scripts/test_lua_tooling.lua 尾端加一段測試掛點）。
  新增：scripts/hero_source_index.lua
        scripts/hero_source/{kv,sha256,serialize,classify,catalog,summary}.lua
        scripts/tests/hero_source_index_test.lua
        docs/plans/2026-10-24-omfue-month-mvp/source-index/{source-catalog.lua,summary.md,candidates-draft.md}
        docs/plans/2026-10-24-omfue-month-mvp/work-D.md
  修改：scripts/test_lua_tooling.lua（只在 xpcall 結尾加 hero_source_index_test 的 process.run；
        該檔已有其他 session 未提交的 maintained_source_deps 段落，未動它）
來源英雄與 variant：
  https://github.com/damody/nobugg_official @ ca6ee8602473f6015a397cc121dac6baeb70b2e5，
  以 `git fetch --depth 1 <commit>` 取到 target/nobugg_src（已確認被 .gitignore `/target` 忽略）。
  只以文字讀取；read_hero.lua、langtext.lua、vscripts 都沒有執行（langtext.lua 只用文字比對抽 zhtw_heroname）。
  全量：97 主檔（A_Oda 36／B_Unified 36／C_Neutral 25，與預期一致）、97 個 `_2`、addon_tchinese2.txt、langtext.lua，共 196 檔；
  966 個 ability 塊（skill 843、sub_ability 14、auxiliary 12、precache 97；variant current 545／old 369／_20 52）。
已執行命令、exit code、build identity：
  tools\lua\lua.exe scripts\hero_source_index.lua index --source target\nobugg_src   → 0；2789 spans 自驗通過；
      source-catalog.lua sha256 db59e79236e88cf77322085da27924440ea1e5007065466f5bc2b54275dffffb（連跑 3 次相同）
  tools\lua\lua.exe scripts\hero_source_index.lua verify --source target\nobugg_src  → 0（2789 spans、196 檔 hash）
  tools\lua\lua.exe scripts\tests\hero_source_index_test.lua                       → 0（13 tests）
  tools\lua\lua.exe scripts\test_lua_tooling.lua                                    → 0（含上項）
L1／L2／L3／L4 證據（fresh／沿用／未執行）：L1 fresh（上列 Lua 測試）；L2／L3／L4 未執行（純工具與資料，不涉及 UE 或對局）
gate 與原 11 主項的變化：無 gate 變化；D1／D2 完成解除 E1 的前置（D1）。原 11 主項不受影響。
失敗、原因、尚未修的風險：
  - 來源異常（照實索引，未修改來源）：A13.txt 根 key 是 DOTAHeroes 但內容是技能；C19_2.txt 根 key 拼成 DOTAHeros；
    B18_2.txt 第 31 行有孤立的 "0"（缺 key），解析器以 orphan_token 診斷並按行恢復，避免後續欄位錯位；
    addon_tchinese2.txt 有 70 個重複 token，取第一次出現者。
  - `_2` 檔不是第二套技能組，而是 DOTAHeroes 單位定義；英雄數只算主檔。`_20` 變體 52 招，來源未說明含義，只照實標記。
  - 757／857 招含 RunScript 指向來源缺少的 scripts/vscripts/...；實際行為只能以描述重建，不能冒稱已還原。
  - D2 粗分類：技能 857 招中 unclassified 19（current 13），名單在 summary.md。描述關鍵字只是弱證據，D3 必須逐招閱讀。
  - ID 對照：B01 name_and_number_match → saika_magoichi；來源 B02（風魔小太郎）與本地 B02_date_masamune 為 conflict_number_only；
    來源 B04（伊達政宗）→ 本地 date_masamune 為 name_match_number_differs。其餘 94 位 unmapped。
  - B01 版本核對：本地 kit 是混合改編，不等於來源 current 或 _old（細節見 candidates-draft.md）。H01 需在 D4／E3 決定用哪個版本。
  - source-catalog.lua 約 7.5 MB（每筆含原文 raw 以滿足「回到原 token」）。若之後要進版控嫌大，可改成只存 offset＋hash，由 verify 回讀。
下一個工作 ID：D3（對 candidates-draft.md 的 6＋2＋4 位逐招標 supported／needs_framework／ambiguous／deferred 與 slot 映射）；E1 可平行開始
人工／硬體需求與是否已取得：無
owned process／資源清理結果：沒有啟動長駐程序；測試 fixture（target/hero-source-tests/...）在測試結束時自行刪除。
  target/nobugg_src 保留供 D3 與 verify 使用（被忽略、不進版控）。
```

```text
日期：2026-10-25
目前工作 ID／狀態：D3 accepted；D4 accepted
玩家或作者新增能做到的事：
  每位候選英雄有逐招能力需求表、W/E/R/T/D/F → Q/W/E/R/passive/extra 映射、bot_usage 草案與所需 G 包能力；
  `tools\lua\lua.exe scripts\hero_source_index.lua requirements` 驗證資料並生成 d3-requirements.md：
  狀態由機制推導（不能宣稱 supported 而機制不是）、每招釘住閱讀時的 raw_sha256（來源變了即判 stale）、
  未映射的來源技能、重複 action、缺 bot_usage（盲測禁止寫）都會失敗。
repo HEAD／本次修改：b1f00b9（未 commit）。
  新增 scripts/hero_source/requirements.lua；hero_source_index.lua 加 requirements 子命令；測試加 1 項（共 14）。
  新增 source-index/{candidate-requirements.lua, d3-requirements.md, roster.md}；candidates-draft.md 加「已被取代」註記。
  錯誤紀錄改寫進共用 errors.md（E004–E009），刪除原本的 errors-D.md（本 session 自建）。
來源英雄與 variant：12 位全用 current；53 招全部讀過（KV 欄位、AbilitySpecial、modifier、script 引用、繁中描述）。
已執行命令、exit code、build identity：
  tools\lua\lua.exe scripts\hero_source_index.lua requirements → 0
  tools\lua\lua.exe scripts\tests\hero_source_index_test.lua   → 0（14 tests）
L1／L2／L3／L4 證據（fresh／沿用／未執行）：L1 fresh；其他未執行（純資料與工具）。
gate 與原 11 主項的變化：無；D4 完成解除 E3 的 D4 前置。
失敗、原因、尚未修的風險：
  - 能力登錄依引擎調查（script-abi、base_content、omoba-core/src/runtime/native、omoba-content-model）：
    AbilityScript::on_tick 沒有 host 呼叫；英雄沒有註冊 UnitScript；直線投射物只命中 Creep；MOBA loadout 上限 4 技能。
    這四點是多數 needs_framework 的根因，已寫在 candidate-requirements.lua 的 capabilities。
  - 12 位全部 needs_framework；A13 為 deferred（illusion、curving_hook）→ H06 換成 B08（roster.md 換人紀錄）。
  - 兩個覆寫：C17T 的 1 秒延遲以施法前搖表達；B01W 夜晚視野在無日夜循環的遊戲中不適用（記為 assumption）。
  - 後續修正（同日，E2 設計時發現，errors.md E014）：本遊戲 loadout 上限 4 格且被動也佔一格，
    原先把 R／D 被動映射為不佔欄位的 passive。已改為 W/E/R/T → Q/W/E/R、D/F → extra1～3，
    extra 必須有 extra_action_slot 機制；C17（H04）與 A04（候補）因此新增 G-3 extra_action_slot 需求，roster.md 已同步。
下一個工作 ID：E1（character_pipeline schema＋H01 package）→ E2
人工／硬體需求與是否已取得：無
owned process／資源清理結果：無長駐程序。
```

## 給後續工作的索引用法

- 重建：`tools\lua\lua.exe scripts\hero_source_index.lua index --source target\nobugg_src`（commit 不符會直接失敗）。
- 回驗：`tools\lua\lua.exe scripts\hero_source_index.lua verify --source target\nobugg_src`。
- 讀取：`load(text, name, "t", {})` 載入 `source-catalog.lua`（純資料，不需要任何環境）。主要欄位：
  `files[]`（path、sha256、bom、root_key、variant）、`heroes[]`（source_hero_id、name_zhtw、listed_on_site、unit、ability_list）、
  `abilities[]`（id、source_hero_id、slot、variant、kind、path、line、start／stop、key_token、raw、raw_sha256、fields、specials 含 ranks、
  behavior_flags、script_refs、tooltip、classification{status, basis, families, tags[evidence]}）、`id_mapping`、`diagnostics`。
- 能力族 id 與 README §5 對照在 `scripts/hero_source/classify.lua` 的 `FAMILIES`。
