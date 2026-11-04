-- The shadow check of the tools that rewrite assets. A shadow may link its Content to live omfue (target/ue-shadow-c does), so
-- comparing the project's path with omfue says nothing: the directories are resolved through their links first, and any link
-- inside them is refused too, since it could lead anywhere.
local M={}

M.WRITTEN={'Content','DShader','DFX'}

local function normal(path)
  return (path:gsub('\\','/'):gsub('/+$',''):gsub('^(%a):',function(d) return d:lower()..':' end))
end

-- PATH with every link on the way replaced by its target.
function M.resolve(lfs,path)
  path=normal(path)
  local prefix,rest=path:match('^(%a:)/(.*)$')
  assert(prefix,'an absolute path is required: '..path)
  local current=prefix
  for part in rest:gmatch('[^/]+') do
    local next_path=current..'/'..part
    local attributes=lfs.symlinkattributes(next_path)
    if attributes and attributes.mode=='link' then
      local target=lfs.symlinkattributes(next_path,'target')
      assert(target and #target>0,'cannot read the link '..next_path)
      current=M.resolve(lfs,target)
    else current=next_path end
  end
  return current
end

local function inside(path,root)
  local a,b=path:lower(),root:lower()
  return a==b or a:sub(1,#b+1)==b..'/'
end

local function first_link(lfs,dir)
  for entry in lfs.dir(dir) do
    if entry~='.' and entry~='..' then
      local full=dir..'/'..entry
      local mode=lfs.symlinkattributes(full,'mode')
      if mode=='link' then return full end
      if mode=='directory' then
        local found=first_link(lfs,full)
        if found then return found end
      end
    end
  end
end

-- Why PROJECT may not be rewritten, or nil. LIVE is the live omfue directory.
function M.refusal(lfs,project,live)
  local resolved_live=M.resolve(lfs,live)
  if inside(M.resolve(lfs,project),resolved_live) then return project..' is live omfue' end
  for _,name in ipairs(M.WRITTEN) do
    local dir=normal(project)..'/'..name
    if lfs.symlinkattributes(dir) then
      local resolved=M.resolve(lfs,dir)
      if inside(resolved,resolved_live) then return dir..' leads to live '..resolved end
      local link=lfs.attributes(resolved,'mode')=='directory' and first_link(lfs,resolved)
      if link then return dir..' contains the link '..link end
    end
  end
end

-- Stops before anything is written unless PROJECT is a shadow with its own Content, DShader and DFX.
function M.require_own(project)
  local b=require('_bootstrap')
  local reason=M.refusal(require('lfs'),project,b.lib('path').join(b.root,'omfue'))
  assert(not reason,'a shadow with its own Content only: this rewrites assets. '..tostring(reason))
end

return M
