local source = debug.getinfo(1, 'S').source:sub(2)
package.path = source:match('^(.*)[/\\]tests[/\\]') .. '/?.lua;' .. package.path
local test = require('ue_renderer_reconnect')
local text = table.concat({
  'OM_MATCH_STATE player=1 phase=1 alive=1 winner=0 hero=5 epoch=1 respawn=0 tick=300',
  'OM_MOBA_HUD player=1 phase=1 alive=1 winner=0 abilities=4 hp=100/100 rate=60',
  'OM_OWNER_ECONOMY player=1 gold=10',
  'OM_PRESENTATION seq=100 heroes=1 [5 k=1 o=1 c=1 (1,2)]',
  'OM_PRESENTATION seq=101 heroes=1 [5 k=1 o=1 c=1 (2,3)]',
  'OM_MINIMAP_INPUT player=1 handled=1 callbacks=1 input=9 accepted=1 target=(-500,-240)',
  'OM_MINIMAP_SMOKE result player=1 input=9 status=0 tick=330',
}, '\n')
local ack = 'renderer first consumed snapshot player=1 team=1 sequence=100'
local function observe(t, a) return test.observe(t or text, a or ack, 1, 60, 120, 40).complete end
assert(observe())
assert(not observe((text:gsub('rate=60','rate=120'))))
assert(not observe((text:gsub('rate=60','rate=0'))))
assert(not observe((text:gsub(' rate=60',''))))
assert(not observe((text:gsub('tick=300','tick=120'))))
assert(not observe(nil, ack:gsub('sequence=100','sequence=40')))
assert(not observe(nil, ack:gsub('player=1','player=2')))
assert(not observe((text:gsub('OM_OWNER_ECONOMY player=1','OM_OWNER_ECONOMY player=2'))))
assert(not observe((text:gsub('input=9 status=0','input=8 status=0'))))
assert(not observe((text:gsub('status=0','status=4'))))
assert(not observe((text:gsub('o=1','o=2'))))
assert(not observe((text:gsub('%(2,3%)','(1,2)'))))
assert(not observe(text .. '\nOM_MOBA_HUD player=1 rate=120'))
local function row(team, tick, verdict)
  return {verdict = verdict or 'PASS', team_id = team, replica_tick = tick,
    pre_repair_parity = true, post_repair_parity = true, expected = 'h',
    external_runtime_pre_repair = 'h', external_runtime_post_repair = 'h',
    external_runtime_frame_hash = 'f', observer_frame_hash = 'f'}
end
local rows = {row(1,120),row(2,120),row(1,480),row(2,480),row(1,600),row(2,600)}
local function decode(s) return rows[tonumber(s)] end
assert(test.parity_after('1\n2\n3\n4\n5\n6\n', decode, 450))
assert(not test.parity_after('1\n2\n3\n4\n3\n4\n', decode, 450)) -- duplicate tick is not two checkpoints
assert(not test.parity_after('1\n2\n3\n4\n5\n', decode, 450)) -- both teams required
rows[6] = row(2,600,'FAIL')
assert(not pcall(test.parity_after,'3\n4\n5\n6\n',decode,450))
print('renderer reconnect observation: 17 scenarios passed')
