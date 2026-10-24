-- Audit official Unreal reports. A successful process exit is not test success.
local M = {}

local function names(list, label)
  assert(type(list) == 'table' and #list > 0, label .. ' must not be empty')
  local result = {}
  for _, name in ipairs(list) do
    assert(type(name) == 'string' and name ~= '' and not result[name], label .. ': invalid or duplicate name')
    result[name] = true
  end
  return result
end

function M.test_names(source)
  if source:find('OMOBA_OWNED_VIEWPORT_IPC_FILE', 1, true) then return {} end
  local result = {}
  for name in source:gmatch('IMPLEMENT_SIMPLE_AUTOMATION_TEST%s*%(%s*[%w_]+%s*,%s*"([^"]+)"') do
    if name:match('^Om%.Generated%.') or name:match('^Om%.Runtime%.')
        or name == 'Om.Playable.ProjectDefaults' or name == 'Om.Playable.RustBlueprintAssetLoad'
        or name:match('^UECP%.MCP%.') or name:match('^UECP%.Tools%.PIE%.PlaySessionPlan') then
      assert(name:match('^[%w_.]+$'), 'unsupported automation test name: ' .. name)
      result[#result + 1] = name
    end
  end
  return result
end

function M.owned_viewport_test_names(source)
  assert(type(source) == 'string' and source:find('OMOBA_OWNED_VIEWPORT_IPC_FILE', 1, true),
    'owned viewport file marker is missing')
  local result = {}
  for name in source:gmatch('IMPLEMENT_SIMPLE_AUTOMATION_TEST%s*%(%s*[%w_]+%s*,%s*"([^"]+)"') do
    assert(name:match('^Om%.Runtime%.'), 'owned viewport file contains a non-runtime test: ' .. name)
    assert(name:match('^[%w_.]+$'), 'unsupported automation test name: ' .. name)
    result[#result + 1] = name
  end
  assert(#result > 0, 'owned viewport file has no tests')
  return result
end

function M.require_owned_viewport_phase(report, expected)
  local wanted = names(expected or {'Om.Runtime.OwnedViewportIpc'}, 'expected owned viewport tests')
  assert(type(report) == 'table', 'owned viewport phase was not recorded')
  assert(report.executed == true, 'owned viewport phase was not executed')
  assert(report.success == true, 'owned viewport phase failed')
  assert(report.not_run == 0, 'owned viewport phase counted an unrun test as accepted')
  assert(report.failed == 0, 'owned viewport phase has failed tests')
  if report.execution_mode == 'phased' then require('ue_owned_phase_coverage').verify_report(report)
  else assert(report.phase == nil or report.phase == 'full', 'a feature phase cannot satisfy full acceptance') end
  assert(type(report.tests) == 'table', 'owned viewport phase has no test list')
  local seen = {}
  for _, test in ipairs(report.tests) do
    local name = test.name
    assert(wanted[name] and not seen[name], 'unexpected or duplicate owned viewport test: ' .. tostring(name))
    seen[name] = true
    assert(test.state == 'Success' and test.errors == 0, 'owned viewport test failed or incomplete: ' .. tostring(name))
  end
  for name in pairs(wanted) do assert(seen[name], 'missing owned viewport test: ' .. name) end
end

function M.native(report, expected)
  local wanted = names(expected, 'expected native tests')
  assert(type(report) == 'table' and type(report.tests) == 'table', 'missing native test report')
  local seen, count, warning_tests, warnings = {}, 0, 0, 0
  for _, test in ipairs(report.tests) do
    local name = test.fullTestPath
    assert(wanted[name] and not seen[name], 'unexpected or duplicate native test: ' .. tostring(name))
    seen[name], count = true, count + 1
    assert(test.state == 'Success', 'native test failed or incomplete: ' .. name .. ' (' .. tostring(test.state) .. ')')
    assert(math.type(test.errors) == 'integer' and test.errors == 0, 'native test errors: ' .. name)
    assert(math.type(test.warnings) == 'integer' and test.warnings >= 0, 'invalid native warning count: ' .. name)
    warnings = warnings + test.warnings
    if test.warnings > 0 then warning_tests = warning_tests + 1 end
  end
  for name in pairs(wanted) do assert(seen[name], 'missing native test: ' .. name) end
  for _, field in ipairs({'failed', 'notRun', 'inProcess'}) do
    assert(report[field] == 0, 'native report ' .. field .. ' is not zero')
  end
  assert(report.succeeded == count - warning_tests and report.succeededWithWarnings == warning_tests,
    'native report counters do not match individual tests')
  return {count = count, succeeded = count - warning_tests, succeeded_with_warnings = warning_tests,
    warnings = warnings, failed = 0, not_run = 0}
end

function M.native_json(bytes, expected)
  -- Unreal's JSON exporter emits a UTF-8 BOM. Keep the original evidence intact.
  if bytes:sub(1, 3) == '\239\187\191' then bytes = bytes:sub(4) end
  return M.native(require('tools.lua.lib.json').decode(bytes), expected)
end

-- Keep exact-name command lines bounded as the native suite grows. Each batch
-- and the final union still go through the same strict report checker.
function M.native_batches(expected, max_count, max_bytes)
  names(expected, 'expected native tests')
  max_count,max_bytes=max_count or 32,max_bytes or 4096
  assert(math.type(max_count)=='integer' and max_count>=1 and max_count<=256, 'invalid native batch count')
  assert(math.type(max_bytes)=='integer' and max_bytes>=1 and max_bytes<=16000, 'invalid native batch bytes')
  local result,current,bytes={}, {},0
  for _,name in ipairs(expected)do
    assert(name:match('^[%w_.]+$') and #name<=max_bytes,'unsupported or oversized native test name')
    local size=#name+(#current>0 and 1 or 0)
    if #current>=max_count or bytes+size>max_bytes then
      result[#result+1]=current;current={};bytes=0;size=#name
    end
    current[#current+1]=name;bytes=bytes+size
  end
  if #current>0 then result[#result+1]=current end
  return result
end

-- Device-dependent tests retain their place in the exact union while running
-- through the workflow that verifies their actual device output.
function M.native_execution_batches(expected, max_count, max_bytes)
  M.native_batches(expected, max_count, max_bytes) -- Validate the complete input first.
  local ordinary, audio = {}, {}
  for _, name in ipairs(expected) do
    local list = name == 'Om.Runtime.OriginalAudioOutput' and audio or ordinary
    list[#list + 1] = name
  end
  local result = {}
  if #ordinary > 0 then
    for _, batch in ipairs(M.native_batches(ordinary, max_count, max_bytes)) do
      result[#result + 1] = {profile = 'native', tests = batch}
    end
  end
  if #audio > 0 then result[#result + 1] = {profile = 'audio-output', tests = audio} end
  return result
end

function M.native_command_file(expected, report_path, log_path)
  M.native_batches(expected) -- Reject delimiters, duplicates and unsupported names.
  local function quoted(value)
    assert(type(value)=='string' and #value>0 and not value:find('["\r\n]'), 'invalid command file path')
    return '"'..value..'"'
  end
  local exact={}
  for _,name in ipairs(expected)do exact[#exact+1]='^'..name..'$'end
  return '-ExecCmds='..quoted('Automation RunTests '..table.concat(exact,'+'))
    ..' -ReportExportPath='..quoted(report_path)..' -abslog='..quoted(log_path)..'\n'
end

function M.blueprints(log, expected)
  local wanted = names(expected, 'expected Blueprint assets')
  local seen, count = {}, 0
  for asset in log:gmatch("Loading and Compiling: '([^']+)'") do
    assert(wanted[asset] and not seen[asset], 'unexpected or duplicate Blueprint asset: ' .. asset)
    seen[asset], count = true, count + 1
  end
  for asset in pairs(wanted) do assert(seen[asset], 'missing Blueprint asset: ' .. asset) end
  local errors, warnings, failed_loads = log:match('Compiling Completed with (%d+) errors and (%d+) warnings and (%d+) blueprints that failed to load')
  assert(errors, 'missing Blueprint commandlet completion summary')
  errors, warnings, failed_loads = tonumber(errors), tonumber(warnings), tonumber(failed_loads)
  assert(errors == 0 and warnings == 0 and failed_loads == 0, 'Blueprint commandlet compiler or load diagnostics')
  return {count = count, compiler_errors = errors, compiler_warnings = warnings, failed_loads = failed_loads}
end

return M
