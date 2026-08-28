local dir=debug.getinfo(1,'S').source:sub(2):match('^(.*)[/\\]');package.path=dir..'/?.lua;'..package.path
local common=require('common');local o=common.options(arg);assert(o.positional[1],'manifest required');local seed=assert(math.tointeger(tonumber(o.seed)),'--seed required');local drop=tonumber(o.drop_rate or 0);local duplicate=tonumber(o.duplicate_rate or 0);assert(drop>=0 and drop<=1 and duplicate>=0 and duplicate<=1,'rates must be in [0, 1]')
-- Stable local PRNG. Its sequence is part of the Lua tool contract.
local state=seed&0xffffffff;local function random()state=(1664525*state+1013904223)&0xffffffff;return state/0x100000000 end
local schedule={};for _,frame in ipairs(common.load_json(o.positional[1]).frames)do local roll=random();local action=roll<drop and'drop'or roll<drop+duplicate and'duplicate'or'deliver';table.insert(schedule,{team_sequence=frame.team_sequence,action=action})end
common.write_result(o.output,{seed=seed,schedule=schedule})
