local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,time,platform=b.lib('path'),b.lib('process'),b.lib('time'),b.lib('platform')
local launch=require('moba_role_launch')
if arg[1]=='--help' then
  print('Usage: tools/lua/lua.exe scripts/run_moba_role_ue.lua [--prepare-only] [--recipe FILE.lua] [--config FILE.toml] [--output NEW_DIRECTORY] [--port 57061] [--profile release|debug] [--ue-root PATH] [--graphics d3d11|d3d12] [--story FOG_2TEAM_DEMO] [--no-build]')
  return
end
local options=launch.options(arg)
if not options.output then
  -- Exclusive mkdir, no reuse/deletion of a previous session directory.
  local lfs=require('lfs')
  local parent=path.mkdir_p(path.join(b.root,'target','role-ue-runs'))
  for suffix=1,1000 do
    local reserved=path.join(parent,os.time()..'-'..suffix)
    if lfs.mkdir(reserved) then options.output=path.join(reserved,'session');break end
  end
  assert(options.output,'cannot reserve run directory')
end
local output=path.absolute(options.output,b.root)
local owns_output=not path.exists(output)
local ok,err=xpcall(function()
local plan=launch.prepare(options,process)
print(('Prepared %d human(s), %d Bot(s), %d Hz: %s'):format(plan.human_count,plan.bot_count,plan.tick_rate_hz,plan.config))
if options.prepare_only then return end
assert(plan.human_count>0,'interactive launch requires a human; use run_moba_headless.lua for all-Bot matches')
local editor,ue_root=launch.editor(options)
local function stage(mode)
  local result=process.run(platform.lua_executable,{path.join(b.root,'scripts','build_ue_moba.lua'),mode,'--ue-root',ue_root},{cwd=b.root})
  print(result.stdout)
end
if not options.no_build then
  stage('--build-only')
  for _,build in ipairs({
    {'--manifest-path',path.join(b.root,'omb','Cargo.toml'),'-p','omobab','--bin','omobab','--features','runtime-lua-content'},
    {'--manifest-path',path.join(b.root,'omoba-client-runtime','Cargo.toml'),'--features','runtime-lua-content'},
  }) do
    table.insert(build,1,'build')
    if options.profile=='release' then build[#build+1]='--release' end
    local result=process.run('cargo',build,{cwd=b.root})
    print(result.stdout)
  end
end
stage('--verify-staged-only')
launch.launch(plan,editor,process,time)
end,debug.traceback)
if not ok then
  -- Never write a failure record into a refused pre-existing output directory.
  if owns_output then
    path.append(path.join(output,'errors.md'),'\n# Role launcher failure\n\n'..tostring(err)..
      '\n\nDecision: fail closed; do not fallback to another build/profile or stop unrelated processes. See retained configuration and process logs.\n')
  end
  error(err,0)
end
