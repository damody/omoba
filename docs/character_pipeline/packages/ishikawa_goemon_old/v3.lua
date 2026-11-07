-- C06 石川五右衛門（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C06"]["11"] = "WERDT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:322）；
-- 自動學習 heromap_autoskill["C06"]["11"] = "D"（addon_game_mode.lua:425）：C06D_old 是 NOT_LEARNABLE，出生時學一級（hero.auto_learn）。
-- W/E/R/T → Q/W/E/R，D 照本體慣例放 D。
-- 繁中說明取自 resource/addon_tchinese.txt:3380-3389（C06*_old 都有）；數字換成佔位符，與實際不符處記在 quirks。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C06_precache 屬規則層或資產，不列入。
-- 慣例照本體 C06：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- 金錢增減走 give_gold／take_gold（reliable = false），GivePlayerGold_UnReliable 的可靠金錢重複計入照本體 FC-129～131 修正。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C06_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = {
      source_id = "C06",
      text = { zh_TW = { name = "石川五右衛門（舊版）" }, en = { name = "Ishikawa Goemon (Old)" } },
      auto_learn = { "c06_old_thief_hiding" },
    },
    hooks = {
      c06e_old_murder_and_loot_hook = { kind = "ability", test = "c06e_old_murder_and_loot_hook_conformance",
        why = "需要在擊殺前讀取目標單位的金錢賞金（原作 GetGoldBounty），詞彙表沒有這個單位屬性，給錢量無法以資料表達",
        behavior = "目標不是英雄時：先讀目標的金錢賞金，給五右衛門 bounty_mult 倍的不可靠金錢，再以等於目標最大生命的純粹傷害擊殺它（擊殺本身照常給一份賞金）。目標是英雄時沒有效果。",
        reads = { "$c06_old_murder_and_loot.bounty_mult" },
        cases = {
          { id = "kills_creep_and_pays",
            given = { { kind = "target_unit", team = "enemy", types = { "creep" } } },
            when = { { kind = "cast", entry = "c06_old_murder_and_loot", target = "unit" } },
            expect = { { kind = "cast_accepted" } } },
        } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c06_old_frenzy", source_id = "C06W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C06.txt:515" },
          mana_cost = { 80, 100, 120, 140, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C06.txt:518" },
          duration = { 8, unit = "s", source = "kv:modifier_C06W_old Duration@npc/C_Neutral/C06.txt:535" },
          attack_speed = { 120, 240, 360, 480, unit = "atk_speed", source = "kv:attack_speed@npc/C_Neutral/C06.txt:567" },
          move_penalty = { 36, 32, 28, 24, unit = "pct", kv = { -36, -32, -28, -24 },
            source = "kv:move_speed@npc/C_Neutral/C06.txt:572",
            source_differs = "KV 以負值寫在 MOVESPEED_BONUS_PERCENTAGE；這裡以正值表示降低量，狀態寫 -$move_penalty" },
        },
        text = {
          zh_TW = { name = "狂氣",
            body = "立即施放（不打斷目前動作）：犧牲 {move_penalty} 移動速度，增加 {attack_speed} 攻擊速度，持續 {duration} 秒。" },
          en = { name = "Frenzy",
            body = "Cast instantly without interrupting the current action: sacrifices {move_penalty} movement speed for +{attack_speed} attack speed for {duration} seconds." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c06w_old_frenzy", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "狂氣", body = "攻擊速度 +{attack_speed}，移動速度降低 {move_penalty}。" },
              en = { name = "Frenzy", body = "+{attack_speed} attack speed, {move_penalty} less movement speed." },
            },
            reveal = "public", cue = "c06w_old_blades",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "-$move_penalty" },
            notes = "原作 modifier_C06W_old：ATTACKSPEED_BONUS_CONSTANT %attack_speed、MOVESPEED_BONUS_PERCENTAGE %move_speed；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = { { op = "apply_status", status = "c06w_old_frenzy", to = "caster" } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C06.txt:563-574",
            text = "AbilitySpecial 兩個條目都用鍵名 \"01\"；KV 保留重複鍵，%變數 依名稱查找，attack_speed 與 move_speed 都照寫入值生效（推定，見 assumptions.md）。" },
          { policy = "note", ref = "resource/addon_tchinese.txt:3383",
            text = "原作說明寫「增加 (120/240/360/480)% 攻擊速度」；KV 是 ATTACKSPEED_BONUS_CONSTANT（攻速點數），說明改寫成點數。" },
          { policy = "as_source", ref = "npc/C_Neutral/C06.txt:520-528",
            text = "和本體完全不同：沒有腳本，只由 KV 對自己套增益；本體是限次的大盜快手。KV 沒寫 AbilityCastPoint（IMMEDIATE），前搖為零。" },
        },
        provenance = { "npc/C_Neutral/C06.txt:503-576", "resource/addon_tchinese.txt:3382-3383" },
      },
      -------------------------------------------------------------------------
      { id = "c06_old_murder_and_loot", source_id = "C06E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        hook = "c06e_old_murder_and_loot_hook",
        values = {
          cooldown = { 60, 50, 40, 30, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C06.txt:594" },
          mana_cost = { 150, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C06.txt:597" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C06.txt:592" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C06.txt:591" },
          bounty_mult = { 4, unit = "ratio", source = "script:vscripts/heroes/C_Neutral/C06.lua:303 GetGoldBounty() * 4" },
        },
        text = {
          zh_TW = { name = "殺人越貨",
            body = "立刻殺死一個敵方非英雄部隊（不能對魔免單位施放），並額外獲得該部隊原本死亡掉落黃金的 {bounty_mult} 倍。" },
          en = { name = "Murder and Loot",
            body = "Instantly kills an enemy non-hero unit (cannot target spell-immune units) and grants {bounty_mult} times its normal gold bounty as extra gold." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "c06e_old_sell_sound", at = "target" },
            { op = "cue", id = "c06e_old_coins", at = "target" },
            { op = "if", cond = "not target.is_hero", actions = {
              -- 給錢量 = 目標的金錢賞金 × bounty_mult：詞彙表讀不到賞金，整招由 hook 執行（見 hooks）
              { op = "damage", to = "target", type = "pure", amount = "target.max_hp" },
            } },
          },
        },
        coverage = "needs_hook", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/C_Neutral/C06.lua:305",
            text = "FC-151 D22 舊版，比照 D13 修：C06E_old_OnSpellStart 在給錢與擊殺之前呼叫 ExecuteOrderFromTable(order)，order 是未宣告的全域變數：通常是 nil，執行期報錯，後面的給錢與擊殺都不會發生，只剩粒子與音效；若別的腳本（B18.lua:174、C13.lua:105 等）留下全域 order，則會重送那道舊指令。修法：照說明殺死目標並給四倍賞金，不送任何指令。" },
          { policy = "fixed", ref = "vscripts/amhc_library/amhc.lua:509-522",
            text = "FC-129（照本體 D13 修正：本作只有一個金錢池，給錢只加金額，原作把可靠金錢重複計入的情況不會發生）。原作：GivePlayerGold_UnReliable 以 GetGold（總額）+gold 寫回不可靠金錢。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C06.lua:302-308",
            text = "賞金在擊殺前讀取（GetGoldBounty 是單位設定的賞金），之後 AMHC 純粹傷害等於最大生命把目標打死，擊殺本身照常再給一份正常賞金，所以總共是五倍。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C06.txt:617-623",
            text = "KV level 3/3/3/4 沒有被使用（原意推測是可殺的單位等級上限）；AbilityUnitDamageType MAGICAL 也沒有被使用（腳本固定用純粹傷害）。" },
          { policy = "as_source", ref = "npc/C_Neutral/C06.txt:584-586",
            text = "目標類型 BASIC、旗標 NONE：只能點非英雄、非魔免的敵方單位，腳本另外檢查 IsHero。產生器的目標類型沒有「只限非英雄」的組合，資料寫 BASIC 清單並在行為裡跳過英雄（點英雄會花掉法力與冷卻、沒有效果），見 assumptions.md。" },
        },
        provenance = { "npc/C_Neutral/C06.txt:578-625", "vscripts/heroes/C_Neutral/C06.lua:294-310",
          "vscripts/amhc_library/amhc.lua:509-522", "resource/addon_tchinese.txt:3384-3385" },
      },
      -------------------------------------------------------------------------
      { id = "c06_old_savage_strength", source_id = "C06R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          damage_pct = { 8, 16, 24, 32, unit = "pct", source = "kv:atk_bonus@npc/C_Neutral/C06.txt:655" },
        },
        text = {
          zh_TW = { name = "野蠻之力", body = "被動：造成的傷害提高 {damage_pct}。" },
          en = { name = "Savage Strength", body = "Passive: deals {damage_pct} more damage." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { outgoing_damage_pct = "$damage_pct" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", ref = "resource/addon_tchinese.txt:3387",
            text = "原作說明寫「增加 (8/16/24/32)% 攻擊力」；KV 實際是 MODIFIER_PROPERTY_DAMAGEOUTGOING_PERCENTAGE，說明照屬性寫成造成的傷害提高。" },
          { policy = "as_source", ref = "npc/C_Neutral/C06.txt:636-646",
            text = "和本體完全不同：本體是有主動護盾的金錢被動，舊版只有這個屬性。" },
        },
        provenance = { "npc/C_Neutral/C06.txt:627-658", "resource/addon_tchinese.txt:3386-3387" },
      },
      -------------------------------------------------------------------------
      { id = "c06_old_robbery", source_id = "C06T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 200, 190, 180, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C06.txt:681" },
          mana_cost = { 500, 600, 700, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C06.txt:684" },
          cast_range = { 250, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C06.txt:687" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C06.txt:673" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades@npc/C_Neutral/C06.txt:677-678" },
          damage = { 900, 1300, 1700, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C06.txt:688",
            note = "腳本以 AMHC 魔法傷害造成（KV AbilityUnitDamageType PURE 沒有被使用）" },
          stun = { 3, 3.5, 4, unit = "s", source = "kv:stun@npc/C_Neutral/C06.txt:716" },
          gold = { 800, unit = "gold", source = "script:vscripts/heroes/C_Neutral/C06.lua:320-321" },
        },
        text = {
          zh_TW = { name = "搶劫",
            body = "對著敵方猛烈一擊（可以對魔免單位施放），造成 {damage} 魔法傷害並暈眩 {stun} 秒，趁對方昏迷期間竊走 {gold} 黃金（對方沒那麼多錢也照樣拿到），之後自動普攻目標。" },
          en = { name = "Robbery",
            body = "Strikes an enemy fiercely (can target spell-immune units), dealing {damage} magic damage and stunning it for {stun} seconds, and steals {gold} gold while it is stunned (Goemon is paid even if the target cannot afford it). Goemon then automatically attacks the target." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "c06t_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "搶劫暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Robbery Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（ApplyDataDrivenModifier 套用）；不在 ultimate_modifiers，受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c06t_old_sell_sound", at = "target" },
            { op = "cue", id = "c06t_old_coins", at = "target" },
            { op = "give_gold", to = "caster", amount = "$gold", reliable = false },
            { op = "if", cond = "target.is_hero", actions = {
              { op = "take_gold", from = "target", amount = "$gold" },
            } },
            { op = "issue_order", who = "caster", order = "attack", target = "target" },
            { op = "apply_status", status = "c06t_old_stun", to = "target" },
            { op = "damage", to = "target", type = "magical", amount = "$damage" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/amhc_library/amhc.lua:509-522",
            text = "FC-131（照本體 D13 修正：本作只有一個金錢池，給錢、扣錢只加減金額（取整數、扣不成負））。原作：給 +800、扣 −800 都走 GivePlayerGold_UnReliable，以 GetGold（總額）寫回不可靠金錢；對方不足 800 時 SetGold 推定被夾在零。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C06.lua:320-321",
            text = "五右衛門固定拿到 800，不看對方實際被扣多少；扣錢對象是 target:GetPlayerOwnerID()，對小兵、野怪是無效玩家，不扣錢但五右衛門照拿。資料以 target.is_hero 判斷是否扣錢（召喚物的擁有者照原作也會被扣，未另外表達）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C06.lua:327-328",
            text = "先暈眩再 AMHC 魔法傷害：魔免目標照樣被暈眩、拿錢，但魔法傷害由規則層歸零。原作說明只寫「造成傷害」，KV 寫 PURE，以腳本的魔法傷害為準。" },
          { policy = "as_source", ref = "npc/C_Neutral/C06.txt:665-669",
            text = "和本體不同：沒有金袋、強化印記；直接拿錢、暈眩與傷害。目標旗標 MAGIC_IMMUNE_ENEMIES，可以點魔免敵人；類型含 BASIC，也能對小兵施放。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C06.lua:316-319", text = "粒子與 General.Sell 音效只是呈現，改用 cue。" },
        },
        provenance = { "npc/C_Neutral/C06.txt:660-720", "vscripts/heroes/C_Neutral/C06.lua:312-329",
          "vscripts/amhc_library/amhc.lua:509-522", "resource/addon_tchinese.txt:3388-3389" },
      },
      -------------------------------------------------------------------------
      { id = "c06_old_thief_hiding", source_id = "C06D_old", slot = "D", kind = "active", max_level = 1, learnable = false,
        tags = { "basic" },
        values = {
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C06.txt:414" },
        },
        text = {
          zh_TW = { name = "盜賊藏匿術",
            body = "只能在夜間施放：停下動作並影化隱形，同時取消與單位的碰撞、降低被攻擊的優先度；下達任何指令就會現身。" },
          en = { name = "Thief's Hiding",
            body = "Night only: stops and melds into the shadows, becoming invisible with no unit collision and low attack priority; issuing any order reveals him." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        requires = { condition = "game.is_night" },
        statuses = {
          { id = "c06d_old_hide", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "盜賊藏匿術", body = "隱形，沒有單位碰撞、被攻擊優先度低；下達任何指令就會現身。" },
              en = { name = "Thief's Hiding", body = "Invisible, no unit collision and low attack priority; any order ends it." },
            },
            reveal = "public", cue = "c06d_old_vendetta",
            states = { "invisible", "no_unit_collision", "low_attack_priority" },
            remove_on = { "attack", "cast", "move" },
            notes = "原作 modifier_C06D_old（NO_UNIT_COLLISION、LOW_ATTACK_PRIORITY、INVISIBILITY_LEVEL 0）加引擎 modifier_invisible，OnOrder 兩個一起移除；沒寫 IsPurgable，KV modifier 暫定 none（引擎隱形不可驅散，合併後取 none）。" },
        },
        events = {
          on_cast = {
            { op = "issue_order", who = "caster", order = "stop" },
            { op = "apply_status", status = "c06d_old_hide", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C06.lua:277-283",
            text = "OnAbilityPhaseStart 在白天（或身上有 modifier_C06D_old_check）時 Interrupt 打斷施法；資料以 requires.condition = 夜晚表達（白天按了不會進入前搖）。modifier_C06D_old_check 沒有任何地方套用，條件只剩白天。" },
          { policy = "as_source", ref = "npc/C_Neutral/C06.txt:452-464",
            text = "OnOrder 移除 modifier_C06D_old 與 modifier_invisible：任何玩家指令（移動、攻擊、施法、停止）都會現身；沒有時間限制，天亮也不會自動解除。產生器不支援狀態的 on_order 觸發，改用 remove_on = attack／cast／move 近似：停止、原地待命指令不會解除，而沒有指令的自動攻擊會解除（原作推定不會），見 assumptions.md。" },
          { policy = "as_source", ref = "npc/C_Neutral/C06.txt:403-416",
            text = "NOT_LEARNABLE、MaxLevel 1、冷卻與耗魔都是 0；heromap_autoskill 讓它出生就是一級。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C06.txt:488-498",
            text = "AbilitySpecial A13D_Damage、A13D_movement_speed 是從服部半藏抄來的，沒有被使用。" },
          { policy = "note", ref = "resource/addon_tchinese.txt:3381",
            text = "原作說明「阻止部隊自動追尋敵人」指施放時 caster:Stop()。" },
        },
        provenance = { "npc/C_Neutral/C06.txt:400-500", "vscripts/heroes/C_Neutral/C06.lua:277-292", "resource/addon_tchinese.txt:3380-3381" },
      },
    },
  }
end
