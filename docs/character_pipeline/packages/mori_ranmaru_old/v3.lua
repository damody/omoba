-- A31 森蘭丸（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:273），
-- 沒有自動學習的技能（heromap_autoskill["11"] = ""，addon_game_mode.lua:376）。W/E/R/T → Q/W/E/R，沒有 D。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 A31*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、A31_precache 屬規則層或資產，不列入。
-- 慣例照本體 A31：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier 寫 "none"；
-- ApplyDamage（不經 AMHC）在攻擊者死亡後仍會造成傷害，寫 require_source_alive = false。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A31_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A31", text = { zh_TW = { name = "森蘭丸（舊版）" }, en = { name = "Mori Ranmaru (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a31_old_thunder_rain", source_id = "A31W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 13, 12, 11, 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1200, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 600, unit = "dist", source = "kv:radius" },
          damage = { 45, 75, 105, 135, unit = "dmg", damage_type = "magical", source = "kv:abilityDamage" },
          tick_interval = { 1, unit = "s",
            source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:364-384 Timer 1 秒後開始、每次 return 1" },
          tick_count = { expr = "$visual_duration / $tick_interval", unit = "count", source = "derived",
            note = "kv:duration（4.0）÷ tick_interval（1.0）= 4",
            source_differs = "腳本 second <= 10 實際打十一次（A31_Mori_Ranmaru.lua:366,379）；KV duration 4 與 AbilityDamage 180/300/420/540（= 四次 × abilityDamage）顯示設計是四次，D22 照設計修" },
          visual_duration = { 4, unit = "s", hidden = true,
            source = "kv:duration", note = "只用在 A31W_2 特效的次數 duration*10，A31_Mori_Ranmaru.lua:354-362" },
        },
        text = {
          zh_TW = { name = "豪雷",
            body = "向目標地點降下箭雨。每 {tick_interval} 秒對範圍 {radius} 內的敵方英雄與一般單位造成 {damage} 魔法傷害，共 {tick_count} 次。" },
          en = { name = "Thunder Rain",
            body = "Rains arrows on the target area. Every {tick_interval} seconds, enemy heroes and basic units within {radius} take {damage} magic damage, {tick_count} times in total." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "a31w_old_vo", at = "caster" },
            { op = "cue", id = "a31w_old_arrow_rain", at = "at", radius = "$radius", duration = "$visual_duration" },
            { op = "cue", id = "a31w_old_cast_gesture", at = "caster" },
            { op = "repeat", interval = "$tick_interval", first = "$tick_interval", count = "$tick_count", actions = {
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon" } },
                actions = {
                  { op = "damage", to = "it", type = "magical", amount = "$damage" },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:364-384",
            text = "FC-172 D22 舊版，比照 D13 修：原作 second 從 0 開始、先加一再判斷 second <= 10，第一次在施放後一秒，共十一次；KV duration 4、AbilityDamage 180/300/420/540（四次 × 傷害）與日文說明「持續 4 秒」都顯示設計是四次。修法：tick_count = duration ÷ tick_interval = 4，施放後第 1～4 秒各一次，與箭雨特效的 4 秒一致。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:367-378",
            text = "FindUnitsInRadius 用技能的 HERO|BASIC、KV 沒寫 AbilityUnitTargetFlags（FLAG_NONE）：魔免排除，隱形與迷霧中的敵人照樣命中；腳本註解寫 MAGIC_IMMUNE_ENEMIES 但 KV 沒有。中心固定在施放時的目標點。" },
          { policy = "as_source",
            text = "傷害走 AMHC:Damage：森蘭丸死亡後計時器繼續，但攻擊者已死不造成傷害（規則層）。舊版沒有緩速，KV slow_duration 沒有使用。" },
          { policy = "no_effect", ref = "npc/A_Oda/A31.txt:503-512",
            text = "OnSpellStart 的 TrackingProjectile 以 TARGET 為目標，但這是點目標技能，沒有目標單位，投射物不會產生；也沒有 OnProjectileHitUnit。" },
          { policy = "note", ref = "npc/A_Oda/A31.txt:522-563",
            text = "KV AbilityDamage、slow_duration、tick_interval、maximum_charges、charge_replenish_time 沒有被腳本使用；duration 只控制特效（每 0.1 秒一次 A31W_2 粒子，keys.section 為 nil 永遠走 SW 分支，屬呈現）。" },
        },
        provenance = { "npc/A_Oda/A31.txt:470-563", "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:342-387",
          "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:144-177", "vscripts/amhc_library/amhc.lua:535-584" },
      },
      -------------------------------------------------------------------------
      { id = "a31_old_shunpo", source_id = "A31E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 20, 16, 12, 8, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 110, 120, 140, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0, unit = "s", source = "kv:AbilityCastPoint" },
          max_blink_range = { 500, 600, 700, 800, unit = "dist", source = "kv:max_blink_range", note = "RunScript MaxBlinkRange" },
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
          { id = "a31e_old_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" },
            notes = "引擎內建 modifier_phased。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a31e_old_vo", at = "caster" },
            { op = "dodge_projectiles", of = "caster", from_team = "any", kind = "tracking" },
            { op = "cue", id = "a31e_old_blink_start", at = "caster" },
            { op = "apply_status", status = "a31e_old_phased", to = "caster" },
            { op = "teleport", who = "caster", to = "point", max_distance = "$max_blink_range", find_clear_space = true },
            { op = "cue", id = "a31e_old_blink_end", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:7-29",
            text = "與本體瞬步共用腳本 A31E，只有 KV 數值不同。沒有 AbilityCastRange，任何距離都能施放，目標點以施法者位置為起點截到 max_blink_range；SetAbsOrigin 後 FindClearSpaceForUnit。" },
          { policy = "as_source", ref = "npc/A_Oda/A31.txt:572",
            text = "ROOT_DISABLES：被定身時不能施放（requires.condition）。DIRECTIONAL、CastFilterRejectCaster 只影響指示器與目標驗證。" },
          { policy = "note", ref = "npc/A_Oda/A31.txt:620-632",
            text = "KV blink_range_clamp 960、blink_damage_cooldown 3 沒有使用（BlinkRangeClamp 也傳 max_blink_range，腳本不讀）。起點的 npc_dummy_unit 只為播粒子與音效，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A31.txt:565-633", "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:1-29" },
      },
      -------------------------------------------------------------------------
      { id = "a31_old_storm_arrow", source_id = "A31R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 18, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          projectile_speed = { 1800, unit = "speed", hidden = true, source = "kv:TrackingProjectile MoveSpeed" },
          vision_radius = { 225, unit = "dist", hidden = true, source = "kv:TrackingProjectile VisionRadius" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:A31R_Damage" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:A31R_Duration" },
        },
        text = {
          zh_TW = { name = "嵐矢",
            body = "對一名敵人射出可閃避的嵐矢，命中時造成 {damage} 魔法傷害並暈眩 {stun} 秒。" },
          en = { name = "Storm Arrow",
            body = "Fires a dodgeable storm arrow at an enemy, dealing {damage} magic damage and stunning it for {stun} seconds on impact." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a31r_old_stun", text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } }, duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", states = { "stun" }, cue = "generic_stunned",
            notes = "原作 modifier_A31R（舊版同名）：IsPurgable、IsStunDebuff（受狀態抗性縮短）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a31r_old_vo", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = true, vision = "$vision_radius", cue = "a31r_old_arrow",
              on_hit = {
                { op = "cue", id = "a31r_old_impact", at = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "apply_status", status = "a31r_old_stun", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source",
            text = "命中結算是 datadriven ActOnTargets 的 Damage，不經 AMHC：森蘭丸在飛行中死亡仍造成傷害。目標在飛行中變魔免時傷害由規則層歸零，暈眩推定照套（同本體）。" },
          { policy = "note", ref = "npc/A_Oda/A31.txt:731-747",
            text = "舊版的暈眩 modifier 與本體同名 modifier_A31R；兩版不會同時存在於同一位英雄身上，資料改用唯一的 a31r_old_stun。" },
        },
        provenance = { "npc/A_Oda/A31.txt:635-753" },
      },
      -------------------------------------------------------------------------
      { id = "a31_old_samidare", source_id = "A31T_old", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          arrow_count = { 3, 5, 7, unit = "count", source = "kv:arrow_count" },
          split_range = { 1000, unit = "dist", source = "kv:range" },
          projectile_speed = { 1200, unit = "speed", hidden = true, source = "kv:projectile_speed" },
          arrow_damage = { 90, unit = "pct",
            source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:233 GetAverageTrueAttackDamage(target) * 0.9" },
          general_pct = { 50, unit = "pct",
            source = "script:vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:234-236 EXCLUDE_TARGET_NAME2 傷害 * 0.5" },
        },
        text = {
          zh_TW = { name = "五月雨",
            body = "被動：普攻出手時，對周圍 {split_range} 內最近的另外最多 {arrow_count} 名可見敵人（含魔免）各射出一支箭，造成自身平均攻擊力 {arrow_damage} 的物理傷害（對武將減為 {general_pct}）。" },
          en = { name = "Samidare",
            body = "Passive: when attacking, also fires an arrow at each of up to {arrow_count} other nearest visible enemies within {split_range} (spell-immune ones included), dealing {arrow_damage} of Ranmaru's average attack damage as physical damage ({general_pct} against generals)." },
        },
        target = { type = "none" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_launch = { actions = {
              { op = "for_each",
                select = { from = "self", shape = "circle", radius = "$split_range", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune", "visible_only" },
                  exclude = "event.target", where = "not has_status_tag(it, engine_invisible)",
                  order = "nearest", limit = "$arrow_count" },
                actions = {
                  { op = "projectile", kind = "tracking", from = "self", to = "it", speed = "$projectile_speed",
                    dodgeable = true, cue = "a31t_old_split_arrow",
                    on_hit = {
                      { op = "cue", id = "a31t_old_split_hit", at = "it" },
                      { op = "damage", to = "it", type = "physical", require_source_alive = false, flags = { "no_amplify" },
                        amount = "avg_attack_against(self, it) * $arrow_damage * select(has_tag(it, boss), $general_pct, 1)" },
                    } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:389-431",
            text = "modifier_split_shot_datadriven（Passive、IsHidden）OnAttack 呼叫 A31T_old_split_shot：以施法者為圓心 FIND_CLOSEST，跳過目前的攻擊目標、看不見的與帶 modifier_invisible 的單位，射滿 arrow_count 支為止。選取類型寫死 HERO + BASIC（KV 的 BUILDING 不用），旗標用 KV 的 MAGIC_IMMUNE_ENEMIES | FOW_VISIBLE。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:225-240",
            text = "箭由 OnProjectileHitUnit 的 SplitShotDamage 結算：平均攻擊力 × 0.9 的物理傷害（受護甲）；EXCLUDE_TARGET_NAME2（兩種武將）減半，資料以 has_tag(it, boss) 表達（同本體）。走 ApplyDamage，森蘭丸死亡後已飛出的箭仍造成傷害。" },
          { policy = "note", ref = "npc/A_Oda/A31.txt:779-797",
            text = "OnUpgrade 的 A31T_Levelup 已註解成空函式；舊版大招純被動，沒有本體的引導啟動。" },
        },
        provenance = { "npc/A_Oda/A31.txt:754-841", "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:225-246",
          "vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:389-431" },
      },
    },
  }
end
