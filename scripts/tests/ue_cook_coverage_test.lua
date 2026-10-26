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
print('ue cook coverage: '..#cooked..' cooked roots cover every runtime soft-path root')
