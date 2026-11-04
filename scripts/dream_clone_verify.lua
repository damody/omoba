-- Rehearses a clone that holds only the text of the text-owned assets, with nothing from the working tree: what is committed,
-- minus those .uasset files. It finds what such a clone lacks (a submodule, an untracked input, a hard-coded path).
--
--   tools\lua\lua.exe scripts\dream_clone_verify.lua clone  --dir DIR   clone HEAD and its submodules from the local repositories, no network
--   tools\lua\lua.exe scripts\dream_clone_verify.lua strip  --dir DIR   delete the text-owned .uasset files from DIR
--   tools\lua\lua.exe scripts\dream_clone_verify.lua build  --dir DIR   DIR's own build_ue_moba
--   tools\lua\lua.exe scripts\dream_clone_verify.lua record --dir DIR --hero ID --output OUT [--port N]
--                                                                       DIR's own recording, holding this repository's live match lock
--
-- A lock lives under its repository's target/, so a clone would not see a match running here: record takes the real one.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

M.SUBMODULES={'omb','specs','log4rs','erps','omfue'}
M.ENGINE='D:/UE5.8'

-- The .uasset of every source under PROJECT's DShader/ and DFX/.
function M.text_owned(lfs,project)
  local files={}
  local function walk(base,dir)
    for entry in lfs.dir(dir) do
      if entry~='.' and entry~='..' and entry~='.dreampass' then
        local full=dir..'/'..entry
        if lfs.attributes(full,'mode')=='directory' then walk(base,full)
        else
          local rel=full:sub(#base+2);rel=rel:match('^(.+)%.dss$') or rel:match('^(.+)%.dsi$') or rel:match('^(.+)%.dfs$')
          if rel then files[#files+1]=project..'/Content/'..rel..'.uasset' end
        end
      end
    end
  end
  for _,name in ipairs{'DShader','DFX'} do
    local base=project..'/'..name
    if lfs.attributes(base,'mode')=='directory' then walk(base,base) end
  end
  table.sort(files)
  return files
end

function M.main(argv)
  local b=require('_bootstrap')
  local p,r=b.lib('path'),b.lib('process')
  local lfs=require('lfs')
  local verb=argv[1]
  local options={}
  local i=2
  while i<=#argv do
    assert(argv[i]:match('^%-%-'),'unknown argument: '..argv[i])
    options[argv[i]:sub(3)]=assert(argv[i+1],argv[i]..' needs a value')
    i=i+2
  end
  local dir=(p.absolute(assert(options.dir,'--dir is required'),b.root):gsub('\\','/'))
  local root=(b.root:gsub('\\','/'))
  assert(dir:lower():sub(1,#root+1)==root:lower()..'/' and dir:lower()~=(root..'/omfue'):lower(),'DIR must be under this repository, not live')
  local function git(args,cwd)
    local result=r.run('git',args,{cwd=cwd,check=false,label='git '..args[1]})
    assert(result.exit_code==0,'git '..table.concat(args,' ')..' failed: '..tostring(result.stderr))
    return result.stdout or ''
  end
  if verb=='clone' then
    assert(not lfs.attributes(dir),'DIR must not exist: '..dir)
    p.mkdir_p(p.parent(dir))
    git({'clone','--local','--no-checkout',root,dir},root)
    git({'checkout','--detach',(git({'rev-parse','HEAD'},root):gsub('%s+',''))},dir)
    for _,name in ipairs(M.SUBMODULES) do
      git({'config','submodule.'..name..'.url',root..'/.git/modules/'..name},dir)
      git({'-c','protocol.file.allow=always','submodule','update','--init','--',name},dir)
    end
    print('cloned '..dir)
    return 0
  elseif verb=='strip' then
    local files=M.text_owned(lfs,dir..'/omfue')
    local removed=0
    for _,file in ipairs(files) do
      if lfs.attributes(file) then assert(os.remove(file),file);removed=removed+1 end
    end
    print(('stripped %d of %d text-owned assets'):format(removed,#files))
    return 0
  elseif verb=='build' then
    local result=r.run(p.join(dir,'tools/lua/lua.exe'),{p.join(dir,'scripts/build_ue_moba.lua')},{cwd=dir,check=false,label='clone build'})
    print(result.stdout or '');print(result.stderr or '')
    print('build exit '..tostring(result.exit_code))
    return result.exit_code
  elseif verb=='record' then
    local machine_lock=require('machine_lock')
    local function record()
      local lock <close> =machine_lock.acquire(root,'live_match',{run=dir,label='dream-clone-record',wait_seconds=3600,grace_seconds=1800})
      local result=r.run(p.join(dir,'tools/lua/lua.exe'),{p.join(dir,'scripts/record_hero_spotlight.lua'),'--hero',assert(options.hero,'--hero is required'),
        '--output',p.absolute(assert(options.output,'--output is required'),b.root),'--port',options.port or '58471','--no-build'},{cwd=dir,check=false,label='clone record'})
      print(result.stdout or '');print(result.stderr or '')
      print('record exit '..tostring(result.exit_code))
      return result.exit_code
    end
    return record()
  end
  error('verb is clone, strip, build or record')
end

if arg and arg[0] and arg[0]:match('dream_clone_verify%.lua$') then
  local ok,code=xpcall(M.main,debug.traceback,arg)
  if not ok then io.stderr:write(tostring(code)..'\n');os.exit(1) end
  os.exit(code)
end
return M
