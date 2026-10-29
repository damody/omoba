-- B21 馬場信房（current 版，heromap_version["16"]，heromap_autoskill ["16"] = "WERT"，沒有 D）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R。B21W_old 等 11.2B 版技能不屬於 current 版，不列入。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B21_precache 只是資產，都不列入。
-- 慣例：原作 HERO | BASIC 寫成 types = { "hero", "creep", "neutral", "summon" }（同 C01、A01）；
-- KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"，引擎內建 modifier_stunned 寫 "strong"（同 A05）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "B21",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "B21",
      text = { zh_TW = { name = "馬場信房" }, en = { name = "Baba Nobufusa" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b21_sky_wave", source_id = "B21W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 105, 125, 145, 165, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1500, unit = "dist", source = "kv:AbilityCastRange", note = "腳本 fDistance = ability:GetCastRange()" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 250, 350, 450, 550, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          speed = { 2000, unit = "speed", source = "kv:projectile_speed" },
          radius = { 200, unit = "dist", source = "script:vscripts/heroes/B_Unified/B21.lua:21-22 fStartRadius/fEndRadius" },
        },
        text = {
          zh_TW = { name = "破空波",
            body = "朝指定方向發出一道破空波，以速度 {speed} 飛行至施法距離，寬 {radius}，對沿途所有敵方英雄與一般單位造成 {damage} 魔法傷害。" },
          en = { name = "Sky-Splitting Wave",
            body = "Sends a wave in the target direction that travels to the full cast range at speed {speed} with a width of {radius}, dealing {damage} magic damage to every enemy hero and unit in its path." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_phase_start = { { op = "cue", id = "b21w_voice", at = "caster" } },
          on_cast = {
            { op = "cue", id = "b21w_breathe_fire_sound", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster",
              direction = "select(distance(caster, point) > 0, direction(caster, point), caster.forward)",
              speed = "$speed", distance = "$cast_range", radius = "$radius", end_radius = "$radius",
              pierce = true, team = "enemy", types = { "hero", "creep", "neutral", "summon" }, cue = "b21w_wave",
              on_hit = {
                { op = "cue", id = "b21w_impact", at = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:16-35",
            text = "飛行距離固定為施法距離（GetCastRange，推定不含施法距離加成），與點選位置遠近無關，只取方向；點在自己身上時沿面向發射。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:26-28",
            text = "目標設定取技能 KV：敵方 HERO|BASIC、沒有 TargetFlags，魔免敵人不被命中；隱形與迷霧中的敵人照樣命中（選取預設）。DeleteOnHit 0，穿透。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:52-58",
            text = "傷害用 ApplyDamage（攻擊者為馬場），馬場在波飛行中死亡時由規則層歸零（require_source_alive 預設）。" },
          { policy = "note", ref = "npc/B_Unified/B21.txt:38",
            text = "AbilityBehavior DIRECTIONAL | POINT：DIRECTIONAL 只影響指示器。bProvidesVision false，不給視野；fExpireTime 5 秒長於實際飛行時間，不影響結算。" },
        },
        provenance = { "npc/B_Unified/B21.txt:31-98", "vscripts/heroes/B_Unified/B21.lua:3-59" },
      },
      -------------------------------------------------------------------------
      { id = "b21_killing_intent", source_id = "B21E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 25, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          buff_duration = { 4, 8, 12, 16, unit = "s", source = "kv:buff_duration" },
          move_speed_pct = { 30, unit = "pct", source = "kv:bonus_movement_speed" },
          attack_speed = { 35, 60, 85, 110, unit = "atk_speed", source = "kv:bonus_attack_speed" },
          stun_hp_threshold = { 65, unit = "pct", source = "kv:stun_hp_threshold" },
          stun_time = { 1.5, unit = "s", source = "kv:stun_time" },
          stun_radius = { 500, unit = "dist", source = "kv:stun_radius", note = "AbilityCastRange 同為 500，只是指示器" },
        },
        text = {
          zh_TW = { name = "殺意",
            body = "解放殺意，{buff_duration} 秒內移動速度提高 {move_speed_pct}、攻擊速度提高 {attack_speed}。施放時周圍 {stun_radius} 內生命值不高於 {stun_hp_threshold} 的非魔免敵方英雄與一般單位會因懼怕而暈眩 {stun_time} 秒。" },
          en = { name = "Killing Intent",
            body = "Unleashes killing intent, gaining +{move_speed_pct} movement speed and +{attack_speed} attack speed for {buff_duration} seconds. On cast, non-spell-immune enemy heroes and units within {stun_radius} at or below {stun_hp_threshold} health are stunned in fear for {stun_time} seconds." },
        },
        target = { type = "none", aoe_radius = "$stun_radius" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b21e_buff", duration = "$buff_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b21e_black_flame",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct" },
            text = {
              zh_TW = { name = "殺意", body = "移動速度提高 {move_speed_pct}、攻擊速度提高 {attack_speed}。" },
              en = { name = "Killing Intent", body = "+{move_speed_pct} movement speed and +{attack_speed} attack speed." },
            },
            notes = "原作 modifier_B21E_buff：IsBuff，沒寫 IsPurgable（推定 basic）。" },
          { id = "b21e_stun", duration = "$stun_time", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "懼怕", body = "因{@b21_killing_intent}而懼怕，暈眩 {stun_time} 秒。" },
              en = { name = "Terrified", body = "Terrified by {@b21_killing_intent}; stunned for {stun_time} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned；受狀態抗性縮短。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "b21e_voice", at = "caster" } },
          on_cast = {
            { op = "apply_status", status = "b21e_buff", to = "caster" },
            { op = "cue", id = "b21e_scream", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$stun_radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, where = "it.hp_pct <= $stun_hp_threshold" },
              actions = {
                { op = "apply_status", status = "b21e_stun", to = "it" },
              } },
            { op = "cue", id = "b21e_chaos_strike_sound", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:75-90",
            text = "FindUnitsInRadius 沒有旗標：魔免敵人不會被暈，隱形與迷霧中的敵人照樣會被暈。判定以施放當下的生命為準。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B21.lua:87",
            text = "GetHealthPercent 回傳整數百分比；資料以比例 hp_pct 比較，邊界上（65% 與 66% 之間）可能差一格。原作說明寫「低於 65%」，實際是不高於。" },
          { policy = "note", ref = "npc/B_Unified/B21.txt:139-143",
            text = "stun_time 的 var_type 是 FIELD_INTEGER 但值寫 1.5；推定 GetSpecialValueFor 照樣讀到 1.5（與說明一致），未實測。" },
          { policy = "note", ref = "npc/B_Unified/B21.txt:107",
            text = "IMMEDIATE：不打斷目前動作；仍有 0.25 秒前搖。" },
        },
        provenance = { "npc/B_Unified/B21.txt:100-203", "vscripts/heroes/B_Unified/B21.lua:61-93" },
      },
      -------------------------------------------------------------------------
      { id = "b21_rebel_armor", source_id = "B21R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          armor = { 6, 9, 12, 15, unit = "armor", source = "kv:add_armor" },
          damage = { 20, 40, 60, 80, unit = "dmg", damage_type = "pure", source = "kv:AbilityDamage" },
          hero_min_damage = { 130, unit = "dmg",
            source = "script:vscripts/heroes/B_Unified/B21.lua:125 keys.attacker:IsHero() and keys.original_damage < 130" },
        },
        text = {
          zh_TW = { name = "反逆甲",
            body = "被動：護甲提高 {armor}。每次受到物理傷害時，對傷害來源造成 {damage} 純粹傷害；來源是英雄時，該次傷害在減免前必須達到 {hero_min_damage} 才會反擊。不反擊建築、無敵單位與反射傷害。" },
          en = { name = "Rebel Armor",
            body = "Passive: +{armor} armor. Whenever Nobufusa takes physical damage, the source takes {damage} pure damage; if the source is a hero, that instance must be at least {hero_min_damage} before reductions. Does not retaliate against buildings, invulnerable units, or reflected damage." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public", visibility = "buff",
          stats = { armor = "$armor" },
          triggers = {
            on_take_damage = {
              filter = "event.damage.is_physical and not event.damage.is_reflection and exists(event.attacker) and not (event.attacker.is_hero and event.damage.raw < $hero_min_damage) and not event.attacker.is_building and not event.attacker.is_invulnerable",
              actions = {
                { op = "cue", id = "b21r_return", at = "self", to = "event.attacker" },
                { op = "damage", to = "event.attacker", source = "self", type = "pure", amount = "$damage", require_source_alive = false,
                  flags = { "reflection" } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:108-144",
            text = "反擊沒有檢查敵我：任何物理傷害來源（包括反補的友軍與物理技能）都會被反擊；說明寫「被敵軍物理攻擊時」。非英雄來源不論傷害多少都會反擊，英雄來源要減免前傷害達 130，說明沒有提。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:125",
            text = "IsHero() 對英雄的幻象也成立，所以幻象的攻擊同樣受 130 門檻限制（推定 v3 的 is_hero 對幻象為真）。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B21.lua:123",
            text = "只排除 damage_flags 恰好等於 REFLECTION 的傷害；帶其他旗標組合的反射傷害仍會觸發。v3 以 is_reflection 判斷，差異只在旗標組合時。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:146-156",
            text = "反擊邏輯在 Lua modifier modifier_b21r_lua，由 modifier_B21R 的 OnCreated 以 ApplyDataDrivenModifier 掛上、OnDestroy 移除；推定 ApplyDataDrivenModifier 對已 LinkLuaModifier 的名稱有效（原作可玩）。Lua modifier 不可驅散，與被動同生同滅。" },
          { policy = "as_source",
            text = "反擊是純粹傷害，魔免來源照樣受傷；馬場被這次傷害打死時，反擊由規則層歸零（攻擊者已死）。" },
          { policy = "no_effect", ref = "npc/B_Unified/B21.txt:236-243",
            text = "被動技能的 OnAbilityPhaseStart（播語音）永遠不會觸發。" },
        },
        provenance = { "npc/B_Unified/B21.txt:205-279", "vscripts/heroes/B_Unified/B21.lua:95-156" },
      },
      -------------------------------------------------------------------------
      { id = "b21_undying_demon", source_id = "B21T", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          str = { 15, 20, 25, unit = "stat", source = "kv:add_str" },
          attack = { 35, 45, 55, unit = "dmg", source = "kv:add_attack" },
          armor = { 4, 8, 12, unit = "armor", source = "kv:add_armor" },
          stack_hp = { 12, unit = "pct", source = "kv:stack_hp" },
          stack_max = { expr = "floor(1 / $stack_hp)", unit = "count",
            note = "FC-041（使用者 2026-10-29 決定全修）：最多幾份（生命損失 12% 一份，最多 8 份）" },
          attack_per_stack = { expr = "$attack / $stack_max", unit = "dmg", note = "FC-041：每份的攻擊加成，滿層合計 attack（原作的固定加成）" },
          armor_per_stack = { expr = "$armor / $stack_max", unit = "armor", note = "FC-041：每份的護甲加成，滿層合計 armor" },
          stun_delay = { 1, unit = "s", source = "script:vscripts/heroes/B_Unified/B21.lua:382 Timers:CreateTimer(1, ...)" },
          stun_radius = { 400, unit = "dist", source = "script:vscripts/heroes/B_Unified/B21.lua:384-386" },
          stun_duration = { 1, unit = "s",
            source = "script:vscripts/heroes/B_Unified/B21.lua:388 {duration=1}（KV modifier_B21T_stun Duration 同為 1）" },
        },
        text = {
          zh_TW = { name = "不死鬼美濃",
            body = "被動：力量永久提高 {str}。生命每損失 {stack_hp} 獲得一層{@b21t_bonus}（最多 {stack_max} 層）：每層攻擊力提高 {attack_per_stack}、護甲提高 {armor_per_stack}（滿層合計 {attack}、{armor}）；生命回復時層數跟著減少。每次學習或升級此技能，{stun_delay} 秒後暈眩周圍 {stun_radius} 內的敵方英雄與一般單位（包括魔免）{stun_duration} 秒。" },
          en = { name = "Undying Demon of Mino",
            body = "Passive: permanently grants +{str} strength. For every {stack_hp} of health Nobufusa has lost he gains a stack of {@b21t_bonus} (up to {stack_max}): each grants +{attack_per_stack} attack damage and +{armor_per_stack} armor (+{attack} and +{armor} at full); stacks drop as he heals. Each time this ability is learned or upgraded, enemy heroes and units within {stun_radius} (including spell-immune ones) are stunned for {stun_duration} seconds after {stun_delay} seconds." },
        },
        target = { type = "none", aoe_radius = "$stun_radius" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public", visibility = "hidden",
          stats = { str = "$str" },
          triggers = {
            on_take_damage = { { op = "call", routine = "b21t_update", args = { who = "self" } } },
            on_heal_received = { { op = "call", routine = "b21t_update", args = { who = "self" } } },
            -- 原作被動 modifier 在死亡時被移除（OnDestroy 重設 stack／modifier），重生時重建
            -- FC-105／FC-106（D13）：加成只看狀態本身在不在（不再有會失效的 handle），死亡時狀態隨 remove_on 移除

          },
        },
        statuses = {
          { id = "b21t_bonus", duration = "infinite", stacking = { max = "$stack_max", value_stacks = true }, dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b21t_aura", remove_on = { "death" },
            stats = { attack_damage = { value = "$attack_per_stack", per_stack = true }, armor = { value = "$armor_per_stack", per_stack = true } },
            text = {
              zh_TW = { name = "不死鬼美濃", body = "生命每損失 {stack_hp} 一層：每層攻擊力 +{attack_per_stack}、護甲 +{armor_per_stack}，最多 {stack_max} 層。" },
              en = { name = "Undying Demon of Mino", body = "One stack per {stack_hp} of health missing: +{attack_per_stack} attack damage and +{armor_per_stack} armor each, up to {stack_max} stacks." },
            },
            notes = "原作 modifier_B21T_stack：datadriven，屬性不乘層數；沒寫 IsPurgable（推定 basic）。FC-041 修正後每層 attack／armor ÷ stack_max，滿層等於原作的固定加成。" },
          { id = "b21t_stun", duration = "$stun_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被{@b21_undying_demon}暈眩 {stun_duration} 秒，不受狀態抗性縮短。" },
              en = { name = "Stunned", body = "Stunned by {@b21_undying_demon} for {stun_duration} seconds; not reduced by status resistance." },
            },
            reveal = "public", status_resistance = "ignore", states = { "stun" },
            notes = "原作 modifier_B21T_stun：IsStunDebuff，沒寫 IsPurgable（推定 basic）。施加者是被暈者自己，不受狀態抗性縮短。" },
        },
        events = {
          on_upgrade = {
            -- 升級後立刻依目前生命重算加成（FC-105：原作清空 handle 讓加成殘留）
            { op = "call", routine = "b21t_update", args = { who = "caster" } },
            { op = "delay", time = "$stun_delay", actions = {
              { op = "if", cond = "exists(caster) and not caster.is_illusion", actions = {
                { op = "for_each",
                  select = { from = "caster", shape = "circle", radius = "$stun_radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                  actions = {
                    { op = "apply_status", status = "b21t_stun", to = "it", source = "it" },
                  } },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/B_Unified/B21.txt:388-395",
            text = "FC-041（使用者 2026-10-29 決定全修：層數真的影響數值，每層 attack／armor ÷ stack_max，滿層等於原作固定加成）。原作：說明寫「生命每降低 12% 便提高攻擊力與護甲」，但 modifier_B21T_stack 是 datadriven，Properties 不乘層數：損失 12% 以上就只有一份固定加成，SetStackCount 只改顯示。" },
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B21.lua:158-208",
            text = "FC-105（D13 修正：加成只看狀態本身，升級後立刻依目前生命重算，不會殘留）。原作：升級大絕時被動重建，ability.modifier 被清空但 modifier_B21T_stack 沒有移除；若之後生命先回到損失不足 12%，因 handle 為空直接 return，加成會殘留在滿血狀態，直到下次生命降到門檻以下再回升才移除。" },
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B21.lua:192-206",
            text = "FC-106（D13 修正：驅散後下一次生命變化就重新套上（看 has_status，不靠 handle））。原作：加成狀態若被驅散（推定可被一般驅散），ability.modifier 仍指向已銷毀的 modifier：之後 Destroy／SetStackCount 都會報錯中斷，加成直到死亡或升級大絕前都不會再出現。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:177-190",
            text = "層數 = floor((100 − GetHealthPercent()) / 12)，只在 OnTakeDamage 與 OnHealthGained 時重算；GetHealthPercent 是整數，資料以比例 hp_pct 計算，邊界可能差一格。新套上加成時沒有 SetStackCount，顯示層數要到下一次層數改變才更新。" },
          { policy = "as_source", ref = "npc/B_Unified/B21.txt:370-377",
            text = "OnHealthGained 是否包含自然回復未實測；推定與 v3 on_heal_received 相同。若原作不含回復，靠回血回到門檻以上時加成會留到下一次受傷或治療。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:379-393",
            text = "OnUpgrade 每次學習、升級都觸發：1 秒後暈眩 400 內敵方 HERO|BASIC（含魔免）1 秒，說明沒有提。只檢查英雄有效且不是幻象，英雄死亡時升級也會在屍體位置暈眩。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B21.lua:388",
            text = "ApplyDataDrivenModifier(enemy, enemy, ...)：施加者是被暈者自己，events.lua 的狀態抗性只對敵方施加者生效，所以這個暈眩不受狀態抗性縮短；暈眩來源也不算馬場。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B21.lua:165",
            text = "OnCreated 把 ability.modifer（拼錯）設為 nil，沒有作用；handle 只靠 OnDestroy 清空。" },
          { policy = "note",
            text = "推定 datadriven Passive modifier 在死亡時移除、重生時重建（OnDestroy／OnCreated 各一次），所以死亡時層數與 handle 重設，加成狀態也隨死亡移除。" },
        },
        provenance = { "npc/B_Unified/B21.txt:281-397", "vscripts/heroes/B_Unified/B21.lua:158-208",
          "vscripts/heroes/B_Unified/B21.lua:379-393", "vscripts/events.lua:385-410" },
      },
    },
    routines = {
      -- 原作 B21T_OnHealthChange
      b21t_update = {
        params = { who = "unit" },
        actions = {
          { op = "let", name = "s", value = "floor((1 - who.hp_pct) / $b21_undying_demon.stack_hp)" },
          { op = "if", cond = "s == 0",
            actions = { { op = "remove_status", from = "who", status = "b21t_bonus" } },
            otherwise = {
              -- 被驅散後下一次生命變化就重新套上（FC-106）
              { op = "if", cond = "has_status(who, b21t_bonus)",
                actions = { { op = "set_stacks", to = "who", status = "b21t_bonus", count = "s" } },
                otherwise = { { op = "apply_status", status = "b21t_bonus", to = "who", stacks = "s" } } },
            } },
        },
      },
    },
  }
end
