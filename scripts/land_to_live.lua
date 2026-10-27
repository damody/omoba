-- Landing changes into live (E143). Sessions edited live lua_data / codegen / omfue sources
-- directly and left half-finished states that broke every other session's build (E076, E107,
-- E109, E139, the 2026-10-27 lua_data minion change). Live inputs change only through here:
--
--   tools\lua\lua.exe scripts\land_to_live.lua --change SRC=LIVE[@BASE] ... [--manifest FILE.json]
--       [--checks auto|codegen,cargo,ubt] [--dry-run]
--
--   SRC   the new file (a shadow project or workspace copy)
--   LIVE  the live path it replaces (repo-relative or absolute; a new file is allowed)
--   BASE  the live file as it was when the change was started: the change is 3-way merged
--         onto the current live file (git merge-file); a conflict refuses the landing.
--         Without BASE, SRC replaces LIVE as is.
--   --manifest  a JSON array of {source, live, base?} rows instead of / besides --change.
--
-- Under the repo build lock it assembles "live + these changes" outside live:
--   target/land/tree     mirror of the Rust/Lua sources (scripts, omoba-*, specs, omfue/codegen,
--                        omfue/bridge), with the changes applied
--   target/land/shadow   an Unreal shadow project synced from live (stage_ue_shadow), with the
--                        omfue changes applied and OmGenerated regenerated from the tree
-- and runs the checks the changed paths need (auto): om_codegen over the tree's lua_data;
-- cargo check of base_content (and of the bridge when it or the client runtime changed);
-- UBT of the shadow when omfue sources, lua_data or codegen changed. Only when all pass, and
-- no live file changed meanwhile, are the merged files written into live, together.
-- check_live_consistent.lua runs the same checks with no changes.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

-- Sources mirrored into the candidate tree for codegen and cargo (targets excluded).
M.TREE={'rust-toolchain.toml','omoba-core','omoba-content-model','omoba-template-ids','omoba-sim',
  'omoba-client-runtime','specs','proto','scripts','omfue/codegen','omfue/bridge','omb','log4rs','erps',
  'tools/lua'} -- build scripts run the repo's own Lua (scripts/base_content/build.rs)
M.SKIP={target=true,['.git']=true,log=true}
M.CHECKS={'codegen','cargo','ubt'}

local function deps_of(deps)
  deps=deps or {}
  local b=require('_bootstrap')
  local d={root=deps.root or b.root,path=deps.path or b.lib('path'),lfs=deps.lfs or require('lfs'),
    process=deps.process or b.lib('process'),platform=deps.platform or b.lib('platform'),
    hash=deps.sha256 or b.lib('hash').sha256,json=deps.json or b.lib('json'),log=deps.log or print,
    lock=deps.lock or require('build_lock'),shadow=deps.shadow or require('stage_ue_shadow'),
    ue_root=deps.ue_root}
  return d
end

local function slash(value) return (value:gsub('\\','/')) end

-- Repo-relative, forward-slash path of a live file; refuses paths outside the repo or in target/.
function M.relative(root,file,d)
  local absolute=slash(d.path.absolute(file,root))
  local base=slash(root):gsub('/$','')..'/'
  assert(absolute:sub(1,#base):lower()==base:lower(),'live path outside the repository: '..file)
  local relative=absolute:sub(#base+1)
  assert(relative~='' and not relative:lower():match('^target/') and not relative:match('^%.git/'),
    'not a live input: '..file)
  return relative
end

function M.parse(argv)
  local options={changes={},checks='auto'}
  local i=1
  local function add(source_file,live,base) options.changes[#options.changes+1]={source=source_file,live=live,base=base} end
  while i<=#argv do
    local v=argv[i]
    if v=='--change' then
      i=i+1;local spec=assert(argv[i],'--change needs SRC=LIVE[@BASE]')
      local source_file,rest=spec:match('^(.-)=(.+)$');assert(source_file,'--change needs SRC=LIVE[@BASE]: '..spec)
      local live,base=rest:match('^(.-)@(.+)$');add(source_file,live or rest,base)
    elseif v=='--manifest' then i=i+1;options.manifest=assert(argv[i],'--manifest needs a file')
    elseif v=='--checks' then i=i+1;options.checks=assert(argv[i],'--checks needs a list')
    elseif v=='--dry-run' then options.dry_run=true
    elseif v=='--ue-root' then i=i+1;options.ue_root=assert(argv[i],'--ue-root needs a path')
    elseif v=='--help' or v=='-h' then options.help=true
    else error('unknown argument: '..v) end
    i=i+1
  end
  return options
end

-- Which checks the changed paths need.
function M.needs(relatives)
  local need={codegen=true} -- cheap, and every landing must leave live generating
  for _,relative in ipairs(relatives) do
    local lower=relative:lower()
    local lua_data=lower:match('^scripts/lua_data/')~=nil
    local codegen=lower:match('^omfue/codegen/')~=nil
    local omfue=lower:match('^omfue/')~=nil and not codegen and not lower:match('^omfue/bridge/')
    if lua_data or codegen or omfue then need.ubt=true end
    if lua_data or codegen or lower:match('%.rs$') or lower:match('cargo%.toml$') or lower:match('cargo%.lock$') then need.cargo=true end
    if lower:match('^omfue/bridge/') or lower:match('^omoba%-client%-runtime/') then need.cargo=true;need.bridge=true end
    -- The authority: its own crate and the shared crates it is built from.
    if lower:match('^omb/') or lower:match('^log4rs/') or lower:match('^erps/') or lower:match('^omoba%-core/')
      or lower:match('^omoba%-sim/') or lower:match('^omoba%-template%-ids/') or lower:match('^scripts/script%-abi/') then
      need.cargo=true;need.server=true
    end
  end
  return need
end

local function checks_from(text,relatives)
  if text=='auto' then return M.needs(relatives) end
  local need={}
  for name in text:gmatch('[^,]+') do
    assert(name=='codegen' or name=='cargo' or name=='ubt' or name=='bridge' or name=='server','unknown check: '..name)
    need[name]=true
  end
  if need.bridge or need.server then need.cargo=true end
  return need
end

local function read(d,file) return d.path.is_file(file) and d.path.read(file,true) or nil end

-- The merged content of every change against the current live file.
function M.plan(changes,d)
  local plan={}
  local seen={}
  for _,change in ipairs(changes) do
    local relative=M.relative(d.root,change.live,d)
    assert(not seen[relative:lower()],'the same live file twice: '..relative);seen[relative:lower()]=true
    local live=d.path.join(d.root,relative)
    local new=assert(read(d,d.path.absolute(change.source,d.root)),'missing change source: '..change.source)
    local current=read(d,live)
    local content=new
    if change.base then
      local base=assert(read(d,d.path.absolute(change.base,d.root)),'missing base: '..change.base)
      if current==nil then error('live file was removed since its base: '..relative,0) end
      if current~=base then
        local result=d.process.run('git',{'merge-file','-p',live,d.path.absolute(change.base,d.root),
          d.path.absolute(change.source,d.root)},{cwd=d.root,check=false,label='merge '..relative})
        assert(result.exit_code==0,'3-way merge conflict, nothing landed: '..relative)
        content=result.stdout
      end
    end
    plan[#plan+1]={relative=relative,live=live,content=content,live_before=current}
  end
  return plan
end

local function write(d,file,content)
  d.path.mkdir_p(d.path.parent(file))
  d.path.write(file,content,true,true)
end

local function run(d,label,exe,args,options)
  d.log('[land] '..label)
  local result=d.process.run(exe,args,{cwd=options and options.cwd or d.root,env=options and options.env,check=false,label=label})
  if result.exit_code~=0 then
    error(('%s failed (exit %s)\n%s%s'):format(label,tostring(result.exit_code),
      tostring(result.stdout or ''):sub(-4000),tostring(result.stderr or ''):sub(-4000)),0)
  end
  return result
end

-- Builds the candidate (tree and, for ubt, shadow) and runs the checks. Raises on failure.
function M.check(plan,need,d)
  local work=d.path.join(d.root,'target','land')
  local tree=d.path.join(work,'tree')
  local env={CARGO_TARGET_DIR=d.path.join(work,'cargo-target')}
  for _,entry in ipairs(M.TREE) do
    local from=d.path.join(d.root,entry)
    if d.path.is_directory(from) then d.shadow.mirror(from,d.path.join(tree,entry),{root=d.root},M.SKIP)
    elseif d.path.is_file(from) then write(d,d.path.join(tree,entry),d.path.read(from,true)) end
  end
  local shadow=d.path.join(work,'shadow')
  if need.ubt then d.shadow.sync(shadow,{root=d.root,log=d.log}) end
  for _,row in ipairs(plan) do
    write(d,d.path.join(tree,row.relative),row.content)
    local inside=row.relative:match('^omfue/(.+)$')
    if need.ubt and inside then write(d,d.path.join(shadow,inside),row.content) end
  end
  local generated=need.ubt and d.path.join(shadow,'Plugins','OmRuntime','Source','OmGenerated')
    or d.path.join(work,'codegen-out')
  if need.codegen then
    run(d,'om_codegen over the candidate lua_data','cargo',{'run','--manifest-path',d.path.join(tree,'omfue','codegen','Cargo.toml'),
      '--','--content-root',d.path.join(tree,'scripts','lua_data'),'--out',generated},{env=env})
  end
  if need.cargo then
    run(d,'cargo check base_content','cargo',{'check','--manifest-path',d.path.join(tree,'scripts','Cargo.toml'),
      '-p','base_content','--features','compiled-content-only'},{env=env})
  end
  if need.server then
    run(d,'cargo check omobab','cargo',{'check','--manifest-path',d.path.join(tree,'omb','Cargo.toml'),
      '-p','omobab','--features','compiled-content-only'},{env=env})
  end
  if need.bridge then
    run(d,'cargo check bridge','cargo',{'check','--manifest-path',d.path.join(tree,'omfue','bridge','Cargo.toml')},{env=env})
    if need.ubt then
      run(d,'C ABI header for the shadow','cbindgen',{'--config',d.path.join(tree,'omfue','bridge','cbindgen.toml'),'--crate','om_bridge',
        '--output',d.path.join(shadow,'Plugins','OmRuntime','Source','ThirdParty','OmBridge','include','om_bridge.h'),
        d.path.join(tree,'omfue','bridge')},{env=env})
    end
  end
  if need.ubt then
    local ue_root=d.ue_root or os.getenv('UE_5_8_ROOT') or os.getenv('UE_ROOT') or 'D:/UE5.8'
    d.shadow.build(shadow,d.path.absolute(ue_root,d.root),{root=d.root,log=d.log,process=d.process})
  end
end

function M.run(options,deps)
  local d=deps_of(deps)
  if options.ue_root then d.ue_root=options.ue_root end
  local changes={}
  for _,change in ipairs(options.changes or {}) do changes[#changes+1]=change end
  if options.manifest then
    for _,row in ipairs(d.json.decode(d.path.read(d.path.absolute(options.manifest,d.root)))) do
      changes[#changes+1]={source=assert(row.source,'manifest row needs source'),live=assert(row.live,'manifest row needs live'),base=row.base}
    end
  end
  local lock <close> = d.lock.acquire(d.root,'land_to_live '..#changes..' file(s)',{log=d.log})
  local plan=M.plan(changes,d)
  local relatives={}
  for _,row in ipairs(plan) do relatives[#relatives+1]=row.relative end
  local need=checks_from(options.checks or 'auto',relatives)
  local names={}
  for _,name in ipairs({'codegen','cargo','server','bridge','ubt'}) do if need[name] then names[#names+1]=name end end
  d.log(('[land] %d file(s); checks: %s'):format(#plan,table.concat(names,', ')))
  local ok,err=pcall(M.check,plan,need,d)
  local report={schema='omoba.land-to-live.v1',files=relatives,checks=names,passed=ok,error=not ok and tostring(err) or nil,
    landed=false,dry_run=options.dry_run==true}
  if ok and not options.dry_run and #plan>0 then
    -- Nothing is written when any live file changed while the checks ran.
    for _,row in ipairs(plan) do
      if read(d,row.live)~=row.live_before then
        report.error='live changed during the checks, nothing landed (run again): '..row.relative
        ok=false;break
      end
    end
    if ok then
      for _,row in ipairs(plan) do write(d,row.live,row.content) end
      report.landed=true
    end
  end
  d.path.mkdir_p(d.path.join(d.root,'target','land'))
  d.path.write(d.path.join(d.root,'target','land','last.json'),d.json.encode(report),true)
  if report.landed then d.log(('[land] landed %d file(s) into live after %s'):format(#plan,table.concat(names,', ')))
  elseif report.passed and not report.error then d.log('[land] checks passed'..(options.dry_run and ' (dry run, nothing written)' or ''))
  else d.log('[land] NOT landed: '..tostring(report.error)) end
  return report
end

function M.usage()
  return 'Usage: tools/lua/lua.exe scripts/land_to_live.lua --change SRC=LIVE[@BASE] ... [--manifest FILE.json] [--checks auto|codegen,cargo,server,bridge,ubt] [--dry-run] [--ue-root PATH]'
end

function M.main(argv,deps)
  local options=M.parse(argv)
  if options.help then print(M.usage());return end
  assert(#options.changes>0 or options.manifest,'nothing to land; see --help (check_live_consistent.lua checks live as it is)')
  local report=M.run(options,deps)
  if not report.passed or report.error then os.exit(1) end
  return report
end

if not pcall(debug.getlocal,4,1) then M.main(arg) end
return M
