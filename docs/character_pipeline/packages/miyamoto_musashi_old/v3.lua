-- C21 宮本武藏（舊版，11.2B 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C21"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:337）；11 版開放（addon_game_mode.lua:235），
-- 自動學習 heromap_autoskill["11"] = ""（addon_game_mode.lua:440），沒有 D。W/E/R/T → Q/W/E/R。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 C21*_old），招式名取自 KV 註解（npc/C_Neutral/C21.txt:409、522、605、665），
-- 說明依 KV 與腳本還原，見 assumptions.md。
-- 腳本檔名含空白：vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua。provenance 的格式不允許空白，
-- 所以腳本出處只寫在數值 source 與 quirks 的 ref（同一個檔），provenance 只列 KV（同本體 C21）。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C21_precache 屬規則層或資產，不列入。
-- 慣例照本體 C21：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- ApplyDamage／datadriven Damage 不檢查施法者存活，寫 require_source_alive = false；AMHC:Damage 會檢查，用預設值。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C21_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C21", text = { zh_TW = { name = "宮本武藏（舊版）" }, en = { name = "Miyamoto Musashi (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c21_old_crescent_slash", source_id = "C21W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 7, 6, 5, 4, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C21.txt:422" },
          mana_cost = { 50, 60, 70, 80, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C21.txt:423" },
          duration = { 0.5, unit = "s", source = "kv:duration@npc/C_Neutral/C21.txt:517" },
          tick = { 0.2, unit = "s", source = "kv:aoe_delay@npc/C_Neutral/C21.txt:507" },
          radius = { 250, unit = "dist", source = "kv:aoe_radius@npc/C_Neutral/C21.txt:512" },
          damage = { 100, 150, 200, 250, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C21.txt:520" },
        },
        text = {
          zh_TW = { name = "奧義．弧月斬",
            body = "迴旋斬擊 {duration} 秒，期間魔法免疫。開始時與之後每 {tick} 秒，對周圍 {radius} 內的非魔免敵人造成 {damage} 魔法傷害。" },
          en = { name = "Secret Art: Crescent Moon Slash",
            body = "Spins for {duration} seconds while spell immune. On start and every {tick} seconds after, deals {damage} magic damage to non-spell-immune enemies within {radius}." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c21w_old_blade_fury", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "弧月斬", body = "魔法免疫；每 {tick} 秒對周圍 {radius} 內的敵人造成 {damage} 魔法傷害。" },
              en = { name = "Crescent Moon Slash", body = "Spell immune; deals {damage} magic damage to enemies within {radius} every {tick} seconds." },
            },
            reveal = "public", states = { "magic_immune" }, cue = "c21w_old_blade_fury_fx",
            triggers = {
              on_created = { actions = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                  actions = {
                    { op = "damage", to = "it", source = "self", type = "magical", amount = "$damage", require_source_alive = false },
                  } },
                { op = "cue", id = "c21w_old_impact_sound", at = "self" },
              } },
              periodic = { interval = "$tick", actions = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                  actions = {
                    { op = "damage", to = "it", source = "self", type = "magical", amount = "$damage", require_source_alive = false },
                  } },
                { op = "cue", id = "c21w_old_impact_sound", at = "self" },
              } },
            },
            notes = "原作 modifier_C21W_old：MAGIC_IMMUNE、IGNORE_INVULNERABLE、未寫 IsPurgable（暫定 basic）；OnCreated 與 ThinkInterval 0.2 都呼叫 C21W_old_apply_damage。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c21w_old_voice", at = "caster" },
            { op = "apply_status", status = "c21w_old_blade_fury", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C21.txt:466-486",
            text = "OnCreated 先打一次，之後每 0.2 秒一次，持續 0.5 秒：一般共三跳（0、0.2、0.4 秒）。和本體不同，舊版沒有沉默。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:354-392",
            text = "選取用技能的 TargetTeam／Type（英雄、一般單位）、沒有 MAGIC_IMMUNE 旗標：魔免敵人不受傷；ApplyDamage 不檢查施法者存活。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C21.txt:424",
            text = "AbilityCastRange 250 沒有被使用（NO_TARGET）。" },
          { policy = "note", ref = "npc/C_Neutral/C21.txt:410-424",
            text = "KV 沒寫 AbilityCastPoint，IMMEDIATE、前搖為零。" },
        },
        provenance = { "npc/C_Neutral/C21.txt:409-521" },
      },
      -------------------------------------------------------------------------
      { id = "c21_old_chain_breaker", source_id = "C21E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 17, 15, 13, 11, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C21.txt:531" },
          mana_cost = { 160, 180, 200, 220, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C21.txt:532" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C21.txt:534" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C21.txt:535" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "physical", source = "kv:AbilityDamage@npc/C_Neutral/C21.txt:533" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:stun_time@npc/C_Neutral/C21.txt:551" },
          hits = { 3, unit = "count", source = "kv:attack_count@npc/C_Neutral/C21.txt:541" },
          interval = { 1, unit = "s", source = "kv:attack_delay@npc/C_Neutral/C21.txt:546" },
          base_attack_time = { 1.4, unit = "s", source = "kv:modifier_C21E_old MODIFIER_PROPERTY_BASE_ATTACK_TIME_CONSTANT@npc/C_Neutral/C21.txt:600" },
          tap_damage = { 1, unit = "dmg", damage_type = "pure", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:401 AMHC:Damage 1 pure" },
          jump_offset = { 100, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:428 radius = 100" },
        },
        text = {
          zh_TW = { name = "奧義．連環鋼破",
            body = "對目標敵人造成 {damage} 物理傷害並暈眩 {stun} 秒。之後每 {interval} 秒瞬移到目標身旁並對它下達攻擊命令，共 {hits} 次；追擊期間武藏無敵，基礎攻擊間隔變為 {base_attack_time} 秒。目標或武藏死亡時提早結束。" },
          en = { name = "Secret Art: Chain Steel Breaker",
            body = "Deals {damage} physical damage to the target enemy and stuns it for {stun} seconds. Then every {interval} seconds Musashi blinks next to it and is ordered to attack it, {hits} times in total; during the chase he is invulnerable and his base attack time becomes {base_attack_time} seconds. Ends early if the target or Musashi dies." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c21e_old_combo", duration = "$hits * $interval", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "連環鋼破", body = "追擊中：無敵，基礎攻擊間隔變為 {base_attack_time} 秒。" },
              en = { name = "Chain Steel Breaker", body = "Chasing: invulnerable, base attack time set to {base_attack_time} seconds." },
            },
            reveal = "public", states = { "invulnerable" },
            stats = { base_attack_time = { op = "set_absolute", value = "$base_attack_time" } },
            notes = "原作 modifier_C21E_old：INVULNERABLE、BASE_ATTACK_TIME_CONSTANT 1.4、OVERRIDE_ANIMATION_RATE 3、未寫 IsPurgable（暫定 basic）；時間 attack_count × attack_delay，由腳本在最後一跳移除。" },
          { id = "c21e_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "連環鋼破暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Chain Steel Breaker Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "datadriven Stun 動作（引擎內建 modifier_stunned）；受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c21e_old_voice", at = "caster" },
            { op = "damage", to = "target", type = "pure", amount = "$tap_damage" },
            { op = "apply_status", status = "c21e_old_combo", to = "caster" },
            { op = "cue", id = "c21e_old_prison_start", at = "target" },
            { op = "damage", to = "target", type = "physical", amount = "$damage", require_source_alive = false },
            { op = "apply_status", status = "c21e_old_stun", to = "target" },
            { op = "repeat", count = "$hits", interval = "$interval", first = "0", as = "k",
              stop_when = "not has_status(caster, c21e_old_combo)", actions = {
                { op = "if", cond = "not alive(target) or not alive(caster)",
                  actions = { { op = "remove_status", from = "caster", status = "c21e_old_combo" } },
                  otherwise = {
                    { op = "teleport", who = "caster", to = "random_point(target, $jump_offset, $jump_offset)", find_clear_space = true },
                    { op = "face", who = "caster", toward = "target" },
                    { op = "issue_order", who = "caster", order = "attack", target = "target" },
                    { op = "cue", id = "c21e_old_flash", at = "target" },
                    { op = "if", cond = "k == $hits - 1", actions = {
                      { op = "remove_status", from = "caster", status = "c21e_old_combo" },
                    } },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:394-425",
            text = "第一跳在 Timers 0 秒（下一幀），之後每 1 秒一跳，共三跳；第三跳後立刻移除無敵，所以無敵實際約 2 秒，不是 modifier 的 3 秒。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:427-450",
            text = "和本體不同：每跳只是 FindClearSpaceForUnit 到目標周圍 100（全圓隨機角度）並下達攻擊命令（ExecuteOrderFromTable），不是立即 PerformAttack；是否真的打到取決於攻擊間隔，玩家可以改下其他命令。沒有距離上限、不禁用其他技能。" },
          { policy = "as_source", ref = "npc/C_Neutral/C21.txt:576-586",
            text = "傷害與暈眩是 datadriven 的 Damage／Stun（物理、引擎 modifier_stunned），腳本另外先打 1 點純粹傷害觸發受傷事件。" },
          { policy = "note", ref = "npc/C_Neutral/C21.txt:599",
            text = "OVERRIDE_ANIMATION_RATE 3 只影響動畫，屬呈現。" },
        },
        provenance = { "npc/C_Neutral/C21.txt:522-604" },
      },
      -------------------------------------------------------------------------
      { id = "c21_old_two_heavens", source_id = "C21R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          crit_chance = { 20, 30, 40, 50, unit = "pct",
            source = "kv:crit_chance@npc/C_Neutral/C21.txt:661",
            note = "腳本 RandomInt(1,100) <= crit_chance，機率剛好是 crit_chance%（vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:459）" },
          pity_after = { 5, 3, 2, 2, unit = "count",
            source = "script:vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:459 C21Rcount > 100/crit_chance" },
          crit_multiplier = { 130, 160, 190, 220, unit = "pct", source = "kv:crit_bonus@npc/C_Neutral/C21.txt:656" },
        },
        text = {
          zh_TW = { name = "二天一流",
            body = "被動：每次出手有 {crit_chance} 機率暴擊，造成 {crit_multiplier} 傷害且必定命中；連續 {pity_after} 次沒有暴擊時，下一次出手必定暴擊。" },
          en = { name = "Two Heavens as One",
            body = "Passive: each attack has a {crit_chance} chance to critically strike for {crit_multiplier} damage and cannot miss; after {pity_after} attacks in a row without a crit, the next one always crits." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_windup = {
              { op = "chance", p = "$crit_chance", pity_after = "$pity_after", counter = "hero", actions = {
                { op = "modify_attack", crit_multiplier = "$crit_multiplier", cannot_miss = true },
                { op = "cue", id = "c21r_old_crit_blur", at = "event.target" },
                { op = "cue", id = "c21r_old_slam_gesture", at = "self" },
              } },
            },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:452-469",
            text = "原作在 OnAttackStart 先移除、再依機率掛一個持續 GetAttackSpeed() + 0.1 秒的暴擊＋CANNOT_MISS modifier；照意圖寫成只影響這一次攻擊的 modify_attack。攻擊被取消時原作的計數照樣增加。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:457-459",
            text = "保底計數 caster.C21Rcount 每次出手 +1、暴擊時歸零，大於 100 / crit_chance 必暴：一到四級分別在連續 5、3、2、2 次沒暴擊後必中；計數存在英雄身上，升級不歸零。對任何目標（含建築）都擲骰。" },
        },
        provenance = { "npc/C_Neutral/C21.txt:605-664" },
      },
      -------------------------------------------------------------------------
      { id = "c21_old_soaring_quake", source_id = "C21T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 150, 130, 110, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C21.txt:683" },
          mana_cost = { 300, 400, 500, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C21.txt:684" },
          cast_range = { 1300, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C21.txt:685" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C21.txt:681" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6@npc/C_Neutral/C21.txt:673-674" },
          radius = { 800, unit = "dist", source = "kv:aoe_radius@npc/C_Neutral/C21.txt:693" },
          damage = { 1000, 1300, 1600, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C21.txt:682" },
          leap_height = { 450, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:511 mid_pos.z = end_pos.z + 450" },
          leap_time = { 1.1, unit = "s", hidden = true,
            source = "assumed:Physics 上升段 6000×剩餘比例² 到最高點、下降段每幀加速度 +3000，施法距離 1300 時估算約 1.1 秒（vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:513-548）" },
          fallback = { 3, unit = "s", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:585 Timers 3 秒強制落地" },
          apex_ratio = { 0.2, unit = "ratio", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:509-510 begin × 0.8 + end × 0.2" },
          vision_radius = { 3000, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:522 AddFOWViewer 3000" },
          vision_time = { 1.5, unit = "s", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:522 AddFOWViewer 1.5" },
        },
        text = {
          zh_TW = { name = "秘奧義‧飛翔烈震斬",
            body = "躍向目標地點，飛行期間無敵、無法操作；落地時對周圍 {radius} 內的非魔免敵人造成 {damage} 魔法傷害，並摧毀範圍內的樹木。" },
          en = { name = "Hidden Secret Art: Soaring Quake Slash",
            body = "Leaps to the target point, invulnerable and uncontrollable while airborne; on landing deals {damage} magic damage to non-spell-immune enemies within {radius} and destroys trees in the area." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "c21t_old_on_move", duration = "$fallback", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "飛翔烈震斬", body = "飛行中：無敵。" },
              en = { name = "Soaring Quake Slash", body = "Airborne: invulnerable." },
            },
            reveal = "public", states = { "invulnerable" },
            notes = "原作 modifier_C21T_old_on_move：INVULNERABLE、IGNORE_INVULNERABLE、未寫 IsPurgable（暫定 basic）；落地或 3 秒的保險計時器移除。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c21t_old_voice", at = "caster" },
            { op = "cue", id = "c21t_old_press", attach = "caster" },
            { op = "apply_status", status = "c21t_old_on_move", to = "caster" },
            { op = "let", name = "land", value = "point" },
            { op = "reveal", at = "lerp_point(caster, land, $apex_ratio)", radius = "$vision_radius", duration = "$vision_time", teams = "own" },
            { op = "leap", who = "caster", to = "land", duration = "$leap_time", height = "$leap_height" },
            { op = "delay", time = "$leap_time", capture = { "land" }, actions = {
              { op = "teleport", who = "caster", to = "land", find_clear_space = false },
              { op = "remove_status", from = "caster", status = "c21t_old_on_move" },
              { op = "cue", id = "c21t_old_echoslam", at = "land" },
              { op = "for_each",
                select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                actions = {
                  { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                } },
              { op = "destroy_trees", at = "caster", radius = "$radius" },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:489-583",
            text = "原作用 Physics 函式庫：先以遞減速度飛到「起點 80%＋終點 20%」上方 450 的最高點，再以逐幀累加的加速度落向終點，落地時 SetAbsOrigin 到施放點。飛行時間隨距離改變，資料以 leap 固定時間（推定約 1.1 秒）表示。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:585-623",
            text = "3 秒的保險計時器：若還沒落地就強制瞬移到終點並照樣結算傷害；資料的 leap 一定在時間內結束，等同正常落地。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:629-667",
            text = "和本體完全不同的大招：落地 AoE。選取用技能的 TargetTeam／Type、沒有 MAGIC_IMMUNE 旗標，魔免敵人不受傷；ApplyDamage 不檢查施法者存活；以落地後武藏的位置為圓心砍樹。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:496-503",
            text = "飛行期間 PreventDI、無視碰撞與地形；落地用 SetAbsOrigin（不 FindClearSpace），之後靠 AutoUnstuck 推出障礙。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C21_Miyamoto Musashi.lua:522",
            text = "施放時在最高點給自己隊伍 3000 範圍、1.5 秒的視野。" },
        },
        provenance = { "npc/C_Neutral/C21.txt:665-729" },
      },
    },
  }
end
