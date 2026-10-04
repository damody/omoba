-- Independent saved-run verification; never initiate inputs or restart processes.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = source:match('^(.*)[/\\]tests[/\\]') .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, hash = b.lib('path'), b.lib('json'), b.lib('hash')
assert(#arg == 1, 'usage: ue_renderer_reconnect_acceptance.lua RUN_DIRECTORY')
local run = path.absolute(arg[1])
local report = json.read(path.join(run, 'unreal-ipc-smoke-report.json'))
local restart = json.read(path.join(run, 'renderer-reconnect-report.json'))
assert(report.success and report.cleanup_verified and report.reconnect_smoke
  and report.gameplay_mode == 'single_lane' and report.tick_rate_hz == 60 and restart.success,
  'source run did not pass and clean up')
local roles = {}
for _, owned in ipairs(report.processes) do
  assert(not roles[owned.role], 'role launched twice')
  roles[owned.role] = owned.pid
end
assert(#report.processes == 6 and roles.server == restart.unchanged_pids.server
  and roles['runtime-p1'] == restart.unchanged_pids.runtime_p1
  and roles['runtime-p2'] == restart.unchanged_pids.runtime_p2
  and roles['ue-p2'] == restart.unchanged_pids.ue_p2
  and roles['ue-p1'] == restart.old_pid and roles['ue-p1-reconnect'] == restart.new_pid
  and restart.old_pid ~= restart.new_pid and restart.graceful_only,
  'backend continuity or distinct renderer generations missing')
local old = path.read(path.join(run, 'logs/ue-p1.stdout.log'))
local before_tick = 0
for tick in old:gmatch('OM_MATCH_STATE player=1 phase=1 alive=1[^\n]-tick=(%d+)') do
  before_tick = math.max(before_tick, tonumber(tick))
end
assert(before_tick == restart.before_tick, 'source Playing tick mismatch')
local runtime_log = path.read(path.join(run, 'logs/runtime-p1.stdout.log'))
  .. path.read(path.join(run, 'logs/runtime-p1.stderr.log'))
local sequences = {}
for seq in runtime_log:gmatch('renderer first consumed snapshot player=1 team=1 sequence=(%d+)') do
  sequences[#sequences + 1] = tonumber(seq)
end
assert(#sequences == 2 and sequences[1] == restart.before_consumed_sequence
  and sequences[2] > sequences[1], 'exactly two acknowledged renderer sessions required')
local text = path.read(path.join(run, 'logs/ue-p1-reconnect.stdout.log'))
local helper = require('ue_renderer_reconnect')
local fresh = helper.observe(text, 'renderer first consumed snapshot player=1 team=1 sequence=' .. sequences[2],
  1, 60, before_tick, sequences[1])
assert(fresh.complete and fresh.consumed_sequence == restart.observed.consumed_sequence,
  'fresh-session HUD/economy/input/movement/rate missing')
-- Renderer-local IDs can restart at 1; the unchanged runtime allocator must not.
local forwarded = {}
for id, tick in runtime_log:gmatch('input forwarded player=1 input_id=(%d+) target_tick=(%d+)') do
  forwarded[#forwarded + 1] = {id = tonumber(id), tick = tonumber(tick)}
end
assert(#forwarded == 2 and forwarded[2].id > forwarded[1].id
  and forwarded[2].tick == fresh.input.result.tick, 'new authority input was not allocated after reconnect')
local applied_x, applied_y
for tick, x, y in runtime_log:gmatch('player_input_tick: pid=1 tick=(%d+) MoveTo target_raw=%(([-%d]+), ([-%d]+)%) queued=false') do
  if tonumber(tick) == fresh.input.result.tick then applied_x, applied_y = tonumber(x), tonumber(y) end
end
assert(applied_x and applied_y and math.abs(applied_x / 1024 - fresh.input.click.x) <= 0.01
  and math.abs(applied_y / 1024 - fresh.input.click.y) <= 0.01,
  'new authoritative MoveTo differs from original minimap callback target')
local passed, parity = helper.parity_after(path.read(path.join(run,'server/three-way-checkpoints.jsonl')),
  json.decode, fresh.input.result.tick + 120)
assert(passed, 'missing both-team independent post-input checkpoints')
local saved = {success = true, kind = 'saved-live-unreal-renderer-reconnect', run = run,
  tick_rate_hz = 60, reconnect = restart, fresh_observed = fresh, three_way_parity = parity,
  authoritative_inputs = forwarded, applied_target_raw = {x = applied_x, y = applied_y},
  cleanup_verified = report.cleanup_verified,
  new_stdout_sha256 = hash.sha256(path.join(run,'logs/ue-p1-reconnect.stdout.log')),
  limitations = {'same-host renderer restart only', 'no full cue replay acceptance', 'not 60FPS frame-time acceptance'}}
local destination = path.join(b.root,'openspec/changes/build-unreal-rust-moba-framework/evidence/unreal-renderer-reconnect')
path.mkdir_p(destination)
local output = path.join(destination, assert(run:match('([^/\\]+)$')) .. '.json')
json.write(output, saved, true)
print('saved live Unreal renderer reconnect: PASS; ' .. output)
