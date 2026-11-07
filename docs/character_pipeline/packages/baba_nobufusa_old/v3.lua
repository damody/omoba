-- B21 馬場信房（舊版，11.2B 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["B21"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:300）；自動學習 heromap_autoskill["11"] = ""
-- （addon_game_mode.lua:403），沒有 D。W/E/R/T → Q/W/E/R。
-- 四招都是舊版獨有的實作（B21W_old／B21E_old／B21R_old／B21T_old，npc/B_Unified/B21.txt:399-790、vscripts/heroes/B_Unified/B21.lua:210-399）；
-- 只有 B21T_old 的 OnUpgrade 和本體共用 B21T_OnUpgrade。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 B21*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、B21_precache 屬規則層或資產，不列入。
-- 慣例照本體 B21：HERO | BASIC 寫成 { "hero", "creep", "neutral", "summon" }；KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"，
-- 引擎內建 modifier_stunned 寫 "strong"。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B21_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B21", text = { zh_TW = { name = "馬場信房（舊版）" }, en = { name = "Baba Nobufusa (Old)" } } },
    unit_state = {
      b21_old_next_hp = { type = "fixed", init = "-1", reveal = "team", scope = "hero",
        note = "原作 caster.nexthp（B21.lua:324,327,338-341,376）：不死鬼美濃發動中，生命不能被打到低於這個值；原作初值 nil、解除時設 -10，資料以 -1 表示「不限制」（生命不會低於 0，結果相同）" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b21_old_triple_wave", source_id = "B21W_old", slot = "Q", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B21.txt:411" },
          mana_cost = { 140, 180, 220, 260, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B21.txt:412" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B21.txt:415",
            note = "腳本 fDistance = ability:GetCastRange()" },
          cast_point = { 0, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B21.txt:416" },
          channel_time = { 1, unit = "s", source = "kv:AbilityChannelTime@npc/B_Unified/B21.txt:419" },
          damage = { 150, 200, 250, 300, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/B_Unified/B21.txt:413" },
          speed = { 2500, unit = "speed", source = "kv:projectile_speed@npc/B_Unified/B21.txt:426" },
          radius = { 300, unit = "dist", source = "script:vscripts/heroes/B_Unified/B21.lua:230-231 fStartRadius/fEndRadius" },
          wave_interval = { 0.4, unit = "s", source = "script:vscripts/heroes/B_Unified/B21.lua:248 fire_delay" },
          wave_slots = { 4, unit = "count", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B21.lua:247,250 fire_num = 3，for i=0,fire_num 共四個 Timer" },
          waves = { expr = "min($wave_slots, floor($channel_time / $wave_interval) + 1)", unit = "count",
            note = "實際發出的道數：第四個 Timer 在 1.2 秒，引導已結束" },
        },
        text = {
          zh_TW = { name = "波動三連斬",
            body = "引導：立即並每 {wave_interval} 秒朝指定方向噴出一道火焰波（引導完整時共 {waves} 道），寬 {radius}、以速度 {speed} 飛行至施法距離，對沿途每個敵方英雄與一般單位各造成 {damage} 魔法傷害；同一個敵人可以被每一道各命中一次。引導中斷就不再發出。" },
          en = { name = "Triple Wave Slash",
            body = "Channel: immediately and every {wave_interval} seconds, breathes a wave of fire in the target direction ({waves} waves over a full channel), {radius} wide and travelling the full cast range at speed {speed}, dealing {damage} magic damage to each enemy hero and unit in its path; an enemy can be hit once by each wave. Stops if the channel is interrupted." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_phase_start = { { op = "cue", id = "b21w_old_voice", at = "caster" } },
          on_cast = {
            { op = "let", name = "origin", value = "caster.position" },
            { op = "let", name = "dir", value = "select(distance(caster, point) > 0, direction(caster, point), caster.forward)" },
            { op = "repeat", interval = "$wave_interval", first = "0", count = "$wave_slots", ["while"] = "caster.is_channeling",
              actions = {
                { op = "cue", id = "b21w_old_breathe_fire_sound", at = "caster" },
                { op = "projectile", kind = "linear", from = "origin", direction = "dir",
                  speed = "$speed", distance = "$cast_range", radius = "$radius", end_radius = "$radius",
                  pierce = true, team = "enemy", types = BASIC, cue = "b21w_old_fire_wave",
                  on_hit = {
                    { op = "cue", id = "b21w_old_impact", at = "it" },
                    { op = "damage", to = "it", type = "magical", amount = "$damage" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:246-257",
            text = "迴圈 for i=0,fire_num（3）建立四個 Timer，分別在 0、0.4、0.8、1.2 秒，各自檢查 IsChanneling 才發射；引導 1 秒，所以完整引導只發三道（招式名「三連斬」），第四個永遠不發。channel_time - 0.05 算了沒用。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:225-244",
            text = "投射物參數在施放當下建立一次：每道都從施放時的位置、朝施放時的方向發射，飛行距離固定是施法距離，與點選遠近無關；點在自己身上時沿面向。DeleteOnHit 0（穿透）；目標取技能 KV：敵方 HERO|BASIC、沒有 TargetFlags，魔免敵人不被命中。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:272-278",
            text = "傷害用 ApplyDamage（攻擊者為馬場），馬場在波飛行中死亡時由規則層歸零（require_source_alive 預設）。" },
          { policy = "note", ref = "npc/B_Unified/B21.txt:406",
            text = "AbilityBehavior DIRECTIONAL | POINT | CHANNELLED：DIRECTIONAL 只影響指示器。bProvidesVision false；fExpireTime 5 秒長於實際飛行時間，不影響結算。命中特效方向存在 ability.dir，只是呈現。" },
        },
        provenance = { "npc/B_Unified/B21.txt:399-467", "vscripts/heroes/B_Unified/B21.lua:212-279" },
      },
      -------------------------------------------------------------------------
      { id = "b21_old_killing_intent", source_id = "B21E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B21.txt:479" },
          mana_cost = { 110, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B21.txt:480" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B21.txt:481" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B21.txt:482" },
          stun_time = { 1, unit = "s", source = "kv:stun_time@npc/B_Unified/B21.txt:514" },
          hp_threshold = { 50, unit = "pct", source = "kv:hp_threshold@npc/B_Unified/B21.txt:494" },
          buff_duration = { 10, unit = "s", source = "kv:buff_duration@npc/B_Unified/B21.txt:489" },
          move_speed_pct = { 30, 40, 50, 60, unit = "pct", source = "kv:bonus_movement_speed@npc/B_Unified/B21.txt:499" },
          attack_speed = { 30, 40, 50, 60, unit = "atk_speed", source = "kv:bonus_attack_speed@npc/B_Unified/B21.txt:504" },
          base_damage_pct = { 30, 40, 50, 60, unit = "pct", source = "kv:bonus_attack@npc/B_Unified/B21.txt:509",
            note = "MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE：基礎攻擊力百分比" },
        },
        text = {
          zh_TW = { name = "殺意",
            body = "以殺意震懾一名敵方英雄或單位，暈眩 {stun_time} 秒並立刻對它發動攻擊。若目標的生命不高於 {hp_threshold}，信房在 {buff_duration} 秒內移動速度提高 {move_speed_pct}、攻擊速度提高 {attack_speed}、基礎攻擊力提高 {base_damage_pct}。" },
          en = { name = "Killing Intent",
            body = "Terrifies an enemy hero or unit with killing intent, stunning it for {stun_time} seconds and immediately attacking it. If the target is at or below {hp_threshold} health, Nobufusa gains +{move_speed_pct} movement speed, +{attack_speed} attack speed and +{base_damage_pct} base attack damage for {buff_duration} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b21e_old_stun", duration = "$stun_time", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被{@b21_old_killing_intent}震懾，暈眩 {stun_time} 秒。" },
              en = { name = "Stunned", body = "Terrified by {@b21_old_killing_intent}; stunned for {stun_time} seconds." },
            },
            reveal = "public", states = { "stun" }, notes = "原作以 ApplyDataDrivenModifier 套引擎內建 modifier_stunned；受狀態抗性縮短。" },
          { id = "b21e_old_buff", duration = "$buff_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b21e_old_black_flame",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct", base_attack_damage_pct = "$base_damage_pct" },
            text = {
              zh_TW = { name = "殺意", body = "移動速度提高 {move_speed_pct}、攻擊速度提高 {attack_speed}、基礎攻擊力提高 {base_damage_pct}。" },
              en = { name = "Killing Intent", body = "+{move_speed_pct} movement speed, +{attack_speed} attack speed and +{base_damage_pct} base attack damage." },
            },
            notes = "原作 modifier_B21E_old_buff：IsBuff，沒寫 IsPurgable（推定 basic）。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "b21e_old_voice", at = "caster" } },
          on_cast = {
            { op = "apply_status", status = "b21e_old_stun", to = "target" },
            { op = "cue", id = "b21e_old_scream", at = "target" },
            { op = "if", cond = "target.hp_pct <= $hp_threshold", actions = {
              { op = "apply_status", status = "b21e_old_buff", to = "caster" },
            } },
            { op = "issue_order", who = "caster", order = "attack", target = "target" },
            { op = "cue", id = "b21e_old_chaos_strike_sound", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B21.txt:468-573",
            text = "和本體完全不同：舊版是單體指向，暈眩目標並下達攻擊命令；只有目標生命不高於 50% 時才給自己加成，加成多了基礎攻擊力百分比。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B21.lua:293",
            text = "GetHealthPercent 回傳整數百分比；資料以比例 hp_pct 比較，邊界上可能差一格。判定在套上暈眩之後，但暈眩不造成傷害，不影響結果。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:297-302",
            text = "ExecuteOrderFromTable ATTACK_TARGET、Queue false：取代馬場目前的命令，改成攻擊目標。" },
        },
        provenance = { "npc/B_Unified/B21.txt:468-573", "vscripts/heroes/B_Unified/B21.lua:281-305" },
      },
      -------------------------------------------------------------------------
      { id = "b21_old_divine_command", source_id = "B21R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic", "aura" },
        values = {
          armor = { 4, 9, 12, 15, unit = "armor", source = "kv:add_armor@npc/B_Unified/B21.txt:592" },
          radius = { 800, unit = "dist", source = "kv:modifier_B21R_old_aura Aura_Radius@npc/B_Unified/B21.txt:609",
            kv = { 900 }, source_differs = "AoERadius 900（B21.txt:585）只是指示器；光環實際半徑取 Aura_Radius 800" },
          aura_linger = { 0.5, unit = "s", hidden = true, source = "assumed:Dota datadriven 光環預設殘留 0.5 秒" },
        },
        text = {
          zh_TW = { name = "領軍如神",
            body = "被動：信房自己護甲提高 {armor}；周圍 {radius} 內的友方英雄與單位也獲得 {armor} 護甲。" },
          en = { name = "Divine Command",
            body = "Passive: Nobufusa gains +{armor} armor, and allied heroes and units within {radius} also gain +{armor} armor." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        passive = {
          reveal = "public", visibility = "buff",
          stats = { armor = "$armor" },
          aura = { radius = "$radius", team = "ally", types = BASIC, status = "b21r_old_buff", linger = "$aura_linger", include_self = false },
        },
        statuses = {
          { id = "b21r_old_buff", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", cue = "b21r_old_buff_ring", stats = { armor = "$armor" },
            text = {
              zh_TW = { name = "領軍如神", body = "護甲 +{armor}。" },
              en = { name = "Divine Command", body = "+{armor} armor." },
            },
            notes = "原作 modifier_B21R_old_buff：光環給的 modifier，沒寫 IsPurgable；光環 modifier 每次刷新都會重新給，寫 none。" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B21.txt:603-623",
            text = "光環 modifier_B21R_old_aura 本身帶 PHYSICAL_ARMOR_BONUS，Aura_ApplyToCaster 0：信房的護甲來自光環 modifier 本身，隊友的來自 modifier_B21R_old_buff，數值相同、不重複。" },
          { policy = "note", ref = "npc/B_Unified/B21.txt:612",
            text = "Aura_Flags 寫 MAGIC_IMMUNE_ENEMIES，對友方光環沒有意義（魔免隊友本來就會被選到）。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B21.lua:307-317",
            text = "OnCreated／OnDestroy 只建立與移除隊友腳下的光圈特效，改用 cue。" },
        },
        provenance = { "npc/B_Unified/B21.txt:574-653", "vscripts/heroes/B_Unified/B21.lua:307-317" },
      },
      -------------------------------------------------------------------------
      { id = "b21_old_undying_demon", source_id = "B21T_old", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          str = { 22, 33, 44, unit = "stat", source = "kv:add_str@npc/B_Unified/B21.txt:668" },
          hp_threshold = { 1600, unit = "hp", source = "kv:hp_threshold@npc/B_Unified/B21.txt:683" },
          attack = { 160, 280, 400, unit = "dmg", source = "kv:add_attack@npc/B_Unified/B21.txt:673" },
          armor = { 30, 50, 70, unit = "armor", source = "kv:add_armor@npc/B_Unified/B21.txt:678" },
          physical_cut = { 50, unit = "pct", kv = { -50 },
            source = "kv:modifier_B21T_old INCOMING_PHYSICAL_DAMAGE_PERCENTAGE@npc/B_Unified/B21.txt:759",
            source_differs = "KV 以 -50 表示減傷，v3 記為正值，行為裡取負號" },
          loss_cap = { 800, 700, 600, unit = "hp", source = "kv:hp_threshold2@npc/B_Unified/B21.txt:693" },
          window = { 1, unit = "s", source = "kv:modifier_B21T_old ThinkInterval@npc/B_Unified/B21.txt:762" },
          stun_delay = { 1, unit = "s", source = "script:vscripts/heroes/B_Unified/B21.lua:382 Timers:CreateTimer(1, ...)" },
          stun_radius = { 400, unit = "dist", source = "script:vscripts/heroes/B_Unified/B21.lua:384-386" },
          stun_duration = { 1, unit = "s",
            source = "script:vscripts/heroes/B_Unified/B21.lua:388 {duration=1}（KV modifier_B21T_stun Duration 同為 1，B21.txt:714）" },
        },
        text = {
          zh_TW = { name = "不死鬼美濃",
            body = "被動：力量永久提高 {str}。生命不高於 {hp_threshold} 時化為不死鬼：基礎攻擊力提高 {attack}、護甲提高 {armor}、受到的物理傷害減少 {physical_cut}，而且每 {window} 秒內生命最多只會被打掉 {loss_cap}（剛發動時生命不會被打到 {hp_threshold} 減 {loss_cap} 以下）；生命回到 {hp_threshold} 以上就解除。每次學習或升級此技能，{stun_delay} 秒後暈眩周圍 {stun_radius} 內的敵方英雄與一般單位（包括魔免）{stun_duration} 秒。" },
          en = { name = "Undying Demon of Mino",
            body = "Passive: permanently grants +{str} strength. While at or below {hp_threshold} health he becomes an undying demon: +{attack} base attack damage, +{armor} armor, {physical_cut} less physical damage taken, and he can lose at most {loss_cap} health in each {window}-second window (right after it triggers his health cannot drop below {hp_threshold} minus {loss_cap}); it ends once his health rises above {hp_threshold}. Each time this ability is learned or upgraded, enemy heroes and units within {stun_radius} (including spell-immune ones) are stunned for {stun_duration} seconds after {stun_delay} seconds." },
        },
        target = { type = "none", aoe_radius = "$stun_radius" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public", visibility = "hidden",
          stats = { str = "$str" },
          triggers = {
            on_take_damage = { { op = "call", routine = "b21t_old_update", args = { who = "self" } } },
            on_heal_received = { { op = "call", routine = "b21t_old_update", args = { who = "self" } } },
          },
        },
        statuses = {
          { id = "b21t_old_demon", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b21t_old_aura", remove_on = { "death" },
            stats = { base_attack_damage = "$attack", armor = "$armor", incoming_physical_pct = "-$physical_cut" },
            text = {
              zh_TW = { name = "不死鬼美濃", body = "基礎攻擊力 +{attack}、護甲 +{armor}、受到的物理傷害減少 {physical_cut}；每 {window} 秒內生命最多只會被打掉 {loss_cap}。生命回到 {hp_threshold} 以上就解除。" },
              en = { name = "Undying Demon of Mino", body = "+{attack} base attack damage, +{armor} armor, {physical_cut} less physical damage taken; loses at most {loss_cap} health in each {window}-second window. Ends once health rises above {hp_threshold}." },
            },
            triggers = {
              on_created = {
                { op = "set_state", var = "b21_old_next_hp", on = "self", value = "$hp_threshold - $loss_cap" },
              },
              periodic = { interval = "$window", actions = {
                { op = "set_state", var = "b21_old_next_hp", on = "self", value = "self.hp - $loss_cap" },
                { op = "delay", time = "$window", actions = {
                  { op = "if", cond = "self.hp > $hp_threshold", actions = {
                    { op = "set_state", var = "b21_old_next_hp", on = "self", value = "-1" },
                  } },
                } },
              } },
            },
            notes = "原作 modifier_B21T_old：datadriven、沒有 Duration（infinite）、不是 Passive，死亡時移除；沒寫 IsPurgable（推定 basic），被驅散後下一次生命變化會重新套上（B21.lua:356-359）。ThinkInterval 1。" },
          { id = "b21t_old_stun", duration = "$stun_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被{@b21_old_undying_demon}暈眩 {stun_duration} 秒，不受狀態抗性縮短。" },
              en = { name = "Stunned", body = "Stunned by {@b21_old_undying_demon} for {stun_duration} seconds; not reduced by status resistance." },
            },
            reveal = "public", status_resistance = "ignore", states = { "stun" },
            notes = "原作 B21T_old 自己的 modifier_B21T_stun：IsStunDebuff，沒寫 IsPurgable（推定 basic）。施加者是被暈者自己，不受狀態抗性縮短。" },
        },
        events = {
          on_upgrade = {
            { op = "delay", time = "$stun_delay", actions = {
              { op = "if", cond = "exists(caster) and not caster.is_illusion", actions = {
                { op = "for_each",
                  select = { from = "caster", shape = "circle", radius = "$stun_radius", team = "enemy",
                    types = BASIC, flags = { "include_magic_immune" } },
                  actions = {
                    { op = "apply_status", status = "b21t_old_stun", to = "it", source = "it" },
                  } },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:332-366",
            text = "OnTakeDamage／OnHealthGained 時：若 caster.nexthp 有值、目前生命低於它且不是幻象，SetHealth(nexthp) 把生命拉回；接著以拉回前的生命判斷是否不高於 1600，決定套上或移除 modifier_B21T_old。拉回在傷害結算之後，致死的傷害照樣打死馬場，這不是免死。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:319-330,368-377",
            text = "modifier 建立時 nexthp = 1600 − hp_threshold2；之後每秒 nexthp = 目前生命 − hp_threshold2，一秒後若生命已高於 1600 就設回 -10（不限制）。等於每秒最多損失 hp_threshold2 的生命。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:324-328",
            text = "nexthp 是英雄上的 Lua 欄位，跨死亡保留：死前最後一次 Think 設的值，一秒後檢查時屍體生命 0 不高於 1600，不會被清掉；重生後滿血時，若一次被打到那個值以下（通常是負值或很小，實際罕見）仍會被拉回。資料用 scope = hero 的 unit_state 照實表達。" },
          { policy = "as_source", ref = "npc/B_Unified/B21.txt:757-760",
            text = "和本體不同：舊版門檻是固定生命 1600（不是百分比），加成是固定值不分層；回血 regen（22/33/44）在 KV 被註解掉，沒有效果，不列入。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:379-393",
            text = "OnUpgrade 和本體共用 B21T_OnUpgrade：每次學習、升級都觸發，1 秒後暈眩 400 內敵方 HERO|BASIC（含魔免）1 秒；ApplyDataDrivenModifier(enemy, enemy, ...) 施加者是被暈者自己，不受狀態抗性縮短。" },
          { policy = "design_intent", ref = "npc/B_Unified/B21.txt:762",
            text = "OnIntervalThink 裡一秒後的清除用 Timer，modifier 被移除後照樣執行；資料寫在 periodic 的 delay 內，推定狀態移除後 delay 仍會跑完（若執行器隨狀態取消，差別只在生命剛回到 1600 以上的一秒內）。" },
          { policy = "note", ref = "npc/B_Unified/B21.txt:732",
            text = "modifier_B21T_old_passive 是 Passive、IsHidden：力量加成看不到圖示。" },
        },
        provenance = { "npc/B_Unified/B21.txt:654-790", "vscripts/heroes/B_Unified/B21.lua:319-399" },
      },
    },
    routines = {
      -- 原作 B21T_old_OnHealthChange
      b21t_old_update = {
        params = { who = "unit" },
        actions = {
          { op = "let", name = "hp", value = "who.hp" },
          { op = "if", cond = "hp < state.b21_old_next_hp and not who.is_illusion", actions = {
            { op = "set_health", to = "who", value = "state.b21_old_next_hp" },
          } },
          { op = "if", cond = "hp <= $b21_old_undying_demon.hp_threshold",
            actions = {
              { op = "if", cond = "not has_status(who, b21t_old_demon)", actions = {
                { op = "apply_status", status = "b21t_old_demon", to = "who" },
              } },
            },
            otherwise = { { op = "remove_status", from = "who", status = "b21t_old_demon" } } },
        },
      },
    },
  }
end
