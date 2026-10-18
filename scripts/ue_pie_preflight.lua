-- Capability negotiation for one owned Editor. Callers start PIE only after prepare returns.
-- list/call are injected. This module never starts PIE and never invents a tool result.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path,json=b.lib('path'),b.lib('json')
local M={}

-- Native tools/list advertises ping. The other three are optional advertised reads.
M.read_only_candidates={'ping','get_editor_info','get_project_info','get_pie_status'}
M.required_tools={
  'get_pie_status','get_pie_actors','call_pie_blueprint_function','take_pie_screenshot',
  'begin_play_in_editor','stop_play_in_editor','get_automation_test_results',
}

local function is_null(value) return value==json.null end

local function is_sequence(value)
  if type(value)~='table' or is_null(value) then return false end
  local meta=getmetatable(value)
  if meta and meta.__json_object then return false end
  local count,max=0,0
  for key in pairs(value) do
    if type(key)~='number' or key<1 or key%1~=0 then return false end
    count=count+1
    if key>max then max=key end
  end
  return max==count
end

function M.capability_names(payload)
  if type(payload)~='table' or is_null(payload) or not is_sequence(payload.tools) then
    error('malformed tools/list')
  end
  local names={}
  for _,entry in ipairs(payload.tools) do
    if type(entry)~='table' or is_null(entry) or is_sequence(entry)
      or type(entry.name)~='string' or entry.name=='' or names[entry.name] then
      error('malformed tools/list')
    end
    names[entry.name]=true
  end
  return names
end

function M.owned_tools_list_args(root,owned_editor,out_file)
  assert(type(root)=='string' and root~='','owned tools/list requires clone root')
  assert(type(owned_editor)=='string' and owned_editor~='','owned tools/list requires original editor record')
  assert(type(out_file)=='string' and out_file~='','owned tools/list requires evidence path')
  return {
    path.join(root,'scripts/ue_mcp.lua'),
    '--owned-editor',owned_editor,
    '--list',
    '--out',out_file,
  }
end

function M.tools_list_evidence_name(ordinal)
  assert(type(ordinal)=='number' and ordinal>=1 and ordinal%1==0 and ordinal<100000,
    'tools/list evidence ordinal malformed')
  return ('mcp-%03d-tools-list.json'):format(ordinal)
end

local function invoke(call,name,args)
  local value=call(name,args)
  assert(type(value)=='table' and not is_null(value),'PIE MCP lacks structured result: '..name)
  assert(value.success~=false and value.ok~=false and value.verdict~='FAIL','PIE MCP negative result: '..name)
  return value
end

function M.prepare(list,call)
  assert(type(list)=='function' and type(call)=='function','PIE preflight requires list and call callbacks')
  local names=M.capability_names(list())
  local read_only
  for _,candidate in ipairs(M.read_only_candidates) do
    if names[candidate] then read_only=candidate break end
  end
  assert(read_only,'no suitable read-only capability')
  invoke(call,read_only,{})
  local extension_activated=false
  if names.enable_extension then
    invoke(call,'enable_extension',{extension_id='Testing',enabled=true})
    extension_activated=true
    names=M.capability_names(list())
  end
  local missing={}
  for _,required in ipairs(M.required_tools) do
    if not names[required] then missing[#missing+1]=required end
  end
  assert(#missing==0,'missing PIE capability: '..table.concat(missing,','))
  local automation=invoke(call,'get_automation_test_results',{})
  if automation.running==nil then error('automation running state unknown') end
  if type(automation.running)~='number' or automation.running<0 or automation.running%1~=0 then
    error('automation running state malformed')
  end
  -- Older extension providers report a count; native providers report a busy bit.
  -- Optional native details must never turn a known busy/unknown source into idle.
  if automation.framework_running~=nil then
    assert(type(automation.framework_running)=='boolean','automation framework state malformed')
    assert(not automation.framework_running,'automation running in owned Editor')
  end
  if automation.controller_state~=nil then
    assert(type(automation.controller_state)=='string' and automation.controller_state~='',
      'automation controller state malformed')
    assert(automation.controller_state:lower()~='unknown','automation controller state unknown')
    assert(automation.controller_state:lower()~='running','automation running in owned Editor')
  end
  assert(automation.running==0,'automation running in owned Editor')
  local pie=invoke(call,'get_pie_status',{})
  if type(pie.pie_running)~='boolean' then error('PIE status malformed') end
  assert(pie.pie_running==false,'fresh Editor already has PIE; never adopt it')
  return {
    read_only_tool=read_only,
    extension_activated=extension_activated,
    automation_running=automation.running,
    pie_running=pie.pie_running,
  }
end

return M
