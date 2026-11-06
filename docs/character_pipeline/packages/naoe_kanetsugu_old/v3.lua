-- B36_old 直江兼續（舊版，11 版技能組 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 版本：原作 11 版。choose_11 拔掉全部技能，依 heromap_skill B36 ["11"] = "WERDT" 逐一 AddAbility("B36" .. 字母 .. "_old")，
--   heromap_autoskill B36 ["11"] = "D"：B36D_old 自動學一級（vscripts/addon_game_mode.lua:315,418、vscripts/items/Addon_Items/tofar_goback.lua:96-140）。
-- 槽位：B36W_old/E_old/R_old/T_old → Q/W/E/R，B36D_old（隱藏被動：遠程攻擊與射程）照本體慣例放 D，以 hero.auto_learn 表達自動學會。
-- 選 16 版或 20 版時會移除 modifier_B36D_old（tofar_goback.lua:35-37,81-83），舊版保留。
-- KV 在 npc/B_Unified/B36.txt:423-763，腳本在 vscripts/heroes/B_Unified/B36.lua:209-328。
-- 說明取 resource/addon_tchinese.txt:3206-3213（W/E/R/T 有 _old 說明），數字換成佔位符；B36D_old 沒有說明，依 KV 還原（assumptions.md）。
-- 慣例照本體 B36：KV 未寫 IsPurgable 的 datadriven modifier 暫定 dispel = "basic"；datadriven Damage 寫 require_source_alive = false，AMHC:Damage 用預設。
-- B36_precache、attribute_bonusx、halloween_attack、magical_resistance 屬規則層或資產，不列入。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B36_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = {
      source_id = "B36",
      auto_learn = { "b36_old_far_reach" },
      text = { zh_TW = { name = "直江兼續（舊版）" }, en = { name = "Naoe Kanetsugu (Old)" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b36_old_binding_seal", source_id = "B36W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B36.txt:434" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B36.txt:435" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B36.txt:436" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B36.txt:432" },
          radius = { 500, unit = "dist", kv = { 400 }, source = "kv:radius@npc/B_Unified/B36.txt:486（modifier_B36W_old Aura_Radius）",
            source_differs = "AOERadius 400（B36.txt:437）只是施放指示圈，光環實際半徑是 radius 500" },
          duration = { 10, unit = "s", source = "script:vscripts/heroes/B_Unified/B36.lua:216 modifier_kill duration = 10" },
          slow = { 20, 30, 40, 50, unit = "pct", kv = { -20, -30, -40, -50 },
            source = "kv:speed@npc/B_Unified/B36.txt:481（modifier_B36W_old2 MOVESPEED_BONUS_PERCENTAGE）",
            source_differs = "KV 以負值表示減少，v3 以正值表示" },
          attack_slow = { 20, 30, 40, 50, unit = "atk_speed", kv = { -20, -30, -40, -50 },
            source = "kv:speed@npc/B_Unified/B36.txt:481（modifier_B36W_old2 ATTACKSPEED_BONUS_CONSTANT 也讀這個變數）",
            source_differs = "KV 以負值表示減少，v3 以正值表示；原作說明只寫降低速度" },
          aura_linger = { 0.5, unit = "s", hidden = true, source = "rules:Dota 光環預設離開後殘留 0.5 秒（KV 未寫）" },
        },
        text = {
          zh_TW = { name = "符咒‧金縛",
            body = "在指定地點召喚一個直江印，持續 {duration} 秒；周圍 {radius} 內的敵方英雄與部隊（包含魔免）移動速度降低 {slow}、攻擊速度降低 {attack_slow}。" },
          en = { name = "Talisman: Binding",
            body = "Summons a Naoe Seal at the target point for {duration} seconds. Enemy heroes and units within {radius} of it (spell-immune ones included) have their movement speed reduced by {slow} and attack speed reduced by {attack_slow}." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b36w_old_seal_aura", duration = "infinite", stacking = "keep", dispel = "none", visibility = "hidden",
            reveal = "public",
            aura = { radius = "$radius", team = "enemy", types = BASIC, flags = { "include_magic_immune" },
              status = "b36w_old_bind", linger = "$aura_linger" },
            notes = "原作 modifier_B36W_old：掛在 npc_dummy_unit 上的光環（ENEMY、HERO | BASIC、MAGIC_IMMUNE_ENEMIES、不給自己）；dummy 有 majia 魔免，驅散不到，寫 none。" },
          { id = "b36w_old_bind", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "b36w_old_move_slow",
            stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            text = {
              zh_TW = { name = "符咒‧金縛", body = "在直江印附近：移動速度降低 {slow}，攻擊速度降低 {attack_slow}。" },
              en = { name = "Talisman: Binding", body = "Near the Naoe Seal: movement speed reduced by {slow}, attack speed reduced by {attack_slow}." },
            },
            notes = "原作 modifier_B36W_old2：IsDebuff、Duration 4（光環給的 modifier 由光環管理時間，推定不生效）；直接寫 MOVESPEED_BONUS_PERCENTAGE 與 ATTACKSPEED_BONUS_CONSTANT（不經 nobu 緩速池）；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "b36w_old_seal", at = "at", duration = "$duration" },
            { op = "create_unit", template = "b36w_old_seal_unit", at = "at", owner = "caster", team = "caster",
              control = "none", lifetime = "$duration", as = "u" },
            { op = "apply_status", status = "b36w_old_seal_aura", to = "u", source = "u" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B36.lua:209-223",
            text = "在施放點建 npc_dummy_unit（modifier_kill 10 秒、majia_vison：無血條、不可選取、攻擊免疫、魔免），掛 modifier_B36W_old 光環；npc_dummy_unit 視野 0，直江印不提供視野。沒有傷害。" },
          { policy = "as_source", ref = "npc/B_Unified/B36.txt:462-473",
            text = "移速百分比與攻速點數讀同一個 KV 變數 speed，所以攻速降低 20/30/40/50 點。光環給的 modifier 寫了 Duration 4；推定光環給的 modifier 由光環管理，離開後照 Dota 預設 0.5 秒殘留。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B36.lua:212",
            text = "腳本把施放點存進 caster.B36D_point（本體履及用的欄位），舊版沒有履及，沒有作用。粒子 10 秒後移除只是呈現。和本體不同（本體直江印是延遲打擊）。" },
        },
        provenance = { "npc/B_Unified/B36.txt:423-489", "vscripts/heroes/B_Unified/B36.lua:209-223",
          "npc/npc_units_custom_load.txt:5111-5145", "npc/npc_abilities_custom_load.txt:5628-5651", "resource/addon_tchinese.txt:3206-3207" },
      },
      -------------------------------------------------------------------------
      { id = "b36_old_charm_barrier", source_id = "B36E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 32, 30, 28, 26, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B36.txt:506" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B36.txt:508" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B36.txt:505" },
          duration = { 20, unit = "s", source = "kv:modifier_B36E_old Duration@npc/B_Unified/B36.txt:516,524" },
          interval = { 0.2, unit = "s", source = "kv:modifier_B36E_old ThinkInterval@npc/B_Unified/B36.txt:528" },
          radius = { 800, unit = "dist", source = "kv:radius2@npc/B_Unified/B36.txt:561（AbilityCastRange 807 同值）" },
          damage = { 30, 35, 40, 45, unit = "dmg", damage_type = "magical", source = "kv:dmg2@npc/B_Unified/B36.txt:566（AbilityDamage 509 同值）" },
          per_level = { 5, unit = "dmg", damage_type = "magical", source = "script:vscripts/heroes/B_Unified/B36.lua:230 dmg2 + 5 * caster:GetLevel()" },
          bolt_speed = { 700, unit = "speed", hidden = true, source = "script:vscripts/heroes/B_Unified/B36.lua:268 iMoveSpeed = 700" },
        },
        text = {
          zh_TW = { name = "符咒‧護符障壁",
            body = "釋放護符之力環繞身旁 {duration} 秒：護符每 {interval} 秒朝周圍 {radius} 內隨機一名看得見的非魔免敵方英雄或部隊射出追蹤符咒，造成 {damage} 魔法傷害，並隨英雄等級每級增加 {per_level}。" },
          en = { name = "Talisman: Charm Barrier",
            body = "Surrounds himself with talismans for {duration} seconds. Every {interval} seconds a talisman homes in on a random visible, non-spell-immune enemy hero or unit within {radius}, dealing {damage} magic damage plus {per_level} per hero level." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b36e_old_barrier", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b36e_old_mist",
            text = {
              zh_TW = { name = "符咒‧護符障壁", body = "每 {interval} 秒對周圍一名敵人射出符咒。" },
              en = { name = "Talisman: Charm Barrier", body = "Fires a talisman at a nearby enemy every {interval} seconds." },
            },
            triggers = {
              periodic = { interval = "$interval", actions = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                    flags = { "visible_only" }, order = "random", limit = 1 },
                  actions = {
                    { op = "projectile", kind = "tracking", from = "self", to = "it", speed = "$bolt_speed", cue = "b36e_old_talisman",
                      on_hit = {
                        { op = "damage", to = "it", source = "caster", type = "magical", amount = "$damage + $per_level * caster.level" },
                      } },
                  } },
              } },
            },
            notes = "原作 modifier_B36E_old：IsBuff、Duration 20、ThinkInterval 0.2；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b36e_old_barrier", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B36.lua:233-274",
            text = "每 0.2 秒以 radius2 800、DOTA_UNIT_TARGET_FLAG_FOW_VISIBLE（不含魔免）搜尋，隨機挑一個射出速度 700 的追蹤投射物（datadriven 投射物可被閃避）；命中時 AMHC:Damage 魔法 dmg2 + 5 × 英雄等級（命中當下的等級；兼續死亡時不造成）。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B36.lua:236-247",
            text = "第一段以 radius 250 搜尋後，迴圈裡的純粹傷害被註解掉，沒有作用；KV radius 250 不列入。" },
          { policy = "note", text = "AbilityCastRange 800 對無目標技能沒有作用。和本體不同（本體 W 是劃心為籬）。" },
        },
        provenance = { "npc/B_Unified/B36.txt:492-569", "vscripts/heroes/B_Unified/B36.lua:225-274", "resource/addon_tchinese.txt:3208-3209" },
      },
      -------------------------------------------------------------------------
      { id = "b36_old_curse_charm", source_id = "B36R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 30, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B36.txt:591" },
          mana_cost = { 80, 100, 120, 140, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B36.txt:595" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B36.txt:586" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B36.txt:587" },
          duration = { 1, 2, 3, 4, unit = "s", source = "kv:duration@npc/B_Unified/B36.txt:615" },
          slow = { 100, unit = "pct", kv = { -100 }, source = "kv:modifier_B36R MOVESPEED_BONUS_PERCENTAGE@npc/B_Unified/B36.txt:626",
            source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          damage = { 1, unit = "dmg", damage_type = "magical", source = "script:vscripts/heroes/B_Unified/B36.lua:284 AMHC:Damage(caster, target, 1, MAGICAL)" },
        },
        text = {
          zh_TW = { name = "符咒‧咒之符",
            body = "將一名非魔免的敵方英雄或部隊變成青蛙 {duration} 秒：無法攻擊、施法與使用道具，閃避與格擋失效，移動速度降低 {slow}；並造成 {damage} 魔法傷害。" },
          en = { name = "Talisman: Curse",
            body = "Turns a non-spell-immune enemy hero or unit into a frog for {duration} seconds: it cannot attack, cast or use items, loses evasion and damage block, and is slowed by {slow}. Also deals {damage} magic damage." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b36r_old_hex", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "cleanse_protected" }, cue = "b36r_old_frog_model",
            states = { "disarm", "mute", "silence", "evade_disabled", "block_disabled" },
            text = {
              zh_TW = { name = "青蛙", body = "變成青蛙：無法攻擊、施法與使用道具，閃避與格擋失效。" },
              en = { name = "Frog", body = "Turned into a frog: cannot attack, cast or use items; evasion and damage block are disabled." },
            },
            notes = "原作 modifier_voodoo_lua（Lua，同 A28W）：IsPurgable true；MODEL_CHANGE 青蛙、MOVESPEED_BASE_OVERRIDE 50、DISARMED、MUTED、EVADE_DISABLED、BLOCK_DISABLED、SILENCED。列在 _G.EXCLUDE_MODIFIER_NAME。" },
          { id = "b36r_old_slow", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "cleanse_protected" },
            stats = { move_speed_pct = "-$slow" },
            text = {
              zh_TW = { name = "咒之符", body = "移動速度降低 {slow}。" },
              en = { name = "Curse", body = "Movement speed reduced by {slow}." },
            },
            notes = "原作 modifier_B36R（B36R_old 的 Modifiers 區塊）：IsPurgable 1、IsDebuff 1、MOVESPEED_BONUS_PERCENTAGE -100；列在 _G.EXCLUDE_MODIFIER_NAME。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b36r_old_strike", at = "target" },
            { op = "apply_status", status = "b36r_old_hex", to = "target" },
            { op = "apply_status", status = "b36r_old_slow", to = "target" },
            { op = "damage", to = "target", type = "magical", amount = "$damage" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B36.lua:276-285",
            text = "同 A28W 的寫法：modifier_voodoo_lua（變青蛙）＋ modifier_B36R（−100% 移速）＋ AMHC 1 點魔法傷害；兩個 modifier 都在 _G.EXCLUDE_MODIFIER_NAME，以 tag cleanse_protected 表達。MOVESPEED_BASE_OVERRIDE 50 與 −100% 同時存在時由最低移速決定，不寫成 stat。" },
          { policy = "note", ref = "vscripts/heroes/modifier_voodoo_lua.lua:15-38",
            text = "模型固定為青蛙；原作說明「轉化為不固定的生物」，其他模型分支是死碼。KV 的 radius 500 與 AOERadius 沒有被使用，不列入。" },
        },
        provenance = { "npc/B_Unified/B36.txt:570-639", "vscripts/heroes/B_Unified/B36.lua:276-285",
          "vscripts/heroes/modifier_voodoo_lua.lua:1-66", "vscripts/library/common/dummy.lua:376-392", "resource/addon_tchinese.txt:3210-3211" },
      },
      -------------------------------------------------------------------------
      { id = "b36_old_far_reach", source_id = "B36D_old", slot = "D", kind = "passive", max_level = 1, learnable = false,
        tags = { "basic" },
        values = {
          range_bonus = { 572, unit = "dist", source = "kv:modifier_B36D_old MODIFIER_PROPERTY_ATTACK_RANGE_BONUS@npc/B_Unified/B36.txt:751" },
        },
        text = {
          zh_TW = { name = "符咒攻擊",
            body = "天生被動：普攻改為遠程攻擊，攻擊距離 +{range_bonus}。" },
          en = { name = "Talisman Strike",
            body = "Innate passive: basic attacks become ranged, and attack range is increased by {range_bonus}." },
        },
        target = { type = "none" },
        events = {
          on_learn = { { op = "apply_status", status = "b36d_old_ranged", to = "caster" } },
        },
        statuses = {
          { id = "b36d_old_ranged", duration = "infinite", stacking = "keep", dispel = "none", visibility = "hidden",
            reveal = "public", scope = "hero", states = { "ranged_attack" },
            stats = { attack_range = "$range_bonus" },
            notes = "原作 modifier_B36D_old：Passive、IsHidden、ATTACK_RANGE_BONUS 572，OnCreated 以 SetAttackCapability 改成遠程攻擊。寫了 IsPurgable 1 與 IsDebuff 1（見 quirks，已修為不可驅散）。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/B_Unified/B36.txt:743-760",
            text = "FC-173 D22 舊版，比照 D13 修：原作天生被動的 modifier_B36D_old 寫了 IsPurgable 1 與 IsDebuff 1，敵方的一般驅散就能把它當減益移除，之後失去 +572 射程直到重新取得（SetAttackCapability 不會還原，變成射程很短的遠程攻擊）。這是天生特性，照設計意圖修為不可驅散（dispel = none）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B36.lua:325-328",
            text = "OnCreated 呼叫 SetAttackCapability(RANGED)，以狀態旗標 ranged_attack 表達；投射物外觀依英雄模型。被動技能自動學會一級（heromap_autoskill \"D\"），不花技能點。" },
          { policy = "note", ref = "npc/B_Unified/B36.txt:719-740",
            text = "AbilityBehavior PASSIVE | HIDDEN：技能列不顯示。KV 沒寫 MaxLevel，只會被 SetLevel(1)，寫 max_level 1。KV 沒有名稱與說明，名稱依效果推定（assumptions.md）。" },
        },
        provenance = { "npc/B_Unified/B36.txt:719-763", "vscripts/heroes/B_Unified/B36.lua:325-328",
          "vscripts/addon_game_mode.lua:418", "vscripts/items/Addon_Items/tofar_goback.lua:35-37,81-83,132-135" },
      },
      -------------------------------------------------------------------------
      { id = "b36_old_kill_formation", source_id = "B36T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 140, 130, 120, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B36.txt:661" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B36.txt:664" },
          cast_point = { 0.5, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B36.txt:658" },
          required_level = { 6, 12, 18, unit = "level",
            source = "rules:Dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          count = { 36, unit = "count", source = "kv:ice_count@npc/B_Unified/B36.txt:683" },
          step_angle = { 10, unit = "deg", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B36.lua:296 delta_theta = 3.14 * 2 / ice_count（約 360° / 36）" },
          distance = { 1000, unit = "dist", source = "kv:aoe_radius@npc/B_Unified/B36.txt:673（fDistance）" },
          width = { 100, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B36.lua:307-308 fStartRadius／fEndRadius 100" },
          speed = { 500, unit = "speed", hidden = true, kv = { 300 },
            source = "script:vscripts/heroes/B_Unified/B36.lua:294 projectile_speed = 500",
            source_differs = "KV speed 300（B36.txt:688）沒有被讀取，速度寫死 500" },
          damage = { 180, 220, 260, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/B_Unified/B36.txt:665（ice_damage 678 同值，OnProjectileHitUnit 讀 AbilityDamage）" },
        },
        text = {
          zh_TW = { name = "禁咒‧殺之陣",
            body = "向周圍放射 {count} 道符咒，各自飛行 {distance}，穿透路徑上的非魔免敵方英雄與部隊，每一道造成 {damage} 魔法傷害；越靠近兼續，被越多道符咒命中。" },
          en = { name = "Forbidden Curse: Killing Formation",
            body = "Fires {count} talismans outward in all directions, each flying {distance} and piercing non-spell-immune enemy heroes and units in its path for {damage} magic damage per talisman; enemies closer to Kanetsugu are hit by more of them." },
        },
        target = { type = "none", aoe_radius = "$distance" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        events = {
          on_cast = {
            { op = "let", name = "center", value = "position(caster)" },
            { op = "repeat", count = "$count", as = "i", actions = {
              { op = "projectile", kind = "linear", from = "center", direction = "rotate(vec(1, 0), $step_angle * (i + 1))",
                speed = "$speed", distance = "$distance", radius = "$width", pierce = true,
                team = "enemy", types = BASIC, cue = "b36t_old_ice",
                on_hit = {
                  { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B36.lua:287-323",
            text = "36 道線性投射物從施法者位置等角度放射（角度步長 2 × 3.14 / 36，比 10° 略小，最後一道差約 0.09°，資料寫 10°），飛行 1000、半徑 100、速度 500、穿透。目標旗標取技能 KV（沒寫 → 不含魔免）。靠近施法者時相鄰投射物重疊，同一單位會被多道命中，這就是說明「越靠近傷害越高」。" },
          { policy = "as_source", ref = "npc/B_Unified/B36.txt:707-716",
            text = "OnProjectileHitUnit 的 datadriven Damage 魔法 AbilityDamage，不檢查施法者存活（require_source_alive = false）。" },
          { policy = "note", text = "SpellImmunityType ENEMIES_YES 但投射物不命中魔免；KV 沒寫 RequiredLevel，照大招預設 6/12/18。和本體不同。" },
        },
        provenance = { "npc/B_Unified/B36.txt:642-717", "vscripts/heroes/B_Unified/B36.lua:287-323", "resource/addon_tchinese.txt:3212-3213" },
      },
    },
    unit_templates = {
      { id = "b36w_old_seal_unit", source_id = "npc_dummy_unit",
        text = { zh_TW = { name = "直江印" }, en = { name = "Naoe Seal" } },
        attack = { kind = "none" },
        vision = { day = "0", night = "0" },
        ai = "none", owner_death = "persist", tags = { "dummy" },
        notes = "原作 npc_dummy_unit（MOVE_NONE、視野 0）加 majia_vison（無血條、不可選取、不在小地圖、無碰撞、攻擊免疫、魔免）；modifier_kill 10 秒。",
        provenance = { "npc/npc_units_custom_load.txt:5111-5145", "vscripts/heroes/B_Unified/B36.lua:215-219" } },
    },
  }
end
