-- A35 島左近（current 版，heromap_version["16"] = "WERT"，沒有 D）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/A_Oda/A35_2.txt:17-22）。11 版（*_old）不列入。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A35_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"（A21 慣例）。
-- ApplyDamage（不經 AMHC）在攻擊者死亡後仍會造成傷害，寫 require_source_alive = false（C17 慣例）。
return function(ctx)
  -- A35T 每一跳（OnCreated 與 OnIntervalThink 呼叫同一個 A35T_OnIntervalThink）
  local function a35t_pulse()
    return {
      { op = "cue", id = "a35t_blast", at = "self" },
      { op = "for_each",
        select = { from = "self", shape = "circle", radius = "ability.cast_range", team = "enemy",
          types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
        actions = {
          { op = "apply_status", status = "a35t_stun", to = "it" },
          { op = "issue_order", who = "it", order = "stop" },
          { op = "burn_mana", to = "it", amount = "$damage", source = "self" },
          { op = "damage", to = "it", source = "self", type = "magical", amount = "$damage", require_source_alive = false },
        } },
    }
  end
  return {
    schema_version = 3, kind = "hero", id = "A35",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "A35",
      text = { zh_TW = { name = "島左近" }, en = { name = "Shima Sakon" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a35_flame_raid", source_id = "A35W", slot = "Q", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 12, 11, 10, 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 900, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical",
            source = "kv:AbilityDamage", note = "腳本 GetAbilityDamage()；AbilitySpecial damage 100/200/300/400 沒有被使用" },
          path_radius = { 200, unit = "dist", hidden = true,
            source = "kv:RunScript path_radius", note = "npc/A_Oda/A35.txt:67 RunScript 直接傳 200；AbilitySpecial path_radius 100 沒有被使用" },
          sqrt2 = { "1.41421356", unit = "ratio", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A35.lua:47 projectileRadius = pathRadius * math.sqrt(2)" },
          hit_radius = { expr = "$path_radius * $sqrt2", unit = "dist", hidden = true, source = "derived" },
          diameter = { 2, unit = "ratio", hidden = true, source = "script:vscripts/heroes/A_Oda/A35.lua:48 pathRadius*2" },
          sample_spacing = { expr = "$path_radius * $diameter", unit = "dist", hidden = true, source = "derived" },
          extra_samples = { 2, unit = "count", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A35.lua:48 floor(pathLength / (pathRadius*2)) + 2" },
          mark_time = { 0.1, unit = "s", hidden = true,
            source = "design:原作以 it.A35W 旗標在同一次施放內去重，結束前清除；以極短的標記狀態表達" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A35.lua:88 modifier_phased 0.1" },
        },
        text = {
          zh_TW = { name = "焰襲",
            body = "化身怒焰瞬間衝到目標地點，對起點到終點路徑上的非魔免敵人各造成一次 {damage} 魔法傷害。" },
          en = { name = "Flame Raid",
            body = "Becomes a burst of flame and instantly rushes to the target point, dealing {damage} magic damage once to each non-spell-immune enemy along the path." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a35w_hit", duration = "$mark_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", notes = "原作 it.A35W 去重旗標。" },
          { id = "a35w_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, notes = "引擎內建 modifier_phased。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a35w_vo", at = "caster" },
            { op = "let", name = "start", value = "caster.position" },
            { op = "let", name = "fwd", value = "direction(caster, point)" },
            { op = "let", name = "len", value = "distance(caster, point)" },
            { op = "let", name = "n", value = "floor(len / $sample_spacing) + $extra_samples" },
            { op = "cue", id = "a35w_flame_path", at = "start", to = "point" },
            { op = "repeat", count = "n", as = "i", actions = {
              { op = "for_each",
                select = { from = "offset(start, fwd, i * len / (n - 1))", shape = "circle", radius = "$hit_radius",
                  team = "enemy", types = { "hero", "creep", "neutral", "summon" },
                  where = "not has_status(it, a35w_hit)" },
                actions = {
                  { op = "apply_status", status = "a35w_hit", to = "it" },
                  { op = "damage", to = "it", type = "magical", amount = "$damage" },
                } },
            } },
            { op = "teleport", who = "caster", to = "point", find_clear_space = false },
            { op = "apply_status", status = "a35w_phased", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A35.lua:41-86",
            text = "路徑判定是沿起點朝施法者面向、等距取 floor(長度/400)+2 個圓（半徑 200×√2），每個敵人只受一次傷害；中段寬度介於 200 與 283 之間，兩端各多出半徑 283 的圓。傷害在瞬移之前同一刻結算。" },
          { policy = "fix_candidate", ref = "vscripts/heroes/A_Oda/A35.lua:87-88",
            text = "瞬移用 SetAbsOrigin(游標位置)，沒有 FindClearSpaceForUnit，只加 0.1 秒 phased：可能停在樹、懸崖或不可通行處卡住。資料照實際（find_clear_space = false）。" },
          { policy = "no_effect", ref = "npc/A_Oda/A35.txt:51-58",
            text = "KV 的 TrackingProjectile Target TARGET 在點目標技能沒有目標，不產生投射物；OnProjectileHitUnit 也沒有效果。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A35.lua:6-12,52",
            text = "長度用 GetCursorPosition（施法距離內的游標點），取樣方向用 caster:GetForwardVector()（前搖後面向目標點）；資料以 direction(caster, point) 表達。KV duration、speed、duration2、path_delay 只給粒子或沒有使用。" },
          { policy = "as_source", text = "傷害走 AMHC:Damage；選取 FLAG_NONE，魔免排除，隱形與迷霧中的敵人照樣命中。" },
        },
        provenance = { "npc/A_Oda/A35.txt:19-118", "vscripts/heroes/A_Oda/A35.lua:3-90" },
      },
      -------------------------------------------------------------------------
      { id = "a35_sealing_blaze", source_id = "A35E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 24, 22, 20, 18, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange", note = "腳本以 GetCastRange() 當作半徑" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 100, 150, 200, 250, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          stun = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A35.lua:112 modifier_stunned duration 1" },
          seal_duration = { 3, unit = "s", source = "script:vscripts/heroes/A_Oda/A35.lua:111 duration 3（KV Duration 也是 3）" },
        },
        text = {
          zh_TW = { name = "熾焰封魔",
            body = "以自身為中心爆出怒焰，對周圍的非魔免敵人造成 {damage} 魔法傷害並暈眩 {stun} 秒，並在 {seal_duration} 秒內無法施法、無法使用物品、被動技能失效。" },
          en = { name = "Sealing Blaze",
            body = "Erupts in flame around Sakon, dealing {damage} magic damage to nearby non-spell-immune enemies and stunning them for {stun} seconds; for {seal_duration} seconds they cannot cast spells or use items, and their passives are disabled." },
        },
        target = { type = "none", range = "$cast_range", aoe_radius = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a35e_seal", duration = "$seal_duration", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", states = { "break", "mute", "silence" }, cue = "a35e_seal_flame",
            notes = "原作 modifier_A35E：IsHidden、PASSIVES_DISABLED、MUTED、SILENCED；沒寫 IsPurgable（暫定 basic）。" },
          { id = "a35e_stun", text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } }, duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" }, notes = "引擎內建 modifier_stunned。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "a35e_vo", at = "caster" } },
          on_cast = {
            { op = "cue", id = "a35e_blast", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "ability.cast_range", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" } },
              actions = {
                { op = "apply_status", status = "a35e_seal", to = "it" },
                { op = "apply_status", status = "a35e_stun", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A35.lua:98-106",
            text = "半徑是 ability:GetCastRange()，會吃到施法距離加成；選取用 KV 的 TEAM_ENEMY、HERO|BASIC、沒有旗標（魔免排除）。" },
          { policy = "as_source", text = "原作說明沒有提到被動失效（PASSIVES_DISABLED），實際有。" },
        },
        provenance = { "npc/A_Oda/A35.txt:121-184", "vscripts/heroes/A_Oda/A35.lua:93-122" },
      },
      -------------------------------------------------------------------------
      { id = "a35_flame_swiftness", source_id = "A35R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          add_agi = { 8, 16, 24, 32, unit = "stat", source = "kv:add_agi" },
        },
        text = {
          zh_TW = { name = "炎之迅捷", body = "被動：靈活提高 {add_agi} 點。" },
          en = { name = "Flame Swiftness", body = "Passive: +{add_agi} agility." },
        },
        target = { type = "none" },
        passive = { reveal = "public", stats = { agi = "$add_agi" } },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/A_Oda/A35.txt:199-210",
            text = "modifier 名稱沿用 modifier_B04R_passive（STATS_AGILITY_BONUS，綠字）；與 B04 的同名 modifier 各自屬於自己的技能。" },
        },
        provenance = { "npc/A_Oda/A35.txt:186-212" },
      },
      -------------------------------------------------------------------------
      { id = "a35_scorching_fire", source_id = "A35T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 125, 110, 95, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, 190, 230, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange", note = "腳本以 GetCastRange() 當作半徑" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          damage = { 300, 500, 700, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage", note = "腳本 GetAbilityDamage()" },
          duration = { 5, unit = "s", source = "kv:duration" },
          think = { 1.2, unit = "s", source = "kv:think" },
          pulse_stun = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A35.lua:143 modifier_stunned 0.1" },
        },
        text = {
          zh_TW = { name = "焦灼之火",
            body = "啟動後 {duration} 秒內，立即與每 {think} 秒對周圍的敵方英雄與部隊（含魔免）造成 {damage} 魔法傷害、燃燒等量法力，並中斷它們的動作。" },
          en = { name = "Scorching Fire",
            body = "For {duration} seconds, immediately and every {think} seconds, deals {damage} magic damage to nearby enemy heroes and units (spell-immune ones included), burns the same amount of mana, and interrupts their actions." },
        },
        target = { type = "none", range = "$cast_range", aoe_radius = "$cast_range" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "a35t_burning", text = { zh_TW = { name = "焦灼之火", body = "立即與每 {think} 秒對周圍的敵人造成 {damage} 魔法傷害、燃燒等量法力，並中斷它們的動作。" }, en = { name = "Scorching Fire", body = "Immediately and every {think} seconds, deals {damage} magic damage to nearby enemies, burns the same amount of mana, and interrupts their actions." } }, duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public",
            triggers = {
              on_created = a35t_pulse(),
              periodic = { interval = "$think", actions = a35t_pulse() },
            },
            notes = "原作 modifier_A35T：datadriven，沒寫 IsPurgable（暫定 basic）；OnCreated 與 ThinkInterval 1.2 都呼叫 A35T_OnIntervalThink，五秒內共五跳（0、1.2、2.4、3.6、4.8）。" },
          { id = "a35t_stun", text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } }, duration = "$pulse_stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" }, notes = "引擎內建 modifier_stunned。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a35t_cast_sound", at = "caster" },
            { op = "apply_status", status = "a35t_burning", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A35.txt:233-246",
            text = "ApplyModifier 的 Target 區塊只有 Center CASTER 與 Flags，沒有 Radius／Teams／Types；推定只掛在施法者身上（說明與可玩性一致），Dota 實際選取未驗證。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A35.lua:124-155",
            text = "每跳：0.1 秒暈眩（中斷動作）、Stop()、ReduceMana(AbilityDamage)、ApplyDamage 魔法傷害；含魔免旗標，魔免敵人仍被中斷與燒魔，魔法傷害由規則層歸零。半徑是 GetCastRange()，吃施法距離加成。" },
          { policy = "as_source",
            text = "modifier 掛在左近身上，可被驅散（推定 basic）而提前結束；左近死亡時 modifier 移除。" },
          { policy = "note", ref = "npc/A_Oda/A35.txt:283-302",
            text = "KV dmg 300/500/700 與 AbilityDamage 同值，腳本讀 AbilityDamage；radius 500 沒有使用。原作說明寫「附近的敵方英雄」，實際含部隊；KV 沒有 AbilityCastPoint。" },
        },
        provenance = { "npc/A_Oda/A35.txt:219-304", "vscripts/heroes/A_Oda/A35.lua:124-155" },
      },
    },
  }
end
