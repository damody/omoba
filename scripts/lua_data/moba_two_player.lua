-- Two human seats (one per team) for the same-machine B+C+C topology and later LAN.
-- Server-owned controller recipe, not a special case in Rust or Unreal; 8 seats stay Bots.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_role_match.lua'))()
local humans={[1]=true,[6]=true}
local found=0
for _,player in ipairs(plan.players) do
  if humans[player.player_id] then player.bot=false;found=found+1 end
end
assert(found==2,'two-player recipe requires player 1 (team 1) and player 6 (team 2)')
return plan
