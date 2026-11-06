-- A06 井伊直政（舊版，原作 11 版「_old」技能組）— package schema v3。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 _G.heromap_skill.A06["11"] = "WERT" 逐一 AddAbility(A06<鍵>_old)
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-131，vscripts/addon_game_mode.lua:248）；
-- heromap_autoskill.A06["11"] = ""（vscripts/addon_game_mode.lua:351），開場不自動學任何技能。沒有 D。
-- 原作槽位 W/E/R/T → Q/W/E/R。原作沒有繁中說明（只有日文 addon_japanese.txt:1103-1110），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A06_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；Lua modifier 預設不可驅散，寫 "none"（v3.md §12.3，推定）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A06_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A06", text = { zh_TW = { name = "井伊直政（舊版）" }, en = { name = "Ii Naomasa (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a06_old_red_demon_power", source_id = "A06W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 18, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 50, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 14, unit = "s", source = "kv:A06W_Duration",
            note = "modifier_A06W_old 的持續在腳本寫死 14（A06_Ii_Naomasa.lua:50），與 KV 相同，共用這個值" },
          move_speed_pct = { 4, 8, 12, 16, unit = "pct", source = "kv:A06W_speed",
            note = "日文說明寫 5/10/15/20%；KV 與 modifier 實際是 4/8/12/16" },
          base_attack_time = { 1.2, 1, 0.85, 0.75, unit = "s", source = "kv:A06W_speed2（MODIFIER_PROPERTY_BASE_ATTACK_TIME_CONSTANT）" },
          backlash = { 40, 50, 60, 70, unit = "pct", source = "kv:A06W_dmg", kv = { 0.4, 0.5, 0.6, 0.7 },
            source_differs = "KV 以倍率 0.4～0.7 寫；v3 以百分比表示同一個數" },
        },
        text = {
          zh_TW = { name = "赤鬼之力",
            body = "解放赤鬼之力 {duration} 秒：基礎攻擊間隔變為 {base_attack_time} 秒，移動速度提高 {move_speed_pct}。代價是期間每次受到傷害，傷害來源會再對直政造成該次原始傷害 {backlash} 的純粹傷害。" },
          en = { name = "Power of Akaoni",
            body = "Unleashes the power of the Akaoni for {duration} seconds: base attack time becomes {base_attack_time} seconds and movement speed is increased by {move_speed_pct}. In exchange, each time Naomasa takes damage during this time, the damage source deals an extra {backlash} of that damage's original amount to him as pure damage." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a06w_old_power", duration = "$duration", stacking = "independent", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "a06w_old_red_greevil_glow",
            stats = { move_speed_pct = "$move_speed_pct",
              base_attack_time = { op = "set_absolute", value = "$base_attack_time" } },
            text = { zh_TW = { name = "赤鬼之力", body = "基礎攻擊間隔變為 {base_attack_time} 秒，移動速度提高 {move_speed_pct}。" },
              en = { name = "Power of Akaoni", body = "Base attack time becomes {base_attack_time} seconds and movement speed +{move_speed_pct}." } },
            notes = "原作 modifier_A06Wx：IsBuff、MULTIPLE、IsPurgable 1；MOVESPEED_BONUS_PERCENTAGE、BASE_ATTACK_TIME_CONSTANT（攻速常數加成那行已註解）。" },
          { id = "a06w_old_backlash", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "debuff",
            reveal = "public",
            text = { zh_TW = { name = "赤鬼反噬", body = "每次受到傷害，傷害來源再造成原始傷害 {backlash} 的純粹傷害。" },
              en = { name = "Akaoni Backlash", body = "Each time damage is taken, its source deals an extra {backlash} of the original damage as pure damage." } },
            triggers = {
              on_take_damage = {
                filter = "exists(event.attacker) and not event.damage.is_reflection",
                actions = {
                  { op = "damage", to = "self", source = "event.attacker", type = "pure",
                    amount = "event.damage.raw * $backlash", flags = { "reflection" } },
                } },
            },
            notes = "原作 modifier_A06W_old（Lua，AddNewModifier duration 14）：MODIFIER_EVENT_ON_TAKEDAMAGE；沒覆寫 IsPurgable／IsDebuff（推定不可驅散）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a06w_old_cast_sound", at = "caster" },
            { op = "apply_status", status = "a06w_old_power", to = "caster" },
            { op = "apply_status", status = "a06w_old_backlash", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:27-42",
            text = "變數命名對調（attacker = event.unit 是受傷者、victim = event.attacker 是來源），判斷結果是：受傷者是直政時，AMHC:Damage(來源, 直政, original_damage × A06W_dmg, 純粹)。也就是增益期間直政額外承受傷害，不是反傷；日文說明「追加の50%の真実ダメージを受ける」與此一致（比例依等級 40～70%）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:35-41",
            text = "caster.compute 旗標防止追加傷害再次觸發自己；資料以 reflection 旗標加 filter 排除。來源可以是任何隊伍（友方、自己造成的傷害也算），AMHC:Damage 要求來源與直政都存活（規則層 require_source_alive）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:33",
            text = "以 original_damage（減免前）計算，魔法傷害帶魔抗時追加量比實際受到的傷害大。" },
          { policy = "as_source", ref = "npc/A_Oda/A06.txt:254-268",
            text = "增益（modifier_A06Wx）IsPurgable 1 可被驅散，反噬（Lua modifier）不可驅散：被驅散後只剩代價。modifier_A06Wx 是 MULTIPLE，冷卻重置後重施會疊兩份移速（BAT 取設定值）；Lua modifier 非 MULTIPLE，重施只刷新。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:17-25",
            text = "modifier_A06W_old 每 0.1 秒記錄 self.hp，但沒有任何地方使用。" },
          { policy = "note", ref = "npc/A_Oda/A06.txt:226-252",
            text = "FireSound A06W_old.vo1 與 item_soul_adder.lua sound（fireblast_cast）只是音效；KV 沒寫 AbilityCastPoint，IMMEDIATE 無前搖。" },
        },
        provenance = { "npc/A_Oda/A06.txt:202-294", "vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:1-53" },
      },
      -------------------------------------------------------------------------
      { id = "a06_old_red_flame_slash", source_id = "A06E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 14, 11, 8, 5, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 250, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          stun = { 1.25, 1.75, 2.25, 3, unit = "s", source = "kv:stun" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:damage",
            note = "日文說明寫 100/190/280/370；KV 實際 100/200/300/400" },
          knockback_distance = { 1250, unit = "dist",
            source = "script:vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:62-63 SetPhysicsVelocity 2000，physics.lua:1379,1391,911 摩擦 0.05／幀、低於 20 停止，30 幀／秒推算約 1250" },
          knockback_time = { 3, unit = "s", hidden = true,
            source = "script:physics.lua:911-942 速度每幀乘 0.95，約 90 幀（3 秒）後低於停止門檻" },
        },
        text = {
          zh_TW = { name = "赤炎斬",
            body = "斬擊目標敵人，使其暈眩 {stun} 秒、受到 {damage} 魔法傷害，並朝遠離直政的方向擊飛約 {knockback_distance} 距離。" },
          en = { name = "Red Flame Slash",
            body = "Slashes the target enemy, stunning it for {stun} seconds, dealing {damage} magic damage and sending it flying roughly {knockback_distance} away from Naomasa." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a06e_old_stun", duration = "$stun", stacking = "independent", dispel = "basic", visibility = "debuff",
            reveal = "public", states = { "stun" }, cue = "a06e_old_stunned_overhead",
            text = { zh_TW = { name = "赤炎斬", body = "暈眩 {stun} 秒。" }, en = { name = "Red Flame Slash", body = "Stunned for {stun} seconds." } },
            notes = "原作 modifier_A06E（A06E_old 自己的 Modifiers）：IsStunDebuff、MULTIPLE；沒寫 IsPurgable，依 §12.3 暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a06e_old_cast_sound", at = "caster" },
            { op = "apply_status", status = "a06e_old_stun", to = "target" },
            { op = "knockback", who = "target", from = "caster", distance = "$knockback_distance",
              duration = "$knockback_time", stun = false },
            { op = "cue", id = "a06e_old_impact_sound", at = "target" },
            { op = "damage", to = "target", type = "magical", amount = "$damage", require_source_alive = false },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:55-64",
            text = "擊飛用 Physics 函式庫：速度 2000 朝遠離直政的方向，摩擦 0.05／幀、速度低於 20 時停下。距離與時間由預設參數推算（約 1250、3 秒），地形碰撞會提早停下；暈眩比擊飛短時，目標在滑行中已能行動。" },
          { policy = "as_source", ref = "npc/A_Oda/A06.txt:472-476",
            text = "stun 的 var_type 是 FIELD_INTEGER 但值有小數；推定 GetSpecialValueFor 照樣讀到小數（同 B21 的推定），未實測。" },
          { policy = "as_source", ref = "npc/A_Oda/A06.txt:458-464",
            text = "modifier_A06E 是 MULTIPLE：短冷卻（5 秒）下重疊施放會有兩份暈眩各自計時（stacking = independent）。SPELL_IMMUNITY_ENEMIES_NO：不能對魔免目標施放。" },
          { policy = "note", ref = "npc/A_Oda/A06.txt:437-451",
            text = "OnSpellStart 依序：音效、RunScript（暈眩＋擊飛）、命中音效、Damage。日文說明的 100/190/280/370 與 KV 不同，照 KV。" },
        },
        provenance = { "npc/A_Oda/A06.txt:415-494", "vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:55-64",
          "vscripts/physics.lua:905-945", "vscripts/physics.lua:1370-1395" },
      },
      -------------------------------------------------------------------------
      { id = "a06_old_war_spirit", source_id = "A06R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          move_speed_pct = { 4, 8, 12, 16, unit = "pct", source = "kv:A06R_SPEED" },
          attack_speed = { 15, 30, 45, 60, unit = "atk_speed", source = "kv:A06R_SPEED2",
            note = "日文說明寫攻速 15/30/45/60%；屬性是 ATTACKSPEED_BONUS_CONSTANT，加的是攻速點數" },
        },
        text = {
          zh_TW = { name = "戰魂",
            body = "被動：移動速度提高 {move_speed_pct}，攻擊速度 +{attack_speed}。" },
          en = { name = "War Spirit",
            body = "Passive: +{move_speed_pct} movement speed and +{attack_speed} attack speed." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public", visibility = "buff",
          stats = { move_speed_pct = "$move_speed_pct", attack_speed = "$attack_speed" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/A_Oda/A06.txt:574-584",
            text = "modifier_A06R：Passive、IsBuff，沒有 IsHidden（狀態列可見）。KV LevelsBetweenUpgrades 2 是一般技能的預設升級間隔。" },
        },
        provenance = { "npc/A_Oda/A06.txt:563-600" },
      },
      -------------------------------------------------------------------------
      { id = "a06_old_red_demon_fury", source_id = "A06T_old", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          radius = { 450, unit = "dist", source = "script:vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:138 FindUnitsInRadius 450（KV AOERadius 同）" },
          chance = { 75, unit = "pct", source = "script:vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:136-137 RandomInt(1,100) > 25" },
          pity_after = { 1, unit = "count", source = "script:vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:137,153 沒觸發時設 nextA06T_old，下一次必定觸發" },
          damage = { 180, 270, 360, unit = "dmg", damage_type = "pure", source = "kv:dmg",
            note = "日文說明寫 170/250/340；KV 實際 180/270/360" },
          immune_pct = { 50, unit = "pct", source = "script:vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:144-145 魔免單位 dmg*0.5" },
        },
        text = {
          zh_TW = { name = "赤鬼之怒",
            body = "被動：普攻命中非建築目標時有 {chance} 機率，對直政周圍 {radius} 內的敵方英雄與部隊（包含魔免單位與被攻擊者）造成 {damage} 純粹傷害，魔免單位只受 {immune_pct}；連續 {pity_after} 次沒有觸發時，下一次命中必定觸發。" },
          en = { name = "Fury of the Red Demon",
            body = "Passive: attacks that hit a non-building target have a {chance} chance to deal {damage} pure damage to enemy heroes and units within {radius} of Naomasa (spell-immune units and the attacked target included); spell-immune units take only {immune_pct}. After {pity_after} hit(s) in a row without triggering, the next hit always triggers." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public", visibility = "buff",
          triggers = {
            on_attack_hit = {
              filter = "not event.target.is_building",
              actions = {
                { op = "chance", p = "$chance", pity_after = "$pity_after", counter = "hero",
                  actions = {
                    { op = "for_each",
                      select = { from = "self", shape = "circle", radius = "$radius", team = "enemy",
                        types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                      actions = {
                        { op = "cue", id = "a06t_old_burst", at = "it" },
                        { op = "damage", to = "it", type = "pure",
                          amount = "select(it.is_magic_immune, $damage * $immune_pct, $damage)" },
                      } },
                    { op = "cue", id = "a06t_old_vo", at = "self" },
                  } },
              } },
            on_kill = {
              filter = "event.victim.is_hero",
              actions = { { op = "cue", id = "a06t_old_title" } } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:136-154",
            text = "RandomInt(1,100) > 25 是 75% 機率；失敗時設 caster.nextA06T_old，下一次非建築命中必定觸發。等於連續失敗一次後保底（pity_after 一次）。攻擊建築不擲骰、不影響保底。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:137-139",
            text = "以直政為圓心（不是被攻擊者），範圍包含被攻擊者本身與魔免敵人；純粹傷害，魔免單位只受一半。AMHC:Damage 要求直政存活。" },
          { policy = "note", ref = "npc/A_Oda/A06.txt:776-784",
            text = "OnHeroKilled 只呼叫 showTitle 顯示大招標題（呈現）。日文說明寫範圍 550、傷害 170/250/340，照腳本與 KV。" },
        },
        provenance = { "npc/A_Oda/A06.txt:745-811", "vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:131-157" },
      },
    },
  }
end
