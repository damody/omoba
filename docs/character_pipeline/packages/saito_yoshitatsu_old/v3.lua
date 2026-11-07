-- C14 齋藤義龍（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C14"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:330）；
-- heromap_autoskill["C14"]["11"] = ""（addon_game_mode.lua:433），沒有出生自動學的技能，也沒有 D。
-- KV 在 npc/C_Neutral/C14.txt:554-861（C14W_old／C14E_old／C14R_old／C14T_old）。W/E/R/T → Q/W/E/R。
-- 舊版沒有 C14E2：C14T_old 施放時腳本找 C14E 失敗而中斷（見大招 quirks），火龍吐息術不換技能。
-- 原作舊版沒有任何繁中說明（panorama/localization/addon_tchinese.txt 查無 C14*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C14_precache 屬規則層或資產，不列入。
-- 慣例照本體 C14：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；HERO | BASIC 寫成 BASIC 清單；緩速寫 slow_pct 正值並帶 tag slow。
local BASIC = { "hero", "creep", "neutral", "summon" }

return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "C14_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C14", text = { zh_TW = { name = "齋藤義龍（舊版）" }, en = { name = "Saito Yoshitatsu (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c14_old_fire_curse", source_id = "C14W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 25, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C14.txt:568" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C14.txt:569" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C14.txt:567" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C14.txt:566" },
          radius = { 300, unit = "dist",
            source = "kv:AOERadius@npc/C_Neutral/C14.txt:570（腳本 vscripts/heroes/C_Neutral/C14.lua:8 寫死 300，同值）" },
          duration = { 6, unit = "s",
            source = "kv:modifier_C14W Duration@npc/C_Neutral/C14.txt:585（腳本 C14.lua:11 {duration = 6}，同值）" },
          slow = { 30, 35, 40, 45, unit = "pct", kv = { -30, -35, -40, -45 },
            source = "kv:speed_bonus@npc/C_Neutral/C14.txt:607",
            source_differs = "KV 以負值表示減速（MOVESPEED_BONUS_PERCENTAGE）；slow_pct 以正值表示" },
          order_delay = { 0.2, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C14.lua:15 Timers 0.2" },
        },
        text = {
          zh_TW = { name = "火靈詛咒",
            body = "詛咒目標敵人與其周圍 {radius} 內非魔免的敵方英雄與部隊 {duration} 秒：沉默，移動速度降低 {slow}。被詛咒的敵人受到{@c14_old_fire_breath}時會持續灼燒。施放後義龍自動攻擊目標。" },
          en = { name = "Curse of the Fire Spirit",
            body = "Curses the target enemy and non-spell-immune enemy heroes and units within {radius} of it for {duration} seconds: silenced and slowed by {slow}. Cursed enemies hit by {@c14_old_fire_breath} keep burning. Yoshitatsu then attacks the target." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC, aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c14w_old_curse", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "slow" }, states = { "silence" }, stats = { slow_pct = "$slow" }, cue = "c14w_old_curse",
            text = {
              zh_TW = { name = "火靈詛咒", body = "沉默，移動速度降低 {slow}；受到{@c14_old_fire_breath}時會持續灼燒。" },
              en = { name = "Curse of the Fire Spirit", body = "Silenced and movement speed reduced by {slow}; keeps burning when hit by {@c14_old_fire_breath}." },
            },
            notes = "原作 modifier_C14W（C14W_old 版）：IsPurgable 1、Duration 6、SILENCED、MOVESPEED_BONUS_PERCENTAGE %speed_bonus。" },
        },
        events = {
          on_cast = {
            { op = "for_each",
              select = { from = "position(target)", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                where = "not it.is_building" },
              actions = { { op = "apply_status", status = "c14w_old_curse", to = "it" } } },
            { op = "cue", id = "c14w_old_cast_gesture", at = "caster" },
            { op = "delay", time = "$order_delay", actions = {
              { op = "issue_order", who = "caster", order = "attack", target = "target" },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C14.lua:4-21",
            text = "和本體共用 C14W_OnSpellStart：以目標為圓心 300 搜尋，用技能的目標隊伍／類型／旗標（NONE），魔免敵人不受詛咒；沒有傷害。0.2 秒後以腳本命令攻擊原目標。和本體不同：冷卻 25、耗魔 120～180、緩速 30%～45%。" },
          { policy = "note", ref = "npc/C_Neutral/C14.txt:558",
            text = "AbilityBehavior 有 AOE 只影響施放指示圈。" },
        },
        provenance = { "npc/C_Neutral/C14.txt:554-611", "vscripts/heroes/C_Neutral/C14.lua:4-21" },
      },
      -------------------------------------------------------------------------
      { id = "c14_old_fire_breath", source_id = "C14E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C14.txt:625" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C14.txt:626" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C14.txt:628" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C14.txt:623" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C14.txt:627" },
          speed = { 2000, unit = "speed", hidden = true, source = "kv:shock_speed@npc/C_Neutral/C14.txt:698" },
          distance = { 800, unit = "dist", source = "kv:FixedDistance@npc/C_Neutral/C14.txt:639" },
          start_radius = { 100, unit = "dist", hidden = true, source = "kv:StartRadius@npc/C_Neutral/C14.txt:636" },
          end_radius = { 300, unit = "dist", source = "kv:EndRadius@npc/C_Neutral/C14.txt:638" },
          burn_damage = { 50, 60, 70, 80, unit = "dmg", damage_type = "magical", source = "kv:damage@npc/C_Neutral/C14.txt:703" },
          burn_duration = { 7, unit = "s", source = "kv:modifier_C14E_effect Duration@npc/C_Neutral/C14.txt:673" },
          burn_interval = { 1, unit = "s", hidden = true, source = "kv:modifier_C14E_effect ThinkInterval@npc/C_Neutral/C14.txt:674" },
        },
        text = {
          zh_TW = { name = "火龍吐息術",
            body = "朝指定方向噴出長 {distance}、末端寬 {end_radius} 的扇形火焰，對非魔免的敵方英雄與部隊造成 {damage} 魔法傷害。被{@c14_old_fire_curse}詛咒的敵人另外灼燒 {burn_duration} 秒，立刻與之後每秒受到 {burn_damage} 魔法傷害，無法驅散。" },
          en = { name = "Fire Dragon Breath",
            body = "Breathes a cone of fire {distance} long and {end_radius} wide at the end in the target direction, dealing {damage} magic damage to non-spell-immune enemy heroes and units. Enemies cursed by {@c14_old_fire_curse} also burn for {burn_duration} seconds, taking {burn_damage} magic damage immediately and every second; the burn cannot be dispelled." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c14e_old_burn", duration = "$burn_duration", stacking = "refresh", dispel = "none", visibility = "debuff",
            text = {
              zh_TW = { name = "火龍吐息灼燒", body = "立刻與之後每秒受到 {burn_damage} 魔法傷害，無法驅散。" },
              en = { name = "Fire Breath Burn", body = "Takes {burn_damage} magic damage immediately and every second. Cannot be dispelled." },
            },
            reveal = "public", cue = "c14e_old_burn",
            triggers = {
              on_created = { filter = "not self.is_building", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$burn_damage", require_source_alive = false },
              } },
              periodic = { interval = "$burn_interval", filter = "not self.is_building", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$burn_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_C14E_effect（C14E_old 版）：IsPurgable 0、Duration 7、ThinkInterval 1，OnCreated 與 OnIntervalThink 都呼叫 C14.lua:233-253 的 modifier_C14E_effect_OnIntervalThink（ApplyDamage，damage 50/60/70/80）。" },
        },
        events = {
          on_cast = {
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$speed", distance = "$distance", radius = "$start_radius", end_radius = "$end_radius",
              pierce = true, team = "enemy", types = BASIC, cue = "c14e_old_breath",
              on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "cue", id = "c14e_old_hit_sound", at = "it" },
                { op = "if", cond = "has_status(it, c14w_old_curse)", actions = {
                  { op = "apply_status", status = "c14e_old_burn", to = "it" },
                } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C14.txt:629-665",
            text = "和本體不同：沒有施放後 0.1 秒的 modifier_C14E 指令限制；冷卻 20、前搖 0.35、傷害 100～400、灼燒每跳 50～80。命中先造成 AbilityDamage，再由 C14E_OnProjectileHitUnit 檢查 modifier_C14W。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C14.lua:24-253",
            text = "modifier_C14E_effect_OnIntervalThink 只定義在 233 行（63、159 行是 modifier_C14T_effect_…，名稱不同）：讀 damage、不檢查有效性、不打建築，ApplyDamage 不檢查施法者存活，灼燒在義龍死後照樣傷害。" },
          { policy = "design_intent", ref = "npc/C_Neutral/C14.txt:668-690",
            text = "灼燒 OnCreated 一次、之後每 1 秒一次，持續 7 秒：第 7 秒那一跳是否發生依引擎時序；照 KV 的間隔與時間寫成 on_created + periodic。再次命中時重新整理時間。" },
          { policy = "note", ref = "npc/C_Neutral/C14.txt:631-644",
            text = "datadriven LinearProjectile：StartRadius 100、EndRadius 300、FixedDistance 800、HasFrontalCone；StartPosition 寫成 attach_orgin（拼錯）退回預設位置。旗標 NONE，魔免敵人不受影響。C14.lua:194-220 的 C14E 函式沒有被舊版呼叫。" },
        },
        provenance = { "npc/C_Neutral/C14.txt:613-707", "vscripts/heroes/C_Neutral/C14.lua:223-253" },
      },
      -------------------------------------------------------------------------
      { id = "c14_old_dragon_vein", source_id = "C14R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          damage_reduction = { 6, 8, 10, 12, unit = "pct", source = "kv:damage_bonus@npc/C_Neutral/C14.txt:749（MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE@C14.txt:737）",
            note = "原作是正值（受傷增加），依設計意圖改為減傷，見 quirks FC-180" },
          armor = { 4, 6, 8, 10, unit = "armor", source = "kv:armor_bonus@npc/C_Neutral/C14.txt:754" },
        },
        text = {
          zh_TW = { name = "龍脈強韌", body = "被動：護甲 +{armor}，受到的傷害降低 {damage_reduction}。" },
          en = { name = "Dragon's Vein", body = "Passive: +{armor} armor and takes {damage_reduction} less damage." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { armor = "$armor", incoming_damage_pct = "-$damage_reduction" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/C_Neutral/C14.txt:737,749",
            text = "FC-180 D22 舊版，比照 D13 修：原作 MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE 寫正值 %damage_bonus（6～12），實際讓義龍受到的傷害提高，和技能名「強韌」與同時給的護甲相反；推定原意是減傷，資料寫成 incoming_damage_pct = -damage_bonus（受到的傷害降低 6%～12%）。另一種可能是原意為增傷（變數名 damage_bonus、本體 C14R 是攻擊加成），列為不確定，見 assumptions.md。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C14.txt:720-727",
            text = "被動技能寫了 OnSpellStart（呼叫 C14W_OnSpellStart），永遠不會施放，是殘留。" },
          { policy = "note", ref = "npc/C_Neutral/C14.txt:730",
            text = "被動 modifier 名稱也叫 modifier_C14W，和火靈詛咒的減益同名；C14E_OnProjectileHitUnit 只查目標身上的 modifier_C14W，敵方不會有這個被動，所以不影響灼燒判定（對手也是舊版義龍時例外，見 assumptions.md）。" },
        },
        provenance = { "npc/C_Neutral/C14.txt:708-758" },
      },
      -------------------------------------------------------------------------
      { id = "c14_old_dragon_form", source_id = "C14T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 165, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C14.txt:771" },
          mana_cost = { 250, 300, 350, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C14.txt:772" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C14.txt:770" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel；DOTA_ABILITY_TYPE_ULTIMATE（C14.txt:765）預設 6/12/18" },
          duration = { 40, 50, 60, unit = "s", source = "kv:During@npc/C_Neutral/C14.txt:852" },
          move_speed = { 50, unit = "move_speed", source = "kv:speed_bonus@npc/C_Neutral/C14.txt:847" },
          attack_range = { 572, unit = "dist", source = "kv:modifier_C14T MODIFIER_PROPERTY_ATTACK_RANGE_BONUS@npc/C_Neutral/C14.txt:799" },
          armor = { 10, unit = "armor", source = "kv:modifier_C14T MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS@npc/C_Neutral/C14.txt:797" },
          bonus_hp = { 500, 1000, 1500, unit = "hp", source = "kv:Bonus_Health@npc/C_Neutral/C14.txt:857" },
          base_damage = { 20, unit = "dmg", source = "kv:modifier_C14T MODIFIER_PROPERTY_BASEATTACK_BONUSDAMAGE@npc/C_Neutral/C14.txt:801" },
          splash_pct = { 60, unit = "pct", source = "script:vscripts/AON_Cleave.lua:292 keys.dmg*0.60" },
          splash_radius = { 300, unit = "dist", source = "script:vscripts/AON_Cleave.lua:286-288 FindUnitsInRadius 300" },
        },
        text = {
          zh_TW = { name = "火龍化身",
            body = "變身為火龍 {duration} 秒：魔法免疫，改為遠程攻擊，攻擊距離 +{attack_range}、移動速度 +{move_speed}、護甲 +{armor}、生命上限 +{bonus_hp}、基礎攻擊力 +{base_damage}，不受攻擊速度緩速影響。普攻命中非建築的敵人時，對目標周圍 {splash_radius} 內的敵方英雄與部隊（包含目標本身與魔免）造成攻擊傷害 {splash_pct} 的物理傷害。死亡時變身結束。" },
          en = { name = "Fire Dragon Form",
            body = "Turns into a fire dragon for {duration} seconds: spell immune, attacks become ranged, with +{attack_range} attack range, +{move_speed} movement speed, +{armor} armor, +{bonus_hp} maximum health and +{base_damage} base attack damage; immune to attack speed slows. Attacks that land on non-building enemies deal {splash_pct} of the attack damage as physical damage to enemy heroes and units within {splash_radius} of the target (the target itself and spell-immune ones included). The form ends on death." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "c14t_old_form", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "火龍化身", body = "魔法免疫，遠程攻擊，攻擊距離 +{attack_range}、移動速度 +{move_speed}、護甲 +{armor}、生命上限 +{bonus_hp}、基礎攻擊力 +{base_damage}，不受攻擊速度緩速影響；普攻會濺射。" },
              en = { name = "Fire Dragon Form", body = "Spell immune, ranged attacks, +{attack_range} attack range, +{move_speed} movement speed, +{armor} armor, +{bonus_hp} maximum health, +{base_damage} base attack damage and immune to attack speed slows; attacks splash." },
            },
            reveal = "public", states = { "magic_immune", "ranged_attack", "attack_slow_immune" }, remove_on = { "death" },
            cue = "c14t_old_dragon_model",
            stats = { move_speed = "$move_speed", attack_range = "$attack_range", armor = "$armor", max_hp = "$bonus_hp",
              base_attack_damage = "$base_damage" },
            triggers = {
              on_attack_hit = {
                { op = "if", cond = "not event.target.is_building", actions = {
                  { op = "for_each",
                    select = { from = "event.target", shape = "circle", radius = "$splash_radius", team = "enemy", types = BASIC,
                      flags = { "include_magic_immune" } },
                    actions = {
                      { op = "damage", to = "it", type = "physical", amount = "event.attack.damage * $splash_pct" },
                    } },
                } },
              },
            },
            notes = "原作 modifier_C14T（C14T_old 版）：PHYSICAL_ARMOR_BONUS 10、MOVESPEED_BONUS_CONSTANT、ATTACK_RANGE_BONUS 572、HEALTH_BONUS、BASEATTACK_BONUSDAMAGE 20、MAGIC_IMMUNE；OnAttackLanded AON_Cleave_C14；OnDeath 移除自己，OnDestroy 改回近戰。沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c14t_old_cast_sound", at = "caster" },
            { op = "apply_status", status = "c14t_old_form", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "no_effect", ref = "vscripts/heroes/C_Neutral/C14.lua:84-102",
            text = "C14T_OnSpellStart 依序：音效、改遠程、套火龍模型、清空 caster.as_slow，接著 FindAbilityByName(\"C14E\") 在舊版找不到（舊版是 C14E_old），96 行對 nil 呼叫 GetLevel 報錯中斷，所以不換 C14E2；之後 KV 的 ApplyModifier modifier_C14T 是另一個動作，照常套用。" },
          { policy = "no_effect", ref = "vscripts/heroes/C_Neutral/C14.lua:104-114",
            text = "C14T_OnDestroy 先改回近戰，再找 C14E2 失敗報錯；改回近戰照常發生。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C14.lua:94",
            text = "化身期間不受攻擊速度緩速：施放時 caster.as_slow = {} 清空 nobu 緩速池的攻速部分，期間 Slow()（vscripts/library/common/dummy.lua:816-857）在目標有 modifier_C14T 時跳過 as_slow；資料用狀態旗標 attack_slow_immune（差異同本體：既有攻速緩速在原作是被刪掉，旗標只在期間忽略）。" },
          { policy = "as_source", ref = "vscripts/AON_Cleave.lua:274-298",
            text = "濺射以目標為圓心搜尋，沒有排除目標本身：主目標另外再吃 60% 的物理傷害（經護甲）；含魔免、排除 _G.EXCLUDE_TARGET_NAME；攻擊建築時不濺射。%attack_damage 推定為護甲前的攻擊傷害。" },
          { policy = "as_source", ref = "npc/C_Neutral/C14.txt:795-807",
            text = "和本體不同：舊版有魔法免疫（MAGIC_IMMUNE 沒註解）、生命上限加成、基礎攻擊力 +20、護甲固定 +10、攻擊距離 +572（本體 +472），普攻不留火窪。冷卻 165、耗魔 250～350、持續 40～60 秒。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C14.lua:24-60,93",
            text = "modifier_C14T_model 換火龍模型只是呈現（cue）；它宣告的 MOVESPEED_BASE_OVERRIDE 沒有對應函式，沒有效果。" },
        },
        provenance = { "npc/C_Neutral/C14.txt:760-861", "vscripts/heroes/C_Neutral/C14.lua:24-114", "vscripts/AON_Cleave.lua:274-298",
          "vscripts/library/common/dummy.lua:816-857" },
      },
    },
  }
end
