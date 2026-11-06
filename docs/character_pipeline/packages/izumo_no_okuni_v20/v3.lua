-- C04 阿國（2.0 版，原作 *_20 技能組）— package schema v3。每個版本是一位獨立英雄（D22）；舊版另成 izumo_no_okuni_old。
-- 槽位依據：選 2.0 版時 choose_20 拔掉全部技能，依 heromap_skill C04 ["20"] = "WERDT" 逐一 AddAbility("C04" .. 字母 .. "_20")，
--   第五個字母前插入 attribute_bonusx；heromap_autoskill C04 ["20"] = "D" 自動學 C04D_20
--   （vscripts/items/Addon_Items/tofar_goback.lua:6-48、vscripts/addon_game_mode.lua:320,423）。
-- W/E/R/T → Q/W/E/R；C04D_20 照慣例放 D，寫在 hero.auto_learn。
-- KV 在 npc/C_Neutral/C04.txt:707-1109；腳本 vscripts/heroes/C_Neutral/C04.lua:37-175（與本體共用）、
--   vscripts/items/Addon_Items/item_devil_supressor_armor.lua:27-68,119-155。原作沒有 *_20 招式的說明文字（resource/addon_*.txt 查無），
--   名稱取自 KV 註解（D 的註解寫錯成「巫魅之舞」，見 assumptions.md），說明依 KV 與腳本還原。英雄單位數值沿用本體 C04。
-- magical_resistance、attribute_bonusx、C04_precache 屬規則層或資產，不列入。
-- 慣例照本體 C04：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
--   datadriven Properties 直接給的移速百分比寫 move_speed_pct；HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C04_v20",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "v20" },
    hero = { source_id = "C04", text = { zh_TW = { name = "阿國（2.0）" }, en = { name = "Izumo no Okuni (2.0)" } },
      auto_learn = { "c04_v20_guardian_dance" } },
    unit_state = {
      c04_v20_e_count = { type = "int", init = "0", reveal = "public", scope = "unit",
        note = "凋零舞步每跳選到的目標數（原作 #units），只在同一跳內使用" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c04_v20_swift_dance", source_id = "C04W_20", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C04.txt:717" },
          mana_cost = { 100, 130, 160, 190, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C04.txt:718" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C04.txt:716" },
          radius = { 10000, unit = "dist", hidden = true, source = "kv:ApplyModifier Target Radius@npc/C_Neutral/C04.txt:739" },
          move_speed_pct = { 30, unit = "pct", source = "kv:modifier_C04W MOVESPEED_BONUS_PERCENTAGE@npc/C_Neutral/C04.txt:752" },
          duration = { 3, 4, 5, 6, unit = "s", source = "kv:duration@npc/C_Neutral/C04.txt:767" },
        },
        text = {
          zh_TW = { name = "神速之舞",
            body = "全地圖所有友方英雄與部隊（含自己）移動速度提高 {move_speed_pct}，持續 {duration} 秒。" },
          en = { name = "Dance of Swiftness",
            body = "Every allied hero and unit on the map (including herself) gains {move_speed_pct} movement speed for {duration} seconds." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c04w_20_haste", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "神速之舞", body = "移動速度提高 {move_speed_pct}。" },
              en = { name = "Dance of Swiftness", body = "Movement speed increased by {move_speed_pct}." },
            },
            reveal = "public", cue = "c04w_20_wind", stats = { move_speed_pct = "$move_speed_pct" },
            notes = "原作 modifier_C04W（C04W_20 版）：MOVESPEED_BONUS_PERCENTAGE 30、Duration %duration；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c04w_20_sound", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "ally", types = BASIC },
              actions = {
                { op = "apply_status", status = "c04w_20_haste", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C04.txt:733-743",
            text = "2.0 版沒有腳本（本體走 C04W_OnSpellStart）：KV ApplyModifier 直接以阿國為中心、半徑 10000 套給友方 HERO|BASIC（地圖內幾乎全部友軍），施加者是阿國。旗標預設：無敵友軍、魔免友軍依引擎預設（推定拿得到）。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C04.txt:713-719,762",
            text = "KV AbilityUnitTargetTeam ENEMY、AbilityUnitDamageType、AbilityCastRange 700、radius 350 都沒有被使用。" },
        },
        provenance = { "npc/C_Neutral/C04.txt:707-770" },
      },
      -------------------------------------------------------------------------
      { id = "c04_v20_withering_step", source_id = "C04E_20", slot = "W", kind = "toggle", max_level = 4, tags = { "basic", "toggle" },
        values = {
          cooldown = { 2, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C04.txt:783" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C04.txt:782" },
          radius = { 500, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C04.lua:69 ability:GetCastRange()（KV AbilityCastRange 500@C04.txt:786）" },
          tick = { 1, unit = "s", source = "kv:modifier_C04E ThinkInterval@npc/C_Neutral/C04.txt:846" },
          mana_per_tick = { 12, 16, 20, 25, unit = "mana", source = "kv:mana@npc/C_Neutral/C04.txt:882", note = "OnIntervalThink 以 SpendMana 每跳扣除" },
          damage = { 70, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C04.txt:785",
            note = "D22 修正：原作腳本讀不存在的 special damage（0）；照設計意圖用 AbilityDamage" },
          hp_pct = { 3, 4, 5, 6, unit = "pct", source = "kv:dmghp@npc/C_Neutral/C04.txt:877", note = "KV 寫成比例 0.03～0.06" },
          int_pct = { 0.02, unit = "pct", source = "script:vscripts/heroes/C_Neutral/C04.lua:57 GetIntellect()/50*0.01（每點智力 0.02%）" },
          single_mult = { 125, unit = "pct", source = "script:vscripts/heroes/C_Neutral/C04.lua:79-81 #units == 1 時 ×1.25" },
          creep_cap = { 600, unit = "dmg", source = "script:vscripts/heroes/C_Neutral/C04.lua:82-84 非英雄上限 600" },
          slow = { 13, unit = "pct", kv = { -13 }, source = "kv:modifier_C04E_2 MOVESPEED_BONUS_PERCENTAGE@npc/C_Neutral/C04.txt:866",
            source_differs = "KV 以負的移速百分比表示；這裡以正值表示降低量" },
          slow_duration = { 1.1, unit = "s", source = "script:vscripts/heroes/C_Neutral/C04.lua:94 duration = 1.1" },
          lock_time = { 1, unit = "s", source = "kv:modifier_C04E_locker Duration@npc/C_Neutral/C04.txt:824" },
        },
        text = {
          zh_TW = { name = "凋零舞步",
            body = "開啟後每 {tick} 秒消耗 {mana_per_tick} 法力；扣完後剩餘法力少於這個數就自動關閉，該跳沒有效果。否則對周圍 {radius} 內非魔免的敵方英雄與部隊造成 {damage} 加上目標最大生命 {hp_pct}（阿國每點智力再加 {int_pct}）的魔法傷害，並使其移動速度降低 {slow}，持續 {slow_duration} 秒。只打到一個目標時傷害為 {single_mult}；對非英雄單位每跳最多 {creep_cap}。開啟後 {lock_time} 秒內不能關閉。" },
          en = { name = "Withering Step",
            body = "While active, spends {mana_per_tick} mana every {tick} seconds; if her remaining mana is then below that amount, the ability turns off and that pulse has no effect. Otherwise, non-spell-immune enemy heroes and units within {radius} take {damage} plus {hp_pct} of their maximum health (plus {int_pct} per point of Okuni's intelligence) as magic damage and are slowed by {slow} for {slow_duration} seconds. If only one target is hit the damage is {single_mult}; non-heroes take at most {creep_cap} per pulse. It cannot be turned off for {lock_time} seconds after being turned on." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c04e_20_dance", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "凋零舞步", body = "每 {tick} 秒消耗 {mana_per_tick} 法力，對周圍 {radius} 內的敵人造成魔法傷害並緩速；法力不足時自動關閉。" },
              en = { name = "Withering Step", body = "Every {tick} seconds spends {mana_per_tick} mana to deal magic damage to enemies within {radius} and slow them; turns off when mana runs short." },
            },
            reveal = "public", cue = "c04e_20_steam", remove_on = { "death" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "spend_mana", from = "self", amount = "$mana_per_tick", ability = "ability" },
                { op = "if", cond = "self.mana < $mana_per_tick",
                  actions = { { op = "toggle_ability", of = "self", ability = "c04_v20_withering_step", on = false } },
                  otherwise = {
                    { op = "set_state", var = "c04_v20_e_count", value = "0", on = "self" },
                    { op = "for_each",
                      select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                      actions = { { op = "set_state", var = "c04_v20_e_count", value = "state.c04_v20_e_count + 1", on = "self" } } },
                    { op = "for_each",
                      select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                      actions = {
                        { op = "let", name = "d",
                          value = "($damage + it.max_hp * ($hp_pct + self.int * $int_pct)) * select(state.c04_v20_e_count == 1, $single_mult, 1)" },
                        { op = "damage", to = "it", type = "magical", require_source_alive = false,
                          amount = "select(it.is_hero, d, min(d, $creep_cap))" },
                        { op = "apply_status", status = "c04e_20_slow", to = "it" },
                      } },
                  } },
              } },
            },
            notes = "原作 modifier_C04E（C04E_20 版）：ThinkInterval 1，OnIntervalThink 先 SpendMana 再跑 C04E_OnIntervalThink（與本體共用）；沒寫 IsPurgable，暫定 basic。" },
          { id = "c04e_20_slow", duration = "$slow_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "凋零舞步緩速", body = "移動速度降低 {slow}。" },
              en = { name = "Withering Step Slow", body = "Movement speed reduced by {slow}." },
            },
            reveal = "public", cue = "c04e_20_steam_debuff", stats = { move_speed_pct = "-$slow" },
            notes = "原作 modifier_C04E_2（C04E_20 版）：Properties 直接 −13% 移速；KV 誤標 IsBuff 1。" },
          { id = "c04e_20_lock", duration = "$lock_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_created = { { op = "set_ability_active", of = "self", ability = "c04_v20_withering_step", active = false } },
              on_removed = { { op = "set_ability_active", of = "self", ability = "c04_v20_withering_step", active = true } },
            },
            notes = "原作 modifier_C04E_locker：SetActivated(false)，OnDestroy 恢復。" },
        },
        events = {
          on_toggle_on = {
            { op = "apply_status", status = "c04e_20_dance", to = "caster" },
            { op = "apply_status", status = "c04e_20_lock", to = "caster" },
          },
          on_toggle_off = { { op = "remove_status", from = "caster", status = "c04e_20_dance" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/C_Neutral/C04.lua:58; npc/C_Neutral/C04.txt:785,873-884",
            text = "FC-156 D22 舊版，比照 D13 修：C04E_OnIntervalThink 以 GetSpecialValueFor(\"damage\") 取固定傷害，但 C04E_20 的 AbilitySpecial 只有 dmghp 與 mana、沒有 damage，取到 0；KV 另寫了沒人讀的 AbilityDamage 70。修法：照設計意圖以 AbilityDamage 70 當固定傷害。原作實際每跳只有最大生命百分比的部分。" },
          { policy = "as_source", ref = "npc/C_Neutral/C04.txt:843-857",
            text = "每跳先 SpendMana 扣法力，腳本才檢查「剩餘法力 < 每跳消耗」並關閉；所以剩餘法力介於一倍與兩倍之間時，這一跳扣了法力卻沒有傷害就關閉。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C04.lua:57,77-92",
            text = "百分比傷害 = dmghp + 智力/50×0.01；×1.25 看這一跳選到的總數是否恰好一個；非英雄上限 600 在乘完之後套用。ApplyDamage 不經 AMHC。選取用技能設定（ENEMY、BASIC|HERO、旗標 NONE），魔免敵人選不到。" },
          { policy = "as_source", ref = "npc/C_Neutral/C04.txt:782-786",
            text = "和本體不同：切換冷卻 2 秒（本體 0）、每跳 12/16/20/25 法力、最大生命 3~6%（本體 1~4%）。" },
          { policy = "note", ref = "npc/C_Neutral/C04.txt:788-808",
            text = "ApplyModifier 的 Target 只有 Center = CASTER 與旗標、沒有 Radius，依引擎行為只套到施法者本人。" },
          { policy = "as_source",
            text = "開關開著死亡時，datadriven 狀態隨死亡移除但開關仍亮著，復活後要重新開關才有效果（推測，同本體）；資料以 remove_on death 表達。" },
        },
        provenance = { "npc/C_Neutral/C04.txt:771-885", "vscripts/heroes/C_Neutral/C04.lua:37-97" },
      },
      -------------------------------------------------------------------------
      { id = "c04_v20_gathering_dance", source_id = "C04R_20", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 21, 19, 17, 15, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C04.txt:897" },
          mana_cost = { 115, 140, 165, 190, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C04.txt:898" },
          cast_range = { 850, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C04.txt:899" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C04.txt:896" },
          radius = { 450, 500, 550, 600, unit = "dist", source = "kv:radius@npc/C_Neutral/C04.txt:926" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C04.txt:929" },
          stun = { 0.1, unit = "s", source = "script:vscripts/heroes/C_Neutral/C04.lua:124 modifier_stunned duration = 0.1" },
          phase_time = { 0.3, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C04.lua:122-123 modifier_phased／modifier_flood_walk 0.3" },
        },
        text = {
          zh_TW = { name = "聚曳之舞",
            body = "把目標地點 {radius} 範圍內的所有敵方英雄與部隊（包含魔免）瞬間拉到中心點，暈眩 {stun} 秒並造成 {damage} 魔法傷害。" },
          en = { name = "Gathering Dance",
            body = "Instantly drags every enemy hero and unit within {radius} of the target point (spell-immune ones included) to its center, stunning them for {stun} seconds and dealing {damage} magic damage." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "c04r_20_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "聚曳之舞暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Gathering Dance Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned，以目標自己為施加者（AddNewModifier(unit, ability, ...)）。" },
          { id = "c04r_20_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision", "flying_for_pathing" },
            notes = "引擎 modifier_phased ＋ modifier_flood_walk 合併。C04R_20 自己沒有宣告 modifier_flood_walk（Modifiers 區塊是空的），ApplyDataDrivenModifier 推定失敗，只剩 phased；D22 照本體修成合併狀態，見 quirks。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c04r_20_sound", at = "caster" },
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "c04r_20_weave_ray", at = "at" },
            { op = "for_each",
              select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC, flags = { "include_magic_immune" } },
              actions = {
                { op = "teleport", who = "it", to = "at", find_clear_space = false },
                { op = "apply_status", status = "c04r_20_phased", to = "it", source = "it" },
                { op = "apply_status", status = "c04r_20_stun", to = "it", source = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C04.lua:99-135",
            text = "與本體共用 C04R_OnSpellStart，數值也相同：SetAbsOrigin 直接把所有目標疊到同一點（不找空位），0.3 秒 phased 讓他們之後自行散開；魔免敵人也會被拉過去並暈眩，魔法傷害由規則層歸零；ApplyDamage 不經 AMHC。" },
          { policy = "fixed", ref = "npc/C_Neutral/C04.txt:918-920; vscripts/heroes/C_Neutral/C04.lua:123",
            text = "FC-157 D22 舊版，比照 D13 修：C04R_20 的 Modifiers 區塊是空的，ApplyDataDrivenModifier(modifier_flood_walk) 推定套不上（本體 C04R 有宣告），原作被拉到中心的單位只有 0.3 秒 phased、沒有穿越地形。修法：照本體的設計意圖給 0.3 秒 phased＋flying_for_pathing 合併狀態。" },
        },
        provenance = { "npc/C_Neutral/C04.txt:886-930", "vscripts/heroes/C_Neutral/C04.lua:99-135" },
      },
      -------------------------------------------------------------------------
      { id = "c04_v20_witch_dance", source_id = "C04T_20", slot = "R", kind = "channel", max_level = 3, tags = { "ultimate", "channel" },
        values = {
          cooldown = { 105, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C04.txt:943" },
          mana_cost = { 200, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C04.txt:944" },
          cast_range = { 600, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C04.txt:945" },
          channel_time = { 5, unit = "s", source = "kv:AbilityChannelTime@npc/C_Neutral/C04.txt:946" },
          required_level = { 6, 12, 18, unit = "level", source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          duration = { 5, unit = "s", source = "kv:duration@npc/C_Neutral/C04.txt:1065" },
          tick = { 0.95, unit = "s", source = "kv:modifier_C04T ThinkInterval@npc/C_Neutral/C04.txt:1030" },
          damage = { 175, 270, 375, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C04.txt:1068" },
        },
        text = {
          zh_TW = { name = "巫魅之舞",
            body = "對一名敵方英雄或部隊（包含魔免）持續施法，使其暈眩最多 {duration} 秒，期間每 {tick} 秒受到 {damage} 魔法傷害；阿國停止施法時效果立即結束。這個暈眩不受狀態抗性縮短，也不會被一般淨化移除。" },
          en = { name = "Bewitching Dance",
            body = "Channels on an enemy hero or unit (spell-immune ones included), stunning it for up to {duration} seconds and dealing {damage} magic damage every {tick} seconds; the effect ends as soon as Okuni stops channeling. The stun ignores status resistance and cannot be removed by a basic dispel." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { channel_time = "$channel_time", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "c04t_20_stun", duration = "$duration", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "巫魅之舞", body = "暈眩，每 {tick} 秒受到 {damage} 魔法傷害，阿國停止施法時結束。不受狀態抗性縮短。" },
              en = { name = "Bewitching Dance", body = "Stunned and takes {damage} magic damage every {tick} seconds until Okuni stops channeling. Not reduced by status resistance." },
            },
            reveal = "public", status_resistance = "ignore", states = { "stun" }, tags = { "cleanse_protected" },
            cue = "c04t_20_shield",
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "cue", id = "c04t_20_zap", at = "self" },
              } },
            },
            notes = "原作 modifier_C04T（C04T_20 版，與本體同名）：IsPurgable 1、IsStunDebuff 1；依名稱列在 ultimate_modifiers（不受狀態抗性）與 _G.EXCLUDE_MODIFIER_NAME（cleanse_protected）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c04t_20_title" },
            { op = "cue", id = "c04t_20_music", at = "caster" },
            { op = "apply_status", status = "c04t_20_stun", to = "target" },
          },
          on_channel_end = {
            { op = "remove_status", from = "target", status = "c04t_20_stun" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C04.txt:932-946,1065",
            text = "和本體不同：冷卻 105（本體 135/125/115）、暈眩固定 5 秒等於施法時間（本體 4/5/6）、目標 HERO|BASIC（部隊也能指定）。OnChannelFinish 與 OnChannelInterrupted 都移除暈眩。" },
          { policy = "design_intent", ref = "npc/C_Neutral/C04.txt:1030-1037",
            text = "ThinkInterval 0.95 讓 5 秒內跳 5 次（0.95～4.75 秒），第一跳在 0.95 秒後。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C04.lua:137-175",
            text = "與本體共用 C04T：Timer 每 0.2 秒檢查，暈眩消失時只是結束特效；阿國的施法不會因此中斷。datadriven Damage 不經 AMHC。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C04.txt:1010-1022",
            text = "modifier_C04T2（施法者身上，沒有 Duration，永久）的 OnAbilityExecuted 呼叫 C04T2_OnAbilityExecuted，原作腳本裡沒有這個函式，執行期報錯、沒有效果；設計意圖無從得知，不修。" },
          { policy = "note", text = "音效、護盾粒子、稱號與兩人的動作只是呈現，改用 cue。" },
        },
        provenance = { "npc/C_Neutral/C04.txt:931-1069", "vscripts/heroes/C_Neutral/C04.lua:137-175",
          "vscripts/events.lua:246-269", "vscripts/library/common/dummy.lua:376-392" },
      },
      -------------------------------------------------------------------------
      { id = "c04_v20_guardian_dance", source_id = "C04D_20", slot = "D", kind = "active", max_level = 1, learnable = false, tags = { "basic" },
        values = {
          cooldown = { 45, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C04.txt:1082" },
          mana_cost = { 180, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C04.txt:1083" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C04.txt:1084" },
          shield = { 45, unit = "dmg", source = "kv:armor@npc/C_Neutral/C04.txt:1105",
            note = "RunScript SHP1 = SHP2 = %armor；目標是英雄時用 SHP2" },
          duration = { 6, unit = "s",
            source = "script:vscripts/items/Addon_Items/item_devil_supressor_armor.lua:67 ShockTarget(keys, keys.target, 6)",
            note = "KV duration 6@C04.txt:1100 同值，但腳本寫死 6" },
        },
        text = {
          zh_TW = { name = "護身之舞",
            body = "為一名友方英雄（可以是自己，幻象無效）施加護盾，吸收敵方造成的魔法傷害，共 {shield} 點，持續 {duration} 秒。開場自動學會。" },
          en = { name = "Guardian Dance",
            body = "Shields an allied hero (herself included; illusions are unaffected), absorbing {shield} magic damage dealt by enemies for {duration} seconds. Learned automatically at the start." },
        },
        target = { type = "unit", range = "$cast_range", team = "ally", types = { "hero" } },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c04d_20_shield", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "護身之舞", body = "吸收敵方造成的魔法傷害，共 {shield} 點。" },
              en = { name = "Guardian Dance", body = "Absorbs {shield} magic damage dealt by enemies." },
            },
            reveal = "public", cue = "c04d_20_shield_glow", tags = { "magic_shield" },
            absorb = { amount = "$shield", types = { "magical" } },
            notes = "原作 modifier_devil_supressor_armor（制魔腹卷的 Lua modifier）：以 original_damage 扣盾，盾量足夠時把生命設回 0.2 秒前記錄的值，破盾時溢出部分照魔抗結算並移除；重套時重設盾量。Lua modifier 沒寫 IsPurgable，暫定 none。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c04d_20_sound", at = "caster" },
            { op = "if", cond = "not target.is_illusion", actions = {
              { op = "apply_status", status = "c04d_20_shield", to = "target" },
            } },
          },
        },
        coverage = "expressible", confidence = "low",
        quirks = {
          { policy = "fixed", ref = "vscripts/items/Addon_Items/item_devil_supressor_armor.lua:61-68,119-136",
            text = "FC-158 D22 舊版，比照 D13 修：比照 FC-062（同一個 ShockTarget，照意圖給魔法護盾；舊版 C04T_old 同修）。原作：Shock_C04D_20 呼叫 ShockTarget，ability:ApplyDataDrivenModifier 套的 modifier_devil_supressor_armor 是 LinkLuaModifier 的 Lua modifier，C04D_20 的 KV 沒有 Modifiers 區塊，推定套用失敗，接著 FindModifierByName(...).caster 對 nil 取值報錯，技能沒有效果。修法：照腳本意圖對友方英雄（不含幻象）套 6 秒、45 點的魔法護盾。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_devil_supressor_armor.lua:27-57",
            text = "護盾只擋敵方且非反射的魔法傷害，以減免前的 original_damage 扣盾；資料以魔法 absorb 表達（減免後扣盾，盾實際比原作耐打），不重現生命回寫的副作用（同舊版 C04T_old）。" },
          { policy = "as_source", ref = "npc/C_Neutral/C04.txt:1072-1108",
            text = "KV 沒有 NOT_LEARNABLE 與 MaxLevel（預設 4 級可加點），但所有數值只有一級：choose_20 自動學到 1 級後再加點沒有變化。資料寫 max_level 1、learnable = false。AbilitySpecial 的 duration 與 armor 都用鍵名 \"01\"，依變數名稱查找都生效（推定）。" },
          { policy = "note", ref = "npc/C_Neutral/C04.txt:1071",
            text = "KV 註解把這招也寫成「巫魅之舞」（與 T 重名）；名稱「護身之舞」為還原，見 assumptions.md。施法前搖 0。" },
        },
        provenance = { "npc/C_Neutral/C04.txt:1071-1108", "vscripts/items/Addon_Items/item_devil_supressor_armor.lua:1-68,119-155",
          "vscripts/items/Addon_Items/tofar_goback.lua:6-48", "vscripts/addon_game_mode.lua:320,423" },
      },
    },
  }
end
