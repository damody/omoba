-- Compile generated hero assets (or explicit packages) without rewriting graphs.
local source = debug.getinfo(1,'S').source:sub(2)
package.path = source:match('^(.*)[/\\]') .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path,json = b.lib('path'),b.lib('json')
local validation = require('ue_blueprint_validation')
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
  local result=b.lib('process').run(b.lib('platform').lua_executable,{
    path.join(b.root,'scripts/ue_mcp.lua'),'--tool',tool,'--arguments',json.encode(json.object(args)),'--out',file,
  },{cwd=b.root,check=false})
  assert(result.exit_code==0, tool..' failed; response: '..file..'; '..result.stderr)
  for _,content in ipairs(json.read(file).content or {}) do
    if content.type=='text' then
      local parsed,value=pcall(json.decode,content.text)
      if parsed and type(value)=='table' then return value end
    end
  end
  error('missing structured result: '..tool)
end
local ok, failure = xpcall(function()
  assert(not call('get_pie_status',{}).pie_running,'stop PIE before compiling or creating assets')
  for index,asset in ipairs(assets) do
    local pending={asset=asset,success=false,status='pending',diagnostics={}}
    report.assets[index]=pending
    json.write(path.join(work,'report.json'),report,true)
    local created=false
    if create_missing and parents[asset] then
      if not call('check_asset_exists',{asset_path=asset}).exists then
        local folder,name=asset:match('^(.*)/([^/]+)$')
        local value=call('create_asset',{asset_type='Blueprint',name=name,save_path=folder,
          options={parent_class='/Script/OmGenerated.'..parents[asset]}})
        assert(validation.package_path(value.asset_path)==asset,'creation returned another asset')
        created=true
      end
      local skeleton=call('get_blueprint_skeleton',{blueprint_path=asset})
      assert(skeleton.parent_class==parents[asset], 'refusing mismatched existing Blueprint parent: '..asset)
    end
    local result = validation.compile(b,asset,path.join(work,index..'-compile.json'))
    result.created=created
    report.assets[index] = result
    json.write(path.join(work,'report.json'),report,true)
    assert(result.success, asset .. ': ' .. table.concat(result.diagnostics,'; '))
    if created then
      local saved=call('save_assets',{asset_paths={asset}})
      assert(saved.failed==0 and saved.saved==1,'asset save incomplete: '..asset)
      result.saved=true
    end
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
