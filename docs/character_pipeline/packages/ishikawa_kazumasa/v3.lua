-- A03 石川數正（current 版，heromap_version["16"] = "WERT"，沒有 D）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/A_Oda/A03_2.txt:21-26；vscripts/addon_game_mode.lua:245）。
-- 開場不自動學技能（heromap_autoskill A03 ["16"] = ""，addon_game_mode.lua:348）。
-- A03W_old／A03E_old／A03R_old／A03T_old／A03TW_old 是 11.2b 舊版，不在 current 版，不列入。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A03_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"（§12.3 推定）。
-- datadriven 的 Damage 動作不經 AMHC:Damage，施法者死亡後仍會造成傷害，寫 require_source_alive = false（同 C17）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A03",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "A03",
      text = { zh_TW = { name = "石川數正" }, en = { name = "Ishikawa Kazumasa" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a03_pacing_gu", source_id = "A03W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, 18, 16, 14, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 140, 160, 180, 200, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          projectile_speed = { 1200, unit = "speed", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A03.lua:14 iMoveSpeed = 1200" },
          duration = { 10, unit = "s", source = "kv:duration", note = "A03.lua:89-90 以 {duration} 套用" },
          attack_speed = { -30, -50, -70, -90, unit = "atk_speed", source = "kv:attack_speed" },
          trigger_len = { 500, 400, 300, 200, unit = "dist", source = "kv:triggerlen" },
          move_damage = { 100, unit = "dmg", damage_type = "pure", source = "kv:dmg", note = "A03.lua:30,39" },
          tick_damage = { 25, unit = "dmg", damage_type = "magical",
            source = "kv:Damage@npc/A_Oda/A03.txt:114", note = "modifier_A03W_debuff OnIntervalThink 的 Damage" },
          tick = { 1, unit = "s", source = "kv:modifier_A03W_debuff ThinkInterval" },
          recheck = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A03.lua:92-104 Timers 0.1" },
        },
        text = {
          zh_TW = { name = "計步蠱",
            body = "射出追蹤蠱蟲，命中敵人後寄生 {duration} 秒：目標攻擊速度 {attack_speed}，每 {tick} 秒受到 {tick_damage} 魔法傷害；目標每累計移動 {trigger_len} 距離，受到 {move_damage} 乘以（一減目標基礎魔抗）的純粹傷害。英雄目標在寄生期間獲得魔免而使蠱被移除時，會以剩餘時間重新寄生。" },
          en = { name = "Pacing Gu",
            body = "Fires a homing gu that infests the enemy it hits for {duration} seconds: the target's attack speed changes by {attack_speed}, and it takes {tick_damage} magic damage every {tick} seconds. Each time the target moves a total of {trigger_len} distance, it takes {move_damage} × (one minus its base magic resistance) pure damage. If a hero target becomes spell immune and the gu is removed, it re-infests for the remaining time." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a03w_debuff", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "計步蠱", body = "攻擊速度 {attack_speed}，每 {tick} 秒受到 {tick_damage} 魔法傷害；每累計移動 {trigger_len} 距離，受到以 {move_damage} 為基礎的純粹傷害。持續 {duration} 秒。" },
              en = { name = "Pacing Gu", body = "Attack speed changed by {attack_speed}; takes {tick_damage} magic damage every {tick} seconds, and pure damage based on {move_damage} for every {trigger_len} distance moved. Lasts {duration} seconds." },
            },
            reveal = "public", cue = "a03w_poison_viper",
            stats = { attack_speed = "$attack_speed" },
            vars = { moved = { type = "fixed", init = "0", reveal = "public" } },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$tick_damage", require_source_alive = false },
                { op = "cue", id = "a03w_tick_sound", at = "self" },
              } },
              on_moved = { {
                op = "let", name = "total", value = "vars.moved + event.distance" }, {
                op = "if", cond = "total > $trigger_len",
                actions = {
                  { op = "damage", to = "self", type = "pure", amount = "$move_damage * (1 - self.base_magic_resist)" },
                  { op = "set_var", var = "moved", value = "total - $trigger_len" },
                },
                otherwise = { { op = "set_var", var = "moved", value = "total" } } },
              },
            },
            notes = "原作 modifier_A03W_debuff：ATTACKSPEED_BONUS_CONSTANT；OnUnitMoved 以 target.lastpos／A03W_len（Lua 欄位）累計移動距離，OnDestroy 清除。沒寫 IsPurgable，§12.3 暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = true, cue = "a03w_projectile",
              on_hit = {
                { op = "let", name = "victim", value = "it" },
                { op = "apply_status", status = "a03w_debuff", to = "victim" },
                -- A03.lua:91-104：每 0.1 秒檢查；英雄目標沒有寄生且正處於魔免時，以 duration − tsum 重新套上
                { op = "repeat", interval = "$recheck", first = "$recheck", count = "$duration / $recheck", as = "n",
                  ["while"] = "alive(victim) and victim.is_hero", actions = {
                    { op = "if", cond = "not has_status(victim, a03w_debuff) and victim.is_magic_immune", actions = {
                      { op = "apply_status", status = "a03w_debuff", to = "victim", duration = "$duration - n * $recheck" },
                    } },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A03.lua:36-41",
            text = "移動距離以每次 OnUnitMoved 的位置差累計，一次事件最多觸發一次傷害並只扣一個 triggerlen；位移、閃現、傳送造成的位置變化也算移動距離（推定 event.distance 同樣計入）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A03.lua:31,39",
            text = "移動傷害是純粹傷害乘上 (1 − base 魔抗)：只看 nobu 魔抗池，裝備魔抗不算；魔免單位照樣受傷。經 AMHC:Damage，石川數正死亡後不造成移動傷害（累計照常扣除）。" },
          { policy = "as_source", ref = "npc/A_Oda/A03.txt:107-115",
            text = "持續傷害寫死 25（不隨等級），是 datadriven Damage，不經 AMHC，施法者死亡後仍會造成；對魔免單位由規則層歸零。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A03.lua:91-104",
            text = "0.1 秒的檢查只在「英雄、存活、沒有寄生、正處於魔免」時重新套上，等於蠱被魔免驅散後以剩餘時間補回；一般驅散後若之後才獲得魔免，也會補回。非英雄目標第一次檢查就停止。重新套上的時間是 duration − tsum（tsum 落後實際經過時間一個 0.1 秒），照實寫成 duration − n × 0.1。" },
          { policy = "as_source", text = "重新套上是新的實例，累計移動距離歸零（原作 OnDestroy 清除 A03W_len）；同一實例被再次施放刷新時累計保留。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A03.lua:32-35",
            text = "套上後第一次 OnUnitMoved 只記錄位置、不累計，差異小於一步。" },
          { policy = "note", ref = "npc/A_Oda/A03.txt:35",
            text = "KV AbilityDamage 50/80/110/140 與 AbilityUnitDamageType PURE 沒有被使用；沒有 AbilityCastPoint（引擎預設）。" },
          { policy = "design_intent", ref = "npc/A_Oda/A03.txt:107",
            text = "ThinkInterval 1 搭配 10 秒持續，第十跳是否在到期當下執行依引擎時序；照設計意圖以 periodic 表達。" },
        },
        provenance = { "npc/A_Oda/A03.txt:20-148", "vscripts/heroes/A_Oda/A03.lua:2-52", "vscripts/heroes/A_Oda/A03.lua:85-105",
          "vscripts/amhc_library/amhc.lua:535-584" },
      },
      -------------------------------------------------------------------------
      { id = "a03_spider_form", source_id = "A03E", slot = "W", kind = "toggle", max_level = 4, tags = { "basic", "toggle" },
        values = {
          cooldown = { 7, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 130, 150, 170, 190, unit = "mana", source = "kv:AbilityManaCost" },
          move_speed = { 38, unit = "move_speed", source = "kv:MODIFIER_PROPERTY_MOVESPEED_BONUS_CONSTANT@npc/A_Oda/A03.txt:215" },
          base_attack_time = { 1.3, unit = "s", source = "kv:MODIFIER_PROPERTY_BASE_ATTACK_TIME_CONSTANT@npc/A_Oda/A03.txt:216" },
          armor = { 0, 3, 6, 9, unit = "armor", source = "kv:armor" },
          hp = { 225, 300, 375, 450, unit = "hp", source = "kv:hp" },
          attack_range = { -470, unit = "dist", source = "kv:MODIFIER_PROPERTY_ATTACK_RANGE_BONUS@npc/A_Oda/A03.txt:219" },
          lifesteal = { 40, unit = "pct", source = "kv:StealPercent@npc/A_Oda/A03.txt:229" },
        },
        text = {
          zh_TW = { name = "蜘蛛化型",
            body = "開啟時化為蜘蛛形態並改為近戰攻擊：移動速度 +{move_speed}、基礎攻擊間隔變為 {base_attack_time} 秒、護甲 +{armor}、最大生命 +{hp}、攻擊距離 {attack_range}。普攻命中非建築目標時，回復該次攻擊傷害的 {lifesteal}，扣除比例只依目標的額外護甲計算。關閉時恢復遠程形態。" },
          en = { name = "Spider Form",
            body = "While active, Kazumasa turns into a spider and attacks in melee: +{move_speed} movement speed, base attack time becomes {base_attack_time} seconds, +{armor} armor, +{hp} maximum health and {attack_range} attack range. Attacks that land on non-building targets heal for {lifesteal} of the attack damage, reduced only by the target's bonus armor. Turning it off restores his ranged form." },
        },
        target = { type = "none" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a03e_spider", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "蜘蛛化型", body = "蜘蛛形態：移動速度 +{move_speed}、基礎攻擊間隔變為 {base_attack_time} 秒、護甲 +{armor}、最大生命 +{hp}、攻擊距離 {attack_range}；普攻命中非建築目標時回復攻擊傷害的 {lifesteal}。持續到關閉為止。" },
              en = { name = "Spider Form", body = "Spider form: +{move_speed} movement speed, base attack time becomes {base_attack_time} seconds, +{armor} armor, +{hp} maximum health and {attack_range} attack range; attacks on non-building targets heal for {lifesteal} of the damage. Lasts until toggled off." },
            },
            reveal = "public", cue = "a03e_spider_model", remove_on = { "death" },
            stats = {
              move_speed = "$move_speed",
              base_attack_time = { op = "set_absolute", value = "$base_attack_time" },
              armor = "$armor", max_hp = "$hp", attack_range = "$attack_range",
            },
            triggers = {
              on_attack_hit = { filter = "not event.target.is_building and not has_status_tag(event.target, isvoid)", actions = {
                { op = "heal", to = "self",
                  amount = "event.attack.damage * $lifesteal * armor_multiplier(event.target.bonus_armor)" },
                { op = "cue", id = "generic_lifesteal", at = "self" },
              } },
              -- FC-102（D13）：開著死亡時關掉開關並還原成遠程
              on_death = {
                { op = "remove_status", from = "self", status = "a03e_melee" },
                { op = "toggle_ability", of = "self", ability = "a03_spider_form", on = false },
              },
            },
            notes = "原作 modifier_A03E_buff：MOVESPEED_BONUS_CONSTANT 38、BASE_ATTACK_TIME_CONSTANT 1.3、PHYSICAL_ARMOR_BONUS、HEALTH_BONUS、ATTACK_RANGE_BONUS −470；OnAttackLanded 呼叫 item_muramasa_katana.lua StealLife2（StealPercent 40）。沒寫 IsPurgable，§12.3 暫定 basic。" },
          { id = "a03e_melee", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", scope = "hero", states = { "melee_attack" },
            notes = "原作 A03.lua:60 SetAttackCapability(DOTA_UNIT_CAP_MELEE_ATTACK)、:71 改回 RANGED：不是 modifier，原作死亡不還原（FC-102 修正後死亡時移除並關掉開關），關閉開關時改回（v3.1 狀態旗標 melee_attack；scope = hero 讓它跨死亡保留）。" },
        },
        events = {
          on_toggle_on = {
            { op = "apply_status", status = "a03e_spider", to = "caster" },
            { op = "apply_status", status = "a03e_melee", to = "caster" },
            { op = "cue", id = "a03e_model_nian_creep", at = "caster" },
          },
          on_toggle_off = {
            { op = "remove_status", from = "caster", status = "a03e_spider" },
            { op = "remove_status", from = "caster", status = "a03e_melee" },
            { op = "cue", id = "a03e_model_a03", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_muramasa_katana.lua:109-137",
            text = "吸血自己算護甲減免：GetPhysicalArmorValue(true)（bIgnoreBase，推定只取額外護甲）帶入 0.06a/(1+0.06|a|)，所以照原作用 armor_multiplier(bonus_armor)；目標帶 isvoid（跨文件 tag）或是建築時不吸血。%attack_damage 推定為減免前的攻擊傷害。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A03.lua:58-73",
            text = "FC-102（D13 修正：開著死亡時關掉開關並還原成遠程（on_death））。原作：開關開著死亡時，datadriven 狀態隨死亡移除，但 SetAttackCapability(近戰) 與模型不會還原、開關仍亮著：復活後是攻擊距離回到遠程數值的近戰單位且沒有任何加成，直到手動關掉再開（推測，未在 Dota 實測）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A03.lua:62,72",
            text = "開啟時模型縮放 0.5、關閉時設為 1.4，但英雄 KV ModelScale 是 1.2：關過一次後模型比原本大，屬呈現。" },
          { policy = "note",
            text = "開關的耗魔與冷卻在開與關時是否都發生依引擎；推定開啟時扣魔、開與關都進冷卻（cooldown_start = on_cast）。" },
          { policy = "as_source", ref = "npc/A_Oda/A03.txt:239",
            text = "一級護甲加成為零。最大生命加成套上時目前生命是否同比例增加依引擎。" },
        },
        provenance = { "npc/A_Oda/A03.txt:150-247", "vscripts/heroes/A_Oda/A03.lua:54-73",
          "vscripts/items/Addon_Items/item_muramasa_katana.lua:109-137" },
      },
      -------------------------------------------------------------------------
      { id = "a03_enchanting_venom", source_id = "A03R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          tick_damage = { 15, 25, 35, 45, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
          attack_speed = { -20, -35, -50, -65, unit = "atk_speed", source = "kv:spd" },
          duration = { 4, unit = "s", source = "kv:duration", note = "A03.lua:79-81 以 {duration} 套用" },
          tick = { 1, unit = "s", source = "kv:modifier_A03R_debuff ThinkInterval" },
        },
        text = {
          zh_TW = { name = "蠱惑術",
            body = "被動：普攻命中非建築目標時使其中毒 {duration} 秒：攻擊速度 {attack_speed}，每 {tick} 秒受到 {tick_damage} 魔法傷害。" },
          en = { name = "Enchanting Venom",
            body = "Passive: attacks that land on non-building targets poison them for {duration} seconds: attack speed changes by {attack_speed}, and they take {tick_damage} magic damage every {tick} seconds." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = { filter = "not event.target.is_building", actions = {
              { op = "apply_status", status = "a03r_poison", to = "event.target" },
            } },
          },
        },
        statuses = {
          { id = "a03r_poison", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "蠱惑術", body = "中毒：攻擊速度 {attack_speed}，每 {tick} 秒受到 {tick_damage} 魔法傷害，持續 {duration} 秒。" },
              en = { name = "Enchanting Venom", body = "Poisoned: attack speed changed by {attack_speed} and takes {tick_damage} magic damage every {tick} seconds for {duration} seconds." },
            },
            reveal = "public", cue = "a03r_poison_viper",
            stats = { attack_speed = "$attack_speed" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$tick_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_A03R_debuff：ATTACKSPEED_BONUS_CONSTANT、ThinkInterval 1 的 datadriven Damage。沒寫 IsPurgable，§12.3 暫定 basic。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A03.lua:75-83",
            text = "只排除建築；OnAttackLanded 對任何被命中的單位都套用，包括反補的友軍單位。魔免單位也會被套上（攻速降低照常），魔法傷害由規則層歸零。" },
          { policy = "note", ref = "npc/A_Oda/A03.txt:277",
            text = "modifier 的 Duration 寫 %debuff_duration，但 AbilitySpecial 沒有這個鍵；腳本以 {duration = 4} 覆寫，所以不影響。" },
          { policy = "as_source", text = "持續傷害是 datadriven Damage，不經 AMHC，石川數正死亡後仍會造成。" },
          { policy = "design_intent", ref = "npc/A_Oda/A03.txt:295",
            text = "ThinkInterval 1 搭配 4 秒持續，第四跳是否在到期當下執行、刷新時計時是否重來依引擎時序；照設計意圖以 periodic 表達。" },
        },
        provenance = { "npc/A_Oda/A03.txt:249-326", "vscripts/heroes/A_Oda/A03.lua:75-83" },
      },
      -------------------------------------------------------------------------
      { id = "a03_insect_swarm", source_id = "A03T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 15, 12, 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 150, 200, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          damage = { 500, 750, 1000, unit = "dmg", damage_type = "magical", source = "kv:A03T_old_damage" },
          speed = { 800, unit = "speed", source = "kv:shock_speed" },
          distance = { 1200, unit = "dist", source = "kv:shock_distance" },
          start_width = { 300, unit = "dist", source = "kv:shock_width_start" },
          end_width = { 400, unit = "dist", source = "kv:shock_width" },
        },
        text = {
          zh_TW = { name = "蟲嗜群列",
            body = "朝目標方向放出蟲群，以 {speed} 速度前進 {distance} 距離，寬度由 {start_width} 擴大到 {end_width}，對經過的非魔免敵方英雄與部隊各造成一次 {damage} 魔法傷害。" },
          en = { name = "Insect Swarm",
            body = "Releases a swarm in the target direction that travels {distance} at speed {speed}, widening from {start_width} to {end_width}, dealing {damage} magic damage once to each non-spell-immune enemy hero and unit it passes." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        events = {
          on_cast = {
            { op = "cue", id = "a03t_swarm_sound", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$speed", distance = "$distance", radius = "$start_width", end_radius = "$end_width",
              pierce = true, team = "enemy", types = { "hero", "creep", "neutral", "summon" }, cue = "a03t_swarm",
              on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "cue", id = "a03t_swarm_hit_sound", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A03.txt:356",
            text = "固定飛行 1200 距離，與施法距離 800 無關；只決定方向。" },
          { policy = "as_source", ref = "npc/A_Oda/A03.txt:373-378",
            text = "傷害是 datadriven Damage（不經 AMHC），石川數正在蟲群飛行中死亡仍會造成傷害。TargetFlags NONE：魔免敵人不被命中，隱形與迷霧中的敵人照樣命中（選取預設）。" },
          { policy = "note", ref = "npc/A_Oda/A03.txt:360",
            text = "HasFrontalCone 與 StartRadius／EndRadius 表示判定範圍沿路徑線性變寬，對應 radius → end_radius。ProvidesVision 0，不給視野。" },
          { policy = "note", ref = "npc/A_Oda/A03.txt:344,391",
            text = "AbilityDamage 與鍵名 A03T_old_damage 數值相同；鍵名沿用舊版，只是命名。" },
        },
        provenance = { "npc/A_Oda/A03.txt:328-414" },
      },
    },
  }
end
