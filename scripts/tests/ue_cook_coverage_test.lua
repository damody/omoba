-- Every content root that runtime C++ loads by soft path must be cooked: such assets
-- have no hard reference, so a package silently ships without them (grey ground,
-- invisible heroes). Scans the plugin source for /Game/<Root>/ soft paths and checks
-- DefaultGame.ini's DirectoriesToAlwaysCook covers each root.
package.path='scripts/?.lua;'..package.path
local b=require('_bootstrap')
local path=b.lib('path')
local lfs=require('lfs')

local ini=path.read(path.join(b.root,'omfue','Config','DefaultGame.ini'))
local cooked={}
for dir in ini:gmatch('%+DirectoriesToAlwaysCook=%(Path="([^"]+)"%)') do cooked[#cooked+1]=dir end

local function covered(asset)
  for _,dir in ipairs(cooked) do
    if asset==dir or asset:sub(1,#dir+1)==dir..'/' then return true end
  end
  return false
end

local roots,missing={}, {}
local function scan(dir)
  for name in lfs.dir(dir) do
    if name~='.' and name~='..' then
      local full=dir..'/'..name
      local mode=lfs.attributes(full,'mode')
      if mode=='directory' then scan(full)
      elseif mode=='file' and (name:match('%.cpp$') or name:match('%.h$')) and not name:match('Tests?%.cpp$') then
        for asset in path.read(full):gmatch('FSoftObjectPath%(TEXT%("(/Game/[%w_]+)/') do
          if not roots[asset] then
            roots[asset]=full
            if not covered(asset) then missing[#missing+1]=asset..' (first seen in '..full..')' end
          end
        end
      end
    end
  end
end
scan(path.join(b.root,'omfue','Plugins','OmRuntime','Source'))

assert(next(roots),'no soft-path content roots found; the scan pattern is stale')
assert(#missing==0,'soft-path content roots not in DirectoriesToAlwaysCook:\n'..table.concat(missing,'\n'))
-- The check itself rejects an uncovered root.
assert(not covered('/Game/NotCooked'))

-- An engine asset the runtime loads only by path (LoadObject / FSoftObjectPath) is not cooked: no
-- cooked package references it, and [Engine.StartupPackages] does not add it either (packaged games had
-- no health-text material, E138). It ships when a native constructor loads it (ConstructorHelpers): the
-- cooker keeps what class defaults reference. Every runtime engine path needs such a constructor
-- reference, ours or the engine's own (listed below with the class whose constructor loads it).
local ENGINE_CONSTRUCTOR_LOADS={
  ['/Engine/EngineMaterials/Widget3DPassThrough_Translucent']='UWidgetComponent',
}
local runtime_paths,constructor_paths,unreferenced={}, {}, {}
local function scan_engine(dir)
  for name in lfs.dir(dir) do
    if name~='.' and name~='..' then
      local full=dir..'/'..name
      local mode=lfs.attributes(full,'mode')
      if mode=='directory' then scan_engine(full)
      elseif mode=='file' and (name:match('%.cpp$') or name:match('%.h$')) and not name:match('Tests?%.cpp$') then
        local text=path.read(full)
        for package in text:gmatch('FObjectFinder<[%w_]+>%s*[%w_]+%(%s*TEXT%("(/Engine/[%w_/]+)') do constructor_paths[package]=true end
        for _,pattern in ipairs({'LoadObject<[%w_]+>%([^)]-TEXT%("(/Engine/[%w_/]+)','FSoftObjectPath%(TEXT%("(/Engine/[%w_/]+)'}) do
          for package in text:gmatch(pattern) do runtime_paths[package]=runtime_paths[package] or full end
        end
      end
    end
  end
end
scan_engine(path.join(b.root,'omfue','Plugins','OmRuntime','Source'))
assert(runtime_paths['/Engine/EngineMaterials/UnlitText'],'no runtime engine load paths found; the scan pattern is stale')
assert(constructor_paths['/Engine/BasicShapes/Cube'],'no constructor engine references found; the scan pattern is stale')
local runtime_count=0
for package,file in pairs(runtime_paths) do
  runtime_count=runtime_count+1
  if not constructor_paths[package] and not ENGINE_CONSTRUCTOR_LOADS[package] then
    unreferenced[#unreferenced+1]=package..' (loaded in '..file..')'
  end
end
table.sort(unreferenced)
assert(#unreferenced==0,'engine assets loaded only by path will not be cooked; add a ConstructorHelpers reference:\n'..table.concat(unreferenced,'\n'))
print('ue cook coverage: '..#cooked..' cooked roots cover every runtime soft-path root; '..runtime_count..' runtime engine paths have constructor references')
