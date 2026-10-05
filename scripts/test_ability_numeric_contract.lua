local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,platform=b.lib('path'),b.lib('process'),b.lib('platform')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','ability-numeric-tests'))
local root
for i=1,1000 do
  local candidate=path.join(parent,os.time()..'-'..i)
  if lfs.mkdir(candidate) then root=candidate;break end
end
assert(root)
local count=0
local function run(name,second,amount,module,requirements)
  local dir=path.join(root,name)
  path.write(path.join(dir,'templates','heroes.lua'),
    'return function() return {{id="numeric_hero",abilities={"numeric_skill"}'..
    (module and ',rust_module="fixture"' or '')..'}} end')
  path.write(path.join(dir,'templates','abilities.lua'),
    'return function() return {{id="numeric_skill",max_level=2,ability_type="active",cast_type="instant",'..
    'target_type="unit",levels={{range=600},{range=600,'..(second or '')..'}},'..
    'extras={damage={0,'..(amount or '20')..'}},effects={{kind="damage",amount_key="damage",damage_kind="physical"}}}} end')
  local result=process.run(platform.lua_executable,
    {path.join(b.root,'scripts','gen_hero_registry.lua'),dir,'registry'},{cwd=b.root,check=false})
  count=count+1
  if requirements then
    assert(result.exit_code~=0 and result.stderr:find(requirements,1,true),name..': wrong rejection: '..result.stderr)
  else
    assert(result.exit_code==0,name..': '..result.stderr)
  end
end
run('defaults')
run('boundaries','cooldown=1/1024,mana_cost=1000000,cast_time=0', '1000000')
-- Lua double literals round to f32 in the shared schema. Both generators
-- must agree at the edge, not reject only in the FFI path.
run('f32-boundaries','cooldown=0.00097656249,mana_cost=1000000.01,range=10000.0001','1000000.01')
local index=0
for _,field in ipairs({'cooldown','mana_cost','cast_time','range'}) do
  for _,value in ipairs({'-1','0.0001','1000001','math.huge','-math.huge','0/0'}) do
    index=index+1
    run('invalid-'..index,field..'='..value,nil,false,'rank 2 '..field)
  end
end
for _,value in ipairs({'0.0001','1000001','math.huge','0/0'}) do
  index=index+1
  run('invalid-amount-'..index,nil,value,false,'effect amount_key damage')
end
-- Custom Rust handlers must not bypass the shared rank-data contract.
run('custom-handler','cooldown=-1',nil,true,'rank 2 cooldown')
run('bad-level','required_hero_level=0',nil,false,'required_hero_level')
run('boolean-level','required_hero_level=false',nil,false,'required_hero_level')
print('ability numeric contract: '..count..'/'..count..' passed')
