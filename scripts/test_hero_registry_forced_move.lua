local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,platform=b.lib('path'),b.lib('process'),b.lib('platform')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','forced-move-registry-tests'))
local root
for i=1,1000 do local p=path.join(parent,os.time()..'-'..i);if lfs.mkdir(p) then root=p;break end end
assert(root)
local count=0
-- One fixture ability: a unit-target forced move, or the payload of a linear
-- projectile when `projectile` is set.
local function run(motion,distance,duration,target,projectile,valid,diagnosis)
  count=count+1
  local directory=path.join(root,tostring(count),'templates')
  path.write(path.join(directory,'heroes.lua'),"return function(ctx) return {{id='fixture',abilities={'shove'}}} end")
  local delivery=projectile and ",projectile={motion='linear',speed_key='speed',radius_key='radius',max_hits=1,wall='stop'}" or ''
  local damage=projectile and "{kind='damage',amount_key='speed',damage_kind='magical'}," or ''
  path.write(path.join(directory,'abilities.lua'),([[return function(ctx) return {{
    id='shove',ability_type='active',cast_type='instant',target_type='%s',max_level=1,
    levels={{range=600}},extras={distance=%s,duration=%s,speed={1000},radius={50}},
    effects={%s{kind='forced_move_enemy',motion=%s,distance_key='distance',duration_key='duration'}}%s
  }} end]]):format(target,distance,duration,damage,motion,delivery))
  local result=process.run(platform.lua_executable,{path.join(b.root,'scripts','gen_hero_registry.lua'),
    path.parent(directory),'registry'},{cwd=b.root,check=false})
  assert((result.exit_code==0)==valid,result.stderr)
  if valid then assert(result.stdout:find('EffectOp::ForcedMoveEnemy',1,true))
  else assert(result.stderr:find(diagnosis,1,true),'missing content diagnosis: '..result.stderr) end
end
for _,motion in ipairs({"'knockback'","'pull'"}) do
  for _,distance in ipairs({'{1/1024}','{350}','{3000}'}) do run(motion,distance,'{0.4}','unit',false,true) end
  for _,duration in ipairs({'{1/1024}','{5}'}) do run(motion,'{350}',duration,'unit',false,true) end
  run(motion,'{1100}','{0.5}','point',true,true)
  for _,distance in ipairs({'{0}','{-1}','{3000.5}','{0/0}','{}','{1,2}'}) do run(motion,distance,'{0.4}','unit',false,false,'forced_move_enemy') end
  for _,duration in ipairs({'{0}','{5.1}','{math.huge}'}) do run(motion,'{350}',duration,'unit',false,false,'forced_move_enemy') end
  for _,target in ipairs({'none','point'}) do run(motion,'{350}','{0.4}',target,false,false,'shove') end
end
for _,motion in ipairs({"'throw'","'Pull'","''","0","nil"}) do run(motion,'{350}','{0.4}','unit',false,false,'motion') end
print(('forced move registry: %d/%d passed; authoring fixtures only'):format(count,count))
