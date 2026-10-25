local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,r,platform=b.lib('path'),b.lib('process'),b.lib('platform');local lfs=require('lfs')
local parent=p.mkdir_p(p.join(b.root,'target','ally-cleanse-contract'));local root
for index=1,1000 do local candidate=p.join(parent,os.time()..'-'..index);if lfs.mkdir(candidate)then root=candidate;break end end;assert(root)
local count=0
local function check(name,target,levels,extras,effects,ok)
 local dir=p.join(root,name);p.write(p.join(dir,'templates','heroes.lua'),'return function() return {{id="arbitrary_guard",abilities={"arbitrary_skill"}}} end')
 p.write(p.join(dir,'templates','abilities.lua'),'return function() return {{id="arbitrary_skill",ability_type="active",cast_type="instant",target_type="'..target..'",max_level=2,levels='..levels..',extras='..extras..',effects={'..effects..'}}} end')
 local result=r.run(platform.lua_executable,{p.join(b.root,'scripts','gen_hero_registry.lua'),dir,'registry'},{cwd=b.root,check=false});count=count+1
 if ok then assert(result.exit_code==0 and result.stdout:find('EffectOp::CleanseAlly',1,true),name..': '..result.stderr)else assert(result.exit_code~=0,name..' should reject')end
end
local cleanse='{kind="cleanse_ally"}';local levels='{{range=600},{range=600}}'
check('valid','unit',levels,'{}',cleanse,true);check('edges','unit','{{range=1/1024},{range=10000}}','{}',cleanse,true)
for _,target in ipairs({'none','point'})do check('target-'..target,target,levels,'{}',cleanse,false)end
for index,value in ipairs({'0','-1','0.0001','10000.01','math.huge','0/0'})do check('range-'..index,'unit','{{range=600},{range='..value..'}}','{}',cleanse,false)end
check('closed','unit',levels,'{}','{kind="cleanse_ally",buffs={"anything"}}',false)
local restore='{kind="restore_mana_self",amount_key="mana"}';local heal='{kind="heal_ally",amount_key="mana"}';local damage='{kind="damage",amount_key="mana",damage_kind="magical"}'
check('composed','unit',levels,'{mana={10,15}}',heal..','..restore..','..cleanse,true)
check('reversed','unit',levels,'{mana={10,15}}',cleanse..','..restore..','..heal,true)
check('conflict','unit',levels,'{mana={10,15}}',cleanse..','..damage,false)
check('conflict-reversed','unit',levels,'{mana={10,15}}',damage..','..cleanse,false)
check('self-spend','unit',levels,'{mana={10,15}}',cleanse..',{kind="spend_mana_self",amount_key="mana"}',false)
print('ally cleanse contract: '..count..'/'..count..' passed')
