-- Recompute compile verdicts from saved raw MCP responses, not success banners.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=source:match('^(.*)[/\\]tests[/\\]')..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local v=require('ue_blueprint_validation')
assert(#arg==1 or #arg==2,'usage: ue_blueprint_editor_acceptance.lua RUN_DIRECTORY [DEFAULT_GATE_REPORT]')
local run=path.absolute(arg[1])
local prior=json.read(path.join(run,'report.json'))
assert(prior.success and prior.kind=='live-blueprint-editor-validation','Editor test failed')
local report={success=false,kind='saved-blueprint-editor-validation',run=run,compiles={}}
local expected={['BP_CompileBoundary-initial']=prior.actor,['WBP_CompileBoundary-initial']=prior.widget,
  ['BP_GeneratedParent-initial']=prior.generated,invalid=prior.actor,repaired=prior.actor,
  ['repeat']=prior.actor,['widget-repeat']=prior.widget,['generated-repeat']=prior.generated}
for label,asset in pairs(expected) do
  assert(asset,'missing required creation test')
  local attempt=assert(prior.compiles[label],'missing compile stage '..label)
  local result=v.result(asset,attempt.exit_code,json.read(attempt.raw_response))
  if label=='invalid' then
    assert(not result.success and result.status=='pending' and result.compiler.error_count==1
      and table.concat(result.diagnostics,' '):find('not a Character',1,true),'no genuine compile error')
  else assert(result.success,'clean stage did not compile: '..label) end
  report.compiles[label]=result
end
local creates,saves,repairs=0,0,0
for _,call in ipairs(prior.calls) do
  assert(call.exit_code==0,'MCP operation failed')
  local raw=json.read(call.raw_response)
  assert(not raw.isError,'MCP operation error')
  local detail
  for _,content in ipairs(raw.content or {}) do
    if content.type=='text' then detail=json.decode(content.text); break end
  end
  assert(detail and detail.ok~=false,'invalid saved operation')
  if call.tool=='create_asset' then
    assert(v.package_path(detail.asset_path)==call.arguments.save_path..'/'..call.arguments.name)
    creates=creates+1
  elseif call.tool=='save_assets' then
    assert(detail.saved==3 and detail.failed==0,'incomplete fixture save'); saves=saves+1
  elseif call.tool=='delete_nodes' then
    assert(detail.deleted_count==1 and call.arguments.blueprint_path==prior.actor); repairs=repairs+1
  end
end
assert(creates==3 and saves==2 and repairs==1,'creation/save/recovery path incomplete')
local pending=json.read(prior.pending_report)
assert(not pending.success and pending.assets[1].asset==prior.actor and pending.assets[1].status=='pending'
  and #pending.assets[1].diagnostics>0,'production CLI lost pending error')
report.production_failure=pending.assets[1]
report.created_assets={prior.actor,prior.widget,prior.generated}
if arg[2] then
  local gate=json.read(path.absolute(arg[2]))
  assert(gate.success and gate.kind=='unreal-blueprint-compile','production default gate failed')
  local expected_assets={}
  local recipe=json.read(path.join(b.root,'omfue/Plugins/OmRuntime/Source/OmGenerated/om_asset_recipe.json'))
  for _,hero in ipairs(recipe.heroes) do
    if hero.blueprint_path and hero.blueprint_path~='' then expected_assets[v.package_path(hero.blueprint_path)]=true end
  end
  for file in require('lfs').dir(path.join(b.root,'omfue/Content/RustBP/UI')) do
    local name=file:match('^(WBP_[%w_]+)%.uasset$')
    if name then expected_assets['/Game/RustBP/UI/'..name]=true end
  end
  report.production_default_gate={}
  for _,attempt in ipairs(gate.assets) do
    assert(expected_assets[attempt.asset],'unexpected or duplicate default asset')
    expected_assets[attempt.asset]=nil
    local result=v.result(attempt.asset,attempt.exit_code,json.read(attempt.raw_response))
    assert(result.success,'production asset did not compile: '..attempt.asset)
    report.production_default_gate[#report.production_default_gate+1]=result
  end
  assert(not next(expected_assets),'default gate omitted a required asset')
end
report.success=true
local destination=path.join(b.root,'openspec/changes/build-unreal-rust-moba-framework/evidence/blueprint-validation')
path.mkdir_p(destination)
local output=path.join(destination,prior.run_id..'.json')
json.write(output,report,true)
print('saved Blueprint Editor validation: PASS; '..output)
