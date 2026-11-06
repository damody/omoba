-- A11 齋藤道三（舊版，11 版 heromap_skill A11 ["11"] = "WERT"，沒有 D）— package schema v3。
-- 版本：選擇 11 版時 tofar_goback.lua choose_11 拔掉全部技能，依 _G.heromap_skill[A11]["11"] = "WERT"
-- 依序 AddAbility("A11W_old"、"A11E_old"、"A11R_old"、"A11T_old")（vscripts/items/Addon_Items/tofar_goback.lua:96-140；
-- vscripts/addon_game_mode.lua:253,356），自動學習清單為空。槽位 W/E/R/T → Q/W/E/R。
-- KV 的 A11E_old 定義了兩次（npc/A_Oda/A11.txt:177 與 :649）；照本專案「重複鍵後者生效」的慣例採第二份（:649），
-- 第一份（與原作說明相符）記在 quirks。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A11_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；Lua modifier 預設不可驅散，寫 "none"。
return function(ctx)
  local UNITS = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "A11_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A11", text = { zh_TW = { name = "齋藤道三（舊版）" }, en = { name = "Saito Dosan (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a11_old_vortex", source_id = "A11W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 180, 200, 220, 240, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 500, unit = "dist", source = "kv:A11W_radius" },
          damage = { 80, 120, 160, 200, unit = "dmg", damage_type = "magical", source = "kv:A11W_damage" },
          building_pct = { 20, unit = "pct", source = "kv:A11W_adjustOnBuilding", note = "KV 寫成倍率 0.2；行為讀到 0.2" },
          duration = { 10, unit = "s", source = "kv:A11W_duration" },
          tick_interval = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A11.lua:56 return 1" },
          ticks = { 10, unit = "count",
            source = "script:vscripts/heroes/A_Oda/A11.lua:19-26 count 從 1 起，count > 0.1+duration 才停：t=0..9 共十跳" },
        },
        text = {
          zh_TW = { name = "妖術．漩渦",
            body = "在目標地點召喚持續 {duration} 秒的漩渦。施放當下起每 {tick_interval} 秒一次、共 {ticks} 次，對範圍 {radius} 內的敵方英雄、一般單位與建築造成 {damage} 魔法傷害；建築只受 {building_pct} 傷害。道三死亡後漩渦照樣造成傷害。" },
          en = { name = "Sorcery: Vortex",
            body = "Summons a vortex at the target point lasting {duration} seconds. Starting on cast, every {tick_interval} seconds for {ticks} pulses, it deals {damage} magic damage to enemy heroes, units and buildings within {radius}; buildings take only {building_pct}. The vortex keeps dealing damage even if Dosan dies." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "repeat", count = "$ticks", interval = "$tick_interval", first = 0, actions = {
              { op = "cue", id = "a11w_vortex", at = "at", duration = "$duration" },
              { op = "cue", id = "a11w_pounce_impact_sound", at = "at" },
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon", "building" } },
                actions = {
                  { op = "if", cond = "alive(caster)",
                    actions = {
                      { op = "damage", to = "it", type = "magical",
                        amount = "select(it.is_building, $damage * $building_pct, $damage)", require_source_alive = false },
                    },
                    otherwise = {
                      { op = "damage", to = "it", type = "magical", require_source_alive = false, credit = "caster",
                        flags = { "no_source_bonuses", "no_amplify" },
                        amount = "select(it.is_building, $damage * $building_pct, $damage)" },
                    } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A11.lua:48-50",
            text = "道三死亡時原作把 attacker 換成漩渦 dummy（同隊、SetOwner 道三），傷害照樣打出、擊殺歸道三；寫法與本體 a11_vortex 相同。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A11.lua:19-26",
            text = "Timer 從 0 秒開始每 1 秒一跳，停止條件 count > 0.1 + duration：t = 0..9 共十跳，第十秒那一跳不發生。" },
          { policy = "as_source", ref = "npc/A_Oda/A11.txt:606",
            text = "搜尋 flags = NONE，預設排除魔免；魔法傷害對魔免歸零屬規則層。舊版 KV 沒寫 SpellImmunityType。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A11.lua:28-33", text = "每跳新建一個持續 duration 秒的粒子，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A11.txt:596-646", "vscripts/heroes/A_Oda/A11.lua:5-58" },
      },
      -------------------------------------------------------------------------
      { id = "a11_old_viper_chain", source_id = "A11E_old", slot = "W", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 25, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A11.txt:659", kv = { 22, 18, 14, 10 },
            source_differs = "A11E_old 定義兩次；照「後者生效」採 :649 那份（第一份 :177 是 22/18/14/10）" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A11.txt:661", kv = { 120, 140, 160, 180 },
            source_differs = "同上，第一份 :190 是 120/140/160/180" },
          cast_range = { 600, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 3, 4, 5, 6, unit = "s", source = "kv:A11E_duration@npc/A_Oda/A11.txt:759", kv = { 2, 2.5, 3, 3.5 },
            source_differs = "同上，第一份 :288 與原作說明是 2.0/2.5/3.0/3.5" },
          damage = { 20, 25, 30, 35, unit = "dmg", damage_type = "magical", source = "kv:A11E_damage@npc/A_Oda/A11.txt:764", kv = { 135 },
            source_differs = "同上，第一份 :293 與原作說明是 135" },
          tick = { 1.05, unit = "s", source = "kv:modifier_A11E ThinkInterval" },
        },
        text = {
          zh_TW = { name = "妖術．冥鎖",
            body = "以冥鎖綁住一名敵方英雄或一般單位並持續施法：目標立即受到 {damage} 魔法傷害並暈眩 {duration} 秒，暈眩期間每 {tick} 秒再受到 {damage} 魔法傷害。中斷施法不會提早解除暈眩。" },
          en = { name = "Sorcery: Nether Chain",
            body = "Binds an enemy hero or unit with a nether chain while channeling: the target immediately takes {damage} magic damage and is stunned for {duration} seconds, taking another {damage} magic damage every {tick} seconds while stunned. Interrupting the channel does not end the stun early." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = UNITS },
        cast = { cast_point = "$cast_point", channel_time = "$duration", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a11e_old_bind", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "冥鎖", body = "暈眩，每 {tick} 秒受到 {damage} 魔法傷害，持續 {duration} 秒。" },
              en = { name = "Nether Chain", body = "Stunned, taking {damage} magic damage every {tick} seconds for {duration} seconds." },
            },
            reveal = "public", states = { "stun" }, cue = "a11e2",
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$damage" },
              } },
            },
            notes = "原作 datadriven modifier_A11E（IsPurgable 1、IsStunDebuff）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a11e_rope", at = "caster", to = "target", duration = "$duration" },
            { op = "apply_status", status = "a11e_old_bind", to = "target" },
            { op = "damage", to = "target", type = "magical", amount = "$damage" },
            { op = "cue", id = "a11e_fatal_bonds_sound", at = "target" },
          },
        },
        coverage = "expressible", confidence = "low",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A11.txt:177,649",
            text = "DOTAAbilities 裡 A11E_old 定義了兩次。照本專案 KV 重複鍵「後者生效」的慣例採 :649（冷卻 25、耗魔 100~160、3~6 秒、每跳 20~35）；原作說明（每秒 135、2.0~3.5 秒）對應第一份 :177。若引擎實際取第一份，只需換數值。" },
          { policy = "as_source", ref = "npc/A_Oda/A11.txt:670-671",
            text = "兩份都呼叫的是本體的 A11E()，不是 A11.lua:153 的 A11E_old()（那個函式沒人呼叫，是死碼）。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A11.lua:65-77",
            text = "FC-116 D22 舊版，比照 D13 修：原作 A11E() 的目的地公式 (caster_pos + target_pos) × distance_percentage / 100 是錯的；舊版又沒有 distance_percentage、讀到 0，目的地變成世界原點，目標被下達走向原點的移動指令並套上 Lua modifier_A11E（無法下指令、固定速度），只剩 KV 的暈眩與傷害。修法照本體 FC-116：目的地改成 lerp_point(目標, 道三, distance_percentage)，舊版的 distance_percentage 是 0，所以目的地就是目標自己的位置、速度 0：目標不再朝原點走，原地被冥鎖暈眩。無法下指令與速度 0 都被同時長的暈眩涵蓋，結束時一起解除，所以資料只寫暈眩與傷害（不另列狀態）。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A11.lua:86",
            text = "A11E() 第 86 行對舊版沒宣告的 modifier_A11E_increase_int 呼叫 ApplyDataDrivenModifier，回傳 nil 再 :SetStackCount 報錯，後面的智力竊取與每 0.2 秒的計時器都不執行；KV 後續的 ApplyModifier、Damage 照常執行。" },
          { policy = "as_source", ref = "npc/A_Oda/A11.txt:673-677,696-712",
            text = "Lua 與 datadriven 兩個 modifier 同名 modifier_A11E。OnChannelInterrupted／Finish 的 RemoveModifier 推定只移除第一個（先加上的 Lua 那個），datadriven 暈眩持續到自己的 Duration，所以中斷施法不會解除暈眩；推定，confidence = low。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A11.lua:71-76",
            text = "原作的移動指令在暈眩結束後是否仍讓目標朝原點走依引擎指令佇列，未實測；FC-116 修正後目的地是目標原地，這個問題不再存在。" },
          { policy = "design_intent", ref = "npc/A_Oda/A11.txt:736",
            text = "ThinkInterval 1.05：3/4/5/6 秒分別兩、三、四、五跳（加上施放時一跳）。" },
          { policy = "no_effect", ref = "npc/A_Oda/A11.txt:716-728",
            text = "modifier_A11E2（施法者身上，施放技能時移除）只給沒人呼叫的 A11E_old() 用，沒有效果，不列入。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A11.lua:78-85", text = "繩索與火焰粒子屬呈現。" },
        },
        provenance = { "npc/A_Oda/A11.txt:176-296", "npc/A_Oda/A11.txt:648-767", "vscripts/heroes/A_Oda/A11.lua:60-151" },
      },
      -------------------------------------------------------------------------
      { id = "a11_old_cunning", source_id = "A11R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          int_bonus = { 10, 20, 30, 40, unit = "stat", source = "kv:A11R_intellectBonus" },
        },
        text = {
          zh_TW = { name = "老奸巨猾", body = "被動：智力增加 {int_bonus}。" },
          en = { name = "Cunning Old Fox", body = "Passive: +{int_bonus} intelligence." },
        },
        target = { type = "none" },
        passive = { reveal = "public", stats = { int = "$int_bonus" } },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A11.txt:770-803",
            text = "舊版只有加智力的被動，沒有本體 A11R 的施法反擊傷害。" },
        },
        provenance = { "npc/A_Oda/A11.txt:769-803" },
      },
      -------------------------------------------------------------------------
      { id = "a11_old_evil_wind", source_id = "A11T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate", "aura" },
        values = {
          cooldown = { 160, 140, 120, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 500, 750, 1000, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          duration = { 15, unit = "s", source = "kv:A11T_duration" },
          move_slow = { 30, 35, 40, unit = "pct", source = "kv:A11T_moveSpeed", kv = { -30, -35, -40 },
            source_differs = "KV 以負值記減益；行為裡取負號" },
          attack_slow = { 30, 60, 90, unit = "atk_speed", source = "kv:A11T_atkSpeed", kv = { -30, -60, -90 },
            source_differs = "KV 以負值記減益；行為裡取負號" },
          aura_radius = { 99999, unit = "dist", hidden = true, source = "kv:modifier_A11T_aura Aura_Radius" },
        },
        text = {
          zh_TW = { name = "禁術．妖風",
            body = "颳起持續 {duration} 秒、遍及全地圖的妖風：所有敵方英雄與一般單位（包含魔免）移動速度降低 {move_slow}，敵方一般單位另外攻擊速度降低 {attack_slow}。道三死亡後妖風照樣持續。" },
          en = { name = "Forbidden Art: Evil Wind",
            body = "Raises an evil wind across the whole map for {duration} seconds: all enemy heroes and units (spell-immune ones included) are slowed by {move_slow}, and enemy non-hero units also lose {attack_slow} attack speed. The wind persists even if Dosan dies." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "a11t_old_slow", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "debuff",
            text = {
              zh_TW = { name = "妖風", body = "移動速度降低 {move_slow}。" },
              en = { name = "Evil Wind", body = "Movement speed reduced by {move_slow}." },
            },
            reveal = "public", cue = "a11t2", stats = { move_speed_pct = "-$move_slow" },
            notes = "原作 modifier_A11T_old（光環子狀態，IsPurgable 0，MOVESPEED_BONUS_PERCENTAGE，不走 nobu 緩速池）。" },
          { id = "a11t_old_attack_slow", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "debuff",
            text = {
              zh_TW = { name = "妖風", body = "攻擊速度降低 {attack_slow}。" },
              en = { name = "Evil Wind", body = "Attack speed reduced by {attack_slow}." },
            },
            reveal = "public", cue = "a11t2", stats = { attack_speed = "-$attack_slow" },
            notes = "原作 modifier_A11T_old2（光環子狀態，只給 DOTA_UNIT_TARGET_BASIC）。" },
          { id = "a11t_old_wind", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            aura = { radius = "$aura_radius", team = "enemy", types = { "hero", "creep", "neutral", "summon" },
              flags = { "include_magic_immune" }, status = "a11t_old_slow" },
            notes = "原作 modifier_A11T_aura（掛在 dummy 上，Duration 15）：移速光環。" },
          { id = "a11t_old_wind_basic", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            aura = { radius = "$aura_radius", team = "enemy", types = { "hero", "creep", "neutral", "summon" },
              flags = { "include_magic_immune" }, where = "not it.is_hero", status = "a11t_old_attack_slow" },
            notes = "原作 modifier_A11T_aura2：攻速光環，只給 DOTA_UNIT_TARGET_BASIC。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a11t_winters_curse_cast_sound", at = "caster" },
            { op = "apply_status", status = "a11t_old_wind", to = "caster" },
            { op = "apply_status", status = "a11t_old_wind_basic", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A11.lua:260-271",
            text = "兩個光環掛在施法位置的 npc_dummy_unit（majia，modifier_kill 15 秒）上，半徑 99999 等於全地圖；道三死亡不影響。資料把兩個光環改掛在道三身上（全地圖光環與中心位置無關，狀態不寫 remove_on death）；若規則層在持有者死亡時停止光環，會與原作不同，需實測。" },
          { policy = "as_source", ref = "npc/A_Oda/A11.txt:844-847,868-871",
            text = "移速光環 Aura_Types = HERO | BASIC，攻速光環只有 BASIC（英雄不吃攻速減益）；兩者都帶 MAGIC_IMMUNE_ENEMIES。" },
          { policy = "as_source", ref = "npc/A_Oda/A11.txt:858",
            text = "移速用 MOVESPEED_BONUS_PERCENTAGE，不經 nobu 緩速池（沒有 tag slow），與緩速池分開疊加。" },
          { policy = "note", ref = "npc/A_Oda/A11.txt:837-871",
            text = "光環子狀態沒寫殘留時間，全地圖光環只有在單位出生或死亡時才會進出，殘留時間不影響，用預設。" },
          { policy = "note", ref = "npc/A_Oda/A11.txt:812",
            text = "KV AbilityUnitDamageType = PURE，但本招沒有傷害。" },
        },
        provenance = { "npc/A_Oda/A11.txt:805-905", "vscripts/heroes/A_Oda/A11.lua:260-271" },
      },
    },
  }
end
