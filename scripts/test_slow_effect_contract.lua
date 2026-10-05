local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,platform=b.lib('path'),b.lib('process'),b.lib('platform')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','slow-effect-tests'))
local root
for i=1,1000 do
  local candidate=path.join(parent,os.time()..'-'..i)
  if lfs.mkdir(candidate) then root=candidate;break end
end
assert(root)
local count=0
local function check(name,target,levels,extras,effect,expected)
  local dir=path.join(root,name)
  path.write(path.join(dir,'templates','heroes.lua'),
    'return function() return {{id="slow_hero",abilities={"slow_skill"}}} end')
  path.write(path.join(dir,'templates','abilities.lua'),
    'return function() return {{id="slow_skill",max_level=2,ability_type="active",cast_type="instant",'..
    'target_type="'..target..'",levels='..levels..',extras='..extras..',effects={'..effect..'}}} end')
  local result=process.run(platform.lua_executable,
    {path.join(b.root,'scripts','gen_hero_registry.lua'),dir,'registry'},{cwd=b.root,check=false})
  count=count+1
  if expected then
    assert(result.exit_code~=0 and result.stderr:find(expected,1,true),name..': '..result.stderr)
  else
    assert(result.exit_code==0 and result.stdout:find('EffectOp::SlowEnemy',1,true),name..': '..result.stderr)
  end
end
local slow='{kind="slow_enemy",reduction_key="reduction",duration_key="duration"}'
local levels='{{range=450},{range=600}}'
check('valid','unit',levels,'{reduction={0.25,0.5},duration={2,3}}',slow)
check('edges','unit','{{range=1/1024},{range=10000}}',
  '{reduction={1/1024,1},duration={1/1024,60}}',slow)
for _,value in ipairs({'0','0.0001','1.01','math.huge'}) do
  check('reduction-'..value,'unit',levels,
    '{reduction={0.25,'..value..'},duration={2,3}}',slow,'reduction')
end
for _,value in ipairs({'0','0.0001','60.01','math.huge'}) do
  check('duration-'..value,'unit',levels,
    '{reduction={0.25,0.5},duration={2,'..value..'}}',slow,'duration')
end
check('missing-duration','unit',levels,'{reduction={0.25,0.5}}',slow,'duration')
check('wrong-target','point',levels,'{reduction={0.25,0.5},duration={2,3}}',slow,'target_type')
check('zero-range','unit','{{range=450},{range=0}}',
  '{reduction={0.25,0.5},duration={2,3}}',slow,'range')
check('same-key','unit',levels,'{shared={0.25,1.01}}',
  '{kind="slow_enemy",reduction_key="shared",duration_key="shared"}','shared')
print('slow effect contract: '..count..'/'..count..' passed')
