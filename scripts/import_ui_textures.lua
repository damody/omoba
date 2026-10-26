-- Stage project-owned catalog UI textures and run the supervised texture commandlet twice.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, process = b.lib('path'), b.lib('json'), b.lib('process')
local importer = require('ui_texture_import')

local engine, run_id, validate_only, source_manifest, kind
local i = 1
while i <= #arg do
  if arg[i] == '--ue-root' then i = i + 1; engine = assert(arg[i], '--ue-root requires a path')
  elseif arg[i] == '--run-id' then i = i + 1; run_id = assert(arg[i], '--run-id requires a value')
  elseif arg[i] == '--kind' then i = i + 1; kind = assert(arg[i], '--kind requires item, trinket or ability')
  elseif arg[i] == '--source-manifest' then i = i + 1; source_manifest = assert(arg[i], '--source-manifest requires a path')
  elseif arg[i] == '--validate-only' then validate_only = true
  elseif arg[i] == '--help' then
    print('Usage: tools/lua/lua.exe scripts/import_ui_textures.lua --kind item|trinket|ability --ue-root PATH --run-id NAME [--source-manifest PATH] [--validate-only]')
    print('Trinket default manifest: omfue/Content/Omoba/UI/trinket-icon-assets-v1.json')
    print('Ability default manifest: omfue/Content/Omoba/UI/Abilities/Source/manifest.json')
    print('Validates catalog identity, source hash, and destination, then runs OmItemIconImport twice.')
    print('Validate-only does not save a package or publish a registry resource.')
    return
  else error('unknown argument: ' .. tostring(arg[i])) end
  i = i + 1
end
kind = assert(kind, '--kind is required')
assert(importer.kinds[kind], '--kind must be item, trinket or ability')
engine = path.absolute(assert(engine, '--ue-root is required'), b.root)
run_id = assert(run_id, '--run-id is required')
assert(run_id:match('^[%w_-]+$'), 'run-id must contain only letters, digits, underscores or hyphens')
if kind == 'item' then assert(source_manifest, '--kind ' .. kind .. ' requires --source-manifest') end
local manifest = source_manifest and path.absolute(source_manifest, b.root) or importer.durable_manifest(b.root, kind)
local plan = importer.validate({root = b.root, manifest = manifest, kind = kind})
process.supervise_workflow(source, arg)
if validate_only then
  print(string.format('[ui-textures] validated %d %s assets from %s', #plan.assets, kind, manifest))
  return
end

local project_dir = path.join(b.root, 'omfue')
local project = path.join(project_dir, 'om.uproject')
local exe = path.join(engine, 'Engine/Binaries/Win64/UnrealEditor-Cmd.exe')
assert(path.is_file(exe), 'selected engine commandlet executable is missing')
assert(path.is_file(project), 'om.uproject is missing')
require('ue_binary_preflight').require_ready(engine, project)
local evidence = path.join(b.root, '.ai-collab/ui-texture-import', run_id)
assert(not path.exists(evidence), 'ui texture evidence directory must be new: ' .. evidence)
local stage_dir = path.join(b.root, importer.kinds[kind].stage)
local staged = importer.stage(plan, stage_dir)
path.mkdir_p(evidence)
local plan_path = path.join(evidence, 'plan.json')
json.write(plan_path, importer.commandlet_plan(plan), false)
local function whole(value)
  assert(type(value) == 'number' and value == math.floor(value), 'import count is not a whole number')
  return math.floor(value)
end
local function mtime(file)
  local attributes = path.attributes(file)
  assert(type(attributes.modification) == 'number', 'uasset mtime is missing: ' .. file)
  return attributes.modification
end
local function package_file(object_path)
  local package = assert(object_path:match('^(.*)%.[^%.]+$'), 'object path has no asset name')
  local relative = assert(package:match('^/Game/(.+)$'), 'object path is outside /Game')
  return path.join(project_dir, 'Content', relative .. '.uasset')
end
local function run_once(label)
  local result_path = path.join(evidence, label .. '-result.json')
  local log_path = path.join(evidence, label .. '-engine.log')
  local arguments = {
    project, '-run=OmItemIconImport', '-OmItemIconPlan=' .. plan_path, '-OmItemIconResult=' .. result_path,
    '-unattended', '-nop4', '-nosplash', '-nopause', '-AllowCommandletRendering', '-d3d12', '-NoSound', '-NoLiveCoding',
    '-abslog=' .. log_path,
  }
  local ran = process.run(exe, arguments, {cwd = project_dir, check = false})
  path.write(path.join(evidence, label .. '-stdout.log'), ran.stdout or '', false)
  path.write(path.join(evidence, label .. '-stderr.log'), ran.stderr or '', false)
  local parsed = path.is_file(result_path) and json.read(result_path) or nil
  assert(ran.exit_code == 0 and parsed, label .. ' commandlet failed: ' .. tostring(ran.exit_code))
  assert(parsed.schema == 'omoba.ui-texture-import-result.v1', label .. ' result schema is invalid')
  parsed.imported, parsed.verified, parsed.failed, parsed.saved =
    whole(parsed.imported), whole(parsed.verified), whole(parsed.failed), whole(parsed.saved)
  assert(parsed.failed == 0, label .. ' reported failed ui textures')
  assert(parsed.imported + parsed.verified == #plan.assets, label .. ' did not account for every ui texture')
  local by_id = {}
  for _, asset in ipairs(parsed.assets) do
    assert(asset.kind == kind, label .. ' result kind does not match the plan')
    assert(type(asset.package_sha256) == 'string' and asset.package_sha256:match('^[0-9a-f]+$') and #asset.package_sha256 == 64,
      label .. ' package sha256 is missing')
    assert(asset.item_id == nil, label .. ' result uses an item id')
    by_id[asset.content_id] = asset
  end
  return parsed, by_id
end
print(string.format('[ui-textures] kind=%s staged=%d present=%d manifest_staged=%d manifest_present=%d',
  kind, staged.staged, staged.present, staged.manifest_staged, staged.manifest_present))
local first, first_assets = run_once('run-1')
assert(first.saved == first.imported, 'first import saved a different count than it imported')
local stamps, package_shas = {}, {}
for _, asset in ipairs(plan.assets) do
  local package = package_file(asset.object_path)
  assert(path.is_file(package), 'imported uasset is missing: ' .. package)
  local result = assert(first_assets[asset.content_id], 'result omitted ' .. asset.content_id)
  local digest = require('tools.lua.lib.hash').sha256(package)
  assert(digest == result.package_sha256, 'package sha256 does not match the saved uasset: ' .. package)
  stamps[package] = mtime(package)
  package_shas[asset.content_id] = digest
end
local second, second_assets = run_once('run-2')
assert(second.imported == 0 and second.saved == 0 and second.verified == #plan.assets,
  'second import was not an idempotent no-save verification')
for package, stamp in pairs(stamps) do
  assert(mtime(package) == stamp, 'second import changed uasset mtime: ' .. package)
end
for _, asset in ipairs(plan.assets) do
  local result = assert(second_assets[asset.content_id], 'second result omitted ' .. asset.content_id)
  assert(result.package_sha256 == package_shas[asset.content_id], 'second import changed package sha256: ' .. asset.content_id)
end
local registry = importer.registry_path(b.root)
local registry_text = path.read(registry, true)
local published = 0
for _, asset in ipairs(plan.assets) do
  local updated, changed = importer.publish_registry_resource(registry_text, kind, asset.content_id, asset.object_path, asset.display_name)
  if changed then published = published + 1 end
  registry_text = updated
end
if published > 0 then path.write(registry, registry_text, true, true) end
local identities = {}
for index, asset in ipairs(plan.assets) do
  identities[index] = {
    catalog_id = asset.catalog_id, content_id = asset.content_id, display_name = asset.display_name,
    object_path = asset.object_path, source_sha256 = asset.sha256, package_sha256 = package_shas[asset.content_id],
  }
end
json.write(path.join(evidence, 'summary.json'), {
  schema = 'omoba.ui-texture-import-summary.v1', run_id = run_id, kind = kind, assets = #plan.assets,
  source_manifest = manifest, staged = staged.staged, present = staged.present,
  manifest_staged = staged.manifest_staged, manifest_present = staged.manifest_present,
  first_imported = first.imported, first_verified = first.verified, first_saved = first.saved,
  second_verified = second.verified, second_saved = second.saved, registry_published = published,
  identities = identities,
}, false)
print(string.format('[ui-textures] imported=%d verified=%d saved=%d; second verified=%d saved=0; registry_published=%d',
  first.imported, first.verified, first.saved, second.verified, published))
