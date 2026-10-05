-- Fixed-Lua generator contract; fixtures are isolated under ignored target/.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,platform=b.lib('path'),b.lib('process'),b.lib('platform')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','mana-effect-tests'))
local root
for i=1,1000 do
  local candidate=path.join(parent,os.time()..'-'..i)
  if lfs.mkdir(candidate) then root=candidate;break end
end
assert(root)
local count=0
local function check(name,target,extras,effects,expected)
  local dir=path.join(root,name)
  path.write(path.join(dir,'templates','heroes.lua'),
    'return function() return {{id="mana_hero",abilities={"mana_skill"}}} end')
  path.write(path.join(dir,'templates','abilities.lua'),
    'return function() return {{id="mana_skill",max_level=2,ability_type="active",cast_type="instant",'..
    'target_type="'..target..'",levels={{},{}},extras='..extras..',effects={'..effects..'}}} end')
  local result=process.run(platform.lua_executable,
    {path.join(b.root,'scripts','gen_hero_registry.lua'),dir,'registry'},{cwd=b.root,check=false})
  count=count+1
  if expected then
    assert(result.exit_code~=0 and result.stderr:find(expected,1,true),name..': '..result.stderr)
  else
    assert(result.exit_code==0,result.stderr)
    if effects:find('mana_buff_self',1,true) then
      assert(result.stdout:find('EffectOp::ManaBuffSelf',1,true))
      assert(result.stdout:find('omoba_content_model::ManaBuffStat::',1,true))
    else
      local restore=assert(result.stdout:find('EffectOp::RestoreManaSelf',1,true))
      local spend=assert(result.stdout:find('EffectOp::SpendManaSelf',1,true))
      assert(restore<spend,'resource declaration order must be preserved')
    end
  end
end
local effects='{kind="restore_mana_self",amount_key="resource"},{kind="spend_mana_self",amount_key="resource"}'
check('valid','none','{resource={20,30}}',effects)
check('zero-max','none','{resource={0,1000000}}',effects)
check('minimum','none','{resource={1/1024,1/1024}}',effects)
for i,value in ipairs({'-1','0.0001','1000001','math.huge','0/0'}) do
  check('amount-'..i,'none','{resource={20,'..value..'}}',effects,'effect amount_key')
end
check('missing','none','{}',effects,'needs one extras value')
check('rank-count','none','{resource={20}}',effects,'needs one extras value')
check('unit-target','unit','{resource={20,30}}',effects,'target_type')
check('point-target','point','{resource={20,30}}',effects,'target_type')
check('unknown','none','{resource={20,30}}','{kind="mana_unknown",amount_key="resource"}','target_type')
local function buff(stat) return '{kind="mana_buff_self",stat="'..stat..'",value_key="value",duration_key="duration"}' end
for _,stat in ipairs({'base_mana_regen','mana_regen_constant','mana_regen_constant_unique','mana_regen_percentage',
  'mana_regen_total_percentage','mana_bonus','extra_mana_bonus'}) do
  check('buff-'..stat,'none','{value={0,2},duration={1/1024,60}}',buff(stat))
end
for i,value in ipairs({'-1.1','16.1','0.0001','math.huge','0/0'}) do
  check('buff-value-'..i,'none','{value={0,'..value..'},duration={2,3}}',buff('mana_regen_percentage'),'invalid mana buff')
end
check('buff-signed','none','{value={-1000000,1000000},duration={2,3}}',buff('mana_bonus'))
for i,duration in ipairs({'0','-1','60.1','0.0001'}) do
  check('buff-duration-'..i,'none','{value={1,2},duration={2,'..duration..'}}',buff('mana_bonus'),'invalid mana buff')
end
check('buff-unknown','none','{value={1,2},duration={2,3}}',buff('damage'),'invalid mana buff stat')
check('buff-duplicate','none','{value={1,2},duration={2,3}}',buff('mana_bonus')..','..buff('mana_bonus'),'duplicate/invalid mana buff stat')
check('buff-target','unit','{value={1,2},duration={2,3}}',buff('mana_bonus'),'target_type')
check('buff-missing','none','{value={1,2}}',buff('mana_bonus'),'requires per-rank duration')
print('mana effect/buff contract: '..count..'/'..count..' passed')
