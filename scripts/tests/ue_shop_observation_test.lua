local source = debug.getinfo(1, 'S').source:sub(2)
package.path = source:match('^(.*)[/\\]tests[/\\]') .. '/?.lua;' .. package.path
local observe = require('ue_two_team_observation').observe_shop
local lines = {
  'OM_SHOP_SMOKE queued player=1 stage=0 accepted=1 request=1 tick=10',
  'OM_SHOP_SMOKE rejected player=1 input=5 code=6 tick=12',
  'OM_SHOP_SMOKE queued player=1 stage=2 accepted=1 request=2 tick=10620',
  'OM_SHOP_SMOKE bought player=1 input=6 gold=0 slot0=1 tick=10622',
  'OM_SHOP_SMOKE queued player=1 stage=4 accepted=1 request=3 tick=10623',
  'OM_SHOP_SMOKE complete player=1 input=7 gold=175 empty=6 tick=10625 pending=0',
}
local text = table.concat(lines, '\n')
assert(observe(text, 1).complete)
assert(not observe(text, 2).complete)
assert(not observe(lines[6], 1).complete)
assert(not observe(text:gsub('accepted=1', 'accepted=0'), 1).complete)
assert(not observe(text:gsub('gold=175', 'gold=174'), 1).complete)
assert(not observe(text:gsub('input=7', 'input=6'), 1).complete)
assert(not observe(text:gsub('request=3', 'request=2'), 1).complete)
assert(not observe(text:gsub('tick=10625', 'tick=10621'), 1).complete)
assert(not observe(text:gsub('code=6', 'code=0'), 1).complete)
assert(not observe(text:gsub('slot0=1', 'slot0=0'), 1).complete)
assert(not observe(text:gsub('empty=6', 'empty=5'), 1).complete)
assert(not observe(text:gsub('pending=0', 'pending=1'), 1).complete)
local bootstrap = require('_bootstrap')
local json = bootstrap.lib('json')
local round_trip = json.decode(json.encode(observe(text, 1)))
assert(round_trip.complete and round_trip.queued['0'].request == 1 and round_trip.queued['4'].request == 3)
print('ue shop observation: 12 ordered settlement scenarios passed')
print('ue shop observation: sparse stage keys JSON round-trip passed')
local observe_buttons = require('ue_two_team_observation').observe_shop_buttons
local clicks = table.concat({
  'OM_SHOP_BUTTON player=1 action=buy value=1 pressed=1 released=1 callbacks=1 request=1 accepted=1',
  'OM_SHOP_BUTTON player=1 action=buy value=1 pressed=1 released=1 callbacks=1 request=2 accepted=1',
  'OM_SHOP_BUTTON player=1 action=sell value=0 pressed=1 released=1 callbacks=1 request=3 accepted=1',
}, '\n')
local button_text = text .. '\n' .. clicks
assert(observe_buttons(button_text, 1).complete)
assert(not observe_buttons(button_text, 2).complete)
assert(not observe_buttons(text, 1).complete)
assert(not observe_buttons(clicks, 1).complete)
assert(not observe_buttons(button_text:gsub('callbacks=1', 'callbacks=2'), 1).complete)
assert(not observe_buttons(button_text:gsub('pressed=1', 'pressed=0'), 1).complete)
assert(not observe_buttons(button_text:gsub('released=1', 'released=0'), 1).complete)
assert(not observe_buttons(button_text:gsub('callbacks=1 request=2 accepted=1', 'callbacks=1 request=2 accepted=0'), 1).complete)
assert(not observe_buttons(button_text:gsub('value=0', 'value=5'), 1).complete)
assert(not observe_buttons(button_text:gsub('request=3 accepted=1', 'request=4 accepted=1'), 1).complete)
assert(not observe_buttons(button_text:gsub('action=sell', 'action=buy'), 1).complete)
assert(not observe_buttons(button_text .. '\n' .. clicks, 1).complete)
assert(json.decode(json.encode(observe_buttons(button_text, 1))).complete)
print('ue shop button observation: 12 pointer/callback/request scenarios and JSON round-trip passed')
