local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local m=require('ue_two_team_observation')
local rows={
 'OM_CAST_KEY player=1 slot=3 callbacks=1 queued=1',
 'OM_FIRST_LEARN_CAST submitted player=1 input=3 slot=3 hp=900.000 max_hp=1200.000 cooldown=0.000 tick=600',
 'OM_FIRST_LEARN_CAST result player=1 input=3 status=0 tick=602',
 'OM_FIRST_LEARN_CAST complete player=1 input=3 slot=3 hp=1040.000 max_hp=1200.000 cooldown=24.950 tick=604',
}
local text=table.concat(rows,'\n')
assert(m.observe_first_learn_cast(text,1).complete)
assert(not m.observe_first_learn_cast(text,2).complete)
for i=1,#rows do
 local copy={};for k,v in ipairs(rows) do if k~=i then copy[#copy+1]=v end end
 assert(not m.observe_first_learn_cast(table.concat(copy,'\n'),1).complete)
 assert(not m.observe_first_learn_cast(text..'\n'..rows[i],1).complete)
end
for _,case in ipairs({{'callbacks=1','callbacks=2'},{'queued=1','queued=0'},{'status=0','status=1'},
 {'slot=3','slot=0'},{'hp=1040.000','hp=900.000'},{'hp=1040.000','hp=1201.000'},
 {'cooldown=24.950','cooldown=-1.000'},{'input=3 slot=3 hp=1040','input=4 slot=3 hp=1040'},
 {'tick=604','tick=599'}}) do assert(not m.observe_first_learn_cast((text:gsub(case[1],case[2])),1).complete) end
assert(not m.observe_first_learn_cast(text..'\nOM_FIRST_LEARN_CAST failed player=1 reason=dispatch tick=604',1).complete)
local learn='OM_UPGRADE_KEY player=1 slot=3 callbacks=1 queued=1\nOM_UPGRADE_SMOKE submitted player=1 input=2 slot=3 rank=0 sp=1 level=1 xp=0 tick=590 hud=1\nOM_UPGRADE_SMOKE result player=1 input=2 status=0 tick=592\nOM_UPGRADE_SMOKE complete player=1 input=2 slot=3 rank=1 sp=0 level=1 xp=0 tick=594 hud=1'
assert(m.observe_upgrade(learn,1,true,3).complete)
assert(not m.observe_upgrade(learn,1,true).complete)
print('first learning cast observation: passed')
