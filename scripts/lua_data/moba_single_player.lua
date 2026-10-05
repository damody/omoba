-- A server-owned controller recipe, not a special case in Rust or Unreal.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_role_match.lua'))()
local found=false
for _,player in ipairs(plan.players) do
  if player.player_id==1 then player.bot=false;found=true end
end
assert(found,'single-player recipe requires its declared human player')
return plan
