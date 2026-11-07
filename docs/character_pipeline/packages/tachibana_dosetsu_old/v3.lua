-- C07 立花道雪（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C07"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:323）；自動學習 heromap_autoskill["C07"]["11"] = ""
-- （addon_game_mode.lua:426），沒有 D，也沒有 C07T2（移動雷雲）。W/E/R/T → Q/W/E/R。
-- 舊版的 Q 是開關技能禦魔之盾（本體放在 D）。原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 C07*_old），
-- 說明依 KV 與腳本還原（招式名取自 KV 註解），見 assumptions.md。
-- C07W_old、C07E_old、C07T_old 呼叫本體的腳本函式（C07D、C07D_ATTACK、C07E、C07E_SE、C07E2_OnAbilityExecuted、C07T、C07_Effect），
-- modifier 名稱也和本體相同，所以 events.lua 依名稱的減療、ultimate_modifiers 與 dummy.lua 的 EXCLUDE 名單照樣作用。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C07_precache 屬規則層或資產，不列入。
-- 慣例照本體 C07：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C07_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C07", text = { zh_TW = { name = "立花道雪（舊版）" }, en = { name = "Tachibana Dosetsu (Old)" } } },
    unit_state = {
      -- 原作 caster.C07E_target：雷電桎梏的目標，給雷神之怒優先落雷；綁定消失時由輪詢清掉（這裡改由讀 has_status 判斷）
      c07_old_e_target = { type = "unit", reveal = "public", scope = "hero" },
      -- 原作 caster.C07T_dummy：最新一朵雷雲
      c07_old_t_storm = { type = "unit", reveal = "public", scope = "hero" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c07_old_magic_shield", source_id = "C07W_old", slot = "Q", kind = "toggle", max_level = 4, tags = { "basic", "toggle" },
        values = {
          physical_taken = { -20, -25, -30, -35, unit = "pct",
            source = "kv:reduce_incoming_physical_damge@npc/C_Neutral/C07.txt:631" },
          mana_regen = { 5, 10, 15, 20, unit = "per_s", source = "kv:regen_mp@npc/C_Neutral/C07.txt:641" },
        },
        text = {
          zh_TW = { name = "禦魔之盾",
            body = "開關：開啟時魔法免疫但無法移動，受到的物理傷害降低 {physical_taken:abs}，法力回復 +{mana_regen}。開關不會打斷移動、施法後搖與持續施法。" },
          en = { name = "Spellward Shield",
            body = "Toggle: while active, Dosetsu is spell immune but rooted, takes {physical_taken:abs} less physical damage and gains +{mana_regen} mana regeneration. Toggling does not interrupt movement, backswing or channeling." },
        },
        target = { type = "none" },
        cast = { immediate = true, ignore_backswing = true },
        statuses = {
          { id = "c07w_old_shield", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "禦魔之盾", body = "魔法免疫、無法移動，受到的物理傷害降低 {physical_taken:abs}，法力回復 +{mana_regen}。" },
              en = { name = "Spellward Shield", body = "Spell immune and rooted; physical damage taken reduced by {physical_taken:abs}; +{mana_regen} mana regeneration." },
            },
            reveal = "public", states = { "magic_immune", "root" }, cue = "c07w_old_pose_freeze", remove_on = { "death" },
            stats = { incoming_physical_pct = "$physical_taken", mana_regen = "$mana_regen" },
            triggers = {
              on_attack_windup = { { op = "cue", id = "c07w_old_attack_pose", at = "self" } },
              -- FC-110（D13）：死亡時關閉開關，復活後重開就有盾（不會開著卻沒有效果）
              on_death = { { op = "toggle_ability", of = "self", ability = "c07_old_magic_shield", on = false } },
            },
            notes = "原作 modifier_C07D（C07W_old 版）：MAGIC_IMMUNE、ROOTED、INCOMING_PHYSICAL_DAMAGE_PERCENTAGE、MANA_REGEN_CONSTANT；沒寫 IsPurgable，暫定 basic。modifier_C07D_2 的 FROZEN 只是凍住舉盾動畫，放在 cue。" },
        },
        events = {
          on_toggle_on = { { op = "apply_status", status = "c07w_old_shield", to = "caster" } },
          on_toggle_off = {
            { op = "remove_status", from = "caster", status = "c07w_old_shield" },
            { op = "cue", id = "c07w_old_pose_freeze", at = "caster", stop = true },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C07.txt:557-643",
            text = "和本體 C07D 不同：可學習四級、放在 W 鍵；狀態是定身（ROOTED）而不是移速固定 200；物理減傷 20～35%、另加法力回復；生命回復那行已註解。AbilityBehavior 有 DONT_CANCEL_MOVEMENT、IGNORE_BACKSWING、DONT_CANCEL_CHANNEL，冷卻 0、沒有耗魔。" },
          { policy = "fixed", ref = "npc/C_Neutral/C07.txt:563-581",
            text = "FC-110（照本體 D13 修正：死亡時關閉開關（on_death toggle off））。原作：modifier_C07D 不是 Permanent，推定死亡時被移除，但開關狀態仍是開啟；復活後要關再開才有盾。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C07.lua:69-95",
            text = "C07D／C07D_ATTACK 只處理手勢與 FROZEN 動畫凍結；改用 cue。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C07.txt:633-637", text = "AbilitySpecial regen_hp 沒有被使用（屬性那行已註解）。" },
        },
        provenance = { "npc/C_Neutral/C07.txt:556-644", "vscripts/heroes/C_Neutral/C07.lua:69-95" },
      },
      -------------------------------------------------------------------------
      { id = "c07_old_thunder_shackle", source_id = "C07E_old", slot = "W", kind = "channel", max_level = 4,
        tags = { "basic", "channel" },
        values = {
          cooldown = { 18, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C07.txt:657" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C07.txt:658" },
          cast_range = { 600, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C07.txt:659" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C07.txt:654" },
          channel_time = { 1, 2, 3, 4, unit = "s", source = "kv:AbilityChannelTime@npc/C_Neutral/C07.txt:660" },
          duration = { 1, 2, 3, 4, unit = "s", source = "kv:duration@npc/C_Neutral/C07.txt:783" },
          damage = { 150, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C07.txt:786" },
          tick = { 0.95, unit = "s", source = "kv:modifier_C07E ThinkInterval@npc/C_Neutral/C07.txt:743" },
        },
        text = {
          zh_TW = { name = "雷電桎梏",
            body = "以雷電連結一名敵人並持續施法：目標暈眩 {duration} 秒，期間每 {tick} 秒受到 {damage} 魔法傷害。施法結束或被打斷時暈眩解除；道雪施放其他非開關技能或使用道具（伐木與回城卷軸除外）時，連結也會斷開。" },
          en = { name = "Thunder Shackle",
            body = "Channels a lightning link to an enemy: the target is stunned for {duration} seconds and takes {damage} magic damage every {tick} seconds. The stun ends when the channel ends or is interrupted, and the link also breaks if Dosetsu casts any other non-toggle ability or uses an item (except cutting trees and Town Portal Scrolls)." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time", interruptible = true, move_while_casting = false },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c07e_old_bind", duration = "$duration", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "雷電桎梏", body = "暈眩，每 {tick} 秒受到 {damage} 魔法傷害；道雪停止施法或連結斷開時結束。" },
              en = { name = "Thunder Shackle", body = "Stunned and takes {damage} magic damage every {tick} seconds; ends when Dosetsu stops channeling or the link breaks." },
            },
            reveal = "public", status_resistance = "normal", states = { "stun" }, cue = "c07e_old_beam",
            tags = { "cleanse_protected" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", type = "magical", amount = "$damage" },
                { op = "cue", id = "c07e_old_spark", at = "self" },
              } },
            },
            notes = "原作 modifier_C07E（C07E_old 版）：IsPurgable 1、IsStunDebuff 1（只有強驅散能移除）；列在 dummy.lua EXCLUDE_MODIFIER_NAME（cleanse_protected）。" },
          { id = "c07e_old_link", duration = "infinite", stacking = "replace", dispel = "basic", visibility = "hidden",
            reveal = "public", tags = { "cleanse_protected" },
            vars = { bound = { type = "unit", reveal = "public" } },
            triggers = {
              on_ability_executed = { who = "self",
                filter = "not event.ability.is_toggle and not is_item(event.ability, item_logging) and not is_item(event.ability, item_tpscroll)",
                actions = {
                  { op = "remove_status", from = "self", status = "c07e_old_link" },
                } },
              on_removed = {
                { op = "if", cond = "exists(vars.bound)", actions = {
                  { op = "remove_status", from = "vars.bound", status = "c07e_old_bind" },
                } },
              },
            },
            notes = "原作 modifier_C07E2，掛在施法者身上；原作以 0.2 秒輪詢 timer 偵測它消失後才移除目標的暈眩。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c07e_old_vo", at = "caster" },
            { op = "apply_status", status = "c07e_old_link", to = "caster", vars = { bound = "target" } },
            { op = "apply_status", status = "c07e_old_bind", to = "target" },
            { op = "set_state", var = "c07_old_e_target", value = "target" },
          },
          on_channel_end = {
            { op = "remove_status", from = "target", status = "c07e_old_bind" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C07.txt:646-786",
            text = "和本體不同：施放瞬間沒有傷害，只有每 0.95 秒一次 150 魔法傷害（rank N 最多 N 次）；冷卻 18、前搖 0.25。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C07.lua:469-475",
            text = "斷開連結的是道雪施放的任何非開關技能或道具，item_logging、item_tpscroll 除外（禦魔之盾是開關技能，不會斷開）。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C07.txt:701-722",
            text = "OnChannelFinish／OnChannelInterrupted 呼叫的 C07E_END 在 C07.lua 不存在，每次都報錯；之前的 RemoveModifier 已執行，結果只是移除暈眩。" },
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C07.lua:22-33",
            text = "原作以 0.2 秒輪詢偵測連結斷開，暈眩最多晚 0.2 秒解除；照設計意圖改成連結斷開當下移除。" },
          { policy = "as_source", ref = "npc/C_Neutral/C07.txt:740-777",
            text = "modifier_C07E 的 datadriven Damage 不經 AMHC（道雪死亡時引導中斷、暈眩移除，不會再有傷害）。modifier_C07E2 沒寫 IsPurgable，暫定 basic。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C07.lua:47-67", text = "C07E_SE 只做手勢與粒子，改用 cue。" },
        },
        provenance = { "npc/C_Neutral/C07.txt:645-787", "vscripts/heroes/C_Neutral/C07.lua:2-67",
          "vscripts/heroes/C_Neutral/C07.lua:469-475", "vscripts/library/common/dummy.lua:376-392" },
      },
      -------------------------------------------------------------------------
      { id = "c07_old_thunderbolt", source_id = "C07R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 6, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C07.txt:798" },
          mana_cost = { 140, 160, 180, 200, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C07.txt:799" },
          cast_range = { 1100, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C07.txt:800" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C07.txt:797" },
          radius = { 500, unit = "dist", source = "kv:radius@npc/C_Neutral/C07.txt:859" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C07.txt:867" },
        },
        text = {
          zh_TW = { name = "雷霆閃電",
            body = "對目標敵人放出閃電，以目標為圓心 {radius} 內所有非魔免的敵方英雄與部隊各受到 {damage} 魔法傷害。" },
          en = { name = "Thunderbolt",
            body = "Hurls lightning at the target enemy; every non-spell-immune enemy hero and unit within {radius} of it takes {damage} magic damage." },
        },
        target = { type = "unit", range = "$cast_range", aoe_radius = "$radius", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "c07r_old_vo", at = "caster" },
            { op = "for_each",
              select = { from = "target", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
              actions = {
                { op = "cue", id = "c07r_old_bolt", at = "caster", to = "it" },
                { op = "damage", require_source_alive = false, to = "it", type = "magical", amount = "$damage" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "no_effect", ref = "npc/C_Neutral/C07.txt:861-865",
            text = "KV target_count 8 被 C07R_old 讀進變數但沒有使用：範圍內所有敵人都會被打到，沒有人數上限。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C07.lua:97-123",
            text = "以目標位置為圓心搜尋（技能設定 ENEMY、HERO|BASIC、旗標 NONE）：魔免敵人選不到；沒有 FOW 旗標，看不見的敵人也會被打。傷害透過 Duration 0 的 modifier_C07R OnCreated Damage 結算（不經 AMHC）。" },
          { policy = "as_source", ref = "npc/C_Neutral/C07.txt:789-801",
            text = "和本體不同：沒有人數上限、半徑 500、射程 1100、傷害 200/300/400/500。" },
        },
        provenance = { "npc/C_Neutral/C07.txt:788-868", "vscripts/heroes/C_Neutral/C07.lua:97-123" },
      },
      -------------------------------------------------------------------------
      { id = "c07_old_raijin_wrath", source_id = "C07T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, 120, 110, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C07.txt:879" },
          mana_cost = { 300, 400, 500, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C07.txt:880" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades@npc/C_Neutral/C07.txt:881-882" },
          duration = { 15, 20, 25, unit = "s", source = "kv:duration@npc/C_Neutral/C07.txt:980" },
          tick = { 1, 0.9, 0.8, unit = "s", source = "kv:tick@npc/C_Neutral/C07.txt:960" },
          radius = { 1400, unit = "dist", source = "kv:radius@npc/C_Neutral/C07.txt:965" },
          strike_radius = { 350, unit = "dist", kv = { 300 },
            source = "script:vscripts/heroes/C_Neutral/C07.lua:332-340,378-386 FindUnitsInRadius 350",
            source_differs = "KV attack_radius=300（C07.txt:970）沒有被讀取；腳本寫死 350" },
          damage = { 200, 270, 340, unit = "dmg", damage_type = "physical", source = "kv:AbilityDamage@npc/C_Neutral/C07.txt:983" },
          building_pct = { 50, unit = "pct", source = "script:vscripts/heroes/C_Neutral/C07.lua:347,393 dmg*0.5" },
          slow = { 35, unit = "pct", kv = { -35 }, source = "kv:slow@npc/C_Neutral/C07.txt:975",
            source_differs = "KV 以負值寫在 MOVESPEED_BONUS_PERCENTAGE；這裡以正值表示降低量，狀態寫 -$slow" },
          slow_time = { 3, unit = "s", source = "kv:modifier_C07T_2 Duration@npc/C_Neutral/C07.txt:951" },
          heal_reduction = { 50, unit = "pct", source = "script:vscripts/events.lua:352-360 decrease_health = 0.5（依名稱 modifier_C07T_2）" },
          cloud_reveal_radius = { 100, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C07.lua:185-186 AddFOWViewer 100" },
        },
        text = {
          zh_TW = { name = "雷神之怒",
            body = "在原地召喚雷雲 {duration} 秒。雷雲每 {tick} 秒對 {radius} 內所有非建築敵人施加{@c07t_old_static}（移動速度降低 {slow}、受到的治療降低 {heal_reduction}，持續 {slow_time} 秒），並落雷一次：優先打被{@c07_old_thunder_shackle}綁住的敵人，否則打最近且看得見的敵人；落雷對目標周圍 {strike_radius} 內的敵人造成 {damage} 物理傷害，建築只受 {building_pct}。施放點周圍 {radius} 的視野持續整段時間。" },
          en = { name = "Wrath of Raijin",
            body = "Summons a thundercloud at Dosetsu's position for {duration} seconds. Every {tick} seconds the cloud applies {@c07t_old_static} to all non-building enemies within {radius} ({slow} slower movement and {heal_reduction} less healing received for {slow_time} seconds) and strikes once: preferring the enemy held by {@c07_old_thunder_shackle}, otherwise the nearest visible enemy. The strike deals {damage} physical damage to enemies within {strike_radius} of its target; buildings take only {building_pct}. Grants vision within {radius} of the cast point for the whole duration." },
        },
        target = { type = "none" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune", "can_target_buildings" },
        statuses = {
          { id = "c07t_old_storm", duration = "infinite", stacking = "keep", dispel = "basic", visibility = "hidden",
            reveal = "public", states = { "invulnerable" }, cue = "c07t_old_cloud",
            triggers = {
              periodic = { interval = "$tick", first = "$tick", actions = {
                -- 原作 C07_Effect：先看 caster.C07E_target（不需可見）。FC-016（D13）：每朵雲以自己判斷距離、可見與傷害來源
                { op = "let", name = "bound", value = "state.c07_old_e_target" },
                { op = "if",
                  cond = "exists(bound) and has_status(bound, c07e_old_bind) and distance(bound, self) < $radius",
                  actions = {
                    { op = "call", routine = "c07t_old_strike",
                      args = { victim = "bound", bound = "true", cloud = "self", storm = "self" } },
                  },
                  otherwise = {
                    { op = "for_each",
                      select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = { "all" },
                        flags = { "include_magic_immune" }, order = "nearest",
                        where = "is_visible_to(it, self) and not has_tag(it, dummy)", limit = "1" },
                      actions = {
                        { op = "call", routine = "c07t_old_strike",
                          args = { victim = "it", bound = "false", cloud = "self", storm = "self" } },
                      } },
                  } },
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = { "all" },
                    flags = { "include_magic_immune" }, where = "not it.is_building" },
                  actions = { { op = "apply_status", status = "c07t_old_static", to = "it" } } },
              } },
            },
            notes = "原作 modifier_C07T，掛在雷雲 dummy 上，沒有 Duration，靠 modifier_kill 結束。" },
          { id = "c07t_old_static", duration = "$slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", status_resistance = "ignore", cue = "c07t2_old_static",
            stats = { move_speed_pct = "-$slow", heal_reduction_pct = "$heal_reduction" },
            text = {
              zh_TW = { name = "雷神靜電", body = "移動速度降低 {slow}，受到的治療降低 {heal_reduction}。" },
              en = { name = "Raijin's Static", body = "Movement speed reduced by {slow} and healing received reduced by {heal_reduction}." },
            },
            notes = "原作 modifier_C07T_2（C07T_old 版）：Properties 直接 MOVESPEED_BONUS_PERCENTAGE −35（不經 nobu 緩速池）；減療由 events.lua HealFilter 依名稱判斷、多來源取最強；名稱列在 ultimate_modifiers（events.lua:262）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c07t_old_vo", at = "caster" },
            { op = "cue", id = "c07t_old_title", at = "caster" },
            { op = "reveal", at = "caster.position", radius = "$radius", duration = "$duration", teams = "own" },
            { op = "create_unit", template = "c07t_old_storm_cloud", at = "caster.position", owner = "caster", team = "caster",
              lifetime = "$duration", control = "none", as = "storm" },
            { op = "apply_status", status = "c07t_old_storm", to = "storm" },
            { op = "set_state", var = "c07_old_t_storm", value = "storm" },
            { op = "reveal", at = "storm.position", radius = "$cloud_reveal_radius", duration = "$duration", teams = "all" },
            { op = "cue", id = "spell_hint", attach = "storm", radius = "$radius", duration = "$duration" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "no_effect", ref = "vscripts/heroes/C_Neutral/C07.lua:217-229",
            text = "C07T 在建立雷雲、視野、modifier_kill 與粒子之後執行 caster:FindAbilityByName(\"C07T2\"):SetLevel(...)；舊版沒有 C07T2，對 nil 呼叫報錯，後面的 timer（C07T2 歸零、銷毀粒子與雷雲）都沒有建立。雷雲照樣靠 modifier_kill 在 duration 秒後死亡，只有雷雲粒子殘留（呈現）；舊版也就沒有移動雷雲。" },
          { policy = "fixed", ref = "vscripts/heroes/C_Neutral/C07.lua:184,312-317",
            text = "FC-016（照本體 D13 修正：每朵雲以自己判斷綁定距離、可見與傷害來源）。原作：C07_Effect 一律讀 caster.C07T_dummy（最新一朵雲）；有兩朵雲時，舊雲以新雲位置判斷綁定目標距離、以新雲為傷害來源。" },
          { policy = "as_source", ref = "npc/C_Neutral/C07.txt:870-983",
            text = "和本體不同：沒有前搖、半徑 1400、每 1/0.9/0.8 秒一跳、持續 15/20/25 秒、物理傷害 200/270/340、冷卻 130/120/110、耗魔 300/400/500；緩速是 KV Properties 直接生效（本體走 nobu 緩速池）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C07.lua:345-361,391-396",
            text = "傷害來源不一致：道雪活著時，綁定目標分支以道雪為來源；一般分支建築以道雪為來源、非建築以雷雲為來源。道雪死後全部以雷雲為來源，雷雲照常造成傷害。" },
          { policy = "no_effect", ref = "vscripts/heroes/C_Neutral/C07.lua:353",
            text = "綁定目標分支在道雪死亡時寫成 AMHC:Damage(dummyx, v2*0.5, …)，打到建築會報錯；v3 中道雪死亡會打斷引導並立即移除綁定，這個分支到不了。" },
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C07.lua:202",
            text = "最後一跳與 modifier_kill 同在 duration 秒，原作實際跳數取決於同幀順序；照設計意圖，雲存在 duration 秒、每 tick 秒一跳。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C07.lua:366",
            text = "一般分支要求雲的隊伍看得見目標，並排除帶 majia 或名稱含 dummy 的單位；靜電則套給範圍內所有非建築敵人，包含看不見的與魔免的。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C07.lua:188-192",
            text = "建立雲時對 200 內單位套 modifier_phased 0.1 秒防卡位；由執行器放置單位時處理。" },
        },
        provenance = { "npc/C_Neutral/C07.txt:869-984", "vscripts/heroes/C_Neutral/C07.lua:170-230",
          "vscripts/heroes/C_Neutral/C07.lua:293-415", "vscripts/events.lua:246-268", "vscripts/events.lua:352-360",
          "npc/npc_units_custom_load.txt:5455-5530" },
      },
    },
    unit_templates = {
      { id = "c07t_old_storm_cloud", source_id = "Dummy_Ver1",
        values = {
          move_speed = { 150, unit = "move_speed", source = "kv:MovementSpeed@npc/npc_units_custom_load.txt:5498" },
          vision = { 0, unit = "dist", source = "kv:VisionDaytimeRange@npc/npc_units_custom_load.txt:5508" },
        },
        text = { zh_TW = { name = "雷雲" }, en = { name = "Thundercloud" } },
        stats = { move_speed = "$move_speed" },
        vision = { day = "$vision", night = "$vision" },
        ai = "none", tags = { "dummy" },
        notes = "與本體同一個 Dummy_Ver1；舊版沒有移動雷雲，雲不會移動。無敵來自 c07t_old_storm。",
        provenance = { "npc/npc_units_custom_load.txt:5455-5530" } },
    },
    routines = {
      -- 原作 C07_Effect 的落雷（C07.lua:320-362、366-408）
      c07t_old_strike = {
        params = { victim = "unit", bound = "bool", cloud = "unit", storm = "unit" },
        actions = {
          { op = "cue", id = "c07t_old_bolt", at = "cloud", to = "victim" },
          { op = "for_each",
            select = { from = "victim", shape = "circle", radius = "$c07_old_raijin_wrath.strike_radius", team = "enemy",
              types = { "all" }, flags = { "include_magic_immune" } },
            actions = {
              { op = "damage", to = "it", type = "physical",
                amount = "select(it.is_building, $c07_old_raijin_wrath.damage * $c07_old_raijin_wrath.building_pct, $c07_old_raijin_wrath.damage)",
                source = "select(alive(caster) and (bound or it.is_building), caster, storm)" },
            } },
        },
      },
    },
  }
end
