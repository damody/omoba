local script = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(script:match('^(.*)[/\\]'))
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local json = b.lib('json')
local observe = require('ue_two_team_observation').observe_minimap_move
local click = 'OM_MINIMAP_INPUT player=2 handled=1 callbacks=1 input=17 accepted=1 target=(1920.000,-288.000)'
local result = 'OM_MINIMAP_SMOKE result player=2 input=17 status=0 tick=3500'
local text = click .. '\n' .. result
assert(observe(text, 2).complete)
assert(json.decode(json.encode(observe(text, 2))).complete)
assert(not observe(text, 1).complete)
assert(not observe(click, 2).complete)
assert(not observe(result, 2).complete)
assert(not observe(text .. '\n' .. click, 2).complete)
for _, replacement in ipairs({
  {'handled=1', 'handled=0'}, {'callbacks=1', 'callbacks=0'}, {'callbacks=1', 'callbacks=2'},
  {'accepted=1', 'accepted=0'}, {'input=17', 'input=0'}, {'status=0', 'status=1'}, {'tick=3500', 'tick=0'},
}) do
  assert(not observe(text:gsub(replacement[1], replacement[2]), 2).complete)
end
assert(not observe(click .. '\n' .. result:gsub('input=17', 'input=18'), 2).complete)
print('minimap observer: 14 assertions including JSON round-trip passed')
