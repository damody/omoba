-- B27 小島貞興（current 版）— package schema v3。
-- current 版＝英雄實際出生時的技能：npc/B_Unified/B27_2.txt:16-22 的 B27W/B27E/B27R/B27D/B27T
-- （heromap_skill B27 ["16"] = "WERT"；choose_16 在 vscripts/library/events/eventforpichero.lua:63-73 被註解掉）。
-- 原作槽位 W/E/R/T → Q/W/E/R。B27D 雖列在 Ability4，但原作沒有任何地方定義 B27D（npc/、vscripts/ 都找不到），
-- 等於空欄，不列入。heromap_autoskill B27 ["16"] = ""。
-- _old（11 版）不做。英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B27_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（v3.md §12.3 推定）；引擎內建 modifier_stunned 寫 "strong"（同 A05）。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
-- nobu 緩速池（dummy.lua Slow／ReturnSpeed）照 A04 寫成 slow_pct 與 attack_speed 負值，緩速狀態帶 tag slow；合成與緩速抗性屬規則層。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B27",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    deferred = {
      { source_id = "B27D", reason = "not_in_source",
        note = "npc/B_Unified/B27_2.txt:19 列為 Ability4，但 npc/ 與 vscripts/ 都沒有 B27D 的定義，英雄出生時這一欄是空的。" },
    },
    unit_state = {
      tsum = { type = "fixed", init = "0", reveal = "public", scope = "match",
        note = "原作全域變數 tsum（沒有 local）：B27T 的補套 Timer 與 B30T 共用同一個值（登錄在 registry-additions-B）" },
    },
    hero = {
      source_id = "B27",
      text = { zh_TW = { name = "小島貞興" }, en = { name = "Kojima Sadaoki" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b27_horsemanship", source_id = "B27W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 23, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 85, 105, 125, 145, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 10, 13, 16, 19, unit = "s", source = "kv:buff_duration" },
          move_bonus = { 10, 13, 16, 19, unit = "pct",
            source = "kv:bonus_movement_speed", note = "FC-055：原作把它當 ms_slow 寫進緩速池（沒有加速）；依 D13 照原意給移速百分比" },
          attack_speed = { 150, unit = "atk_speed", source = "kv:bonus_attack_speed",
            note = "FC-055：原作只傳給緩速池的 as_slow，沒有效果；依 D13 照原意給攻速" },
        },
        text = {
          zh_TW = { name = "騎術",
            body = "強化自身 {duration} 秒：移動速度提高 {move_bonus}，攻擊速度提高 {attack_speed}。" },
          en = { name = "Horsemanship",
            body = "Empowers Sadaoki for {duration} seconds: +{move_bonus} movement speed and +{attack_speed} attack speed." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b27w_buff", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "騎術", body = "移動速度提高 {move_bonus}，攻擊速度提高 {attack_speed}。" },
              en = { name = "Horsemanship", body = "+{move_bonus} movement speed and +{attack_speed} attack speed." },
            },
            reveal = "public", cue = "b27w_armlet_glow",
            stats = { move_speed_pct = "$move_bonus", attack_speed = "$attack_speed" },
            notes = "原作 modifier_B27W_buff：IsBuff；OnCreated 以 dummy.lua Slow 寫 ms_slow／as_slow（名稱 modifier_B27W_buff），OnDestroy 以 ReturnSpeed 移除；沒寫 IsPurgable，暫定 basic。FC-055 修正後改成被註解掉的原意 Properties（移速百分比、攻速）。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "b27w_vo", at = "caster" } },
          on_cast = {
            { op = "apply_status", status = "b27w_buff", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/B_Unified/B27.txt:111-125",
            text = "FC-055（D13 修正，資料改成被註解掉的 Properties：+移速百分比 move_bonus、+攻速 attack_speed）。原作：Properties（+攻速、+移速百分比）已被註解，改成呼叫 dummy.lua Slow 並傳入正值 ms_slow 10/13/16/19、as_slow 150。緩速池（dummy.lua:866-901）把 ms_slow 取負號後以 1−Π(1−v) 合成，只有合成結果大於零才套用緩速；as_slow 只取最小（最負）的值。結果：正的 ms_slow 只能以乘法抵銷其他緩速、永遠不會變成加速；as_slow 150 完全沒有作用。技能名稱「騎術」與原作說明的加速、加攻速都沒有實現。" },
          { policy = "note", ref = "npc/B_Unified/B27.txt:59-63",
            text = "KV bonus_attack_speed 150 與 bonus_movement_speed 共用鍵 \"02\"；FC-055 修正後兩個都照原意用上。" },
          { policy = "note", ref = "npc/B_Unified/B27.txt:83-91",
            text = "ApplyModifier 的 Target 只寫 Center CASTER 與 Flags、沒有 Radius，原意是只掛在自己身上（同 C01W）。" },
          { policy = "note", ref = "npc/B_Unified/B27.txt:41",
            text = "KV 沒寫 AbilityCastPoint（IMMEDIATE）；OnAbilityPhaseStart 只播語音。" },
        },
        provenance = { "npc/B_Unified/B27.txt:34-137", "vscripts/library/common/dummy.lua:816-901" },
      },
      -------------------------------------------------------------------------
      { id = "b27_mighty_strike", source_id = "B27E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, 17, 14, 12, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 85, 110, 135, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          stun = { 1, 1.5, 2, 2.5, unit = "s", source = "kv:duration", note = "modifier_B27E OnCreated Stun" },
          projectile_speed = { 900, unit = "speed", hidden = true, source = "kv:TrackingProjectile MoveSpeed" },
          projectile_vision = { 225, unit = "dist", hidden = true, source = "kv:TrackingProjectile VisionRadius" },
          mark_time = { 0.2, unit = "s", hidden = true, source = "kv:modifier_B27E Duration" },
          follow_delay = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B27.lua:138 Timers:CreateTimer(0.1)" },
          phase_time = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B27.lua:141 modifier_phased duration" },
        },
        text = {
          zh_TW = { name = "剛力擊",
            body = "向目標敵人擲出追蹤斧（可被閃避），命中時造成 {damage} 魔法傷害並暈眩 {stun} 秒；隨後貞興瞬間移到目標所在位置並對它下達攻擊命令。目標在這段極短時間內死亡或被驅散命中標記時，不會移動。" },
          en = { name = "Mighty Strike",
            body = "Hurls a tracking axe at an enemy (dodgeable). On hit it deals {damage} magic damage and stuns for {stun} seconds; Sadaoki then instantly moves onto the target and is ordered to attack it. If the target dies or the hit mark is dispelled in that brief moment, he does not move." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b27e_mark", duration = "$mark_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "剛力擊", body = "被{@b27_mighty_strike}擊中；標記還在時，貞興會瞬間移到這個單位身上並攻擊它。" },
              en = { name = "Mighty Strike", body = "Hit by {@b27_mighty_strike}; while this mark remains, Sadaoki instantly moves onto this unit and attacks it." },
            },
            reveal = "public",
            notes = "原作 modifier_B27E：IsPurgable 1、IsDebuff、Duration 0.2；OnCreated 造成暈眩與傷害，之後只當成「0.1 秒後還在不在」的標記。" },
          { id = "b27e_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被{@b27_mighty_strike}暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Stunned by {@b27_mighty_strike} for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作 datadriven Stun 動作：引擎內建 modifier_stunned，受狀態抗性縮短。" },
          { id = "b27e_phase", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" },
            notes = "原作 AddNewModifier(nil, nil, modifier_phased, 0.1)：沒有施加者。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b27e_vo", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = true, vision = "$projectile_vision", cue = "b27e_axe",
              on_hit = {
                { op = "apply_status", status = "b27e_mark", to = "it" },
                { op = "apply_status", status = "b27e_stun", to = "it" },
                { op = "damage", require_source_alive = false, to = "it", type = "magical", amount = "$damage" },
                { op = "cue", id = "b27e_hit_sound", at = "it" },
                { op = "delay", time = "$follow_delay", actions = {
                  { op = "if", cond = "exists(it) and has_status(it, b27e_mark)", actions = {
                    { op = "teleport", who = "caster", to = "position(it)", find_clear_space = false },
                    { op = "apply_status", status = "b27e_phase", to = "caster" },
                    { op = "issue_order", who = "caster", order = "attack", target = "it" },
                  } },
                } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B27.lua:134-148",
            text = "命中 0.1 秒後，若目標仍有 modifier_B27E（0.2 秒標記），以 SetAbsOrigin 把施法者放到目標的位置（不找空地，兩者重疊），掛 0.1 秒 modifier_phased 並下令攻擊。不檢查施法者是否存活、被暈眩或與目標的距離，也不檢查目標是否隱形。" },
          { policy = "as_source", ref = "npc/B_Unified/B27.txt:170-179",
            text = "追蹤投射物 Dodgeable 1、ProvidesVision 225：目標閃避（例如瞬移）時不會命中，也就沒有暈眩、傷害與突進。" },
          { policy = "as_source", ref = "npc/B_Unified/B27.txt:198-222",
            text = "暈眩與傷害寫在 modifier_B27E 的 OnCreated（datadriven Stun 與 Damage），所以標記被驅散不影響已造成的暈眩；暈眩本身是 modifier_stunned。" },
          { policy = "note", text = "技能沒有 SpellImmunityType，不能對魔免敵人施放；施放後目標才變成魔免時，魔法傷害由規則層歸零。" },
        },
        provenance = { "npc/B_Unified/B27.txt:139-233", "vscripts/heroes/B_Unified/B27.lua:134-148" },
      },
      -------------------------------------------------------------------------
      { id = "b27_fearless_charge", source_id = "B27R", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 27, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 125, 150, 175, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 6, 7, 8, 9, unit = "s", source = "kv:duration" },
          attack_bonus = { 40, 50, 60, 70, unit = "dmg", source = "kv:attack_bonus", note = "modifier_C03w BASEATTACK_BONUSDAMAGE" },
        },
        text = {
          zh_TW = { name = "無畏衝鋒",
            body = "鼓起勇氣 {duration} 秒：基礎攻擊力 +{attack_bonus}，並獲得魔法免疫。施放時移除身上所有由敵方施加的狀態（不論能否驅散）。" },
          en = { name = "Fearless Charge",
            body = "Steels himself for {duration} seconds: +{attack_bonus} base attack damage and spell immunity. On cast, removes every status on Sadaoki that was applied by an enemy, dispellable or not." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b27r_courage", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "無畏衝鋒", body = "基礎攻擊力 +{attack_bonus}，魔法免疫。" },
              en = { name = "Fearless Charge", body = "+{attack_bonus} base attack damage and spell immunity." },
            },
            reveal = "public", cue = "b27r_b15w3_aura",
            stats = { base_attack_damage = "$attack_bonus" },
            states = { "magic_immune" },
            notes = "原作 modifier_C03w（與 C03 的 modifier 同名）：BASEATTACK_BONUSDAMAGE（白字）、MAGIC_IMMUNE；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b27r_vo", at = "caster" },
            { op = "apply_status", status = "b27r_courage", to = "caster" },
            { op = "cue", id = "b27r_model_grow", at = "caster", duration = "$duration" },
            { op = "remove_status", from = "caster", dispel = "all", team_of_source = "enemy" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B27.lua:118-132",
            text = "datadriven 先套 modifier_C03w，再由腳本逐一檢查身上所有 modifier：施加者存在且隊伍與自己不同就 RemoveModifierByName，不看可否驅散（含大招暈眩）；parent 隊伍的比較恆為假。施加者已不存在的敵方狀態不會被移除。原作說明沒有提到這個淨化（同 B15W）。" },
          { policy = "note", ref = "npc/B_Unified/B27.txt:281",
            text = "modifier 名稱 modifier_C03w 與 C03（npc/C_Neutral/C03.txt）相同，只是借用名稱；同名 RemoveModifierByName 會互相影響，資料以獨立狀態表達。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B27.lua:123",
            text = "AMHC:AddModelScale 放大模型 1.3 倍，只是呈現，改用 cue。KV 沒寫 AbilityCastPoint（IMMEDIATE）。" },
        },
        provenance = { "npc/B_Unified/B27.txt:235-313", "vscripts/heroes/B_Unified/B27.lua:118-132" },
      },
      -------------------------------------------------------------------------
      { id = "b27_rending_blade", source_id = "B27T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 95, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, 175, 200, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          radius = { 1000, unit = "dist", kv = { 1200 },
            source = "script:vscripts/heroes/B_Unified/B27.lua:157-159 FindUnitsInRadius 半徑寫死 1000",
            source_differs = "KV radius 與 AOERadius 是 1200，腳本實際搜尋 1000（AbilityCastRange 1000 只是指示）" },
          damage = { 400, 600, 800, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
          duration = { 5, 6, 7, unit = "s", source = "kv:duration" },
          slow = { 50, unit = "pct", kv = { -50 },
            source = "kv:speed_bonus", note = "modifier_B27T OnCreated 以 dummy.lua Slow 的 ms_slow 寫進 nobu 緩速池",
            source_differs = "KV 以負值表示減速；slow_pct 以正值表示緩速" },
          attack_slow = { 50, unit = "atk_speed", kv = { -50 },
            source = "kv:attack_bonus", note = "dummy.lua Slow 的 as_slow，nobu 緩速池的攻擊速度",
            source_differs = "KV 以負值表示降低；資料以正值記錄、行為取負號" },
          damage_reduction = { 35, 50, 65, unit = "pct", kv = { -35, -50, -65 },
            source = "kv:damage_bonus", note = "modifier_B27T_2 INCOMING_DAMAGE_PERCENTAGE",
            source_differs = "KV 以負值表示減傷；資料以正值記錄、行為取負號" },
          reapply_delay = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B27.lua:168-178 Timers:CreateTimer(0.1)、tsum = 0.1" },
        },
        text = {
          zh_TW = { name = "裂地刃",
            body = "猛擊地面，對自身周圍 {radius} 內的敵方英雄與部隊（包含魔免）造成 {damage} 魔法傷害，並使其移動速度降低 {slow}、攻擊速度降低 {attack_slow}，持續 {duration} 秒；若英雄的緩速在一瞬間內被移除，會再套上一次。同時自身受到的傷害降低 {damage_reduction}，持續 {duration} 秒。" },
          en = { name = "Rending Blade",
            body = "Slams the ground, dealing {damage} magic damage to enemy heroes and units within {radius} (spell-immune ones included) and slowing their movement speed by {slow} and attack speed by {attack_slow} for {duration} seconds; if a hero's slow is removed within a moment, it is applied once more. Sadaoki also takes {damage_reduction} less damage for {duration} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b27t_slow", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "裂地刃", body = "移動速度降低 {slow}，攻擊速度降低 {attack_slow}。" },
              en = { name = "Rending Blade", body = "Movement speed reduced by {slow} and attack speed reduced by {attack_slow}." },
            },
            reveal = "public", tags = { "slow" }, cue = "b27t_move_slow",
            stats = { slow_pct = "$slow", attack_speed = "-$attack_slow" },
            notes = "原作 modifier_B27T：IsDebuff、IsPurgable 1；OnCreated 以 dummy.lua Slow 寫進 nobu 緩速池（名稱 modifier_B27T），OnDestroy 以 ReturnSpeed 移除。施加者是目標自己（ApplyDataDrivenModifier(it, it, ...)）。" },
          { id = "b27t_guard", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "裂地刃", body = "受到的傷害降低 {damage_reduction}。" },
              en = { name = "Rending Blade", body = "Takes {damage_reduction} less damage." },
            },
            reveal = "public",
            stats = { incoming_damage_pct = "-$damage_reduction" },
            notes = "原作 modifier_B27T_2：Passive 1、IsPurgable 0、Duration %duration、INCOMING_DAMAGE_PERCENTAGE %damage_bonus。" },
        },
        events = {
          on_upgrade = {
            { op = "apply_status", status = "b27t_guard", to = "caster" },
          },
          on_cast = {
            { op = "cue", id = "b27t_vo", at = "caster" },
            { op = "cue", id = "b27t_title" },
            { op = "cue", id = "b27t_slam", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                flags = { "include_magic_immune" } },
              actions = {
                { op = "if", cond = "it.is_hero", actions = { { op = "cue", id = "b27t_screen_shake", at = "it" } } },
                { op = "damage", to = "it", type = "magical", amount = "$damage" },
                { op = "apply_status", status = "b27t_slow", to = "it", source = "it" },
                { op = "delay", time = "$reapply_delay", actions = {
                  { op = "if", cond = "it.is_hero and exists(it) and not has_status(it, b27t_slow)", actions = {
                    { op = "apply_status", status = "b27t_slow", to = "it", source = "it", duration = "$duration - state.tsum" },
                  } },
                } },
              } },
            { op = "set_state", var = "tsum", value = "$reapply_delay" },
            -- 原作同一個 Timer 在補套完所有英雄後才 tsum + 0.1；之後排程的 delay 在同一步較晚執行
            { op = "delay", time = "$reapply_delay", actions = {
              { op = "set_state", var = "tsum", value = "state.tsum + $reapply_delay" },
            } },
            { op = "apply_status", status = "b27t_guard", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B27.lua:150-179",
            text = "實際搜尋半徑寫死 1000，KV radius／AOERadius 1200 沒有用到；傷害是 KV dmg 400/600/800，AbilityDamage 400 沒有用到。魔免敵人照樣被選到（MAGIC_IMMUNE_ENEMIES），魔法傷害由規則層歸零，但緩速照樣套用。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B27.lua:166",
            text = "緩速以 ApplyDataDrivenModifier(it, it, ...) 套用，施加者是目標自己：不受狀態抗性縮短（events.lua 只縮短敵方施加的狀態），也不會被「移除敵方施加的狀態」類淨化（B15W、B27R 等）移除，但一般驅散可以移除。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B27.lua:168-178",
            text = "0.1 秒的 Timer 沒有 return，只執行一次：此時已失去緩速的英雄（例如施放瞬間被驅散）再套上 duration − 0.1 秒；一般單位不補套。tsum 是全域變數，施放時設為 0.1、補套後加 0.1；同一瞬間有 B30T 的點燃 Timer 在跑時，補套時間會被改到。資料以 scope = match 的 unit_state tsum 照原作表達。" },
          { policy = "as_source", ref = "npc/B_Unified/B27.txt:410-420",
            text = "modifier_B27T_2 標了 Passive 1：推定學會與每次升級時（Dota 重建被動 modifier）也會得到一段同樣持續時間的減傷，資料以 on_upgrade 表達。重生時是否重新套上未驗證，資料不補。" },
          { policy = "note", ref = "npc/B_Unified/B27.txt:316",
            text = "原作註解「減傷變成永久被動 or 點招的幾秒內才有減傷．開大無減傷」是設計討論，實際行為以 KV 與腳本為準。" },
          { policy = "note", ref = "npc/B_Unified/B27.txt:380-404",
            text = "modifier_B27T 的 Properties 版緩速已註解，實際走 nobu 緩速池；目標有 modifier_C14T 時不寫攻速緩速、緩速抗性折算都屬規則層（dummy.lua:827-857）。" },
          { policy = "note", text = "showTitle、震動特效、b27t 粒子與音效都屬呈現，改用 cue。KV 沒寫 AbilityCastPoint。" },
        },
        provenance = { "npc/B_Unified/B27.txt:315-456", "vscripts/heroes/B_Unified/B27.lua:150-179",
          "vscripts/library/common/dummy.lua:816-901" },
      },
    },
  }
end
