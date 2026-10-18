-- Validate and stage original item-icon PNGs. This module does not launch the editor.
local json = require('tools.lua.lib.json')
local path = require('tools.lua.lib.path')
local hash = require('tools.lua.lib.hash')

local M = {}
local EXPECTED_EDGE = 1254

local function whole(value, min_value, max_value)
  return type(value) == 'number' and value == math.floor(value) and value >= min_value and value <= max_value
end

local function identifier(value)
  return type(value) == 'string' and #value <= 64 and value:match('^[a-z][a-z0-9_]*$') ~= nil
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
  return path.join(root, 'omfue/Content/Omoba/UI/Items/Source/manifest.json')
end

function M.inside_project(root, value)
  if escapes(value) then return false end
  local absolute = path.absolute(value, root)
  if escapes(absolute) then return false end
  local root_abs = path.absolute(root)
  local item, base = normalize(absolute), normalize(root_abs)
  return item == base or item:sub(1, #base + 1) == base .. '/'
end

function M.png_size(data)
  assert(type(data) == 'string' and #data >= 24, 'png is truncated')
  assert(data:sub(1, 8) == '\137PNG\r\n\26\n', 'png signature is invalid')
  assert(data:sub(13, 16) == 'IHDR', 'png IHDR is missing')
  local width, height = string.unpack('>I4I4', data, 17)
  assert(width > 0 and height > 0, 'png dimensions are invalid')
  return width, height
end

function M.authored_items(root)
  local chunk, err = loadfile(path.join(root, 'scripts/lua_data/templates/moba_items.lua'))
  assert(chunk, err)
  local builder = assert(chunk(), 'item catalog did not return a builder')
  assert(type(builder) == 'function', 'item catalog builder is invalid')
  local items = assert(builder({}), 'item catalog builder returned nothing')
  assert(type(items) == 'table' and #items > 0 and #items <= 64, 'authored item list is empty or too large')
  return items
end

function M.validate(options)
  assert(type(options) == 'table', 'validate options are required')
  local root = assert(options.root, 'repository root is required')
  local manifest_path = path.absolute(assert(options.manifest, 'manifest path is required'), root)
  assert(not escapes(manifest_path), 'manifest path cannot contain a parent segment')
  local manifest = json.read(manifest_path)
  assert(manifest.schema == 'omoba.item-icon-assets.v1', 'unsupported item icon manifest schema')
  assert(manifest.tool == 'builtin-image_gen', 'item icon manifest tool is not the original generator')
  assert(type(manifest.status) == 'string' and manifest.status ~= '', 'item icon manifest status is missing')
  assert(type(manifest.assets) == 'table' and #manifest.assets > 0 and #manifest.assets <= 64, 'item icon asset list is invalid')
  local candidate_dir = path.absolute(options.candidate_dir or path.parent(manifest_path), root)
  assert(M.inside_project(root, candidate_dir), 'candidate png directory is outside the project: ' .. candidate_dir)
  local authored = M.authored_items(root)
  local by_catalog, by_id = {}, {}
  for _, item in ipairs(authored) do
    assert(whole(item.catalog_id, 1, 64), 'authored catalog id is out of bounds')
    assert(identifier(item.id), 'authored item id is invalid')
    assert(not by_catalog[item.catalog_id] and not by_id[item.id], 'authored item identity is duplicated')
    assert(item.charges == nil, 'authored items have no charge contract')
    by_catalog[item.catalog_id] = item
    by_id[item.id] = item
  end
  assert(#manifest.assets == #authored, 'manifest assets must cover the authored catalog exactly')
  local seen_catalog, seen_id, seen_dest, assets = {}, {}, {}, {}
  for _, asset in ipairs(manifest.assets) do
    assert(type(asset) == 'table', 'manifest asset is not an object')
    assert(whole(asset.catalog_id, 1, 64), 'manifest catalog id is out of bounds')
    assert(identifier(asset.item_id), 'manifest item id is invalid')
    assert(asset.file == asset.item_id .. '.png', 'manifest file must be the flat item png')
    assert(not asset.file:find('[/\\]') and not asset.file:find('%.%.'), 'manifest file must stay inside the candidate directory')
    assert(asset.width == EXPECTED_EDGE and asset.height == EXPECTED_EDGE, 'manifest dimensions must be the actual 1254 source')
    assert(whole(asset.bytes, 1, 16 * 1024 * 1024), 'manifest byte size is invalid')
    assert(type(asset.sha256) == 'string' and asset.sha256:match('^[0-9a-f]+$') and #asset.sha256 == 64, 'manifest sha256 is invalid')
    assert(type(asset.prompt) == 'string' and asset.prompt ~= '', 'manifest prompt provenance is missing')
    assert(type(asset.visual_review) == 'string' and asset.visual_review ~= '', 'manifest visual review is missing')
    assert(type(asset.source) == 'string' and asset.source ~= '', 'manifest source provenance is missing')
    local item = by_catalog[asset.catalog_id]
    assert(item and item.id == asset.item_id, 'manifest item does not match the authored catalog id')
    assert(not seen_catalog[asset.catalog_id], 'duplicate manifest catalog id')
    assert(not seen_id[asset.item_id], 'duplicate manifest item id')
    seen_catalog[asset.catalog_id] = true
    seen_id[asset.item_id] = true
    local source_png = path.join(candidate_dir, asset.file)
    assert(M.inside_project(root, source_png), 'candidate png is outside the project: ' .. source_png)
    assert(path.is_file(source_png), 'candidate png is missing: ' .. source_png)
    local data = path.read(source_png, true)
    assert(#data == asset.bytes, 'candidate png byte size does not match the manifest')
    local width, height = M.png_size(data)
    assert(width == EXPECTED_EDGE and height == EXPECTED_EDGE, 'candidate png is not 1254x1254')
    local digest = hash.sha256(source_png)
    assert(digest == asset.sha256, 'candidate png sha256 does not match the manifest')
    local object_path = '/Game/Omoba/UI/Items/' .. asset.item_id .. '.' .. asset.item_id
    assert(not seen_dest[object_path], 'duplicate item icon destination')
    seen_dest[object_path] = true
    assets[#assets + 1] = {
      item_id = asset.item_id, catalog_id = asset.catalog_id, source_png = path.absolute(source_png),
      sha256 = digest, width = width, height = height, bytes = #data,
      object_path = object_path, destination_path = '/Game/Omoba/UI/Items',
    }
  end
  for catalog_id in pairs(by_catalog) do
    assert(seen_catalog[catalog_id], 'authored catalog item has no icon: ' .. catalog_id)
  end
  table.sort(assets, function(a, b) return a.catalog_id < b.catalog_id end)
  return {
    schema = 'omoba.item-icon-import-plan.v1', assets = assets,
    source_manifest = manifest_path, root = root,
  }
end

local function destination_state(dest, data, label)
  if not path.is_file(dest) then return 'new' end
  assert(path.read(dest, true) == data, 'existing ' .. label .. ' differs and was not overwritten: ' .. dest)
  return 'present'
end

function M.stage(plan, stage_dir)
  assert(plan and plan.schema == 'omoba.item-icon-import-plan.v1', 'stage requires a validated plan')
  assert(type(plan.root) == 'string' and plan.root ~= '', 'stage requires the project root')
  assert(type(plan.source_manifest) == 'string' and plan.source_manifest ~= '', 'stage requires the source manifest')
  assert(not escapes(plan.source_manifest), 'source manifest path cannot contain a parent segment')
  stage_dir = path.absolute(stage_dir)
  assert(M.inside_project(plan.root, stage_dir), 'stage directory is outside the project: ' .. stage_dir)
  local manifest_bytes = path.read(plan.source_manifest, true)
  local manifest_dest = path.join(stage_dir, 'manifest.json')
  local prepared = {}
  for _, asset in ipairs(plan.assets) do
    assert(M.inside_project(plan.root, asset.source_png), 'staged source png is outside the project: ' .. tostring(asset.source_png))
    local data = path.read(asset.source_png, true)
    assert(#data == asset.bytes, 'staged source byte size changed: ' .. asset.item_id)
    assert(hash.sha256(asset.source_png) == asset.sha256, 'staged source hash changed: ' .. asset.item_id)
    prepared[#prepared + 1] = {asset = asset, data = data, dest = path.join(stage_dir, asset.item_id .. '.png')}
  end
  local manifest_state = destination_state(manifest_dest, manifest_bytes, 'source manifest')
  for _, item in ipairs(prepared) do
    item.state = destination_state(item.dest, item.data, 'staged item icon')
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
    assets[index] = {
      item_id = asset.item_id, catalog_id = asset.catalog_id, png = asset.staged_png,
      sha256 = asset.sha256, width = asset.width, height = asset.height, bytes = asset.bytes,
      object_path = asset.object_path, destination_path = asset.destination_path,
    }
  end
  return {schema = plan.schema, assets = assets}
end

return M
