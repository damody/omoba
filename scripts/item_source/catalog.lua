-- Item provenance catalog (E155) from a read-only checkout of damody/history_nobu, the
-- same pinned source as the heroes (source-index, E169). Files are only read as text; no
-- script of the source (vscripts) is executed. Tooltips are UTF-16LE (hero_source.text);
-- descriptions expand `%special%` placeholders from the item's AbilitySpecial values.
--
-- One row per purchasable item `item_<name>` of scripts/npc/npc_items_custom_load.txt (recipe
-- scrolls `item_recipe_*` are folded into their result as `recipe`). Every row and tooltip
-- carries path, line, byte span and the SHA-256 of its original text, so `verify` can
-- re-read the source and prove the catalog still matches it.
local kv = require("hero_source.kv")
local sha256 = require("hero_source.sha256")
local M = {}

M.SCHEMA = "omoba.item_source_catalog/1"
M.ADDON = "game/dota_addons/nobu"
M.ITEM_FILE = M.ADDON .. "/scripts/npc/npc_items_custom_load.txt"
M.SHOP_FILE = M.ADDON .. "/scripts/shops/nobu_shops.txt"
M.TOOLTIP_FILE = M.ADDON .. "/resource/addon_tchinese.txt"
M.VSCRIPTS_DIR = M.ADDON .. "/scripts/vscripts"
local text_mod = require("hero_source.text")
local script = require("item_source.script")

local function read_file(file)
  return text_mod.read(file)
end

local function number(value)
  local n = tonumber(value)
  if n and math.type(n) == "float" and n == math.floor(n) then n = math.tointeger(n) end
  return n
end

-- Every modifier of the item. Some source items repeat the top-level "Modifiers" block (E183:
-- e.g. the boots keep their stats in a second block, which their descriptions and the game
-- show); all blocks are read, in order, as one list.
local function modifier_blocks(block)
  local out = {}
  for _, n in ipairs(block.children or {}) do
    if n.key == "Modifiers" and n.children then
      for _, modifier in ipairs(n.children) do out[#out + 1] = modifier end
    end
  end
  return out
end

-- "MODIFIER_PROPERTY_X" -> value, resolving "%name" through the item's AbilitySpecial.
local function properties_of(block, specials)
  local out = {}
  for _, modifier in ipairs(modifier_blocks(block)) do
    local props = modifier.children and kv.find(modifier.children, "Properties")
    local passive = modifier.children and kv.value(modifier.children, "Passive") == "1"
    for _, n in ipairs(props and props.children or {}) do
      if n.key and n.value ~= nil then
        local raw = n.value
        local ref = raw:match("^%%([%w_]+)$")
        local value = ref and specials[ref] and specials[ref].value or raw
        -- An empty property value ("") reads as 0 in the engine; keep raw as written.
        if value == "" then value = "0" end
        out[#out + 1] = { modifier = modifier.key, property = n.key, raw = raw, special = ref,
          value = number(value) or value, passive = passive or nil, line = n.line }
      end
    end
  end
  return out
end

local function specials_of(block)
  local list, by_name = {}, {}
  local special = kv.find(block.children, "AbilitySpecial")
  for _, entry in ipairs(special and special.children or {}) do
    for _, n in ipairs(entry.children or {}) do
      if n.key and n.key:lower() ~= "var_type" and not n.children then
        local row = { name = n.key, value = n.value, number = number(n.value), line = n.line }
        list[#list + 1] = row
        by_name[n.key] = row
      end
    end
  end
  return list, by_name
end

local function script_refs(block)
  local refs, seen = {}, {}
  kv.walk(block.children, function(n, _, parent)
    if n.key == "ScriptFile" and n.value then
      local fn = parent and kv.value(parent.children, "Function")
      local ref = n.value .. "#" .. tostring(fn)
      if not seen[ref] then seen[ref] = true; refs[#refs + 1] = { file = n.value, ["function"] = fn } end
    end
  end)
  return refs
end

-- Plain, order-preserving copy of a KV subtree: { {key=, value=} | {key=, children={...}} }.
-- Duplicate keys and unkeyed blocks are kept as written (datadriven actions repeat keys).
local function tree(nodes)
  local out = {}
  for _, n in ipairs(nodes or {}) do
    local row = { key = n.key }
    if n.children then row.children = tree(n.children) else row.value = n.value end
    out[#out + 1] = row
  end
  return out
end

-- The item's datadriven active: OnSpellStart actions and its Modifiers blocks, as written.
local function active_kv(item_node)
  local spell = kv.find(item_node.children, "OnSpellStart")
  local blocks = modifier_blocks(item_node)
  local mods = #blocks > 0 and { children = blocks } or nil
  local hit = kv.find(item_node.children, "OnProjectileHitUnit")
  local toggle = kv.find(item_node.children, "OnToggleOn")
  if not spell and not mods and not toggle then return nil end
  return { on_spell_start = spell and tree(spell.children) or nil, modifiers = mods and tree(mods.children) or nil,
    on_projectile_hit = hit and tree(hit.children) or nil, on_toggle_on = toggle and tree(toggle.children) or nil }
end

local function span(text, file, n)
  return { path = file, line = n.line, start = n.start, stop = n.stop, raw_sha256 = sha256(text:sub(n.start, n.stop)) }
end

local function tooltips(text)
  local doc = kv.parse(text)
  local lang = doc.nodes[1]
  local tokens_node = lang and kv.find(lang.children, "Tokens")
  local tokens = {}
  for _, n in ipairs(tokens_node and tokens_node.children or {}) do
    if n.key and n.value ~= nil then
      local lower = n.key:lower()
      if not tokens[lower] then tokens[lower] = n end
    end
  end
  return tokens
end

local function tooltip(tokens, text, key)
  local n = tokens[key:lower()]
  if not n then return nil end
  return { key = n.key, value = n.value, path = M.TOOLTIP_FILE, line = n.line, start = n.start, stop = n.stop,
    raw_sha256 = sha256(text:sub(n.start, n.stop)) }
end

-- Plain text of a tooltip value: <br /> as line breaks, other markup removed.
function M.plain(value)
  if not value then return nil end
  local s = value:gsub("<[Bb][Rr]%s*/?>", "\n"):gsub("<[^>]+>", ""):gsub("\\n", "\n")
  s = s:gsub("^%s+", ""):gsub("%s+$", "")
  return s
end

-- shops.txt: category -> items (in order), item -> categories.
local function shops(text)
  local by_item, categories, order = {}, {}, {}
  local current
  for line in (text .. "\n"):gmatch("([^\n]*)\n") do
    local category = line:match('^%s*"([%w_]+)"%s*$')
    if category and category ~= "dota_shops" then current = category; categories[current] = {}; order[#order + 1] = current end
    local item = line:match('"item"%s+"item_([%w_]+)"')
    if item and current and not line:match("^%s*//") then
      local list = categories[current]
      list[#list + 1] = item
      by_item[item] = by_item[item] or {}
      by_item[item][#by_item[item] + 1] = current
    end
  end
  return by_item, categories, order
end

function M.build(source)
  local item_text = read_file(source .. "/" .. M.ITEM_FILE)
  local tip_text = read_file(source .. "/" .. M.TOOLTIP_FILE)
  local shop_text = read_file(source .. "/" .. M.SHOP_FILE)
  local doc = kv.parse(item_text)
  local root = doc.nodes[1]
  assert(root and root.key == "DOTAAbilities", "unexpected item file root")
  local tokens = tooltips(tip_text)
  local shop_of, shop_categories, shop_order = shops(shop_text)

  local recipes = {}
  for _, n in ipairs(root.children) do
    local name = n.key and n.key:match("^item_recipe_(.+)$")
    if name and n.children then
      local result = kv.value(n.children, "ItemResult")
      local requirements = {}
      local req = kv.find(n.children, "ItemRequirements")
      for _, alt in ipairs(req and req.children or {}) do
        if alt.value then
          local parts = {}
          for part in alt.value:gmatch("[^;]+") do parts[#parts + 1] = part:gsub("^item_", "") end
          requirements[#requirements + 1] = parts
        end
      end
      if result then
        local key = result:gsub("^item_", "")
        recipes[key] = recipes[key] or {}
        recipes[key][#recipes[key] + 1] = { key = n.key, cost = number(kv.value(n.children, "ItemCost")) or 0,
          requirements = requirements, block = span(item_text, M.ITEM_FILE, n) }
      end
    end
  end

  local items, seen = {}, {}
  for _, n in ipairs(root.children) do
    local name = n.key and n.key:match("^item_(.+)$")
    if name and not name:match("^recipe_") and n.children then
      if seen[name] then
        seen[name].duplicates = (seen[name].duplicates or 0) + 1
      else
        local specials, special_by_name = specials_of(n)
        local c = n.children
        local row = {
          key = name,
          source_id = number(kv.value(c, "ID")),
          cost = number(kv.value(c, "ItemCost")),
          quality = kv.value(c, "ItemQuality"),
          behavior = kv.value(c, "AbilityBehavior"),
          target_team = kv.value(c, "AbilityUnitTargetTeam"),
          target_types = kv.value(c, "AbilityUnitTargetType"),
          damage_type = kv.value(c, "AbilityUnitDamageType"),
          cooldown = kv.value(c, "AbilityCooldown"),
          mana_cost = kv.value(c, "AbilityManaCost"),
          cast_range = kv.value(c, "AbilityCastRange"),
          shared_cooldown = kv.value(c, "AbilitySharedCooldown"),
          stackable = kv.value(c, "ItemStackable"),
          initial_charges = kv.value(c, "ItemInitialCharges"),
          side_shop = kv.value(c, "SideShop"),
          specials = specials,
          properties = properties_of(n, special_by_name),
          scripts = script_refs(n),
          active_kv = active_kv(n),
          shops = shop_of[name] or {},
          recipe = recipes[name],
          block = span(item_text, M.ITEM_FILE, n),
          tooltip = {
            name = tooltip(tokens, tip_text, "DOTA_Tooltip_Ability_item_" .. name),
            description = tooltip(tokens, tip_text, "DOTA_Tooltip_Ability_item_" .. name .. "_Description"),
            lore = tooltip(tokens, tip_text, "DOTA_Tooltip_Ability_item_" .. name .. "_Lore"),
            note = tooltip(tokens, tip_text, "DOTA_Tooltip_Ability_item_" .. name .. "_Note0"),
          },
        }
        row.name = row.tooltip.name and row.tooltip.name.value or nil
        if row.tooltip.description then
          local values = {}
          for _, s in ipairs(specials) do values[s.name] = s.value end
          local expanded, unknown = text_mod.expand(row.tooltip.description.value, values)
          row.description = M.plain(expanded)
          if #unknown > 0 then row.description_unknown_placeholders = unknown end
        end
        -- Item scripts present in the source (history_nobu ships items/Addon_Items/*.lua).
        for _, ref in ipairs(row.scripts) do
          local f = io.open(source .. "/" .. M.VSCRIPTS_DIR .. "/" .. ref.file, "rb")
          ref.present_in_source = f ~= nil
          if f then
            -- What the referenced function does, read as text (item_source/script.lua).
            local script_text = f:read("a")
            f:close()
            if ref["function"] then
              local analysis = script.analyze(script_text, ref["function"])
              analysis.reviewed_presentation = script.REVIEWED_PRESENTATION[ref.file .. "#" .. ref["function"]]
              local reviewed = script.REVIEWED_EFFECTS[ref.file .. "#" .. ref["function"]]
              local body = reviewed and script.function_body(script_text, ref["function"])
              local matches, missing = true, nil
              if reviewed then matches, missing = script.review_matches(reviewed, body or "") end
              -- A review whose fragments are gone from the source no longer describes it (E188).
              analysis.reviewed_effect = matches and reviewed or nil
              analysis.reviewed_mismatch = (not matches) and missing or nil
              ref.analysis = analysis
            end
          end
        end
        row.has_active = (row.behavior or ""):find("PASSIVE", 1, true) == nil and row.behavior ~= nil
        row.listed = #row.shops > 0
        seen[name] = row
        items[#items + 1] = row
      end
    end
  end
  table.sort(items, function(a, b) return a.key < b.key end)
  local orphan_recipes = {}
  for key in pairs(recipes) do if not seen[key] then orphan_recipes[#orphan_recipes + 1] = key end end
  table.sort(orphan_recipes)
  return {
    schema = M.SCHEMA,
    files = { items = M.ITEM_FILE, tooltips = M.TOOLTIP_FILE, shops = M.SHOP_FILE },
    shop_categories = shop_categories,
    shop_order = shop_order,
    items = items,
    orphan_recipes = orphan_recipes,
    diagnostics = #doc.diagnostics,
  }
end

-- Re-reads every recorded span and compares its SHA-256. Returns a list of mismatches.
function M.verify(catalog, source)
  local texts, problems = {}, {}
  local function text_of(file)
    texts[file] = texts[file] or read_file(source .. "/" .. file)
    return texts[file]
  end
  local function check(entry, label)
    if not entry then return end
    local text = text_of(entry.path)
    if sha256(text:sub(entry.start, entry.stop)) ~= entry.raw_sha256 then problems[#problems + 1] = label end
  end
  for _, item in ipairs(catalog.items) do
    check(item.block, item.key .. " block")
    for kind, entry in pairs(item.tooltip or {}) do check(entry, item.key .. " tooltip " .. kind) end
    for _, recipe in ipairs(item.recipe or {}) do check(recipe.block, item.key .. " recipe " .. recipe.key) end
  end
  return problems
end

return M
