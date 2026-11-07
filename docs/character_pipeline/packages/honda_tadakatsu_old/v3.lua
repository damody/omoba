-- A07 本多忠勝（舊版，原作 11 版「_old」技能組）— package schema v3。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 _G.heromap_skill.A07["11"] = "WERT" 逐一 AddAbility(A07<鍵>_old)
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-131，vscripts/addon_game_mode.lua:249）；
-- heromap_autoskill.A07["11"] = ""（vscripts/addon_game_mode.lua:352），不自動學技能。舊版沒有 D（A07D 只在 16 版）。
-- 原作槽位 W/E/R/T → Q/W/E/R。2.0 版（A07*_20）另成一份文件，不在這裡。
-- 原作沒有繁中說明（只有日文 addon_japanese.txt:1122-1129 與赤雷擊的 modifier 名稱），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A07_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；Lua modifier 預設不可驅散，寫 "none"（v3.md §12.3，推定）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A07_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A07", text = { zh_TW = { name = "本多忠勝（舊版）" }, en = { name = "Honda Tadakatsu (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a07_old_unscathed", source_id = "A07W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 30, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 250, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 3, 4, 5, 6, unit = "s", source = "kv:duration" },
          physical_cut = { 300, unit = "pct", hidden = true, kv = { -300 },
            source = "kv:modifier_a07w_old_borrowed_time MODIFIER_PROPERTY_INCOMING_PHYSICAL_DAMAGE_PERCENTAGE",
            source_differs = "KV 以負值表示減傷，v3 以正值表示；−300% 使物理傷害歸零" },
          slow = { 40, unit = "pct", kv = { -40 },
            source = "kv:modifier_a07w_old_borrowed_time MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE",
            source_differs = "KV 以負值寫移速加成；v3 存減速量，行為取負" },
          convert = { 30, unit = "pct",
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:36-38 非物理傷害 SetHealth(hp + original_damage × 0.3)" },
        },
        text = {
          zh_TW = { name = "毫髮無傷",
            body = "持續 {duration} 秒：免疫物理傷害，但自身移動速度降低 {slow}。期間受到敵方的魔法或純粹傷害時，該次傷害無效（致死的一擊也擋得住），並回復該次原始傷害的 {convert}。" },
          en = { name = "Unscathed",
            body = "For {duration} seconds Tadakatsu is immune to physical damage, but his movement speed is reduced by {slow}. Magic or pure damage from enemies during this time is negated (even a lethal blow), and he recovers {convert} of that damage's original amount." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a07w_old_guard", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "a07w_old_shield",
            stats = { incoming_physical_pct = "-$physical_cut", move_speed_pct = "-$slow" },
            text = { zh_TW = { name = "毫髮無傷", body = "免疫物理傷害，移動速度降低 {slow}。" },
              en = { name = "Unscathed", body = "Immune to physical damage; movement speed reduced by {slow}." } },
            triggers = {
              on_removed = { { op = "remove_status", from = "self", status = "a07w_old_convert" } },
            },
            notes = "原作 modifier_a07w_old_borrowed_time：IsBuff（不隱藏）、INCOMING_PHYSICAL_DAMAGE_PERCENTAGE −300、MOVESPEED_BONUS_PERCENTAGE −40；OnDestroy／OnDeath 呼叫 A07W_EndShieldParticle 移除 modifier_A07W。沒寫 IsPurgable，依 §12.3 暫定 basic。" },
          { id = "a07w_old_convert", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              -- FC-025（比照本體 D13）：結算前取消敵方的非物理傷害（致死的一擊也擋得住），再回復原始傷害的 convert
              on_take_damage_pre = {
                filter = "is_enemy(event.attacker, self) and not event.damage.is_physical",
                actions = {
                  { op = "modify_damage", cancel = true },
                  { op = "set_health", to = "self", value = "min(self.hp + event.damage.raw * $convert, self.max_hp)" },
                } },
            },
            notes = "原作 modifier_A07W（Lua，由 A07W_SE 以 ApplyDataDrivenModifier 套上，持續 = %duration）：每 0.1 秒記下生命，OnTakeDamage 對敵方的非物理傷害 SetHealth(記下的生命 + original_damage × 0.3)。FC-025 改成結算前取消。" },
        },
        events = {
          on_cast = {
            { op = "call", routine = "a07w_old_cast", args = {} },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:123-130",
            text = "A07W_SE 以 ApplyDataDrivenModifier 套用 Lua modifier modifier_A07W（不在 A07W_old 的 Modifiers 裡）；與本體 A07W 相同寫法，依本體的推定視為引擎接受。若不接受，FindModifierByName 回傳 nil 使腳本中斷，只剩物理免疫與減速。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27-46",
            text = "FC-025 D22 舊版，比照 D13 修：原作回寫生命在 OnTakeDamage（受傷之後），一次就致死的魔法或純粹傷害照樣擊殺本多忠勝，毫髮無傷擋不住。修法：改用 on_take_damage_pre，結算前取消敵方的非物理傷害，再把生命設為目前生命 + 原始傷害 × convert（截在最大生命；原作 SetHealth 同樣受最大生命限制）。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:17-25",
            text = "原作的「受傷前生命」是每 0.1 秒取樣一次的快照，取樣後到受傷前的治療會被回寫抹掉；照意圖以結算前取消後的目前生命為準（等於這次傷害之前的生命）。物理傷害的 SetHealth(記下的生命) 在物理傷害歸零下只會抹掉取樣後的治療，照意圖不寫。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:29-36",
            text = "只處理敵隊來源；回復量用 original_damage（減免前），所以帶魔抗時回寫後的生命比受傷前多。用 SetHealth，不受治療增減影響。" },
          { policy = "as_source", ref = "npc/A_Oda/A07.txt:318-340",
            text = "舊版施放時不先呼叫 A07W_EndShieldParticle（本體會），重施時兩個狀態各自刷新。被驅散時 OnDestroy 一併移除回寫狀態。" },
          { policy = "note", ref = "npc/A_Oda/A07.txt:376-405",
            text = "KV hp_threshold、duration_scepter、redirect、redirect_range_tooltip_scepter 沒有被使用。日文說明「法術による被ダメージを生命値に転化」即本效果。" },
        },
        provenance = { "npc/A_Oda/A07.txt:290-406", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:1-46",
          "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:123-163" },
      },
      -------------------------------------------------------------------------
      { id = "a07_old_red_thunder", source_id = "A07E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 200, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:duration" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
        },
        text = {
          zh_TW = { name = "赤雷擊",
            body = "以巨槍猛擊目標敵人，使其暈眩 {stun} 秒並造成 {damage} 魔法傷害。" },
          en = { name = "Red Thunder Strike",
            body = "Smashes the target enemy with the great spear, stunning it for {stun} seconds and dealing {damage} magic damage." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a07e_old_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", states = { "stun" }, cue = "a07e_old_stunned_overhead",
            text = { zh_TW = { name = "赤雷擊", body = "被如赤雷般的攻擊打暈，暈眩 {stun} 秒。" },
              en = { name = "Red Thunder Strike", body = "Stunned by a thunderous attack for {stun} seconds." } },
            notes = "原作 modifier_A07E_old：IsDebuff、IsStunDebuff、STUNNED；OnCreated 造成 %AbilityDamage 魔法傷害。沒寫 IsPurgable，依 §12.3 暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "call", routine = "a07e_old_cast", args = { victim = "target" } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A07.txt:781-797",
            text = "傷害寫在暈眩 modifier 的 OnCreated：先暈再傷害。冷卻 10 秒長於暈眩，不會在暈眩中重新套用。SPELL_IMMUNITY_ENEMIES_NO：不能對魔免目標施放。" },
          { policy = "note", ref = "npc/A_Oda/A07.txt:753-776",
            text = "Hero_EarthShaker.Gravelmaw 音效與 A07E_SE 粒子只是呈現。英文 modifier 名稱 Red Thunder Strike 取自 addon_english.txt:1630。" },
        },
        provenance = { "npc/A_Oda/A07.txt:718-818", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:195-203" },
      },
      -------------------------------------------------------------------------
      { id = "a07_old_shattering_spear", source_id = "A07R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          crit_chance = { "3100/101", unit = "pct", kv = { 30 },
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:86,91 RandomInt(0,100) <= 30",
            source_differs = "RandomInt(0,100) ≤ 30 是 31/101，約 30.69%；日文說明寫 36%" },
          misses = { 4, unit = "count",
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:91-94 A07R_noncrit_count > 4 必中（連續 4 次沒暴擊，第 5 次必中）" },
          crit_multiplier = { 120, 140, 160, 180, unit = "pct", source = "kv:crit_persent",
            note = "日文說明寫 1.4/1.6/1.8/2 倍；KV 實際 120/140/160/180%" },
        },
        text = {
          zh_TW = { name = "碎擊槍",
            body = "被動：每次攻擊出手時有 {crit_chance} 機率暴擊，造成 {crit_multiplier} 傷害；連續 {misses} 次沒有暴擊時，下一次必定暴擊。" },
          en = { name = "Shattering Spear",
            body = "Passive: each attack has a {crit_chance} chance to critically strike for {crit_multiplier} damage; after {misses} attacks in a row without a critical strike, the next one always crits." },
        },
        target = { type = "none" },
        unit_state = {
          a07r_old_noncrit = { type = "int", init = "0", reveal = "team", scope = "hero",
            note = "原作 caster.A07R_noncrit_count：連續沒有暴擊的次數（FC-046 修正後真的累計）" },
        },
        passive = {
          reveal = "public", visibility = "hidden",
          triggers = {
            -- FC-046（比照本體 D13）：連續 misses 次沒有暴擊，下一次必定暴擊（原作計數寫成 0 or count，永遠歸零）
            on_attack_windup = {
              { op = "if", cond = "state.a07r_old_noncrit >= $misses",
                actions = {
                  { op = "modify_attack", crit_multiplier = "$crit_multiplier" },
                  { op = "cue", id = "a07r_old_crit", at = "event.target" },
                  { op = "cue", id = "a07r_old_slam_gesture", at = "self" },
                  { op = "set_state", var = "a07r_old_noncrit", value = "0" },
                },
                otherwise = {
                  { op = "chance", p = "$crit_chance",
                    actions = {
                      { op = "modify_attack", crit_multiplier = "$crit_multiplier" },
                      { op = "cue", id = "a07r_old_crit", at = "event.target" },
                      { op = "cue", id = "a07r_old_slam_gesture", at = "self" },
                      { op = "set_state", var = "a07r_old_noncrit", value = "0" },
                    },
                    otherwise = { { op = "set_state", var = "a07r_old_noncrit", value = "state.a07r_old_noncrit + 1" } } },
                } },
            },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:87-95",
            text = "FC-046 D22 舊版，比照 D13 修：原作保底計數寫成 caster.A07R_noncrit_count = 0 or caster.A07R_noncrit_count，Lua 裡永遠得 0：每次攻擊先歸零，「超過四次沒暴擊必中」永不成立，實際是單純的 31/101 機率。修法：以 unit_state a07r_old_noncrit 真的累計，連續 misses（4）次沒有暴擊，下一次必定暴擊（與原作 count > 4 的序列相同）。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:94-100",
            text = "原作在 OnAttackStart 掛一個持續「攻擊間隔 + 0.1 秒」的 Lua 暴擊 modifier，下次出手時先移除；照意圖寫成只影響這一次攻擊的 modify_attack。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:89",
            text = "目標判斷 not IsUnselectable() or IsUnselectable() 恆為真，對任何目標（含建築）都擲骰。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:115-118",
            text = "caster.maximum_critical_damage 只是戰績統計；ECHO_SLAM 動作與音效是呈現。OnUpgrade A07R_Levelup 歸零計數，沒有作用。" },
        },
        provenance = { "npc/A_Oda/A07.txt:499-552", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:52-121" },
      },
      -------------------------------------------------------------------------
      { id = "a07_old_mightiest_awakening", source_id = "A07T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, 120, 110, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, 200, 250, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          duration = { 19, 24, 29, unit = "s", source = "kv:duration",
            note = "日文說明寫 15/20/25；KV 實際 19/24/29" },
          move_speed = { 60, unit = "move_speed", source = "kv:bonus_movement_speed" },
          base_attack_time = { 0.8, unit = "s", source = "kv:modifier_A07T MODIFIER_PROPERTY_BASE_ATTACK_TIME_CONSTANT" },
          radius = { 400, unit = "dist", source = "script:vscripts/AON_Cleave.lua:181 FindUnitsInRadius 400（KV AOERadius 同）" },
          cleave_pct = { 100, unit = "pct", hidden = true, source = "kv:damage（AON_Cleave.lua:170 × 0.01）" },
          other_pct = { 135, unit = "pct", source = "script:vscripts/AON_Cleave.lua:190 dmg*1.35" },
          target_pct = { 35, unit = "pct", source = "script:vscripts/AON_Cleave.lua:192 dmg*0.35" },
          scale_delay = { 0.3, unit = "s", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:303-307 Timers 0.3 秒後 AddModelScale(1.3, duration − 0.3)",
            note = "放大倍率 1.3 屬呈現，由 cue a07t_old_model_scale 負責" },
        },
        text = {
          zh_TW = { name = "戰國最強．啟動",
            body = "立刻以目前等級免費施放毫髮無傷，並對赤雷擊射程內最近的敵人免費施放赤雷擊，解除身上的暈眩，體型放大。{duration} 秒內魔法免疫，移動速度 +{move_speed}，基礎攻擊間隔變為 {base_attack_time} 秒。每次普攻命中非建築目標時，對忠勝周圍 {radius} 內的其他敵方英雄與部隊造成該次攻擊傷害 {other_pct} 的物理傷害，被攻擊者另受 {target_pct}。" },
          en = { name = "Mightiest of the Warring States: Awaken",
            body = "Instantly casts Unscathed at its current level for free, casts Red Thunder Strike for free on the nearest enemy within its cast range, removes stuns on Tadakatsu and enlarges him. For {duration} seconds Tadakatsu is spell immune, gains +{move_speed} movement speed and his base attack time becomes {base_attack_time} seconds. Each attack that hits a non-building target deals physical damage equal to {other_pct} of that attack's damage to other enemy heroes and units within {radius} of him, and an extra {target_pct} to the attacked target." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a07t_old_mightiest", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", states = { "magic_immune" }, cue = "a07t_old_lightning_aura",
            stats = { move_speed = "$move_speed",
              base_attack_time = { op = "set_absolute", value = "$base_attack_time" } },
            text = { zh_TW = { name = "戰國最強", body = "魔法免疫，移動速度 +{move_speed}，基礎攻擊間隔 {base_attack_time} 秒；普攻命中時對周圍 {radius} 內的敵人造成擴散物理傷害。" },
              en = { name = "Mightiest of the Warring States", body = "Spell immune, +{move_speed} movement speed, base attack time {base_attack_time} seconds; attacks deal physical splash damage to enemies within {radius}." } },
            triggers = {
              on_attack_hit = {
                filter = "not event.target.is_building",
                actions = {
                  { op = "cue", id = "a07t_old_hit", at = "event.target" },
                  -- %attack_damage：這次攻擊對主目標造成的傷害（護甲後），推定為 attack.damage × 主目標護甲倍率
                  { op = "let", name = "base", value = "event.attack.damage * armor_multiplier(event.target.armor) * $cleave_pct" },
                  { op = "let", name = "main", value = "event.target" },
                  { op = "for_each",
                    select = { from = "self", shape = "circle", radius = "$radius", team = "enemy",
                      types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                    actions = {
                      { op = "damage", to = "it", type = "physical",
                        amount = "select(it == main, base * $target_pct, base * $other_pct)" },
                    } },
                } },
            },
            notes = "原作 modifier_A07T（A07T_old 自己的 Modifiers）：IsPurgable 0、IGNORE_INVULNERABLE、AllowIllusionDuplicate；MOVESPEED_BONUS_CONSTANT、BASE_ATTACK_TIME_CONSTANT 0.8、MAGIC_IMMUNE；OnAttackLanded 呼叫 AON_Cleave_A07_old。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a07t_old_cast_sound", at = "caster" },
            { op = "apply_status", status = "a07t_old_mightiest", to = "caster" },
            -- FC-143 D22 舊版：A07T_Transform 照設計意圖找舊版技能（A07W_old／A07E_old），免費施放、冷卻不變
            { op = "if", cond = "ability_level(caster, a07_old_unscathed) > 0", actions = {
              { op = "call", routine = "a07w_old_cast", args = {} },
            } },
            { op = "if", cond = "ability_level(caster, a07_old_red_thunder) > 0", actions = {
              { op = "for_each",
                select = { from = "caster", shape = "circle", radius = "$a07_old_red_thunder.cast_range", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon" }, order = "nearest", limit = "1" },
                actions = {
                  { op = "call", routine = "a07e_old_cast", args = { victim = "it" } },
                } },
            } },
            { op = "remove_status", from = "caster", states = { "stun" }, team_of_source = "enemy" },
            { op = "delay", time = "$scale_delay", actions = {
              { op = "if", cond = "alive(caster) and has_status(caster, a07t_old_mightiest)", actions = {
                { op = "cue", id = "a07t_old_model_scale", attach = "caster", duration = "$duration - $scale_delay" },
              } },
            } },
            { op = "cue", id = "a07t_old_title" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:256-310",
            text = "FC-144 D22 舊版，比照 D13 修：原作 A07T_Transform 找 FindAbilityByName(\"A07W\")，舊版英雄只有 A07W_old，回傳 nil 後呼叫 GetCooldownTime 報錯中斷：重置冷卻、立即施放毫髮無傷與赤雷擊、解除暈眩、模型放大都不會發生（之後的音效與 showTitle 是另外的 RunScript，照常執行）。修法：照本體 A07T 的設計意圖改找舊版技能：已學的毫髮無傷以共用 routine 免費施放；赤雷擊舊版是單體指向技，對射程（cast_range）內最近、可被指定的敵人免費施放，沒有目標就不施放；兩招都照原作 EndCooldown → CastAbilityImmediately → 退還法力 → StartCooldown(原本剩餘冷卻)，等於免費施放、冷卻不變；接著移除敵方施加的暈眩；0.3 秒後若仍在戰國最強（魔免）中，體型放大 1.3 倍到結束（呈現 cue）。舊版沒有 A07D，不換 A07D2。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:300-301",
            text = "keys.modifier_one 沒有傳入（nil），A07T_Transform 裡的 ApplyDataDrivenModifier 沒有效果；modifier_A07T 已由 KV 的 ApplyModifier 套上。" },
          { policy = "as_source", ref = "vscripts/AON_Cleave.lua:165-197",
            text = "擴散以忠勝為圓心、半徑 400，含魔免敵人與被攻擊者：被攻擊者另受 0.35 倍、其他人 1.35 倍的物理傷害，基準是 %attack_damage × damage（100%）。排除 EXCLUDE_TARGET_NAME（selector 預設排除 untargetable）。攻擊建築不擴散。%attack_damage 的語意（護甲後）屬引擎推定，以 attack.damage × armor_multiplier(主目標護甲) 表達。" },
          { policy = "no_effect", ref = "npc/A_Oda/A07.txt:1172-1177",
            text = "CleaveAttack 的 CleavePercent 0、CleaveRadius %radius（不存在的 special），只播 CleaveEffect；AbilitySpecial 的 CleaveRadius 400、CleavePercent 100 沒有被使用。" },
          { policy = "note", ref = "npc/A_Oda/A07.txt:1189-1225",
            text = "OVERRIDE_ANIMATION_RATE 3、TRANSLATE_ACTIVITY alternate、每秒的閃電粒子與音效都是呈現。AllowIllusionDuplicate：幻象也會帶這個狀態（移植由幻象複製規則處理）。" },
          { policy = "as_source", ref = "npc/A_Oda/A07.txt:1131-1140",
            text = "ApplyModifier 沒寫 Duration，用 modifier 自己的 Duration %duration。日文說明的 2.5 倍攻速、+60 攻擊力與「混沌攻擊」在 KV 都找不到，照 KV。" },
        },
        provenance = { "npc/A_Oda/A07.txt:1083-1288", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:256-312",
          "vscripts/AON_Cleave.lua:165-197" },
      },
    },
    routines = {
      a07w_old_cast = {
        owner = "a07_old_unscathed", params = {},
        actions = {
          { op = "cue", id = "a07w_old_cast_sound", at = "caster" },
          { op = "apply_status", status = "a07w_old_guard", to = "caster" },
          { op = "apply_status", status = "a07w_old_convert", to = "caster" },
        },
      },
      a07e_old_cast = {
        owner = "a07_old_red_thunder", params = { victim = "unit" },
        actions = {
          { op = "cue", id = "a07e_old_cast_sound", at = "caster" },
          { op = "apply_status", status = "a07e_old_stun", to = "victim" },
          { op = "damage", to = "victim", type = "magical", amount = "$damage", require_source_alive = false },
          { op = "cue", id = "a07e_old_slam", at = "victim" },
        },
      },
    },
  }
end
