local script = debug.getinfo(1, "S").source:sub(2)
local root = script:gsub("[/\\]docs[/\\]tools[/\\]migrate_sk_callers.lua$", "")
package.path = root .. "/?.lua;" .. package.path
local args = require("tools.lua.lib.args").parse(arg)
local path = require("tools.lua.lib.path")
local lfs = require("lfs")

local stat = path.absolute(args["stat-keys"] or
  path.join(root, "scripts", "script-abi", "src", "stat_keys.rs"))
local src = path.absolute(args["source-root"] or path.join(root, "omb", "src"))
local dry_run = args["dry-run"] == true
assert(path.is_file(stat), "stat key source missing: " .. stat)
assert(path.is_directory(src), "caller source root missing: " .. src)

local mapping = {}
for pascal, wire in path.read(stat):gmatch('StatKey::([%w_]+)%s*=>%s*"([a-z0-9_]+)"') do
  mapping[wire:upper()] = pascal
end
local mapping_count = 0
for _ in pairs(mapping) do mapping_count = mapping_count + 1 end
print("Loaded " .. mapping_count .. " StatKey mappings.")

local skip = { BUFF_ID_STUN=true, BUFF_ID_ROOT=true, BUFF_ID_SILENCE=true,
  BUFF_ID_INVISIBLE=true, BUFF_ID_INVULNERABLE=true }
local total, changed = 0, 0
local function visit(dir)
  local names = {}
  for file_name in lfs.dir(dir) do
    if file_name ~= "." and file_name ~= ".." then table.insert(names, file_name) end
  end
  table.sort(names)
  for _, file_name in ipairs(names) do
    local file = path.join(dir, file_name)
    if lfs.attributes(file, "mode") == "directory" then
      visit(file)
    elseif file_name:match("%.rs$") then
      local original, count = path.read(file), 0
      local function replace(prefix, key, suffix)
        if skip[key] or not mapping[key] then return prefix .. key .. suffix end
        count = count + 1
        return "StatKey::" .. mapping[key] ..
          (suffix == ".into()" and ".as_str().into()" or "")
      end
      local value = original
        :gsub("(%f[%w]sk::)([A-Z][A-Z0-9_]+)(%.into%(%))", replace)
        :gsub("(%f[%w]stat_keys::)([A-Z][A-Z0-9_]+)(%.into%(%))", replace)
        :gsub("(%f[%w]sk::)([A-Z][A-Z0-9_]+)(%f[^%w_])", replace)
        :gsub("(%f[%w]stat_keys::)([A-Z][A-Z0-9_]+)(%f[^%w_])", replace)
      if count > 0 then
        value = value:gsub("use%s+omb_script_abi::stat_keys%s+as%s+sk%s*;",
          "use omb_script_abi::stat_keys::StatKey;")
        if not dry_run then path.write(file, value, true) end
        total, changed = total + count, changed + 1
        print("  " .. file .. ": " .. count ..
          (dry_run and " replacements (dry-run)" or " replacements"))
      end
    end
  end
end
visit(src)
print(string.format("Done: %d replacements across %d files.", total, changed))
