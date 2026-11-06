-- B15 今川義元（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["B15"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:294），
-- 沒有自動學習的技能（heromap_autoskill["B15"]["11"] = ""，addon_game_mode.lua:397）；eventforpichero.lua:163-171 的 8 級 SetLevel
-- 只找本體 B15D（已被拔掉），所以沒有 D。舊版的多重箭是 T 槽大招 B15T_old。W/E/R/T → Q/W/E/R。
-- 原作舊版沒有任何語言的說明（resource/addon_*.txt 查無 B15*_old），名稱取 KV 註解（npc/B_Unified/B15.txt:107,331,537,642），
-- 說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、B15_precache 屬規則層或資產，不列入。
-- 慣例照本體 B15：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（v3.md §12.3 推定）；
-- 原作 HERO | BASIC 寫成 types = { "hero", "creep", "neutral", "summon" }；EXCLUDE_TARGET_NAME2 以單位 tag general 表達。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B15_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B15", text = { zh_TW = { name = "今川義元（舊版）" }, en = { name = "Imagawa Yoshimoto (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b15_old_giant_strength", source_id = "B15W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 50, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B15.txt:118" },
          mana_cost = { 175, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B15.txt:121" },
          duration = { 40, unit = "s", source = "kv:ApplyModifier Duration@npc/B_Unified/B15.txt:147" },
          attack_bonus = { 20, 30, 40, 50, unit = "dmg", source = "kv:attack_bonus@npc/B_Unified/B15.txt:187",
            note = "modifier_b15w BASEATTACK_BONUSDAMAGE（白字）" },
          str_bonus = { 10, 20, 30, 40, unit = "stat", source = "kv:power@npc/B_Unified/B15.txt:192",
            note = "modifier_b15w STATS_STRENGTH_BONUS" },
        },
        text = {
          zh_TW = { name = "巨人之臂力",
            body = "化身巨人 {duration} 秒：基礎攻擊力 +{attack_bonus}，力量 +{str_bonus}。" },
          en = { name = "Giant's Strength",
            body = "Becomes a giant for {duration} seconds: +{attack_bonus} base attack damage and +{str_bonus} strength." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b15w_old_giant",
            text = { zh_TW = { name = "巨人之臂力", body = "基礎攻擊力提高 {attack_bonus}，力量提高 {str_bonus}。" },
              en = { name = "Giant's Strength", body = "+{attack_bonus} base attack damage and +{str_bonus} strength." } },
            duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b15w_old_giant_aura",
            stats = { base_attack_damage = "$attack_bonus", str = "$str_bonus" },
            notes = "原作 modifier_b15w（與本體同名）：BASEATTACK_BONUSDAMAGE、STATS_STRENGTH_BONUS；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b15w_old_vo", at = "caster" },
            { op = "apply_status", status = "b15w_old_giant", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B15.txt:143-148",
            text = "持續時間寫死在 ApplyModifier Duration 40，不隨等級變化；AbilitySpecial 沒有 duration。" },
          { policy = "as_source", ref = "npc/B_Unified/B15.txt:130-149",
            text = "舊版 OnSpellStart 沒有 RunScript：不呼叫 B15W_on_spell_start，沒有模型放大、沒有淨化敵方狀態，modifier 也沒有魔法免疫（皆為本體 B15W 才有）。" },
          { policy = "note", ref = "npc/B_Unified/B15.txt:153-176",
            text = "modifier 與本體 B15W 同名 modifier_b15w；兩者不會同時存在於同一英雄，資料用獨立狀態 id b15w_old_giant。粒子與音效屬呈現，改用 cue。" },
          { policy = "note", ref = "npc/B_Unified/B15.txt:114",
            text = "KV 沒寫 AbilityCastPoint（IMMEDIATE）。" },
        },
        provenance = { "npc/B_Unified/B15.txt:107-195" },
      },
      -------------------------------------------------------------------------
      { id = "b15_old_flame_arrow", source_id = "B15E_old", slot = "W", kind = "toggle", max_level = 4, tags = { "basic", "toggle", "orb" },
        values = {
          mana_per_attack = { 10, 12, 14, 16, unit = "mana", source = "kv:mana_cost@npc/B_Unified/B15.txt:425",
            note = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:87-101 SpendMana" },
          damage = { 40, 80, 120, 160, unit = "dmg", damage_type = "magical", source = "kv:damage_bonus@npc/B_Unified/B15.txt:420" },
          building_pct = { 20, unit = "pct", source = "script:vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:66-67 dmg*0.2" },
        },
        text = {
          zh_TW = { name = "烈焰箭",
            body = "開啟時，每次普攻開始都消耗 {mana_per_attack} 法力；法力不足時自動關閉並耗盡剩餘法力。普攻命中時額外造成 {damage} 魔法傷害，對建築只造成 {building_pct}。{@b15_old_multishot}射出的每支箭也會額外造成 {damage} 魔法傷害。" },
          en = { name = "Flame Arrows",
            body = "While active, each attack Yoshimoto starts costs {mana_per_attack} mana; if he lacks the mana, the ability turns off and his remaining mana is drained. Attacks that land deal {damage} bonus magic damage ({building_pct} against buildings). Each arrow fired by {@b15_old_multishot} also deals {damage} bonus magic damage." },
        },
        target = { type = "none" },
        passive = { orb_slot = { release = "status_end", claim = true, overridable = true } },
        statuses = {
          { id = "b15e_old_orb",
            text = { zh_TW = { name = "烈焰箭", body = "每次普攻開始消耗 {mana_per_attack} 法力；普攻命中時額外造成 {damage} 魔法傷害（對建築為 {building_pct}）。" },
              en = { name = "Flame Arrows", body = "Each attack costs {mana_per_attack} mana; attacks that land deal {damage} bonus magic damage ({building_pct} against buildings)." } },
            duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b15e_old_searing_arrow_projectile", remove_on = { "death" },
            triggers = {
              -- FC-104（D13，照本體）：死亡時關閉開關（釋放法球槽），復活後不會開著卻沒有效果
              on_death = { { op = "toggle_ability", of = "self", ability = "b15_old_flame_arrow", on = false } },
              on_attack_windup = {
                { op = "if", cond = "self.mana < $mana_per_attack", actions = {
                  { op = "toggle_ability", of = "self", ability = "b15_old_flame_arrow", on = false },
                } },
                { op = "spend_mana", from = "self", amount = "min(self.mana, $mana_per_attack)" },
              },
              on_attack_hit = {
                filter = "orb_slot_free(self, ability)",
                actions = {
                  { op = "damage", to = "event.target", type = "magical",
                    amount = "select(event.target.is_building, $damage * $building_pct, $damage)" },
                  { op = "cue", id = "b15e_old_split_sound", at = "event.target" },
                } },
            },
            notes = "原作 modifier_searing_arrow（與本體同名）：Orb 只換投射物外觀（CastAttack 0）；OnAttackStart 扣魔（B15E）、OnAttackLanded 加傷（B15E_attack3）；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_toggle_on = { { op = "apply_status", status = "b15e_old_orb", to = "caster" } },
          on_toggle_off = { { op = "remove_status", from = "caster", status = "b15e_old_orb" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:27-37,64",
            text = "法球槽 nobuorb1：開啟時無條件寫成 B15E_1（覆寫道具佔用），之後法球道具觸發時又會覆寫，這時烈焰箭照扣魔但不加傷；關閉時只有槽仍是自己才清空。照本體寫成 orb_slot = { claim, overridable, release = status_end }，命中加傷以 orb_slot_free 判斷。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:87-101",
            text = "扣魔在 OnAttackStart（前搖開始）：攻擊被取消仍扣魔，打建築也扣；扣魔不看法球槽。法力不足時先 CastAbilityToggle 關閉，再 SpendMana 把剩餘法力扣到零，這一擊沒有加傷（狀態已移除）。" },
          { policy = "fixed", ref = "npc/B_Unified/B15.txt:379-381",
            text = "FC-104（D13 修正，照本體 B15E）：死亡時關閉開關並釋放法球槽。原作 modifier_searing_arrow 不是 Passive，死亡時被移除，但開關仍亮、nobuorb1 仍是 B15E_1（推定）：復活後不扣魔也不加傷，多重箭的火傷仍會觸發，必須手動關再開。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:57-72",
            text = "舊版命中走 B15E_attack3：以 FindAbilityByName(\"B15E_old\") 讀等級，建築只吃 20%（本體 B15E_attack1 是 25%）；加傷走 AMHC:Damage（義元已死不傷害）。" },
          { policy = "as_source", ref = "npc/B_Unified/B15.txt:411-427",
            text = "原作 KV 沒有 AbilityManaCost 與 AbilityCooldown；開關本身不耗魔，每次攻擊扣 mana_cost。" },
        },
        provenance = { "npc/B_Unified/B15.txt:331-428", "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:27-37",
          "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:57-72", "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:87-101" },
      },
      -------------------------------------------------------------------------
      { id = "b15_old_giant_hide", source_id = "B15R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          block = { 40, 80, 120, 160, unit = "dmg", source = "kv:Damage_Income@npc/B_Unified/B15.txt:560", note = "PHYSICAL_CONSTANT_BLOCK" },
        },
        text = {
          zh_TW = { name = "巨人之外皮",
            body = "被動：每次受到的物理傷害減少 {block}。" },
          en = { name = "Giant's Hide",
            body = "Passive: each instance of physical damage taken is reduced by {block}." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { physical_block = "$block" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B15.txt:543-554",
            text = "block_B15R_old：MODIFIER_PROPERTY_PHYSICAL_CONSTANT_BLOCK，每次物理傷害固定格擋、100% 機率；舊版沒有本體的力量加成。是否也格擋物理技能傷害依引擎實作，交給規則層的 physical_block（同本體）。" },
        },
        provenance = { "npc/B_Unified/B15.txt:537-563" },
      },
      -------------------------------------------------------------------------
      { id = "b15_old_multishot", source_id = "B15T_old", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades@npc/B_Unified/B15.txt:657-658" },
          arrow_count = { 2, 4, 5, unit = "count", source = "kv:arrow_count@npc/B_Unified/B15.txt:706" },
          split_range = { 800, unit = "dist", source = "kv:range@npc/B_Unified/B15.txt:711" },
          projectile_speed = { 1800, unit = "speed", hidden = true, source = "kv:projectile_speed@npc/B_Unified/B15.txt:716" },
          arrow_damage = { 90, unit = "pct",
            source = "script:vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:198 GetAverageTrueAttackDamage(target) * 0.9" },
          illusion_arrow_damage = { 27, unit = "pct", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:195-196 幻象 * 0.27" },
          general_pct = { 50, unit = "pct",
            source = "script:vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:200-202 EXCLUDE_TARGET_NAME2 傷害 * 0.5" },
        },
        text = {
          zh_TW = { name = "多重箭",
            body = "被動：普攻出手時，對周圍 {split_range} 內最近的另外最多 {arrow_count} 名可見敵方英雄與部隊（含魔免）各射出一支箭，造成自身平均攻擊力 {arrow_damage} 的物理傷害（對武將減為 {general_pct}）。{@b15_old_flame_arrow}開啟時，每支箭另外帶火。" },
          en = { name = "Multishot",
            body = "Passive: when attacking, fires an extra arrow at each of up to {arrow_count} other nearest visible enemy heroes and units within {split_range} (spell-immune ones included), dealing {arrow_damage} of Yoshimoto's average attack damage as physical damage ({general_pct} against generals). While {@b15_old_flame_arrow} is on, each arrow also carries its fire." },
        },
        target = { type = "none" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_launch = { actions = {
              { op = "for_each",
                select = { from = "self", shape = "circle", radius = "$split_range", team = "enemy",
                  types = BASIC, flags = { "include_magic_immune", "visible_only" },
                  exclude = "event.target", where = "not has_status_tag(it, engine_invisible)",
                  order = "nearest", limit = "$arrow_count" },
                actions = {
                  { op = "projectile", kind = "tracking", from = "self", to = "it", speed = "$projectile_speed",
                    dodgeable = true, cue = "b15t_old_split_arrow",
                    on_hit = {
                      { op = "cue", id = "b15t_old_split_hit", at = "it" },
                      { op = "if", cond = "has_status(caster, b15e_old_orb)", actions = {
                        { op = "damage", to = "it", type = "magical", amount = "$b15_old_flame_arrow.damage" },
                      } },
                      { op = "damage", to = "it", type = "physical", require_source_alive = false, flags = { "no_amplify" },
                        amount = "avg_attack_against(caster, it) * select(caster.is_illusion, $illusion_arrow_damage, $arrow_damage) * select(has_tag(it, general), $general_pct, 1)" },
                    } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B15.txt:642-719",
            text = "舊版的多重箭是 T 槽大招（ULTIMATE、需要 6/12/18 級），箭數隨等級 2/4/5；本體是 8 級自動學會的 D 槽、固定 2 支。腳本與本體共用 B15D_SplitShotLaunch／B15D_SplitShotDamage。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:133-182",
            text = "OnAttack（出手）時以自己為圓心 FIND_CLOSEST 搜尋 KV 的 HERO|BASIC＋MAGIC_IMMUNE_ENEMIES，跳過目前攻擊目標、己方看不見的與帶 modifier_invisible 的單位，射滿 arrow_count 支為止；攻擊建築時也會分箭。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:157-160,203-207",
            text = "舊版會找到 B15E_old：烈焰箭有等級且 nobuorb1 == B15E_1 時，箭改用火箭粒子（呈現），命中時另以 AMHC:Damage 造成 damage_bonus 魔法傷害（取烈焰箭等級；義元已死不傷害；箭不打建築所以沒有建築折減）。條件以「身上有 b15e_old_orb」近似（產生器不支援對其他技能呼叫 orb_slot_free）：法球道具覆寫 nobuorb1 期間原作不加火傷，v3 仍會加，見 assumptions.md。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:185-211",
            text = "物理箭傷走 ApplyDamage（不經 AMHC），義元死亡後已飛出的箭仍造成傷害；不觸發攻擊特效。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:195-196",
            text = "舊版沒有虛張聲勢，義元本身不產生幻象；幻象倍率 0.27 只在其他來源（道具）複製出的幻象上生效，照腳本保留。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:152-156",
            text = "在 pairs 迴圈中 table.remove 掉帶 majia 技能的單位；majia 是馬甲 dummy，selector 預設排除，不影響結算。" },
        },
        provenance = { "npc/B_Unified/B15.txt:642-719", "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:130-211",
          "vscripts/library/common/dummy.lua:327-330" },
      },
    },
  }
end
