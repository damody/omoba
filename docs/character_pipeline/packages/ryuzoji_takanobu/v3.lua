-- C13 龍造寺隆信（current 版，heromap_skill C13 ["16"] = "WERT"）— package schema v3.1。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/C_Neutral/C13_2.txt:14-19）。current 版用 C13W/E/R/T；C13D_old 與 *_old 是 11 版。
-- heromap_autoskill C13 ["16"] = ""，沒有出生自動學的技能。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，C13_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"（v3.md §12.3 推定）。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
local BASIC = { "hero", "creep", "neutral", "summon" }
local BASIC_BUILDING = { "hero", "creep", "neutral", "summon", "building" }

return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "C13",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "C13",
      text = { zh_TW = { name = "龍造寺隆信" }, en = { name = "Ryuzoji Takanobu" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c13_siphon", source_id = "C13W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 125, 145, 165, 185, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 900, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          stun = { 0.5, unit = "s", source = "script:vscripts/heroes/C_Neutral/C13.lua:16 modifier_stunned duration 0.5" },
          tick_damage = { 100, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
          interval = { 0.5, unit = "s", source = "kv:interval" },
          pull = { 190, unit = "dist", source = "kv:move" },
          count = { 2, 3, 4, 5, unit = "count", source = "kv:count" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C13.lua:23 modifier_phased 0.1" },
        },
        text = {
          zh_TW = { name = "虹吸",
            body = "暈眩目標敵人 {stun} 秒，之後每 {interval} 秒把它朝隆信拉近 {pull} 並造成 {tick_damage} 魔法傷害，共 {count} 次。" },
          en = { name = "Siphon",
            body = "Stuns the target enemy for {stun} seconds, then every {interval} seconds pulls it {pull} toward Takanobu and deals {tick_damage} magic damage, {count} times in total." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c13w_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "虹吸暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Siphon Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" }, notes = "原作 ApplyDataDrivenModifier 引擎內建 modifier_stunned。" },
          { id = "c13w_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, notes = "原作 modifier_phased 0.1 秒。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "c13w_stun", to = "target" },
            { op = "repeat", count = "$count", interval = "$interval", first = "$interval", ["while"] = "exists(target)",
              actions = {
                { op = "damage", to = "target", type = "magical", amount = "$tick_damage" },
                { op = "teleport", who = "target", to = "offset(target, direction(target, caster), $pull)", find_clear_space = false },
                { op = "apply_status", status = "c13w_phased", to = "target" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C13.txt:79-99",
            text = "每跳傷害用 KV dmg 100（各級相同）；AbilityDamage 200/300/400/500 沒有被使用。AoERadius 指向不存在的 %radius。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C13.lua:18-29",
            text = "只檢查 IsValidEntity：目標死亡後屍體仍會被拉；以 SetAbsOrigin 移動、不找空位，距離不足 190 時會越過隆信。傷害走 AMHC，隆信已死時不傷害但照樣拉。" },
          { policy = "note", text = "原作說明寫拉 180；KV move 是 190。" },
        },
        provenance = { "npc/C_Neutral/C13.txt:30-100", "vscripts/heroes/C_Neutral/C13.lua:8-30" },
      },
      -------------------------------------------------------------------------
      { id = "c13_tornado_wave", source_id = "C13E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, 12, 9, 6, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          wave_distance = { 1200, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C13.lua:41 fDistance 1200" },
          wave_radius = { 200, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C13.lua:42-43" },
          wave_speed = { 1000, unit = "speed", hidden = true, source = "script:vscripts/heroes/C_Neutral/C13.lua:51 dir * 1000" },
        },
        text = {
          zh_TW = { name = "龍捲波",
            body = "朝指定方向丟出長 {wave_distance}、寬 {wave_radius} 的龍捲波，貫穿路徑上非魔免的敵方英雄與部隊，各造成 {damage} 魔法傷害。" },
          en = { name = "Tornado Wave",
            body = "Throws a tornado wave {wave_distance} long and {wave_radius} wide in the target direction, passing through non-spell-immune enemy heroes and units and dealing {damage} magic damage to each." },
        },
        target = { type = "point", aoe_radius = "$wave_radius" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_phase_start = { { op = "cue", id = "c13e_vo", at = "caster" } },
          on_cast = {
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$wave_speed", distance = "$wave_distance", radius = "$wave_radius", pierce = true,
              team = "enemy", types = BASIC, cue = "c13e_wind",
              on_hit = { { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false } } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C13.txt:103-171",
            text = "KV 沒寫 AbilityCastRange 與 AbilityCastPoint：任何距離的地點都能指定（只取方向），沒有前搖。datadriven Damage 不檢查施法者存活。" },
          { policy = "note", ref = "npc/C_Neutral/C13.txt:117-134",
            text = "AbilitySpecial dmg／interval／move 與 AbilityUnitTargetTeam FRIENDLY 都沒有被使用。" },
        },
        provenance = { "npc/C_Neutral/C13.txt:102-171", "vscripts/heroes/C_Neutral/C13.lua:32-54" },
      },
      -------------------------------------------------------------------------
      { id = "c13_gale_grip", source_id = "C13R", slot = "E", kind = "passive", max_level = 4, tags = { "basic", "summon" },
        values = {
          str = { 4, 8, 12, 16, unit = "stat", source = "kv:str" },
          move_speed = { 12, 24, 36, 48, unit = "move_speed", source = "kv:move" },
          lifetime = { 15, unit = "s", source = "script:vscripts/heroes/C_Neutral/C13.lua:59 duration = 15" },
          max_winds = { 6, unit = "count", source = "script:vscripts/heroes/C_Neutral/C13.lua:61 C13Rcount < 6" },
          wind_damage = { 70, unit = "dmg", source = "script:vscripts/heroes/C_Neutral/C13.lua:68-69 70 + level*5" },
          wind_damage_per_level = { 5, unit = "dmg", source = "script:vscripts/heroes/C_Neutral/C13.lua:68-69 70 + level*5" },
          wind_search = { 1000, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C13.lua:97-99 FindUnitsInRadius 1000" },
          wind_think = { 1, unit = "s", hidden = true, source = "kv:modifier_C13R2 ThinkInterval" },
          building_pct = { 20, unit = "pct",
            source = "script:vscripts/heroes/C_Neutral/C13.lua:86-91 SetHealth(hp - hp2*0.2)：建築只承受兩成" },
        },
        text = {
          zh_TW = { name = "狂嵐之握",
            body = "被動：力量 +{str}、移動速度 +{move_speed}。每當隆信或他的龍捲風擊殺一個單位，在隆信身旁召喚一個無法控制、無敵的{@c13r_wind}，存在 {lifetime} 秒，同時最多 {max_winds} 個。龍捲風攻擊力為 {wind_damage} 加隆信等級 × {wind_damage_per_level}，會自動追擊 {wind_search} 內最近、看得見的敵人（包含建築）；對建築只造成 {building_pct} 傷害。" },
          en = { name = "Gale Grip",
            body = "Passive: +{str} Strength and +{move_speed} movement speed. Whenever Takanobu or one of his tornadoes kills a unit, an uncontrollable, invulnerable {@c13r_wind} appears beside Takanobu for {lifetime} seconds, up to {max_winds} at a time. A tornado's attack damage is {wind_damage} plus Takanobu's level × {wind_damage_per_level}; it chases the nearest visible enemy within {wind_search} (buildings included) and deals only {building_pct} damage to buildings." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public", visibility = "buff",
          stats = { str = "$str", move_speed = "$move_speed" },
          triggers = {
            on_kill = { { op = "call", routine = "c13r_spawn" } },
          },
        },
        statuses = {
          { id = "c13r_wind_ai", duration = "$lifetime", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "unselectable", "invulnerable", "no_unit_collision" },
            triggers = {
              periodic = { interval = "$wind_think", actions = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$wind_search", team = "enemy", types = BASIC_BUILDING,
                    flags = { "visible_only", "include_magic_immune" }, order = "nearest", limit = "1" },
                  actions = {
                    { op = "issue_order", who = "self", order = "attack", target = "it", force = true },
                  } },
              } },
              on_kill = { { op = "call", routine = "c13r_spawn" } },
              on_deal_damage_pre = {
                filter = "event.victim.is_building and event.damage.is_attack",
                actions = { { op = "modify_damage", multiplier = "$building_pct" } } },
            },
            notes = "原作 modifier_C13R2（ThinkInterval 1、UNSELECTABLE、OnKill、OnAttackLanded）＋ modifier_invulnerable、modifier_kill 15 秒、for_no_collision。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C13.lua:56-78",
            text = "datadriven OnKill：擊殺任何單位都觸發（包含反補友軍與小兵）；龍捲風身上的 modifier_C13R2 也有 OnKill，風殺死單位也會再召喚。上限以 caster.C13Rcount 計，15 秒後才扣回；風無敵、一定活滿 15 秒，等於同時存在的數量上限。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C13.lua:68-69",
            text = "攻擊力在召喚當下以隆信等級設定（SetBaseDamageMin/Max），之後升級不變。" },
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C13.lua:80-93",
            text = "對建築是在 OnAttackLanded 記下生命，0.01 秒後 SetHealth 回補八成：那 0.01 秒內其他來源的傷害也會被回補八成，致死的一擊則來不及回補。照設計意圖寫成只把風對建築的攻擊傷害乘兩成。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C13.lua:95-115",
            text = "每秒對 1000 內最近、看得見的敵人（含建築與魔免）下攻擊命令並 SetForceAttackTarget；沒有目標時取消強制目標，交給一般索敵（AttackAcquisitionRange 1500）。CanEntityBeSeenByMyTeam 的分支因 FOW_VISIBLE 已篩過而永遠不成立。" },
          { policy = "note", ref = "npc/C_Neutral/C13.txt:184-190",
            text = "被動的 AbilityCooldown／ManaCost／AbilityDamage／CastRange／AoERadius 沒有被使用；原作說明寫「基礎跑速」，KV 是 MOVESPEED_BONUS_CONSTANT。" },
        },
        provenance = { "npc/C_Neutral/C13.txt:173-280", "vscripts/heroes/C_Neutral/C13.lua:56-115",
          "npc/npc_units_custom_load.txt:8-72", "npc/npc_abilities_custom_load.txt:5247-5263" },
      },
      -------------------------------------------------------------------------
      { id = "c13_tornado_vortex", source_id = "C13T", slot = "R", kind = "channel", max_level = 3, tags = { "ultimate", "channel" },
        values = {
          cooldown = { 130, 120, 110, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 300, 400, 500, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          channel_time = { 5.5, unit = "s", source = "kv:AbilityChannelTime" },
          required_level = { 6, 12, 18, unit = "level", source = "assumed:KV 沒寫 RequiredLevel；DOTA_ABILITY_TYPE_ULTIMATE 預設 6/12/18" },
          radius = { 500, unit = "dist", source = "kv:radius" },
          tick_damage = { 28, 40, 52, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
          tick = { 0.1, unit = "s", source = "script:vscripts/heroes/C_Neutral/C13.lua:173-175 tsum += 0.1" },
          tick_count = { 55, unit = "count", hidden = true, source = "script:vscripts/heroes/C_Neutral/C13.lua:174 tsum < 5.5" },
          pull = { 50, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C13.lua:145 dir*50" },
          swirl = { 100, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C13.lua:142-145 right*100" },
          right_angle = { 90, unit = "deg", hidden = true, source = "script:vscripts/heroes/C_Neutral/C13.lua:142-144 right = (dir.y, -dir.x)" },
          hold = { 0.2, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C13.lua:146 {duration = 0.2}" },
          item_lock = { 0.3, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C13.lua:154 item:StartCooldown(0.3)" },
        },
        text = {
          zh_TW = { name = "龍捲之漩渦",
            body = "持續施法，以自身為漩渦中心：每 {tick} 秒把周圍 {radius} 內的敵方英雄與部隊（包含魔免）朝隆信拉近 {pull} 並橫向捲動 {swirl}，使其定身、沉默、繳械、不能閃避，並造成 {tick_damage} 魔法傷害。" },
          en = { name = "Tornado Vortex",
            body = "Channels as the eye of a vortex: every {tick} seconds, enemy heroes and units within {radius} (spell-immune ones included) are pulled {pull} toward Takanobu and swept {swirl} sideways, rooted, silenced, disarmed and unable to evade, and take {tick_damage} magic damage." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "c13t_hold", duration = "$hold", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "龍捲之漩渦", body = "被捲入漩渦：定身、沉默、繳械，且無法閃避攻擊。" },
              en = { name = "Tornado Vortex", body = "Caught in the vortex: rooted, silenced, disarmed and unable to evade attacks." },
            },
            reveal = "public", states = { "root", "evade_disabled", "silence", "disarm", "no_unit_collision" },
            notes = "原作 modifier_C13T（KV 寫 IsBuff 1，但掛在敵人身上）：ROOTED、EVADE_DISABLED、SILENCED、DISARMED，每跳 0.2 秒；另加 modifier_phased 0.2 秒（no_unit_collision）。沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c13t_title" },
            { op = "cue", id = "c13t_vortex", attach = "caster" },
            { op = "repeat", count = "$tick_count", interval = "$tick", first = "0", ["while"] = "caster.is_channeling",
              actions = {
                { op = "for_each",
                  select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                    flags = { "include_magic_immune", "include_untargetable" } },
                  actions = {
                    { op = "if", cond = "not has_state(it, untargetable_by_spells)", actions = {
                      { op = "teleport", who = "it", find_clear_space = false,
                        to = "offset(offset(it, direction(it, caster), $pull), rotate(direction(it, caster), -$right_angle), $swirl)" },
                      { op = "apply_status", status = "c13t_hold", to = "it" },
                      { op = "if", cond = "it.is_hero and not it.is_illusion", actions = {
                        { op = "modify_cooldown", of = "it", item_tag = "blink", mode = "start", amount = "$item_lock" },
                      } },
                    } },
                    { op = "damage", to = "it", type = "magical", amount = "$tick_damage", require_source_alive = false },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C13.lua:138-167",
            text = "_G.EXCLUDE_TARGET_NAME 的單位不被拉、不被定身，但 ApplyDamage 在判斷外面，照樣受傷；v3 以 include_untargetable 選進來、只對一般單位拉動。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C13.lua:141-145",
            text = "每跳以 SetAbsOrigin 朝隆信移 50、往右（順時針切線）移 100，不找空位：敵人繞著隆信打轉並慢慢靠近，貼近時會越過中心。" },
          { policy = "as_source", ref = "npc/C_Neutral/C13.txt:296-320",
            text = "每跳傷害是 dmg 28/40/52（每 0.1 秒），AbilityDamage 280/400/520 是每秒量、沒有被使用；全程共約 1540/2200/2860。ApplyDamage 不檢查施法者存活。" },
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C13.lua:125-179",
            text = "第一跳在施放當下，之後每 0.1 秒、只要仍在引導；tsum 以浮點累加，第 56 跳是否落在 5.5 秒依引擎時序。照設計意圖 55 跳。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C13.lua:148-157",
            text = "對英雄（非分身）身上的 item_magic_ring、item_flash_ring、item_flash_shoes 每跳 StartCooldown(0.3)，等於漩渦中不能閃現；只看六格背包。不檢查 IsCooldownReady，正在冷卻的閃現道具也被改成 0.3 秒（冷卻反而縮短）。資料用 modify_cooldown 的 item_tag = blink：閃現戒指、閃現鞋的道具文件要給它們道具 tag blink；item_magic_ring 在原作 KV 不存在（batch_I2 deferred），這個名稱永遠不成立。" },
          { policy = "note", ref = "npc/C_Neutral/C13.txt:283-299",
            text = "KV 沒寫 RequiredLevel／MaxLevel；ultimate 類型推定為 6/12/18、三級。AbilityCastRange 500 只是指示，等於 radius。" },
        },
        provenance = { "npc/C_Neutral/C13.txt:282-352", "vscripts/heroes/C_Neutral/C13.lua:1-6", "vscripts/heroes/C_Neutral/C13.lua:117-180" },
      },
    },
    unit_templates = {
      { id = "c13r_wind", source_id = "c13r_wind",
        values = {
          hp = { 1000, unit = "hp", source = "kv:StatusHealth@npc/npc_units_custom_load.txt:9" },
          armor = { 1000, unit = "armor", source = "kv:ArmorPhysical@npc/npc_units_custom_load.txt:9" },
          magic_resist = { 100, unit = "pct", source = "kv:MagicalResistance@npc/npc_units_custom_load.txt:9" },
          move_speed = { 250, unit = "move_speed", source = "kv:MovementSpeed@npc/npc_units_custom_load.txt:9" },
          attack_range = { 128, unit = "dist", source = "kv:AttackRange@npc/npc_units_custom_load.txt:9" },
          attack_interval = { 0.6, unit = "s", source = "kv:AttackRate@npc/npc_units_custom_load.txt:9" },
          attack_damage = { 100, unit = "dmg", hidden = true, source = "kv:AttackDamageMin@npc/npc_units_custom_load.txt:9",
            note = "召喚時以 70 + 等級 × 5 覆寫" },
          acquisition = { 1500, unit = "dist", source = "kv:AttackAcquisitionRange@npc/npc_units_custom_load.txt:9" },
          vision = { 500, unit = "dist", source = "kv:VisionDaytimeRange@npc/npc_units_custom_load.txt:9" },
        },
        text = { zh_TW = { name = "龍捲風" }, en = { name = "Tornado" } },
        stats = { hp = "$hp", armor = "$armor", magic_resist = "$magic_resist", move_speed = "$move_speed" },
        attack = { kind = "melee", range = "$attack_range", interval = "$attack_interval", damage = "$attack_damage", attack_point = "0" },
        vision = { day = "$vision", night = "$vision" },
        acquisition_range = "$acquisition",
        ai = "idle", tags = { "summon" },
        notes = "npc_dota_creature，Level 2、Bounty 100（無敵，實際拿不到）、MovementTurnRate 3、HasInventory 0；模型與粒子屬呈現。",
        provenance = { "npc/npc_units_custom_load.txt:8-72" },
      },
    },
    routines = {
      c13r_spawn = {
        owner = "c13_gale_grip",
        actions = {
          { op = "create_unit", template = "c13r_wind", at = "position(caster_hero)", owner = "caster_hero", control = "none",
            lifetime = "$lifetime", limit_per_owner = "$max_winds", over_limit = "reject", as = "u",
            overrides = {
              attack_damage = "$wind_damage + caster_hero.level * $wind_damage_per_level",
              attack_damage_max = "$wind_damage + caster_hero.level * $wind_damage_per_level",
            } },
          { op = "if", cond = "exists(u)", actions = {
            { op = "apply_status", status = "c13r_wind_ai", to = "u", source = "caster_hero" },
            { op = "cue", id = "c13r_wind_fx", attach = "u", duration = "$lifetime" },
          } },
        },
      },
    },
  }
end
