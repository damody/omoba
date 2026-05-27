local package_dir = arg[1]

if not package_dir or package_dir == "" then
  io.stderr:write("SchemaError: usage: lua5.4 validate_package.lua <package_dir>\n")
  os.exit(2)
end

local required_files = {
  "character.lua",
  "prompts.lua",
  "manifest.lua",
  "toolchain.lua",
  "omoba_stub.lua",
}

local errors = {}
local warnings = {}

local function add_error(message)
  table.insert(errors, "SchemaError: " .. message)
end

local function add_warning(message)
  table.insert(warnings, "GameContractWarning: " .. message)
end

local function is_array(t)
  if type(t) ~= "table" then
    return false
  end
  local max = 0
  local count = 0
  for k, _ in pairs(t) do
    if type(k) ~= "number" or k < 1 or k % 1 ~= 0 then
      return false
    end
    if k > max then
      max = k
    end
    count = count + 1
  end
  return max == count
end

local function load_lua_table(file_name)
  local path = package_dir .. "/" .. file_name
  local ok, chunk_or_err = pcall(dofile, path)
  if not ok then
    add_error(file_name .. " failed to load: " .. tostring(chunk_or_err))
    return nil
  end
  if type(chunk_or_err) ~= "function" then
    add_error(file_name .. " must return function(ctx)")
    return nil
  end
  local ok_call, result = pcall(chunk_or_err, {})
  if not ok_call then
    add_error(file_name .. " failed when called: " .. tostring(result))
    return nil
  end
  if type(result) ~= "table" then
    add_error(file_name .. " must return a table")
    return nil
  end
  if result.schema_version ~= 1 then
    add_error(file_name .. " must set schema_version = 1")
  end
  return result
end

local function valid_snake_case(value)
  return type(value) == "string"
    and value:match("^[a-z][a-z0-9_]*$") ~= nil
    and value:match("__") == nil
end

local function validate_character(character)
  if type(character.hero) ~= "table" then
    add_error("character.lua missing hero table")
    return
  end
  local hero = character.hero
  if not valid_snake_case(hero.id) then
    add_error("invalid hero id: " .. tostring(hero.id))
  end

  for _, field in ipairs({
    "display_name",
    "title",
    "gender",
    "role",
    "combat_read",
  }) do
    if type(hero[field]) ~= "string" or hero[field] == "" then
      add_error("character.lua hero." .. field .. " must be a non-empty string")
    end
  end
  if type(hero.personality) ~= "table" then
    add_error("character.lua hero.personality must be a table")
  end
  if type(hero.art_direction) ~= "table" then
    add_error("character.lua hero.art_direction must be a table")
  end
  if type(character.gameplay) ~= "table" then
    add_error("character.lua missing gameplay table")
  end
  if type(character.automation) ~= "table" then
    add_error("character.lua missing automation table")
  end

  if type(character.abilities) ~= "table" or not is_array(character.abilities) then
    add_error("character.lua abilities must be an array table")
    return
  end

  local seen_ids = {}
  local seen_slots = {}
  local allowed_slots = { Q = true, W = true, E = true, R = true }
  for index, ability in ipairs(character.abilities) do
    if type(ability) ~= "table" then
      add_error("ability #" .. index .. " must be a table")
    else
      if not valid_snake_case(ability.id) then
        add_error("invalid ability id: " .. tostring(ability.id))
      elseif seen_ids[ability.id] then
        add_error("duplicate ability id: " .. ability.id)
      else
        seen_ids[ability.id] = true
      end
      if not allowed_slots[ability.slot] then
        add_error("invalid ability slot: " .. tostring(ability.slot))
      elseif seen_slots[ability.slot] then
        add_error("duplicate slot: " .. ability.slot)
      else
        seen_slots[ability.slot] = true
      end
      for _, field in ipairs({ "name", "gameplay_intent", "visual_motif" }) do
        if type(ability[field]) ~= "string" or ability[field] == "" then
          add_error("ability " .. tostring(ability.id) .. " missing " .. field)
        end
      end
    end
  end
end

local function collect_providers(toolchain)
  local providers = {}
  if type(toolchain.providers) ~= "table" then
    add_error("toolchain.lua missing providers table")
    return providers
  end
  for name, _ in pairs(toolchain.providers) do
    providers[name] = true
  end
  return providers
end

local function validate_prompt_providers(node, providers, path)
  if type(node) ~= "table" then
    return
  end
  if type(node.provider) == "string" and not providers[node.provider] then
    add_error(path .. ".provider references unknown provider: " .. node.provider)
  end
  for key, value in pairs(node) do
    if type(value) == "table" then
      validate_prompt_providers(value, providers, path .. "." .. tostring(key))
    end
  end
end

local function unsafe_path(value)
  if type(value) ~= "string" then
    return false
  end
  if value:find("\\", 1, true) then
    return true
  end
  if value == ".." or value:match("^%.%./") or value:match("/%.%./") or value:match("/%.%.$") then
    return true
  end
  return false
end

local function validate_manifest_paths(node, path)
  if type(node) ~= "table" then
    return
  end
  for key, value in pairs(node) do
    local child_path = path .. "." .. tostring(key)
    if key == "path" and unsafe_path(value) then
      add_error("unsafe path at " .. child_path .. ": " .. tostring(value))
    elseif type(value) == "table" then
      validate_manifest_paths(value, child_path)
    end
  end
end

local function validate_omoba_stub(stub)
  if type(stub.hero) ~= "table" then
    add_error("omoba_stub.lua missing hero table")
  end
  if type(stub.abilities) ~= "table" then
    add_error("omoba_stub.lua missing abilities table")
  end
  if type(stub.assets) ~= "table" then
    add_error("omoba_stub.lua missing assets table")
  end
  if type(stub.animations) ~= "table" then
    add_error("omoba_stub.lua missing animations table")
  else
    for _, clip in ipairs({ "idle", "run", "attack", "cast_q", "cast_w", "cast_e", "cast_r", "death" }) do
      if type(stub.animations[clip]) ~= "string" or stub.animations[clip] == "" then
        add_error("omoba_stub.lua missing animation clip: " .. clip)
      end
    end
  end
  if stub.apply_automatically ~= false then
    add_warning("omoba_stub.lua should set apply_automatically = false for version one")
  end
  if type(stub.assets) == "table" and type(stub.assets.model) == "string" then
    add_warning("model path is a future import slot and is not expected to exist yet: " .. stub.assets.model)
  end
end

local loaded = {}
for _, file_name in ipairs(required_files) do
  loaded[file_name] = load_lua_table(file_name)
end

if loaded["character.lua"] then
  validate_character(loaded["character.lua"])
end

local providers = {}
if loaded["toolchain.lua"] then
  providers = collect_providers(loaded["toolchain.lua"])
end
if loaded["prompts.lua"] then
  validate_prompt_providers(loaded["prompts.lua"], providers, "prompts")
end
if loaded["manifest.lua"] then
  validate_manifest_paths(loaded["manifest.lua"], "manifest")
end
if loaded["omoba_stub.lua"] then
  validate_omoba_stub(loaded["omoba_stub.lua"])
end

for _, warning in ipairs(warnings) do
  io.stderr:write(warning .. "\n")
end

if #errors > 0 then
  for _, err in ipairs(errors) do
    io.stderr:write(err .. "\n")
  end
  os.exit(1)
end

print("OK: package valid")
os.exit(0)

