-- Fixed acceptance (E323): one human and nine Bots in Unreal, from hero selection to the result
-- screen, within one 10-minute live-match lock. It runs the run_ue.bat demo flow (interactive
-- selection, the human hero picked by --selection-smoke-hero and left idle) on the live-check
-- recipe moba_single_player_quick.lua (three_lane_quick_training), whose waves and structures
-- are tuned so a match finishes inside a live run. The plain run_ue.bat demo keeps the full map:
-- it is the user's own entry and has no time limit.
--
-- Usage: tools/lua/lua.exe scripts/verify_ue_demo_finish.lua [--hero HERO] [--output NEW_DIRECTORY]
--   [--wait-live-lock SECONDS] [--no-build]
-- Writes OUTPUT/demo-finish-report.json; exits 1 when any check fails.
local source = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(source:match('^(.*)[/\\]'))
package.path = dir .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, process, json = b.lib('path'), b.lib('process'), b.lib('json')

local LIMIT_SECONDS = 600
local RECIPE = 'scripts/lua_data/moba_single_player_quick.lua'
local options = { wait = '1800' }
local i = 1
while i <= #arg do
  local v = arg[i]
  if v == '--hero' then options.hero = assert(arg[i + 1], '--hero HERO'); i = i + 1
  elseif v == '--output' then options.output = assert(arg[i + 1], '--output DIR'); i = i + 1
  elseif v == '--wait-live-lock' then options.wait = assert(arg[i + 1], '--wait-live-lock SECONDS'); i = i + 1
  elseif v == '--no-build' then options.no_build = true
  elseif v == '--help' then
    print('Usage: tools/lua/lua.exe scripts/verify_ue_demo_finish.lua [--hero HERO] [--output NEW_DIRECTORY] [--wait-live-lock SECONDS] [--no-build]')
    return
  else error('unknown argument: ' .. v) end
  i = i + 1
end

-- The human seat's recipe hero unless --hero picks another.
if not options.hero then
  local plan = assert(loadfile(path.join(b.root, RECIPE)))()
  for _, player in ipairs(plan.players) do
    if player.bot == false then options.hero = player.hero; break end
  end
  assert(options.hero, 'the recipe has no human seat')
end
local output = options.output and path.absolute(options.output, b.root)
  or path.join(b.root, 'target', 'ue-demo-finish', tostring(os.time()))
assert(not path.exists(output), 'output must be a new directory: ' .. output)

local args = { path.join(dir, 'run_ue.lua'), '--recipe', RECIPE, '--selection-smoke-hero', options.hero,
  '--finish-timeout-seconds', tostring(LIMIT_SECONDS), '--wait-live-lock', options.wait, '--output', output }
if options.no_build then args[#args + 1] = '--no-build' end
local run = process.run(b.lib('platform').lua_executable, args, { cwd = b.root, check = false })

local report = { schema = 'omoba-ue-demo-finish/1', recipe = RECIPE, hero = options.hero, output = output,
  exit_code = run.exit_code, limit_seconds = LIMIT_SECONDS, checks = {} }
local function check(name, ok, detail)
  report.checks[#report.checks + 1] = { name = name, ok = ok and true or false, detail = detail }
end
check('launch exited 0', run.exit_code == 0, 'exit ' .. tostring(run.exit_code))
local observation_file = path.join(output, 'native-result-observation.json')
local observation = path.is_file(observation_file) and json.read(observation_file) or {}
report.winner_team = observation.winner_team
check('result observed in Unreal', observation.complete == true and observation.winner_team ~= nil,
  observation.complete and ('winner team ' .. tostring(observation.winner_team)) or 'no complete result observation')
check('result screenshot', path.is_file(path.join(output, 'result-p1.png')), 'result-p1.png')
-- The whole match, startup included, inside one live-match lock of at most LIMIT_SECONDS (E322).
local acquired, released
local function same(x, y) return x:gsub('\\', '/'):lower() == y:gsub('\\', '/'):lower() end
local history = path.join(b.root, 'target', 'locks', 'live_match.history.jsonl')
if path.is_file(history) then
  for line in path.read(history):gmatch('[^\r\n]+') do
    local ok, row = pcall(json.decode, line)
    if ok and type(row) == 'table' and type(row.run) == 'string' and same(row.run, output) then
      if row.event == 'acquired' then acquired = row.at elseif row.event == 'released' then released = row.at end
    end
  end
end
report.lock_seconds = acquired and released and released - acquired
check('live-match lock held at most ' .. LIMIT_SECONDS .. ' s', report.lock_seconds ~= nil and report.lock_seconds <= LIMIT_SECONDS,
  report.lock_seconds and (report.lock_seconds .. ' s') or 'no acquire/release pair in ' .. history)

report.passed = true
for _, row in ipairs(report.checks) do
  print(('%s %s (%s)'):format(row.ok and 'PASS' or 'FAIL', row.name, tostring(row.detail)))
  report.passed = report.passed and row.ok
end
if path.is_directory(output) then json.write(path.join(output, 'demo-finish-report.json'), report, true) end
print(('ue demo finish: %s, %s'):format(report.passed and 'passed' or 'FAILED', output))
if not report.passed then os.exit(1) end
