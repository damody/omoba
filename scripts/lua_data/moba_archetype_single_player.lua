local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_archetype_match.lua'))()
for _,player in ipairs(plan.players) do if player.player_id==1 then player.bot=false end end
return plan
