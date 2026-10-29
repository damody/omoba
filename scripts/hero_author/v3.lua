-- Package schema v3 documents in the hero build (M1, docs/character_pipeline/schema/
-- v3-content-model-proposal.md §2.3). A package that carries v3.lua (the hero's single authoring
-- source; values, text and the data behavior) gets, per ability:
--   v3           the compiled block (values with Fixed64 raw, info line, parsed text)
--   description  the zh_TW template rendered with every rank (deprecated field)
--   extras       every static v3 value by its v3 name, plus `v2_key` aliases for the
--                host contract keys (charges.max_key, targeting_shape.radius_key, ...)
--   levels       cooldown / mana / cast point / range / required level from the info line
-- The v2 abilities.lua numbers stay until M5; any that differ from the kit fail the build,
-- and every extras key the package's Rust handlers read must exist.
local M = {}

-- root: the repo root (its docs/character_pipeline/tools holds the v3 validator and compiler).
local function tools_dir(root) return root .. "/docs/character_pipeline/tools" end

local loaded = {}
local function modules(repo_root)
  local root = tools_dir(repo_root)
  if not loaded[root] then
    PACKAGE_V3_AS_MODULE = true
    local v3 = dofile(root .. "/package_v3.lua")
    PACKAGE_V3_AS_MODULE = nil
    loaded[root] = { validator = v3, compiler = dofile(root .. "/v3_compile.lua") }
  end
  return loaded[root]
end

-- Validates and compiles a package's v3.lua table; errors name the first problems.
function M.compile(repo_root, kit)
  local m = modules(repo_root)
  local errors = m.validator.validate_doc(kit)
  assert(#errors == 0, "v3.lua is not a valid v3 document:\n" .. table.concat(errors, "\n", 1, math.min(#errors, 20)))
  assert(kit.kind == "hero", "v3.lua must be a hero document")
  return m.compiler.compile(kit, { names = m.compiler.item_names(repo_root .. "/docs/character_pipeline/items") })
end

-- Host contract keys an entry references (omoba-content-model *_key fields).
function M.host_keys(entry)
  local keys = {}
  local function add(k) if type(k) == "string" then keys[k] = true end end
  add(entry.targeting_reach_key)
  for _, field in ipairs({ "targeting_shape", "channel", "charges", "cast_cooldown_refund", "attack_orb" }) do
    local t = entry[field]
    if type(t) == "table" then
      for k, v in pairs(t) do if type(k) == "string" and k:match("_key$") then add(v) end end
    end
  end
  return keys
end

-- Extras keys omoba-core reads by name (not through a *_key field): kept under their v2
-- name whenever the v2 generator emitted them.
M.ENGINE_KEYS = {
  summon_owner_cap = true,                       -- omoba-core summon_owner_cap.rs EXTRA_KEY
  attack_orb_attack_speed_slow_constant = true,  -- omoba-core attack_orb.rs ATTACK_SPEED_CONSTANT_KEY
}

-- omoba-core AbilityLevelData flattens extras next to these per-rank fields, so an extras key
-- may not reuse them; the kit's cooldown / mana cost already become `levels`.
M.RESERVED = { cooldown = true, mana_cost = true, cast_time = true, range = true,
  cast_interrupt_on_damage = true, required_hero_level = true }

-- The v2 generator stores a few host keys converted; an alias gets the same conversion.
local CONVERT = {
  shape_half_angle = function(v) return math.rad(v) end,
  attack_orb_move_slow = function(v) return v / 100 end,
  attack_orb_attack_slow = function(v) return v / 100 end,
}

local function same(a, b)
  return math.abs(a - b) <= 1e-6 * math.max(1, math.abs(a), math.abs(b))
end

local function list_text(t)
  local out = {}
  for i, v in ipairs(t or {}) do out[i] = tostring(v) end
  return "{" .. table.concat(out, ", ") .. "}"
end

-- entry: the v2 ability_entry result; compiled: M.compile result. Mutates entry.
function M.apply(entry, compiled)
  local id = entry.id
  local block = compiled.abilities[id]
  assert(block, "v3.lua has no ability " .. id)
  assert(block.max_level == entry.max_level, id .. ": kit max_level " .. block.max_level .. " differs from abilities.lua " .. entry.max_level)
  local v2_extras = entry.extras or {}
  local extras = {}
  for name, nums in pairs(compiled.extras[id] or {}) do
    if not M.RESERVED[name] then extras[name] = nums end
  end
  local host = M.host_keys(entry)
  for key in pairs(M.ENGINE_KEYS) do if v2_extras[key] then host[key] = true end end
  for _, row in ipairs(block.values or {}) do
    if row.v2_key then
      assert(row.ranks, id .. ": value " .. row.name .. " has v2_key but no static ranks")
      assert(not host[row.v2_key] or extras[row.v2_key] == nil or row.v2_key == row.name,
        id .. ": v2_key " .. row.v2_key .. " collides with a v3 value name")
      local conv = CONVERT[row.v2_key] or function(v) return v end
      local nums = {}
      for i = 1, entry.max_level do nums[i] = conv(compiled.extras[id][row.name][i] or compiled.extras[id][row.name][1]) end
      local old = v2_extras[row.v2_key]
      if old then
        for i = 1, entry.max_level do
          assert(same(nums[i], old[i]), id .. ": v3 " .. row.name .. " " .. list_text(nums) .. " differs from abilities.lua "
            .. row.v2_key .. " " .. list_text(old) .. " (fix the v2 copy, the kit is the source)")
        end
      end
      -- only host contract keys keep their v2 name; handlers read the v3 name
      if host[row.v2_key] then extras[row.v2_key] = nums end
    end
  end
  -- omoba-core reads the stored-charges rank-up policy as an extras flag (comp/hero.rs KEEP_COUNT_KEY)
  local mapped = {}
  if block.charges_on_rank_up == "keep_restart" then
    local flag = {}
    for i = 1, entry.max_level do flag[i] = 1 end
    local old = v2_extras.custom_charges_rank_up_keep_count
    if old then for i = 1, entry.max_level do assert(same(old[i], 1), id .. ": abilities.lua charges_rank_up_keep_count is not 1") end end
    extras.custom_charges_rank_up_keep_count = flag
    mapped.custom_charges_rank_up_keep_count = true
  end
  -- every number the v2 generator emitted must have a v3 home (no number left behind)
  for _, row in ipairs(block.values or {}) do if row.v2_key then mapped[row.v2_key] = true end end
  local orphans = {}
  for key in pairs(v2_extras) do if not mapped[key] then orphans[#orphans + 1] = key end end
  table.sort(orphans)
  assert(#orphans == 0, id .. ": abilities.lua numbers without a kit value (add v2_key to the value that replaces each): "
    .. table.concat(orphans, ", "))
  for key in pairs(host) do
    assert(extras[key], id .. ": host key " .. key .. " needs a kit value with v2_key = \"" .. key .. "\"")
  end
  for name, nums in pairs(extras) do
    for i = 1, entry.max_level do
      if nums[i] == nil then nums[i] = nums[#nums] end
    end
  end
  entry.extras = extras
  -- levels: the info line is the source; the v2 copy must agree
  local lv = compiled.levels[id] or {}
  for i = 1, entry.max_level do
    local old, new = entry.levels[i], lv[i] or {}
    for _, f in ipairs({ "cooldown", "mana_cost", "cast_time", "range" }) do
      local v = new[f]
      if v == nil then
        assert((old[f] or 0) == 0, id .. ": abilities.lua " .. f .. "[" .. i .. "] = " .. tostring(old[f]) .. " but the kit has no " .. f)
      else
        assert(same(v, old[f] or 0), id .. ": kit " .. f .. "[" .. i .. "] = " .. v .. " differs from abilities.lua " .. tostring(old[f]))
        old[f] = v
      end
    end
    if new.required_hero_level ~= nil then
      assert(old.required_hero_level == nil or old.required_hero_level == new.required_hero_level,
        id .. ": kit required_level[" .. i .. "] differs from abilities.lua")
      old.required_hero_level = new.required_hero_level
    end
  end
  entry.description = compiled.description[id] or entry.description
  entry.display_name = block.text and block.text.zh_TW and block.text.zh_TW.name or entry.display_name
  entry.v3 = block
end

-- Every `extra...("key"` read in the package's Rust files must exist in some ability's extras,
-- and no string literal may still name a replaced v2 key (consts, match arms and payload keys
-- read extras too, so a leftover literal is a read that would fail at run time).
function M.check_rust_keys(rust_texts, entries)
  local have, replaced = {}, {}
  for _, e in ipairs(entries) do
    for k in pairs(e.extras or {}) do have[k] = true end
    for _, row in ipairs(e.v3 and e.v3.values or {}) do
      if row.v2_key and row.v2_key ~= row.name then replaced[row.v2_key] = row.name end
    end
  end
  local missing = {}
  for file, text in pairs(rust_texts) do
    for key in text:gmatch('extra[%w_]*%s*%(%s*[%w_]*%s*,?%s*"([%w_]+)"') do
      if not have[key] then missing[#missing + 1] = file .. ": " .. key end
    end
    for key in text:gmatch('"([%w_]+)"') do
      if replaced[key] and not have[key] then
        missing[#missing + 1] = file .. ": \"" .. key .. "\" is a replaced v2 key (now \"" .. replaced[key] .. "\")"
      end
    end
  end
  local seen, unique = {}, {}
  for _, m in ipairs(missing) do if not seen[m] then seen[m] = true; unique[#unique + 1] = m end end
  missing = unique
  table.sort(missing)
  assert(#missing == 0, "Rust handlers read extras keys the kit does not produce (rename them to the v3 value names):\n"
    .. table.concat(missing, "\n"))
end

return M
