local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,r,platform=b.lib('path'),b.lib('process'),b.lib('platform');local lfs=require('lfs')
local parent=p.mkdir_p(p.join(b.root,'target','recovery-skill-contract'));local root
for i=1,1000 do local candidate=p.join(parent,os.time()..'-'..i);if lfs.mkdir(candidate)then root=candidate;break end end;assert(root)
local count=0
local function check(pulse,duration,interval,cast,target,preparation,extra,expected)
 count=count+1;local dir=p.join(root,tostring(count))
 p.write(p.join(dir,'templates/heroes.lua'),"return function()return {{id='arbitrary_hero',abilities={'arbitrary_channel'}}}end")
 p.write(p.join(dir,'templates/abilities.lua'),string.format([[return function()return {{id='arbitrary_channel',max_level=1,ability_type='active',cast_type='%s',target_type='%s',levels={{cast_time=%s}},extras={pulse={%s},duration={%s},interval={%s}},effects={{kind='recovery_channel_self',amount_key='pulse',duration_key='duration',interval_key='interval'}%s}}}end]],cast or 'channeled',target or 'none',preparation or '0',pulse,duration,interval,extra or ''))
 local result=r.run(platform.lua_executable,{p.join(b.root,'scripts/gen_hero_registry.lua'),dir,'registry'},{cwd=b.root,check=false})
 assert((result.exit_code==0)==expected,'case'..count..': '..(result.stderr or ''))
 if expected then assert(result.stdout:find('EffectOp::RecoveryChannelSelf',1,true))end
end
check('35','2','0.5',nil,nil,nil,nil,true)
check('1/1024','0.25','0.25',nil,nil,nil,nil,true)
for _,spec in ipairs({{'0','2','0.5'},{'0.1','2','0.5'},{'1000001','2','0.5'},{'35','61','0.5'},{'35','1.25','0.5'},
 {'35','2','0.1'},{'35','2','11'},{'false','2','0.5'},{'35','0','0.5'}})do check(spec[1],spec[2],spec[3],nil,nil,nil,nil,false)end
check('35','2','0.5','instant',nil,nil,nil,false)
check('35','2','0.5',nil,'point',nil,nil,false)
check('35','2','0.5',nil,nil,'0.5',nil,false)
check('35','2','0.5',nil,nil,nil,",{kind='heal_self',amount_key='pulse'}",false)
local cache,ctx={},{}
ctx.include=function(file)
 if cache[file]then return cache[file]end
 local result=assert(loadfile(p.join(b.root,'scripts/lua_data',file)))()(ctx)
 if file=='templates/moba_archetypes.lua'then result.abilities[#result.abilities+1]={id='renamed_channel',cast_type='channeled',target_type='none',ability_type='active',max_level=1,levels={{}},
  extras={pulse={2},duration={3},interval={1}},effects={{kind='recovery_channel_self',amount_key='pulse',duration_key='duration',interval_key='interval'}}}end
 cache[file]=result;return result
end
local actual,other,renamed
for _,ability in ipairs(ctx.include('templates/abilities.lua'))do if ability.id=='apprentice_mend'then actual=ability elseif ability.id=='lumen_mend'then other=ability elseif ability.id=='renamed_channel'then renamed=ability end end
assert(actual and actual.cast_type=='channeled' and actual.description:find('35/55/75/95',1,true) and actual.description:find('140/220/300/380',1,true));count=count+1
assert(other and other.cast_type=='instant');count=count+1
assert(renamed and renamed.description:find('引導3秒',1,true) and renamed.description:find('每1秒恢復2生命',1,true) and renamed.description:find('總計6',1,true));count=count+1
print(('recovery skill Lua contract: %d/%d passed; closed generator and data-derived actual/arbitrary skill guidance'):format(count,count))
