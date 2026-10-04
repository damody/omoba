-- Exercise the real build entry's early, read-only stage gate without touching
-- any user's DLL or launching a build/Editor.
local function verify(built, staged)
  local hash_calls, process_calls = 0, 0
  local modules = {
    path = {join = function(...) return table.concat({...}, '/') end},
    process = {run = function() process_calls = process_calls + 1; error('unexpected process') end},
    platform = {},
    hash = {sha256 = function()
      hash_calls = hash_calls + 1
      local value
      if hash_calls == 1 then value = built else value = staged end
      assert(value, 'missing DLL fixture')
      return value
    end},
  }
  local env = setmetatable({
    arg = {'--verify-staged-only'}, package = {path = ''}, print = function() end,
    require = function(name)
      assert(name == '_bootstrap')
      return {root = 'fixture', lib = function(key) return assert(modules[key]) end}
    end,
  }, {__index = _G})
  local entry = assert(loadfile('scripts/build_ue_moba.lua', 't', env))
  local ok, error_message = pcall(entry)
  assert(process_calls == 0, 'verify-only must not build, stage or launch')
  return ok, tostring(error_message)
end
assert(verify(string.rep('a', 64), string.rep('a', 64)))
local ok, message = verify(string.rep('a', 64), string.rep('b', 64))
assert(not ok and message:find('staged bridge DLL differs', 1, true))
assert(not verify(nil, string.rep('a', 64)))
print('ue_bridge_stage: identical, mismatched and missing DLL cases passed; no process launched')
