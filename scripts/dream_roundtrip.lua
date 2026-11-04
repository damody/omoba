-- Proves omfue/DreamSources rebuilds the project's materials, in a shadow project with its own copy of Content:
--
--   tools\lua\lua.exe scripts\dream_roundtrip.lua prep --project DIR     instances retargeted at mirror assets, pair list
--   tools\lua\lua.exe scripts\dream_assets.lua --project DIR shader compile -All -Force
--   tools\lua\lua.exe scripts\dream_roundtrip.lua facts --project DIR    OmMaterialFacts over originals and mirrors
--   tools\lua\lua.exe scripts\dream_roundtrip.lua compare --project DIR  stored properties that really differ
--   tools\lua\lua.exe scripts\dream_assets.lua --project DIR --filter ^/Game/Decompiled/%a+/Game/ shader decompile-all
--   tools\lua\lua.exe scripts\dream_roundtrip.lua text --project DIR     decompiled text of each original and its mirror
--
-- DreamFX has this built in: fx build -All, fx mirror-diff, fx asset-diff.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

-- Identity of the asset or the rebuild, not its look.
M.IGNORED={ParameterStateId=true,ExpressionCollection=true,EditorOnlyData=true}
-- Recomputed from the graph whenever the material is next built.
M.DERIVED={bCanMaskedBeAssumedOpaque=true}

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
  for entry in lfs.dir(dir) do
    if entry~='.' and entry~='..' then
      local full=dir..'/'..entry
      if lfs.attributes(full,'mode')=='directory' then walk(lfs,full,visit) else visit(full) end
    end
  end
end

-- Parent material of a decompiled instance, read as its scalar defaults.
local function parent_scalars(decompiled,cache,parent)
  if cache[parent] then return cache[parent] end
  local values={}
  local text=read(decompiled..'Materials/Game/'..parent:gsub('^/Game/','')..'.dss') or ''
  for name,value in text:gmatch('uniform float ([%w_]+) = ([-%d%.e]+)') do values[name]=tonumber(value) end
  cache[parent]=values
  return values
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
  for path in (value or ''):gmatch("'([^']+)'") do names[#names+1]=path:gsub('/Game/Decompiled/%a+/Game/','/Game/') end
  table.sort(names)
  return table.concat(names,',')
end

function M.prep(lfs,decompiled,pairs_path)
  local rows={}
  walk(lfs,decompiled..'Instances/Game',function(path)
    local rel=path:sub(#(decompiled..'Instances/Game/')+1):match('^(.*)%.dsi$')
    if not rel then return end
    local mirror='/Game/Decompiled/Instances/Game/'..rel
    local text=read(path)
    text=text:gsub('/// @name [^\r\n]+','/// @name '..mirror,1)
    write(path,text)
    local name=rel:match('([^/]+)$')
    rows[#rows+1]='/Game/'..rel..'.'..name..'\t'..mirror..'.'..name
  end)
  walk(lfs,decompiled..'Materials/Game',function(path)
    local rel=path:sub(#(decompiled..'Materials/Game/')+1):match('^(.*)%.dss$')
    if not rel then return end
    local name=rel:match('([^/]+)$')
    rows[#rows+1]='/Game/'..rel..'.'..name..'\t/Game/Decompiled/Materials/Game/'..rel..'.'..name
  end)
  table.sort(rows)
  write(pairs_path,table.concat(rows,'\n')..'\n')
  return #rows
end

function M.compare(decompiled,pairs_path,facts_path)
  local facts={}
  for line in io.lines(facts_path) do
    local asset,key,value=line:match('^(.-)\t(.-)\t(.*)$')
    if asset then facts[asset]=facts[asset] or {};facts[asset][key]=value end
  end
  local cache={}
  local real,same,total,noted={},0,0,{defaults=0,order=0,derived=0}
  for line in io.lines(pairs_path) do
    local original,mirror=line:match('^(.-)\t(.*)$')
    total=total+1
    local a,b=facts[original] or {},facts[mirror] or {}
    local rel=original:match('^/Game/(.-)%.')
    local instance=read(decompiled..'Instances/Game/'..rel..'.dsi')
    local parent=instance and instance:match('Parent = "([^"]+)"')
    local defaults=parent and parent_scalars(decompiled,cache,parent) or {}
    local ok=true
    local keys={}
    for key in pairs(a) do keys[key]=true end
    for key in pairs(b) do keys[key]=true end
    for key in pairs(keys) do
      if a[key]~=b[key] and not M.IGNORED[key] then
        if M.DERIVED[key] then noted.derived=noted.derived+1
        elseif key=='ScalarParameterValues' then
          local sa,sb=scalars(a[key]),scalars(b[key])
          for name,value in pairs(sa) do
            local other=sb[name] or defaults[name]
            if not other or math.abs(other-value)>1e-5 then ok=false;real[#real+1]=original..' '..name..' '..value..' -> '..tostring(other) end
            if not sb[name] then noted.defaults=noted.defaults+1 end
          end
          for name,value in pairs(sb) do
            if not sa[name] and (not defaults[name] or math.abs(defaults[name]-value)>1e-5) then ok=false;real[#real+1]=original..' added '..name..' '..value end
          end
        elseif key=='ReferencedDefaultTextures' then
          if texture_set(a[key])==texture_set(b[key]) then noted.order=noted.order+1
          else ok=false;real[#real+1]=original..' textures' end
        else ok=false;real[#real+1]=original..' '..key..': '..tostring(a[key]):sub(1,100)..' -> '..tostring(b[key]):sub(1,100) end
      end
    end
    if ok then same=same+1 end
  end
  table.sort(real)
  return {same=same,total=total,real=real,noted=noted}
end

local function normal_text(text)
  local lines={}
  text=text:gsub('\r',''):gsub('/Game/Decompiled/%a+/Game/','/Game/')
  for line in text:gmatch('[^\n]+') do
    if not line:match('^// Decompiled by') and not line:match('^#pragma layout') and not line:match('^/// @name') and not line:match('^/// @sort') then lines[#lines+1]=line end
  end
  return table.concat(lines,'\n')
end

function M.text(lfs,decompiled)
  local same,different,missing=0,{},{}
  for _,kind in ipairs{{'Materials','dss'},{'Instances','dsi'}} do
    local base=decompiled..kind[1]..'/Game/'
    walk(lfs,base,function(path)
      local rel=path:sub(#base+1):match('^(.*)%.'..kind[2]..'$')
      if not rel or rel:match('^/*Decompiled/') then return end
      local mirror=read(base..'Decompiled/'..kind[1]..'/Game/'..rel..'.'..kind[2])
      if not mirror then missing[#missing+1]=rel
      elseif normal_text(read(path))==normal_text(mirror) then same=same+1
      else different[#different+1]=rel end
    end)
  end
  return {same=same,different=different,missing=missing}
end

function M.main(argv)
  local b=require('_bootstrap')
  local p,r=b.lib('path'),b.lib('process')
  local lfs=require('lfs')
  local verb=argv[1]
  local project
  for i=2,#argv do if argv[i]=='--project' then project=argv[i+1] end end
  assert(project,'--project DIR is required: this rewrites sources and needs a shadow with its own Content')
  project=p.absolute(project):gsub('\\','/')
  assert(project~=(b.root..'/omfue'):gsub('\\','/'),'not on live omfue')
  local decompiled=project..'/DShader/Decompiled/'
  local pairs_path=project..'/Saved/DreamAssets/material-pairs.txt'
  local facts_path=project..'/Saved/DreamAssets/material-facts.txt'
  if verb=='prep' then
    p.mkdir_p(p.parent(pairs_path))
    print(M.prep(lfs,decompiled,pairs_path)..' pairs')
  elseif verb=='facts' then
    local list={}
    for line in io.lines(pairs_path) do local a,c=line:match('^(.-)\t(.*)$');list[#list+1]=a;list[#list+1]=c end
    local list_path=project..'/Saved/DreamAssets/material-list.txt'
    write(list_path,table.concat(list,'\n')..'\n')
    local editor='D:/UE5.8/Engine/Binaries/Win64/UnrealEditor-Cmd.exe'
    local result=r.run(editor,{project..'/om.uproject','-run=OmMaterialFacts','-List='..list_path,'-Out='..facts_path,
      '-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-nullrhi'},{cwd=project,check=false,label='material facts'})
    return result.exit_code or 1
  elseif verb=='compare' then
    local report=M.compare(decompiled,pairs_path,facts_path)
    print(('%d of %d the same; overrides equal to the parent default dropped %d; texture order only %d; derived flags %d; real differences %d'):format(
      report.same,report.total,report.noted.defaults,report.noted.order,report.noted.derived,#report.real))
    for _,line in ipairs(report.real) do print(line) end
    return #report.real==0 and 0 or 1
  elseif verb=='text' then
    local report=M.text(lfs,decompiled)
    print(('text: %d the same, %d different, %d missing'):format(report.same,#report.different,#report.missing))
    for _,rel in ipairs(report.different) do print('different '..rel) end
    for _,rel in ipairs(report.missing) do print('missing '..rel) end
    return (#report.different+#report.missing)==0 and 0 or 1
  else error('first argument: prep, facts, compare or text') end
  return 0
end

if arg and arg[0] and arg[0]:match('dream_roundtrip%.lua$') then
  local ok,code=xpcall(M.main,debug.traceback,arg)
  if not ok then io.stderr:write(tostring(code)..'\n');os.exit(1) end
  os.exit(code)
end
return M
