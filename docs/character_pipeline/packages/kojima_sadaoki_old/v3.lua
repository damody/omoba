-- B27 小島貞興（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["B27"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:306）；heromap_autoskill["B27"]["11"] = ""
-- （addon_game_mode.lua:409），沒有 D。W/E/R/T → Q/W/E/R。
-- 原作舊版有繁中說明（resource/addon_tchinese.txt:3667-3674），與 KV、腳本的差異記在 quirks。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、B27_precache 屬規則層或資產，不列入。
-- 慣例照本體 B27：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- datadriven Damage（不經 AMHC）在貞興死亡後仍會造成傷害，寫 require_source_alive = false；HERO | BASIC 寫成 BASIC 清單。
-- 和本體不同：舊版騎術與天崩地裂直接用 Properties（不經 nobu 緩速池），所以寫 move_speed_pct／attack_speed，不帶 slow tag。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B27_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B27", text = { zh_TW = { name = "小島貞興（舊版）" }, en = { name = "Kojima Sadaoki (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b27_old_horsemanship", source_id = "B27W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 23, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B27.txt:468" },
          mana_cost = { 85, 105, 125, 145, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B27.txt:469" },
          duration = { 14, 16, 18, 20, unit = "s", source = "kv:buff_duration@npc/B_Unified/B27.txt:477" },
          move_bonus = { 25, 30, 35, 40, unit = "pct", source = "kv:bonus_movement_speed@npc/B_Unified/B27.txt:482" },
          attack_speed = { 40, 60, 80, 100, unit = "atk_speed", source = "kv:bonus_attack_speed@npc/B_Unified/B27.txt:487" },
        },
        text = {
          zh_TW = { name = "騎術",
            body = "強化自身 {duration} 秒：攻擊速度提高 {attack_speed}，移動速度提高 {move_bonus}。" },
          en = { name = "Horsemanship",
            body = "Empowers Sadaoki for {duration} seconds: +{attack_speed} attack speed and +{move_bonus} movement speed." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b27w_old_buff", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "騎術", body = "攻擊速度提高 {attack_speed}，移動速度提高 {move_bonus}。" },
              en = { name = "Horsemanship", body = "+{attack_speed} attack speed and +{move_bonus} movement speed." },
            },
            reveal = "public", cue = "b27w_old_armlet_glow",
            stats = { move_speed_pct = "$move_bonus", attack_speed = "$attack_speed" },
            notes = "原作 modifier_B27W_buff：IsBuff、Duration %buff_duration；Properties ATTACKSPEED_BONUS_CONSTANT、MOVESPEED_BONUS_PERCENTAGE；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "b27w_old_vo", at = "caster" } },
          on_cast = {
            { op = "apply_status", status = "b27w_old_buff", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B27.txt:536-542",
            text = "和本體不同：舊版直接用 Properties（攻速常數、移速百分比），沒有經過 dummy.lua 緩速池，所以加速確實生效（本體的 FC-055 不存在於舊版）。" },
          { policy = "as_source", ref = "resource/addon_tchinese.txt:3668",
            text = "原作說明寫「增加 (40/60/80/100)% 的攻擊」，實際是攻擊速度常數 +40/60/80/100（ATTACKSPEED_BONUS_CONSTANT），不是攻擊力百分比。說明照實際改寫。" },
          { policy = "note", ref = "npc/B_Unified/B27.txt:478-488",
            text = "AbilitySpecial 的 bonus_movement_speed 與 bonus_attack_speed 都用 \"02\" 當鍵；推定 KV 保留重複鍵、GetSpecialValueFor／%變數依名稱查找，兩個都生效（同 B08R_old 的判斷）。" },
          { policy = "note", ref = "npc/B_Unified/B27.txt:512-519",
            text = "ApplyModifier 的 Target 只有 Center CASTER 與 Flags、沒有 Radius，只掛在自己身上。KV 沒寫 AbilityCastPoint（IMMEDIATE）；OnAbilityPhaseStart 只播 C05W.vo1 語音。" },
        },
        provenance = { "npc/B_Unified/B27.txt:459-545", "resource/addon_tchinese.txt:3667-3668" },
      },
      -------------------------------------------------------------------------
      { id = "b27_old_mighty_strike", source_id = "B27E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, 13, 11, 9, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B27.txt:557" },
          mana_cost = { 85, 110, 135, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B27.txt:558" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B27.txt:559" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B27.txt:555" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/B_Unified/B27.txt:634" },
          stun = { 1, 1.5, 2, 2.5, unit = "s", source = "kv:duration@npc/B_Unified/B27.txt:631" },
          projectile_speed = { 900, unit = "speed", hidden = true, source = "kv:TrackingProjectile MoveSpeed@npc/B_Unified/B27.txt:584" },
          projectile_vision = { 225, unit = "dist", hidden = true, source = "kv:TrackingProjectile VisionRadius@npc/B_Unified/B27.txt:585" },
        },
        text = {
          zh_TW = { name = "剛力燄",
            body = "向目標敵人擲出追蹤斧（可被閃避），命中時造成 {damage} 魔法傷害並暈眩 {stun} 秒。" },
          en = { name = "Mighty Blaze",
            body = "Hurls a tracking axe at an enemy (dodgeable). On hit it deals {damage} magic damage and stuns for {stun} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b27e_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被{@b27_old_mighty_strike}暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Stunned by {@b27_old_mighty_strike} for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作 modifier_B27E_old OnCreated 的 datadriven Stun：引擎內建 modifier_stunned，受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b27e_old_vo", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = true, vision = "$projectile_vision", cue = "b27e_old_axe",
              on_hit = {
                { op = "apply_status", status = "b27e_old_stun", to = "it" },
                { op = "damage", require_source_alive = false, to = "it", type = "magical", amount = "$damage" },
                { op = "cue", id = "b27e_old_hit_sound", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B27.txt:597-624",
            text = "暈眩與傷害寫在 modifier_B27E_old 的 OnCreated（Duration 0、IsPurgable 1）；這個 modifier 本身沒有屬性也沒有後續效果，資料不另建狀態，命中時直接暈眩與傷害。和本體不同：舊版沒有命中後瞬移到目標身旁並攻擊的腳本。" },
          { policy = "as_source", ref = "npc/B_Unified/B27.txt:576-587",
            text = "追蹤投射物 Dodgeable 1、ProvidesVision 225：目標閃避（例如瞬移）時不會命中。" },
          { policy = "note", ref = "resource/addon_tchinese.txt:3669",
            text = "原作說明名稱是「剛力燄」（本體是「剛力擊」），照原作舊版名稱。說明沒提到可被閃避。技能沒有 SpellImmunityType，不能對魔免敵人施放。" },
        },
        provenance = { "npc/B_Unified/B27.txt:547-635", "resource/addon_tchinese.txt:3669-3670" },
      },
      -------------------------------------------------------------------------
      { id = "b27_old_spell_shield", source_id = "B27R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B27.txt:648（腳本 StartCooldown(GetCooldown(-1))）" },
          max_stacks = { 2, 4, 6, 8, unit = "count",
            source = "script:vscripts/heroes/B_Unified/B27.lua:108 handle:GetStackCount() < ability:GetLevel()*2" },
          think_interval = { 0.1, unit = "s", hidden = true, source = "kv:ThinkInterval@npc/B_Unified/B27.txt:668" },
          snapshot_interval = { 0.2, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B27.lua:17 StartIntervalThink(0.2)" },
          purge_delay = { 0.01, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B27.lua:40 Timers:CreateTimer(0.01)" },
        },
        text = {
          zh_TW = { name = "法術護盾",
            body = "被動：技能冷卻完畢時自動累積一層護盾，最多 {max_stacks} 層，每層之間需要冷卻。受到敵人造成的非物理傷害時，消耗一層，把生命與法力回復到片刻前的數值，並在瞬間後移除身上由敵方施加的狀態。" },
          en = { name = "Spell Shield",
            body = "Passive: whenever the ability is off cooldown it gains a shield charge, up to {max_stacks} charges, with the cooldown between charges. When an enemy deals non-physical damage to Sadaoki, one charge is consumed, his health and mana are restored to their values a moment earlier, and an instant later the statuses enemies applied to him are removed." },
        },
        target = { type = "none" },
        cost = { cooldown = "$cooldown", cooldown_start = "manual" },
        statuses = {
          { id = "b27r_old_shield", duration = "infinite", stacking = { max = "$max_stacks" }, dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "法術護盾", body = "受到敵人的非物理傷害時消耗一層，回復生命與法力並移除敵方施加的狀態。" },
              en = { name = "Spell Shield", body = "Enemy non-physical damage consumes a charge, restoring health and mana and removing enemy-applied statuses." },
            },
            reveal = "public", cue = "b27r_old_shield_bubble",
            vars = {
              hp = { type = "fixed", init = "self.hp", reveal = "public" },
              mp = { type = "fixed", init = "self.mana", reveal = "public" },
            },
            triggers = {
              periodic = { interval = "$snapshot_interval", actions = {
                { op = "set_var", var = "hp", value = "self.hp" },
                { op = "set_var", var = "mp", value = "self.mana" },
              } },
              on_take_damage = {
                filter = "exists(event.attacker) and is_enemy(event.attacker, self) and not event.damage.is_reflection and not event.damage.is_physical",
                actions = {
                  { op = "if", cond = "self.alive", actions = {
                    { op = "set_health", to = "self", value = "vars.hp" },
                    { op = "set_mana", to = "self", value = "vars.mp" },
                    { op = "cue", id = "b27r_old_block", at = "self" },
                  } },
                  { op = "delay", time = "$purge_delay", actions = {
                    { op = "if", cond = "exists(self)", actions = {
                      { op = "remove_status", from = "self", dispel = "all", team_of_source = "enemy",
                        exclude_tags = { "cleanse_protected" } },
                      { op = "if", cond = "stacks(self, b27r_old_shield) > 1",
                        actions = { { op = "add_stacks", status = "b27r_old_shield", to = "self", count = -1 } },
                        otherwise = { { op = "remove_status", from = "self", status = "b27r_old_shield" } } },
                    } },
                  } },
                } },
            },
            notes = "原作 modifier_protection_b27r_old（Lua modifier，以 ApplyDataDrivenModifier 套用，同御守推定可套上）：每 0.2 秒記錄生命與法力；OnTakeDamage 在扣血後把生命、法力設回記錄值，0.01 秒後淨化並減一層。" },
        },
        passive = {
          reveal = "public",
          triggers = {
            periodic = { interval = "$think_interval", actions = {
              { op = "if", cond = "not has_status(self, b27r_old_shield)",
                actions = {
                  { op = "if", cond = "ability.is_ready", actions = {
                    { op = "apply_status", status = "b27r_old_shield", to = "self", stacks = 1 },
                    { op = "modify_cooldown", of = "self", ability = "b27_old_spell_shield", mode = "start", amount = "$cooldown" },
                  } },
                },
                otherwise = {
                  { op = "if", cond = "stacks(self, b27r_old_shield) < $max_stacks and ability.is_ready", actions = {
                    { op = "add_stacks", status = "b27r_old_shield", to = "self", count = 1 },
                    { op = "modify_cooldown", of = "self", ability = "b27_old_spell_shield", mode = "start", amount = "$cooldown" },
                  } },
                } },
            } },
          },
        },
        coverage = "pending", confidence = "medium",
        pending = {
          { what = "淨化時也要移除「受保護但剛套上不到 0.1 秒」的敵方狀態",
            why = "vscripts/heroes/B_Unified/B27.lua:48-53：_G.EXCLUDE_MODIFIER_NAME 名單內的狀態平常保留，但 GetElapsedTime() < 0.1 時照樣移除（例如同一招帶來的變青蛙）",
            gap = "remove_status 加 max_elapsed（只移除已存在時間小於某值的狀態），或可寫 where 條件讀 status.elapsed" },
        },
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B27.lua:27-37",
            text = "OnTakeDamage 的變數名稱顛倒（attacker = event.unit 是受害者、victim = event.attacker 是攻擊者），實際條件是：受害者是貞興、攻擊者是敵人、不是反射傷害、傷害類型不是物理（純粹傷害也算）。普通攻擊不觸發。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B27.lua:20-25,64-72",
            text = "不是擋下傷害，而是在扣血之後把生命與法力設回最近一次（每 0.2 秒）的記錄值：這段時間內受到的其他傷害（含物理）也會一併被抹掉，期間的回復則被吃掉；致死的一擊已經殺死貞興，擋不住。同一瞬間多筆傷害每筆都各自回復並各消耗一層。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B27.lua:42",
            text = "Purge(false, true, true, true, true) 的第三個參數是 BuffsCreatedThisFrameOnly，只移除這一幀建立的負面狀態，幾乎沒有作用；實際淨化是後面的迴圈：移除施加者或持有者隊伍與貞興不同的狀態（不論能否驅散），_G.EXCLUDE_MODIFIER_NAME 名單內的保留（cleanse_protected）。施加者已不存在的敵方狀態不會被移除。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B27.lua:84-112",
            text = "被動每 0.1 秒檢查：沒有護盾且技能冷卻完畢就給一層並開始冷卻（8 秒）；已有護盾且層數小於等級×2、冷卻完畢就加一層並開始冷卻。冷卻是累積層數用的，消耗層數不會觸發冷卻；用完所有層後下一次冷卻完畢才重新給一層。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B27.lua:61",
            text = "最後一層用完時 caster.B27R_effect = nil 的 caster 是未定義的全域變數，執行期報錯；此時 modifier 與特效都已移除，沒有影響結算。" },
          { policy = "note", ref = "npc/B_Unified/B27.txt:652-658",
            text = "KV 的 stun_time 0.5/1/1.5/2 沒有任何地方使用，不列入。原作說明「最高連續抵擋 (2/4/6/8) 次」與腳本相符。" },
        },
        provenance = { "npc/B_Unified/B27.txt:637-681", "vscripts/heroes/B_Unified/B27.lua:1-112",
          "vscripts/library/common/dummy.lua:376-392", "resource/addon_tchinese.txt:3671-3672" },
      },
      -------------------------------------------------------------------------
      { id = "b27_old_earth_shatter", source_id = "B27T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 95, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B27.txt:696" },
          mana_cost = { 150, 175, 200, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B27.txt:697" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades@npc/B_Unified/B27.txt:694-695" },
          radius = { 1000, unit = "dist", kv = { 1200 },
            source = "script:vscripts/heroes/B_Unified/B27.lua:188-190 FindUnitsInRadius 半徑寫死 1000",
            source_differs = "KV radius／AOERadius 與原作說明是 1200，腳本實際搜尋 1000（AbilityCastRange 1000 只是指示）" },
          damage = { 700, 900, 1100, unit = "dmg", damage_type = "magical", source = "kv:dmg@npc/B_Unified/B27.txt:772" },
          duration = { 8, 9, 10, unit = "s", source = "kv:duration@npc/B_Unified/B27.txt:767" },
          attack_slow = { 46, 51, 56, unit = "atk_speed", kv = { -46, -51, -56 },
            source = "kv:speed_bonus@npc/B_Unified/B27.txt:757（modifier_B27T ATTACKSPEED_BONUS_CONSTANT）",
            source_differs = "KV 以負值表示降低；資料以正值記錄、行為取負號" },
          slow = { 46, 51, 56, unit = "pct", kv = { -46, -51, -56 },
            source = "kv:attack_bonus@npc/B_Unified/B27.txt:762（modifier_B27T MOVESPEED_BONUS_PERCENTAGE）",
            source_differs = "KV 以負值表示降低；資料以正值記錄、行為取負號" },
          reapply_delay = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B27.lua:199-208 Timers:CreateTimer(0.1)、tsum = 0.1" },
        },
        text = {
          zh_TW = { name = "天崩地裂",
            body = "敲擊地面，對自身周圍 {radius} 內的敵方英雄與部隊（包含魔免）造成 {damage} 魔法傷害，並降低他們 {slow} 移動速度與 {attack_slow} 攻擊速度，持續 {duration} 秒；若英雄的減速在一瞬間內被移除，會再套上一次。" },
          en = { name = "Heaven Falls, Earth Splits",
            body = "Slams the ground, dealing {damage} magic damage to enemy heroes and units within {radius} (spell-immune ones included) and reducing their movement speed by {slow} and attack speed by {attack_slow} for {duration} seconds; if a hero's slow is removed within a moment, it is applied once more." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b27t_old_slow", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "天崩地裂", body = "移動速度降低 {slow}，攻擊速度降低 {attack_slow}。" },
              en = { name = "Heaven Falls, Earth Splits", body = "Movement speed reduced by {slow} and attack speed reduced by {attack_slow}." },
            },
            reveal = "public", cue = "b27t_old_move_slow",
            stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            notes = "原作 modifier_B27T（B27T_old 的 Modifiers 區塊）：IsDebuff、IsPurgable 1；Properties ATTACKSPEED_BONUS_CONSTANT %speed_bonus、MOVESPEED_BONUS_PERCENTAGE %attack_bonus。施加者是目標自己（ApplyDataDrivenModifier(it, it, ...)）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b27t_old_vo", at = "caster" },
            { op = "cue", id = "b27t_old_title" },
            { op = "cue", id = "b27t_old_slam", at = "caster", radius = "$radius" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                flags = { "include_magic_immune" } },
              actions = {
                { op = "if", cond = "it.is_hero", actions = { { op = "cue", id = "b27t_old_screen_shake", at = "it" } } },
                { op = "damage", to = "it", type = "magical", amount = "$damage" },
                { op = "apply_status", status = "b27t_old_slow", to = "it", source = "it" },
                { op = "delay", time = "$reapply_delay", actions = {
                  { op = "if", cond = "it.is_hero and exists(it) and not has_status(it, b27t_old_slow)", actions = {
                    { op = "apply_status", status = "b27t_old_slow", to = "it", source = "it", duration = "$duration - $reapply_delay" },
                  } },
                } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B27.lua:181-209",
            text = "實際搜尋半徑寫死 1000，KV radius／AOERadius 與原作說明的 1200 沒有用到。傷害讀 KV dmg（AMHC:Damage）。魔免敵人照樣被選到（MAGIC_IMMUNE_ENEMIES），魔法傷害由規則層歸零，但減速照樣套用。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B27.lua:197",
            text = "減速以 ApplyDataDrivenModifier(it, it, ...) 套用，施加者是目標自己：不受狀態抗性縮短，也不會被「移除敵方施加的狀態」類淨化移除，但一般驅散可以移除。" },
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B27.lua:199-208",
            text = "0.1 秒的 Timer 沒有 return，只執行一次：這時已失去減速的英雄再套上 duration − tsum 秒；一般單位不補套。tsum 是全域變數（施放時設 0.1，補套後加 0.1），只有在這 0.1 秒內另有 B27T／B27T_old／B30T 施放時才會被改到。資料照設計意圖固定用 duration − 0.1，不建跨英雄共用的 tsum。" },
          { policy = "note", ref = "npc/B_Unified/B27.txt:734-745",
            text = "KV 的 speed_bonus 接到攻速、attack_bonus 接到移速，名稱顛倒但兩者數值相同，沒有影響。舊版沒有本體的自身減傷（modifier_B27T_2）。AbilityDamage 與 dmg 同值，腳本讀 dmg。" },
          { policy = "note", text = "showTitle、震動特效、b27t 粒子與 Roshan.Slam 音效都屬呈現，改用 cue。KV 沒寫 AbilityCastPoint；AnimationPlaybackRate 3 只是動作。" },
        },
        provenance = { "npc/B_Unified/B27.txt:683-776", "vscripts/heroes/B_Unified/B27.lua:181-209",
          "resource/addon_tchinese.txt:3673-3674" },
      },
    },
  }
end
