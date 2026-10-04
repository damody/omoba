local script = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(script:match('^(.*)[/\\]')) .. '/../?.lua;' .. package.path
local observe = require('ue_two_team_observation').observe_upgrade
local rows={
 'OM_UPGRADE_KEY player=1 slot=0 callbacks=1 queued=1',
 'OM_UPGRADE_SMOKE submitted player=1 input=2 slot=0 rank=1 sp=1 level=2 xp=0 tick=500 hud=1',
 'OM_UPGRADE_SMOKE result player=1 input=2 status=0 tick=510',
 'OM_UPGRADE_SMOKE complete player=1 input=2 slot=0 rank=2 sp=0 level=2 xp=0 tick=510 hud=1',
}
local text=table.concat(rows,'\n')
assert(observe(text,1).complete)
assert(not observe(text,2).complete)
for i=1,#rows do local copy={};for k,v in ipairs(rows) do if k~=i then copy[#copy+1]=v end end;assert(not observe(table.concat(copy,'\n'),1).complete) end
for _,case in ipairs({{'callbacks=1','callbacks=2'},{'queued=1','queued=0'},{'status=0','status=1'},{'rank=2','rank=3'},{'sp=0','sp=1'},{'hud=1','hud=0'},{'tick=510','tick=490'}}) do assert(not observe((text:gsub(case[1],case[2])),1).complete) end
assert(not observe(text..'\n'..rows[1],1).complete)
assert(not observe(text..'\nOM_UPGRADE_SMOKE failed player=1 reason=HUD tick=511',1).complete)
local nextlevel=text:gsub('rank=2 sp=0 level=2','rank=2 sp=1 level=3')
assert(observe(nextlevel,1).complete)
local first=text:gsub('rank=1 sp=1 level=2','rank=0 sp=1 level=1'):gsub('rank=2 sp=0 level=2','rank=1 sp=0 level=1')
assert(observe(first,1,true).complete)
assert(not observe(first,1).complete)
assert(not observe(text,1,true).complete)
assert(not observe(first:gsub('rank=0 sp=1','rank=0 sp=2'),1,true).complete)
assert(not observe(first:gsub('rank=1 sp=0','rank=1 sp=1'),1,true).complete)
assert(not observe(first..'\n'..rows[1],1,true).complete)
for i=1,#rows do
 local copy={}; for k,v in ipairs(rows) do if k~=i then copy[#copy+1]=v end end
 local missing=table.concat(copy,'\n'):gsub('rank=1 sp=1 level=2','rank=0 sp=1 level=1'):gsub('rank=2 sp=0 level=2','rank=1 sp=0 level=1')
 assert(not observe(missing,1,true).complete)
end
print('upgrade observation: passed')
