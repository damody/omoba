-- A23 宇喜多秀家（current 版，heromap_version["16"] = "WERT"，沒有 D，autoskill 空）— package schema v3。
-- 原作 11 版不存在（heromap_version A23 ["11"] = false）。原作槽位 W/E/R/T → Q/W/E/R。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A23_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"；IsPurgable 0 寫 "none"。
-- 原作 ApplyDamage 與 datadriven Damage 不經 AMHC，攻擊者死亡仍造成傷害（照 C17 寫 require_source_alive = false）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A23",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "A23",
      text = { zh_TW = { name = "宇喜多秀家" }, en = { name = "Ukita Hideie" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a23_cursed_blood", source_id = "A23W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 19, 16, 13, 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 110, 130, 150, 170, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          delay = { 2, unit = "s", source = "script:vscripts/heroes/A_Oda/A23.lua:15,33 Timers:CreateTimer(2)" },
          radius = { 300, unit = "dist", source = "kv:A23W_radius" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:A23W_damage" },
          stun = { 1, 1.5, 2, 2.5, unit = "s", source = "kv:A23W_stun" },
        },
        text = {
          zh_TW = { name = "鬼術．惡血咒",
            body = "將惡血咒注入目標單位（敵我皆可），{delay} 秒後在目標當時的位置爆發，對周圍 {radius} 內的敵方英雄與一般單位造成 {damage} 魔法傷害並暈眩 {stun} 秒。若施放當下目標身上有{@a23e_bind}，爆發也會波及魔法免疫的敵人（只受暈眩，魔法傷害無效）。" },
          en = { name = "Demon Art: Cursed Blood",
            body = "Injects cursed blood into a target unit (ally or enemy). After {delay} seconds it bursts at the target's current position, dealing {damage} magic damage to enemy heroes and units within {radius} and stunning them for {stun} seconds. If the target had {@a23e_bind} when the spell was cast, the burst also hits spell-immune enemies (they are stunned but take no magic damage)." },
        },
        target = { type = "unit", range = "$cast_range", team = "both", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a23w_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被惡血咒的爆發擊暈，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Stunned by the Cursed Blood burst for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作是引擎的 modifier_stunned（AddNewModifier），不在 ultimate_modifiers，受 nobu 狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "marked", value = "has_status(target, a23e_bind)" },
            { op = "cue", id = "a23w_inject", at = "target" },
            { op = "delay", time = "$delay", capture = { "marked" }, actions = {
              { op = "cue", id = "a23w_burst", at = "target" },
              { op = "if", cond = "marked",
                actions = {
                  { op = "for_each",
                    select = { from = "target", shape = "circle", radius = "$radius", team = "enemy",
                      types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                    actions = {
                      { op = "apply_status", status = "a23w_stun", to = "it" },
                      { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                    } },
                },
                otherwise = {
                  { op = "for_each",
                    select = { from = "target", shape = "circle", radius = "$radius", team = "enemy",
                      types = { "hero", "creep", "neutral", "summon" } },
                    actions = {
                      { op = "apply_status", status = "a23w_stun", to = "it" },
                      { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                    } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A23.lua:14",
            text = "是否有靈魂鎖縛在施放當下判定（不是爆發時）；任何來源的 modifier_A23E 都算。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A23.lua:19,37",
            text = "爆發中心是兩秒後目標的當下位置（target:GetOrigin() 在 Timer 內才讀）；目標可以是友軍，爆發只傷敵人。" },
          { policy = "as_source", ref = "vscripts/library/events/eventfordamage.lua:30-32",
            text = "有鎖縛時選取帶 MAGIC_IMMUNE_ENEMIES：魔免敵人被暈眩，但魔法傷害被傷害過濾器歸零（規則層）。原作說明「震暈無視魔免」與此一致。" },
          { policy = "as_source",
            text = "傷害用 ApplyDamage，不經 AMHC：秀家在兩秒內死亡仍會爆發並造成傷害。目標英雄死亡時以屍體位置爆發；目標是已被移除的一般單位時 Timer 報錯、不爆發（v3 執行器以最後位置爆發，差異只在此邊界）。" },
          { policy = "note", ref = "npc/A_Oda/A23.txt:27",
            text = "KV TEAM_BOTH：能否以自己為目標無法判定，未加 self_cast_allowed。" },
          { policy = "note", text = "FindUnitsInRadius 沒有 FOW 旗標，隱形與迷霧中的敵人也會被波及（selector 預設）。" },
        },
        provenance = { "npc/A_Oda/A23.txt:20-82", "vscripts/heroes/A_Oda/A23.lua:4-51",
          "vscripts/library/events/eventfordamage.lua:30-32", "vscripts/events.lua:246-268" },
      },
      -------------------------------------------------------------------------
      { id = "a23_soul_bind", source_id = "A23E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 23, 20, 17, 14, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 600, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 3, 4, 5, 6, unit = "s", source = "kv:A23E_duration" },
          tick = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A23.lua:75-94 return 1" },
          base_damage = { 100, unit = "dmg", damage_type = "pure",
            source = "script:vscripts/heroes/A_Oda/A23.lua:82 (100 + caster:GetIntellect())" },
          increase_distance = { 250, 200, 150, 100, unit = "dist", source = "kv:A23E_increaseDistance" },
          step_bonus = { 5, unit = "pct", source = "script:vscripts/heroes/A_Oda/A23.lua:82 1 + 0.05*floor(距離/increaseDistance)" },
          anchor_vision = { 300, unit = "dist", source = "script:vscripts/heroes/A_Oda/A23.lua:64-65 AddFOWViewer 300" },
          target_vision = { 100, unit = "dist", source = "script:vscripts/heroes/A_Oda/A23.lua:101-102 AddFOWViewer 100" },
          vision_tick = { 0.2, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A23.lua:96-104 return 0.2" },
          vision_time = { 0.5, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A23.lua:101-102 AddFOWViewer 0.5" },
          purge_check = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A23.lua:118 Timers:CreateTimer(0.1)" },
        },
        text = {
          zh_TW = { name = "鬼術．靈魂鎖縛",
            body = "在自身位置立下結界，與目標敵方英雄之間建立連鎖，持續 {duration} 秒：施放當下與之後每 {tick} 秒，對目標造成（{base_damage} ＋ 秀家智力）的純粹傷害；目標與結界的距離每 {increase_distance}，傷害提高 {step_bonus}。結界周圍 {anchor_vision} 與目標周圍 {target_vision} 範圍對雙方可見。連鎖提前解除（例如狀態抗性縮短或目標死亡）時，若目標當時不是魔法免疫，傷害就停止。" },
          en = { name = "Demon Art: Soul Bind",
            body = "Plants a ward at Hideie's position and chains it to the target enemy hero for {duration} seconds: on cast and every {tick} seconds after, deals ({base_damage} + Hideie's intelligence) pure damage to the target, increased by {step_bonus} for every {increase_distance} between the target and the ward. Both teams have vision within {anchor_vision} of the ward and within {target_vision} of the target. If the chain ends early (for example shortened by status resistance, or the target dies) while the target is not spell immune, the damage stops." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        unit_state = {
          a23e_purged = { type = "bool", init = "false", reveal = "public", scope = "ability",
            note = "原作 ability.state == \"PURGE\"；存在技能上，每次施放重設。" },
        },
        statuses = {
          { id = "a23e_bind", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "debuff",
            reveal = "public", cue = "a23e_chain",
            text = { zh_TW = { name = "靈魂鎖縛", body = "被鎖鏈連向秀家的結界：每 {tick} 秒受到（{base_damage} ＋ 秀家智力）的純粹傷害，與結界距離每 {increase_distance} 傷害提高 {step_bonus}。持續 {duration} 秒。" }, en = { name = "Soul Bind", body = "Chained to Hideie's ward: takes ({base_damage} + Hideie's intelligence) pure damage every {tick} seconds, increased by {step_bonus} for every {increase_distance} from the ward. Lasts {duration} seconds." } },
            triggers = {
              -- modifier_A23E OnDestroy：0.1 秒後若目標不是魔免，ability.state = PURGE（傷害迴圈停止）
              on_removed = { { op = "delay", time = "$purge_check", actions = {
                { op = "if", cond = "not self.is_magic_immune", actions = {
                  { op = "set_state", var = "a23e_purged", value = "true", on = "caster" },
                  { op = "cue", id = "a23e_chain_break", at = "self", stop = true },
                } },
              } } },
            },
            notes = "原作 modifier_A23E（IsDebuff 1、IsPurgable 0），施加者是結界 dummy（同隊），所以受 nobu 狀態抗性縮短（events.lua:397-410）。" },
        },
        events = {
          on_cast = {
            { op = "set_state", var = "a23e_purged", value = "false" },
            { op = "let", name = "anchor", value = "position(caster)" },
            { op = "apply_status", status = "a23e_bind", to = "target" },
            { op = "reveal", at = "anchor", radius = "$anchor_vision", duration = "$duration", teams = "all" },
            { op = "cue", id = "a23e_ward", at = "anchor", duration = "$duration" },
            { op = "repeat", interval = "$tick", first = 0, count = "$duration / $tick", actions = {
              { op = "if", cond = "not state.a23e_purged", actions = {
                { op = "damage", to = "target", type = "pure", require_source_alive = false,
                  amount = "($base_damage + caster.int) * (1 + $step_bonus * floor(distance(target, anchor) / $increase_distance))" },
              } },
            } },
            { op = "repeat", interval = "$vision_tick", first = 0, count = "$duration / $vision_tick", actions = {
              { op = "reveal", at = "target", radius = "$target_vision", duration = "$vision_time", teams = "all" },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A23.lua:73-94",
            text = "傷害 Timer 在 t=0,1,…,duration−1 各一跳（count 從 1 起算、count > duration+0.1 停止），共 duration 跳；第一跳在施放當下。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A23.lua:82",
            text = "傷害每跳讀秀家當下智力與目標到結界（施放時秀家位置）的中心距離；純粹傷害，目標中途魔免也照樣受傷。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A23.lua:114-128",
            text = "modifier_A23E 不可驅散，但施加者是同隊 dummy，nobu 的狀態抗性會縮短它；提前結束 0.1 秒後若目標不是魔免，ability.state = PURGE，傷害迴圈之後每跳都跳過。目標若在那一刻魔免，傷害照常持續到迴圈結束。自然到期時迴圈已結束，沒有影響。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A23.lua:71,81",
            text = "state 存在 ability 上、施放時重設為 NONE：冷卻被重置後連續施放，上一次的鎖縛提前結束也會停掉新一次的傷害；反之新施放會讓上一次已停止的迴圈恢復傷害。v3 以 scope = ability 的 unit_state 照實表達。" },
          { policy = "as_source", text = "傷害用 ApplyDamage，不經 AMHC：秀家死亡後鎖縛傷害仍持續。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A23.lua:96-104",
            text = "目標視野每 0.2 秒以浮點累加計數，迴圈在 count2 > duration+0.1 時停止，共 duration/0.2 次（t=0 到 duration−0.2），每次 0.5 秒；照此寫成固定次數。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A23.lua:60-62",
            text = "結界是 A32E_stick dummy（無敵、無血條、無碰撞、不可選取、繳械），只用來當距離錨點、視野與特效端點，v3 以施放位置 anchor 表達，不建立單位。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A23.lua:80,83", text = "迴圈內兩個 print 是除錯輸出。" },
        },
        provenance = { "npc/A_Oda/A23.txt:84-159", "vscripts/heroes/A_Oda/A23.lua:53-128", "vscripts/events.lua:385-410" },
      },
      -------------------------------------------------------------------------
      { id = "a23_falling_fire", source_id = "A23R", slot = "E", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 12, 11, 10, 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 135, 150, 165, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0, unit = "s", source = "kv:AbilityCastPoint" },
          channel_time = { 4, unit = "s", source = "kv:AbilityChannelTime" },
          radius = { 350, unit = "dist", source = "kv:A23R_aoeRadius" },
          wave_interval = { 0.3, unit = "s", source = "script:vscripts/heroes/A_Oda/A23.lua:139 wave_delay = 0.3" },
          max_wave = { 10, unit = "count", source = "kv:A23R_maxWave", note = "FC-070：原作腳本沒用到這個鍵（引導 4 秒共 14 波）；依 D13 照說明最多 10 波" },
          damage = { 40, 80, 120, 160, unit = "dmg", damage_type = "magical", source = "kv:A23R_damage" },
          building_pct = { 20, unit = "pct", source = "kv:adjust_damage_at_building", note = "KV 寫 0.2（比例）" },
          dot = { 20, 25, 30, 35, unit = "dmg", damage_type = "magical", source = "kv:A23R_dot" },
          dot_duration = { 4, unit = "s", source = "kv:A23R_dotDuration" },
          dot_interval = { 1, unit = "s", source = "kv:modifier_A23R_dot ThinkInterval" },
          hit_delay = { 0.3, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A23.lua:175 Timers:CreateTimer(0.3)" },
          tree_delay = { 0.45, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A23.lua:192 Timers:CreateTimer(0.45)" },
          detect_radius = { 1000, unit = "dist", source = "script:vscripts/heroes/A_Oda/A23.lua:153 FindUnitsInRadius 1000" },
          caster_vision = { 100, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A23.lua:155-156 AddFOWViewer 100" },
          vision_time = { 1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A23.lua:155-159 AddFOWViewer 1" },
          area_vision_mul = { 1.1, unit = "ratio", hidden = true, source = "script:vscripts/heroes/A_Oda/A23.lua:159 aoe_radius*1.1" },
        },
        text = {
          zh_TW = { name = "鬼術．墜火",
            body = "持續施法期間，每 {wave_interval} 秒在目標地點 {radius} 範圍內降下一波火雨（最多 {max_wave} 波），稍後對範圍內的敵方英雄、一般單位與建築造成 {damage} 魔法傷害（建築只受 {building_pct}），並使非建築的敵人燃燒 {dot_duration} 秒、每 {dot_interval} 秒受到 {dot} 魔法傷害，燃燒可疊加。施放後摧毀範圍內的樹木。施法期間若秀家周圍 {detect_radius} 內有敵方單位，雙方都看得到秀家。" },
          en = { name = "Demon Art: Falling Fire",
            body = "While channeling, calls down a wave of fire rain within {radius} of the target point every {wave_interval} seconds (at most {max_wave} waves). Shortly after, each wave deals {damage} magic damage to enemy heroes, units and buildings in the area (buildings take only {building_pct}) and sets non-building enemies ablaze for {dot_duration} seconds, taking {dot} magic damage every {dot_interval} seconds; burns stack. Destroys trees in the area on cast. While channeling, Hideie is visible to both teams if any enemy unit is within {detect_radius} of him." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a23r_burn", duration = "$dot_duration", stacking = "independent", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "墜火燃燒", body = "燃燒：每 {dot_interval} 秒受到 {dot} 魔法傷害，持續 {dot_duration} 秒；可疊加多份。" },
              en = { name = "Falling Fire Burn", body = "Burning: takes {dot} magic damage every {dot_interval} seconds for {dot_duration} seconds; multiple burns stack." },
            },
            reveal = "public", cue = "a23r_burn_fx",
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$dot", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_A23R_dot：MULTIPLE、IsDebuff，沒寫 IsPurgable（依 §12.3 暫定 basic）。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "center", value = "point" },
            { op = "delay", time = "$tree_delay", capture = { "center" }, actions = {
              { op = "destroy_trees", at = "center", radius = "$radius" },
            } },
            { op = "repeat", interval = "$wave_interval", first = 0, count = "$max_wave", ["while"] = "caster.is_channeling", actions = {
              -- 秀家周圍 1000 內有任何敵方單位（含建築、魔免、隱形）就讓雙方看到他 1 秒
              { op = "for_each",
                select = { from = "caster", shape = "circle", radius = "$detect_radius", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon", "building" }, flags = { "include_magic_immune" },
                  limit = 1 },
                actions = {
                  { op = "reveal", at = "caster", radius = "$caster_vision", duration = "$vision_time", teams = "all" },
                } },
              { op = "reveal", at = "center", radius = "$radius * $area_vision_mul", duration = "$vision_time", teams = "own" },
              { op = "cue", id = "a23r_meteor_rain", at = "center", radius = "$radius" },
              { op = "for_each",
                select = { from = "center", shape = "circle", radius = "$radius", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon", "building" } },
                actions = {
                  -- 選取在波次當下，結算延遲 0.3 秒，不再檢查是否仍在範圍內
                  { op = "delay", time = "$hit_delay", actions = {
                    { op = "if", cond = "alive(it)", actions = {
                      { op = "if", cond = "not it.is_building", actions = {
                        { op = "apply_status", status = "a23r_burn", to = "it" },
                      } },
                      { op = "damage", to = "it", type = "magical", require_source_alive = false, flags = { "no_amplify" },
                        amount = "select(it.is_building, $damage * $building_pct, $damage)" },
                    } },
                  } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A23.lua:135-189",
            text = "FC-070（D13 修正：repeat count = $max_wave，最多 10 波）。原作：KV A23R_maxWave = 10 與原作說明「總共可降下10波」都沒有被腳本使用：迴圈只看 IsChanneling，4 秒引導每 0.3 秒一波（t=0…3.9）共 14 波，傷害與燃燒層數比設計多四成。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A23.lua:179",
            text = "對建築的燃燒（modifier_A23R_dot_building，KV A23R_dot_building 10/15/20/25）已被註解，建築只吃每波 20% 傷害。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A23.lua:161-184",
            text = "選取在波次發生時（排除魔免），傷害與燃燒在 0.3 秒後結算，不論目標是否仍在範圍內；這 0.3 秒內變魔免的目標仍會被上燃燒，魔法傷害由規則層歸零。引導中斷後已選取的目標照常結算。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A23.lua:153-157",
            text = "偵測用 GetOpposingTeamNumber 的友方搜尋，類型 HERO|BASIC|BUILDING：靠近敵方塔或任何敵人（含隱形、迷霧中）時，秀家每波都暴露 1 秒給雙方。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A23.lua:192-194",
            text = "砍樹只在施放後 0.45 秒做一次，與引導是否被打斷無關。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A23.lua:176-183",
            text = "原作以 IsValidEntity 判斷；死亡英雄仍是有效實體，原作會對屍體套燃燒（無效果）；已移除的單位在 IsBuilding() 先報錯。v3 以 alive(it) 表達同樣的結算結果。" },
          { policy = "design_intent", ref = "npc/A_Oda/A23.txt:201-218",
            text = "燃燒 ThinkInterval 1、Duration 4：到期當下那一跳依引擎時序，照設計意圖算 4 跳（每層總計 dot × 4）。" },
          { policy = "note", text = "傷害與燃燒都不經 AMHC，秀家死亡後已排程的結算與燃燒照常（死亡也會中斷引導，不再有新波次）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A23.lua:197-205", text = "每波五顆隕石特效與音效屬呈現，改用 cue。" },
        },
        provenance = { "npc/A_Oda/A23.txt:161-278", "vscripts/heroes/A_Oda/A23.lua:130-205",
          "vscripts/library/events/eventfordamage.lua:30-32" },
      },
      -------------------------------------------------------------------------
      { id = "a23_yomi_flame", source_id = "A23T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 120, 100, 80, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 250, 275, 300, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          duration = { 6, unit = "s", source = "kv:modifier_A23T Duration" },
          tick = { 1, unit = "s", source = "kv:modifier_A23T ThinkInterval" },
          damage = { 115, 180, 280, unit = "dmg", damage_type = "magical", source = "kv:A23T_damage" },
        },
        text = {
          zh_TW = { name = "禁術．黃泉之焰",
            body = "使目標敵方英雄陷入黃泉之火 {duration} 秒（可對魔法免疫目標施放）：每 {tick} 秒受到 {damage} 魔法傷害，並且被沉默、無法使用物品、被動技能失效。" },
          en = { name = "Forbidden Art: Flames of Yomi",
            body = "Engulfs the target enemy hero in the flames of Yomi for {duration} seconds (can target spell-immune heroes): it takes {damage} magic damage every {tick} seconds, and is silenced, cannot use items and has its passives disabled." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero" } },
        flags = { "pierce_magic_immune" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a23t_curse", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "debuff",
            text = {
              zh_TW = { name = "黃泉之焰", body = "沉默、無法使用物品、被動技能失效，並每 {tick} 秒受到 {damage} 魔法傷害，持續 {duration} 秒。" },
              en = { name = "Flames of Yomi", body = "Silenced, cannot use items and passives disabled; takes {damage} magic damage every {tick} seconds for {duration} seconds." },
            },
            reveal = "public", states = { "silence", "mute", "break" }, cue = "a23t_burn_fx",
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_A23T：PASSIVES_DISABLED、SILENCED、MUTED，IsPurgable 0；不在 ultimate_modifiers，受 nobu 狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "a23t_curse", to = "target" },
            { op = "cue", id = "a23t_cast", at = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A23.txt:287",
            text = "可以指定魔免英雄（MAGIC_IMMUNE_ENEMIES）：魔免目標照樣被沉默、禁物品、破被動，但魔法傷害被傷害過濾器歸零。" },
          { policy = "as_source", ref = "vscripts/events.lua:385-410",
            text = "modifier_A23T 不在 ultimate_modifiers，敵方英雄的 nobu 狀態抗性會縮短持續時間與跳數。" },
          { policy = "design_intent", ref = "npc/A_Oda/A23.txt:319-336",
            text = "Duration 6、ThinkInterval 1：到期當下那一跳依引擎時序，照設計意圖算 6 跳。" },
          { policy = "as_source", text = "原作說明沒提被動失效（PASSIVES_DISABLED），說明已照實際行為補上。datadriven Damage 不經 AMHC，秀家死亡後仍持續傷害。" },
        },
        provenance = { "npc/A_Oda/A23.txt:280-349", "vscripts/heroes/A_Oda/A23.lua:207-212",
          "vscripts/events.lua:246-268", "vscripts/library/events/eventfordamage.lua:30-32" },
      },
    },
  }
end
