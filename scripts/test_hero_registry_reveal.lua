local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,process,platform=b.lib('path'),b.lib('process'),b.lib('platform')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','reveal-registry-tests'))
local root
for i=1,1000 do local p=path.join(parent,os.time()..'-'..i);if lfs.mkdir(p) then root=p;break end end
assert(root)
local count=0
local function run(radius,duration,range,target,valid)
  count=count+1
  local directory=path.join(root,tostring(count),'templates')
  path.write(path.join(directory,'heroes.lua'),"return function(ctx) return {{id='fixture',abilities={'flare'}}} end")
  path.write(path.join(directory,'abilities.lua'),([[return function(ctx) return {{
    id='flare',ability_type='active',cast_type='instant',target_type='%s',max_level=1,
    levels={{range=%s}},extras={radius=%s,time=%s},effects={{kind='reveal_area',radius_key='radius',duration_key='time'}}
  }} end]]):format(target,range,radius,duration))
  local result=process.run(platform.lua_executable,{path.join(b.root,'scripts','gen_hero_registry.lua'),
    path.parent(directory),'registry'},{cwd=b.root,check=false})
  assert((result.exit_code==0)==valid,result.stderr)
  if valid then assert(result.stdout:find('EffectOp::RevealArea',1,true))
  else assert(result.stderr:find('flare',1,true),'missing content diagnosis') end
end
for _,radius in ipairs({'{1/1024}','{450}','{2000}'}) do run(radius,'{4}','1000','point',true) end
for _,duration in ipairs({'{1/1024}','{30}'}) do run('{450}',duration,'1000','point',true) end
for _,radius in ipairs({'{0}','{2000.5}','{0/0}','{}','{1,2}'}) do run(radius,'{4}','1000','point',false) end
for _,duration in ipairs({'{0}','{30.5}','{math.huge}'}) do run('{450}',duration,'1000','point',false) end
for _,range in ipairs({'0','10001'}) do run('{450}','{4}',range,'point',false) end
for _,target in ipairs({'none','unit'}) do run('{450}','{4}','1000',target,false) end
print(('reveal registry: %d/%d passed; authoring fixtures only'):format(count,count))
