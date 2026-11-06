-- A31 森蘭丸（2.0 版，原作 *_20 技能組）— package schema v3。每個版本是一位獨立英雄（D22）；舊版另成 mori_ranmaru_old。
-- 槽位依據：選 2.0 版時 choose_20 拔掉全部技能，依 heromap_skill A31 ["20"] = "WERDT" 逐一 AddAbility("A31" .. 字母 .. "_20")，
--   第五個字母前插入 attribute_bonusx；heromap_autoskill A31 ["20"] = "D" 自動學 A31D_20
--   （vscripts/items/Addon_Items/tofar_goback.lua:6-48、vscripts/addon_game_mode.lua:273,376）。
-- W/E/R/T → Q/W/E/R；A31D_20（觀雲，NOT_LEARNABLE）放 D，寫在 hero.auto_learn。
-- KV 在 npc/A_Oda/A31.txt:842-1278；腳本 vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua。原作沒有 *_20 招式的說明文字（各語系 addon_*.txt 都沒有），
--   招式名稱取自 KV 檔的註解，說明依 KV 與腳本還原（assumptions.md）。英雄單位數值沿用本體 A31。
-- magical_resistance、attribute_bonusx、A31_precache 屬規則層或資產，不列入。
-- 慣例照本體：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "A31_v20",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "v20" },
    hero = { source_id = "A31", text = { zh_TW = { name = "森蘭丸（2.0）" }, en = { name = "Mori Ranmaru (2.0)" } },
      auto_learn = { "a31_v20_cloud_watch" } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a31_v20_echoing_rain", source_id = "A31W_20", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 13, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A31.txt:855" },
          mana_cost = { 80, 100, 120, 140, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A31.txt:856" },
          cast_range = { 1200, unit = "dist", source = "kv:AbilityCastRange@npc/A_Oda/A31.txt:858" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/A_Oda/A31.txt:854" },
          radius = { 700, 800, 900, 1000, unit = "dist", source = "kv:radius@npc/A_Oda/A31.txt:896" },
          damage = { 18, 30, 50, 65, unit = "dmg", damage_type = "magical", source = "kv:abilityDamage@npc/A_Oda/A31.txt:906" },
          first_tick = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:119 Timers:CreateTimer(1, ...)" },
          tick_interval = { 1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:137 return 1" },
          ticks = { 11, unit = "count",
            source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:121,136-140 second 從 1 起、second <= 10 時繼續，第 1～11 秒共 11 次" },
          magic_resist = { 35, unit = "pct", source = "kv:modifier_A31W MODIFIER_PROPERTY_MAGICAL_RESISTANCE_BONUS@npc/A_Oda/A31.txt:885" },
          resist_time = { 10, unit = "s", source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:117 ApplyDataDrivenModifier(..., {duration = 10})" },
        },
        text = {
          zh_TW = { name = "響雨",
            body = "對目標地點 {radius} 範圍降下箭雨，施放 {first_tick} 秒後起每秒結算一次，共 {ticks} 次，每次對範圍內非建築的敵人造成 {damage} 魔法傷害。施放後 {resist_time} 秒內森蘭丸魔法抗性提高 {magic_resist}。" },
          en = { name = "Echoing Rain",
            body = "Rains arrows on a {radius} area at the target point; starting {first_tick} seconds after the cast it strikes every second, {ticks} times in total, dealing {damage} magic damage to non-building enemies inside. Ranmaru gains {magic_resist} magic resistance for {resist_time} seconds after casting." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a31w_20_resist", duration = "$resist_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", stats = { magic_resist = "$magic_resist" },
            text = { zh_TW = { name = "響雨", body = "魔法抗性提高 {magic_resist}。" }, en = { name = "Echoing Rain", body = "+{magic_resist} magic resistance." } },
            notes = "原作 modifier_A31W（A31W_20 宣告）：IsHidden 0、IsDebuff 0、MAGICAL_RESISTANCE_BONUS 35；未寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a31w_20_vo", at = "caster" },
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "a31w_20_arrow_rain", at = "at", radius = "$radius", duration = "$resist_time" },
            { op = "apply_status", status = "a31w_20_resist", to = "caster" },
            { op = "repeat", count = "$ticks", interval = "$tick_interval", first = "$first_tick", actions = {
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = { "all" },
                  where = "not it.is_building" },
                actions = {
                  { op = "damage", to = "it", type = "magical", amount = "$damage" },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:119-141",
            text = "傷害 Timer 在 1 秒後開始，每次 second + 1 後結算，second <= 10 時繼續，所以第 1～11 秒共 11 跳；箭雨特效（A31W_2，每 0.1 秒一次共 duration × 10 次）與魔抗增益都是 10 秒，最後一跳落在特效結束後。選取 ALL、旗標 NONE：排除魔免，包含守衛等所有非建築單位。傷害走 AMHC。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:93-141",
            text = "2.0 版沒有本體的減速疊層：A31W_20 只造成傷害，modifier_A31W 改成套在森蘭丸自己身上的魔抗增益。" },
          { policy = "no_effect", ref = "npc/A_Oda/A31.txt:891-935",
            text = "KV duration 10（只用於特效次數）、slow_duration、tick_interval、maximum_charges、charge_replenish_time、dmg 6～24 沒有影響結算。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:144-177",
            text = "A31W_2 每 0.1 秒在範圍內隨機點播放箭雨粒子，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A31.txt:842-936", "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:93-177" },
      },
      -------------------------------------------------------------------------
      { id = "a31_v20_shunpo", source_id = "A31E_20", slot = "W", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 23, 19, 15, 11, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A31.txt:950" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A31.txt:953" },
          cast_point = { 0, unit = "s", source = "kv:AbilityCastPoint@npc/A_Oda/A31.txt:947" },
          max_blink_range = { 450, 550, 650, 750, unit = "dist", source = "kv:max_blink_range@npc/A_Oda/A31.txt:992", note = "RunScript MaxBlinkRange" },
          phase_time = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:24 modifier_phased duration 0.1" },
        },
        text = {
          zh_TW = { name = "瞬步",
            body = "瞬間移動到目標地點，最遠 {max_blink_range}；超過時停在該方向的最遠處。移動前躲開所有飛向自己的投射物。" },
          en = { name = "Shunpo",
            body = "Instantly teleports to the target point, up to {max_blink_range} away; if the point is farther, he stops at the maximum distance in that direction. Dodges all projectiles headed for Ranmaru before moving." },
        },
        target = { type = "point" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { condition = "not has_state(caster, root)" },
        statuses = {
          { id = "a31e_20_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" },
            notes = "引擎內建 modifier_phased。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a31e_20_vo", at = "caster" },
            { op = "dodge_projectiles", of = "caster", from_team = "any", kind = "tracking" },
            { op = "cue", id = "a31e_20_blink_start", at = "caster" },
            { op = "apply_status", status = "a31e_20_phased", to = "caster" },
            { op = "teleport", who = "caster", to = "point", max_distance = "$max_blink_range", find_clear_space = true },
            { op = "cue", id = "a31e_20_blink_end", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:7-29",
            text = "與本體共用 A31E：沒有 AbilityCastRange，任何距離都能施放，目標點截到 max_blink_range；SetAbsOrigin 後 FindClearSpaceForUnit。ROOT_DISABLES：被定身時不能施放。2.0 版只有冷卻與耗魔不同。" },
          { policy = "no_effect", ref = "npc/A_Oda/A31.txt:994-1003",
            text = "KV blink_range_clamp 960、blink_damage_cooldown 3 沒有使用。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:11-15",
            text = "起點建 npc_dummy_unit 只為播粒子與音效，屬呈現，改用 cue。" },
        },
        provenance = { "npc/A_Oda/A31.txt:937-1006", "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:1-29" },
      },
      -------------------------------------------------------------------------
      { id = "a31_v20_storm_arrow", source_id = "A31R_20", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 9, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A31.txt:1021" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A31.txt:1022" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange@npc/A_Oda/A31.txt:1023" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/A_Oda/A31.txt:1020" },
          projectile_speed = { 1800, unit = "speed", hidden = true, source = "kv:TrackingProjectile MoveSpeed@npc/A_Oda/A31.txt:1045" },
          vision_radius = { 225, unit = "dist", hidden = true, source = "kv:TrackingProjectile VisionRadius@npc/A_Oda/A31.txt:1046" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:A31R_Damage@npc/A_Oda/A31.txt:1122" },
          stun = { 0.7, 1.0, 1.3, 1.6, unit = "s", source = "kv:A31R_Duration@npc/A_Oda/A31.txt:1117" },
        },
        text = {
          zh_TW = { name = "嵐矢",
            body = "對一名敵人射出可閃避的嵐矢，命中時造成 {damage} 魔法傷害並暈眩 {stun} 秒。" },
          en = { name = "Storm Arrow",
            body = "Fires a dodgeable storm arrow at an enemy, dealing {damage} magic damage and stunning it for {stun} seconds on impact." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a31r_20_stun", text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } }, duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", states = { "stun" }, cue = "generic_stunned",
            notes = "原作 modifier_A31R（A31R_20 宣告的同名 modifier）：IsPurgable、IsStunDebuff（受狀態抗性縮短）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a31r_20_vo", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = true, vision = "$vision_radius", cue = "a31r_20_arrow",
              on_hit = {
                { op = "cue", id = "a31r_20_impact", at = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "apply_status", status = "a31r_20_stun", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source",
            text = "命中結算是 datadriven ActOnTargets 的 Damage，不經 AMHC：森蘭丸在飛行中死亡仍造成傷害。目標在飛行中變魔免時傷害由規則層歸零，暈眩推定照套（同本體）。" },
          { policy = "note", ref = "npc/A_Oda/A31.txt:1097-1108",
            text = "暈眩 modifier 與本體、舊版同名 modifier_A31R；資料改用唯一的 a31r_20_stun。RequiredLevel 1 與一般技能相同，不另列。" },
        },
        provenance = { "npc/A_Oda/A31.txt:1007-1125" },
      },
      -------------------------------------------------------------------------
      { id = "a31_v20_samidare", source_id = "A31T_20", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6@npc/A_Oda/A31.txt:1142-1143" },
          arrow_count = { 2, 3, 4, unit = "count", source = "kv:arrow_count@npc/A_Oda/A31.txt:1206" },
          split_range = { 1000, unit = "dist", source = "kv:range@npc/A_Oda/A31.txt:1211" },
          projectile_speed = { 1200, unit = "speed", hidden = true, source = "kv:projectile_speed@npc/A_Oda/A31.txt:1216" },
          attack_speed = { 15, 20, 30, unit = "atk_speed", source = "kv:bonus_attack_speed@npc/A_Oda/A31.txt:1221" },
          arrow_damage = { 90, unit = "pct",
            source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:235 GetAverageTrueAttackDamage(target) * 0.9" },
          general_pct = { 50, unit = "pct",
            source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:236-238 EXCLUDE_TARGET_NAME2 傷害 * 0.5" },
        },
        text = {
          zh_TW = { name = "五月雨",
            body = "被動：攻擊速度 +{attack_speed}。普攻出手時，對周圍 {split_range} 內最近的另外最多 {arrow_count} 名可見敵人（含魔免）各射出一支箭，造成自身平均攻擊力 {arrow_damage} 的物理傷害（對武將減為 {general_pct}）。" },
          en = { name = "Samidare",
            body = "Passive: +{attack_speed} attack speed. When attacking, also fires an arrow at each of up to {arrow_count} other nearest visible enemies within {split_range} (spell-immune ones included), dealing {arrow_damage} of Ranmaru's average attack damage as physical damage ({general_pct} against generals)." },
        },
        target = { type = "none" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public",
          stats = { attack_speed = "$attack_speed" },
          triggers = {
            on_attack_launch = { actions = {
              { op = "for_each",
                select = { from = "self", shape = "circle", radius = "$split_range", team = "enemy",
                  types = BASIC, flags = { "include_magic_immune", "visible_only" },
                  exclude = "event.target", where = "not has_status_tag(it, engine_invisible)",
                  order = "nearest", limit = "$arrow_count" },
                actions = {
                  { op = "projectile", kind = "tracking", from = "self", to = "it", speed = "$projectile_speed",
                    dodgeable = true, cue = "a31t_20_split_arrow",
                    on_hit = {
                      { op = "cue", id = "a31t_20_split_hit", at = "it" },
                      { op = "damage", to = "it", type = "physical", require_source_alive = false, flags = { "no_amplify" },
                        amount = "avg_attack_against(self, it) * $arrow_damage * select(has_tag(it, boss), $general_pct, 1)" },
                    } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A31.txt:1177-1197",
            text = "modifier_split_shot_datadriven（Passive、IsHidden）帶 ATTACKSPEED_BONUS_CONSTANT %bonus_attack_speed，OnAttack 呼叫舊版的 A31T_old_split_shot（vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:389-431）：以施法者為圓心 FIND_CLOSEST，跳過目前攻擊目標、看不見的與帶 modifier_invisible 的單位，射滿 arrow_count 支。選取類型寫死 HERO + BASIC，旗標用 KV 的 MAGIC_IMMUNE_ENEMIES | FOW_VISIBLE。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:225-240",
            text = "箭由 OnProjectileHitUnit 的 SplitShotDamage 結算：平均攻擊力 × 0.9 的物理傷害（受護甲）；EXCLUDE_TARGET_NAME2 減半，以 has_tag(it, boss) 表達（同本體）。走 ApplyDamage，森蘭丸死亡後已飛出的箭仍造成傷害。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:433-452",
            text = "modifier_a31t_20（3 秒攻速疊層）只由 A31T_20_OnAttack 套用，但 KV 沒有任何事件呼叫這個函式，是死碼；攻速只來自被動 modifier。" },
        },
        provenance = { "npc/A_Oda/A31.txt:1127-1225", "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:225-240",
          "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:389-452" },
      },
      -------------------------------------------------------------------------
      { id = "a31_v20_cloud_watch", source_id = "A31D_20", slot = "D", kind = "active", max_level = 1, learnable = false,
        tags = { "basic" },
        values = {
          cooldown = { 60, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A31.txt:1236" },
          required_level = { 1, unit = "level", source = "design:D 欄一級自動學會（heromap_autoskill A31 [\"20\"] = \"D\"）" },
          duration = { 10, unit = "s", source = "kv:modifier_A31D_20 Duration@npc/A_Oda/A31.txt:1265" },
          attack_range = { 300, unit = "dist", source = "kv:bonus_attack_range@npc/A_Oda/A31.txt:1275" },
          vision_radius = { 1000, unit = "dist", source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:459 AddFOWViewer(..., 1000, 1, false)" },
          vision_first = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:457 CreateTimer(0.1)" },
          vision_interval = { 0.5, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:460 return 0.5" },
          vision_time = { 1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:459 AddFOWViewer duration 1" },
        },
        text = {
          zh_TW = { name = "觀雲",
            body = "{duration} 秒內攻擊距離 +{attack_range}，並持續看見自身周圍 {vision_radius} 的範圍（不穿透地形）。" },
          en = { name = "Cloud Watch",
            body = "For {duration} seconds Ranmaru gains +{attack_range} attack range and keeps vision of a {vision_radius} area around himself (blocked by terrain)." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a31d_20_watch", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", stats = { attack_range = "$attack_range" },
            text = { zh_TW = { name = "觀雲", body = "攻擊距離 +{attack_range}，看見周圍 {vision_radius} 的範圍。" },
              en = { name = "Cloud Watch", body = "+{attack_range} attack range; reveals {vision_radius} around Ranmaru." } },
            triggers = {
              periodic = { interval = "$vision_interval", actions = {
                { op = "reveal", at = "position(self)", radius = "$vision_radius", duration = "$vision_time", teams = "own" },
              } },
            },
            notes = "原作 modifier_A31D_20：IsHidden 0、ATTACK_RANGE_BONUS %bonus_attack_range、Duration 10；未寫 IsPurgable，暫定 basic。視野由 A31D_20_OnSpellStart 的 Timer 在持有期間每 0.5 秒建立。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "a31d_20_watch", to = "caster" },
            { op = "delay", time = "$vision_first", actions = {
              { op = "if", cond = "has_status(caster, a31d_20_watch)", actions = {
                { op = "reveal", at = "position(caster)", radius = "$vision_radius", duration = "$vision_time", teams = "own" },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:454-462",
            text = "Timer 0.1 秒後開始、每 0.5 秒在森蘭丸當下位置建 1 秒的 AddFOWViewer（不穿透地形），只要身上還有 modifier_A31D_20；被驅散或死亡就停。資料以第一次 0.1 秒的 delay 加狀態的 0.5 秒 periodic 表達，時間點差 0.1 秒以內（design_intent 等級的差異）。" },
          { policy = "as_source", ref = "npc/A_Oda/A31.txt:1227-1250",
            text = "ApplyModifier 的 Target 只有 Center CASTER（沒有 Radius），只套在自己身上。NOT_LEARNABLE、沒寫 MaxLevel 與耗魔；只由 autoskill 設為一級，以 max_level 1 表達。" },
        },
        provenance = { "npc/A_Oda/A31.txt:1226-1278", "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:454-462", "vscripts/addon_game_mode.lua:376" },
      },
    },
  }
end
