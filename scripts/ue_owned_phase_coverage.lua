-- Full inventory coverage across bounded, independently cleaned editor sessions.
-- This never represents a single-session full run as successful.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local p, j, h = b.lib('path'), b.lib('json'), b.lib('hash')
local ipc = require('moba_owned_viewport_ipc')
local fixture = require('moba_renderer_fixture')
local M = {}
local PHASES = {'foundation', 'tail'}
local CURSORS = {
  {'accessible-keyboard-cursor','held input moves real Slate cursor',true},
  {'accessible-keyboard-cursor','release or focus loss stops cursor',false},
  {'accessible-cursor-speed','held input moves real Slate cursor',true},
  {'accessible-cursor-speed','release or focus loss stops cursor',false},
  {'accessible-analog-cursor','held input moves real Slate cursor',true},
  {'accessible-analog-cursor','release or focus loss stops cursor',false},
  {'accessible-cursor-focus','held input moves real Slate cursor',true},
  {'accessible-cursor-focus','release or focus loss stops cursor',false},
  {'accessible-cursor-focus','other neutral axis cannot rearm held axis',false},
  {'accessible-cursor-focus','fresh neutral-admitted axis recovers',true},
  {'accessible-text-guard','editable focus blocks keyboard cursor',false},
  {'accessible-source-retire','source epoch retires held cursor',false},
  {'accessible-source-retire','fresh key recovers after source epoch',true},
}
function M.expected_cursor_observations() return j.decode(j.encode(CURSORS)) end
local parent = p.join(b.root, 'omfue/Saved/Automation/OwnedViewportIpc')
local binaries = {
  'omfue/Binaries/Win64/UnrealEditor-OmGame.dll',
  'omfue/Plugins/OmRuntime/Binaries/Win64/UnrealEditor-OmRuntime.dll',
  'omfue/Plugins/OmRuntime/Binaries/Win64/UnrealEditor-OmGenerated.dll',
  'omfue/Plugins/OmRuntime/Binaries/Win64/UnrealEditor-OmEditor.dll',
  'omfue/Plugins/OmRuntime/Binaries/Win64/om_bridge.dll',
  'omfue/Plugins/OmRuntime/Binaries/Win64/base_content.dll',
  'omoba-client-runtime/target/debug/omoba-renderer-fixture.exe',
}
local function cohort()
  local result = {}
  for _, file in ipairs(binaries) do result[#result + 1] = {file=file, sha256=h.sha256(p.join(b.root,file))} end
  return result
end
local function same(a, c, label) assert(j.encode(a) == j.encode(c), label) end
local function lines(file)
  local result = {}
  for line in (p.read(file,true)..'\n'):gmatch('([^\r\n]+)') do result[#result+1]=j.decode(line) end
  return result
end
local function load(run_id, phase)
  assert(fixture.valid_run_id(run_id), 'invalid phase run id')
  local root=p.join(parent,run_id)
  local files={startup=p.join(root,'startup.json'), native=root..'.native.json',
    intents=p.join(root,'intents.jsonl'), history=p.join(root,'control-history.jsonl'),
    ack=p.join(root,'control.ack.json'), cleanup=p.join(root,'cleanup.json'),
    ue_index=root..'.ue-report/index.json', workflow=root..'.workflow.json'}
  local bundle={phase=phase};local seals={}
  for key,file in pairs(files) do
    bundle[key]=(key=='intents' or key=='history') and lines(file)
      or key=='ue_index' and p.read(file,true) or j.read(file)
    seals[key]={file=file,sha256=h.sha256(file)}
  end
  local workflow=bundle.workflow
  assert(workflow.executed==true and workflow.success==true and workflow.failed==0 and workflow.not_run==0,
    'phase workflow failed or incomplete')
  assert(workflow.phase==phase and workflow.full_case_inventory==false, 'phase workflow inventory mismatch')
  assert(workflow.publisher_retired==true and workflow.editor_retired==true, 'phase lifetimes were not retired')
  assert(#workflow.cleanups==3,'phase cleanup inventory mismatch')
  for index,label in ipairs({'publisher','editor','profile'}) do
    local entry=workflow.cleanups[index]
    assert(entry.label==label and entry.ok==true,'phase cleanup failed: '..label)
  end
  return bundle,seals
end

function M.verify_union(bundles)
  assert(type(bundles)=='table' and #bundles==#PHASES,'phase count mismatch')
  local all,stages=ipc.expected_case_names(),ipc.expected_stages()
  local seen,case_index,stage_index,wires,cursors={},0,0,0,0
  local identity
  for index,phase in ipairs(PHASES) do
    local bundle=bundles[index]
    assert(bundle.phase==phase,'phase order mismatch')
    assert(bundle.cleanup,'phase cleanup proof is missing')
    ipc.verify_artifacts(bundle)
    local native=bundle.native
    assert(not seen[native.run_id],'phase run id was reused');seen[native.run_id]=true
    local current={}
    for _,key in ipairs({'content_hash','content_catalog_identity_hash','moba_map_catalog_hash',
      'generated_content_hash','compiled_content_hash','class_surface_signature','compiled_class_surface',
      'lockstep_step_fps','catalog_data_hash'}) do current[key]=native[key] end
    if identity then same(identity,current,'phase content or ABI identity drifted') else identity=current end
    for _,record in ipairs(native.cases) do
      case_index=case_index+1;assert(record.name==all[case_index],'phase case union has a gap, duplicate or reordering')
    end
    for _,record in ipairs(bundle.history) do
      stage_index=stage_index+1;assert(record.stage==stages[stage_index],'phase stage union differs')
    end
    wires=wires+#bundle.intents
    local witnesses=native.accessible_input or {}
    assert(#witnesses==(phase=='tail' and 13 or 0),'phase cursor observation count mismatch')
    for cursor_index,entry in ipairs(witnesses) do
      local spec=CURSORS[cursor_index]
      assert(entry.case==spec[1],'cursor case mismatch')
      assert(entry.observation==spec[2] and entry.user==0 and entry.expected_motion==spec[3],'cursor observation mismatch')
      local distance=0
      for _,axis in ipairs({'x','y'}) do
        local before,after=entry['before_'..axis],entry['after_'..axis]
        assert(type(before)=='number' and type(after)=='number' and math.abs(before)<math.huge
          and math.abs(after)<math.huge,'cursor coordinate missing or nonfinite')
        distance=distance+(after-before)^2
      end
      assert(entry.expected_motion and distance>1 or not entry.expected_motion and distance<1,'cursor motion proof differs')
      cursors=cursors+1
    end
  end
  assert(case_index==#all and stage_index==#stages and wires==222 and cursors==13,'incomplete full inventory union')
  return {cases=case_index,intents=wires,stages=stage_index,cursor_observations=cursors}
end

function M.verify_report(report)
  assert(report.execution_mode=='phased' and report.single_session_full_success==false,
    'phased coverage cannot claim single-session success')
  assert(report.full_case_coverage==true and report.full_case_inventory==false,'phased coverage flags differ')
  assert(type(report.phases)=='table' and #report.phases==#PHASES,'phase report count mismatch')
  local bundles={}
  for index,phase in ipairs(PHASES) do
    local entry=report.phases[index]
    assert(entry.phase==phase,'phase report order mismatch')
    local bundle,seals=load(entry.run_id,phase)
    same(entry.artifacts,seals,'phase evidence changed after verification')
    bundles[index]=bundle
  end
  same(report.cohort,cohort(),'phase binary cohort drifted')
  same(report.coverage,M.verify_union(bundles),'phase coverage counters differ')
  return true
end

function M.run(options)
  local report={executed=true,success=false,not_run=0,failed=1,tests={},execution_mode='phased',
    full_case_inventory=false,full_case_coverage=false,single_session_full_success=false,phases={}}
  local ok,err=xpcall(function()
    assert(type(options)=='table' and type(options.run_id)=='string','phased run id required')
    for _,phase in ipairs(PHASES) do
      local run_id=options.run_id..'-'..phase
      assert(fixture.valid_run_id(run_id),'phase run id exceeds bounded identifier')
      assert(not p.exists(p.join(parent,run_id)) and not p.exists(p.join(parent,run_id..'.workflow.json')),
        'phased evidence must be fresh')
    end
    fixture.build()
    report.cohort=cohort()
    local bundles={}
    for index,phase in ipairs(PHASES) do
      same(report.cohort,cohort(),'binary cohort changed before phase')
      local run_id=options.run_id..'-'..phase
      local result=ipc.run({ue_root=options.ue_root,run_id=run_id,phase=phase,record_video=options.record_video})
      report.phases[index]={phase=phase,run_id=run_id,workflow=result}
      assert(result.success,'owned '..phase..' failed: '..tostring(result.error))
      local bundle,seals=load(run_id,phase)
      report.phases[index].artifacts=seals;bundles[index]=bundle
      same(report.cohort,cohort(),'binary cohort changed during phase')
    end
    report.coverage=M.verify_union(bundles)
    report.full_case_coverage=true
    M.verify_report(report)
  end,debug.traceback)
  report.success=ok;report.failed=ok and 0 or 1;report.error=ok and nil or err
  report.tests={{name='Om.Runtime.OwnedViewportIpc',state=ok and 'Success' or 'Fail',errors=ok and 0 or 1,warnings=0}}
  return report
end
return M
