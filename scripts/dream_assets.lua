-- Text sources of the project's Niagara systems and materials: DreamFX (DFX/) and DreamShader (DShader/) commandlets.
--
--   tools\lua\lua.exe scripts\dream_assets.lua [--project DIR] [--ue-root PATH] {fx|shader} VERB [-Option[=value] ...]
--   tools\lua\lua.exe scripts\dream_assets.lua pinned [--filter LUA_PATTERN]
--   tools\lua\lua.exe scripts\dream_assets.lua owned [--project DIR]
--   tools\lua\lua.exe scripts\dream_assets.lua ensure [--project DIR] [--ue-root PATH]
--
--   ensure  rebuilds what is missing or older than its source, then runs owned. A clean clone has only the sources.
--   owned   every source under DShader/ and DFX/ has its asset, still marked as built from text, and none was overwritten by
--           another tool. A tool that replaces such an asset drops the mark; one that loads and resaves it keeps the mark but
--           is recorded by OmDreamGuard. Exit 1 when any is missing, unmarked or overwritten and not yet restored.
--   pinned  materials and Niagara systems that a tracked file pins by SHA-256 (source-registry.json, provenance.json, ...).
--           Their owner refuses them once their bytes change, so they cannot become text. Exit 1 when the filter finds any.
--
--   fx      build [-File=F|-All] [-Force]   verify   lint   decompile -Decompile=ASSET [-Out=F]
--           decompile-all -Path=/Game/...   coverage -Path=   asset-diff -Path=   mirror-diff -Path=
--   shader  compile [-Source=F|-All] [-Force]   check [-Source=F|-All]   decompile -Asset=ASSET [-Out=F]
--           list-generated [-All]   dump-graph [-All] [-Out=DIR]
--           decompile-all [--shard I/N] [--filter LUA_PATTERN]  every material, function and instance under Content,
--           one commandlet each (the commandlet takes one asset); run N shards side by side for speed
--
-- DIR is the project to run on, live omfue by default. A verb that saves live assets holds the repo build lock and needs
-- no other editor on that project: two processes saving the same packages race silently. Exit code is the commandlet's.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

M.COMMANDS={
  fx={build='',verify='-Verify',lint='-Lint',decompile='',['decompile-all']='-DecompileAll',coverage='-Coverage',
    ['asset-diff']='-AssetDiff',['mirror-diff']='-MirrorDiff'},
  shader={compile='compile',check='check',decompile='decompile',['decompile-all']='decompile',['list-generated']='list-generated',['dump-graph']='dump-graph'},
}
M.COMMANDLET={fx='DreamFX',shader='DreamShader'}
M.SAVES={fx={build=true,['decompile-all']=true,['mirror-diff']=true},shader={compile=true}}
-- A package's asset registry section names its asset's class in full; nothing else in it does.
M.MATERIAL_CLASSES={'MaterialInstanceConstant','MaterialFunctionMaterialLayerBlend','MaterialFunctionMaterialLayer','MaterialFunction','Material'}
M.SYSTEM_CLASS='/Script/Niagara.NiagaraSystem\0'
M.FLAGS={'-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-nullrhi'}

function M.parse(argv)
  local options={passed={}}
  local i=1
  while i<=#argv do
    local v=argv[i]
    if v=='--project' then i=i+1;options.project=assert(argv[i],'--project needs a directory')
    elseif v=='--ue-root' then i=i+1;options.ue_root=assert(argv[i],'--ue-root needs a path')
    elseif v=='--shard' then
      i=i+1;options.shard,options.shards=(argv[i] or ''):match('^(%d+)/(%d+)$')
      options.shard,options.shards=assert(tonumber(options.shard),'--shard needs I/N'),tonumber(options.shards)
    elseif v=='--filter' then i=i+1;options.filter=assert(argv[i],'--filter needs a pattern')
    elseif not options.tool then options.tool=v
    elseif not options.verb then options.verb=v
    else options.passed[#options.passed+1]=v end
    i=i+1
  end
  local verbs=assert(M.COMMANDS[options.tool or ''],'first argument is fx or shader')
  assert(verbs[options.verb or ''],'unknown '..tostring(options.tool)..' verb: '..tostring(options.verb))
  return options
end

function M.command_args(options,uproject,log)
  local args={uproject,'-run='..M.COMMANDLET[options.tool]}
  local command=M.COMMANDS[options.tool][options.verb]
  if command~='' then args[#args+1]=command end
  for _,v in ipairs(options.passed) do args[#args+1]=v end
  for _,v in ipairs(M.FLAGS) do args[#args+1]=v end
  args[#args+1]='-abslog='..log
  return args
end

-- Object paths of the project's materials, sorted.
function M.material_assets(content)
  local lfs=require('lfs')
  local assets={}
  local function walk(dir)
    for entry in lfs.dir(dir) do
      if entry~='.' and entry~='..' then
        local full=dir..'/'..entry
        if lfs.attributes(full,'mode')=='directory' then walk(full)
        elseif entry:match('%.uasset$') then
          local f=assert(io.open(full,'rb'));local data=f:read('a');f:close()
          for _,class in ipairs(M.MATERIAL_CLASSES) do
            if data:find('/Script/Engine.'..class..'\0',1,true) then
              local name=entry:gsub('%.uasset$','')
              assets[#assets+1]='/Game'..full:sub(#content+1):gsub('%.uasset$','')..'.'..name
              break
            end
          end
        end
      end
    end
  end
  walk(content)
  table.sort(assets)
  return assets
end

-- Package path -> where it is pinned, for FILES {package path -> uasset file}. A pin names the bytes (the asset's SHA-256
-- anywhere in a tracked or new file outside docs and Saved) or the package (a package_sha256 key or package_file row, which
-- still pins an asset that has already drifted from it).
function M.pinned(root,files)
  local b=require('_bootstrap')
  local h,r,j=b.lib('hash'),b.lib('process'),b.lib('json')
  local omfue=root..'/omfue'
  local by_sha,found={},{}
  local function pin(package,where)
    found[package]=found[package] or {}
    found[package][where]=true
  end
  local shas={}
  for package,file in pairs(files) do
    local sha=h.sha256(file)
    by_sha[sha]=by_sha[sha] or {}
    table.insert(by_sha[sha],package)
    shas[#shas+1]=sha
  end
  if #shas==0 then return found end
  local list=root..'/target/dream-pinned-shas.txt'
  local f=assert(io.open(list,'wb'));f:write(table.concat(shas,'\n'),'\n');f:close()
  for _,repo in ipairs{{root,{'.',':!docs',':!target'}},{omfue,{'.',':!Saved'}}} do
    local args={'grep','--untracked','-o','-F','-f',list,'--'}
    for _,v in ipairs(repo[2]) do args[#args+1]=v end
    local result=r.run('git',args,{cwd=repo[1],check=false})
    for line in (result.stdout or ''):gmatch('[^\r\n]+') do
      local where,sha=line:match('^(.*):(%x+)$')
      for _,package in ipairs(by_sha[sha] or {}) do pin(package,where) end
    end
  end
  os.remove(list)
  local tracked=r.run('git',{'ls-files','Content'},{cwd=omfue,check=false}).stdout or ''
  for rel in tracked:gmatch('[^\r\n]+') do
    if rel:match('%.json$') then
      local g=assert(io.open(omfue..'/'..rel,'rb'));local text=g:read('a');g:close()
      if text:find('package_sha256',1,true) or text:find('package_file',1,true) then
        local value=j.decode(text)
        for package in pairs(value.package_sha256 or {}) do if files[package] then pin(package,rel) end end
        for _,row in ipairs(value.materials or {}) do
          local package=type(row)=='table' and row.package_file and row.package_file:match('^omfue/Content/(.*)%.uasset$')
          if package and files['/Game/'..package] then pin('/Game/'..package,rel) end
        end
      end
    end
  end
  return found
end

-- Every material and Niagara system under CONTENT: package path -> file.
function M.text_candidates(content)
  local lfs=require('lfs')
  local files={}
  for _,asset in ipairs(M.material_assets(content)) do
    local package=asset:match('^(.-)%.')
    files[package]=content..package:sub(#'/Game'+1)..'.uasset'
  end
  local function walk(dir)
    for entry in lfs.dir(dir) do
      if entry~='.' and entry~='..' then
        local full=dir..'/'..entry
        if lfs.attributes(full,'mode')=='directory' then walk(full)
        elseif entry:match('%.uasset$') then
          local f=assert(io.open(full,'rb'));local data=f:read('a');f:close()
          if data:find(M.SYSTEM_CLASS,1,true) then files['/Game'..full:sub(#content+1):gsub('%.uasset$','')]=full end
        end
      end
    end
  end
  walk(content)
  return files
end

function M.report_pinned(root,filter)
  local files={}
  for package,file in pairs(M.text_candidates(root..'/omfue/Content')) do
    if not filter or package:match(filter) then files[package]=file end
  end
  -- An asset that already has a source has moved past its pin: its owner reads the source.
  local function has_source(package)
    local rel=package:gsub('^/Game/','')
    for _,kind in ipairs{'/omfue/DShader/'..rel..'.dss','/omfue/DShader/'..rel..'.dsi','/omfue/DFX/'..rel..'.dfs'} do
      local f=io.open(root..kind,'rb')
      if f then f:close();return true end
    end
    return false
  end
  for package in pairs(files) do if has_source(package) then files[package]=nil end end
  local found,names,count=M.pinned(root,files),{},0
  for package in pairs(files) do count=count+1 end
  for package in pairs(found) do names[#names+1]=package end
  table.sort(names)
  for _,package in ipairs(names) do
    local where={}
    for file in pairs(found[package]) do where[#where+1]=file end
    table.sort(where)
    print(package..'\t'..table.concat(where,', '))
  end
  print(('[dream-assets] %d checked, %d pinned'):format(count,#names))
  return #names==0 and 0 or 1
end

-- Sources of PROJECT whose asset is missing or older than the source: tool ('shader' or 'fx') -> count.
function M.stale(project)
  local lfs=require('lfs')
  local stale={shader=0,fx=0}
  local function walk(tool,base,dir)
    for entry in lfs.dir(dir) do
      if entry~='.' and entry~='..' and entry~='.dreampass' then
        local full=dir..'/'..entry
        local attributes=lfs.attributes(full)
        if attributes.mode=='directory' then walk(tool,base,full)
        else
          local rel=full:sub(#base+2);rel=rel:match('^(.+)%.dss$') or rel:match('^(.+)%.dsi$') or rel:match('^(.+)%.dfs$')
          if rel then
            local asset=lfs.attributes(project..'/Content/'..rel..'.uasset')
            if not asset or asset.modification<attributes.modification then stale[tool]=stale[tool]+1 end
          end
        end
      end
    end
  end
  for _,kind in ipairs{{'shader','DShader'},{'fx','DFX'}} do
    local base=project..'/'..kind[2]
    if lfs.attributes(base,'mode')=='directory' then walk(kind[1],base,base) end
  end
  return stale
end

-- Assets version control ignores that nothing rebuilds: a clone would not have them. FILES is what is ignored, relative to PROJECT;
-- OWNED has what the sources and the image list build.
function M.orphans(files,owned)
  local found={}
  for _,file in ipairs(files) do
    if not owned[file] then
      found[#found+1]=file..' (ignored by version control with no source: list it in fx_family.lua IMAGES or MESHES, add its source under DShader/ or DFX/, or delete it)'
    end
  end
  table.sort(found)
  return found
end

-- What PROJECT's .gitignore block ignores, on disk.
function M.ignored_assets(project)
  local lfs=require('lfs')
  local untrack=require('dream_untrack')
  local files={}
  for _,dir in ipairs(untrack.WHOLE) do
    if lfs.attributes(project..'/'..dir,'mode')=='directory' then
      for entry in lfs.dir(project..'/'..dir) do
        if entry:match('%.uasset$') then files[#files+1]=dir..'/'..entry end
      end
    end
  end
  local ignore=io.open(project..'/.gitignore','rb')
  if ignore then
    local text=ignore:read('a');ignore:close()
    local first=text:find(untrack.BEGIN,1,true)
    local last=first and text:find(untrack.END,first,true)
    for line in (first and last and text:sub(first,last-1) or ''):gmatch('[^\r\n]+') do
      if line:match('%.uasset$') and not line:find('*',1,true) and lfs.attributes(project..'/'..line) then files[#files+1]=line end
    end
  end
  return files
end

-- Everything the sources under PROJECT and the image list build, relative to PROJECT.
function M.owned_assets(project)
  local owned={}
  for _,file in ipairs(require('dream_clone_verify').text_owned(require('lfs'),project)) do owned[file:sub(#project+2)]=true end
  for _,row in ipairs(require('fx_family').image_assets()) do owned[row.asset]=true end
  return owned
end

-- The text-owned assets of PROJECT that are missing or lost their mark, and how many there are.
function M.unowned(project)
  local lfs=require('lfs')
  local bad,count={},0
  local function walk(base,mark,dir)
    for entry in lfs.dir(dir) do
      if entry~='.' and entry~='..' and entry~='.dreampass' then
        local full=dir..'/'..entry
        if lfs.attributes(full,'mode')=='directory' then walk(base,mark,full)
        else
          local rel=full:sub(#base+2);rel=rel:match('^(.+)%.dss$') or rel:match('^(.+)%.dsi$') or rel:match('^(.+)%.dfs$')
          if rel then
            count=count+1
            local f=io.open(project..'/Content/'..rel..'.uasset','rb')
            local data=f and f:read('a')
            if f then f:close() end
            if not data then bad[#bad+1]=rel..' (missing)'
            elseif not data:find(mark,1,true) then bad[#bad+1]=rel..' (not built from its source)' end
          end
        end
      end
    end
  end
  for _,kind in ipairs{{'DShader','DreamShader.SourceFile'},{'DFX','DreamFX.SourcePath'}} do
    local base=project..'/'..kind[1]
    if lfs.attributes(base,'mode')=='directory' then walk(base,kind[2],base) end
  end
  -- A recorded overwrite counts until the file changes again, which restoring it does.
  local record=io.open(project..'/Saved/DreamGuard/overwritten.txt','rb')
  if record then
    local reported={}
    for line in record:lines() do
      local file,size,time=line:match('^([^\t]+)\t(%d+)\t(%d+)')
      local now=file and lfs.attributes(project..'/'..file)
      if now and not reported[file] and now.size==tonumber(size) and math.abs(now.modification-tonumber(time))<=2 then
        reported[file]=true
        bad[#bad+1]=file..' (overwritten by another tool; delete it and run dream_assets.lua ensure)'
      end
    end
    record:close()
  end
  table.sort(bad)
  return bad,count
end

function M.decompile_all(options,project,run,p)
  local assets=M.material_assets(p.join(project,'Content'):gsub('\\','/'))
  local done,failed=0,{}
  for index,asset in ipairs(assets) do
    local mine=not options.shards or (index-1)%options.shards==options.shard-1
    if mine and (not options.filter or asset:match(options.filter)) then
      local args={table.unpack(options.passed)}
      table.insert(args,1,'-Asset='..asset)
      local code=run(args,asset:match('([^.]+)$'))
      done=done+1
      print((code==0 and 'ok     ' or 'FAILED ')..asset);io.stdout:flush()
      if code~=0 then failed[#failed+1]=asset end
    end
  end
  print(('[dream-assets] %d decompiled, %d failed'):format(done-#failed,#failed))
  return #failed==0 and 0 or 1
end

function M.main(argv)
  local b=require('_bootstrap')
  local p,r=b.lib('path'),b.lib('process')
  if argv[1]=='pinned' then
    local filter=argv[2]=='--filter' and assert(argv[3],'--filter needs a pattern') or nil
    return M.report_pinned((b.root:gsub('\\','/')),filter)
  end
  if argv[1]=='ensure' then
    local passed,given,ue_root={},nil,nil
    local i=2
    while i<=#argv do
      if argv[i]=='--project' then i=i+1;given=assert(argv[i],'--project needs a directory')
      elseif argv[i]=='--ue-root' then i=i+1;ue_root=assert(argv[i],'--ue-root needs a path');passed[#passed+1]='--ue-root';passed[#passed+1]=ue_root
      else error('unknown ensure argument: '..argv[i]) end
      i=i+1
    end
    if given then table.insert(passed,1,given);table.insert(passed,1,'--project') end
    local project=given and p.absolute(given,b.root) or p.join(b.root,'omfue')
    -- First: the instances name these images.
    require('fx_family').ensure_images((project:gsub('\\','/')),M.FLAGS,ue_root)
    local stale=M.stale((project:gsub('\\','/')))
    print(('[dream-assets] %d material and %d Niagara assets to rebuild'):format(stale.shader,stale.fx))
    local function build(tool,verb,...)
      local args={table.unpack(passed)}
      args[#args+1]=tool;args[#args+1]=verb
      for _,v in ipairs{...} do args[#args+1]=v end
      assert(M.main(args)==0,tool..' '..verb..' failed')
    end
    if stale.shader>0 then build('shader','compile','-All') end
    if stale.fx>0 then build('fx','build','-All') end
    return M.main(given and {'owned','--project',given} or {'owned'})
  end
  if argv[1]=='owned' then
    local project=argv[2]=='--project' and p.absolute(assert(argv[3],'--project needs a directory'),b.root) or p.join(b.root,'omfue')
    assert(p.is_file(p.join(project,'om.uproject')),'no project: '..project)
    local bad,count=M.unowned((project:gsub('\\','/')))
    local forward=project:gsub('\\','/')
    for _,line in ipairs(M.orphans(M.ignored_assets(forward),M.owned_assets(forward))) do bad[#bad+1]=line end
    for _,line in ipairs(bad) do print(line) end
    print(('[dream-assets] %d text-owned, %d not built from text'):format(count,#bad))
    return #bad==0 and 0 or 1
  end
  local options=M.parse(argv)
  local project=options.project and p.absolute(options.project) or p.join(b.root,'omfue')
  local uproject=p.join(project,'om.uproject')
  assert(p.is_file(uproject),'no project: '..uproject)
  if options.project and M.SAVES[options.tool][options.verb] then require('dream_shadow').require_own(project) end
  local editor=p.join(b.lib('platform').ue_root(options.ue_root),'Engine/Binaries/Win64/UnrealEditor-Cmd.exe')
  local log=p.join(project,'Saved/DreamAssets',options.tool..'-'..options.verb..'-'..os.date('%Y%m%d-%H%M%S')..'.log')
  p.mkdir_p(p.parent(log))
  -- A shadow's assets are its own: only live needs the lock.
  local lock <close> = not options.project and M.SAVES[options.tool][options.verb] and require('build_lock').acquire(b.root,'dream_assets '..options.tool..' '..options.verb,{log=print}) or nil
  local function run(passed,suffix)
    local run_options=setmetatable({passed=passed},{__index=options})
    local run_log=suffix and log:gsub('%.log$','-'..suffix..'.log') or log
    local result=r.run(editor,M.command_args(run_options,uproject,run_log),{cwd=project,check=false,label='dream_assets '..options.tool..' '..options.verb})
    return result.exit_code or 1
  end
  if options.verb=='decompile-all' and options.tool=='shader' then return M.decompile_all(options,project,run,p) end
  local code=run(options.passed)
  print('[dream-assets] log '..log)
  return code
end

if arg and arg[0] and arg[0]:match('dream_assets%.lua$') then
  local ok,code=xpcall(M.main,debug.traceback,arg)
  if not ok then io.stderr:write(tostring(code)..'\n');os.exit(1) end
  os.exit(code)
end
return M
