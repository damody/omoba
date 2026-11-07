-- B28 松永久秀（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["B28"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:307）；heromap_autoskill["B28"]["11"] = ""
-- （addon_game_mode.lua:410），沒有 D。W/E/R/T → Q/W/E/R。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 B28*_old），名稱取 KV 註解，說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、B28_precache 屬規則層或資產，不列入。
-- 慣例照本體 B28：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- datadriven Damage 與 ApplyDamage（不經 AMHC）在久秀死亡後仍會造成傷害，寫 require_source_alive = false；HERO | BASIC 寫成 BASIC 清單。
-- modifier_B28E 列在 _G.EXCLUDE_MODIFIER_NAME（vscripts/library/common/dummy.lua:376-392），狀態帶 tag cleanse_protected。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  local BASIC_BUILDING = { "hero", "creep", "neutral", "summon", "building" }
  return {
    schema_version = 3, kind = "hero", id = "B28_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B28", text = { zh_TW = { name = "松永久秀（舊版）" }, en = { name = "Matsunaga Hisahide (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b28_old_fire_attack", source_id = "B28W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 12, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B28.txt:518" },
          mana_cost = { 170, 190, 210, 230, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B28.txt:519" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B28.txt:517" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B28.txt:516" },
          radius = { 450, unit = "dist", source = "kv:B28W_old_radius@npc/B_Unified/B28.txt:571" },
          first_duration = { 5, unit = "s", source = "kv:B28W_old_duration1@npc/B_Unified/B28.txt:536" },
          second_duration = { 5, unit = "s", source = "kv:B28W_old_duration2@npc/B_Unified/B28.txt:541" },
          first_tick = { 0.2, unit = "s", source = "kv:B28W_old_tick1@npc/B_Unified/B28.txt:546" },
          second_tick = { 0.3, unit = "s", source = "kv:B28W_old_tick2@npc/B_Unified/B28.txt:551" },
          first_damage = { 10, 20, 30, 40, unit = "dmg", damage_type = "magical", source = "kv:B28W_old_damage1@npc/B_Unified/B28.txt:556" },
          second_damage = { 5, 10, 15, 20, unit = "dmg", damage_type = "magical", source = "kv:B28W_old_damage2@npc/B_Unified/B28.txt:561" },
          building_pct = { 20, unit = "pct", source = "kv:B28W_old_adjustOnBuilding@npc/B_Unified/B28.txt:566（0.2 倍）" },
          first_ticks = { 24, unit = "count", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B28.lua:314-320 count1 += tick1，count1 > duration1 停止；0.2 的單精度值累加 25 次超過 5，只跳 24 次" },
          second_ticks = { 16, unit = "count", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B28.lua:346-352 count2 += tick2，count2 > duration2 停止；0.3 累加 17 次超過 5，跳 16 次" },
        },
        text = {
          zh_TW = { name = "鬼謀．火攻",
            body = "在目標地點 {radius} 範圍點火。前 {first_duration} 秒每 {first_tick} 秒對範圍內的敵人（含建築）造成 {first_damage} 魔法傷害，之後 {second_duration} 秒火勢轉弱，每 {second_tick} 秒造成 {second_damage} 魔法傷害；建築只受 {building_pct} 傷害。" },
          en = { name = "Demon Scheme: Fire Attack",
            body = "Sets the target area ({radius}) ablaze. For the first {first_duration} seconds, enemies inside (buildings included) take {first_damage} magic damage every {first_tick} seconds; then the fire weakens for {second_duration} seconds, dealing {second_damage} magic damage every {second_tick} seconds. Buildings take only {building_pct} damage." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "b28w_old_fire", at = "at", radius = "$radius", duration = "$first_duration + $second_duration" },
            { op = "repeat", interval = "$first_tick", count = "$first_ticks", first = 0, actions = {
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC_BUILDING },
                actions = {
                  { op = "damage", to = "it", type = "magical", require_source_alive = false,
                    amount = "select(it.is_building, $first_damage * $building_pct, $first_damage)" },
                } },
            } },
            { op = "delay", time = "$first_duration", capture = { "at" }, actions = {
              { op = "repeat", interval = "$second_tick", count = "$second_ticks", first = 0, actions = {
                { op = "for_each",
                  select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC_BUILDING },
                  actions = {
                    { op = "damage", to = "it", type = "magical", require_source_alive = false,
                      amount = "select(it.is_building, $second_damage * $building_pct, $second_damage)" },
                  } },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:314-377",
            text = "兩段 Timer：第一段從施放當下起每 0.2 秒一跳，計數先加再比較（count1 > duration1 才停），KV 的 0.2 是單精度（0.2000000030），累加 25 次略超過 5，所以只跳 24 次（0～4.6 秒）；第二段在第 5 秒開始每 0.3 秒一跳，跳 16 次（5～9.5 秒）。次數照實際累加結果寫成隱藏值。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:319-376",
            text = "搜尋用技能自己的目標設定：ENEMY、HERO | BASIC | BUILDING、Flags NONE（不含魔免）；以 ApplyDamage（不經 AMHC）造成，久秀死亡後火勢仍照常傷害。建築傷害乘 0.2。" },
          { policy = "note", text = "火焰與字樣粒子只是呈現，改用 cue；第二段結束時銷毀字樣粒子。" },
        },
        provenance = { "npc/B_Unified/B28.txt:504-575", "vscripts/heroes/B_Unified/B28.lua:294-379" },
      },
      -------------------------------------------------------------------------
      { id = "b28_old_binding", source_id = "B28E_old", slot = "W", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 25, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B28.txt:587" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B28.txt:589" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B28.txt:590" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B28.txt:588" },
          channel_time = { 3, 4, 5, 6, unit = "s", source = "kv:B28E_duration@npc/B_Unified/B28.txt:697（AbilityChannelTime 與 modifier_B28E Duration）" },
          damage = { 20, 25, 30, 35, unit = "dmg", damage_type = "magical", source = "kv:B28E_damage@npc/B_Unified/B28.txt:702" },
          tick = { 1.01, unit = "s", source = "kv:modifier_B28E ThinkInterval@npc/B_Unified/B28.txt:674" },
          share = { 30, unit = "pct", source = "script:vscripts/heroes/B_Unified/B28.lua:90 damage*0.3" },
          near_dist = { 200, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B28.lua:17-21 dis < 200 → 200、r = dis − 200" },
          falloff = { 600, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B28.lua:22 (600 − r) / 600 * duration" },
          half = { 0.5, unit = "ratio", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B28.lua:23-24 duration/2" },
          check_interval = { 0.2, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B28.lua:40-58 Timer 0.2" },
        },
        text = {
          zh_TW = { name = "鬼謀．綑綁",
            body = "以火焰繩索綁住目標敵人並持續施法：立即造成 {damage} 魔法傷害並使其暈眩，暈眩期間每 {tick} 秒再受 {damage} 魔法傷害。暈眩時間依兩者距離決定，貼近時等於整個施法時間，越遠越短，最短為一半；暈眩結束時施法也跟著結束，施法中斷時暈眩解除。暈眩期間，久秀受到的傷害有 {share} 以純粹傷害轉給目標。施法中久秀若施放其他技能，繩索立即解除。" },
          en = { name = "Demon Scheme: Binding",
            body = "Binds an enemy with fiery rope and channels: deals {damage} magic damage at once and stuns the target, dealing {damage} magic damage again every {tick} seconds while stunned. The stun lasts the whole channel at close range, shorter the farther away, down to half; the channel ends when the stun does, and interrupting the channel ends the stun. While the target is stunned, {share} of the damage Hisahide takes is dealt to it as pure damage. If Hisahide casts another ability during the channel, the rope breaks immediately." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b28e_old_stun", duration = "$channel_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "鬼謀．綑綁", body = "被火焰繩索暈眩，每 {tick} 秒受到 {damage} 魔法傷害；久秀受到的傷害有 {share} 以純粹傷害轉給這個單位。久秀的施法結束時解除。" },
              en = { name = "Demon Scheme: Binding", body = "Stunned by fiery rope, taking {damage} magic damage every {tick} seconds; {share} of the damage Hisahide takes is dealt to this unit as pure damage. Ends when Hisahide's channel ends." },
            },
            reveal = "public", states = { "stun" }, tags = { "cleanse_protected" }, cue = "b28e_old_rope",
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$damage", require_source_alive = false },
              } },
              periodic_2 = { interval = "$check_interval", actions = {
                { op = "if", cond = "not has_status(caster, b28e_old_guard)", actions = {
                  { op = "remove_status", from = "self", status = "b28e_old_stun" },
                } },
              } },
            },
            notes = "原作 modifier_B28E（B28E_old 的 Modifiers 區塊）：Duration %B28E_duration、IsPurgable 1、IsStunDebuff、STUNNED、ThinkInterval 1.01 的 datadriven Damage。" },
          { id = "b28e_old_bond", duration = "$channel_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            vars = { target = { type = "unit", reveal = "public" } },
            triggers = {
              on_take_damage = {
                filter = "self.is_hero and exists(vars.target) and has_status(vars.target, b28e_old_stun)",
                actions = {
                  { op = "damage", to = "vars.target", source = "self", type = "pure", amount = "event.damage.final * $share" },
                } },
            },
            notes = "原作 modifier_B28E_OnTakeDamage（Lua，IsHidden；沒覆寫 IsPurgable 與 RemoveOnDeath，暫定 none）：修正後照 KV B28E_duration 持續整個引導時間；對象存在 caster.B28E_target，0.2 秒檢查迴圈在目標失去暈眩時清空，資料以「目標仍被暈眩」表達。" },
          { id = "b28e_old_guard", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", tags = { "cleanse_protected" },
            triggers = {
              on_ability_executed = { who = "self",
                actions = {
                  { op = "remove_status", from = "self", status = "b28e_old_guard" },
                } },
            },
            notes = "原作 modifier_B28E2（B28E_old 的 Modifiers 區塊）：IsHidden、沒有 Duration；datadriven OnAbilityExecuted 施放任何技能（舊版不排除開關技能與道具）就移除自己。沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "dist", value = "max(edge_distance(caster, target), $near_dist)" },
            { op = "let", name = "stun_time",
              value = "$channel_time * max($half, ($falloff + $near_dist - dist) / $falloff)" },
            { op = "apply_status", status = "b28e_old_bond", to = "caster", vars = { target = "target" } },
            { op = "set_var", on = "caster", status = "b28e_old_bond", var = "target", value = "target" },
            { op = "apply_status", status = "b28e_old_stun", to = "target", duration = "stun_time" },
            { op = "damage", to = "target", type = "magical", amount = "$damage", require_source_alive = false },
            { op = "cue", id = "b28e_old_fatal_bonds_sound", at = "target" },
            { op = "apply_status", status = "b28e_old_guard", to = "caster" },
            { op = "delay", time = "stun_time", actions = {
              { op = "end_channel", interrupted = true },
            } },
          },
          on_channel_end = {
            { op = "remove_status", from = "target", status = "b28e_old_stun" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/B_Unified/B28.txt:595-602、vscripts/heroes/B_Unified/B28.lua:5-22",
            text = "FC-170 D22 舊版，比照 D13 修：照本體 B28E（RunScript 傳 duration = %B28E_duration）補上持續時間：分傷只在引導期間、目標仍被暈眩時生效；暈眩時間 = 引導時間 × max(0.5, (600 − (max(OBB 距離, 200) − 200)) / 600)，時間到以 EndChannel(true) 結束引導；0.2 秒檢查在 modifier_B28E2 消失（施放其他技能）時解除暈眩。原作：舊版 OnSpellStart 的 RunScript B28E 沒有傳 duration（本體 B28.txt:175 有）：keys.duration 是 nil，AddNewModifier 套上不限時的分傷 modifier，接著第 22 行 (600 − r) / 600 * nil 執行期報錯，腳本後半（依距離縮短暈眩、時間到 EndChannel、0.2 秒檢查迴圈、鎖鏈粒子）都沒有執行。結果：caster.B28E_target 永遠不清空，久秀之後受到的所有傷害（任何來源）都有 30% 以純粹傷害打到最後一次綁住的目標，直到久秀死亡。" },
          { policy = "as_source", ref = "npc/B_Unified/B28.txt:604-650,665-688",
            text = "腳本之後 datadriven 以 Duration %B28E_duration 再套一次 modifier_B28E（刷新成整個施法時間）、造成 B28E_damage 魔法傷害、套 modifier_B28E2；實際暈眩由距離時間到的 EndChannel(true) 觸發 OnChannelInterrupted 的 RemoveModifier 結束，結果與以距離時間套用相同。" },
          { policy = "as_source", ref = "npc/B_Unified/B28.txt:674",
            text = "暈眩的 ThinkInterval 是 1.01 秒，不是 1 秒：六秒的暈眩只跳五次（不含施放時那一下）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:83-93",
            text = "分傷讀 event.damage（減免後傷害），以 AMHC:Damage 造成純粹傷害（久秀或目標已死時不傷害）；目標實體被移除前都有效，即使目標已不在暈眩中、遠在地圖另一端。" },
          { policy = "as_source", ref = "npc/B_Unified/B28.txt:654-664",
            text = "modifier_B28E2 套上後只會在久秀施放下一個技能時移除自己（舊版不排除開關技能與道具）。原作腳本報錯，讀它的 0.2 秒檢查迴圈沒有執行；修正後迴圈照常執行，它不在時解除暈眩但不中斷引導。" },
          { policy = "note", text = "KV 沒有 B28E_share_percentage，caster.B28E_share_percentage 讀到 0 但沒有使用；分傷比例寫死 0.3。AnimationPlaybackRate 0.5 只是動作。" },
        },
        provenance = { "npc/B_Unified/B28.txt:576-706", "vscripts/heroes/B_Unified/B28.lua:1-97",
          "vscripts/library/common/dummy.lua:376-392" },
      },
      -------------------------------------------------------------------------
      { id = "b28_old_flame_storm", source_id = "B28R_old", slot = "E", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 20, 17, 15, 12, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B28.txt:719" },
          mana_cost = { 140, 160, 180, 200, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B28.txt:718" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B28.txt:721" },
          channel_time = { 4, 3.6, 3.2, 2.8, unit = "s", source = "kv:AbilityChannelTime@npc/B_Unified/B28.txt:722" },
          wave_delay = { 0.5, 0.45, 0.4, 0.35, unit = "s", source = "kv:B28R_old_waveDelay@npc/B_Unified/B28.txt:780" },
          max_wave = { 8, unit = "count", hidden = true,
            source = "kv:B28R_old_maxWave@npc/B_Unified/B28.txt:785（腳本讀了但沒用；施法時間 ÷ 間隔恰為 8）" },
          radius = { 400, unit = "dist", source = "kv:B28R_old_radius@npc/B_Unified/B28.txt:790" },
          damage = { 40, 60, 80, 100, unit = "dmg", damage_type = "magical", source = "kv:B28R_old_damage@npc/B_Unified/B28.txt:795" },
          dot_damage = { 2, 4, 8, 10, unit = "dmg", damage_type = "magical", source = "kv:B28R_old_dotDamage@npc/B_Unified/B28.txt:800" },
          dot_duration = { 5, unit = "s", source = "kv:B28R_old_dotDuration@npc/B_Unified/B28.txt:805" },
          dot_interval = { 1, unit = "s", source = "kv:modifier_B28R_old_dot ThinkInterval@npc/B_Unified/B28.txt:762" },
          building_pct = { 20, unit = "pct", source = "kv:B28R_old_adjustOnBuilding@npc/B_Unified/B28.txt:810（0.2 倍）" },
          stun = { 0.25, unit = "s", source = "script:vscripts/heroes/B_Unified/B28.lua:429 modifier_stunned duration 0.25" },
          impact_delay = { 0.3, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B28.lua:426 Timers:CreateTimer(0.3)" },
          tree_delay = { 0.45, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B28.lua:440 Timers:CreateTimer(0.45)" },
          reveal_scale = { 1.1, unit = "ratio", hidden = true, source = "script:vscripts/heroes/B_Unified/B28.lua:409 aoe_radius*1.1" },
          reveal_time = { 1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B28.lua:406-409 AddFOWViewer duration 1" },
          caster_reveal = { 100, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B28.lua:406-407 AddFOWViewer 雙方 100" },
        },
        text = {
          zh_TW = { name = "鬼謀．焰暴",
            body = "持續施法，每 {wave_delay} 秒朝目標地點 {radius} 範圍降下一波火雨：落下時對範圍內的敵人（含建築）造成 {damage} 魔法傷害，非建築單位另外暈眩 {stun} 秒並被點燃 {dot_duration} 秒，每 {dot_interval} 秒受到 {dot_damage} 魔法傷害（每波各自計算，可以疊加）；建築只受 {building_pct} 傷害。施法期間照亮目標區域，久秀的位置也會暴露給雙方，並燒毀範圍內的樹木。" },
          en = { name = "Demon Scheme: Flame Storm",
            body = "Channels, calling down a wave of fire on the target area ({radius}) every {wave_delay} seconds. Each wave deals {damage} magic damage to enemies inside (buildings included); non-building units are also stunned for {stun} seconds and ignited for {dot_duration} seconds, taking {dot_damage} magic damage every {dot_interval} seconds (each wave ignites separately and stacks). Buildings take only {building_pct} damage. While channeling, the target area is revealed, Hisahide's position is revealed to both teams, and trees in the area are burned down." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { channel_time = "$channel_time" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b28r_old_dot", duration = "$dot_duration", stacking = "independent", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "鬼謀．焰暴", body = "燃燒，每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" },
              en = { name = "Demon Scheme: Flame Storm", body = "Burning, taking {dot_damage} magic damage every {dot_interval} seconds." },
            },
            reveal = "public", cue = "b28r_old_burn",
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$dot_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_B28R_old_dot：IsDebuff、Duration %B28R_old_dotDuration、MODIFIER_ATTRIBUTE_MULTIPLE、ThinkInterval 1.0 的 datadriven Damage；沒寫 IsPurgable，暫定 basic。" },
          { id = "b28r_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被{@b28_old_flame_storm}暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Stunned by {@b28_old_flame_storm} for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作以 ApplyDataDrivenModifier 套引擎內建 modifier_stunned（duration 0.25）。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "center", value = "point" },
            { op = "repeat", interval = "$wave_delay", count = "$max_wave", first = 0, ["while"] = "caster.is_channeling", actions = {
              { op = "reveal", at = "caster", radius = "$caster_reveal", duration = "$reveal_time", teams = "all" },
              { op = "reveal", at = "center", radius = "$radius * $reveal_scale", duration = "$reveal_time", teams = "own" },
              { op = "cue", id = "b28r_old_meteors", at = "center", radius = "$radius" },
              { op = "for_each",
                select = { from = "center", shape = "circle", radius = "$radius", team = "enemy", types = BASIC_BUILDING },
                actions = {
                  { op = "let", name = "victim", value = "it" },
                  { op = "delay", time = "$impact_delay", capture = { "victim" }, actions = {
                    { op = "if", cond = "exists(victim)", actions = {
                      { op = "if", cond = "not victim.is_building", actions = {
                        { op = "apply_status", status = "b28r_old_dot", to = "victim" },
                        { op = "apply_status", status = "b28r_old_stun", to = "victim" },
                      } },
                      { op = "damage", to = "victim", type = "magical", require_source_alive = false,
                        amount = "select(victim.is_building, $damage * $building_pct, $damage)", flags = { "no_amplify" } },
                    } },
                  } },
                } },
            } },
            { op = "delay", time = "$tree_delay", capture = { "center" }, actions = {
              { op = "destroy_trees", at = "center", radius = "$radius" },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B28.lua:400-437",
            text = "Timer 從施放當下起每 waveDelay 一波，直到 channel_interrupt 把 ability.isChanneling 設成 false；KV 的 maxWave 8 讀了但沒用。施法時間恰好是間隔的 8 倍，第 9 波與施法結束在同一瞬間，誰先執行取決於引擎順序；資料照 maxWave 的設計意圖最多 8 波。ability.isChanneling 是腳本自己的旗標（施法結束或中斷時清除），資料以 caster.is_channeling 表達。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:410-434",
            text = "選取在每波開始時進行，傷害、暈眩與點燃在 0.3 秒後才結算，期間離開範圍的單位照樣受到；施法在這 0.3 秒內被打斷，已選到的單位仍會受到這一波。ApplyDamage 不經 AMHC、沒帶 ability（不吃法術增幅，no_amplify），久秀死亡後照常傷害。目標設定 HERO | BASIC | BUILDING、Flags NONE（不含魔免）。" },
          { policy = "as_source", ref = "npc/B_Unified/B28.txt:756-772",
            text = "點燃是 MODIFIER_ATTRIBUTE_MULTIPLE：每一波各自套一個，同時可以有好幾個在跑，傷害疊加。建築不被點燃也不被暈眩，只受 0.2 倍傷害。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:406-409",
            text = "每一波都以 AddFOWViewer 把久秀所在位置（半徑 100）暴露給雙方 1 秒，並給己方目標區域 1.1 倍半徑的視野 1 秒。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B28.lua:439-442",
            text = "0.45 秒後只砍一次樹（不是每波）。隕石粒子（每波五顆、隨機落點）只是呈現，改用 cue。KV AbilityCastPoint 0。" },
        },
        provenance = { "npc/B_Unified/B28.txt:707-814", "vscripts/heroes/B_Unified/B28.lua:381-453", "vscripts/heroes/B_Unified/B28.lua:507-509" },
      },
      -------------------------------------------------------------------------
      { id = "b28_old_earth_rend", source_id = "B28T_old", slot = "R", kind = "channel", max_level = 3, tags = { "ultimate", "channel" },
        values = {
          cooldown = { 80, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B28.txt:828" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B28.txt:829" },
          cast_range = { 1300, 1450, 1600, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B28.txt:826" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B28.txt:827" },
          channel_time = { 8, unit = "s", source = "kv:AbilityChannelTime@npc/B_Unified/B28.txt:830" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel；DOTA_ABILITY_TYPE_ULTIMATE（npc/B_Unified/B28.txt:823）預設 6/12/18" },
          radius = { 700, unit = "dist", source = "kv:B28T_old_radius@npc/B_Unified/B28.txt:896" },
          building_damage = { 120, 200, 300, unit = "dmg", damage_type = "magical", source = "kv:B28T_old_damage@npc/B_Unified/B28.txt:891" },
          slow = { 85, unit = "pct", kv = { -85 },
            source = "kv:B28T_old_moveSpeed@npc/B_Unified/B28.txt:901（MOVESPEED_BONUS_PERCENTAGE）",
            source_differs = "KV 以負值表示降低；資料以正值記錄、行為取負號" },
          attack_slow = { 85, unit = "atk_speed", kv = { -85 },
            source = "kv:B28T_old_moveSpeed@npc/B_Unified/B28.txt:901（同一個值也接到 ATTACKSPEED_BONUS_CONSTANT，881 行）",
            source_differs = "KV 以負值表示降低；資料以正值記錄、行為取負號" },
          tick = { 1, unit = "s", source = "script:vscripts/heroes/B_Unified/B28.lua:503 return 1" },
          caster_reveal = { 200, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B28.lua:476-477 AddFOWViewer 雙方 200" },
          reveal_time = { 1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B28.lua:476-477 AddFOWViewer duration 1" },
          aura_linger = { 0.5, unit = "s", hidden = true,
            source = "assumed:Dota datadriven 光環離開範圍後 modifier 殘留約 0.5 秒（引擎預設，KV 沒寫）" },
          dummy_hp = { 1000, unit = "hp", hidden = true, source = "kv:npc_dummy_unit StatusHealth@npc/npc_units_custom_load.txt:5136" },
          dummy_armor = { 10000, unit = "armor", hidden = true, source = "kv:npc_dummy_unit ArmorPhysical@npc/npc_units_custom_load.txt:5132" },
        },
        text = {
          zh_TW = { name = "撕裂大地",
            body = "持續施法，撕裂目標地點 {radius} 範圍的大地：範圍內的敵方英雄與部隊（包含魔免）移動速度降低 {slow}、攻擊速度降低 {attack_slow}；每 {tick} 秒對範圍內的敵方建築造成 {building_damage} 魔法傷害。施法期間久秀的位置會暴露給雙方。施法結束或中斷後，裂地最多再殘留片刻。" },
          en = { name = "Earth Rend",
            body = "Channels, rending the earth in the target area ({radius}): enemy heroes and units inside (spell-immune ones included) have movement speed reduced by {slow} and attack speed reduced by {attack_slow}; every {tick} seconds enemy buildings inside take {building_damage} magic damage. Hisahide's position is revealed to both teams while channeling. After the channel ends or is interrupted, the rift may linger for a moment." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b28t_old_aura", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", cue = "b28t_old_rift",
            states = { "invulnerable", "unselectable", "no_healthbar", "not_on_minimap", "root" },
            aura = { radius = "$radius", team = "enemy", types = BASIC,
              flags = { "include_magic_immune" }, status = "b28t_old_slow", linger = "$aura_linger" },
            notes = "原作 modifier_B28T_old_debuffAura：以 ApplyDataDrivenModifier(caster, dummy, ...) 套在目標點的 dummy 上；Aura_Types HERO | BASIC、含魔免敵人、不套 dummy 自己。dummy 的 majia 給的無敵、不可選取以 states 表達。" },
          { id = "b28t_old_slow", duration = "infinite", stacking = "keep", dispel = "none", visibility = "debuff",
            text = {
              zh_TW = { name = "撕裂大地", body = "移動速度降低 {slow}，攻擊速度降低 {attack_slow}。" },
              en = { name = "Earth Rend", body = "Movement speed reduced by {slow} and attack speed reduced by {attack_slow}." },
            },
            reveal = "public", stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            notes = "原作 modifier_B28T_old_debuff：IsDebuff、IsPurgable 0；Properties MOVESPEED_BONUS_PERCENTAGE 與 ATTACKSPEED_BONUS_CONSTANT 都是 %B28T_old_moveSpeed。光環給的狀態。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "create_unit", template = "b28t_old_dummy", at = "at", owner = "caster", team = "caster",
              lifetime = "$channel_time", as = "d" },
            { op = "apply_status", status = "b28t_old_aura", to = "d" },
            { op = "repeat", interval = "$tick", first = 0, ["while"] = "exists(d)", actions = {
              { op = "if", cond = "not caster.is_channeling",
                actions = { { op = "remove_unit", to = "d" } },
                otherwise = {
                  { op = "reveal", at = "caster", radius = "$caster_reveal", duration = "$reveal_time", teams = "all" },
                  { op = "cue", id = "b28t_old_split", at = "at", radius = "$radius" },
                  { op = "for_each",
                    select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = { "building" } },
                    actions = {
                      { op = "damage", to = "it", type = "magical", amount = "$building_damage", require_source_alive = false, flags = { "no_amplify" } },
                    } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:463-505",
            text = "目標點建 npc_dummy_unit（modifier_kill = 施法時間 8 秒）掛減速光環。Timer 從施放當下起每秒一次：久秀已不在施法中就 ForceKill dummy 並停止，所以施法被打斷後光環最多殘留一秒（加上光環本身的殘留）；否則對範圍內的敵方建築造成傷害（技能目標類型只有 BUILDING），並把久秀的位置（半徑 200）暴露給雙方 1 秒。完整施法時建築受到 8 次傷害（0～7 秒）。" },
          { policy = "as_source", ref = "npc/B_Unified/B28.txt:873-884",
            text = "減速狀態 IsPurgable 0，移速百分比與攻速常數用同一個 KV 值 -85，攻速降低 85 點。英雄與部隊不受傷害，只有建築受傷。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:483-499",
            text = "ApplyDamage 不經 AMHC、沒帶 ability（no_amplify）；久秀死亡時施法本來就中斷，下一秒 dummy 被移除。" },
          { policy = "note", text = "裂地粒子（每秒九處隨機位置）與 EarthSplitter 音效只是呈現，改用 cue。AbilityType ULTIMATE 但沒寫 RequiredLevel，推定 6/12/18（見 assumptions.md）。" },
        },
        provenance = { "npc/B_Unified/B28.txt:815-905", "vscripts/heroes/B_Unified/B28.lua:455-509",
          "npc/npc_units_custom_load.txt:5111-5145" },
      },
    },
    unit_templates = {
      { id = "b28t_old_dummy", source_id = "npc_dummy_unit",
        text = { zh_TW = { name = "撕裂大地" }, en = { name = "Earth Rend" } },
        stats = { hp = "$b28_old_earth_rend.dummy_hp", armor = "$b28_old_earth_rend.dummy_armor", move_speed = "0" },
        attack = { kind = "none" },
        vision = { day = "0", night = "0" },
        ai = "none", owner_death = "persist", tags = { "dummy" },
        notes = "原作 npc_dummy_unit 加 majia，B28T_old 另套 modifier_B28T_old_debuffAura 並以 modifier_kill 存在施法時間；SetOwner(caster)。",
        provenance = { "npc/npc_units_custom_load.txt:5111-5145", "vscripts/heroes/B_Unified/B28.lua:463-469" } },
    },
  }
end
