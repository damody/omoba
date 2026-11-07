-- M3' ② hero_hot_reload.lua: request file, generations and the authority's answer (no build,
-- no server: the answer is written by a fake sleep, as the authority would between ticks).
package.path='scripts/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local reload=require('hero_hot_reload')
local root=path.join(b.root,'target','hero-hot-reload-test-'..os.time()..'-'..math.random(0,999999))
local run=path.join(root,'run')
local function fixture()
  path.mkdir_p(path.join(run,'artifacts','scripts'))
  path.write(path.join(run,'artifacts','scripts','scripts-manifest.json'),'{}',true)
  local target=path.join(root,'cargo')
  path.mkdir_p(path.join(target,reload.PROFILE))
  path.write(path.join(target,reload.PROFILE,'hero_a02.dll'),'fake dll',true,true)
  return target
end
local target=fixture()
assert(reload.row('inahime').module=='hero_a02')
assert(not pcall(reload.row,'not_a_hero'))
local hot=path.join(run,'artifacts',reload.HOT_DIR)
assert(reload.next_generation(hot,'hero_a02')==1)
local options=reload.options({'--hero','inahime','--run',run,'--no-generate','--no-build','--target-dir',target,'--timeout','5'})
local logs={}
local function answer(ok)
  return function()
    for name in require('lfs').dir(hot) do
      local generation=name:match('^request%-(%d+)%.json$')
      if generation and not path.is_file(path.join(hot,'request-'..generation..'.result.json')) then
        local request=json.decode(path.read(path.join(hot,name)))
        assert(request.module=='hero_a02' and request.hero=='inahime' and #request.sha256==64)
        assert(path.is_file(request.file) and request.file:match('hero_a02%.'..generation..'%.dll$'))
        path.write(path.join(hot,'request-'..generation..'.result.json'),
          ok and json.encode({ok=true,tick=90}) or json.encode({ok=false,error='identity changed'}),true)
      end
    end
  end
end
local result=reload.run(options,{sleep_ms=answer(true),log=function(line) logs[#logs+1]=line end})
assert(result.ok and result.tick==90)
assert(reload.next_generation(hot,'hero_a02')==2,'each request gets a new DLL name')
local ok,message=pcall(reload.run,options,{sleep_ms=answer(false),log=function() end})
assert(not ok and tostring(message):find('identity changed',1,true),'a refusal is an error')
assert(reload.next_generation(hot,'hero_a02')==3)
print('hero_hot_reload: 3 groups passed (fake build output and authority answer)')
