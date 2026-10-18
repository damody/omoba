local source=debug.getinfo(1,'S').source:sub(2)
local tests=assert(source:match('^(.*)[/\\]'))
package.path=assert(tests:match('^(.*)[/\\]tests$'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local preflight=require('ue_pie_preflight')
local count=0
local function test(name,fn) fn();count=count+1;print('PASS '..name) end

local REQUIRED={
  'get_pie_status','get_pie_actors','call_pie_blueprint_function','take_pie_screenshot',
  'begin_play_in_editor','stop_play_in_editor','get_automation_test_results',
}
local function tool(name)
  return {name=name,inputSchema={type='object',additionalProperties=true}}
end
local function catalog(names,extra)
  local tools={}
  for i,name in ipairs(names) do tools[i]=tool(name) end
  local payload={tools=tools}
  if extra then for key,value in pairs(extra) do payload[key]=value end end
  return payload
end
local function concat(left,right)
  local out={}
  for _,name in ipairs(left) do out[#out+1]=name end
  for _,name in ipairs(right) do out[#out+1]=name end
  return out
end
local function without(names,drop)
  local out={}
  for _,name in ipairs(names) do if name~=drop then out[#out+1]=name end end
  return out
end
local NATIVE=concat({'ping','health'},REQUIRED)

local function trace(events)
  local parts={}
  for _,event in ipairs(events) do
    parts[#parts+1]=event.op=='list' and 'list' or ('call:'..event.name)
  end
  return table.concat(parts,',')
end
local function run(lists,results,fail)
  local events={}
  local index=0
  local function list()
    index=index+1
    events[#events+1]={op='list',index=index}
    local payload=lists[index]
    if payload==nil then error('unexpected tools/list '..index) end
    return payload
  end
  local function call(name,args)
    events[#events+1]={op='call',name=name,args=args}
    if fail and fail[name] then error(fail[name]) end
    local spec=results and results[name]
    if type(spec)=='function' then return spec(args,events) end
    if spec~=nil then return spec end
    if name=='get_automation_test_results' then
      return {success=true,running=0,framework_running=false,controller_state='not_loaded',
        scope='read-only-automation-busy-state',
        source='GIsAutomationTesting+FAutomationTestFramework.GetCurrentTest+AutomationController.not_loaded'}
    end
    if name=='get_pie_status' then return {success=true,pie_running=false} end
    if name=='ping' then return {success=true,type='pong',bridge='uecp'} end
    if name=='enable_extension' then return {success=true} end
    return {success=true}
  end
  local began=false
  local ok,value=pcall(function()
    local report=preflight.prepare(list,call)
    began=true
    call('begin_play_in_editor',{num_clients=1,net_mode='standalone'})
    return report
  end)
  return ok,value,events,began
end
local function assert_rejected(ok,value,events,began,reason)
  assert(not ok and not began,tostring(value))
  assert(tostring(value):find(reason,1,true),tostring(value))
  assert(not trace(events):find('call:begin_play_in_editor',1,true),trace(events))
end
local function assert_empty(args)
  assert(type(args)=='table' and next(args)==nil)
end

test('native provider reads ping before status and never calls enable_extension',function()
  local ok,report,events,began=run({catalog(NATIVE)})
  assert(ok and began,tostring(report))
  assert(trace(events)=='list,call:ping,call:get_automation_test_results,call:get_pie_status,call:begin_play_in_editor',trace(events))
  assert_empty(events[2].args)
  assert_empty(events[3].args)
  assert_empty(events[4].args)
  assert(events[5].args.num_clients==1 and events[5].args.net_mode=='standalone')
  assert(report.read_only_tool=='ping' and report.extension_activated==false)
  assert(report.automation_running==0 and report.pie_running==false and report.passed==nil)
end)

test('extension provider refreshes after Testing and only then accepts capabilities',function()
  local initial=catalog({'ping','enable_extension','get_pie_status'})
  local refreshed=catalog(concat({'ping','enable_extension'},REQUIRED))
  local ok,report,events,began=run({initial,refreshed})
  assert(ok and began,tostring(report))
  assert(trace(events)=='list,call:ping,call:enable_extension,list,call:get_automation_test_results,call:get_pie_status,call:begin_play_in_editor',trace(events))
  local enable=events[3]
  assert(enable.args.extension_id=='Testing' and enable.args.enabled==true)
  local keys=0
  for _ in pairs(enable.args) do keys=keys+1 end
  assert(keys==2)
  assert(report.extension_activated==true and report.read_only_tool=='ping')
end)

test('extension refresh that drops a required tool does not begin',function()
  local initial=catalog(concat({'ping','enable_extension'},REQUIRED))
  local refreshed=catalog(concat({'ping','enable_extension'},without(REQUIRED,'get_automation_test_results')))
  local ok,value,events,began=run({initial,refreshed})
  assert_rejected(ok,value,events,began,'missing PIE capability: get_automation_test_results')
  assert(trace(events)=='list,call:ping,call:enable_extension,list',trace(events))
end)

test('missing native capability is named and does not call the missing tool',function()
  local ok,value,events,began=run({catalog(without(NATIVE,'get_automation_test_results'))})
  assert_rejected(ok,value,events,began,'missing PIE capability: get_automation_test_results')
  assert(trace(events)=='list,call:ping',trace(events))
end)

test('no advertised read-only capability fails closed before enable_extension',function()
  local ok,value,events,began=run({catalog({'enable_extension','begin_play_in_editor','get_pie_actors'})})
  assert_rejected(ok,value,events,began,'no suitable read-only capability')
  assert(trace(events)=='list',trace(events))
end)

test('ping wins over later read-only candidates',function()
  local names=concat({'get_project_info','get_editor_info','ping'},REQUIRED)
  local ok,report,events=run({catalog(names)})
  assert(ok,tostring(report))
  assert(events[2].name=='ping')
  for _,event in ipairs(events) do
    assert(event.name~='get_editor_info' and event.name~='get_project_info')
  end
end)

test('get_editor_info is the unlock when ping is absent and runs before enable',function()
  local initial=catalog(concat({'get_project_info','get_editor_info','enable_extension'},REQUIRED))
  local ok,report,events=run({initial,initial})
  assert(ok,tostring(report))
  assert(trace(events):find('^list,call:get_editor_info,call:enable_extension,list,',1,false),trace(events))
  for _,event in ipairs(events) do assert(event.name~='ping' and event.name~='get_project_info') end
  assert(report.read_only_tool=='get_editor_info')
end)

test('get_project_info is next and get_pie_status is last read-only candidate',function()
  local project=catalog(concat({'get_project_info'},REQUIRED))
  local ok,report,events=run({project})
  assert(ok and events[2].name=='get_project_info' and report.read_only_tool=='get_project_info',trace(events))
  local status_only=catalog(REQUIRED)
  local ok2,report2,events2=run({status_only})
  assert(ok2 and report2.read_only_tool=='get_pie_status',trace(events2))
  assert(trace(events2)=='list,call:get_pie_status,call:get_automation_test_results,call:get_pie_status,call:begin_play_in_editor',trace(events2))
end)

test('framework and controller busy responses both refuse before fresh PIE and begin',function()
  local framework={success=true,running=1,framework_running=true,controller_state='not_loaded',passed=false}
  local controller={success=true,running=1,framework_running=false,controller_state='running'}
  for _,payload in ipairs{framework,controller,{success=true,running=2}} do
    local ok,value,events,began=run({catalog(NATIVE)},{get_automation_test_results=payload})
    assert_rejected(ok,value,events,began,'automation running in owned Editor')
    assert(trace(events)=='list,call:ping,call:get_automation_test_results',trace(events))
  end
end)

test('unknown, malformed, and failed automation status do not begin',function()
  local cases={
    {{success=true},'automation running state unknown'},
    {{success=true,running=json.null},'automation running state malformed'},
    {{success=true,running=false},'automation running state malformed'},
    {{success=true,running='0'},'automation running state malformed'},
    {{success=true,running=-1},'automation running state malformed'},
    {{success=true,running=0.5},'automation running state malformed'},
    {{success=true,running=math.huge},'automation running state malformed'},
    {{success=false,running=0},'PIE MCP negative result: get_automation_test_results'},
  }
  for _,case in ipairs(cases) do
    local ok,value,events,began=run({catalog(NATIVE)},{get_automation_test_results=case[1]})
    assert_rejected(ok,value,events,began,case[2])
    assert(not trace(events):find('call:get_pie_status',1,true),trace(events)..' '..case[2])
  end
end)

test('native details cannot contradict idle or hide an unknown controller',function()
  local cases={
    {{success=true,running=0,framework_running=true},'automation running in owned Editor'},
    {{success=true,running=0,controller_state='running'},'automation running in owned Editor'},
    {{success=true,running=0,controller_state='unknown'},'automation controller state unknown'},
    {{success=true,running=0,framework_running='false'},'automation framework state malformed'},
    {{success=true,running=0,controller_state=false},'automation controller state malformed'},
  }
  for _,case in ipairs(cases) do
    local ok,value,events,began=run({catalog(NATIVE)},{get_automation_test_results=case[1]})
    assert_rejected(ok,value,events,began,case[2])
    assert(not trace(events):find('call:get_pie_status',1,true),trace(events))
  end
end)

test('existing PIE and malformed PIE status do not begin',function()
  local active={success=true,pie_running=true}
  local ok,value,events,began=run({catalog(NATIVE)},{get_pie_status=active})
  assert_rejected(ok,value,events,began,'fresh Editor already has PIE; never adopt it')
  assert(trace(events)=='list,call:ping,call:get_automation_test_results,call:get_pie_status',trace(events))
  for _,payload in ipairs{{success=true},{success=true,pie_running=0},{success=true,pie_running='false'},{success=false,pie_running=false}} do
    local failed,err,log,started_play=run({catalog(NATIVE)},{get_pie_status=payload})
    local reason=payload.success==false and 'PIE MCP negative result: get_pie_status' or 'PIE status malformed'
    assert_rejected(failed,err,log,started_play,reason)
  end
end)

test('malformed tools/list is not unwrapped and does not call tools',function()
  local payloads={
    42,'tools',json.null,
    {jsonrpc='2.0',id=1,result={tools={tool('ping')}}},
    {tools=json.null},{tools='ping'},{tools={name='ping'}},
    {tools={tool('ping'),tool('ping')}},
    {tools={[1]=tool('ping'),[3]=tool('get_pie_status')}},
    {tools={'ping'}},{tools={{inputSchema={type='object'}}}},{tools={{name=1}}},
    {tools={{name=''}}},
  }
  for _,payload in ipairs(payloads) do
    local ok,value,events,began=run({payload})
    assert_rejected(ok,value,events,began,'malformed tools/list')
    assert(trace(events)=='list',trace(events))
  end
  local events={}
  local began=false
  local ok,value=pcall(function()
    preflight.prepare(function() events[#events+1]={op='list'} return nil end,function(name)
      events[#events+1]={op='call',name=name}
      return {success=true}
    end)
    began=true
  end)
  assert_rejected(ok,value,events,began,'malformed tools/list')
end)

test('empty tools array fails closed before a tool call',function()
  local ok,value,events,began=run({catalog({})})
  assert_rejected(ok,value,events,began,'no suitable read-only capability')
  assert(trace(events)=='list',trace(events))
end)

test('negative or unstructured unlock does not enable an extension',function()
  local listed=catalog({'ping','enable_extension'})
  local ok,value,events,began=run({listed,listed},{ping={success=false}})
  assert_rejected(ok,value,events,began,'PIE MCP negative result: ping')
  assert(trace(events)=='list,call:ping',trace(events))
  local ok2,value2,events2,began2=run({listed,listed},{ping=function() return nil end})
  assert_rejected(ok2,value2,events2,began2,'PIE MCP lacks structured result: ping')
  assert(trace(events2)=='list,call:ping',trace(events2))
end)

test('failed enable_extension does not refresh or begin',function()
  local listed=catalog(concat({'ping','enable_extension'},REQUIRED))
  local ok,value,events,began=run({listed,listed},nil,{enable_extension='injected enable failure'})
  assert_rejected(ok,value,events,began,'injected enable failure')
  assert(trace(events)=='list,call:ping,call:enable_extension',trace(events))
end)

test('idle running is not treated as a passed-test report',function()
  local payload={success=true,running=0,framework_running=false,controller_state='disabled',passed=true,selected=4}
  local ok,report=run({catalog(NATIVE)},{get_automation_test_results=payload})
  assert(ok and report.passed==nil and report.automation_running==0,tostring(report))
end)

test('owned tools/list argv is the original record, --list, and evidence path',function()
  local root,owned,out='D:/omoba','D:/evidence/owned-editor.json','D:/evidence/mcp-001-tools-list.json'
  local args=preflight.owned_tools_list_args(root,owned,out)
  assert(#args==6)
  assert(args[1]==path.join(root,'scripts/ue_mcp.lua'))
  assert(args[2]=='--owned-editor' and args[3]==owned)
  assert(args[4]=='--list' and args[5]=='--out' and args[6]==out)
  for _,arg in ipairs(args) do
    assert(arg~='--list-tools' and arg~='--tool' and not tostring(arg):find('http',1,true))
  end
  assert(preflight.tools_list_evidence_name(1)=='mcp-001-tools-list.json')
  assert(preflight.tools_list_evidence_name(12)=='mcp-012-tools-list.json')
  for _,bad in ipairs{0,1.5,-1,'1',''} do
    assert(not pcall(preflight.tools_list_evidence_name,bad))
  end
  assert(not pcall(preflight.owned_tools_list_args,'',owned,out))
  assert(not pcall(preflight.owned_tools_list_args,root,'',out))
  assert(not pcall(preflight.owned_tools_list_args,root,owned,''))
end)

test('runner keeps begin after prepare and does not call enable_extension itself',function()
  local runner=path.read(path.join(b.root,'scripts/run_moba_pie_ue.lua'))
  local module_text=path.read(path.join(b.root,'scripts/ue_pie_preflight.lua'))
  local prepare_at=assert(runner:find('pie_preflight.prepare',1,true))
  local begin_at=assert(runner:find("mcp('begin_play_in_editor'",1,true))
  assert(prepare_at<begin_at)
  assert(runner:find('owned_tools_list_args',1,true) and runner:find('tools_list_evidence_name',1,true))
  assert(runner:find("num_clients=1,net_mode='standalone'",1,true))
  assert(runner:find('stop_play_in_editor',1,true) and runner:find('finish_timeout_seconds',1,true))
  assert(runner:find('selection_smoke_hero',1,true))
  assert(not runner:find("mcp('enable_extension'",1,true))
  assert(not runner:find('--list-tools',1,true))
  assert(not module_text:find("invoke(call,'begin_play_in_editor'",1,true))
  assert(not module_text:find('begin_play_in_editor(',1,true))
end)

print(('PIE capability preflight: %d scenarios passed; injected list/call order, no Editor/MCP/build'):format(count))
