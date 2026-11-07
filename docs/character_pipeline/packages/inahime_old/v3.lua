-- A02_old 稻姬（舊版，11 版技能組；KV 註解標「11.2B」）— package schema v3。
-- 版本：原作 heromap_version A02 ["11"] = true，heromap_skill A02 ["11"] = "WERT"（vscripts/addon_game_mode.lua:142,244），
-- heromap_autoskill A02 ["11"] = ""（vscripts/addon_game_mode.lua:347）。
-- 槽位依據：vscripts/items/Addon_Items/tofar_goback.lua:105-133 先移除全部技能，再依 "WERT" 順序
-- AddAbility A02W_old、A02E_old、A02R_old、attribute_bonusx、A02T_old，所以 W/E/R/T → Q/W/E/R；沒有 D。
-- 舊版不會加回 magical_resistance；attribute_bonusx、A02_precache、halloween_attack、buff_* 屬規則層，都不列入。
-- 原作沒有舊版招式的繁中說明（resource/addon_tchinese.txt 沒有 A02*_old），招式名取 KV 註解，說明依 KV 與腳本還原，見 assumptions.md。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（v3.md §12.3 推定）。
-- 原作 HERO | BASIC 照本體寫成 types = { "hero", "creep", "neutral", "summon" }。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A02_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A02", text = { zh_TW = { name = "稻姬（舊版）" }, en = { name = "Inahime (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a02_old_venom_arrow", source_id = "A02W_old", slot = "Q", kind = "toggle", max_level = 4, tags = { "basic", "toggle", "orb" },
        values = {
          dot_damage = { 3, 6, 9, 12, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          dot_interval = { 1, unit = "s", hidden = true, source = "kv:modifier_A02W_old_debuff ThinkInterval" },
          debuff_duration = { 4, 6, 8, 10, unit = "s", source = "kv:debuff_duration" },
          attack_slow = { -1, -2, -3, -4, unit = "atk_speed", source = "kv:debuff_attack_speed" },
          move_slow = { -1, -2, -3, -4, unit = "pct", source = "kv:debuff_movement_speed" },
          mana_per_attack = { 5, unit = "mana", source = "kv:mana_cost",
            note = "每次對非建築、非魔免目標出手時消耗（A02.lua:107-109）" },
          no_heal = { 100, unit = "pct", hidden = true, source = "kv:modifier_A02W_old_debuff MODIFIER_PROPERTY_DISABLE_HEALING 1",
            note = "禁療以減療 100% 加生命回復增幅 −100% 表達（狀態旗標 cannot_be_healed 產生器尚未支援）" },
        },
        text = {
          zh_TW = { name = "蠍毒箭",
            body = "切換：開啟期間，每次對非建築、非魔免目標出手時消耗 {mana_per_attack} 法力；普攻命中時使目標每秒受到 {dot_damage} 魔法傷害、移動速度降低 {move_slow:abs}、攻擊速度降低 {attack_slow:abs} 點，並且無法回復生命，持續 {debuff_duration} 秒；效果可以重疊。法力不足時自動關閉，該次攻擊不帶毒。" },
          en = { name = "Venom Arrow",
            body = "Toggle: while on, each attack started against a non-building, non-spell-immune target costs {mana_per_attack} mana; on hit the target takes {dot_damage} magic damage per second, loses {move_slow:abs} movement speed and {attack_slow:abs} attack speed, and cannot be healed for {debuff_duration} seconds; the effect stacks. When mana runs short the toggle switches off and that attack is not poisoned." },
        },
        target = { type = "none" },
        statuses = {
          { id = "a02w_old_orb", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", cue = "a02w_orb_projectile",
            triggers = {
              -- FC-149 D22 舊版：拿掉永遠成立的法球槽檢查（本體同一段已註解），照原作接下來的法力檢查與扣魔
              on_attack_windup = { actions = {
                { op = "if", cond = "self.mana < $mana_per_attack",
                  actions = {
                    { op = "toggle_ability", of = "self", ability = "a02_old_venom_arrow", on = false },
                  },
                  otherwise = {
                    -- FC-078（比照本體 D13）：建築與魔免不扣魔，也不多打一下（原作改呼叫 PerformAttack）
                    { op = "if", cond = "not event.target.is_building and not event.target.is_magic_immune",
                      actions = { { op = "spend_mana", from = "self", amount = "$mana_per_attack", ability = "a02_old_venom_arrow" } } },
                  } },
              } },
              on_attack_hit = {
                { op = "apply_status", status = "a02w_old_poison", to = "event.target", source = "self" },
              },
            },
            notes = "原作 modifier_A02W（A02W_old 自己的 Modifiers 區塊）：IsHidden、Orb（Priority ABILITY，只換投射物外觀）；OnAttackStart 跑 A02W_old_OnAttackStart，OnAttackLanded 套 modifier_A02W_old_debuff。由 OnToggleOn 掛上、OnToggleOff 移除。" },
          { id = "a02w_old_poison", duration = "$debuff_duration", stacking = "independent", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "蠍毒", body = "每秒受到 {dot_damage} 魔法傷害，移動速度降低 {move_slow:abs}、攻擊速度降低 {attack_slow:abs} 點，無法回復生命；效果可以重疊。" },
              en = { name = "Scorpion Venom", body = "Takes {dot_damage} magic damage per second, loses {move_slow:abs} movement speed and {attack_slow:abs} attack speed, and cannot be healed; stacks." },
            },
            reveal = "public", cue = "a02w_poison_glow",
            stats = { move_speed_pct = "$move_slow", attack_speed = "$attack_slow",
              heal_reduction_pct = "$no_heal", hp_regen_amp_pct = "-$no_heal" },
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$dot_damage", require_source_alive = false },
                { op = "cue", id = "a02w_poison_number", at = "self" },
              } },
            },
            notes = "原作 modifier_A02W_old_debuff：IsDebuff、MODIFIER_ATTRIBUTE_MULTIPLE、MOVESPEED_BONUS_PERCENTAGE %debuff_movement_speed、ATTACKSPEED_BONUS_CONSTANT %debuff_attack_speed、DISABLE_HEALING 1、ThinkInterval 1 跑 A02W_old_apply_dot；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_toggle_on = {
            { op = "apply_status", status = "a02w_old_orb", to = "caster" },
          },
          on_toggle_off = { { op = "remove_status", from = "caster", status = "a02w_old_orb" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A02.lua:94-99",
            text = "FC-150 D22 舊版，比照 D13 修：原作 A02W_old_OnAttackStart 先檢查 caster.nobuorb1 ~= \"A02W\" 就 ToggleAbility 並 return；但 A02W_OnToggleOn 寫入 nobuorb1 的那行已註解（A02.lua:9-10），全專案也沒有別處寫入 \"A02W\"，所以這個檢查永遠成立：每次開啟後，下一次攻擊前搖開始就自動關閉並移除 modifier_A02W，那一擊不帶毒、不扣魔，只有前搖中才開啟的那一擊會帶毒。修法：比照本體（同一段檢查在本體 A02W_OnAttackStart 已整段註解），拿掉法球槽檢查，開關持續開著，照原作接下來的邏輯：法力不足 mana_per_attack 時關閉、該擊不帶毒；否則對非建築、非魔免目標扣魔，命中時上毒。" },
          { policy = "as_source", ref = "npc/A_Oda/A02.txt:541-549",
            text = "攻擊前搖中才開啟的那一擊沒有經過 OnAttackStart，不扣魔，命中時 OnAttackLanded 仍會套毒。OnAttackLanded 的 ApplyModifier 不分敵我、不排除建築與魔免：反補友軍也會中毒；魔免目標仍會被緩速並禁療，魔法傷害由規則層歸零。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A02.lua:107-112",
            text = "FC-078 D22 舊版，比照 D13 修：拿掉法球槽檢查後這段才會執行：原作目標是建築或魔免時不扣魔，改呼叫 PerformAttack(target, false, false, false, false, true, false, false)，前搖開始時立刻多射出一發普通攻擊，等於對建築與魔免每次出手打兩下。修法同本體：只是不扣魔，不多打一下。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A02.lua:101-105",
            text = "法力檢查在 OnAttackStart（前搖開始），在建築判定之前：對建築出手時法力不足也會關閉。扣魔在前搖開始，攻擊被取消仍然扣魔。" },
          { policy = "design_intent", ref = "npc/A_Oda/A02.txt:576-585",
            text = "ThinkInterval 1.0：第一跳在套上後 1 秒；持續時間剛好是整數秒，最後一跳是否落在到期當下依引擎時序，照 KV 間隔寫成 periodic。" },
          { policy = "as_source", ref = "npc/A_Oda/A02.txt:557",
            text = "MODIFIER_ATTRIBUTE_MULTIPLE：每次命中各一個實例，傷害、緩速與持續時間各自計算（stacking = independent）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A02.lua:128-130",
            text = "每跳以 OVERHEAD_ALERT_BONUS_POISON_DAMAGE 顯示數字，屬呈現。OnAbilityPhaseStart 的語音對切換技能不會觸發。" },
        },
        provenance = { "npc/A_Oda/A02.txt:450-588", "vscripts/heroes/A_Oda/A02.lua:3-20", "vscripts/heroes/A_Oda/A02.lua:88-131" },
      },
      -------------------------------------------------------------------------
      { id = "a02_old_wind_power", source_id = "A02E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          attack_speed = { 60, 90, 120, 150, unit = "atk_speed", source = "kv:buff_attack_speed" },
          move_speed_pct = { 0, 10, 20, 30, unit = "pct", source = "kv:buff_movement_speed" },
          duration = { 15, unit = "s", source = "kv:buff_duration" },
        },
        text = {
          zh_TW = { name = "風之力",
            body = "使一名友方英雄或部隊（可以是自己）在 {duration} 秒內攻擊速度 +{attack_speed}、移動速度 +{move_speed_pct}。" },
          en = { name = "Power of Wind",
            body = "Grants an allied hero or unit (herself included) +{attack_speed} attack speed and +{move_speed_pct} movement speed for {duration} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "ally", types = { "hero", "creep", "neutral", "summon" },
          flags = { "include_magic_immune" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a02e_old_buff", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "風之力", body = "攻擊速度 +{attack_speed}、移動速度 +{move_speed_pct}。" },
              en = { name = "Power of Wind", body = "+{attack_speed} attack speed and +{move_speed_pct} movement speed." },
            },
            reveal = "public", cue = "a02e_old_buff",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct" },
            notes = "原作 modifier_A02E_old_buff：IsBuff、ATTACKSPEED_BONUS_CONSTANT、MOVESPEED_BONUS_PERCENTAGE；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a02e_cast", at = "target" },
            { op = "apply_status", status = "a02e_old_buff", to = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A02.txt:613",
            text = "第一級移動速度加成是 0，只有攻速。" },
          { policy = "as_source", ref = "npc/A_Oda/A02.txt:639-648",
            text = "ApplyModifier 的 Target 只寫 Center TARGET（沒有 Radius），只給被指定的那一個單位；旗標含 INVULNERABLE，無敵的友軍也會拿到。" },
          { policy = "note", ref = "npc/A_Oda/A02.txt:649-654",
            text = "healing_salve 音效與 a02e_old_cast_flash 特效屬呈現。" },
        },
        provenance = { "npc/A_Oda/A02.txt:590-687" },
      },
      -------------------------------------------------------------------------
      { id = "a02_old_self_recovery", source_id = "A02R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          regen = { 40, 60, 80, 100, unit = "per_s", source = "kv:add_hp_regen" },
          duration = { 10, unit = "s", source = "kv:buff_duration" },
        },
        text = {
          zh_TW = { name = "自我恢復",
            body = "使一名友方英雄或部隊（可以是自己）在 {duration} 秒內每秒回復 {regen} 生命。" },
          en = { name = "Self Recovery",
            body = "An allied hero or unit (herself included) regenerates {regen} health per second for {duration} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "ally", types = { "hero", "creep", "neutral", "summon" },
          flags = { "include_magic_immune" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a02r_old_regen", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "自我恢復", body = "每秒回復 {regen} 生命。" },
              en = { name = "Self Recovery", body = "+{regen} health regeneration." },
            },
            reveal = "public", cue = "a02r_natures_attendants",
            stats = { hp_regen = "$regen" },
            notes = "原作 modifier_A02R_old：IsBuff、HEALTH_REGEN_CONSTANT %add_hp_regen；時間由腳本以 buff_duration 給；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a02r_vo", at = "caster" },
            { op = "apply_status", status = "a02r_old_regen", to = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A02.txt:696-699",
            text = "名稱是「自我恢復」，但 KV 是友方單體目標，可以施放在隊友身上。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A02.lua:139-152",
            text = "施法者與目標身上的 enchantress 特效在 buff_duration 後銷毀，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A02.txt:689-757", "vscripts/heroes/A_Oda/A02.lua:133-155" },
      },
      -------------------------------------------------------------------------
      { id = "a02_old_shadow_meld", source_id = "A02T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 110, 100, 90, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 240, 280, 320, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level", source = "assumed:原作沒寫 RequiredLevel，大招取 6/12/18（同本體）" },
          duration = { 14, 16, 18, unit = "s", source = "kv:duration" },
          base_attack_time = { 0.9, unit = "s", source = "kv:base_attack_time" },
        },
        text = {
          zh_TW = { name = "融影術",
            body = "隱形 {duration} 秒，攻擊時不會現形；期間基礎攻擊間隔變為 {base_attack_time} 秒，並且無單位碰撞。" },
          en = { name = "Shadow Meld",
            body = "Turns invisible for {duration} seconds; attacking does not reveal her. Meanwhile her base attack time becomes {base_attack_time} seconds and she has no unit collision." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a02t_old_meld", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "融影術", body = "隱形，攻擊時不會現形；基礎攻擊間隔 {base_attack_time} 秒，無單位碰撞。" },
              en = { name = "Shadow Meld", body = "Invisible; attacking does not reveal her. Base attack time {base_attack_time} seconds and no unit collision." },
            },
            reveal = "public", cue = "a02t_windrun", tags = { "invisible" },
            states = { "invisible", "no_unit_collision" },
            stats = { base_attack_time = { op = "set_absolute", value = "$base_attack_time" } },
            notes = "原作 modifier_A02T_old：MODIFIER_STATE_INVISIBLE、NO_UNIT_COLLISION、BASE_ATTACK_TIME_CONSTANT %base_attack_time、windrun 動作替換。隱形是 modifier 狀態，不是引擎 modifier_invisible，攻擊施法不會破除。沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a02t_vo", at = "caster" },
            { op = "cue", id = "a02t_title", at = "caster" },
            { op = "apply_status", status = "a02t_old_meld", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A02.txt:829-858",
            text = "舊版只有隱形、無碰撞與基礎攻擊間隔 0.9，沒有本體的移動速度加成。" },
          { policy = "note", ref = "npc/A_Oda/A02.txt:766-769",
            text = "KV 沒寫 RequiredLevel（見 required_level 的 source）；showTitle 與 icy_wind 音效屬呈現。" },
        },
        provenance = { "npc/A_Oda/A02.txt:759-860" },
      },
    },
  }
end
