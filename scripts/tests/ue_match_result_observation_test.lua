local script = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(script:match('^(.*)[/\\]')) .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local json = b.lib('json')
local observe = require('ue_two_team_observation').observe_match_result_ui
local lifecycle = 'OM_MATCH_STATE player=7 phase=1 alive=1 winner=0 hero=4 epoch=1 respawn=0 tick=10\n'
  .. 'OM_MATCH_STATE player=7 phase=1 alive=0 winner=0 hero=0 epoch=0 respawn=5 tick=20\n'
  .. 'OM_MATCH_STATE player=7 phase=1 alive=1 winner=0 hero=4 epoch=2 respawn=0 tick=30\n'
  .. 'OM_MATCH_STATE player=7 phase=2 alive=1 winner=2 hero=4 epoch=2 respawn=0 tick=40\n'
local ui = 'OM_MATCH_RESULT_UI player=7 team=2 winner=2 outcome=Victory tick=41'
local count = 0
local function check(value) assert(value); count = count + 1 end
check(observe(lifecycle .. ui, 7, 2).complete)
check(json.decode(json.encode(observe(lifecycle .. ui, 7, 2))).outcome == 'Victory')
check(not observe(lifecycle, 7, 2).complete)
check(not observe(ui, 7, 2).complete)
check(not observe(lifecycle .. ui, 1, 2).complete)
check(not observe(lifecycle .. ui, 7, 0).complete)
for _, replacement in ipairs({{'team=2','team=1'}, {'winner=2','winner=1'}, {'Victory','Defeat'}, {'tick=41','tick=39'}}) do
  check(not observe(lifecycle .. ui:gsub(replacement[1], replacement[2]), 7, 2).complete)
end
check(not observe(lifecycle .. ui:gsub('Victory','Defeat') .. '\n' .. ui, 7, 2).complete)
check(observe(lifecycle:gsub('winner=2','winner=1') .. ui:gsub('winner=2','winner=1'):gsub('Victory','Defeat'), 7, 2).complete)
check(observe(lifecycle:gsub('winner=2','winner=0') .. ui:gsub('winner=2','winner=0'):gsub('Victory','Draw'), 7, 2).complete)
check(not observe(lifecycle:gsub('phase=2','phase=1') .. ui, 7, 2).complete)
print('match result observer: ' .. count .. ' assertions including JSON round-trip passed')
