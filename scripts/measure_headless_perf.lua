-- Controlled headless performance comparison (E328): release moba-headless builds measured on one
-- role plan, interleaved rounds, step_wall_ms_mean per run and the median per build.
--
--   tools/lua/lua.exe scripts/measure_headless_perf.lua --output NEW_DIR --role-plan PLAN.json
--     --build NAME=RUN_DIR (repeatable; RUN_DIR holds artifacts/scripts, and EXE defaults to
--     RUN_DIR-cargo/release/moba-headless.exe; NAME=RUN_DIR=EXE names it)
--     [--rounds 5] [--game-seconds 600] [--wait-seconds 3600]
--
-- The performance lock is held for the whole comparison, so no other measurement overlaps. The
-- live_match lock is taken per run, not for the whole comparison: each run holds it at most
-- LIVE_LIMIT_SECONDS from taking it (E322) and is stopped past that, and live matches can run
-- between runs. A comparison of 10 runs used to hold live_match for 20 minutes.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, process, json, time = b.lib('path'), b.lib('process'), b.lib('json'), b.lib('time')
local machine_lock = require('machine_lock')

local M = {LIVE_LIMIT_SECONDS = 600}

function M.options(argv)
  local o = {builds = {}, rounds = 5, game_seconds = 600, wait_seconds = 3600}
  local i = 1
  while i <= #argv do
    local v, value = argv[i], argv[i + 1]
    if v == '--output' then o.output = assert(value, '--output DIR')
    elseif v == '--role-plan' then o.role_plan = assert(value, '--role-plan FILE')
    elseif v == '--build' then
      local name, dir, exe = assert(value, '--build NAME=RUN_DIR[=EXE]'):match('^([%w_%-]+)=([^=]+)=?(.*)$')
      assert(name and dir, '--build NAME=RUN_DIR[=EXE]')
      o.builds[#o.builds + 1] = {name = name, dir = dir, exe = exe ~= '' and exe or (dir .. '-cargo/release/moba-headless.exe')}
    elseif v == '--rounds' then o.rounds = assert(math.tointeger(tonumber(value)), '--rounds N')
    elseif v == '--game-seconds' then o.game_seconds = assert(math.tointeger(tonumber(value)), '--game-seconds S')
    elseif v == '--wait-seconds' then o.wait_seconds = assert(math.tointeger(tonumber(value)), '--wait-seconds S')
    else error('unknown argument: ' .. tostring(v)) end
    i = i + 2
  end
  assert(o.output and o.role_plan and #o.builds >= 1, 'needs --output, --role-plan and at least one --build')
  assert(o.rounds >= 1 and o.game_seconds >= 1, 'rounds and game seconds must be positive')
  return o
end

function M.median(list)
  if #list == 0 then return nil end
  local sorted = {}
  for i, v in ipairs(list) do sorted[i] = v end
  table.sort(sorted)
  local n = #sorted
  return n % 2 == 1 and sorted[(n + 1) // 2] or (sorted[n // 2] + sorted[n // 2 + 1]) / 2
end

-- One run under its own live_match lock; stopped LIVE_LIMIT_SECONDS after the lock was taken.
local function run_once(o, build, round, output)
  local report = path.join(output, ('%s-%d.json'):format(build.name, round))
  local log = path.join(output, ('%s-%d.log'):format(build.name, round))
  local live <close> = machine_lock.acquire(b.root, 'live_match', {run = output,
    label = 'measure_headless_perf ' .. build.name .. ' ' .. round, wait_seconds = o.wait_seconds})
  local deadline = time.monotonic_ms() + M.LIVE_LIMIT_SECONDS * 1000
  local _, record = process.spawn_owned(path.absolute(build.exe, b.root), {'--scripts-dir', 'artifacts/scripts',
    '--role-plan', path.absolute(o.role_plan, b.root), '--report', report,
    '--max-game-seconds', tostring(o.game_seconds), '--stall-game-seconds', tostring(o.game_seconds)},
    {cwd = path.absolute(build.dir, b.root), stdout = log, stderr = log .. '.err',
     env = require('moba_compiled_content').environment({})})
  live:attach(record, 'moba-headless')
  local timed_out = false
  while not process.wait_owned(record, 1000) do
    if time.monotonic_ms() >= deadline then
      process.stop_owned(record); timed_out = true; break
    end
  end
  local text = path.is_file(report) and path.read(report) or ''
  return {build = build.name, round = round, timed_out = timed_out,
    step_wall_ms_mean = tonumber(text:match('"step_wall_ms_mean"%s*:%s*([%d%.]+)')),
    observed_tick = tonumber(text:match('"observed_tick"%s*:%s*(%d+)'))}
end

function M.main(argv)
  local o = M.options(argv)
  local output = path.absolute(o.output, b.root)
  assert(not path.exists(output), 'output must be a new directory: ' .. output)
  for _, build in ipairs(o.builds) do
    assert(path.is_file(path.absolute(build.exe, b.root)), 'missing moba-headless: ' .. build.exe)
    assert(path.is_directory(path.join(path.absolute(build.dir, b.root), 'artifacts', 'scripts')), 'missing artifacts/scripts in ' .. build.dir)
  end
  path.mkdir_p(output)
  local performance <close> = machine_lock.acquire(b.root, 'performance', {run = output,
    label = 'measure_headless_perf', grace_seconds = 3 * 3600, wait_seconds = o.wait_seconds})
  local rows, per_build = {}, {}
  for round = 1, o.rounds do
    for _, build in ipairs(o.builds) do
      local row = run_once(o, build, round, output)
      rows[#rows + 1] = row
      if row.step_wall_ms_mean and not row.timed_out then
        per_build[build.name] = per_build[build.name] or {}
        table.insert(per_build[build.name], row.step_wall_ms_mean)
      end
      print(('%s round %d: step_wall_ms_mean=%s ticks=%s%s'):format(build.name, round, tostring(row.step_wall_ms_mean),
        tostring(row.observed_tick), row.timed_out and ' STOPPED at the live-match limit' or ''))
      io.stdout:flush()
    end
  end
  local summary = {schema = 'omoba-headless-perf/1', role_plan = o.role_plan, rounds = o.rounds,
    game_seconds = o.game_seconds, runs = rows, medians = {}}
  for _, build in ipairs(o.builds) do
    summary.medians[build.name] = M.median(per_build[build.name] or {})
    print(('median %s: %s ms'):format(build.name, tostring(summary.medians[build.name])))
  end
  json.write(path.join(output, 'summary.json'), summary, true)
  return summary
end

if not pcall(debug.getlocal, 4, 1) then M.main(arg) end
return M
