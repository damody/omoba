-- A14 前田利家（舊版，11 版 heromap_skill A14 ["11"] = "WERDT"）— package schema v3。
-- 版本：選擇 11 版時 tofar_goback.lua choose_11 拔掉全部技能，依 _G.heromap_skill[A14]["11"] = "WERDT"
-- 依序 AddAbility("A14W_old"、"A14E_old"、"A14R_old"、"A14D_old"、"A14T_old")（vscripts/items/Addon_Items/tofar_goback.lua:96-140；
-- vscripts/addon_game_mode.lua:256,359），自動學習清單為空。槽位 W/E/R/T → Q/W/E/R，A14D_old 照本體慣例放 D。
-- A14D_old 是 NOT_LEARNABLE：A14T_old 施放時把它設成大招等級並啟用，大招結束時停用（vscripts/heroes/A_Oda/A14.lua:388-389,419）。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A14_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"。
return function(ctx)
  local UNITS = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "A14_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A14", text = { zh_TW = { name = "前田利家（舊版）" }, en = { name = "Maeda Toshiie (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a14_old_instinct_strike", source_id = "A14W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 300, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:stun" },
          push_distance = { 1070, unit = "dist",
            source = "script:vscripts/heroes/A_Oda/A14.lua:243-244,277 Physics 初速 1700/30、預設摩擦 0.05（libraries/physics.lua:1731），速度低於 100 時撞擊；推算約 1070（同本體）" },
          push_duration = { 1.87, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A14.lua:277 推算約 56 幀" },
          order_delay = { 0.2, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A14.lua:236 Timers 0.2" },
          contact_radius = { 100, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A14.lua:286 CalcDistanceBetweenEntityOBB(unit, target) <= 100" },
        },
        text = {
          zh_TW = { name = "本能之打擊",
            body = "對目標敵人造成 {damage} 魔法傷害並暈眩 {stun} 秒，同時把它朝遠離自己的方向擊飛約 {push_distance}；撞到其他單位或停下時，目標再受到 {damage} 魔法傷害。之後利家會自動攻擊該目標。" },
          en = { name = "Instinct Strike",
            body = "Deals {damage} magic damage to the target enemy and stuns it for {stun} seconds while launching it about {push_distance} away from Toshiie; when it hits another unit or stops, it takes another {damage} magic damage. Toshiie then attacks the target." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = UNITS },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a14w_old_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被本能之打擊擊飛，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Knocked flying by Instinct Strike and stunned for {stun} seconds." },
            },
            reveal = "public", states = { "stun" }, notes = "原作 ApplyDataDrivenModifier 引擎內建 modifier_stunned。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a14w_old_hit_sound", at = "target" },
            { op = "delay", time = "$order_delay", actions = {
              { op = "issue_order", who = "caster", order = "attack", target = "target" },
            } },
            { op = "if", cond = "not target.is_building", actions = {
              { op = "if", cond = "not target.is_magic_immune", actions = {
                { op = "apply_status", status = "a14w_old_stun", to = "target" },
                { op = "damage", to = "target", type = "magical", amount = "$damage" },
              } },
              { op = "knockback", who = "target", from = "caster", distance = "$push_distance", duration = "$push_duration",
                stun = false, stop_on_contact = true, contact_radius = "$contact_radius",
                on_end = {
                  { op = "cue", id = "a14w_impact", at = "target" },
                  { op = "damage", to = "target", type = "magical", amount = "$damage", require_source_alive = false },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "low",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:249-252,282,292",
            text = "同一個 damageTable 用了兩次：施放時（目標非魔免）一次，撞擊或停下時再一次，只打目標本人；舊版沒有本體的範圍傷害與緩速。暈眩不在撞擊時移除（移除那行已註解）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:285-298",
            text = "接觸判定的迴圈在檢查第一個單位後就 return：每幀只看搜尋結果（目標前方 300、半徑 250、敵我都算）的第一個單位，OBB 距離 <= 100 才撞擊；資料以 knockback.stop_on_contact 表達，實際較原作容易撞到。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:259-266",
            text = "速度方向改變（被地形擋住）時只停住目標、不造成傷害也不結束計時器，下一幀因速度 < 100 才撞擊並造成傷害；結果等同停下時撞擊。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:277",
            text = "停止條件是 timecounter < 80 且速度 < 100；正常約 56 幀就停下。超過 80 幀仍未停下時計時器永不結束（不再造成傷害），屬極端情形。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A14.lua:243-244",
            text = "Physics 每幀以 0.95 衰減；擊飛距離與時間照 30Hz 積分推算（同本體 a14_impact_spear）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A14.lua:247,254",
            text = "cur_target_vec、timecounter 是未加 local 的全域變數，與本體 A14W 共用；同時施放會互相干擾，資料不重現。" },
          { policy = "note", ref = "npc/A_Oda/A14.txt:505", text = "KV flyTime 0.8 沒有被舊版腳本使用。" },
        },
        provenance = { "npc/A_Oda/A14.txt:460-513", "vscripts/heroes/A_Oda/A14.lua:222-302", "vscripts/libraries/physics.lua:1009,1731" },
      },
      -------------------------------------------------------------------------
      { id = "a14_old_wild_slash", source_id = "A14E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 300, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 250, 350, 450, 550, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          ground_damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:damage_on_ground" },
          fly_time = { 1.5, unit = "s", source = "kv:flyTime" },
          stun = { 2, unit = "s", source = "kv:stun_Time" },
          height = { 600, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A14.lua:331 knockback_height = 600" },
          ground_radius = { 300, unit = "dist", source = "script:vscripts/heroes/A_Oda/A14.lua:359 FindUnitsInRadius 300" },
          order_delay = { 0.2, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A14.lua:336 Timers 0.2" },
        },
        text = {
          zh_TW = { name = "野生之斬擊",
            body = "對目標敵人造成 {damage} 魔法傷害，把它原地挑上空中 {fly_time} 秒並暈眩 {stun} 秒。目標落地時，對落點 {ground_radius} 內的非建築敵人打斷動作，非魔免者再受到 {ground_damage} 魔法傷害。之後利家會自動攻擊該目標。" },
          en = { name = "Wild Slash",
            body = "Deals {damage} magic damage to the target enemy, tosses it into the air in place for {fly_time} seconds and stuns it for {stun} seconds. When it lands, non-building enemies within {ground_radius} of it are interrupted, and those not spell immune take {ground_damage} magic damage. Toshiie then attacks the target." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = UNITS },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a14e_old_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被野生之斬擊挑起，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Tossed by Wild Slash and stunned for {stun} seconds." },
            },
            reveal = "public", states = { "stun" }, notes = "原作 AddNewModifier 引擎內建 modifier_stunned。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a14e_old_thunder_start", at = "target" },
            { op = "cue", id = "a14w_old_hit_sound", at = "target" },
            { op = "knockback", who = "target", from = "target", distance = "0", duration = "$fly_time", height = "$height", stun = true },
            { op = "delay", time = "$order_delay", actions = {
              { op = "issue_order", who = "caster", order = "attack", target = "target" },
            } },
            { op = "apply_status", status = "a14e_old_stun", to = "target" },
            { op = "damage", to = "target", type = "magical", amount = "$damage" },
            { op = "delay", time = "$fly_time", actions = {
              { op = "if", cond = "exists(target)", actions = {
                { op = "cue", id = "a14e_old_thunder_clap", at = "target" },
                { op = "cue", id = "a14w_old_hit_sound", at = "target" },
                { op = "for_each",
                  select = { from = "target", shape = "circle", radius = "$ground_radius", team = "enemy",
                    types = UNITS, flags = { "include_magic_immune" } },
                  actions = {
                    { op = "issue_order", who = "it", order = "stop" },
                    { op = "if", cond = "not it.is_magic_immune", actions = {
                      { op = "damage", to = "it", type = "magical", amount = "$ground_damage", require_source_alive = false },
                    } },
                  } },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:323-334",
            text = "modifier_knockback distance 0、height 600、should_stun：原地挑起 1.5 秒；另外套 2 秒 modifier_stunned，落地後仍暈 0.5 秒。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:356-376",
            text = "落地範圍包含被挑起的目標本人，所以目標一共受到 damage + damage_on_ground；Stop() 對所有範圍內非建築敵人（含魔免）生效。" },
          { policy = "as_source", ref = "npc/A_Oda/A14.txt:549-566",
            text = "落地效果由 KV DelayedAction（flyTime）執行，與目標是否仍被挑起無關；目標死亡或消失時 IsValidEntity 失敗，什麼都不做。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A14.lua:312-317,353-354", text = "雷擊與震地粒子屬呈現。" },
        },
        provenance = { "npc/A_Oda/A14.txt:514-591", "vscripts/heroes/A_Oda/A14.lua:306-378" },
      },
      -------------------------------------------------------------------------
      { id = "a14_old_war_spirit", source_id = "A14R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          attack_speed = { 40, 70, 100, 130, unit = "atk_speed", source = "kv:atkspeed_bonus" },
          move_speed_pct = { 4, 8, 12, 16, unit = "pct", source = "kv:speed_bonus" },
        },
        text = {
          zh_TW = { name = "戰魂", body = "被動：攻擊速度提高 {attack_speed}，移動速度提高 {move_speed_pct}。" },
          en = { name = "War Spirit", body = "Passive: +{attack_speed} attack speed and +{move_speed_pct} movement speed." },
        },
        target = { type = "none" },
        passive = { reveal = "public", stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct" } },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/A_Oda/A14.txt:600-601",
            text = "KV 寫了 AbilityUnitDamageType PHYSICAL 與 SpellImmunityType，但本招沒有傷害；IsPurgable 0。" },
        },
        provenance = { "npc/A_Oda/A14.txt:594-631" },
      },
      -------------------------------------------------------------------------
      { id = "a14_old_fire_phoenix", source_id = "A14D_old", slot = "D", kind = "active", max_level = 3, learnable = false,
        tags = { "basic" },
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 2000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 350, 450, 550, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          speed = { 2000, unit = "speed", hidden = true, source = "kv:shock_speed" },
          width = { 200, unit = "dist", source = "kv:shock_width" },
          distance = { 2000, unit = "dist", source = "kv:shock_distance" },
          vision = { 200, unit = "dist", hidden = true, source = "kv:shock_vision" },
        },
        text = {
          zh_TW = { name = "火鳳凰",
            body = "只在{@a14_old_rage_spear}期間可用，等級與其相同。朝目標方向放出寬 {width}、飛行 {distance} 的火鳳凰衝擊波，對路徑上所有敵方英雄與一般單位造成 {damage} 魔法傷害。" },
          en = { name = "Fire Phoenix",
            body = "Usable only during {@a14_old_rage_spear}, at its level. Releases a fire phoenix shockwave {width} wide that travels {distance} toward the target, dealing {damage} magic damage to every enemy hero and unit in its path." },
        },
        target = { type = "unit_or_point", range = "$cast_range", team = "enemy", types = UNITS },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "a14d_old_vo", at = "caster" },
            { op = "cue", id = "a14d_old_shockwave_cast", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)", speed = "$speed",
              distance = "$distance", radius = "$width", team = "enemy", types = UNITS, pierce = true, vision = "$vision",
              cue = "a14d_old_phoenix",
              on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "cue", id = "a14d_old_shockwave_target", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:388-389,419",
            text = "NOT_LEARNABLE：出生等級 0 無法施放；每次施放大招時 SetLevel(大招等級) 並 SetActivated(true)，大招狀態結束時 SetActivated(false)（等級保留）。資料以大招的 set_ability_level／set_ability_active 表達。" },
          { policy = "as_source", ref = "npc/A_Oda/A14.txt:833-848",
            text = "datadriven LinearProjectile 朝目標點（單位目標時取其位置）、TargetFlags NONE：不打魔免；DeleteOnHit 0，穿透全部。" },
          { policy = "note", ref = "npc/A_Oda/A14.txt:803", text = "HotKeyOverride W 只影響熱鍵，槽位照本體慣例放 D。" },
        },
        provenance = { "npc/A_Oda/A14.txt:793-889", "vscripts/heroes/A_Oda/A14.lua:382-428" },
      },
      -------------------------------------------------------------------------
      { id = "a14_old_rage_spear", source_id = "A14T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 108, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel", note = "RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          duration = { 40, unit = "s", source = "kv:duration" },
          move_speed = { 60, unit = "move_speed", source = "kv:bonus_movement_speed" },
          attack_bonus = { 200, 260, 320, unit = "dmg", source = "kv:atk_bonus", note = "MODIFIER_PROPERTY_BASEATTACK_BONUSDAMAGE" },
          hp_regen = { 90, 110, 130, unit = "per_s", source = "kv:heal_bonus" },
          debuff_duration = { 4, 5, 6, unit = "s", source = "kv:debuff_duration" },
          debuff_pct = { 50, 60, 70, unit = "pct", source = "kv:debuff_decrease_speed", kv = { -50, -60, -70 },
            source_differs = "KV 以負值記減益；行為裡取負號" },
          roar_radius = { 700, unit = "dist", source = "script:vscripts/heroes/A_Oda/A14.lua:391,421 FindUnitsInRadius 700" },
          reapply_delay = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A14.lua:400-401 tsum = 0.1、Timers 0.1" },
        },
        text = {
          zh_TW = { name = "狂怒．槍之又左",
            body = "進入狂怒 {duration} 秒：移動速度增加 {move_speed}、基礎攻擊力增加 {attack_bonus}、每秒回復 {hp_regen} 生命，並可使用{@a14_old_fire_phoenix}。狂怒開始與結束時各怒吼一次，範圍 {roar_radius} 內的敵方英雄與一般單位（包含魔免）造成的傷害與移動速度都降低 {debuff_pct}，持續 {debuff_duration} 秒；不受狀態抗性影響。" },
          en = { name = "Fury: Spear of Matazaemon",
            body = "Enters a fury for {duration} seconds: +{move_speed} movement speed, +{attack_bonus} base attack damage, {hp_regen} health regeneration per second, and access to {@a14_old_fire_phoenix}. When the fury starts and when it ends, he roars: enemy heroes and units within {roar_radius} (spell-immune ones included) deal {debuff_pct} less damage and move {debuff_pct} slower for {debuff_duration} seconds, unaffected by status resistance." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "a14t_old_rage", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "狂怒．槍之又左", body = "移動速度 +{move_speed}、基礎攻擊力 +{attack_bonus}、每秒回復 {hp_regen} 生命，可使用{@a14_old_fire_phoenix}。結束時再怒吼一次。" },
              en = { name = "Fury: Spear of Matazaemon", body = "+{move_speed} movement speed, +{attack_bonus} base attack damage, {hp_regen} health regeneration per second, and {@a14_old_fire_phoenix} is usable. Roars again when it ends." },
            },
            reveal = "public", cue = "a14t_old_aura",
            stats = { move_speed = "$move_speed", base_attack_damage = "$attack_bonus", hp_regen = "$hp_regen" },
            triggers = {
              on_removed = {
                { op = "cue", id = "a14t_old_roar", at = "self" },
                { op = "set_ability_active", of = "self", ability = "a14_old_fire_phoenix", active = false },
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$roar_radius", team = "enemy",
                    types = UNITS, flags = { "include_magic_immune" } },
                  actions = { { op = "apply_status", status = "a14t_old_debuff", to = "it", source = "it" } } },
              },
            },
            notes = "原作 modifier_A14T_old（IsPurgable 0、AllowIllusionDuplicate）；OnDestroy 執行 modifier_A14T_old_OnDestroy。" },
          { id = "a14t_old_debuff", duration = "$debuff_duration", stacking = "independent", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "怒吼", body = "造成的傷害與移動速度降低 {debuff_pct}，持續 {debuff_duration} 秒。" },
              en = { name = "Roar", body = "Deals {debuff_pct} less damage and moves {debuff_pct} slower for {debuff_duration} seconds." },
            },
            reveal = "public", status_resistance = "ignore", cue = "a14t_old_move_slow",
            stats = { outgoing_damage_pct = "-$debuff_pct", move_speed_pct = "-$debuff_pct" },
            notes = "原作 modifier_A14T_old_debuff（IsPurgable 1、MODIFIER_ATTRIBUTE_MULTIPLE）：以單位自己為施加者，不受狀態抗性縮短；多個實例的屬性各自疊加。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a14t_old_berserker_sound", at = "caster" },
            { op = "cue", id = "a14t_title", at = "caster" },
            { op = "apply_status", status = "a14t_old_rage", to = "caster" },
            { op = "cue", id = "a14t_old_roar", at = "caster" },
            { op = "set_ability_level", of = "caster", ability = "a14_old_fire_phoenix", level = "ability.level" },
            { op = "set_ability_active", of = "caster", ability = "a14_old_fire_phoenix", active = true },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$roar_radius", team = "enemy",
                types = UNITS, flags = { "include_magic_immune" } },
              actions = {
                { op = "apply_status", status = "a14t_old_debuff", to = "it", source = "it" },
                { op = "if", cond = "it.is_hero", actions = {
                  { op = "let", name = "u", value = "it" },
                  { op = "delay", time = "$reapply_delay", capture = { "u" }, actions = {
                    { op = "if", cond = "exists(u) and not has_status(u, a14t_old_debuff)", actions = {
                      { op = "apply_status", status = "a14t_old_debuff", to = "u", source = "u",
                        duration = "$debuff_duration - $reapply_delay" },
                    } },
                  } },
                } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "no_effect", ref = "npc/A_Oda/A14.txt:685-690",
            text = "OnSpellStart 呼叫不存在的 A14T_old_OnSpellStart，報錯沒有效果；之後的 RunScript modifier_A14T_old_OnCreated 照常執行（推定 RunScript 錯誤不中斷後續 KV 動作）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:399-410",
            text = "0.1 秒後只對一開始選到的英雄檢查一次：身上沒有減益（例如被驅散）就補一個 duration − 0.1 的；計時器只跑一次（沒有 return）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A14.lua:400,405",
            text = "tsum 是未加 local 的全域變數（與 B30、B27 共用的 match 層 tsum 同名）；這 0.1 秒內若被其他腳本改寫，補套時間會不同。資料以固定 0.1 表達，不登錄共用 unit_state。" },
          { policy = "as_source", ref = "npc/A_Oda/A14.txt:717-725",
            text = "OnDestroy 在狀態到期、死亡移除時都會觸發：停用火鳳凰，並在利家當下位置再怒吼一次。" },
          { policy = "as_source", ref = "npc/A_Oda/A14.txt:742",
            text = "減益是 MODIFIER_ATTRIBUTE_MULTIPLE：開始與結束兩次怒吼若都命中同一目標，兩個實例的 −傷害、−移速各自疊加（stacking = independent）。" },
          { policy = "as_source", ref = "npc/A_Oda/A14.txt:739-740",
            text = "移速減益用 MOVESPEED_BONUS_PERCENTAGE，不經 nobu 緩速池（沒有 tag slow）。" },
          { policy = "note", ref = "npc/A_Oda/A14.txt:702-707", text = "showTitle、b34t 粒子與 shake 屬呈現。" },
        },
        provenance = { "npc/A_Oda/A14.txt:633-789", "vscripts/heroes/A_Oda/A14.lua:382-428" },
      },
    },
  }
end
