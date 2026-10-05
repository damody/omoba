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
local sharing=fixture('sharing',[[return function(ctx)
  local data=ctx.include('templates/shared.lua')
  return {data.hero,{id='second',abilities={'fixture_skill'}}}
end]],shared)
local sharing_registry=run(sharing,'registry')
local _,registrations=sharing_registry.stdout:gsub('generic_effect_ffi','')
assert(sharing_registry.exit_code==0 and registrations==1,'shared ability must register only once')
local _,ids=run(sharing,'ids').stdout:gsub('"fixture_skill"','')
assert(ids==1,'shared ability must export only one ID')
print('PASS shared ability registers once')
local conflict=fixture('conflict',[[return function(ctx)
  local data=ctx.include('templates/shared.lua')
  return {data.hero,{id='second',rust_module='special',abilities={'fixture_skill'}}}
end]],shared)
local rejected=run(conflict,'registry')
assert(rejected.exit_code~=0 and rejected.stderr:find('conflicting hero handler implementations',1,true))
print('PASS conflicting shared implementation rejected')
for _,hostile in ipairs({"{kind='damage',amount_key='damage',damage_kind='physical'}",
  "{kind='slow_enemy',reduction_key='reduction',duration_key='duration'}",
  "{kind='stun_enemy',duration_key='duration'}",
  "{kind='control_enemy',control='root',duration_key='duration'}",
  "{kind='control_enemy',control='silence',duration_key='duration'}"}) do
  for _,ally_first in ipairs({true,false}) do
    local ally="{kind='heal_ally',amount_key='damage'}"
    local effects=ally_first and (ally..','..hostile) or (hostile..','..ally)
    local mixed=shared:gsub("effects=%b{}","effects={"..effects.."}")
      :gsub("extras={damage={20}}","extras={damage={20},reduction={0.25},duration={2}}",1)
    local dir=fixture('mixed-'..assert(hostile:match("kind='([%w_]+)'"))..'-'..(hostile:match("control='([%w_]+)'") or 'none')..'-'..tostring(ally_first),
      'return function(ctx) return {ctx.include("templates/shared.lua").hero} end',mixed)
    local result=run(dir,'registry')
    assert(result.exit_code~=0 and result.stderr:find('conflicting unit target allegiance',1,true))
  end
end
print('PASS mixed ally/enemy effects rejected in either order')
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
print('hero registry include functionality: 8/8 passed')
