-- Session shadow Unreal project (E116, E124): a session builds and tests its own Unreal
-- binaries in a shadow project while the user's editor maps live omfue's DLLs, then runs
-- them through OM_UE_RUN_BINARIES (ue_run_snapshot.lua). Each session used to keep its own
-- copy script (shell mirrors plus a manual cp of om_bridge.dll / base_content.dll), and a
-- bridge or base_content rebuild by another session made the copies disagree with the
-- launch preflight. This entry is the one shared flow:
--
--   tools\lua\lua.exe scripts\stage_ue_shadow.lua --project DIR [--sync] [--build] [--ue-root PATH]
--
--   --sync   mirror live omfue's uproject, Source, Config and plugin sources into DIR (only
--            changed files are copied, so UBT stays incremental; a new shadow is seeded once
--            with live Binaries and Intermediate/Build). Binaries are never mirrored after.
--   (always) generate content C++ into DIR, build the bridge and debug base_content, write
--            the header with cbindgen and stage header/lib/bridge/base_content into DIR with
--            a stage record (ue_bridge_stage.lua). Live omfue is never written.
--   --build  build OmGameEditor Win64 Development for DIR with UBT (-WaitMutex).
--
-- A launch with OM_UE_RUN_BINARIES=DIR then checks DIR's bridge against its stage record and
-- runs its own base_content copy (run_artifacts.lua), so later rebuilds elsewhere never
-- block it.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

-- Live entries mirrored into a shadow; plugin Binaries/Intermediate are the shadow's own.
M.PROJECT_DIRS={'Source','Config'}
M.PLUGIN_SKIP={Binaries=true,Intermediate=true}
-- Seeded once into a new shadow so its first UBT build is incremental.
M.SEED_DIRS={'Binaries','Intermediate/Build'}

local function deps_of(deps)
  deps=deps or {}
  local b=require('_bootstrap')
  return {root=deps.root or b.root,path=deps.path or b.lib('path'),lfs=deps.lfs or require('lfs'),
    process=deps.process or b.lib('process'),platform=deps.platform or b.lib('platform'),
    log=deps.log or print,stage=deps.stage or require('ue_bridge_stage')}
end

local function entries(lfs,dir)
  local list={}
  for name in lfs.dir(dir) do if name~='.' and name~='..' then list[#list+1]=name end end
  table.sort(list)
  return list
end

local function copy_file(d,from,to)
  local input=assert(io.open(from,'rb'))
  local output=assert(io.open(to,'wb'))
  while true do
    local chunk=input:read(4*1024*1024)
    if not chunk then break end
    output:write(chunk)
  end
  input:close();output:close()
  -- The copy is new to UBT even when live's change is older than the shadow's last build (another
  -- session landed it earlier): stamp it now, never in the past, or UBT keeps a stale object (E136).
  local modified=math.max(d.lfs.attributes(from,'modification'),os.time())
  d.lfs.touch(to,modified,modified)
end

local function remove(d,entry)
  if d.lfs.symlinkattributes(entry,'mode')=='directory' then
    for _,name in ipairs(entries(d.lfs,entry)) do remove(d,d.path.join(entry,name)) end
    assert(d.lfs.rmdir(entry))
  else
    assert(os.remove(entry))
  end
end

-- Makes TO equal FROM: copies files whose size differs or whose source is newer than the copy, removes
-- what FROM lacks.
-- Links are skipped on both sides (never followed, never deleted through). SKIP names
-- top-level children of FROM that are left alone. Returns the number of files copied.
function M.mirror(from,to,deps,skip)
  local d=deps_of(deps)
  local copied=0
  d.path.mkdir_p(to)
  local wanted={}
  for _,name in ipairs(entries(d.lfs,from)) do
    if not (skip and skip[name]) then
      local source_entry,target_entry=d.path.join(from,name),d.path.join(to,name)
      local mode=d.lfs.symlinkattributes(source_entry,'mode')
      if mode=='directory' then
        wanted[name]=true
        if d.lfs.symlinkattributes(target_entry,'mode')=='file' then remove(d,target_entry) end
        copied=copied+M.mirror(source_entry,target_entry,d)
      elseif mode=='file' then
        wanted[name]=true
        local a,t=d.lfs.attributes(source_entry),d.lfs.symlinkattributes(target_entry)
        if t and t.mode=='directory' then remove(d,target_entry);t=nil end
        if not t or t.size~=a.size or a.modification>t.modification then
          copy_file(d,source_entry,target_entry);copied=copied+1
        end
      end
    end
  end
  for _,name in ipairs(entries(d.lfs,to)) do
    local entry=d.path.join(to,name)
    if not wanted[name] and not (skip and skip[name]) and d.lfs.symlinkattributes(entry,'mode')~='link' then
      remove(d,entry)
    end
  end
  return copied
end

function M.sync(project,deps)
  local d=deps_of(deps)
  local live=d.path.join(d.root,'omfue')
  assert(d.path.absolute(project,d.root):lower()~=live:lower(),'the shadow project cannot be live omfue')
  local fresh=not d.path.is_directory(d.path.join(project,'Binaries'))
  d.path.mkdir_p(project)
  copy_file(d,d.path.join(live,'om.uproject'),d.path.join(project,'om.uproject'))
  local copied=0
  for _,name in ipairs(M.PROJECT_DIRS) do
    copied=copied+M.mirror(d.path.join(live,name),d.path.join(project,name),d)
  end
  for _,plugin in ipairs(entries(d.lfs,d.path.join(live,'Plugins'))) do
    local from=d.path.join(live,'Plugins',plugin)
    if d.path.is_directory(from) then
      copied=copied+M.mirror(from,d.path.join(project,'Plugins',plugin),d,M.PLUGIN_SKIP)
      if fresh and d.path.is_directory(d.path.join(from,'Binaries')) then
        M.mirror(d.path.join(from,'Binaries'),d.path.join(project,'Plugins',plugin,'Binaries'),d)
      end
    end
  end
  if fresh then
    for _,relative in ipairs(M.SEED_DIRS) do
      local from=d.path.join(live,relative)
      if d.path.is_directory(from) then M.mirror(from,d.path.join(project,relative),d) end
    end
  end
  d.log(('[ue-shadow] synced %s (%d files copied%s)'):format(project,copied,fresh and ', seeded build state' or ''))
  return copied
end

local function run(d,label,exe,args,cwd)
  d.log('[ue-shadow] '..label);io.stdout:flush()
  local result=d.process.run(exe,args,{cwd=cwd or d.root,check=false})
  if result.exit_code~=0 then io.stderr:write(result.stdout or '',result.stderr or '') end
  assert(result.exit_code==0,label..' failed with exit '..tostring(result.exit_code))
  return result
end

function M.stage(project,deps)
  local d=deps_of(deps)
  local ue=d.path.join(d.root,'omfue')
  local plugin=d.path.join(project,'Plugins','OmRuntime')
  assert(d.path.is_directory(plugin),'shadow project has no OmRuntime plugin (run with --sync): '..project)
  if not deps or not deps.process then require('ue_build_prerequisites').require_ready(d.process) end
  run(d,'content codegen','cargo',{'run','--manifest-path',d.path.join(ue,'codegen','Cargo.toml'),'--',
    '--content-root',d.path.join(d.root,'scripts','lua_data'),'--out',d.path.join(plugin,'Source','OmGenerated')})
  run(d,'Rust bridge','cargo',{'build','--manifest-path',d.path.join(ue,'bridge','Cargo.toml')})
  run(d,'script DLLs (debug)','cargo',require('script_dlls').cargo_args({'build','--manifest-path',
    d.path.join(d.root,'scripts','Cargo.toml')},'compiled-content-only'))
  local header=d.path.join(project,'Saved','BuildBridge','om_bridge.h')
  d.path.mkdir_p(d.path.parent(header))
  run(d,'C ABI header','cbindgen',{'--config',d.path.join(ue,'bridge','cbindgen.toml'),'--crate','om_bridge',
    '--output',header,d.path.join(ue,'bridge')})
  return d.stage.stage(project,d.stage.sources(d.root,header,d),{log=d.log})
end

function M.build(project,ue_root,deps)
  local d=deps_of(deps)
  local build=d.path.join(ue_root,'Engine','Build','BatchFiles','Build.bat')
  assert(d.path.is_file(build),'missing Unreal build script: '..build)
  return run(d,'UBT OmGameEditor (shadow)',build,{'OmGameEditor','Win64','Development',
    '-Project='..d.path.join(project,'om.uproject'),'-WaitMutex','-NoHotReloadFromIDE','-NoEngineChanges'},project)
end

function M.main(argv,deps)
  local d=deps_of(deps)
  local options={}
  local i=1
  while i<=#argv do
    local value=argv[i]
    if value=='--project' then i=i+1;options.project=assert(argv[i],'--project requires a directory')
    elseif value=='--ue-root' then i=i+1;options.ue_root=assert(argv[i],'--ue-root requires a path')
    elseif value=='--sync' then options.sync=true
    elseif value=='--build' then options.build=true
    elseif value=='--help' or value=='-h' then
      print('Usage: tools/lua/lua.exe scripts/stage_ue_shadow.lua --project DIR [--sync] [--build] [--ue-root PATH]')
      return
    else error('unknown argument: '..value) end
    i=i+1
  end
  local project=d.path.absolute(assert(options.project,'--project DIR is required'),d.root)
  assert(project:lower()~=d.path.join(d.root,'omfue'):lower(),'the shadow project cannot be live omfue')
  if options.sync then M.sync(project,d) end
  M.stage(project,d)
  if options.build then
    M.build(project,d.path.absolute(options.ue_root or os.getenv('UE_5_8_ROOT') or os.getenv('UE_ROOT') or 'D:/UE5.8'),d)
  end
  d.log('[ue-shadow] ready: set OM_UE_RUN_BINARIES='..project..' for launches that should run it')
end

if not pcall(debug.getlocal,4,1) then M.main(arg) end
return M
