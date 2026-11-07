-- Can the Lua-side content loader read every template (E159)? The build reads scripts/lua_data
-- twice: om_codegen (Rust, mlua) and the Lua build steps that use scripts/content_builder.lua
-- (sync_ue_hud_manifest_labels, gen_hero_registry, ue_hud_portrait_coverage, hero_author). The
-- two loaders must accept the same templates; content_builder also requires every builder,
-- including a nested include, to return a table.
--
--   tools\lua\lua.exe scripts\lua_content_check.lua [CONTENT_ROOT]
--
-- Loads, through content_builder, every template templates.lua includes (and with them all their
-- nested includes). land_to_live.lua and check_live_consistent.lua run it on the candidate tree.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local M = {}

-- The templates templates.lua includes, in order (a builder may include more).
function M.templates(root_text)
  local list, seen = {}, {}
  for relative in root_text:gmatch('ctx%.include%(%s*["\']([^"\']+)["\']%s*%)') do
    if not relative:match('include_cycle_') and not seen[relative] then seen[relative] = true; list[#list + 1] = relative end
  end
  return list
end

-- Errors ({file, message}) of loading each template with content_builder over `content_root`.
function M.check(content_root)
  local handle = assert(io.open(content_root .. '/templates.lua', 'rb'))
  local text = handle:read('a')
  handle:close()
  local failures = {}
  for _, relative in ipairs(M.templates(text)) do
    -- A fresh context per template: one failure does not hide the next.
    local ok, err = pcall(require('content_builder').new(content_root).include, relative)
    if not ok then failures[#failures + 1] = { file = relative, message = tostring(err) } end
  end
  return failures, #M.templates(text)
end

function M.main(argv)
  local b = require('_bootstrap')
  local root = argv[1] or b.lib('path').join(b.root, 'scripts', 'lua_data')
  local failures, total = M.check(root)
  for _, failure in ipairs(failures) do io.stderr:write(('content_builder cannot load %s:\n%s\n'):format(failure.file, failure.message)) end
  print(('lua content loaders: %d template(s), %d failure(s)'):format(total, #failures))
  if #failures > 0 then os.exit(1) end
end

if not pcall(debug.getlocal, 4, 1) then M.main(arg) end
return M
