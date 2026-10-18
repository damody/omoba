-- Validate and stage catalog UI textures. This module does not launch the editor.
-- Gameplay identity comes from the authored catalog row, never from the PNG file name.
local json = require('tools.lua.lib.json')
local path = require('tools.lua.lib.path')
local hash = require('tools.lua.lib.hash')

local M = {}
local EXPECTED_EDGE = 1254

M.kinds = {
  item = {
    catalog = 'scripts/lua_data/templates/moba_items.lua',
    destination = '/Game/Omoba/UI/Items',
    stage = 'omfue/Content/Omoba/UI/Items/Source',
    file_matches_content = true,
    reject_charges = true,
    label = 'name',
  },
  trinket = {
    catalog = 'scripts/lua_data/templates/moba_trinkets.lua',
    destination = '/Game/Omoba/UI/Trinkets',
    stage = 'omfue/Content/Omoba/UI/Trinkets/Source',
    file_matches_content = false,
    reject_charges = false,
    label = 'display_name',
  },
}

local function whole(value, min_value, max_value)
  return type(value) == 'number' and value == math.floor(value) and value >= min_value and value <= max_value
end

local function identifier(value)
  return type(value) == 'string' and #value <= 64 and value:match('^[a-z][a-z0-9_]*$') ~= nil
end

local function flat_png(value)
  return type(value) == 'string' and #value <= 128 and not value:find('[/\\]') and not value:find('%.%.')
    and value:match('^[a-z0-9][a-z0-9_.%-]*%.png$') ~= nil
end

local function escapes(value)
  for segment in tostring(value):gsub('\\', '/'):gmatch('[^/]+') do
    if segment == '..' then return true end
  end
  return false
end

local function normalize(value)
  local text = tostring(value):gsub('\\', '/'):gsub('/+$', '')
  if text:match('^%a:') then text = text:lower() end
  return text
end

function M.durable_manifest(root)
  return path.join(root, 'omfue/Content/Omoba/UI/trinket-icon-assets-v1.json')
end

function M.registry_path(root)
  return path.join(root, 'omfue/Content/Omoba/UI/hud-assets-v1.json')
end

function M.inside_project(root, value)
  if escapes(value) then return false end
  local absolute = path.absolute(value, root)
  if escapes(absolute) then return false end
  local root_abs = path.absolute(root)
  local item, base = normalize(absolute), normalize(root_abs)
  return item == base or item:sub(1, #base + 1) == base .. '/'
end

function M.object_path(kind, content_id)
  local spec = assert(M.kinds[kind], 'unknown ui texture kind')
  assert(identifier(content_id), 'content id is invalid')
  return spec.destination .. '/' .. content_id .. '.' .. content_id
end

function M.png_size(data)
  assert(type(data) == 'string' and #data >= 24, 'png is truncated')
  assert(data:sub(1, 8) == '\137PNG\r\n\26\n', 'png signature is invalid')
  assert(data:sub(13, 16) == 'IHDR', 'png IHDR is missing')
  local width, height = string.unpack('>I4I4', data, 17)
  assert(width > 0 and height > 0, 'png dimensions are invalid')
  return width, height
end

local function authored_rows(root, kind)
  local spec = assert(M.kinds[kind], 'unknown ui texture kind')
  local chunk, err = loadfile(path.join(root, spec.catalog))
  assert(chunk, err)
  local builder = assert(chunk(), 'catalog did not return a builder')
  assert(type(builder) == 'function', 'catalog builder is invalid')
  local rows = assert(builder({}), 'catalog builder returned nothing')
  assert(type(rows) == 'table' and #rows > 0 and #rows <= 64, 'authored catalog is empty or too large')
  return rows, spec
end

function M.authored(root, kind)
  return authored_rows(root, kind)
end

function M.validate(options)
  assert(type(options) == 'table', 'validate options are required')
  local root = assert(options.root, 'repository root is required')
  local kind = assert(options.kind, 'catalog kind is required')
  local manifest_path = path.absolute(assert(options.manifest, 'manifest path is required'), root)
  assert(not escapes(manifest_path), 'manifest path cannot contain a parent segment')
  local manifest = json.read(manifest_path)
  assert(manifest.schema == 'omoba.ui-texture-assets.v1', 'unsupported ui texture manifest schema')
  assert(manifest.kind == kind, 'manifest kind does not match the requested catalog')
  assert(manifest.tool == 'builtin-image_gen', 'ui texture manifest tool is not the original generator')
  assert(type(manifest.status) == 'string' and manifest.status ~= '', 'ui texture manifest status is missing')
  assert(manifest.status ~= 'imported', 'source manifest cannot claim the texture is imported')
  assert(type(manifest.assets) == 'table' and #manifest.assets > 0 and #manifest.assets <= 64, 'ui texture asset list is invalid')
  local candidate_dir = path.absolute(options.candidate_dir or path.parent(manifest_path), root)
  assert(M.inside_project(root, candidate_dir), 'candidate png directory is outside the project: ' .. candidate_dir)
  local authored, spec = authored_rows(root, kind)
  local by_catalog, by_id = {}, {}
  for _, row in ipairs(authored) do
    assert(whole(row.catalog_id, 1, 64), 'authored catalog id is out of bounds')
    assert(identifier(row.id), 'authored content id is invalid')
    assert(type(row[spec.label]) == 'string' and row[spec.label] ~= '', 'authored display name is missing')
    assert(not by_catalog[row.catalog_id] and not by_id[row.id], 'authored catalog identity is duplicated')
    if spec.reject_charges then assert(row.charges == nil, 'item catalog rows have no charge contract') end
    by_catalog[row.catalog_id] = row
    by_id[row.id] = row
  end
  assert(#manifest.assets == #authored, 'manifest assets must cover the authored catalog exactly')
  local seen_catalog, seen_id, seen_dest, assets = {}, {}, {}, {}
  for _, asset in ipairs(manifest.assets) do
    assert(type(asset) == 'table', 'manifest asset is not an object')
    assert(asset.item_id == nil, 'ui texture manifest cannot use an item id')
    assert(whole(asset.catalog_id, 1, 64), 'manifest catalog id is out of bounds')
    assert(identifier(asset.content_id), 'manifest content id is invalid')
    assert(flat_png(asset.file), 'manifest file must be a flat png name')
    if spec.file_matches_content then
      assert(asset.file == asset.content_id .. '.png', 'item file must match its content id')
    end
    assert(asset.width == EXPECTED_EDGE and asset.height == EXPECTED_EDGE, 'manifest dimensions must be the actual 1254 source')
    assert(whole(asset.bytes, 1, 16 * 1024 * 1024), 'manifest byte size is invalid')
    assert(type(asset.sha256) == 'string' and asset.sha256:match('^[0-9a-f]+$') and #asset.sha256 == 64, 'manifest sha256 is invalid')
    assert(type(asset.source) == 'string' and asset.source ~= '' and not escapes(asset.source), 'manifest source provenance is missing')
    local row = by_catalog[asset.catalog_id]
    assert(row and row.id == asset.content_id, 'manifest content id does not match the authored catalog id')
    assert(not seen_catalog[asset.catalog_id], 'duplicate manifest catalog id')
    assert(not seen_id[asset.content_id], 'duplicate manifest content id')
    seen_catalog[asset.catalog_id] = true
    seen_id[asset.content_id] = true
    local source_png = path.join(candidate_dir, asset.file)
    assert(M.inside_project(root, source_png), 'candidate png is outside the project: ' .. source_png)
    assert(path.is_file(source_png), 'candidate png is missing: ' .. source_png)
    local data = path.read(source_png, true)
    assert(#data == asset.bytes, 'candidate png byte size does not match the manifest')
    local width, height = M.png_size(data)
    assert(width == EXPECTED_EDGE and height == EXPECTED_EDGE, 'candidate png is not 1254x1254')
    local digest = hash.sha256(source_png)
    assert(digest == asset.sha256, 'candidate png sha256 does not match the manifest')
    local object_path = M.object_path(kind, row.id)
    assert(not seen_dest[object_path], 'duplicate ui texture destination')
    seen_dest[object_path] = true
    assets[#assets + 1] = {
      kind = kind, content_id = row.id, catalog_id = row.catalog_id, display_name = row[spec.label],
      source_png = path.absolute(source_png), source_file = asset.file,
      sha256 = digest, width = width, height = height, bytes = #data,
      object_path = object_path, destination_path = spec.destination,
    }
  end
  for catalog_id in pairs(by_catalog) do
    assert(seen_catalog[catalog_id], 'authored catalog row has no texture: ' .. catalog_id)
  end
  table.sort(assets, function(a, b) return a.catalog_id < b.catalog_id end)
  return {
    schema = 'omoba.ui-texture-import-plan.v1', kind = kind, assets = assets,
    source_manifest = manifest_path, root = root,
  }
end

local function destination_state(dest, data, label)
  if not path.is_file(dest) then return 'new' end
  assert(path.read(dest, true) == data, 'existing ' .. label .. ' differs and was not overwritten: ' .. dest)
  return 'present'
end

function M.stage(plan, stage_dir)
  assert(plan and plan.schema == 'omoba.ui-texture-import-plan.v1', 'stage requires a validated plan')
  assert(M.kinds[plan.kind], 'stage kind is invalid')
  assert(type(plan.root) == 'string' and plan.root ~= '', 'stage requires the project root')
  assert(type(plan.source_manifest) == 'string' and plan.source_manifest ~= '', 'stage requires the source manifest')
  assert(not escapes(plan.source_manifest), 'source manifest path cannot contain a parent segment')
  stage_dir = path.absolute(stage_dir)
  assert(M.inside_project(plan.root, stage_dir), 'stage directory is outside the project: ' .. stage_dir)
  local manifest_bytes = path.read(plan.source_manifest, true)
  local manifest_dest = path.join(stage_dir, 'manifest.json')
  local prepared = {}
  for _, asset in ipairs(plan.assets) do
    assert(asset.kind == plan.kind, 'staged asset kind does not match the plan')
    assert(M.object_path(plan.kind, asset.content_id) == asset.object_path, 'staged object path does not match the catalog id')
    assert(M.inside_project(plan.root, asset.source_png), 'staged source png is outside the project: ' .. tostring(asset.source_png))
    local data = path.read(asset.source_png, true)
    assert(#data == asset.bytes, 'staged source byte size changed: ' .. asset.content_id)
    assert(hash.sha256(asset.source_png) == asset.sha256, 'staged source hash changed: ' .. asset.content_id)
    prepared[#prepared + 1] = {asset = asset, data = data, dest = path.join(stage_dir, asset.content_id .. '.png')}
  end
  local manifest_state = destination_state(manifest_dest, manifest_bytes, 'source manifest')
  for _, item in ipairs(prepared) do
    item.state = destination_state(item.dest, item.data, 'staged ui texture')
  end
  if manifest_state == 'new' then path.write(manifest_dest, manifest_bytes, false, true) end
  local staged, present = 0, 0
  for _, item in ipairs(prepared) do
    if item.state == 'new' then
      path.write(item.dest, item.data, false, true)
      staged = staged + 1
    else
      present = present + 1
    end
    item.asset.staged_png = path.absolute(item.dest)
  end
  return {
    staged = staged, present = present,
    manifest_staged = manifest_state == 'new' and 1 or 0,
    manifest_present = manifest_state == 'present' and 1 or 0,
  }
end

function M.commandlet_plan(plan)
  local assets = {}
  for index, asset in ipairs(plan.assets) do
    assert(type(asset.staged_png) == 'string' and asset.staged_png ~= '', 'staged png is missing at ' .. index)
    assert(asset.item_id == nil, 'commandlet plan cannot use an item id')
    assets[index] = {
      content_id = asset.content_id, catalog_id = asset.catalog_id, png = asset.staged_png,
      sha256 = asset.sha256, width = asset.width, height = asset.height, bytes = asset.bytes,
      object_path = asset.object_path, destination_path = asset.destination_path,
    }
  end
  return {schema = plan.schema, kind = plan.kind, assets = assets}
end

function M.publish_registry_resource(registry_text, kind, content_id, object_path)
  assert(type(registry_text) == 'string' and registry_text ~= '', 'registry text is missing')
  assert(object_path == M.object_path(kind, content_id), 'registry resource is not the catalog object path')
  local registry = json.decode(registry_text)
  assert(type(registry) == 'table' and type(registry.entries) == 'table', 'registry entries are missing')
  local target
  for _, entry in ipairs(registry.entries) do
    assert(type(entry) == 'table', 'registry entry is not an object')
    if entry.id == content_id then
      assert(not target, 'registry content id is duplicated: ' .. content_id)
      assert(entry.kind == kind, 'registry kind does not match the catalog')
      target = entry
    end
  end
  assert(target, 'registry entry is missing: ' .. content_id)
  if target.resource ~= nil then
    assert(type(target.resource) == 'string' and target.resource == object_path,
      'registry resource differs and was not overwritten: ' .. content_id)
    return registry_text, false
  end
  target.resource = object_path
  return json.encode(registry), true
end

return M
