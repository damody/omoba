-- A32 瀧川一益（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:274），
-- 沒有自動學習的技能（heromap_autoskill["11"] = ""，addon_game_mode.lua:377）。W/E/R/T → Q/W/E/R，沒有 D、F。
-- 原作舊版沒有任何語言的說明（resource/addon_*.txt 查無 A32*_old），名稱與說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、A32_precache 屬規則層或資產，不列入。
-- 慣例照本體 A32：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- ApplyDamage（不經 AMHC）在攻擊者死亡後仍會造成傷害，寫 require_source_alive = false。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A32_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A32", text = { zh_TW = { name = "瀧川一益（舊版）" }, en = { name = "Takigawa Kazumasu (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a32_old_flying_slash", source_id = "A32W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 12, 11, 10, 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 300, unit = "dist", source = "kv:A32W_Radius" },
          fly_time = { 0.6, unit = "s", source = "kv:A32W_flyTime" },
          leap_height = { 300, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A32.lua:415 knockback_height = 300" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:A32W_damage" },
          stun = { 0.8, 1.1, 1.4, 1.7, unit = "s", source = "kv:A32W_stunDuration" },
        },
        text = {
          zh_TW = { name = "忍法．怨靈飛翔斬",
            body = "躍向目標地點，經過 {fly_time} 秒落地，對落點 {radius} 內的非魔免敵人造成 {damage} 魔法傷害並暈眩 {stun} 秒。" },
          en = { name = "Ninpo: Vengeful Flying Slash",
            body = "Leaps to the target point, landing after {fly_time} seconds, dealing {damage} magic damage to non-spell-immune enemies within {radius} and stunning them for {stun} seconds." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a32w_old_stun", text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } }, duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" },
            notes = "原作以 ApplyDataDrivenModifier 套引擎內建 modifier_stunned（未在本技能 Modifiers 宣告，依賴引擎允許內建名稱，同本體）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a32w_old_leap_gesture", at = "caster" },
            { op = "leap", who = "caster", to = "point", duration = "$fly_time", height = "$leap_height" },
            { op = "delay", time = "$fly_time", actions = {
              { op = "cue", id = "a32w_old_thunderclap", at = "caster", radius = "$radius" },
              { op = "for_each",
                select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon" },
                  where = "not it.is_building" },
                actions = {
                  { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false, flags = { "no_amplify" } },
                  { op = "apply_status", status = "a32w_old_stun", to = "it" },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A32.lua:427-464",
            text = "落地判定以施法者落地位置為圓心；選取帶 MAGIC_IMMUNE_ENEMIES 但迴圈內 IsMagicImmune 就跳過，等同排除魔免。先傷害再暈眩（本體相反）。沒有本體的二段斬與馬甲排除；count 計數沒有使用。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A32.lua:402-425",
            text = "以 modifier_knockback（負距離、高度 300、0.6 秒、不暈眩）飛向目標點；傷害在 DelayedAction 0.6 秒後結算，與實際是否到達無關。飛行時間固定讀第一級 A32W_flyTime。" },
          { policy = "as_source", text = "傷害走 ApplyDamage，落地前死亡仍會結算（DelayedAction 照常執行）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A32.lua:433", text = "local A32W2_duration = 5 宣告未使用；地板裂痕粒子屬呈現。" },
        },
        provenance = { "npc/A_Oda/A32.txt:504-571", "vscripts/heroes/A_Oda/A32.lua:402-466" },
      },
      -------------------------------------------------------------------------
      { id = "a32_old_banish", source_id = "A32E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 30, 25, 20, 15, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 4, 5, 6, 7, unit = "s", source = "kv:A32E_old_duration" },
          slow = { 40, unit = "pct", source = "kv:A32E_old_speedDecrease", kv = { -40 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          damage_increase = { 66, unit = "pct", source = "kv:A32E_old_damageIncrease", note = "INCOMING_DAMAGE_PERCENTAGE" },
        },
        text = {
          zh_TW = { name = "忍法．放逐",
            body = "使一名敵方或友方單位進入虛空狀態 {duration} 秒：不受物理傷害、無法攻擊、移動速度降低 {slow}，受到的其他傷害增加 {damage_increase}。" },
          en = { name = "Ninpo: Banish",
            body = "Puts an enemy or allied unit into a void state for {duration} seconds: immune to physical damage, unable to attack, {slow} less movement speed, and {damage_increase} more damage taken from other sources." },
        },
        target = { type = "unit", range = "$cast_range", team = "both", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a32e_old_banish", text = { zh_TW = { name = "放逐", body = "不受物理傷害、無法攻擊，移動速度降低 {slow}，受到的其他傷害增加 {damage_increase}。" }, en = { name = "Banished", body = "Immune to physical damage and unable to attack; movement speed reduced by {slow} and other damage taken increased by {damage_increase}." } }, duration = "$duration", stacking = "refresh", dispel = "none", visibility = "debuff",
            reveal = "public", states = { "disarm", "physical_immune" }, tags = { "isvoid" }, cue = "a32e_old_void_glow",
            stats = { incoming_damage_pct = "$damage_increase", move_speed_pct = "-$slow" },
            notes = "原作 modifier_A32E_old_Banish（IsPurgable 0、DISARMED、INCOMING_DAMAGE +66%、移速 −40%）＋ target.isvoid = 1（傷害過濾取消物理傷害，OnDestroy 清除），同本體以 physical_immune 表達；tag isvoid 已登錄在 v3-shared-registry.lua（A03、A15、B35、鬼面、村正檢查用）。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "a32e_old_banish", to = "target" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/library/events/eventfordamage.lua:42-44",
            text = "target.isvoid 且攻擊者沒有 attackvoid 時，傷害過濾對物理傷害 return false（整筆取消）；以 physical_immune 表達：物理傷害歸零，+66% 增傷只作用於魔法與純粹傷害。開著舊版大招（attackvoid）的瀧川對虛空目標的物理傷害會轉成魔法傷害打進去。" },
          { policy = "as_source", ref = "npc/A_Oda/A32.txt:578",
            text = "目標隊伍是 BOTH：可以把友軍放逐來躲物理傷害；FLAG_NONE，不能對魔免單位施放。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A32.lua:468-483",
            text = "isvoid 在 OnSpellStart 設為 1、modifier OnDestroy 清除，刷新不會提早清除；SetRenderColor 屬呈現。" },
          { policy = "note", ref = "npc/A_Oda/A32.txt:612-613",
            text = "MODIFIER_STATE_ATTACK_IMMUNE 被註解掉，被放逐者仍可被普攻（普攻的物理傷害被取消）。" },
        },
        provenance = { "npc/A_Oda/A32.txt:574-657", "vscripts/heroes/A_Oda/A32.lua:468-483",
          "vscripts/library/events/eventfordamage.lua:42-54" },
      },
      -------------------------------------------------------------------------
      { id = "a32_old_phantom_blade", source_id = "A32R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          crit_chance = { 20, unit = "pct", source = "kv:modifier_A32R_old_Crit Random Chance" },
          crit_multiplier = { 200, 230, 260, 290, unit = "pct", source = "kv:A32R_old_CritScale" },
        },
        text = {
          zh_TW = { name = "忍術．幽魄刀",
            body = "被動：普攻出手時有 {crit_chance} 機率暴擊，造成 {crit_multiplier} 傷害。" },
          en = { name = "Ninjutsu: Phantom Blade",
            body = "Passive: each attack has a {crit_chance} chance to critically strike for {crit_multiplier} damage." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_windup = { {
              op = "chance", p = "$crit_chance",
              actions = { { op = "modify_attack", crit_multiplier = "$crit_multiplier" } } } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "design_intent", ref = "npc/A_Oda/A32.txt:683-720",
            text = "原作在 OnAttackStart 先移除再以 datadriven Random（非 PRD）掛 modifier_A32R_old_Crit2，OnAttackLanded 移除；照設計意圖直接記在這一次攻擊（modify_attack）。" },
          { policy = "no_effect", ref = "npc/A_Oda/A32.txt:665-678",
            text = "KV 留有 OnSpellStart（套 modifier_A32R_Banish、呼叫 A32R_OnSpellStart），但技能是 PASSIVE，不會施放，是死碼。" },
          { policy = "as_source",
            text = "開著舊版大招時物理傷害（含暴擊後的數值）被轉成魔法傷害，所以暴擊仍會放大轉換後的傷害。" },
        },
        provenance = { "npc/A_Oda/A32.txt:660-737" },
      },
      -------------------------------------------------------------------------
      { id = "a32_old_soul_blade", source_id = "A32T_old", slot = "R", kind = "toggle", max_level = 3, tags = { "ultimate", "toggle" },
        values = {
          cooldown = { 2, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 150, 200, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel；Dota 大招預設 6 級起、每 6 級升一次（同本體 A32T）" },
          radius = { 400, unit = "dist",
            source = "script:vscripts/heroes/A_Oda/A32.lua:126 FindUnitsInRadius 半徑 400", note = "KV AbilityCastRange、AOERadius 也是 400，只影響指示器" },
          aura_damage = { 120, 180, 240, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage",
            note = "A32T_old_OnIntervalThink 讀 GetAbilityDamage()" },
          tick = { 0.5, unit = "s", source = "kv:modifier_A32T_old ThinkInterval" },
          mana_per_tick = { 5, 10, 15, unit = "mana", source = "kv:mana" },
          attack_bonus = { 25, 40, 55, unit = "dmg", source = "kv:attack_bonus", note = "BASEATTACK_BONUSDAMAGE" },
          convert_interval = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A32.lua:497-509 Timer 0.1、return 0.1" },
          lock_time = { 1, unit = "s", hidden = true, source = "kv:modifier_A32T_locker Duration" },
        },
        text = {
          zh_TW = { name = "忍術．斬魂刀",
            body = "開啟時基礎攻擊力增加 {attack_bonus}，每 {tick} 秒消耗 {mana_per_tick} 魔力，並對周圍 {radius} 內的非魔免敵人造成 {aura_damage} 魔法傷害；魔力不足時自動關閉。開啟期間，自身造成的物理傷害全部取消，改為以同樣數值的魔法傷害打在原目標上。" },
          en = { name = "Ninjutsu: Soul-Cutting Blade",
            body = "While active, base attack damage is increased by {attack_bonus}; every {tick} seconds it drains {mana_per_tick} mana and deals {aura_damage} magic damage to non-spell-immune enemies within {radius}, turning off when mana runs short. While active, all physical damage Kazumasu deals is canceled and dealt to the same target as magic damage of the same amount instead." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        unit_state = {
          a32_old_void = { type = "bool", init = "false", reveal = "public", scope = "unit",
            note = "原作 caster.attackvoid：開啟時為 1，關閉時清除；FC-133 修正後死亡時隨切換關閉一起清除" },
          a32_old_next_victim = { type = "unit", reveal = "public", scope = "unit",
            note = "原作 caster.next_attack.victim：最後一筆被取消的物理傷害的目標" },
          a32_old_next_damage = { type = "fixed", reveal = "public", scope = "unit",
            note = "原作 caster.next_attack.damage：傷害過濾當下的 filterTable.damage" },
        },
        statuses = {
          { id = "a32t_old_soul_blade", text = { zh_TW = { name = "斬魂刀", body = "基礎攻擊力增加 {attack_bonus}，每 {tick} 秒消耗 {mana_per_tick} 魔力並對周圍 {radius} 內的敵人造成 {aura_damage} 魔法傷害。" }, en = { name = "Soul-Cutting Blade", body = "Base attack damage increased by {attack_bonus}; every {tick} seconds drains {mana_per_tick} mana and deals {aura_damage} magic damage to enemies within {radius}." } }, duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", remove_on = { "death" }, cue = "a32t_old_aura",
            stats = { base_attack_damage = "$attack_bonus" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "spend_mana", from = "self", amount = "$mana_per_tick" },
                { op = "if", cond = "self.mana < $mana_per_tick",
                  actions = {
                    { op = "toggle_ability", of = "self", ability = "a32_old_soul_blade", on = false },
                  },
                  otherwise = {
                    { op = "for_each",
                      select = { from = "self", shape = "circle", radius = "$radius", team = "enemy",
                        types = { "hero", "creep", "neutral", "summon" } },
                      actions = {
                        { op = "damage", to = "it", type = "magical", amount = "$aura_damage" },
                      } },
                  } },
              } },
              -- FC-133（D13，本體同招修法）：死亡時關閉切換，復活後不會只把物理傷害轉換而沒有加攻、耗魔與範圍傷害
              on_death = { { op = "toggle_ability", of = "self", ability = "a32_old_soul_blade", on = false } },
            },
            notes = "原作 modifier_A32T_old（datadriven，IsBuff，沒寫 IsPurgable 暫定 basic；非 Passive，死亡移除）。以 Target { Center CASTER, Flags MAGIC_IMMUNE_ENEMIES | INVULNERABLE } 套在自己身上。" },
          { id = "a32t_old_locker", duration = "$lock_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_created = { { op = "set_ability_active", of = "self", ability = "a32_old_soul_blade", active = false } },
              on_removed = { { op = "set_ability_active", of = "self", ability = "a32_old_soul_blade", active = true } },
            },
            notes = "原作 modifier_A32T_locker：OnCreated SetActivated(false)、OnDestroy SetActivated(true)。" },
        },
        passive = {
          reveal = "public",
          triggers = {
            on_deal_damage_pre = { filter = "state.a32_old_void and event.damage.is_physical", actions = {
              { op = "set_state", var = "a32_old_next_victim", value = "event.victim" },
              { op = "set_state", var = "a32_old_next_damage", value = "event.damage.after_armor" },
              { op = "modify_damage", cancel = true },
            } },
            periodic = { interval = "$convert_interval", actions = {
              { op = "if", cond = "exists(state.a32_old_next_victim)", actions = {
                { op = "damage", to = "state.a32_old_next_victim", type = "magical", amount = "state.a32_old_next_damage",
                  require_source_alive = false },
                { op = "clear_state", var = "a32_old_next_victim" },
                { op = "clear_state", var = "a32_old_next_damage" },
              } },
            } },
          },
        },
        events = {
          on_toggle_on = {
            { op = "set_state", var = "a32_old_void", value = "true" },
            { op = "apply_status", status = "a32t_old_soul_blade", to = "caster" },
            { op = "apply_status", status = "a32t_old_locker", to = "caster" },
          },
          on_toggle_off = {
            { op = "set_state", var = "a32_old_void", value = "false" },
            { op = "remove_status", from = "caster", status = "a32t_old_soul_blade" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/library/events/eventfordamage.lua:45-54",
            text = "caster.attackvoid 時，凡是 DAMAGE_TYPE_PHYSICAL 的傷害（普攻、物理技能、道具）都記進 caster.next_attack（傷害過濾當下的數值，推定已扣護甲）並取消；A32T_old_OnToggleOn 的 0.1 秒計時器把它以 ApplyDamage 魔法傷害打出。轉換後的魔法傷害再受魔抗減免（護甲與魔抗都吃到）；腳本裡把護甲還原的程式被註解掉。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A32.lua:497-509",
            text = "next_attack 只存一筆：同一個 0.1 秒內有多筆物理傷害時只有最後一筆被轉換，其餘直接消失。資料以 0.1 秒的 periodic 清空同一份 unit_state 表達。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A32.lua:493-512",
            text = "FC-133 D22 舊版，比照本體 A32F 的 FC-133 修：死亡時關閉切換（on_toggle_off 清掉 attackvoid 並移除 modifier）。原作：每次開啟都新建一個永不停止的 0.1 秒計時器（關閉也不停），多個計時器搶同一個 next_attack，結果與單一計時器相同。切換在死亡時不會自動關閉，但 modifier_A32T_old 死亡移除：復活後 attackvoid 仍為真、物理傷害繼續轉換，卻沒有加攻、耗魔與範圍傷害，直到手動關再開。修正後死亡即關閉，復活後不再轉換傷害；多計時器搶同一筆的行為與單一計時器相同，照舊。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A32.lua:497",
            text = "ApplyDataDrivenModifier(caster, caster, \"modifier_A32F\")：modifier_A32F 不在 A32T_old 的 Modifiers 裡，套用失敗、沒有效果。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A32.lua:119-139",
            text = "每 0.5 秒先 SpendMana，再檢查剩餘魔力是否低於每跳耗魔：不足就 CastAbilityToggle 關閉（這一跳沒有傷害），否則對 400 內 HERO|BASIC、FLAG_NONE 的敵人以 AMHC:Damage 造成 AbilityDamage 魔法傷害（攻擊者死亡不傷害、魔免歸零屬規則層）。" },
          { policy = "note", ref = "npc/A_Oda/A32.txt:799-826",
            text = "modifier_A32T_locker 讓技能停用一秒，比兩秒冷卻短，實際沒有額外限制。" },
          { policy = "note", ref = "npc/A_Oda/A32.txt:840-877",
            text = "AbilitySpecial 有兩個 \"03\"（attack_bonus、move_speed_bonus），推定兩筆都被讀進來；attack_range_bonus、armor_bonus、move_speed_bonus 對應的 Properties 都被註解掉，沒有使用。" },
        },
        provenance = { "npc/A_Oda/A32.txt:739-878", "vscripts/heroes/A_Oda/A32.lua:119-139",
          "vscripts/heroes/A_Oda/A32.lua:485-519", "vscripts/library/events/eventfordamage.lua:42-54" },
      },
    },
  }
end
