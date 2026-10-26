-- One human seat (player 1) and nine bots on the live-check map: the same layout and rules
-- as moba_two_player_quick.lua, so a single-human match finishes inside a 10-minute run.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_two_player_quick.lua'))()
local humans=0
for _,player in ipairs(plan.players) do
  if player.player_id~=1 then player.bot=true end
  if player.bot==false then humans=humans+1 end
end
assert(humans==1,'single-player quick recipe requires exactly its declared human player')
return plan
