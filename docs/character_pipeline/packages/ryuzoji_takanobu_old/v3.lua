-- C13 龍造寺隆信（舊版，11 版 *_old，KV 註解「11.2b」）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C13"]["11"] = "WERDT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:329）；KV 五招都存在（npc/C_Neutral/C13.txt:353-661）。
-- 自動學習 heromap_autoskill["C13"]["11"] = "D"（addon_game_mode.lua:432）：C13D_old（NOT_LEARNABLE 主動）出生一級，寫在 hero.auto_learn。
-- W/E/R/T → Q/W/E/R，D 照慣例放 D。舊版五招都有繁中說明（resource/addon_tchinese.txt:3762-3771），說明以腳本實際行為為準。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C13_precache 不列入。
-- 慣例照本體 C13：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；HERO | BASIC 寫成 BASIC 清單；
-- ApplyDamage／datadriven Damage（不經 AMHC）在隆信死亡後仍會造成傷害，寫 require_source_alive = false。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  local BASIC_BUILDING = { "hero", "creep", "neutral", "summon", "building" }
  return {
    schema_version = 3, kind = "hero", id = "C13_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = {
      source_id = "C13",
      text = { zh_TW = { name = "龍造寺隆信（舊版）" }, en = { name = "Ryuzoji Takanobu (Old)" } },
      auto_learn = { "c13_old_wind_chain" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c13_old_wind_chain", source_id = "C13D_old", slot = "D", kind = "active", max_level = 1, learnable = false,
        tags = { "basic" },
        values = {
          cooldown = { 30, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C13.txt:367" },
          mana_cost = { 150, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C13.txt:368" },
          cast_range = { 900, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C13.txt:369" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C13.txt:366" },
          duration = { 2, unit = "s", source = "script:vscripts/heroes/C_Neutral/C13.lua:186 {duration = 2}（KV Duration 2@C13.txt:395 同值）" },
          damage = { 200, unit = "dmg", damage_type = "magical",
            source = "kv:AbilityDamage@npc/C_Neutral/C13.txt:408",
            note = "原作腳本沒有造成這個傷害；依 D22 照說明（addon_tchinese.txt:3763）補上，見 quirks" },
        },
        text = {
          zh_TW = { name = "風之鎖",
            body = "以旋風鎖住目標敵人 {duration} 秒，使其無法移動、攻擊、施法，失明且無法閃避，並造成 {damage} 魔法傷害。本技能無法手動學習，出生即有一級。" },
          en = { name = "Wind Chain",
            body = "Binds the target enemy in a whirlwind for {duration} seconds: it cannot move, attack or cast, is blinded and cannot evade, and takes {damage} magic damage. Cannot be learned manually; starts at level one." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c13d_old_bind", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "風之鎖", body = "被旋風鎖住：無法移動、攻擊、施法，失明且無法閃避。" },
              en = { name = "Wind Chain", body = "Bound by a whirlwind: cannot move, attack or cast, blinded and cannot evade." },
            },
            reveal = "public", cue = "c13d_old_whirl",
            states = { "root", "evade_disabled", "silence", "blind", "disarm" },
            notes = "原作 modifier_C13D_old：ROOTED、EVADE_DISABLED、SILENCED、BLIND、DISARMED，Duration 2、IsDebuff；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "c13d_old_bind", to = "target" },
            { op = "damage", to = "target", type = "magical", amount = "$damage", require_source_alive = false },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/C_Neutral/C13.lua:182-187 / npc/C_Neutral/C13.txt:408",
            text = "FC-177 D22 舊版，比照 D13 修：原作 C13D_old_OnSpellStart 只套 modifier_C13D_old，KV 的 AbilityDamage 200 沒有任何地方使用，實際不造成傷害；原作說明寫「給予200點傷害」。修法：照說明在套上狀態後造成 AbilityDamage 魔法傷害（AbilityUnitDamageType MAGICAL；推定以 ApplyDamage 實作，隆信死後施放中的不會發生，require_source_alive 不影響）。" },
          { policy = "as_source", ref = "npc/C_Neutral/C13.txt:357-370",
            text = "UNIT_TARGET | AOE | NOT_LEARNABLE；AoERadius 指向不存在的 %radius（零），只是指示。SpellImmunityType ENEMIES_NO、沒有 MAGIC_IMMUNE 旗標：不能對魔免敵人施放。HotKeyOverride E 只是熱鍵。" },
          { policy = "note", ref = "npc/C_Neutral/C13.txt:385-398", text = "b09t 旋風粒子只是呈現。" },
        },
        provenance = { "npc/C_Neutral/C13.txt:353-409", "vscripts/heroes/C_Neutral/C13.lua:182-187",
          "vscripts/addon_game_mode.lua:432", "resource/addon_tchinese.txt:3762-3763" },
      },
      -------------------------------------------------------------------------
      { id = "c13_old_bloody_gale", source_id = "C13W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic", "summon" },
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C13.txt:423" },
          mana_cost = { 130, 160, 190, 220, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C13.txt:424" },
          cast_point = { 0, unit = "s", source = "rules:KV 沒寫 AbilityCastPoint（預設 0），且是 IMMEDIATE" },
          lifetime = { 40, unit = "s", source = "kv:duration@npc/C_Neutral/C13.txt:436" },
          wind_damage = { 50, 70, 90, 110, unit = "dmg", source = "kv:damage@npc/C_Neutral/C13.txt:431（C13.lua:199-200 damage + level*5）" },
          wind_damage_per_level = { 5, unit = "dmg", source = "script:vscripts/heroes/C_Neutral/C13.lua:199-200 damage + caster:GetLevel()*5" },
          wind_search = { 1000, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C13.lua:97-99 FindUnitsInRadius 1000" },
          wind_think = { 1, unit = "s", hidden = true, source = "kv:modifier_C13W_old ThinkInterval@npc/C_Neutral/C13.txt:459" },
          building_pct = { 20, unit = "pct",
            source = "script:vscripts/heroes/C_Neutral/C13.lua:86-91 SetHealth(hp - hp2*0.2)：建築只承受兩成" },
        },
        text = {
          zh_TW = { name = "血腥狂風",
            body = "立即在隆信身旁召喚一個無法控制、無敵的{@c13w_old_wind}，存在 {lifetime} 秒。龍捲風攻擊力為 {wind_damage} 加隆信等級 × {wind_damage_per_level}，會自動追擊 {wind_search} 內最近、看得見的敵人（包含建築）；對建築只造成 {building_pct} 傷害。" },
          en = { name = "Bloody Gale",
            body = "Instantly summons an uncontrollable, invulnerable {@c13w_old_wind} beside Takanobu for {lifetime} seconds. The tornado's attack damage is {wind_damage} plus Takanobu's level × {wind_damage_per_level}; it chases the nearest visible enemy within {wind_search} (buildings included) and deals only {building_pct} damage to buildings." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c13w_old_wind_ai", duration = "$lifetime", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "unselectable", "invulnerable", "no_unit_collision" },
            triggers = {
              periodic = { interval = "$wind_think", actions = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$wind_search", team = "enemy", types = BASIC_BUILDING,
                    flags = { "visible_only", "include_magic_immune" }, order = "nearest", limit = "1" },
                  actions = {
                    { op = "issue_order", who = "self", order = "attack", target = "it", force = true },
                  } },
              } },
              on_deal_damage_pre = {
                filter = "event.victim.is_building and event.damage.is_attack",
                actions = { { op = "modify_damage", multiplier = "$building_pct" } } },
            },
            notes = "原作 modifier_C13W_old（ThinkInterval 1 借用 C13R_OnIntervalThink、UNSELECTABLE、OnAttackLanded 借用 C13R_OnAttackLanded）＋ modifier_invulnerable、modifier_kill 40 秒、for_no_collision。" },
        },
        events = {
          on_cast = {
            { op = "create_unit", template = "c13w_old_wind", at = "position(caster)", owner = "caster", control = "none",
              lifetime = "$lifetime", as = "u",
              overrides = {
                attack_damage = "$wind_damage + caster.level * $wind_damage_per_level",
                attack_damage_max = "$wind_damage + caster.level * $wind_damage_per_level",
              } },
            { op = "apply_status", status = "c13w_old_wind_ai", to = "u" },
            { op = "cue", id = "c13w_old_wind_fx", attach = "u", duration = "$lifetime" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C13.lua:189-207",
            text = "和本體 C13R 的龍捲風同一種單位（c13r_wind），但改成主動召喚：沒有數量上限（冷卻 8 秒、存在 40 秒，最多約五個同時存在），風身上沒有 OnKill，擊殺不會再召喚。攻擊力在召喚當下以隆信等級設定，之後升級不變。原作說明「每0.6秒攻擊一次」即單位的 AttackRate。" },
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C13.lua:80-93",
            text = "對建築是在 OnAttackLanded 記下生命，0.01 秒後 SetHealth 回補八成：那 0.01 秒內其他來源的傷害也會被回補八成，致死的一擊則來不及回補。照設計意圖（同本體）寫成只把風對建築的攻擊傷害乘兩成。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C13.lua:95-115",
            text = "每秒對 1000 內最近、看得見的敵人（含建築與魔免）下攻擊命令並 SetForceAttackTarget；沒有目標時取消強制目標，交給一般索敵（AttackAcquisitionRange 1500）。CanEntityBeSeenByMyTeam 的分支因 FOW_VISIBLE 已篩過而永遠不成立（同本體）。" },
          { policy = "note", ref = "npc/C_Neutral/C13.txt:417-421",
            text = "KV 的 AbilityUnitTargetFlags 與傷害類型沒有作用；c13r 粒子只是呈現。" },
        },
        provenance = { "npc/C_Neutral/C13.txt:410-483", "vscripts/heroes/C_Neutral/C13.lua:80-115,189-207",
          "npc/npc_units_custom_load.txt:8-72", "npc/npc_abilities_custom_load.txt:5247-5263" },
      },
      -------------------------------------------------------------------------
      { id = "c13_old_tornado_wave", source_id = "C13E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, 12, 9, 6, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C13.txt:496" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C13.txt:497" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C13.txt:498" },
          wave_distance = { 1200, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C13.lua:41 fDistance 1200" },
          wave_radius = { 200, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C13.lua:42-43" },
          wave_speed = { 1000, unit = "speed", hidden = true, source = "script:vscripts/heroes/C_Neutral/C13.lua:51 dir * 1000" },
        },
        text = {
          zh_TW = { name = "龍捲波",
            body = "朝指定方向丟出長 {wave_distance}、寬 {wave_radius} 的龍捲波，貫穿路徑上非魔免的敵方英雄與部隊，各造成 {damage} 魔法傷害。" },
          en = { name = "Tornado Wave",
            body = "Throws a tornado wave {wave_distance} long and {wave_radius} wide in the target direction, passing through non-spell-immune enemy heroes and units and dealing {damage} magic damage to each." },
        },
        target = { type = "point", aoe_radius = "$wave_radius" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_phase_start = { { op = "cue", id = "c13e_old_vo", at = "caster" } },
          on_cast = {
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$wave_speed", distance = "$wave_distance", radius = "$wave_radius", pierce = true,
              team = "enemy", types = BASIC, cue = "c13e_old_wind",
              on_hit = { { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false } } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C13.txt:486-555",
            text = "和本體 C13E 完全相同（同一個腳本 C13E_OnSpellStart、同樣的 KV 數值）。KV 沒寫 AbilityCastRange 與 AbilityCastPoint：任何距離的地點都能指定（只取方向），沒有前搖。datadriven Damage 不檢查施法者存活。" },
          { policy = "note", ref = "npc/C_Neutral/C13.txt:500-516",
            text = "AbilitySpecial dmg／interval／move 與 AbilityUnitTargetTeam FRIENDLY 都沒有被使用。" },
        },
        provenance = { "npc/C_Neutral/C13.txt:485-555", "vscripts/heroes/C_Neutral/C13.lua:32-54" },
      },
      -------------------------------------------------------------------------
      { id = "c13_old_strength_up", source_id = "C13R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          str = { 8, 16, 24, 32, unit = "stat", source = "kv:add_str@npc/C_Neutral/C13.txt:570" },
        },
        text = {
          zh_TW = { name = "力量強化", body = "被動：力量 +{str}。" },
          en = { name = "Strength Up", body = "Passive: +{str} Strength." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { str = "$str" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/C_Neutral/C13.txt:574-586",
            text = "modifier 名稱是 modifier_A08R_passive（和其他英雄同名，隱藏、Passive 的 STATS_STRENGTH_BONUS 綠字）；同名 modifier 不會同時出現在一位英雄身上。" },
        },
        provenance = { "npc/C_Neutral/C13.txt:556-588" },
      },
      -------------------------------------------------------------------------
      { id = "c13_old_tornado_vortex", source_id = "C13T_old", slot = "R", kind = "channel", max_level = 3, tags = { "ultimate", "channel" },
        values = {
          cooldown = { 130, 120, 110, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C13.txt:601" },
          mana_cost = { 300, 400, 500, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C13.txt:602" },
          cast_point = { 1.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C13.txt:606" },
          channel_time = { 4, unit = "s", source = "kv:AbilityChannelTime@npc/C_Neutral/C13.txt:604" },
          required_level = { 6, 12, 18, unit = "level", source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          radius = { 500, unit = "dist", source = "kv:radius@npc/C_Neutral/C13.txt:632（AbilityCastRange 500@C13.txt:605 同值）" },
          tick_damage = { 24, 35, 46, unit = "dmg", damage_type = "magical", source = "kv:dmg@npc/C_Neutral/C13.txt:627" },
          tick = { 0.03, unit = "s", source = "script:vscripts/heroes/C_Neutral/C13.lua:264-266 tsum += 0.03、return 0.03" },
          tick_count = { 134, unit = "count", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C13.lua:264-266 tsum < 4 才繼續：第 134 跳時 tsum 超過 4" },
          pull = { 30, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C13.lua:237 dir*30" },
          swirl = { 50, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C13.lua:234-237 right*50" },
          right_angle = { 90, unit = "deg", hidden = true, source = "script:vscripts/heroes/C_Neutral/C13.lua:234-236 right = (dir.y, -dir.x)" },
          hold = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C13.lua:238 {duration = 0.1}" },
          item_lock = { 0.3, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C13.lua:245 item:StartCooldown(0.3)" },
          blast_radius = { 300, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C13.lua:271 FindUnitsInRadius 300" },
          blast_hold = { 1.5, unit = "s", source = "script:vscripts/heroes/C_Neutral/C13.lua:281 modifier_C13T {duration = 1.5}" },
          knockback_distance = { 800, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C13.lua:291 knockback_distance 800" },
          knockback_time = { 0.3, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C13.lua:289-290 duration 0.3" },
        },
        text = {
          zh_TW = { name = "龍捲之漩渦",
            body = "持續施法，以自身為漩渦中心：每 {tick} 秒把周圍 {radius} 內的敵方英雄與部隊（包含魔免）朝隆信拉近 {pull} 並橫向捲動 {swirl}，使其浮空、定身、沉默、繳械、不能閃避，並造成 {tick_damage} 魔法傷害。完整施法結束時，把 {blast_radius} 內的敵人吹飛 {knockback_distance}，並在 {blast_hold} 秒內維持捲入狀態。" },
          en = { name = "Tornado Vortex",
            body = "Channels as the eye of a vortex: every {tick} seconds, enemy heroes and units within {radius} (spell-immune ones included) are pulled {pull} toward Takanobu and swept {swirl} sideways; they are lifted, rooted, silenced, disarmed and unable to evade, and take {tick_damage} magic damage. When the channel completes, enemies within {blast_radius} are blown {knockback_distance} away and stay caught for {blast_hold} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "c13t_old_hold", duration = "$hold", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "龍捲之漩渦", body = "被捲入漩渦：浮空、定身、沉默、繳械，且無法閃避攻擊。" },
              en = { name = "Tornado Vortex", body = "Caught in the vortex: lifted, rooted, silenced, disarmed and unable to evade attacks." },
            },
            reveal = "public", states = { "root", "evade_disabled", "silence", "disarm", "flying" },
            notes = "原作 C13T_old 的 modifier_C13T（KV 寫 IsBuff 1，但掛在敵人身上）：ROOTED、EVADE_DISABLED、SILENCED、DISARMED、FLYING；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c13t_old_title" },
            { op = "cue", id = "c13t_old_vortex", attach = "caster" },
            { op = "repeat", count = "$tick_count", interval = "$tick", first = "0", ["while"] = "caster.is_channeling",
              actions = {
                { op = "for_each",
                  select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                    flags = { "include_magic_immune", "include_untargetable" } },
                  actions = {
                    { op = "if", cond = "not has_state(it, untargetable_by_spells)", actions = {
                      { op = "teleport", who = "it", find_clear_space = false,
                        to = "offset(offset(it, direction(it, caster), $pull), rotate(direction(it, caster), -$right_angle), $swirl)" },
                      { op = "apply_status", status = "c13t_old_hold", to = "it" },
                      { op = "if", cond = "it.is_hero and not it.is_illusion", actions = {
                        { op = "modify_cooldown", of = "it", item_tag = "blink", mode = "start", amount = "$item_lock" },
                      } },
                    } },
                    { op = "damage", to = "it", type = "magical", amount = "$tick_damage", require_source_alive = false },
                  } },
              } },
          },
          on_channel_end = {
            { op = "cue", id = "c13t_old_vortex", stop = true },
            { op = "if", cond = "not event.interrupted", actions = {
              { op = "for_each",
                select = { from = "caster", shape = "circle", radius = "$blast_radius", team = "enemy", types = BASIC,
                  flags = { "include_magic_immune" } },
                actions = {
                  { op = "apply_status", status = "c13t_old_hold", to = "it", duration = "$blast_hold" },
                  { op = "knockback", who = "it", from = "position(caster)", distance = "$knockback_distance",
                    duration = "$knockback_time", stun = true },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/C_Neutral/C13.lua:210-306",
            text = "FC-178 D22 舊版，比照 D13 修：原作的吹飛寫在 Timer 第 134 跳（tsum 累加 0.03 超過 4）之後，但 Timers 每幀（30Hz，約 0.033 秒）才跑一次，134 跳要約 4.4 秒，比 4 秒引導還長：引導結束時 IsChanneling 為假，Timer 直接結束，實際上永遠不會吹飛。原作說明寫「拉扯持續4秒，結束後吹飛所有敵人」。修法：完整引導結束時（on_channel_end 且沒被打斷）對 300 內敵人套 1.5 秒捲入狀態並從隆信位置擊退 800（0.3 秒、擊退中暈眩）；被打斷時照原作不吹飛。" },
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C13.lua:218-266",
            text = "第一跳在施放當下，之後每 0.03 秒、只要仍在引導；30Hz 下實際每幀一跳（4 秒約 120 跳），照腳本的 0.03 寫，由執行器對齊幀。每跳傷害 24/35/46，AbilityDamage 240/350/460 沒有被使用。ApplyDamage 不檢查施法者存活。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C13.lua:229-250",
            text = "_G.EXCLUDE_TARGET_NAME 的單位不被拉、不被定身，但 ApplyDamage 在判斷外面，照樣受傷；v3 以 include_untargetable 選進來、只對一般單位拉動（同本體）。每跳以 SetAbsOrigin 朝隆信移 30、往右（順時針切線）移 50，不找空位。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C13.lua:1-6,239-248",
            text = "對英雄（非分身）身上的 item_magic_ring、item_flash_ring、item_flash_shoes 每跳 StartCooldown(0.3)，等於漩渦中不能閃現；資料用 modify_cooldown 的 item_tag = blink（同本體）。" },
          { policy = "as_source", ref = "npc/C_Neutral/C13.txt:637-655",
            text = "和本體不同：舊版捲入狀態多了 FLYING、沒有 phased；每跳只維持 0.1 秒。前搖 1.1 秒、引導 4 秒。" },
          { policy = "note", ref = "npc/C_Neutral/C13.txt:608-620",
            text = "showTitle 與 c13r 粒子只是呈現。" },
        },
        provenance = { "npc/C_Neutral/C13.txt:589-659", "vscripts/heroes/C_Neutral/C13.lua:1-6,210-306",
          "resource/addon_tchinese.txt:3770-3771" },
      },
    },
    unit_templates = {
      { id = "c13w_old_wind", source_id = "c13r_wind",
        values = {
          hp = { 1000, unit = "hp", source = "kv:StatusHealth@npc/npc_units_custom_load.txt:9" },
          armor = { 1000, unit = "armor", source = "kv:ArmorPhysical@npc/npc_units_custom_load.txt:9" },
          magic_resist = { 100, unit = "pct", source = "kv:MagicalResistance@npc/npc_units_custom_load.txt:9" },
          move_speed = { 250, unit = "move_speed", source = "kv:MovementSpeed@npc/npc_units_custom_load.txt:9" },
          attack_range = { 128, unit = "dist", source = "kv:AttackRange@npc/npc_units_custom_load.txt:9" },
          attack_interval = { 0.6, unit = "s", source = "kv:AttackRate@npc/npc_units_custom_load.txt:9" },
          attack_damage = { 100, unit = "dmg", hidden = true, source = "kv:AttackDamageMin@npc/npc_units_custom_load.txt:9",
            note = "召喚時以 damage + 等級 × 5 覆寫" },
          acquisition = { 1500, unit = "dist", source = "kv:AttackAcquisitionRange@npc/npc_units_custom_load.txt:9" },
          vision = { 500, unit = "dist", source = "kv:VisionDaytimeRange@npc/npc_units_custom_load.txt:9" },
        },
        text = { zh_TW = { name = "血腥狂風" }, en = { name = "Bloody Gale" } },
        stats = { hp = "$hp", armor = "$armor", magic_resist = "$magic_resist", move_speed = "$move_speed" },
        attack = { kind = "melee", range = "$attack_range", interval = "$attack_interval", damage = "$attack_damage", attack_point = "0" },
        vision = { day = "$vision", night = "$vision" },
        acquisition_range = "$acquisition",
        ai = "idle", tags = { "summon" },
        notes = "與本體 c13r_wind 同一個原作單位（npc_dota_creature，Level 2、Bounty 100、MovementTurnRate 3、HasInventory 0）；模型與粒子屬呈現。",
        provenance = { "npc/npc_units_custom_load.txt:8-72" },
      },
    },
  }
end
