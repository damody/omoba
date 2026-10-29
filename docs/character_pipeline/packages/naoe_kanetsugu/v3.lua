-- B36 直江兼續（current 版，heromap_skill B36 ["16"] = "WERDT"，autoskill ["16"] = ""）— package schema v3。
-- 原作槽位 W/E/R/D/T → Q/W/E/D/R（npc/B_Unified/B36_2.txt:17-23）。*_old 是 11 版，不列入。
-- B36D（履及）是 NOT_LEARNABLE，由 B36W 施放時 SetLevel(1) 並啟用 5 秒（vscripts/heroes/B_Unified/B36.lua:9-14）。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B36_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"（v3.md §12.3 推定）。
-- nobu 緩速池（dummy.lua Slow／ReturnSpeed）照 A04／A16 寫成 slow_pct，狀態帶 tag slow；合成與緩速抗性屬規則層。
-- ApplyDamage（不檢查施法者存活）寫 require_source_alive = false；AMHC:Damage 會檢查，用預設。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B36",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "B36",
      text = { zh_TW = { name = "直江兼續" }, en = { name = "Naoe Kanetsugu" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b36_naoe_seal", source_id = "B36W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 12, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, 100, 120, 140, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 550, 700, 850, 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 400, unit = "dist", source = "kv:radius" },
          delay = { 1, unit = "s", source = "kv:delaytime" },
          damage = { 300, 400, 500, 600, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          slow = { 30, unit = "pct", kv = { -30 }, source = "kv:modifier_B36W ms_slow",
            source_differs = "KV 以負值表示減速，v3 以正值表示", note = "dummy.lua Slow 的 ms_slow，nobu 緩速池" },
          slow_time = { 4, unit = "s", source = "kv:modifier_B36W Duration" },
          mark_duration = { 5, unit = "s", source = "kv:mark_duration", note = "腳本 Timer 5 與 AddFOWViewer 5 寫死同值" },
          vision_radius = { 500, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B36.lua:15 AddFOWViewer 500" },
        },
        text = {
          zh_TW = { name = "直江印",
            body = "在目標地點留下直江印，{delay} 秒後打擊 {radius} 範圍內非魔免的敵方英雄與部隊，造成 {damage} 魔法傷害並降低 {slow} 移動速度 {slow_time} 秒。直江印留存 {mark_duration} 秒並提供視野，期間可以施放{@b36_footsteps}。" },
          en = { name = "Naoe Seal",
            body = "Places the Naoe Seal at the target point. After {delay} seconds it strikes non-spell-immune enemy heroes and units within {radius}, dealing {damage} magic damage and slowing them by {slow} for {slow_time} seconds. The seal lasts {mark_duration} seconds and grants vision, during which {@b36_footsteps} can be cast." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b36w_slow", duration = "$slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "直江印", body = "移動速度降低 {slow}。" },
              en = { name = "Naoe Seal", body = "Movement speed reduced by {slow}." },
            },
            reveal = "public", tags = { "slow" }, cue = "b36w_move_slow",
            stats = { slow_pct = "$slow" },
            notes = "原作 modifier_B36W：Duration 4；OnCreated 以 dummy.lua Slow 寫進 nobu 緩速池（名稱 modifier_B36W）；沒寫 IsPurgable，暫定 basic。" },
          { id = "b36w_mark", duration = "$mark_duration", stacking = "replace", dispel = "none", visibility = "hidden",
            reveal = "public", scope = "hero",
            vars = { at = { type = "point", reveal = "public" } },
            text = { zh_TW = { name = "直江印" }, en = { name = "Naoe Seal" } },
            notes = "代表 caster.B36D_point 與 B36D 的 SetActivated(true) 五秒窗口。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "b36w_seal", at = "at", duration = "$mark_duration" },
            { op = "set_ability_level", of = "caster", ability = "b36_footsteps", level = "1" },
            { op = "apply_status", status = "b36w_mark", to = "caster", vars = { at = "at" } },
            { op = "reveal", at = "at", radius = "$vision_radius", duration = "$mark_duration", teams = "own" },
            { op = "delay", time = "$delay", capture = { "at" }, actions = {
              { op = "cue", id = "b36w_strike", at = "at" },
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                actions = {
                  { op = "apply_status", status = "b36w_slow", to = "it" },
                  { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false, flags = { "no_amplify" } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B36.lua:19-44",
            text = "打擊以 ApplyDamage 結算，不檢查施法者存活：兼續在延遲中死亡仍會造成傷害（require_source_alive = false）。旗標 0，魔免單位本來就選不到，腳本的 IsMagicImmune 檢查是多餘的。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B36.lua:11-14",
            text = "每次施放各自一個 5 秒 timer 把履及停用；若冷卻被重置而在 5 秒內再施放，前一次的 timer 會提早關掉新的窗口。資料以 replace 狀態表達，不重現這個交錯。" },
          { policy = "note", ref = "npc/B_Unified/B36.txt:103-122",
            text = "AbilitySpecial 兩筆都用鍵 \"03\"（damage 與 radius）；原作可玩，假設兩者都保留。" },
          { policy = "note", ref = "npc/B_Unified/B36.txt:80-83", text = "註解掉的 Properties 不生效，緩速走 nobu 緩速池。" },
        },
        provenance = { "npc/B_Unified/B36.txt:24-123", "vscripts/heroes/B_Unified/B36.lua:4-44" },
      },
      -------------------------------------------------------------------------
      { id = "b36_footsteps", source_id = "B36D", slot = "D", kind = "active", max_level = 1, learnable = false,
        values = {
          cooldown = { 1, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 25, unit = "mana", source = "kv:AbilityManaCost" },
        },
        text = {
          zh_TW = { name = "履及", body = "立刻移動到{@b36w_mark}所在的位置。" },
          en = { name = "Footsteps", body = "Instantly moves to the location of the {@b36w_mark}." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { has_status = "b36w_mark" },
        events = {
          on_cast = {
            { op = "let", name = "dest", value = "status_var(caster, b36w_mark, at)" },
            { op = "remove_status", from = "caster", status = "b36w_mark" },
            { op = "teleport", who = "caster", to = "dest", find_clear_space = true },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B36.lua:47-51",
            text = "沒有距離限制，FindClearSpaceForUnit 直接移到印記位置；使用後立即停用，同一個印記只能用一次。學會前（等級 0）不能用，第一次施放直江印時才升到一級。" },
          { policy = "as_source",
            text = "印記窗口存在英雄層級：兼續在窗口內死亡不會清掉它（scope = hero），復活時若還在五秒內仍可使用。" },
        },
        provenance = { "npc/B_Unified/B36.txt:124-144", "vscripts/heroes/B_Unified/B36.lua:4-16,47-51" },
      },
      -------------------------------------------------------------------------
      { id = "b36_heart_fence", source_id = "B36E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          radius = { 400, unit = "dist", source = "kv:radius" },
          duration = { 4, unit = "s", source = "kv:modifier_B36E Duration" },
          damage_reduction = { 25, unit = "pct", kv = { -25 },
            source = "kv:modifier_B36E MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE",
            source_differs = "KV 以負值表示減傷，v3 以正值表示", note = "原作說明寫減少 50% 傷害，KV 是 -25" },
          absorb = { 250, 380, 510, 640, unit = "dmg", source = "kv:damage_absorb" },
          mana_pct = { 20, unit = "pct", source = "kv:addition_absord_mul_mana_percent" },
          armor = { 4, 6, 8, 10, unit = "armor", source = "kv:aromrbonus" },
        },
        text = {
          zh_TW = { name = "劃心為籬",
            body = "為自身與周圍 {radius} 內的其他友方英雄（不含分身）各架起一面護盾，持續 {duration} 秒：受到的傷害降低 {damage_reduction}，並吸收受到的傷害，共可吸收 {absorb} 加上兼續當下法力 {mana_pct}。若周圍沒有其他友方英雄，兼續額外獲得 {armor} 護甲，護盾耗盡時一併消失。" },
          en = { name = "Heart Fence",
            body = "Grants a shield to Kanetsugu and every other allied hero (illusions excluded) within {radius} for {duration} seconds: damage taken is reduced by {damage_reduction}, and the shield absorbs damage taken, up to {absorb} plus {mana_pct} of Kanetsugu's current mana. If no other allied hero is nearby, Kanetsugu also gains +{armor} armor, which ends when his shield breaks." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "0", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b36e_shield", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "劃心為籬", body = "受到的傷害降低 {damage_reduction}，並吸收受到的傷害，共 {absorb} 加上兼續施放時法力 {mana_pct}；耗盡時消失。" },
              en = { name = "Heart Fence", body = "Takes {damage_reduction} less damage and absorbs damage taken, up to {absorb} plus {mana_pct} of Kanetsugu's mana at cast; ends when depleted." },
            },
            reveal = "public", cue = "b36e_shield_fx",
            stats = { incoming_damage_pct = "-$damage_reduction" },
            -- FC-002／FC-032（D13）：結算前吸收（擋得住致死傷害），每個護盾自己的剩餘量（不再共用 AphoticShieldRemaining）
            absorb = { amount = "$absorb + caster.mana * $mana_pct",
              on_broken = {
                { op = "if", cond = "self == caster", actions = {
                  { op = "remove_status", from = "self", status = "b36e_armor" },
                } },
              } },
            notes = "原作 modifier_B36E：OnCreated 把 unit.AphoticShieldRemaining 設成護盾量，OnTakeDamage 以 SetHealth 把該次傷害補回；修正後改成 absorb（耗盡時狀態連同減傷一起移除）。沒寫 IsPurgable，暫定 basic。" },
          { id = "b36e_armor", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "劃心為籬", body = "護甲 +{armor}；兼續的護盾耗盡時一併消失。" },
              en = { name = "Heart Fence", body = "+{armor} armor; ends when Kanetsugu's shield breaks." },
            },
            reveal = "public", stats = { armor = "$armor" },
            notes = "原作 modifier_B36E_aromr：Duration 4、PHYSICAL_ARMOR_BONUS；沒寫 IsPurgable，暫定 basic。" },
        },
        unit_state = {
          b36e_allies = { type = "int", init = "0", reveal = "public", scope = "ability",
            note = "原作 B36E_OnSpellStart 的區域變數 counter：套到的其他友方真英雄數。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b36e_cast", at = "caster" },
            { op = "set_state", var = "b36e_allies", value = "0" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "ally_not_self", types = { "hero" },
                flags = { "exclude_illusions" } },
              actions = {
                { op = "set_state", var = "b36e_allies", value = "state.b36e_allies + 1" },
                { op = "apply_status", status = "b36e_shield", to = "it" },
              } },
            { op = "if", cond = "state.b36e_allies == 0", actions = {
              { op = "apply_status", status = "b36e_armor", to = "caster" },
            } },
            { op = "apply_status", status = "b36e_shield", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B36.lua:89-110",
            text = "FC-032（D13 修正：改成 absorb，在傷害結算前吸收，擋得住致死傷害）。原作：護盾不是吸收，而是在 OnTakeDamage（傷害已結算）之後以 SetHealth 把血補回：一次致死的傷害照樣打死目標，護盾擋不住。原意應是吸收傷害。" },
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B36.lua:85",
            text = "FC-002（D13 修正：每個護盾用 absorb 自己的剩餘量，不再共用 AphoticShieldRemaining）。原作：剩餘量存在 unit.AphoticShieldRemaining，A04、A24、B08、C06、C12 與道具河童之殼也用同一個欄位：同一個單位身上同時有兩種護盾時互相覆寫剩餘量、兩邊都依它補血。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B36.lua:82",
            text = "護盾量讀的是兼續在 OnCreated 當下（已扣本次法力）的法力，不是最大法力；每個被套用的友軍各算一次，各自獨立。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B36.lua:103-113",
            text = "原作機制（FC-032 修正前）：補血用 SetHealth，不算治療（不受減療、不觸發治療事件）；傷害超過剩餘量時只補剩餘量並移除護盾。減傷 25% 在護盾消失後也一併消失。" },
          { policy = "note", ref = "npc/B_Unified/B36.txt:154-156",
            text = "選取用技能的目標設定：友方 HERO、旗標 0（不含魔免以外的特殊排除）。KV 的 %attack_damage 在 OnTakeDamage 推定為最終傷害（event.damage.final）。" },
        },
        provenance = { "npc/B_Unified/B36.txt:146-240", "vscripts/heroes/B_Unified/B36.lua:53-123" },
      },
      -------------------------------------------------------------------------
      { id = "b36_seven_spears", source_id = "B36R", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, 8, 6, 4, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 190, 165, 140, 115, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 300, unit = "dist", source = "kv:AbilityCastRange", note = "腳本以 GetCastRange() 當搜尋半徑" },
          damage = { 250, 350, 450, 550, unit = "dmg", damage_type = "magical", source = "kv:B36R_damage" },
          fly_time = { 1.2, unit = "s", source = "kv:B36R_flyTime" },
          fly_height = { 500, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B36.lua:167 knockback_height = 500" },
          attack_speed = { 40, 50, 60, 70, unit = "atk_speed", source = "kv:B36R_attack_speed" },
        },
        text = {
          zh_TW = { name = "技擊．七柱槍",
            body = "被動：攻擊速度 +{attack_speed}。啟動：以槍術打擊自身周圍 {radius} 內的敵方英雄與部隊，造成 {damage} 魔法傷害，並把它們擊飛到半空、暈眩 {fly_time} 秒。" },
          en = { name = "Seven Spears",
            body = "Passive: +{attack_speed} attack speed. Active: strikes enemy heroes and units within {radius}, dealing {damage} magic damage and knocking them into the air, stunning them for {fly_time} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        passive = {
          reveal = "public",
          stats = { attack_speed = "$attack_speed" },
        },
        events = {
          on_cast = {
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
              actions = {
                { op = "cue", id = "b36r_strike", at = "it" },
                { op = "knockback", who = "it", from = "it", distance = "0", duration = "$fly_time",
                  height = "$fly_height", stun = true },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false, flags = { "no_amplify" } },
              } },
            { op = "cue", id = "b36r_attack_gesture", at = "caster" },
            { op = "delay", time = "$fly_time", actions = { { op = "cue", id = "b36r_land_sound", at = "caster" } } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B36.lua:130-138",
            text = "搜尋半徑是 ability:GetCastRange()（KV 300）；施法距離加成是否影響這個值依引擎實作，推定不受影響。旗標 0，魔免敵人不受影響。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B36.lua:160-175",
            text = "先套 modifier_knockback（原地、高度 500、1.2 秒、should_stun）再以 ApplyDamage 造成傷害，不檢查施法者存活（require_source_alive = false）。" },
          { policy = "as_source", ref = "npc/B_Unified/B36.txt:290-299",
            text = "modifier_B36R 是 Passive 1，學會就永久加攻速；它列在 _G.EXCLUDE_MODIFIER_NAME，被動的隱含狀態本來就不可驅散。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B36.lua:178-190",
            text = "B36R_DelayedAction（落地依損失生命造成傷害）沒有被呼叫，B36R_damage_on_ground 也不存在；KV 的 DelayedAction 只播音效。" },
        },
        provenance = { "npc/B_Unified/B36.txt:241-322", "vscripts/heroes/B_Unified/B36.lua:126-176" },
      },
      -------------------------------------------------------------------------
      { id = "b36_aizen_myoo", source_id = "B36T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 105, 90, 75, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel，DOTA_ABILITY_TYPE_ULTIMATE 的引擎預設 6/12/18" },
          duration = { 25, unit = "s", source = "kv:modifier_B36T Duration" },
          bonus_hp = { 1000, 1300, 1600, unit = "hp", source = "kv:Bonus_Health" },
          base_attack_time = { 1.35, unit = "s", source = "kv:modifier_B36T MODIFIER_PROPERTY_BASE_ATTACK_TIME_CONSTANT" },
          damage = { 120, 180, 240, unit = "dmg", damage_type = "pure", source = "kv:damage" },
        },
        text = {
          zh_TW = { name = "愛染明王",
            body = "獲得愛染明王之力 {duration} 秒：最大生命 +{bonus_hp} 並立即回復同量生命，基礎攻擊間隔變為 {base_attack_time} 秒，每次普攻命中非建築目標時額外造成 {damage} 乘以（一減目標基礎魔抗）的純粹傷害。" },
          en = { name = "Aizen Myoo",
            body = "Gains the power of Aizen Myoo for {duration} seconds: +{bonus_hp} maximum health and heals for the same amount, base attack time becomes {base_attack_time} seconds, and each attack that lands on a non-building target deals an extra {damage} × (one minus the target's base magic resistance) pure damage." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "b36t_aizen", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "愛染明王", body = "最大生命 +{bonus_hp}，基礎攻擊間隔變為 {base_attack_time} 秒，普攻命中非建築目標時額外造成 {damage} 乘以（一減目標基礎魔抗）的純粹傷害。不能被驅散。" },
              en = { name = "Aizen Myoo", body = "+{bonus_hp} maximum health, base attack time of {base_attack_time} seconds, and attacks that land on non-building targets deal an extra {damage} × (one minus the target's base magic resistance) pure damage. Cannot be dispelled." },
            },
            reveal = "public", cue = "b36t_aura_fx",
            stats = { max_hp = "$bonus_hp", base_attack_time = { op = "set_absolute", value = "$base_attack_time" } },
            triggers = {
              on_attack_hit = { filter = "not event.target.is_building", actions = {
                { op = "damage", to = "event.target", source = "self", type = "pure",
                  amount = "$damage * (1 - event.target.base_magic_resist)" },
              } },
            },
            notes = "原作 modifier_B36T：IsPurgable 0、AllowIllusionDuplicate 1、Duration 25；OVERRIDE_ANIMATION_RATE 3、TRANSLATE_ACTIVITY_MODIFIERS alternate 屬呈現。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b36t_aizen", to = "caster" },
            { op = "heal", to = "caster", source = "caster", amount = "$bonus_hp" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B36.lua:195-205",
            text = "額外傷害是純粹但乘上 (1−base 魔抗)：只看 nobu 魔抗池，裝備魔抗不算，魔免單位照樣受傷；以 AMHC:Damage 結算，兼續死亡時不造成。變數 health（5% 最大生命）算了沒用。" },
          { policy = "as_source", ref = "npc/B_Unified/B36.txt:347-372",
            text = "AllowIllusionDuplicate 1：施放期間產生的分身也帶這個狀態，分身攻擊同樣造成額外純粹傷害（來源是分身本身）。" },
          { policy = "as_source", ref = "npc/B_Unified/B36.txt:340-345",
            text = "先套 modifier 再 Heal：最大生命先提高再回復 Bonus_Health。" },
          { policy = "note", ref = "npc/B_Unified/B36.txt:349",
            text = "modifier 裡的 AttackAnimationPoint 0.2 不是 datadriven modifier 支援的鍵，推定沒有效果。" },
        },
        provenance = { "npc/B_Unified/B36.txt:324-420", "vscripts/heroes/B_Unified/B36.lua:195-205" },
      },
    },
  }
end
