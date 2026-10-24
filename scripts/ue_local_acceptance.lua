-- Official commandlet acceptance for local native code and existing RustBP assets.
-- This is a separate acceptance scope from vendor MCP, live gameplay and LAN.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, process = b.lib('path'), b.lib('json'), b.lib('process')
local lfs = require('lfs')
local contract = require('ue_local_acceptance_contract')
local ipc = require('moba_owned_viewport_ipc')
local engine, run_id
local native_rhi = 'd3d12'
local record_video = false
local owned_execution = 'single'
local i = 1
while i <= #arg do
  if arg[i] == '--ue-root' then i = i + 1; engine = assert(arg[i], '--ue-root requires a path')
  elseif arg[i] == '--run-id' then i = i + 1; run_id = assert(arg[i], '--run-id requires a value')
  elseif arg[i] == '--native-rhi' then
    i = i + 1; native_rhi = assert(arg[i], '--native-rhi requires d3d12 or null')
    assert(native_rhi == 'd3d12' or native_rhi == 'null', '--native-rhi requires d3d12 or null')
  elseif arg[i] == '--record-video' then record_video = true
  elseif arg[i] == '--owned-execution' then
    i = i + 1; owned_execution = assert(arg[i], '--owned-execution requires single or phased')
    assert(owned_execution == 'single' or owned_execution == 'phased', '--owned-execution requires single or phased')
  elseif arg[i] == '--help' then
    print('Usage: tools/lua/lua.exe scripts/ue_local_acceptance.lua --ue-root PATH [--run-id UNIQUE_NAME] [--native-rhi d3d12|null] [--record-video] [--owned-execution single|phased]')
    print('Runs local native tests, the owned viewport IPC phase, and official CompileAllBlueprints, without saving existing assets or starting a match.')
    print('Native defaults to d3d12 for painted viewport input tests; Blueprint compilation always uses NullRHI.')
    return
  else error('unknown argument: ' .. tostring(arg[i])) end
  i = i + 1
end
engine = path.absolute(assert(engine, '--ue-root is required'), b.root)
run_id = run_id or os.date('!%Y%m%dT%H%M%SZ')
assert(run_id:match('^[%w_-]+$'), 'run-id must contain only letters, digits, underscores or hyphens')
assert(#run_id<=74, 'run-id must fit the bounded audio output identifier with its suffix')
assert(require('moba_renderer_fixture').valid_run_id(run_id..'-ipc'..(owned_execution=='phased' and '-foundation' or '')),
  'run-id must fit every bounded owned viewport phase identifier')
local project_dir = path.join(b.root, 'omfue')
local project = path.join(project_dir, 'om.uproject')
local exe = path.join(engine, 'Engine/Binaries/Win64/UnrealEditor-Cmd.exe')
assert(path.is_file(exe), 'selected engine commandlet executable is missing')
assert(not path.exists(path.join(project_dir, 'Saved/BuildTemp/source-engine-bootstrap.json')),
  'finish pending source-engine initialization before acceptance')
require('ue_binary_preflight').require_ready(engine, project)
assert(path.is_file(path.join(b.root,'omb/target/release/moba-config.exe'))
    or path.is_file(path.join(b.root,'omb/target/debug/moba-config.exe')),
  'HUD selection acceptance requires moba-config.exe; build explicitly: cargo build --manifest-path omb/Cargo.toml -p omobab --bin moba-config')
require('moba_stage_contract').verify(require('moba_stage_contract').bindings(b.root))
process.supervise_workflow(source, arg)

local relative = 'Saved/Automation/LocalAcceptance/' .. run_id
local work = path.join(project_dir, relative)
assert(not path.exists(work), 'acceptance evidence directory must be new: ' .. work)
path.mkdir_p(work)
local report = {success = false, scope = 'local-native-and-existing-assets', engine_root = engine,
  run_id = run_id, native_rhi = native_rhi, blueprint_rhi = 'null', expected_tests = {}, expected_assets = {}, processes = {}}
local function visit(directory, fn)
  for name in lfs.dir(directory) do
    if name ~= '.' and name ~= '..' then
      local file = path.join(directory, name)
      local mode = assert(lfs.symlinkattributes(file), 'cannot inspect: ' .. file).mode
      if mode == 'directory' then visit(file, fn)
      elseif mode == 'file' then fn(file, name) end
    end
  end
end
local function run(label, arguments)
  local result = process.run(exe, arguments, {cwd = project_dir, check = false})
  path.write(path.join(work, label .. '-stdout.log'), result.stdout or '', false)
  path.write(path.join(work, label .. '-stderr.log'), result.stderr or '', false)
  report.processes[#report.processes + 1] = {label = label, exit_code = result.exit_code}
  assert(result.exit_code == 0, label .. ' process failed: ' .. tostring(result.exit_code))
end
local base = {project, '-unattended', '-nop4', '-NoSound', '-NoLiveCoding'}
local function args(extra, rhi)
  local result = {}; for _, value in ipairs(base) do result[#result + 1] = value end
  result[#result + 1] = rhi == 'd3d12' and '-d3d12' or '-NullRHI'
  for _, value in ipairs(extra) do result[#result + 1] = value end
  return result
end
local native_ok, native_failure = xpcall(function()
  for _, folder in ipairs({'Plugins/OmRuntime/Source/OmEditor/Private',
      'Plugins/BpGeneratorUltimate/Source/UECPMCPBridge/Private/Tests',
      'Plugins/BpGeneratorUltimate/Source/UECPTools/Private/Tests'}) do
    visit(path.join(project_dir, folder), function(file, name)
      if name:match('%.cpp$') then
        for _, test in ipairs(contract.test_names(path.read(file))) do
          report.expected_tests[#report.expected_tests + 1] = test
        end
      end
    end)
  end
  table.sort(report.expected_tests)
  assert(#report.expected_tests > 0, 'no native tests discovered from source')
  visit(path.join(project_dir, 'Content/RustBP'), function(file, name)
    if name:match('%.uasset$') and (name:match('^BP_') or name:match('^WBP_') or name:match('^ABP_')) then
      local content = path.join(project_dir, 'Content')
      local relative_asset = file:sub(#content + 2):gsub('\\', '/')
      local stem = assert(name:match('^(.*)%.uasset$'))
      local asset = '/Game/' .. relative_asset:gsub('%.uasset$', '') .. '.' .. stem
      assert(not asset:find("['\r\n]"), 'unsupported asset path: ' .. asset)
      report.expected_assets[#report.expected_assets + 1] = asset
    end
  end)
  table.sort(report.expected_assets)
  assert(#report.expected_assets > 0, 'no existing RustBP Blueprint assets discovered')
  json.write(path.join(work, 'expected.json'), report, false)
  print('[ue-local-acceptance] native tests: ' .. #report.expected_tests)
  local combined={tests={},succeeded=0,succeededWithWarnings=0,failed=0,notRun=0,inProcess=0}
  report.native_batches={}
  local batch_errors={}
  for index,plan in ipairs(contract.native_execution_batches(report.expected_tests))do
    local batch=plan.tests
    local label=string.format('native-%03d',index)
    print('[ue-local-acceptance] '..label..': '..#batch..' exact tests')
    local entry={label=label,profile=plan.profile,expected_tests=batch}
    report.native_batches[#report.native_batches+1]=entry
    local batch_ok,batch_error=xpcall(function()
      local report_file=path.join(work,label..'-report/index.json')
      if plan.profile=='audio-output' then
        local audio_id=run_id..'-audio'
        local audio=process.run(path.join(b.root,'tools/lua/lua.exe'),{
          path.join(b.root,'scripts/verify_original_audio_output.lua'),'--ue-root',engine,'--run-id',audio_id},
          {cwd=b.root,check=false})
        path.write(path.join(work,label..'-stdout.log'),audio.stdout or '',false)
        path.write(path.join(work,label..'-stderr.log'),audio.stderr or '',false)
        report.processes[#report.processes+1]={label=label,exit_code=audio.exit_code}
        assert(audio.exit_code==0,'actual audio output workflow failed')
        report.audio_output=json.read(path.join(b.root,'.ai-collab/original-audio-output',audio_id,'summary.json'))
        assert(report.audio_output.success and #report.audio_output.recordings==5,'actual PCM verification missing')
        report_file=path.join(b.root,'.ai-collab/original-audio-output',audio_id,'native-report/index.json')
      else
        -- UE's native CmdLineFile keeps the complete, reviewable manifest out
        -- of the OS process argument list. Engine logs also record expansion.
        local command_file=path.join(work,label..'-command.txt')
        path.write(command_file,contract.native_command_file(batch,path.join(work,label..'-report'),
          path.join(work,label..'-engine.log')),false)
        entry.command_file=command_file
        -- TestExit is registered before UE expands CmdLineFile, so it must
        -- remain an early launch argument rather than part of that manifest.
        run(label, args({'-TestExit=Automation Test Queue Empty','-CmdLineFile='..command_file}, native_rhi))
      end
      local bytes=path.read(report_file,true)
      if bytes:sub(1,3)=='\239\187\191'then bytes=bytes:sub(4)end
      local value=json.decode(bytes)
      for _,test in ipairs(value.tests)do combined.tests[#combined.tests+1]=test end
      for _,field in ipairs({'succeeded','succeededWithWarnings','failed','notRun','inProcess'})do
        combined[field]=combined[field]+value[field]
      end
      entry.summary=contract.native(value,batch)
    end,debug.traceback)
    entry.success=batch_ok
    if not batch_ok then
      entry.error=batch_error
      batch_errors[#batch_errors+1]=label..': '..tostring(batch_error)
      path.write(path.join(work,label..'-error.md'),'# Native batch failure\n\n```text\n'..tostring(batch_error)..'\n```\n',false)
      print('[ue-local-acceptance] '..label..' failed; preserving evidence and continuing remaining batches')
    end
    json.write(path.join(work,'native-progress.json'),{complete=false,batches=report.native_batches},true)
  end
  json.write(path.join(work,'native-combined.json'),combined,false)
  json.write(path.join(work,'native-progress.json'),{complete=true,batches=report.native_batches},true)
  assert(#batch_errors==0,table.concat(batch_errors,'\n'))
  report.native=contract.native(combined,report.expected_tests)
end, debug.traceback)
-- The IPC phase always runs. A native failure must not omit it or count it as accepted.
local ipc_ok, ipc_result = xpcall(function()
  local runner = owned_execution == 'phased' and require('ue_owned_phase_coverage') or ipc
  return runner.run({ue_root = engine, run_id = run_id .. '-ipc', record_video = record_video})
end, debug.traceback)
if ipc_ok then report.owned_viewport_ipc = ipc_result
else report.owned_viewport_ipc = {executed = false, success = false, not_run = 1, failed = 1, tests = {}, error = ipc_result} end
local blueprint_ok, blueprint_failure = false, nil
if native_ok then
  blueprint_ok, blueprint_failure = xpcall(function()
    path.write(path.join(work, 'blueprint-allowlist.txt'), table.concat(report.expected_assets, '\n') .. '\n', false)
    print('[ue-local-acceptance] existing Blueprint assets: ' .. #report.expected_assets)
    run('blueprints', args({'-run=CompileAllBlueprints', '-AllowListFile=' .. relative .. '/blueprint-allowlist.txt',
      '-SimpleAssetList', '-abslog=' .. path.join(work, 'blueprint-engine.log')}, 'null'))
    report.blueprints = contract.blueprints(path.read(path.join(work, 'blueprint-engine.log')), report.expected_assets)
  end, debug.traceback)
else
  report.blueprints_skipped = true
  blueprint_failure = 'blueprints were not started because native acceptance failed'
end
local phase_ok, phase_failure = pcall(contract.require_owned_viewport_phase, report.owned_viewport_ipc, {'Om.Runtime.OwnedViewportIpc'})
local errors = {}
if not native_ok then errors[#errors + 1] = 'native: ' .. tostring(native_failure) end
if not (report.owned_viewport_ipc and report.owned_viewport_ipc.success) then
  errors[#errors + 1] = 'owned viewport: ' .. tostring(report.owned_viewport_ipc and report.owned_viewport_ipc.error or ipc_result)
end
if not blueprint_ok then errors[#errors + 1] = 'blueprints: ' .. tostring(blueprint_failure) end
if not phase_ok then errors[#errors + 1] = 'owned viewport phase: ' .. tostring(phase_failure) end
report.success = native_ok and blueprint_ok and phase_ok
if not report.success then
  report.error = table.concat(errors, '\n\n')
  path.write(path.join(work, 'errors.md'), '# Local Unreal acceptance failure\n\n```text\n' .. report.error .. '\n```\n', false)
end
json.write(path.join(work, 'summary.json'), report, false)
assert(report.success, report.error)
print('[ue-local-acceptance] PASS: ' .. #report.expected_tests .. ' native tests, owned viewport IPC, ' .. #report.expected_assets .. ' Blueprint assets; evidence: ' .. work)
