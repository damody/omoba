-- Authoritative single-lane acceptance. No Editor or renderer is required.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(script:match('^(.*)[/\\]'))
package.path = dir .. '/?.lua;' .. package.path
local bootstrap = require('_bootstrap')
local path, process = bootstrap.lib('path'), bootstrap.lib('process')
local batch = require('moba_headless_batch')
local root = bootstrap.root
local forwarded = {}
local plan_only = false
local recipe_seen = false
local index = 1
while index <= #arg do
  if arg[index] == '--role-plan-lua' then
    assert(not recipe_seen,'duplicate role-plan-lua option')
    recipe_seen = true
    local input = assert(arg[index+1],'role-plan-lua requires a path')
    local output = path.join(root,'omb/target/moba-headless',
      ('role-plan-%016x.json'):format(math.random(0,math.maxinteger)))
    assert(not path.exists(output),'role plan output collision')
    require('export_moba_role_plan').export(input,output)
    forwarded[#forwarded+1]='--role-plan'; forwarded[#forwarded+1]=output
    index=index+2
  else
    if arg[index] == '--plan-only' then plan_only = arg[index+1] == 'true' end
    forwarded[#forwarded+1]=arg[index]; index=index+1
  end
end
local function run(label, args)
  print('[moba-headless] ' .. label)
  local result = process.run('cargo', args, {cwd = root, env={OMB_LUA_CONTENT='0',OMB_LUA_HOT_RELOAD='0'}, check = false})
  io.write(result.stdout or '')
  io.stderr:write(result.stderr or '')
  if result.exit_code ~= 0 then os.exit(result.exit_code) end
end
local function reserve_report_dir()
  local parent = path.mkdir_p(path.join(root, 'omb', 'target', 'moba-headless-runs'))
  for suffix = 1, 1000 do
    local candidate = path.join(parent, tostring(os.time()) .. '-' .. suffix)
    if require('lfs').mkdir(candidate) then return candidate end
  end
end
local report_args, report = batch.with_single_report(forwarded, reserve_report_dir)
batch.assert_fresh_output(report, batch.paired_diagnostic_path(report), path.exists)
if not plan_only then
  run('build generated script DLL', {'build', '--manifest-path', path.join(root, 'scripts/Cargo.toml'),
    '-p', 'base_content', '--features', 'compiled-content-only', '--release'})
end
local args = {'run', '--manifest-path', path.join(root, 'omb/Cargo.toml'),
  '-p', 'omobab', '--bin', 'moba-headless', '--features', 'compiled-content-only', '--',
  '--scripts-dir', path.join(root, 'scripts/target/release')}
for _, value in ipairs(report_args) do args[#args + 1] = value end
run((plan_only and 'validate match configuration only' or 'play a match and replay every tick') .. ': ' .. report, args)
