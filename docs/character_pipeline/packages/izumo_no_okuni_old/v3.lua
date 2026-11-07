-- C04 阿國（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。2.0 版（*_20）另做。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C04"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:320）；自動學習 heromap_autoskill["11"] = ""
-- （addon_game_mode.lua:423），沒有 D。W/E/R/T → Q/W/E/R。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 C04*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C04_precache 屬規則層或資產，不列入。
-- 慣例照本體 C04：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- datadriven Properties 直接給的移速百分比寫 move_speed_pct；HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C04_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C04", text = { zh_TW = { name = "阿國（舊版）" }, en = { name = "Izumo no Okuni (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c04_old_swift_dance", source_id = "C04W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 35, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C04.txt:430" },
          mana_cost = { 105, 125, 145, 165, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C04.txt:431" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C04.txt:429" },
          radius = { 10000, unit = "dist", hidden = true,
            source = "kv:ApplyModifier Target Radius@npc/C_Neutral/C04.txt:453" },
          move_speed_pct = { 20, 25, 30, 35, unit = "pct", source = "kv:spd@npc/C_Neutral/C04.txt:476" },
          duration = { 6, 8, 10, 12, unit = "s", source = "kv:duration@npc/C_Neutral/C04.txt:481" },
        },
        text = {
          zh_TW = { name = "神速之舞",
            body = "全地圖所有友方英雄與部隊（含自己）移動速度提高 {move_speed_pct}，持續 {duration} 秒。" },
          en = { name = "Dance of Swiftness",
            body = "Every allied hero and unit on the map (including herself) gains {move_speed_pct} movement speed for {duration} seconds." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c04w_old_haste", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "神速之舞", body = "移動速度提高 {move_speed_pct}。" },
              en = { name = "Dance of Swiftness", body = "Movement speed increased by {move_speed_pct}." },
            },
            reveal = "public", cue = "c04w_old_wind", stats = { move_speed_pct = "$move_speed_pct" },
            notes = "原作 modifier_C04W（C04W_old 版）：MOVESPEED_BONUS_PERCENTAGE %spd、Duration %duration；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c04w_old_sound", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "ally", types = BASIC },
              actions = {
                { op = "apply_status", status = "c04w_old_haste", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C04.txt:446-457",
            text = "和本體不同：沒有腳本，KV ApplyModifier 直接以阿國為中心、半徑 10000 套給友方 HERO|BASIC（地圖內幾乎全部友軍），施加者是阿國；移速 20/25/30/35%、持續 6/8/10/12 秒、冷卻 35。旗標預設：無敵友軍不會拿到。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C04.txt:425-432",
            text = "KV AbilityUnitTargetTeam ENEMY、AbilityUnitDamageType、AbilityCastRange 700 都沒有被使用。" },
          { policy = "note", ref = "npc/C_Neutral/C04.txt:440-445",
            text = "item_soul_adder.lua sound 只播 powershot_falcon_bow 音效，改用 cue。" },
        },
        provenance = { "npc/C_Neutral/C04.txt:418-484", "vscripts/items/Addon_Items/item_soul_adder.lua:61-90" },
      },
      -------------------------------------------------------------------------
      { id = "c04_old_withering_step", source_id = "C04E_old", slot = "W", kind = "toggle", max_level = 4, tags = { "basic", "toggle" },
        values = {
          cooldown = { 2, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C04.txt:497" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C04.txt:496" },
          radius = { 600, unit = "dist",
            source = "script:vscripts/heroes/C_Neutral/C04.lua:189 ability:GetCastRange()（KV AbilityCastRange 600@C04.txt:500，AOERadius 也是 600）" },
          tick = { 0.2, unit = "s", source = "kv:modifier_C04E ThinkInterval@npc/C_Neutral/C04.txt:560" },
          mana_per_tick = { 2, 3, 4, 5, unit = "mana", source = "kv:mana@npc/C_Neutral/C04.txt:603", note = "OnIntervalThink 以 SpendMana 每跳扣除" },
          damage = { 15, 25, 35, 45, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C04.txt:499" },
          lock_time = { 1, unit = "s", hidden = true, source = "kv:modifier_C04E_locker Duration@npc/C_Neutral/C04.txt:538" },
        },
        text = {
          zh_TW = { name = "凋零舞步",
            body = "開啟後每 {tick} 秒消耗 {mana_per_tick} 法力；扣完後剩餘法力少於這個數就自動關閉，該跳沒有效果。否則對周圍 {radius} 內非魔免的敵方英雄與部隊造成 {damage} 魔法傷害。" },
          en = { name = "Withering Step",
            body = "While active, spends {mana_per_tick} mana every {tick} seconds; if her remaining mana is then below that amount, the ability turns off and that pulse has no effect. Otherwise, non-spell-immune enemy heroes and units within {radius} take {damage} magic damage." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c04e_old_dance", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "凋零舞步", body = "每 {tick} 秒消耗 {mana_per_tick} 法力，對周圍 {radius} 內的敵人造成 {damage} 魔法傷害；法力不足時自動關閉。" },
              en = { name = "Withering Step", body = "Every {tick} seconds spends {mana_per_tick} mana to deal {damage} magic damage to enemies within {radius}; turns off when mana runs short." },
            },
            reveal = "public", cue = "c04e_old_steam", remove_on = { "death" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "spend_mana", from = "self", amount = "$mana_per_tick", ability = "ability" },
                { op = "if", cond = "self.mana < $mana_per_tick",
                  actions = { { op = "toggle_ability", of = "self", ability = "c04_old_withering_step", on = false } },
                  otherwise = {
                    { op = "for_each",
                      select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                      actions = {
                        { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                        { op = "cue", id = "c04e_old_steam_hit", at = "it" },
                      } },
                  } },
              } },
            },
            notes = "原作 modifier_C04E（C04E_old 版）：ThinkInterval 0.2，OnIntervalThink 先 SpendMana 再跑 C04E_old_OnIntervalThink；沒寫 IsPurgable，暫定 basic。" },
          { id = "c04e_old_lock", duration = "$lock_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_created = { { op = "set_ability_active", of = "self", ability = "c04_old_withering_step", active = false } },
              on_removed = { { op = "set_ability_active", of = "self", ability = "c04_old_withering_step", active = true } },
            },
            notes = "原作 modifier_C04E_locker：SetActivated(false)，OnDestroy 恢復。" },
        },
        events = {
          on_toggle_on = {
            { op = "apply_status", status = "c04e_old_dance", to = "caster" },
            { op = "apply_status", status = "c04e_old_lock", to = "caster" },
          },
          on_toggle_off = { { op = "remove_status", from = "caster", status = "c04e_old_dance" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C04.txt:487-603",
            text = "和本體不同：每 0.2 秒一跳、每跳只扣 2/3/4/5 法力，固定 15/25/35/45 魔法傷害，沒有最大生命百分比、智力加成、單體加成、非英雄上限與緩速；範圍 600；切換冷卻 2 秒。" },
          { policy = "as_source", ref = "npc/C_Neutral/C04.txt:580-588",
            text = "每跳先 SpendMana 扣法力，腳本才檢查「剩餘法力 < 每跳消耗」並 ToggleAbility 關閉；所以剩餘法力介於一倍與兩倍之間時，這一跳扣了法力卻沒有傷害就關閉。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C04.lua:186-205",
            text = "選取用技能設定（ENEMY、BASIC|HERO、旗標 NONE），魔免敵人選不到；沒有 FOW 旗標，看不見的敵人也會被打。ApplyDamage 不經 AMHC。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C04.lua:206-208",
            text = "命中後套的 modifier_C04E_2（1.1 秒）只有特效、沒有屬性（KV 誤標 IsBuff），改用 cue。" },
          { policy = "note", ref = "npc/C_Neutral/C04.txt:503-524",
            text = "ApplyModifier 的 Target 只有 Center = CASTER 與旗標、沒有 Radius，依引擎行為只套到施法者本人。開啟時的 1 秒鎖定短於 2 秒冷卻，沒有額外效果，照原作保留。" },
          { policy = "as_source",
            text = "開關開著死亡時，datadriven 狀態隨死亡移除但開關仍亮著，復活後要重新開關才有效果（推測，同本體）；資料以 remove_on death 表達。冷卻在開啟時起算是推定。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C04.lua:45-52", text = "開關期間的循環音效只是呈現，改用狀態 cue。" },
        },
        provenance = { "npc/C_Neutral/C04.txt:485-606", "vscripts/heroes/C_Neutral/C04.lua:37-52", "vscripts/heroes/C_Neutral/C04.lua:177-210" },
      },
      -------------------------------------------------------------------------
      { id = "c04_old_dance_of_death", source_id = "C04R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, 14, 13, 12, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C04.txt:617" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C04.txt:618" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C04.txt:619" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C04.txt:616" },
          step = { 300, unit = "dist",
            source = "script:vscripts/heroes/C_Neutral/C04.lua:221-222 三個落點在 caster + dir × (i × 300)，i = 1..3" },
          radius = { 300, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C04.lua:227 SEARCH_RADIUS 300" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:damage@npc/C_Neutral/C04.txt:645" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:duration@npc/C_Neutral/C04.txt:650" },
        },
        text = {
          zh_TW = { name = "死亡之舞",
            body = "朝指定方向接連舞出三處落點，相隔 {step}、各半徑 {radius}，摧毀範圍內的樹木；被波及的敵方單位（不含建築）受到 {damage} 魔法傷害並暈眩 {stun} 秒，每個單位只受一次。" },
          en = { name = "Dance of Death",
            body = "Dances across three spots in the target direction, {step} apart with radius {radius} each, destroying trees there. Enemy units caught (buildings excluded) take {damage} magic damage and are stunned for {stun} seconds, once per unit." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c04r_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "死亡之舞暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Dance of Death Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（ApplyDataDrivenModifier 套用）；受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c04r_old_sound", at = "caster" },
            { op = "let", name = "dir", value = "direction(caster, point)" },
            { op = "let", name = "p1", value = "offset(caster, dir, $step)" },
            { op = "let", name = "p2", value = "offset(caster, dir, $step + $step)" },
            { op = "let", name = "p3", value = "offset(caster, dir, $step + $step + $step)" },
            { op = "cue", id = "c04r_old_burst", at = "p1" },
            { op = "cue", id = "c04r_old_burst", at = "p2" },
            { op = "cue", id = "c04r_old_burst", at = "p3" },
            { op = "destroy_trees", at = "p1", radius = "$radius" },
            { op = "destroy_trees", at = "p2", radius = "$radius" },
            { op = "destroy_trees", at = "p3", radius = "$radius" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$step + $step + $step + $radius", team = "enemy", types = BASIC,
                where = "not it.is_building and (distance(it, p1) <= $radius or distance(it, p2) <= $radius or distance(it, p3) <= $radius)" },
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$damage" },
                { op = "apply_status", status = "c04r_old_stun", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C04.lua:221-247",
            text = "三個落點依序結算：相隔 300、半徑 300，彼此重疊；以 it.b09e 標記，同一次施放每個單位只受一次傷害與暈眩。資料以三圈聯集一次選取表達（同一單位只被選一次）。選取 DOTA_UNIT_TARGET_ALL、flags NONE：魔免敵人不受影響，建築被跳過；傷害經 AMHC:Damage（阿國已死不造成傷害），先傷害再暈眩。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C04.lua:241-264",
            text = "去重旗標 it.b09e 與真田昌幸 B09E 共用欄位名；兩者都在同一個函式內設定並清除，不會互相影響。" },
          { policy = "as_source", ref = "npc/C_Neutral/C04.txt:640-652",
            text = "AbilitySpecial 兩個條目都用鍵名 \"01\"；KV 保留重複鍵，GetSpecialValueFor 依變數名稱查找，damage 與 duration 都照寫入值生效（推定，同 B09E_old）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C04.lua:218-222",
            text = "落點固定在 300／600／900，不看施放點的距離，只取方向；點在自己腳下時方向為零，三個落點都在阿國身上（邊界情況，不另外處理）。" },
          { policy = "note", ref = "npc/C_Neutral/C04.txt:627-632", text = "item_soul_adder.lua sound 只播 dispel_magic 音效，改用 cue。" },
        },
        provenance = { "npc/C_Neutral/C04.txt:607-653", "vscripts/heroes/C_Neutral/C04.lua:214-268" },
      },
      -------------------------------------------------------------------------
      { id = "c04_old_witch_dance", source_id = "C04T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 110, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C04.txt:666" },
          mana_cost = { 200, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C04.txt:667" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C04.txt:668" },
          required_level = { 6, 12, 18, unit = "level",
            source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 只寫 AbilityType ULTIMATE@C04.txt:664）" },
          shield = { 1700, 2500, 3300, unit = "dmg", source = "kv:armor@npc/C_Neutral/C04.txt:701",
            note = "RunScript SHP1 = SHP2 = %armor；目標是英雄時用 SHP2" },
          duration = { 90, unit = "s",
            source = "script:vscripts/items/Addon_Items/item_devil_supressor_armor.lua:76 ShockTarget(keys, keys.target, 90)",
            note = "KV duration 90@C04.txt:696 同值，但腳本寫死 90" },
        },
        text = {
          zh_TW = { name = "巫魅之舞",
            body = "為一名友方英雄（可以是自己，幻象無效）施加護盾，吸收敵方造成的魔法傷害，共 {shield} 點，持續 {duration} 秒。" },
          en = { name = "Bewitching Dance",
            body = "Shields an allied hero (herself included; illusions are unaffected), absorbing {shield} magic damage dealt by enemies for {duration} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "ally", types = { "hero" } },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "c04t_old_shield", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "巫魅之舞", body = "吸收敵方造成的魔法傷害，共 {shield} 點。" },
              en = { name = "Bewitching Dance", body = "Absorbs {shield} magic damage dealt by enemies." },
            },
            reveal = "public", cue = "c04t_old_shield_glow", tags = { "magic_shield" },
            absorb = { amount = "$shield", types = { "magical" } },
            notes = "原作 modifier_devil_supressor_armor（制魔腹卷的 Lua modifier）：以 original_damage 扣盾，盾量足夠時把生命設回 0.2 秒前記錄的值，破盾時溢出部分照魔抗結算並移除；重套時重設盾量。Lua modifier 沒寫 IsPurgable，暫定 none。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c04t_old_title" },
            { op = "cue", id = "c04t_old_sound", at = "caster" },
            { op = "if", cond = "not target.is_illusion", actions = {
              { op = "apply_status", status = "c04t_old_shield", to = "target" },
            } },
          },
        },
        coverage = "expressible", confidence = "low",
        quirks = {
          { policy = "fixed", ref = "vscripts/items/Addon_Items/item_devil_supressor_armor.lua:70-77,119-136",
            text = "FC-155 D22 舊版，比照 D13 修：比照 FC-062（同一個 ShockTarget，照意圖給魔法護盾）。原作：Shock_C04_old 呼叫 ShockTarget，ability:ApplyDataDrivenModifier 套的 modifier_devil_supressor_armor 是 LinkLuaModifier 的 Lua modifier，C04T_old 的 KV 沒有 Modifiers 區塊，推定套用失敗，接著 FindModifierByName(...).caster 對 nil 取值報錯，大招只剩稱號特效與音效（另外兩個 RunScript）。修法：照腳本意圖對友方英雄（不含幻象）套 90 秒魔法護盾。見 assumptions.md。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_devil_supressor_armor.lua:27-57",
            text = "護盾只擋「敵方」且非反射（DOTA_DAMAGE_FLAG_REFLECTION）的魔法傷害，以減免前的 original_damage 扣盾；盾量足夠時把生命設回 0.2 秒前記錄的值（同時抹掉這 0.2 秒內其他傷害），破盾時溢出部分乘 (1 − 魔抗) 後扣血並移除。資料以魔法 absorb 表達（產生器不支援 phase = pre_mitigation，改用預設的減免後扣盾，盾實際比原作耐打），不分敵我、也不重現生命回寫的副作用。" },
          { policy = "as_source", ref = "npc/C_Neutral/C04.txt:657-668",
            text = "和本體完全不同：本體是對敵人的引導暈眩，舊版是對友方英雄的護盾；施法前搖 0、冷卻 110、耗魔 200、射程 1000。KV AbilityUnitDamageType 與 AbilityDamage 175/270/375 沒有被使用。" },
          { policy = "note", ref = "npc/C_Neutral/C04.txt:677-689",
            text = "dummy.lua showTitle 顯示稱號粒子 c04t11，item_soul_adder.lua sound 播 jbrice_01.respawn，都只是呈現，改用 cue。" },
        },
        provenance = { "npc/C_Neutral/C04.txt:654-705", "vscripts/items/Addon_Items/item_devil_supressor_armor.lua:1-77,119-155" },
      },
    },
  }
end
