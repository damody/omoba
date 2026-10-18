-- Stage project-owned item icon sources and run the supervised texture commandlet twice.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, process = b.lib('path'), b.lib('json'), b.lib('process')
local importer = require('item_icon_import')

local engine, run_id, validate_only, source_manifest
local i = 1
while i <= #arg do
  if arg[i] == '--ue-root' then i = i + 1; engine = assert(arg[i], '--ue-root requires a path')
  elseif arg[i] == '--run-id' then i = i + 1; run_id = assert(arg[i], '--run-id requires a value')
  elseif arg[i] == '--source-manifest' then i = i + 1; source_manifest = assert(arg[i], '--source-manifest requires a path')
  elseif arg[i] == '--validate-only' then validate_only = true
  elseif arg[i] == '--help' then
    print('Usage: tools/lua/lua.exe scripts/import_item_icons.lua --ue-root PATH --run-id NAME [--source-manifest PATH] [--validate-only]')
    print('First bootstrap: --source-manifest .ai-collab/candidates/item-icons-v1/manifest.json')
    print('Later runs omit --source-manifest and use omfue/Content/Omoba/UI/Items/Source/manifest.json')
    print('Validates project-contained PNGs, stages byte-identical PNGs and the source manifest, and runs OmItemIconImport twice.')
    return
  else error('unknown argument: ' .. tostring(arg[i])) end
  i = i + 1
end
engine = path.absolute(assert(engine, '--ue-root is required'), b.root)
run_id = assert(run_id, '--run-id is required')
assert(run_id:match('^[%w_-]+$'), 'run-id must contain only letters, digits, underscores or hyphens')
local manifest = source_manifest and path.absolute(source_manifest, b.root) or importer.durable_manifest(b.root)
local plan = importer.validate({root = b.root, manifest = manifest})
process.supervise_workflow(source, arg)
if validate_only then
  print(string.format('[item-icons] validated %d assets from %s', #plan.assets, manifest))
  return
end

local project_dir = path.join(b.root, 'omfue')
local project = path.join(project_dir, 'om.uproject')
local exe = path.join(engine, 'Engine/Binaries/Win64/UnrealEditor-Cmd.exe')
assert(path.is_file(exe), 'selected engine commandlet executable is missing')
assert(path.is_file(project), 'om.uproject is missing')
require('ue_binary_preflight').require_ready(engine, project)
local evidence = path.join(b.root, '.ai-collab/item-icon-import', run_id)
assert(not path.exists(evidence), 'item icon evidence directory must be new: ' .. evidence)
local stage_dir = path.join(project_dir, 'Content/Omoba/UI/Items/Source')
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
  assert(parsed.schema == 'omoba.item-icon-import-result.v1', label .. ' result schema is invalid')
  parsed.imported, parsed.verified, parsed.failed, parsed.saved =
    whole(parsed.imported), whole(parsed.verified), whole(parsed.failed), whole(parsed.saved)
  assert(parsed.failed == 0, label .. ' reported failed item icons')
  assert(parsed.imported + parsed.verified == #plan.assets, label .. ' did not account for every item icon')
  return parsed
end
print(string.format('[item-icons] staged=%d present=%d manifest_staged=%d manifest_present=%d',
  staged.staged, staged.present, staged.manifest_staged, staged.manifest_present))
local first = run_once('run-1')
assert(first.saved == first.imported, 'first import saved a different count than it imported')
local stamps = {}
for _, asset in ipairs(plan.assets) do
  local package = path.join(project_dir, 'Content/Omoba/UI/Items', asset.item_id .. '.uasset')
  assert(path.is_file(package), 'imported uasset is missing: ' .. package)
  stamps[package] = mtime(package)
end
local second = run_once('run-2')
assert(second.imported == 0 and second.saved == 0 and second.verified == #plan.assets,
  'second import was not an idempotent no-save verification')
for package, stamp in pairs(stamps) do
  assert(mtime(package) == stamp, 'second import changed uasset mtime: ' .. package)
end
json.write(path.join(evidence, 'summary.json'), {
  schema = 'omoba.item-icon-import-summary.v1', run_id = run_id, assets = #plan.assets,
  source_manifest = manifest, staged = staged.staged, present = staged.present,
  manifest_staged = staged.manifest_staged, manifest_present = staged.manifest_present,
  first_imported = first.imported, first_verified = first.verified, first_saved = first.saved,
  second_verified = second.verified, second_saved = second.saved,
}, false)
print(string.format('[item-icons] imported=%d verified=%d saved=%d; second verified=%d saved=0',
  first.imported, first.verified, first.saved, second.verified))
