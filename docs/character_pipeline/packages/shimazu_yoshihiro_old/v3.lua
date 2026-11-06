-- C03 島津義弘（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C03"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:319）；自動學習 heromap_autoskill["11"] = ""
-- （addon_game_mode.lua:422），沒有 D。W/E/R/T → Q/W/E/R。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 C03*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- KV 的 C03E_old、C03R_old、C03T_old 都呼叫本體的腳本函式（C03E_OnSpellStart、C03R_*、C03T_OnSpellStart），
-- 只有 C03W_old 用 C03W_old_OnSpellStart；數值都讀舊版自己的 AbilitySpecial。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C03_precache 屬規則層或資產，不列入。
-- 慣例照本體 C03：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C03_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C03", text = { zh_TW = { name = "島津義弘（舊版）" }, en = { name = "Shimazu Yoshihiro (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c03_old_iron_shirt", source_id = "C03W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 55, 50, 45, 40, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C03.txt:340" },
          mana_cost = { 170, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C03.txt:343" },
          duration = { 13, unit = "s", source = "kv:modifier_C03w Duration@npc/C_Neutral/C03.txt:376" },
          base_attack = { 10, 20, 30, 40, unit = "dmg", source = "kv:attack_bonus@npc/C_Neutral/C03.txt:398",
            note = "KV 屬性 MODIFIER_PROPERTY_BASEATTACK_BONUSDAMAGE" },
          armor = { 4, 6, 8, 10, unit = "armor", source = "kv:armor@npc/C_Neutral/C03.txt:403" },
          max_hp = { 200, 300, 400, 500, unit = "hp", source = "kv:hpadd@npc/C_Neutral/C03.txt:408",
            note = "KV 屬性 MODIFIER_PROPERTY_HEALTH_BONUS" },
        },
        text = {
          zh_TW = { name = "鐵布衫",
            body = "立即施放（不打斷目前動作）：移除自身所有由敵方施加的狀態，並在 {duration} 秒內魔法免疫，基礎攻擊力 +{base_attack}、護甲 +{armor}、最大生命 +{max_hp}。" },
          en = { name = "Iron Shirt",
            body = "Cast instantly without interrupting the current action: removes every status applied to Yoshihiro by enemies, then for {duration} seconds he is spell immune and gains +{base_attack} base attack damage, +{armor} armor and +{max_hp} maximum health." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c03w_old_iron", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "鐵布衫", body = "魔法免疫，基礎攻擊力 +{base_attack}、護甲 +{armor}、最大生命 +{max_hp}。" },
              en = { name = "Iron Shirt", body = "Spell immune; +{base_attack} base attack damage, +{armor} armor and +{max_hp} maximum health." },
            },
            reveal = "public", cue = "c03w_old_iron_glow",
            states = { "magic_immune" },
            stats = { base_attack_damage = "$base_attack", armor = "$armor", max_hp = "$max_hp" },
            notes = "原作 modifier_C03w（C03W_old 版）：BASEATTACK_BONUSDAMAGE、PHYSICAL_ARMOR_BONUS、HEALTH_BONUS、MAGIC_IMMUNE；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c03w_old_vo", at = "caster" },
            { op = "apply_status", status = "c03w_old_iron", to = "caster" },
            { op = "remove_status", from = "caster", dispel = "all", team_of_source = "enemy" },
            { op = "cue", id = "c03w_old_grow", at = "caster", duration = "$duration" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C03.txt:340-408",
            text = "和本體不同：持續 13 秒（本體 6）、冷卻 55/50/45/40（本體 20）、耗魔固定 170、基礎攻擊力 +10～40、護甲 +4～10；最大生命加成相同。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C03.lua:24-31",
            text = "C03W_old_OnSpellStart 與本體相同：FindAllModifiers 後以 RemoveModifierByName 移除「施加者隊伍不同」的每一個 modifier，不看可驅散屬性；施加者已不存在的 modifier 保留。資料以 dispel = all、team_of_source = enemy 表達。" },
          { policy = "as_source", ref = "npc/C_Neutral/C03.txt:356-367",
            text = "KV 先套 modifier_C03w 再跑腳本；施加者是自己，不會被移除。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C03.lua:23",
            text = "AMHC:AddModelScale 讓模型在 13 秒內放大 1.3 倍，只是呈現，改用 cue。" },
          { policy = "note", ref = "npc/C_Neutral/C03.txt:336", text = "KV 沒寫 AbilityCastPoint（IMMEDIATE），前搖為零。" },
        },
        provenance = { "npc/C_Neutral/C03.txt:329-411", "vscripts/heroes/C_Neutral/C03.lua:19-32" },
      },
      -------------------------------------------------------------------------
      { id = "c03_old_quake_axe", source_id = "C03E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 16, 15, 14, 13, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C03.txt:427" },
          mana_cost = { 115, 135, 155, 175, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C03.txt:428" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C03.txt:433" },
          radius = { 400, unit = "dist",
            source = "script:vscripts/heroes/C_Neutral/C03.lua:44 ability:GetCastRange()（KV AbilityCastRange 400@C03.txt:431，AOERadius 也是 400）" },
          damage = { 90, 180, 270, 360, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C03.txt:429" },
          slow = { 20, 30, 40, 50, unit = "pct", kv = { -20, -30, -40, -50 },
            source = "kv:debuff_movement_speed@npc/C_Neutral/C03.txt:445",
            source_differs = "KV 以負值寫在 MOVESPEED_BONUS_PERCENTAGE；這裡以正值表示降低量，狀態寫 -$slow" },
          attack_slow = { 20, 30, 40, 50, unit = "atk_speed", kv = { -20, -30, -40, -50 },
            source = "kv:debuff_attack_speed@npc/C_Neutral/C03.txt:440",
            source_differs = "KV 以負值寫在 ATTACKSPEED_BONUS_CONSTANT；這裡以正值表示降低量" },
          slow_duration = { 4, unit = "s", source = "kv:modifier_C03E_debuff Duration@npc/C_Neutral/C03.txt:483" },
        },
        text = {
          zh_TW = { name = "震地斧",
            body = "重擊地面，對自身周圍 {radius} 內非魔免的敵方英雄與部隊造成 {damage} 魔法傷害，並使其移動速度降低 {slow}、攻擊速度降低 {attack_slow}，持續 {slow_duration} 秒。" },
          en = { name = "Quake Axe",
            body = "Slams the ground, dealing {damage} magic damage to non-spell-immune enemy heroes and units within {radius} and slowing their movement speed by {slow} and attack speed by {attack_slow} for {slow_duration} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c03e_old_slow", duration = "$slow_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "震地斧緩速", body = "移動速度降低 {slow}、攻擊速度降低 {attack_slow}。" },
              en = { name = "Quake Axe Slow", body = "Movement speed reduced by {slow} and attack speed reduced by {attack_slow}." },
            },
            reveal = "public", cue = "c03e_old_move_slow", tags = { "slow" },
            stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            notes = "原作 modifier_C03E_debuff（C03E_old 版）：Properties 直接寫 ATTACKSPEED_BONUS_CONSTANT 與 MOVESPEED_BONUS_PERCENTAGE（不經 nobu 緩速池）；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c03e_old_vo", at = "caster" } },
          on_cast = {
            { op = "cue", id = "c03e_old_thunder_clap", at = "caster", radius = "$radius" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
              actions = {
                { op = "apply_status", status = "c03e_old_slow", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C03.txt:480-491",
            text = "和本體不同：緩速是 KV Properties（MOVESPEED_BONUS_PERCENTAGE、ATTACKSPEED_BONUS_CONSTANT）直接生效，不寫進 nobu 緩速池；移速與攻速都降低 20/30/40/50，持續 4 秒。傷害 90/180/270/360、冷卻 16～13。" },
          { policy = "as_source", ref = "npc/C_Neutral/C03.txt:463",
            text = "OnSpellStart 呼叫本體的 C03E_OnSpellStart；ability 是 C03E_old 本身，所以讀到舊版的施法距離、傷害與 modifier。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C03.lua:41-49",
            text = "搜尋用技能本身的目標設定（ENEMY、HERO|BASIC、旗標 NONE）：魔免敵人選不到；沒有 FOW 旗標，隱形與迷霧中的敵人也會被打到。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C03.lua:63-64",
            text = "先套緩速再造成傷害（ApplyDamage，不經 AMHC）。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C03.lua:39", text = "StartGestureWithPlaybackRate 只是動作，改用 cue。" },
        },
        provenance = { "npc/C_Neutral/C03.txt:413-495", "vscripts/heroes/C_Neutral/C03.lua:35-66" },
      },
      -------------------------------------------------------------------------
      { id = "c03_old_thunder_wrath", source_id = "C03R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          damage = { 100, 180, 260, 340, unit = "dmg", damage_type = "magical", source = "kv:dmg@npc/C_Neutral/C03.txt:511" },
          chance = { 31, unit = "pct", source = "script:vscripts/heroes/C_Neutral/C03.lua:73-79 RandomInt(1,100) <= 31" },
          pity_after = { 3, unit = "count",
            source = "script:vscripts/heroes/C_Neutral/C03.lua:75-79 caster.C03E 先 +1，> 3 時必中（連續三次沒中，第四次必中）" },
          stun = { 0.4, unit = "s", source = "script:vscripts/heroes/C_Neutral/C03.lua:105,108 modifier_stunned duration = 0.4" },
        },
        text = {
          zh_TW = { name = "暴雷怒擊",
            body = "被動：對非建築目標出手時有 {chance} 機率蓄雷；連續 {pity_after} 次沒有蓄雷，下一次必定蓄雷。蓄雷的這一擊不會落空，命中非建築目標時暈眩 {stun} 秒並造成 {damage} 魔法傷害（魔免目標只會暈眩）。" },
          en = { name = "Thunder Wrath",
            body = "Passive: each attack started against a non-building has a {chance} chance to charge with thunder; after {pity_after} attacks in a row without a charge, the next one always charges. A charged attack cannot miss, and when it lands on a non-building it stuns for {stun} seconds and deals {damage} magic damage (spell-immune targets are only stunned)." },
        },
        target = { type = "none" },
        unit_state = {
          c03_old_r_armed = { type = "bool", init = "false", reveal = "public", scope = "unit",
            note = "原作 caster.C03E_go：蓄雷後等到下一次命中非建築才消耗" },
        },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_windup = { filter = "not event.target.is_building", actions = {
              { op = "chance", p = "$chance", pity_after = "$pity_after", counter = "hero", actions = {
                { op = "set_state", var = "c03_old_r_armed", value = "true", on = "self" },
                { op = "modify_attack", cannot_miss = true },
              } },
            } },
            on_attack_hit = { filter = "not event.target.is_building and state.c03_old_r_armed", actions = {
              { op = "set_state", var = "c03_old_r_armed", value = "false", on = "self" },
              { op = "cue", id = "c03r_old_thunder", at = "event.target" },
              { op = "apply_status", status = "c03r_old_stun", to = "event.target" },
              { op = "damage", to = "event.target", type = "magical", amount = "$damage" },
            } },
          },
        },
        statuses = {
          { id = "c03r_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暴雷暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Thunder Wrath Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（ApplyDataDrivenModifier 套用）；受狀態抗性縮短。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C03.txt:497-546",
            text = "和本體相同的腳本與數值（C03R_OnAttackStart／C03R_OnAttackLanded、dmg 100/180/260/340）；只有 KV AbilityUnitDamageType 寫 PURE，但腳本固定用 AMHC 魔法傷害，類型欄位沒有作用。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C03.lua:79-81",
            text = "蓄雷旗標 C03E_go 在命中時才清除：蓄雷那一擊若被取消或改打建築，旗標保留，下一次命中非建築時才觸發。" },
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C03.lua:82-83",
            text = "必中是套 modifier_C03R_nomiss，持續 GetAttackSpeed()+0.1 秒，每次出手先移除；照設計意圖寫成只作用在這一次攻擊（modify_attack cannot_miss）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C03.lua:104-109",
            text = "魔免目標只暈眩、不受傷害；資料的魔法傷害由規則層對魔免歸零。傷害走 AMHC:Damage，義弘已死不造成傷害。" },
          { policy = "as_source",
            text = "OnAttackLanded 不分敵我，反補友軍時若有蓄雷也會暈眩並消耗旗標。" },
        },
        provenance = { "npc/C_Neutral/C03.txt:496-547", "vscripts/heroes/C_Neutral/C03.lua:68-112" },
      },
      -------------------------------------------------------------------------
      { id = "c03_old_iron_hammer", source_id = "C03T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 150, 140, 130, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C03.txt:560" },
          mana_cost = { 250, 350, 450, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C03.txt:564" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C03.txt:558" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades@npc/C_Neutral/C03.txt:561-562" },
          damage = { 70, 110, 150, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C03.txt:563",
            note = "AbilityUnitDamageType DAMAGE_TYPE_MAGICAL（C03.txt:555）" },
          radius = { 500, unit = "dist",
            source = "script:vscripts/heroes/C_Neutral/C03.lua:167 FindUnitsInRadius 500（KV AOERadius 也是 500@C03.txt:565）" },
          stun = { 0.4, 0.45, 0.51, unit = "s", source = "kv:stun_time@npc/C_Neutral/C03.txt:624" },
          interval = { 0.5, unit = "s", source = "kv:check_time@npc/C_Neutral/C03.txt:629" },
          total_time = { 7, unit = "s", source = "kv:total_time@npc/C_Neutral/C03.txt:634" },
          speed = { 100, unit = "speed",
            source = "script:vscripts/heroes/C_Neutral/C03.lua:135 speed = 100（knockback 2000 距離、20 秒）" },
          forward_offset = { 300, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C03.lua:162 pos + dir * 300" },
        },
        text = {
          zh_TW = { name = "不動戰錘",
            body = "朝目標方向召喚巨大戰錘，以每秒 {speed} 的速度前進 {total_time} 秒。從施放當下起每 {interval} 秒，戰錘前方 {forward_offset} 處 {radius} 範圍內非魔免的敵方英雄與部隊受到 {damage} 魔法傷害並暈眩 {stun} 秒。" },
          en = { name = "Immovable War Hammer",
            body = "Summons a giant war hammer that advances in the target direction at {speed} per second for {total_time} seconds. Starting immediately and every {interval} seconds, non-spell-immune enemy heroes and units within {radius} of a point {forward_offset} ahead of the hammer take {damage} magic damage and are stunned for {stun} seconds." },
        },
        target = { type = "point", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "c03t_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "不動戰錘暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "War Hammer Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（ApplyDataDrivenModifier 套用）；不在 ultimate_modifiers，受狀態抗性縮短。KV 的 modifier_C03T 沒有被使用。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c03t_old_vo", at = "caster" },
            { op = "let", name = "start", value = "position(caster)" },
            { op = "let", name = "dir", value = "select(distance(caster, point) > 0, direction(caster, point), caster.forward)" },
            { op = "cue", id = "c03t_old_hammer", at = "start", to = "offset(start, dir, $speed * $total_time)", duration = "$total_time" },
            { op = "repeat", count = "floor($total_time / $interval) + 1", interval = "$interval", first = "0", as = "k",
              actions = {
                { op = "let", name = "center", value = "offset(start, dir, $speed * $interval * k + $forward_offset)" },
                { op = "cue", id = "c03t_old_hammer_hit", at = "center", radius = "$radius" },
                { op = "for_each",
                  select = { from = "center", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                  actions = {
                    { op = "apply_status", status = "c03t_old_stun", to = "it" },
                    { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C03.txt:582",
            text = "C03T_old 的 OnSpellStart 呼叫本體的 C03T_OnSpellStart（不是同檔的 C03T_old_OnSpellStart，那個函式沒有人呼叫）；兩者差別只在魔免判斷，而舊版 KV 沒有 MAGIC_IMMUNE_ENEMIES 旗標，魔免敵人本來就選不到，結果相同。stun_time2 不存在、讀到 0，該行已註解。" },
          { policy = "as_source", ref = "npc/C_Neutral/C03.txt:555-634",
            text = "和本體不同：魔法傷害 70/110/150（本體物理）、暈眩 0.4/0.45/0.51、每 0.5 秒一跳、戰錘存在 7 秒、冷卻 150/140/130、耗魔 250/350/450；沒有 MAGIC_IMMUNE_ENEMIES 旗標，魔免敵人不受影響。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C03.lua:158-199",
            text = "Timer 從 0 秒起每 0.5 秒一跳，戰錘在 7 秒時銷毀；第 14 次重排剛好落在 7 秒，和銷毀 Timer 同一個 think，執行順序依 Timers 表的走訪順序而定。資料照 floor(總時間/間隔)+1 算到 7 秒那一跳（共 15 跳），見 assumptions.md。同一個敵人在範圍內每跳都會被打。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C03.lua:121-156",
            text = "戰錘是 C03T_UNIT（majia：無敵、不可選取），以 modifier_knockback 推 2000 距離、20 秒，等於每秒 100；7 秒後被殺掉，實際只前進約 700。單位本身只是呈現，資料以起點加位移計算位置、用 cue 呈現。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C03.lua:174-192",
            text = "ApplyDamage 不經 AMHC：義弘死亡後戰錘照樣造成傷害（require_source_alive = false）。" },
          { policy = "note", ref = "npc/C_Neutral/C03.txt:549-566",
            text = "KV 沒寫 AbilityCastRange：點目標不限距離，只用來決定方向。指定點等於自身位置時改用面向。" },
        },
        provenance = { "npc/C_Neutral/C03.txt:548-637", "vscripts/heroes/C_Neutral/C03.lua:115-200",
          "npc/npc_units_custom_load.txt:5833-5861" },
      },
    },
  }
end
