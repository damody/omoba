-- A07 本多忠勝（2.0 版，原作 *_20 技能組）— package schema v3。每個版本是一位獨立英雄（D22）；舊版另成 honda_tadakatsu_old。
-- 槽位依據：選 2.0 版時 choose_20 拔掉全部技能，依 heromap_skill A07 ["20"] = "WFERDT" 逐一 AddAbility("A07" .. 字母 .. "_20")，
--   第五個字母前插入 attribute_bonusx；heromap_autoskill A07 ["20"] = "D" 自動學 A07D_20
--   （vscripts/items/Addon_Items/tofar_goback.lua:6-48、vscripts/addon_game_mode.lua:249,352）。
-- W/E/R/T → Q/W/E/R；A07D_20（NOT_LEARNABLE、MaxLevel 1）放 D，寫在 hero.auto_learn。
-- A07F_20（暴雷擊）是 DOTA_ABILITY_BEHAVIOR_HIDDEN：出生時藏著，只能由 A07D_20 與 A07E_20 互換（A07_Honda_Tadakatsu.lua:332-344），
--   所以寫成 slot = "sub"、level_link 指向赤雷擊；兩招互相同步等級（OnUpgrade）與冷卻（A07E_20CD／A07F_20CD）。
-- KV 在 npc/A_Oda/A07.txt:1289-1958；腳本 vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua、vscripts/AON_Cleave.lua:131-163。
--   原作沒有 *_20 招式的說明文字（各語系 addon_*.txt 都沒有），招式名稱取自 KV 檔的註解，說明依 KV 與腳本還原（assumptions.md）。
-- 英雄單位數值沿用本體 A07。magical_resistance、attribute_bonusx、A07_precache 屬規則層或資產，不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 暫定 dispel = "basic"；Lua modifier 預設不可驅散，寫 "none"（v3.md §12.3，推定）。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "A07_v20",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "v20" },
    hero = { source_id = "A07", text = { zh_TW = { name = "本多忠勝（2.0）" }, en = { name = "Honda Tadakatsu (2.0)" } },
      auto_learn = { "a07_v20_switch_strike" } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a07_v20_unscathed", source_id = "A07W_20", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 13, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A07.txt:1305" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A07.txt:1308" },
          duration = { 2, 3, 4, 5, unit = "s", source = "kv:duration@npc/A_Oda/A07.txt:1386" },
          physical_cut = { 300, unit = "pct", hidden = true, kv = { -300 },
            source = "kv:modifier_a07w_borrowed_time MODIFIER_PROPERTY_INCOMING_PHYSICAL_DAMAGE_PERCENTAGE@npc/A_Oda/A07.txt:1351",
            source_differs = "KV 以負值表示減傷，v3 以正值表示；−300% 使物理傷害歸零" },
          convert = { 30, unit = "pct",
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:36-38 非物理傷害 SetHealth(hp + original_damage × 0.3)" },
        },
        text = {
          zh_TW = { name = "毫髮無傷",
            body = "持續 {duration} 秒免疫物理傷害。期間受到敵方的魔法或純粹傷害時，生命會回到受傷前，並再回復該次原始傷害的 {convert}。" },
          en = { name = "Unscathed",
            body = "For {duration} seconds Tadakatsu is immune to physical damage. Magic or pure damage from enemies during this time is undone, and he additionally recovers {convert} of that damage's original amount." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a07w_20_guard", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "a07w_20_shield",
            stats = { incoming_physical_pct = "-$physical_cut" },
            text = { zh_TW = { name = "毫髮無傷", body = "免疫物理傷害；受到的魔法與純粹傷害會被抹消並轉為回復。" },
              en = { name = "Unscathed", body = "Immune to physical damage; magic and pure damage taken is undone and turned into healing." } },
            triggers = {
              on_removed = { { op = "remove_status", from = "self", status = "a07w_20_convert" } },
            },
            notes = "原作 modifier_a07w_borrowed_time：IsBuff、INCOMING_PHYSICAL_DAMAGE_PERCENTAGE −300；OnDestroy／OnDeath 呼叫 A07W_EndShieldParticle 移除 modifier_A07W。沒寫 IsPurgable，依 §12.3 暫定 basic。2.0 版沒有舊版的 −40% 移速。" },
          { id = "a07w_20_convert", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              -- FC-025（D13，沿用本體）：結算前取消敵方的非物理傷害（致死的一擊也擋得住），再以 SetHealth 回復原始傷害的 30%
              on_take_damage_pre = {
                filter = "is_enemy(event.attacker, self) and not event.damage.is_physical",
                actions = {
                  { op = "modify_damage", cancel = true },
                  { op = "set_health", to = "self", value = "min(self.hp + event.damage.raw * $convert, self.max_hp)" },
                } },
            },
            notes = "原作 modifier_A07W（Lua，由 A07W_SE 以 ApplyDataDrivenModifier 套上，持續 = %duration）：每 0.1 秒記下生命，OnTakeDamage 對敵方的非物理傷害 SetHealth(記下的生命 + original_damage × 0.3)。" },
        },
        events = {
          on_cast = {
            { op = "call", routine = "a07w_20_cast", args = {} },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:123-130",
            text = "A07W_SE 以 ApplyDataDrivenModifier 套用 Lua modifier modifier_A07W（不在 A07W_20 的 Modifiers 裡）；與本體、舊版相同寫法，依本體的推定視為引擎接受。若不接受，FindModifierByName 回傳 nil 使腳本中斷，只剩物理免疫。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27-46",
            text = "FC-025（沿用本體 D13 修正：結算前取消敵方的非物理傷害，再以 SetHealth 回復原始傷害的 30%，致死的一擊也擋得住）。原作：回寫生命在 OnTakeDamage（受傷之後），一次就致死的魔法或純粹傷害照樣擊殺本多忠勝。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:17-25",
            text = "原作的「受傷前生命」是每 0.1 秒取樣一次的快照，取樣後到受傷前的治療會被回寫抹掉；照意圖以這次傷害之前的生命為基準（取消傷害後即為目前生命）。物理傷害的 SetHealth(記下的生命) 在物理傷害歸零下只會抹掉取樣後的治療，照意圖不寫。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:29-36",
            text = "只處理敵隊來源；回復量用 original_damage（減免前），所以帶魔抗時回寫後的生命比受傷前多。用 SetHealth，不受治療增減影響。" },
          { policy = "note", ref = "npc/A_Oda/A07.txt:1374-1403",
            text = "KV hp_threshold、duration_scepter、redirect、redirect_range_tooltip_scepter 沒有被使用。音效 A07W.vo1、Hero_Abaddon.BorrowedTime 與護盾粒子是呈現。" },
        },
        provenance = { "npc/A_Oda/A07.txt:1289-1404", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:1-46",
          "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:123-163" },
      },
      -------------------------------------------------------------------------
      { id = "a07_v20_red_thunder", source_id = "A07E_20", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 13, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A07.txt:1424" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A07.txt:1427" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/A_Oda/A07.txt:1419" },
          radius = { 525, unit = "dist", source = "kv:radius@npc/A_Oda/A07.txt:1523" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/A_Oda/A07.txt:1543" },
          slow = { 75, unit = "pct", kv = { -75 },
            source = "kv:modifier_torrent_slow_debuff_datadriven MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE@npc/A_Oda/A07.txt:1512",
            source_differs = "KV 以負值寫移速加成；v3 存減速量，行為取負" },
          slow_time = { 2, 3, 4, 5, unit = "s", source = "kv:slow_duration@npc/A_Oda/A07.txt:1533" },
        },
        text = {
          zh_TW = { name = "赤雷擊",
            body = "以巨槍重擊地面，對周圍 {radius} 內的敵方英雄與一般單位造成 {damage} 魔法傷害，並降低其移動速度 {slow}，持續 {slow_time} 秒。施放後{@a07_v20_thunder_burst}一同進入冷卻。" },
          en = { name = "Red Thunder Strike",
            body = "Slams the great spear into the ground, dealing {damage} magic damage to enemy heroes and units within {radius} and slowing their movement speed by {slow} for {slow_time} seconds. {@a07_v20_thunder_burst} goes on cooldown as well." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a07e_20_slow", duration = "$slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public",
            stats = { move_speed_pct = "-$slow" },
            text = { zh_TW = { name = "赤雷擊", body = "移動速度降低 {slow}。" },
              en = { name = "Red Thunder Strike", body = "Movement speed reduced by {slow}." } },
            notes = "原作 modifier_torrent_slow_debuff_datadriven：IsDebuff、Duration %slow_duration、MOVESPEED_BONUS_PERCENTAGE −75（引擎移速百分比，不走 dummy.lua 緩速池）。沒寫 IsPurgable，依 §12.3 暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "call", routine = "a07e_20_cast", args = {} },
            { op = "modify_cooldown", ability = "a07_v20_thunder_burst", of = "caster", mode = "start", amount = "$cooldown" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A07.txt:1463-1503",
            text = "DelayedAction 0 秒：同一幀以施法者為圓心結算；ActOnTargets 的 Flags NOT_MAGIC_IMMUNE_ALLIES 只影響友軍，敵方魔免單位照預設排除。Damage 是 datadriven 動作，不經 AMHC（require_source_alive = false）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:364-371",
            text = "A07E_20CD 以 ability:GetCooldown(-1)（本招目前等級的冷卻 13 秒）讓暴雷擊進入冷卻；冷卻結束時間相同，等於兩招共用冷卻。" },
          { policy = "no_effect", ref = "npc/A_Oda/A07.txt:1519-1541",
            text = "KV movespeed_bonus −75（modifier 直接寫死 −75，數字相同）、stun_duration 1.53 沒有被讀取；AbilityModifierSupportValue 不影響結算。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:195-203",
            text = "A07E_SE 只建 a07e 粒子；A07E.vo1、Hero_EarthShaker.Gravelmaw 音效是呈現。" },
        },
        provenance = { "npc/A_Oda/A07.txt:1406-1548", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:195-203",
          "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:346-371" },
      },
      -------------------------------------------------------------------------
      { id = "a07_v20_thunder_burst", source_id = "A07F_20", slot = "sub", kind = "active", max_level = 4, learnable = false,
        level_link = "a07_v20_red_thunder", tags = { "sub_ability" },
        values = {
          cooldown = { 13, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A07.txt:1569" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A07.txt:1572" },
          cast_range = { 200, unit = "dist", source = "kv:AbilityCastRange@npc/A_Oda/A07.txt:1575" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/A_Oda/A07.txt:1564" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:duration@npc/A_Oda/A07.txt:1652" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/A_Oda/A07.txt:1657" },
        },
        text = {
          zh_TW = { name = "暴雷擊",
            body = "以巨槍猛擊目標敵人，使其暈眩 {stun} 秒並造成 {damage} 魔法傷害。施放後{@a07_v20_red_thunder}一同進入冷卻。" },
          en = { name = "Raging Thunder Strike",
            body = "Smashes the target enemy with the great spear, stunning it for {stun} seconds and dealing {damage} magic damage. {@a07_v20_red_thunder} goes on cooldown as well." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a07f_20_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", states = { "stun" }, cue = "a07f_20_stunned_overhead",
            text = { zh_TW = { name = "暴雷擊", body = "被如雷般的攻擊打暈，暈眩 {stun} 秒。" },
              en = { name = "Raging Thunder Strike", body = "Stunned by a thunderous attack for {stun} seconds." } },
            notes = "原作 modifier_A07E_old（A07F_20 自己宣告的同名 modifier）：IsDebuff、IsStunDebuff、STUNNED；OnCreated 造成 %AbilityDamage 魔法傷害。沒寫 IsPurgable，依 §12.3 暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a07f_20_cast_sound", at = "caster" },
            { op = "apply_status", status = "a07f_20_stun", to = "target" },
            { op = "damage", to = "target", type = "magical", amount = "$damage", require_source_alive = false },
            { op = "cue", id = "a07f_20_slam", at = "target" },
            { op = "modify_cooldown", ability = "a07_v20_red_thunder", of = "caster", mode = "start", amount = "$cooldown" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A07.txt:1551-1555",
            text = "DOTA_ABILITY_BEHAVIOR_HIDDEN：出生時藏在 F 格，只能靠 A07D_20 換到赤雷擊的位置。OnUpgrade 互相把對方升到同等級（A07_Honda_Tadakatsu.lua:346-362），以 level_link 表達。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:373-380",
            text = "A07F_20CD 以本招冷卻 13 秒讓赤雷擊進入冷卻，兩招共用冷卻。" },
          { policy = "as_source", ref = "npc/A_Oda/A07.txt:1628-1645",
            text = "傷害寫在暈眩 modifier 的 OnCreated：先暈再傷害。SPELL_IMMUNITY_ENEMIES_NO：不能對魔免目標施放。modifier 名稱沿用舊版 modifier_A07E_old，資料改用唯一的 a07f_20_stun。" },
          { policy = "note", ref = "npc/A_Oda/A07.txt:1595-1620",
            text = "A07E_old.vo1、Hero_EarthShaker.Gravelmaw 音效與 A07E_SE 粒子是呈現。" },
        },
        provenance = { "npc/A_Oda/A07.txt:1550-1661", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:195-203",
          "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:355-380" },
      },
      -------------------------------------------------------------------------
      { id = "a07_v20_shattering_spear", source_id = "A07R_20", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          radius = { 250, 300, 350, 400, unit = "dist", source = "kv:radius@npc/A_Oda/A07.txt:1751",
            note = "AON_Cleave_A07_20 以 GetLevelSpecialValueFor(\"radius\") 搜尋；AOERadius 245～365 只是顯示" },
          cleave_pct = { 50, unit = "pct", source = "script:vscripts/AON_Cleave.lua:155 keys.dmg*0.5" },
          awake_pct = { 100, unit = "pct",
            source = "script:vscripts/AON_Cleave.lua:152-153 caster:HasModifier(\"modifier_A07T\") 時 AMHC:Damage(keys.dmg)" },
          crit_chance = { "3100/101", unit = "pct", kv = { 30 },
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:86,91 RandomInt(0,100) <= 30",
            source_differs = "RandomInt(0,100) ≤ 30 是 31/101，約 30.69%" },
          crit_multiplier = { 140, 160, 180, 200, unit = "pct",
            source = "kv:CleavePercent（第二個，04）@npc/A_Oda/A07.txt:1756",
            note = "A07R_old 讀 crit_persent，A07R_20 沒有這個鍵；照設計意圖取這組 140～200 的暴擊倍率（FC 見 quirks）" },
          crit_pity = { 4, unit = "count",
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:91 A07R_noncrit_count > 4 時必中" },
        },
        text = {
          zh_TW = { name = "碎擊槍",
            body = "被動：普攻命中非建築目標時，對忠勝周圍 {radius} 內的其他敵方英雄與一般單位（包含魔免）造成該次攻擊傷害 {cleave_pct} 的物理傷害；{@a07_v20_mightiest_awakening}期間改為 {awake_pct}。每次攻擊出手時有 {crit_chance} 機率暴擊，造成 {crit_multiplier} 傷害；連續 {crit_pity} 次沒有暴擊，下一次必定暴擊。" },
          en = { name = "Shattering Spear",
            body = "Passive: when an attack hits a non-building target, other enemy heroes and units within {radius} of Tadakatsu (spell-immune ones included) take physical damage equal to {cleave_pct} of that attack's damage; {awake_pct} during {@a07_v20_mightiest_awakening}. Each attack has a {crit_chance} chance to critically strike for {crit_multiplier} damage; after {crit_pity} attacks in a row without one, the next always crits." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        passive = {
          reveal = "public", visibility = "hidden",
          triggers = {
            on_attack_windup = {
              { op = "chance", p = "$crit_chance", pity_after = "$crit_pity", counter = "hero", actions = {
                { op = "modify_attack", crit_multiplier = "$crit_multiplier" },
                { op = "cue", id = "a07r_20_crit_sound", at = "event.target" },
                { op = "cue", id = "a07r_20_slam_gesture", at = "self" },
              } },
            },
            on_attack_hit = {
              filter = "not event.target.is_building",
              actions = {
                { op = "cue", id = "a07r_20_cleave", at = "event.target" },
                -- %attack_damage：這次攻擊對主目標造成的傷害（護甲後），照舊版推定為 attack.damage × 主目標護甲倍率
                { op = "let", name = "base", value = "event.attack.damage * armor_multiplier(event.target.armor)" },
                { op = "let", name = "main", value = "event.target" },
                { op = "let", name = "pct", value = "select(has_status(self, a07t_20_mightiest), $awake_pct, $cleave_pct)" },
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy",
                    types = BASIC, flags = { "include_magic_immune" }, exclude = "main" },
                  actions = {
                    { op = "damage", to = "it", type = "physical", amount = "base * pct" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:81-121",
            text = "FC-145 D22 舊版，比照 D13 修：暴擊共用舊版的 A07R_old，它讀 crit_persent，A07R_20 沒有這個鍵（KV 把 140/160/180/200 寫成第二個 CleavePercent），GetLevelSpecialValueFor 回 0，A07R_critical 回 0，原作實際沒有暴擊、只播音效與動作。修法：照設計意圖以 31/101 機率暴擊，倍率取 KV 的 140～200%。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:87-95",
            text = "FC-046（沿用本體 D13 修正：連續 4 次沒有暴擊，下一次必定暴擊）。原作：保底計數寫成 caster.A07R_noncrit_count = 0 or caster.A07R_noncrit_count，永遠先歸零，保底永不成立。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:94-100",
            text = "原作在 OnAttackStart 掛一個持續「攻擊間隔 + 0.1 秒」的 Lua 暴擊 modifier；照意圖寫成只影響這一次攻擊的 modify_attack。" },
          { policy = "as_source", ref = "vscripts/AON_Cleave.lua:131-163",
            text = "以忠勝為圓心、半徑隨等級，含魔免敵人，排除被攻擊者與 _G.EXCLUDE_TARGET_NAME（selector 預設排除 untargetable）；攻擊建築不擴散。身上有 modifier_A07T（戰國最強）時擴散為 100%，否則 50%。傷害走 AMHC。" },
          { policy = "as_source", ref = "vscripts/AON_Cleave.lua:131-163",
            text = "keys.dmg 是 %attack_damage；語意（護甲後）屬引擎推定，照舊版以 attack.damage × armor_multiplier(主目標護甲) 表達。targetArmor 與 damageReduction 計算了但沒有使用。" },
          { policy = "no_effect", ref = "npc/A_Oda/A07.txt:1689-1758",
            text = "CleaveAttack 的 CleavePercent 0 只播粒子；KV CleaveRadius、兩個 CleavePercent（50 與 140～200，後者覆寫前者）沒有被讀取。OnUpgrade A07R_Learn_Skill 找 A07D／A07D2，2.0 版沒有，什麼也不做。" },
        },
        provenance = { "npc/A_Oda/A07.txt:1663-1759", "vscripts/AON_Cleave.lua:131-163",
          "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:52-121", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:236-251" },
      },
      -------------------------------------------------------------------------
      { id = "a07_v20_mightiest_awakening", source_id = "A07T_20", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A07.txt:1781" },
          mana_cost = { 300, 400, 500, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A07.txt:1784" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6@npc/A_Oda/A07.txt:1775-1776" },
          duration = { 15, 20, 25, unit = "s", source = "kv:duration@npc/A_Oda/A07.txt:1919" },
          move_speed = { 60, unit = "move_speed", source = "kv:bonus_movement_speed@npc/A_Oda/A07.txt:1924" },
          strength = { 15, 25, 35, unit = "stat", source = "kv:str@npc/A_Oda/A07.txt:1929" },
          base_attack_time = { 0.8, unit = "s", source = "kv:modifier_A07T MODIFIER_PROPERTY_BASE_ATTACK_TIME_CONSTANT@npc/A_Oda/A07.txt:1878" },
        },
        text = {
          zh_TW = { name = "戰國最強．啟動",
            body = "立刻以目前等級免費施放{@a07_v20_unscathed}與{@a07_v20_red_thunder}（不耗法力、不影響兩招的冷卻），並解除身上敵方造成的暈眩。{duration} 秒內魔法免疫，移動速度 +{move_speed}，力量 +{strength}，基礎攻擊間隔變為 {base_attack_time} 秒；期間{@a07_v20_shattering_spear}的擴散傷害提高。" },
          en = { name = "Mightiest of the Warring States: Awaken",
            body = "Instantly casts {@a07_v20_unscathed} and {@a07_v20_red_thunder} at their current levels for free (no mana, their cooldowns are unaffected) and removes stuns applied by enemies. For {duration} seconds Tadakatsu is spell immune, gains +{move_speed} movement speed and +{strength} strength, and his base attack time becomes {base_attack_time} seconds; {@a07_v20_shattering_spear} splash damage is increased meanwhile." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a07t_20_mightiest", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", states = { "magic_immune" }, cue = "a07t_20_lightning_aura",
            stats = { move_speed = "$move_speed", str = "$strength",
              base_attack_time = { op = "set_absolute", value = "$base_attack_time" } },
            text = { zh_TW = { name = "戰國最強", body = "魔法免疫，移動速度 +{move_speed}，力量 +{strength}，基礎攻擊間隔 {base_attack_time} 秒。" },
              en = { name = "Mightiest of the Warring States", body = "Spell immune, +{move_speed} movement speed, +{strength} strength, base attack time {base_attack_time} seconds." } },
            triggers = {
              on_attack_hit = { { op = "cue", id = "a07t_20_hit", at = "event.target" } },
            },
            notes = "原作 modifier_A07T（A07T_20 自己的 Modifiers）：IsPurgable 0、IGNORE_INVULNERABLE、AllowIllusionDuplicate；MOVESPEED_BONUS_CONSTANT、BASE_ATTACK_TIME_CONSTANT 0.8、STATS_STRENGTH_BONUS %str、MAGIC_IMMUNE；OnAttackLanded 只播粒子。名稱 modifier_A07T 讓碎擊槍的擴散變為 100%。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a07t_20_cast_sound", at = "caster" },
            { op = "apply_status", status = "a07t_20_mightiest", to = "caster" },
            -- FC-146 D22：補上原作 A07T_Transform 的連發與解暈（2.0 版腳本找不到 A07W 而中斷）
            { op = "if", cond = "ability_level(caster, a07_v20_unscathed) > 0", actions = {
              { op = "call", routine = "a07w_20_cast", args = {} },
            } },
            { op = "if", cond = "ability_level(caster, a07_v20_red_thunder) > 0", actions = {
              { op = "call", routine = "a07e_20_cast", args = {} },
            } },
            { op = "remove_status", from = "caster", states = { "stun" }, team_of_source = "enemy" },
            { op = "cue", id = "a07t_20_title" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:256-265",
            text = "FC-147 D22 舊版，比照 D13 修：照 A07T_Transform 的設計意圖（同本體 a07_strongest），施放時以目前等級免費施放毫髮無傷（A07W_20）與赤雷擊（A07E_20）：原作先 EndCooldown、CastAbilityImmediately、補回法力，再以施放前的剩餘冷卻 StartCooldown，所以兩招的冷卻不受影響；並移除敵方施加的暈眩類 modifier。未學的招不施放（同本體）。換 A07D2 不適用（2.0 版的 D 是雷擊切換，沒有 A07D2）；模型放大屬呈現。原作：A07T_Transform 找 FindAbilityByName(\"A07W\")，2.0 版只有 A07W_20，回傳 nil 後呼叫 GetCooldownTime 報錯中斷，上述效果都不會發生；之後的音效與 showTitle 是另外的 RunScript，照常執行。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:265-276",
            text = "免費施放的赤雷擊照原作會經 A07E_20CD 讓暴雷擊進入冷卻；照「不影響冷卻」的意圖寫成只結算效果（routine 不含共用冷卻）。D 已換成暴雷擊時仍施放赤雷擊（原作找的是 A07E 本招）。" },
          { policy = "as_source", ref = "npc/A_Oda/A07.txt:1811-1818",
            text = "ApplyModifier 的 Target 只有 Center CASTER（沒有 Radius），只套在自己身上；沒寫 Duration，用 modifier 自己的 Duration %duration。" },
          { policy = "note", ref = "npc/A_Oda/A07.txt:1846-1909",
            text = "AttackAnimationPoint 0.2、OVERRIDE_ANIMATION_RATE 3、TRANSLATE_ACTIVITY alternate、每秒的閃電粒子與音效都是呈現。AllowIllusionDuplicate：幻象也會帶這個狀態（移植由幻象複製規則處理）。" },
        },
        provenance = { "npc/A_Oda/A07.txt:1761-1933", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:256-312" },
      },
      -------------------------------------------------------------------------
      { id = "a07_v20_switch_strike", source_id = "A07D_20", slot = "D", kind = "active", max_level = 1, learnable = false,
        tags = { "basic" },
        values = {
          required_level = { 1, unit = "level", source = "design:D 欄一級自動學會（heromap_autoskill A07 [\"20\"] = \"D\"）" },
        },
        text = {
          zh_TW = { name = "雷擊切換",
            body = "把{@a07_v20_red_thunder}與{@a07_v20_thunder_burst}互換：範圍緩速與單體暈眩擇一使用，兩招共用冷卻。" },
          en = { name = "Thunder Switch",
            body = "Swaps {@a07_v20_red_thunder} and {@a07_v20_thunder_burst}: choose between the area slow and the single-target stun; the two share a cooldown." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        requires = { hero_level = "$required_level" },
        unit_state = {
          a07_v20_burst_shown = { type = "bool", init = "false", reveal = "public", scope = "hero",
            note = "原作 caster.currentD（\"E\"／\"F\"）：目前顯示的是赤雷擊還是暴雷擊" },
        },
        events = {
          on_cast = {
            { op = "if", cond = "state.a07_v20_burst_shown",
              actions = {
                { op = "swap_abilities", on = "caster", from = "a07_v20_thunder_burst", to = "a07_v20_red_thunder", cooldown_policy = "continue" },
                { op = "set_state", var = "a07_v20_burst_shown", value = "false" },
              },
              otherwise = {
                { op = "swap_abilities", on = "caster", from = "a07_v20_red_thunder", to = "a07_v20_thunder_burst", cooldown_policy = "continue" },
                { op = "set_state", var = "a07_v20_burst_shown", value = "true" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:332-344",
            text = "SwapAbilities 交換兩招的位置與顯示，各自保留冷卻（cooldown_policy = continue）；caster.currentD 從 \"E\" 起算，存在英雄 handle 上，跨死亡保留。" },
          { policy = "as_source", ref = "npc/A_Oda/A07.txt:1935-1957",
            text = "NOT_LEARNABLE、IMMEDIATE、冷卻 0、不耗魔：可以隨時切換，不打斷動作。KV 註解寫「碎擊槍」是複製殘留；名稱「雷擊切換」為本檔自擬（assumptions.md）。" },
        },
        provenance = { "npc/A_Oda/A07.txt:1934-1957", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:332-344",
          "vscripts/addon_game_mode.lua:352" },
      },
    },
    routines = {
      a07w_20_cast = {
        owner = "a07_v20_unscathed", params = {},
        actions = {
          { op = "cue", id = "a07w_20_cast_sound", at = "caster" },
          { op = "apply_status", status = "a07w_20_guard", to = "caster" },
          { op = "apply_status", status = "a07w_20_convert", to = "caster" },
        },
      },
      a07e_20_cast = {
        owner = "a07_v20_red_thunder", params = {},
        actions = {
          { op = "cue", id = "a07e_20_cast_sound", at = "caster" },
          { op = "for_each",
            select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
            actions = {
              { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              { op = "apply_status", status = "a07e_20_slow", to = "it" },
            } },
          { op = "cue", id = "a07e_20_slam", at = "caster" },
        },
      },
    },
  }
end
