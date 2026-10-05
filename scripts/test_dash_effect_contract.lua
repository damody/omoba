local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,platform=b.lib('path'),b.lib('process'),b.lib('platform')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','dash-effect-tests'))
local root
for i=1,1000 do
  local candidate=path.join(parent,os.time()..'-'..i)
  if lfs.mkdir(candidate) then root=candidate;break end
end
assert(root)
local count=0
local function check(name,target,levels,effects,expected)
  local dir=path.join(root,name)
  path.write(path.join(dir,'templates','heroes.lua'),
    'return function() return {{id="dash_hero",abilities={"dash_skill"}}} end')
  path.write(path.join(dir,'templates','abilities.lua'),
    'return function() return {{id="dash_skill",max_level=2,ability_type="active",cast_type="instant",'..
    'target_type="'..target..'",levels='..levels..',effects='..effects..'}} end')
  local result=process.run(platform.lua_executable,
    {path.join(b.root,'scripts','gen_hero_registry.lua'),dir,'registry'},{cwd=b.root,check=false})
  count=count+1
  if expected then
    assert(result.exit_code~=0 and result.stderr:find(expected,1,true),name..': '..result.stderr)
  else
    assert(result.exit_code==0 and result.stdout:find('EffectOp::DashToPoint',1,true),name..': '..result.stderr)
  end
end
local dash='{{kind="dash_to_point"}}'
check('valid','point','{{range=450},{range=600}}',dash)
check('edges','point','{{range=1/1024},{range=10000}}',dash)
for _,range in ipairs({'0','0.0001','10001'}) do
  check('range-'..range,'point','{{range=450},{range='..range..'}}',dash,'range')
end
check('wrong-target','unit','{{range=450},{range=600}}',dash,'target_type')
check('mixed','point','{{range=450},{range=600}}',
  '{{kind="dash_to_point"},{kind="area_damage"}}','exclusive')
print('dash effect contract: '..count..'/'..count..' passed')
