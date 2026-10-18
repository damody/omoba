-- Rendered PIE capture fixture: selection only, never starts a match authority.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json,process,time,platform=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('time'),b.lib('platform')
local options={}
local i=1
while i<=#arg do
  local key=arg[i]
  if key=='--help' then print('Usage: tools/lua/lua.exe scripts/ue_pie_capture_validation.lua --ue-root PATH --candidate FILE --output NEW_DIRECTORY');return end
  assert(key=='--ue-root' or key=='--candidate' or key=='--output','unknown argument: '..key)
  i=i+1;options[key]=assert(arg[i],key..' requires a value');i=i+1
end
local output=path.absolute(assert(options['--output'],'--output required'),b.root)
local candidate=path.absolute(assert(options['--candidate'],'--candidate required'),b.root)
assert(path.is_file(candidate),'compiled selection candidate missing')
assert(not path.exists(output),'capture fixture output must be new')
local editor,ue_root=require('moba_role_launch').editor({ue_root=options['--ue-root']})
require('moba_frontend_preflight').verify(ue_root,process)
local config=require('moba_config_command').resolve({profile='release'})
process.supervise_workflow(source,arg)
path.mkdir_p(path.parent(output));assert(require('lfs').mkdir(output),'fixture directory reservation failed')
local report={success=false,simulations_executed=0,scope='selection-only-rendered-PIE-capture-fixture'}
local record,started,ordinal
ordinal=0
local function mcp(name,args)
  ordinal=ordinal+1
  local file=path.join(output,('mcp-%03d-%s.json'):format(ordinal,name))
  local result=process.run(platform.lua_executable,{path.join(b.root,'scripts/ue_mcp.lua'),
    '--owned-editor',path.join(output,'owned-editor.json'),'--tool',name,
    '--request-timeout-seconds',tostring(require('ue_mcp_request_budget').for_tool(name)),
    '--arguments',json.encode(json.object(args)),'--out',file},{cwd=b.root,check=false})
  assert(result.exit_code==0,'fixture MCP failed: '..name..'\n'..result.stderr..result.stdout)
  local envelope=json.read(file);assert(not envelope.isError,'fixture MCP isError')
  for _,item in ipairs(envelope.content or {}) do
    if item.type=='text' then
      local ok,detail=pcall(json.decode,item.text)
      if ok and type(detail)=='table' then
        assert(detail.success~=false and detail.ok~=false,'fixture MCP negative result: '..name)
        return detail
      end
    end
  end
  error('fixture MCP lacks structured result')
end
local ok,err=xpcall(function()
  local _,owned=process.spawn_owned(editor,{path.join(b.root,'omfue/om.uproject'),'/Game/Map/Main',
    '-d3d12','-noraytracing','-windowed','-ResX=1280','-ResY=720','-nosplash','-nop4','-NoLiveCoding',
    '-AllowMultipleInstances','-UserDir='..path.join(output,'user'),'-abslog='..path.join(output,'editor.log'),
    '-stdout','-FullStdOutLogOutput','-om-networked','-om-presentation-only','-om-native-content',
    '-om-player=1','-om-team=1','-om-presentation=127.0.0.1:57062',
    '-om-hero-selection','-om-selection-in-place','-om-selection-player=1',
    '-om-selection-exe='..config,'-om-selection-plan='..candidate,
    '-om-selection-result='..path.join(output,'selection-reply.json')},
    {cwd=path.join(b.root,'omfue'),stdout=path.join(output,'editor.stdout.log'),stderr=path.join(output,'editor.stderr.log')})
  record=process.validate_owned(owned);json.write(path.join(output,'owned-editor.json'),record)
  local endpoint=require('ue_mcp_endpoint')
  require('moba_role_launch').await_owned_editor_ready(process,record,{editor_ready_timeout_seconds=3600},function()
    local found,value=pcall(endpoint.resolve,path.join(b.root,'omfue/om.uproject'))
    if found then endpoint.require_owned(value,record,process);return true end
    local reason=tostring(value)
    assert(reason:find('found 0',1,true) or reason:find('registry unavailable',1,true),reason)
    return false
  end)
  mcp('ping',{});local status=mcp('get_pie_status',{});assert(status.pie_running==false,'fixture Editor already has PIE')
  mcp('begin_play_in_editor',{num_clients=1,net_mode='standalone'});started=true
  local ready=require('moba_selection_readiness').reader(1,false)
  process.poll_owned_ready(record,180000,function() return ready:poll(path.join(output,'editor.stdout.log'),false) end,'fixture selection readiness')
  local filename=path.join(output,'selection.png')
  local capture=mcp('take_pie_screenshot',{file_path=filename})
  assert(capture.source=='pie_widget','capture is not exact PIE widget evidence')
  assert(path.absolute(capture.file_path)==path.absolute(filename),'fixture screenshot path mismatch')
  assert(capture.width>0 and capture.height>0,'fixture screenshot dimensions invalid')
  assert(path.is_file(filename) and path.read(filename,true):sub(1,8)=='\137PNG\13\10\26\10','fixture PNG missing or invalid')
  report.capture=capture;report.success=true
end,debug.traceback)
local cleanup={}
local function retire()
  if record then
    local clean,reason=pcall(function()
      if process.owned_alive(record) then process.stop_owned(record) end
      assert(process.wait_owned(record,10000),'fixture owned Editor remained')
    end)
    if not clean then cleanup[#cleanup+1]=tostring(reason) end
  end
  return cleanup
end
if started and record then
  local stopped=require('ue_pie_lifecycle').await_pie_stop({owned_alive=function() return process.owned_alive(record) end,
    call=mcp,now_ms=time.monotonic_ms,sleep_ms=time.sleep_ms,retire=retire})
  report.pie_stop=stopped;cleanup=stopped.cleanup_errors
else retire() end
report.cleanup_errors=cleanup;report.cleanup_verified=#cleanup==0 and (not record or not process.owned_alive(record))
if not ok then report.error=err;report.success=false end
if not report.cleanup_verified or report.pie_stop and report.pie_stop.pie_cleanup_error then report.success=false end
json.write(path.join(output,'report.json'),report)
assert(report.success,'rendered PIE capture fixture failed: '..output)
print('rendered PIE capture fixture PASS: '..path.join(output,'selection.png'))
