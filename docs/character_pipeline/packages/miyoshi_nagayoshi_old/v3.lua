-- B10 三好長慶（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["B10"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:289）；自動學習 heromap_autoskill["11"] = ""
-- （addon_game_mode.lua:392），沒有 D。KV 有 B10W_old／B10E_old／B10R_old／B10T_old（npc/B_Unified/B10.txt:587、659、735、823）。
-- W/E/R/T → Q/W/E/R（B10T_old 的 HotKeyOverride "W" 只改快捷鍵，槽位照 T）。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt，UTF-16，查無 B10*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、B10_precache 屬規則層或資產，不列入。
-- 慣例照本體 B10：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；datadriven Stun（引擎 modifier_stunned）寫 "strong"；
-- HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B10_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B10", text = { zh_TW = { name = "三好長慶（舊版）" }, en = { name = "Miyoshi Nagayoshi (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b10_old_swift_march", source_id = "B10W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B10.txt:594" },
          mana_cost = { 50, 55, 60, 65, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B10.txt:595" },
          move_speed = { 15, 20, 25, 30, unit = "pct", source = "kv:movespeed@npc/B_Unified/B10.txt:602" },
          duration = { 15, 16, 17, 18, unit = "s", source = "kv:duration@npc/B_Unified/B10.txt:606" },
        },
        text = {
          zh_TW = { name = "疾行",
            body = "{duration} 秒內移動速度 +{move_speed}，並可以穿過其他單位。" },
          en = { name = "Swift March",
            body = "For {duration} seconds, gains +{move_speed} movement speed and can pass through other units." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b10w_old_march",
            text = { zh_TW = { name = "疾行", body = "移動速度 +{move_speed}，可以穿過其他單位。" },
              en = { name = "Swift March", body = "+{move_speed} movement speed; can pass through other units." } },
            duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b10w_old_march_glow", states = { "no_unit_collision" },
            stats = { move_speed_pct = "$move_speed" },
            notes = "原作 modifier_B10W（B10W_old 版）：IsBuff、MOVESPEED_BONUS_PERCENTAGE、states NO_UNIT_COLLISION；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b10w_old_march", to = "caster" },
            { op = "cue", id = "b10w_old_cast_sound", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B10.txt:650-653",
            text = "States 區塊的鍵寫成小寫 \"states\"；推定 KeyValues 鍵名不分大小寫，NO_UNIT_COLLISION 照樣生效（見 assumptions.md）。" },
          { policy = "as_source", ref = "npc/B_Unified/B10.txt:619-627",
            text = "ApplyModifier 的 Target 只有 Center CASTER，只套在自己身上。和本體不同：舊版 W 是單純的加速，不是亡靈之怒。" },
          { policy = "note", ref = "npc/B_Unified/B10.txt:628-633",
            text = "item_soul_adder.lua sound 只播音效 powershot_falcon_bow；圖示沿用 A33T。" },
        },
        provenance = { "npc/B_Unified/B10.txt:587-656" },
      },
      -------------------------------------------------------------------------
      { id = "b10_old_axe_throw", source_id = "B10E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B10.txt:669" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B10.txt:670" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B10.txt:671" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B10.txt:667" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/B_Unified/B10.txt:732" },
          stun = { 1, 1.5, 2, 2.5, unit = "s", source = "kv:duration@npc/B_Unified/B10.txt:729" },
          bolt_speed = { 3500, unit = "speed", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B10.lua:8-32 每次 timer 前進 35，間隔 0.01 秒（照本體 B10E 的換算）" },
        },
        text = {
          zh_TW = { name = "飛斧",
            body = "擲出飛斧追向目標敵人，追上時暈眩 {stun} 秒並造成 {damage} 魔法傷害。飛斧無法躲避。" },
          en = { name = "Axe Throw",
            body = "Hurls an axe that chases the target enemy; when it catches up it stuns for {stun} seconds and deals {damage} magic damage. The axe cannot be dodged." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b10e_old_stun", runtime_id = "stun",
            text = { zh_TW = { name = "暈眩", body = "無法移動、攻擊或施法。" }, en = { name = "Stunned", body = "Cannot move, attack or cast." } },
            duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" }, cue = "b10e_old_stunned",
            notes = "原作 modifier_B10E（B10E_old 版，Duration 0）的 OnCreated 用 datadriven Stun，等於引擎內建 modifier_stunned；照本體慣例 dispel = strong。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b10e_old_vo", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$bolt_speed",
              dodgeable = false, cue = "b10e_old_axe",
              on_hit = {
                { op = "apply_status", status = "b10e_old_stun", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "cue", id = "b10e_old_hit_sound", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B10.lua:16-34",
            text = "飛斧與本體 B10E 共用同一個函式：每 0.01 秒前進 35 的 timer，實際受伺服器幀率限制（30Hz 約每秒 1050）；照本體的設計意圖寫成速度 3500 的追蹤投射物。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B10.lua:16-25",
            text = "不是真正的投射物：每一步重新追向目標當下位置，閃現或隱形都躲不掉（dodgeable = false）；與目標距離小於 50 時命中。目標中途死亡時立即結算，打在死亡單位上沒有效果。" },
          { policy = "as_source", ref = "npc/B_Unified/B10.txt:698-722",
            text = "datadriven Stun 與 Damage 不經 AMHC：長慶在飛斧飛行中死亡仍會暈眩與傷害（require_source_alive = false）。先暈眩再傷害。和本體不同：冷卻 15、耗魔 100～160、暈眩 1～2.5 秒、傷害 100～400。" },
          { policy = "as_source", ref = "npc/B_Unified/B10.txt:659-671",
            text = "沒寫 SpellImmunityType 與目標旗標：不能指定魔免敵人；飛行途中目標變成魔免仍會被暈眩，魔法傷害由規則層歸零。" },
          { policy = "note", ref = "npc/B_Unified/B10.txt:680-694",
            text = "FireSound B34W.vo1 與 Hero_Beastmaster.Wild_Axes 只是呈現；粒子沿用本體 b10e。" },
        },
        provenance = { "npc/B_Unified/B10.txt:659-733", "vscripts/heroes/B_Unified/B10.lua:1-35" },
      },
      -------------------------------------------------------------------------
      { id = "b10_old_war_spirit", source_id = "B10R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic", "aura" },
        values = {
          radius = { 900, unit = "dist", source = "kv:modifier_B10R_aura Aura_Radius@npc/B_Unified/B10.txt:774（AoERadius 745 同值）" },
          attack_speed = { 20, 30, 40, 50, unit = "atk_speed", source = "kv:add_atkspeed@npc/B_Unified/B10.txt:752" },
          move_speed = { 2, 4, 6, 8, unit = "pct", source = "kv:add_movespeed@npc/B_Unified/B10.txt:757" },
          aura_linger = { 0.5, unit = "s", hidden = true, source = "assumed:Dota datadriven 光環預設殘留 0.5 秒" },
        },
        text = {
          zh_TW = { name = "戰魂",
            body = "光環：周圍 {radius} 內的友方英雄與部隊（含自己）攻擊速度 +{attack_speed}、移動速度 +{move_speed}。長慶自己另外再獲得一份同樣的加成。" },
          en = { name = "War Spirit",
            body = "Aura: allied heroes and units within {radius} (Nagayoshi included) gain +{attack_speed} attack speed and +{move_speed} movement speed. Nagayoshi himself gets the same bonus a second time." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        statuses = {
          { id = "b10r_old_buff",
            text = { zh_TW = { name = "戰魂", body = "攻擊速度 +{attack_speed}、移動速度 +{move_speed}。" },
              en = { name = "War Spirit", body = "+{attack_speed} attack speed and +{move_speed} movement speed." } },
            duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", cue = "b10r_old_soft_ring",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed" },
            notes = "原作 modifier_B10R_buff：光環子狀態，ATTACKSPEED_BONUS_CONSTANT、MOVESPEED_BONUS_PERCENTAGE；OnCreated／OnDestroy 只掛粒子。由光環管理。" },
        },
        passive = {
          reveal = "public",
          stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed" },
          aura = { radius = "$radius", team = "ally", types = BASIC, flags = { "include_magic_immune" },
            status = "b10r_old_buff", linger = "$aura_linger", include_self = true },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B10.txt:769-796",
            text = "光環本體 modifier_B10R_aura（Passive，在長慶身上）自己就有同樣的 Properties，而且 Aura_ApplyToCaster 1 又給長慶一份 modifier_B10R_buff：長慶得到兩倍加成，隊友一份。" },
          { policy = "as_source", ref = "npc/B_Unified/B10.txt:754-758",
            text = "add_movespeed 的條目把 var_type 拼成 vsr_type；推定 GetSpecialValueFor 照樣讀到數值（見 assumptions.md）。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B10.lua:37-47",
            text = "B10R_buff_OnCreated／OnDestroy 只建立與清除隊友腳下的光圈粒子（借用 B21R_old 的特效）。" },
        },
        provenance = { "npc/B_Unified/B10.txt:735-819", "vscripts/heroes/B_Unified/B10.lua:37-47" },
      },
      -------------------------------------------------------------------------
      { id = "b10_old_demonic_frenzy", source_id = "B10T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 135, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B10.txt:837" },
          mana_cost = { 250, 310, 370, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B10.txt:838" },
          required_level = { 6, 12, 18, unit = "level",
            source = "kv:RequiredLevel@npc/B_Unified/B10.txt:835", note = "RequiredLevel 6、LevelsBetweenUpgrades 6（836）→ 6/12/18" },
          duration = { 25, unit = "s", source = "kv:B10T_Duration@npc/B_Unified/B10.txt:893" },
          attack_speed = { 40, 70, 100, unit = "atk_speed", source = "kv:atkspeed@npc/B_Unified/B10.txt:913" },
          base_damage = { 10, 20, 30, unit = "pct", source = "kv:atk（BASEDAMAGEOUTGOING_PERCENTAGE）@npc/B_Unified/B10.txt:903" },
          move_speed = { 10, 20, 30, unit = "pct", source = "kv:movespeed@npc/B_Unified/B10.txt:898" },
          agi = { 110, 150, 200, unit = "stat", source = "kv:ADD_AGI（STATS_AGILITY_BONUS）@npc/B_Unified/B10.txt:908" },
        },
        text = {
          zh_TW = { name = "魔性狂化",
            body = "狂化 {duration} 秒：攻擊速度 +{attack_speed}、基礎攻擊力 +{base_damage}、移動速度 +{move_speed}、敏捷 +{agi}。" },
          en = { name = "Demonic Frenzy",
            body = "Goes berserk for {duration} seconds: +{attack_speed} attack speed, +{base_damage} base attack damage, +{move_speed} movement speed and +{agi} agility." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "b10t_old_frenzy",
            text = { zh_TW = { name = "魔性狂化", body = "攻擊速度 +{attack_speed}、基礎攻擊力 +{base_damage}、移動速度 +{move_speed}、敏捷 +{agi}。" },
              en = { name = "Demonic Frenzy", body = "+{attack_speed} attack speed, +{base_damage} base attack damage, +{move_speed} movement speed and +{agi} agility." } },
            duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b10t_old_red_aura",
            stats = { attack_speed = "$attack_speed", base_attack_damage_pct = "$base_damage", move_speed_pct = "$move_speed",
              agi = "$agi" },
            notes = "原作 modifier_A06W（B10T_old 自己的 Modifiers 區塊）：IsBuff、IsPurgable 1；ATTACKSPEED_BONUS_CONSTANT、BASEDAMAGEOUTGOING_PERCENTAGE、MOVESPEED_BONUS_PERCENTAGE、STATS_AGILITY_BONUS。Duration 由 ApplyModifier 的 %B10T_Duration 給。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b10t_old_cast_vo", at = "caster" },
            { op = "apply_status", status = "b10t_old_frenzy", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B10.txt:873-886",
            text = "modifier 名稱沿用 A06W，但定義在 B10T_old 自己的 Modifiers 區塊、數值讀 B10T_old 的 AbilitySpecial，和 A06 的同名 modifier 互不影響。IsPurgable 1：敵方的一般驅散可以移除。" },
          { policy = "as_source", ref = "npc/B_Unified/B10.txt:854-863",
            text = "ApplyModifier 的 Target 只有 Center CASTER，只套在自己身上。持續時間三級都是 25 秒。" },
          { policy = "note", ref = "npc/B_Unified/B10.txt:833,841-842",
            text = "HotKeyOverride W、AbilityCastRange 0、AbilityCastRangeBuffer 250 對無目標技能沒有作用；FireSound A06W.vo1 與 berserkercaster 只是呈現。" },
        },
        provenance = { "npc/B_Unified/B10.txt:823-916" },
      },
    },
  }
end
