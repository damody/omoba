-- B09 真田昌幸（current 版，heromap_skill B09 ["16"] = "WERT"）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/B_Unified/B09_2.txt:12-17）。current 版用 B09W/E/R/T，*_old 是 11 版。
-- heromap_autoskill B09 ["16"] = ""，沒有出生自動學的技能。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B09_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"；Lua modifier 預設不可驅散，寫 "none"。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "B09",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "B09",
      text = { zh_TW = { name = "真田昌幸" }, en = { name = "Sanada Masayuki" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b09_backlash_rift", source_id = "B09W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 26, 22, 18, 14, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 145, 170, 195, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          stack_damage = { 7, 10, 13, 16, unit = "dmg", damage_type = "pure", source = "kv:AbilityDamage" },
          fixed_damage = { 50, unit = "dmg", damage_type = "pure", kv = { 20, 30, 40, 50 },
            source = "script:vscripts/heroes/B_Unified/B09.lua:18 GetAbilityDamage() × 次數 + 50",
            source_differs = "KV basicDMG 20/30/40/50 與原作說明沒有被腳本使用，固定傷害寫死 50" },
          tick = { 1, unit = "s", source = "script:vscripts/heroes/B_Unified/B09.lua:16-33 Timer 0、return 1" },
          base_ticks = { 8, unit = "count", source = "script:vscripts/heroes/B_Unified/B09.lua:9 target.max_count = 8" },
          max_count = { 20, unit = "count", source = "kv:max_count" },
          rift_duration = { 20, unit = "s", hidden = true, source = "kv:modifier_B09W duration" },
        },
        text = {
          zh_TW = { name = "反噬裂隙",
            body = "在目標敵方英雄身上撕開裂隙：立即並之後每 {tick} 秒造成 {fixed_damage} 加上（已觸發次數 × {stack_damage}）的純粹傷害，基本觸發 {base_ticks} 次。目標每施放一個非切換技能或使用道具，觸發次數加一，最多 {max_count} 次。裂隙標記被驅散時提前結束（目標魔免時則重新標記並暫停傷害）。" },
          en = { name = "Backlash Rift",
            body = "Tears a rift on the target enemy hero: immediately and every {tick} seconds it deals {fixed_damage} plus ({stack_damage} × hits so far) pure damage, {base_ticks} times by default. Each non-toggle ability or item the target uses adds one more hit, up to {max_count}. Dispelling the rift mark ends it early (if the target is spell immune, the mark is reapplied and the damage pauses)." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b09w_rift", duration = "$rift_duration", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            vars = { max_ticks = { type = "int", init = "$base_ticks", reveal = "public" } },
            triggers = {
              on_ability_executed = { who = "self", filter = "not event.ability.is_toggle", actions = {
                { op = "if", cond = "vars.max_ticks < $max_count", actions = {
                  { op = "set_var", var = "max_ticks", value = "vars.max_ticks + 1" },
                  { op = "set_display_stacks", status = "b09w_counter", to = "self", count = "vars.max_ticks" },
                } },
              } },
            },
            notes = "原作 modifier_B09W（IsHidden、duration 20）只負責 OnAbilityExecuted；觸發上限存在目標的 Lua 欄位 target.max_count，這裡放進狀態 vars。" },
          { id = "b09w_counter", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "b09w_rift_glow",
            text = { zh_TW = { name = "反噬裂隙", body = "每 {tick} 秒受到純粹傷害，傷害隨觸發次數增加；每施放一個非切換技能或使用道具，觸發次數加一。被驅散時提前結束。" }, en = { name = "Backlash Rift", body = "Takes pure damage every {tick} seconds, growing with each hit; every non-toggle ability or item used adds one more hit. Ends early if dispelled." } },
            notes = "原作 modifier_B09W_counter：沒有 Duration、沒寫 IsPurgable（暫定 basic）；層數只顯示觸發上限。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b09w_cast_sound", at = "caster" },
            { op = "let", name = "tgt", value = "target" },
            { op = "apply_status", status = "b09w_rift", to = "tgt" },
            { op = "apply_status", status = "b09w_counter", to = "tgt" },
            { op = "repeat", interval = "$tick", first = "0", count = "$max_count", as = "k",
              stop_when = "not has_status(tgt, b09w_rift)", actions = {
                { op = "if", cond = "has_status(tgt, b09w_counter) or tgt.is_magic_immune",
                  actions = {
                    { op = "if", cond = "has_status(tgt, b09w_counter)",
                      actions = {
                        { op = "damage", to = "tgt", type = "pure", amount = "$stack_damage * k + $fixed_damage" },
                      },
                      otherwise = {
                        { op = "apply_status", status = "b09w_counter", to = "tgt", duration = "$max_count - k" },
                      } },
                    { op = "if", cond = "k + 1 >= status_var(tgt, b09w_rift, max_ticks)", actions = {
                      { op = "remove_status", from = "tgt", status = "b09w_rift" },
                      { op = "remove_status", from = "tgt", status = "b09w_counter" },
                    } },
                  },
                  otherwise = {
                    { op = "remove_status", from = "tgt", status = "b09w_rift" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "vscripts/heroes/B_Unified/B09.lua:18",
            text = "原作說明與 KV basicDMG 的固定傷害是 20/30/40/50，腳本寫死 50（等於四級值），一到三級固定傷害偏高。資料照實際寫 50，是否改用 basicDMG 由使用者決定。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B09.lua:16-33",
            text = "第一跳在施放當下（Timer 0，次數 0 只有固定傷害），之後每秒一跳，第 n 跳傷害 = 固定 + (n−1) × AbilityDamage；預設八跳。傷害以 AMHC:Damage 造成（施法者需存活），目標死亡後狀態消失即停止。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B09.lua:17-23",
            text = "每跳只檢查顯示用的 modifier_B09W_counter：被驅散就停止；若當下目標魔免，改為以 20 − 次數 的時間重新套上 counter，這一跳不造成傷害但仍計次。純粹傷害本身會穿透魔免。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B09.lua:36-52",
            text = "目標的技能與道具（切換技能除外）每用一次把上限加一，最多二十；同時把隱藏 modifier 延長一秒，但結束由計數決定，延長沒有實際影響。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B09.lua:9",
            text = "觸發上限 max_count 存在目標單位上：兩位昌幸先後對同一目標施放時，第二次會把上限重設為八並同時影響第一道裂隙；v3 各自存在自己的狀態 vars。" },
          { policy = "note",
            text = "b09w_rift 寫 dispel = none，代表原作存在單位欄位上的計數不受驅散；原作的隱藏 modifier_B09W 若被驅散，之後施法不再加次數，這個差異未實測。" },
        },
        provenance = { "npc/B_Unified/B09.txt:24-111", "vscripts/heroes/B_Unified/B09.lua:3-52" },
      },
      -------------------------------------------------------------------------
      { id = "b09_mind_burst", source_id = "B09E", slot = "W", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 7, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          channel_time = { 10, unit = "s", source = "kv:AbilityChannelTime" },
          tick = { 0.5, unit = "s", source = "kv:modifier_B09E ThinkInterval" },
          radius = { 400, unit = "dist", source = "script:vscripts/heroes/B_Unified/B09.lua:79 FindUnitsInRadius 400（同 KV AOERadius）" },
          damage = { 50, 80, 110, 140, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          mana_pct = { 5, unit = "pct", source = "script:vscripts/heroes/B_Unified/B09.lua:72 5/100 × GetMaxMana()" },
          max_range = { 1000, unit = "dist", source = "kv:max_range" },
        },
        text = {
          zh_TW = { name = "意念爆發",
            body = "持續施法，引導法力在目標敵人身上爆發：施法開始時與之後每 {tick} 秒，對目標 {radius} 範圍內非魔免的敵方英雄與部隊造成最大法力 {mana_pct} 加 {damage} 的魔法傷害。目標超出 {max_range}、變成魔免或爆發標記被驅散時結束（驅散時施法不會中斷）。" },
          en = { name = "Mind Burst",
            body = "Channels mana into the target enemy: when the channel starts and every {tick} seconds after, non-spell-immune enemy heroes and units within {radius} of it take magic damage equal to {mana_pct} of Masayuki's maximum mana plus {damage}. Ends if the target goes beyond {max_range}, becomes spell immune, or the burst mark is dispelled (dispelling does not stop the channel)." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" },
          aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b09e_burst", text = { zh_TW = { name = "意念爆發", body = "每 {tick} 秒，自身 {radius} 範圍內非魔免的敵方英雄與部隊受到魔法傷害；被驅散時結束。" }, en = { name = "Mind Burst", body = "Every {tick} seconds, non-spell-immune enemy heroes and units within {radius} take magic damage. Ends if dispelled." } }, duration = "$channel_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "b09e_burst_glow",
            notes = "原作 modifier_B09E：Duration 10、IsPurgable 1、IsDebuff、ThinkInterval 0.5；OnCreated 與每次 think 各爆發一次。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b09e_link", at = "caster", to = "target" },
            { op = "apply_status", status = "b09e_burst", to = "target" },
            { op = "call", routine = "b09e_pulse", args = { tgt = "target" } },
          },
          on_channel_tick = { interval = "$tick", filter = "has_status(target, b09e_burst)", actions = {
            { op = "call", routine = "b09e_pulse", args = { tgt = "target" } },
          } },
          on_channel_end = {
            { op = "remove_status", from = "target", status = "b09e_burst" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B09.lua:64-107",
            text = "傷害以目標當下位置為圓心、技能的 FLAG_NONE 選取（排除魔免）；傷害後才檢查距離與魔免，所以最後一跳照樣打出。距離用 3D Length（含高度差）。" },
          { policy = "as_source", ref = "npc/B_Unified/B09.txt:178-185",
            text = "modifier_B09E 可被一般驅散；被驅散後不再爆發，但昌幸繼續引導到時間結束或被打斷（腳本只在 think 裡中斷引導）。目標死亡時同樣只停止爆發。" },
          { policy = "design_intent", ref = "npc/B_Unified/B09.txt:178-206",
            text = "OnCreated 一次加上十秒內每 0.5 秒一次；第十秒那一次與引導結束、modifier 到期同時，照設計意圖以引導中為準。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B09.lua:102-106,110-131",
            text = "think 後 0.1 秒的「沒在引導就移除」檢查、0.2 秒一次的連線粒子與音效，在 v3 由 on_channel_end 與 cue 處理。" },
        },
        provenance = { "npc/B_Unified/B09.txt:113-219", "vscripts/heroes/B_Unified/B09.lua:55-131" },
      },
      -------------------------------------------------------------------------
      { id = "b09_will_shield", source_id = "B09R", slot = "E", kind = "toggle", max_level = 4, tags = { "basic", "toggle" },
        values = {
          mana_ratio = { 2, 3, 4, 5, unit = "ratio", source = "kv:manadamage" },
          absorb_pct = { 30, unit = "pct", source = "script:vscripts/heroes/B_Unified/B09.lua:171 dmg × 0.3" },
        },
        text = {
          zh_TW = { name = "實念之盾",
            body = "開啟後，每次受到敵方傷害，以法力補回該次傷害的 {absorb_pct}：每一點法力抵 {mana_ratio} 點傷害；法力不足或剩餘生命不高於補回量時不生效。關閉即解除。" },
          en = { name = "Shield of Will",
            body = "While toggled on, each instance of damage from an enemy is partly undone with mana: {absorb_pct} of it is restored as health, each point of mana covering {mana_ratio} damage; it does nothing if mana is insufficient or remaining health is not above the restored amount. Toggle off to end." },
        },
        target = { type = "none" },
        statuses = {
          { id = "b09r_shield", text = { zh_TW = { name = "實念之盾", body = "受到敵方傷害時，以法力補回該次傷害的 {absorb_pct}，每一點法力抵 {mana_ratio} 點傷害。" }, en = { name = "Shield of Will", body = "When damaged by an enemy, {absorb_pct} of the damage is restored using mana, each point of mana covering {mana_ratio} damage." } }, duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", cue = "b09r_shield", remove_on = { "death" },
            triggers = {
              on_take_damage = { filter = "is_enemy(event.attacker, self)", actions = {
                { op = "let", name = "part", value = "event.damage.final * $absorb_pct" },
                { op = "let", name = "need", value = "part / $mana_ratio" },
                { op = "if", cond = "self.mana >= need and self.hp > part", actions = {
                  { op = "spend_mana", from = "self", amount = "need", ability = "ability" },
                  { op = "set_health", to = "self", value = "self.hp + part" },
                } },
              } },
            },
            notes = "原作 modifier_B09R（Lua modifier，未覆寫 IsPurgable，暫定 none）：OnTakeDamage 受傷後以 SpendMana + SetHealth 補回。" },
        },
        events = {
          on_toggle_on = {
            { op = "cue", id = "b09r_toggle_sound", at = "caster" },
            { op = "apply_status", status = "b09r_shield", to = "caster" },
          },
          on_toggle_off = {
            { op = "cue", id = "b09r_toggle_sound", at = "caster" },
            { op = "remove_status", from = "caster", status = "b09r_shield" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B09.lua:149-176",
            text = "原作不讀這次的傷害量，而是以「每 0.1 秒取樣的生命 − 目前生命」當作傷害：同一個 0.1 秒內的多次傷害（與期間的回血）會重複計入，補回量依 think 時序而變。照設計意圖（說明：吸收每次受到的 30% 傷害）以該次最終傷害計算。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B09.lua:169-176",
            text = "只有傷害來源與昌幸不同隊時生效（友軍與自己造成的傷害不吸收）；是受傷後以 SetHealth 補回，不經治療事件與減療，致死的傷害擋不住。SpendMana 會觸發耗魔事件。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B09.lua:161-163",
            text = "腳本把 event.unit（受傷者）取名 attacker、event.attacker 取名 victim，名稱相反但邏輯正確。" },
          { policy = "note",
            text = "Lua modifier 死亡時移除；切換技能在重生後是否仍顯示開啟、需要重新開關，未驗證。" },
          { policy = "note", ref = "npc/B_Unified/B09.txt:225-226",
            text = "AbilityCooldown 與 AbilityManaCost 都是 0：開關沒有成本。" },
        },
        provenance = { "npc/B_Unified/B09.txt:220-266", "vscripts/heroes/B_Unified/B09.lua:134-206" },
      },
      -------------------------------------------------------------------------
      { id = "b09_mind_suppression", source_id = "B09T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 140, 130, 120, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 250, 300, 350, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel，推定 Dota 大招預設 6／12／18" },
          radius = { 700, unit = "dist", source = "kv:AOERadius", note = "腳本兩次 FindUnitsInRadius 都寫死 700" },
          heal = { 400, 550, 700, unit = "hp", source = "kv:heal" },
          delay = { 1, unit = "s", source = "kv:delay" },
          damage = { 400, 550, 700, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          silence = { 5, 6, 7, unit = "s", source = "kv:duration" },
          mana_loss = { 60, unit = "pct", source = "script:vscripts/heroes/B_Unified/B09.lua:265 SetMana(GetMana() × 0.4)" },
        },
        text = {
          zh_TW = { name = "心靈壓制",
            body = "指定全地圖任一地點：{radius} 範圍內的友方英雄與部隊立即回復 {heal} 生命並補滿法力。{delay} 秒後，該範圍內的敵方英雄與部隊（包含魔免）流失目前法力的 {mana_loss}、被沉默 {silence} 秒，並受到 {damage} 魔法傷害。" },
          en = { name = "Mind Suppression",
            body = "Targets any point on the map: allied heroes and units within {radius} instantly restore {heal} health and refill their mana. After {delay} seconds, enemy heroes and units in the area (spell-immune ones included) lose {mana_loss} of their current mana, are silenced for {silence} seconds and take {damage} magic damage." },
        },
        target = { type = "point", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b09t_silence", text = { zh_TW = { name = "沉默", body = "無法施放技能。" }, en = { name = "Silenced", body = "Cannot cast abilities." } }, duration = "$silence", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", status_resistance = "ignore", states = { "silence" },
            notes = "原作 modifier_B09T：IsPurgable 1、IsDebuff；列在 ultimate_modifiers，不受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "spell_hint", at = "at", radius = "$radius" },
            { op = "cue", id = "b09t_cast_sound", at = "at" },
            { op = "for_each",
              select = { from = "at", shape = "circle", radius = "$radius", team = "ally",
                types = { "hero", "creep", "neutral", "summon" } },
              actions = {
                { op = "set_health", to = "it", value = "min(it.hp + $heal, it.max_hp)" },
                { op = "set_mana", to = "it", value = "it.max_mana" },
              } },
            { op = "delay", time = "$delay", capture = { "at" }, actions = {
              { op = "cue", id = "b09t_flash", at = "at" },
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                actions = {
                  { op = "set_mana", to = "it", value = "it.mana * (1 - $mana_loss)" },
                  { op = "apply_status", status = "b09t_silence", to = "it" },
                  { op = "damage", to = "it", type = "magical", amount = "$damage" },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B09.txt:268-349",
            text = "KV 沒寫 AbilityCastRange（0），施放距離不受限，全地圖可放；也沒寫 RequiredLevel，需要等級照 Dota 大招預設推定。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B09.lua:223-237",
            text = "友軍回復是 SetHealth／SetMana，不經治療事件與增減療；昌幸自己在範圍內也算，扣完耗魔後立刻補滿法力。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B09.lua:254-269",
            text = "一秒後以施放點為中心結算；MAGIC_IMMUNE_ENEMIES：魔免敵人照樣流失法力與被沉默，魔法傷害由規則層歸零；傷害以 AMHC:Damage 造成，昌幸死亡時沒有傷害但仍沉默與扣魔。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B09.lua:216-222,245-253",
            text = "hide_unit／npc_dummy_unit 只用來顯示 spell hint 與特效，改用 cue。" },
        },
        provenance = { "npc/B_Unified/B09.txt:268-349", "vscripts/heroes/B_Unified/B09.lua:209-270",
          "vscripts/events.lua:246-268" },
      },
    },
    routines = {
      b09e_pulse = {
        params = { tgt = "unit" },
        actions = {
          { op = "cue", id = "b09e_pulse", at = "tgt" },
          { op = "for_each",
            select = { from = "tgt", shape = "circle", radius = "$b09_mind_burst.radius", team = "enemy",
              types = { "hero", "creep", "neutral", "summon" }, where = "not it.is_building" },
            actions = {
              { op = "damage", to = "it", type = "magical",
                amount = "caster.max_mana * $b09_mind_burst.mana_pct + $b09_mind_burst.damage", require_source_alive = false },
            } },
          { op = "if", cond = "distance(caster, tgt) > $b09_mind_burst.max_range or tgt.is_magic_immune", actions = {
            { op = "remove_status", from = "tgt", status = "b09e_burst" },
            { op = "end_channel", interrupted = true },
          } },
        },
        notes = "modifier_B09E_OnIntervalThink（B09.lua:64-107）。",
      },
    },
  }
end
