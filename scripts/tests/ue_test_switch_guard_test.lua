-- Test-automation switches (-om-*-smoke, -om-*-screenshot=) must go through
-- OmTestSwitch/OmTestSwitchValue (OmTestSwitches.h), which Shipping compiles to
-- false; a direct FParse on such a switch would let a release build be scripted
-- from its command line (errors.md E065). Editor-only test modules are exempt.
package.path='scripts/?.lua;'..package.path
local b=require('_bootstrap')
local path=b.lib('path')
local lfs=require('lfs')

local root=path.join(b.root,'omfue','Plugins','OmRuntime','Source')
local offenders,checked={},0
local function scan(dir)
  for name in lfs.dir(dir) do
    if name~='.' and name~='..' and name~='OmEditor' then
      local full=dir..'/'..name
      local mode=lfs.attributes(full,'mode')
      if mode=='directory' then scan(full)
      elseif mode=='file' and (name:match('%.cpp$') or name:match('%.h$')) then
        checked=checked+1
        local line=0
        for text in io.lines(full) do
          line=line+1
          for switch in text:gmatch('FParse::%a+%([^,]+,%s*TEXT%("(om%-[%w%-]+=?)"%)') do
            if switch:find('smoke',1,true) or switch:find('screenshot',1,true) then
              offenders[#offenders+1]=('%s:%d %s'):format(full:sub(#root+2),line,switch)
            end
          end
        end
      end
    end
  end
end
scan(root)
assert(checked>0,'no runtime sources found under '..root)
assert(#offenders==0,'test switches bypass OmTestSwitch (Shipping would honour them):\n'..table.concat(offenders,'\n'))
assert(path.read(path.join(root,'OmRuntime','Public','OmTestSwitches.h')):find('#if UE_BUILD_SHIPPING',1,true),'OmTestSwitches.h must disable switches in Shipping')
print(('ue test switch guard: %d runtime sources, no test switch reachable in Shipping'):format(checked))
