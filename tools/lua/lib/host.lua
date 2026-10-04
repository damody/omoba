local json = require("tools.lua.lib.json")
local path = require("tools.lua.lib.path")
local platform = require("tools.lua.lib.platform")
local lfs = require("lfs")
local M = {}

local root = path.repo_root()
local manifest = path.join(root, "tools", "lua-host", "Cargo.toml")
local executable = path.join(root, "tools", "lua-host", "target", "debug", platform.is_windows and "omoba-lua-host.exe" or "omoba-lua-host")

local function newest_source_time()
  local newest = 0
  for _, candidate in ipairs({
    manifest,
    path.join(root, "tools", "lua-host", "Cargo.lock"),
    path.join(root, "tools", "lua-host", "src", "main.rs"),
  }) do
    if path.is_file(candidate) then
      newest = math.max(newest, path.attributes(candidate).modification or 0)
    end
  end
  return newest
end

local function ensure()
  if path.is_file(executable)
      and (path.attributes(executable).modification or 0) >= newest_source_time() then return end
  local command = "cargo build --manifest-path " .. path.quote(manifest)
  local ok, _, code = os.execute(command)
  assert(ok and code == 0, "failed to build omoba-lua-host")
  assert(path.is_file(executable), "omoba-lua-host output missing")
end

function M.call(operation, params)
  ensure()
  -- Nested Lua processes can repeat time/random values. Reserve an exclusive
  -- directory atomically, not a check-then-write request filename.
  local prefix = path.join(os.getenv("TEMP") or os.getenv("TMP") or root,
    string.format("omoba-lua-host-%d-%06d", os.time(), math.random(0,999999)))
  local reserved
  for attempt = 1, 1000 do
    local candidate = prefix .. '-' .. attempt
    local made, err = lfs.mkdir(candidate)
    if made then reserved = candidate; break end
    assert(path.exists(candidate), 'cannot reserve Lua host directory: ' .. tostring(err))
  end
  assert(reserved, 'Lua host temporary directory reservation exhausted')
  local temporary = path.join(reserved, 'request.json')
  local response_file = temporary .. ".response"
  path.write(temporary, json.encode({version=1,operation=operation,params=params or {}}), false)
  local ok, _, code = os.execute(path.quote(executable) .. " " .. path.quote(temporary)
    .. " " .. path.quote(response_file))
  local output = path.is_file(response_file) and path.read(response_file) or ""
  assert(os.remove(temporary), "failed to remove Lua host request: " .. temporary)
  if path.exists(response_file) then
    assert(os.remove(response_file), "failed to remove Lua host response: " .. response_file)
  end
  assert(lfs.rmdir(reserved), 'failed to remove owned Lua host directory: ' .. reserved)
  local decoded_ok, response = pcall(json.decode, output)
  assert(decoded_ok, "invalid omoba-lua-host response: " .. output)
  assert(ok and code == 0 and response.ok, response.error or ("omoba-lua-host " .. operation .. " failed"))
  return response.result
end

function M.executable() ensure(); return executable end
return M
