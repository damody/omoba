# Bot 通用技能學習（2026-10-05）

## 計畫、問題與決策

1. rank 0 出生的英雄不能只靠施法政策參戰；新增独立 `ability_learning`，Lua 用有序 `{ability,rank}` 宣告每次投入的一點。省略時預設空陣列，舊移動／施法配方行為不變。
2. 明確驗證最多64步、compiled技能存在、rank从1連續递增、不可重複／跳級／超過max_level或level表。可交錯不同技能，不限制四個技能ID或特定英雄；不存在當前owner loadout的項目跳過，支持混合英雄配方。
3. 無技能點不提出升級；每個think只選一個合法未完成步驟。以當前owner loadout找到正式0..3槽位，再檢查owner當前rank與compiled下一級required_hero_level。尚未達門檻跳過，後續合法步驟可先學，達門檻後自動回補。沒有固定QWER／英雄ID分支。
4. 升級優先於施法，走正式PlayerInput::UpgradeAbility；不直接更改rank、skill_points或command queue。既有權威handler重新檢查owner／Playing／存活／等級／點數／上限，扣點並發SkillLearn。planner自己不扣點，也不繞過原有腳本初始化。
5. 保留原有pause／phase／recall／committed owner disclosure／freshness gate。不讀敵方等級或技能資料，學習政策與施法意圖互不推導。異常i32::MAX rank使用checked_add避免panic。
6. 範例Lua配方按rank交錯四技能，共16步；舊英雄出生已學的rank自動跳過。本批rank0端到端以真實training_apprentice驗證，不修改其出生設定。

## 當前功能確認

- core role_bot_learning 2/2：schema、catalog、rank順序／上限／重複／64步界線、無點數／滿級／缺loadout／異常rank、compiled等級門檻與實際槽位重排。
- core role_bot_plan 2/2：新欄位從plan轉runtime config，非法學習配方在compile拒絕，既有真人＋九Bot命名lane／控制配方檢查仍通過。
- base_content role_bot_learning_rank_zero_through_formal_60hz_upgrade_and_cast 1/1：實際rank0＋一點出生，先跳過level6的lance，槽位0→3後正式學bolt；planner不扣點，authority扣1且rank=1；後續正式slot3施法精確80傷害。以明確level6／一點fixture確認先前blocked lance走正式slot2學習／扣點，非XP獎勵測試。
- 固定Lua run_moba_headless --role-plan-lua --plan-only：exit0，10 players／10bots／16學習步，保存 `omb/target/moba-headless/role-learning-plan-check.json`。這只確認Lua→嚴格Rust配置入口，不是完整對局。
- 無DLL stage、Unreal重建／Editor／MCP／LAN或100場完整验收。保持20/30，5.5不勾選。

## 正式server接線查核（尚未實作）

`omb/src/state/core.rs` 正式State::tick已有 accumulated→MobaMatch input gate→canonical accepted projection→PendingPlayerInputs流程；與headless不同。canonical accepted inputs目前从CONFIG.AUTHENTICATED_TEAM_BINDINGS找team，而main也把同一表交給KCP authorize_player_team。

決策：接入Bot不可為了讓projection有team就把Bot全加進真人認證表。需要分開match roster／Bot控制權與外部可認證真人，拒絕外部控制Bot，且server生成Bot輸入仍經正式authority dispatch與安全accepted projection。尚未修改認證或server tick，不宣稱已有一真人九Bot KCP模式。下一步應一次完成這個控制權邊界，而不是直接從Bot函式改ECS或只接PendingPlayerInputs漏掉projection。

## 防錯

本批PowerShell曾使用shell brace expansion `{tasks,design}.md`，解析失敗；改成兩個明確檔案路徑。數次合併讀取輸出截斷，後以小範圍逐段確認相關控制權與projection實作，不能將截斷內容視為完整讀取。詳見E162。
