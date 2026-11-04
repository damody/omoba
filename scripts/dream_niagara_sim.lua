-- Proves a Niagara system rebuilt from text behaves like the original: both run with fixed seeds and steps (OmNiagaraSim), and
-- particle counts, every attribute's range per frame and the renderers' settings are compared. Ids that differ by nature are ignored.
--
--   tools\lua\lua.exe scripts\dream_niagara_sim.lua --first DIR --second DIR --list FILE [--out DIR] [--show N]
--
-- DIR is a project holding the systems (a shadow, or a project whose Content is an older snapshot); FILE lists object paths,
-- one per line. Run --first twice to check that the comparison is deterministic. Exit 0 only with no difference.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local M={}

M.EDITOR='D:/UE5.8/Engine/Binaries/Win64/UnrealEditor-Cmd.exe'
M.IGNORED={MergeId=true,OuterEmitterVersion=true}
M.TOLERANCE=1e-3

-- REPORT text -> rows by key, in order.
function M.parse(text)
  local rows,order={},{}
  for line in text:gmatch('[^\r\n]+') do
    local f={}
    for part in (line..'\t'):gmatch('(.-)\t') do f[#f+1]=part end
    local key,value
    if f[1]=='R' then
      if not M.IGNORED[f[5]] then key=table.concat({'R',f[2],f[3],f[4],f[5]},'|');value=f[6] or '' end
    elseif f[1]=='C' then key=table.concat({'C',f[2],f[3],f[4]},'|');value=tonumber(f[5])
    elseif f[1]=='V' then key=table.concat({'V',f[2],f[3],f[4],f[5]},'|');value={tonumber(f[6]),tonumber(f[7]),tonumber(f[8])} end
    if key then rows[key]=value;order[#order+1]=key end
  end
  return rows,order
end

local function close(a,b)
  return a==b or (a~=a and b~=b) or math.abs(a-b)<=M.TOLERANCE*math.max(1,math.abs(a),math.abs(b))
end

-- Differences between two reports, plus how many rows of each kind were compared.
function M.compare(first,second)
  local a,order_a=M.parse(first)
  local b,order_b=M.parse(second)
  local differences,counts={},{R=0,C=0,V=0}
  for _,key in ipairs(order_a) do
    local kind=key:sub(1,1)
    counts[kind]=counts[kind]+1
    local x,y=a[key],b[key]
    if y==nil then differences[#differences+1]=key..' only in first'
    elseif kind=='V' then
      if not (close(x[1],y[1]) and close(x[2],y[2]) and close(x[3],y[3])) then
        differences[#differences+1]=('%s %g,%g,%g -> %g,%g,%g'):format(key,x[1],x[2],x[3],y[1],y[2],y[3])
      end
    elseif x~=y then differences[#differences+1]=key..' '..tostring(x):sub(1,120)..' -> '..tostring(y):sub(1,120) end
  end
  for _,key in ipairs(order_b) do
    if a[key]==nil then differences[#differences+1]=key..' only in second' end
  end
  return differences,counts
end

local function read(path)
  local f=assert(io.open(path,'rb'));local text=f:read('a');f:close()
  return text
end

function M.main(argv)
  local b=require('_bootstrap')
  local p,r=b.lib('path'),b.lib('process')
  local options={show=20}
  local i=1
  while i<=#argv do
    local v=argv[i]
    if v=='--first' then i=i+1;options.first=argv[i]
    elseif v=='--second' then i=i+1;options.second=argv[i]
    elseif v=='--list' then i=i+1;options.list=argv[i]
    elseif v=='--out' then i=i+1;options.out=argv[i]
    elseif v=='--show' then i=i+1;options.show=tonumber(argv[i])
    else error('unknown argument: '..v) end
    i=i+1
  end
  assert(options.first and options.second and options.list,'--first, --second and --list are required')
  local out=p.absolute(options.out or p.join(b.root,'target/dream-niagara-sim'))
  p.mkdir_p(out)
  local function run(project,label)
    local report=p.join(out,label..'.txt')
    local result=r.run(M.EDITOR,{p.join(p.absolute(project),'om.uproject'),'-run=OmNiagaraSim','-List='..p.absolute(options.list),'-Out='..report,
      '-AllowCommandletRendering','-d3d12','-unattended','-nop4','-nosplash','-nopause','-NoSound','-NoLiveCoding','-abslog='..p.join(out,label..'.log')},
      {cwd=p.absolute(project),check=false,label='OmNiagaraSim '..label})
    assert(result.exit_code==0,'simulation failed in '..project..'; see '..p.join(out,label..'.log'))
    return read(report)
  end
  local differences,counts=M.compare(run(options.first,'first'),run(options.second,'second'))
  print(('[dream-niagara-sim] compared %d renderer settings, %d particle counts, %d values: %d different'):format(counts.R,counts.C,counts.V,#differences))
  for index=1,math.min(options.show,#differences) do print('  '..differences[index]) end
  return #differences==0 and 0 or 1
end

if arg and arg[0] and arg[0]:match('dream_niagara_sim%.lua$') then
  local ok,code=xpcall(M.main,debug.traceback,arg)
  if not ok then io.stderr:write(tostring(code)..'\n');os.exit(1) end
  os.exit(code)
end
return M
