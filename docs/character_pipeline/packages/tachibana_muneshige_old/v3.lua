-- C11 立花宗茂（舊版，11 版 *_old，KV 註解「11.2B」）— package schema v3。每個版本是一位獨立英雄（D22）；2.0 版（*_20）另成 tachibana_muneshige_v20。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C11"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:327）；沒有 D，自動學習 heromap_autoskill["C11"]["11"] = ""（addon_game_mode.lua:430）。
-- W/E/R/T → Q/W/E/R。原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 C11*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C11_precache 不列入。
-- 慣例照本體 C11：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；
-- ApplyDamage（不經 AMHC）在宗茂死亡後仍會造成傷害，寫 require_source_alive = false；HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C11_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C11", text = { zh_TW = { name = "立花宗茂（舊版）" }, en = { name = "Tachibana Muneshige (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c11_old_raikiri", source_id = "C11W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C11.txt:541" },
          mana_cost = { 130, 160, 190, 220, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C11.txt:542" },
          cast_point = { 0, unit = "s", source = "rules:KV 沒寫 AbilityCastPoint（預設 0），且是 IMMEDIATE" },
          duration = { 20, unit = "s", source = "kv:duration@npc/C_Neutral/C11.txt:564" },
          base_damage = { 30, 40, 50, 60, unit = "dmg", source = "kv:add_damage@npc/C_Neutral/C11.txt:559（BASEATTACK_BONUSDAMAGE）" },
          crit_chance = { 20, unit = "pct", source = "kv:crit_chance@npc/C_Neutral/C11.txt:549",
            note = "C11.lua:343-346 RandomInt(1,100) <= crit_chance，實際機率正好 20%" },
          crit = { 200, 250, 300, 350, unit = "pct", source = "kv:crit_bonus@npc/C_Neutral/C11.txt:554（PREATTACK_CRITICALSTRIKE）" },
        },
        text = {
          zh_TW = { name = "雷切",
            body = "立即在 {duration} 秒內基礎攻擊力 +{base_damage}，每次出手有 {crit_chance} 機率造成 {crit} 的爆擊。" },
          en = { name = "Raikiri",
            body = "Instantly gains +{base_damage} base attack damage for {duration} seconds, and each attack has a {crit_chance} chance to critically strike for {crit} damage." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c11w_old_blade", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "雷切", body = "基礎攻擊力 +{base_damage}；每次出手有 {crit_chance} 機率造成 {crit} 的爆擊。" },
              en = { name = "Raikiri", body = "+{base_damage} base attack damage; each attack has a {crit_chance} chance to crit for {crit} damage." },
            },
            reveal = "public", cue = "c11w_old_buff",
            stats = { base_attack_damage = "$base_damage" },
            triggers = {
              on_attack_windup = {
                { op = "chance", p = "$crit_chance", actions = {
                  { op = "modify_attack", crit_multiplier = "$crit" },
                } },
              },
            },
            notes = "原作 modifier_C11W_old：BASEATTACK_BONUSDAMAGE；OnAttackStart 先移除再以 crit_chance 套上隱藏的 modifier_C11W_old_crit（PREATTACK_CRITICALSTRIKE），OnDestroy 移除爆擊；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c11w_old_vo", at = "caster" } },
          on_cast = { { op = "apply_status", status = "c11w_old_blade", to = "caster" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C11.lua:339-350",
            text = "原作在 OnAttackStart 擲骰並套一個 PREATTACK_CRITICALSTRIKE 的隱藏 modifier，留到下一次出手前搖才移除；v3 直接 modify_attack 只影響這一次攻擊。前搖被取消時原作的爆擊 modifier 會留著，但下一次出手前又會重擲，結果相同。" },
          { policy = "as_source", ref = "npc/C_Neutral/C11.txt:530-645",
            text = "和本體完全不同：舊版雷切是自身增益（基礎攻擊力與爆擊），不是穿梭彈跳。BASEATTACK_BONUSDAMAGE 加在基礎攻擊力（白字）。" },
          { policy = "note", ref = "npc/C_Neutral/C11.txt:568-626",
            text = "C11W_old.vo1 與 c11w_old_buff 粒子只是呈現；KV 的 AbilityUnitTargetFlags 與傷害類型沒有作用。" },
        },
        provenance = { "npc/C_Neutral/C11.txt:528-645", "vscripts/heroes/C_Neutral/C11.lua:339-350" },
      },
      -------------------------------------------------------------------------
      { id = "c11_old_thunder_haste", source_id = "C11E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C11.txt:656" },
          mana_cost = { 50, 55, 60, 65, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C11.txt:657" },
          cast_point = { 0, unit = "s", source = "rules:KV 沒寫 AbilityCastPoint（預設 0），且是 IMMEDIATE" },
          duration = { 20, unit = "s", source = "kv:AbilityDuration@npc/C_Neutral/C11.txt:658" },
          attack_speed = { 20, 40, 60, 80, unit = "atk_speed", source = "kv:bonus_attack_speed@npc/C_Neutral/C11.txt:665" },
          move_speed = { 7, 9, 11, 13, unit = "pct", source = "kv:bonus_movement_speed@npc/C_Neutral/C11.txt:670" },
        },
        text = {
          zh_TW = { name = "雷之疾", body = "立即在 {duration} 秒內攻擊速度 +{attack_speed}、移動速度提高 {move_speed}。" },
          en = { name = "Thunder Haste", body = "Instantly gains +{attack_speed} attack speed and {move_speed} movement speed for {duration} seconds." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c11e_old_haste", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "雷之疾", body = "攻擊速度 +{attack_speed}，移動速度提高 {move_speed}。" },
              en = { name = "Thunder Haste", body = "+{attack_speed} attack speed and {move_speed} movement speed." },
            },
            reveal = "public", cue = "c11e_old_static_link",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed" },
            notes = "原作 modifier_C11E_old：ATTACKSPEED_BONUS_CONSTANT、MOVESPEED_BONUS_PERCENTAGE；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c11e_old_vo", at = "caster" } },
          on_cast = { { op = "apply_status", status = "c11e_old_haste", to = "caster" } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C11.txt:647-732",
            text = "只對自己（ApplyModifier Target CASTER）；KV 的 TEAM_FRIENDLY、HERO 沒有作用。攻速是固定值（ATTACKSPEED_BONUS_CONSTANT），不是百分比。" },
          { policy = "note", ref = "npc/C_Neutral/C11.txt:708-722", text = "c11e_debuff 與 razor_static_link_buff 粒子只是呈現。" },
        },
        provenance = { "npc/C_Neutral/C11.txt:646-732" },
      },
      -------------------------------------------------------------------------
      { id = "c11_old_thunder_eye", source_id = "C11R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 18, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C11.txt:745" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C11.txt:746" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C11.txt:749" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C11.txt:750" },
          radius = { 450, unit = "dist", source = "kv:radius@npc/C_Neutral/C11.txt:764（AoERadius@C11.txt:752 同值）" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C11.txt:747" },
          stun = { 0.4, 0.8, 1.1, 1.6, unit = "s", source = "kv:stun_time@npc/C_Neutral/C11.txt:759" },
          vision_radius = { 600, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C11.lua:191 AddFOWViewer 600" },
          vision_time = { 2, unit = "s", source = "script:vscripts/heroes/C_Neutral/C11.lua:191 AddFOWViewer 2" },
          bolt_delay = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C11.lua:243" },
          bolt_vision_time = { 0.5, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C11.lua:245" },
        },
        text = {
          zh_TW = { name = "雷之眼",
            body = "在目標區域 {radius} 範圍降下落雷，對其中的敵方英雄與部隊造成 {damage} 魔法傷害並暈眩 {stun} 秒；並取得該處 {vision_radius} 範圍的視野 {vision_time} 秒。" },
          en = { name = "The Eye of Thunder",
            body = "Calls down lightning on the target area, dealing {damage} magic damage to enemy heroes and units within {radius} and stunning them for {stun} seconds; also grants vision within {vision_radius} there for {vision_time} seconds." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c11r_old_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "雷之眼暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Eye of Thunder Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" }, cue = "c11r_old_shock",
            notes = "原作 C11R_old 自己的 modifier_C11R_stun：IsStunDebuff、STUNNED、Duration %stun_time；沒寫 IsPurgable，暫定 basic（同本體）。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c11r_old_vo", at = "caster" } },
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "reveal", at = "at", radius = "$vision_radius", duration = "$vision_time", teams = "own" },
            { op = "for_each",
              select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
              actions = {
                { op = "apply_status", status = "c11r_old_stun", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false, flags = { "no_amplify" } },
              } },
            { op = "cue", id = "c11r_old_thunder_ring", at = "at" },
            { op = "delay", time = "$bolt_delay", capture = { "at" }, actions = {
              { op = "cue", id = "c11r_old_thunder_sound", at = "at" },
              { op = "reveal", at = "at", radius = "$radius", duration = "$bolt_vision_time", teams = "own" },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C11.txt:766-790",
            text = "和本體共用腳本 C11R_start（C11.lua:179-248），舊版只差數值：施法距離 700、前搖 0.25、暈眩 0.4/0.8/1.1/1.6。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:192-215",
            text = "沒有 MAGIC_IMMUNE 旗標，魔免敵人不受影響；沒有 FOW 旗標，隱形與迷霧中的敵人照樣受傷。先暈眩再以 ApplyDamage（沒有傳 ability，同本體寫 no_amplify）造成傷害。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C11.lua:216-246",
            text = "六道落雷特效、螢幕特效與音效 dummy 屬呈現，改用 cue；0.1 秒後另給 450 範圍 0.5 秒的視野。" },
        },
        provenance = { "npc/C_Neutral/C11.txt:734-812", "vscripts/heroes/C_Neutral/C11.lua:179-248,328-335" },
      },
      -------------------------------------------------------------------------
      { id = "c11_old_unrivaled", source_id = "C11T_old", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          required_level = { 6, 12, 18, unit = "level", source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          remove_mana = { 150, 200, 250, unit = "mana", source = "kv:remove_mana@npc/C_Neutral/C11.txt:830" },
          hero_ratio = { 1.35, 1.45, 1.55, unit = "ratio", source = "kv:damage_adjust_for_hero@npc/C_Neutral/C11.txt:835" },
          creep_ratio = { 1.8, 2, 2.2, unit = "ratio", source = "kv:damage_adjust_for_creep@npc/C_Neutral/C11.txt:840" },
        },
        text = {
          zh_TW = { name = "西國無雙",
            body = "被動：普攻命中有法力的非建築目標時，消去它最多 {remove_mana} 法力，並造成消去量 {hero_ratio} 倍（對非英雄為 {creep_ratio} 倍）的純粹傷害。" },
          en = { name = "Unrivaled in the West",
            body = "Passive: attacks that land on a non-building with mana remove up to {remove_mana} of its mana and deal pure damage equal to {hero_ratio} times the mana removed ({creep_ratio} times against non-heroes)." },
        },
        target = { type = "none" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = { filter = "not event.target.is_building and event.target.max_mana > 0", actions = {
              { op = "let", name = "removed", value = "min(event.target.mana, $remove_mana)" },
              { op = "if", cond = "event.target.mana >= $remove_mana", actions = {
                { op = "cue", id = "c11t_old_moonfall", at = "event.target" },
              } },
              { op = "burn_mana", to = "event.target", source = "self", amount = "removed" },
              { op = "damage", to = "event.target", type = "pure", require_source_alive = false,
                amount = "select(event.target.is_hero, removed * $hero_ratio, removed * $creep_ratio)" },
            } },
            on_kill = { filter = "event.victim.is_hero", actions = {
              { op = "cue", id = "c11t_old_title" },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:363-412",
            text = "OnAttackLanded 不分敵我（反補友軍也會消魔與傷害）；建築與最大法力為零的單位跳過。消去量取 min(目前法力, remove_mana)，以 ReduceMana 扣除；傷害是消去量乘倍率的純粹傷害（ApplyDamage，不經 AMHC），法力為零的單位不受傷。分身判定用 IsHero（分身也算英雄）。" },
          { policy = "as_source", ref = "npc/C_Neutral/C11.txt:826-842",
            text = "AbilitySpecial 兩個條目都用鍵名 \"02\"；KV 保留重複鍵，GetSpecialValueFor 依變數名稱查找，推定兩個倍率都生效（同 asai_nagamasa_old，見 assumptions.md）。" },
          { policy = "as_source", ref = "npc/C_Neutral/C11.txt:814-822",
            text = "和本體不同：舊版只有被動，沒有加全屬性與主動閃雷；消魔改成固定上限並以消去量計算傷害。" },
          { policy = "note", ref = "npc/C_Neutral/C11.txt:862-870 / vscripts/heroes/C_Neutral/C11.lua:383-410",
            text = "OnHeroKilled 的 showTitle、moonfall 粒子（只在法力足夠時播）與跳字只是呈現。" },
        },
        provenance = { "npc/C_Neutral/C11.txt:813-877", "vscripts/heroes/C_Neutral/C11.lua:363-412" },
      },
    },
  }
end
