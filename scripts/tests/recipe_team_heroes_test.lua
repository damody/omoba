local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path = b.lib('path')
local lfs = require('lfs')
local launch = require('moba_role_launch')

-- One hero per team (LoL normal game; the teams may share heroes). Every shipped recipe
-- (scripts/lua_data, source heroes) obeys it; RoleBotMatchPlan::compile refuses a clash and the
-- server moves a clashing bot (resolve_bot_hero_conflicts). Test fixtures under
-- scripts/test_fixtures may repeat a training kit (compile_fixture).
local checks = 0
local function ok(cond, msg) checks = checks + 1; if not cond then error(msg, 2) end end
local data = path.join(b.root, 'scripts', 'lua_data')
for name in lfs.dir(data) do
  if name:match('%.lua$') then
    local loaded, value = pcall(function() return assert(loadfile(path.join(data, name)))() end)
    if loaded and type(value) == 'table' and type(value.players) == 'table' then
      local seen = {}
      for _, player in ipairs(value.players) do
        local key = tostring(player.team_id) .. ':' .. tostring(player.hero)
        ok(not seen[key], name .. ' seats ' .. key .. ' twice')
        seen[key] = true
      end
    end
  end
end

-- --hero: two humans of one team cannot pick the same hero; a bot teammate's hero is fine
-- (the server moves the bot).
local recipe = { players = {
  { player_id = 1, team_id = 1, hero = 'oichi', bot = false },
  { player_id = 2, team_id = 1, hero = 'tamako', bot = false },
  { player_id = 3, team_id = 1, hero = 'saika_magoichi', bot = true },
  { player_id = 6, team_id = 2, hero = 'saika_magoichi', bot = true },
} }
ok(launch.select_heroes(recipe, { [1] = 'saika_magoichi' }).players[1].hero == 'saika_magoichi', 'a bot teammate does not block a pick')
ok(not pcall(launch.select_heroes, recipe, { [1] = 'tamako' }), 'a human teammate blocks the pick')
ok(not pcall(launch.select_heroes, recipe, { [1] = 'date_masamune', [2] = 'date_masamune' }), 'two humans of one team cannot pick alike')
print(('recipe team heroes: %d checks passed'):format(checks))
