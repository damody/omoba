-- Real MCP creation / compile-failure / recovery; only new isolated test assets.
local source = debug.getinfo(1,'S').source:sub(2)
package.path = source:match('^(.*)[/\\]tests[/\\]') .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path,json,process = b.lib('path'),b.lib('json'),b.lib('process')
local validation = require('ue_blueprint_validation')
local run_id = 'blueprint-' .. os.time()
local work = path.join(b.root,'target/blueprint-validation-runs',run_id)
assert(not path.exists(work),'test run already exists')
path.mkdir_p(work)
local folder = '/Game/OmAutomation/BlueprintValidation_' .. os.time()
local actor, widget = folder .. '/BP_CompileBoundary', folder .. '/WBP_CompileBoundary'
local generated = folder .. '/BP_GeneratedParent'
local report = {success=false,kind='live-blueprint-editor-validation',run_id=run_id,
  actor=actor,widget=widget,generated=generated,calls={},compiles={}}
local ordinal = 0
local function call(name,args)
  ordinal = ordinal+1
  local file = path.join(work,ordinal .. '-' .. name .. '.json')
  local command = process.run(b.lib('platform').lua_executable, {
    path.join(b.root,'scripts/ue_mcp.lua'),'--tool',name,'--arguments',json.encode(json.object(args)),'--out',file,
  }, {cwd=b.root,check=false})
  report.calls[#report.calls+1]={tool=name,arguments=args,raw_response=file,exit_code=command.exit_code}
  assert(command.exit_code==0,name .. ' failed: ' .. command.stderr)
  for _,content in ipairs(json.read(file).content or {}) do
    if content.type=='text' then
      local ok,value=pcall(json.decode,content.text)
      if ok and type(value)=='table' then return value end
    end
  end
  error('missing structured result: '..name)
end
local function compile(asset,label)
  local result=validation.compile(b,asset,path.join(work,label..'-compile.json'))
  report.compiles[label]=result
  return result
end
local ok,failure=xpcall(function()
  assert(not call('get_editor_dialog',{}).dialog_open,'Editor modal blocks test')
  assert(not call('get_pie_status',{}).pie_running,'stop PIE before asset test')
  for _,entry in ipairs({{asset=actor,name='BP_CompileBoundary',kind='Blueprint',parent='Actor'},
      {asset=widget,name='WBP_CompileBoundary',kind='WidgetBlueprint',parent='UserWidget'},
      {asset=generated,name='BP_GeneratedParent',kind='Blueprint',parent='OmHeroDateMasamune',
        class='/Script/OmGenerated.OmHeroDateMasamune'}}) do
    assert(call('check_asset_exists',{asset_path=entry.asset}).exists==false,'refusing unowned test asset')
    local created=call('create_asset',{asset_type=entry.kind,name=entry.name,save_path=folder,
      options={parent_class=entry.class or entry.parent}})
    assert(validation.package_path(created.asset_path)==entry.asset,'unexpected creation path')
    local skeleton=call('get_blueprint_skeleton',{blueprint_path=entry.asset})
    assert(skeleton.parent_class==entry.parent,'fixture parent mismatch')
    assert(compile(entry.asset,entry.name..'-initial').success,'initial compile failed')
  end
  local saved=call('save_assets',{asset_paths={actor,widget,generated}})
  assert(saved.failed==0 and saved.saved==3,'fixture save failed')
  local built=call('build_blueprint_graph',{blueprint_path=actor,graph_name='EventGraph',
    nodes={{id='begin',handle='ev.ReceiveBeginPlay'},{id='invalid_jump',handle='fn.Character.Jump'}},
    connections={{from='begin.then',to='invalid_jump.execute'}},
    comments={{text='Negative compile fixture: Actor self is not Character',node_ids={'begin','invalid_jump'}}}})
  assert(built.nodes.failed==0 and built.connections.failed==0,'invalid-target fixture failed to build')
  local failed=compile(actor,'invalid')
  assert(not failed.success and failed.status=='pending' and failed.compiler.error_count>0,
    'real compiler failure was accepted')
  assert(table.concat(failed.diagnostics,' '):find('not a Character',1,true),'wrong failure diagnostic')
  -- Exercise the production CLI and its persisted pending/error path as well.
  local command=process.run(b.lib('platform').lua_executable, {
    path.join(b.root,'scripts/ue_validate_blueprints.lua'),actor,
  }, {cwd=b.root,check=false})
  assert(command.exit_code~=0,'production CLI accepted broken Blueprint')
  local pending_file=assert(command.stdout:match('report: ([^\r\n]+)'),'missing pending report path')
  report.pending_report=pending_file
  local pending=json.read(pending_file)
  assert(not pending.success and pending.assets[1].asset==actor and pending.assets[1].status=='pending'
    and #pending.assets[1].diagnostics>0,'production failure lost asset/diagnostics')
  local deleted=call('delete_nodes',{blueprint_path=actor,graph_name='EventGraph',node_ids={'invalid_jump'}})
  assert(deleted.deleted_count==1,'owned fixture repair failed')
  assert(compile(actor,'repaired').success,'same asset did not recover')
  local skeleton=call('get_blueprint_skeleton',{blueprint_path=actor})
  assert(skeleton.parent_class=='Actor','repair changed parent')
  local intent=call('get_graph_intent',{blueprint_path=actor,graph_name='EventGraph'})
  assert(intent.intent:find('ReceiveBeginPlay',1,true) and not intent.intent:find('Jump',1,true),
    'repair left invalid call')
  report.repaired_intent=intent.intent
  local final_save=call('save_assets',{asset_paths={actor,widget,generated}})
  assert(final_save.failed==0 and final_save.saved==3,'clean fixtures not saved')
  assert(compile(actor,'repeat').success and compile(widget,'widget-repeat').success,'saved repeat failed')
  assert(compile(generated,'generated-repeat').success,'generated native parent repeat failed')
  report.success=true
end,debug.traceback)
if not ok then report.error=tostring(failure) end
json.write(path.join(work,'report.json'),report,true)
print('[ue-blueprint-editor] success='..tostring(report.success)..'; report: '..path.join(work,'report.json'))
assert(ok,failure)
