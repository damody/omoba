-- Independently re-read the retained screenshots and restored source binary.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = source:match('^(.*)[/\\]tests[/\\]') .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, hash = b.lib('path'), b.lib('json'), b.lib('hash')
assert(#arg == 1, 'usage: ue_art_swap_acceptance.lua RUN_DIRECTORY')
local work = path.absolute(arg[1], b.root)
local raw = json.read(path.join(work, 'report.json'))
assert(raw.work == work and raw.success and raw.source_restored and raw.code_and_blueprints_unchanged, 'art replacement did not complete/restore')
assert(raw.source == path.join(b.root, 'scripts/lua_data/templates/heroes/saika_magoichi/saika_magoichi_mat.png'), 'unexpected source')
assert(hash.sha256(raw.source) == raw.original_sha256 and hash.sha256(path.join(work, 'original.png')) == raw.original_sha256, 'source differs from saved original')
assert(hash.sha256(raw.replacement) == raw.replacement_sha256 and raw.original_sha256 ~= raw.replacement_sha256, 'replacement source mismatch')
local function dimensions(file)
  local bytes = path.read(file, true)
  assert(bytes:sub(1, 8) == '\137PNG\r\n\26\n' and bytes:sub(13, 16) == 'IHDR', 'invalid PNG')
  return string.unpack('>I4I4', bytes, 17)
end
local report = {success = false, run = work, original_sha256 = raw.original_sha256,
  replacement_sha256 = raw.replacement_sha256, source_restored = true,
  code_and_blueprints_unchanged = raw.code_and_blueprints_unchanged, phases = {}}
for _, label in ipairs({'baseline', 'replacement', 'restored'}) do
  local phase = assert(raw.phases[label])
  local w, h = dimensions(label == 'replacement' and raw.replacement or raw.source)
  assert(phase.texture.width == w and phase.texture.height == h and phase.texture.srgb, 'texture dimensions not independently verified')
  assert(phase.pie.success and phase.pie.native_mesh_rendered and phase.pie.native_expected_scale > 0, 'missing actual PIE rendering/configuration')
  local idem = phase.idempotence
  assert(idem.success and idem.package_count > 0 and idem.second.imported == 0
    and idem.second.material_bindings_changed == 0 and idem.second.skipped == #idem.second.jobs, 'non-idempotent recipe')
  assert(phase.screenshot == path.join(work, label .. '.png') and hash.sha256(phase.screenshot) == phase.screenshot_sha256, 'screenshot missing/modified')
  local sw, sh = dimensions(phase.screenshot)
  assert(sw > 0 and sh > 0, 'empty screenshot')
  report.phases[label] = {texture_width = w, texture_height = h, native_scale = phase.pie.native_expected_scale,
    native_mesh_rendered = true, imported = idem.first.imported, repeat_imported = 0,
    verified_jobs = #idem.second.jobs, unchanged_packages = idem.package_count,
    screenshot = phase.screenshot, screenshot_sha256 = phase.screenshot_sha256, width = sw, height = sh}
end
report.success = true
local output = path.join(b.root, 'openspec/changes/build-unreal-rust-moba-framework/evidence/art-replacement', assert(work:match('([^/\\]+)$')) .. '.json')
json.write(output, report, true)
print('saved art replacement acceptance: PASS; ' .. output)
