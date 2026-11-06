-- B04 伊達政宗（舊版，11 版 *_old；KV 註解標「11.2B」）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["B04"]["11"] = "WERDT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:283），自動學習 heromap_autoskill["11"] = ""（addon_game_mode.lua:386）。
-- 順序 W、E、R、D、attribute_bonusx、T，所以 W/E/R/T → Q/W/E/R，D 留在 D。
-- B04E_old（踏火無痕）是 NOT_LEARNABLE：B04W_old 的 OnUpgrade（B04.lua:225-233）把它設成和火焰刀同等級，所以 W 槽 learnable = false、level_link 指向 Q。
-- B04D_old（靈活加成）是一般可學的被動（沒有 NOT_LEARNABLE），本體 current 版沒有 D，照 B08_old 等舊版慣例放 D 槽。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 B04*_old），招式名取 KV 註解，說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、B04_precache 屬規則層或資產，不列入。
-- 慣例照本體 B04：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- datadriven Damage／ApplyDamage（不經 AMHC）在政宗死亡後仍會造成傷害，寫 require_source_alive = false；HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  local STUN_TEXT = { zh_TW = { name = "暈眩", body = "無法移動、攻擊或施法。" }, en = { name = "Stunned", body = "Cannot move, attack or cast." } }
  return {
    schema_version = 3, kind = "hero", id = "B04_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B04", text = { zh_TW = { name = "伊達政宗（舊版）" }, en = { name = "Date Masamune (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b04_old_flame_blade", source_id = "B04W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B04.txt:457" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B04.txt:458" },
          cast_range = { 1100, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B04.txt:459" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B04.txt:454" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical",
            source = "kv:AbilityDamage@npc/B_Unified/B04.txt:571" },
          speed = { 2000, unit = "speed", source = "kv:shock_speed@npc/B_Unified/B04.txt:553" },
          radius = { 205, unit = "dist", source = "kv:shock_width@npc/B_Unified/B04.txt:558" },
          distance = { 1100, unit = "dist", source = "kv:shock_distance@npc/B_Unified/B04.txt:563" },
          vision_radius = { 205, unit = "dist", source = "kv:shock_vision@npc/B_Unified/B04.txt:568" },
        },
        text = {
          zh_TW = { name = "火焰刀",
            body = "朝指定方向斬出一道火焰刀氣（半徑 {radius}、飛行 {distance}、速度 {speed}），穿透路徑上的敵方英雄與一般單位，各造成 {damage} 魔法傷害一次；刀氣飛行時提供 {vision_radius} 視野。升級時踏火無痕同步升到相同等級。" },
          en = { name = "Flame Blade",
            body = "Slashes a wave of flame in the target direction (radius {radius}, travels {distance} at speed {speed}) that pierces enemy heroes and basic units, dealing {damage} magic damage to each once; the wave grants {vision_radius} vision while flying. Leveling it also raises Flame Trail to the same level." },
        },
        target = { type = "unit_or_point", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_phase_start = { { op = "cue", id = "b04w_old_blade_glow", at = "caster" } },
          on_cast = {
            { op = "cue", id = "b04w_old_cast_sound", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$speed", distance = "$distance", radius = "$radius", pierce = true, team = "enemy", types = BASIC,
              vision = "$vision_radius", cue = "b04w_old_wave",
              on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "cue", id = "b04w_old_hit_sound", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B04.txt:447",
            text = "AbilityBehavior 是 UNIT_TARGET | POINT：可以點單位或點地，LinearProjectile Target POINT 都朝施放點（點單位時是單位位置）發射，固定飛行 shock_distance。" },
          { policy = "as_source", ref = "npc/B_Unified/B04.txt:491-519",
            text = "StartRadius = EndRadius = 205 當半徑；TargetFlags NONE：魔免敵人不被命中，隱形與迷霧中的敵人照樣命中。DeleteOnHit 0 穿透。命中是 datadriven Damage（不經 AMHC），政宗在刀氣飛行中死亡仍照常傷害。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B04.lua:225-233",
            text = "OnUpgrade 把 B04E_old 設成同等級（B04E_old 是 NOT_LEARNABLE），以踏火無痕的 level_link 表達。" },
          { policy = "note", ref = "npc/B_Unified/B04.txt:472-547",
            text = "前搖的 modifier_shockwave_hidden_cast（0.5 秒、隱藏）只掛刀光粒子；FireSound 與 item_soul_adder sound 只是呈現。LevelsBetweenUpgrades 2 對一般技能沒有作用。" },
        },
        provenance = { "npc/B_Unified/B04.txt:442-572", "vscripts/heroes/B_Unified/B04.lua:225-233" },
      },
      -------------------------------------------------------------------------
      { id = "b04_old_flame_trail", source_id = "B04E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic", "movement" },
        learnable = false, level_link = "b04_old_flame_blade",
        values = {
          cooldown = { 30, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B04.txt:586" },
          mana_cost = { 80, 100, 120, 140, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B04.txt:587" },
          cast_range = { 900, 1000, 1100, 1200, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B04.txt:590" },
          cast_point = { 0, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B04.txt:591" },
          damage = { 50, 70, 90, 110, unit = "dmg", damage_type = "magical",
            source = "kv:AbilityDamage@npc/B_Unified/B04.txt:588" },
          tick = { 0.1, unit = "s", source = "kv:damage_delay@npc/B_Unified/B04.txt:599" },
          hit_radius = { 75, unit = "dist", source = "kv:hit_radius@npc/B_Unified/B04.txt:604" },
          speed = { 1500, unit = "speed", source = "kv:speed@npc/B_Unified/B04.txt:609" },
          aura_linger = { 0.5, unit = "s", hidden = true,
            source = "assumed:Dota datadriven 光環離開範圍後 modifier 殘留約 0.5 秒（引擎預設，KV 沒寫）" },
        },
        text = {
          zh_TW = { name = "踏火無痕",
            body = "以 {speed} 的速度直線衝到指定地點。衝刺期間身旁 {hit_radius} 內的敵方英雄與一般單位每 {tick} 秒受到 {damage} 魔法傷害（離開範圍後仍會短暫持續）。等級跟著火焰刀。" },
          en = { name = "Flame Trail",
            body = "Dashes in a straight line to the target point at speed {speed}. While dashing, enemy heroes and basic units within {hit_radius} take {damage} magic damage every {tick} seconds (lingering briefly after they leave the area). Its level follows Flame Blade." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b04e_old_aura", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public",
            aura = { radius = "$hit_radius", team = "enemy", types = BASIC, status = "b04e_old_dot", linger = "$aura_linger" },
            notes = "原作 modifier_B04W_old_aura：IsHidden，Aura modifier_B04W_old_dot（ENEMY、HERO | BASIC、不含自己），持續時間 = 衝刺時間；OnDeath 移除。沒寫 IsPurgable：暫定 basic（推定）。施放時以 duration 覆寫。" },
          { id = "b04e_old_dot", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public",
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_B04W_old_dot：IsHidden、IsDebuff，ThinkInterval damage_delay，OnIntervalThink datadriven Damage %AbilityDamage。沒寫 IsPurgable：暫定 basic（推定）。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "b04e_old_vo", at = "caster" } },
          on_cast = {
            { op = "let", name = "dist", value = "distance(caster, point)" },
            { op = "apply_status", status = "b04e_old_aura", to = "caster", duration = "dist / $speed" },
            { op = "dash", who = "caster", to = "point", speed = "$speed" },
            { op = "cue", id = "b04e_old_trail", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B04.lua:235-270",
            text = "以 modifier_knockback（中心設在身後一單位、高度 0、不暈眩）把自己推到施放點，時間 = 距離 / 1500；光環 modifier 的持續時間同樣是這段時間。施放方向為零時改用面向、推一單位（資料的 dash 到原地即結束）。" },
          { policy = "as_source", ref = "npc/B_Unified/B04.txt:632-667",
            text = "傷害是光環子 modifier 的 ThinkInterval 0.1：範圍內每 0.1 秒一次 AbilityDamage（每秒十倍），離開範圍後光環殘留期間繼續；第一次傷害在進入後一個間隔。光環預設不含魔免敵人，傷害是 datadriven Damage（不經 AMHC）。" },
          { policy = "as_source", ref = "npc/B_Unified/B04.txt:581",
            text = "NOT_LEARNABLE，等級由火焰刀的 OnUpgrade 設定；火焰刀零級時本招也是零級、不能施放。" },
          { policy = "design_intent", ref = "npc/B_Unified/B04.txt:634-640",
            text = "Dota 光環每隔一段時間才掃描範圍，殘留時間是引擎預設（推定 0.5 秒）；資料照設計意圖寫成進入範圍就套用、殘留 aura_linger。" },
          { policy = "note", ref = "npc/B_Unified/B04.txt:581",
            text = "KV 名稱是 B04E_old、說明註解是踏火無痕，但腳本函式叫 B04W_old_OnSpellStart、modifier 叫 modifier_B04W_old_*（沿用本體 B04W 的命名）。" },
        },
        provenance = { "npc/B_Unified/B04.txt:573-667", "vscripts/heroes/B_Unified/B04.lua:235-270" },
      },
      -------------------------------------------------------------------------
      { id = "b04_old_fire_blast", source_id = "B04R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 14, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B04.txt:682" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B04.txt:683" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B04.txt:686" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B04.txt:687" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical",
            source = "kv:AbilityDamage@npc/B_Unified/B04.txt:684" },
          radius = { 500, unit = "dist", source = "kv:radius@npc/B_Unified/B04.txt:698" },
          stun = { 0.3, 0.6, 0.9, 1.2, unit = "s", source = "kv:stun_time@npc/B_Unified/B04.txt:703" },
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
          { id = "b04r_old_stun", runtime_id = "stun", text = STUN_TEXT, duration = "$stun", stacking = "refresh", dispel = "strong",
            visibility = "debuff", reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（ApplyDataDrivenModifier 直接套用）；受狀態抗性縮短。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "b04r_old_vo", at = "caster" } },
          on_cast = {
            { op = "for_each",
              select = { from = "point", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
              actions = {
                { op = "apply_status", status = "b04r_old_stun", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "cue", id = "b04r_old_impact", at = "it" },
              } },
            { op = "cue", id = "b04r_old_blast", at = "point", radius = "$radius" },
            { op = "destroy_trees", at = "point", radius = "$radius" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B04.lua:88-138",
            text = "和本體共用 B04E_OnSpellStart：FindUnitsInRadius 用技能的 ENEMY、HERO | BASIC 與預設旗標（不含建築、不含魔免），先暈眩再傷害，傷害走 ApplyDamage（不經 AMHC）。舊版數值：冷卻 14、前搖 0.25、傷害 200～500、暈眩 0.3～1.2。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B04.lua:137",
            text = "原作砍樹；本遊戲沒有可破壞樹木。" },
        },
        provenance = { "npc/B_Unified/B04.txt:669-731", "vscripts/heroes/B_Unified/B04.lua:88-138" },
      },
      -------------------------------------------------------------------------
      { id = "b04_old_agility_boost", source_id = "B04D_old", slot = "D", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          agility = { 8, 16, 24, 32, unit = "stat", source = "kv:add_agi@npc/B_Unified/B04.txt:747" },
        },
        text = {
          zh_TW = { name = "靈活加成",
            body = "被動：永久提升 {agility} 點敏捷（依屬性規則換算為護甲與攻擊速度；敏捷是政宗的主屬性，也提高攻擊力）。" },
          en = { name = "Agility Boost",
            body = "Passive: permanently grants +{agility} agility (converted to armor and attack speed by the attribute rules; agility is Masamune's primary attribute, so it also raises attack damage)." },
        },
        target = { type = "none" },
        passive = { reveal = "public", stats = { agi = "$agility" } },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B04.txt:750-763",
            text = "MODIFIER_PROPERTY_STATS_AGILITY_BONUS（modifier 名 modifier_B04R_old_passive），升級以新等級數值取代、不疊加。KV 沒寫 MaxLevel，一般技能預設四級。" },
        },
        provenance = { "npc/B_Unified/B04.txt:733-764" },
      },
      -------------------------------------------------------------------------
      { id = "b04_old_matchlock", source_id = "B04T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 110, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B04.txt:776" },
          mana_cost = { 260, 310, 360, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B04.txt:777" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B04.txt:780" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel；依 history_nobu 大招慣例 6/12/18（同本體 B04T）" },
          duration = { 45, unit = "s", source = "kv:duration@npc/B_Unified/B04.txt:793" },
          range_bonus = { 572, 672, 772, unit = "dist", source = "kv:add_attack_range@npc/B_Unified/B04.txt:788" },
          attack_bonus = { 90, 130, 170, unit = "dmg", source = "kv:add_base_attack@npc/B_Unified/B04.txt:798" },
          stun_chance = { 87, unit = "pct",
            source = "kv:stun_chance@npc/B_Unified/B04.txt:803（stun_chance >= RandomInt(1,100)，恰好 87%）" },
          stun_time = { 0.1, 0.2, 0.3, unit = "s", source = "kv:stun_time@npc/B_Unified/B04.txt:808" },
        },
        text = {
          zh_TW = { name = "火繩銃",
            body = "換上火繩銃 {duration} 秒：改為遠程攻擊，攻擊距離 +{range_bonus}，基礎攻擊力 +{attack_bonus}，普攻不會落空。每次普攻發射時有 {stun_chance} 機率使目標暈眩 {stun_time} 秒（只對英雄與小兵，不對魔法免疫的目標，不看是否命中）。" },
          en = { name = "Matchlock",
            body = "Equips a matchlock for {duration} seconds: attacks become ranged, +{range_bonus} attack range, +{attack_bonus} base attack damage, and attacks cannot miss. Each attack fired has a {stun_chance} chance to stun the target for {stun_time} seconds (heroes and creeps only, not spell-immune targets, hit or miss)." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "b04t_old_gun",
            text = {
              zh_TW = { name = "火繩銃", body = "遠程攻擊，攻擊距離 +{range_bonus}，基礎攻擊力 +{attack_bonus}，普攻不會落空。每次普攻發射時有 {stun_chance} 機率使目標暈眩 {stun_time} 秒。" },
              en = { name = "Matchlock", body = "Ranged attacks, +{range_bonus} attack range, +{attack_bonus} base attack damage, attacks cannot miss. Each attack fired has a {stun_chance} chance to stun the target for {stun_time} seconds." },
            },
            duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b04t_old_strafe", states = { "ranged_attack", "cannot_miss" },
            stats = { base_attack_damage = "$attack_bonus", attack_range = "$range_bonus" },
            triggers = {
              on_attack_launch = { actions = {
                { op = "if", cond = "(event.target.is_hero or event.target.is_creep) and not event.target.is_magic_immune", actions = {
                  { op = "chance", p = "$stun_chance", actions = {
                    { op = "apply_status", status = "b04t_old_stun", to = "event.target" },
                  } },
                } },
                { op = "cue", id = "b04t_old_shot", at = "self" },
              } },
            },
            notes = "原作 modifier_B04T_old：BASEATTACK_BONUSDAMAGE（白字）、ATTACK_RANGE_BONUS、動作替換、MODIFIER_STATE_CANNOT_MISS；OnCreated SetAttackCapability(RANGED)，OnDestroy 改回 MELEE；OnAttack（普攻發射時）呼叫 B04T_old_OnAttackStart。沒寫 IsPurgable：暫定 basic（推定，同本體）。" },
          { id = "b04t_old_stun", runtime_id = "stun", text = STUN_TEXT, duration = "$stun_time", stacking = "refresh", dispel = "strong",
            visibility = "debuff", reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（ApplyDataDrivenModifier 直接套用）；受狀態抗性縮短。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "b04t_old_vo", at = "caster" } },
          on_cast = {
            { op = "apply_status", status = "b04t_old_gun", to = "caster" },
            { op = "cue", id = "b04t_old_title", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B04.lua:272-283",
            text = "暈眩在普攻發射時擲 stun_chance >= RandomInt(1,100)（恰好 87%），不看是否命中；只檢查 IsHero 或 IsCreep 與 IsMagicImmune，不檢查敵我：反補友方小兵時也會暈它，建築不暈。和本體不同：舊版沒有穿透子彈。" },
          { policy = "as_source", ref = "npc/B_Unified/B04.txt:823-835",
            text = "ApplyModifier 的 Target 區塊只有 Center CASTER（沒有 Radius），實際只套在政宗自己身上。" },
          { policy = "as_source", ref = "npc/B_Unified/B04.txt:788",
            text = "射程加成照 KV（註解寫基礎射程 128 + 672 = 800），英雄基礎射程是原作近戰值；KV AbilityCastRange 800 只是說明用的最終射程，不列入。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B04.lua:140-154",
            text = "OnCreated／OnDestroy 切換遠程與近戰攻擊，以狀態 ranged_attack 表達；結束時固定改回近戰。" },
          { policy = "note", ref = "npc/B_Unified/B04.txt:813-841,888-893",
            text = "clinkz_strafe 粒子、showTitle 標題、berserkercaster 與 rocket_barrage_target01 音效屬呈現，改用 cue。" },
        },
        provenance = { "npc/B_Unified/B04.txt:766-897", "vscripts/heroes/B_Unified/B04.lua:140-154", "vscripts/heroes/B_Unified/B04.lua:272-283" },
      },
    },
  }
end
