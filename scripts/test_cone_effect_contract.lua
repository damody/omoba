local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,r,platform=b.lib('path'),b.lib('process'),b.lib('platform');local lfs=require('lfs')
local parent=p.mkdir_p(p.join(b.root,'target','cone-effect-tests'));local root
for n=1,1000 do local dir=p.join(parent,os.time()..'-'..n);if lfs.mkdir(dir)then root=dir;break end end;assert(root)
local count=0
local function check(name,angle,mutate,error)
 local dir=p.join(root,name);local skill={id='arbitrary_sector',max_level=2,ability_type='active',cast_type='instant',target_type='point',
  levels={{range=700},{range=900}},extras={hit={100,200},angle=angle},effects={{kind='cone_damage',amount_key='hit',half_angle_key='angle',damage_kind='physical'}}}
 if mutate then mutate(skill)end
 local function literal(v)if type(v)=='string'then return string.format('%q',v)elseif type(v)=='number'then return tostring(v)elseif type(v)=='table'then local parts={};for k,x in pairs(v)do parts[#parts+1]='['..literal(k)..']='..literal(x)end;return '{'..table.concat(parts,',')..'}'end;error('unsupported fixture')end
 p.write(p.join(dir,'templates','heroes.lua'),'return function() return {{id="unrelated",abilities={"arbitrary_sector"}}} end')
 p.write(p.join(dir,'templates','abilities.lua'),'return function() return {'..literal(skill)..'} end')
 local v=r.run(platform.lua_executable,{p.join(b.root,'scripts','gen_hero_registry.lua'),dir,'registry'},{cwd=b.root,check=false});count=count+1
 if error then assert(v.exit_code~=0 and v.stderr:find(error,1,true),name..': '..v.stderr)else assert(v.exit_code==0 and v.stdout:find('EffectOp::ConeDamage',1,true),name..': '..v.stderr)end
end
check('valid',{math.pi/6,math.pi/3});check('edges',{1/1024,math.pi/2})
for _,angle in ipairs({0,0.0001,-1,math.pi})do check('angle'..angle,{math.pi/6,angle},nil,'angle')end
check('missing-rank',{math.pi/6},nil,'angle')
check('wrong-target',{1,1},function(s)s.target_type='unit'end,'target_type')
check('mixed',{1,1},function(s)s.effects[2]={kind='heal_self',amount_key='hit'}end,'exclusive')
check('ambiguous',{1,1},function(s)s.aim_geometry={kind='point_circle',radius_key='hit'}end,'exclusive')
check('zero-range',{1,1},function(s)s.levels[2].range=0 end,'range')
print('cone effect contract: '..count..'/'..count..' passed')
