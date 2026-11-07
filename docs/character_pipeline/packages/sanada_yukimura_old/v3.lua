-- B06 真田幸村（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["B06"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:285）；自動學習 heromap_autoskill["11"] = ""
-- （addon_game_mode.lua:388），沒有 D。KV 有 B06W_old／B06E_old／B06R_old／B06T_old（npc/B_Unified/B06.txt:93、270、489、726）。
-- W/E/R/T → Q/W/E/R。原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 B06*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、B06_precache 屬規則層或資產，不列入。
-- 慣例照本體 B06：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；HERO | BASIC 寫成 BASIC 清單；
-- datadriven Damage（不經 AMHC）在幸村死亡後仍會造成傷害，寫 require_source_alive = false。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B06_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B06", text = { zh_TW = { name = "真田幸村（舊版）" }, en = { name = "Sanada Yukimura (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b06_old_brave_warriors", source_id = "B06W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B06.txt:100" },
          mana_cost = { 140, 160, 180, 200, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B06.txt:101" },
          radius = { 500, unit = "dist", source = "kv:ActOnTargets Radius@npc/B_Unified/B06.txt:122（AOERadius 102 同值）" },
          duration = { 20, unit = "s", source = "kv:modifier_B06W Duration@npc/B_Unified/B06.txt:141" },
          hp_regen = { 10, 20, 30, 40, unit = "per_s", source = "kv:Re_hp@npc/B_Unified/B06.txt:155" },
          attack_bonus = { 40, 50, 60, 70, unit = "dmg", source = "kv:Attack_Bonus@npc/B_Unified/B06.txt:160" },
        },
        text = {
          zh_TW = { name = "真田勇士",
            body = "振奮周圍的友方英雄與部隊（含自己），{duration} 秒內攻擊 +{attack_bonus}、每秒回復 {hp_regen} 生命。重複施放時各自計時、效果可以疊加。" },
          en = { name = "Sanada Braves",
            body = "Rallies nearby allied heroes and units (Yukimura included): for {duration} seconds they gain +{attack_bonus} attack damage and regenerate {hp_regen} health per second. Repeated casts stack, each with its own duration." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b06w_old_rally",
            text = { zh_TW = { name = "真田勇士", body = "攻擊 +{attack_bonus}，每秒回復 {hp_regen} 生命。" },
              en = { name = "Sanada Braves", body = "+{attack_bonus} attack damage; regenerates {hp_regen} health per second." } },
            duration = "$duration", stacking = "independent", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b06w_old_rally_glow",
            stats = { attack_damage = "$attack_bonus", hp_regen = "$hp_regen" },
            notes = "原作 modifier_B06W（B06W_old 版）：MODIFIER_ATTRIBUTE_MULTIPLE、Duration 20、HEALTH_REGEN_CONSTANT、PREATTACK_BONUS_DAMAGE（綠字）；IsBuff，沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b06w_old_rally_sound", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "ally", types = BASIC,
                flags = { "include_magic_immune" } },
              actions = {
                { op = "apply_status", status = "b06w_old_rally", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B06.txt:136-147",
            text = "和本體不同：modifier_B06W 有 MODIFIER_ATTRIBUTE_MULTIPLE，每次施放都另掛一份、各自計時，數值疊加（冷卻 10 秒、持續 20 秒，同一目標最多同時有兩份）；持續 20 秒、只回血不回魔、攻擊 +40～70。" },
          { policy = "as_source", ref = "npc/B_Unified/B06.txt:114-133",
            text = "ActOnTargets 友方 HERO|BASIC、以施法者為中心 500：包含自己與魔免友軍，排除建築；隱形友軍照樣選到。" },
          { policy = "note", ref = "npc/B_Unified/B06.txt:107-113",
            text = "item_soul_adder.lua sound 只播音效 overwhelming_buff。" },
        },
        provenance = { "npc/B_Unified/B06.txt:93-164", "vscripts/items/Addon_Items/item_soul_adder.lua:61-80" },
      },
      -------------------------------------------------------------------------
      { id = "b06_old_flame_fang", source_id = "B06E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B06.txt:278" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B06.txt:279" },
          cast_range = { 250, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B06.txt:280" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B06.txt:277" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/B_Unified/B06.txt:365" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:Stun_Time@npc/B_Unified/B06.txt:362" },
          projectile_speed = { 3000, unit = "speed", source = "kv:TrackingProjectile MoveSpeed@npc/B_Unified/B06.txt:301" },
        },
        text = {
          zh_TW = { name = "炎突",
            body = "從槍尖射出炎彈（速度 {projectile_speed}）攻擊指定敵人，命中造成 {damage} 魔法傷害並暈眩 {stun} 秒。炎彈可以被閃躲。" },
          en = { name = "Flame Thrust",
            body = "Launches a flame bolt from the spear tip (speed {projectile_speed}) at the target enemy; on hit it deals {damage} magic damage and stuns for {stun} seconds. The bolt can be dodged." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b06e_old_stun", runtime_id = "stun",
            text = { zh_TW = { name = "暈眩", body = "無法移動、攻擊或施法。" }, en = { name = "Stunned", body = "Cannot move, attack or cast." } },
            duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", states = { "stun" }, cue = "b06e_old_stunned",
            notes = "原作 modifier_B06E（B06E_old 版）：IsPurgable 1、IsStunDebuff、STUNNED；OnCreated 只播音效與粒子。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b06e_old_cast_vo", at = "caster" },
            { op = "cue", id = "b06e_old_spear_glow", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed", dodgeable = true,
              cue = "b06e_old_flame_bolt",
              on_hit = {
                { op = "apply_status", status = "b06e_old_stun", to = "it" },
                { op = "cue", id = "b06e_old_hit", at = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B06.txt:310-325",
            text = "和本體不同：OnProjectileHitUnit 只有 ApplyModifier 與 Damage，沒有 RunScript b06e_Shot，所以不彈跳；炎彈速度 3000、冷卻 15、暈眩 0.5～2 秒、傷害 150～450。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B06_Sanada_Yukimura.lua:69-87",
            text = "OnSpellStart 仍呼叫 b06e_start：把 caster.b06ecountofattacked 設為 5，但舊版沒有讀它的地方；其餘只是 0.01～0.7 秒的粒子。" },
          { policy = "as_source", ref = "npc/B_Unified/B06.txt:270-281",
            text = "沒寫 SpellImmunityType 與 AbilityUnitTargetFlags：不能指定魔免敵人。飛行中目標變成魔免時，魔法傷害由規則層歸零，暈眩照樣套上（datadriven ApplyModifier，推定）。" },
          { policy = "as_source", ref = "npc/B_Unified/B06.txt:314-324",
            text = "命中的 Damage 是 datadriven（不經 AMHC），幸村在炎彈飛行中死亡仍照常傷害與暈眩。" },
        },
        provenance = { "npc/B_Unified/B06.txt:270-366", "vscripts/heroes/B_Unified/B06_Sanada_Yukimura.lua:69-87" },
      },
      -------------------------------------------------------------------------
      { id = "b06_old_mind_eye", source_id = "B06R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          evasion = { 8, 16, 24, 32, unit = "pct", source = "kv:miss（modifier_ninja_cloth EVASION_CONSTANT）@npc/B_Unified/B06.txt:559" },
        },
        text = {
          zh_TW = { name = "心眼", body = "被動：{evasion} 機率閃避攻擊。" },
          en = { name = "Mind's Eye", body = "Passive: {evasion} chance to evade attacks." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { evasion_pct = "$evasion" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B06.txt:489-563",
            text = "和本體不同：只有閃避（modifier_ninja_cloth EVASION_CONSTANT 8/16/24/32），沒有被指定時的真田心眼術魔抗。" },
          { policy = "as_source", ref = "npc/B_Unified/B06.txt:522-533",
            text = "modifier_C19T（Passive、借用 C19.lua 的 C19T_OnIntervalThink）每秒檢查 modifier_ninja_cloth，被移除就重新套上、層數設為一；效果等於永久閃避，驅散不掉。" },
          { policy = "note", ref = "npc/B_Unified/B06.txt:500-519",
            text = "OnUpgrade 先移除 modifier_ninja_cloth、0.1 秒後重掛，讓新等級的閃避生效；升級後有 0.1 秒沒有閃避，不影響結算，不另外表達。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C19.lua:284-305",
            text = "modifier_ninja_cloth 與 C01、C02、C10、C19、C21、C22、C24、B35 的同名 modifier 不會同時出現在同一位英雄身上；層數只是顯示。" },
        },
        provenance = { "npc/B_Unified/B06.txt:489-563", "vscripts/heroes/C_Neutral/C19.lua:284-305" },
      },
      -------------------------------------------------------------------------
      { id = "b06_old_first_warrior", source_id = "B06T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 140, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B06.txt:735" },
          mana_cost = { 300, 400, 500, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B06.txt:739" },
          required_level = { 6, 12, 18, unit = "level",
            source = "kv:RequiredLevel@npc/B_Unified/B06.txt:737", note = "RequiredLevel 6、LevelsBetweenUpgrades 6（738）→ 6/12/18" },
          duration = { 40, 50, 60, unit = "s", source = "kv:During@npc/B_Unified/B06.txt:834" },
          base_attack_time = { 0.9, unit = "s",
            source = "kv:modifier_B06T MODIFIER_PROPERTY_BASE_ATTACK_TIME_CONSTANT@npc/B_Unified/B06.txt:791（Basic_Attack_Speed 824 同值但沒被引用）" },
          max_hp = { 500, 750, 1000, unit = "hp", source = "kv:Bonus_Health@npc/B_Unified/B06.txt:829（HEALTH_BONUS 792 與 Heal 769 共用）" },
          move_speed = { 250, unit = "speed", source = "kv:modifier_B06T MODIFIER_PROPERTY_MOVESPEED_BONUS_CONSTANT@npc/B_Unified/B06.txt:793" },
          base_attack = { 30, 40, 50, unit = "dmg", source = "kv:attack（BASEATTACK_BONUSDAMAGE）@npc/B_Unified/B06.txt:839" },
          armor = { 3, 6, 9, unit = "armor", source = "kv:defend（PHYSICAL_ARMOR_BONUS）@npc/B_Unified/B06.txt:844" },
        },
        text = {
          zh_TW = { name = "日本第一武士",
            body = "持續 {duration} 秒：基礎攻擊間隔變為 {base_attack_time} 秒、生命上限 +{max_hp} 並立即回復等量生命、基礎攻擊 +{base_attack}、護甲 +{armor}、移動速度 +{move_speed}。死亡時效果結束。" },
          en = { name = "Japan's Greatest Warrior",
            body = "For {duration} seconds: base attack time becomes {base_attack_time} seconds, +{max_hp} maximum health with an instant heal of the same amount, +{base_attack} base attack damage, +{armor} armor and +{move_speed} movement speed. Ends on death." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "b06t_old_warrior",
            text = { zh_TW = { name = "日本第一武士", body = "基礎攻擊間隔 {base_attack_time} 秒、生命上限 +{max_hp}、基礎攻擊 +{base_attack}、護甲 +{armor}、移動速度 +{move_speed}。" },
              en = { name = "Japan's Greatest Warrior", body = "Base attack time {base_attack_time} seconds, +{max_hp} maximum health, +{base_attack} base attack damage, +{armor} armor and +{move_speed} movement speed." } },
            duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", remove_on = { "death" }, cue = "b06t_old_strafe",
            stats = { base_attack_time = { op = "set_absolute", value = "$base_attack_time" }, max_hp = "$max_hp",
              move_speed = "$move_speed", base_attack_damage = "$base_attack", armor = "$armor" },
            triggers = {
              on_attack_hit = { { op = "cue", id = "b06t_old_hit_shake", at = "event.target" } },
            },
            notes = "原作 modifier_B06T（B06T_old 版）：BASE_ATTACK_TIME_CONSTANT 0.9、HEALTH_BONUS、MOVESPEED_BONUS_CONSTANT 250、BASEATTACK_BONUSDAMAGE（白字）、PHYSICAL_ARMOR_BONUS；OnDeath 移除自己；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b06t_old_cast_vo", at = "caster" },
            { op = "apply_status", status = "b06t_old_warrior", to = "caster" },
            { op = "heal", to = "caster", source = "caster", amount = "$max_hp" },
            { op = "cue", id = "b06t_old_aura_fx", at = "caster" },
            { op = "cue", id = "b06t_old_title", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B06.txt:726-847",
            text = "和本體不同：沒有全軍移速光環；冷卻固定 140；移速是固定 +250（MOVESPEED_BONUS_CONSTANT），不是 +30%；基礎攻擊 +30/40/50，多了護甲 +3/6/9。" },
          { policy = "as_source", ref = "npc/B_Unified/B06.txt:757-770",
            text = "先套 modifier_B06T（生命上限提高）再 Heal Bonus_Health，回復量不會被上限截掉；Heal 走一般治療。ApplyModifier 的 Target 只有 Center CASTER，只套在自己身上。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B06_Sanada_Yukimura.lua:136-158",
            text = "B06T_effect 每 10 秒重播一次粒子、B06T_SE 在命中時播震動粒子、showTitle 播稱號，都只是呈現。" },
        },
        provenance = { "npc/B_Unified/B06.txt:726-847", "vscripts/heroes/B_Unified/B06_Sanada_Yukimura.lua:136-158" },
      },
    },
  }
end
