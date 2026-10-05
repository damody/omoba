# Jungle共用清野焦點

## 計畫與實作

1. 核對普攻／技能中立目標：單體與neutral施法本已支援，修正最初低HP排序假設。实际需要統一的是範圍中心與近處camp的戰鬥焦點。
2. jungle_farm_target純函式供普攻decide與role_combat_focus共用，current披露kind3／team0／alive且距本人550內，distance／canonical ID穩定選取；不看camp私有HP／respawn／aggro。
3. 既有可見隊友assist焦點優先，沒有assist才選farm。正常EnemyPoint的focus／cluster排序將可行近camp優先；min_targets不足或焦點不在技能射程，仍依原規則選其他合法中心，不把focus當授權。
4. 保留單體／point／dash原range、本人Mana、CD、作者政策順序及正式PlayerInput／script結算，不新增Lua內容、英雄C++或Blueprint graph。

## 當前確認

- core新1首輪passed：近camp與far cluster選點、最低命中數fallback、hidden／dead／friendly／非neutral拒絕。
- base正式60Hz新1首輪passed：training_ranger正常generated handler在near camp施放ranger_volley，near HP下降／CD開始，far與第三隻neutral未受傷；attack-only同near目標、Hidden index移除後不能從cache重建。
- shared helper最後確認：上述兩項再次passed；既有jungle_assist_60hz指定1亦passed，助戰優先與披露邊界保持。

第三neutral為正常ECS元件fixture，不在camp reward／respawn帳本，因此本批不宣稱其營地生命週期完成。未全套／100場／UE／stage。C ABI13／IPC4／content hash不變，OpenSpec20/30及完整5.5維持待完成。錯誤與更正記E207。
