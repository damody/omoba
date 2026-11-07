-- A34 阿松（2.0 版，原作 *_20 技能組）— package schema v3。每個版本是一位獨立英雄（D22）；舊版另成 omatsu_old。
-- 槽位依據：選 2.0 版時 choose_20 拔掉全部技能，依 heromap_skill A34 ["20"] = "WERDT" 逐一 AddAbility("A34" .. 字母 .. "_20")，
--   第五個字母前插入 attribute_bonusx；heromap_autoskill A34 ["20"] = "D" 自動學 A34D_20
--   （vscripts/items/Addon_Items/tofar_goback.lua:6-48、vscripts/addon_game_mode.lua:276,379）。
-- W/E/R/T → Q/W/E/R；A34D_20（PASSIVE | NOT_LEARNABLE、Maxlevel 1）照慣例放 D，寫在 hero.auto_learn。
-- KV 在 npc/A_Oda/A34.txt:1144-1645；腳本 vscripts/heroes/A_Oda/A34_Matsu.lua:162-193。原作沒有 *_20 招式的說明文字，
--   招式名稱取自 KV 檔的註解，說明依 KV 與腳本還原（assumptions.md）。英雄單位數值沿用本體 A34。
-- magical_resistance、attribute_bonusx、A34_precache 屬規則層或資產，不列入。
-- 慣例照本體與舊版：KV 未寫 IsPurgable 的 datadriven modifier 暫定 dispel = "basic"；datadriven Damage（不經 AMHC）
--   在阿松死亡後仍會造成傷害，寫 require_source_alive = false。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "A34_v20",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "v20" },
    hero = { source_id = "A34", text = { zh_TW = { name = "阿松（2.0）" }, en = { name = "Omatsu (2.0)" } },
      auto_learn = { "a34_v20_kaga_million_koku" } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a34_v20_frost_bolt", source_id = "A34W_20", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A34.txt:1162" },
          mana_cost = { 100, 125, 150, 175, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A34.txt:1165" },
          cast_range = { 825, unit = "dist", source = "kv:AbilityCastRange@npc/A_Oda/A34.txt:1168" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/A_Oda/A34.txt:1159" },
          projectile_speed = { 2000, unit = "speed", hidden = true, source = "kv:TrackingProjectile MoveSpeed@npc/A_Oda/A34.txt:1190" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:A34W_Damage@npc/A_Oda/A34.txt:1240" },
          freeze = { 1, 1.5, 2, 2.5, unit = "s", source = "kv:duration@npc/A_Oda/A34.txt:1235" },
          non_hero_bonus = { 1.5, unit = "s",
            source = "script:vscripts/heroes/A_Oda/A34_Matsu.lua:189-190 非英雄 duration + 1.5" },
        },
        text = {
          zh_TW = { name = "寒霜彈",
            body = "對敵人或敵方建築投出可閃避的追蹤寒霜彈，命中時結凍目標 {freeze} 秒（非英雄單位再多 {non_hero_bonus} 秒）並造成 {damage} 魔法傷害。" },
          en = { name = "Frost Bolt",
            body = "Throws a dodgeable homing frost bolt at an enemy unit or building; on hit it freezes the target for {freeze} seconds ({non_hero_bonus} seconds longer on non-heroes) and deals {damage} magic damage." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon", "building" } },
        flags = { "can_target_buildings" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a34w_20_freeze", text = { zh_TW = { name = "結凍", body = "無法移動、攻擊或施法。" }, en = { name = "Frozen", body = "Cannot move, attack or cast." } }, duration = "$freeze", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", states = { "stun" }, cue = "a34w_20_frozen",
            notes = "原作 modifier_frost_bite_root_datadriven：IsPurgable 1、IsDebuff、IsStunDebuff、STUNNED；持續時間由 A34W_20_OnProjectileHitUnit 給。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a34w_20_vo", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = true, cue = "a34w_20_bolt",
              on_hit = {
                { op = "apply_status", status = "a34w_20_freeze", to = "it",
                  duration = "select(it.is_hero, $freeze, $freeze + $non_hero_bonus)" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A34_Matsu.lua:182-193",
            text = "OnProjectileHitUnit 先跑腳本（非英雄結凍 duration + 1.5，英雄 duration），再做 datadriven Damage；建築也算非英雄。" },
          { policy = "as_source", ref = "npc/A_Oda/A34.txt:1151-1155",
            text = "可以打建築（TargetType 含 BUILDING）；SPELL_IMMUNITY_ENEMIES_NO 讓魔免敵人不能被選為目標。追蹤彈 Dodgeable，目標閃避時失效。" },
          { policy = "note", text = "2.0 版沒有本體急速冷卻的魔抗疊層（沒有 A34T_OnHit）。" },
        },
        provenance = { "npc/A_Oda/A34.txt:1144-1243", "vscripts/heroes/A_Oda/A34_Matsu.lua:182-193" },
      },
      -------------------------------------------------------------------------
      { id = "a34_v20_iceberg_strike", source_id = "A34E_20", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 14, 12, 10, 8, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A34.txt:1262" },
          mana_cost = { 125, 145, 165, 185, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A34.txt:1265" },
          cast_range = { 900, unit = "dist", source = "kv:AbilityCastRange@npc/A_Oda/A34.txt:1268" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/A_Oda/A34.txt:1259" },
          radius = { 335, unit = "dist", source = "kv:radius@npc/A_Oda/A34.txt:1352" },
          target_damage = { 100, unit = "dmg", damage_type = "magical", source = "kv:aoe_damage@npc/A_Oda/A34.txt:1372", note = "OnSpellStart 對主目標的 Damage" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/A_Oda/A34.txt:1378" },
          slow_duration = { 5, unit = "s", source = "kv:AbilityDuration@npc/A_Oda/A34.txt:1375", note = "ApplyModifier Duration；modifier 自己的 %duration 也是 5" },
          slow = { 30, unit = "pct", source = "kv:slow_movement_speed@npc/A_Oda/A34.txt:1362", kv = { -30 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          attack_slow = { 20, unit = "atk_speed", source = "kv:slow_attack_speed@npc/A_Oda/A34.txt:1367", kv = { -20 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
        },
        text = {
          zh_TW = { name = "冰山之襲",
            body = "以冰山砸向一名敵人，對其造成 {target_damage} 魔法傷害；並對它周圍 {radius} 內的敵方英雄與單位（含主目標）造成 {damage} 魔法傷害，移動速度降低 {slow}、攻擊速度降低 {attack_slow}，持續 {slow_duration} 秒。" },
          en = { name = "Iceberg Strike",
            body = "Drops an iceberg on an enemy, dealing {target_damage} magic damage to it; enemy heroes and units within {radius} of it (the target included) take {damage} magic damage and lose {slow} movement speed and {attack_slow} attack speed for {slow_duration} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC, aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a34e_20_slow", text = { zh_TW = { name = "冰山之襲", body = "移動速度降低 {slow}，攻擊速度降低 {attack_slow}。" }, en = { name = "Iceberg Strike", body = "Movement speed reduced by {slow}; attack speed reduced by {attack_slow}." } }, duration = "$slow_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "a34e_20_frost",
            stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            notes = "原作 modifier_frost_nova_slow：IsPurgable 1、IsDebuff、MOVESPEED_BONUS_PERCENTAGE、ATTACKSPEED_BONUS_CONSTANT（Properties 版本，不經 dummy.lua 緩速池）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a34e_20_vo", at = "caster" },
            { op = "cue", id = "a34e_20_iceberg", at = "target", radius = "$radius" },
            { op = "damage", to = "target", type = "magical", amount = "$target_damage", require_source_alive = false },
            { op = "for_each",
              select = { from = "target", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
              actions = {
                { op = "apply_status", status = "a34e_20_slow", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A34.txt:1292-1323",
            text = "主目標先吃 aoe_damage 100，再被範圍 ActOnTargets 選到，一共受 100 + AbilityDamage。範圍選取 FLAG_NONE，排除魔免；不能對建築施放。" },
          { policy = "note", ref = "npc/A_Oda/A34.txt:1382",
            text = "AbilityModifierSupportValue 0.3 只影響引擎的觸發係數，不影響結算。" },
        },
        provenance = { "npc/A_Oda/A34.txt:1245-1383" },
      },
      -------------------------------------------------------------------------
      { id = "a34_v20_frost_breath", source_id = "A34R_20", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 12, 10, 8, 6, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A34.txt:1405" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A34.txt:1408" },
          cast_range = { 900, unit = "dist", source = "kv:AbilityCastRange@npc/A_Oda/A34.txt:1411" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/A_Oda/A34.txt:1400" },
          projectile_speed = { 1250, unit = "speed", hidden = true, source = "kv:speed@npc/A_Oda/A34.txt:1512" },
          distance = { 1200, unit = "dist", source = "kv:range@npc/A_Oda/A34.txt:1507", note = "LinearProjectile FixedDistance" },
          start_radius = { 150, unit = "dist", hidden = true, source = "kv:start_radius@npc/A_Oda/A34.txt:1497" },
          end_radius = { 500, unit = "dist", hidden = true, source = "kv:end_radius@npc/A_Oda/A34.txt:1502" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/A_Oda/A34.txt:1532" },
          dot_damage = { 30, 40, 50, 60, unit = "dmg", damage_type = "magical", source = "kv:debuff_damage@npc/A_Oda/A34.txt:1527" },
          dot_duration = { 6, unit = "s", source = "kv:damage_duration@npc/A_Oda/A34.txt:1517" },
          dot_interval = { 1, unit = "s", source = "kv:damage_interval@npc/A_Oda/A34.txt:1522", note = "modifier ThinkInterval" },
        },
        text = {
          zh_TW = { name = "冰霜之息",
            body = "朝前方吐出漸寬的冰霜，飛行 {distance}，穿透沿途的敵方英雄與單位：造成 {damage} 魔法傷害，之後 {dot_duration} 秒內每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" },
          en = { name = "Frost Breath",
            body = "Breathes a widening wave of frost forward that travels {distance}, piercing enemy heroes and units: each takes {damage} magic damage, then {dot_damage} magic damage every {dot_interval} seconds for {dot_duration} seconds." },
        },
        target = { type = "direction", vector_length = "$distance", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a34r_20_frostbite", text = { zh_TW = { name = "冰霜之息", body = "每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" }, en = { name = "Frost Breath", body = "Takes {dot_damage} magic damage every {dot_interval} seconds." } }, duration = "$dot_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "a34r_20_frost",
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$dot_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_A34R_datadriven：IsPurgable 1、IsDebuff；modifier 的 Duration 讀不存在的 %AbilityDuration，實際由 ApplyModifier 的 %damage_duration 6 決定；ThinkInterval 1 每次 datadriven Damage。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a34r_20_vo", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$projectile_speed", distance = "$distance", radius = "$start_radius", end_radius = "$end_radius",
              pierce = true, team = "enemy", types = BASIC, cue = "a34r_20_breath",
              on_hit = {
                { op = "apply_status", status = "a34r_20_frostbite", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A34.txt:1420-1436",
            text = "LinearProjectile 速度 1250、半徑 150 漸寬到 500、固定飛行 1200，穿透（DeleteOnHit 0）；TargetFlags NONE，技能欄的 MAGIC_IMMUNE_ENEMIES 對投射物無效。TargetTypes 裡的 DOTA_UNITY_TARGET_MECHANICAL 拼錯，無效。" },
          { policy = "design_intent", ref = "npc/A_Oda/A34.txt:1458-1473",
            text = "ThinkInterval 1、持續 6 秒：第六跳與到期同時，原作可能只有五跳；照設計意圖每秒一跳寫。" },
        },
        provenance = { "npc/A_Oda/A34.txt:1385-1533" },
      },
      -------------------------------------------------------------------------
      { id = "a34_v20_snow_blessing", source_id = "A34T_20", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 120, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A34.txt:1546" },
          mana_cost = { 150, 200, 250, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A34.txt:1547" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/A_Oda/A34.txt:1545" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel；Dota 大招預設 6 級起、每 6 級升一次" },
          duration = { 30, unit = "s", source = "kv:AbilityDuration@npc/A_Oda/A34.txt:1548" },
          hp_regen = { 35, 40, 45, unit = "per_s", source = "kv:add_hp_regen@npc/A_Oda/A34.txt:1593" },
          damage_bonus = { 10, 20, 30, unit = "pct", source = "kv:per_phy_atk@npc/A_Oda/A34.txt:1598", note = "DAMAGEOUTGOING_PERCENTAGE" },
          armor = { 5, 10, 15, unit = "armor", source = "kv:add_phy_def@npc/A_Oda/A34.txt:1603" },
          invuln_time = { 0.5, unit = "s", source = "kv:modifier_invulnerable Duration@npc/A_Oda/A34.txt:1567" },
        },
        text = {
          zh_TW = { name = "雪女的祝福",
            body = "賦予自己或一名友方英雄冰之力，持續 {duration} 秒：生命回復增加 {hp_regen}、造成的傷害增加 {damage_bonus}、護甲增加 {armor}。施放時目標無敵 {invuln_time} 秒。" },
          en = { name = "Blessing of the Yuki Onna",
            body = "Grants herself or an allied hero the power of ice for {duration} seconds: +{hp_regen} health regeneration, +{damage_bonus} damage dealt and +{armor} armor. The target is invulnerable for {invuln_time} seconds on cast." },
        },
        target = { type = "unit", team = "ally", types = { "hero" } },
        flags = { "self_cast_allowed" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a34t_20_buff", text = { zh_TW = { name = "雪女的祝福", body = "提升生命回復、造成的傷害與護甲。" }, en = { name = "Blessing of the Yuki Onna", body = "Gains health regeneration, damage dealt and armor." } }, duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "a34t_20_glow",
            stats = { hp_regen = "$hp_regen", outgoing_damage_pct = "$damage_bonus", armor = "$armor" },
            notes = "原作 A34T_20 宣告的 modifier 名稱沿用 modifier_A34T_old_buff（與舊版同名，屬不同技能）：datadriven，沒寫 IsPurgable（暫定 basic）、MODIFIER_ATTRIBUTE_IGNORE_INVULNERABLE；2.0 版只有回血、增傷、護甲三項。" },
          { id = "a34t_20_invulnerable", duration = "$invuln_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "invulnerable" },
            notes = "引擎內建 modifier_invulnerable，以 ApplyModifier Duration 0.5 套上。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a34t_20_vo", at = "caster" },
            { op = "apply_status", status = "a34t_20_buff", to = "target" },
            { op = "apply_status", status = "a34t_20_invulnerable", to = "target" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A34.txt:1536-1547",
            text = "KV 沒有 AbilityCastRange 與 MaxLevel：推定任何距離都能對友方英雄施放（TEAM_FRIENDLY 含自己），大招預設三級。" },
          { policy = "note", ref = "npc/A_Oda/A34.txt:1556,1569",
            text = "音效與 modifier 名稱沿用舊版的 A34T_old.vo1、modifier_A34T_old_buff；數值與舊版不同（冷卻固定 120、無魔抗與回魔），在 v3 用獨立狀態 a34t_20_buff。" },
          { policy = "note", text = "施放時對目標套 0.5 秒 modifier_invulnerable（會打斷正在對它施放的單體技能），屬原作效果。" },
        },
        provenance = { "npc/A_Oda/A34.txt:1535-1606" },
      },
      -------------------------------------------------------------------------
      { id = "a34_v20_kaga_million_koku", source_id = "A34D_20", slot = "D", kind = "passive", max_level = 1, tags = { "basic" },
        learnable = false,
        values = {
          cooldown_refund = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A34_Matsu.lua:174-175 StartCooldown(t - 1)" },
        },
        text = {
          zh_TW = { name = "加賀百萬石",
            body = "被動：每施放一個技能或使用一個道具（開關型除外），寒霜彈、冰山之襲、冰霜之息、雪女的祝福正在冷卻的剩餘時間各減少 {cooldown_refund} 秒。開場自動學會，無法升級。" },
          en = { name = "Kaga Million Koku",
            body = "Passive: each time she casts an ability or uses an item (toggles excluded), the remaining cooldowns of Frost Bolt, Iceberg Strike, Frost Breath and Blessing of the Yuki Onna are each reduced by {cooldown_refund} seconds. Learned automatically at the start; cannot be leveled." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public", visibility = "buff",
          triggers = {
            on_ability_executed = { who = "self", filter = "not event.ability.is_toggle",
              actions = {
                { op = "modify_cooldown", of = "self", ability = "a34_v20_frost_bolt", mode = "reduce", amount = "$cooldown_refund" },
                { op = "modify_cooldown", of = "self", ability = "a34_v20_iceberg_strike", mode = "reduce", amount = "$cooldown_refund" },
                { op = "modify_cooldown", of = "self", ability = "a34_v20_frost_breath", mode = "reduce", amount = "$cooldown_refund" },
                { op = "modify_cooldown", of = "self", ability = "a34_v20_snow_blessing", mode = "reduce", amount = "$cooldown_refund" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A34_Matsu.lua:162-180",
            text = "OnAbilityExecuted 對技能與道具都觸發，只排除開關型與 attribute_bonusx；對 A34W/E/R/T_20 中正在冷卻的每一招 EndCooldown 後 StartCooldown(剩餘 − 1)，剩不到 1 秒等於結束冷卻。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A34_Matsu.lua:168-176",
            text = "迴圈也包含剛施放的那一招：OnAbilityExecuted 時該招冷卻若已開始，自己也會被減 1 秒（推定已開始，confidence medium）；大招也在減冷卻清單內，和本體只減三招不同。" },
          { policy = "note", ref = "npc/A_Oda/A34.txt:1609-1645",
            text = "KV 的 HotKeyOverride R、CastPoint、TargetType 等欄位對被動無作用；modifier_A34D_20 IsHidden 0、IsBuff。" },
        },
        provenance = { "npc/A_Oda/A34.txt:1608-1645", "vscripts/heroes/A_Oda/A34_Matsu.lua:162-180",
          "vscripts/items/Addon_Items/tofar_goback.lua:6-48", "vscripts/addon_game_mode.lua:276,379" },
      },
    },
  }
end
