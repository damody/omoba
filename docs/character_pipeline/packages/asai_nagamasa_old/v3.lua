-- B08 淺井長政（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["B08"]["11"] = "WERDT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:287）。
-- KV 沒有 B08D_old（npc/B_Unified/B08.txt 只有 B08W_old／B08E_old／B08R_old／B08T_old；B08.txt:74 的 "B08D_old" 是本體 B08D 呼叫的腳本函式名），
-- AddAbility("B08D_old") 失敗，舊版實際只有四招；自動學習 heromap_autoskill["11"] = ""（addon_game_mode.lua:390），
-- eventforpichero.lua:153-160 的 16 級 SetLevel 只找本體 B08D（已被拔掉），所以沒有 D。
-- 舊版 AI 單位（npc_units_custom_load.txt:15795-15799）也是 B08W_old／B08E_old／B08R_old／B08T_old，佐證。
-- W/E/R/T → Q/W/E/R。原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 B08*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、B08_precache 屬規則層或資產，不列入。
-- 慣例照本體 B08：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_phased 寫 "none"；
-- datadriven Damage（不經 AMHC）在長政死亡後仍會造成傷害，寫 require_source_alive = false；HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B08_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B08", text = { zh_TW = { name = "淺井長政（舊版）" }, en = { name = "Asai Nagamasa (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b08_old_fang_slash", source_id = "B08W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 7, 6, 5, 4, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B08.txt:304" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B08.txt:306" },
          cast_range = { 1300, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B08.txt:307" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B08.txt:301" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical",
            source = "kv:AbilityDamage@npc/B_Unified/B08.txt:417" },
          speed = { 2000, unit = "speed", source = "kv:shock_speed@npc/B_Unified/B08.txt:399" },
          radius = { 175, unit = "dist", source = "kv:shock_width@npc/B_Unified/B08.txt:404" },
          distance = { 1400, unit = "dist", source = "kv:shock_distance@npc/B_Unified/B08.txt:409" },
          vision_radius = { 175, unit = "dist", source = "kv:shock_vision@npc/B_Unified/B08.txt:414" },
        },
        text = {
          zh_TW = { name = "一刀．牙",
            body = "朝指定方向斬出刀牙（半徑 {radius}、飛行 {distance}、速度 {speed}），穿透路徑上的敵方英雄與一般單位，各造成 {damage} 魔法傷害一次；刀牙飛行時提供 {vision_radius} 視野。" },
          en = { name = "Fang Slash",
            body = "Slashes a fang wave in the target direction (radius {radius}, travels {distance} at speed {speed}) that pierces enemy heroes and basic units, dealing {damage} magic damage to each once; the wave grants {vision_radius} vision while flying." },
        },
        target = { type = "unit_or_point", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_phase_start = { { op = "cue", id = "b08w_old_blade_glow", at = "caster" } },
          on_cast = {
            { op = "cue", id = "b08w_old_cast_vo", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$speed", distance = "$distance", radius = "$radius", pierce = true, team = "enemy", types = BASIC,
              vision = "$vision_radius", cue = "b08w_old_fang",
              on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "cue", id = "b08w_old_hit_sound", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B08.txt:322-355",
            text = "和本體不同：OnSpellStart 沒有 RunScript B08W，施放後沒有 modifier_B08W 的指令限制；前搖 0.2、施法距離 1300、飛行 1400、速度 2000、耗魔固定 100。" },
          { policy = "as_source", ref = "npc/B_Unified/B08.txt:295",
            text = "AbilityBehavior 是 UNIT_TARGET | POINT：可以點單位或點地，LinearProjectile Target POINT 都朝施放點（點單位時是單位位置）發射，固定飛行 shock_distance。" },
          { policy = "as_source", ref = "npc/B_Unified/B08.txt:338-364",
            text = "StartRadius = EndRadius = 175 是半徑；TargetFlags NONE：魔免敵人不被命中，隱形與迷霧中的敵人照樣命中。DeleteOnHit 0 穿透。命中是 datadriven Damage（不經 AMHC），長政在刀牙飛行中死亡仍照常傷害。" },
          { policy = "note", ref = "npc/B_Unified/B08.txt:313-319,371-392",
            text = "前搖的 modifier_shockwave_hidden_cast（0.5 秒、隱藏）只掛刀光粒子；三個 FireSound 只是呈現。LevelsBetweenUpgrades 2 對一般技能沒有作用。" },
        },
        provenance = { "npc/B_Unified/B08.txt:289-418" },
      },
      -------------------------------------------------------------------------
      { id = "b08_old_flash_cut", source_id = "B08E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 12, 11, 10, 9, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B08.txt:547" },
          mana_cost = { 120, 130, 140, 150, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B08.txt:548" },
          cast_range = { 1150, 1200, 1250, 1300, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B08.txt:549" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B08.txt:545" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical",
            source = "kv:B08E_Damage@npc/B_Unified/B08.txt:615（AbilityDamage@B08.txt:550 同值，腳本讀 B08E_Damage）" },
          phase_time = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:180-181 modifier_phased 0.1" },
        },
        text = {
          zh_TW = { name = "一刀．閃瞬",
            body = "瞬移到指定敵人所在位置並斬擊，造成 {damage} 魔法傷害。可以對魔法免疫的敵人施放，但只瞬移、不造成傷害。" },
          en = { name = "Flash Cut",
            body = "Blinks to the target enemy and strikes, dealing {damage} magic damage. Can be cast on spell-immune enemies, but then only blinks without dealing damage." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b08e_old_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" },
            notes = "引擎 modifier_phased（AddNewModifier(nil, nil, ...)），長政與目標各套一份，避免重疊卡位。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "b08e_old_blade_glow", at = "caster" } },
          on_cast = {
            { op = "cue", id = "b08e_old_cast_vo", at = "caster" },
            { op = "apply_status", status = "b08e_old_phased", to = "caster" },
            { op = "apply_status", status = "b08e_old_phased", to = "target" },
            { op = "teleport", who = "caster", to = "position(target)", find_clear_space = false },
            { op = "if", cond = "not target.is_magic_immune", actions = {
              { op = "damage", to = "target", type = "magical", amount = "$damage" },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:173-187",
            text = "B08E_Action2 只有：長政與目標各 phased 0.1 秒 → SetAbsOrigin 站到目標位置 → 目標不是魔免時 AMHC 魔法傷害。和本體不同：沒有 ProjectileDodge、暈眩、燒魔、燒魔追加傷害與 modifier_B08E2 指令限制。" },
          { policy = "as_source", ref = "npc/B_Unified/B08.txt:538-539",
            text = "AbilityUnitTargetFlags MAGIC_IMMUNE_ENEMIES：可以點魔免敵人（瞬移照常，傷害因腳本檢查而跳過）；沒有 FOW_VISIBLE。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:185",
            text = "傷害經 AMHC:Damage：長政已死亡時不造成傷害（規則層）。" },
          { policy = "note", ref = "npc/B_Unified/B08.txt:578-603",
            text = "前搖 modifier_B08E 只掛刀光粒子；兩個 FireSound 只是呈現。" },
        },
        provenance = { "npc/B_Unified/B08.txt:530-618", "vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:173-187",
          "vscripts/amhc_library/amhc.lua:535-584" },
      },
      -------------------------------------------------------------------------
      { id = "b08_old_living_soul", source_id = "B08R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          attack_speed = { 15, 30, 45, 60, unit = "atk_speed", source = "kv:b08r_attack_speed@npc/B_Unified/B08.txt:735" },
          move_speed = { 4, 6, 8, 10, unit = "pct", source = "kv:b08r_run_speed@npc/B_Unified/B08.txt:740" },
          buff_attack_speed = { 25, unit = "atk_speed",
            source = "kv:modifier_B08R_buff MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT@npc/B_Unified/B08.txt:715" },
          buff_damage = { 5, unit = "pct",
            source = "kv:modifier_B08R_buff MODIFIER_PROPERTY_DAMAGEOUTGOING_PERCENTAGE@npc/B_Unified/B08.txt:716" },
          duration = { 8, unit = "s", source = "kv:modifier_B08R_buff Duration@npc/B_Unified/B08.txt:718" },
          max_stacks = { 4, 5, 6, 7, unit = "count",
            source = "script:vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:260 上限 3+level" },
        },
        text = {
          zh_TW = { name = "活心．魂",
            body = "被動：攻擊速度 +{attack_speed}，移動速度 +{move_speed}。每次使用技能或物品後 {duration} 秒內攻擊速度再 +{buff_attack_speed}、造成的傷害提高 {buff_damage}；持續期間再次使用只刷新時間，層數最多 {max_stacks}，只計數、不疊加數值。切換技與伐木、回城、武士服不觸發。" },
          en = { name = "Living Soul",
            body = "Passive: +{attack_speed} attack speed and +{move_speed} movement speed. After each ability or item use, gains another +{buff_attack_speed} attack speed and deals {buff_damage} more damage for {duration} seconds; using another refreshes the duration. Stacks count up to {max_stacks} but do not add more bonus. Toggles, logging, teleport scrolls and the samurai cloth do not trigger it." },
        },
        target = { type = "none" },
        statuses = {
          { id = "b08r_old_buff", text = { zh_TW = { name = "活心．魂", body = "攻擊速度 +{buff_attack_speed}，造成的傷害提高 {buff_damage}。層數只計數，不疊加數值。" }, en = { name = "Living Soul", body = "+{buff_attack_speed} attack speed and deals {buff_damage} more damage. Stacks only count; the bonus does not add up." } },
            duration = "$duration", stacking = { max = "$max_stacks" }, dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b08r_old_soul_shield",
            stats = { attack_speed = "$buff_attack_speed", outgoing_damage_pct = "$buff_damage" },
            notes = "原作 modifier_B08R_buff（B08R_old 版）：ATTACKSPEED_BONUS_CONSTANT 25、DAMAGEOUTGOING_PERCENTAGE 5（固定值，不乘層數）；沒寫 IsPurgable，暫定 basic。OnDestroy 只清粒子。" },
        },
        passive = {
          reveal = "public",
          stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed" },
          triggers = {
            on_ability_executed = { who = "self",
              filter = "not has_tag(event.ability, toggle) and not is_item(event.ability, item_logging) and not is_item(event.ability, item_tpscroll) and not is_item(event.ability, item_samurai_cloth)",
              actions = {
                { op = "if", cond = "has_status(self, b08r_old_buff)",
                  actions = {
                    { op = "add_stacks", status = "b08r_old_buff", to = "self", count = "1", max = "$max_stacks", refresh = true },
                  },
                  otherwise = {
                    { op = "apply_status", status = "b08r_old_buff", to = "self", stacks = "1" },
                    -- FC-005（D13）：原作 B08R_SE 讀不存在的 damage_absorb，把長政身上共用的 AphoticShieldRemaining 清零；照本體修正後不碰其他護盾
                    { op = "cue", id = "b08r_old_soul_shield_burst", at = "self" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B08.txt:704-717",
            text = "和本體不同：被動多了移動速度百分比（b08r_run_speed）；增益是攻速 +25 與 DAMAGEOUTGOING_PERCENTAGE +5，不是本體的攻擊力 +15 與減傷 4%。被動 modifier_B08R 沒寫 IsHidden（本體是隱藏），只是顯示差異。" },
          { policy = "as_source", ref = "npc/B_Unified/B08.txt:730-742",
            text = "AbilitySpecial 兩個條目都用鍵名 \"01\"；KV 保留重複鍵，GetSpecialValueFor 依變數名稱查找，兩個數值都照寫入值生效（推定，見 assumptions.md）。" },
          { policy = "as_source", ref = "npc/B_Unified/B08.txt:714-717",
            text = "modifier_B08R_buff 的 Properties 是固定值，不乘 StackCount；層數只顯示。" },
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:264,268",
            text = "FC-135（D13 修正，照本體：每次觸發都刷新增益的持續時間）。原作：264、268 行引用未定義的 hCaster，該處的刷新不會發生。" },
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:194-198",
            text = "FC-005（D13 修正，照本體：只處理自己的增益，不清空長政身上其他人給的護盾）。原作：B08R_old 也沒有 damage_absorb，B08R_SE 得 0 並把 unit.AphoticShieldRemaining 設為零。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:250-278",
            text = "與本體共用 B08R_Action：對技能與物品都觸發，只排除切換技與 item_logging、item_tpscroll、item_samurai_cloth；上限 3+等級，數值不乘層數。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:200-247",
            text = "B08R_SE 的三個護盾粒子與 EndB08R_SEParticle 的音效只是呈現。" },
        },
        provenance = { "npc/B_Unified/B08.txt:680-743", "vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:190-278" },
      },
      -------------------------------------------------------------------------
      { id = "b08_old_sky_fang", source_id = "B08T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 110, 100, 90, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B08.txt:884" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B08.txt:900" },
          cast_range = { 10000, unit = "dist",
            source = "design:global_range 原作 B08T_old 沒寫 AbilityCastRange（Dota 預設 0，點地不受施法距離限制），取 content 上限涵蓋整張地圖（同 oichi）" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B08.txt:895" },
          required_level = { 6, 12, 18, unit = "level",
            source = "kv:RequiredLevel@npc/B_Unified/B08.txt:893", note = "RequiredLevel 6、LevelsBetweenUpgrades 6 → 6/12/18" },
          damage = { 900, 1300, 1800, unit = "dmg", damage_type = "magical",
            source = "kv:AbilityDamage@npc/B_Unified/B08.txt:1004" },
          speed = { 3000, unit = "speed", source = "kv:dragon_slave_speed@npc/B_Unified/B08.txt:984" },
          radius = { 500, unit = "dist", source = "kv:dragon_slave_width_initial@npc/B_Unified/B08.txt:989" },
          end_width = { 500, unit = "dist", hidden = true, source = "kv:dragon_slave_width_end@npc/B_Unified/B08.txt:994" },
          distance = { 30000, unit = "dist", source = "kv:dragon_slave_distance@npc/B_Unified/B08.txt:999" },
        },
        text = {
          zh_TW = { name = "一刀．空牙",
            body = "朝指定方向斬出巨大刀牙（半徑 {radius}、飛行 {distance}、速度 {speed}），穿透路徑上的敵方英雄與一般單位，各造成 {damage} 魔法傷害一次。" },
          en = { name = "Sky Fang",
            body = "Slashes a huge fang wave in the target direction (radius {radius}, travels {distance} at speed {speed}) that pierces enemy heroes and basic units, dealing {damage} magic damage to each once." },
        },
        target = { type = "unit_or_point", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        events = {
          on_cast = {
            { op = "cue", id = "b08t_old_cast_vo", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$speed", distance = "$distance", radius = "$radius", end_radius = "$end_width", pierce = true,
              team = "enemy", types = BASIC, cue = "b08t_old_sky_fang",
              on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            { op = "cue", id = "b08t_old_blade_glow", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B08.txt:999",
            text = "dragon_slave_distance 是 30000（本體 3000），刀牙以速度 3000 飛約十秒、橫越整張地圖；看起來像多打一個零，但就是原作數值。" },
          { policy = "as_source", ref = "npc/B_Unified/B08.txt:878-900",
            text = "沒有 AbilityCastRange（Dota 預設 0，點地不受施法距離限制，見 assumptions.md）；AbilityBehavior 多了 UNIT_TARGET，可以點單位。也沒有 AbilityType ULTIMATE（技能類型是一般），但 RequiredLevel 6 與 LevelsBetweenUpgrades 6 仍限制在 6/12/18 級學習，槽位照 T 鍵放 R。" },
          { policy = "as_source", ref = "npc/B_Unified/B08.txt:917-952",
            text = "命中是 datadriven Damage（不經 AMHC），長政死亡後刀牙仍會傷害。TargetFlags NONE：魔免敵人不被命中。ProvidesVision 0，不給視野。HasFrontalCone 1 在起訖半徑相同時沒有影響。" },
          { policy = "note", ref = "npc/B_Unified/B08.txt:937-976",
            text = "施放後自身 modifier_B08T（0.5 秒、隱藏）只掛刀光粒子，沒有狀態；FireSound 只是呈現。" },
        },
        provenance = { "npc/B_Unified/B08.txt:877-1005" },
      },
    },
  }
end
