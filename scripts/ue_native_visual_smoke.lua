-- Native art playback plus legacy Blueprint compatibility acceptance.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(script:match('^(.*)[/\\]'))
package.path = dir .. '/?.lua;' .. package.path
local bootstrap = require('_bootstrap')
local path, json = bootstrap.lib('path'), bootstrap.lib('json')
local process, platform = bootstrap.lib('process'), bootstrap.lib('platform')
local time = bootstrap.lib('time')
local work = path.join(bootstrap.root, 'omfue/Saved/McpAutomation/NativeVisual')
path.mkdir_p(work)
local report = {success = false, results = {}, runs = {}, tests = {
  'Om.Generated.NativeHeroPresentation', 'Om.Generated.AnimationStateSmoke',
  'Om.Generated.SaikaEventDispatch', 'Om.Generated.BlueprintSurface',
  'Om.Generated.ProjectileCueHistory', 'Om.Generated.WorldBridgeSyntheticFrameSmoke',
  'Om.Generated.RememberedGhostPresentation',
  'Om.Runtime.UiOverlaySurface',
  'Om.Runtime.GameplayInputSurface',
  'Om.Runtime.NativeAbilityProgression',
  'Om.Runtime.NativeAbilityUpgradeBinding',
  'Om.Runtime.RenderedUiCaptureOptIn',
  'Om.Runtime.NativeShopInput',
  'Om.Runtime.NativeMinimap',
  'Om.Runtime.NativeMinimapInput',
  'Om.Runtime.NativeMatchResult',
  'Om.Runtime.NativeOwnerScore',
  'Om.Runtime.NativeScoreboard',
  'Om.Runtime.MatchSmokeObjective',
}}
local ordinal = 0
local function call(name, arguments)
  ordinal = ordinal + 1
  local file = path.join(work, string.format('%03d-%s.json', ordinal, name))
  local result = process.run(platform.lua_executable, {path.join(bootstrap.root, 'scripts/ue_mcp.lua'),
    '--tool', name, '--arguments', json.encode(json.object(arguments)), '--out', file},
    {cwd = bootstrap.root, check = false})
  assert(result.exit_code == 0, name .. ' failed: ' .. result.stderr .. result.stdout)
  for _, content in ipairs(json.read(file).content or {}) do
    if content.type == 'text' then
      local parsed, value = pcall(json.decode, content.text)
      if parsed and type(value) == 'table' then report.results[#report.results + 1] = {tool = name, value = value}; return value end
    end
  end
  error(name .. ' returned no structured result')
end
local ok, failure = xpcall(function()
  assert(not call('get_editor_dialog', {}).dialog_open, 'Editor modal blocks acceptance; inspect before choosing an answer')
  assert(not call('get_pie_status', {}).pie_running, 'stop PIE before Editor automation')
  call('enable_extension', {extension_id = 'Testing', enabled = true})
  local discovered
  for _ = 1, 20 do
    discovered = call('list_automation_tests', {filter = 'Om.', limit = 100})
    if discovered.ready then break end
    time.sleep_ms(250)
  end
  assert(discovered.ready, 'Editor test discovery did not finish')
  local available = {}
  for _, test in ipairs(discovered.tests or {}) do available[test.full_path] = true end
  for _, wanted in ipairs(report.tests) do assert(available[wanted], 'required test missing: ' .. wanted) end
  local prior = call('get_automation_test_results', {})
  assert((prior.running or 0) == 0, 'another Editor automation run is active')
  -- Repeat in the same Editor session: a single run cannot catch transient
  -- UObject naming collisions or state leaked by a previous automation run.
  for run = 1, 2 do
    local started = call('run_automation_tests', {tests = report.tests})
    assert(started.started and started.count == #report.tests, 'required tests were not all started')
    local result
    for _ = 1, 60 do
      result = call('get_automation_test_results', {})
      if result.complete then break end
      time.sleep_ms(250)
    end
    assert(result.complete, 'Editor tests did not finish in bounded polling window')
    assert(result.total == #report.tests and result.passed == #report.tests and result.failed == 0 and
      result.skipped == 0 and result.not_run == 0, 'native/legacy Editor test failure: ' .. json.encode(result))
    local passed = {}
    for _, test in ipairs(result.results) do
      assert(test.state == 'passed' and test.error_count == 0, 'test failed: ' .. test.full_path)
      passed[test.full_path] = true
    end
    for _, wanted in ipairs(report.tests) do assert(passed[wanted], 'expected test result missing: ' .. wanted) end
    report.runs[run] = result
  end
  report.success = true
end, debug.traceback)
if not ok then report.error = tostring(failure) end
json.write(path.join(work, 'report.json'), report, true)
print('[ue-native-visual] success=' .. tostring(report.success) .. '; report: ' .. path.join(work, 'report.json'))
assert(ok, failure)
