-- C05 前田慶次（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C05"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:321）；自動學習 heromap_autoskill["11"] = ""
-- （addon_game_mode.lua:424），沒有 D。W/E/R/T → Q/W/E/R。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 C05*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- C05E_old 的 OnSpellStart 呼叫本體的 C05E_OnSpellStart（發射追蹤雷電），命中改呼叫 C05E_old_OnProjectileHitUnit。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C05_precache 屬規則層或資產，不列入。
-- 慣例照本體 C05：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- datadriven Properties 直接給的移速百分比寫 move_speed_pct；HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C05_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C05", text = { zh_TW = { name = "前田慶次（舊版）" }, en = { name = "Maeda Keiji (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c05_old_horsemanship", source_id = "C05W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 29, 26, 23, 20, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C05.txt:409" },
          mana_cost = { 160, 180, 200, 220, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C05.txt:410" },
          duration = { 14, 16, 18, 20, unit = "s", source = "kv:buff_duration@npc/C_Neutral/C05.txt:417" },
          move_speed = { 25, 30, 35, 40, unit = "pct", source = "kv:bonus_movement_speed@npc/C_Neutral/C05.txt:422" },
          attack_speed = { 25, 30, 35, 40, unit = "atk_speed", source = "kv:bonus_attack_speed@npc/C_Neutral/C05.txt:427" },
        },
        text = {
          zh_TW = { name = "騎術",
            body = "立即施放（不打斷目前動作）：{duration} 秒內移動速度提高 {move_speed}、攻擊速度 +{attack_speed}。" },
          en = { name = "Horsemanship",
            body = "Cast instantly without interrupting the current action: for {duration} seconds, gains {move_speed} movement speed and +{attack_speed} attack speed." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c05w_old_buff", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "騎術", body = "移動速度提高 {move_speed}、攻擊速度 +{attack_speed}。" },
              en = { name = "Horsemanship", body = "{move_speed} more movement speed and +{attack_speed} attack speed." },
            },
            reveal = "public", cue = "c05w_old_armlet",
            stats = { move_speed_pct = "$move_speed", attack_speed = "$attack_speed" },
            notes = "原作 modifier_C05W_old_buff：ATTACKSPEED_BONUS_CONSTANT %bonus_attack_speed、MOVESPEED_BONUS_PERCENTAGE %bonus_movement_speed；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c05w_old_vo", at = "caster" } },
          on_cast = {
            { op = "apply_status", status = "c05w_old_buff", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C05.txt:413-429",
            text = "AbilitySpecial 的 bonus_movement_speed 與 bonus_attack_speed 都用鍵名 \"02\"；KV 保留重複鍵，GetSpecialValueFor 與 %變數 依名稱查找，兩個數值都照寫入值生效（推定，見 assumptions.md）。" },
          { policy = "as_source", ref = "npc/C_Neutral/C05.txt:445-454",
            text = "和本體完全不同：沒有腳本，只由 KV ApplyModifier 對自己套增益（持續 %buff_duration）；KV 沒寫 AbilityCastPoint（IMMEDIATE），前搖為零。" },
          { policy = "note", ref = "npc/C_Neutral/C05.txt:466-474", text = "OnCreated 的 AttachEffect 與 EffectName 只是呈現，改用狀態 cue。" },
        },
        provenance = { "npc/C_Neutral/C05.txt:400-482" },
      },
      -------------------------------------------------------------------------
      { id = "c05_old_thunder_dance", source_id = "C05E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 22, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C05.txt:497" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C05.txt:498" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C05.txt:502" },
          radius = { 400, unit = "dist",
            source = "script:vscripts/heroes/C_Neutral/C05.lua:26 ability:GetCastRange()（KV AbilityCastRange 400@C05.txt:501）" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C05.txt:499" },
          projectile_speed = { 2000, unit = "speed", source = "kv:projectile_speed@npc/C_Neutral/C05.txt:517" },
          attack_slow = { 20, 30, 40, 50, unit = "atk_speed", kv = { -20, -30, -40, -50 },
            source = "kv:debuff_attack_speed@npc/C_Neutral/C05.txt:522",
            source_differs = "KV 以負值寫在 ATTACKSPEED_BONUS_CONSTANT；這裡以正值表示降低量" },
          slow = { 20, 30, 40, 50, unit = "pct", kv = { -20, -30, -40, -50 },
            source = "kv:debuff_movement_speed@npc/C_Neutral/C05.txt:527",
            source_differs = "KV 以負值寫在 MOVESPEED_BONUS_PERCENTAGE；這裡以正值表示降低量" },
          slow_duration = { 4, unit = "s", source = "kv:debuff_duration@npc/C_Neutral/C05.txt:532" },
        },
        text = {
          zh_TW = { name = "雷舞",
            body = "對周圍 {radius} 內看得見、非魔免的每個敵方英雄與部隊各射出一道追蹤雷電（速度 {projectile_speed}，可被閃躲），命中造成 {damage} 魔法傷害，並使其攻擊速度降低 {attack_slow}、移動速度降低 {slow}，持續 {slow_duration} 秒。" },
          en = { name = "Thunder Dance",
            body = "Fires a homing bolt (speed {projectile_speed}, dodgeable) at every visible, non-spell-immune enemy hero and unit within {radius}; each bolt deals {damage} magic damage on impact and reduces attack speed by {attack_slow} and movement speed by {slow} for {slow_duration} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c05e_old_debuff", duration = "$slow_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "雷舞", body = "攻擊速度降低 {attack_slow}、移動速度降低 {slow}。" },
              en = { name = "Thunder Dance", body = "Attack speed reduced by {attack_slow} and movement speed by {slow}." },
            },
            reveal = "public", cue = "c05e_old_atos", tags = { "slow" },
            stats = { attack_speed = "-$attack_slow", move_speed_pct = "-$slow" },
            notes = "原作 modifier_C05E_old_debuff：Properties 直接 ATTACKSPEED_BONUS_CONSTANT 與 MOVESPEED_BONUS_PERCENTAGE（不經 nobu 緩速池）；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c05e_old_vo", at = "caster" } },
          on_cast = {
            { op = "cue", id = "c05e_old_cast_sound", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                flags = { "visible_only" } },
              actions = {
                { op = "let", name = "victim", value = "it" },
                { op = "projectile", kind = "tracking", from = "caster", to = "victim", speed = "$projectile_speed",
                  dodgeable = true, cue = "c05e_old_bolt",
                  on_hit = {
                    { op = "apply_status", status = "c05e_old_debuff", to = "it" },
                    { op = "cue", id = "c05e_old_impact", at = "it" },
                    { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C05.txt:557,567",
            text = "發射沿用本體 C05E_OnSpellStart；命中改呼叫 C05E_old_OnProjectileHitUnit：先套 modifier_C05E_old_debuff 再 ApplyDamage（不經 AMHC，慶次死亡後雷電照樣造成傷害）。和本體不同：多了攻速與移速降低，冷卻 22。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C05.lua:23-31",
            text = "搜尋用技能設定（ENEMY、HERO|BASIC、FOW_VISIBLE）：只選看得見的敵人，魔免敵人選不到。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C05.lua:9-20",
            text = "projectile_table 是全域變數、Target 初值引用未定義的 target，迴圈內逐一覆寫，沒有影響。" },
        },
        provenance = { "npc/C_Neutral/C05.txt:484-587", "vscripts/heroes/C_Neutral/C05.lua:3-40", "vscripts/heroes/C_Neutral/C05.lua:198-214" },
      },
      -------------------------------------------------------------------------
      { id = "c05_old_thunder_strike", source_id = "C05R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C05.txt:601" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C05.txt:602" },
          cast_range = { 200, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C05.txt:605" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C05.txt:606" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C05.txt:603" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:stun_time@npc/C_Neutral/C05.txt:614" },
        },
        text = {
          zh_TW = { name = "雷擊",
            body = "以落雷打擊一名敵方英雄或部隊，暈眩 {stun} 秒並造成 {damage} 魔法傷害。" },
          en = { name = "Thunder Strike",
            body = "Strikes an enemy hero or unit with lightning, stunning it for {stun} seconds and dealing {damage} magic damage." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c05r_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "雷擊暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Thunder Strike Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（ApplyDataDrivenModifier 套用）；受狀態抗性縮短。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c05r_old_vo", at = "caster" } },
          on_cast = {
            { op = "apply_status", status = "c05r_old_stun", to = "target" },
            { op = "cue", id = "c05r_old_lightning_bolt", at = "caster", to = "target" },
            { op = "cue", id = "c05r_old_screen_flash", at = "target" },
            { op = "damage", to = "target", type = "magical", amount = "$damage", require_source_alive = false },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C05.lua:225-248",
            text = "先套 modifier_stunned 再 ApplyDamage（不經 AMHC）。和本體不同：本體 C05R 是被動，舊版是點目標的雷擊。KV 目標旗標 NONE：不能對魔免敵人施放。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C05.lua:216-238",
            text = "CreateScreenEffect 只對真實英雄的玩家畫面閃光，落雷粒子與音效都只是呈現，改用 cue。" },
        },
        provenance = { "npc/C_Neutral/C05.txt:589-641", "vscripts/heroes/C_Neutral/C05.lua:216-248" },
      },
      -------------------------------------------------------------------------
      { id = "c05_old_thunder_spear", source_id = "C05T_old", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          required_level = { 6, 12, 18, unit = "level",
            source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 只寫 AbilityType ULTIMATE@C05.txt:648）" },
          damage = { 70, 110, 180, unit = "dmg", damage_type = "pure", source = "kv:damage_per_think@npc/C_Neutral/C05.txt:661" },
          immune_ratio = { 50, unit = "pct",
            source = "script:vscripts/heroes/C_Neutral/C05.lua:269,300 魔免目標 damage_per_think*0.5" },
          duration = { 7, unit = "s", source = "kv:debuff_duration@npc/C_Neutral/C05.txt:666" },
          attack_slow = { 40, 60, 80, unit = "atk_speed", kv = { -40, -60, -80 },
            source = "kv:debuff_attack_speed@npc/C_Neutral/C05.txt:671",
            source_differs = "KV 以負值寫在 ATTACKSPEED_BONUS_CONSTANT；這裡以正值表示降低量" },
          tick = { 1, unit = "s", source = "kv:modifier_C05T_old_debuff ThinkInterval@npc/C_Neutral/C05.txt:718" },
        },
        text = {
          zh_TW = { name = "雷鳴豪槍",
            body = "被動：普攻命中英雄或部隊時造成 {damage} 純粹傷害，並讓目標帶電 {duration} 秒：攻擊速度降低 {attack_slow}，每 {tick} 秒再受到 {damage} 純粹傷害。魔免目標受到的傷害只有 {immune_ratio}。再次命中會刷新持續時間。" },
          en = { name = "Thunderclap Spear",
            body = "Passive: attacks that land on heroes or units deal {damage} pure damage and electrify the target for {duration} seconds, reducing its attack speed by {attack_slow} and dealing {damage} pure damage every {tick} seconds. Spell-immune targets take only {immune_ratio} of this damage. Hitting again refreshes the duration." },
        },
        target = { type = "none" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "c05t_old_debuff", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "雷鳴豪槍", body = "攻擊速度降低 {attack_slow}，每 {tick} 秒受到 {damage} 純粹傷害（魔免時只有 {immune_ratio}）。" },
              en = { name = "Thunderclap Spear", body = "Attack speed reduced by {attack_slow}; takes {damage} pure damage every {tick} seconds ({immune_ratio} while spell immune)." },
            },
            reveal = "public", cue = "c05t_old_disarm_arc",
            stats = { attack_speed = "-$attack_slow" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "cue", id = "c05t_old_arc_impact", at = "self" },
                { op = "damage", to = "self", type = "pure", require_source_alive = false,
                  amount = "select(self.is_magic_immune, $damage * $immune_ratio, $damage)" },
              } },
            },
            notes = "原作 modifier_C05T_old_debuff：ATTACKSPEED_BONUS_CONSTANT %debuff_attack_speed、ThinkInterval 1；施加者是目標自己，傷害來源另存在 .caster（慶次）。沒寫 IsPurgable，暫定 basic。" },
        },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = { filter = "event.target.is_hero or event.target.is_creep", actions = {
              { op = "apply_status", status = "c05t_old_debuff", to = "event.target" },
              { op = "cue", id = "c05t_old_bodyarc", at = "event.target" },
              { op = "damage", to = "event.target", type = "pure", require_source_alive = false,
                amount = "select(event.target.is_magic_immune, $damage * $immune_ratio, $damage)" },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/C_Neutral/C05.lua:286-293",
            text = "FC-166 D22 舊版，比照 D13 修：C05T_old_OnIntervalThink 在宣告 local target（292 行）之前就用 target 建粒子與 target:GetAbsOrigin()（287-291 行），讀到的是未定義的全域 target（nil），執行期報錯，每秒的純粹傷害從未發生，只剩命中當下的傷害與攻速降低。修法：照意圖每秒對帶電目標造成 damage_per_think 純粹傷害（魔免減半），來源是慶次。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C05.lua:250-284",
            text = "只對 IsHero() 或 IsCreep() 的目標生效（建築與守衛等不算）；OnAttackLanded 不分敵我，反補友軍也會套。命中先套狀態再 ApplyDamage（不經 AMHC）。KV 是 PURE，腳本對魔免目標手動減半。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C05.lua:256-257",
            text = "modifier_C05T_old_debuff 以目標自己為施加者（ApplyDataDrivenModifier(target, target, ...)），傷害來源另存慶次；資料以慶次為施加者，差別只在依施加者判斷的規則（例如鐵布衫類「移除敵方施加的狀態」移不掉它）。" },
          { policy = "as_source", ref = "npc/C_Neutral/C05.txt:644-660",
            text = "和本體完全不同：本體是引導落雷的大招，舊版是被動法球；KV 沒寫 RequiredLevel，照大招預設。" },
        },
        provenance = { "npc/C_Neutral/C05.txt:643-733", "vscripts/heroes/C_Neutral/C05.lua:250-314" },
      },
    },
  }
end
