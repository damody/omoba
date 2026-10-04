-- Declarative event migration through the existing Editor MCP. Default is preview.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=source:match('^(.*)[/\\]')..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json,hash=b.lib('path'),b.lib('json'),b.lib('hash')
local planner,validation=require('ue_event_migration'),require('ue_blueprint_validation')
local plan_file,apply
for _,value in ipairs(arg) do
  if value=='--apply' then apply=true else assert(not plan_file,'one plan path required');plan_file=value end
end
local rule=json.read(assert(plan_file,'usage: ue_migrate_event_branch.lua PLAN.json [--apply]'))
local asset=validation.package_path(rule.blueprint_path)
local relative=asset:sub(#'/Game/'+1)
local package=path.join(b.root,'omfue/Content',relative..'.uasset')
assert(path.is_file(package),'asset package missing')
local work=path.join(b.root,'target/event-migration-runs','migration-'..os.time())
assert(not path.exists(work),'run already exists');path.mkdir_p(work)
local report={success=false,asset=asset,work=work,apply=apply==true,initial_sha256=hash.sha256(package)}
local ordinal=0
local function call(tool,args)
  ordinal=ordinal+1
  local output=path.join(work,ordinal..'-'..tool..'.json')
  local result=b.lib('process').run(b.lib('platform').lua_executable,{
    path.join(b.root,'scripts/ue_mcp.lua'),'--tool',tool,'--arguments',json.encode(json.object(args)),'--out',output
  },{cwd=b.root,check=false})
  assert(result.exit_code==0,tool..' failed: '..output)
  local response=json.read(output);assert(not response.isError,tool..' isError')
  for _,content in ipairs(response.content or {}) do
    if content.type=='text' then
      local ok,value=pcall(json.decode,content.text)
      if ok and type(value)=='table' then assert(value.ok~=false and value.success~=false,tool..' failed');return value end
    end
  end
  error('no structured response: '..tool)
end
local function snapshot()
  return call('get_graph_snapshot',{blueprint_path=asset,graph_name=rule.graph_name,
    view='compact',include_hashes=true,include_unconnected_pins=true,max_nodes=1000})
end
local function compile()
  ordinal=ordinal+1
  local result=validation.compile(b,asset,path.join(work,ordinal..'-compile.json'))
  assert(result.success,table.concat(result.diagnostics,'; '));return result
end
local function save()
  local saved=call('save_assets',{asset_paths={asset}})
  assert(saved.failed==0 and saved.saved==1,'asset save failed')
end
local ok,err=xpcall(function()
  assert(not call('get_editor_dialog',{}).dialog_open,'Editor modal open')
  assert(not call('get_pie_status',{}).pie_running,'PIE must be stopped')
  local before=snapshot()
  local migration=planner.plan(before,rule)
  report.removed=migration.removed;report.already_migrated=migration.already_migrated
  if migration.already_migrated then report.after_sha256=hash.sha256(package);report.success=true;return end
  if #migration.patch > 0 then
    report.preview=call('apply_graph_patch',{blueprint_path=asset,graph_name=rule.graph_name,patch=migration.patch,preview=true})
    planner.patch_result(report.preview,#migration.patch,true)
  end
  json.write(path.join(work,'report.json'),report,true)
  if not apply then report.success=true;return end
  -- Compile and save only this exact package, then preserve its binary before mutation.
  report.baseline_compile=compile();save()
  report.backup=path.join(work,'before.uasset')
  path.write(report.backup,path.read(package,true),false,true)
  report.backup_sha256=hash.sha256(report.backup)
  json.write(path.join(work,'report.json'),report,true)
  -- Re-read after baseline compile: compile can reconstruct nodes and change CAS hashes.
  before=snapshot();migration=planner.plan(before,rule)
  if #migration.patch > 0 then
    report.patch=call('apply_graph_patch',{blueprint_path=asset,graph_name=rule.graph_name,patch=migration.patch,preview=false})
    planner.patch_result(report.patch,#migration.patch,false)
  end
  -- CAS deliberately refuses entry points. Verify the now-isolated exact event,
  -- then use the documented single-node deletion API; no name/handle wildcard.
  planner.verify(migration,snapshot(),rule,true)
  report.root_delete=call('delete_nodes',{blueprint_path=asset,graph_name=rule.graph_name,
    node_ids={migration.root},allow_destructive=true})
  planner.verify(migration,snapshot(),rule)
  report.compile=compile();save()
  planner.verify(migration,snapshot(),rule)
  report.after_sha256=hash.sha256(package)
  report.success=true
end,debug.traceback)
if not ok then report.error=tostring(err) end
json.write(path.join(work,'report.json'),report,true)
print('[ue-event-migration] success='..tostring(report.success)..'; report: '..path.join(work,'report.json'))
assert(ok,err)
