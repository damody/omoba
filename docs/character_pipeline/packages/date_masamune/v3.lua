-- H02 伊達政宗（原作 B04 current）— package schema v3 正式來源（唯一一份；kit 與 census-v3 B04 合併）。
-- 數值與說明照現行移植（M1）：values 的名稱、數字、v2_key 與 text 照已上架的 kit，Rust handler 依名稱讀 extras。
-- 行為照原作（census-v3 B04 移入）：events、passive、statuses、文件層 routines，值名稱改成 kit 的名稱。
-- `coverage` 在 M5 切換成資料執行前維持 `rust_handler`；行為區塊目前只供 IR 編譯與 M5 使用。
-- 原作槽位 W/E/R/T → Q/W/E/R（current 版 heromap_version["16"] = "WERT"，沒有 D）。逐招對照見 parity.md，假設見 assumptions.md／source.lua。
-- 每個數值的 v2_key 是現行 v2 extras 鍵；冷卻／耗魔／前搖／射程／需要等級取代 v2 levels。
-- census 註記：英雄共通魔抗池（magical_resistance）屬規則層；B04_precache 的 modifier_aggressive 只是動作替換，都不列入。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "B04",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "current" },
    hero = {
      source_id = "B04",
      text = { zh_TW = { name = "伊達政宗" }, en = { name = "Date Masamune" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b04_flame_trail_dash", source_id = "B04W", slot = "Q", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 21, 19, 17, 15, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B04.txt:61" },
          mana_cost = { 80, 100, 120, 140, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B04.txt:62" },
          cast_range = { 900, 1000, 1100, 1200, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B04.txt:65" },
          cast_point = { 0.01, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B04.txt:66" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical",
            source = "kv:AbilityDamage@npc/B_Unified/B04.txt:58", v2_key = "damage_amount" },
          dash_range = { expr = "$cast_range", unit = "dist", hidden = true, source = "derived",
            v2_key = "dash_max_distance", note = "v2 effects.dash.max_distance 的複本；handler 以施法距離為準" },
          dash_speed = { 3000, unit = "speed", source = "kv:speed@npc/B_Unified/B04.txt:74", v2_key = "dash_speed" },
          trigger_radius = { 300, unit = "dist", source = "kv:aura radius@npc/B_Unified/B04.txt:79", v2_key = "dash_trigger_radius" },
          fire_speed = { expr = "$dash_speed * $fire_speed_ratio", unit = "speed", source = "derived",
            v2_key = "projectile_speed", note = "原作 B04.lua:65 iMoveSpeed = speed*0.5：火焰速度跟著衝刺速度（D13，2026-10-29 改成推導值）" },
          fire_speed_ratio = { 50, unit = "pct", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B04.lua:65 iMoveSpeed = speed*0.5" },
          fire_radius = { 16, unit = "dist", hidden = true, source = "design:追蹤火焰的碰撞半徑（原作追蹤投射物沒有半徑）",
            v2_key = "projectile_radius" },
          fire_lifetime = { 10, unit = "s", hidden = true,
            source = "design:host G-1 投射物必須有壽命（上限 30 秒）；原作追蹤投射物沒有壽命", v2_key = "projectile_lifetime" },
          max_targets = { 32, unit = "count",
            source = "design:host 每來源存活投射物上限 ability_projectile::MAX_LIVE_PER_SOURCE；原作沒有目標上限",
            v2_key = "projectile_max_targets" },
          indicator_width = { expr = "$trigger_radius + $trigger_radius", unit = "dist", hidden = true, source = "derived",
            v2_key = "shape_width", note = "host targeting_shape.width_key（路徑兩側觸發範圍）" },
        },
        text = {
          zh_TW = { name = "踏火無痕",
            body = "以 {dash_speed} 的速度衝刺到指定地點，受地形阻擋時停在可到達處。衝刺路徑兩側 {trigger_radius} 內的敵方英雄與一般單位（依接觸先後，最多 {max_targets} 個）各被一道速度 {fire_speed} 的追蹤火焰鎖定，火焰命中時造成 {damage} 魔法傷害；目標死亡或失效則火焰消失。施放時躲開追蹤自己的投射物。" },
          en = { name = "Flame Trail Dash",
            body = "Dashes to the target point at speed {dash_speed}, stopping early if terrain blocks the way. Each enemy hero or unit within {trigger_radius} of the path (in order of contact, up to {max_targets}) is targeted by a homing flame of speed {fire_speed} that deals {damage} magic damage on hit; the flame fizzles if its target dies or becomes invalid. Dodges projectiles tracking Masamune on cast." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b04w_aura", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public",
            aura = { radius = "$trigger_radius", team = "enemy", types = { "hero", "creep", "neutral", "summon" }, status = "b04w_trigger" },
            notes = "原作 modifier_B04W_aura：IsHidden，Aura modifier_B04W_trigger（ENEMY、HERO | BASIC、不含自己），持續時間 = 衝刺時間；OnDeath 移除。沒寫 IsPurgable：依 §12.3 暫定 basic（推定）。" },
          { id = "b04w_trigger", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_created = {
                -- fire_speed = $dash_speed * $fire_speed_ratio（原作 speed × 0.5）
                { op = "projectile", kind = "tracking", from = "caster", to = "self", speed = "$fire_speed",
                  dodgeable = true, cue = "b04w_fire",
                  on_hit = {
                    { op = "damage", to = "it", source = "caster", type = "magical", amount = "$damage", require_source_alive = false },
                    { op = "cue", id = "b04w_fire_hit", at = "it" },
                  } },
              },
            },
            notes = "原作 modifier_B04W_trigger：光環子 modifier，OnCreated 呼叫 B04W_OnTrigger 從施法者當下位置射出追蹤火焰（bDodgeable、不給視野）。" },
        },
        events = {
          on_cast = {
            { op = "dodge_projectiles", of = "caster", from_team = "any" },
            { op = "let", name = "dist", value = "distance(caster, point)" },
            { op = "apply_status", status = "b04w_aura", to = "caster", duration = "dist / $dash_speed" },
            { op = "dash", who = "caster", to = "point", speed = "$dash_speed" },
            { op = "cue", id = "b04w_path", at = "caster" },
          },
        },
        coverage = "rust_handler",
        handler = { module = "No5_flame_trail_dash", reads = { "$dash_range", "$fire_radius", "$fire_lifetime", "$indicator_width" },
          notes = "dash_range 不被 handler 讀取（只是 v2 數字的對應），衝刺距離由施法距離決定；indicator_width 由 host 以 v2 名稱 shape_width 讀取。" },
        confidence = "medium",
        quirks = {
          { policy = "port_adaptation", ref = "vscripts/heroes/B_Unified/B04.lua:52-70",
            text = "原作衝刺期間帶光環，敵人進入時從施法者當下位置發射火焰；現行在施法時依核准的衝刺路徑決定目標（路徑兩側觸發範圍內），火焰從衝刺起點送出。逐 tick 觸發是 needs_framework。" },
          { policy = "port_adaptation", ref = "vscripts/heroes/B_Unified/B04.lua:52-70",
            text = "原作沒有目標上限與投射物壽命；現行取 host 每來源上限與 G-1 壽命，皆為引擎限制。原作沒有隨機，現行依路徑接觸先後決定性排序。" },
          { policy = "as_source", ref = "npc/B_Unified/B04.txt:157-165",
            text = "modifier_B04W2（STUNNED＋COMMAND_RESTRICTED）在 B04.lua:31 被註解，衝刺不暈眩自己。" },
          { policy = "as_source", ref = "npc/B_Unified/B04.txt:129-130",
            text = "光環只選敵方英雄與一般單位，不含建築；只鎖定施法方隊伍可見的敵人（replica 一致）。" },
          -- 以下由 census-v3 B04 移入（census 的 52-70 port_adaptation 與上面兩條同 ref 同一件事，不重複）
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B04.lua:3-50",
            text = "以 modifier_knockback（中心設在身後一單位、高度 0、不暈眩）把自己推到目標點，速度 3000，時間 = 距離 / 3000；光環 modifier 的持續時間同樣是這段時間。施放方向為零時改用面向、推一單位。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B04.lua:52-86",
            text = "每個進入光環的敵人在 modifier 建立當下由施法者位置射出一道追蹤火焰（速度 3000 × 0.5），沒有目標上限、沒有壽命、沒有隨機；離開後超過光環殘留時間再進入會再射一次。傷害走 ApplyDamage（不經 AMHC），政宗死亡後火焰仍會傷害。光環預設不含魔免敵人。" },
          { policy = "design_intent", ref = "npc/B_Unified/B04.txt:124-131",
            text = "Dota 光環每隔一段時間才掃描範圍，衝刺很快時可能漏掉擦身而過的敵人；資料照設計意圖寫成進入範圍就觸發（規則層 aura 的掃描頻率）。" },
        },
        provenance = { "npc/B_Unified/B04.txt:48-167", "vscripts/heroes/B_Unified/B04.lua:3-86" },
      },
      -------------------------------------------------------------------------
      { id = "b04_fire_blast", source_id = "B04E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B04.txt:183" },
          mana_cost = { 140, 155, 170, 185, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B04.txt:184" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B04.txt:187" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B04.txt:188" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical",
            source = "kv:AbilityDamage@npc/B_Unified/B04.txt:185", v2_key = "damage_amount" },
          radius = { 500, unit = "dist", source = "kv:radius@npc/B_Unified/B04.txt:199", v2_key = "shape_radius" },
          stun = { 0.5, 0.8, 1.1, 1.4, unit = "s", source = "kv:stun_time@npc/B_Unified/B04.txt:204", v2_key = "control_duration" },
        },
        text = {
          zh_TW = { name = "火焰強襲",
            body = "在指定地點施展巨型火焰，暈眩範圍內的敵方英雄與一般單位 {stun} 秒，並造成 {damage} 魔法傷害。" },
          en = { name = "Fire Blast",
            body = "Unleashes a great blaze at the target point, stunning enemy heroes and units in the area for {stun} seconds and dealing {damage} magic damage." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b04e_stun", runtime_id = "stun", text = { zh_TW = { name = "暈眩", body = "無法移動、攻擊或施法。" }, en = { name = "Stunned", body = "Cannot move, attack or cast." } }, duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（ApplyDataDrivenModifier 直接套用）；受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "for_each",
              select = { from = "point", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" } },
              actions = {
                { op = "apply_status", status = "b04e_stun", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "cue", id = "b04e_impact", at = "it" },
              } },
            { op = "cue", id = "b04e_blast", at = "point", radius = "$radius" },
            { op = "destroy_trees", at = "point", radius = "$radius" },
          },
        },
        coverage = "expressible",
        confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B04.lua:96-123",
            text = "只打敵方英雄與一般單位，不含建築；先暈眩再傷害。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B04.lua:137",
            text = "原作砍樹；本遊戲沒有可破壞樹木。" },
          -- 以下由 census-v3 B04 移入（census 的 137 port_adaptation 與上面同 ref 同一件事，不重複）
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B04.lua:88-138",
            text = "FindUnitsInRadius 用技能的 ENEMY、HERO | BASIC 與預設旗標：不含建築、不含魔免。先暈眩再傷害，傷害走 ApplyDamage（不經 AMHC）。" },
        },
        provenance = { "npc/B_Unified/B04.txt:170-238", "vscripts/heroes/B_Unified/B04.lua:88-138" },
      },
      -------------------------------------------------------------------------
      { id = "b04_flame_swiftness", source_id = "B04R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          agility = { 10, 20, 30, 40, unit = "stat", source = "kv:add_agi@npc/B_Unified/B04.txt:254", v2_key = "custom_agility_bonus" },
        },
        text = {
          zh_TW = { name = "炎之迅捷",
            body = "被動：永久提升 {agility} 點敏捷（依屬性規則換算為護甲與攻擊速度；敏捷是政宗的主屬性，也提高攻擊力）。" },
          en = { name = "Flame Swiftness",
            body = "Passive: permanently grants +{agility} agility (converted to armor and attack speed by the attribute rules; agility is Masamune's primary attribute, so it also raises attack damage)." },
        },
        target = { type = "none" },
        passive = { reveal = "public", stats = { agi = "$agility" } },
        coverage = "expressible",
        confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B04.txt:260-269",
            text = "MODIFIER_PROPERTY_STATS_AGILITY_BONUS，升級以新等級數值取代、不疊加；換算係數由全域 attribute_rules 決定。" },
          -- 以下由 census-v3 B04 移入（census 的 260-269 as_source 與上面同 ref 同一件事，不重複）
          { policy = "port_adaptation",
            text = "現行以 StatsAgilityBonus 永久 buff 實作，升級以同一 buff id 取代，換算交給 host attribute_rules；換算係數是否對齊原作由全域規則決定。" },
        },
        provenance = { "npc/B_Unified/B04.txt:241-271" },
      },
      -------------------------------------------------------------------------
      { id = "b04_tanegashima", source_id = "B04T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        flags = { "pierce_magic_immune" },
        values = {
          cooldown = { 110, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B04.txt:284" },
          mana_cost = { 150, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B04.txt:285" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B04.txt:288" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel；依 history_nobu 慣例 RequiredLevel 6、LevelsBetweenUpgrades 6（假設 tanegashima_levels）" },
          duration = { 28, 32, 36, unit = "s", source = "kv:duration@npc/B_Unified/B04.txt:301", v2_key = "stat_buff_duration" },
          range_bonus = { 672, unit = "dist", source = "kv:add_attack_range@npc/B_Unified/B04.txt:296",
            v2_key = "stat_buff_attack_range_bonus" },
          attack_bonus = { 50, 70, 90, unit = "dmg", source = "kv:add_base_attack@npc/B_Unified/B04.txt:306",
            v2_key = "stat_buff_attack_damage_bonus" },
          stun_chance = { 0.38, unit = "ratio", hidden = true,
            source = "kv:stun_chance@npc/B_Unified/B04.txt:311", kv = { 38 },
            source_differs = "KV 38 是百分比（stun_chance >= RandomInt(1,100)，機率相同）；handler 以 StatKey AttackStunChance 的 0～1 比例讀取（self_transform Encoding::Normalized），v2 存 0.38，M1 不改 handler，顯示用 stun_pct",
            v2_key = "stat_buff_attack_stun_chance" },
          stun_pct = { expr = "$stun_chance", unit = "pct", source = "derived", note = "stun_chance 的百分比顯示" },
          stun_time = { 0.2, unit = "s", source = "kv:stun_time@npc/B_Unified/B04.txt:316", v2_key = "stat_buff_attack_stun_duration" },
          stun_on_launch = { 1, unit = "number", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B04.lua:171-175 發射時擲骰並套用，不看命中", v2_key = "custom_stun_on_launch" },
          stun_skip_buildings = { 1, unit = "number", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B04.lua:171-175 只限 IsHero() or IsCreep()", v2_key = "custom_stun_skip_buildings" },
          bullet_damage = { 35, 50, 65, unit = "dmg", damage_type = "magical",
            source = "kv:damage@npc/B_Unified/B04.txt:321", v2_key = "custom_bullet_damage" },
          bullet_distance = { 1200, unit = "dist", source = "script:vscripts/heroes/B_Unified/B04.lua:181-196", v2_key = "custom_bullet_distance" },
          bullet_radius = { 100, unit = "dist", source = "script:vscripts/heroes/B_Unified/B04.lua:181-196", v2_key = "custom_bullet_radius" },
          bullet_speed = { 2000, unit = "speed", source = "script:vscripts/heroes/B_Unified/B04.lua:181-196", v2_key = "custom_bullet_speed" },
          bullet_lifetime = { 5, unit = "s", hidden = true, source = "design:G-1 投射物必須有壽命；足以飛完 bullet_distance",
            v2_key = "custom_bullet_lifetime" },
          extra_shot_hits = { 26, unit = "count", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B04.lua:199-210 RandomInt(0,100) <= 25 的成功結果數", v2_key = "custom_extra_shot_hits" },
          extra_shot_outcomes = { 101, unit = "count", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B04.lua:199-210 RandomInt(0,100) 的結果數", v2_key = "custom_extra_shot_outcomes" },
          extra_shot_chance = { expr = "$extra_shot_hits / $extra_shot_outcomes", unit = "pct", source = "derived" },
          extra_delay_first = { 0.05, unit = "s", source = "script:vscripts/heroes/B_Unified/B04.lua:199-204",
            v2_key = "custom_extra_shot_delay_first" },
          extra_delay_second = { 0.1, unit = "s", source = "script:vscripts/heroes/B_Unified/B04.lua:205-210",
            v2_key = "custom_extra_shot_delay_second" },
        },
        text = {
          zh_TW = { name = "種子島",
            body = "換上種子島鐵砲 {duration} 秒：攻擊距離 +{range_bonus}，基礎攻擊力 +{attack_bonus}。每次普攻發射時有 {stun_pct} 機率使目標暈眩 {stun_time} 秒（不含建築，不看是否命中），並沿攻擊方向射出一發穿透直線子彈（半徑 {bullet_radius}、速度 {bullet_speed}、距離 {bullet_distance}），對路徑上的敵方英雄與一般單位造成 {bullet_damage} 魔法傷害；另各有 {extra_shot_chance} 機率在 {extra_delay_first} 秒與 {extra_delay_second} 秒後追加一發。" },
          en = { name = "Tanegashima",
            body = "Equips the Tanegashima matchlock for {duration} seconds: +{range_bonus} attack range and +{attack_bonus} base attack damage. Each attack fired has a {stun_pct} chance to stun the target for {stun_time} seconds (not buildings, hit or miss) and launches a piercing shot along the attack direction (radius {bullet_radius}, speed {bullet_speed}, travels {bullet_distance}) that deals {bullet_damage} magic damage to enemy heroes and units in its path; two independent {extra_shot_chance} rolls can each add another shot after {extra_delay_first} and {extra_delay_second} seconds." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "b04t_gun", runtime_id = "b04_tanegashima", text = { zh_TW = { name = "種子島", body = "攻擊距離 +{range_bonus}，基礎攻擊力 +{attack_bonus}。每次普攻發射時有 {stun_pct} 機率使目標暈眩 {stun_time} 秒，並射出穿透子彈，對路徑上的敵人造成 {bullet_damage} 魔法傷害。" }, en = { name = "Tanegashima", body = "+{range_bonus} attack range and +{attack_bonus} base attack damage. Each attack fired has a {stun_pct} chance to stun the target for {stun_time} seconds and launches a piercing shot that deals {bullet_damage} magic damage to enemies in its path." } }, duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b04t_gun_model", states = { "ranged_attack" },
            stats = { base_attack_damage = "$attack_bonus", attack_range = "$range_bonus" },
            triggers = {
              on_attack_launch = { actions = {
                -- census 寫 $stun_chance（pct 38 → 運算式讀到 0.38）；kit 的 stun_chance 是 ratio 0.38（不換算），語意相同
                { op = "chance", p = "$stun_chance", actions = {
                  { op = "if", cond = "event.target.is_hero or event.target.is_creep", actions = {
                    { op = "apply_status", status = "b04t_stun", to = "event.target" },
                  } },
                } },
                { op = "let", name = "origin", value = "position(self)" },
                { op = "let", name = "dir", value = "direction(self, event.target)" },
                { op = "call", routine = "b04t_bullet", args = { at = "origin", toward = "dir" } },
                -- census 寫 $extra_chance（pct "2600/101" → 運算式讀到 26/101）；kit 拆成 hits／outcomes 兩個 count，
                -- 直接相除得精確的 26/101。kit 的 extra_shot_chance（pct 顯示值 25.74…，運算式再除以 100）結果相同，
                -- 但多經一次 Fixed64 換算的捨入，所以用 hits／outcomes。
                { op = "chance", p = "$extra_shot_hits / $extra_shot_outcomes", actions = {
                  { op = "delay", time = "$extra_delay_first", capture = { "origin", "dir" }, actions = {
                    { op = "call", routine = "b04t_bullet", args = { at = "origin", toward = "dir" } },
                  } },
                } },
                { op = "chance", p = "$extra_shot_hits / $extra_shot_outcomes", actions = {
                  { op = "delay", time = "$extra_delay_second", capture = { "origin", "dir" }, actions = {
                    { op = "call", routine = "b04t_bullet", args = { at = "origin", toward = "dir" } },
                  } },
                } },
                { op = "cue", id = "b04t_shot", at = "self" },
              } },
            },
            notes = "原作 modifier_B04T：BASEATTACK_BONUSDAMAGE（白字）、ATTACK_RANGE_BONUS、動作替換；OnCreated SetAttackCapability(RANGED)，OnDestroy 改回 MELEE；OnAttack（普攻發射時）呼叫 B04T_OnAttack。沒寫 IsPurgable：依 §12.3 暫定 basic（推定）。" },
          { id = "b04t_stun", runtime_id = "stun", text = { zh_TW = { name = "暈眩", body = "無法移動、攻擊或施法。" }, en = { name = "Stunned", body = "Cannot move, attack or cast." } }, duration = "$stun_time", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（ApplyDataDrivenModifier 直接套用，不檢查魔免）；受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b04t_gun", to = "caster" },
            { op = "cue", id = "b04t_title", at = "caster" },
          },
        },
        coverage = "rust_handler",
        handler = { module = "No8_tanegashima",
          reads = { "$stun_chance", "$stun_on_launch", "$stun_skip_buildings", "$bullet_lifetime", "$extra_shot_hits", "$extra_shot_outcomes" },
          notes = "self_transform（射程、基礎攻擊、host 攻擊暈眩）；host 普攻發射 hook AbilityScript::on_attack_launch 射出 G-1 直線子彈，追加子彈用 G-1 發射延遲。" },
        confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B04.txt:296",
            text = "射程加成照 KV；英雄基礎射程是原作近戰值，加成後等於原作說明的最終射程。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B04.lua:171-175",
            text = "暈眩在普攻發射時擲骰並立即套用，未命中也會暈，只對英雄與小兵；不檢查魔免（ENEMIES_YES），子彈魔法傷害對魔免歸零。" },
          { policy = "port_adaptation", ref = "vscripts/heroes/B_Unified/B04.lua:177",
            text = "原作 projectile_table 是全域覆寫，兩位種子島同時攻擊時延遲子彈會用錯方向；判定為缺陷，現行不移植。" },
          { policy = "port_adaptation", ref = "npc/B_Unified/B04.txt:278-288",
            text = "KV 沒寫 RequiredLevel；現行採 6/12/18（同 kit 的 A02T、B01T 也是）。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B04.lua:199-210",
            text = "追加子彈在等待延遲期間會以靜止投射物出現在發射點（launch fact 在建立時送出），屬呈現。" },
          -- 以下由 census-v3 B04 移入
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B04.lua:157-175",
            text = "暈眩在出手時擲 stun_chance >= RandomInt(1,100)（恰好 38%），不看是否命中；只檢查 IsHero 或 IsCreep，不檢查敵我與魔免：暈得到魔免單位，反補友方小兵時也會暈它，建築不暈。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B04.lua:163-198",
            text = "子彈從政宗位置沿政宗→目標方向飛行（角度換算用 3.14 近似 π，方向誤差可忽略），敵方 HERO + BASIC、穿透、不給視野；對任何目標出手都會射。命中時 B04T_OnHit 再檢查 IsMagicImmune，經 AMHC:Damage（政宗已死不傷害）。" },
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B04.lua:177-210",
            text = "FC-008（D13 修正：資料本來就照意圖寫，維持現狀）。原作：追加子彈的 Timer 讀全域變數 projectile_table：0.05／0.1 秒內若另一次 B04T 出手，或 C02、C05、C14 的腳本覆寫了同名全域表，追加子彈會用別人的位置與方向（甚至別人的技能）。資料以出手當下的位置與方向寫（原作的共用全域表無法以資料重現）。" },
          { policy = "note", ref = "npc/B_Unified/B04.txt:287",
            text = "KV AbilityCastRange 800 只是說明用的最終射程（128 基礎射程 + 672）。KV 沒寫 RequiredLevel（見 required_level 的 source）。" },
          { policy = "port_adaptation", ref = "vscripts/heroes/B_Unified/B04.lua:140-154",
            text = "現行不切換成遠程攻擊，只加射程 672（近戰攻擊距離 800）；暈眩以 attack_stun_on_launch／attack_stun_skip_buildings 在發射時套用；追加子彈的機率以 rand_unit 264/1024（約 25.8%，原作 26/101 約 25.7%）；延遲子彈在等待的幾個 tick 內以靜止投射物出現在發射點。" },
        },
        provenance = { "npc/B_Unified/B04.txt:274-440", "vscripts/heroes/B_Unified/B04.lua:157-221",
          "vscripts/heroes/B_Unified/B04.lua:140-221" },
      },
    },
    routines = {
      b04t_bullet = {
        owner = "b04_tanegashima", params = { at = "point", toward = "vec" },
        actions = {
          { op = "projectile", kind = "linear", from = "at", direction = "toward", speed = "$bullet_speed",
            distance = "$bullet_distance", radius = "$bullet_radius", pierce = true, team = "enemy",
            types = { "hero", "creep", "neutral", "summon" }, cue = "b04t_bullet",
            on_hit = {
              { op = "if", cond = "not it.is_magic_immune", actions = {
                { op = "damage", to = "it", type = "magical", amount = "$bullet_damage" },
              } },
            } },
        },
      },
    },
  }
end
