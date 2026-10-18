local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]tests[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, hash = b.lib('path'), b.lib('json'), b.lib('hash')
local importer = require('ui_texture_import')
local items = require('item_icon_import')
local lfs = require('lfs')

local manifest = importer.durable_manifest(b.root)
assert(importer.inside_project(b.root, manifest))
assert(not importer.inside_project(b.root, path.join(b.root, '..', 'omoba-ui-texture-outside')))
assert(not importer.inside_project(b.root, path.join(b.root, 'omfue', '..', 'scripts')))
local plan = importer.validate({root = b.root, manifest = manifest, kind = 'trinket'})
assert(plan.schema == 'omoba.ui-texture-import-plan.v1' and plan.kind == 'trinket' and #plan.assets == 1)
local lantern = plan.assets[1]
assert(lantern.catalog_id == 1)
assert(lantern.content_id == 'training_scout_lantern_v1')
assert(lantern.display_name == '偵察燈')
assert(lantern.source_file == 'training-scout-lantern-v1.png')
assert(lantern.source_file ~= lantern.content_id .. '.png')
assert(lantern.sha256 == 'c140213f6610c027ae1c7310b05459bc631293f5fdee7a249f256678e081b499')
assert(lantern.bytes == 1751762 and lantern.width == 1254 and lantern.height == 1254)
assert(lantern.object_path == '/Game/Omoba/UI/Trinkets/training_scout_lantern_v1.training_scout_lantern_v1')
assert(lantern.destination_path == '/Game/Omoba/UI/Trinkets')
assert(lantern.item_id == nil)
assert(importer.inside_project(b.root, lantern.source_png))
local authored = importer.authored(b.root, 'trinket')
assert(#authored == 1 and authored[1].id == lantern.content_id and authored[1].catalog_id == 1)

local item_manifest_path = path.is_file(items.durable_manifest(b.root)) and items.durable_manifest(b.root)
  or path.join(b.root, '.ai-collab/candidates/item-icons-v1/manifest.json')
local item_doc = json.read(item_manifest_path)
local converted = {schema = 'omoba.ui-texture-assets.v1', kind = 'item', tool = 'builtin-image_gen', status = 'source-registered-import-pending', assets = {}}
for _, asset in ipairs(item_doc.assets) do
  converted.assets[#converted.assets + 1] = {
    catalog_id = asset.catalog_id, content_id = asset.item_id, file = asset.file,
    width = asset.width, height = asset.height, bytes = asset.bytes, sha256 = asset.sha256,
    source = asset.source,
  }
end

local token = string.format('fix-%d-%s', os.time(), tostring({}):match('0x%x+') or 'x')
local fixture = path.join(b.root, '.ai-collab', 'ui-texture-lua-fixtures', token)
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
local converted_path = path.join(fixture, 'item-general.json')
json.write(converted_path, converted, false)
local item_plan = importer.validate({
  root = b.root, manifest = converted_path, kind = 'item', candidate_dir = path.parent(item_manifest_path),
})
assert(item_plan.kind == 'item' and #item_plan.assets == 11)
local by_id = {}
for _, asset in ipairs(item_plan.assets) do
  assert(asset.destination_path == '/Game/Omoba/UI/Items')
  assert(asset.object_path == '/Game/Omoba/UI/Items/' .. asset.content_id .. '.' .. asset.content_id)
  assert(asset.item_id == nil)
  by_id[asset.content_id] = asset
end
assert(by_id.moba_sword.catalog_id == 1 and by_id.moba_ward_charm.catalog_id == 8 and by_id.moba_training_hex.catalog_id == 11)

local function rejected(mutate)
  local copy = json.decode(json.encode(json.read(manifest)))
  mutate(copy)
  local file = path.join(fixture, 'negative-manifest.json')
  json.write(file, copy, true)
  local ok = pcall(importer.validate, {root = b.root, manifest = file, kind = 'trinket'})
  os.remove(file)
  assert(not ok, 'invalid trinket manifest was accepted')
end
rejected(function(doc) doc.schema = 'omoba.item-icon-assets.v1' end)
rejected(function(doc) doc.kind = 'item' end)
rejected(function(doc) doc.status = 'imported' end)
rejected(function(doc) doc.assets[1].sha256 = string.rep('ab', 32) end)
rejected(function(doc) doc.assets[1].file = '../training-scout-lantern-v1.png' end)
rejected(function(doc) doc.assets[1].bytes = 1 end)
rejected(function(doc) doc.assets[1].width = 1024 end)
rejected(function(doc) doc.assets[1].content_id = 'training-scout-lantern-v1' end)
rejected(function(doc) doc.assets[1].content_id = 'moba_sword' end)
rejected(function(doc) doc.assets[1].catalog_id = 2 end)
rejected(function(doc) doc.assets[1].item_id = doc.assets[1].content_id end)
rejected(function(doc) doc.assets[#doc.assets + 1] = doc.assets[1] end)
rejected(function(doc) doc.assets = {} end)
assert(not pcall(importer.validate, {
  root = b.root, manifest = manifest, kind = 'trinket', candidate_dir = path.join(b.root, '..', 'omoba-ui-texture-outside'),
}))

local registry_original = path.read(importer.registry_path(b.root), true)
local registry_doc = json.decode(registry_original)
for _, entry in ipairs(registry_doc.entries) do
  if entry.kind == 'trinket' and entry.id == lantern.content_id then entry.resource = nil end
end
local registry_before = json.encode(registry_doc)
local registry_copy = path.join(fixture, 'hud-assets-v1.json')
path.write(registry_copy, registry_before, true, true)
local published, changed = importer.publish_registry_resource(registry_before, 'trinket', lantern.content_id, lantern.object_path)
assert(changed and published ~= registry_before)
assert(published:find(lantern.object_path, 1, true))
assert(published:find('/Game/Omoba/UI/Items/moba_sword.moba_sword', 1, true))
local again, changed_again = importer.publish_registry_resource(published, 'trinket', lantern.content_id, lantern.object_path)
assert(not changed_again and again == published)
assert(not pcall(importer.publish_registry_resource, published, 'trinket', lantern.content_id, '/Game/Omoba/UI/Items/training_scout_lantern_v1.training_scout_lantern_v1'))
assert(path.read(importer.registry_path(b.root), true) == registry_original)
assert(path.read(registry_copy, true) == registry_before)

-- Registry publication consumes JSON, including whitespace and braces in strings.
local formatted = registry_before:gsub(':', ': ')
local formatted_result, formatted_changed = importer.publish_registry_resource(formatted, 'trinket', lantern.content_id, lantern.object_path)
assert(formatted_changed and json.encode(json.decode(formatted_result)) == json.encode(json.decode(published)))
local with_strings = json.decode(registry_before)
with_strings.description = 'unrelated { "id":"' .. lantern.content_id .. '" } text'
local strings_result = importer.publish_registry_resource(json.encode(with_strings), 'trinket', lantern.content_id, lantern.object_path)
assert(json.decode(strings_result).description == with_strings.description)
local duplicate = json.decode(registry_before)
for _, entry in ipairs(registry_doc.entries) do
  if entry.id == lantern.content_id then duplicate.entries[#duplicate.entries + 1] = entry break end
end
assert(not pcall(importer.publish_registry_resource, json.encode(duplicate), 'trinket', lantern.content_id, lantern.object_path))

local source_png = path.join(fixture, 'training-scout-lantern-v1.png')
local payload = 'same-ui-texture-bytes'
path.write(source_png, payload, true, true)
local manifest_bytes = '{"schema":"omoba.ui-texture-assets.v1","kind":"trinket","status":"fixture"}'
local source_manifest = path.join(fixture, 'source-manifest.json')
path.write(source_manifest, manifest_bytes, false, true)
local stage_plan = {
  schema = plan.schema, kind = 'trinket', root = b.root, source_manifest = source_manifest,
  assets = {{
    kind = 'trinket', content_id = lantern.content_id, source_file = 'training-scout-lantern-v1.png',
    source_png = source_png, bytes = #payload, sha256 = hash.sha256(source_png),
    object_path = lantern.object_path,
  }},
}
local stage_dir = path.join(fixture, 'stage')
local first = importer.stage(stage_plan, stage_dir)
assert(first.staged == 1 and first.present == 0 and first.manifest_staged == 1 and first.manifest_present == 0)
assert(path.read(stage_plan.assets[1].staged_png, true) == payload)
assert(stage_plan.assets[1].staged_png:find('training_scout_lantern_v1.png', 1, true))
local second = importer.stage(stage_plan, stage_dir)
assert(second.staged == 0 and second.present == 1 and second.manifest_staged == 0 and second.manifest_present == 1)
path.write(stage_plan.assets[1].staged_png, 'different-bytes', true, true)
assert(not pcall(importer.stage, stage_plan, stage_dir))
assert(path.read(stage_plan.assets[1].staged_png, true) == 'different-bytes')
path.write(stage_plan.assets[1].staged_png, payload, true, true)
local outside_plan = {
  schema = plan.schema, kind = 'trinket', root = b.root, source_manifest = source_manifest,
  assets = {{
    kind = 'trinket', content_id = lantern.content_id, source_png = path.join(b.root, '..', 'training-scout-lantern-v1.png'),
    bytes = #payload, sha256 = hash.sha256(source_png), object_path = lantern.object_path,
  }},
}
assert(not pcall(importer.stage, outside_plan, path.join(fixture, 'outside-stage')))
assert(not pcall(importer.stage, stage_plan, path.join(b.root, '..', 'omoba-ui-texture-outside-stage')))
local commandlet = importer.commandlet_plan(stage_plan)
assert(commandlet.schema == plan.schema and commandlet.kind == 'trinket' and commandlet.assets[1].item_id == nil)
assert(commandlet.assets[1].content_id == 'training_scout_lantern_v1')
assert(commandlet.assets[1].destination_path == nil)
stage_plan.assets[1].destination_path = lantern.destination_path
commandlet = importer.commandlet_plan(stage_plan)
assert(commandlet.assets[1].destination_path == '/Game/Omoba/UI/Trinkets')

remove_tree(fixture)
pcall(lfs.rmdir, path.join(b.root, '.ai-collab', 'ui-texture-lua-fixtures'))
print('ui texture import validation tests passed')
