-- Item texts and values from package schema v3 (M1, docs/character_pipeline/schema/
-- v3-content-model-proposal.md §2.4). Build-time only; the game never runs Lua.
--
--   tools\lua\lua.exe scripts\gen_item_v3.lua [--check] [--content-root DIR] [--items-dir DIR] [--out FILE]
--
-- Reads every v3 document in docs/character_pipeline/items/*.lua (kind = "items"; every
-- history_nobu shop item, the website's source catalog reads the same files), validates and
-- compiles it, and checks it against the shelf (templates/moba_items.lua):
--   * every shelf item has exactly one v3 entry (matched by source_id = the source key);
--     items not on the shelf are catalog-only and skipped here;
--   * every number of the shelf row (cost, stats, modifiers, active, target, cooldown, ...)
--     is mapped by a v3 value's `v2_key` (its dotted path in the row) with the same number,
--     so the shelf numbers and the texts cannot drift apart.
-- Writes templates/item_texts_v3.lua: { [shelf id] = v3 block }. moba_items.lua attaches the
-- block as `v3` and replaces the description with the rendered zh_TW template.
-- Writes templates/item_shop_v3.lua: { [shelf id] = { numbers = { [path] = n }, recipe = { id, ... } } },
-- every number of the shelf row and the recipe resolved from the v3 values and `shop.recipe`.
-- moba_items.lua takes the row's numbers and recipe from it, so the v3 documents are the single
-- source of prices, recipes and stats (value-single-source §3.3); the source catalog export only
-- shapes the row, and this script compares the v3 numbers with it (a difference needs
-- `source_differs` on the value).
-- An item whose effects run in items.dll (the third field of SOURCE_ITEMS) and whose source row has
-- no translated active takes its active from the v3 document: `scripted` in item_shop_v3.lua holds
-- the catalog fields the host checks (M.scripted_active).
-- --check verifies both files are current instead of writing them.
local source = debug.getinfo(1, "S").source:sub(2)
local scripts_dir = assert(source:match("^(.*)[/\\]"))
package.path = scripts_dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local lfs = require("lfs")
local serialize = require("hero_source.serialize")

local M = {}
M.OUT = "templates/item_texts_v3.lua"
M.SHOP_OUT = "templates/item_shop_v3.lua"
-- Row fields that are identity, wiring or presentation, not gameplay numbers.
M.SKIP = { catalog_id = true, recipe = true, shares_cooldown_with = true, id = true, name = true,
  description = true, source_ref = true, v3 = true, test_only = true, purchase_group = true }

local function tools()
  PACKAGE_V3_AS_MODULE = true
  local v = dofile(b.root .. "/docs/character_pipeline/tools/package_v3.lua")
  PACKAGE_V3_AS_MODULE = nil
  return v, dofile(b.root .. "/docs/character_pipeline/tools/v3_compile.lua")
end

-- The shelf numbers a v3 passive stat is (the host applies an item's passive stats from its shelf
-- row, not from items.dll: v3_rust.lua). A value without `v2_key` maps through these.
M.STAT_PATHS = {
  attack_damage = { "atk" }, armor = { "armor" }, max_hp = { "hp" },
  move_speed_pct = { "modifiers.movespeed_bonus_percentage" },
  attack_speed = { "modifiers.attackspeed_bonus_constant" }, hp_regen = { "modifiers.health_regen_constant" },
  mana_regen = { "modifiers.mana_regen_constant" }, max_mana = { "modifiers.mana_bonus" },
  spell_amp_pct = { "modifiers.spell_amplify_percentage" },
  str = { "modifiers.stats_strength_bonus" }, agi = { "modifiers.stats_agility_bonus" }, int = { "modifiers.stats_intellect_bonus" },
  all_stats = { "modifiers.stats_strength_bonus", "modifiers.stats_agility_bonus", "modifiers.stats_intellect_bonus" },
}

-- number leaves of a row: { ["active.damage"] = 300, cost = 150, ... }
function M.numbers(row)
  local out = {}
  local function walk(t, prefix)
    for k, v in pairs(t) do
      if not (prefix == "" and M.SKIP[k]) then
        local path = prefix == "" and tostring(k) or (prefix .. "." .. tostring(k))
        if type(v) == "number" then out[path] = v
        elseif type(v) == "table" then walk(v, path) end
      end
    end
  end
  walk(row, "")
  return out
end

local function same(a, b) return math.abs(a - b) <= 1e-6 * math.max(1, math.abs(a), math.abs(b)) end

-- The catalog fields of a scripted active (the host admits the use, items.dll runs the effect):
-- cooldown, mana, target and the shared cooldown, from the v3 active. `number` resolves "$name".
-- Returns the fields, or nil and why the catalog cannot state this active.
function M.scripted_active(item, number, cooldown_groups)
  local active, cost = item.active, item.active.cost or {}
  local target = active.target or { type = "none" }
  local out = { active = { kind = "scripted" }, cooldown = number(cost.cooldown), mana_cost = number(cost.mana) }
  local include_magic_immune = false
  for _, flag in ipairs(target.flags or {}) do
    if flag == "include_magic_immune" then include_magic_immune = true end
  end
  if include_magic_immune and target.type ~= "unit" then return nil, "only a unit target can admit magic immune units" end
  if target.type == "point" then
    -- no range: anywhere on the map
    out.target = { shape = "point", range = number(target.range) or 0, team = "any", requires_visible = false, out_of_range = "reject" }
  elseif target.type == "unit" then
    local range = number(target.range)
    if not range then return nil, "a unit target needs a range" end
    out.target = { shape = "unit", range = range, team = target.team == "both" and "any" or target.team,
      requires_visible = true, out_of_range = "reject", include_magic_immune = include_magic_immune or nil }
  elseif target.type ~= "none" then
    return nil, "target type " .. tostring(target.type) .. " has no catalog target"
  end
  local consumable = item.shop and item.shop.consumable
  if consumable then
    if number(consumable.charges) ~= 1 then return nil, "the catalog consumes one use, this item has charges" end
    out.consumable = true
  end
  local shared, seen = {}, { [item.id] = true }
  for _, group in ipairs(cost.cooldown_groups or {}) do
    for _, other in ipairs(cooldown_groups[group.group]) do
      if not seen[other] then seen[other] = true; shared[#shared + 1] = (other:gsub("^item_", "")) end
    end
  end
  table.sort(shared)
  if #shared > 0 then out.shares_cooldown_with = shared end
  return out
end

-- shelf rows by source key (item_xxx); docs: list of v3 documents
function M.build(shelf, docs, partial, scripted)
  local validator, compiler = tools()
  local script_stats = dofile(b.root .. "/docs/character_pipeline/tools/v3_vocab.lua").ITEM_SCRIPT_STATS
  local problems, blocks, seen, shops = {}, {}, {}, {}
  local cooldown_groups = {}
  for _, d in ipairs(docs) do
    for _, item in ipairs(d.doc.items or {}) do
      local cost = type(item.active) == "table" and item.active.cost
      for _, group in ipairs(type(cost) == "table" and cost.cooldown_groups or {}) do
        cooldown_groups[group.group] = cooldown_groups[group.group] or {}
        table.insert(cooldown_groups[group.group], item.id)
      end
    end
  end
  local by_key = {}
  for _, row in ipairs(shelf) do
    if not row.test_only and row.source_ref then by_key[row.source_ref:match(":(item_[%w_']+)$")] = row end
  end
  for _, d in ipairs(docs) do
    local errs = validator.validate_doc(d.doc)
    for _, e in ipairs(errs) do problems[#problems + 1] = d.path .. ": " .. e end
    if #errs == 0 then
      assert(d.doc.kind == "items", d.path .. ": item documents must be kind = \"items\"")
      local names = {}
      for _, other in ipairs(docs) do
        for _, it in ipairs(other.doc.items or {}) do if it.id and it.text then names[it.id] = it.text end end
      end
      local compiled = compiler.compile(d.doc, { names = names })
      for _, item in ipairs(d.doc.items or {}) do
        local row = by_key[item.source_id]
        if not row then
          -- catalog-only (not on the shelf yet)
        elseif seen[row.id] then
          problems[#problems + 1] = d.path .. ": " .. item.id .. ": shelf item " .. row.id .. " already has a v3 entry in " .. seen[row.id]
        else
          seen[row.id] = d.path
          local block = compiled.items[item.id]
          -- A text can quote a value of a unit template (the totem's aura radius). The game's catalog has
          -- no template owners, so the item carries the values it quotes.
          local quoted = {}
          for _, text in pairs(block.text or {}) do
            for _, piece in ipairs(text.pieces or {}) do
              local owner = piece.owner and compiled.owner_values[piece.owner]
              if owner and owner.kind == "template" then
                quoted[piece.owner] = quoted[piece.owner] or {}
                quoted[piece.owner][piece.value] = true
              end
            end
          end
          for owner, names in pairs(quoted) do
            block.owner_values = block.owner_values or {}
            block.owner_values[owner] = {}
            for _, value in ipairs(compiled.owner_values[owner].values) do
              if names[value.name] then table.insert(block.owner_values[owner], value) end
            end
          end
          local mapped = {}
          for _, value in ipairs(block.values or {}) do
            if value.v2_key then
              mapped[value.v2_key] = value
            end
          end
          local nums = M.numbers(row)
          -- no v2_key: `shop.cost` and `passive.stats` name the value (the numbers are still compared)
          local by_name = {}
          for _, value in ipairs(block.values or {}) do by_name[value.name] = value end
          local function infer(path, expr)
            local name = type(expr) == "string" and expr:match("^%$([%w_]+)$")
            if name and by_name[name] and not mapped[path] then mapped[path] = by_name[name] end
          end
          infer("cost", item.shop and item.shop.cost)
          for stat, expr in pairs(item.passive and item.passive.stats or {}) do
            for _, path in ipairs(M.STAT_PATHS[stat] or {}) do if nums[path] ~= nil then infer(path, expr) end end
          end
          local shop = { numbers = {} }
          local is_scripted = scripted and scripted[row.id]
          -- The source export sums every Modifier block of an item into its row, also what only a status
          -- gives (the mountain scroll's +50 armor while active). A scripted item's status runs in
          -- items.dll, so such a number leaves the row; it must still be a number of the document, or
          -- two of them (the scroll's chapter: 50 + 6).
          local function only_in_a_status(n)
            local numbers = {}
            for _, value in ipairs(block.values or {}) do
              local v = value.ranks and compiler.value_of(value.ranks[1])
              if v then
                numbers[#numbers + 1] = v
                if value.unit == "pct" then numbers[#numbers + 1] = v / 100 end
              end
            end
            for i, a in ipairs(numbers) do
              if same(a, n) then return true end
              for j = i + 1, #numbers do if same(a + numbers[j], n) then return true end end
            end
            return false
          end
          -- a number the document gives through a status, not its passive stats, is the status's
          local in_passive, in_status = {}, {}
          local function names(stats, into)
            for _, expr in pairs(stats or {}) do
              local name = type(expr) == "string" and expr:match("^%$([%w_]+)$")
              if name then into[name] = true end
            end
          end
          names(item.passive and item.passive.stats, in_passive)
          for _, s in ipairs(item.statuses or {}) do names(s.stats, in_status) end
          for path, n in pairs(nums) do
            local value = mapped[path]
            local passive_number = path:match("^modifiers%.") or not path:find(".", 1, true)
            local active_field = path:match("^active%.") or path:match("^target%.") or path == "cooldown" or path == "mana_cost"
            if is_scripted and item.active and active_field then
              -- replaced by the document's active (M.scripted_active)
            elseif is_scripted and path:match("^aura%.") then
              -- the aura runs in items.dll: checked below
            elseif not value and is_scripted and path == "unique_passive.priority" then
              shop.numbers[path] = n -- the row's ranking of its own number, no number of the document
            elseif not value and is_scripted and path == "unique_passive.ms" and only_in_a_status(n) then
              shop.numbers[path] = n
            elseif value and is_scripted and path:match("^modifiers%.") and in_status[value.name] and not in_passive[value.name] then
              value = nil
              shop.not_passive = shop.not_passive or {}
              table.insert(shop.not_passive, path)
              table.sort(shop.not_passive)
            elseif not value and is_scripted and passive_number and only_in_a_status(n) then
              shop.not_passive = shop.not_passive or {}
              table.insert(shop.not_passive, path)
              table.sort(shop.not_passive)
            elseif not value then
              problems[#problems + 1] = d.path .. ": " .. item.id .. ": shelf number " .. path .. " = " .. n .. " has no v3 value (v2_key = \"" .. path .. "\")"
            elseif not value.ranks then
              problems[#problems + 1] = d.path .. ": " .. item.id .. ": value " .. value.name .. " maps " .. path .. " but has no static ranks"
            else
              local v = compiler.value_of(value.ranks[1])
              -- the shelf stores some percents as fractions (0.075); a pct value is the percent
              local fraction = value.unit == "pct" and (same(v / 100, n) or (not same(v, n) and math.abs(n) < 1))
              local out = fraction and v / 100 or v
              if math.type(n) == "integer" and out == math.floor(out) then out = math.tointeger(out) end
              if not same(out, n) and not item.values[value.name].source_differs then
                problems[#problems + 1] = d.path .. ": " .. item.id .. ": v3 " .. value.name .. " = " .. value.ranks[1] ..
                  " differs from the source catalog " .. path .. " = " .. n .. " (write source_differs when it is meant)"
              end
              shop.numbers[path] = out
            end
          end
          -- the recipe: v3 shop.recipe.components (item ids) against the source row (game ids)
          local recipe = item.shop and item.shop.recipe and item.shop.recipe.components
          if recipe then
            shop.recipe = {}
            for i, c in ipairs(recipe) do shop.recipe[i] = (tostring(c):gsub("^item_", "")) end
          end
          if table.concat(shop.recipe or {}, ",") ~= table.concat(row.recipe or {}, ",") then
            problems[#problems + 1] = d.path .. ": " .. item.id .. ": v3 recipe {" .. table.concat(shop.recipe or {}, ", ") ..
              "} differs from the source catalog {" .. table.concat(row.recipe or {}, ", ") .. "}"
          end
          -- the host applies passive stats from the shelf row: a stat without a shelf number would be lost
          -- The source row may lack one (its export reads other modifier blocks): the document's goes on it.
          for stat, expr in pairs(is_scripted and item.passive and item.passive.stats or {}) do
            local found = false
            local paths = M.STAT_PATHS[stat] or {}
            for _, path in ipairs(paths) do found = found or nums[path] ~= nil end
            local name = type(expr) == "string" and expr:match("^%$([%w_]+)$")
            local value = name and by_name[name]
            if not found and #paths > 0 and value and value.ranks then
              shop.added = shop.added or {}
              for _, path in ipairs(paths) do
                local number = compiler.value_of(value.ranks[1])
                if value.unit == "pct" then number = number / 100 end
                if math.type(number) == "float" and number == math.floor(number) then number = math.tointeger(number) end
                shop.added[path] = number
                shop.numbers[path] = number
              end
            elseif not found and not script_stats[stat] then
              problems[#problems + 1] = d.path .. ": " .. item.id .. ": passive stat " .. stat .. " has no number on the shelf row"
            end
          end
          -- movement speed of group move_speed_unique is the row's unique passive (v3_rust drops the
          -- status's); the row and the status never hold the same number twice
          local unique_status = false
          for _, s in ipairs(item.statuses or {}) do unique_status = unique_status or s.unique_group == "move_speed_unique" end
          if is_scripted and unique_status ~= (row.unique_passive ~= nil and row.unique_passive.group == "movement") then
            problems[#problems + 1] = d.path .. ": " .. item.id .. ": a move_speed_unique status and the row's movement unique passive must come together"
          end
          if is_scripted and row.aura then
            if item.passive and item.passive.aura then shop.no_native_aura = true
            else problems[#problems + 1] = d.path .. ": " .. item.id .. ": the shelf row has an aura, the document's passive has none" end
          end
          if is_scripted and item.active then
            local function number(expr)
              if type(expr) == "number" then return expr end
              local name = type(expr) == "string" and expr:match("^%$([%w_]+)$")
              local value = name and by_name[name]
              if value and value.ranks then return compiler.value_of(value.ranks[1]) end
            end
            local fields, why = M.scripted_active(item, number, cooldown_groups)
            if fields then shop.scripted = fields
            else problems[#problems + 1] = d.path .. ": " .. item.id .. ": " .. why end
          end
          shops[row.id] = shop
          for path, value in pairs(mapped) do
            if nums[path] == nil then
              problems[#problems + 1] = d.path .. ": " .. item.id .. ": value " .. value.name .. " v2_key " .. path .. " is not a number of the shelf row"
            end
          end
          blocks[row.id] = block
        end
      end
    end
  end
  for key, row in pairs(by_key) do
    if not partial and not seen[row.id] then problems[#problems + 1] = "shelf item " .. row.id .. " (" .. key .. ") has no v3 entry" end
  end
  table.sort(problems)
  return blocks, problems, shops
end

function M.render(blocks, header)
  header = header or ("-- GENERATED by scripts/gen_item_v3.lua from docs/character_pipeline/items/*.lua; do not edit.\n" ..
    "-- Shelf item id -> package schema v3 block (values, info line, text); see templates/moba_items.lua.\n")
  local body = serialize.serialize(blocks, header)
  return (body:gsub("^(.-)return ", "%1local data = ", 1)) .. "return function(ctx) return data end\n"
end

local function load_doc(path)
  local env = { pairs = pairs, ipairs = ipairs, string = string, table = table, math = math, utf8 = utf8 }
  local chunk = assert(loadfile(path, "t", env))
  local v = chunk()
  if type(v) == "function" then v = v({}) end
  return v
end

function M.main(argv)
  local opts = {}
  local i = 1
  while i <= #argv do
    local a = argv[i]
    if a == "--check" then opts.check = true
    elseif a == "--content-root" then opts.content_root = argv[i + 1]; i = i + 1
    elseif a == "--items-dir" then opts.items_dir = argv[i + 1]; i = i + 1
    elseif a == "--out" then opts.out = argv[i + 1]; i = i + 1
    elseif a == "--partial" then opts.partial = true
    else error("unknown argument " .. a) end
    i = i + 1
  end
  local content_root = opts.content_root or (b.root .. "/scripts/lua_data")
  local items_dir = opts.items_dir or (b.root .. "/docs/character_pipeline/items")
  -- the shelf as the source catalog shapes it: the comparison runs before the v3 numbers replace its own
  local ctx = require("content_builder").new(content_root)
  ctx.source_shelf = true
  local shelf = ctx.include("templates/moba_items.lua")
  local docs = {}
  if lfs.attributes(items_dir, "mode") == "directory" then
    local names = {}
    for name in lfs.dir(items_dir) do if name:match("%.lua$") then names[#names + 1] = name end end
    table.sort(names)
    for _, name in ipairs(names) do docs[#docs + 1] = { path = name, doc = load_doc(items_dir .. "/" .. name) } end
  end
  local blocks, problems, shops = M.build(shelf, docs, opts.partial, ctx.scripted_items)
  if #problems > 0 then
    for _, p in ipairs(problems) do io.stderr:write(p, "\n") end
    io.stderr:write(#problems .. " problem(s)\n")
    return 1
  end
  local out = opts.out or (content_root .. "/" .. M.OUT)
  local shop_out = opts.out and (opts.out:gsub("[^/\\]+$", "item_shop_v3.lua")) or (content_root .. "/" .. M.SHOP_OUT)
  local files = {
    { out, M.render(blocks) },
    { shop_out, M.render(shops, "-- GENERATED by scripts/gen_item_v3.lua from docs/character_pipeline/items/*.lua; do not edit.\n" ..
      "-- Shelf item id -> the row's numbers (path = value) and recipe from the v3 values; see templates/moba_items.lua.\n") },
  }
  if opts.check then
    for _, file in ipairs(files) do
      local f = io.open(file[1], "rb")
      local current = f and f:read("a"); if f then f:close() end
      if not current or current:gsub("\r\n", "\n") ~= file[2] then io.stderr:write(file[1] .. " is stale; run scripts/gen_item_v3.lua\n"); return 1 end
      print("OK: " .. file[1] .. " is current")
    end
    return 0
  end
  for _, file in ipairs(files) do
    local f = assert(io.open(file[1], "wb")); f:write(file[2]); f:close()
  end
  local n = 0 for _ in pairs(blocks) do n = n + 1 end
  print("wrote " .. out .. " and " .. shop_out .. " (" .. n .. " items)")
  return 0
end

if rawget(_G, "GEN_ITEM_V3_AS_MODULE") then return M end
os.exit(M.main(arg))
