local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,platform=b.lib('path'),b.lib('process'),b.lib('platform')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','hero-include-tests'))
local root
for i=1,1000 do local candidate=path.join(parent,os.time()..'-'..i);if lfs.mkdir(candidate) then root=candidate;break end end
assert(root)
local function fixture(name,heroes,shared)
  local dir=path.join(root,name)
  path.write(path.join(dir,'templates','heroes.lua'),heroes)
  path.write(path.join(dir,'templates','abilities.lua'),
    'return function(ctx) return {ctx.include("templates/shared.lua").ability} end')
  if shared then path.write(path.join(dir,'templates','shared.lua'),shared) end
  return dir
end
local shared=[[return function(ctx) return {
  hero={id='fixture_hero',abilities={'fixture_skill'}},
  ability={id='fixture_skill',ability_type='active',cast_type='instant',target_type='unit',
    max_level=1,levels={{range=600}},extras={damage={20}},effects={{kind='damage',amount_key='damage',damage_kind='physical'}}}
} end]]
local function run(root,mode)
  return process.run(platform.lua_executable,{path.join(b.root,'scripts','gen_hero_registry.lua'),root,mode},
    {cwd=b.root,check=false})
end
local valid=fixture('valid','return function(ctx) return {ctx.include("templates/./shared.lua").hero} end',shared)
local registered=run(valid,'registry')
assert(registered.exit_code==0 and registered.stdout:find('generic_effect_ffi',1,true))
assert(registered.stdout:find('DamageKind::Physical',1,true))
assert(run(valid,'ids').stdout:find('"fixture_skill"',1,true))
print('PASS shared include drives both registry and IDs')
for _,case in ipairs({
  {'parent','../outside.lua','parent-directory escape'},
  {'absolute','C:/outside.lua','content-relative path'},
  {'cycle','templates/heroes.lua','include cycle'},
  {'case-cycle','TEMPLATES/HEROES.LUA','include cycle'},
}) do
  local dir=fixture(case[1],'return function(ctx) return ctx.include("'..case[2]..'") end',shared)
  local result=run(dir,'registry')
  assert(result.exit_code~=0 and result.stderr:find(case[3],1,true),case[1]..' did not fail closed')
  print('PASS '..case[1])
end
print('hero registry include functionality: 5/5 passed')
