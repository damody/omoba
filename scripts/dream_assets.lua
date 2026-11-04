-- Text sources of the project's Niagara systems and materials: DreamFX (DFX/) and DreamShader (DShader/) commandlets.
--
--   tools\lua\lua.exe scripts\dream_assets.lua [--project DIR] [--ue-root PATH] {fx|shader} VERB [-Option[=value] ...]
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
  local options=M.parse(argv)
  local project=options.project and p.absolute(options.project) or p.join(b.root,'omfue')
  local uproject=p.join(project,'om.uproject')
  assert(p.is_file(uproject),'no project: '..uproject)
  local editor=p.join(options.ue_root or 'D:/UE5.8','Engine/Binaries/Win64/UnrealEditor-Cmd.exe')
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
