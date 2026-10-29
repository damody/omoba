-- Role match with each role's source hero (inherited from moba_role_match.lua) plus the
-- bot cleanse summoner. Test-only prototypes live in
-- scripts/test_fixtures/recipes/moba_training_archetype_match.lua, never in a game recipe.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_role_match.lua'))()
-- Stationary recovery needs a channel-heal item; the shop has none from the source yet
-- (moba_recovery_charm is an engine fixture, E155), so supports do not buy one.
-- Bot assignment plus explicit equipment enables owner-only automatic cleansing.
-- Keep D unavailable and preserve omitted loadouts for other roles/humans.
local null=require('tools.lua.lib.json').null
plan.summoner_loadouts=plan.summoner_loadouts or {}
for _,player in ipairs(plan.players) do
  if player.bot and player.role=='support' then
    plan.summoner_loadouts[tostring(player.player_id)]={null,'training_control_cleanse_v1'}
  end
end
return plan
