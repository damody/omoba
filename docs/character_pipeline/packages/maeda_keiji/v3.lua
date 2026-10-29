-- C05 前田慶次（current 版，heromap_skill C05 ["16"] = "WERT"，沒有 D）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R。*_old 是 11 版，不做。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，C05_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（v3.md §12.3 推定）；引擎內建 modifier_stunned 寫 "strong"。
-- datadriven Properties 直接給的移速百分比（不經 nobu 緩速池）寫成 move_speed_pct。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "C05",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "C05",
      text = { zh_TW = { name = "前田慶次" }, en = { name = "Maeda Keiji" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c05_matsukaze", source_id = "C05W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 11, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, 100, 120, 140, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 8, unit = "s", source = "kv:buff_duration" },
          move_speed_pct = { 10, 20, 30, 40, unit = "pct", source = "kv:bonus_movement_speed" },
          attack_speed = { 40, 60, 80, 100, unit = "atk_speed", source = "kv:bonus_attack_speed" },
        },
        text = {
          zh_TW = { name = "松風",
            body = "立即施放（不打斷目前動作）：{duration} 秒內移動速度提高 {move_speed_pct}、攻擊速度提高 {attack_speed}。" },
          en = { name = "Matsukaze",
            body = "Cast instantly without interrupting the current action: for {duration} seconds Keiji gains {move_speed_pct} movement speed and +{attack_speed} attack speed." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c05w_haste", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "松風", body = "移動速度提高 {move_speed_pct}、攻擊速度提高 {attack_speed}。" },
              en = { name = "Matsukaze", body = "Movement speed increased by {move_speed_pct} and attack speed by {attack_speed}." },
            },
            reveal = "public", cue = "c05w_armlet",
            stats = { move_speed_pct = "$move_speed_pct", attack_speed = "$attack_speed" },
            notes = "原作 modifier_C05W_buff：ATTACKSPEED_BONUS_CONSTANT、MOVESPEED_BONUS_PERCENTAGE；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c05w_vo", at = "caster" } },
          on_cast = { { op = "apply_status", status = "c05w_haste", to = "caster" } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/C_Neutral/C05.txt:46-62",
            text = "AbilitySpecial 兩筆都用鍵 \"02\"（bonus_movement_speed 與 bonus_attack_speed）；原作可玩，假設兩者都保留。" },
          { policy = "note", ref = "npc/C_Neutral/C05.txt:42", text = "KV 沒寫 AbilityCastPoint（IMMEDIATE），前搖為零。" },
        },
        provenance = { "npc/C_Neutral/C05.txt:34-117" },
      },
      -------------------------------------------------------------------------
      { id = "c05_thunder_dance", source_id = "C05E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 5, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 90, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 900, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C05.lua:26 ability:GetCastRange()（KV AbilityCastRange 900，AoERadius 也是 900）" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          projectile_speed = { 2000, unit = "speed", source = "kv:projectile_speed" },
        },
        text = {
          zh_TW = { name = "雷舞",
            body = "對周圍 {radius} 內看得見、非魔免的每個敵方英雄與部隊各射出一道追蹤雷電（速度 {projectile_speed}，可被閃躲），命中造成 {damage} 魔法傷害。" },
          en = { name = "Thunder Dance",
            body = "Fires a homing bolt (speed {projectile_speed}, dodgeable) at every visible, non-spell-immune enemy hero and unit within {radius}; each bolt deals {damage} magic damage on impact." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_phase_start = { { op = "cue", id = "c05e_vo", at = "caster" } },
          on_cast = {
            { op = "cue", id = "c05e_cast_sound", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "visible_only" } },
              actions = {
                { op = "let", name = "victim", value = "it" },
                { op = "projectile", kind = "tracking", from = "caster", to = "victim", speed = "$projectile_speed",
                  dodgeable = true, cue = "c05e_bolt",
                  on_hit = {
                    { op = "cue", id = "c05e_impact", at = "it" },
                    { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C05.txt:126-128",
            text = "搜尋用技能設定（ENEMY、HERO|BASIC、FOW_VISIBLE）：只打施法者隊伍看得見的敵人；沒有 MAGIC_IMMUNE 旗標，施放當下魔免的敵人沒有投射物。飛行中才變魔免的目標，魔法傷害由規則層歸零。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C05.lua:42-56",
            text = "命中時以 ApplyDamage 造成傷害（不經 AMHC），慶次在投射物飛行中死亡仍會造成傷害。投射物 bDodgeable，被閃現或 ProjectileDodge 時失效。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C05.lua:9-20",
            text = "projectile_table 是全域變數、Target 初值引用未定義的 target（nil），迴圈內逐一覆寫，沒有影響。" },
        },
        provenance = { "npc/C_Neutral/C05.txt:118-184", "vscripts/heroes/C_Neutral/C05.lua:3-56" },
      },
      -------------------------------------------------------------------------
      { id = "c05_thunder_spear", source_id = "C05R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          base_attack = { 15, 25, 35, 45, unit = "dmg", source = "kv:add_attack_damage", note = "KV 屬性 MODIFIER_PROPERTY_BASEATTACK_BONUSDAMAGE" },
          duration = { 3, unit = "s", source = "kv:duration" },
          attack_slow = { 15, 30, 45, 60, unit = "atk_speed", kv = { -15, -30, -45, -60 }, source = "kv:debuff_attack_speed",
            source_differs = "KV 以負值表示減益；這裡以正值表示降低量" },
          slow = { 10, 13, 16, 20, unit = "pct", kv = { -10, -13, -16, -20 }, source = "kv:debuff_movement_speed",
            source_differs = "KV 以負值表示減益；這裡以正值表示降低量" },
        },
        text = {
          zh_TW = { name = "雷鳴豪槍",
            body = "被動：基礎攻擊力 +{base_attack}。普攻命中英雄或小兵類單位時，使其移動速度降低 {slow}、攻擊速度降低 {attack_slow}，持續 {duration} 秒。" },
          en = { name = "Thunderclap Spear",
            body = "Passive: +{base_attack} base attack damage. Attacks that land on heroes or creep-type units reduce their movement speed by {slow} and attack speed by {attack_slow} for {duration} seconds." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { base_attack_damage = "$base_attack" },
          triggers = {
            on_attack_hit = { filter = "event.target.is_hero or event.target.is_creep", actions = {
              { op = "cue", id = "c05r_chain_lightning", at = "self", to = "event.target" },
              { op = "apply_status", status = "c05r_slow", to = "event.target" },
            } },
          },
        },
        statuses = {
          { id = "c05r_slow", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "雷鳴豪槍緩速", body = "移動速度降低 {slow}、攻擊速度降低 {attack_slow}。" },
              en = { name = "Thunderclap Spear Slow", body = "Movement speed reduced by {slow} and attack speed reduced by {attack_slow}." },
            },
            reveal = "public", cue = "c05r_tempest",
            stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            notes = "原作 modifier_C05R_debuff：Properties 直接減移速與攻速（dummy.lua Slow 版本已註解，不經 nobu 緩速池）；沒寫 IsPurgable，暫定 basic。" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C05.lua:63-71",
            text = "判斷 IsHero() or IsCreep()：不分敵我（反補友軍也會減速）、不檢查魔免（魔免目標照樣被減速）；建築與守衛不算。" },
          { policy = "note", ref = "npc/C_Neutral/C05.txt:232-237",
            text = "OnAbilityPhaseStart 的語音對被動技能不會觸發。" },
        },
        provenance = { "npc/C_Neutral/C05.txt:185-290", "vscripts/heroes/C_Neutral/C05.lua:58-72" },
      },
      -------------------------------------------------------------------------
      { id = "c05_flash_thunder", source_id = "C05T", slot = "R", kind = "channel", max_level = 3, tags = { "ultimate", "channel", "movement" },
        values = {
          cooldown = { 110, 95, 80, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          channel_time = { 1.5, unit = "s", source = "kv:delay", note = "AbilityChannelTime 寫 %delay" },
          required_level = { 6, 12, 18, unit = "level", source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          radius = { 600, unit = "dist", source = "kv:radius" },
          damage = { 400, 700, 1000, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          stun = { 1, unit = "s", source = "kv:stun_time" },
          restrict_time = { 1.5, unit = "s", source = "kv:modifier_C05T Duration" },
          interrupt_cooldown = { 20, unit = "s", source = "script:vscripts/heroes/C_Neutral/C05.lua:139-140 EndCooldown + StartCooldown(20)" },
          hint_delay = { 0.75, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C05.lua:101 Timer 0.75" },
          blink_lead = { 0.3, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C05.lua:107 GetChannelTime() - 0.3" },
        },
        text = {
          zh_TW = { name = "瞬雷閃",
            body = "對任意地點引導 {channel_time} 秒（前搖起 {restrict_time} 秒內無法下達指令），完成後瞬移到該處，摧毀樹木，並對周圍 {radius} 內的敵方英雄與部隊（包含魔免）造成 {damage} 魔法傷害、暈眩 {stun} 秒。引導被打斷時冷卻改為 {interrupt_cooldown} 秒。施放後敵方會看到落點提示。" },
          en = { name = "Flash Thunder",
            body = "Channels for {channel_time} seconds on any location (Keiji cannot be commanded for {restrict_time} seconds from the start of the cast), then teleports there, destroys trees and deals {damage} magic damage to enemy heroes and units within {radius} (spell-immune ones included), stunning them for {stun} seconds. If the channel is interrupted, the cooldown becomes {interrupt_cooldown} seconds. Enemies are shown the landing area during the channel." },
        },
        target = { type = "point", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time", interruptible = true, ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "c05t_restrict", duration = "$restrict_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "瞬雷閃引導", body = "引導瞬移中，{restrict_time} 秒內無法下達指令。" },
              en = { name = "Flash Thunder Channel", body = "Channeling the blink; cannot be given orders for {restrict_time} seconds." },
            },
            reveal = "public", states = { "command_restricted" },
            notes = "原作 modifier_C05T：COMMAND_RESTRICTED，在 OnAbilityPhaseStart 套上；沒寫 IsPurgable，暫定 basic。" },
          { id = "c05t_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "瞬雷閃暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Flash Thunder Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（AddNewModifier(caster, ...)）；受狀態抗性縮短。" },
        },
        events = {
          on_phase_start = {
            { op = "cue", id = "c05t_vo", at = "caster" },
            { op = "apply_status", status = "c05t_restrict", to = "caster" },
          },
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "c05t_aoe_hint", at = "at", radius = "$radius", team = "own" },
            { op = "delay", time = "$hint_delay", capture = { "at" }, actions = {
              { op = "cue", id = "c05t_aoe_hint_enemy", at = "at", radius = "$radius", team = "enemy" },
            } },
            { op = "delay", time = "$channel_time - $blink_lead", actions = {
              { op = "if", cond = "caster.is_channeling", actions = {
                { op = "cue", id = "c05t_blink_start", at = "caster" },
              } },
            } },
          },
          on_channel_end = {
            { op = "if", cond = "event.interrupted",
              actions = {
                { op = "modify_cooldown", of = "caster", ability = "c05_flash_thunder", mode = "set", amount = "$interrupt_cooldown" },
              },
              otherwise = {
                { op = "teleport", who = "caster", to = "point" },
                { op = "cue", id = "c05t_eclipse_area", at = "point", radius = "$radius" },
                { op = "destroy_trees", at = "point", radius = "$radius" },
                { op = "for_each",
                  select = { from = "point", shape = "circle", radius = "$radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                  actions = {
                    { op = "apply_status", status = "c05t_stun", to = "it" },
                    { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                  } },
                { op = "cue", id = "c05t_blink_end", at = "caster" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C05.txt:292-311",
            text = "KV 沒寫 AbilityCastRange：點目標不限距離，等於全地圖瞬移。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C05.lua:135-141",
            text = "引導被打斷時 EndCooldown 後 StartCooldown(20)：冷卻改成固定 20 秒，法力不退還。" },
          { policy = "as_source", ref = "npc/C_Neutral/C05.txt:356-363,388-395",
            text = "COMMAND_RESTRICTED 1.5 秒從前搖開始算（前搖 0.1 ＋引導 1.5），最後 0.1 秒可以下指令取消引導；暈眩、沉默等控制照樣打斷。DONT_CANCEL_CHANNEL：施放其他技能不會取消引導（但指令受限期間也無法施放）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C05.lua:151-186",
            text = "落地以 FindClearSpaceForUnit 找空位；搜尋與砍樹以指定點為圓心（不是落地後的位置）。魔免敵人也會暈眩，魔法傷害由規則層歸零；ApplyDamage 不經 AMHC。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C05.lua:95-116",
            text = "己方立刻看到範圍提示、敵方在 0.75 秒後看到（CreateParticleForTeam），以及引導結束前 0.3 秒的閃光，都只是呈現，改用 cue。" },
          { policy = "no_effect", ref = "vscripts/heroes/C_Neutral/C05.lua:119-133",
            text = "C05T_OnAbilityPhaseStart（3000 範圍內有 com_general 就打斷施法）沒有任何 KV 引用，是死碼。" },
        },
        provenance = { "npc/C_Neutral/C05.txt:291-399", "vscripts/heroes/C_Neutral/C05.lua:74-194" },
      },
    },
  }
end
