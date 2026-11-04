-- Makes the text under DIR/DShader and DIR/DFX build the assets at their own paths, in a shadow with its own Content.
-- Originals stay in place: each is marked as built from its source, then overwritten by the rebuild.
--
--   tools\lua\lua.exe scripts\dream_switch.lua --project DIR --sources omfue/DreamSources [--only LUA_PATTERN ...] [--except LUA_PATTERN ...] [--allow-pinned]
--
-- Copies the chosen sources into DIR, records every chosen asset's stored properties (Saved/DreamAssets/facts-before.txt),
-- checks that the text keeps each usage flag the asset has, marks the assets, rebuilds, records them again
-- (facts-after.txt) and compares. Exit 0 only with no difference that changes how an asset looks.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

M.IGNORED={ParameterStateId=true,ExpressionCollection=true,EditorOnlyData=true}
-- Recomputed from the graph whenever the material is next built.
M.DERIVED={bCanMaskedBeAssumedOpaque=true}
M.EDITOR='D:/UE5.8/Engine/Binaries/Win64/UnrealEditor-Cmd.exe'

local function read(path)
  local f=io.open(path,'rb')
  if not f then return nil end
  local text=f:read('a');f:close()
  return text
end

local function write(path,text)
  local f=assert(io.open(path,'wb'));f:write(text);f:close()
end

local function walk(lfs,dir,visit)
  if not lfs.attributes(dir) then return end
  for entry in lfs.dir(dir) do
    if entry~='.' and entry~='..' then
      local full=dir..'/'..entry
      if lfs.attributes(full,'mode')=='directory' then walk(lfs,full,visit) else visit(full) end
    end
  end
end

function M.chosen(rel,only,except)
  local keep=#only==0
  for _,pattern in ipairs(only) do if rel:match(pattern) then keep=true end end
  for _,pattern in ipairs(except) do if rel:match(pattern) then keep=false end end
  return keep
end

-- rel without extension -> /Game object path.
function M.object_path(rel)
  return '/Game/'..rel..'.'..rel:match('([^/]+)$')
end

function M.read_facts(path)
  local facts={}
  for line in io.lines(path) do
    local asset,key,value=line:match('^(.-)\t(.-)\t(.*)$')
    if asset then facts[asset]=facts[asset] or {};facts[asset][key]=value end
  end
  return facts
end

-- Usage flags the decompiler leaves out of the text: a source without one rebuilds a material that can't draw on that mesh type.
function M.missing_flags(facts,material_sources)
  local missing={}
  for asset,text in pairs(material_sources) do
    for key,value in pairs(facts[asset] or {}) do
      if key:match('^bUsedWith') and value=='True' and not text:find(key,1,true) then missing[#missing+1]=asset..' '..key end
    end
  end
  table.sort(missing)
  return missing
end

function M.add_flags(text,flags)
  local head,tail=text:match('^(.-#pragma material%()(.*)$')
  assert(head,'no #pragma material')
  return head..table.concat(flags,' = true, ')..' = true, '..tail
end

-- The decompiler writes a texture sampled at the default UV as a node, but a .dsi can only override a uniform of a parent
-- that has a source (DSH7258). The same sample, as a uniform.
function M.uniform_textures(text)
  local declared={}
  text=text:gsub('UE%.TextureSampleParameter2D%(MipValue = %-1, ParameterName = "([%w_]+)"(.-)%)',function(name,rest)
    declared[#declared+1]={name,rest:match('Texture = "([^"]+)"')}
    return name..'.Sample(UE.TextureCoordinate())'
  end)
  if #declared==0 then return text end
  local lines={}
  for _,texture in ipairs(declared) do
    if texture[2] then lines[#lines+1]='/// @default '..texture[2] end
    lines[#lines+1]='uniform Texture2D '..texture[1]..';'
  end
  local head,tail=text:match('^(.-)(export void .*)$')
  assert(head,'no export')
  return head..table.concat(lines,'\n')..'\n\n'..tail
end

local function scalars(value)
  local out={}
  for name,number,rest in (value or ''):gmatch('Name="([^"]+)"%),ParameterValue=([-%d%.e]+)([^)]*)') do
    -- No ExpressionGUID: the parent has no such parameter, so the override does nothing.
    if rest:find('ExpressionGUID',1,true) then out[name]=tonumber(number) end
  end
  return out
end

local function texture_set(value)
  local names={}
  for path in (value or ''):gmatch("'([^']+)'") do names[#names+1]=path end
  table.sort(names)
  return table.concat(names,',')
end

-- Each rebuild gives a parameter a new id.
local function without_ids(value)
  return value and (value:gsub(',ExpressionGUID=%x+','')) or nil
end

-- parent(asset) gives the scalar defaults of an instance's parent.
function M.compare(before,after,assets,parent_defaults)
  local real,noted={},{defaults=0,order=0,derived=0,same=0}
  for _,asset in ipairs(assets) do
    local a,b=before[asset] or {},after[asset] or {}
    local defaults=parent_defaults(asset)
    local ok=true
    local keys={}
    for key in pairs(a) do keys[key]=true end
    for key in pairs(b) do keys[key]=true end
    for key in pairs(keys) do
      if without_ids(a[key])~=without_ids(b[key]) and not M.IGNORED[key] then
        if M.DERIVED[key] then noted.derived=noted.derived+1
        elseif key=='ScalarParameterValues' then
          local sa,sb=scalars(a[key]),scalars(b[key])
          for name,value in pairs(sa) do
            local other=sb[name] or defaults[name]
            if not other or math.abs(other-value)>1e-5 then ok=false;real[#real+1]=asset..' '..name..' '..value..' -> '..tostring(other) end
            if not sb[name] then noted.defaults=noted.defaults+1 end
          end
          for name,value in pairs(sb) do
            if not sa[name] and (not defaults[name] or math.abs(defaults[name]-value)>1e-5) then ok=false;real[#real+1]=asset..' added '..name..' '..value end
          end
        elseif key=='ReferencedDefaultTextures' then
          if texture_set(a[key])==texture_set(b[key]) then noted.order=noted.order+1
          else ok=false;real[#real+1]=asset..' textures' end
        else ok=false;real[#real+1]=asset..' '..key..': '..tostring(a[key]):sub(1,100)..' -> '..tostring(b[key]):sub(1,100) end
      end
    end
    if ok then noted.same=noted.same+1 end
  end
  table.sort(real)
  return real,noted
end

function M.main(argv)
  local b=require('_bootstrap')
  local p,r=b.lib('path'),b.lib('process')
  local lfs=require('lfs')
  local options={only={},except={}}
  local i=1
  while i<=#argv do
    local v=argv[i]
    if v=='--project' then i=i+1;options.project=argv[i]
    elseif v=='--sources' then i=i+1;options.sources=argv[i]
    elseif v=='--only' then i=i+1;options.only[#options.only+1]=argv[i]
    elseif v=='--except' then i=i+1;options.except[#options.except+1]=argv[i]
    elseif v=='--fix-flags' then options.fix_flags=true
    elseif v=='--compare-only' then options.compare_only=true
    elseif v=='--allow-pinned' then options.allow_pinned=true
    else error('unknown argument: '..v) end
    i=i+1
  end
  assert(options.project and options.sources,'--project and --sources are required')
  local project=p.absolute(options.project):gsub('\\','/')
  require('dream_shadow').require_own(project)
  local sources=p.absolute(options.sources):gsub('\\','/')
  local saved=project..'/Saved/DreamAssets/'
  p.mkdir_p(saved)
  local function commandlet(args)
    local list={project..'/om.uproject'}
    for _,v in ipairs(args) do list[#list+1]=v end
    for _,v in ipairs{'-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-nullrhi'} do list[#list+1]=v end
    return r.run(M.EDITOR,list,{cwd=project,check=false,label=args[1]}).exit_code or 1
  end
  -- An asset pinned by SHA keeps its owner's bytes: rebuilding it would fail the owner's checks.
  local root=b.root:gsub('\\','/')
  local dream_assets=require('dream_assets')
  local candidates,known={},dream_assets.text_candidates(root..'/omfue/Content')
  for _,kind in ipairs{{'Materials','dss'},{'Instances','dsi'}} do
    local base=sources..'/DShader/'..kind[1]..'/'
    walk(lfs,base,function(path)
      local rel=path:sub(#base+1):gsub('^/+',''):match('^(.*)%.'..kind[2]..'$')
      local package=rel and '/Game/'..rel
      if package and M.chosen(rel,options.only,options.except) and known[package] then candidates[package]=known[package] end
    end)
  end
  local pinned,names=dream_assets.pinned(root,candidates),{}
  for package in pairs(pinned) do names[#names+1]=package end
  table.sort(names)
  -- --allow-pinned: the owner of these has been changed to read their source instead of their bytes.
  assert(#names==0 or options.allow_pinned,'pinned by SHA, exclude with --except (dream_assets.lua pinned):\n'..table.concat(names,'\n'))

  -- Materials and instances first, by source file.
  local assets,adopt,material_sources,parents={},{},{},{}
  for _,kind in ipairs{{'Materials','dss'},{'Instances','dsi'}} do
    local base=sources..'/DShader/'..kind[1]..'/'
    walk(lfs,base,function(path)
      local rel=path:sub(#base+1):gsub('^/+',''):match('^(.*)%.'..kind[2]..'$')
      if not rel or not M.chosen(rel,options.only,options.except) then return end
      local destination=project..'/DShader/'..rel..'.'..kind[2]
      p.mkdir_p(p.parent(destination))
      local text=read(path)
      if kind[2]=='dss' then text=M.uniform_textures(text) end
      write(destination,text)
      local asset=M.object_path(rel)
      assets[#assets+1]=asset
      adopt[#adopt+1]=asset..'\tDShader/'..rel..'.'..kind[2]
      if kind[2]=='dss' then material_sources[asset]=text
      else parents[asset]=text:match('Parent = "([^"]+)"') end
    end)
  end
  table.sort(assets)
  local list=saved..'switch-assets.txt'
  write(list,table.concat(assets,'\n')..'\n')
  if not options.compare_only then
  assert(commandlet{'-run=OmMaterialFacts','-List='..list,'-Out='..saved..'facts-before.txt'}==0,'facts before failed')
  end
  local before=M.read_facts(saved..'facts-before.txt')
  if not options.compare_only then
  local missing=M.missing_flags(before,material_sources)
  if #missing>0 and options.fix_flags then
    local by={}
    for _,line in ipairs(missing) do local a,f=line:match('^(%S+) (%S+)$');by[a]=by[a] or {};by[a][#by[a]+1]=f end
    for asset,flags in pairs(by) do
      local rel=asset:match('^/Game/(.-)%.')
      local destination=project..'/DShader/'..rel..'.dss'
      write(destination,M.add_flags(read(destination),flags))
      print('[dream-switch] added '..table.concat(flags,',')..' to '..rel)
    end
  elseif #missing>0 then
    for _,line in ipairs(missing) do print('missing flag '..line) end
    error('the text drops usage flags the assets have: pass --fix-flags to add them')
  end
  local adopt_list=saved..'switch-adopt.txt'
  write(adopt_list,table.concat(adopt,'\n')..'\n')
  assert(commandlet{'-run=OmDreamAdopt','-List='..adopt_list}==0,'marking the assets failed')
  local compile_log=saved..'switch-compile.log'
  local code=commandlet{'-run=DreamShader','compile','-All','-Force','-abslog='..compile_log}
  assert(code==0,'compile failed, see '..compile_log)
  assert(commandlet{'-run=OmMaterialFacts','-List='..list,'-Out='..saved..'facts-after.txt'}==0,'facts after failed')
  end
  local after=M.read_facts(saved..'facts-after.txt')
  local cache={}
  local real,noted=M.compare(before,after,assets,function(asset)
    local parent=parents[asset]
    if not parent then return {} end
    if not cache[parent] then
      local values={}
      local text=read(project..'/DShader/'..parent:gsub('^/Game/','')..'.dss') or ''
      for name,value in text:gmatch('uniform float ([%w_]+) = ([-%d%.e]+)') do values[name]=tonumber(value) end
      cache[parent]=values
    end
    return cache[parent]
  end)
  print(('[dream-switch] %d of %d the same; overrides equal to the parent default dropped %d; texture order only %d; derived flags %d; real differences %d'):format(
    noted.same,#assets,noted.defaults,noted.order,noted.derived,#real))
  for _,line in ipairs(real) do print(line) end
  return #real==0 and 0 or 1
end

if arg and arg[0] and arg[0]:match('dream_switch%.lua$') then
  local ok,code=xpcall(M.main,debug.traceback,arg)
  if not ok then io.stderr:write(tostring(code)..'\n');os.exit(1) end
  os.exit(code)
end
return M
