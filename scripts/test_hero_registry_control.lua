local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,platform=b.lib('path'),b.lib('process'),b.lib('platform')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','control-registry-tests'))
local root
for i=1,1000 do local p=path.join(parent,os.time()..'-'..i);if lfs.mkdir(p) then root=p;break end end
assert(root)
local count=0
local function run(control,duration,range,target,valid)
  count=count+1
  local directory=path.join(root,tostring(count),'templates')
  path.write(path.join(directory,'heroes.lua'),"return function(ctx) return {{id='fixture',abilities={'control'}}} end")
  path.write(path.join(directory,'abilities.lua'),([[return function(ctx) return {{
    id='control',ability_type='active',cast_type='instant',target_type='%s',max_level=1,
    levels={{range=%s}},extras={duration=%s},effects={{kind='control_enemy',control=%s,duration_key='duration'}}
  }} end]]):format(target,range,duration,control))
  local result=process.run(platform.lua_executable,{path.join(b.root,'scripts','gen_hero_registry.lua'),
    path.parent(directory),'registry'},{cwd=b.root,check=false})
  assert((result.exit_code==0)==valid,result.stderr)
  if valid then assert(result.stdout:find('EffectOp::ControlEnemy',1,true))
  else assert(result.stderr:find('control',1,true),'missing content diagnosis') end
end
for _,control in ipairs({"'stun'","'root'","'silence'"}) do
  for _,duration in ipairs({'{1/1024}','{1}','{60}'}) do run(control,duration,'300','unit',true) end
  for _,duration in ipairs({'{0}','{-1}','{0.0001}','{60.1}','{0/0}','{math.huge}','{}','{1,2}'}) do
    run(control,duration,'300','unit',false)
  end
  for _,range in ipairs({'0','10001'}) do run(control,'{1}',range,'unit',false) end
  for _,target in ipairs({'none','point'}) do run(control,'{1}','300',target,false) end
end
for _,control in ipairs({"'burn'","'Root'","''","0","nil"}) do run(control,'{1}','300','unit',false) end
print(('control registry: %d/%d passed; authoring fixtures only'):format(count,count))
