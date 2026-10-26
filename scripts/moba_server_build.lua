-- Dedicated authority build: no renderer, bridge, restart tool or editor required.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,platform=b.lib('path'),b.lib('platform')
local M={}
local function profile(options)
  local value=options.profile or 'release'
  assert(value=='debug' or value=='release','invalid server build profile')
  return value
end
function M.commands(options)
  local selected=profile(options)
  local commands={}
  local function build(manifest,package_name,binary)
    local args={'build','--manifest-path',path.join(b.root,manifest),'-p',package_name,
      '--features','compiled-content-only'}
    if binary then args[#args+1]='--bin';args[#args+1]=binary end
    if selected=='release' then args[#args+1]='--release' end
    commands[#commands+1]={exe='cargo',args=args}
  end
  build('scripts/Cargo.toml','base_content')
  build('omb/Cargo.toml','omobab','moba-config')
  build('omb/Cargo.toml','omobab','omobab')
  commands[#commands+1]={exe=platform.lua_executable,args={
    path.join(b.root,'scripts/dev_run_freshness.lua'),'--action','stage-dll','--profile',selected}}
  return commands
end
function M.build(options,process)
  for _,command in ipairs(M.commands(options)) do
    local result=process.run(command.exe,command.args,{cwd=b.root,check=false})
    if result.stdout and result.stdout~='' then print(result.stdout) end
    if result.stderr and result.stderr~='' then io.stderr:write(result.stderr) end
    assert(result.exit_code==0,'dedicated authority build/stage failed: '..command.exe..
      ' exit '..tostring(result.exit_code))
  end
end
function M.verify(options,dependencies)
  local selected=profile(options)
  dependencies=dependencies or {is_file=path.is_file,sha256=b.lib('hash').sha256}
  for _,binary in ipairs({'omobab.exe','moba-config.exe'}) do
    local file=path.join(b.root,'omb/target',selected,binary)
    assert(dependencies.is_file(file),'missing dedicated authority executable: '..file)
  end
  -- The staged DLL is shared by every process on this machine. A same-machine
  -- --server-only next to --connect clients finds the frontend-staged debug DLL;
  -- accept it when it is a current build of either profile, never a stale copy.
  local stage_contract=require('moba_stage_contract')
  local staged=path.join(b.root,'scripts/base_content.dll')
  local first_err
  for _,candidate in ipairs({selected,selected=='release' and 'debug' or 'release'}) do
    local built=path.join(b.root,'scripts/target',candidate,'base_content.dll')
    if candidate==selected or dependencies.is_file(built) then
      local ok,report=pcall(stage_contract.verify,{{role='base_content',built=built,copies={staged}}},dependencies)
      if ok then report.base_content_profile=candidate;return report end
      first_err=first_err or report
    end
  end
  error(first_err,0)
end
return M
