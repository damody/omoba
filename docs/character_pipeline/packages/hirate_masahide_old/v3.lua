-- A10 平手政秀（舊版，原作 11 版「_old」技能組）— package schema v3。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 _G.heromap_skill.A10["11"] = "WERT" 逐一 AddAbility(A10<鍵>_old)
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-131，vscripts/addon_game_mode.lua:252）；
-- heromap_autoskill.A10["11"] = ""（vscripts/addon_game_mode.lua:355），不自動學技能。舊版沒有 D（A10D 只在 16 版）。
-- 原作槽位 W/E/R/T → Q/W/E/R。繁中說明在 resource/addon_tchinese.txt:3480-3487。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A10_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（v3.md §12.3，推定）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A10_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A10", text = { zh_TW = { name = "平手政秀（舊版）" }, en = { name = "Hirate Masahide (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a10_old_swap", source_id = "A10W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 26, 23, 20, 17, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 110, 120, 130, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 600, 700, 800, 900, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 50, 100, 150, 200, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          stun = { 1, 1.5, 2, 2.5, unit = "s", source = "kv:stun_duration" },
          swap_delay = { 0.05, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A10.lua:425 Timers 0.05" },
        },
        text = {
          zh_TW = { name = "移型換位",
            body = "與目標單位（敵我皆可，不能是自己）交換位置。目標是敵人時先暈眩 {stun} 秒並受到 {damage} 魔法傷害；若因此死亡就不交換。" },
          en = { name = "Swap",
            body = "Swaps positions with the target unit (ally or enemy, not himself). An enemy is first stunned for {stun} seconds and takes {damage} magic damage; if that kills it, no swap happens." },
        },
        target = { type = "unit", range = "$cast_range", team = "both", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "no_target_self" },
        statuses = {
          { id = "a10w_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" }, cue = "a10w_old_stunned_overhead",
            text = { zh_TW = { name = "暈頭轉向", body = "暈眩 {stun} 秒。" }, en = { name = "Dazed", body = "Stunned for {stun} seconds." } },
            notes = "原作以 AddNewModifier 套引擎內建的 modifier_stunned（只能被強驅散）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a10w_old_fade_bolt", at = "caster", to = "target" },
            { op = "if", cond = "is_enemy(target, caster)", actions = {
              { op = "apply_status", status = "a10w_old_stun", to = "target" },
              { op = "damage", to = "target", type = "magical", amount = "$damage", require_source_alive = false },
            } },
            { op = "let", name = "other", value = "target" },
            { op = "delay", time = "$swap_delay", capture = { "other" }, actions = {
              { op = "if", cond = "alive(other)", actions = {
                { op = "swap_positions", a = "caster", b = "other" },
                { op = "cue", id = "a10w_old_swap_sound", at = "caster" },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A10.lua:280-286",
            text = "OnAbilityPhaseStart 若目標在 _G.EXCLUDE_TARGET_NAME 或是自己就 Interrupt；資料以 no_target_self 與規則層的 untargetable_by_spells 表達（不扣魔、不進冷卻，等效）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A10.lua:415-461",
            text = "暈眩在施放時直接套上；傷害在 modifier_A10W_old（0.1 秒）OnCreated 打出；交換在 0.05 秒後，兩邊都 FindClearSpace。目標死亡時 modifier 移除並取消交換計時器，所以擊殺時不交換。友方目標只交換、不暈不傷。" },
          { policy = "note", ref = "npc/A_Oda/A10.txt:619-623",
            text = "KV max_range 600 沒有使用。SpellImmunityType 沒寫，對魔免敵人能否施放依引擎預設（selector 預設排除魔免）。" },
        },
        provenance = { "npc/A_Oda/A10.txt:542-632", "vscripts/heroes/A_Oda/A10.lua:280-286", "vscripts/heroes/A_Oda/A10.lua:415-461" },
      },
      -------------------------------------------------------------------------
      { id = "a10_old_flying_stones", source_id = "A10E_old", slot = "W", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 15, 14, 13, 12, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          channel_time = { 2.5, unit = "s", source = "kv:AbilityChannelTime" },
          per_sec = { 6, 7, 8, 9, unit = "count", source = "kv:pertime_mount" },
          damage = { 90, 160, 230, 300, unit = "dmg", damage_type = "physical", source = "kv:damage",
            note = "KV AbilityUnitDamageType 寫 PURE；腳本實際以物理傷害結算" },
          width = { 500, unit = "dist", source = "kv:radius（生成線的半寬）" },
          back_mult = { 2, unit = "ratio", hidden = true, source = "script:vscripts/heroes/A_Oda/A10.lua:484 radius * 2 * backwardVec" },
          right_angle = { 90, unit = "deg", hidden = true, source = "script:vscripts/heroes/A_Oda/A10.lua:481 rightVec = (f.y, −f.x)（垂直方向）" },
          collision_radius = { 200, unit = "dist", source = "kv:collision_radius" },
          splash_radius = { 200, unit = "dist", source = "kv:splash_radius" },
          speed = { 700, unit = "speed", hidden = true, source = "kv:speed" },
          distance = { 2000, unit = "dist", hidden = true, source = "kv:distance" },
        },
        text = {
          zh_TW = { name = "飛石擊",
            body = "持續施法，每秒召喚出 {per_sec} 個飛石，從政秀後方一條寬 {width} 的線上隨機位置朝目標方向飛出。飛石（碰撞半徑 {collision_radius}）碰到第一個敵方英雄或部隊（含魔免）時碎裂，對周圍 {splash_radius} 內的敵人造成 {damage} 物理傷害。施法中斷即停止。" },
          en = { name = "Flying Stones",
            body = "Channels to summon {per_sec} flying stones per second, launched from random spots along a line of half-width {width} behind Masahide toward the target direction. A stone (collision radius {collision_radius}) shatters on the first enemy hero or unit it touches (spell-immune included), dealing {damage} physical damage to enemies within {splash_radius}. Stops if the channel is interrupted." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "a10e_old_cast_sound", at = "caster" },
            { op = "let", name = "fwd", value = "select(distance(caster, point) > 0, direction(caster, point), caster.forward)" },
            { op = "let", name = "middle", value = "offset(caster.position, fwd, -1 * $width * $back_mult)" },
            { op = "let", name = "side", value = "rotate(fwd, $right_angle)" },
            { op = "repeat", interval = "1 / $per_sec", first = "0", ["while"] = "caster.is_channeling", actions = {
              { op = "let", name = "spawn", value = "offset(middle, side, random_int(-$width, $width))" },
              { op = "projectile", kind = "linear", from = "spawn", direction = "fwd", speed = "$speed",
                distance = "$distance", radius = "$collision_radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" }, max_hits = "1",
                cue = "a10e_old_stone",
                on_hit = {
                  { op = "for_each",
                    select = { from = "it", shape = "circle", radius = "$splash_radius", team = "enemy",
                      types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                    actions = {
                      { op = "damage", to = "it", type = "physical", amount = "$damage" },
                    } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A10.lua:463-531",
            text = "生成線中心在施放位置後方兩倍 radius（一千）處，橫向 RandomInt(−500, 500)，飛石飛 2000，所以會掃過施放位置前方一千距離；起點、方向在施放時固定，之後政秀轉向不影響。計時器第一顆立即發射，之後每 1/pertime 秒一顆，發射後才檢查是否仍在引導。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A10.lua:533-549",
            text = "傷害一律物理（魔免分支與一般分支相同），範圍含魔免敵人，圓心是被碰到的單位。AMHC:Damage 要求政秀存活：已發射的飛石在政秀死亡後不再造成傷害（規則層 require_source_alive）。" },
          { policy = "as_source", ref = "npc/A_Oda/A10.txt:663-671",
            text = "DeleteOnHit 1：每顆飛石碰到第一個符合條件的單位就消失。施法距離 1000 只決定方向。說明寫「最多可持續 2.5 秒」即引導時間。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A10.lua:471-473,494-505",
            text = "mount_persec、duration 讀不存在的 special（零），沒有使用；五種飛石粒子隨機挑一種，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A10.txt:634-720", "vscripts/heroes/A_Oda/A10.lua:463-557" },
      },
      -------------------------------------------------------------------------
      { id = "a10_old_bad_breath", source_id = "A10R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          radius = { 450, unit = "dist", source = "kv:radius" },
          death_radius = { 1000, unit = "dist", source = "kv:radius2" },
          duration = { 35, 40, 45, 50, unit = "s", source = "kv:duration" },
          regen_loss = { 40, 60, 80, 100, unit = "per_s", kv = { -40, -60, -80, -100 }, source = "kv:damage2（MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT）",
            source_differs = "KV 以負的生命回復寫；v3 存每秒損失量，行為取負。KV damage 40/60/80/100（說明用的數字）與它相同，只用於已註解的 ApplyDamage" },
          think = { 0.5, unit = "s", hidden = true, source = "kv:modifier_A10R_old_passive ThinkInterval" },
          check = { 1, unit = "s", hidden = true, source = "kv:modifier_A10R_old ThinkInterval" },
          min_hp = { 1, unit = "hp", hidden = true, source = "script:vscripts/heroes/A_Oda/A10.lua:601 GetHealth() == 1 時移除" },
        },
        text = {
          zh_TW = { name = "口臭",
            body = "被動：令附近 {radius} 內的敵方英雄與部隊受到感染，生命回復每秒降低 {regen_loss}，持續 {duration} 秒；生命被扣到只剩一點時感染解除，所以不會致命。平手死亡時，附近 {death_radius} 內的敵方英雄與部隊也會受到感染。" },
          en = { name = "Bad Breath",
            body = "Passive: enemy heroes and units within {radius} become infected, losing {regen_loss} health regeneration per second for {duration} seconds; the infection ends once they are down to a single hit point, so it is never lethal. When Hirate dies, enemy heroes and units within {death_radius} are infected as well." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        passive = {
          reveal = "public", visibility = "buff",
          triggers = {
            periodic = { interval = "$think", actions = {
              -- 死亡時的感染由隱藏的 a10r_old_last_breath 狀態觸發（產生器不支援被動的 on_owner_death）
              { op = "if", cond = "not has_status(self, a10r_old_last_breath)", actions = {
                { op = "apply_status", status = "a10r_old_last_breath", to = "self" },
              } },
              { op = "for_each",
                select = { from = "self", shape = "circle", radius = "$radius", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon" } },
                actions = { { op = "apply_status", status = "a10r_old_infection", to = "it" } } },
            } },
          },
        },
        statuses = {
          { id = "a10r_old_last_breath", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_death = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$death_radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon" } },
                  actions = { { op = "apply_status", status = "a10r_old_infection", to = "it" } } },
              },
            },
            notes = "代表原作 modifier_A10R_old_passive 的 OnDeath：被動每 0.5 秒確認自己身上有這個隱藏狀態，死亡時在屍體位置感染 1000 內的敵人。" },
          { id = "a10r_old_infection", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "a10r_old_shadow_word",
            stats = { hp_regen = "-$regen_loss" },
            text = { zh_TW = { name = "口臭", body = "生命回復每秒降低 {regen_loss}；只剩一點生命時解除。" },
              en = { name = "Bad Breath", body = "Health regeneration reduced by {regen_loss} per second; ends at a single hit point." } },
            triggers = {
              periodic = { interval = "$check", actions = {
                { op = "if", cond = "self.hp <= $min_hp", actions = {
                  { op = "remove_status", from = "self", status = "a10r_old_infection" },
                } },
              } },
            },
            notes = "原作 modifier_A10R_old：IsDebuff、IsPurgable 1、HEALTH_REGEN_CONSTANT %damage2；每秒檢查生命是否剛好為 1。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A10.txt:745-766",
            text = "效果是負的生命回復，不是傷害：不受魔抗影響、不觸發受傷事件。Dota 負回復不會把生命扣到零以下（停在一點左右），推定 host 規則層相同；若 host 的負回復會致死，需另行處理。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A10.lua:573-585",
            text = "每 0.5 秒對 450 內的敵人重新套用並刷新持續時間（35～50 秒），所以只要靠近一下就長時間感染。FindUnitsInRadius 旗標 0：排除魔免，沒有 FOW 篩選。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A10.lua:587-604",
            text = "原作以 GetHealth() == 1 判斷；資料寫成 ≤ 1（生命是整數時相同）。被感染者生命回復高於損失量時不會掉血。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A10.lua:559-571",
            text = "死亡時以屍體位置為圓心、半徑 1000 感染。被動 modifier 推定死亡中仍保留（Passive），其 0.5 秒計時在死亡期間是否繼續依引擎行為，未驗證。" },
          { policy = "design_intent", ref = "npc/A_Oda/A10.txt:745-756",
            text = "原作 OnDeath 寫在被動 modifier 上；v3_rust 目前不產生被動的 on_death／on_owner_death，改由被動定期補上的隱藏狀態 a10r_old_last_breath 的 on_death 結算。學會後第一個 0.5 秒內死亡不會觸發（原作會）。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A10.lua:591-599",
            text = "每秒的 non-lethal ApplyDamage 已註解，KV damage 只剩說明用途。" },
        },
        provenance = { "npc/A_Oda/A10.txt:721-813", "vscripts/heroes/A_Oda/A10.lua:559-604" },
      },
      -------------------------------------------------------------------------
      { id = "a10_old_feint", source_id = "A10T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate", "movement" },
        values = {
          cooldown = { 150, 120, 90, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 300, 350, 400, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 1.5, 1, 0.5, unit = "s", source = "kv:AbilityCastPoint" },
          cast_range = { 10000, unit = "dist",
            source = "design:global_range AbilityCastRange 0（說明寫全地圖），取 content 上限涵蓋整張地圖" },
          required_level = { 6, 12, 18, unit = "level", source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          camera_delay = { 0.2, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A10.lua:622 Timers 0.2 後移動鏡頭" },
        },
        text = {
          zh_TW = { name = "聲東擊西",
            body = "指定全地圖的一位友方英雄（不能是自己），施法完成後將其傳送到政秀身旁。" },
          en = { name = "Feint",
            body = "Targets one allied hero anywhere on the map (not himself); when the cast completes, that hero is teleported to Masahide's side." },
        },
        target = { type = "unit", range = "$cast_range", team = "ally", types = { "hero" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "no_target_self" },
        events = {
          on_cast = {
            { op = "cue", id = "a10t_old_blink_start", at = "target" },
            { op = "teleport", who = "target", to = "caster.position" },
            { op = "cue", id = "a10t_old_blink_end", at = "target" },
            { op = "let", name = "ally", value = "target" },
            { op = "delay", time = "$camera_delay", capture = { "ally" }, actions = {
              { op = "cue", id = "a10t_old_camera_focus", at = "ally", team = "own" },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A10.lua:606-612",
            text = "OnAbilityPhaseStart 若目標是自己或在 EXCLUDE_TARGET_NAME 就 Interrupt；資料以 no_target_self 與規則層 untargetable_by_spells 表達。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A10.lua:614-626",
            text = "原作以 FindClearSpaceForUnit 放到政秀身旁；產生器目前只支援施法者的 find_clear_space，這裡用預設的 teleport（落點的卡位處理交給 host）。FindClearSpaceForUnit 不會打斷目標正在做的事（引導、施法照常），modifier_phased 0.01 秒只是避免卡位。0.2 秒後把目標玩家的鏡頭移到目標身上（呈現）。" },
          { policy = "as_source", ref = "npc/A_Oda/A10.txt:826",
            text = "AbilityCastRange 0：Dota 對單位目標視為不限距離，說明寫全地圖；施法前搖 1.5/1/0.5 秒中可被打斷。" },
        },
        provenance = { "npc/A_Oda/A10.txt:814-857", "vscripts/heroes/A_Oda/A10.lua:606-627" },
      },
    },
  }
end
