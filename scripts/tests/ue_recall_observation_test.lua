local script = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(script:match('^(.*)[/\\]')) .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local json = b.lib('json')
local observe = require('ue_two_team_observation').observe_recall
local rows = {
  'OM_RECALL_INPUT player=1 input=2 queued=1',
  'OM_RECALL_KEY player=1 callbacks=1 queued=1',
  'OM_RECALL_SMOKE result player=1 cycle=1 input=2 status=0 tick=500',
  'OM_RECALL_SMOKE active player=1 cycle=1 input=2 remaining=7.983 tick=500 hud=1',
  'OM_RECALL_SMOKE move_result player=1 input=3 status=0 tick=621',
  'OM_RECALL_SMOKE canceled player=1 input=2 move=3 tick=621',
  'OM_RECALL_INPUT player=1 input=4 queued=1',
  'OM_RECALL_KEY player=1 callbacks=1 queued=1',
  'OM_RECALL_SMOKE result player=1 cycle=2 input=4 status=0 tick=700',
  'OM_RECALL_SMOKE active player=1 cycle=2 input=4 remaining=7.983 tick=700 hud=1',
  'OM_RECALL_SMOKE complete player=1 input=4 tick=1179 x=0.000 y=0.000 base_x=0.000 base_y=0.000 hud=1',
}
local text = table.concat(rows, '\n')
assert(observe(text, 1).complete)
assert(json.decode(json.encode(observe(text, 1))).complete)
assert(not observe(text, 2).complete)
for i = 1, #rows do
  local copy = {}; for j, row in ipairs(rows) do if i ~= j then copy[#copy+1] = row end end
  assert(not observe(table.concat(copy, '\n'), 1).complete, 'missing row ' .. i)
end
for _, pair in ipairs({{'status=0', 'status=1'}, {'callbacks=1', 'callbacks=2'}, {'hud=1', 'hud=0'}, {'remaining=7.983', 'remaining=0'}, {'input=4', 'input=2'}, {'x=0.000 y=0.000', 'x=20.000 y=0.000'}, {'tick=621', 'tick=501'}}) do
  local changed = text:gsub(pair[1], pair[2])
  assert(not observe(changed, 1).complete, 'contradiction ' .. pair[1])
end
assert(not observe(text .. '\n' .. rows[1], 1).complete)
assert(not observe(text:gsub('tick=1179', 'tick=701'), 1).complete)
assert(not observe(text .. '\nOM_RECALL_SMOKE failed player=1 reason=test tick=1200', 1).complete)
print('Recall observation: positive, roundtrip, missing evidence and contradictions passed')
