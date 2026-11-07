-- C15 玉子（舊版，11 版 *_old，KV 註記「玉子 11.2B」）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C15"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:331）；
-- heromap_autoskill["C15"]["11"] = ""（addon_game_mode.lua:434），沒有出生自動學的技能，也沒有 D。
-- KV 在 npc/C_Neutral/C15.txt:469-798（C15W_old／C15E_old／C15R_old／C15T_old）。W/E/R/T → Q/W/E/R。
-- 原作舊版沒有任何繁中說明（panorama/localization/addon_tchinese.txt 查無 C15*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C15_precache 屬規則層或資產，不列入。
-- 慣例照本體 C15：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned（datadriven Stun）寫 "strong"；
-- HERO | BASIC 寫成 BASIC 清單；datadriven Damage（不經 AMHC）寫 require_source_alive = false。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C15_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C15", text = { zh_TW = { name = "玉子（舊版）" }, en = { name = "Tamako (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c15_old_triple_arrow", source_id = "C15W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 30, 27, 24, 20, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C15.txt:488" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C15.txt:491" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C15.txt:492" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C15.txt:484" },
          damage = { 50, 100, 150, 200, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C15.txt:481" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:stun_duration@npc/C_Neutral/C15.txt:550" },
          arrow_count = { 3, unit = "count", source = "script:vscripts/heroes/C_Neutral/C15.lua:162 projectile_max = 3" },
          arrow_delay = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C15.lua:163 projectile_delay = 0.1" },
          arrow_speed = { 1500, unit = "speed", hidden = true, source = "script:vscripts/heroes/C_Neutral/C15.lua:175 iMoveSpeed = 1500" },
        },
        text = {
          zh_TW = { name = "三連矢",
            body = "對目標敵人連續射出 {arrow_count} 支可閃避的追蹤箭，每支命中都擊暈 {stun} 秒並造成 {damage} 魔法傷害。" },
          en = { name = "Triple Arrow",
            body = "Fires {arrow_count} dodgeable homing arrows in quick succession at the target enemy; each one that hits stuns for {stun} seconds and deals {damage} magic damage." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c15w_old_stun", runtime_id = "stun", text = { zh_TW = { name = "暈眩", body = "無法移動、攻擊或施法。" }, en = { name = "Stunned", body = "Cannot move, attack or cast." } },
            duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" },
            notes = "原作 OnProjectileHitUnit 的 datadriven Stun（引擎 modifier_stunned）；受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c15w_old_cast_vo", at = "caster" },
            { op = "cue", id = "c15w_old_starfall_circle", at = "caster" },
            { op = "repeat", count = "$arrow_count", interval = "$arrow_delay", first = "0", actions = {
              { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$arrow_speed", dodgeable = true,
                cue = "c15w_old_arrow",
                on_hit = {
                  { op = "apply_status", status = "c15w_old_stun", to = "it" },
                  { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                  { op = "cue", id = "c15w_old_impact", at = "it" },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C15.lua:156-183",
            text = "C15W_old 以 Timers 在 0、0.1、0.2 秒各發一支追蹤彈（速度 1500、可閃避），三支都各自暈眩並傷害：總傷害三倍，暈眩時間每次重新整理。" },
          { policy = "as_source", ref = "npc/C_Neutral/C15.txt:525-544",
            text = "命中是 datadriven Stun 與 Damage（不經 AMHC），玉子在箭飛行中死亡仍照常暈眩與傷害。DeleteOnHit 1。bProvidesVision 但 iVisionRadius 0，沒有視野。" },
          { policy = "note", ref = "npc/C_Neutral/C15.txt:503-518",
            text = "FireSound 與 starfall 圓環粒子只是呈現；圖示借用 B33W。" },
        },
        provenance = { "npc/C_Neutral/C15.txt:471-553", "vscripts/heroes/C_Neutral/C15.lua:156-183" },
      },
      -------------------------------------------------------------------------
      { id = "c15_old_searing_arrow", source_id = "C15E_old", slot = "W", kind = "toggle", max_level = 4, tags = { "basic", "toggle" },
        values = {
          mana_per_attack = { 14, 16, 18, 20, unit = "mana", source = "kv:mana_per_attack@npc/C_Neutral/C15.txt:634",
            note = "vscripts/heroes/C_Neutral/C15.lua:185-197 SpendMana" },
          damage = { 60, 100, 140, 180, unit = "dmg", damage_type = "magical", source = "kv:damage_bonus@npc/C_Neutral/C15.txt:629" },
          building_pct = { 20, unit = "pct", source = "script:vscripts/heroes/C_Neutral/C15.lua:206 dmg*0.2" },
        },
        text = {
          zh_TW = { name = "烈焰箭",
            body = "開啟時，每次普攻開始都消耗 {mana_per_attack} 法力；法力不足時自動關閉。普攻命中時額外造成 {damage} 魔法傷害，對建築只造成 {building_pct}；對非英雄的部隊再多造成兩次 {damage} 魔法傷害。" },
          en = { name = "Searing Arrows",
            body = "While active, each attack Tamako starts costs {mana_per_attack} mana; if she lacks the mana, the ability turns off. Attacks that land deal {damage} bonus magic damage ({building_pct} against buildings); against non-hero units they deal {damage} bonus magic damage two more times." },
        },
        target = { type = "none" },
        statuses = {
          { id = "c15e_old_orb",
            text = { zh_TW = { name = "烈焰箭", body = "每次普攻開始消耗 {mana_per_attack} 法力；普攻命中時額外造成 {damage} 魔法傷害（對建築為 {building_pct}，對非英雄部隊共三次）。" },
              en = { name = "Searing Arrows", body = "Each attack costs {mana_per_attack} mana; attacks that land deal {damage} bonus magic damage ({building_pct} against buildings, three times against non-hero units)." } },
            duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "c15e_old_searing_arrow_projectile", remove_on = { "death" },
            triggers = {
              on_death = { { op = "toggle_ability", of = "self", ability = "c15_old_searing_arrow", on = false } },
              on_attack_windup = {
                { op = "if", cond = "self.mana < $mana_per_attack",
                  actions = {
                    { op = "toggle_ability", of = "self", ability = "c15_old_searing_arrow", on = false },
                  },
                  otherwise = {
                    { op = "spend_mana", from = "self", amount = "$mana_per_attack" },
                  } },
              },
              on_attack_hit = {
                { op = "damage", to = "event.target", type = "magical",
                  amount = "select(event.target.is_building, $damage * $building_pct, $damage)" },
                { op = "if", cond = "not event.target.is_hero and not event.target.is_building", actions = {
                  { op = "damage", to = "event.target", type = "magical", amount = "$damage" },
                  { op = "damage", to = "event.target", type = "magical", amount = "$damage" },
                } },
                { op = "cue", id = "c15e_old_split_sound", at = "event.target" },
              },
            },
            notes = "原作 modifier_C15E_old：Passive 0、Orb（CastAttack 0，只換投射物外觀）；OnAttackStart C15E_old_orb_fire 扣魔、OnAttackLanded C15E_old_orb_apply_damage 加傷；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_toggle_on = { { op = "apply_status", status = "c15e_old_orb", to = "caster" } },
          on_toggle_off = { { op = "remove_status", from = "caster", status = "c15e_old_orb" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C15.lua:185-197",
            text = "扣魔在 OnAttackStart（前搖開始）：攻擊被取消仍扣魔，打建築也扣。法力不足時只 CastAbilityToggle 關閉、不扣魔（和今川義元的烈焰箭不同，不會把剩餘法力扣光），這一擊沒有加傷（狀態已移除）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C15.lua:199-215",
            text = "加傷走 AMHC:Damage（玉子已死不傷害）：先依建築與否造成一次（建築 20%），再對「不是英雄也不是建築」的目標多打兩次，部隊實際吃三倍。沒有法球槽（nobuorb1）處理，和其他法球道具並存時兩邊都觸發。" },
          { policy = "fixed", ref = "npc/C_Neutral/C15.txt:588-591",
            text = "FC-186 D22 舊版，比照 D13 修（同今川義元烈焰箭 FC-104 的修法）：原作 modifier_C15E_old 不是 Passive，死亡時被移除，但開關仍亮（推定），復活後不扣魔也不加傷，要手動關再開；資料在死亡時關閉開關。" },
          { policy = "as_source", ref = "npc/C_Neutral/C15.txt:555-637",
            text = "原作 KV 沒有 AbilityManaCost、AbilityCooldown 與 AbilityCastPoint；開關本身不耗魔，每次攻擊扣 mana_per_attack。" },
        },
        provenance = { "npc/C_Neutral/C15.txt:554-637", "vscripts/heroes/C_Neutral/C15.lua:185-215" },
      },
      -------------------------------------------------------------------------
      { id = "c15_old_precise_light", source_id = "C15R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          damage_pct = { 20, 30, 40, 50, unit = "pct", source = "kv:per_atk@npc/C_Neutral/C15.txt:660（DAMAGEOUTGOING_PERCENTAGE@C15.txt:651）" },
        },
        text = {
          zh_TW = { name = "精準之光", body = "被動：攻擊傷害 +{damage_pct}。" },
          en = { name = "Precise Light", body = "Passive: +{damage_pct} attack damage." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { attack_damage_pct = "$damage_pct" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C15.txt:646-653",
            text = "modifier_C15R（Passive）：DAMAGEOUTGOING_PERCENTAGE 照本體寫成 attack_damage_pct（同 C17E）。和本體不同：舊版沒有敏捷加成，數值 20%～50%。" },
        },
        provenance = { "npc/C_Neutral/C15.txt:638-663" },
      },
      -------------------------------------------------------------------------
      { id = "c15_old_hundred_arrows", source_id = "C15T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate", "summon" },
        values = {
          cooldown = { 140, 115, 90, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C15.txt:682" },
          mana_cost = { 250, 300, 350, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C15.txt:685" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C15.txt:688" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C15.txt:679" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel@npc/C_Neutral/C15.txt:677 + LevelsBetweenUpgrades@C15.txt:678" },
          stun = { 1, 1.5, 2, unit = "s", source = "kv:stun_duration@npc/C_Neutral/C15.txt:785" },
          damage = { 5, unit = "dmg", damage_type = "magical", source = "kv:modifier_C15T_2 OnCreated Damage@npc/C_Neutral/C15.txt:773" },
          arrow_count = { 10, unit = "count", source = "script:vscripts/heroes/C_Neutral/C15.lua:72-90 迴圈 10 次、deg += 36" },
          ring_radius = { 300, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C15.lua:68 distance = 300" },
          angle_step = { 36, unit = "deg", hidden = true, source = "script:vscripts/heroes/C_Neutral/C15.lua:73 deg = deg + 36" },
          arrow_lifetime = { 10, unit = "s", source = "kv:life_duration@npc/C_Neutral/C15.txt:790" },
          arrow_damage = { 90, 120, 160, unit = "dmg", damage_type = "physical", source = "kv:attack_dmg@npc/C_Neutral/C15.txt:795",
            note = "modifier_C15T BASEATTACK_BONUSDAMAGE（C15.txt:726）" },
          arrow_attack_interval = { 1, unit = "s", source = "kv:C15T_dummy AttackRate@npc/npc_units_custom_load.txt:5187-5259" },
          arrow_move_speed = { 900, unit = "move_speed", source = "kv:C15T_dummy MovementSpeed@npc/npc_units_custom_load.txt:5187-5259" },
          retarget_radius = { 900, unit = "dist", source = "kv:C15T_dummy AttackAcquisitionRange@npc/npc_units_custom_load.txt:5187-5259" },
        },
        text = {
          zh_TW = { name = "百箭齊發",
            body = "擊暈目標 {stun} 秒並造成 {damage} 魔法傷害，同時在目標周圍 {ring_radius} 放出 {arrow_count} 支致命箭矢攻擊目標；目標死亡後，箭矢改找 {retarget_radius} 內的敵人。每支箭矢無敵、飛行（移速 {arrow_move_speed}、近戰）、存在 {arrow_lifetime} 秒、每 {arrow_attack_interval} 秒攻擊一次，每擊造成 {arrow_damage} 物理傷害。可對魔免的敵人施放。" },
          en = { name = "Hundred Arrows",
            body = "Stuns the target for {stun} seconds and deals {damage} magic damage, releasing {arrow_count} Deadly Arrows in a ring of {ring_radius} around it that attack it; if the target dies, the arrows look for enemies within {retarget_radius}. Each arrow is invulnerable and flies (movement speed {arrow_move_speed}, melee), lasts {arrow_lifetime} seconds and attacks once every {arrow_attack_interval} seconds for {arrow_damage} physical damage. Can target spell-immune enemies." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        flags = { "pierce_magic_immune" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "c15t_old_stun", runtime_id = "stun", text = { zh_TW = { name = "暈眩", body = "無法移動、攻擊或施法。" }, en = { name = "Stunned", body = "Cannot move, attack or cast." } },
            duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", status_resistance = "ignore", states = { "stun" }, cue = "c15t_old_stunned",
            triggers = {
              on_created = { { op = "damage", to = "self", type = "magical", amount = "$damage", require_source_alive = false } },
            },
            notes = "原作 modifier_C15T_2（C15T_old 版）：IsPurgable 1、IsStunDebuff、STUNNED；OnCreated datadriven Damage 5。名稱列在 ultimate_modifiers（vscripts/events.lua:268），不受狀態抗性縮短。" },
          { id = "c15t_old_arrow_power", text = { zh_TW = { name = "致命箭矢", body = "無敵；每擊造成 {arrow_damage} 物理傷害。" }, en = { name = "Deadly Arrow", body = "Invulnerable; each attack deals {arrow_damage} physical damage." } },
            duration = "$arrow_lifetime", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", states = { "invulnerable" }, stats = { base_attack_damage = "$arrow_damage" }, cue = "c15t_old_arrow_glow",
            triggers = {
              on_attack_launch = { { op = "cue", id = "c15t_old_arrow_attack", at = "self" } },
              on_removed = { { op = "remove_unit", to = "self" } },
            },
            notes = "原作 modifier_C15T（C15T_old 版）：INVULNERABLE、BASEATTACK_BONUSDAMAGE attack_dmg、Duration life_duration；OnDestroy new_C15T_end ForceKill＋Destroy 箭矢；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c15t_old_cast_vo", at = "caster" },
            { op = "apply_status", status = "c15t_old_stun", to = "target" },
            { op = "let", name = "center", value = "position(target)" },
            { op = "repeat", count = "$arrow_count", as = "i", actions = {
              { op = "create_unit", template = "c15_old_arrow", at = "polar(center, $angle_step * (i + 1), $ring_radius)",
                owner = "caster", facing = "caster.forward", control = "ai", as = "arrow" },
              { op = "apply_status", status = "c15t_old_arrow_power", to = "arrow" },
              { op = "issue_order", who = "arrow", order = "attack", target = "target", queue = true },
            } },
            { op = "cue", id = "c15t_old_release", at = "caster" },
            { op = "issue_order", who = "caster", order = "attack", target = "target", queue = true },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C15.lua:50-107",
            text = "和本體共用 new_C15T：先套 modifier_C15T_2（暈眩、OnCreated 魔法傷害），再在目標當下位置 + 300·(cos 36i°, sin 36i°)（i = 1..10）產生箭矢，面向玉子的面向，各自以佇列攻擊指令鎖定目標；最後對玉子自己下佇列攻擊目標指令。目標死亡後由 Dota 單位 AI 在 AttackAcquisitionRange 900 內自動找敵人。" },
          { policy = "as_source", ref = "npc/C_Neutral/C15.txt:719-777",
            text = "和本體不同：舊版沒有 modifier_C15T_trigger，沒有「每八次普攻放一支箭」的被動；耗魔 250～350、前搖 0.35、箭矢攻擊力 90/120/160。" },
          { policy = "as_source", ref = "npc/C_Neutral/C15.txt:671-675",
            text = "AbilityUnitTargetFlags MAGIC_IMMUNE_ENEMIES：可對魔免敵人施放並暈眩；箭矢普攻是物理。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C15.lua:74",
            text = "point_tem 的 z 寫成 point2.z（高度加倍），CreateUnitByName 會貼地，沒有影響。箭矢加 majia_vison（不可選取、無血條、無碰撞、攻擊免疫、魔免）。" },
        },
        provenance = { "npc/C_Neutral/C15.txt:664-798", "vscripts/heroes/C_Neutral/C15.lua:50-152",
          "npc/npc_units_custom_load.txt:5187-5259", "npc/npc_abilities_custom_load.txt:5628-5651", "vscripts/events.lua:246-270" },
      },
    },
    unit_templates = {
      { id = "c15_old_arrow", source_id = "C15T_dummy",
        values = {
          hp = { 10, unit = "hp", source = "kv:StatusHealth@npc/npc_units_custom_load.txt:5187-5259" },
          hp_regen = { -1, unit = "per_s", source = "kv:StatusHealthRegen@npc/npc_units_custom_load.txt:5187-5259" },
          armor = { 0, unit = "armor", source = "kv:ArmorPhysical@npc/npc_units_custom_load.txt:5187-5259" },
          magic_resist = { 0, unit = "pct", source = "kv:MagicalResistance@npc/npc_units_custom_load.txt:5187-5259" },
          move_speed = { expr = "$c15_old_hundred_arrows.arrow_move_speed", unit = "move_speed", source = "derived" },
          turn_rate = { 3, unit = "number", source = "kv:MovementTurnRate@npc/npc_units_custom_load.txt:5187-5259" },
          attack_range = { 10, unit = "dist", source = "kv:AttackRange@npc/npc_units_custom_load.txt:5187-5259" },
          attack_interval = { expr = "$c15_old_hundred_arrows.arrow_attack_interval", unit = "s", source = "derived" },
          attack_point = { 0, unit = "s", source = "kv:AttackAnimationPoint@npc/npc_units_custom_load.txt:5187-5259" },
          attack_min = { 0, unit = "dmg", damage_type = "physical", source = "kv:AttackDamageMin@npc/npc_units_custom_load.txt:5187-5259" },
          attack_max = { 0, unit = "dmg", damage_type = "physical", source = "kv:AttackDamageMax@npc/npc_units_custom_load.txt:5187-5259" },
          acquisition = { expr = "$c15_old_hundred_arrows.retarget_radius", unit = "dist", source = "derived" },
          vision_day = { 400, unit = "dist", source = "kv:VisionDaytimeRange@npc/npc_units_custom_load.txt:5187-5259" },
          vision_night = { 400, unit = "dist", source = "kv:VisionNighttimeRange@npc/npc_units_custom_load.txt:5187-5259" },
          bounty_gold = { 0, unit = "gold", source = "kv:BountyGoldMin@npc/npc_units_custom_load.txt:5187-5259" },
        },
        text = { zh_TW = { name = "致命箭矢（舊版）" }, en = { name = "Deadly Arrow (Old)" } },
        stats = { hp = "$hp", hp_regen = "$hp_regen", armor = "$armor", magic_resist = "$magic_resist", move_speed = "$move_speed" },
        attack = { kind = "melee", range = "$attack_range", interval = "$attack_interval", attack_point = "$attack_point",
          damage = "$attack_min", damage_max = "$attack_max", damage_type = "physical" },
        bounty = { gold = "$bounty_gold" },
        vision = { day = "$vision_day", night = "$vision_night" },
        acquisition_range = "$acquisition", turn_rate = "$turn_rate", flying = true,
        passive = { reveal = "public", states = { "unselectable", "no_healthbar", "not_on_minimap", "no_unit_collision",
          "attack_immune", "magic_immune" } },
        ai = "guard", owner_death = "persist", tags = { "summon" },
        notes = "和本體共用的原作單位 C15T_dummy（npc_dota_creature，MOVE_FLY、近戰射程 10、DOTA_HULL_SIZE_HERO、ProjectileSpeed 10000），加 majia_vison。舊版另立模板 id，攻擊力由 c15t_old_arrow_power 給。存活由 c15t_old_arrow_power 的時間決定。",
        provenance = { "npc/npc_units_custom_load.txt:5187-5259", "npc/npc_abilities_custom_load.txt:5628-5651" } },
    },
  }
end
