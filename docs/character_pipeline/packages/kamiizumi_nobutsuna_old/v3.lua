-- B35_old 上泉信綱（舊版，11 版技能組 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 版本：原作 11 版。choose_11 拔掉全部技能，依 heromap_skill B35 ["11"] = "WERT" 逐一 AddAbility("B35" .. 字母 .. "_old")，
--   heromap_autoskill B35 ["11"] = ""（不自動學；vscripts/addon_game_mode.lua:314,417、vscripts/items/Addon_Items/tofar_goback.lua:96-140）。
-- 槽位：B35W_old/E_old/R_old/T_old → Q/W/E/R，沒有 D。
-- KV 在 npc/B_Unified/B35.txt:387-740，腳本在 vscripts/heroes/B_Unified/B35.lua:215-477。
-- 原作 resource/addon_tchinese.txt 沒有任何 B35*_old 說明，四招說明依 KV 與腳本還原（assumptions.md）；名稱取 KV 註解。
-- 慣例照本體 B35：KV 未寫 IsPurgable 的 datadriven modifier 暫定 dispel = "basic"；datadriven Damage／ApplyDamage 寫 require_source_alive = false。
-- B35_precache、attribute_bonusx、halloween_attack、magical_resistance 屬規則層或資產，不列入。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B35_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B35", text = { zh_TW = { name = "上泉信綱（舊版）" }, en = { name = "Kamiizumi Nobutsuna (Old)" } } },
    unit_state = {
      b35_old_crit_count = { type = "int", init = "0", reveal = "team", scope = "hero",
        note = "原作 caster.C21R：B35R_old 未暴擊的出手計數（vscripts/heroes/B_Unified/B35.lua:296-303），跨死亡保留。" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b35_old_sword_essence", source_id = "B35W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 21, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B35.txt:400" },
          mana_cost = { 150, 180, 210, 240, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B35.txt:401" },
          duration = { 20, unit = "s", source = "kv:B35W_Duration@npc/B_Unified/B35.txt:451" },
          hp_regen = { 10, 20, 30, 40, unit = "per_s", source = "kv:hp@npc/B_Unified/B35.txt:456", note = "MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT" },
          armor = { 2, 4, 6, 8, unit = "armor", source = "kv:armor@npc/B_Unified/B35.txt:461" },
          base_attack_pct = { 25, 35, 45, 55, unit = "pct", source = "kv:atk@npc/B_Unified/B35.txt:466", note = "MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE" },
          mana_regen = { 3, 6, 9, 12, unit = "per_s", source = "kv:mp@npc/B_Unified/B35.txt:471", note = "MODIFIER_PROPERTY_MANA_REGEN_CONSTANT" },
        },
        text = {
          zh_TW = { name = "劍之極意",
            body = "凝聚精神 {duration} 秒：基礎攻擊力提高 {base_attack_pct}、護甲 +{armor}、每秒回復 {hp_regen} 生命與 {mana_regen} 魔力。效果可以重疊。" },
          en = { name = "Essence of the Sword",
            body = "Focuses his spirit for {duration} seconds: +{base_attack_pct} base attack damage, +{armor} armor, {hp_regen} health and {mana_regen} mana regeneration per second. The effect stacks with itself." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b35w_old_focus", duration = "$duration", stacking = "independent", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "劍之極意", body = "基礎攻擊力提高 {base_attack_pct}、護甲 +{armor}、每秒回復 {hp_regen} 生命與 {mana_regen} 魔力；可以重疊。" },
              en = { name = "Essence of the Sword", body = "+{base_attack_pct} base attack damage, +{armor} armor, {hp_regen} health and {mana_regen} mana regeneration per second; stacks." },
            },
            reveal = "public", cue = "b35w_old_aura_fx",
            stats = { base_attack_damage_pct = "$base_attack_pct", armor = "$armor", hp_regen = "$hp_regen", mana_regen = "$mana_regen" },
            notes = "原作 modifier_B35W_old：IsBuff、IsPurgable 1、MULTIPLE。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b35w_old_vo", at = "caster" },
            { op = "apply_status", status = "b35w_old_focus", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B35.txt:441",
            text = "modifier_B35W_old 是 MULTIPLE：冷卻（21 秒）縮減到短於持續時間（20 秒）時，多次施放的效果會疊加。和本體相比，回血較低並多了回魔。" },
          { policy = "note", ref = "npc/B_Unified/B35.txt:417-426",
            text = "ApplyModifier 的 Target 寫成 Center CASTER 加 Flags、沒寫 Radius／Teams；原意是只掛在自己身上（同本體）。施放音效借用 A06W；AbilityCastRange 0 加 CastRangeBuffer 250 對無目標技能沒有作用。" },
        },
        provenance = { "npc/B_Unified/B35.txt:387-474" },
      },
      -------------------------------------------------------------------------
      { id = "b35_old_issen", source_id = "B35E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 12, 11, 10, 8, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B35.txt:490" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B35.txt:491" },
          cast_range = { 1150, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B35.txt:488" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B35.txt:489" },
          speed = { 6000, unit = "speed", hidden = true, source = "kv:B35E_speed@npc/B_Unified/B35.txt:540" },
          distance = { 1150, unit = "dist", source = "kv:B35E_distance@npc/B_Unified/B35.txt:555" },
          radius = { 175, unit = "dist", kv = { 150 },
            source = "script:vscripts/heroes/B_Unified/B35.lua:252-253 fStartRadius／fEndRadius 175",
            source_differs = "KV B35E_radius 150（B35.txt:545）沒有被讀取，劍氣半徑寫死 175" },
          wave_delay = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:264 Timers 0.1" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:B35E_damage@npc/B_Unified/B35.txt:550" },
        },
        text = {
          zh_TW = { name = "新陰流．一閃",
            body = "朝指定方向瞬間突進 {distance}，並沿同一路徑放出一道劍氣（半徑 {radius}）。劍氣穿過的每個非魔免敵方英雄與部隊都會受到信綱一次必定命中的普攻，並受到 {damage} 魔法傷害。" },
          en = { name = "Shinkage-ryu: Issen",
            body = "Dashes {distance} in the target direction in an instant, releasing a sword wave (radius {radius}) along the same path. Every non-spell-immune enemy hero and unit the wave passes through takes a guaranteed basic attack from Nobutsuna and {damage} magic damage." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "let", name = "start", value = "position(caster)" },
            { op = "let", name = "dir", value = "select(distance(caster, point) > 0, direction(caster, point), caster.forward)" },
            { op = "dash", who = "caster", direction = "dir", distance = "$distance", speed = "$speed" },
            { op = "cue", id = "b35e_old_wind_walk", at = "caster" },
            { op = "delay", time = "$wave_delay", capture = { "start", "dir" }, actions = {
              { op = "projectile", kind = "linear", from = "start", direction = "dir", speed = "$speed", distance = "$distance",
                radius = "$radius", pierce = true, team = "enemy", types = BASIC, cue = "b35e_old_sword_wave",
                on_hit = {
                  { op = "if", cond = "not it.is_magic_immune", actions = {
                    { op = "attack", attacker = "caster", target = "it", cannot_miss = true, use_orb = true,
                      trigger_effects = true, ignore_range = true },
                  } },
                  { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:215-268",
            text = "以 modifier_knockback（中心放在自身後方一單位、高度 0、不暈眩）把自己推出 1150，耗時 1150/6000 秒；0.1 秒後才從施放當下的位置射出線性投射物（速度、距離同突進，半徑寫死 175）。施放點與自身重合時改用面向。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:270-276; npc/B_Unified/B35.txt:502-517",
            text = "OnProjectileHitUnit（DeleteOnHit 0，穿透）：目標不是魔免才 PerformAttack（使用法球、觸發特效、略過攻擊冷卻、無視隱形、必定命中），之後 datadriven Damage 魔法 B35E_damage（不檢查施法者存活）。投射物的 iUnitTargetFlags NONE 本來就不會命中魔免單位，魔免檢查是多餘的。" },
          { policy = "no_effect", ref = "npc/B_Unified/B35.txt:518-534; vscripts/heroes/B_Unified/B35.lua:278-289",
            text = "modifier_B35E_old（普攻命中時縮短本技能冷卻 等級 × 0.5 秒）是 Passive 0，沒有任何地方套用，是死碼，不列入。" },
          { policy = "note", ref = "npc/B_Unified/B35.txt:485; vscripts/heroes/B_Unified/B35.lua:260-261",
            text = "AbilityUnitDamageType 寫物理，實際 Damage 是魔法。投射物 bProvidesVision 但 iVisionRadius 是未定義的全域變數（nil），推定沒有視野，不列入。" },
        },
        provenance = { "npc/B_Unified/B35.txt:476-558", "vscripts/heroes/B_Unified/B35.lua:215-289" },
      },
      -------------------------------------------------------------------------
      { id = "b35_old_shinkage_ryu", source_id = "B35R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic", "attack_modifier" },
        values = {
          evasion = { 6, 12, 18, 24, unit = "pct", source = "kv:miss@npc/B_Unified/B35.txt:619（modifier_ninja_cloth EVASION_CONSTANT）" },
          crit_chance = { 9, unit = "pct", source = "kv:crit_chance@npc/B_Unified/B35.txt:624（B35.lua:300 RandomInt(1,100) <= crit_chance）" },
          pity_after = { 9, unit = "count", source = "script:vscripts/heroes/B_Unified/B35.lua:299-300 計數 > 9 時必定觸發" },
          crit_multiplier = { 200, 300, 400, 500, unit = "pct", source = "kv:crit@npc/B_Unified/B35.txt:629（modifier_B35R2 PREATTACK_CRITICALSTRIKE）" },
        },
        text = {
          zh_TW = { name = "新陰流",
            body = "被動：閃避 {evasion} 的攻擊。每次出手有 {crit_chance} 機率暴擊，造成 {crit_multiplier} 傷害且必定命中；連續 {pity_after} 次沒有暴擊，下一次出手必定暴擊。" },
          en = { name = "Shinkage-ryu",
            body = "Passive: evades {evasion} of attacks. Each attack has a {crit_chance} chance to critically strike for {crit_multiplier} damage and cannot miss; after {pity_after} attacks in a row without a critical strike, the next one always crits." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { evasion_pct = "$evasion" },
          triggers = {
            on_attack_windup = {
              { op = "chance", p = "$crit_chance", pity_after = "$pity_after", counter_var = "b35_old_crit_count",
                actions = {
                  { op = "modify_attack", crit_multiplier = "$crit_multiplier", cannot_miss = true },
                } },
            },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B35.lua:291-305",
            text = "原作在 OnAttackStart 先移除舊的 modifier_B35R2，再擲骰並掛持續 GetAttackSpeed()+0.1 秒的暴擊＋必定命中 modifier；能否套到本次攻擊依引擎時序。照設計意圖，暴擊與必中直接記在這一次攻擊（modify_attack）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:296-303",
            text = "計數存在 caster.C21R（與宮本武藏同名欄位，只在同一單位上共用），每次出手先 +1，計數 > 9 或擲中就暴擊並歸零：連續九次沒暴擊後第十次必定暴擊。對任何目標（含建築、友軍）都擲骰。" },
          { policy = "as_source", ref = "npc/B_Unified/B35.txt:588-597",
            text = "閃避是 Passive 的 modifier_ninja_cloth（和忍者衣等道具同名的 modifier），資料寫成本技能的被動閃避；同名 modifier 互相覆蓋的細節不列入。KV 註解「跟柳生一模一樣」。" },
        },
        provenance = { "npc/B_Unified/B35.txt:560-632", "vscripts/heroes/B_Unified/B35.lua:291-305" },
      },
      -------------------------------------------------------------------------
      { id = "b35_old_final_secret_sword", source_id = "B35T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 110, 100, 90, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B35.txt:651" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B35.txt:652" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B35.txt:648" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B35.txt:649" },
          required_level = { 6, 12, 18, unit = "level",
            source = "rules:Dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          damage = { 1000, 1200, 1400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/B_Unified/B35.txt:653（AbilityUnitDamageType 魔法）" },
          play_time = { 2.5, unit = "s", source = "kv:play_time@npc/B_Unified/B35.txt:660" },
          stun_time = { 3.5, 4, 4.5, unit = "s", source = "kv:play_time2@npc/B_Unified/B35.txt:665" },
          recast_delay = { 0.15, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:353 Timers 0.15 重新套暈眩與無敵" },
          dash_lead = { 1.1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:404 play_time - 1.1" },
          dash_start = { expr = "$play_time - $dash_lead", unit = "s", hidden = true, source = "derived" },
          dash_time = { 1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:411-412 knockback_duration 1" },
          dash_height = { 300, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:414 knockback_height 300" },
          dash_offset = { 400, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:347-348 target ± direction * 400" },
          settle_delay = { 1.1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:421 Timers 1.1 FindClearSpaceForUnit" },
          copy_radius = { 300, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:460-467 最後一個分身位置（角度 360°，半徑 300）" },
          vision_radius = { 500, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:366 AddFOWViewer 500" },
          no_heal = { 100, unit = "pct", hidden = true, source = "kv:modifier_B35T_old_playing MODIFIER_PROPERTY_DISABLE_HEALING 1@npc/B_Unified/B35.txt:713" },
          tap_damage = { 1, unit = "dmg", damage_type = "magical", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:471-476 ApplyDamage damage = 1" },
        },
        text = {
          zh_TW = { name = "絕‧陰流最終祕劍轉",
            body = "對一名敵方英雄或部隊（包含魔免）施展祕劍：雙方停止動作，在 {play_time} 秒內都無敵且無法被治療，信綱暈眩 {play_time} 秒、目標暈眩 {stun_time} 秒。信綱在目標周圍連續斬擊，最後從目標前方躍過目標，結束時對目標造成 {damage} 魔法傷害。" },
          en = { name = "Final Secret Sword",
            body = "Unleashes the secret sword on an enemy hero or unit (spell-immune ones included): both stop, and for {play_time} seconds both are invulnerable and cannot be healed; Nobutsuna is stunned for {play_time} seconds and the target for {stun_time} seconds. Nobutsuna slashes around the target, then leaps over it from the front; at the end the target takes {damage} magic damage." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b35t_old_stunned", duration = "$stun_time", stacking = "refresh", dispel = "strong", visibility = "hidden",
            reveal = "public", states = { "stun" },
            notes = "原作 modifier_B35T_old_stunned：IsHidden、IsStunDebuff、Priority ULTRA；沒寫 IsPurgable。暈眩類以 strong 處理（同本體暈眩慣例）。" },
          { id = "b35t_old_playing", duration = "$play_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b35t_old_arena", states = { "invulnerable" },
            stats = { heal_reduction_pct = "$no_heal" },
            text = {
              zh_TW = { name = "最終祕劍轉", body = "無敵，無法被治療。" },
              en = { name = "Final Secret Sword", body = "Invulnerable and cannot be healed." },
            },
            notes = "原作 modifier_B35T_old_playing：MODIFIER_STATE_INVULNERABLE、MODIFIER_PROPERTY_DISABLE_HEALING、Priority ULTRA；沒寫 IsPurgable，暫定 basic。DISABLE_HEALING 以 heal_reduction_pct 100% 表達（產生器不支援 cannot_be_healed）。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "b35t_old_vo", at = "caster" } },
          on_cast = {
            { op = "let", name = "victim", value = "target" },
            { op = "let", name = "dir", value = "select(distance(caster, target) > 0, direction(caster, target), caster.forward)" },
            { op = "let", name = "spos", value = "offset(target, dir, -$dash_offset)" },
            { op = "let", name = "epos", value = "offset(target, dir, $dash_offset)" },
            { op = "issue_order", who = "caster", order = "stop" },
            { op = "issue_order", who = "target", order = "stop" },
            { op = "apply_status", status = "b35t_old_stunned", to = "caster", duration = "$play_time" },
            { op = "apply_status", status = "b35t_old_playing", to = "caster" },
            { op = "apply_status", status = "b35t_old_stunned", to = "target" },
            { op = "apply_status", status = "b35t_old_playing", to = "target" },
            { op = "reveal", at = "position(target)", radius = "$vision_radius", duration = "$play_time", teams = "own" },
            { op = "cue", id = "b35t_old_title", at = "target" },
            { op = "cue", id = "b35t_old_copies", at = "target", radius = "$copy_radius" },
            { op = "teleport", who = "caster", to = "offset(target, vec(1, 0), $copy_radius)", find_clear_space = false },
            { op = "damage", to = "target", type = "magical", amount = "$tap_damage", require_source_alive = false },
            { op = "delay", time = "$recast_delay", capture = { "victim" }, actions = {
              { op = "if", cond = "exists(victim)", actions = {
                { op = "apply_status", status = "b35t_old_stunned", to = "victim" },
                { op = "apply_status", status = "b35t_old_playing", to = "victim" },
              } },
            } },
            { op = "cue", id = "b35t_old_slashes", at = "target", duration = "$dash_start" },
            { op = "delay", time = "$dash_start", capture = { "spos", "epos", "dir" }, actions = {
              { op = "teleport", who = "caster", to = "spos", find_clear_space = false },
              { op = "face", who = "caster", toward = "epos" },
              { op = "leap", who = "caster", to = "epos", duration = "$dash_time", height = "$dash_height" },
              { op = "delay", time = "$settle_delay", actions = {
                { op = "teleport", who = "caster", to = "position(caster)", find_clear_space = true },
              } },
            } },
            { op = "delay", time = "$play_time", capture = { "victim" }, actions = {
              { op = "if", cond = "exists(victim)", actions = {
                { op = "cue", id = "b35t_old_finish", at = "victim" },
                { op = "remove_status", from = "victim", status = "b35t_old_playing" },
                { op = "damage", to = "victim", type = "magical", amount = "$damage", require_source_alive = false },
              } },
              { op = "remove_status", from = "caster", status = "b35t_old_stunned" },
            } },
          },
        },
        coverage = "expressible", confidence = "low",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:349-358",
            text = "雙方先 Stop；信綱與目標都套暈眩（play_time2）與無敵＋禁止治療（play_time）。0.15 秒後對目標重新套一次兩個 modifier（目標暈眩實際 play_time2 + 0.15）。信綱的暈眩在 play_time 結束後的下一個 frame 移除，所以只暈 play_time。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:425-448",
            text = "play_time 時先移除目標的無敵再以 ApplyDamage 造成 AbilityDamage 魔法傷害（不檢查施法者存活）；魔免目標可以被指定，但魔法傷害由規則層歸零。目標暈眩比無敵長，結束後還有 1～2 秒暈眩可以被追擊。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:471-476",
            text = "施放最後的 ApplyDamage 1 點魔法傷害打在已經無敵的目標上，推定沒有效果；照原作寫上（規則層對無敵目標不造成傷害）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:362-424",
            text = "斬擊階段：28 次 Timer 只把信綱移到目標周圍 150 處並播攻擊動作與音效，沒有傷害（呈現，cue）；play_time − 1.1 秒時移到目標前方 400，以 modifier_knockback 在 1 秒內跳到目標後方 400（高度 300），再 1.1 秒後找空地。資料以 teleport + leap 表達最後位置。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B35.lua:307-336,450-470",
            text = "六個 B35T_SE 分身（無敵、恐懼動作、2.6 秒後消失）只是呈現（cue）；建立分身時信綱依序被移到目標周圍 300 的六個點，最後停在角度 360° 那一點，資料以 vec(1, 0) 方向 teleport 表達。" },
          { policy = "note", ref = "npc/B_Unified/B35.txt:634-655",
            text = "KV 註解「柳生的拿過來放」：和本體完全不同（本體是穿刺加暴擊）。KV 沒寫 RequiredLevel，照大招預設 6/12/18。" },
        },
        provenance = { "npc/B_Unified/B35.txt:634-740", "vscripts/heroes/B_Unified/B35.lua:307-477", "npc/npc_units_custom_load.txt:75" },
      },
    },
  }
end
