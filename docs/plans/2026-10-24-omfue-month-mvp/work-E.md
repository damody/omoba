# 工作紀錄：E 包（英雄 package 與作者工具）

格式依 schedule.md §5。本紀錄由支線 session 寫，不修改 `execution-status.md`；主 session 合併時再抄入。

```text
日期：2026-10-25
目前工作 ID／狀態：E1 accepted；E2 accepted（build 只驗證指令組合與回滾，未對真實樹執行）
玩家或作者新增能做到的事：
  - 作者可寫 schema v2 英雄 package（玩法契約，README §4.2），用 validate_package.lua 或 hero_author.lua validate 檢查。
  - `tools\lua\lua.exe scripts\hero_author.lua index|validate|build`：
    index    = hero_source_index（來源索引）
    validate = schema v2＋來源 catalog（commit／variant／raw_sha256）＋D3 能力狀態＋缺 handler 診斷，
               並在 scratch 複本上生成 authored 模板，再跑 gen_hero_registry（registry、ids）。
    build    = 同上檢查，再依 kit 狀態把關（非 supported 必須用 --slice 只建 supported 技能），
               沒有 --execute 時只列計畫；--execute 寫入 templates/authored_heroes.lua 與 include hook，
               接著跑 gen_hero_registry ×2 → build_ue_moba.lua → build_ue_moba.lua --verify-staged-only →
               test_compiled_content_contract.lua。任一步失敗就把內容檔、built 與 staged DLL 全部還原。
repo HEAD／本次修改：b1f00b9（未 commit）。
  新增：docs/character_pipeline/tools/package_v2.lua
        docs/character_pipeline/packages/saika_magoichi/{character,source,abilities,capabilities,presentation}.lua, assumptions.md, acceptance.md
        scripts/hero_author.lua、scripts/hero_author/{cli,generate,install,build}.lua
        scripts/tests/{character_package_test,hero_author_test}.lua
  修改：docs/character_pipeline/tools/validate_package.lua（依 schema_version 分派；可當模組；新增 --catalog／--requirements；v1 行為不變，仍 CRLF）
        docs/character_pipeline/schema/README.md（加 Schema v2 章節）
        scripts/hero_source/serialize.lua（有洞的整數 key 表不再被當陣列，見 errors.md E015）
        scripts/test_lua_tooling.lua（尾端加 character_package_test、hero_author_test 掛點）
  scripts/lua_data/：未修改。hook 與 authored_heroes.lua 只有在 `build --execute` 時才會附加；測試在 fixture 複本上進行。
來源英雄與 variant：H01＝來源 B01 current（使用者決定）；package 釘住 B01W／E／R／T 的 raw_sha256 與 tooltip hash。
已執行命令、exit code、build identity：
  tools\lua\lua.exe docs\character_pipeline\tools\validate_package.lua docs\character_pipeline\packages\saika_magoichi --catalog …\source-catalog.lua --requirements …\candidate-requirements.lua → 0
  tools\lua\lua.exe docs\character_pipeline\tools\validate_package.lua docs\character_pipeline\packages\hero_example（及 hero_smoke_astral_ranger）→ 0（v1 不變）
  tools\lua\lua.exe scripts\hero_author.lua validate docs\character_pipeline\packages\saika_magoichi → 0（kit needs_framework；列出 4 個缺 handler）
  tools\lua\lua.exe scripts\hero_author.lua build  …\saika_magoichi → 1（kit needs_framework，要求 --slice）
  tools\lua\lua.exe scripts\hero_author.lua build  …\saika_magoichi --slice b01_sniper_mode → 1（b01_sniper_mode_ffi 尚未匯出）
  tools\lua\lua.exe scripts\tests\character_package_test.lua → 0（7）
  tools\lua\lua.exe scripts\tests\hero_author_test.lua      → 0（6）
  tools\lua\lua.exe scripts\test_lua_tooling.lua            → 0
  未執行：cargo、build_ue_moba.lua、DLL staging、UE（依指示，E3 由主 session 執行）。
L1／L2／L3／L4 證據（fresh／沿用／未執行）：L1 fresh；L2／L3／L4 未執行。
gate 與原 11 主項的變化：無 gate 變化；E1、E2 完成後 E3 的工具前置齊備（E3 另需 D4，已完成）。
失敗、原因、尚未修的風險：
  - H01 kit 是 needs_framework：b01_saika_corps 需要 G-2 charges 與 G-3 定點召喚。E3 薄切片可用
    `build --slice b01_sniper_mode[,b01_rain_iron_cannon,b01_three_stage] --execute`，前提是先在
    scripts/base_content/src/heroes/B01_saika_magoichi/ 寫好並匯出 `b01_*_ffi`（package implementation.module 已列名稱）。
  - 回滾範圍：內容檔（authored_heroes.lua、heroes.lua、abilities.lua）與 moba_stage_contract 的 built＋staged DLL
    （base_content、om_bridge）。OmGame C++ 模組 binary 不在快照內；catalog 身分以 bridge／DLL 為準。
  - build 的 gen_hero_registry 會在寫入後對真實內容再跑一次（與 validate 的 scratch 檢查重複，刻意保留以在寫入後確認）。
  - omoba-content-model 的 Rust 驗證在 build_ue_moba（cargo build）中執行；validate 只做 Lua 端相同數值範圍與 gen_hero_registry 檢查，不跑 cargo。
  - errors.md 同時被其他 session 追加；本 session 的新紀錄改號為 E014–E016，避免與 A3 的 E012／E013 衝突。
下一個工作 ID：E3（主 session：寫 B01 handler → build --slice --execute → UE 施法）；本 session 可接 E4 或 G 包需求整理
人工／硬體需求與是否已取得：無
owned process／資源清理結果：無長駐程序；fixture 與 scratch（target/character-package-tests、target/hero-author-tests、
  target/hero-author-validate）在測試／validate 結束時刪除。
```

## E3 使用方式（給主 session）

1. 在 `scripts/base_content/src/heroes/B01_saika_magoichi/` 新增 handler，並在 `mod.rs` 匯出：
   - `b01_sniper_mode_ffi`
   - `b01_rain_iron_cannon_ffi`
   - `b01_three_stage_ffi`

   檔名見各技能 `implementation.module`。數值從 `extras` 讀取，key 由 generator 產生：
   - 狙擊模式：`stat_buff_duration`、`stat_buff_attack_damage_bonus`、`stat_buff_attack_range_bonus`
   - 雨鐵炮：`on_attack_bonus_by_target_class_building`、`_hero`、`_creep`
   - 三段：`stat_buff_base_damage_outgoing_pct`
2. 先只列計畫：`tools\lua\lua.exe scripts\hero_author.lua build docs\character_pipeline\packages\saika_magoichi --slice b01_sniper_mode`。
3. 確認後加 `--execute`。成功後英雄 `saika_magoichi` 的 Q 欄改為 `b01_sniper_mode`，其他欄維持原技能。新技能附加在模板尾端，舊技能 ID 不變。
