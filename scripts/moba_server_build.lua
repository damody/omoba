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
  -- The script DLL set: base_content and every hero DLL (script_dlls.lua).
  commands[#commands+1]={exe='cargo',args=require('script_dlls').cargo_args({'build','--manifest-path',
    path.join(b.root,'scripts/Cargo.toml')},'compiled-content-only',selected=='release' and {'--release'} or nil)}
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
  local artifacts=require('run_artifacts') -- OM_RUST_RUN_BINARIES aware (E125)
  for _,binary in ipairs({'omobab.exe','moba-config.exe'}) do
    local file=artifacts.rust_binary(b.root,'omb',selected,binary)
    assert(dependencies.is_file(file),'missing dedicated authority executable: '..file)
  end
  -- No shared staged DLL is checked: each launch copies the current base_content build into
  -- its own run directory (run_artifacts.lua, E124), so only a build has to exist.
  for _,candidate in ipairs({selected,selected=='release' and 'debug' or 'release'}) do
    local built=artifacts.rust_binary(b.root,'scripts',candidate,'base_content.dll')
    if dependencies.is_file(built) then
      for _,module in ipairs(require('script_dlls').modules()) do
        local file=artifacts.rust_binary(b.root,'scripts',candidate,module.file)
        assert(dependencies.is_file(file),'missing script DLL beside base_content: '..file)
      end
      return {schema_version=1,scope='dedicated-authority-build',base_content_profile=candidate,base_content=built}
    end
  end
  error('missing base_content build: scripts/target/{debug,release}/base_content.dll',0)
end
return M
