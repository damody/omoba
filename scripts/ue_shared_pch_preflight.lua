-- Optional read-only warming of existing MSVC shared PCH inputs. Never builds or repairs them.
local M = {}
local lfs = require('lfs')
function M.canonical(value, base)
  value=value:gsub('\\','/')
  if not value:match('^%a:/') and value:sub(1,1)~='/' then value=assert(base)..'/'..value end
  local prefix,rest=value:match('^(%a:/)(.*)$')
  if not prefix then
    local server,share,tail=value:match('^//([^/]+)/([^/]+)/(.*)$')
    if server then prefix,rest='//'..server..'/'..share..'/',tail
    elseif value:sub(1,1)=='/' then prefix,rest='/',value:sub(2)
    else return nil end
  end
  local parts={}
  for part in rest:gmatch('[^/]+') do
    if part=='..' then if #parts==0 then return nil end;table.remove(parts)
    elseif part~='.' then parts[#parts+1]=part end
  end
  return prefix..table.concat(parts,'/')
end
function M.candidates(responses, engine, attributes)
  local source=assert(M.canonical(engine..'/Engine/Source'))
  local allowed=assert(M.canonical(engine..'/Engine/Intermediate')):lower()..'/'
  local found,seen={},{}
  for _,response in ipairs(responses) do
    for line in response:gmatch('[^\r\n]+') do
      local value=line:match('^%s*/Fp"([^"]+)"%s*$') or line:match('^%s*/Fp([^%s"]+)%s*$')
      if value then
        local file=M.canonical(value,source)
        local key=file and file:lower()
        if key and key:sub(1,#allowed)==allowed and key:match('/sharedpch[^/]*%.pch$') and not seen[key] then
          seen[key]=true
          local stat=attributes(file)
          if stat and stat.mode=='file' then found[#found+1]={path=file,size=stat.size,modification=stat.modification} end
        end
      end
    end
  end
  table.sort(found,function(a,b)return a.path:lower()<b.path:lower() end)
  return found
end
function M.warm(candidates, options)
  local result={bytes=0,files=0,skipped=0,rows={}}
  local budget=options.budget or 8*1024^3
  for _,entry in ipairs(candidates) do
    if entry.size>budget-result.bytes then result.skipped=result.skipped+1
    else
      local before=assert(options.attributes(entry.path),'PCH disappeared before prefetch')
      assert(before.size==entry.size and before.modification==entry.modification,'PCH changed before prefetch')
      local started=options.clock()
      local handle=assert(options.open(entry.path,'rb'))
      local bytes=0
      while true do
        local chunk,err=handle:read(1024*1024)
        if not chunk then assert(not err,err);break end
        bytes=bytes+#chunk
        if bytes>entry.size then handle:close();error('PCH grew during prefetch') end
      end
      assert(handle:close())
      local after=assert(options.attributes(entry.path),'PCH disappeared during prefetch')
      assert(bytes==entry.size and after.size==before.size and after.modification==before.modification,'PCH changed during prefetch')
      local row={path=entry.path,bytes=bytes,elapsed_ms=options.clock()-started}
      result.rows[#result.rows+1]=row;result.files=result.files+1;result.bytes=result.bytes+bytes
      if options.report then options.report(row) end
    end
  end
  return result
end
function M.run(project,engine,clock,report)
  local candidates,seen,visited={}, {},0
  local function walk(dir,depth,plugins)
    if depth>16 then return end
    local stat=lfs.symlinkattributes(dir)
    if not stat or stat.mode~='directory' then return end
    if plugins then
      -- A descriptor defines a plugin boundary. Its SDK/Web/private trees are
      -- not build inventories; only its own Intermediate and nested Plugins are.
      for name in lfs.dir(dir) do
        if name:lower():match('%.uplugin$') and lfs.symlinkattributes(dir..'/'..name,'mode')=='file' then
          walk(dir..'/Intermediate/Build',0,false)
          walk(dir..'/Plugins',depth+1,true)
          return
        end
      end
    end
    for name in lfs.dir(dir) do
      if name~='.' and name~='..' then
        local file=dir..'/'..name
        local attr=lfs.symlinkattributes(file)
        if attr and attr.mode=='directory' then
          if plugins then
            if name~='Intermediate' and name~='Source' and name~='Content' and name~='Binaries' and name~='Resources'
              and name~='.git' and name~='node_modules' and name~='.cache' and name~='Saved' then walk(file,depth+1,true) end
          else walk(file,depth+1,false) end
        elseif not plugins and attr and attr.mode=='file' and name:lower():match('%.rsp$') then
          visited=visited+1;assert(visited<=4096,'shared PCH response inventory exceeds 4096 files')
          if attr.size<=1024*1024 then
            local handle=assert(io.open(file,'rb'));local response=assert(handle:read('a'));assert(handle:close())
            for _,entry in ipairs(M.candidates({response},engine,lfs.attributes)) do
              local key=entry.path:lower()
              if not seen[key] then seen[key]=true;candidates[#candidates+1]=entry end
            end
          end
        end
      end
    end
  end
  walk(project..'/Intermediate/Build',0,false);walk(project..'/Plugins',0,true)
  table.sort(candidates,function(a,b)return a.path:lower()<b.path:lower() end)
  return M.warm(candidates,{attributes=lfs.attributes,open=io.open,clock=clock,report=report})
end
return M
