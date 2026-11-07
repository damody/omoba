-- C10 香宗我部親泰（舊版，11 版 *_old，KV 註解「11.2B」）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C10"]["11"] = "WERDT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:326）。
-- KV 沒有 C10D_old（npc/C_Neutral/C10.txt 只有 C10W_old／C10E_old／C10R_old／C10T_old），AddAbility("C10D_old") 失敗，舊版實際只有四招；
-- 自動學習 heromap_autoskill["C10"]["11"] = ""（addon_game_mode.lua:429）。W/E/R/T → Q/W/E/R。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 C10*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C10_precache 不列入。
-- 慣例照本體 C10：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；datadriven Stun（modifier_stunned）寫 "strong"；
-- ApplyDamage／datadriven Damage（不經 AMHC）在親泰死亡後仍會造成傷害，寫 require_source_alive = false；HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C10_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C10", text = { zh_TW = { name = "香宗我部親泰（舊版）" }, en = { name = "Kosokabe Chikayasu (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c10_old_black_dragon_wave", source_id = "C10W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 9, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C10.txt:552" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C10.txt:553" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C10.txt:556" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C10.txt:555" },
          damage = { 250, 350, 450, 550, unit = "dmg", damage_type = "magical",
            source = "kv:AbilityDamage@npc/C_Neutral/C10.txt:554（C10.lua:242 GetAbilityDamage）" },
          jump_radius = { 800, unit = "dist", source = "kv:radius@npc/C_Neutral/C10.txt:624" },
          jump_count = { 20, unit = "count", source = "kv:jump_count@npc/C_Neutral/C10.txt:629",
            note = "包含第一個目標：第一擊後減一，大於零才跳，共命中 20 個單位" },
          jump_delay = { 0.2, unit = "s", source = "kv:jump_delay@npc/C_Neutral/C10.txt:634" },
        },
        text = {
          zh_TW = { name = "黑龍波",
            body = "放出彈跳的黑色電光打向目標敵人，造成 {damage} 魔法傷害，之後每 {jump_delay} 秒跳向 {jump_radius} 內最近、看得見且還沒被這道電光打過的敵方英雄或部隊，每個各造成 {damage} 魔法傷害，最多命中 {jump_count} 個單位。" },
          en = { name = "Black Dragon Wave",
            body = "Releases bouncing black lightning at the target enemy, dealing {damage} magic damage, then every {jump_delay} seconds it jumps to the nearest visible enemy hero or unit within {jump_radius} that this bolt has not hit yet, dealing {damage} magic damage to each, hitting up to {jump_count} units." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC, flags = { "visible_only" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "c10w_old_vo", at = "caster" },
            { op = "cue", id = "c10w_old_cast_sound", at = "caster" },
            { op = "chain", from = "caster", first = "target", jumps = "$jump_count - 1", radius = "$jump_radius",
              interval = "$jump_delay", team = "enemy", types = BASIC, flags = { "visible_only" }, order = "nearest",
              cue = "c10w_old_bolt",
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C10.lua:233-316",
            text = "YOLOSPAGHETTI 的 arc lightning 寫法：每個目標套隱藏 modifier_arc_lightning_datadriven，OnCreated 以 ApplyDamage（不經 AMHC，親泰死後照常傷害）造成 AbilityDamage，0.2 秒後從該目標當下位置找 800 內最近、這一串還沒打過的敵人（ENEMY、HERO|BASIC、FOW_VISIBLE，魔免選不到）。目標死亡後仍以它的位置往下跳。" },
          { policy = "fixed", ref = "vscripts/heroes/C_Neutral/C10.lua:206-223,245-256",
            text = "FC-165 D22 舊版，比照 D13 修：原作以 ability.target[i] == target 反查「目前是第幾串」：兩串同時打到同一個單位時（冷卻 9 秒、彈跳最長約 4 秒，冷卻被重置時才會發生），後面的迴圈會把兩串都認成同一串，計數與命中表混在一起、其中一串提早或延後結束。修法：每串各自計數與命中表（chain 的語意）。" },
          { policy = "note", ref = "npc/C_Neutral/C10.txt:566-605",
            text = "C10W_old.vo1、ArcLightning 音效與 c10w_old 粒子只是呈現。名稱「黑龍波」沿用本體，行為是連鎖閃電。" },
        },
        provenance = { "npc/C_Neutral/C10.txt:541-637", "vscripts/heroes/C_Neutral/C10.lua:200-316" },
      },
      -------------------------------------------------------------------------
      { id = "c10_old_energy_drain", source_id = "C10E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 12, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C10.txt:649" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C10.txt:650" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C10.txt:651" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C10.txt:647" },
          drain = { 300, 400, 500, 600, unit = "dmg", damage_type = "pure",
            source = "kv:Heal@npc/C_Neutral/C10.txt:711（Damage 與 HealAmount 都讀它；AbilityDamage@C10.txt:652 同值）" },
          stun = { 1, unit = "s", source = "kv:stun_duration@npc/C_Neutral/C10.txt:716" },
        },
        text = {
          zh_TW = { name = "能量奪取",
            body = "奪取目標敵人的生命：造成 {drain} 純粹傷害，親泰回復 {drain} 生命，並使目標暈眩 {stun} 秒。" },
          en = { name = "Energy Drain",
            body = "Drains the target enemy's life, dealing {drain} pure damage, healing Chikayasu for {drain} and stunning the target for {stun} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c10e_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "能量奪取暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Energy Drain Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "datadriven Stun 動作（引擎 modifier_stunned）。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c10e_old_mana_void_cast", at = "target" } },
          on_cast = {
            { op = "cue", id = "c10e_old_vo", at = "caster" },
            { op = "damage", to = "target", type = "pure", amount = "$drain", require_source_alive = false },
            { op = "heal", to = "caster", source = "caster", amount = "$drain" },
            { op = "apply_status", status = "c10e_old_stun", to = "target" },
            { op = "cue", id = "c10e_old_mana_void", at = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C10.txt:655-704",
            text = "全是 datadriven 動作：Damage（純粹，不經 AMHC）→ Heal 施法者固定量（不看實際造成的傷害）→ Stun。目標魔免時選不到（沒有 MAGIC_IMMUNE 旗標）。" },
          { policy = "note", ref = "npc/C_Neutral/C10.txt:657-700",
            text = "C10E_old.vo1、ManaVoid 音效與粒子只是呈現；AbilityCastAnimation 只是動作。" },
        },
        provenance = { "npc/C_Neutral/C10.txt:638-719" },
      },
      -------------------------------------------------------------------------
      { id = "c10_old_mushin_rebirth", source_id = "C10R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          evasion = { 8, 16, 24, 32, unit = "pct", source = "kv:miss@npc/C_Neutral/C10.txt:786（modifier_ninja_cloth EVASION_CONSTANT）" },
        },
        text = {
          zh_TW = { name = "無想轉生", body = "被動：閃避 {evasion}。" },
          en = { name = "Mushin Rebirth", body = "Passive: {evasion} evasion." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { evasion_pct = "$evasion" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C10.txt:721-788 / vscripts/heroes/C_Neutral/C19.lua:284-305",
            text = "閃避是 Passive 的 modifier_ninja_cloth（和忍者衣等道具、B06、B35、C19 同名的 modifier），另由 modifier_C19T（借用 C19.lua 的 C19T_OnIntervalThink）每秒檢查、被移除就重新套上；OnUpgrade 先移除再於 0.1 秒後重套以更新數值。效果等於永久閃避，資料寫成被動屬性；同名 modifier 互相覆蓋的細節不列入（同 miyamoto_musashi、sanada_yukimura_old）。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C19.lua:300-305",
            text = "C19T_OnOnCreated 設層數 1 只是給 C19T 的標記，舊版親泰沒有用到。" },
        },
        provenance = { "npc/C_Neutral/C10.txt:720-789", "vscripts/heroes/C_Neutral/C19.lua:284-305" },
      },
      -------------------------------------------------------------------------
      { id = "c10_old_rising_dragon_dance", source_id = "C10T_old", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          required_level = { 6, 12, 18, unit = "level", source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          attack_bonus = { 25, 45, 65, unit = "dmg", source = "kv:buff_phy_atk@npc/C_Neutral/C10.txt:844（PREATTACK_BONUS_DAMAGE）" },
          chance = { 50, unit = "pct", source = "kv:chance@npc/C_Neutral/C10.txt:849",
            note = "C10.lua:335-340 RandomInt(1,100) > chance 才不觸發，實際機率正好 50%" },
          wave_radius = { 245, unit = "dist", source = "kv:radius@npc/C_Neutral/C10.txt:854" },
          wave_distance = { 800, unit = "dist", source = "kv:distance@npc/C_Neutral/C10.txt:859" },
          wave_speed = { 1380, unit = "speed", source = "kv:speed@npc/C_Neutral/C10.txt:864" },
          damage = { 160, 235, 310, unit = "dmg", damage_type = "pure",
            source = "kv:damage@npc/C_Neutral/C10.txt:869（AbilityDamage@C10.txt:802 同值，腳本讀 damage）" },
          immune_ratio = { 50, unit = "pct", source = "script:vscripts/heroes/C_Neutral/C10.lua:323-324 魔免目標 damage*0.5" },
        },
        text = {
          zh_TW = { name = "昇龍亂舞",
            body = "被動：攻擊力 +{attack_bonus}。對非建築目標出手時有 {chance} 機率朝面向放出劍氣（半徑 {wave_radius}、飛行 {wave_distance}、速度 {wave_speed}），穿透路徑上的敵方英雄與部隊（包含魔免），各造成 {damage} 純粹傷害；對魔免目標只造成 {immune_ratio}。" },
          en = { name = "Rising Dragon Dance",
            body = "Passive: +{attack_bonus} attack damage. When starting an attack against a non-building, has a {chance} chance to release a sword wave in the facing direction (radius {wave_radius}, travels {wave_distance} at speed {wave_speed}) that pierces enemy heroes and units (spell-immune ones included), dealing {damage} pure damage to each; spell-immune targets take only {immune_ratio} of it." },
        },
        target = { type = "none" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public",
          stats = { attack_damage = "$attack_bonus" },
          triggers = {
            on_attack_launch = { filter = "not event.target.is_building", actions = {
              { op = "chance", p = "$chance", actions = {
                { op = "cue", id = "c10t_old_gesture", at = "self" },
                { op = "projectile", kind = "linear", from = "self", direction = "self.forward", speed = "$wave_speed",
                  distance = "$wave_distance", radius = "$wave_radius", pierce = true, team = "enemy", types = BASIC,
                  flags = { "include_magic_immune" }, vision = "$wave_radius", cue = "c10t_old_wave",
                  on_hit = {
                    { op = "damage", to = "it", type = "pure",
                      amount = "select(it.is_magic_immune, $damage * $immune_ratio, $damage)" },
                  } },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C10.txt:806-831 / vscripts/heroes/C_Neutral/C10.lua:331-372",
            text = "OnAttack（出手瞬間）擲骰，對建築不觸發；劍氣從親泰位置沿當下面向（不是朝攻擊目標）飛出，ENEMY、HERO|BASIC、MAGIC_IMMUNE_ENEMIES，DeleteOnHit 0 穿透，提供 245 視野。反補友軍時也會觸發，但劍氣只打敵人。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C10.lua:319-329",
            text = "OnProjectileHitUnit 以 AMHC:Damage 純粹傷害：親泰在劍氣飛行中死亡時不再傷害（規則層）；魔免目標傷害減半。" },
          { policy = "as_source", ref = "npc/C_Neutral/C10.txt:795",
            text = "AbilityBehavior 有 DIRECTIONAL，但技能是被動，沒有作用。PREATTACK_BONUS_DAMAGE 是綠字攻擊力。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C10.lua:371",
            text = "StartGestureWithPlaybackRate(ACT_DOTA_CAST_ABILITY_2) 與 c10w 粒子只是呈現。" },
        },
        provenance = { "npc/C_Neutral/C10.txt:790-873", "vscripts/heroes/C_Neutral/C10.lua:319-372" },
      },
    },
  }
end
