-- Opt-in: moba_two_player_quick with explicit D/F summoner loadouts for every seat, so loading
-- cards, the HUD D/F slots and bot spell use can be checked live. The base recipes stay
-- unchanged (no seat equips a spell there, and the backend never adds a default spell).
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_two_player_quick.lua'))()
local rows={
  {'training_point_blink_v1','training_self_heal_v1'},
  {'training_self_shield_v1','training_point_blink_v1'},
  {'training_self_heal_v1','training_self_sprint_v1'},
  {'training_control_cleanse_v1','training_self_heal_v1'},
  {'training_point_blink_v1','training_self_guard_v1'},
}
plan.summoner_loadouts={}
for index,player in ipairs(plan.players) do
  plan.summoner_loadouts[tostring(player.player_id)]=rows[(index-1)%#rows+1]
end
return plan
