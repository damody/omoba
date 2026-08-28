local json = require("tools.lua.lib.json")
local path = require("tools.lua.lib.path")
local platform = require("tools.lua.lib.platform")
local M = {}

local root = path.repo_root()
local manifest = path.join(root, "tools", "lua-host", "Cargo.toml")
local executable = path.join(root, "tools", "lua-host", "target", "debug", platform.is_windows and "omoba-lua-host.exe" or "omoba-lua-host")

local function ensure()
  if path.is_file(executable) then return end
  local command = "cargo build --manifest-path " .. path.quote(manifest)
  local ok, _, code = os.execute(command)
  assert(ok and code == 0, "failed to build omoba-lua-host")
  assert(path.is_file(executable), "omoba-lua-host output missing")
end

function M.call(operation, params)
  ensure()
  local temporary = path.join(os.getenv("TEMP") or os.getenv("TMP") or root,
    string.format("omoba-lua-host-%d-%06d.json", os.time(), math.random(0,999999)))
  path.write(temporary, json.encode({version=1,operation=operation,params=params or {}}), false)
  local pipe = assert(io.popen(path.quote(executable) .. " " .. path.quote(temporary) .. " 2>&1", "r"))
  local output = pipe:read("a"); local ok = pipe:close(); os.remove(temporary)
  local decoded_ok, response = pcall(json.decode, output)
  assert(decoded_ok, "invalid omoba-lua-host response: " .. output)
  assert(ok and response.ok, response.error or ("omoba-lua-host " .. operation .. " failed"))
  return response.result
end

function M.executable() ensure(); return executable end
return M
