local source = debug.getinfo(1,'S').source:sub(2)
package.path = source:match('^(.*)[/\\]tests[/\\]') .. '/?.lua;' .. package.path
local v = require('ue_blueprint_validation')
local j = require('tools.lua.lib.json')
local asset = '/Game/OmAutomation/BP_Test'
local passed = 0
local function scenario(name, condition)
  passed = passed + 1
  assert(condition, name)
end
local function check(detail, exit, is_error)
  return v.result(asset, exit or 0, {isError=is_error, content={{type='text', text=j.encode(detail)}}})
end
local function incomplete(result)
  return not result.success and result.status == 'pending'
    and result.diagnostics[1] == 'compile status incomplete or requires action'
end
local function native_clean()
  return {
    blueprint_path = asset,
    success = true,
    tool = 'compile_blueprint',
    compile_status = 'compiled',
    error_count = 0,
    warning_count = 0,
    errors = {},
    warnings = {},
  }
end
local good = {blueprint_path=asset, ok=true, error_count=0, errors={}}
local legacy = check(good)
scenario('ok-only legacy provider compiles', legacy.success and legacy.status == 'compiled'
  and legacy.compiler.ok == true and legacy.compiler.success == nil and legacy.asset == asset)
scenario('nonzero exit rejects', not check(good, 1).success)
scenario('response isError rejects', not check(good, 0, true).success)
local missing = v.result(asset, 0, {content={}})
scenario('missing structured result rejects', not missing.success
  and missing.diagnostics[1] == 'missing structured Blueprint compile result')
scenario('nil response rejects', not v.result(asset, 1, nil).success)
scenario('missing error_count rejects', not check({blueprint_path=asset, ok=true}).success)
local other = check({blueprint_path='/Game/Other', ok=true, error_count=0})
scenario('other asset path rejects', not other.success
  and other.diagnostics[1] == 'compiler result belongs to another asset')
local bad = check({blueprint_path=asset, ok=false, error_count=1, errors={'Target required'}}, 1, true)
scenario('compiler error stays pending', not bad.success and bad.status == 'pending' and bad.diagnostics[1] == 'Target required')
scenario('nonempty errors reject', not check({blueprint_path=asset, ok=true, error_count=0, errors={'contradiction'}}).success)
scenario('health issues reject', not check({blueprint_path=asset, ok=true, error_count=0, health_issues={{kind='orphan'}}}).success)
scenario('action_required rejects', not check({blueprint_path=asset, ok=true, error_count=0, action_required=true}).success)
scenario('ok true and success false rejects', incomplete(check({blueprint_path=asset, ok=true, success=false, error_count=0})))
scenario('class object path normalizes', v.package_path(asset .. '.BP_Test_C') == asset)
scenario('mismatched object path rejects', not pcall(v.package_path, asset .. '.BP_Other_C'))
scenario('parent segment rejects', not pcall(v.package_path, '/Game/../BP_Test'))
scenario('empty segment rejects', not pcall(v.package_path, '/Game//BP_Test'))
scenario('filesystem path rejects', not pcall(v.package_path, 'D:/code/omoba/BP_Test'))

local native = check(native_clean())
scenario('success-only native provider compiles', native.success and native.status == 'compiled'
  and #native.diagnostics == 0 and native.compiler.success == true and native.compiler.ok == nil
  and native.asset == asset and native.exit_code == 0)
scenario('both explicit positives compile', check({
  blueprint_path=asset, ok=true, success=true, error_count=0, errors={}, health_issues={},
}).success)
scenario('success true and ok false rejects', incomplete(check({
  blueprint_path=asset, ok=false, success=true, error_count=0, errors={},
})))
scenario('clean legacy ok false rejects', incomplete(check({blueprint_path=asset, ok=false, error_count=0, errors={}})))
scenario('clean native success false rejects', incomplete(check({blueprint_path=asset, success=false, error_count=0, errors={}})))
scenario('both flags absent rejects', incomplete(check({blueprint_path=asset, error_count=0, errors={}, health_issues={}})))

local function malformed(detail)
  return incomplete(check(detail))
end
scenario('string ok rejects', malformed({blueprint_path=asset, ok='true', error_count=0, errors={}}))
scenario('unknown ok rejects', malformed({blueprint_path=asset, ok='unknown', error_count=0, errors={}}))
scenario('numeric ok rejects', malformed({blueprint_path=asset, ok=1, error_count=0, errors={}}))
scenario('zero ok rejects beside success', malformed({blueprint_path=asset, ok=0, success=true, error_count=0, errors={}}))
scenario('null ok rejects beside success', malformed({blueprint_path=asset, ok=j.null, success=true, error_count=0, errors={}}))
scenario('empty ok table rejects', malformed({blueprint_path=asset, ok={}, success=true, error_count=0, errors={}}))
scenario('string success rejects', malformed({blueprint_path=asset, success='true', error_count=0, errors={}}))
scenario('unknown success rejects', malformed({blueprint_path=asset, success='unknown', error_count=0, errors={}}))
scenario('numeric success rejects', malformed({blueprint_path=asset, success=1, error_count=0, errors={}}))
scenario('zero success rejects beside ok', malformed({blueprint_path=asset, ok=true, success=0, error_count=0, errors={}}))
scenario('null success rejects beside ok', malformed({blueprint_path=asset, ok=true, success=j.null, error_count=0, errors={}}))
scenario('string success rejects beside ok', malformed({blueprint_path=asset, ok=true, success='true', error_count=0, errors={}}))

local function native_mutated(mutate, exit, is_error)
  local detail = native_clean()
  mutate(detail)
  return check(detail, exit, is_error)
end
scenario('native nonzero exit rejects', not native_mutated(function() end, 1).success)
scenario('native isError rejects', not native_mutated(function() end, 0, true).success)
scenario('native missing error_count rejects', not native_mutated(function(detail) detail.error_count = nil end).success)
scenario('native string error_count rejects', not native_mutated(function(detail) detail.error_count = '0' end).success)
scenario('native error_count rejects', not native_mutated(function(detail)
  detail.error_count = 1
  detail.errors = {'pin type'}
end).success)
local native_errors = native_mutated(function(detail) detail.errors = {'contradiction'} end)
scenario('native nonempty errors reject', not native_errors.success and native_errors.diagnostics[1] == 'contradiction')
scenario('native health issues reject', not native_mutated(function(detail)
  detail.health_issues = {{kind='orphan'}}
end).success)
scenario('native action_required object rejects', not native_mutated(function(detail)
  detail.action_required = {tool='clear_blueprint_graph'}
end).success)
local native_other = native_mutated(function(detail) detail.blueprint_path = '/Game/Other' end)
scenario('native other asset rejects', not native_other.success
  and native_other.diagnostics[1] == 'compiler result belongs to another asset')
local same_class = native_mutated(function(detail) detail.blueprint_path = asset .. '.BP_Test_C' end)
scenario('native class suffix keeps asset identity', same_class.success and same_class.asset == asset)

local gate = {success=false, error_code='mcp_runtime_gate_failed',
  error='MCP runtime gate blocked compile_blueprint: telemetry cache is not currently valid.',
  diagnostics={telemetry_cache_ok=false}, original_response=native_clean()}
local blocked = check(gate, 1, true)
scenario('runtime gate preserves diagnostic and rejects nested compiler success',
  not blocked.success and blocked.status=='pending' and blocked.compiler==nil
  and blocked.provider_error_code==gate.error_code
  and blocked.provider_diagnostics.telemetry_cache_ok==false
  and blocked.diagnostics[1]==gate.error_code .. ': ' .. gate.error)
for _, reverse in ipairs({false,true}) do
  local content={{type='text',text=j.encode(gate)},{type='text',text=j.encode(native_clean())}}
  if reverse then content[1],content[2]=content[2],content[1] end
  local result=v.result(asset,0,{content=content})
  scenario('separate successful content cannot override rejection '..tostring(reverse),
    not result.success and result.provider_error_code==gate.error_code)
end
local code_only=check({error_code='unavailable',original_response=native_clean()})
scenario('code-only wrapper rejects instead of accepting original response',
  not code_only.success and code_only.diagnostics[1]=='unavailable: provider rejected Blueprint compile')
local malformed_gate=check({success=false,error={},error_code=7,original_response=native_clean()})
scenario('malformed negative diagnostic stays rejected',not malformed_gate.success
  and malformed_gate.diagnostics[1]=='provider rejected Blueprint compile')

print('Blueprint compile boundary: ' .. passed .. ' scenarios passed')
