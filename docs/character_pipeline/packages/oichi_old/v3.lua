-- C17 阿市（舊版，11 版 *_old，KV 註記「阿市 11.2B」）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C17"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:333）；
-- heromap_autoskill["C17"]["11"] = ""（addon_game_mode.lua:436），沒有 D（本體的 C17D 是 16 版才有）。
-- KV 在 npc/C_Neutral/C17.txt:477-831（C17W_old／C17E_old／C17R_old／C17T_old）。W/E/R/T → Q/W/E/R。
-- 繁中說明在 panorama/localization/addon_tchinese.txt:990-998（UTF-16），數值以 KV 與腳本為準。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C17_precache 屬規則層或資產，不列入。
-- 慣例照本體 C17：HERO | BASIC 寫成 BASIC 清單；ApplyDamage 寫 require_source_alive = false；SetHealth／SetMana 直接改數值，不經治療事件。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  local ALLY_FLAGS = { "include_magic_immune", "include_invulnerable" }
  return {
    schema_version = 3, kind = "hero", id = "C17_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C17", text = { zh_TW = { name = "阿市（舊版）" }, en = { name = "Oichi (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c17_old_light_essence", source_id = "C17W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 2, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C17.txt:493" },
          mana_cost = { 70, 90, 110, 130, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C17.txt:494" },
          radius = { 800, unit = "dist", source = "kv:ActOnTargets Radius@npc/C_Neutral/C17.txt:519（AOERadius@C17.txt:491 同值）" },
          heal = { 75, 150, 225, 300, unit = "heal", source = "kv:health_recover@npc/C_Neutral/C17.txt:546" },
        },
        text = {
          zh_TW = { name = "光之精華",
            body = "無視負面狀態，直接回復周圍 {radius} 內友方英雄與部隊（含自己與無敵單位）{heal} 生命，不受治療增減影響。" },
          en = { name = "Essence of Light",
            body = "Directly restores {heal} health to allied heroes and units within {radius} (Oichi and invulnerable units included), ignoring negative effects and unaffected by healing increases or reductions." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "c17w_old_cast_vo", at = "caster" },
            { op = "cue", id = "c17w_old_light", at = "caster", radius = "$radius" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "ally", types = BASIC, flags = ALLY_FLAGS },
              actions = {
                { op = "set_health", to = "it", value = "min(it.hp + $heal, it.max_hp)" },
                { op = "cue", id = "c17w_old_heal_fx", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C17.lua:328-333",
            text = "C17W_old_health_recover 以 SetHealth(目前生命 + health_recover) 回復：不經治療事件，治療降低或禁止治療都擋不住（說明的「無視負面狀態」）；Dota 的 SetHealth 截在最大生命。" },
          { policy = "as_source", ref = "npc/C_Neutral/C17.txt:514-523",
            text = "ActOnTargets 以阿市為圓心、友方 HERO | BASIC、INVULNERABLE 旗標：含阿市自己與無敵單位，不含建築。冷卻只有 2 秒。" },
        },
        provenance = { "npc/C_Neutral/C17.txt:479-549", "vscripts/heroes/C_Neutral/C17.lua:328-333", "panorama/localization/addon_tchinese.txt:990-991" },
      },
      -------------------------------------------------------------------------
      { id = "c17_old_soul_essence", source_id = "C17E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 2, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C17.txt:565" },
          radius = { 800, unit = "dist", source = "kv:ActOnTargets Radius@npc/C_Neutral/C17.txt:591（AOERadius@C17.txt:563 同值）" },
          mana_restore = { 50, 100, 150, 200, unit = "mana", source = "kv:mana_recover@npc/C_Neutral/C17.txt:618" },
        },
        text = {
          zh_TW = { name = "魂之精華",
            body = "直接回復周圍 {radius} 內友方英雄與部隊（含自己與無敵單位）{mana_restore} 法力。不消耗法力。" },
          en = { name = "Essence of Soul",
            body = "Directly restores {mana_restore} mana to allied heroes and units within {radius} (Oichi and invulnerable units included). Costs no mana." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "c17e_old_cast_vo", at = "caster" },
            { op = "cue", id = "c17e_old_soul", at = "caster", radius = "$radius" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "ally", types = BASIC, flags = ALLY_FLAGS },
              actions = {
                { op = "set_mana", to = "it", value = "min(it.mana + $mana_restore, it.max_mana)" },
                { op = "cue", id = "c17e_old_mana_fx", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C17.lua:335-340",
            text = "C17E_old_mana_recover 以 SetMana(目前法力 + mana_recover) 回復，截在最大法力；AbilityManaCost 0、冷卻 2 秒，阿市自己也回魔，可以連續施放。" },
        },
        provenance = { "npc/C_Neutral/C17.txt:551-621", "vscripts/heroes/C_Neutral/C17.lua:335-340", "panorama/localization/addon_tchinese.txt:992-993" },
      },
      -------------------------------------------------------------------------
      { id = "c17_old_stasis_trap", source_id = "C17R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic", "summon" },
        values = {
          cooldown = { 7, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C17.txt:635" },
          mana_cost = { 150, 120, 90, 60, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C17.txt:636" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C17.txt:638" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C17.txt:634" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C17.txt:639" },
          stun = { 2.5, unit = "s", source = "kv:duration_stun@npc/C_Neutral/C17.txt:730" },
          trigger_radius = { 275, unit = "dist", source = "kv:radius_trigger@npc/C_Neutral/C17.txt:735" },
          lifetime = { 200, unit = "s", source = "kv:max_life_time@npc/C_Neutral/C17.txt:740" },
          stun_radius = { 500, unit = "dist", source = "kv:radius_stun@npc/C_Neutral/C17.txt:745" },
          stun_delay = { 0.4, unit = "s", hidden = true, source = "kv:delay_stun@npc/C_Neutral/C17.txt:750" },
          arm_delay = { 1.5, unit = "s", source = "kv:modifier_C17R_old_delay_enable Duration@npc/C_Neutral/C17.txt:670" },
          trap_hp = { 200, unit = "hp", source = "script:vscripts/heroes/C_Neutral/C17.lua:358 SetHealth(200)" },
        },
        text = {
          zh_TW = { name = "靜滯陷阱",
            body = "在指定地點放置一個{@c17_old_trap}（生命 {trap_hp}），{arm_delay} 秒後隱形並啟動，存在 {lifetime} 秒。啟動後敵方英雄或部隊（含魔免）靠近到 {trigger_radius} 內時陷阱碎裂，稍後對周圍 {stun_radius} 內非魔免的敵方英雄與部隊造成 {damage} 魔法傷害並擊暈 {stun} 秒，同時摧毀範圍內的樹木。陷阱魔法免疫，被擊殺時不會引爆。" },
          en = { name = "Stasis Trap",
            body = "Places a {@c17_old_trap} ({trap_hp} health) at the target point; after {arm_delay} seconds it turns invisible and arms, lasting {lifetime} seconds. Once armed, when an enemy hero or unit (spell-immune ones included) comes within {trigger_radius}, the trap shatters and shortly after deals {damage} magic damage to non-spell-immune enemy heroes and units within {stun_radius} and stuns them for {stun} seconds, also destroying trees in the area. The trap is spell immune and does not detonate if killed." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$stun_radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c17r_old_trap_base", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "magic_immune", "no_unit_collision" },
            notes = "for_magic_immune、for_no_collision 兩個常駐技能（npc/npc_abilities_custom_load.txt:5073-5090,5247-5265）。" },
          { id = "c17r_old_arming", text = { zh_TW = { name = "靜滯陷阱（準備中）", body = "即將隱形並啟動。" }, en = { name = "Stasis Trap (Arming)", body = "About to turn invisible and arm." } },
            duration = "$arm_delay", stacking = "refresh", dispel = "none", visibility = "buff", reveal = "public",
            triggers = {
              on_removed = {
                { op = "if", cond = "alive(self)", actions = {
                  { op = "apply_status", status = "c17r_old_detector", to = "self", source = "self" },
                } },
              },
            },
            notes = "原作 modifier_C17R_old_delay_enable：Duration 1.5，OnDestroy C17R_old_OnEnable 掛光環、modifier_invisible 與 modifier_kill(max_life_time)。" },
          { id = "c17r_old_detector", text = { zh_TW = { name = "靜滯陷阱", body = "隱形的陷阱；敵人進入 {trigger_radius} 內就會引爆。" }, en = { name = "Stasis Trap", body = "An invisible trap; detonates when an enemy comes within {trigger_radius}." } },
            duration = "$lifetime", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", states = { "invisible" }, tags = { "invisible" }, cue = "c17r_old_trigger_ring",
            aura = { radius = "$trigger_radius", team = "enemy", types = BASIC, flags = { "include_magic_immune" },
              status = "c17r_old_tripped" },
            triggers = {
              on_removed = {
                { op = "if", cond = "alive(self)", actions = { { op = "kill", to = "self", silent = true } } },
              },
            },
            notes = "原作 modifier_C17R_old_aura（施加者是陷阱自己）＋ modifier_invisible ＋ modifier_kill。光環 modifier 另有 DISABLE_HEALING（陷阱不能被治療，產生器不支援 cannot_be_healed，未寫）與 NO_UNIT_COLLISION。" },
          { id = "c17r_old_tripped", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_created = { { op = "call", routine = "c17r_old_explode", args = { trap = "caster" } } },
            },
            notes = "原作 modifier_C17R_old_trigger：光環給的狀態（MULTIPLE），OnCreated 呼叫 C17R_old_on_trigger（keys.caster 是陷阱）。" },
          { id = "c17r_old_stun", runtime_id = "stun", text = { zh_TW = { name = "暈眩", body = "無法移動、攻擊或施法。" }, en = { name = "Stunned", body = "Cannot move, attack or cast." } },
            duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" },
            notes = "原作 AddNewModifier(caster, ability, \"modifier_stunned\", {duration})：引擎暈眩，受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c17r_old_cast_vo", at = "caster" },
            { op = "create_unit", template = "c17_old_trap", at = "point", owner = "caster", team = "caster", control = "none", as = "trap" },
            { op = "set_health", to = "trap", value = "$trap_hp" },
            { op = "apply_status", status = "c17r_old_trap_base", to = "trap" },
            { op = "apply_status", status = "c17r_old_arming", to = "trap", source = "trap" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C17.lua:342-365",
            text = "陷阱單位 StatusHealth 10000，腳本 SetHealth(200)：目前生命 200、最大生命仍是 10000，生命回復 0；SetHullRadius 175 但有 for_no_collision。SetMana(AbilityDamage) 存的法力之後沒有被讀取，不列入。阿市換 school 造型時改用 C17R_old_SUMMEND_UNIT_bag_hero（只換模型），屬呈現。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C17.lua:378-450",
            text = "觸發時移除光環與隱形並 ForceKill 陷阱，0.4 秒後以陷阱位置為圓心 500：摧毀樹木，對技能目標隊伍／類型／旗標（敵方 HERO | BASIC、NONE，魔免排除）造成 ApplyDamage 魔法 AbilityDamage 並 modifier_stunned 2.5 秒；攻擊者是阿市，不檢查存活。觸發用光環含魔免（MAGIC_IMMUNE_ENEMIES），所以魔免敵人會觸發但不受傷害。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C17.lua:424-441",
            text = "unit.c17r_old 記錄單位一秒內被哪個陷阱炸過：被另一個陷阱炸到時延後 0.1 秒結算（仍然照常暈眩與傷害）。資料同一刻結算，差 0.1 秒的時序。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C17.lua:383-388",
            text = "多名敵人同一刻進入觸發範圍時，原作光環可能在陷阱被 ForceKill 前對每人各觸發一次；資料推定只爆一次（同百地三太夫隱雷），需實測。陷阱被擊殺時給 25 金且不引爆。" },
          { policy = "as_source", ref = "npc/C_Neutral/C17.txt:636",
            text = "耗魔隨等級降低（150/120/90/60），冷卻 7 秒，陷阱沒有數量上限。原作說明寫敵人靠近 275、暈眩 425 範圍；腳本實際用 radius_stun 500。" },
        },
        provenance = { "npc/C_Neutral/C17.txt:623-753", "vscripts/heroes/C_Neutral/C17.lua:342-450",
          "npc/npc_units_custom_load.txt:4956-5072", "npc/npc_abilities_custom_load.txt:5073-5090,5247-5265",
          "panorama/localization/addon_tchinese.txt:994-995,998" },
      },
      -------------------------------------------------------------------------
      { id = "c17_old_aurora_guard", source_id = "C17T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, 120, 110, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C17.txt:771" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C17.txt:772" },
          cast_range = { 1050, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C17.txt:773" },
          cast_point = { 1.5, 1, 0.5, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C17.txt:769",
            note = "KV delay 1.5/1.0/0.5（C17.txt:828）同值但腳本不讀；說明的「延遲」就是前搖" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel@npc/C_Neutral/C17.txt:761 + LevelsBetweenUpgrades@C17.txt:762" },
          radius = { 800, 900, 1000, unit = "dist", source = "kv:raduis@npc/C_Neutral/C17.txt:823" },
          damage = { 900, 1300, 1800, unit = "dmg", damage_type = "magical", source = "kv:damage@npc/C_Neutral/C17.txt:818" },
        },
        text = {
          zh_TW = { name = "極光守護",
            body = "延遲施法後在指定地點 {radius} 範圍釋放極光：非魔免的敵方英雄與部隊受到 {damage} 魔法傷害；範圍內友方英雄與部隊（含自己與無敵單位）無視負面狀態，生命直接回滿。" },
          en = { name = "Aurora Guard",
            body = "After a delayed cast, releases an aurora within {radius} of the target point: non-spell-immune enemy heroes and units take {damage} magic damage, while allied heroes and units in the area (Oichi and invulnerable units included) are restored to full health, ignoring negative effects." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        events = {
          on_phase_start = { { op = "cue", id = "c17t_old_cast_vo", at = "caster" } },
          on_cast = {
            { op = "cue", id = "c17t_old_aurora", at = "point", radius = "$radius" },
            { op = "for_each",
              select = { from = "point", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                flags = { "include_invulnerable" } },
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$damage" },
              } },
            { op = "for_each",
              select = { from = "point", shape = "circle", radius = "$radius", team = "ally", types = BASIC, flags = ALLY_FLAGS },
              actions = {
                { op = "set_health", to = "it", value = "it.max_hp" },
                { op = "cue", id = "c17t_old_health_fx", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C17.lua:452-471",
            text = "RunScript Target 區塊以施放點為圓心、TEAM_BOTH、HERO | BASIC、INVULNERABLE 旗標：敵人 AMHC 魔法傷害（阿市已死不傷害，規則層），友方 SetHealth(最大生命) 直接回滿，不經治療事件。魔免敵人選不到，魔免友方照常回滿。" },
          { policy = "as_source", ref = "npc/C_Neutral/C17.txt:764-774",
            text = "和本體不同：沒有範圍提示、沒有最大生命比例；冷卻 130～110、傷害 900～1800、範圍 800～1000、前搖 1.5/1/0.5。KV delay 與前搖同值但沒有被讀，不另列。" },
        },
        provenance = { "npc/C_Neutral/C17.txt:755-831", "vscripts/heroes/C_Neutral/C17.lua:452-471",
          "vscripts/amhc_library/amhc.lua:535-584", "panorama/localization/addon_tchinese.txt:996-997" },
      },
    },
    routines = {
      c17r_old_explode = {
        params = { trap = "unit" },
        actions = {
          { op = "let", name = "center", value = "position(trap)" },
          { op = "kill", to = "trap", silent = true },
          { op = "delay", time = "$c17_old_stasis_trap.stun_delay", capture = { "center" }, actions = {
            { op = "destroy_trees", at = "center", radius = "$c17_old_stasis_trap.stun_radius" },
            { op = "for_each",
              select = { from = "center", shape = "circle", radius = "$c17_old_stasis_trap.stun_radius", team = "enemy", types = BASIC },
              actions = {
                { op = "apply_status", status = "c17r_old_stun", to = "it", source = "caster_hero" },
                { op = "damage", to = "it", type = "magical", amount = "$c17_old_stasis_trap.damage", require_source_alive = false,
                  source = "caster_hero" },
              } },
            { op = "cue", id = "c17r_old_boom", at = "center", radius = "$c17_old_stasis_trap.stun_radius" },
          } },
        },
      },
    },
    unit_templates = {
      { id = "c17_old_trap", source_id = "C17R_old_SUMMEND_UNIT_hero",
        values = {
          hp = { 10000, unit = "hp", hidden = true, source = "kv:StatusHealth@npc/npc_units_custom_load.txt:4956-5033",
            note = "最大生命；施放時目前生命設為 c17_old_stasis_trap.trap_hp" },
          armor = { 0, unit = "armor", source = "kv:ArmorPhysical@npc/npc_units_custom_load.txt:4956-5033" },
          vision = { 200, unit = "dist", source = "kv:VisionDaytimeRange／VisionNighttimeRange@npc/npc_units_custom_load.txt:4956-5033" },
          bounty_gold = { 25, unit = "gold", source = "kv:BountyGoldMin@npc/npc_units_custom_load.txt:4956-5033" },
        },
        text = { zh_TW = { name = "靜滯陷阱" }, en = { name = "Stasis Trap" } },
        stats = { hp = "$hp", armor = "$armor", move_speed = "0" },
        bounty = { gold = "$bounty_gold" },
        vision = { day = "$vision", night = "$vision" },
        ai = "none", tags = { "summon" },
        notes = "npc_dota_creature，MOVE_NONE、攻擊 0、AttackAcquisitionRange 0（不會主動攻擊），AttackRange 275 只用來顯示；模型 jugg_healing_ward。",
        provenance = { "npc/npc_units_custom_load.txt:4956-5033" } },
    },
  }
end
