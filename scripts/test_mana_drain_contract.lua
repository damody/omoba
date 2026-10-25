local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,r,platform=b.lib('path'),b.lib('process'),b.lib('platform');local lfs=require('lfs')
local parent=p.mkdir_p(p.join(b.root,'target','mana-drain-contract'));local root
for i=1,1000 do local dir=p.join(parent,os.time()..'-'..i);if lfs.mkdir(dir)then root=dir;break end end;assert(root)
local count=0
local function check(name,target,levels,extras,effects,ok)
 local dir=p.join(root,name);p.write(p.join(dir,'templates','heroes.lua'),'return function() return {{id="arbitrary_hero",abilities={"arbitrary_loss"}}} end')
 p.write(p.join(dir,'templates','abilities.lua'),'return function() return {{id="arbitrary_loss",ability_type="active",cast_type="instant",target_type="'..target..'",max_level=2,levels='..levels..',extras='..extras..',effects={'..effects..'}}} end')
 local result=r.run(platform.lua_executable,{p.join(b.root,'scripts','gen_hero_registry.lua'),dir,'registry'},{cwd=b.root,check=false});count=count+1
 if ok then assert(result.exit_code==0 and result.stdout:find('EffectOp::DrainManaEnemy',1,true),name..': '..result.stderr)else assert(result.exit_code~=0,name..' should reject')end
end
local effect='{kind="drain_mana_enemy",amount_key="amount"}';local levels='{{range=600},{range=600}}';local extras='{amount={20,30}}'
check('valid','unit',levels,extras,effect,true)
check('edges','unit','{{range=1/1024},{range=10000}}','{amount={1/1024,1000000}}',effect,true)
for i,value in ipairs({'0','-1','0.0001','1000001','math.huge','0/0'})do check('amount-'..i,'unit',levels,'{amount={20,'..value..'}}',effect,false)end
for i,value in ipairs({'{}','{amount={20}}'})do check('missing-'..i,'unit',levels,value,effect,false)end
for _,target in ipairs({'none','point'})do check('target-'..target,target,levels,extras,effect,false)end
for i,value in ipairs({'{{},{}}','{{range=0},{range=600}}','{{range=600},{range=10000.01}}'})do check('range-'..i,'unit',value,extras,effect,false)end
local heal='{kind="heal_ally",amount_key="amount"}';check('conflict','unit',levels,extras,heal..','..effect,false);check('conflict-reversed','unit',levels,extras,effect..','..heal,false)
for _,kind in ipairs({'damage','restore_mana_self'})do local sibling='{kind="'..kind..'",amount_key="amount",damage_kind="magical"}'
 check(kind,'unit',levels,extras,effect..','..sibling,true);check(kind..'-reversed','unit',levels,extras,sibling..','..effect,true)end
check('self-spend','unit',levels,extras,effect..',{kind="spend_mana_self",amount_key="amount"}',false)
assert(count==22);print('mana drain contract: '..count..'/'..count..' passed')
