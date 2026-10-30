-- E333: the UE frontend builds the bridge and base_content, stages them, and verifies the stage
-- against one target directory: ue_bridge_stage.rust_target (OM_RUST_RUN_BINARIES or the crate's
-- own target). The build, the stage sources and moba_stage_contract must name the same files,
-- with the variable set and without it.
package.path = "scripts/?.lua;" .. package.path
local checks = 0
local function ok(cond, name) assert(cond, name); checks = checks + 1 end
local function read(p) local f = assert(io.open(p, 'rb')); local s = f:read('a'); f:close(); return s end

ok(read('scripts/build_ue_bridge.lua'):find("'--target-dir',require('ue_bridge_stage').rust_target(root,'omfue','bridge','target')", 1, true),
  'the bridge build names the stage target directory')
ok(read('scripts/build_ue_moba.lua'):find("'--target-dir', require('ue_bridge_stage').rust_target(root, 'scripts', 'target')", 1, true),
  'the script DLL build names the stage target directory')

-- Prints "bridge|base_content" as the stage reads them and as the contract verifies them.
local probe = [[package.path='scripts/?.lua;'..package.path
local s=require('ue_bridge_stage') local c=require('moba_stage_contract')
local src={} for _,i in ipairs(s.sources('R','h')) do src[i.role]=i.source end
local con={} for _,i in ipairs(c.bindings('R','P')) do con[i.role]=i.built end
print(src.bridge..'|'..src.base_content..'|'..con.bridge..'|'..con.base_content)]]
local probe_file = os.tmpname()
do local f = assert(io.open(probe_file, 'wb')); f:write(probe); f:close() end
local lua = arg[-1] or 'tools\\lua\\lua.exe'
local function run(env)
  local cmd = (env and ('set "OM_RUST_RUN_BINARIES=' .. env .. '"&& ') or 'set "OM_RUST_RUN_BINARIES="&& ')
    .. '"' .. lua .. '" "' .. probe_file .. '"'
  local h = assert(io.popen('cmd /d /c "' .. cmd .. '"'))
  local out = h:read('a'); h:close()
  local a, b2, c, d = out:match('([^|\r\n]+)|([^|\r\n]+)|([^|\r\n]+)|([^|\r\n]+)')
  assert(a, 'probe failed: ' .. out)
  return a, b2, c, d
end
for _, env in ipairs({ false, 'D:\\elsewhere\\session-target' }) do
  local sb, sc, cb, cc = run(env or nil)
  local norm = function(p) return (p:gsub('\\', '/')) end
  ok(norm(sb) == norm(cb) and norm(sc) == norm(cc), 'stage and contract agree (' .. tostring(env) .. '): ' .. sb .. ' / ' .. cb)
  if env then ok(norm(cb):find('D:/elsewhere/session-target', 1, true), 'the contract follows OM_RUST_RUN_BINARIES')
  else ok(norm(cb):find('omfue/bridge/target/debug', 1, true), 'without it, the bridge crate target') end
end
os.remove(probe_file)
print('frontend_target_dir_test: ' .. checks .. ' checks passed')
