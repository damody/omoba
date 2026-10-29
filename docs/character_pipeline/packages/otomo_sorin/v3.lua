-- C12 大友宗麟（current 版，heromap_skill C12 ["16"] = "WERDT"，沒有 11 版）— package schema v3.1。
-- 原作槽位 W/E/R/D/T → Q/W/E/D/R（npc/C_Neutral/C12_2.txt:14-19）。
-- 開場自動學 C12D（heromap_autoskill C12 ["16"] = "D"，vscripts/addon_game_mode.lua:431），寫在 hero.auto_learn。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，C12_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"（v3.md §12.3 推定）。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
-- 護盾剩餘量是原作的 unit.AphoticShieldRemaining，依 v3.1 §12.10 用跨文件共用的 unit_state
-- `aphotic_shield_remaining`（scope = unit、fixed）；其他讀寫同名欄位的文件（A04、A24、B08、B36、C06、kappa 道具）應改用同一個名稱。
local BASIC = { "hero", "creep", "neutral", "summon" }

return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "C12",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "C12",
      text = { zh_TW = { name = "大友宗麟" }, en = { name = "Otomo Sorin" } },
      auto_learn = { "c12_homing_lock" },
    },
    unit_state = {
      c12d_target = { type = "unit", reveal = "public", scope = "hero",
        note = "原作 caster.C12D_target：最近一次定位導彈標記的目標" },
      aphotic_shield_remaining = { type = "fixed", init = "0", reveal = "public", scope = "unit",
        note = "原作 unit.AphoticShieldRemaining：與其他英雄、道具的護盾共用同一個單位欄位（v3.1 跨文件同名共用）" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c12_homing_lock", source_id = "C12D", slot = "D", kind = "active", max_level = 1, learnable = false,
        values = {
          cooldown = { 5, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 35, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          break_range = { 1000, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C12.lua:16 len > 1000" },
        },
        text = {
          zh_TW = { name = "定位導彈",
            body = "標記一名敵方英雄（包含魔免）：標記期間宗麟的隊伍能看見該敵人，{@c12_missile_launch}改為只朝它發射一枚追蹤導彈。被標記者移動時若與宗麟相距超過 {break_range}，標記消失；重新標記會把連續命中次數歸零。" },
          en = { name = "Homing Lock",
            body = "Marks an enemy hero (spell-immune ones included): while marked, Sorin's team can see it and {@c12_missile_launch} instead fires a single homing missile at it. If the marked enemy moves while more than {break_range} away from Sorin, the mark ends; re-marking resets the consecutive hit count." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero" } },
        flags = { "pierce_magic_immune" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c12d_mark", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "定位導彈", body = "被宗麟的隊伍看見，{@c12_missile_launch}只朝這個目標發射；與宗麟相距超過 {break_range} 時消失。" },
              en = { name = "Homing Lock", body = "Visible to Sorin's team and the sole target of {@c12_missile_launch}; ends if more than {break_range} away from Sorin." },
            },
            reveal = "public", states = { "provides_vision" }, cue = "c12d_portrait", remove_on = { "death" },
            triggers = {
              on_moved = {
                filter = "distance(self, caster) > $break_range",
                actions = {
                  { op = "remove_status", from = "self", status = "c12d_count" },
                  { op = "remove_status", from = "self", status = "c12d_mark" },
                } },
            },
            notes = "原作 modifier_C12D_debuff：沒有 Duration（C12D_duration 60 沒被讀），PROVIDES_VISION，OnUnitMoved 檢查距離；沒寫 IsPurgable，暫定 basic。" },
          { id = "c12d_count", duration = "infinite", stacking = "replace", dispel = "none", visibility = "hidden",
            reveal = "public",
            vars = { n = { type = "int", init = "0", reveal = "public" } },
            notes = "原作 target.C12D_count（單位上的 Lua 欄位）：標記時設 0，鎖定發射時 +1；只有因距離解除時清掉，死亡、驅散都不會清。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "c12d_mark", to = "target" },
            { op = "apply_status", status = "c12d_count", to = "target", vars = { n = "0" } },
            { op = "set_state", var = "c12d_target", value = "target" },
            { op = "cue", id = "c12d_matrix_grid", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C12.lua:10-22",
            text = "距離只在被標記者移動（OnUnitMoved）時檢查：宗麟自己走遠、目標站著不動，標記不會消失。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C12.lua:1-8",
            text = "只記一個目標（caster.C12D_target），改標別人時舊目標身上的標記與計數仍留著（照樣提供視野），只是導彈不再鎖定它。" },
          { policy = "fix_candidate", ref = "vscripts/heroes/C_Neutral/C12.lua:7,31-35",
            text = "C12D_count 是單位上的 Lua 欄位，死亡或被驅散時標記消失但計數留著；之後一般發射的導彈打到這個單位，仍會吃到智慧加成傷害，計數為偶數時還會暈眩，等於沒有標記也觸發鎖定效果。" },
          { policy = "note", ref = "npc/C_Neutral/C12.txt:78-85", text = "C12D_duration 60 沒有被使用，標記沒有時間上限。" },
        },
        provenance = { "npc/C_Neutral/C12.txt:22-86", "vscripts/heroes/C_Neutral/C12.lua:1-22", "vscripts/addon_game_mode.lua:431" },
      },
      -------------------------------------------------------------------------
      { id = "c12_missile_launch", source_id = "C12W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 3, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 35, 50, 65, 70, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 700, unit = "dist", source = "kv:AbilityCastRange", note = "腳本以 ability:GetCastRange() 當搜尋半徑" },
          damage = { 120, 180, 240, 300, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          int_rate = { 1.2, 1.7, 2.2, 2.7, unit = "ratio", source = "kv:intRate" },
          stun = { 1.5, unit = "s", source = "script:vscripts/heroes/C_Neutral/C12.lua:32 modifier_stunned duration 1.5" },
          parity = { 2, unit = "count", hidden = true, source = "script:vscripts/heroes/C_Neutral/C12.lua:31 math.mod(count, 2) == 0" },
          missile_speed = { 1000, unit = "speed", hidden = true, source = "script:vscripts/heroes/C_Neutral/C12.lua:54,90 iMoveSpeed 1000" },
        },
        text = {
          zh_TW = { name = "導彈發射！",
            body = "朝周圍 {radius} 內看得見的每個敵方英雄與部隊各發射一枚追蹤導彈，造成 {damage} 魔法傷害。若有被{@c12_homing_lock}標記的目標，改為只朝它發射一枚（不限距離）：額外造成智慧 × {int_rate} 的魔法傷害，每第二次命中另暈眩 {stun} 秒。" },
          en = { name = "Missile Launch!",
            body = "Fires a homing missile at every visible enemy hero and unit within {radius}, each dealing {damage} magic damage. If a target is marked by {@c12_homing_lock}, fires a single missile at it instead (any distance): it deals an extra Intelligence × {int_rate} magic damage, and every second hit also stuns for {stun} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c12w_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "導彈暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Missile Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" }, notes = "原作 ApplyDataDrivenModifier 引擎內建 modifier_stunned。" },
        },
        events = {
          on_cast = {
            { op = "if", cond = "exists(state.c12d_target) and has_status(state.c12d_target, c12d_mark)",
              actions = {
                { op = "set_var", on = "state.c12d_target", status = "c12d_count", var = "n",
                  value = "status_var(state.c12d_target, c12d_count, n) + 1" },
                { op = "projectile", kind = "tracking", from = "caster", to = "state.c12d_target", speed = "$missile_speed",
                  cue = "c12w_missile", on_hit = { { op = "call", routine = "c12w_hit", args = { victim = "it" } } } },
              },
              otherwise = {
                { op = "for_each",
                  select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                    flags = { "visible_only" } },
                  actions = {
                    { op = "projectile", kind = "tracking", from = "caster", to = "it", speed = "$missile_speed",
                      cue = "c12w_missile", on_hit = { { op = "call", routine = "c12w_hit", args = { victim = "it" } } } },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C12.lua:43-58",
            text = "鎖定分支不檢查距離：只要標記還在，全圖都會發射；計數在發射時 +1（不是命中時），所以第一發計數 1 不暈，第二發 2 才暈。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C12.lua:24-38",
            text = "命中時才讀計數與智慧；計數存在（不論有沒有標記）就加智慧傷害，偶數再暈眩。傷害走 AMHC，施法者已死不造成傷害，但暈眩照樣生效。" },
          { policy = "as_source", ref = "npc/C_Neutral/C12.txt:97",
            text = "一般發射用 FOW_VISIBLE、沒有 MAGIC_IMMUNE 旗標：只打看得見、非魔免的敵人；導彈是可閃避的追蹤投射物。" },
          { policy = "note", text = "原作說明寫「連續被擊中 2 次將會暈眩」；實際是計數為偶數的那一發。" },
        },
        provenance = { "npc/C_Neutral/C12.txt:88-135", "vscripts/heroes/C_Neutral/C12.lua:24-97" },
      },
      -------------------------------------------------------------------------
      { id = "c12_space_rift", source_id = "C12E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 11, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          add_int = { 5, 10, 15, 20, unit = "stat", source = "kv:add_int" },
          blink_range = { 600, unit = "dist", source = "kv:blink_range" },
          hit_radius = { 250, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C12.lua:121-129 FindUnitsInRadius 250" },
          damage = { 120, 180, 240, 300, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          shield_duration = { 10, unit = "s", source = "kv:modifier_C12E Duration" },
          shield_int_mult = { 5, unit = "ratio", source = "script:vscripts/heroes/C_Neutral/C12.lua:144 GetIntellect()*5" },
        },
        text = {
          zh_TW = { name = "空間撕裂裝置",
            body = "被動：智力 +{add_int}。啟動：朝面向瞬移 {blink_range}，對落點 {hit_radius} 內看得見的敵方英雄與部隊造成 {damage} 魔法傷害；若有打到敵人，獲得 {shield_duration} 秒的脈衝護盾，在受到傷害後補回生命，總量為智力 × {shield_int_mult}。" },
          en = { name = "Space Rift Device",
            body = "Passive: +{add_int} Intelligence. Active: blinks {blink_range} in the facing direction and deals {damage} magic damage to visible enemy heroes and units within {hit_radius} of the landing spot; if any enemy is hit, gains a pulse shield for {shield_duration} seconds that restores health after damage is taken, up to Intelligence × {shield_int_mult} in total." },
        },
        target = { type = "none" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        passive = { reveal = "public", visibility = "buff", stats = { int = "$add_int" } },
        unit_state = {
          c12e_hit = { type = "bool", init = "false", reveal = "public", scope = "ability", note = "原作 #units > 0" },
        },
        statuses = {
          { id = "c12e_shield", duration = "$shield_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "脈衝護盾", body = "受到傷害後補回生命，總量為智力 × {shield_int_mult}，用完即消失。" },
              en = { name = "Pulse Shield", body = "Restores health lost to damage, up to Intelligence × {shield_int_mult} in total; ends when used up." },
            },
            reveal = "public", cue = "c12e_pulse_shield",
            triggers = {
              -- 原作在傷害結算後補血（SetHealth），不是吸收
              on_take_damage = {
                { op = "set_health", to = "self", value = "self.hp + min(event.damage.final, state.aphotic_shield_remaining)" },
                { op = "set_state", on = "self", var = "aphotic_shield_remaining",
                  value = "state.aphotic_shield_remaining - event.damage.final" },
                { op = "if", cond = "state.aphotic_shield_remaining <= 0", actions = {
                  { op = "clear_state", on = "self", var = "aphotic_shield_remaining" },
                  { op = "remove_status", from = "self", status = "c12e_shield" },
                } },
              },
            },
            notes = "原作 modifier_C12E：Duration 10，OnTakeDamage 呼叫 C12E_damage_check；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "dodge_projectiles", of = "caster", kind = "tracking" },
            { op = "cue", id = "c12e_blink_start", at = "caster" },
            { op = "teleport", who = "caster", to = "offset(caster, caster.forward, $blink_range)", find_clear_space = false },
            { op = "cue", id = "c12e_blink_end", at = "caster" },
            { op = "set_state", var = "c12e_hit", value = "false" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$hit_radius", team = "enemy", types = BASIC,
                flags = { "visible_only" } },
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "set_state", var = "c12e_hit", value = "true" },
              } },
            { op = "if", cond = "state.c12e_hit", actions = {
              { op = "apply_status", status = "c12e_shield", to = "caster" },
              { op = "set_state", on = "caster", var = "aphotic_shield_remaining", value = "caster.int * $shield_int_mult" },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "vscripts/heroes/C_Neutral/C12.lua:271-289",
            text = "護盾是在 OnTakeDamage（傷害已扣血）後 SetHealth 補回，擋不住致死傷害。資料照原作寫 on_take_damage + set_health；若修正，改用 absorb。" },
          { policy = "fix_candidate", ref = "vscripts/heroes/C_Neutral/C12.lua:148",
            text = "剩餘量存在 unit.AphoticShieldRemaining，與 A04、A24、B08、B36、C06、kappa 道具共用同名欄位，同時存在會互相覆寫、重複補血。v3.1 以跨文件共用的 unit_state 照實表達。" },
          { policy = "no_effect", ref = "vscripts/heroes/C_Neutral/C12.lua:296-299",
            text = "護盾耗盡時套 modifier_C12E_Boom，C12E 沒有定義這個 modifier，沒有效果（原意是爆炸）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C12.lua:110-118",
            text = "無目標技能：一律朝面向瞬移剛好 600，以 SetAbsOrigin 移動、不找空位（只給 0.1 秒 phased），可能落在樹林或懸崖裡。施放時 ProjectileDodge。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C12.lua:121-143",
            text = "傷害用 AbilityDamage 120/180/240/300，KV dmg 100/200/300/400 沒有被讀（原作說明寫 dmg）；護盾量在施放當下讀智力。只有打到至少一個敵人才給護盾。" },
          { policy = "as_source", text = "OnTakeDamage 不分傷害類型與來源；DamageTaken 是減免後的實際傷害（event.damage.final）；SetHealth 不吃治療增減。" },
        },
        provenance = { "npc/C_Neutral/C12.txt:137-242", "vscripts/heroes/C_Neutral/C12.lua:100-166", "vscripts/heroes/C_Neutral/C12.lua:271-301" },
      },
      -------------------------------------------------------------------------
      { id = "c12_optical_mask", source_id = "C12R", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 23, 20, 17, 14, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          heal = { 0, 150, 300, 450, unit = "heal", source = "kv:C12R_heal" },
          invis_time = { 3, unit = "s", source = "script:vscripts/heroes/C_Neutral/C12.lua:174,176 寫死 3", note = "與 KV C12R_duration 同值，但腳本沒有讀它" },
          full_range = { 400, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C12.lua:173 len < 400" },
          decay_step = { 100, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C12.lua:176 (len-400)/100" },
          decay_time = { 0.2, unit = "s", source = "script:vscripts/heroes/C_Neutral/C12.lua:176 *0.2" },
          min_time = { 0.8, unit = "s", source = "script:vscripts/heroes/C_Neutral/C12.lua:177-178" },
        },
        text = {
          zh_TW = { name = "光學遮罩",
            body = "使自身或任一友方英雄或部隊隱形並回復 {heal} 生命。距離 {full_range} 內隱形 {invis_time} 秒；超過時每多 {decay_step} 少 {decay_time} 秒，最少 {min_time} 秒。" },
          en = { name = "Optical Mask",
            body = "Turns Sorin or any allied hero or unit invisible and heals it for {heal}. Within {full_range} the invisibility lasts {invis_time} seconds; beyond that it is {decay_time} seconds shorter per {decay_step}, down to {min_time} seconds." },
        },
        target = { type = "unit", team = "ally", types = BASIC },
        flags = { "self_cast_allowed" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c12r_invis", duration = "$invis_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "光學遮罩", body = "隱形，敵方看不見。" },
              en = { name = "Optical Mask", body = "Invisible to enemies." },
            },
            reveal = "public", tags = { "invisible", "engine_invisible" }, states = { "invisible" }, cue = "c12r_mask",
            notes = "原作 ApplyDataDrivenModifier 引擎內建 modifier_invisible；可否驅散依引擎（暫定 basic）。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "c12r_invis", to = "target",
              duration = "max($min_time, $invis_time - max(distance(caster, target) - $full_range, 0) / $decay_step * $decay_time)" },
            { op = "heal", to = "target", source = "caster", amount = "$heal" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "npc/C_Neutral/C12.txt:244",
            text = "原作說明寫「攻擊或施放技能會使隱形提前結束」，KV 註解寫「普攻 放技能 解除不了隱形…沒弄」：引擎 modifier_invisible 不因攻擊或施法解除。資料照原作不解除；若修正，加 remove_on = { attack, cast }。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C12.lua:172-181",
            text = "距離用 3D 的 Length()（含高度差）；v3 用平面距離，差異只在高低地。時間是連續線性遞減，不是每 100 一階。" },
          { policy = "as_source", ref = "npc/C_Neutral/C12.txt:245-291",
            text = "KV 沒寫 AbilityCastRange，推定不限距離（原作說明寫全戰場）；一級的回復是 0。先隱形再回復。" },
          { policy = "note", ref = "npc/C_Neutral/C12.txt:253", text = "AbilityUnitTargetFlags MAGIC_IMMUNE_ENEMIES 對友方目標沒有作用。" },
        },
        provenance = { "npc/C_Neutral/C12.txt:244-292", "vscripts/heroes/C_Neutral/C12.lua:168-182" },
      },
      -------------------------------------------------------------------------
      { id = "c12_annihilation_beam", source_id = "C12T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 100, 85, 70, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 300, 400, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          damage = { 100, 160, 220, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          beam_speed = { 2000, unit = "speed", hidden = true, source = "script:vscripts/heroes/C_Neutral/C12.lua:208 vec * 2000" },
          beam_distance = { 2000, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C12.lua:209 fDistance 2000" },
          beam_radius = { 100, unit = "dist", hidden = true, source = "script:vscripts/heroes/C_Neutral/C12.lua:210-211" },
          wide_angle = { 20, unit = "deg", hidden = true, source = "script:vscripts/heroes/C_Neutral/C12.lua:192-195 ±20°" },
          narrow_angle = { 10, unit = "deg", hidden = true, source = "script:vscripts/heroes/C_Neutral/C12.lua:196-199 ±10°" },
          fan_interval = { 0.2, unit = "s", source = "script:vscripts/heroes/C_Neutral/C12.lua:254 return 0.2" },
          fan_volleys = { 6, unit = "count",
            source = "script:vscripts/heroes/C_Neutral/C12.lua:225-256 count 0→5，第六次檢查 count < 5 不成立才停：共六輪" },
          line_delay = { 1, unit = "s", source = "script:vscripts/heroes/C_Neutral/C12.lua:257 CreateTimer(1)" },
          line_interval = { 0.33, unit = "s", source = "script:vscripts/heroes/C_Neutral/C12.lua:266 return 0.33" },
          line_volleys = { 25, unit = "count",
            source = "script:vscripts/heroes/C_Neutral/C12.lua:257-268 與第一段共用 count，從 5 數到 29：共 25 輪" },
        },
        text = {
          zh_TW = { name = "殲敵光線",
            body = "兩段式光線，方向跟著宗麟當下的面向。第一段每 {fan_interval} 秒朝前方扇形射出四道光線，共 {fan_volleys} 輪；施放 {line_delay} 秒後第二段開始，每 {line_interval} 秒朝正前方射出兩道重疊的光線，共 {line_volleys} 輪。每道光線長 {beam_distance}，貫穿路徑上非魔免的敵方英雄與部隊，各造成 {damage} 魔法傷害。宗麟死亡後停止發射。" },
          en = { name = "Annihilation Beam",
            body = "A two-stage barrage that follows Sorin's current facing. Stage one fires four beams in a fan every {fan_interval} seconds, {fan_volleys} volleys in total; {line_delay} seconds after the cast stage two fires two overlapping beams straight ahead every {line_interval} seconds, {line_volleys} volleys in total. Each beam is {beam_distance} long and passes through non-spell-immune enemy heroes and units, dealing {damage} magic damage to each. Firing stops when Sorin dies." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        events = {
          on_cast = {
            { op = "cue", id = "c12t_vo", at = "caster" },
            { op = "cue", id = "c12t_charge", attach = "caster" },
            { op = "repeat", count = "$fan_volleys", interval = "$fan_interval", first = "0", ["while"] = "alive(caster)",
              actions = {
                { op = "call", routine = "c12t_beam", args = { dir = "rotate(caster.forward, $wide_angle)" } },
                { op = "call", routine = "c12t_beam", args = { dir = "rotate(caster.forward, -$wide_angle)" } },
                { op = "call", routine = "c12t_beam", args = { dir = "rotate(caster.forward, $narrow_angle)" } },
                { op = "call", routine = "c12t_beam", args = { dir = "rotate(caster.forward, -$narrow_angle)" } },
              } },
            { op = "delay", time = "$line_delay", actions = {
              { op = "repeat", count = "$line_volleys", interval = "$line_interval", first = "0", ["while"] = "alive(caster)",
                actions = {
                  { op = "call", routine = "c12t_beam", args = { dir = "caster.forward" } },
                  { op = "call", routine = "c12t_beam", args = { dir = "caster.forward" } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C12.lua:259-262",
            text = "第二段每輪建立兩道完全相同的光線，各自判定命中：正前方的敵人每輪吃兩次傷害。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C12.lua:223-268",
            text = "兩個 Timer 共用 count：第一段 0～1.0 秒共六輪（24 道），第二段 1～8.92 秒共 25 輪（50 道）。原作說明寫每 0.22 秒五次、第二段 24 次。" },
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C12.lua:252,264",
            text = "原作先發射再檢查 IsAlive，宗麟死亡後還會多發一輪（從屍體位置）；v3 以每輪發射前 alive(caster) 判斷。datadriven Damage 不檢查施法者存活，已發出的光線照樣傷害。" },
          { policy = "as_source", ref = "npc/C_Neutral/C12.txt:300",
            text = "KV Behavior 同時有 NO_TARGET 與 POINT／DIRECTIONAL，腳本只看面向（GetForwardVector），施放視為無目標。宗麟可以邊轉身邊掃射。" },
          { policy = "note", ref = "npc/C_Neutral/C12.txt:386-407",
            text = "dragon_slave_* 四個 AbilitySpecial 沒有被使用；iVisionRadius 讀未定義的全域變數，光線不提供視野。modifier_B08T（0.5 秒特效）屬呈現。" },
        },
        provenance = { "npc/C_Neutral/C12.txt:292-412", "vscripts/heroes/C_Neutral/C12.lua:185-269" },
      },
    },
    routines = {
      c12w_hit = {
        owner = "c12_missile_launch",
        params = { victim = "unit" },
        actions = {
          { op = "damage", to = "victim", type = "magical", amount = "$damage" },
          { op = "if", cond = "has_status(victim, c12d_count)", actions = {
            { op = "if", cond = "status_var(victim, c12d_count, n) % $parity == 0", actions = {
              { op = "apply_status", status = "c12w_stun", to = "victim" },
            } },
            { op = "damage", to = "victim", type = "magical", amount = "caster.int * $int_rate" },
          } },
          { op = "cue", id = "c12w_impact", at = "victim" },
        },
      },
      c12t_beam = {
        owner = "c12_annihilation_beam",
        params = { dir = "vec" },
        actions = {
          { op = "projectile", kind = "linear", from = "caster", direction = "dir", speed = "$beam_speed",
            distance = "$beam_distance", radius = "$beam_radius", pierce = true, team = "enemy", types = BASIC,
            cue = "c12t_beam",
            on_hit = { { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false } } },
        },
      },
    },
  }
end
