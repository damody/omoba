local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]tests[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, hash = b.lib('path'), b.lib('json'), b.lib('hash')
local importer = require('item_icon_import')
local lfs = require('lfs')

local durable = path.parent(importer.durable_manifest(b.root))
local candidates = path.is_file(importer.durable_manifest(b.root)) and durable or path.join(b.root, '.ai-collab/candidates/item-icons-v1')
local manifest = path.join(candidates, 'manifest.json')
assert(importer.inside_project(b.root, candidates))
assert(importer.inside_project(b.root, manifest))
assert(importer.durable_manifest(b.root) == path.join(b.root, 'omfue/Content/Omoba/UI/Items/Source/manifest.json'))
assert(not importer.inside_project(b.root, path.join(b.root, '..', 'omoba-item-icon-outside')))
assert(not importer.inside_project(b.root, path.join(b.root, 'omfue', '..', 'scripts')))

local plan = importer.validate({root = b.root, manifest = manifest})
assert(plan.schema == 'omoba.item-icon-import-plan.v1' and #plan.assets == 11)
assert(plan.source_manifest == path.absolute(manifest))
local by_id = {}
for _, asset in ipairs(plan.assets) do
  assert(not by_id[asset.item_id], 'duplicate planned item')
  assert(importer.inside_project(b.root, asset.source_png))
  by_id[asset.item_id] = asset
  assert(asset.width == 1254 and asset.height == 1254)
  assert(asset.object_path == '/Game/Omoba/UI/Items/' .. asset.item_id .. '.' .. asset.item_id)
  assert(asset.charges == nil)
end
assert(by_id.moba_sword.catalog_id == 1)
assert(by_id.moba_ward_charm.catalog_id == 8)
assert(by_id.moba_training_hex.catalog_id == 11)
local authored = importer.authored_items(b.root)
local ward
for _, item in ipairs(authored) do if item.id == 'moba_ward_charm' then ward = item end end
assert(ward and ward.catalog_id == 8 and ward.active.kind == 'damage_reduce' and ward.charges == nil)

local token = string.format('fix-%d-%s', os.time(), tostring({}):match('0x%x+') or 'x')
local fixture = path.join(b.root, '.ai-collab', 'item-icon-lua-fixtures', token)
local function remove_tree(dir)
  if not path.exists(dir) then return end
  for entry in lfs.dir(dir) do
    if entry ~= '.' and entry ~= '..' then
      local child = path.join(dir, entry)
      if path.is_directory(child) then remove_tree(child) else os.remove(child) end
    end
  end
  lfs.rmdir(dir)
end
path.mkdir_p(fixture)

local function rejected(mutate)
  local copy = json.decode(json.encode(json.read(manifest)))
  mutate(copy)
  local file = path.join(fixture, 'negative-manifest.json')
  json.write(file, copy, true)
  local ok = pcall(importer.validate, {root = b.root, manifest = file, candidate_dir = candidates})
  os.remove(file)
  assert(not ok, 'invalid manifest was accepted')
end
rejected(function(doc) doc.schema = 'other' end)
rejected(function(doc) doc.assets[1].sha256 = string.rep('ab', 32) end)
rejected(function(doc) doc.assets[1].file = '../moba_sword.png' end)
rejected(function(doc) doc.assets[1].bytes = 1 end)
rejected(function(doc) doc.assets[1].width = 1024 end)
rejected(function(doc) doc.assets[#doc.assets + 1] = doc.assets[1] end)
rejected(function(doc) table.remove(doc.assets) end)
rejected(function(doc) doc.assets[1].item_id = 'moba_armor' end)
assert(not pcall(importer.validate, {
  root = b.root, manifest = manifest, candidate_dir = path.join(b.root, '..', 'omoba-item-icon-outside'),
}))
assert(not pcall(importer.validate, {
  root = b.root, manifest = manifest, candidate_dir = path.join(b.root, 'omfue', '..', 'omfue', 'Content'),
}))

local source_png = path.join(fixture, 'moba_sword.png')
local payload = 'same-item-icon-bytes'
path.write(source_png, payload, true, true)
local manifest_bytes = '{"schema":"omoba.item-icon-assets.v1","status":"fixture"}'
local source_manifest = path.join(fixture, 'source-manifest.json')
path.write(source_manifest, manifest_bytes, false, true)
local stage_plan = {schema = plan.schema, root = b.root, source_manifest = source_manifest, assets = {{
  item_id = 'moba_sword', source_png = source_png, bytes = #payload, sha256 = hash.sha256(source_png),
}}}
local stage_dir = path.join(fixture, 'stage')
local first = importer.stage(stage_plan, stage_dir)
assert(first.staged == 1 and first.present == 0 and first.manifest_staged == 1 and first.manifest_present == 0)
assert(path.read(stage_plan.assets[1].staged_png, true) == payload)
assert(path.read(path.join(stage_dir, 'manifest.json'), true) == manifest_bytes)
local second = importer.stage(stage_plan, stage_dir)
assert(second.staged == 0 and second.present == 1 and second.manifest_staged == 0 and second.manifest_present == 1)
path.write(stage_plan.assets[1].staged_png, 'different-bytes', true, true)
assert(not pcall(importer.stage, stage_plan, stage_dir))
assert(path.read(stage_plan.assets[1].staged_png, true) == 'different-bytes')
assert(path.read(path.join(stage_dir, 'manifest.json'), true) == manifest_bytes)
path.write(path.join(stage_dir, 'manifest.json'), 'different-manifest', true, true)
path.write(stage_plan.assets[1].staged_png, payload, true, true)
assert(not pcall(importer.stage, stage_plan, stage_dir))
assert(path.read(path.join(stage_dir, 'manifest.json'), true) == 'different-manifest')
assert(path.read(stage_plan.assets[1].staged_png, true) == payload)
path.write(path.join(stage_dir, 'manifest.json'), manifest_bytes, true, true)
stage_plan.assets[1].sha256 = string.rep('cd', 32)
assert(not pcall(importer.stage, stage_plan, stage_dir))
assert(path.read(stage_plan.assets[1].staged_png, true) == payload)
local outside_plan = {schema = plan.schema, root = b.root, source_manifest = source_manifest, assets = {{
  item_id = 'moba_sword', source_png = path.join(b.root, '..', 'moba_sword.png'),
  bytes = #payload, sha256 = hash.sha256(source_png),
}}}
local outside_stage = path.join(fixture, 'outside-stage')
assert(not pcall(importer.stage, outside_plan, outside_stage))
assert(not path.is_file(path.join(outside_stage, 'moba_sword.png')))
assert(not path.is_file(path.join(outside_stage, 'manifest.json')))
assert(not pcall(importer.stage, stage_plan, path.join(b.root, '..', 'omoba-item-icon-outside-stage')))

remove_tree(fixture)
pcall(lfs.rmdir, path.join(b.root, '.ai-collab', 'item-icon-lua-fixtures'))
print('item icon import validation tests passed')
