-- A27 柴田勝家（舊版／11.2B，variant = "old"）— package schema v3。原作槽位 W/E/R/T → Q/W/E/R。
-- 版本與槽位依據：舊版是道具 tofar_goback 的「11 版」切換（vscripts/items/Addon_Items/tofar_goback.lua:100-139）：
-- 拔掉英雄所有技能後依 heromap_skill A27 ["11"] = "WERT"（vscripts/addon_game_mode.lua:269）逐一
-- AddAbility("A27" .. 字母 .. "_old")，所以舊版只有 A27W_old／A27E_old／A27R_old／A27T_old 四招，沒有 current 版的 A27D；
-- heromap_autoskill A27 ["11"] = ""（vscripts/addon_game_mode.lua:372），沒有自動學習。
-- 英雄共通的 magical_resistance（nobu 魔抗池）、attribute_bonusx、A27_precache、halloween_attack 屬規則層或資產，不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"、modifier_invulnerable 寫 "none"。
-- 原作 addon_tchinese 沒有舊版四招的技能說明，說明依 KV 與腳本還原，見同資料夾 assumptions.md。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A27_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A27", text = { zh_TW = { name = "柴田勝家（舊版）" }, en = { name = "Shibata Katsuie (Old)" } } },
    unit_state = {
      a27_old_noncrit = { type = "int", init = "0", reveal = "team", scope = "unit",
        note = "原作 caster.A27Rcount；存在單位欄位上，分身各自一份，暴擊時歸零" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a27_old_oni_shibata", source_id = "A27W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 70, 80, 90, 100, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 650, unit = "dist", source = "kv:radius", note = "同 AbilityCastRange／AoERadius" },
          duration = { 18, unit = "s", source = "kv:buff_duration" },
          attack = { 40, 60, 80, 100, unit = "dmg", source = "kv:buff_phy_damage" },
          armor = { 5, 10, 15, 20, unit = "armor", source = "kv:buff_phy_armor" },
        },
        text = {
          zh_TW = { name = "鬼柴田",
            body = "使自身與周圍 {radius} 內的友方英雄和一般單位獲得{@a27w_old_buff}，持續 {duration} 秒：攻擊力提高 {attack}、護甲提高 {armor}。" },
          en = { name = "Demon Shibata",
            body = "Grants Katsuie and allied heroes and units within {radius} {@a27w_old_buff} for {duration} seconds: +{attack} attack damage and +{armor} armor." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a27w_old_buff", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "a27w_old_aura", stats = { attack_damage = "$attack", armor = "$armor" },
            text = {
              zh_TW = { name = "鬼柴田", body = "攻擊力提高 {attack}、護甲提高 {armor}。" },
              en = { name = "Demon Shibata", body = "+{attack} attack damage and +{armor} armor." },
            },
            notes = "原作 modifier_A27T_old_buff（命名沿用 T，實際掛在 A27W_old）：IsBuff、AllowIllusionDuplicate 1、未寫 IsPurgable（暫定 basic）；PREATTACK_BONUS_DAMAGE 與 PHYSICAL_ARMOR_BONUS。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a27w_old_voice", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "ally",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
              actions = {
                { op = "apply_status", status = "a27w_old_buff", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A27.txt:507-518",
            text = "ApplyModifier 的 Target 區塊以施法者為中心、Teams FRIENDLY，包含勝家自己；Flags 的 MAGIC_IMMUNE_ENEMIES 只影響敵人，魔免友軍本來就選得到。" },
          { policy = "note", ref = "npc/A_Oda/A27.txt:519-528",
            text = "風暴之靈的音效與語音只屬呈現，改用 cue。原作 KV 註解把這招叫「鬼柴田」，modifier 卻命名為 modifier_A27T_old_buff。" },
        },
        provenance = { "npc/A_Oda/A27.txt:468-573" },
      },
      -------------------------------------------------------------------------
      { id = "a27_old_earth_hammer", source_id = "A27E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 6, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 450, unit = "dist", source = "kv:radius", note = "同 AbilityCastRange／AoERadius" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          stun = { 1, 1.4, 1.8, 2.2, unit = "s", source = "kv:stun_time" },
        },
        text = {
          zh_TW = { name = "動地鎚",
            body = "重鎚擊地，對周圍 {radius} 內的敵方英雄與一般單位造成 {damage} 魔法傷害並暈眩 {stun} 秒。" },
          en = { name = "Earthshaker Hammer",
            body = "Slams the ground, dealing {damage} magic damage to enemy heroes and units within {radius} and stunning them for {stun} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a27e_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" },
            text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } },
            notes = "datadriven Stun 動作套引擎內建 modifier_stunned；受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a27e_old_smash", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" } },
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$damage" },
                { op = "apply_status", status = "a27e_old_stun", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A27.txt:617-625",
            text = "ActOnTargets 沒寫 Flags：魔免敵人不會被選到；包含隱形與迷霧中的敵人。先傷害再暈眩。" },
          { policy = "note", ref = "npc/A_Oda/A27.txt:606-616",
            text = "item_soul_adder.lua sound(boulder_smash_target) 與語音只播音效，改用 cue。" },
        },
        provenance = { "npc/A_Oda/A27.txt:575-643" },
      },
      -------------------------------------------------------------------------
      { id = "a27_old_way_of_warrior", source_id = "A27R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          evasion = { 4, 8, 12, 16, unit = "pct", source = "kv:evasion" },
          crit_chance = { 20, unit = "pct",
            source = "script:vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:303-306 RandomInt(1,100) <= crit_chance（kv:crit_chance）" },
          pity_after = { 4, unit = "count",
            source = "script:vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:304-306 A27Rcount >= 5 時強制暴擊，連續失敗四次後第五次必中" },
          crit_multiplier = { 180, 220, 260, 300, unit = "pct", source = "kv:crit_bonus" },
        },
        text = {
          zh_TW = { name = "武者之道",
            body = "被動：閃避 {evasion}。普攻出手時有 {crit_chance} 機率暴擊，造成 {crit_multiplier} 傷害；連續 {pity_after} 次沒有暴擊，下一次必定暴擊。" },
          en = { name = "Way of the Warrior",
            body = "Passive: {evasion} evasion. Each attack has a {crit_chance} chance to critically strike for {crit_multiplier} damage; after {pity_after} attacks in a row without one, the next always crits." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { evasion_pct = "$evasion" },
          triggers = {
            on_attack_windup = { {
              op = "chance", p = "$crit_chance", pity_after = "$pity_after", counter_var = "a27_old_noncrit",
              actions = {
                { op = "modify_attack", crit_multiplier = "$crit_multiplier" },
              } } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:304-309",
            text = "每次出手先 +1 再判定 RandomInt(1,100) <= 20 或 count >= 5，暴擊時歸零：等於連續四次沒暴擊後第五次必中。計數存在單位欄位，跨死亡、跨升級保留，分身各自一份（counter_var 用 scope = unit 的 unit_state）。" },
          { policy = "design_intent", ref = "npc/A_Oda/A27.txt:678-693",
            text = "原作在 OnAttackStart 掛沒有時間的 modifier_A27R_old_critical_strike，OnAttackLanded 才移除；攻擊被取消時暴擊 modifier 會留到下一次攻擊命中，期間其他攻擊（PerformAttack 等）也會暴擊。照設計意圖只記在這一次攻擊（modify_attack）。" },
          { policy = "note", text = "閃避是 MODIFIER_PROPERTY_EVASION_CONSTANT，與其他閃避來源的疊加方式屬規則層。" },
        },
        provenance = { "npc/A_Oda/A27.txt:645-713", "vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:301-311" },
      },
      -------------------------------------------------------------------------
      { id = "a27_old_split_bamboo", source_id = "A27T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, 120, 110, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          invuln_time = { 1, unit = "s",
            source = "script:vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:329 modifier_invulnerable duration=1" },
          illusions_count = { 3, 4, 5, unit = "count", source = "kv:illusions_count" },
          illusion_duration = { 28, unit = "s", source = "kv:illusion_duration" },
          outgoing = { 0, unit = "pct", hidden = true, source = "kv:illusion_outgoing_damage" },
          incoming = { 0, unit = "pct", hidden = true, source = "kv:illusion_incoming_damage" },
          spawn_radius = { 141.42, 170.13, 200, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:338-342 model_radius(100)/sin(pi/(illusions_count+1))" },
          full_turn = { 360, unit = "deg", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:341-343 delta_theta = 2pi/total、start_theta = RandomFloat(0,2pi)" },
        },
        text = {
          zh_TW = { name = "破竹之勢",
            body = "停止動作並無敵 {invuln_time} 秒，製造 {illusions_count} 個分身，持續 {illusion_duration} 秒；本體與分身隨機排列在原地周圍的一圈上。分身繼承等級、技能、裝備、目前生命與法力，以及本體身上來自技能的狀態，造成與承受的傷害都和本體相同。" },
          en = { name = "Unstoppable Advance",
            body = "Stops, becomes invulnerable for {invuln_time} seconds and creates {illusions_count} illusions lasting {illusion_duration} seconds; Katsuie and the illusions are placed at random around a ring at his position. The illusions copy his level, abilities, items, current health and mana, and the ability-granted statuses on him, and deal and take damage exactly as he does." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a27t_old_invulnerable", duration = "$invuln_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "invulnerable" }, notes = "原作引擎 modifier_invulnerable" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a27t_old_voice", at = "caster" },
            { op = "apply_status", status = "a27t_old_invulnerable", to = "caster" },
            { op = "issue_order", who = "caster", order = "stop" },
            { op = "let", name = "center", value = "position(caster)" },
            { op = "let", name = "theta", value = "random(0, $full_turn)" },
            { op = "let", name = "real", value = "random_int(0, $illusions_count)" },
            { op = "repeat", count = "$illusions_count + 1", as = "i", actions = {
              { op = "let", name = "spot", value = "polar(center, theta + i * $full_turn / ($illusions_count + 1), $spawn_radius)" },
              { op = "if", cond = "i == real",
                actions = { { op = "teleport", who = "caster", to = "spot" } },
                otherwise = {
                  { op = "create_illusion", of = "caster", control = "owner", at = "spot", duration = "$illusion_duration",
                    copy = { level = true, abilities = true, items = "fresh", statuses = { "all" }, hp = "caster.hp", mana = "caster.mana",
                      unit_state = false, status_duration = "original", status_caster = "illusion",
                      status_tags = { "from_ability" } },
                    outgoing = { all = "$outgoing" }, incoming = { all = "$incoming" } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:338-360",
            text = "illusions_count+1 個位置等分一圈（半徑 model_radius/sin(半個間隔角)：三級分別約 141／170／200），起始角隨機，本體隨機佔其中一格（real_index），其餘各生一個分身；與 current 版 A27W 是同一個 MirrorImage 函式。" },
          { policy = "as_source", ref = "npc/A_Oda/A27.txt:748-762",
            text = "illusion_outgoing_damage 與 illusion_incoming_damage 都是 0：分身造成全額傷害、承受一般傷害，和本體一樣。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:428-435",
            text = "CreateMirror 對本體每個來自非道具技能的 modifier 呼叫 ApplyDataDrivenModifier(illusion, illusion, name, {duration = 總時長})：分身上重新計滿、施加者是分身（copy.status_duration = original、status_caster = illusion、status_tags = from_ability，同 current 版）。鬼柴田增益因此會複製到分身。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:346-360",
            text = "本體與分身依 create_delay 0.01 秒先後出現；0.01 秒間隔屬 Timer 精度，照設計意圖同時出現。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:437-441",
            text = "分身生命、法力等於本體目前值；magical_resistance 欄位照抄（規則層魔抗池）；illusion_damage 欄位 = (100+0)%。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:356",
            text = "本體以 FindClearSpaceForUnit 移動，分身以 CreateUnitByName(..., true) 找空地生成。" },
        },
        provenance = { "npc/A_Oda/A27.txt:715-781", "vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:325-442" },
      },
    },
  }
end
