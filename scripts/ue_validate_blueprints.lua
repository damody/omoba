-- Compile generated hero assets (or explicit packages) without rewriting graphs.
local source = debug.getinfo(1,'S').source:sub(2)
package.path = source:match('^(.*)[/\\]') .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path,json = b.lib('path'),b.lib('json')
local validation = require('ue_blueprint_validation')
local blueprints = require('ue_blueprint_assets')
local assets, seen, parents = {}, {}, {}
local create_missing = false
local function add(asset)
  asset = validation.package_path(asset)
  assert(not seen[asset], 'duplicate Blueprint: ' .. asset)
  seen[asset] = true; assets[#assets+1] = asset
end
for _, asset in ipairs(arg) do
  if asset == '--create-missing' then create_missing = true else add(asset) end
end
assert(not create_missing or #assets == 0, '--create-missing only accepts generated recipe assets')
if #assets == 0 then
  local recipe = json.read(path.join(b.root,'omfue/Plugins/OmRuntime/Source/OmGenerated/om_asset_recipe.json'))
  for _, hero in ipairs(recipe.heroes) do
    if hero.blueprint_path and hero.blueprint_path ~= '' then
      add(hero.blueprint_path)
      assert(hero.generated_class:match('^AOmHero[%w_]+$'), 'invalid generated hero class')
      parents[assets[#assets]] = hero.generated_class:sub(2)
    end
  end
  -- Existing compatibility widgets remain required until their native migration.
  -- Do not generate new UMG; compile only saved WBP packages in this exact folder.
  local ui=path.join(b.root,'omfue/Content/RustBP/UI')
  if path.is_directory(ui) then
    for file in require('lfs').dir(ui) do
      local name=file:match('^(WBP_[%w_]+)%.uasset$')
      if name then add('/Game/RustBP/UI/'..name) end
    end
  end
  table.sort(assets)
end
assert(#assets > 0, 'no Blueprint assets to validate')
local work = path.join(b.root,'target/blueprint-validation-runs','compile-' .. os.time())
assert(not path.exists(work), 'compile run already exists')
path.mkdir_p(work)
local report = {success=false,kind='unreal-blueprint-compile',assets={}}
local ordinal=0
local function call(tool,args)
  ordinal=ordinal+1
  local file=path.join(work,'op-'..ordinal..'-'..tool..'.json')
  assert(not path.exists(file), 'refusing stale response: '..file)
  local result=b.lib('process').run(b.lib('platform').lua_executable,{
    path.join(b.root,'scripts/ue_mcp.lua'),'--tool',tool,'--arguments',json.encode(json.object(args)),'--out',file,
  },{cwd=b.root,check=false})
  local value
  if path.is_file(file) then
    local parsed,envelope=pcall(json.read,file)
    if parsed then value=blueprints.unwrap_tool(envelope) end
  end
  if result.exit_code~=0 or not value or blueprints.is_explicit_negative(value) then
    local detail=value and blueprints.negative_message(value) or result.stderr
    error(tool..' failed; response: '..file..'; '..tostring(detail))
  end
  return value
end
local ok, failure = xpcall(function()
  local pie=call('get_pie_status',{})
  blueprints.require_idle(pie)
  local provider
  if create_missing then
    ordinal=ordinal+1
    local listed_file=path.join(work,'op-'..ordinal..'-tools-list.json')
    assert(not path.exists(listed_file), 'refusing stale tools/list response: '..listed_file)
    local listed=b.lib('process').run(b.lib('platform').lua_executable,{
      path.join(b.root,'scripts/ue_mcp.lua'),'--list','--out',listed_file,
    },{cwd=b.root,check=false})
    assert(listed.exit_code==0, 'tools/list failed; response: '..listed_file..'; '..listed.stderr)
    provider=blueprints.select_provider(json.read(listed_file))
  end
  for index,asset in ipairs(assets) do
    local pending={asset=asset,success=false,status='pending',diagnostics={}}
    report.assets[index]=pending
    json.write(path.join(work,'report.json'),report,true)
    local result
    if create_missing and parents[asset] then
      local outcome=blueprints.bootstrap(provider,asset,parents[asset],{
        call=call,
        on_created=function()
          pending.created=true
          json.write(path.join(work,'report.json'),report,true)
        end,
        compile=function(created)
          local compiled=validation.compile(b,asset,path.join(work,index..'-compile.json'))
          compiled.created=created
          report.assets[index]=compiled
          json.write(path.join(work,'report.json'),report,true)
          return compiled
        end,
      })
      result=outcome.compile
      if outcome.saved then result.saved=true end
      json.write(path.join(work,'report.json'),report,true)
    else
      result=validation.compile(b,asset,path.join(work,index..'-compile.json'))
      result.created=false
      report.assets[index]=result
      json.write(path.join(work,'report.json'),report,true)
    end
    assert(result.success, asset .. ': ' .. table.concat(result.diagnostics or {},'; '))
  end
  report.success = true
end,debug.traceback)
if not ok then
  report.error = tostring(failure)
  local last = report.assets[#report.assets]
  if last then
    last.success=false; last.status='pending'
    if #last.diagnostics == 0 then last.diagnostics={report.error} end
  end
end
json.write(path.join(work,'report.json'),report,true)
print('[ue-blueprints] success=' .. tostring(report.success) .. '; report: ' .. path.join(work,'report.json'))
assert(ok,failure)
