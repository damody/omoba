-- C19 松姬（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C19"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:335）；
-- heromap_autoskill["C19"]["11"] = ""（addon_game_mode.lua:438），沒有 D（本體的 C19D 奇襲是 16 版才有；舊版的奇襲是 R）。
-- KV 在 npc/C_Neutral/C19.txt:101-744（C19W_old／C19E_old／C19R_old／C19T_old，與本體的 C19E、C19R、C19T 交錯）。W/E/R/T → Q/W/E/R。
-- 原作舊版沒有任何繁中說明（panorama/localization/addon_tchinese.txt 查無 C19*_old），名稱取 KV 註解，說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C19_precache 屬規則層或資產，不列入。
-- 慣例照本體 C19：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；datadriven Damage 寫 require_source_alive = false；
-- HERO | BASIC 寫成 BASIC 清單；緩速寫 slow_pct 正值並帶 tag slow。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C19_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C19", text = { zh_TW = { name = "松姬（舊版）" }, en = { name = "Matsuhime (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c19_old_paralyzing_poison", source_id = "C19W_old", slot = "Q", kind = "toggle", max_level = 4, tags = { "basic", "toggle" },
        values = {
          cooldown = { 0.4, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C19.txt:107" },
          attack_bonus = { 30, 40, 50, 60, unit = "dmg", source = "kv:atk@npc/C_Neutral/C19.txt:197（PREATTACK_BONUS_DAMAGE@C19.txt:137）" },
          mana_per_attack = { 15, 20, 25, 30, unit = "mana", source = "kv:mana@npc/C_Neutral/C19.txt:202",
            note = "vscripts/heroes/C_Neutral/C19.lua:211-227 SetMana" },
          slow = { 10, 20, 30, 40, unit = "pct", kv = { -10, -20, -30, -40 }, source = "kv:bonus_moveandattackspeed@npc/C_Neutral/C19.txt:192",
            source_differs = "KV 以負值表示減速（MOVESPEED_BONUS_PERCENTAGE）；slow_pct 以正值表示" },
          attack_slow = { 10, 20, 30, 40, unit = "atk_speed", kv = { -10, -20, -30, -40 }, source = "kv:bonus_moveandattackspeed@npc/C_Neutral/C19.txt:192",
            source_differs = "同一個 KV 也是 ATTACKSPEED_BONUS_CONSTANT；以正值表示攻速降低量" },
          poison_duration = { 6, unit = "s", source = "kv:modifier_incapacitating_bite_debuff_datadriven Duration@npc/C_Neutral/C19.txt:165" },
          dot_interval = { 1, unit = "s", source = "kv:modifier_incapacitating_bite_debuff_datadriven ThinkInterval@npc/C_Neutral/C19.txt:182" },
          dot_damage = { 20, 40, 60, 80, unit = "dmg", damage_type = "magical", source = "kv:damage@npc/C_Neutral/C19.txt:207" },
        },
        text = {
          zh_TW = { name = "癱瘓毒",
            body = "開啟時攻擊力 +{attack_bonus}，每次對非建築目標開始普攻都消耗 {mana_per_attack} 法力；法力不足時自動關閉並耗盡剩餘法力。普攻命中英雄或一般單位時使其中毒 {poison_duration} 秒：移動速度降低 {slow}、攻擊速度降低 {attack_slow}，每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" },
          en = { name = "Paralyzing Poison",
            body = "While active, Matsuhime gains +{attack_bonus} attack damage, and each attack she starts against a non-building target costs {mana_per_attack} mana; if she lacks the mana, the ability turns off and her remaining mana is drained. Attacks that land on heroes or units poison them for {poison_duration} seconds: movement speed reduced by {slow}, attack speed reduced by {attack_slow}, and {dot_damage} magic damage every {dot_interval} seconds." },
        },
        target = { type = "none" },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c19w_old_bite", text = { zh_TW = { name = "癱瘓毒", body = "攻擊力 +{attack_bonus}；每次普攻消耗 {mana_per_attack} 法力，命中時使目標中毒。" }, en = { name = "Paralyzing Poison", body = "+{attack_bonus} attack damage; each attack costs {mana_per_attack} mana and poisons the target on hit." } },
            duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", remove_on = { "death" },
            stats = { attack_damage = "$attack_bonus" },
            triggers = {
              on_death = { { op = "toggle_ability", of = "self", ability = "c19_old_paralyzing_poison", on = false } },
              on_attack_windup = { filter = "not event.target.is_building", actions = {
                { op = "if", cond = "self.mana < $mana_per_attack",
                  actions = {
                    { op = "toggle_ability", of = "self", ability = "c19_old_paralyzing_poison", on = false },
                    { op = "set_mana", to = "self", value = "0" },
                  },
                  otherwise = {
                    { op = "set_mana", to = "self", value = "self.mana - $mana_per_attack" },
                  } },
              } },
              on_attack_hit = { filter = "not event.target.is_building", actions = {
                { op = "apply_status", status = "c19w_old_poison", to = "event.target" },
              } },
            },
            notes = "原作 modifier_incapacitating_bite_datadriven（C19W_old 版）：Passive 0、IsHidden 1；PREATTACK_BONUS_DAMAGE %atk；OnAttackStart C19W_old 扣魔；OnAttackLanded 套中毒。沒寫 IsPurgable，暫定 basic。" },
          { id = "c19w_old_poison", duration = "$poison_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "slow" }, cue = "c19w_old_poison_fx",
            text = {
              zh_TW = { name = "癱瘓毒", body = "移動速度降低 {slow}、攻擊速度降低 {attack_slow}，每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" },
              en = { name = "Paralyzing Poison", body = "Movement speed reduced by {slow} and attack speed by {attack_slow}; takes {dot_damage} magic damage every {dot_interval} seconds." },
            },
            stats = { slow_pct = "$slow", attack_speed = "-$attack_slow" },
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$dot_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_incapacitating_bite_debuff_datadriven（C19W_old 版）：IsDebuff、Duration 6、MOVESPEED_BONUS_PERCENTAGE 與 ATTACKSPEED_BONUS_CONSTANT（datadriven，不經 dummy.lua 緩速池）、ThinkInterval 1 datadriven Damage。" },
        },
        events = {
          on_toggle_on = { { op = "apply_status", status = "c19w_old_bite", to = "caster" } },
          on_toggle_off = { { op = "remove_status", from = "caster", status = "c19w_old_bite" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:211-227",
            text = "扣魔在 OnAttackStart（前搖開始），目標是建築時不扣；以 SetMana 直接扣減（不算消耗法力事件）。法力不足時 CastAbilityToggle 關閉並 SetMana(0)，這一擊沒有加攻與中毒（狀態已移除）。攻擊力加成對建築照樣有效。" },
          { policy = "as_source", ref = "npc/C_Neutral/C19.txt:148-160",
            text = "中毒以 Target 區塊（Center TARGET、HERO | BASIC、MAGIC_IMMUNE_ENEMIES）套上：照本體 C19R 的讀法只套被命中的目標，含魔免、不含建築。" },
          { policy = "fixed", ref = "npc/C_Neutral/C19.txt:131-134",
            text = "FC-167 D22 舊版，比照 D13 修（同今川義元烈焰箭 FC-104 的修法）：原作 modifier_incapacitating_bite_datadriven 不是 Passive，死亡時被移除，但開關仍亮（推定），復活後沒有加攻也不扣魔，要手動關再開；資料在死亡時關閉開關。" },
          { policy = "design_intent", ref = "npc/C_Neutral/C19.txt:162-183",
            text = "Duration 6、ThinkInterval 1：最後一跳是否落在到期當下依引擎時序；照設計意圖每秒一跳。" },
          { policy = "note", ref = "npc/C_Neutral/C19.txt:105-107",
            text = "沒有 AbilityManaCost，開關本身不耗魔；冷卻 0.4 秒只防連點。modifier 名稱與本體 C19R 的被動同名，兩者不會同時出現在一位英雄身上。" },
        },
        provenance = { "npc/C_Neutral/C19.txt:101-210", "vscripts/heroes/C_Neutral/C19.lua:211-227" },
      },
      -------------------------------------------------------------------------
      { id = "c19_old_healing_mist", source_id = "C19E_old", slot = "W", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 15, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C19.txt:247" },
          mana_cost = { 100, 120, 140, 180, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C19.txt:248" },
          cast_range = { 600, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C19.txt:249" },
          channel_time = { 2.1, 2.6, 3.1, 3.6, unit = "s", source = "kv:C19E_old_Duration@npc/C_Neutral/C19.txt:273（AbilityChannelTime@C19.txt:252）" },
          radius = { 500, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C19.lua:243 FindUnitsInRadius 500（AOERadius@C19.txt:250 同值）" },
          heal = { 100, 110, 120, 130, unit = "heal", source = "kv:healv@npc/C_Neutral/C19.txt:278" },
          tick = { 0.5, unit = "s", source = "script:vscripts/heroes/C_Neutral/C19.lua:252 return 0.5" },
          vision_radius = { 600, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C19.lua:235 AddFOWViewer 600" },
          vision_time = { 6, unit = "s", source = "script:vscripts/heroes/C_Neutral/C19.lua:235 AddFOWViewer 6.0" },
        },
        text = {
          zh_TW = { name = "治療噴霧",
            body = "對指定地點噴灑治療噴霧並持續引導：施放時與引導期間每 {tick} 秒治療 {radius} 範圍內的友方英雄與部隊（含自己）{heal} 生命，引導結束時再治療一次。施放時獲得該地點 {vision_radius} 視野 {vision_time} 秒。" },
          en = { name = "Healing Mist",
            body = "Sprays a healing mist at the target point and channels: on cast and every {tick} seconds while channeling, heals allied heroes and units within {radius} (Matsuhime included) for {heal}, plus once more when the channel ends. Grants {vision_radius} vision of the point for {vision_time} seconds on cast." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { channel_time = "$channel_time" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "c19e_old_cast_vo", at = "caster" },
            { op = "cue", id = "c19e_old_mist", at = "caster", to = "point" },
            { op = "reveal", at = "point", radius = "$vision_radius", duration = "$vision_time", teams = "own" },
            { op = "call", routine = "c19e_old_heal", args = { at = "point" } },
          },
          on_channel_tick = { interval = "$tick", actions = {
            { op = "call", routine = "c19e_old_heal", args = { at = "point" } },
          } },
          on_channel_end = {
            { op = "call", routine = "c19e_old_heal", args = { at = "point" } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C19.lua:241-257",
            text = "原作 Timer 從 0 秒起每 0.5 秒「先治療、再檢查是否仍在引導」：施放當下一次、引導期間每 0.5 秒一次，引導結束（或被打斷）後的下一個 0.5 秒刻度再一次。資料寫成 on_cast＋on_channel_tick＋on_channel_end，次數相同（滿引導 6/7/8/9 次），最後一次在引導結束當下結算，比原作最多早 0.5 秒。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:242-248",
            text = "治療是 it:Heal（一般治療，受治療增減影響），以施放點為圓心、友方 HERO | BASIC；施放點固定，松姬不能移動施放點。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C19.lua:238-240,250",
            text = "粒子與 StartGesture 屬呈現；skillcount 沒有被使用。" },
        },
        provenance = { "npc/C_Neutral/C19.txt:239-281", "vscripts/heroes/C_Neutral/C19.lua:230-258" },
      },
      -------------------------------------------------------------------------
      { id = "c19_old_surprise_raid", source_id = "C19R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 60, 50, 40, 30, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C19.txt:446" },
          mana_cost = { 50, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C19.txt:448" },
          cast_point = { 1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C19.txt:447" },
          unseen_time = { 3, unit = "s", source = "script:vscripts/heroes/C_Neutral/C19.lua:201 caster.befow > -3 就中斷（每秒減一）" },
          check_interval = { 1, unit = "s", hidden = true, source = "kv:modifier_C15R_check ThinkInterval@npc/C_Neutral/C19.txt:551" },
          tower_radius = { 800, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C19.lua:188 FindUnitsInRadius 800" },
          slow_radius = { 500, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C19.lua:173 FindUnitsInRadius 500" },
          slow = { 50, unit = "pct", kv = { -50 }, source = "kv:modifier_C19R_old2 MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE@npc/C_Neutral/C19.txt:540",
            source_differs = "KV 以負值表示減速；slow_pct 以正值表示" },
          slow_time = { 5, unit = "s", source = "kv:modifier_C19R_old2 Duration@npc/C_Neutral/C19.txt:545" },
        },
        text = {
          zh_TW = { name = "奇襲",
            body = "傳送到任意地點：松姬必須已經連續 {unseen_time} 秒沒有被敵方看見，且目標地點 {tower_radius} 內沒有敵方建築，否則施法失敗。施法時間內下其他指令就會取消。抵達時閃避飛行中的投射物，並使周圍 {slow_radius} 內的敵方英雄與部隊（含魔免）移動速度降低 {slow}，持續 {slow_time} 秒。" },
          en = { name = "Surprise Raid",
            body = "Teleports to any point. Matsuhime must have gone unseen by the enemy for {unseen_time} consecutive seconds and there must be no enemy building within {tower_radius} of the point, or the cast fails. Issuing another order during the cast cancels it. On arrival she dodges incoming projectiles and slows enemy heroes and units within {slow_radius} (spell-immune ones included) by {slow} for {slow_time} seconds." },
        },
        target = { type = "point" },
        cast = { cast_point = "$cast_point", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        unit_state = {
          c19r_old_unseen = { type = "int", init = "0", reveal = "team", scope = "hero",
            note = "原作 caster.befow：每秒被敵方看見時設為 1，否則減一；≤ -3 才能施放（Lua 欄位，跨死亡保留）" },
        },
        passive = {
          reveal = "public", visibility = "hidden",
          triggers = {
            periodic = { interval = "$check_interval", actions = {
              -- check_FOW：取第一個敵方英雄，以它的隊伍視野判斷；看得到設 1，否則減一
              { op = "set_state", on = "self", var = "c19r_old_unseen", value = "state.c19r_old_unseen - 1" },
              { op = "for_each",
                select = { from = "self", shape = "global", team = "enemy", types = { "hero" }, limit = "1",
                  flags = { "include_magic_immune", "include_invulnerable", "include_hidden", "include_dead", "include_untargetable" } },
                actions = {
                  { op = "if", cond = "is_visible_to(self, it)", actions = {
                    { op = "set_state", on = "self", var = "c19r_old_unseen", value = "1" },
                  } },
                } },
            } },
          },
        },
        statuses = {
          { id = "c19r_old_slow", duration = "$slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow" }, cue = "c19r_old_move_slow",
            text = {
              zh_TW = { name = "奇襲", body = "移動速度降低 {slow}。" },
              en = { name = "Surprise Raid", body = "Movement speed reduced by {slow}." },
            },
            notes = "原作 modifier_C19R_old2：IsDebuff、Duration 5、MOVESPEED_BONUS_PERCENTAGE -50；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = {
            { op = "if", cond = "state.c19r_old_unseen > -$unseen_time", actions = {
              { op = "cancel_cast", reason = "被敵方看見未滿時間" },
            } },
            { op = "for_each",
              select = { from = "point", shape = "circle", radius = "$tower_radius", team = "enemy", types = { "building" },
                flags = { "include_magic_immune", "include_invulnerable" }, limit = "1" },
              actions = { { op = "cancel_cast", reason = "目標附近有敵方建築" } } },
            { op = "cue", id = "c19r_old_teleport_start", at = "caster" },
            { op = "cue", id = "c19r_old_teleport_end", at = "point" },
          },
          on_cast = {
            { op = "cue", id = "c19r_old_voice", at = "caster" },
            { op = "dodge_projectiles", of = "caster" },
            { op = "teleport", who = "caster", to = "point", find_clear_space = true },
            { op = "issue_order", who = "caster", order = "stop" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$slow_radius", team = "enemy", types = BASIC,
                flags = { "include_magic_immune" } },
              actions = { { op = "apply_status", status = "c19r_old_slow", to = "it" } } },
          },
        },
        coverage = "expressible", confidence = "low",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/C_Neutral/C19.lua:163-181",
            text = "FC-168 D22 舊版，比照 D13 修：原作 Teleport2 第一行 ProjectileManager:ProjectileDodge(keys.caster) 引用未定義的 keys（參數名是 event），執行期報錯中斷：施法成功（扣魔、進冷卻）但不傳送、不閃避、不緩速。依設計意圖寫成閃避投射物 → 傳送到目標點（FindClearSpace）→ 停止 → 緩速周圍 500 內的敵人（含魔免）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:183-209,321-340",
            text = "施法開始時檢查：caster.befow > -3（三秒內被敵方看見過）就 Interrupt。check_FOW 由被動 modifier_C15R_check 每秒執行：找 HeroList 第一個敵方英雄，以 CanEntityBeSeenByMyTeam 判斷敵隊看不看得到松姬（等於敵隊視野）；看得到設為 1，否則減一。資料照原作取第一個敵方英雄（含死亡、隱藏）判斷其隊伍視野。學會前不計數，計數存在英雄上、跨死亡保留。" },
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C19.lua:188-196",
            text = "原作只在目標點 800 內有名為 com_towera（織田軍箭塔）的敵方建築時中斷；推定原意是「不能傳到敵方塔旁」；產生器的選取不支援 types tower，資料寫成目標點 800 內有任何敵方建築（含兵營、主城）就中斷，比原作嚴格。com_towera 以外的塔是否刻意不算無法判定，見 assumptions.md。" },
          { policy = "as_source", ref = "npc/C_Neutral/C19.txt:465-534",
            text = "前搖 1 秒期間 modifier_teleportation 的 OnOrder 會結束傳送特效並移除自己，Dota 的新指令本來就取消前搖，所以下指令等於取消施放（cast.interruptible）。施法距離 0 表示全圖。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C19.txt:561-570",
            text = "modifier_C15D（凍結、禁止指令）在 C19R_old 沒有任何地方套用。" },
        },
        provenance = { "npc/C_Neutral/C19.txt:436-572", "vscripts/heroes/C_Neutral/C19.lua:151-209", "vscripts/heroes/C_Neutral/C19.lua:321-340",
          "npc/npc_units_custom_load.txt:13432-13440" },
      },
      -------------------------------------------------------------------------
      { id = "c19_old_inviolable", source_id = "C19T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 100, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C19.txt:691" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C19.txt:692" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel@npc/C_Neutral/C19.txt:689 + LevelsBetweenUpgrades@C19.txt:690" },
          evasion = { 30, 40, 50, unit = "pct", source = "kv:bonus_evasion@npc/C_Neutral/C19.txt:741" },
          duration = { 10, unit = "s", source = "script:vscripts/heroes/C_Neutral/C19.lua:315 Timers 10 後 SetStackCount(1)" },
        },
        text = {
          zh_TW = { name = "不容侵犯",
            body = "被動：閃避 {evasion} 的攻擊。主動：{duration} 秒內閃避加倍，再額外閃避 {evasion} 的攻擊。" },
          en = { name = "Inviolable",
            body = "Passive: evades {evasion} of attacks. Active: for {duration} seconds the evasion doubles, evading a further {evasion} of attacks." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public", visibility = "hidden",
          stats = { evasion_pct = "$evasion" },
        },
        statuses = {
          { id = "c19t_old_double", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "不容侵犯", body = "額外閃避 {evasion} 的攻擊。" },
              en = { name = "Inviolable", body = "Evades a further {evasion} of attacks." },
            },
            reveal = "public", cue = "c19t_old_aura_fx", stats = { evasion_pct = "$evasion" },
            notes = "代表原作 modifier_ninja_cloth 的第二層（SetStackCount(2)，10 秒後回到 1）；FC-169 依設計意圖讓第二層真的給閃避。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "c19t_old_double", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "low",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/C_Neutral/C19.lua:307-319",
            text = "FC-169 D22 舊版，比照 D13 修：原作主動重新套上 modifier_ninja_cloth 並 SetStackCount(2)，10 秒後設回 1；但 datadriven 的 EVASION_CONSTANT 不乘層數，全專案也沒有讀這個層數的程式，主動只有特效、沒有任何效果（耗魔 200～300、冷卻 100）。推定原意是兩層＝兩份閃避，資料寫成 10 秒內再加一份 bonus_evasion 閃避（多個閃避來源怎麼合成由規則層決定）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:284-305",
            text = "被動 modifier_C19T 每秒檢查 modifier_ninja_cloth，不在或層數小於一就重新套上：效果等於永久閃避。和本體不同：沒有魔抗加成、沒有擊退。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C19.lua:312",
            text = "nc 寫成全域變數，與 C19T_OnIntervalThink 的區域變數無關；粒子 a02e 只是呈現。" },
        },
        provenance = { "npc/C_Neutral/C19.txt:680-744", "vscripts/heroes/C_Neutral/C19.lua:284-319" },
      },
    },
    routines = {
      c19e_old_heal = {
        owner = "c19_old_healing_mist",
        params = { at = "point" },
        actions = {
          { op = "for_each",
            select = { from = "at", shape = "circle", radius = "$radius", team = "ally", types = BASIC,
              flags = { "include_magic_immune" } },
            actions = { { op = "heal", to = "it", amount = "$heal" } } },
        },
      },
    },
  }
end
