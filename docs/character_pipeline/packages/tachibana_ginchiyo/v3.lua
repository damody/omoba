-- C09 立花誾千代（current 版，heromap_skill C09 ["16"] = "WERT"，沒有 D）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/C_Neutral/C09_2.txt Ability1-3、Ability6）。*_old 是 11 版，不做。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，C09_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（v3.md §12.3 推定）。
-- datadriven Properties 直接給的移速百分比（不經 nobu 緩速池，dummy.lua Slow 版本都已註解）寫成 move_speed_pct。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "C09",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "C09",
      text = { zh_TW = { name = "立花誾千代" }, en = { name = "Tachibana Ginchiyo" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c09_frost_thorn", source_id = "C09W", slot = "Q", kind = "toggle", max_level = 4, tags = { "basic", "toggle", "orb" },
        values = {
          mana_per_attack = { 6, 7, 8, 9, unit = "mana", source = "kv:mana_cost" },
          damage = { 40, 50, 60, 70, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          slow = { 20, 30, 40, 50, unit = "pct", kv = { -20, -30, -40, -50 }, source = "kv:debuff_movement_speed",
            source_differs = "KV 以負值表示減益；這裡以正值表示降低量" },
          attack_slow = { 25, 45, 65, 85, unit = "atk_speed", kv = { -25, -45, -65, -85 }, source = "kv:debuff_attack_speed",
            source_differs = "KV 以負值表示減益；這裡以正值表示降低量" },
          duration = { 2, unit = "s", source = "kv:debuff_duration" },
        },
        text = {
          zh_TW = { name = "霜刺",
            body = "開啟時每次對非建築目標出手消耗 {mana_per_attack} 法力；法力不足時自動關閉。命中非建築目標時額外造成 {damage} 魔法傷害；目標不是魔免時，移動速度降低 {slow}、攻擊速度降低 {attack_slow}，持續 {duration} 秒。對建築出手不消耗法力，也不附加效果。" },
          en = { name = "Frost Thorn",
            body = "While active, each attack started against a non-building costs {mana_per_attack} mana; the ability turns off when Ginchiyo lacks the mana. Attacks that land on non-buildings deal {damage} bonus magic damage and, if the target is not spell immune, slow its movement speed by {slow} and attack speed by {attack_slow} for {duration} seconds. Attacks against buildings cost no mana and carry no effect." },
        },
        target = { type = "none" },
        statuses = {
          { id = "c09w_orb", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", remove_on = { "death" },
            triggers = {
              on_attack_windup = {
                { op = "if", cond = "self.mana < $mana_per_attack",
                  actions = { { op = "toggle_ability", of = "self", ability = "c09_frost_thorn", on = false } },
                  otherwise = {
                    -- FC-090（D13）：對建築不扣魔、不套法球，也不多打一次
                    { op = "if", cond = "not event.target.is_building",
                      actions = {
                        { op = "spend_mana", from = "self", amount = "$mana_per_attack", ability = "ability" },
                        { op = "modify_attack", is_orb = true },
                      } },
                  } },
              },
              on_attack_hit = { filter = "not event.target.is_building", actions = {
                { op = "damage", to = "event.target", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "if", cond = "not event.target.is_magic_immune", actions = {
                  { op = "apply_status", status = "c09w_slow", to = "event.target" },
                } },
              } },
            },
            notes = "原作 modifier_C09W：IsHidden、Orb Priority ABILITY；沒寫 IsPurgable，暫定 basic。" },
          { id = "c09w_slow", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "c09w_ice",
            text = {
              zh_TW = { name = "霜刺減速", body = "移動速度降低 {slow}、攻擊速度降低 {attack_slow}。" },
              en = { name = "Frost Thorn Slow", body = "Movement speed reduced by {slow} and attack speed reduced by {attack_slow}." },
            },
            stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            notes = "原作 modifier_C09W_debuff：IsPurgable 1，Properties 直接減移速與攻速。" },
        },
        events = {
          on_toggle_on = { { op = "apply_status", status = "c09w_orb", to = "caster" } },
          on_toggle_off = { { op = "remove_status", from = "caster", status = "c09w_orb" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/C_Neutral/C09.lua:43-47",
            text = "FC-090（D13 修正：對建築不扣魔、不套法球，拿掉多打的那一次）。原作：對建築出手時呼叫 PerformAttack(target, …, useProjectile = true, …)，在每次出手時額外射出一次真實普攻（processProcs = false，不觸發法球與攻擊特效）；開著霜刺拆塔等於攻擊次數加倍。看起來原意是「對建築不扣法力、不套法球」，實際卻多打一次。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C09.lua:36-40",
            text = "法力檢查在 OnAttackStart，對建築出手也檢查；法力不足時關閉開關並移除法球狀態，這一擊沒有效果。扣魔在出手前搖，攻擊被取消或落空仍然扣魔。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C09.lua:55-68",
            text = "OnAttackLanded 不分敵我：反補友軍也會造成魔法傷害與減速。ApplyDamage 不經 AMHC，誾千代死後才命中的遠程普攻仍會造成傷害。魔免目標只吃（被規則層歸零的）魔法傷害，不被減速。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C09.lua:9-11,19-20,29-34",
            text = "nobu 法球槽（nobuorb1）的佔用與檢查都已註解：霜刺不佔也不讀法球槽，只靠 datadriven Orb 優先度。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C09.txt:76-85",
            text = "modifier_C09W_debuff2（隱藏、沒有屬性）和 debuff 一起套上，但沒有任何地方讀它，沒有效果。" },
          { policy = "as_source",
            text = "開關開著死亡時，datadriven 狀態隨死亡移除但開關仍亮著，復活後要重新開關才有效果（推測，未在 Dota 實測）；資料以 remove_on death 表達。" },
          { policy = "note", ref = "npc/C_Neutral/C09.txt:20-31", text = "KV 沒寫冷卻與耗魔；耗魔全在每次出手。" },
        },
        provenance = { "npc/C_Neutral/C09.txt:19-158", "vscripts/heroes/C_Neutral/C09.lua:4-70" },
      },
      -------------------------------------------------------------------------
      { id = "c09_white_heat", source_id = "C09E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 5, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, 90, 100, 110, unit = "mana", kv = { 80, 90, 100, 110, 120 }, source = "kv:AbilityManaCost",
            source_differs = "KV 寫了五個等級的耗魔，技能只有四級（其他 AbilitySpecial 都是四個值），第五個用不到" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          heal = { 100, 175, 250, 325, unit = "heal", source = "kv:C09E_heal" },
          attack_speed = { 110, 130, 150, 170, unit = "atk_speed", source = "kv:C09E_asBonus" },
          duration = { 4, 5, 6, 7, unit = "s", source = "kv:C09E_duration" },
        },
        text = {
          zh_TW = { name = "白熱",
            body = "回復自身 {heal} 生命，並在 {duration} 秒內提高 {attack_speed} 攻擊速度。" },
          en = { name = "White Heat",
            body = "Heals Ginchiyo for {heal} and grants +{attack_speed} attack speed for {duration} seconds." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c09e_fervor", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "白熱", body = "攻擊速度提高 {attack_speed}。" },
              en = { name = "White Heat", body = "Attack speed increased by {attack_speed}." },
            },
            reveal = "public", cue = "c09e_stomp_glow", stats = { attack_speed = "$attack_speed" },
            notes = "原作 modifier_C09E：ATTACKSPEED_BONUS_CONSTANT，時間由腳本以 C09E_duration 給；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "c09e_fervor", to = "caster" },
            { op = "heal", to = "caster", source = "caster", amount = "$heal" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C09.txt:170-182",
            text = "腳本先套攻速狀態，KV 的 datadriven Heal 再回血（經全域治療規則）；PopupNumbers 只是呈現。" },
        },
        provenance = { "npc/C_Neutral/C09.txt:159-217", "vscripts/heroes/C_Neutral/C09.lua:72-79" },
      },
      -------------------------------------------------------------------------
      { id = "c09_ice_soul_dance", source_id = "C09R", slot = "E", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 16, 13, 10, 7, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          distance = { 550, 600, 650, 700, unit = "dist", source = "kv:C09R_distance" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:C09R_damage" },
          radius = { 300, unit = "dist", source = "kv:C09R_radius" },
          duration = { 3, unit = "s", source = "kv:C09R_debuffDuration" },
          slow = { 50, unit = "pct", kv = { -50 }, source = "kv:C09R_debuffAsMs",
            source_differs = "KV -50 同時給 MOVESPEED_BONUS_PERCENTAGE；這裡以正值表示降低量" },
          attack_slow = { 50, unit = "atk_speed", kv = { -50 }, source = "kv:C09R_debuffAsMs",
            source_differs = "KV -50 同時給 ATTACKSPEED_BONUS_CONSTANT；這裡以正值表示降低量" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C09.lua:96 modifier_phased 0.1" },
        },
        text = {
          zh_TW = { name = "冰靈舞魂",
            body = "躲開所有追蹤投射物，朝目標地點瞬移最多 {distance}，對落點 {radius} 內非魔免的敵方英雄與部隊造成 {damage} 魔法傷害，並使其移動速度降低 {slow}、攻擊速度降低 {attack_slow}，持續 {duration} 秒。這個減速無法驅散；目標在魔免期間被移除時會立刻補回。" },
          en = { name = "Ice Spirit Dance",
            body = "Dodges all tracking projectiles and blinks up to {distance} toward the target point, dealing {damage} magic damage to non-spell-immune enemy heroes and units within {radius} of the landing spot and slowing their movement speed by {slow} and attack speed by {attack_slow} for {duration} seconds. The slow cannot be dispelled, and if it is removed while the target is spell immune it is reapplied at once." },
        },
        target = { type = "point", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c09r_slow", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "debuff",
            text = {
              zh_TW = { name = "冰靈舞魂緩速", body = "移動速度降低 {slow}、攻擊速度降低 {attack_slow}。無法驅散，在魔免期間被移除時會立刻補回。" },
              en = { name = "Ice Spirit Dance Slow", body = "Movement speed reduced by {slow} and attack speed reduced by {attack_slow}. Cannot be dispelled; reapplied at once if removed while spell immune." },
            },
            reveal = "public", cue = "c09r_ice",
            stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            triggers = {
              on_removed = { filter = "status.remaining > 0 and self.is_magic_immune", actions = {
                { op = "apply_status", status = "c09r_slow", to = "self", source = "caster", duration = "status.remaining" },
              } },
            },
            notes = "原作 modifier_C09R_debuff：IsPurgable 0；OnDestroy 記錄被移除當下是否魔免，腳本 Timer 每 0.1 秒在魔免時以剩餘時間補回。" },
          { id = "c09r_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, notes = "引擎 modifier_phased。" },
        },
        events = {
          on_cast = {
            { op = "dodge_projectiles", of = "caster", from_team = "any", kind = "tracking" },
            { op = "let", name = "dest",
              value = "select(distance(caster, point) > $distance, offset(caster, direction(caster, point), $distance), point)" },
            { op = "teleport", who = "caster", to = "dest" },
            { op = "apply_status", status = "c09r_phased", to = "caster" },
            { op = "cue", id = "c09r_frost_nova", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" } },
              actions = {
                { op = "apply_status", status = "c09r_slow", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C09.txt:219-232",
            text = "KV 沒寫 AbilityCastRange：任何距離都能施放（不會走過去），超過 C09R_distance 時沿方向截斷，落點以 FindClearSpaceForUnit 找空位。距離以含高度差的 3D 長度判斷，資料忽略高度。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C09.lua:103-116",
            text = "搜尋以落地後的位置為圓心，用技能設定（ENEMY、HERO|BASIC、旗標 NONE）：魔免敵人選不到；沒有 FOW 旗標。先套減速再 ApplyDamage（不經 AMHC）。" },
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C09.lua:117-144",
            text = "原作每 0.1 秒檢查：減速不見且最後一次移除時目標是魔免，就以 Timer 累減的剩餘時間補回；移除時不是魔免（被強制移除、死亡）就不再補。補回最多晚 0.1 秒；照設計意圖寫成移除當下立即補回。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C09.lua:115,140-142",
            text = "C09R_state 是存在單位上的欄位，多次施放共用；最新一次移除的結果決定所有 Timer 是否繼續補。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C09.lua:82",
            text = "ProjectileManager:ProjectileDodge 躲開所有正在追蹤誾千代的投射物（不分敵我）。" },
        },
        provenance = { "npc/C_Neutral/C09.txt:218-315", "vscripts/heroes/C_Neutral/C09.lua:81-144" },
      },
      -------------------------------------------------------------------------
      { id = "c09_armor_break", source_id = "C09T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 100, 90, 80, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          radius = { 800, unit = "dist", source = "kv:C09T_radius", note = "AbilityCastRange 也是 800" },
          armor = { 30, 38, 46, unit = "armor", kv = { -30, -38, -46 }, source = "kv:C09T_armor",
            source_differs = "KV 以負的護甲加成表示；這裡以正值表示減少量" },
          duration = { 7, unit = "s", source = "kv:C09T_duration" },
          frost_armor = { 7, 9, 11, unit = "armor", kv = { -7, -9, -11 }, source = "kv:C09T_passive",
            source_differs = "KV 以負的護甲加成表示；這裡以正值表示減少量" },
          tick = { 0.1, unit = "s", hidden = true, source = "kv:modifier_C09T_passive ThinkInterval" },
          hint_time = { 0.5, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C09.lua:153 spell_hint duration 0.5" },
        },
        text = {
          zh_TW = { name = "破甲術",
            body = "被動：周圍 {radius} 內帶有{@c09w_slow}的敵方英雄與部隊（包含魔免）護甲降低 {frost_armor}，持續到減速結束。啟動：使周圍 {radius} 內的敵方英雄與部隊（包含魔免）護甲降低 {armor}，持續 {duration} 秒，不受狀態抗性縮短。" },
          en = { name = "Armor Break",
            body = "Passive: enemy heroes and units within {radius} (spell-immune ones included) that carry {@c09w_slow} lose {frost_armor} armor until that slow ends. Active: enemy heroes and units within {radius} (spell-immune ones included) lose {armor} armor for {duration} seconds; the duration is not reduced by status resistance." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        passive = {
          reveal = "public",
          triggers = {
            periodic = { interval = "$tick", actions = {
              { op = "for_each",
                select = { from = "self", shape = "circle", radius = "$radius", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" },
                  where = "has_status(it, c09w_slow)" },
                actions = {
                  { op = "apply_status", status = "c09t_frost_armor", to = "it", source = "self",
                    duration = "status_remaining(it, c09w_slow)" },
                } },
            } },
          },
        },
        statuses = {
          { id = "c09t_armor_break", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "破甲術", body = "護甲降低 {armor}，不受狀態抗性縮短。" },
              en = { name = "Armor Break", body = "Armor reduced by {armor}. Duration not reduced by status resistance." },
            },
            reveal = "public", status_resistance = "ignore", cue = "c09t_stomp_debuff", stats = { armor = "-$armor" },
            notes = "原作 modifier_C09T_debuff：PHYSICAL_ARMOR_BONUS，時間由腳本給；列在 ultimate_modifiers（不受狀態抗性）；沒寫 IsPurgable，暫定 basic。" },
          { id = "c09t_frost_armor", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "霜刺破甲", body = "護甲降低 {frost_armor}，持續到{@c09w_slow}結束。" },
              en = { name = "Frost Armor Break", body = "Armor reduced by {frost_armor} until {@c09w_slow} ends." },
            },
            reveal = "public", stats = { armor = "-$frost_armor" },
            notes = "原作 modifier_C09T_debuff3：PHYSICAL_ARMOR_BONUS，KV 沒寫 Duration，每次以霜刺減速的剩餘時間套上；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "spell_hint", at = "caster", radius = "$radius", duration = "$hint_time" } },
          on_cast = {
            { op = "cue", id = "c09t_burst", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
              actions = {
                { op = "apply_status", status = "c09t_armor_break", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "no_effect", ref = "npc/C_Neutral/C09.txt:398-412",
            text = "modifier_C09T_debuff2（暈眩 1 秒）沒有任何地方套用，C09T_stun_duration 讀進變數但沒有使用；施放沒有暈眩。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C09.txt:332-339",
            text = "OnUpgrade 呼叫 C09T_OnUpgrade，該函式不存在，執行期報錯、沒有效果。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C09.lua:169-175,184-193",
            text = "主動與被動都以施法者為圓心、帶 MAGIC_IMMUNE_ENEMIES、排除 _G.EXCLUDE_TARGET_NAME（對應選取預設排除 untargetable_by_spells）；沒有 FOW 旗標，看不見的敵人也算。" },
          { policy = "as_source", ref = "npc/C_Neutral/C09.txt:381-392",
            text = "學會大絕後就有被動：每 0.1 秒找範圍內帶 modifier_C09W_debuff 的敵人（不管是誰施加的），以該減速的剩餘時間套 −7/−9/−11 護甲；離開範圍後殘留到減速結束。主動的 −30 與被動的 −7 是不同 modifier，可以相加。" },
          { policy = "design_intent", ref = "npc/C_Neutral/C09.txt:381-392",
            text = "被動每 0.1 秒選一次（30Hz 下約每三幀）；照 KV 間隔寫成 periodic。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C09.lua:146-158",
            text = "前搖時以 hide_unit 加 nobu_modifier_spell_hint 顯示 0.5 秒範圍提示；showTitle 已註解。都只是呈現，改用 cue。" },
        },
        provenance = { "npc/C_Neutral/C09.txt:316-443", "vscripts/heroes/C_Neutral/C09.lua:146-194", "vscripts/events.lua:246-269" },
      },
    },
  }
end
