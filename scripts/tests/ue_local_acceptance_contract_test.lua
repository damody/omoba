local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/../?.lua;' .. package.path
require('_bootstrap')
local contract = require('ue_local_acceptance_contract')
local json = require('tools.lua.lib.json')
local expected = {'Om.Generated.FrameContract', 'Om.Runtime.NativeShopInput'}
local original = {succeeded = 1, succeededWithWarnings = 1, failed = 0, notRun = 0, inProcess = 0,
  tests = {{fullTestPath = expected[1], state = 'Success', warnings = 0, errors = 0},
    {fullTestPath = expected[2], state = 'Success', warnings = 2, errors = 0}}}
local count = 0
local function reject(name, mutate, needle)
  local report = json.decode(json.encode(original))
  mutate(report)
  local ok, err = pcall(contract.native, report, expected)
  assert(not ok and tostring(err):find(needle, 1, true), name .. ': ' .. tostring(err))
  count = count + 1
end
local accepted = contract.native(original, expected)
assert(accepted.count == 2 and accepted.succeeded == 1 and accepted.succeeded_with_warnings == 1 and accepted.warnings == 2)
count = count + 1
local batches=contract.native_batches(expected,1,4096)
assert(#batches==2 and #batches[1]==1 and batches[1][1]==expected[1] and batches[2][1]==expected[2])
assert(#contract.native_batches(expected,32,#expected[1]+#expected[2])==2,'separator must count against byte budget')
assert(#contract.native_batches(expected,32,#expected[1]+#expected[2]+1)==1)
assert(not pcall(contract.native_batches,{expected[1],expected[1]}),'duplicate native batch names accepted')
assert(not pcall(contract.native_batches,{},32,4096),'empty suite accepted')
assert(not pcall(contract.native_batches,expected,0,4096),'zero batch count accepted')
assert(not pcall(contract.native_batches,expected,32,0),'zero byte budget accepted')
assert(not pcall(contract.native_batches,{'Om.Runtime.Bad+Name'},32,4096),'command delimiter accepted')
assert(not pcall(contract.native_batches,expected,32,4),'oversized single name accepted')
local many={};for i=1,169 do many[i]='Om.Runtime.Bounded'..i end
local seen={};local total=0
for _,batch in ipairs(contract.native_batches(many))do
 assert(#batch<=32 and #table.concat(batch,'+')<=4096)
 for _,name in ipairs(batch)do assert(not seen[name]);seen[name]=true;total=total+1 end
end
assert(total==169)
count=count+10
assert(contract.native_json('\239\187\191' .. json.encode(original), expected).count == 2)
assert(contract.native_json(json.encode(original), expected).warnings == 2)
assert(not pcall(contract.native_json, '\255\254' .. json.encode(original), expected))
count = count + 3
reject('zero-test exit is not acceptance', function(r) r.tests = {} end, 'missing native test')
reject('missing test', function(r) table.remove(r.tests) end, 'missing native test')
reject('extra test', function(r) r.tests[2].fullTestPath = 'Om.Playable.LinkedSinglePlayerLockstepSmoke' end, 'unexpected')
reject('duplicate', function(r) r.tests[2].fullTestPath = expected[1] end, 'duplicate')
reject('failed state despite success counters', function(r) r.tests[2].state = 'Fail' end, 'failed or incomplete')
reject('not run state', function(r) r.tests[2].state = 'NotRun' end, 'failed or incomplete')
reject('error count', function(r) r.tests[2].errors = 1 end, 'native test errors')
reject('missing warning count', function(r) r.tests[2].warnings = nil end, 'invalid native warning count')
reject('warning counter mismatch', function(r) r.succeededWithWarnings = 0 end, 'counters')
reject('success counter mismatch', function(r) r.succeeded = 2 end, 'counters')
for _, field in ipairs({'failed', 'notRun', 'inProcess'}) do
  reject(field, function(r) r[field] = 1 end, field)
end

local assets = {'/Game/RustBP/Heroes/BP_Hero.BP_Hero', '/Game/RustBP/UI/WBP_Hud.WBP_Hud'}
local loading = "Loading and Compiling: '" .. assets[1] .. "'\nLoading and Compiling: '" .. assets[2] .. "'\n"
local completion = 'Compiling Completed with 0 errors and 0 warnings and 0 blueprints that failed to load'
assert(contract.blueprints(loading .. completion, assets).count == 2)
count = count + 1
local function reject_assets(label, log, needle)
  local ok, err = pcall(contract.blueprints, log, assets)
  assert(not ok and tostring(err):find(needle, 1, true), label .. ': ' .. tostring(err))
  count = count + 1
end
reject_assets('missing load', completion, 'missing Blueprint')
reject_assets('duplicate load', loading .. loading .. completion, 'duplicate')
reject_assets('unknown asset', loading .. "Loading and Compiling: '/Game/Foreign.Foreign'\n" .. completion, 'unexpected')
reject_assets('unfinished commandlet', loading, 'completion summary')
reject_assets('compiler errors', loading .. completion:gsub('0 errors', '46 errors'), 'diagnostics')
reject_assets('compiler warnings', loading .. completion:gsub('0 warnings', '1 warnings'), 'diagnostics')
reject_assets('failed loads', loading .. completion:gsub('0 blueprints', '1 blueprints'), 'diagnostics')

local discovered = contract.test_names([[
IMPLEMENT_SIMPLE_AUTOMATION_TEST(FFrame,
  "Om.Generated.FrameContract", Flags)
IMPLEMENT_SIMPLE_AUTOMATION_TEST(FAsset, "Om.Playable.RustBlueprintAssetLoad", Flags)
IMPLEMENT_SIMPLE_AUTOMATION_TEST(FSimulation, "Om.Playable.LinkedSinglePlayerLockstepSmoke", Flags)
IMPLEMENT_SIMPLE_AUTOMATION_TEST(FForeign, "Foreign.Test", Flags)
]])
assert(#discovered == 2 and discovered[1] == expected[1] and discovered[2] == 'Om.Playable.RustBlueprintAssetLoad')
count = count + 1

local b = require('_bootstrap')
local path = b.lib('path')
local ipc_source = [[
// OMOBA_OWNED_VIEWPORT_IPC_FILE
IMPLEMENT_SIMPLE_AUTOMATION_TEST(FOmOwnedViewportIpcTest, "Om.Runtime.OwnedViewportIpc",
  EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
]]
assert(#contract.test_names(ipc_source) == 0, 'the dedicated IPC file must stay out of the disconnected suite')
local ipc_names = contract.owned_viewport_test_names(ipc_source)
assert(#ipc_names == 1 and ipc_names[1] == 'Om.Runtime.OwnedViewportIpc')
count = count + 2
local function reject_ipc_names(label, source_text, needle)
  local ok, err = pcall(contract.owned_viewport_test_names, source_text)
  assert(not ok and tostring(err):find(needle, 1, true), label .. ': ' .. tostring(err))
  count = count + 1
end
reject_ipc_names('missing marker', ipc_source:gsub('// OMOBA_OWNED_VIEWPORT_IPC_FILE\n', ''), 'marker is missing')
reject_ipc_names('foreign name', ipc_source .. '\nIMPLEMENT_SIMPLE_AUTOMATION_TEST(FOther, "Om.Playable.Nope", Flags)\n', 'non-runtime')
local cpp = path.read(path.join(b.root, 'omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmOwnedViewportIpcTests.cpp'))
assert(#contract.test_names(cpp) == 0, 'the real IPC file leaked into the disconnected suite')
local real_names = contract.owned_viewport_test_names(cpp)
assert(#real_names == 1 and real_names[1] == 'Om.Runtime.OwnedViewportIpc')
count = count + 2

local phase = {executed = true, success = true, not_run = 0, failed = 0,
  tests = {{name = 'Om.Runtime.OwnedViewportIpc', state = 'Success', errors = 0}}}
contract.require_owned_viewport_phase(phase, {'Om.Runtime.OwnedViewportIpc'})
contract.require_owned_viewport_phase(json.decode(json.encode(phase)))
count = count + 2
local function reject_phase(label, mutate, needle)
  local report = json.decode(json.encode(phase))
  mutate(report)
  local ok, err = pcall(contract.require_owned_viewport_phase, report, {'Om.Runtime.OwnedViewportIpc'})
  assert(not ok and tostring(err):find(needle, 1, true), label .. ': ' .. tostring(err))
  count = count + 1
end
reject_phase('not executed', function(r) r.executed = false end, 'not executed')
reject_phase('failed phase', function(r) r.success = false end, 'phase failed')
reject_phase('unrun counted as accepted', function(r) r.not_run = 1 end, 'unrun')
reject_phase('failed test counter', function(r) r.failed = 1 end, 'failed tests')
reject_phase('wrong test name', function(r) r.tests[1].name = 'Om.Runtime.Other' end, 'unexpected')
reject_phase('incomplete state', function(r) r.tests[1].state = 'Fail' end, 'failed or incomplete')
reject_phase('error count', function(r) r.tests[1].errors = 1 end, 'failed or incomplete')

local acceptance = path.read(path.join(b.root, 'scripts/ue_local_acceptance.lua'))
assert(acceptance:find("require('moba_owned_viewport_ipc')", 1, true), 'acceptance does not load the IPC workflow')
assert(acceptance:find('require_owned_viewport_phase', 1, true), 'acceptance can omit the IPC phase')
local call_at = assert(acceptance:find('ipc%.run%('), 'acceptance does not call the IPC workflow')
local window = acceptance:sub(math.max(1, call_at - 120), call_at - 1)
assert(not window:find('native_ok', 1, true), 'IPC invocation is gated on the disconnected native result: ' .. window)
count = count + 3
local audio_name='Om.Runtime.OriginalAudioOutput'
local plans=contract.native_execution_batches({expected[1],audio_name,expected[2]},1,4096)
assert(#plans==3 and plans[1].profile=='native' and plans[1].tests[1]==expected[1]
  and plans[2].tests[1]==expected[2] and plans[3].profile=='audio-output' and plans[3].tests[1]==audio_name)
count=count+1
local audio_only=contract.native_execution_batches({audio_name})
assert(#audio_only==1 and audio_only[1].profile=='audio-output' and #audio_only[1].tests==1);count=count+1
local regular_only=contract.native_execution_batches(expected)
assert(#regular_only==1 and regular_only[1].profile=='native' and #regular_only[1].tests==2);count=count+1
assert(not pcall(contract.native_execution_batches,{audio_name,audio_name}));count=count+1
assert(not pcall(contract.native_execution_batches,{audio_name},0,4096));count=count+1
local full={audio_name};for index=1,168 do full[#full+1]='Om.Runtime.ProfileCase'..index end
local union,union_count={},0
for _,plan in ipairs(contract.native_execution_batches(full))do
 assert(#plan.tests<=32)
 for _,name in ipairs(plan.tests)do assert(not union[name]);union[name]=true;union_count=union_count+1 end
end
assert(union_count==169 and union[audio_name]);count=count+1
local command_file=contract.native_command_file(expected,'D:/Project Space/report','D:/Project Space/test.log')
assert(command_file:find('-ExecCmds="Automation RunTests ^'..expected[1]..'$+^'..expected[2]..'$"',1,true)
  and command_file:find('-ReportExportPath="D:/Project Space/report"',1,true)
  and not command_file:find('TestExit',1,true));count=count+1
for _,bad in ipairs({'','D:/bad"path','D:/bad\npath','D:/bad\rpath'})do
 assert(not pcall(contract.native_command_file,expected,bad,'D:/test.log'));count=count+1
end
assert(not pcall(contract.native_command_file,{'Om.Runtime.A+Om.Runtime.B'},'D:/report','D:/test.log'));count=count+1
print('local Unreal acceptance contract: ' .. count .. ' cases passed')
