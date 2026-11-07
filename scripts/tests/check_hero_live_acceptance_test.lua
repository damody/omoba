-- Fixture tests for scripts/check_hero_live_acceptance.lua: expectations come
-- from packages, evidence from the server log and autopilot coverage/trace.
local source = debug.getinfo(1, 'S').source:sub(2)
local tests_dir = assert(source:match('^(.*)[/\\]'))
package.path = tests_dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path = b.lib('path')
local checker = require('check_hero_live_acceptance')

local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end
local root = path.join(b.root, 'target', 'hero-live-acceptance-tests', tostring(os.time()) .. '-' .. math.random(1, 1e9))

local function package_files(dir, hero, abilities)
  path.write(path.join(dir, 'character.lua'), 'return function(ctx) return { schema_version = 2, hero = { id = "' .. hero .. '" } } end\n', true, true)
  local rows = {}
  for _, a in ipairs(abilities) do
    rows[#rows + 1] = ('{ id = "%s", action = "%s", ability_type = "%s"%s%s },'):format(a.id, a.action, a.type,
      a.auto and ', auto_learn = true' or '', a.level and (', required_hero_level = { ' .. a.level .. ' }') or '')
  end
  -- Same shape as real packages: a local helper using pairs inside the sandbox.
  path.write(path.join(dir, 'abilities.lua'), 'return function(ctx)\n  local n = 0\n  for _ in pairs({ a = 1 }) do n = n + 1 end\n' ..
    '  return { schema_version = 2, abilities = {\n' .. table.concat(rows, '\n') .. '\n} }\nend\n', true, true)
end

local packages = path.join(root, 'packages')
package_files(path.join(packages, 'hero_a'), 'hero_a', {
  { id = 'a_bolt', action = 'Q', type = 'active' },
  { id = 'a_orb', action = 'W', type = 'toggle' },
  { id = 'a_skin', action = 'E', type = 'passive' },
  { id = 'a_ult', action = 'R', type = 'ultimate', level = 6 },
  { id = 'a_late', action = 'extra1', type = 'ultimate', auto = true, level = 16 },
})
package_files(path.join(packages, 'hero_b'), 'hero_b', {
  { id = 'b_aura', action = 'extra1', type = 'passive', auto = true, level = 1 },
  { id = 'b_strike', action = 'Q', type = 'active' },
})

local function run(name, server, p1_coverage, p6_trace, players)
  local dir = path.join(root, name)
  path.write(path.join(dir, 'server', 'match-plan.json'), players or
    '{"players":[{"player_id":1,"team_id":1,"hero":"hero_a","bot":false},{"player_id":2,"team_id":1,"hero":"murakami_yoshikiyo","bot":true},' ..
    '{"player_id":6,"team_id":2,"hero":"hero_b","bot":false}]}', true, true)
  path.write(path.join(dir, 'server', 'server.stdout.log'), server, true, true)
  if p1_coverage then path.write(path.join(dir, 'p1', 'autopilot-p1.coverage.json'), p1_coverage, true, true) end
  if p6_trace then path.write(path.join(dir, 'p6', 'autopilot-p6.jsonl'), p6_trace, true, true) end
  return dir
end

local function by_ability(rows)
  local out = {}
  for _, r in ipairs(rows) do out[r.ability] = r end
  return out
end

local GOOD_SERVER = table.concat({
  "10-26 00:01:00 [INFO] - [scripting] learn ability='b_aura' caster=40 rank=1   (omoba_core::scripting::dispatch 1170)",
  "10-26 00:01:05 [INFO] - [scripting] cast ok ability='a_bolt' caster=12 rank=1   (omoba_core::scripting::dispatch 1150)",
  "10-26 00:01:06 [INFO] - [scripting] cast ok ability='a_orb' caster=12 rank=1",
  "10-26 00:01:07 [INFO] - [scripting] learn ability='a_skin' caster=12 rank=1",
  "10-26 00:03:00 [INFO] - [scripting] learn ability='a_ult' caster=12 rank=1",
  "10-26 00:03:02 [INFO] - [scripting] cast ok ability='a_ult' caster=12 rank=1",
  "10-26 00:03:04 [INFO] - [scripting] cast ok ability='b_strike' caster=40 rank=1",
}, '\n') .. '\n'
local GOOD_COVERAGE = '{"schema_version":1,"intents":{"cast_q_target":{"ack_accepted":3},"cast_w_toggle":{"ack_accepted":1},' ..
  '"cast_r_instant":{"ack_accepted":2},"upgrade_e":{"ack_accepted":1},"cast_e_target":{"ack_accepted":0}}}'
local GOOD_TRACE = '{"event":"start"}\n' ..
  '{"event":"authority","tick":10,"autopilot_intent":"cast_q_target","request_id":3,"ack_result":"accepted"}\n' ..
  '{"event":"authority","tick":12,"autopilot_intent":"cast_q_target","request_id":4,"ack_result":"rejected"}\n'

test('every ability passes with server and autopilot evidence; level-16 auto_learn is not reachable', function()
  local ok, rows = checker.check(run('good', GOOD_SERVER, GOOD_COVERAGE, GOOD_TRACE), { packages = packages })
  local r = by_ability(rows)
  for _, id in ipairs({ 'a_bolt', 'a_orb', 'a_skin', 'a_ult', 'b_aura', 'b_strike' }) do assert(r[id].status == 'PASS', id .. ': ' .. r[id].detail) end
  assert(r.a_late.status == 'NOT_REACHABLE_IN_10MIN', r.a_late.status)
  for _, row in ipairs(rows) do assert(row.player ~= 2, 'bot players are not checked') end
  assert(ok, 'overall PASS')
end)

test('a cast without an accepted autopilot intent, or a passive never learned, fails', function()
  local server = GOOD_SERVER:gsub("[^\n]*learn ability='a_skin'[^\n]*\n", '')
  local coverage = GOOD_COVERAGE:gsub('"cast_w_toggle":{"ack_accepted":1}', '"cast_w_toggle":{"ack_accepted":0}')
  local ok, rows = checker.check(run('missing', server, coverage, GOOD_TRACE), { packages = packages })
  local r = by_ability(rows)
  assert(r.a_skin.status == 'FAIL' and r.a_orb.status == 'FAIL' and r.a_bolt.status == 'PASS')
  assert(not ok)
end)

test('a reached late auto_learn is checked like any ability; run-wide gates fail the run', function()
  local server = GOOD_SERVER .. "[scripting] learn ability='a_late' caster=12 rank=1\n"
  local ok, rows = checker.check(run('late', server, GOOD_COVERAGE, GOOD_TRACE), { packages = packages })
  assert(by_ability(rows).a_late.status == 'FAIL', 'learned but never cast')
  assert(not ok)
  ok, rows = checker.check(run('late-low', GOOD_SERVER, GOOD_COVERAGE, GOOD_TRACE), { packages = packages, reachable_level = 20 })
  assert(by_ability(rows).a_late.status == 'FAIL', 'within the reachable level it is required')
  ok, rows = checker.check(run('mismatch', GOOD_SERVER .. 'observer mismatch at tick 99\n', GOOD_COVERAGE, GOOD_TRACE), { packages = packages })
  assert(not ok and by_ability(rows)['run:no-observer-mismatch'].status == 'FAIL')
end)

test('a human hero without a package is skipped, not failed', function()
  local plan = '{"players":[{"player_id":1,"team_id":1,"hero":"murakami_yoshikiyo","bot":false},{"player_id":6,"team_id":2,"hero":"hero_b","bot":false}]}'
  local ok, rows = checker.check(run('unauthored', GOOD_SERVER, nil, GOOD_TRACE, plan), { packages = packages })
  assert(rows[1].status == 'SKIP' and rows[1].hero == 'murakami_yoshikiyo')
  assert(ok, 'hero_b still passes')
end)

test('trace fallback counts accepted acks only; a missing plan fails', function()
  local casts, where = checker.accepted_casts(path.join(root, 'good', 'p6'), 6)
  assert(where == 'trace' and casts.q == 1, 'one accepted, one rejected')
  local ok, rows = checker.check(path.join(root, 'nowhere'), { packages = packages })
  assert(not ok and rows[1].status == 'FAIL' and rows[1].ability == 'match-plan')
end)

test('an ultimate the match ended too early to reach is inconclusive, not failed; a reached one still fails', function()
  local server = GOOD_SERVER:gsub("[^\n]*a_ult[^\n]*\n", '')
  local short = run('short', server, GOOD_COVERAGE, GOOD_TRACE)
  path.write(path.join(short, 'p1', 'autopilot-p1.jsonl'), '{"event":"match_finished","tick":11442,"winner_team":2,"local_team":1,"hero_level":4}\n', true, true)
  local ok, rows, inconclusive = checker.check(short, { packages = packages })
  local r = by_ability(rows)
  assert(ok and inconclusive and r.a_ult.status == 'INCONCLUSIVE_RUN_SHORT', 'level 4 cannot reach a level-6 rank')
  assert(r.a_bolt.status == 'PASS' and r.a_skin.status == 'PASS', 'reachable abilities are still judged')
  local reached = run('reached', server, GOOD_COVERAGE, GOOD_TRACE)
  path.write(path.join(reached, 'p1', 'autopilot-p1.jsonl'), '{"event":"match_finished","tick":30000,"winner_team":2,"local_team":1,"hero_level":7}\n', true, true)
  ok, rows, inconclusive = checker.check(reached, { packages = packages })
  assert(not ok and not inconclusive and by_ability(rows).a_ult.status == 'FAIL', 'a reachable unlearned ultimate fails')
  ok, rows, inconclusive = checker.check(run('unknown-level', server, GOOD_COVERAGE, GOOD_TRACE), { packages = packages })
  assert(not ok and by_ability(rows).a_ult.status == 'FAIL', 'an unknown final level keeps the strict verdict')
end)

print(count .. ' hero live acceptance checker tests passed')
