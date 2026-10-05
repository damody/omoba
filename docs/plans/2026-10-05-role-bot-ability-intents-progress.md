# Bot 通用技能意圖（2026-10-05）

## 計畫、問題與決策

1. 查核發現 AbilityDef.effects_preview 明確只是 tooltip 預覽，不代表 handler 執行語義。禁止依 preview／英雄名稱／槽位猜傷害或治療。
2. 新增共用 BotAbilityPolicy／BotAbilityIntent，Lua對局配方按明確 ability ID 宣告 EnemyUnit 或 SelfHeal(below_hp_per_mille)，宣告順序就是priority。RoleBotMatchPlan 的 ability_policies 可省略，預設空陣列保持既有移動／普攻配置相容。
3. schema／compile／role config 共同驗證最多64政策、ID唯一、active compiled技能、Active或Ultimate／Instant、Unit或None目標型別與1..1000健康門檻。Toggle／Passive／Channeled／Point未支援，拒絕而非假裝會用。
4. 施放從當前owner Hero loadout找slot（僅正式0..3）、rank與cooldown，再用compiled每級Fixed64 range檢查可見且正HP的敵方單位。不讀敵方ECS位置、HP、cooldown、input、記憶ghost；目標完整canonical generation驗過後才取ECS index。
5. 自療只讀owner健康，以i128整數比較門檻，不用浮點比例。施法提交正式PlayerInput::CastAbility，後续cast gate／script handler／效果結算與冷卻都沿原權威流程；政策不直接改HP／CD。
6. Lua範例配方明確配置四個lumen技能意圖，policy IDs只在內容，不在通用Rust決策新增英雄／技能分支。未知技能／不在loadout／未學／冷卻／超距離跳過。
7. Mana目前沒有完整權威current-mana契約；本批不假算mana，也不把metadata mana_cost當成資源驗證成功。explicit intent是作者AI控制設定，不保證任意自訂handler一定產生政策名稱所述效果，實際判定仍在handler。

## 功能確認

- core role_bot_abilities：2/2 passed，驗政策ID／target mismatch／門檻、loadout重排後slot、rank0拒絕、冷卻拒絕、590在600距離內與601超距離、自療低HP與滿HP。
- base_content role_bot_abilities_cast_damage_and_heal_through_formal_60hz_pipeline：最後1/1 passed。將lumen_bolt由0換到2，正式Bot輸入精確target與slot2、實際80傷害／正CD；單獨移除敵人disclosure（保留owner）不施法；lumen_touch正式slot1自療100→170／正CD。
- 真實固定Lua配方→headless --plan-only：exit0／10 players、10bots；新政策通過compiled配置檢查。這是配置入口，不是對局終局驗收。
- 首次core編譯E0382（移走health後仍借用），改測試clone；ECS第一次與移除TAttack後第二次皆失敗（總傷害169.121而非80），定位MOBA私有NPC塔攻擊後還原TAttack、改位置fixture離塔／camp範圍，最後通過。詳見E161，不藏失敗、不修改實際傷害值迎合測試。
- 既有dead-code／protoc fallback warnings保留。無DLL stage、Unreal／MCP／網路施法、100場或完整驗收。

## 還需完成

學習／升級policy、Point／位移／Buff／Toggle／Channeled等技能、完整mana與conditions契約、三種完整英雄原型、KCP Bot控制身分與正式admission／安全投影接線、完整100場及Unreal／LAN最後驗收。

5.5仍未勾選，20/30不變。
