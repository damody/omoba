-- C09 立花誾千代（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C09"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:325）；沒有 D，自動學習 heromap_autoskill["C09"]["11"] = ""（addon_game_mode.lua:428）。
-- W/E/R/T → Q/W/E/R。原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 C09*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C09_precache 不列入。
-- 慣例照本體 C09：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；datadriven Properties 的移速百分比寫成 move_speed_pct；
-- ApplyDamage（不經 AMHC）在誾千代死亡後仍會造成傷害，寫 require_source_alive = false；HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C09_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C09", text = { zh_TW = { name = "立花誾千代（舊版）" }, en = { name = "Tachibana Ginchiyo (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c09_old_frost_slash", source_id = "C09W_old", slot = "Q", kind = "toggle", max_level = 4,
        tags = { "basic", "toggle", "orb" },
        values = {
          mana_per_attack = { 2, 3, 4, 5, unit = "mana", source = "kv:mana_cost@npc/C_Neutral/C09.txt:561" },
          damage = { 5, 10, 15, 20, unit = "dmg", damage_type = "magical", source = "kv:damage@npc/C_Neutral/C09.txt:566" },
          slow = { 15, 25, 35, 45, unit = "pct", kv = { -15, -25, -35, -45 },
            source = "kv:debuff_movement_speed@npc/C_Neutral/C09.txt:556",
            source_differs = "KV 以負值表示減益；這裡以正值表示降低量" },
          attack_slow = { 10, 15, 20, 25, unit = "atk_speed", kv = { -10, -15, -20, -25 },
            source = "kv:debuff_attack_speed@npc/C_Neutral/C09.txt:551",
            source_differs = "KV 以負值表示減益；這裡以正值表示降低量" },
          duration = { 1, unit = "s", source = "kv:debuff_duration@npc/C_Neutral/C09.txt:546" },
        },
        text = {
          zh_TW = { name = "冰霜斬擊",
            body = "開啟時佔用法球，每次對非建築目標出手消耗 {mana_per_attack} 法力；法力不足或法球被其他法球取代時自動關閉。命中非建築目標時額外造成 {damage} 魔法傷害並降低移動速度 {slow}，目標不是魔免時再降低攻擊速度 {attack_slow}，持續 {duration} 秒。對建築出手不消耗法力，也不附加效果。" },
          en = { name = "Frost Slash",
            body = "While active it occupies the orb slot, and each attack started against a non-building costs {mana_per_attack} mana; it turns off when Ginchiyo lacks the mana or another orb takes over. Attacks that land on non-buildings deal {damage} bonus magic damage and slow movement speed by {slow}; if the target is not spell immune, its attack speed is also reduced by {attack_slow}. The slows last {duration} seconds. Attacks against buildings cost no mana and carry no effect." },
        },
        target = { type = "none" },
        passive = { orb_slot = { release = "status_end", claim = true, overridable = true } },
        statuses = {
          { id = "c09w_old_orb", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", remove_on = { "death" },
            triggers = {
              on_attack_windup = {
                { op = "if", cond = "not orb_slot_free(self, ability) or self.mana < $mana_per_attack",
                  actions = { { op = "toggle_ability", of = "self", ability = "c09_old_frost_slash", on = false } },
                  otherwise = {
                    -- FC-090（D13）：對建築不扣魔、不套法球，也不多打一次
                    { op = "if", cond = "not event.target.is_building", actions = {
                      { op = "spend_mana", from = "self", amount = "$mana_per_attack", ability = "ability" },
                      { op = "modify_attack", is_orb = true },
                    } },
                  } },
              },
              on_attack_hit = { filter = "not event.target.is_building", actions = {
                { op = "damage", to = "event.target", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "if", cond = "exists(event.target)", actions = {
                  { op = "apply_status", status = "c09w_old_slow", to = "event.target" },
                  { op = "if", cond = "not event.target.is_magic_immune", actions = {
                    { op = "apply_status", status = "c09w_old_attack_slow", to = "event.target" },
                  } },
                } },
              } },
            },
            notes = "原作 modifier_C09W_old：IsHidden、Orb Priority ABILITY；沒寫 IsPurgable，暫定 basic。" },
          { id = "c09w_old_slow", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "冰霜斬擊減速", body = "移動速度降低 {slow}。" },
              en = { name = "Frost Slash Slow", body = "Movement speed reduced by {slow}." },
            },
            reveal = "public", cue = "c09w_old_ice", stats = { move_speed_pct = "-$slow" },
            notes = "原作 modifier_C09W_old_debuff：IsPurgable 1、MOVESPEED_BONUS_PERCENTAGE。" },
          { id = "c09w_old_attack_slow", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", stats = { attack_speed = "-$attack_slow" },
            notes = "原作 modifier_C09W_old_debuff2：IsPurgable 1、IsHidden、ATTACKSPEED_BONUS_CONSTANT。" },
        },
        events = {
          on_toggle_on = { { op = "apply_status", status = "c09w_old_orb", to = "caster" } },
          on_toggle_off = { { op = "remove_status", from = "caster", status = "c09w_old_orb" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/C_Neutral/C09.lua:236-240",
            text = "FC-090（本體 C09W 的 D13 修正，舊版是同一段寫法）：對建築不扣魔、不套法球，拿掉多打的那一次。原作：對建築出手時呼叫 PerformAttack(target, …, useProjectile = true, …)，每次出手額外射出一次真實普攻；開著冰霜斬擊拆塔等於攻擊次數加倍。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C09.lua:196-226",
            text = "和本體不同，舊版真的使用法球槽 nobuorb1：開啟時無條件寫成 C09W_old（記下原本的法球），關閉時寫回原本的值；出手前搖時槽已被其他法球改寫就關閉。以 orb_slot = { claim = true, overridable = true, release = status_end } 表達；關閉時寫回先前法球的細節由法球槽規則處理。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C09.lua:228-232",
            text = "法力檢查在 OnAttackStart，對建築出手也檢查；法力不足時關閉開關，這一擊沒有效果。扣魔在出手前搖，攻擊被取消或落空仍然扣魔。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C09.lua:242-262",
            text = "OnAttackLanded 不分敵我（反補友軍也有效果）。移速減益不看魔免，魔免目標也會被緩速；只有攻速減益跳過魔免。ApplyDamage 不經 AMHC。" },
          { policy = "as_source",
            text = "開關開著死亡時，datadriven 狀態隨死亡移除但開關仍亮著（同本體推定）；資料以 remove_on death 表達。" },
          { policy = "note", ref = "npc/C_Neutral/C09.txt:445-470", text = "KV 沒寫冷卻與耗魔；耗魔全在每次出手。PopupNumbers 與冰晶粒子只是呈現。" },
        },
        provenance = { "npc/C_Neutral/C09.txt:444-564", "vscripts/heroes/C_Neutral/C09.lua:196-262" },
      },
      -------------------------------------------------------------------------
      { id = "c09_old_white_heat", source_id = "C09E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, 14, 13, 12, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C09.txt:581" },
          mana_cost = { 70, 80, 90, 100, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C09.txt:583" },
          cast_range = { 1200, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C09.txt:584" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C09.txt:582" },
          attack_speed = { 50, 100, 150, 200, unit = "atk_speed", source = "kv:C09E_asBonus@npc/C_Neutral/C09.txt:619" },
          duration = { 15, unit = "s", source = "kv:C09E_duration@npc/C_Neutral/C09.txt:624" },
        },
        text = {
          zh_TW = { name = "白熱", body = "使一名友方英雄或部隊在 {duration} 秒內攻擊速度提高 {attack_speed}。" },
          en = { name = "White Heat", body = "Grants an allied hero or unit +{attack_speed} attack speed for {duration} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "ally", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c09e_old_fervor", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "白熱", body = "攻擊速度提高 {attack_speed}。" },
              en = { name = "White Heat", body = "Attack speed increased by {attack_speed}." },
            },
            reveal = "public", cue = "c09e_old_stomp_glow", stats = { attack_speed = "$attack_speed" },
            notes = "原作 modifier_C09E_old：ATTACKSPEED_BONUS_CONSTANT，時間由腳本以 C09E_duration 給；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = { { op = "apply_status", status = "c09e_old_fervor", to = "target" } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C09.txt:572-577",
            text = "和本體不同：舊版是對友方單位施放（TEAM_FRIENDLY、HERO|BASIC，可以點自己），只給攻速，沒有治療（C09E_heal 是 0，腳本的回血與跳字都已註解）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C09.lua:270-274",
            text = "目標名稱在 _G.EXCLUDE_TARGET_NAME 時不套狀態、EndCooldown（法力不退）；目標選取預設排除 untargetable_by_spells，資料不寫這個分支。" },
        },
        provenance = { "npc/C_Neutral/C09.txt:571-628", "vscripts/heroes/C_Neutral/C09.lua:264-276" },
      },
      -------------------------------------------------------------------------
      { id = "c09_old_ice_soul_dance", source_id = "C09R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 16, 14, 12, 10, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C09.txt:640" },
          mana_cost = { 70, 80, 90, 100, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C09.txt:642" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C09.txt:641" },
          distance = { 600, 700, 800, 900, unit = "dist", source = "kv:C09R_distance@npc/C_Neutral/C09.txt:683" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:C09R_damage@npc/C_Neutral/C09.txt:688" },
          radius = { 300, unit = "dist", source = "kv:C09R_radius@npc/C_Neutral/C09.txt:693" },
          duration = { 6, 6, 7, 7, unit = "s", source = "kv:C09R_debuffDuration@npc/C_Neutral/C09.txt:698" },
          slow = { 30, unit = "pct", kv = { -30 }, source = "kv:C09R_debuffAsMs@npc/C_Neutral/C09.txt:703",
            source_differs = "KV -30 同時給 MOVESPEED_BONUS_PERCENTAGE；這裡以正值表示降低量" },
          attack_slow = { 30, unit = "atk_speed", kv = { -30 }, source = "kv:C09R_debuffAsMs@npc/C_Neutral/C09.txt:703",
            source_differs = "KV -30 同時給 ATTACKSPEED_BONUS_CONSTANT；這裡以正值表示降低量" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C09.lua:294 modifier_phased 0.1" },
        },
        text = {
          zh_TW = { name = "冰靈舞魂",
            body = "朝目標地點瞬移最多 {distance}，並朝落點攻擊移動；對落點 {radius} 內非魔免的敵方英雄與部隊造成 {damage} 魔法傷害，並使其移動速度降低 {slow}、攻擊速度降低 {attack_slow}，持續 {duration} 秒。這個減速無法驅散；目標在魔免期間被移除時會立刻補回。" },
          en = { name = "Ice Spirit Dance",
            body = "Blinks up to {distance} toward the target point and attack-moves there, dealing {damage} magic damage to non-spell-immune enemy heroes and units within {radius} of the landing spot and slowing their movement speed by {slow} and attack speed by {attack_slow} for {duration} seconds. The slow cannot be dispelled, and if it is removed while the target is spell immune it is reapplied at once." },
        },
        target = { type = "point", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c09r_old_slow", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "debuff",
            text = {
              zh_TW = { name = "冰靈舞魂緩速", body = "移動速度降低 {slow}、攻擊速度降低 {attack_slow}。無法驅散，在魔免期間被移除時會立刻補回。" },
              en = { name = "Ice Spirit Dance Slow", body = "Movement speed reduced by {slow} and attack speed reduced by {attack_slow}. Cannot be dispelled; reapplied at once if removed while spell immune." },
            },
            reveal = "public", cue = "c09r_old_ice",
            stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            triggers = {
              on_removed = { filter = "status.remaining > 0 and self.is_magic_immune", actions = {
                { op = "apply_status", status = "c09r_old_slow", to = "self", source = "caster", duration = "status.remaining" },
              } },
            },
            notes = "原作 modifier_C09R_old_debuff：IsPurgable 0；OnDestroy 記錄被移除當下是否魔免，腳本 Timer 每 0.1 秒在魔免時以剩餘時間補回。" },
          { id = "c09r_old_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, notes = "引擎 modifier_phased（AddNewModifier(nil, nil, ...)）。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "dest",
              value = "select(distance(caster, point) > $distance, offset(caster, direction(caster, point), $distance), point)" },
            { op = "teleport", who = "caster", to = "dest" },
            { op = "apply_status", status = "c09r_old_phased", to = "caster" },
            { op = "issue_order", who = "caster", order = "attack_move", at = "dest" },
            { op = "cue", id = "c09r_old_frost_nova", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
              actions = {
                { op = "apply_status", status = "c09r_old_slow", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C09.txt:630-644",
            text = "KV 沒寫 AbilityCastRange：任何距離都能施放，超過 C09R_distance 時沿方向截斷，落點以 FindClearSpaceForUnit 找空位。距離以含高度差的 3D 長度判斷，資料忽略高度。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C09.lua:296",
            text = "和本體不同：舊版沒有 ProjectileDodge；瞬移後對自己下攻擊移動指令（ATTACK_MOVE 到落點），會取消玩家原本的指令。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C09.lua:298-311",
            text = "搜尋以落地後的位置為圓心，用技能設定（ENEMY、HERO|BASIC、旗標 NONE）：魔免敵人選不到；沒有 FOW 旗標。先套減速再 ApplyDamage（不經 AMHC）。" },
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C09.lua:312-325",
            text = "原作每 0.1 秒檢查：減速不見且最後一次移除時目標是魔免，就以 Timer 累減的剩餘時間補回；移除時不是魔免（被強制移除、死亡）就不再補。補回最多晚 0.1 秒；照設計意圖寫成移除當下立即補回（同本體）。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C09.lua:310,329-336",
            text = "C09R_old_state 是存在單位上的欄位，多次施放共用；最新一次移除的結果決定所有 Timer 是否繼續補（同本體）。a34e2 粒子與 FrostNova 音效只是呈現。" },
        },
        provenance = { "npc/C_Neutral/C09.txt:629-707", "vscripts/heroes/C_Neutral/C09.lua:278-336" },
      },
      -------------------------------------------------------------------------
      { id = "c09_old_armor_break", source_id = "C09T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 110, 100, 90, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C09.txt:720" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C09.txt:721" },
          cast_point = { 0, unit = "s", source = "rules:KV 沒寫 AbilityCastPoint（預設 0），且是 IMMEDIATE" },
          required_level = { 6, 12, 18, unit = "level", source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          radius = { 800, unit = "dist", source = "kv:C09T_radius@npc/C_Neutral/C09.txt:768", note = "AbilityCastRange 也是 800" },
          armor = { 42, 54, 66, unit = "armor", kv = { -42, -54, -66 }, source = "kv:C09T_armor@npc/C_Neutral/C09.txt:758",
            source_differs = "KV 以負的護甲加成表示；這裡以正值表示減少量" },
          duration = { 25, 30, 35, unit = "s", source = "kv:C09T_duration@npc/C_Neutral/C09.txt:763" },
        },
        text = {
          zh_TW = { name = "破甲術",
            body = "立即使周圍 {radius} 內的敵方英雄與部隊（包含魔免）護甲降低 {armor}，持續 {duration} 秒。" },
          en = { name = "Armor Break",
            body = "Instantly reduces the armor of enemy heroes and units within {radius} (spell-immune ones included) by {armor} for {duration} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "c09t_old_armor_break", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "破甲術", body = "護甲降低 {armor}。" },
              en = { name = "Armor Break", body = "Armor reduced by {armor}." },
            },
            reveal = "public", cue = "c09t_old_stomp_debuff", stats = { armor = "-$armor" },
            notes = "原作 modifier_C09T_old_debuff：PHYSICAL_ARMOR_BONUS，時間由腳本給；不在 ultimate_modifiers 名單（受狀態抗性縮短）；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c09t_old_title" },
            { op = "cue", id = "c09t_old_burst", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                flags = { "include_magic_immune" } },
              actions = {
                { op = "apply_status", status = "c09t_old_armor_break", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C09.txt:710-767",
            text = "和本體不同：舊版沒有被動，只有主動；護甲降低 42/54/66、持續 25/30/35 秒，數值遠大於本體，但就是原作 KV。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C09.lua:338-352",
            text = "以施法者為圓心、帶 MAGIC_IMMUNE_ENEMIES、排除 _G.EXCLUDE_TARGET_NAME（對應選取預設排除 untargetable_by_spells）；沒有 FOW 旗標，看不見的敵人也算。" },
          { policy = "note", ref = "npc/C_Neutral/C09.txt:724-737",
            text = "showTitle 顯示大招標題、b05t3 粒子只是呈現，改用 cue。" },
        },
        provenance = { "npc/C_Neutral/C09.txt:709-770", "vscripts/heroes/C_Neutral/C09.lua:338-352" },
      },
    },
  }
end
