-- B15 今川義元（2.0 版，原作 *_20 技能組）— package schema v3。每個版本是一位獨立英雄（D22）；舊版另成 imagawa_yoshimoto_old。
-- 槽位依據：選 2.0 版時 choose_20 拔掉全部技能，依 heromap_skill B15 ["20"] = "WERDT" 逐一 AddAbility("B15" .. 字母 .. "_20")，
--   第五個字母前插入 attribute_bonusx；heromap_autoskill B15 ["20"] = "D" 自動學 B15D_20
--   （vscripts/items/Addon_Items/tofar_goback.lua:6-48、vscripts/addon_game_mode.lua:294,397）。
-- W/E/R/T → Q/W/E/R；B15D_20（PASSIVE | NOT_LEARNABLE、MaxLevel 1）照慣例放 D，寫在 hero.auto_learn。
-- KV 在 npc/B_Unified/B15.txt:721-1083；腳本 vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:15-24,74-101,326-430。
-- 原作沒有 *_20 招式的說明文字（resource/addon_*.txt 查無），名稱取自 KV 註解，說明依 KV 與腳本還原（assumptions.md）。
-- 英雄單位數值沿用本體 B15。magical_resistance、attribute_bonusx、B15_precache 屬規則層或資產，不列入。
-- 慣例照本體 B15：KV 未寫 IsPurgable 的 datadriven modifier 暫定 dispel = "basic"；原作 HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "B15_v20",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "v20" },
    hero = { source_id = "B15", text = { zh_TW = { name = "今川義元（2.0）" }, en = { name = "Imagawa Yoshimoto (2.0)" } },
      auto_learn = { "b15_v20_hundred_paces" } },
    unit_state = {
      b15_v20_illusion_count = { type = "int", init = "0", reveal = "team", scope = "hero",
        note = "原作 B15T_20 的 ability.count：目前存活的東海道一弓取幻象數。" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b15_v20_hundred_paces", source_id = "B15D_20", slot = "D", kind = "passive", max_level = 1, tags = { "basic" },
        learnable = false,
        values = {
          stun_chance = { 10, unit = "pct", source = "script:vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:426 RandomInt(1,100) <= 10" },
          stun = { 0.1, unit = "s", source = "script:vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:427 modifier_stunned duration 0.1" },
        },
        text = {
          zh_TW = { name = "百步穿楊",
            body = "被動：普攻命中非建築目標時，有 {stun_chance} 機率使其暈眩 {stun} 秒。開場自動學會，無法升級。" },
          en = { name = "Hundred-Pace Marksman",
            body = "Passive: attacks that land on a non-building have a {stun_chance} chance to stun the target for {stun} seconds. Learned automatically at the start; cannot be leveled." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = {
              filter = "not event.target.is_building",
              actions = {
                { op = "chance", p = "$stun_chance", actions = {
                  { op = "apply_status", status = "b15d_20_stun", to = "event.target" },
                } },
              } },
          },
        },
        statuses = {
          { id = "b15d_20_stun", text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } },
            duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned，以 ApplyDataDrivenModifier duration 0.1 套上。" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:421-430",
            text = "OnAttackLanded 對非建築擲 RandomInt(1,100) <= 10，恰好 10%；暈眩 0.1 秒（可打斷引導）。KV 的 DAMAGE_TYPE_PURE 沒有用到，沒有傷害。" },
          { policy = "note", text = "東海道一弓取的幻象也複製這個被動，幻象的普攻同樣會觸發。" },
        },
        provenance = { "npc/B_Unified/B15.txt:721-746", "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:421-430",
          "vscripts/items/Addon_Items/tofar_goback.lua:6-48", "vscripts/addon_game_mode.lua:294,397" },
      },
      -------------------------------------------------------------------------
      { id = "b15_v20_giant_strength", source_id = "B15W_20", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 45, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B15.txt:759" },
          mana_cost = { 140, 160, 180, 200, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B15.txt:762" },
          duration = { 40, unit = "s", source = "kv:ApplyModifier Duration@npc/B_Unified/B15.txt:788" },
          attack_bonus = { 20, 30, 40, 50, unit = "dmg", source = "kv:attack_bonus@npc/B_Unified/B15.txt:828",
            note = "modifier_b15w BASEATTACK_BONUSDAMAGE（白字）" },
          str_bonus = { 5, 10, 15, 20, unit = "stat", source = "kv:power@npc/B_Unified/B15.txt:833",
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
          { id = "b15w_20_giant",
            text = { zh_TW = { name = "巨人之臂力", body = "基礎攻擊力提高 {attack_bonus}，力量提高 {str_bonus}。" },
              en = { name = "Giant's Strength", body = "+{attack_bonus} base attack damage and +{str_bonus} strength." } },
            duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b15w_20_giant_aura",
            stats = { base_attack_damage = "$attack_bonus", str = "$str_bonus" },
            notes = "原作 modifier_b15w（與本體、舊版同名）：BASEATTACK_BONUSDAMAGE、STATS_STRENGTH_BONUS；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b15w_20_vo", at = "caster" },
            { op = "apply_status", status = "b15w_20_giant", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B15.txt:784-789",
            text = "持續時間寫死在 ApplyModifier Duration 40，不隨等級變化。" },
          { policy = "as_source", ref = "npc/B_Unified/B15.txt:771-790",
            text = "2.0 版 OnSpellStart 沒有 RunScript：不呼叫 B15W_on_spell_start，沒有模型放大、沒有淨化敵方狀態（皆為本體才有）。音效沿用 B15W_old.vo1。" },
        },
        provenance = { "npc/B_Unified/B15.txt:748-836" },
      },
      -------------------------------------------------------------------------
      { id = "b15_v20_flame_arrow", source_id = "B15E_20", slot = "W", kind = "toggle", max_level = 4, tags = { "basic", "toggle", "orb" },
        values = {
          mana_per_attack = { 6, 8, 10, 12, unit = "mana", source = "kv:mana_cost@npc/B_Unified/B15.txt:974",
            note = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:87-101 SpendMana" },
          damage = { 15, 30, 45, 60, unit = "dmg", damage_type = "magical", source = "kv:damage_bonus@npc/B_Unified/B15.txt:964",
            note = "D22 修正後本體命中加傷（原作報錯從未生效）" },
          illusion_damage = { 15, 30, 45, 60, unit = "dmg", damage_type = "magical", source = "kv:damage_bonus2@npc/B_Unified/B15.txt:969",
            note = "B15E_attack2 幻象命中加傷" },
        },
        text = {
          zh_TW = { name = "烈焰箭",
            body = "開啟時普攻改為火箭，每次普攻開始都消耗 {mana_per_attack} 法力；法力不足時自動關閉並耗盡剩餘法力。普攻命中時額外造成 {damage} 魔法傷害（建築也一樣）。{@b15_v20_tokaido_archer}的幻象在開啟期間射出的火箭額外造成 {illusion_damage} 魔法傷害。" },
          en = { name = "Flame Arrows",
            body = "While active, Yoshimoto fires flaming arrows and each attack he starts costs {mana_per_attack} mana; if he lacks the mana, the ability turns off and his remaining mana is drained. Attacks that land deal {damage} bonus magic damage (buildings included). While it is on, flaming arrows from {@b15_v20_tokaido_archer} illusions deal {illusion_damage} bonus magic damage." },
        },
        target = { type = "none" },
        passive = { orb_slot = { release = "status_end", claim = true, overridable = true } },
        statuses = {
          { id = "b15e_20_orb",
            text = { zh_TW = { name = "烈焰箭", body = "每次普攻開始消耗 {mana_per_attack} 法力；普攻命中時額外造成 {damage} 魔法傷害。" },
              en = { name = "Flame Arrows", body = "Each attack costs {mana_per_attack} mana; attacks that land deal {damage} bonus magic damage." } },
            duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b15e_20_searing_arrow_projectile", remove_on = { "death" },
            triggers = {
              -- FC-104（D13，照本體）：死亡時關閉開關（釋放法球槽），復活後不會開著卻沒有效果
              on_death = { { op = "toggle_ability", of = "self", ability = "b15_v20_flame_arrow", on = false } },
              on_attack_windup = {
                { op = "if", cond = "self.mana < $mana_per_attack", actions = {
                  { op = "toggle_ability", of = "self", ability = "b15_v20_flame_arrow", on = false },
                } },
                { op = "spend_mana", from = "self", amount = "min(self.mana, $mana_per_attack)" },
              },
              on_attack_hit = {
                filter = "orb_slot_free(self, ability)",
                actions = {
                  { op = "damage", to = "event.target", type = "magical", amount = "$damage" },
                  { op = "cue", id = "b15e_20_split_sound", at = "event.target" },
                } },
            },
            notes = "原作 modifier_searing_arrow（與本體、舊版同名）：Orb 只換投射物外觀（CastAttack 0）；OnAttackStart 扣魔（B15E）、OnAttackLanded 呼叫 B15E_attack2（原作報錯，D22 修正為加傷）；沒寫 IsPurgable，暫定 basic。" },
          { id = "b15e_20_illusion_orb",
            text = { zh_TW = { name = "烈焰箭", body = "每次普攻開始消耗 {mana_per_attack} 法力；本體開著烈焰箭時，普攻命中額外造成 {illusion_damage} 魔法傷害。" },
              en = { name = "Flame Arrows", body = "Each attack costs {mana_per_attack} mana; while the original has Flame Arrows on, attacks that land deal {illusion_damage} bonus magic damage." } },
            duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b15e_20_searing_arrow_projectile",
            triggers = {
              on_attack_windup = { { op = "spend_mana", from = "self", amount = "min(self.mana, $mana_per_attack)" } },
              on_attack_hit = {
                { op = "if", cond = "exists(self.owner) and has_status(self.owner, b15e_20_orb)", actions = {
                  { op = "damage", to = "event.target", type = "magical", amount = "$illusion_damage" },
                } },
                { op = "cue", id = "b15e_20_split_sound", at = "event.target" },
              },
            },
            notes = "東海道一弓取在本體帶 modifier_searing_arrow 時，以本體的 B15E_20 對幻象套同名 modifier（duration 999）：幻象每次攻擊照扣自己的法力；加傷原作報錯，D22 修正為檢查本體的開關後加 damage_bonus2。幻象只活 4 秒，duration 999 寫成 infinite。" },
        },
        events = {
          on_toggle_on = { { op = "apply_status", status = "b15e_20_orb", to = "caster" } },
          on_toggle_off = { { op = "remove_status", from = "caster", status = "b15e_20_orb" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/B_Unified/B15.txt:906-912; vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:74-85",
            text = "FC-148 D22 舊版，比照 D13 修：2.0 版本體的 modifier_searing_arrow 命中時呼叫的是幻象用的 B15E_attack2（本體與舊版是 B15E_attack1／B15E_attack3）：它讀 caster.owner.nobuorb1，而英雄本身與 B15T_20 建立的幻象都沒有設 owner（只有本體 B15T_create_illusion 會設），nil 取欄位報錯中止，damage_bonus 15/30/45/60 與 damage_bonus2 從未造成傷害。修法照 B15E_attack2 的設計意圖：本體命中時檢查自己的法球槽（nobuorb1 == B15E_1，以 orb_slot_free 表達）加 damage_bonus 魔法傷害；幻象命中時檢查本體開關加 damage_bonus2。B15E_attack2 沒有建築折減，照它不折減。加傷走 AMHC:Damage（義元已死不傷害）；幻象的魔法傷害另由規則層乘 illusion_damage 0.3。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:15-24",
            text = "法球槽 nobuorb1：開啟時寫成 B15E_1（覆寫道具佔用），關閉時只有槽仍是自己才清空；照本體寫成 orb_slot = { claim, overridable, release = status_end }。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:87-101",
            text = "扣魔在 OnAttackStart（前搖開始）：攻擊被取消仍扣魔，打建築也扣。法力不足時先 CastAbilityToggle 關閉，再 SpendMana 把剩餘法力扣到零。" },
          { policy = "fixed", ref = "npc/B_Unified/B15.txt:887-889",
            text = "FC-104（D13 修正，照本體 B15E）：死亡時關閉開關並釋放法球槽。原作 modifier_searing_arrow 不是 Passive，死亡時被移除，但開關仍亮（推定）。" },
          { policy = "as_source", ref = "npc/B_Unified/B15.txt:849-851",
            text = "原作 KV 沒有 AbilityCooldown，AbilityManaCost 0：開關本身不耗魔，每次攻擊扣 mana_cost。modifier_searing_arrow2 宣告了但 2.0 腳本沒有使用。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:87-101",
            text = "幻象的 b15e_20_illusion_orb：原作在幻象法力不足時以幻象對本體的技能呼叫 CastAbilityToggle（行為不明），資料只照扣法力、不關閉。" },
        },
        provenance = { "npc/B_Unified/B15.txt:838-977", "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:15-24",
          "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:74-101", "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:405-410" },
      },
      -------------------------------------------------------------------------
      { id = "b15_v20_giant_hide", source_id = "B15R_20", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          str_bonus = { 5, 10, 15, 20, unit = "stat", source = "kv:Stronger_Bonus@npc/B_Unified/B15.txt:1010" },
          block = { 16, 29, 42, 55, unit = "dmg", source = "kv:Damage_Income@npc/B_Unified/B15.txt:1015", note = "PHYSICAL_CONSTANT_BLOCK" },
        },
        text = {
          zh_TW = { name = "巨人之外皮",
            body = "被動：力量 +{str_bonus}，每次受到的物理傷害減少 {block}。" },
          en = { name = "Giant's Hide",
            body = "Passive: +{str_bonus} strength, and each instance of physical damage taken is reduced by {block}." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { str = "$str_bonus", physical_block = "$block" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B15.txt:985-1004",
            text = "modifier_B15R（力量）與 block_B15R_20（PHYSICAL_CONSTANT_BLOCK，每次物理傷害固定格擋）；數值與本體相同。是否也格擋物理技能傷害依引擎實作，交給規則層的 physical_block（同本體）。" },
        },
        provenance = { "npc/B_Unified/B15.txt:979-1018" },
      },
      -------------------------------------------------------------------------
      { id = "b15_v20_tokaido_archer", source_id = "B15T_20", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades@npc/B_Unified/B15.txt:1032-1033" },
          chance = { 23, 28, 33, unit = "pct", source = "kv:chance@npc/B_Unified/B15.txt:1080", note = "datadriven Random Chance" },
          max_illusions = { 10, unit = "count", source = "script:vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:335 ability.count < 10" },
          illusion_offset = { 100, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:344-351 上下左右 100" },
          illusion_duration = { 4, unit = "s", source = "script:vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:415 modifier_illusion duration 4" },
          illusion_dealt = { 20, unit = "pct",
            source = "script:vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:415 outgoing_damage = -80" },
          illusion_spell_dealt = { 30, unit = "pct", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:403 illusion_damage = 0.3；eventfordamage.lua:63-64 非物理傷害 × illusion_damage" },
          illusion_incoming = { 200, unit = "pct", source = "script:vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:415 incoming_damage = 200" },
        },
        text = {
          zh_TW = { name = "東海道一弓取",
            body = "被動：普攻命中時有 {chance} 機率在身旁（上下左右隨機一側）產生一個自身的幻象，最多同時 {max_illusions} 個。幻象持續 {illusion_duration} 秒，生命與本體當下相同，只造成 {illusion_dealt} 傷害，承受的傷害增加 {illusion_incoming}；幻象沒有這個技能。本體開著{@b15_v20_flame_arrow}時，幻象的普攻也是火箭並消耗自己的法力。" },
          en = { name = "Finest Archer of the Tokaido",
            body = "Passive: attacks that land have a {chance} chance to create an illusion of Yoshimoto beside him (north, south, east or west at random), up to {max_illusions} at once. Illusions last {illusion_duration} seconds, start with his current health, deal only {illusion_dealt} damage and take {illusion_incoming} more damage; they do not have this ability. If {@b15_v20_flame_arrow} is on, the illusions also fire flaming arrows that cost their own mana." },
        },
        target = { type = "none" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = { actions = {
              { op = "chance", p = "$chance", actions = {
                { op = "if", cond = "state.b15_v20_illusion_count < $max_illusions and alive(self)", actions = {
                  { op = "set_state", var = "b15_v20_illusion_count", on = "self", value = "state.b15_v20_illusion_count + 1" },
                  { op = "random_pick", as = "spot", options = {
                    { weight = "1", value = "offset(self, vec(0, 1), $illusion_offset)" },
                    { weight = "1", value = "offset(self, vec(0, -1), $illusion_offset)" },
                    { weight = "1", value = "offset(self, vec(1, 0), $illusion_offset)" },
                    { weight = "1", value = "offset(self, vec(-1, 0), $illusion_offset)" },
                  } },
                  { op = "create_illusion", of = "self", as = "ill", control = "owner", at = "spot",
                    duration = "$illusion_duration",
                    copy = { level = true, abilities = true, except_abilities = { "b15_v20_tokaido_archer" }, items = "fresh",
                      statuses = {}, hp = "self.hp", mana = "self.max_mana", unit_state = false },
                    outgoing = { physical = "$illusion_dealt - 1", magical = "$illusion_spell_dealt - 1", pure = "$illusion_spell_dealt - 1" },
                    incoming = { all = "$illusion_incoming" } },
                  { op = "apply_status", status = "b15t_20_count", to = "ill", source = "self" },
                  { op = "if", cond = "has_status(self, b15e_20_orb)", actions = {
                    { op = "apply_status", status = "b15e_20_illusion_orb", to = "ill", source = "ill" },
                  } },
                } },
              } },
            } },
          },
        },
        statuses = {
          { id = "b15t_20_count", duration = "infinite", stacking = "keep", dispel = "none", visibility = "hidden",
            reveal = "team",
            triggers = {
              on_death = { { op = "set_state", var = "b15_v20_illusion_count", on = "caster", value = "state.b15_v20_illusion_count - 1" } },
            },
            notes = "原作 modifier_B15T_20_count：套在幻象上，OnDeath 讓 ability.count − 1（幻象到期也算死亡）。沒寫 IsPurgable；隱藏計數，寫 dispel none。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B15.txt:1037-1060",
            text = "datadriven Random Chance 23/28/33%（不是 PRD）；攻擊建築命中也會觸發。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:331-357",
            text = "ability.count < 10 才產生並 +1；幻象死亡（含到期）時 modifier_B15T_20_count 的 OnDeath −1。方位 RandomInt(1,4) 決定上下左右 100；本體已死不產生。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:360-418",
            text = "幻象以 CreateUnitByName 建立並升到本體等級，照本體技能等級加入除 B15T_20 以外的技能（所以帶百步穿楊、巨人之外皮），複製 0～5 格道具為新實例，生命等於本體目前生命，可由玩家控制；不複製狀態。法力是新英雄的滿法力，copy.mana 寫本體最大法力（推定相同）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:403,415; vscripts/library/events/eventfordamage.lua:63-64",
            text = "modifier_illusion outgoing −80%、incoming +200%；全域傷害過濾另把幻象的非物理傷害乘 illusion_damage 0.3。Dota 的 −80% 是否也作用於幻象的魔法傷害依引擎，資料照本體 B15 慣例只取 30%，待 M2 實測。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:404",
            text = "幻象出生時套 0.1 秒 modifier_phased 避免卡位，屬出生細節，交給 create_illusion 的空地尋找。" },
        },
        provenance = { "npc/B_Unified/B15.txt:1020-1083", "vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:326-419",
          "vscripts/library/events/eventfordamage.lua:63-64" },
      },
    },
  }
end
