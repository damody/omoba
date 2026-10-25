local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,r,platform=b.lib('path'),b.lib('process'),b.lib('platform');local lfs=require('lfs')
local parent=p.mkdir_p(p.join(b.root,'target','ally-shield-contract'));local root
for index=1,1000 do local candidate=p.join(parent,os.time()..'-'..index);if lfs.mkdir(candidate)then root=candidate;break end end;assert(root)
local count=0
local function check(name,target,levels,extras,effects,ok)
 local dir=p.join(root,name);p.write(p.join(dir,'templates','heroes.lua'),'return function() return {{id="arbitrary_guard",abilities={"arbitrary_skill"}}} end')
 p.write(p.join(dir,'templates','abilities.lua'),'return function() return {{id="arbitrary_skill",ability_type="active",cast_type="instant",target_type="'..target..'",max_level=2,levels='..levels..',extras='..extras..',effects={'..effects..'}}} end')
 local result=r.run(platform.lua_executable,{p.join(b.root,'scripts','gen_hero_registry.lua'),dir,'registry'},{cwd=b.root,check=false});count=count+1
 if ok then assert(result.exit_code==0 and result.stdout:find('EffectOp::ShieldAlly',1,true),name..': '..result.stderr)
 else assert(result.exit_code~=0,name..' should reject')end
end
local shield='{kind="shield_ally",amount_key="amount",duration_key="duration"}'
local levels='{{range=600},{range=600}}';local extras='{amount={70,100},duration={2,2}}'
check('valid','unit',levels,extras,shield,true)
check('edges','unit','{{range=1/1024},{range=10000}}','{amount={1/1024,1000000},duration={1/1024,60}}',shield,true)
for index,value in ipairs({'0','-1','0.0001','1000001','math.huge','0/0'})do check('amount-'..index,'unit',levels,'{amount={70,'..value..'},duration={2,2}}',shield,false)end
for index,value in ipairs({'0','-1','0.0001','60.01','math.huge','0/0'})do check('duration-'..index,'unit',levels,'{amount={70,100},duration={2,'..value..'}}',shield,false)end
for index,value in ipairs({'{}','{amount={70,100}}','{amount={70},duration={2,2}}'})do check('missing-'..index,'unit',levels,value,shield,false)end
for _,target in ipairs({'none','point'})do check('target-'..target,target,levels,extras,shield,false)end
for index,value in ipairs({'{{},{}}','{{range=0},{range=600}}','{{range=600},{range=10000.01}}'})do check('range-'..index,'unit',value,extras,shield,false)end
local damage='{kind="damage",amount_key="amount",damage_kind="magical"}'
check('conflict','unit',levels,extras,shield..','..damage,false);check('conflict-reversed','unit',levels,extras,damage..','..shield,false)
local heal='{kind="heal_ally",amount_key="amount"}';local restore='{kind="restore_mana_self",amount_key="amount"}'
check('composed','unit',levels,extras,heal..','..restore..','..shield,true);check('composed-reversed','unit',levels,extras,shield..','..restore..','..heal,true)
check('self-only-spend','unit',levels,extras,shield..',{kind="spend_mana_self",amount_key="amount"}',false)
print('ally shield contract: '..count..'/'..count..' passed')
