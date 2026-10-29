-- Rebuilds the authority, client runtime and Unreal bridge/game from one stable
-- source snapshot. Several sessions may edit the shared checkout; if any input
-- changed while the separate cargo/UE builds ran, the binaries may embed
-- different protocol or content versions (errors.md E018). The whole set is
-- rebuilt until a full pass starts and ends on the same source fingerprint.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,platform=b.lib('path'),b.lib('process'),b.lib('platform')
local lfs=require('lfs')
local M={}

M.INPUT_ROOTS={'omoba-core/src','omoba-sim/src','omoba-template-ids','omoba-content-model/src',
  'omoba-client-runtime/src','omb/src','scripts/lua_data','scripts/base_content/src','scripts/script-abi/src','scripts/ability_kit/src','scripts/heroes',
  'omfue/bridge/src','omfue/codegen/src','omfue/Plugins/OmRuntime/Source','proto','omoba-netem-proxy/src'}
M.MAX_PASSES=8
-- Each pass starts only after the inputs have been still for START_QUIET_SECONDS,
-- so concurrent writers' landings fall between passes instead of inside them.
M.START_QUIET_SECONDS=180
M.MAX_START_WAIT_SECONDS=1800
M.START_POLL_MS=15000
-- A failure while another writer is mid-edit is not a verdict: wait for the
-- inputs to settle (no edit within QUIET_SECONDS) and retry the whole set.
M.QUIET_SECONDS=600
M.BUSY_WAIT_MS=120000
M.MAX_BUSY_RETRIES=15

-- Another UnrealBuildTool on this machine holds the global UBT mutex (another
-- session's build or package). Not a build verdict: wait and retry.
function M.ubt_busy(result)
  local text=(result.stdout or '')..'\n'..(result.stderr or '')
  return text:find('ConflictingInstance',1,true)~=nil or text:find('UnrealBuildTool_Mutex',1,true)~=nil
end

function M.newest_input(root,roots)
  local newest=0
  for row in M.fingerprint(root,roots):gmatch('[^\n]+') do
    local modified=tonumber(row:match('|(%d+)$'))
    if modified and modified>newest then newest=modified end
  end
  return newest
end

-- Size+mtime of every file under the inputs; build outputs are excluded.
function M.fingerprint(root,roots)
  local rows={}
  local function walk(dir,rel)
    for name in lfs.dir(dir) do
      if name~='.' and name~='..' and name~='target' then
        local full=dir..'/'..name
        local attr=lfs.attributes(full)
        if attr and attr.mode=='directory' then walk(full,rel..'/'..name)
        elseif attr and attr.mode=='file' then rows[#rows+1]=('%s/%s|%d|%d'):format(rel,name,attr.size,attr.modification) end
      end
    end
  end
  for _,relative in ipairs(roots or M.INPUT_ROOTS) do
    local dir=path.join(root,relative)
    if path.is_directory(dir) then walk(dir,relative) end
  end
  table.sort(rows)
  return table.concat(rows,'\n')
end

function M.commands(root,ue_root)
  local function cargo(manifest,extra)
    local args={'build','--manifest-path',path.join(root,manifest),'--features','compiled-content-only','--release'}
    for _,value in ipairs(extra or {}) do args[#args+1]=value end
    return {exe='cargo',args=args}
  end
  return {
    cargo('omb/Cargo.toml',{'-p','omobab','--bin','omobab','--bin','moba-config'}),
    cargo('omoba-client-runtime/Cargo.toml'),
    {exe='cargo',args={'build','--manifest-path',path.join(root,'omoba-netem-proxy/Cargo.toml'),'--release'}},
    {exe=platform.lua_executable,args={path.join(root,'scripts','build_ue_moba.lua'),'--build-only','--ue-root',ue_root}},
  }
end

function M.run(options,runner)
  runner=runner or function(command) return process.run(command.exe,command.args,{cwd=b.root,check=false}) end
  local fingerprint=options.fingerprint or function() return M.fingerprint(b.root) end
  local newest=options.newest_input or function() return M.newest_input(b.root) end
  local now=options.now or os.time
  local sleep=options.sleep_ms or function(ms) b.lib('time').sleep_ms(ms) end
  local busy_retries=0
  local pass=0
  while pass<M.MAX_PASSES do
    pass=pass+1
    local waited=0
    while now()-newest()<M.START_QUIET_SECONDS and waited<M.MAX_START_WAIT_SECONDS do
      if waited==0 then print(('consistent build: waiting for %ds of quiet inputs before pass %d'):format(M.START_QUIET_SECONDS,pass)) end
      sleep(M.START_POLL_MS)
      waited=waited+M.START_POLL_MS//1000
    end
    local before=fingerprint()
    local failure
    local ubt_busy=false
    for _,command in ipairs(M.commands(b.root,options.ue_root)) do
      local result=runner(command)
      local failed=result.exit_code~=0
      -- Failures always show the build output; the reason is usually only on stdout.
      if result.stdout and result.stdout~='' and (failed or not options.quiet) then print(result.stdout) end
      if failed then
        if result.stderr and result.stderr~='' then io.stderr:write(result.stderr) end
        failure=('consistent build failed: %s %s exit %s'):format(command.exe,table.concat(command.args,' '),tostring(result.exit_code))
        ubt_busy=M.ubt_busy(result)
        break
      end
    end
    if failure then
      local recent=ubt_busy or now()-newest()<M.QUIET_SECONDS
      if not recent or busy_retries>=M.MAX_BUSY_RETRIES then
        error(failure..(recent and ' (inputs still changing after '..busy_retries..' waits)' or ''),0)
      end
      busy_retries=busy_retries+1
      print((ubt_busy and 'consistent build: another UnrealBuildTool holds the build mutex, waiting %ds (retry %d/%d)'
        or ('consistent build: failure while inputs changed within '..M.QUIET_SECONDS..'s; another writer is mid-edit, waiting %ds (retry %d/%d)'))
        :format(M.BUSY_WAIT_MS//1000,busy_retries,M.MAX_BUSY_RETRIES))
      sleep(M.BUSY_WAIT_MS)
      pass=pass-1
      goto continue
    end
    if fingerprint()==before then
      print(('consistent build: stable after pass %d'):format(pass))
      return pass
    end
    print(('consistent build: sources changed during pass %d; rebuilding the whole set'):format(pass))
    ::continue::
  end
  error('consistent build: sources kept changing for '..M.MAX_PASSES..' passes; another writer is active',0)
end

if arg and arg[0] and arg[0]:match('consistent_build%.lua$') then
  if arg[1]=='--help' then
    print('Usage: tools/lua/lua.exe scripts/consistent_build.lua [--ue-root D:/UE5.8]')
    print('Builds release omobab/moba-config, release client runtime and the Unreal bridge/game, repeating the whole set until no input changed during a pass.')
    return
  end
  local ue_root='D:/UE5.8'
  for index=1,#arg do if arg[index]=='--ue-root' then ue_root=assert(arg[index+1],'--ue-root requires a path') end end
  M.run({ue_root=ue_root})
end
return M
