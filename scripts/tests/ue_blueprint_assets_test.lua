local source = debug.getinfo(1, 'S').source:sub(2)
local scripts = assert(source:match('^(.*)[/\\]tests[/\\]'))
package.path = scripts .. '/?.lua;' .. package.path
local M = require('ue_blueprint_assets')
local validation = require('ue_blueprint_validation')
local j = require('tools.lua.lib.json')

local asset = '/Game/RustBP/Heroes/BP_SaikaMagoichi'
local recipe = asset .. '.BP_SaikaMagoichi_C'
local folder = '/Game/RustBP/Heroes'
local name = 'BP_SaikaMagoichi'
local parent = 'OmHeroSaikaMagoichi'
local parent_path = '/Script/OmGenerated.' .. parent
local object_path = asset .. '.' .. name
local passed = 0

local function scenario(label, condition)
  passed = passed + 1
  assert(condition, label)
end

local function fails(label, fn)
  passed = passed + 1
  local ok, err = pcall(fn)
  assert(not ok, label)
  return tostring(err)
end

local function has(err, text)
  return tostring(err):find(text, 1, true) ~= nil
end

local function same(actual, expected)
  if type(actual) ~= type(expected) then return false end
  if type(actual) ~= 'table' then return actual == expected end
  if actual == j.null or expected == j.null then return actual == expected end
  local function count(value)
    local total = 0
    for _ in pairs(value) do total = total + 1 end
    return total
  end
  if count(actual) ~= count(expected) then return false end
  for key, value in pairs(expected) do
    if not same(actual[key], value) then return false end
  end
  return true
end

local function order(events)
  local names = {}
  for _, event in ipairs(events) do names[#names + 1] = event.tool end
  return table.concat(names, ',')
end

local function wrote(events)
  for _, event in ipairs(events) do
    if event.tool == 'create_blueprint' or event.tool == 'create_asset'
      or event.tool == 'save_asset' or event.tool == 'save_assets'
      or event.tool == 'created' then
      return true
    end
  end
  return false
end

local function attempt(provider, tools, compile_result, asset_path)
  local events = {}
  local ok, outcome = xpcall(function()
    return M.bootstrap(provider, asset_path or asset, parent, {
      call = function(tool, args)
        events[#events + 1] = {tool = tool, args = args}
        local response = tools[tool]
        if response == nil then error('unexpected tool ' .. tool) end
        if type(response) == 'function' then return response(args) end
        return response
      end,
      on_created = function() events[#events + 1] = {tool = 'created'} end,
      compile = function(created)
        events[#events + 1] = {tool = 'compile', created = created}
        if type(compile_result) == 'function' then return compile_result(created) end
        return compile_result or {success = true, diagnostics = {}}
      end,
    })
  end, debug.traceback)
  return ok, outcome, events
end

local function entry(entry_name, entry_path, class)
  return {
    name = entry_name,
    path = entry_path or (folder .. '/' .. entry_name .. '.' .. entry_name),
    class = class or 'Blueprint',
  }
end

local function listing(rows, count, folder_path)
  return {
    folder_path = folder_path or folder,
    assets = rows,
    count = count == nil and #rows or count,
  }
end

local function native_create(path, created_name, asset_path_field)
  local value = {
    success = true,
    blueprint_path = path or object_path,
    name = created_name or name,
    note = 'EventGraph is empty',
  }
  if asset_path_field ~= nil then value.asset_path = asset_path_field end
  return value
end

local function native_parent(short, path, blueprint)
  return {
    success = true,
    summary = 'Parent Class: ' .. (short or parent),
    blueprint_path = blueprint or object_path,
    parent_class = short or parent,
    parent_class_path = path or parent_path,
    metadata_version = 1,
  }
end

local function native_save()
  return {
    success = true,
    asset_path = object_path,
    normalized_asset_path = recipe,
    asset_class = 'Blueprint',
  }
end

local function native_tools(list)
  return {
    list_assets = list,
    create_blueprint = native_create(),
    get_blueprint_summary = native_parent(),
    save_asset = native_save(),
  }
end

local function catalog(names)
  local tools = {}
  for _, tool_name in ipairs(names) do
    tools[#tools + 1] = {name = tool_name, inputSchema = {type = 'object'}}
  end
  return {tools = tools}
end

scenario('generated parent path', M.parent_path(parent) == parent_path)
fails('leading A is not the short parent', function() M.parent_path('AOmHeroSaikaMagoichi') end)
fails('bare prefix is not a parent', function() M.parent_path('OmHero') end)
fails('foreign class is not a generated parent', function() M.parent_path('Actor') end)
scenario('recipe object path canonicalizes', validation.package_path(recipe) == asset)

local present = listing({entry(name, object_path)})
local ok, outcome, events = attempt('native', native_tools(present))
scenario('existing blueprint is not created or saved', ok and outcome.created == false and outcome.saved == false
  and outcome.compile.success == true and order(events) == 'list_assets,get_blueprint_summary,compile'
  and events[#events].created == false)
scenario('list asks for the whole folder', same(events[1].args, {folder_path = folder, recursive = false}))
scenario('summary asks for the canonical asset', same(events[2].args, {blueprint_path = asset}))

local decoded_present = j.decode('{"folder_path":"/Game/RustBP/Heroes","count":1,"assets":[{"name":"BP_SaikaMagoichi","path":"/Game/RustBP/Heroes/BP_SaikaMagoichi.BP_SaikaMagoichi","class":"Blueprint"}]}')
scenario('native list needs no success flag', decoded_present.success == nil
  and not M.is_explicit_negative(decoded_present)
  and M.classify_native(folder, asset, decoded_present) == 'present')
local decoded_absent = j.decode('{"folder_path":"/Game/RustBP/Heroes","assets":[],"count":0}')
scenario('complete empty folder is absence', not M.is_explicit_negative(decoded_absent)
  and M.classify_native(folder, asset, decoded_absent) == 'missing')

ok, outcome, events = attempt('native', native_tools(listing({})), nil, recipe)
scenario('absence creates, checks parent, compiles, then saves', ok and outcome.created == true and outcome.saved == true
  and order(events) == 'list_assets,create_blueprint,created,get_blueprint_summary,compile,save_asset'
  and events[5].created == true)
scenario('native create uses the full parent path', same(events[2].args, {
  name = name, parent_class = parent_path, save_path = folder,
}))
scenario('native save uses the canonical asset', same(events[6].args, {asset_path = asset}))
local agreed = native_tools(listing({}))
agreed.create_blueprint = native_create(object_path, name, asset)
ok = attempt('native', agreed)
scenario('agreeing asset_path does not reject creation', ok)

local function rejected_create(label, response, fragment)
  local tools = native_tools(listing({}))
  tools.create_blueprint = response
  local succeeded, err, trace = attempt('native', tools)
  scenario(label, not succeeded and has(err, fragment) and order(trace) == 'list_assets,create_blueprint')
end
rejected_create('renamed blueprint path rejects', native_create('/Game/Other/BP_Other.BP_Other', 'BP_Other'),
  'creation returned another asset')
rejected_create('asset_path without blueprint_path rejects',
  {success = true, asset_path = asset, name = name}, 'creation returned another asset')
rejected_create('conflicting asset_path rejects', native_create(object_path, name, '/Game/Other/BP_Other'),
  'creation returned another asset')
rejected_create('create name mismatch rejects', native_create(object_path, 'BP_Other'),
  'creation returned another asset')
rejected_create('negative create rejects', {success = false, error = 'exists', blueprint_path = object_path, name = name,
  original_response = native_create()}, 'blueprint creation rejected')
rejected_create('missing create status rejects', {blueprint_path = object_path, name = name},
  'blueprint creation status unknown')
rejected_create('string create status rejects', {success = 'true', blueprint_path = object_path, name = name},
  'blueprint creation rejected')

local wrong_parent = native_tools(present)
wrong_parent.get_blueprint_summary = native_parent('OmHeroDateMasamune', '/Script/OmGenerated.OmHeroDateMasamune')
ok, outcome, events = attempt('native', wrong_parent)
scenario('wrong direct parent rejects before compile', not ok and has(outcome, 'mismatched')
  and order(events) == 'list_assets,get_blueprint_summary' and not wrote(events))
local text_only = native_tools(present)
text_only.get_blueprint_summary = {success = true, summary = 'Parent Class: ' .. parent}
ok, outcome, events = attempt('native', text_only)
scenario('summary text is not parent metadata', not ok and has(outcome, 'parent status unknown')
  and order(events) == 'list_assets,get_blueprint_summary')
local version = native_tools(present)
version.get_blueprint_summary = native_parent()
version.get_blueprint_summary.metadata_version = 2
ok, outcome, events = attempt('native', version)
scenario('unexpected metadata version rejects', not ok and order(events) == 'list_assets,get_blueprint_summary')
local short_only = native_tools(present)
short_only.get_blueprint_summary = native_parent(parent, '/Script/Engine.OmHeroSaikaMagoichi')
ok, outcome, events = attempt('native', short_only)
scenario('parent path must be the generated class', not ok and has(outcome, 'mismatched') and not wrote(events))
local nested_parent = {
  success = false,
  error_code = 'mcp_runtime_gate_failed',
  error = 'telemetry cache is not currently valid',
  original_response = native_parent(),
}
local nested_tools = native_tools(present)
nested_tools.get_blueprint_summary = nested_parent
ok, outcome, events = attempt('native', nested_tools)
scenario('negative parent wrapper cannot supply metadata', not ok and has(outcome, 'blueprint parent rejected')
  and not has(outcome, parent_path) and not wrote(events))
scenario('negative message keeps the gate', M.negative_message(nested_parent)
  == 'mcp_runtime_gate_failed: telemetry cache is not currently valid')
local envelope = {isError = true, content = {
  {type = 'text', text = j.encode(nested_parent)},
  {type = 'text', text = j.encode(native_parent())},
}}
local unwrapped = M.unwrap_tool(envelope)
scenario('unwrap keeps the first negative body', unwrapped.success == false and unwrapped.parent_class == nil
  and unwrapped.original_response.parent_class == parent)
scenario('content map is not a tool result', M.unwrap_tool({content = {type = 'text', text = '{}'}}) == nil)

local function rejected_list(label, list, fragment)
  local succeeded, err, trace = attempt('native', native_tools(list))
  scenario(label, not succeeded and has(err, fragment) and order(trace) == 'list_assets' and not wrote(trace))
end
rejected_list('wrong class rejects', listing({entry(name, object_path, 'WidgetBlueprint')}), 'non-blueprint')
rejected_list('lowercase class rejects', listing({entry(name, object_path, 'blueprint')}), 'non-blueprint')
rejected_list('foreign folder header rejects', listing({}, 0, '/Game/Other'), 'foreign folder')
rejected_list('foreign asset row rejects', listing({entry('BP_Other', '/Game/Other/BP_Other.BP_Other')}), 'foreign folder')
rejected_list('foreign row hides a real asset', listing({
  entry('BP_Other', '/Game/Other/BP_Other.BP_Other'), entry(name, object_path),
}), 'foreign folder')
rejected_list('count mismatch rejects', listing({entry(name, object_path)}, 2), 'count mismatch')
do
  local flagged = listing({entry(name, object_path)}, 0)
  flagged.success = true
  rejected_list('success flag cannot repair a count', flagged, 'count mismatch')
end
rejected_list('duplicate asset path rejects', listing({entry(name, object_path), entry(name, recipe)}), 'duplicate')
rejected_list('malformed asset row rejects', listing({{name = 'BP_Other', path = folder .. '/BP_Other.BP_Other'}}),
  'malformed asset inventory')
rejected_list('identity mismatch rejects', listing({{
  name = name, path = folder .. '/BP_Other.BP_Other', class = 'Blueprint',
}}), 'malformed asset identity')
rejected_list('filesystem asset path rejects', listing({{
  name = name, path = 'D:/omoba/BP_SaikaMagoichi', class = 'Blueprint',
}}), 'malformed asset inventory')
rejected_list('null asset list rejects', {folder_path = folder, assets = j.null, count = 0}, 'malformed asset inventory')
rejected_list('sparse asset list rejects', {
  folder_path = folder,
  assets = {[1] = entry('BP_Other'), [3] = entry('BP_Third')},
  count = 2,
}, 'malformed asset inventory')
rejected_list('old folder list shape rejects', {
  success = true, folder = folder, recursive = false, count = 0, assets = {},
}, 'foreign folder')
rejected_list('negative list cannot masquerade as inventory', {
  success = false,
  error = 'telemetry cache is not currently valid',
  original_response = present,
}, 'asset inventory rejected')
rejected_list('null error rejects inventory', {folder_path = folder, assets = {}, count = 0, error = j.null},
  'asset inventory rejected')
rejected_list('false ok rejects inventory', {folder_path = folder, assets = {}, count = 0, ok = false, success = true},
  'asset inventory rejected')

ok, outcome, events = attempt('native', native_tools(listing({entry('BP_Other')})))
scenario('another blueprint does not block creation', ok and outcome.created == true
  and same(events[2].args, {name = name, parent_class = parent_path, save_path = folder}))

local dirty = native_tools(listing({}))
ok, outcome, events = attempt('native', dirty, {success = false, diagnostics = {'pin type'}})
scenario('failed compile is not saved', not ok and has(outcome, 'pin type')
  and order(events) == 'list_assets,create_blueprint,created,get_blueprint_summary,compile')
ok, outcome, events = attempt('native', native_tools(listing({})), {success = true, diagnostics = {'graph health: orphan'}})
scenario('compiler diagnostics block save', not ok and has(outcome, 'graph health: orphan')
  and order(events) == 'list_assets,create_blueprint,created,get_blueprint_summary,compile')
ok, outcome, events = attempt('native', native_tools(listing({})), function() return nil end)
scenario('missing compile result rejects before save', not ok and has(outcome, 'compile failed')
  and order(events) == 'list_assets,create_blueprint,created,get_blueprint_summary,compile')

local function rejected_save(label, response, fragment)
  local tools = native_tools(listing({}))
  tools.save_asset = response
  local succeeded, err, trace = attempt('native', tools)
  scenario(label, not succeeded and has(err, fragment)
    and order(trace) == 'list_assets,create_blueprint,created,get_blueprint_summary,compile,save_asset')
end
rejected_save('negative save rejects', {
  success = false, error_code = 'save_failed', error = 'could not save',
  asset_path = asset, normalized_asset_path = asset, asset_class = 'Blueprint',
  original_response = native_save(),
}, 'asset save rejected')
rejected_save('missing save status rejects', {
  asset_path = asset, normalized_asset_path = asset, asset_class = 'Blueprint',
}, 'asset save status unknown')
rejected_save('string save status rejects', {
  success = 'true', asset_path = asset, normalized_asset_path = asset, asset_class = 'Blueprint',
}, 'asset save rejected')
rejected_save('save error overrides success', {
  success = true, error = 'gate', asset_path = object_path, normalized_asset_path = recipe, asset_class = 'Blueprint',
}, 'asset save rejected')
rejected_save('save class mismatch rejects', {
  success = true, asset_path = object_path, normalized_asset_path = recipe, asset_class = 'WidgetBlueprint',
}, 'asset save class mismatch')
rejected_save('save path mismatch rejects', {
  success = true, asset_path = '/Game/Other/BP_Other', normalized_asset_path = recipe, asset_class = 'Blueprint',
}, 'asset save identity mismatch')
rejected_save('normalized save path mismatch rejects', {
  success = true, asset_path = object_path, normalized_asset_path = '/Game/Other/BP_Other', asset_class = 'Blueprint',
}, 'asset save identity mismatch')
rejected_save('missing save class rejects', {
  success = true, asset_path = object_path, normalized_asset_path = recipe,
}, 'asset save class mismatch')

local legacy_absent = {
  check_asset_exists = {exists = false},
  create_asset = {asset_path = object_path},
  get_blueprint_skeleton = {parent_class = parent},
  save_assets = {failed = 0, saved = 1},
}
ok, outcome, events = attempt('legacy', legacy_absent)
scenario('legacy absence preserves create, parent, compile, save', ok and outcome.created == true and outcome.saved == true
  and order(events) == 'check_asset_exists,create_asset,created,get_blueprint_skeleton,compile,save_assets')
scenario('legacy existence uses the canonical asset', same(events[1].args, {asset_path = asset}))
scenario('legacy create keeps the options shape', same(events[2].args, {
  asset_type = 'Blueprint', name = name, save_path = folder, options = {parent_class = parent_path},
}))
local legacy_wire = j.decode(j.encode(events[2].args))
scenario('legacy options encode as an object', legacy_wire.options.parent_class == parent_path
  and legacy_wire.asset_type == 'Blueprint')
scenario('legacy skeleton uses the canonical asset', same(events[4].args, {blueprint_path = asset}))
scenario('legacy save requests one asset', same(events[6].args, {asset_paths = {asset}}))

local legacy_present = {
  check_asset_exists = {exists = true},
  get_blueprint_skeleton = {parent_class = parent},
  create_asset = function() error('legacy create called') end,
  save_assets = function() error('legacy save called') end,
}
ok, outcome, events = attempt('legacy', legacy_present)
scenario('existing legacy blueprint is not created or saved', ok and outcome.created == false and outcome.saved == false
  and order(events) == 'check_asset_exists,get_blueprint_skeleton,compile')

local function rejected_legacy(label, tools, fragment, expected)
  local succeeded, err, trace = attempt('legacy', tools)
  scenario(label, not succeeded and has(err, fragment) and order(trace) == expected and not wrote(trace))
end
rejected_legacy('unknown existence rejects', {check_asset_exists = {}}, 'asset existence unknown', 'check_asset_exists')
rejected_legacy('string existence rejects', {check_asset_exists = {exists = 'false'}}, 'asset existence unknown',
  'check_asset_exists')
rejected_legacy('negative existence rejects', {
  check_asset_exists = {success = false, exists = false, original_response = {exists = false}},
}, 'asset existence rejected', 'check_asset_exists')
rejected_legacy('legacy wrong parent rejects', {
  check_asset_exists = {exists = true},
  get_blueprint_skeleton = {parent_class = parent_path},
}, 'mismatched', 'check_asset_exists,get_blueprint_skeleton')

local legacy_bad_create = {
  check_asset_exists = {exists = false},
  create_asset = {success = false, error = 'denied', asset_path = object_path, original_response = {asset_path = asset}},
}
ok, outcome, events = attempt('legacy', legacy_bad_create)
scenario('negative legacy create rejects', not ok and has(outcome, 'blueprint creation rejected')
  and order(events) == 'check_asset_exists,create_asset')
local legacy_bad_save = {
  check_asset_exists = {exists = false},
  create_asset = {asset_path = recipe},
  get_blueprint_skeleton = {parent_class = parent},
  save_assets = {success = false, failed = 0, saved = 1, error = 'denied'},
}
ok, outcome, events = attempt('legacy', legacy_bad_save)
scenario('negative legacy save rejects', not ok and has(outcome, 'asset save rejected')
  and order(events) == 'check_asset_exists,create_asset,created,get_blueprint_skeleton,compile,save_assets')
legacy_bad_save.save_assets = {failed = 1, saved = 0}
ok, outcome, events = attempt('legacy', legacy_bad_save)
scenario('legacy failed save rejects', not ok and has(outcome, 'asset save incomplete'))
legacy_bad_save.save_assets = {failed = 0, saved = 2}
ok, outcome, events = attempt('legacy', legacy_bad_save)
scenario('legacy extra save count rejects', not ok and has(outcome, 'asset save incomplete'))
legacy_bad_save.save_assets = {failed = 0, saved = '1'}
ok, outcome, events = attempt('legacy', legacy_bad_save)
scenario('legacy string save count rejects', not ok and has(outcome, 'asset save incomplete'))

local calls = 0
local err = fails('incomplete provider rejects', function()
  local provider = M.select_provider(catalog({'list_assets', 'save_asset', 'check_asset_exists'}))
  M.bootstrap(provider, asset, parent, {
    call = function() calls = calls + 1 end,
    compile = function() calls = calls + 1 return {success = true, diagnostics = {}} end,
  })
end)
scenario('incomplete provider writes nothing', calls == 0 and has(err, 'create_blueprint') and has(err, 'create_asset'))
err = fails('summary alias is not a native capability', function()
  M.select_provider(catalog({'list_assets', 'create_blueprint', 'get_detailed_blueprint_summary', 'save_asset'}))
end)
scenario('alias remains a missing summary tool', has(err, 'get_blueprint_summary'))
fails('duplicate tool name rejects', function()
  M.select_provider({tools = {{name = 'list_assets'}, {name = 'list_assets'}}})
end)
fails('empty tool name rejects', function() M.select_provider({tools = {{name = ''}}}) end)
fails('null tool name rejects', function() M.select_provider({tools = {{name = j.null}}}) end)
fails('null tools payload rejects', function() M.select_provider(j.null) end)
fails('tools object rejects', function() M.select_provider({tools = {name = 'list_assets'}}) end)
scenario('complete native catalog wins', M.select_provider(catalog({
  'list_assets', 'create_blueprint', 'get_blueprint_summary', 'save_asset',
  'check_asset_exists', 'create_asset', 'get_blueprint_skeleton', 'save_assets',
})) == 'native')
scenario('complete legacy catalog remains available', M.select_provider(catalog({
  'check_asset_exists', 'create_asset', 'get_blueprint_skeleton', 'save_assets', 'compile_blueprint',
})) == 'legacy')
calls = 0
err = fails('unknown provider rejects', function()
  M.bootstrap('vendor', asset, parent, {
    call = function() calls = calls + 1 end,
    compile = function() calls = calls + 1 return {success = true, diagnostics = {}} end,
  })
end)
scenario('unknown provider writes nothing', calls == 0 and has(err, 'unknown blueprint asset provider'))

-- Negative content and envelope errors dominate all positive metadata.
local positive = {type='text',text=j.encode(native_parent())}
local negative = {type='text',text=j.encode(nested_parent)}
for _, contents in ipairs({{positive,negative},{negative,positive}}) do
  local value=M.unwrap_tool({isError=false,content=contents})
  scenario('negative dominates content order', M.is_explicit_negative(value)
    and value.error_code=='mcp_runtime_gate_failed')
end
scenario('error envelope rejects positive metadata', M.is_explicit_negative(
  M.unwrap_tool({isError=true,content={positive}})))
scenario('malformed error envelope rejects positive metadata', M.is_explicit_negative(
  M.unwrap_tool({isError='false',content={positive}})))
scenario('valid positive envelope accepted', M.unwrap_tool({isError=false,content={positive}}).parent_class==parent)
fails('negative catalog cannot advertise capabilities',function()
  local payload=catalog(M.native_tools); payload.success=false; M.select_provider(payload)
end)
for _, value in ipairs({{success=true,pie_running=false},{ok=true,pie_running=false}}) do
  scenario('explicit provider idle accepted',pcall(M.require_idle,value))
end
for _, value in ipairs({{}, {success=true}, {success=true,pie_running=true},
  {success=true,pie_running='false'}, {success=true,ok=false,pie_running=false},
  {success='true',pie_running=false}, {success=true,pie_running=false,error_code='blocked'}}) do
  fails('unknown or rejected idle blocks writes',function() M.require_idle(value) end)
end
print('ue_blueprint_assets_test: ' .. passed .. ' passed')
