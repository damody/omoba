-- Source item catalog vs the game's items (E155): what matches, what the game invented, and
-- what the effect engine cannot express yet.
--
--   tools\lua\lua.exe scripts\item_source_compare.lua [--out <md>]
--
-- "Representable" means every non-zero source property maps to a field the item compiler
-- supports today, any active translates (item_source/active.lua, E160), and no source script
-- outside that active (whose effect would be unmodelled). Anything else is needs_framework,
-- with the missing capabilities named.
local source_file = debug.getinfo(1, "S").source:sub(2)
package.path = assert(source_file:match("^(.*)[/\\]")) .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local path = b.lib("path")
local active = require("item_source.active")
local M = {}

M.CATALOG = path.join(b.root, "docs", "plans", "2026-10-24-omfue-month-mvp", "source-index", "source-item-catalog.lua")
M.OUT = path.join(b.root, "docs", "plans", "2026-10-24-omfue-month-mvp", "source-index", "item-compare.md")

-- Source property -> item compiler field supported today.
M.SUPPORTED = {
  MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE = "atk",
  MODIFIER_PROPERTY_HEALTH_BONUS = "hp",
  MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS = "armor",
}

-- Source property -> passive stat modifier (BuffStore key, scale to the runtime reader's unit),
-- for keys whose reader matches the Dota meaning (E155 C). Percentages become fractions.
M.MODIFIERS = {
  MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT = { "attackspeed_bonus_constant", 1 },
  MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT = { "health_regen_constant", 1 },
  MODIFIER_PROPERTY_MANA_REGEN_CONSTANT = { "mana_regen_constant", 1 },
  MODIFIER_PROPERTY_MANA_BONUS = { "mana_bonus", 1 },
  MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE = { "movespeed_bonus_percentage", 0.01 },
  MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE = { "basedamageoutgoing_percentage", 0.01 },
  MODIFIER_PROPERTY_EVASION_CONSTANT = { "evasion_constant", 0.01 },
  MODIFIER_PROPERTY_MAGICAL_RESISTANCE_BONUS = { "magical_resistance_bonus", 1 },
  MODIFIER_PROPERTY_ATTACK_RANGE_BONUS = { "attack_range_bonus", 1 },
  -- E188: skill and item damage +x% (every packet but attacks), summed over sources.
  MODIFIER_PROPERTY_SPELL_AMPLIFY_PERCENTAGE = { "spell_amplify_percentage", 0.01 },
  -- Attributes (templates/attribute_rules.lua turns them into HP, armor, mana, ...).
  MODIFIER_PROPERTY_STATS_STRENGTH_BONUS = { "stats_strength_bonus", 1 },
  MODIFIER_PROPERTY_STATS_AGILITY_BONUS = { "stats_agility_bonus", 1 },
  MODIFIER_PROPERTY_STATS_INTELLECT_BONUS = { "stats_intellect_bonus", 1 },
}

local function short(property) return (property:gsub("^MODIFIER_PROPERTY_", "")) end

-- AbilitySharedCooldown group -> sorted item keys, for the active translator. Callers index the
-- catalog once before asking for actives (M.index).
local groups
function M.index(catalog)
  groups = {}
  for _, it in ipairs(catalog.items) do
    if it.shared_cooldown then
      groups[it.shared_cooldown] = groups[it.shared_cooldown] or {}
      table.insert(groups[it.shared_cooldown], it.key)
    end
  end
  for _, list in pairs(groups) do table.sort(list) end
end

-- The item's translated active (or nil, reason); cached per catalog row.
local translations = setmetatable({}, { __mode = "k" })
function M.active(item)
  if not item.has_active then return nil, "no active" end
  assert(groups, "item_source_compare.index(catalog) must run before translating actives")
  local hit = translations[item]
  if not hit then
    local result, why = active.translate(item, groups)
    hit = { result = result, why = why }
    translations[item] = hit
  end
  return hit.result, hit.why
end

-- The item's passive aura (or nil, reason); cached per catalog row.
local auras = setmetatable({}, { __mode = "k" })
function M.aura(item)
  local hit = auras[item]
  if not hit then
    local result, why = active.aura(item)
    hit = { result = result, why = why }
    auras[item] = hit
  end
  return hit.result, hit.why
end

-- Properties the item holds itself: those of modifiers its active applies belong to the active,
-- and those of its aura (holder and aura modifiers) belong to the aura.
function M.passive_properties(item)
  local applied = item.has_active and (M.active(item) or {}).applied or {}
  local aura_mods = (M.aura(item) or {}).modifiers or {}
  local out = {}
  for _, p in ipairs(item.properties) do
    if not (p.modifier and (applied[p.modifier] or aura_mods[p.modifier])) then out[#out + 1] = p end
  end
  return out
end

-- Scripts the translated active accounts for (presentation or parameters it maps).
local function active_scripts(item)
  local set = {}
  local translated = M.active(item)
  if not translated then return set end
  -- A translated slow is the target modifier's dummy.lua Slow / ReturnSpeed pair (E188).
  if translated.active.slow then
    set["scripts/vscripts/library/common/dummy.lua|Slow"] = true
    set["scripts/vscripts/library/common/dummy.lua|ReturnSpeed"] = true
  end
  local actions = {}
  for _, action in ipairs(item.active_kv.on_spell_start or {}) do actions[#actions + 1] = action end
  for _, action in ipairs(item.active_kv.on_projectile_hit or {}) do actions[#actions + 1] = action end
  for _, action in ipairs(item.active_kv.on_toggle_on or {}) do actions[#actions + 1] = action end
  for _, action in ipairs(actions) do
    if action.key == "RunScript" then
      local file, fn
      for _, p in ipairs(action.children or {}) do
        if p.key == "ScriptFile" then file = p.value elseif p.key == "Function" then fn = p.value end
      end
      set[tostring(file) .. "|" .. tostring(fn)] = true
    end
  end
  return set
end

-- MOVESPEED_BONUS_UNIQUE (E183): Dota applies only the highest unique movespeed among the items a
-- unit holds. The game's unique passive group "movement" does the same (the highest priority
-- wins), so the value is both the bonus and the priority.
M.UNIQUE = { MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE = true }
function M.unique_passive(item)
  local best
  for _, p in ipairs(M.passive_properties(item)) do
    local value = tonumber(p.value)
    if M.UNIQUE[p.property] and value and value > 0 and (not best or value > best) then best = value end
  end
  if not best then return nil end
  return { group = "movement", name = item.name, priority = math.floor(best), ms = best }
end

-- Passive stat modifiers of a source item ({key = value}), summed over its properties.
function M.modifiers(item)
  local out = {}
  for _, p in ipairs(M.passive_properties(item)) do
    local map = M.MODIFIERS[p.property]
    local value = tonumber(p.value)
    if map and value and value ~= 0 then out[map[1]] = (out[map[1]] or 0) + value * map[2] end
  end
  return out
end

function M.total_cost(item, by_key)
  local recipe = item.recipe and item.recipe[1]
  if not recipe or not recipe.requirements[1] then return item.cost end
  local sum = recipe.cost
  for _, key in ipairs(recipe.requirements[1]) do
    local part = by_key[key]
    if not part then return nil end
    sum = sum + (M.total_cost(part, by_key) or 0)
  end
  return sum
end

-- Missing capabilities of one source item (empty when representable today). With `by_key`, a
-- recipe component that is not representable is a gap too ("COMPONENT:<key>"): the shop cannot
-- sell a recipe whose part it cannot sell, nor one whose part no shop lists.
function M.gaps(item, by_key)
  local gaps = {}
  local passive = M.passive_properties(item)
  for _, p in ipairs(passive) do
    if p.value ~= 0 and p.value ~= "0" and not M.SUPPORTED[p.property] and not M.MODIFIERS[p.property] and not M.UNIQUE[p.property] then
      gaps[#gaps + 1] = short(p.property)
    end
  end
  local translated = M.active(item)
  if item.has_active and not translated then gaps[#gaps + 1] = "ACTIVE" end
  local aura, aura_why = M.aura(item)
  if not aura and aura_why then gaps[#gaps + 1] = "AURA" end
  local covered = active_scripts(item)
  for _, s in ipairs(item.scripts) do
    if not covered[tostring(s.file) .. "|" .. tostring(s["function"])] then gaps[#gaps + 1] = "SCRIPT"; break end
  end
  -- No property, active or script, yet the description names an effect (an upgrade part).
  local any = translated ~= nil or aura ~= nil
  for _, p in ipairs(passive) do if p.value ~= 0 and p.value ~= "0" then any = true end end
  -- A pure upgrade part (E183, e.g. 破魔寶珠) has no effect of its own: what its text describes
  -- is the upgraded item's, which a recipe builds from it. Only an item no recipe uses is suspect.
  local part = false
  for _, other in pairs(by_key or {}) do
    for _, alt in ipairs((other.recipe and other.recipe[1] and other.recipe[1].requirements) or {}) do
      for _, key in ipairs(alt) do if key == item.key then part = true end end
    end
  end
  if not any and #gaps == 0 and not part then gaps[#gaps + 1] = "DESCRIPTION_ONLY" end
  local recipe = by_key and item.recipe and item.recipe[1]
  for _, key in ipairs(recipe and recipe.requirements[1] or {}) do
    local part = by_key[key]
    if not part or not part.listed or #M.gaps(part, by_key) > 0 then gaps[#gaps + 1] = "COMPONENT:" .. key end
  end
  local seen, out = {}, {}
  for _, g in ipairs(gaps) do if not seen[g] then seen[g] = true; out[#out + 1] = g end end
  return out
end

-- The game's items through the same includes the build uses. OM_ITEM_TEMPLATES points at a
-- workspace copy of scripts/lua_data/templates before it is landed.
function M.game_items()
  local root = os.getenv("OM_ITEM_TEMPLATES") or path.join(b.root, "scripts", "lua_data", "templates")
  local ctx
  ctx = { include = function(rel) return assert(loadfile(path.join(root, rel:match("templates/(.+)"))))()(ctx) end }
  return ctx.include("templates/moba_items.lua")
end

function M.render(catalog, game)
  local by_key, by_name = {}, {}
  M.index(catalog)
  for _, item in ipairs(catalog.items) do by_key[item.key] = item; if item.name then by_name[item.name] = item end end
  local lines = {"# 來源道具與遊戲道具對照（E155）", "",
    ("由 `tools\\lua\\lua.exe scripts\\item_source_compare.lua` 產生，不要手改。來源：history_nobu @ `%s`。"):format(catalog.source.commit), ""}
  -- Game items.
  lines[#lines + 1] = "## 目前遊戲道具"
  lines[#lines + 1] = ""
  lines[#lines + 1] = "| 遊戲 id | 名稱 | 價錢 | 對得上的來源 | 判定 |"
  lines[#lines + 1] = "|---|---|---|---|---|"
  local matched, on_sale = 0, 0
  for _, item in ipairs(game) do
    local ref = item.source_ref and by_key[(item.source_ref:match(":item_([%w_]+)$") or "")] or by_name[item.name]
    local verdict = ref and ("上架，來源 `item_" .. ref.key .. "`")
      or (item.test_only and "引擎測試 fixture（test_only，不上架）" or "來源以外自創")
    if ref then matched = matched + 1 end
    if not item.test_only then on_sale = on_sale + 1 end
    lines[#lines + 1] = ("| `%s` | %s | %d | %s | %s |"):format(item.id, item.name, item.cost,
      ref and (ref.name .. "（" .. tostring(M.total_cost(ref, by_key)) .. "）") or "—", verdict)
  end
  lines[#lines + 1] = ""
  lines[#lines + 1] = ("遊戲道具 %d 個：上架 %d 個（全部引用來源），對得上來源 %d 個，其餘為 test_only 引擎 fixture。"):format(
    #game, on_sale, matched)
  lines[#lines + 1] = ""
  -- Source items in shops.
  local representable, framework, gap_count = {}, {}, {}
  for _, item in ipairs(catalog.items) do
    if item.listed then
      local gaps = M.gaps(item, by_key)
      if #gaps == 0 then representable[#representable + 1] = item else framework[#framework + 1] = {item = item, gaps = gaps} end
      for _, g in ipairs(gaps) do gap_count[g] = (gap_count[g] or 0) + 1 end
    end
  end
  lines[#lines + 1] = ("## 來源商店道具：目前引擎可完整呈現（%d）"):format(#representable)
  lines[#lines + 1] = ""
  lines[#lines + 1] = "| 來源 key | 名稱 | 總價 | 分類 | 屬性 |"
  lines[#lines + 1] = "|---|---|---|---|---|"
  for _, item in ipairs(representable) do
    local props = {}
    if M.active(item) then props[#props + 1] = "主動 " .. M.active(item).active.kind end
    for _, p in ipairs(M.passive_properties(item)) do
      if p.value ~= 0 and p.value ~= "0" then
        local field = M.SUPPORTED[p.property] or (M.MODIFIERS[p.property] and M.MODIFIERS[p.property][1]) or short(p.property)
        props[#props + 1] = field .. " " .. tostring(p.value)
      end
    end
    lines[#lines + 1] = ("| `item_%s` | %s | %s | %s | %s |"):format(item.key, item.name or "?", tostring(M.total_cost(item, by_key)),
      table.concat(item.shops, "、"), table.concat(props, "、"))
  end
  lines[#lines + 1] = ""
  lines[#lines + 1] = "## 能力缺口（needs_framework）"
  lines[#lines + 1] = ""
  lines[#lines + 1] = "缺口：需要的能力，以及需要它的商店道具數。一個道具可能同時缺好幾項。"
  lines[#lines + 1] = ""
  lines[#lines + 1] = "| 能力 | 道具數 |"
  lines[#lines + 1] = "|---|---|"
  local gaps = {}
  for g, n in pairs(gap_count) do gaps[#gaps + 1] = {g, n} end
  table.sort(gaps, function(x, y) return x[2] > y[2] or (x[2] == y[2] and x[1] < y[1]) end)
  for _, g in ipairs(gaps) do lines[#lines + 1] = ("| %s | %d |"):format(g[1], g[2]) end
  lines[#lines + 1] = ""
  lines[#lines + 1] = ("## 來源商店道具：needs_framework（%d）"):format(#framework)
  lines[#lines + 1] = ""
  lines[#lines + 1] = "| 來源 key | 名稱 | 總價 | 缺少 | 說明 |"
  lines[#lines + 1] = "|---|---|---|---|---|"
  for _, row in ipairs(framework) do
    local item = row.item
    lines[#lines + 1] = ("| `item_%s` | %s | %s | %s | %s |"):format(item.key, item.name or "?", tostring(M.total_cost(item, by_key)),
      table.concat(row.gaps, "、"), ((item.description or ""):gsub("\n", " "):gsub("|", "／")))
  end
  lines[#lines + 1] = ""
  -- Actives (E160): what translated, and why the rest wait.
  local translated, deferred = {}, {}
  for _, item in ipairs(catalog.items) do
    if item.listed and item.has_active then
      local result, why = M.active(item)
      if result then translated[#translated + 1] = item else deferred[#deferred + 1] = { item = item, why = why } end
    end
  end
  lines[#lines + 1] = ("## 主動效果（E160）：可轉換 %d、deferred %d"):format(#translated, #deferred)
  lines[#lines + 1] = ""
  lines[#lines + 1] = "由 `scripts/item_source/active.lua` 從來源 datadriven KV 轉換；來源沒有任何道具 vscript（items/Addon_Items/*.lua 不在 repo），只靠腳本的效果無法照來源實作，標 deferred。"
  lines[#lines + 1] = "可轉換的道具若被動屬性或配方另有缺口，仍列在 needs_framework。"
  lines[#lines + 1] = ""
  lines[#lines + 1] = "| 來源 key | 名稱 | 判定 |"
  lines[#lines + 1] = "|---|---|---|"
  for _, item in ipairs(translated) do
    local a = M.active(item).active
    local parts = {}
    for _, m in ipairs(a.modifiers or {}) do
      local stats = {}
      for k, v in pairs(m.stats) do stats[#stats + 1] = k .. " " .. tostring(v) end
      table.sort(stats)
      parts[#parts + 1] = ("%s%s %s 秒：%s"):format(m.name, m.area and ("（範圍 " .. tostring(a.radius) .. (a.include_basic and "，含小兵" or "") .. "）") or "（自身）",
        tostring(m.duration), table.concat(stats, "、"))
    end
    if a.heal then parts[#parts + 1] = "回復生命 " .. tostring(a.heal) end
    if a.mana then parts[#parts + 1] = "回復法力 " .. tostring(a.mana) end
    lines[#lines + 1] = ("| `item_%s` | %s | 可轉換：%s |"):format(item.key, item.name or "?", table.concat(parts, "；"))
  end
  for _, row in ipairs(deferred) do
    lines[#lines + 1] = ("| `item_%s` | %s | deferred：%s |"):format(row.item.key, row.item.name or "?", (row.why:gsub("|", "／")))
  end
  lines[#lines + 1] = ""
  -- Price inconsistencies inside the source.
  lines[#lines + 1] = "## 來源內部價錢不一致（合成結果 ItemCost ≠ 零件＋合成費）"
  lines[#lines + 1] = ""
  lines[#lines + 1] = "Dota 實際售價是零件＋合成費，遊戲以此為準；下列只供來源維護者參考。"
  lines[#lines + 1] = ""
  for _, item in ipairs(catalog.items) do
    local total = M.total_cost(item, by_key)
    if item.recipe and total and item.cost and total ~= item.cost then
      lines[#lines + 1] = ("- `item_%s` %s：ItemCost %d，零件＋合成費 %d"):format(item.key, item.name or "?", item.cost, total)
    end
  end
  return table.concat(lines, "\n") .. "\n", representable, framework
end

function M.main(argv)
  local out = M.OUT
  for i = 1, #argv do if argv[i] == "--out" then out = path.absolute(argv[i + 1], b.root) end end
  local catalog = dofile(M.CATALOG)
  local text, representable, framework = M.render(catalog, M.game_items())
  path.write(out, text, true, true)
  print(("representable %d, needs_framework %d -> %s"):format(#representable, #framework, out))
end

if not pcall(debug.getlocal, 4, 1) then M.main(arg) end
return M
