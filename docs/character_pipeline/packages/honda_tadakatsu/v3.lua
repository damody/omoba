-- 本多忠勝（原作 A07 current）— package schema v3 正式來源（單一份）。
-- 數值與說明照現行移植（M1：values、text、handler 由已上架的 kit 而來，Rust handler 依名稱讀 extras）；
-- 行為（events、passive、statuses、routines）照原作，由 census-v3（docs/plans/2026-10-28-history-nobu-full-port/census-v3/heroes/A07.lua）移入，
-- 行為裡的值名稱已改成 kit 的名稱。`coverage` 在 M5 切換前維持 `rust_handler`；a07_rouse_free 是 census 才有的子技能，照 census 的 coverage。
-- 原作槽位 W/E/R/T/D → Q/W/E/R/D（v2 action extra1 → D）。逐招出處見 acceptance.md 的原作行為對照卡，
-- 假設見 assumptions.md／source.lua。每個數值的 v2_key 是現行 v2 extras 鍵；冷卻／耗魔／前搖／需要等級取代 v2 levels。
-- 原作現行版 heromap_version["16"] = "WERDT"，autoskill "D"；_old、_20 變體（含 A07F_20）不屬於 current；
-- 英雄共通魔抗池（magical_resistance）屬規則層，A07_precache 只是資產，都不列入。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A07",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "current" },
    hero = {
      source_id = "A07",
      text = { zh_TW = { name = "本多忠勝" }, en = { name = "Honda Tadakatsu" } },
      -- eventforpichero.lua heromap_autoskill（A07 16 版為 D）；現行 v2 auto_learn = true
      -- 原作 16 版 autoskill "D"（addon_game_mode.lua:352、eventforpichero.lua:59-62）
      auto_learn = { "a07_rouse" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a07_unscathed", source_id = "A07W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        -- bot：與比賽設定 moba_role_match.lua 實機驗證過的 policy 相同
        bot = { kind = "self_engage", threat_radius = 700 },
        values = {
          cooldown = { 13, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 110, 120, 130, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 1.25, 2.25, 3.25, 4.25, unit = "s", source = "kv:duration", v2_key = "custom_duration" },
          absorb_magical = { 30, unit = "pct",
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27 非物理傷害回復 original_damage 的 30%",
            v2_key = "custom_absorb_magical_pct" },
          absorb_pure = { 30, unit = "pct",
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27 非物理傷害回復 original_damage 的 30%",
            v2_key = "custom_absorb_pure_pct" },
          physical_immunity = { 1, unit = "number", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27 每次受傷後把生命設回受傷前", v2_key = "custom_physical_immunity" },
          magical_immunity = { 1, unit = "number", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27", v2_key = "custom_magical_immunity" },
          pure_immunity = { 1, unit = "number", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27", v2_key = "custom_pure_immunity" },
          -- census 行為用（a07w_guard 的 incoming_physical_pct）；kit 以 physical_immunity 旗標表達同一件事，沒有這個數字
          physical_cut = { 300, unit = "pct", hidden = true, kv = { -300 },
            source = "kv:MODIFIER_PROPERTY_INCOMING_PHYSICAL_DAMAGE_PERCENTAGE",
            source_differs = "KV 以負值表示減傷，v3 以正值表示；−300% 使物理傷害歸零",
            note = "census 行為用；現行 Rust handler 不讀" },
        },
        text = {
          zh_TW = { name = "毫髮無傷",
            body = "持續 {duration} 秒：受到的所有傷害無效，魔法傷害的 {absorb_magical} 與純粹傷害的 {absorb_pure} 回復為生命。" },
          en = { name = "Unscathed",
            body = "For {duration} seconds, all damage taken is negated; {absorb_magical} of magic damage and {absorb_pure} of pure damage taken heals Tadakatsu instead." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a07w_guard", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", cue = "a07w_shield", stats = { incoming_physical_pct = "-$physical_cut" },
            triggers = {
              on_removed = { { op = "remove_status", from = "self", status = "a07w_convert" } },
            },
            notes = "原作 modifier_a07w_borrowed_time：IsBuff、IsHidden，INCOMING_PHYSICAL_DAMAGE_PERCENTAGE −300；OnDestroy／OnDeath 呼叫 A07W_EndShieldParticle 移除 modifier_A07W。沒寫 IsPurgable：依 §12.3 暫定 basic（推定）。" },
          { id = "a07w_convert", runtime_id = "a07_unscathed:{caster}", text = { zh_TW = { name = "毫髮無傷", body = "受到的所有傷害無效；魔法傷害的 {absorb_magical} 與純粹傷害的 {absorb_pure} 回復為生命。" }, en = { name = "Unscathed", body = "All damage taken is negated; {absorb_magical} of magic damage and {absorb_pure} of pure damage taken heals instead." } }, duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public",
            triggers = {
              -- FC-025（D13）：結算前取消敵方的非物理傷害（致死的一擊也擋得住），再回復原始傷害的 30%
              on_take_damage_pre = {
                filter = "is_enemy(event.attacker, self) and not event.damage.is_physical",
                actions = {
                  -- census 單一 $convert（30%）＝ kit 依類型分開的 absorb_magical／absorb_pure（都是 30%）
                  { op = "heal", to = "self", amount = "event.damage.raw * select(event.damage.is_magical, $absorb_magical, $absorb_pure)" },
                  { op = "modify_damage", cancel = true },
                } },
            },
            notes = "原作 modifier_A07W（Lua modifier，沒寫 IsPurgable → none）：每 0.1 秒記下目前生命，OnTakeDamage 對敵方造成的非物理傷害以 SetHealth(記下的生命 + original_damage × 0.3) 回寫並更新記錄。" },
        },
        events = {
          on_cast = {
            { op = "call", routine = "a07w_cast", args = {} },
          },
        },
        coverage = "rust_handler",
        handler = { module = "No1_unscathed", reads = { "$physical_immunity", "$magical_immunity", "$pure_immunity" },
          notes = "自身 stat buff（absolute_no_damage_*、absorb_*_as_heal），host 於傷害結算套用；學習時放 rank 標記給戰國最強。" },
        confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27",
            text = "原作 OnTakeDamage 在每次受到敵方傷害後把生命設回受傷前，非物理傷害另回復原始傷害的一部分；現行以結算後的傷害計算回復。" },
          { policy = "note", ref = "npc/A_Oda/A07.txt:167",
            text = "持續與耗魔依 history_nobu；nobugg 版本不同（持續 1/2/3/4、耗魔 100～160），不採用。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27-46",
            text = "FC-025（D13 修正：結算前取消敵方的非物理傷害再回復原始傷害的 30%，致死的一擊也擋得住）。原作：回寫生命在 OnTakeDamage（受傷之後）：一次就致死的魔法或純粹傷害照樣擊殺本多忠勝，毫髮無傷擋不住。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:17-25",
            text = "原作的「受傷前生命」是每 0.1 秒取樣一次的快照，取樣後到受傷前的治療與回復會被回寫抹掉；資料照設計意圖寫成「這次傷害之前的生命」（self.hp + event.damage.final）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:29-36",
            text = "event.unit／event.attacker 的命名對調，但判斷結果是：受傷者是本多忠勝、來源是敵隊。友方或自己造成的魔法傷害不回寫。回復量用 original_damage（減免前，含魔抗前的數字），所以帶魔抗時回寫後的生命比受傷前多。" },
          { policy = "as_source", ref = "npc/A_Oda/A07.txt:211-216",
            text = "施放時先 ApplyModifier 再呼叫 A07W_EndShieldParticle 移除舊的 modifier_A07W，接著 A07W_SE 重新掛上：重施等於兩個狀態一起刷新。被驅散時 OnDestroy 一併移除回寫狀態。" },
          { policy = "port_adaptation", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27-46",
            text = "現行 host damage_immunity 在傷害結算時把物理、魔法、純粹傷害都歸零（absolute_no_damage_*），魔法與純粹傷害以減免後的值 × 30% 化為回復（absorb_*_as_heal）；對所有來源都生效，也擋得住致死傷害。原作只處理敵方傷害、以減免前的值計算、致死傷害照樣致死。" },
          { policy = "note", ref = "npc/A_Oda/A07.txt:260-287",
            text = "KV hp_threshold 400、duration_scepter、redirect、redirect_range_tooltip_scepter 沒有被使用。" },
        },
        provenance = { "npc/A_Oda/A07.txt:167-290", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:7-55",
          "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:123",
          "npc/A_Oda/A07.txt:167-288", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:1-46",
          "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:123-162" },
      },
      -------------------------------------------------------------------------
      { id = "a07_red_thunder", source_id = "A07E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        -- bot：與比賽設定 moba_role_match.lua 實機驗證過的 policy 相同
        bot = { kind = "self_area", radius_key = "$radius", min_targets = 1 },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 525, unit = "dist", source = "kv:radius", v2_key = "shape_radius" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:damage",
            v2_key = "damage_amount" },
          slow = { 75, unit = "pct", source = "kv:ms_slow@npc/A_Oda/A07.txt:669", kv = { -75 },
            source_differs = "KV 以負數寫減速（dummy.lua Slow 的 ms_slow -75）；v3 存減速量，handler 轉成 move_speed_bonus 負值",
            v2_key = "slow_move_speed_pct" },
          slow_duration = { 2, 3, 4, 5, unit = "s", source = "kv:slow_duration", v2_key = "slow_duration" },
          unslow_delay = { 1, unit = "s", source = "kv:Delay@npc/A_Oda/A07.txt:634（第二個 DelayedAction → A07E_Unslow）",
            v2_key = "custom_self_unslow_delay" },
        },
        text = {
          zh_TW = { name = "赤雷擊",
            body = "巨槍猛擊地面，對周圍的敵方英雄與一般單位造成 {damage} 魔法傷害，並減緩 {slow} 移動速度，持續 {slow_duration} 秒；施放 {unslow_delay} 秒後解除自身的移動減速。" },
          en = { name = "Red Thunder",
            body = "Slams the great spear into the ground, dealing {damage} magic damage to nearby enemy heroes and units and slowing their movement speed by {slow} for {slow_duration} seconds; {unslow_delay} seconds after casting, removes movement slows on Tadakatsu himself." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a07e_slow", runtime_id = "generic_slow:a07_red_thunder:{caster}", text = { zh_TW = { name = "赤雷擊", body = "移動速度降低 {slow}。" }, en = { name = "Red Thunder", body = "Movement speed reduced by {slow}." } }, duration = "$slow_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow" },
            notes = "原作 modifier_torrent_slow_debuff_datadriven：IsDebuff，Properties 版本已註解；OnCreated 以 dummy.lua Slow 寫進緩速池（名稱 A07E），OnDestroy 以 ReturnSpeed 移除。沒寫 IsPurgable：依 §12.3 暫定 basic（推定）。" },
        },
        events = {
          on_cast = {
            { op = "call", routine = "a07e_cast", args = {} },
          },
        },
        coverage = "rust_handler",
        handler = { module = "No2_red_thunder",
          notes = "以施法者為中心、不含建築，依 id 排序；自身解除減速由英雄 UnitScript tick 送 self_slow_cleanse 封包。radius 也以 v2 名稱 shape_radius 給 host 指示器。" },
        confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A07.txt:704",
            text = "AbilitySpecial 有 stun_duration，但 OnSpellStart 只有傷害與減速，不暈眩。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:205",
            text = "DelayedAction 後 A07E_Unslow 清空施法者自己的 ms_slow／as_slow 表；描述沒提，但程式確實解除本多忠勝自身減速。現行只移除移動減速，不碰暈眩。" },
          { policy = "as_source", ref = "npc/A_Oda/A07.txt:599-622",
            text = "ActOnTargets 以施法者為中心，敵方 HERO | BASIC，旗標 NOT_MAGIC_IMMUNE_ALLIES 對敵人沒有作用：魔免敵人不選。Damage 是 datadriven（不經 AMHC），施法者已死仍會傷害。外層 DelayedAction Delay 0 只是同一刻結算。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:205-209",
            text = "一秒後的解緩速清空 nobu 緩速池的所有項目（移速與攻速、任何來源），不碰以 Properties 直接減移速的效果，也不碰暈眩；之後重新套用的緩速照常生效。說明沒有提到。A07E_Unslow 直接 caster.as_slow = {}、caster.ms_slow = {}，狀態本身仍在，以 clear_slow_pool 表達（v3.2）。" },
          { policy = "port_adaptation", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:205-209",
            text = "現行以英雄 UnitScript tick 在一秒後送 self_slow_cleanse 封包，只移除自己的移動減速，不清攻速緩速；目標排除建築、依 id 排序。" },
        },
        provenance = { "npc/A_Oda/A07.txt:555-718", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:195-213",
          "vscripts/library/common/dummy.lua:816",
          "npc/A_Oda/A07.txt:555-716", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:195-209",
          "vscripts/library/common/dummy.lua:816-864" },
      },
      -------------------------------------------------------------------------
      { id = "a07_shattering_spear", source_id = "A07R", slot = "E", kind = "passive", max_level = 4, tags = { "basic", "attack_modifier" },
        values = {
          crit_roll_max = { 100, unit = "count", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:81 RandomInt(0,100)", v2_key = "custom_crit_roll_max" },
          crit_roll_at_most = { 30, unit = "count", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:81 ≤ 30 暴擊", v2_key = "custom_crit_roll_at_most" },
          crit_chance = { expr = "($crit_roll_at_most + 1) / ($crit_roll_max + 1)", unit = "pct", source = "derived",
            note = "RandomInt(0, roll_max) ≤ roll_at_most 的實際機率" },
          misses = { 4, unit = "count", source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:81 連續未暴擊保底",
            v2_key = "custom_misses_before_guaranteed" },
          crit_multiplier = { 120, 140, 160, 180, unit = "pct", source = "kv:crit_persent",
            v2_key = "custom_crit_multiplier_pct" },
        },
        text = {
          zh_TW = { name = "碎擊槍",
            body = "被動：每次攻擊有 {crit_chance} 機率造成 {crit_multiplier} 傷害的暴擊；連續 {misses} 次沒有暴擊時，下一次必定暴擊。學會或升級時，奮起的等級隨碎擊槍提高。" },
          en = { name = "Shattering Spear",
            body = "Passive: each attack has a {crit_chance} chance to critically strike for {crit_multiplier} damage; after {misses} attacks in a row without a critical strike, the next one always crits. Learning or upgrading it also raises Rouse's level." },
        },
        target = { type = "none" },
        unit_state = {
          a07r_noncrit = { type = "int", init = "0", reveal = "team", scope = "hero",
            note = "原作 caster.A07R_noncrit_count：連續沒有暴擊的次數（FC-046 修正後真的累計）" },
        },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_windup = { actions = {
              -- census crit_chance "3100/101"（pct，30.693%）＝ kit expr (30 + 1) / (100 + 1)：運算式裡都讀到比例 31/101
              -- FC-046（D13）：連續 misses 次沒有暴擊，下一次必定暴擊（原作計數寫成 0 or count，永遠歸零）
              { op = "if", cond = "state.a07r_noncrit >= $misses",
                actions = {
                  { op = "modify_attack", crit_multiplier = "$crit_multiplier" },
                  { op = "cue", id = "a07r_crit", at = "event.target" },
                  { op = "set_state", var = "a07r_noncrit", value = "0" },
                },
                otherwise = {
                  { op = "chance", p = "$crit_chance",
                    actions = {
                      { op = "modify_attack", crit_multiplier = "$crit_multiplier" },
                      { op = "cue", id = "a07r_crit", at = "event.target" },
                      { op = "set_state", var = "a07r_noncrit", value = "0" },
                    },
                    otherwise = { { op = "set_state", var = "a07r_noncrit", value = "state.a07r_noncrit + 1" } } },
                } },
            } },
          },
        },
        events = {
          on_upgrade = {
            { op = "set_ability_level", of = "caster", ability = "a07_rouse", level = "ability.level + 1" },
            { op = "set_ability_active", of = "caster", ability = "a07_rouse", active = true },
          },
        },
        coverage = "rust_handler",
        handler = { module = "No3_shattering_spear", reads = { "$crit_roll_max", "$crit_roll_at_most" },
          notes = "on_learn 放 rank 標記並擲第一次；英雄 UnitScript（rust/hero_unit.rs）on_attack_hit 擲下一次。" },
        confidence = "medium",
        quirks = {
          { policy = "port_adaptation", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:81",
            text = "原作在攻擊開始時決定當次是否暴擊；現行在每次普攻命中時決定下一次普攻（PreattackCriticalStrike＋CritMultiplier buff），機率與保底序列相同。" },
          { policy = "no_effect", ref = "npc/A_Oda/A07.txt:464-484",
            text = "KV 的 CleaveRadius／CleavePercent 只出現在已註解的 CleaveAttack，不做濺射。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:236",
            text = "A07R_Learn_Skill 每次升級碎擊槍時把奮起設為碎擊槍等級 +1；現行奮起依碎擊槍 rank 標記選攻速。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:87-93",
            text = "FC-046（D13 修正：連續 4 次沒有暴擊，下一次必定暴擊（unit_state a07r_noncrit））。原作：保底計數寫成 caster.A07R_noncrit_count = 0 or caster.A07R_noncrit_count，Lua 裡永遠得 0：每次攻擊都先歸零，「超過 4 次沒暴擊必中」永不成立，實際是單純的 31/101 機率。" },
          { policy = "port_adaptation", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:81-121",
            text = "現行 handler 有保底：連續四次沒暴擊後第五次必中（照註解意圖），而且在每次普攻命中時決定下一次是否暴擊（以 PreattackCriticalStrike 交給 host）；原作在攻擊開始時決定當次，沒有保底。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:88-97",
            text = "每次攻擊開始先移除上一次的暴擊 modifier 再擲骰；IsUnselectable 判斷恆真，對建築、友軍出手也會擲骰。暴擊 modifier 持續 攻速＋0.1 秒，期間由技能發動、不經攻擊開始的攻擊也會吃到這次暴擊（極少見，資料以 modify_attack 只影響當次）。" },
          { policy = "note", ref = "npc/A_Oda/A07.txt:419-495",
            text = "KV AOERadius、CleaveRadius、CleavePercent、radius 只給已註解的 CleaveAttack 用，碎擊槍沒有濺射。caster.maximum_critical_damage 只是統計。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:236-251",
            text = "A07R_Learn_Skill 在每次升級時把 A07D（與存在時的 A07D2）設為碎擊槍等級 + 1，並 SetActivated(true)；奮起出生時 1 級（autoskill），所以奮起等級 = 碎擊槍等級 + 1。" },
        },
        provenance = { "npc/A_Oda/A07.txt:409-499", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:56-122",
          "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:236",
          "npc/A_Oda/A07.txt:409-497", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:49-121",
          "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:236-251" },
      },
      -------------------------------------------------------------------------
      { id = "a07_strongest", source_id = "A07T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        -- bot：與比賽設定 moba_role_match.lua 實機驗證過的 policy 相同
        bot = { kind = "self_engage", threat_radius = 900 },
        values = {
          cooldown = { 100, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, 200, 250, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel" },
          duration = { 8, 14, 20, unit = "s", source = "kv:duration", v2_key = "stat_buff_duration" },
          attack_speed_multiplier = { 2.25, unit = "ratio",
            source = "kv:MODIFIER_PROPERTY_BASE_ATTACK_TIME_CONSTANT", kv = { 0.8 },
            source_differs = "原作把基礎攻擊間隔設為 0.8；本遊戲沒有改 BAT 的屬性，以攻速乘數（原作 AttackRate 1.8 ÷ 0.8）表達，結果相同",
            v2_key = "stat_buff_attack_speed_multiplier" },
          strength = { 15, 25, 35, unit = "stat", source = "kv:str", v2_key = "custom_strength_bonus" },
          magic_resist = { 50, unit = "number", source = "kv:magical_resistance",
            v2_key = "stat_buff_magic_resist_bonus" },
          damage_taken = { -50, unit = "pct", source = "kv:damage_incoming_percentage",
            v2_key = "custom_damage_taken_pct" },
          status_resistance = { 50, unit = "pct", source = "kv:states_resistance",
            v2_key = "custom_status_resistance_pct" },
          slow_resistance = { 50, unit = "pct", source = "kv:unslow_percentage",
            v2_key = "custom_slow_resistance_pct" },
          cleave_radius = { 365, unit = "dist", source = "kv:radius",
            v2_key = "custom_2_cleave_radius" },
          cleave_pct = { 100, unit = "pct", source = "kv:CleavePercent",
            v2_key = "custom_2_cleave_pct" },
          aoe_radius = { expr = "$cleave_radius", unit = "dist", source = "derived",
            v2_key = "shape_radius", note = "host targeting_shape.radius_key（濺射範圍指示器）" },
          transform = { 1, unit = "number", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:256-313 A07T_Transform", v2_key = "custom_3_transform" },
          -- census 行為用：原作 BAT 常數本身。kit 只存換算後的 attack_speed_multiplier（kv 記 0.8）；
          -- 運算式不能寫未命名數字，無法由乘數換回 BAT，所以另列
          base_attack_time = { 0.8, unit = "s", hidden = true, source = "kv:MODIFIER_PROPERTY_BASE_ATTACK_TIME_CONSTANT",
            note = "census 行為用；現行 Rust handler 不讀（現行以 attack_speed_multiplier 表達同一個 KV 0.8）" },
          -- census 行為用：magic_resist_pool 在運算式裡要讀比例 0.5；kit 的 magic_resist 是 unit number（運算式讀到 50），
          -- 運算式不能寫 / 100，所以另列 pct 版本
          magic_resist_pool = { 50, unit = "pct", hidden = true, source = "kv:magical_resistance",
            note = "census 行為用；現行 Rust handler 不讀（與 magic_resist 同一個 KV 數字，改以 pct 供魔抗池 StatKey 讀比例）" },
        },
        text = {
          zh_TW = { name = "戰國最強．啟動",
            body = "立刻以目前等級免費施放毫髮無傷與赤雷擊，並解除身上的暈眩。持續 {duration} 秒：攻擊速度乘以 {attack_speed_multiplier}、力量 +{strength}、魔抗 +{magic_resist}、受到的傷害降低 {damage_taken:abs}、敵方效果持續時間縮短 {status_resistance}、減速效果減弱 {slow_resistance}；每次攻擊命中時，對自身周圍 {cleave_radius} 內其他敵方英雄與一般單位造成攻擊力 {cleave_pct} 的物理傷害；期間奮起不耗法力。" },
          en = { name = "Strongest of the Warring States",
            body = "Instantly casts Unscathed and Red Thunder at their current levels for free and removes stuns on Tadakatsu. For {duration} seconds: attack speed multiplied by {attack_speed_multiplier}, +{strength} strength, +{magic_resist} magic resistance, {damage_taken:abs} less damage taken, enemy effects last {status_resistance} shorter and slows are {slow_resistance} weaker; each landed attack deals physical damage equal to {cleave_pct} of his attack damage to other enemy heroes and units within {cleave_radius}; Rouse costs no mana." },
        },
        target = { type = "none", aoe_radius = "$aoe_radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a07t_strongest", runtime_id = "a07_strongest_r{rank}:{caster}", text = { zh_TW = { name = "戰國最強", body = "攻擊速度乘以 {attack_speed_multiplier}、力量 +{strength}、魔抗 +{magic_resist}、受到的傷害降低 {damage_taken:abs}、敵方效果持續時間縮短 {status_resistance}、減速效果減弱 {slow_resistance}；攻擊命中時對周圍 {cleave_radius} 內其他敵人造成攻擊力 {cleave_pct} 的物理傷害；奮起不耗法力。" }, en = { name = "Strongest of the Warring States", body = "Attack speed multiplied by {attack_speed_multiplier}, +{strength} strength, +{magic_resist} magic resistance, {damage_taken:abs} less damage taken, enemy effects last {status_resistance} shorter and slows are {slow_resistance} weaker; landed attacks deal physical damage equal to {cleave_pct} of his attack damage to other enemies within {cleave_radius}; Rouse costs no mana." } }, duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", cue = "a07t_eyes",
            stats = {
              base_attack_time = { op = "set_absolute", value = "$base_attack_time" },
              str = "$strength",
              incoming_damage_pct = "$damage_taken",
              magic_resist_pool = "$magic_resist_pool",
              status_resistance_pct = "$status_resistance",
              slow_resistance_pct = "$slow_resistance",
            },
            triggers = {
              on_created = {
                { op = "swap_abilities", on = "self", from = "a07_rouse", to = "a07_rouse_free", cooldown_policy = "continue" },
              },
              on_removed = {
                { op = "swap_abilities", on = "self", from = "a07_rouse_free", to = "a07_rouse", cooldown_policy = "continue" },
              },
              on_attack_hit = {
                filter = "not event.target.is_building",
                actions = {
                  { op = "let", name = "main", value = "event.target" },
                  { op = "let", name = "dealt", value = "event.attack.damage" },
                  { op = "for_each",
                    select = { from = "self", shape = "circle", radius = "$cleave_radius", team = "enemy",
                      types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" },
                      where = "not (it == main)" },
                    actions = {
                      { op = "damage", to = "it", type = "physical", amount = "dealt * $cleave_pct" },
                    } },
                  { op = "cue", id = "a07t_cleave", at = "event.target" },
                } },
            },
            notes = "原作 modifier_A07T：IsPurgable 0、IsBuff；BASE_ATTACK_TIME_CONSTANT 0.8、STATS_STRENGTH_BONUS、INCOMING_DAMAGE_PERCENTAGE；OnCreated 以 item_states_resistance.lua OnEquip 寫入 states_res（狀態抗性，取各來源最大值）與 ms_unslow（緩速抗性），以 dummy.lua Add_magical_resistance 加魔抗池；OnDestroy 反向並呼叫 A07T_OnDestory 把 A07D2 換回 A07D（等級與剩餘冷卻沿用）。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "a07t_strongest", to = "caster" },
            { op = "if", cond = "ability_level(caster, a07_unscathed) > 0", actions = {
              { op = "call", routine = "a07w_cast", args = {} },
            } },
            { op = "if", cond = "ability_level(caster, a07_red_thunder) > 0", actions = {
              { op = "call", routine = "a07e_cast", args = {} },
            } },
            { op = "remove_status", from = "caster", states = { "stun" }, team_of_source = "enemy" },
            { op = "cue", id = "a07t_title", at = "caster" },
          },
        },
        coverage = "rust_handler",
        handler = { module = "No4_strongest", reads = { "$transform" },
          notes = "自身 stat buff＋rank 標記；毫髮無傷與赤雷擊依各自 rank 標記直接結算；濺射由英雄 UnitScript（rust/hero_unit.rs）on_attack_hit。aoe_radius 由 host 以 v2 名稱 shape_radius 讀取。" },
        confidence = "medium",
        quirks = {
          { policy = "port_adaptation", ref = "npc/A_Oda/A07.txt:1064",
            text = "原作魔抗是自訂加法系統（基礎加上 magical_resistance）；現行以 MagicalResistanceBonus 套用，與英雄自身魔抗以 Dota 乘法疊加。" },
          { policy = "port_adaptation", ref = "vscripts/AON_Cleave.lua:67",
            text = "原作濺射以該次攻擊的傷害（%attack_damage）計；現行用最終攻擊力（get_final_atk，不含暴擊）。" },
          { policy = "port_adaptation", ref = "npc/A_Oda/A07.txt:92",
            text = "原作大招期間換成零法力的 A07D2；現行以施放奮起後退還本次法力表達，施放當下仍要有足夠法力。" },
          { policy = "note", ref = "npc/A_Oda/A07.txt:904",
            text = "期間攻擊前搖改變（AttackAnimationPoint）與模型放大屬呈現或沒有消費者，未實作。依 history_nobu 沒有魔法免疫（nobugg 版本有）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:256-276",
            text = "A07T_Transform 以 EndCooldown＋CastAbilityImmediately 立刻施放 A07W、A07E，再以 SetMana 退還法力、StartCooldown 還原原本的剩餘冷卻：效果等於以目前等級免費施放，冷卻不變。資料以共用 routine 表達；W、E 未學（等級 0）時原作的結果無法判定，資料不施放。CastAbilityImmediately 也會觸發其他技能或道具的施放事件，資料不重現。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:284-290",
            text = "移除身上敵方施加的 IsStunDebuff modifier；但暈眩中本來就不能施放戰國最強，這段幾乎沒有效果。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:302-310",
            text = "keys.modifier_one 沒有傳入（nil），ApplyDataDrivenModifier 沒有效果；之後的模型放大 1.3 倍也要 IsMagicImmune 才成立（魔免狀態已註解），不會發生。" },
          { policy = "as_source", ref = "vscripts/AON_Cleave.lua:67-96",
            text = "濺射以施法者為中心（不是目標），敵方 HERO + BASIC、含魔免，排除主目標與 EXCLUDE_TARGET_NAME（規則層 untargetable）；目標是建築時整段不做。傷害 = %attack_damage × CleavePercent/100 經 AMHC:Damage（物理，施法者已死不傷害）。CleaveAttack CleavePercent 0 只是特效。" },
          { policy = "as_source", ref = "vscripts/library/common/dummy.lua:1036-1070",
            text = "魔抗 +50 加進 nobu 魔抗池（基礎 30，上限 70），以技能名稱去重。狀態抗性（events.lua:402-409）縮短敵方施加在英雄身上、不在 ultimate_modifiers 名單內的所有 modifier 時間；緩速抗性在 dummy.lua Slow 寫入時削弱 ms_slow 與 as_slow，都屬規則層。" },
          { policy = "note", ref = "npc/A_Oda/A07.txt:904",
            text = "modifier 裡的 AttackAnimationPoint 0.2 不是 datadriven modifier 的有效欄位（推定沒有效果）；OVERRIDE_ANIMATION_RATE、動作替換屬呈現。KV damage 20/30/40、bonus_movement_speed 0 沒有作用，不列入。" },
          { policy = "port_adaptation", ref = "npc/A_Oda/A07.txt:988",
            text = "現行以 AttackSpeedMultiplier 2.25 表達 BAT 0.8（來源 AttackRate 1.8 時等價）；狀態抗性只縮短暈眩、定身、沉默、技能減速與技能 DoT；濺射用最終攻擊力（不含暴擊），原作是該次攻擊的傷害；攻擊前搖 0.2 與模型放大未實作。" },
          { policy = "port_adaptation", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:277-283",
            text = "現行不換技能：大招期間施放奮起後退還 70 法力，施放當下仍需要 70 法力；原作換成 0 法力的 A07D2（等級與冷卻沿用）。" },
        },
        provenance = { "npc/A_Oda/A07.txt:820-1080", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:256-325",
          "vscripts/AON_Cleave.lua:67-97", "vscripts/events.lua:409", "vscripts/library/common/dummy.lua:816",
          "npc/A_Oda/A07.txt:820-1082", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:256-324",
          "vscripts/AON_Cleave.lua:67-96", "vscripts/events.lua:383-410", "vscripts/items/Addon_Items/item_states_resistance.lua:1-48",
          "vscripts/library/common/dummy.lua:1036-1070" },
      },
      -------------------------------------------------------------------------
      { id = "a07_rouse", source_id = "A07D", slot = "D", kind = "active", max_level = 1, learnable = false, tags = { "basic" },
        values = {
          cooldown = { 5, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 70, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 1, unit = "level", source = "design:D 欄一級自動學會（eventforpichero.lua heromap_autoskill）" },
          heal_pct = { 3, unit = "pct", source = "kv:heal_percentage", kv = { 4 },
            source_differs = "腳本 A07D_HealHP（A07_Honda_Tadakatsu.lua:226）寫死 0.03 × 最大生命，不讀 heal_percentage",
            v2_key = "custom_heal_max_hp_pct" },
          duration = { 3, unit = "s", source = "kv:Duration@npc/A_Oda/A07.txt:71", v2_key = "custom_2_duration" },
          attack_speed_spear_0 = { 60, unit = "atk_speed", source = "design:atk_speed（A07.txt:81）第一級；奮起等級＝碎擊槍等級＋1",
            v2_key = "custom_2_attack_speed_spear_0" },
          attack_speed_spear_1 = { 70, unit = "atk_speed", source = "design:atk_speed（A07.txt:81）第二級",
            v2_key = "custom_2_attack_speed_spear_1" },
          attack_speed_spear_2 = { 80, unit = "atk_speed", source = "design:atk_speed（A07.txt:81）第三級",
            v2_key = "custom_2_attack_speed_spear_2" },
          attack_speed_spear_3 = { 90, unit = "atk_speed", source = "design:atk_speed（A07.txt:81）第四級",
            v2_key = "custom_2_attack_speed_spear_3" },
          attack_speed_spear_4 = { 100, unit = "atk_speed", source = "design:atk_speed（A07.txt:81）第五級",
            v2_key = "custom_2_attack_speed_spear_4" },
        },
        text = {
          zh_TW = { name = "奮起",
            body = "回復最大生命的 {heal_pct}，並提升攻擊速度 {duration} 秒：未學碎擊槍時 +{attack_speed_spear_0}，碎擊槍每一級依序為 +{attack_speed_spear_1}、+{attack_speed_spear_2}、+{attack_speed_spear_3}、+{attack_speed_spear_4}。戰國最強期間不耗法力。" },
          en = { name = "Rouse",
            body = "Restores {heal_pct} of maximum health and grants bonus attack speed for {duration} seconds: +{attack_speed_spear_0} without Shattering Spear, then +{attack_speed_spear_1}, +{attack_speed_spear_2}, +{attack_speed_spear_3} and +{attack_speed_spear_4} for each Shattering Spear level. Costs no mana during Strongest of the Warring States." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          -- census $attack_speed（五級 60/70/80/90/100，奮起等級＝碎擊槍等級＋1）＝ kit 依碎擊槍等級 0～4 選 attack_speed_spear_<n>。
          -- 運算式只允許 0、1、-1 的數字，所以門檻寫成 1 + 1 …
          { id = "a07d_rouse", runtime_id = "a07_rouse:{caster}", text = { zh_TW = { name = "奮起", body = "攻擊速度提升：未學碎擊槍時 +{attack_speed_spear_0}，碎擊槍每一級依序為 +{attack_speed_spear_1}、+{attack_speed_spear_2}、+{attack_speed_spear_3}、+{attack_speed_spear_4}。" }, en = { name = "Rouse", body = "Bonus attack speed: +{attack_speed_spear_0} without Shattering Spear, then +{attack_speed_spear_1}, +{attack_speed_spear_2}, +{attack_speed_spear_3} and +{attack_speed_spear_4} for each Shattering Spear level." } }, duration = "$duration", stacking = "independent", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "a07d_glow",
            stats = { attack_speed = "select(ability_level(caster, a07_shattering_spear) < 1, $attack_speed_spear_0, "
              .. "select(ability_level(caster, a07_shattering_spear) < 1 + 1, $attack_speed_spear_1, "
              .. "select(ability_level(caster, a07_shattering_spear) < 1 + 1 + 1, $attack_speed_spear_2, "
              .. "select(ability_level(caster, a07_shattering_spear) < 1 + 1 + 1 + 1, $attack_speed_spear_3, $attack_speed_spear_4))))" },
            notes = "原作 modifier_A07D：MODIFIER_ATTRIBUTE_MULTIPLE、IsBuff、ATTACKSPEED_BONUS_CONSTANT；A07D 與 A07D2 各自宣告同名 modifier。沒寫 IsPurgable：依 §12.3 暫定 basic（推定）。" },
        },
        events = {
          on_cast = { { op = "call", routine = "a07d_cast", args = {} } },
        },
        coverage = "rust_handler",
        handler = { module = "No5_rouse", notes = "攻速依碎擊槍 rank 標記（a07_ranks.rs）選 attack_speed_spear_<rank>；大招期間退還法力。" },
        confidence = "medium",
        quirks = {
          { policy = "port_adaptation", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:236",
            text = "原作奮起是五級技能，等級＝碎擊槍等級＋1（碎擊槍的 on_upgrade 照原作設定，a07_rouse_free 的 level_link 依賴它）；現行為一級技能，攻速依碎擊槍等級從五個數值選一個。D13（2026-10-29）：照原作結構。M5 切換成資料執行時，a07_rouse 改為 max_level 5，attack_speed 逐 rank 寫 60/70/80/90/100，刪掉 attack_speed_spear_0～4，狀態直接讀 $attack_speed；現在受 v2 max_level 1 限制，先不改。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:214-229",
            text = "回復以 SetHealth(目前生命 + 最大生命 × 0.03)，不經治療事件與治療增減，截在最大生命。說明寫 4%。" },
          { policy = "as_source", ref = "vscripts/library/events/eventforpichero.lua:59-62",
            text = "NOT_LEARNABLE；出生由 autoskill 設為 1 級，之後只由碎擊槍升級改變（見 hero.auto_learn 與 a07_shattering_spear 的 on_upgrade）。" },
        },
        provenance = { "npc/A_Oda/A07.txt:17-91", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:214-235",
          "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:236",
          "npc/A_Oda/A07.txt:17-89", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:214-251",
          "vscripts/addon_game_mode.lua:352", "vscripts/library/events/eventforpichero.lua:59-62" },
      },
      -------------------------------------------------------------------------
      -- census 才有的條目（原作 A07D2），整段照 census 搬入（含 coverage）；
      -- 現行移植沒有這招，以奮起退還法力表達（見 a07_strongest 的 port_adaptation）
      { id = "a07_rouse_free", source_id = "A07D2", slot = "sub", kind = "active", max_level = 5, learnable = false,
        level_link = "a07_rouse",
        values = {
          cooldown = { 5, unit = "s", source = "kv:AbilityCooldown" },
        },
        text = {
          zh_TW = { name = "奮起", body = "戰國最強期間取代奮起：效果相同，不耗法力。" },
          en = { name = "Rouse", body = "Replaces Rouse during Strongest of the Warring States: same effect, no mana cost." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { has_status = "a07t_strongest" },
        events = {
          on_cast = { { op = "call", routine = "a07d_cast", args = {} } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:277-283",
            text = "A07T_Transform 移除 A07D、加上 A07D2（AbilityManaCost 0），沿用等級與剩餘冷卻；modifier_A07T 結束時 A07T_OnDestory 反向換回。" },
        },
        provenance = { "npc/A_Oda/A07.txt:92-164", "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:277-283",
          "vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:314-324" },
      },
    },
    routines = {
      a07w_cast = {
        owner = "a07_unscathed", params = {},
        actions = {
          { op = "apply_status", status = "a07w_guard", to = "caster" },
          { op = "apply_status", status = "a07w_convert", to = "caster" },
          { op = "cue", id = "a07w_cast", at = "caster" },
        },
      },
      a07e_cast = {
        owner = "a07_red_thunder", params = {},
        actions = {
          { op = "for_each",
            select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
              types = { "hero", "creep", "neutral", "summon" } },
            actions = {
              { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              { op = "apply_status", status = "a07e_slow", to = "it" },
            } },
          { op = "cue", id = "a07e_slam", at = "caster" },
          { op = "delay", time = "$unslow_delay", actions = {
            { op = "clear_slow_pool", of = "caster" },
          } },
        },
      },
      a07d_cast = {
        owner = "a07_rouse", params = {},
        actions = {
          { op = "apply_status", status = "a07d_rouse", to = "caster" },
          { op = "set_health", to = "caster", value = "min(caster.hp + caster.max_hp * $heal_pct, caster.max_hp)" },
          { op = "cue", id = "a07d_cast", at = "caster" },
        },
      },
    },
  }
end
