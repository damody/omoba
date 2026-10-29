-- 道具批次 I2 — package schema v3。這是正式來源（docs/character_pipeline/items/，取代 shelf_a／shelf_b）。
-- census-v3 原標題：M4 全量普查道具批次 I2（25 件）— package schema v3。
-- 已上架的 10 件：數值（名稱、數字、v2_key）與說明照現行移植（M1，shelf 版本一字不改）；
-- 行為（active.events、passive.triggers／aura、statuses、unit_state…）照原作（census-v3 移入），並改用 shelf 的值名稱；
-- coverage 在 M5 切換前維持 rust_handler，玩法仍由 templates/source_items.lua 經 templates/moba_items.lua 編成 Rust 道具資料。
-- 其餘道具是 census-v3 普查結果，只進目錄（catalog-only）。
-- 每個上架數值的 v2_key 是道具列裡的欄位路徑，scripts/gen_item_v3.lua 檢查數字相同。
-- 來源：C:\nobu\history_nobu（commit f966720），路徑相對於 game/dota_addons/nobu/scripts/。
-- 慣例（沿用 pilot_a／pilot_b／I0／I1）：
--   * 價格一律用「零件＋合成費」（零件遞迴展開到可直購的道具，即 Dota 實際售價）；與 KV ItemCost 不同時寫 kv + source_differs。
--   * 原作 HERO | BASIC 一律寫成 types = { "hero", "creep", "neutral", "summon" }。
--   * datadriven modifier 沒寫 IsPurgable 時暫定 dispel = "basic"（v3.md §12.3）；Passive 的常駐 modifier 寫 none。
--   * datadriven 重套同名 modifier 是刷新，不會再跑 OnCreated（推定，與 I0 一致）。
--   * 非 MULTIPLE 的 Passive modifier 帶觸發時，以單一實例狀態表達（與 I1 黑雲、pilot 忍者裝束相同做法）。
--   * 原作 nobu 魔抗池（unit.magical_resistance）用 v3.1 StatKey magic_resist_pool；「同名道具只算一次」以
--     scope = item_instance 的狀態加 unique_group 表達（與 I0、I3、I4 相同）。
--   * item_magic_ring、item_poisonous_ring 只出現在商店清單，KV 沒有定義：coverage = deferred（not_in_source，v3.1），不移植。
local KV = "npc/npc_items_custom_load.txt"
local SCR = "vscripts/items/Addon_Items/"
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  -- shelf（M1 上架）條目用的 helper
  local function at(line) return "@" .. KV .. ":" .. line end
  local function handler(reads) return { module = "moba_items", reads = reads or {} } end
  local RECIPE_TOTAL = "合成道具的售價是零件加合成卷軸的合計（Dota 商店實際售價），道具列照此計算；KV 本身的 ItemCost 沒有被使用"
  -- 鞋類與般若湯共用的魔免主動（item_perceive_wine.lua#Shock）
  local IMMUNITY_ZH = "主動：自身獲得魔法免疫 {immunity} 秒，並以強驅散移除敵方施加的負面效果。與般若湯及其他鞋類共用冷卻。"
  local IMMUNITY_EN = "Active: gain spell immunity for {immunity} seconds and apply a strong dispel that removes negative effects enemies applied to you. Shares its cooldown with Hannya Soup and the other boots."
  local IMMUNE_STATUS_ZH = "魔法免疫：不受魔法傷害。"
  local IMMUNE_STATUS_EN = "Spell immune: takes no magic damage."
  local UNIQUE_MS_ZH = "移動速度 +{move_speed}，不疊加：持有多件時只取最高的一件。"
  local UNIQUE_MS_EN = "+{move_speed} movement speed. Movement speed bonuses from boots do not stack; only the highest applies."
  local MANA_BOOTS_ZH = "每秒回復 {mana_regen} 法力，法術增幅 {spell_amp}；持有多件只算一份。"
  local MANA_BOOTS_EN = "Restores {mana_regen} mana per second and grants +{spell_amp} spell amplification; multiple copies count once."
  local function immunity_quirks(time_line, special_line, special)
    return {
      { policy = "as_source", ref = SCR .. "item_perceive_wine.lua:2-33",
        text = "魔免秒數取 RunScript 的 Time（" .. KV .. ":" .. time_line .. "）；AbilitySpecial duration " .. special .. "（" .. KV .. ":" .. special_line .. "）腳本沒有讀，不使用。" },
      { policy = "note", ref = SCR .. "item_perceive_wine.lua:19-31",
        text = "原作每 0.1 秒檢查魔免是否被移除並重掛剩餘時間；現行魔免只會因到期或死亡結束（E179），結果相同。" },
      { policy = "note", ref = KV .. ":" .. time_line,
        text = "原作是 TOGGLE 道具，OnToggleOn 執行效果後腳本立刻 OnToggle 關回；現行當成一般施放（E179）。" },
    }
  end
  return {
    schema_version = 3, kind = "items", id = "batch_i2",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },

    ---------------------------------------------------------------------------
    -- 召喚物用的技能（道具文件裡只能是 hidden）
    abilities = {
      { id = "ninja2_burrow", source_id = "ninja_underground", slot = "hidden", kind = "active", max_level = 1,
        values = {
          cooldown = { 6, unit = "s", source = "kv:AbilityCooldown" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
        },
        text = {
          zh_TW = { name = "掘地", body = "鑽進地底：無法移動與攻擊，但隱形且魔法免疫；再施放一次就鑽出地面。" },
          en = { name = "Burrow", body = "Burrows underground: cannot move or attack, but is invisible and spell immune. Cast again to resurface." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "ninja2_underground", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", tags = { "invisible" }, states = { "root", "disarm", "invisible", "magic_immune" },
            notes = "原作 modifier_ninja_underground（datadriven，IsHidden、沒寫 IsPurgable → 暫定 basic）" },
          { id = "ninja2_underground_model", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", tags = { "invisible" }, states = { "invisible" }, cue = "ninja2_burrow_crater",
            notes = "原作 modifier_B13D（Lua，IsPurgable false）：模型換成坑洞並隱形" },
        },
        events = { on_cast = {
          { op = "if", cond = "has_status(caster, ninja2_underground)",
            actions = {
              { op = "remove_status", from = "caster", status = "ninja2_underground" },
              { op = "remove_status", from = "caster", status = "ninja2_underground_model" },
            },
            otherwise = {
              { op = "apply_status", status = "ninja2_underground", to = "caster" },
              { op = "apply_status", status = "ninja2_underground_model", to = "caster" },
            } },
        } },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/library/common/dummy.lua:1097-1107",
            text = "開關判斷只看 modifier_ninja_underground：若它被驅散而 modifier_B13D 還在，下一次施放會再鑽地（兩個狀態都重套），再下一次才鑽出。" },
          { policy = "note", ref = "npc/npc_abilities_custom_load.txt:6375",
            text = "NO_TARGET | NOT_LEARNABLE，單位出生即有；OverrideAnimation 屬呈現。" },
        },
        provenance = { "npc/npc_abilities_custom_load.txt:6372-6406", "vscripts/library/common/dummy.lua:1097-1107",
          "vscripts/heroes/modifier_B13D.lua:1-42" } },
    },

    ---------------------------------------------------------------------------
    unit_templates = {
      { id = "ninja_unit2", source_id = "ninja_unit2",
        values = {
          hp = { 5, unit = "hp", source = "kv:StatusHealth" },
          mana = { 200, unit = "mana", source = "kv:StatusMana" },
          armor = { 5, unit = "armor", source = "kv:ArmorPhysical" },
          magic_resist = { 0, unit = "pct", source = "kv:MagicalResistance" },
          move_speed = { 320, unit = "move_speed", source = "kv:MovementSpeed" },
          attack_damage = { 1, unit = "dmg", source = "assumed:KV 沒寫 AttackDamageMin／Max，推定沿用 Dota npc_dota_units_base 的 1" },
          attack_interval = { 0.8, unit = "s", source = "kv:AttackRate" },
          attack_range = { 128, unit = "dist", source = "kv:AttackRange" },
          attack_point = { 0.3, unit = "s", source = "kv:AttackAnimationPoint" },
          acquisition = { 600, unit = "dist", source = "kv:AttackAcquisitionRange" },
          bounty_gold = { 15, unit = "gold", source = "kv:BountyGoldMin／BountyGoldMax" },
          vision = { 800, unit = "dist", source = "kv:VisionDaytimeRange／VisionNighttimeRange" },
        },
        text = { zh_TW = { name = "中忍" }, en = { name = "Chunin" } },
        stats = { hp = "$hp", mana = "$mana", armor = "$armor", magic_resist = "$magic_resist", move_speed = "$move_speed" },
        attack = { kind = "melee", range = "$attack_range", interval = "$attack_interval", damage = "$attack_damage",
          attack_point = "$attack_point" },
        acquisition_range = "$acquisition",
        bounty = { gold = "$bounty_gold" },
        vision = { day = "$vision", night = "$vision" },
        ai = "owner_controlled", tags = { "summon" },
        abilities = { "ninja2_burrow" },
        statuses = { "ninja2_passive" },
        notes = "Ability1 ninja_underground、Ability2 for_no_collision、Ability3 update_buy_ninja2、Ability4 phased_dummy、Ability5 ninja_takedamage、Ability6 ninja_true_sight；被動合併成 ninja2_passive。",
        quirks = {
          { policy = "note", ref = "npc/npc_abilities_custom_load.txt:6216-6275",
            text = "phased_dummy 每 0.1 秒讓 50 內的單位一秒無碰撞，自己本來就無碰撞，不另外表達（與 pilot 下忍相同）。" },
          { policy = "as_source", ref = "npc/npc_units_custom_load.txt:2289,2329",
            text = "Level 2、TeamName NEUTRALS 不影響結果（建立時指定施法者的隊伍）。" },
        },
        provenance = { "npc/npc_units_custom_load.txt:2281-2348" } },
    },

    items = {
      -------------------------------------------------------------------------
      { id = "item_lesser_armor", source_id = "item_lesser_armor",
        values = {
          cost = { 400, unit = "gold", source = "kv:ItemCost" },
          block_chance = { 50, unit = "pct", source = "kv:shield_modifier Chance", note = "shield_modifier OnAttacked Random" },
          block = { 50, unit = "dmg", source = "kv:block_modifier MODIFIER_PROPERTY_PHYSICAL_CONSTANT_BLOCK" },
        },
        text = {
          zh_TW = { name = "短甲",
            body = "被動：每次被攻擊命中時有 {block_chance} 機率架起格擋，下一次被攻擊命中時抵擋 {block} 點攻擊傷害；格擋生效的那一擊之後格擋就會撤下，不會連續兩擊都格擋。" },
          en = { name = "Short Armor",
            body = "Passive: each attack that hits you has a {block_chance} chance to raise a guard that blocks {block} damage from the next attack that hits you; the guard drops after the hit it blocks, so two hits in a row are never both blocked." },
        },
        shop = { cost = "$cost" },
        passive = {
          triggers = {
            on_created = { { op = "apply_status", status = "lesser_armor_roll", to = "self" } },
            on_removed = { { op = "remove_status", from = "self", status = "lesser_armor_block" } },
          },
        },
        statuses = {
          { id = "lesser_armor_roll", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", remove_on = { "unequip" },
            triggers = {
              on_attacked_hit = {
                { op = "let", name = "had_block", value = "has_status(self, lesser_armor_block)" },
                { op = "chance", p = "$block_chance", actions = {
                  { op = "apply_status", status = "lesser_armor_block", to = "self" } } },
                { op = "if", cond = "had_block", actions = {
                  { op = "remove_status", from = "self", status = "lesser_armor_block" } } },
              },
            },
            notes = "原作 shield_modifier（Passive、非 MULTIPLE）OnAttacked → Random 50 → block_modifier；以單一實例狀態表達" },
          { id = "lesser_armor_block", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", stats = { physical_block = "$block" },
            notes = "原作 block_modifier（IsBuff 0、IsHidden、沒寫 IsPurgable → 暫定 basic）：自己的 OnAttacked 移除自己" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:2938-2974",
            text = "格擋 modifier 在擲骰命中後才掛，作用於下一次被攻擊；被攻擊時兩個 OnAttacked 都會執行。推定依 modifier 建立順序先擲骰（已有時只是刷新）、再由 block_modifier 移除自己，所以格擋生效的那一擊之後一定撤下，長期平均約三分之一的攻擊被格擋；若順序相反則約一半。待 M2 實機確認。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:2942",
            text = "datadriven OnAttacked 推定只在攻擊命中時觸發（on_attacked_hit）；Random Chance 50 是一般機率，不是 RandomInt(0,100)。" },
          { policy = "as_source", text = "原作說明「50% 機率減少受到的 50 攻擊傷害」；實際機率被上面的時序降低。shield_modifier 非 MULTIPLE，持有多件只擲一次（推定）。" },
          { policy = "no_effect", ref = "vscripts/items/Addon_Items/item_lesser_armor.lua:1-32",
            text = "item_lesser_armor.lua 的 Shock（受傷時隨機回血 50）沒有被 KV 引用，是死碼。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:2903-2976", "shops/nobu_shops.txt:64" } },

      -------------------------------------------------------------------------
      { id = "item_lesser_shield", source_id = "item_lesser_shield",
        values = {
          cost = { 250, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(3091) },
          armor = { 2, unit = "armor", v2_key = "armor", source = "kv:MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" .. at(3111) },
        },
        text = { zh_TW = { name = "足輕盾" }, en = { name = "Infantry Shield" } },
        shop = { cost = "$cost" },
        passive = { stats = { armor = "$armor" } },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        provenance = { KV .. ":3077-3115", "shops/nobu_shops.txt:66" } },

      -------------------------------------------------------------------------
      { id = "item_lightning_katana", source_id = "item_lightning_katana",
        values = {
          cost = { 2470, unit = "gold", source = "kv:ItemCost（= item_katana 1120 + item_power_ingot 300 + item_lightning_scroll 750 + 合成費）" },
          recipe_cost = { 300, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:8542", note = "item_recipe_lightning_katana ItemCost" },
          attack = { 19, unit = "dmg", source = "kv:Passive_lightning_katana MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" },
          attack_speed = { 18, unit = "atk_speed", source = "kv:Passive_lightning_katana MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" },
          ranged_chance = { "1100/101", unit = "pct", source = "script:vscripts/items/Addon_Items/item_lightning_scroll.lua:216-220,235-238 RandomInt(0,100) <= 10" },
          melee_chance = { "1900/101", unit = "pct", source = "script:vscripts/items/Addon_Items/item_lightning_scroll.lua:226-231 trigger = 18" },
          ranged_pity = { 7, unit = "count", source = "script:vscripts/items/Addon_Items/item_lightning_scroll.lua:221,238 raikiri >= 8" },
          melee_pity = { 6, unit = "count", source = "script:vscripts/items/Addon_Items/item_lightning_scroll.lua:230,238 raikiri >= 7" },
          melee_range = { 200, unit = "dist", hidden = true, source = "script:vscripts/items/Addon_Items/item_lightning_scroll.lua:226 GetBaseAttackRange() < 200" },
          counter_reset_delay = { 0.1, unit = "s", hidden = true, source = "script:vscripts/items/Addon_Items/item_lightning_scroll.lua:264-266" },
          chain_radius = { 500, unit = "dist", source = "script:vscripts/items/Addon_Items/item_lightning_scroll.lua:252-256" },
          targets = { 6, unit = "count", hidden = true, source = "kv:targets" },
          chain_max = { expr = "$targets - 1", unit = "count", source = "derived",
            note = "item_lightning_scroll.lua:261-263 targets_shocked 從 1 起算、條件 < targets" },
          chain_damage = { 200, unit = "dmg", damage_type = "magical", kv = { 375 },
            source = "script:vscripts/items/Addon_Items/item_lightning_scroll.lua:292-295 lightning_katana 時 dmg = 200",
            source_differs = "KV AbilitySpecial damage 375 沒有被 item_raikiri 使用" },
        },
        text = {
          zh_TW = { name = "舞雷刀",
            body = "被動：普攻命中非建築目標時，遠程有 {ranged_chance}、近戰有 {melee_chance} 機率觸發雷擊（遠程連續 {ranged_pity} 次、近戰連續 {melee_pity} 次沒觸發時下一次必定觸發），對目標 {chain_radius} 內最多 {chain_max} 個看得見的非魔免敵人各造成 {chain_damage} 魔法傷害，並讓接下來一次攻擊必定命中。" },
          en = { name = "Sword of Dancing Thunder",
            body = "Passive: attacks that land on non-building targets have a {ranged_chance} (ranged) or {melee_chance} (melee) chance to call lightning (guaranteed after {ranged_pity} ranged or {melee_pity} melee landed attacks in a row without it), dealing {chain_damage} magic damage to up to {chain_max} visible, non-spell-immune enemies within {chain_radius} of the target, and your next attack cannot miss." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_katana", "item_power_ingot", "item_lightning_scroll" }, recipe_cost = "$recipe_cost" } },
        unit_state = {
          raikiri = { type = "int", init = "0", reveal = "public", scope = "hero",
            note = "原作 caster.raikiri；與大千鳥十文字槍、雷切共用（同名同型別 hero 變數跨文件是同一個）" },
        },
        passive = {
          stats = { attack_damage = "$attack", attack_speed = "$attack_speed" },
          triggers = {
            on_attack_hit = { filter = "not event.target.is_building", actions = {
              { op = "chance",
                p = "select(self.base_attack_range < $melee_range and not is_hero_id(self, b04), $melee_chance, $ranged_chance)",
                pity_after = "select(self.base_attack_range < $melee_range and not is_hero_id(self, b04), $melee_pity, $ranged_pity)",
                counter_var = "raikiri",
                actions = {
                  { op = "apply_status", status = "lightning_katana_cannot_miss", to = "self" },
                  { op = "for_each",
                    select = { from = "event.target", shape = "circle", radius = "$chain_radius", team = "enemy", types = BASIC,
                      flags = { "visible_only" }, limit = "$chain_max" },
                    actions = {
                      { op = "cue", id = "d09_lightning", at = "self", to = "it" },
                      { op = "damage", to = "it", type = "magical", amount = "$chain_damage * event.attack.effect_multiplier" },
                      { op = "delay", time = "$counter_reset_delay", actions = {
                        { op = "set_state", var = "raikiri", value = "0", on = "self" }, -- FC-019～021（D13）：重設攻擊者自己的計數
                      } },
                    } },
                } },
            } },
          },
        },
        statuses = {
          { id = "lightning_katana_cannot_miss", duration = "self.attack_interval", stacking = "refresh", dispel = "none",
            visibility = "hidden", reveal = "public", states = { "cannot_miss" },
            notes = "原作 modifier_chance_cannot_miss（本道具版只有 CANNOT_MISS，沒有大千鳥版的 101% 爆擊）：持續 1/GetAttacksPerSecond()，作用於下一次攻擊" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_lightning_scroll.lua:219-233",
            text = "以 GetName() 判斷舞雷刀：遠程 RandomInt(0,100) <= 10（11/101）、保底 raikiri >= 8；近戰（GetBaseAttackRange() < 200 且不是 B04）<= 18（19/101）、保底 >= 7。失敗先 +1 再比較，等於遠程連續七次、近戰連續六次未中後下一次必中。原作說明寫 18%／10%。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_lightning_scroll.lua:223-238",
            text = "保底計數是英雄身上的 caster.raikiri，與大千鳥十文字槍、雷切共用（unit_state raikiri，scope hero）。" },
          { policy = "fixed", ref = "vscripts/items/Addon_Items/item_lightning_scroll.lua:264-266",
            text = "FC-020（D13 修正：0.1 秒後重設攻擊者自己的 raikiri）。原作：觸發後 0.1 秒把被攻擊者自己的 raikiri 設為 nil：打到持有雷系道具的敵方英雄會重設對方的保底計數（與 pilot 大千鳥同一段程式）。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_lightning_scroll.lua:252-263",
            text = "搜尋以目標為圓心 500、FOW_VISIBLE、沒有魔免旗標，主目標本身也可能被選中（舞雷刀不先標記主目標）；最多五個單位，FIND_ANY_ORDER 以 entity id 排序表達。腳本內的 IsMagicImmune 檢查是多餘的。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_lightning_scroll.lua:296-298",
            text = "傷害乘 caster.orb，對應 event.attack.effect_multiplier。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:3660-3678",
            text = "Passive_lightning_katana 是 MULTIPLE，帶兩把各自擲骰（共用保底計數）；原作說明寫「唯一被動」。" },
          { policy = "no_effect", ref = "npc/npc_items_custom_load.txt:3633-3635,3650-3657",
            text = "AbilityCooldown 25、AbilityCastPoint 0.2、AbilitySharedCooldown share_lightning_scroll 沒有作用：主動 OnSpellStart 已註解，道具是 PASSIVE，雷擊也不看冷卻。cast_distance／start_radius／end_radius／end_distance 沒有被使用。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:3620-3723", "npc/npc_items_custom_load.txt:8542-8554",
          "vscripts/items/Addon_Items/item_lightning_scroll.lua:190-329", "shops/nobu_shops.txt:118" } },

      -------------------------------------------------------------------------
      { id = "item_lightning_scroll", source_id = "item_lightning_scroll",
        values = {
          cost = { 750, unit = "gold", source = "kv:ItemCost" },
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange" },
          damage = { 350, unit = "dmg", damage_type = "magical", source = "kv:damage", note = "AbilityDamage 相同" },
          neutral_pct = { 70, unit = "pct", source = "script:vscripts/items/Addon_Items/item_lightning_scroll.lua:62-63,82-83 damage*0.7" },
          bounce_radius = { 700, unit = "dist", source = "script:vscripts/items/Addon_Items/item_lightning_scroll.lua:45" },
          targets = { 6, unit = "count", hidden = true, source = "kv:targets" },
          extra_max = { expr = "$targets - 1", unit = "count", source = "derived",
            note = "item_lightning_scroll.lua:48-50 targets_shocked 從 1 起算、條件 < targets；主目標先標記 has_D09，不佔名額" },
        },
        text = {
          zh_TW = { name = "雷之卷",
            body = "主動：對目標敵人施放分叉閃電，並另外打中目標周圍 {bounce_radius} 內最近的 {extra_max} 個看得見的敵人，每人受到 {damage} 魔法傷害（中立野怪只受 {neutral_pct}）。" },
          en = { name = "Book of Thunder",
            body = "Active: forked lightning strikes the target enemy and the {extra_max} nearest visible enemies within {bounce_radius} of it, dealing {damage} magic damage to each (neutral creeps take only {neutral_pct})." },
        },
        shop = { cost = "$cost" },
        active = { kind = "active", target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
          cast = { cast_point = "$cast_point" },
          cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast",
            cooldown_groups = { { group = "lightning_family" } } },
          events = { on_cast = {
            { op = "cue", id = "d09_lightning", at = "caster", to = "target" },
            { op = "for_each",
              select = { from = "target", shape = "circle", radius = "$bounce_radius", team = "enemy", types = BASIC,
                flags = { "visible_only" }, exclude = "target", order = "nearest", limit = "$extra_max" },
              actions = {
                { op = "cue", id = "d09_lightning", at = "caster", to = "it" },
                { op = "damage", require_source_alive = false, flags = { "no_amplify" }, to = "it", type = "magical",
                  amount = "select(not it.is_hero and has_tag(it, neutral), $damage * $neutral_pct, $damage)" },
              } },
            { op = "damage", require_source_alive = false, flags = { "no_amplify" }, to = "target", type = "magical",
              amount = "select(not target.is_hero and has_tag(target, neutral), $damage * $neutral_pct, $damage)" },
            { op = "cue", id = "item_d09_sound", at = "caster" },
          } } },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_lightning_scroll.lua:45-86",
            text = "先打周圍的人、最後才打主目標；搜尋以目標為圓心 700、FOW_VISIBLE、FIND_CLOSEST、沒有魔免旗標（魔免敵人不會被打到）。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_lightning_scroll.lua:62,82",
            text = "非英雄且帶 for_cp_position 技能的單位（中立野怪）只受七成傷害，以 has_tag(it, neutral) 表達（與 I1 凍牙輪相同的推定）。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:1663",
            text = "AbilitySharedCooldown share_lightning_scroll：與舞雷刀同組，但舞雷刀沒有主動，共用冷卻沒有實際影響。" },
          { policy = "no_effect", ref = "vscripts/items/Addon_Items/item_lightning_scroll.lua:95-169",
            text = "GetEnemiesInCone 沒有被呼叫；KV 的 cast_distance、start_radius、end_radius、end_distance 沒有用途。" },
          { policy = "note", ref = "vscripts/items/Addon_Items/item_lightning_scroll.lua:88-91", text = "粒子與 ITEM_D09.sound 屬呈現，改用 cue。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:1649-1725", "vscripts/items/Addon_Items/item_lightning_scroll.lua:7-92",
          "shops/nobu_shops.txt:26,98" } },

      -------------------------------------------------------------------------
      { id = "item_liquor", source_id = "item_liquor",
        values = {
          cost = { 85, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(6831) },
          cooldown = { 35, unit = "s", v2_key = "cooldown", source = "kv:AbilityCooldown" .. at(6823) },
          duration = { 17, unit = "s", v2_key = "active.modifiers.1.duration", source = "kv:Passive_liquor Duration" .. at(6871) },
          attack_speed = { 35, unit = "atk_speed", v2_key = "active.modifiers.1.stats.attackspeed_bonus_constant",
            source = "kv:MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" .. at(6876) },
          charges = { 1, unit = "count", hidden = true, source = "kv:ItemInitialCharges", note = "census 行為用；現行 Rust handler 不讀" },
        },
        text = {
          zh_TW = { name = "燒酒", body = "使用：自身在 {duration} 秒內增加 {attack_speed} 攻擊速度。使用後消耗。" },
          en = { name = "Shochu", body = "Active: gain +{attack_speed} attack speed for {duration} seconds. Consumed on use." },
        },
        shop = { cost = "$cost", consumable = { remove_when_empty = true, charges = "$charges" }, shareable = true },
        active = { kind = "active", target = { type = "none" }, cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
          cast = { immediate = true, ignore_backswing = true },
          events = { on_cast = {
            { op = "apply_status", status = "liquor_haste", to = "caster" },
            { op = "cue", id = "item_liquor_sound", at = "caster" },
          } },
        },
        statuses = {
          { id = "liquor_haste", runtime_id = "Passive_liquor", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", stats = { attack_speed = "$attack_speed" }, cue = "lone_druid_rabid_glow",
            text = { zh_TW = { name = "燒酒", body = "增加 {attack_speed} 攻擊速度。" }, en = { name = "Shochu", body = "+{attack_speed} attack speed." } },
            notes = "原作 Passive_liquor（IsPurgable 1、IsBuff 1、Duration 17；名稱雖叫 Passive 但只由 OnSpellStart 套上）" },
        },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "design_intent", ref = KV .. ":6848-6856",
            text = "原作在 0.1 秒後才 SpendCharge；現行使用成功就移除道具。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:6876",
            text = "ATTACKSPEED_BONUS_CONSTANT 35 點；原作說明寫「35%」。" },
          { policy = "note", ref = "vscripts/items/Addon_Items/item_soul_adder.lua:61-103",
            text = "sound 函式把音效名稱寫成字串 \"keys.sound\"，實際播不出 tidebringer；屬呈現，改用 cue。" },
          { policy = "note", ref = "shops/nobu_shops.txt:10,37,250", text = "同時列在三個商店分頁。" },
        },
        provenance = { KV .. ":6815-6880", "shops/nobu_shops.txt:10" } },

      -------------------------------------------------------------------------
      { id = "item_logs_big_tachi", source_id = "item_logs_big_tachi",
        values = {
          cost = { 2100, unit = "gold", kv = { 2150 },
            source = "kv:item_recipe_logs_big_tachi ItemCost + 零件 ItemCost（item_gorgeous_tachi 展開為 item_tachi、三種鋼與合成費）",
            source_differs = "KV ItemCost 是 2150（以陣太刀 KV 2000 計）；Dota 實際售價是 1950（陣太刀）+合成費 150" },
          recipe_cost = { 150, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:8633", note = "item_recipe_logs_big_tachi ItemCost" },
          str = { 5, unit = "stat", source = "kv:Passive_lightning_katana MODIFIER_PROPERTY_STATS_STRENGTH_BONUS" },
          agi = { 15, unit = "stat", source = "kv:Passive_lightning_katana MODIFIER_PROPERTY_STATS_AGILITY_BONUS" },
          int = { 5, unit = "stat", source = "kv:Passive_lightning_katana MODIFIER_PROPERTY_STATS_INTELLECT_BONUS" },
          attack = { 7, unit = "dmg", source = "kv:Passive_lightning_katana MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" },
          cooldown = { 30, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 75, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          ally_regen = { 40, unit = "per_s", source = "kv:modifier_spell_book_friendly MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT" },
          ally_duration = { 10, unit = "s", source = "kv:modifier_spell_book_friendly Duration" },
          enemy_damage = { 1, unit = "dmg", damage_type = "magical", source = "kv:modifier_spell_book_enemy OnCreated Damage Damage" },
          slow = { 30, unit = "pct", kv = { -30 }, source = "kv:modifier_spell_book_enemy ms_slow", source_differs = "KV 的 RunScript 以負數傳給 dummy.lua Slow；v3 的 slow_pct 寫緩速量（正值）" },
          slow_time = { 4, unit = "s", source = "kv:modifier_spell_book_enemy Duration" },
        },
        text = {
          zh_TW = { name = "戒杖刀",
            body = "主動：對友方單位施放時，目標每秒回復 {ally_regen} 生命，持續 {ally_duration} 秒；對敵方單位施放時，造成 {enemy_damage} 魔法傷害並緩速 {slow}，持續 {slow_time} 秒。" },
          en = { name = "Kaijoutou",
            body = "Active: on an ally, the target regenerates {ally_regen} health per second for {ally_duration} seconds; on an enemy, deals {enemy_damage} magic damage and slows it by {slow} for {slow_time} seconds." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_gorgeous_tachi" }, recipe_cost = "$recipe_cost" } },
        passive = { stats = { str = "$str", agi = "$agi", int = "$int", attack_damage = "$attack" } },
        active = { kind = "active", target = { type = "unit", range = "$cast_range", team = "both", types = BASIC },
          cast = { cast_point = "$cast_point" },
          cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
          events = { on_cast = {
            { op = "if", cond = "is_ally(target, caster)",
              actions = { { op = "apply_status", status = "logs_big_tachi_regen", to = "target" } },
              otherwise = { { op = "apply_status", status = "logs_big_tachi_slow", to = "target" } } },
            { op = "cue", id = "a03t_old_j", at = "target" },
            { op = "cue", id = "black_hole_loop", at = "caster" },
          } } },
        statuses = {
          { id = "logs_big_tachi_regen", duration = "$ally_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", stats = { hp_regen = "$ally_regen" }, cue = "item_spell_book_glow",
            text = { zh_TW = { name = "戒杖回復", body = "每秒回復 {ally_regen} 生命。" }, en = { name = "Kaijoutou Regeneration", body = "Regenerates {ally_regen} health per second." } },
            notes = "原作本道具的 modifier_spell_book_friendly（沒寫 IsPurgable → 暫定 basic）" },
          { id = "logs_big_tachi_slow", duration = "$slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow" }, cue = "sniper_headshot_slow",
            triggers = { on_created = { { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$enemy_damage" } } },
            text = { zh_TW = { name = "緩速", body = "移動速度降低 {slow}。" }, en = { name = "Slowed", body = "Movement speed reduced by {slow}." } },
            notes = "原作本道具的 modifier_spell_book_enemy：IsPurgable 1；OnCreated 造成 1 點魔法傷害並以全域緩速表鍵 big_tachi 緩速，OnDestroy ReturnSpeed。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_spell_book.lua:2-13",
            text = "主動直接呼叫軍略書（item_spell_book.lua）的 Shock，依隊伍套友方或敵方 modifier；modifier 是本道具自己宣告的版本。TEAM_BOTH，自己也能當目標（套友方效果）。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:4639-4663",
            text = "敵方 modifier 重套只刷新，推定不會再跑 OnCreated，所以 1 點傷害與緩速表寫入只在第一次套上時發生（緩速仍持續到刷新後的結束時間）。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:4665-4684",
            text = "屬性 modifier 名為 Passive_lightning_katana（MULTIPLE），只是命名沿用；KV 敏捷 15，原作說明寫「靈活 +10、全能力 +5」（等於敏捷 15）。" },
          { policy = "note", text = "原作說明「回復 400 生命」＝每秒 40 × 10 秒。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:4569-4686", "npc/npc_items_custom_load.txt:8633-8645",
          "vscripts/items/Addon_Items/item_spell_book.lua:2-13", "vscripts/library/common/dummy.lua:816-864", "shops/nobu_shops.txt:124" } },

      -------------------------------------------------------------------------
      { id = "item_magic_defence_boots", source_id = "item_magic_defence_boots",
        values = {
          cost = { 2700, unit = "gold", kv = { 2800 },
            source = "kv:item_recipe_magic_defence_boots ItemCost + 零件 ItemCost（item_pleated_skirt、item_regeneration_ingot×2、item_magic_resistance_ingot）",
            source_differs = "KV ItemCost 是 2800；Dota 實際售價是 500＋650＋650＋650＋合成費 250" },
          recipe_cost = { 250, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:9217", note = "item_recipe_magic_defence_boots ItemCost" },
          hp_regen = { 30, unit = "per_s", source = "kv:Passive_item_pleated_skirt MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT" },
          move_speed = { 70, unit = "move_speed", source = "kv:Passive_item_pleated_skirt MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE" },
          magic_resist_pool = { 15, unit = "pct", source = "kv:magical_resistance", note = "item_magical_resistance.lua OnEquip" },
          cooldown = { 60, unit = "s", source = "kv:AbilityCooldown" },
          immune_duration = { 4.5, unit = "s", source = "kv:OnToggleOn Time", note = "OnToggleOn RunScript 參數" },
        },
        text = {
          zh_TW = { name = "禦魔長踏",
            body = "被動：移動速度 +{move_speed}（唯一，與其他鞋類取最高）；基礎魔法抗性（魔抗池）+{magic_resist_pool}，同名道具只算一次。主動：移除自身所有來自敵方的狀態，並獲得魔法免疫 {immune_duration} 秒。與般若湯同組道具共用冷卻。" },
          en = { name = "Boots of Evil Resist",
            body = "Passive: +{move_speed} movement speed (unique, highest among boots); +{magic_resist_pool} base magic resistance; multiple copies count once. Active: removes every status enemies applied to you and grants spell immunity for {immune_duration} seconds. Shares its cooldown with Hannya Soup and items in its group." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_pleated_skirt", "item_regeneration_ingot", "item_regeneration_ingot", "item_magic_resistance_ingot" },
          recipe_cost = "$recipe_cost" } },
        passive = {
          stats = { hp_regen = "$hp_regen" },
          triggers = { on_created = {
            { op = "apply_status", status = "magic_defence_boots_unique_ms", to = "self" },
            { op = "apply_status", status = "magic_defence_boots_pool", to = "self" },
          } },
        },
        active = { kind = "active", target = { type = "none" },
          cast = { immediate = true },
          cost = { cooldown = "$cooldown", cooldown_groups = { { group = "boots_family" } } },
          events = { on_cast = {
            { op = "remove_status", from = "caster", dispel = "all", team_of_source = "enemy", exclude_tags = { "true_sight", "c25e_mark" } },
            { op = "apply_status", status = "magic_defence_boots_immune", to = "caster" },
            { op = "cue", id = "item_perceive_wine", at = "caster" },
          } } },
        statuses = {
          { id = "magic_defence_boots_unique_ms", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "move_speed_unique", dispel = "none", visibility = "hidden", reveal = "public", remove_on = { "unequip" },
            stats = { move_speed = "$move_speed" },
            notes = "原作 Passive_item_pleated_skirt 的 MOVESPEED_BONUS_UNIQUE：Dota 只取所有唯一移速來源的最高值（與暗部長靴同組）。" },
          { id = "magic_defence_boots_immune", duration = "$immune_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", tags = { "perceive_wine" }, states = { "magic_immune" }, cue = "item_perceive_wine", reapply_if_removed = true,
            text = { zh_TW = { name = "般若之護", body = "魔法免疫：不受魔法傷害。" }, en = { name = "Prajna Ward", body = "Spell immune: takes no magic damage." } },
            notes = "原作 modifier_perceive_wine（沒寫 IsPurgable，暫定 basic）；item_perceive_wine.lua Shock 每 0.1 秒補回剩餘時間。" },
          { id = "magic_defence_boots_pool", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "magic_resist_pool_item_magic_defence_boots", dispel = "none", visibility = "hidden", reveal = "public",
            remove_on = { "unequip" }, stats = { magic_resist_pool = "$magic_resist_pool" },
            notes = "原作 OnEquip／OnUnequip 呼叫 item_magical_resistance.lua:1-33，以 caster.items[道具名] 計數，同名只加一次；dummy.lua:690-700 把池寫成基礎魔抗（上限由規則層）。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_perceive_wine.lua:2-33",
            text = "主動與暗部長靴、流風捷履同一支腳本：移除敵方來源的所有狀態（不分能否驅散，排除 modifier_truesight 與名稱含 C25E 者），魔免 0.1 秒補回；Purge 的 frameOnly 參數使它幾乎沒有作用。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:12557",
            text = "KV 生命回復 30，原作說明寫 20（破魔版 KV 才是 20）；照 KV。說明的「降低所受的法術傷害 15%」就是魔抗池 +15。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:12500",
            text = "主動以 TOGGLE 實作，腳本最後呼叫 ability:OnToggle() 復位，等同一次性主動。" },
          { policy = "no_effect", ref = "npc/npc_items_custom_load.txt:12580-12584", text = "AbilitySpecial duration 5 沒有被使用；實際魔免時間是 RunScript 參數 Time 4.5。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:12495-12591", "npc/npc_items_custom_load.txt:9217-9229",
          "vscripts/items/Addon_Items/item_magical_resistance.lua:1-33", "vscripts/items/Addon_Items/item_perceive_wine.lua:2-33",
          "vscripts/library/common/dummy.lua:690-700", "shops/nobu_shops.txt:142" } },

      -------------------------------------------------------------------------
      { id = "item_magic_defence_boots_1", source_id = "item_magic_defence_boots_1",
        values = {
          cost = { 4700, unit = "gold", kv = { 4400 },
            source = "kv:item_recipe_magic_defence_boots_1 ItemCost + 零件 ItemCost（item_hama_orb、item_magic_defence_boots 展開）",
            source_differs = "KV ItemCost 是 4400；Dota 實際售價是 2000＋2700＋合成費 0" },
          recipe_cost = { 0, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:15506", note = "item_recipe_magic_defence_boots_1 ItemCost" },
          hp_regen = { 20, unit = "per_s", source = "kv:Passive_item_pleated_skirt MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT" },
          move_speed = { 70, unit = "move_speed", source = "kv:Passive_item_pleated_skirt MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE" },
          magic_resist_pool = { 15, unit = "pct", source = "kv:magical_resistance", note = "item_magical_resistance.lua OnEquip" },
          cooldown = { 60, unit = "s", source = "kv:AbilityCooldown" },
          immune_duration = { 9, unit = "s", source = "kv:OnToggleOn Time", note = "OnToggleOn RunScript 參數" },
        },
        text = {
          zh_TW = { name = "破魔．禦魔長踏",
            body = "被動：移動速度 +{move_speed}（唯一，與其他鞋類取最高）；基礎魔法抗性（魔抗池）+{magic_resist_pool}，同名道具只算一次。主動：移除自身所有來自敵方的狀態，並獲得魔法免疫 {immune_duration} 秒。與般若湯同組道具共用冷卻。" },
          en = { name = "Hama, Boots of Evil Resist",
            body = "Passive: +{move_speed} movement speed (unique, highest among boots); +{magic_resist_pool} base magic resistance; multiple copies count once. Active: removes every status enemies applied to you and grants spell immunity for {immune_duration} seconds. Shares its cooldown with Hannya Soup and items in its group." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_hama_orb", "item_magic_defence_boots" }, recipe_cost = "$recipe_cost" } },
        passive = {
          stats = { hp_regen = "$hp_regen" },
          triggers = { on_created = {
            { op = "apply_status", status = "magic_defence_boots_1_unique_ms", to = "self" },
            { op = "apply_status", status = "magic_defence_boots_1_pool", to = "self" },
          } },
        },
        active = { kind = "active", target = { type = "none" },
          cast = { immediate = true },
          cost = { cooldown = "$cooldown", cooldown_groups = { { group = "boots_family" } } },
          events = { on_cast = {
            { op = "remove_status", from = "caster", dispel = "all", team_of_source = "enemy", exclude_tags = { "true_sight", "c25e_mark" } },
            { op = "apply_status", status = "magic_defence_boots_1_immune", to = "caster" },
            { op = "cue", id = "item_perceive_wine", at = "caster" },
          } } },
        statuses = {
          { id = "magic_defence_boots_1_unique_ms", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "move_speed_unique", dispel = "none", visibility = "hidden", reveal = "public", remove_on = { "unequip" },
            stats = { move_speed = "$move_speed" }, notes = "同禦魔長踏" },
          { id = "magic_defence_boots_1_immune", duration = "$immune_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", tags = { "perceive_wine" }, states = { "magic_immune" }, cue = "item_perceive_wine", reapply_if_removed = true,
            text = { zh_TW = { name = "般若之護", body = "魔法免疫：不受魔法傷害。" }, en = { name = "Prajna Ward", body = "Spell immune: takes no magic damage." } },
            notes = "同禦魔長踏，Time 9" },
          { id = "magic_defence_boots_1_pool", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "magic_resist_pool_item_magic_defence_boots_1", dispel = "none", visibility = "hidden", reveal = "public",
            remove_on = { "unequip" }, stats = { magic_resist_pool = "$magic_resist_pool" },
            notes = "原作 item_magical_resistance.lua:1-33 以 caster.items[ability:GetName()] 計數：同名只加一次；與禦魔長踏名稱不同，兩者同時持有會各加一次（unique_group 不同）。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:15792",
            text = "升級版生命回復 20，比基本版的 30 低；原作說明兩者都寫 20。照 KV。" },
          { policy = "as_source", text = "原作說明寫魔免 9 秒，與 RunScript Time 一致；AbilitySpecial duration 10 沒有被使用。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:15749", text = "KV ItemCost 4400，零件＋合成費 4700。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:15728-15826", "npc/npc_items_custom_load.txt:15506-15518",
          "vscripts/items/Addon_Items/item_magical_resistance.lua:1-33", "vscripts/items/Addon_Items/item_perceive_wine.lua:2-33",
          "shops/nobu_shops.txt:229" } },

      -------------------------------------------------------------------------
      { id = "item_magic_resistance_ingot", source_id = "item_magic_resistance_ingot",
        values = {
          cost = { 650, unit = "gold", source = "kv:ItemCost" },
          magic_resist_pool = { 7, unit = "pct", source = "kv:magical_resistance", note = "item_magical_resistance.lua OnEquip" },
        },
        text = {
          zh_TW = { name = "禦魔鋼", body = "被動：基礎魔法抗性（魔抗池）+{magic_resist_pool}，同名道具只算一次。" },
          en = { name = "Steel of Evil Resist", body = "Passive: +{magic_resist_pool} base magic resistance; multiple copies count once." },
        },
        shop = { cost = "$cost" },
        passive = { triggers = { on_created = { { op = "apply_status", status = "magic_resistance_ingot_pool", to = "self" } } } },
        statuses = {
          { id = "magic_resistance_ingot_pool", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "magic_resist_pool_item_magic_resistance_ingot", dispel = "none", visibility = "hidden", reveal = "public",
            remove_on = { "unequip" }, stats = { magic_resist_pool = "$magic_resist_pool" },
            notes = "原作 OnEquip／OnUnequip 呼叫 item_magical_resistance.lua:1-33，以 caster.items[道具名] 計數，同名只加一次；dummy.lua:690-700 把池寫成基礎魔抗（上限由規則層）。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "note", ref = "npc/npc_items_custom_load.txt:2090-2106", text = "Passive_magicresistance_ingot 屬性全 0，沒有效果。" },
          { policy = "as_source", text = "原作說明「減少所受的法術傷害 7%」就是魔抗池 +7；池上限 70，帶多件不同名的魔抗道具時可能被截斷。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:2048-2117", "vscripts/items/Addon_Items/item_magical_resistance.lua:1-33",
          "vscripts/library/common/dummy.lua:690-700", "shops/nobu_shops.txt:78" } },

      -------------------------------------------------------------------------
      { id = "item_magic_ring", source_id = "item_magic_ring",
        values = {
          cost = { 0, unit = "gold", source = "assumed:原作 KV 沒有這個道具（也沒有合成表）；Dota 商店不會顯示，暫填 0" },
        },
        text = { zh_TW = { name = "法術戒指" }, en = { name = "Spell Ring" } },
        shop = { cost = "$cost", purchasable = false },
        coverage = "deferred", deferred_reason = "not_in_source", confidence = "high",
        notes = "商店清單有 item_magic_ring（法術戒指），但 npc_items_custom_load.txt 沒有道具定義、也沒有腳本（shops/nobu_shops.txt:74；只在 npc/npc_items_custom_load.txt:1044 有別的道具裡的 modifier 名稱 Passive_magic_ring，以及 B08_ai.lua:493、C13.lua:3、C18.lua:78 以名稱比對）。依主 session D13 決定（原作不存在的道具不移植），不移植。",
        quirks = {
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C13.lua:2-6",
            text = "C13 的 flash_items、C18 的閃現判斷、B08 AI 都以名稱列出 item_magic_ring；道具不存在，這些分支永遠不成立。" },
        },
        provenance = { "shops/nobu_shops.txt:74", "npc/npc_items_custom_load.txt:1044", "vscripts/heroes/C_Neutral/C13.lua:2-6" } },

      -------------------------------------------------------------------------
      { id = "item_mana_boots", source_id = "item_mana_boots",
        values = {
          cost = { 2650, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(12423) },
          recipe_cost = { 1500, unit = "gold", source = "kv:item_recipe_mana_boots ItemCost" .. at(9206) },
          intellect = { 5, unit = "stat", v2_key = "modifiers.stats_intellect_bonus", source = "kv:MODIFIER_PROPERTY_STATS_INTELLECT_BONUS" .. at(12480) },
          mana_regen = { 8.5, unit = "per_s", v2_key = "modifiers.mana_regen_constant", source = "kv:MODIFIER_PROPERTY_MANA_REGEN_CONSTANT" .. at(12466) },
          spell_amp = { 8, unit = "pct", v2_key = "modifiers.spell_amplify_percentage", source = "kv:MODIFIER_PROPERTY_SPELL_AMPLIFY_PERCENTAGE" .. at(12468) },
          move_speed = { 60, unit = "move_speed", v2_key = "unique_passive.ms", source = "kv:MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE" .. at(12488) },
          priority = { 60, unit = "number", hidden = true, v2_key = "unique_passive.priority",
            source = "kv:MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE" .. at(12488), note = "唯一移速以數值本身當優先度（E183）" },
          cooldown = { 60, unit = "s", v2_key = "cooldown", source = "kv:AbilityCooldown" .. at(12416) },
          immunity = { 4.5, unit = "s", v2_key = "active.magic_immunity", source = "kv:RunScript Time" .. at(12432) },
        },
        text = {
          zh_TW = { name = "仕手能靴", body = UNIQUE_MS_ZH .. "\n" .. MANA_BOOTS_ZH .. "\n" .. IMMUNITY_ZH },
          en = { name = "Boots of Shi'te", body = UNIQUE_MS_EN .. "\n" .. MANA_BOOTS_EN .. "\n" .. IMMUNITY_EN },
        },
        shop = { cost = "$cost", recipe = { components = { "item_pleated_skirt", "item_spirit_ingot" }, recipe_cost = "$recipe_cost" } },
        passive = { stats = { int = "$intellect" },
          triggers = { on_created = {
            { op = "apply_status", status = "mana_boots_unique_ms", to = "self" },
            { op = "apply_status", status = "mana_boots_unique", to = "self" },
          } },
        },
        active = { kind = "active", target = { type = "none" }, cost = { cooldown = "$cooldown", cooldown_groups = { { group = "boots_family" } } },
          cast = { immediate = true },
          events = { on_cast = {
            { op = "remove_status", from = "caster", dispel = "all", team_of_source = "enemy", exclude_tags = { "true_sight", "c25e_mark" } },
            { op = "apply_status", status = "mana_boots_immune", to = "caster" },
            { op = "cue", id = "item_perceive_wine", at = "caster" },
          } },
        },
        statuses = {
          { id = "mana_boots_unique_ms", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "move_speed_unique", dispel = "none", visibility = "hidden", reveal = "public", remove_on = { "unequip" },
            stats = { move_speed = "$move_speed" },
            notes = "原作 Passive_item_pleated_skirt（MULTIPLE）的 MOVESPEED_BONUS_UNIQUE：Dota 只取所有唯一移速來源的最高值。" },
          { id = "mana_boots_unique", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "passive_item_mana_boots", dispel = "none", visibility = "hidden", reveal = "public", remove_on = { "unequip" },
            stats = { mana_regen = "$mana_regen", spell_amp_pct = "$spell_amp" },
            notes = "原作 Passive_item_mana_boots（非 MULTIPLE，與破魔．仕手能靴同名）：多件只算一份，逐屬性取最高。" },
          { id = "mana_boots_immune", runtime_id = "magic_immune", duration = "$immunity", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", tags = { "perceive_wine" }, states = { "magic_immune" }, cue = "item_perceive_wine", reapply_if_removed = true,
            text = { zh_TW = { name = "般若之護", body = IMMUNE_STATUS_ZH }, en = { name = "Prajna Ward", body = IMMUNE_STATUS_EN } },
            notes = "原作 modifier_perceive_wine（沒寫 IsPurgable，暫定 basic）；item_perceive_wine.lua Shock 每 0.1 秒補回剩餘時間。" },
        },
        coverage = "rust_handler", handler = handler({ "$priority" }), confidence = "medium",
        quirks = (function()
          local q = immunity_quirks(12432, 12454, 5)
          q[#q + 1] = { policy = "as_source",
            text = "移動速度、回魔與法術增幅只由唯一狀態提供（原作 MOVESPEED_BONUS_UNIQUE 只取最高；Passive_item_mana_boots 非 MULTIPLE，多件只算一份），智力照常相加；passive.stats 不再重寫這些數值，也不再把整組被動設成 unique_group（D13 決定，2026-10-29）。" }
          q[#q + 1] = { policy = "as_source", ref = KV .. ":12458-12491",
            text = "屬性寫在第二個頂層 Modifiers 區塊；現行依序合併所有 Modifiers 區塊（E183）。" }
          q[#q + 1] = { policy = "as_source", ref = KV .. ":12468",
            text = "法術增幅只放大帶 inflictor 的傷害（KV Damage、帶 ability 的 ApplyDamage）；AMHC:Damage 與普攻不放大（E188）。" }
          q[#q + 1] = { policy = "note", ref = KV .. ":12460-12470",
            text = "原作說明把回魔與增幅標成【唯一被動】，KV 是一般 Passive modifier；現行照一般屬性加成。" }
          q[#q + 1] = { policy = "as_source", ref = "npc/npc_items_custom_load.txt:12435-12491",
            text = "同一道具有兩個 \"Modifiers\" 區塊（第一塊 modifier_perceive_wine、第二塊 Passive_item_mana_boots 與 Passive_item_pleated_skirt）；與暗部長靴相同，推定兩塊都有效；若只取第一塊，所有屬性整個失效。需實機確認。" }
          q[#q + 1] = { policy = "as_source", ref = "vscripts/items/Addon_Items/item_perceive_wine.lua:2-33",
            text = "主動與暗部長靴、流風捷履同一支腳本：移除敵方來源的所有狀態（不分能否驅散，排除 modifier_truesight 與名稱含 C25E 者），魔免 0.1 秒補回。" }
          return q
        end)(),
        provenance = { KV .. ":12404-12492", KV .. ":9202-9214", SCR .. "item_perceive_wine.lua:1-33", "vscripts/items/Addon_Items/item_perceive_wine.lua:2-33", "shops/nobu_shops.txt:141" } },

      -------------------------------------------------------------------------
      { id = "item_mana_boots_1", source_id = "item_mana_boots_1",
        values = {
          cost = { 4650, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(15656),
            kv = { 3950 }, source_differs = RECIPE_TOTAL },
          intellect = { 5, unit = "stat", v2_key = "modifiers.stats_intellect_bonus", source = "kv:MODIFIER_PROPERTY_STATS_INTELLECT_BONUS" .. at(15713) },
          mana_regen = { 8.5, unit = "per_s", v2_key = "modifiers.mana_regen_constant", source = "kv:MODIFIER_PROPERTY_MANA_REGEN_CONSTANT" .. at(15699) },
          spell_amp = { 10, unit = "pct", v2_key = "modifiers.spell_amplify_percentage", source = "kv:MODIFIER_PROPERTY_SPELL_AMPLIFY_PERCENTAGE" .. at(15701) },
          move_speed = { 60, unit = "move_speed", v2_key = "unique_passive.ms", source = "kv:MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE" .. at(15721) },
          priority = { 60, unit = "number", hidden = true, v2_key = "unique_passive.priority",
            source = "kv:MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE" .. at(15721), note = "唯一移速以數值本身當優先度（E183）" },
          cooldown = { 60, unit = "s", v2_key = "cooldown", source = "kv:AbilityCooldown" .. at(15648) },
          immunity = { 9, unit = "s", v2_key = "active.magic_immunity", source = "kv:RunScript Time" .. at(15665) },
        },
        text = {
          zh_TW = { name = "破魔．仕手能靴", body = UNIQUE_MS_ZH .. "\n" .. MANA_BOOTS_ZH .. "\n" .. IMMUNITY_ZH },
          en = { name = "Hama, Boots of Shi'te", body = UNIQUE_MS_EN .. "\n" .. MANA_BOOTS_EN .. "\n" .. IMMUNITY_EN },
        },
        shop = { cost = "$cost", recipe = { components = { "item_hama_orb", "item_mana_boots" } } },
        passive = { stats = { int = "$intellect" },
          triggers = { on_created = {
            { op = "apply_status", status = "mana_boots_1_unique_ms", to = "self" },
            { op = "apply_status", status = "mana_boots_1_unique", to = "self" },
          } },
        },
        active = { kind = "active", target = { type = "none" }, cost = { cooldown = "$cooldown", cooldown_groups = { { group = "boots_family" } } },
          cast = { immediate = true },
          events = { on_cast = {
            { op = "remove_status", from = "caster", dispel = "all", team_of_source = "enemy", exclude_tags = { "true_sight", "c25e_mark" } },
            { op = "apply_status", status = "mana_boots_1_immune", to = "caster" },
            { op = "cue", id = "item_perceive_wine", at = "caster" },
          } },
        },
        statuses = {
          { id = "mana_boots_1_unique_ms", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "move_speed_unique", dispel = "none", visibility = "hidden", reveal = "public", remove_on = { "unequip" },
            stats = { move_speed = "$move_speed" }, notes = "同仕手能靴" },
          { id = "mana_boots_1_unique", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "passive_item_mana_boots", dispel = "none", visibility = "hidden", reveal = "public", remove_on = { "unequip" },
            stats = { mana_regen = "$mana_regen", spell_amp_pct = "$spell_amp" },
            notes = "原作 Passive_item_mana_boots（非 MULTIPLE，與仕手能靴同名）：多件只算一份，逐屬性取最高。" },
          { id = "mana_boots_1_immune", runtime_id = "magic_immune", duration = "$immunity", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", tags = { "perceive_wine" }, states = { "magic_immune" }, cue = "item_perceive_wine", reapply_if_removed = true,
            text = { zh_TW = { name = "般若之護", body = IMMUNE_STATUS_ZH }, en = { name = "Prajna Ward", body = IMMUNE_STATUS_EN } },
            notes = "同仕手能靴，Time 9" },
        },
        coverage = "rust_handler", handler = handler({ "$priority" }), confidence = "medium",
        quirks = (function()
          local q = immunity_quirks(15665, 15687, 10)
          q[#q + 1] = { policy = "as_source",
            text = "移動速度、回魔與法術增幅只由唯一狀態提供（原作 MOVESPEED_BONUS_UNIQUE 只取最高；Passive_item_mana_boots 非 MULTIPLE，多件只算一份），智力照常相加；passive.stats 不再重寫這些數值，也不再把整組被動設成 unique_group（D13 決定，2026-10-29）。" }
          q[#q + 1] = { policy = "as_source", ref = KV .. ":15691-15724",
            text = "屬性寫在第二個頂層 Modifiers 區塊；現行依序合併所有 Modifiers 區塊（E183）。合成卷軸 item_recipe_mana_boots_1 沒有合成費。" }
          q[#q + 1] = { policy = "as_source", ref = KV .. ":15701",
            text = "法術增幅只放大帶 inflictor 的傷害（E188）。" }
          q[#q + 1] = { policy = "as_source", ref = "npc/npc_items_custom_load.txt:15668-15724",
            text = "與仕手能靴相同，有兩個 \"Modifiers\" 區塊，推定兩塊都有效；需實機確認。" }
          q[#q + 1] = { policy = "as_source", ref = "npc/npc_items_custom_load.txt:15693-15703",
            text = "Passive_item_mana_boots 與基本版同名且非 MULTIPLE：同時持有兩雙時推定只有一份生效（以 unique_group 逐屬性取最高，即技能增幅 10%）。" }
          q[#q + 1] = { policy = "as_source", text = "原作說明寫魔免 9 秒，與 RunScript Time 一致；AbilitySpecial duration 10 沒有被使用。" }
          q[#q + 1] = { policy = "note", ref = "npc/npc_items_custom_load.txt:15656", text = "KV ItemCost 3950，零件＋合成費 4650。" }
          return q
        end)(),
        provenance = { KV .. ":15635-15725", KV .. ":15491-15503", SCR .. "item_perceive_wine.lua:1-33", "vscripts/items/Addon_Items/item_perceive_wine.lua:2-33", "shops/nobu_shops.txt:228" } },

      -------------------------------------------------------------------------
      { id = "item_mana_ingot", source_id = "item_mana_ingot",
        values = {
          cost = { 650, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(2275) },
          mana = { 300, unit = "mana", v2_key = "modifiers.mana_bonus", source = "kv:MODIFIER_PROPERTY_MANA_BONUS" .. at(2299) },
        },
        text = { zh_TW = { name = "法力鋼" }, en = { name = "Steel of Mana" } },
        shop = { cost = "$cost" },
        passive = { stats = { max_mana = "$mana" } },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/npc_items_custom_load.txt:2287", text = "modifier 名稱沿用 Passive_strength_ingot（MULTIPLE），只是命名。" },
          { policy = "no_effect", ref = "vscripts/items/Addon_Items/item_mana_ingot.lua:1-17",
            text = "item_mana_ingot.lua 的 OnEquip／OnUnequip（非英雄改 caster.mana 欄位）沒有被 KV 引用，是死碼。" },
          { policy = "note", ref = "shops/nobu_shops.txt:30,86", text = "同時列在偷買（pregame）與基礎道具兩個商店分頁。" },
        },
        provenance = { KV .. ":2261-2305", "shops/nobu_shops.txt:30" } },

      -------------------------------------------------------------------------
      { id = "item_mana_thief", source_id = "item_mana_thief",
        values = {
          cost = { 4100, unit = "gold", source = "kv:ItemCost（= item_energy_ingot 1900 + item_speed_clothes 700 + 合成費）" },
          recipe_cost = { 1500, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:8497", note = "item_recipe_mana_thief ItemCost" },
          all_stats = { 10, unit = "stat", source = "kv:Passive_lightning_katana MODIFIER_PROPERTY_STATS_STRENGTH_BONUS", note = "力量、敏捷、智力三項同為 10" },
          attack_speed = { 25, unit = "atk_speed", source = "kv:Passive_lightning_katana MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" },
          burn_base = { 30, unit = "mana", source = "script:vscripts/items/Addon_Items/item_mana_thief.lua:173 30 + GetLevel()*2" },
          burn_per_level = { 2, unit = "mana", source = "script:vscripts/items/Addon_Items/item_mana_thief.lua:173" },
          burn_cap = { 120, unit = "mana", source = "script:vscripts/items/Addon_Items/item_mana_thief.lua:174-176" },
          low_mana = { 40, unit = "pct", source = "script:vscripts/items/Addon_Items/item_mana_thief.lua:191 GetMana()/GetMaxMana() <= 0.4" },
          low_mana_ratio = { 50, unit = "pct", source = "script:vscripts/items/Addon_Items/item_mana_thief.lua:192 mana_to_burn * 0.5" },
          cooldown = { 25, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          active_burn = { 350, unit = "mana", source = "script:vscripts/items/Addon_Items/item_mana_thief.lua:31" },
        },
        text = {
          zh_TW = { name = "大俱利伽羅",
            body = "被動：普攻命中非建築目標時，燒掉目標 {burn_base} + 等級 × {burn_per_level} 點法力（最多 {burn_cap}，不超過目標現有法力），並造成等量魔法傷害；目標是法力不高於 {low_mana} 的英雄時效果減為 {low_mana_ratio}。魔免目標的法力照樣被燒掉，但不受傷害。持有多件只觸發一次。主動：燒掉目標敵人最多 {active_burn} 點法力並造成等量魔法傷害。" },
          en = { name = "Divine Ring",
            body = "Passive: attacks that land on non-building targets burn {burn_base} + level × {burn_per_level} mana (at most {burn_cap}, and no more than the target has) and deal that much magic damage; against heroes with {low_mana} mana or less, the effect is reduced to {low_mana_ratio}. Spell-immune targets still lose the mana but take no damage. Several copies trigger only once. Active: burns up to {active_burn} mana from the target enemy and deals that much magic damage." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_energy_ingot", "item_speed_clothes" }, recipe_cost = "$recipe_cost" } },
        passive = {
          stats = { all_stats = "$all_stats", attack_speed = "$attack_speed" },
          triggers = { on_created = { { op = "apply_status", status = "mana_thief_burn", to = "self" } } },
        },
        active = { kind = "active", target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
          cast = { cast_point = "$cast_point" },
          cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
          events = { on_cast = {
            { op = "let", name = "amt", value = "select(target.is_magic_immune, 0, min(target.mana, $active_burn))" },
            { op = "burn_mana", to = "target", amount = "amt", source = "caster" },
            { op = "cue", id = "nyx_mana_burn", at = "target" },
            { op = "damage", require_source_alive = false, flags = { "no_amplify" }, to = "target", type = "magical", amount = "amt" },
          } } },
        statuses = {
          { id = "mana_thief_burn", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", remove_on = { "unequip" },
            triggers = {
              on_attack_hit = { filter = "not event.target.is_building", actions = {
                { op = "let", name = "amt",
                  value = "min(event.target.mana, min($burn_base + self.level * $burn_per_level, $burn_cap)) * self.illusion_damage * event.attack.effect_multiplier * select(event.target.is_hero and event.target.mana_pct <= $low_mana, $low_mana_ratio, 1)" },
                { op = "burn_mana", to = "event.target", amount = "amt", source = "self" },
                { op = "cue", id = "nyx_mana_burn", at = "event.target" },
                { op = "damage", require_source_alive = false, flags = { "no_amplify" }, to = "event.target", type = "magical", amount = "amt" },
              } },
            },
            notes = "原作 Passive_lightning_katana_Unique（Passive、非 MULTIPLE）OnAttackLanded → item_mana_thief.lua Shock；以單一實例狀態表達" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_mana_thief.lua:185-187",
            text = "分身攻擊時燒魔量再乘分身的傷害倍率 caster.illusion_damage（各英雄建立分身時設定：A13、A27、B11、B15、B17、C02、C16），以 v3.1 單位屬性 illusion_damage 表達（本體為 1）；全域傷害規則另外對分身的非物理傷害乘同一倍率，由 create_illusion.outgoing 與規則層處理。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_mana_thief.lua:168-220",
            text = "被動燒魔不檢查魔免：魔免目標照樣失去法力，魔法傷害由規則層歸零（原作說明「無視魔免消去」）。也會對友方（反補）生效。燒魔量乘 caster.orb（event.attack.effect_multiplier）。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_mana_thief.lua:191",
            text = "法力比例在燒魔前判定；只對英雄減半。等級是攻擊者的 GetLevel()（分身為本體等級）。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:6982-6994",
            text = "Passive_lightning_katana_Unique 非 MULTIPLE：以單一實例的 mana_thief_burn 狀態表達；持有兩件時卸下一件是否讓另一件的被動失效，待 M2 確認。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_mana_thief.lua:26-71",
            text = "主動燒掉 min(目前法力, 350)；目標魔免時為 0（施放時本來就不能指定魔免敵人）。傷害類型取 AbilityUnitDamageType（魔法）。" },
          { policy = "no_effect", ref = "vscripts/items/Addon_Items/item_mana_thief.lua:1-24",
            text = "OnEquip／OnUnequip 依主屬性加 15 點的程式已註解；modifier_add_strength／agility／intellect 不會被套上。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:6961", text = "屬性 modifier 名稱沿用 Passive_lightning_katana，只是命名。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:6883-7023", "npc/npc_items_custom_load.txt:8497-8509",
          "vscripts/items/Addon_Items/item_mana_thief.lua:1-220", "shops/nobu_shops.txt:176" } },

      -------------------------------------------------------------------------
      { id = "item_moon_mirror", source_id = "item_moon_mirror",
        values = {
          cost = { 1800, unit = "gold", kv = { 2100 },
            source = "kv:item_recipe_moon_mirror ItemCost + 零件 ItemCost（item_tea_bowl）",
            source_differs = "KV ItemCost 是 2100；Dota 實際售價是 1300＋合成費 500（與 pilot 靈都宮燈的展開一致）" },
          recipe_cost = { 500, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:9275", note = "item_recipe_moon_mirror ItemCost" },
          aura_radius = { 700, unit = "dist", source = "kv:modifier_item_moon_mirror_aura Aura_Radius" },
          aura_hp_regen = { 10, unit = "per_s", source = "kv:modifier_item_moon_mirror MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT" },
          aura_mana_regen = { 5, unit = "per_s", source = "kv:modifier_item_moon_mirror MODIFIER_PROPERTY_MANA_REGEN_CONSTANT" },
          cooldown = { 45, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          heal = { 300, unit = "heal", source = "kv:modifier_arc_moon_mirror HealAmount", note = "modifier_arc_moon_mirror OnCreated Heal" },
          jump_count = { 8, unit = "count", source = "kv:jump_count" },
          jump_delay = { 0.1, unit = "s", source = "kv:jump_delay" },
          radius = { 650, unit = "dist", source = "kv:radius" },
          lockout = { 40, unit = "s", source = "kv:heal_cd" },
          marker_duration = { expr = "$jump_count * $jump_delay", unit = "s", hidden = true, source = "derived",
            note = "本次施放的「已命中」標記只需存活到彈跳結束" },
        },
        text = {
          zh_TW = { name = "月輪鏡",
            body = "光環：{aura_radius} 內的友方英雄與單位（含自己）每秒回復 {aura_hp_regen} 生命與 {aura_mana_regen} 法力，多面不疊加。主動：對友方單位施放治療光波，回復 {heal} 生命，之後每 {jump_delay} 秒跳向 {radius} 內最近、這一次還沒跳過的友方單位，最多 {jump_count} 個單位。被治療的單位 {lockout} 秒內不能再被月輪鏡治療；光波跳到這種單位就會停止，第一個目標已在冷卻中則整個效果不發生。光波離開一個單位時，該單位 {lockout} 秒內也不能受靈都宮燈治療（最後一個被治療的單位不會）。" },
          en = { name = "Moon Mirror",
            body = "Aura: allied heroes and units within {aura_radius} (including you) regenerate {aura_hp_regen} health and {aura_mana_regen} mana per second; multiple mirrors do not stack. Active: sends a healing wave to an allied unit that restores {heal} health, then every {jump_delay} seconds jumps to the nearest ally within {radius} not yet hit by this cast, up to {jump_count} units. Healed units cannot be healed by a Moon Mirror again for {lockout} seconds; the wave stops when it jumps to such a unit, and does nothing if the first target is locked out. When the wave leaves a unit, that unit also cannot be healed by a Reito Palace Lantern for {lockout} seconds (the last unit healed is spared)." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_tea_bowl" }, recipe_cost = "$recipe_cost" } },
        unit_state = {
          moon_mirror_cur = { type = "unit", reveal = "public", scope = "item_instance", note = "原作 ability.target[instance]" },
          moon_mirror_live = { type = "bool", init = "false", reveal = "public", scope = "item_instance" },
        },
        passive = {
          aura = { radius = "$aura_radius", team = "ally", types = BASIC, flags = { "include_magic_immune" },
            status = "moon_mirror_aura", include_self = true, unique_group = "moon_mirror_aura" },
        },
        active = { kind = "active",
          target = { type = "unit", range = "$cast_range", team = "ally", types = BASIC, flags = { "include_magic_immune" } },
          cast = { cast_point = "$cast_point" },
          cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
          events = { on_cast = {
            { op = "cue", id = "moon_mirror_cast", at = "caster" },
            { op = "if", cond = "not has_status(target, moon_mirror_lockout)", actions = {
              { op = "heal", to = "target", amount = "$heal" },
              { op = "apply_status", status = "moon_mirror_lockout", to = "target" },
              { op = "apply_status", status = "moon_mirror_lantern_lock", to = "target" },
              { op = "apply_status", status = "moon_mirror_hit", to = "target" },
              { op = "set_state", var = "moon_mirror_cur", value = "target" },
              { op = "set_state", var = "moon_mirror_live", value = "true" },
              { op = "repeat", interval = "$jump_delay", first = "$jump_delay", count = "$jump_count - 1",
                stop_when = "not state.moon_mirror_live", actions = {
                  { op = "set_state", var = "moon_mirror_live", value = "false" },
                  { op = "for_each",
                    select = { from = "state.moon_mirror_cur", shape = "circle", radius = "$radius", team = "ally", types = BASIC,
                      flags = { "include_magic_immune" }, where = "not has_status(it, moon_mirror_hit)", order = "nearest", limit = 1 },
                    actions = {
                      { op = "cue", id = "moon_mirror_wave", at = "state.moon_mirror_cur", to = "it" },
                      { op = "if", cond = "not has_status(it, moon_mirror_lockout)", actions = {
                        { op = "heal", to = "it", amount = "$heal" },
                        { op = "apply_status", status = "moon_mirror_lockout", to = "it" },
                        { op = "apply_status", status = "moon_mirror_lantern_lock", to = "it" }, -- FC-128（D13）：冷卻掛在這次被治療的人身上
                        { op = "apply_status", status = "moon_mirror_hit", to = "it" },
                        { op = "set_state", var = "moon_mirror_live", value = "true" },
                      } },
                      { op = "set_state", var = "moon_mirror_cur", value = "it" },
                    } },
                } },
            } },
          } } },
        statuses = {
          { id = "moon_mirror_aura", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", unique_group = "moon_mirror_aura", unique_rule = "latest",
            stats = { hp_regen = "$aura_hp_regen", mana_regen = "$aura_mana_regen" },
            text = { zh_TW = { name = "月輪鏡", body = "每秒回復 {aura_hp_regen} 生命與 {aura_mana_regen} 法力。" }, en = { name = "Moon Mirror", body = "Regenerates {aura_hp_regen} health and {aura_mana_regen} mana per second." } },
            notes = "原作 modifier_item_moon_mirror（光環子狀態，非 MULTIPLE）" },
          { id = "moon_mirror_lockout", duration = "$lockout", stacking = "refresh", dispel = "none", visibility = "debuff",
            reveal = "public",
            text = { zh_TW = { name = "月輪鏡治療冷卻", body = "不能再被月輪鏡治療。" }, en = { name = "Mirror Heal Lockout", body = "Cannot be healed by a Moon Mirror again." } },
            notes = "原作 modifier_arc_moon_mirror_cd：IsDebuff、IsPurgable 0" },
          { id = "moon_mirror_lantern_lock", duration = "$lockout", stacking = "refresh", dispel = "none", visibility = "debuff",
            reveal = "public", tags = { "reito_lantern_lockout" },
            text = { zh_TW = { name = "宮燈治療冷卻", body = "不能受靈都宮燈治療。" }, en = { name = "Lantern Heal Lockout", body = "Cannot be healed by a Reito Palace Lantern." } },
            notes = "原作 modifier_arc_reito_lantern_cd（本道具也宣告同名 modifier）；靈都宮燈（pilot_a）的同名狀態也帶 tag reito_lantern_lockout，並以 has_status_tag 判斷" },
          { id = "moon_mirror_hit", duration = "$marker_duration", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", notes = "原作 unit.hit[instance]：本次施放已跳過的單位" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/items/Addon_Items/item_moon_mirror.lua:79-83",
            text = "FC-128（D13 修正：宮燈冷卻掛在這次被治療的人身上，最後一位也有）。原作：跳到新目標時，modifier_arc_reito_lantern_cd 掛在「上一個」目標（target）而不是新目標：除了第一個目標在施放時就掛上，每一跳把宮燈冷卻補給前一個人，最後一個被治療的人不會有宮燈冷卻。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_moon_mirror.lua:79-87,109-113",
            text = "下一個目標身上有月輪鏡治療冷卻時不套 modifier_arc_moon_mirror，彈跳在這裡結束；第一個目標在冷卻中時整個效果不發生（仍扣法力、進冷卻）。與 pilot 靈都宮燈同一套程式。" },
          { policy = "no_effect", ref = "vscripts/items/Addon_Items/item_moon_mirror.lua:94-98",
            text = "Jump 最後以 GetAbilityDamage() 治療，KV 沒有 AbilityDamage，回 0；實際的 300 來自 modifier_arc_moon_mirror 的 Heal。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_moon_mirror.lua:48-69",
            text = "選下一跳用「距離 < radius」、以目前目標位置為圓心、MAGIC_IMMUNE 旗標；可以跳回施放者本人。unit.hit 以施放編號區隔、永不清除，v3 改成存活到彈跳結束的隱藏標記，結果相同（同時兩次施放的差異見 pilot 靈都宮燈）。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:13287-13298",
            text = "宮燈冷卻以 tag reito_lantern_lockout 表達；pilot_a 的靈都宮燈以 has_status_tag 判斷，所以會擋下（跨文件引用，登錄於 status_tags）。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:13299-13320", text = "Passive_item_moon_mirror 屬性全 0，沒有效果。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:13207", text = "KV ItemCost 2100，零件＋合成費 1800。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:13184-13349", "npc/npc_items_custom_load.txt:9275-9287",
          "vscripts/items/Addon_Items/item_moon_mirror.lua:1-137", "shops/nobu_shops.txt:149" } },

      -------------------------------------------------------------------------
      { id = "item_moon_spear", source_id = "item_moon_spear",
        values = {
          cost = { 1950, unit = "gold", v2_key = "cost",
            source = "kv:ItemCost" .. at(4814) .. "（= item_naginata + item_spear + item_strength_ingot + 合成費）" },
          recipe_cost = { 200, unit = "gold", source = "kv:item_recipe_moon_spear ItemCost" .. at(8667) },
          attack = { 21, unit = "dmg", v2_key = "atk", source = "kv:MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" .. at(4831) },
          str = { 15, unit = "stat", v2_key = "modifiers.stats_strength_bonus",
            source = "kv:MODIFIER_PROPERTY_STATS_STRENGTH_BONUS" .. at(4828) },
        },
        text = { zh_TW = { name = "斬月槍" }, en = { name = "Moon Chopper" } },
        shop = { cost = "$cost", recipe = { components = { "item_naginata", "item_spear", "item_strength_ingot" }, recipe_cost = "$recipe_cost" } },
        passive = { stats = { attack_damage = "$attack", str = "$str" } },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/npc_items_custom_load.txt:4804-4808",
            text = "KV 的目標類型、傷害類型只是樣板；modifier 名稱沿用 Passive_lightning_katana（MULTIPLE）。" },
        },
        provenance = { KV .. ":4798-4842", KV .. ":8663-8675", "shops/nobu_shops.txt:126" } },

      -------------------------------------------------------------------------
      { id = "item_nagamaki", source_id = "item_nagamaki",
        values = {
          cost = { 2150, unit = "gold", kv = { 2100 },
            source = "kv:item_recipe_nagamaki ItemCost + 零件 ItemCost（item_spear×2、item_strength_ingot）",
            source_differs = "KV ItemCost 是 2100；Dota 實際售價是 700＋700＋350＋合成費 400" },
          recipe_cost = { 400, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:8618", note = "item_recipe_nagamaki ItemCost" },
          str = { 5, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_STRENGTH_BONUS" },
          attack = { 28, unit = "dmg", source = "kv:MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" },
          cooldown = { 33, unit = "s", source = "kv:AbilityCooldown" },
          hp_cost = { 10, unit = "pct", source = "script:vscripts/items/Addon_Items/item_nagamaki.lua:5-9 max_hp * 0.1" },
          attack_speed = { 95, unit = "atk_speed", source = "kv:modifier_nagamaki MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" },
          duration = { 10, unit = "s", source = "script:vscripts/items/Addon_Items/item_nagamaki.lua:10" },
        },
        text = {
          zh_TW = { name = "長捲",
            body = "主動：犧牲最大生命的 {hp_cost}，攻擊速度提高 {attack_speed}，持續 {duration} 秒。生命不足以支付時不會發動，冷卻也立即結束。" },
          en = { name = "Nagamaki",
            body = "Active: sacrifice {hp_cost} of maximum health to gain +{attack_speed} attack speed for {duration} seconds. If you cannot afford the health, nothing happens and the cooldown ends immediately." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_spear", "item_spear", "item_strength_ingot" }, recipe_cost = "$recipe_cost" } },
        passive = { stats = { str = "$str", attack_damage = "$attack" } },
        active = { kind = "active", target = { type = "none" },
          cast = { immediate = true },
          cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
          events = { on_cast = {
            { op = "if", cond = "caster.hp - caster.max_hp * $hp_cost <= 1",
              actions = { { op = "refund", cooldown = true }, { op = "stop" } } },
            { op = "set_health", to = "caster", value = "caster.hp - caster.max_hp * $hp_cost" },
            { op = "apply_status", status = "nagamaki_frenzy", to = "caster" },
          } } },
        statuses = {
          { id = "nagamaki_frenzy", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", stats = { attack_speed = "$attack_speed" },
            text = { zh_TW = { name = "長捲", body = "攻擊速度提高 {attack_speed}。" }, en = { name = "Nagamaki", body = "+{attack_speed} attack speed." } },
            notes = "原作 modifier_nagamaki（沒寫 IsPurgable → 暫定 basic）" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_nagamaki.lua:5-9",
            text = "扣血後剩不到一點（hp − 10% 最大生命 <= 1）時 EndCooldown 並結束；SetHealth 直接改血量，不是傷害，不觸發受傷事件。" },
          { policy = "note", text = "ATTACKSPEED_BONUS_CONSTANT 95 點；原作說明寫「95%」。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:4128-4188", "npc/npc_items_custom_load.txt:8618-8630",
          "vscripts/items/Addon_Items/item_nagamaki.lua:1-11", "shops/nobu_shops.txt:123" } },

      -------------------------------------------------------------------------
      { id = "item_naginata", source_id = "item_naginata",
        values = {
          cost = { 700, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(2604) },
          str = { 10, unit = "stat", v2_key = "modifiers.stats_strength_bonus",
            source = "kv:MODIFIER_PROPERTY_STATS_STRENGTH_BONUS" .. at(2623) },
        },
        text = { zh_TW = { name = "薙刀" }, en = { name = "Razor" } },
        shop = { cost = "$cost" },
        passive = { stats = { str = "$str" } },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/npc_items_custom_load.txt:2616", text = "modifier 名稱沿用 Passive_strength_ingot（MULTIPLE），只是命名。" },
        },
        provenance = { KV .. ":2589-2634", "shops/nobu_shops.txt:57" } },

      -------------------------------------------------------------------------
      { id = "item_nannbann_armor", source_id = "item_nannbann_armor",
        values = {
          cost = { 5500, unit = "gold", kv = { 5950 },
            source = "kv:item_recipe_nannbann_armor ItemCost + 零件 ItemCost（item_the_hooate_of_ghost×2 展開為 item_health_ingot + 合成費 450、item_protection_amulet、item_stable_shield）",
            source_differs = "KV ItemCost 是 5950；Dota 實際售價是 1350＋1350＋1250＋550＋合成費 1000" },
          recipe_cost = { 1000, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:9068", note = "item_recipe_nannbann_armor ItemCost" },
          hp = { 600, unit = "hp", source = "kv:Passive_nannbann_armor MODIFIER_PROPERTY_HEALTH_BONUS" },
          hp_regen = { 20, unit = "per_s", source = "kv:Passive_nannbann_armor MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT" },
          armor = { 5, unit = "armor", source = "kv:Passive_nannbann_armor MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" },
        },
        text = { zh_TW = { name = "南蠻胴具足" }, en = { name = "Nanbando Armor" } },
        shop = { cost = "$cost", recipe = { components = { "item_the_hooate_of_ghost", "item_the_hooate_of_ghost", "item_protection_amulet", "item_stable_shield" },
          recipe_cost = "$recipe_cost" } },
        passive = { stats = { max_hp = "$hp", hp_regen = "$hp_regen", armor = "$armor" } },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "no_effect", ref = "vscripts/items/Addon_Items/item_nannbann_armor.lua:39",
            text = "modifier_nannbann_armor 的 OnTakeDamage 條件寫了 `and false`，整段「受到非物理傷害時還原血量與法力、移除敵方狀態、給 500 點魔法護盾（modifier_nannbann_armor2，5 秒）、道具進 60 秒冷卻」永遠不執行；ShockTarget 只從這段呼叫，modifier_nannbann_purge 沒有人套。原作說明的「自動抵擋一次指定技能」沒有效果，說明不寫。" },
          { policy = "note", ref = "vscripts/items/Addon_Items/item_nannbann_armor.lua:103-142",
            text = "OnEquip 每 0.5 秒替真英雄補上 modifier_nannbann_armor 並顯示護盾粒子（particles/item/protection.vpcf）；modifier 本身只每 0.2 秒記錄血量與法力，沒有效果，粒子會讓玩家以為有護盾。" },
          { policy = "no_effect", ref = "npc/npc_items_custom_load.txt:11187", text = "AbilityCooldown 60 只在死碼裡被 StartCooldown 使用。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:11192", text = "KV ItemCost 5950，零件＋合成費 5500。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:11175-11256", "npc/npc_items_custom_load.txt:9068-9080",
          "npc/npc_items_custom_load.txt:9038-9050", "vscripts/items/Addon_Items/item_nannbann_armor.lua:1-255", "shops/nobu_shops.txt:187" } },

      -------------------------------------------------------------------------
      { id = "item_napalm_bomb", source_id = "item_napalm_bomb",
        values = {
          cost = { 150, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(15043) },
          cooldown = { 10, unit = "s", v2_key = "cooldown", source = "kv:AbilityCooldown" .. at(15035) },
          cast_range = { 600, unit = "dist", v2_key = "target.range", source = "kv:AbilityCastRange" .. at(15037) },
          radius = { 300, unit = "dist", v2_key = "active.radius",
            source = "script:" .. SCR .. "item_napalm_bomb.lua:15 SEARCH_RADIUS（KV AOERadius" .. at(15038) .. " 相同）" },
          damage = { 300, unit = "dmg", damage_type = "magical", v2_key = "active.damage",
            source = "script:" .. SCR .. "item_napalm_bomb.lua:27 dmg" },
          charges = { 1, unit = "count", hidden = true, source = "kv:ItemInitialCharges", note = "census 行為用；現行 Rust handler 不讀" },
          stock_max = { 10, unit = "count", hidden = true, source = "kv:ItemStockMax", note = "census 行為用；現行 Rust handler 不讀" },
          stock_initial = { 0, unit = "count", hidden = true, source = "kv:ItemStockInitial", note = "census 行為用；現行 Rust handler 不讀" },
          restock_time = { 0.01, unit = "s", hidden = true, source = "kv:ItemStockTime", note = "census 行為用；現行 Rust handler 不讀" },
          first_restock = { 210, unit = "s", hidden = true, source = "kv:ItemInitialStockTime", note = "census 行為用；現行 Rust handler 不讀" },
        },
        text = {
          zh_TW = { name = "爆烈彈",
            body = "使用：對指定地點投擲爆裂彈，範圍內的敵方單位（含中立，不含建築）受到 {damage} 魔法傷害。傷害不受法術增幅加成。使用後消耗。" },
          en = { name = "Explosive Bomb",
            body = "Active: throws a bomb at the target point. Enemy units in the area (neutral creeps included, buildings excluded) take {damage} magic damage. The damage is not increased by spell amplification. Consumed on use." },
        },
        shop = { cost = "$cost", consumable = { remove_when_empty = true, charges = "$charges" },
          shareable = true,
          stock = { max = "$stock_max", initial = "$stock_initial", restock = "$restock_time", first_restock = "$first_restock" },
        },
        active = { kind = "active", target = { type = "point", range = "$cast_range", aoe_radius = "$radius" }, cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
          events = { on_cast = {
            { op = "cue", id = "snapjaw_rocket_explosion", at = "point" },
            { op = "destroy_trees", at = "point", radius = "$radius" },
            { op = "for_each",
              select = { from = "point", shape = "circle", radius = "$radius", team = "enemy", types = { "all" },
                exclude_types = { "building" }, flags = { "include_magic_immune" } },
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$damage" },
                { op = "cue", id = "a17t_sound", at = "it" },
              } },
          } },
        },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "port_adaptation", ref = KV .. ":15050-15053",
            text = "原作商店庫存 ItemStockMax 10、ItemInitialStockTime 210（說明「遊戲開始3分30秒後才能使用」）、ItemStockTime 0.01；道具列沒有庫存，現行開局就能購買、沒有庫存上限。" },
          { policy = "as_source", ref = SCR .. "item_napalm_bomb.lua:17-34",
            text = "FindUnitsInRadius TEAM_ENEMY 含中立；帶 MAGIC_IMMUNE_ENEMIES，但魔法傷害對魔免單位依規則為 0。傷害是 AMHC:Damage，不吃法術增幅（E178、E188）。" },
          { policy = "note", ref = SCR .. "item_napalm_bomb.lua:16",
            text = "DestroyTreesAroundPoint 在本遊戲地圖沒有可破壞的樹，沒有作用（E178）。" },
          { policy = "port_adaptation", ref = KV .. ":15036",
            text = "原作 AbilityCastPoint 0.2；道具列沒有前搖欄位，現行施放沒有前搖。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:15065-15073",
            text = "SpendCharge 在 0.1 秒後才執行；v3 由 consumable 在使用時扣次數，結果相同。" },
          { policy = "note", ref = "vscripts/items/Addon_Items/item_napalm_bomb.lua:8-13,26,32", text = "三個爆炸粒子與 A17T 音效屬呈現，改用 cue。" },
          { policy = "note", ref = "shops/nobu_shops.txt:11,38", text = "同時列在兩個商店分頁。" },
        },
        provenance = { KV .. ":15022-15076", SCR .. "item_napalm_bomb.lua:1-35", "shops/nobu_shops.txt:11" } },

      -------------------------------------------------------------------------
      { id = "item_ninja2", source_id = "item_ninja2",
        -- 中忍的追蹤欄位（原作 caster.ninja2）：英雄層級，召喚與中忍死亡時讀寫；只有本道具使用，以 ninja2_ 前綴私有
        unit_state = {
          ninja2_unit = { type = "unit", reveal = "team", scope = "hero", note = "原作 caster.ninja2（被追蹤的那名中忍）" },
          ninja2_tracked = { type = "bool", init = "false", reveal = "team", scope = "hero",
            note = "caster.ninja2 是否為 nil；原作任何一名中忍死亡時 update_ninja2 設回 nil" },
          ninja2_current = { type = "bool", init = "false", reveal = "team", scope = "unit",
            note = "FC-094（D13）：中忍自己是不是主人目前追蹤的那名；被頂替時先設回 false，它的死亡就不會清掉新中忍的追蹤" },
        },
        values = {
          cost = { 100, unit = "gold", source = "kv:ItemCost" },
          cooldown = { 1, unit = "s", source = "kv:AbilityCooldown" },
          cast_range = { 400, unit = "dist", source = "kv:AbilityCastRange" },
          charges = { 1, unit = "count", source = "kv:ItemInitialCharges" },
          stock = { 5, unit = "count", source = "kv:ItemStockMax", note = "ItemStockInitial 相同" },
          restock_time = { 15, unit = "s", source = "kv:ItemStockTime" },
          lifetime = { 240, unit = "s", source = "script:vscripts/items/Addon_Items/item_ninja.lua:51 modifier_kill duration" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/items/Addon_Items/item_ninja.lua:50" },
        },
        text = {
          zh_TW = { name = "中忍",
            body = "消耗品：在目標地點召喚一名可控制的{@ninja_unit2}，持續 {lifetime} 秒。中忍不受傷害、每被攻擊一次失去一點生命，沒有碰撞、不能被技能指定，能看穿周圍的隱形敵人，並可使用{@ninja2_burrow}。每位玩家最多同時存在一名中忍：再召喚時舊的那名會消失。商店每 {restock_time} 秒補一件，可與隊友共用。" },
          en = { name = "Chunin",
            body = "Consumable: summons a controllable {@ninja_unit2} at the target point for {lifetime} seconds. The Chunin takes no damage but loses one health each time it is attacked, has no collision, cannot be targeted by spells, reveals nearby invisible enemies and can use {@ninja2_burrow}. Each player can have only one Chunin at a time: summoning another makes the old one vanish. The shop restocks one every {restock_time} seconds; shareable with allies." },
        },
        shop = { cost = "$cost", stock = { max = "$stock", initial = "$stock", restock = "$restock_time" }, shareable = true,
          consumable = { charges = "$charges", remove_when_empty = true } },
        active = { kind = "active",
          target = { type = "point", range = "$cast_range" },
          cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
          events = { on_cast = {
            { op = "create_unit", template = "ninja_unit2", at = "point", owner = "caster", control = "owner", as = "u" },
            { op = "apply_status", status = "ninja2_spawn_phased", to = "u" },
            { op = "apply_status", status = "ninja2_lifetime", to = "u" },
            -- FC-094（D13）：一人最多同時一名中忍，舊的那名消失（原作移除舊中忍的 modifier_kill，讓它永久留下、新中忍不被追蹤）
            { op = "if", cond = "state.ninja2_tracked and exists(state.ninja2_unit) and alive(state.ninja2_unit)", actions = {
              { op = "set_state", on = "state.ninja2_unit", var = "ninja2_current", value = "false" },
              { op = "kill", to = "state.ninja2_unit", silent = true },
            } },
            { op = "set_state", var = "ninja2_unit", value = "u" },
            { op = "set_state", var = "ninja2_tracked", value = "true" },
            { op = "set_state", on = "u", var = "ninja2_current", value = "true" },
          } } },
        statuses = {
          { id = "ninja2_spawn_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, notes = "原作 modifier_phased 0.1 秒" },
          { id = "ninja2_lifetime", duration = "$lifetime", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public",
            triggers = { on_expired = { { op = "kill", to = "self" } } },
            text = { zh_TW = { name = "召喚時限", body = "時間到時死亡。" }, en = { name = "Summon Timer", body = "Dies when this expires." } },
            notes = "原作 modifier_kill（240 秒）；原作超額時被 RemoveModifierByName 移除（FC-094 修正為舊中忍直接消失）" },
          { id = "ninja2_passive", duration = "infinite", stacking = "keep", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision", "untargetable_by_spells" },
            values = {
              damage_block = { 100, unit = "pct", kv = { -100 }, source = "kv:MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE@npc/npc_abilities_custom_load.txt:6367", source_differs = "KV 以負的受傷百分比表示減免；v3 寫減免量（正值）", note = "中忍單位技能 ninja_takedamage 的 passive_ninja_takedamage" },
              hp_per_attack = { 1, unit = "hp", source = "script:vscripts/library/common/dummy.lua:1016-1021 SetHealth(health - 1)" },
              sight_radius = { 900, unit = "dist", source = "kv:Aura_Radius@npc/npc_abilities_custom_load.txt:1140", note = "中忍單位技能 ninja_true_sight 的 Passive_insight_gem_shard" },
            },
            stats = { incoming_damage_pct = "-$damage_block" },
            aura = { radius = "$sight_radius", team = "enemy", types = { "hero", "creep", "neutral", "summon", "building" },
              flags = { "include_magic_immune", "include_invulnerable" }, status = "ninja2_true_sight_reveal" },
            triggers = {
              on_attacked_hit = { { op = "set_health", to = "self", value = "self.hp - $hp_per_attack" } },
              on_death = { { op = "if", cond = "state.ninja2_current", actions = {
                { op = "set_state", var = "ninja2_tracked", value = "false", on = "self.owner" } } } },
            },
            text = {
              zh_TW = { name = "中忍體質", body = "沒有碰撞、不能被技能指定，不受任何傷害（減免 {damage_block}）；每被攻擊命中一次失去 {hp_per_attack} 點生命。看穿 {sight_radius} 內的隱形敵人。" },
              en = { name = "Chunin Body", body = "No collision, cannot be targeted by spells, takes no damage ({damage_block} reduction); loses {hp_per_attack} health each time an attack hits it. Reveals invisible enemies within {sight_radius}." },
            },
            notes = "原作 for_no_collision、ninja_takedamage、ninja_true_sight、update_buy_ninja2 四個被動合併；不能被技能指定來自 dummy.lua _G.EXCLUDE_TARGET_NAME（規則層）" },
          { id = "ninja2_true_sight_reveal", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "revealed" },
            notes = "原作光環子狀態 modifier_insight_gem_truesight：OnCreated 對目標套引擎 modifier_truesight、OnDestroy 移除" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/items/Addon_Items/item_ninja.lua:53-61",
            text = "FC-094（D13 修正：一人最多同時一名中忍：再召喚時舊的那名消失，新中忍被追蹤；只有被追蹤的中忍死亡才清空追蹤）。原作：追蹤中的中忍還活著時再召喚，會移除舊中忍的 modifier_kill（永久存在）而且 caster.ninja2 不更新成新的；之後只要舊的還活著，新召喚的都是普通四分鐘中忍。任何一名中忍死亡（update_ninja2 不看是哪一名）就清空追蹤，下一次召喚又成為被追蹤者，再下一次又讓它永久化：反覆購買可以累積多名永久中忍。原作說明寫「一人最多同時存在一隻」。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/tofar_goback.lua:655-659",
            text = "update_ninja2 在中忍死亡時把 owner.ninja2 設為 nil；以 bool 的 ninja2_tracked 表達 nil（unit 變數沒有空值字面值）。IsValidEntity 失敗的分支只在中忍已被移除但未觸發死亡時才會走到，與 exists() 一致。" },
          { policy = "as_source", ref = "npc/npc_abilities_custom_load.txt:1135-1176",
            text = "看穿用的光環與 I1 洞察碎片同一套（modifier_insight_gem_truesight），光環子狀態結束時移除任意一個 modifier_truesight 的問題相同（見 I1 fix_candidate）。" },
          { policy = "as_source", ref = "vscripts/library/common/dummy.lua:1016-1021",
            text = "受到傷害 -100%，每次被攻擊以 SetHealth(hp - 1) 扣一點，五次後生命為零；SetHealth(0) 是否直接致死需實機確認。datadriven OnAttacked 推定只在命中時觸發（pilot 下忍寫成 on_attacked，兩者待 M2 統一）。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:17302-17310",
            text = "SpendCharge 在 0.1 秒後才執行；v3 由 consumable 在使用時扣次數，結果相同。ITEM_FULLY_SHAREABLE_STACKING：可堆疊、可與隊友共用。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:17279,17294", text = "AbilityCastRange 400 重複宣告。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:17273-17312", "vscripts/items/Addon_Items/item_ninja.lua:42-61",
          "npc/npc_units_custom_load.txt:2281-2348", "npc/npc_abilities_custom_load.txt:1089-1180",
          "npc/npc_abilities_custom_load.txt:5247-5263", "npc/npc_abilities_custom_load.txt:6347-6371",
          "vscripts/items/Addon_Items/tofar_goback.lua:655-659", "vscripts/library/common/dummy.lua:331-375",
          "vscripts/library/common/dummy.lua:1016-1021", "shops/nobu_shops.txt:18" } },

      -------------------------------------------------------------------------
      { id = "item_ninja_sword", source_id = "item_ninja_sword",
        values = {
          cost = { 3320, unit = "gold", source = "kv:ItemCost（= item_katana 1120 + item_tachi 700 + 合成費）" },
          recipe_cost = { 1500, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:8753", note = "item_recipe_ninja_sword ItemCost" },
          agi = { 10, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_AGILITY_BONUS" },
          attack = { 14, unit = "dmg", source = "kv:MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" },
          attack_speed = { 20, unit = "atk_speed", source = "kv:MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" },
          cooldown = { 80, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 6, unit = "s", source = "kv:modifier_ninja_sword Duration", note = "modifier_ninja_sword；item_ninja_sword.lua:6-7 相同" },
          move_speed_pct = { 15, unit = "pct", source = "kv:modifier_ninja_sword MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE" },
        },
        text = {
          zh_TW = { name = "忍者刀",
            body = "主動：施展迷蹤步，隱形並提高 {move_speed_pct} 移動速度、無視單位碰撞，最多持續 {duration} 秒；發動攻擊或使用技能、道具時結束。" },
          en = { name = "Ninjato",
            body = "Active: use Vanishing Step to turn invisible, gain +{move_speed_pct} movement speed and ignore unit collision for up to {duration} seconds; ends when you attack or use an ability or item." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_katana", "item_tachi" }, recipe_cost = "$recipe_cost" } },
        passive = { stats = { agi = "$agi", attack_damage = "$attack", attack_speed = "$attack_speed" } },
        active = { kind = "active", target = { type = "none" },
          cast = { immediate = true },
          cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
          events = { on_cast = {
            { op = "apply_status", status = "ninja_sword_invisible", to = "caster" },
            { op = "apply_status", status = "ninja_sword_walk", to = "caster" },
            { op = "cue", id = "powershot_falcon_bow", at = "caster" },
          } } },
        statuses = {
          { id = "ninja_sword_invisible", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", tags = { "invisible", "engine_invisible" }, states = { "invisible" },
            text = { zh_TW = { name = "隱形", body = "隱形；發動攻擊或使用技能、道具時結束。" }, en = { name = "Invisible", body = "Invisible; ends when you attack or use an ability or item." } },
            notes = "原作引擎 modifier_invisible（AddNewModifier，6 秒）；可否驅散未知，暫定 none" },
          { id = "ninja_sword_walk", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", states = { "no_unit_collision" }, stats = { move_speed_pct = "$move_speed_pct" },
            cue = "nyx_vendetta_start",
            text = { zh_TW = { name = "迷蹤步", body = "移動速度提高 {move_speed_pct}，無視單位碰撞；發動攻擊或使用技能、道具時結束。" }, en = { name = "Vanishing Step", body = "+{move_speed_pct} movement speed and no unit collision; ends when you attack or use an ability or item." } },
            triggers = {
              on_attack_launch = {
                { op = "if", cond = "not event.target.is_building", actions = {
                  { op = "cue", id = "nyx_vendetta_strike", at = "self", to = "event.target" } } },
                { op = "remove_status", from = "self", status = "ninja_sword_walk" },
                { op = "remove_status", from = "self", status = "ninja_sword_invisible" },
              },
              on_ability_executed = { who = "self", actions = {
                { op = "remove_status", from = "self", status = "ninja_sword_walk" },
                { op = "remove_status", from = "self", status = "ninja_sword_invisible" },
              } },
            },
            notes = "原作 modifier_ninja_sword（沒寫 IsPurgable → 暫定 basic）：OnAttack → Shock2、OnAbilityExecuted 移除自己與 modifier_invisible" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:7532",
            text = "KV 移速 +15%，原作說明寫 10%；照 KV。" },
          { policy = "no_effect", ref = "vscripts/items/Addon_Items/item_ninja_sword.lua:24-25",
            text = "Shock2 的 350 物理傷害已註解；出手時只有粒子與音效（非建築目標），隨即移除兩個 modifier。原作說明本來就沒寫破隱傷害。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:7509-7529",
            text = "datadriven OnAttack 是出手時（on_attack_launch）；OnAbilityExecuted 包含使用道具。只結束本道具的迷蹤步與 modifier_invisible；迷蹤步被驅散時隱形不會跟著消失。" },
          { policy = "note", ref = "vscripts/items/Addon_Items/item_ninja_sword.lua:12",
            text = "`not IsUnselectable() or IsUnselectable()` 永遠成立，只用來在目標是道具時讓腳本報錯。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:7436-7540", "npc/npc_items_custom_load.txt:8753-8765",
          "vscripts/items/Addon_Items/item_ninja_sword.lua:1-30", "shops/nobu_shops.txt:205" } },

      -------------------------------------------------------------------------
      { id = "item_pleated_skirt", source_id = "item_pleated_skirt",
        values = {
          cost = { 500, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(2794) },
          move_speed = { 35, unit = "move_speed", v2_key = "unique_passive.ms", source = "kv:MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE" .. at(2814) },
          priority = { 35, unit = "number", hidden = true, v2_key = "unique_passive.priority",
            source = "kv:MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE" .. at(2814), note = "唯一移速以數值本身當優先度，多件只取最高（E183）" },
        },
        text = {
          zh_TW = { name = "褶裙", body = UNIQUE_MS_ZH },
          en = { name = "Pleated Skirt", body = UNIQUE_MS_EN },
        },
        shop = { cost = "$cost" },
        passive = {
          triggers = { on_created = { { op = "apply_status", status = "pleated_skirt_unique_ms", to = "self" } } },
        },
        statuses = {
          { id = "pleated_skirt_unique_ms", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "move_speed_unique", dispel = "none", visibility = "hidden", reveal = "public", remove_on = { "unequip" },
            stats = { move_speed = "$move_speed" },
            notes = "原作 Passive_item_pleated_skirt1（MULTIPLE）的 MOVESPEED_BONUS_UNIQUE：Dota 只取所有唯一移速來源的最高值。" },
        },
        coverage = "rust_handler", handler = handler({ "$priority" }), confidence = "high",
        quirks = {
          { policy = "note", ref = "shops/nobu_shops.txt:24,61", text = "同時列在偷買（pregame）與基礎道具兩個商店分頁。" },
          { policy = "as_source",
            text = "移動速度只由唯一狀態提供（原作 MOVESPEED_BONUS_UNIQUE 只取所有唯一來源的最高值），攻擊與攻速等其他屬性照常相加；passive.stats 不再重寫移速，也不再把整組被動設成 unique_group（D13 決定，2026-10-29）。" },
        },
        provenance = { KV .. ":2780-2818", "shops/nobu_shops.txt:24" } },

      -------------------------------------------------------------------------
      { id = "item_poisonous_ring", source_id = "item_poisonous_ring",
        values = {
          cost = { 0, unit = "gold", source = "assumed:原作 KV 沒有這個道具（也沒有合成表）；Dota 商店不會顯示，暫填 0" },
        },
        text = { zh_TW = { name = "毒之戒指" }, en = { name = "Poison Ring" } },
        shop = { cost = "$cost", purchasable = false },
        coverage = "deferred", deferred_reason = "not_in_source", confidence = "high",
        notes = "商店清單有 item_poisonous_ring（毒之戒指），但 npc_items_custom_load.txt 沒有道具定義（shops/nobu_shops.txt:251；只有 vscripts/items/Addon_Items/item_the_great_sword_of_toxic.lua:76-82 的殘留函式 Shock_poisonous_ring（套 modifier_poisonous_ring，KV 也沒有這個 modifier））。依主 session D13 決定（原作不存在的道具不移植），不移植。",
        provenance = { "shops/nobu_shops.txt:251", "vscripts/items/Addon_Items/item_the_great_sword_of_toxic.lua:76-82" } },

      -------------------------------------------------------------------------
      { id = "item_power_ingot", source_id = "item_power_ingot",
        values = {
          cost = { 300, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(2415) },
          attack = { 5, unit = "dmg", v2_key = "atk", source = "kv:MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" .. at(2437) },
        },
        text = { zh_TW = { name = "強化鋼" }, en = { name = "Steel of Intensity" } },
        shop = { cost = "$cost" },
        passive = { stats = { attack_damage = "$attack" } },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "note", ref = KV .. ":2427", text = "modifier 名稱沿用 Passive_strength_ingot（MULTIPLE），其餘屬性都是 0，只是命名。" },
        },
        provenance = { KV .. ":2401-2444", "shops/nobu_shops.txt:82" } },
    },
  }
end
