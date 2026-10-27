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
  }) do
    if path.is_file(candidate) then
      newest = math.max(newest, path.attributes(candidate).modification or 0)
    end
  end
  local function visit(directory)
    for name in lfs.dir(directory) do
      if name~='.' and name~='..' then
        local file=path.join(directory,name)
        local attributes=assert(lfs.symlinkattributes(file),'cannot inspect Lua host source: '..file)
        if attributes.mode=='directory' then visit(file)
        elseif attributes.mode=='file' and name:match('%.rs$') then
          newest=math.max(newest,attributes.modification or 0)
        end
      end
    end
  end
  visit(path.join(root,'tools','lua-host','src'))
  return newest
end

-- Calls run a copy of the host, never the build output (E137, as E125 for the game): a
-- long-running host (a supervised workflow) locked target/debug/omoba-lua-host.exe, so a
-- host source change made every later cargo build fail and every Lua workflow on the
-- machine stopped with "failed to build omoba-lua-host". The copy lives in
-- target/run/<size>-<time>/ and is made once per build.
local run_copy
local function copy_of(source)
  local attributes = path.attributes(source)
  local key = string.format("%d-%d", attributes.size or 0, attributes.modification or 0)
  local directory = path.join(root, "tools", "lua-host", "target", "run", key)
  local copy = path.join(directory, platform.is_windows and "omoba-lua-host.exe" or "omoba-lua-host")
  if path.is_file(copy) and path.attributes(copy).size == attributes.size then return copy end
  path.mkdir_p(directory)
  local partial = copy .. "." .. os.time() .. "-" .. math.random(0, 999999) .. ".partial"
  local input = assert(io.open(source, "rb"));local output = assert(io.open(partial, "wb"))
  output:write(input:read("a"));input:close();output:close()
  if not platform.is_windows then os.execute("chmod +x " .. path.quote(partial)) end
  -- Another process may have made the same copy meanwhile; either one is identical.
  if not os.rename(partial, copy) then os.remove(partial) end
  assert(path.is_file(copy), "omoba-lua-host run copy missing: " .. copy)
  return copy
end

local function ensure()
  if run_copy then return run_copy end
  if not path.is_file(executable)
      or (path.attributes(executable).modification or 0) < newest_source_time() then
    local command = "cargo build --manifest-path " .. path.quote(manifest)
    local ok, _, code = os.execute(command)
    -- A host built before still serves every existing operation; only a missing one fails.
    assert((ok and code == 0) or path.is_file(executable), "failed to build omoba-lua-host")
    if not (ok and code == 0) then
      io.stderr:write("[lua-host] rebuild failed; using the previous host build\n")
    end
  end
  assert(path.is_file(executable), "omoba-lua-host output missing")
  run_copy = copy_of(executable)
  return run_copy
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
  local ok, _, code = os.execute(path.quote(ensure()) .. " " .. path.quote(temporary)
    .. " " .. path.quote(response_file))
  local output = path.is_file(response_file) and path.read(response_file) or ""
  assert(os.remove(temporary), "failed to remove Lua host request: " .. temporary)
  if path.exists(response_file) then
    assert(os.remove(response_file), "failed to remove Lua host response: " .. response_file)
  end
  assert(lfs.rmdir(reserved), 'failed to remove owned Lua host directory: ' .. reserved)
  local decoded_ok, response = pcall(json.decode, output)
  assert(decoded_ok, "invalid omoba-lua-host response (exit " .. tostring(code) .. "): " .. output)
  assert(ok and code == 0 and response.ok, response.error or ("omoba-lua-host " .. operation .. " failed"))
  return response.result
end

function M.executable() return ensure() end
return M
