-- A05_old 酒井忠次（舊版，11 版技能組；KV 註解標「11.2B」）— package schema v3。
-- 版本：原作 heromap_version A05 ["11"] = true，heromap_skill A05 ["11"] = "WERT"（vscripts/addon_game_mode.lua:145,247），
-- heromap_autoskill A05 ["11"] = ""（vscripts/addon_game_mode.lua:350）。
-- 槽位依據：vscripts/items/Addon_Items/tofar_goback.lua:105-133 先移除全部技能，再依 "WERT" 順序
-- AddAbility A05W_old、A05E_old、A05R_old、attribute_bonusx、A05T_old，所以 W/E/R/T → Q/W/E/R；沒有 D。
-- 舊版的忠之極意是大招 A05T_old（被動、可升三級），不是本體的 D。
-- 舊版不會加回 magical_resistance；attribute_bonusx、A05_precache、halloween_attack、buff_* 屬規則層，都不列入。
-- 原作沒有舊版招式的繁中說明（resource/addon_tchinese.txt 沒有 A05*_old），招式名取 KV 註解，說明依 KV 與腳本還原，見 assumptions.md。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"（同本體）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A05_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A05", text = { zh_TW = { name = "酒井忠次（舊版）" }, en = { name = "Sakai Tadatsugu (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a05_old_mikawa_valor", source_id = "A05W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 22, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 90, 105, 120, 135, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 15, unit = "s", source = "kv:modifier_A05W_old Duration" },
          bonus_attack = { 20, 40, 60, 80, unit = "dmg", source = "kv:bonus_attack" },
          steal = { 30, 35, 40, 45, unit = "pct", source = "kv:steal" },
        },
        text = {
          zh_TW = { name = "三河之勇",
            body = "{duration} 秒內攻擊力 +{bonus_attack}，普攻命中非建築目標時依造成的傷害吸血 {steal}（再扣一次目標護甲的減免）。效果可以重疊。" },
          en = { name = "Mikawa Valor",
            body = "For {duration} seconds he gains +{bonus_attack} attack damage, and attacks that land on non-building targets heal him for {steal} of the damage dealt (reduced once more by the target's armor). The effect stacks." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a05w_old_fury", duration = "$duration", stacking = "independent", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "三河之勇", body = "攻擊力 +{bonus_attack}，普攻命中非建築目標時吸血 {steal}。" },
              en = { name = "Mikawa Valor", body = "+{bonus_attack} attack damage; attacks that land on non-building targets heal for {steal} of the damage." },
            },
            reveal = "public", cue = "a05w_red_aura",
            stats = { attack_damage = "$bonus_attack" },
            triggers = {
              on_attack_hit = {
                filter = "not event.target.is_building and not has_status_tag(event.target, isvoid)",
                actions = {
                  -- %attack_damage：這次攻擊對目標造成的傷害（護甲後），推定為 attack.damage × 目標護甲倍率（同 A06、C20）；StealLife2 再乘一次（1 − 護甲減免）
                  { op = "heal", to = "self", source = "self",
                    amount = "event.attack.damage * armor_multiplier(event.target.armor) * $steal * armor_multiplier(event.target.armor)" },
                  { op = "cue", id = "a05w_old_lifesteal", at = "self" },
                } },
            },
            notes = "原作 modifier_A05W_old：IsBuff、Duration 15、MODIFIER_ATTRIBUTE_MULTIPLE、PREATTACK_BONUS_DAMAGE %bonus_attack、OnAttackLanded 跑 item_muramasa_katana.lua StealLife2；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a05w_vo", at = "caster" },
            { op = "apply_status", status = "a05w_old_fury", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A05.lua:30-56",
            text = "OnSpellStart 也呼叫與本體共用的 A05W_RunScript，以 ability:GetCastRange() 當半徑、GetAbilityDamage() 當傷害；A05W_old 的 KV 沒有 AbilityCastRange 也沒有 AbilityDamage，半徑與傷害都是 0，沒有效果。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_muramasa_katana.lua:109-137",
            text = "StealLife2 以 %attack_damage（已經過護甲）× steal% ×（1 − 目標護甲減免）回血，等於護甲算了兩次；負護甲時減免取負值。目標帶 isvoid（A32 虛化）或是建築時不吸血。回血經全域 HealFilter。" },
          { policy = "as_source", ref = "npc/A_Oda/A05.txt:476",
            text = "MODIFIER_ATTRIBUTE_MULTIPLE：冷卻縮減讓冷卻短於 15 秒時可重疊，攻擊力相加，每個實例各吸一次血。" },
          { policy = "note", ref = "npc/A_Oda/A05.txt:449-457",
            text = "ApplyModifier 的 Target 只寫 Center CASTER（沒有 Radius），只套在自己身上；pipe 音效屬呈現。" },
        },
        provenance = { "npc/A_Oda/A05.txt:420-508", "vscripts/heroes/A_Oda/A05.lua:30-56",
          "vscripts/items/Addon_Items/item_muramasa_katana.lua:109-137" },
      },
      -------------------------------------------------------------------------
      { id = "a05_old_warrior_way", source_id = "A05E_old", slot = "W", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          evasion = { 4, 8, 12, 16, unit = "pct", source = "kv:evasion" },
          crit_chance = { 20, unit = "pct", source = "kv:crit_chance", note = "RandomInt(1,100) <= 20，剛好 20%" },
          pity_after = { 4, unit = "count", source = "script:vscripts/heroes/A_Oda/A05.lua:206-209 A05Ecount >= 5 必中" },
          crit_bonus = { 200, 250, 300, 350, unit = "pct", source = "kv:crit_bonus" },
        },
        text = {
          zh_TW = { name = "武者之道",
            body = "被動：閃避 +{evasion}。普攻出手時有 {crit_chance} 機率暴擊，造成 {crit_bonus} 傷害；連續 {pity_after} 次沒有暴擊，下一次必定暴擊。" },
          en = { name = "Way of the Warrior",
            body = "Passive: +{evasion} evasion. Each attack has a {crit_chance} chance to critically strike for {crit_bonus} damage; after {pity_after} attacks in a row without one, the next attack always crits." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { evasion_pct = "$evasion" },
          triggers = {
            on_attack_windup = {
              { op = "chance", p = "$crit_chance", pity_after = "$pity_after", counter = "ability", actions = {
                { op = "modify_attack", crit_multiplier = "$crit_bonus" },
              } },
            },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "npc/A_Oda/A05.txt:534-559",
            text = "原作在 OnAttackStart 擲骰，成功就掛 modifier_A05E_old_critical_strike（PREATTACK_CRITICALSTRIKE），到下一次 OnAttackLanded 才移除：攻擊前搖被取消或落空時，暴擊會留到之後命中的那一擊。照設計意圖寫成只影響這次攻擊的 modify_attack。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A05.lua:202-212",
            text = "計數 caster.A05Ecount 每次出手加一，第五次（連續四次沒暴擊後）必中，暴擊後歸零；以 pity_after 表達。攻擊建築也會擲骰與計數。" },
          { policy = "note", ref = "npc/A_Oda/A05.txt:532",
            text = "EVASION_CONSTANT 是 Dota 閃避，多來源依規則層乘法疊加。" },
        },
        provenance = { "npc/A_Oda/A05.txt:510-579", "vscripts/heroes/A_Oda/A05.lua:202-212" },
      },
      -------------------------------------------------------------------------
      { id = "a05_old_mighty_blow", source_id = "A05R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          proc_chance = { 25, unit = "pct", source = "script:vscripts/heroes/A_Oda/A05.lua:174-179 RandomInt(1,100) <= 25" },
          pity_after = { 4, unit = "count", source = "script:vscripts/heroes/A_Oda/A05.lua:178-179 caster.A05R > 4 必中" },
          damage = { 100, 150, 200, 250, unit = "dmg", damage_type = "pure", source = "kv:dmg" },
          stun = { 0.35, unit = "s", source = "script:vscripts/heroes/A_Oda/A05.lua:194 modifier_stunned duration = 0.35" },
        },
        text = {
          zh_TW = { name = "怪力擊",
            body = "被動：普攻不會落空。普攻命中非建築目標時有 {proc_chance} 機率重擊，暈眩目標 {stun} 秒並造成 {damage} 純粹傷害（魔免目標只暈眩）；連續 {pity_after} 次沒有觸發，下一次必定觸發。" },
          en = { name = "Mighty Blow",
            body = "Passive: his attacks never miss. Attacks that land on non-building targets have a {proc_chance} chance to bash, stunning the target for {stun} seconds and dealing {damage} pure damage (spell-immune targets are only stunned); after {pity_after} hits in a row without one, the next always bashes." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = { filter = "not event.target.is_building", actions = {
              { op = "chance", p = "$proc_chance", pity_after = "$pity_after", counter = "ability", actions = {
                { op = "cue", id = "a05r_old_bash", at = "event.target" },
                { op = "apply_status", status = "a05r_old_stun", to = "event.target" },
                { op = "if", cond = "not event.target.is_magic_immune", actions = {
                  { op = "damage", to = "event.target", type = "pure", amount = "$damage" },
                } },
              } },
            } },
          },
        },
        statuses = {
          { id = "a05r_old_nomiss", duration = "infinite", stacking = "keep", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "cannot_miss" },
            notes = "原作 modifier_A05R_nomiss：Passive 1、IsHidden、MODIFIER_STATE_CANNOT_MISS；學會時套上的常駐狀態（被動 passive.states 產生器尚未支援，改用 on_learn）。" },
          { id = "a05r_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被怪力擊擊暈，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Stunned by Mighty Blow for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（ApplyDataDrivenModifier）；受狀態抗性縮短，不檢查魔免。" },
        },
        events = {
          on_learn = { { op = "apply_status", status = "a05r_old_nomiss", to = "caster" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A05.txt:612-620",
            text = "modifier_A05R_nomiss 是 Passive 1 的常駐 CANNOT_MISS：學會後所有普攻都不會落空，不只觸發重擊的那一下。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A05.lua:182-183",
            text = "觸發時另以 ApplyDataDrivenModifier 套 modifier_C03R_nomiss，但 A05R_old 沒有定義這個 modifier，沒有效果（常駐的必中已經涵蓋）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A05.lua:173-198",
            text = "計數 caster.A05R 只在命中非建築時加一，第五次（連續四次沒觸發後）必中，觸發後歸零；以 pity_after 表達。暈眩不檢查魔免與敵我（反補友軍也會暈），純粹傷害只對非魔免目標，經 AMHC（酒井死亡時不傷害）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A05.lua:187-193",
            text = "ITEM_D09 音效與 a07t2 特效屬呈現。" },
        },
        provenance = { "npc/A_Oda/A05.txt:581-622", "vscripts/heroes/A_Oda/A05.lua:168-200" },
      },
      -------------------------------------------------------------------------
      { id = "a05_old_loyal_secret", source_id = "A05T_old", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 190, 150, 110, unit = "s", source = "kv:cd", note = "AbilityCooldown 也是 190/150/110；A05.lua:235-236 StartCooldown" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          guard_time = { 0.5, unit = "s", source = "script:vscripts/heroes/A_Oda/A05.lua:234 modifier_A05T_old2 duration = 0.5" },
        },
        text = {
          zh_TW = { name = "忠之極意",
            body = "被動：受到致死的傷害時不會死亡：生命與法力全滿，移除所有敵方施加的狀態，並在 {guard_time} 秒內無敵且隱藏血條。分身不會觸發。" },
          en = { name = "Essence of Loyalty",
            body = "Passive: instead of dying from a lethal hit, his health and mana are fully restored, all statuses applied by enemies are removed, and he is invulnerable with no health bar for {guard_time} seconds. Illusions do not trigger it." },
        },
        target = { type = "none" },
        cost = { cooldown = "$cooldown", cooldown_start = "manual" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a05t_old_guard", duration = "$guard_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "忠之極意", body = "無敵且隱藏血條，持續 {guard_time} 秒。" },
              en = { name = "Essence of Loyalty", body = "Invulnerable with no health bar for {guard_time} seconds." },
            },
            reveal = "public", states = { "invulnerable", "no_healthbar" }, cue = "a05t_old_guard",
            notes = "原作 modifier_A05T_old2：IGNORE_INVULNERABLE、NO_HEALTH_BAR、INVULNERABLE；沒寫 IsPurgable（推定 basic）。" },
        },
        passive = {
          reveal = "public",
          triggers = {
            -- 同 FC-024（D13）：受到致死傷害時取消這一筆並觸發（原作在扣血後比較，擋不住致死一擊）
            on_take_damage_pre = {
              filter = "self.alive and event.damage.final >= self.hp and not self.is_illusion and ability.is_ready",
              actions = {
                { op = "modify_damage", cancel = true },
                { op = "set_health", to = "self", value = "self.max_hp" },
                { op = "set_mana", to = "self", value = "self.max_mana" },
                { op = "remove_status", from = "self", dispel = "all", team_of_source = "enemy" },
                { op = "apply_status", status = "a05t_old_guard", to = "self" },
                { op = "modify_cooldown", of = "self", ability = "a05_old_loyal_secret", mode = "start", amount = "$cooldown" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A05.lua:223",
            text = "同 FC-024（本體 A05D 的 D13 修正，套用到同一段舊版腳本）：受到致死傷害時在結算前取消這一筆並觸發。原作 A05T_old_OnTakeDamage 與 A05D_OnTakeDamage 相同，在 OnTakeDamage（已扣血）以 damage > GetHealth() 比較受傷後的剩餘生命：致死的一擊已經殺死英雄，擋不住死亡；反而是大於剩餘生命的非致死重擊（約超過受擊前生命一半）會觸發。是否沿用 FC-024 請工作線 D 確認。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A05.lua:226-233",
            text = "移除條件是 modifier 的施加者存在且隊伍不同；施加者已不存在的敵方狀態不會被移除。以 remove_status dispel = all、team_of_source = enemy 表達。" },
          { policy = "as_source", ref = "npc/A_Oda/A05.txt:662-674",
            text = "舊版保護狀態只有 0.5 秒無敵與隱藏血條，沒有本體的倒地暈眩與 ACT_DOTA_DIE 動作。" },
          { policy = "note", ref = "npc/A_Oda/A05.txt:639",
            text = "被動技能的 AbilityCooldown 只用於顯示，實際冷卻由腳本 StartCooldown(cd) 啟動；以 cooldown_start = manual 表達。" },
        },
        provenance = { "npc/A_Oda/A05.txt:624-686", "vscripts/heroes/A_Oda/A05.lua:215-240" },
      },
    },
  }
end
